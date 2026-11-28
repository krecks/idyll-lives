/**
 * The pieces of dry land a shaped land's water cuts it into, and where a bridge could cross the
 * water between two of them. The generator (`macro.ts` `generateFromSpec`) puts a village on each
 * piece big enough and joins it to the town over a bridge; step 2's Village tool (`edit/villages.ts`)
 * bridges a village it puts across the water. A leaf: it imports only types (`bridges.ts` imports
 * the generator, which imports this).
 */

import type { Cells } from './countryRouter';
import type { Pt } from './roads';
import type { Rect } from './town';
import * as pmath from './pmath';

/** The generator's cells (m), as `countryRouter.ts` `CELL`. */
const CELL = 8;

/** Where a box's corner may go: [x0, z0, x1, z1] (m). */
export type Bounds = [number, number, number, number];

/** A land's pieces of dry land (8 m cells joined side to side; see `dryPieces`). */
interface DryPieces {
  /** Per 8 m cell, the piece it is on; -1 where it is water. */
  piece: Int32Array;
  /** Each piece's size (cells), by id: the ids go in the order the cells are read from the corner. */
  sizes: number[];
  /** The pieces' ids, the largest first (the first read first among those the same size). */
  bySize: number[];
  /** The piece under point `p` (m); -1 on the water or off the land. */
  of(p: Pt): number;
  /** Whether box `r` and the land `by` m round it stand all on piece `id`. */
  holds(id: number, r: Rect, by: number): boolean;
  /** The place nearest box `r`'s (its corner, in `CELL` steps from `bounds`' corner) where it stands with `by` m round it on piece `id` (where it is, if it does there); null if nowhere. */
  nearest(id: number, r: Rect, bounds: Bounds, by?: number): Pt | null;
  /** As `nearest`, on whichever of `ids` (all, largest first, if not given) has the nearest such place: that place and the piece; null if none has one. */
  nearestAny(r: Rect, bounds: Bounds, by?: number, ids?: readonly number[]): { at: Pt; id: number } | null;
}

/**
 * The pieces of dry land on `land`'s 8 m cells (the generator's: `macro.ts` `fillFrom`), found by
 * flooding from each dry cell not yet on one, read from the corner. Whether a box stands on one
 * is told from a sum of the cells off it (made for a piece when first asked about).
 */
export function dryPieces(land: Pick<Cells, 'gw' | 'gd' | 'water'>): DryPieces {
  const { gw, gd } = land;
  const piece = new Int32Array(gw * gd).fill(-1);
  const sizes: number[] = [];
  for (let start = 0; start < gw * gd; start++) {
    if (land.water[start] || piece[start] >= 0) continue;
    const id = sizes.length;
    const stack = [start];
    piece[start] = id;
    let n = 0;
    while (stack.length) {
      const i = stack.pop()!;
      n++;
      const [x, z] = [i % gw, Math.floor(i / gw)];
      for (const [nx, nz] of [[x - 1, z], [x + 1, z], [x, z - 1], [x, z + 1]] as const) {
        const j = nz * gw + nx;
        if (nx < 0 || nz < 0 || nx >= gw || nz >= gd || land.water[j] || piece[j] >= 0) continue;
        piece[j] = id;
        stack.push(j);
      }
    }
    sizes.push(n);
  }
  const bySize = sizes.map((_, id) => id).sort((a, b) => sizes[b] - sizes[a] || a - b);
  /** Per piece, its cells off it summed from the corner. */
  const sats = new Map<number, Int32Array>();
  const satOf = (id: number) => {
    let sat = sats.get(id);
    if (sat) return sat;
    sat = new Int32Array((gw + 1) * (gd + 1));
    for (let z = 0; z < gd; z++) {
      let row = 0;
      for (let x = 0; x < gw; x++) {
        row += piece[z * gw + x] === id ? 0 : 1;
        sat[(z + 1) * (gw + 1) + x + 1] = sat[z * (gw + 1) + x + 1] + row;
      }
    }
    sats.set(id, sat);
    return sat;
  };
  const of = ([x, z]: Pt) => {
    const [cx, cz] = [Math.floor(x / CELL), Math.floor(z / CELL)];
    return cx < 0 || cz < 0 || cx >= gw || cz >= gd ? -1 : piece[cz * gw + cx];
  };
  const holds = (id: number, r: Rect, by: number) => {
    const [x0, z0] = [Math.floor((r.x - by) / CELL), Math.floor((r.z - by) / CELL)];
    const [x1, z1] = [Math.floor((r.x + r.w + by) / CELL) + 1, Math.floor((r.z + r.d + by) / CELL) + 1];
    if (id < 0 || id >= sizes.length || x0 < 0 || z0 < 0 || x1 > gw || z1 > gd) return false;
    const sat = satOf(id);
    return sat[z1 * (gw + 1) + x1] - sat[z0 * (gw + 1) + x1] - sat[z1 * (gw + 1) + x0] + sat[z0 * (gw + 1) + x0] === 0;
  };
  /** The nearest place on piece `id` within `far` m of the box's corner; null if none is nearer. */
  const search = (id: number, r: Rect, [bx0, bz0, bx1, bz1]: Bounds, by: number, far: number): { at: Pt; d: number } | null => {
    if (r.x >= bx0 && r.z >= bz0 && r.x <= bx1 && r.z <= bz1 && holds(id, r, by)) return { at: [Math.round(r.x), Math.round(r.z)], d: 0 };
    let best: { at: Pt; d: number } | null = null;
    for (let z = bz0; z <= bz1; z += CELL) {
      for (let x = bx0; x <= bx1; x += CELL) {
        const d = pmath.hypot(x - r.x, z - r.z);
        if (d >= far || !holds(id, { x, z, w: r.w, d: r.d }, by)) continue;
        best = { at: [Math.round(x), Math.round(z)], d };
        far = d;
      }
    }
    return best;
  };
  const nearest = (id: number, r: Rect, bounds: Bounds, by = CELL) => search(id, r, bounds, by, Infinity)?.at ?? null;
  const nearestAny = (r: Rect, bounds: Bounds, by = CELL, ids: readonly number[] = bySize) => {
    // (A piece with fewer cells than the box has can't hold it.)
    const least = Math.floor(r.w / CELL) * Math.floor(r.d / CELL);
    let best: { at: Pt; id: number; d: number } | null = null;
    for (const id of ids) {
      if (sizes[id] < least) continue;
      const found = search(id, r, bounds, by, best ? best.d : Infinity);
      if (found && (!best || found.d < best.d)) best = { ...found, id };
    }
    return best ? { at: best.at, id: best.id } : null;
  };
  return { piece, sizes, bySize, of, holds, nearest, nearestAny };
}

/**
 * Where a bridge could cross the water: from bank `a` to bank `b` (each the first dry point past the
 * water, m), the way across `u` (unit, `a` to `b`: square to both banks), and how far that is (m).
 */
interface BridgeSite {
  a: Pt;
  b: Pt;
  u: Pt;
  across: number;
}

/** The lattice the banks are looked for on (m), how near the water a point on it is a bank's (m), how far round the water is read for a bank's way (m). */
const SEED = 6;
const NEAR = 3;
const AROUND = 8;
/** Steps along a line across the water (m). */
const STEP = 0.5;
/** Two banks this near parallel (radians) are crossed square to both. */
const PARALLEL = (35 * Math.PI) / 180;

const round2 = (v: number) => Math.round(v * 100) / 100;

/**
 * The places a straight bridge could cross the water of `wet` (`W` × `D` tiles of 1 m, 1 wet;
 * `landEdit.ts` `wetTiles`), whatever the water is (a painted river, a lake's neck): from points
 * on a `SEED` m lattice by the water, across it the way its bank faces (from the wet tiles round
 * the point), to the far bank; where that faces back within `PARALLEL`, the line square to both
 * banks through the water's middle, `most` m across at most, with `landing` m of dry land in the
 * world past each bank. One site for each place (their middles 2 m apart, either way across),
 * in the order they are found; no dice.
 */
export function bridgeSites(wet: Uint8Array, W: number, D: number, most = 40, landing = 24): BridgeSite[] {
  const tile = (x: number, z: number) => (x < 0 || z < 0 || x >= W || z >= D ? -1 : wet[Math.floor(z) * W + Math.floor(x)]);
  const isWet = (x: number, z: number) => tile(x, z) === 1;
  const isDry = (x: number, z: number) => tile(x, z) === 0;
  /** The way into the water from (x, z): the mean offset of the wet tiles within `AROUND` m; null if none. */
  const into = (x: number, z: number): Pt | null => {
    let [sx, sz, n] = [0, 0, 0];
    for (let dz = -AROUND; dz <= AROUND; dz++) {
      for (let dx = -AROUND; dx <= AROUND; dx++) {
        if (dx * dx + dz * dz > AROUND * AROUND || !isWet(x + dx, z + dz)) continue;
        [sx, sz, n] = [sx + dx, sz + dz, n + 1];
      }
    }
    const l = pmath.hypot(sx, sz);
    return n && l > 1e-6 ? [sx / l, sz / l] : null;
  };
  /** How far from `p` along `d` the water ends (m: the first dry point), within `max`; -1 if it doesn't (or leaves the world). */
  const out = (p: Pt, d: Pt, max: number) => {
    for (let s = 0; s <= max; s += STEP) {
      const t = tile(p[0] + d[0] * s, p[1] + d[1] * s);
      if (t === 0) return s;
      if (t < 0) return -1;
    }
    return -1;
  };
  const nearWater = (x: number, z: number) => {
    for (let dz = -NEAR; dz <= NEAR; dz++) for (let dx = -NEAR; dx <= NEAR; dx++) if (isWet(x + dx, z + dz)) return true;
    return false;
  };
  const sites: BridgeSite[] = [];
  const seen = new Set<string>();
  const cos = pmath.cos(PARALLEL);
  // (Not by the world's edge: the water past it is not read, and its bank's way would lean.)
  for (let z = SEED / 2; z < D; z += SEED) {
    for (let x = SEED / 2; x < W; x += SEED) {
      if (Math.min(x, z, W - x, D - z) < AROUND * 2 || !isDry(x, z) || !nearWater(x, z)) continue;
      const n = into(x, z);
      if (!n) continue;
      // Across the way the bank faces: where the water begins, then where it ends.
      let t0 = -1;
      for (let s = 0; s <= NEAR * 2; s += STEP) if (isWet(x + n[0] * s, z + n[1] * s)) { t0 = s; break; }
      if (t0 < 0) continue;
      const w = out([x + n[0] * t0, z + n[1] * t0], n, most + 1);
      if (w < 0) continue;
      const far: Pt = [x + n[0] * (t0 + w), z + n[1] * (t0 + w)];
      const m = Math.min(far[0], far[1], W - far[0], D - far[1]) < AROUND * 2 ? null : into(...far);
      if (!m || -(m[0] * n[0] + m[1] * n[1]) < cos) continue;
      // Square to both banks, through the water's middle.
      let u: Pt = [n[0] - m[0], n[1] - m[1]];
      const l = pmath.hypot(...u);
      u = [u[0] / l, u[1] / l];
      if (u[0] < 0 || (u[0] === 0 && u[1] < 0)) u = [-u[0], -u[1]];
      const mid: Pt = [x + n[0] * (t0 + w / 2), z + n[1] * (t0 + w / 2)];
      if (!isWet(...mid)) continue;
      const da = out(mid, [-u[0], -u[1]], most);
      const db = out(mid, u, most);
      if (da < 0 || db < 0 || da + db > most) continue;
      // (Its banks a little onto the dry land: rounded, they stay there.)
      const a: Pt = [mid[0] - u[0] * (da + 0.1), mid[1] - u[1] * (da + 0.1)];
      const b: Pt = [mid[0] + u[0] * (db + 0.1), mid[1] + u[1] * (db + 0.1)];
      let landed = true;
      for (let s = 0; s <= landing && landed; s += 1) landed = isDry(a[0] - u[0] * s, a[1] - u[1] * s) && isDry(b[0] + u[0] * s, b[1] + u[1] * s);
      if (!landed) continue;
      const c: Pt = [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2];
      const key = `${Math.round(c[0] / 2)},${Math.round(c[1] / 2)}`;
      if (seen.has(key)) continue;
      seen.add(key);
      sites.push({ a: [round2(a[0]), round2(a[1])], b: [round2(b[0]), round2(b[1])], u: [Math.round(u[0] * 1e4) / 1e4, Math.round(u[1] * 1e4) / 1e4], across: round2(da + db) });
    }
  }
  return sites;
}
