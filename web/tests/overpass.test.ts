/**
 * The motorway as drawn (`render/shared/motorway.ts`, `street.ts` `overpass`): its carriageways found as a pair and the
 * barrier between them; an overpass's slab, parapets, piers and abutments; the decks' heights on a world with no built
 * surface (`deckLevels`). On the road hierarchy's fixture (`render/roadHierarchyFixture.ts`). (The overview's plain
 * decks: `roads.test.ts`.)
 */
import { describe, expect, test } from 'bun:test';
import { line, points } from './support/geometry';
import type { RoadPlan, WorldStructure } from '../src/core/protocol';
import { Geo } from '../src/render/shared/geometry';
import { allBarriers, barrier, deckLevels, insideOf, liftedStretches, MOTORWAY, motorwayPairs, OVERPASS, signSpots, without } from '../src/render/shared/motorway';
import { nearest } from '../src/game/roads';
import { raisedSpans, ROAD_Y, roadHeightsOf } from '../src/render/shared/roads';
import { overpass, Painter, pierStations } from '../src/render/shared/street';
import * as F from '../src/render/roadHierarchyFixture';

const h = F.roadHierarchy();

describe("a motorway's two carriageways", () => {
  test('are found as a pair, each with the other on its inside', () => {
    const pairs = motorwayPairs(h.roads);
    expect(pairs.get(F.MOTORWAY_EAST)?.other).toBe(F.MOTORWAY_WEST);
    expect(pairs.get(F.MOTORWAY_WEST)?.other).toBe(F.MOTORWAY_EAST);
    expect(pairs.size).toBe(2);
    // (Both run along +x: `insideOf`'s +1, (z, -x), is -z, their right as the game shows it. The westbound lies at +z of
    // the eastbound, traffic keeping right (sim-core `cars::right_of`): each one's inside the left of the way its cars go.)
    expect(insideOf(h.roads, F.MOTORWAY_EAST)).toBe(-1);
    expect(insideOf(h.roads, F.MOTORWAY_WEST)).toBe(1);
  });

  test('a lone carriageway, or a two-way road, has its inside on the left of the way its cars go', () => {
    const lone: RoadPlan = { points: line(0, 0, 100, 0, 100), width: 9, ends: ['out', 'out'], kind: 'motorway', oneway: -1 };
    expect(motorwayPairs([lone]).size).toBe(0);
    expect(insideOf([lone], 0)).toBe(1);
    expect(insideOf([{ ...lone, oneway: 1 }], 0)).toBe(-1);
  });

  test('the barrier runs down the line between them, edge to edge, once', () => {
    const all = allBarriers(h.roads);
    expect(all.length).toBe(1);
    const [{ runs }] = all;
    expect(runs.length).toBe(1);
    const run = runs[0];
    for (const p of run) {
      expect(p.z).toBeCloseTo((F.EAST_Z + F.WEST_Z) / 2, 6);
      expect(p.verge).toBeCloseTo(0, 6);
    }
    expect(run[0].x).toBeLessThan(F.MOTORWAY_X0 + 3);
    expect(run.at(-1)!.x).toBeGreaterThan(F.MOTORWAY_X1 - 3);
  });

  test('the barrier: concrete as high as a barrier, as wide at its foot, closed at its ends', () => {
    const g = new Geo();
    const run = allBarriers(h.roads)[0].runs[0].slice(0, 10);
    barrier(g, run, () => 2);
    const ps = points(g);
    expect(Math.min(...ps.map((p) => p[1]))).toBeCloseTo(2, 6);
    expect(Math.max(...ps.map((p) => p[1]))).toBeCloseTo(2 + MOTORWAY.high, 6);
    const zs = ps.map((p) => p[2] - (F.EAST_Z + F.WEST_Z) / 2);
    expect(Math.min(...zs)).toBeCloseTo(-MOTORWAY.base / 2, 6);
    expect(Math.max(...zs)).toBeCloseTo(MOTORWAY.base / 2, 6);
    // Every face's normal points out of it: away from the line, or up, or out of its ends.
    for (let i = 0; i < g.idx.length; i += 3) {
      const k = g.idx[i];
      const [x, y, z] = [g.pos[k * 3], g.pos[k * 3 + 1], g.pos[k * 3 + 2]];
      const [nx, ny, nz] = [g.nor[k * 3], g.nor[k * 3 + 1], g.nor[k * 3 + 2]];
      const out = (z - (F.EAST_Z + F.WEST_Z) / 2) * nz + (y - 2 - MOTORWAY.high / 2) * ny * 0.2 + (x - (run[0].x + run.at(-1)!.x) / 2) * nx;
      expect(out).toBeGreaterThan(-1e-6);
    }
  });
});

describe("the decks' heights on a world with no built surface", () => {
  const levels = deckLevels(h.roads)!;

  test('none raised: none', () => {
    expect(deckLevels([{ points: line(0, 0, 100, 0, 100), width: 7, ends: ['out', 'out'] }])).toBeNull();
  });

  test('a raised stretch clear of the street under it, climbing to it at its grade, on the ground far off', () => {
    const east = h.roads[F.MOTORWAY_EAST];
    const [[a, b]] = raisedSpans(east);
    for (const s of [a, (a + b) / 2, b]) expect(levels.road(east, s)).toBeCloseTo(MOTORWAY.clear, 6);
    expect(levels.road(east, F.RIVER_X)).toBe(0);
    // (Climbing no steeper than about twice its grade, smoothed.)
    for (let s = 0; s < a; s += 2) expect(Math.abs(levels.road(east, s + 2) - levels.road(east, s)) / 2).toBeLessThan(0.05 * 1.6);
    // The street under it stays on the ground.
    expect(levels.road(h.roads[F.BRIDGE_STREET], F.EAST_Z - F.AVENUE_Z)).toBe(0);
    expect(levels.junction({ x: F.STREET_X, z: F.AVENUE_Z })).toBe(0);
  });

  test("a ramp from its carriageway's height where it merges down to the ground at the street", () => {
    const [off, on] = [h.roads[F.OFF_RAMP], h.roads[F.ON_RAMP]];
    const east = h.roads[F.MOTORWAY_EAST];
    expect(levels.road(off, 0)).toBeCloseTo(levels.road(east, 20 - F.MOTORWAY_X0), 6);
    expect(levels.road(off, 1e6)).toBe(0);
    expect(levels.road(on, 0)).toBe(0);
  });

  test('stretches up off the ground, and less what is cut', () => {
    const east = h.roads[F.MOTORWAY_EAST];
    const lifted = liftedStretches(east, levels);
    expect(lifted.length).toBe(1);
    const [[a, b]] = raisedSpans(east);
    expect(lifted[0][0]).toBeLessThan(a);
    expect(lifted[0][1]).toBeGreaterThan(b);
    expect(without(lifted, [[a, b]])).toEqual([
      [lifted[0][0], a],
      [b, lifted[0][1]],
    ]);
  });
});

describe('an overpass', () => {
  /** An overpass along x from 0 to `long`, its road `width` wide, at `y` over flat ground at 0, drawn into one geometry. */
  const draw = (long: number, width: number, y: number, options: Parameters<typeof overpass>[4] = {}) => {
    const g = new Geo();
    const p = new Painter(g);
    overpass(line(0, 0, long, 0, Math.ceil(long)), width, () => p, { deck: () => y, ground: () => 0 }, options);
    return points(g);
  };

  test('its slab under the road, its parapets over it, its piers and abutments down into the ground', () => {
    const ps = draw(60, 9, 6.4);
    const ys = ps.map((p) => p[1]);
    expect(Math.max(...ys)).toBeCloseTo(6.4 + OVERPASS.parapet, 6);
    expect(Math.min(...ys)).toBeCloseTo(-OVERPASS.sink, 6);
    // (Two columns under a wide deck, at a pier every 20 m: at 20 and 40.)
    const feet = ps.filter((p) => p[1] < 0);
    const xs = [...new Set(feet.map((p) => Math.round(p[0] / 5) * 5))].sort((a, b) => a - b);
    expect(xs).toContain(20);
    expect(xs).toContain(40);
    expect(new Set(feet.map((p) => Math.sign(p[2]))).size).toBe(2);
    // An abutment under each end.
    expect(feet.some((p) => p[0] < 2)).toBe(true);
    expect(feet.some((p) => p[0] > 58)).toBe(true);
  });

  test('one column under a narrow deck', () => {
    // (Its piers', not its abutments' at its ends.)
    const feet = draw(45, 5, 6.4).filter((p) => p[1] < 0 && p[0] > 2 && p[0] < 43);
    expect(feet.length).toBeGreaterThan(0);
    for (const [, , z] of feet) expect(Math.abs(z)).toBeLessThanOrEqual(OVERPASS.column / 2 + 1e-6);
  });

  test('on the ground: nothing stands under it', () => {
    const ps = draw(60, 9, 0.5);
    expect(Math.min(...ps.map((p) => p[1]))).toBeGreaterThanOrEqual(0.5 - OVERPASS.slab - 1e-6);
  });

  test("no parapet on its inside (its pair's side), its slab reaching half the verge there", () => {
    const ps = draw(30, 9, 6.4, { inner: -1, innerApron: 0 });
    const high = ps.filter((p) => p[1] > 6.4 + 0.5);
    // (Local +x, the deck's left, is -z along +x.)
    expect(high.every((p) => p[2] < 0)).toBe(true);
    expect(Math.max(...ps.filter((p) => p[1] > 4).map((p) => p[2]))).toBeCloseTo(4.5, 6);
  });

  test('over a road: no pier on it, one each side of it instead', () => {
    // (The farm world's overpass: 31 m, the road under it across its middle; its one pier, at 15.5 m, was on the road
    // and left out, the deck running from bank to bank.)
    const clear: [number, number][] = [[6.7, 24.3]];
    expect(pierStations(31, 2, clear)).toEqual([6.7, 24.3]);
    const feet = draw(31, 9, 6.4, { clear }).filter((p) => p[1] < 0 && p[0] > 2 && p[0] < 29);
    const xs = [...new Set(feet.map((p) => Math.round(p[0])))];
    expect(xs.some((x) => Math.abs(x - 6.7) < 1.2)).toBe(true);
    expect(xs.some((x) => Math.abs(x - 24.3) < 1.2)).toBe(true);
    for (const [x] of feet) expect(x < 6.7 + OVERPASS.cap || x > 24.3 - OVERPASS.cap).toBe(true);
  });

  test('its piers: evenly where nothing passes under, none at its ends or too near them', () => {
    expect(pierStations(60, 3, [])).toEqual([20, 40]);
    expect(pierStations(18, 1, [[5, 13]])).toEqual([]);
    // (The road under it near an end: the pier on that side would stand at the end, left out; the other stands.)
    expect(pierStations(40, 2, [[1, 21]])).toEqual([21]);
    expect(pierStations(40, 2, [[17, 25]])).toEqual([17, 25]);
    // (Two roads under it side by side: beside them, and between them where their clear stretches meet.)
    expect(pierStations(60, 3, [[12, 21], [21, 30]])).toEqual([12, 21, 40]);
  });

  test('its slab closed under the road, for its shadow: a face looking up across its width, below the road', () => {
    const g = new Geo();
    const p = new Painter(g);
    overpass(line(0, 0, 30, 0, 30), 9, () => p, { deck: () => 6.4, ground: () => 0 });
    // (A face looking up below the road's surface, as wide as the slab, all along it.)
    const lid: [number, number][] = [];
    for (let k = 0; k < g.pos.length / 3; k++) {
      const [y, ny] = [g.pos[k * 3 + 1], g.nor[k * 3 + 1]];
      if (ny > 0.99 && y < 6.4 && y > 6.4 - OVERPASS.slab) lid.push([g.pos[k * 3], g.pos[k * 3 + 2]]);
    }
    expect(Math.min(...lid.map((q) => q[0]))).toBeCloseTo(0, 6);
    expect(Math.max(...lid.map((q) => q[0]))).toBeCloseTo(30, 6);
    expect(Math.max(...lid.map((q) => q[1])) - Math.min(...lid.map((q) => q[1]))).toBeCloseTo(9 + 2 * OVERPASS.apron, 6);
  });

  test('a solid ramp: walls down its sides, no piers, no slab under it', () => {
    const ps = draw(40, 9, 3, { solid: true });
    const under = ps.filter((p) => p[1] < 3 - 0.5);
    expect(under.length).toBeGreaterThan(0);
    for (const [, , z] of under) expect(Math.abs(z)).toBeGreaterThan(4.5 + OVERPASS.apron - OVERPASS.foot - 1e-6);
  });
});

describe("the motorway's direction signs", () => {
  test('in the off-ramp\'s gore and by the on-ramp at the street, off every road, facing the traffic coming', () => {
    const spots = signSpots(h.roads);
    expect(spots.length).toBe(2);
    for (const p of spots) for (const r of h.roads) expect(nearest(r.points, p.x, p.z).dist).toBeGreaterThan(r.width / 2 + 0.5);
    const [gore, entry] = spots;
    // (The gore between the eastbound carriageway and the off-ramp, past where they part; its traffic going +x: the sign's face, its +z turned to (sin yaw, cos yaw), to -x.)
    // (Off the eastbound's right, -z: sim-core `cars::right_of`.)
    expect(gore.z).toBeLessThan(F.EAST_Z - 4.5);
    expect(gore.x).toBeGreaterThan(110);
    expect(gore.x).toBeLessThan(F.STREET_X);
    expect(Math.sin(gore.yaw)).toBeLessThan(-0.5);
    // (By the on-ramp's start at the street, its cars going -x: facing +x.)
    expect(Math.hypot(entry.x - F.STREET_X, entry.z - (F.WEST_Z + 25))).toBeLessThan(15);
    expect(Math.sin(entry.yaw)).toBeGreaterThan(0.5);
  });

  test('before an exit along its carriageway where it runs that far, outside its shoulder', () => {
    // (The fixture's off-ramp moved 400 m on: the eastbound carriageway made long enough before it.)
    const shift = (r: RoadPlan): RoadPlan => ({ ...r, points: r.points.map(([x, z]) => [x + 400, z]) });
    const east: RoadPlan = { ...h.roads[F.MOTORWAY_EAST], points: line(0, F.EAST_Z, 900, F.EAST_Z, 900), raised: undefined, bridge: undefined, crosses: undefined };
    const west: RoadPlan = { ...east, points: line(0, F.WEST_Z, 900, F.WEST_Z, 900), oneway: -1 };
    const roads = [east, west, shift(h.roads[F.OFF_RAMP])];
    const before = signSpots(roads).filter((p) => p.x < 420);
    expect(before.map((p) => Math.round(420 - p.x)).sort((a, b) => a - b)).toEqual([150, 300]);
    // (Outside: away from the westbound carriageway, at -z.)
    for (const p of before) expect(p.z).toBeLessThan(F.EAST_Z - 4.5);
  });
});

describe("step 2's outline round a road picked (`roadHeightsOf`)", () => {
  const world = { width: 400, depth: 300, plots: [], meta: { roads: h.roads } } as unknown as WorldStructure;
  const heights = roadHeightsOf(world)!;

  test('on the motorway\'s deck over the street, not on the ground under it', () => {
    const east = h.roads[F.MOTORWAY_EAST];
    // (Its line as the editor has it: a copy of the road's points.)
    const along = heights(east.points.map(([x, z]): [number, number] => [x, z]))!;
    const [[a, b]] = raisedSpans(east);
    expect(along((a + b) / 2)).toBeCloseTo(MOTORWAY.clear + ROAD_Y, 6);
    expect(along(F.RIVER_X)).toBeCloseTo(ROAD_Y, 6);
    // The street under it: on the ground.
    expect(heights(h.roads[F.BRIDGE_STREET].points)!(F.EAST_Z - F.AVENUE_Z)).toBeCloseTo(ROAD_Y, 6);
  });

  test('a line the town has no road along: none', () => {
    expect(heights(line(0, 0, 50, 0, 5))).toBeNull();
    expect(roadHeightsOf({ ...world, meta: { roads: [] } } as unknown as WorldStructure)).toBeNull();
  });
});
