/**
 * The farmland's patchwork (`render/shared/farmland.ts`): fields farmed whole where the farmland covers most of them,
 * none touching water, each boundary listed once with its hedge, the same hedges whichever box asks, and the bales
 * on the stubble inside their field.
 */
import { describe, expect, test } from 'bun:test';
import { Patchwork, type FieldTest } from '../src/render/shared/farmland';

/** Farmland in a disc of radius 150 m round (300, 300), a pond at (420, 300). */
const disc: FieldTest = (x, z) => {
  if (Math.hypot(x - 420, z - 300) < 30) return 'never';
  return Math.hypot(x - 300, z - 300) < 150 ? 'in' : 'out';
};
const BOX: [number, number, number, number] = [0, 0, 600, 600];

describe('the patchwork', () => {
  const pw = new Patchwork({ size: 34, angle: 0.4 }, 7, disc);

  test('a field is farmed whole: its middle in the farmland, nothing of it in the water', () => {
    const fields = pw.fields(BOX);
    expect(fields.length).toBeGreaterThan(20);
    for (const f of fields) {
      const [x, z] = pw.xz(f.u0 + f.len / 2, f.v0 + f.depth / 2);
      expect(Math.hypot(x - 300, z - 300)).toBeLessThan(150);
      for (const [a, b] of [[0.12, 0.12], [0.88, 0.12], [0.12, 0.88], [0.88, 0.88]]) {
        const [cx, cz] = pw.xz(f.u0 + f.len * a, f.v0 + f.depth * b);
        expect(Math.hypot(cx - 420, cz - 300)).toBeGreaterThanOrEqual(30);
      }
      // (Every point of it says it is in it, farmed.)
      const at = pw.at(x, z);
      expect([at.row, at.col, at.farmed, at.crop]).toEqual([f.row, f.col, true, f.crop]);
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
