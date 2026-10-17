/**
 * Sitting: the seated loop, talking and desk work seated, sitting down and standing up, eating
 * (also the seated upper-body layer) and a sip from a cup.
 *
 * The seated pelvis sits where a 0.45 m seat puts it on the reference body (`seatDrop` above the
 * seat; the runtime fits every clip's seat to each body and seat with `measureSit` on `sit`),
 * back from the standing place so a resident stands in front of the seat and sits back onto it.
 * Every seated clip shares that pelvis, the thighs about level and the feet flat on the floor.
 */

import { clip, cycle, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { breathe, bump, hangArms, stand, TAU } from './idle';
import { armIK, footIK, wrist } from './ik';

/** Pelvis offset when seated (reference skeleton), and the left foot's offset from its rest place. */
export const SEAT_MOVE: Vec = [0, -0.348, -0.19];
const SEAT_FEET: Vec = [-0.02, 0, 0.25];

export interface SeatOptions {
  breath?: number;
  depth?: number;
  /** Back bent forward (deg) and turned (deg, + to the left); the head's own nod / turn / tilt. */
  lean?: number;
  twist?: number;
  head?: Vec;
  /** Pelvis offset (default seated) and the feet's offset (default seated). */
  move?: Vec;
  feet?: Vec;
  /** Pelvis tilt (deg, + forward). */
  tilt?: number;
}

/** The seated body: pelvis on the seat, back upright, feet on the floor (leg IK). */
export function seatedBody(o: SeatOptions = {}): PoseSpec {
  const b = breathe(o.breath ?? 0, o.depth ?? 0.8);
  const lean = o.lean ?? 0;
  const twist = o.twist ?? 0;
  const head = o.head ?? [0, 0, 0];
  const m = o.move ?? SEAT_MOVE;
  const tilt = o.tilt ?? -6;
  const body: PoseSpec = {
    move: [m[0], m[1] + (b.move?.[1] ?? 0), m[2]],
    pelvis: [tilt, 0, 0],
    spine_01: [-tilt * 0.7 + lean * 0.4, twist * 0.2, 0],
    spine_02: [-tilt * 0.3 + lean * 0.35 + (b.spine_02?.[0] ?? 0), twist * 0.35, 0],
    spine_03: [lean * 0.25 + (b.spine_03?.[0] ?? 0), twist * 0.45, 0],
    clavicle_l: b.clavicle,
    clavicle_r: [0, 0, -(b.clavicle?.[2] ?? 0)],
    neck_01: [-lean * 0.3 + (b.neck_01?.[0] ?? 0) + head[0] * 0.3, -twist * 0.3 + head[1] * 0.35, head[2] * 0.3],
    Head: [-lean * 0.3 + (b.Head?.[0] ?? 0) + head[0] * 0.7, -twist * 0.5 + head[1] * 0.65, head[2] * 0.7],
  };
  const f = o.feet ?? SEAT_FEET;
  return { ...body, ...footIK('l', f, body, { foot: [0, -6, 0] }), ...footIK('r', [-f[0], f[1], f[2]], body, { foot: [0, 6, 0] }) };
}

/** Hands resting on the thighs, palms down. */
function lap(body: PoseSpec, dl: Vec = [0, 0, 0], dr: Vec = [0, 0, 0]): PoseSpec {
  return {
    ...armIK('l', [-0.13 + dl[0], 0.665 + dl[1], 0.04 + dl[2]], body, { palm: true, pole: [-0.3, -0.5, -1], pronate: 80, hand: wrist('l', 10, 0) }),
    ...armIK('r', [0.13 + dr[0], 0.665 + dr[1], 0.04 + dr[2]], body, { palm: true, pole: [0.3, -0.5, -1], pronate: 80, hand: wrist('r', 10, 0) }),
  };
}

function sitPose(p: number): PoseSpec {
  const body = seatedBody({ breath: p, depth: 1, head: [2 * Math.sin(p * TAU + 1), 4 * Math.sin(p * TAU), 0] });
  const s = 0.004 * Math.sin(p * TAU);
  return { ...body, ...lap(body, [0, s, 0], [0, s, 0]) };
}

/** Seated talk: the hands lift off the thighs into gestures in turn, the back and head follow. */
function sitTalkPose(p: number): PoseSpec {
  const g1 = bump(p, 0.04, 0.36, 0.1);
  const g2 = bump(p, 0.38, 0.66, 0.1);
  const g3 = bump(p, 0.68, 0.98, 0.1);
  const beat = Math.sin(p * TAU * 6);
  const body = seatedBody({ breath: p, lean: 3 + 3 * g2, twist: 5 * g1 - 5 * g3, head: [3 * g2 + 2 * Math.max(0, beat) * (g1 + g3), 5 * g3 - 6 * g1, 3 * g1 - 3 * g3] });
  const l = lap(body);
  const r = lap(body);
  const out: PoseSpec = { ...body, ...l, ...r };
  const mixArm = (side: 'l' | 'r', at: Vec, pron: number, w: number) => {
    if (w <= 0.001) return {};
    const rest: Vec = side === 'l' ? [-0.13, 0.665, 0.04] : [0.13, 0.665, 0.04];
    const t: Vec = [rest[0] + (at[0] - rest[0]) * w, rest[1] + (at[1] - rest[1]) * w, rest[2] + (at[2] - rest[2]) * w];
    return armIK(side, t, body, { palm: true, pole: [side === 'l' ? -0.4 : 0.4, -0.6, -0.7], pronate: 80 + (pron - 80) * w, hand: wrist(side, 10 - 25 * w, 0) });
  };
  Object.assign(out, mixArm('r', [0.24 + 0.02 * Math.sin(p * TAU * 3), 0.84 + 0.02 * beat * g1, 0.2], -60, g1));
  Object.assign(out, mixArm('l', [-0.14 - 0.03 * Math.sin(p * TAU * 3), 0.86, 0.22], -40, g2));
  Object.assign(out, mixArm('r', [0.14 + 0.03 * Math.sin(p * TAU * 3), 0.86, 0.22], -40, g2 * (1 - g1)));
  Object.assign(out, mixArm('l', [-0.2, 0.86 + 0.03 * beat * g3, 0.22], 10, g3));
  return out;
}

/**
 * Forearms on a desk at 0.72 m, fingers busy on keys: small alternating hand bobs, an occasional look down.
 * The hands `z` m ahead of the standing place (0.48 m ahead of the hips) and `y` m high, leaning in `lean` degrees.
 */
function deskPose(p: number, z = 0.29, y = 0.76, lean = 10): PoseSpec {
  const glance = bump(p, 0.55, 0.9, 0.12);
  const body = seatedBody({ breath: p, depth: 0.6, lean, head: [4 + 8 * glance, 3 * Math.sin(p * TAU), 0], tilt: -2 });
  const tap = (k: number) => 0.006 * Math.max(0, Math.sin(p * TAU * 5 + k));
  return {
    ...body,
    ...armIK('l', [-0.15 + 0.01 * Math.sin(p * TAU), y + tap(0), z], body, { pole: [-0.5, -1, -0.4], pronate: 85, hand: wrist('l', -8 + 8 * Math.sin(p * TAU * 5), 0) }),
    ...armIK('r', [0.15 + 0.012 * Math.sin(p * TAU + 2), y + tap(2.4), z], body, { pole: [0.5, -1, -0.4], pronate: 85, hand: wrist('r', -8 + 8 * Math.sin(p * TAU * 5 + 2.4), 0) }),
  };
}

/** At a shallow desk with its chair built in close (a writing desk's top 0.3 m ahead of the hips, about 0.77 m high): the hands nearer and a little higher. */
function nearDeskPose(p: number): PoseSpec {
  return deskPose(p, 0.11, 0.78, 8);
}

/**
 * At a table further off than the chair could be pulled in to (drawer pedestals or a drawer box where the knees
 * would go: `TUCK` in characters.ts): the hands further out, leaning in more.
 */
function farDeskPose(p: number): PoseSpec {
  return deskPose(p, 0.43, 0.77, 22);
}

/** Standing in front of the seat (the start of sitDown, the end of standUp). */
function standing(): PoseSpec {
  return { ...stand(), ...hangArms() };
}

/** Halfway down or up: hips back and low over the feet, chest well forward, arms forward for balance. */
function crouchOver(move: Vec, lean: number, feet: Vec, reach = 0.25): PoseSpec {
  const body = seatedBody({ move, feet, lean, tilt: lean * 0.45, head: [-lean * 0.45, 0, 0] });
  return {
    ...body,
    ...armIK('l', [-0.16, 0.72 + reach * 0.3, 0.2 + reach], body, { pole: [-0.4, -0.7, -0.6], pronate: 70, hand: wrist('l', 10, 0) }),
    ...armIK('r', [0.16, 0.72 + reach * 0.3, 0.2 + reach], body, { pole: [0.4, -0.7, -0.6], pronate: 70, hand: wrist('r', 10, 0) }),
  };
}

function seated(): PoseSpec {
  return sitPose(0);
}

/** Hand to mouth (eating; the seated upper-body layer too): the left hand holds the food low in front. */
function eatPose(t: number, at: Vec, head: Vec, lean: number): PoseSpec {
  const body = stand({ shift: 0.25, breath: t * 0.5, lean, head });
  return {
    ...body,
    ...armIK('l', [-0.1, 1.02, 0.26], body, { palm: true, pole: [-0.4, -1, -0.3], pronate: -40, hand: wrist('l', 0, 0) }),
    ...armIK('r', at, body, { palm: true, pole: [0.5, -1, 0.1], pronate: -30, hand: wrist('r', 20, 10) }),
  };
}

export function seatedClips(): ClipDef[] {
  const low: Vec = [0.11, 1.03, 0.27];
  const mouth: Vec = [0.035, 1.5, 0.17];
  return [
    cycle('sit', { seconds: 50 / 30, loop: true }, sitPose),
    cycle('sitTalk', { seconds: 88 / 30, loop: true }, sitTalkPose),
    // Sitting down: a dip, the hips go back and down over the planted feet as the chest leans out,
    // the seat takes the weight, the back comes up and the feet slide forward.
    clip('sitDown', { seconds: 40 / 30, loop: false }, [
      [0, standing],
      [0.14, () => crouchOver([0, -0.03, 0.0], 8, [0, 0, 0], 0.02), 'smooth'],
      [0.48, () => crouchOver([0, -0.24, -0.12], 34, [0, 0, 0.02], 0.2), 'smooth'],
      [0.7, () => crouchOver([0, -0.345, -0.18], 24, [-0.01, 0, 0.08], 0.12), 'out'],
      [0.86, () => ({ ...seatedBody({ lean: -3, head: [2, 0, 0], feet: [-0.02, 0, 0.21] }), ...lap(seatedBody({ lean: -3, feet: [-0.02, 0, 0.21] }), [0, 0.01, 0.03], [0, 0.01, 0.03]) }), 'smooth'],
      [1, seated, 'smooth'],
    ]),
    // Standing up: the feet draw back, the chest swings forward over them, a push up and a settle.
    clip('standUp', { seconds: 32 / 30, loop: false }, [
      [0, seated],
      [0.2, () => crouchOver([0, -0.345, -0.18], 18, [-0.01, 0, 0.1], 0.06), 'smooth'],
      [0.42, () => crouchOver([0, -0.3, -0.13], 36, [0, 0, 0.03], 0.22), 'smooth'],
      [0.72, () => crouchOver([0, -0.08, -0.03], 14, [0, 0, 0], 0.08), 'out'],
      [0.88, () => ({ ...stand({ lean: -2, head: [-2, 0, 0] }), ...hangArms(6, 8, 18) }), 'smooth'],
      [1, standing, 'smooth'],
    ]),
    cycle('sitHands', { seconds: 50 / 30, loop: true }, (p) => deskPose(p)),
    cycle('sitHandsNear', { seconds: 50 / 30, loop: true }, nearDeskPose),
    cycle('sitHandsFar', { seconds: 50 / 30, loop: true }, farDeskPose),
    clip('eat', { seconds: 40 / 30, loop: false }, [
      [0, () => eatPose(0, low, [0, 0, 0], 0)],
      [0.32, () => eatPose(0.32, mouth, [-4, 0, 0], 4), 'smooth'],
      [0.4, () => eatPose(0.4, [mouth[0], mouth[1] + 0.01, mouth[2] - 0.01], [-6, 0, 0], 5), 'out'],
      [0.6, () => eatPose(0.6, [0.06, 1.4, 0.2], [6, 0, 2], 3), 'smooth'],
      [0.82, () => eatPose(0.82, [0.1, 1.08, 0.27], [4, 0, 0], 1), 'smooth'],
      [1, () => eatPose(1, low, [0, 0, 0], 0), 'smooth'],
    ]),
    // A sip from a cup (phase 2): cup up from the chest, the head tips back for the sip, back down.
    clip('drink', { seconds: 2, loop: false }, [
      [0, () => eatPose(0, [0.12, 1.15, 0.27], [0, 0, 0], 0)],
      [0.3, () => eatPose(0.3, [0.05, 1.49, 0.17], [-4, 0, 0], 2), 'smooth'],
      [0.42, () => eatPose(0.42, [0.05, 1.53, 0.15], [-14, 0, 0], -3), 'smooth'],
      [0.6, () => eatPose(0.6, [0.05, 1.53, 0.15], [-16, 0, 0], -4), 'smooth'],
      [0.75, () => eatPose(0.75, [0.08, 1.3, 0.25], [3, 0, 0], 1), 'smooth'],
      [1, () => eatPose(1, [0.12, 1.15, 0.27], [0, 0, 0], 0), 'smooth'],
    ]),
  ];
}

