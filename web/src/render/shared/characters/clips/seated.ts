/**
 * Sitting: the seated loop, talking and desk work seated, sitting down and standing up, eating
 * (also the seated upper-body layer), a sip from a cup, and sitting cross-legged on the floor (down to it and up).
 *
 * The seated pelvis sits where a 0.45 m seat puts it on the reference body (`seatDrop` above the
 * seat; the runtime fits every clip's seat to each body and seat with `measureSit` on `sit`),
 * back from the standing place so a resident stands in front of the seat and sits back onto it.
 * Every seated clip shares that pelvis, the thighs about level and the feet flat on the floor.
 */

import { clip, cycle, reference, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { breathe, bump, hangArms, stand, TAU } from './idle';
import { armIK, axisAngle, footIK, lerpV, mul, sub, toEuler, world, wrist } from './ik';

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

// --- on the floor -----------------------------------------------------------------------------------------

/**
 * Sitting on the floor (characters.ts `FLOOR_SIT`), in two ways, each with its sitting down and getting up:
 * - `Floor`: cross-legged on a floor cushion (the floor cushion, the electric fireplace's hearth cushion), the
 *   ankles crossed in front, each under the other knee, the hands on the knees;
 * - `Ground`: on the ground against a tree's trunk ("Relax in the shade"), leant back against it, the legs out
 *   long and crossed at the ankles, the hands in the lap.
 * `seat`: the seat height each is made for (m; the pelvis `seatDrop` above it, as `SEAT_MOVE` on a chair's,
 * a little back from the standing place). `legs`: where the ankles go (m from the pelvis joint across and
 * ahead, and up from the floor, reference skeleton), where the knees point and how the feet turn (`sx`: -1
 * for the left). `tilt`, `lean`: the pelvis's tilt and the back's lean (deg, as `seatedBody`'s).
 */
export interface FloorSit {
  name: 'Floor' | 'Ground';
  seat: number;
  legs: { l: Vec; r: Vec; pole: (sx: number) => Vec; foot: (sx: number, k: number) => Vec };
  tilt: number;
  lean: number;
  /** The left hand: across from the pelvis joint, up from the floor, ahead (m; the right mirrors it). */
  hands: Vec;
}
export const FLOOR_SITS: readonly FloorSit[] = [
  {
    name: 'Floor',
    seat: 0.14,
    legs: {
      l: [0.12, 0.09, 0.23],
      r: [-0.12, 0.065, 0.13],
      // The knees out to the sides, the feet turned in across and rolled onto their outer edges.
      pole: (sx) => [sx, -0.1, 0.6],
      foot: (sx, k) => toEuler(mul(axisAngle([0, 1, 0], -50 * sx * k), axisAngle([0, 0, 1], -95 * sx * k))),
    },
    tilt: -10,
    lean: 9,
    hands: [-0.24, 0.25, 0.25],
  },
  {
    name: 'Ground',
    seat: 0,
    legs: {
      l: [0.03, 0.095, 0.74],
      r: [-0.03, 0.07, 0.7],
      // The knees up (nearly straight), the toes up and falling out.
      pole: (sx) => [0.1 * sx, 1, 0.3],
      foot: (sx, k) => toEuler(mul(axisAngle([0, 1, 0], 14 * sx * k), axisAngle([1, 0, 0], -72 * k))),
    },
    tilt: -14,
    lean: -12,
    hands: [-0.1, 0.24, 0.3],
  },
];

/** Its pelvis offset (as `SEAT_MOVE`). */
function floorMove(f: FloorSit): Vec {
  return [0, SEAT_MOVE[1] - (0.45 - f.seat), -0.1];
}

/** A point `at` from the pelvis joint across and ahead, `y` m above the floor, as an ankle offset of `footIK`. */
function ankleAt(body: PoseSpec, side: 'l' | 'r', at: Vec): Vec {
  const p = world(body, 'pelvis').p;
  return sub([p[0] + at[0], at[1], p[2] + at[2]], reference().joint(`foot_${side}`));
}

/**
 * The legs on the floor: as `f` has them (`k` 1), or (0) the feet flat in front, the knees up, `ahead` m in
 * front of the pelvis, or between.
 */
function floorLegs(f: FloorSit, body: PoseSpec, k: number, ahead = 0.32): PoseSpec {
  const legs: PoseSpec = {};
  for (const side of ['l', 'r'] as const) {
    const sx = side === 'l' ? -1 : 1;
    const up: Vec = [0.11 * sx, 0.08, ahead];
    const pole = lerpV([0.15 * sx, 0.2, 1], f.legs.pole(sx), k);
    Object.assign(legs, footIK(side, ankleAt(body, side, lerpV(up, f.legs[side], k)), body, { pole, foot: f.legs.foot(sx, k) }));
  }
  return legs;
}

/** The hands where `f` rests them (`wide` 1), or in nearer and lower. */
function floorHands(f: FloorSit, body: PoseSpec, wide = 1): PoseSpec {
  const p = world(body, 'pelvis').p;
  const [x, y, z] = f.hands;
  const at = (sx: number): Vec => [p[0] + sx * (0.12 + (-x - 0.12) * wide), y - 0.05 * (1 - wide), p[2] + z];
  return {
    ...armIK('l', at(-1), body, { palm: true, pole: [-0.6, -0.4, -0.7], pronate: 80, hand: wrist('l', 15, 0) }),
    ...armIK('r', at(1), body, { palm: true, pole: [0.6, -0.4, -0.7], pronate: 80, hand: wrist('r', 15, 0) }),
  };
}

/** Sitting on the floor as `f` does, breathing, the head moving a little. */
function floorPose(f: FloorSit, p: number): PoseSpec {
  const body = seatedBody({ breath: p, depth: 1, move: floorMove(f), tilt: f.tilt, lean: f.lean, head: [6 + 2 * Math.sin(p * TAU + 1), 4 * Math.sin(p * TAU), 0] });
  return { ...body, ...floorLegs(f, body, 1), ...floorHands(f, body) };
}

/** Down on the floor or up from it: the pelvis at `move`, leaning `lean`, the legs as `f`'s by `k` (or flat, `ahead`), a hand down on the floor. */
function floorBetween(f: FloorSit, move: Vec, lean: number, k: number, ahead: number, hand: number): PoseSpec {
  const body = seatedBody({ move, lean, tilt: lean * 0.3 - 6, head: [-lean * 0.3, 0, 0] });
  const p = world(body, 'pelvis').p;
  const reach: Vec = [p[0] + 0.24, 0.04 + (1 - hand) * 0.3, p[2] - 0.05 + (1 - hand) * 0.3];
  return {
    ...body,
    ...floorLegs(f, body, k, ahead),
    ...floorHands(f, body, 0.6),
    ...armIK('r', reach, body, { palm: true, pole: [0.8, -0.2, -0.5], pronate: 80, hand: wrist('r', -40 * hand, 0) }),
  };
}

/** Squatting over the feet (getting down to the floor or up from it), leaning `lean`, the arms forward. */
function squat(move: Vec, lean: number): PoseSpec {
  return crouchOver(move, lean, [0, 0, 0.04], 0.3);
}

/**
 * `sit<name>` (the loop), `sitDown<name>` (a squat, a hand down to the floor, sitting back, the legs into
 * place) and `standUp<name>` (the legs drawn in, forward over the feet into a squat and up) for `f`.
 */
function floorClips(f: FloorSit): ClipDef[] {
  const m = floorMove(f);
  const sat = () => floorPose(f, 0);
  return [
    cycle(`sit${f.name}`, { seconds: 50 / 30, loop: true }, (p) => floorPose(f, p)),
    clip(`sitDown${f.name}`, { seconds: 48 / 30, loop: false }, [
      [0, standing],
      [0.28, () => squat([0, -0.42, -0.06], 28), 'smooth'],
      [0.55, () => floorBetween(f, [m[0], m[1] + 0.02, m[2] - 0.03], 6, 0, 0.32, 1), 'smooth'],
      [0.8, () => floorBetween(f, m, (6 + f.lean) / 2, 0.6, 0.3, 0.3), 'smooth'],
      [1, sat, 'smooth'],
    ]),
    clip(`standUp${f.name}`, { seconds: 44 / 30, loop: false }, [
      [0, sat],
      [0.24, () => floorBetween(f, m, 10, 0, 0.26, 0.6), 'smooth'],
      [0.52, () => squat([0, -0.46, -0.04], 34), 'smooth'],
      [0.78, () => squat([0, -0.18, -0.02], 16), 'out'],
      [1, standing, 'smooth'],
    ]),
  ];
}

/**
 * Hand to mouth (eating, a sip from a cup; the seated upper-body layer too): the left hand holds the food low in front.
 * Also the cup held between sips (`holdCup`, clips/props.ts, at `CUP_HELD`).
 */
export function eatPose(t: number, at: Vec, head: Vec, lean: number): PoseSpec {
  const body = stand({ shift: 0.25, breath: t * 0.5, lean, head });
  return {
    ...body,
    ...armIK('l', [-0.1, 1.02, 0.26], body, { palm: true, pole: [-0.4, -1, -0.3], pronate: -40, hand: wrist('l', 0, 0) }),
    ...armIK('r', at, body, { palm: true, pole: [0.5, -1, 0.1], pronate: -30, hand: wrist('r', 20, 10) }),
  };
}

/** Where the cup is held between sips (the palm, m, reference skeleton standing; the seated upper layer the same about the shoulder). */
export const CUP_HELD: Vec = [0.12, 1.15, 0.27];

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
    // On the floor (`FLOOR_SITS`): cross-legged on a cushion, on the ground against a trunk.
    ...FLOOR_SITS.flatMap(floorClips),
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
    // A sip from a cup (actions.ts `PROPS.mug`): cup up from the chest, the head tips back for the sip, back down.
    clip('drink', { seconds: 2, loop: false }, [
      [0, () => eatPose(0, CUP_HELD, [0, 0, 0], 0)],
      [0.3, () => eatPose(0.3, [0.05, 1.49, 0.17], [-4, 0, 0], 2), 'smooth'],
      [0.42, () => eatPose(0.42, [0.05, 1.53, 0.15], [-14, 0, 0], -3), 'smooth'],
      [0.6, () => eatPose(0.6, [0.05, 1.53, 0.15], [-16, 0, 0], -4), 'smooth'],
      [0.75, () => eatPose(0.75, [0.08, 1.3, 0.25], [3, 0, 0], 1), 'smooth'],
      [1, () => eatPose(1, CUP_HELD, [0, 0, 0], 0), 'smooth'],
    ]),
  ];
}

