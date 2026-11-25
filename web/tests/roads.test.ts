/**
 * Roads as drawn (`render/shared/roads.ts`): a lane off a road and a dead end. The sidewalks stop
 * where the lane meets the road (none across either road), the dead end gets its turning circle,
 * and an older town's straight streets are roads of their own width. An avenue's median, kerbed and grassed, cut
 * at its junctions; the overview's decks and barrier (`townRoadMesh`), on the road hierarchy's fixture.
 */
import { describe, expect, test } from 'bun:test';
import { line } from './support/geometry';
import type { RoadPlan, WorldStructure } from '../src/core/protocol';
import { lengths, nearest, trimJoins } from '../src/game/roads';
import { Kerb, KERB_Y, RoadIndex, roadMeshes, roadTiles, SIDEWALK, townRoadMesh, townRoads, WALK_Y } from '../src/render/shared/roads';
import { Geo } from '../src/render/shared/geometry';
import { junctionStretches } from '../src/render/shared/markings';
import { MOTORWAY } from '../src/render/shared/motorway';
import * as F from '../src/render/roadHierarchyFixture';
import { JunctionCuts } from '../src/render/shared/junctions';
import { deriveJunctions, plansOf } from './fixtures/junctions';

/** A road along x at z = 50, and a lane off it to the south (+z) ending in a dead end: beginning at the road's kerb, as the town's lanes do (`trimJoins`). */
const road: RoadPlan = { points: line(0, 50, 100, 50, 100), width: 7, ends: ['out', 'out'] };
const [, lane] = trimJoins([road, { points: line(50, 50, 50, 90, 40), width: 6, ends: ['join', 'dead'] }]);
const index = new RoadIndex([road, lane]);

describe('roads as drawn', () => {
  test('sidewalks stop where a lane meets the road', () => {
    const { walk } = roadMeshes([road, lane], null, 0, {}, 'both', plansOf([road, lane]));
    // The lane begins at the road's kerb (its first point past it; they're a metre apart).
    expect(lane.points[0][1]).toBeGreaterThanOrEqual(50 + 3.5 - SIDEWALK);
    expect(lane.points[0][1]).toBeLessThan(50 + 3.5 - SIDEWALK + 1);
    for (let i = 0; i < walk.pos.length; i += 3) {
      const [x, y, z] = [walk.pos[i], walk.pos[i + 1], walk.pos[i + 2]];
      if (y < WALK_Y - 0.001) continue;
      // Nothing of a sidewalk on the carriageway of either road (a little in from their kerbs).
      for (const [k, r] of [road, lane].entries()) expect(index.distance(k, x, z)).toBeGreaterThan(r.width / 2 - SIDEWALK - 0.01);
    }
    // The road's sidewalk on the lane's side is cut across the lane's mouth, kept elsewhere and across the road.
    const cuts = JunctionCuts.of(plansOf([road, lane]));
    const south = new Kerb(road, 0, -1, index, cuts);
    expect(south.open(50)).toBe(false);
    expect(south.open(20)).toBe(true);
    expect(new Kerb(road, 0, 1, index, cuts).open(50)).toBe(true);
    // The lane's sidewalks begin at the road's.
    expect(new Kerb(lane, 1, 1, index, cuts).open(0.5)).toBe(false);
    expect(new Kerb(lane, 1, 1, index, cuts).open(6)).toBe(true);
  });

  test('a sidewalk runs up to a crossing on both sides of it', () => {
    // Two long straight roads crossing: a point each end, so a segment spans the whole road.
    const a: RoadPlan = { points: line(0, 50, 100, 50, 1), width: 7, ends: ['out', 'out'] };
    const b: RoadPlan = { points: line(50, 0, 50, 100, 1), width: 7, ends: ['out', 'out'] };
    const { walk } = roadMeshes([a, b], null, 0, {}, 'both', plansOf([a, b]));
    const xs: number[] = [];
    for (let i = 0; i < walk.pos.length; i += 3) {
      const [x, y, z] = [walk.pos[i], walk.pos[i + 1], walk.pos[i + 2]];
      // Along `a`'s north sidewalk (its band is `a`'s kerb in from its edge), not `b`'s.
      if (Math.abs(y - WALK_Y) > 0.001 || z < 52.15 || z > 53.5 || x < 0 || x > 100) continue;
      xs.push(x);
    }
    // West and east of the crossing, not cut off whole.
    expect(xs.some((x) => x < 46)).toBe(true);
    expect(xs.some((x) => x > 54)).toBe(true);
    // Up to the crossing road's edge on either side, not short of it.
    expect(xs.some((x) => x > 45.8 && x < 46.6)).toBe(true);
    expect(xs.some((x) => x > 53.4 && x < 54.2)).toBe(true);
  });

  test('a dead end has a turning circle', () => {
    const kerb = new Kerb(lane, 1, 1, index);
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

describe("an avenue's median", () => {
  const h = F.roadHierarchy();
  const plans = plansOf(h.roads);
  const avenue = h.roads[F.AVENUE];
  const drawn = () => {
    const [walk, median] = [new Geo(), new Geo()];
    roadMeshes(h.roads, null, 0, { walk, median }, 'sidewalks', plans);
    return { walk, median };
  };
  const verts = (g: Geo) => Array.from({ length: g.pos.length / 3 }, (_, i): [number, number, number] => [g.pos[i * 3], g.pos[i * 3 + 1], g.pos[i * 3 + 2]]);

  test('grass down its middle, as wide as the median less its kerbs, cut where its junctions are', () => {
    const { median } = drawn();
    const grass = verts(median);
    expect(grass.length).toBeGreaterThan(0);
    const cut = junctionStretches(avenue, plans, lengths(avenue.points));
    for (const [x, y, z] of grass) {
      expect(y).toBeCloseTo(WALK_Y, 6);
      expect(Math.abs(z - F.AVENUE_Z)).toBeLessThanOrEqual(1.3 - 0.14 + 1e-6);
      const s = nearest(avenue.points, x, z).s;
      expect(cut.some(([a, b]) => s > a + 1e-3 && s < b - 1e-3)).toBe(false);
    }
    // (None on any other road.)
    expect(grass.every(([, , z]) => Math.abs(z - F.AVENUE_Z) < 2)).toBe(true);
  });

  test("kerbs along both its sides, as high as the sidewalks' kerbs, and across its ends at a junction", () => {
    const { walk } = drawn();
    const kerbs = verts(walk).filter(([x, , z]) => Math.abs(z - F.AVENUE_Z) <= 1.3 + 1e-6 && x > 20 && x < 280);
    expect(kerbs.some(([, , z]) => z > F.AVENUE_Z + 1.25)).toBe(true);
    expect(kerbs.some(([, , z]) => z < F.AVENUE_Z - 1.25)).toBe(true);
    expect(Math.max(...kerbs.map(([, y]) => y))).toBeCloseTo(KERB_Y, 6);
    // Its nose at the junction at x = 90: a face across it, from the road up to the kerb's top.
    const [[a]] = junctionStretches(avenue, plans).filter(([p, q]) => p < 89.5 && q > 89.5);
    const nose = kerbs.filter(([x]) => Math.abs(x - (a + F.MOTORWAY_X0)) < 0.2);
    expect(nose.some(([, , z]) => z < F.AVENUE_Z - 1) && nose.some(([, , z]) => z > F.AVENUE_Z + 1)).toBe(true);
  });

  test('no median without one in its kind, nor when only the surface is drawn', () => {
    const median = new Geo();
    roadMeshes(h.roads.filter((r) => r.kind !== 'avenue'), null, 0, { median }, 'sidewalks', plans);
    expect(median.empty).toBe(true);
    roadMeshes(h.roads, null, 0, { median }, 'surface', plans);
    expect(median.empty).toBe(true);
  });
});

test("the overview draws the decks and the barrier, plainly, with the markings", () => {
  const h = F.roadHierarchy();
  const world = { width: F.WIDTH, depth: F.DEPTH, plots: [], junctions: deriveJunctions(h.roads), meta: { roads: h.roads } } as unknown as WorldStructure;
  const mesh = townRoadMesh(world);
  expect(mesh.marks).not.toBeNull();
  const ys: number[] = [];
  for (let i = 1; i < mesh.marks!.positions.length; i += 3) ys.push(mesh.marks!.positions[i]);
  // (The deck's parapet over the street; the markings at the road.)
  expect(Math.max(...ys)).toBeGreaterThan(MOTORWAY.clear + 0.5);
  // And the road climbs to the deck.
  let top = 0;
  for (let i = 1; i < mesh.surface.positions.length; i += 3) top = Math.max(top, mesh.surface.positions[i]);
  expect(top).toBeCloseTo(MOTORWAY.clear + 0.012, 3);
});
