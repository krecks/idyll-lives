/**
 * What the production build's compressed files need (`tools/assets/optimize.mjs`): KTX2 textures
 * (Basis Universal ETC1S) transcoded by PlayCanvas's Basis transcoder, served from our own
 * `decoders/basis/` (`cargo decoders`), and glTF geometry compressed with `EXT_meshopt_compression`,
 * decoded here with meshoptimizer's decoder through PlayCanvas's buffer-view hook (PlayCanvas has
 * no meshopt support of its own). The dev server serves the uncompressed sources.
 */

import { MeshoptDecoder } from 'meshoptimizer/decoder';
import * as pc from './engine';

const BASE = new URL(`${import.meta.env.BASE_URL}decoders/basis/`, location.href).href;
let basisStarted = false;

/** Sets up the Basis transcoder (once a page; its workers start with the first KTX2). */
export function setupDecoders(): void {
  if (basisStarted) return;
  basisStarted = true;
  pc.basisInitialize({ glueUrl: `${BASE}basis.wasm.js`, wasmUrl: `${BASE}basis.wasm.wasm`, lazyInit: true, numWorkers: 2 });
}

interface MeshoptExtension {
  buffer: number;
  byteOffset?: number;
  byteLength: number;
  byteStride: number;
  count: number;
  mode: 'ATTRIBUTES' | 'TRIANGLES' | 'INDICES';
  filter?: 'NONE' | 'OCTAHEDRAL' | 'QUATERNION' | 'EXPONENTIAL';
}

/**
 * Options for glTF containers (`asset.options`): a buffer view compressed with
 * `EXT_meshopt_compression` is decoded from its compressed source; others load as they are.
 */
export const containerOptions = {
  bufferView: {
    processAsync(view: { extensions?: { EXT_meshopt_compression?: MeshoptExtension } }, buffers: Promise<Uint8Array>[], callback: (err: unknown, result: Uint8Array | null) => void): void {
      const ext = view.extensions?.EXT_meshopt_compression;
      if (!ext) return callback(null, null);
      Promise.all([buffers[ext.buffer], MeshoptDecoder.ready])
        .then(([buffer]) => {
          const source = new Uint8Array(buffer.buffer, buffer.byteOffset + (ext.byteOffset ?? 0), ext.byteLength);
          const target = new Uint8Array(ext.count * ext.byteStride);
          MeshoptDecoder.decodeGltfBuffer(target, ext.count, ext.byteStride, source, ext.mode, ext.filter);
          callback(null, target);
        })
        .catch((err: unknown) => callback(err, null));
    },
  },
};
