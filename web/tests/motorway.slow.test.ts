/**
 * The motorway (`game/motorway.ts`) as the generator lays it in real worlds (`macro.ts`): edge to edge, clear of the
 * town and the villages, laid for traffic on the right, one junction of two ramps, a `raised` span at every road under
 * it and a `bridge` over its water, every crossing listed on both roads, lots and woods kept off, the same world twice
 * alike. (Its course, profile, crossings and half-diamond on land made for the test: `motorway.test.ts`.)
 */
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';
import type { RoadPlan } from '../src/core/protocol';
import { BEND_RADIUS, crossings, HAMLET_CLEAR, motorwayProblems, TOWN_CLEAR } from '../src/game/motorway';
import { deckSpans, lengths, MOTORWAY_KEEP, nearest, sharpestTurn, type Pt } from '../src/game/roads';

const offRect = (r: { x: number; z: number; w: number; d: number }, [x, z]: Pt) => Math.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d));
/** Which edge a point is on (north, east, south, west), -1 none. */
const edge = ([x, z]: Pt, W: number, D: number) => [z <= 1, x >= W - 1, z >= D - 1, x <= 1].indexOf(true);

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
