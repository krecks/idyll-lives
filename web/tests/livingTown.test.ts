// The living town's drawing at town scale: a backdrop
// building fitted to the town view's unit meshes (`render/shared/townBuildings.ts`), their faces facing out, and
// the debug fixture (`debug/ltFixture.ts`, `?ltFixture`) keeping the contract's rules on a made-up town.
import { describe, expect, test } from 'bun:test';
import type { BackdropBuilding, PlotInfo, RoadPlan, WorldStructure } from '../src/core/protocol';
import { ltFixture } from '../src/debug/ltFixture';
import { BACKDROP_PALETTE, backdropBuilding, backdropGround, gardenHedges, homeLights, litAt, lotBuilding, ROOFS, unitMeshes } from '../src/render/shared/townBuildings';

const flat = () => 0;

describe("the town view's buildings", () => {
  test('a rectangle is fitted exactly: along its ridge, its walls from under the ground to its storeys', () => {
    // A 12 × 8 m house turned 30°, its ridge along its long side.
    const a = Math.PI / 6;
    const [ux, uz] = [Math.cos(a), Math.sin(a)];
    const [vx, vz] = [-uz, ux];
    const at = (u: number, v: number): [number, number] => [100 + u * ux + v * vx, 50 + u * uz + v * vz];
    const b: BackdropBuilding = { poly: [at(-6, -4), at(6, -4), at(6, 4), at(-6, 4)], y: 2, storeys: 2, roof: 'gable', ridge: -a, use: 'house', look: 3, area: 0 };
    // (`ridge` turns round +y: (cos r, −sin r) in x, z, so this ridge's is −30°.)
    const t = backdropBuilding(b, 0, () => 1.5);
    expect(t.cx).toBeCloseTo(100, 6);
    expect(t.cz).toBeCloseTo(50, 6);
    expect(t.w).toBeCloseTo(12, 6);
    expect(t.d).toBeCloseTo(8, 6);
    expect(t.ux).toBeCloseTo(ux, 6);
    expect(t.uz).toBeCloseTo(uz, 6);
    // (Its floor at its `y`, its walls from under its lowest ground, two storeys of 3 m, a 34° roof over 4 m.)
    expect(t.floor).toBe(2);
    expect(t.base).toBeLessThan(1.5);
    expect(t.top).toBe(8);
    expect(t.rise).toBeCloseTo(4 * Math.tan((34 * Math.PI) / 180), 6);
  });

  test("a hip's ridge runs along the longer side; a flat roof is a parapet; the palette is taken modulo", () => {
    const b: BackdropBuilding = { poly: [[0, 0], [6, 0], [6, 14], [0, 14]], y: 0, storeys: 3, roof: 'hip', use: 'flats', look: BACKDROP_PALETTE.length * 2 + 1, area: 0 };
    const t = backdropBuilding(b, 4, flat);
    expect(t.w).toBeCloseTo(14, 6);
    expect(t.d).toBeCloseTo(6, 6);
    expect(Math.abs(t.uz)).toBeCloseTo(1, 6);
    const f = backdropBuilding({ ...b, roof: 'flat' }, 4, flat);
    expect(f.rise).toBeLessThan(1);
    expect(f.top).toBe(9);
  });

  test('a footprint that is not a rectangle is drawn as the rectangle round it', () => {
    const l: BackdropBuilding = { poly: [[0, 0], [10, 0], [10, 4], [4, 4], [4, 8], [0, 8]], y: 0, storeys: 1, roof: 'flat', use: 'shop', look: 0, area: 0 };
    const t = backdropBuilding(l, 0, flat);
    expect(t.w * t.d).toBeCloseTo(80, 6);
    expect(t.cx).toBeCloseTo(5, 6);
    expect(t.cz).toBeCloseTo(4, 6);
  });

  test("a far lot's house keeps its own look and box", () => {
    const t = lotBuilding({ id: 3, chunk: 0, box: [10, 20, 18, 30], storeys: 2, wall: '#ECE6D8', roof: '#5C626A', shape: 'hip', rise: 2.4 }, flat);
    expect([t.cx, t.cz, t.w, t.d, t.top, t.rise, t.roof]).toEqual([14, 25, 10, 8, 6, 2.4, 'hip']);
    expect(t.uz).toBe(1);
  });

  test('lights go by the life there: a shop’s while it is open, a home’s by the hour, a lot’s house’s while someone is up', () => {
    // Homes: all their lights in the evening, most out by half past midnight, on again at breakfast.
    expect(homeLights(20 * 60)).toBe(1);
    expect(homeLights(23 * 60)).toBeLessThan(0.8);
    expect(homeLights(24 * 60 + 30)).toBeCloseTo(0.15);
    expect(homeLights(3 * 60)).toBeCloseTo(0.15);
    expect(homeLights(8 * 60)).toBeCloseTo(0.7);
    let last = 1;
    for (let m = 22 * 60; m <= 24 * 60 + 30; m += 10) {
      expect(homeLights(m)).toBeLessThanOrEqual(last);
      last = homeLights(m);
    }
    const shop: BackdropBuilding = { poly: [[0, 0], [10, 0], [10, 6], [0, 6]], y: 0, storeys: 2, roof: 'gable', ridge: 0, use: 'shop', look: 1, place: 0, area: 0 };
    // Open 09:00 to 18:00, Monday to Saturday.
    const places = [{ name: 'Shop', kind: 'shop' as const, x: 5, z: 7, hours: { open: 540, close: 1080, days: 0b0111111 }, area: 0 }];
    const s = backdropBuilding(shop, 0, flat, places);
    expect(s.hours).toEqual(places[0].hours);
    expect(litAt(s, 17 * 60, 0)).toBe(s.lit);
    expect(litAt(s, 20 * 60, 0)).toBeLessThan(s.lit * 0.1);
    expect(litAt(s, 12 * 60, 6)).toBeLessThan(s.lit * 0.1);
    // A pub open 17:00 to 01:00 on Friday is still open after midnight on Saturday's clock.
    const pub = { lit: 0.9, hours: { open: 1020, close: 60, days: 0b0010000 } };
    expect(litAt(pub, 23 * 60, 4)).toBe(0.9);
    expect(litAt(pub, 30, 5)).toBe(0.9);
    expect(litAt(pub, 120, 5)).toBeLessThan(0.1);
    expect(litAt(pub, 23 * 60, 5)).toBeLessThan(0.1);
    // A home nobody plays: by the hour. A place without hours: its own share.
    const home = backdropBuilding({ ...shop, use: 'flats', place: undefined }, 1, flat);
    expect(home.home).toBe(true);
    expect(litAt(home, 20 * 60, 0)).toBe(home.lit);
    expect(litAt(home, 2 * 60, 0)).toBeCloseTo(home.lit * 0.15);
    const hall = backdropBuilding({ ...shop, use: 'hall', place: undefined }, 2, flat);
    expect(litAt(hall, 2 * 60, 0)).toBe(hall.lit);
    // A lot's house far off: lit only while someone is up in it.
    const lot = { id: 3, chunk: 0, box: [10, 20, 18, 30] as const, storeys: 2, wall: '#ECE6D8', roof: '#5C626A', shape: 'hip' as const, rise: 2.4 };
    expect(lotBuilding(lot, flat, true).lit).toBeGreaterThan(0.3);
    expect(lotBuilding(lot, flat, false).lit).toBe(0);
  });

  test('a pitched roof is toned down from its palette colour (bright tile read as lit beside the game’s own roofs)', () => {
    const b: BackdropBuilding = { poly: [[0, 0], [10, 0], [10, 6], [0, 6]], y: 0, storeys: 1, roof: 'gable', ridge: 0, use: 'house', look: 0, area: 0 };
    const t = backdropBuilding(b, 0, flat);
    const palette = parseInt(BACKDROP_PALETTE[0][1].slice(1, 3), 16) / 255;
    expect(t.roofColor[0]).toBeLessThan(palette * 0.85);
    expect(t.roofColor[0]).toBeGreaterThan(palette * 0.6);
  });

  test('a backdrop building stands on something: a forecourt, a yard or a hedged garden, with a way to the road', () => {
    const road: RoadPlan = { points: [[-50, -20], [80, -20]], width: 10 } as RoadPlan;
    const house: BackdropBuilding = { poly: [[0, 0], [10, 0], [10, 8], [0, 8]], y: 0, storeys: 2, roof: 'gable', ridge: 0, use: 'house', look: 0, area: 0 };
    const shop: BackdropBuilding = { ...house, poly: [[40, 0], [52, 0], [52, 8], [40, 8]], use: 'shop' };
    const barn: BackdropBuilding = { ...house, poly: [[0, 200], [10, 200], [10, 208], [0, 208]], use: 'barn' };
    const grounds = [house, shop, barn].map((b) => backdropGround(b, [road]));
    expect(grounds.map((g) => g.kind)).toEqual(['garden', 'forecourt', 'yard']);
    // Its way goes to the nearest point of the road; none for the barn 200 m off.
    expect(grounds[0].to).toEqual([5, -20]);
    expect(grounds[1].to).toEqual([46, -20]);
    expect(grounds[2].to).toBeNull();
    // Its ground reaches a few metres round its walls.
    const xs = grounds[0].round.map((q) => q[0]);
    expect(Math.min(...xs)).toBeLessThan(-3);
    expect(Math.max(...xs)).toBeGreaterThan(13);
    // The garden's hedge: three sides, the one to the road left open; low, and no windows.
    const hedges = gardenHedges([house, shop, barn], grounds, [], [road], flat);
    expect(hedges.length).toBe(3);
    for (const h of hedges) {
      expect(h.top - 0).toBeLessThan(1.5);
      expect(h.d).toBeLessThan(1);
      expect(h.style).toBe(0);
      // (None along the side nearest the road, z about -3.5.)
      expect(h.cz).toBeGreaterThan(-3);
    }
    // No hedge along a side that would stand on a lot (the one towards it goes), nor on a road or its pavement.
    expect(gardenHedges([house], [grounds[0]], [{ x: 12, z: -10, w: 20, d: 15 }], [road], flat).length).toBe(2);
    const near: RoadPlan = { points: [[-50, -9], [80, -9]], width: 10 } as RoadPlan;
    const byRoad = gardenHedges([house], [backdropGround(house, [near])], [], [near], flat);
    expect(byRoad.length).toBe(1);
    expect(byRoad[0].cz).toBeGreaterThan(10);
    // Between two houses whose gardens meet there is none: the two are one.
    const next: BackdropBuilding = { ...house, poly: [[15, 0], [25, 0], [25, 8], [15, 8]] };
    const pair = [house, next].map((b) => backdropGround(b, [road]));
    expect(gardenHedges([house, next], pair, [], [road], flat).length).toBe(4);
  });

  test("every unit mesh's faces face out from its middle (none drawn inside out)", () => {
    const meshes = unitMeshes();
    for (const key of ['walls', ...ROOFS] as const) {
      const d = meshes[key].meshData();
      const p = d.positions;
      for (let i = 0; i < d.indices.length; i += 3) {
        const [a, b, c] = [d.indices[i], d.indices[i + 1], d.indices[i + 2]];
        const e1 = [p[a * 3] - p[b * 3], p[a * 3 + 1] - p[b * 3 + 1], p[a * 3 + 2] - p[b * 3 + 2]];
        const e2 = [p[c * 3] - p[b * 3], p[c * 3 + 1] - p[b * 3 + 1], p[c * 3 + 2] - p[b * 3 + 2]];
        const n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
        // (The winding's normal agrees with the vertex normal: `Geo`'s convention.)
        const vn = [d.normals[a * 3], d.normals[a * 3 + 1], d.normals[a * 3 + 2]];
        expect(n[0] * vn[0] + n[1] * vn[1] + n[2] * vn[2]).toBeGreaterThanOrEqual(0);
      }
      expect(d.indices.length / 3).toBeLessThan(80);
    }
  });
});

/** A made-up Big world: an east–west street with a row of lots on its north side, and open land beyond. */
function town(): WorldStructure {
  const road: RoadPlan = { points: Array.from({ length: 61 }, (_, i): [number, number] => [20 + i * 5, 200]), width: 9.7, ends: ['out', 'out'], kind: 'street', area: 0 };
  const plots: PlotInfo[] = Array.from({ length: 8 }, (_, i) => ({ id: i, name: `${i}`, x: 40 + i * 20, z: 175, w: 16, d: 20, public: false, entry: null, house: null }));
  return {
    width: 400,
    depth: 400,
    plots,
    exits: [],
    rooms: [],
    meta: { seed: 9, roads: [road], areas: [{ name: 'Town', kind: 'hamlet', x: 0, z: 0, w: 400, d: 400 }], world: { kind: 'small', size: 'big', landscape: { hills: 0, rise: 1, rim: 0, rimFrom: 0, forest: 0, treeline: 0, conifers: 0.5, birches: 0.5, grass: ['#000000', '#000000', '#000000', '#000000'] } } },
  } as unknown as WorldStructure;
}

function overlaps(a: readonly [number, number][], b: readonly [number, number][]): boolean {
  // Separating axes of two convex polygons.
  for (const poly of [a, b]) {
    for (let i = 0; i < poly.length; i++) {
      const [p, q] = [poly[i], poly[(i + 1) % poly.length]];
      const n = [-(q[1] - p[1]), q[0] - p[0]];
      const span = (s: readonly [number, number][]) => s.map(([x, z]) => x * n[0] + z * n[1]);
      const [sa, sb] = [span(a), span(b)];
      if (Math.max(...sa) <= Math.min(...sb) + 1e-6 || Math.max(...sb) <= Math.min(...sa) + 1e-6) return false;
    }
  }
  return true;
}

describe('the debug fixture (`?ltFixture`)', () => {
  const world = town();
  const fx = ltFixture(world);

  test('fills the land beside the road, the same each time', () => {
    expect(fx.backdrop.length).toBeGreaterThan(20);
    expect(ltFixture(world)).toBe(fx);
    expect(fx.centre.length).toBe(1);
  });

  test('no building on a lot (or within a metre of one), on the road and its pavements, or on another', () => {
    const road = world.meta!.roads![0];
    for (const [i, b] of fx.backdrop.entries()) {
      expect(b.poly.length).toBeGreaterThanOrEqual(4);
      expect(b.storeys).toBeGreaterThanOrEqual(1);
      expect(b.storeys).toBeLessThanOrEqual(6);
      for (const p of world.plots) {
        const grown: [number, number][] = [[p.x - 1, p.z - 1], [p.x + p.w + 1, p.z - 1], [p.x + p.w + 1, p.z + p.d + 1], [p.x - 1, p.z + p.d + 1]];
        expect(overlaps(b.poly, grown)).toBe(false);
      }
      for (const [x, z] of b.poly) if (x > 20 && x < 320) expect(Math.abs(z - 200)).toBeGreaterThan(road.width / 2);
      for (const o of fx.backdrop.slice(i + 1)) expect(overlaps(b.poly, o.poly)).toBe(false);
    }
  });

  test('each faces the road: a side along it', () => {
    for (const b of fx.backdrop) {
      const [p, q] = [b.poly[0], b.poly[1]];
      const along = Math.abs(q[1] - p[1]) / Math.hypot(q[0] - p[0], q[1] - p[1]);
      // (Either side of the rectangle along the road: its first edge along it or across it.)
      expect(Math.min(along, Math.sqrt(1 - along * along))).toBeLessThan(1e-6);
    }
  });

  test("a place's building names its place", () => {
    for (const b of fx.backdrop) if (b.place !== undefined) expect(fx.places[b.place].area).toBe(b.area);
  });
});
