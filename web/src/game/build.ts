/**
 * Step 3's Build (the `build` job): the town made from step 2's layout, its streets and the uses the
 * player painted (`uses.ts`), on the land.
 *
 * - A layout untouched (its `layoutHash` the proposal's) with the proposal's own seed (`PROPOSAL_SEED`)
 *   builds the generator's town as it is (`Proposal.built`): today's town and households.
 * - Else the town is built again where the layout changed: what stands on the changed cells (and 20 m
 *   round them) or fronts a road that is gone is cleared, and the frontage there is filled again road by
 *   road, side by side, stretch by stretch (`proposal.ts` `stretchesOf`: the use most of the land 6 to
 *   18 m behind the kerb is painted), each stretch from a stream of its own (the seed, the road, the use,
 *   the side and where the stretch begins), so a stretch the player didn't touch is filled the same way
 *   whatever else changed. With another seed ("Build again"), everything zoned is built again from it.
 *
 * What goes along a stretch, by its use:
 * - homes: houses of the place's lot mix at its gaps (`LotPlacer`, as `fillStreet` fills a street);
 * - terraces and flats: rows of terraced houses on about half of it (`terraceRows`), then the backdrop's
 *   houses, terraces and flats in what they leave (`BackdropPass.rowsAlong`), a row behind (`secondRow`);
 * - shops and work: the backdrop's shops, cafés, restaurants, offices and workshops (the centre's inside
 *   the town centre), a row behind, and a shop lot for an area that has none;
 * - parks and green: a park, then a playground; a green with no frontage near a town or village becomes
 *   an outdoor spot, a trail to it from the area's streets (`trailTo`);
 * - public places: the civic places the area lacks, the most needed first (`civicChoices`, its size as
 *   its homes now make it), each nearest the area's middle (`BackdropPass.putCivic`).
 * A lot or a building goes only where `SHARE` of the cells under it are painted its use and none is a
 * green; unpainted frontage stays empty, and so does land zoned more than `BUILD_REACH` m from a road.
 * The areas are those the layout has, a street drawn by hand given to the nearest within 30 m (its box
 * grown to it) or a village of its own (`areasFor`). Each changed area's parking is laid again, the
 * houses numbered along their streets, and the places named. The land is settled under it all, giving up
 * only lots and backdrop buildings (`settleLand`'s `keepRoads`: the streets were judged in step 2).
 */

import type { Content } from '../content/content';
import type { BackdropUse, RoadPlan, TownArea } from '../core/protocol';
import { BackdropPass, civicChoices, facesRoad, type Row } from './backdrop';
import { roadKey } from './edit/buildup';
import { frontOf, inCentre, townGround } from './edit/ground';
import { removeLots, renumber, withLotPlace } from './edit/lots';
import { roadReach, usesOfDraft } from './edit/uses';
import { encodeLand, heightAt } from './land';
import { openUnderBuilt, settleLand } from './landFollow';
import { unrle } from './landMasks';
import { AREA_APART, BEND, BEND_MOST, cells8, cornersOf, grown, hamletOf, parkingFor, rowHouse, trailTo, type AreaPlan } from './macro';
import { mulberry32 } from './noise';
import { fronted, greenSpots, hatched, SPOT_NEAR, SPOT_REACH, stretchesOf, type Stretch } from './proposal';
import { inside, lengths, leftOf, pointAt, RoadIndex, straightAround, type Pt } from './roads';
import { boundsOf, frontageOf, generatedHouses, Ground, lotAt, LotPlacer, plotRect, slotDims, wantedLots, type Facing, type Frontage, type NeighbourhoodDraft, type PlotSlot, type PublicTemplate, type Rect, type Templates, type Unplaced } from './town';
import { previewWorld } from './townPreview';
import type { LaidTown, Proposal } from './townJobs';
import { layoutHash, unpackUses, USE_CELL, USE_CODE, type UseKind } from './uses';
import { placeOf, specOf, type SizeSpec, type WorldSpec } from './world';

type Rand = () => number;

/** The build seed of the proposal's own town: an untouched layout built with it is the generator's town. "Build again" draws another. */
export const PROPOSAL_SEED = 0;

/** A lot or a building goes where this share of the cells under it at least are painted its use (and none a green, but a green's). */
export const SHARE = 0.6;
/** What stands within this many cells (20 m) of a changed cell is built again. */
const DIRTY_GROW = 5;
/** A street drawn by hand this near an area's box (m) is one of its streets; a stretch this near grows it. */
const NEAR_AREA = 30;
/** A painted cluster this big at least (cells: 1,600 m²), with frontage of its own and `AREA_APART` from every area, is a new village. */
const VILLAGE_CELLS = 100;
/** A village keeps this many lots at least (`townCheck.ts`); a new one with fewer is folded into the nearest area. */
const VILLAGE_LOTS = 4;
/** A patch of green the lots leave this big at least (cells) is given an outdoor spot if it has room for one. */
const SPOT_LEAST = 9;
/** How far behind the kerb (m) a road's frontage is read for what goes along it. */
const PROBES = [4, 8, 12] as const;
/** The backdrop's buildings each use is built with (by use code): homes, terraces and flats, shops and work, (none on a green), public places. */
const BUILDINGS: readonly (readonly BackdropUse[] | null)[] = [null, ['house'], ['terrace', 'flats'], ['shop', 'cafe', 'restaurant', 'office', 'workshop'], null, ['hall']];
/** Which use each kind of backdrop building goes on (as `proposal.ts` paints it). */
const USE_OF: Partial<Record<BackdropUse, UseKind>> = { house: 'homes', terrace: 'rows', flats: 'rows', shop: 'shops', cafe: 'shops', restaurant: 'shops', office: 'shops', workshop: 'shops', hall: 'public', church: 'public', school: 'public' };
/** The shop lots an area with shops and none of its own is given one of, and a green stretch's lots. */
const SHOP_LOTS = ['grocer', 'bakery', 'cafe'];
const GREEN_LOTS = ['park', 'playground'];
/** Public frontage is read in runs this many metres a step. */
const RUN_STEP = 2;
/** A civic place goes on a run of public frontage this much of its kind's narrowest at least; as wide as a run no longer than `CIVIC_SNUG` times its kind's widest (no wider than that widest), else as wide as its kind is. */
const CIVIC_SHORT = 0.8;
const CIVIC_SNUG = 1.25;
/** A civic place that doesn't fit at the end of its run is tried this far on along it (m), and on. */
const CIVIC_STEP = 3;
/** Of land newly painted terraces and flats, the share terraced lots take (the backdrop's terraces and flats the rest), and a terraced lot's ground, about (m²). */
const TERRACE_SHARE = 0.5;
const TERRACE_LOT = 220;
/** Houses drawn in a row that fit nowhere before an area's houses are done. */
const HOUSE_MISSES = 8;
/** The blocks are filled in a building every so far (m) over the land painted for one and not built on. */
const INFILL_APART = 10;

// ---- streams ----------------------------------------------------------------------------------------

/** One 32-bit seed from a seed and words (FNV-1a over them, scrambled with MurmurHash3's last step), as `edit/buildup.ts` draws its streams. */
function mix(seed: number, ...words: (string | number)[]): number {
  let h = 0x811c9dc5 ^ (seed >>> 0);
  for (const ch of words.join('|')) h = Math.imul(h ^ ch.charCodeAt(0), 0x01000193);
  h ^= h >>> 16;
  h = Math.imul(h, 0x85ebca6b);
  h ^= h >>> 13;
  h = Math.imul(h, 0xc2b2ae35);
  h ^= h >>> 16;
  return h >>> 0;
}

const dist = (a: Pt, b: Pt) => Math.hypot(a[0] - b[0], a[1] - b[1]);
const offRect = (r: Rect, [x, z]: Pt) => Math.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d));
const middle = (a: Rect): Pt => [a.x + a.w / 2, a.z + a.d / 2];
const centroidOf = (poly: readonly Pt[]): Pt => [poly.reduce((s, p) => s + p[0], 0) / poly.length, poly.reduce((s, p) => s + p[1], 0) / poly.length];

/** A road's identity between the proposal and the layout: its line, kind and width (not its name nor area). */
const roadId = (r: RoadPlan) => JSON.stringify([r.points, r.kind ?? 'street', r.width]);

/** `draft` without its uses (a town built carries none on: they never go into its file). */
function withoutUses(draft: NeighbourhoodDraft): NeighbourhoodDraft {
  if (!draft.open || !('uses' in draft.open)) return draft;
  const { uses: _uses, ...open } = draft.open;
  return { ...draft, open };
}

// ---- cells ------------------------------------------------------------------------------------------

/** The uses grid's cells whose middles a rectangle covers (the one under its middle if none). */
function rectCells(r: Rect, cols: number, rows: number): number[] {
  const out: number[] = [];
  const [i0, i1] = [Math.max(0, Math.ceil(r.x / USE_CELL - 0.5)), Math.min(cols - 1, Math.floor((r.x + r.w) / USE_CELL - 0.5))];
  const [j0, j1] = [Math.max(0, Math.ceil(r.z / USE_CELL - 0.5)), Math.min(rows - 1, Math.floor((r.z + r.d) / USE_CELL - 0.5))];
  for (let j = j0; j <= j1; j++) for (let i = i0; i <= i1; i++) out.push(j * cols + i);
  if (!out.length) {
    const [ci, cj] = [Math.floor((r.x + r.w / 2) / USE_CELL), Math.floor((r.z + r.d / 2) / USE_CELL)];
    if (ci >= 0 && cj >= 0 && ci < cols && cj < rows) out.push(cj * cols + ci);
  }
  return out;
}

/** The uses grid's cells whose middles a polygon covers (the one under its middle if none). */
function polyCells(poly: readonly Pt[], cols: number, rows: number): number[] {
  const out: number[] = [];
  const xs = poly.map((p) => p[0]);
  const zs = poly.map((p) => p[1]);
  const [i0, i1] = [Math.max(0, Math.floor(Math.min(...xs) / USE_CELL)), Math.min(cols - 1, Math.floor(Math.max(...xs) / USE_CELL))];
  const [j0, j1] = [Math.max(0, Math.floor(Math.min(...zs) / USE_CELL)), Math.min(rows - 1, Math.floor(Math.max(...zs) / USE_CELL))];
  for (let j = j0; j <= j1; j++) for (let i = i0; i <= i1; i++) if (inside(poly as Pt[], (i + 0.5) * USE_CELL, (j + 0.5) * USE_CELL)) out.push(j * cols + i);
  if (!out.length) {
    const [cx, cz] = centroidOf(poly);
    const [ci, cj] = [Math.floor(cx / USE_CELL), Math.floor(cz / USE_CELL)];
    if (ci >= 0 && cj >= 0 && ci < cols && cj < rows) out.push(cj * cols + ci);
  }
  return out;
}

/** Whether `cells` (of `codes`) are painted `kind` enough for a lot or a building of it: `SHARE` of them at least, none a green but a green's. */
function painted(codes: Uint8Array, cells: readonly number[], kind: UseKind): boolean {
  if (!cells.length) return false;
  const [code, green] = [USE_CODE[kind], USE_CODE.green];
  let n = 0;
  for (const c of cells) {
    if (codes[c] === code) n++;
    else if (codes[c] === green) return false;
  }
  return n >= SHARE * cells.length;
}

// ---- areas ------------------------------------------------------------------------------------------

/**
 * `draft`'s areas as Build makes them (area 0 always the main town; the layout's own kept in their order):
 * each street or lane of no area (drawn by hand) is given the nearest area within `NEAR_AREA` m, whose
 * box grows to take it in, and each stretch zoned to be built (`dirty`, `fresh`: as `stretchesOf` reads
 * them; all with neither) grows the box of its road's area (a country road's: the nearest within
 * `NEAR_AREA` m), where the box's middle stays `AREA_APART` from every other's. A painted cluster
 * (`VILLAGE_CELLS` or more, further than `NEAR_AREA` from every box) along streets of no area, its box's
 * middle `AREA_APART` from every area's, is a new village (a hamlet, named as `hamletOf` names one, from
 * `seed`); the rest of the roads of no area go with the nearest. The draft, and the new villages' indices.
 */
export function areasFor(draft: NeighbourhoodDraft, content: Content, { seed = PROPOSAL_SEED, dirty, fresh }: { seed?: number; dirty?: Uint8Array; fresh?: ReadonlySet<number> } = {}): { draft: NeighbourhoodDraft; villages: number[] } {
  const open = draft.open;
  if (!open?.areas.length) return { draft, villages: [] };
  const areas: TownArea[] = open.areas.map((a) => ({ ...a }));
  const roads = draft.roads.slice();
  const grow: Rect[][] = areas.map(() => []);
  const nearestArea = (pts: readonly Pt[]) => {
    let [best, far] = [0, Infinity];
    areas.forEach((a, k) => {
      for (const p of pts) {
        const d = offRect(a, p);
        if (d < far) [best, far] = [k, d];
      }
    });
    return { area: best, far };
  };
  const bounds = (pts: readonly Pt[]): Rect => {
    const xs = pts.map((p) => p[0]);
    const zs = pts.map((p) => p[1]);
    return { x: Math.min(...xs), z: Math.min(...zs), w: Math.max(...xs) - Math.min(...xs), d: Math.max(...zs) - Math.min(...zs) };
  };
  // Streets and lanes of no area: the nearest within reach takes them in.
  const pending: number[] = [];
  roads.forEach((r, i) => {
    if (r.area !== undefined || r.kind === 'country' || r.bridge) return;
    const { area, far } = nearestArea(r.points);
    if (r.kind !== 'trail' && far > NEAR_AREA) return void pending.push(i);
    roads[i] = { ...r, area };
    if (r.kind !== 'trail') grow[area].push(bounds(r.points));
  });

  // A painted cluster out of every area's reach, along streets of no area: a village of its own.
  const villages: number[] = [];
  const layer = usesOfDraft(draft);
  const { cols, rows } = layer;
  const codes = unpackUses(layer);
  if (pending.length) {
    const far = new Uint8Array(codes.length);
    for (let k = 0; k < codes.length; k++) {
      if (!codes[k]) continue;
      const c: Pt = [((k % cols) + 0.5) * USE_CELL, (Math.floor(k / cols) + 0.5) * USE_CELL];
      if (areas.every((a) => offRect(a, c) > NEAR_AREA)) far[k] = 1;
    }
    const seen = new Uint8Array(codes.length);
    const spec = specFor(draft);
    for (let k0 = 0; k0 < far.length; k0++) {
      if (!far[k0] || seen[k0]) continue;
      const cells: number[] = [];
      const stack = [k0];
      seen[k0] = 1;
      while (stack.length) {
        const k = stack.pop()!;
        cells.push(k);
        const [i, j] = [k % cols, Math.floor(k / cols)];
        for (let dj = -1; dj <= 1; dj++) {
          for (let di = -1; di <= 1; di++) {
            const [a, b] = [i + di, j + dj];
            if (a < 0 || b < 0 || a >= cols || b >= rows) continue;
            const n = b * cols + a;
            if (far[n] && !seen[n]) {
              seen[n] = 1;
              stack.push(n);
            }
          }
        }
      }
      if (cells.length < VILLAGE_CELLS || (dirty && !cells.some((k) => dirty[k]))) continue;
      // (Its streets: those of no area running by it, its frontage.)
      const near = new Uint8Array(codes.length);
      for (const k of cells) near[k] = 1;
      const by = grown(near, cols, rows, Math.ceil(24 / USE_CELL));
      const own = pending.filter((i) => roads[i].points.some(([x, z]) => {
        const [ci, cj] = [Math.floor(x / USE_CELL), Math.floor(z / USE_CELL)];
        return ci >= 0 && cj >= 0 && ci < cols && cj < rows && by[cj * cols + ci] === 1;
      }));
      if (!own.length) continue;
      const pts: Pt[] = [...cells.flatMap((k): Pt[] => [[(k % cols) * USE_CELL, Math.floor(k / cols) * USE_CELL], [((k % cols) + 1) * USE_CELL, (Math.floor(k / cols) + 1) * USE_CELL]]), ...own.flatMap((i) => roads[i].points)];
      const b = bounds(pts);
      const box = { x: Math.floor(b.x), z: Math.floor(b.z), w: Math.ceil(b.x + b.w) - Math.floor(b.x), d: Math.ceil(b.z + b.d) - Math.floor(b.z) };
      if (areas.some((a) => dist(middle(a), middle(box)) < AREA_APART)) continue;
      const name = hamletOf(content, spec, mulberry32(mix(seed, 'village', villages.length)), areas.length - 1, areas.map((a) => a.name)).name;
      const at = areas.push({ name, kind: 'hamlet', ...box }) - 1;
      grow.push([]);
      villages.push(at);
      for (const i of own) roads[i] = { ...roads[i], area: at };
      pending.splice(0, pending.length, ...pending.filter((i) => !own.includes(i)));
    }
  }
  // The rest of the roads of no area: the nearest area's, its box as it was.
  for (const i of pending) roads[i] = { ...roads[i], area: nearestArea(roads[i].points).area };

  // The stretches to be built grow their areas' boxes.
  const placed: NeighbourhoodDraft = { ...draft, roads, open: { ...open, areas } };
  for (const st of stretchesOf(placed, { dirty, fresh })) {
    if (dirty && !st.dirty) continue;
    const r = roads[st.road];
    let a = r.area;
    if (a === undefined) {
      const { area, far } = nearestArea([st.mid]);
      if (far > NEAR_AREA) continue;
      a = area;
    }
    const ls = lengths(r.points);
    const ends = [pointAt(r.points, st.s0, ls), pointAt(r.points, st.s1, ls)];
    const back = r.width / 2 + 18;
    grow[a].push(bounds(ends.flatMap(({ p, t }): Pt[] => {
      const n = leftOf(t).map((v) => v * st.side);
      return [p, [p[0] + n[0] * back, p[1] + n[1] * back]];
    })));
  }
  grow.forEach((list, k) => {
    if (!list.length) return;
    const a = areas[k];
    const b = bounds([[a.x, a.z], [a.x + a.w, a.z + a.d], ...list.flatMap((r): Pt[] => [[r.x, r.z], [r.x + r.w, r.z + r.d]])]);
    const box = { x: Math.max(0, Math.floor(b.x)), z: Math.max(0, Math.floor(b.z)), w: 0, d: 0 };
    [box.w, box.d] = [Math.min(draft.width, Math.ceil(b.x + b.w)) - box.x, Math.min(draft.depth, Math.ceil(b.z + b.d)) - box.z];
    if (areas.some((o, j) => j !== k && dist(middle(o), middle(box)) < AREA_APART)) return;
    areas[k] = { ...a, ...box };
  });
  return { draft: placed, villages };
}

/** The spec `draft`'s world was laid out from (its lot mix, gaps, terraces, hamlets). */
function specFor(draft: NeighbourhoodDraft): WorldSpec {
  return specOf(placeOf(draft.world?.kind ?? 'country'), draft.world?.options);
}

// ---- what of the proposal's town stays ---------------------------------------------------------------

/** The edge of the nearest road but a footpath to `p` (m, from its middle line less half its width), on `index`'s roads; Infinity if none within 60 m. */
function toWay(index: RoadIndex, p: Pt): number {
  let best = Infinity;
  for (const [r, hit] of index.around(p[0], p[1], 60)) if (index.roads[r].kind !== 'trail') best = Math.min(best, hit.dist - index.roads[r].width / 2);
  return best;
}

/** The footpath outdoor lot `s` is at the end of (its end within 6 m of it), or -1. */
function ownTrail(roads: readonly RoadPlan[], t: Templates, s: PlotSlot): number {
  const r = plotRect(t, s);
  return roads.findIndex((road) => road.kind === 'trail' && offRect(r, road.points[road.points.length - 1]) <= 6);
}

/**
 * What of the proposal's town (`town`, its land as laid) is cleared for the layout `plan`: the lots and
 * backdrop buildings over `dirty` cells (a farm and its buildings over cells `zoned` instead, if given:
 * built again only where painted), in an area the layout no longer has, on its roads, fronting a road
 * that is gone (the nearest road to their front further than it was) or turned from a new road nearer
 * (a building), and an outdoor spot whose trail is gone; a farm's buildings with it. Lots and buildings
 * by index, and the footpaths that led to the outdoor spots cleared (by their index in `plan`).
 */
function clearedFor(town: NeighbourhoodDraft, plan: NeighbourhoodDraft, t: Templates, dirty: Uint8Array, zoned: Uint8Array | null, areaMap: readonly number[]): { lots: Set<number>; backdrop: Set<number>; trails: Set<number> } {
  const { cols, rows } = usesOfDraft(plan);
  const lots = new Set<number>();
  const backdrop = new Set<number>();
  const trails = new Set<number>();
  const planIds = plan.roads.map(roadId);
  const ids = new Set(planIds);
  const same = town.roads.length === plan.roads.length && town.roads.every((r) => ids.has(roadId(r)));
  // (The roads but the footpaths: an outdoor spot's own runs a metre into it.)
  const ground = new Ground(plan.width, plan.depth, plan.roads.filter((r) => r.kind !== 'trail'), plan.greens);
  const [was, now] = [new RoadIndex(town.roads), new RoadIndex(plan.roads)];
  const over = (cells: readonly number[], mask: Uint8Array) => cells.some((c) => mask[c] === 1);
  const farms: Pt[] = [];
  for (const s of town.slots) {
    const r = plotRect(t, s);
    const farm = s.lot === 'farm';
    let gone = areaMap[s.area ?? 0] < 0 || over(rectCells(r, cols, rows), farm && zoned ? zoned : dirty) || !ground.free(r, 0, true);
    if (!gone && s.kind === 'outdoor') {
      const trail = ownTrail(town.roads, t, s);
      gone = trail < 0 || !ids.has(roadId(town.roads[trail]));
    } else if (!gone && !same) {
      const f = frontOf(t, s);
      gone = toWay(now, f) > toWay(was, f) + 0.5;
    }
    if (!gone) continue;
    lots.add(s.index);
    if (farm) farms.push(middle(r));
    if (s.kind === 'outdoor') {
      const trail = ownTrail(plan.roads, t, s);
      if (trail >= 0) trails.add(trail);
    }
  }
  (town.open?.backdrop ?? []).forEach((b, i) => {
    const c = centroidOf(b.poly);
    const farmish = b.use === 'farm' || b.use === 'barn';
    let gone = areaMap[b.area] < 0 || over(polyCells(b.poly, cols, rows), farmish && zoned ? zoned : dirty) || (farmish && farms.some((m) => dist(m, c) < 70));
    if (!gone && !same) {
      const [before, after] = [toWay(was, c), toWay(now, c)];
      gone = after > before + 0.5 || (after < before - 0.5 && !facesRoad(b.poly, plan.roads, now)) || b.poly.some((p, k) => {
        const q = b.poly[(k + 1) % b.poly.length];
        return [0, 0.25, 0.5, 0.75].some((u) => now.onRoad(p[0] + (q[0] - p[0]) * u, p[1] + (q[1] - p[1]) * u, 0.5));
      });
    }
    if (gone) backdrop.add(i);
  });
  return { lots, backdrop, trails };
}

// ---- the fill ---------------------------------------------------------------------------------------

/** A public lot of `template` (a park, a playground, a shop) as `wantedLots` makes one, in area `area`. */
function publicLot(t: Templates, template: string, area: number): Unplaced | null {
  const p = t.parks.find((q) => q.id === template);
  if (!p) return null;
  return { kind: p.kind === 'shop' ? 'shop' : 'park', template: p.id, name: p.name, lot: 'medium', w: t.plot.width, d: t.plot.depth, area };
}

/**
 * Puts each of `wanted` along `frontage` as `LotPlacer.place` puts a lot (its first place where it fits,
 * snug to its road, a car able to drive out), but only where the cells under it are painted `kind`
 * (`painted`). How many went down.
 */
function placePainted(placer: LotPlacer, t: Templates, roads: readonly RoadPlan[], wanted: readonly Unplaced[], frontage: readonly Frontage[], codes: Uint8Array, cols: number, rows: number, kind: UseKind): number {
  let n = 0;
  for (const u of wanted) {
    const { width, depth } = slotDims(t, u);
    for (const f of frontage) {
      const at = lotAt(placer.ground, roads[f.road], f, width, depth, true);
      if (!at || !placer.drivesOut(u, at)) continue;
      if (!painted(codes, rectCells(plotRect(t, { ...u, ...at }), cols, rows), kind)) continue;
      placer.put(u, at, f);
      n++;
      break;
    }
  }
  return n;
}

/**
 * A row of terraced houses `units` along the first of `frontage` where the whole row fits as one lot (`lotAt`,
 * snug) on land painted terraces and flats (`painted`), put down by `LotPlacer.placeRow`; none if it fits nowhere.
 */
function rowAlong(placer: LotPlacer, t: Templates, roads: readonly RoadPlan[], units: readonly Unplaced[], frontage: readonly Frontage[], codes: Uint8Array, cols: number, rows: number): PlotSlot[] {
  const total = units.reduce((n, u) => n + slotDims(t, u).width, 0);
  const depth = Math.max(...units.map((u) => slotDims(t, u).depth));
  for (const f of frontage) {
    const at = lotAt(placer.ground, roads[f.road], f, total, depth, true);
    if (!at || !painted(codes, rectCells(plotRect(t, { lot: 'terrace', w: total, d: depth, ...at }), cols, rows), 'rows')) continue;
    const placed = placer.placeRow(units, [f], units[0].area);
    if (placed.length) return placed;
  }
  return [];
}

/** The smallest home's lot (m along its road, m deep), for `roomAlong`. */
const SMALLEST: readonly [number, number] = [12, 16];

/** Whether any of `frontage` has room for the smallest home's lot (`SMALLEST`) on land painted homes: no house is drawn where none could go. */
function roomAlong(placer: LotPlacer, roads: readonly RoadPlan[], frontage: readonly Frontage[], codes: Uint8Array, cols: number, rows: number): boolean {
  const [w, d] = SMALLEST;
  return frontage.some((f) => {
    const at = lotAt(placer.ground, roads[f.road], f, w, d, true);
    return !!at && painted(codes, rectCells(at.facing % 2 ? { x: at.x, z: at.z, w: d, d: w } : { x: at.x, z: at.z, w, d }, cols, rows), 'homes');
  });
}

/**
 * Where outdoor place `place` goes on a patch of green (`cells`, of a grid `cols` wide), as `macro.ts` `siteFor`
 * scores a site: by the water (its back to it), up a hill, in the woods (five of the nine cells round it wooded),
 * in a meadow; facing the area's middle `home` (its trail comes from there) or away from the water; on land nothing
 * takes (`ground`), dry, painted green (`SHARE` of `codes`' cells under it), and (`reach`, for a green with no
 * frontage) no nearer a road than `SPOT_NEAR`. Its rectangle's corner and facing; null if it has no site there.
 */
function siteIn(place: PublicTemplate, cells: readonly number[], home: Pt, land: ReturnType<typeof cells8>, water: Uint8Array, ground: Ground, t: Templates, codes: Uint8Array, cols: number, rows: number, W: number, D: number, reach: Float32Array | null): { x: number; z: number; facing: Facing } | null {
  const frame = place.lot ?? t.plot;
  const wet = (x: number, z: number) => x >= 0 && z >= 0 && x < W && z < D && water[Math.floor(z) * W + Math.floor(x)] === 1;
  let best: { at: { x: number; z: number; facing: Facing }; score: number } | null = null;
  for (const k of cells) {
    const c = cellMid(k, cols);
    let n: Pt = [c[0] - home[0], c[1] - home[1]];
    let score = 0;
    if (place.site === 'water') {
      let [sx, sz, hits] = [0, 0, 0];
      for (let a = 0; a < 16; a++) {
        const [dx, dz] = [Math.cos((a / 16) * Math.PI * 2), Math.sin((a / 16) * Math.PI * 2)];
        if (wet(c[0] + dx * 16, c[1] + dz * 16)) [sx, sz, hits] = [sx + dx, sz + dz, hits + 1];
      }
      if (!hits || hits > 9) continue;
      n = [sx, sz];
      score = -hits;
    } else if (place.site === 'hill') score = -land.at(c[0], c[1]) + land.slope(land.cell(c[0], c[1])) * 60;
    else if (place.site === 'woods') {
      let trees = 0;
      for (let dz = -1; dz <= 1; dz++) for (let dx = -1; dx <= 1; dx++) trees += land.woods[land.cell(c[0] + dx * 16, c[1] + dz * 16)];
      if (trees < 5) continue;
      score = -trees + land.slope(land.cell(c[0], c[1])) * 60;
    } else score = land.woods[land.cell(c[0], c[1])] * 4 + land.slope(land.cell(c[0], c[1])) * 60;
    // (Nearer the patch's own middle the better, a little: a tie goes to the middle.)
    if (best && score >= best.score) continue;
    const facing = (Math.abs(n[1]) >= Math.abs(n[0]) ? (n[1] > 0 ? 0 : 2) : n[0] > 0 ? 1 : 3) as Facing;
    const [w, d] = facing % 2 ? [frame.depth, frame.width] : [frame.width, frame.depth];
    const r = { x: Math.round(c[0] - w / 2), z: Math.round(c[1] - d / 2), w, d };
    if (!ground.free({ x: r.x - 3, z: r.z - 3, w: r.w + 6, d: r.d + 6 }, 8)) continue;
    if (land.wet(c[0], c[1], 0) || [0, 0.5, 1].some((u) => [0, 0.5, 1].some((q) => wet(r.x - 2 + (r.w + 4) * u, r.z - 2 + (r.d + 4) * q)))) continue;
    const under = rectCells(r, cols, rows);
    if (!painted(codes, under, 'green') || (reach && under.some((q) => reach[q] <= SPOT_NEAR))) continue;
    best = { at: { x: r.x, z: r.z, facing }, score };
  }
  return best?.at ?? null;
}

/** The patches of green in `codes` (a grid `cols` × `rows`; cells side by side) but the cells of `taken`, in the grid's order. */
function greenPatches(codes: Uint8Array, cols: number, rows: number, taken: readonly number[]): number[][] {
  const seen = new Uint8Array(codes.length);
  for (const k of taken) seen[k] = 1;
  const out: number[][] = [];
  for (let k0 = 0; k0 < codes.length; k0++) {
    if (seen[k0] || codes[k0] !== USE_CODE.green) continue;
    const cells: number[] = [];
    const stack = [k0];
    seen[k0] = 1;
    while (stack.length) {
      const k = stack.pop()!;
      cells.push(k);
      const [i, j] = [k % cols, Math.floor(k / cols)];
      for (const [a, b] of [[i + 1, j], [i - 1, j], [i, j + 1], [i, j - 1]]) {
        const n = b * cols + a;
        if (a < 0 || b < 0 || a >= cols || b >= rows || seen[n] || codes[n] !== USE_CODE.green) continue;
        seen[n] = 1;
        stack.push(n);
      }
    }
    out.push(cells);
  }
  return out;
}

/**
 * The town `plan` (step 2's layout) builds, before its land is settled: see the top of this file.
 * `base`: the proposal it was laid out from (none: everything zoned is built from the uses).
 */
export function layOutUses(base: Proposal | null, plan: NeighbourhoodDraft, t: Templates, content: Content, seed: number): NeighbourhoodDraft {
  const open = plan.open;
  if (!open?.areas.length) return plan;
  const [W, D] = [plan.width, plan.depth];
  const layer = usesOfDraft(plan);
  const { cols, rows } = layer;
  const painted0 = unpackUses(layer);
  // (Land zoned too far from a road builds nothing, as step 2 hatches it: but a green that becomes an outdoor spot.)
  const codes = painted0.slice();
  const far = hatched(plan);
  for (let k = 0; k < codes.length; k++) if (far[k]) codes[k] = 0;
  const full = !base || seed !== PROPOSAL_SEED;
  const town = base ? withoutUses(base.builtPlan ?? base.built) : null;

  // ---- what is built again: the changed cells and 20 m round them; everything, built again from another seed ----
  let dirty: Uint8Array = new Uint8Array(codes.length);
  let zoned: Uint8Array | null = null;
  if (full) {
    dirty.fill(1);
    zoned = new Uint8Array(codes.length);
    for (let k = 0; k < codes.length; k++) zoned[k] = painted0[k] ? 1 : 0;
  } else {
    const was = unpackUses(usesOfDraft(base!.plan));
    for (let k = 0; k < codes.length; k++) if (painted0[k] !== was[k]) dirty[k] = 1;
    dirty = grown(dirty, cols, rows, DIRTY_GROW);
  }

  // ---- what of the proposal's town stays ----
  let draft: NeighbourhoodDraft = { ...plan, slots: [], households: [], open: { ...open, backdrop: [], places: [], parking: [], pads: {} }, diagnostics: { dropped: [], ...(plan.diagnostics?.roads ? { roads: plan.diagnostics.roads } : {}) } };
  const touched = new Set<number>();
  let roads = plan.roads;
  if (town?.open) {
    const areaMap = town.open.areas.map((a) => open.areas.findIndex((b) => b.name === a.name));
    const gone = clearedFor(town, plan, t, dirty, zoned, areaMap);
    for (const i of gone.lots) touched.add(areaMap[town.slots[i].area ?? 0]);
    for (const i of gone.backdrop) touched.add(areaMap[town.open.backdrop![i].area]);
    const left = removeLots(town, gone.lots, gone.backdrop);
    const lo = left.open!;
    const ground = new Ground(W, D, plan.roads, plan.greens);
    // (A space as the generator laid it: 6 m along the verge, 3 m deep.)
    const space = (p: { x: number; z: number; rot: number }) => (p.rot % 2 ? { x: Math.round(p.x - 3), z: Math.round(p.z - 1.5), w: 6, d: 3 } : { x: Math.round(p.x - 1.5), z: Math.round(p.z - 3), w: 3, d: 6 });
    const parking = lo.parking.filter((p) => areaMap[p.area] >= 0 && ground.free(space(p), 0)).map((p) => ({ ...p, area: areaMap[p.area] }));
    roads = plan.roads.filter((_, i) => !gone.trails.has(i));
    draft = {
      ...draft,
      roads,
      slots: left.slots.map((s) => ({ ...s, area: areaMap[s.area ?? 0] })),
      open: {
        ...draft.open!,
        backdrop: (lo.backdrop ?? []).map((b) => ({ ...b, area: areaMap[b.area] })),
        places: (lo.places ?? []).map((p) => ({ ...p, area: areaMap[p.area] })),
        pads: lo.pads,
        parking,
      },
    };
  }
  touched.delete(-1);

  // ---- the areas, and the stretches to fill ----
  const baseIds = new Set(town?.roads.map(roadId) ?? []);
  const fresh = new Set(roads.flatMap((r, i) => (baseIds.has(roadId(r)) ? [] : [i])));
  const made = areasFor(draft, content, { seed, dirty, fresh });
  draft = made.draft;
  roads = draft.roads;
  const areas = draft.open!.areas;
  const nearestArea = (p: Pt) => areas.reduce((k, a, i) => (offRect(a, p) < offRect(areas[k], p) ? i : k), 0);
  const areaOf = (st: Stretch) => roads[st.road].area ?? nearestArea(st.mid);
  const stretches = stretchesOf(draft, { dirty, fresh })
    .filter((st) => st.dirty)
    .map((st) => ({ ...st, area: areaOf(st) }))
    .sort((a, b) => a.area - b.area || dist(a.mid, middle(areas[a.area])) - dist(b.mid, middle(areas[b.area])));
  for (const st of stretches) touched.add(st.area);
  for (const i of fresh) if (roads[i]?.area !== undefined) touched.add(roads[i].area!);
  for (const v of made.villages) touched.add(v);
  if (full) areas.forEach((_, a) => touched.add(a));

  // (Each changed area's parking is laid again after: out of the way of what is built till then.)
  draft = { ...draft, open: { ...draft.open!, parking: draft.open!.parking.filter((q) => !touched.has(q.area)) } };

  // ---- where new things go: the land painted a use, and to be built again ----
  const mask = new Uint8Array(codes.length);
  for (let k = 0; k < codes.length; k++) if (dirty[k]) mask[k] = codes[k];
  const codeAt = (x: number, z: number) => {
    const [ci, cj] = [Math.floor(x / USE_CELL), Math.floor(z / USE_CELL)];
    return ci >= 0 && cj >= 0 && ci < cols && cj < rows ? mask[cj * cols + ci] : 0;
  };
  /** Whether the land `PROBES` behind the kerb at `p` (`n`: away from road `r`) is painted `kind` anywhere. */
  const behind = (r: number, p: Pt, n: Pt, kind: UseKind) => {
    const [code, half] = [USE_CODE[kind], roads[r].width / 2];
    return PROBES.some((b) => codeAt(p[0] + n[0] * (half + b), p[1] + n[1] * (half + b)) === code);
  };
  const paintedPoly = (poly: readonly Pt[], kind: UseKind) => painted(mask, polyCells(poly, cols, rows), kind);
  /** How deep the land painted `kind` reaches back from road `r` at `s` on `side` (m, from 3 m past its edge, where a civic place's front is). */
  const deepAt = (r: number, s: number, side: 1 | -1, kind: UseKind) => {
    const { p, t: tt } = pointAt(roads[r].points, s);
    const n = leftOf(tt).map((v) => v * side);
    const from = roads[r].width / 2 + 3;
    let d = 0;
    while (d < 60 && codeAt(p[0] + n[0] * (from + d + 1), p[1] + n[1] * (from + d + 1)) === USE_CODE[kind]) d++;
    return d + 1;
  };
  const kindsAt = (p: Pt) => BUILDINGS[codeAt(p[0], p[1])] ?? null;

  // ---- the frontage, area by area: each area's roads, and a country road's nearest the area ----
  // (Each area's nearest its middle first, with a little noise, as `frontageOf` sorts it; only what is painted behind.)
  const spec = specFor(draft);
  const frontedRoads = roads.flatMap((r, i) => (fronted(r) ? [i] : []));
  // (No lot by a junction or on a tight bend of the areas' roads; along a country road, where it runs straight, as the generator spills lots out along one.)
  const offCorners = cornersOf(roads, frontedRoads.filter((r) => roads[r].kind !== 'country'));
  const lens = new Map(frontedRoads.map((r) => [r, lengths(roads[r].points)]));
  const corners = (f: Frontage) => (roads[f.road].kind === 'country' ? straightAround(roads[f.road].points, f.s, BEND, BEND_MOST, lens.get(f.road)) : offCorners(f));
  const byArea: Frontage[][] = areas.map(() => []);
  for (const f of frontageOf(roads, frontedRoads, [0, 0], () => 0)) {
    if (PROBES.some((b) => codeAt(f.p[0] + f.n[0] * (roads[f.road].width / 2 + b), f.p[1] + f.n[1] * (roads[f.road].width / 2 + b)))) byArea[roads[f.road].area ?? nearestArea(f.p)].push(f);
  }
  // (The roads with land painted along them, either side: the backdrop and the civic places go on a side lots don't, as the generator lays them.)
  const builtAlong = new Set(frontedRoads.filter((r) => {
    const road = roads[r];
    const ls = lengths(road.points);
    for (let s = 2; s < ls[ls.length - 1]; s += 4) {
      const { p, t: tt } = pointAt(road.points, s, ls);
      const n = leftOf(tt);
      if ([1, -1].some((side) => PROBES.some((b) => codeAt(p[0] + n[0] * side * (road.width / 2 + b), p[1] + n[1] * side * (road.width / 2 + b))))) return true;
    }
    return false;
  }));
  byArea.forEach((list, a) => {
    const rand = mulberry32(mix(seed, 'frontage', areas[a].name));
    const mid = middle(areas[a]);
    for (const f of list) f.far = dist(f.p, mid) + rand() * 10;
    list.sort((x, y) => x.far - y.far);
  });
  const heartOf = (a: number): Pt => {
    const c = draft.open!.centre?.find((q) => q.area === a && q.poly.length >= 3);
    return c ? centroidOf(c.poly) : middle(areas[a]);
  };
  const wanted = (a: number) => byArea[a].length > 0 && (full || touched.has(a));

  // ---- the civic places: those each area lacks (`civicChoices`), the most needed first, nearest its heart, where painted public ----
  // (The public frontage in runs along each road's side, m along it, each with the area it is nearest and what civic places take of it.)
  const publicRuns: { r: number; side: 1 | -1; s0: number; s1: number; mid: Pt; area: number; taken: [number, number][] }[] = [];
  for (const r of frontedRoads) {
    if (!builtAlong.has(r)) continue;
    const road = roads[r];
    const ls = lengths(road.points);
    const L = ls[ls.length - 1];
    for (const side of [1, -1] as const) {
      let s0 = -1;
      const close = (s1: number) => {
        const mid = pointAt(road.points, (s0 + s1) / 2, ls).p;
        publicRuns.push({ r, side, s0, s1, mid, area: road.area ?? nearestArea(mid), taken: [] });
        s0 = -1;
      };
      for (let s = RUN_STEP; s <= L - RUN_STEP; s += RUN_STEP) {
        const { p, t: tt } = pointAt(road.points, s, ls);
        const on = behind(r, p, leftOf(tt).map((v) => v * side) as Pt, 'public');
        if (on && s0 < 0) s0 = s - RUN_STEP / 2;
        else if (!on && s0 >= 0) close(s - RUN_STEP / 2);
      }
      if (s0 >= 0) close(L - RUN_STEP / 2);
    }
  }
  /** The civic places area `a` of `from` lacks (`civicChoices`), the most needed first, each on the run of public frontage nearest the area's heart it fits snugly (else any it fits), as wide as the run lets it be. The buildings laid. */
  const civicIn = (pass: BackdropPass, from: NeighbourhoodDraft, a: number): number[] => {
    const heart = heartOf(a);
    const runs = publicRuns.filter((q) => q.area === a).sort((x, y) => dist(x.mid, heart) - dist(y.mid, heart));
    if (!runs.length) return [];
    const rand = mulberry32(mix(seed, 'civic', areas[a].name, pass.count));
    const laid: number[] = [];
    pass.allow = (poly) => paintedPoly(poly, 'public');
    for (const c of civicChoices(from, a)) {
      const [w0, w1] = c.w;
      let at = -1;
      for (const run of runs) {
        const ls = lengths(roads[run.r].points);
        const near = (s: number) => dist(pointAt(roads[run.r].points, s, ls).p, heart);
        for (const [u0, u1] of freeOf(run.s0, run.s1, run.taken)) {
          const len = u1 - u0;
          if (len < w0 * CIVIC_SHORT) continue;
          // (As wide as a run that just holds it; else as its kind is, at the run's end nearer the heart, as the generator packs them.)
          // (Nudged on along the run, `CIVIC_STEP` m at a time, and made shallower and narrower where it doesn't quite fit: against a square, a corner, say.)
          const w = len <= w1 * CIVIC_SNUG ? Math.min(w1, Math.max(w0 * CIVIC_SHORT, len - 1)) : w0 + (w1 - w0) * rand();
          const dir = near(u0 + w / 2) <= near(u1 - w / 2) ? 1 : -1;
          const tries: [number, number][] = [[Math.max(w0 * CIVIC_SHORT, w - 2), 1]];
          for (let shift = 0; shift + w <= len; shift += CIVIC_STEP) tries.splice(tries.length - 1, 0, [w, shift]);
          let s = 0;
          for (const [ww, shift] of tries) {
            if (at >= 0 || ww + shift > len) continue;
            s = dir > 0 ? u0 + shift + ww / 2 : u1 - shift - ww / 2;
            // (No deeper than the public land is there, as far as its kind allows.)
            const deep = Math.max(c.d[0] * CIVIC_SHORT, Math.min(c.d[1], deepAt(run.r, s, run.side, 'public')));
            for (const d of [Math.min(deep, c.d[0] + (c.d[1] - c.d[0]) * rand()), Math.min(deep, c.d[0])]) if (at < 0) at = pass.putCivic(c.index, run.r, s, run.side, rand, { w: ww, d });
            if (at >= 0) run.taken.push([s - ww / 2 - 1, s + ww / 2 + 1]);
          }
          if (at < 0) continue;
          laid.push(at);
          break;
        }
        if (at >= 0) break;
      }
    }
    pass.allow = undefined;
    return laid;
  };
  {
    const pass = new BackdropPass({ draft, t, content, squares: 'outline' });
    const laid = areas.flatMap((_, a) => (wanted(a) ? civicIn(pass, draft, a) : []));
    if (laid.length) {
      const places = pass.placesFor(laid, mulberry32(mix(seed, 'civic places')));
      draft = { ...draft, open: { ...draft.open!, backdrop: pass.result().backdrop, places: [...(draft.open!.places ?? []), ...places] } };
    }
  }

  // ---- the lots, area by area as the generator lays them: the public places, the terraces, the houses ----
  const ground = townGround(draft, t);
  const placers: LotPlacer[] = [];
  const newSlots = () => placers.flatMap((p) => p.slots);
  // A park on each green the roads go round that is painted green, changed and has none, as the generator puts one there.
  const greenParks = new Set<number>();
  for (const [g, poly] of draft.greens.entries()) {
    const cells = polyCells(poly, cols, rows);
    if (!cells.some((k) => dirty[k]) || !painted(codes, cells, 'green')) continue;
    const box = boundsOf(poly);
    const taken = [...draft.slots, ...newSlots()].some((s) => {
      const r = plotRect(t, s);
      return r.x < box.x + box.w && box.x < r.x + r.w && r.z < box.z + box.d && box.z < r.z + r.d;
    });
    if (taken) continue;
    const c = centroidOf(poly);
    const area = nearestArea(c);
    const u = publicLot(t, GREEN_LOTS[0], area);
    if (!u) continue;
    const placer = new LotPlacer(content, t, ground, roads, [0, 0], mulberry32(mix(seed, 'green', g)), true);
    if (!placer.place([u], [], [poly], c, area, true).length) greenParks.add(area);
    placers.push(placer);
  }
  // (The cells painted terraces and flats now, to be built, that the proposal had painted otherwise or not at all: by the area each is nearest.)
  const freshRows = areas.map(() => 0);
  {
    const was = base ? unpackUses(usesOfDraft(base.plan)) : null;
    for (let k = 0; k < mask.length; k++) if (mask[k] === USE_CODE.rows && (!was || was[k] !== USE_CODE.rows)) freshRows[nearestArea(cellMid(k, cols))]++;
  }
  for (const [a, fronts] of byArea.entries()) {
    if (!wanted(a)) continue;
    const rand = mulberry32(mix(seed, 'lots', areas[a].name));
    // (No gaps or meadows drawn after the lots: the land painted has them where the town had them.)
    const placer = new LotPlacer(content, t, ground, roads, [0, 0], rand, true);
    placers.push(placer);
    const lots = fronts.filter(corners);
    const along = (kind: UseKind, list = lots) => list.filter((f) => behind(f.road, f.p, f.n, kind));
    // The public places the area's lot mix wants (as `wantedLots` draws them for the main town or a village) and it hasn't: a shop on land painted shops, a park on green.
    const size: SizeSpec = a === 0 ? { ...spec.town, lots: 0, farms: [0, 0] } : { box: spec.hamlet.box, lots: 0, scale: spec.hamlet.scale, places: spec.hamlet.places, farms: [0, 0] };
    const has = new Set([...draft.slots, ...newSlots()].flatMap((s) => ((s.area ?? 0) === a && s.kind !== 'house' ? [s.template] : [])));
    if (greenParks.has(a)) has.add(GREEN_LOTS[0]);
    const [shops, greens] = [along('shops'), along('green')];
    for (const u of wantedLots(content, t, spec, size, rand, a === 0 ? { park: true } : { park: false, avoid: ['market'] })) {
      if (u.kind === 'house' || has.has(u.template)) continue;
      const kind: UseKind = u.kind === 'shop' || u.template === 'market' ? 'shops' : 'green';
      if (placePainted(placer, t, roads, [{ ...u, area: a }], kind === 'shops' ? shops : greens, mask, cols, rows, kind)) has.add(u.template);
    }
    // (Shops painted and no shop lot: one, the area's grocer, baker or café.)
    if (shops.length && ![...has].some((id) => SHOP_LOTS.includes(id))) {
      const u = publicLot(t, SHOP_LOTS[Math.floor(rand() * SHOP_LOTS.length)], a);
      if (u) placePainted(placer, t, roads, [u], shops, mask, cols, rows, 'shops');
    }
    // (Each green stretch painted along a road with room for one more: a park or a playground.)
    for (const st of stretches) {
      if (st.area !== a || st.kind !== 'green') continue;
      const u = publicLot(t, GREEN_LOTS[Math.floor(rand() * GREEN_LOTS.length)], a);
      if (u) placePainted(placer, t, roads, [u], greens.filter((f) => f.road === st.road && f.side === st.side && f.s >= st.s0 && f.s <= st.s1), mask, cols, rows, 'green');
    }
    // Terraces: rows of terraced houses along the land painted terraces and flats (off the centre), down to a pair,
    // as many as the main town's terraces (`WorldSpec.terraces`) less those standing, and of the land newly painted
    // terraces and flats as much again as half of it holds (the rest the backdrop's terraces and flats).
    const rowFronts = along('rows', fronts).filter((f) => !inCentre(draft, [f.p[0] + f.n[0] * 10, f.p[1] + f.n[1] * 10]));
    const rowSpec = { rows: spec.rows[1] >= 2 ? spec.rows : ([2, 4] as [number, number]) };
    const standing = draft.slots.filter((s) => s.lot === 'terrace' && (s.area ?? 0) === a).length;
    const drawn = a === 0 ? spec.terraces[0] + Math.floor(rand() * (spec.terraces[1] - spec.terraces[0] + 1)) : 0;
    let left = Math.max(0, drawn - standing) + Math.round((freshRows[a] * USE_CELL * USE_CELL * TERRACE_SHARE) / TERRACE_LOT);
    for (let k = 0; k < 400 && left >= 2 && rowFronts.length; k++) {
      let n = rowSpec.rows[0] + Math.floor(rand() * (rowSpec.rows[1] - rowSpec.rows[0] + 1));
      n = Math.min(n, left - (left - n === 1 ? 1 : 0));
      const units = Array.from({ length: Math.max(2, n) }, () => rowHouse(t, rand))
        .filter((h) => !!h)
        .map((h): Unplaced => ({ kind: 'house', name: '', lot: 'terrace', area: a, ...h }));
      if (units.length < 2) break;
      let placed: PlotSlot[] = [];
      for (let m = units.length; m >= 2 && !placed.length; m--) placed = rowAlong(placer, t, roads, units.slice(0, m), rowFronts, mask, cols, rows);
      if (!placed.length) break;
      left -= placed.length;
    }
    // Houses: one of the place's lot mix at a time while one fits, on the land painted homes (none in a town centre: its frontage is the centre's buildings').
    const homes = along('homes').filter((f) => !inCentre(draft, [f.p[0] + f.n[0] * 10, f.p[1] + f.n[1] * 10]));
    const one = { ...spec.town, lots: 1, places: [0, 0] as [number, number], farms: [0, 0] as [number, number] };
    for (let k = 0, missed = 0; k < 600 && missed < HOUSE_MISSES && homes.length; k++) {
      // (A house is drawn only where there is room for the smallest.)
      if (missed && !roomAlong(placer, roads, homes, mask, cols, rows)) break;
      const u = wantedLots(content, t, spec, one, rand, { park: false }).map((w) => ({ ...w, area: a }));
      missed = placePainted(placer, t, roads, u, homes, mask, cols, rows, 'homes') ? 0 : missed + 1;
    }
  }
  // Each green with no frontage near a town or village with no outdoor spot on it, then each patch of green the lots
  // left with none on it: an outdoor spot (of those the place has, the first whose land is there: `siteIn`), a trail to it from the area's streets.
  let land: ReturnType<typeof cells8> | null = null;
  let water: Uint8Array | null = null;
  const trails: RoadPlan[] = [];
  const names = new Set(roads.map((r) => r.name));
  const reach = roadReach(draft);
  const outdoor = new Map(t.parks.filter((p) => p.kind === 'outdoor').map((p) => [p.id, p]));
  const spotIds = [...new Set([...spec.spots, ...outdoor.keys()])].filter((id) => outdoor.has(id));
  const greenCode = USE_CODE.green;
  const lotCells = () => [...draft.slots, ...newSlots()].flatMap((s) => rectCells(plotRect(t, s), cols, rows));
  const patches = [
    ...greenSpots(draft).map((g) => ({ ...g, far: true })),
    ...greenPatches(mask, cols, rows, [...lotCells(), ...draft.greens.flatMap((g) => polyCells(g, cols, rows))])
      .filter((cells) => cells.length >= SPOT_LEAST && cells.some((k) => reach[k] <= SPOT_NEAR))
      .map((cells) => {
        const centre = cells.reduce((m, k): Pt => [m[0] + cellMid(k, cols)[0] / cells.length, m[1] + cellMid(k, cols)[1] / cells.length], [0, 0] as Pt);
        return { cells, centre, area: nearestArea(centre), far: false };
      }),
  ];
  const spotsIn = areas.map(() => 0);
  for (const spot of patches) {
    if (!spot.cells.some((k) => mask[k] === greenCode)) continue;
    const kept = [...draft.slots, ...newSlots()].some((s) => s.kind === 'outdoor' && rectCells(plotRect(t, s), cols, rows).some((k) => spot.cells.includes(k)));
    if (kept) continue;
    const rand = mulberry32(mix(seed, 'spot', Math.round(spot.centre[0]), Math.round(spot.centre[1])));
    water ??= open.water ? unrle(open.water, W * D) : new Uint8Array(W * D);
    land ??= open.land ? cells8(open.land) : null;
    if (!land) break;
    const home = middle(areas[spot.area]);
    const first = spotsIn[spot.area]++;
    let site: { place: PublicTemplate; at: { x: number; z: number; facing: Facing } } | null = null;
    for (let k = 0; k < spotIds.length && !site; k++) {
      const place = outdoor.get(spotIds[(first + k) % spotIds.length])!;
      const at = siteIn(place, spot.cells, home, land, water, ground, t, mask, cols, rows, W, D, spot.far ? reach : null);
      if (at) site = { place, at };
    }
    if (!site) continue;
    const { place, at } = site;
    const frame = place.lot ?? t.plot;
    const placer = new LotPlacer(content, t, ground, roads, [0, 0], rand, true);
    const area = areas[spot.area];
    const slot = placer.put({ kind: 'outdoor', template: place.id, name: `${area.name} ${place.name}`, lot: 'medium', w: frame.width, d: frame.depth, area: spot.area }, at, null);
    const areaRoads = roads.flatMap((r, i) => (r.area === spot.area && r.kind !== 'trail' && r.kind !== 'country' ? [i] : []));
    const plan: AreaPlan = { name: area.name, rect: { x: area.x, z: area.z, w: area.w, d: area.d }, centre: home, roads: areaRoads, greens: [], spec: spec.town, ports: [] };
    const others = [...draft.slots, ...placers.flatMap((p) => p.slots), slot];
    const trail = areaRoads.length ? trailTo(slot, plan, [...roads, ...trails], others, water, land, t, W, D) : null;
    if (!trail) continue;
    let name = `${slot.name} Trail`;
    if (names.has(name)) name = `${slot.name} Path`;
    for (let k = 2; names.has(name); k++) name = `${slot.name} Trail ${k}`;
    names.add(name);
    trails.push({ ...trail, name, area: spot.area });
    for (const [x, z] of trail.points) ground.take({ x: Math.floor(x) - 1, z: Math.floor(z) - 1, w: 3, d: 3 });
    placers.push(placer);
  }
  const added = placers.flatMap((p) => p.slots).map((s, k): PlotSlot => ({ ...s, index: draft.slots.length + k }));
  roads = [...roads, ...trails];
  const pads = { ...draft.open!.pads };
  for (const s of added) {
    if (s.kind !== 'outdoor' || !open.land) continue;
    const r = plotRect(t, s);
    pads[s.index] = Math.round(heightAt(open.land, r.x + r.w / 2, r.z + r.d / 2) * 10) / 10;
  }
  draft = { ...draft, roads, slots: [...draft.slots, ...added], open: { ...draft.open!, pads } };
  for (const s of added) if (s.kind !== 'house') draft = withLotPlace(draft, t, content, s.index);

  // ---- the backdrop, as the generator lays it: rows along the roads, a row behind, the blocks filled in; each building on land painted its use ----
  const pass = new BackdropPass({ draft, t, content, squares: 'outline' });
  const from = pass.count;
  // (Civic places the area's homes now make it big enough for, that it lacked before its lots went down.)
  for (const [a] of areas.entries()) if (wanted(a)) civicIn(pass, draft, a);
  pass.allow = (poly, use) => {
    const kind = USE_OF[use];
    return !!kind && paintedPoly(poly, kind);
  };
  const laidRows: { rows: Row[]; rand: Rand }[] = [];
  for (const r of frontedRoads) {
    if (!builtAlong.has(r)) continue;
    const road = roads[r];
    const rand = mulberry32(mix(seed, 'rows', roadKey(road)));
    const a = road.area ?? nearestArea(pointAt(road.points, lengths(road.points)[road.points.length - 1] / 2).p);
    laidRows.push({ rows: pass.rowsAlong(a, r, [1, -1], rand, undefined, { kindsAt }), rand });
  }
  for (const { rows: row, rand } of laidRows) pass.secondRow(row, rand);
  // The blocks filled in: a building on each patch of land painted for one and not yet built on, every `INFILL_APART` m of it.
  const spots = infillSpots(mask, cols, rows, [...draft.slots.map((s) => rectCells(plotRect(t, s), cols, rows)), ...pass.result().backdrop.map((b) => polyCells(b.poly, cols, rows))]);
  const areaAt = (p: Pt) => {
    const k = areas.findIndex((q) => offRect(q, p) === 0);
    return k >= 0 ? k : nearestArea(p);
  };
  const infillRand = mulberry32(mix(seed, 'infill'));
  for (const p of spots) pass.infillPoints(areaAt(p), [p], infillRand, kindsAt);
  pass.allow = undefined;
  const keep = Array.from({ length: pass.count - from }, (_, k) => from + k);
  const places = pass.placesFor(keep, mulberry32(mix(seed, 'places')));
  draft = { ...draft, open: { ...draft.open!, backdrop: pass.result().backdrop, places: [...(draft.open!.places ?? []), ...places] } };

  // ---- parking in each area changed; a village with too few lots goes with the nearest area ----
  // (A village built again with too few lots, new or the layout's own, goes the same way.)
  const villages = [...made.villages, ...[...touched].filter((a) => a > 0 && !made.villages.includes(a))].sort((a, b) => a - b);
  draft = foldSmallVillages(parkedAgain(draft, t, touched, seed), villages, t);

  // ---- the houses numbered along every street ----
  return renumber(draft, t, roads.map((_, i) => i));
}

/**
 * Where the blocks are filled in: over each patch of `codes` (a grid `cols` × `rows`) painted for a building (homes,
 * terraces and flats, shops and work, public places; one use, its cells side by side) but the cells of `built` (each a
 * lot's or a building's), the middles of cells `INFILL_APART` m apart at least, from the patch's middle out. In
 * the grid's order of the patches.
 */
function infillSpots(codes: Uint8Array, cols: number, rows: number, built: readonly (readonly number[])[]): Pt[] {
  const taken = new Uint8Array(codes.length);
  for (const cells of built) for (const k of cells) taken[k] = 1;
  const seen = new Uint8Array(codes.length);
  const out: Pt[] = [];
  const apart = INFILL_APART / USE_CELL;
  for (let k0 = 0; k0 < codes.length; k0++) {
    const code = codes[k0];
    if (seen[k0] || taken[k0] || !code || code === USE_CODE.green) continue;
    const cells: number[] = [];
    const stack = [k0];
    seen[k0] = 1;
    while (stack.length) {
      const k = stack.pop()!;
      cells.push(k);
      const [i, j] = [k % cols, Math.floor(k / cols)];
      for (const [a, b] of [[i + 1, j], [i - 1, j], [i, j + 1], [i, j - 1]]) {
        const n = b * cols + a;
        if (a < 0 || b < 0 || a >= cols || b >= rows || seen[n] || taken[n] || codes[n] !== code) continue;
        seen[n] = 1;
        stack.push(n);
      }
    }
    const [mi, mj] = [cells.reduce((s, k) => s + (k % cols), 0) / cells.length, cells.reduce((s, k) => s + Math.floor(k / cols), 0) / cells.length];
    const picked: [number, number][] = [];
    for (const k of cells.sort((x, y) => Math.hypot((x % cols) - mi, Math.floor(x / cols) - mj) - Math.hypot((y % cols) - mi, Math.floor(y / cols) - mj) || x - y)) {
      const [i, j] = [k % cols, Math.floor(k / cols)];
      if (picked.some(([pi, pj]) => Math.hypot(pi - i, pj - j) < apart)) continue;
      picked.push([i, j]);
      out.push([(i + 0.5) * USE_CELL, (j + 0.5) * USE_CELL]);
    }
  }
  return out;
}

/** What of `s0`..`s1` the intervals `taken` leave, in order. */
function freeOf(s0: number, s1: number, taken: readonly (readonly [number, number])[]): [number, number][] {
  let free: [number, number][] = [[s0, s1]];
  for (const [a, b] of taken) free = free.flatMap(([u0, u1]): [number, number][] => (b <= u0 || a >= u1 ? [[u0, u1]] : [...(a > u0 ? [[u0, a] as [number, number]] : []), ...(b < u1 ? [[b, u1] as [number, number]] : [])]));
  return free;
}

/** The middle of cell `k` of a grid `cols` wide (m). */
const cellMid = (k: number, cols: number): Pt => [((k % cols) + 0.5) * USE_CELL, (Math.floor(k / cols) + 0.5) * USE_CELL];

/**
 * `draft` with each of `villages` (areas but the main town, by index) that has fewer than `VILLAGE_LOTS` lots
 * (`townCheck.ts` wants so many of a village) folded into the nearest other area (its streets, lots, buildings,
 * places and parking), and gone with its name; an outdoor spot of it further than `SPOT_REACH` from that area's box
 * goes, and its trail.
 */
function foldSmallVillages(draft: NeighbourhoodDraft, villages: readonly number[], t: Templates): NeighbourhoodDraft {
  if (!draft.open || !villages.length) return draft;
  const small = villages.filter((v) => v > 0 && draft.slots.filter((s) => (s.area ?? 0) === v && s.kind !== 'outdoor').length < VILLAGE_LOTS);
  if (!small.length) return draft;
  const areas = draft.open.areas;
  const keep = areas.flatMap((_, i) => (small.includes(i) ? [] : [i]));
  /** The area (of `keep`) area `a` goes with: itself, else the one whose middle is nearest its middle. */
  const into = (a: number) => (keep.includes(a) ? a : keep.reduce((k, b) => (dist(middle(areas[b]), middle(areas[a])) < dist(middle(areas[k]), middle(areas[a])) ? b : k), keep[0]));
  // (Its outdoor spots too far from the area it goes with: gone, and their trails.)
  const spots = new Set(draft.slots.flatMap((s) => (s.kind === 'outdoor' && small.includes(s.area ?? 0) && offRect(areas[into(s.area ?? 0)], middle(plotRect(t, s))) > SPOT_REACH ? [s.index] : [])));
  if (spots.size) {
    const trails = new Set([...spots].map((i) => ownTrail(draft.roads, t, draft.slots[i])).filter((r) => r >= 0));
    draft = removeLots(draft, spots);
    draft = { ...draft, roads: draft.roads.filter((_, i) => !trails.has(i)) };
  }
  const open = draft.open!;
  const to = (a: number) => keep.indexOf(into(a));
  return {
    ...draft,
    roads: draft.roads.map((r) => (r.area === undefined ? r : { ...r, area: to(r.area) })),
    slots: draft.slots.map((s) => ({ ...s, area: to(s.area ?? 0) })),
    open: {
      ...open,
      areas: keep.map((a) => areas[a]),
      backdrop: (open.backdrop ?? []).map((b) => ({ ...b, area: to(b.area) })),
      places: (open.places ?? []).map((p) => ({ ...p, area: to(p.area) })),
      parking: open.parking.map((p) => ({ ...p, area: to(p.area) })),
    },
  };
}

/** `draft` with the parking of each area of `areas` laid again as the generator lays it (`parkingFor` its homes, by the verge nearest its middle). */
function parkedAgain(draft: NeighbourhoodDraft, t: Templates, areas: ReadonlySet<number>, seed: number): NeighbourhoodDraft {
  const open = draft.open;
  if (!open || !areas.size) return draft;
  const parking = open.parking.filter((p) => !areas.has(p.area));
  const ground = townGround({ ...draft, open: { ...open, parking } }, t);
  const { roads } = draft;
  for (const a of [...areas].sort((x, y) => x - y)) {
    const area = open.areas[a];
    if (!area) continue;
    const own = roads.flatMap((r, i) => (r.area === a && fronted(r) ? [i] : []));
    if (!own.length) continue;
    const homes = draft.slots.filter((s) => s.kind === 'house' && (s.area ?? 0) === a).length;
    const spaces = parkingFor(homes, a === 0);
    let n = 0;
    for (const f of frontageOf(roads, own, middle(area), mulberry32(mix(seed, 'parking', a)))) {
      if (n >= spaces) break;
      const facing = (Math.abs(f.n[1]) >= Math.abs(f.n[0]) ? (f.n[1] > 0 ? 0 : 2) : f.n[0] > 0 ? 1 : 3) as Facing;
      const reach = roads[f.road].width / 2 + 0.5;
      const [qx, qz] = [f.p[0] + f.n[0] * (reach + 3), f.p[1] + f.n[1] * (reach + 3)];
      const r = facing % 2 ? { x: Math.round(qx - 3), z: Math.round(qz - 1.5), w: 6, d: 3 } : { x: Math.round(qx - 1.5), z: Math.round(qz - 3), w: 3, d: 6 };
      if (!ground.free(r, 8) || parking.some((p) => Math.hypot(p.x - r.x, p.z - r.z) < 12)) continue;
      ground.take(r, 1);
      parking.push({ x: r.x + r.w / 2, z: r.z + r.d / 2, rot: facing, area: a });
      n++;
    }
  }
  return { ...draft, open: { ...open, parking } };
}

// ---- the job ----------------------------------------------------------------------------------------

/**
 * The town step 2's layout `plan` builds (see the top of this file), from `base` (the proposal it was laid
 * out from; none: everything zoned built from its uses) and the build's `seed`: its draft (no uses), its
 * preview, the houses its lots use, and (a land whose town follows its slopes) its plan (the land as laid,
 * step 3's edits start from it) and what in it is over a slope limit (`marks`).
 */
export function buildFromUses(base: Proposal | null, plan: NeighbourhoodDraft, t: Templates, content: Content, seed: number): LaidTown {
  const preview = (d: NeighbourhoodDraft) => previewWorld(content, t, d, null);
  if (base && seed === PROPOSAL_SEED && layoutHash(plan) === base.hash) {
    const draft = withoutUses(base.built);
    const built = base.builtPlan ? { plan: withoutUses(base.builtPlan), marks: base.builtMarks ?? [] } : {};
    return { draft, preview: preview(draft), houses: generatedHouses(t, draft.slots.map((s) => s.template)), ...built };
  }
  const laid = withoutUses(layOutUses(base, plan, t, content, seed));
  const settled = settleLand(laid, preview, undefined, content.careerCategories, { keepRoads: true });
  let draft = settled.draft;
  const world = settled.world;
  const land = draft.open?.land;
  if (draft.open && land && draft !== laid) {
    // The cover opened under what was built since the land was laid, and the woods, meadows and fields made again from it (as step 2's settle does).
    const opened = { ...land, cover: land.cover.slice() };
    const { zones } = openUnderBuilt(opened, world, draft.open.zones);
    draft = { ...draft, open: { ...draft.open, land: opened, zones } };
    world.meta = { ...world.meta, zones, land: encodeLand(opened) };
  }
  const asLaid = laid.open?.land;
  const own = draft.open && asLaid?.seeded === 'follow' && draft.open.land !== asLaid ? { plan: { ...draft, open: { ...draft.open, land: asLaid } }, marks: settled.marks } : {};
  return { draft, preview: world, houses: generatedHouses(t, draft.slots.map((s) => s.template)), ...own };
}

