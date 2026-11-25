/**
 * The motorway (`game/motorway.ts`): its course over a land made for the test (edge to edge past the town, clear of
 * it, bending no tighter than `BEND_RADIUS`, none on an island nor up a mountainside), its profile, its crossings and
 * its half-diamond; then as the generator lays it in real worlds (`macro.ts`): edge to edge, clear of the town and the
 * villages, laid for traffic on the right, one junction of two ramps, a `raised` span at every road under it and a
 * `bridge` over its water, every crossing listed on both roads, lots and woods kept off, the same world twice alike.
 */
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';
import type { RoadPlan } from '../src/core/protocol';
import { Cells } from '../src/game/countryRouter';
import { APART, BEND_RADIUS, carriageways, clearOfMotorway, crossings, finishMotorway, HAMLET_CLEAR, motorwayProblems, profileOf, RAMP_LENGTH, routeMotorway, TOWN_CLEAR, TOWN_REACH, withVillages, type MotorwayRoute } from '../src/game/motorway';
import { deckSpans, lengths, MEET, MOTORWAY_KEEP, nearest, pointAt, sharpestTurn, type Pt } from '../src/game/roads';

const offRect = (r: { x: number; z: number; w: number; d: number }, [x, z]: Pt) => Math.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d));
const len = (r: RoadPlan) => lengths(r.points).at(-1)!;
/** Which edge a point is on (north, east, south, west), -1 none. */
const edge = ([x, z]: Pt, W: number, D: number) => [z <= 1, x >= W - 1, z >= D - 1, x <= 1].indexOf(true);

/** A flat land `W` × `D` with height `h(x, z)` and water where `wet` says, a town's box in its middle. */
function site(W: number, D: number, h: (x: number, z: number) => number = () => 0, wet: (x: number, z: number) => boolean = () => false) {
  const land = new Cells(W, D);
  for (let i = 0; i < land.gw * land.gd; i++) {
    const [x, z] = land.centre(i);
    land.height[i] = h(x, z);
    land.water[i] = wet(x, z) ? 1 : 0;
  }
  const rect = { x: W / 2 - 100, z: D / 2 - 100, w: 200, d: 200 };
  return { land, areas: [{ rect, centre: [W / 2, D / 2] as Pt }], roads: [] as RoadPlan[], wet, seed: 7 };
}

describe('its course', () => {
  const s = site(832, 832);
  const route = routeMotorway(s) as MotorwayRoute;

  test('on open flat land: edge to edge past the town, clear of it, no tighter than its bend', () => {
    expect('failed' in route).toBe(false);
    const { middle, ways } = route;
    expect((edge(middle[0], 832, 832) + 2) % 4).toBe(edge(middle.at(-1)!, 832, 832));
    for (const w of ways) {
      expect(w.kind).toBe('motorway');
      expect(w.ends).toEqual(['out', 'out']);
      expect(edge(w.points[0], 832, 832)).toBeGreaterThanOrEqual(0);
      expect(edge(w.points.at(-1)!, 832, 832)).toBeGreaterThanOrEqual(0);
      expect(sharpestTurn(w.points, 25).angle).toBeLessThan(25 / BEND_RADIUS + 0.01);
    }
    const near = Math.min(...middle.map((p) => offRect(s.areas[0].rect, p)));
    expect(near).toBeGreaterThanOrEqual(TOWN_CLEAR + APART);
    expect(near).toBeLessThanOrEqual(TOWN_REACH);
  });

  test('two carriageways APART m apart, laid for traffic on the right', () => {
    const [a, b] = route.ways;
    const mid = pointAt(a.points, len(a) / 2).p;
    expect(nearest(b.points, mid[0], mid[1]).dist).toBeCloseTo(APART, 1);
    // (The first's cars go the way the middle line runs, on its right; the second's against it, on its left.)
    expect([a.oneway, b.oneway]).toEqual([1, -1]);
    expect(motorwayProblems(route.ways)).toEqual(['0 ramps']);
  });

  test("the country roads keep off it but at its gates, the junction's the nearest the town", () => {
    const { land } = s;
    const { blocked, gates, middle } = route;
    expect(gates.length).toBeGreaterThanOrEqual(1);
    expect(gates.length).toBeLessThanOrEqual(3);
    const town = s.areas[0].centre;
    const nearestS = nearest(middle, town[0], town[1]).s;
    expect(Math.abs(gates[0].s - nearestS)).toBeLessThan(120);
    // (On the middle line: blocked, but at a gate.)
    const ls = lengths(middle);
    for (let k = 0; k < middle.length; k += 10) {
      const atGate = gates.some((g) => Math.abs(g.s - ls[k]) < 6);
      if (!atGate && ls[k] > 20 && ls[k] < ls.at(-1)! - 20) expect(blocked[land.cell(middle[k][0], middle[k][1])]).toBe(1);
    }
    for (const g of gates) expect(blocked[land.cell(g.p[0], g.p[1])]).toBe(0);
  });

  test('the same land, the same course', () => {
    const again = routeMotorway(site(832, 832)) as MotorwayRoute;
    expect(JSON.stringify(again.ways)).toBe(JSON.stringify(route.ways));
    expect(again.gates).toEqual(route.gates);
  });

  test('none on an island with a wide sea round it, nor up a mountainside: a note why, not a fault', () => {
    const island = routeMotorway(site(832, 832, () => 0, (x, z) => Math.hypot(x - 416, z - 416) > 300));
    expect('failed' in island && island.failed).toMatch(/^No motorway: /);
    const steep = routeMotorway(site(832, 832, (x, z) => (x + z) * 0.2));
    expect('failed' in steep && steep.failed).toMatch(/climbs too steeply/);
  });

  test("on an island a strait off the world's edge: out over it on a bridge at either end", () => {
    // (A rounded square of land 60 m in from every edge.)
    const wet = (x: number, z: number) => Math.min(x, z, 832 - x, 832 - z) < 60;
    const route = routeMotorway(site(832, 832, () => 0, wet)) as MotorwayRoute;
    expect('failed' in route).toBe(false);
    const { middle } = route;
    expect(wet(...middle[0]) && wet(...middle.at(-1)!)).toBe(true);
    // (Dry between, its gates on the land.)
    expect(middle.slice(80, -80).every((p) => !wet(...p))).toBe(true);
    for (const g of route.gates) expect(wet(...g.p)).toBe(false);
  });

  test('the villages after it keep clear of it, further by its junction; its gates chosen again by them', () => {
    const { middle, gates } = route;
    const ls = lengths(middle);
    /** A village's 100 m square box, its middle `off` + 50 m from its middle line (on either side) at `s` along it. */
    const box = (s: number, off: number, side = 1) => {
      const { p, t } = pointAt(middle, s, ls);
      const [cx, cz] = [p[0] - t[1] * (off + 50) * side, p[1] + t[0] * (off + 50) * side];
      return { x: cx - 50, z: cz - 50, w: 100, d: 100 };
    };
    // (Away from its junction: `HAMLET_CLEAR` m off its outer edges; by it, further.)
    const far = gates[0].s > ls.at(-1)! / 2 ? 120 : ls.at(-1)! - 120;
    expect(clearOfMotorway(route, box(far, 10))).toBe(false);
    expect(clearOfMotorway(route, box(far, HAMLET_CLEAR + APART + 15))).toBe(true);
    expect(clearOfMotorway(route, box(gates[0].s, HAMLET_CLEAR + APART + 15))).toBe(false);
    expect(clearOfMotorway(route, box(gates[0].s, 90))).toBe(true);
    // (Its gates chosen again with villages down on either side of it: the junction's as it was, all of them clear of the boxes.)
    const villages = [box(far, 60, 1), box(far, 60, -1)].map((rect) => ({ rect, centre: [rect.x + 50, rect.z + 50] as Pt }));
    const again = withVillages(route, { ...s, areas: [...s.areas, ...villages] });
    expect(again.gates[0]).toEqual(gates[0]);
    expect(again.middle).toBe(route.middle);
    for (const v of villages) expect(again.blocked[s.land.cell(v.centre[0], v.centre[1])]).toBe(0);
  });
});

describe('its profile', () => {
  test('within its grade, cut and filled no more than it may, lifted where a road passes under', () => {
    expect(profileOf([0, 0, 0, 0])!.cut).toBe(0);
    // (A 20 m step: no profile within 5 % and 7 m of cut and fill over 4 m.)
    expect(profileOf([0, 0, 20, 20])).toBeNull();
    const flat = new Array(80).fill(0);
    const lifted = profileOf(flat, new Map([[40, 6.6]]))!;
    expect(lifted.y[40]).toBeGreaterThanOrEqual(6.6);
    for (let k = 1; k < flat.length; k++) expect(Math.abs(lifted.y[k] - lifted.y[k - 1])).toBeLessThanOrEqual(0.05 * 4 + 1e-9);
    // (Over the water, any height.)
    expect(profileOf([0, NaN, NaN, 0])).not.toBeNull();
  });
});

describe('its crossings and its junction', () => {
  // A middle line along x at z = 400, a country road north to south under it at x = 420, the town to the north.
  const W = 832;
  const middle: Pt[] = Array.from({ length: W + 1 }, (_, i): Pt => [i, 400]);
  const ways = carriageways([[-20, 400], ...middle, [W + 20, 400]], W, W)!;
  const road: RoadPlan = { points: Array.from({ length: 601 }, (_, i): Pt => [420, 100 + i]), width: 7, ends: ['join', 'out'], kind: 'country', name: 'Under Road' };
  const route: MotorwayRoute = { middle, ways, blocked: new Uint8Array(1), gates: [{ s: 420, p: [420, 400], across: [0, 1] }], name: 'M9', junction: 4, heights: [] };
  const dry = () => false;
  const named = (n: string) => n;
  const laid = finishMotorway(route, [road], dry, [420, 200], 1, named)!;

  test('where a road crosses its middle line, and how square', () => {
    const found = crossings(middle, [road]);
    expect(found.length).toBe(1);
    expect(found[0].m).toBeCloseTo(420, 6);
    expect(found[0].s).toBeCloseTo(300, 6);
    expect(found[0].angle).toBeCloseTo(Math.PI / 2, 6);
  });

  test('a raised span over the road on each carriageway, listed on both roads', () => {
    expect(laid).not.toBeNull();
    const [a, b] = laid.roads;
    expect([a.name, b.name]).toEqual(['M9 Eastbound', 'M9 Westbound']);
    for (const [k, w] of [a, b].entries()) {
      // (Its points from x = 0.5, the world's edge.)
      expect(w.crosses).toEqual([{ road: 0, s: 419.5 }]);
      expect(deckSpans(w).some(([s0, s1]) => s0 < 415 && s1 > 425)).toBe(true);
      expect(laid.under.get(0)!.some((u) => u.road === 1 + k)).toBe(true);
    }
    expect(laid.junction).toBe(0);
  });

  test('a half-diamond: two ramps of 110 to 140 m to their carriageways right, a T on the road under it', () => {
    const roads = [road, ...laid.roads];
    expect(motorwayProblems(roads)).toEqual([]);
    const ramps = laid.roads.filter((r) => r.kind === 'ramp');
    expect(ramps.length).toBe(2);
    expect(ramps.map((r) => r.ends)).toEqual([
      ['merge', 'join'],
      ['join', 'merge'],
    ]);
    for (const r of ramps) {
      expect(len(r)).toBeGreaterThanOrEqual(RAMP_LENGTH[0]);
      expect(len(r)).toBeLessThanOrEqual(RAMP_LENGTH[1]);
      const T = r.ends[0] === 'join' ? r.points[0] : r.points.at(-1)!;
      expect(nearest(road.points, T[0], T[1]).dist).toBeLessThan(0.5);
      // (Out 25 m or more past its carriageway's middle; met at MEET or squarer.)
      expect(Math.abs(T[1] - 400)).toBeGreaterThanOrEqual(APART / 2 + 24);
      const e = r.ends[0] === 'join' ? 8 : len(r) - 8;
      const t = pointAt(r.points, e).t;
      expect(Math.abs(t[0])).toBeGreaterThan(Math.sin(MEET));
    }
    // (Both on one side of the road under it.)
    const sides = ramps.map((r) => Math.sign(pointAt(r.points, len(r) / 2).p[0] - 420));
    expect(sides[0]).toBe(sides[1]);
  });
});

describe('in the generated worlds', async () => {
  usePublicFetch();
  const { assembleTown, plotRect, populate, useTownContent, vacantSlots } = await import('../src/game/town');
  const { AssetRegistry } = await import('../src/assets/registry');
  const { randomHousehold } = await import('../src/game/household');
  const { mulberry32 } = await import('../src/game/noise');
  const { landDraft, layOn } = await import('../src/game/macro');
  const content = await loadContent();
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json');
  useTownContent(content);
  const t = await templatesOf(content);
  const make = (kind: string, size: 'small' | 'medium' | 'large', seed: number) => layOn(content, t, landDraft(content, t, kind, { size }, seed), {}, { waterKept: true });

  // (A farm's world and a river's, whose motorway crosses the river.)
  for (const [kind, size, seed] of [
    ['farm', 'medium', 104733],
    ['river', 'large', 104734],
  ] as const) {
    test(
      `${kind} ${size} ${seed}`,
      () => {
        const d = make(kind, size, seed);
        const roads = d.roads;
        const ways = roads.flatMap((r, i) => (r.kind === 'motorway' ? [i] : []));
        const ramps = roads.filter((r) => r.kind === 'ramp');
        expect(ways.length).toBe(2);
        expect(ramps.length).toBe(2);
        expect(motorwayProblems(roads)).toEqual([]);
        const areas = d.open!.areas;
        for (const i of ways) {
          const w = roads[i];
          // Edge to edge, opposite ones.
          const [a, b] = [edge(w.points[0], d.width, d.depth), edge(w.points.at(-1)!, d.width, d.depth)];
          expect(a).toBeGreaterThanOrEqual(0);
          expect((a + 2) % 4).toBe(b);
          expect(sharpestTurn(w.points, 25).angle).toBeLessThan(25 / BEND_RADIUS + 0.02);
          // Clear of the town and the villages.
          const off = (k: number) => Math.min(...w.points.map((p) => offRect(areas[k], p)));
          expect(off(0) - w.width / 2).toBeGreaterThanOrEqual(TOWN_CLEAR - 1);
          for (let k = 1; k < areas.length; k++) expect(off(k) - w.width / 2).toBeGreaterThanOrEqual(HAMLET_CLEAR - 1);
          // A raised span at every road crossing it, listed on both; a bridge over its water.
          const decks = deckSpans(w);
          for (const c of w.crosses ?? []) {
            expect(decks.some(([s0, s1]) => c.s > s0 && c.s < s1)).toBe(true);
            expect(roads[c.road].crosses?.some((x) => x.road === i)).toBe(true);
          }
          const others = roads.filter((r) => r.kind !== 'motorway' && r.kind !== 'ramp' && r.kind !== 'trail');
          const under = new Set((w.crosses ?? []).map((c) => roads[c.road]));
          for (const r of others) {
            const crossing = crossings(w.points, [r]);
            if (crossing.length) expect(under.has(r)).toBe(true);
          }
          const water = (d.open!.zones ?? []).filter((z) => z.kind === 'water').map((z) => z.poly);
          const ls = lengths(w.points);
          const inWater = (p: Pt) => water.some((poly) => insidePoly(poly, p));
          w.points.forEach((p, k) => {
            if (inWater(p)) expect(decks.some(([s0, s1]) => ls[k] >= s0 && ls[k] <= s1)).toBe(true);
          });
        }
        // Lots kept MOTORWAY_KEEP m from each carriageway's middle (their corners and middles).
        for (const s of d.slots) {
          const r = plotRect(t, s);
          for (const p of [[r.x, r.z], [r.x + r.w, r.z], [r.x, r.z + r.d], [r.x + r.w, r.z + r.d], [r.x + r.w / 2, r.z + r.d / 2]] as Pt[]) {
            for (const i of ways) expect(nearest(roads[i].points, p[0], p[1]).dist).toBeGreaterThan(MOTORWAY_KEEP - 1);
          }
        }
        // And the woods: no tree within its kept land (their outlines' points).
        for (const z of d.open!.zones.filter((o) => o.kind === 'woods')) {
          for (const p of z.poly) for (const i of ways) expect(nearest(roads[i].points, p[0], p[1]).dist).toBeGreaterThan(MOTORWAY_KEEP - 4);
        }
        // The same world twice alike.
        expect(JSON.stringify(make(kind, size, seed).roads)).toBe(JSON.stringify(roads));
        // Its town file's roads and its meta's one list, road for road (the simulation reads the decks from the meta's by index).
        const realRandom = Math.random;
        Math.random = mulberry32(seed);
        try {
          const town = populate(content, assets, t, d);
          const file = JSON.parse(assembleTown(content, t, town, randomHousehold(content, assets, 2), vacantSlots(town)[0].index)) as { roads: RoadPlan[]; meta: { roads: RoadPlan[] } };
          expect(file.roads.length).toBe(file.meta.roads.length);
          file.roads.forEach((r, k) => expect(JSON.stringify(r)).toBe(JSON.stringify(file.meta.roads[k])));
          expect(file.roads.filter((r) => r.kind === 'motorway').length).toBe(2);
        } finally {
          Math.random = realRandom;
        }
      },
      120_000,
    );
  }
});

/** Whether `p` is inside polygon `poly` (even-odd). */
function insidePoly(poly: readonly Pt[], [x, z]: Pt): boolean {
  let hit = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const [[xi, zi], [xj, zj]] = [poly[i], poly[j]];
    if (zi > z !== zj > z && x < ((xj - xi) * (z - zi)) / (zj - zi) + xi) hit = !hit;
  }
  return hit;
}
