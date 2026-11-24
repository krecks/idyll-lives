/**
 * Masks over a world's ground and the polygons drawn round them: run-length masks (`Rle`), a cell
 * mask's smoothed outlines, polygons filled back into 1 m tiles, and the polyline simplifying they
 * use. The world generator (`macro.ts`) lays its water and countryside with them, and the land
 * editor turns painted cover and water back into `WorldZone`s with them.
 */

import type { Rle } from '../core/protocol';
import { dist, inside, round2, type Pt } from './roads';
import * as pmath from './pmath';

/** The world generator's land cells (m): `outlines`' cells unless told otherwise. */
const MACRO_CELL = 8;


/** A polyline with the points that hardly change its course taken out (Douglas-Peucker, `tol` m). */
export function simplify(pts: readonly Pt[], tol: number): Pt[] {
  if (pts.length < 3) return [...pts];
  const keep = new Uint8Array(pts.length);
  keep[0] = keep[pts.length - 1] = 1;
  const stack: [number, number][] = [[0, pts.length - 1]];
  while (stack.length) {
    const [a, b] = stack.pop()!;
    const [[ax, az], [bx, bz]] = [pts[a], pts[b]];
    const l = pmath.hypot(bx - ax, bz - az) || 1;
    let [far, at] = [0, -1];
    for (let i = a + 1; i < b; i++) {
      const d = Math.abs((pts[i][0] - ax) * (bz - az) - (pts[i][1] - az) * (bx - ax)) / l;
      if (d > far) [far, at] = [d, i];
    }
    if (far > tol && at > 0) {
      keep[at] = 1;
      stack.push([a, at], [at, b]);
    }
  }
  return pts.filter((_, i) => keep[i]);
}

/** A closed polygon simplified as `simplify` does a polyline: in two halves, from its first point and the point furthest from it. */
export function simplifyRing(ring: readonly Pt[], tol: number): Pt[] {
  if (ring.length < 4) return [...ring];
  let far = 0;
  for (let i = 1; i < ring.length; i++) if (dist(ring[i], ring[0]) > dist(ring[far], ring[0])) far = i;
  const a = simplify(ring.slice(0, far + 1), tol);
  const b = simplify([...ring.slice(far), ring[0]], tol);
  return [...a, ...b.slice(1, -1)];
}

/** A closed polygon with its corners cut (Chaikin), once. */
export function chaikin(poly: readonly Pt[]): Pt[] {
  const out: Pt[] = [];
  for (let i = 0; i < poly.length; i++) {
    const [a, b] = [poly[i], poly[(i + 1) % poly.length]];
    out.push([a[0] * 0.75 + b[0] * 0.25, a[1] * 0.75 + b[1] * 0.25], [a[0] * 0.25 + b[0] * 0.75, a[1] * 0.25 + b[1] * 0.75]);
  }
  return out;
}

const area2 = (poly: readonly Pt[]) => poly.reduce((n, p, i) => n + p[0] * poly[(i + 1) % poly.length][1] - poly[(i + 1) % poly.length][0] * p[1], 0);

/**
 * A closed polygon with every edge longer than twice `most` m given a point `most` m in from each end: cut by
 * `chaikin` (a quarter of each edge), a corner then rounds over a quarter of that at most, however long its edges run.
 */
function cornersHeld(poly: readonly Pt[], most: number): Pt[] {
  const out: Pt[] = [];
  for (let i = 0; i < poly.length; i++) {
    const [a, b] = [poly[i], poly[(i + 1) % poly.length]];
    out.push(a);
    const l = dist(a, b);
    if (l <= 2 * most) continue;
    const t = most / l;
    out.push([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t], [b[0] - (b[0] - a[0]) * t, b[1] - (b[1] - a[1]) * t]);
  }
  return out;
}

/**
 * The outlines of a cell mask as polygons (m), smoothed: each piece of land one ring, its holes
 * joined to it by a seam (so filled even-odd, as `inside` does, the holes stay open). The mask is
 * `gw` × `gd` cells of `cell` m, row-major; pieces under `minCells` cells are left out. Each straight
 * run of the mask's edge is one edge, its corners rounded over a quarter of the edges either side of
 * them; with `round`, over `round` m of them at most, so a corner between long straight runs stays
 * within about 0.8 × `round` m of the mask's (with none, a straight cross of rivers 400 m long is rounded
 * some 70 m into its dry corners).
 */
export function outlines(mask: Uint8Array, gw: number, gd: number, minCells = 6, cell = MACRO_CELL, round = Infinity): Pt[][] {
  const on = (x: number, z: number) => x >= 0 && z >= 0 && x < gw && z < gd && mask[z * gw + x] === 1;
  // Edges round each set cell where its neighbour isn't, clockwise (z down the page): corner keys.
  const key = (x: number, z: number) => z * (gw + 1) + x;
  const next = new Map<number, number[]>();
  const edge = (ax: number, az: number, bx: number, bz: number) => {
    const k = key(ax, az);
    const list = next.get(k);
    if (list) list.push(key(bx, bz));
    else next.set(k, [key(bx, bz)]);
  };
  let cells = 0;
  for (let z = 0; z < gd; z++) {
    for (let x = 0; x < gw; x++) {
      if (!on(x, z)) continue;
      cells++;
      if (!on(x, z - 1)) edge(x, z, x + 1, z);
      if (!on(x + 1, z)) edge(x + 1, z, x + 1, z + 1);
      if (!on(x, z + 1)) edge(x + 1, z + 1, x, z + 1);
      if (!on(x - 1, z)) edge(x, z + 1, x, z);
    }
  }
  if (!cells) return [];
  const loops: Pt[][] = [];
  for (const start of [...next.keys()]) {
    while (next.get(start)?.length) {
      const loop: Pt[] = [];
      let k = start;
      do {
        loop.push([(k % (gw + 1)) * cell, Math.floor(k / (gw + 1)) * cell]);
        const list = next.get(k)!;
        const n = list.pop()!;
        if (!list.length) next.delete(k);
        k = n;
      } while (k !== start && next.has(k));
      // Straight runs as one edge.
      const corners = loop.filter((p, i) => {
        const [a, b] = [loop[(i + loop.length - 1) % loop.length], loop[(i + 1) % loop.length]];
        return (b[0] - p[0]) * (p[1] - a[1]) !== (b[1] - p[1]) * (p[0] - a[0]);
      });
      if (corners.length >= 3 && Math.abs(area2(corners)) / 2 >= minCells * cell * cell) loops.push(chaikin(chaikin(round < Infinity ? cornersHeld(corners, round * 4) : corners)));
    }
  }
  const outer = loops.filter((l) => area2(l) > 0).sort((a, b) => area2(a) - area2(b));
  const holes = loops.filter((l) => area2(l) < 0);
  const rings = outer.map((l) => [...l]);
  for (const hole of holes) {
    const o = rings.findIndex((ring, i) => inside(outer[i], hole[0][0], hole[0][1]) && ring.length);
    if (o < 0) continue;
    const ring = rings[o];
    let [bi, bj, bd] = [0, 0, Infinity];
    for (let i = 0; i < ring.length; i += 2) {
      for (let j = 0; j < hole.length; j += 2) {
        const d = dist(ring[i], hole[j]);
        if (d < bd) [bi, bj, bd] = [i, j, d];
      }
    }
    rings[o] = [...ring.slice(0, bi + 1), ...hole.slice(bj), ...hole.slice(0, bj + 1), ...ring.slice(bi)];
  }
  return rings.map((r) => simplifyRing(r, 0.6).map(([x, z]): Pt => [round2(x), round2(z)]));
}

/** Tiles (1 m) of a `W` × `D` world inside any of `polys` (even-odd, at the tiles' middles). */
export function rasterize(polys: readonly Pt[][], W: number, D: number): Uint8Array {
  const mask = new Uint8Array(W * D);
  for (const poly of polys) {
    for (let z = 0; z < D; z++) {
      const zc = z + 0.5;
      const xs: number[] = [];
      for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
        const [[xi, zi], [xj, zj]] = [poly[i], poly[j]];
        if (zi > zc !== zj > zc) xs.push(((xj - xi) * (zc - zi)) / (zj - zi) + xi);
      }
      xs.sort((a, b) => a - b);
      for (let k = 0; k + 1 < xs.length; k += 2) {
        for (let x = Math.max(0, Math.ceil(xs[k] - 0.5)); x < Math.min(W, Math.ceil(xs[k + 1] - 0.5)); x++) mask[z * W + x] ^= 1;
      }
    }
  }
  return mask;
}

/** A 0/1 mask, run-length (`Rle`): [run, value, run, value, ...] from tile 0. */
export function rle(mask: Uint8Array): Rle {
  const out: Rle = [];
  let [run, value] = [0, mask[0] ?? 0];
  for (const v of mask) {
    if (v === value) run++;
    else {
      out.push(run, value);
      [run, value] = [1, v];
    }
  }
  if (run) out.push(run, value);
  return out;
}

/** A run-length mask (`Rle`) back to one value per tile. */
export function unrle(runs: Rle, size: number): Uint8Array {
  const mask = new Uint8Array(size);
  let at = 0;
  for (let i = 0; i + 1 < runs.length; i += 2) {
    if (runs[i + 1]) mask.fill(1, at, Math.min(size, at + runs[i]));
    at += runs[i];
  }
  return mask;
}
