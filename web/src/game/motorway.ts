/**
 * The motorway: one in every world where it fits, edge to edge past the main town, on overpasses over the roads it
 * crosses and on a bridge over its water, with one half-diamond junction to the town. The generator lays it in two
 * goes (`macro.ts` `generateFromSpec`):
 *
 * - **`routeMotorway`**, after the main town (and a river's bridge, an island's causeways) and before the villages:
 *   the best of `CANDIDATES` cubic curves from one edge of the world to the opposite one (either way across), passing
 *   the main town `TOWN_CLEAR` m off its box at least and `TOWN_REACH` m at most, `HAMLET_CLEAR` m off any village's
 *   given, clear of the roads laid already, bending no tighter than `BEND_RADIUS`, over the water at most once between
 *   its ends, `WATER_ANGLE` or squarer to it and `WATER_MOST` m at most (and at an end off an island's coast to the
 *   world's edge, `WATER_END` m at most), and with a profile within its grade (`MOTORWAY_GRADE`) cut or filled no more
 *   than `CUT` (`profileOf`), standing `OVER` m over the land at each crossing. Scored by its cut and fill, its water
 *   and how far it passes from the town; of the best that fit, the one leaving the villages the most room, none
 *   costing the world a village (`MotorwaySite.room`). The villages then keep `HAMLET_CLEAR` m off
 *   it, further by its junction (`clearOfMotorway`), and once they are down its gates are chosen again by them
 *   (`withVillages`).
 *   Its middle line is offset into two one-way `motorway` carriageways `APART` m apart, laid for traffic on the right as
 *   the game shows it (`carriageways`: each has the other on the left of the way its cars go, sim-core `cars::right_of`).
 *   The country roads (`CountryRouter`) may
 *   not cross it (`blocked`, `BAND` m either side of its middle line) but at 1 to 3 gates square to it (`gates`), the
 *   first the nearest the main town (`junction`, kept clear wider and longer for its ramps).
 * - **`finishMotorway`**, once the country roads are laid: where they pass under it (`crossings`) its carriageways are
 *   `raised` (spans closer than `MERGE_SPANS` m made one), over its water a `bridge`, each crossing listed in
 *   `crosses` on both roads (none if its profile can't stand clear over each as low as that road could lie there:
 *   `clearsUnder`); and its junction (`halfDiamond`) at its junction's gate: an off-ramp from one carriageway and an
 *   on-ramp onto the other, both on one side of the road under it, each about 125 m, merging into its carriageway at one
 *   end and meeting the road under it in a T `RAMP_T` m out from the carriageway at the other.
 *
 * A world it doesn't fit (mountains, an island whose roads leave no room for its junction, a small world it would
 * cost a village) has none: `failed` says why, and the world is laid as it was. All its maths is `pmath`'s (the same towns on every engine).
 */

import type { RoadPlan } from '../core/protocol';
import { isSeparated, ROAD_KIND } from '../core/roadKinds';
import { CROSS_CLEAR, CUT_MAX_MOTORWAY, cutMostOf } from '../render/shared/builtSurface';
import { CELL, type Cells } from './countryRouter';
import { curve, dist, lengths, MEET, mergeSpans, nearest, nearestOn, pointAt, resample, rightOf, RoadIndex, round2, segmentAngle, gridKey, segmentCross, segmentGrid, SEG_CELL, sharpestTurn, tangent, type Pt } from './roads';
import { mulberry32 } from './noise';
import type { Rect } from './town';
import * as pmath from './pmath';

/** How far apart the carriageways' middles lie (m), and each one's width. */
export const APART = 9;
const WIDTH = ROAD_KIND.motorway.width;
/** From the middle line to a carriageway's outer edge (m). */
const OUTER = APART / 2 + WIDTH / 2;
/** How many courses it tries. */
export const CANDIDATES = 24;
/** The tightest it bends (m of radius), measured over `BEND_SPAN` m either way. */
export const BEND_RADIUS = 150;
const BEND_SPAN = 25;
/** How far its outer edge keeps off the main town's box, and a village's (m); how far its middle line passes from the town's box at most. */
export const TOWN_CLEAR = 50;
export const HAMLET_CLEAR = 30;
export const TOWN_REACH = 220;
/** How far from the town's box its middle line is best passing (m): clear of the lots spilling out along the roads from it, near enough for the way to its junction. */
const REACH_AIM = 150;
/** How far its middle line keeps off the world's edges but the two it runs between (m). */
export const SIDE_CLEAR = 40;
/** Its courses along an edge of the world (`candidates`) this far in from `SIDE_CLEAR` and `TOWN_REACH` (m: room for the road under its junction to go on beyond it, `gatesOf`). */
const EDGE_IN = 40;
/** How far its middle line keeps off the roads laid before it (bridges, causeways: m). */
const ROAD_CLEAR = 40;
/** Its water: crossed this square or squarer (radians), each crossing this long at most (m), crossings at most. */
export const WATER_ANGLE = Math.PI / 3;
const WATER_MOST = 100;
const WATER_CROSSINGS = 1;
/** At an end, off the coast to the world's edge (an island's sea), this long at most (m). */
const WATER_END = 90;
/** Its deck over the water reaches this far past the banks (m). */
const DECK_PAST = 8;
/** The grade it is routed within (a little within its kind's, `ROAD_KIND.motorway.grade`), and the most it is cut or filled (m, within `CUT_MAX_MOTORWAY`). */
const MOTORWAY_GRADE = ROAD_KIND.motorway.grade * 0.85;
const CUT = CUT_MAX_MOTORWAY - 1;
/** How high over the land it stands where a road passes under it (m): the road's clearance, the deck and a spare. */
const OVER = 7;
/** A road from the town to the junction's gate climbs no more than this share of a country road's grade on the way (`macro.ts` `REACHABLE`). */
const REACHABLE = 0.7;
const COUNTRY_GRADE = ROAD_KIND.country.grade;
/** How high over the lower of its banks its bridge's deck may stand (m: the land's surface holds a deck low over its water). */
const WATER_DECK = 2.5;
/** How far apart its profile is sampled (m). */
const STEP = 4;
/** The country roads keep this far off its middle line (m), but at a gate. */
const BAND = 20;
/** About the junction's gate, wider and longer: no road but the one under it within `JUNCTION_BAND` m of the middle line for `JUNCTION_REACH` m either way. */
const JUNCTION_BAND = 36;
const JUNCTION_REACH = 150;
/** A village's box keeps this far off the middle line about the junction's gate (m: its ramps, their T and a village's gap). */
const JUNCTION_CLEAR = JUNCTION_BAND + 24;
/** No cell within this of an area's box is kept from the country roads (its ways out go on from there). */
const AREA_OPEN = 24;
/** Gates: at most this many, this far apart along it (m), this far from its ends and from its water (m), crossed within `GATE_SQUARE` (radians) of square. */
const GATES = 3;
const GATE_APART = 260;
const GATE_END = 160;
const GATE_WATER = 90;
const GATE_SQUARE = (20 * Math.PI) / 180;
/** A gate's place sought this far either way of where it is wanted (m). */
const GATE_LOOK = 80;
/** An overpass reaches at least this far either side of the road under it (m), and spans closer than `MERGE_SPANS` m are one. */
const OVER_HALF = 15;
export const MERGE_SPANS = 150;
/** A crossing this much squarer than not (radians) at least. */
export const CROSS_ANGLE = (45 * Math.PI) / 180;
/**
 * A ramp: from its merge, `RAMP_ALONG` m along its carriageway from where it meets the road under it, out over
 * `RAMP_TAPER` m to `RAMP_BESIDE` m beside it (their edges touching), along beside it, then out to the road under it,
 * meeting it in a T `RAMP_T` m from the carriageway's middle. Its length is about 125 m.
 */
const RAMP_WIDTH = ROAD_KIND.ramp.width;
const RAMP_ALONG = 118;
const RAMP_TAPER = 40;
const RAMP_BESIDE = WIDTH / 2 + RAMP_WIDTH / 2;
export const RAMP_T = 25;
/** The ramp's run in to its T (m along the carriageway before it), tried in turn; and its T's distance out, tried in turn (a road under it slanting away). */
const RAMP_INS = [38, 26, 52, 16, 70];
const RAMP_TS = [RAMP_T, RAMP_T + 7, RAMP_T + 14];
/** No other road's end within this of a ramp's T (m): another junction too near. */
const TEE_APART = 25;
/** A ramp's length (m): within these. */
export const RAMP_LENGTH: [number, number] = [110, 140];
/** How many courses that fit are tried for the villages' room at most (`MotorwaySite.room`). */
const ROOM_TRIES = 6;
/** The draws its course and names come from (apart from the world's). */
const MOTOR = 0x6d6f746f;

const offRect = (r: Rect, [x, z]: Pt) => pmath.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d));

/** A gate the country roads cross the motorway by: where along its middle line (m), there, and the way across (a unit step of the land's cells). */
export interface Gate {
  s: number;
  p: Pt;
  across: Pt;
}

/** The motorway's course, before the country roads: see the top of this file. */
export interface MotorwayRoute {
  /** Its middle line, a point every metre or so, from one edge of the world to the other. */
  middle: Pt[];
  /**
   * Its carriageways: the first's cars go the way the middle line runs, the second's against it. The first is off the
   * middle line to the right of its way as the game shows it: `rightOf`'s other side, as `rightOf` is the right on the
   * town map, which draws z down the page (the game's view is that mirrored, `render/pc/space.ts`).
   */
  ways: [RoadPlan, RoadPlan];
  /** Per land cell (`Cells`), 1 where a country road may not go. */
  blocked: Uint8Array;
  /** The gates, the junction's first. */
  gates: Gate[];
  /** Its base name (`M5`), and its junction's number. */
  name: string;
  junction: number;
  /** Its profile's samples along its middle line (every `STEP` m, NaN over its water): where a gate may lift it over a road (`withVillages`). */
  heights: number[];
  /** The land's height at (x, z) its course was judged on (`MotorwaySite.height`): the roads under it judged on it too (`clearsUnder`). None: not judged. */
  height?: (x: number, z: number) => number;
}

/** A road crossing the motorway's middle line: which, how far along it (m) and along the middle line (m), and how square (radians from along). */
export interface Crossing {
  road: number;
  s: number;
  m: number;
  angle: number;
}

/** The land's height at (x, z) as `site` has it seeded (its own, else its cells', between their middles). */
const heightOf = (site: MotorwaySite) => site.height ?? ((x: number, z: number) => site.land.heightAt(x, z));

/** A cubic Bézier through `p` (its four points), a point about every metre. */
function bezier([a, b, c, d]: readonly Pt[]): Pt[] {
  const rough = dist(a, b) + dist(b, c) + dist(c, d);
  const n = Math.max(8, Math.ceil(rough * 2));
  const fine: Pt[] = [];
  for (let k = 0; k <= n; k++) {
    const t = k / n;
    const u = 1 - t;
    const [w0, w1, w2, w3] = [u * u * u, 3 * u * u * t, 3 * u * t * t, t * t * t];
    fine.push([w0 * a[0] + w1 * b[0] + w2 * c[0] + w3 * d[0], w0 * a[1] + w1 * b[1] + w2 * c[1] + w3 * d[1]]);
  }
  return resample(fine, 1);
}

/** `pts` moved `by` m to its right (the way its points run), each point square off it (its way over two points either side). */
function offset(pts: readonly Pt[], by: number): Pt[] {
  return pts.map((p, i) => {
    const r = rightOf(tangent(pts, i, 2));
    return [p[0] + r[0] * by, p[1] + r[1] * by];
  });
}

/**
 * `pts` cut to the world (`W` × `D`, `inset` m in from its edges): the run of it inside, each end exactly on the edge
 * where it crosses it; null if it runs out of the world and back in (or never in).
 */
function clipTo(pts: readonly Pt[], W: number, D: number, inset = 0.5): Pt[] | null {
  const within = ([x, z]: Pt) => x >= inset && z >= inset && x <= W - inset && z <= D - inset;
  const first = pts.findIndex(within);
  if (first < 0) return null;
  let last = first;
  while (last + 1 < pts.length && within(pts[last + 1])) last++;
  for (let k = last + 1; k < pts.length; k++) if (within(pts[k])) return null;
  const edge = (a: Pt, b: Pt): Pt => {
    let t = 1;
    for (const [v, dv] of [
      [a[0], b[0] - a[0]],
      [a[1], b[1] - a[1]],
    ] as const) {
      const hi = v === a[0] ? W - inset : D - inset;
      if (v + dv * t < inset) t = (inset - v) / dv;
      if (v + dv * t > hi) t = (hi - v) / dv;
    }
    return [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
  };
  const out = pts.slice(first, last + 1);
  // (A point inside within half a metre of the edge gives way to the point on it: no stub of a step at an end.)
  if (first > 0) {
    const e = edge(pts[first], pts[first - 1]);
    if (out.length > 2 && dist(e, out[0]) < 0.5) out[0] = e;
    else out.unshift(e);
  }
  if (last < pts.length - 1) {
    const e = edge(pts[last], pts[last + 1]);
    if (out.length > 2 && dist(e, out[out.length - 1]) < 0.5) out[out.length - 1] = e;
    else out.push(e);
  }
  return out;
}

/** Which edge of the world (north, east, south, west: `FACING`'s order) `p` lies on, within a metre; -1 none. */
function edgeOf([x, z]: Pt, W: number, D: number): number {
  return [z <= 1, x >= W - 1, z >= D - 1, x <= 1].indexOf(true);
}

/**
 * A profile along a line sampled every `STEP` m: the land's heights `h` (NaN over the water: its bridge, held low over
 * it, as the land's surface holds a bridge's deck, no more than `WATER_DECK` over the lower of its banks), held within
 * `grade` (`MOTORWAY_GRADE`) and no more than `most` (`CUT`) off the land, and over it by `lift[k]` at least where given.
 * The heights, and how much it cuts and fills (m², summed over the samples); null if no such profile exists.
 */
export function profileOf(h: readonly number[], lift: ReadonlyMap<number, number> = new Map(), grade = MOTORWAY_GRADE, most = CUT): { y: number[]; cut: number } | null {
  const n = h.length;
  const g = grade * STEP;
  const lo = new Array<number>(n);
  const hi = new Array<number>(n);
  // (Over the water: its banks' lower.)
  const bank = new Array<number>(n).fill(NaN);
  for (let k = 0; k < n; k++) {
    if (!Number.isNaN(h[k]) || !Number.isNaN(bank[k])) continue;
    let e = k;
    while (e + 1 < n && Number.isNaN(h[e + 1])) e++;
    const sides = [h[k - 1], h[e + 1]].filter((v) => v !== undefined && !Number.isNaN(v));
    bank.fill(sides.length ? Math.min(...sides) : Infinity, k, e + 1);
  }
  // (Lifted: on its deck, as high as it likes.)
  const bound = (k: number): [number, number] =>
    Number.isNaN(h[k]) ? (Number.isFinite(bank[k]) ? [bank[k] - 1, bank[k] + WATER_DECK] : [-Infinity, Infinity]) : lift.has(k) ? [h[k] + lift.get(k)!, Infinity] : [h[k] - most, h[k] + most];
  for (let k = 0; k < n; k++) {
    const [a, b] = bound(k);
    [lo[k], hi[k]] = k === 0 ? [a, b] : [Math.max(a, lo[k - 1] - g), Math.min(b, hi[k - 1] + g)];
    if (lo[k] > hi[k] + 1e-9) return null;
  }
  const y = new Array<number>(n);
  let cut = 0;
  for (let k = n - 1; k >= 0; k--) {
    const want = Number.isNaN(h[k]) ? (k + 1 < n ? y[k + 1] : 0) : h[k];
    const [a, b] = k + 1 < n ? [Math.max(lo[k], y[k + 1] - g), Math.min(hi[k], y[k + 1] + g)] : [lo[k], hi[k]];
    y[k] = Math.max(a, Math.min(b, Number.isFinite(want) ? want : Number.isFinite(a) ? a : b));
    if (!Number.isNaN(h[k])) cut += Math.abs(y[k] - h[k]) * STEP;
  }
  return { y, cut };
}

/** The stretches of `pts` over the water (`wet`): their first and last point's arc lengths (m). */
function wetRuns(pts: readonly Pt[], wet: (x: number, z: number) => boolean, ls = lengths(pts)): [number, number, number, number][] {
  const out: [number, number, number, number][] = [];
  let from = -1;
  for (let i = 0; i <= pts.length; i++) {
    const w = i < pts.length && wet(pts[i][0], pts[i][1]);
    if (w && from < 0) from = i;
    if (!w && from >= 0) {
      out.push([ls[from], ls[i - 1], from, i - 1]);
      from = -1;
    }
  }
  return out;
}

/** What the motorway's course is tried against: the land, the areas (the main town's first), the roads laid already, the water at the metre (`wet`). */
export interface MotorwaySite {
  land: Cells;
  /** The land's height at (x, z) as seeded (m; default its cells', between their middles). */
  height?: (x: number, z: number) => number;
  areas: readonly { rect: Rect; centre: Pt }[];
  roads: readonly RoadPlan[];
  wet: (x: number, z: number) => boolean;
  seed: number;
  /**
   * How many villages would find room with the motorway's land kept clear as `clear` says (the generator's own count,
   * before it places them): of the courses that fit, none leaving room for fewer than none would; the best leaving as
   * many as none would is taken. None given: the best course.
   */
  room?: (clear: (r: Rect) => boolean) => number;
  /** How many of the courses it would take to pass over (the generator's: those it found no room for the junction on, or the country roads no way round). */
  skip?: number;
}


/** A course tried: its middle line, its score (lower better), and its profile's samples. */
interface Course {
  middle: Pt[];
  /** Its whole line, on past the world's edges (its carriageways are offset from it, then cut to the world). */
  line: Pt[];
  score: number;
  heights: number[];
}

/** Why a course is turned away, as `failed` lists the most common. */
type Why = 'edge' | 'side' | 'bend' | 'town' | 'far' | 'village' | 'road' | 'water' | 'grade' | 'gate' | 'room';
const WHY: Record<Why, string> = {
  edge: 'runs out of the world and back',
  side: `runs within ${SIDE_CLEAR} m of the world's edge`,
  gate: 'has no place a road from the town could climb to and pass under it',
  room: 'leaves room for fewer villages',
  bend: `bends tighter than ${BEND_RADIUS} m`,
  town: `comes within ${TOWN_CLEAR} m of the town`,
  far: `passes further than ${TOWN_REACH} m from the town`,
  village: `comes within ${HAMLET_CLEAR} m of a village`,
  road: 'runs into a road',
  water: 'crosses the water too long, too slanting or too often',
  grade: `climbs too steeply for its ${Math.round(ROAD_KIND.motorway.grade * 100)} % within ${CUT_MAX_MOTORWAY} m of cut and fill`,
};

/** The heights its profile reads along `middle` (its arc lengths `ls`), every `STEP` m (NaN over the water). */
function heightsAlong(middle: readonly Pt[], site: MotorwaySite, ls: readonly number[]): number[] {
  const h = heightOf(site);
  return resample(middle, STEP, ls).map(([x, z]) => (site.wet(x, z) ? NaN : h(x, z)));
}

/** Course `middle` judged (see the top of this file): its score, or why it doesn't do. `index`: the roads laid already (not trails), if any. */
function judge(middle: Pt[], line: Pt[], site: MotorwaySite, index: RoadIndex | null): Course | Why {
  const { land, areas, wet } = site;
  const [W, D] = [land.W, land.D];
  const [a, b] = [edgeOf(middle[0], W, D), edgeOf(middle[middle.length - 1], W, D)];
  if (a < 0 || b < 0 || (a + 2) % 4 !== b) return 'edge';
  if (sharpestTurn(middle, BEND_SPAN).angle > BEND_SPAN / BEND_RADIUS) return 'bend';
  // (Not along an edge of the world: out from the one it comes in by, on to the one it leaves by.)
  const lm = lengths(middle);
  const Lm = lm[lm.length - 1];
  for (let i = 0; i < middle.length; i += 4) {
    const [x, z] = middle[i];
    const by = [z, W - x, D - z, x];
    if (by.some((v, k) => k !== a && k !== b && v < SIDE_CLEAR) && lm[i] > SIDE_CLEAR && lm[i] < Lm - SIDE_CLEAR) return 'side';
  }
  const main = areas[0].rect;
  let reach = Infinity;
  for (let i = 0; i < middle.length; i += 2) {
    const p = middle[i];
    const off = offRect(main, p);
    reach = Math.min(reach, off);
    if (off < TOWN_CLEAR + OUTER) return 'town';
    if (areas.some((v, k) => k > 0 && offRect(v.rect, p) < HAMLET_CLEAR + OUTER)) return 'village';
  }
  if (reach > TOWN_REACH) return 'far';
  if (index) for (let i = 0; i < middle.length; i += 4) if (index.nearest(middle[i][0], middle[i][1], ROAD_CLEAR)) return 'road';
  // Its water: once at most between its ends, square enough, not too long; and at an end, off the coast to the
  // world's edge (an island's), `WATER_END` m at most.
  const ls = lm;
  const runs = wetRuns(middle, wet, ls);
  const atEnd = (i0: number, i1: number) => i0 === 0 || i1 === middle.length - 1;
  if (runs.filter(([, , i0, i1]) => !atEnd(i0, i1)).length > WATER_CROSSINGS) return 'water';
  let water = 0;
  for (const [s0, s1, i0, i1] of runs) {
    const along = s1 - s0 + 1;
    if (along > (atEnd(i0, i1) ? WATER_END : WATER_MOST) || (i0 === 0 && i1 === middle.length - 1)) return 'water';
    // (How far the water runs square across it there: a river crossed at `θ` from along it runs `along · tan θ` across.)
    const { p, t } = pointAt(middle, (s0 + s1) / 2, ls);
    const n = rightOf(t);
    let across = 0;
    for (const k of [1, -1]) for (let d = 1; d < 300 && wet(p[0] + n[0] * d * k, p[1] + n[1] * d * k); d++) across++;
    if (across < along * pmath.sin(WATER_ANGLE) / pmath.cos(WATER_ANGLE)) return 'water';
    water += along;
  }
  const heights = heightsAlong(middle, site, ls);
  const made = profileOf(heights);
  if (!made) return 'grade';
  return { middle, line, heights, score: (made.cut / ls[ls.length - 1]) * 20 + water * 0.4 + Math.abs(reach - REACH_AIM) * 0.05 };
}

/**
 * The courses it tries: `CANDIDATES` cubic curves from one edge to the opposite one, passing either side of the town,
 * near and further off; and straight along each edge `EDGE_IN` m in from where it may pass (`SIDE_CLEAR`), or from
 * as far off the town as it may pass (`TOWN_REACH`) if that is nearer.
 */
function candidates(site: MotorwaySite): Pt[][] {
  const { land, areas, seed } = site;
  const [W, D] = [land.W, land.D];
  const rand = mulberry32((seed ^ MOTOR) >>> 0);
  const main = areas[0].rect;
  const out: Pt[][] = [];
  const OUT = 60;
  /**
   * The world along axis `across` (0: along x, west to east, passing north or south of the town; 1: along z, north to
   * south, west or east of it): its length and breadth, the town's box across it, a box's span across it, and a curve
   * through control points given along it.
   */
  const frame = (across: 0 | 1) => ({
    L: across === 0 ? W : D,
    S: across === 0 ? D : W,
    lo: across === 0 ? main.z : main.x,
    hi: across === 0 ? main.z + main.d : main.x + main.w,
    span: (r: Rect): [number, number] => (across === 0 ? [r.z, r.d] : [r.x, r.w]),
    lay: (ctrl: Pt[]) => bezier(across === 0 ? ctrl : ctrl.map(([x, z]): Pt => [z, x])),
  });
  for (const across of [0, 1] as const) {
    const { L, S, lo, hi, span, lay } = frame(across);
    for (const side of [-1, 1]) {
      // (Where a line straight along that side of the town would pass clear of the villages: those offsets first.)
      const clear = (off: number) => {
        const at = side < 0 ? lo - off : hi + off;
        return areas.every((v, k) => {
          const [a, w] = span(v.rect);
          return k === 0 || a - at > HAMLET_CLEAR + OUTER + 10 || at - a - w > HAMLET_CLEAR + OUTER + 10;
        });
      };
      const open: number[] = [];
      for (let off = TOWN_CLEAR + OUTER + 6; off <= TOWN_REACH - 10; off += 10) if (clear(off)) open.push(off);
      const offs = open.length ? [open[0], open[Math.floor(open.length / 2)], open[open.length - 1]] : [80, 150, 210];
      for (const off of offs) {
        const pass = side < 0 ? lo - off : hi + off;
        for (let v = 0; v < 2; v++) {
          const tilt = () => pass + (rand() - 0.5) * S * (v === 0 ? 0.12 : 0.5);
          const [e0, e1] = [Math.max(40, Math.min(S - 40, tilt())), Math.max(40, Math.min(S - 40, tilt()))];
          if (pass < 40 || pass > S - 40) continue;
          const k = 0.35;
          out.push(
            lay([
              [-OUT, e0],
              [L * 0.36, pass + (e0 - pass) * k],
              [L * 0.64, pass + (e1 - pass) * k],
              [L + OUT, e1],
            ]),
          );
        }
      }
    }
  }
  // And straight along the edges past the town as far off it as it may pass (where the villages could go between).
  const drawn = out.slice(0, CANDIDATES);
  for (const across of [0, 1] as const) {
    const { L, S, lo, hi, lay } = frame(across);
    for (const side of [-1, 1]) {
      const pass = side < 0 ? Math.max(SIDE_CLEAR + EDGE_IN, lo - TOWN_REACH + EDGE_IN) : Math.min(S - SIDE_CLEAR - EDGE_IN, hi + TOWN_REACH - EDGE_IN);
      if ((side < 0 ? lo - pass : pass - hi) < TOWN_CLEAR + OUTER + 6) continue;
      drawn.push(
        lay([
          [-OUT, pass],
          [L * 0.36, pass],
          [L * 0.64, pass],
          [L + OUT, pass],
        ]),
      );
    }
  }
  return drawn;
}

/** The base names a motorway may have (`M` and a number), and its junctions' numbers. */
function namesOf(seed: number): { name: string; junction: number } {
  const rand = mulberry32((seed ^ MOTOR ^ 0x6e616d65) >>> 0);
  return { name: `M${1 + Math.floor(rand() * 69)}`, junction: 2 + Math.floor(rand() * 28) };
}

/** Its carriageways off middle line `middle` (running on past the world's edges), cut to the world: see `MotorwayRoute.ways`. */
export function carriageways(middle: readonly Pt[], W: number, D: number): [RoadPlan, RoadPlan] | null {
  const ways = ([1, -1] as const).map((side) => {
    // (Traffic on the right as the game shows it: the first off the middle line to `rightOf`'s other side.)
    const pts = clipTo(offset(middle, (-side * APART) / 2), W, D);
    return pts && pts.length >= 2 ? { points: pts.map(([x, z]): Pt => [round2(x), round2(z)]), width: WIDTH, ends: ['out', 'out'], kind: 'motorway', oneway: side } satisfies RoadPlan : null;
  });
  return ways[0] && ways[1] ? [ways[0], ways[1]] : null;
}

/**
 * The motorway's course over `site` (see the top of this file): its middle line, carriageways (not yet named, nor
 * raised), the cells the country roads keep off and their gates; or why none fits.
 */
export function routeMotorway(site: MotorwaySite): MotorwayRoute | { failed: string } {
  const { land } = site;
  const [W, D] = [land.W, land.D];
  const why = new Map<Why, number>();
  const judgeAll = (lines: readonly Pt[][]) => {
    const fit: Course[] = [];
    // (The roads laid already, not trails, as every course is judged against them.)
    const others = site.roads.filter((r) => r.kind !== 'trail');
    const index = others.length ? new RoadIndex(others) : null;
    for (const line of lines) {
      const middle = clipTo(line, W, D, 0);
      const judged = middle ? judge(middle, line, site, index) : 'edge';
      if (typeof judged === 'string') why.set(judged, (why.get(judged) ?? 0) + 1);
      else fit.push(judged);
    }
    return fit;
  };
  /** The room the villages would have with none (`MotorwaySite.room`). */
  const full = site.room?.(() => true) ?? 0;
  /**
   * Of courses `fit`, the best first with a gate a road from the town could climb to (`gatesOf`), the first
   * `ROOM_TRIES` of those, the most room they leave the villages first (the better of two leaving as much); the
   * first `want` of them (no more worked out than that needs), and the room each leaves.
   */
  const ranked = (fit: Course[], want: number): { route: MotorwayRoute; room: number }[] => {
    fit.sort((a, b) => a.score - b.score);
    const out: { route: MotorwayRoute; room: number }[] = [];
    let tries = 0;
    for (const best of fit) {
      const gates = gatesOf(best, site);
      if (!gates.length) {
        why.set('gate', (why.get('gate') ?? 0) + 1);
        continue;
      }
      const ways = carriageways(best.line, W, D);
      if (!ways) continue;
      const route: MotorwayRoute = { middle: best.middle, ways, blocked: blockedOf(best.middle, gates, land, site.areas), gates, ...namesOf(site.seed), heights: best.heights, height: heightOf(site) };
      const room = site.room ? site.room((r) => clearOfMotorway(route, r)) : full;
      tries++;
      // (Not where it leaves the villages too little room.)
      if (room < full) why.set('room', (why.get('room') ?? 0) + 1);
      else out.push({ route, room });
      // (One leaving as much room as none would: none after it is put before it.)
      if ((want === 1 && room >= full) || tries >= ROOM_TRIES) break;
    }
    return out.sort((a, b) => b.room - a.room);
  };
  // (Passed over to the next: only one leaving as many villages room as the first.)
  const skip = site.skip ?? 0;
  const all = ranked(judgeAll(candidates(site)), skip + 1);
  const route = all[skip];
  if (route && route.room >= all[0].room) return route.route;
  if (skip > 0) return { failed: 'No motorway: no course tried leaves its junction room and the country roads a way round it.' };
  // (Why: the villages' room if a course that fits was turned away for it, else what turned the most away.)
  const most = why.has('room') ? (['room', 1] as const) : [...why.entries()].sort((a, b) => b[1] - a[1])[0];
  return { failed: `No motorway: ${most?.[0] === 'room' ? 'every course that fits' : 'every course tried'} ${most ? WHY[most[0]] : 'fails'}.` };
}

/**
 * Route `route`, chosen before the villages, once they are down (`site.areas`, the main town's first): its gates
 * chosen again (the junction's as it was, then by the villages across it from the town and along its longest
 * stretches) and the cells the country roads keep off with them.
 */
export function withVillages(route: MotorwayRoute, site: MotorwaySite): MotorwayRoute {
  const gates = gatesOf(route, site);
  if (!gates.length) return route;
  return { ...route, gates, blocked: blockedOf(route.middle, gates, site.land, site.areas) };
}

/**
 * Whether a village's box `rect` keeps clear of route `route` (as the generator places the villages after it):
 * `HAMLET_CLEAR` m off its carriageways' outer edges, and `JUNCTION_CLEAR` m off its middle line by its junction's
 * gate, where its ramps and the road under it go.
 */
export function clearOfMotorway(route: Pick<MotorwayRoute, 'middle' | 'gates'>, rect: Rect): boolean {
  const { middle, gates } = route;
  const junction = gates[0]?.s ?? -Infinity;
  const { ls, boxes } = chunksOf(middle);
  const reach = Math.max(HAMLET_CLEAR + OUTER, JUNCTION_CLEAR);
  for (const [i0, i1, x0, z0, x1, z1] of boxes) {
    // (A stretch whose box lies further off than either keeps: none of its points nearer.)
    if (x0 - reach > rect.x + rect.w || rect.x - reach > x1 || z0 - reach > rect.z + rect.d || rect.z - reach > z1) continue;
    for (let i = i0; i <= i1; i++) {
      const off = offRect(rect, middle[i]);
      if (off < HAMLET_CLEAR + OUTER || (off < JUNCTION_CLEAR && Math.abs(ls[i] - junction) <= JUNCTION_REACH)) return false;
    }
  }
  return true;
}

/** A middle line's lengths, and its stretches of `CHUNK` points with their boxes (first and last point, x and z least and most), worked out once. */
const chunked = new WeakMap<readonly Pt[], { ls: number[]; boxes: [number, number, number, number, number, number][] }>();
const CHUNK = 32;
function chunksOf(middle: readonly Pt[]) {
  let got = chunked.get(middle);
  if (!got) {
    const boxes: [number, number, number, number, number, number][] = [];
    for (let i0 = 0; i0 < middle.length; i0 += CHUNK) {
      const i1 = Math.min(middle.length - 1, i0 + CHUNK - 1);
      const pts = middle.slice(i0, i1 + 1);
      boxes.push([i0, i1, Math.min(...pts.map((p) => p[0])), Math.min(...pts.map((p) => p[1])), Math.max(...pts.map((p) => p[0])), Math.max(...pts.map((p) => p[1]))]);
    }
    got = { ls: lengths(middle), boxes };
    chunked.set(middle, got);
  }
  return got;
}

/** The gates on course `c` (see `MotorwayRoute.gates`): the junction's nearest the main town first; none if it has no room for one. */
function gatesOf(c: Pick<Course, 'middle' | 'heights'>, site: MotorwaySite): Gate[] {
  const { middle, heights } = c;
  const ls = lengths(middle);
  const L = ls[ls.length - 1];
  const runs = wetRuns(middle, site.wet, ls);
  const town = site.areas[0].centre;
  /** The octant step nearest square across at `s`, and how far off square it is (worked out once a spot). */
  const squares = new Map<number, { across: Pt; off: number }>();
  const squareAt = (s: number): { across: Pt; off: number } => {
    const had = squares.get(s);
    if (had) return had;
    const { t } = pointAt(middle, s, ls);
    const n = rightOf(t);
    const k = Math.round(pmath.atan2(n[1], n[0]) / (Math.PI / 4));
    const a = (k * Math.PI) / 4;
    const across: Pt = [Math.round(pmath.cos(a)), Math.round(pmath.sin(a))];
    const off = pmath.acos(Math.min(1, Math.abs(n[0] * across[0] + n[1] * across[1]) / pmath.hypot(across[0], across[1])));
    squares.set(s, { across, off });
    return { across, off };
  };
  const chosen: Gate[] = [];
  const lifts = new Map<number, number>();
  /** Lifting the profile at `s` (and the gates chosen) still leaves one within its grade. */
  const liftable = (s: number) => {
    const at = new Map(lifts);
    const k0 = Math.round(s / STEP);
    for (let k = k0 - 3; k <= k0 + 3; k++) if (k >= 0 && k < heights.length) at.set(k, OVER);
    return profileOf(heights, at) ? at : null;
  };
  /** Whether a gate may go at `s` whatever the gates chosen: within its ends, off its water, square enough, and across it in the world and dry (worked out once a spot). */
  const fits = new Map<number, boolean>();
  const fitsAt = (s: number) => {
    let got = fits.get(s);
    if (got !== undefined) return got;
    got = s >= GATE_END && s <= L - GATE_END && runs.every(([s0, s1]) => s < s0 - GATE_WATER || s > s1 + GATE_WATER) && squareAt(s).off <= GATE_SQUARE;
    if (got) {
      const { p, t } = pointAt(middle, s, ls);
      const n = rightOf(t);
      got =
        // (Its way across in the world, out past where the country roads keep off.)
        [-1, 1].every((k) => {
          const q: Pt = [p[0] + n[0] * k * (JUNCTION_BAND + CELL * 4), p[1] + n[1] * k * (JUNCTION_BAND + CELL * 4)];
          return q[0] > 8 && q[1] > 8 && q[0] < site.land.W - 8 && q[1] < site.land.D - 8;
        }) &&
        // (Dry across it, out past where the country roads keep off.)
        [-1, 1].every((k) => {
          for (let d = 0; d <= JUNCTION_BAND + CELL; d += 4) if (site.wet(p[0] + n[0] * d * k, p[1] + n[1] * d * k)) return false;
          return true;
        });
    }
    fits.set(s, got);
    return got;
  };
  const ok = (s: number, apart: number) => chosen.every((g) => Math.abs(g.s - s) >= apart) && fitsAt(s);
  /** The best gate within `GATE_LOOK` m of `want` (the squarest, then the nearest), if one may go there. */
  const place = (want: number, apart: number, allow: (s: number) => boolean = () => true): Gate | null => {
    let pick: { s: number; score: number; lift: Map<number, number> } | null = null;
    for (let s = Math.max(0, want - GATE_LOOK); s <= Math.min(L, want + GATE_LOOK); s += 4) {
      if (!ok(s, apart) || !allow(s)) continue;
      const score = squareAt(s).off * 200 + Math.abs(s - want);
      if (pick && score >= pick.score) continue;
      const lift = liftable(s);
      if (lift) pick = { s, score, lift };
    }
    if (!pick) return null;
    for (const [k, v] of pick.lift) lifts.set(k, v);
    const gate = { s: pick.s, p: pointAt(middle, pick.s, ls).p, across: squareAt(pick.s).across };
    chosen.push(gate);
    return gate;
  };
  // The junction's: the nearest the main town (where it may go: tried outwards from there), where a road from the town
  // could climb to it.
  const box = site.areas[0].rect;
  const reach = (s: number) => {
    const p = pointAt(middle, s, ls).p;
    const q: Pt = [Math.max(box.x, Math.min(box.x + box.w, p[0])), Math.max(box.z, Math.min(box.z + box.d, p[1]))];
    const h = heightOf(site);
    return Math.abs(h(p[0], p[1]) - h(q[0], q[1])) <= REACHABLE * COUNTRY_GRADE * dist(p, q);
  };
  const byTown = ls.map((s, i) => ({ s, d: dist(middle[i], town) })).filter((_, i) => i % 8 === 0).sort((a, b) => a.d - b.d || a.s - b.s);
  for (const { s } of byTown) {
    if (place(s, 0, reach)) break;
    if (byTown.length && dist(pointAt(middle, s, ls).p, town) > TOWN_REACH * 2.5) break;
  }
  if (!chosen.length) return [];
  // Then by the villages across it from the town, then in the middle of the longest stretch left.
  const sideOf = (p: Pt) => {
    const m = nearest(middle, p[0], p[1], ls);
    return m.side;
  };
  const townSide = sideOf(town);
  const apart = Math.max(GATE_APART, JUNCTION_REACH + GATE_LOOK);
  for (const v of site.areas.slice(1)) {
    if (chosen.length >= GATES) break;
    if (sideOf(v.centre) === townSide) continue;
    place(nearest(middle, v.centre[0], v.centre[1], ls).s, apart);
  }
  while (chosen.length < GATES) {
    const cuts = [0, ...chosen.map((g) => g.s).sort((a, b) => a - b), L];
    let longest = [0, 0];
    for (let k = 0; k + 1 < cuts.length; k++) if (cuts[k + 1] - cuts[k] > longest[1] - longest[0]) longest = [cuts[k], cuts[k + 1]];
    if (longest[1] - longest[0] < apart * 2) break;
    if (!place((longest[0] + longest[1]) / 2, apart)) break;
  }
  return chosen;
}

/** The land cells the country roads keep off (see `MotorwayRoute.blocked`): `BAND` m of the middle line, wider about the junction's gate, but the gates and by the areas. */
function blockedOf(middle: readonly Pt[], gates: readonly Gate[], land: Cells, areas: readonly { rect: Rect }[]): Uint8Array {
  const { gw, gd } = land;
  const out = new Uint8Array(gw * gd);
  const { ls } = chunksOf(middle);
  const junction = gates[0];
  const open = (c: Pt) => areas.every((a) => offRect(a.rect, c) >= AREA_OPEN);
  for (let i = 0; i < middle.length; i += 2) land.mark(out, middle[i], Math.abs(ls[i] - junction.s) <= JUNCTION_REACH ? JUNCTION_BAND : BAND, open);
  for (const [k, g] of gates.entries()) {
    const reach = (k === 0 ? JUNCTION_BAND : BAND) + CELL * 2;
    const [cx, cz] = [Math.floor(g.p[0] / CELL), Math.floor(g.p[1] / CELL)];
    const [dx, dz] = g.across;
    const n = Math.ceil(reach / (CELL * pmath.hypot(dx, dz))) + 1;
    for (let j = -n; j <= n; j++) {
      const open = (x: number, z: number) => {
        if (x >= 0 && z >= 0 && x < gw && z < gd) out[z * gw + x] = 0;
      };
      open(cx + j * dx, cz + j * dz);
      // (A diagonal way through two cells wide: no corner of a blocked cell cut.)
      if (dx && dz) open(cx + j * dx + dx, cz + j * dz);
    }
  }
  return out;
}

/**
 * Where a road under the motorway at its junction's gate runs through it (`MotorwayRoute.gates`' first): the gate's
 * cells' ends either side, the near one on the side `town` is on, and the way across from it (a unit vector).
 */
export function gateEnds(route: MotorwayRoute, town: Pt): { near: Pt; far: Pt; dir: Pt } {
  const g = route.gates[0];
  const c: Pt = [(Math.floor(g.p[0] / CELL) + 0.5) * CELL, (Math.floor(g.p[1] / CELL) + 0.5) * CELL];
  const l = pmath.hypot(g.across[0], g.across[1]);
  let dir: Pt = [g.across[0] / l, g.across[1] / l];
  if ((town[0] - c[0]) * dir[0] + (town[1] - c[1]) * dir[1] > 0) dir = [-dir[0], -dir[1]];
  const reach = (Math.ceil((JUNCTION_BAND + CELL * 2) / (CELL * l)) + 1) * CELL * l;
  return { near: [c[0] - dir[0] * reach, c[1] - dir[1] * reach], far: [c[0] + dir[0] * reach, c[1] + dir[1] * reach], dir };
}

/** Where each road of `roads` (not a motorway's, a ramp or a trail) crosses middle line `middle`, in the roads' order. */
export function crossings(middle: readonly Pt[], roads: readonly RoadPlan[]): Crossing[] {
  // (The middle line's segments filed by cell: by their end's index, `segmentGrid`.)
  const grid = segmentGrid(middle);
  const { ls: lm } = chunksOf(middle);
  const out: Crossing[] = [];
  roads.forEach((r, ri) => {
    if (isSeparated(r.kind) || r.kind === 'trail') return;
    const lr = lengths(r.points);
    for (let j = 0; j + 1 < r.points.length; j++) {
      const [p, q] = [r.points[j], r.points[j + 1]];
      const seen = new Set<number>();
      for (let cz = Math.floor(Math.min(p[1], q[1]) / SEG_CELL); cz <= Math.floor(Math.max(p[1], q[1]) / SEG_CELL); cz++) {
        for (let cx = Math.floor(Math.min(p[0], q[0]) / SEG_CELL); cx <= Math.floor(Math.max(p[0], q[0]) / SEG_CELL); cx++) {
          for (const e of grid.get(gridKey(cx, cz)) ?? []) {
            const k = e - 1;
            if (seen.has(k)) continue;
            seen.add(k);
            const [a, b] = [middle[k], middle[k + 1]];
            const hit = segmentCross(p, q, a, b);
            if (!hit || hit.t < 0 || hit.t >= 1 || hit.u < 0 || hit.u >= 1) continue;
            out.push({ road: ri, s: lr[j] + hit.t * (lr[j + 1] - lr[j]), m: lm[k] + hit.u * (lm[k + 1] - lm[k]), angle: segmentAngle(p, q, a, b) });
          }
        }
      }
    }
  });
  return out.sort((x, y) => x.road - y.road || x.s - y.s);
}

/** The motorway as laid (`finishMotorway`): its roads (two carriageways, then its ramps), and what each road it crosses gets in its `crosses`. */
export interface MotorwayLaid {
  roads: RoadPlan[];
  /** Per road it passes over (by index), the carriageways it passes under (their indices once laid: `base` on) and where along that road. */
  under: Map<number, { road: number; s: number }[]>;
  /** Its junction's road under it (index), and the crossings. */
  junction: number;
  crossings: Crossing[];
}

/**
 * The motorway laid on `route` over the roads laid since (`roads`, to which its own go at index `base` on): its
 * carriageways raised where each road crosses under them and bridged over its water, and its half-diamond on the
 * crossing nearest the main town (`town`) that has room for one (`halfDiamond`), at its junction's gate (`gates`' first:
 * where a road from the town could climb to it; not a village's). Named by `name` (each base made unique by the
 * caller). Null if no crossing there has room for its junction.
 */
export function finishMotorway(route: MotorwayRoute, roads: readonly RoadPlan[], wet: (x: number, z: number) => boolean, town: Pt, base: number, name: (wanted: string) => string): MotorwayLaid | null {
  const found = crossings(route.middle, roads);
  // (Each crossing's distance from the town worked out once.)
  const { ls: lm } = chunksOf(route.middle);
  const far = new Map(found.map((c) => [c, dist(pointAt(route.middle, c.m, lm).p, town)]));
  const byTown = [...found].sort((a, b) => far.get(a)! - far.get(b)! || a.road - b.road);
  if (!clearsUnder(route, found, roads, wet)) return null;
  const index = new RoadIndex(roads);
  for (const c of byTown) {
    if (roads[c.road].kind === 'trail' || c.angle < CROSS_ANGLE || Math.abs(c.m - route.gates[0].s) > GATE_APART / 2) continue;
    const ramps = halfDiamond(route, c, roads, index, found, wet);
    if (!ramps) continue;
    const ways = route.ways.map((w) => decked(w, found, roads, wet));
    const dir = compass(route.middle);
    const laid: RoadPlan[] = [
      { ...ways[0].road, name: name(`${route.name} ${dir[0]}`) },
      { ...ways[1].road, name: name(`${route.name} ${dir[1]}`) },
      { ...ramps.off, name: name(`${route.name} Junction ${route.junction} Exit`) },
      { ...ramps.on, name: name(`${route.name} Junction ${route.junction} Entry`) },
    ];
    // (The ramps are off one carriageway and onto the other: which, by `side`.)
    const under = new Map<number, { road: number; s: number }[]>();
    ways.forEach((w, k) => {
      for (const x of w.crosses) {
        const list = under.get(x.road) ?? [];
        list.push({ road: base + k, s: x.under });
        under.set(x.road, list);
      }
    });
    for (const list of under.values()) list.sort((a, b) => a.s - b.s || a.road - b.road);
    return { roads: laid, under, junction: c.road, crossings: found };
  }
  return null;
}

/**
 * Whether its profile (`profileOf`, at its kind's own grade and `CUT_MAX_MOTORWAY`: the limits the built land is held
 * to) still fits with its deck `CROSS_CLEAR` m over each road crossing under it (`found`) as low as that road could lie
 * there: within its grade and its cut (`cutMostOf`, a mountain country road's at most) of the land all along it. (A
 * road climbing a steep hill just past it lies well over the land there, and the deck over it higher still: an
 * embankment taller than a motorway's either side.) Not judged without the land (`route.height`).
 */
function clearsUnder(route: MotorwayRoute, found: readonly Crossing[], roads: readonly RoadPlan[], wet: (x: number, z: number) => boolean): boolean {
  const { heights, height } = route;
  if (!height) return true;
  const lift = new Map<number, number>();
  for (const c of found) {
    const r = roads[c.road];
    const kind = r.kind ?? 'street';
    const [grade, cut] = [ROAD_KIND[kind].grade, cutMostOf(kind, true)];
    const ls = lengths(r.points);
    let low = -Infinity;
    r.points.forEach(([x, z], j) => {
      if (!wet(x, z)) low = Math.max(low, height(x, z) - cut - grade * Math.abs(ls[j] - c.s));
    });
    const k0 = Math.round(c.m / STEP);
    for (let k = k0 - 3; k <= k0 + 3; k++) if (k >= 0 && k < heights.length && !Number.isNaN(heights[k])) lift.set(k, Math.max(lift.get(k) ?? -CUT_MAX_MOTORWAY, low + CROSS_CLEAR - heights[k]));
  }
  return !!profileOf(heights, lift, ROAD_KIND.motorway.grade, CUT_MAX_MOTORWAY);
}

/** Which way each carriageway's cars go, by the compass (the first's along the middle line, the second's against it). */
function compass(middle: readonly Pt[]): [string, string] {
  const [a, b] = [middle[0], middle[middle.length - 1]];
  const [dx, dz] = [b[0] - a[0], b[1] - a[1]];
  // (North is -z.)
  return Math.abs(dx) >= Math.abs(dz) ? (dx > 0 ? ['Eastbound', 'Westbound'] : ['Westbound', 'Eastbound']) : dz > 0 ? ['Southbound', 'Northbound'] : ['Northbound', 'Southbound'];
}

/**
 * Carriageway `way` with its decks: `raised` over each road crossing it (`found`, on the middle line; on it, where
 * that road crosses it), closer than `MERGE_SPANS` m made one; `bridge` over its water (`DECK_PAST` m past each
 * bank); `crosses` listing the roads under it. And where along each road it crosses it is (`under`).
 */
function decked(way: RoadPlan, found: readonly Crossing[], roads: readonly RoadPlan[], wet: (x: number, z: number) => boolean): { road: RoadPlan; crosses: { road: number; under: number }[] } {
  const ls = lengths(way.points);
  const L = ls[ls.length - 1];
  const crosses: { road: number; s: number; under: number }[] = [];
  const spans: [number, number][] = [];
  for (const c of found) {
    const r = roads[c.road];
    const hit = crossOne(way.points, r.points);
    if (!hit) continue;
    const half = Math.max(OVER_HALF, (r.width / 2 + 8) / Math.max(0.3, pmath.sin(hit.angle)) + 4);
    crosses.push({ road: c.road, s: round2(hit.s), under: round2(hit.u) });
    spans.push([Math.max(0, hit.s - half), Math.min(L, hit.s + half)]);
  }
  const out: RoadPlan = { ...way };
  const raised = mergeSpans(spans, MERGE_SPANS).map(([a, b]): [number, number] => [round2(a), round2(b)]);
  if (raised.length) out.raised = raised;
  const runs = wetRuns(way.points, wet, ls);
  if (runs.length) {
    // (A deck out to its end reaches it: rounded up, not down short of it, the last metre over the water off it.)
    const decks = runs.map(([s0, s1]): [number, number] => [round2(Math.max(0, s0 - DECK_PAST)), s1 + DECK_PAST >= L ? Math.ceil(L * 100) / 100 : round2(s1 + DECK_PAST)]);
    out.bridge = decks[0];
    if (decks.length > 1) out.bridges = decks.slice(1);
  }
  if (crosses.length) out.crosses = crosses.sort((a, b) => a.s - b.s).map(({ road, s }) => ({ road, s }));
  return { road: out, crosses: crosses.map(({ road, under }) => ({ road, under })) };
}

/**
 * Where polyline `a` crosses polyline `b` first (the first of `a`'s segments that does, the first of `b`'s on it):
 * along each (m), and how square (radians from along). `a`'s segments are looked up by cell (`segmentGrid`, made
 * once a line), only those whose bounds meet the segment of `b`'s tried.
 */
function crossOne(a: readonly Pt[], b: readonly Pt[]): { s: number; u: number; angle: number } | null {
  let grid = grids.get(a);
  if (!grid) grids.set(a, (grid = segmentGrid(a)));
  let best: { i: number; j: number; t: number; u: number } | null = null;
  for (let j = 0; j + 1 < b.length; j++) {
    const [c, d] = [b[j], b[j + 1]];
    const tried = new Set<number>();
    for (let cz = Math.floor(Math.min(c[1], d[1]) / SEG_CELL); cz <= Math.floor(Math.max(c[1], d[1]) / SEG_CELL); cz++) {
      for (let cx = Math.floor(Math.min(c[0], d[0]) / SEG_CELL); cx <= Math.floor(Math.max(c[0], d[0]) / SEG_CELL); cx++) {
        for (const e of grid.get(gridKey(cx, cz)) ?? []) {
          const i = e - 1;
          if (tried.has(i) || (best && i >= best.i)) continue;
          tried.add(i);
          const [p, q] = [a[i], a[i + 1]];
          if (Math.max(c[0], d[0]) < Math.min(p[0], q[0]) || Math.min(c[0], d[0]) > Math.max(p[0], q[0]) || Math.max(c[1], d[1]) < Math.min(p[1], q[1]) || Math.min(c[1], d[1]) > Math.max(p[1], q[1])) continue;
          const hit = segmentCross(p, q, c, d);
          if (!hit || hit.t < 0 || hit.t > 1 || hit.u < 0 || hit.u > 1) continue;
          best = { i, j, t: hit.t, u: hit.u };
        }
      }
    }
  }
  if (!best) return null;
  const { i, j, t, u } = best;
  const [la, lb] = [lengths(a), lengths(b)];
  return { s: la[i] + t * (la[i + 1] - la[i]), u: lb[j] + u * (lb[j + 1] - lb[j]), angle: segmentAngle(a[i], a[i + 1], b[j], b[j + 1]) };
}
const grids = new WeakMap<readonly Pt[], ReturnType<typeof segmentGrid>>();

/**
 * The half-diamond at crossing `c` (a road under the motorway): an off-ramp from one carriageway and an on-ramp onto
 * the other, both on one side of that road (the side tried first the one with more of the motorway beyond it), each
 * to its carriageway's right; each merging into its carriageway `RAMP_ALONG` m or so from the crossing, and meeting
 * the road under it in a T `RAMP_T` m out from its carriageway, at `MEET` or squarer. Null if neither side has room:
 * a ramp over the water, near another road, by another crossing or a junction on the road under it, or past the
 * world's edge.
 */
export function halfDiamond(route: MotorwayRoute, c: Crossing, roads: readonly RoadPlan[], index: RoadIndex, found: readonly Crossing[], wet: (x: number, z: number) => boolean): { off: RoadPlan; on: RoadPlan; side: -1 | 1 } | null {
  const { middle } = route;
  const { ls: lm } = chunksOf(middle);
  const L = lm[lm.length - 1];
  const under = roads[c.road];
  const lu = lengths(under.points);
  const sides: (-1 | 1)[] = c.m > L / 2 ? [-1, 1] : [1, -1];
  for (const side of sides) {
    // (Side -1: the ramps where the middle line comes from. Off the first carriageway (its cars coming that way) and
    // onto the second; side +1: off the second and onto the first.)
    const offWay = side < 0 ? 0 : 1;
    const ramps: RoadPlan[] = [];
    for (const k of [offWay, 1 - offWay]) {
      // (The first carriageway off the middle line to `rightOf`'s other side: `carriageways`.)
      const lane = k === 0 ? -1 : 1;
      const ramp = rampOf(middle, lm, c, side, lane, under, lu);
      if (!ramp) break;
      const pts = k === offWay ? ramp : [...ramp].reverse();
      ramps.push({ points: pts.map(([x, z]): Pt => [round2(x), round2(z)]), width: RAMP_WIDTH, ends: k === offWay ? ['merge', 'join'] : ['join', 'merge'], kind: 'ramp', oneway: 1 });
    }
    if (ramps.length < 2) continue;
    // (Room for both: dry, clear of every road but its carriageway's and the one under it, of the motorway's other
    // crossings, and of junctions on the road under it.)
    const along = [c.m + side * (RAMP_ALONG + 15), c.m - side * 10];
    if (Math.min(...along) < 10 || Math.max(...along) > L - 10) continue;
    if (found.some((o) => o !== c && o.m >= Math.min(...along) - 20 && o.m <= Math.max(...along) + 20)) continue;
    // (Its own ends aside: a road going on from one, end to end, meets no junction there.)
    const ends = [under.points[0], under.points[under.points.length - 1]];
    const fits = ramps.every((r) => {
      const ls = lengths(r.points);
      const T = r.ends[0] === 'join' ? r.points[0] : r.points[r.points.length - 1];
      return r.points.every((p, i) => {
        if (wet(p[0], p[1])) return false;
        const fromT = r.ends[0] === 'join' ? ls[i] : ls[ls.length - 1] - ls[i];
        if (fromT < 12) return true;
        const near = index.nearest(p[0], p[1], RAMP_WIDTH / 2 + 7 + 4);
        return !near || (near.road === c.road && fromT < 30);
      }) && !roads.some((o, i) => i !== c.road && o.kind !== 'trail' && [o.points[0], o.points[o.points.length - 1]].some((e) => dist(e, T) < TEE_APART && !ends.some((u) => dist(u, e) < 1)));
    });
    if (!fits) continue;
    return { off: ramps[0], on: ramps[1], side };
  }
  return null;
}

/**
 * A ramp's points from its merge to its T (`halfDiamond`): on the carriageway `lane` of the middle line (+1: `rightOf`
 * it, the second carriageway; -1: the first), out beyond it (its cars' right as the game shows it), `side` along from
 * crossing `c`; null if the road
 * under it doesn't run on `RAMP_T` m past the carriageway there, or meets the ramp too slantingly.
 */
function rampOf(middle: readonly Pt[], lm: readonly number[], c: Crossing, side: -1 | 1, lane: 1 | -1, under: RoadPlan, lu: readonly number[]): Pt[] | null {
  /** A point `a` m along from the crossing (on `side`), `b` m out to the carriageway's right from its middle. */
  const at = (a: number, b: number): Pt => {
    const { p, t } = pointAt(middle, c.m + side * a, lm);
    const r = rightOf(t);
    const out = lane * (APART / 2 + b);
    return [p[0] + r[0] * out, p[1] + r[1] * out];
  };
  /** How far across the middle line (m, + its right) and along from the crossing (on `side`) the road under it is at `s` along it. */
  const off = (s: number) => {
    const { p } = pointAt(under.points, s, lu);
    const m = nearestOn(middle, lm, p[0], p[1]);
    const { p: q, t } = pointAt(middle, m.s, lm);
    const r = rightOf(t);
    return { across: (p[0] - q[0]) * r[0] + (p[1] - q[1]) * r[1], along: (m.s - c.m) * side };
  };
  /** Where the road under it is `want` m across the middle line (on the side its sign says), nearest the crossing; null if it ends first. */
  const teeAt = (want: number): { s: number; along: number } | null => {
    for (const dir of [1, -1]) {
      for (let s = c.s; s >= 0 && s <= lu[lu.length - 1]; s += dir * 0.5) {
        const o = off(s);
        if (Math.abs(o.across) > Math.abs(want) + 1) break;
        if (Math.sign(o.across) === Math.sign(want) && Math.abs(o.across) >= Math.abs(want)) return s < 15 || s > lu[lu.length - 1] - 15 ? null : { s, along: o.along };
      }
    }
    return null;
  };
  const tu = (s: number) => pointAt(under.points, s, lu).t;
  // (Further out where the road under it slants away: a T nearer square to it.)
  for (const out of RAMP_TS) {
    const T = teeAt(lane * (APART / 2 + out));
    if (!T) continue;
    for (const runIn of RAMP_INS) {
      const t0 = T.along;
      const ctrl: Pt[] = [
        at(t0 + RAMP_ALONG, 0),
        at(t0 + RAMP_ALONG - 12, 0.6),
        at(t0 + RAMP_ALONG - RAMP_TAPER / 2, RAMP_BESIDE / 2),
        at(t0 + RAMP_ALONG - RAMP_TAPER, RAMP_BESIDE),
        at(t0 + runIn, RAMP_BESIDE),
        pointAt(under.points, T.s, lu).p,
      ];
      const pts = curve(ctrl, 1);
      const len = lengths(pts).at(-1)!;
      if (len < RAMP_LENGTH[0] || len > RAMP_LENGTH[1]) continue;
      // (At its T, square enough to the road under it.)
      const t = tu(T.s);
      const e = pts.length - 1;
      const [a, b] = [pts[Math.max(0, e - 8)], pts[e]];
      const l = dist(a, b) || 1;
      if (Math.abs(((b[0] - a[0]) * t[1] - (b[1] - a[1]) * t[0]) / l) >= pmath.sin(MEET + 0.05)) return pts;
    }
  }
  return null;
}

/** What is wrong with a world's motorway as laid (its carriageways and ramps among `roads`), for the tests and the sample tool: none if all is well. */
export function motorwayProblems(roads: readonly RoadPlan[]): string[] {
  const out: string[] = [];
  const ways = roads.flatMap((r, i) => (r.kind === 'motorway' ? [i] : []));
  const ramps = roads.flatMap((r, i) => (r.kind === 'ramp' ? [i] : []));
  if (!ways.length) return out;
  if (ways.length !== 2) out.push(`${ways.length} carriageways`);
  if (ramps.length !== 2) out.push(`${ramps.length} ramps`);
  // Traffic on the right as the game shows it (sim-core `cars::right_of`: going (x, z), its right (z, -x), `rightOf`'s
  // other side): each carriageway's other on the left of the way its cars go; each ramp to its right.
  for (const i of ways) {
    const r = roads[i];
    const ls = lengths(r.points);
    const { p, t } = pointAt(r.points, ls[ls.length - 1] / 2, ls);
    const go: Pt = [t[0] * (r.oneway ?? 1), t[1] * (r.oneway ?? 1)];
    const other = ways.find((j) => j !== i);
    if (other === undefined) continue;
    const q = nearest(roads[other].points, p[0], p[1]);
    const qp = pointAt(roads[other].points, q.s).p;
    const left = (qp[1] - p[1]) * go[0] - (qp[0] - p[0]) * go[1];
    if (left <= 0) out.push(`${r.name}: the other carriageway on its right`);
  }
  for (const i of ramps) {
    const r = roads[i];
    // (Never across a carriageway: under the other's deck it would be on the motorway's level in the simulation.)
    for (const j of ways) if (crossOne(r.points.slice(3, -3), roads[j].points)) out.push(`${r.name} crosses ${roads[j].name}`);
    const merge = mergeOf(roads, r);
    if (!merge) {
      out.push(`${r.name}: no merge`);
      continue;
    }
    const { end: e, way } = merge;
    if (!way || way.dist > roads[way.road].width / 2) {
      out.push(`${r.name}: its merge off its carriageway`);
      continue;
    }
    const w = roads[way.road];
    const { t } = pointAt(w.points, way.s);
    const go: Pt = [t[0] * (w.oneway ?? 1), t[1] * (w.oneway ?? 1)];
    const mid = pointAt(r.points, lengths(r.points).at(-1)! / 2).p;
    const q = pointAt(w.points, nearest(w.points, mid[0], mid[1]).s).p;
    if ((mid[0] - q[0]) * go[1] - (mid[1] - q[1]) * go[0] <= 0) out.push(`${r.name}: on its carriageway's left`);
    // (Off a carriageway it leaves it, onto one it joins it, the way its cars go.)
    const rt = pointAt(r.points, e === 0 ? 1 : lengths(r.points).at(-1)! - 1).t;
    if (rt[0] * go[0] + rt[1] * go[1] < 0.5) out.push(`${r.name}: against its carriageway's way`);
  }
  return out;
}

/**
 * Where ramp `ramp` merges with the motorway (`roads` its carriageways among them): which of its ends (its first
 * `merge` end), there, and the carriageway nearest there (its index, how far its middle is and where along it, m;
 * none without a carriageway). Null: no end of it merges.
 */
export function mergeOf(roads: readonly RoadPlan[], ramp: RoadPlan): { end: 0 | 1; at: Pt; way?: { road: number; dist: number; s: number } } | null {
  const e = ramp.ends.indexOf('merge');
  if (e < 0) return null;
  const at = e === 0 ? ramp.points[0] : ramp.points[ramp.points.length - 1];
  let way: { road: number; dist: number; s: number } | undefined;
  roads.forEach((r, j) => {
    if (r.kind !== 'motorway') return;
    const n = nearest(r.points, at[0], at[1]);
    if (!way || n.dist < way.dist) way = { road: j, dist: n.dist, s: n.s };
  });
  return { end: e as 0 | 1, at, way };
}
