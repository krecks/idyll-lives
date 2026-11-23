/**
 * The road hierarchy's fixture (`render/roadHierarchyFixture.ts`; the simulation's `tests/sim/travel_hierarchy.rs`
 * lays out the same): every kind of road in its place, the motorway's carriageways, crossings and decks, and the
 * half-diamond's ramps.
 */
import { describe, expect, test } from 'bun:test';
import type { RoadPlan } from '../src/core/protocol';
import { ROAD_KIND } from '../src/core/roadKinds';
import { lengths, pointAt } from '../src/game/roads';
import * as F from '../src/render/roadHierarchyFixture';

describe('the road hierarchy fixture', () => {
  const h = F.roadHierarchy();
  const len = (r: RoadPlan) => lengths(r.points).at(-1)!;

  test('every road in its place', () => {
    expect(h.roads.map((r) => [r.kind, r.oneway ?? 0])).toEqual([
      ['motorway', 1],
      ['motorway', -1],
      ['street', 0],
      ['ramp', 1],
      ['ramp', 1],
      ['avenue', 0],
      ['street', 0],
      ['street', 0],
      ['street', 0],
      ['street', 1],
      ['street', 1],
      ['street', 1],
      ['street', -1],
    ]);
    for (const r of h.roads) {
      expect(r.width).toBe(ROAD_KIND[r.kind ?? 'street'].width);
      for (const [x, z] of r.points) expect(x >= 0 && x <= h.width && z >= 0 && z <= h.depth).toBe(true);
    }
  });

  test('the motorway: two carriageways 9 m apart, over the street and the river', () => {
    const [east, west] = [h.roads[F.MOTORWAY_EAST], h.roads[F.MOTORWAY_WEST]];
    expect(east.points[0][1] - west.points[0][1]).toBe(9);
    const street = h.roads[F.BRIDGE_STREET];
    for (const [i, m] of [east, west].entries()) {
      const { road, s } = m.crosses![0];
      expect(road).toBe(F.BRIDGE_STREET);
      expect(pointAt(m.points, s).p[0]).toBeCloseTo(F.STREET_X, 6);
      const [r0, r1] = m.raised![0];
      expect(r0 < s && s < r1).toBe(true);
      expect(m.bridge![0]).toBeLessThan(F.RIVER_X - F.MOTORWAY_X0);
      expect(m.bridge![1]).toBeGreaterThan(F.RIVER_X + F.RIVER_W - F.MOTORWAY_X0);
      // (And the street knows it passes under.)
      expect(street.crosses![i].road).toBe(i === 0 ? F.MOTORWAY_EAST : F.MOTORWAY_WEST);
    }
  });

  test('laid for traffic on the right: each carriageway the other on the left of the way its cars go, its ramp to its right', () => {
    const middle = (r: RoadPlan) => pointAt(r.points, len(r) / 2);
    for (const [ri, other, ramp] of [
      [F.MOTORWAY_EAST, F.MOTORWAY_WEST, F.OFF_RAMP],
      [F.MOTORWAY_WEST, F.MOTORWAY_EAST, F.ON_RAMP],
    ] as const) {
      const r = h.roads[ri];
      const { p, t } = middle(r);
      // (The way its cars go, and their left: (z, -x) of it.)
      const go = [t[0] * r.oneway!, t[1] * r.oneway!];
      const left = [go[1], -go[0]];
      const q = middle(h.roads[other]).p;
      expect((q[0] - p[0]) * left[0] + (q[1] - p[1]) * left[1]).toBeCloseTo(9, 6);
      // (The ramp's middle, out beside it: to its right.)
      const m = middle(h.roads[ramp]).p;
      expect((m[0] - p[0]) * left[0] + (m[1] - p[1]) * left[1]).toBeLessThan(-4);
    }
    // Eastbound (+x) to the south (+z: north is -z).
    expect(F.EAST_Z).toBeGreaterThan(F.WEST_Z);
  });

  test('the half-diamond: ramps 110 to 140 m, merging at one end and meeting the street at the other', () => {
    const [off, on] = [h.roads[F.OFF_RAMP], h.roads[F.ON_RAMP]];
    expect(off.ends).toEqual(['merge', 'join']);
    expect(on.ends).toEqual(['join', 'merge']);
    for (const r of [off, on]) {
      expect(len(r)).toBeGreaterThanOrEqual(110);
      expect(len(r)).toBeLessThanOrEqual(140);
    }
    expect(off.points[0][1]).toBe(F.EAST_Z);
    expect(on.points.at(-1)![1]).toBe(F.WEST_Z);
    expect(off.points.at(-1)).toEqual([F.STREET_X, F.EAST_Z + 25]);
    expect(on.points[0]).toEqual([F.STREET_X, F.WEST_Z - 25]);
  });
});
