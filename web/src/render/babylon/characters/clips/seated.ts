/**
 * Sitting: the seated loop, talking and desk work seated, sitting down and standing up, and eating
 * (also the seated upper-body layer). STAND-IN versions (stream 0); stream C replaces them.
 *
 * The seated pelvis sits where a 0.45 m seat puts it (`seatDrop` above the seat; the runtime fits
 * the clip to each seat with `measureSit`), thighs level, shins down to the floor.
 */

import { add, clip, cycle, legIK, type ClipDef, type PoseSpec } from './dsl';
import { ARMS, breathe, TAU } from './idle';

/** Pelvis offset when seated (reference skeleton), and the feet's offset from their rest place. */
const SEAT_MOVE: [number, number, number] = [0, -0.35, -0.04];
const SEAT_FEET: [number, number, number] = [0, 0.0, 0.36];

/** The seated legs and back (a function: the leg IK runs while the clips bake). */
export function seated(): PoseSpec {
  return add(legIK('l', [0.02, SEAT_FEET[1], SEAT_FEET[2]], SEAT_MOVE), legIK('r', [-0.02, SEAT_FEET[1], SEAT_FEET[2]], SEAT_MOVE), { spine_01: [-4, 0, 0], move: SEAT_MOVE });
}

/** Hands resting on the thighs. */
const LAP: PoseSpec = { upperarm: [-28, 0, 34], lowerarm: [-48, 8, 0], hand: [-8, 0, 0] };

export function seatedClips(): ClipDef[] {
  return [
    cycle('sit', { seconds: 50 / 30, loop: true }, (p) => add(seated(), LAP, breathe(p, 0.7))),
    cycle('sitTalk', { seconds: 88 / 30, loop: true }, (p) => {
      const g = Math.max(0, Math.sin(p * TAU * 2));
      return add(seated(), breathe(p, 0.7), { ...LAP, upperarm_r: [-30 - 10 * g, 0, -26], lowerarm_r: [-70 - 25 * g, -10, 0], Head: [0, 5 * Math.sin(p * TAU), 0] });
    }),
    clip('sitDown', { seconds: 39 / 30, loop: false }, [
      [0, ARMS],
      [0.45, () => add(legIK('l', [0, 0, 0.12], [0, -0.17, -0.06]), legIK('r', [0, 0, 0.12], [0, -0.17, -0.06]), { ...LAP, spine_01: [18, 0, 0], spine_02: [8, 0, 0], move: [0, -0.17, -0.06] })],
      [1, () => add(seated(), LAP)],
    ]),
    clip('standUp', { seconds: 31 / 30, loop: false }, [
      [0, () => add(seated(), LAP)],
      [0.4, () => add(legIK('l', [0, 0, 0.14], [0, -0.22, 0.02]), legIK('r', [0, 0, 0.14], [0, -0.22, 0.02]), { ...LAP, spine_01: [24, 0, 0], spine_02: [10, 0, 0], move: [0, -0.22, 0.02] })],
      [1, ARMS],
    ]),
    // Forearms on a desk at 0.72 m, fingers busy.
    cycle('sitHands', { seconds: 50 / 30, loop: true }, (p) =>
      add(seated(), breathe(p, 0.6), {
        upperarm: [-32, 0, 22],
        lowerarm: [-62, 14, 0],
        hand_l: [6 * Math.sin(p * TAU * 4), 0, 0],
        hand_r: [6 * Math.sin(p * TAU * 4 + 2), 0, 0],
        spine_01: [6, 0, 0],
        Head: [10, 0, 0],
      }),
    ),
    // Hand to mouth (standing, and the seated upper-body layer).
    clip('eat', { seconds: 40 / 30, loop: false }, [
      [0, ARMS],
      [0.4, { ...ARMS, upperarm_r: [-38, 0, -18], lowerarm_r: [-140, -35, 0], hand_r: [-10, 0, 0], Head: [-4, 0, 0] }],
      [0.65, { ...ARMS, upperarm_r: [-36, 0, -18], lowerarm_r: [-138, -35, 0], hand_r: [-10, 0, 0] }],
      [1, ARMS],
    ]),
  ];
}
