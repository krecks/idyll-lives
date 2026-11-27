/**
 * The residents' pose texture on PlayCanvas: RGBA32F, one row per resident (its bone matrices, then
 * its appearance and face morph texels: `shared/characters/layout.ts`), sampled texel by texel by the
 * character shader (`shaders/character.ts`).
 *
 * Each frame only the band of rows that changed goes to the GPU (`uploadRows`): a few of the 96
 * rows most frames, none while the game is paused. PlayCanvas's own `lock`/`unlock` (and `upload`)
 * sends the whole level, and its `write` exists only on WebGL2, makes a pixel buffer each call and
 * waits on a fence. So, per backend:
 *
 * - WebGPU: `GPUQueue.writeTexture` of the band straight from the CPU copy (an offset into it, no
 *   copy made, no row alignment asked of a queue write). It is queued at once, ahead of the frame's
 *   command buffers, which PlayCanvas submits when the frame ends, so the frame draws the new poses.
 * - WebGL2: `texSubImage2D` of the band from the CPU copy (its `srcOffset` overload: no copy made),
 *   the texture bound through the device's own texture-unit cache so PlayCanvas's state stays true.
 *
 * The texture's level is the CPU copy itself, so whenever PlayCanvas uploads it whole (the first
 * time it's drawn, after a lost device restored), it uploads the poses as they are now.
 */

import * as pc from './engine';

/**
 * What the texel upload reaches inside PlayCanvas 2.23 (pinned; `check:chunks` checks the names):
 * the WebGPU device's `GPUDevice` and the texture's `GPUTexture`, the WebGL2 device's texture-unit
 * cache and the texture's GL object.
 */
interface WebgpuDeviceInternals {
  wgpu: GPUDevice;
}
interface TextureInternals {
  impl: { gpuTexture?: GPUTexture | null; _glTexture?: WebGLTexture | null };
  _needsUpload: boolean;
}

/** The WebGPU write's destination, layout and extent, rewritten in place (no allocations per frame). */
const dest: GPUTexelCopyTextureInfo & { origin: [number, number, number] } = { texture: null as unknown as GPUTexture, origin: [0, 0, 0] };
const layout = { offset: 0, bytesPerRow: 0, rowsPerImage: 0 };
const extent = { width: 0, height: 0 };

/**
 * Texels `rect` (`[i0, j0, i1, j1]`, last ones included) of `texture`'s level 0 (no mips), from `data` (its CPU copy,
 * `bytes` a texel), sent to the GPU alone: WebGPU's `GPUQueue.writeTexture` (an offset into the copy, no row alignment
 * asked of a queue write), WebGL2's `texSubImage2D` (`glFormat`, `glType`) with the copy's row length. A texture not on
 * the GPU yet is left to its first draw, which sends the level whole. False: there was nothing on the GPU to write
 * into (no GPU texture, no WebGL2 device), the caller's to send whole or not.
 */
export function uploadTexels(texture: pc.Texture, data: Uint8Array | Float32Array, rect: readonly [number, number, number, number], bytes: number, glFormat: 'RG' | 'RGBA', glType: 'UNSIGNED_BYTE' | 'FLOAT'): boolean {
  const t = texture as pc.Texture & TextureInternals;
  if (t._needsUpload) return true;
  const [i0, j0, i1, j1] = rect;
  const w = i1 - i0 + 1;
  const h = j1 - j0 + 1;
  const offset = (j0 * texture.width + i0) * bytes;
  const device = texture.device;
  if (device.isWebGPU) {
    const gpu = t.impl.gpuTexture;
    if (!gpu) return false;
    dest.texture = gpu;
    dest.origin[0] = i0;
    dest.origin[1] = j0;
    layout.offset = offset;
    layout.bytesPerRow = texture.width * bytes;
    layout.rowsPerImage = h;
    extent.width = w;
    extent.height = h;
    // SAFETY: PlayCanvas 2.23's WebGPU device keeps its `GPUDevice` in `wgpu` (tools/render/chunks.json).
    (device as unknown as WebgpuDeviceInternals).wgpu.queue.writeTexture(dest, data, layout, extent);
    return true;
  }
  if (!(device instanceof pc.WebglGraphicsDevice) || !t.impl._glTexture) return false;
  // SAFETY: bound through the device's texture-unit cache and unpack state, so PlayCanvas's record stays true; the row
  // length, which it never sets, is put back to 0 (tools/render/chunks.json).
  device.activeTexture(0);
  device.bindTexture(texture);
  device.setUnpackFlipY(false);
  device.setUnpackPremultiplyAlpha(false);
  device.setUnpackAlignment(1);
  const g = device.gl;
  g.pixelStorei(g.UNPACK_ROW_LENGTH, texture.width);
  g.texSubImage2D(g.TEXTURE_2D, 0, i0, j0, w, h, g[glFormat], g[glType], data, offset / data.BYTES_PER_ELEMENT);
  g.pixelStorei(g.UNPACK_ROW_LENGTH, 0);
  return true;
}

export class PoseTexture {
  readonly texture: pc.Texture;
  /** The CPU copy (the texture's level 0): `width` texels of 4 floats a row, `height` rows. */
  readonly data: Float32Array;

  constructor(
    device: pc.GraphicsDevice,
    /** Texels a row. */
    readonly width: number,
    /** Rows (residents). */
    readonly height: number,
    /** The CPU copy (`width * height * 4` floats, rewritten in place by its owner); a new one by default. */
    data: Float32Array = new Float32Array(width * height * 4),
    name = 'simPoses',
  ) {
    this.data = data;
    this.texture = new pc.Texture(device, {
      name,
      width,
      height,
      format: pc.PIXELFORMAT_RGBA32F,
      mipmaps: false,
      minFilter: pc.FILTER_NEAREST,
      magFilter: pc.FILTER_NEAREST,
      addressU: pc.ADDRESS_CLAMP_TO_EDGE,
      addressV: pc.ADDRESS_CLAMP_TO_EDGE,
      levels: [this.data],
    });
  }

  /** Sends the whole texture (after a world build changed every row). */
  uploadAll(): void {
    this.texture.upload();
  }

  /** Sends rows `lo` to `hi` (inclusive) of the CPU copy to the GPU, in one write. */
  uploadRows(lo: number, hi: number): void {
    // (A band outside the texture is a caller's mistake the driver would report as a validation error: send nothing.)
    if (hi < lo || lo < 0 || hi >= this.height) return;
    uploadTexels(this.texture, this.data, [0, lo, this.width - 1, hi], 16, 'RGBA', 'FLOAT');
  }

  destroy(): void {
    this.texture.destroy();
  }
}
