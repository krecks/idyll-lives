/**
 * The shade on the ground round what stands on it: a house's walls, fences and hedges, the things in a garden,
 * a tree's trunk. Less of the sky reaches the ground at their foot, so it is darker there, most where they
 * touch it and fading out over a distance that goes with their height. It is what makes a thing sit on the
 * lawn instead of lying on a picture of one.
 *
 * One byte a texel over the town's ground, as fine as the lawn's mask (`GRASS.townTexels` a metre), which it
 * shares a texture with (`pc/grass.ts`: the mask's second channel), so the blades and the ground under them
 * read it with the read they already make (`pc/shaders/grass.ts`): no draw, no pass, the same on every preset.
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
 * fast it fades with the distance (1: evenly; more: most of it close by).
 */
export const GROUND_SHADE = tuned('shade', {
  wall: [0.65, 1.8],
  fence: [0.5, 0.9],
  thing: [0.65, 0.8],
  trunk: [0.6, 1.5],
  inset: 0.14,
  trunkRadius: 0.2,
  falloff: 1.3,
});

/** Half a wall's and a fence's thickness (m). */
const HALF_WALL = 0.1;
const HALF_FENCE = 0.06;

/** Trees among the world's things (their kinds' ids): a trunk in the middle, not a footprint. */
const TREE = /^(tree|pine|appleTree|lemonTree|garden\.(maple|cherry|birch|willow|spruce|japaneseMaple|palm|apple|lemon|oak))$/;

/**
 * The ground's shade for `world` now: `world.width * n` × its ground's depth `* n` bytes, rows along x, 0 none
 * and 255 all the light taken. Into `into` when it is that size.
 */
export function groundShade(world: WorldStructure, n: number, into?: Uint8Array): Uint8Array {
  const S = GROUND_SHADE;
  const [tw, td] = [world.width, groundDepth(world)];
  const [W, D] = [tw * n, td * n];
  const out = into && into.length === W * D ? into.fill(0) : new Uint8Array(W * D);
  /**
   * A rectangle's shade (m): `strength` on it, fading out over `reach` round it. `over`: laid over what is
   * there (two things side by side shade more than one); else the darker of the two (a wall's segments, end
   * to end, are one wall).
   */
  const stamp = (x0: number, z0: number, x1: number, z1: number, strength: number, reach: number, over: boolean): void => {
    if (strength <= 0 || reach <= 0) return;
    const i0 = Math.max(0, Math.floor((x0 - reach) * n));
    const i1 = Math.min(W - 1, Math.ceil((x1 + reach) * n) - 1);
    const j0 = Math.max(0, Math.floor((z0 - reach) * n));
    const j1 = Math.min(D - 1, Math.ceil((z1 + reach) * n) - 1);
    for (let j = j0; j <= j1; j++) {
      const z = (j + 0.5) / n;
      const dz = Math.max(z0 - z, 0, z - z1);
      for (let i = i0; i <= i1; i++) {
        const x = (i + 0.5) / n;
        const dx = Math.max(x0 - x, 0, x - x1);
        const d = Math.sqrt(dx * dx + dz * dz);
        if (d >= reach) continue;
        const t = 1 - d / reach;
        const v = strength * Math.pow(t, S.falloff);
        const k = j * W + i;
        const was = out[k] / 255;
        out[k] = Math.round(255 * (over ? 1 - (1 - was) * (1 - v) : Math.max(was, v)));
      }
    }
  };
  const rooms = world.rooms;
  const outdoors = (x: number, z: number) => x < 0 || z < 0 || x >= tw || z >= td || rooms[z * world.width + x] === 0;
  // Outside walls (one with a room on both its sides shades no ground).
  const [ws, wr] = S.wall;
  for (const e of world.walls) {
    if (e.axis === 'h') {
      if (e.z > td || !(outdoors(e.x, e.z - 1) || outdoors(e.x, e.z))) continue;
      stamp(e.x, e.z - HALF_WALL, e.x + 1, e.z + HALF_WALL, ws, wr, false);
    } else {
      if (e.z >= td || !(outdoors(e.x - 1, e.z) || outdoors(e.x, e.z))) continue;
      stamp(e.x - HALF_WALL, e.z, e.x + HALF_WALL, e.z + 1, ws, wr, false);
    }
  }
  for (const d of world.diagonals ?? []) {
    // (A diagonal wall: its tile's middle, as far as its corners.)
    if (d.z < td && (d.rooms[0] === 0 || d.rooms[1] === 0)) stamp(d.x + 0.3, d.z + 0.3, d.x + 0.7, d.z + 0.7, ws, wr, false);
  }
  const [fs, fr] = S.fence;
  for (const e of world.fences ?? []) {
    if (e.kind === 'gate') continue;
    if (e.axis === 'h') {
      if (e.z <= td) stamp(e.x, e.z - HALF_FENCE, e.x + 1, e.z + HALF_FENCE, fs, fr, false);
    } else if (e.z < td) stamp(e.x - HALF_FENCE, e.z, e.x + HALF_FENCE, e.z + 1, fs, fr, false);
  }
  // Things outdoors on the ground, and trunks.
  const [os, or] = S.thing;
  const [ts, tr] = S.trunk;
  for (const o of world.objects) {
    if (o.z >= td || o.lift !== undefined || !outdoors(o.x, o.z)) continue;
    const cx = o.x + o.w / 2 + (o.offset?.[0] ?? 0) / 100;
    const cz = o.z + o.d / 2 + (o.offset?.[1] ?? 0) / 100;
    if (TREE.test(o.def)) {
      stamp(cx - S.trunkRadius, cz - S.trunkRadius, cx + S.trunkRadius, cz + S.trunkRadius, ts, tr, true);
      continue;
    }
    // (A resized thing's own size, turned as its footprint is.)
    const turned = o.rot % 2 === 1;
    const [sw, sd] = o.size ? [o.size[turned ? 1 : 0] / 100, o.size[turned ? 0 : 1] / 100] : [o.w, o.d];
    const hw = Math.max(0.1, Math.min(sw, o.w) / 2 - S.inset);
    const hd = Math.max(0.1, Math.min(sd, o.d) / 2 - S.inset);
    stamp(cx - hw, cz - hd, cx + hw, cz + hd, os, or, true);
  }
  return out;
}
