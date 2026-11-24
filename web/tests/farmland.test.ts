/**
 * The farmland's patchwork (`render/shared/farmland.ts`): fields farmed where the farmland covers half of them or
 * more, none touching water where nothing clips them, each boundary listed once with its hedge, the same hedges
 * whichever box asks, and the bales on the stubble inside their field; cut at the farmland's edge (a hedge along it)
 * and the dry land's (none on the beach), their crop a margin in from both; and no crop's colour on the ground where
 * the town builds or a road runs (`nature.ts` `terrainColour`).
 */
import { describe, expect, test } from 'bun:test';
import type { RoadPlan } from '../src/core/protocol';
import { CountryRoads } from '../src/render/shared/countryRoads';
import { MARGIN, Patchwork, type FieldClip, type FieldTest } from '../src/render/shared/farmland';
import { LandscapeLayout } from '../src/render/shared/nature';
import { plainWorld } from './support/landFixtures';

/** Farmland in a disc of radius 150 m round (300, 300), a pond at (420, 300). */
const disc: FieldTest = (x, z) => {
  if (Math.hypot(x - 420, z - 300) < 30) return 'never';
  return Math.hypot(x - 300, z - 300) < 150 ? 'in' : 'out';
};
const BOX: [number, number, number, number] = [0, 0, 600, 600];

describe('the patchwork', () => {
  const pw = new Patchwork({ size: 34, angle: 0.4 }, 7, disc);

  test('a field is farmed with half of it or more in the farmland, nothing of it asked in the water', () => {
    const fields = pw.fields(BOX);
    expect(fields.length).toBeGreaterThan(20);
    for (const f of fields) {
      let inside = 0;
      for (const a of [0.125, 0.375, 0.625, 0.875]) {
        for (const b of [0.125, 0.375, 0.625, 0.875]) {
          const [cx, cz] = pw.xz(f.u0 + f.len * a, f.v0 + f.depth * b);
          expect(Math.hypot(cx - 420, cz - 300)).toBeGreaterThanOrEqual(30);
          if (Math.hypot(cx - 300, cz - 300) < 150) inside++;
        }
      }
      expect(inside).toBeGreaterThanOrEqual(8);
      // (Every point of it says it is in it, farmed: nothing clips it.)
      const at = pw.at(...pw.xz(f.u0 + f.len / 2, f.v0 + f.depth / 2));
      expect([at.row, at.col, at.farmed, at.crop]).toEqual([f.row, f.col, true, f.crop]);
      expect(pw.cut(f)).toBeNull();
    }
  });

  test('every boundary of a farmed field once, a hedge only beside one', () => {
    const edges = pw.edges(BOX);
    const keys = edges.map((e) => [e.a, e.b].map((p) => p.map((n) => n.toFixed(3)).join(',')).join(' '));
    expect(new Set(keys).size).toBe(keys.length);
    expect(edges.some((e) => e.hedge)).toBe(true);
    expect(edges.some((e) => !e.hedge)).toBe(true);
    // (A hedge's middle is on the edge of a farmed field: a step either side of it finds one.)
    for (const e of edges.filter((q) => q.hedge)) {
      const [mu, mv] = [(e.a[0] + e.b[0]) / 2, (e.a[1] + e.b[1]) / 2];
      const [nu, nv] = e.a[0] === e.b[0] ? [0.5, 0] : [0, 0.5];
      const sides = [pw.at(...pw.xz(mu - nu, mv - nv)), pw.at(...pw.xz(mu + nu, mv + nv))];
      expect(sides.some((s) => s.farmed)).toBe(true);
      expect(Math.min(...sides.map((s) => s.hedge))).toBeLessThan(0.6);
    }
  });

  test('the same hedges whichever box asks: a square in four gives the whole one’s', () => {
    const hedges = (box: [number, number, number, number]) => pw.edges(box).filter((e) => e.hedge).length;
    const whole = new Patchwork({ size: 34, angle: 0.4 }, 7, disc).edges(BOX).filter((e) => e.hedge);
    expect(hedges(BOX)).toBe(whole.length);
    // (Each stretch found by the quarters it runs through, and by no other.)
    const quarters = [[0, 0, 300, 300], [300, 0, 600, 300], [0, 300, 300, 600], [300, 300, 600, 600]] as [number, number, number, number][];
    const seen = new Set<string>();
    for (const q of quarters) for (const e of pw.edges(q)) if (e.hedge) seen.add(`${e.a.join(',')} ${e.b.join(',')}`);
    expect(seen.size).toBe(whole.length);
  });

  test('bales stand in rows on the stubble, inside their field’s margin', () => {
    const stubble = pw.fields(BOX).filter((f) => f.crop === 'stubble');
    expect(stubble.length).toBeGreaterThan(0);
    let n = 0;
    for (const f of stubble) {
      for (const [u, v] of pw.bales(f)) {
        n++;
        expect(u).toBeGreaterThan(f.u0 + 2);
        expect(u).toBeLessThan(f.u0 + f.len - 2);
        expect(v).toBeGreaterThan(f.v0 + 2);
        expect(v).toBeLessThan(f.v0 + f.depth - 2);
      }
    }
    expect(n).toBeGreaterThan(0);
  });

  test('the same patchwork from the same seed, another from another', () => {
    const again = new Patchwork({ size: 34, angle: 0.4 }, 7, disc).fields(BOX).map((f) => `${f.row},${f.col},${f.crop}`);
    expect(again).toEqual(pw.fields(BOX).map((f) => `${f.row},${f.col},${f.crop}`));
    const other = new Patchwork({ size: 34, angle: 0.4 }, 8, disc).fields(BOX).map((f) => `${f.row},${f.col},${f.crop}`);
    expect(other).not.toEqual(again);
  });
});

/** The farmland a disc of radius 150 m round (300, 300), a river 40 m wide along z = 300 through it with a 6 m beach either side. */
const edge = (x: number, z: number) => 150 - Math.hypot(x - 300, z - 300);
const dry = (_x: number, z: number) => Math.abs(z - 300) - 26;
const clip: FieldClip = (x, z) => [edge(x, z), dry(x, z)];
const clipped: FieldTest = (x, z) => (edge(x, z) > 0 && dry(x, z) > 0 ? 'in' : 'out');

describe('the patchwork cut at the farmland’s edge and the dry land’s', () => {
  const pw = new Patchwork({ size: 34, angle: 0.4 }, 7, clipped, clip);
  const fields = pw.fields(BOX);

  test('fields half in are farmed and cut; a whole one where neither edge comes near', () => {
    const cut = fields.filter((f) => pw.cut(f));
    expect(cut.length).toBeGreaterThan(8);
    expect(fields.length - cut.length).toBeGreaterThan(5);
    // (Their crop: cells inside the margin from both edges, their corners as far in as the line between the grid's points has it.)
    for (const f of cut) {
      for (const poly of pw.cut(f)!.crop) {
        for (let k = 0; k < poly.length; k += 2) {
          const [x, z] = pw.xz(poly[k], poly[k + 1]);
          expect(Math.min(edge(x, z), dry(x, z))).toBeGreaterThan(MARGIN - 0.05);
        }
      }
    }
  });

  test('no point farmed past either edge, and its crop a margin in from them', () => {
    let farmed = 0;
    for (let x = 140; x < 460; x += 1.3) {
      for (let z = 140; z < 460; z += 1.3) {
        const at = pw.at(x, z);
        if (!at.farmed) continue;
        farmed++;
        expect(edge(x, z)).toBeGreaterThan(0);
        expect(dry(x, z)).toBeGreaterThan(0);
        expect(at.edge).toBeLessThanOrEqual(Math.min(edge(x, z), dry(x, z)) + 1e-6);
      }
    }
    expect(farmed).toBeGreaterThan(10_000);
  });

  test('a hedge along the farmland’s edge, none along the water or on the beach', () => {
    const edges = pw.edges(BOX);
    const contour = edges.filter((e) => e.contour);
    expect(contour.length).toBeGreaterThan(50);
    for (const e of contour) {
      expect(e.hedge).toBe(true);
      for (const p of [e.a, e.b]) {
        const [x, z] = pw.xz(...p);
        // (The disc's edge as the grid's points have it: a line between them, a few centimetres inside its curve.)
        expect(Math.abs(edge(x, z))).toBeLessThan(0.1);
        expect(dry(x, z)).toBeGreaterThan(-1);
      }
    }
    // (The patchwork's own lines run on no more than a metre past the farmland's edge, and stop short of the beach.)
    for (const e of edges.filter((q) => !q.contour)) {
      for (const t of [0, 0.5, 1]) {
        const [x, z] = pw.xz(e.a[0] + (e.b[0] - e.a[0]) * t, e.a[1] + (e.b[1] - e.a[1]) * t);
        expect(edge(x, z)).toBeGreaterThan(-2);
        expect(dry(x, z)).toBeGreaterThan(0);
      }
    }
    // (And the ground's hedge shade beside the farmland's edge where a field is cut by it.)
    const [cx, cz] = pw.xz(...contour[0].a);
    expect(pw.at(cx, cz).hedge).toBeLessThan(0.2);
  });

  test('bales only on a stubble field’s crop, where it is cut too', () => {
    let n = 0;
    for (const f of fields.filter((q) => q.crop === 'stubble')) {
      for (const [u, v] of pw.bales(f)) {
        n++;
        const [x, z] = pw.xz(u, v);
        expect(Math.min(edge(x, z), dry(x, z))).toBeGreaterThanOrEqual(MARGIN + 3);
      }
    }
    expect(n).toBeGreaterThan(0);
  });

  test('the same hedges whichever box asks', () => {
    const whole = new Set(new Patchwork({ size: 34, angle: 0.4 }, 7, clipped, clip).edges(BOX).filter((e) => e.hedge).map((e) => `${e.a.join(',')} ${e.b.join(',')}`));
    const seen = new Set<string>();
    const quarters = [[0, 0, 300, 300], [300, 0, 600, 300], [0, 300, 300, 600], [300, 300, 600, 600]] as [number, number, number, number][];
    for (const q of quarters) for (const e of pw.edges(q)) if (e.hedge) seen.add(`${e.a.join(',')} ${e.b.join(',')}`);
    expect([...seen].sort()).toEqual([...whole].sort());
  });
});

describe('the ground’s own colour', () => {
  test('no crop or hedge shade on what is built or a road’s verge', () => {
    // (The fixture's town with a country road along the edge of its fields, a band z 320 to 380 m.)
    const world = plainWorld();
    const road: RoadPlan = { points: [[250, 326], [530, 326]], width: 7, ends: ['join', 'out'], kind: 'country' };
    world.meta!.roads = [...world.meta!.roads!, road];
    const land = new LandscapeLayout(world, (l, w, name) => new CountryRoads(l, w, name));
    const bare = new LandscapeLayout(world, (l, w, name) => new CountryRoads(l, w, name));
    (bare as unknown as { patchwork(): null }).patchwork = () => null;
    const pw = (land as unknown as { patchwork(): Patchwork }).patchwork();
    const [a, b] = [{ r: 0, g: 0, b: 0 }, { r: 0, g: 0, b: 0 }];
    let verge = 0;
    let crop = 0;
    for (let x = 260; x < 520; x += 1.7) {
      for (let z = 316; z < 380; z += 0.9) {
        const at = pw.at(x, z);
        if (!pw.included(at.row, at.col)) continue;
        const h = land.heightAt(x, z);
        const lawn = [land.terrainColour(x, z, h, a), bare.terrainColour(x, z, h, b)];
        if (Math.abs(z - 326) < 3.5 + 2.5) {
          // (On the road and its verge: as with no fields at all.)
          verge++;
          expect(lawn[0]).toBeCloseTo(lawn[1], 6);
          expect([a.r, a.g, a.b]).toEqual([b.r, b.g, b.b]);
        } else if (at.farmed && at.crop !== 'pasture' && at.edge > MARGIN + 4 && land.distanceToTown(x, z) > 3) {
          // (Well inside a field of a crop, off it: the crop's colour.)
          crop++;
          expect(Math.abs(a.r - b.r) + Math.abs(a.g - b.g) + Math.abs(a.b - b.b)).toBeGreaterThan(0.01);
        }
      }
    }
    expect(verge).toBeGreaterThan(100);
    expect(crop).toBeGreaterThan(100);
  });
});
