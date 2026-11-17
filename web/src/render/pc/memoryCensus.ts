/**
 * Where the renderer's bytes are, counted on demand for the gameplay benchmark (`window.__memCensus()`,
 * `debug/gameplayStats.ts`): the copies PlayCanvas keeps on the CPU of what is on the GPU (each
 * buffer's `storage`, textures' typed-array levels), the model files' decoded pieces, and the GPU's
 * vertex and index buffers by who draws them (the scene graph under the root, by the names of the
 * entities three levels down). A walk of the whole scene: never per frame.
 */

import * as pc from './engine';
import type { MemoryCensus } from '../types';

type Buffer = { storage?: ArrayBufferLike | ArrayBufferView | null; numBytes?: number; usage?: number };

/** A buffer's kind (by class: a build's minified names say nothing). */
function kindOf(b: Buffer): string {
  const usage = b.usage === pc.BUFFER_STATIC ? ' static' : ' dynamic';
  if (b instanceof pc.VertexBuffer) return `vertex${usage}`;
  if (b instanceof pc.IndexBuffer) return `index${usage}`;
  if (b instanceof pc.StorageBuffer) return 'storage';
  // (PlayCanvas doesn't export its uniform buffers' class: they're what is left.)
  return 'uniform';
}

/** Bytes of a level of a texture kept on the CPU (a typed array, or a cube's six). */
function levelBytes(level: unknown): number {
  if (ArrayBuffer.isView(level)) return level.byteLength;
  if (Array.isArray(level)) return level.reduce((a: number, l) => a + levelBytes(l), 0);
  return 0;
}

/** `modelFiles`: the model files' pieces as their own count gives them (`models.ts` `modelFileCensus`). */
export function memoryCensus(app: pc.AppBase, modelFiles: MemoryCensus['modelFiles']): MemoryCensus {
  const device = app.graphicsDevice as pc.GraphicsDevice & { buffers: Set<Buffer>; textures: (pc.Texture & { _levels?: unknown[] })[] };
  const storage: MemoryCensus['storage'] = {};
  for (const b of device.buffers) {
    const kind = kindOf(b);
    const s = (storage[kind] ??= { count: 0, bytes: 0, released: 0, gpuBytes: 0 });
    const bytes = b.storage ? b.storage.byteLength : 0;
    s.count++;
    s.bytes += bytes;
    s.gpuBytes += b.numBytes ?? bytes;
    if (!bytes && b.numBytes) s.released++;
  }
  let textureLevels = 0;
  let texturesWithLevels = 0;
  for (const t of device.textures) {
    const bytes = (t._levels ?? []).reduce((a: number, l) => a + levelBytes(l), 0);
    textureLevels += bytes;
    if (bytes) texturesWithLevels++;
  }

  // The geometry the scene draws, by owner: each buffer counted once, under the first owner met.
  const owners: Record<string, { meshes: number; gpuBytes: number; cpuBytes: number }> = {};
  const counted = new Set<unknown>();
  const visit = (entity: pc.GraphNode, path: string, depth: number) => {
    const owner = depth <= 3 ? `${path}/${entity.name || '?'}` : path;
    const render = (entity as pc.Entity).render;
    for (const mi of render?.meshInstances ?? []) {
      const mesh = mi.mesh;
      if (!mesh) continue;
      for (const b of [mesh.vertexBuffer, ...mesh.indexBuffer] as (Buffer | null | undefined)[]) {
        if (!b || counted.has(b)) continue;
        counted.add(b);
        const o = (owners[owner] ??= { meshes: 0, gpuBytes: 0, cpuBytes: 0 });
        o.gpuBytes += b.numBytes ?? 0;
        o.cpuBytes += b.storage?.byteLength ?? 0;
      }
      (owners[owner] ??= { meshes: 0, gpuBytes: 0, cpuBytes: 0 }).meshes++;
      // (An instanced draw's matrices are a buffer of their own.)
      const inst = (mi as pc.MeshInstance & { instancingData?: { vertexBuffer?: Buffer | null } }).instancingData?.vertexBuffer;
      if (inst && !counted.has(inst)) {
        counted.add(inst);
        const o = owners[`${owner} (instances)`] ??= { meshes: 0, gpuBytes: 0, cpuBytes: 0 };
        o.gpuBytes += inst.numBytes ?? 0;
        o.cpuBytes += inst.storage?.byteLength ?? 0;
      }
    }
    for (const child of entity.children) visit(child, owner, depth + 1);
  };
  visit(app.root, '', 0);
  // What no entity draws: templates waiting for their next copy, meshes held for a later build.
  const unreached = { meshes: 0, gpuBytes: 0, cpuBytes: 0 };
  for (const b of device.buffers) {
    if (counted.has(b) || !(b instanceof pc.VertexBuffer || b instanceof pc.IndexBuffer)) continue;
    unreached.meshes++;
    unreached.gpuBytes += b.numBytes ?? 0;
    unreached.cpuBytes += b.storage?.byteLength ?? 0;
  }
  owners['(not drawn)'] = unreached;
  return { storage, textureLevels, texturesWithLevels, modelFiles, owners };
}
