/**
 * The living town's world data, laid out with a big
 * world (`macro.ts`):
 *
 * - each main town's centre (`chooseCentre`): its high street, the stretch of its street nearest
 *   its middle, and a square off it, kept clear of lots before they go down;
 * - the backdrop (`layBackdrop`): buildings nobody lives in that make the town dense, laid after
 *   the lots, the parking, the trails and the orchards: rows along every street (shops, cafés and
 *   flats of two to four storeys shoulder to shoulder in the centre; houses, terraces and flats
 *   further out), rows behind them, the blocks filled in, and barns and a farm building by every
 *   farmstead. None of them is a place: the town's named places are lots (`placeLots.ts`, put down
 *   by `macro.ts` before the houses): its civic places (a town hall, a church, a school, a police and
 *   a fire station, a medical centre, offices, workshops, a pub, shops: every career a resident may
 *   hold has a workplace in a main town), and its shops, cafés, restaurants, offices, workshops and
 *   farm shops (`ZONE`);
 * - the places the town tags name (`TownPlace`): every public lot's, with its opening hours (the
 *   kind's, widened to the shifts of the careers that work there) and those careers' categories
 *   (`work`).
 *
 * The rules a backdrop building keeps (tools/worlds/sample.ts checks them): it never overlaps a
 * lot (and keeps `LOT_CLEAR` m off its boundary), a road with its sidewalks or a junction (it keeps
 * more than half a road's width off its middle line), the water, a trail, the parking, a square or
 * another backdrop building (two may touch: a row of shops). Its footprint is a rectangle turned to
 * the road nearest it (`FACING`: its long side or its gable end along the road). Every main town
 * has as many as its size wants (`BackdropInput.least`: `MAIN_LEAST` in a town or a city), every
 * hamlet `HAMLET_LEAST` (its named places' lots counted with them); the centre is the densest. A main
 * town has the civic places of its size (`Civic.least`): a hamlet a parish hall, a church, an inn and
 * a shop; a village a school, a garage, builders, studios and a garden centre besides; a town or a
 * city every one. Every other area has a hamlet's.
 *
 * Footprints go counter-clockwise seen from above (+y up, x right: z runs down the screen, so the
 * signed area x·z' − x'·z is negative), in town metres. `y` is 0, the town's flat land: they stand
 * within reach of the roads and lots, where the land is levelled (the renderer may count them in
 * its built field as it does lots). `ridge` is the angle a turn round +y takes +x to the ridge's
 * way (right-handed, y up: the ridge runs along (cos r, −sin r) in x, z), in [0, π).
 */

import type { Content } from '../content/content';
import type { BackdropBuilding, BuildingUse, TownPlace, TownSize } from '../core/protocol';
import { townGround } from './edit/ground';
import { inside, leftOf, lengths, pointAt, RoadIndex, type Pt, type RoadPlan } from './roads';
import { plotRect, type Ground, type NeighbourhoodDraft, type PlotSlot, type Rect, type Templates } from './town';
import { slotPlace } from './placeLots';
import { BANDS, centreDensity, DENSEST, EDGE, outline, selfCrossing, type Centre } from './centre';
import { centroidOf, facesRoad, fillPoly, overlaps, rectFrom, rectPoly, tilesOf } from './footprints';
import { draftAreaSize, HOURS, hoursFor, K, namerFor, PLACE_KINDS, type Kind, type Namer } from './places';
import { placeOf, specOf } from './world';

// (Moved to their own files: the footprints' geometry, the centre and the named places. Kept
// here too, so what imports them from here still finds them.)
export { FACING, facesRoad, fillPoly, overlaps, signedArea2, toOutline } from './footprints';
export { centreDensity, chooseCentre, type Centre } from './centre';
export { areaSize, byNeed, CIVIC, civicChoices, civicLabel, civicOf, civicTooBig, draftAreaSize, draftSize, HOURS, hoursFor, KIND_WORK, LOT_HOURS, LOT_WORK, namerFor, placeName, ZONE, type Civic, type Kind, type Namer, type ZoneKind } from './places';

type Rand = () => number;

/** A backdrop building keeps this far off a lot's boundary (m). */
export const LOT_CLEAR = 1;
/** At least this many backdrop buildings in a main town of a town or a city, in a hamlet. */
export const MAIN_LEAST = 60;
export const HAMLET_LEAST = 8;
/** How far a building in a block's middle may stand from the nearest road's edge (m): the main town's, a hamlet's. */
const INFILL_REACH = [34, 24];

const add = (a: Pt, b: Pt, k = 1): Pt => [a[0] + b[0] * k, a[1] + b[1] * k];
const dist = (a: Pt, b: Pt) => Math.hypot(a[0] - b[0], a[1] - b[1]);
const r2 = (v: number) => Math.round(v * 100) / 100;
const range = (rand: Rand, lo: number, hi: number) => lo + (hi - lo) * rand();
const between = (rand: Rand, lo: number, hi: number) => lo + Math.floor(rand() * (hi - lo + 1));
const pick = <T>(rand: Rand, list: readonly T[]): T => list[Math.floor(rand() * list.length)];

/**
 * Keeps a centre's land for the backdrop: every tile inside its outline nothing takes yet is taken
 * on `ground` (no house goes there: its public places went down first), and listed (one per tile,
 * row-major) for `layBackdrop` to build on.
 */
export function reserveCentre(centre: Centre, ground: Ground): Uint8Array {
  const out = new Uint8Array(ground.w * ground.d);
  const xs = centre.poly.map((p) => p[0]);
  const zs = centre.poly.map((p) => p[1]);
  for (let z = Math.max(0, Math.floor(Math.min(...zs))); z < Math.min(ground.d, Math.ceil(Math.max(...zs))); z++) {
    for (let x = Math.max(0, Math.floor(Math.min(...xs))); x < Math.min(ground.w, Math.ceil(Math.max(...xs))); x++) {
      if (!inside(centre.poly, x + 0.5, z + 0.5) || ground.takenAt(x + 0.5, z + 0.5)) continue;
      out[z * ground.w + x] = 1;
      ground.take({ x, z, w: 1, d: 1 });
    }
  }
  return out;
}


/** The kinds by where they stand: the centre (2 to 4 storeys, shoulder to shoulder), the rest of a main town, a hamlet. */
const KINDS = {
  centre: [
    [K('shop', [6, 10], [10, 14], [2, 3], ['gable', 'flat', 'mansard']), 34],
    [K('flats', [12, 20], [10, 14], [3, 4], ['flat', 'mansard', 'hip']), 20],
    [K('cafe', [6, 9], [9, 12], [2, 3], ['gable', 'mansard']), 10],
    [K('restaurant', [9, 13], [10, 14], [2, 3], ['gable', 'hip']), 7],
    [K('office', [14, 22], [11, 15], [3, 4], ['flat', 'mansard']), 12],
    [K('terrace', [16, 30], [9, 11], [2, 3], ['gable', 'mansard']), 12],
    [K('hall', [14, 18], [10, 14], [2, 2], ['gable', 'hip']), 3],
  ],
  town: [
    [K('house', [8, 11], [8, 10], [1, 2], ['gable', 'hip']), 30],
    [K('terrace', [18, 32], [9, 11], [2, 2], ['gable']), 24],
    [K('flats', [12, 20], [10, 14], [2, 4], ['flat', 'hip', 'mansard']), 16],
    [K('shop', [7, 10], [9, 12], [1, 2], ['gable', 'flat']), 7],
    [K('workshop', [12, 20], [10, 16], [1, 2], ['gable', 'flat']), 8],
    [K('office', [14, 20], [10, 14], [2, 3], ['flat', 'hip']), 5],
    [K('cafe', [6, 9], [9, 11], [1, 2], ['gable']), 3],
    [K('restaurant', [9, 12], [10, 13], [1, 2], ['gable', 'hip']), 2],
    [K('hall', [12, 16], [10, 12], [1, 2], ['gable']), 2],
  ],
  hamlet: [
    [K('house', [8, 11], [8, 10], [1, 2], ['gable', 'hip']), 48],
    [K('terrace', [16, 24], [9, 10], [2, 2], ['gable']), 12],
    [K('workshop', [10, 16], [10, 14], [1, 2], ['gable']), 10],
    [K('barn', [12, 18], [9, 12], [2, 2], ['gable']), 10],
    [K('shop', [7, 9], [9, 11], [1, 2], ['gable']), 6],
    [K('cafe', [6, 8], [9, 11], [1, 2], ['gable']), 4],
    [K('hall', [12, 15], [9, 12], [1, 2], ['gable']), 3],
  ],
} as const satisfies Record<string, readonly (readonly [Kind, number])[]>;
/** Where a backdrop building stands, which sets its kinds, gaps and setback: a main town's centre, the rest of a main town, a hamlet. */
export type Zone = keyof typeof KINDS;

const FARM_BUILDINGS = ['Dairy', 'Stables', 'Farm Shop', 'Creamery', 'Granary'];


export interface BackdropInput {
  content: Content;
  t: Templates;
  W: number;
  D: number;
  roads: readonly RoadPlan[];
  /** The areas (0: the main town): name, box, middle, roads (indices into `roads`). */
  areas: readonly { name: string; rect: Rect; centre: Pt; roads: readonly number[] }[];
  slots: readonly PlotSlot[];
  rectOf: (s: PlotSlot) => Rect;
  ground: Ground;
  centres: readonly Centre[];
  /** The centres' land kept for the backdrop (`reserveCentre`), if any. */
  reserved?: Uint8Array;
  /** The greens and the water (one per tile), for how built over the centre is (`centreDensity`). */
  greens: readonly Pt[][];
  water: Uint8Array;
  /** How much of a main town's land is built over, 0..1 (a dense town's most). */
  density: number;
  /** The fewest backdrop buildings in the main town (its named places' lots among them), and how big a town it is (its places' names: `Namer.size`). */
  least: number;
  civic: TownSize;
  rand: Rand;
}

/** What a pass over a town laid already is made from (`BackdropPass`): its draft, the house templates its lots are measured by, the content its places' names and careers come from, and how its squares are kept clear. */
export interface BackdropDraft {
  draft: NeighbourhoodDraft;
  t: Templates;
  content: Content;
}

/**
 * A building of a row along a road (`BackdropPass.rowsAlong`): its index, the road, how far along it its middle is and
 * on which side, how far back from the road its back is, its zone, and the kinds the row was limited to (if it was:
 * the row behind it keeps to them too, `secondRow`).
 */
export interface Row {
  at: number;
  r: number;
  s: number;
  side: 1 | -1;
  back: number;
  zone: Zone;
  kinds?: readonly BuildingUse[];
}

/**
 * Where along its road a row goes (m, `from` to `to`; the whole road if not given) and what its buildings may be
 * (`kinds`: any of its zone's if not given; `kindsAt`: those the land a metre along and 6 m back from the
 * sidewalk asks for, none there if null, in place of `kinds`, with no gaps but where it asks for none).
 */
export interface RowLimits {
  from?: number;
  to?: number;
  kinds?: readonly BuildingUse[];
  kindsAt?: (p: Pt) => readonly BuildingUse[] | null;
}

/** The buildings in each 24 m cell (`BackdropPass`'s overlap test). */
const BUCKET = 24;
const bucketKey = (cx: number, cz: number) => cx * 4096 + cz;

/**
 * The backdrop laid in steps (see the top of this file): rows along each road (`rowsAlong`), a second row behind them (`secondRow`), the blocks filled in (`infill`),
 * the centre made the densest (`thin`), the farm buildings (`farms`) and the public lots' places
 * (`places`); `result` gives the buildings and the places. `layBackdrop` takes the steps in
 * that order with one stream, as the generator does.
 *
 * Made from a draft (`BackdropDraft`), it builds on a town laid already: what may not be built on
 * is the town's ground as the edits see it (`edit/ground.ts` `townGround`: its roads, water, lots,
 * parking, squares and trails) with every lot's margin and the land before it to its road marked,
 * and every building standing is in the overlap test (a new one may touch it, as the generator's
 * do). New buildings are added after those (`rowsAlong`, `secondRow`, `put`, `infillPoints`), none
 * of them a place; the town's own are not moved. A draft's pass has no centres to thin (`thin`).
 */
export class BackdropPass {
  private readonly content: Content;
  /** The layouts the lots are measured by (a public lot's place: `slotPlace`). */
  private readonly t: Templates;
  private readonly W: number;
  private readonly D: number;
  private readonly roads: readonly RoadPlan[];
  private readonly areas: BackdropInput['areas'];
  private readonly slots: readonly PlotSlot[];
  private readonly rectOf: (s: PlotSlot) => Rect;
  private readonly ground: Ground;
  private readonly centres: readonly Centre[];
  /** A draft's centres' outlines (it has no `Centre`s), for which zone a building is in. */
  private readonly outlines: readonly Pt[][];
  private readonly greens: readonly Pt[][];
  private readonly water: Uint8Array;
  private readonly density: number;
  private readonly least: number;
  /** How big area `a` is for its places' names (`areaSize` on a draft, the main town's size for the generator). */
  private readonly sizeOf: (a: number) => TownSize;
  private readonly index: RoadIndex;
  /** Which roads are ways (not trails). */
  private readonly ways: boolean[];
  /** What a building may not stand on (a tile each). */
  private readonly blocked: Uint8Array;
  private readonly built: BackdropBuilding[] = [];
  private readonly buckets = new Map<number, number[]>();
  /** The places so far (a draft's, then those named since). */
  private readonly placeList: TownPlace[] = [];
  /** What else a building must be to go down (by hand only: Build keeps each to the land painted its use); anything if unset. */
  allow?: (poly: readonly Pt[], use: BuildingUse) => boolean;

  constructor(input: BackdropInput | BackdropDraft) {
    if ('draft' in input) {
      const { draft, t, content } = input;
      const open = draft.open;
      const spec = specOf(placeOf(draft.world?.kind ?? 'country'), draft.world?.options);
      this.content = content;
      this.t = t;
      this.W = draft.width;
      this.D = draft.depth;
      this.roads = draft.roads;
      this.areas = (open?.areas ?? []).map((a, i) => ({
        name: a.name,
        rect: { x: a.x, z: a.z, w: a.w, d: a.d },
        centre: [a.x + a.w / 2, a.z + a.d / 2] as Pt,
        roads: draft.roads.flatMap((r, k) => (r.area === i && r.kind !== 'trail' ? [k] : [])),
      }));
      this.slots = draft.slots;
      this.rectOf = (s) => plotRect(t, s);
      // (The ground without the buildings standing: they are in the overlap test, where two may
      // touch, as a row of shops does; the ground keeps lots a metre off their boxes, which would
      // keep a new building off its neighbours.)
      this.ground = townGround(open ? { ...draft, open: { ...open, backdrop: [] } } : draft, t);
      this.centres = [];
      this.outlines = (open?.centre ?? []).map((c) => c.poly);
      // (Only `thin` reads the greens and the water, and a draft's pass has no centres to thin.)
      this.greens = draft.greens;
      this.water = new Uint8Array(0);
      this.density = spec.density;
      this.least = spec.mainLeast;
      // (By hand, a village's civic place is named as a village's, its village hall; the main town's by its size now.)
      this.sizeOf = (a) => draftAreaSize(draft, a);
      this.placeList.push(...(open?.places ?? []));
    } else {
      this.content = input.content;
      this.t = input.t;
      this.W = input.W;
      this.D = input.D;
      this.roads = input.roads;
      this.areas = input.areas;
      this.slots = input.slots;
      this.rectOf = input.rectOf;
      this.ground = input.ground;
      this.centres = input.centres;
      this.outlines = [];
      this.greens = input.greens;
      this.water = input.water;
      this.density = input.density;
      this.least = input.least;
      const size = input.civic;
      this.sizeOf = () => size;
    }
    const { W, D, roads, slots, ground } = this;
    this.index = new RoadIndex(roads);
    this.ways = roads.map((r) => r.kind !== 'trail');
    // What a building may not stand on (a tile each): what the ground takes (roads, greens, water,
    // lots and the gaps after them, the parking, the trails, the orchards, the squares), every lot
    // a metre round, and the land between a lot's front and its road (its driveway crosses it).
    const blocked = (this.blocked = ground.mask());
    const reserved = 'draft' in input ? undefined : input.reserved;
    if (reserved) for (let i = 0; i < blocked.length; i++) if (reserved[i]) blocked[i] = 0;
    const roadTiles = ground.roadTiles();
    const mark = (r: Rect) => {
      for (let z = Math.max(0, r.z); z < Math.min(D, r.z + r.d); z++) for (let x = Math.max(0, r.x); x < Math.min(W, r.x + r.w); x++) blocked[z * W + x] = 1;
    };
    for (const s of slots) {
      const r = this.rectOf(s);
      mark({ x: r.x - LOT_CLEAR, z: r.z - LOT_CLEAR, w: r.w + 2 * LOT_CLEAR, d: r.d + 2 * LOT_CLEAR });
      // Out from its front to the road (a wedge on a bend), 14 m at most.
      const [dx, dz] = [[0, -1], [-1, 0], [0, 1], [1, 0]][s.facing];
      const along = s.facing % 2 ? r.d : r.w;
      for (let k = 0; k < along; k++) {
        let [x, z] = s.facing === 0 ? [r.x + k, r.z - 1] : s.facing === 1 ? [r.x - 1, r.z + k] : s.facing === 2 ? [r.x + k, r.z + r.d] : [r.x + r.w, r.z + k];
        for (let step = 0; step < 14 && x >= 0 && z >= 0 && x < W && z < D && !roadTiles[z * W + x]; step++, x += dx, z += dz) blocked[z * W + x] = 1;
      }
    }
    // (A draft's buildings standing: in the overlap test.)
    if ('draft' in input) {
      for (const b of input.draft.open?.backdrop ?? []) {
        const at = this.built.push(b) - 1;
        this.bucket(b.poly, at);
      }
    }
  }

  /** How many buildings there are so far (a draft's among them). */
  get count(): number {
    return this.built.length;
  }

  /** Files building `at` under the cells its footprint's box covers. */
  private bucket(poly: readonly Pt[], at: number): void {
    const xs = poly.map((p) => p[0]);
    const zs = poly.map((p) => p[1]);
    for (let cz = Math.floor(Math.min(...zs) / BUCKET); cz <= Math.floor(Math.max(...zs) / BUCKET); cz++) {
      for (let cx = Math.floor(Math.min(...xs) / BUCKET); cx <= Math.floor(Math.max(...xs) / BUCKET); cx++) {
        const list = this.buckets.get(bucketKey(cx, cz));
        if (list) list.push(at);
        else this.buckets.set(bucketKey(cx, cz), [at]);
      }
    }
  }

  /** Whether a footprint may go down (see the top of this file). */
  fits(poly: Pt[]): boolean {
    const { W, D, blocked, index, built } = this;
    const bad = tilesOf(poly, 0, (x, z) => x < EDGE || z < EDGE || x >= W - EDGE || z >= D - EDGE || blocked[z * W + x] === 1);
    if (bad) return false;
    // (Off every road and trail by more than half its width, measured from its middle.)
    for (let i = 0; i < poly.length; i++) {
      const [a, b] = [poly[i], poly[(i + 1) % poly.length]];
      const n = Math.max(1, Math.ceil(dist(a, b)));
      for (let k = 0; k <= n; k++) {
        const p = add(a, [b[0] - a[0], b[1] - a[1]], k / n);
        if (index.onRoad(p[0], p[1], 0.5)) return false;
      }
    }
    const xs = poly.map((p) => p[0]);
    const zs = poly.map((p) => p[1]);
    for (let cz = Math.floor(Math.min(...zs) / BUCKET); cz <= Math.floor(Math.max(...zs) / BUCKET); cz++) {
      for (let cx = Math.floor(Math.min(...xs) / BUCKET); cx <= Math.floor(Math.max(...xs) / BUCKET); cx++) {
        // (Less than `overlaps`' slack: rounded to the centimetre in the file, it still holds.)
        for (const j of this.buckets.get(bucketKey(cx, cz)) ?? []) if (overlaps(poly, built[j].poly, 0.02)) return false;
      }
    }
    return true;
  }

  /** The road (not a trail) nearest a point, within `within` m: as `RoadIndex.nearest`. */
  private nearestWay(x: number, z: number, within: number): { road: number; dist: number; s: number; side: 1 | -1 } | null {
    let best: { road: number; dist: number; s: number; side: 1 | -1 } | null = null;
    for (const [road, hit] of this.index.around(x, z, within)) {
      if (!this.ways[road] || (best && best.dist <= hit.dist)) continue;
      // (+1: the road's left, as `RoadIndex.nearest` says.)
      const { p } = pointAt(this.roads[road].points, hit.s);
      const side: 1 | -1 = (x - p[0]) * hit.t[1] - (z - p[1]) * hit.t[0] >= 0 ? 1 : -1;
      best = { road, dist: hit.dist, s: hit.s, side };
    }
    return best;
  }

  /** The area a point is in (its box), else the one whose middle is nearest. */
  private areaOf(p: Pt): number {
    const { areas } = this;
    const at = areas.findIndex((a) => p[0] >= a.rect.x && p[1] >= a.rect.z && p[0] <= a.rect.x + a.rect.w && p[1] <= a.rect.z + a.rect.d);
    return at >= 0 ? at : areas.reduce((k, a, i) => (dist(a.centre, p) < dist(areas[k].centre, p) ? i : k), 0);
  }

  /** Whether a point is in a centre (its outline as it is now: `thin` may narrow it). */
  private centreAt(p: Pt): boolean {
    return this.centres.some((c) => inside(c.poly, p[0], p[1])) || this.outlines.some((poly) => inside(poly, p[0], p[1]));
  }

  /**
   * Lays a building of `kind` down, `w` along `t` and `d` back along `n` (away from the road) from
   * the middle of its front `c`, if it fits and faces its road (its roof, storeys and look from
   * `rand`; a gable end to the road if `endOn`): its index, else -1.
   */
  put(c: Pt, t: Pt, n: Pt, w: number, d: number, kind: Kind, rand: Rand, endOn = false): number {
    const poly = rectFrom(c, t, n, w, d);
    if ((this.allow && !this.allow(poly, kind.use)) || !this.fits(poly) || !facesRoad(poly, this.roads, this.index, 0.3)) return -1;
    const roof = pick(rand, kind.roofs);
    // (The ridge along the longer side; a church's back from the road.)
    const way = endOn || d > w ? n : t;
    let ridge = Math.atan2(-way[1], way[0]);
    if (ridge < 0) ridge += Math.PI;
    if (ridge >= Math.PI) ridge -= Math.PI;
    const b: BackdropBuilding = {
      poly: poly.map(([x, z]): [number, number] => [r2(x), r2(z)]),
      y: 0,
      storeys: between(rand, ...kind.storeys),
      roof,
      // (Never -0, which a report or the town file reads back as 0.)
      ...(roof === 'gable' || roof === 'mansard' ? { ridge: Math.round(ridge * 1000) / 1000 || 0 } : {}),
      use: kind.use,
      look: Math.floor(rand() * 1024),
      area: this.areaOf(centroidOf(poly)),
    };
    const at = this.built.push(b) - 1;
    this.bucket(poly, at);
    return at;
  }

  /** Where a building's front goes from road `r` at `s` on `side`, set back `back` m from its sidewalk's edge: the middle of its front, the way along, the way back from the road. */
  private frontAt(r: number, s: number, side: 1 | -1, back: number, ls?: number[]): { c: Pt; t: Pt; n: Pt } {
    const road = this.roads[r];
    const { p, t } = pointAt(road.points, s, ls);
    const n = leftOf(t).map((v) => v * side) as Pt;
    return { c: add(p, n, road.width / 2 + back), t, n };
  }

  /** A kind of `zone`'s drawn by its weight (`kinds`: of those only; null if the zone has none of them). */
  private pickKind(zone: Zone, rand: Rand): Kind;
  private pickKind(zone: Zone, rand: Rand, kinds: readonly BuildingUse[] | undefined): Kind | null;
  private pickKind(zone: Zone, rand: Rand, kinds?: readonly BuildingUse[]): Kind | null {
    const all = KINDS[zone] as readonly (readonly [Kind, number])[];
    const list = kinds ? all.filter(([k]) => kinds.includes(k.use)) : all;
    if (!list.length) return null;
    let r = rand() * list.reduce((n, [, w]) => n + w, 0);
    for (const [k, w] of list) if ((r -= w) <= 0) return k;
    return list[0][0];
  }

  /** (Past the sidewalk: off the tiles the road's land takes, a metre and more past its edge.) */
  private setback(zone: Zone, rand: Rand): number {
    return zone === 'centre' ? range(rand, 2.3, 2.8) : zone === 'town' ? range(rand, 2.5, 5) : range(rand, 3, 7);
  }

  /**
   * A row of buildings along road `road` (of area `area`) on each of `sides`: the centre shoulder to
   * shoulder, the rest with gaps, each of its zone's kinds (`zone`: that zone all along; else the
   * centre's where it is in one, the main town's in area 0, a hamlet's elsewhere), smaller where it
   * doesn't fit. `limits`: only from `from` to `to` m along the road (each building wholly within),
   * and only of those `kinds` (`RowLimits`). The buildings of the row as laid.
   */
  rowsAlong(area: number, road: number, sides: readonly (1 | -1)[], rand: Rand, zone?: Zone, limits: RowLimits = {}): Row[] {
    const { roads, density } = this;
    const { from, to, kindsAt } = limits;
    const r = road;
    const ls = lengths(roads[r].points);
    const L = ls[ls.length - 1];
    const end = Math.min(L - 4, to ?? Infinity);
    const rows: Row[] = [];
    for (const side of sides) {
      // (From 4 to 8 m along the road; from 0 to 1 m past `from`, if given, to make the most of a short stretch.)
      let s = from === undefined ? 4 + rand() * 4 : Math.max(4, from + rand());
      // (Where the land asks for kinds: how far on along the road it asks for the same, read every 2 m.)
      let [runKinds, runEnd]: [readonly BuildingUse[] | null | undefined, number] = [undefined, 0];
      while (s < end) {
        const probe = this.frontAt(r, s, side, 6, ls).c;
        // (The kinds the land asks for here, if it is asked: on along the road where it asks for none.)
        let kinds = kindsAt ? kindsAt(this.frontAt(r, Math.min(s + 1, L), side, 6, ls).c) : limits.kinds;
        if (kinds === null) {
          s += 2;
          continue;
        }
        if (kindsAt && (kinds !== runKinds || s + 1 >= runEnd)) {
          let x = s + 1;
          while (x + 2 <= L && kindsAt(this.frontAt(r, x + 2, side, 6, ls).c) === kinds) x += 2;
          [runKinds, runEnd] = [kinds, x + 1];
        }
        const z: Zone = zone ?? (this.centreAt(probe) ? 'centre' : area === 0 ? 'town' : 'hamlet');
        // (Of those, only the kinds that fit what is left of the land asking for them.)
        if (kindsAt && kinds) {
          const room = runEnd - s;
          kinds = KINDS[z].flatMap(([k]) => (kinds!.includes(k.use) && k.w[0] * 0.6 <= room ? [k.use] : []));
        }
        // (Now and then a gap: the more the further out. Where the land asks for kinds, its gaps are where it asks for none.)
        const skip = z === 'centre' ? 0.04 : z === 'town' ? 0.22 * (1.2 - density) : 0.55;
        if (!kindsAt && rand() < skip) {
          s += range(rand, 6, 16);
          continue;
        }
        const kind = this.pickKind(z, rand, kinds);
        if (!kind) {
          s += 3;
          continue;
        }
        let done = false;
        // (Smaller where it doesn't fit: narrower, and shallower in front of what stands behind.)
        for (const shrink of [1, 0.75, 0.55]) {
          const w = Math.min(Math.max(kind.w[0] * 0.6, range(rand, ...kind.w) * shrink), kindsAt ? runEnd - s : Infinity);
          const d = Math.max(6, (range(rand, ...kind.d) * (shrink + 1)) / 2);
          const back = this.setback(z, rand);
          // (Within the stretch asked for, if one was.)
          if (to !== undefined && s + w > to) continue;
          const { c, t, n } = this.frontAt(r, s + w / 2, side, back, ls);
          const at = this.put(c, t, n, w, d, kind, rand);
          if (at < 0) continue;
          rows.push({ at, r, s: s + w / 2, side, back: back + d, zone: z, ...(kinds ? { kinds } : {}) });
          s += w + (z === 'centre' ? 0 : z === 'town' ? range(rand, 1, 4) : range(rand, 4, 12));
          done = true;
          break;
        }
        if (!done) s += 3;
      }
    }
    return rows;
  }

  /** A second row behind `rows` (the main towns' only): yards between, the road nearest still, more often the denser the town; of a row's own kinds if it was limited to some (`Row.kinds`). */
  secondRow(rows: readonly Row[], rand: Rand): void {
    const { density } = this;
    for (const row of rows) {
      if (row.zone === 'hamlet' || rand() > (row.zone === 'centre' ? 0.9 : 0.55 * density + 0.2)) continue;
      const kind = this.pickKind(row.zone === 'centre' ? 'centre' : 'town', rand, row.kinds);
      if (!kind) continue;
      const w = range(rand, ...kind.w);
      const d = range(rand, ...kind.d);
      const { c, t, n } = this.frontAt(row.r, row.s, row.side, row.back + range(rand, 2, 5));
      const mid = add(c, n, d / 2);
      const near = this.nearestWay(mid[0], mid[1], 80);
      if (!near || near.road !== row.r) continue;
      this.put(c, t, n, w, d, kind, rand);
    }
  }

  /** Area `a`'s blocks filled in (`within`: inside that outline only), on a jittered grid `step` m apart, a building at each point with `chance`, turned to the road nearest. */
  infillArea(a: number, chance: number, step: number, rand: Rand, within?: Pt[]): void {
    const { areas } = this;
    const xs = within?.map((p) => p[0]);
    const zs = within?.map((p) => p[1]);
    const rect = xs && zs ? { x: Math.min(...xs) + 20, z: Math.min(...zs) + 20, w: Math.max(...xs) - Math.min(...xs) - 40, d: Math.max(...zs) - Math.min(...zs) - 40 } : areas[a].rect;
    for (let z = rect.z - 20; z < rect.z + rect.d + 20; z += step) {
      for (let x = rect.x - 20; x < rect.x + rect.w + 20; x += step) {
        const p: Pt = [x + range(rand, -step / 3, step / 3), z + range(rand, -step / 3, step / 3)];
        if (within && !inside(within, p[0], p[1])) continue;
        this.infillAt(a, p, rand, chance);
      }
    }
  }

  /**
   * A building of area `a` at each of `points` (set by hand), turned to the road nearest as the blocks are filled in
   * (`infillArea`), of the kinds the land there asks for (`kindsAt`: none if null), smaller where it doesn't fit.
   * The buildings laid, by index.
   */
  infillPoints(a: number, points: readonly Pt[], rand: Rand, kindsAt: (p: Pt) => readonly BuildingUse[] | null): number[] {
    return points.flatMap((p) => {
      const at = this.infillAt(a, p, rand, 1, kindsAt);
      return at < 0 ? [] : [at];
    });
  }

  /** A building at `p` of area `a` with `chance`, turned to the road nearest within its reach, as `infillArea` lays one (`kindsAt`: see `infillPoints`). Its index, else -1. */
  private infillAt(a: number, p: Pt, rand: Rand, chance: number, kindsAt?: (p: Pt) => readonly BuildingUse[] | null): number {
    const { roads, W, D, blocked } = this;
    const reach = INFILL_REACH[a === 0 ? 0 : 1];
    if (p[0] < EDGE || p[1] < EDGE || p[0] >= W - EDGE || p[1] >= D - EDGE || blocked[Math.floor(p[1]) * W + Math.floor(p[0])]) return -1;
    const kinds = kindsAt?.(p);
    if (kinds === null) return -1;
    if (rand() > chance) return -1;
    const near = this.nearestWay(p[0], p[1], reach + 10);
    if (!near || near.dist - roads[near.road].width / 2 > reach) return -1;
    const { t } = pointAt(roads[near.road].points, near.s);
    const n = leftOf(t).map((v) => v * near.side) as Pt;
    const zone: Zone = this.centreAt(p) ? 'centre' : a === 0 ? 'town' : 'hamlet';
    const kind = this.pickKind(zone, rand, kinds);
    if (!kind) return -1;
    // (In the centre smaller where it doesn't fit: it is built up close; and anywhere the land asks for its kinds.)
    for (const shrink of zone === 'centre' || kindsAt ? [1, 0.75, 0.55] : [1]) {
      const w = Math.max(kind.w[0] * 0.6, range(rand, ...kind.w) * shrink);
      const d = Math.max(6, (range(rand, ...kind.d) * (shrink + 1)) / 2);
      const at = this.put(add(p, n, -d / 2), t, n, w, d, kind, rand);
      if (at >= 0) return at;
    }
    return -1;
  }

  /** How many buildings area `a` has: its backdrop buildings, and its named places' lots (each a building of the town's, as tools/worlds/sample.ts counts them). */
  private countIn(a: number): number {
    return this.built.filter((b) => b.area === a).length + this.slots.filter((s) => s.dress && (s.area ?? 0) === a).length;
  }

  /** The blocks filled in: the centres first, closest (the densest); the main town; each hamlet; then more closely where an area is short of its least (`least`, `HAMLET_LEAST`). */
  infill(rand: Rand): void {
    const { centres, areas, density, least } = this;
    for (const c of centres) this.infillArea(c.area, 1, 6, rand, c.poly);
    this.infillArea(0, 0.3 + 0.45 * density, 12, rand);
    for (let a = 1; a < areas.length; a++) this.infillArea(a, 0.25, 16, rand);
    for (let a = 0; a < areas.length; a++) for (let k = 0; k < 3 && this.countIn(a) < (a === 0 ? least : HAMLET_LEAST); k++) this.infillArea(a, 1, 9 - k * 2, rand);
  }

  /**
   * Each centre made the densest: its outline narrowed to the street where the land round it is the
   * emptier, then houses and such taken away round it (the furthest first, never a shop or the like,
   * never below the town's least) till it is. Its square counts as open land, a named place's lot as
   * built over: in a town whose centre is mostly its square, the rest is thinned.
   */
  thin(): void {
    const { built, roads, areas, slots, greens, water, W, D, least } = this;
    for (const c of this.centres) {
      const box = areas[c.area].rect;
      // (The roads' tiles as the ground has them, a little wider than the check's: `DENSEST` spares it.)
      // (A named place's lot is built over, as a building is: `placeLots.ts`; the other lots are not open land.)
      const taken = this.ground.roadTiles().slice();
      const placed: Pt[][] = [];
      for (const s of slots) {
        const r = this.rectOf(s);
        if (s.dress) placed.push(rectPoly(r));
        else for (let z = Math.max(0, r.z); z < Math.min(D, r.z + r.d); z++) for (let x = Math.max(0, r.x); x < Math.min(W, r.x + r.w); x++) taken[z * W + x] = 1;
      }
      for (const g of greens) fillPoly(g, W, D, (i) => (taken[i] = 1));
      for (let i = 0; i < taken.length; i++) if (water[i]) taken[i] = 1;
      const polys = () => [...built.map((b) => b.poly), ...placed];
      let d = centreDensity(W, D, taken, box, c.poly, polys());
      // (Not the densest: the outline narrowed to the street, where the land round it is the emptier.)
      for (const band of BANDS.filter((b) => b < c.band)) {
        if (d.centre >= d.rest * DENSEST) break;
        const poly = outline(roads, c, band);
        if (selfCrossing(poly)) continue;
        const n = centreDensity(W, D, taken, box, poly, polys());
        if (n.centre / Math.max(1e-6, n.rest) <= d.centre / Math.max(1e-6, d.rest)) continue;
        [c.poly, c.band, d] = [poly, band, n];
      }
      let rest = d.built[1];
      // (Those not a place, outside the centre, in its box: the furthest from the centre first.)
      const mid = centroidOf(c.poly);
      const spare = built
        .map((b, i) => ({ i, far: dist(centroidOf(b.poly), mid) }))
        .filter(({ i }) => d.per[i] > 0 && built[i].area === c.area && !PLACE_KINDS.includes(built[i].use as TownPlace['kind']) && !this.centreAt(centroidOf(built[i].poly)))
        .sort((x, y) => y.far - x.far);
      const drop = new Set<number>();
      const want = d.centre / DENSEST;
      for (const { i } of spare) {
        if (rest / Math.max(1, d.open[1]) < want || built.length - drop.size <= 0 || this.countIn(c.area) - drop.size <= least) break;
        drop.add(i);
        rest -= d.per[i];
      }
      if (drop.size) {
        const keep = built.filter((_, i) => !drop.has(i));
        built.length = 0;
        built.push(...keep);
        // (The buckets anew, by the buildings' new indices.)
        this.buckets.clear();
        built.forEach((b, at) => this.bucket(b.poly, at));
      }
    }
  }

  /** By every farmstead: a farm building and a barn or two, behind it or beside. */
  farms(rand: Rand): void {
    const { roads } = this;
    const FARM = K('farm', [10, 14], [8, 11], [1, 2], ['gable']);
    const BARN = K('barn', [12, 18], [9, 12], [2, 2], ['gable']);
    for (const s of this.slots) {
      if (s.lot !== 'farm') continue;
      const r = this.rectOf(s);
      const mid: Pt = [r.x + r.w / 2, r.z + r.d / 2];
      const front = ([[0, -1], [-1, 0], [0, 1], [1, 0]] as Pt[])[s.facing];
      const aside: Pt = [-front[1], front[0]];
      const [depth, width] = s.facing % 2 ? [r.w, r.d] : [r.d, r.w];
      // (Turned to the road nearest, as every building is.)
      const near = this.nearestWay(mid[0], mid[1], 200);
      const t: Pt = near ? pointAt(roads[near.road].points, near.s).t : aside;
      const n: Pt = [-t[1], t[0]];
      const want = between(rand, 2, 3);
      let made = 0;
      for (const off of [4, 9, 15]) {
        for (const [u, v] of [[0, 1], [0.6, 1], [-0.6, 1], [1, 0], [-1, 0], [1, 0.6], [-1, 0.6]] as const) {
          if (made >= want) break;
          const kind = made === 0 ? FARM : BARN;
          const [w, d] = [range(rand, ...kind.w), range(rand, ...kind.d)];
          // Behind the lot (`v`) or beside it (`u`), `off` m clear of it and its own half again.
          const c = add(add(mid, front, -v * (depth / 2 + off + 7)), aside, u * (width / 2 + off + 9));
          const at = this.put(add(c, n, -d / 2), t, n, w, d, kind, rand);
          if (at < 0) continue;
          // (A farm building's name is still drawn, as when it was a place, so the draws after it stay as they were.)
          if (made === 0) pick(rand, FARM_BUILDINGS);
          made++;
        }
      }
    }
  }

  /** The names for places in area `a` (see `namerFor`), none of `taken`. */
  private namer(a: number, rand: Rand, taken: Set<string>): Namer {
    const { roads, areas } = this;
    const lastNames = this.content.names.last.length ? this.content.names.last : ['Smith'];
    const streets = areas[a].roads.map((r) => roads[r].name).filter((n): n is string => !!n && !/Trail|Path/.test(n));
    return namerFor(rand, areas[a].name, this.sizeOf(a), lastNames, streets, taken);
  }

  /**
   * The places: every public lot's (`placeLots.ts` `slotPlace`), a named place's (a civic place's, a shop's,
   * an office's...: `macro.ts` puts and names them) as its lot is called, another's (the sim's own shops,
   * the park) as its lot is unless a place has that name already (then with its area's name before it, as
   * a hamlet's café is called by its hamlet's). No backdrop building is a place. Then every career not
   * working in a main town yet goes to its first office.
   */
  places(rand: Rand): void {
    const { slots, areas, placeList: places, t, content } = this;
    const categories = content.careerCategories;
    const draft = { name: areas[0]?.name ?? '', open: { areas } };
    const taken = new Set<string>(slots.flatMap((s) => (s.kind !== 'house' && s.dress ? [s.name] : [])));
    for (const s of slots) {
      if (s.kind === 'house') continue;
      places.push(s.dress ? slotPlace(draft, t, content, s) : slotPlace(draft, t, content, s, this.namer(s.area ?? 0, rand, taken)));
    }
    // Every career a resident may hold works somewhere in each main town: any its places leave out
    // (content added since) go to its first office.
    for (const centre of this.centres) {
      const here = places.filter((p) => p.area === centre.area);
      const office = here.find((p) => p.kind === 'office');
      if (!office) continue;
      for (const c of categories) {
        if (here.some((p) => p.work?.includes(c.id))) continue;
        office.work = [...(office.work ?? []), c.id];
        office.hours = hoursFor(office.hours ?? HOURS.office, [c.id], categories);
      }
    }
  }

  /** The buildings (a draft's first, then the new) and the places. */
  result(): { backdrop: BackdropBuilding[]; places: TownPlace[] } {
    return { backdrop: [...this.built], places: [...this.placeList] };
  }
}

/** The backdrop and the places of a big world: see the top of this file. */
export function layBackdrop(input: BackdropInput): { backdrop: BackdropBuilding[]; places: TownPlace[] } {
  const { rand } = input;
  const pass = new BackdropPass(input);
  // Rows along the roads: the centres shoulder to shoulder, the rest with gaps.
  const rows: Row[] = [];
  input.areas.forEach((area, a) => {
    for (const r of area.roads) if (input.roads[r].kind !== 'trail') rows.push(...pass.rowsAlong(a, r, [1, -1], rand));
  });
  pass.secondRow(rows, rand);
  pass.infill(rand);
  pass.thin();
  pass.farms(rand);
  pass.places(rand);
  const { backdrop, places } = pass.result();
  return { backdrop, places };
}
