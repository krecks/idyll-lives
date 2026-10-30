/**
 * Roads as drawn (`render/shared/roads.ts`): a lane off a road and a dead end. The sidewalks stop
 * where the lane meets the road (none across either road), the dead end gets its turning circle,
 * and an older town's straight streets are roads of their own width.
 */
import { describe, expect, test } from 'bun:test';
import type { RoadPlan, WorldStructure } from '../src/core/protocol';
import { Kerb, RoadIndex, roadMeshes, roadTiles, SIDEWALK, sidewalkCuts, townRoads, WALK_Y } from '../src/render/shared/roads';

const line = (x0: number, z0: number, x1: number, z1: number, n: number): [number, number][] => Array.from({ length: n + 1 }, (_, i) => [x0 + ((x1 - x0) * i) / n, z0 + ((z1 - z0) * i) / n]);
/** A road along x at z = 50, and a lane off it to the south (+z) ending in a dead end. */
const road: RoadPlan = { points: line(0, 50, 100, 50, 100), width: 7, ends: ['out', 'out'] };
const lane: RoadPlan = { points: line(50, 50, 50, 90, 40), width: 6, ends: ['join', 'dead'] };

describe('roads as drawn', () => {
  test('sidewalks stop where a lane meets the road', () => {
    const { walk } = roadMeshes([road, lane]);
    const index = new RoadIndex([road, lane]);
    for (let i = 0; i < walk.pos.length; i += 3) {
      const [x, y, z] = [walk.pos[i], walk.pos[i + 1], walk.pos[i + 2]];
      if (y < WALK_Y - 0.001) continue;
      // Nothing of a sidewalk on the carriageway of either road (a little in from their kerbs).
      for (const [k, r] of [road, lane].entries()) expect(index.distance(k, x, z)).toBeGreaterThan(r.width / 2 - SIDEWALK - 0.01);
    }
    // The road's sidewalk on the lane's side is cut across the lane's mouth, kept elsewhere and across the road.
    const cuts = sidewalkCuts([road, lane]);
    const south = new Kerb(road, 0, -1, cuts);
    expect(south.open(50)).toBe(false);
    expect(south.open(20)).toBe(true);
    expect(new Kerb(road, 0, 1, cuts).open(50)).toBe(true);
    // The lane's sidewalks begin at the road's.
    expect(new Kerb(lane, 1, 1, cuts).open(2)).toBe(false);
    expect(new Kerb(lane, 1, 1, cuts).open(8)).toBe(true);
  });

  test('a dead end has a turning circle', () => {
    const kerb = new Kerb(lane, 1, 1, sidewalkCuts([road, lane]));
    expect(kerb.inTurning(50, 90)).toBe(true);
    expect(kerb.open(kerb.length - 0.5)).toBe(false);
    expect(kerb.open(kerb.length / 2)).toBe(true);
    // Its tiles: wider than the lane at the end.
    const tiles = roadTiles([lane], 100, 100);
    expect(tiles[90 * 100 + 54]).toBe(1);
    expect(tiles[70 * 100 + 54]).toBe(0);
  });

  test("an older town's streets are straight roads", () => {
    const world = { width: 40, depth: 30, meta: { streets: [{ x: 0, z: 12, w: 40, d: 6 }] } } as unknown as WorldStructure;
    expect(townRoads(world)).toEqual([{ points: [[0, 15], [40, 15]], width: 6, ends: ['out', 'out'] }]);
    // Things on the sidewalk face the road: along +x, the left (smaller z) side faces +z.
    const kerb = new Kerb(townRoads(world)[0], 0, 1, new RoadIndex(townRoads(world)));
    expect(kerb.yaw(10)).toBeCloseTo(0);
    expect(kerb.at(10, 0)).toEqual([10, 12]);
  });
});
