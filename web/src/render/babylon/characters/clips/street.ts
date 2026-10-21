/**
 * Things out in the town used up against them: bent over a drinking fountain's spout, and round a climbing
 * frame: up its rope net and down its fireman's pole. Each comes in several sizes, named by the height (cm) it is fitted to (`STEP_IN` in
 * characters.ts picks the one for a resident's height): the object's heights and depths are fixed, so each
 * is fitted to a body of its share `s` of the reference skeleton's size (1.77 m), standing `to` m from the
 * object's middle (the object's numbers, in metres, divided by `s` give the clip's).
 */

import { cycle, reference, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { breathe, bump, TAU } from './idle';
import { armIK, footIK, lerpV, pointOn, ramp, smooth, wrist } from './ik';

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
export const REF_HEIGHT = 1.77;
/** The sizes of `name` for these shares of the reference body's size (the tallest first), each fitted by `fit`. */
export function sizes<F extends Fit>(name: string, shares: readonly number[], fit: (s: number) => F): Size<F>[] {
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
 * 4 cm up, 0.9 m wide from 0.75 m to the frame's left of its middle (`CLIMB_LANES`: two climbers side by
 * side); above it the deck (its top 1.13 m up) to 0.9 m out, under a roof whose front beam is 2.2 m up at
 * 0.85 m out; the fireman's pole 0.55 m to the right of the middle, 0.95 m out, up to 2.05 m.
 */
type ClimbFit = Fit & { reach: number; wide: number };
/**
 * (`wide`: a child's hips are wider for its legs than the reference's, about 0.12 m from the middle on the
 * reference's scale against its 0.096 (rig.rs proportions), and a woman's a little (0.10): the feet are
 * placed for those of the size's users (the tallest is for the men and the women), so that they stay put on
 * their skeletons as the hips turn over them, `footIK`'s `wide`.)
 */
export const CLIMB: Size<ClimbFit>[] = sizes('climb', [0.97, 0.83, 0.63], (s) => ({ s, to: 0.9 + 0.24 * s, reach: s < 0.7 ? 3 : 4, wide: s < 0.7 ? 0.024 : s > 0.9 ? 0.004 : 0 }));
/** Where the climbers go up the net: m to the frame's right of its middle (their feet's middle). */
export const CLIMB_LANES = [-0.53, -0.1] as const;
/** The clip of size `clip` for lane `lane` (an index of `CLIMB_LANES`). */
export const climbClip = (clip: string, lane: number): string => `${clip}${'ab'[lane]}`;
/** A round's length (s). */
const CLIMB_S = 13;
const RUNG = 0.208;
const RUNG0 = 0.04;
const RUNGS = 6;
/** The net's depth from the middle at a height (m): 0.96 at the ground, 0.86 at the top. */
const netZ = (y: number) => 0.96 - (0.1 * (y - RUNG0)) / (RUNG * (RUNGS - 1));
const DECK: [y: number, z: number] = [1.12, 0.9];
/** The deck's top (m), the pole (to the right of the middle, out from it; its radius), and how high on it a hand may go (under its arm). */
const DECK_TOP = 1.13;
const POLE = { x: 0.55, z: 0.95, r: 0.025, top: 1.98 };
/** Room under the roof over the deck where the climbers go (m), and the top of the head from its joint (m, reference skeleton). */
const HEADROOM = 1.0;
const HEAD_TOP: Vec = [0, 0.19, 0.02];
/** Ropes climbed before going over onto the deck. */
const CLIMB_ROPES = 3;

/**
 * When each part of a round happens (shares of it), per lane: up the net (to `net`), over the edge onto the
 * deck (`mount`), a look round (to `wait`: the second lane waits for the first to slide), across the deck
 * to the pole (`cross`), hands on it and round it (`swing`), the slide (`slide`), the landing (`land`), and
 * back round to the net (`back`; on again by the end).
 */
const ROUND = [
  { net: 0.22, mount: 0.32, wait: 0.34, cross: 0.45, swing: 0.52, slide: 0.58, land: 0.62, back: 0.93 },
  { net: 0.22, mount: 0.32, wait: 0.5, cross: 0.58, swing: 0.65, slide: 0.71, land: 0.75, back: 0.94 },
] as const;
/** How far in from the deck's edge each lane goes across it (m out from the middle): the first behind the second. */
const DECK_Z = [0.42, 0.7];
/** How far out (m) they walk back round to the net, clear of someone on it. */
const BACK_Z = 1.62;

/**
 * A limb's hold: from `t0` to `t1` (shares of the round) it is at `at` (the clip's metres; null: a hand free,
 * by the side), the foot turned `yaw` degrees; between holds it moves on, lifted.
 */
type Hold = { t0: number; t1: number; at: Vec | null; yaw?: number };
/** A key of the body: at `t`, the pelvis at `at` (the clip's metres), turned `yaw` and leant forward `lean` degrees. */
type BodyKey = { t: number; at: Vec; yaw: number; lean: number };

/** A track of holds at `p`: where the limb is (`free` for a free hand), how far it is lifted off (0..1 between holds), and its yaw. */
function holdAt(holds: readonly Hold[], p: number, free: Vec = [0, 0, 0]): { at: Vec; lift: number; yaw: number } {
  for (let i = 0; i < holds.length; i++) {
    const h = holds[i];
    if (p >= h.t0 && p <= h.t1) return { at: h.at ?? free, lift: 0, yaw: h.yaw ?? 0 };
    const n = holds[i + 1];
    if (n && p > h.t1 && p < n.t0) {
      const u = smooth((p - h.t1) / (n.t0 - h.t1));
      // (Lifted clear until it is all but there.)
      return { at: lerpV(h.at ?? free, n.at ?? free, u), lift: Math.sqrt(Math.sin(Math.PI * u)), yaw: (h.yaw ?? 0) + ((n.yaw ?? 0) - (h.yaw ?? 0)) * u };
    }
  }
  const last = holds[holds.length - 1];
  return { at: last.at ?? free, lift: 0, yaw: last.yaw ?? 0 };
}

/** The body's keys at `p`: eased from key to key. */
function bodyAt(keys: readonly BodyKey[], p: number): BodyKey {
  let i = 0;
  while (i < keys.length - 2 && p > keys[i + 1].t) i++;
  const [a, b] = [keys[i], keys[i + 1]];
  const u = smooth((p - a.t) / Math.max(1e-6, b.t - a.t));
  return { t: p, at: lerpV(a.at, b.at, u), yaw: a.yaw + (b.yaw - a.yaw) * u, lean: a.lean + (b.lean - a.lean) * u };
}

/**
 * The climber's body: the pelvis at `at` (the clip's metres), turned `yaw` and leant over `lean` degrees (the
 * head kept up), as on the net by `onIt` (the hips back, the back and head up to the hands), the head turned
 * `look` degrees.
 */
function climbBody(at: Vec, yaw: number, lean: number, onIt: number, look: number): PoseSpec {
  const restP = reference().joint('pelvis');
  return {
    move: [at[0] - restP[0], at[1] - restP[1], at[2] - restP[2]],
    pelvis: [8 * onIt + lean * 0.45, yaw, 0],
    spine_01: [3 * onIt + lean * 0.2, 0, 0],
    spine_02: [3 * onIt + lean * 0.2, 0, 0],
    spine_03: [2 * onIt + lean * 0.15, 0, 0],
    neck_01: [-6 * onIt - lean * 0.4, look, 0],
    Head: [-10 * onIt - lean * 0.4, 3 * look, 0],
  };
}

/** A direction in the clip's frame turned by `yaw` degrees (as the pelvis turns it: + towards the clip's +x). */
function turned(v: Vec, yaw: number): Vec {
  const a = (yaw * Math.PI) / 180;
  return [v[0] * Math.cos(a) + v[2] * Math.sin(a), v[1], -v[0] * Math.sin(a) + v[2] * Math.cos(a)];
}

/**
 * A round on the climbing frame from lane `lane` (an index of `CLIMB_LANES`): up the net, hands and feet in
 * turn (right hand, left foot, left hand, right foot) three ropes, over the edge onto the deck and a look
 * round (the second lane waits there for the first to slide), across the deck ducking under the roof (as far
 * as the body's height needs) to the pole, hands on it and round it, the legs round it as it slides down, a
 * landing with bent knees, and back round, out clear of the net, to step on again where the round began.
 */
function climbPose(fit: ClimbFit, lane: number): (p: number) => PoseSpec {
  const { s, to, reach } = fit;
  const L = CLIMB_LANES[lane];
  const R = ROUND[lane];
  // The frame's metres (to its right of the middle, up, out from it) in the clip's (from the start, +z towards the frame).
  const C = (x: number, y: number, z: number): Vec => [(L - x) / s, y / s, (to - z) / s];
  // Found on the first frame baked (the reference skeleton is there only while baking).
  let made: { body: BodyKey[]; lf: Hold[]; rf: Hold[]; lh: Hold[]; rh: Hold[] } | null = null;

  // --- on the net (the clip's metres; c ropes climbed) ---
  const rope = (k: number): [number, number] => {
    if (k > RUNGS - 1 + 1e-6) return [DECK[0] / s, (to - DECK[1]) / s];
    const y = RUNG0 + RUNG * k;
    return [y / s, (to - netZ(y)) / s];
  };
  const along = (k: number): [y: number, z: number, off: number] => {
    const a = Math.floor(k);
    const t = k - a;
    const [y0, z0] = rope(a);
    const [y1, z1] = rope(a + 1);
    return [y0 + (y1 - y0) * t, z0 + (z1 - z0) * t, Math.sin(Math.PI * t) * 0.06];
  };
  const step = (c: number, at: number) => {
    const x = c - at;
    const f = Math.floor(x);
    return f + smooth((x - f) / 0.22);
  };
  /** On the net, `c` ropes up: the pelvis (absolute), and the limbs (ankles absolute, palms). */
  const onNet = (c: number) => {
    const rh = 1 + reach + step(c, 0);
    const lf = 1 + step(c, 0.25);
    const lh = 1 + reach + step(c, 0.5);
    const rf = 1 + step(c, 0.75);
    const [footY, footZ] = along(Math.min(lf, rf));
    const shift = 0.025 * Math.sin((c + 0.1) * Math.PI);
    const ankle = (sx: number, k: number): Vec => {
      const [y, z, off] = along(k);
      return [sx * (0.11 / s), y + 0.078, z - 0.15 - off];
    };
    const palm = (sx: number, k: number): Vec => {
      const [y, z, off] = along(k);
      return [sx * (0.17 / s), y + 0.03, z - 0.03 - off];
    };
    // (The hips 0.56 m over the lower foot and 0.4 m back from it, as the rest pose's are over the feet.)
    const rest = reference().joint('pelvis');
    return { pelvis: [rest[0] + shift, footY + 0.56 - 0.947 + rest[1], rest[2] + footZ - 0.4] as Vec, lf: ankle(-1, lf), rf: ankle(1, rf), lh: palm(-1, lh), rh: palm(1, rh) };
  };

  const make = () => {
    const ref = reference();
    const restP = ref.joint('pelvis');
    const pz = restP[2];
    const ank = 0.085;
    // Ducking under the roof on the deck (the top of the head no more than `HEADROOM` over it, clear of its
    // front beam): leant over as far as the body's height needs, the rest in the knees.
    const lean = Math.min(50, Math.max(0, 1.77 * s - HEADROOM) * 90);
    let crouch = 0.92;
    while (crouch > 0.42 && pointOn(climbBody([0, crouch, pz], 0, lean, 0, 0), 'Head', HEAD_TOP)[1] > HEADROOM / s) crouch -= 0.01;
    const top = onNet(CLIMB_ROPES);
    const start = onNet(0);
    const body: BodyKey[] = [];
    const lf: Hold[] = [];
    const rf: Hold[] = [];
    const lh: Hold[] = [];
    const rh: Hold[] = [];
    // The pelvis at a place on the frame (its metres) at height `h` (the clip's metres) over `floor` (m).
    const P = (x: number, floor: number, z: number, h: number): Vec => {
      const c = C(x, floor, z);
      return [c[0], c[1] + h, c[2] + pz];
    };
    // A foot on `floor` (m) beside a body at (x, z) of the frame turned `yaw`, `side` to its right and `fwd` ahead (the clip's metres).
    const F = (x: number, floor: number, z: number, yaw: number, side: number, fwd = 0): Vec => {
      const c = C(x, floor, z);
      const r = turned([side, 0, fwd], yaw);
      return [c[0] + r[0], c[1] + ank, c[2] + r[2]];
    };
    const deck = DECK_TOP;
    const zD = DECK_Z[lane];
    const xP = POLE.x;
    // 1. Up the net: the clip's own steps (`onNet`) until `net`.
    // 2. Over the edge: hands onto the deck, up over it leaning in, a foot each onto it, up into a crouch.
    const m0 = R.net;
    const m = (k: number) => m0 + (R.mount - m0) * k;
    body.push({ t: 0, at: start.pelvis, yaw: 0, lean: 0 });
    body.push({ t: m0, at: top.pelvis, yaw: 0, lean: 0 });
    body.push({ t: m(0.5), at: P(L, deck, DECK[1] + 0.05, 0.25), yaw: 0, lean: 50 });
    body.push({ t: R.mount, at: P(L, deck, zD, crouch), yaw: 0, lean });
    const hand = (side: number, x: number, y: number, z: number): Vec => {
      const c = C(x, y, z);
      return [c[0] + side * 0.2, c[1], c[2]];
    };
    lh.push({ t0: 0, t1: m0, at: top.lh }, { t0: m(0.15), t1: m(0.7), at: hand(-1, L, deck + 0.02, DECK[1] - 0.25) });
    rh.push({ t0: 0, t1: m0, at: top.rh }, { t0: m(0.1), t1: m(0.75), at: hand(1, L, deck + 0.02, DECK[1] - 0.22) });
    rf.push({ t0: 0, t1: m(0.3), at: top.rf }, { t0: m(0.6), t1: 0, at: F(L, deck, zD + 0.12, 0, 0.1) });
    lf.push({ t0: 0, t1: m(0.55), at: top.lf }, { t0: m(0.85), t1: 0, at: F(L, deck, zD + 0.05, 0, -0.1) });
    lh.push({ t0: m(0.95), t1: R.cross + 0.2 * (R.swing - R.cross), at: null });
    rh.push({ t0: m(0.95), t1: R.cross + 0.25 * (R.swing - R.cross), at: null });
    // 3. A look round; then a quarter turn to the right of the frame and across the deck, crouched, and a
    // quarter turn more to face out at the pole.
    const c0 = R.wait;
    const c1 = R.cross;
    const cr = (k: number) => c0 + (c1 - c0) * k;
    body.push({ t: c0, at: P(L, deck, zD, crouch), yaw: 0, lean });
    body.push({ t: cr(0.15), at: P(L, deck, zD, crouch), yaw: -90, lean });
    body.push({ t: cr(0.8), at: P(xP, deck, zD, crouch), yaw: -90, lean });
    body.push({ t: c1, at: P(xP, deck, zD, crouch), yaw: -180, lean });
    // Footsteps: turning on the spot, then steps of about 0.3 m (the body's), then turning to face out.
    const feetDeck = (x: number, yaw: number) => ({ l: F(x, deck, zD, yaw, -0.1), r: F(x, deck, zD, yaw, 0.1) });
    const turnA = feetDeck(L, -90);
    rf[rf.length - 1].t1 = cr(0.04);
    lf[lf.length - 1].t1 = cr(0.1);
    rf.push({ t0: cr(0.1), t1: cr(0.17), at: turnA.r, yaw: -90 });
    lf.push({ t0: cr(0.15), t1: cr(0.22), at: turnA.l, yaw: -90 });
    const span = xP - L;
    const n = Math.max(2, Math.round(span / (0.3 * s)));
    for (let k = 1; k <= n; k++) {
      const x = L + (span * k) / n;
      const t = cr(0.15 + (0.65 * k) / n);
      const f = feetDeck(x, -90);
      (k % 2 ? rf : lf).push({ t0: t, t1: t + 0.04 * (c1 - c0), at: k % 2 ? f.r : f.l, yaw: -90 });
      // (The hips between the feet as each lands.)
      body.push({ t, at: P(L + (span * (k - 0.5)) / n, deck, zD, crouch), yaw: -90, lean });
    }
    const last = feetDeck(xP, -90);
    (n % 2 ? lf : rf).push({ t0: cr(0.84), t1: cr(0.86), at: n % 2 ? last.l : last.r, yaw: -90 });
    const out = feetDeck(xP, -180);
    rf.push({ t0: cr(0.92), t1: 0, at: out.r, yaw: -180 });
    lf.push({ t0: c1, t1: 0, at: out.l, yaw: -180 });
    // 4. Hands on the pole (the left above), round it to the left: the body hung from it, out over the
    // edge, round to the outside facing the frame; the legs round it.
    const s0 = c1;
    const s1 = R.swing;
    const sw = (k: number) => s0 + (s1 - s0) * k;
    const hTop = Math.min(POLE.top, deck + crouch * s + 0.5 * s);
    const hLow = hTop - 0.2 * s;
    const ring = 0.08 * s + 0.1;
    // Round the pole: φ from 0 (inside, facing out) to 1 (outside, facing in), by its left (the middle's side).
    const round = (f: number) => {
      const a = Math.PI * f;
      return { x: xP - ring * Math.sin(a), z: POLE.z - ring * Math.cos(a), yaw: -180 + 180 * f };
    };
    const hang = hTop - 0.86 * s;
    const grip = (y: number, f: number, side: number): Vec => {
      // On the pole, on the side towards the body, a hand's width apart.
      const r = round(f);
      const [dx, dz] = [r.x - xP, r.z - POLE.z];
      const d = Math.hypot(dx, dz) || 1;
      const c = C(xP + (dx / d) * POLE.r * 1.6, y, POLE.z + (dz / d) * POLE.r * 1.6);
      const side3 = turned([side * 0.02 / s, 0, 0], r.yaw);
      return [c[0] + side3[0], c[1], c[2] + side3[2]];
    };
    lh.push({ t0: sw(0.25), t1: sw(0.3), at: grip(hTop, 0, 0) });
    rh.push({ t0: sw(0.3), t1: sw(0.35), at: grip(hLow, 0, 0) });
    rf[rf.length - 1].t1 = sw(0.38);
    lf[lf.length - 1].t1 = sw(0.42);
    for (let k = 1; k <= 6; k++) {
      const f = k / 6;
      const t = sw(0.35 + 0.65 * f);
      const r = round(f);
      // Stepping round on the deck's edge, still ducking under the roof's edge, until out past it, then
      // hanging from the hands.
      const down = smooth((f - 0.3) / 0.6);
      body.push({ t, at: P(r.x, 0, r.z, crouch + deck / s + (hang / s - crouch - deck / s) * down), yaw: r.yaw, lean: lean + (5 - lean) * down });
      lh.push({ t0: t, t1: t, at: grip(hTop, f, 0) });
      rh.push({ t0: t, t1: t, at: grip(hLow, f, 0) });
      if (f <= 0.5) {
        rf.push({ t0: t, t1: t, at: F(r.x, deck, r.z, r.yaw, 0.1, -0.05), yaw: r.yaw });
        lf.push({ t0: t + 0.01, t1: t + 0.01, at: F(r.x, deck, r.z, r.yaw, -0.1, -0.05), yaw: r.yaw });
      }
    }
    // The legs round the pole: the ankles crossed behind it, a little below the knees' height.
    const wrapped = (y: number, side: number): Vec => {
      const r = round(1);
      const c = C(xP, y, POLE.z);
      const o = turned([side * 0.05 / s, 0, 0.03 / s], r.yaw);
      return [c[0] + o[0], c[1], c[2] + o[2]];
    };
    const wrapY = hang - 0.62 * s;
    rf.push({ t0: s1, t1: s1, at: wrapped(wrapY, 0.6) });
    lf.push({ t0: s1, t1: s1, at: wrapped(wrapY - 0.04 * s, -0.6) });
    // 5. Down the pole, hands and legs round it; the feet let go just before the ground.
    const d1 = R.slide;
    const land = 0.84 * s;
    const yLand = land;
    body.push({ t: d1, at: P(round(1).x, 0, round(1).z, yLand / s), yaw: 0, lean: 5 });
    const drop = hang - yLand;
    lh.push({ t0: d1, t1: d1, at: grip(hTop - drop, 1, 0) });
    rh.push({ t0: d1, t1: d1, at: grip(hLow - drop, 1, 0) });
    const slideAt = (k: number) => s1 + (d1 - s1) * k;
    rf.push({ t0: slideAt(0.75), t1: slideAt(0.75), at: wrapped(wrapY - drop * 0.75, 0.6) });
    lf.push({ t0: slideAt(0.75), t1: slideAt(0.75), at: wrapped(wrapY - 0.04 * s - drop * 0.75, -0.6) });
    // 6. Landed: feet either side of the pole's foot, knees giving; hands off it.
    const o = round(1);
    const g = { l: F(o.x, 0, o.z, 0, -0.12, -0.04), r: F(o.x, 0, o.z, 0, 0.12, -0.04) };
    const l1 = R.land;
    rf.push({ t0: d1, t1: l1 + 0.01, at: g.r, yaw: 0 });
    lf.push({ t0: d1, t1: l1 + 0.02, at: g.l, yaw: 0 });
    body.push({ t: d1 + (l1 - d1) * 0.4, at: P(o.x, 0, o.z, (yLand - 0.08 * s) / s), yaw: 0, lean: 12 });
    body.push({ t: l1, at: P(o.x, 0, o.z, 0.92), yaw: 0, lean: 0 });
    lh.push({ t0: d1 + (l1 - d1) * 0.3, t1: d1 + (l1 - d1) * 0.3, at: grip(hTop - drop - 0.05 * s, 1, 0) });
    rh.push({ t0: d1 + (l1 - d1) * 0.2, t1: d1 + (l1 - d1) * 0.2, at: grip(hLow - drop - 0.05 * s, 1, 0) });
    lh.push({ t0: d1 + (l1 - d1) * 0.6, t1: R.back + (1 - R.back) * 0.1, at: null });
    rh.push({ t0: d1 + (l1 - d1) * 0.5, t1: R.back + (1 - R.back) * 0.2, at: null });
    // 7. Back: two steps back from the pole, a quarter turn, across, a quarter turn back to the net, in.
    const b0 = l1;
    const b1 = R.back;
    const bk = (k: number) => b0 + (b1 - b0) * k;
    const zB = BACK_Z;
    const xS = L;
    const startC = start.pelvis;
    // Where they stand on the ground before stepping on: under the start's pelvis, a little back.
    const zIn = to + 0.08;
    body.push({ t: bk(0.18), at: P(o.x, 0, zB, 0.9), yaw: 0, lean: 0 });
    body.push({ t: bk(0.27), at: P(o.x, 0, zB, 0.9), yaw: 90, lean: 0 });
    body.push({ t: bk(0.75), at: P(xS, 0, zB, 0.9), yaw: 90, lean: 0 });
    body.push({ t: bk(0.84), at: P(xS, 0, zB, 0.9), yaw: 0, lean: 0 });
    body.push({ t: b1, at: P(xS, 0, zIn, 0.92), yaw: 0, lean: 0 });
    const gf = (x: number, z: number, yaw: number) => ({ l: F(x, 0, z, yaw, -0.1), r: F(x, 0, z, yaw, 0.1) });
    const steps: [number, 'l' | 'r', Vec, number][] = [];
    const mid = (o.z + zB) / 2;
    steps.push([bk(0.05), 'r', gf(o.x, mid, 0).r, 0], [bk(0.12), 'l', gf(o.x, zB, 0).l, 0], [bk(0.18), 'r', gf(o.x, zB, 0).r, 0]);
    steps.push([bk(0.22), 'l', gf(o.x, zB, 90).l, 90], [bk(0.27), 'r', gf(o.x, zB, 90).r, 90]);
    const across = o.x - xS;
    const na = Math.max(2, Math.round(across / (0.42 * s)));
    for (let k = 1; k <= na; k++) {
      const x = o.x - (across * k) / na;
      const f = gf(x, zB, 90);
      steps.push([bk(0.27 + (0.48 * k) / na), k % 2 ? 'l' : 'r', k % 2 ? f.l : f.r, 90]);
      body.push({ t: bk(0.27 + (0.48 * k) / na), at: P(o.x - (across * (k - 0.5)) / na, 0, zB, 0.9), yaw: 90, lean: 0 });
    }
    const fin = gf(xS, zB, 90);
    steps.push([bk(0.78), na % 2 ? 'r' : 'l', na % 2 ? fin.r : fin.l, 90]);
    steps.push([bk(0.81), 'l', gf(xS, zB, 0).l, 0], [bk(0.84), 'r', gf(xS, zB, 0).r, 0]);
    steps.push([bk(0.92), 'l', gf(xS, (zB + zIn) / 2, 0).l, 0], [b1, 'r', gf(xS, zIn, 0).r, 0], [b1 + 0.01, 'l', gf(xS, zIn, 0).l, 0]);
    let prevL = lf[lf.length - 1];
    let prevR = rf[rf.length - 1];
    for (const [t, side, at, yaw] of steps) {
      const list = side === 'l' ? lf : rf;
      const prev = side === 'l' ? prevL : prevR;
      const dur = 0.3 / CLIMB_S;
      prev.t1 = Math.max(prev.t0, t - dur);
      const h = { t0: t, t1: t, at, yaw };
      list.push(h);
      if (side === 'l') prevL = h;
      else prevR = h;
    }
    // 8. On again: hands up onto the ropes, the right foot onto the second rope, then the left.
    body.push({ t: 1, at: startC, yaw: 0, lean: 0 });
    prevR.t1 = b1 + (1 - b1) * 0.3;
    prevL.t1 = b1 + (1 - b1) * 0.55;
    rf.push({ t0: b1 + (1 - b1) * 0.65, t1: 1, at: start.rf });
    lf.push({ t0: 1, t1: 1, at: start.lf });
    rh.push({ t0: b1 + (1 - b1) * 0.35, t1: 1, at: start.rh });
    lh.push({ t0: b1 + (1 - b1) * 0.2, t1: 1, at: start.lh });
    body.sort((a, b) => a.t - b.t);
    return { body, lf, rf, lh, rh };
  };

  return (p) => {
    made ??= make();
    const ref = reference();
    const net = p < ROUND[lane].net;
    const c = CLIMB_ROPES * ramp(p, 0, ROUND[lane].net);
    const n = onNet(c);
    const k = bodyAt(made.body, p);
    const at = net ? n.pelvis : k.at;
    const yaw = net ? 0 : k.yaw;
    const lean = net ? 0 : k.lean;
    // On the net: the hips back, the back and head up to the hands; elsewhere upright (and leant over).
    const onIt = net ? 1 : 1 - ramp(p, ROUND[lane].net, ROUND[lane].mount) + ramp(p, ROUND[lane].back, 1);
    const look = bump(p, ROUND[lane].mount, ROUND[lane].wait + 0.02, 0.03);
    const body = climbBody(at, yaw, lean, onIt, 10 * look * Math.sin(p * TAU * 6));
    const foot = (side: 'l' | 'r', holds: Hold[], netAt: Vec): PoseSpec => {
      const sx = side === 'l' ? -1 : 1;
      const h = holdAt(holds, p);
      const a = net ? netAt : h.at;
      const lift = net ? 0 : h.lift * 0.07;
      const rest = ref.joint(`foot_${side}`);
      const pole = lerpV(turned([0.25 * sx, 0, 1], yaw), [0.6 * sx, 0, 0.8], onIt);
      const wrap = p > ROUND[lane].cross + 0.3 * (ROUND[lane].swing - ROUND[lane].cross) && p < ROUND[lane].slide;
      return footIK(side, [a[0] - rest[0], a[1] + lift - rest[1], a[2] - rest[2]], body, {
        pole: wrap ? turned([0.9 * sx, 0, 0.45], yaw) : pole,
        foot: [wrap ? 30 : 0, (net ? 0 : h.yaw) + 10 * sx * (0.3 + 0.7 * onIt), 0],
        wide: fit.wide,
      });
    };
    const arm = (side: 'l' | 'r', holds: Hold[], netAt: Vec): PoseSpec => {
      const sx = side === 'l' ? -1 : 1;
      // Free across the deck and after the landing: by the sides, swinging a little.
      const free = turned([sx * 0.2, -0.27, 0.06 + 0.04 * Math.sin(p * TAU * 9 + (sx > 0 ? 0 : Math.PI))], yaw);
      const target: Vec = net ? netAt : holdAt(holds, p, [at[0] + free[0], at[1] + free[1], at[2] + free[2]]).at;
      return armIK(side, target, body, { palm: true, pole: turned([0.8 * sx, -0.6, -0.3], yaw), pronate: 80, hand: wrist(side, -20, 0) });
    };
    return { ...body, ...foot('l', made.lf, n.lf), ...foot('r', made.rf, n.rf), ...arm('l', made.lh, n.lh), ...arm('r', made.rh, n.rh) };
  };
}

export function streetClips(): ClipDef[] {
  return [
    ...FOUNTAIN.map(({ clip, fit }) => cycle(clip, { seconds: 3, loop: true }, fountainPose(fit))),
    ...CLIMB.flatMap(({ clip, fit }) => CLIMB_LANES.map((_, lane) => cycle(climbClip(clip, lane), { seconds: CLIMB_S, loop: true }, climbPose(fit, lane)))),
  ];
}
