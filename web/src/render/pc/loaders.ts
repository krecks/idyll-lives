/**
 * What the production build's compressed files need (`cargo assets`, `crates/tools/src/assets.rs`): KTX2 textures
 * (Basis Universal ETC1S) transcoded by PlayCanvas's Basis transcoder, served from our own
 * `decoders/basis/` (`cargo decoders`), and glTF geometry compressed with `EXT_meshopt_compression`,
 * decoded here with meshoptimizer's decoder through PlayCanvas's buffer-view hook (PlayCanvas has
 * no meshopt support of its own), on workers of its own (a big model's geometry took the main thread
 * for milliseconds). The dev server serves the uncompressed sources.
 */

import { MeshoptDecoder } from 'meshoptimizer/decoder';
import { withBuild } from '../../assets/build';
import * as pc from './engine';

const BASE = new URL(`${import.meta.env.BASE_URL}decoders/basis/`, location.href).href;
let basisStarted = false;
/** Workers decoding meshopt geometry (started with the first compressed model). */
const MESHOPT_WORKERS = 2;
let meshoptWorkers = false;

/** Decodes a meshopt buffer view on the decoder's workers (or here, if workers can't start). */
function decodeMeshopt(count: number, stride: number, source: Uint8Array, mode: MeshoptExtension['mode'], filter?: MeshoptExtension['filter']): Promise<Uint8Array> {
  if (!meshoptWorkers) {
    meshoptWorkers = true;
    try {
      MeshoptDecoder.useWorkers(MESHOPT_WORKERS);
    } catch (err) {
      console.warn('[render] meshopt workers unavailable, decoding on the page', err);
    }
  }
  // (The decoder hands a worker a copy of the source: the file's buffer stays for its other views.)
  return MeshoptDecoder.decodeGltfBufferAsync(count, stride, source, mode, filter);
}

/** Sets up the Basis transcoder (once a page; its workers start with the first KTX2). */
export function setupDecoders(): void {
  if (basisStarted) return;
  basisStarted = true;
  // (By their content hash in a build, as the game's data: kept across releases.)
  pc.basisInitialize({ glueUrl: withBuild(`${BASE}basis.wasm.js`), wasmUrl: withBuild(`${BASE}basis.wasm.wasm`), lazyInit: true, numWorkers: 2 });
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
        .then(([buffer]) => decodeMeshopt(ext.count, ext.byteStride, new Uint8Array(buffer.buffer, buffer.byteOffset + (ext.byteOffset ?? 0), ext.byteLength), ext.mode, ext.filter))
        .then((target) => callback(null, target))
        .catch((err: unknown) => callback(err, null));
    },
  },
};
