/**
 * The motorway: one in every world where it fits, edge to edge past the main town, on overpasses over the roads it
 * crosses and on a bridge over its water, with one half-diamond junction to the town. The generator lays it in two
 * goes (`macro.ts` `generateFromSpec`):
 *
 * - **`routeMotorway`**, after the villages and before the country roads: the best of `CANDIDATES` cubic curves from
 *   one edge of the world to the opposite one (either way across), passing the main town `TOWN_CLEAR` m off its box at
 *   least and `TOWN_REACH` m at most, `HAMLET_CLEAR` m off the villages', clear of the roads laid already, bending no
 *   tighter than `BEND_RADIUS`, over the water at most once, `WATER_ANGLE` or squarer to it and `WATER_MOST` m at most,
 *   and with a profile within its grade (`MOTORWAY_GRADE`) cut or filled no more than `CUT` (`profileOf`), standing
 *   `OVER` m over the land at each crossing. Scored by its cut and fill, its water and how far it passes from the town.
 *   Its middle line is offset into two one-way `motorway` carriageways `APART` m apart, laid for traffic on the right as
 *   the game shows it (`carriageways`: each has the other on the left of the way its cars go, sim-core `cars::right_of`). The country roads (`CountryRouter`) may
 *   not cross it (`blocked`, `BAND` m either side of its middle line) but at 1 to 3 gates square to it (`gates`), the
 *   first the nearest the main town (`junction`, kept clear wider and longer for its ramps).
 * - **`finishMotorway`**, once the country roads are laid: where they pass under it (`crossings`) its carriageways are
 *   `raised` (spans closer than `MERGE_SPANS` m made one), over its water a `bridge`, each crossing listed in
 *   `crosses` on both roads; and its junction (`halfDiamond`): an off-ramp from one carriageway and an on-ramp onto the
 *   other, both on one side of the road under it, each about 125 m, merging into its carriageway at one end and meeting
 *   the road under it in a T `RAMP_T` m out from the carriageway at the other.
 *
 * A world it doesn't fit (too small, an island, mountains) has none: `failed` says why, and the world is laid as it
 * was. All its maths is `pmath`'s (the same towns on every engine).
 */

import type { RoadPlan } from '../core/protocol';
import { ROAD_KIND } from '../core/roadKinds';
import { CUT_MAX_MOTORWAY } from '../render/shared/builtSurface';
import { CELL, type Cells } from './countryRouter';
import { curve, lengths, MEET, nearest, pointAt, RoadIndex, sharpestTurn, type Pt } from './roads';
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
/** How far its middle line keeps off the roads laid before it (bridges, causeways: m). */
const ROAD_CLEAR = 40;
/** Its water: crossed this square or squarer (radians), each crossing this long at most (m), crossings at most. */
export const WATER_ANGLE = Math.PI / 3;
const WATER_MOST = 100;
const WATER_CROSSINGS = 1;
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
/** The draws its course and names come from (apart from the world's). */
const MOTOR = 0x6d6f746f;

const dist = (a: Pt, b: Pt) => pmath.hypot(a[0] - b[0], a[1] - b[1]);
const round2 = (v: number) => Math.round(v * 100) / 100;
const offRect = (r: Rect, [x, z]: Pt) => pmath.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d));
/** The right of a direction (x, z), as seen from above with z down the page and north up (`leftOf` turned over). */
const rightOf = ([x, z]: Pt): Pt => [-z, x];

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
}

/** A road crossing the motorway's middle line: which, how far along it (m) and along the middle line (m), and how square (radians from along). */
export interface Crossing {
  road: number;
  s: number;
  m: number;
  angle: number;
}

/** The land's height at (x, z) (m), between its cells' middles. */
function heightAt(land: Cells, x: number, z: number): number {
  const fx = Math.max(0, Math.min(land.gw - 1, x / CELL - 0.5));
  const fz = Math.max(0, Math.min(land.gd - 1, z / CELL - 0.5));
  const [x0, z0] = [Math.floor(fx), Math.floor(fz)];
  const [x1, z1] = [Math.min(land.gw - 1, x0 + 1), Math.min(land.gd - 1, z0 + 1)];
  const [u, v] = [fx - x0, fz - z0];
  const h = (cx: number, cz: number) => land.height[cz * land.gw + cx];
  return (h(x0, z0) * (1 - u) + h(x1, z0) * u) * (1 - v) + (h(x0, z1) * (1 - u) + h(x1, z1) * u) * v;
}

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

/** A polyline with a point every `step` m (its last point kept). */
function resample(pts: readonly Pt[], step: number): Pt[] {
  const ls = lengths(pts);
  const L = ls[ls.length - 1];
  const n = Math.max(1, Math.round(L / step));
  const out: Pt[] = [];
  let i = 0;
  for (let k = 0; k <= n; k++) {
    const s = (L * k) / n;
    while (i < ls.length - 2 && ls[i + 1] < s) i++;
    const f = Math.max(0, Math.min(1, (s - ls[i]) / (ls[i + 1] - ls[i] || 1)));
    out.push([pts[i][0] + (pts[i + 1][0] - pts[i][0]) * f, pts[i][1] + (pts[i + 1][1] - pts[i][1]) * f]);
  }
  return out;
}

/** The direction along a polyline at point `i` (over its points either side). */
function dirAt(pts: readonly Pt[], i: number): Pt {
  const [a, b] = [pts[Math.max(0, i - 2)], pts[Math.min(pts.length - 1, i + 2)]];
  const l = dist(a, b) || 1;
  return [(b[0] - a[0]) / l, (b[1] - a[1]) / l];
}

/** `pts` moved `by` m to its right (the way its points run), each point square off it. */
function offset(pts: readonly Pt[], by: number): Pt[] {
  return pts.map((p, i) => {
    const r = rightOf(dirAt(pts, i));
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
  if (first > 0) out.unshift(edge(pts[first], pts[first - 1]));
  if (last < pts.length - 1) out.push(edge(pts[last], pts[last + 1]));
  return out;
}

/** Which edge of the world (north, east, south, west: `FACING`'s order) `p` lies on, within a metre; -1 none. */
function edgeOf([x, z]: Pt, W: number, D: number): number {
  return [z <= 1, x >= W - 1, z >= D - 1, x <= 1].indexOf(true);
}

/**
 * A profile along a line sampled every `STEP` m: the land's heights `h` (NaN over the water: its bridge, held low over
 * it, as the land's surface holds a bridge's deck, no more than `WATER_DECK` over the lower of its banks), held within
 * `MOTORWAY_GRADE` and no more than `CUT` off the land, and over it by `lift[k]` at least where given. The heights, and
 * how much it cuts and fills (m², summed over the samples); null if no such profile exists.
 */
export function profileOf(h: readonly number[], lift: ReadonlyMap<number, number> = new Map()): { y: number[]; cut: number } | null {
  const n = h.length;
  const g = MOTORWAY_GRADE * STEP;
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
    Number.isNaN(h[k]) ? (Number.isFinite(bank[k]) ? [bank[k] - 1, bank[k] + WATER_DECK] : [-Infinity, Infinity]) : lift.has(k) ? [h[k] + lift.get(k)!, Infinity] : [h[k] - CUT, h[k] + CUT];
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
type Why = 'edge' | 'side' | 'bend' | 'town' | 'far' | 'village' | 'road' | 'water' | 'grade' | 'gate';
const WHY: Record<Why, string> = {
  edge: 'runs out of the world and back',
  side: `runs within ${SIDE_CLEAR} m of the world's edge`,
  gate: 'has no place a road from the town could climb to and pass under it',
  bend: `bends tighter than ${BEND_RADIUS} m`,
  town: `comes within ${TOWN_CLEAR} m of the town`,
  far: `passes further than ${TOWN_REACH} m from the town`,
  village: `comes within ${HAMLET_CLEAR} m of a village`,
  road: 'runs into a road',
  water: 'crosses the water too long, too slanting or too often',
  grade: `climbs too steeply for its ${Math.round(ROAD_KIND.motorway.grade * 100)} % within ${CUT_MAX_MOTORWAY} m of cut and fill`,
};

/** The heights its profile reads along `middle`, every `STEP` m (NaN over the water). */
function heightsAlong(middle: readonly Pt[], site: MotorwaySite): number[] {
  const ls = lengths(middle);
  const L = ls[ls.length - 1];
  const n = Math.max(1, Math.round(L / STEP));
  const out: number[] = [];
  for (let k = 0; k <= n; k++) {
    const { p } = pointAt(middle, (L * k) / n, ls);
    out.push(site.wet(p[0], p[1]) ? NaN : (site.height ?? ((x, z) => heightAt(site.land, x, z)))(p[0], p[1]));
  }
  return out;
}

/** Course `middle` judged (see the top of this file): its score, or why it doesn't do. */
function judge(middle: Pt[], line: Pt[], site: MotorwaySite): Course | Why {
  const { land, areas, roads, wet } = site;
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
  const others = roads.filter((r) => r.kind !== 'trail');
  if (others.length) {
    const index = new RoadIndex(others);
    for (let i = 0; i < middle.length; i += 4) if (index.nearest(middle[i][0], middle[i][1], ROAD_CLEAR)) return 'road';
  }
  // Its water: once at most, square enough, not too long, not at an end.
  const ls = lengths(middle);
  const runs = wetRuns(middle, wet, ls);
  if (runs.length > WATER_CROSSINGS) return 'water';
  let water = 0;
  for (const [s0, s1, i0, i1] of runs) {
    const along = s1 - s0 + 1;
    if (along > WATER_MOST || i0 === 0 || i1 === middle.length - 1) return 'water';
    // (How far the water runs square across it there: a river crossed at `θ` from along it runs `along · tan θ` across.)
    const { p, t } = pointAt(middle, (s0 + s1) / 2, ls);
    const n = rightOf(t);
    let across = 0;
    for (const k of [1, -1]) for (let d = 1; d < 300 && wet(p[0] + n[0] * d * k, p[1] + n[1] * d * k); d++) across++;
    if (across < along * pmath.sin(WATER_ANGLE) / pmath.cos(WATER_ANGLE)) return 'water';
    water += along;
  }
  const heights = heightsAlong(middle, site);
  const made = profileOf(heights);
  if (!made) return 'grade';
  return { middle, line, heights, score: (made.cut / ls[ls.length - 1]) * 20 + water * 0.4 + Math.abs(reach - REACH_AIM) * 0.05 };
}

/** The courses it tries: `CANDIDATES` cubic curves from one edge to the opposite one, passing either side of the town, near and further off. */
function candidates(site: MotorwaySite): Pt[][] {
  const { land, areas, seed } = site;
  const [W, D] = [land.W, land.D];
  const rand = mulberry32((seed ^ MOTOR) >>> 0);
  const main = areas[0].rect;
  const out: Pt[][] = [];
  const OUT = 60;
  for (const across of [0, 1] as const) {
    // (Along x, west to east: passing north or south of the town; along z, north to south: west or east of it.)
    const [L, S] = across === 0 ? [W, D] : [D, W];
    const [lo, hi] = across === 0 ? [main.z, main.z + main.d] : [main.x, main.x + main.w];
    for (const side of [-1, 1]) {
      // (Where a line straight along that side of the town would pass clear of the villages: those offsets first.)
      const clear = (off: number) => {
        const at = side < 0 ? lo - off : hi + off;
        return areas.every((v, k) => k === 0 || (across === 0 ? v.rect.z - at > HAMLET_CLEAR + OUTER + 10 || at - v.rect.z - v.rect.d > HAMLET_CLEAR + OUTER + 10 : v.rect.x - at > HAMLET_CLEAR + OUTER + 10 || at - v.rect.x - v.rect.w > HAMLET_CLEAR + OUTER + 10));
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
          const ctrl: Pt[] = [
            [-OUT, e0],
            [L * 0.36, pass + (e0 - pass) * k],
            [L * 0.64, pass + (e1 - pass) * k],
            [L + OUT, e1],
          ];
          out.push(bezier(across === 0 ? ctrl : ctrl.map(([x, z]): Pt => [z, x])));
        }
      }
    }
  }
  return out.slice(0, CANDIDATES);
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
  const fit: Course[] = [];
  for (const line of candidates(site)) {
    const middle = clipTo(line, W, D, 0);
    const judged = middle ? judge(middle, line, site) : 'edge';
    if (typeof judged === 'string') why.set(judged, (why.get(judged) ?? 0) + 1);
    else fit.push(judged);
  }
  // (The best first, with a gate a road from the town could climb to: `gatesOf`.)
  fit.sort((a, b) => a.score - b.score);
  for (const best of fit) {
    const gates = gatesOf(best, site);
    if (!gates.length) {
      why.set('gate', (why.get('gate') ?? 0) + 1);
      continue;
    }
    const ways = carriageways(best.line, W, D);
    if (!ways) continue;
    return { middle: best.middle, ways, blocked: blockedOf(best.middle, gates, land, site.areas), gates, ...namesOf(site.seed) };
  }
  const most = [...why.entries()].sort((a, b) => b[1] - a[1])[0];
  return { failed: `No motorway: every course tried ${most ? WHY[most[0]] : 'fails'}.` };
}

/** The gates on course `c` (see `MotorwayRoute.gates`): the junction's nearest the main town first; none if it has no room for one. */
function gatesOf(c: Course, site: MotorwaySite): Gate[] {
  const { middle, heights } = c;
  const ls = lengths(middle);
  const L = ls[ls.length - 1];
  const runs = wetRuns(middle, site.wet, ls);
  const town = site.areas[0].centre;
  /** The octant step nearest square across at `s`, and how far off square it is. */
  const squareAt = (s: number): { across: Pt; off: number } => {
    const { t } = pointAt(middle, s, ls);
    const n = rightOf(t);
    const k = Math.round(pmath.atan2(n[1], n[0]) / (Math.PI / 4));
    const a = (k * Math.PI) / 4;
    const across: Pt = [Math.round(pmath.cos(a)), Math.round(pmath.sin(a))];
    const off = pmath.acos(Math.min(1, Math.abs(n[0] * across[0] + n[1] * across[1]) / pmath.hypot(across[0], across[1])));
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
  const ok = (s: number, apart: number) =>
    s >= GATE_END &&
    s <= L - GATE_END &&
    runs.every(([s0, s1]) => s < s0 - GATE_WATER || s > s1 + GATE_WATER) &&
    chosen.every((g) => Math.abs(g.s - s) >= apart) &&
    squareAt(s).off <= GATE_SQUARE &&
    // (Its way across in the world, out past where the country roads keep off.)
    [-1, 1].every((k) => {
      const { p, t } = pointAt(middle, s, ls);
      const n = rightOf(t);
      const q: Pt = [p[0] + n[0] * k * (JUNCTION_BAND + CELL * 4), p[1] + n[1] * k * (JUNCTION_BAND + CELL * 4)];
      return q[0] > 8 && q[1] > 8 && q[0] < site.land.W - 8 && q[1] < site.land.D - 8;
    }) &&
    // (Dry across it, out past where the country roads keep off.)
    [-1, 1].every((k) => {
      const { p, t } = pointAt(middle, s, ls);
      const n = rightOf(t);
      for (let d = 0; d <= JUNCTION_BAND + CELL; d += 4) if (site.wet(p[0] + n[0] * d * k, p[1] + n[1] * d * k)) return false;
      return true;
    });
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
    const h = site.height ?? ((x: number, z: number) => heightAt(site.land, x, z));
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
  const ls = lengths(middle);
  const junction = gates[0];
  const mark = (p: Pt, r: number, v: number) => {
    for (let cz = Math.max(0, Math.floor((p[1] - r) / CELL)); cz <= Math.min(gd - 1, Math.floor((p[1] + r) / CELL)); cz++) {
      for (let cx = Math.max(0, Math.floor((p[0] - r) / CELL)); cx <= Math.min(gw - 1, Math.floor((p[0] + r) / CELL)); cx++) {
        const c: Pt = [(cx + 0.5) * CELL, (cz + 0.5) * CELL];
        if (pmath.hypot(c[0] - p[0], c[1] - p[1]) <= r && areas.every((a) => offRect(a.rect, c) >= AREA_OPEN)) out[cz * gw + cx] = v;
      }
    }
  };
  for (let i = 0; i < middle.length; i += 2) mark(middle[i], Math.abs(ls[i] - junction.s) <= JUNCTION_REACH ? JUNCTION_BAND : BAND, 1);
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
  const G = 16;
  const grid = new Map<number, number[]>();
  for (let k = 0; k + 1 < middle.length; k++) {
    const [a, b] = [middle[k], middle[k + 1]];
    for (let cz = Math.floor(Math.min(a[1], b[1]) / G); cz <= Math.floor(Math.max(a[1], b[1]) / G); cz++) {
      for (let cx = Math.floor(Math.min(a[0], b[0]) / G); cx <= Math.floor(Math.max(a[0], b[0]) / G); cx++) {
        const key = (cx + 1000) * 4096 + cz + 1000;
        const list = grid.get(key);
        if (list) list.push(k);
        else grid.set(key, [k]);
      }
    }
  }
  const lm = lengths(middle);
  const out: Crossing[] = [];
  roads.forEach((r, ri) => {
    if (r.kind === 'motorway' || r.kind === 'ramp' || r.kind === 'trail') return;
    const lr = lengths(r.points);
    for (let j = 0; j + 1 < r.points.length; j++) {
      const [p, q] = [r.points[j], r.points[j + 1]];
      const seen = new Set<number>();
      for (let cz = Math.floor(Math.min(p[1], q[1]) / G); cz <= Math.floor(Math.max(p[1], q[1]) / G); cz++) {
        for (let cx = Math.floor(Math.min(p[0], q[0]) / G); cx <= Math.floor(Math.max(p[0], q[0]) / G); cx++) {
          for (const k of grid.get((cx + 1000) * 4096 + cz + 1000) ?? []) {
            if (seen.has(k)) continue;
            seen.add(k);
            const [a, b] = [middle[k], middle[k + 1]];
            const [ux, uz, vx, vz] = [q[0] - p[0], q[1] - p[1], b[0] - a[0], b[1] - a[1]];
            const den = ux * vz - uz * vx;
            if (Math.abs(den) < 1e-12) continue;
            const t = ((a[0] - p[0]) * vz - (a[1] - p[1]) * vx) / den;
            const u = ((a[0] - p[0]) * uz - (a[1] - p[1]) * ux) / den;
            if (t < 0 || t >= 1 || u < 0 || u >= 1) continue;
            const lu = pmath.hypot(ux, uz) || 1;
            const lv = pmath.hypot(vx, vz) || 1;
            const angle = pmath.acos(Math.min(1, Math.abs(ux * vx + uz * vz) / (lu * lv)));
            out.push({ road: ri, s: lr[j] + t * (lr[j + 1] - lr[j]), m: lm[k] + u * (lm[k + 1] - lm[k]), angle });
          }
        }
      }
    }
  });
  return out.sort((x, y) => x.road - y.road || x.s - y.s);
}

/** Spans (`[s0, s1]`, in order) with those closer than `gap` m made one. */
function merged(spans: readonly [number, number][], gap: number): [number, number][] {
  const sorted = [...spans].sort((a, b) => a[0] - b[0]);
  const out: [number, number][] = [];
  for (const [a, b] of sorted) {
    const last = out[out.length - 1];
    if (last && a - last[1] < gap) last[1] = Math.max(last[1], b);
    else out.push([a, b]);
  }
  return out;
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
 * crossing nearest the main town (`town`) that has room for one (`halfDiamond`). Named by `name` (each base made
 * unique by the caller). Null if no crossing has room for its junction.
 */
export function finishMotorway(route: MotorwayRoute, roads: readonly RoadPlan[], wet: (x: number, z: number) => boolean, town: Pt, base: number, name: (wanted: string) => string): MotorwayLaid | null {
  const found = crossings(route.middle, roads);
  const byTown = [...found].sort((a, b) => dist(pointAt(route.middle, a.m).p, town) - dist(pointAt(route.middle, b.m).p, town) || a.road - b.road);
  const index = new RoadIndex(roads);
  for (const c of byTown) {
    if (roads[c.road].kind === 'trail' || c.angle < CROSS_ANGLE) continue;
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
  const raised = merged(spans, MERGE_SPANS).map(([a, b]): [number, number] => [round2(a), round2(b)]);
  if (raised.length) out.raised = raised;
  const runs = wetRuns(way.points, wet, ls);
  if (runs.length) {
    const decks = runs.map(([s0, s1]): [number, number] => [round2(Math.max(0, s0 - DECK_PAST)), round2(Math.min(L, s1 + DECK_PAST))]);
    out.bridge = decks[0];
    if (decks.length > 1) out.bridges = decks.slice(1);
  }
  if (crosses.length) out.crosses = crosses.sort((a, b) => a.s - b.s).map(({ road, s }) => ({ road, s }));
  return { road: out, crosses: crosses.map(({ road, under }) => ({ road, under })) };
}

/** Where polyline `a` crosses polyline `b` first: along each (m), and how square (radians from along). */
function crossOne(a: readonly Pt[], b: readonly Pt[]): { s: number; u: number; angle: number } | null {
  const [la, lb] = [lengths(a), lengths(b)];
  for (let i = 0; i + 1 < a.length; i++) {
    const [p, q] = [a[i], a[i + 1]];
    const [minX, maxX, minZ, maxZ] = [Math.min(p[0], q[0]), Math.max(p[0], q[0]), Math.min(p[1], q[1]), Math.max(p[1], q[1])];
    for (let j = 0; j + 1 < b.length; j++) {
      const [c, d] = [b[j], b[j + 1]];
      if (Math.max(c[0], d[0]) < minX || Math.min(c[0], d[0]) > maxX || Math.max(c[1], d[1]) < minZ || Math.min(c[1], d[1]) > maxZ) continue;
      const [ux, uz, vx, vz] = [q[0] - p[0], q[1] - p[1], d[0] - c[0], d[1] - c[1]];
      const den = ux * vz - uz * vx;
      if (Math.abs(den) < 1e-12) continue;
      const t = ((c[0] - p[0]) * vz - (c[1] - p[1]) * vx) / den;
      const u = ((c[0] - p[0]) * uz - (c[1] - p[1]) * ux) / den;
      if (t < 0 || t > 1 || u < 0 || u > 1) continue;
      const angle = pmath.acos(Math.min(1, Math.abs(ux * vx + uz * vz) / ((pmath.hypot(ux, uz) || 1) * (pmath.hypot(vx, vz) || 1))));
      return { s: la[i] + t * (la[i + 1] - la[i]), u: lb[j] + u * (lb[j + 1] - lb[j]), angle };
    }
  }
  return null;
}

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
  const lm = lengths(middle);
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
    const m = nearest(middle, p[0], p[1], lm);
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
    const e = r.ends.indexOf('merge');
    if (e < 0) {
      out.push(`${r.name}: no merge`);
      continue;
    }
    const m = e === 0 ? r.points[0] : r.points[r.points.length - 1];
    const way = ways.map((j) => ({ j, n: nearest(roads[j].points, m[0], m[1]) })).sort((a, b) => a.n.dist - b.n.dist)[0];
    if (!way || way.n.dist > roads[way.j].width / 2) {
      out.push(`${r.name}: its merge off its carriageway`);
      continue;
    }
    const w = roads[way.j];
    const { t } = pointAt(w.points, way.n.s);
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
