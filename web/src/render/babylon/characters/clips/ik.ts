/**
 * Helpers for the clips: rotation matrices and the DSL's Euler angles (degrees, x then y then z
 * about the parent's axes, so R = Rz Ry Rx), forward kinematics of a pose on the reference
 * skeleton, and two-bone IK for the arms and legs that follows the pose's pelvis, spine and
 * clavicles. Everything works in model space of the reference skeleton (metres, +z forward, the
 * resident's left at -x) and only while the clips bake (`reference()` in dsl.ts).
 */

import { reference, type BoneKey, type PoseSpec, type Rot, type Vec } from './dsl';

/** A 3x3 rotation, row-major. */
export type M3 = readonly number[];

const R2D = 180 / Math.PI;
const D2R = Math.PI / 180;

export const I3: M3 = [1, 0, 0, 0, 1, 0, 0, 0, 1];

export function mul(a: M3, b: M3): M3 {
  const o: number[] = [];
  for (let r = 0; r < 3; r++) for (let c = 0; c < 3; c++) o.push(a[r * 3] * b[c] + a[r * 3 + 1] * b[3 + c] + a[r * 3 + 2] * b[6 + c]);
  return o;
}

export function tr(a: M3): M3 {
  return [a[0], a[3], a[6], a[1], a[4], a[7], a[2], a[5], a[8]];
}

export function apply(m: M3, v: Vec): Vec {
  return [m[0] * v[0] + m[1] * v[1] + m[2] * v[2], m[3] * v[0] + m[4] * v[1] + m[5] * v[2], m[6] * v[0] + m[7] * v[1] + m[8] * v[2]];
}

/** Rotation by `deg` about `axis` (right-handed about the axis, as the clips' quaternions). */
export function axisAngle(axis: Vec, deg: number): M3 {
  const [x, y, z] = norm(axis);
  const c = Math.cos(deg * D2R);
  const s = Math.sin(deg * D2R);
  const t = 1 - c;
  return [t * x * x + c, t * x * y - s * z, t * x * z + s * y, t * x * y + s * z, t * y * y + c, t * y * z - s * x, t * x * z - s * y, t * y * z + s * x, t * z * z + c];
}

/** The matrix of DSL Euler angles (x applied first, then y, then z). */
export function euler(e: Rot): M3 {
  return mul(axisAngle([0, 0, 1], e[2]), mul(axisAngle([0, 1, 0], e[1]), axisAngle([1, 0, 0], e[0])));
}

/** DSL Euler angles of a rotation. */
export function toEuler(m: M3): [number, number, number] {
  const sb = Math.max(-1, Math.min(1, -m[6]));
  const b = Math.asin(sb);
  if (Math.abs(sb) > 0.9999) return [Math.atan2(-m[5], m[4]) * R2D, b * R2D, 0];
  return [Math.atan2(m[7], m[8]) * R2D, b * R2D, Math.atan2(m[3], m[0]) * R2D];
}

export function sub(a: Vec, b: Vec): Vec {
  return [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}
export function addV(a: Vec, b: Vec, k = 1): Vec {
  return [a[0] + b[0] * k, a[1] + b[1] * k, a[2] + b[2] * k];
}
export function dot(a: Vec, b: Vec): number {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}
export function cross(a: Vec, b: Vec): Vec {
  return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
}
export function len(a: Vec): number {
  return Math.hypot(a[0], a[1], a[2]);
}
export function norm(a: Vec): Vec {
  const l = len(a) || 1;
  return [a[0] / l, a[1] / l, a[2] / l];
}

/** A pose's Euler angles for one bone (a side-less key mirrored onto the right: x, -y, -z). */
export function anglesOf(spec: PoseSpec, bone: string): Rot {
  const own = (spec as Record<string, Rot | undefined>)[bone];
  if (own) return own;
  const m = /^(.*)_(l|r)$/.exec(bone);
  const base = m ? (spec as Record<string, Rot | undefined>)[m[1]] : undefined;
  if (!base || !m) return [0, 0, 0];
  return m[2] === 'l' ? base : [base[0], -base[1], -base[2]];
}

/** World rotation and joint position of a bone in a pose (reference skeleton, model space). */
export function world(spec: PoseSpec, bone: string): { r: M3; p: Vec } {
  const ref = reference();
  const chain: string[] = [];
  for (let b = bone; b; b = ref.parent(b)) chain.unshift(b);
  let r = I3;
  let p: Vec = [0, 0, 0];
  let restParent: Vec = [0, 0, 0];
  for (const b of chain) {
    const rest = ref.joint(b);
    p = addV(p, apply(r, sub(rest, restParent)));
    if (b === 'pelvis' && spec.move) p = addV(p, spec.move);
    r = mul(r, euler(anglesOf(spec, b)));
    restParent = rest;
  }
  return { r, p };
}

/** A point given in a bone's rest frame (an offset from its joint), in the pose (model space). */
export function pointOn(spec: PoseSpec, bone: string, offset: Vec): Vec {
  const w = world(spec, bone);
  return addV(w.p, apply(w.r, offset));
}

/**
 * Two-bone IK: the world rotation of the first bone and the local rotation of the second that put
 * the end at `target`, bending about the hinge `h0` (rest frame of the first bone; a positive turn
 * about it bends the limb), the middle joint towards `pole`.
 */
function twoBone(root: Vec, target: Vec, l1: number, l2: number, u0: Vec, v0: Vec, h0: Vec, pole: Vec): { r1: M3; bend: number; reach: number } {
  const d = sub(target, root);
  const want = len(d);
  const dist = Math.max(Math.abs(l1 - l2) + 1e-4, Math.min(want, (l1 + l2) * 0.9995));
  const dir = norm(d);
  const a = Math.acos(Math.max(-1, Math.min(1, (l1 * l1 + dist * dist - l2 * l2) / (2 * l1 * dist))));
  let perp = sub(pole, [dir[0] * dot(dir, pole), dir[1] * dot(dir, pole), dir[2] * dot(dir, pole)]);
  if (len(perp) < 1e-6) perp = norm(cross(dir, [1, 0, 0]));
  perp = norm(perp);
  const u = norm(addV([dir[0] * Math.cos(a), dir[1] * Math.cos(a), dir[2] * Math.cos(a)], perp, Math.sin(a)));
  const elbow = addV(root, u, l1);
  const w = norm(sub(addV(root, dir, dist), elbow));
  let h = cross(u, w);
  h = len(h) > 1e-5 ? norm(h) : norm(cross(perp, u));
  const h0n = norm(sub(h0, [u0[0] * dot(u0, h0), u0[1] * dot(u0, h0), u0[2] * dot(u0, h0)]));
  const f0 = [u0, h0n, cross(u0, h0n)];
  const f1 = [u, h, cross(u, h)];
  // r1 = F1 * F0^T (columns are the frame vectors).
  const r1: number[] = [];
  for (let r = 0; r < 3; r++) for (let c = 0; c < 3; c++) r1.push(f1[0][r] * f0[0][c] + f1[1][r] * f0[1][c] + f1[2][r] * f0[2][c]);
  const v = apply(r1, v0);
  const bend = Math.atan2(dot(cross(v, w), h), dot(v, w)) * R2D;
  return { r1, bend, reach: want / (l1 + l2) };
}

export interface ArmOptions {
  /** Where the elbow points (model space direction); default down, back and out. */
  pole?: Vec;
  /** Forearm roll (degrees): + turns the palm down when the forearm points forward. */
  pronate?: number;
  /** The hand's own rotation (DSL Euler, before the forearm roll); default: the pose's. */
  hand?: Rot;
  /** `target` is the palm centre (`grip`) rather than the wrist joint. */
  palm?: boolean;
}

/**
 * Upper arm, forearm and hand rotations that put the wrist (or palm) of arm `side` at `target`
 * (model space, reference skeleton) in `spec`'s pose (its spine and clavicle).
 */
export function armIK(side: 'l' | 'r', target: Vec, spec: PoseSpec, o: ArmOptions = {}): PoseSpec {
  const ref = reference();
  const sx = side === 'l' ? -1 : 1;
  const S = ref.joint(`upperarm_${side}`);
  const E = ref.joint(`lowerarm_${side}`);
  const W = ref.joint(`hand_${side}`);
  const u0 = norm(sub(E, S));
  const v0 = norm(sub(W, E));
  const h0 = norm(cross(u0, [0, 0, 1]));
  const clav = world(spec, `clavicle_${side}`);
  const root = world(spec, `upperarm_${side}`).p;
  const pole = o.pole ?? [0.35 * sx, -0.6, -0.7];
  const twist = (o.pronate ?? 0) * sx;
  const handRot = euler(o.hand ?? anglesOf(spec, `hand_${side}`));
  const handLocal = mul(axisAngle(v0, twist * 0.4), handRot);
  let goal = target;
  let out: PoseSpec = {};
  for (let pass = 0; pass < (o.palm ? 3 : 1); pass++) {
    const { r1, bend } = twoBone(root, goal, len(sub(E, S)), len(sub(W, E)), u0, v0, h0, pole);
    const upper = mul(tr(clav.r), r1);
    const lower = mul(axisAngle(h0, bend), axisAngle(v0, twist * 0.6));
    out = { [`upperarm_${side}`]: toEuler(upper), [`lowerarm_${side}`]: toEuler(lower), [`hand_${side}`]: toEuler(handLocal) } as PoseSpec;
    if (o.palm) {
      // Move the wrist target so the palm centre lands on `target`.
      const handWorld = mul(mul(r1, lower), handLocal);
      const g: Vec = [ref.grip[0] * -sx, ref.grip[1], ref.grip[2]];
      goal = sub(target, apply(handWorld, g));
    }
  }
  return out;
}

export interface LegOptions {
  /** Where the knee points (model space direction); default forward and a touch out. */
  pole?: Vec;
  /** The foot's world rotation (DSL Euler; default level: the sole flat, toes forward). */
  foot?: Rot;
  /**
   * Solved as if the hip joint were this much further out to its side of the pelvis (m, its frame): for a
   * clip played on hips wider for their legs than the reference's (a child's), so that a planted foot stays
   * put while the hips turn over it there.
   */
  wide?: number;
}

/**
 * Thigh, calf and foot rotations that put the ankle of leg `side` at its rest position + `ankle`
 * (model space, reference skeleton) in `spec`'s pose (its pelvis rotation and `move`).
 */
export function footIK(side: 'l' | 'r', ankle: Vec, spec: PoseSpec, o: LegOptions = {}): PoseSpec {
  const ref = reference();
  const sx = side === 'l' ? -1 : 1;
  const H = ref.joint(`thigh_${side}`);
  const K = ref.joint(`calf_${side}`);
  const A = ref.joint(`foot_${side}`);
  const pelvis = world(spec, 'pelvis');
  const root = addV(world(spec, `thigh_${side}`).p, apply(pelvis.r, [sx * (o.wide ?? 0), 0, 0]));
  const { r1, bend } = twoBone(root, addV(A, ankle), ref.thigh, ref.shin, norm(sub(K, H)), norm(sub(A, K)), [1, 0, 0], o.pole ?? [0.12 * sx, 0, 1]);
  const calf = axisAngle([1, 0, 0], bend);
  const foot = mul(tr(mul(r1, calf)), euler(o.foot ?? [0, 0, 0]));
  return { [`thigh_${side}`]: toEuler(mul(tr(pelvis.r), r1)), [`calf_${side}`]: toEuler(calf), [`foot_${side}`]: toEuler(foot) } as PoseSpec;
}

/** Both legs with the feet at their rest places + offsets (mirrored x for the right by default). */
export function legs(spec: PoseSpec, left: Vec, right: Vec = [-left[0], left[1], left[2]], o: { l?: LegOptions; r?: LegOptions } = {}): PoseSpec {
  return { ...footIK('l', left, spec, o.l), ...footIK('r', right, spec, o.r) };
}

/** Upper arm rotation hanging the arm: out from the side by `out` degrees, then raised forward by `fwd` and turned about its length by `roll`. */
export function hang(side: 'l' | 'r', fwd: number, out: number, roll = 0): Rot {
  const sx = side === 'l' ? -1 : 1;
  // From the A-pose (45 degrees out) down to `out`, roll about the arm, then raise forward.
  const down = axisAngle([0, 0, 1], (45 - out) * -sx);
  const arm = norm(apply(down, norm(sub(reference().joint(`lowerarm_${side}`), reference().joint(`upperarm_${side}`)))));
  return toEuler(mul(axisAngle([1, 0, 0], -fwd), mul(axisAngle(arm, roll * -sx), down)));
}

/** Forearm rotation bending the elbow by `deg` (about its true hinge, forward from the hanging arm), rolled by `pronate` (+ turns the palm down). */
export function elbow(side: 'l' | 'r', deg: number, pronate = 0): Rot {
  const ref = reference();
  const sx = side === 'l' ? -1 : 1;
  const u0 = norm(sub(ref.joint(`lowerarm_${side}`), ref.joint(`upperarm_${side}`)));
  const v0 = norm(sub(ref.joint(`hand_${side}`), ref.joint(`lowerarm_${side}`)));
  return toEuler(mul(axisAngle(norm(cross(u0, [0, 0, 1])), deg), axisAngle(v0, pronate * sx)));
}

/** Hand rotation at the wrist: `flex` towards the palm, `dev` towards the thumb, `roll` about the hand (+ palm down, as `pronate`). */
export function wrist(side: 'l' | 'r', flex: number, dev = 0, roll = 0): Rot {
  const ref = reference();
  const sx = side === 'l' ? -1 : 1;
  const v0 = norm(sub(ref.joint(`fingers_01_${side}`), ref.joint(`hand_${side}`)));
  const h0 = norm(cross(v0, [0, 0, 1]));
  return toEuler(mul(axisAngle(v0, roll * sx), mul(axisAngle(h0, dev), axisAngle([0, 0, 1], flex * -sx))));
}

export type Bones = Partial<Record<BoneKey, Rot>>;

/** Smoothstep, and a clamped ramp from a to b. */
export function smooth(t: number): number {
  const c = Math.max(0, Math.min(1, t));
  return c * c * (3 - 2 * c);
}
export function ramp(t: number, a: number, b: number): number {
  return smooth((t - a) / (b - a));
}
/** Linear interpolation of vectors. */
export function lerpV(a: Vec, b: Vec, t: number): Vec {
  return [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
}
