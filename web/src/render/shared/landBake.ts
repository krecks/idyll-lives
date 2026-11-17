/**
 * The land as a height field (`game/land.ts`) on the renderer's side: surveyed from the land as it is drawn
 * (`bakeLand`, in the render data worker), and what the drawing reads of it besides its heights: how far each
 * point is from the water's edge (`wetDistance`), the water standing above the sea's level as quads
 * (`pondQuads`), and where a ray from the eye meets the ground (`marchToGround`, the brushes' picking).
 * Engine-neutral.
 */

import { blankLand, COVER, COVER_CELL, LAND_CELL, NONE, SEA_CM, type Land, type LandRect } from '../../game/land';

/** What `bakeLand` reads of the landscape (`LandscapeLayout`): the land's height, the water's edge and the zones' cover. */
export interface BakeSource {
  ground(x: number, z: number): number;
  waterEdge(x: number, z: number): number;
  /** What covers the ground at (x, z) (`COVER`). */
  coverAt(x: number, z: number): number;
}

/**
 * The land of a `w` × `d` m world as `layout` draws it: each sample's height its `ground` (rounded to the
 * centimetre), the sea's level (`SEA_CM`) on the samples past the water's edge, and each cover cell what the
 * zones have at its middle. Drawn from it (`LandscapeLayout.setLand`), the land stands where it stood.
 */
export function bakeLand(layout: BakeSource, w: number, d: number): Land {
  const land = blankLand(w, d, 'drawn');
  const { cols, rows, h, wl, cover } = land;
  for (let j = 0; j < rows; j++) {
    const z = j * LAND_CELL;
    for (let i = 0, k = j * cols; i < cols; i++, k++) {
      const x = i * LAND_CELL;
      h[k] = Math.max(-32767, Math.min(32767, Math.round(layout.ground(x, z) * 100)));
      if (layout.waterEdge(x, z) > 0) wl[k] = SEA_CM;
    }
  }
  const cw = Math.ceil((cols * LAND_CELL) / COVER_CELL);
  const cd = cover.length / cw;
  for (let j = 0; j < cd; j++) for (let i = 0; i < cw; i++) cover[j * cw + i] = layout.coverAt((i + 0.5) * COVER_CELL, (j + 0.5) * COVER_CELL);
  return land;
}

/** A copy of `land` (its arrays its own). */
export function copyLand(land: Land): Land {
  return { ...land, h: land.h.slice(), wl: land.wl.slice(), cover: land.cover.slice(), ...(land.flow ? { flow: land.flow.slice() } : {}) };
}

/** Whether two lands hold the same heights, water, rivers' ways (none and all still alike) and cover. */
export function sameLand(a: Land, b: Land): boolean {
  if (a.cols !== b.cols || a.rows !== b.rows || a.cover.length !== b.cover.length) return false;
  for (let k = 0; k < a.h.length; k++) if (a.h[k] !== b.h[k] || a.wl[k] !== b.wl[k] || (a.flow?.[k] ?? 0) !== (b.flow?.[k] ?? 0)) return false;
  for (let k = 0; k < a.cover.length; k++) if (a.cover[k] !== b.cover[k]) return false;
  return true;
}

/** The whole of `land`'s samples. */
export function fullRect(land: Land): LandRect {
  return { i0: 0, j0: 0, i1: land.cols, j1: land.rows };
}

/** The box (m: x0, z0, x1, z1) a rectangle of samples spans, out to the samples beside it (whose heights it leans on). */
export function rectBox(rect: LandRect, pad = 0): [number, number, number, number] {
  return [(rect.i0 - 1) * LAND_CELL - pad, (rect.j0 - 1) * LAND_CELL - pad, rect.i1 * LAND_CELL + pad, rect.j1 * LAND_CELL + pad];
}

/** Whether sample `k`'s water stands at the sea's level (the water sheet draws it). */
const seaAt = (land: Land, k: number) => land.wl[k] === SEA_CM;

/** Whether any sample is under water, and any at the sea's level. */
export function landWater(land: Land): { any: boolean; sea: boolean } {
  let any = false;
  for (let k = 0; k < land.h.length; k++) {
    const v = land.wl[k];
    if (v === NONE || v <= land.h[k]) continue;
    any = true;
    if (v === SEA_CM) return { any, sea: true };
  }
  return { any, sea: false };
}

/** The land's highest sample (m). */
export function landTop(land: Land): number {
  let top = -32768;
  for (let k = 0; k < land.h.length; k++) if (land.h[k] > top) top = land.h[k];
  return top / 100;
}

/**
 * How far each sample is inside the water (m; below 0: out of it, that far from it): the distance to the nearest
 * sample on the other side (two chamfer passes each way), less half a sample's spacing, signed. `null` with no
 * water at all.
 */
export function wetDistance(land: Land): Float32Array | null {
  const { cols, rows } = land;
  const wet = new Uint8Array(cols * rows);
  let any = false;
  for (let k = 0; k < wet.length; k++) {
    if (land.wl[k] !== NONE && land.wl[k] > land.h[k]) {
      wet[k] = 1;
      any = true;
    }
  }
  if (!any) return null;
  const toWet = chamfer(wet, cols, rows, 1);
  const toDry = chamfer(wet, cols, rows, 0);
  const half = LAND_CELL / 2;
  for (let k = 0; k < wet.length; k++) toWet[k] = wet[k] ? toDry[k] - half : -(toWet[k] - half);
  return toWet;
}

/** Distance (m) from each sample to the nearest whose `mask` is `value` (two chamfer passes). */
export function chamfer(mask: Uint8Array, gw: number, gd: number, value: 0 | 1): Float32Array {
  const dist = new Float32Array(gw * gd);
  const far = (gw + gd) * LAND_CELL;
  for (let i = 0; i < dist.length; i++) dist[i] = mask[i] === value ? 0 : far;
  const c = LAND_CELL;
  const diag = c * Math.SQRT2;
  for (let z = 0; z < gd; z++) {
    for (let x = 0; x < gw; x++) {
      const i = z * gw + x;
      let v = dist[i];
      if (x > 0) v = Math.min(v, dist[i - 1] + c);
      if (z > 0) {
        v = Math.min(v, dist[i - gw] + c);
        if (x > 0) v = Math.min(v, dist[i - gw - 1] + diag);
        if (x < gw - 1) v = Math.min(v, dist[i - gw + 1] + diag);
      }
      dist[i] = v;
    }
  }
  for (let z = gd - 1; z >= 0; z--) {
    for (let x = gw - 1; x >= 0; x--) {
      const i = z * gw + x;
      let v = dist[i];
      if (x < gw - 1) v = Math.min(v, dist[i + 1] + c);
      if (z < gd - 1) {
        v = Math.min(v, dist[i + gw] + c);
        if (x < gw - 1) v = Math.min(v, dist[i + gw + 1] + diag);
        if (x > 0) v = Math.min(v, dist[i + gw - 1] + diag);
      }
      dist[i] = v;
    }
  }
  return dist;
}

/**
 * `wetDistance`'s field at (x, z): read between the samples, and past the land's edge the edge's carried on out
 * (deeper in water that reaches it, further from water that doesn't). -Infinity without water.
 */
export function wetEdgeAt(land: Land, sd: Float32Array | null, x: number, z: number): number {
  if (!sd) return -Infinity;
  const { cols, rows } = land;
  const fx = x / LAND_CELL;
  const fz = z / LAND_CELL;
  const cx = Math.min(cols - 1, Math.max(0, fx));
  const cz = Math.min(rows - 1, Math.max(0, fz));
  const out = Math.hypot(fx - cx, fz - cz) * LAND_CELL;
  const i = Math.min(Math.floor(cx), Math.max(0, cols - 2));
  const j = Math.min(Math.floor(cz), Math.max(0, rows - 2));
  const [u, v] = [cx - i, cz - j];
  const [i1, j1] = [Math.min(i + 1, cols - 1), Math.min(j + 1, rows - 1)];
  const e = (sd[j * cols + i] * (1 - u) + sd[j * cols + i1] * u) * (1 - v) + (sd[j1 * cols + i] * (1 - u) + sd[j1 * cols + i1] * u) * v;
  return e > 0 ? e + out : e - out;
}

/** The cover cell's value at (x, z) (`COVER`; open off the land). */
export function coverAt(land: Land, x: number, z: number): number {
  const cw = Math.ceil((land.cols * LAND_CELL) / COVER_CELL);
  const cd = land.cover.length / cw;
  const i = Math.floor(x / COVER_CELL);
  const j = Math.floor(z / COVER_CELL);
  return i < 0 || j < 0 || i >= cw || j >= cd ? COVER.open : land.cover[j * cw + i];
}

/** How far (m) a pond's water reaches up its bank past where the field's ground meets it: under the ground drawn there, no gap at the shore. */
const SHORE_LAP = 0.08;

/**
 * The water above the sea's level over the cells (a cell: the square between four samples) whose corner (x0, z0)
 * lies in [x0, x1) × [z0, z1) (m): for each cell with a corner under such water, the part of it where the ground
 * (between its corners) is under the water, wound up as the ground is. The water stands at each wet corner's own
 * level and slopes between them (bilinear over the cell): flat on a pond, and where a river steps down from one
 * level to the next (or into the sea, whose corners count at its level) a rapid, not a step. A dry corner takes the
 * highest level of the water beside it (its eight neighbours), the same for every cell round it, so two cells meet
 * along their shared edge and the water has no gap between them. Its shore follows the field's (each cell's part
 * cut where the ground crosses the water, a line from edge to edge), however coarse the ground is drawn round it.
 * Each point's normal is the slope of the water there (straight up on a pond). Null where there is none; the
 * sea's is the water sheet's.
 */
export function pondQuads(land: Land, x0: number, z0: number, x1: number, z1: number): { positions: Float32Array; normals: Float32Array; indices: Uint32Array; cells: number } | null {
  const { cols, rows, h, wl } = land;
  const i0 = Math.max(0, Math.ceil(x0 / LAND_CELL));
  const j0 = Math.max(0, Math.ceil(z0 / LAND_CELL));
  const i1 = Math.min(cols - 1, Math.ceil(x1 / LAND_CELL));
  const j1 = Math.min(rows - 1, Math.ceil(z1 / LAND_CELL));
  const pos: number[] = [];
  const nrm: number[] = [];
  const idx: number[] = [];
  let cells = 0;
  const wet = (k: number) => wl[k] !== NONE && wl[k] > h[k];
  const pond = (k: number) => wet(k) && !seaAt(land, k);
  /** The water's level at sample `k` (cm): its own where it is wet, else the highest of its wet neighbours'. */
  const levelAt = (k: number) => {
    if (wet(k)) return wl[k];
    const [i, j] = [k % cols, Math.floor(k / cols)];
    let top = NONE;
    for (let b = Math.max(0, j - 1); b <= Math.min(rows - 1, j + 1); b++) {
      for (let a = Math.max(0, i - 1); a <= Math.min(cols - 1, i + 1); a++) {
        const n = b * cols + a;
        if (wet(n) && wl[n] > top) top = wl[n];
      }
    }
    return top;
  };
  const c = LAND_CELL;
  // (A cell's corners round it, the water's level at each (m), and how far under it each stands.)
  const corner = [0, 0, 0, 0];
  const lv = [0, 0, 0, 0];
  const under = [0, 0, 0, 0];
  const ring: number[] = [];
  for (let j = j0; j < j1; j++) {
    for (let i = i0; i < i1; i++) {
      const k = j * cols + i;
      // Round the cell: (i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1).
      [corner[0], corner[1], corner[2], corner[3]] = [k, k + 1, k + cols + 1, k + cols];
      if (!pond(corner[0]) && !pond(corner[1]) && !pond(corner[2]) && !pond(corner[3])) continue;
      // (A dry sample is out of the water whatever its height: the cut stays short of one lower than the level.)
      for (let q = 0; q < 4; q++) {
        const kq = corner[q];
        lv[q] = levelAt(kq) / 100;
        const u = lv[q] + SHORE_LAP - h[kq] / 100;
        under[q] = wet(kq) ? u : Math.min(u, -SHORE_LAP);
      }
      // The water's slope over the cell at (u, v) of it (its rise along x and along z, m a metre).
      const slope = (u: number, v: number): [number, number] => [((lv[1] - lv[0]) * (1 - v) + (lv[2] - lv[3]) * v) / c, ((lv[3] - lv[0]) * (1 - u) + (lv[2] - lv[1]) * u) / c];
      // The cell's wet part: its corners under the water, and where each edge crosses it.
      ring.length = 0;
      for (let q = 0; q < 4; q++) {
        const [cu, cv] = [q === 1 || q === 2 ? 1 : 0, q >= 2 ? 1 : 0];
        const n = (q + 1) % 4;
        const [nu, nv] = [n === 1 || n === 2 ? 1 : 0, n >= 2 ? 1 : 0];
        if (under[q] >= 0) ring.push(cu, cv, lv[q]);
        if (under[q] >= 0 !== under[n] >= 0) {
          const t = under[q] / (under[q] - under[n]);
          ring.push(cu + (nu - cu) * t, cv + (nv - cv) * t, lv[q] + (lv[n] - lv[q]) * t);
        }
      }
      if (ring.length < 9) continue;
      cells++;
      const first = pos.length / 3;
      for (let v = 0; v < ring.length; v += 3) {
        const [u, w] = [ring[v], ring[v + 1]];
        pos.push((i + u) * c, ring[v + 2], (j + w) * c);
        const [sx, sz] = slope(u, w);
        const l = Math.hypot(sx, 1, sz);
        nrm.push(-sx / l, 1 / l, -sz / l);
      }
      // (A fan, wound as the sheet is: front up.)
      for (let v = 1; v + 1 < ring.length / 3; v++) idx.push(first, first + v, first + v + 1);
    }
  }
  return cells ? { positions: Float32Array.from(pos), normals: Float32Array.from(nrm), indices: Uint32Array.from(idx), cells } : null;
}

/**
 * Where a ray from `origin` along `dir` (unit length, game space) first meets the ground `height` gives: marched
 * in `step` m steps until it is below the ground, then bisected between the last two. Null if it starts under the
 * ground, or doesn't meet it within `reach` m (or rises past `top`, the land's highest, without meeting it).
 */
export function marchToGround(
  origin: { x: number; y: number; z: number },
  dir: { x: number; y: number; z: number },
  height: (x: number, z: number) => number,
  reach = 6000,
  step = 2,
  top = Infinity,
): { x: number; y: number; z: number } | null {
  const below = (t: number) => origin.y + dir.y * t <= height(origin.x + dir.x * t, origin.z + dir.z * t);
  if (below(0)) return null;
  let t0 = 0;
  for (let t = step; t <= reach + step; t += step) {
    if (dir.y >= 0 && origin.y + dir.y * t > top) return null;
    if (!below(t)) {
      t0 = t;
      continue;
    }
    let [a, b] = [t0, t];
    for (let k = 0; k < 24; k++) {
      const m = (a + b) / 2;
      if (below(m)) b = m;
      else a = m;
    }
    const [x, z] = [origin.x + dir.x * b, origin.z + dir.z * b];
    return { x, y: height(x, z), z };
  }
  return null;
}
