/**
 * Dancing and fighting: a bouncing dance, the jab and the cross (heavy bag, fights) and the hit
 * recoil. STAND-IN versions (stream 0); stream C replaces them.
 */

import { clip, cycle, type ClipDef, type PoseSpec } from './dsl';
import { TAU } from './idle';

/** Fists up in front of the chin. */
const GUARD: PoseSpec = { upperarm: [-42, 0, 18], lowerarm: [-118, 10, 0], spine_02: [0, 0, 0], thigh_l: [-8, 0, 0], calf_l: [10, 0, 0], foot_l: [-2, 0, 0], move: [0, -0.02, 0] };

export function actionClips(): ClipDef[] {
  return [
    cycle('dance', { seconds: 1, loop: true }, (p) => {
      const b = Math.abs(Math.sin(p * TAU));
      const s = Math.sin(p * TAU);
      return {
        move: [0.02 * s, -0.04 * b, 0],
        thigh: [-12 * b, 0, 0],
        calf: [22 * b, 0, 0],
        foot: [-10 * b, 0, 0],
        upperarm_l: [-30 - 25 * Math.max(0, s), 0, 20],
        upperarm_r: [-30 - 25 * Math.max(0, -s), 0, -20],
        lowerarm: [-90, 0, 0],
        spine_02: [0, 0, 6 * s],
        Head: [0, 0, -5 * s],
      };
    }),
    clip('punchJab', { seconds: 26 / 30, loop: false }, [
      [0, GUARD],
      [0.35, { ...GUARD, upperarm_l: [-85, 0, 8], lowerarm_l: [-12, 0, 0], spine_02: [0, 12, 0] }, 'out'],
      [0.6, GUARD],
      [1, GUARD],
    ]),
    clip('punchCross', { seconds: 1, loop: false }, [
      [0, GUARD],
      [0.4, { ...GUARD, upperarm_r: [-85, 0, -8], lowerarm_r: [-12, 0, 0], spine_02: [0, -22, 0], pelvis: [0, -10, 0] }, 'out'],
      [0.7, GUARD],
      [1, GUARD],
    ]),
    clip('hit', { seconds: 10 / 30, loop: false }, [
      [0, GUARD],
      [0.35, { ...GUARD, spine_02: [-14, 0, 0], spine_03: [-6, 0, 0], Head: [-14, 8, 0], move: [0, -0.02, -0.04] }, 'out'],
      [1, GUARD],
    ]),
  ];
}
