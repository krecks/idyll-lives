/**
 * The shade on the ground round what stands on it: a house's walls, fences and hedges, the things in a garden,
 * a tree's trunk. Less of the sky reaches the ground at their foot, so it is darker there, most where they
 * touch it and fading out over a distance that goes with their height. It is what makes a thing sit on the
 * lawn instead of lying on a picture of one.
 *
 * One byte a texel over the town's ground, as fine as the lawn's mask (`GRASS.townTexels` a metre), which it
 * shares a texture with (`pc/grass.ts`: the mask's second channel), so the blades and the ground under them
 * read it with the read they already make (`pc/shaders/grass.ts`): no draw, no pass, the same on every preset.
 * Made again only round what changed (`updateGroundShade`): a big world's is millions of texels.
 * Paths, patios and floors don't read it (things there have their contact shadows, `shared/contactShadow.ts`,
 * which the blades hide on a lawn).
 *
 * Drawn only: made from the world as it stands, again whenever the lot changes (`townGrass`).
 */

import { groundDepth, type WorldStructure } from '../../core/protocol';
import { tuned } from '../tune';

/**
 * How dark and how far, each `[the share of the light taken at the foot, the reach in metres]`: an outside
 * wall's, a fence's or hedge's, a thing's (round its footprint, drawn in by `inset` m a side: few things
 * fill their tiles) and a tree's trunk (round the middle of its footprint, `trunkRadius` m). `falloff`: how
 * fast it fades with the distance (1: evenly; more: most of it close by). A thing's and a trunk's are light: at
 * [0.4, 0.6] and [0.6, 1.5], with the soil round a bed or a trunk under them, a crate and a planter stood in a
 * brown-black smudge heavier than a shadow, and a tree on a grey bare patch.
 */
export const GROUND_SHADE = tuned('shade', {
  wall: [0.65, 1.8],
  fence: [0.5, 0.9],
  thing: [0.3, 0.5],
  trunk: [0.4, 1.3],
  inset: 0.14,
  trunkRadius: 0.2,
  falloff: 1.3,
});

/** Half a wall's and a fence's thickness (m). */
const HALF_WALL = 0.1;
const HALF_FENCE = 0.06;

/** Trees among the world's things (their kinds' ids): a trunk in the middle, not a footprint. */
const TREE = /^(tree|pine|appleTree|lemonTree|garden\.(maple|cherry|birch|willow|spruce|japaneseMaple|palm|apple|lemon|oak))$/;

/** The rectangles that shade the ground (m), `STAMP` numbers each: `x0, z0, x1, z1, strength, reach, over` (see `stamp`; `over` 1 or 0). */
const STAMP = 7;

/** A rectangle of texels, `[i0, j0, i1, j1]`, its last column and row included. */
export type TexelRect = [number, number, number, number];

/** The stamps each shade map was last made from, in order (`groundShade`, `updateGroundShade`). */
const madeFrom = new WeakMap<Uint8Array, { n: number; W: number; stamps: Float64Array }>();

/** More changed stamps than this and the whole map is made again (a house built at once, the first time). */
const MOST_CHANGED = 64;

/**
 * The ground's shade for `world` now: `world.width * n` × its ground's depth `* n` bytes, rows along x, 0 none
 * and 255 all the light taken. Into `into` when it is that size.
 */
export function groundShade(world: WorldStructure, n: number, into?: Uint8Array): Uint8Array {
  const [W, D] = [world.width * n, groundDepth(world) * n];
  const out = into && into.length === W * D ? into.fill(0) : new Uint8Array(W * D);
  const stamps = shadeStamps(world);
  for (let k = 0; k < stamps.length; k += STAMP) stamp(out, W, D, n, stamps, k, null);
  madeFrom.set(out, { n, W, stamps });
  return out;
}

/**
 * `out` (a map `groundShade` made, or this updated) made the ground's shade for `world` now, again only round
 * what changed since (a thing bought, a fence built): the same bytes as `groundShade` makes (as long as the
 * things that stayed kept their order, as the simulation's list does). The texels made again (null: all of
 * them, or a new map: when `out` isn't one for this world's size).
 */
export function updateGroundShade(world: WorldStructure, n: number, out: Uint8Array): { shade: Uint8Array; changed: TexelRect[] | null } {
  const [W, D] = [world.width * n, groundDepth(world) * n];
  const before = madeFrom.get(out);
  if (!before || before.n !== n || before.W !== W || out.length !== W * D) return { shade: groundShade(world, n, out), changed: null };
  const stamps = shadeStamps(world);
  const old = before.stamps;
  // The stamps gone and the new ones: past those the two lists start and end with alike, by what they are
  // (the others' order is the same).
  const same = (k: number, j: number): boolean => {
    for (let c = 0; c < STAMP; c++) if (stamps[k + c] !== old[j + c]) return false;
    return true;
  };
  let start = 0;
  while (start < stamps.length && start < old.length && same(start, start)) start += STAMP;
  let end = 0;
  while (end < stamps.length - start && end < old.length - start && same(stamps.length - end - STAMP, old.length - end - STAMP)) end += STAMP;
  if (stamps.length - start - end > MOST_CHANGED * 16 * STAMP || old.length - start - end > MOST_CHANGED * 16 * STAMP) return { shade: groundShade(world, n, out), changed: null };
  const key = (list: Float64Array, k: number) => list.subarray(k, k + STAMP).join();
  const count = new Map<string, number>();
  for (let k = start; k < old.length - end; k += STAMP) {
    const s = key(old, k);
    count.set(s, (count.get(s) ?? 0) + 1);
  }
  const rects: TexelRect[] = [];
  for (let k = start; k < stamps.length - end; k += STAMP) {
    const s = key(stamps, k);
    const c = count.get(s) ?? 0;
    if (c > 0) count.set(s, c - 1);
    else rects.push(reachOf(stamps, k, W, D, n));
  }
  for (let k = start; k < old.length - end; k += STAMP) {
    const s = key(old, k);
    const c = count.get(s) ?? 0;
    if (c > 0) {
      count.set(s, c - 1);
      rects.push(reachOf(old, k, W, D, n));
    }
  }
  if (rects.length > MOST_CHANGED) return { shade: groundShade(world, n, out), changed: null };
  // Each changed stamp's reach made again: cleared, and every stamp over it laid again in order.
  const changed = rects.filter((r) => r[0] <= r[2] && r[1] <= r[3]);
  for (const r of changed) {
    for (let j = r[1]; j <= r[3]; j++) out.fill(0, j * W + r[0], j * W + r[2] + 1);
    for (let k = 0; k < stamps.length; k += STAMP) stamp(out, W, D, n, stamps, k, r);
  }
  madeFrom.set(out, { n, W, stamps });
  return { shade: out, changed };
}

/** The texels stamp `k` of `stamps` reaches (`[i0, j0, i1, j1]`, empty where `i0 > i1` or `j0 > j1`). */
function reachOf(stamps: Float64Array, k: number, W: number, D: number, n: number): TexelRect {
  const [x0, z0, x1, z1, reach] = [stamps[k], stamps[k + 1], stamps[k + 2], stamps[k + 3], stamps[k + 5]];
  return [Math.max(0, Math.floor((x0 - reach) * n)), Math.max(0, Math.floor((z0 - reach) * n)), Math.min(W - 1, Math.ceil((x1 + reach) * n) - 1), Math.min(D - 1, Math.ceil((z1 + reach) * n) - 1)];
}

/**
 * A rectangle's shade (m) into `out`: `strength` on it, fading out over `reach` round it. `over`: laid over what
 * is there (two things side by side shade more than one); else the darker of the two (a wall's segments, end to
 * end, are one wall). Only within `clip` if given.
 */
function stamp(out: Uint8Array, W: number, D: number, n: number, stamps: Float64Array, k: number, clip: TexelRect | null): void {
  const [x0, z0, x1, z1, strength, reach, over] = [stamps[k], stamps[k + 1], stamps[k + 2], stamps[k + 3], stamps[k + 4], stamps[k + 5], stamps[k + 6]];
  if (strength <= 0 || reach <= 0) return;
  // (Clipped: most stamps are nowhere near `clip`.)
  if (clip && (Math.ceil((x1 + reach) * n) - 1 < clip[0] || Math.floor((x0 - reach) * n) > clip[2] || Math.ceil((z1 + reach) * n) - 1 < clip[1] || Math.floor((z0 - reach) * n) > clip[3])) return;
  let [i0, j0, i1, j1] = reachOf(stamps, k, W, D, n);
  if (clip) {
    i0 = Math.max(i0, clip[0]);
    j0 = Math.max(j0, clip[1]);
    i1 = Math.min(i1, clip[2]);
    j1 = Math.min(j1, clip[3]);
  }
  const falloff = GROUND_SHADE.falloff;
  for (let j = j0; j <= j1; j++) {
    const z = (j + 0.5) / n;
    const dz = Math.max(z0 - z, 0, z - z1);
    for (let i = i0; i <= i1; i++) {
      const x = (i + 0.5) / n;
      const dx = Math.max(x0 - x, 0, x - x1);
      const d = Math.sqrt(dx * dx + dz * dz);
      if (d >= reach) continue;
      const t = 1 - d / reach;
      const v = strength * Math.pow(t, falloff);
      const o = j * W + i;
      const was = out[o] / 255;
      out[o] = Math.round(255 * (over ? 1 - (1 - was) * (1 - v) : Math.max(was, v)));
    }
  }
}

/** What shades the ground in `world`, in the order it is laid: outside walls, diagonals, fences, things and trunks. */
function shadeStamps(world: WorldStructure): Float64Array {
  const S = GROUND_SHADE;
  const [tw, td] = [world.width, groundDepth(world)];
  const list: number[] = [];
  const out = { push: (s: readonly number[]) => list.push(...s) };
  const rooms = world.rooms;
  const outdoors = (x: number, z: number) => x < 0 || z < 0 || x >= tw || z >= td || rooms[z * world.width + x] === 0;
  // Outside walls (one with a room on both its sides shades no ground).
  const [ws, wr] = S.wall;
  for (const e of world.walls) {
    if (e.axis === 'h') {
      if (e.z > td || !(outdoors(e.x, e.z - 1) || outdoors(e.x, e.z))) continue;
      out.push([e.x, e.z - HALF_WALL, e.x + 1, e.z + HALF_WALL, ws, wr, 0]);
    } else {
      if (e.z >= td || !(outdoors(e.x - 1, e.z) || outdoors(e.x, e.z))) continue;
      out.push([e.x - HALF_WALL, e.z, e.x + HALF_WALL, e.z + 1, ws, wr, 0]);
    }
  }
  for (const d of world.diagonals ?? []) {
    // (A diagonal wall: its tile's middle, as far as its corners.)
    if (d.z < td && (d.rooms[0] === 0 || d.rooms[1] === 0)) out.push([d.x + 0.3, d.z + 0.3, d.x + 0.7, d.z + 0.7, ws, wr, 0]);
  }
  const [fs, fr] = S.fence;
  for (const e of world.fences ?? []) {
    if (e.kind === 'gate') continue;
    if (e.axis === 'h') {
      if (e.z <= td) out.push([e.x, e.z - HALF_FENCE, e.x + 1, e.z + HALF_FENCE, fs, fr, 0]);
    } else if (e.z < td) out.push([e.x - HALF_FENCE, e.z, e.x + HALF_FENCE, e.z + 1, fs, fr, 0]);
  }
  // Things outdoors on the ground, and trunks.
  const [os, or] = S.thing;
  const [ts, tr] = S.trunk;
  for (const o of world.objects) {
    if (o.z >= td || o.lift !== undefined || !outdoors(o.x, o.z)) continue;
    const cx = o.x + o.w / 2 + (o.offset?.[0] ?? 0) / 100;
    const cz = o.z + o.d / 2 + (o.offset?.[1] ?? 0) / 100;
    if (TREE.test(o.def)) {
      out.push([cx - S.trunkRadius, cz - S.trunkRadius, cx + S.trunkRadius, cz + S.trunkRadius, ts, tr, 1]);
      continue;
    }
    // (A resized thing's own size, turned as its footprint is.)
    const turned = o.rot % 2 === 1;
    const [sw, sd] = o.size ? [o.size[turned ? 1 : 0] / 100, o.size[turned ? 0 : 1] / 100] : [o.w, o.d];
    const hw = Math.max(0.1, Math.min(sw, o.w) / 2 - S.inset);
    const hd = Math.max(0.1, Math.min(sd, o.d) / 2 - S.inset);
    out.push([cx - hw, cz - hd, cx + hw, cz + hd, os, or, 1]);
  }
  return Float64Array.from(list);
}
