/**
 * Placed objects' moving parts on Babylon (a fridge's doors): the engine-neutral motion
 * (`shared/objectMotion.ts`: which clips play, how far each is) writing into thin instances. Each
 * instance's parts draw from buffers of their own (`MovingTarget`), written only while something
 * moves, and sent up once a frame however many of their instances moved.
 */

import { writeMoving, type PopTarget } from './buildFx';
import { Matrix, type Mesh } from './core';
import type { ModelClip } from '../shared/modelClips';
import { ObjectMotion as SharedObjectMotion, poseParts as sharedPoseParts, type MotionHooks, type MovingObject as SharedMovingObject } from '../shared/objectMotion';

export type { Moved } from '../shared/objectMotion';

/** A placed object with moving parts. */
export type MovingObject = SharedMovingObject<Mesh> & { target: PopTarget };

const HOOKS: MotionHooks<Mesh> = {
  writeMoving: (target, written) => writeMoving(target as PopTarget, written),
  upload(mesh, bounds) {
    if (mesh.isDisposed()) return;
    mesh.thinInstanceAllowAutomaticStaticBufferRecreation = true;
    mesh.thinInstanceBufferUpdated('matrix');
    if (bounds) mesh.thinInstanceRefreshBoundingInfo(false);
  },
};

export class ObjectMotion extends SharedObjectMotion<Mesh> {
  private readonly rideFound = new Matrix();

  constructor() {
    super(HOOKS);
  }

  /**
   * How object `id`'s seat (`motions.ride`: the first part its clip moves) has moved from rest, in
   * world space (a point at rest times it is where it is now), for the residents sitting or lying
   * on it; null while it rests.
   */
  rideMatrix(id: number): Matrix | null {
    const m = this.ride(id);
    return m ? Matrix.FromArrayToRef(m, 0, this.rideFound) : null;
  }
}

/**
 * Writes an instance's moving parts where the clips playing (`playing`, each at its seconds) have
 * them together (at rest where none does), after its placement (`writeMoving`, which `written` is
 * passed on to). The game's objects, and the colour studio's stage.
 */
export function poseParts(target: PopTarget, playing: readonly { clip: ModelClip; time: number; kind?: string }[], written?: Set<Mesh>): void {
  sharedPoseParts(target, playing, HOOKS.writeMoving, written);
}
