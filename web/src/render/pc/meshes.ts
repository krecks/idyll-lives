/**
 * Plain vertex arrays (the simulation's lot meshes, `shared/geometry.ts`'s builder) uploaded into
 * PlayCanvas meshes, in game space (they go under the world root), and the entities that draw them.
 */

import * as pc from './engine';
import { WALL_CUT_SEMANTIC } from './shaders/chunks';
import type { MeshArrays } from '../../core/protocol';
import type { MeshData } from '../shared/geometry';

/**
 * How a mesh made here is used after it's made: `static` (drawn as made: once on the GPU, the copy
 * PlayCanvas keeps on the CPU is let go, `gpuOnly`), `dynamic` (its vertices change later: set again
 * and `update`d, written into that copy) or `read` (as made, but its vertices are read back:
 * `getPositions`, the birds' roof ridge).
 */
export type MeshUse = 'static' | 'dynamic' | 'read';

/**
 * A mesh from `data` (colours in RGBA, as `Geo` writes them, and the cutaway attribute where it has
 * one: a `cutaway` material needs it); null when there is nothing to draw. `use`: see `MeshUse`.
 * `Geo`'s colours and cutaway are a byte a channel: the colours read back as 0..1 (normalised), the
 * cutaway as its whole numbers (PlayCanvas converts them to floats on WebGPU too: `shaders/cutaway.ts`
 * reads them as before).
 */
export function meshFrom(device: pc.GraphicsDevice, data: MeshArrays | MeshData, use: MeshUse = 'static'): pc.Mesh | null {
  if (data.positions.length === 0) return null;
  const mesh = new pc.Mesh(device);
  if (use === 'dynamic') mesh.clear(true, false);
  mesh.setPositions(data.positions);
  mesh.setNormals(data.normals);
  if (data.uvs.length) mesh.setUvs(0, data.uvs);
  if ('colors' in data && data.colors) mesh.setColors32(data.colors);
  if ('cut' in data && data.cut) mesh.setVertexStream(WALL_CUT_SEMANTIC, data.cut, 4, undefined, pc.TYPE_UINT8, false);
  mesh.setIndices(data.indices);
  mesh.update(pc.PRIMITIVE_TRIANGLES);
  if (use === 'static') gpuOnly(mesh);
  return mesh;
}

/** What a buffer `gpuOnly` let go of holds on the CPU: nothing (a reader finds no vertices, never wrong ones). */
const LET_GO = new ArrayBuffer(0);

/**
 * Lets go of the copy of `mesh`'s vertices and indices PlayCanvas keeps on the CPU (each buffer's
 * `storage`) once they are on the GPU: a few hundred MB of a Big world's town, street and ground.
 * Only for a mesh nothing reads back (`getPositions`, `getIndices`, a `VertexIterator`) and whose
 * streams are never set again; its box (`aabb`) was worked out from the streams as it was made.
 * Static buffers only (a dynamic one is written into its copy) and only once uploaded (both
 * backends copy the data as it's uploaded, `unlock`).
 *
 * PlayCanvas uploads every buffer again from its copy when a lost device comes back
 * (`restoreContext`); the game doesn't come back from one (`devicelost` is the crash screen: reload),
 * so such a buffer has nothing to upload then.
 */
export function gpuOnly(mesh: pc.Mesh): pc.Mesh {
  for (const buffer of [mesh.vertexBuffer, ...mesh.indexBuffer]) {
    if (!buffer || buffer.usage !== pc.BUFFER_STATIC || !buffer.impl.initialized || buffer.storage === LET_GO) continue;
    const bytes = buffer.storage.byteLength;
    buffer.storage = LET_GO;
    // SAFETY: PlayCanvas 2.23's `destroy` takes `storage.byteLength` off the device's count of GPU
    // memory (`adjustVramSizeTracking`, after the constructor's its only call): the bytes on the GPU
    // instead. `restoreContext` uploads the copy again (tools/render/chunks.json).
    const count = buffer.adjustVramSizeTracking.bind(buffer);
    buffer.adjustVramSizeTracking = (vram: unknown) => count(vram, -bytes);
    buffer.restoreContext = () => {};
  }
  return mesh;
}

export interface DrawOptions {
  castShadows?: boolean;
  receiveShadows?: boolean;
  /** Layer ids (default: the world layer). */
  layers?: number[];
  /** Drawn after the other blended draws of a lower index (see `drawAfter`). */
  alphaIndex?: number;
}

/** An entity under `parent` drawing `mesh` in `material` (it takes the mesh: destroying the entity lets go of it). */
export function drawMesh(parent: pc.Entity, name: string, mesh: pc.Mesh, material: pc.Material, options: DrawOptions = {}): pc.Entity {
  const entity = new pc.Entity(name);
  const mi = new pc.MeshInstance(mesh, material);
  if (options.alphaIndex) drawAfter(mi, options.alphaIndex);
  entity.addComponent('render', {
    meshInstances: [mi],
    castShadows: options.castShadows ?? false,
    receiveShadows: options.receiveShadows ?? true,
    ...(options.layers ? { layers: options.layers } : {}),
  });
  parent.addChild(entity);
  return entity;
}

/**
 * An alpha index on a PlayCanvas draw (drawn after the blended draws of a lower index): blended
 * draws go back to front within their draw bucket, and a higher bucket goes first
 * (`Layer._calculateSortDistances`), so a draw with a higher index is put in a lower bucket.
 */
export function drawAfter(mi: pc.MeshInstance, alphaIndex: number): void {
  mi.drawBucket = 127 - alphaIndex;
}

