/**
 * Lying down (phase 2): `lie`, on the back in the standing convention (the runtime turns the
 * resident flat with its -90 degree pitch). Legs long and loose (knees a touch bent, feet falling
 * out), down on the mattress: the heels lie as deep as the back (the ankles about 7 cm below the
 * pelvis joint, `check_clips.ts`), not lifted. Hands resting on the belly, the head propped on the
 * pillow and turned a little aside, slow breathing. For `CLIP_OF.lie` and `lyingPose`; the runtime
 * plays lying at a third of the speed, so the loop holds two breaths (about four seconds each in game).
 */

import { cycle, type ClipDef, type PoseSpec } from './dsl';
import { TAU } from './idle';
import { armIK, footIK, wrist } from './ik';

/** The lying pose at phase `p` (made standing: the runtime turns it flat; the bunk's ladder ends in it, `rides.ts`). */
export function liePose(p: number): PoseSpec {
  const b = Math.sin(p * TAU * 2);
  const body: PoseSpec = {
    move: [0, 0, 0],
    pelvis: [-4, 0, 0],
    spine_01: [2, 0, 0],
    spine_02: [-1.4 * b, 0, 0],
    spine_03: [-1.0 * b, 0, 0],
    neck_01: [10, 4, 0],
    Head: [8 + 0.6 * b, 9, 4],
  };
  const lift = 0.004 * b;
  return {
    ...body,
    ...footIK('l', [-0.03, 0.002, -0.055], body, { foot: [30, -16, 0], pole: [-0.2, 0, 1] }),
    ...footIK('r', [0.03, 0.002, -0.055], body, { foot: [30, 16, 0], pole: [0.2, 0, 1] }),
    ...armIK('l', [-0.08, 1.02, 0.14 + lift], body, { palm: true, pole: [-1, -0.4, -0.3], pronate: 60, hand: wrist('l', 10, 0) }),
    ...armIK('r', [0.07, 1.05, 0.15 + lift], body, { palm: true, pole: [1, -0.4, -0.3], pronate: 60, hand: wrist('r', 10, 0) }),
  };
}

export function lieClips(): ClipDef[] {
  return [cycle('lie', { seconds: 2.4, loop: true }, liePose)];
}
