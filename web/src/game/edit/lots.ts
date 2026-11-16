/**
 * Step 2's lot edits: a lot added along a road (a home, a farm, a public place, an outdoor spot),
 * moved, a new street filled with homes, a lot's house, garden or public place changed, a civic
 * building's use swapped, lots removed, and the houses along streets numbered again. Each is a pure
 * function from draft to draft and keeps the generator's own rules: a lot goes where `town.ts`
 * `lotAt` puts it on the town's ground (`ground.ts` `townGround`), its middle off the corners and
 * tight bends as `macro.ts` keeps an area's lots, no home in a town centre, and only where a car can
 * turn out of its driveway onto the road (`LotPlacer`'s driveway test). A street is filled by
 * `LotPlacer.place` from the world's lot mix (`wantedLots`), a garden cut again as `withGarden` cuts it.
 *
 * A lot is known by where it stands (`lotKey`), to the whole metre, as the people's streams key
 * its household (`people.ts` `peopleStreams`): its slot index changes as lots before it go.
 *
 * The generator's own pieces are used as it uses them: the corner and bend limits (`macro.ts`), the
 * driveway test (`town.ts` `drivesOut`), the public lots' hours and careers and the civic places
 * (`backdrop.ts` `LOT_HOURS`, `LOT_WORK`, `HOURS`, `CIVIC` and the names it draws: `namerFor`).
 */

import type { Content } from '../../content/content';
import type { BackdropBuilding, RoadPlan, TownPlace } from '../../core/protocol';
import { CIVIC, civicOf, HOURS, hoursFor, LOT_HOURS, LOT_WORK, namerFor } from '../backdrop';
import { weighted, type GardenSize } from '../housegen';
import { heightAt } from '../land';
import { BEND, BEND_MOST, CORNER, TIGHT_BEND, TRAIL_WIDTH } from '../macro';
import { mulberry32 } from '../noise';
import { evenly, leftOf, lengths, pointAt, roadIndex, straightAround, tangent, type Pt } from '../roads';
import {
  drivesOut as drivesOutOn,
  fitsLot,
  fronting,
  frontageOf,
  generatedId,
  houseFor,
  houseTemplate,
  lotAt,
  LotPlacer,
  plotRect,
  shuffled,
  slotDims,
  wantedLots,
  type Facing,
  type Frontage,
  type Ground,
  type LotSize,
  type NeighbourhoodDraft,
  type PlotSlot,
  type Rect,
  type Templates,
  type Unplaced,
} from '../town';
import { placeOf, specOf, type WorldSpec } from '../world';
import type { Edited } from './edited';
import { entriesOf, frontOf, groundOf, inCentre, townGround, type RoadSpot } from './ground';

type Rand = () => number;

/** What goes on a lot being added: a home of a size (with a garden), a farm, or a public place or outdoor spot (its template). */
export type LotChoice = { kind: 'house'; size: Exclude<LotSize, 'farm' | 'terrace'>; garden?: GardenSize } | { kind: 'farm' } | { kind: 'park' | 'shop' | 'outdoor'; template: string };

/** Where a lot would go (`canPlace`): its rectangle on the town and the way it faces. */
export interface LotFit extends Rect {
  facing: Facing;
}

/** An edit that cleared nothing out of its way. */
const NOTHING: { lots: number[]; backdrop: number[] } = { lots: [], backdrop: [] };

// ---- identity -----------------------------------------------------------------------------------

/**
 * A lot's identity across edits and re-settles: where it stands, to the whole metre (its corner on
 * the town), as `people.ts` `peopleStreams.lot(x, z)` keys the household that lives there. Shared
 * by the renderer, the people and the edit history.
 */
export function lotKey(slot: Pick<PlotSlot, 'x' | 'z'>): string {
  return `${Math.round(slot.x)},${Math.round(slot.z)}`;
}

/** The slot index of the lot `key` names in `draft` (`lotKey`), or -1 if it is gone. */
export function slotOf(draft: NeighbourhoodDraft, key: string): number {
  return draft.slots.findIndex((s) => lotKey(s) === key);
}

// ---- the generator's rules ------------------------------------------------------------------------

/** Nor right at a road's ends (m; `frontageOf`). */
const END = 4;

/** Each set of roads' junctions: per road, the arc lengths along it where another road meets it. */
const junctions = new WeakMap<readonly RoadPlan[], Map<number, number[]>>();

/**
 * Where each road meets another (arc lengths along it): its own joined ends, and where another
 * road's end joins it or another runs by close enough to touch, as `macro.ts` works it out before
 * it puts an area's lots down (trails are not there yet then, so they don't count here either).
 */
function meetsOf(roads: readonly RoadPlan[]): Map<number, number[]> {
  const had = junctions.get(roads);
  if (had) return had;
  const out = new Map<number, number[]>();
  roads.forEach((r, i) => {
    if (r.kind === 'trail') return;
    const ls = lengths(r.points);
    out.set(i, r.ends.flatMap((e, k) => (e === 'join' ? [k ? ls[ls.length - 1] : 0] : [])));
  });
  const index = roadIndex(roads);
  roads.forEach((o, j) => {
    if (o.kind === 'trail') return;
    for (const [x, z] of evenly(o.points, 2)) {
      for (const [r, hit] of index.around(x, z, 10)) {
        if (r !== j && out.has(r) && hit.dist <= (o.width + roads[r].width) / 2 + 2) out.get(r)!.push(hit.s);
      }
    }
  });
  junctions.set(roads, out);
  return out;
}

/** Whether a lot's middle may be at `s` along road `r`: off its corners and tight bends, and on a country road where it runs straight. */
function offCorners(roads: readonly RoadPlan[], r: number, s: number): boolean {
  const road = roads[r];
  const ls = lengths(road.points);
  if (!(meetsOf(roads).get(r) ?? []).every((m) => Math.abs(s - m) >= CORNER)) return false;
  if (!straightAround(road.points, s, CORNER / 2, TIGHT_BEND, ls)) return false;
  return road.kind !== 'country' || straightAround(road.points, s, BEND, BEND_MOST, ls);
}

/** Why no lot may front road `road` on `side` at all (a footpath, a bridge, the water side), or null. */
function noLotsOn(road: RoadPlan | undefined, side: 1 | -1): string | null {
  if (!road) return 'There is no road there.';
  if (road.kind === 'trail') return 'A footpath has no lots along it.';
  if (road.bridge) return 'No lot stands on a bridge.';
  if (road.lots && road.lots !== side) return 'That side of the road looks out over the water.';
  return null;
}

/** The frontage at `at` (snapped to the road's nearest point, as `frontageOf` has one at each), or why there is none. */
function frontageAt(roads: readonly RoadPlan[], at: RoadSpot): Frontage | string {
  const road = roads[at.road];
  const no = noLotsOn(road, at.side);
  if (no) return no;
  const ls = lengths(road.points);
  let i = 0;
  while (i + 1 < ls.length && Math.abs(ls[i + 1] - at.s) <= Math.abs(ls[i] - at.s)) i++;
  if (ls[i] <= END || ls[i] >= ls[ls.length - 1] - END) return 'Too near the end of the road.';
  const n = leftOf(tangent(road.points, i)).map((v) => v * at.side) as Pt;
  return { road: at.road, s: ls[i], side: at.side, p: road.points[i], n, far: 0 };
}

/**
 * Where lot `u` goes if it fronts road spot `at` on `draft` (`except`: that lot left off the
 * ground: the one being moved), as the generator would put it there: off the corners, no home in
 * a town centre, snapped by `lotAt` onto free ground, a car able to drive out. Or why not.
 */
function fitAt(draft: NeighbourhoodDraft, t: Templates, at: RoadSpot, u: Unplaced, except = -1): (LotFit & { anchor: number; f: Frontage }) | { refused: string } {
  const f = frontageAt(draft.roads, at);
  if (typeof f === 'string') return { refused: f };
  if (!offCorners(draft.roads, f.road, f.s)) return { refused: 'Too near a corner or a bend.' };
  if (u.kind === 'house' && inCentre(draft, [f.p[0] + f.n[0] * 10, f.p[1] + f.n[1] * 10])) return { refused: "The town centre's frontage is kept for its shops and buildings, not homes." };
  const { width, depth } = slotDims(t, u);
  const lot = lotAt(groundOf(draft, t, except), draft.roads[f.road], f, width, depth, true);
  if (!lot) return { refused: 'There is no room for it here.' };
  if (!drivesOut(draft.roads, t, u, lot)) return { refused: 'A car could not turn out of its driveway onto the road here.' };
  const r = plotRect(t, { ...u, ...lot });
  return { ...r, facing: lot.facing, anchor: lot.anchor, f };
}

/**
 * Whether a car can turn out of a house on lot `u` at `at` onto the road (`town.ts` `drivesOut`),
 * with things' footprints from the templates' object infos (as the content gives them).
 */
function drivesOut(roads: readonly RoadPlan[], t: Templates, u: Unplaced, at: { x: number; z: number; facing: Facing }): boolean {
  return drivesOutOn(roads, t, u, at, (def) => t.info?.(def)?.footprint ?? [1, 1]);
}

// ---- the world's lot mix, areas, places ------------------------------------------------------------

/** Each world's spec (its lot mix and gaps), made once. */
const specs = new WeakMap<object, WorldSpec>();
const PLAIN = {};

/** The spec `draft`'s world was laid out from (`world.ts` `specOf`): its lot mix, gaps and main town. */
function specFor(draft: NeighbourhoodDraft): WorldSpec {
  const key = draft.world ?? PLAIN;
  let spec = specs.get(key);
  if (!spec) specs.set(key, (spec = specOf(placeOf(draft.world?.kind ?? 'country'), draft.world?.options)));
  return spec;
}

/** The area a lot fronting road `road` at `p` is in: the road's own, else the area whose box is nearest. */
function areaAt(draft: NeighbourhoodDraft, road: number, [x, z]: Pt): number {
  const own = draft.roads[road]?.area;
  if (own !== undefined) return own;
  const areas = draft.open?.areas ?? [];
  let [best, far] = [0, Infinity];
  areas.forEach((a, i) => {
    const d = Math.hypot(Math.max(a.x - x, 0, x - a.x - a.w), Math.max(a.z - z, 0, z - a.z - a.d));
    if (d < far) [best, far] = [i, d];
  });
  return best;
}

/** The middle of area `a`'s box (`macro.ts` `areaCentre`), else of the world. */
function middleOf(draft: NeighbourhoodDraft, a: number): Pt {
  const area = draft.open?.areas[a];
  return area ? [area.x + area.w / 2, area.z + area.d / 2] : [draft.width / 2, draft.depth / 2];
}

/** What kind of place a public lot is (as `backdrop.ts` `layBackdrop` names them): a market is a shop, a café a café. */
function placeKindOf(s: Pick<PlotSlot, 'kind' | 'template'>): TownPlace['kind'] {
  return s.kind === 'outdoor' ? 'outdoor' : s.kind === 'park' ? (s.template === 'market' ? 'shop' : 'park') : s.template === 'cafe' ? 'cafe' : 'shop';
}

const r2 = (v: number) => Math.round(v * 100) / 100;

/** A name not among `taken`: `name`, else the area's name before it, else numbered (as `layBackdrop` names a hamlet's café). */
function freeName(name: string, taken: ReadonlySet<string>, area: string): string {
  if (!taken.has(name)) return name;
  let out = `${area} ${name.replace(/^The /, '')}`;
  for (let k = 2; taken.has(out); k++) out = `${name} ${k}`;
  return out;
}

/** The place public lot `s` is (as `layBackdrop` makes one): its kind, its name (none twice), its tag at its door or in its middle, hours and the careers working there. */
function lotPlace(draft: NeighbourhoodDraft, t: Templates, content: Content, s: PlotSlot, keep?: TownPlace): TownPlace {
  const kind = placeKindOf(s);
  const area = s.area ?? 0;
  const taken = new Set((draft.open?.places ?? []).filter((q) => q !== keep).map((q) => q.name));
  const name = freeName(s.name, taken, draft.open?.areas[area]?.name ?? draft.name);
  const r = plotRect(t, s);
  const front = ([[0.5, 0], [0, 0.5], [0.5, 1], [1, 0.5]] as const)[s.facing];
  const [x, z] = kind === 'park' || kind === 'outdoor' ? [r.x + r.w / 2, r.z + r.d / 2] : [r.x + r.w * front[0], r.z + r.d * front[1]];
  const work = LOT_WORK[s.template] ?? [];
  return {
    name,
    kind,
    x: r2(x),
    z: r2(z),
    plot: s.index,
    ...(kind === 'park' || kind === 'outdoor' ? {} : { hours: hoursFor(LOT_HOURS, work, content.careerCategories) }),
    ...(work.length ? { work } : {}),
    area,
  };
}

/** `draft` with the place of public lot `slot` (`lotPlace`) put in, in place of the one it had (a village placed by hand gives its public lots theirs: `villages.ts`). */
export function withLotPlace(draft: NeighbourhoodDraft, t: Templates, content: Content, slot: number): NeighbourhoodDraft {
  const open = draft.open;
  if (!open?.places) return draft;
  const had = open.places.findIndex((q) => q.plot === slot);
  const place = lotPlace(draft, t, content, draft.slots[slot], open.places[had]);
  const places = had < 0 ? [...open.places, place] : open.places.map((q, i) => (i === had ? place : q));
  return { ...draft, open: { ...open, places } };
}

/** `draft` with the tag of lot `slot`'s place (if it has one) moved to it as it now stands, its area with it. */
function placeFollows(draft: NeighbourhoodDraft, t: Templates, slot: number): NeighbourhoodDraft {
  const open = draft.open;
  const at = open?.places?.findIndex((q) => q.plot === slot) ?? -1;
  if (!open?.places || at < 0) return draft;
  const s = draft.slots[slot];
  const old = open.places[at];
  const r = plotRect(t, s);
  const front = ([[0.5, 0], [0, 0.5], [0.5, 1], [1, 0.5]] as const)[s.facing];
  const [x, z] = old.kind === 'park' || old.kind === 'outdoor' ? [r.x + r.w / 2, r.z + r.d / 2] : [r.x + r.w * front[0], r.z + r.d * front[1]];
  const places = open.places.map((q, i) => (i === at ? { ...q, x: r2(x), z: r2(z), area: s.area ?? 0 } : q));
  return { ...draft, open: { ...open, places } };
}

/** How high an outdoor lot's ground is levelled (m): the land's height at its middle, where the draft has its land. */
function padOf(draft: NeighbourhoodDraft, t: Templates, s: PlotSlot): number {
  const land = draft.open?.land;
  const r = plotRect(t, s);
  return land ? Math.round(heightAt(land, r.x + r.w / 2, r.z + r.d / 2) * 10) / 10 : 0;
}

/** The trail into outdoor lot `s` from the road it fronts (`f`): from the road's middle to a metre inside its front. Its name is the lot's with "Trail" (none twice). */
function trailInto(draft: NeighbourhoodDraft, t: Templates, s: PlotSlot, f: Frontage): RoadPlan {
  const [fx, fz] = frontOf(t, s);
  const [nx, nz] = f.n;
  const names = new Set(draft.roads.map((r) => r.name));
  let name = `${s.name} Trail`;
  if (names.has(name)) name = `${s.name} Path`;
  for (let k = 2; names.has(name); k++) name = `${s.name} Trail ${k}`;
  const p = (x: number, z: number): [number, number] => [r2(x), r2(z)];
  return { points: [p(f.p[0], f.p[1]), p(fx + nx, fz + nz)], width: TRAIL_WIDTH, ends: ['join', 'dead'], kind: 'trail', name, area: s.area ?? 0 };
}

/**
 * The trail leading to outdoor lot `slot` alone (its end within 6 m of it, as the town checks want,
 * no other lot near that end, no road joining it there), or -1: what goes when the lot moves.
 */
function ownTrail(draft: NeighbourhoodDraft, t: Templates, slot: number): number {
  const r = plotRect(t, draft.slots[slot]);
  const off = (q: Rect, [x, z]: Pt) => Math.hypot(Math.max(q.x - x, 0, x - q.x - q.w), Math.max(q.z - z, 0, z - q.z - q.d));
  return draft.roads.findIndex((road, i) => {
    if (road.kind !== 'trail') return false;
    const end = road.points[road.points.length - 1];
    if (off(r, end) > 6) return false;
    if (draft.slots.some((s) => s.index !== slot && off(plotRect(t, s), end) <= 6)) return false;
    return !draft.roads.some((o, j) => j !== i && [o.points[0], o.points[o.points.length - 1]].some(([x, z]) => Math.hypot(x - end[0], z - end[1]) < 3));
  });
}

/** Each draft's last lot being moved: the draft it is moved on (without an outdoor spot's own trail) and that trail's index (-1: none taken up). */
const moving = new WeakMap<NeighbourhoodDraft, { slot: number; base: NeighbourhoodDraft; trail: number }>();

/** The draft lot `slot` is moved on (`moveLot`, `canPlace`): `draft`, or without the outdoor spot's own trail (its index in `trail`). Kept per draft, for the pointer's checks. */
function moveBase(draft: NeighbourhoodDraft, t: Templates, slot: number): { base: NeighbourhoodDraft; trail: number } {
  const had = moving.get(draft);
  if (had?.slot === slot) return had;
  const trail = draft.slots[slot]?.kind === 'outdoor' ? ownTrail(draft, t, slot) : -1;
  const out = { slot, base: trail < 0 ? draft : { ...draft, roads: draft.roads.filter((_, i) => i !== trail) }, trail };
  moving.set(draft, out);
  return out;
}

// ---- what goes on a new lot ----------------------------------------------------------------------

/** The public places (not the outdoor ones) a lot of `frame` can be: the six that share one lot in the shipped content. */
export function publicChoices(t: Templates, frame: { width: number; depth: number } = t.plot): string[] {
  return t.parks.filter((p) => p.kind !== 'outdoor' && (p.lot ?? t.plot).width === frame.width && (p.lot ?? t.plot).depth === frame.depth).map((p) => p.id);
}

/** What `choice` puts on a new lot, drawn from `seed` (the same seed, the same house), or why it can't be. Its name comes later (`addLot`). */
function unplacedOf(draft: NeighbourhoodDraft, t: Templates, choice: LotChoice, seed: number): Unplaced | string {
  const rand = mulberry32(seed);
  const garden = (): GardenSize => (weighted(rand, specFor(draft).gardens as Record<string, number>) ?? 'roomy') as GardenSize;
  if (choice.kind === 'house') return { kind: 'house', name: '', lot: choice.size, ...houseFor(t, choice.size, choice.garden ?? garden(), rand) };
  if (choice.kind === 'farm') return t.lotSizes?.farm ? { kind: 'house', name: '', lot: 'farm', ...houseFor(t, 'farm', garden(), rand) } : 'This world has no farm lots.';
  const p = t.parks.find((q) => q.id === choice.template);
  if (!p) return `There is no public place "${choice.template}".`;
  const kind = p.kind === 'outdoor' ? 'outdoor' : p.kind === 'shop' ? 'shop' : 'park';
  if (kind !== choice.kind) return `${p.name} is not a ${choice.kind === 'outdoor' ? 'place out of doors' : choice.kind}.`;
  // (A public place on a lot of the frame it is drawn for, as `wantedLots` has one; an outdoor place on its own, as `macro.ts` `siteFor`.)
  const frame = kind === 'outdoor' ? (p.lot ?? t.plot) : t.plot;
  return { kind, template: p.id, name: p.name, lot: 'medium', w: frame.width, d: frame.depth };
}

/** A farm's name not yet in `draft` (`wantedLots`'s farm words). */
function farmName(draft: NeighbourhoodDraft, content: Content, rand: Rand): string {
  const taken = new Set(draft.slots.map((s) => s.name));
  const words = shuffled(content.names.worlds?.farm?.start ?? ['Meadow', 'Barley', 'Clover'], rand);
  const free = words.map((w) => `${w} Farm`).find((n) => !taken.has(n));
  if (free) return free;
  let k = 2;
  while (taken.has(`${words[0]} Farm ${k}`)) k++;
  return `${words[0]} Farm ${k}`;
}

// ---- the edits -------------------------------------------------------------------------------------

/**
 * Where a lot would go at road spot `at`, cheaply enough to ask on every pointer move (the ground is
 * made once per draft and kept): `what` is a new lot (`LotChoice`, its house drawn from `seed`, as
 * `addLot` draws it) or a lot of the draft being moved (`slot`). Its rectangle and the way it faces,
 * or why it can't go there. The first call for a new house or a new draft makes them.
 */
export function canPlace(draft: NeighbourhoodDraft, t: Templates, at: RoadSpot, what: LotChoice | { slot: number }, seed = 0): LotFit | { refused: string } {
  if ('slot' in what) {
    const s = draft.slots[what.slot];
    if (!s) return { refused: 'There is no such lot.' };
    if (s.lot === 'terrace') return { refused: 'A terraced house stays in its row.' };
    const { base, trail } = moveBase(draft, t, what.slot);
    if (at.road === trail) return { refused: 'A footpath has no lots along it.' };
    const fit = fitAt(base, t, trail >= 0 && at.road > trail ? { ...at, road: at.road - 1 } : at, s, what.slot);
    return 'refused' in fit ? fit : { x: fit.x, z: fit.z, w: fit.w, d: fit.d, facing: fit.facing };
  }
  const u = unplacedOf(draft, t, what, seed);
  if (typeof u === 'string') return { refused: u };
  const fit = fitAt(draft, t, at, u);
  return 'refused' in fit ? fit : { x: fit.x, z: fit.z, w: fit.w, d: fit.d, facing: fit.facing };
}

/**
 * `draft` with a lot of `choice` at `at`, its house drawn from `seed` (the same seed, the same
 * house): where `canPlace` says it fits. A home is numbered along its road (`renumber`), a farm gets
 * its fields and farmyard (`LotPlacer`) and a name, a public place its place (named, with hours), an
 * outdoor spot its place, its level and a short trail into it from the road.
 */
export function addLot(draft: NeighbourhoodDraft, t: Templates, content: Content, at: RoadSpot, choice: LotChoice, seed: number): Edited {
  const u0 = unplacedOf(draft, t, choice, seed);
  if (typeof u0 === 'string') return { refused: u0 };
  const fit = fitAt(draft, t, at, u0);
  if ('refused' in fit) return fit;
  const rand = mulberry32((seed ^ 0x5bd1e995) >>> 0);
  const area = areaAt(draft, at.road, fit.f.p);
  const u: Unplaced = { ...u0, area, ...(u0.lot === 'farm' ? { name: farmName(draft, content, rand) } : {}), ...(u0.kind === 'outdoor' ? { name: `${draft.open?.areas[area]?.name ?? draft.name} ${u0.name}` } : {}) };
  // (Put down by the generator's own placer: a farm's fields and farmyard drawn round its house.)
  const placer = new LotPlacer(content, t, townGround(draft, t), draft.roads, [0, 0], rand, true);
  const slot: PlotSlot = { ...placer.put(u, { x: fit.x, z: fit.z, facing: fit.facing, anchor: fit.anchor }, fit.f), index: draft.slots.length };
  let next: NeighbourhoodDraft = { ...draft, slots: [...draft.slots, slot] };
  if (slot.kind !== 'house') next = withLotPlace(next, t, content, slot.index);
  if (slot.kind === 'outdoor' && next.open) next = { ...next, roads: [...next.roads, trailInto(next, t, slot, fit.f)], open: { ...next.open, pads: { ...next.open.pads, [slot.index]: padOf(next, t, slot) } } };
  if (slot.kind === 'house') next = renumber(next, t, [at.road]);
  return { draft: next, cleared: NOTHING };
}

/**
 * `draft` with the lot on `slot` moved to `at`: taken up and put down again as `addLot` puts one,
 * keeping its house, garden, kind, farm and household (its slot index too). The houses along the
 * road it left and the road it went to are numbered again; a public place's tag goes with it; an
 * outdoor spot's own trail is taken up and a short one laid from its new road.
 */
export function moveLot(draft: NeighbourhoodDraft, t: Templates, slot: number, at: RoadSpot): Edited {
  const old = draft.slots[slot];
  if (!old) return { refused: 'There is no such lot.' };
  if (old.lot === 'terrace') return { refused: 'A terraced house stays in its row.' };
  const from = entriesOf(draft, t)[slot]?.road ?? -1;
  const { base, trail } = moveBase(draft, t, slot);
  if (at.road === trail) return { refused: 'A footpath has no lots along it.' };
  const to = trail >= 0 && at.road > trail ? at.road - 1 : at.road;
  const left = trail >= 0 ? -1 : from;
  const fit = fitAt(base, t, { ...at, road: to }, old, slot);
  if ('refused' in fit) return fit;
  const area = areaAt(base, to, fit.f.p);
  const moved: PlotSlot = { ...old, x: fit.x, z: fit.z, facing: fit.facing, anchor: fit.anchor, area };
  let next: NeighbourhoodDraft = { ...base, slots: base.slots.map((s) => (s.index === slot ? moved : s)) };
  next = placeFollows(next, t, slot);
  if (moved.kind === 'outdoor' && next.open) next = { ...next, roads: [...next.roads, trailInto(next, t, moved, fit.f)], open: { ...next.open, pads: { ...next.open.pads, [slot]: padOf(next, t, moved) } } };
  if (moved.kind === 'house') next = renumber(next, t, left >= 0 && left !== to ? [left, to] : [to]);
  return { draft: next, cleared: NOTHING };
}

/**
 * `draft` with homes along road `road`, drawn from `seed` (the same seed, the same houses): the
 * world's lot mix (`wantedLots`: sizes and gardens) at its gaps (`LotPlacer`), as many as its
 * frontage holds, put down by `LotPlacer.place` nearest its area's middle first, off the corners
 * and tight bends, none in a town centre. The houses along it are numbered. Refused if none fits.
 */
export function fillStreet(draft: NeighbourhoodDraft, t: Templates, content: Content, road: number, seed: number): Edited {
  const r = draft.roads[road];
  const no = noLotsOn(r, r?.lots ?? 1);
  if (no) return { refused: no };
  const spec = specFor(draft);
  const rand = mulberry32(seed);
  const ls = lengths(r.points);
  const area = areaAt(draft, road, pointAt(r.points, ls[ls.length - 1] / 2, ls).p);
  const centre = middleOf(draft, area);
  const frontage = frontageOf(draft.roads, [road], centre, rand).filter((f) => offCorners(draft.roads, road, f.s) && !inCentre(draft, [f.p[0] + f.n[0] * 10, f.p[1] + f.n[1] * 10]));
  const placer = new LotPlacer(content, t, townGround(draft, t), draft.roads, spec.gaps, rand, true);
  // A few homes at a time, until a batch finds no room at all (the frontage is full).
  const batch = { ...spec.town, lots: 4, places: [0, 0] as [number, number], farms: [0, 0] as [number, number] };
  for (let k = 0; frontage.length && k < 40; k++) {
    const before = placer.slots.length;
    placer.place(wantedLots(content, t, spec, batch, rand, { park: false }), frontage, [], centre, area, true);
    if (placer.slots.length === before) break;
  }
  if (!placer.slots.length) return { refused: 'There is no room for homes along it.' };
  const added = placer.slots.map((s, k): PlotSlot => ({ ...s, index: draft.slots.length + k }));
  return { draft: renumber({ ...draft, slots: [...draft.slots, ...added] }, t, [road]), cleared: NOTHING };
}

/** `draft` with house `template` on lot `slot` in the lot `h` cuts for it, its front kept where it was (as `withGarden` keeps it); refused where it doesn't fit there or no car could drive out. */
function refit(draft: NeighbourhoodDraft, t: Templates, slot: number, template: string): Edited {
  const old = draft.slots[slot];
  const h = houseTemplate(t, template);
  if (!h) return { refused: 'There is no such house.' };
  const generated = template.startsWith('gen:');
  const size = generated ? (template.split(':')[1] as LotSize) : (old.lot ?? 'medium');
  if (!generated && !fitsLot(t, h, size)) return { refused: 'That house does not fit a lot of this size.' };
  // (A generated house's lot is cut to it; a fixed layout's is its size's.)
  const s: PlotSlot = { ...old, template, lot: size };
  delete s.w;
  delete s.d;
  delete s.garden;
  if (h.lot) [s.w, s.d] = [h.lot.width, h.lot.depth];
  const garden = generated ? (template.split(':')[3] as GardenSize | undefined) : undefined;
  if (garden) s.garden = garden;
  const was = plotRect(t, old);
  const now = plotRect(t, s);
  // The front stays on the street, its middle where it was cut round (`anchor`), as `withGarden` cuts a lot again.
  const front = [was.z, was.x, was.z + was.d, was.x + was.w][old.facing];
  const anchor = old.anchor ?? (old.facing % 2 ? was.z + was.d / 2 : was.x + was.w / 2);
  const at = fronting(old.facing, front, anchor, now.w, now.d);
  if (!freeBeyond(groundOf(draft, t, slot), at, was)) return { refused: 'There is no room for it here.' };
  Object.assign(s, { x: at.x, z: at.z, anchor });
  if (!drivesOut(draft.roads, t, s, s)) return { refused: 'A car could not turn out of its driveway onto the road here.' };
  return { draft: { ...draft, slots: draft.slots.map((q) => (q.index === slot ? s : q)) }, cleared: NOTHING };
}

/**
 * Whether lot `r`, cut again from `was`, is clear (`Ground.free`, a metre inside the land's edge)
 * past what it took already: what was laid after a lot (a trail beside it, parking, an orchard)
 * takes the ground a little way round it, onto the lot itself.
 */
function freeBeyond(ground: Ground, r: Rect, was: Rect): boolean {
  if (r.x < 1 || r.z < 1 || r.x + r.w > ground.w - 1 || r.z + r.d > ground.d - 1) return false;
  for (let z = r.z; z < r.z + r.d; z++) {
    for (let x = r.x; x < r.x + r.w; x++) if (!(x >= was.x && x < was.x + was.w && z >= was.z && z < was.z + was.d) && ground.takenAt(x + 0.5, z + 0.5)) return false;
  }
  return true;
}

/**
 * `draft` with another house (`template`: a generated house's id, `generatedId`, or a fixed
 * layout's) on home `slot`, its lot cut for it with its front where it was, where it fits as
 * `withGarden` fits one and a car can drive out. Not a farm's or a terraced house's.
 */
export function anotherHouse(draft: NeighbourhoodDraft, t: Templates, slot: number, template: string): Edited {
  const old = draft.slots[slot];
  if (!old || old.kind !== 'house') return { refused: 'That is not a home.' };
  if (old.lot === 'farm') return { refused: 'A farm keeps its farmhouse.' };
  if (old.lot === 'terrace') return { refused: 'A terraced house stays in its row.' };
  if (old.template === template) return { refused: 'That house is there already.' };
  if (template.startsWith('gen:') && !['small', 'medium', 'large'].includes(template.split(':')[1])) return { refused: 'That house is not a home of its own.' };
  return refit(draft, t, slot, template);
}

/**
 * `draft` with the garden on home `slot` made `garden`: its lot cut again round the same house with
 * its front where it was (as `withGarden` cuts it), where that fits on the town's ground (the water,
 * parking and trails too) and a car can still drive out. A generated house's only; not a farm's or
 * a terraced house's.
 */
export function setGarden(draft: NeighbourhoodDraft, t: Templates, slot: number, garden: GardenSize): Edited {
  const old = draft.slots[slot];
  if (!old || old.kind !== 'house') return { refused: 'That is not a home.' };
  if (old.lot === 'farm') return { refused: "A farm's garden is its fields." };
  if (old.lot === 'terrace') return { refused: 'A terraced house stays in its row.' };
  if (!old.template.startsWith('gen:')) return { refused: 'This house has a garden of its own size.' };
  if (old.garden === garden) return { refused: 'It has that garden already.' };
  const cut = refit(draft, t, slot, withGardenId(old.template, garden));
  return 'refused' in cut && cut.refused === 'There is no room for it here.' ? { refused: 'There is no room for that garden here.' } : cut;
}

/** A generated house's id with another garden, for the garden choices a card offers (`generatedId`). */
export function withGardenId(template: string, garden: GardenSize): string {
  const [, size, seed] = template.split(':');
  return generatedId(size as LotSize, Number(seed), garden);
}

/** Whether a lot is a park (its place one: the town park, a playground; not a market). */
const isPark = (s: Pick<PlotSlot, 'kind' | 'template'>) => s.kind === 'park' && placeKindOf(s) === 'park';

/**
 * `draft` with the public lot on `slot` made another public place of its size (`template`, one of
 * `publicChoices`): its kind, template and name, and its place (named, with the hours and the
 * careers of its kind). The town's last park stays one.
 */
export function swapPlace(draft: NeighbourhoodDraft, t: Templates, content: Content, slot: number, template: string): Edited {
  const old = draft.slots[slot];
  if (!old || (old.kind !== 'park' && old.kind !== 'shop')) return { refused: 'That is not a public place.' };
  if (old.template === template) return { refused: 'It is that place already.' };
  const p = t.parks.find((q) => q.id === template);
  if (!p || !publicChoices(t, slotDims(t, old)).includes(template)) return { refused: 'That place does not go on a lot of this size.' };
  const s: PlotSlot = { ...old, kind: p.kind === 'shop' ? 'shop' : 'park', template, name: p.name };
  if (isPark(old) && !isPark(s) && !draft.slots.some((q) => q.index !== slot && isPark(q))) return { refused: 'The town keeps its last park.' };
  const next = { ...draft, slots: draft.slots.map((q) => (q.index === slot ? s : q)) };
  return { draft: withLotPlace(next, t, content, slot), cleared: NOTHING };
}

// ---- civic places (`backdrop.ts` `CIVIC`) ---------------------------------------------------------

/**
 * The civic places (`backdrop.ts` `CIVIC`), as step 2's cards know them: their indices are `CIVIC`'s.
 * (Passed on as it is, not read here as this module loads: `backdrop.ts` reaches this module again
 * through `ground.ts`, `macro.ts` and `landFollow.ts`, and may not have made `CIVIC` yet.)
 */
export { CIVIC as CIVIC_PLACES } from '../backdrop';

/** The civic places a civic building (`building`, in `open.backdrop`) may become: those its area doesn't have yet. */
export function civicSwaps(draft: NeighbourhoodDraft, building: number): { civic: number; label: string }[] {
  const b = draft.open?.backdrop?.[building];
  const places = draft.open?.places ?? [];
  if (!b || civicOf(places[b.place ?? -1]) < 0) return [];
  const has = new Set(places.filter((p) => p.area === b.area).map(civicOf));
  return CIVIC.flatMap((c, i) => (has.has(i) ? [] : [{ civic: i, label: c.label }]));
}

/**
 * `draft` with civic building `building` (its index in `open.backdrop`) made civic place `civic`
 * (an index into `CIVIC_PLACES`): its use, storeys and roof as that kind's (its footprint kept), and its
 * place's kind, name (drawn from `seed`, none twice), hours and careers. Careers the old place
 * took on beyond its own (the main town's catch-all office) go with it. Refused for a building that
 * is no civic place, or a civic place its area already has.
 */
export function swapCivic(draft: NeighbourhoodDraft, building: number, civic: number, content: Content, seed = 0): Edited {
  const open = draft.open;
  const b = open?.backdrop?.[building];
  const c = CIVIC[civic];
  if (!open?.backdrop || !open.places || !b || b.place === undefined) return { refused: 'That is not a civic building.' };
  if (!c) return { refused: 'There is no such civic place.' };
  const old = open.places[b.place];
  const was = civicOf(old);
  if (was < 0) return { refused: 'That is not a civic building.' };
  if (was === civic) return { refused: `It is a ${c.label.toLowerCase()} already.` };
  const area = open.areas[b.area];
  if (open.places.some((p, i) => i !== b.place && p.area === b.area && civicOf(p) === civic)) return { refused: `${area?.name ?? draft.name} has a ${c.label.toLowerCase()} already.` };
  // A name as the generator names one, none twice in the world (the lots' names among them).
  const rand = mulberry32(seed);
  const lastNames = content.names.last.length ? content.names.last : ['Smith'];
  const streets = draft.roads.filter((r) => r.area === b.area && r.name && !/Trail|Path/.test(r.name)).map((r) => r.name!);
  const taken = new Set([...open.places.filter((_, i) => i !== b.place).map((p) => p.name), ...draft.slots.map((s) => s.name)]);
  const namer = namerFor(rand, area?.name ?? draft.name, draft.world?.options?.town ?? 'village', lastNames, streets, taken);
  const name = namer.unique(() => c.name(namer));
  const extra = (old.work ?? []).filter((w) => !CIVIC[was].work.includes(w));
  const work = [...c.work, ...extra.filter((w) => !c.work.includes(w))];
  const place: TownPlace = { name, kind: c.place, x: old.x, z: old.z, hours: hoursFor(HOURS[c.place as keyof typeof HOURS], work, content.careerCategories), ...(work.length ? { work } : {}), area: b.area };
  // The footprint stays; the storeys and the roof are the new kind's (a ridge along the footprint's longest side if the roof wants one and had none).
  const { use, storeys: [fewest, most], roofs } = c.kind;
  const storeys = Math.min(most, Math.max(fewest, b.storeys));
  const roof = roofs.includes(b.roof) ? b.roof : roofs[0];
  const ridged = roof === 'gable' || roof === 'mansard';
  const built: BackdropBuilding = { ...b, use, storeys, roof, ...(ridged ? { ridge: b.ridge ?? ridgeOf(b.poly) } : {}) };
  if (!ridged) delete built.ridge;
  const backdrop = open.backdrop.map((q, i) => (i === building ? built : q));
  const places = open.places.map((p, i) => (i === b.place ? place : p));
  return { draft: { ...draft, open: { ...open, backdrop, places } }, cleared: NOTHING };
}

/** A ridge along a footprint's longest side (rad: a turn round +y, running along (cos r, -sin r)). */
function ridgeOf(poly: readonly [number, number][]): number {
  let [best, long] = [0, -1];
  poly.forEach((a, i) => {
    const b = poly[(i + 1) % poly.length];
    const l = Math.hypot(b[0] - a[0], b[1] - a[1]);
    if (l > long) [best, long] = [Math.atan2(-(b[1] - a[1]), b[0] - a[0]), l];
  });
  return Math.round(best * 1e4) / 1e4;
}

// ---- numbering and removal -------------------------------------------------------------------------

/**
 * `draft` with the houses along `roads` numbered again as `LotPlacer.number` numbers them: by
 * where each meets its road (`entriesOf`), odd on the road's left and even on its right, from its
 * start; farms keep their names. A road with no name is left as it is.
 */
export function renumber(draft: NeighbourhoodDraft, t: Templates, roads: readonly number[]): NeighbourhoodDraft {
  const spots = entriesOf(draft, t);
  const named = new Map<number, string>();
  for (const r of new Set(roads)) {
    const name = draft.roads[r]?.name;
    if (name === undefined) continue;
    const along = draft.slots.flatMap((s, i) => (s.kind === 'house' && s.lot !== 'farm' && spots[i].road === r ? [{ i, ...spots[i] }] : [])).sort((a, b) => a.s - b.s);
    for (const side of [1, -1] as const) along.filter((a) => a.side === side).forEach((a, k) => named.set(a.i, `${k * 2 + (side > 0 ? 1 : 2)} ${name}`));
  }
  if (![...named].some(([i, name]) => draft.slots[i].name !== name)) return draft;
  return { ...draft, slots: draft.slots.map((s, i) => (named.has(i) && named.get(i) !== s.name ? { ...s, name: named.get(i)! } : s)) };
}

/** Why the lots on `lots` (slot indices) may not all go from `draft`, or null: the town keeps its last home and its last park. */
export function removalRefused(draft: NeighbourhoodDraft, lots: ReadonlySet<number>): string | null {
  const left = draft.slots.filter((s) => !lots.has(s.index));
  if (!left.some((s) => s.kind === 'house') && draft.slots.some((s) => s.kind === 'house')) return 'The town keeps its last home.';
  if (!left.some(isPark) && draft.slots.some(isPark)) return 'The town keeps its last park.';
  return null;
}

/** `removeLots` as an edit: refused if it would take the town's last home or last park (`removalRefused`). */
export function removeLotsEdit(draft: NeighbourhoodDraft, lots: ReadonlySet<number>, backdrop: ReadonlySet<number> = new Set()): Edited {
  const why = removalRefused(draft, lots);
  if (why) return { refused: why };
  return { draft: removeLots(draft, lots, backdrop), cleared: { lots: [...lots].sort((a, b) => a - b), backdrop: [...backdrop].sort((a, b) => a - b) } };
}

/**
 * `draft` without the lots on `lots` (slot indices) and the backdrop buildings `backdrop` (their
 * indices in `open.backdrop`): the slots left numbered again from 0, and the households, pads,
 * places and backdrop buildings that name them with them; a household goes with its lot, a place
 * with its lot or its building. No refusal here (the slope limits' last resort uses it too:
 * `landFollow.ts` `settleLand`); the town edits use `removeLotsEdit`.
 */
export function removeLots(draft: NeighbourhoodDraft, lots: ReadonlySet<number>, backdrop: ReadonlySet<number> = new Set()): NeighbourhoodDraft {
  const open = draft.open;
  const kept = draft.slots.filter((s) => !lots.has(s.index));
  const renumber = new Map(kept.map((s, i) => [s.index, i]));
  const slots = kept.map((s, i) => ({ ...s, index: i }));
  const households = draft.households.filter((h) => renumber.has(h.slot)).map((h) => (h.slot === renumber.get(h.slot) ? h : { ...h, slot: renumber.get(h.slot)! }));
  if (!open) return { ...draft, slots, households };
  const pads: Record<number, number> = {};
  for (const [k, v] of Object.entries(open.pads)) if (renumber.has(Number(k))) pads[renumber.get(Number(k))!] = v;
  const places = open.places ?? [];
  const placeGone = new Set<number>();
  places.forEach((q, i) => {
    if (q.plot !== undefined && !renumber.has(q.plot)) placeGone.add(i);
  });
  (open.backdrop ?? []).forEach((b, i) => {
    if (backdrop.has(i) && b.place !== undefined) placeGone.add(b.place);
  });
  const placeNumber = new Map<number, number>();
  places.forEach((_, i) => !placeGone.has(i) && placeNumber.set(i, placeNumber.size));
  const keptPlaces = places.filter((_, i) => !placeGone.has(i)).map((q) => (q.plot === undefined ? q : { ...q, plot: renumber.get(q.plot)! }));
  const keptBackdrop = (open.backdrop ?? []).filter((_, i) => !backdrop.has(i)).map((b) => (b.place === undefined ? b : { ...b, place: placeNumber.get(b.place)! }));
  return { ...draft, slots, households, open: { ...open, pads, ...(open.places ? { places: keptPlaces } : {}), ...(open.backdrop ? { backdrop: keptBackdrop } : {}) } };
}
