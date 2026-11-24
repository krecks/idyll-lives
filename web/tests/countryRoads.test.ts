/**
 * `CountryRoads.near` through its 8 m grid finds what a scan of every segment finds, ties and
 * misses alike. A motorway goes on past the edge, over the land and over the water; the roads out
 * past the edge keep off a river that runs on past it, and cross one that swings across their way.
 */
import { describe, expect, test } from 'bun:test';
import type { WorldStructure } from '../src/core/protocol';
import { CountryRoads, DECK_OVER, MOTORWAY_WIDTH, ROAD_VERGE, STEP, type NearRoad, type Point, type RoadLand } from '../src/render/shared/countryRoads';

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

  test('a road crosses a river that swings across its way, not on along its bed', () => {
    // A river 20 m wide out of the west edge at z = 200, swinging 80 m north within 80 m of it and
    // on west along z = 120, in a valley: across the way of a road out west at z = 150.
    const mid = (x: number) => 200 - 80 * Math.min(1, Math.max(0, -x / 80));
    const wet = (x: number, z: number) => 10 - Math.abs(z - mid(x));
    const out = { points: [[40, 150], [0, 150]], width: 7, ends: ['join', 'out'], kind: 'country' };
    const world = { width: 400, depth: 400, meta: { roads: [out] } } as unknown as WorldStructure;
    for (let seed = 1; seed <= 8; seed++) {
      const land: RoadLand = {
        seed,
        w: 400,
        d: 400,
        ring: 260,
        ground: (x, z) => Math.abs(z - mid(x)) * 0.4,
        waterEdge: wet,
        distanceToTown: (x, z) => Math.hypot(Math.max(-x, 0, x - 400), Math.max(-z, 0, z - 400)),
      };
      const [road] = new CountryRoads(land, world, 'Test').roads;
      expect(road.at(-1)![0]).toBeLessThan(-200);
      // Over it once, a few steps (it ran on along the bed for 54 of its 74 points).
      expect(road.filter(([x, z]) => wet(x, z) > 0).length).toBeLessThanOrEqual(8);
    }
  });
});

describe("a motorway past the world's edge", () => {
  // (A town 200 m square, its motorway's two carriageways along z = 100 and 109 from edge to edge, on a bridge at the
  // west edge; past it the sea, 3 m deep, past the east edge the land.)
  const WATER = -0.35;
  const line = (z: number, oneway: 1 | -1) => ({ points: Array.from({ length: 21 }, (_, i): [number, number] => [i * 10, z]), width: MOTORWAY_WIDTH, ends: ['out', 'out'], kind: 'motorway', oneway });
  const world = { width: 200, depth: 200, meta: { roads: [line(100, 1), line(109, -1)] } } as unknown as WorldStructure;
  const outside = (x: number, z: number) => Math.max(0, -x, x - 200, -z, z - 200);
  const land = (bridge: number | null): RoadLand => ({
    seed: 1,
    w: 200,
    d: 200,
    ring: 300,
    ground: (x) => (x < 1 ? -3 : 0.5),
    waterEdge: (x) => (x < 1 ? 1 - x : -100),
    distanceToTown: outside,
    follows: true,
    water: WATER,
    ...(bridge === null ? {} : { surface: () => bridge }),
  });

  test('goes on over the water on a deck, its carriageways side by side, to the end of the land', () => {
    const roads = new CountryRoads(land(4), world, 'Test');
    const west = [...roads.motorways].filter((r) => roads.roads[r][0][0] < 1);
    expect(west.length).toBe(2);
    for (const r of west) {
      const road = roads.roads[r];
      // (Out to the land's end, not stopped at the shore.)
      expect(road.at(-1)![0]).toBeLessThan(-280);
      const deck = roads.decks.get(r)!;
      expect(deck.on.every((w) => w)).toBe(true);
      // From the bridge's height at the edge, down to a low deck over the water, never under it.
      expect(road[0][2]).toBeCloseTo(4, 6);
      for (const p of road) expect(p[2]).toBeGreaterThanOrEqual(WATER + DECK_OVER - 1e-6);
      expect(road.at(-1)![2]).toBeCloseTo(WATER + DECK_OVER, 6);
      expect(deck.bed.every((b) => b === -3)).toBe(true);
      // Its pair beside it, at one height.
      const other = roads.roads[roads.pairs.get(r)!];
      road.forEach((p, i) => expect(p[2]).toBeCloseTo(other[i][2], 6));
    }
    expect(roads.barriers.some((b) => b[0][0] < 1 && b.at(-1)![0] < -280)).toBe(true);
    // The sea floor under it isn't raised to it (no levelling under a deck: `near` finds no road there).
    expect(roads.near(-100, 104.5)).toBeNull();
    // Over the land past the east edge it lies on the land, levelled into it.
    const east = [...roads.motorways].filter((r) => roads.roads[r][0][0] > 199);
    expect(east.length).toBe(2);
    for (const r of east) expect(roads.decks.has(r)).toBe(false);
    expect(roads.near(260, 100)?.h).toBeCloseTo(0.5, 6);
  });

  test('no bridge height to start from (no surface): a low deck all the way', () => {
    const roads = new CountryRoads(land(null), world, 'Test');
    for (const r of [...roads.motorways].filter((r) => roads.roads[r][0][0] < 1)) {
      const road = roads.roads[r];
      expect(road.length).toBeGreaterThan(280 / STEP);
      for (const p of road.slice(1)) expect(p[2]).toBeGreaterThanOrEqual(WATER + DECK_OVER - 1e-6);
    }
  });
});
