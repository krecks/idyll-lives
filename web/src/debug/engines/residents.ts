/**
 * The engine comparison's residents (`scene.ts`, `bench.ts`): the game's own bodies
 * (`public/assets/characters`, docs/characters-format.md) read straight from `rig.json` and the
 * `.bin` files, and a procedural walk that both drivers apply bone for bone with their engine's
 * standard skinning (Babylon's `Skeleton`, PlayCanvas's `SkinInstance`).
 *
 * Not the game's resident renderer: no morphs (faces), no painted clothes or stages, no
 * instancing of residents, and each resident has its own skeleton, as each engine does it.
 *
 * The files are in Babylon's left-handed space (a resident faces +z, clockwise faces); the
 * comparison runs right-handed in both engines, so z is turned round and the faces with it (a
 * resident then faces -z). Rest rotations are all the identity (the format's convention), so a
 * bone's rest frame is a translation and its inverse bind is that translation's negative.
 */

const BASE = `${import.meta.env.BASE_URL}assets/characters/`;

/** What a part is shaded as (one shared material each in both engines). */
export type PartLook = 'skin' | 'eye' | 'lash' | 'mouth' | 'hair' | 'cloth';

export interface BodyPart {
  name: string;
  look: PartLook;
  positions: Float32Array;
  normals: Float32Array;
  /** Four bone indices per vertex. */
  joints: Uint8Array;
  /** Four weights per vertex, summing to 1. */
  weights: Float32Array;
  indices: Uint16Array;
}

export interface Body {
  family: 'male' | 'female';
  parts: BodyPart[];
  bones: string[];
  parents: number[];
  /** Each bone's offset from its parent's joint (x, y, z per bone). */
  restT: Float32Array;
  /** Each bone's joint in model space at rest (x, y, z per bone). */
  restWorld: Float32Array;
}

/** What each family wears here: the parts drawn, one hairstyle and one pair of shoes. */
const OUTFIT: Record<Body['family'], Record<string, PartLook>> = {
  male: { body: 'skin', eyes: 'eye', lashes: 'lash', brows: 'hair', teeth: 'mouth', tongue: 'mouth', 'hair.short': 'hair', 'shoes.sneakers': 'cloth' },
  female: { body: 'skin', eyes: 'eye', lashes: 'lash', brows: 'hair', teeth: 'mouth', tongue: 'mouth', 'hair.long': 'hair', 'shoes.sneakers': 'cloth' },
};

interface PartEntry {
  vertices: number;
  indices: number;
  positions: number;
  normals: number;
  joints: number;
  weights: number;
  index: number;
}

interface Rig {
  bones: string[];
  parents: number[];
  bodies: Record<string, { mesh: string; restT: number[]; parts: Record<string, PartEntry> }>;
}

function readPart(bin: ArrayBuffer, name: string, look: PartLook, e: PartEntry): BodyPart {
  const n = e.vertices;
  const positions = new Float32Array(bin, e.positions, n * 3).slice();
  const packedNormals = new Int16Array(bin, e.normals, n * 4);
  const normals = new Float32Array(n * 3);
  const joints = new Uint8Array(bin, e.joints, n * 4).slice();
  const packedWeights = new Uint8Array(bin, e.weights, n * 4);
  const weights = new Float32Array(n * 4);
  for (let v = 0; v < n; v++) {
    positions[v * 3 + 2] = -positions[v * 3 + 2];
    normals[v * 3] = packedNormals[v * 4] / 32767;
    normals[v * 3 + 1] = packedNormals[v * 4 + 1] / 32767;
    normals[v * 3 + 2] = -packedNormals[v * 4 + 2] / 32767;
    for (let k = 0; k < 4; k++) weights[v * 4 + k] = packedWeights[v * 4 + k] / 255;
  }
  // Clockwise in the left-handed file, counter-clockwise once z is turned round: swap two corners.
  const indices = new Uint16Array(bin, e.index, e.indices).slice();
  for (let i = 0; i < indices.length; i += 3) [indices[i + 1], indices[i + 2]] = [indices[i + 2], indices[i + 1]];
  return { name, look, positions, normals, joints, weights, indices };
}

/** Both families, as `OUTFIT` dresses them. */
export async function loadBodies(): Promise<Body[]> {
  const rig = (await (await fetch(`${BASE}rig.json`)).json()) as Rig;
  return Promise.all(
    (['male', 'female'] as const).map(async (family) => {
      const entry = rig.bodies[family];
      const bin = await (await fetch(`${BASE}${entry.mesh}`)).arrayBuffer();
      const parts = Object.entries(OUTFIT[family]).map(([name, look]) => {
        const part = entry.parts[name];
        if (!part) throw new Error(`${family}.bin has no part ${name}`);
        return readPart(bin, name, look, part);
      });
      const count = rig.bones.length;
      const restT = new Float32Array(entry.restT);
      for (let b = 0; b < count; b++) restT[b * 3 + 2] = -restT[b * 3 + 2];
      const restWorld = new Float32Array(count * 3);
      for (let b = 0; b < count; b++) {
        const p = rig.parents[b];
        for (let k = 0; k < 3; k++) restWorld[b * 3 + k] = restT[b * 3 + k] + (p >= 0 ? restWorld[p * 3 + k] : 0);
      }
      return { family, parts, bones: rig.bones, parents: rig.parents, restT, restWorld };
    }),
  );
}

/** The bones the walk turns, each about its own x axis (forward is +angle), by name. */
const SWING: { bone: string; amp: number; offset: number; phase: number; bend?: boolean }[] = [
  { bone: 'thigh_l', amp: 0.45, offset: 0, phase: 0 },
  { bone: 'thigh_r', amp: 0.45, offset: 0, phase: Math.PI },
  // Knees fold backwards only, most while the leg swings through.
  { bone: 'calf_l', amp: -0.7, offset: 0, phase: Math.PI / 2, bend: true },
  { bone: 'calf_r', amp: -0.7, offset: 0, phase: Math.PI * 1.5, bend: true },
  { bone: 'upperarm_l', amp: 0.35, offset: 0, phase: Math.PI },
  { bone: 'upperarm_r', amp: 0.35, offset: 0, phase: 0 },
  { bone: 'lowerarm_l', amp: 0.15, offset: 0.35, phase: Math.PI },
  { bone: 'lowerarm_r', amp: 0.15, offset: 0.35, phase: 0 },
];

/** A resident's walk: round a circle at walking pace. */
export interface Walk {
  centre: [number, number];
  radius: number;
  /** Where on the circle it starts (radians) and its step's phase. */
  start: number;
}

/** Walking pace (m/s) and steps per second (both feet). */
const PACE = 1.1;
const STEPS = 1.8;

export interface Pose {
  /** The resident's place on the floor and its turn round y (radians). */
  x: number;
  z: number;
  yaw: number;
  /** Per swung bone (`SWUNG` order): its local rotation as a quaternion about x (x, y, z, w). */
  rotations: Float32Array;
}

/** The bone names the pose turns, in the order of `Pose.rotations`. */
export const SWUNG = SWING.map((s) => s.bone);

/** Where a resident is and how its limbs are turned `t` seconds in (the same in both engines). */
export function poseAt(walk: Walk, t: number, out: Pose): Pose {
  const theta = walk.start + (t * PACE) / walk.radius;
  out.x = walk.centre[0] + Math.cos(theta) * walk.radius;
  out.z = walk.centre[1] + Math.sin(theta) * walk.radius;
  // Facing -z at rest (see the file's comment): turned to face along the circle.
  out.yaw = Math.PI - theta;
  const step = (t * STEPS + walk.start) * Math.PI;
  SWING.forEach((s, i) => {
    const wave = Math.sin(step + s.phase);
    const angle = s.offset + s.amp * (s.bend ? Math.max(0, wave) : wave);
    out.rotations[i * 4] = Math.sin(angle / 2);
    out.rotations[i * 4 + 1] = 0;
    out.rotations[i * 4 + 2] = 0;
    out.rotations[i * 4 + 3] = Math.cos(angle / 2);
  });
  return out;
}

export function newPose(): Pose {
  return { x: 0, z: 0, yaw: 0, rotations: new Float32Array(SWING.length * 4) };
}

/** Where `count` residents walk: small circles round the house, men and women in turn. */
export function walks(count: number): Walk[] {
  return Array.from({ length: count }, (_, i) => {
    const around = (i / count) * Math.PI * 2;
    const ring = 4 + (i % 3) * 1.5;
    return { centre: [Math.cos(around) * ring, Math.sin(around) * ring], radius: 1 + (i % 4) * 0.35, start: i * 1.7 };
  });
}

/** One colour per look, shared by every resident (linear-ish sRGB values). */
export const LOOK_COLOR: Record<PartLook, [number, number, number]> = {
  skin: [0.8, 0.6, 0.48],
  eye: [0.92, 0.92, 0.9],
  lash: [0.06, 0.05, 0.05],
  mouth: [0.55, 0.25, 0.25],
  hair: [0.22, 0.14, 0.08],
  cloth: [0.2, 0.24, 0.34],
};
