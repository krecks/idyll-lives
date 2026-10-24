// `evaluate`, `slerp` and `decompose` are ported from Babylon.js 9.30 (`Animation.evaluate`,
// `Quaternion.Slerp`, `Matrix.decompose`). Changed for Idyll Lives: on the clips' own tracks and
// plain arrays.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
/**
 * Builds renderable templates from asset entries on PlayCanvas: glTF files, or nothing when there's
 * no file or it doesn't load (a plain box for things placed, held or pictured, `BuildOptions.standIn`). A file that fails is tried again in the background,
 * and whoever drew the box builds it again once it comes (`onModelFileArrived`, `TemplateCache`).
 * A template is data (meshes with their materials, in model space): whoever draws it gives each
 * mesh its instances (`instancing.ts`), one draw call per mesh however many copies exist.
 *
 * Primitive parts (a character's limbs, the few parts a file can't carry) are merged per
 * (tint slot, finish) so a model costs one draw call per distinct finish (`shared/modelParts.ts`
 * builds their vertices). glTF entries may also carry `fit`, `materials` (restyled with the game's
 * finishes, mirror glass, glows) and `parts`: see `shared/modelEntry.ts` `ModelExtras`.
 *
 * Game space. A file loads through PlayCanvas's glTF container (meshopt and KTX2 in the production
 * build: `loaders.ts`), but its geometry, nodes and animations are read here from the file's own
 * buffers and baked into model space, the game's left-handed space: the file's x turned round (a
 * `__root__` node above the file's scene, scaled -1 in z and turned half round, `ROOT`). So a
 * model's left and right are the ones the game's own numbers (`fit`, `offset`, parts, clips' poses)
 * were made for. Its clips are sampled here too (`readClip`): the game plays them itself
 * (`shared/modelClips.ts`), not PlayCanvas's anim system.
 *
 * Winding: every mesh keeps the game's winding in game space (its builders' front faces: `pushTri`),
 * and the materials take that for the front under the world root's mirror (`GAME_FRONT_FACE`). A
 * file's triangles go the same way: its x turned round and its winding swapped, so they keep the
 * file's front faces, drawn with copies of its materials that take the game's front face too. Its
 * tangents keep their handedness as the file has it: PlayCanvas builds the bitangent in world
 * space, past the mirror, where the file's frame is only turned.
 */

import * as pc from './engine';
import { containerOptions, setupDecoders } from './loaders';
import { DEFAULT_FINISH, encodeFinish, GAME_FRONT_FACE, INSTANCE_COLOR_SEMANTIC, MaterialLibrary } from './materials';
import { installGameChunks } from './shaders/chunks';
import { optimizedUrl } from '../../assets/registry';
import { foreground } from '../../assets/stream';
import type { ModelEntry, PrimitivePart, Vec3 } from '../../assets/types';
import { compose, invert, multiply, transformPoint, translation, type Mat4, type Quat } from '../shared/math';
import { CLIP_RATE, type ModelClip } from '../shared/modelClips';
import { entrySpace, namedFinish, tintInName, type BoneName, type MaterialOverride, type ModelExtras, type Resize, type StandIn, type TintSlot } from '../shared/modelEntry';
import { ModelFiles } from '../shared/modelFiles';
import { deviceDestroyed } from './device';
import {
  computeNormals,
  copyData,
  dataBox,
  determinant,
  fileMirror,
  fitInto,
  flipFaces,
  glowData,
  hexColor,
  mergeData,
  mirrorSurface,
  partBox,
  partData,
  resizeData,
  resizeMaps,
  resizes,
  restNormals,
  restyledData,
  transformData,
  type MirrorFace,
  type VData,
} from '../shared/modelParts';
import { WALL_HALF, WALL_HEIGHT } from '../shared/walls';

export type { BoneName, TintSlot } from '../shared/modelEntry';
export { entrySpace } from '../shared/modelEntry';

/** A mesh of a template: its geometry (model space) and what it's drawn with. */
export interface ModelMesh {
  name: string;
  mesh: pc.Mesh;
  material: pc.Material;
  /** A part grown from nothing lies flat where it hides at rest: leaf flutter would lift it into view (nature). */
  windStill?: boolean;
}

export interface ModelTemplate {
  /** Meshes in model space; each receives the same instance matrices. */
  meshes: ModelMesh[];
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
   * gives each instance its strength (the instance colour, `INSTANCE_COLOR`) and draws it. `screen`
   * marks the ones lit by use rather than by the room. Not in `meshes`: nothing else draws them.
   */
  glows?: { mesh: ModelMesh; part: number; screen?: boolean }[];
  /**
   * The files it was built without, as they failed to load: its far version (it's the full model
   * then), or every one (it's drawn as nothing, or the box asked for). It's built again when one of
   * them comes: by a cache (`TemplateCache.invalidate`), the landscape, the selection marker.
   */
  failed?: string[];
  /** It's the stand-in box (`BuildOptions.standIn`): its file is missing, or failed to load or to build. */
  boxed?: boolean;
}

/** A template's meshes and its glows' (all it draws). */
export function withGlows(t: ModelTemplate): ModelMesh[] {
  return [...t.meshes, ...(t.glows ?? []).map((g) => g.mesh)];
}

/** Lets go of a template's meshes (its materials are shared and stay). */
export function disposeTemplate(t: { meshes: ModelMesh[]; glows?: { mesh: ModelMesh }[] }): void {
  for (const m of [...t.meshes, ...(t.glows ?? []).map((g) => g.mesh)]) if (!deviceDestroyed(m.mesh.device)) m.mesh.destroy();
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
   * instrument) is placed by its own frame.
   */
  names?: string[];
  rests?: Float32Array[];
}

/** A mirror's reflecting face (`shared/modelParts.ts` `MirrorFace`), and the moving part it's on (a mirror cabinet's door; absent or -1: the still body). */
export interface MirrorSurface extends MirrorFace {
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
   * can be clicked: placed objects, buy mode's copy of one in hand, food and dishes, what residents
   * carry, and the catalog's pictures. The landscape, foliage, kerbside props and the selection marker
   * draw nothing then, until the file comes (`onModelFileArrived`).
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

export async function buildModel(app: pc.AppBase, name: string, entry: ModelEntry | undefined, footprint: [number, number], options: BuildOptions = {}): Promise<ModelTemplate> {
  const failed: string[] = [];
  if (entry?.url) {
    // Far off, the far version (without moving parts); the full one if that fails.
    const far = (options.far || options.lod === 1) && entry.far ? entry.far : undefined;
    const instead = (url: string) => (url === far ? 'the full model' : options.standIn ? 'a box' : 'nothing');
    for (const url of far ? [far, entry.url] : [entry.url]) {
      let file: LoadedFile;
      try {
        file = await filesOf(app).load(url);
      } catch (err) {
        // Only a file that didn't load is `failed` (and comes later, `onModelFileArrived`).
        failed.push(url);
        // (Once per file: each build of it tries it again, and a page of things may share one.)
        if (!warnedFiles.has(url)) console.warn(`[render] failed to load ${url}, using ${instead(url)}`, err);
        warnedFiles.add(url);
        continue;
      }
      try {
        const template = buildGltf(app, name, entry, options.parts ? splitParts(file, options.parts) : file, footprint, options.resize);
        return failed.length ? { ...template, failed } : template;
      } catch (err) {
        // One that loads but can't be built won't build when tried again either: not `failed`.
        if (!warnedFiles.has(url)) console.warn(`[render] failed to build ${name} from ${url}, using ${instead(url)}`, err);
        warnedFiles.add(url);
      }
    }
  }
  // No file (the registry said so, once per key), it failed to load or to build: nothing, or the box
  // asked for. A cache keeps either; one whose file failed to load is built again when it comes.
  const template: ModelTemplate = options.standIn ? { ...standIn(app, name, options.resize?.size ?? footprint, options.standIn, options.resize), boxed: true } : { meshes: [], tints: [], bones: [], height: 0 };
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
  const rests = names.map((n) => file.named.get(n) ?? identityMatrix());
  return {
    pieces: file.pieces.map((p) => ({ ...p, part: partOf(p.nodes), full: undefined })),
    parts: names.map((n, i) => ({ name: n, pivot: [rests[i][12], rests[i][13], rests[i][14]], rest: rests[i] })),
    clips: new Map(),
    named: file.named,
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
function standIn(app: pc.AppBase, name: string, [w, d]: [number, number], { layer, radius }: StandIn, resize?: Resize): ModelTemplate {
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
  const groups = new PartGroups(app);
  groups.addPart(part, 0, 0);
  const { meshes, tints, bones } = groups.build(name);
  const { lo, hi } = partBox(part);
  return { meshes, tints, bones, height: groups.height, bottom: groups.bottom, outline: { minX: lo[0], maxX: hi[0], minZ: lo[2], maxZ: hi[2] } };
}

/** Group key for parts drawn with the shared plain finish. */
const PLAIN = 'plain';

/**
 * Meshes grouped per (tint slot, finish, bone) and merged into one draw call each. Untextured
 * opaque finishes share one material: roughness/metalness go into the vertex alpha.
 */
class PartGroups {
  private readonly lib: MaterialLibrary;
  private readonly groups = new Map<string, { tint: TintSlot | null; finish: string; bone: BoneName | null; data: VData[] }>();
  height = 0;
  bottom = Infinity;

  constructor(private readonly app: pc.AppBase) {
    this.lib = MaterialLibrary.for(app);
  }

  /** Vertex alpha and group finish for a finish key. */
  finish(key: string): { alpha: number; group: string } {
    const info = this.lib.finishInfo(key);
    return info.plain ? { alpha: encodeFinish(info.roughness, info.metallic), group: PLAIN } : { alpha: 1, group: key };
  }

  addPart(part: PrimitivePart, lod: 0 | 1, index: number): void {
    const { alpha, group } = this.finish(part.material ?? DEFAULT_FINISH);
    const box = partBox(part);
    this.height = Math.max(this.height, box.hi[1]);
    this.bottom = Math.min(this.bottom, box.lo[1]);
    this.add(partData(part, lod, index, alpha), part.tint ?? null, group, part.bone ?? null);
  }

  add(data: VData, tint: TintSlot | null, finish: string, bone: BoneName | null): void {
    const key = `${tint ?? ''}|${finish}|${bone ?? ''}`;
    const group = this.groups.get(key) ?? { tint, finish, bone, data: [] };
    group.data.push(data);
    this.groups.set(key, group);
  }

  /** One mesh per group: merged, uploaded, with its finish. */
  build(name: string): { meshes: ModelMesh[]; tints: (TintSlot | null)[]; bones: (BoneName | null)[] } {
    const built: { meshes: ModelMesh[]; tints: (TintSlot | null)[]; bones: (BoneName | null)[] } = { meshes: [], tints: [], bones: [] };
    for (const { tint, finish, bone, data } of this.groups.values()) {
      const suffix = finish === PLAIN ? '' : `@${finish.replace('material.finish.', '')}`;
      built.meshes.push({
        name: `${name}${tint ? `:${tint}` : ''}${bone ? `#${bone}` : ''}${suffix}`,
        mesh: uploadData(this.app.graphicsDevice, mergeData(data)),
        material: finish === PLAIN ? this.lib.plainFinish() : this.lib.finish(finish),
      });
      built.tints.push(tint);
      built.bones.push(bone);
    }
    return built;
  }
}

/**
 * A PlayCanvas mesh of `data` (game space, wound as the game's builders wind front faces: `pushTri`;
 * drawn with materials whose front face is `GAME_FRONT_FACE`).
 */
export function uploadData(device: pc.GraphicsDevice, data: VData): pc.Mesh {
  const count = data.positions.length / 3;
  const indices = count > 65535 ? new Uint32Array(data.indices) : new Uint16Array(data.indices);
  const mesh = new pc.Mesh(device);
  mesh.setPositions(data.positions);
  if (data.normals) mesh.setNormals(data.normals);
  if (data.uvs) mesh.setUvs(0, data.uvs);
  if (data.colors) mesh.setColors(data.colors);
  if (data.tangents) mesh.setVertexStream(pc.SEMANTIC_TANGENT, data.tangents, 4);
  mesh.setIndices(indices);
  mesh.update(pc.PRIMITIVE_TRIANGLES);
  return mesh;
}

// ---- files --------------------------------------------------------------------------

/**
 * A file as read for an app, once (`filesOf`): each of its meshes baked into model space at rest,
 * with its material. Every build of it (the lot's, the town's, buy mode's held copy, a catalog
 * picture, a resized copy) copies the geometry and shares the materials.
 */
interface LoadedFile {
  pieces: FilePiece[];
  /** Its moving parts: the nodes its animations move, each with its pivot (the node's origin at rest, in model space) and its whole place then. */
  parts: { name: string; pivot: [number, number, number]; rest: Float32Array }[];
  /** Its animations, by name, in the file's model space. */
  clips: Map<string, ModelClip>;
  /** Every named node's place at rest in model space (`BuildOptions.parts` makes moving parts of some by name). */
  named: Map<string, Float32Array>;
}

interface FilePiece {
  /** In the file's model space (its nodes' transforms baked in); builds copy it, never change it. */
  data: VData;
  /** The file's own material, kept as loaded (builds draw with copies of it, `fileMaterial`), and its name in the file. */
  material: pc.StandardMaterial | null;
  materialName: string;
  /** The moving part it belongs to, or -1. */
  part: number;
  /** The names of its node and the nodes above it, nearest first (`BuildOptions.parts`). */
  nodes?: string[];
  /** Where its part is grown, for a part scaled to nothing at rest: the shape its normals and UVs come from. */
  full?: VData;
}

/** An app's model files (`shared/modelFiles.ts`): they go with the app. */
const appFiles = new WeakMap<pc.AppBase, ModelFiles<LoadedFile>>();

function filesOf(app: pc.AppBase): ModelFiles<LoadedFile> {
  const known = appFiles.get(app);
  if (known) return known;
  let gone = false;
  const files = new ModelFiles<LoadedFile>(
    (url, now) => importFile(app, url, now),
    () => gone,
  );
  appFiles.set(app, files);
  app.once('destroy', () => {
    gone = true;
    files.dispose();
  });
  return files;
}

/**
 * Calls `fn` with a file's URL when it loads for `app` after failing to (in a background try, or
 * for any build): whatever was built without it (a box, or nothing) can be built again. Returns
 * what stops it. An app nobody listens to doesn't try its failed files in the background.
 */
export function onModelFileArrived(app: pc.AppBase, fn: (url: string) => void): () => void {
  return filesOf(app).onArrived(fn);
}

/**
 * Tells `app`'s listeners (`onModelFileArrived`) again that the file at `url` has come: a template
 * still being built when it came turned out to be built without it (`TemplateCache`).
 */
export function retellModelFileArrived(app: pc.AppBase, url: string): void {
  filesOf(app).retell(url);
}

/**
 * The colour (sRGB hex) `entry`'s file gives its material `name`, as made (before the player paints
 * it), from the file as read for `app` (shared with its builds); undefined without one.
 */
export async function fileColour(app: pc.AppBase, entry: ModelEntry | undefined, name: string): Promise<string | undefined> {
  if (!entry?.url) return undefined;
  const file = await filesOf(app).load(entry.url);
  const material = file.pieces.find((p) => p.materialName === name)?.material;
  return material ? hexString(material.diffuse) : undefined;
}

/** A colour as `#RRGGBB`, upper case, each channel rounded to 0..255. */
function hexString(c: pc.Color): string {
  const hex = (v: number) => Math.max(0, Math.min(255, Math.round(v * 255))).toString(16).toUpperCase().padStart(2, '0');
  return `#${hex(c.r)}${hex(c.g)}${hex(c.b)}`;
}

/** What PlayCanvas's container gives of a glTF file: the JSON, its buffer views decoded, its materials. */
interface Container {
  gltf: Gltf;
  views: (Uint8Array | undefined)[];
  materials: (pc.StandardMaterial | null)[];
}

/**
 * The file at `url` through PlayCanvas's glTF container (`containerOptions`: meshopt, KTX2), its
 * buffer views kept as they're decoded. PlayCanvas's own meshes of it are let go of at once: the
 * geometry is read from the views (`importFile`); its materials stay.
 */
function loadContainer(app: pc.AppBase, url: string): Promise<Container> {
  setupDecoders();
  const decoded = new Map<object, Uint8Array>();
  const options = {
    ...containerOptions,
    bufferView: { ...containerOptions.bufferView, postprocess: (view: object, data: Uint8Array) => decoded.set(view, data) },
  };
  return new Promise((resolve, reject) => {
    const asset = new pc.Asset(url, 'container', { url }, undefined, options as never);
    asset.once('load', () => {
      const resource = asset.resource as pc.ContainerResource & { data: { gltf: Gltf; materials: pc.StandardMaterial[] } };
      const gltf = resource.data.gltf;
      const container = { gltf, views: (gltf.bufferViews ?? []).map((v) => decoded.get(v)), materials: resource.data.materials.map((m) => m ?? null) };
      // The meshes go (their GPU buffers with them); the materials (and their textures) stay with the file.
      for (const render of resource.renders) {
        render.unload();
        app.assets.remove(render);
      }
      for (const sub of [...resource.materials, ...resource.animations]) app.assets.remove(sub);
      app.assets.remove(asset);
      resolve(container);
    });
    asset.once('error', (err: unknown) => {
      app.assets.remove(asset);
      reject(new Error(`${url}: ${String(err)}`));
    });
    app.assets.add(asset);
    app.assets.load(asset);
  });
}

/**
 * The build's compressed copy of the file if there is one; the file itself if that fails to load.
 * A model being built is wanted `now`: background downloads wait for it.
 */
function importModel(app: pc.AppBase, url: string, now: boolean): Promise<Container> {
  const load = async () => {
    const copy = optimizedUrl(url);
    if (copy === url) return loadContainer(app, url);
    try {
      return await loadContainer(app, copy);
    } catch (err) {
      console.warn(`[render] compressed ${copy} failed, loading ${url}`, err);
      return loadContainer(app, url);
    }
  };
  return now ? foreground(load) : load();
}

// The parts of glTF read here.
interface Gltf {
  scene?: number;
  scenes?: { nodes?: number[] }[];
  nodes?: GltfNode[];
  meshes?: { primitives: GltfPrimitive[] }[];
  accessors?: GltfAccessor[];
  bufferViews?: { byteStride?: number }[];
  materials?: { name?: string }[];
  animations?: { name?: string; channels: { sampler: number; target: { node?: number; path: string } }[]; samplers: { input: number; output: number; interpolation?: string }[] }[];
}
interface GltfNode {
  name?: string;
  children?: number[];
  mesh?: number;
  matrix?: number[];
  translation?: number[];
  rotation?: number[];
  scale?: number[];
}
interface GltfPrimitive {
  attributes: Record<string, number>;
  indices?: number;
  material?: number;
  mode?: number;
}
interface GltfAccessor {
  bufferView?: number;
  byteOffset?: number;
  componentType: number;
  normalized?: boolean;
  count: number;
  type: string;
  sparse?: unknown;
}

const COMPONENTS: Record<string, number> = { SCALAR: 1, VEC2: 2, VEC3: 3, VEC4: 4, MAT2: 4, MAT3: 9, MAT4: 16 };
const SIZES: Record<number, number> = { 5120: 1, 5121: 1, 5122: 2, 5123: 2, 5125: 4, 5126: 4 };

/** An accessor's values as floats (normalised integers to 0..1 or -1..1), with their count per element. */
function readAccessor(file: Container, index: number): { values: Float32Array; components: number } {
  const a = file.gltf.accessors![index];
  const components = COMPONENTS[a.type];
  const size = SIZES[a.componentType];
  if (!components || !size) throw new Error(`accessor ${index}: unknown type ${a.type}/${a.componentType}`);
  if (a.sparse) throw new Error(`accessor ${index}: sparse accessors aren't read`);
  const values = new Float32Array(a.count * components);
  if (a.bufferView === undefined) return { values, components };
  const bytes = file.views[a.bufferView];
  if (!bytes) throw new Error(`accessor ${index}: buffer view ${a.bufferView} wasn't loaded`);
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const stride = file.gltf.bufferViews?.[a.bufferView]?.byteStride ?? components * size;
  const read: (at: number) => number = {
    5120: (at: number) => (a.normalized ? Math.max(view.getInt8(at) / 127, -1) : view.getInt8(at)),
    5121: (at: number) => (a.normalized ? view.getUint8(at) / 255 : view.getUint8(at)),
    5122: (at: number) => (a.normalized ? Math.max(view.getInt16(at, true) / 32767, -1) : view.getInt16(at, true)),
    5123: (at: number) => (a.normalized ? view.getUint16(at, true) / 65535 : view.getUint16(at, true)),
    5125: (at: number) => view.getUint32(at, true),
    5126: (at: number) => view.getFloat32(at, true),
  }[a.componentType]!;
  const base = a.byteOffset ?? 0;
  for (let k = 0; k < a.count; k++) for (let c = 0; c < components; c++) values[k * components + c] = read(base + k * stride + c * size);
  return { values, components };
}

/**
 * A file's node: its place (`position`, `rotationQuaternion`, `scaling`, which the clips set), its
 * parent (-1: the `__root__` node that takes glTF into left-handed space).
 */
interface FileNode {
  name: string;
  parent: number;
  t: number[];
  r: Quat;
  s: number[];
}

/** The `__root__` above a file's scene: scaled -1 in z and turned half round y, the file's x turned round. */
const ROOT = compose(1, 1, -1, [0, 1, 0, 0], 0, 0, 0, new Float32Array(16));

/** A clip's channel: frames at 60 a second, from 0 to its group's end (every group starts at frame 0). */
interface Track {
  node: number;
  prop: 't' | 'r' | 's';
  frames: number[];
  values: number[][];
  step: boolean;
}

/** A glTF animation as a group of tracks (from frame 0 to `to`). */
interface Group {
  name: string;
  tracks: Track[];
  to: number;
}

/** The frame rate the clips are keyed at. */
const FPS = 60;

async function importFile(app: pc.AppBase, url: string, now: boolean): Promise<LoadedFile> {
  const file = await importModel(app, url, now);
  const { gltf } = file;
  const nodes: FileNode[] = (gltf.nodes ?? []).map((n, i) => {
    let [t, r, s] = [n.translation ?? [0, 0, 0], (n.rotation ?? [0, 0, 0, 1]) as Quat, n.scale ?? [1, 1, 1]];
    if (n.matrix) [t, r, s] = decompose(n.matrix);
    return { name: n.name ?? `node${i}`, parent: -1, t: [...t], r: [...r] as Quat, s: [...s] };
  });
  // The scene's nodes, in the file's order, each with its parent.
  const inScene = new Set<number>();
  const visit = (i: number, parent: number) => {
    inScene.add(i);
    nodes[i].parent = parent;
    for (const c of gltf.nodes![i].children ?? []) visit(c, i);
  };
  for (const i of gltf.scenes?.[gltf.scene ?? 0]?.nodes ?? []) visit(i, -1);

  /** A node's world matrix (model space), its parents' worked out first (they may have moved too). */
  const world = (i: number, out: Float32Array = new Float32Array(16)): Float32Array => {
    const n = nodes[i];
    const parent = n.parent < 0 ? ROOT : world(n.parent);
    multiply(compose(n.s[0], n.s[1], n.s[2], n.r, n.t[0], n.t[1], n.t[2], new Float32Array(16)), parent, out);
    return out;
  };

  const groups = readGroups(file, nodes);
  const set = (track: Track, frame: number) => {
    const v = evaluate(track, frame);
    const n = nodes[track.node];
    if (track.prop === 't') n.t = v;
    else if (track.prop === 's') n.s = v;
    else n.r = v as Quat;
  };
  const atRest = (group: Group) => {
    for (const track of group.tracks) set(track, 0);
  };
  const atEnd = (group: Group) => {
    for (const track of group.tracks) set(track, group.to);
  };
  // The nodes the clips move: each, with all below it, is a moving part.
  const moved = [...new Set(groups.flatMap((g) => g.tracks.map((t) => t.node)))];
  // At rest: where each clip starts.
  groups.forEach(atRest);
  const rest = moved.map((i) => world(i));
  const partOf = (node: number): number => {
    for (let n = node; n >= 0; n = nodes[n].parent) {
      const i = moved.indexOf(n);
      if (i >= 0) return i;
    }
    return -1;
  };
  // A part grown from nothing (water, flames: scaled to almost 0 at rest) bakes flat, so its normals and the
  // finishes' UVs come from where a clip ends with it largest (`FilePiece.full`).
  const grownBy = moved.map((): Group | null => null);
  const largest = rest.map((m) => Math.abs(determinant(m)));
  for (const group of groups) {
    atEnd(group);
    moved.forEach((node, i) => {
      const size = Math.abs(determinant(world(node)));
      if (size > largest[i]) [largest[i], grownBy[i]] = [size, group];
    });
    atRest(group);
  }
  moved.forEach((_, i) => {
    if (grownBy[i] && Math.abs(determinant(rest[i])) > largest[i] * 0.01) grownBy[i] = null;
  });

  const pieces: FilePiece[] = [];
  const named = new Map<string, Float32Array>();
  (gltf.nodes ?? []).forEach((gn, index) => {
    if (!inScene.has(index)) return;
    if (!named.has(nodes[index].name)) named.set(nodes[index].name, world(index));
    if (gn.mesh === undefined) return;
    const primitives = gltf.meshes?.[gn.mesh]?.primitives ?? [];
    primitives.forEach((primitive, p) => {
      if ((primitive.mode ?? 4) !== 4 || primitive.attributes.POSITION === undefined) return;
      const raw = readPrimitive(file, primitive);
      if (raw.positions.length === 0) return;
      const part = partOf(index);
      // The names of its mesh and the nodes above it (a node's second and later primitives' meshes are named apart, `_primitive<p>`).
      const names: string[] = primitives.length > 1 ? [`${nodes[index].name}_primitive${p}`] : [];
      for (let n = index; n >= 0; n = nodes[n].parent) names.push(nodes[n].name);
      names.push('__root__');
      const m = primitive.material;
      const piece: FilePiece = {
        // Bake the full hierarchy transform so instance matrices place the model directly.
        data: transformData(copyData(raw), world(index)),
        material: m === undefined ? null : (file.materials[m] ?? null),
        materialName: m === undefined ? '__GLTFLoader._default' : (gltf.materials?.[m]?.name ?? `material${m}`),
        part,
        nodes: names,
      };
      const group = part >= 0 ? grownBy[part] : null;
      if (group) {
        // Its normals at rest by the inverse transpose of its matrix (flattened, they lean towards the
        // grow axis): the game's instance matrix (the clip's pose) turns them back into the grown shape's.
        restNormals(piece.data, raw.normals, world(index));
        atEnd(group);
        piece.full = transformData(copyData(raw), world(index));
        atRest(group);
      }
      pieces.push(piece);
    });
  });
  const parts = moved.map((node, i) => ({ name: nodes[node].name, pivot: [rest[i][12], rest[i][13], rest[i][14]] as [number, number, number], rest: rest[i] }));
  // (Each read with the others at rest: a part inside another is read with it closed.)
  const clips = new Map(
    groups.map((group) => {
      const clip = readClip(group, nodes, moved, rest, world, set);
      atRest(group);
      return [group.name, clip];
    }),
  );
  return { pieces, parts, clips, named };
}

/** A primitive's vertex arrays (positions, normals, UVs, RGBA colours, tangents, triangles) as the file has them. */
function readPrimitive(file: Container, primitive: GltfPrimitive): VData {
  const at = primitive.attributes;
  const positions = readAccessor(file, at.POSITION).values;
  const count = positions.length / 3;
  const indices = primitive.indices !== undefined ? Uint32Array.from(readAccessor(file, primitive.indices).values) : Uint32Array.from({ length: count }, (_, i) => i);
  let normals = at.NORMAL !== undefined ? readAccessor(file, at.NORMAL).values : null;
  if (!normals) {
    normals = new Float32Array(positions.length);
    computeNormals(positions, indices, normals);
  }
  let colors: Float32Array | null = null;
  if (at.COLOR_0 !== undefined) {
    const { values, components } = readAccessor(file, at.COLOR_0);
    colors = components === 4 ? values : new Float32Array(count * 4).map((_, i) => (i % 4 === 3 ? 1 : values[(i >> 2) * 3 + (i % 4)]));
  }
  return {
    positions,
    normals,
    uvs: at.TEXCOORD_0 !== undefined ? readAccessor(file, at.TEXCOORD_0).values : null,
    colors,
    tangents: at.TANGENT !== undefined ? readAccessor(file, at.TANGENT).values : null,
    indices,
  };
}

/** The file's animations as tracks (translation, rotation and scale; empty ones left out). */
function readGroups(file: Container, nodes: FileNode[]): Group[] {
  const groups: Group[] = [];
  (file.gltf.animations ?? []).forEach((animation, index) => {
    const tracks: Track[] = [];
    for (const channel of animation.channels) {
      const prop = ({ translation: 't', rotation: 'r', scale: 's' } as const)[channel.target.path as 'translation'];
      const node = channel.target.node;
      if (!prop || node === undefined || !nodes[node]) continue;
      const sampler = animation.samplers[channel.sampler];
      const input = readAccessor(file, sampler.input).values;
      const { values: output, components } = readAccessor(file, sampler.output);
      const cubic = sampler.interpolation === 'CUBICSPLINE';
      // (A cubic spline's keys are read at their values, played as straight lines between them.)
      const stride = cubic ? components * 3 : components;
      const at = cubic ? components : 0;
      tracks.push({
        node,
        prop,
        frames: Array.from(input, (t) => t * FPS),
        values: Array.from(input, (_, k) => Array.from(output.subarray(k * stride + at, k * stride + at + components))),
        step: sampler.interpolation === 'STEP',
      });
    }
    if (!tracks.length) return;
    // From frame 0 to the last key of any channel, each channel held at its first and last values beyond its own keys.
    const to = Math.max(...tracks.map((t) => t.frames[t.frames.length - 1]));
    for (const t of tracks) {
      if (t.frames[0] > 0) {
        t.frames.unshift(0);
        t.values.unshift(t.values[0]);
      }
      if (t.frames[t.frames.length - 1] < to) {
        t.frames.push(to);
        t.values.push(t.values[t.values.length - 1]);
      }
    }
    groups.push({ name: animation.name || `animation${index}`, tracks, to });
  });
  return groups;
}

/** A track's value at `frame` (`Animation.evaluate`, see the header): linear (a rotation spherically), or stepped. */
function evaluate(track: Track, frame: number): number[] {
  const { frames, values } = track;
  let key = 0;
  while (key >= 0 && frame < frames[key]) --key;
  while (key + 1 <= frames.length - 1 && frame >= frames[key + 1]) ++key;
  if (key < 0) return values[0].slice();
  if (key + 1 > frames.length - 1) return values[frames.length - 1].slice();
  const a = values[key];
  const b = values[key + 1];
  if (track.step) return (frames[key + 1] > frame ? a : b).slice();
  const gradient = (frame - frames[key]) / (frames[key + 1] - frames[key]);
  return track.prop === 'r' ? slerp(a, b, gradient) : a.map((v, k) => v + (b[k] - v) * gradient);
}

/** Spherical interpolation of two quaternions (`Quaternion.Slerp`, see the header). */
function slerp(left: number[], right: number[], amount: number): number[] {
  let dot = left[0] * right[0] + left[1] * right[1] + left[2] * right[2] + left[3] * right[3];
  let flip = false;
  if (dot < 0) {
    flip = true;
    dot = -dot;
  }
  let a: number;
  let b: number;
  if (dot > 0.999999) {
    a = 1 - amount;
    b = flip ? -amount : amount;
  } else {
    const angle = Math.acos(dot);
    const inv = 1 / Math.sin(angle);
    a = Math.sin((1 - amount) * angle) * inv;
    b = flip ? -Math.sin(amount * angle) * inv : Math.sin(amount * angle) * inv;
  }
  return [0, 1, 2, 3].map((k) => a * left[k] + b * right[k]);
}

/**
 * A clip's poses of the parts it moves (`ModelClip`): each node's world matrix then, relative to its
 * world matrix at rest. The parts inside one it moves (a fridge's lever on its door) go with it.
 */
function readClip(group: Group, nodes: FileNode[], moved: number[], rest: Float32Array[], world: (i: number, out?: Float32Array) => Float32Array, set: (track: Track, frame: number) => void): ModelClip {
  const duration = Math.max(0, group.to / FPS);
  const samples = Math.max(2, Math.ceil(duration * CLIP_RATE) + 1);
  const animated = new Set(group.tracks.map((t) => t.node));
  // Per moving part: the depth of the outermost node of it, or around it, that the clip moves (-1: none).
  // (The `__root__` node counts as a parent.)
  const depthOf = (node: number): number => {
    let depth = -1;
    let d = 1;
    for (let n = nodes[node].parent; n >= 0; n = nodes[n].parent) d++;
    for (let n = node; n >= 0; n = nodes[n].parent, d--) if (animated.has(n)) depth = d;
    return depth;
  };
  const targets = moved.filter((node) => depthOf(node) >= 0);
  const tracks = targets.map((node) => ({ part: moved.indexOf(node), poses: new Float32Array(samples * 16), depth: depthOf(node) }));
  const inverse = targets.map((node) => invert(rest[moved.indexOf(node)], new Float32Array(16)));
  const now = new Float32Array(16);
  const pose = new Float32Array(16);
  for (let s = 0; s < samples; s++) {
    const frame = Math.min(s / CLIP_RATE, duration) * FPS;
    for (const track of group.tracks) set(track, frame);
    targets.forEach((node, i) => {
      // Vertices at rest, back into the node's own space, then out to where the node is now.
      multiply(inverse[i], world(node, now), pose);
      tracks[i].poses.set(pose, s * 16);
    });
  }
  return { duration, tracks };
}

/** A node's `matrix` as translation, rotation and scale (`Matrix.decompose`, see the header, without a mirror). */
function decompose(m: number[]): [number[], Quat, number[]] {
  const s = [Math.hypot(m[0], m[1], m[2]), Math.hypot(m[4], m[5], m[6]), Math.hypot(m[8], m[9], m[10])];
  if (determinant(m) <= 0) s[1] *= -1;
  const r = [0, 1, 2].map((i) => [0, 1, 2].map((j) => (s[i] ? m[i * 4 + j] / s[i] : 0)));
  // A rotation matrix (rows: where each axis goes) as a quaternion.
  const [[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]] = r;
  const trace = m00 + m11 + m22;
  let q: Quat;
  if (trace > 0) {
    const k = 0.5 / Math.sqrt(trace + 1);
    q = [(m12 - m21) * k, (m20 - m02) * k, (m01 - m10) * k, 0.25 / k];
  } else if (m00 > m11 && m00 > m22) {
    const k = 2 * Math.sqrt(1 + m00 - m11 - m22);
    q = [0.25 * k, (m10 + m01) / k, (m20 + m02) / k, (m12 - m21) / k];
  } else if (m11 > m22) {
    const k = 2 * Math.sqrt(1 + m11 - m00 - m22);
    q = [(m10 + m01) / k, 0.25 * k, (m21 + m12) / k, (m20 - m02) / k];
  } else {
    const k = 2 * Math.sqrt(1 + m22 - m00 - m11);
    q = [(m20 + m02) / k, (m21 + m12) / k, 0.25 * k, (m01 - m10) / k];
  }
  return [[m[12], m[13], m[14]], q, s];
}

function identityMatrix(): Float32Array {
  return translation(0, 0, 0, new Float32Array(16)) as Float32Array;
}

// ---- building from a file -----------------------------------------------------------

/** Copies of a file's materials as entries draw them (as loaded, or restyled by `color`, `roughness`, `metallic`), by restyling. */
const materialCopies = new WeakMap<pc.Material, Map<string, pc.StandardMaterial>>();
/** What a file's meshes without a material of their own are drawn with (white, coloured by the file's vertices), per app. */
const plainFileMaterials = new WeakMap<pc.AppBase, pc.StandardMaterial>();

/**
 * The material a build draws a file's own `material` with: a copy per restyling, made once (the
 * file's stays as loaded, for the next entry's restyling), and adopted into the game's lighting.
 */
function fileMaterial(app: pc.AppBase, lib: MaterialLibrary, material: pc.StandardMaterial | null, ov: MaterialOverride | undefined): pc.StandardMaterial {
  if (!material) {
    let plain = plainFileMaterials.get(app);
    if (!plain) {
      plain = new pc.StandardMaterial();
      plain.name = '__GLTFLoader._default';
      plain.frontFace = GAME_FRONT_FACE;
      plain.diffuseVertexColor = true;
      lib.adopt(plain);
      plain.update();
      plainFileMaterials.set(app, plain);
    }
    return plain;
  }
  const key = `${ov?.color ?? ''}|${ov?.roughness ?? ''}|${ov?.metallic ?? ''}`;
  const copies = materialCopies.get(material) ?? new Map<string, pc.StandardMaterial>();
  materialCopies.set(material, copies);
  let copy = copies.get(key);
  if (!copy) {
    const made = material.clone();
    made.name = `${material.name}${key === '||' ? '' : `|${key}`}`;
    // (Its triangles wound as the game's meshes are, see the file header.)
    made.frontFace = GAME_FRONT_FACE;
    // (PlayCanvas keeps the file's base colour in sRGB: times the restyling's sRGB colour is the
    // product in linear light, as a 2.2 power multiplies.)
    if (ov?.color) {
      const [r, g, b] = hexColor(ov.color);
      made.diffuse.set(made.diffuse.r * r, made.diffuse.g * g, made.diffuse.b * b);
    }
    // (The file's roughness is PlayCanvas's inverted gloss.)
    if (ov?.roughness !== undefined) made.gloss = ov.roughness;
    if (ov?.metallic !== undefined) made.metalness = ov.metallic;
    lib.adopt(made);
    made.update();
    copies.set(key, (copy = made));
  }
  return copy;
}

/** A template from a file read (`filesOf`); throws if the build fails, having let go of what it made. */
function buildGltf(app: pc.AppBase, name: string, entry: ModelEntry, file: LoadedFile, footprint: [number, number], resize: Resize | undefined): ModelTemplate {
  const made: pc.Mesh[] = [];
  try {
    return buildFromFile(app, name, entry, file, footprint, resize, made);
  } catch (err) {
    for (const mesh of made) mesh.destroy();
    throw err;
  }
}

/** `buildGltf`'s build; what it uploads goes into `made`. */
function buildFromFile(app: pc.AppBase, name: string, entry: ModelEntry, file: LoadedFile, footprint: [number, number], resize: Resize | undefined, made: pc.Mesh[]): ModelTemplate {
  const extras = entry as ModelEntry & ModelExtras;
  const fit = extras.fit;
  const [ox, oy, oz] = entry.offset ?? [0, 0, 0];
  // The file's model space to this build's: `scale`, `rotationY` and `offset`, or `fit`.
  let space = entrySpace(entry, new Float32Array(16)) as Float32Array;
  const overrides = extras.materials ?? {};
  const lib = MaterialLibrary.for(app);
  const overrideFor = (p: FilePiece): MaterialOverride | undefined => overrides[p.materialName] ?? namedFinish(p.materialName, lib) ?? overrides['*'];
  const device = app.graphicsDevice;
  const upload = (data: VData) => {
    const mesh = uploadData(device, data);
    made.push(mesh);
    return mesh;
  };

  // This build's copy of the geometry it shows.
  const pieces = file.pieces.filter((p) => !overrideFor(p)?.hide).map((p) => ({ source: p, data: transformData(copyData(p.data), space), full: p.full && transformData(copyData(p.full), space), part: p.part }));
  if (fit && pieces.length) space = multiply(space, fitInto(pieces.map((p) => p.data), fit, extras.align ?? 'centre', [ox, oy, oz]), new Float32Array(16)) as Float32Array;
  let boxes = pieces.map((p) => dataBox(p.data.positions));
  // Resized by the player as primitive parts are; each moving part's pivot goes where its hinge went.
  const pivots = file.parts.map((p) => transformPoint(space, p.pivot[0], p.pivot[1], p.pivot[2]));
  const shifts = pivots.map(() => [0, 0, 0]);
  if (resize && boxes.length && resizes(footprint, resize)) {
    const { maps, slopes } = resizeMaps(footprint, resize, Math.min(...boxes.map((b) => b.lo[1])), Math.max(...boxes.map((b) => b.hi[1])));
    for (const p of pieces) for (const data of p.full ? [p.data, p.full] : [p.data]) resizeData(data, maps, slopes);
    pivots.forEach((p, i) => (shifts[i] = [maps[0](p[0]) - p[0], maps[1](p[1]) - p[1], maps[2](p[2]) - p[2]]));
    boxes = pieces.map((p) => dataBox(p.data.positions));
  }

  // The still body, then each moving part: meshes of their own (merged per finish within each).
  const meshes: ModelMesh[] = [];
  const tints: (TintSlot | null)[] = [];
  const parts: number[] = [];
  let height = 0;
  let bottom = Infinity;
  for (let part = -1; part < file.parts.length; part++) {
    const meshName = part < 0 ? name : `${name}|${file.parts[part].name}`;
    const groups = new PartGroups(app);
    // The file's own materials, a mesh per material (glTF files often split a model into several nodes).
    const kept = new Map<pc.StandardMaterial, VData[]>();
    pieces.forEach(({ source, data, full, part: of }, i) => {
      if (of !== part) return;
      height = Math.max(height, boxes[i].hi[1]);
      bottom = Math.min(bottom, boxes[i].lo[1]);
      const ov = overrideFor(source);
      if (ov?.finish) {
        const finish = groups.finish(ov.finish);
        const base: [number, number, number] = source.material ? [source.material.diffuse.r, source.material.diffuse.g, source.material.diffuse.b] : [1, 1, 1];
        groups.add(restyledData(data, base, ov, finish.alpha, full), tintInName(source.materialName), finish.group, null);
        return;
      }
      const material = fileMaterial(app, lib, source.material, ov);
      // (Baked into game space, `transformData` turned its winding round with its x: turned back, the
      // builders' winding (`pushTri`), the file's front faces still in front for `GAME_FRONT_FACE`.)
      const own = copyData(data);
      flipFaces(own.indices);
      kept.set(material, [...(kept.get(material) ?? []), own]);
    });
    if (part < 0) {
      extras.parts?.forEach((p, i) => groups.addPart(p, 0, i));
      height = Math.max(height, groups.height);
      bottom = Math.min(bottom, groups.bottom);
    }
    const merged = [...kept].map(([material, list]): ModelMesh => ({ name: meshName, mesh: upload(mergeData(list)), material }));
    const restyled = groups.build(meshName);
    made.push(...restyled.meshes.map((m) => m.mesh));
    // A part grown from nothing lies flat where it hides at rest: leaf flutter would lift it into view (nature).
    if (pieces.some((p) => p.part === part && p.full)) for (const mesh of restyled.meshes) mesh.windStill = true;
    meshes.push(...merged, ...restyled.meshes);
    tints.push(...merged.map((m) => tintInName(m.material.name)), ...restyled.tints);
    parts.push(...[...merged, ...restyled.meshes].map(() => part));
  }

  const lo = (k: number) => Math.min(...boxes.map((b) => b.lo[k]));
  const hi = (k: number) => Math.max(...boxes.map((b) => b.hi[k]));
  const outline = boxes.length ? { minX: lo(0), maxX: hi(0), minZ: lo(2), maxZ: hi(2) } : undefined;
  // Mirror glass: the pieces of a `mirror` material (on a moving part, following it), and primitive parts drawn with the file.
  const mirrors: MirrorSurface[] = [
    ...pieces.flatMap(({ source, data, part }, i) => {
      const side = overrideFor(source)?.mirror;
      return side ? [{ ...fileMirror(data, boxes[i], side), part }] : [];
    }),
    ...(extras.parts ?? []).filter((p) => p.mirror).map(mirrorSurface),
  ];
  // What glows (a lamp's shade at night, a screen while it's used): its faces again, in the glow's
  // colour, a mesh per part they're on.
  const glowing = Map.groupBy(
    pieces.filter(({ source }) => overrideFor(source)?.glow),
    ({ part }) => part,
  );
  const glows = [...glowing].map(([part, list]) => ({
    part,
    // A screen (a TV, a monitor: its `screen`, `crt` or `projection` material, or a manifest's
    // `screen`) is only lit while someone is using it; a lamp's shade or a fire glows with its room.
    screen: list.some(({ source }) => overrideFor(source)?.screen ?? /screen|crt|projection/i.test(source.materialName)),
    mesh: {
      name: `${name}|glow${part < 0 ? '' : `|${file.parts[part].name}`}`,
      mesh: upload(glowData(list.map(({ source, data }) => ({ data, color: overrideFor(source)!.glow! })))),
      material: glowMaterial(app),
    },
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
      ? { motion: { parts, clips: buildClips(file.clips, space, shifts), names: file.parts.map((p) => p.name), rests: file.parts.map((p) => multiply(p.rest, space, new Float32Array(16)) as Float32Array) } }
      : {}),
  };
}

/**
 * A file's clips in a build's model space: the file's `space` (`scale`, `rotationY`, `offset`,
 * `fit`) taken into account, and each part's pose moved with its pivot (`shifts`) when resized.
 */
function buildClips(clips: Map<string, ModelClip>, space: Mat4, shifts: number[][]): Map<string, ModelClip> {
  const inverse = invert(space, new Float32Array(16));
  const out = new Float32Array(16);
  const back = new Float32Array(16);
  const on = new Float32Array(16);
  return new Map(
    [...clips].map(([name, clip]) => [
      name,
      {
        duration: clip.duration,
        tracks: clip.tracks.map(({ part, poses, depth }) => {
          const [x, y, z] = shifts[part];
          translation(-x, -y, -z, back);
          translation(x, y, z, on);
          const built = new Float32Array(poses.length);
          for (let o = 0; o < poses.length; o += 16) {
            // Back to where the part was before resizing, into the file's space, posed, and out again.
            multiply(back, inverse, out);
            multiply(out, poses.subarray(o, o + 16), out);
            multiply(out, space, out);
            multiply(out, on, out);
            built.set(out, o);
          }
          return { part, poses: built, depth };
        }),
      },
    ]),
  );
}

/** The material glows are drawn with, per app: unlit, added to what's behind, coloured per vertex. */
const glowMaterials = new WeakMap<pc.AppBase, pc.StandardMaterial>();
/** How far (depth-buffer steps) a glow is drawn in front of the faces it lights. */
const GLOW_DEPTH_STEPS = -4;

/**
 * Glowing faces' material: unlit, its emissive white times the vertex colour (the glow's; the
 * instance's strength is the materials' instance colour), added on top, a hair in front of the
 * faces it lights.
 */
function glowMaterial(app: pc.AppBase): pc.StandardMaterial {
  let material = glowMaterials.get(app);
  if (material) return material;
  material = new pc.StandardMaterial();
  material.name = 'glow';
  material.useLighting = false;
  material.diffuse.set(0, 0, 0);
  material.emissive.set(1, 1, 1);
  material.emissiveVertexColor = true;
  // The glow's colour (linear) linearised once more, as the look was tuned: the glow's colour is taken
  // as sRGB (and linearised for the image processing).
  (material as pc.StandardMaterial & { vertexColorGamma: boolean }).vertexColorGamma = true;
  material.blendType = pc.BLEND_ADDITIVE;
  material.depthWrite = false;
  // Its faces are the lit ones again, so it only has to win their own depth: a few steps of the
  // depth buffer, not a slope-scaled offset, which at a glancing angle (or from far off) pulled a
  // glow through the parts in front of it (the grill's coals through its shut lid).
  material.depthBias = GLOW_DEPTH_STEPS;
  material.slopeDepthBias = 0;
  // Both sides (a shade glows inside too).
  material.cull = pc.CULLFACE_NONE;
  material.useFog = false;
  material.update();
  glowMaterials.set(app, material);
  return material;
}

/** Each glow material's copy with a strength per instance (`glowByInstance`). */
const instanceGlows = new WeakMap<pc.Material, pc.StandardMaterial>();

/**
 * A glow's material drawn with a strength per instance (the `INSTANCE_COLOR` stream, `instancing.ts`):
 * its vertex colour (the glow's) times the instance's colour (a lamp's shade as bright as its lamp,
 * a screen as its flicker), linearised together (`vertexColorGamma`). Draws without that stream keep the glow's own material.
 */
export function glowByInstance(app: pc.AppBase, glow: pc.Material): pc.Material {
  if (!(glow instanceof pc.StandardMaterial)) return glow;
  let copy = instanceGlows.get(glow);
  if (!copy) {
    copy = glow.clone();
    copy.name = `${glow.name}|instance`;
    installGameChunks(copy, app.graphicsDevice);
    // (Unlit: none of the look; the instance's colour scales the vertex colour, `shaders/surface.ts`.)
    copy.setDefine('GAME_LOOK', false);
    copy.setAttribute('game_instanceColor', INSTANCE_COLOR_SEMANTIC);
    copy.setDefine('GAME_GLOW_TINT', '');
    copy.update();
    instanceGlows.set(glow, copy);
  }
  return copy;
}
