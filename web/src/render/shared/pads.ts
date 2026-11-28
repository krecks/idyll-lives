/**
 * The lots of a land the town follows as the ground draws them (engine-neutral; `pc/nature.ts` draws
 * them): each lot is a level pad (`builtSurface.ts`), and a 2 m terrain can't hold the step at its edge,
 * so inside a lot the terrain isn't drawn (`padMask`: the ground's `GAME_PAD_MASK`) and the
 * pad is a slab of its own at its level, coloured as the terrain round it; where the land beside a lot
 * stands higher or lower than its pad by `STEP_LEAST` or more, a retaining edge stands from the lower to
 * the higher, `EDGE_FOOT` into the ground (`padGeometry`).
 */

import type { BuiltSurface } from './builtSurface';
import { Geo, type V3 } from './geometry';

/** The least step (m) between a pad and the land beside it that gets a retaining edge. */
export const STEP_LEAST = 0.04;
/** How far a retaining edge reaches below the lower of the two (m): no gap under it where the land dips. */
export const EDGE_FOOT = 0.3;
/**
 * A slab's edges hang this far down (m), and reach a third as far out under the land beside, in the slab's own colours,
 * where the land meets the pad at its edge (no retaining edge): the slab and the terrain beside it are two meshes, and
 * the hairline between them showed the sky through it, dot by dot along the edge. Not where a retaining edge stands:
 * in its plane, the two fought, in teeth along it.
 */
export const SLAB_LIP = 0.15;
/** A slab's points are this far apart (m) on the land's grain (its colours follow the terrain's). */
const SLAB_STEP = 2;
/** A retaining edge's pieces are this long at most (m). */
const EDGE_STEP = 1;
/** The retaining edges' tint over the foundations' stone (sRGB). */
const EDGE_TINT = '#E2DCD2';

/** A lot as the pads take it: its rectangle and its pad (m). */
interface Pad {
  x: number;
  z: number;
  w: number;
  d: number;
  y: number;
}

/** The pads of the first `count` of `surface`'s plots (the world's lots). */
export function padsOf(surface: BuiltSurface, count: number): Pad[] {
  const out: Pad[] = [];
  for (let i = 0; i < Math.min(count, surface.plots.length); i++) {
    const p = surface.plots[i];
    if (p.w > 0 && p.d > 0) out.push({ x: p.x, z: p.z, w: p.w, d: p.d, y: surface.padY(i) });
  }
  return out;
}

/** The pad mask: a byte a metre over a `w` × `d` m world from its corner, 255 where the metre's middle is inside a lot (the ground's `GAME_PAD_MASK`). */
export function padMask(pads: readonly Pad[], w: number, d: number): Uint8Array {
  const [cols, rows] = [Math.max(1, Math.ceil(w)), Math.max(1, Math.ceil(d))];
  const out = new Uint8Array(cols * rows);
  for (const p of pads) {
    const [i0, i1] = [Math.max(0, Math.ceil(p.x - 0.5)), Math.min(cols - 1, Math.floor(p.x + p.w - 0.5))];
    const [j0, j1] = [Math.max(0, Math.ceil(p.z - 0.5)), Math.min(rows - 1, Math.floor(p.z + p.d - 0.5))];
    for (let j = j0; j <= j1; j++) out.fill(255, j * cols + i0, j * cols + i1 + 1);
  }
  return out;
}

/** The slabs and the retaining edges of one group of lots (a ground tile's). */
interface PadGeometry {
  slabs: Geo;
  edges: Geo;
}

/**
 * The slabs and retaining edges of `pads`, grouped by `groupOf` (where a slab's lot's middle is, where an
 * edge piece's middle is). `colour(x, z, y, out)`: the terrain's colour there (sRGB 0..1 into `out`) and how
 * much of it is lawn (the slab's vertex alpha, as the terrain's). `uv(x, z)`: the ground's texture
 * coordinates there (as the terrain's). The land beside a lot is `land` at its edge (what the terrain draws
 * there: `LandscapeLayout.terrainAt`), or the pad of a lot beside it (the higher of two draws the edge between
 * them).
 */
export function padGeometry(
  land: (x: number, z: number) => number,
  pads: readonly Pad[],
  groupOf: (x: number, z: number) => number,
  colour: (x: number, z: number, y: number, out: { r: number; g: number; b: number }) => number,
  uv: (x: number, z: number) => [number, number],
): Map<number, PadGeometry> {
  const out = new Map<number, PadGeometry>();
  const group = (x: number, z: number): PadGeometry => {
    const k = groupOf(x, z);
    let g = out.get(k);
    if (!g) out.set(k, (g = { slabs: new Geo().color('#FFFFFF'), edges: new Geo().color(EDGE_TINT) }));
    return g;
  };
  const tmp = { r: 0, g: 0, b: 0 };
  const up: V3 = [0, 1, 0];
  // Which pad holds a point (the highest), and the land beside a lot.
  const padAt = (x: number, z: number, not: Pad): number => {
    let best = -Infinity;
    for (const q of pads) if (q !== not && x >= q.x - 1e-6 && x <= q.x + q.w + 1e-6 && z >= q.z - 1e-6 && z <= q.z + q.d + 1e-6) best = Math.max(best, q.y);
    return best;
  };
  for (const p of pads) {
    // The slab: on the land's 2 m grid inside it, its own edges too.
    const xs = steps(p.x, p.x + p.w, SLAB_STEP);
    const zs = steps(p.z, p.z + p.d, SLAB_STEP);
    const g = group(p.x + p.w / 2, p.z + p.d / 2);
    const vert = (x: number, z: number) => {
      const lawn = colour(x, z, p.y, tmp);
      return { at: [x, p.y, z] as V3, tint: [tmp.r, tmp.g, tmp.b] as V3, lawn, uv: uv(x, z) };
    };
    const row: ReturnType<typeof vert>[][] = zs.map((z) => xs.map((x) => vert(x, z)));
    for (let j = 0; j + 1 < zs.length; j++) {
      for (let i = 0; i + 1 < xs.length; i++) {
        const q = [row[j][i], row[j][i + 1], row[j + 1][i + 1], row[j + 1][i]];
        g.slabs.poly(
          q.map((v) => v.at),
          up,
          q.map((v) => v.uv),
          q.map((v) => v.lawn),
          q.map((v) => v.tint),
        );
      }
    }
    // Its retaining edges, a piece a metre round it.
    const sides: [number, number, number, number, number, number][] = [
      // (from x, z, to x, z, outward x, z)
      [p.x, p.z, p.x + p.w, p.z, 0, -1],
      [p.x + p.w, p.z, p.x + p.w, p.z + p.d, 1, 0],
      [p.x + p.w, p.z + p.d, p.x, p.z + p.d, 0, 1],
      [p.x, p.z + p.d, p.x, p.z, -1, 0],
    ];
    for (const [ax, az, bx, bz, nx, nz] of sides) {
      const len = Math.hypot(bx - ax, bz - az);
      const n = Math.max(1, Math.ceil(len / EDGE_STEP - 1e-6));
      const beside = (x: number, z: number): number => {
        const other = padAt(x + nx * 0.01, z + nz * 0.01, p);
        if (other > -Infinity) return other > p.y ? NaN : other;
        return land(x, z);
      };
      const at = (t: number): [number, number] => [ax + ((bx - ax) * t) / n, az + ((bz - az) * t) / n];
      for (let k = 0; k < n; k++) {
        const [[x0, z0], [x1, z1]] = [at(k), at(k + 1)];
        const [o0, o1] = [beside(x0, z0), beside(x1, z1)];
        if (Number.isNaN(o0) || Number.isNaN(o1)) continue;
        if (Math.abs(o0 - p.y) < STEP_LEAST && Math.abs(o1 - p.y) < STEP_LEAST) {
          // Where no retaining edge stands (the land beside within `STEP_LEAST` of the pad), the slab's lip (`SLAB_LIP`): down
          // from its edge, both faces, and out under the land beside. (Not where an edge stands: in its plane, the two fought.)
          const lip = (q: V3[], n: V3) => {
            const lawn: number[] = [];
            const tint: V3[] = [];
            for (const v of q) {
              lawn.push(colour(v[0], v[2], p.y, tmp));
              tint.push([tmp.r, tmp.g, tmp.b]);
            }
            const l = Math.hypot(...n);
            g.slabs.poly(
              q,
              [n[0] / l, n[1] / l, n[2] / l],
              q.map((v) => uv(v[0], v[2])),
              lawn,
              tint,
            );
          };
          const [t0, t1, d0, d1]: V3[] = [
            [x0, p.y, z0],
            [x1, p.y, z1],
            [x0, p.y - SLAB_LIP, z0],
            [x1, p.y - SLAB_LIP, z1],
          ];
          // (Each face lit nearly as the top, tipped its own way a little: enough to face it so.)
          for (const s of [1, -1]) lip([t0, t1, d1, d0], [nx * s * 0.2, 1, nz * s * 0.2]);
          // (Steeply: a land a little lower than the pad stays over it.)
          const out = SLAB_LIP / 3;
          lip([t0, t1, [x1 + nx * out, p.y - SLAB_LIP, z1 + nz * out], [x0 + nx * out, p.y - SLAB_LIP, z0 + nz * out]], up);
          continue;
        }
        const [top0, top1] = [Math.max(o0, p.y), Math.max(o1, p.y)];
        const [low0, low1] = [Math.min(o0, p.y) - EDGE_FOOT, Math.min(o1, p.y) - EDGE_FOOT];
        // (Facing the lower side: out from a pad over the land, in from the land over a pad.)
        const face: V3 = (o0 + o1) / 2 < p.y ? [nx, 0, nz] : [-nx, 0, -nz];
        group((x0 + x1) / 2, (z0 + z1) / 2).edges.poly(
          [
            [x0, low0, z0],
            [x1, low1, z1],
            [x1, top1, z1],
            [x0, top0, z0],
          ],
          face,
        );
      }
    }
  }
  return out;
}

/** `a` to `b` by the land's grain: `a`, every multiple of `step` strictly between, `b`. */
function steps(a: number, b: number, step: number): number[] {
  const out = [a];
  for (let v = Math.floor(a / step + 1e-6) * step + step; v < b - 1e-6; v += step) if (v > a + 1e-6) out.push(v);
  out.push(b);
  return out;
}
