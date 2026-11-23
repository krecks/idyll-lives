/**
 * One-way roads and whether cars still get everywhere (`stronglyConnected`): the roads joined into a network as the
 * simulation joins them (sim-core `travel/graph` `Graph::build`: each road's ends, where another's end meets it part
 * way along, where two cross), each one-way road's stretches gone along its way only. From every place a car can come
 * onto the roads (a node on a road for cars but the motorway and its ramps: `Road::enterable`) it can drive to every
 * other place in its network, and back: no home cut off, none a car comes to and can't leave
 * (`Graph::strongly_connected`). The generator gives a few pairs of streets in a town's or a city's dense middle
 * opposite ways (`oneWayPairs`), each pair only if that still holds; the town's check (`townCheck.ts`) and the
 * editor ask the same.
 */

import type { RoadKind, RoadPlan } from '../core/protocol';
import { ROAD_KIND } from '../core/roadKinds';
import * as pmath from './pmath';
import { lengths, nearest, pointAt, type Pt } from './roads';

/** A road's stretch between two nodes (`RoadGraph.edges`): road `road`, from node `a` (its lower arc length) to `b`. */
export interface GraphEdge {
  road: number;
  a: number;
  b: number;
}

/** The roads' network as the simulation makes it (`Graph::build`), but for its merging of near junctions (which only joins more). */
export interface RoadGraph {
  nodes: Pt[];
  edges: GraphEdge[];
  /** Per node, the stretches meeting it. */
  adj: number[][];
  /** Per node, whether a car comes onto the roads there (a road for cars but a motorway or a ramp meets it). */
  enterable: boolean[];
  /** Per node, whether a road leads out of town there (an `out` end of a road for cars). */
  out: boolean[];
}

/** Splits along a road this near (m) are one node, as the simulation's. */
const SAME = 0.5;
/** Two roads' middle lines this near (m) away from their ends meet there (sim-core `roads.rs` `CROSS_NEAR`). */
const CROSS_NEAR = 2;
/** A run of near places no longer than this (m) is one crossing; longer, two (`CROSS_RUN`). */
const CROSS_RUN = 20;
/** A gap this long (m along the first road) between near places is two runs (`CROSS_GAP`). */
const CROSS_GAP = 4;
/** Cells (m) of the segment grids `crossings` looks through. */
const CELL = 8;

const drivable = (kind: RoadKind) => kind !== 'trail';
const enterableKind = (kind: RoadKind) => drivable(kind) && kind !== 'motorway' && kind !== 'ramp';
const kindOf = (r: RoadPlan): RoadKind => r.kind ?? 'street';
const dist = (a: Pt, b: Pt) => pmath.hypot(a[0] - b[0], a[1] - b[1]);

/** Whether an end of a road of kind `end` meets a road of kind `along` part way along it (sim-core `level_join`): only a ramp or a carriageway the motorway. */
function levelJoin(end: RoadKind, along: RoadKind): boolean {
  if ((end === 'ramp' || end === 'motorway') && along === 'motorway') return true;
  return end !== 'motorway' && along !== 'motorway';
}

/** A road as the graph sees it: its points (repeats dropped), their arc lengths, its box. */
interface Line {
  src: number;
  pts: Pt[];
  cum: number[];
  box: [number, number, number, number];
  width: number;
  kind: RoadKind;
}

function lineOf(r: RoadPlan, src: number): Line | null {
  const pts: Pt[] = [];
  for (const p of r.points) {
    if (!Number.isFinite(p[0]) || !Number.isFinite(p[1])) continue;
    const q = pts[pts.length - 1];
    if (q && dist(q, p) < 1e-3) continue;
    pts.push(p);
  }
  if (pts.length < 2) return null;
  const cum = lengths(pts);
  const box: [number, number, number, number] = [Infinity, Infinity, -Infinity, -Infinity];
  for (const [x, z] of pts) [box[0], box[1], box[2], box[3]] = [Math.min(box[0], x), Math.min(box[1], z), Math.max(box[2], x), Math.max(box[3], z)];
  return { src, pts, cum, box, width: Number.isFinite(r.width) && r.width > 0 ? r.width : 7, kind: kindOf(r) };
}

/** The nearest place on segment `i` of `l` to `p`: its arc length, how far off. */
function projectSegment(l: Line, i: number, p: Pt): [number, number] {
  const [a, b] = [l.pts[i], l.pts[i + 1]];
  const [dx, dz] = [b[0] - a[0], b[1] - a[1]];
  const len2 = Math.max(dx * dx + dz * dz, 1e-9);
  const t = Math.min(1, Math.max(0, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dz) / len2));
  return [l.cum[i] + t * Math.sqrt(len2), dist(p, [a[0] + dx * t, a[1] + dz * t])];
}

function project(l: Line, p: Pt): [number, number] {
  let best: [number, number] = [0, Infinity];
  for (let i = 0; i + 1 < l.pts.length; i++) {
    const got = projectSegment(l, i, p);
    if (got[1] < best[1]) best = got;
  }
  return best;
}

/** The point `s` m along `l`. */
function at(l: Line, s: number): Pt {
  const L = l.cum[l.cum.length - 1];
  const c = Math.min(L, Math.max(0, s));
  let i = 1;
  while (i < l.pts.length - 1 && l.cum[i] <= c) i++;
  const span = Math.max(l.cum[i] - l.cum[i - 1], 1e-6);
  const t = Math.min(1, Math.max(0, (c - l.cum[i - 1]) / span));
  return [l.pts[i - 1][0] + (l.pts[i][0] - l.pts[i - 1][0]) * t, l.pts[i - 1][1] + (l.pts[i][1] - l.pts[i - 1][1]) * t];
}

/** The nearest points of segments a0–a1 and b0–b1: how far along each (0..1), how far apart (sim-core `segments_nearest`). */
function segmentsNearest(a0: Pt, a1: Pt, b0: Pt, b1: Pt): [number, number, number] {
  const u: Pt = [a1[0] - a0[0], a1[1] - a0[1]];
  const v: Pt = [b1[0] - b0[0], b1[1] - b0[1]];
  const w0: Pt = [a0[0] - b0[0], a0[1] - b0[1]];
  const dot = (p: Pt, q: Pt) => p[0] * q[0] + p[1] * q[1];
  const [aa, bb, cc, dd, ee] = [dot(u, u), dot(u, v), dot(v, v), dot(u, w0), dot(v, w0)];
  const den = aa * cc - bb * bb;
  let s = den > 1e-9 ? Math.min(1, Math.max(0, (bb * ee - cc * dd) / den)) : 0;
  let t = cc > 1e-9 ? Math.min(1, Math.max(0, (bb * s + ee) / cc)) : 0;
  if (aa > 1e-9) s = Math.min(1, Math.max(0, (bb * t - dd) / aa));
  if (cc > 1e-9) t = Math.min(1, Math.max(0, (bb * s + ee) / cc));
  const p: Pt = [a0[0] + u[0] * s, a0[1] + u[1] * s];
  const q: Pt = [b0[0] + v[0] * t, b0[1] + v[1] * t];
  return [s, t, dist(p, q)];
}

/** Where two roads for cars cross or touch away from their ends, neither a motorway (sim-core `crossings`): the two lines, how far along each, the place. */
function crossings(lines: readonly Line[]): [number, number, number, number, Pt][] {
  const out: [number, number, number, number, Pt][] = [];
  // (Each line's segments by grid cell, made when first asked.)
  const grids = new Map<number, Map<number, number[]>>();
  const gridOf = (q: number) => {
    let g = grids.get(q);
    if (g) return g;
    g = new Map();
    const { pts } = lines[q];
    for (let j = 0; j + 1 < pts.length; j++) {
      const [a, b] = [pts[j], pts[j + 1]];
      for (let cz = Math.floor((Math.min(a[1], b[1]) - CROSS_NEAR) / CELL); cz <= Math.floor((Math.max(a[1], b[1]) + CROSS_NEAR) / CELL); cz++) {
        for (let cx = Math.floor((Math.min(a[0], b[0]) - CROSS_NEAR) / CELL); cx <= Math.floor((Math.max(a[0], b[0]) + CROSS_NEAR) / CELL); cx++) {
          const key = cx * 65536 + cz;
          const list = g.get(key);
          if (list) list.push(j);
          else g.set(key, [j]);
        }
      }
    }
    grids.set(q, g);
    return g;
  };
  for (let r = 0; r < lines.length; r++) {
    for (let q = r + 1; q < lines.length; q++) {
      const [a, b] = [lines[r], lines[q]];
      const [x, y] = [a.box, b.box];
      if (!drivable(a.kind) || !drivable(b.kind) || a.kind === 'motorway' || b.kind === 'motorway') continue;
      if (x[2] + CROSS_NEAR < y[0] || y[2] + CROSS_NEAR < x[0] || x[3] + CROSS_NEAR < y[1] || y[3] + CROSS_NEAR < x[1]) continue;
      const snap = Math.max(a.width, b.width) / 2 + 1;
      const inside = (l: Line, s: number) => s >= snap && s <= l.cum[l.cum.length - 1] - snap;
      const grid = gridOf(q);
      const near: [number, number, number, Pt][] = [];
      // (Each of `b`'s segments once per segment of `a`: `seen` holds the last `i` it was looked at for.)
      const seen = new Int32Array(b.pts.length).fill(-1);
      for (let i = 0; i + 1 < a.pts.length; i++) {
        const [p0, p1] = [a.pts[i], a.pts[i + 1]];
        if (Math.max(p0[0], p1[0]) < y[0] - CROSS_NEAR || Math.min(p0[0], p1[0]) > y[2] + CROSS_NEAR || Math.max(p0[1], p1[1]) < y[1] - CROSS_NEAR || Math.min(p0[1], p1[1]) > y[3] + CROSS_NEAR) continue;
        const js: number[] = [];
        for (let cz = Math.floor(Math.min(p0[1], p1[1]) / CELL); cz <= Math.floor(Math.max(p0[1], p1[1]) / CELL); cz++) {
          for (let cx = Math.floor(Math.min(p0[0], p1[0]) / CELL); cx <= Math.floor(Math.max(p0[0], p1[0]) / CELL); cx++) {
            for (const j of grid.get(cx * 65536 + cz) ?? []) {
              if (seen[j] === i) continue;
              seen[j] = i;
              js.push(j);
            }
          }
        }
        for (const j of js.sort((m, n) => m - n)) {
          const [ta, tb, d] = segmentsNearest(p0, p1, b.pts[j], b.pts[j + 1]);
          if (d > CROSS_NEAR) continue;
          const sa = a.cum[i] + (a.cum[i + 1] - a.cum[i]) * ta;
          const sb = b.cum[j] + (b.cum[j + 1] - b.cum[j]) * tb;
          if (!inside(a, sa) || !inside(b, sb)) continue;
          const [pa, pb] = [at(a, sa), at(b, sb)];
          near.push([d, sa, sb, [(pa[0] + pb[0]) / 2, (pa[1] + pb[1]) / 2]]);
        }
      }
      near.sort((m, n) => m[1] - n[1] || m[0] - n[0]);
      for (let start = 0; start < near.length; ) {
        let end = start + 1;
        while (end < near.length && near[end][1] - near[end - 1][1] < CROSS_GAP) end++;
        const run = near.slice(start, end);
        const least = Math.min(...run.map((m) => m[0]));
        if (run[run.length - 1][1] - run[0][1] <= CROSS_RUN) {
          const m = run.find((m) => m[0] === least)!;
          out.push([r, q, m[1], m[2], m[3]]);
        } else {
          const met = (m: (typeof run)[number]) => m[0] <= least + 0.3;
          const [first, last] = [run.find(met)!, [...run].reverse().find(met)!];
          out.push([r, q, first[1], first[2], first[3]], [r, q, last[1], last[2], last[3]]);
        }
        start = end;
      }
    }
  }
  return out;
}

/** The roads' network as the simulation joins them (sim-core `Graph::build`): nodes at their ends, where an end meets another road, where two cross. */
export function roadGraph(roads: readonly RoadPlan[]): RoadGraph {
  const lines = roads.flatMap((r, i) => lineOf(r, i) ?? []);
  const parent: number[] = [];
  const pos: Pt[] = [];
  const splits: [number, number][][] = lines.map(() => []);
  const ends: [number, number][] = [];
  for (const [k, l] of lines.entries()) {
    const a = parent.push(parent.length) - 1;
    pos.push(l.pts[0]);
    const last = l.pts[l.pts.length - 1];
    let b = a;
    if (dist(l.pts[0], last) >= 0.5) {
      b = parent.push(parent.length) - 1;
      pos.push(last);
    }
    splits[k].push([0, a], [l.cum[l.cum.length - 1], b]);
    ends.push([a, b]);
  }
  const find = (i: number): number => {
    while (parent[i] !== i) i = parent[i] = parent[parent[i]];
    return i;
  };
  const union = (a: number, b: number) => {
    const [ra, rb] = [find(a), find(b)];
    if (ra !== rb) parent[Math.max(ra, rb)] = Math.min(ra, rb);
  };
  for (const [r, l] of lines.entries()) {
    for (const [k, end] of ends[r].entries()) {
      const p = k === 0 ? l.pts[0] : l.pts[l.pts.length - 1];
      for (const [q, other] of lines.entries()) {
        if (q === r) continue;
        const snap = Math.max(other.width, l.width) / 2 + 1;
        const bx = other.box;
        if (p[0] < bx[0] - snap || p[0] > bx[2] + snap || p[1] < bx[1] - snap || p[1] > bx[3] + snap) continue;
        const [s, d] = project(other, p);
        if (d > snap) continue;
        const L = other.cum[other.cum.length - 1];
        const nearEnd = s < snap || s > L - snap;
        if (!nearEnd && !levelJoin(l.kind, other.kind)) continue;
        if (nearEnd) union(end, s < L / 2 ? ends[q][0] : ends[q][1]);
        else splits[q].push([s, end]);
      }
    }
  }
  for (const [r, q, sr, sq, p] of crossings(lines)) {
    const n = parent.push(parent.length) - 1;
    pos.push(p);
    splits[r].push([sr, n]);
    splits[q].push([sq, n]);
  }
  for (const list of splits) {
    list.sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    for (let i = 1; i < list.length; i++) if (list[i][0] - list[i - 1][0] < SAME) union(list[i][1], list[i - 1][1]);
  }
  const id = new Array<number>(parent.length).fill(-1);
  const nodes: Pt[] = [];
  for (let i = 0; i < parent.length; i++) {
    const root = find(i);
    if (id[root] < 0) id[root] = nodes.push(pos[root]) - 1;
    id[i] = id[root];
  }
  const edges: GraphEdge[] = [];
  const adj: number[][] = nodes.map(() => []);
  for (const [r, list] of splits.entries()) {
    let prev: [number, number] | null = null;
    for (const [s, n0] of list) {
      const n = id[n0];
      if (prev) {
        if (s - prev[0] < SAME) continue;
        const e = edges.push({ road: lines[r].src, a: prev[1], b: n }) - 1;
        adj[prev[1]].push(e);
        if (prev[1] !== n) adj[n].push(e);
      }
      prev = [s, n];
    }
  }
  const enterable = adj.map((list) => list.some((e) => enterableKind(kindOf(roads[edges[e].road]))));
  const out = nodes.map(() => false);
  for (const [r, l] of lines.entries()) {
    const road = roads[l.src];
    if (!drivable(l.kind)) continue;
    for (const k of [0, 1] as const) if (road.ends[k] === 'out') out[id[ends[r][k]]] = true;
  }
  return { nodes, edges, adj, enterable, out };
}

/** Whether a car may drive along stretch `e` from its node `from` (`RoadPlan.oneway`; a stretch from a node back to itself goes the way the road's points run). */
function mayGo(roads: readonly RoadPlan[], g: RoadGraph, e: number, from: number): boolean {
  const way = roads[g.edges[e].road].oneway;
  return !way || (g.edges[e].a === from) === (way > 0);
}

/** The nodes reached from `start` along roads for cars: either way (`any`), the way cars go, or (`back`) the way back against it. */
function reach(roads: readonly RoadPlan[], g: RoadGraph, start: number, any: boolean, back: boolean): boolean[] {
  const seen = g.nodes.map(() => false);
  seen[start] = true;
  const stack = [start];
  while (stack.length) {
    const u = stack.pop()!;
    for (const e of g.adj[u]) {
      const edge = g.edges[e];
      if (!drivable(kindOf(roads[edge.road]))) continue;
      const v = edge.a === u ? edge.b : edge.a;
      if (!seen[v] && (any || mayGo(roads, g, e, back ? v : u))) {
        seen[v] = true;
        stack.push(v);
      }
    }
  }
  return seen;
}

/**
 * The roads (indices) where a car can come onto the roads and not drive from there to the way out of town and back
 * (in each network, to and from its first way out; one with none, its first place a car comes on): what one-way roads
 * cut off. None: `stronglyConnected`.
 */
export function cutOff(roads: readonly RoadPlan[], g: RoadGraph = roadGraph(roads)): number[] {
  const bad = new Set<number>();
  const done = g.nodes.map(() => false);
  for (let start = 0; start < g.nodes.length; start++) {
    if (done[start] || !g.enterable[start]) continue;
    const network = reach(roads, g, start, true, false);
    const members = network.flatMap((on, v) => (on ? [v] : []));
    for (const v of members) done[v] = true;
    const from = members.find((v) => g.out[v] && g.enterable[v]) ?? start;
    const [ahead, back] = [reach(roads, g, from, false, false), reach(roads, g, from, false, true)];
    for (const v of members) {
      if (!g.enterable[v] || (ahead[v] && back[v])) continue;
      for (const e of g.adj[v]) if (enterableKind(kindOf(roads[g.edges[e].road]))) bad.add(g.edges[e].road);
    }
  }
  return [...bad].sort((a, b) => a - b);
}

/**
 * Whether, in each network of roads for cars, every place a car can come onto the roads (a node on a road for cars
 * but the motorway and its ramps) can be driven to from every other, one-way roads gone along their way only (sim-core
 * `Graph::strongly_connected`). True without any one-way road.
 */
export function stronglyConnected(roads: readonly RoadPlan[], g?: RoadGraph): boolean {
  if (!roads.some((r) => r.oneway)) return true;
  return cutOff(roads, g ?? roadGraph(roads)).length === 0;
}

/** One-way pairs: their streets this far apart (m, middle to middle) along where they run side by side, … */
const PAIR_APART: [number, number] = [30, 80];
/** … side by side for this share of the shorter at least, and this long (m), … */
const PAIR_ALONG = 0.6;
const PAIR_LEAST = 40;
/** … within this of each other's way (radians), … */
const PAIR_TURN = (25 * Math.PI) / 180;
/** … both reaching within this (m) of the town's dense middle (its high street's), … */
const PAIR_REACH = 120;
/** … neither shorter than this (m). */
const PAIR_SHORTEST = 50;

/** Whether road `r` may be made one-way: a town's street or lane, both its ends joined (no dead end, no way out, not a ring or a roundabout), on land (no bridge), long enough. */
function oneWayable(r: RoadPlan): boolean {
  const kind = kindOf(r);
  if (kind !== 'street' && kind !== 'lane') return false;
  if (r.ends[0] !== 'join' || r.ends[1] !== 'join' || r.island || r.bridge || r.bridges?.length || r.oneway) return false;
  if (dist(r.points[0], r.points[r.points.length - 1]) < 0.5) return false;
  return lengths(r.points).at(-1)! >= PAIR_SHORTEST;
}

/** How roads `a` and `b` run side by side: the length (m along `a`) where they are `PAIR_APART` apart and run the same way or opposite (`same`: the same way, by the most). */
function alongside(a: RoadPlan, b: RoadPlan): { along: number; same: boolean } {
  const ls = lengths(a.points);
  const L = ls[ls.length - 1];
  const lb = lengths(b.points);
  let [along, dots] = [0, 0];
  const step = 4;
  for (let s = 0; s < L; s += step) {
    const { p, t } = pointAt(a.points, s, ls);
    const near = nearest(b.points, p[0], p[1], lb);
    if (near.dist < PAIR_APART[0] || near.dist > PAIR_APART[1]) continue;
    const tb = pointAt(b.points, near.s, lb).t;
    const dot = t[0] * tb[0] + t[1] * tb[1];
    if (Math.abs(dot) < pmath.cos(PAIR_TURN)) continue;
    along += step;
    dots += dot;
  }
  return { along, same: dots >= 0 };
}

/**
 * A few pairs of streets in a town's dense middle (round `middle`) made one-way, each the other's way back: of the
 * roads `own` (the main town's), streets and lanes both of whose ends join another road (`oneWayable`), side by side
 * `PAIR_APART` m apart for most of the shorter, nearest the middle first; each pair (a lane of it made a street, as
 * wide) only if every place on the roads can still be driven to and from every other (`stronglyConnected`), first
 * with the first of the two one-way along its points, else the other way round. `most` pairs at most. The roads with the
 * pairs made (the same array if none), and the pairs (road indices).
 */
export function oneWayPairs(roads: readonly RoadPlan[], own: readonly number[], middle: Pt, most: number): { roads: RoadPlan[]; pairs: [number, number][] } {
  const pairs: [number, number][] = [];
  let out = roads as RoadPlan[];
  if (most <= 0) return { roads: out, pairs };
  const near = (r: RoadPlan) => nearest(r.points, middle[0], middle[1]).dist;
  const ok = own.filter((i) => oneWayable(roads[i]) && near(roads[i]) <= PAIR_REACH);
  const cands: { a: number; b: number; same: boolean; score: number }[] = [];
  for (const [k, a] of ok.entries()) {
    for (const b of ok.slice(k + 1)) {
      const [ra, rb] = [roads[a], roads[b]];
      const shorter = Math.min(lengths(ra.points).at(-1)!, lengths(rb.points).at(-1)!);
      const { along, same } = alongside(ra, rb);
      if (along < Math.max(PAIR_LEAST, shorter * PAIR_ALONG)) continue;
      cands.push({ a, b, same, score: near(ra) + near(rb) });
    }
  }
  cands.sort((x, y) => x.score - y.score || x.a - y.a || x.b - y.b);
  const used = new Set<number>();
  for (const c of cands) {
    if (pairs.length >= most) break;
    if (used.has(c.a) || used.has(c.b)) continue;
    const street = (r: RoadPlan): RoadPlan => ({ ...r, kind: 'street', width: Math.max(r.width, ROAD_KIND.street.width) });
    for (const way of [1, -1] as const) {
      const tried = out.map((r, i): RoadPlan => (i === c.a ? { ...street(r), oneway: way } : i === c.b ? { ...street(r), oneway: (c.same ? -way : way) as 1 | -1 } : r));
      if (!stronglyConnected(tried)) continue;
      out = tried;
      pairs.push([c.a, c.b]);
      used.add(c.a).add(c.b);
      break;
    }
  }
  return { roads: out, pairs };
}
