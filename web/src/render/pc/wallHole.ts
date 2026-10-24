/**
 * The wall hole (Babylon's `holeFor` and `swapBuilt`): while Build mode's preview of walls, doors
 * and windows stands in for a stretch of wall that is already built, that stretch of the built
 * walls isn't drawn, so the preview shows in its place. The cutaway shader drops the fragments of
 * every `cutaway` material near the segment (`WallCutState.hole`, `shaders/cutaway.ts`), in render
 * space: game space with an upper storey's rows over the ground's.
 */

import type { WallCutState } from './materials';
import { WALL_HALF, WALL_HEIGHT } from '../shared/walls';
import type { EdgePreview } from '../types';

/** A stretch of wall to hide: the segment (x0, z0)–(x1, z1), everything within `width` of it, between heights `y0` and `y1`. */
export type WallHole = readonly [x0: number, z0: number, x1: number, z1: number, width: number, y0: number, y1: number];

/**
 * The stretch of wall `edges` cover on `storey` (whose rows start at lot row `shift`), in one line
 * (a door or window and the wall it goes into); null for none or for a bent run.
 */
export function wallHoleFor(edges: readonly EdgePreview[], storey: number, shift: number): WallHole | null {
  // In the world, this storey's rows lie over the ground's (see `setStorey`).
  const ends = edges.map((e): [number, number, number, number] => {
    const t = e.axis === 'h' || e.axis === 'v' ? WALL_HALF : WALL_HALF * Math.SQRT1_2;
    const z = e.z - shift;
    if (e.axis === 'h') return [e.x + t, z, e.x + 1 - t, z];
    if (e.axis === 'v') return [e.x, z + t, e.x, z + 1 - t];
    if (e.axis === 'dp') return [e.x + t, z + t, e.x + 1 - t, z + 1 - t];
    return [e.x + t, z + 1 - t, e.x + 1 - t, z + t];
  });
  if (!ends.length) return null;
  // One segment from the first to the farthest end along the line (none for a bent run).
  const [x0, z0, x1, z1] = ends[0];
  const [ux, uz] = [x1 - x0, z1 - z0];
  const along = (x: number, z: number) => (x - x0) * ux + (z - z0) * uz;
  const off = (x: number, z: number) => Math.abs((x - x0) * uz - (z - z0) * ux);
  if (ends.some(([a, b, c, d]) => off(a, b) > 1e-6 || off(c, d) > 1e-6)) return null;
  const points = ends.flatMap(([a, b, c, d]) => [[a, b], [c, d]]);
  const lo = points.reduce((m, p) => (along(p[0], p[1]) < along(m[0], m[1]) ? p : m));
  const hi = points.reduce((m, p) => (along(p[0], p[1]) > along(m[0], m[1]) ? p : m));
  // Within reach of trim and sills (a crossing wall loses no more than that); this storey's walls only.
  const y0 = storey * WALL_HEIGHT;
  return [lo[0], lo[1], hi[0], hi[1], WALL_HALF + 0.08, storey ? y0 + 0.005 : -1, y0 + WALL_HEIGHT + 0.005];
}

/** Hides the stretch `hole` of the built walls (null: none hidden). */
export function setWallHole(cut: WallCutState, hole: WallHole | null): void {
  if (hole) {
    const [x0, z0, x1, z1, w, y0, y1] = hole;
    cut.hole.set(x0, z0, x1, z1);
    cut.holeSpan.set(w, y0, y1, 0);
  } else cut.holeSpan.x = 0;
}
