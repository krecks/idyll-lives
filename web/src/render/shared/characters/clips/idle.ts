/**
 * Standing clips on the relaxed stance: idle, talk, folded arms, nodding and shaking the head, the
 * phone, reading a book (`lantern`) and a wave. The feet stay planted (leg IK) while the weight
 * shifts over them; hands are placed with arm IK in model space of the reference skeleton.
 */

import { clip, cycle, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { armIK, elbow, footIK, hang, pointOn, ramp, smooth, wrist } from './ik';

export const TAU = Math.PI * 2;

/** Relaxed hanging arms: forward `fwd`, out `out` degrees, elbows bent `bend` (computed while baking). */
export function hangArms(fwd = 4, out = 7, bend = 14, l: Partial<{ fwd: number; out: number; bend: number }> = {}, r: Partial<{ fwd: number; out: number; bend: number }> = {}): PoseSpec {
  const one = (side: 'l' | 'r', o: Partial<{ fwd: number; out: number; bend: number }>): PoseSpec => ({
    [`upperarm_${side}`]: hang(side, o.fwd ?? fwd, o.out ?? out),
    [`lowerarm_${side}`]: elbow(side, o.bend ?? bend, 12),
    [`hand_${side}`]: wrist(side, 8),
  });
  return { ...one('l', l), ...one('r', r) };
}

/** Breathing (phase 0..1 of one breath; `depth` scales it): the chest lifts, shoulders rise a touch, the head stays level. */
export function breathe(p: number, depth = 1): PoseSpec {
  const s = Math.sin(p * TAU);
  return { spine_02: [-1.0 * s * depth, 0, 0], spine_03: [-0.8 * s * depth, 0, 0], clavicle: [0, 0, -0.8 * s * depth], neck_01: [0.6 * s * depth, 0, 0], Head: [0.9 * s * depth, 0, 0], move: [0, 0.002 * s * depth, 0] };
}

export interface StandOptions {
  /** Weight over the left (-1) .. right (+1) leg. */
  shift?: number;
  /** Breath phase and depth. */
  breath?: number;
  depth?: number;
  /** Pelvis lowered (m) and moved forward (m); back bent forward (deg, spread over the spine). */
  sink?: number;
  fwd?: number;
  lean?: number;
  /** Turn of the chest (deg, + to the left) and the head's own turn / nod / tilt (deg). */
  twist?: number;
  head?: Vec;
  /** Feet offsets from rest (left; the right mirrors unless given). */
  feet?: Vec;
  footR?: Vec;
}

/**
 * The standing body: pelvis over the feet, the weight shifted to one leg (the hip drops on the
 * free side, the chest stays upright), breathing, the feet planted where they are.
 */
export function stand(o: StandOptions = {}): PoseSpec {
  const shift = o.shift ?? 0;
  const b = breathe(o.breath ?? 0, o.depth ?? 1);
  const lean = o.lean ?? 0;
  const twist = o.twist ?? 0;
  const head = o.head ?? [0, 0, 0];
  const bm = b.move ?? [0, 0, 0];
  const body: PoseSpec = {
    move: [0.022 * shift, -(o.sink ?? 0) - 0.006 * Math.abs(shift) + bm[1], o.fwd ?? 0],
    pelvis: [0, -2 * shift, 3 * shift],
    spine_01: [lean * 0.35, 1 * shift + twist * 0.2, -2 * shift],
    spine_02: [lean * 0.35 + (b.spine_02?.[0] ?? 0), twist * 0.35, -1.2 * shift],
    spine_03: [lean * 0.3 + (b.spine_03?.[0] ?? 0), twist * 0.45 + 1 * shift, -0.4 * shift],
    clavicle_l: b.clavicle,
    clavicle_r: [0, 0, -(b.clavicle?.[2] ?? 0)],
    neck_01: [-lean * 0.35 + (b.neck_01?.[0] ?? 0) + head[0] * 0.3, -twist * 0.3 + head[1] * 0.35, -shift * 1 + head[2] * 0.3],
    Head: [-lean * 0.35 + (b.Head?.[0] ?? 0) + head[0] * 0.7, -twist * 0.5 + head[1] * 0.65, head[2] * 0.7 + 1.5 * shift],
  };
  const fl = o.feet ?? [0, 0, 0];
  const fr = o.footR ?? [-fl[0], fl[1], fl[2]];
  return { ...body, ...footIK('l', fl, body, { foot: [0, -4, 0] }), ...footIK('r', fr, body, { foot: [0, 4, 0] }) };
}

/** Both arms placed by IK on top of a body pose (targets in model space; omit a side to leave it). */
export function reach(body: PoseSpec, l: Parameters<typeof armIK>[3] & { at: Vec } | null, r: Parameters<typeof armIK>[3] & { at: Vec } | null): PoseSpec {
  return { ...(l ? armIK('l', l.at, body, l) : {}), ...(r ? armIK('r', r.at, body, r) : {}) };
}

/** A smooth bump: 0 outside a..b, rising to 1 by a+edge and falling from b-edge. */
export function bump(t: number, a: number, b: number, edge = 0.12): number {
  return ramp(t, a, a + edge) * (1 - ramp(t, b - edge, b));
}

function idlePose(p: number): PoseSpec {
  // One breath per loop, the weight drifting from leg to leg; a glance aside and a small fidget.
  const shift = 0.55 * Math.sin(p * TAU - 0.6);
  const look = 6 * Math.sin(p * TAU + 0.9) * smooth(0.5 + 0.5 * Math.sin(p * TAU));
  const body = stand({ shift, breath: p, head: [1.5 * Math.sin(p * TAU * 2), look, 1.5 * Math.sin(p * TAU)] });
  const fidget = bump(p, 0.55, 0.85, 0.1);
  return {
    ...body,
    ...hangArms(4 + 1.5 * Math.sin(p * TAU + 2), 7, 14, { fwd: 3 - 1.5 * Math.sin(p * TAU + 2) }, { bend: 14 + 14 * fidget, fwd: 4 + 5 * fidget }),
    hand_r: wrist('r', 8 + 12 * fidget * Math.sin(p * TAU * 6), 4 * fidget),
  };
}

/** Talking: three gestures in turn (an open hand offered, both hands shaping something, a beat with the other hand), nods on the beats. */
function talkPose(p: number): PoseSpec {
  const g1 = bump(p, 0.02, 0.34, 0.1);
  const g2 = bump(p, 0.34, 0.66, 0.1);
  const g3 = bump(p, 0.66, 0.98, 0.1);
  const beat = Math.sin(p * TAU * 6);
  const body = stand({ shift: 0.35 * Math.sin(p * TAU), breath: p, lean: 2 + 2 * g2, twist: 6 * g1 - 6 * g3, head: [3 * g2 + 2.5 * Math.max(0, beat) * (g1 + g3), 5 * g3 - 6 * g1, 3 * g1 - 3 * g3] });
  // Rest places of the hands (in front of the thighs), and the gesture places.
  const restL: Vec = [-0.2, 0.9, 0.08];
  const restR: Vec = [0.2, 0.9, 0.08];
  const mixV = (a: Vec, b: Vec, t: number): Vec => [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
  const offer: Vec = [0.3 + 0.03 * Math.sin(p * TAU * 3), 1.07 + 0.02 * beat * g1, 0.3];
  const shapeL: Vec = [-0.15 - 0.04 * Math.sin(p * TAU * 3), 1.12, 0.3];
  const shapeR: Vec = [0.15 + 0.04 * Math.sin(p * TAU * 3), 1.12, 0.3];
  const point: Vec = [-0.2, 1.15 + 0.035 * beat * g3, 0.32];
  const l = mixV(mixV(restL, shapeL, g2), point, g3);
  const r = mixV(mixV(restR, offer, g1), shapeR, g2);
  return {
    ...body,
    ...armIK('l', l, body, { pronate: -50 * g2 - 10 * g3, hand: wrist('l', 10 - 20 * g2, 0) }),
    ...armIK('r', r, body, { pronate: -70 * g1 - 50 * g2, hand: wrist('r', 10 - 25 * g1 - 20 * g2, 0) }),
  };
}

/** Arms folded across the chest, the left forearm over the right; weight on one leg. */
function foldPose(p: number): PoseSpec {
  const body = stand({ shift: -0.6 + 0.12 * Math.sin(p * TAU), breath: p, depth: 1.2, head: [-1, 4 * Math.sin(p * TAU + 1), 3] });
  const lift = 0.004 * Math.sin(p * TAU);
  return {
    ...body,
    ...armIK('l', [0.15, 1.17 + lift, 0.18], body, { pole: [-0.3, -1, 0.05], pronate: 10, hand: wrist('l', 25, 0) }),
    ...armIK('r', [-0.14, 1.14 + lift, 0.14], body, { pole: [0.3, -1, 0.05], pronate: 10, hand: wrist('r', 25, 0) }),
  };
}

/**
 * Phone to the right ear (the palm over it, the fingers up the side of the head, the phone between: actions.ts
 * `PROPS.phoneEar`), the left arm across the waist holding the right elbow; small nods and shifts.
 */
function phonePose(p: number): PoseSpec {
  const talk = Math.max(0, Math.sin(p * TAU * 2));
  const body = stand({ shift: 0.5 + 0.15 * Math.sin(p * TAU), breath: p, twist: -4, head: [4 + 3 * talk * Math.sin(p * TAU * 8), -6 + 4 * Math.sin(p * TAU), -9] });
  return {
    ...body,
    ...armIK('r', pointOn(body, 'Head', PHONE_AT), body, { palm: true, pole: [0.5, -1, 0.5], pronate: 60, hand: wrist('r', -10, PHONE_DEV) }),
    ...armIK('l', [0.12, 1.06, 0.13], body, { pole: [-0.3, -1, 0], pronate: 70, hand: wrist('l', 30, 0) }),
  };
}

/** The palm on the phone at the right ear, from the head joint (m, the head's frame), and the wrist turned to the little finger so the fingers go up. */
const PHONE_AT: Vec = [0.11, 0.04, 0.03];
const PHONE_DEV = -30;

/** Reading: an open book held in both hands at the chest, the head bowed over it; a page turns. */
function readPose(p: number): PoseSpec {
  const turn = bump(p, 0.6, 0.85, 0.08);
  const across = Math.sin(Math.PI * ramp(p, 0.6, 0.85));
  const body = stand({ shift: 0.3 * Math.sin(p * TAU), breath: p, lean: 3, head: [18 - 2 * turn, 3 * Math.sin(p * TAU) - 4 * across, 0] });
  return {
    ...body,
    ...armIK('l', [-0.11, 1.22, 0.3], body, { palm: true, pole: [-0.3, -1, -0.2], pronate: -60, hand: wrist('l', -10, 0) }),
    ...armIK('r', [0.11 - 0.17 * across, 1.22 + 0.03 * turn, 0.3 + 0.02 * turn], body, { palm: true, pole: [0.3, -1, -0.2], pronate: -60 + 50 * turn, hand: wrist('r', -10 + 20 * turn, 0) }),
  };
}

export function idleClips(): ClipDef[] {
  const yesBase = (p: number, nod: number): PoseSpec => {
    const body = stand({ shift: 0.3, breath: p * 0.8, head: [nod, 0, 2], lean: nod * 0.08 });
    return { ...body, ...hangArms(5, 7, 16) };
  };
  return [
    cycle('idle', { seconds: 2.5, loop: true }, idlePose),
    cycle('talk', { seconds: 88 / 30, loop: true }, talkPose),
    cycle('foldArms', { seconds: 2.5, loop: true }, foldPose),
    // Nodding: a ready lift of the head, two nods (the second smaller), settling.
    clip('yes', { seconds: 2.5, loop: false }, [
      [0, () => yesBase(0, 0)],
      [0.1, () => yesBase(0.1, -5), 'out'],
      [0.24, () => yesBase(0.24, 16)],
      [0.36, () => yesBase(0.36, -3)],
      [0.48, () => yesBase(0.48, 12)],
      [0.6, () => yesBase(0.6, 0)],
      [0.72, () => yesBase(0.72, 4)],
      [1, () => yesBase(1, 0)],
    ]),
    // Shaking the head (argue, disagree): side to side, the free hand turned out in protest.
    cycle('no', { seconds: 2.5, loop: true }, (p) => {
      const amp = 0.75 + 0.25 * Math.cos(p * TAU);
      const shake = Math.sin(p * TAU * 3);
      const body = stand({ shift: -0.4 * Math.cos(p * TAU), breath: p, lean: -1, head: [2, 15 * amp * shake, -2 * shake], twist: -3 * shake });
      const up = 0.5 + 0.5 * Math.cos(p * TAU);
      return { ...body, ...hangArms(5, 7, 16), ...armIK('r', [0.24 + 0.03 * shake, 1.0 + 0.06 * up, 0.24], body, { pronate: -40, hand: wrist('r', -20, 0) }) };
    }),
    cycle('phone', { seconds: 88 / 30, loop: true }, phonePose),
    cycle('lantern', { seconds: 2.5, loop: true }, readPose),
    // A friendly wave (phase 2): the right hand up beside the head, three waves, down again.
    clip('wave', { seconds: 2.2, loop: false }, [
      ...[0, 0.15, 0.25, 0.35, 0.45, 0.55, 0.65, 0.75, 0.86, 1].map((t, i): [number, () => PoseSpec, 'smooth'] => [
        t,
        () => {
          const up = t > 0.05 && t < 0.9 ? 1 : 0;
          const body = stand({ shift: -0.3, breath: t, twist: 4 * up, head: [-2 * up, -4 * up, 4 * up] });
          const side = i % 2 ? 0.05 : -0.05;
          return {
            ...body,
            ...hangArms(),
            ...(up ? armIK('r', [0.33 + side, 1.63, 0.12], body, { pole: [1, -0.7, -0.2], pronate: -10, hand: wrist('r', -10, side * 200) }) : {}),
          };
        },
        'smooth',
      ]),
    ]),
  ];
}
