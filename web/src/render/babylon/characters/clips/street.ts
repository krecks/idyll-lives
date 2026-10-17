/**
 * Things out in the town used up against them: bent over a drinking fountain's spout. Each comes in several sizes, named by the height (cm) it is fitted to (`STEP_IN` in
 * characters.ts picks the one for a resident's height): the object's heights and depths are fixed, so each
 * is fitted to a body of its share `s` of the reference skeleton's size (1.77 m), standing `to` m from the
 * object's middle (the object's numbers, in metres, divided by `s` give the clip's).
 */

import { cycle, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { breathe, bump, TAU } from './idle';
import { armIK, footIK, pointOn, smooth, wrist } from './ik';

/** Where a user stands and how big they are: their share of the reference body's size, the feet's distance from the object's middle, the ground's height there (m). */
export interface Fit {
  s: number;
  to: number;
  up?: number;
}
/** A size of a clip: its name, the shortest resident it is for (m: from half way to the next size down), and its fit. */
export interface Size<F extends Fit = Fit> {
  clip: string;
  min: number;
  fit: F;
}
const REF_HEIGHT = 1.77;
/** The sizes of `name` for these shares of the reference body's size (the tallest first), each fitted by `fit`. */
function sizes<F extends Fit>(name: string, shares: readonly number[], fit: (s: number) => F): Size<F>[] {
  return shares.map((s, i) => ({
    clip: `${name}${Math.round(s * REF_HEIGHT * 100)}`,
    min: i < shares.length - 1 ? ((s + shares[i + 1]) / 2) * REF_HEIGHT : 0,
    fit: fit(s),
  }));
}

/**
 * The drinking fountain (street furniture, `model.drinkingFountain`): the water arcs from the spout to its
 * highest, 1.13 m, over the bowl's middle and falls to the bowl at 0.1 m to the front; the bowl's rim is
 * 1.0 m up, 0.22 m out. Five sizes, 1.81 m down to 1.12 m: the mouth `mouth` m up and to the front of the
 * middle, over the spout or (the smaller two, on tiptoe by `tip`: m, the clip's) at the falling end, the
 * smallest (a child) up on the slab's front edge (8 cm); the feet nearer the smaller they are.
 */
type FountainFit = Fit & { mouth: [y: number, z: number]; tip?: number };
export const FOUNTAIN: Size<FountainFit>[] = sizes('fountainDrink', [1.02, 0.92, 0.83, 0.73, 0.63], (s) =>
  s < 0.68
    ? { s, to: 0.27, up: 0.08, mouth: [1.04, 0.09], tip: 0.035 }
    : s < 0.78
      ? { s, to: 0.29, mouth: [1.08, 0.07], tip: 0.035 }
      : { s, to: 0.27 + 0.44 * (s - 0.63), mouth: [1.13, 0.04] },
);
/** The mouth from the head's joint (m, reference skeleton at rest). */
const MOUTH: Vec = [0, 0.04, 0.1];
/** The bowl's rim where the hands rest: out to either side, up, to the front of the middle (m). */
const RIM: Vec = [0.19, 0.99, 0.06];

/**
 * Bent at the hips and the back by `bend` degrees, the hips `back` m forward of the feet (back from them
 * when bent over; a child leans in over them, holding on to the rim), on tiptoe by `tip` m; the head lifted
 * to look at the water.
 */
function bentBody(bend: number, back: number, tip: number, p: number, turn: number): PoseSpec {
  const b = breathe(p, 0.6);
  const hip = 0.45 * bend;
  const lean = bend - hip;
  // The face kept turned down towards the water (about 30 degrees past level), a little to one side.
  const head: Vec = [32 - 0.85 * bend, turn, -0.4 * turn];
  const body: PoseSpec = {
    move: [0, tip - 0.015 * smooth(bend / 60) + (b.move?.[1] ?? 0), back],
    pelvis: [hip, 0, 0],
    spine_01: [lean * 0.35, 0, 0],
    spine_02: [lean * 0.35 + (b.spine_02?.[0] ?? 0), 0, 0],
    spine_03: [lean * 0.3 + (b.spine_03?.[0] ?? 0), 0, 0],
    clavicle_l: b.clavicle,
    clavicle_r: [0, 0, -(b.clavicle?.[2] ?? 0)],
    neck_01: [head[0] * 0.4, head[1] * 0.35, head[2] * 0.3],
    Head: [head[0] * 0.6, head[1] * 0.65, head[2] * 0.7],
  };
  // On tiptoe the heels come up about the balls of the feet (0.17 m ahead of the ankles).
  const toe = Math.asin(Math.min(0.9, tip / 0.17));
  const ankle: Vec = [0, tip, -0.17 * (1 - Math.cos(toe))];
  const deg = (toe * 180) / Math.PI;
  return { ...body, ...footIK('l', ankle, body, { foot: [deg, -4, 0] }), ...footIK('r', ankle, body, { foot: [deg, 4, 0] }) };
}

/**
 * Drinking from the fountain's arc: bent over the bowl with the mouth at the water (found while baking: the
 * bend for the height, the hips back or forward for the reach), both hands on the rim; a sip now and then.
 */
function fountainPose(fit: FountainFit): (p: number) => PoseSpec {
  const { s, to, mouth } = fit;
  const up = fit.up ?? 0;
  // The targets in the clip's metres, from the feet (+z towards the fountain).
  const goal: Vec = [0, (mouth[0] - up) / s, (to - mouth[1]) / s];
  const tip = fit.tip ?? 0;
  // (Found on the first frame baked: the reference skeleton is there only while baking.)
  let bend = -1;
  let back = 0;
  const solve = () => {
    bend = 30;
    for (let k = 0; k < 40; k++) {
      const m = pointOn(bentBody(bend, back, tip, 0, 0), 'Head', MOUTH);
      bend = Math.min(95, Math.max(0, bend + (m[1] - goal[1]) * 90));
      back = Math.min(0.18, Math.max(-0.25, back + (goal[2] - m[2]) * 0.8));
    }
  };
  const rim = (sx: number): Vec => [(sx * RIM[0]) / s, (RIM[1] - up) / s, (to - RIM[2]) / s];
  return (p) => {
    if (bend < 0) solve();
    // Two sips a loop: down to the water and up a little between them.
    const sip = bump(p, 0.05, 0.45, 0.12) + bump(p, 0.55, 0.95, 0.12);
    const body = bentBody(bend - 4 + 4 * sip, back, tip, p, 10 + 3 * Math.sin(p * TAU));
    return {
      ...body,
      ...armIK('l', rim(-1), body, { palm: true, pole: [-0.9, -0.3, -0.3], pronate: 85, hand: wrist('l', -25, 0) }),
      ...armIK('r', rim(1), body, { palm: true, pole: [0.9, -0.3, -0.3], pronate: 85, hand: wrist('r', -25, 0) }),
    };
  };
}

export function streetClips(): ClipDef[] {
  return [
    ...FOUNTAIN.map(({ clip, fit }) => cycle(clip, { seconds: 3, loop: true }, fountainPose(fit))),
  ];
}
