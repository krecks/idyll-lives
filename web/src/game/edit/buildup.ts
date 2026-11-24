/**
 * Step 2's build-up: a street built up with the town's other buildings (real lots, as the generator
 * lays them: shops, cafés, offices and workshops, buildings of flats, terraces and homes on tight
 * gardens), those buildings cleared again, and a civic place put down by hand (a lot of its own:
 * `lots.ts` `addLot`). Each is a pure function from draft to draft, laid by the generator's own dense
 * pass (`dense.ts` `DensePass`, made from the draft: everything standing is kept clear of, nothing
 * standing moves) and its own lot fill (`lots.ts` `fillStreet`). A new named place's lot is called as
 * its place is and given its place, flats go over most shops, cafés and restaurants in a centre and
 * real floors over offices (`dense.ts` `raisePlaces`), and a building of flats is called as its
 * template names it.
 *
 * A street is built up as the generator builds one, its homes first and the buildings in what they
 * leave, the zone set stretch by stretch: inside the town centre's outline the centre's kinds
 * shoulder to shoulder and no homes of the lot mix; elsewhere in the main town homes on a share of
 * the frontage (the denser the town, the smaller the share), then rows of the town's kinds in the
 * gaps; in a village homes at the lot mix's gaps, then rows of the hamlet's kinds with its gaps. A
 * country road out of every town and village is not built up.
 *
 * The homes and the buildings draw from streams of their own, keyed by the op's seed and the road
 * (its ends and kind, so its index may change), so the homes stay the same whichever is asked for.
 * A building whose ground would break a slope limit is left out and counted (decision 32: it is
 * filler, not the player's own placement): the land under it is judged here, cheaply, as the
 * settle's surface would grade it (`builtSurface.ts`), with a margin, since the settle itself runs
 * later, in the worker.
 */
import type { Content } from '../../content/content';
import type { RoadPlan } from '../../core/protocol';
import { isSeparated, kindInfo } from '../../core/roadKinds';
import { CUT_MAX, ENTRY_MAX, RETAIN_MAX } from '../../render/shared/builtSurface';
import { CIVIC, civicChoices, civicLabel, civicTooBig, draftAreaSize, ZONE } from '../places';
import { DensePass, raisePlaces } from '../dense';
import { heightAt, LAND_CELL, type Land } from '../land';
import { mulberry32 } from '../noise';
import { lengths, pointAt, RoadIndex } from '../roads';
import { plotRect, type NeighbourhoodDraft, type PlotSlot, type Rect, type Templates } from '../town';
import { placeOfTemplate } from '../placeLots';
import { placeOf, specOf } from '../world';
import type { Edited } from './edited';
import { entriesOf, frontOf, inCentre } from './ground';
import { addLot, fillStreet, removeLotsEdit, renumber, withLotPlace, withPlaceName } from './lots';

/** What goes along a street built up: homes from the lot mix, the town's other buildings, or both. */
export type BuildMode = 'homes' | 'buildings' | 'both';

/** A build-up asked for: the road, what goes along it, and the seed its homes and buildings are drawn from. */
export interface BuildUpOp {
  road: number;
  mode: BuildMode;
  seed: number;
}

/** A civic place put down by hand: which (an index into `CIVIC`), along which road, how far along it (m) and on which side, and the seed its look and name are drawn from. */
export interface CivicOp {
  civic: number;
  road: number;
  s: number;
  side: 1 | -1;
  seed: number;
}

/** A country road counts as an area's where this share of it runs inside the area's box. */
const INSIDE_SHARE = 0.5;
/** A building along a country road stands no further outside its area's box than this (m). */
const NEAR_BOX = 10;
/** The share of a main town's frontage its homes take: this at a hamlet-sized town's density, falling to `SHARE_DENSE` at a city's. */
const SHARE_SPARSE = 0.85;
const SHARE_DENSE = 0.6;
const DENSITY_SPARSE = 0.25;
const DENSITY_DENSE = 0.85;
/** A road's height is the land along it averaged this far either way (m), as `builtSurface.ts` starts it. */
const PROFILE_REACH = 12;
/** How far within the slope limits a building is kept (m): the settle grades the land, this only judges it. */
const SLOPE_MARGIN = 0.5;
/** Two pads touching may differ by this much (m) at most and still be brought within `RETAIN_MAX` of each other: each moves `ENTRY_MAX` off its road. */
const PADS_MEET = 2 * ENTRY_MAX + RETAIN_MAX - 0.05;

// ---- streams ----------------------------------------------------------------------------------------

/** A 32-bit hash's last mixing step (MurmurHash3's). */
function scramble(h: number): number {
  h ^= h >>> 16;
  h = Math.imul(h, 0x85ebca6b);
  h ^= h >>> 13;
  h = Math.imul(h, 0xc2b2ae35);
  h ^= h >>> 16;
  return h >>> 0;
}

/** One 32-bit seed from a seed and words (FNV-1a over them, scrambled). */
function mix(seed: number, ...words: string[]): number {
  let h = 0x811c9dc5 ^ (seed >>> 0);
  for (const ch of words.join('|')) h = Math.imul(h ^ ch.charCodeAt(0), 0x01000193);
  return scramble(h >>> 0);
}

/** A road's identity across edits, as the settle's marks know one: where it begins and ends (to the metre) and its kind. Its index changes as roads go. */
export function roadKey(r: RoadPlan): string {
  const [a, b] = [r.points[0], r.points[r.points.length - 1]];
  return `${Math.round(a[0])},${Math.round(a[1])} ${Math.round(b[0])},${Math.round(b[1])} ${r.kind ?? 'street'}`;
}

// ---- where a road is ------------------------------------------------------------------------------

/**
 * The area road `road` of `draft` is built up as: its own (a street or a lane of a town or a
 * village), else, for a country road, the area whose box it mostly runs inside; -1 if none (a
 * country road out of every town and village, or no such road).
 */
export function areaOfRoad(draft: NeighbourhoodDraft, road: number): number {
  const r = draft.roads[road];
  if (!r) return -1;
  if (r.area !== undefined) return r.area;
  const areas = draft.open?.areas ?? [];
  const ls = lengths(r.points);
  const L = ls[ls.length - 1];
  const n = Math.max(2, Math.ceil(L / 4));
  const inBox = areas.map(() => 0);
  for (let k = 0; k <= n; k++) {
    const { p } = pointAt(r.points, (L * k) / n, ls);
    areas.forEach((a, i) => {
      if (p[0] >= a.x && p[0] <= a.x + a.w && p[1] >= a.z && p[1] <= a.z + a.d) inBox[i]++;
    });
  }
  const best = inBox.reduce((b, c, i) => (c > inBox[b] ? i : b), 0);
  return areas.length && inBox[best] >= (n + 1) * INSIDE_SHARE ? best : -1;
}

/** Why nothing may be built along road `road` of `draft` at all (a footpath, a bridge, the motorway, a road out of every town and village), or null. */
export function notBuilt(draft: NeighbourhoodDraft, road: number): string | null {
  const r = draft.roads[road];
  if (!r) return 'there is no road there';
  const name = r.name ?? 'it';
  if (r.kind === 'trail') return 'nothing is built along a footpath';
  if (isSeparated(r.kind)) return 'nothing is built along the motorway';
  if (r.bridge) return 'nothing is built on a bridge';
  if (areaOfRoad(draft, road) < 0) return `${name} is ${kindInfo(r).article} out of the town and its villages: nothing is built along it`;
  return null;
}

/** The share of the main town's frontage its homes take, by how dense the town is (`WorldSpec.density`). */
function homeShare(draft: NeighbourhoodDraft): number {
  const { density } = specOf(placeOf(draft.world?.kind ?? 'country'), draft.world?.options);
  const k = Math.min(1, Math.max(0, (density - DENSITY_SPARSE) / (DENSITY_DENSE - DENSITY_SPARSE)));
  return SHARE_SPARSE + (SHARE_DENSE - SHARE_SPARSE) * k;
}

// ---- the slope limits, judged ------------------------------------------------------------------

/**
 * The slope limits judged on the land as laid (`land`), building by building, as the settle's
 * surface would grade it: a road's height is the land along it averaged `PROFILE_REACH` m either
 * way; a pad stands within `ENTRY_MAX` of its road where it meets it (the road nearest it), as near
 * the middle of the land under it as it can; it is cut into the land or stands over it by no more
 * than `CUT_MAX`, and two pads touching are no further apart than their roads let them be brought
 * within `RETAIN_MAX` of each other. Kept `SLOPE_MARGIN` within both.
 */
class SlopeJudge {
  private readonly index: RoadIndex;
  private readonly lens: number[][];
  /** The pads standing (the lots') and those kept so far: their boxes and their roads' heights. */
  private readonly pads: { box: Rect; entry: number }[] = [];

  constructor(
    private readonly land: Land,
    private readonly roads: readonly RoadPlan[],
    standing: readonly Rect[],
  ) {
    this.index = new RoadIndex(roads);
    this.lens = roads.map((r) => lengths(r.points));
    for (const box of standing) {
      const entry = this.entryOf(box);
      if (entry !== null) this.pads.push({ box, entry });
    }
  }

  /** The height of the road nearest a box's middle (not a trail) where it meets it, if one is near. */
  private entryOf(box: Rect): number | null {
    const [cx, cz] = [box.x + box.w / 2, box.z + box.d / 2];
    let best: { road: number; s: number; dist: number } | null = null;
    for (const [road, m] of this.index.around(cx, cz, 40)) {
      if (this.roads[road].kind === 'trail' || (best && best.dist <= m.dist)) continue;
      best = { road, s: m.s, dist: m.dist };
    }
    if (!best) return null;
    const { points } = this.roads[best.road];
    const ls = this.lens[best.road];
    const L = ls[ls.length - 1];
    let [sum, n] = [0, 0];
    for (let s = Math.max(0, best.s - PROFILE_REACH); s <= Math.min(L, best.s + PROFILE_REACH); s += LAND_CELL) {
      const { p } = pointAt(points, s, ls);
      sum += heightAt(this.land, p[0], p[1]);
      n++;
    }
    return n ? sum / n : null;
  }

  /** Whether a lot on `box` keeps within the limits; kept as standing if it does. */
  keep(box: Rect): boolean {
    const entry = this.entryOf(box);
    if (entry === null) return true;
    let [lo, hi] = [Infinity, -Infinity];
    for (let j = Math.ceil(box.z / LAND_CELL); j <= Math.floor((box.z + box.d) / LAND_CELL); j++) {
      for (let i = Math.ceil(box.x / LAND_CELL); i <= Math.floor((box.x + box.w) / LAND_CELL); i++) {
        const h = heightAt(this.land, i * LAND_CELL, j * LAND_CELL);
        [lo, hi] = [Math.min(lo, h), Math.max(hi, h)];
      }
    }
    if (!Number.isFinite(lo)) [lo, hi] = [heightAt(this.land, box.x + box.w / 2, box.z + box.d / 2), heightAt(this.land, box.x + box.w / 2, box.z + box.d / 2)];
    // (The least cut and fill: midway between the land's lowest and highest under it, within reach of its road.)
    const pad = Math.min(entry + ENTRY_MAX, Math.max(entry - ENTRY_MAX, (lo + hi) / 2));
    if (Math.max(pad - lo, hi - pad) > CUT_MAX - SLOPE_MARGIN) return false;
    const g = LAND_CELL;
    for (const q of this.pads) {
      const b = q.box;
      const touches = box.x - g <= b.x + b.w && b.x - g <= box.x + box.w && box.z - g <= b.z + b.d && b.z - g <= box.z + box.d;
      if (touches && Math.abs(q.entry - entry) > PADS_MEET - SLOPE_MARGIN) return false;
    }
    this.pads.push({ box, entry });
    return true;
  }
}

/** The land a draft's new lots are judged on: its plan's land as laid (step 2), else none to judge (a classic town: nothing is left out). */
function landOf(draft: NeighbourhoodDraft): Land | null {
  return draft.open?.land ?? null;
}

// ---- the edits --------------------------------------------------------------------------------

/**
 * `draft` with road `op.road` built up (see the top of this file): homes from the lot mix
 * (`op.mode` `homes`, `both`: `fillStreet`, a share of the frontage in the main town outside its
 * centre for `both`), the town's other buildings in what they leave (`buildings`, `both`: rows of
 * lots along it, `DensePass.rowsAlong`). `leftOut`: how many buildings were left out where the ground
 * would break a slope limit. Refused where nothing is built along the road, or nothing more fits along it.
 */
export function buildUp(draft: NeighbourhoodDraft, t: Templates, content: Content, op: BuildUpOp): Edited & { leftOut?: number } {
  const no = notBuilt(draft, op.road);
  if (no) return { refused: no };
  const r = draft.roads[op.road];
  const name = r.name ?? 'it';
  const key = roadKey(r);
  const area = areaOfRoad(draft, op.road);

  // ---- the homes: the lot mix at its gaps; in the main town, a share of the frontage when buildings go along too ----
  let next = draft;
  let homes = 0;
  if (op.mode !== 'buildings') {
    const filled = fillStreet(draft, t, content, op.road, mix(op.seed, key, 'lots'));
    if ('draft' in filled) {
      // (None whose front is in the town centre, past the fill's own test of the land 10 m in from the road.)
      const added = filled.draft.slots.slice(draft.slots.length).filter((s) => !inCentre(draft, frontOf(t, s)));
      const most = op.mode === 'both' && area === 0 ? Math.round(added.length * homeShare(draft)) : added.length;
      // (The first homes the fill put down, nearest the town's middle: the same whichever mode, so they stay when it changes.)
      const slots = added.slice(0, most).map((s, k) => ({ ...s, index: draft.slots.length + k }));
      next = renumber({ ...draft, slots: [...draft.slots, ...slots] }, t, [op.road]);
      homes = most;
    }
    if (op.mode === 'homes') return homes ? { draft: next, cleared: { lots: [] } } : { refused: `nothing more fits along ${name}` };
  }

  // ---- the buildings: rows of lots along it in what the homes left (`dense.ts`) ----
  const rand = mulberry32(mix(op.seed, key, 'rows'));
  const pass = new DensePass({ draft: next, t, content });
  const from = pass.count;
  const sides: (1 | -1)[] = r.lots ? [r.lots] : [1, -1];
  pass.rowsAlong(area, op.road, sides, rand);
  const { slots } = pass.result();
  // Only those near its area; and the ground judged: a lot that would break a slope limit is left out (decision 32).
  const box = next.open?.areas[area];
  const near = (s: PlotSlot) => {
    if (r.area !== undefined || !box) return true;
    const c = plotRect(t, s);
    const [x, z] = [c.x + c.w / 2, c.z + c.d / 2];
    return x >= box.x - NEAR_BOX && z >= box.z - NEAR_BOX && x <= box.x + box.w + NEAR_BOX && z <= box.z + box.d + NEAR_BOX;
  };
  const land = landOf(next);
  const judge = land ? new SlopeJudge(land, next.roads, next.slots.map((s) => plotRect(t, s))) : null;
  const fresh = slots.slice(from).filter(near);
  const kept = judge ? fresh.filter((s) => judge.keep(plotRect(t, s))) : fresh;
  const leftOut = fresh.length - kept.length;
  if (!homes && !kept.length) return { refused: leftOut ? `the ground along ${name} is too steep for anything more to be built there` : `nothing more fits along ${name}` };
  // (Numbered on from the lots standing.)
  const added = kept.map((s, k): PlotSlot => ({ ...s, index: from + k }));
  const built = { ...next, slots: [...slots.slice(0, from), ...added] };
  return { draft: renumber(withBuildings(built, t, content, from, mix(op.seed, key, 'places')), t, [op.road]), cleared: { lots: [] }, ...(leftOut ? { leftOut } : {}) };
}

/**
 * `draft` with its lots from slot `from` on (laid by the dense pass: `dense.ts`) made what the generator makes
 * of them: flats over a shop, a café or a restaurant and real floors over an office (`raisePlaces`), a
 * named place's lot called as its place is (none
 * twice: `withPlaceName`), and every public lot (a park, an outdoor spot, a shop) given its place
 * (`withLotPlace`). Drawn from `seed`.
 */
export function withBuildings(draft: NeighbourhoodDraft, t: Templates, content: Content, from: number, seed: number): NeighbourhoodDraft {
  const rand = mulberry32(seed);
  const fresh = draft.slots.slice(from).map((s) => ({ ...s }));
  raisePlaces(t, fresh, (draft.open?.centre ?? []).map((c) => c.poly), rand);
  let next: NeighbourhoodDraft = { ...draft, slots: [...draft.slots.slice(0, from), ...fresh] };
  for (const s of fresh) {
    if (s.kind === 'house' || s.kind === 'flats') continue;
    next = withPlaceName(next, content, s.index, rand);
    next = withLotPlace(next, t, content, s.index);
  }
  return next;
}

/**
 * Whether lot `s` is one of the town's buildings the build-up lays (`dense.ts`) rather than a home of the
 * lot mix, a farm or a place put down on its own: a building of flats, a terraced home, a home on a tight
 * garden, a named place of a compact layout (`places.ts` `ZONE`'s `compact`).
 */
export function isTownBuilding(s: Pick<PlotSlot, 'kind' | 'lot' | 'garden' | 'template'>): boolean {
  if (s.kind === 'flats') return true;
  if (s.kind === 'house') return s.lot === 'terrace' || s.garden === 'tight';
  const what = s.kind === 'shop' ? placeOfTemplate(s.template) : null;
  return !!what && 'zone' in what && ZONE[what.zone].compact.includes(s.template);
}

/** The town's buildings (`isTownBuilding`) fronting road `road` (where each meets its road: `ground.ts` `entriesOf`), by slot index. */
export function buildingsOn(draft: NeighbourhoodDraft, t: Templates, road: number): number[] {
  if (!draft.roads[road]) return [];
  const spots = entriesOf(draft, t);
  return draft.slots.flatMap((s, i) => (spots[i].road === road && isTownBuilding(s) ? [s.index] : []));
}

/**
 * `draft` without the town's buildings fronting road `road` (`buildingsOn`), the flats in them and the places
 * of those that are places with them; the homes of the lot mix and the places put down on their own stay.
 * Refused if none fronts it, or where the town would lose its last home (`lots.ts` `removeLotsEdit`).
 */
export function clearBuildings(draft: NeighbourhoodDraft, t: Templates, road: number): Edited {
  const r = draft.roads[road];
  if (!r) return { refused: 'there is no road there' };
  const on = buildingsOn(draft, t, road);
  if (!on.length) return { refused: `nothing is built along ${r.name ?? 'it'}` };
  return removeLotsEdit(draft, new Set(on));
}

/** Civic places set by hand last: per draft, the edit each op made (the Lot tool asks for the same on every pointer move). */
const civicMade = new WeakMap<NeighbourhoodDraft, Map<string, Edited>>();

/**
 * `draft` with civic place `op.civic` (an index into `CIVIC`) put down by hand fronting road
 * `op.road` at `op.s` along it (to the metre) on `op.side`: a lot of its own, built from its layout
 * for its area's size (a village's hall the parish hall's, a town's the town hall's: `placeLots.ts`
 * `placeUnit`), put down as `lots.ts` `addLot` puts a named place's (its front by the road, on free
 * ground), its look drawn from `op.seed`, named as one and given its place. Only one its area lacks
 * (`civicChoices`), on a road of a town or a village. Kept per draft, so asking again on the same
 * draft is cheap. The settle judges its slopes, as any placement's.
 */
export function addCivic(draft: NeighbourhoodDraft, t: Templates, content: Content, op: CivicOp): Edited {
  const s = Math.round(op.s);
  const memo = `${op.civic}|${op.road}|${s}|${op.side}|${op.seed}`;
  let made = civicMade.get(draft);
  const had = made?.get(memo);
  if (had) return had;
  const out = civicAt(draft, t, content, { ...op, s });
  if (!made) civicMade.set(draft, (made = new Map()));
  if (made.size > 64) made.clear();
  made.set(memo, out);
  return out;
}

/** `addCivic`, worked out. */
function civicAt(draft: NeighbourhoodDraft, t: Templates, content: Content, op: CivicOp): Edited {
  const c = CIVIC[op.civic];
  if (!c) return { refused: 'there is no such civic place' };
  const no = notBuilt(draft, op.road);
  if (no) return { refused: no.replace('nothing is built', 'no civic place goes') };
  const r = draft.roads[op.road];
  if (r.lots && r.lots !== op.side) return { refused: 'that side of the road looks out over the water' };
  const area = areaOfRoad(draft, op.road);
  const size = draftAreaSize(draft, area);
  const label = civicLabel(c, size).toLowerCase();
  const where = draft.open?.areas[area]?.name ?? draft.name;
  if (civicTooBig(c, size)) return { refused: `${where} is too small for a ${label}` };
  if (!civicChoices(draft, area).some((k) => k.index === op.civic)) return { refused: `${where} has a ${label} already` };
  const e = addLot(draft, t, content, { road: op.road, s: op.s, side: op.side }, { kind: 'place', what: { civic: op.civic }, area }, mix(op.seed, roadKey(r), 'civic', String(op.civic)));
  if ('refused' in e) return { refused: e.refused === 'There is no room for it here.' ? `there is no room for the ${label} here` : e.refused };
  return e;
}

/** The civic places area `area` of `draft` may be given by hand (`civicChoices`): what each is called, by its index into `CIVIC`. */
export function civicOffered(draft: NeighbourhoodDraft, area: number): { civic: number; label: string }[] {
  return area < 0 ? [] : civicChoices(draft, area).map((c) => ({ civic: c.index, label: c.name }));
}

/**
 * A street just drawn (`drawn`, made by `streets.ts` `drawStreet` on `before`, with no homes of its
 * own) built up as `mode` asks: each road it made (the pieces either side of its bridges; not a
 * deck, nor a country road out of every town and village) built up by `buildUp`, its seed drawn from
 * `seed`. One edit: what the street cleared out of its way stays in `cleared`, and `leftOut` counts
 * the buildings left out along all of it. A piece nothing fits along is left as drawn.
 */
export function builtAlong(before: NeighbourhoodDraft, drawn: Edited, t: Templates, content: Content, mode: BuildMode, seed: number): Edited & { leftOut?: number } {
  if ('refused' in drawn) return drawn;
  const had = new Set(before.roads);
  // (Its own pieces, not the footpaths it cut: `Edited.made`.)
  const fresh = drawn.made ?? drawn.draft.roads.flatMap((r, i) => (had.has(r) ? [] : [i]));
  let draft = drawn.draft;
  let leftOut = 0;
  for (const [k, road] of fresh.entries()) {
    const e = buildUp(draft, t, content, { road, mode, seed: (seed + k * 0x9e3779b1) >>> 0 });
    if ('refused' in e) continue;
    draft = e.draft;
    leftOut += e.leftOut ?? 0;
  }
  return { draft, cleared: drawn.cleared, made: fresh, ...(leftOut ? { leftOut } : {}) };
}
