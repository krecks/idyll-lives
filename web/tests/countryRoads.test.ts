/**
 * `CountryRoads.near` through its 8 m grid finds what a scan of every segment finds, ties and
 * misses alike; the roads out past the edge keep off a river that runs on past it.
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

  test("two roads out either side of a river through the town's middle keep off it, apart", () => {
    // A 400 m world, a river 20 m wide along z = 200 (its middle) out past the west edge, in a
    // valley; a way out west 50 m north of it and one 60 m south.
    const wet = (_x: number, z: number) => Math.min(z - 190, 210 - z);
    const land: RoadLand = {
      seed: 11,
      w: 400,
      d: 400,
      ring: 260,
      ground: (_x, z) => Math.abs(z - 200) * 0.4,
      waterEdge: wet,
      distanceToTown: (x, z) => Math.hypot(Math.max(-x, 0, x - 400), Math.max(-z, 0, z - 400)),
    };
    const out = (z: number) => ({ points: [[40, z], [0, z]], width: 7, ends: ['join', 'out'], kind: 'country' });
    const world = { width: 400, depth: 400, meta: { roads: [out(150), out(260)] } } as unknown as WorldStructure;
    const [north, south] = new CountryRoads(land, world, 'Test').roads;
    // (Out to the end of the land, west.)
    for (const road of [north, south]) expect(road.at(-1)![0]).toBeLessThan(-200);
    // Never within 2 m of the water (they used to turn into it and run on along its bed).
    for (const [x, z] of [...north, ...south]) expect(wet(x, z)).toBeLessThan(-2);
    // Each on its own side of it, never on top of the other.
    for (const [, z] of north) expect(z).toBeLessThan(190);
    for (const [, z] of south) expect(z).toBeGreaterThan(210);
  });
});
