/**
 * Standing clips on the relaxed stance: idle, talk, folded arms, nodding / shaking the head, the
 * phone and the book (`lantern`). STAND-IN versions (stream 0): simple valid poses with the right
 * names, loops and lengths; stream C replaces them.
 */

import { add, clip, cycle, type ClipDef, type PoseSpec } from './dsl';

/** Arms down by the sides (from the rest A-pose), elbows soft. */
export const ARMS: PoseSpec = { clavicle: [0, 0, 4], upperarm: [-4, 0, 38], lowerarm: [-12, 0, 0] };

export const TAU = Math.PI * 2;

/** Breathing and a faint weight shift (added to a pose; phase 0..1 of the loop). */
export function breathe(p: number, depth = 1): PoseSpec {
  const s = Math.sin(p * TAU);
  return { spine_02: [-1.2 * s * depth, 0, 0], spine_03: [-0.6 * s * depth, 0, 0], Head: [0.8 * s * depth, 0, 0], move: [0.004 * Math.sin(p * TAU + 1), 0.002 * s, 0] };
}

export function idleClips(): ClipDef[] {
  return [
    cycle('idle', { seconds: 2.5, loop: true }, (p) => add(ARMS, breathe(p), { upperarm_r: [0, 0, -2 * Math.sin(p * TAU)] })),
    cycle('talk', { seconds: 88 / 30, loop: true }, (p) => {
      const g = Math.max(0, Math.sin(p * TAU * 2));
      const h = Math.max(0, Math.sin(p * TAU * 3 + 1));
      const arms: PoseSpec = {
        ...ARMS,
        upperarm_r: [-20 - 12 * g, 0, -30],
        lowerarm_r: [-50 - 25 * g, -10, 0],
        upperarm_l: [-10 - 8 * h, 0, 32],
        lowerarm_l: [-30 - 30 * h, 10, 0],
      };
      return add(arms, breathe(p), { Head: [2 * h, 4 * g - 2, 0] });
    }),
    cycle('foldArms', { seconds: 2.5, loop: true }, (p) => add({ clavicle: [0, 0, 2], upperarm: [-28, 0, 30], lowerarm: [-95, 62, 0], hand: [0, 0, -10] }, breathe(p))),
    clip('yes', { seconds: 2.5, loop: false }, [
      [0, ARMS],
      [0.15, { ...ARMS, Head: [14, 0, 0], neck_01: [4, 0, 0] }],
      [0.3, { ...ARMS, Head: [-3, 0, 0] }],
      [0.45, { ...ARMS, Head: [12, 0, 0], neck_01: [3, 0, 0] }],
      [0.62, ARMS],
      [1, ARMS],
    ]),
    cycle('no', { seconds: 2.5, loop: true }, (p) => add(ARMS, breathe(p), { Head: [0, 16 * Math.sin(p * TAU * 3), 0], neck_01: [0, 5 * Math.sin(p * TAU * 3), 0] })),
    cycle('phone', { seconds: 88 / 30, loop: true }, (p) =>
      add({ ...ARMS, upperarm_r: [-30, 0, -20], lowerarm_r: [-135, -30, 0], hand_r: [0, 0, 10] }, breathe(p), { Head: [0, 0, -4 + Math.sin(p * TAU)] }),
    ),
    cycle('lantern', { seconds: 2.5, loop: true }, (p) => add({ upperarm: [-18, 0, 34], lowerarm: [-92, 18, 0], hand: [-10, 0, 0], Head: [14, 0, 0], neck_01: [6, 0, 0] }, breathe(p))),
  ];
}
