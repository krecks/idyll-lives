/**
 * `CountryRoads.near` through its 8 m grid finds what a scan of every segment finds, ties and
 * misses alike.
 */
import { describe, expect, test } from 'bun:test';
import type { WorldStructure } from '../src/core/protocol';
import { CountryRoads, ROAD_VERGE, type NearRoad, type Point, type RoadLand } from '../src/render/shared/countryRoads';

function scan(roads: Point[][], x: number, z: number): NearRoad | null {
  let best: NearRoad | null = null;
  for (const road of roads) {
    for (let i = 0; i + 1 < road.length; i++) {
      const [ax, az, ah, aw] = road[i];
      const [bx, bz, bh, bw] = road[i + 1];
      const [dx, dz] = [bx - ax, bz - az];
      const t = Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / (dx * dx + dz * dz)));
      const dist = Math.hypot(x - (ax + dx * t), z - (az + dz * t));
      if (dist < (best?.dist ?? aw / 2 + ROAD_VERGE)) best = { dist, h: ah + (bh - ah) * t, width: aw + (bw - aw) * t };
    }
  }
  return best;
}

describe('country roads', () => {
  test('the grid finds the nearest road as a scan does', () => {
    const land: RoadLand = { seed: 3, w: 100, d: 100, ring: 260, ground: () => 0, waterEdge: () => -100, distanceToTown: () => 0 };
    const world = { width: 100, depth: 100, meta: { roads: [] } } as unknown as WorldStructure;
    const roads = new CountryRoads(land, world, 'Test');
    let seed = 7;
    const rand = () => ((seed = (Math.imul(seed, 1103515245) + 12345) >>> 0) / 4294967296);
    for (let r = 0; r < 4; r++) {
      const road: Point[] = [];
      let [x, z, a] = [rand() * 400 - 200, rand() * 400 - 200, rand() * 6.28];
      for (let i = 0; i < 120; i++) {
        road.push([x, z, rand() * 3, 5 + rand()]);
        a += (rand() - 0.5) * 0.3;
        x += Math.cos(a) * 4;
        z += Math.sin(a) * 4;
      }
      roads.roads.push(road);
    }
    for (let k = 0; k < 20000; k++) {
      const [x, z] = [rand() * 900 - 450, rand() * 900 - 450];
      expect(roads.near(x, z)).toEqual(scan(roads.roads, x, z));
    }
  });
});
