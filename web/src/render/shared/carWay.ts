/**
 * Where a car is drawn between two snapshots (engine-neutral; `render/pc/cars.ts` places it there each frame): along
 * the way it went (sim-core `snapshot::car::TRAIL`, its poses tick by tick over the last few steps, as few kept as draw
 * it within a few centimetres of all of them), not on the straight line between them. At a faster speed a step runs
 * many ticks (at super speed up to 60, some 50 m of driving): on the straight line a car cut across the bends and
 * turns it took, onto the wrong side of the road and into the cars beside it. At normal speed a step is a tick, its way
 * has no point between two snapshots, and a car goes on the straight line as it always did.
 */

import type { SnapshotLayout } from '../../core/snapshot';
import { lerpAngle } from './characters/rig';

/** A car's middle on the ground and which way it faces (rad, as the snapshot's yaw). */
export interface WayPose {
  x: number;
  z: number;
  yaw: number;
}

/** A snapshot's way for one car: `n` points from float `at` of `snap`, `stride` apart, laid out as `fields`. */
export interface Way {
  snap: ArrayLike<number>;
  at: number;
  n: number;
  stride: number;
  fields: NonNullable<SnapshotLayout['trail']>;
}

/** No way: the straight line from the snapshot before. */
export const NO_WAY: Way = { snap: [], at: 0, n: 0, stride: 4, fields: { t: 0, x: 1, z: 2, yaw: 3 } };

/** Car row `o`'s way in `snap` (`car.trail`, `car.trailLen`), or `NO_WAY` in a layout without ways. */
export function wayOf(snap: ArrayLike<number>, layout: SnapshotLayout, o: number, into: Way): Way {
  const f = layout.car;
  if (f?.trail === undefined || f.trailLen === undefined || layout.trailBase === undefined || layout.trailStride === undefined || !layout.trail) return NO_WAY;
  into.snap = snap;
  into.n = snap[o + f.trailLen] | 0;
  into.stride = layout.trailStride;
  into.at = layout.trailBase + (snap[o + f.trail] | 0) * layout.trailStride;
  into.fields = layout.trail;
  return into;
}

/**
 * Where a car is at `r` steps from the newest snapshot (`curr`, at 0), `prev` being where the one `steps` before it had
 * it: from -`steps` to 0 along its `way` (its points after `prev`, then `curr`); past 0 (the next snapshot late) on along
 * the last stretch of it, as far as it goes in `ahead` steps at most. Into `out`.
 */
export function poseOnWay(prev: WayPose, curr: WayPose, way: Way, steps: number, r: number, ahead: number, out: WayPose): WayPose {
  const { snap, at, n, stride, fields: f } = way;
  const from = -Math.max(steps, 0);
  const t = Math.min(Math.max(r, from), ahead > 0 ? ahead : 0);
  // The stretch `t` is on: from point `a` (-1: `prev`) to point `b` (`n`: `curr`), the points before `prev` left out.
  let a = -1;
  let b = 0;
  while (b < n && snap[at + b * stride + f.t] <= from) b++;
  while (b < n && snap[at + b * stride + f.t] < t) a = b++;
  // (Past `curr`: on along the last stretch.)
  if (t > 0) b = n;
  const ta = a < 0 ? from : snap[at + a * stride + f.t];
  const tb = b >= n ? 0 : snap[at + b * stride + f.t];
  const [ax, az, ayaw] = a < 0 ? [prev.x, prev.z, prev.yaw] : [snap[at + a * stride + f.x], snap[at + a * stride + f.z], snap[at + a * stride + f.yaw]];
  const [bx, bz, byaw] = b >= n ? [curr.x, curr.z, curr.yaw] : [snap[at + b * stride + f.x], snap[at + b * stride + f.z], snap[at + b * stride + f.yaw]];
  const k = tb > ta ? (t - ta) / (tb - ta) : 1;
  out.x = ax + (bx - ax) * k;
  out.z = az + (bz - az) * k;
  out.yaw = lerpAngle(ayaw, byaw, k);
  return out;
}
