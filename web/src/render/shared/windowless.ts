/**
 * Rooms without windows, kept out of daylight on the storey in view as a basement is (the
 * materials' mask: `DigState.darken` in either renderer's `materials.ts`). Engine-neutral: which
 * rooms have no window (as sim-core `lights::windowless`), and their tiles in the view.
 */

import type { WorldStructure } from '../../core/protocol';
import type { ViewRect } from '../types';
import { WALL_HEIGHT } from './walls';

/**
 * The bits of a tile in the windowless rooms' mask: dark whole, or the halves of a tile split by a
 * diagonal wall that are, and which way the wall runs (as the materials' `WINDOWLESS_BITS`).
 */
const B = { tile: 1, half0: 2, half1: 4, dn: 8 } as const;

/** Tiles kept out of daylight: `w` by `d` of them from (x0, z0), heights y0 to y1, a byte per tile (the materials' `DarkTiles`). */
export interface DarkTiles {
  x0: number;
  z0: number;
  w: number;
  d: number;
  y0: number;
  y1: number;
  tiles: Uint8Array;
}

/** The rooms without windows of a town, and the arrays they were found from (`windowlessRooms`). */
export interface Windowless {
  rooms: ReadonlySet<number>;
  of: readonly unknown[];
}

/**
 * The rooms of `world` without windows: a window lets daylight into the rooms on both its sides
 * (sim-core `rooms.rs`). Which rooms have none only changes with the walls (new `rooms`,
 * `openings` and `diagonals` arrays): `was` is kept while they're the same.
 */
export function windowlessRooms(world: WorldStructure, was: Windowless): Windowless {
  const of = [world.rooms, world.openings, world.diagonals];
  if (!of.some((a, i) => a !== was.of[i])) return was;
  const W = world.width;
  const room = (x: number, z: number) => (x >= 0 && z >= 0 && x < W && z < world.depth ? world.rooms[z * W + x] : 0);
  const lit = new Set<number>();
  for (const o of world.openings ?? []) {
    if (o.kind !== 'window') continue;
    lit.add(room(o.x, o.z));
    lit.add(o.axis === 'h' ? room(o.x, o.z - 1) : room(o.x - 1, o.z));
  }
  for (const d of world.diagonals ?? []) if (d.kind === 'window') for (const r of d.rooms) lit.add(r);
  const rooms = new Set<number>();
  for (const r of world.rooms) if (r !== 0 && !lit.has(r)) rooms.add(r);
  for (const d of world.diagonals ?? []) for (const r of d.rooms) if (r !== 0 && !lit.has(r)) rooms.add(r);
  return { rooms, of };
}

/**
 * The windowless rooms (`dark`) of storey `storey` (its first lot row `row`) in `view`: the rooms
 * (`dimmed`), and every one of their tiles in view (`dimming`, null when none), dark whole or the
 * halves of one split by a diagonal wall that are, in the box round them.
 */
export function darkTiles(world: WorldStructure, view: ViewRect | null, storey: number, row: number, dark: ReadonlySet<number>): { dimmed: Set<number>; dimming: DarkTiles | null } {
  const W = world.width;
  const dimmed = new Set<number>();
  let dimming: DarkTiles | null = null;
  if (view && storey >= 0 && dark.size) {
    const [vw, vd] = [view.w, view.d];
    // The view's tiles: dark whole, or the halves of one split by a diagonal wall that are; and
    // the box round the dark ones (x0, z0, x1, z1, in the view), all the materials are given.
    const tiles = new Uint8Array(vw * vd);
    const box = [vw, vd, 0, 0];
    const mark = (x: number, z: number, bits: number) => {
      tiles[z * vw + x] = bits;
      box.splice(0, 4, Math.min(box[0], x), Math.min(box[1], z), Math.max(box[2], x + 1), Math.max(box[3], z + 1));
    };
    const diagonals = new Map((world.diagonals ?? []).map((d) => [d.z * W + d.x, d]));
    for (let z = 0; z < vd; z++) {
      for (let x = 0; x < vw; x++) {
        const i = (z + view.z + row) * W + x + view.x;
        const d = diagonals.get(i);
        if (!d) {
          if (!dark.has(world.rooms[i])) continue;
          mark(x, z, B.tile);
          dimmed.add(world.rooms[i]);
          continue;
        }
        let bits = 0;
        d.rooms.forEach((r, h) => {
          if (!dark.has(r)) return;
          bits |= h === 0 ? B.half0 : B.half1;
          dimmed.add(r);
        });
        if (bits) mark(x, z, bits | (d.axis === 'dn' ? B.dn : 0));
      }
    }
    if (dimmed.size) {
      const [x0, z0, w, d] = [box[0], box[1], box[2] - box[0], box[3] - box[1]];
      const crop = new Uint8Array(w * d);
      for (let z = 0; z < d; z++) crop.set(tiles.subarray((z0 + z) * vw + x0, (z0 + z) * vw + x0 + w), z * w);
      // From under the floor of the storey in view to just under the next one's.
      dimming = { x0: view.x + x0, z0: view.z + z0, w, d, y0: storey * WALL_HEIGHT - 0.5, y1: (storey + 1) * WALL_HEIGHT - 0.02, tiles: crop };
    }
  }
  return { dimmed, dimming };
}
