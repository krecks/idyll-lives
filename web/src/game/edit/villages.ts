/**
 * Step 2's village edits: a village placed by hand, moved, or removed with everything in it.
 *
 * A village placed by hand is a hamlet of the place's own kind, laid as the generator lays one
 * (decision 14):
 * - its site is tested as the generator tests a hamlet's (`macro.ts` `hamletFits`: in the world, dry,
 *   clear of the other areas' boxes by 24 m, not too steep, clear of the bridges), its middle
 *   `AREA_APART` m from the other areas' (as the town checks want), with no road, lot or building
 *   already standing there;
 * - its streets are the place's hamlet shapes (`hamletOf`: `joinUp`, `settleRoads`) put on the world
 *   as the generator puts one (`putHamlet`);
 * - a country road is routed from one of its ways out (its ports) to the country roads already laid,
 *   by the generator's own router (`countryRouter.ts`), sharing a road where it runs along one as the
 *   generator's do (`CountryRouter.course`); its other ways out end inside its box, as the generator
 *   ends a port nobody took;
 * - its lots are the hamlet's own (`hamletLots`: the place's lot mix by `LotPlacer`), a public lot
 *   gets its place, the homes are numbered, and two parking spaces go by the verge nearest its middle;
 * - it and its streets are named, with names nobody in town has.
 * Where no road can be routed to it, it is placed all the same and the town's checks say it has no
 * way out ("No road to Ashby" in the Problems list), for the player to draw one. It gets no backdrop
 * (decision 15). Every random choice is drawn from the edit's seed: the same draft, place and seed
 * give the same village.
 *
 * A village moved is translated by whole metres: its streets, the lots fronting them (with their
 * places, pads and orchards), its backdrop buildings, its parking and its greens. Its site is tested
 * again (its own old things aside). The country roads that led to it are taken up where no lot
 * stands on them and the rest of the roads stay one network; those with homes on them are left
 * as dead ends, cut back past their last lot, and said in the edit's `notes`. A road is routed again
 * from where its streets were met (else from a dead end of theirs by the edge of its box).
 *
 * A village removed takes its streets, the lots fronting them, its backdrop buildings and their
 * places, its parking and greens, the country roads that led only to it (as a move takes them up),
 * and its area: the area indexes on everything else are numbered again (`removeArea`). It is refused
 * where the town would be left fewer than two homes, its last park, or no road out of the world, or
 * where another area reaches the roads only through it.
 */

import type { Content } from '../../content/content';
import type { RoadPlan, TownArea } from '../../core/protocol';
import { along, clearOfBoxes, CountryRouter, TRIES, type Cells, type Onward, type Port, type Way } from '../countryRouter';
import { heightAt, type Land } from '../land';
import { AREA_APART, cells8, cornersOf, hamletFits, hamletLots, hamletOf, parkingFor, putHamlet, type AreaPlan } from '../macro';
import { mulberry32 } from '../noise';
import { COURT, endDir, evenly, leftOf, lengths, nearest, pointAt, RoadIndex, roadIndex, tangent, type Pt } from '../roads';
import { boundsOf, frontageOf, LotPlacer, plotRect, type Facing, type NeighbourhoodDraft, type PlotSlot, type Rect, type Templates } from '../town';
import { placeOf, specOf, streetNames, type WorldSpec } from '../world';
import type { Edited } from './edited';
import { entriesOf, townGround } from './ground';
import { removalRefused, removeLots, renumber, withLotPlace } from './lots';

/** Where a village is placed (its box's middle, m), whether its shape is turned across, and the seed its streets, lots and names are drawn from. */
export interface VillageAt {
  x: number;
  z: number;
  swap: boolean;
  seed: number;
}

/** A village's box on the town (m), where it would go. */
export type VillageBox = Rect;

/** What removing a village takes: how many of its own roads and of the country roads that led to it, the lots and backdrop buildings (indices in the draft it is asked of), how many parking spaces; or why it may not go. */
export type VillageGoes = { roads: number; others: number; lots: number[]; backdrop: number[]; parking: number } | { refused: string };

// ---- the generator's measures ------------------------------------------------------------------

/** A hamlet's box keeps this far inside the world's edge, and this far from another area's box (m): `macro.ts` `EDGE`, `BOX_GAP` (`hamletFits` tests both; these only say which it was). */
const EDGE = 50;
const BOX_GAP = 24;
/** Nothing already standing (a road, a lot, a building) within this of a village's box (m): its streets reach its edges. */
const WAY = 8;
/** A road on from a port meets what lies within this of straight on before what lies to the side, which costs it more (`countryRouter.ts` `AHEAD`, `ASIDE`). */
const AHEAD = Math.PI / 3;
const ASIDE = 30;
/** What `CountryRouter.onward` counts against a road too steep to lay (`STEEP_FAULT`). */
const STEEP_FAULT = 1000;
/** A dead end left of an old country road reaches this far past its last lot (m). */
const STUB_PAST = 10;
/** The fewest lots a hamlet has (`townCheck.ts`: a hamlet of fewer is a fault). */
const FEWEST_LOTS = 4;

const NOTHING: { lots: number[]; backdrop: number[] } = { lots: [], backdrop: [] };
const r2 = (v: number) => Math.round(v * 100) / 100;
const dist = (a: Pt, b: Pt) => Math.hypot(a[0] - b[0], a[1] - b[1]);
const middleOf = (r: Rect): Pt => [r.x + r.w / 2, r.z + r.d / 2];
const rectOf = (a: TownArea): Rect => ({ x: a.x, z: a.z, w: a.w, d: a.d });
const meets = (a: Rect, b: Rect) => a.x < b.x + b.w && b.x < a.x + a.w && a.z < b.z + b.d && b.z < a.z + a.d;
const grow = (r: Rect, by: number): Rect => ({ x: r.x - by, z: r.z - by, w: r.w + 2 * by, d: r.d + 2 * by });
const insideRect = (r: Rect, [x, z]: Pt) => x >= r.x && x <= r.x + r.w && z >= r.z && z <= r.z + r.d;
const offRect = (r: Rect, [x, z]: Pt) => Math.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d));
const middleOfPoly = (poly: readonly Pt[]): Pt => middleOf(boundsOf(poly));

// ---- what a draft's world is ---------------------------------------------------------------------

/** Each world's spec, made once. */
const specs = new WeakMap<object, WorldSpec>();
const PLAIN = {};
function specFor(draft: NeighbourhoodDraft): WorldSpec {
  const key = draft.world ?? PLAIN;
  let spec = specs.get(key);
  if (!spec) specs.set(key, (spec = specOf(placeOf(draft.world?.kind ?? 'country'), draft.world?.options)));
  return spec;
}

/** Each land's 8 m cells (`cells8`), as the generator and its router read it: made once per land. */
const cellsKept = new WeakMap<Land, Cells>();
function cellsFor(land: Land): Cells {
  let cells = cellsKept.get(land);
  if (!cells) cellsKept.set(land, (cells = cells8(land)));
  return cells;
}

/** Every name in town: its roads', its places', its areas' and its own. */
function namesIn(draft: NeighbourhoodDraft): Set<string> {
  const open = draft.open;
  return new Set([draft.name, ...draft.roads.map((r) => r.name ?? ''), ...(open?.places ?? []).map((p) => p.name), ...(open?.areas ?? []).map((a) => a.name)].filter(Boolean));
}

/** A name for a country road to `place` nobody in town has (as the generator names one: "Ashby Road", else "Ashby Lane"), taken. */
function roadName(place: string, taken: Set<string>): string {
  let name = [`${place} Road`, `${place} Lane`].find((n) => !taken.has(n));
  for (let k = 2; !name; k++) if (!taken.has(`${place} Road ${k}`)) name = `${place} Road ${k}`;
  taken.add(name);
  return name;
}

// ---- roads joined as the sim joins them ------------------------------------------------------------

/** Whether an end at `p` of road `self` (width `w`) touches another road: within half the wider one's width and a metre of its middle. */
function touches(roads: readonly RoadPlan[], index: RoadIndex, p: Pt, self: number, w: number): boolean {
  for (const [j, m] of index.around(p[0], p[1], 12)) if (j !== self && m.dist <= Math.max(roads[j].width, w) / 2 + 1) return true;
  return false;
}

/** Each road's network (a road's index stands for it), but those `without`: roads joined as the sim joins them. */
function networks(roads: readonly RoadPlan[], without: ReadonlySet<number>): Map<number, number> {
  const parent = roads.map((_, i) => i);
  const find = (i: number): number => (parent[i] === i ? i : (parent[i] = find(parent[i])));
  const index = roadIndex(roads);
  roads.forEach((r, i) => {
    if (without.has(i)) return;
    for (const end of [r.points[0], r.points[r.points.length - 1]]) {
      for (const [j, m] of index.around(end[0], end[1], 12)) if (j !== i && !without.has(j) && m.dist <= Math.max(roads[j].width, r.width) / 2 + 1) parent[find(i)] = find(j);
    }
  });
  return new Map(roads.flatMap((_, i) => (without.has(i) ? [] : [[i, find(i)] as [number, number]])));
}

/** Whether roads `a` and `b` meet: an end of one touches the other. */
function joined(roads: readonly RoadPlan[], a: number, b: number): boolean {
  const ends = (r: RoadPlan) => [r.points[0], r.points[r.points.length - 1]];
  const [ra, rb] = [roads[a], roads[b]];
  const reach = Math.max(ra.width, rb.width) / 2 + 1;
  return ends(ra).some(([x, z]) => nearest(rb.points, x, z).dist <= reach) || ends(rb).some(([x, z]) => nearest(ra.points, x, z).dist <= reach);
}

/** `roads` with every joined end that joins nothing any more a dead end (but those of roads `keep`). */
function deadEnds(roads: readonly RoadPlan[], keep: ReadonlySet<number> = new Set()): RoadPlan[] {
  const index = new RoadIndex(roads);
  return roads.map((r, i) => {
    if (keep.has(i)) return r;
    const ends = r.ends.map((e, k) => (e === 'join' && !touches(roads, index, k ? r.points[r.points.length - 1] : r.points[0], i, r.width) ? 'dead' : e)) as RoadPlan['ends'];
    return ends[0] === r.ends[0] && ends[1] === r.ends[1] ? r : { ...r, ends };
  });
}

/** Whether village roads `own` are on a network with a road out of the world on it. */
function reaches(roads: readonly RoadPlan[], own: Iterable<number>): boolean {
  const nets = networks(roads, new Set());
  const out = new Set(roads.flatMap((r, i) => (r.ends.includes('out') ? [nets.get(i)] : [])));
  for (const i of own) if (out.has(nets.get(i))) return true;
  return false;
}

// ---- a village's things ---------------------------------------------------------------------------

/** What belongs to a village (area `area`), by index in the draft: its roads (streets, lanes, trails), the lots fronting them, its backdrop buildings, parking, greens, the orchards by its farms; and the country roads that meet its roads (`links`). */
interface Parts {
  area: number;
  roads: Set<number>;
  lots: Set<number>;
  backdrop: Set<number>;
  parking: Set<number>;
  greens: Set<number>;
  orchards: Set<number>;
  links: Set<number>;
}

/** Each draft's villages' parts, worked out once (the pointer asks again and again while one is dragged). */
const partsKept = new WeakMap<NeighbourhoodDraft, Map<number, Parts>>();
function partsOf(draft: NeighbourhoodDraft, t: Templates, area: number): Parts {
  let byArea = partsKept.get(draft);
  if (!byArea) partsKept.set(draft, (byArea = new Map()));
  let parts = byArea.get(area);
  if (parts) return parts;
  const open = draft.open;
  const box = open?.areas[area] ? rectOf(open.areas[area]) : { x: 0, z: 0, w: 0, d: 0 };
  const roads = new Set(draft.roads.flatMap((r, i) => (r.area === area ? [i] : [])));
  const spots = entriesOf(draft, t);
  const lots = new Set(draft.slots.flatMap((s, i) => (roads.has(spots[i]?.road ?? -1) ? [s.index] : [])));
  const farms = draft.slots.filter((s) => lots.has(s.index) && s.lot === 'farm').map((s) => grow(plotRect(t, s), 4));
  const links = new Set<number>();
  draft.roads.forEach((r, i) => {
    if (roads.has(i) || r.kind !== 'country' || r.area !== undefined) return;
    for (const v of roads) if (draft.roads[v].kind !== 'trail' && joined(draft.roads, i, v)) links.add(i);
  });
  // (Its buildings: those whose nearest road is its own or one into it; a farm's barn out along a country road stays with that road.)
  const index = roadIndex(draft.roads);
  const fronts = (poly: readonly Pt[]) => {
    const [x, z] = middleOfPoly(poly);
    const near = index.nearest(x, z, 40);
    return near ? roads.has(near.road) || links.has(near.road) : insideRect(box, [x, z]);
  };
  parts = {
    area,
    roads,
    lots,
    backdrop: new Set((open?.backdrop ?? []).flatMap((b, i) => (b.area === area && fronts(b.poly) ? [i] : []))),
    parking: new Set((open?.parking ?? []).flatMap((p, i) => (p.area === area ? [i] : []))),
    greens: new Set(draft.greens.flatMap((g, i) => (g.length && insideRect(box, middleOfPoly(g)) ? [i] : []))),
    orchards: new Set((open?.zones ?? []).flatMap((z, i) => (z.kind === 'orchard' && z.poly.length && farms.some((f) => meets(f, boundsOf(z.poly))) ? [i] : []))),
    links,
  };
  byArea.set(area, parts);
  return parts;
}

// ---- the site --------------------------------------------------------------------------------------

/** What a site test leaves aside: a village's own things (one being moved). */
interface Aside {
  area: number;
  roads: ReadonlySet<number>;
  lots: ReadonlySet<number>;
  backdrop: ReadonlySet<number>;
}

/** Each set of roads' bounds (their width round them), for the test of what stands where a village would go. */
const boundsKept = new WeakMap<readonly RoadPlan[], Rect[]>();
function roadBounds(roads: readonly RoadPlan[]): Rect[] {
  let out = boundsKept.get(roads);
  if (!out) boundsKept.set(roads, (out = roads.map((r) => grow(r.points.length ? boundsOf(r.points) : { x: 0, z: 0, w: 0, d: 0 }, r.width / 2))));
  return out;
}

/** What stands on `rect` (and `WAY` m round it) but what is `aside`, in words; null: nothing. */
function standing(draft: NeighbourhoodDraft, t: Templates, rect: Rect, aside?: Aside): string | null {
  const r = grow(rect, WAY);
  const bounds = roadBounds(draft.roads);
  for (const [i, road] of draft.roads.entries()) {
    if (aside?.roads.has(i) || !meets(bounds[i], r)) continue;
    const half = road.width / 2;
    const pts = road.points;
    for (let k = 0; k < pts.length; k++) {
      const [a, b] = [pts[k], pts[Math.min(pts.length - 1, k + 1)]];
      const n = Math.max(1, Math.ceil(dist(a, b) / 2));
      for (let j = 0; j <= n; j++) if (offRect(r, [a[0] + ((b[0] - a[0]) * j) / n, a[1] + ((b[1] - a[1]) * j) / n]) <= half) return `${road.name ?? 'a road'} runs through there`;
    }
  }
  const lots = draft.slots.filter((s) => !aside?.lots.has(s.index) && meets(plotRect(t, s), r)).length;
  if (lots) return `${lots === 1 ? 'a lot stands' : `${lots} lots stand`} there`;
  const buildings = (draft.open?.backdrop ?? []).filter((b, i) => !aside?.backdrop.has(i) && meets(boundsOf(b.poly), r)).length;
  if (buildings) return `${buildings === 1 ? 'a building stands' : `${buildings} buildings stand`} there`;
  return null;
}

/** The rectangles of what stands on `draft` (its lots, buildings and parking spaces) but a village's own (`parts`: one being moved, inside its box, which the router keeps out of but by its ways out). */
function thingsOf(draft: NeighbourhoodDraft, t: Templates, parts?: Parts): Rect[] {
  const space = (p: { x: number; z: number; rot: number }): Rect => (p.rot % 2 ? { x: p.x - 3, z: p.z - 1.5, w: 6, d: 3 } : { x: p.x - 1.5, z: p.z - 3, w: 3, d: 6 });
  return [
    ...draft.slots.flatMap((s) => (parts?.lots.has(s.index) ? [] : [plotRect(t, s)])),
    ...(draft.open?.backdrop ?? []).flatMap((b, i) => (parts?.backdrop.has(i) ? [] : [boundsOf(b.poly)])),
    ...(draft.open?.parking ?? []).flatMap((p, i) => (parts?.parking.has(i) ? [] : [space(p)])),
  ];
}

/**
 * Whether a village's box `bw` × `bd` may go round `middle` on `draft` (its own things `aside`, one
 * being moved): the generator's hamlet test (`hamletFits`), the other areas' middles `AREA_APART`
 * apart, nothing standing there. Its box (whole metres), or why not.
 */
function siteOf(draft: NeighbourhoodDraft, t: Templates, middle: Pt, [bw, bd]: readonly [number, number], aside?: Aside): { rect: Rect } | { refused: string } {
  const open = draft.open;
  const land = open?.land;
  if (!open || !land) return { refused: 'this town has no land a village can go on' };
  const { width: W, depth: D } = draft;
  const rect = { x: Math.round(middle[0] - bw / 2), z: Math.round(middle[1] - bd / 2), w: bw, d: bd };
  if (rect.x < EDGE || rect.z < EDGE || rect.x + bw > W - EDGE || rect.z + bd > D - EDGE) return { refused: `it would reach within ${EDGE} m of the edge of the world` };
  const others = open.areas.filter((_, k) => k !== aside?.area);
  for (const a of others) if (meets(grow(rectOf(a), BOX_GAP), rect)) return { refused: `it would come within ${BOX_GAP} m of ${a.name}` };
  const mid = middleOf(rect);
  for (const a of others) {
    const d = dist(middleOf(rectOf(a)), mid);
    if (d < AREA_APART) return { refused: `its middle would be ${Math.round(d)} m from ${a.name}'s (${AREA_APART} m at least)` };
  }
  const crossings = draft.roads.filter((r) => r.bridge).map((r): Pt[] => [r.points[0], r.points[r.points.length - 1]]);
  const cells = cellsFor(land);
  const site = { water: specFor(draft).water, boxes: others.map(rectOf), crossings };
  if (!hamletFits(cells, mid, [bw, bd], site)) {
    if (crossings.some((ends) => ends.some((q) => offRect(rect, q) < 50))) return { refused: 'it would be too near a bridge' };
    return { refused: hamletFits(cells, mid, [bw, bd], { ...site, relax: 1 }) ? 'the land there is too steep for a village' : 'it would stand in the water or too near it' };
  }
  const way = standing(draft, t, rect, aside);
  return way ? { refused: way } : { rect };
}

/** The box (`w` × `d`) of a hamlet of `draft`'s place, turned across if `swap`. */
export function villageSize(draft: NeighbourhoodDraft, swap: boolean): [number, number] {
  const hb = specFor(draft).hamlet.box;
  return swap ? [hb.d, hb.w] : [hb.w, hb.d];
}

/**
 * Where a village placed at `at` (its middle, and whether it is turned across) would go on `draft`:
 * its box, or why it may not go there. Cheap enough for every pointer move (under 5 ms: the land's
 * cells are made once per land, the roads' bounds once per set of roads).
 */
export function canPlaceVillage(draft: NeighbourhoodDraft, t: Templates, at: Pick<VillageAt, 'x' | 'z' | 'swap'>): VillageBox | { refused: string } {
  const site = siteOf(draft, t, [at.x, at.z], villageSize(draft, at.swap));
  return 'refused' in site ? site : site.rect;
}

/** What a move of a village leaves aside when its new site is tested: its roads, lots and buildings, and the country roads that met it. */
const asideOf = (parts: Parts): Aside => ({ area: parts.area, roads: new Set([...parts.roads, ...parts.links]), lots: parts.lots, backdrop: parts.backdrop });

/** Why area `area` is no village that may be moved or removed, or null. */
function notVillage(draft: NeighbourhoodDraft, area: number): string | null {
  const areas = draft.open?.areas ?? [];
  if (!Number.isInteger(area) || area < 0 || area >= areas.length) return 'there is no such village';
  if (area === 0) return 'the main town is moved by Move the town';
  return null;
}

/**
 * Where village `area` moved by (`dx`, `dz`) (whole metres) would go on `draft`: its box there, or
 * why not. Cheap enough for every pointer move: what is the village's is worked out once per draft.
 */
export function canMoveVillage(draft: NeighbourhoodDraft, t: Templates, area: number, dx: number, dz: number): VillageBox | { refused: string } {
  const no = notVillage(draft, area);
  if (no) return { refused: no };
  const a = draft.open!.areas[area];
  const site = siteOf(draft, t, [a.x + a.w / 2 + Math.round(dx), a.z + a.d / 2 + Math.round(dz)], [a.w, a.d], asideOf(partsOf(draft, t, area)));
  return 'refused' in site ? site : site.rect;
}

// ---- the road to it ----------------------------------------------------------------------------------

/**
 * The country road on from port `q` to the network: as `CountryRouter.onward` lays one, but only to
 * a country road already laid out in the country (not a bridge), never to the world's edge (a road
 * out of the world from a village alone would be a network of its own). Null if there is none, or
 * every way is too steep.
 */
function toNetwork(router: CountryRouter, q: Port, blocked: Uint8Array): Onward | null {
  const { cells: land, roads } = router;
  const { gw, gd } = land;
  const plain = (p: Pt) => clearOfBoxes(router.areas, p, 30);
  const country = new Map<number, { road: number; t: Pt }>();
  roads.forEach((r, i) => {
    if (r.kind === 'country' && !r.bridge && i !== q.road) r.points.forEach((p, j) => plain(p) && country.set(land.cell(p[0], p[1]), { road: i, t: tangent(r.points, j) }));
  });
  if (!country.size) return null;
  const from: Way = { p: q.p, dir: q.dir, port: q };
  const inFront = (p: Pt) => q.dir[0] * (p[0] - q.p[0]) + q.dir[1] * (p[1] - q.p[1]) >= dist(p, q.p) * Math.cos(AHEAD);
  const banned = new Set<number>();
  // A country road met square to it (give or take an eighth of a turn); what lies off to the side or behind the port costs more.
  const goal = (i: number, h?: number) => {
    const on = country.get(i);
    if (banned.has(i) || !on || (h !== undefined && Math.abs(along(h, on.t)) >= 0.75)) return -1;
    return inFront(land.centre(i)) ? 0 : ASIDE;
  };
  // (Not over a lot, a building or a parking space: the generator lays its roads before them; by the port, as it may.)
  const free = (cost: (i: number) => number) => (i: number) => (blocked[i] && dist(land.centre(i), q.p) > 12 ? Infinity : cost(i));
  const costs = [free(router.cost([q.p])), free(router.relaxed([q.p]))] as const;
  let made: Onward['made'] | null = null;
  for (let tries = 0; tries < TRIES && (!made || made.bad > 0); tries++) {
    const cells = router.route([from], goal, () => 0, undefined, costs)?.cells;
    const end = cells?.[cells.length - 1];
    if (!cells || end === undefined) break;
    // Into the other road square to it, ending on its middle (met from the side the way comes from).
    const other = country.get(end)!.road;
    const c = cells.length >= 3 ? land.centre(cells[cells.length - 3]) : q.p;
    const pts = roads[other].points;
    const at = nearest(pts, c[0], c[1]);
    const { p: T, t: tq } = pointAt(pts, at.s);
    const n = leftOf(tq);
    const b: Way = { p: T, dir: [n[0] * at.side, n[1] * at.side] };
    const course = cells.slice(0, -1);
    const road = router.road(from, course, b, ['join', 'join'], '');
    const bad = router.fault(road) + (router.tooSteep(road.points) ? STEEP_FAULT : 0);
    if (!made || bad < made.bad) made = { road, bad, cells: course, b, side: -1 };
    // (Not there again.)
    const [ex, ez] = [end % gw, Math.floor(end / gw)];
    for (let dz = -3; dz <= 3; dz++) for (let dx = -3; dx <= 3; dx++) if (ex + dx >= 0 && ez + dz >= 0 && ex + dx < gw && ez + dz < gd) banned.add((ez + dz) * gw + ex + dx);
  }
  return made && made.bad < STEEP_FAULT ? { made, from } : null;
}

/**
 * Port `q` of a village (box `rect`) settled: a joined end where a road meets it, else a dead end
 * cut back inside its box (10 m in), not back past where another road joins it, as the generator
 * ends a port nobody took. Changes `roads` (the port's road replaced).
 */
function settlePort(roads: RoadPlan[], q: Port, rect: Rect): void {
  const road = roads[q.road];
  const p = q.end ? road.points[road.points.length - 1] : road.points[0];
  const used = touches(roads, new RoadIndex(roads), p, q.road, road.width);
  const ends: RoadPlan['ends'] = q.end === 0 ? [used ? 'join' : 'dead', road.ends[1]] : [road.ends[0], used ? 'join' : 'dead'];
  if (used) {
    roads[q.road] = { ...road, ends };
    return;
  }
  const within = ([x, z]: Pt) => x > rect.x + 10 && z > rect.z + 10 && x < rect.x + rect.w - 10 && z < rect.z + rect.d - 10;
  const pts = q.end === 0 ? [...road.points].reverse() : [...road.points];
  let keep = pts.length;
  while (keep > 2 && !within(pts[keep - 1])) keep--;
  // (Not back past where another road joins it: a road laid off it in a T would be left on its own.)
  for (const [i, o] of roads.entries()) {
    if (i === q.road) continue;
    for (const e of [o.points[0], o.points[o.points.length - 1]]) {
      for (let k = pts.length - 1; k >= keep; k--) {
        if (dist(pts[k], e) > Math.max(o.width, road.width) / 2 + 1) continue;
        keep = k + 1;
        break;
      }
    }
  }
  const cut = pts.slice(0, keep);
  roads[q.road] = { ...road, ends, points: q.end === 0 ? cut.reverse() : cut };
  // (A court no longer than the generator's: `COURT` m past the last road meeting it.)
  const met = Math.max(0, ...meetings(roads, q.road, q.end));
  roads[q.road] = cutTo(roads[q.road], q.end, met + COURT - 1);
}

/** Where the other roads' ends meet road `i` (and its own joined far end), as arc lengths from its end other than `end`. */
function meetings(roads: readonly RoadPlan[], i: number, end: 0 | 1): number[] {
  const r = roads[i];
  const ls = lengths(r.points);
  const L = ls[ls.length - 1];
  const out = r.ends[1 - end] === 'join' ? [0] : [];
  roads.forEach((o, j) => {
    if (j === i) return;
    for (const e of [o.points[0], o.points[o.points.length - 1]]) {
      const at = nearest(r.points, e[0], e[1], ls);
      if (at.dist <= Math.max(o.width, r.width) / 2 + 1) out.push(end === 1 ? at.s : L - at.s);
    }
  });
  return out;
}

/** Road `r` with its end `end` cut back so it is `keep` m long from its other end (whole, if it is that short already, or the cut would leave under 8 m). */
function cutTo(r: RoadPlan, end: 0 | 1, keep: number): RoadPlan {
  const ls = lengths(r.points);
  const L = ls[ls.length - 1];
  if (keep >= L - 1 || keep < 8) return r;
  const [a, b] = end === 1 ? [0, keep] : [L - keep, L];
  const pts: Pt[] = [pointAt(r.points, a, ls).p];
  for (let k = 0; k < r.points.length; k++) if (ls[k] > a + 0.01 && ls[k] < b - 0.01) pts.push(r.points[k]);
  pts.push(pointAt(r.points, b, ls).p);
  return { ...r, points: pts.map(([x, z]): Pt => [r2(x), r2(z)]) };
}

/**
 * `roads` (each from road `from[i]` of `draft`; past its end, or -1, a new one) with the other
 * areas' streets and lanes (not village `area`'s) left ending in a dead end where a road that went
 * met them (village `place` removed or moved) held to `COURT` m past the last road they meet, as a
 * drawn street's dead end is (`streets.ts`): cut back to that, and said in the notes. `kept`: per
 * road cut back (by its index in `draft`), the stretch of it kept (m along it as it was). Refused,
 * the reason opening with `why`, where a lot stands on the stretch that would be cut off (its way
 * in there: `lotsOn`, the lots left on a road of `draft`). (A country road left so is `tidy`'s.)
 */
function capCourts(draft: NeighbourhoodDraft, roads: readonly RoadPlan[], from: readonly number[], lotsOn: (road: number) => readonly Rect[], place: string, area: number, why: string): { roads: RoadPlan[]; kept: Map<number, [number, number]>; notes: string[] } | { refused: string } {
  const out = roads.slice();
  const kept = new Map<number, [number, number]>();
  const notes: string[] = [];
  for (const [i, r] of roads.entries()) {
    const old = from[i] ?? -1;
    if (old < 0 || r.kind === 'country' || r.kind === 'trail' || r.area === area) continue;
    for (const end of [0, 1] as const) {
      if (draft.roads[old].ends[end] !== 'join' || out[i].ends[end] !== 'dead') continue;
      const ls = lengths(out[i].points);
      const L = ls[ls.length - 1];
      const last = Math.max(0, ...meetings(out, i, end));
      if (L - last <= COURT + 0.5) continue;
      // (Kept from its other end: `last` and a court's length.)
      const keep = last + COURT - 1;
      const was = lengths(draft.roads[old].points);
      const lot = lotsOn(old).find((q) => {
        const at = nearest(draft.roads[old].points, q.x + q.w / 2, q.z + q.d / 2, was);
        return (end === 1 ? at.s : was[was.length - 1] - at.s) > keep;
      });
      const what = r.name ?? 'a street';
      if (lot) return { refused: `${why}, ${what} would end ${Math.round(L - last)} m past the last road it meets, with homes on it past a dead end's ${COURT} m` };
      out[i] = cutTo(out[i], end, keep);
      const Lw = was[was.length - 1];
      const [a, b] = kept.get(old) ?? [0, Lw];
      kept.set(old, end === 1 ? [a, Math.min(b, a + keep)] : [Math.max(a, b - keep), b]);
      notes.push(`${what} is cut back to a dead end where ${place} was`);
    }
  }
  return { roads: out, kept, notes };
}

/**
 * A road laid from one of a village's ports (`ports`, in box `rect`, the village named `place`) to
 * the network (`toNetwork`), the port nearest a country road first, its pieces named; every port
 * then settled (`settlePort`). The router keeps out of `areas`' boxes (the village's among them).
 * Changes `roads` (added to; the ports' roads replaced). Whether a road was laid.
 */
function linkUp(draft: NeighbourhoodDraft, roads: RoadPlan[], areas: readonly Rect[], ports: readonly Port[], rect: Rect, place: string, taken: Set<string>, things: readonly Rect[]): boolean {
  const cells = cellsFor(draft.open!.land!);
  const router = new CountryRouter(cells, areas.map((r) => ({ rect: r })), roads, specFor(draft));
  // The 8 m cells the lots, buildings and parking stand on, which a road keeps off.
  const blocked = new Uint8Array(cells.gw * cells.gd);
  for (const r of things) {
    for (let cz = Math.max(0, Math.floor((r.z - 2) / 8)); cz <= Math.min(cells.gd - 1, Math.floor((r.z + r.d + 2) / 8)); cz++) {
      for (let cx = Math.max(0, Math.floor((r.x - 2) / 8)); cx <= Math.min(cells.gw - 1, Math.floor((r.x + r.w + 2) / 8)); cx++) blocked[cz * cells.gw + cx] = 1;
    }
  }
  /** Whether a road laid runs over one of `things` (its width and a metre and a half round it). */
  const clashes = (r: RoadPlan) => evenly(r.points, 1).some((p) => things.some((q) => offRect(q, p) < r.width / 2 + 1.5));
  // (How near a country road each port is.)
  const country = new RoadIndex(roads.filter((r) => r.kind === 'country' && !r.bridge));
  const near = (q: Port) => country.nearest(q.p[0], q.p[1], 4000)?.dist ?? Infinity;
  let laid = false;
  for (const q of [...ports].sort((a, b) => near(a) - near(b))) {
    const on = toNetwork(router, q, blocked);
    if (!on) continue;
    const pieces = router.layOnward(on).laid;
    if (pieces.some(clashes)) continue;
    for (const r of pieces) roads.push({ ...r, name: roadName(place, taken) });
    laid = true;
    break;
  }
  for (const q of ports) settlePort(roads, q, rect);
  return laid;
}

// ---- placing ------------------------------------------------------------------------------------------

/**
 * `draft` with a village placed `at` (see the top of this file): a hamlet of the place's own kind,
 * its streets, its road to the network where one can be routed, its lots, parking and names, all
 * drawn from `at.seed`. The new area is the last of `open.areas`. Refused where its site is not one
 * (with why), or where its streets hold too few lots. Where no road could be routed, `notes` says so.
 */
export function placeVillage(draft: NeighbourhoodDraft, t: Templates, content: Content, at: VillageAt): Edited {
  const swap = !!at.swap;
  const site = siteOf(draft, t, [at.x, at.z], villageSize(draft, swap));
  if ('refused' in site) return site;
  const open = draft.open!;
  const spec = specFor(draft);
  const { width: W, depth: D } = draft;
  const rand = mulberry32(at.seed >>> 0);
  const taken = namesIn(draft);
  const a = open.areas.length;
  // Its streets, as the generator lays a hamlet out and puts it on the world.
  const hamlet = hamletOf(content, spec, rand, a - 1, [...taken]);
  taken.add(hamlet.name);
  const roads = [...draft.roads];
  const plans: AreaPlan[] = open.areas.map((o) => ({ name: o.name, rect: rectOf(o), centre: middleOf(rectOf(o)), roads: [], greens: [], spec: hamlet.size, ports: [] }));
  const plan = putHamlet({ W, D, roads, plans }, hamlet, site.rect.x, site.rect.z, swap);
  const streets = streetNames(content, spec.names, rand, plan.roads.length, [...taken], plan.name);
  plan.roads.forEach((r, k) => {
    roads[r] = { ...roads[r], name: streets[k] };
    taken.add(streets[k]);
  });
  // Its road to the network, and its ways out nobody took ended inside its box.
  const linked = linkUp(draft, roads, [...open.areas.map(rectOf), plan.rect], plan.ports, plan.rect, plan.name, taken, thingsOf(draft, t));
  // Its lots, as the generator puts a hamlet's: along its own streets, off the corners, nearest its middle first.
  const greens = [...draft.greens, ...plan.greens.map((g) => g.map(([x, z]): Pt => [r2(x), r2(z)]))];
  const laid: NeighbourhoodDraft = { ...draft, roads, greens };
  const ground = townGround(laid, t);
  const placer = new LotPlacer(content, t, ground, roads, spec.gaps, rand, true);
  hamletLots(content, t, spec, plan, a, placer, roads, cornersOf(roads, plan.roads), rand);
  if (placer.slots.length < FEWEST_LOTS) return { refused: 'there is no room for a village’s homes along its streets there' };
  const slots = placer.slots.map((s, k): PlotSlot => ({ ...s, index: draft.slots.length + k, area: a }));
  // Its parking: a few spaces by the verge nearest its middle, more for more homes (as the generator lays a hamlet's).
  const spaces = parkingFor(slots.filter((s) => s.kind === 'house').length, false);
  const parking: { x: number; z: number; rot: number; area: number }[] = [];
  for (const f of frontageOf(roads, plan.roads, plan.centre, rand)) {
    if (parking.length >= spaces) break;
    const facing = (Math.abs(f.n[1]) >= Math.abs(f.n[0]) ? (f.n[1] > 0 ? 0 : 2) : f.n[0] > 0 ? 1 : 3) as Facing;
    const reach = roads[f.road].width / 2 + 0.5;
    const [qx, qz] = [f.p[0] + f.n[0] * (reach + 3), f.p[1] + f.n[1] * (reach + 3)];
    const r = facing % 2 ? { x: Math.round(qx - 3), z: Math.round(qz - 1.5), w: 6, d: 3 } : { x: Math.round(qx - 1.5), z: Math.round(qz - 3), w: 3, d: 6 };
    if (!ground.free(r, 8) || parking.some((p) => Math.hypot(p.x - r.x, p.z - r.z) < 12)) continue;
    ground.take(r, 1);
    parking.push({ x: r.x + r.w / 2, z: r.z + r.d / 2, rot: facing, area: a });
  }
  const area: TownArea = { name: plan.name, kind: 'hamlet', x: plan.rect.x, z: plan.rect.z, w: plan.rect.w, d: plan.rect.d };
  let next: NeighbourhoodDraft = { ...laid, slots: [...draft.slots, ...slots], open: { ...open, areas: [...open.areas, area], parking: [...open.parking, ...parking] } };
  for (const s of slots) if (s.kind !== 'house') next = withLotPlace(next, t, content, s.index);
  next = renumber(next, t, plan.roads);
  const notes = linked && reaches(next.roads, plan.roads) ? [] : [`no road could be found to ${plan.name}: draw one to it`];
  return { draft: next, cleared: NOTHING, ...(notes.length ? { notes } : {}) };
}

// ---- the roads that led to a village ---------------------------------------------------------------

/**
 * What goes with village `parts` besides its own roads: the country roads that led only to it and
 * carry no lot or building (`remove`; removing it, those with homes on them too, and their lots:
 * `lots`); and the roads a lot or building stands on (`carries`). Or why it may not go: another area,
 * or (moving it) homes on a road, would be left with no road to the town.
 */
function detach(draft: NeighbourhoodDraft, t: Templates, parts: Parts, removing: boolean): { remove: Set<number>; lots: Set<number>; carries: Set<number> } | { refused: string } {
  const { roads } = draft;
  const areas = draft.open?.areas ?? [];
  // The roads a lot or a building stands on (not the village's own).
  const carries = new Set<number>();
  const spots = entriesOf(draft, t);
  draft.slots.forEach((s, i) => {
    if (!parts.lots.has(s.index) && spots[i] && spots[i].road >= 0) carries.add(spots[i].road);
  });
  const index = roadIndex(roads);
  (draft.open?.backdrop ?? []).forEach((b, i) => {
    if (parts.backdrop.has(i)) return;
    const [x, z] = middleOfPoly(b.poly);
    const near = index.nearest(x, z, 30);
    if (near) carries.add(near.road);
  });
  // The main town's network (its first road's), without what is `gone`.
  const mainOf = (gone: ReadonlySet<number>) => {
    const nets = networks(roads, gone);
    const first = roads.findIndex((r, i) => !gone.has(i) && r.area === 0 && r.kind !== 'trail');
    return { nets, root: nets.get(first >= 0 ? first : roads.findIndex((r, i) => !gone.has(i) && r.ends.includes('out'))) };
  };
  const remove = new Set<number>();
  // Pieces hanging off the village alone: another area's refuses it; a bare one goes; one with homes stays.
  const start = mainOf(parts.roads);
  const hanging = new Map<number, number[]>();
  for (const [i, n] of start.nets) if (n !== start.root) hanging.set(n, [...(hanging.get(n) ?? []), i]);
  const lots = new Set<number>();
  for (const list of hanging.values()) {
    const other = list.find((i) => roads[i].area !== undefined && roads[i].area !== parts.area);
    if (other !== undefined) return { refused: `${areas[roads[other].area!]?.name ?? 'another village'} reaches the roads only through it` };
    // (One with homes on it: removed, it goes with the village, its lots with it; moved, the village may not leave it behind.)
    const held = list.filter((i) => carries.has(i));
    if (held.length && !removing) return { refused: `${listed(held.map((i) => roads[i].name ?? 'a country road'))} and the homes on ${held.length > 1 ? 'them' : 'it'} reach the town only through it` };
    for (const i of list) remove.add(i);
  }
  // The roads into it from the main network: taken up where they carry nothing and the rest holds
  // together (a bare road they alone held goes with them; one with homes, or another area's, keeps them).
  // (Removed, it takes even those with homes on them: a country road that led only to it would be
  // left a dead end out in the country, which no taxi turns round in; the card lists what goes.)
  const inMain = [...parts.links].filter((i) => start.nets.get(i) === start.root);
  const tried = new Set(inMain.filter((i) => removing || !carries.has(i)));
  for (let round = 0; round < 16 && tried.size; round++) {
    const gone = new Set([...parts.roads, ...remove, ...tried]);
    const now = mainOf(gone);
    const cut = [...start.nets.entries()].filter(([i, n]) => n === start.root && !gone.has(i) && now.nets.get(i) !== now.root).map(([i]) => i);
    if (!cut.length) break;
    const needed = cut.filter((i) => carries.has(i) || roads[i].area !== undefined);
    if (!needed.length) {
      for (const i of cut) remove.add(i);
      continue;
    }
    const back = [...tried].filter((c) => needed.some((i) => joined(roads, c, i)));
    if (!back.length) {
      tried.clear();
      break;
    }
    for (const c of back) tried.delete(c);
  }
  for (const i of tried) remove.add(i);
  draft.slots.forEach((s, k) => {
    if (spots[k] && remove.has(spots[k].road) && !parts.lots.has(s.index)) lots.add(s.index);
  });
  return { remove, lots, carries };
}

/** A list in words: "a", "a and b", "a, b and c". */
const listed = (items: readonly string[]) => (items.length < 2 ? (items[0] ?? '') : `${items.slice(0, -1).join(', ')} and ${items[items.length - 1]}`);

/**
 * `roads[i]`, a road left with a dead end where it met a village (`end`), cut back to `STUB_PAST` m
 * past the last of `lots` on it, or past the last road meeting it, whichever is further on.
 */
function cutStub(roads: readonly RoadPlan[], i: number, end: 0 | 1, lots: readonly Rect[]): RoadPlan {
  const r = roads[i];
  const ls = lengths(r.points);
  const L = ls[ls.length - 1];
  let last = Math.max(0, ...meetings(roads, i, end));
  for (const q of lots) {
    const at = nearest(r.points, q.x + q.w / 2, q.z + q.d / 2, ls);
    last = Math.max(last, (end === 1 ? at.s : L - at.s) + Math.max(q.w, q.d) / 2);
  }
  return cutTo(r, end, last + STUB_PAST);
}

/** The lots on each road of `draft` (by where they meet it: `entriesOf`) but those `gone`, as rectangles. */
function lotsOnOf(draft: NeighbourhoodDraft, t: Templates, gone: ReadonlySet<number>): (road: number) => Rect[] {
  const spots = entriesOf(draft, t);
  return (road) => draft.slots.flatMap((s, i) => (!gone.has(s.index) && spots[i]?.road === road ? [plotRect(t, s)] : []));
}

/**
 * `roads` (each from road `from[i]` of `draft`, -1 a new one) with the country roads left with a dead
 * end where they met what went: taken up where nothing stands on them and no road leads off them,
 * else cut back past their last lot (`cutStub`; `lotsOn`: the lots left on a road of `draft`) and said in the notes. The roads `keep` (of `draft`)
 * are left as they are. A join that meets nothing any more is a dead end (`deadEnds`).
 */
function tidy(draft: NeighbourhoodDraft, roads: RoadPlan[], from: number[], keep: ReadonlySet<number>, carries: ReadonlySet<number>, lotsOn: (road: number) => readonly Rect[], place: string): { roads: RoadPlan[]; from: number[]; notes: string[] } {
  const notes: string[] = [];
  const done = new Set<number>();
  for (let round = 0; round < 64; round++) {
    roads = deadEnds(roads, new Set(from.flatMap((o, i) => (keep.has(o) ? [i] : []))));
    let taken = -1;
    for (const [i, r] of roads.entries()) {
      const old = from[i];
      if (old < 0 || keep.has(old) || done.has(old) || r.kind !== 'country' || r.area !== undefined) continue;
      const was = draft.roads[old];
      const end = ([0, 1] as const).find((e) => was.ends[e] === 'join' && r.ends[e] === 'dead');
      if (end === undefined) continue;
      // (A road leads off it where another's end meets it past its own joined end: one that meets it
      // end to end there is only left a dead end in turn.)
      const held = meetings(roads, i, end).some((s) => s > r.width + 1);
      if (!carries.has(old) && !held) {
        taken = i;
        break;
      }
      done.add(old);
      roads[i] = cutStub(roads, i, end, lotsOn(old));
      notes.push(`${r.name ?? 'a country road'} is left ending where ${place} was${carries.has(old) ? ': homes stand on it' : ''}`);
    }
    if (taken < 0) break;
    roads = roads.filter((_, i) => i !== taken);
    from = from.filter((_, i) => i !== taken);
  }
  return { roads, from, notes };
}

// ---- moving -------------------------------------------------------------------------------------------

/**
 * Where a road may meet a village's streets (roads `own`, as they were: `was`; box `rect`): the ends
 * of theirs that were joined and now join nothing (where its roads met it before), else their dead
 * ends near the edge of its box heading out of it.
 */
function portsOf(roads: readonly RoadPlan[], own: readonly number[], was: readonly RoadPlan[], rect: Rect, area: number): Port[] {
  const index = new RoadIndex(roads);
  const portAt = (i: number, e: 0 | 1): Port => {
    const pts = e ? roads[i].points : [...roads[i].points].reverse();
    return { road: i, end: e, area, p: pts[pts.length - 1], dir: endDir(pts), used: false };
  };
  const ports: Port[] = [];
  for (const [k, i] of own.entries()) {
    const r = roads[i];
    if (r.kind === 'trail') continue;
    for (const e of [0, 1] as const) if (was[k].ends[e] === 'join' && !touches(roads, index, e ? r.points[r.points.length - 1] : r.points[0], i, r.width)) ports.push(portAt(i, e));
  }
  if (ports.length) return ports;
  for (const i of own) {
    if (roads[i].kind === 'trail') continue;
    for (const e of [0, 1] as const) {
      if (roads[i].ends[e] !== 'dead') continue;
      const q = portAt(i, e);
      const edge = Math.min(q.p[0] - rect.x, q.p[1] - rect.z, rect.x + rect.w - q.p[0], rect.z + rect.d - q.p[1]);
      if (edge < 30 && !insideRect(rect, [q.p[0] + q.dir[0] * 30, q.p[1] + q.dir[1] * 30])) ports.push(q);
    }
  }
  return ports;
}

/**
 * `draft` with village `area` moved by (`dx`, `dz`), rounded to whole metres (see the top of this
 * file): its streets, lots, buildings, parking and greens with it; the country roads that led to it
 * taken up where nothing stands on them (else left, said in `notes`); a road routed to it again where
 * one can be. Refused for the main town (moved by "Move the town"), where its new site is not one
 * (with why), or where another area reaches the roads only through it.
 */
export function moveVillage(draft: NeighbourhoodDraft, t: Templates, area: number, dx: number, dz: number): Edited {
  const no = notVillage(draft, area);
  if (no) return { refused: no };
  [dx, dz] = [Math.round(dx), Math.round(dz)];
  if (!dx && !dz) return { refused: 'it is there already' };
  const open = draft.open!;
  const box = rectOf(open.areas[area]);
  const parts = partsOf(draft, t, area);
  const site = siteOf(draft, t, [box.x + box.w / 2 + dx, box.z + box.d / 2 + dz], [box.w, box.d], asideOf(parts));
  if ('refused' in site) return site;
  const goes = detach(draft, t, parts, false);
  if ('refused' in goes) return goes;
  const name = open.areas[area].name;
  const rect = site.rect;
  const moved = ([x, z]: Pt): Pt => [r2(x + dx), r2(z + dz)];
  const land = open.land!;

  // Its lots, buildings, places, parking, greens and orchards with it; the parking by a road taken up goes.
  const slots = draft.slots.map((s) => (parts.lots.has(s.index) ? { ...s, x: s.x + dx, z: s.z + dz, ...(s.anchor === undefined ? {} : { anchor: s.anchor + (s.facing % 2 ? dz : dx) }) } : s));
  const backdrop = open.backdrop?.map((b, i) => {
    if (!parts.backdrop.has(i)) return b;
    const [x, z] = middleOfPoly(b.poly);
    return { ...b, poly: b.poly.map(moved), y: r2(b.y + heightAt(land, x + dx, z + dz) - heightAt(land, x, z)) };
  });
  const placesMoved = new Set((open.backdrop ?? []).flatMap((b, i) => (parts.backdrop.has(i) && b.place !== undefined ? [b.place] : [])));
  const places = open.places?.map((p, i) => (placesMoved.has(i) || (p.plot !== undefined && parts.lots.has(p.plot)) ? { ...p, x: r2(p.x + dx), z: r2(p.z + dz) } : p));
  const index = roadIndex(draft.roads);
  const parking = open.parking.flatMap((p, i) => {
    if (parts.parking.has(i)) return [{ ...p, x: p.x + dx, z: p.z + dz }];
    const near = index.nearest(p.x, p.z, 10);
    return near && goes.remove.has(near.road) ? [] : [p];
  });
  const greens = draft.greens.map((g, i) => (parts.greens.has(i) ? g.map(moved) : g));
  const zones = open.zones.map((z, i) => (parts.orchards.has(i) ? { ...z, poly: z.poly.map(moved) } : z));
  const pads = { ...open.pads };
  for (const s of slots) {
    if (!parts.lots.has(s.index) || pads[s.index] === undefined) continue;
    const r = plotRect(t, s);
    pads[s.index] = Math.round(heightAt(land, r.x + r.w / 2, r.z + r.d / 2) * 10) / 10;
  }

  // The roads: the village's moved, those that led only to it taken up, the others' joins that meet nothing dead ends.
  const own = [...parts.roads].sort((a, b) => a - b);
  const shifted = draft.roads.map((r, i) => (parts.roads.has(i) ? { ...r, points: r.points.map(moved) } : r));
  const left = tidy(
    draft,
    shifted.filter((_, i) => !goes.remove.has(i)),
    shifted.flatMap((_, i) => (goes.remove.has(i) ? [] : [i])),
    parts.roads,
    goes.carries,
    lotsOnOf(draft, t, parts.lots),
    name,
  );
  const roads = left.roads;
  const notes = left.notes;
  const ownAt = own.map((i) => left.from.indexOf(i));
  // Its road to the network, from where its streets were met.
  const ports = portsOf(roads, ownAt, own.map((i) => draft.roads[i]), rect, area);
  const areas = open.areas.map((a, k) => (k === area ? rect : rectOf(a)));
  const linked = ports.length > 0 && linkUp(draft, roads, areas, ports, rect, name, namesIn({ ...draft, roads }), thingsOf(draft, t, parts));
  // The other areas' streets and lanes it leaves ending where it was, held to a court's length (once
  // its new road is down: that may meet one again); the buildings and parking by a stretch cut off go.
  const capped = capCourts(draft, roads, left.from, lotsOnOf(draft, t, parts.lots), name, area, `with ${name} moved`);
  if ('refused' in capped) return capped;
  notes.push(...capped.notes);
  const byCut = (x: number, z: number, within: number) => {
    const near = index.nearest(x, z, within);
    const kept = near && capped.kept.get(near.road);
    return !!kept && (near.s < kept[0] - 0.5 || near.s > kept[1] + 0.5);
  };
  const cut = new Set((open.backdrop ?? []).flatMap((b, i) => (!parts.backdrop.has(i) && byCut(...middleOfPoly(b.poly), 40) ? [i] : [])));

  let next: NeighbourhoodDraft = {
    ...draft,
    roads: deadEnds(capped.roads),
    slots,
    greens,
    open: { ...open, areas: open.areas.map((a, k) => (k === area ? { ...a, x: rect.x, z: rect.z } : a)), zones, pads, parking: parking.filter((p) => !byCut(p.x, p.z, 10)), ...(backdrop ? { backdrop } : {}), ...(places ? { places } : {}) },
  };
  if (cut.size) next = removeLots(next, new Set(), cut);
  if (!linked || !reaches(next.roads, ownAt)) notes.push(`no road could be found to ${name}: draw one to it`);
  return { draft: next, cleared: cut.size ? { lots: [], backdrop: [...cut].sort((a, b) => a - b) } : NOTHING, ...(notes.length ? { notes } : {}) };
}

// ---- removing -----------------------------------------------------------------------------------------

/** Each draft's villages removed (`removal`), kept: the card asks what removing one takes as the pointer and the panel change. */
const removals = new WeakMap<NeighbourhoodDraft, Map<number, Edited>>();

/**
 * `draft` without village `area` (see `removeVillage`), made once per draft and village: the card
 * says what it takes (`villageGoes`) before the player removes it.
 */
function removal(draft: NeighbourhoodDraft, t: Templates, area: number): Edited {
  let byArea = removals.get(draft);
  if (!byArea) removals.set(draft, (byArea = new Map()));
  let made = byArea.get(area);
  if (!made) byArea.set(area, (made = removeOnce(draft, t, area)));
  return made;
}

function removeOnce(draft: NeighbourhoodDraft, t: Templates, area: number): Edited {
  if (area === 0) return { refused: 'the main town stays: move it with Move the town' };
  const no = notVillage(draft, area);
  if (no) return { refused: no };
  const open = draft.open!;
  const name = open.areas[area].name;
  const parts = partsOf(draft, t, area);
  const goes = detach(draft, t, parts, true);
  if ('refused' in goes) return goes;
  const lots = new Set([...parts.lots, ...goes.lots]);
  const kept = removalRefused(draft, lots);
  if (kept) return { refused: `${kept.replace(/\.$/, '')}: without ${name} it would go` };
  const homes = draft.slots.filter((s) => s.kind === 'house' && !lots.has(s.index)).length;
  if (homes < 2) return { refused: `without ${name} the town would have fewer than two homes` };
  // Its roads and those that led only to it, and the roads left ending where it was taken up or cut back.
  const gone = new Set([...parts.roads, ...goes.remove]);
  const left = tidy(
    draft,
    draft.roads.filter((_, i) => !gone.has(i)),
    draft.roads.flatMap((_, i) => (gone.has(i) ? [] : [i])),
    new Set(),
    goes.carries,
    lotsOnOf(draft, t, lots),
    name,
  );
  for (const i of draft.roads.keys()) if (!left.from.includes(i)) gone.add(i);
  if (!left.roads.some((r) => r.ends.includes('out')) || !reaches(left.roads, left.roads.flatMap((r, i) => (r.area === 0 ? [i] : [])))) return { refused: `without ${name} no road would lead out of the world` };
  // The other areas' streets and lanes it leaves ending where it was, held to a court's length.
  const capped = capCourts(draft, left.roads, left.from, lotsOnOf(draft, t, lots), name, area, `without ${name}`);
  if ('refused' in capped) return capped;
  left.notes.push(...capped.notes);
  // Its buildings (all of its area's: a farm's barn too), and those by a road that goes; the parking by its roads and theirs.
  const index = roadIndex(draft.roads);
  const byGone = (x: number, z: number, within: number) => {
    const near = index.nearest(x, z, within);
    return !!near && gone.has(near.road);
  };
  // (And those by the stretch of a street cut back: `capCourts`.)
  const byCut = (x: number, z: number, within: number) => {
    const near = index.nearest(x, z, within);
    const kept = near && capped.kept.get(near.road);
    return !!kept && (near.s < kept[0] - 0.5 || near.s > kept[1] + 0.5);
  };
  const backdrop = new Set(parts.backdrop);
  (open.backdrop ?? []).forEach((b, i) => (b.area === area || byGone(...middleOfPoly(b.poly), 40) || byCut(...middleOfPoly(b.poly), 40)) && backdrop.add(i));
  const parking = new Set(open.parking.flatMap((p, i) => (parts.parking.has(i) || byGone(p.x, p.z, 10) || byCut(p.x, p.z, 10) ? [i] : [])));
  let next: NeighbourhoodDraft = {
    ...draft,
    roads: capped.roads,
    greens: draft.greens.filter((_, i) => !parts.greens.has(i)),
    open: { ...open, parking: open.parking.filter((_, i) => !parking.has(i)), zones: open.zones.filter((_, i) => !parts.orchards.has(i)) },
  };
  next = removeArea(removeLots(next, lots, backdrop), area);
  const cleared = { lots: [...lots].sort((a, b) => a - b), backdrop: [...backdrop].sort((a, b) => a - b) };
  return { draft: next, cleared, ...(left.notes.length ? { notes: left.notes } : {}) };
}

/**
 * What removing village `area` from `draft` takes (see the top of this file): how many roads (its
 * own and the country roads that led only to it), the lots and buildings (by index in `draft`), how
 * many parking spaces; or why it may not go.
 */
export function villageGoes(draft: NeighbourhoodDraft, t: Templates, area: number): VillageGoes {
  const made = removal(draft, t, area);
  if ('refused' in made) return made;
  const open = draft.open!;
  const roads = draft.roads.length - made.draft.roads.length;
  const own = draft.roads.filter((r) => r.area === area).length;
  return { roads: own, others: roads - own, lots: made.cleared.lots, backdrop: made.cleared.backdrop, parking: open.parking.length - (made.draft.open?.parking.length ?? 0) };
}

/**
 * `draft` without village `area`: its roads, the lots fronting them, its buildings and their places,
 * its parking, greens and orchards; the country roads that led only to it (with the homes on them,
 * listed first by `villageGoes`); and its area, the area indexes on everything else numbered again
 * (`removeArea`). Refused for the main town, where the town would be left fewer than two homes, its
 * last park or no road out of the world, or where another area reaches the roads only through it.
 */
export function removeVillage(draft: NeighbourhoodDraft, t: Templates, area: number): Edited {
  return removal(draft, t, area);
}

/**
 * `draft` without area `area` (not the main town), the area indexes on everything else numbered
 * again: the lots', the roads', the backdrop buildings', the places', the parking's and the
 * centres'. Anything still in the area (a farm out on a country road, say) goes to the nearest area left.
 */
export function removeArea(draft: NeighbourhoodDraft, area: number): NeighbourhoodDraft {
  const open = draft.open;
  if (!open || area <= 0 || area >= open.areas.length) return draft;
  const areas = open.areas.filter((_, k) => k !== area);
  const nearestTo = (p: Pt) => {
    let [best, far] = [0, Infinity];
    areas.forEach((a, k) => {
      const d = offRect(rectOf(a), p);
      if (d < far) [best, far] = [k, d];
    });
    return best;
  };
  const to = (k: number, at: () => Pt) => (k < area ? k : k > area ? k - 1 : nearestTo(at()));
  const slots = draft.slots.map((s) => (s.area === undefined || s.area < area ? s : { ...s, area: to(s.area, () => [s.x, s.z]) }));
  const roads = draft.roads.map((r) => (r.area === undefined || r.area < area ? r : { ...r, area: to(r.area, () => r.points[Math.floor(r.points.length / 2)]) }));
  const backdrop = open.backdrop?.map((b) => (b.area < area ? b : { ...b, area: to(b.area, () => b.poly[0]) }));
  const places = open.places?.map((p) => (p.area < area ? p : { ...p, area: to(p.area, () => [p.x, p.z]) }));
  const parking = open.parking.map((p) => (p.area < area ? p : { ...p, area: to(p.area, () => [p.x, p.z]) }));
  const centre = open.centre?.map((c) => (c.area < area ? c : { ...c, area: to(c.area, () => c.poly[0]) }));
  return { ...draft, slots, roads, open: { ...open, areas, parking, ...(backdrop ? { backdrop } : {}), ...(places ? { places } : {}), ...(centre ? { centre } : {}) } };
}

/** `draft` without the villages (areas past the main town) left with no road and no lot of their own (what removing a village's last road leaves), the last first. */
export function withoutEmptyAreas(draft: NeighbourhoodDraft): NeighbourhoodDraft {
  const areas = draft.open?.areas ?? [];
  const empty = areas.flatMap((_, k) => (k > 0 && !draft.roads.some((r) => r.area === k) && !draft.slots.some((s) => s.area === k) ? [k] : []));
  let next = draft;
  for (const k of empty.reverse()) next = removeArea(next, k);
  return next;
}
