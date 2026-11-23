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
 */

import type { RoadPlan } from '../core/protocol';
import { kindInfo, onewayOf } from '../core/roadKinds';
import { deckSpans, lengths, nearest, RoadIndex } from './roads';

/** Two car roads cross on the level where a point of one comes this near the other's middle (m; their points about a metre apart). */
const CROSS = 0.75;

/** Whether road `r` passes over or under road `other` at `s` m along it rather than crossing it: it says so (`crosses`), or it is on a deck there. */
const over = (r: RoadPlan, other: number, s: number) => !!r.crosses?.some((c) => c.road === other) || deckSpans(r).some(([a, b]) => s >= a && s <= b);

/** Whether from every car road a car can reach every other and come back (see the top of this file). */
export function stronglyConnected(roads: readonly RoadPlan[]): boolean {
  return cutOffRoads(roads).length === 0;
}

/**
 * The car roads (indices) a car can't both reach from the world beyond its ways out and get back from (with no way
 * out, from the first car road): some stretch of each one-way-only, or off on its own. `twoWay`: as if every road
 * were driven both ways (what is cut off by no one-way road).
 */
export function cutOffRoads(roads: readonly RoadPlan[], twoWay = false): number[] {
  const cars = roads.flatMap((r, i) => (kindInfo(r).lanesEachWay > 0 && r.points.length >= 2 ? [i] : []));
  if (!cars.length) return [];
  const isCar = new Set(cars);
  const index = new RoadIndex(roads);
  const widest = Math.max(0, ...cars.map((i) => roads[i].width));
  const lens = roads.map((r, i) => (isCar.has(i) ? lengths(r.points) : []));
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
      if (r.ends[e] === 'out') join(ends[e], 0);
      const [x, z] = e ? r.points[r.points.length - 1] : r.points[0];
      for (const [j, m] of index.around(x, z, widest / 2 + 1)) {
        if (j === i || !isCar.has(j) || !meets(r, roads[j]) || m.dist > Math.max(r.width, roads[j].width) / 2 + 1) continue;
        join(ends[e], stopAt(j, nearest(roads[j].points, x, z, lens[j]).s));
      }
    }
    // (Where it crosses another car road on the level: a crossroads, at the point of it nearest the other's middle.)
    let run: { j: number; s: number; t: number; dist: number } | null = null;
    const close = () => {
      if (run) join(stopAt(i, run.s), stopAt(run.j, run.t));
      run = null;
    };
    r.points.forEach(([x, z], k) => {
      let hit: { j: number; s: number; t: number; dist: number } | null = null;
      for (const [j, m] of index.around(x, z, CROSS)) {
        if (j <= i || !isCar.has(j) || m.dist > CROSS || !meets(r, roads[j]) || over(r, j, lens[i][k]) || over(roads[j], i, m.s)) continue;
        if (!hit || m.dist < hit.dist) hit = { j, s: lens[i][k], t: m.s, dist: m.dist };
      }
      if (run && (!hit || hit.j !== run.j)) close();
      if (hit && (!run || hit.dist < run.dist)) run = hit;
    });
    close();
  }
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
    const way = twoWay ? 0 : onewayOf(roads[i]);
    const order = stops[i].map((s, k) => [s, ids[i][k]] as const).sort((a, b) => a[0] - b[0]);
    for (let k = 0; k + 1 < order.length; k++) {
      if (way >= 0) edge(order[k][1], order[k + 1][1]);
      if (way <= 0) edge(order[k + 1][1], order[k][1]);
    }
  }
  const out = cars.some((i) => roads[i].ends.includes('out'));
  const root = find(out ? 0 : ids[cars[0]][0]);
  const reach = (from: number, ways: Map<number, number[]>) => {
    const seen = new Set([from]);
    const todo = [from];
    while (todo.length) for (const b of ways.get(todo.pop()!) ?? []) if (!seen.has(b)) (seen.add(b), todo.push(b));
    return seen;
  };
  const [ahead, behind] = [reach(root, next), reach(root, back)];
  return cars.filter((i) => ids[i].some((id) => !ahead.has(find(id)) || !behind.has(find(id))));
}
