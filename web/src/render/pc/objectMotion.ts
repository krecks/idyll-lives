/**
 * Placed objects' moving parts on PlayCanvas (a fridge's doors; ported from the Babylon renderer, removed: see git history up to f15b839b):
 * the engine-neutral motion (`shared/objectMotion.ts`: which clips play, how far each is) writing
 * into instanced draws (`instancing.ts`). Each instance's parts draw from buffers of their own
 * (`MovingTarget`), written only while something moves, and sent up once a frame however many of
 * their instances moved.
 */

import type { Instances } from './instancing';
import { multiply } from '../shared/math';
import type { ModelClip } from '../shared/modelClips';
import { ObjectMotion as SharedObjectMotion, poseParts as sharedPoseParts, type MotionHooks, type MotionTarget, type MovingObject as SharedMovingObject } from '../shared/objectMotion';

export type { Moved } from '../shared/objectMotion';

/** One placed object's instance: its draws, where its matrix lives in their buffer (game space), and its moving parts. */
export type PopTarget = MotionTarget<Instances> & { meshes: Instances[] };

/** A placed object with moving parts. */
export type MovingObject = SharedMovingObject<Instances> & { target: PopTarget };

const base = new Float32Array(16);
const posed = new Float32Array(16);

/**
 * Writes an instance's moving parts after its matrix: each part's pose, then the instance's
 * placement. The draws go up now, or, given `written`, go in it for the caller to send up once
 * (many instances of one model moving in a frame).
 */
export function writeMoving(target: MotionTarget<Instances>, written?: Set<Instances>): void {
  const moving = target.moving;
  if (!moving) return;
  const o = target.index * 16;
  base.set(target.matrices.subarray(o, o + 16));
  moving.parts.forEach(({ meshes, matrices }, k) => {
    // (Babylon's `pose.multiply(base)`: posed in the part's own frame, then placed.)
    posed.set(moving.poses.subarray(k * 16, k * 16 + 16));
    multiply(posed, base, posed);
    matrices.set(posed, o);
    for (const draw of meshes) {
      if (draw.destroyed) continue;
      draw.setMatrix(target.index, matrices, o);
      if (written) written.add(draw);
      else draw.update();
    }
  });
}

/** Writes `matrix` as an instance's placement (its draws and its moving parts after it), sent up now. */
export function writeInstance(target: PopTarget, matrix: ArrayLike<number>): void {
  target.matrices.set(matrix, target.index * 16);
  for (const draw of target.meshes) {
    if (draw.destroyed) continue;
    draw.setMatrix(target.index, target.matrices, target.index * 16);
    draw.update();
  }
  writeMoving(target);
}

const HOOKS: MotionHooks<Instances> = {
  writeMoving,
  upload(draw, bounds) {
    if (!draw.destroyed) draw.update(bounds);
  },
};

export class ObjectMotion extends SharedObjectMotion<Instances> {
  constructor() {
    super(HOOKS);
  }
}

/**
 * Writes an instance's moving parts where the clips playing (`playing`, each at its seconds) have
 * them together (at rest where none does), after its placement (`writeMoving`, which `written` is
 * passed on to).
 */
export function poseParts(target: PopTarget, playing: readonly { clip: ModelClip; time: number; kind?: string }[], written?: Set<Instances>): void {
  sharedPoseParts(target, playing, writeMoving, written);
}
