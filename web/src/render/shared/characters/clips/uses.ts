/**
 * Standing uses that need a clip of their own: bent down to a telescope's eyepiece and reaching up into a
 * fruit tree, both in sizes like the street's (`street.ts`: the object's heights are fixed, so each size is
 * fitted to a body of its share `s` of the reference's size, standing `to` m from the object's middle;
 * `STEP_IN` in characters.ts picks the size for a resident's height and steps them up to it); washing in the
 * shower; and holding a guitar or a ukulele across the body, strumming.
 */

import { clip, cycle, reference, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { breathe, bump, hangArms, stand, TAU } from './idle';
import { armIK, elbow, footIK, hang, len, lerpV, pointOn, ramp, smooth, sub, turned, wrist } from './ik';
import { SHARES_177, SHARES_181, sizes, type Fit, type Size } from './street';

/**
 * The telescope (`model.telescope`) as it is used, its tube tilted up its `aim` (35° more, about the axle
 * 1.12 m up): the eyecup is 0.755 m up and 0.27 m to the front of the middle, looking out to the front
 * and 20° up; the eye is just off it (`EYECUP`, m up and to the front). The tube there, where a hand
 * steadies it, about 0.85 m up and 0.15 m out (`TUBE`).
 */
const EYECUP: [y: number, z: number] = [0.765, 0.3];
const TUBE: [y: number, z: number] = [0.85, 0.15];
/** The gaze into the eyepiece: this far (deg) below level. */
const SCOPE_GAZE = 20;
/** Sizes from 1.81 m down to 1.12 m, the feet nearer the eyepiece the smaller they are (less bent over it). */
export const EYEPIECE: Size[] = sizes('eyepiece', SHARES_181, (s) => ({ s, to: 0.54 + 0.41 * (s - 0.63) }));

/**
 * Bent over by `bend` degrees (the most of it at the hips), the knees giving as it deepens, the hips `back`
 * m behind the feet and moved `side` m to the left (to bring the right eye over the eyepiece); the face
 * kept `gaze` degrees below level; the feet planted a little apart.
 */
function stoopBody(bend: number, back: number, side: number, p: number, gaze: number): PoseSpec {
  const b = breathe(p, 0.7);
  const hip = 0.55 * bend;
  const lean = bend - hip;
  const sink = Math.max(0, bend - 35) * 0.0055;
  const neck = gaze - bend;
  const body: PoseSpec = {
    move: [side, -sink + (b.move?.[1] ?? 0), back],
    pelvis: [hip, 0, 0],
    spine_01: [lean * 0.35, 0, 0],
    spine_02: [lean * 0.35 + (b.spine_02?.[0] ?? 0), 0, 0],
    spine_03: [lean * 0.3 + (b.spine_03?.[0] ?? 0), 0, 0],
    clavicle_l: b.clavicle,
    clavicle_r: [0, 0, -(b.clavicle?.[2] ?? 0)],
    neck_01: [neck * 0.45, 0, 0],
    Head: [neck * 0.55, 0, 0],
  };
  return { ...body, ...footIK('l', [-0.04, 0, 0.01], body, { foot: [0, -8, 0] }), ...footIK('r', [0.04, 0, 0.01], body, { foot: [0, 8, 0] }) };
}

/**
 * At the telescope's eyepiece: bent down to it with the right eye at the eyecup (found while baking: the
 * bend for the height, the hips back or forward for the reach), the right hand steadying the tube and
 * now and then turning the focus, the left on the knee; once a loop the head comes up off it for a look at
 * the sky and goes back.
 */
function eyepiecePose(fit: Fit): (p: number) => PoseSpec {
  const { s, to } = fit;
  const goal: Vec = [0, EYECUP[0] / s, (to - EYECUP[1]) / s];
  let bend = -1;
  let back = 0;
  let eye: Vec = [0, 0, 0];
  const solve = () => {
    const ref = reference();
    const socket = ref.joint('eye_r');
    const head = ref.joint('Head');
    eye = [socket[0] - head[0], socket[1] - head[1], socket[2] - head[2]];
    bend = 30;
    for (let k = 0; k < 60; k++) {
      const e = pointOn(stoopBody(bend, back, -eye[0], 0, SCOPE_GAZE), 'Head', eye);
      bend = Math.min(100, Math.max(0, bend + (e[1] - goal[1]) * 80));
      back = Math.min(0.15, Math.max(-0.35, back + (goal[2] - e[2]) * 0.8));
    }
  };
  const tube: Vec = [0.07 / s, TUBE[0] / s, (to - TUBE[1]) / s];
  return (p) => {
    if (bend < 0) solve();
    // Up off the eyepiece for a look at the sky (0.62 to 0.8 of the loop), and back.
    const up = bump(p, 0.62, 0.8, 0.06);
    const body = stoopBody(bend - 12 * up, back, -eye[0] * (1 - up), p, SCOPE_GAZE - 45 * up);
    const focus = bump(p, 0.15, 0.4, 0.06) * Math.sin(p * TAU * 6);
    // (On the left knee: just above its joint and in front.)
    const knee = pointOn(body, 'calf_l', [0, 0.06, 0.06]);
    return {
      ...body,
      ...armIK('r', tube, body, { palm: true, pole: [0.8, -0.6, -0.2], pronate: 80 + 25 * focus, hand: wrist('r', -10, 0) }),
      ...armIK('l', [knee[0], knee[1] + 0.03, knee[2] + 0.02], body, { palm: true, pole: [-0.8, -0.2, -0.5], pronate: 90, hand: wrist('l', -15, 0) }),
    };
  };
}

/**
 * The fruit trees' picked fruit (`trees.py`; each tree's `pick` clip shrinks it away 1.6 s into the use,
 * `PICK_DELAY` there, as the hand closes on it): m to the picker's right (facing the tree), up and to the
 * front of the trunk. The apple hangs under the crown's front, the lemon in the potted tree's crown.
 */
const FRUIT = { apple: [0.05, 1.84, 0.7], lemon: [-0.1, 1.62, 0.6] } as const;

/** The pickers (`SHARES_177`: a grown-up, a teen, a child), and the feet's distance from the trunk for each (the shorter nearer, the fruit nearer overhead). */
type PickFit = Fit & { fruit: readonly [x: number, y: number, z: number] };
export const PICK_APPLE: Size<PickFit>[] = sizes('pickApple', SHARES_177, (s) => ({ s, to: s > 0.9 ? 1 : 0.78, fruit: FRUIT.apple }));
export const PICK_LEMON: Size<PickFit>[] = sizes('pickLemon', SHARES_177, (s) => ({ s, to: s > 0.9 ? 0.95 : s > 0.7 ? 0.88 : 0.85, fruit: FRUIT.lemon }));
/** The picking clip's length (s), and when (shares of it) the hand reaches the fruit, twists it off and is down with it. */
const PICK_S = 3.3;
const PICK_AT = { reach: 0.42, twist: 0.6, down: 0.82 };

/**
 * Standing to reach up: on tiptoe by `tip` (m, the clip's), the chest lifting and leaning back `back`
 * degrees, the head tipped back `look` degrees to the hand.
 */
function reachBody(tip: number, back: number, look: number, p: number): PoseSpec {
  const body = stand({ breath: p, lean: -back, shift: 0.25, head: [-look, 4, 0] });
  if (tip <= 0) return body;
  const lifted: PoseSpec = { ...body, move: [body.move![0], body.move![1] + tip, body.move![2]] };
  // On tiptoe the heels come up about the balls of the feet (0.17 m ahead of the ankles).
  const toe = Math.asin(Math.min(0.9, tip / 0.17));
  const ankle: Vec = [0, tip, -0.17 * (1 - Math.cos(toe))];
  const deg = (toe * 180) / Math.PI;
  return { ...lifted, ...footIK('l', ankle, lifted, { foot: [deg, -4, 0] }), ...footIK('r', ankle, lifted, { foot: [deg, 4, 0] }), ball_l: [-deg, 0, 0], ball_r: [-deg, 0, 0] };
}

/**
 * Picking fruit from a tree: a look up at it, the right hand up to it (on tiptoe if it is high for them,
 * as far as they reach if out of it), the left steadying the bough above; a twist off, and down to the
 * chest with it (the `eat` clip follows).
 */
function pickKeys(fit: PickFit): [number, () => PoseSpec, 'smooth'?][] {
  const { s, to, fruit } = fit;
  const at: Vec = [fruit[0] / s, fruit[1] / s, (to - fruit[2]) / s];
  const bough: Vec = [at[0] - 0.16, at[1] + 0.1, at[2] - 0.03];
  // How high on tiptoe (found while baking): enough for the right shoulder to come within reach of the fruit.
  let tip = -1;
  const lift = () => {
    if (tip >= 0) return tip;
    const ref = reference();
    const arm = len(sub(ref.joint('upperarm_r'), ref.joint('lowerarm_r'))) + len(sub(ref.joint('lowerarm_r'), ref.joint('hand_r'))) + 0.06;
    const shoulder = pointOn(reachBody(0, 8, 30, 0), 'upperarm_r', [0, 0, 0]);
    tip = Math.min(0.1, Math.max(0, len(sub(at, shoulder)) - arm * 0.96));
    return tip;
  };
  const rest = () => ({ ...stand(), ...hangArms() });
  const up = (t: number, twist: number, k = 1) => {
    const body = reachBody(lift() * k, 8 * k, 30 + 10 * k, t);
    return {
      ...body,
      ...armIK('r', [at[0], at[1] - 0.03 * (1 - k), at[2]], body, { palm: true, pole: [0.7, -0.4, -0.6], pronate: -40 + 70 * twist, hand: wrist('r', -25 + 15 * twist, 0, 0) }),
      ...armIK('l', bough, body, { palm: true, pole: [-0.7, -0.4, -0.6], pronate: 40, hand: wrist('l', -20, 0) }),
    };
  };
  const down = (t: number, k: number) => {
    const body = stand({ breath: t, shift: 0.2, head: [12 * k, 4 - 8 * k, 0] });
    return { ...body, ...hangArms(), ...armIK('r', [0.08, 1.12, 0.3], body, { palm: true, pole: [0.6, -1, -0.3], pronate: -50, hand: wrist('r', 15, 0) }) };
  };
  return [
    [0, rest],
    [0.12, () => ({ ...stand({ breath: 0.12, shift: 0.2, head: [-30, 4, 0] }), ...hangArms() }), 'smooth'],
    [0.3, () => up(0.3, 0, 0.6), 'smooth'],
    [PICK_AT.reach, () => up(PICK_AT.reach, 0), 'smooth'],
    [0.5, () => up(0.5, 1), 'smooth'],
    [PICK_AT.twist, () => up(PICK_AT.twist, 0.2), 'smooth'],
    [0.7, () => up(0.7, 0.1, 0.5), 'smooth'],
    [PICK_AT.down, () => down(PICK_AT.down, 1), 'smooth'],
    [1, () => down(1, 0.6), 'smooth'],
  ];
}

/**
 * Under the shower (the water falls from overhead, the head under it): when (shares of a round) the hair is
 * lathered, rinsed, the body turned a quarter to the left and soaped (the left arm, the right, the chest),
 * turned back, the face rinsed.
 */
const SHOWER_S = 14;
const SHOWER = { lather: [0.04, 0.3], rinse: [0.3, 0.4], turn: [0.4, 0.48], soap: [0.48, 0.68], back: [0.68, 0.76], face: [0.76, 0.92] } as const;
/**
 * The turn's yaw (deg, + to the right), and the steps: each foot's times and turn. A child turns the chest
 * instead (`showerChild`, `SHOWER_TWIST`): their hips are wider for their legs than the clips' reference's,
 * so turned over planted feet they would slide.
 */
const SHOWER_TURN = -50;
const SHOWER_TWIST = -40;
/** The shortest resident (m) who showers turning round (`shower`); shorter, `showerChild`. */
export const SHOWER_CHILD = 1.29;
const SHOWER_STEPS = [
  { side: 'l', t: [0.405, 0.435], from: 0, to: 1 },
  { side: 'r', t: [0.44, 0.47], from: 0, to: 1 },
  { side: 'r', t: [0.685, 0.715], from: 1, to: 0 },
  { side: 'l', t: [0.72, 0.75], from: 1, to: 0 },
] as const;

/** A foot's ankle offset from rest with the body turned `yaw` degrees about its middle (and the foot turned with it). */
function turnedFoot(side: 'l' | 'r', yaw: number): Vec {
  const a = reference().joint(`foot_${side}`);
  const t = turned(a, yaw);
  return [t[0] - a[0], 0, t[2] - a[2]];
}

/**
 * Showering: lathering the hair with both hands, the head bowed; rinsing it, the face up to the water and
 * the hands smoothing it back; a quarter turn in two steps, soaping the left arm, the right and the chest;
 * back round, and the face rinsed, the hands wiping the water down it.
 */
function showerPose(turn: number, twist: number): (p: number) => PoseSpec {
  return (p) => showerAt(p, turn, twist);
}

function showerAt(p: number, turn: number, twist: number): PoseSpec {
  const S = SHOWER;
  const lather = bump(p, S.lather[0], S.lather[1], 0.03);
  const rinse = bump(p, S.rinse[0], S.rinse[1], 0.025);
  const soap = bump(p, S.soap[0], S.soap[1], 0.025);
  const face = bump(p, S.face[0], S.face[1], 0.025);
  const turnK = ramp(p, S.turn[0], S.turn[1]) * (1 - ramp(p, S.back[0], S.back[1]));
  const yaw = turn * turnK;
  const scrub = Math.sin(p * TAU * 34);
  // The head: bowed lathering, back into the water rinsing.
  const nod = 14 * lather - 24 * rinse - 18 * face + 6 * soap;
  // (The hips kept still but for the turn, the weight a little on the right: the planted feet stay put.)
  const body0 = stand({ breath: p * 4, depth: 0.6, sink: 0.012, shift: 0.15, twist: twist * turnK, head: [nod + 2 * scrub * lather, 4 * Math.sin(p * TAU * 3) * soap + 0.6 * twist * turnK, 0], lean: -3 * (rinse + face) + 6 * soap });
  // (The breath in the chest only: the hips move only to turn.)
  const body: PoseSpec = { ...body0, move: [body0.move![0], -0.012, body0.move![2]], pelvis: [body0.pelvis![0], body0.pelvis![1] + yaw, body0.pelvis![2]] };
  // The feet: planted where they last stepped, lifted between (the turn's two steps there and back).
  const feet = (side: 'l' | 'r'): PoseSpec => {
    let at = 0;
    let lift = 0;
    for (const st of SHOWER_STEPS) {
      if (st.side !== side) continue;
      if (p >= st.t[1]) at = st.to;
      else if (p > st.t[0]) {
        // Up first, across, and down onto the new place.
        const u = (p - st.t[0]) / (st.t[1] - st.t[0]);
        at = st.from + (st.to - st.from) * smooth((u - 0.2) / 0.6);
        lift = Math.sin(Math.PI * u) * 0.045;
      }
    }
    const off = turnedFoot(side, turn * at);
    const sx = side === 'l' ? -1 : 1;
    return footIK(side, [off[0], off[1] + lift, off[2]], body, { foot: [lift * 150, turn * at + 4 * sx, 0] });
  };
  const legs = { ...feet('l'), ...feet('r') };
  const head = (x: number, y: number, z: number): Vec => pointOn(body, 'Head', [x, y, z]);
  // The arms at rest by the sides, and each task's hand places (palms on the scalp, smoothing back, on the arms, down the face).
  const rest = hangArms(4, 9, 18);
  const side = (sx: number): PoseSpec => {
    const k = sx < 0 ? 'l' : 'r';
    const o = sx < 0 ? 0 : Math.PI;
    const scalp = head(sx * 0.075 + 0.015 * Math.cos(p * TAU * 34 + o), 0.15 + 0.015 * Math.sin(p * TAU * 34 + o), 0.0);
    // Smoothing the hair back: from the forehead over the crown to the nape, twice.
    const back = ((p - S.rinse[0]) / (S.rinse[1] - S.rinse[0])) * 2 % 1;
    const smoothBack = head(sx * 0.05, 0.12 + 0.06 * Math.sin(Math.PI * back), 0.1 - 0.2 * back);
    // Wiping the face: the palms down it from the brow to the chin, twice.
    const wipe = ((p - S.face[0]) / (S.face[1] - S.face[0])) * 2 % 1;
    const down = head(sx * 0.045, 0.11 - 0.14 * wipe, 0.13);
    const w = lather + rinse + face;
    const at = lather ? scalp : rinse ? smoothBack : down;
    return w > 0.01 ? armIK(k, lerpV(pointOn({ ...body, ...rest }, `hand_${k}`, [0, 0, 0]), at, smooth(Math.min(1, w))), body, { palm: true, pole: [sx * 1, 0.1, -0.2], pronate: lather ? 20 : -10, hand: wrist(k, -20, 0) }) : {};
  };
  // Soaping: the right hand along the left arm (held a little out), then the left along the right, then both over the chest.
  let arms: PoseSpec = { ...rest, ...side(-1), ...side(1) };
  if (soap > 0.01) {
    const u = (p - S.soap[0]) / (S.soap[1] - S.soap[0]);
    const rub = Math.sin(p * TAU * 26);
    if (u < 0.4) {
      const held: PoseSpec = { ...body, upperarm_l: hang('l', 25, 22), lowerarm_l: elbow('l', 35, 40), hand_l: wrist('l', 10) };
      const along = 0.15 + 0.6 * (0.5 + 0.5 * Math.sin((u / 0.4) * Math.PI * 3));
      const on = along < 0.5 ? pointOn(held, 'upperarm_l', [-0.14 * (along / 0.5) * 2, -0.14 * (along / 0.5) * 2, 0.05]) : pointOn(held, 'lowerarm_l', [-0.18 * (along - 0.5) * 2, -0.18 * (along - 0.5) * 2, 0.05]);
      arms = { ...rest, upperarm_l: held.upperarm_l, lowerarm_l: held.lowerarm_l, hand_l: held.hand_l, ...armIK('r', lerpV(pointOn({ ...body, ...rest }, 'hand_r', [0, 0, 0]), on, soap), body, { palm: true, pole: [0.6, -0.8, -0.3], pronate: 60, hand: wrist('r', 10, 0) }) };
    } else if (u < 0.8) {
      const held: PoseSpec = { ...body, upperarm_r: hang('r', 25, 22), lowerarm_r: elbow('r', 35, 40), hand_r: wrist('r', 10) };
      const along = 0.15 + 0.6 * (0.5 + 0.5 * Math.sin(((u - 0.4) / 0.4) * Math.PI * 3));
      const on = along < 0.5 ? pointOn(held, 'upperarm_r', [0.14 * (along / 0.5) * 2, -0.14 * (along / 0.5) * 2, 0.05]) : pointOn(held, 'lowerarm_r', [0.18 * (along - 0.5) * 2, -0.18 * (along - 0.5) * 2, 0.05]);
      arms = { ...rest, upperarm_r: held.upperarm_r, lowerarm_r: held.lowerarm_r, hand_r: held.hand_r, ...armIK('l', lerpV(pointOn({ ...body, ...rest }, 'hand_l', [0, 0, 0]), on, soap), body, { palm: true, pole: [-0.6, -0.8, -0.3], pronate: 60, hand: wrist('l', 10, 0) }) };
    } else {
      const chest = (sx: number): Vec => pointOn(body, 'spine_03', [sx * 0.07 + 0.03 * rub, -0.05 + 0.05 * Math.cos(p * TAU * 26), 0.14]);
      arms = {
        ...rest,
        ...armIK('l', lerpV(pointOn({ ...body, ...rest }, 'hand_l', [0, 0, 0]), chest(-1), soap), body, { palm: true, pole: [-0.8, -0.8, -0.2], pronate: 70, hand: wrist('l', 10, 0) }),
        ...armIK('r', lerpV(pointOn({ ...body, ...rest }, 'hand_r', [0, 0, 0]), chest(1), soap), body, { palm: true, pole: [0.8, -0.8, -0.2], pronate: 70, hand: wrist('r', 10, 0) }),
      };
    }
  }
  return { ...body, ...legs, ...arms };
}

/**
 * A guitar held across the body (characters.ts `writeHeld` draws it from where its body rests, `GUITAR`, to
 * the left palm): its body against the belly a little to the right (`GUITAR_BODY`, m from the spine's base:
 * to the right, up, out in front), the neck out to the left in the plane of the body, tilted up 30° and
 * angled 13° forward (`GUITAR_NECK`), the left hand on it `reach` m along (about the third fret: 0.55 m on
 * a guitar, 0.27 on a ukulele); the right forearm over the body's upper edge, the hand strumming over the
 * sound hole.
 */
const GUITAR_BODY: Vec = [0.08, -0.03, 0.17];
const GUITAR_NECK: Vec = [-0.844, 0.5, 0.195];
export const GUITAR_REACH = { long: 0.55, short: 0.27 };
/** A short one (a ukulele) is held this much higher, up at the chest (characters.ts `GUITAR.upShort`). */
const GUITAR_UP_SHORT = 0.12;

/** Playing a guitar (`reach`: the neck's length to the left hand; held `up` m higher): two bars of down-up strums, the head bent to the strings, nodding, the left hand shifting along the neck. */
function strumPose(reach: number, up = 0): (p: number) => PoseSpec {
  return (p) => {
    const nod = Math.sin(p * TAU * 2);
    const body = stand({ breath: p, shift: 0.3 + 0.1 * Math.sin(p * TAU), lean: 4, twist: 4, head: [18 + 3 * nod, -14, -4] });
    const base = pointOn(body, 'spine_01', [0, 0, 0]);
    const b: Vec = [base[0] + GUITAR_BODY[0], base[1] + GUITAR_BODY[1] + up, base[2] + GUITAR_BODY[2]];
    // The fretting hand moves up the neck and back once a loop (a chord change).
    const fret = reach * (1 - 0.12 * bump(p, 0.45, 0.95, 0.1));
    const l: Vec = [b[0] + GUITAR_NECK[0] * fret, b[1] + GUITAR_NECK[1] * fret - 0.02, b[2] + GUITAR_NECK[2] * fret + 0.04];
    // Down-up strums, four a loop: over the sound hole, a hand's width out from the face.
    const strum = Math.sin(p * TAU * 4);
    const r: Vec = [b[0] - 0.05, b[1] + 0.02 + 0.06 * strum, b[2] + 0.09];
    return {
      ...body,
      ...armIK('l', l, body, { palm: true, pole: [-0.5, -1, -0.2], pronate: -70, hand: wrist('l', 25, 10) }),
      ...armIK('r', r, body, { palm: true, pole: [0.7, 0.3, -0.6], pronate: 70, hand: wrist('r', 10 + 12 * strum, 0) }),
    };
  };
}

/**
 * The vibration plate (`lazy.vibePlate`, pack_lazy.py): stood on its deck (content `stand`, 0.12 m up) at the
 * middle, facing its column; its grips, bars running fore and aft, 0.26 m to either side, 0.96 m over the deck and
 * 0.12 m ahead (`PLATE_GRIP`, m: across, up from the deck, ahead). A size per height (characters.ts `GRIPPED`).
 */
const PLATE_GRIP: Vec = [0.26, 0.96, 0.12];
export const PLATE_HOLD: Size[] = sizes('plateHold', SHARES_177, (s) => ({ s, to: 0 }));

/** Standing on the plate holding both grips, the knees soft, the weight shifting a little while it shakes. */
function platePose(fit: Fit): (p: number) => PoseSpec {
  const { s } = fit;
  return (p) => {
    const body = stand({ shift: 0.25 * Math.sin(p * TAU), breath: p, lean: 3, head: [6, 3 * Math.sin(p * TAU), 0] });
    const grip = (sx: number): Vec => [(sx * PLATE_GRIP[0]) / s, PLATE_GRIP[1] / s, PLATE_GRIP[2] / s];
    return {
      ...body,
      ...armIK('l', grip(-1), body, { palm: true, pole: [-0.5, -1, -0.4], pronate: 10, hand: wrist('l', 0, 0) }),
      ...armIK('r', grip(1), body, { palm: true, pole: [0.5, -1, -0.4], pronate: 10, hand: wrist('r', 0, 0) }),
    };
  };
}

/** Warming up at a fire (`warmHands`): seconds round, and the hands held out to it (model space, m: across, up, out). */
const WARM_S = 7;
const WARM_OUT: Vec = [0.17, 1.0, 0.45];

/**
 * Warming up standing at a fire (a fire pit, `ACTIONS.relax`): leant a little towards it, the hands held out over
 * it, palms to the flames, turned now and then; then brought together and rubbed, and held out again.
 */
function warmPose(p: number): PoseSpec {
  const rub = bump(p, 0.5, 0.86, 0.1);
  const turn = Math.sin(p * TAU * 2) * (1 - rub);
  const scrub = Math.sin(p * TAU * 14) * rub;
  const body = stand({ shift: 0.4 * Math.sin(p * TAU), breath: p * 2, lean: 7 - 3 * rub, head: [14 - 6 * rub, 4 * Math.sin(p * TAU), 0] });
  const out = (sx: number): Vec => [sx * (WARM_OUT[0] + 0.02 * turn * sx), WARM_OUT[1] + 0.01 * turn, WARM_OUT[2]];
  const together = (sx: number): Vec => [sx * 0.035, 1.07 + 0.03 * scrub * sx, 0.3 + 0.01 * scrub];
  const at = (sx: number) => lerpV(out(sx), together(sx), rub);
  return {
    ...body,
    ...armIK('l', at(-1), body, { pole: [-0.4, -1, -0.3], pronate: 75 - 75 * rub + 12 * turn, hand: wrist('l', -35 + 35 * rub, 0) }),
    ...armIK('r', at(1), body, { pole: [0.4, -1, -0.3], pronate: 75 - 75 * rub - 12 * turn, hand: wrist('r', -35 + 35 * rub, 0) }),
  };
}

export function useClips(): ClipDef[] {
  return [
    ...EYEPIECE.map(({ clip, fit }) => cycle(clip, { seconds: 6, loop: true }, eyepiecePose(fit))),
    ...[...PICK_APPLE, ...PICK_LEMON].map(({ clip: name, fit }) => clip(name, { seconds: PICK_S, loop: false }, pickKeys(fit))),
    cycle('shower', { seconds: SHOWER_S, loop: true, hands: 'open' }, showerPose(SHOWER_TURN, 0)),
    cycle('showerChild', { seconds: SHOWER_S, loop: true, hands: 'open' }, showerPose(0, SHOWER_TWIST)),
    cycle('strum', { seconds: 2, loop: true }, strumPose(GUITAR_REACH.long)),
    cycle('strumShort', { seconds: 2, loop: true }, strumPose(GUITAR_REACH.short, GUITAR_UP_SHORT)),
    cycle('warmHands', { seconds: WARM_S, loop: true, hands: 'open' }, warmPose),
    ...PLATE_HOLD.map(({ clip, fit }) => cycle(clip, { seconds: 3, loop: true }, platePose(fit))),
  ];
}
