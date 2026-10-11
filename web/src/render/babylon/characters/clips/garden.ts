/**
 * Gardening: watering, harvesting, planting, kneeling to weed, crouching to smell flowers.
 * STAND-IN versions (stream 0); stream C replaces them.
 */

import { add, clip, cycle, legIK, type ClipDef, type PoseSpec } from './dsl';
import { ARMS, breathe, TAU } from './idle';

/** A deep crouch (feet flat, knees forward) with the pelvis `depth` m down. */
function crouch(depth: number, lean = 30): PoseSpec {
  const move: [number, number, number] = [0, -depth, -0.1 * depth];
  return add(legIK('l', [-0.02, 0, 0.04], move), legIK('r', [0.02, 0, 0.04], move), { spine_01: [lean, 0, 0], spine_02: [lean * 0.5, 0, 0], move });
}

export function gardenClips(): ClipDef[] {
  return [
    cycle('water', { seconds: 114 / 30, loop: true }, (p) =>
      add(breathe(p), { ...ARMS, upperarm_r: [-42, 0, -24], lowerarm_r: [-24, 0, 0], hand_r: [0, 0, -12 - 10 * Math.sin(p * TAU)], Head: [18, 0, 0] }),
    ),
    cycle('harvest', { seconds: 2.5, loop: true }, (p) => {
      const s = Math.sin(p * TAU);
      return { spine_01: [26, 0, 0], spine_02: [14, 0, 0], upperarm_l: [-55 - 12 * s, 0, 30], upperarm_r: [-55 + 12 * s, 0, -30], lowerarm: [-25, 0, 0], Head: [10, 0, 0], move: [0, -0.02, -0.03] };
    }),
    clip('plant', { seconds: 83 / 30, loop: false }, [
      [0, ARMS],
      [0.25, () => add(crouch(0.4), { upperarm: [-60, 0, 28], lowerarm: [-20, 0, 0] })],
      [0.45, () => add(crouch(0.4), { upperarm: [-45, 0, 28], lowerarm: [-35, 0, 0] })],
      [0.6, () => add(crouch(0.4), { upperarm: [-62, 0, 28], lowerarm: [-15, 0, 0] })],
      [0.75, () => add(crouch(0.4), { upperarm: [-50, 0, 28], lowerarm: [-30, 0, 0] })],
      [1, ARMS],
    ]),
    // Kneeling to weed: on both knees, hands at the soil.
    cycle('kneel', { seconds: 156 / 30, loop: true }, (p) => {
      const s = Math.sin(p * TAU * 2);
      return { move: [0, -0.48, -0.12], thigh: [-15, 0, -4], calf: [105, 0, 0], foot: [-40, 0, 0], spine_01: [32, 0, 0], spine_02: [14, 0, 0], upperarm_l: [-62 - 8 * s, 0, 26], upperarm_r: [-62 + 8 * s, 0, -26], lowerarm: [-15, 0, 0], Head: [8, 0, 0] };
    }),
    cycle('crouch', { seconds: 88 / 30, loop: true }, (p) => add(crouch(0.42, 34), breathe(p), { upperarm: [-40, 0, 32], lowerarm: [-40, 0, 0], Head: [20, 0, 0], neck_01: [8, 0, 0] })),
  ];
}
