/**
 * The save's picture (Babylon's `captureThumbnail`): the game camera's view drawn small into a
 * target kept for it, with the game's look, and read back as a JPEG data URL (quality 0.8, as
 * Babylon's `thumbnailUrl`). Also the debug report's screenshot, at the canvas's size.
 *
 * - A camera of its own, off but for the frame that draws a picture: it takes the game camera's
 *   place, lens, layers and clear colour, and draws into its target through the game's chain
 *   (`PostProcessing.attach`: the linear scene multisampled as the preset says, Babylon's image
 *   processing, the night grade, Low's FXAA). The target and the chain are made once, and resized
 *   for a picture of another size.
 * - It draws in the renderer's next frame, before the game camera (WebGPU hands the canvas a new
 *   texture each frame: a frame of the picture alone would blank the screen). Behind a full-screen
 *   menu the loop draws only frames something waits for, so the picture asks for one (`wantFrame`).
 * - The sun's map is fitted per camera in PlayCanvas (its render data, with the light's matrix),
 *   and the game draws it only when something changed: a picture's frame draws it again for both
 *   (`shadowsMoved`), so the picture's camera has the map's matrix of now.
 * - The read is recorded at the end of that frame (after its passes, before it is submitted). The
 *   rows come back bottom-up on WebGL2 (`readPixels`) and top-down on WebGPU (`copyTextureToBuffer`):
 *   turned the right way up per backend.
 *
 * Babylon drew its picture with the game camera into a target the canvas's size (its post-processes'
 * own, the last of them sampling it down into the picture); this one is drawn at the picture's size.
 */

import * as pc from './engine';
import type { PostAttachment, PostProcessing } from './post';

export interface ThumbnailHost {
  /** A frame drawn soon, even behind a full-screen menu. */
  wantFrame(): void;
  /** The sun's map drawn again in the next frame. */
  shadowsMoved(): void;
}

/** A picture not drawn by then (the loop stopped, frames failing) is given up. */
const WAIT_MS = 3000;
/** Babylon's (`thumbnailUrl`). */
const JPEG_QUALITY = 0.8;

interface Request {
  width: number;
  height: number;
  resolve: (url: string | null) => void;
  timer: ReturnType<typeof setTimeout>;
}

export class Thumbnails {
  private readonly entity: pc.Entity;
  private readonly camera: pc.CameraComponent;
  private target: pc.RenderTarget | null = null;
  private chain: PostAttachment | null = null;
  private readonly waiting: Request[] = [];
  /** The picture this frame draws. */
  private drawing: Request | null = null;
  private readonly onPrerender = () => this.prerender();
  private readonly onPostrender = () => this.postrender();

  constructor(
    private readonly app: pc.AppBase,
    private readonly game: pc.CameraComponent,
    private readonly post: PostProcessing | null,
    private readonly host: ThumbnailHost,
  ) {
    this.entity = new pc.Entity('thumbnailCamera');
    // (Before the game camera.)
    this.entity.addComponent('camera', { priority: game.priority - 1, clearColor: new pc.Color(0, 0, 0, 1) });
    this.camera = this.entity.camera!;
    this.entity.enabled = false;
    app.root.addChild(this.entity);
    app.on('prerender', this.onPrerender);
    app.on('postrender', this.onPostrender);
  }

  /** The view as the game camera has it, `width` × `height`, as a JPEG data URL (null: not drawn). */
  capture(width: number, height: number): Promise<string | null> {
    width = Math.max(1, Math.round(width));
    height = Math.max(1, Math.round(height));
    return new Promise((resolve) => {
      const request: Request = {
        width,
        height,
        resolve,
        timer: setTimeout(() => {
          const i = this.waiting.indexOf(request);
          if (i >= 0) this.waiting.splice(i, 1);
          resolve(null);
        }, WAIT_MS),
      };
      this.waiting.push(request);
      this.host.wantFrame();
    });
  }

  private prerender(): void {
    const request = this.waiting.shift();
    if (!request) return;
    try {
      this.prepare(request.width, request.height);
    } catch (err) {
      console.warn('[render] thumbnail capture failed', err);
      this.finish(request, null);
      return;
    }
    this.drawing = request;
    const from = this.game.entity;
    this.entity.setPosition(from.getPosition());
    this.entity.setRotation(from.getRotation());
    const c = this.camera;
    const g = this.game;
    c.fov = g.fov;
    c.nearClip = g.nearClip;
    c.farClip = g.farClip;
    c.clearColor = g.clearColor;
    // (All of the game camera's layers, the thing in hand and build mode's effects too: Babylon's picture had every rendering group and its particles.)
    if (c.layers.length !== g.layers.length || c.layers.some((id, i) => id !== g.layers[i])) c.layers = g.layers.slice();
    this.entity.enabled = true;
    this.host.shadowsMoved();
  }

  private postrender(): void {
    const request = this.drawing;
    if (!request) return;
    this.drawing = null;
    this.entity.enabled = false;
    const target = this.target!;
    const { width, height } = request;
    // (Recorded now, after this frame's passes, so it reads them.)
    const read = target.colorBuffer.read(0, 0, width, height, { renderTarget: target }) as Promise<Uint8Array> | undefined;
    const bottomUp = !this.app.graphicsDevice.isWebGPU;
    if (!read) this.finish(request, null);
    else
      read.then(
        (pixels) => this.finish(request, thumbnailUrl(pixels, width, height, bottomUp)),
        (err: unknown) => {
          console.warn('[render] thumbnail capture failed', err);
          this.finish(request, null);
        },
      );
    if (this.waiting.length) this.host.wantFrame();
  }

  private finish(request: Request, url: string | null): void {
    clearTimeout(request.timer);
    request.resolve(url);
  }

  /** The target (made once, resized for another size) and the chain into it. */
  private prepare(width: number, height: number): void {
    if (this.target && this.target.width === width && this.target.height === height) return;
    if (this.target) {
      this.target.resize(width, height);
      this.chain?.resize(width, height);
      return;
    }
    const device = this.app.graphicsDevice;
    const texture = new pc.Texture(device, {
      name: 'saveThumbnail',
      width,
      height,
      format: pc.PIXELFORMAT_RGBA8,
      mipmaps: false,
      minFilter: pc.FILTER_LINEAR,
      magFilter: pc.FILTER_LINEAR,
      addressU: pc.ADDRESS_CLAMP_TO_EDGE,
      addressV: pc.ADDRESS_CLAMP_TO_EDGE,
    });
    // (A depth buffer for drawing without the chain; the chain draws its scene into its own.)
    this.target = new pc.RenderTarget({ name: 'saveThumbnail', colorBuffer: texture, depth: !this.post?.frame });
    if (this.post) this.chain = this.post.attach(this.app, this.camera, this.target);
    else this.camera.renderTarget = this.target;
  }

  destroy(): void {
    this.app.off('prerender', this.onPrerender);
    this.app.off('postrender', this.onPostrender);
    for (const request of this.waiting.splice(0)) this.finish(request, null);
    if (this.drawing) this.finish(this.drawing, null);
    this.drawing = null;
    this.chain?.destroy();
    this.chain = null;
    if (this.target) {
      this.target.destroyTextureBuffers();
      this.target.destroy();
      this.target = null;
    }
    this.entity.destroy();
  }
}

/** The picture's rows (from the top, or from the bottom: `bottomUp`) as a JPEG data URL, opaque. */
function thumbnailUrl(data: Uint8Array, width: number, height: number, bottomUp: boolean): string {
  const img = new ImageData(width, height);
  const row = width * 4;
  for (let y = 0; y < height; y++) {
    const from = (bottomUp ? height - 1 - y : y) * row;
    img.data.set(data.subarray(from, from + row), y * row);
  }
  // (The compose leaves alpha as the scene had it: a JPEG of a canvas with alpha below 1 would darken.)
  for (let i = 3; i < img.data.length; i += 4) img.data[i] = 255;
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  canvas.getContext('2d')!.putImageData(img, 0, 0);
  return canvas.toDataURL('image/jpeg', JPEG_QUALITY);
}
