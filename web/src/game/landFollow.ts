/**
 * A new place's land as a height field whose town follows its slopes (`Land.seeded` `'follow'`):
 * seeded from the generator's own heights (`OpenWorldDraft.heights`), the built land not
 * flattened, then the roads, the lots and the land round them graded into it
 * (`render/shared/builtSurface.ts`), so the land a world stores agrees with its town. Classic
 * worlds and older saves have none (they stay flat); a land a player shaped keeps its heights and
 * is graded again under the town laid on it.
 */

import { DEFAULT_LANDSCAPE, type RoadPlan, type WorldStructure, type WorldZone } from '../core/protocol';
import { builtSurfaceOf, type BuiltSurface, type SurfaceWorld } from '../render/shared/builtSurface';
import { blankLand, COVER, COVER_CELL, coverSpan, LAND_CELL, SEA_CM, type Land } from './land';
import { APRON, builtOf, Protection } from './landEdit';
import { fbm, ridged, smoothstep } from './noise';
import type { NeighbourhoodDraft } from './town';
import type { Pt } from './roads';

/** The steepest a town's or a village's ground is tilted (rise per m: `gentler`). */
export const GENTLE_TILT = 0.06;

/** An area's lots this near its box and streets (m) are on its gentle ground (`gentler`); those further out have their own. */
const AREA_LOTS = 30;
/** How much more a town's or a village's plane has to say in the blend than a lone lot's (`gentler`). */
const TOWN_SAY = 1000;

/** Dry land stays this far over the sea's level (m) in a world with water: the sea is one sheet over the whole world. */
const DRY_OVER = 0.15;
/** The land beside water rises from this far over it (m), no steeper than the world's slope. */
const SHORE_OVER = 0.1;
/** A pond's level lies this far below the lowest of the land round it (m). */
const POND_BELOW = 0.1;
/** Water that reaches the world's edge or is bigger than this (m²) is at the sea's level (the sea, a lake, a river); smaller is a pond, at its own. */
const POND_MOST = 8000;
/** A pond whose level would come within this (m) of the sea's is at the sea's (one sheet). */
const POND_OVER = 0.2;
/** The deepest water (m), and its bed going down from the shore as the renderer's sheet's does (`nature.ts` `ground`). */
const DEPTH_MOST = 4;
const bedBelow = (e: number) => Math.min(DEPTH_MOST, e * 0.22 + e * e * 0.004);

/** What of a world the seeding and the grading read: its size, plots and meta (roads, paths, backdrop, look), as a menu's preview or a town file has them. */
export type FollowWorld = Pick<WorldStructure, 'width' | 'depth' | 'plots' | 'meta'> & SurfaceWorld;

/**
 * `draft` with its land (`open.land`) whose town follows its slopes: a place's new world seeded
 * (`seedFollow`) and graded (`gradeLand`); a world laid again on a land the player shaped
 * (`open.land`, seeded `'follow'`) graded again. Anything else (a classic world, a draft from older
 * code, a land drawn flat under its town) as it was. `world`: the draft's preview (`previewWorld`),
 * for its lots, their paths and its roads. `seeded`: given the land before it is graded (a copy).
 */
export function withLand(draft: NeighbourhoodDraft, world: FollowWorld, seeded?: (land: Land) => void): NeighbourhoodDraft {
  const open = draft.open;
  if (!open) return draft;
  let land: Land | null;
  if (open.land) land = open.land.seeded === 'follow' ? { ...open.land, h: open.land.h.slice() } : null;
  else land = seedFollow(draft, world);
  if (!land) return draft;
  seeded?.({ ...land, h: land.h.slice(), wl: land.wl.slice(), cover: land.cover.slice() });
  gradeLand(land, world);
  return { ...draft, open: { ...open, land } };
}

/** The roads, lots and the land round them graded into `land` (its heights changed in place; water left as it is): the surface it was graded to. */
export function gradeLand(land: Land, world: SurfaceWorld): BuiltSurface {
  const surface = builtSurfaceOf(world, land);
  land.h.set(surface.graded());
  return surface;
}

/**
 * A place's land before its town is graded into it, from its draft (null: it has no
 * `open.heights`, a classic world or an older draft). Every `LAND_CELL` m:
 * - the generator's heights between its cells' middles, and fading in from what is built (as the
 *   renderer's hills fade in, over the look's `rise`), the finer hills, some roughness, and among
 *   mountains their ridges;
 * - under each town and village, and each lot out on its own, gentler ground (`gentler`);
 * - its water from the water zones: what reaches the world's edge or is bigger than `POND_MOST`
 *   at the sea's level (`SEA_CM`), each pond at its own, a little below the lowest land round it;
 *   the land beside water no higher than over it by the world's slope, the bed going down from the
 *   shore as the renderer's does, and in a world with water the dry land over the sea's level;
 * - its cover from the woods, meadow and fields zones.
 */
export function seedFollow(draft: NeighbourhoodDraft, world: FollowWorld): Land | null {
  const open = draft.open;
  const heights = open?.heights;
  if (!open || !heights) return null;
  const land = blankLand(draft.width, draft.depth, 'follow');
  const { cols, rows, h: cm, wl } = land;
  const look = { ...DEFAULT_LANDSCAPE, ...draft.world?.landscape };
  const seed = draft.seed ?? 1;
  const bank = look.slope ?? 0.5;
  const dist = new Protection(cols, rows, builtOf(world)).dist;
  const h = new Float32Array(cols * rows);
  for (let j = 0; j < rows; j++) {
    for (let i = 0; i < cols; i++) {
      const [x, z] = [i * LAND_CELL, j * LAND_CELL];
      const k = j * cols + i;
      const base = between(heights, x, z);
      const rise = smoothstep(APRON, APRON + look.rise, dist[k]);
      let detail = 0;
      if (rise > 0) {
        detail = (fbm(x * 0.018, z * 0.018, seed) - 0.5) * look.hills + base * 0.3 * (fbm(x * 0.012, z * 0.012, seed + 517, 3) - 0.5);
        if (look.mountains && base > 15) detail += base * 0.35 * (ridged(x * 0.0075, z * 0.0075, seed + 211) - 0.5) * smoothstep(15, 25, base);
      }
      h[k] = base + rise * detail;
    }
  }
  gentler(h, cols, rows, dist, world, open.areas, draft.roads, draft.slots.map((s) => s.area ?? 0), look.rise);

  // ---- the water ----
  const wet = scan(
    open.zones.filter((z) => z.kind === 'water').map((z) => z.poly),
    cols,
    rows,
    LAND_CELL,
    0,
  );
  const sea = SEA_CM / 100;
  if (wet.includes(1)) {
    for (let k = 0; k < h.length; k++) if (!wet[k]) h[k] = Math.max(h[k], sea + DRY_OVER);
    // Each piece of water (side by side) at its level.
    const piece = new Int32Array(cols * rows).fill(-1);
    for (let start = 0; start < piece.length; start++) {
      if (!wet[start] || piece[start] >= 0) continue;
      const members = [start];
      piece[start] = start;
      let [edge, rim] = [false, Infinity];
      for (let n = 0; n < members.length; n++) {
        const k = members[n];
        const [i, j] = [k % cols, Math.floor(k / cols)];
        if (i === 0 || j === 0 || i === cols - 1 || j === rows - 1) edge = true;
        for (const [ni, nj] of [[i - 1, j], [i + 1, j], [i, j - 1], [i, j + 1]] as const) {
          if (ni < 0 || nj < 0 || ni >= cols || nj >= rows) continue;
          const q = nj * cols + ni;
          if (!wet[q]) rim = Math.min(rim, h[q]);
          else if (piece[q] < 0) {
            piece[q] = start;
            members.push(q);
          }
        }
      }
      const pond = !edge && members.length * LAND_CELL * LAND_CELL <= POND_MOST && rim - POND_BELOW > sea + POND_OVER;
      const level = pond ? Math.round((rim - POND_BELOW) * 100) : SEA_CM;
      for (const k of members) wl[k] = level;
    }
    // How far each wet sample is from dry land, and the highest the dry land may stand by the water.
    const into = new Float32Array(cols * rows);
    const top = new Float32Array(cols * rows);
    for (let k = 0; k < h.length; k++) {
      into[k] = wet[k] ? Infinity : 0;
      top[k] = wet[k] ? wl[k] / 100 + SHORE_OVER : Infinity;
    }
    chamfer(cols, rows, (k, n, d) => {
      if (into[n] + d < into[k]) into[k] = into[n] + d;
      if (top[n] + bank * d < top[k]) top[k] = top[n] + bank * d;
    });
    for (let k = 0; k < h.length; k++) h[k] = wet[k] ? wl[k] / 100 - bedBelow(into[k]) : Math.min(h[k], Math.max(top[k], sea + DRY_OVER));
  }
  for (let k = 0; k < h.length; k++) cm[k] = Math.max(-32767, Math.min(32767, Math.round(h[k] * 100)));

  // ---- the cover ----
  const [cw, cd] = [coverSpan(cols), coverSpan(rows)];
  for (const kind of ['fields', 'meadow', 'woods'] as const) {
    const mask = scan(
      open.zones.filter((z: WorldZone) => z.kind === kind).map((z) => z.poly),
      cw,
      cd,
      COVER_CELL,
      COVER_CELL / 2,
    );
    for (let c = 0; c < mask.length; c++) if (mask[c]) land.cover[c] = COVER[kind];
  }
  return land;
}

/**
 * The land (`h`, m per sample) eased under what is built towards gentle ground: each area's box (a
 * town or a village, with its own streets and lanes where they reach out past it), and each lot or backdrop building out of every box, gets a plane fitted to
 * the land under what is built in it (least squares, tilted no more than `GENTLE_TILT`), the land
 * on it that plane, and the land round it eased back to its own height over `reach` m (the look's
 * `rise`). Where two come near each other, their planes are blended by how near each is (in its
 * own box, almost wholly its own; near a town or a village, its plane before a lone lot's). The hills
 * and mountains rise round the towns, not under them. `dist`: each sample's distance from what is
 * built (`Protection.dist`: 0 on it).
 */
function gentler(h: Float32Array, cols: number, rows: number, dist: Float32Array, world: FollowWorld, areas: readonly { x: number; z: number; w: number; d: number }[], roads: readonly RoadPlan[], areaOf: readonly number[], reach: number): void {
  const c = LAND_CELL;
  type Box = { x0: number; z0: number; x1: number; z1: number };
  const grow = (box: Box, x0: number, z0: number, x1: number, z1: number) => Object.assign(box, { x0: Math.min(box.x0, x0), z0: Math.min(box.z0, z0), x1: Math.max(box.x1, x1), z1: Math.max(box.z1, z1) });
  // (An area's box, its own streets and lanes, which may reach out past it, and its own lots by them.)
  const boxes: Box[] = areas.map((a, n) => {
    const box = { x0: a.x, z0: a.z, x1: a.x + a.w, z1: a.z + a.d };
    for (const r of roads) {
      if ((r.area ?? -1) !== n || r.kind === 'country' || r.kind === 'trail') continue;
      for (const [x, z] of r.points) grow(box, x, z, x, z);
    }
    const near = { ...box };
    world.plots.forEach((p, i) => {
      if (areaOf[i] === n && p.x < near.x1 + AREA_LOTS && p.x + p.w > near.x0 - AREA_LOTS && p.z < near.z1 + AREA_LOTS && p.z + p.d > near.z0 - AREA_LOTS) grow(box, p.x, p.z, p.x + p.w, p.z + p.d);
    });
    return box;
  });
  const within = (b: Box, x: number, z: number) => x >= b.x0 && x <= b.x1 && z >= b.z0 && z <= b.z1;
  const alone: Box[] = [];
  const footprints = [
    ...world.plots.map((p) => ({ x0: p.x, z0: p.z, x1: p.x + p.w, z1: p.z + p.d })),
    ...(world.meta?.backdrop ?? []).map((b) => {
      const xs = b.poly.map((q) => q[0]);
      const zs = b.poly.map((q) => q[1]);
      return { x0: Math.min(...xs), z0: Math.min(...zs), x1: Math.max(...xs), z1: Math.max(...zs) };
    }),
  ];
  for (const f of footprints) if (!boxes.some((b) => within(b, (f.x0 + f.x1) / 2, (f.z0 + f.z1) / 2))) alone.push(f);
  // Each one's plane: through the land under what is built in it (in a box: every built sample; a lot alone: its own).
  const planes: { box: Box; a: number; bx: number; bz: number; cx: number; cz: number; town: boolean }[] = [];
  for (const [n, box] of [...boxes, ...alone].entries()) {
    const own = n >= boxes.length;
    const [i0, i1] = [Math.max(0, Math.ceil(box.x0 / c)), Math.min(cols - 1, Math.floor(box.x1 / c))];
    const [j0, j1] = [Math.max(0, Math.ceil(box.z0 / c)), Math.min(rows - 1, Math.floor(box.z1 / c))];
    const pts: [number, number, number][] = [];
    for (let j = j0; j <= j1; j++) for (let i = i0; i <= i1; i++) if (own || dist[j * cols + i] === 0) pts.push([i * c, j * c, h[j * cols + i]]);
    if (!pts.length) continue;
    const cx = pts.reduce((t, p) => t + p[0], 0) / pts.length;
    const cz = pts.reduce((t, p) => t + p[1], 0) / pts.length;
    const mean = pts.reduce((t, p) => t + p[2], 0) / pts.length;
    let [sxx, sxz, szz, sxh, szh] = [0, 0, 0, 0, 0];
    for (const [x, z, v] of pts) {
      const [dx, dz, dv] = [x - cx, z - cz, v - mean];
      sxx += dx * dx;
      sxz += dx * dz;
      szz += dz * dz;
      sxh += dx * dv;
      szh += dz * dv;
    }
    const det = sxx * szz - sxz * sxz;
    let [bx, bz] = Math.abs(det) > 1e-9 ? [(sxh * szz - szh * sxz) / det, (szh * sxx - sxh * sxz) / det] : [sxx > 0 ? sxh / sxx : 0, szz > 0 ? szh / szz : 0];
    const tilt = Math.hypot(bx, bz);
    if (tilt > GENTLE_TILT) [bx, bz] = [(bx * GENTLE_TILT) / tilt, (bz * GENTLE_TILT) / tilt];
    // (Tilted less, the plane through the land's mean again.)
    planes.push({ box, a: mean, bx, bz, cx, cz, town: !own });
  }
  // Each sample eased towards the planes near it: the nearest's weight, their heights weighted by each one's.
  const sum = new Float64Array(cols * rows);
  const weight = new Float64Array(cols * rows);
  const most = new Float32Array(cols * rows);
  for (const p of planes) {
    const [i0, i1] = [Math.max(0, Math.ceil((p.box.x0 - reach) / c)), Math.min(cols - 1, Math.floor((p.box.x1 + reach) / c))];
    const [j0, j1] = [Math.max(0, Math.ceil((p.box.z0 - reach) / c)), Math.min(rows - 1, Math.floor((p.box.z1 + reach) / c))];
    for (let j = j0; j <= j1; j++) {
      for (let i = i0; i <= i1; i++) {
        const [x, z] = [i * c, j * c];
        const off = Math.hypot(Math.max(p.box.x0 - x, 0, x - p.box.x1), Math.max(p.box.z0 - z, 0, z - p.box.z1));
        const w = 1 - smoothstep(0, reach, off);
        if (w <= 0) continue;
        const k = j * cols + i;
        // (Each plane's say in the blend falls off far faster than its pull: in its own box, its own; a town's or a
        // village's over a lone lot's near it.)
        const say = w ** 8 * (p.town ? TOWN_SAY : 1);
        sum[k] += say * (p.a + p.bx * (x - p.cx) + p.bz * (z - p.cz));
        weight[k] += say;
        if (w > most[k]) most[k] = w;
      }
    }
  }
  for (let k = 0; k < h.length; k++) if (weight[k] > 0) h[k] += most[k] * (sum[k] / weight[k] - h[k]);
}

/** The generator's heights (m) at (`x`, `z`), between its cells' middles (the edge's past the edge). */
function between(g: NonNullable<NonNullable<NeighbourhoodDraft['open']>['heights']>, x: number, z: number): number {
  const gx = Math.min(g.cols - 1, Math.max(0, x / g.cell - 0.5));
  const gz = Math.min(g.rows - 1, Math.max(0, z / g.cell - 0.5));
  const [x0, z0] = [Math.floor(gx), Math.floor(gz)];
  const [x1, z1] = [Math.min(g.cols - 1, x0 + 1), Math.min(g.rows - 1, z0 + 1)];
  const [u, v] = [gx - x0, gz - z0];
  const at = (c: number, r: number) => g.h[r * g.cols + c];
  return (at(x0, z0) * (1 - u) + at(x1, z0) * u) * (1 - v) + (at(x0, z1) * (1 - u) + at(x1, z1) * u) * v;
}

/** The points of a `gw` × `gd` grid (point (i, j) at (i × `step` + `offset`, j × `step` + `offset`) m) inside `polys` (even-odd over them all, as `landMasks.ts` `rasterize`). */
function scan(polys: readonly Pt[][], gw: number, gd: number, step: number, offset: number): Uint8Array {
  const mask = new Uint8Array(gw * gd);
  for (const poly of polys) {
    let [lo, hi] = [Infinity, -Infinity];
    for (const p of poly) [lo, hi] = [Math.min(lo, p[1]), Math.max(hi, p[1])];
    for (let j = Math.max(0, Math.ceil((lo - offset) / step)); j < Math.min(gd, Math.floor((hi - offset) / step) + 1); j++) {
      const zc = j * step + offset;
      const xs: number[] = [];
      for (let a = 0, b = poly.length - 1; a < poly.length; b = a++) {
        const [[xa, za], [xb, zb]] = [poly[a], poly[b]];
        if (za > zc !== zb > zc) xs.push(((xb - xa) * (zc - za)) / (zb - za) + xa);
      }
      xs.sort((p, q) => p - q);
      for (let k = 0; k + 1 < xs.length; k += 2) {
        for (let i = Math.max(0, Math.ceil((xs[k] - offset) / step)); i < Math.min(gw, Math.ceil((xs[k + 1] - offset) / step)); i++) mask[j * gw + i] ^= 1;
      }
    }
  }
  return mask;
}

/** Two chamfer passes over a `cols` × `rows` grid of `LAND_CELL` m samples: `relax(k, n, d)` for each sample `k` and the neighbour `n` before it in each pass, `d` m apart. */
function chamfer(cols: number, rows: number, relax: (k: number, n: number, d: number) => void): void {
  const [side, diag] = [LAND_CELL, LAND_CELL * Math.SQRT2];
  for (let j = 0; j < rows; j++) {
    for (let i = 0; i < cols; i++) {
      const k = j * cols + i;
      if (i > 0) relax(k, k - 1, side);
      if (j > 0) {
        relax(k, k - cols, side);
        if (i > 0) relax(k, k - cols - 1, diag);
        if (i < cols - 1) relax(k, k - cols + 1, diag);
      }
    }
  }
  for (let j = rows - 1; j >= 0; j--) {
    for (let i = cols - 1; i >= 0; i--) {
      const k = j * cols + i;
      if (i < cols - 1) relax(k, k + 1, side);
      if (j < rows - 1) {
        relax(k, k + cols, side);
        if (i < cols - 1) relax(k, k + cols + 1, diag);
        if (i > 0) relax(k, k + cols - 1, diag);
      }
    }
  }
}
