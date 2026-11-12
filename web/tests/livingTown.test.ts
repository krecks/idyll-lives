// The living town's drawing at town scale (docs/design/living-town.md, "Town-scale drawing"): a backdrop
// building fitted to the town view's unit meshes (`render/shared/townBuildings.ts`), their faces facing out, and
// the debug fixture (`debug/ltFixture.ts`, `?ltFixture`) keeping the contract's rules on a made-up town.
import { describe, expect, test } from 'bun:test';
import type { BackdropBuilding, PlotInfo, RoadPlan, WorldStructure } from '../src/core/protocol';
import { ltFixture } from '../src/debug/ltFixture';
import { BACKDROP_PALETTE, backdropBuilding, lotBuilding, ROOFS, unitMeshes } from '../src/render/shared/townBuildings';

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
