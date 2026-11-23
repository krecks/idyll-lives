// One-way roads never cut a place off (`game/oneway.ts`): the roads' network as the simulation joins them, each
// one-way road gone along its way only, every place a car comes onto the roads reaching every other and back
// (sim-core `Graph::strongly_connected`, on the same fixture: `travel_hierarchy.rs`
// `one_way_roads_keep_every_place_reachable`); and the generator's one-way pairs.
import { describe, expect, test } from 'bun:test';
import type { RoadPlan } from '../src/core/protocol';
import { cutOff, oneWayPairs, roadGraph, stronglyConnected } from '../src/game/oneway';
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
    expect(cutOff(roads)).toEqual([]);
  });

  test("the fixture's loop with a side turned round: a corner cars come to and can't leave", () => {
    const { roads } = roadHierarchy();
    const turned = roads.map((r, i) => (i === LOOP[3] ? { ...r, oneway: 1 as const } : r));
    expect(stronglyConnected(turned)).toBe(false);
    // (The loop's roads are what is cut off; the rest of the town is not.)
    const cut = cutOff(turned);
    expect(cut.length).toBeGreaterThan(0);
    expect(cut.every((r) => (LOOP as readonly number[]).includes(r))).toBe(true);
    // Two-way again: all well.
    expect(stronglyConnected(roads.map((r, i) => (i === LOOP[3] ? { ...r, oneway: undefined } : r)))).toBe(true);
  });

  test("the fixture's graph: the motorway meets nothing it passes over; the crosses meet the avenue", () => {
    const { roads } = roadHierarchy();
    const g = roadGraph(roads);
    // No node on a carriageway but its ends and the ramps' merges.
    const onMotorway = g.nodes.filter((_, n) => g.adj[n].some((e) => roads[g.edges[e].road].kind === 'motorway'));
    expect(onMotorway.length).toBe(4 + 2);
    // Each cross street crosses the avenue: a node of four stretches.
    expect(g.adj.filter((list) => list.length === 4).length).toBeGreaterThanOrEqual(2);
  });

  test('without a one-way road it holds', () => {
    expect(stronglyConnected(ladder())).toBe(true);
  });

  test('a one-way road out of a dead end cuts it off', () => {
    const roads = [...ladder(), street([100, 50], [100, 90], ['join', 'dead'], -1)];
    expect(stronglyConnected(roads)).toBe(false);
    expect(cutOff(roads)).toContain(6);
  });

  test('a pair of one-way streets either way round the ladder holds; the same way does not', () => {
    const pair = (a: 1 | -1, b: 1 | -1) => ladder().map((r, i) => (i === 3 ? { ...r, oneway: a } : i === 4 ? { ...r, oneway: b } : r));
    expect(stronglyConnected(pair(1, -1))).toBe(true);
    expect(stronglyConnected(pair(-1, 1))).toBe(true);
    // (Both the same way: the first rung and the street round the end still go both ways, so it holds.)
    expect(stronglyConnected(pair(1, 1))).toBe(true);
    // Every rung and the end one-way north: nothing comes back south.
    const north = ladder().map((r, i) => (i >= 2 ? { ...r, oneway: 1 as const } : r));
    expect(stronglyConnected(north)).toBe(false);
  });

  test('two networks: each on its own', () => {
    const far = [street([500, 0], [600, 0], ['dead', 'join']), street([600, 0], [600, 60], ['join', 'dead'], 1)];
    expect(stronglyConnected([...ladder(), ...far])).toBe(false);
    // (Only the one-way road's far end is cut off: its own road is all that meets it.)
    expect(cutOff([...ladder(), ...far])).toEqual([7]);
  });

  test('a road crossing another part way along meets it there', () => {
    // (A one-way road across the ladder's long streets, out of town north and south: cars drive north along it, and
    // come back south by the ladder's end; its ends out of town reached and left.)
    const across = street([120, -40], [120, 90], ['out', 'out'], 1);
    const g = roadGraph([...ladder(), across]);
    const at = (x: number, z: number) => g.nodes.findIndex(([nx, nz]) => Math.hypot(nx - x, nz - z) < 1);
    expect(at(120, 0)).toBeGreaterThanOrEqual(0);
    expect(at(120, 50)).toBeGreaterThanOrEqual(0);
    // The south end is a way in only, the north a way out only: each can't be driven to and from the other.
    expect(stronglyConnected([...ladder(), across])).toBe(false);
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
    expect(stronglyConnected(out)).toBe(true);
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
