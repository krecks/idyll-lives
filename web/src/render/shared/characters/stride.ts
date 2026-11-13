/**
 * Starting and stopping a walk on feet that stay where they are put (`residents.ts` `start`, `update`).
 *
 * A crossfade between standing and a gait moves every bone at once while the body starts or stops moving over the
 * floor at once, so both feet went along the floor: starting, the standing pose was carried off with the body
 * while the walk faded in from its heel strike, the feet 60 cm apart, at a rate that only caught up with the
 * ground after a third of a second (0.34 to 0.53 m of a foot on the floor); stopping, the walk went on stepping
 * on the spot while it faded out (0.19 to 0.28 m on average, 0.42 at worst). `tools/characters/transitions.ts`
 * measures it, on the runtime itself.
 *
 * Starting (`strideStart`): the gait begins where it is nearest to the pose before it (`gaitPhase`: from a stand,
 * one foot under the hip and the other passing it, on either foot); the legs take the gait's pose in
 * `STRIDE.legs` s, sooner than the rest of the body fades (they are nearly there); the foot the gait has on the
 * floor stays where it stood for as long (`legPin`); and while the gait fades in, its cycle goes on by the way the
 * body has gone, not by the clock (`residents.ts`), so that foot is still however unevenly the first step comes.
 *
 * Stopping (`strideStop`): the gait stops with the body (it goes on only by the way the body still goes), and the
 * feet come to where the new pose has them one at a time, each in a step: first the one in the air, or with both
 * down the one behind, while the other stays where it stands on the floor; then the other. A step takes longer the
 * further the foot goes (`STRIDE.step`: the fade is as long as its two steps) and goes through the air on an arc
 * (`STRIDE.lift`), so what is on the floor does not move.
 *
 * Where a foot is to be, the leg is put there (`legPin`): the fade's pose of the leg (its knee's way, its foot's
 * turn) turned at the hip and bent at the knee until the ankle is on the point, as rotations of the residents'
 * additive layer, which `skin` applies. A leg hung from a pelvis that is fading between two poses swings with it
 * otherwise (5 cm at the foot for the walk's 3.5° of roll).
 */

import type { BodyData, CharacterSet, Clip } from './data';
import { addRotation, qmul, rotate, sampleClip, type Additive } from './pose';

export const STRIDE = {
  /** Starting: the legs are the gait's within this (s). */
  legs: 0.1,
  /** Starting: the gait begins this much of a cycle on from where it is nearest the pose before it (`gaitPhase`). */
  lead: 0,
  /** Starting with the foot on the floor further than this (m) from where the gait has it, the fade is the plain one. */
  standing: 0.12,
  /** Stopping for a pose with an ankle further than this (m) from where it is at rest, the fade is the plain one. */
  stand: 0.25,
  /** Stopping: a foot's step takes `step[0]` s and `step[1]` s per metre it goes, `step[2]` at most. */
  step: [0.12, 0.5, 0.3] as readonly [number, number, number],
  /** Stopping: a foot going less far than this (m) is put there with no step of its own; the fade is no shorter than `least` s. */
  near: 0.015,
  least: 0.2,
  /** A stepping foot lifts `lift[0]` of the way it goes, `lift[1]` (m, on the reference's legs: `lift[2]` m from hip to ankle) at most. */
  lift: [0.22, 0.045, 0.81] as readonly [number, number, number],
} as const;

/** A resident's feet over a start or a stop (`SimRig.stride`). */
export interface Stride {
  /** 0: none (every bone fades together); 1: starting; 2: stopping. */
  kind: 0 | 1 | 2;
  /** Starting: how many times sooner than the fade the legs are done. */
  legs: number;
  /**
   * Stopping: the leg that steps first (0 left, 1 right) and the share of the fade its step takes. Starting: the
   * leg the gait stands on.
   */
  first: number;
  split: number;
  /** Stopping: how high each foot's step goes (m, at the body's size; left, right), 0 for one with no step of its own. */
  lift: [number, number];
  /**
   * Once `held`. Stopping: where each ankle was on the floor as the fade began (world; x, y, z of the left, then of
   * the right). Starting: how far each was then from where the gait has it (world, the same six).
   */
  from: Float32Array;
  held: boolean;
  /** The body's place the last time its feet were put (`legPin`'s `place`), for a fade cut short (`legBake`). */
  place: Float32Array;
}

export function newStride(): Stride {
  return { kind: 0, legs: 1, first: 0, split: 0.5, lift: [0, 0], from: new Float32Array(6), held: false, place: new Float32Array(10) };
}

const smooth = (t: number): number => (t <= 0 ? 0 : t >= 1 ? 1 : t * t * (3 - 2 * t));

/** A leg as posed (`legPose`): its hip, knee and ankle (m; 9 numbers), then its thigh's rotation (4). */
const LEG = new Float32Array(13);
const WQ = new Float32Array(4);
const LQ = new Float32Array(4);
const WT = new Float32Array(3);
const V = new Float32Array(3);

/**
 * Leg `side` (0 left, 1 right) of the pose (`q`, `pelvis`: as `sampleClip` gives it) on `body`, in its own frame at
 * its own size, with the additive layer `add` on it if given (as `skin` applies it): the hip, the knee and the
 * ankle (m) and the thigh's rotation, into `out` (13 numbers). Rest rotations are the identity (the format's).
 */
export function legPose(set: CharacterSet, body: BodyData, q: Float32Array, pelvis: ArrayLike<number>, add: Additive | null, side: number, out: Float32Array): void {
  const B = set.bone;
  const { restT } = body;
  const a = add?.active ? add.q : null;
  WQ[0] = WQ[1] = WQ[2] = 0;
  WQ[3] = 1;
  WT.fill(0);
  const foot = side ? B.foot_r : B.foot_l;
  const calf = set.parents[foot];
  const thigh = set.parents[calf];
  // From the root down: the bones over the thigh (the root, the pelvis), then the leg.
  let depth = 0;
  for (let b = foot; b >= 0; b = set.parents[b]) depth++;
  for (let d = depth - 1; d >= 0; d--) {
    let b = foot;
    for (let k = 0; k < d; k++) b = set.parents[b];
    let tx = restT[b * 3];
    let ty = restT[b * 3 + 1];
    let tz = restT[b * 3 + 2];
    if (b === B.pelvis) {
      tx += (pelvis[0] - set.sourcePelvis[0]) * body.pelvisScale;
      ty += (pelvis[1] - set.sourcePelvis[1]) * body.pelvisScale;
      tz += (pelvis[2] - set.sourcePelvis[2]) * body.pelvisScale;
    }
    rotate(WQ[0], WQ[1], WQ[2], WQ[3], tx, ty, tz, V, 0);
    WT[0] += V[0];
    WT[1] += V[1];
    WT[2] += V[2];
    qmul(WQ[0], WQ[1], WQ[2], WQ[3], q[b * 4], q[b * 4 + 1], q[b * 4 + 2], q[b * 4 + 3], LQ, 0);
    if (a && a[b * 4 + 3] !== 1) qmul(a[b * 4], a[b * 4 + 1], a[b * 4 + 2], a[b * 4 + 3], LQ[0], LQ[1], LQ[2], LQ[3], WQ, 0);
    else WQ.set(LQ);
    if (b === thigh) {
      out.set(WT, 0);
      out.set(WQ, 9);
    } else if (b === calf) out.set(WT, 3);
    else if (b === foot) out.set(WT, 6);
  }
}

/** The ankles of a pose on `body` (`legPose`, no additive layer): x, y, z of the left, then of the right, into `out`. */
export function ankles(set: CharacterSet, body: BodyData, q: Float32Array, pelvis: ArrayLike<number>, out: Float32Array): void {
  for (let side = 0; side < 2; side++) {
    legPose(set, body, q, pelvis, null, side, LEG);
    out[side * 3] = LEG[6];
    out[side * 3 + 1] = LEG[7];
    out[side * 3 + 2] = LEG[8];
  }
}

/** Phases a gait is looked at (`gaitPhase`), and what the foot in the air counts for there beside the one on the floor. */
const PHASES = 48;
const SWING = 0.15;
const gaits = new WeakMap<Clip, WeakMap<BodyData, Float32Array>>();
const GQ: { q: Float32Array | null } = { q: null };
const GP = new Float32Array(3);

/** The ankles (`ankles`) of gait `clip` on `body` at each of `PHASES` phases of its cycle, worked out once. */
function gaitAnkles(set: CharacterSet, body: BodyData, clip: Clip): Float32Array {
  let by = gaits.get(clip);
  if (!by) gaits.set(clip, (by = new WeakMap()));
  let at = by.get(body);
  if (!at) {
    at = new Float32Array(PHASES * 6);
    const q = (GQ.q = GQ.q && GQ.q.length === set.bones.length * 4 ? GQ.q : new Float32Array(set.bones.length * 4));
    for (let k = 0; k < PHASES; k++) {
      sampleClip(set, clip, (k / PHASES) * clip.duration, q, GP);
      ankles(set, body, q, GP, at.subarray(k * 6, k * 6 + 6));
    }
    by.set(body, at);
  }
  return at;
}

/**
 * Where in its cycle (0..1) gait `clip` on `body` is nearest to the pose whose ankles are `from` (`ankles`): the
 * phase a gait begins at, so that it takes the feet up where they are. The foot the gait has on the floor there
 * counts for most (it has to stay where it is; the other is on its way anyway). From a stand that is one foot
 * under the hip and the other passing it, on either foot; from a stop cut short, the stride that was left. `on`
 * (out, 2 numbers): the leg the gait stands on there (0 left, 1 right), and how far (m) its foot is from where the
 * gait has it.
 */
export function gaitPhase(set: CharacterSet, body: BodyData, clip: Clip, from: Float32Array, on: Float32Array): number {
  const at = gaitAnkles(set, body, clip);
  let best = 0;
  let least = Infinity;
  let foot = 0;
  for (let k = 0; k < PHASES; k++) {
    const o = k * 6;
    const down = at[o + 1] <= at[o + 4] ? 0 : 3;
    let cost = 0;
    for (let f = 0; f < 6; f += 3) cost += (f === down ? 1 : SWING) * ((from[f] - at[o + f]) ** 2 + (from[f + 1] - at[o + f + 1]) ** 2 + (from[f + 2] - at[o + f + 2]) ** 2);
    if (cost < least) [least, best, foot] = [cost, k, down];
  }
  // Between two of the phases looked at: the foot on the floor goes back evenly, so as far on as it is ahead.
  const o = best * 6 + foot;
  const next = ((best + 1) % PHASES) * 6 + foot;
  const back = at[o + 2] - at[next + 2];
  const on2 = back > 1e-4 ? Math.min(0.5, Math.max(-0.5, (at[o + 2] - from[foot + 2]) / back)) : 0;
  on[0] = foot / 3;
  on[1] = Math.hypot(from[foot] - at[o], from[foot + 2] - (at[o + 2] - on2 * back));
  return (((best + on2) / PHASES) % 1 + 1) % 1;
}

/**
 * A gait begun standing on leg `on`, its foot `gap` m from where the pose before had it (`gaitPhase`): from near (a
 * stand, a stride left a moment ago: within `STRIDE.standing`) the legs first, that foot kept where it stood
 * meanwhile (`Stride.kind` 1); from anything else (half up from a seat, a lunge) every bone together, as before.
 * `fade`: the fade's length (s).
 */
export function strideStart(s: Stride, on: number, gap: number, fade: number): void {
  s.kind = gap <= STRIDE.standing ? 1 : 0;
  s.held = false;
  s.first = on;
  s.lift[0] = s.lift[1] = 0;
  s.legs = Math.max(1, fade / STRIDE.legs);
}

const rests = new WeakMap<BodyData, Float32Array>();

/** `body`'s ankles at rest (`ankles`). */
function restAnkles(set: CharacterSet, body: BodyData): Float32Array {
  let at = rests.get(body);
  if (!at) {
    at = new Float32Array(6);
    ankles(set, body, set.sourceRestQ, set.sourcePelvis, at);
    rests.set(body, at);
  }
  return at;
}

/**
 * A gait left for a pose whose ankles are `to`, from the pose it stopped in (`from`; `ankles`, on `body`, whose
 * legs are `legs` m long from hip to ankle): the feet step there one at a time (`Stride.kind` 2). Not for a pose
 * that is no stand (an ankle `STRIDE.stand` or more from where it is at rest: onto a saddle, into a seat's own
 * clip part way), which every bone fades to together as before (`Stride.kind` 0). Returns the fade's length (s):
 * its two steps', `fade` at least.
 */
export function strideStop(s: Stride, set: CharacterSet, body: BodyData, from: Float32Array, to: Float32Array, legs: number, fade: number): number {
  const rest = restAnkles(set, body);
  s.kind = 0;
  for (let k = 0; k < 6; k += 3) if (Math.hypot(to[k] - rest[k], to[k + 1] - rest[k + 1], to[k + 2] - rest[k + 2]) > STRIDE.stand) return fade;
  const way = [Math.hypot(from[0] - to[0], from[2] - to[2]), Math.hypot(from[3] - to[3], from[5] - to[5])];
  const up = [from[1] - to[1], from[4] - to[4]];
  // First the one in the air (the higher by a centimetre or more), else the one behind.
  s.first = Math.abs(up[0] - up[1]) > 0.01 ? (up[0] > up[1] ? 0 : 1) : from[2] - to[2] < from[5] - to[5] ? 0 : 1;
  const time = [0, 0];
  for (let side = 0; side < 2; side++) {
    time[side] = way[side] < STRIDE.near ? 0 : Math.min(STRIDE.step[2], STRIDE.step[0] + STRIDE.step[1] * way[side]);
    s.lift[side] = time[side] > 0 ? Math.min(STRIDE.lift[0] * way[side], (STRIDE.lift[1] * legs) / STRIDE.lift[2]) : 0;
  }
  const [a, b] = [time[s.first], time[1 - s.first]];
  s.kind = 2;
  s.held = false;
  // (With one step only, it is the whole fade.)
  s.split = a > 0 && b > 0 ? a / (a + b) : a > 0 ? 1 : 0;
  return Math.max(fade, STRIDE.least, a + b);
}

/** How far along its own way leg `side` is (0..1, not eased) at `w` of the fade (0..1, not eased). */
function along(s: Stride, side: number, w: number): number {
  if (s.kind === 1) return Math.min(1, w * s.legs);
  if (s.kind !== 2) return w;
  // (A leg with no step of its own goes with the body.)
  if (s.lift[side] === 0) return w;
  if (side === s.first) return s.split > 0 ? Math.min(1, w / s.split) : 1;
  return s.split < 1 ? Math.max(0, (w - s.split) / (1 - s.split)) : 0;
}

/** The weight of the pose faded to on leg `side` (0 left, 1 right) at `w` of the fade (0..1, not eased); the rest of the body's is the eased `w`. */
export function legWeight(s: Stride, side: number, w: number): number {
  return smooth(along(s, side, w));
}

const AXIS = new Float32Array(3);
const TO = new Float32Array(3);
const AT = new Float32Array(3);
const GOAL = new Float32Array(13);

/**
 * Puts leg `side`'s ankle of the fade's pose (`q`, `pelvis`, with `add` on it) at the point (x, y, z) (m, in the
 * body's own frame at its own size): turned at the hip and bent at the knee as far as that takes, by rotations
 * added to `add` for its thigh, its calf and its foot (the foot keeps the way it faces). A point out of the
 * leg's reach is reached for with a straight leg.
 */
function legTo(set: CharacterSet, body: BodyData, q: Float32Array, pelvis: Float32Array, add: Additive, side: number, x: number, y: number, z: number): void {
  const B = set.bone;
  legPose(set, body, q, pelvis, add, side, LEG);
  const [hx, hy, hz, kx, ky, kz] = [LEG[0], LEG[1], LEG[2], LEG[3], LEG[4], LEG[5]];
  // Knee to hip (u) and knee to ankle (v): the knee's angle between them, and the angle the point's distance asks.
  const [ux, uy, uz, vx, vy, vz] = [hx - kx, hy - ky, hz - kz, LEG[6] - kx, LEG[7] - ky, LEG[8] - kz];
  const l1 = Math.hypot(ux, uy, uz);
  const l2 = Math.hypot(vx, vy, vz);
  const reach = Math.min(Math.max(Math.hypot(x - hx, y - hy, z - hz), Math.abs(l1 - l2) + 1e-4), (l1 + l2) * 0.9995);
  const now = Math.acos(Math.min(1, Math.max(-1, (ux * vx + uy * vy + uz * vz) / (l1 * l2))));
  const want = Math.acos(Math.min(1, Math.max(-1, (l1 * l1 + l2 * l2 - reach * reach) / (2 * l1 * l2))));
  // The knee's axis: across the leg's own plane, or for a straight leg the thigh's side axis (the knee bends back).
  let [nx, ny, nz] = [uy * vz - uz * vy, uz * vx - ux * vz, ux * vy - uy * vx];
  let n = Math.hypot(nx, ny, nz);
  if (n < 1e-4 * l1 * l2) {
    rotate(LEG[9], LEG[10], LEG[11], LEG[12], -1, 0, 0, AXIS, 0);
    [nx, ny, nz, n] = [AXIS[0], AXIS[1], AXIS[2], 1];
  }
  [nx, ny, nz] = [nx / n, ny / n, nz / n];
  const bend = want - now;
  // The ankle after the bend (v turned about the knee's axis), then the whole leg turned at the hip onto the point.
  const [sb, cb] = [Math.sin(bend / 2), Math.cos(bend / 2)];
  rotate(nx * sb, ny * sb, nz * sb, cb, vx, vy, vz, AT, 0);
  const [ax, ay, az] = [kx + AT[0] - hx, ky + AT[1] - hy, kz + AT[2] - hz];
  const [tx, ty, tz] = [x - hx, y - hy, z - hz];
  let [rx, ry, rz] = [ay * tz - az * ty, az * tx - ax * tz, ax * ty - ay * tx];
  const r = Math.hypot(rx, ry, rz);
  const turn = r > 1e-9 ? Math.atan2(r, ax * tx + ay * ty + az * tz) : 0;
  if (r > 1e-9) [rx, ry, rz] = [rx / r, ry / r, rz / r];
  else [rx, ry, rz] = [1, 0, 0];
  const [thigh, calf, foot] = side ? [B.thigh_r, B.calf_r, B.foot_r] : [B.thigh_l, B.calf_l, B.foot_l];
  // (The calf's bend is about the knee's axis as the hip's turn leaves it; the foot is turned back by both.)
  const [st, ct] = [Math.sin(turn / 2), Math.cos(turn / 2)];
  rotate(rx * st, ry * st, rz * st, ct, nx, ny, nz, AXIS, 0);
  addRotation(add, thigh, rx, ry, rz, turn);
  addRotation(add, calf, AXIS[0], AXIS[1], AXIS[2], bend);
  addRotation(add, foot, AXIS[0], AXIS[1], AXIS[2], -bend);
  addRotation(add, foot, rx, ry, rz, -turn);
}

/**
 * The feet where a start or a stop wants them this frame (`Stride`), at `w` of the fade (0..1, not eased): the
 * fade's pose (`q`, `pelvis`, with the additive layer `add` on it; the legs faded by `legWeight`) with each leg
 * put (`legTo`) on its ankle's point. `goalQ`, `goalPelvis`: the pose faded to, alone. `place`: the body's place
 * (m, world), its rotation (a quaternion), its scale, and how far it went over the floor this frame (x, z), 10
 * numbers: the points are kept in the world, so a body still creeping to its place leaves the feet where they are.
 *
 * Stopping: before its step a foot is where it stood as the fade began; over its step it goes from there to where
 * the pose faded to has it, lifted on an arc; after, it is there (the pelvis still fading over it). Starting: the
 * foot the gait stands on is where the gait has it (still on the floor: the gait goes on by the way gone) and as
 * far from there as it stood when the fade began, less and less over the fade (a few centimetres).
 */
export function legPin(set: CharacterSet, body: BodyData, s: Stride, w: number, q: Float32Array, pelvis: Float32Array, goalQ: Float32Array, goalPelvis: Float32Array, add: Additive, place: Float32Array): void {
  if (!s.kind || w >= 1) return;
  const [px, py, pz, qx, qy, qz, qw, scale] = place;
  if (place !== s.place) s.place.set(place);
  if (!s.held) {
    for (let side = 0; side < 2; side++) {
      // Where the ankle is as the fade begins (its pose is still the one faded from), in the world.
      legPose(set, body, q, pelvis, add, side, LEG);
      rotate(qx, qy, qz, qw, LEG[6] * scale, LEG[7] * scale, LEG[8] * scale, TO, 0);
      s.from[side * 3] = TO[0] + px;
      s.from[side * 3 + 1] = TO[1] + py;
      s.from[side * 3 + 2] = TO[2] + pz;
      if (s.kind === 1) {
        // Starting: from where the gait has it now to where it stood before this frame's way (the stand was
        // carried along by it, the gait's foot was not).
        legPose(set, body, goalQ, goalPelvis, add, side, GOAL);
        rotate(qx, qy, qz, qw, GOAL[6] * scale, GOAL[7] * scale, GOAL[8] * scale, TO, 0);
        s.from[side * 3] -= TO[0] + px + place[8];
        s.from[side * 3 + 1] -= TO[1] + py;
        s.from[side * 3 + 2] -= TO[2] + pz + place[9];
      }
    }
    s.held = true;
  }
  for (let side = 0; side < 2; side++) {
    let x: number, y: number, z: number;
    if (s.kind === 1) {
      // (The other foot is the one that steps off: the fade's own.)
      if (side !== s.first) continue;
      legPose(set, body, goalQ, goalPelvis, add, side, GOAL);
      const left = (1 - smooth(w)) / scale;
      rotate(-qx, -qy, -qz, qw, s.from[side * 3] * left, s.from[side * 3 + 1] * left, s.from[side * 3 + 2] * left, TO, 0);
      [x, y, z] = [GOAL[6] + TO[0], GOAL[7] + TO[1], GOAL[8] + TO[2]];
    } else {
      // Where it stood, in the body's frame now; over its own step, on to where the pose faded to has it.
      const a = along(s, side, w);
      rotate(-qx, -qy, -qz, qw, s.from[side * 3] - px, s.from[side * 3 + 1] - py, s.from[side * 3 + 2] - pz, TO, 0);
      [x, y, z] = [TO[0] / scale, TO[1] / scale, TO[2] / scale];
      if (a > 0) {
        legPose(set, body, goalQ, goalPelvis, add, side, GOAL);
        const e = smooth(a);
        x += (GOAL[6] - x) * e;
        y += (GOAL[7] - y) * e + s.lift[side] * Math.sin(Math.PI * a);
        z += (GOAL[8] - z) * e;
      }
    }
    legTo(set, body, q, pelvis, add, side, x, y, z);
  }
}

/**
 * The rotations `legPin` added to `add` for the legs, taken into the pose `q` itself (each leg bone's own rotation
 * with its share of them) and out of `add`: the pose as drawn, for a fade cut short to hold and fade on from
 * (`residents.ts` `holdFade`). `add` has nothing else on it.
 */
export function legBake(set: CharacterSet, q: Float32Array, add: Additive): void {
  if (!add.active) return;
  const B = set.bone;
  const a = add.q;
  for (let side = 0; side < 2; side++) {
    const foot = side ? B.foot_r : B.foot_l;
    const calf = set.parents[foot];
    const thigh = set.parents[calf];
    // The rotation over the thigh: the root's and the pelvis'.
    WQ[0] = WQ[1] = WQ[2] = 0;
    WQ[3] = 1;
    let depth = 0;
    for (let b = set.parents[thigh]; b >= 0; b = set.parents[b]) depth++;
    for (let d = depth; d >= 1; d--) {
      let b = thigh;
      for (let k = 0; k < d; k++) b = set.parents[b];
      qmul(WQ[0], WQ[1], WQ[2], WQ[3], q[b * 4], q[b * 4 + 1], q[b * 4 + 2], q[b * 4 + 3], LQ, 0);
      WQ.set(LQ);
    }
    for (const b of [thigh, calf, foot]) {
      const o = b * 4;
      // As `skin` has it: the bone's rotation is its addition times its parent's times its own. Its own, with
      // the addition in it: the parent's undone from that.
      qmul(WQ[0], WQ[1], WQ[2], WQ[3], q[o], q[o + 1], q[o + 2], q[o + 3], LQ, 0);
      const [cx, cy, cz, cw] = [-WQ[0], -WQ[1], -WQ[2], WQ[3]];
      qmul(a[o], a[o + 1], a[o + 2], a[o + 3], LQ[0], LQ[1], LQ[2], LQ[3], WQ, 0);
      qmul(cx, cy, cz, cw, WQ[0], WQ[1], WQ[2], WQ[3], q, o);
      a[o] = a[o + 1] = a[o + 2] = 0;
      a[o + 3] = 1;
    }
  }
}
