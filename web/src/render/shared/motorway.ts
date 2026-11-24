/**
 * A motorway as drawn (engine-neutral). A motorway is two one-way `motorway` roads, one carriageway each way
 * (`roadKinds.ts`): `motorwayPairs` finds which two run side by side, and `barrierRuns` the line between them where
 * the concrete barrier stands (`barrier`, drawn by the street and, plainer, by the overview), the verge between their
 * edges as wide as they lie apart. `insideOf` says which side of a carriageway is its inside, towards its pair (its lanes
 * and shoulder across it: `markings.ts` `laneLayout`).
 *
 * `deckLevels`: the heights of a road raised over another (`RoadPlan.raised`) where the world has no built surface to
 * give them (`landSurface.ts` `levelsOf` null: a land not followed): its deck clear of what passes under, its
 * approaches climbing to it at its kind's grade, a ramp's from the carriageway it merges with down to its other end.
 * On a land the town follows the surface gives them (`builtSurface.ts`).
 */

import type { RoadPlan } from '../../core/protocol';
import { isSeparated, kindInfo, onewayOf } from '../../core/roadKinds';
import { leftOf, lengths, nearest, pointAt, raisedSpans, RoadIndex, type Pt } from '../../game/roads';
import { encodeFinish, type Geo, type V3 } from './geometry';
import type { RoadLevels } from './roads';

/**
 * A motorway's measures (m): its carriageway's lanes, each `lane` wide, begin `inner` in from its inside edge (where the
 * barrier stands, half of it on each carriageway), the shoulder outside them. The barrier: `base` wide at the road,
 * `top` wide at `high`, its foot `foot` high before its face leans in. Two carriageways are a pair where their middles
 * lie no more than `apart` further apart than their half widths. `clear`: how high a deck's road stands over what
 * passes under it on a world with no built surface (`deckLevels`: 5 m of headroom and the deck).
 */
export const MOTORWAY = { inner: 0.55, lane: 3.5, base: 0.6, top: 0.2, high: 0.85, foot: 0.08, kink: 0.28, apart: 24, clear: 6.4 } as const;
/** The barrier's concrete (a vertex colour in the plain finish: no material of its own). */
export const BARRIER_CONCRETE = '#A8A59D';
export const BARRIER_FINISH = encodeFinish(0.92, 0);
/**
 * An overpass's measures (m, `street.ts` `overpass`): its concrete slab `slab` deep under the road, reaching `apron` past the road's edges, closed
 * `lid` under the road's surface (out of sight, there for its shadow, clear of the road's own in the sun's map); its
 * parapet (a safety-shape barrier: a foot `foot` wide and `footHigh` high, then up to `parapet` over the road, `top`
 * wide) along each outer edge; a pier every `pier` m or a little less, beside a road it passes over rather than on it
 * (`street.ts` `pierStations`; `column` square columns, one under a deck
 * narrower than `wide`, two under a wider one, on a cross-head `cap` along and `capDeep` deep), each `sink` into the
 * ground; an abutment `abutment` thick under each end that stands clear of it. Where the deck stands less than `low`
 * over the ground, nothing stands under it.
 */
export const OVERPASS = { slab: 1.1, lid: 0.3, apron: 0.5, parapet: 1.0, foot: 0.5, footHigh: 0.3, top: 0.24, pier: 20, column: 1.1, cap: 1.2, capDeep: 0.8, sink: 0.4, abutment: 1.2, wide: 8, low: 2.4 } as const;
/** Its concrete (and the overview's plain decks', `lowDecks`). */
export const OVERPASS_CONCRETE = '#ABA8A0';

/** How far apart along a carriageway the barrier's line is taken (m). */
const BARRIER_STEP = 2;

/**
 * A carriageway's pair: the other one's index, its side (+1: towards (t.z, -t.x) of the way its points run, as
 * `markings.ts` `Lane.off`: its left on the town map, its right as the game shows it), and the verge between their edges
 * at this one's middle (m).
 */
export interface MotorwayPair {
  other: number;
  side: 1 | -1;
  verge: number;
}

const pairsOf = new WeakMap<readonly RoadPlan[], Map<number, MotorwayPair>>();

/**
 * Each motorway carriageway's pair among `roads`: the other carriageway running beside it, the nearest whose middle is
 * within `MOTORWAY.apart` of its edge at its own middle and runs the other way (or, one-way either way, alongside).
 */
export function motorwayPairs(roads: readonly RoadPlan[]): Map<number, MotorwayPair> {
  const had = pairsOf.get(roads);
  if (had) return had;
  const out = new Map<number, MotorwayPair>();
  const ways = roads.flatMap((r, i) => (r.kind === 'motorway' && r.points.length >= 2 ? [i] : []));
  for (const i of ways) {
    const a = roads[i];
    const ls = lengths(a.points);
    const { p, t } = pointAt(a.points, ls[ls.length - 1] / 2, ls);
    let best: (MotorwayPair & { d: number }) | null = null;
    for (const j of ways) {
      if (j === i) continue;
      const b = roads[j];
      const near = nearest(b.points, p[0], p[1]);
      if (near.dist > (a.width + b.width) / 2 + MOTORWAY.apart || (best && near.dist >= best.d)) continue;
      const q = pointAt(b.points, near.s);
      if (Math.abs(q.t[0] * t[0] + q.t[1] * t[1]) < 0.9) continue;
      // (Towards (t.z, -t.x): `MotorwayPair.side`.)
      const side = (q.p[0] - p[0]) * t[1] - (q.p[1] - p[1]) * t[0] >= 0 ? 1 : -1;
      best = { other: j, side, verge: Math.max(0, near.dist - (a.width + b.width) / 2), d: near.dist };
    }
    if (best) out.set(i, { other: best.other, side: best.side, verge: best.verge });
  }
  pairsOf.set(roads, out);
  return out;
}

/**
 * Road `ri`'s inside (+1: towards (t.z, -t.x) of the way its points run, as `MotorwayPair.side`; -1: the other side):
 * towards its pair, if it has one; else, its cars keeping to the right as the game shows it, the left of the way they
 * go (sim-core `Graph::inside`, of the opposite sign: its +1 is the game's left of the way its points run).
 */
export function insideOf(roads: readonly RoadPlan[], ri: number): 1 | -1 {
  const pair = motorwayPairs(roads).get(ri);
  if (pair) return pair.side;
  return onewayOf(roads[ri]) < 0 ? 1 : -1;
}

/** A point on the line between two carriageways: where, how far along each (m), and the verge between their edges (m). */
export interface BarrierPoint {
  x: number;
  z: number;
  sa: number;
  sb: number;
  verge: number;
}

const runsOf = new WeakMap<RoadPlan, Map<RoadPlan, BarrierPoint[][]>>();

/**
 * The line between carriageways `a` and `b` (a pair: `motorwayPairs`), in runs: a point every `BARRIER_STEP` m along
 * `a` where `b` runs beside it (its middle within `MOTORWAY.apart` of `a`'s edge, and no nearer than their edges
 * meeting), midway between their middles. A run breaks where they part (a run of fewer than two points is left out).
 */
export function barrierRuns(a: RoadPlan, b: RoadPlan): BarrierPoint[][] {
  let byB = runsOf.get(a);
  const had = byB?.get(b);
  if (had) return had;
  const lsA = lengths(a.points);
  const total = lsA[lsA.length - 1];
  const index = new RoadIndex([b]);
  const lsB = lengths(b.points);
  const touch = (a.width + b.width) / 2;
  const runs: BarrierPoint[][] = [];
  let run: BarrierPoint[] = [];
  const n = Math.max(1, Math.round(total / BARRIER_STEP));
  for (let k = 0; k <= n; k++) {
    const sa = (total * k) / n;
    const { p, t } = pointAt(a.points, sa, lsA);
    const d = index.distance(0, p[0], p[1], touch + MOTORWAY.apart);
    const sb = d < Infinity ? index.along(0, p[0], p[1]) : 0;
    const q = d < Infinity ? pointAt(b.points, sb, lsB) : null;
    // (Beside it: alongside, not where its end turns away, nor past its end.)
    const beside = q && d >= touch - 1 && Math.abs(q.t[0] * t[0] + q.t[1] * t[1]) > 0.9 && sb > 0.01 && sb < lsB[lsB.length - 1] - 0.01;
    if (!beside) {
      if (run.length > 1) runs.push(run);
      run = [];
      continue;
    }
    run.push({ x: (p[0] + q.p[0]) / 2, z: (p[1] + q.p[1]) / 2, sa, sb, verge: Math.max(0, d - touch) });
  }
  if (run.length > 1) runs.push(run);
  if (!byB) runsOf.set(a, (byB = new Map()));
  byB.set(b, runs);
  return runs;
}

/** Every pair's barrier runs among `roads`, each pair once: the two carriageways' indexes with each run. */
export function allBarriers(roads: readonly RoadPlan[]): { a: number; b: number; runs: BarrierPoint[][] }[] {
  const out: { a: number; b: number; runs: BarrierPoint[][] }[] = [];
  for (const [a, { other: b }] of motorwayPairs(roads)) {
    // (Each pair once: from the first of the two, or from either where only one found the other.)
    if (a > b && motorwayPairs(roads).get(b)?.other === a) continue;
    out.push({ a, b, runs: barrierRuns(roads[a], roads[b]) });
  }
  return out;
}

/**
 * The barrier's profile across (m from its middle, m up): the full one, a foot then the face leaning in to its top
 * (a safety-shape barrier); `plain`, the overview's, three faces.
 */
function profile(plain: boolean): [number, number][] {
  const M = MOTORWAY;
  const [b, t] = [M.base / 2, M.top / 2];
  if (plain) return [[-b, 0], [-t, M.high], [t, M.high], [b, 0]];
  return [[-b, 0], [-b, M.foot], [-b + 0.11, M.kink], [-t, M.high], [t, M.high], [b - 0.11, M.kink], [b, M.foot], [b, 0]];
}

/**
 * The barrier along `run` into `g` (its concrete as a vertex colour in the plain finish), standing at `y(p)` (the
 * carriageways' height there: their mean), its ends closed. `plain`: the overview's, three faces a piece.
 */
export function barrier(g: Geo, run: readonly BarrierPoint[], y: (p: BarrierPoint) => number, plain = false): void {
  if (run.length < 2) return;
  const prof = profile(plain);
  g.color(BARRIER_CONCRETE, BARRIER_FINISH);
  const ys = run.map(y);
  // Each point's left (x, z turned a quarter: (z, -x)), from the line through its neighbours.
  const left = run.map((_, i) => {
    const [a, b] = [run[Math.max(0, i - 1)], run[Math.min(run.length - 1, i + 1)]];
    const l = Math.hypot(b.x - a.x, b.z - a.z) || 1;
    return [(b.z - a.z) / l, -(b.x - a.x) / l] as Pt;
  });
  const at = (i: number, [u, v]: [number, number]): V3 => [run[i].x + left[i][0] * u, ys[i] + v, run[i].z + left[i][1] * u];
  for (let i = 0; i + 1 < run.length; i++) {
    for (let k = 0; k + 1 < prof.length; k++) {
      const [p, q] = [prof[k], prof[k + 1]];
      // (The face's normal across: its profile's edge turned out, from the right (-u) over the top to the left.)
      const [du, dv] = [q[0] - p[0], q[1] - p[1]];
      const l = Math.hypot(du, dv) || 1;
      const [nu, nv] = [-dv / l, du / l];
      const L = left[i];
      g.poly([at(i, p), at(i + 1, p), at(i + 1, q), at(i, q)], [L[0] * nu, nv, L[1] * nu]);
    }
  }
  // Its two ends.
  for (const [i, s] of [[0, -1], [run.length - 1, 1]] as const) {
    const j = s < 0 ? 1 : run.length - 2;
    const [dx, dz] = [run[i].x - run[j].x, run[i].z - run[j].z];
    const l = Math.hypot(dx, dz) || 1;
    g.poly(prof.map((p) => at(i, p)), [dx / l, 0, dz / l]);
  }
}

/** Each set of roads' deck heights on a world with no built surface (`deckLevels`), once. */
const decksOf = new WeakMap<readonly RoadPlan[], RoadLevels | null>();

/**
 * The heights of `roads` raised over others (`RoadPlan.raised`) on a world with no built surface to give them (null:
 * none is raised, everything at 0 as ever): a raised stretch's road `MOTORWAY.clear` m up, its approaches climbing to it
 * at its kind's grade (smoothly, from the ground); a ramp merging into a raised carriageway from that carriageway's
 * height at its merging end down to the ground at its other; every other road and every junction at 0.
 */
export function deckLevels(roads: readonly RoadPlan[]): RoadLevels | null {
  if (decksOf.has(roads)) return decksOf.get(roads)!;
  const raised = new Map<RoadPlan, [number, number][]>();
  for (const r of roads) {
    const spans = raisedSpans(r);
    if (spans.length) raised.set(r, spans);
  }
  if (!raised.size) {
    decksOf.set(roads, null);
    return null;
  }
  const deck = (r: RoadPlan, s: number): number => {
    const spans = raised.get(r);
    if (!spans) return 0;
    const reach = MOTORWAY.clear / Math.max(0.02, kindInfo(r).grade);
    let y = 0;
    for (const [a, b] of spans) {
      const off = s < a ? a - s : s > b ? s - b : 0;
      const t = Math.max(0, 1 - off / reach);
      y = Math.max(y, MOTORWAY.clear * t * t * (3 - 2 * t));
    }
    return y;
  };
  // A ramp's height from its merging end's (the carriageway's there) down to its other.
  const ramps = new Map<RoadPlan, { from: number; y: number; length: number }>();
  const carriageways = roads.filter((r) => raised.has(r));
  for (const r of roads) {
    const e = r.ends[0] === 'merge' ? 0 : r.ends[1] === 'merge' ? 1 : -1;
    if (e < 0 || r.points.length < 2) continue;
    const p = e === 0 ? r.points[0] : r.points[r.points.length - 1];
    let y = 0;
    for (const c of carriageways) {
      const near = nearest(c.points, p[0], p[1]);
      if (near.dist <= c.width / 2) y = Math.max(y, deck(c, near.s));
    }
    const ls = lengths(r.points);
    if (y > 0) ramps.set(r, { from: e, y, length: ls[ls.length - 1] });
  }
  const levels: RoadLevels = {
    road(r, s) {
      const ramp = ramps.get(r);
      if (ramp) {
        const t = Math.max(0, Math.min(1, ramp.from === 0 ? 1 - s / ramp.length : s / ramp.length));
        return ramp.y * t * t * (3 - 2 * t);
      }
      return deck(r, s);
    },
    junction: () => 0,
  };
  decksOf.set(roads, levels);
  return levels;
}

/** A road stands up off the ground (on walls, `liftedStretches`) where its height (`deckLevels`) is more than this (m). */
const LIFTED = 0.4;

/** Where `road` stands up off the ground on `levels` (`deckLevels`): its stretches (arc lengths, m), a metre at a time. */
export function liftedStretches(road: RoadPlan, levels: RoadLevels): [number, number][] {
  if (road.points.length < 2) return [];
  const ls = lengths(road.points);
  const total = ls[ls.length - 1];
  const out: [number, number][] = [];
  const n = Math.max(1, Math.ceil(total));
  let from: number | null = null;
  for (let k = 0; k <= n; k++) {
    const s = (total * k) / n;
    const up = levels.road(road, s) > LIFTED;
    if (up && from === null) from = s;
    if (!up && from !== null) {
      out.push([from, s]);
      from = null;
    }
  }
  if (from !== null) out.push([from, total]);
  return out;
}

/** `stretches` less `cut` (both arc lengths, in order), each left longer than `least`. */
export function without(stretches: readonly [number, number][], cut: readonly [number, number][], least = 0.05): [number, number][] {
  let out = stretches.map(([a, b]): [number, number] => [a, b]);
  for (const [c0, c1] of cut) out = out.flatMap(([a, b]): [number, number][] => (c1 <= a || c0 >= b ? [[a, b]] : [...(c0 > a ? [[a, c0] as [number, number]] : []), ...(c1 < b ? [[c1, b] as [number, number]] : [])]));
  return out.filter(([a, b]) => b - a > least);
}

/**
 * The point `off` m to the side of `p` on a road going `t` there (+: towards (t.z, -t.x), `leftOf`: its left on the town
 * map, its right as the game shows it; as `Lane.off` and `MotorwayPair.side`).
 */
export function aside(p: Pt, t: Pt, off: number): Pt {
  const l = leftOf(t);
  return [p[0] + l[0] * off, p[1] + l[1] * off];
}

/** A road's points by arc length, its way along and its left there: walked forward (`at`), back only from the start. */
export class Walk {
  private i = 0;
  constructor(
    private readonly pts: readonly Pt[],
    private readonly ls: readonly number[],
  ) {}

  at(s: number): { p: Pt; t: Pt } {
    const { pts, ls } = this;
    if (s < ls[this.i]) this.i = 0;
    while (this.i < ls.length - 2 && ls[this.i + 1] < s) this.i++;
    const i = this.i;
    const len = ls[i + 1] - ls[i] || 1;
    const f = Math.max(0, Math.min(1, (s - ls[i]) / len));
    const [a, b] = [pts[i], pts[i + 1]];
    const t: Pt = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
    return { p: [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f], t };
  }
}

/** How long the overview's deck pieces are at most (m). */
const LOW_DECK_STEP = 4;

/**
 * The overview's decks (the overpasses, `RoadPlan.raised`, and a motorway's or a ramp's bridges) and the barriers
 * between the motorways' carriageways, plainly, into `g` (vertex colours in the plain finish): a deck's two sides from
 * its parapet's top down to under its slab, its parapets' faces to the road and its underside; on a world with no built
 * surface (`levels` from `deckLevels`, `solid`), a ramp up to one walled down to the ground. Their slab and parapet as
 * deep and as high as an overpass's (`OVERPASS`).
 */
export function lowDecks(g: Geo, roads: readonly RoadPlan[], levels: RoadLevels | null, solid: boolean, spansOf: (r: RoadPlan) => [number, number][]): void {
  g.color(OVERPASS_CONCRETE, BARRIER_FINISH);
  const pairs = motorwayPairs(roads);
  roads.forEach((road, ri) => {
    if (road.points.length < 2) return;
    const raised = raisedSpans(road);
    const concrete = isSeparated(road.kind);
    const open = [...raised, ...(concrete ? spansOf(road) : [])];
    const walled = solid && levels ? without(liftedStretches(road, levels), open) : [];
    if (!open.length && !walled.length) return;
    const ls = lengths(road.points);
    const walk = new Walk(road.points, ls);
    const inner = pairs.get(ri)?.side ?? 0;
    const Y = (s: number) => (levels ? levels.road(road, s) : 0);
    const half = road.width / 2;
    for (const [list, down] of [[open, false], [walled, true]] as const) {
      for (const [s0, s1] of list) {
        const n = Math.max(1, Math.ceil((s1 - s0) / LOW_DECK_STEP));
        for (let k = 0; k < n; k++) {
          const [a, b] = [s0 + ((s1 - s0) * k) / n, s0 + ((s1 - s0) * (k + 1)) / n];
          const [pa, pb] = [walk.at(a), walk.at(b)];
          const [ya, yb] = [Y(a), Y(b)];
          const P = (q: { p: Pt; t: Pt }, off: number, y: number): V3 => {
            const [x, z] = aside(q.p, q.t, off);
            return [x, y, z];
          };
          for (const side of [1, -1] as const) {
            if (side === inner) continue;
            const o = side * half;
            const out: V3 = [pa.t[1] * side, 0, -pa.t[0] * side];
            const [ba, bb] = down ? [0, 0] : [ya - OVERPASS.slab, yb - OVERPASS.slab];
            g.poly([P(pa, o, ba), P(pb, o, bb), P(pb, o, yb + OVERPASS.parapet), P(pa, o, ya + OVERPASS.parapet)], out);
            g.poly([P(pa, o, ya + OVERPASS.parapet), P(pb, o, yb + OVERPASS.parapet), P(pb, o, yb), P(pa, o, ya)], [-out[0], 0, -out[2]]);
          }
          if (!down) g.poly([P(pa, half, ya - OVERPASS.slab), P(pa, -half, ya - OVERPASS.slab), P(pb, -half, yb - OVERPASS.slab), P(pb, half, yb - OVERPASS.slab)], [0, -1, 0]);
        }
      }
    }
  });
  for (const { a, b, runs } of allBarriers(roads)) for (const run of runs) barrier(g, run, (q) => (levels ? (levels.road(roads[a], q.sa) + levels.road(roads[b], q.sb)) / 2 : 0), true);
}

/** The motorway's direction sign (`crates/art/outside/src/street_furniture/motorway.rs`), its face (+z) to the traffic coming. */
export const MOTORWAY_SIGN = 'model.motorwaySign';
/** Where the signs stand (m): before an exit along its carriageway, how far out past a road's edge, and how wide a gore must be for one. */
const SIGN_BEFORE = [150, 300] as const;
const SIGN_OUT = 1.6;
const SIGN_GORE = 4;
/** How far from a ramp's end at the street its sign stands (m). */
const SIGN_RAMP_IN = 10;

/** A sign's spot: where it stands, which way it faces (yaw: its +z turned to (sin, cos)), and how high its foot is. */
export interface SignSpot {
  x: number;
  z: number;
  yaw: number;
  y: number;
}

/** A sign's place, its height still to come: on road `on[k][0]` at `on[k][1]` m along it (two: at their mean height). */
interface SignPlace {
  x: number;
  z: number;
  yaw: number;
  on: readonly (readonly [RoadPlan, number])[];
}

/** Each set of roads' signs' places (`signPlaces`), once: the heights alone move with the land. */
const signsOf = new WeakMap<readonly RoadPlan[], SignPlace[]>();

/**
 * Where the motorway's direction signs stand among `roads`, at their heights (`levels`; none: 0): for each off-ramp
 * (a ramp leaving its carriageway at its merging end), beside the carriageway `SIGN_BEFORE` m before the exit where it
 * runs that far, outside its shoulder, and in the gore where the ramp has parted from it far enough; for each on-ramp, by
 * the street where it begins, on its right. Each faces the traffic coming to it.
 */
export function signSpots(roads: readonly RoadPlan[], levels?: RoadLevels | null): SignSpot[] {
  let places = signsOf.get(roads);
  if (!places) signsOf.set(roads, (places = signPlaces(roads)));
  const Y = ([r, s]: readonly [RoadPlan, number]) => (levels ? levels.road(r, s) : 0);
  return places.map(({ x, z, yaw, on }) => ({ x, z, yaw, y: on.length > 1 ? (Y(on[0]) + Y(on[1])) / 2 : Y(on[0]) }));
}

/** Where `signSpots`' signs stand, and on what. */
function signPlaces(roads: readonly RoadPlan[]): SignPlace[] {
  const out: SignPlace[] = [];
  const facing = (t: Pt, way: number) => Math.atan2(-t[0] * way, -t[1] * way);
  // (Each carriageway's index, for the gore: built when first asked.)
  const indexes = new Map<RoadPlan, RoadIndex>();
  roads.forEach((ramp) => {
    if (ramp.kind !== 'ramp' || ramp.points.length < 2) return;
    const e = ramp.ends[0] === 'merge' ? 0 : ramp.ends[1] === 'merge' ? 1 : -1;
    if (e < 0) return;
    const way = onewayOf(ramp) < 0 ? -1 : 1;
    const ls = lengths(ramp.points);
    const total = ls[ls.length - 1];
    // (Leaving: its cars go from its merging end; joining: towards it.)
    const leaves = (e === 0) === way > 0;
    const mp = e === 0 ? ramp.points[0] : ramp.points[ramp.points.length - 1];
    const ci = roads.findIndex((c) => c.kind === 'motorway' && c.points.length >= 2 && nearest(c.points, mp[0], mp[1]).dist <= c.width / 2);
    if (leaves && ci >= 0) {
      const c = roads[ci];
      const cls = lengths(c.points);
      const cWay = onewayOf(c) < 0 ? -1 : 1;
      const at = nearest(c.points, mp[0], mp[1], cls).s;
      const outside = -insideOf(roads, ci);
      for (const before of SIGN_BEFORE) {
        const s = at - before * cWay;
        if (s < 0 || s > cls[cls.length - 1]) continue;
        const { p, t } = pointAt(c.points, s, cls);
        const [x, z] = aside(p, t, outside * (c.width / 2 + SIGN_OUT));
        out.push({ x, z, yaw: facing(t, cWay), on: [[c, s]] });
      }
      // In the gore: the first place along the ramp it has parted from the carriageway far enough, midway between.
      let index = indexes.get(c);
      if (!index) indexes.set(c, (index = new RoadIndex([c])));
      const apart = (c.width + ramp.width) / 2 + SIGN_GORE;
      for (let k = 1; k * 2 < total; k++) {
        const s = e === 0 ? k * 2 : total - k * 2;
        const { p, t } = pointAt(ramp.points, s, ls);
        // (Plainly too near, by the index: a millimetre short of far enough, so the measure below never decides otherwise.)
        if (index.distance(0, p[0], p[1], apart - 1e-3) < Infinity) continue;
        const near = nearest(c.points, p[0], p[1], cls);
        const gap = near.dist - (c.width + ramp.width) / 2;
        if (gap < SIGN_GORE) continue;
        const q = pointAt(c.points, near.s, cls).p;
        // (Midway across the gap, out from both roads' middles: their edges half the gap away each.)
        const f = (ramp.width / 2 + gap / 2) / (near.dist || 1);
        out.push({ x: p[0] + (q[0] - p[0]) * f, z: p[1] + (q[1] - p[1]) * f, yaw: facing(t, way), on: [[ramp, s], [c, near.s]] });
        break;
      }
    }
    if (!leaves) {
      // By the street where it begins: on its right as the game shows it ((t.z, -t.x) going along its points), a
      // little way in.
      const s = e === 0 ? total - SIGN_RAMP_IN : SIGN_RAMP_IN;
      if (s <= 0 || s >= total) return;
      const { p, t } = pointAt(ramp.points, s, ls);
      const [x, z] = aside(p, t, way * (ramp.width / 2 + SIGN_OUT));
      out.push({ x, z, yaw: facing(t, way), on: [[ramp, s]] });
    }
  });
  return out;
}
