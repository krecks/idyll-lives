/**
 * Shaders and pipelines made once for what draws the same, where PlayCanvas 2.23 would make them again:
 *
 * - Unlit materials. `StandardMaterial.getShaderVariant` fills one options object, kept from call to
 *   call, and keys its shader cache by it. An unlit material (`useLighting` false) skips the lights and
 *   leaves `litOptions.lightMaskDynamic` as the material before it set it (whether that one's mesh had
 *   the lamps): two keys for the same code, so the glass of a mirror, a lamp's glow or the garden's
 *   light pools compiled again in play, depending on what was drawn just before. Here it is set the
 *   same (true) for every unlit material, through the material's `onUpdateShader` hook, which runs
 *   before the key is made.
 * - Meshes built from their streams (`Mesh.setPositions` and the rest: the house, the street, every
 *   game-made mesh) keep their attributes one after another in one buffer, so the attributes' offsets
 *   in it, and with them the vertex format's `renderingHash`, depend on the vertex count. WebGPU's
 *   render pipelines are cached by that hash, but a pipeline doesn't depend on those offsets (each
 *   attribute is bound as a buffer of its own, at its offset, when drawn): every mesh with another
 *   vertex count made a pipeline of its own, the walls built again for a wall mode one more. On
 *   WebGPU such formats get a hash without the offsets (its own numbers, apart from PlayCanvas's);
 *   WebGL2 keys its vertex arrays by the hash with the buffer, and keeps PlayCanvas's.
 *
 * Once per page, for every app (the previews' too). `check:chunks` checks what this relies on.
 */

import * as pc from './engine';

type Options = { litOptions: { lightMaskDynamic: boolean } };
type Hook = ((options: Options) => Options) | undefined;

/** A vertex format as far as its hash is worked out here (PlayCanvas 2.23's fields). */
interface Format {
  device: pc.GraphicsDevice;
  interleaved: boolean;
  elements: { name: string; dataType: number; numComponents: number; normalize: boolean; asInt: boolean; stride: number; size: number }[];
  renderingHash: number;
  _evaluateHash(): void;
}

/** The hashes of non-interleaved formats without their offsets: numbers from 2³⁰ up (PlayCanvas's count from 0). */
const streamHashes = new Map<string, number>();

let done = false;

export function steadyShaderKeys(): void {
  if (done) return;
  done = true;
  const material = pc.StandardMaterial.prototype as unknown as { getShaderVariant(this: pc.StandardMaterial, params: unknown): pc.Shader };
  const variant = material.getShaderVariant;
  material.getShaderVariant = function (params) {
    if (this.useLighting) return variant.call(this, params);
    const m = this as unknown as { onUpdateShader: Hook };
    const own = m.onUpdateShader;
    m.onUpdateShader = (options) => {
      options.litOptions.lightMaskDynamic = true;
      return own ? own(options) : options;
    };
    try {
      return variant.call(this, params);
    } finally {
      m.onUpdateShader = own;
    }
  };
  // SAFETY: `VertexFormat._evaluateHash` sets `renderingHash` (the constructor and `update` call it);
  // on WebGPU the hash is read only by the render pipeline cache and the draw's pipeline check.
  const format = pc.VertexFormat.prototype as unknown as Format;
  const evaluate = format._evaluateHash;
  format._evaluateHash = function (this: Format) {
    evaluate.call(this);
    if (this.interleaved || !this.device?.isWebGPU) return;
    const key = this.elements.map((e) => `${e.name}${e.dataType}${e.numComponents}${e.normalize}${e.asInt}${e.stride}${e.size}`).join('_');
    let hash = streamHashes.get(key);
    if (hash === undefined) streamHashes.set(key, (hash = 2 ** 30 + streamHashes.size));
    this.renderingHash = hash;
  };
}
