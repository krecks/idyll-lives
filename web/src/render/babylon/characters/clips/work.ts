/**
 * Standing work: reaching to use something, picking up from the floor, hands busy at a counter
 * (`spell`), pushing a mop or vacuum, leaning on a rail (telescope), and (phase 2) chopping,
 * stirring, flipping and carrying.
 *
 * Counters are 0.9 m high with their front edge ~0.5 m ahead of where a resident stands (the
 * centre of the tile in front): the weight goes forward over planted feet and the back leans in so
 * the hands reach the top.
 */

import { clip, cycle, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { bump, hangArms, stand, TAU } from './idle';
import { armIK, smooth, wrist, type ArmOptions } from './ik';

/** Hands on a counter top: height and reach (reference skeleton, model space). */
const COUNTER_Y = 1.0;
const COUNTER_Z = 0.5;

type Hand = ArmOptions & { at: Vec };

/** A body pose with both hands placed. */
function hands(body: PoseSpec, l: Hand | null, r: Hand | null): PoseSpec {
  return { ...body, ...(l ? armIK('l', l.at, body, l) : {}), ...(r ? armIK('r', r.at, body, r) : {}) };
}

/** Leaning in at the counter: weight forward over the feet, the back bent `lean` degrees. */
function atCounter(p: number, lean = 22, extra: Partial<Parameters<typeof stand>[0]> = {}): PoseSpec {
  return stand({ breath: p, fwd: 0.07, sink: 0.012, lean, shift: 0.2 * Math.sin(p * TAU), head: [8, 0, 0], ...extra });
}

const L = (at: Vec, o: ArmOptions = {}): Hand => ({ at, pole: [-0.5, -1, -0.4], pronate: 70, hand: wrist('l', -5, 0), ...o });
const R = (at: Vec, o: ArmOptions = {}): Hand => ({ at, pole: [0.5, -1, -0.4], pronate: 70, hand: wrist('r', -5, 0), ...o });

/** Hands busy at a counter: each hand works small circles in turn, the head follows the work. */
function counterPose(p: number): PoseSpec {
  const a = p * TAU * 3;
  const body = atCounter(p, 22, { twist: 3 * Math.sin(p * TAU), head: [10, 6 * Math.sin(p * TAU), 0] });
  return hands(
    body,
    L([-0.14 + 0.03 * Math.cos(a), COUNTER_Y + 0.015 * Math.max(0, Math.sin(a)), COUNTER_Z + 0.03 * Math.sin(a)], { pronate: 70 + 15 * Math.sin(a) }),
    R([0.14 + 0.03 * Math.cos(a + 2.2), COUNTER_Y + 0.015 * Math.max(0, Math.sin(a + 2.2)), COUNTER_Z + 0.03 * Math.sin(a + 2.2)], { pronate: 70 + 15 * Math.sin(a + 2.2) }),
  );
}

/** Chopping (phase 2): the left hand holds the food, the right brings the knife down in quick strokes. */
function chopPose(p: number): PoseSpec {
  const s = (p * 4) % 1;
  // A fast fall and a slower lift (anticipation at the top), the knife creeping left along the food.
  const up = s < 0.35 ? 1 - smooth(s / 0.35) : smooth((s - 0.35) / 0.65);
  const along = 0.03 * Math.sin(p * TAU);
  const body = atCounter(p, 23, { head: [12, -5, 0], twist: -3 });
  return hands(body, L([-0.06, COUNTER_Y + 0.02, COUNTER_Z + 0.02], { pronate: 85, hand: wrist('l', 15, 0) }), R([0.1 + along, COUNTER_Y + 0.02 + 0.09 * up, COUNTER_Z - 0.01], { pronate: 20, hand: wrist('r', 10 - 25 * up, 0) }));
}

/** Stirring (phase 2): the right hand circles over the pot, the left steadies its handle. */
function stirPose(p: number): PoseSpec {
  const a = p * TAU * 2;
  const body = atCounter(p, 20, { head: [10, 3 * Math.sin(a), 0], shift: 0.3 * Math.sin(a) });
  return hands(body, L([-0.2, COUNTER_Y + 0.05, COUNTER_Z - 0.04], { pronate: 30, hand: wrist('l', 30, 0) }), R([0.04 + 0.07 * Math.cos(a), COUNTER_Y + 0.13, COUNTER_Z + 0.06 * Math.sin(a)], { pronate: 30, hand: wrist('r', 30, 0) }));
}

/** Flipping (phase 2): the right hand tosses the pan up with a wrist flick, catches, settles; the left waits by the stove. */
function flipPose(p: number): PoseSpec {
  const toss = bump(p, 0.35, 0.65, 0.12);
  const flick = bump(p, 0.42, 0.56, 0.05);
  const body = atCounter(p, 20, { head: [10 - 10 * toss, 0, 0], lean: 20 - 4 * toss, sink: 0.01 + 0.02 * bump(p, 0.25, 0.42, 0.08) });
  return hands(
    body,
    L([-0.2, COUNTER_Y + 0.03, COUNTER_Z - 0.05], { pronate: 75 }),
    R([0.15, COUNTER_Y + 0.06 + 0.12 * toss, COUNTER_Z - 0.04 + 0.04 * toss], { pronate: 10, hand: wrist('r', 10 - 30 * flick, 0) }),
  );
}

/** Carrying something in both hands in front (an upper-body layer, phase 2): a plate or a pot at the waist. */
function carryPose(p: number): PoseSpec {
  const bob = 0.008 * Math.cos(p * TAU * 2);
  const body = stand({ breath: p, lean: -2, head: [6, 0, 0] });
  return hands(body, L([-0.16, 1.05 + bob, 0.27], { pole: [-0.5, -1, -0.3], pronate: -10, hand: wrist('l', 20, 0) }), R([0.16, 1.05 + bob, 0.27], { pole: [0.5, -1, -0.3], pronate: -10, hand: wrist('r', 20, 0) }));
}

/** Carrying something in the right hand (an upper-body layer, phase 2): forearm forward, the left arm swings at the walk's pace. */
function carryOnePose(p: number): PoseSpec {
  const swing = -Math.cos(TAU * (p - 0.05));
  const body = stand({ breath: p });
  const at: Vec = [0.2, 1.0 + 0.006 * Math.cos(p * TAU * 2), 0.26];
  return { ...body, ...hangArms(3 + 14 * swing, 7, 16 + 12 * Math.max(0, swing)), ...armIK('r', at, body, { pole: [0.4, -1, -0.5], pronate: 20, hand: wrist('r', 10, 0) }) };
}

/** Mopping or vacuuming: a handle held at the right side, pushed out and drawn back twice, the weight rocking with it. */
function pushPose(p: number): PoseSpec {
  const s = Math.sin(p * TAU * 2 - 0.5);
  const out = 0.5 + 0.5 * s;
  const body = stand({ breath: p, lean: 10 + 8 * out, fwd: 0.02 + 0.05 * out, sink: 0.03, shift: 0.3 - 0.2 * out, twist: -10 + 4 * s, head: [16, 8, 0], feet: [0, 0, 0.12], footR: [0, 0, -0.1] });
  return hands(
    body,
    L([-0.02 + 0.03 * out, 1.18 - 0.05 * out, 0.22 + 0.12 * out], { pole: [-0.6, -1, -0.2], pronate: 80, hand: wrist('l', 25, 0) }),
    R([0.17, 0.98 - 0.08 * out, 0.26 + 0.2 * out], { pole: [0.6, -1, -0.4], pronate: 0, hand: wrist('r', 25, 0) }),
  );
}

/** Leaning on a rail (the telescope): forearms on it, the weight on one leg, looking out. */
function railPose(p: number): PoseSpec {
  const body = stand({ breath: p, lean: 22, fwd: 0.06, sink: 0.02, shift: -0.7 + 0.1 * Math.sin(p * TAU), head: [-16, 8 * Math.sin(p * TAU + 1), 2], footR: [0.02, 0, -0.08] });
  return hands(body, L([-0.13, 1.08, 0.42], { pole: [-0.6, -1, 0], pronate: 80, hand: wrist('l', 10, 0) }), R([0.1, 1.08, 0.44], { pole: [0.6, -1, 0], pronate: 80, hand: wrist('r', 10, 0) }));
}

/** Squatting to the floor with the feet planted: hips back and down, chest forward, hands low. */
function squat(depth: number, reach: number, t: number): PoseSpec {
  const body = stand({ breath: t, sink: depth, fwd: -0.14 * depth / 0.35, lean: 45 * Math.min(1, depth / 0.3), head: [-12 * Math.min(1, depth / 0.3), 0, 0] });
  const y = 0.9 - (0.9 - 0.16) * reach;
  return hands(body, L([-0.13, y, 0.18 + 0.18 * reach], { pole: [-0.5, -0.4, -0.9], pronate: 20, hand: wrist('l', 10, 0) }), R([0.13, y, 0.18 + 0.18 * reach], { pole: [0.5, -0.4, -0.9], pronate: 20, hand: wrist('r', 10, 0) }));
}

export function workClips(): ClipDef[] {
  const rest = (): PoseSpec => ({ ...stand(), ...hangArms() });
  const reach = (t: number, rx: number, ry: number, rz: number, lean: number, left: Vec | null): PoseSpec => {
    const body = atCounter(t, lean, { head: [12, -4, 0], twist: -4 });
    return hands(body, left ? L(left) : null, R([rx, ry, rz], { pronate: 60 }));
  };
  return [
    // Reaching to use something on a counter: a small draw back, the reach, two hands at work, back.
    clip('interact', { seconds: 60 / 30, loop: false }, [
      [0, rest],
      [0.1, () => ({ ...stand({ lean: -2, head: [4, 0, 0] }), ...hangArms(-2, 7, 20) }), 'smooth'],
      [0.3, () => reach(0.3, 0.14, COUNTER_Y + 0.04, COUNTER_Z + 0.04, 24, null), 'smooth'],
      [0.42, () => reach(0.42, 0.12, COUNTER_Y + 0.01, COUNTER_Z + 0.02, 23, [-0.12, COUNTER_Y + 0.03, COUNTER_Z]), 'smooth'],
      [0.55, () => reach(0.55, 0.06, COUNTER_Y + 0.03, COUNTER_Z + 0.05, 24, [-0.08, COUNTER_Y + 0.01, COUNTER_Z + 0.03]), 'smooth'],
      [0.68, () => reach(0.68, 0.1, COUNTER_Y + 0.05, COUNTER_Z, 22, [-0.13, COUNTER_Y + 0.02, COUNTER_Z + 0.01]), 'smooth'],
      [0.85, () => ({ ...stand({ lean: 3, head: [6, 0, 0] }), ...hangArms(8, 7, 30) }), 'smooth'],
      [1, rest, 'smooth'],
    ]),
    // Squat and grab: down over the planted feet, both hands to the floor, up again holding it.
    clip('pickUp', { seconds: 26 / 30, loop: false }, [
      [0, rest],
      [0.14, () => squat(0.04, 0.05, 0.14), 'smooth'],
      [0.46, () => squat(0.36, 1, 0.46), 'smooth'],
      [0.56, () => squat(0.37, 1.02, 0.56), 'smooth'],
      [0.85, () => squat(0.04, 0.12, 0.85), 'out'],
      [1, () => {
        const body = stand();
        return hands(body, L([-0.15, 1.02, 0.25], { pronate: 0 }), R([0.15, 1.02, 0.25], { pronate: 0 }));
      }, 'smooth'],
    ]),
    cycle('spell', { seconds: 63 / 30, loop: true }, counterPose),
    cycle('push', { seconds: 80 / 30, loop: true }, pushPose),
    cycle('rail', { seconds: 2.5, loop: true }, railPose),
    cycle('chop', { seconds: 1.6, loop: true }, chopPose),
    cycle('stir', { seconds: 2, loop: true }, stirPose),
    cycle('flip', { seconds: 2, loop: true }, flipPose),
    cycle('carry', { seconds: 1.1, loop: true }, carryPose),
    cycle('carryOne', { seconds: 1.1, loop: true }, carryOnePose),
  ];
}
