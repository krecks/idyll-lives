/**
 * Engine-neutral mesh data (`shared/geometry.ts`) uploaded into Babylon meshes.
 */

import { Mesh, VertexData, type Scene } from './core';
import { WALL_CUT_KIND, type Geo, type MeshData } from '../shared/geometry';

/** A new static mesh from `data`; `updatable` to change its vertices later. */
export function meshFrom(data: MeshData, name: string, scene: Scene, updatable = false): Mesh {
  const mesh = new Mesh(name, scene);
  const v = new VertexData();
  v.positions = data.positions;
  v.normals = data.normals;
  v.uvs = data.uvs;
  v.indices = data.indices;
  if (data.colors) v.colors = data.colors;
  v.applyToMesh(mesh, updatable);
  if (data.cut) mesh.setVerticesData(WALL_CUT_KIND, data.cut, false, 4);
  if (data.vertexAlpha) mesh.hasVertexAlpha = true;
  return mesh;
}

/** Uploads `geo` into a new static mesh (null when empty); `updatable` to change its vertices later. */
export function toMesh(geo: Geo, name: string, scene: Scene, options: { colors?: boolean; cut?: boolean; updatable?: boolean } = {}): Mesh | null {
  return geo.empty ? null : meshFrom(geo.meshData(options), name, scene, options.updatable ?? false);
}
