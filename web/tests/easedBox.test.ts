import { expect, test } from 'bun:test';
import { EASE, Geo, type V3 } from '../src/render/shared/geometry';

/**
 * Built boxes with their edges chamfered (`Geo.easing`, `easedBox`): only the edges the gate would count, and the box
 * still a closed solid wound outwards, whatever faces it leaves out, staying inside its own extent.
 */

const SIZES = [0.005, 0.02, 0.04, 0.07, 0.09, 0.3, 1.2];
const SKIPS = ['', 'ny', 'nx px', 'ny py', 'ny pz', 'nx px ny', 'ny nz', 'py'];

function triangles(g: Geo): { a: V3; b: V3; c: V3; n: V3 }[] {
  const [p, nor, idx] = [g.pos, g.nor, g.idx];
  const v = (i: number): V3 => [p[i * 3], p[i * 3 + 1], p[i * 3 + 2]];
  const out = [];
  for (let t = 0; t + 2 < idx.length; t += 3) {
    out.push({ a: v(idx[t]), b: v(idx[t + 1]), c: v(idx[t + 2]), n: [nor[idx[t] * 3], nor[idx[t] * 3 + 1], nor[idx[t] * 3 + 2]] as V3 });
  }
  return out;
}

const key = (p: V3) => p.map((v) => Math.round(v * 1e6)).join(',');

for (const sx of SIZES) {
  for (const sy of SIZES) {
    for (const sz of SIZES) {
      test(`a ${sx} × ${sy} × ${sz} box`, () => {
        for (const skip of SKIPS) {
          const g = new Geo().easing(EASE).box(0, 0, 0, sx, sy, sz, skip);
          const tris = triangles(g);
          expect(tris.length).toBeGreaterThan(0);
          const centre: V3 = [sx / 2, sy / 2, sz / 2];
          // Each edge of a triangle runs the other way in exactly one other triangle, but round the faces left out.
          const directed = new Map<string, number>();
          for (const { a, b, c, n } of tris) {
            // (Geo's front faces: (a - b) × (c - b) along the normal, as models.ts has them.)
            const e1: V3 = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            const e2: V3 = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];
            const f: V3 = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
            const area = Math.hypot(...f);
            expect(area).toBeGreaterThan(1e-12);
            // Wound to its normal, and facing out of the box.
            expect(f[0] * n[0] + f[1] * n[1] + f[2] * n[2]).toBeGreaterThan(0);
            const m = [(a[0] + b[0] + c[0]) / 3 - centre[0], (a[1] + b[1] + c[1]) / 3 - centre[1], (a[2] + b[2] + c[2]) / 3 - centre[2]];
            expect(m[0] * n[0] + m[1] * n[1] + m[2] * n[2]).toBeGreaterThan(0);
            for (const p of [a, b, c]) {
              for (const [i, s] of [sx, sy, sz].entries()) {
                expect(p[i]).toBeGreaterThanOrEqual(-1e-9);
                expect(p[i]).toBeLessThanOrEqual(s + 1e-9);
              }
            }
            for (const [p, q] of [[a, b], [b, c], [c, a]]) {
              const k = `${key(p)}>${key(q)}`;
              directed.set(k, (directed.get(k) ?? 0) + 1);
            }
          }
          // (A plain box's faces are whole quads: its edges are met the same way. Only a skipped face's rim is open.)
          const open = [...directed.keys()].filter((k) => !directed.has(k.split('>').reverse().join('>')));
          for (const k of open) {
            const [p, q] = k.split('>').map((s) => s.split(',').map((v) => Number(v) / 1e6));
            // An open edge lies in the plane of a face left out.
            const onSkipped = (['nx', 'px', 'ny', 'py', 'nz', 'pz'] as const).some((face, f) => {
              if (!skip.includes(face)) return false;
              const axis = f >> 1;
              const plane = f & 1 ? [sx, sy, sz][axis] : 0;
              return Math.abs(p[axis] - plane) < 1e-6 && Math.abs(q[axis] - plane) < 1e-6;
            });
            expect(onSkipped).toBe(true);
          }
          for (const n of directed.values()) expect(n).toBe(1);
        }
      });
    }
  }
}

test('only the edges the gate counts are chamfered', () => {
  const count = (g: Geo) => g.idx.length / 3;
  // A small piece (under 10 cm), and a thin board's: sharp, a plain box's twelve triangles.
  expect(count(new Geo().easing(EASE).box(0, 0, 0, 0.04, 0.04, 0.04))).toBe(12);
  expect(count(new Geo().easing(EASE).box(0, 0, 0, 1, 0.3, 0.008))).toBe(12);
  // A bar 2 cm square: its four long edges, its ends cut at the corners (8 sides + 8 chamfers + 2 × 6 ends).
  expect(count(new Geo().easing(EASE).box(0, 0, 0, 0.02, 1, 0.02))).toBe(28);
  // A big box: every edge, a triangle at each corner (12 + 24 + 8).
  expect(count(new Geo().easing(EASE).box(0, 0, 0, 0.3, 0.3, 0.3))).toBe(44);
  // Sharp with no easing.
  expect(count(new Geo().box(0, 0, 0, 0.3, 0.3, 0.3))).toBe(12);
});
