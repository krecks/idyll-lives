/**
 * Junctions as drawn (`render/shared/junctions.ts`, meshes in `render/shared/roads.ts`): worked out
 * from the roads for a test's bare structure (`fixtures/junctions.ts`), or taken from the
 * simulation's `WorldStructure.junctions`; each one's pad of asphalt with a kerb return round each
 * corner, the sidewalk turning the corner, where the roads' own sidewalks stop for it (and for a
 * country road's mouth), and the stop lines and crosswalks on its arms.
 */
import { describe, expect, test } from 'bun:test';
import type { Junction, RoadPlan, WorldStructure } from '../src/core/protocol';
import { SIDEWALK, trimJoins } from '../src/game/roads';
import { Geo } from '../src/render/shared/geometry';
import { CROSSWALK, JunctionCuts, junctionsOf, KERB_RETURN, planJunctions, STOP_PAST, type JunctionPlan } from '../src/render/shared/junctions';
import { Kerb, MARK_Y, roadIndex, roadMeshes, ROAD_Y, WALK_Y, walkStops } from '../src/render/shared/roads';
import { deriveJunctions } from './fixtures/junctions';

const line = (x0: number, z0: number, x1: number, z1: number, n: number): [number, number][] => Array.from({ length: n + 1 }, (_, i) => [x0 + ((x1 - x0) * i) / n, z0 + ((z1 - z0) * i) / n]);
/** A street along x at z = 50; a lane off it to the south (+z), begun at its kerb (`trimJoins`). */
const street: RoadPlan = { points: line(0, 50, 100, 50, 100), width: 7, ends: ['out', 'out'] };
const T = trimJoins([street, { points: line(50, 50, 50, 90, 40), width: 7, ends: ['join', 'dead'], kind: 'lane' }]);
/** Two streets crossing at (50, 50). */
const CROSS: RoadPlan[] = [street, { points: line(50, 0, 50, 100, 100), width: 7, ends: ['out', 'out'] }];
/** A country road (no sidewalks) from the south ending on the street's middle, as the worlds' do. */
const COUNTRY: RoadPlan[] = [street, { points: line(50, 50, 50, 100, 50), width: 7, ends: ['join', 'out'], kind: 'country' }];
/** Half a street's carriageway (kerb to kerb, the sidewalks either side). */
const HALF = 3.5 - SIDEWALK;

/** The triangles of a mesh lying flat at height `y`, as (x, z) corners. */
function flat(g: Geo, y: number): [number, number][][] {
  const out: [number, number][][] = [];
  for (let i = 0; i < g.idx.length; i += 3) {
    const v = [g.idx[i], g.idx[i + 1], g.idx[i + 2]];
    if (v.some((k) => Math.abs(g.pos[k * 3 + 1] - y) > 1e-4) || g.nor[v[0] * 3 + 1] < 0.99) continue;
    out.push(v.map((k): [number, number] => [g.pos[k * 3], g.pos[k * 3 + 2]]));
  }
  return out;
}

/** Whether a point is on one of the triangles. */
function covered(tris: [number, number][][], x: number, z: number): boolean {
  return tris.some(([a, b, c]) => {
    const s = (p: [number, number], q: [number, number]) => (q[0] - p[0]) * (z - p[1]) - (q[1] - p[1]) * (x - p[0]);
    const [d0, d1, d2] = [s(a, b), s(b, c), s(c, a)];
    return (d0 >= -1e-9 && d1 >= -1e-9 && d2 >= -1e-9) || (d0 <= 1e-9 && d1 <= 1e-9 && d2 <= 1e-9);
  });
}

/** The junctions (as drawn) of a set of roads. */
function plansOf(roads: RoadPlan[]): JunctionPlan[] {
  return planJunctions(deriveJunctions(roads), roads);
}

/** The junction (as drawn) of a set of roads: the only one. */
function only(roads: RoadPlan[]): JunctionPlan {
  const plans = plansOf(roads);
  expect(plans.length).toBe(1);
  return plans[0];
}

describe('junctions worked out from the roads', () => {
  test('a lane off a street is a T: three arms, its middle on the street', () => {
    const [j] = deriveJunctions(T);
    expect(deriveJunctions(T).length).toBe(1);
    // On the street's middle line, not at the lane's end on the kerb.
    expect(j.x).toBeCloseTo(50, 1);
    expect(j.z).toBeCloseTo(50, 1);
    expect(j.roundabout).toBeUndefined();
    const ways = j.arms.map((a) => [Math.round(a.dir[0]), Math.round(a.dir[1]), a.road]);
    expect(ways).toEqual(expect.arrayContaining([[1, 0, 0], [-1, 0, 0], [0, 1, 1]]));
    for (const a of j.arms) {
      expect(a.width).toBeCloseTo(2 * HALF);
      // The crosswalk out past the box, the stop line past it; giving way, as the simulation has a
      // lane meeting a street: no lights.
      expect(a.walkTo - a.walkFrom).toBeCloseTo(CROSSWALK);
      expect(a.stop - a.walkTo).toBeCloseTo(STOP_PAST);
      expect(a.walkFrom).toBeGreaterThanOrEqual(6);
      expect(a.light).toBeNull();
    }
    // A zebra across the street on one side of the lane's mouth (its west arm, the first round from
    // z towards x), none across the lane.
    const zebras = j.arms.filter((a) => a.crosswalk).map((a) => [Math.round(a.dir[0]), Math.round(a.dir[1])]);
    expect(zebras).toEqual([[-1, 0]]);
  });

  test('a crossroads is lit, every arm with a zebra; a country T lit, but for the country road its arms', () => {
    const [x] = deriveJunctions(CROSS);
    expect(x.arms.every((a) => a.light !== null && a.crosswalk)).toBe(true);
    const [c] = deriveJunctions(COUNTRY);
    expect(c.arms.every((a) => a.light !== null)).toBe(true);
    expect(c.arms.map((a) => a.crosswalk)).toEqual(c.arms.map((a) => a.road === 0));
  });

  test('two quiet Ts close together along a street share one zebra, between them; far apart, one each', () => {
    // Two lanes off the street 17 m apart, either side of it or on one side: one zebra, on the
    // stretch between them (sim-core `signal_at`); 60 m apart, a zebra at each.
    const lane = (x: number, south: boolean): RoadPlan => ({ points: south ? line(x, 50, x, 90, 40) : line(x, 50, x, 10, 40), width: 7, ends: ['join', 'dead'], kind: 'lane' });
    for (const south of [true, false]) {
      const js = deriveJunctions(trimJoins([street, lane(40, true), lane(57, south)]));
      expect(js.length).toBe(2);
      const zebras = js.flatMap((j) => j.arms.filter((a) => a.crosswalk).map((a) => j.x + a.dir[0] * (a.walkFrom + CROSSWALK / 2)));
      expect(zebras.length).toBe(1);
      expect(zebras[0]).toBeGreaterThan(40);
      expect(zebras[0]).toBeLessThan(57);
    }
    const far = deriveJunctions(trimJoins([street, lane(20, true), lane(80, false)]));
    expect(far.map((j) => j.arms.filter((a) => a.crosswalk).length)).toEqual([1, 1]);
  });

  test('country roads meeting give way, three or four; a country road meeting a street is lit', () => {
    const road = (pts: [number, number][]): RoadPlan => ({ points: pts, width: 7, ends: ['out', 'out'], kind: 'country' });
    const [t] = deriveJunctions([road(line(0, 50, 100, 50, 100)), { ...road(line(50, 50, 50, 100, 50)), ends: ['join', 'out'] }]);
    expect(t.arms.length).toBe(3);
    expect(t.arms.every((a) => a.light === null && !a.crosswalk)).toBe(true);
    const [x] = deriveJunctions([road(line(0, 50, 100, 50, 100)), road(line(50, 0, 50, 100, 100))]);
    expect(x.arms.length).toBe(4);
    expect(x.arms.every((a) => a.light === null)).toBe(true);
    const [c] = deriveJunctions(COUNTRY);
    expect(c.arms.every((a) => a.light !== null)).toBe(true);
  });

  test('two streets crossing are a crossroads; a country road joining a street a country T', () => {
    const [x] = deriveJunctions(CROSS);
    expect(x.arms.length).toBe(4);
    const c = only(COUNTRY);
    expect(c.arms.length).toBe(3);
    const arm = c.arms.find((a) => a.road === 1)!;
    // A country road's carriageway is its whole width, and it has no sidewalks.
    expect(arm.half).toBeCloseTo(3.5);
    expect(arm.walks).toBe(false);
    expect(c.arms.filter((a) => a.walks).length).toBe(2);
  });

  test('a roundabout keeps its own look: no pad, markings or corners', () => {
    const list: Junction[] = [{ x: 0, z: 0, roundabout: true, arms: [0, 1, 2].map((k) => ({ road: 0, dir: [Math.cos(k * 2), Math.sin(k * 2)], width: 4.3, walkFrom: 6, walkTo: 9, stop: 9.5, light: null })) }];
    expect(planJunctions(list, [street])).toEqual([]);
  });
});

describe('junctions as drawn', () => {
  test('a T: a kerb return round each corner off the lane, none across the street', () => {
    const j = only(T);
    expect(j.corners.length).toBe(2);
    for (const c of j.corners) {
      // The arcs' middles: `KERB_RETURN` out from both kerbs, south of the street either side of the lane.
      expect(Math.abs(c.o[0])).toBeCloseTo(HALF + KERB_RETURN, 1);
      expect(c.o[1]).toBeCloseTo(HALF + KERB_RETURN, 1);
      expect(c.walk).toBe(true);
    }
    const { surface, walk } = roadMeshes(T, null, 0, {}, 'both', plansOf(T));
    const pad = flat(surface, ROAD_Y);
    const walks = flat(walk, WALK_Y);
    for (const sx of [1, -1]) {
      const [ox, oz] = [50 + sx * (HALF + KERB_RETURN), 50 + HALF + KERB_RETURN];
      // Round the corner: asphalt on the road's side of the arc, a sidewalk on the land's.
      const out = (r: number): [number, number] => [ox - sx * r * Math.SQRT1_2, oz - r * Math.SQRT1_2];
      expect(covered(pad, ...out(KERB_RETURN + 0.3))).toBe(true);
      expect(covered(walks, ...out(KERB_RETURN - 0.4))).toBe(true);
      // No sidewalk on the carriageway past the arc, and the square corner of old is gone.
      expect(covered(walks, ...out(KERB_RETURN + 0.3))).toBe(false);
      expect(covered(walks, 50 + sx * (HALF + 0.3), 50 + HALF + 0.3)).toBe(false);
    }
    // The street's sidewalk across from the lane runs on unbroken.
    const cuts = JunctionCuts.of(plansOf(T));
    const north = new Kerb(street, 0, 1, roadIndex(T), cuts);
    expect([40, 50, 60].every((s) => north.open(s))).toBe(true);
    // On the lane's side it stops where the arcs begin, and runs again past them.
    const south = new Kerb(street, 0, -1, roadIndex(T), cuts);
    expect(south.open(50 + HALF + KERB_RETURN - 0.3)).toBe(false);
    expect(south.open(50 + HALF + KERB_RETURN + 0.3)).toBe(true);
  });

  test('a crossroads: four corners, a pad joining the four carriageways', () => {
    const j = only(CROSS);
    expect(j.corners.length).toBe(4);
    const cross = plansOf(CROSS);
    const pad = flat(roadMeshes(CROSS, null, 0, {}, 'both', cross).surface, ROAD_Y);
    // The middle, and out along each arm to its crosswalk.
    for (const [dx, dz] of [[0, 0], [8, 0], [-8, 0], [0, 8], [0, -8]]) expect(covered(pad, 50 + dx, 50 + dz)).toBe(true);
    const walks = flat(roadMeshes(CROSS, null, 0, {}, 'both', cross).walk, WALK_Y);
    // Each corner's sidewalk at the land's corner.
    for (const [sx, sz] of [[1, 1], [1, -1], [-1, 1], [-1, -1]]) expect(covered(walks, 50 + sx * 3.3, 50 + sz * 3.3)).toBe(true);
  });

  test("a country T: the street's sidewalk stops at the country road's mouth, and only there", () => {
    const stops = walkStops(COUNTRY);
    // The country road begun at the street's kerb (the sidewalks don't stop for it across the street).
    expect(stops[1].points[0][1]).toBeGreaterThanOrEqual(50 + HALF - 0.01);
    const index = roadIndex(stops);
    const plans = planJunctions(deriveJunctions(COUNTRY), COUNTRY);
    const { walk } = roadMeshes(stops, null, 0, {}, 'sidewalks', plans);
    const walks = flat(walk, WALK_Y);
    // South (the country road's side): none across its mouth; north: unbroken.
    expect(covered(walks, 50, 50 + HALF + SIDEWALK / 2)).toBe(false);
    expect(covered(walks, 50, 50 - HALF - SIDEWALK / 2)).toBe(true);
    const cuts = JunctionCuts.of(plans);
    expect(new Kerb(street, 0, -1, index, cuts).open(50)).toBe(false);
    expect(new Kerb(street, 0, 1, index, cuts).open(50)).toBe(true);
    // The kerb returns round from the street's kerb into the country road's edge.
    const j = plans[0];
    expect(j.corners.length).toBe(2);
    for (const c of j.corners) {
      expect(Math.abs(c.o[0])).toBeCloseTo(3.5 + KERB_RETURN, 1);
      expect(c.o[1]).toBeCloseTo(HALF + KERB_RETURN, 1);
    }
    // No sidewalk along the country road itself, past the corners.
    expect(covered(walks, 50 + 3.5 + 0.5, 50 + 14)).toBe(false);
  });
});

describe('road markings', () => {
  /** The paint lying on the road, as (x, z) triangles. */
  const paint = (roads: RoadPlan[], plans: JunctionPlan[] = plansOf(roads)) => flat(roadMeshes(roads, null, 0, { marks: new Geo() }, 'surface', plans).marks!, MARK_Y);

  test("a stop line across the half coming in at the arm's stop, a zebra across the whole between its crosswalk's ends", () => {
    const j = only(CROSS);
    const lane = j.arms.find((a) => a.road === 1 && a.dir[1] > 0)!;
    const marks = paint(CROSS);
    // The south arm runs south (+z): those coming in drive north on its east (+x) half (on the right).
    const z = 50 + lane.stop - 0.15;
    expect(covered(marks, 50 + HALF / 2, z)).toBe(true);
    expect(covered(marks, 50 - HALF / 2, z)).toBe(false);
    expect(covered(marks, 50 + HALF / 2, 50 + lane.stop + 0.1)).toBe(false);
    // The zebra's stripes run the way of the arm, across the whole carriageway.
    const mid = 50 + (lane.walkFrom + lane.walkTo) / 2;
    const across = Array.from({ length: 81 }, (_, k) => 50 - HALF + (2 * HALF * k) / 80).map((x) => covered(marks, x, mid));
    const stripes = across.filter((on, k) => on && !across[k - 1]).length;
    expect(stripes).toBeGreaterThanOrEqual(3);
    expect(across.some((on, k) => on && k < 20)).toBe(true);
    expect(across.some((on, k) => on && k > 60)).toBe(true);
    // Nothing in the junction's box.
    expect(covered(marks, 50, 50 + 2)).toBe(false);
  });

  test("a T giving way: a zebra and its stop line on one of the street's arms, nothing across the lane", () => {
    const j = only(T);
    const [west, east, lane] = [j.arms.find((a) => a.dir[0] < -0.9)!, j.arms.find((a) => a.dir[0] > 0.9)!, j.arms.find((a) => a.road === 1)!];
    const marks = paint(T);
    // (Painted anywhere across the carriageway, `along` m out the arm's way `dir`.)
    const across = (dir: number[], along: number) =>
      Array.from({ length: 41 }, (_, k) => -HALF + (2 * HALF * k) / 40).some((c) => covered(marks, 50 + dir[0] * along - dir[1] * c, 50 + dir[1] * along + dir[0] * c));
    // West: the zebra across the street, and the stop line on its south half (those coming in east).
    expect(across(west.dir, (west.walkFrom + west.walkTo) / 2)).toBe(true);
    expect(covered(marks, 50 - west.stop + 0.15, 50 + HALF / 2)).toBe(true);
    // East and across the lane: nothing.
    expect(across(east.dir, (east.walkFrom + east.walkTo) / 2)).toBe(false);
    expect(across(east.dir, east.stop - 0.15)).toBe(false);
    expect(across(lane.dir, (lane.walkFrom + lane.walkTo) / 2)).toBe(false);
    expect(across(lane.dir, lane.stop - 0.15)).toBe(false);
  });

  test("on a country road's arm only a stop line, and only while it's lit", () => {
    const plans = planJunctions(deriveJunctions(COUNTRY), COUNTRY);
    const c = plans[0].arms.find((a) => a.road === 1)!;
    const marks = paint(COUNTRY, plans);
    expect(covered(marks, 50 + 1.5, 50 + c.stop - 0.15)).toBe(true);
    expect(covered(marks, 50, 50 + (c.walkFrom + c.walkTo) / 2)).toBe(false);
    const unlit = planJunctions(deriveJunctions(COUNTRY).map((j) => ({ ...j, arms: j.arms.map((a) => ({ ...a, light: null })) })), COUNTRY);
    expect(covered(paint(COUNTRY, unlit), 50 + 1.5, 50 + c.stop - 0.15)).toBe(false);
  });

  test("the simulation's junctions are drawn when it sends them, and none without", () => {
    const world = { width: 100, depth: 100, meta: { roads: CROSS } } as unknown as WorldStructure;
    const derived = deriveJunctions(CROSS);
    // Not sent (an older sim, a test's bare structure): nothing to draw.
    expect(junctionsOf(world, CROSS)).toEqual([]);
    // Given (the sim's, its stop lines further out): those.
    const given: Junction[] = derived.map((j) => ({ ...j, arms: j.arms.map((a) => ({ ...a, walkFrom: 8, walkTo: 11, stop: 12 })) }));
    const plans = junctionsOf({ ...world, junctions: given }, CROSS);
    expect(plans[0].arms.map((a) => a.stop)).toEqual([12, 12, 12, 12]);
    expect(plans[0].corners.length).toBe(4);
    const marks = paint(CROSS, plans);
    // The east arm (+x): those coming in drive west on its north (-z) half.
    expect(covered(marks, 50 + 12 - 0.15, 50 - HALF / 2)).toBe(true);
    expect(covered(marks, 50 + derived[0].arms[0].stop - 0.15, 50 - HALF / 2)).toBe(false);
  });
});
