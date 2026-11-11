/**
 * The miniature focus: the picture's far and near edges out of focus, as in a photograph of a model, where
 * only a band of the table is sharp. The game looks down on a house as one looks at a model, and with the
 * top and the bottom of the picture soft the eye stays on the lot in its middle.
 *
 * What is sharp is a band of the ground by its distance from the eye: between `near` and `far` times the
 * camera's distance from what it looks at. So the rows follow the camera's tilt (looking straight down,
 * the whole ground is as far away and nothing blurs; looking low, the far ground blurs from higher up) and
 * not its zoom. From each edge of the band the blur grows to `px` (its radius in pixels of a picture 1080
 * high) at the picture's edge. Zoomed in on someone it would only be in the way: it fades out between
 * `radius`'s two camera distances (m). At the camera a game opens with (24 m, 0.86 from straight down) the
 * band is the rows 0.34 to 0.72, the look chosen from pictures side by side on 2026-11-11.
 *
 * Drawn by the compose (`pc/shaders/post.ts`): twelve taps of the scene's colour round a pixel outside the band.
 */

import { tuned } from '../tune';

export const TILT = tuned('tilt', { near: 0.854, far: 1.164, px: 8, radius: [9, 18] });

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));

/**
 * The rows (0 the picture's top, 1 its bottom) between which the ground is in focus, for a camera `beta` from
 * straight down (rad) with the vertical field of view `fov` (rad), looking at the ground. Always a band round
 * the middle: the first above it, the second below (at the very edge: nothing blurs there).
 */
export function tiltRows(beta: number, fov: number, near: number = TILT.near, far: number = TILT.far): [number, number] {
  const down = Math.cos(beta);
  // The ray `t` above the view's middle meets the ground cos(beta) / cos(beta + t) camera distances off: the row whose
  // ground is `k` of them off. (No ground is nearer than straight below the eye, cos(beta) of them: then no row is.)
  const row = (k: number) => (down >= k ? Infinity : 0.5 - Math.tan(Math.acos(down / k) - beta) / (2 * Math.tan(fov / 2)));
  return [clamp(row(far), 0.001, 0.499), clamp(row(near), 0.501, 0.999)];
}

/** How much of the blur a camera `radius` m from what it looks at has (0: none, zoomed in; 1: all of it). */
export function tiltShare(radius: number): number {
  const t = clamp((radius - TILT.radius[0]) / (TILT.radius[1] - TILT.radius[0]), 0, 1);
  return t * t * (3 - 2 * t);
}
