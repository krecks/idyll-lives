/**
 * Builds renderable templates from asset entries: glTF files, or nothing when there's no file or
 * it doesn't load (a plain box for things placed, held or pictured, `BuildOptions.standIn`). A file
 * that fails is tried again in the background, and whoever drew the box builds it again once it
 * comes (`onModelFileArrived`, `TemplateCache`). Each template is drawn with thin instances (one
 * draw call per mesh, however many copies exist).
 *
 * Primitive parts (a character's limbs, the few parts a file can't carry) are merged per
 * (tint slot, finish) so a model costs one draw call per distinct finish. Boxes get rounded
 * edges so they catch highlights; every part gets metre-scale UVs so textured finishes
 * (fabric, wood, bark) tile at a realistic density.
 *
 * glTF entries may also carry (see `ModelExtras`):
 * - `fit`: scale the model (non-uniformly if needed) into an exact W x H x D box, bottom on the
 *   floor and centred on the footprint, so files from any source line up with the gameplay
 *   footprint and seat/bed heights.
 * - `materials`: restyle the file's materials by name: tint them, or replace them with one of the
 *   game's finishes (`material.finish.*`, vertex-coloured, box-projected UVs in metres). Replaced
 *   parts merge with primitive parts of the same finish, so a flat-coloured kit model costs a few
 *   draw calls and matches the rest of the furniture. Style variants reuse one file this way. A
 *   material with `mirror` is mirror glass: its face reflects (`mirrors`), as primitive parts' do.
 * - `parts`: extra primitive parts drawn with the file (books on a shelf, pillows, a lamp).
 */

import {
  Color3,
  Constants,
  StandardMaterial,
  ImportMeshAsync,
  TransformNode,
  PBRMaterial,
  Matrix,
  Mesh,
  MeshBuilder,
  Quaternion,
  Vector3,
  VertexBuffer,
  VertexData,
  type Scene,
} from './core';
import './loaders';
import type { AbstractMesh, AnimationGroup, Material, Node } from './core';
import { optimizedUrl } from '../../assets/registry';
import { background, foreground, prefetch } from '../../assets/stream';
import { paintSlots } from '../../assets/paint';
import type { ModelEntry, PrimitivePart, Vec3 } from '../../assets/types';
import type { PartColors } from '../../core/protocol';
import { DEFAULT_FINISH, encodeFinish, MaterialLibrary } from './materials';
import { WALL_HALF, WALL_HEIGHT } from './walls';
import type { Content, ObjectDef } from '../../content/content';

export type TintSlot = NonNullable<PrimitivePart['tint']>;
export type BoneName = NonNullable<PrimitivePart['bone']>;

export interface ModelTemplate {
  /** Meshes in model space; each receives the same instance matrices. */
  meshes: Mesh[];
  /** Per mesh: which `look` colour tints it, or null for none. */
  tints: (TintSlot | null)[];
  /** Per mesh: the limb it belongs to (characters), or null for the body. */
  bones: (BoneName | null)[];
  /** Height of the model's bounding box, for picking. */
  height: number;
  /** Bottom of its bounding box (above 0 for things on a wall or the ceiling), for picking. */
  bottom?: number;
  /** Its mirror glass, if any (parts with `mirror`). */
  mirrors?: MirrorSurface[];
  /** Its outline on the floor in model space (m: x and z, before turning), for picking it by what's drawn rather than its tiles. */
  outline?: { minX: number; maxX: number; minZ: number; maxZ: number };
  /** Its moving parts (a file's animated nodes: a fridge's doors) and the clips that move them. */
  motion?: ModelMotion;
  /**
   * What glows (materials with `glow`: a lamp's shade at night, a screen while it's watched): a copy
   * of those faces drawn added on top, per moving part (-1: the still body), off until the renderer
   * gives each instance its strength (thin instance `color`, 0..1) and turns it on. `screen` marks
   * the ones lit by use rather than by the room. Not in `meshes`: nothing else draws them.
   */
  glows?: { mesh: Mesh; part: number; screen?: boolean }[];
  /**
   * The files it was built without, as they failed to load: its far version (it's the full model
   * then), or every one (it's drawn as nothing, or the box asked for). It's built again when one of
   * them comes: by a cache (`TemplateCache.invalidate`), the landscape (`Landscape.rebuild`), the
   * selection marker.
   */
  failed?: string[];
  /** It's the stand-in box (`BuildOptions.standIn`): its file is missing, or failed to load or to build. */
  boxed?: boolean;
}

/** A template's meshes and its glows' (all it draws). */
export function withGlows(t: ModelTemplate): Mesh[] {
  return [...t.meshes, ...(t.glows ?? []).map((g) => g.mesh)];
}

/**
 * A model's moving parts: each is drawn by meshes of its own (at rest, where the file has it),
 * which an instance moves with a matrix of its own (`clipPose`, before its placement).
 */
export interface ModelMotion {
  /** Per mesh of the template: the moving part it belongs to, or -1 for the still body. */
  parts: number[];
  /** By the file's animation name. */
  clips: Map<string, ModelClip>;
  /**
   * Per moving part: its node's name, and where its node is at rest in model space (its vertices in its
   * own frame times this are where they're drawn; not resized). A part drawn on its own (a resident's held
   * instrument, `Characters`) is placed by its own frame.
   */
  names?: string[];
  rests?: Matrix[];
}

export interface ModelClip {
  /** Seconds. */
  duration: number;
  /**
   * Per part it moves (or carries: a part inside one it moves), its pose in model space, sampled
   * `CLIP_RATE` times a second (16 floats each), and `depth`, how deep in the file's nodes the
   * outermost node it moves of that part's is (0: the root's children), so a part posed by several
   * clips at once takes the inner ones' poses first (`partPose`).
   */
  tracks: { part: number; poses: Float32Array; depth: number }[];
}

/** Samples per second of a clip's poses. */
const CLIP_RATE = 30;

/**
 * Where `clip` puts `part` at `seconds` (clamped to the clip) into `out`, relative to rest: its
 * vertices at rest times `out` are where they are then. False (and `out` untouched) if the clip
 * doesn't move that part.
 */
export function clipPose(clip: ModelClip, part: number, seconds: number, out: Float32Array, offset = 0): boolean {
  const track = clip.tracks.find((t) => t.part === part);
  if (!track) return false;
  const last = track.poses.length / 16 - 1;
  const at = clamp(seconds * CLIP_RATE, 0, last);
  const i = Math.min(Math.floor(at), Math.max(0, last - 1));
  const f = last > 0 ? at - i : 0;
  const a = i * 16;
  const b = Math.min(i + 1, last) * 16;
  // Samples are close together: blending the matrices is as good as turning between them.
  for (let k = 0; k < 16; k++) out[offset + k] = track.poses[a + k] * (1 - f) + track.poses[b + k] * f;
  return true;
}

/**
 * Where the clips playing (`playing`, each at its seconds) put `part` together, into `out` (as
 * `clipPose`): one clip's pose as it is; several multiply, the clips that move the part itself
 * before those that move a part it is inside (a record spinning on an arm that swings up: the spin
 * and then the swing), so each clip of a nested part plays as if the others did too. On one node, a
 * crop's growth clip (`kind` `growth`) comes first and the others play on top of it, about the part's
 * rest origin: a veggie bed's plants sway as they stand at their stage, seedlings by their own small
 * height. False (`out` untouched) if none moves it.
 */
export function partPose(playing: readonly { clip: ModelClip; time: number; kind?: string }[], part: number, out: Float32Array, offset = 0): boolean {
  let found = 0;
  let first: { clip: ModelClip; time: number } | undefined;
  for (const p of playing) {
    if (!p.clip.tracks.some((t) => t.part === part)) continue;
    if (!found++) first = p;
  }
  if (found <= 1) return found === 1 && clipPose(first!.clip, part, first!.time, out, offset);
  const posed: { depth: number; growth: number; pose: Float32Array }[] = [];
  for (const p of playing) {
    const track = p.clip.tracks.find((t) => t.part === part);
    if (!track) continue;
    const pose = new Float32Array(16);
    clipPose(p.clip, part, p.time, pose);
    posed.push({ depth: track.depth, growth: p.kind === 'growth' ? 1 : 0, pose });
  }
  // Deepest first, and on one node the growth stage first (row vectors: a vertex times the first, then the next).
  posed.sort((a, b) => b.depth - a.depth || b.growth - a.growth);
  const acc = posed[0].pose;
  const tmp = new Float32Array(16);
  for (let n = 1; n < posed.length; n++) {
    const m = posed[n].pose;
    for (let i = 0; i < 4; i++) for (let j = 0; j < 4; j++) tmp[i * 4 + j] = acc[i * 4] * m[j] + acc[i * 4 + 1] * m[4 + j] + acc[i * 4 + 2] * m[8 + j] + acc[i * 4 + 3] * m[12 + j];
    acc.set(tmp);
  }
  out.set(acc, offset);
  return true;
}

/**
 * A mirror's reflecting face in model space: `matrix` takes the unit square (or, `round`, the
 * unit-diameter disc) in the xy plane, facing +z, onto the face, just in front of the glass.
 */
export interface MirrorSurface {
  round: boolean;
  matrix: Matrix;
  /** The moving part it's on (a mirror cabinet's door), whose pose it follows; absent or -1: the still body. */
  part?: number;
}

export interface BuildOptions {
  /** 0 = full detail, 1 = reduced tessellation for distant instances (a file: its `far` version, if it has one). */
  lod?: 0 | 1;
  /** Only seen from afar (the town overview): a file's `far` version, if it has one. */
  far?: boolean;
  /** Resized by the player: drawn at this size (see `resizeMaps`). */
  resize?: Resize;
  /**
   * A plain box to draw when the model has no file or its file fails, so the thing stays in view and
   * can be clicked: placed objects ask for one (the lot's, public places' seen from another lot, the
   * town's), buy mode's copy of one in hand, food and dishes, what residents carry, and the catalog's
   * pictures. The landscape, foliage, kerbside props and the selection marker draw nothing then,
   * until the file comes (`onModelFileArrived`).
   */
  standIn?: StandIn;
}

/** What shapes a stand-in box: the thing's content layer and, on a top, its radius. */
export interface StandIn {
  /** `item`: food, a dish, something in a hand: a small block (`ITEM_STAND_IN`). */
  layer: ObjectDef['layer'] | 'item';
  /** Things on a top: the radius (m) of the disc they take (`Content.radius`). */
  radius: number;
}

/** The stand-in of an item on a top or in a hand: a block about 10 cm across. */
export const ITEM_STAND_IN: StandIn = { layer: 'item', radius: 0.05 };

/** The box a kind of thing is drawn as without its model's file (`BuildOptions.standIn`). */
export function standInOf(content: Content, def: ObjectDef | undefined): StandIn {
  return { layer: def?.layer, radius: content.radius(def) };
}

/** A player's resizing: its own size (metres across and deep, before turning) and how much taller. */
export interface Resize {
  size: [number, number];
  /** Centimetres taller (negative: shorter). */
  tall: number;
  /** Things hanging from the ceiling grow downwards. */
  ceiling?: boolean;
}

export async function buildModel(
  scene: Scene,
  name: string,
  entry: ModelEntry | undefined,
  footprint: [number, number],
  options: BuildOptions = {},
): Promise<ModelTemplate> {
  const failed: string[] = [];
  if (entry?.url) {
    // Far off, the far version (without moving parts); the full one if that fails.
    const far = (options.far || options.lod === 1) && entry.far ? entry.far : undefined;
    const instead = (url: string) => (url === far ? 'the full model' : options.standIn ? 'a box' : 'nothing');
    for (const url of far ? [far, entry.url] : [entry.url]) {
      let file: LoadedFile;
      try {
        file = await loadFile(scene, url);
      } catch (err) {
        // Only a file that didn't load is `failed` (and comes later, `onModelFileArrived`).
        failed.push(url);
        // (Once per file: each build of it tries it again, and a page of things may share one.)
        if (!warnedFiles.has(url)) console.warn(`[render] failed to load ${url}, using ${instead(url)}`, err);
        warnedFiles.add(url);
        continue;
      }
      try {
        const template = buildGltf(scene, name, entry, file, footprint, options.resize);
        return failed.length ? { ...template, failed } : template;
      } catch (err) {
        // One that loads but can't be built won't build when tried again either: not `failed`.
        if (!warnedFiles.has(url)) console.warn(`[render] failed to build ${name} from ${url}, using ${instead(url)}`, err);
        warnedFiles.add(url);
      }
    }
  }
  // No file (the registry said so, once per key), it failed to load or to build: nothing, or the box
  // asked for. A cache keeps either; one whose file failed to load is built again when it comes
  // (`onModelFileArrived`).
  const template: ModelTemplate = options.standIn ? { ...standIn(scene, name, options.resize?.size ?? footprint, options.standIn, options.resize), boxed: true } : { meshes: [], tints: [], bones: [], height: 0 };
  return failed.length ? { ...template, failed } : template;
}

/** Files whose failure has been warned about (once per file for the page). */
const warnedFiles = new Set<string>();

/**
 * A plain neutral box standing in for a model (`BuildOptions.standIn`), placed as the model would be:
 * most of the footprint on the floor, a thin mat for a rug, a small block on a top (or a smaller one
 * for an item: food, a dish, something in a hand), a panel at eye height on the wall face, a block
 * hanging from the ceiling; taller as the player made it.
 */
function standIn(scene: Scene, name: string, [w, d]: [number, number], { layer, radius }: StandIn, resize?: Resize): ModelTemplate {
  // Its height as the player changed it, never flat or inside out (things shrink by up to a metre).
  const tall = (h: number) => Math.max(0.1, h + (resize?.tall ?? 0) / 100);
  const box = (size: Vec3, at: Vec3): PrimitivePart => ({ shape: 'box', size, at, color: '#B8B2AA' });
  const r = radius * 1.6;
  // On the wall: its back on the wall's face.
  const face = -d / 2 + WALL_HALF;
  const part =
    layer === 'item' ? box([radius * 2, radius * 2, radius * 2], [0, radius, 0])
    : layer === 'surface' ? box([r, tall(r), r], [0, tall(r) / 2, 0])
    : layer === 'rug' ? box([w * 0.9, 0.02, d * 0.9], [0, 0.01, 0])
    : layer === 'wall' ? box([w * 0.8, tall(0.6), 0.06], [0, 1.2 + tall(0.6) / 2, face + 0.03])
    : layer === 'ceiling' ? box([w * 0.5, tall(0.3), d * 0.5], [0, WALL_HEIGHT - tall(0.3) / 2, 0])
    : box([w * 0.8, tall(0.8), d * 0.8], [0, tall(0.8) / 2, 0]);
  const groups = new PartGroups(scene);
  groups.addPart(part, 0, 0);
  const { meshes, tints, bones } = groups.build(name);
  const { lo, hi } = partBox(part);
  return { meshes, tints, bones, height: groups.height, bottom: groups.bottom, outline: { minX: lo[0], maxX: hi[0], minZ: lo[2], maxZ: hi[2] } };
}

/** Share of a model's height above which everything rises with it (seats, tops, backs; sim-core `Top::of`). */
const RISE_FROM = 0.4;
/** Along the floor: the band at each edge (m, at most this share of half the width) that moves with the edge unstretched. */
const EDGE_BAND = 0.35;
const EDGE_SHARE = 0.4;

/** Whether `resize` changes a model of `from` tiles. */
function resizes(from: readonly [number, number], resize: Resize): boolean {
  return resize.size[0] !== from[0] || resize.size[1] !== from[1] || !!resize.tall;
}

/**
 * The resizing of a model of `from` tiles whose height runs from `bottom` to `top`, as a map per
 * axis (x, y, z) of where a point goes, and its slope there (how much the axis stretches).
 */
function resizeMaps(from: readonly [number, number], resize: Resize, bottom: number, top: number): { maps: ((v: number) => number)[]; slopes: ((v: number) => number)[]; span: number } {
  const [tw, td] = resize.size;
  const dy = resize.tall / 100;
  const span = Math.max(top - bottom, 1e-3);
  // Upward (or, from the ceiling, downward): what lies past the line moves by `dy`.
  const line = resize.ceiling ? top - RISE_FROM * span : bottom + RISE_FROM * span;
  const lift = (v: number) => (resize.ceiling ? (v < line ? v - dy : v) : v > line ? v + dy : v);
  // Along the floor: a band at each edge moves with it; the middle scales to fill.
  const across = (half: number, half2: number) => {
    const band = Math.min(EDGE_BAND, EDGE_SHARE * half);
    const band2 = Math.min(band, EDGE_SHARE * half2);
    const inner = half - band;
    const inner2 = half2 - band2;
    const map = (v: number) => {
      const a = Math.abs(v);
      const mapped = a <= inner ? (a * inner2) / Math.max(inner, 1e-6) : inner2 + ((a - inner) * band2) / band;
      return Math.sign(v) * mapped;
    };
    const slope = (v: number) => (Math.abs(v) <= inner ? inner2 / Math.max(inner, 1e-6) : band2 / band);
    return { map, slope };
  };
  const [x, z] = [across(from[0] / 2, tw / 2), across(from[1] / 2, td / 2)];
  return { maps: [x.map, lift, z.map], slopes: [x.slope, () => 1, z.slope], span };
}

/**
 * A file's geometry resized by `resizeMaps`, point by point: the edges move out keeping their
 * thickness, the middle stretches, and what lies above the rise line rises (a triangle across it
 * stretches). Normals follow the stretch.
 */
function resizeData(data: VertexData, maps: ((v: number) => number)[], slopes: ((v: number) => number)[]): void {
  const pos = data.positions!;
  const nor = data.normals;
  for (let i = 0; i < pos.length; i += 3) {
    const [x, y, z] = [pos[i], pos[i + 1], pos[i + 2]];
    if (nor) {
      // A normal scales by the inverse of the stretch.
      const n = [nor[i] / slopes[0](x), nor[i + 1] / slopes[1](y), nor[i + 2] / slopes[2](z)];
      const l = Math.hypot(n[0], n[1], n[2]) || 1;
      for (let k = 0; k < 3; k++) nor[i + k] = n[k] / l;
    }
    pos[i] = maps[0](x);
    pos[i + 1] = maps[1](y);
    pos[i + 2] = maps[2](z);
  }
}

/**
 * A part's exact bounds in model space (`lo`, `hi`: along x, y, z), turned as `partMesh` turns it:
 * a box by its corners, a cylinder or cone by its rings' corners, a round shape by its own extent (the
 * gate, crates/tools/src/checkglb.rs `bounds`, measures a file's boxes by their corners, turned the same
 * way). And which of its own axes lies along each model axis (-1: none; it's turned off the axes).
 */
function partBox(p: PrimitivePart): { lo: number[]; hi: number[]; along: number[] } {
  const [rx, ry, rz] = (p.rotate ?? [0, 0, 0]).map((d) => (d * Math.PI) / 180);
  const m = Matrix.FromQuaternionToRef(Quaternion.FromEulerAngles(rx, ry, rz), new Matrix()).m;
  // R[j][k]: model axis k of local axis j (row j: where local axis j goes).
  const r = (j: number, k: number) => m[j * 4 + k];
  const along = [0, 1, 2].map((k) => [0, 1, 2].find((j) => Math.abs(r(j, k)) > 0.999) ?? -1);
  const [sx, sy, sz] = p.size;
  const ellipsoid = (k: number, ax: number, ay: number, az: number) => Math.hypot(r(0, k) * ax, r(1, k) * ay, r(2, k) * az);
  const shape = p.shape as PrimitivePart['shape'] | 'torus' | 'lathe';
  let lo: number[];
  let hi: number[];
  if (shape === 'cylinder' || shape === 'cone') {
    // `CreateCylinder`'s rings: `segments` corners from angle 0, scaled in x and z.
    const n = p.segments ?? (shape === 'cone' ? 16 : 20);
    const top = shape === 'cone' ? (p.taper ?? 0) : (p.taper ?? 1);
    const points: number[][] = [];
    for (const [d, y] of [[1, -sy / 2], [top, sy / 2]]) {
      for (let j = 0; j < n; j++) points.push([(d / 2) * sx * Math.cos((2 * Math.PI * j) / n), y, (d / 2) * sz * Math.sin((2 * Math.PI * j) / n)]);
    }
    const dot = (q: number[], k: number) => q[0] * r(0, k) + q[1] * r(1, k) + q[2] * r(2, k);
    lo = [0, 1, 2].map((k) => Math.min(...points.map((q) => dot(q, k))));
    hi = [0, 1, 2].map((k) => Math.max(...points.map((q) => dot(q, k))));
  } else if (shape === 'lathe') {
    // Each profile point (radius, y) is a ring, scaled by the size.
    const profile = (p as PrimitivePart & ExtraShapes).profile ?? [[0, -0.5], [0.5, -0.5], [0.5, 0.5], [0, 0.5]];
    lo = [0, 1, 2].map((k) => Math.min(...profile.map(([d, y]) => r(1, k) * y * sy - d * Math.hypot(r(0, k) * sx, r(2, k) * sz))));
    hi = [0, 1, 2].map((k) => Math.max(...profile.map(([d, y]) => r(1, k) * y * sy + d * Math.hypot(r(0, k) * sx, r(2, k) * sz))));
  } else {
    const half = [0, 1, 2].map((k) => {
      if (shape === 'sphere' || shape === 'blob') return ellipsoid(k, sx / 2, sy / 2, sz / 2);
      // `sy` tall overall, radius sx/2, then scaled to sz in z.
      if (shape === 'capsule') return Math.abs(r(1, k)) * Math.max(0, sy / 2 - sx / 2) + ellipsoid(k, sx / 2, sx / 2, sz / 2);
      if (shape === 'torus') {
        // A ring (sx - sy across) swept by a tube sy thick, then scaled to sz in z.
        const [q, ring, tube] = [sz / sx, (sx - sy) / 2, sy / 2];
        return Math.hypot(r(0, k) * ring, r(2, k) * ring * q) + ellipsoid(k, tube, tube, tube * q);
      }
      // A box: model axis k gathers |R[j][k]| * size[j] from each local axis j.
      return [0, 1, 2].reduce((sum, j) => sum + Math.abs(r(j, k)) * p.size[j], 0) / 2;
    });
    lo = half.map((h) => -h);
    hi = half;
  }
  return { lo: p.at.map((c, k) => c + lo[k]), hi: p.at.map((c, k) => c + hi[k]), along };
}

/** The reflecting face of a mirror part (see `MirrorSurface`), placed as `partMesh` places the part. */
function mirrorSurface(p: PrimitivePart): MirrorSurface {
  const [sx, sy, sz] = p.size;
  const round = p.shape === 'cylinder';
  // The face's normal before the part turns: a disc's axis (y), a box's thinnest side.
  const axis = round ? 1 : [sx, sy, sz].indexOf(Math.min(sx, sy, sz));
  const [rx, ry, rz] = (p.rotate ?? [0, 0, 0]).map((d) => (d * Math.PI) / 180);
  const turn = Quaternion.FromEulerAngles(rx, ry, rz);
  const unit = (k: number, v: number) => new Vector3(k === 0 ? v : 0, k === 1 ? v : 0, k === 2 ? v : 0);
  const turned = unit(axis, 1).rotateByQuaternionToRef(turn, new Vector3());
  const sign = (turned.z >= 0) === ((p.mirror ?? 'front') === 'front') ? 1 : -1;
  // Unit square / disc in xy facing +z -> sized, onto the face on `axis` (a hair in front), turned and placed.
  const [w, h] = axis === 0 ? [sz, sy] : axis === 1 ? [sx, sz] : [sx, sy];
  const onto = axis === 0 ? Matrix.RotationY((sign * Math.PI) / 2) : axis === 1 ? Matrix.RotationX((-sign * Math.PI) / 2) : sign > 0 ? Matrix.Identity() : Matrix.RotationY(Math.PI);
  const offset = unit(axis, (p.size[axis] / 2 + 0.002) * sign);
  const matrix = Matrix.Scaling(w, h, 1)
    .multiply(onto)
    .multiply(Matrix.Translation(offset.x, offset.y, offset.z))
    .multiply(Matrix.Compose(Vector3.One(), turn, new Vector3(p.at[0], p.at[1], p.at[2])));
  return { round, matrix };
}

/**
 * Meshes grouped per (tint slot, finish, bone) and merged into one draw call each. Untextured
 * opaque finishes share one material: roughness/metalness go into the vertex alpha.
 */
class PartGroups {
  private readonly lib: MaterialLibrary;
  private readonly groups = new Map<string, { tint: TintSlot | null; finish: string; bone: BoneName | null; meshes: Mesh[] }>();
  height = 0;
  bottom = Infinity;

  constructor(private readonly scene: Scene) {
    this.lib = MaterialLibrary.for(scene);
  }

  /** Vertex alpha and group finish for a finish key. */
  finish(key: string): { alpha: number; group: string } {
    const info = this.lib.finishInfo(key);
    return info.plain ? { alpha: encodeFinish(info.roughness, info.metallic), group: PLAIN } : { alpha: 1, group: key };
  }

  addPart(part: PrimitivePart, lod: 0 | 1, index: number): Mesh {
    const { alpha, group } = this.finish(part.material ?? DEFAULT_FINISH);
    const mesh = partMesh(this.scene, part, lod, index, alpha);
    const box = partBox(part);
    this.height = Math.max(this.height, box.hi[1]);
    this.bottom = Math.min(this.bottom, box.lo[1]);
    this.add(mesh, part.tint ?? null, group, part.bone ?? null);
    return mesh;
  }

  add(mesh: Mesh, tint: TintSlot | null, finish: string, bone: BoneName | null): void {
    const key = `${tint ?? ''}|${finish}|${bone ?? ''}`;
    const group = this.groups.get(key) ?? { tint, finish, bone, meshes: [] };
    group.meshes.push(mesh);
    this.groups.set(key, group);
  }

  build(name: string): BuiltGroups {
    const built: BuiltGroups = { meshes: [], tints: [], bones: [] };
    for (const group of this.groups.values()) this.merge(name, group, built);
    return built;
  }

  /** Merges one group into a mesh (added to `built`). */
  private merge(name: string, { tint, finish, bone, meshes: group }: { tint: TintSlot | null; finish: string; bone: BoneName | null; meshes: Mesh[] }, built: BuiltGroups): Mesh | null {
    const merged = group.length === 1 ? group[0] : Mesh.MergeMeshes(group, true, true);
    if (!merged) return null;
    const suffix = finish === PLAIN ? '' : `@${finish.replace('material.finish.', '')}`;
    merged.name = `${name}${tint ? `:${tint}` : ''}${bone ? `#${bone}` : ''}${suffix}`;
    merged.material = finish === PLAIN ? this.lib.plainFinish() : this.lib.finish(finish);
    built.meshes.push(merged);
    built.tints.push(tint);
    built.bones.push(bone);
    return merged;
  }
}

/** A model's merged meshes, with each one's tint slot and bone. */
interface BuiltGroups {
  meshes: Mesh[];
  tints: (TintSlot | null)[];
  bones: (BoneName | null)[];
}

/** Group key for parts drawn with the shared plain finish. */
const PLAIN = 'plain';

function partMesh(scene: Scene, p: PrimitivePart, lod: 0 | 1, index: number, alpha: number): Mesh {
  const [sx, sy, sz] = p.size;
  const lo = lod === 1;
  let mesh: Mesh;
  let shade: ((nx: number, ny: number, nz: number) => number) | null = null;
  // `torus` and `lathe` extend the manifest schema (see `ExtraShapes`).
  const ext = p as PrimitivePart & ExtraShapes;
  switch (p.shape as PrimitivePart['shape'] | 'torus' | 'lathe') {
    case 'torus':
      // size: [outer diameter x, tube thickness, outer diameter z].
      mesh = MeshBuilder.CreateTorus('part', { diameter: Math.max(sx - sy, 1e-3), thickness: sy, tessellation: p.segments ?? (lo ? 12 : 24) }, scene);
      mesh.scaling.set(1, 1, sz / sx);
      break;
    case 'lathe':
      mesh = latheMesh(scene, ext.profile ?? [[0, -0.5], [0.5, -0.5], [0.5, 0.5], [0, 0.5]], p.segments ?? (lo ? 12 : 24));
      mesh.scaling.set(sx, sy, sz);
      break;
    case 'cylinder':
    case 'cone': {
      const top = p.shape === 'cone' ? (p.taper ?? 0) : (p.taper ?? 1);
      mesh = MeshBuilder.CreateCylinder(
        'part',
        { height: sy, diameterTop: top, diameterBottom: 1, tessellation: p.segments ?? (lo ? 8 : p.shape === 'cone' ? 16 : 20) },
        scene,
      );
      mesh.scaling.set(sx, 1, sz);
      break;
    }
    case 'sphere':
      mesh = MeshBuilder.CreateSphere('part', { diameterX: sx, diameterY: sy, diameterZ: sz, segments: p.segments ?? (lo ? 6 : 12) }, scene);
      break;
    case 'capsule':
      mesh = MeshBuilder.CreateCapsule('part', { height: sy, radius: sx / 2, tessellation: 16, capSubdivisions: 6 }, scene);
      mesh.scaling.set(1, 1, sz / sx);
      break;
    case 'blob':
      mesh = blobMesh(scene, sx, sy, sz, p.noise ?? 0.18, p.segments ?? (lo ? 1 : 2), index * 7.31 + sx * 3.7);
      // Fake ambient occlusion: undersides and the inner canopy are darker.
      shade = (_nx, ny) => 0.55 + 0.45 * smoothstep(-0.7, 0.75, ny);
      break;
    default: {
      const r = p.radius ?? Math.min(0.012, Math.min(sx, sy, sz) * 0.25);
      mesh = roundedBox(scene, sx, sy, sz, lo ? 0 : r);
    }
  }
  if (p.rotate) {
    const [rx, ry, rz] = p.rotate.map((d) => (d * Math.PI) / 180);
    mesh.rotationQuaternion = Quaternion.FromEulerAngles(rx, ry, rz);
  }
  mesh.position.set(p.at[0], p.at[1], p.at[2]);
  mesh.bakeCurrentTransformIntoVertices();
  boxProjectUVs(mesh);

  const c = Color3.FromHexString(p.color);
  const count = mesh.getTotalVertices();
  const colors = new Float32Array(count * 4);
  const positions = shade ? mesh.getVerticesData(VertexBuffer.PositionKind) : null;
  for (let i = 0; i < count; i++) {
    let k = 1;
    if (shade && positions) {
      // Use the direction from the part centre (not the bumpy normal) for smooth occlusion.
      const dx = (positions[i * 3] - p.at[0]) / (sx / 2);
      const dy = (positions[i * 3 + 1] - p.at[1]) / (sy / 2);
      const dz = (positions[i * 3 + 2] - p.at[2]) / (sz / 2);
      const len = Math.hypot(dx, dy, dz) || 1;
      k = shade(dx / len, dy / len, dz / len);
    }
    colors[i * 4] = c.r * k;
    colors[i * 4 + 1] = c.g * k;
    colors[i * 4 + 2] = c.b * k;
    colors[i * 4 + 3] = alpha;
  }
  mesh.setVerticesData(VertexBuffer.ColorKind, colors);
  return mesh;
}

/** Shapes beyond the base schema. */
interface ExtraShapes {
  /**
   * `lathe`: profile [radius, y] points in unit space (radius 0..0.5, y -0.5..0.5), revolved
   * around the vertical axis and scaled by `size`. Walk the outer surface upwards and any inner
   * surface downwards (the surface faces the right-hand side of the walk); bends sharper than
   * 40 degrees get a hard edge.
   */
  profile?: [number, number][];
}

/** Surface of revolution from a profile (see `ExtraShapes.profile`), centred on the origin. */
function latheMesh(scene: Scene, profile: [number, number][], segments: number): Mesh {
  // Per profile point: one vertex ring (smooth) or two (hard edge), each with its 2D normal.
  const rings: { r: number; y: number; nr: number; ny: number }[][] = [];
  const segNormal = (a: [number, number], b: [number, number]): [number, number] => {
    const dr = b[0] - a[0];
    const dy = b[1] - a[1];
    const l = Math.hypot(dr, dy) || 1;
    return [dy / l, -dr / l];
  };
  for (let i = 0; i < profile.length; i++) {
    const [r, y] = profile[i];
    const prev = i > 0 ? segNormal(profile[i - 1], profile[i]) : null;
    const next = i < profile.length - 1 ? segNormal(profile[i], profile[i + 1]) : null;
    if (prev && next && prev[0] * next[0] + prev[1] * next[1] > Math.cos((40 * Math.PI) / 180)) {
      const nr = prev[0] + next[0];
      const ny = prev[1] + next[1];
      const l = Math.hypot(nr, ny) || 1;
      rings.push([{ r, y, nr: nr / l, ny: ny / l }]);
    } else {
      const a = prev ?? next!;
      const b = next ?? prev!;
      rings.push([{ r, y, nr: a[0], ny: a[1] }, { r, y, nr: b[0], ny: b[1] }]);
    }
  }
  const positions: number[] = [];
  const normals: number[] = [];
  const indices: number[] = [];
  // Vertex rows: the incoming vertex of point i ends segment i-1, the outgoing one starts segment i.
  const rowStart: { in: number; out: number }[] = [];
  for (const ring of rings) {
    const starts: number[] = [];
    for (const v of ring) {
      starts.push(positions.length / 3);
      for (let k = 0; k <= segments; k++) {
        const a = (k / segments) * Math.PI * 2;
        const c = Math.cos(a);
        const s = Math.sin(a);
        positions.push(v.r * c, v.y, v.r * s);
        normals.push(v.nr * c, v.ny, v.nr * s);
      }
    }
    rowStart.push({ in: starts[0], out: starts[starts.length - 1] });
  }
  for (let i = 0; i < rings.length - 1; i++) {
    const a0 = rowStart[i].out;
    const b0 = rowStart[i + 1].in;
    for (let k = 0; k < segments; k++) {
      pushTri(positions, normals, indices, a0 + k, a0 + k + 1, b0 + k);
      pushTri(positions, normals, indices, a0 + k + 1, b0 + k + 1, b0 + k);
    }
  }
  const mesh = new Mesh('part', scene);
  const data = new VertexData();
  data.positions = positions;
  data.normals = normals;
  data.indices = indices;
  data.uvs = new Array((positions.length / 3) * 2).fill(0);
  data.applyToMesh(mesh, true);
  return mesh;
}

/** World-aligned planar UVs (metres) by dominant normal axis, so textures keep their scale. */
function boxProjectUVs(mesh: Mesh): void {
  const pos = mesh.getVerticesData(VertexBuffer.PositionKind);
  const nor = mesh.getVerticesData(VertexBuffer.NormalKind);
  if (!pos || !nor) return;
  const uvs = new Float32Array((pos.length / 3) * 2);
  for (let i = 0, j = 0; i < pos.length; i += 3, j += 2) {
    const ax = Math.abs(nor[i]);
    const ay = Math.abs(nor[i + 1]);
    const az = Math.abs(nor[i + 2]);
    if (ax >= ay && ax >= az) {
      uvs[j] = pos[i + 2];
      uvs[j + 1] = pos[i + 1];
    } else if (ay >= az) {
      uvs[j] = pos[i];
      uvs[j + 1] = pos[i + 2];
    } else {
      uvs[j] = pos[i];
      uvs[j + 1] = pos[i + 1];
    }
  }
  mesh.setVerticesData(VertexBuffer.UVKind, uvs);
}

/**
 * Box with rounded edges and corners (radius `r`), centred on the origin. Each face is a grid
 * whose outer rows are bent around the edge, so normals are smooth across the bevel.
 */
function roundedBox(scene: Scene, sx: number, sy: number, sz: number, r: number): Mesh {
  const half = [sx / 2, sy / 2, sz / 2];
  const radius = Math.max(0, Math.min(r, ...half.map((h) => h * 0.98)));
  const steps = radius > 0 ? 3 : 0;
  // Grid coordinates along one axis: the bevel band is subdivided, the flat middle is one span.
  const axis = (h: number): number[] => {
    if (steps === 0) return [-h, h];
    const inner = h - radius;
    const out: number[] = [];
    for (let i = 0; i <= steps; i++) out.push(-h + (radius * i) / steps);
    for (let i = 0; i <= steps; i++) out.push(inner + (radius * i) / steps);
    return out;
  };
  const positions: number[] = [];
  const normals: number[] = [];
  const indices: number[] = [];
  const p = new Vector3();
  const inner = new Vector3();
  // Faces: [normal axis, sign, u axis, v axis]
  const faces: [number, number, number, number][] = [
    [0, 1, 2, 1],
    [0, -1, 2, 1],
    [1, 1, 0, 2],
    [1, -1, 0, 2],
    [2, 1, 0, 1],
    [2, -1, 0, 1],
  ];
  for (const [n, sign, ua, va] of faces) {
    const us = axis(half[ua]);
    const vs = axis(half[va]);
    const base = positions.length / 3;
    for (const v of vs) {
      for (const u of us) {
        const q = [0, 0, 0];
        q[n] = sign * half[n];
        q[ua] = u;
        q[va] = v;
        p.set(q[0], q[1], q[2]);
        const lim = half.map((h) => h - radius);
        inner.set(clamp(p.x, -lim[0], lim[0]), clamp(p.y, -lim[1], lim[1]), clamp(p.z, -lim[2], lim[2]));
        const d = p.subtract(inner);
        const len = d.length();
        if (radius > 0 && len > 1e-6) {
          d.scaleInPlace(1 / len);
          positions.push(inner.x + d.x * radius, inner.y + d.y * radius, inner.z + d.z * radius);
          normals.push(d.x, d.y, d.z);
        } else {
          const nn = [0, 0, 0];
          nn[n] = sign;
          positions.push(p.x, p.y, p.z);
          normals.push(nn[0], nn[1], nn[2]);
        }
      }
    }
    const w = us.length;
    for (let j = 0; j < vs.length - 1; j++) {
      for (let i = 0; i < w - 1; i++) {
        const a = base + j * w + i;
        const b = a + 1;
        const c = a + w;
        const e = c + 1;
        pushTri(positions, normals, indices, a, b, c);
        pushTri(positions, normals, indices, b, e, c);
      }
    }
  }
  const mesh = new Mesh('part', scene);
  const data = new VertexData();
  data.positions = positions;
  data.normals = normals;
  data.indices = indices;
  data.uvs = new Array((positions.length / 3) * 2).fill(0);
  data.applyToMesh(mesh, true);
  return mesh;
}

/**
 * Adds a triangle wound the way Babylon expects front faces: the same orientation that
 * `VertexData.ComputeNormals` derives, i.e. (a - b) x (c - b) points along the vertex normal.
 */
function pushTri(pos: number[], nor: number[], out: number[], a: number, b: number, c: number): void {
  const ax = pos[a * 3] - pos[b * 3];
  const ay = pos[a * 3 + 1] - pos[b * 3 + 1];
  const az = pos[a * 3 + 2] - pos[b * 3 + 2];
  const cx = pos[c * 3] - pos[b * 3];
  const cy = pos[c * 3 + 1] - pos[b * 3 + 1];
  const cz = pos[c * 3 + 2] - pos[b * 3 + 2];
  const fx = ay * cz - az * cy;
  const fy = az * cx - ax * cz;
  const fz = ax * cy - ay * cx;
  const nx = nor[a * 3] + nor[b * 3] + nor[c * 3];
  const ny = nor[a * 3 + 1] + nor[b * 3 + 1] + nor[c * 3 + 1];
  const nz = nor[a * 3 + 2] + nor[b * 3 + 2] + nor[c * 3 + 2];
  if (fx * nx + fy * ny + fz * nz >= 0) out.push(a, b, c);
  else out.push(a, c, b);
}

/** Lumpy sphere (foliage clumps, rocks): an icosphere displaced by smooth 3D noise. */
function blobMesh(scene: Scene, sx: number, sy: number, sz: number, amount: number, subdivisions: number, seed: number): Mesh {
  const mesh = MeshBuilder.CreateIcoSphere('part', { radius: 0.5, subdivisions, flat: false, updatable: true }, scene);
  const pos = mesh.getVerticesData(VertexBuffer.PositionKind)!;
  for (let i = 0; i < pos.length; i += 3) {
    const x = pos[i] * 2;
    const y = pos[i + 1] * 2;
    const z = pos[i + 2] * 2;
    const n = noise3(x * 1.7 + seed, y * 1.7, z * 1.7) * 0.7 + noise3(x * 3.9, y * 3.9 + seed, z * 3.9) * 0.3;
    const k = 1 + (n - 0.5) * 2 * amount;
    pos[i] = pos[i] * k * sx;
    pos[i + 1] = pos[i + 1] * k * sy;
    pos[i + 2] = pos[i + 2] * k * sz;
  }
  const normals: number[] = [];
  VertexData.ComputeNormals(pos, mesh.getIndices()!, normals);
  // The icosphere repeats vertices along its base faces' edges: average their normals so the
  // lumps shade smoothly instead of in facets.
  const shared = new Map<string, number[]>();
  for (let i = 0; i < pos.length; i += 3) {
    const key = `${pos[i].toFixed(4)},${pos[i + 1].toFixed(4)},${pos[i + 2].toFixed(4)}`;
    const list = shared.get(key);
    if (list) list.push(i);
    else shared.set(key, [i]);
  }
  for (const list of shared.values()) {
    if (list.length < 2) continue;
    let [nx, ny, nz] = [0, 0, 0];
    for (const i of list) [nx, ny, nz] = [nx + normals[i], ny + normals[i + 1], nz + normals[i + 2]];
    const l = Math.hypot(nx, ny, nz) || 1;
    for (const i of list) [normals[i], normals[i + 1], normals[i + 2]] = [nx / l, ny / l, nz / l];
  }
  mesh.setVerticesData(VertexBuffer.PositionKind, pos);
  mesh.setVerticesData(VertexBuffer.NormalKind, normals);
  return mesh;
}

/** Optional model entry fields beyond the base schema (read defensively; see the file header). */
interface ModelExtras {
  /** Exact size [width x, height y, depth z] in metres after `rotationY`; replaces `scale`. */
  fit?: Vec3;
  /** With `fit`: `back` puts the back (-Z) of the model on the footprint's back edge instead of centring it. */
  align?: 'centre' | 'back';
  /** Restyles materials by glTF material name; `*` applies to every material not listed. */
  materials?: Record<string, MaterialOverride>;
  /** Extra primitive parts drawn with the model (in the final, fitted space). */
  parts?: PrimitivePart[];
  /** When the file's clips play (`objectMotion.ts`). */
  motions?: MotionsEntry;
}

/** A use's clip in a manifest's `motions.uses`: its name, or the name and its own `hold` (and `delay`, `every`, `slot`, `out`); or several of these. */
type UseEntry = string | { clip: string; hold?: number; delay?: number; every?: number; slot?: number; out?: number };

/** A model entry's `motions` as the manifest writes it: when its clips play (see `Motions`). */
interface MotionsEntry {
  uses?: Record<string, UseEntry | UseEntry[]>;
  hold?: number;
  open?: string;
  loops?: Record<string, string>;
  idle?: string;
  night?: string;
  growth?: string;
  ride?: string;
  clock?: ClockClips;
}

/** When a model's clips play: while residents use the object, and while it's open. */
export interface Motions {
  /**
   * The clips a use plays, by the interaction's id or, failing that, its animation tag (`fetch`,
   * `drink`), each with the seconds after the use starts that it plays back (a door that closes
   * again; absent: when the use ends). The manifest gives a clip's name (with the model's `hold`),
   * `{clip, hold}`, or a list of them (a shower's door shutting after 1 s while its water runs on).
   */
  uses: Record<string, readonly UseClip[]>;
  /**
   * The clip a use plays round and round while it lasts (a rocking chair, a washer's drum), by
   * interaction id or animation tag: from rest to rest, finishing its round when the use ends.
   */
  loops: Record<string, string>;
  /** The clip held open while the object is open (a wall bed folded down; the sim's `open`, set by `leaves`). */
  open?: string;
  /** The clip that goes round all the time, near the camera (a ceiling fan, a fountain's water; `ambientRange`). */
  idle?: string;
  /** The clip held open from dusk to dawn, while the lamps are lit (curtains drawn), whoever is asleep. */
  night?: string;
  /**
   * A crop's clip, held where its growth stage is (`GROWTH_TIMES`): ripe at rest, then picked,
   * growing and just planted along it, so each stage plays on to the next.
   */
  growth?: string;
  /** The clip whose (first) part carries the residents sitting or lying on the object: a rocking chair's seat, a swing. */
  ride?: string;
  /** A clock's hands, set from the game's time (`ClockClips`). */
  clock?: ClockClips;
}

export interface UseClip {
  clip: string;
  /** Seconds after it starts that it plays back (absent: when the use ends). */
  hold?: number;
  /** Seconds after the use starts that it starts (a net swishing after the shot). */
  delay?: number;
  /** Plays again every so many seconds while the use lasts, from rest to rest each time (a net swishing after each shot). */
  every?: number;
  /** Only for the resident in this slot (content `slots`, from 0: the one nearest its place, `slotOf`, as sim-core's `slot_position` has them; a bunk's 1 is the upper one). */
  slot?: number;
  /** Opens again for so many seconds when the use ends (a phone box's door letting the caller out). */
  out?: number;
}

/**
 * A clock's hands (`motions.clock`): each clip turns its hand once round from rest, the `hour` one
 * in twelve hours and the `minute` one in an hour, and the game holds each where the game's time
 * has it. `at`: the time the hands show at rest, in minutes past twelve (default 0: twelve o'clock).
 */
export interface ClockClips {
  hour?: string;
  minute?: string;
  at?: number;
}

/** Of a placed object's `colors`, those on parts its model lets the player paint; undefined for none. */
export function paintColors(entry: ModelEntry | undefined, colors: PartColors | undefined): PartColors | undefined {
  if (!colors) return undefined;
  const shown = Object.fromEntries(paintSlots(entry).flatMap(({ material }) => (colors[material] ? [[material, colors[material]]] : [])));
  return Object.keys(shown).length ? shown : undefined;
}

/**
 * `entry` with each part its `paint` names in its colour from `colors` (`#rrggbb` by material, a
 * placed object's colours), each keeping its finish (mapped by name or `*`); a material without a
 * finish has its file colour multiplied by it. The entry itself when nothing is painted.
 */
export function painted(entry: ModelEntry | undefined, colors: PartColors | undefined): ModelEntry | undefined {
  const shown = paintColors(entry, colors);
  if (!entry || !shown) return entry;
  const materials = { ...((entry as ModelEntry & ModelExtras).materials ?? {}) };
  for (const [name, color] of Object.entries(shown)) materials[name] = { ...(materials[name] ?? namedFinish(name) ?? materials['*']), color };
  return { ...entry, materials } as ModelEntry;
}

/** A model entry's `motions`, if it has any. */
export function motionsOf(entry: ModelEntry | undefined): Motions | undefined {
  const m = (entry as (ModelEntry & ModelExtras) | undefined)?.motions;
  if (!m) return undefined;
  const uses = Object.fromEntries(
    Object.entries(m.uses ?? {}).map(([use, value]) => [use, (Array.isArray(value) ? value : [value]).map((u) => (typeof u === 'string' ? { clip: u, hold: m.hold } : { ...u, hold: u.hold ?? m.hold }))]),
  );
  const loops = m.loops ?? {};
  const clock = m.clock?.hour || m.clock?.minute ? m.clock : undefined;
  return Object.keys(uses).length || Object.keys(loops).length || m.open || m.idle || m.night || m.growth || clock
    ? { uses, loops, open: m.open, idle: m.idle, night: m.night, growth: m.growth, ride: m.ride, clock }
    : undefined;
}

interface MaterialOverride {
  /** Replace with a game finish (`material.finish.*`) coloured by `color` (default: the file's colour). */
  finish?: string;
  /** sRGB hex: the finish colour, or (without `finish`) a multiplier on the file's base colour. */
  color?: string;
  roughness?: number;
  metallic?: number;
  /** Drop the meshes that use this material. */
  hide?: boolean;
  /**
   * Mirror glass, reflecting the room live (`mirrors.ts`) from its face towards the front (+z) or
   * the back: a flat piece (the face is across its thinnest side) of the still body, square or
   * round as it is drawn.
   */
  mirror?: 'front' | 'back';
  /**
   * Glows in this colour (sRGB hex): a lamp's shade at night, as the lamps come on and out with its
   * room's lights (`TemplateGlow`); a screen while a resident uses it, with `screen`.
   */
  glow?: string;
  /** A screen: lit only while a resident uses the object (a TV, a monitor), not with the room. */
  screen?: boolean;
}

/** Downloads the files of these models in the background, so they're at hand when shown. */
export function prefetchModels(entries: Iterable<ModelEntry | undefined>): void {
  prefetch([...entries].flatMap((e) => (e?.url ? [optimizedUrl(e.url)] : [])));
}

/** Idle time (ms) left below which `buildAhead` waits for the next idle moment. */
const IDLE_SLICE_MS = 6;

/**
 * Runs `jobs` in idle moments, as many as fit in each, until they are done or `current()` turns
 * false: models built ahead without holding up a frame.
 */
export function buildAhead(jobs: (() => void)[], current: () => boolean): void {
  const more = () => {
    if (jobs.length) idle(step);
  };
  const step = (deadline?: IdleDeadline) => {
    if (!current()) return;
    while (jobs.length) {
      jobs.shift()!();
      if (!deadline || deadline.timeRemaining() < IDLE_SLICE_MS) break;
    }
    more();
  };
  idle(step);
}

/** Runs `fn` when the page is idle (soon anyway; a short timeout where idle callbacks are missing). */
function idle(fn: (deadline?: IdleDeadline) => void, timeout = 2000): void {
  if ('requestIdleCallback' in window) requestIdleCallback(fn, { timeout });
  else setTimeout(fn, 50);
}

/**
 * The build's compressed copy of the file if there is one; the file itself if that fails to load.
 * A model being built is wanted `now`: background downloads wait for it. One tried again after it
 * failed runs in the background slot it was given (`scheduleRetry`).
 */
function importModel(url: string, scene: Scene, now: boolean): ReturnType<typeof ImportMeshAsync> {
  // Clips stay still: they're read (`readClips`), not played.
  const options = { pluginOptions: { gltf: { animationStartMode: 0 } } } as Parameters<typeof ImportMeshAsync>[2];
  const load = async () => {
    const copy = optimizedUrl(url);
    if (copy === url) return ImportMeshAsync(url, scene, options);
    try {
      return await ImportMeshAsync(copy, scene, options);
    } catch (err) {
      console.warn(`[render] compressed ${copy} failed, loading ${url}`, err);
      return ImportMeshAsync(url, scene, options);
    }
  };
  return now ? foreground(load) : load();
}

/**
 * A file as imported into a scene, once (`loadFile`): each of its meshes baked into model space at
 * rest, with its material. Every build of it (the lot's, the town's, buy mode's held copy, a
 * catalog picture, a resized copy) copies the geometry and shares the materials, so nothing of
 * the file is left behind when a build's meshes go.
 */
interface LoadedFile {
  pieces: FilePiece[];
  /** Its moving parts: the nodes its animations move, each with its pivot (the node's origin at rest, in model space) and its whole place then. */
  parts: { name: string; pivot: Vector3; rest: Matrix }[];
  /** Its animations, by name, in the file's model space. */
  clips: Map<string, ModelClip>;
}

interface FilePiece {
  /** In the file's model space (its nodes' transforms baked in); builds copy it, never change it. */
  data: VertexData;
  /** The file's own material, kept as loaded: builds draw with copies of it (`fileMaterial`). */
  material: Material | null;
  /** The moving part it belongs to, or -1. */
  part: number;
  /** Where its part is grown, for a part scaled to nothing at rest: the shape its normals and UVs come from. */
  full?: VertexData;
}

/**
 * After a file fails to load, how long (ms) until it's tried again in the background: each try that
 * fails waits for the next, and after the last nothing more is tried (until the browser comes back
 * online, when it starts over). Builds meanwhile still try it themselves, as ever.
 */
const RETRY_AFTER_MS = [5_000, 30_000, 120_000];

/** A scene's imported files and the ones that failed (they go with the scene). */
interface SceneFiles {
  /** Imported (or being imported), by URL. A failed one is dropped, so the next build tries it again. */
  files: Map<string, Promise<LoadedFile>>;
  /** Failed to load, by URL: how many background tries have been made, and the next one's timer. */
  failed: Map<string, { tries: number; timer?: ReturnType<typeof setTimeout> }>;
  /** Told when a file that failed has come (`onModelFileArrived`). */
  arrived: Set<(url: string) => void>;
}

const sceneFiles = new WeakMap<Scene, SceneFiles>();

function filesOf(scene: Scene): SceneFiles {
  const known = sceneFiles.get(scene);
  if (known) return known;
  const state: SceneFiles = { files: new Map(), failed: new Map(), arrived: new Set() };
  sceneFiles.set(scene, state);
  // Back online: what failed (in an outage, most likely) is tried again from the start.
  const online = () => {
    for (const [url, f] of state.failed) {
      clearTimeout(f.timer);
      f.timer = undefined;
      f.tries = 0;
      scheduleRetry(scene, state, url);
    }
  };
  window.addEventListener('online', online);
  scene.onDisposeObservable.addOnce(() => {
    window.removeEventListener('online', online);
    for (const f of state.failed.values()) clearTimeout(f.timer);
    state.failed.clear();
    state.arrived.clear();
  });
  return state;
}

/**
 * Calls `fn` with a file's URL when it loads in `scene` after failing to (in a background try, or
 * for any build): whatever was built without it (a box, or nothing) can be built again. Returns
 * what stops it. A scene nobody listens to doesn't try its failed files in the background.
 */
export function onModelFileArrived(scene: Scene, fn: (url: string) => void): () => void {
  const { arrived } = filesOf(scene);
  arrived.add(fn);
  return () => arrived.delete(fn);
}

/**
 * Tells `scene`'s listeners (`onModelFileArrived`) again that the file at `url` has come: a template
 * still being built when it came turned out to be built without it (`TemplateCache`).
 */
export function retellModelFileArrived(scene: Scene, url: string): void {
  tellArrived(filesOf(scene), url);
}

/**
 * The file at `url` as imported into `scene`, once. A failed import isn't kept, so each build tries
 * it again (none is held back or failed at once because of an earlier failure), and it's tried in the
 * background too (`scheduleRetry`). Only the import counts as the file failing, not a build from it
 * (`buildFromFile`): a file that loads but can't be built isn't fetched again. A background try imports in the slot it was given (`now` false), not as a load
 * someone waits for; a build that wants the file meanwhile shares the download already under way.
 */
function loadFile(scene: Scene, url: string, now = true): Promise<LoadedFile> {
  const state = filesOf(scene);
  let file = state.files.get(url);
  if (!file) {
    const loading = importFile(scene, url, now);
    state.files.set(url, (file = loading));
    // (Both outcomes are handled here; whoever asked handles the rejection it's given.)
    loading.then(
      () => fileArrived(state, url),
      () => state.files.get(url) === loading && state.files.delete(url),
    );
  }
  file.catch(() => {
    if (!state.failed.has(url)) state.failed.set(url, { tries: 0 });
    scheduleRetry(scene, state, url);
  });
  return file;
}

/** A file loaded: one that had failed isn't tried in the background any more, and the listeners are told. */
function fileArrived(state: SceneFiles, url: string): void {
  const f = state.failed.get(url);
  if (!f) return;
  clearTimeout(f.timer);
  state.failed.delete(url);
  tellArrived(state, url);
}

function tellArrived(state: SceneFiles, url: string): void {
  for (const fn of [...state.arrived]) {
    try {
      fn(url);
    } catch (err) {
      console.error(`[render] building again with ${url} failed`, err);
    }
  }
}

/** Sets the next background try of a failed file, unless one is set already or none are left. */
function scheduleRetry(scene: Scene, state: SceneFiles, url: string): void {
  const f = state.failed.get(url);
  if (!f || f.timer !== undefined || f.tries >= RETRY_AFTER_MS.length || !state.arrived.size || scene.isDisposed) return;
  // A load under way (a build's) is this try; if it fails, the next is set from here. Otherwise the
  // try loads the file itself.
  const sharedTry = () => {
    const loading = state.files.get(url);
    if (loading) loading.catch(() => scheduleRetry(scene, state, url));
    return !!loading;
  };
  f.timer = setTimeout(() => {
    f.timer = undefined;
    f.tries++;
    if (scene.isDisposed || !state.failed.has(url) || sharedTry()) return;
    // It waits its turn behind what's on screen; by then the file may have come, or a build be loading it.
    void background(async () => {
      if (scene.isDisposed || !state.failed.has(url) || sharedTry()) return;
      // `loadFile` notes the outcome (and sets the next try); the rejection itself is nobody's to handle.
      await loadFile(scene, url, false).catch(() => {});
    }).catch(() => {});
  }, RETRY_AFTER_MS[f.tries]);
}

async function importFile(scene: Scene, url: string, now: boolean): Promise<LoadedFile> {
  const result = await importModel(url, scene, now);
  for (const group of result.animationGroups) group.stop();
  // The nodes the clips move: each, with all below it, is a moving part.
  const moved = [...new Set(result.animationGroups.flatMap((g) => g.targetedAnimations.map((t) => t.target)))].filter((t): t is TransformNode => t instanceof TransformNode);
  const atRest = (group: AnimationGroup) => {
    for (const { animation, target } of group.targetedAnimations) (target as Record<string, unknown>)[animation.targetProperty] = animation.evaluate(group.from);
  };
  // At rest: where each clip starts.
  result.animationGroups.forEach(atRest);
  const rest = moved.map((node) => computeFromRoot(node).clone());
  const partOf = (node: Node | null): number => {
    for (let n = node; n; n = n.parent) {
      const i = moved.indexOf(n as TransformNode);
      if (i >= 0) return i;
    }
    return -1;
  };
  // A part grown from nothing (water, flames: scaled to almost 0 at rest) bakes flat, so its normals and the
  // finishes' UVs come from where a clip ends with it largest (`FilePiece.full`).
  const grownBy = moved.map((): AnimationGroup | null => null);
  const largest = rest.map((m) => Math.abs(m.determinant()));
  for (const group of result.animationGroups) {
    for (const { animation, target } of group.targetedAnimations) (target as Record<string, unknown>)[animation.targetProperty] = animation.evaluate(group.to);
    moved.forEach((node, i) => {
      const size = Math.abs(computeFromRoot(node).determinant());
      if (size > largest[i]) [largest[i], grownBy[i]] = [size, group];
    });
    atRest(group);
  }
  moved.forEach((_, i) => {
    if (grownBy[i] && Math.abs(rest[i].determinant()) > largest[i] * 0.01) grownBy[i] = null;
  });
  const pieces: FilePiece[] = [];
  for (const node of result.meshes) {
    if (!(node instanceof Mesh) || node.getTotalVertices() === 0) continue;
    const part = partOf(node);
    // Bake the full hierarchy transform so thin-instance matrices place the model directly.
    const piece: FilePiece = { data: VertexData.ExtractFromMesh(node, true, true).transform(computeFromRoot(node)), material: node.material, part };
    const group = part >= 0 ? grownBy[part] : null;
    if (group) {
      // Its normals at rest by the inverse transpose of its matrix (flattened, they lean towards the
      // grow axis): the game's instance matrix (the clip's pose) turns them back into the grown shape's.
      restNormals(piece.data, VertexData.ExtractFromMesh(node, true, true).normals, computeFromRoot(node));
      for (const { animation, target } of group.targetedAnimations) (target as Record<string, unknown>)[animation.targetProperty] = animation.evaluate(group.to);
      piece.full = VertexData.ExtractFromMesh(node, true, true).transform(computeFromRoot(node));
      atRest(group);
    }
    pieces.push(piece);
  }
  const parts = moved.map((node, i) => ({ name: node.name, pivot: rest[i].getTranslation(), rest: rest[i] }));
  // (Each read with the others at rest: a part inside another is read with it closed.)
  const clips = new Map(
    result.animationGroups.map((group) => {
      const clip = readClip(group, moved, rest);
      atRest(group);
      return [group.name, clip];
    }),
  );
  result.animationGroups.forEach(atRest);
  // The nodes and clips go; the materials stay with the file.
  for (const node of [...result.transformNodes, ...result.meshes]) node.dispose(true, false);
  for (const group of result.animationGroups) group.dispose();
  return { pieces, parts, clips };
}

/** Sets `data`'s normals to `local` (a node's own) taken to model space by `world`'s inverse transpose, as a normal goes. */
function restNormals(data: VertexData, local: ArrayLike<number> | null | undefined, world: Matrix): void {
  if (!local || !data.normals) return;
  const m = world.clone().invert().transpose();
  const v = new Vector3();
  const normals = Float32Array.from(local);
  for (let i = 0; i < normals.length; i += 3) {
    Vector3.TransformNormalFromFloatsToRef(normals[i], normals[i + 1], normals[i + 2], m, v);
    v.normalize();
    normals[i] = v.x;
    normals[i + 1] = v.y;
    normals[i + 2] = v.z;
  }
  data.normals = normals;
}

/**
 * A clip's poses of the parts it moves (`ModelClip`): each node's world matrix then, relative to its
 * world matrix at rest. The parts inside one it moves (a fridge's lever on its door) go with it.
 */
function readClip(group: AnimationGroup, moved: TransformNode[], rest: Matrix[]): ModelClip {
  const animations = group.targetedAnimations;
  const fps = animations[0]?.animation.framePerSecond || 60;
  const duration = Math.max(0, (group.to - group.from) / fps);
  const samples = Math.max(2, Math.ceil(duration * CLIP_RATE) + 1);
  const animated = new Set(animations.map((a) => a.target));
  // Per moving part: the depth of the outermost node of it, or around it, that the clip moves (-1: none).
  const depthOf = (node: TransformNode): number => {
    let depth = -1;
    let d = 0;
    for (let n: Node | null = node.parent; n; n = n.parent) d++;
    for (let n: Node | null = node; n; n = n.parent, d--) if (animated.has(n)) depth = d;
    return depth;
  };
  const targets = moved.filter((node) => depthOf(node) >= 0);
  const tracks = targets.map((node) => ({ part: moved.indexOf(node), poses: new Float32Array(samples * 16), depth: depthOf(node) }));
  const inverse = targets.map((node) => rest[moved.indexOf(node)].clone().invert());
  const pose = new Matrix();
  for (let s = 0; s < samples; s++) {
    const frame = group.from + Math.min(s / CLIP_RATE, duration) * fps;
    for (const { animation, target } of animations) (target as Record<string, unknown>)[animation.targetProperty] = animation.evaluate(frame);
    targets.forEach((node, i) => {
      // Vertices at rest, back into the node's own space, then out to where the node is now.
      inverse[i].multiplyToRef(computeFromRoot(node), pose);
      pose.copyToArray(tracks[i].poses, s * 16);
    });
  }
  return { duration, tracks };
}

/** A node's world matrix, its parents' worked out first (they may have moved too). */
function computeFromRoot(node: TransformNode): Matrix {
  const chain: TransformNode[] = [];
  for (let n: Node | null = node; n; n = n.parent) if (n instanceof TransformNode) chain.unshift(n);
  for (const n of chain) n.computeWorldMatrix(true);
  return node.getWorldMatrix();
}

/** The arrays of `data`, copied. */
function copyData(data: VertexData): VertexData {
  const copy = new VertexData();
  for (const key of ['positions', 'normals', 'tangents', 'uvs', 'uvs2', 'colors', 'indices'] as const) {
    const values = data[key];
    if (values) (copy as unknown as Record<string, unknown>)[key] = values.slice();
  }
  return copy;
}

/** A material named for its finish, `stone@matte` (`tunic@fabric#body`): drawn with `material.finish.matte` (unless the entry maps it), if `lib` has it. */
function namedFinish(name: string, lib?: MaterialLibrary): MaterialOverride | undefined {
  const named = /.@([a-z]+)$/.exec(name.replace(TINT_IN_NAME, ''));
  const finish = named && `material.finish.${named[1]}`;
  return finish && (!lib || lib.hasFinish(finish)) ? { finish } : undefined;
}

/** A tint slot in a material's name: `tunic#body` is tinted as a primitive part with `tint: 'body'` is. */
const TINT_IN_NAME = /#(body|skin|hair)(?=@|\||$)/;

/** The tint slot a file's material carries in its name (`TINT_IN_NAME`), or null. */
function namedTint(material: Material | null): TintSlot | null {
  return (material && (TINT_IN_NAME.exec(material.name)?.[1] as TintSlot | undefined)) ?? null;
}

/** Copies of a file's materials as entries draw them (as loaded, or restyled by `color`, `roughness`, `metallic`), by restyling. */
const materialCopies = new WeakMap<Material, Map<string, Material>>();

/**
 * The material a build draws a file's own `material` with: a copy per restyling, made once (the
 * file's stays as loaded, for the next entry's restyling), and adopted into the game's lighting.
 */
function fileMaterial(lib: MaterialLibrary, material: Material | null, ov: MaterialOverride | undefined): Material | null {
  if (!(material instanceof PBRMaterial)) return material;
  const key = `${ov?.color ?? ''}|${ov?.roughness ?? ''}|${ov?.metallic ?? ''}`;
  const copies = materialCopies.get(material) ?? new Map<string, Material>();
  materialCopies.set(material, copies);
  let copy = copies.get(key);
  if (!copy) {
    const made = material.clone(`${material.name}${key === '||' ? '' : `|${key}`}`);
    if (ov?.color) made.albedoColor = material.albedoColor.multiply(Color3.FromHexString(ov.color).toLinearSpace());
    if (ov?.roughness !== undefined) made.roughness = ov.roughness;
    if (ov?.metallic !== undefined) made.metallic = ov.metallic;
    lib.adopt(made);
    copies.set(key, (copy = made));
  }
  return copy;
}

/** A template from a file imported (`loadFile`); throws if the build fails, having let go of what it made. */
function buildGltf(scene: Scene, name: string, entry: ModelEntry, file: LoadedFile, footprint: [number, number], resize: Resize | undefined): ModelTemplate {
  // What the build makes, let go of if it throws part way. It runs without a break, so nothing else
  // adds meshes meanwhile.
  const made: AbstractMesh[] = [];
  const watching = scene.onNewMeshAddedObservable.add((mesh) => made.push(mesh));
  try {
    return buildFromFile(scene, name, entry, file, footprint, resize);
  } catch (err) {
    for (const mesh of made) if (!mesh.isDisposed()) mesh.dispose(false, false);
    throw err;
  } finally {
    scene.onNewMeshAddedObservable.remove(watching);
  }
}

/** `buildGltf`'s build. */
function buildFromFile(scene: Scene, name: string, entry: ModelEntry, file: LoadedFile, footprint: [number, number], resize?: Resize): ModelTemplate {
  const extras = entry as ModelEntry & ModelExtras;
  const fit = extras.fit;
  const s = fit ? 1 : (entry.scale ?? 1);
  const [ox, oy, oz] = entry.offset ?? [0, 0, 0];
  // The file's model space to this build's: `scale`, `rotationY` and `offset`, or `fit`.
  let space = Matrix.Compose(
    new Vector3(s, s, s),
    Quaternion.RotationAxis(Vector3.Up(), ((entry.rotationY ?? 0) * Math.PI) / 180),
    fit ? Vector3.Zero() : new Vector3(ox, oy, oz),
  );
  const overrides = extras.materials ?? {};
  const finishes = MaterialLibrary.for(scene);
  const overrideFor = (m: Material | null): MaterialOverride | undefined => (m && (overrides[m.name] ?? namedFinish(m.name, finishes))) ?? overrides['*'];

  // This build's copy of the geometry it shows.
  const pieces = file.pieces.filter((p) => !overrideFor(p.material)?.hide).map((p) => ({ data: copyData(p.data).transform(space), full: p.full && copyData(p.full).transform(space), material: p.material, part: p.part }));
  if (fit && pieces.length) space = space.multiply(fitInto(pieces.map((p) => p.data), fit, extras.align ?? 'centre', [ox, oy, oz]));
  const boxes = pieces.map((p) => dataBox(p.data));
  // Resized by the player as primitive parts are; each moving part's pivot goes where its hinge went.
  const pivots = file.parts.map((p) => Vector3.TransformCoordinates(p.pivot, space));
  const shifts = pivots.map(() => Vector3.Zero());
  if (resize && boxes.length && resizes(footprint, resize)) {
    const { maps, slopes } = resizeMaps(footprint, resize, Math.min(...boxes.map((b) => b.lo[1])), Math.max(...boxes.map((b) => b.hi[1])));
    for (const p of pieces) for (const data of p.full ? [p.data, p.full] : [p.data]) resizeData(data, maps, slopes);
    pivots.forEach((p, i) => shifts[i].set(maps[0](p.x) - p.x, maps[1](p.y) - p.y, maps[2](p.z) - p.z));
    boxes.splice(0, boxes.length, ...pieces.map((p) => dataBox(p.data)));
  }

  // The still body, then each moving part: meshes of their own (merged per finish within each).
  const lib = MaterialLibrary.for(scene);
  const meshes: Mesh[] = [];
  const tints: (TintSlot | null)[] = [];
  const parts: number[] = [];
  let height = 0;
  let bottom = Infinity;
  for (let part = -1; part < file.parts.length; part++) {
    const groups = new PartGroups(scene);
    const kept: Mesh[] = [];
    pieces.forEach(({ data, full, material, part: of }, i) => {
      if (of !== part) return;
      height = Math.max(height, boxes[i].hi[1]);
      bottom = Math.min(bottom, boxes[i].lo[1]);
      const ov = overrideFor(material);
      if (ov?.finish) {
        groups.add(restyledMesh(scene, data, material, ov, groups.finish(ov.finish).alpha, full), namedTint(material), groups.finish(ov.finish).group, null);
        return;
      }
      const mesh = new Mesh(name, scene);
      data.applyToMesh(mesh);
      mesh.material = fileMaterial(lib, material, ov);
      kept.push(mesh);
    });
    if (part < 0) {
      extras.parts?.forEach((p, i) => groups.addPart(p, 0, i));
      height = Math.max(height, groups.height);
      bottom = Math.min(bottom, groups.bottom);
    }
    const merged = mergeByMaterial(kept, part < 0 ? name : `${name}|${file.parts[part].name}`);
    const restyled = groups.build(part < 0 ? name : `${name}|${file.parts[part].name}`);
    // A part grown from nothing lies flat where it hides at rest: leaf flutter would lift it into view (nature.ts).
    if (pieces.some((p) => p.part === part && p.full)) for (const mesh of restyled.meshes) mesh.metadata = { ...mesh.metadata, windStill: true };
    meshes.push(...merged, ...restyled.meshes);
    tints.push(...merged.map((m) => namedTint(m.material)), ...restyled.tints);
    parts.push(...[...merged, ...restyled.meshes].map(() => part));
  }

  const lo = (k: number) => Math.min(...boxes.map((b) => b.lo[k]));
  const hi = (k: number) => Math.max(...boxes.map((b) => b.hi[k]));
  const outline = boxes.length ? { minX: lo(0), maxX: hi(0), minZ: lo(2), maxZ: hi(2) } : undefined;
  // Mirror glass: the pieces of a `mirror` material (on a moving part, following it), and primitive parts drawn with the file.
  const mirrors = [
    ...pieces.flatMap(({ data, material, part }, i) => {
      const side = overrideFor(material)?.mirror;
      return side ? [{ ...fileMirror(data, boxes[i], side), part }] : [];
    }),
    ...(extras.parts ?? []).filter((p) => p.mirror).map(mirrorSurface),
  ];
  // What glows (a lamp's shade at night, a screen while it's used): its faces again, in the glow's
  // colour, a mesh per part they're on.
  const glowing = Map.groupBy(
    pieces.filter(({ material }) => overrideFor(material)?.glow),
    ({ part }) => part,
  );
  const glows = [...glowing].map(([part, list]) => ({
    part,
    // A screen (a TV, a monitor: its `screen`, `crt` or `projection` material, or a manifest's
    // `screen`) is only lit while someone is using it; a lamp's shade or a fire glows with its room.
    screen: list.some(({ material }) => overrideFor(material)?.screen ?? /screen|crt|projection/i.test(material?.name ?? '')),
    mesh: glowMesh(scene, `${name}|glow${part < 0 ? '' : `|${file.parts[part].name}`}`, list.map(({ data, material }) => ({ data, color: overrideFor(material)!.glow! }))),
  }));
  return {
    meshes,
    tints,
    bones: meshes.map(() => null),
    height,
    bottom,
    outline,
    ...(mirrors.length ? { mirrors } : {}),
    ...(glows.length ? { glows } : {}),
    ...(file.parts.length && file.clips.size
      ? { motion: { parts, clips: buildClips(file.clips, space, shifts), names: file.parts.map((p) => p.name), rests: file.parts.map((p) => p.rest.multiply(space)) } }
      : {}),
  };
}

/**
 * A file's clips in a build's model space: the file's `space` (`scale`, `rotationY`, `offset`,
 * `fit`) taken into account, and each part's pose moved with its pivot (`shifts`) when resized.
 */
function buildClips(clips: Map<string, ModelClip>, space: Matrix, shifts: Vector3[]): Map<string, ModelClip> {
  const inverse = space.clone().invert();
  const pose = new Matrix();
  const out = new Matrix();
  return new Map(
    [...clips].map(([name, clip]) => [
      name,
      {
        duration: clip.duration,
        tracks: clip.tracks.map(({ part, poses, depth }) => {
          const shift = shifts[part];
          const [back, on] = [Matrix.Translation(-shift.x, -shift.y, -shift.z), Matrix.Translation(shift.x, shift.y, shift.z)];
          const built = new Float32Array(poses.length);
          for (let o = 0; o < poses.length; o += 16) {
            Matrix.FromArrayToRef(poses, o, pose);
            // Back to where the part was before resizing, into the file's space, posed, and out again.
            back.multiplyToRef(inverse, out);
            out.multiplyToRef(pose, out);
            out.multiplyToRef(space, out);
            out.multiplyToRef(on, out);
            out.copyToArray(built, o);
          }
          return { part, poses: built, depth };
        }),
      },
    ]),
  );
}

/**
 * The reflecting face of a file's mirror glass (`MaterialOverride.mirror`), as `mirrorSurface` of a
 * primitive part: across the glass's thinnest side, a hair in front of the face towards `side`
 * (front: +z, or +x / +y for glass turned that way). Round if its triangles facing that way cover
 * well under its box (a disc covers π/4 of its square).
 */
function fileMirror(data: VertexData, { lo, hi }: { lo: number[]; hi: number[] }, side: 'front' | 'back'): MirrorSurface {
  const size = [0, 1, 2].map((k) => hi[k] - lo[k]);
  const axis = size.indexOf(Math.min(...size));
  const sign = side === 'front' ? 1 : -1;
  const [w, h] = axis === 0 ? [size[2], size[1]] : axis === 1 ? [size[0], size[2]] : [size[0], size[1]];
  // How much of the box the face covers: the triangles facing `side`, measured across `axis`.
  const pos = data.positions ?? [];
  const nor = data.normals ?? [];
  const idx = data.indices ?? Array.from({ length: pos.length / 3 }, (_, i) => i);
  const [u, v] = [0, 1, 2].filter((k) => k !== axis);
  let area = 0;
  for (let t = 0; t + 2 < idx.length; t += 3) {
    const [a, b, c] = [idx[t] * 3, idx[t + 1] * 3, idx[t + 2] * 3];
    if ((nor[a + axis] + nor[b + axis] + nor[c + axis]) * sign <= 0) continue;
    area += Math.abs((pos[b + u] - pos[a + u]) * (pos[c + v] - pos[a + v]) - (pos[c + u] - pos[a + u]) * (pos[b + v] - pos[a + v])) / 2;
  }
  const round = w > 0 && h > 0 && area / (w * h) < 0.9;
  const unit = (k: number, x: number) => new Vector3(k === 0 ? x : 0, k === 1 ? x : 0, k === 2 ? x : 0);
  const onto = axis === 0 ? Matrix.RotationY((sign * Math.PI) / 2) : axis === 1 ? Matrix.RotationX((-sign * Math.PI) / 2) : sign > 0 ? Matrix.Identity() : Matrix.RotationY(Math.PI);
  const centre = new Vector3((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, (lo[2] + hi[2]) / 2).addInPlace(unit(axis, (size[axis] / 2 + 0.002) * sign));
  return { round, matrix: Matrix.Scaling(w, h, 1).multiply(onto).multiply(Matrix.Translation(centre.x, centre.y, centre.z)) };
}

/** The material glows are drawn with: unlit, added to what's behind, coloured per vertex and per instance. */
const glowMaterials = new WeakMap<Scene, StandardMaterial>();

/**
 * One mesh of the glowing faces (`MaterialOverride.glow`), each piece in its glow colour, a hair in
 * front of the faces it lights; off (the renderer turns it on at night).
 */
function glowMesh(scene: Scene, name: string, pieces: { data: VertexData; color: string }[]): Mesh {
  let material = glowMaterials.get(scene);
  if (!material) {
    material = new StandardMaterial('glow', scene);
    // Unlit: the emissive white, times the vertex colour (the glow's) and the instance's strength.
    material.disableLighting = true;
    material.diffuseColor = Color3.Black();
    material.emissiveColor = Color3.White();
    material.specularColor = Color3.Black();
    material.alphaMode = Constants.ALPHA_ADD;
    material.alpha = 0.999;
    material.disableDepthWrite = true;
    material.zOffset = -2;
    // Both sides (a shade glows inside too; the faces' winding varies with the file's handedness).
    material.backFaceCulling = false;
    material.fogEnabled = false;
    glowMaterials.set(scene, material);
  }
  const all = new VertexData();
  const positions: number[] = [];
  const normals: number[] = [];
  const colors: number[] = [];
  const indices: number[] = [];
  for (const { data, color } of pieces) {
    const base = positions.length / 3;
    const pos = data.positions ?? [];
    const c = Color3.FromHexString(color).toLinearSpace();
    positions.push(...pos);
    normals.push(...(data.normals ?? new Array<number>(pos.length).fill(0)));
    for (let i = 0; i < pos.length / 3; i++) colors.push(c.r, c.g, c.b, 1);
    for (const i of data.indices ?? Array.from({ length: pos.length / 3 }, (_, k) => k)) indices.push(base + i);
  }
  Object.assign(all, { positions, normals, colors, indices });
  const mesh = new Mesh(name, scene);
  all.applyToMesh(mesh);
  mesh.material = material;
  mesh.isPickable = false;
  mesh.setEnabled(false);
  return mesh;
}

/** The box (`lo`, `hi`: corners along x, y, z) of a mesh's positions. */
function dataBox(data: VertexData): { lo: number[]; hi: number[] } {
  const lo = [Infinity, Infinity, Infinity];
  const hi = [-Infinity, -Infinity, -Infinity];
  const pos = data.positions ?? [];
  for (let i = 0; i < pos.length; i += 3) {
    for (let k = 0; k < 3; k++) {
      lo[k] = Math.min(lo[k], pos[i + k]);
      hi[k] = Math.max(hi[k], pos[i + k]);
    }
  }
  return { lo, hi };
}

/** Scales and moves geometry so its joint bounds fill `size`, bottom on y = 0; the matrix it did that with. */
function fitInto(pieces: VertexData[], [w, h, d]: Vec3, align: 'centre' | 'back', [ox, oy, oz]: Vec3): Matrix {
  const min = new Vector3(Infinity, Infinity, Infinity);
  const max = new Vector3(-Infinity, -Infinity, -Infinity);
  for (const data of pieces) {
    const { lo, hi } = dataBox(data);
    min.minimizeInPlace(Vector3.FromArray(lo));
    max.maximizeInPlace(Vector3.FromArray(hi));
  }
  const ext = max.subtract(min);
  const sx = w / Math.max(ext.x, 1e-6);
  const sy = h / Math.max(ext.y, 1e-6);
  const sz = d / Math.max(ext.z, 1e-6);
  const cx = (min.x + max.x) / 2;
  const cz = align === 'back' ? min.z : (min.z + max.z) / 2;
  const tz = align === 'back' ? -0.5 + oz : oz;
  const m = Matrix.Scaling(sx, sy, sz).multiply(Matrix.Translation(ox - cx * sx, oy - min.y * sy, tz - cz * sz));
  for (const data of pieces) data.transform(m);
  return m;
}

/**
 * A copy of a file's geometry as a primitive part (positions, normals, metre UVs, vertex colour
 * carrying the finish), so it merges with primitive parts of the same finish. `full`, the same
 * geometry where a part grown from nothing is grown, gives the normals, winding and UVs its flat rest
 * shape can't (a textured finish's normal map draws nothing on UVs collapsed along an axis).
 */
function restyledMesh(scene: Scene, data: VertexData, material: Material | null, ov: MaterialOverride, alpha: number, full?: VertexData): Mesh {
  const shape = full ?? data;
  const positions = shape.positions!;
  const indices = data.indices ?? Array.from({ length: positions.length / 3 }, (_, i) => i);
  let normals = shape.normals;
  if (!normals) {
    normals = new Float32Array(positions.length);
    VertexData.ComputeNormals(positions, indices, normals);
  }
  // The source winding depends on the file's handedness and the baked transform; re-wind every
  // triangle to match its vertex normals the way the finish materials expect (see `pushTri`).
  const pos = Array.from(positions);
  const nor = Array.from(normals);
  const wound: number[] = [];
  for (let i = 0; i + 2 < indices.length; i += 3) pushTri(pos, nor, wound, indices[i], indices[i + 1], indices[i + 2]);
  const mesh = new Mesh('part', scene);
  const restyled = new VertexData();
  restyled.positions = Float32Array.from(positions);
  restyled.normals = Float32Array.from(normals);
  restyled.indices = Uint32Array.from(wound);
  restyled.uvs = new Float32Array((positions.length / 3) * 2);
  restyled.applyToMesh(mesh, true);
  boxProjectUVs(mesh);
  if (full) {
    mesh.updateVerticesData(VertexBuffer.PositionKind, Float32Array.from(data.positions!));
    // (Normals at rest that the instance's pose turns into the grown shape's: `restNormals`.)
    if (data.normals) mesh.updateVerticesData(VertexBuffer.NormalKind, Float32Array.from(data.normals));
  }
  const c = ov.color ? Color3.FromHexString(ov.color) : material instanceof PBRMaterial ? material.albedoColor.toGammaSpace() : Color3.White();
  const count = positions.length / 3;
  const colors = new Float32Array(count * 4);
  for (let i = 0; i < count; i++) colors.set([c.r, c.g, c.b, alpha], i * 4);
  mesh.setVerticesData(VertexBuffer.ColorKind, colors);
  mesh.refreshBoundingInfo();
  return mesh;
}

/** One mesh (draw call) per material: glTF files often split a model into several nodes. */
function mergeByMaterial(meshes: Mesh[], name: string): Mesh[] {
  const byMaterial = Map.groupBy(meshes, (m) => m.material);
  const out: Mesh[] = [];
  for (const group of byMaterial.values()) {
    if (group.length === 1) {
      out.push(group[0]);
      continue;
    }
    try {
      const merged = Mesh.MergeMeshes(group, true, true);
      if (merged) {
        merged.name = name;
        out.push(merged);
        continue;
      }
    } catch (err) {
      console.warn(`[render] could not merge parts of ${name}`, err);
    }
    out.push(...group.filter((m) => !m.isDisposed()));
  }
  return out;
}

/**
 * The natural variation of a placed plant (manifest `vary`): its own turn (radians) and size,
 * fixed by the tile it stands on so it looks the same after a reload. None when `vary` is 0.
 */
export function placementVariation(vary: number, x: number, z: number): { turn: number; size: number } {
  if (!vary) return { turn: 0, size: 1 };
  return { turn: hash3(x, z, 101) * Math.PI * 2, size: 1 + (hash3(x, z, 102) * 2 - 1) * vary };
}

// ---- small math helpers -------------------------------------------------------------

function clamp(v: number, lo: number, hi: number): number {
  return v < lo ? lo : v > hi ? hi : v;
}

function smoothstep(a: number, b: number, x: number): number {
  const t = clamp((x - a) / (b - a), 0, 1);
  return t * t * (3 - 2 * t);
}

function hash3(x: number, y: number, z: number): number {
  let h = (Math.imul(x, 374761393) + Math.imul(y, 668265263) + Math.imul(z, 1440662683)) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

/** Smooth value noise in [0, 1]. */
function noise3(x: number, y: number, z: number): number {
  const xi = Math.floor(x);
  const yi = Math.floor(y);
  const zi = Math.floor(z);
  const fx = x - xi;
  const fy = y - yi;
  const fz = z - zi;
  const u = fx * fx * (3 - 2 * fx);
  const v = fy * fy * (3 - 2 * fy);
  const w = fz * fz * (3 - 2 * fz);
  const l = (a: number, b: number, t: number) => a + (b - a) * t;
  const c = (dx: number, dy: number, dz: number) => hash3(xi + dx, yi + dy, zi + dz);
  return l(
    l(l(c(0, 0, 0), c(1, 0, 0), u), l(c(0, 1, 0), c(1, 1, 0), u), v),
    l(l(c(0, 0, 1), c(1, 0, 1), u), l(c(0, 1, 1), c(1, 1, 1), u), v),
    w,
  );
}
