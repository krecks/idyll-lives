/**
 * Gardening: watering, harvesting, picking herbs from a planter and fruit from a tree, planting, kneeling to weed, crouching to smell flowers. Feet
 * stay planted (leg IK); hands are placed by arm IK at the plants and the soil in front.
 */

import { clip, cycle, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { bump, hangArms, stand, TAU } from './idle';
import { armIK, footIK, wrist, type ArmOptions } from './ik';

type Hand = ArmOptions & { at: Vec };

function hands(body: PoseSpec, l: Hand | null, r: Hand | null): PoseSpec {
  return { ...body, ...(l ? armIK('l', l.at, body, { pole: [-0.5, -0.8, -0.5], ...l }) : {}), ...(r ? armIK('r', r.at, body, { pole: [0.5, -0.8, -0.5], ...r }) : {}) };
}

/**
 * Down on the haunches: the pelvis `depth` m down (and back to stay over the feet), the heels flat
 * as long as the shins allow, the back leaning `lean` degrees.
 */
function crouchBody(depth: number, lean: number, t: number, head: Vec = [0, 0, 0], shift = 0): PoseSpec {
  const k = Math.min(1, depth / 0.45);
  return stand({ breath: t, sink: depth, fwd: -0.1 * k, lean, shift, head, feet: [-0.03 * k, 0, 0.02 * k] });
}

/** Watering: the can held out in the right hand, tipped to pour and swept slowly along the bed, the left arm loose. */
function waterPose(p: number): PoseSpec {
  const sweep = Math.sin(p * TAU);
  const pour = 0.5 + 0.5 * Math.cos(p * TAU * 2);
  const body = stand({ breath: p * 2, lean: 10, shift: -0.3 + 0.3 * sweep, twist: -8 * sweep, head: [24, -10 * sweep, 0] });
  return {
    ...body,
    ...hangArms(6, 9, 18),
    ...armIK('r', [0.2 - 0.07 * sweep, 1.0 + 0.03 * pour, 0.4], body, { pole: [0.6, -1, -0.3], pronate: 0, hand: wrist('r', 10 + 25 * pour, 0) }),
  };
}

/** Harvesting: bent over the plant, the right hand picks with a twist, the left holds the gathered crop at the hip. */
function harvestPose(p: number): PoseSpec {
  const pick = bump(p, 0.08, 0.62, 0.16);
  const twist = bump(p, 0.3, 0.46, 0.06);
  const body = stand({ breath: p, lean: 22 + 10 * pick, sink: 0.04 + 0.04 * pick, fwd: -0.02, shift: 0.3 - 0.2 * pick, twist: 6 * pick - 4, head: [14 + 6 * pick, -6 * pick, 0] });
  const reach: Vec = [0.16 - 0.05 * pick, 0.8 - 0.12 * pick + 0.03 * twist, 0.3 + 0.16 * pick];
  return hands(body, { at: [-0.12, 0.96, 0.2], pronate: -30, hand: wrist('l', 30, 0) }, { at: reach, pronate: 30 + 50 * twist, hand: wrist('r', 20 - 10 * pick + 25 * twist, 0) });
}

/**
 * Picking herbs from a waist-high planter (soil at 0.73 m, the plants' tops about 0.85 m, half a metre
 * ahead): leaning in over it, the left hand steadying the plant, the right pinching off a sprig and
 * drawing it up, twice a loop.
 */
function pickPose(p: number): PoseSpec {
  const n = (p * 2) % 1;
  const pinch = bump(n, 0.08, 0.5, 0.14);
  const draw = bump(n, 0.42, 0.88, 0.14);
  const body = stand({ breath: p, lean: 24 + 4 * pinch, sink: 0.015, fwd: 0.04, shift: -0.25, twist: 5, head: [18 + 4 * pinch, 6, 0] });
  return hands(
    body,
    { at: [-0.2, 0.88, 0.46], pronate: 70, hand: wrist('l', 5, 0) },
    { at: [-0.08 + 0.02 * draw, 0.86 - 0.03 * pinch + 0.09 * draw, 0.5 - 0.06 * draw], pronate: 40 + 30 * pinch, hand: wrist('r', 10 + 15 * draw, 0) },
  );
}

/**
 * Picking fruit from a tree (the apple tree's crown, a potted lemon tree's): standing under it, looking up,
 * the right hand reaching up into the branches, twisting a fruit off and bringing it down to the left hand
 * holding the gathered ones at the hip, twice a loop.
 */
function pickHighPose(p: number): PoseSpec {
  const n = (p * 2) % 1;
  const reach = bump(n, 0.04, 0.5, 0.16);
  const twist = bump(n, 0.36, 0.5, 0.05);
  const body = stand({ breath: p, lean: -4 * reach, shift: 0.15, twist: 4, head: [-14 - 18 * reach, 4, 0] });
  return hands(
    body,
    { at: [-0.12, 0.98, 0.22], pronate: -30, hand: wrist('l', 30, 0) },
    { at: [0.14, 1.18 + 0.62 * reach, 0.3 + 0.1 * reach], pronate: 40 + 60 * twist, hand: wrist('r', 10 + 20 * twist, 0) },
  );
}

/** Kneeling on both knees, sitting back a little; hands pull weeds in turn and toss them aside. */
export function kneelBody(t: number, lean: number, twist: number, head: Vec): PoseSpec {
  const body: PoseSpec = {
    move: [0, -0.455 + 0.004 * Math.sin(t * TAU * 3), -0.01],
    pelvis: [8, 0, 0],
    spine_01: [lean * 0.4, twist * 0.25, 0],
    spine_02: [lean * 0.35, twist * 0.35, 0],
    spine_03: [lean * 0.25, twist * 0.4, 0],
    neck_01: [head[0] * 0.3 - lean * 0.2, head[1] * 0.35, 0],
    Head: [head[0] * 0.7 - lean * 0.3, head[1] * 0.65 - twist * 0.5, head[2]],
  };
  // Knees on the ground ahead of the hips, shins flat behind, the feet on their tops.
  const leg = (side: 'l' | 'r') => footIK(side, [side === 'l' ? -0.02 : 0.02, -0.03, -0.27], body, { pole: [0, -1, 0.6], foot: [168, side === 'l' ? -8 : 8, 0] });
  return { ...body, ...leg('l'), ...leg('r') };
}

function kneelPose(p: number): PoseSpec {
  // Three pulls per loop: left, right, both; each reaches, grips, pulls back and tosses to the side.
  const n = (p * 3) % 1;
  const which = Math.floor(p * 3);
  const reach = bump(n, 0.0, 0.55, 0.2);
  const yank = bump(n, 0.4, 0.75, 0.12);
  const toss = bump(n, 0.62, 1.0, 0.15);
  const side = which === 0 ? -1 : which === 1 ? 1 : 0;
  const body = kneelBody(p, 40 + 12 * reach - 10 * yank, 8 * side * reach + 14 * side * toss, [8 + 4 * reach, 10 * side * toss, 0]);
  const soil = (sx: number): Vec => [0.12 * sx + 0.05 * side, 0.1 + 0.14 * yank, 0.44 + 0.06 * reach - 0.12 * yank];
  const work = (s: 'l' | 'r') => (side === 0 || (s === 'l' ? side < 0 : side > 0) ? 1 : 0.25);
  const at = (s: 'l' | 'r'): Vec => {
    const sx = s === 'l' ? -1 : 1;
    const w = work(s);
    const restP: Vec = [0.17 * sx, 0.3, 0.32];
    const goal = soil(sx);
    const tossP: Vec = [0.35 * sx, 0.3, 0.2];
    const a: Vec = [restP[0] + (goal[0] - restP[0]) * w, restP[1] + (goal[1] - restP[1]) * w, restP[2] + (goal[2] - restP[2]) * w];
    const tw = toss * w * (side === 0 || (sx < 0) === (side < 0) ? 1 : 0);
    return [a[0] + (tossP[0] - a[0]) * tw, a[1] + (tossP[1] - a[1]) * tw, a[2] + (tossP[2] - a[2]) * tw];
  };
  return hands(body, { at: at('l'), pronate: 60, hand: wrist('l', 10 + 20 * yank, 0) }, { at: at('r'), pronate: 60, hand: wrist('r', 10 + 20 * yank, 0) });
}

/** Crouching low to smell a flower: on the haunches, forearms on the knees, the head leaning in for slow deep sniffs. */
function crouchPose(p: number): PoseSpec {
  const sniff = bump(p, 0.2, 0.6, 0.15);
  const body = crouchBody(0.42, 30 + 8 * sniff, p, [10 + 10 * sniff, 4 * Math.sin(p * TAU), 6 * sniff]);
  return hands(body, { at: [-0.15, 0.6, 0.36], pronate: 40, hand: wrist('l', 20, 0) }, { at: [0.12, 0.62 + 0.04 * sniff, 0.4 + 0.04 * sniff], pronate: 30, hand: wrist('r', 20, 0) });
}

/** Planting: crouch, dig two scoops with the right hand, set the seedling in with both hands, press the soil, stand. */
function plantKey(t: number, depth: number, l: Vec, r: Vec, lean = 34, head: Vec = [16, 0, 0]): PoseSpec {
  const body = crouchBody(depth, lean, t, head);
  return hands(body, { at: l, pronate: 70, hand: wrist('l', 10, 0) }, { at: r, pronate: 70, hand: wrist('r', 20, 0) });
}

export function gardenClips(): ClipDef[] {
  const rest = (): PoseSpec => ({ ...stand(), ...hangArms() });
  const knee = (sx: number): Vec => [0.15 * sx, 0.55, 0.3];
  return [
    cycle('water', { seconds: 114 / 30, loop: true }, waterPose),
    cycle('harvest', { seconds: 2.5, loop: true }, harvestPose),
    cycle('pick', { seconds: 3, loop: true }, pickPose),
    cycle('pickHigh', { seconds: 3.2, loop: true }, pickHighPose),
    clip('plant', { seconds: 83 / 30, loop: false }, [
      [0, rest],
      [0.14, () => plantKey(0.14, 0.42, knee(-1), knee(1), 30), 'smooth'],
      [0.24, () => plantKey(0.24, 0.44, knee(-1), [0.1, 0.1, 0.42]), 'smooth'],
      [0.32, () => plantKey(0.32, 0.44, knee(-1), [0.12, 0.2, 0.32], 32), 'smooth'],
      [0.4, () => plantKey(0.4, 0.44, knee(-1), [0.08, 0.1, 0.43]), 'smooth'],
      [0.48, () => plantKey(0.48, 0.44, knee(-1), [0.13, 0.22, 0.3], 32), 'smooth'],
      [0.58, () => plantKey(0.58, 0.45, [-0.05, 0.14, 0.42], [0.05, 0.14, 0.42], 38, [20, 0, 0]), 'smooth'],
      [0.68, () => plantKey(0.68, 0.46, [-0.07, 0.09, 0.4], [0.07, 0.09, 0.4], 40, [22, 0, 0]), 'smooth'],
      [0.76, () => plantKey(0.76, 0.45, [-0.07, 0.12, 0.4], [0.07, 0.12, 0.4], 38, [18, 0, 0]), 'smooth'],
      [0.84, () => plantKey(0.84, 0.42, knee(-1), knee(1), 28, [6, 0, 0]), 'smooth'],
      [1, rest, 'smooth'],
    ]),
    cycle('kneel', { seconds: 156 / 30, loop: true }, kneelPose),
    cycle('crouch', { seconds: 88 / 30, loop: true }, crouchPose),
  ];
}
