/**
 * Walking and jogging. STAND-IN versions (stream 0): the planted foot slides back at the walking
 * speed and the swing foot arcs forward (leg IK), the pelvis bobs twice per cycle, the arms swing
 * against the legs; stream C replaces them.
 */

import { add, cycle, legIK, type ClipDef, type PoseSpec } from './dsl';
import { ARMS, TAU } from './idle';

/** Reference walk: 1.1 m/s over a 1.1 s cycle. */
const WALK_SPEED = 1.1;
const WALK_SECONDS = 1.1;

/** Ankle offset of a foot at phase p of its own cycle (0..0.5 planted, sliding back; then swinging). */
function foot(p: number, stride: number, lift: number): [number, number, number] {
  const q = ((p % 1) + 1) % 1;
  if (q < 0.5) return [0, 0, stride / 2 - (q / 0.5) * stride];
  const s = (q - 0.5) / 0.5;
  const e = s * s * (3 - 2 * s);
  return [0, Math.sin(s * Math.PI) * lift, -stride / 2 + e * stride];
}

function walkPose(p: number, stride: number, lift: number, bob: number, sink: number, arm: number): PoseSpec {
  const move: [number, number, number] = [0, -sink - bob * (0.5 - 0.5 * Math.cos(p * TAU * 2)), 0];
  const swing = Math.sin(p * TAU);
  return add(
    legIK('l', foot(p, stride, lift), move),
    legIK('r', foot(p + 0.5, stride, lift), move),
    { ...ARMS, upperarm_l: [-arm * swing, 0, 36], upperarm_r: [arm * swing, 0, -36], lowerarm: [-18, 0, 0] },
    { spine_02: [0, 4 * swing, 0], move },
  );
}

export function locomotionClips(): ClipDef[] {
  const stride = (WALK_SPEED * WALK_SECONDS) / 2;
  return [
    cycle('walk', { seconds: WALK_SECONDS, loop: true, speed: WALK_SPEED }, (p) => walkPose(p, stride, 0.06, 0.02, 0.062, 18)),
    // Jogging on the spot (treadmills, exercise): knees up, arms bent.
    cycle('jog', { seconds: 28 / 30, loop: true }, (p) => {
      const l = Math.max(0, Math.sin(p * TAU));
      const r = Math.max(0, -Math.sin(p * TAU));
      const swing = Math.sin(p * TAU);
      return {
        thigh_l: [-50 * l, 0, 0],
        calf_l: [85 * l, 0, 0],
        foot_l: [-30 * l, 0, 0],
        thigh_r: [-50 * r, 0, 0],
        calf_r: [85 * r, 0, 0],
        foot_r: [-30 * r, 0, 0],
        upperarm_l: [-30 * swing - 10, 0, 34],
        upperarm_r: [30 * swing - 10, 0, -34],
        lowerarm: [-85, 0, 0],
        spine_01: [6, 0, 0],
        move: [0, 0.03 * Math.abs(Math.sin(p * TAU)) - 0.02, 0],
      };
    }),
  ];
}
