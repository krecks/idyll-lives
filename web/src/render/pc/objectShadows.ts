/**
 * Low's sun-shadow casters for the lot's objects (`QualitySettings.mergedObjectShadows`): every
 * opaque body mesh's copies baked into meshes of game-space triangles, so the furniture costs the
 * shadow map a few draws, not one per model mesh. The template meshes' own draws then don't cast
 * (see-through ones still do, from their own copies; moving parts don't cast on Low).
 *
 * They are drawn only by the sun's shadow map (the shadow-only layer, `layers.ts`). A copy written
 * anew (hidden: scaled to nothing, every vertex in one point and no triangle left; shown; moved)
 * is baked again in place (`bake`), and the vertices of its mesh uploaded. The copies are split
 * over meshes of at most `CHUNK_VERTICES` (`chunks`): a chair pulled up to the table, which moves a
 * little every frame while someone sits down, uploads its own chunk, not the whole lot's furniture
 * (3 MB a frame on a big world's lot, which held frames up for over 100 ms now and then).
 */

import * as pc from './engine';
import type { PopTarget } from './objectMotion';

/** A body mesh to merge: its vertices (local), and its copies' matrices (game space, 16 floats each). */
export interface ShadowSource {
  mesh: pc.Mesh;
  matrices: Float32Array;
  count: number;
}

/**
 * The most vertices a caster mesh holds (16-bit indices; 384 KB of positions): a big world's lot
 * (about 260 000 vertices) is 8 or 9 meshes, so 8 or 9 draws in the shadow map rather than one, and
 * a copy moved uploads at most this much.
 */
const CHUNK_VERTICES = 32768;

/** One caster mesh: its positions (game space) and whether they changed since the last upload. */
interface Chunk {
  mesh: pc.Mesh;
  positions: Float32Array;
  dirty: boolean;
}

/** One body mesh's copies: its vertices, and for each copy the chunk it's in and the vertex it starts at there. */
interface Part {
  pos: Float32Array;
  matrices: Float32Array;
  count: number;
  chunk: Uint16Array;
  first: Uint32Array;
}

export class ObjectShadows {
  private readonly entities: pc.Entity[] = [];
  private chunks: Chunk[] = [];
  /** Each body mesh's copies, by their matrix buffer (the one a `PopTarget` writes). */
  private readonly parts = new Map<Float32Array, Part[]>();
  private readonly material: pc.StandardMaterial;

  constructor(
    private readonly device: pc.GraphicsDevice,
    private readonly parent: pc.Entity,
    private readonly layer: number,
  ) {
    // (Drawn only into the shadow map: a plain material, both sides.)
    this.material = new pc.StandardMaterial();
    this.material.name = 'objectShadowsMerged';
    this.material.cull = pc.CULLFACE_NONE;
    this.material.useLighting = false;
    this.material.update();
  }

  /** Bakes `sources` into the casters (none without triangles), replacing the last. */
  build(sources: readonly ShadowSource[]): void {
    this.clear();
    // Each copy goes whole into the chunk being filled, or starts the next one.
    const parts: (Part & { idx: number[] })[] = [];
    const sizes: { vertices: number; indices: number }[] = [];
    let open = { vertices: 0, indices: 0 };
    for (const { mesh, matrices, count } of sources) {
      const pos: number[] = [];
      const idx: number[] = [];
      if (!count || !mesh.getPositions(pos) || !mesh.getIndices(idx) || !idx.length) continue;
      const n = pos.length / 3;
      // (A mesh bigger than a chunk has chunks of its own, a copy each.)
      const part = { pos: new Float32Array(pos), idx, matrices, count, chunk: new Uint16Array(count), first: new Uint32Array(count) };
      for (let i = 0; i < count; i++) {
        if (open.vertices && open.vertices + n > CHUNK_VERTICES) {
          sizes.push(open);
          open = { vertices: 0, indices: 0 };
        }
        part.chunk[i] = sizes.length;
        part.first[i] = open.vertices;
        open.vertices += n;
        open.indices += idx.length;
      }
      parts.push(part);
    }
    if (open.vertices) sizes.push(open);
    if (!sizes.length) return;
    const indices = sizes.map((s) => (s.vertices > 65535 ? new Uint32Array(s.indices) : new Uint16Array(s.indices)));
    const at = sizes.map(() => 0);
    this.chunks = sizes.map((s) => ({ mesh: new pc.Mesh(this.device), positions: new Float32Array(s.vertices * 3), dirty: false }));
    for (const part of parts) {
      for (let i = 0; i < part.count; i++) {
        this.bakeCopy(part, i);
        const c = part.chunk[i];
        const base = part.first[i];
        const out = indices[c];
        for (let k = 0; k < part.idx.length; k++) out[at[c]++] = base + part.idx[k];
      }
      const list = this.parts.get(part.matrices);
      if (list) list.push(part);
      else this.parts.set(part.matrices, [part]);
    }
    this.chunks.forEach((chunk, c) => {
      // (Its vertices are written again as copies move: kept dynamic.)
      const mesh = chunk.mesh;
      mesh.clear(true, false);
      mesh.setPositions(chunk.positions);
      mesh.setIndices(indices[c]);
      mesh.update(pc.PRIMITIVE_TRIANGLES);
      chunk.dirty = false;
      const entity = new pc.Entity('objectShadowsMerged');
      entity.addComponent('render', { meshInstances: [new pc.MeshInstance(mesh, this.material)], castShadows: true, receiveShadows: false, layers: [this.layer] });
      this.parent.addChild(entity);
      this.entities.push(entity);
    });
  }

  /**
   * An object's copy written anew (hidden, shown, moved): its vertices in its caster follow,
   * uploaded once for all the copies written in the same task, or at `flush` (the cutaway moves every
   * wall item at once).
   */
  bake(target: PopTarget): void {
    const parts = this.parts.get(target.matrices);
    if (!parts) return;
    for (const part of parts) if (target.index < part.count) this.bakeCopy(part, target.index);
    if (this.uploadDue) return;
    this.uploadDue = true;
    queueMicrotask(() => this.flush());
  }
  private uploadDue = false;

  /** The copies written since the last upload, uploaded now (a caller writing many at once, in a frame, ends with it): each changed caster's vertices. */
  flush(): void {
    if (!this.uploadDue) return;
    this.uploadDue = false;
    for (const chunk of this.chunks) {
      if (!chunk.dirty) continue;
      chunk.dirty = false;
      chunk.mesh.setPositions(chunk.positions);
      chunk.mesh.update(pc.PRIMITIVE_TRIANGLES, false);
    }
  }

  /** Copy `i` of a part into its chunk's positions, as its matrix stands now. */
  private bakeCopy(part: Part, i: number): void {
    const m = part.matrices;
    const o = i * 16;
    const { pos } = part;
    const chunk = this.chunks[part.chunk[i]];
    chunk.dirty = true;
    const start = part.first[i] * 3;
    const out = chunk.positions;
    for (let p = 0; p < pos.length; p += 3) {
      const x = pos[p];
      const y = pos[p + 1];
      const z = pos[p + 2];
      out[start + p] = m[o] * x + m[o + 4] * y + m[o + 8] * z + m[o + 12];
      out[start + p + 1] = m[o + 1] * x + m[o + 5] * y + m[o + 9] * z + m[o + 13];
      out[start + p + 2] = m[o + 2] * x + m[o + 6] * y + m[o + 10] * z + m[o + 14];
    }
  }

  clear(): void {
    for (const e of this.entities) e.destroy();
    this.entities.length = 0;
    this.chunks = [];
    this.parts.clear();
  }

  destroy(): void {
    this.clear();
    this.material.destroy();
  }
}
