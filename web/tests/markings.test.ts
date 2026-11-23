/**
 * The lines painted along the roads (`render/shared/markings.ts`): each kind's lanes and lines across, none on a
 * two-way street, none across a junction, the motorway's edge lines broken where a ramp leaves it, a one-way road's
 * arrows pointing the way its cars go. On the road hierarchy's fixture (`render/roadHierarchyFixture.ts`). (The
 * junctions' stop lines across every lane coming in: `junctions.test.ts`.)
 */
import { describe, expect, test } from 'bun:test';
import type { RoadPlan } from '../src/core/protocol';
import { lengths, nearest, pointAt, SIDEWALK } from '../src/game/roads';
import { Geo } from '../src/render/shared/geometry';
import { carriageway } from '../src/render/shared/junctions';
import { ARROW_EVERY, junctionStretches, laneLayout, roadMarkings } from '../src/render/shared/markings';
import { MOTORWAY } from '../src/render/shared/motorway';
import { MARK_Y } from '../src/render/shared/roads';
import * as F from '../src/render/roadHierarchyFixture';
import { plansOf } from './fixtures/junctions';

const h = F.roadHierarchy();
const plans = plansOf(h.roads);
const line = (x0: number, z0: number, x1: number, z1: number, n: number): [number, number][] => Array.from({ length: n + 1 }, (_, i) => [x0 + ((x1 - x0) * i) / n, z0 + ((z1 - z0) * i) / n]);

/** The vertices of a mesh, as (x, y, z). */
function points(g: Geo): [number, number, number][] {
  const out: [number, number, number][] = [];
  for (let i = 0; i < g.pos.length; i += 3) out.push([g.pos[i], g.pos[i + 1], g.pos[i + 2]]);
  return out;
}

/** The markings `roadMarkings` paints on the fixture's road `ri` alone (the others there for its pair and its gore). */
function marksOf(ri: number): [number, number, number][] {
  const g = new Geo();
  const only = h.roads.map((r, i) => (i === ri ? r : { ...r, kind: 'street' as const, oneway: undefined }));
  // (The pair and the ramps kept as they are: a motorway's inside and its gore are found by them.)
  for (const k of [F.MOTORWAY_EAST, F.MOTORWAY_WEST, F.OFF_RAMP, F.ON_RAMP]) only[k] = { ...h.roads[k] };
  roadMarkings(only, g, plans);
  const road = h.roads[ri];
  return points(g).filter(([x, , z]) => nearest(road.points, x, z).dist <= road.width / 2 + 0.01);
}

describe("each kind's lanes and lines", () => {
  test('a two-way street, lane or country road is left unmarked', () => {
    for (const kind of ['street', 'lane', 'country', 'trail'] as const) expect(laneLayout([{ points: line(0, 0, 50, 0, 50), width: 7, ends: ['out', 'out'], kind }], 0)).toBeNull();
    const g = new Geo();
    roadMarkings([h.roads[F.BRIDGE_STREET], h.roads[F.CROSS_WEST]], g, []);
    expect(g.empty).toBe(true);
  });

  test("an avenue: two lanes each way between its kerbs and its median, a dashed line between each two going the same way", () => {
    const lay = laneLayout(h.roads, F.AVENUE)!;
    const avenue = h.roads[F.AVENUE];
    const half = carriageway(avenue) / 2;
    expect(lay.lanes.length).toBe(4);
    // (Cars going along its points keep right: its right, -; those coming back on its left.)
    for (const l of lay.lanes) {
      expect(Math.abs(l.off)).toBeGreaterThan(1.3);
      expect(Math.abs(l.off)).toBeLessThan(half);
      expect(Math.sign(l.off)).toBe(-l.way);
    }
    expect(lay.dividers.length).toBe(2);
    for (const d of lay.dividers) expect(Math.abs(d)).toBeCloseTo((1.3 + half) / 2, 9);
    expect(lay.edges).toEqual([]);
    expect(lay.dash).toBe('town');
  });

  test("a motorway's carriageway: its lanes from its inside, towards its pair, the shoulder outside them, an edge line each side", () => {
    for (const ri of [F.MOTORWAY_EAST, F.MOTORWAY_WEST]) {
      const lay = laneLayout(h.roads, ri)!;
      const road = h.roads[ri];
      // (The pair's middle, from this one's: its left is (z, -x) of the way along, here -z.)
      const towards = ri === F.MOTORWAY_EAST ? F.WEST_Z - F.EAST_Z : F.EAST_Z - F.WEST_Z;
      const inside = -Math.sign(towards);
      expect(lay.lanes.length).toBe(2);
      for (const l of lay.lanes) expect(l.way).toBe(road.oneway!);
      // The first lane by the barrier, then the second, the shoulder (2 m or so) outside them.
      const offs = lay.lanes.map((l) => l.off * inside).sort((a, b) => b - a);
      expect(offs[0]).toBeCloseTo(road.width / 2 - MOTORWAY.inner - MOTORWAY.lane / 2, 6);
      expect(offs[0] - offs[1]).toBeCloseTo(MOTORWAY.lane, 6);
      const [inner, outer] = lay.edges.map((e) => e * inside);
      expect(inner).toBeGreaterThan(offs[0]);
      expect(outer).toBeLessThan(offs[1]);
      expect(outer + road.width / 2).toBeGreaterThan(1.2);
      expect(lay.dividers.length).toBe(1);
      expect(lay.dash).toBe('motorway');
    }
  });

  test('a ramp: one lane between two edge lines; a one-way street: one lane, no lines', () => {
    const ramp = laneLayout(h.roads, F.OFF_RAMP)!;
    expect(ramp.lanes).toEqual([{ off: 0, way: 1 }]);
    expect(ramp.edges.length).toBe(2);
    for (const e of ramp.edges) expect(Math.abs(e)).toBeLessThan(h.roads[F.OFF_RAMP].width / 2);
    expect(laneLayout(h.roads, F.LOOP[3])).toEqual({ lanes: [{ off: 0, way: -1 }], dividers: [], edges: [], dash: 'town' });
  });
});

describe('the lines along the roads', () => {
  test('none across a junction', () => {
    for (const ri of [F.AVENUE, F.OFF_RAMP, ...F.LOOP]) {
      const road = h.roads[ri];
      const cut = junctionStretches(road, plans);
      // (The avenue's: its two cross streets, and Bridge Street's end on it.)
      if (ri === F.AVENUE) expect(cut.length).toBe(3);
      for (const [x, , z] of marksOf(ri)) {
        const s = nearest(road.points, x, z).s;
        expect(cut.some(([a, b]) => s > a + 0.6 && s < b - 0.6)).toBe(false);
      }
    }
  });

  test('on the road, a hair over it', () => {
    for (const ri of [F.AVENUE, F.MOTORWAY_EAST, F.ON_RAMP]) {
      const ms = marksOf(ri);
      expect(ms.length).toBeGreaterThan(0);
      for (const [, y] of ms) expect(y).toBeCloseTo(MARK_Y, 6);
    }
  });

  test("the motorway's edge lines break where a ramp runs on it, and the ramp's where it runs on the motorway", () => {
    const east = h.roads[F.MOTORWAY_EAST];
    const off = h.roads[F.OFF_RAMP];
    // (Where the off-ramp's surface lies over the carriageway, its first 20 m or so: well inside it, a line's pieces
    // 2 m long, each drawn or not by its middle.)
    const onBoth = (x: number, z: number) => nearest(east.points, x, z).dist < east.width / 2 - 0.1 && nearest(off.points, x, z).dist < off.width / 2 - 0.6;
    expect(marksOf(F.MOTORWAY_EAST).filter(([x, , z]) => onBoth(x, z) && Math.abs(z - F.EAST_Z) > 2).length).toBe(0);
    expect(marksOf(F.OFF_RAMP).filter(([x, , z]) => onBoth(x, z)).length).toBe(0);
    // Away from the ramps, both edge lines run on.
    const far = marksOf(F.MOTORWAY_EAST).filter(([x]) => x > 250 && x < 300);
    expect(far.some(([, , z]) => z > F.EAST_Z + 3)).toBe(true);
    expect(far.some(([, , z]) => z < F.EAST_Z - 3)).toBe(true);
  });

  test("a one-way road's arrows point the way its cars go, every 40 m or so", () => {
    const road = (oneway: 1 | -1): RoadPlan => ({ points: line(0, 0, 100, 0, 100), width: 7, ends: ['out', 'out'], kind: 'street', oneway });
    for (const way of [1, -1] as const) {
      const g = new Geo();
      roadMarkings([road(way)], g, []);
      const xs = points(g).map(([x]) => x);
      expect(xs.length).toBeGreaterThan(0);
      // (Arrows at 20 and 60 m, their heads ahead of their middles.)
      const mean = xs.reduce((a, b) => a + b, 0) / xs.length;
      expect(Math.sign(mean - (ARROW_EVERY / 2 + ARROW_EVERY * 1.5) / 2)).toBe(way);
      // Down the lane's middle, the carriageway's (one lane).
      for (const [, , z] of points(g)) expect(Math.abs(z)).toBeLessThan(0.5);
    }
  });
});

test('the fixture is laid out as the markings expect (a sanity check of its widths)', () => {
  expect(carriageway(h.roads[F.AVENUE])).toBeCloseTo(14 - 2 * SIDEWALK, 6);
  const ls = lengths(h.roads[F.MOTORWAY_EAST].points);
  expect(pointAt(h.roads[F.MOTORWAY_EAST].points, ls.at(-1)! / 2, ls).p[1]).toBe(F.EAST_Z);
});
