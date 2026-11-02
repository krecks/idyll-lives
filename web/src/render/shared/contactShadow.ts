/**
 * Contact shadows shaped like what casts them: a model's darkness on the floor, worked out once
 * from its geometry at rest (`contactShape`), and an atlas of those (`contactAtlas`) that one
 * instanced draw reads, each copy its own tile (`render/pc/lotContents.ts`).
 *
 * The shape is how near the model comes to the floor, seen from above: every triangle laid flat
 * on a grid round its outline, each cell keeping the lowest point over it. What touches the floor
 * (a cabinet's plinth, a chair's feet) is darkest; a seat or a table top above it darkens less, the
 * higher it is (`HALF`); a lamp's shade hardly at all. Softened by two box blurs, so the shadow
 * spills past the model's edge as the round blob did, and its opacity the shade's square root: the
 * spill, half the shade at the edge and fading out, is what shows round the model.
 *
 * A tile is `CONTACT_TILE` texels a side (one byte each: the shadow's opacity), its outermost ring
 * left clear so that the tiles sampled side by side in the atlas don't run into each other. Rows
 * go along +z and columns along +x of the model's space (the quad's v and u: `contactBlob`).
 */

import { dataBox } from './modelParts';

/** Texels a side of a tile. */
export const CONTACT_TILE = 32;
/** Cells a side of the grid the triangles are laid on, per texel (thin legs fall between cells less). */
const SUPERSAMPLE = 4;
/** Room round the model's outline (m): where the blur spills. */
const MARGIN = 0.3;
/** Radius of each of the two box blurs (m; a texel at least). */
const BLUR = 0.12;
/** Height above the model's bottom (m) at which something over the floor shades it half as dark as touching it. */
const HALF = 0.5;
/** The shadow's opacity where the model touches the floor. */
const PEAK = 0.6;

/** A model's contact shadow: its tile, and the rectangle of model space (m, x and z) it covers. */
export interface ContactShape {
  minX: number;
  maxX: number;
  minZ: number;
  maxZ: number;
  /** `CONTACT_TILE`² opacities, rows along +z. */
  alpha: Uint8Array;
}

/** Geometry to lay flat: positions (x, y, z) and triangles. */
export interface ContactPiece {
  positions: ArrayLike<number>;
  indices: ArrayLike<number>;
}

/** The contact shadow of `pieces` (model space, at rest), or undefined when they have no triangles. */
export function contactShape(pieces: readonly ContactPiece[]): ContactShape | undefined {
  if (!pieces.some((p) => p.indices.length >= 3)) return undefined;
  const lo = [Infinity, Infinity, Infinity];
  const hi = [-Infinity, -Infinity, -Infinity];
  for (const box of pieces.map((p) => dataBox(p.positions))) {
    for (let k = 0; k < 3; k++) {
      lo[k] = Math.min(lo[k], box.lo[k]);
      hi[k] = Math.max(hi[k], box.hi[k]);
    }
  }
  if (!Number.isFinite(lo[0] + lo[1] + lo[2] + hi[0] + hi[2])) return undefined;
  const [loX, loY, loZ] = lo;
  const [hiX, , hiZ] = hi;
  const minX = loX - MARGIN;
  const minZ = loZ - MARGIN;
  const w = hiX - loX + 2 * MARGIN;
  const d = hiZ - loZ + 2 * MARGIN;
  const N = CONTACT_TILE * SUPERSAMPLE;
  // The lowest point over each cell (its centre), above the model's bottom.
  const near = new Float32Array(N * N).fill(Infinity);
  const sx = N / w;
  const sz = N / d;
  for (const { positions: p, indices } of pieces) {
    for (let t = 0; t + 2 < indices.length; t += 3) {
      const a = indices[t] * 3;
      const b = indices[t + 1] * 3;
      const c = indices[t + 2] * 3;
      // In cells, a cell's centre at whole numbers.
      const ax = (p[a] - minX) * sx - 0.5, az = (p[a + 2] - minZ) * sz - 0.5;
      const bx = (p[b] - minX) * sx - 0.5, bz = (p[b + 2] - minZ) * sz - 0.5;
      const cx = (p[c] - minX) * sx - 0.5, cz = (p[c + 2] - minZ) * sz - 0.5;
      const area = (bx - ax) * (cz - az) - (cx - ax) * (bz - az);
      // (Upright faces cover no floor.)
      if (Math.abs(area) < 1e-9) continue;
      const inv = 1 / area;
      const ay = p[a + 1] - loY;
      const by = p[b + 1] - loY;
      const cy = p[c + 1] - loY;
      const i0 = Math.max(0, Math.ceil(Math.min(ax, bx, cx)));
      const i1 = Math.min(N - 1, Math.floor(Math.max(ax, bx, cx)));
      const j0 = Math.max(0, Math.ceil(Math.min(az, bz, cz)));
      const j1 = Math.min(N - 1, Math.floor(Math.max(az, bz, cz)));
      for (let j = j0; j <= j1; j++) {
        for (let i = i0; i <= i1; i++) {
          // Barycentric weights (either winding: the signed areas share the triangle's sign).
          const wa = ((bx - i) * (cz - j) - (cx - i) * (bz - j)) * inv;
          const wb = ((cx - i) * (az - j) - (ax - i) * (cz - j)) * inv;
          const wc = 1 - wa - wb;
          if (wa < -1e-6 || wb < -1e-6 || wc < -1e-6) continue;
          const y = wa * ay + wb * by + wc * cy;
          const k = j * N + i;
          if (y < near[k]) near[k] = y;
        }
      }
    }
  }
  // Each texel: the shade of its cells, averaged.
  const T = CONTACT_TILE;
  let shade = new Float32Array(T * T);
  for (let j = 0; j < N; j++) {
    for (let i = 0; i < N; i++) {
      const h = near[j * N + i];
      if (h === Infinity) continue;
      const r = h / HALF;
      shade[((j / SUPERSAMPLE) | 0) * T + ((i / SUPERSAMPLE) | 0)] += 1 / (1 + r * r) / (SUPERSAMPLE * SUPERSAMPLE);
    }
  }
  const rx = Math.max(1, Math.round(BLUR / (w / T)));
  const rz = Math.max(1, Math.round(BLUR / (d / T)));
  for (let pass = 0; pass < 2; pass++) shade = boxBlur(boxBlur(shade, rx, 1, T), rz, T, T);
  const alpha = new Uint8Array(T * T);
  for (let j = 1; j < T - 1; j++) for (let i = 1; i < T - 1; i++) alpha[j * T + i] = Math.round(255 * PEAK * Math.sqrt(Math.min(1, shade[j * T + i])));
  return { minX, maxX: minX + w, minZ, maxZ: minZ + d, alpha };
}

/** `src` (a `size`² grid) blurred by a box `r` texels each way along one axis (`step`: 1 along rows, `size` across them); past the edge is clear. */
function boxBlur(src: Float32Array, r: number, step: number, size: number): Float32Array<ArrayBuffer> {
  const out = new Float32Array(src.length);
  const across = step === 1 ? size : 1;
  for (let line = 0; line < size; line++) {
    const base = line * across;
    let sum = 0;
    for (let k = 0; k < r; k++) sum += src[base + k * step];
    for (let k = 0; k < size; k++) {
      if (k + r < size) sum += src[base + (k + r) * step];
      if (k - r - 1 >= 0) sum -= src[base + (k - r - 1) * step];
      out[base + k * step] = sum / (2 * r + 1);
    }
  }
  return out;
}

/** Tiles side by side: the atlas's size and texels (a byte each), and each tile's rectangle in it (u, v, width, height; 0 to 1). */
export interface ContactAtlas {
  width: number;
  height: number;
  texels: Uint8Array;
  rects: Float32Array[];
}

/** The atlas of `tiles` (`CONTACT_TILE`² each), in rows as square as they go. */
export function contactAtlas(tiles: readonly Uint8Array[]): ContactAtlas {
  const T = CONTACT_TILE;
  const cols = Math.max(1, Math.ceil(Math.sqrt(tiles.length)));
  const rows = Math.max(1, Math.ceil(tiles.length / cols));
  const width = cols * T;
  const height = rows * T;
  const texels = new Uint8Array(width * height);
  const rects = tiles.map((tile, n) => {
    const [c, r] = [n % cols, Math.floor(n / cols)];
    for (let j = 0; j < T; j++) texels.set(tile.subarray(j * T, j * T + T), (r * T + j) * width + c * T);
    return new Float32Array([(c * T) / width, (r * T) / height, T / width, T / height]);
  });
  return { width, height, texels, rects };
}
