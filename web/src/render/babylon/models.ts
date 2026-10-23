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
import { foreground } from '../../assets/stream';
import type { ModelEntry, PrimitivePart, Vec3 } from '../../assets/types';
import { DEFAULT_FINISH, encodeFinish, MaterialLibrary } from './materials';
import { WALL_HALF, WALL_HEIGHT } from '../shared/walls';
import { CLIP_RATE, type ModelClip } from '../shared/modelClips';
import { entrySpace as entrySpaceArray, namedFinish, tintInName, type BoneName, type ExtraShapes, type MaterialOverride, type ModelExtras, type Resize, type StandIn, type TintSlot } from '../shared/modelEntry';
import { ModelFiles } from '../shared/modelFiles';
import { partBox, resizeData, resizeMaps, resizes } from '../shared/modelParts';

export type { BoneName, TintSlot } from '../shared/modelEntry';

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
  /**
   * Named nodes (each with all below it) made moving parts of their own, though the file has no clips
   * to move them (the birds' head, tail and wings, posed in code): the template's `motion.parts` says
   * which part each mesh is (index into `parts`, -1: a node under none of them) and `motion.rests`
   * where each node is at rest (a missing one at the origin); the file's own clips are left out.
   */
  parts?: readonly string[];
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
        const template = buildGltf(scene, name, entry, options.parts ? splitParts(file, options.parts) : file, footprint, options.resize);
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

/** A file with its named nodes `names` (each with all below it) as its moving parts, and no clips (`BuildOptions.parts`). */
function splitParts(file: LoadedFile, names: readonly string[]): LoadedFile {
  const partOf = (nodes: readonly string[] = []) => {
    for (const n of nodes) {
      const i = names.indexOf(n);
      if (i >= 0) return i;
    }
    return -1;
  };
  const rests = names.map((n) => file.named?.get(n) ?? Matrix.Identity());
  return {
    pieces: file.pieces.map((p) => ({ ...p, part: partOf(p.nodes), full: undefined })),
    parts: names.map((n, i) => ({ name: n, pivot: rests[i].getTranslation(), rest: rests[i] })),
    clips: new Map(),
  };
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

/** The reflecting face of a mirror part (see `MirrorSurface`), placed as `partMesh` places the part. (Exported for `tests/modelParts.test.ts`.) */
export function mirrorSurface(p: PrimitivePart): MirrorSurface {
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

/** A primitive part's mesh in model space (exported for `tests/modelParts.test.ts`, against `shared/modelParts.ts` `partData`). */
export function partMesh(scene: Scene, p: PrimitivePart, lod: 0 | 1, index: number, alpha: number): Mesh {
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

/**
 * The colour (sRGB hex) `entry`'s file gives its material `name`, as made (before the player paints
 * it), from the file as `scene` imported it (shared with its builds); undefined without one.
 */
export async function fileColour(scene: Scene, entry: ModelEntry | undefined, name: string): Promise<string | undefined> {
  if (!entry?.url) return undefined;
  const file = await loadFile(scene, entry.url);
  const material = file.pieces.find((p) => p.material?.name === name)?.material;
  return material instanceof PBRMaterial ? material.albedoColor.toGammaSpace().toHexString() : undefined;
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
  /** Every named node's place at rest in model space (`BuildOptions.parts` makes moving parts of some by name). */
  named?: Map<string, Matrix>;
}

interface FilePiece {
  /** In the file's model space (its nodes' transforms baked in); builds copy it, never change it. */
  data: VertexData;
  /** The file's own material, kept as loaded: builds draw with copies of it (`fileMaterial`). */
  material: Material | null;
  /** The moving part it belongs to, or -1. */
  part: number;
  /** The names of its node and the nodes above it, nearest first (`BuildOptions.parts`). */
  nodes?: string[];
  /** Where its part is grown, for a part scaled to nothing at rest: the shape its normals and UVs come from. */
  full?: VertexData;
}

/** A scene's model files (`shared/modelFiles.ts`): they go with the scene. */
const sceneFiles = new WeakMap<Scene, ModelFiles<LoadedFile>>();

function filesOf(scene: Scene): ModelFiles<LoadedFile> {
  const known = sceneFiles.get(scene);
  if (known) return known;
  const files = new ModelFiles<LoadedFile>(
    (url, now) => importFile(scene, url, now),
    () => scene.isDisposed,
  );
  sceneFiles.set(scene, files);
  scene.onDisposeObservable.addOnce(() => files.dispose());
  return files;
}

/**
 * Calls `fn` with a file's URL when it loads in `scene` after failing to (in a background try, or
 * for any build): whatever was built without it (a box, or nothing) can be built again. Returns
 * what stops it. A scene nobody listens to doesn't try its failed files in the background.
 */
export function onModelFileArrived(scene: Scene, fn: (url: string) => void): () => void {
  return filesOf(scene).onArrived(fn);
}

/**
 * Tells `scene`'s listeners (`onModelFileArrived`) again that the file at `url` has come: a template
 * still being built when it came turned out to be built without it (`TemplateCache`).
 */
export function retellModelFileArrived(scene: Scene, url: string): void {
  filesOf(scene).retell(url);
}

/** The file at `url` as imported into `scene`, once (`ModelFiles.load`). */
function loadFile(scene: Scene, url: string): Promise<LoadedFile> {
  return filesOf(scene).load(url);
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
    const nodes: string[] = [];
    for (let n: Node | null = node; n; n = n.parent) if (n.name) nodes.push(n.name);
    const piece: FilePiece = { data: VertexData.ExtractFromMesh(node, true, true).transform(computeFromRoot(node)), material: node.material, part, nodes };
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
  const named = new Map<string, Matrix>();
  for (const node of [...result.transformNodes, ...result.meshes]) if (node.name && !named.has(node.name)) named.set(node.name, computeFromRoot(node).clone());
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
  return { pieces, parts, clips, named };
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

/** The tint slot a file's material carries in its name (`tintInName`), or null. */
function namedTint(material: Material | null): TintSlot | null {
  return tintInName(material?.name);
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

/** A model entry's own placement of its file's model space (`shared/modelEntry.ts` `entrySpace`). */
export function entrySpace(entry: ModelEntry): Matrix {
  return Matrix.FromArray(entrySpaceArray(entry));
}

/** `buildGltf`'s build. */
function buildFromFile(scene: Scene, name: string, entry: ModelEntry, file: LoadedFile, footprint: [number, number], resize?: Resize): ModelTemplate {
  const extras = entry as ModelEntry & ModelExtras;
  const fit = extras.fit;
  const [ox, oy, oz] = entry.offset ?? [0, 0, 0];
  // The file's model space to this build's: `scale`, `rotationY` and `offset`, or `fit`.
  let space = entrySpace(entry);
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
    for (const p of pieces) for (const data of p.full ? [p.data, p.full] : [p.data]) resizeData({ positions: data.positions!, normals: data.normals }, maps, slopes);
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
    // (A file's parts are what its clips move; `BuildOptions.parts` names its own, without clips.)
    ...(file.parts.length
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
/** How far (depth-buffer steps) a glow is drawn in front of the faces it lights. */
const GLOW_DEPTH_STEPS = -4;

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
    // Its faces are the lit ones again, so it only has to win their own depth: a few steps of the
    // depth buffer, not a slope-scaled offset, which at a glancing angle (or from far off) pulled a
    // glow through the parts in front of it (the grill's coals through its shut lid).
    material.zOffset = 0;
    material.zOffsetUnits = GLOW_DEPTH_STEPS;
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
  // The file's grey vertex colour is the shading baked into its creases (gamekit/shade.py), linear: the
  // colour here is sRGB (the finish shader raises it to 2.2), so it takes the shading's 1/2.2 power.
  const baked = data.colors && data.colors.length >= count * 4 ? data.colors : null;
  for (let i = 0; i < count; i++) {
    const k = baked ? Math.max(0, baked[i * 4]) ** (1 / 2.2) : 1;
    colors.set([c.r * k, c.g * k, c.b * k, alpha], i * 4);
  }
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
