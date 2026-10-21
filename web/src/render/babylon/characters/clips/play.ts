/**
 * Children's play inside things (`STEP_IN` in characters.ts steps them in and out): in the ball pit (over its rim,
 * wading and jumping in the balls, down to sit in them tossing balls up, and up again) and in the play tent
 * (down on hands and knees at its door, crawling in, sitting back on the heels to play on its mat, and crawling
 * back out). One size each, for a child (1.12 m: the share `s` of the reference body, `sizes` in street.ts); the
 * objects' numbers in metres divided by `s` give the clip's.
 */

import { cycle, type ClipDef, type PoseSpec, type Rot, type Vec } from './dsl';
import { arms, body, moved, track, type Body, type Key } from './games';
import { TAU } from './idle';
import { addV, smooth, wrist } from './ik';
import { sizes, type Fit, type Size } from './street';

/** On the knees and the tops of the feet (as the yoga's table): the ankles, the feet's turn, the knees' way. */
const KNEEL: Vec = [0, -0.035, -0.02];
const FLAT: Rot = [170, 0, 0];

// --- the ball pit -------------------------------------------------------------------------------

/**
 * The ball pit (`model.ballPit`): 0.42 m inside, its rim 0.36–0.4 m up, the balls heaped to about 0.43 m, its
 * middle 0.035 m to the front. The child stands in its middle on the bottom (3 cm up), stepping in over the rim
 * (`over`: its distance from the middle, its height).
 */
type PitFit = Fit & { over: { at: number; h: number } };
export const BALL_PIT: Size<PitFit>[] = sizes('ballPit', [0.63], (s) => ({ s, to: 0.035, up: 0.03, over: { at: 0.47, h: 0.4 } }));
const PIT_S = 12;
/** The balls' top over the bottom (m). */
const BALLS = 0.4;

function pitPose(f: PitFit): (p: number) => PoseSpec {
  const s = f.s;
  const top = BALLS / s;
  return (p) => {
    const t = p * PIT_S;
    // Wading (a bounce in the knees), a jump, down to sit in the balls, up again.
    const wade = (t < 2.4 || t > 10.8 ? 1 : 0) * 0.5 * (1 - Math.cos(((t % 0.8) / 0.8) * TAU));
    const at = track<Vec>(
      [
        [0, [0, 0, 0]],
        [2.4, [0, 0, 0]],
        [2.7, [0, -0.2, -0.04], 'out'],
        [2.95, [0, 0.14, 0.02], 'out'],
        [3.2, [0, -0.2, -0.04], 'in'],
        [3.6, [0, 0, 0]],
        [4.1, [0, -0.46, -0.08]],
        [4.6, [0, -0.74, -0.12]],
        [9.4, [0, -0.74, -0.12]],
        [9.9, [0, -0.46, -0.06]],
        [10.5, [0, 0, 0]],
      ],
      t,
    );
    const hip = track<number>([[0, 4], [2.4, 4], [2.7, 26], [2.95, 0], [3.2, 26], [3.6, 4], [4.1, 34], [4.6, -4], [9.4, -4], [9.9, 34], [10.5, 4]], t);
    // The feet: on the bottom, drawn up in the jump, out in front sitting (the knees up).
    const feet = moved([[0, [0, 0, 0]], [2.7, [0, 0, 0]], [2.95, [0, 0.16, -0.06], 'out'], [3.2, [0, 0, 0], 'in'], [4.1, [0, 0, 0.06]], [4.6, [0, 0, 0.42]], [9.4, [0, 0, 0.42]], [9.9, [0, 0, 0.08]], [10.5, [0, 0, 0]]], t, [0, 0.05, 0]);
    const knee = track<number>([[0, 0], [4.1, 0], [4.6, 1], [9.4, 1], [9.9, 0]], t);
    // Sitting, a ball scooped up and tossed: the left at 5.0, the right at 6.2, … (1.2 s each).
    const toss = t > 4.8 && t < 9.6 ? ((t - 4.8) / 1.2) % 1 : -1;
    const leftToss = t > 4.8 && t < 9.6 && Math.floor((t - 4.8) / 1.2) % 2 === 0;
    const up = toss < 0 ? 0 : Math.sin(Math.PI * smooth(toss));
    const look = track<number>([[0, 10], [2.4, 10], [2.95, -10], [3.6, 10], [4.6, 0], [9.4, 0], [10.5, 10]], t);
    const b: Body = {
      at: addV(at, [0, -0.035 * wade, 0]),
      hip: hip + 6 * wade,
      bend: 6,
      head: [look - 30 * up, 0, 0],
      breath: t / 4,
      l: addV(feet, [0, 0, 0]),
      r: addV(feet, [0, 0, 0]),
      kl: [-0.12 - 0.3 * knee, knee * 0.8, 1],
      kr: [0.12 + 0.3 * knee, knee * 0.8, 1],
    };
    const pose = body(b);
    // The hands through the balls standing (swishing them about), over the head in the jump, scooping and tossing sitting.
    const swish = 0.08 * Math.sin((t / 1.6) * TAU);
    const sitting = smooth(Math.max(0, Math.min(1, (t - 4.1) / 0.5, (9.9 - t) / 0.5)));
    const jump = Math.max(0, 1 - Math.abs(t - 2.95) / 0.3);
    const hand = (sx: number, tossing: boolean): Vec => {
      const wading: Vec = [sx * 0.3 + swish, top + 0.04, 0.22];
      const air: Vec = [sx * 0.28, 1.9, 0.12];
      const low: Vec = [sx * 0.26, top - 0.02, 0.36];
      const high: Vec = [sx * 0.14, top + 0.75, 0.24];
      const seat: Vec = tossing ? [low[0] + (high[0] - low[0]) * up, low[1] + (high[1] - low[1]) * up, low[2] + (high[2] - low[2]) * up] : low;
      const stood: Vec = [wading[0] + (air[0] - wading[0]) * jump, wading[1] + (air[1] - wading[1]) * jump, wading[2] + (air[2] - wading[2]) * jump];
      return [stood[0] + (seat[0] - stood[0]) * sitting, stood[1] + (seat[1] - stood[1]) * sitting, stood[2] + (seat[2] - stood[2]) * sitting];
    };
    return arms(
      pose,
      { at: hand(-1, leftToss), palm: true, pole: [-0.8, -0.5, -0.3], pronate: 40, hand: wrist('l', 10, 0) },
      { at: hand(1, !leftToss && toss >= 0), palm: true, pole: [0.8, -0.5, -0.3], pronate: 40, hand: wrist('r', 10, 0) },
    );
  };
}

// --- the play tent ------------------------------------------------------------------------------

/**
 * The play tent (`model.playTent`): a six-sided teepee 0.46 m out to its corners, its front face 0.4 m from the
 * middle with the door in it (0.32 m wide at the foot, 0.56 m high), a mat on its floor (3 cm). The child kneels
 * down at the door (the feet `to` m out), crawls in `crawl` m (the hips to the middle) and sits back on the heels.
 */
type TentFit = Fit & { crawl: number };
export const PLAY_TENT: Size<TentFit>[] = sizes('tentPlay', [0.63], (s) => ({ s, to: 0.66, crawl: 0.64 }));
const TENT_S = 24;
/** On hands and knees (the yoga's table, a child's): the pelvis, its pitch, the hands ahead. */
const TABLE = { at: [0, -0.48, 0.32] as Vec, hip: 85, hands: 0.8 };
const SEATED = { at: [0, -0.62, -0.02] as Vec, hip: -2 };

/** In steps: a limb's share of the way from `t0` to `t1` in `n` steps (each in its `half` of a step), and its lift (0..1). */
function gait(t: number, t0: number, t1: number, n: number, half: number): [number, number] {
  if (t <= t0) return [0, 0];
  if (t >= t1) return [1, 0];
  const c = ((t - t0) / (t1 - t0)) * n;
  const i = Math.floor(c);
  const k = Math.max(0, Math.min(1, (c - i - half * 0.5) / 0.5));
  return [(i + smooth(k)) / n, Math.sin(Math.PI * k)];
}

function tentPose(f: TentFit): (p: number) => PoseSpec {
  const d = f.crawl / f.s;
  // In, then out (the same steps backwards).
  const IN: [number, number] = [1.8, 4.8];
  const OUT: [number, number] = [20, 23];
  return (p) => {
    const t = p * TENT_S;
    const going = t < 12 ? IN : OUT;
    const way = (k: number) => (t < 12 ? k : 1 - k);
    const steps = 4;
    // The body's way in (or out), even; each limb's in its steps, the left hand with the right knee.
    const fk = Math.max(0, Math.min(1, (t - going[0]) / (going[1] - going[0])));
    const reach = way(t < going[0] ? 0 : fk) * d;
    const limb = (half: number): [number, number] => {
      const [k, lift] = gait(t, going[0], going[1], steps, half);
      return [way(k) * d, lift];
    };
    const [hl, liftHl] = limb(0);
    const [hr, liftHr] = limb(1);
    // (The knees slide along the soft mat, the shins on it.)
    const [kr] = limb(0);
    const [kl] = limb(1);
    // Down to hands and knees, back onto the heels inside, forward again, up outside.
    const k4 = (keys: readonly Key<number>[]) => track<number>(keys, t);
    const down = k4([[0, 0], [0.7, 0.5], [1.6, 1], [19.9, 1], [23.2, 1], [23.6, 0.5], [24, 0]]);
    const sat = k4([[0, 0], [5, 0], [5.8, 1], [19, 1], [19.8, 0]]);
    const stood: Vec = [0, 0, 0];
    const crouched: Vec = [0, -0.42, -0.06];
    const table: Vec = addV(TABLE.at, [0, 0, reach]);
    const seated: Vec = addV(SEATED.at, [0, 0, d]);
    const mix = (a: Vec, b: Vec, u: number): Vec => [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u, a[2] + (b[2] - a[2]) * u];
    const pelvis = mix(down < 0.5 ? mix(stood, crouched, down * 2) : mix(crouched, table, (down - 0.5) * 2), seated, sat);
    const hip = (down < 0.5 ? 30 * down * 2 : 30 + (TABLE.hip - 30) * (down - 0.5) * 2) * (1 - sat) + SEATED.hip * sat;
    // Playing, sat back: the hands busy on the mat in front, the head down to them; a look up now and then.
    const play = sat * (0.5 + 0.5 * Math.sin((t / 1.4) * TAU));
    const lookUp = sat * Math.max(0, 1 - Math.abs(t - 12.5) / 1.2);
    // From the feet (crouched) onto the knees and the tops of the feet, the knees then stepping in (or out) in turn.
    const kneel = smooth(Math.max(0, Math.min(1, (down - 0.5) * 2)));
    const ankle = (z: number): Vec => mix([0, 0, 0], addV(KNEEL, [0, 0, z]), kneel);
    const b: Body = {
      at: pelvis,
      hip,
      bend: -6 * (1 - sat) + 8 * sat,
      head: [-6 * (1 - sat) + (22 - 34 * lookUp) * sat, 10 * Math.sin((t / 5) * TAU) * sat, 0],
      breath: t / 4,
      l: ankle(kl),
      r: ankle(kr),
      fl: kneel > 0.5 ? FLAT : [0, -4, 0],
      fr: kneel > 0.5 ? FLAT : [0, 4, 0],
      kl: [0, -0.3 * kneel, 1],
      kr: [0, -0.3 * kneel, 1],
    };
    const pose = body(b);
    const onFloor = (z: number, lift: number, sx: number): Vec => [sx * 0.2, 0.03 + 0.05 * lift, TABLE.hands + z];
    const toys = (sx: number): Vec => [sx * (0.12 + 0.05 * play), 0.12 + 0.06 * (sx < 0 ? play : 1 - play), d + 0.42];
    const hand = (sx: number, z: number, lift: number): Vec => {
      const floor = onFloor(z, lift, sx);
      const knees: Vec = [sx * 0.14, 0.62, 0.3];
      const at = down < 1 ? mix(knees, floor, Math.max(0, down * 2 - 1)) : floor;
      return mix(at, toys(sx), sat);
    };
    return arms(
      pose,
      { at: hand(-1, hl, liftHl), palm: true, pole: [-0.8, -0.5, -0.3], pronate: 85, hand: wrist('l', -50 * (1 - sat), 0) },
      { at: hand(1, hr, liftHr), palm: true, pole: [0.8, -0.5, -0.3], pronate: 85, hand: wrist('r', -50 * (1 - sat), 0) },
    );
  };
}

export function playClips(): ClipDef[] {
  return [
    ...BALL_PIT.map(({ clip, fit }) => cycle(clip, { seconds: PIT_S, loop: true, hands: 'open' }, pitPose(fit))),
    ...PLAY_TENT.map(({ clip, fit }) => cycle(clip, { seconds: TENT_S, loop: true, hands: 'open' }, tentPose(fit))),
  ];
}
