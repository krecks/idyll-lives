/**
 * Low's one sun-shadow caster for the lot's objects (`QualitySettings.mergedObjectShadows`): every
 * opaque body mesh's copies baked into one mesh of game-space triangles, so the furniture costs the
 * shadow map one draw, not one per model mesh. The template meshes' own draws then don't cast
 * (see-through ones still do, from their own copies; moving parts don't cast on Low).
 *
 * It is drawn only by the sun's shadow map (the shadow-only layer, `layers.ts`). A copy written
 * anew (hidden: scaled to nothing, every vertex in one point and no triangle left; shown; moved)
 * is baked again in place (`bake`), and its vertices uploaded.
 */

import * as pc from './engine';
import type { PopTarget } from './objectMotion';

/** A body mesh to merge: its vertices (local), and its copies' matrices (game space, 16 floats each). */
export interface ShadowSource {
  mesh: pc.Mesh;
  matrices: Float32Array;
  count: number;
}

/** One body mesh's copies in the merged mesh: its vertices, and the merged vertex its first copy starts at. */
interface Part {
  pos: Float32Array;
  matrices: Float32Array;
  count: number;
  first: number;
}

export class ObjectShadows {
  private entity: pc.Entity | null = null;
  private mesh: pc.Mesh | null = null;
  private positions = new Float32Array(0);
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

  /** Bakes `sources` into the one caster (none without triangles), replacing the last. */
  build(sources: readonly ShadowSource[]): void {
    this.clear();
    const parts: (Part & { idx: number[] })[] = [];
    let vertices = 0;
    let indexCount = 0;
    for (const { mesh, matrices, count } of sources) {
      const pos: number[] = [];
      const idx: number[] = [];
      if (!count || !mesh.getPositions(pos) || !mesh.getIndices(idx)) continue;
      parts.push({ pos: new Float32Array(pos), idx, matrices, count, first: vertices });
      vertices += (pos.length / 3) * count;
      indexCount += idx.length * count;
    }
    if (!indexCount) return;
    const positions = (this.positions = new Float32Array(vertices * 3));
    const indices = vertices > 65535 ? new Uint32Array(indexCount) : new Uint16Array(indexCount);
    let at = 0;
    for (const part of parts) {
      const n = part.pos.length / 3;
      for (let i = 0; i < part.count; i++) {
        this.bakeCopy(part, i);
        const base = part.first + i * n;
        for (let k = 0; k < part.idx.length; k++) indices[at++] = base + part.idx[k];
      }
      const list = this.parts.get(part.matrices);
      if (list) list.push(part);
      else this.parts.set(part.matrices, [part]);
    }
    // (Its vertices are written again as copies move: kept dynamic.)
    const mesh = (this.mesh = new pc.Mesh(this.device));
    mesh.clear(true, false);
    mesh.setPositions(positions);
    mesh.setIndices(indices);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    const entity = (this.entity = new pc.Entity('objectShadowsMerged'));
    entity.addComponent('render', { meshInstances: [new pc.MeshInstance(mesh, this.material)], castShadows: true, receiveShadows: false, layers: [this.layer] });
    this.parent.addChild(entity);
  }

  /** An object's copy written anew (hidden, shown, moved): its vertices in the caster follow. */
  bake(target: PopTarget): void {
    const parts = this.parts.get(target.matrices);
    const mesh = this.mesh;
    if (!parts || !mesh) return;
    for (const part of parts) if (target.index < part.count) this.bakeCopy(part, target.index);
    mesh.setPositions(this.positions);
    mesh.update(pc.PRIMITIVE_TRIANGLES, false);
  }

  /** Copy `i` of a part into `positions`, as its matrix stands now. */
  private bakeCopy(part: Part, i: number): void {
    const m = part.matrices;
    const o = i * 16;
    const { pos } = part;
    const start = (part.first + (i * pos.length) / 3) * 3;
    const out = this.positions;
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
    this.entity?.destroy();
    this.entity = null;
    this.mesh = null;
    this.parts.clear();
  }

  destroy(): void {
    this.clear();
    this.material.destroy();
  }
}
