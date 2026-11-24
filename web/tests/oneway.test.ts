// One-way roads never cut a place off (`game/oneway.ts`): the roads joined as the simulation joins them, each
// one-way road gone along its way only, every car road reached from the world beyond the ways out and left for it
// again (sim-core `Graph::strongly_connected` on the same fixture: `travel_hierarchy.rs`
// `one_way_roads_keep_every_place_reachable`); and the generator's one-way pairs.
import { describe, expect, test } from 'bun:test';
import type { RoadPlan } from '../src/core/protocol';
import { cutOffAnyWay, cutOffRoads, oneWayPairs, stronglyConnected } from '../src/game/oneway';
import { LOOP, roadHierarchy } from '../src/render/roadHierarchyFixture';

type Pt = [number, number];
const line = (a: Pt, b: Pt): Pt[] => {
  const n = Math.max(1, Math.ceil(Math.hypot(a[0] - b[0], a[1] - b[1])));
  return Array.from({ length: n + 1 }, (_, i): Pt => [a[0] + ((b[0] - a[0]) * i) / n, a[1] + ((b[1] - a[1]) * i) / n]);
};
const street = (a: Pt, b: Pt, ends: RoadPlan['ends'] = ['join', 'join'], oneway?: 1 | -1): RoadPlan => ({ points: line(a, b), width: 7, ends, kind: 'street', ...(oneway ? { oneway } : {}) });

/**
 * A ladder: two long streets 50 m apart (z = 0 and z = 50) from x = 0 to 200, out of town at their west ends, and
 * three rungs across between them (x = 40, 100, 160).
 */
function ladder(): RoadPlan[] {
  return [
    street([0, 0], [200, 0], ['out', 'join']),
    street([0, 50], [200, 50], ['out', 'join']),
    street([40, 0], [40, 50]),
    street([100, 0], [100, 50]),
    street([160, 0], [160, 50]),
    // (The streets' east ends joined round the end.)
    street([200, 0], [200, 50]),
  ];
}

describe('stronglyConnected', () => {
  test('the fixture: the one-way loop and the motorway keep every place reachable', () => {
    const { roads } = roadHierarchy();
    expect(stronglyConnected(roads)).toBe(true);
    expect(cutOffRoads(roads)).toEqual([]);
    // (By the roads alone too, as the simulation's own check has it: the motorway driven along, not asked about.)
    expect(cutOffRoads(roads, { within: true })).toEqual([]);
  });

  test("the fixture's loop with a side turned round: a corner cars come to and can't leave", () => {
    const { roads } = roadHierarchy();
    const turned = roads.map((r, i) => (i === LOOP[3] ? { ...r, oneway: 1 as const } : r));
    expect(stronglyConnected(turned)).toBe(false);
    // (The loop's roads are what is cut off; the rest of the town is not.)
    const cut = cutOffRoads(turned);
    expect(cut.length).toBeGreaterThan(0);
    expect(cut.every((r) => (LOOP as readonly number[]).includes(r))).toBe(true);
    // Two-way again: all well.
    expect(stronglyConnected(roads.map((r, i) => (i === LOOP[3] ? { ...r, oneway: undefined } : r)))).toBe(true);
  });

  test('without a one-way road it holds', () => {
    expect(stronglyConnected(ladder())).toBe(true);
  });

  test('a one-way road out of a dead end cuts it off', () => {
    const roads = [...ladder(), street([100, 50], [100, 90], ['join', 'dead'], -1)];
    expect(stronglyConnected(roads)).toBe(false);
    expect(cutOffRoads(roads)).toContain(6);
  });

  test('a pair of one-way streets either way round the ladder holds; the same way does not', () => {
    const pair = (a: 1 | -1, b: 1 | -1) => ladder().map((r, i) => (i === 3 ? { ...r, oneway: a } : i === 4 ? { ...r, oneway: b } : r));
    expect(stronglyConnected(pair(1, -1))).toBe(true);
    expect(stronglyConnected(pair(-1, 1))).toBe(true);
    // (Both the same way: the first rung and the street round the end still go both ways, so it holds.)
    expect(stronglyConnected(pair(1, 1))).toBe(true);
    // Every rung and the end one-way north: back south only by the world beyond (out by the north street, in by the
    // south one), so it holds; the north street one-way east besides, nothing leaves it.
    const north = ladder().map((r, i) => (i >= 2 ? { ...r, oneway: 1 as const } : r));
    expect(stronglyConnected(north)).toBe(true);
    // (By the roads alone, as the simulation routes a trip in town: not.)
    expect(stronglyConnected(north, true)).toBe(false);
    expect(stronglyConnected(north.map((r, i) => (i === 1 ? { ...r, oneway: 1 as const } : r)))).toBe(false);
  });

  test('a second network with no way out is cut off', () => {
    const far = [street([500, 0], [600, 0], ['dead', 'join']), street([600, 0], [600, 60], ['join', 'dead'], 1)];
    expect(stronglyConnected([...ladder(), ...far])).toBe(false);
    // (A network with no way out is off on its own, whichever way its roads go.)
    expect(cutOffRoads([...ladder(), ...far])).toEqual([6, 7]);
    expect(cutOffAnyWay([...ladder(), ...far])).toEqual([6, 7]);
  });

  test('a road crossing another part way along meets it there', () => {
    // (Across the ladder's long streets, a dead end either side: reached only where it crosses them.)
    const across = street([120, -40], [120, 90], ['dead', 'dead']);
    expect(cutOffAnyWay([...ladder(), across])).toEqual([]);
    // One-way north: its north end can't be left.
    expect(stronglyConnected([...ladder(), { ...across, oneway: 1 }])).toBe(false);
  });
});

describe('oneWayPairs', () => {
  test('two parallel streets 50 m apart become a pair, opposite ways; the main roads out of town stay two-way', () => {
    const roads = [
      street([0, 0], [300, 0], ['out', 'out']),
      street([0, 160], [300, 160], ['out', 'out']),
      // Two rungs 50 m apart between the main roads, and the main roads.
      street([100, 0], [100, 160]),
      street([150, 0], [150, 160]),
    ];
    const { roads: out, pairs } = oneWayPairs(roads, [0, 1, 2, 3], [125, 80], 2);
    expect(pairs).toEqual([[2, 3]]);
    expect(out[2].oneway).toBeDefined();
    expect(out[3].oneway).toBe((-out[2].oneway!) as 1 | -1);
    expect(out[0].oneway).toBeUndefined();
    expect(stronglyConnected(out, true)).toBe(true);
  });

  test('a pair turned against each other, their points running opposite ways: opposite ways still', () => {
    const roads = [street([0, 0], [300, 0], ['out', 'out']), street([0, 160], [300, 160], ['out', 'out']), street([100, 0], [100, 160]), street([150, 160], [150, 0])];
    const { roads: out, pairs } = oneWayPairs(roads, [0, 1, 2, 3], [125, 80], 2);
    expect(pairs).toEqual([[2, 3]]);
    // (Along their points the same way: so cars go opposite ways.)
    expect(out[3].oneway).toBe(out[2].oneway);
  });

  test('no pair of a dead end, a lane too far apart, or where a one-way would cut a place off', () => {
    const base = [street([0, 0], [300, 0], ['out', 'out']), street([0, 160], [300, 160], ['out', 'out'])];
    // (Too far apart: 100 m.)
    expect(oneWayPairs([...base, street([100, 0], [100, 160]), street([200, 0], [200, 160])], [0, 1, 2, 3], [150, 80], 2).pairs).toEqual([]);
    // (One a dead end.)
    expect(oneWayPairs([...base, street([100, 0], [100, 160]), street([150, 0], [150, 120], ['join', 'dead'])], [0, 1, 2, 3], [125, 80], 2).pairs).toEqual([]);
    // (None if `most` is 0.)
    expect(oneWayPairs([...base, street([100, 0], [100, 160]), street([150, 0], [150, 160])], [0, 1, 2, 3], [125, 80], 0).pairs).toEqual([]);
  });

  test('a lane made one of a pair is a street, as wide', () => {
    const lane = (a: Pt, b: Pt): RoadPlan => ({ ...street(a, b), kind: 'lane', width: 6 });
    const roads = [street([0, 0], [300, 0], ['out', 'out']), street([0, 160], [300, 160], ['out', 'out']), lane([100, 0], [100, 160]), lane([150, 0], [150, 160])];
    const { roads: out } = oneWayPairs(roads, [0, 1, 2, 3], [125, 80], 1);
    expect(out[2]).toMatchObject({ kind: 'street', width: 7 });
    expect(out[3]).toMatchObject({ kind: 'street', width: 7 });
  });
});
