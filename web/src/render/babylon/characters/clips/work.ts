/**
 * Standing work: reaching to use something, picking up, hands busy at a counter (`spell`),
 * pushing a mop or vacuum, leaning on a rail (telescope). STAND-IN versions (stream 0); stream C
 * replaces them.
 */

import { add, clip, cycle, legIK, type ClipDef } from './dsl';
import { ARMS, breathe, TAU } from './idle';

export function workClips(): ClipDef[] {
  const reach = { upperarm: [-58, 0, 26], lowerarm: [-28, 0, 0], spine_01: [6, 0, 0], spine_02: [4, 0, 0] } as const;
  const squat = (depth: number) => {
    const move: [number, number, number] = [0, -depth, -0.08 * depth];
    return add(legIK('l', [0, 0, 0.03], move), legIK('r', [0, 0, 0.03], move), { spine_01: [40 * depth, 0, 0], spine_02: [20 * depth, 0, 0], upperarm: [-50, 0, 30], lowerarm: [-15, 0, 0], move });
  };
  return [
    clip('interact', { seconds: 60 / 30, loop: false }, [
      [0, ARMS],
      [0.3, reach],
      [0.5, { ...reach, lowerarm_r: [-40, 0, 0], hand_r: [0, 0, 15] }],
      [0.7, { ...reach, lowerarm_l: [-40, 0, 0], hand_l: [0, 0, 15] }],
      [1, ARMS],
    ]),
    clip('pickUp', { seconds: 25 / 30, loop: false }, [
      [0, ARMS],
      [0.5, () => squat(0.32)],
      [1, ARMS],
    ]),
    cycle('spell', { seconds: 63 / 30, loop: true }, (p) =>
      add(breathe(p), {
        upperarm: [-36, 0, 28],
        lowerarm: [-62, 12, 0],
        lowerarm_l: [-62 - 8 * Math.sin(p * TAU * 3), 12, 0],
        lowerarm_r: [-62 - 8 * Math.sin(p * TAU * 3 + 1.5), -12, 0],
        spine_01: [8, 0, 0],
        Head: [16, 0, 0],
      }),
    ),
    cycle('push', { seconds: 80 / 30, loop: true }, (p) => {
      const s = Math.sin(p * TAU);
      return { upperarm: [-40 - 14 * s, 0, 22], lowerarm: [-35 + 12 * s, 0, 0], spine_01: [10 + 4 * s, 0, 0], move: [0, -0.01, 0.03 * s] };
    }),
    cycle('rail', { seconds: 2.5, loop: true }, (p) => add(breathe(p), { spine_01: [14, 0, 0], spine_02: [10, 0, 0], upperarm: [-72, 0, 24], lowerarm: [-55, 0, 0], Head: [-12, 0, 0] })),
  ];
}
