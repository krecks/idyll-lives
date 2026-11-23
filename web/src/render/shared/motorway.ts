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
import { kindInfo, onewayOf } from '../../core/roadKinds';
import { lengths, nearest, pointAt, RoadIndex, type Pt } from '../../game/roads';
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
const BARRIER_FINISH = encodeFinish(0.92, 0);
/** How far apart along a carriageway the barrier's line is taken (m). */
const BARRIER_STEP = 2;

/** A carriageway's pair: the other one's index, and its side (+1: this one's left, looking along its points). */
export interface MotorwayPair {
  other: number;
  side: 1 | -1;
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
      // (The left of the way along: (t.z, -t.x).)
      const side = (q.p[0] - p[0]) * t[1] - (q.p[1] - p[1]) * t[0] >= 0 ? 1 : -1;
      best = { other: j, side, d: near.dist };
    }
    if (best) out.set(i, { other: best.other, side: best.side });
  }
  pairsOf.set(roads, out);
  return out;
}

/**
 * Road `ri`'s inside (+1: its left, looking along its points; -1: its right): towards its pair, if it has one; else, its
 * cars keeping to the right, the left of the way they go (a two-way road: its left).
 */
export function insideOf(roads: readonly RoadPlan[], ri: number): 1 | -1 {
  const pair = motorwayPairs(roads).get(ri);
  if (pair) return pair.side;
  return onewayOf(roads[ri]) < 0 ? -1 : 1;
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

/**
 * A road's raised stretches as arc lengths (m) along it, each start and end (`RoadPlan.raised`), kept within it and in
 * order.
 */
export function raisedSpans(road: RoadPlan): [number, number][] {
  if (!road.raised?.length || road.points.length < 2) return [];
  const ls = lengths(road.points);
  const total = ls[ls.length - 1];
  return road.raised
    .map(([a, b]): [number, number] => [Math.max(0, Math.min(a, b)), Math.min(total, Math.max(a, b))])
    .filter(([a, b]) => b - a > 0.01)
    .sort((p, q) => p[0] - q[0]);
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

/** `stretches` less `cut` (both arc lengths, in order). */
export function without(stretches: readonly [number, number][], cut: readonly [number, number][]): [number, number][] {
  let out = stretches.map(([a, b]): [number, number] => [a, b]);
  for (const [c0, c1] of cut) out = out.flatMap(([a, b]): [number, number][] => (c1 <= a || c0 >= b ? [[a, b]] : [...(c0 > a ? [[a, c0] as [number, number]] : []), ...(c1 < b ? [[c1, b] as [number, number]] : [])]));
  return out.filter(([a, b]) => b - a > 0.05);
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

/** An overpass's measures as the overview draws them (m: `street.ts` `OVERPASS`, which this module can't import): the slab's depth, the parapet's height. */
const LOW_DECK = { slab: 1.1, parapet: 1.0, step: 4 } as const;
const LOW_CONCRETE = '#ABA8A0';

/**
 * The overview's decks (the overpasses, `RoadPlan.raised`, and a motorway's or a ramp's bridges) and the barriers
 * between the motorways' carriageways, plainly, into `g` (vertex colours in the plain finish): a deck's two sides from
 * its parapet's top down to under its slab, its parapets' faces to the road and its underside; on a world with no built
 * surface (`levels` from `deckLevels`, `solid`), a ramp up to one walled down to the ground.
 */
export function lowDecks(g: Geo, roads: readonly RoadPlan[], levels: RoadLevels | null, solid: boolean, spansOf: (r: RoadPlan) => [number, number][]): void {
  const finish = encodeFinish(0.92, 0);
  g.color(LOW_CONCRETE, finish);
  const pairs = motorwayPairs(roads);
  roads.forEach((road, ri) => {
    if (road.points.length < 2) return;
    const raised = raisedSpans(road);
    const concrete = road.kind === 'motorway' || road.kind === 'ramp';
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
        const n = Math.max(1, Math.ceil((s1 - s0) / LOW_DECK.step));
        for (let k = 0; k < n; k++) {
          const [a, b] = [s0 + ((s1 - s0) * k) / n, s0 + ((s1 - s0) * (k + 1)) / n];
          const [pa, pb] = [walk.at(a), walk.at(b)];
          const [ya, yb] = [Y(a), Y(b)];
          const P = (q: { p: Pt; t: Pt }, off: number, y: number): V3 => [q.p[0] + q.t[1] * off, y, q.p[1] - q.t[0] * off];
          for (const side of [1, -1] as const) {
            if (side === inner) continue;
            const o = side * half;
            const out: V3 = [pa.t[1] * side, 0, -pa.t[0] * side];
            const [ba, bb] = down ? [0, 0] : [ya - LOW_DECK.slab, yb - LOW_DECK.slab];
            g.poly([P(pa, o, ba), P(pb, o, bb), P(pb, o, yb + LOW_DECK.parapet), P(pa, o, ya + LOW_DECK.parapet)], out);
            g.poly([P(pa, o, ya + LOW_DECK.parapet), P(pb, o, yb + LOW_DECK.parapet), P(pb, o, yb), P(pa, o, ya)], [-out[0], 0, -out[2]]);
          }
          if (!down) g.poly([P(pa, half, ya - LOW_DECK.slab), P(pa, -half, ya - LOW_DECK.slab), P(pb, -half, yb - LOW_DECK.slab), P(pb, half, yb - LOW_DECK.slab)], [0, -1, 0]);
        }
      }
    }
  });
  for (const { a, b, runs } of allBarriers(roads)) for (const run of runs) barrier(g, run, (q) => (levels ? (levels.road(roads[a], q.sa) + levels.road(roads[b], q.sb)) / 2 : 0), true);
}
