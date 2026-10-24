/**
 * Plain vertex arrays (the simulation's lot meshes, `shared/geometry.ts`'s builder) uploaded into
 * PlayCanvas meshes, in game space (they go under the world root), and the entities that draw them.
 */

import * as pc from './engine';
import { WALL_CUT_SEMANTIC } from './shaders/chunks';
import type { MeshArrays } from '../../core/protocol';
import type { MeshData } from '../shared/geometry';

/**
 * A static mesh from `data` (colours in RGBA, as `Geo` writes them, and the cutaway attribute where
 * it has one: a `cutaway` material needs it); null when there is nothing to draw. `dynamic`: its
 * vertices change later (set again and `update`d).
 */
export function meshFrom(device: pc.GraphicsDevice, data: MeshArrays | MeshData, dynamic = false): pc.Mesh | null {
  if (data.positions.length === 0) return null;
  const mesh = new pc.Mesh(device);
  if (dynamic) mesh.clear(true, false);
  mesh.setPositions(data.positions);
  mesh.setNormals(data.normals);
  if (data.uvs.length) mesh.setUvs(0, data.uvs);
  if ('colors' in data && data.colors) mesh.setColors(data.colors);
  if ('cut' in data && data.cut) mesh.setVertexStream(WALL_CUT_SEMANTIC, data.cut, 4);
  mesh.setIndices(data.indices);
  mesh.update(pc.PRIMITIVE_TRIANGLES);
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

