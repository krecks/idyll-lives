// The street's and the garden's hard ground toned by vertex, engine-neutral (`render/shared/paving.ts`):
// paving darker at the sides the lawn is on and clean down a narrow path's middle, a drive darker where
// wheels run, the asphalt pale in the wheels' tracks and darkest along the kerbs, the same road always the same.
import { describe, expect, test } from 'bun:test';
import { Geo } from '../src/render/shared/geometry';
import { carriageway, pavedRects, repair, STREET_LOOK } from '../src/render/shared/paving';
import { roadMeshes } from '../src/render/shared/roads';

/** The red of the vertex nearest `(x, z)` in `g` (0..1, sRGB). */
function redAt(g: Geo, x: number, z: number): number {
  let [best, at] = [Infinity, 0];
  for (let i = 0; i < g.pos.length / 3; i++) {
    const d = Math.hypot(g.pos[i * 3] - x, g.pos[i * 3 + 2] - z);
    if (d < best) [best, at] = [d, i];
  }
  return g.col[at * 4];
}

describe('paving', () => {
  test('a path is darker and greener at its sides, clean down its middle and at its ends', () => {
    const path = { x: 4, z: 2, w: 1, d: 6 };
    const g = new Geo();
    pavedRects(g, [path], [path], 0.014);
    const [side, middle] = [redAt(g, 4, 5), redAt(g, 4.5, 5)];
    expect(side).toBeLessThan(middle * 0.75);
    expect(middle).toBeGreaterThan(0.88);
    // Greener at the side: its red under its green there.
    const i = g.pos.findIndex((v, k) => k % 3 === 0 && v === 4 && g.pos[k + 2] === 5) / 3;
    expect(g.col[i * 4]).toBeLessThan(g.col[i * 4 + 1]);
    // The end, at a door or the pavement, is as its middle is.
    expect(redAt(g, 4.5, 2)).toBeGreaterThan(0.88);
    for (const c of g.col) expect(c).toBeLessThanOrEqual(1);
  });

  test('a side another path goes on from is not an edge', () => {
    const [a, b] = [{ x: 0, z: 0, w: 3, d: 3 }, { x: 3, z: 0, w: 3, d: 3 }];
    const g = new Geo();
    pavedRects(g, [a], [a, b], 0.014);
    expect(redAt(g, 3, 1.5)).toBeGreaterThan(redAt(g, 0, 1.5) * 1.2);
  });

  test('a drive is darker where wheels run', () => {
    const drive = { x: 0, z: 0, w: 3, d: 6 };
    const g = new Geo();
    pavedRects(g, [drive], [drive], 0.014);
    // (Compared a metre apart along one line across it: the patches differ little there.)
    expect(redAt(g, 0.75, 3)).toBeLessThan(redAt(g, 1.5, 3));
    expect(redAt(g, 2.25, 3)).toBeLessThan(redAt(g, 1.5, 3));
  });

  test('a carriageway is pale where wheels run and darkest along its kerbs, the same on both sides', () => {
    for (const half of [1.65, 3, 4]) {
      const { at, tone } = carriageway(half);
      expect(at[0]).toBeCloseTo(-half);
      expect(at[at.length - 1]).toBeCloseTo(half);
      for (const o of at) expect(tone(o)).toBeCloseTo(tone(-o));
      expect(tone(half)).toBe(STREET_LOOK.gutter);
      const wheel = half >= 2.6 ? half / 2 + 0.8 : 0.8;
      expect(tone(wheel)).toBe(STREET_LOOK.wheel);
      for (let o = -half; o <= half; o += 0.1) expect(tone(o)).toBeGreaterThanOrEqual(STREET_LOOK.gutter - 1e-9);
    }
  });

  test('a road has the same repairs every time, in about one stretch in seven', () => {
    let n = 0;
    for (let k = 0; k < 2000; k++) {
      const fix = repair(3, k);
      expect(repair(3, k)).toEqual(fix);
      if (!fix) continue;
      n++;
      expect(Math.abs(fix.across) + fix.wide).toBeLessThanOrEqual(0.92 + 1e-9);
    }
    expect(n / 2000).toBeGreaterThan(STREET_LOOK.patches * 0.7);
    expect(n / 2000).toBeLessThan(STREET_LOOK.patches * 1.3);
  });

  test('a worn road covers what the plain one does, toned between its kerbs', () => {
    const road = { points: [[0, 10], [40, 10]] as [number, number][], width: 9, ends: ['out', 'out'] as ['out', 'out'] };
    const plain = roadMeshes([road], null, 0, {}, 'surface').surface;
    const worn = roadMeshes([road], null, 0, {}, 'surface', () => 9 / 2 - 1.35).surface;
    const span = (g: Geo, k: number) => [Math.min(...g.pos.filter((_, i) => i % 3 === k)), Math.max(...g.pos.filter((_, i) => i % 3 === k))];
    expect(span(worn, 0)).toEqual(span(plain, 0));
    expect(span(worn, 2)).toEqual(span(plain, 2));
    expect(plain.col.every((c) => c === 1)).toBe(true);
    expect(worn.col.some((c) => c < 0.9)).toBe(true);
    expect(worn.idx.length).toBeGreaterThan(plain.idx.length);
  });
});
