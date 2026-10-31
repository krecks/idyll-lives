/**
 * Instanced drawing on PlayCanvas: one draw call for every copy of a mesh (a template's mesh,
 * `models.ts`), each copy placed by a matrix of its own.
 *
 * The matrices are in game space (the layout of `shared/math.ts`: 16 floats, translation at
 * 12, 13, 14, which is PlayCanvas's column-major `Mat4.data`), and the draw hangs under the world
 * root (`space.ts`): PlayCanvas's instancing transform is `matrix_model * instance`, so the root's
 * mirror applies to every copy, and its negative scale turns the faces' winding round with it.
 *
 * Per instance, one interleaved vertex buffer (PlayCanvas instances from a single buffer):
 *
 *   floats 0..15   the matrix, its four columns at `SEMANTIC_ATTR11`, `ATTR12`, `ATTR14`, `ATTR15`
 *                  (`instance_line1..4`, as PlayCanvas's own instancing format has them)
 *   then           each stream asked for, in the order given: `INSTANCE_COLOR` (RGBA at
 *                  `INSTANCE_COLOR_SEMANTIC`, ATTR10; drawn with `MaterialLibrary.instanceColored`),
 *                  and whatever a later shader feature declares (wind, blankets: `InstanceStream`,
 *                  on attributes the mesh doesn't use: not ATTR13, PlayCanvas's tangent)
 *
 * The buffer grows as copies are added (doubling) and is written straight into the GPU buffer's
 * CPU copy, uploaded by `update` once per change (only the copies drawn: the room past them isn't). The draw is culled by the box round all its
 * copies (`update` works it out), or not at all (`cull: false`: things spread over the whole lot).
 * The mesh and material stay their owner's: a draw destroyed never takes its mesh with it (a
 * template's is shared by its draws, build after build; the template's owner destroys it).
 */

import * as pc from './engine';
import { INSTANCE_COLOR_SEMANTIC } from './materials';

/** A per-instance stream beside the matrix: the vertex attribute (semantic) a shader reads it from, and its floats. */
export interface InstanceStream {
  semantic: string;
  components: 1 | 2 | 3 | 4;
}

/** The instance colour (RGBA; sRGB where the material converts vertex colours). */
export const INSTANCE_COLOR: InstanceStream = { semantic: INSTANCE_COLOR_SEMANTIC, components: 4 };

/** The matrix's columns' vertex attributes (PlayCanvas's `instance_line1..4`). */
const MATRIX_SEMANTICS = [pc.SEMANTIC_ATTR11, pc.SEMANTIC_ATTR12, pc.SEMANTIC_ATTR14, pc.SEMANTIC_ATTR15];

export interface InstancesOptions {
  name?: string;
  /** Streams beside the matrix (`INSTANCE_COLOR`, ...). */
  streams?: readonly InstanceStream[];
  castShadows?: boolean;
  receiveShadows?: boolean;
  /** Layer ids (default: the world layer). */
  layers?: number[];
  /** Copies room is made for at first (it grows as needed). */
  capacity?: number;
  /** Culled by the box round its copies (default), or never (false). */
  cull?: boolean;
}

/** Every draw of a mesh, by mesh (a template set aside hides its draws: `templateCache.ts`). */
const drawsOf = new WeakMap<pc.Mesh, Set<Instances>>();

/** The instanced draws of `mesh` there are. */
export function instancesOf(mesh: pc.Mesh): ReadonlySet<Instances> {
  return drawsOf.get(mesh) ?? new Set();
}

export class Instances {
  readonly entity: pc.Entity;
  readonly meshInstance: pc.MeshInstance;
  /** Copies drawn. */
  count = 0;
  /** Floats per instance, and where each stream starts in it. */
  readonly stride: number;
  private readonly offsets = new Map<InstanceStream, number>();
  private readonly format: pc.VertexFormat;
  private buffer: pc.VertexBuffer | null = null;
  /** The buffer's CPU copy (`capacity` instances). */
  private data = new Float32Array(0);
  private capacity = 0;
  private readonly cull: boolean;
  private readonly box = new pc.BoundingBox();
  /** The entity's `_aabbVer` the world box was last worked out at (-1: work it out again, the box changed). */
  private worldVer = -1;
  private readonly lo = new pc.Vec3();
  private readonly hi = new pc.Vec3();

  constructor(
    private readonly device: pc.GraphicsDevice,
    parent: pc.Entity,
    readonly mesh: pc.Mesh,
    material: pc.Material,
    options: InstancesOptions = {},
  ) {
    const elements: { semantic: string; components: number; type: number }[] = MATRIX_SEMANTICS.map((semantic) => ({ semantic, components: 4, type: pc.TYPE_FLOAT32 }));
    let stride = 16;
    for (const stream of options.streams ?? []) {
      this.offsets.set(stream, stride);
      elements.push({ semantic: stream.semantic, components: stream.components, type: pc.TYPE_FLOAT32 });
      stride += stream.components;
    }
    this.stride = stride;
    this.format = new pc.VertexFormat(device, elements);
    this.cull = options.cull ?? true;
    // On its parent's app: the page's current one may be another (`device.ts` `createSideApp`).
    // SAFETY: an entity keeps its app in `_app` (the constructor's `app`; tools/render/check_chunks.ts).
    this.entity = new pc.Entity(options.name ?? 'instances', (parent as unknown as { _app: pc.AppBase })._app);
    this.meshInstance = new pc.MeshInstance(mesh, material, this.entity);
    this.reserve(Math.max(1, options.capacity ?? 1));
    this.meshInstance.instancingCount = 0;
    this.entity.addComponent('render', {
      meshInstances: [this.meshInstance],
      castShadows: options.castShadows ?? true,
      receiveShadows: options.receiveShadows ?? true,
      ...(options.layers ? { layers: options.layers } : {}),
    });
    parent.addChild(this.entity);
    let draws = drawsOf.get(mesh);
    if (!draws) drawsOf.set(mesh, (draws = new Set()));
    draws.add(this);
    byEntity.set(this.entity, this);
    // The mesh is its owner's (a template's, a cache's): a draw holds it, never destroys it (a mesh
    // instance destroyed lets go of its mesh, and destroys it once nothing else holds it).
    mesh.incRefCount();
  }

  get enabled(): boolean {
    return this.entity.enabled;
  }
  set enabled(on: boolean) {
    this.entity.enabled = on;
  }

  get material(): pc.Material {
    return this.meshInstance.material!;
  }
  set material(material: pc.Material) {
    this.meshInstance.material = material;
  }

  /** Room for `count` copies (the buffer is made again, larger, keeping what it held). */
  reserve(count: number): void {
    if (count <= this.capacity) return;
    const capacity = Math.max(count, this.capacity * 2);
    const buffer = new pc.VertexBuffer(this.device, this.format, capacity, { usage: pc.BUFFER_DYNAMIC });
    const data = new Float32Array(buffer.lock() as ArrayBuffer);
    data.set(this.data.subarray(0, Math.min(this.data.length, data.length)));
    // Made on the GPU now, whole: a draw before the first `update` (the warm-up's) has its buffer.
    buffer.unlock();
    this.buffer?.destroy();
    this.buffer = buffer;
    this.data = data;
    this.capacity = capacity;
    this.meshInstance.setInstancing(buffer, false);
    this.meshInstance.cull = this.cull;
  }

  /** Sets the number of copies drawn (room made for them). */
  setCount(count: number): void {
    this.reserve(count);
    this.count = count;
  }

  /** Copy `i`'s matrix (game space) from `m` at `offset`. */
  setMatrix(i: number, m: ArrayLike<number>, offset = 0): void {
    const o = i * this.stride;
    for (let k = 0; k < 16; k++) this.data[o + k] = m[offset + k];
  }

  /** `count` copies' matrices, 16 floats each (game space); the count drawn becomes `count`. */
  setMatrices(matrices: ArrayLike<number>, count = Math.floor(matrices.length / 16)): void {
    this.setCount(count);
    if (this.stride === 16) {
      this.data.set(matrices instanceof Float32Array && matrices.length === count * 16 ? matrices : Array.prototype.slice.call(matrices, 0, count * 16), 0);
      return;
    }
    for (let i = 0; i < count; i++) this.setMatrix(i, matrices, i * 16);
  }

  /** Copy `i`'s values of `stream` (its `components` floats from `values` at `offset`). */
  set(stream: InstanceStream, i: number, values: ArrayLike<number>, offset = 0): void {
    const at = this.offsets.get(stream);
    if (at === undefined) throw new Error(`no ${stream.semantic} stream in this draw`);
    const o = i * this.stride + at;
    for (let k = 0; k < stream.components; k++) this.data[o + k] = values[offset + k];
  }

  /** Every copy's values of `stream`, `components` floats each. */
  setAll(stream: InstanceStream, values: ArrayLike<number>): void {
    for (let i = 0; i < this.count; i++) this.set(stream, i, values, i * stream.components);
  }

  /**
   * Uploads what changed (the whole buffer) and draws `count` copies, its box round them (worked out
   * again unless `bounds` is false: copies that stay about where they were, a fan's blades turning).
   */
  update(bounds = true): void {
    if (!this.buffer) return;
    // Only the copies drawn (at least one's room: a new buffer's first upload makes it, whole).
    this.buffer.unlock(0, Math.max(1, this.count) * this.stride * 4);
    this.meshInstance.instancingCount = this.count;
    if (this.cull && bounds) this.updateBox();
  }

  /** Whether it has been destroyed. */
  get destroyed(): boolean {
    return this.buffer === null;
  }

  /** The box round every copy (game space: the entity's own, under the root), from the mesh's. */
  private updateBox(): void {
    const local = this.mesh.aabb;
    const c = local.center;
    const h = local.halfExtents;
    if (this.count === 0) return;
    const lo = this.lo.set(Infinity, Infinity, Infinity);
    const hi = this.hi.set(-Infinity, -Infinity, -Infinity);
    const d = this.data;
    for (let i = 0; i < this.count; i++) {
      const o = i * this.stride;
      // (An all-zero matrix is a copy hidden: the lawn's indoors.)
      if (d[o + 15] === 0) continue;
      for (let k = 0; k < 8; k++) {
        const x = c.x + (k & 1 ? h.x : -h.x);
        const y = c.y + (k & 2 ? h.y : -h.y);
        const z = c.z + (k & 4 ? h.z : -h.z);
        const px = x * d[o] + y * d[o + 4] + z * d[o + 8] + d[o + 12];
        const py = x * d[o + 1] + y * d[o + 5] + z * d[o + 9] + d[o + 13];
        const pz = x * d[o + 2] + y * d[o + 6] + z * d[o + 10] + d[o + 14];
        if (px < lo.x) lo.x = px;
        if (py < lo.y) lo.y = py;
        if (pz < lo.z) lo.z = pz;
        if (px > hi.x) hi.x = px;
        if (py > hi.y) hi.y = py;
        if (pz > hi.z) hi.z = pz;
      }
    }
    // (Every copy hidden: nothing to see, nowhere.)
    if (lo.x > hi.x) {
      lo.set(0, 0, 0);
      hi.set(0, 0, 0);
    }
    this.box.setMinMax(lo, hi);
    this.worldVer = -1;
    // The world box is worked out again only when the box or the entity's place changes: for a custom box
    // PlayCanvas does it every frame, 0.08 ms a frame on Low and 0.19 ms with its woods in squares (518 draws).
    // SAFETY: a mesh instance asks `_updateAabbFunc` for its world box, and a node's `_aabbVer` moves
    // whenever its world transform does (tools/render/check_chunks.ts).
    const mi = this.meshInstance as unknown as { _updateAabbFunc: ((out: pc.BoundingBox) => pc.BoundingBox) | null };
    mi._updateAabbFunc ??= (out) => {
      const ver = (this.entity as unknown as { _aabbVer: number })._aabbVer;
      if (ver !== this.worldVer) {
        out.setFromTransformedAabb(this.box, this.entity.getWorldTransform());
        this.worldVer = ver;
      }
      return out;
    };
  }

  /** Stops drawing and lets go of the buffer (the mesh and material are the template's). */
  destroy(): void {
    if (!this.buffer) return;
    drawsOf.get(this.mesh)?.delete(this);
    byEntity.delete(this.entity);
    this.entity.destroy();
    this.mesh.decRefCount();
    this.buffer.destroy();
    this.buffer = null;
  }
}

/** Each draw by its entity (`destroyDraw`). */
const byEntity = new WeakMap<pc.Entity, Instances>();

/**
 * Takes down a draw by its entity: an instanced one (`Instances.destroy`: its buffer goes, the mesh
 * stays its owner's), or any other entity (its own meshes go with it).
 */
export function destroyDraw(entity: pc.Entity): void {
  const draw = byEntity.get(entity);
  if (draw) draw.destroy();
  else entity.destroy();
}
