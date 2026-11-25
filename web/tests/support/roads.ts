// Road networks as the sim joins them, for the town tests.
import type { RoadPlan } from '../../src/core/protocol';
import { nearest } from '../../src/game/roads';

/** Per road, its network, as the sim joins them (an end within half the wider road's width and a metre of the other's middle). */
export function networks(roads: readonly RoadPlan[]): number {
  const parent = roads.map((_, i) => i);
  const find = (i: number): number => (parent[i] === i ? i : (parent[i] = find(parent[i])));
  roads.forEach((r, i) => {
    for (const end of [r.points[0], r.points[r.points.length - 1]]) {
      roads.forEach((o, j) => {
        if (j !== i && nearest(o.points, end[0], end[1]).dist <= Math.max(o.width, r.width) / 2 + 1) parent[find(i)] = find(j);
      });
    }
  });
  return new Set(roads.map((_, i) => find(i))).size;
}
