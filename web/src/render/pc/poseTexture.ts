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
 * What the band upload reaches inside PlayCanvas 2.23 (pinned; `check:chunks` checks the names):
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

export class PoseTexture {
  readonly texture: pc.Texture;
  /** The CPU copy (the texture's level 0): `width` texels of 4 floats a row, `height` rows. */
  readonly data: Float32Array;
  /** The WebGPU band write's destination, layout and extent, rewritten in place (no allocations per frame; the
   * destination is made on the first write, when there is a GPU texture to name). */
  private dest: GPUTexelCopyTextureInfo | null = null;
  private readonly origin: [number, number, number] = [0, 0, 0];
  private readonly layout = { offset: 0, bytesPerRow: 0, rowsPerImage: 0 };
  private readonly size = { width: 0, height: 0 };

  constructor(
    private readonly device: pc.GraphicsDevice,
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

  /** Floats a row. */
  get stride(): number {
    return this.width * 4;
  }

  /** Sends the whole texture (after a world build changed every row). */
  uploadAll(): void {
    this.texture.upload();
  }

  /** Sends rows `lo` to `hi` (inclusive) of the CPU copy to the GPU, in one write. */
  uploadRows(lo: number, hi: number): void {
    const device = this.device;
    const texture = this.texture as pc.Texture & TextureInternals;
    // (Not on the GPU yet: the first draw uploads the level whole, as it is then. A band outside the
    // texture is a caller's mistake the driver would report as a validation error: send nothing.)
    if (texture._needsUpload || hi < lo || lo < 0 || hi >= this.height) return;
    const stride = this.stride;
    const rows = hi - lo + 1;
    if (device.isWebGPU) {
      const gpu = texture.impl.gpuTexture;
      if (!gpu) return;
      const { origin, layout, size } = this;
      const dest = (this.dest ??= { texture: gpu, origin });
      dest.texture = gpu;
      origin[1] = lo;
      layout.offset = lo * stride * 4;
      layout.bytesPerRow = stride * 4;
      layout.rowsPerImage = rows;
      size.width = this.width;
      size.height = rows;
      (device as unknown as WebgpuDeviceInternals).wgpu.queue.writeTexture(dest, this.data, layout, size);
      return;
    }
    if (!(device instanceof pc.WebglGraphicsDevice) || !texture.impl._glTexture) return;
    const gl = device;
    gl.activeTexture(0);
    gl.bindTexture(texture);
    gl.setUnpackFlipY(false);
    gl.setUnpackPremultiplyAlpha(false);
    gl.setUnpackAlignment(1);
    const g = gl.gl;
    g.texSubImage2D(g.TEXTURE_2D, 0, 0, lo, this.width, rows, g.RGBA, g.FLOAT, this.data, lo * stride);
  }

  destroy(): void {
    this.texture.destroy();
  }
}
