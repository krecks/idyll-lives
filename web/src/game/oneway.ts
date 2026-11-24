/**
 * Whether one-way roads leave every car road reachable: from anywhere a car drives, a car can get anywhere else and
 * back. The generator's one-way pairs and the editor's one-way streets are kept only where this holds.
 *
 * The roads as a car sees them: a stop at each road's ends and wherever another road's end meets it (as the sim joins
 * them: an end within half the wider road's width and a metre of the other's middle; a motorway only by its ramps'
 * ends; and where two cross on the level, not over or under each other: `crosses`, a deck), a stretch between each
 * two stops, driven both ways or, on a one-way road (`onewayOf`), its one way. A ring's ends are one stop. Every way
 * out of the world is one place, the world beyond it: a car leaves by one and comes back by another (the motorway's
 * carriageways only reach the town so).
 *
 * The generator gives a few pairs of streets in a town's or a city's dense middle opposite ways (`oneWayPairs`), each
 * pair only if this still holds; the town's check (`townCheck.ts`), the editor's Street tool and its problems ask the
 * same.
 */

import type { RoadPlan } from '../core/protocol';
import { isSeparated, kindInfo, onewayOf, ROAD_KIND } from '../core/roadKinds';
import * as pmath from './pmath';
import { deckSpans, inSpans, lengths, nearest, passesOver, pointAt, RoadIndex, type Pt } from './roads';

/** Two car roads cross on the level where a point of one comes this near the other's middle (m; their points about a metre apart). */
const CROSS = 0.75;

/** Whether from every car road a car can reach every other and come back (see the top of this file; `within`, as `cutOffRoads`). */
export function stronglyConnected(roads: readonly RoadPlan[], within = false): boolean {
  return cutOffRoads(roads, { within }).length === 0;
}

/** What `cutOffRoads` asks: `within`, by the roads alone (see there); `index`, the roads' own (`RoadIndex`) if made already. */
export interface CutOffAsk {
  within?: boolean;
  index?: RoadIndex;
}

/**
 * The car roads (indices) a car can't both reach from the world beyond its ways out and get back from (with no way
 * out, from the first car road): some stretch of each one-way-only, or off on its own. `within`: by the roads alone,
 * no way round through the world beyond (a trip in town keeps to its roads: the simulation's own check, sim-core
 * `Graph::strongly_connected`), from the first road a car comes onto the roads by; the motorway and its ramps not
 * asked about (no car comes onto the roads there), only driven along.
 */
export function cutOffRoads(roads: readonly RoadPlan[], { within = false, index }: CutOffAsk = {}): number[] {
  const net = networkOf(roads, within, index);
  return net ? cutOff(net, onewayOf) : [];
}

/** What `cutOffRoads` gives as if every road were driven both ways: what is cut off by no one-way road. */
export function cutOffAnyWay(roads: readonly RoadPlan[], { within = false, index }: CutOffAsk = {}): number[] {
  const net = networkOf(roads, within, index);
  return net ? cutOff(net, () => 0) : [];
}

/** What the one-way roads alone cut off (`cutOffRoads`, less what is cut off whichever way the roads go: `cutOffAnyWay`), the roads as a car sees them made once. */
export function oneWayCut(roads: readonly RoadPlan[], { within = false, index }: CutOffAsk = {}): number[] {
  const net = networkOf(roads, within, index);
  if (!net) return [];
  const anyway = new Set(cutOff(net, () => 0));
  return cutOff(net, onewayOf).filter((i) => !anyway.has(i));
}

/** What `after` cuts off (`cutOffRoads`) that `before` doesn't (its cut off given, else worked out): by index, as `after` numbers its roads. */
export function newlyCutOff(before: readonly RoadPlan[] | ReadonlySet<number>, after: readonly RoadPlan[], { within = false }: Pick<CutOffAsk, 'within'> = {}): number[] {
  const had = 'has' in before ? before : new Set(cutOffRoads(before, { within }));
  return cutOffRoads(after, { within }).filter((i) => !had.has(i));
}

/**
 * The roads (indices, with where each comes nearest) end `e` of road `i` meets as the simulation joins them: an end
 * within half the wider road's width and a metre of the other's middle. Looked for within `reach` m (`index` over
 * `roads`).
 */
export function endMeets(roads: readonly RoadPlan[], index: RoadIndex, i: number, e: 0 | 1, reach: number): [number, { dist: number; s: number }][] {
  const r = roads[i];
  const [x, z] = e ? r.points[r.points.length - 1] : r.points[0];
  return [...index.around(x, z, reach)].filter(([j, m]) => j !== i && m.dist <= Math.max(r.width, roads[j].width) / 2 + 1);
}

/** The roads as a car sees them (see the top of this file), before which way each is driven: the car roads, each one's stops and the stops made one place. */
interface Network {
  roads: readonly RoadPlan[];
  within: boolean;
  /** The car roads (indices), and per road its stops (arc lengths) and their ids (`find`: the place each is; 0, the world beyond). */
  cars: number[];
  stops: number[][];
  ids: number[][];
  find: (id: number) => number;
}

/** `roads` as a car sees them (`Network`; `within`: no way out joined to the world beyond, as `cutOffRoads`; `given`, their index if made); null with no car road. */
function networkOf(roads: readonly RoadPlan[], within: boolean, given?: RoadIndex): Network | null {
  const cars = roads.flatMap((r, i) => (kindInfo(r).lanesEachWay > 0 && r.points.length >= 2 ? [i] : []));
  if (!cars.length) return null;
  const isCar = new Set(cars);
  const index = given ?? new RoadIndex(roads);
  const widest = Math.max(0, ...cars.map((i) => roads[i].width));
  const lens = roads.map((r, i) => (isCar.has(i) ? lengths(r.points) : []));
  const decks = roads.map((r, i) => (isCar.has(i) ? deckSpans(r) : []));
  // Each road's stops (arc lengths), and the stops one place (union-find over `node` ids). Node 0: the world beyond.
  const stops: number[][] = roads.map(() => []);
  const ids: number[][] = roads.map(() => []);
  const parent = [0];
  const find = (a: number): number => (parent[a] === a ? a : (parent[a] = find(parent[a])));
  const join = (a: number, b: number) => void (parent[find(a)] = find(b));
  const stopAt = (i: number, s: number): number => {
    const list = stops[i];
    const k = list.findIndex((q) => Math.abs(q - s) < 0.5);
    if (k >= 0) return ids[i][k];
    list.push(s);
    ids[i].push(parent.push(parent.length) - 1);
    return ids[i][ids[i].length - 1];
  };
  // (A motorway meets only its ramps: what passes under or beside it joins it nowhere.)
  const meets = (a: RoadPlan, b: RoadPlan) => (a.kind === 'motorway' ? b.kind === 'ramp' : b.kind === 'motorway' ? a.kind === 'ramp' : true);
  for (const i of cars) {
    const r = roads[i];
    const L = lens[i][lens[i].length - 1];
    const ends = [stopAt(i, 0), stopAt(i, L)];
    if (r.ends[0] === 'loop' || r.ends[1] === 'loop') join(ends[0], ends[1]);
    for (const e of [0, 1] as const) {
      if (r.ends[e] === 'out' && !within) join(ends[e], 0);
      const [x, z] = e ? r.points[r.points.length - 1] : r.points[0];
      for (const [j] of endMeets(roads, index, i, e, widest / 2 + 1)) {
        if (!isCar.has(j) || !meets(r, roads[j])) continue;
        join(ends[e], stopAt(j, nearest(roads[j].points, x, z, lens[j]).s));
      }
    }
    // (Where it crosses another car road on the level: a crossroads, at the point of it nearest the other's middle;
    // not where either passes over or under the other, `crosses`, or is on a deck.)
    let run: { j: number; s: number; t: number; dist: number } | null = null;
    const close = () => {
      if (run) join(stopAt(i, run.s), stopAt(run.j, run.t));
      run = null;
    };
    r.points.forEach(([x, z], k) => {
      let hit: { j: number; s: number; t: number; dist: number } | null = null;
      for (const [j, m] of index.around(x, z, CROSS)) {
        if (j <= i || !isCar.has(j) || m.dist > CROSS || !meets(r, roads[j]) || passesOver(roads, i, j) || inSpans(decks[i], lens[i][k]) || inSpans(decks[j], m.s)) continue;
        if (!hit || m.dist < hit.dist) hit = { j, s: lens[i][k], t: m.s, dist: m.dist };
      }
      if (run && (!hit || hit.j !== run.j)) close();
      if (hit && (!run || hit.dist < run.dist)) run = hit;
    });
    close();
  }
  return { roads, within, cars, stops, ids, find };
}

/** What network `net` cuts off (`cutOffRoads`), each car road driven the way `way` says (+1 along its points, -1 against them, 0 both). */
function cutOff(net: Network, way: (r: RoadPlan) => 1 | -1 | 0): number[] {
  const { roads, within, cars, stops, ids, find } = net;
  // The stretches between each road's stops, each way it is driven.
  const next = new Map<number, number[]>();
  const back = new Map<number, number[]>();
  const edge = (a: number, b: number) => {
    [a, b] = [find(a), find(b)];
    if (a === b) return;
    (next.get(a) ?? next.set(a, []).get(a)!).push(b);
    (back.get(b) ?? back.set(b, []).get(b)!).push(a);
  };
  for (const i of cars) {
    const w = way(roads[i]);
    const order = stops[i].map((s, k) => [s, ids[i][k]] as const).sort((a, b) => a[0] - b[0]);
    for (let k = 0; k + 1 < order.length; k++) {
      if (w >= 0) edge(order[k][1], order[k + 1][1]);
      if (w <= 0) edge(order[k + 1][1], order[k][1]);
    }
  }
  // (Within: the motorway and its ramps aren't asked about.)
  const asked = within ? cars.filter((i) => !isSeparated(roads[i].kind)) : cars;
  if (!asked.length) return [];
  const out = !within && cars.some((i) => roads[i].ends.includes('out'));
  const root = find(out ? 0 : ids[asked[0]][0]);
  const reach = (from: number, ways: Map<number, number[]>) => {
    const seen = new Set([from]);
    const todo = [from];
    while (todo.length) for (const b of ways.get(todo.pop()!) ?? []) if (!seen.has(b)) (seen.add(b), todo.push(b));
    return seen;
  };
  const [ahead, behind] = [reach(root, next), reach(root, back)];
  return asked.filter((i) => ids[i].some((id) => !ahead.has(find(id)) || !behind.has(find(id))));
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
  const kind = r.kind ?? 'street';
  if (kind !== 'street' && kind !== 'lane') return false;
  if (r.ends[0] !== 'join' || r.ends[1] !== 'join' || r.island || r.bridge || r.bridges?.length || onewayOf(r)) return false;
  const [a, b] = [r.points[0], r.points[r.points.length - 1]];
  if (pmath.hypot(a[0] - b[0], a[1] - b[1]) < 0.5) return false;
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
 * wide) only if it cuts nothing off (`cutOffRoads` `within`: every car road still driven to and back by the roads alone), first
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
  // (By the town's roads alone, as the simulation routes a trip in town: `within`. The roads as a car sees them hang
  // on their widths alone, not on which way each is driven: made once for each set of widths tried.)
  const nets = new Map<string, Network | null>();
  const cutOffIn = (tried: readonly RoadPlan[]) => {
    const key = tried.map((r) => r.width).join();
    let net = nets.get(key);
    if (net === undefined) nets.set(key, (net = networkOf(tried, true)));
    return net ? cutOff({ ...net, roads: tried }, onewayOf) : [];
  };
  // (What is cut off already, whatever the pairs: nothing, in a town of one network.)
  const had = new Set(cutOffIn(roads));
  for (const c of cands) {
    if (pairs.length >= most) break;
    if (used.has(c.a) || used.has(c.b)) continue;
    const street = (r: RoadPlan): RoadPlan => ({ ...r, kind: 'street', width: Math.max(r.width, ROAD_KIND.street.width) });
    for (const way of [1, -1] as const) {
      const tried = out.map((r, i): RoadPlan => (i === c.a ? { ...street(r), oneway: way } : i === c.b ? { ...street(r), oneway: (c.same ? -way : way) as 1 | -1 } : r));
      if (cutOffIn(tried).some((i) => !had.has(i))) continue;
      out = tried;
      pairs.push([c.a, c.b]);
      used.add(c.a).add(c.b);
      break;
    }
  }
  return { roads: out, pairs };
}
