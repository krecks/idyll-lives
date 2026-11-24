// The road hierarchy's surface (`render/shared/builtSurface.ts`) on its fixture (`render/roadHierarchyFixture.ts`): the
// motorway's carriageways held `CROSS_CLEAR` over the street they pass over, at one height side by side, within their
// grade and their cut; its ramps running from the street's height to the carriageway's without a step; the street
// under the deck owning the ground there, the deck not land; and the land the motorway keeps (`roadMask`, `keepOf`)
// and keeps walkers off (`walkerBlocked`). On a flat land and on one that climbs across the motorway.
// `bun test tests/roadHierarchySurface.test.ts`.
import { describe, expect, test } from 'bun:test';
import type { RoadPlan } from '../src/core/protocol';
import { ROAD_KIND } from '../src/core/roadKinds';
import { blankLand, heightAt, LAND_CELL, type Land } from '../src/game/land';
import { deckSpans, keepOf, lengths, MOTORWAY_KEEP, pointAt, raisedSpans, roadMask, walkerBlocked } from '../src/game/roads';
import { BuiltSurface, CROSS_CLEAR, CUT_MAX_MOTORWAY, cutMostOf, DECK_UNDER, GRADE } from '../src/render/shared/builtSurface';
import * as F from '../src/render/roadHierarchyFixture';

const h = F.roadHierarchy();
const { roads } = h;
const len = (r: RoadPlan) => lengths(r.points).at(-1)!;
/** Arc length along either carriageway at `x`. */
const along = (x: number) => x - F.MOTORWAY_X0;

/** The fixture's land: its river (water 0.5 m deep, its bed 1.5 m down), the rest `height(x, z)`. */
function landOf(height: (x: number, z: number) => number): Land {
  const land = blankLand(h.width, h.depth, 'follow');
  for (let j = 0; j < land.rows; j++) {
    for (let i = 0; i < land.cols; i++) {
      const [x, z] = [i * LAND_CELL, j * LAND_CELL];
      const k = j * land.cols + i;
      if (x >= F.RIVER_X && x <= F.RIVER_X + F.RIVER_W && z >= F.RIVER_Z0) {
        land.h[k] = Math.round((height(x, z) - 1.5) * 100);
        land.wl[k] = Math.round((height(x, z) - 1) * 100);
      } else land.h[k] = Math.round(height(x, z) * 100);
    }
  }
  return land;
}

const LANDS: [string, (x: number, z: number) => number][] = [
  ['flat', () => 0],
  // (Up across the motorway and gently along it, a hollow at the street.)
  ['sloping', (x, z) => 0.03 * z + 0.01 * x - 2 * Math.exp(-(((x - F.STREET_X) / 30) ** 2))],
];

for (const [name, height] of LANDS) {
  describe(`the motorway's surface, ${name}`, () => {
    const land = landOf(height);
    const surface = new BuiltSurface(land, roads, []);
    const [east, west, street] = [F.MOTORWAY_EAST, F.MOTORWAY_WEST, F.BRIDGE_STREET];

    test('each carriageway at least CROSS_CLEAR over the street it passes over, and no junction there', () => {
      const over = surface.overpasses;
      expect(over.map((o) => [o.over, o.under])).toEqual([
        [east, street],
        [west, street],
      ]);
      for (const o of over) {
        expect(o.yOver - o.yUnder).toBeGreaterThanOrEqual(CROSS_CLEAR - 1e-3);
        // (All across the street and its sidewalks, a metre past them.)
        const half = roads[street].width / 2 + 1;
        for (let s = o.s - half; s <= o.s + half; s += 0.5) expect(surface.roadY(o.over, s) - surface.roadY(street, o.sUnder)).toBeGreaterThanOrEqual(CROSS_CLEAR - 1e-3);
      }
      for (const j of surface.junctionHeights) expect(Math.abs(j.x - F.STREET_X) < 6 && j.z > F.EAST_Z - 6 && j.z < F.WEST_Z + 6).toBe(false);
    });

    test('the two carriageways at one height side by side', () => {
      let most = 0;
      for (let s = 0; s <= len(roads[east]); s += 1) most = Math.max(most, Math.abs(surface.roadY(east, s) - surface.roadY(west, s)));
      expect(most).toBeLessThan(0.02);
    });

    test('every road within its grade, the motorway within its cut', () => {
      roads.forEach((r, i) => {
        const { s, y } = surface.profile(i);
        let steepest = 0;
        for (let k = 0; k + 1 < s.length; k++) if (s[k + 1] - s[k] > 0.5) steepest = Math.max(steepest, Math.abs(y[k + 1] - y[k]) / (s[k + 1] - s[k]));
        expect(steepest).toBeLessThanOrEqual(GRADE[r.kind ?? 'street'] + 0.01);
      });
      expect(GRADE.motorway).toBe(ROAD_KIND.motorway.grade);
      for (const m of [east, west]) {
        const r = roads[m];
        const decks = deckSpans(r);
        const { s, y } = surface.profile(m);
        for (let k = 0; k < s.length; k++) {
          if (decks.some(([a, b]) => s[k] >= a && s[k] <= b)) continue;
          const { p } = pointAt(r.points, s[k]);
          expect(Math.abs(y[k] - heightAt(land, p[0], p[1]))).toBeLessThanOrEqual(CUT_MAX_MOTORWAY);
        }
      }
      expect(cutMostOf('motorway')).toBe(CUT_MAX_MOTORWAY);
      expect(cutMostOf('street')).toBe(4);
    });

    test('the ramps run from the street to their carriageway without a step', () => {
      const ends: [number, number, 0 | 1, number][] = [
        // (Ramp, its end on the carriageway, its end on the street.)
        [F.OFF_RAMP, east, 0, 1],
        [F.ON_RAMP, west, 1, 0],
      ];
      for (const [ramp, carriageway, merge, meet] of ends) {
        const r = roads[ramp];
        const L = len(r);
        const at = (end: number) => (end === 0 ? 0 : L);
        const [mx] = r.points[merge === 0 ? 0 : r.points.length - 1];
        expect(Math.abs(surface.roadY(ramp, at(merge)) - surface.roadY(carriageway, along(mx)))).toBeLessThan(0.02);
        const [, mz] = r.points[meet === 0 ? 0 : r.points.length - 1];
        expect(Math.abs(surface.roadY(ramp, at(meet)) - surface.roadY(street, mz - F.AVENUE_Z))).toBeLessThan(0.02);
        // (And no step anywhere along it.)
        for (let s = 0; s + 0.5 <= L; s += 0.5) expect(Math.abs(surface.roadY(ramp, s + 0.5) - surface.roadY(ramp, s))).toBeLessThanOrEqual(GRADE.ramp * 0.5 + 0.01);
      }
    });

    test('under the deck the street owns the ground: the deck is not land', () => {
      const deckY = surface.roadY(east, along(F.STREET_X));
      const streetY = surface.roadY(street, F.EAST_Z - F.AVENUE_Z);
      // (On the street under it: the street, not the deck, whether asked of the surface or of the ground under it.)
      expect(surface.at(F.STREET_X, F.EAST_Z)).toBeCloseTo(streetY, 3);
      expect(surface.groundUnder(F.STREET_X, F.EAST_Z)).toBeCloseTo(streetY, 3);
      expect(surface.roadAt(F.STREET_X, F.EAST_Z)?.road).toBe(street);
      expect(surface.raisedAt(east, along(F.STREET_X))).toBe(true);
      expect(surface.onDeck(east, along(F.STREET_X))).toBe(true);
      expect(surface.onDeck(east, along(100))).toBe(false);
      // (Beside the street under the deck, the land held down under it: never up to the deck.)
      for (const x of [127, 132, 148, 153]) {
        for (const z of [F.EAST_Z, F.WEST_Z]) {
          const deck = surface.roadY(z === F.EAST_Z ? east : west, along(x));
          expect(surface.groundUnder(x, z)).toBeLessThanOrEqual(deck - DECK_UNDER + 1e-3);
          expect(surface.at(x, z)).toBe(surface.landAt(x, z));
        }
      }
      expect(deckY - surface.landAt(F.STREET_X, F.EAST_Z)).toBeGreaterThan(CROSS_CLEAR - 0.1);
      // (The graded land stored: under the deck no higher than the deck less DECK_UNDER either.)
      const g = surface.graded();
      const k = Math.round(F.EAST_Z / LAND_CELL) * land.cols + Math.round(132 / LAND_CELL);
      expect(g[k] / 100).toBeLessThanOrEqual(deckY);
      // (Where it lies on the land, the motorway is what stands there.)
      expect(surface.at(100, F.EAST_Z)).toBeCloseTo(surface.roadY(east, along(100)), 3);
      expect(surface.groundUnder(100, F.EAST_Z)).toBeCloseTo(surface.roadY(east, along(100)), 3);
      // (Over the river, the bridge's deck clear of its water, the ground under it the river's bed.)
      const bridge = along(F.RIVER_X + F.RIVER_W / 2);
      expect(surface.onDeck(east, bridge)).toBe(true);
      expect(surface.raisedAt(east, bridge)).toBe(false);
      expect(surface.groundUnder(F.RIVER_X + F.RIVER_W / 2, F.EAST_Z)).toBeLessThan(surface.roadY(east, bridge) - 1);
    });

    test('solved again the same', () => {
      const again = new BuiltSurface(land, roads, []);
      expect(again.roadsSig()).toBe(surface.roadsSig());
    });
  });
}

describe('the land the motorway keeps', () => {
  const [W, D] = [h.width, h.depth];
  const mask = roadMask(roads, W, D);
  const tile = (m: Uint8Array, x: number, z: number) => m[Math.floor(z) * W + Math.floor(x)];

  test('lots kept MOTORWAY_KEEP m from each carriageway, the gap between them and under its deck too', () => {
    expect(keepOf(roads[F.MOTORWAY_EAST])).toBe(MOTORWAY_KEEP - 4.5);
    expect(keepOf(roads[F.BRIDGE_STREET])).toBe(0);
    for (const x of [60, 100, 135, 250, 400]) {
      for (let z = F.EAST_Z - MOTORWAY_KEEP + 1; z <= F.WEST_Z + MOTORWAY_KEEP - 1; z++) expect(tile(mask, x, z)).toBe(1);
    }
    // (Past it, open, but where a ramp or the street is.)
    expect(tile(mask, 250, F.EAST_Z - MOTORWAY_KEEP - 2)).toBe(0);
    expect(tile(mask, 250, F.WEST_Z + MOTORWAY_KEEP + 2)).toBe(0);
  });

  test("a town's own streets keep only their edge, as before", () => {
    const streets = roads.filter((r) => r.kind !== 'motorway' && r.kind !== 'ramp');
    const plain = (r: RoadPlan) => r.width / 2 + 0.5;
    const keep = roadMask(streets, W, D);
    // (West Cross, x = 90: its edge and the clearance, no more.)
    expect(tile(keep, 90 + plain(roads[F.CROSS_WEST]) - 0.2, 100)).toBe(1);
    expect(tile(keep, 90 + plain(roads[F.CROSS_WEST]) + 1.5, 100)).toBe(0);
  });

  test('walkers kept off the motorway and its ramps where they lie on the land, not under the deck', () => {
    const blocked = walkerBlocked(roads, W, D)!;
    expect(blocked).not.toBeNull();
    // (On a carriageway, and between the two.)
    for (const z of [F.EAST_Z - 4, F.EAST_Z, (F.EAST_Z + F.WEST_Z) / 2, F.WEST_Z, F.WEST_Z + 4]) expect(tile(blocked, 100, z)).toBe(1);
    // (Under its deck over the street: the street, and the land beside it.)
    const [r0, r1] = raisedSpans(roads[F.MOTORWAY_EAST])[0];
    for (const x of [F.STREET_X, F.STREET_X - 6, F.STREET_X + 8]) {
      expect(x - F.MOTORWAY_X0 > r0 && x - F.MOTORWAY_X0 < r1).toBe(true);
      expect(tile(blocked, x, F.EAST_Z)).toBe(0);
      expect(tile(blocked, x, F.WEST_Z)).toBe(0);
    }
    // (On a ramp, but not where it meets the street.)
    expect(tile(blocked, 80, F.EAST_Z - 7)).toBe(1);
    expect(tile(blocked, F.STREET_X, F.EAST_Z - 25)).toBe(0);
    expect(tile(blocked, F.STREET_X + 1, F.WEST_Z + 25)).toBe(0);
    // (Off it, open.)
    expect(tile(blocked, 250, F.EAST_Z - 10)).toBe(0);
    expect(tile(blocked, 250, F.AVENUE_Z)).toBe(0);
  });

  test('no motorway, nothing kept from walkers', () => {
    expect(walkerBlocked(roads.filter((r) => r.kind !== 'motorway' && r.kind !== 'ramp'), W, D)).toBeNull();
  });
});
