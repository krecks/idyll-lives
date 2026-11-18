// Bridges and causeways as drawn (`RoadPlan.bridge`): the deck is the stretch of the road over its span
// (`roads.ts` `bridgeDeck`); the street draws a stone deck `width + 4` m wide under it from below the water to
// just under the road, parapets 0.9 m high along its edges and a pier's nose each side every 8 m (`street.ts`
// `bridge`), in the chunks it crosses; no stop lines or crosswalks where a bridge meets the next country road (two
// arms: no junction); an island's sea reaches the horizon all round. `bun test`.
import { describe, expect, test } from 'bun:test';
import { Geo } from '../src/render/shared/geometry';
import { bridgeDeck, roadMeshes, ROAD_Y } from '../src/render/shared/roads';
import { junctionsOf } from '../src/render/shared/junctions';
import { CountryRoads } from '../src/render/shared/countryRoads';
import { GROUND_TILE, LandscapeLayout, WATER_Y } from '../src/render/shared/nature';
import { islandWorld, riverWorld, RIVER, SIDE } from './support/landFixtures';

/** The bounds of a geometry's vertices (x, y, z), and the vertices themselves. */
function bounds(pos: ArrayLike<number>): { min: number[]; max: number[]; at: [number, number, number][] } {
  const at: [number, number, number][] = [];
  for (let i = 0; i < pos.length; i += 3) at.push([pos[i], pos[i + 1], pos[i + 2]]);
  const min = [0, 1, 2].map((k) => Math.min(...at.map((p) => p[k])));
  const max = [0, 1, 2].map((k) => Math.max(...at.map((p) => p[k])));
  return { min, max, at };
}

describe('a bridge deck', () => {
  test("the stretch of a road over its span, along the road's own points", () => {
    const world = riverWorld();
    const roads = world.meta!.roads!;
    const deck = bridgeDeck(roads.find((r) => r.bridge)!)!;
    // From 2 m short of the water's south bank to 2 m past its north one.
    expect(deck).toEqual([
      [256, RIVER.z1 + 2],
      [256, RIVER.z0 - 2],
    ]);
    for (const r of roads) if (!r.bridge) expect(bridgeDeck(r)).toBeNull();
    // Kept within the road, and through its bends.
    const bent = { points: [[0, 0], [10, 0], [10, 10]] as [number, number][], width: 7, ends: ['join', 'join'] as const, bridge: [5, 40] as [number, number] };
    expect(bridgeDeck({ ...bent, ends: ['join', 'join'] })).toEqual([
      [5, 0],
      [10, 0],
      [10, 10],
    ]);
    expect(bridgeDeck({ ...bent, ends: ['join', 'join'], bridge: [3, 3] })).toBeNull();
  });
});

describe("a bridge's stone", async () => {
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' }, requestAnimationFrame: () => 0, window: {} });
  const { bridge, BRIDGE, Painter, TownStreet, NO_FOCUS } = await import('../src/render/shared/street');
  const { HousePlanner } = await import('../src/render/shared/house');
  const { chunkAt } = await import('../src/render/shared/chunks');

  test('a deck width + 4 m wide, from under the water to just under the road; parapets; a nose each side every 8 m', () => {
    const geo = new Geo();
    const painter = new Painter(geo);
    const pieces: [number, number][] = [];
    // 30 m north to south at x = 100: four pieces of 7.5 m.
    bridge(
      [
        [100, 200],
        [100, 170],
      ],
      7,
      (x, z) => {
        pieces.push([x, z]);
        return painter;
      },
    );
    expect(pieces.map((p) => p[1])).toEqual([196.25, 188.75, 181.25, 173.75]);
    const { min, max, at } = bounds(geo.pos);
    const half = 3.5 + BRIDGE.apron;
    // Noses jut out past the deck's sides.
    expect(min[0]).toBeCloseTo(100 - half - BRIDGE.nose, 5);
    expect(max[0]).toBeCloseTo(100 + half + BRIDGE.nose, 5);
    expect(min[1]).toBeCloseTo(BRIDGE.under, 5);
    expect(BRIDGE.under).toBeLessThan(WATER_Y);
    // The end posts are the highest; the parapets 0.9 m over the deck.
    expect(max[1]).toBeCloseTo(BRIDGE.top + BRIDGE.postUp, 5);
    expect(min[2]).toBeCloseTo(170 - BRIDGE.post / 2, 5);
    expect(max[2]).toBeCloseTo(200 + BRIDGE.post / 2, 5);
    const parapets = at.filter(([x, , z]) => Math.abs(Math.abs(x - 100) - half) < 0.06 && z > 171 && z < 199);
    expect(Math.max(...parapets.map((p) => p[1]))).toBeCloseTo(BRIDGE.top + BRIDGE.parapet, 5);
    // The deck's top: under the road's surface, over the levelled ground (0); nothing else between it and the water.
    const deckTop = at.filter(([, y]) => y > WATER_Y && y < 0.4).map((p) => p[1]);
    expect(deckTop.length).toBeGreaterThan(8);
    expect(new Set(deckTop)).toEqual(new Set([BRIDGE.top]));
    expect(BRIDGE.top).toBeLessThan(ROAD_Y);
    expect(BRIDGE.top).toBeGreaterThan(0);
    // Noses at the three piers between the pieces (none at the banks), each side.
    const tips = at.filter(([x]) => Math.abs(x - 100) > half + BRIDGE.nose - 0.01);
    expect([...new Set(tips.map((p) => `${p[0]}:${p[2]}`))].sort()).toEqual(['105.5', '94.5'].flatMap((x) => [192.5, 185, 177.5].map((z) => `${Number(x) + (x === '105.5' ? 0.9 : -0.9)}:${z}`)).sort());
  });

  test("on a land the town follows, the deck is cut where the road takes its heights: its top follows the road's, never over it", () => {
    // 30 m north to south, the road coming down 3 m off the bank over its first 6 m and on level (a kink at 6 m): a
    // piece from pier to pier (7.5 m) straight would stand 0.6 m over the road at the kink.
    const road = (u: number) => Math.max(0, 3 - 0.5 * u);
    const draw = (stops?: number[]) => {
      const geo = new Geo();
      bridge(
        [
          [100, 200],
          [100, 170],
        ],
        7,
        () => new Painter(geo),
        { deck: road, bed: () => -3, ...(stops ? { stops } : {}) },
      );
      // (The deck's top, straight from corner to corner of each piece: how far over the road's own surface, at its
      // most, every 25 cm along.)
      const half = 3.5 + BRIDGE.apron;
      const top = bounds(geo.pos)
        .at.filter(([x, y, z]) => Math.abs(x - 100) < half + 0.01 && Math.abs(y - BRIDGE.top - road(200 - z)) < 1e-6)
        .map(([, y, z]): [number, number] => [200 - z, y - BRIDGE.top])
        .sort((a, b) => a[0] - b[0]);
      let most = -Infinity;
      for (let u = 0; u <= 30; u += 0.25) {
        const k = Math.max(0, top.findIndex(([v]) => v >= u) - 1);
        const [[u0, y0], [u1, y1]] = [top[k], top[Math.min(top.length - 1, k + 1)]];
        most = Math.max(most, (u1 > u0 ? y0 + ((y1 - y0) * (u - u0)) / (u1 - u0) : y0) - road(u));
      }
      return most;
    };
    expect(draw()).toBeGreaterThan(0.5);
    expect(draw(Array.from({ length: 29 }, (_, k) => k + 1))).toBeLessThan(1e-5);
  });

  test("a generated road's deck (points a metre apart) is drawn as its straight runs; round a bend, no more piers", () => {
    const draw = (deck: [number, number][]) => {
      const geo = new Geo();
      const painter = new Painter(geo);
      let pieces = 0;
      bridge(deck, 7, () => (pieces++, painter));
      return { pos: geo.pos, pieces };
    };
    const sparse = draw([
      [100, 200],
      [100, 170],
    ]);
    const dense = draw(Array.from({ length: 31 }, (_, k): [number, number] => [100, 200 - k]));
    expect(dense.pieces).toBe(sparse.pieces);
    expect(dense.pos.length).toBe(sparse.pos.length);
    dense.pos.forEach((v, i) => expect(v).toBeCloseTo(sparse.pos[i], 9));
    // An arc of 60 m radius, 40 m along it: five pieces' worth of piers (four), each side; the stone along the arc.
    const arc = Array.from({ length: 41 }, (_, k): [number, number] => [60 * Math.cos(k / 60), 60 * Math.sin(k / 60)]);
    const bent = draw(arc);
    const { at } = bounds(bent.pos);
    const half = 3.5 + BRIDGE.apron;
    const tips = new Set(at.filter(([x, , z]) => Math.abs(Math.hypot(x, z) - 60) > half + BRIDGE.nose - 0.05).map(([x, , z]) => `${x.toFixed(3)}:${z.toFixed(3)}`));
    expect(tips.size).toBe(8);
    for (const [x, , z] of at) expect(Math.abs(Math.hypot(x, z) - 60)).toBeLessThan(half + BRIDGE.nose + 0.3);
  });

  test('the street draws it in the chunks the bridge crosses, and nothing for the roads without one', async () => {
    const options = { has: () => true, entry: () => undefined, lived: true, ground: true, look: () => undefined };
    const world = riverWorld();
    const street = new TownStreet(new HousePlanner(), world, options, 'game');
    const deck = bridgeDeck(world.meta!.roads!.find((r) => r.bridge)!)!;
    const chunks = new Set(deck.map(([x, z]) => chunkAt(x, z)));
    let stone = 0;
    for (const id of street.chunkIds()) {
      const { at } = bounds(street.chunk(id, NO_FOCUS).props.pos);
      const over = at.filter(([x, y, z]) => Math.abs(x - 256) <= 6.5 && z >= RIVER.z0 - 2.5 && z <= RIVER.z1 + 2.5 && y < 0);
      if (over.length) expect(chunks.has(id)).toBe(true);
      stone += over.length;
    }
    expect(stone).toBeGreaterThan(20);
    // (Without the bridge: no stone under the water anywhere.)
    const plain = riverWorld();
    for (const r of plain.meta!.roads!) delete r.bridge;
    const none = new TownStreet(new HousePlanner(), plain, options, 'game');
    for (const id of none.chunkIds()) expect(bounds(none.chunk(id, NO_FOCUS).props.pos).at.filter(([, y]) => y < 0).length).toBe(0);
  });

  test('no stop lines or crosswalks where a bridge meets the next road', () => {
    const world = riverWorld();
    const roads = world.meta!.roads!;
    // (Two-arm joints, even sent as junctions, aren't drawn as junctions.)
    expect(world.junctions!.length).toBe(2);
    expect(junctionsOf(world, roads)).toEqual([]);
    const marks = new Geo();
    roadMeshes(roads, null, 0, { marks }, 'surface', junctionsOf(world, roads));
    expect(marks.pos.length).toBe(0);
  });
});

describe('an island', () => {
  test('its sea reaches past the edge all round to the horizon, but for the causeway out', () => {
    const world = islandWorld();
    const land = new LandscapeLayout(world, (l, w, name) => new CountryRoads(l, w, name));
    const grown = land as unknown as { waterSheet(): { size: number; x: number; z: number }; terrainGrid(): { size: number } };
    expect(land.watered).toBe(true);
    // The water's sheet is under the whole ground, as far as it reaches: the terrain's square (its last tiles stop at
    // its edge, though a whole tile would reach past it).
    const sheet = grown.waterSheet();
    const size = grown.terrainGrid().size;
    expect(size % GROUND_TILE).not.toBe(0);
    const [x0, x1] = [SIDE / 2 - size / 2, SIDE / 2 + size / 2];
    expect(sheet.x - sheet.size / 2).toBeCloseTo(x0, 6);
    expect(sheet.x + sheet.size / 2).toBeCloseTo(x1, 6);
    expect(sheet.z - sheet.size / 2).toBeCloseTo(x0, 6);
    expect(sheet.z + sheet.size / 2).toBeCloseTo(x1, 6);
    const causeway = world.meta!.roads!.find((r) => r.bridge)!;
    const [, cz] = causeway.points[0];
    let n = 0;
    for (let a = 0; a < 360; a += 2) {
      const [c, s] = [Math.cos((a * Math.PI) / 180), Math.sin((a * Math.PI) / 180)];
      const t = Math.min(SIDE / 2 / Math.abs(c || 1e-9), SIDE / 2 / Math.abs(s || 1e-9));
      for (const out of [4, 40, 120, 250]) {
        const [x, z] = [SIDE / 2 + c * (t + out), SIDE / 2 + s * (t + out)];
        // (The causeway's way on, east past the edge, is a bank over the sea.)
        if (x > SIDE && Math.abs(z - cz) < 16) continue;
        expect(land.waterEdge(x, z)).toBeGreaterThan(0);
        expect(land.heightAt(x, z)).toBeLessThan(WATER_Y);
        n++;
      }
    }
    expect(n).toBeGreaterThan(600);
    // The causeway's deck reaches the edge.
    const deck = bridgeDeck(causeway)!;
    expect(deck[deck.length - 1][0]).toBe(SIDE);
  });
});
