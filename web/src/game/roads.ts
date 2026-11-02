/**
 * A town's roads, laid out from a few shapes (`through`, `branch`, `loop`, `ring`, `lane`), each
 * a smooth curve through a handful of points (Catmull-Rom, a point every metre). Each kind of
 * world puts its own together (`WorldRecipe.roads` in `world.ts`): a winding high street, a
 * crescent round a green, a crossroads, a lane along the shore, a ring round a clearing. The lots
 * go along them (`town.ts` `generateWorld`), turned to face the nearest one.
 *
 * Roads are drawn only: the simulation knows the town's tiles, its plots and the ways out of
 * town (`exits`, where roads leave it). See `RoadPlan` in `core/protocol.ts` for what is kept.
 */

import type { RoadEnd, RoadPlan } from '../core/protocol';

export type { RoadEnd, RoadPlan };
type Rand = () => number;
export type Pt = [number, number];

/** Road widths (m, kerb to kerb with the sidewalks): the main road and the lanes off it. */
export const MAIN_WIDTH = 7;
export const LANE_WIDTH = 6;
/** Sidewalk width (including the kerb) on each side of a road. */
export const SIDEWALK = 1.35;


/** The area a town's roads are laid out in (m). */
export interface Box {
  w: number;
  d: number;
}

const lerp = (a: number, b: number, t: number) => a + (b - a) * t;
/** `lo..hi`, evenly. */
export const range = (rand: Rand, lo: number, hi: number) => lerp(lo, hi, rand());

/** Points along a Catmull-Rom curve through `ctrl` (its ends included), about `step` m apart. */
export function curve(ctrl: readonly Pt[], step = 1, closed = false): Pt[] {
  const n = ctrl.length;
  const at = (i: number) => (closed ? ctrl[((i % n) + n) % n] : ctrl[Math.max(0, Math.min(n - 1, i))]);
  const out: Pt[] = [];
  const spans = closed ? n : n - 1;
  for (let i = 0; i < spans; i++) {
    const [p0, p1, p2, p3] = [at(i - 1), at(i), at(i + 1), at(i + 2)];
    const k = Math.max(2, Math.ceil(Math.hypot(p2[0] - p1[0], p2[1] - p1[1]) / step));
    for (let j = 0; j < k; j++) {
      const t = j / k;
      const [t2, t3] = [t * t, t * t * t];
      const c = (a: number, b: number, c: number, d: number) => 0.5 * (2 * b + (-a + c) * t + (2 * a - 5 * b + 4 * c - d) * t2 + (-a + 3 * b - 3 * c + d) * t3);
      out.push([c(p0[0], p1[0], p2[0], p3[0]), c(p0[1], p1[1], p2[1], p3[1])]);
    }
  }
  out.push(closed ? [...out[0]] : [...ctrl[n - 1]]);
  return out;
}

/** Arc length at each point of a polyline. */
export function lengths(pts: readonly Pt[]): number[] {
  const out = [0];
  for (let i = 1; i < pts.length; i++) out.push(out[i - 1] + Math.hypot(pts[i][0] - pts[i - 1][0], pts[i][1] - pts[i - 1][1]));
  return out;
}

/** The unit direction along a polyline at point `i`. */
export function tangent(pts: readonly Pt[], i: number): Pt {
  const [a, b] = [pts[Math.max(0, i - 1)], pts[Math.min(pts.length - 1, i + 1)]];
  const l = Math.hypot(b[0] - a[0], b[1] - a[1]) || 1;
  return [(b[0] - a[0]) / l, (b[1] - a[1]) / l];
}

/** The left of a direction (x, z): a quarter turn, as seen from above with z down the page. */
export const leftOf = ([x, z]: Pt): Pt => [z, -x];

/** The point at arc length `s` along a polyline, and the direction there. */
export function pointAt(pts: readonly Pt[], s: number, ls = lengths(pts)): { p: Pt; t: Pt; i: number } {
  let i = 0;
  while (i < ls.length - 2 && ls[i + 1] < s) i++;
  const f = Math.max(0, Math.min(1, (s - ls[i]) / (ls[i + 1] - ls[i] || 1)));
  const [a, b] = [pts[i], pts[i + 1] ?? pts[i]];
  return { p: [lerp(a[0], b[0], f), lerp(a[1], b[1], f)], t: tangent(pts, f < 0.5 ? i : i + 1), i };
}

/** The nearest point of a polyline: its distance, arc length and which side (+1: the left). */
export function nearest(pts: readonly Pt[], x: number, z: number, ls = lengths(pts)): { dist: number; s: number; side: 1 | -1 } {
  let best = { dist: Infinity, s: 0, side: 1 as 1 | -1 };
  for (let i = 0; i + 1 < pts.length; i++) {
    const [[ax, az], [bx, bz]] = [pts[i], pts[i + 1]];
    const [dx, dz] = [bx - ax, bz - az];
    const l2 = dx * dx + dz * dz || 1;
    const t = Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / l2));
    const dist = Math.hypot(x - (ax + dx * t), z - (az + dz * t));
    if (dist < best.dist) {
      // Left of the way along: (dx, dz) turned a quarter, as `leftOf`.
      const cross = (x - ax) * dz - (z - az) * dx;
      best = { dist, s: ls[i] + t * Math.sqrt(l2), side: cross >= 0 ? 1 : -1 };
    }
  }
  return best;
}

// ---- shapes ------------------------------------------------------------------------------

/**
 * A road across the whole of `box` (along x, or along z), winding gently: `bends` turns, as far as
 * `amp` m off the middle (`at`: where across the box its middle runs, 0..1). Both ends lead out of town.
 */
export function through(rand: Rand, box: Box, { along = 'x', at = 0.5, amp = 10, bends = 2, width = MAIN_WIDTH }: { along?: 'x' | 'z'; at?: number; amp?: number; bends?: number; width?: number } = {}): RoadPlan {
  const [len, across] = along === 'x' ? [box.w, box.d] : [box.d, box.w];
  const n = bends + 2;
  const ctrl: Pt[] = [];
  for (let i = 0; i <= n; i++) {
    const u = lerp(-24, len + 24, i / n);
    // Straight where it leaves town (the country road goes on from there).
    const off = i === 0 || i === n ? 0 : (i % 2 ? 1 : -1) * amp * range(rand, 0.45, 1);
    const v = across * at + off;
    ctrl.push(along === 'x' ? [u, v] : [v, u]);
  }
  return { points: curve(ctrl), width, ends: ['out', 'out'] };
}

/**
 * A lane off `parent` at arc length `s`, on its `side`, `length` m long, curling `curl` radians
 * as it goes (a dead end, or `end`).
 */
export function branch(parent: RoadPlan, s: number, side: 1 | -1, length: number, curl: number, { width = LANE_WIDTH, end = 'dead' as RoadEnd } = {}): RoadPlan {
  const { p, t } = pointAt(parent.points, s);
  const n = leftOf(t).map((v) => v * side) as Pt;
  const turn = (a: number): Pt => [n[0] * Math.cos(a) - n[1] * Math.sin(a), n[0] * Math.sin(a) + n[1] * Math.cos(a)];
  const ctrl: Pt[] = [p];
  for (const k of [0.34, 0.67, 1]) {
    const d = turn(curl * k);
    const prev = ctrl[ctrl.length - 1];
    ctrl.push([prev[0] + d[0] * length * 0.34, prev[1] + d[1] * length * 0.34]);
  }
  return { points: curve(ctrl), width, ends: ['join', end] };
}

/**
 * A crescent off `parent`: leaves it at arc length `s0` and comes back at `s1`, bowing out `depth`
 * m on its `side` round a green (the green: what it encloses with the parent, `green`).
 */
export function loop(parent: RoadPlan, s0: number, s1: number, side: 1 | -1, depth: number, width = LANE_WIDTH): { road: RoadPlan; green: Pt[] } {
  const a = pointAt(parent.points, s0);
  const b = pointAt(parent.points, s1);
  const out = (q: { p: Pt; t: Pt }, k: number): Pt => {
    const n = leftOf(q.t);
    return [q.p[0] + n[0] * side * depth * k, q.p[1] + n[1] * side * depth * k];
  };
  const mid = pointAt(parent.points, (s0 + s1) / 2);
  const ctrl: Pt[] = [a.p, out(a, 0.75), out(mid, 1.05), out(b, 0.75), b.p];
  const points = curve(ctrl);
  const ls = lengths(parent.points);
  const back = parent.points.filter((_, i) => ls[i] > s0 && ls[i] < s1).reverse();
  return { road: { points, width, ends: ['join', 'join'] }, green: [...points, ...back] };
}

/** A lane round a clearing: a closed loop round (`cx`, `cz`), `rx` by `rz` m, wobbling a little. */
export function ring(rand: Rand, cx: number, cz: number, rx: number, rz: number, width = LANE_WIDTH): RoadPlan {
  const n = 7;
  const turn = rand() * Math.PI * 2;
  const ctrl: Pt[] = [];
  for (let i = 0; i < n; i++) {
    const a = turn + (i / n) * Math.PI * 2;
    const k = range(rand, 0.88, 1.08);
    ctrl.push([cx + Math.cos(a) * rx * k, cz + Math.sin(a) * rz * k]);
  }
  return { points: curve(ctrl, 1, true), width, ends: ['loop', 'loop'] };
}

/** A roundabout: a small ring round (`cx`, `cz`), `r` m to the middle of its lane, a fountain in the middle. */
export function roundabout(cx: number, cz: number, r: number, width = LANE_WIDTH): RoadPlan {
  const n = Math.max(12, Math.round((2 * Math.PI * r) / 1.5));
  const ctrl: Pt[] = Array.from({ length: n }, (_, i): Pt => [cx + Math.cos((i / n) * Math.PI * 2) * r, cz + Math.sin((i / n) * Math.PI * 2) * r]);
  return { points: curve(ctrl, 1, true), width, ends: ['loop', 'loop'], island: 'fountain' };
}

/** Where two roads cross (nearest the middle of each, if they cross more than once): null if they don't come within `within` m. */
export function crossing(a: RoadPlan, b: RoadPlan, within = 2): Pt | null {
  let best: { d: number; p: Pt } | null = null;
  for (const p of a.points) {
    const { dist } = nearest(b.points, p[0], p[1]);
    if (dist < within && (!best || dist < best.d)) best = { d: dist, p };
  }
  return best?.p ?? null;
}

/** `road` with what lies within `r` m of `c` taken out: the pieces either side, joining what goes there. */
export function cutAround(road: RoadPlan, c: Pt, r: number): RoadPlan[] {
  const near = (p: Pt) => Math.hypot(p[0] - c[0], p[1] - c[1]) < r;
  const first = road.points.findIndex(near);
  if (first < 0) return [road];
  let last = first;
  while (last + 1 < road.points.length && near(road.points[last + 1])) last++;
  const pieces: RoadPlan[] = [];
  if (first >= 2) pieces.push({ ...road, points: road.points.slice(0, first + 1), ends: [road.ends[0], 'join'] });
  if (road.points.length - last >= 3) pieces.push({ ...road, points: road.points.slice(last), ends: ['join', road.ends[1]] });
  return pieces;
}

/** A lane through points of the caller's choosing (`ends`: what is at each end). */
export function lane(ctrl: readonly Pt[], ends: [RoadEnd, RoadEnd], width = LANE_WIDTH): RoadPlan {
  return { points: curve(ctrl), width, ends };
}

// ---- joining the roads up ------------------------------------------------------------------

/** A lane left with a dead end (a court) reaches at most this far past the last road it meets (m). */
export const COURT = 45;
/** Roads running alongside each other keep this far apart (m, from the other's edge to the middle). */
const APART = 18;
/** Cross streets keep further from the roads they run alongside (room for a lot between, m). */
const STREETS_APART = 30;
/** Where a road meets another, no third comes within this of it (m). */
const ROOM = 14;
/** The tightest bend a new stretch of road takes (m, to its middle), and the least angle it meets a road at. */
const TIGHTEST = 9;
const MEET = (35 * Math.PI) / 180;
/** How far a dead end looks for a road to go on to (m). */
const REACH = 120;

/** Where new stretches of road may run: inside `box`, `margin` m from its edge (default 14), off the `greens`. */
export interface Fit {
  box: Box;
  margin?: number;
  greens?: readonly Pt[][];
}

const unit = (x: number, z: number): Pt => {
  const l = Math.hypot(x, z) || 1;
  return [x / l, z / l];
};
/** `v` turned `a` radians. */
const turned = ([x, z]: Pt, a: number): Pt => [x * Math.cos(a) - z * Math.sin(a), x * Math.sin(a) + z * Math.cos(a)];
const dot = (a: Pt, b: Pt) => a[0] * b[0] + a[1] * b[1];
/** The signed angle from `a` to `b` (radians). */
const angleTo = (a: Pt, b: Pt) => Math.atan2(a[0] * b[1] - a[1] * b[0], dot(a, b));

/** The way a road leaves at its last point (a unit direction, from 3 m back). */
function endDir(pts: readonly Pt[]): Pt {
  const ls = lengths(pts);
  const back = pointAt(pts, ls[ls.length - 1] - 3, ls).p;
  const e = pts[pts.length - 1];
  return unit(e[0] - back[0], e[1] - back[1]);
}

/**
 * A smooth stretch of road from `a`, leaving it along `da`, to `b`, arriving along `db` (unit
 * directions): a cubic Bézier, as a `curve` (a point every metre or so).
 */
export function connector(a: Pt, da: Pt, b: Pt, db: Pt): Pt[] {
  const k = Math.hypot(b[0] - a[0], b[1] - a[1]) * 0.38;
  const p1: Pt = [a[0] + da[0] * k, a[1] + da[1] * k];
  const p2: Pt = [b[0] - db[0] * k, b[1] - db[1] * k];
  const ctrl: Pt[] = [];
  for (let i = 0; i <= 6; i++) {
    const t = i / 6;
    const [u0, u1, u2, u3] = [(1 - t) ** 3, 3 * (1 - t) ** 2 * t, 3 * (1 - t) * t * t, t ** 3];
    ctrl.push([u0 * a[0] + u1 * p1[0] + u2 * p2[0] + u3 * b[0], u0 * a[1] + u1 * p1[1] + u2 * p2[1] + u3 * b[1]]);
  }
  return curve(ctrl);
}

/** How far a polyline turns left and right (radians, over 3 m steps), and whether it bends tighter than `TIGHTEST` anywhere. */
export function bends(pts: readonly Pt[]): { left: number; right: number; tight: boolean } {
  const ls = lengths(pts);
  const L = ls[ls.length - 1];
  let [left, right, tight] = [0, 0, false];
  let prev: Pt | null = null;
  for (let s = 0; s + 3 <= L + 1e-6; s += 3) {
    const a = pointAt(pts, s, ls).p;
    const b = pointAt(pts, Math.min(L, s + 3), ls).p;
    const t = unit(b[0] - a[0], b[1] - a[1]);
    if (prev) {
      const d = angleTo(prev, t);
      if (d > 0) left += d;
      else right -= d;
      if (Math.abs(d) > 3 / TIGHTEST) tight = true;
    }
    prev = t;
  }
  return { left, right, tight };
}

/**
 * Whether a new stretch of road (`pts`, `width` wide) fits among `roads`: inside the box and off
 * the greens, touching no road but road `from` at its start and road `to` at its end (-1: none),
 * and nowhere running alongside another nearer than `apart` (m, from its edge).
 */
export function clearOf(pts: readonly Pt[], width: number, roads: readonly RoadPlan[], index: RoadIndex, fit: Fit, from: number, to: number, apart = APART): boolean {
  const m = fit.margin ?? 14;
  if (pts.some(([x, z]) => x < m || z < m || x > fit.box.w - m || z > fit.box.d - m)) return false;
  const ls = lengths(pts);
  const L = ls[ls.length - 1];
  const greens = fit.greens ?? [];
  const widest = Math.max(width, ...roads.map((r) => r.width));
  for (let i = 0; i < pts.length; i++) {
    const [x, z] = pts[i];
    const [s, u] = [ls[i], L - ls[i]];
    if (s > 4 && u > 4 && greens.some((g) => inside(g, x, z))) return false;
    const t = tangent(pts, i);
    for (const [r, near] of index.around(x, z, widest / 2 + apart)) {
      const clear = roads[r].width / 2 + width / 2 + 1;
      // (Where it leaves its road and where it meets the other, it comes in at 35° or more.)
      const free = clear * 1.8 + 1;
      if (near.dist < clear && !((r === from && s < free) || (r === to && u < free))) return false;
      // Alongside: beside the other (not beyond its end), running the same way.
      if (s > 20 && u > 20 && near.dist < roads[r].width / 2 + apart && Math.abs(dot(t, near.t)) > Math.cos(MEET)) {
        const q = pointAt(roads[r].points, near.s).p;
        if (Math.abs(dot([x - q[0], z - q[1]], near.t)) < near.dist * 0.5) return false;
      }
    }
  }
  return true;
}

/** Whether no road but `except` comes within `ROOM` m of `p` (room for a junction there). */
function roomAt(index: RoadIndex, p: Pt, except: readonly number[]): boolean {
  for (const r of index.around(p[0], p[1], ROOM).keys()) if (!except.includes(r)) return false;
  return true;
}

/** A way for road `from`'s dead end (its last point) to go on: the new points, the road it meets (`to`, at its dead end if `dead`), its cost. */
interface Join {
  from: number;
  to: number;
  dead: boolean;
  pts: Pt[];
  cost: number;
}

/** The best way on from road `i`'s dead end (its last point) to another road, or to another's dead end; null if none fits. */
function joinFor(roads: readonly RoadPlan[], index: RoadIndex, i: number, fit: Fit): Join | null {
  const r = roads[i];
  const E = r.points[r.points.length - 1];
  const d = endDir(r.points);
  const cands: { T: Pt; a: Pt; to: number; dead: boolean; est: number }[] = [];
  const add = (T: Pt, a: Pt, to: number, dead: boolean) => {
    const v = unit(T[0] - E[0], T[1] - E[1]);
    const L = Math.hypot(T[0] - E[0], T[1] - E[1]);
    if (L < 8 || L > REACH || dot(v, d) < -0.3) return;
    // One bend: the way there between the way out and the way in.
    const [turn, first] = [angleTo(d, a), angleTo(d, v)];
    const between = Math.sign(first) === Math.sign(turn) || Math.abs(first) < 0.15 || Math.abs(angleTo(v, a)) < 0.15;
    cands.push({ T, a, to, dead, est: L + 12 * Math.abs(turn) + (between ? 0 : 40) - (dead ? 20 : 0) });
  };
  roads.forEach((o, q) => {
    if (q === i || o.island || o.kind === 'trail') return;
    if (o.ends[1] === 'dead') {
      const dq = endDir(o.points);
      add(o.points[o.points.length - 1], [-dq[0], -dq[1]], q, true);
    }
    const ls = lengths(o.points);
    const Lq = ls[ls.length - 1];
    const closed = o.ends[0] === 'loop';
    for (let s = closed ? 0 : 10; s <= (closed ? Lq - 2 : Lq - 10); s += 4) {
      const { p: T, t } = pointAt(o.points, s, ls);
      const n = leftOf(t);
      const sgn = dot([E[0] - T[0], E[1] - T[1]], n) >= 0 ? 1 : -1;
      // Into the road from this side: square to it, or up to 32° off.
      const a0: Pt = [-n[0] * sgn, -n[1] * sgn];
      for (const tw of [0, 0.55, -0.55]) add(T, turned(a0, tw), q, false);
    }
  });
  cands.sort((x, y) => x.est - y.est);
  let best: Join | null = null;
  let valid = 0;
  for (const c of cands.slice(0, 120)) {
    if (best && c.est > best.cost + 40) break;
    const pts = connector(E, d, c.T, c.a);
    const b = bends(pts);
    if (b.tight || Math.min(b.left, b.right) > 0.3) continue;
    if (!c.dead) {
      // Square enough to the road it meets, with room there.
      const tq = pointAt(roads[c.to].points, index.along(c.to, c.T[0], c.T[1])).t;
      if (Math.abs(Math.sin(angleTo(endDir(pts), tq))) < Math.sin(MEET)) continue;
      if (!roomAt(index, c.T, [c.to, i])) continue;
    }
    if (!clearOf(pts, r.width, roads, index, fit, i, c.to)) continue;
    const cost = lengths(pts)[pts.length - 1] + 10 * (b.left + b.right) - (c.dead ? 20 : 0);
    if (!best || cost < best.cost) best = { from: i, to: c.to, dead: c.dead, pts, cost };
    if (++valid >= 4) break;
  }
  return best;
}

/** `roads` with join `j` made: the lane goes on to the road it meets, or the two lanes are one. */
function joined(roads: readonly RoadPlan[], j: Join): RoadPlan[] {
  const r = roads[j.from];
  const on = j.pts.slice(1);
  if (!j.dead) return roads.map((x, k) => (k === j.from ? { ...r, points: [...r.points, ...on], ends: [r.ends[0], 'join'] } : x));
  const o = roads[j.to];
  const one: RoadPlan = { ...r, width: Math.max(r.width, o.width), points: [...r.points, ...on.slice(0, -1), ...[...o.points].reverse()], ends: [r.ends[0], o.ends[0]] };
  return roads.flatMap((x, k) => (k === j.to ? [] : k === j.from ? [one] : [x]));
}

/** Arc length along road `i` of the last place another road's end meets it (0: none past its start). */
function lastJoin(roads: readonly RoadPlan[], index: RoadIndex, i: number): number {
  let last = 0;
  roads.forEach((o, k) => {
    if (k === i) return;
    for (const p of [o.points[0], o.points[o.points.length - 1]]) {
      if (index.distance(i, p[0], p[1], Math.max(o.width, roads[i].width) / 2 + 1) < Infinity) last = Math.max(last, index.along(i, p[0], p[1]));
    }
  });
  return last;
}

/** Road `r` up to arc length `s`. */
function cutAt(r: RoadPlan, s: number): Pt[] {
  const ls = lengths(r.points);
  const keep = r.points.filter((_, k) => ls[k] < s - 0.5);
  return [...keep, pointAt(r.points, s, ls).p];
}

/**
 * The town's lanes joined up: every dead end goes on to the nearest road it fits to (or another
 * dead end, the two lanes made one), by a smooth stretch of road with one bend at most, crossing
 * nothing, meeting the other road at 35° or more and keeping clear of the roads alongside it
 * (`Fit`: inside the box, off the greens). Those it can't stay courts, the shortest `courts` of
 * them (cut to `COURT` m past the last road they meet); the others are taken back to that road.
 */
export function connect(laid: readonly RoadPlan[], fit: Fit, courts = 2): RoadPlan[] {
  // Each lane's dead end at its end.
  let roads = laid.map((r): RoadPlan => (r.ends[0] === 'dead' && r.ends[1] !== 'dead' ? { ...r, points: [...r.points].reverse(), ends: [r.ends[1], r.ends[0]] } : r));
  const lane = (r: RoadPlan) => r.ends[1] === 'dead' && r.kind !== 'trail';
  for (let guard = 0; guard < 40; guard++) {
    const index = new RoadIndex(roads);
    let best: Join | null = null;
    for (const [i, r] of roads.entries()) {
      if (!lane(r)) continue;
      const j = joinFor(roads, index, i, fit);
      if (j && (!best || j.cost < best.cost)) best = j;
    }
    if (!best) break;
    roads = joined(roads, best);
  }
  // The rest: the shortest stay courts, the others go back to the last road they meet.
  for (let guard = 0; guard < 40; guard++) {
    const index = new RoadIndex(roads);
    const dead = roads
      .flatMap((r, i) => {
        if (!lane(r)) return [];
        const from = lastJoin(roads, index, i);
        return [{ i, from, len: lengths(r.points)[r.points.length - 1] - from }];
      })
      .sort((a, b) => a.len - b.len);
    const long = dead.slice(0, courts).find((c) => c.len > COURT + 0.5);
    if (long) {
      const r = roads[long.i];
      roads = roads.map((x, k) => (k === long.i ? { ...r, points: cutAt(r, long.from + COURT) } : x));
      continue;
    }
    const extra = dead[courts];
    if (!extra) break;
    if (extra.from < 0.5) {
      roads = roads.filter((_, k) => k !== extra.i);
      continue;
    }
    roads = turnCorner(roads, extra.i, extra.from);
  }
  return roads;
}

/**
 * Road `i` cut back to `s`, where another road's end meets it, and the two made one road round
 * the corner (rounded off), so neither ends at the other.
 */
function turnCorner(roads: readonly RoadPlan[], i: number, s: number): RoadPlan[] {
  const r = roads[i];
  const corner = pointAt(r.points, s).p;
  const k = roads.findIndex((o, n) => n !== i && [o.points[0], o.points[o.points.length - 1]].some((p) => Math.hypot(p[0] - corner[0], p[1] - corner[1]) < Math.max(o.width, r.width) / 2 + 1.5));
  if (k < 0) return roads.map((x, n) => (n === i ? { ...x, points: cutAt(x, s), ends: [x.ends[0], 'join'] } : x));
  const o = roads[k];
  const first = o.points[0];
  // The other road leading away from the corner.
  const away = Math.hypot(first[0] - corner[0], first[1] - corner[1]) < Math.hypot(o.points[o.points.length - 1][0] - corner[0], o.points[o.points.length - 1][1] - corner[1]);
  const on = away ? o.points : [...o.points].reverse();
  const onEnds: [RoadEnd, RoadEnd] = away ? o.ends : [o.ends[1], o.ends[0]];
  const before = cutAt(r, Math.max(1, s - 8));
  const lsOn = lengths(on);
  const after = on.filter((_, n) => lsOn[n] > 8);
  const a = before[before.length - 1];
  const b = after[0] ?? on[on.length - 1];
  const da = endDir(before);
  const db = unit((after[1] ?? b)[0] - b[0], (after[1] ?? b)[1] - b[1]);
  const round = connector(a, da, b, db);
  const one: RoadPlan = { ...r, width: Math.max(r.width, o.width), points: [...before, ...round.slice(1, -1), ...after], ends: [r.ends[0], onEnds[1]] };
  return roads.flatMap((x, n) => (n === k ? [] : n === i ? [one] : [x]));
}

/**
 * A cross street from road `from` at `p` (leaving along `dir`) to the first road ahead of it,
 * 30 to `reach` m away, if one fits as `connect`'s lanes do; null if not.
 */
export function crossStreet(roads: readonly RoadPlan[], index: RoadIndex, from: number, p: Pt, dir: Pt, fit: Fit, reach = 110, width = LANE_WIDTH): RoadPlan | null {
  let hit: { k: number; road: number; s: number } | null = null;
  for (let k = 8; k <= reach && !hit; k += 2) {
    const q: Pt = [p[0] + dir[0] * k, p[1] + dir[1] * k];
    const near = index.nearest(q[0], q[1], 8);
    if (near && near.dist < roads[near.road].width / 2) hit = { k, road: near.road, s: near.s };
  }
  if (!hit || hit.k < 40 || hit.road === from) return null;
  const to = roads[hit.road];
  if (to.island || to.kind === 'trail') return null;
  const { p: T, t: tq } = pointAt(to.points, hit.s);
  const n = leftOf(tq);
  const sgn = dot([p[0] - T[0], p[1] - T[1]], n) >= 0 ? 1 : -1;
  const a = unit(dir[0] - n[0] * sgn, dir[1] - n[1] * sgn);
  const pts = connector(p, dir, T, a);
  const b = bends(pts);
  if (b.tight || Math.min(b.left, b.right) > 0.3) return null;
  const tp = pointAt(roads[from].points, index.along(from, p[0], p[1])).t;
  if (Math.abs(Math.sin(angleTo(dir, tp))) < Math.sin(MEET) || Math.abs(Math.sin(angleTo(endDir(pts), tq))) < Math.sin(MEET)) return null;
  if (!roomAt(index, p, [from]) || !roomAt(index, T, [hit.road])) return null;
  if (!clearOf(pts, width, roads, index, fit, from, hit.road, STREETS_APART)) return null;
  return { points: pts, width, ends: ['join', 'join'] };
}

// ---- the land the roads take ---------------------------------------------------------------

/**
 * Tiles of a `w` × `d` area (1, else 0) that roads reach: those whose middle is within `reach(road)`
 * of a road's middle (plus `slack`: within it, any of the tile).
 */
export function rasterRoads(roads: readonly RoadPlan[], w: number, d: number, reach: (r: RoadPlan) => number, slack = 0): Uint8Array {
  const mask = new Uint8Array(w * d);
  for (const r of roads) {
    const far = reach(r) + slack;
    for (let i = 0; i + 1 < r.points.length; i++) {
      const [[ax, az], [bx, bz]] = [r.points[i], r.points[i + 1]];
      const [dx, dz] = [bx - ax, bz - az];
      const l2 = dx * dx + dz * dz || 1;
      const [x0, x1] = [Math.max(0, Math.floor(Math.min(ax, bx) - far)), Math.min(w - 1, Math.ceil(Math.max(ax, bx) + far))];
      const [z0, z1] = [Math.max(0, Math.floor(Math.min(az, bz) - far)), Math.min(d - 1, Math.ceil(Math.max(az, bz) + far))];
      for (let z = z0; z <= z1; z++) {
        for (let x = x0; x <= x1; x++) {
          if (mask[z * w + x]) continue;
          const [px, pz] = [x + 0.5, z + 0.5];
          const t = Math.max(0, Math.min(1, ((px - ax) * dx + (pz - az) * dz) / l2));
          if (Math.hypot(px - (ax + dx * t), pz - (az + dz * t)) < far) mask[z * w + x] = 1;
        }
      }
    }
  }
  return mask;
}

/** Tiles of a `w` × `d` area within `clear` m of a road's edge (1), else 0: where no lot may lie. */
export function roadMask(roads: readonly RoadPlan[], w: number, d: number, clear = 0.5): Uint8Array {
  // (Any of the tile: a road's edge through its corner too.)
  return rasterRoads(roads, w, d, (r) => r.width / 2 + clear, 0.7);
}

/** Grid cells (m) roads' segments are filed under, for `RoadIndex`'s questions. */
const CELL = 8;

/**
 * The roads, for questions about points near them: how far a road's middle is, whether a point is
 * on a road, the road nearest it. Each road's segments are filed by grid cell, so a question looks
 * only at those nearby. (One per set of roads: `roadIndex`.)
 */
export class RoadIndex {
  private readonly cells = new Map<number, number[]>();
  /** Per segment (in `cells`): its road, its first point, the arc length there. */
  private readonly segs: { road: number; k: number; s: number }[] = [];
  /** Each road's arc lengths, worked out when first asked (`along`). */
  private readonly lens: (number[] | undefined)[] = [];
  private readonly widest: number;

  constructor(readonly roads: readonly RoadPlan[]) {
    this.widest = Math.max(0, ...roads.map((r) => r.width));
    roads.forEach((r, road) => {
      let s = 0;
      for (let k = 0; k + 1 < r.points.length; k++) {
        const [[ax, az], [bx, bz]] = [r.points[k], r.points[k + 1]];
        const id = this.segs.push({ road, k, s }) - 1;
        for (let cz = Math.floor(Math.min(az, bz) / CELL); cz <= Math.floor(Math.max(az, bz) / CELL); cz++) {
          for (let cx = Math.floor(Math.min(ax, bx) / CELL); cx <= Math.floor(Math.max(ax, bx) / CELL); cx++) {
            const key = cx * 65536 + cz;
            const list = this.cells.get(key);
            if (list) list.push(id);
            else this.cells.set(key, [id]);
          }
        }
        s += Math.hypot(bx - ax, bz - az);
      }
    });
  }

  /**
   * Calls `fn` with each segment filed within `r` m of (x, z), until it returns true (then true).
   * A segment in several cells comes more than once: harmless for a nearest or an any.
   */
  private near(x: number, z: number, r: number, fn: (id: number) => boolean | void): boolean {
    for (let cz = Math.floor((z - r) / CELL); cz <= Math.floor((z + r) / CELL); cz++) {
      for (let cx = Math.floor((x - r) / CELL); cx <= Math.floor((x + r) / CELL); cx++) {
        const list = this.cells.get(cx * 65536 + cz);
        if (list) for (const id of list) if (fn(id)) return true;
      }
    }
    return false;
  }

  /** Distance from (x, z) to segment `id`, the arc length there and the side (+1: the road's left). */
  private measure(id: number, x: number, z: number): [dist: number, s: number, side: 1 | -1] {
    const { road, k, s } = this.segs[id];
    const [[ax, az], [bx, bz]] = [this.roads[road].points[k], this.roads[road].points[k + 1]];
    const [dx, dz] = [bx - ax, bz - az];
    const l = Math.hypot(dx, dz) || 1;
    const t = Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / (l * l)));
    // Left of the way along: (dx, dz) turned a quarter, as `leftOf`.
    return [Math.hypot(x - (ax + dx * t), z - (az + dz * t)), s + t * l, (x - ax) * dz - (z - az) * dx >= 0 ? 1 : -1];
  }

  /** Distance from road `i`'s middle, if it's within `within` m (else Infinity). */
  distance(i: number, x: number, z: number, within = this.roads[i].width / 2 + 2): number {
    let best = Infinity;
    this.near(x, z, within, (id) => {
      if (this.segs[id].road === i) best = Math.min(best, this.measure(id, x, z)[0]);
    });
    return best <= within ? best : Infinity;
  }

  /** Arc length along road `i` nearest (x, z), however far. */
  along(i: number, x: number, z: number): number {
    const pts = this.roads[i].points;
    return nearest(pts, x, z, (this.lens[i] ??= lengths(pts))).s;
  }

  /** Whether (x, z) is on a road other than `skip` (within its width and `margin`). */
  onRoad(x: number, z: number, margin = 0, skip = -1): boolean {
    return this.near(x, z, this.widest / 2 + margin, (id) => {
      const road = this.segs[id].road;
      return road !== skip && this.measure(id, x, z)[0] < this.roads[road].width / 2 + margin;
    });
  }

  /** Each road whose middle passes within `within` m of (x, z): how far, the arc length there and the way it runs there (a unit direction). */
  around(x: number, z: number, within: number): Map<number, { dist: number; s: number; t: Pt }> {
    const out = new Map<number, { dist: number; s: number; t: Pt }>();
    this.near(x, z, within, (id) => {
      const [dist, s] = this.measure(id, x, z);
      const { road, k } = this.segs[id];
      const had = out.get(road);
      if (dist > within || (had && had.dist <= dist)) return;
      const [[ax, az], [bx, bz]] = [this.roads[road].points[k], this.roads[road].points[k + 1]];
      const l = Math.hypot(bx - ax, bz - az) || 1;
      out.set(road, { dist, s, t: [(bx - ax) / l, (bz - az) / l] });
    });
    return out;
  }

  /** The road nearest (x, z) within `within` m, how far its middle is, the arc length there and which side (+1: its left). */
  nearest(x: number, z: number, within = 64): { road: number; dist: number; s: number; side: 1 | -1 } | null {
    let best: { road: number; dist: number; s: number; side: 1 | -1 } | null = null;
    this.near(x, z, within, (id) => {
      const [dist, s, side] = this.measure(id, x, z);
      if (dist <= within && (!best || dist < best.dist)) best = { road: this.segs[id].road, dist, s, side };
    });
    return best;
  }
}

/** Each set of roads' `RoadIndex`, built once. */
const indexes = new WeakMap<readonly RoadPlan[], RoadIndex>();
export function roadIndex(roads: readonly RoadPlan[]): RoadIndex {
  let index = indexes.get(roads);
  if (!index) indexes.set(roads, (index = new RoadIndex(roads)));
  return index;
}

/**
 * Lanes that join a road begin at its kerb, not its middle: a lane's sidewalks then begin at the
 * road's, and it cuts the road's sidewalk only on its own side. (The road it joins: the one whose
 * middle is nearest the lane's end.)
 */
export function trimJoins(roads: readonly RoadPlan[]): RoadPlan[] {
  const all = new RoadIndex(roads);
  return roads.map((r, i) => {
    let pts = r.points;
    for (const e of [0, 1] as const) {
      if (r.ends[e] !== 'join') continue;
      const [ex, ez] = e === 0 ? pts[0] : pts[pts.length - 1];
      let parent = -1;
      let near = Infinity;
      roads.forEach((_, k) => {
        const d = k === i ? Infinity : all.distance(k, ex, ez, 4);
        if (d < near) [parent, near] = [k, d];
      });
      if (parent < 0) continue;
      const kerb = roads[parent].width / 2 - SIDEWALK;
      const keep = ([x, z]: Pt) => all.distance(parent, x, z, kerb + 1) >= kerb;
      const kept = e === 0 ? pts.slice(Math.max(0, pts.findIndex(keep))) : pts.slice(0, pts.length - Math.max(0, [...pts].reverse().findIndex(keep)));
      if (kept.length >= 2) pts = kept;
    }
    return pts === r.points ? r : { ...r, points: pts };
  });
}

/** A polygon's middle (the mean of its points). */
export function centroid(poly: readonly Pt[]): Pt {
  return [poly.reduce((n, p) => n + p[0], 0) / poly.length, poly.reduce((n, p) => n + p[1], 0) / poly.length];
}

/** Whether a point is inside a polygon (even-odd). */
export function inside(poly: readonly Pt[], x: number, z: number): boolean {
  let hit = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const [[xi, zi], [xj, zj]] = [poly[i], poly[j]];
    if (zi > z !== zj > z && x < ((xj - xi) * (z - zi)) / (zj - zi) + xi) hit = !hit;
  }
  return hit;
}

/**
 * Roads cut to the town's rectangle (`w` × `d`, after it moved by `dx`, `dz`): a road's end
 * outside it now ends at its edge (`out`).
 */
export function cropRoads(roads: readonly RoadPlan[], dx: number, dz: number, w: number, d: number): RoadPlan[] {
  const out: RoadPlan[] = [];
  const within = ([x, z]: Pt) => x >= 0 && z >= 0 && x <= w && z <= d;
  for (const r of roads) {
    const pts = r.points.map(([x, z]): Pt => [x + dx, z + dz]);
    const first = pts.findIndex(within);
    if (first < 0) continue;
    let last = first;
    while (last + 1 < pts.length && within(pts[last + 1])) last++;
    const kept = pts.slice(first, last + 1);
    // Where it crosses the edge, exactly on it.
    const edge = (inner: Pt, outer: Pt): Pt => {
      let t = 1;
      for (const [v, lo, hi, dv] of [[inner[0], 0, w, outer[0] - inner[0]], [inner[1], 0, d, outer[1] - inner[1]]] as const) {
        if (v + dv * t < lo) t = (lo - v) / dv;
        if (v + dv * t > hi) t = (hi - v) / dv;
      }
      return [inner[0] + (outer[0] - inner[0]) * t, inner[1] + (outer[1] - inner[1]) * t];
    };
    const ends: [RoadEnd, RoadEnd] = [...r.ends];
    if (first > 0) {
      kept.unshift(edge(pts[first], pts[first - 1]));
      ends[0] = 'out';
    }
    if (last < pts.length - 1) {
      kept.push(edge(pts[last], pts[last + 1]));
      ends[1] = 'out';
    }
    // (A ring cut open has no middle left to stand anything in.)
    const { island: _, ...open } = r;
    if (kept.length >= 2) out.push({ ...(ends[0] === r.ends[0] && ends[1] === r.ends[1] ? r : open), points: kept.map(([x, z]): Pt => [Math.round(x * 100) / 100, Math.round(z * 100) / 100]), ends });
  }
  return out;
}

/** Where the roads lead out of town (their `out` ends), and the way out there (a unit direction). */
export function roadEnds(roads: readonly RoadPlan[]): { x: number; z: number; dx: number; dz: number }[] {
  return roads.flatMap((r) =>
    ([0, 1] as const)
      .filter((e) => r.ends[e] === 'out')
      .map((e) => {
        const pts = r.points;
        const [end, before] = e === 0 ? [pts[0], pts[1]] : [pts[pts.length - 1], pts[pts.length - 2]];
        const l = Math.hypot(end[0] - before[0], end[1] - before[1]) || 1;
        return { x: end[0], z: end[1], dx: (end[0] - before[0]) / l, dz: (end[1] - before[1]) / l };
      }),
  );
}

/** Where residents leave town: the roads' `out` ends, a little inside the town's edge. */
export function roadExits(roads: readonly RoadPlan[], w: number, d: number): Pt[] {
  return roadEnds(roads).map(({ x, z }): Pt => [Math.max(0.5, Math.min(w - 0.5, x)), Math.max(0.5, Math.min(d - 0.5, z))]);
}
