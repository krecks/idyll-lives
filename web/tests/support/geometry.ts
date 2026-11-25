// Small geometry the tests share: a straight polyline, a mesh's vertices, a triangle's height over a point.
import type { Geo } from '../../src/render/shared/geometry';

/** A straight line from (x0, z0) to (x1, z1) in `n` steps (n + 1 points). */
export const line = (x0: number, z0: number, x1: number, z1: number, n: number): [number, number][] => Array.from({ length: n + 1 }, (_, i) => [x0 + ((x1 - x0) * i) / n, z0 + ((z1 - z0) * i) / n]);

/** The vertices of a mesh, as (x, y, z). */
export function points(g: Geo): [number, number, number][] {
  const out: [number, number, number][] = [];
  for (let i = 0; i < g.pos.length; i += 3) out.push([g.pos[i], g.pos[i + 1], g.pos[i + 2]]);
  return out;
}

/** The height of triangle (p, q, r) over (x, z), in plan; undefined where (x, z) is outside its footprint or it stands on edge. */
export function heightOver(p: readonly number[], q: readonly number[], r: readonly number[], x: number, z: number): number | undefined {
  // Barycentric, in plan: is (x, z) inside this triangle's footprint?
  const d = (q[0] - p[0]) * (r[2] - p[2]) - (r[0] - p[0]) * (q[2] - p[2]);
  if (Math.abs(d) < 1e-9) return undefined;
  const u = ((q[0] - x) * (r[2] - z) - (r[0] - x) * (q[2] - z)) / d;
  const v = ((r[0] - x) * (p[2] - z) - (p[0] - x) * (r[2] - z)) / d;
  const w = 1 - u - v;
  if (u < -1e-9 || v < -1e-9 || w < -1e-9) return undefined;
  return u * p[1] + v * q[1] + w * r[1];
}
