/**
 * Plain vertex arrays (the simulation's lot meshes, `shared/geometry.ts`'s builder) uploaded into
 * PlayCanvas meshes, in game space (they go under the world root).
 */

import * as pc from './engine';
import { WALL_CUT_SEMANTIC } from './shaders/chunks';
import type { MeshArrays } from '../../core/protocol';
import type { MeshData } from '../shared/geometry';

/**
 * A static mesh from `data` (colours in RGBA, as `Geo` writes them, and the cutaway attribute where
 * it has one: a `cutaway` material needs it); null when there is nothing to draw.
 */
export function meshFrom(device: pc.GraphicsDevice, data: MeshArrays | MeshData): pc.Mesh | null {
  if (data.positions.length === 0) return null;
  const mesh = new pc.Mesh(device);
  mesh.setPositions(data.positions);
  mesh.setNormals(data.normals);
  if (data.uvs.length) mesh.setUvs(0, data.uvs);
  if ('colors' in data && data.colors) mesh.setColors(data.colors);
  if ('cut' in data && data.cut) mesh.setVertexStream(WALL_CUT_SEMANTIC, data.cut, 4);
  mesh.setIndices(data.indices);
  mesh.update(pc.PRIMITIVE_TRIANGLES);
  return mesh;
}
