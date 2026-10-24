/**
 * Bed quilts (residents sleep under the covers), engine-neutral: the quilt's shape, draped over a body's lying pose,
 * and its tint per bed. One quilt mesh per body type, drawn with one instance per resident asleep in a bed (none
 * at all while nobody is), in that bed's colours (`residents.ts` `writeBlanket`); the engines make the mesh and its
 * material (`pc/characterMaterial.ts` `blanketMaterial`, `pc/shaders/blanket.ts`).
 *
 * The quilt is a height field draped over the actual lying pose (the body skinned once on the CPU at
 * load): heights are tented and blurred so the cloth bridges between the feet, knees, hips and the
 * hands resting on the belly, then it drapes over both sides and the foot end and has a turned-down
 * sheet band and a thick edge at the chest. Local frame: the Sim's slot on the mattress (x across,
 * y up from the mattress top, z towards the feet; scale 1); the instance matrix fits it to the Sim's
 * size, the slot width and the bed's foot end.
 */

/** Nominal width (m) of the quilt's top; instances scale it to the slot. */
export const BLANKET_WIDTH = 1.0;
/** Length (m) of the quilt past the toes before it is stretched to the bed. */
const STRETCH = 0.32;
/** Lowest quilt height above the mattress top (clears the bed model's foot runner). */
const FLOOR = 0.1;
const CELL = 0.02;
const DROP = 0.32;

/** A quilt's mesh (`quiltGeometry`) and where its edges are. */
export interface QuiltGeometry {
  /** Vertices (x, y, z), its texture coordinates (u across, v metres from the top edge) and triangles (facing up, the game's winding: clockwise seen from above with +z towards the viewer's bottom); normals are the engine's to compute (smooth, from the triangles). */
  positions: Float32Array;
  uvs: Float32Array;
  indices: number[];
  /** Local z of the foot edge (where the foot drape starts), for fitting to the bed. */
  footZ: number;
  /** Local z just past the toes: the quilt beyond it stretches to the bed's foot end (`blanketExt`). */
  toeZ: number;
  /** Local z of the top (chest) edge. */
  topZ: number;
}

/**
 * Builds the quilt from lying-pose vertex positions (x, y above the mattress, z; scale 1).
 * `chestZ` is where the top edge goes (just below the armpits), `feetZ` the tips of the toes.
 */
export function quiltGeometry(lying: Float32Array, chestZ: number, feetZ: number): QuiltGeometry {
  const topZ = chestZ;
  const toeZ = feetZ + 0.03;
  const footZ = feetZ + STRETCH;
  const half = BLANKET_WIDTH / 2;
  const nx = Math.round(BLANKET_WIDTH / CELL) + 1;
  const nz = Math.round((footZ - topZ) / CELL) + 1;
  const h = new Float32Array(nx * nz);
  // Splat the body (max height per cell, a little dilated).
  for (let v = 0; v < lying.length; v += 3) {
    const x = lying[v];
    const y = lying[v + 1];
    const z = lying[v + 2];
    if (z < topZ - 0.05 || y <= 0) continue;
    const ci = Math.round((x + half) / CELL);
    const cj = Math.round((z - topZ) / CELL);
    for (let dj = -2; dj <= 2; dj++) {
      for (let di = -2; di <= 2; di++) {
        const i = ci + di;
        const j = cj + dj;
        if (i < 0 || j < 0 || i >= nx || j >= nz) continue;
        const k = j * nx + i;
        if (y > h[k]) h[k] = y;
      }
    }
  }
  // Tent: the cloth hangs between high points (gentle along the body, steeper across it).
  const tent = (stride: number, count: number, lines: number, lineStride: number, slope: number) => {
    for (let l = 0; l < lines; l++) {
      const base = l * lineStride;
      for (let a = 1; a < count; a++) h[base + a * stride] = Math.max(h[base + a * stride], h[base + (a - 1) * stride] - slope * CELL);
      for (let a = count - 2; a >= 0; a--) h[base + a * stride] = Math.max(h[base + a * stride], h[base + (a + 1) * stride] - slope * CELL);
    }
  };
  tent(nx, nz, nx, 1, 0.45);
  tent(1, nx, nz, nx, 0.9);
  // Blur (separable box, 3 passes) and lift by the clearance / thickness.
  const tmp = new Float32Array(h.length);
  for (let pass = 0; pass < 3; pass++) {
    for (let j = 0; j < nz; j++)
      for (let i = 0; i < nx; i++) {
        let s = 0;
        let n = 0;
        for (let d = -2; d <= 2; d++) {
          const ii = i + d;
          if (ii >= 0 && ii < nx) {
            s += h[j * nx + ii];
            n++;
          }
        }
        tmp[j * nx + i] = s / n;
      }
    for (let j = 0; j < nz; j++)
      for (let i = 0; i < nx; i++) {
        let s = 0;
        let n = 0;
        for (let d = -2; d <= 2; d++) {
          const jj = j + d;
          if (jj >= 0 && jj < nz) {
            s += tmp[jj * nx + i];
            n++;
          }
        }
        h[j * nx + i] = s / n;
      }
  }
  // Clearance (shoes, breathing); past the toes the cloth falls to the mattress, then lies flat.
  for (let j = 0; j < nz; j++) {
    const z = topZ + j * CELL;
    for (let i = 0; i < nx; i++) {
      const k = j * nx + i;
      let v = h[k] + 0.04;
      if (z > toeZ) v = Math.min(v, h[Math.round((toeZ - topZ) / CELL) * nx + i] + 0.04 - 1.1 * (z - toeZ));
      // Floor: over the bed's own throw / runner at the foot end.
      h[k] = Math.max(v, FLOOR);
    }
  }
  // Edges: the quilt rounds off over the mattress edge before it drops.
  for (let j = 0; j < nz; j++) {
    for (let i = 0; i < nx; i++) {
      const e = Math.min(i, nx - 1 - i) * CELL;
      if (e < 0.08) h[j * nx + i] = Math.max(FLOOR * 0.6, h[j * nx + i] * (0.35 + 0.65 * (e / 0.08)) + 0.012 * (1 - e / 0.08));
    }
  }

  // Grid with drapes: columns [drop, rounded edge, top..., edge, drop], rows [lip, top..., foot edge, drop].
  const xs: number[] = [-half - 0.03, -half - 0.012];
  for (let i = 0; i < nx; i++) xs.push(-half + i * CELL);
  xs.push(half + 0.012, half + 0.03);
  const zs: number[] = [topZ + 0.004];
  for (let j = 0; j < nz; j++) zs.push(topZ + j * CELL);
  zs.push(footZ + 0.012, footZ + 0.03);
  const W = xs.length;
  const D = zs.length;
  const pos = new Float32Array(W * D * 3);
  const uv = new Float32Array(W * D * 2);
  const hAt = (c: number, r: number) => h[clampI(r, 0, nz - 1) * nx + clampI(c, 0, nx - 1)];
  for (let r = 0; r < D; r++) {
    for (let c = 0; c < W; c++) {
      const gi = c - 2;
      const gj = r - 1;
      let y = hAt(gi, gj);
      const sideDrop = c === 0 || c === W - 1 ? DROP : c === 1 || c === W - 2 ? 0.05 : 0;
      const footDrop = r === D - 1 ? DROP : r === D - 2 ? 0.05 : 0;
      y -= Math.max(sideDrop, footDrop);
      if (r === 0) y = 0.012; // thick top edge, tucked to the mattress
      const k = r * W + c;
      pos[k * 3] = xs[c];
      pos[k * 3 + 1] = y;
      pos[k * 3 + 2] = zs[r];
      // u: across (0..1 over the top, beyond on the drapes); v: metres from the top edge.
      uv[k * 2] = (xs[c] + half) / BLANKET_WIDTH;
      uv[k * 2 + 1] = zs[r] - topZ + (r === 0 ? -0.02 : 0) + footDrop;
    }
  }
  const idx: number[] = [];
  for (let r = 0; r < D - 1; r++) {
    for (let c = 0; c < W - 1; c++) {
      const a = r * W + c;
      const b = a + 1;
      const cc = a + W + 1;
      const d = a + W;
      // Facing up (the game's winding: clockwise seen from above with z towards the viewer's bottom).
      idx.push(a, d, cc, a, cc, b);
    }
  }
  return { positions: pos, uvs: uv, indices: idx, footZ, toeZ, topZ };
}

function clampI(v: number, lo: number, hi: number): number {
  return v < lo ? lo : v > hi ? hi : v;
}

/** Depth (m, from the top edge) of the turned-down sheet band, which keeps its own colour. */
export const SHEET = 0.17;
/** The cover's grey in the texture (sRGB byte): a quilt's tint is its bed's colour over it (`quiltTint`). */
export const COVER = 0xe6;

/** An sRGB hex colour channel (`#rrggbb`, from `at`) in linear light: the byte / 255, to the power 2.2 (the game's decoding). */
function hexLinear(hex: string, at: number): number {
  return hex.substring(0, 1) !== '#' || hex.length !== 7 ? 0 : Math.pow(parseInt(hex.substring(at, at + 2), 16) / 255, 2.2);
}

/** A quilt's `blanketTint` (linear rgb) for a bed's bedding colour (sRGB hex): that colour where the cover is plain. */
export function quiltTint(hex: string, out: Float32Array, o: number): void {
  const k = 1 / Math.pow(COVER / 255, 2.2);
  out[o] = hexLinear(hex, 1) * k;
  out[o + 1] = hexLinear(hex, 3) * k;
  out[o + 2] = hexLinear(hex, 5) * k;
}
