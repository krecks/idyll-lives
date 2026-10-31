/**
 * A small language for the residents' animation clips, baked at load into the runtime's `Clip`s
 * (30 fps quaternions per clip bone, plus the pelvis position; see `sampleClip` in pose.ts).
 *
 * Conventions (docs/characters-format.md):
 * - A pose gives bones local rotations as Euler angles in degrees, `[x, y, z]`, applied about x,
 *   then y, then z of the parent's frame. Every rest rotation is the identity, so at rest these are
 *   the model axes: +z forward, +y up, the character's left at -x. The signs match `addRotation`:
 *   -x on a thigh or upper arm raises it forward, +x on a calf bends the knee, -x on a forearm
 *   bends the elbow, +z on the left upper arm brings it down to the side.
 * - A side-less key (`thigh`) sets the left bone and the mirrored right one (x, -y, -z); a sided
 *   key (`thigh_r`) sets one side and wins over the side-less one. Bones a pose doesn't name rest.
 * - `move` is the pelvis' offset from its rest position (m, on the reference skeleton; the runtime
 *   scales it per body with `pelvisScale`).
 * - Every clip holds the hands in a fist (`GRIP`) unless it sets the finger bones itself (or
 *   `hands: 'open'`): the runtime opens them towards the rest pose (the open hand) per activity.
 * - Keys are at phase t in 0..1 of the clip; loops wrap from the last key to the first.
 */

import type { CharacterSet, Clip } from '../data';

export type Rot = readonly [x: number, y: number, z: number];
export type Vec = readonly [x: number, y: number, z: number];

const SIDED = ['clavicle', 'upperarm', 'lowerarm', 'hand', 'thumb_01', 'thumb_02', 'fingers_01', 'fingers_02', 'thigh', 'calf', 'foot', 'ball'] as const;
type Sided = (typeof SIDED)[number];
export type BoneKey = 'pelvis' | 'spine_01' | 'spine_02' | 'spine_03' | 'neck_01' | 'Head' | Sided | `${Sided}_l` | `${Sided}_r`;
export type PoseSpec = { [K in BoneKey]?: Rot } & { move?: Vec };
export type PoseFn = (phase: number) => PoseSpec;
export type Ease = 'linear' | 'smooth' | 'in' | 'out' | ((t: number) => number);
/** A key: phase (0..1), the pose (or a function making it at bake time, for `legIK`), the easing into it. */
export type Key = readonly [t: number, pose: PoseSpec | (() => PoseSpec), ease?: Ease];
export type Mask = 'upper' | 'legs' | 'hands';

export interface ClipOptions {
  seconds: number;
  loop: boolean;
  /** Locomotion: ground speed of the planted foot (m/s, reference skeleton). */
  speed?: number;
  /** 'open': keep the rest (open) hand instead of the fist. */
  hands?: 'grip' | 'open';
}

interface Layer {
  pose: PoseSpec | PoseFn;
  mask?: Mask;
  additive: boolean;
}

/**
 * The fist every clip holds: fingers curl past a right angle at the knuckles and more at the middle
 * joints, so the tips fold back onto the palm (a closed fist from every side, no hole); the thumb
 * turns in front of them and lies across.
 */
export const GRIP: PoseSpec = { fingers_01: [0, 0, 100], fingers_02: [0, 0, 110], thumb_01: [11, -3, 12], thumb_02: [48, -16, -23] };

/** A clip: made by `clip` (keys) or `cycle` (a function of the phase); `layer` adds on top. */
export class ClipDef {
  readonly layers: Layer[] = [];
  readonly name: string;
  readonly options: ClipOptions;
  readonly sample: PoseFn;
  // (Plain fields, no parameter properties: tools/characters/check_clips.ts runs this file in Node's type stripping.)
  constructor(name: string, options: ClipOptions, sample: PoseFn) {
    this.name = name;
    this.options = options;
    this.sample = sample;
  }

  /** A layer over the clip: replaces the bones it names (within `mask`), or adds its angles (`additive`). */
  layer(pose: PoseSpec | PoseFn, opts: { mask?: Mask; additive?: boolean } = {}): this {
    this.layers.push({ pose, mask: opts.mask, additive: !!opts.additive });
    return this;
  }
}

/** A clip from key poses (phase 0..1). */
export function clip(name: string, options: ClipOptions, keys: readonly Key[]): ClipDef {
  if (!keys.length) throw new Error(`clip ${name}: no keys`);
  const sorted = [...keys].sort((a, b) => a[0] - b[0]);
  return new ClipDef(name, options, (phase) => {
    const resolve = (k: Key) => (typeof k[1] === 'function' ? k[1]() : k[1]);
    if (sorted.length === 1) return resolve(sorted[0]);
    const i = sorted.findIndex((k) => k[0] > phase) - 1;
    let a: Key, b: Key, t0: number, t1: number;
    if (i === -2) {
      // Past the last key: hold it, or (loops) blend towards the first.
      a = sorted[sorted.length - 1];
      b = options.loop ? sorted[0] : a;
      t0 = a[0];
      t1 = options.loop ? b[0] + 1 : a[0] + 1;
    } else if (i === -1) {
      // Before the first key: (loops) from the last one, else hold the first.
      b = sorted[0];
      a = options.loop ? sorted[sorted.length - 1] : b;
      t0 = options.loop ? a[0] - 1 : -1;
      t1 = b[0];
    } else {
      a = sorted[i];
      b = sorted[i + 1];
      t0 = a[0];
      t1 = b[0];
    }
    const u = t1 > t0 ? (phase - t0) / (t1 - t0) : 0;
    return mix(resolve(a), resolve(b), ease(b[2] ?? 'smooth', Math.min(1, Math.max(0, u))));
  });
}

/** A clip sampled from a function of the phase every frame (walk cycles, breathing). */
export function cycle(name: string, options: ClipOptions, fn: PoseFn): ClipDef {
  return new ClipDef(name, options, fn);
}

function ease(e: Ease, t: number): number {
  if (typeof e === 'function') return e(t);
  switch (e) {
    case 'linear':
      return t;
    case 'in':
      return t * t;
    case 'out':
      return 1 - (1 - t) * (1 - t);
    default:
      return t * t * (3 - 2 * t);
  }
}

/** Euler angles per bone name (both sides expanded; side-less keys mirrored onto the right). */
type Expanded = Map<string, [number, number, number]>;

function expand(spec: PoseSpec): Expanded {
  const out: Expanded = new Map();
  for (const [k, v] of Object.entries(spec) as [string, Rot | Vec | undefined][]) {
    if (k === 'move' || !v) continue;
    if ((SIDED as readonly string[]).includes(k)) {
      if (!(`${k}_l` in spec)) out.set(`${k}_l`, [v[0], v[1], v[2]]);
      if (!(`${k}_r` in spec)) out.set(`${k}_r`, [v[0], -v[1], -v[2]]);
    } else {
      out.set(k, [v[0], v[1], v[2]]);
    }
  }
  return out;
}

/** Linear blend of two poses (angles and `move`); bones missing on one side blend from rest. */
export function mix(a: PoseSpec, b: PoseSpec, t: number): PoseSpec {
  const ea = expand(a);
  const eb = expand(b);
  const out: Record<string, unknown> = {};
  for (const k of new Set([...ea.keys(), ...eb.keys()])) {
    const x = ea.get(k) ?? [0, 0, 0];
    const y = eb.get(k) ?? [0, 0, 0];
    out[k] = [x[0] + (y[0] - x[0]) * t, x[1] + (y[1] - x[1]) * t, x[2] + (y[2] - x[2]) * t];
  }
  const ma = a.move ?? [0, 0, 0];
  const mb = b.move ?? [0, 0, 0];
  if (a.move || b.move) out.move = [ma[0] + (mb[0] - ma[0]) * t, ma[1] + (mb[1] - ma[1]) * t, ma[2] + (mb[2] - ma[2]) * t];
  return out as PoseSpec;
}

/** Poses added together (angles and moves summed per bone; sided keys stay sided). */
export function add(...specs: PoseSpec[]): PoseSpec {
  const out: Record<string, unknown> = {};
  let move: [number, number, number] | null = null;
  for (const s of specs) {
    for (const [k, v] of expand(s)) {
      const o = (out[k] as [number, number, number] | undefined) ?? [0, 0, 0];
      out[k] = [o[0] + v[0], o[1] + v[1], o[2] + v[2]];
    }
    if (s.move) {
      move ??= [0, 0, 0];
      move = [move[0] + s.move[0], move[1] + s.move[1], move[2] + s.move[2]];
    }
  }
  if (move) out.move = move;
  return out as PoseSpec;
}

const MASKS: Record<Mask, (bone: string) => boolean> = {
  upper: (b) => /^(spine_02|spine_03|neck_01|Head|clavicle|upperarm|lowerarm|hand|thumb|fingers)/.test(b),
  legs: (b) => /^(thigh|calf|foot|ball)_/.test(b),
  hands: (b) => /^(hand|thumb|fingers)/.test(b),
};

// --- leg IK -----------------------------------------------------------------------------------

/** The reference skeleton while clips bake: rest joints (model space), parents, leg lengths, the palm centre. */
export interface Reference {
  joint: (name: string) => Vec;
  /** The parent bone's name ('' for the root). */
  parent: (name: string) => string;
  thigh: number;
  shin: number;
  /** The left palm centre from the wrist (rest frame; the right mirrors x). */
  grip: Vec;
}

/** Rest joints of the reference skeleton while baking (for `legIK`). */
let ref: Reference | null = null;

/** The reference skeleton; only while clips bake (for IK helpers such as clips/ik.ts). */
export function reference(): Reference {
  if (!ref) throw new Error('reference: only while baking clips');
  return ref;
}

/**
 * Thigh, calf and foot rotations that put the ankle of leg `side` at its rest position + `ankle`
 * (m, reference skeleton) while the pelvis is moved by `move`, the knee bending forward and the
 * sole staying level. Two-bone analytic IK on the reference leg lengths (from rig.json); usable
 * only while clips bake (inside `cycle` functions and key functions).
 */
export function legIK(side: 'l' | 'r', ankle: Vec, move: Vec = [0, 0, 0]): PoseSpec {
  if (!ref) throw new Error('legIK: only while baking clips');
  const hip = ref.joint(`thigh_${side}`);
  const foot = ref.joint(`foot_${side}`);
  const dx = foot[0] + ankle[0] - (hip[0] + move[0]);
  const dy = foot[1] + ankle[1] - (hip[1] + move[1]);
  const dz = foot[2] + ankle[2] - (hip[2] + move[2]);
  const { thigh: L1, shin: L2 } = ref;
  const D = Math.min(Math.hypot(dx, dy, dz), (L1 + L2) * 0.9995);
  const knee = Math.PI - Math.acos(clamp((L1 * L1 + L2 * L2 - D * D) / (2 * L1 * L2), -1, 1));
  const lift = Math.acos(clamp((L1 * L1 + D * D - L2 * L2) / (2 * L1 * D), -1, 1));
  const pitch = Math.atan2(dz, -dy);
  const roll = Math.asin(clamp(dx / Math.max(D, 1e-6), -1, 1));
  // Pitches (forward of straight down) of the thigh and shin: at rest, and wanted. A rotation of
  // -a about x pitches a bone forward by a.
  const k = ref.joint(`calf_${side}`);
  const rest1 = Math.atan2(k[2] - hip[2], hip[1] - k[1]);
  const rest2 = Math.atan2(foot[2] - k[2], k[1] - foot[1]);
  const thigh = -(pitch + lift - rest1);
  const calf = rest2 - thigh - (pitch + lift - knee);
  const deg = 180 / Math.PI;
  return {
    [`thigh_${side}`]: [thigh * deg, 0, roll * deg],
    [`calf_${side}`]: [calf * deg, 0, 0],
    [`foot_${side}`]: [-(thigh + calf) * deg, 0, -roll * deg],
  } as PoseSpec;
}

function clamp(v: number, lo: number, hi: number): number {
  return v < lo ? lo : v > hi ? hi : v;
}

// --- baking -----------------------------------------------------------------------------------

/** The reference skeleton's rest joint positions (model space) from its body's local translations. */
function referenceJoints(set: CharacterSet): (name: string) => Vec {
  const body = set.bodies.get(set.reference?.body ?? 'male') ?? set.bodies.values().next().value!;
  const n = set.bones.length;
  const world = new Float32Array(n * 3);
  for (let b = 0; b < n; b++) {
    const p = set.parents[b];
    for (let k = 0; k < 3; k++) world[b * 3 + k] = body.restT[b * 3 + k] + (p >= 0 ? world[p * 3 + k] : 0);
  }
  return (name) => {
    const b = set.bone[name];
    return [world[b * 3], world[b * 3 + 1], world[b * 3 + 2]];
  };
}

/** Bakes clip definitions into the runtime's clips for `set`. */
export function bake(defs: readonly ClipDef[], set: CharacterSet): Map<string, Clip> {
  const joint = referenceJoints(set);
  const len = (a: string, b: string) => {
    const p = joint(a);
    const q = joint(b);
    return Math.hypot(p[0] - q[0], p[1] - q[1], p[2] - q[2]);
  };
  const body = set.bodies.get(set.reference?.body ?? 'male') ?? set.bodies.values().next().value!;
  ref = {
    joint,
    parent: (name) => set.bones[set.parents[set.bone[name]]] ?? '',
    thigh: set.reference?.thigh ?? len('thigh_l', 'calf_l'),
    shin: set.reference?.shin ?? len('calf_l', 'foot_l'),
    grip: body.grip ?? [-0.02, -0.055, 0],
  };
  const out = new Map<string, Clip>();
  try {
    for (const def of defs) out.set(def.name, bakeOne(def, set));
  } finally {
    ref = null;
  }
  return out;
}

const D2R = Math.PI / 360;

function bakeOne(def: ClipDef, set: CharacterSet): Clip {
  const { seconds, loop } = def.options;
  const CB = set.clipBones;
  const steps = Math.max(1, Math.round(seconds * set.fps));
  const frames = loop ? steps : steps + 1;
  const q = new Int16Array(frames * CB * 4);
  const pelvis = new Float32Array(frames * 3);
  const sp = set.sourcePelvis;
  const grip = def.options.hands === 'open' ? new Map() : expand(GRIP);
  for (let f = 0; f < frames; f++) {
    const phase = f / steps;
    const spec = def.sample(phase);
    const pose = expand(spec);
    let move = spec.move ?? [0, 0, 0];
    for (const [k, v] of grip) if (!pose.has(k)) pose.set(k, v);
    for (const layer of def.layers) {
      const over = typeof layer.pose === 'function' ? layer.pose(phase) : layer.pose;
      const test = layer.mask ? MASKS[layer.mask] : null;
      for (const [k, v] of expand(over)) {
        if (test && !test(k)) continue;
        const o = pose.get(k) ?? [0, 0, 0];
        pose.set(k, layer.additive ? [o[0] + v[0], o[1] + v[1], o[2] + v[2]] : v);
      }
      const m = over.move;
      if (m && (!layer.mask || layer.mask === 'legs')) move = layer.additive ? [move[0] + m[0], move[1] + m[1], move[2] + m[2]] : m;
    }
    for (let b = 0; b < CB; b++) {
      const e = pose.get(set.bones[b]);
      const o = (f * CB + b) * 4;
      if (!e) {
        q[o + 3] = 32767;
        continue;
      }
      // q = qz * qy * qx (x applied first).
      const sx = Math.sin(e[0] * D2R);
      const cx = Math.cos(e[0] * D2R);
      const sy = Math.sin(e[1] * D2R);
      const cy = Math.cos(e[1] * D2R);
      const sz = Math.sin(e[2] * D2R);
      const cz = Math.cos(e[2] * D2R);
      // qy * qx
      const ax = cy * sx;
      const ay = sy * cx;
      const az = -sy * sx;
      const aw = cy * cx;
      // qz * (qy * qx)
      const x = cz * ax - sz * ay;
      const y = cz * ay + sz * ax;
      const z = cz * az + sz * aw;
      const w = cz * aw - sz * az;
      q[o] = Math.round(x * 32767);
      q[o + 1] = Math.round(y * 32767);
      q[o + 2] = Math.round(z * 32767);
      q[o + 3] = Math.round(w * 32767);
    }
    pelvis[f * 3] = sp[0] + move[0];
    pelvis[f * 3 + 1] = sp[1] + move[1];
    pelvis[f * 3 + 2] = sp[2] + move[2];
  }
  return { name: def.name, frames, loop, q, pelvis, speed: def.options.speed ?? 0, duration: (loop ? frames : frames - 1) / set.fps };
}
