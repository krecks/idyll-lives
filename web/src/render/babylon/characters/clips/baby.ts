/**
 * A baby lying on its back in the crib, in the standing convention like `lie` (the runtime turns
 * the baby flat and plays these at their own speed): `babyLieAsleep` and `babyLieAwake`.
 *
 * The base pose is a newborn's, not a grown-up's: hips and knees bent with the legs drawn up and
 * fallen apart ("frog legs"), the soles turned in towards each other; the arms bent at the elbows
 * with the hands up by the head; the head turned a little to one side. Asleep the baby is still,
 * breathing quickly and shallowly from the belly, with one small twitch of a hand and a foot each
 * loop. Awake the legs kick in bursts (one after the other), the arms wave and the head turns from
 * side to side. Everything stays in front of the back (on the mattress) and within the shoulders'
 * and hips' width plus a hand (inside the crib's bars); `check_clips.ts` checks both.
 */

import { add, cycle, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { TAU } from './idle';
import { armIK, axisAngle, euler, mul, smooth, toEuler, wrist } from './ik';

/** Seconds per loop: asleep six breaths of 1.5 s, awake four (and whole kicks, waves and a turn of the head). */
const ASLEEP_S = 9;
const AWAKE_S = 6;

interface BabyOptions {
  /** Breath phase (0..1) and depth. */
  breath: number;
  depth: number;
  /** The head's turn (deg, + to the baby's right), nod (+ chin down) and tilt. */
  head: Vec;
  /** Per leg: extension 0 (drawn up) .. 1 (kicked out), and a turn of the thigh out (deg); the hips' flex let go (deg, both legs lower). */
  kickL: number;
  kickR: number;
  outL: number;
  outR: number;
  ease: number;
  /** Per arm: the wrist's offset from its place by the head (m, reference skeleton). */
  armL: Vec;
  armR: Vec;
  /** Per hand: flex at the wrist (deg). */
  flexL: number;
  flexR: number;
}

const REST: BabyOptions = { breath: 0, depth: 1, head: [0, 0, 0], kickL: 0, kickR: 0, outL: 0, outR: 0, ease: 0, armL: [0, 0, 0], armR: [0, 0, 0], flexL: 0, flexR: 0 };

/** One leg: drawn up (`kick` 0) or kicked out towards straight (1), the knee falling out by `out` more degrees. */
function leg(side: 'l' | 'r', kick: number, out: number, ease: number): PoseSpec {
  const s = side === 'l' ? 1 : -1;
  const k = Math.max(0, Math.min(1, kick));
  // Hip: rolled out about the thigh's length (the knee falls out, the shin comes in: the soles
  // towards each other), then flexed (-x raises the thigh forward, here up off the mattress) and
  // spread apart (y, z); less of each as the leg kicks out.
  const roll = axisAngle([0, 1, 0], s * -(34 - 14 * k));
  const thigh = toEuler(mul(euler([-66 + ease + (38 - ease) * k, s * -(22 + out), s * -(12 + out * 0.4)]), roll));
  const calf: Vec = [94 - 64 * k, 0, 0];
  // The foot: pointed a little, the sole turned in.
  const foot: Vec = [22 - 10 * k, s * 6, s * 14];
  return { [`thigh_${side}`]: thigh, [`calf_${side}`]: calf, [`foot_${side}`]: foot } as PoseSpec;
}

/** The lying baby's pose (see the file comment). */
function babyPose(o: BabyOptions): PoseSpec {
  const b = Math.sin(o.breath * TAU) * o.depth;
  // A rounded back: the pelvis tucked (the bottom up off the mattress with the legs drawn up), the
  // chest level; breathing lifts the belly more than the chest.
  const body: PoseSpec = {
    move: [0, 0, 0.012 + 0.0015 * b],
    pelvis: [-12, 0, 0],
    spine_01: [7 - 1.6 * b, 0, 0],
    spine_02: [4 - 0.8 * b, 0, 0],
    spine_03: [1 - 0.4 * b, 0, 0],
    clavicle: [0, 0, -6 - 0.6 * b],
    neck_01: [2, o.head[0] * 0.3, 0],
    Head: [3 + o.head[1] + 0.4 * b, o.head[0] * 0.7, o.head[2]],
  };
  // The hands up by the head, the elbows out and down by the shoulders (on the mattress), the
  // palms in, loosely closed.
  const handL: Vec = [-0.34 + o.armL[0], 1.6 + o.armL[1], 0.06 + o.armL[2]];
  const handR: Vec = [0.34 + o.armR[0], 1.6 + o.armR[1], 0.06 + o.armR[2]];
  return {
    ...body,
    ...leg('l', o.kickL, o.outL, o.ease),
    ...leg('r', o.kickR, o.outR, o.ease),
    ...armIK('l', handL, body, { pole: [-1, -0.7, -0.15], pronate: -30, hand: wrist('l', 20 + o.flexL, 0) }),
    ...armIK('r', handR, body, { pole: [1, -0.7, -0.15], pronate: -30, hand: wrist('r', 20 + o.flexR, 0) }),
  };
}

/** A bump at phase `at` (0..1 of the loop), `width` wide, wrapping round the loop. */
function bump(p: number, at: number, width: number): number {
  let d = Math.abs(p - at);
  d = Math.min(d, 1 - d);
  return d < width ? smooth(1 - d / width) : 0;
}

/** Asleep: still, the head well over to one side, quick shallow breaths, a twitch of the right hand and the left foot. */
function asleep(p: number): PoseSpec {
  const twitch = bump(p, 0.58, 0.018);
  const foot = bump(p, 0.63, 0.014);
  return babyPose({
    ...REST,
    breath: p * 6,
    depth: 1,
    head: [44, 0, 8],
    outL: 18,
    outR: 15,
    ease: 12,
    armL: [0.01, 0.0, 0],
    armR: [-0.02, -0.05, 0.03 + 0.03 * twitch],
    flexR: 18 * twitch,
    kickL: 0.18 * foot,
  });
}

/** Awake: kicking in bursts, one leg then the other; the arms waving; the head turning from side to side. */
function awake(p: number): PoseSpec {
  const a = p * TAU;
  // Two bursts of kicking per loop, the legs out of step.
  const burst = 0.5 - 0.5 * Math.cos(a * 2);
  const kickL = burst * (0.5 - 0.5 * Math.cos(a * 6));
  const kickR = burst * (0.5 - 0.5 * Math.cos(a * 6 + Math.PI * 0.85));
  // The arms wave up off the mattress and about, the right more than the left.
  const waveL = Math.sin(a * 4 + 0.6);
  const waveR = Math.sin(a * 5 + 2.1);
  const base = babyPose({
    ...REST,
    breath: p * 4,
    depth: 1.4,
    head: [26 * Math.sin(a), -3 + 3 * Math.sin(a * 2 + 1), 3 * Math.sin(a)],
    kickL: 0.85 * kickL,
    kickR: 0.85 * kickR,
    outL: 4 * Math.sin(a * 3),
    outR: 4 * Math.sin(a * 3 + 1),
    armL: [-0.02 + 0.07 * Math.sin(a * 3), -0.14 + 0.09 * waveL, 0.12 + 0.1 * (0.5 + 0.5 * Math.sin(a * 4 + 2))],
    armR: [0.02 - 0.08 * Math.sin(a * 2 + 0.4), -0.12 + 0.1 * waveR, 0.14 + 0.12 * (0.5 + 0.5 * Math.sin(a * 5 + 3.5))],
    flexL: 10 * Math.sin(a * 4),
    flexR: 12 * Math.sin(a * 5 + 1),
  });
  // A wriggle of the hips with the kicks.
  return add(base, { pelvis: [0, 3 * Math.sin(a * 6), 2 * Math.sin(a * 2)] });
}

export function babyClips(): ClipDef[] {
  return [cycle('babyLieAsleep', { seconds: ASLEEP_S, loop: true }, asleep), cycle('babyLieAwake', { seconds: AWAKE_S, loop: true }, awake)];
}
