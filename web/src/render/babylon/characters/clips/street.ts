/**
 * Things out in the town used up against them: bent over a drinking fountain's spout, and up on a climbing
 * frame's rope net. Each comes in several sizes, named by the height (cm) it is fitted to (`STEP_IN` in
 * characters.ts picks the one for a resident's height): the object's heights and depths are fixed, so each
 * is fitted to a body of its share `s` of the reference skeleton's size (1.77 m), standing `to` m from the
 * object's middle (the object's numbers, in metres, divided by `s` give the clip's).
 */

import { cycle, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { breathe, bump, TAU } from './idle';
import { armIK, footIK, pointOn, ramp, smooth, wrist } from './ik';

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

/**
 * The climbing frame (playground, `model.climbingFrame`): a rope net up its front, from the deck's edge
 * (1.1 m up, 0.86 m out from the middle) down to the ground 0.96 m out, its ropes across 0.21 m apart from
 * 4 cm up; above it the deck's front edge, 0.9 m out.
 */
type ClimbFit = Fit & { reach: number };
export const CLIMB: Size<ClimbFit>[] = sizes('climb', [0.97, 0.83, 0.63], (s) => ({ s, to: 0.9 + 0.24 * s, reach: s < 0.7 ? 3 : 4 }));
const RUNG = 0.208;
const RUNG0 = 0.04;
const RUNGS = 6;
/** The net's depth from the middle at a height (m): 0.96 at the ground, 0.86 at the top. */
const netZ = (y: number) => 0.96 - (0.1 * (y - RUNG0)) / (RUNG * (RUNGS - 1));
const DECK: [y: number, z: number] = [1.12, 0.9];

/**
 * Climbing up the net two ropes and down again, hands and feet in turn (right hand, left foot, left hand,
 * right foot); a look round at the top. The feet start on the second rope, the hands `reach` ropes higher;
 * a hand past the top rope takes the deck's edge.
 */
function climbPose(fit: ClimbFit): (p: number) => PoseSpec {
  const { s, to, reach } = fit;
  // A rope's place in the clip's metres (+z towards the net): its height, and the net's depth there.
  const rope = (k: number): [number, number] => {
    if (k > RUNGS - 1 + 1e-6) return [DECK[0] / s, (to - DECK[1]) / s];
    const y = RUNG0 + RUNG * k;
    return [y / s, (to - netZ(y)) / s];
  };
  // Between two ropes (a limb on its way from one to the next): along the straight line, lifted off the net.
  const along = (k: number): [y: number, z: number, off: number] => {
    const a = Math.floor(k);
    const t = k - a;
    const [y0, z0] = rope(a);
    const [y1, z1] = rope(a + 1);
    return [y0 + (y1 - y0) * t, z0 + (z1 - z0) * t, Math.sin(Math.PI * t) * 0.06];
  };
  // A limb's rope as the body climbs `c` ropes: it moves on in the quarter starting at `at`.
  const step = (c: number, at: number) => {
    const x = c - at;
    const f = Math.floor(x);
    return f + smooth((x - f) / 0.22);
  };
  return (p) => {
    // Up (0..0.4), a look round, down (0.5..0.9), a rest.
    const c = 2 * (ramp(p, 0, 0.4) - ramp(p, 0.5, 0.9));
    const rh = 1 + reach + step(c, 0);
    const lf = 1 + step(c, 0.25);
    const lh = 1 + reach + step(c, 0.5);
    const rf = 1 + step(c, 0.75);
    const [footY, footZ] = along(Math.min(lf, rf));
    // The hips 0.56 m over the lower foot and well back from the net, swaying a little to the side of the
    // limb on its way; the knees out to the sides.
    const look = bump(p, 0.4, 0.5, 0.03);
    const shift = 0.025 * Math.sin((c + 0.1) * Math.PI);
    const body: PoseSpec = {
      move: [shift, footY + 0.56 - 0.947, footZ - 0.4],
      pelvis: [8, 0, 0],
      spine_01: [3, 0, 0],
      spine_02: [3, 0, 0],
      spine_03: [2, 0, 0],
      neck_01: [-6, 10 * look * Math.sin(p * TAU * 10), 0],
      Head: [-10 + 6 * look, 25 * look * Math.sin((p - 0.4) * TAU * 10), 0],
    };
    const foot = (side: 'l' | 'r', k: number): PoseSpec => {
      const sx = side === 'l' ? -1 : 1;
      const [y, z, off] = along(k);
      // The ball of the foot on the rope: the ankle 8 cm above it and 0.15 m back from it (offsets from the
      // ankle's place at rest), lifted off between ropes.
      const at: Vec = [sx * (0.11 / s) - sx * 0.096, y + 0.078 - 0.085, z - 0.15 - off + 0.018];
      return footIK(side, at, body, { pole: [0.6 * sx, 0, 0.8], foot: [0, 10 * sx, 0] });
    };
    const hand = (side: 'l' | 'r', k: number): PoseSpec => {
      const sx = side === 'l' ? -1 : 1;
      const [y, z, off] = along(k);
      // The palm over the rope, the fingers round it; lifted off between ropes.
      const at: Vec = [sx * (0.17 / s), y + 0.03, z - 0.03 - off];
      return armIK(side, at, body, { palm: true, pole: [0.8 * sx, -0.6, -0.3], pronate: 80, hand: wrist(side, -20, 0) });
    };
    return { ...body, ...foot('l', lf), ...foot('r', rf), ...hand('l', lh), ...hand('r', rh) };
  };
}

export function streetClips(): ClipDef[] {
  return [
    ...FOUNTAIN.map(({ clip, fit }) => cycle(clip, { seconds: 3, loop: true }, fountainPose(fit))),
    ...CLIMB.map(({ clip, fit }) => cycle(clip, { seconds: 8, loop: true }, climbPose(fit))),
  ];
}
