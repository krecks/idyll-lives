/**
 * Builds renderable templates from asset entries: glTF files when provided,
 * otherwise placeholder primitives from the manifest. Each template is drawn
 * with thin instances (one draw call per mesh, however many copies exist).
 *
 * Placeholder parts are merged per (tint slot, finish) so a model costs one draw call per
 * distinct finish. Boxes get rounded edges so they catch highlights; every part gets
 * metre-scale UVs so textured finishes (fabric, wood, bark) tile at a realistic density.
 *
 * glTF entries may also carry (see `ModelExtras`):
 * - `fit`: scale the model (non-uniformly if needed) into an exact W x H x D box, bottom on the
 *   floor and centred on the footprint, so files from any source line up with the gameplay
 *   footprint and seat/bed heights.
 * - `materials`: restyle the file's materials by name: tint them, or replace them with one of the
 *   game's finishes (`material.finish.*`, vertex-coloured, box-projected UVs in metres). Replaced
 *   parts merge with placeholder parts of the same finish, so a flat-coloured kit model costs a few
 *   draw calls and matches the rest of the furniture. Style variants reuse one file this way.
 * - `parts`: extra placeholder primitives drawn with the file (books on a shelf, pillows, a lamp).
 */

import {
  Color3,
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
import type { AnimationGroup, Material, Node } from './core';
import { optimizedUrl } from '../../assets/registry';
import { foreground, prefetch } from '../../assets/stream';
import { paintSlots } from '../../assets/paint';
import type { ModelEntry, PlaceholderPart, Vec3 } from '../../assets/types';
import type { PartColors } from '../../core/protocol';
import { DEFAULT_FINISH, encodeFinish, MaterialLibrary } from './materials';

export type TintSlot = NonNullable<PlaceholderPart['tint']>;
export type BoneName = NonNullable<PlaceholderPart['bone']>;

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
}

export interface ModelClip {
  /** Seconds. */
  duration: number;
  /** Per part it moves: its pose in model space, sampled `CLIP_RATE` times a second (16 floats each). */
  tracks: { part: number; poses: Float32Array }[];
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
 * A mirror's reflecting face in model space: `matrix` takes the unit square (or, `round`, the
 * unit-diameter disc) in the xy plane, facing +z, onto the face, just in front of the glass.
 */
export interface MirrorSurface {
  round: boolean;
  matrix: Matrix;
}

export interface BuildOptions {
  /** 0 = full detail, 1 = reduced tessellation for distant instances. */
  lod?: 0 | 1;
  /** Things on a top: the radius (m) of the disc they take, for the stand-in when the model is missing. */
  radius?: number;
  /** Resized by the player: drawn at this size (see `resizeParts`). */
  resize?: Resize;
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
  if (entry?.url) {
    try {
      return await loadGltf(scene, name, entry, footprint, options.resize);
    } catch (err) {
      console.warn(`[render] failed to load ${entry.url}, using placeholder`, err);
    }
  }
  const parts = entry?.placeholder ?? fallbackParts(footprint, options.radius);
  const resized = options.resize ? resizeParts(parts, footprint, options.resize) : parts;
  // Built ahead (`buildAhead`): in stretches of idle time.
  const stretch = aheadStretch;
  if (stretch) return (stretch.build = buildPlaceholderAhead(scene, name, resized, options.lod ?? 0, stretch));
  return buildPlaceholder(scene, name, resized, options.lod ?? 0);
}

/** Share of a model's height above which everything rises with it (seats, tops, backs; sim-core `Top::of`). */
const RISE_FROM = 0.4;
/** Along the floor: the band at each edge (m, at most this share of half the width) that moves with the edge unstretched. */
const EDGE_BAND = 0.35;
const EDGE_SHARE = 0.4;
/** Parts smaller than this share of the width (or height) move without stretching. */
const SMALL = 0.15;

/**
 * Smart resizing for a model of `from` tiles drawn at `resize`: along the floor, the parts at
 * each edge (legs, side panels, frames) move out with it, keeping their thickness, while the
 * middle stretches (a table top, a sofa's cushions); upward, everything above `RISE_FROM` of its
 * height rises by the extra height and what spans that line (legs, a wardrobe's body) stretches
 * to meet it. Small parts only move. Parts turned off the model's axes only move, too.
 */
export function resizeParts(parts: readonly PlaceholderPart[], from: readonly [number, number], resize: Resize): PlaceholderPart[] {
  if (!resizes(from, resize)) return [...parts];
  // Each part's turn and axis-aligned box in model space.
  const boxes = parts.map(partBox);
  const bottom = Math.min(...boxes.map((b) => b.lo[1]));
  const top = Math.max(...boxes.map((b) => b.hi[1]));
  const { maps, span } = resizeMaps(from, resize, bottom, top);
  const widths = [from[0], span, from[1]];
  return parts.map((p, i) => {
    const { lo, hi, along } = boxes[i];
    const at = [...p.at] as Vec3;
    const size = [...p.size] as Vec3;
    for (let k = 0; k < 3; k++) {
      const map = maps[k];
      const length = hi[k] - lo[k];
      const j = along[k];
      if (j < 0 || length < SMALL * widths[k]) {
        // Moves as a whole: its middle goes where the map takes it.
        at[k] = map(p.at[k]);
        continue;
      }
      const [a, b] = [map(lo[k]), map(hi[k])];
      at[k] = (a + b) / 2;
      size[j] = Math.max(0.005, p.size[j] + (b - a) - length);
    }
    return { ...p, at, size };
  });
}

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
 * A file's geometry resized as `resizeParts` resizes parts, point by point: the edges move out
 * keeping their thickness, the middle stretches, and what lies above the rise line rises (a
 * triangle across it stretches). Normals follow the stretch.
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

function buildPlaceholder(scene: Scene, name: string, parts: PlaceholderPart[], lod: 0 | 1): ModelTemplate {
  const groups = new PartGroups(scene);
  parts.forEach((part, index) => groups.addPart(part, lod, index));
  return placeholderTemplate(parts, groups, groups.build(name));
}

/**
 * `buildPlaceholder` for a model built ahead: part by part, then group by group, while its stretch
 * of idle time lasts, and on in the next idle moment. The pieces stay hidden until it's whole.
 * Nothing (an empty model) if the scene is let go meanwhile.
 */
async function buildPlaceholderAhead(scene: Scene, name: string, parts: PlaceholderPart[], lod: 0 | 1, stretch: Stretch): Promise<ModelTemplate> {
  const groups = new PartGroups(scene);
  for (const [index, part] of parts.entries()) {
    if (stretch.spent && !(await stretch.next(scene))) return { meshes: [], tints: [], bones: [], height: 0 };
    groups.addPart(part, lod, index).setEnabled(false);
  }
  const built = await groups.buildAhead(name, stretch);
  return built ? placeholderTemplate(parts, groups, built) : { meshes: [], tints: [], bones: [], height: 0 };
}

function placeholderTemplate(parts: PlaceholderPart[], groups: PartGroups, { meshes, tints, bones }: BuiltGroups): ModelTemplate {
  const mirrors = parts.filter((p) => p.mirror).map(mirrorSurface);
  const boxes = parts.map(partBox);
  const outline = boxes.length ? { minX: Math.min(...boxes.map((b) => b.lo[0])), maxX: Math.max(...boxes.map((b) => b.hi[0])), minZ: Math.min(...boxes.map((b) => b.lo[2])), maxZ: Math.max(...boxes.map((b) => b.hi[2])) } : undefined;
  return { meshes, tints, bones, height: groups.height, bottom: groups.bottom, outline, ...(mirrors.length ? { mirrors } : {}) };
}

/**
 * A part's box in model space (`lo`, `hi`: corners along x, y, z), turned as `partMesh` turns it,
 * and which of its own axes lies along each model axis (-1: none; it's turned off the axes).
 */
function partBox(p: PlaceholderPart): { lo: number[]; hi: number[]; along: number[] } {
  const [rx, ry, rz] = (p.rotate ?? [0, 0, 0]).map((d) => (d * Math.PI) / 180);
  const m = Matrix.FromQuaternionToRef(Quaternion.FromEulerAngles(rx, ry, rz), new Matrix()).m;
  // Model axis k gathers |R[j][k]| * size[j] from each local axis j (row j: where local axis j goes).
  const half = [0, 1, 2].map((k) => [0, 1, 2].reduce((sum, j) => sum + Math.abs(m[j * 4 + k]) * p.size[j], 0) / 2);
  const along = [0, 1, 2].map((k) => [0, 1, 2].find((j) => Math.abs(m[j * 4 + k]) > 0.999) ?? -1);
  return { lo: p.at.map((c, k) => c - half[k]), hi: p.at.map((c, k) => c + half[k]), along };
}

/** The reflecting face of a mirror part (see `MirrorSurface`), placed as `partMesh` places the part. */
function mirrorSurface(p: PlaceholderPart): MirrorSurface {
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

  addPart(part: PlaceholderPart, lod: 0 | 1, index: number): Mesh {
    const { alpha, group } = this.finish(part.material ?? DEFAULT_FINISH);
    const mesh = partMesh(this.scene, part, lod, index, alpha);
    this.height = Math.max(this.height, part.at[1] + part.size[1] / 2);
    this.bottom = Math.min(this.bottom, part.at[1] - part.size[1] / 2);
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

  /** `build` in stretches of idle time (see `buildPlaceholderAhead`); the meshes show once all are merged. */
  async buildAhead(name: string, stretch: Stretch): Promise<BuiltGroups | null> {
    const built: BuiltGroups = { meshes: [], tints: [], bones: [] };
    for (const group of this.groups.values()) {
      if (stretch.spent && !(await stretch.next(this.scene))) return null;
      this.merge(name, group, built)?.setEnabled(false);
    }
    for (const mesh of built.meshes) mesh.setEnabled(true);
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

function partMesh(scene: Scene, p: PlaceholderPart, lod: 0 | 1, index: number, alpha: number): Mesh {
  const [sx, sy, sz] = p.size;
  const lo = lod === 1;
  let mesh: Mesh;
  let shade: ((nx: number, ny: number, nz: number) => number) | null = null;
  // `torus` and `lathe` extend the manifest schema (see `ExtraShapes`).
  const ext = p as PlaceholderPart & ExtraShapes;
  switch (p.shape as PlaceholderPart['shape'] | 'torus' | 'lathe') {
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

/** Placeholder shapes beyond the base schema. */
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

/** A neutral box when a model key is missing entirely (for something on a top, a small one its size). */
function fallbackParts([w, d]: [number, number], radius?: number): PlaceholderPart[] {
  if (radius) return [{ shape: 'cylinder', size: [radius * 1.8, radius * 1.6, radius * 1.8], at: [0, radius * 0.8, 0], color: '#B8B2AA' }];
  return [{ shape: 'box', size: [w * 0.8, 0.8, d * 0.8], at: [0, 0.4, 0], color: '#B8B2AA' }];
}

/** Optional model entry fields beyond the base schema (read defensively; see the file header). */
interface ModelExtras {
  /** Exact size [width x, height y, depth z] in metres after `rotationY`; replaces `scale`. */
  fit?: Vec3;
  /** With `fit`: `back` puts the back (-Z) of the model on the footprint's back edge instead of centring it. */
  align?: 'centre' | 'back';
  /** Restyles materials by glTF material name; `*` applies to every material not listed. */
  materials?: Record<string, MaterialOverride>;
  /** Extra placeholder primitives drawn with the model (in the final, fitted space). */
  parts?: PlaceholderPart[];
  /** When the file's clips play (`objectMotion.ts`). */
  motions?: MotionsEntry;
}

/** A use's clip in a manifest's `motions.uses`: its name, or the name and its own `hold`; or several of these. */
type UseEntry = string | { clip: string; hold?: number };

/** A model entry's `motions` as the manifest writes it: when its clips play (see `Motions`). */
interface MotionsEntry {
  uses?: Record<string, UseEntry | UseEntry[]>;
  hold?: number;
  open?: string;
  loops?: Record<string, string>;
  idle?: string;
  night?: string;
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
}

export interface UseClip {
  clip: string;
  hold?: number;
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
  for (const [name, color] of Object.entries(shown)) materials[name] = { ...(materials[name] ?? materials['*']), color };
  return { ...entry, materials } as ModelEntry;
}

/** A model entry's `motions`, if it has any. */
export function motionsOf(entry: ModelEntry | undefined): Motions | undefined {
  const m = (entry as (ModelEntry & ModelExtras) | undefined)?.motions;
  if (!m) return undefined;
  const uses = Object.fromEntries(
    Object.entries(m.uses ?? {}).map(([use, value]) => [use, (Array.isArray(value) ? value : [value]).map((u) => (typeof u === 'string' ? { clip: u, hold: m.hold } : { clip: u.clip, hold: u.hold ?? m.hold }))]),
  );
  const loops = m.loops ?? {};
  return Object.keys(uses).length || Object.keys(loops).length || m.open || m.idle || m.night ? { uses, loops, open: m.open, idle: m.idle, night: m.night } : undefined;
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
}

/** Downloads the files of these models in the background, so they're at hand when shown. */
export function prefetchModels(entries: Iterable<ModelEntry | undefined>): void {
  prefetch([...entries].flatMap((e) => (e?.url ? [optimizedUrl(e.url)] : [])));
}

/** Idle time (ms) left below which `buildAhead` waits for the next idle moment. */
const IDLE_SLICE_MS = 6;
/**
 * Longest stretch of work (ms) on a model built ahead before it waits for the next idle moment.
 * A whole tree or park table takes 10–15 ms on a fast laptop and three times that on a slow one.
 */
const AHEAD_STRETCH_MS = 4;
/** Longest wait (ms) for an idle moment once a model is under way, so it's never left half made for long. */
const AHEAD_WAIT_MS = 200;

/** The work on one model built ahead, in stretches of idle time. */
class Stretch {
  private until = performance.now() + AHEAD_STRETCH_MS;
  /** It waited for more idle time: `build` is done when the model is whole. */
  waited = false;
  build: Promise<unknown> | null = null;

  get spent(): boolean {
    return performance.now() >= this.until;
  }

  /** Waits for the next idle moment, for another stretch; false if the scene was let go meanwhile. */
  async next(scene: Scene): Promise<boolean> {
    this.waited = true;
    await new Promise<void>((resolve) => idle(() => resolve(), AHEAD_WAIT_MS));
    this.until = performance.now() + AHEAD_STRETCH_MS;
    return !scene.isDisposed;
  }
}

/** Set while `buildAhead` runs a job: the generated model it builds goes in stretches of idle time. */
let aheadStretch: Stretch | null = null;

/**
 * Runs `jobs` in idle moments, as many as fit in each, until they are done or `current()` turns
 * false: generated models built ahead without holding up a frame. A model that takes longer than
 * a stretch goes on in the next idle moments, and the next job waits for it.
 */
export function buildAhead(jobs: (() => void)[], current: () => boolean): void {
  const more = () => {
    if (jobs.length) idle(step);
  };
  const step = (deadline?: IdleDeadline) => {
    if (!current()) return;
    while (jobs.length) {
      const stretch = (aheadStretch = new Stretch());
      try {
        jobs.shift()!();
      } finally {
        aheadStretch = null;
      }
      if (stretch.waited) return void stretch.build?.then(more, more);
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
 * A model being built is wanted now: background downloads wait for it.
 */
function importModel(url: string, scene: Scene): ReturnType<typeof ImportMeshAsync> {
  // Clips stay still: they're read (`readClips`), not played.
  const options = { pluginOptions: { gltf: { animationStartMode: 0 } } } as Parameters<typeof ImportMeshAsync>[2];
  return foreground(async () => {
    const copy = optimizedUrl(url);
    if (copy === url) return ImportMeshAsync(url, scene, options);
    try {
      return await ImportMeshAsync(copy, scene, options);
    } catch (err) {
      console.warn(`[render] compressed ${copy} failed, loading ${url}`, err);
      return ImportMeshAsync(url, scene, options);
    }
  });
}

/**
 * A file as imported into a scene, once (`loadFile`): each of its meshes baked into model space at
 * rest, with its material. Every build of it (the lot's, the town's, buy mode's held copy, a
 * catalog picture, a resized copy) copies the geometry and shares the materials, so nothing of
 * the file is left behind when a build's meshes go.
 */
interface LoadedFile {
  pieces: FilePiece[];
  /** Its moving parts: the nodes its animations move, each with its pivot (the node's origin at rest, in model space). */
  parts: { name: string; pivot: Vector3 }[];
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
}

/** Files imported per scene, by URL (they go with the scene). */
const loadedFiles = new WeakMap<Scene, Map<string, Promise<LoadedFile>>>();

function loadFile(scene: Scene, url: string): Promise<LoadedFile> {
  const files = loadedFiles.get(scene) ?? new Map<string, Promise<LoadedFile>>();
  loadedFiles.set(scene, files);
  let file = files.get(url);
  if (!file) {
    const loading = importFile(scene, url);
    files.set(url, (file = loading));
    // One that failed is tried again by the next build.
    loading.catch(() => files.get(url) === loading && files.delete(url));
  }
  return file;
}

async function importFile(scene: Scene, url: string): Promise<LoadedFile> {
  const result = await importModel(url, scene);
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
  const pieces: FilePiece[] = [];
  for (const node of result.meshes) {
    if (!(node instanceof Mesh) || node.getTotalVertices() === 0) continue;
    // Bake the full hierarchy transform so thin-instance matrices place the model directly.
    pieces.push({ data: VertexData.ExtractFromMesh(node, true, true).transform(computeFromRoot(node)), material: node.material, part: partOf(node) });
  }
  const parts = moved.map((node, i) => ({ name: node.name, pivot: rest[i].getTranslation() }));
  const clips = new Map(result.animationGroups.map((group) => [group.name, readClip(group, moved, rest)]));
  result.animationGroups.forEach(atRest);
  // The nodes and clips go; the materials stay with the file.
  for (const node of [...result.transformNodes, ...result.meshes]) node.dispose(true, false);
  for (const group of result.animationGroups) group.dispose();
  return { pieces, parts, clips };
}

/** A clip's poses of the parts it moves (`ModelClip`): each node's world matrix then, relative to its world matrix at rest. */
function readClip(group: AnimationGroup, moved: TransformNode[], rest: Matrix[]): ModelClip {
  const animations = group.targetedAnimations;
  const fps = animations[0]?.animation.framePerSecond || 60;
  const duration = Math.max(0, (group.to - group.from) / fps);
  const samples = Math.max(2, Math.ceil(duration * CLIP_RATE) + 1);
  const targets = [...new Set(animations.map((a) => a.target))].filter((t): t is TransformNode => t instanceof TransformNode);
  const tracks = targets.map((node) => ({ part: moved.indexOf(node), poses: new Float32Array(samples * 16) }));
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

async function loadGltf(scene: Scene, name: string, entry: ModelEntry, footprint: [number, number], resize?: Resize): Promise<ModelTemplate> {
  const extras = entry as ModelEntry & ModelExtras;
  const file = await loadFile(scene, entry.url!);
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
  const overrideFor = (m: Material | null): MaterialOverride | undefined => (m && overrides[m.name]) ?? overrides['*'];

  // This build's copy of the geometry it shows.
  const pieces = file.pieces.filter((p) => !overrideFor(p.material)?.hide).map((p) => ({ data: copyData(p.data).transform(space), material: p.material, part: p.part }));
  if (fit && pieces.length) space = space.multiply(fitInto(pieces.map((p) => p.data), fit, extras.align ?? 'centre', [ox, oy, oz]));
  const boxes = pieces.map((p) => dataBox(p.data));
  // Resized by the player as placeholders are; each moving part's pivot goes where its hinge went.
  const pivots = file.parts.map((p) => Vector3.TransformCoordinates(p.pivot, space));
  const shifts = pivots.map(() => Vector3.Zero());
  if (resize && boxes.length && resizes(footprint, resize)) {
    const { maps, slopes } = resizeMaps(footprint, resize, Math.min(...boxes.map((b) => b.lo[1])), Math.max(...boxes.map((b) => b.hi[1])));
    for (const p of pieces) resizeData(p.data, maps, slopes);
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
    pieces.forEach(({ data, material, part: of }, i) => {
      if (of !== part) return;
      height = Math.max(height, boxes[i].hi[1]);
      bottom = Math.min(bottom, boxes[i].lo[1]);
      const ov = overrideFor(material);
      if (ov?.finish) {
        groups.add(restyledMesh(scene, data, material, ov, groups.finish(ov.finish).alpha), null, groups.finish(ov.finish).group, null);
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
    meshes.push(...merged, ...restyled.meshes);
    tints.push(...merged.map(() => null), ...restyled.tints);
    parts.push(...[...merged, ...restyled.meshes].map(() => part));
  }

  const lo = (k: number) => Math.min(...boxes.map((b) => b.lo[k]));
  const hi = (k: number) => Math.max(...boxes.map((b) => b.hi[k]));
  const outline = boxes.length ? { minX: lo(0), maxX: hi(0), minZ: lo(2), maxZ: hi(2) } : undefined;
  return {
    meshes,
    tints,
    bones: meshes.map(() => null),
    height,
    bottom,
    outline,
    ...(file.parts.length && file.clips.size ? { motion: { parts, clips: buildClips(file.clips, space, shifts) } } : {}),
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
        tracks: clip.tracks.map(({ part, poses }) => {
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
          return { part, poses: built };
        }),
      },
    ]),
  );
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
 * A copy of a file's geometry in placeholder form (positions, normals, metre UVs, vertex colour
 * carrying the finish), so it merges with placeholder parts of the same finish.
 */
function restyledMesh(scene: Scene, data: VertexData, material: Material | null, ov: MaterialOverride, alpha: number): Mesh {
  const positions = data.positions!;
  const indices = data.indices ?? Array.from({ length: positions.length / 3 }, (_, i) => i);
  let normals = data.normals;
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
