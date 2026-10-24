/**
 * Picking, as the Babylon renderer did it (`pick`, `pickPlane`, `pickVertical`, `pickOnWall`,
 * `objectBox`): analytic, on the CPU, no GPU picking. A ray from the game camera through the
 * pointer (`GameCamera.ray`, in game space through `space.ts`) meets the floor of the storey in
 * view, the placed objects' boxes (`LotContents.placed`, the ones prepared with the world: what's
 * drawn of each, never past its tiles; things on a wall that went with it aren't picked) and the
 * items' boxes (a plate on a table is picked before the table). Residents are picked by the game
 * on screen (`simHead` and `project`), walls not at all, as with Babylon.
 *
 * The world is drawn in render space: an upper storey's lot rows lie over the ground's (shifted
 * back by `rowOf`) and lifted by a storey's height, so what is answered in lot rows is shifted
 * forth again. The last ray is kept for the drags that follow a pick (`pickPlane` and the rest).
 * No allocations but the result (the game keeps the points it is given).
 */

import type { GameCamera } from './camera';
import type { LotContents } from './lotContents';
import { WALL_HEIGHT } from '../shared/house';
import type { PickResult } from '../types';

/** How picking reads the storeys (the renderer's). */
export interface PickView {
  /** The storey in view. */
  storey(): number;
  /** The storey a lot row is on. */
  storeyOf(z: number): number;
  /** The first lot row of storey `k`. */
  rowOf(k: number): number;
  /** The canvas's size in CSS pixels. */
  readonly size: { width: number; height: number };
}

/** A plate counts as on what it stands on when it is no farther than this along the ray (m). */
const ITEM_REACH = 0.3;

export class Picker {
  /** The last pick's ray (game space, render rows; `dir` of unit length). */
  private readonly origin = { x: 0, y: 0, z: 0 };
  private readonly dir = { x: 0, y: -1, z: 0 };
  private readonly forward = { x: 0, y: 0, z: 0 };

  constructor(
    private readonly camera: GameCamera,
    private readonly lot: LotContents,
    private readonly view: PickView,
  ) {}

  pick(x: number, y: number): PickResult {
    const { size } = this.view;
    this.camera.ray(x, y, size.width, size.height, this.origin, this.dir);
    const o = this.origin;
    const d = this.dir;
    const storey = this.view.storey();
    let ground: PickResult['ground'] = null;
    // The floor of the storey in view, answered in its rows of the lot.
    const floor = storey * WALL_HEIGHT;
    if (d.y < -1e-6 && o.y > floor) {
      const t = (floor - o.y) / d.y;
      ground = { x: o.x + d.x * t, z: o.z + d.z * t + this.view.rowOf(storey) };
    }
    let objectId: number | null = null;
    let nearest = Infinity;
    for (const p of this.lot.placed) {
      if (this.lot.wallItemHidden(p.id)) continue;
      const t = rayBox(o, d, p.minX, p.y0 + p.bottom, p.minZ, p.maxX, p.y0 + p.height, p.maxZ);
      if (t < nearest) {
        nearest = t;
        objectId = p.id;
      }
    }
    // A plate on a table is picked before the table.
    let itemId: number | null = null;
    let nearestItem = Infinity;
    for (const p of this.lot.items.placed) {
      const t = rayBox(o, d, p.minX, p.y0, p.minZ, p.maxX, p.y0 + p.height, p.maxZ);
      if (t < nearestItem && t <= nearest + ITEM_REACH) {
        nearestItem = t;
        itemId = p.id;
      }
    }
    return { ground, objectId, itemId };
  }

  /** Where the last ray meets the level `height` m above the floor in view (lot rows), or null. */
  pickPlane(height: number): { x: number; z: number } | null {
    const o = this.origin;
    const d = this.dir;
    const storey = this.view.storey();
    const y = storey * WALL_HEIGHT + height;
    if (d.y > -1e-6 || o.y <= y) return null;
    const t = (y - o.y) / d.y;
    return { x: o.x + d.x * t, z: o.z + d.z * t + this.view.rowOf(storey) };
  }

  /** The last ray's height over lot point (`x`, `z`) above its storey's floor, on the upright plane through it square to the view. */
  pickVertical(x: number, z: number): number | null {
    const o = this.origin;
    const d = this.dir;
    const k = this.view.storeyOf(z);
    const pz = z - this.view.rowOf(k);
    // The upright plane through the point, square to the camera's view across the floor.
    const f = this.camera.forwardInGame(this.forward);
    const len = Math.hypot(f.x, f.z);
    if (len < 1e-6) return null;
    const nx = f.x / len;
    const nz = f.z / len;
    const along = d.x * nx + d.z * nz;
    if (Math.abs(along) < 1e-6) return null;
    const t = ((x - o.x) * nx + (pz - o.z) * nz) / along;
    return o.y + d.y * t - k * WALL_HEIGHT;
  }

  /** Where the last ray meets wall line `line` (`h`: lot z, `v`: x; in `storeyRow`'s storey's rows): along it and above that storey's floor. */
  pickOnWall(axis: 'h' | 'v', line: number, storeyRow: number): { along: number; y: number } | null {
    const o = this.origin;
    const d = this.dir;
    const k = this.view.storeyOf(storeyRow);
    const D = this.view.rowOf(k);
    // The wall's plane in render space (rows of upper storeys are drawn at the ground's).
    const plane = axis === 'h' ? line - D : line;
    const across = axis === 'h' ? d.z : d.x;
    if (Math.abs(across) < 1e-6) return null;
    const t = (plane - (axis === 'h' ? o.z : o.x)) / across;
    if (t <= 0) return null;
    return { along: axis === 'h' ? o.x + d.x * t : o.z + d.z * t + D, y: o.y + d.y * t - k * WALL_HEIGHT };
  }

  /** The object `id`'s box as drawn: its height and how high its bottom is (things on a wall or the ceiling). */
  objectBox(id: number): { height: number; bottom: number } | null {
    for (const p of this.lot.placed) if (p.id === id) return { height: p.height, bottom: p.bottom };
    return null;
  }
}

type V3 = { readonly x: number; readonly y: number; readonly z: number };

/** Ray against an axis-aligned box (slab test): the distance along the ray to where it enters (0 inside), or Infinity. */
function rayBox(o: V3, d: V3, x0: number, y0: number, z0: number, x1: number, y1: number, z1: number): number {
  let tMin = 0;
  let tMax = Infinity;
  for (let axis = 0; axis < 3; axis++) {
    const origin = axis === 0 ? o.x : axis === 1 ? o.y : o.z;
    const dir = axis === 0 ? d.x : axis === 1 ? d.y : d.z;
    const lo = axis === 0 ? x0 : axis === 1 ? y0 : z0;
    const hi = axis === 0 ? x1 : axis === 1 ? y1 : z1;
    if (Math.abs(dir) < 1e-9) {
      if (origin < lo || origin > hi) return Infinity;
      continue;
    }
    let t0 = (lo - origin) / dir;
    let t1 = (hi - origin) / dir;
    if (t0 > t1) {
      const t = t0;
      t0 = t1;
      t1 = t;
    }
    tMin = Math.max(tMin, t0);
    tMax = Math.min(tMax, t1);
    if (tMin > tMax) return Infinity;
  }
  return tMin;
}
