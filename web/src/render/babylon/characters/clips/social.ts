/**
 * Socials' own body language, standing on the relaxed stance (`stand`, the feet planted): the
 * socials the runtime tells apart (characters.ts `GESTURES`) each have a gesture of their own instead
 * of the chat's `talk`, short and looping for as long as a turn of the conversation lasts. Then the
 * reactions: laughing, a shrug, being turned down (`rebuffed`) and turning someone down (`refuse`).
 * And the phone held in front (`phoneText`: texting and scrolling; a call at the ear is `phone`, in
 * idle.ts). The hands are placed with arm IK on the reference skeleton (the shoulders at 1.39 m, the
 * chin at about 1.53 m, the front of the chest about 0.12 m before the spine); an arm not gesturing
 * hangs (`hangArms`), blended into the gesture by its weight, so nothing jumps as one fades in and out.
 */

import { cycle, mix, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { bump, hangArms, stand, TAU, type StandOptions } from './idle';
import { armIK, lerpV, smooth, wrist, type ArmOptions } from './ik';

/** A hand placed by IK (`at`: the palm centre), weighted `w` (default 1) against the hanging arm. */
type Hand = ArmOptions & { at: Vec; w?: number };

/** One side's bones of a pose. */
function side(spec: PoseSpec, s: 'l' | 'r'): PoseSpec {
  return Object.fromEntries(Object.entries(spec).filter(([k]) => k.endsWith(`_${s}`))) as PoseSpec;
}

/** Both arms on a body pose: each placed by IK, faded in by its weight from the hanging arm (`rest`). */
function arms(body: PoseSpec, l: Hand, r: Hand, rest: PoseSpec = hangArms(5, 7, 16)): PoseSpec {
  const one = (s: 'l' | 'r', hand: Hand): PoseSpec => {
    const w = hand.w ?? 1;
    const own = side(rest, s);
    if (w <= 0.001) return own;
    const ik = armIK(s, hand.at, body, { palm: true, pole: [s === 'l' ? -0.4 : 0.4, -1, -0.2], ...hand });
    return w >= 0.999 ? ik : mix(own, ik, w);
  };
  return { ...body, ...one('l', l), ...one('r', r) };
}

/** Two gestures in turn (weights `a` and `b`, not overlapping): the weight of either, and how far from the first to the second. */
function turn(a: number, b: number): { w: number; k: number } {
  const w = a + b;
  return { w, k: w > 1e-6 ? b / w : 0 };
}

/** The shoulders raised (+) or dropped (-), degrees (the clavicles, on top of the body's breathing). */
function shoulders(body: PoseSpec, up: number): PoseSpec {
  const l = body.clavicle_l ?? [0, 0, 0];
  const r = body.clavicle_r ?? [0, 0, 0];
  return { ...body, clavicle_l: [l[0], l[1], l[2] - up], clavicle_r: [r[0], r[1], r[2] + up] };
}

/** An open hand, the fingers a little curled (gestures), and the fingers curled like claws (the prank's "boo"). */
const OPEN: PoseSpec = { fingers_01: [0, 0, 12], fingers_02: [0, 0, 10], thumb_01: [4, 0, 4] };
const CLAW: PoseSpec = { fingers_01: [0, 0, 35], fingers_02: [0, 0, 60], thumb_01: [10, -3, 10] };

function body(p: number, o: StandOptions): PoseSpec {
  return stand({ breath: p, ...o });
}

const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

/** Compliment: the right hand opens out to the other, palm up ("you look lovely"), then two light claps of delight. */
function complimentPose(p: number): PoseSpec {
  const offer = bump(p, 0.02, 0.5, 0.12);
  const clap = bump(p, 0.52, 0.96, 0.1);
  const beat = Math.max(0, Math.sin((p - 0.56) * TAU * 5)) * clap;
  const r = turn(offer, clap);
  const b = body(p, { shift: 0.4 * Math.sin(p * TAU), lean: 2, twist: 8 * offer, head: [4 * clap - 3 * offer, -6 * offer, 9 * offer] });
  return {
    ...arms(
      b,
      { at: [-0.03 - 0.05 * beat, 1.22, 0.3], w: clap, pronate: 5, hand: wrist('l', -5) },
      { at: lerpV([0.26, 1.13 + 0.03 * Math.sin(p * TAU * 3), 0.36], [0.03 + 0.05 * beat, 1.22, 0.3], r.k), w: r.w, pronate: lerp(-80, 5, r.k), hand: wrist('r', lerp(-15, -5, r.k)) },
    ),
    ...OPEN,
  };
}

/** Deep talk: leaning in, the fingertips together before the belly, then the chin on the right hand, the left arm across under its elbow (thinking); slow nods. */
function deepTalkPose(p: number): PoseSpec {
  const think = bump(p, 0.5, 0.98, 0.14);
  const tap = 0.012 * Math.sin(p * TAU * 6) * (1 - think);
  const b = body(p, { shift: -0.35 + 0.15 * Math.sin(p * TAU), lean: 5, fwd: 0.015, depth: 1.3, head: [6 + 5 * Math.sin(p * TAU * 2) * (1 - think) - 6 * think, 4 * think, 6 * think] });
  return {
    ...arms(
      b,
      { at: lerpV([-0.03, 1.1, 0.25 + tap], [0.12, 1.12, 0.15], think), pole: [-0.3, -1, 0], pronate: 10 + 60 * think, hand: wrist('l', -20 + 45 * think) },
      { at: lerpV([0.03, 1.1, 0.25 + tap], [0.03, 1.44, 0.17], think), pole: [0.5, -1, 0.1], pronate: 10 - 30 * think, hand: wrist('r', -25 - 30 * think) },
    ),
    ...OPEN,
  };
}

/** Flirting: the weight on one hip, the left hand on it, the right running through the hair behind the ear; a coy tilt of the head. */
function flirtPose(p: number): PoseSpec {
  const hair = bump(p, 0.08, 0.62, 0.16);
  const b = body(p, { shift: 0.75 + 0.2 * Math.sin(p * TAU), twist: -6, head: [-2 + 4 * hair, 6 - 4 * hair, 12 + 4 * Math.sin(p * TAU)] });
  return {
    ...arms(
      b,
      { at: [-0.25, 0.98, -0.02], pole: [-1, -0.2, -0.6], pronate: 60, hand: wrist('l', -30, 20) },
      { at: [0.13, 1.6 - 0.04 * Math.sin(p * TAU * 2), -0.03], w: hair, pole: [1, -0.2, -0.3], pronate: -40, hand: wrist('r', 20) },
    ),
    ...OPEN,
  };
}

/** Sweet talk: the hands clasped at the heart, leaning in and swaying; then the right reaches out softly towards the other's arm. */
function sweetTalkPose(p: number): PoseSpec {
  const reach = bump(p, 0.52, 0.92, 0.14);
  const sway = Math.sin(p * TAU);
  const b = body(p, { shift: 0.5 * sway, lean: 5, fwd: 0.02, twist: 4 * sway, head: [4, -5 * sway, 10 - 3 * sway] });
  return {
    ...arms(
      b,
      { at: [-0.02, 1.2, 0.2], pole: [-0.5, -1, -0.1], pronate: 10, hand: wrist('l', 10) },
      { at: lerpV([0.03, 1.2, 0.22], [0.2, 1.22, 0.45], reach), pole: [0.5, -1, -0.1], pronate: 10 + 40 * reach, hand: wrist('r', 10 - 10 * reach) },
    ),
    ...OPEN,
  };
}

/** Arguing: leaning in, both palms up and shaken ("what?"), then the right hand chopping down hard twice. */
function arguePose(p: number): PoseSpec {
  const what = bump(p, 0.02, 0.48, 0.1);
  const chop = bump(p, 0.5, 0.98, 0.08);
  const down = Math.abs(Math.sin((p - 0.5) * TAU * 2));
  const shake = Math.sin(p * TAU * 8) * what;
  const r = turn(what, chop);
  const b = body(p, { shift: 0.25 * Math.sin(p * TAU), lean: 7, fwd: 0.02, twist: -8 * chop, head: [-6 + 6 * chop * down, 6 * shake, 0] });
  return {
    ...arms(
      b,
      { at: [-0.24 - 0.02 * shake, 1.18, 0.32], w: what, pronate: -80, hand: wrist('l', -20) },
      { at: lerpV([0.24 + 0.02 * shake, 1.18, 0.32], [0.12, 1.32 - 0.24 * down, 0.38], r.k), w: r.w, pronate: lerp(-80, 0, r.k), hand: wrist('r', lerp(-20, 0, r.k)) },
    ),
    ...OPEN,
  };
}

/** An insult: the chin up, the left hand on the hip, the right flicking out at the other, dismissive. */
function insultPose(p: number): PoseSpec {
  const flick = bump(p, 0.15, 0.7, 0.1);
  const snap = Math.sin(Math.PI * smooth((p - 0.3) / 0.25));
  const b = body(p, { shift: -0.6, lean: -3, twist: 10 * flick, head: [-10, -10 + 6 * flick, -6] });
  return {
    ...arms(
      b,
      { at: [-0.25, 0.98, -0.02], pole: [-1, -0.2, -0.6], pronate: 60, hand: wrist('l', -30, 20) },
      { at: [0.24 + 0.08 * snap, 1.22, 0.38 + 0.06 * snap], w: flick, pronate: 40 + 60 * snap, hand: wrist('r', -20 - 30 * snap) },
    ),
    ...OPEN,
  };
}

/** Apologising: the head bowed, the right hand flat on the heart; then both hands open low, palms up ("I'm sorry"), the shoulders up. */
function apologizePose(p: number): PoseSpec {
  const heart = bump(p, 0.02, 0.5, 0.12);
  const open = bump(p, 0.52, 0.98, 0.14);
  const r = turn(heart, open);
  const b = shoulders(body(p, { shift: 0.3 * Math.sin(p * TAU), lean: 4, head: [16 - 4 * open, 0, 6 * open] }), 6 * open);
  return {
    ...arms(
      b,
      { at: [-0.3, 1.04, 0.22], w: open, pronate: -80, hand: wrist('l', -15) },
      { at: lerpV([-0.05, 1.28, 0.14], [0.3, 1.04, 0.22], r.k), w: r.w, pole: [lerp(0.6, 0.4, r.k), -1, lerp(-0.3, -0.2, r.k)], pronate: lerp(20, -80, r.k), hand: wrist('r', lerp(15, -15, r.k)) },
    ),
    ...OPEN,
  };
}

/** Telling a joke: leaning in, the hands shaping the story; then the punchline, both arms thrown out wide, leaning back. */
function jokePose(p: number): PoseSpec {
  const tell = bump(p, 0.02, 0.5, 0.1);
  const punch = bump(p, 0.55, 0.95, 0.08);
  const s = Math.sin(p * TAU * 4);
  const t = turn(tell, punch);
  const b = body(p, { shift: 0.3 * Math.sin(p * TAU), lean: 6 * tell - 7 * punch, fwd: 0.02 * tell - 0.015 * punch, twist: 6 * s * tell, head: [5 * tell - 12 * punch, 6 * s * tell, 4 * punch] });
  return {
    ...arms(
      b,
      { at: lerpV([-0.13 - 0.03 * s, 1.15, 0.3], [-0.4, 1.3, 0.22], t.k), w: t.w, pronate: lerp(-40, -80, t.k), hand: wrist('l', -15) },
      { at: lerpV([0.13 - 0.03 * s, 1.15, 0.3], [0.4, 1.3, 0.22], t.k), w: t.w, pronate: lerp(-40, -80, t.k), hand: wrist('r', -15) },
    ),
    ...OPEN,
  };
}

/** Introducing oneself: the right hand to one's own chest ("I'm …"), then held out for a handshake, thumb up, pumping. */
function introducePose(p: number): PoseSpec {
  const me = bump(p, 0.02, 0.42, 0.1);
  const shake = bump(p, 0.48, 0.96, 0.12);
  const pump = 0.025 * Math.sin((p - 0.5) * TAU * 6) * shake;
  const r = turn(me, shake);
  const b = body(p, { shift: 0.3 * Math.sin(p * TAU), lean: 2 + 4 * shake, head: [6 * shake + 3 * me, 0, 4 * me] });
  return {
    ...arms(
      b,
      { at: [0, 0, 0], w: 0 },
      { at: lerpV([0.01, 1.25, 0.15], [0.12, 1.05 + pump, 0.42], r.k), w: r.w, pole: [lerp(0.6, 0.4, r.k), -1, lerp(-0.3, -0.2, r.k)], pronate: lerp(20, 0, r.k), hand: wrist('r', lerp(10, -5, r.k)) },
    ),
    ...OPEN,
  };
}

/** Asking to be partners: the hands clasped low, the head down, shy; then both held out, palms up, the head up, hoping. */
function askPartnerPose(p: number): PoseSpec {
  const ask = bump(p, 0.5, 0.96, 0.14);
  const b = body(p, { shift: 0.45 * Math.sin(p * TAU * 2) * (1 - ask), lean: 2 + 3 * ask, head: [12 - 16 * ask, 0, 8 * ask] });
  return {
    ...arms(
      b,
      { at: lerpV([-0.02, 1.0, 0.16], [-0.14, 1.1, 0.36], ask), pole: [-0.4, -1, -0.1], pronate: -30 - 50 * ask, hand: wrist('l', 0) },
      { at: lerpV([0.02, 1.0, 0.17], [0.14, 1.1, 0.36], ask), pole: [0.4, -1, -0.1], pronate: -30 - 50 * ask, hand: wrist('r', 0) },
    ),
    ...OPEN,
  };
}

/** Breaking up: the right hand rubbing the brow, the head down; then both hands, palms down, sweeping apart ("it's over"). */
function breakUpPose(p: number): PoseSpec {
  const brow = bump(p, 0.02, 0.46, 0.12);
  const cut = bump(p, 0.52, 0.98, 0.1);
  const apart = smooth((p - 0.58) / 0.22);
  const r = turn(brow, cut);
  const b = body(p, { shift: -0.4, lean: 3, head: [14 * brow - 4 * cut, 0, 0] });
  return {
    ...arms(
      b,
      { at: [-0.06 - 0.26 * apart, 1.12, 0.34], w: cut, pronate: 85, hand: wrist('l', -5) },
      { at: lerpV([0.06 + 0.02 * Math.sin(p * TAU * 6), 1.58, 0.16], [0.06 + 0.26 * apart, 1.12, 0.34], r.k), w: r.w, pole: [lerp(0.8, 0.4, r.k), -1, -0.2], pronate: lerp(-20, 85, r.k), hand: wrist('r', lerp(20, -5, r.k)) },
    ),
    ...OPEN,
  };
}

/** A prank: crouched and creeping in, then up with both clawed hands by the head ("boo!"), then a laugh and a point at the other. */
function prankPose(p: number): PoseSpec {
  const creep = bump(p, 0, 0.38, 0.1);
  const boo = bump(p, 0.36, 0.66, 0.06);
  const laugh = bump(p, 0.66, 1, 0.1);
  const bounce = Math.abs(Math.sin(p * TAU * 6)) * laugh;
  const t = turn(boo, laugh);
  const b = body(p, { sink: 0.06 * creep + 0.01 * bounce, lean: 12 * creep - 6 * boo + 6 * laugh, fwd: 0.03 * boo, head: [-6 * creep - 8 * boo - 8 * laugh + 4 * bounce, 0, 0] });
  return {
    ...arms(
      b,
      { at: lerpV([-0.22, 1.55, 0.25], [-0.04, 1.08, 0.16], t.k), w: t.w, pronate: lerp(-90, 20, t.k), hand: wrist('l', lerp(-30, 0, t.k)) },
      { at: lerpV([0.22, 1.55, 0.25], [0.18, 1.25, 0.42], t.k), w: t.w, pronate: lerp(-90, 0, t.k), hand: wrist('r', lerp(-30, 0, t.k)) },
    ),
    ...mix(OPEN, CLAW, boo),
  };
}

/** Laughing: the head thrown back, the shoulders shaking, a hand on the belly; bent over once with a slap of the thigh. */
function laughPose(p: number): PoseSpec {
  const fold = bump(p, 0.45, 0.8, 0.12);
  const shake = Math.abs(Math.sin(p * TAU * 7));
  const b = shoulders(body(p, { shift: 0.3 * Math.sin(p * TAU), lean: -5 + 16 * fold + 2 * shake, head: [-14 + 18 * fold + 5 * shake, 0, 4] }), 3 * shake);
  return {
    ...arms(
      b,
      { at: [-0.2, 0.86, 0.2], w: fold, pronate: 40, hand: wrist('l', 0) },
      { at: [0.02, 1.07, 0.15], pole: [0.6, -1, -0.3], pronate: 30, hand: wrist('r', 10) },
    ),
    ...OPEN,
  };
}

/** A shrug: the shoulders up, the forearms out with the palms up, the head tilted; and down again. */
function shrugPose(p: number): PoseSpec {
  const up = bump(p, 0.1, 0.75, 0.2);
  const b = shoulders(body(p, { shift: -0.3, lean: -1, head: [0, -4 * up, 12 * up] }), 12 * up);
  return {
    ...arms(b, { at: [-0.34, 1.04, 0.2], w: up, pronate: -85, hand: wrist('l', -20) }, { at: [0.34, 1.04, 0.2], w: up, pronate: -85, hand: wrist('r', -20) }),
    ...OPEN,
  };
}

/** Turned down: the shoulders dropped, the head bowed and turned aside, a sigh; a hand rubbing the back of the neck. */
function rebuffedPose(p: number): PoseSpec {
  const neck = bump(p, 0.35, 0.85, 0.15);
  const b = shoulders(body(p, { shift: -0.7, lean: 6, depth: 2, head: [20, -14 + 4 * neck, -5] }), -6 + 3 * neck);
  return arms(
    b,
    { at: [0, 0, 0], w: 0 },
    { at: [0.06 + 0.015 * Math.sin(p * TAU * 6), 1.5, -0.05], w: neck, pole: [0.5, 0.3, 1], pronate: -60, hand: wrist('r', 40) },
    hangArms(9, 4, 12),
  );
}

/** Turning someone down (a hug, a kiss): leaning away, the head drawn back and turned aside, the right hand up, palm out, between them. */
function refusePose(p: number): PoseSpec {
  const push = 0.02 * Math.sin(p * TAU * 2);
  const b = body(p, { shift: -0.6, lean: -9, fwd: -0.04, twist: -8, head: [-6, -22 + 3 * Math.sin(p * TAU), 6] });
  return {
    ...arms(b, { at: [-0.12, 1.15, 0.16], pole: [-0.5, -1, -0.2], pronate: 30, hand: wrist('l', 10) }, { at: [0.1, 1.36, 0.42 + push], pole: [0.5, -1, -0.2], pronate: 0, hand: wrist('r', -65) }),
    ...OPEN,
  };
}

/** Texting and scrolling: the phone in both hands before the chest, the head bowed over it, the thumbs tapping and swiping. */
function phoneTextPose(p: number): PoseSpec {
  const swipe = bump(p, 0.55, 0.78, 0.06);
  const tapR = Math.max(0, Math.sin(p * TAU * 9)) * (1 - swipe);
  const tapL = Math.max(0, Math.sin(p * TAU * 9 + 2)) * (1 - swipe);
  const b = body(p, { shift: 0.35 * Math.sin(p * TAU), lean: 4, head: [26 + 2 * Math.sin(p * TAU * 2), 2 * Math.sin(p * TAU), 0] });
  return {
    ...arms(
      b,
      { at: [-0.045, 1.16, 0.27], pole: [-0.5, -1, -0.1], pronate: -55, hand: wrist('l', -10) },
      { at: [0.045, 1.16 + 0.01 * swipe, 0.27], pole: [0.5, -1, -0.1], pronate: -55, hand: wrist('r', -10) },
    ),
    fingers_01: [0, 0, 55],
    fingers_02: [0, 0, 45],
    thumb_01_l: [8 + 6 * tapL, -3, 10],
    thumb_02_l: [18 + 16 * tapL, -10, -15],
    thumb_01_r: [8 + 6 * tapR + 4 * swipe, -3, 10 + 10 * Math.sin(Math.PI * swipe)],
    thumb_02_r: [18 + 16 * tapR, -10, -15],
  };
}

export function socialClips(): ClipDef[] {
  return [
    cycle('compliment', { seconds: 3, loop: true, hands: 'open' }, complimentPose),
    cycle('deepTalk', { seconds: 4, loop: true, hands: 'open' }, deepTalkPose),
    cycle('flirt', { seconds: 3.2, loop: true, hands: 'open' }, flirtPose),
    cycle('sweetTalk', { seconds: 3.4, loop: true, hands: 'open' }, sweetTalkPose),
    cycle('argue', { seconds: 2.2, loop: true, hands: 'open' }, arguePose),
    cycle('insult', { seconds: 2.4, loop: true, hands: 'open' }, insultPose),
    cycle('apologize', { seconds: 3.4, loop: true, hands: 'open' }, apologizePose),
    cycle('joke', { seconds: 2.8, loop: true, hands: 'open' }, jokePose),
    cycle('introduce', { seconds: 3, loop: true, hands: 'open' }, introducePose),
    cycle('askPartner', { seconds: 3.2, loop: true, hands: 'open' }, askPartnerPose),
    cycle('breakUp', { seconds: 3, loop: true, hands: 'open' }, breakUpPose),
    cycle('prank', { seconds: 2.6, loop: true, hands: 'open' }, prankPose),
    cycle('laugh', { seconds: 2, loop: true, hands: 'open' }, laughPose),
    cycle('shrug', { seconds: 2.4, loop: true, hands: 'open' }, shrugPose),
    cycle('rebuffed', { seconds: 2.6, loop: true }, rebuffedPose),
    cycle('refuse', { seconds: 2.4, loop: true, hands: 'open' }, refusePose),
    cycle('phoneText', { seconds: 3, loop: true }, phoneTextPose),
  ];
}
