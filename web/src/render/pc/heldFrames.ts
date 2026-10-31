/**
 * Held frames (the preset's `holdStillFrames`, for the menus' town): a view standing still is drawn
 * once and shown again, not drawn anew, until something in it changes. The canvas is the render
 * target that holds it: a frame not drawn leaves the canvas's last picture on screen (WebGPU's
 * canvas keeps its last texture, WebGL2's its last drawing buffer), so a held frame costs neither
 * the CPU (no culling, no draw calls) nor the GPU.
 *
 * What changes a held view, each found without being told:
 * - the game camera's pose, lens, layers or picture size; the drawing's size (a resized canvas starts blank);
 * - the light: the sun's way and colour, the fog, the clear colour (the town's hour easing);
 * - what the layers draw: a draw added to or taken from any layer (builds, things streaming in);
 * - a file or texture that came (`app.assets`, `onModelFileArrived`), the page shown again.
 * What it can't see (a material's colour or a draw's matrices changed in place: the overview's
 * highlights, birds flying) its owner says with `changed()`.
 *
 * After a change it draws `SETTLE` frames more before it holds (late material swaps, uploads). It
 * holds only while `active` (the renderer: the menus' town standing still, no build running) and
 * the preset allows; a frame something waits for (`draw(true)`: `framesRendered`) is always drawn.
 * Wind and the clouds' drift stand still while a frame is held. Per frame: no allocations.
 */

import * as pc from './engine';
import { onModelFileArrived } from './models';

/** Frames still drawn after a change before the view is held. */
const SETTLE = 3;
/** Numbers sampled each frame (see `sample`). */
const SAMPLES = 64;

export class HeldFrames {
  /** Holding is allowed now (the renderer says: the menus' town, nothing being built). */
  active = false;
  /** The last frame was held (not drawn). */
  holding = false;
  private left = SETTLE;
  private readonly last = new Float64Array(SAMPLES);
  private readonly now = new Float64Array(SAMPLES);
  private readonly unlisten: (() => void)[] = [];
  private readonly onChange = () => this.changed();

  /** `camera`: the view held; `sun`: its light's way and colour count as the view's. */
  constructor(
    private readonly app: pc.AppBase,
    private readonly camera: pc.CameraComponent,
    private readonly sun: pc.Entity,
    /** The preset's `holdStillFrames`. */
    private readonly enabled: boolean,
  ) {
    app.assets.on('load', this.onChange);
    app.graphicsDevice.on('resizecanvas', this.onChange);
    document.addEventListener('visibilitychange', this.onChange);
    this.unlisten.push(
      () => app.assets.off('load', this.onChange),
      () => app.graphicsDevice.off('resizecanvas', this.onChange),
      () => document.removeEventListener('visibilitychange', this.onChange),
      onModelFileArrived(app, this.onChange),
    );
  }

  /** Something in the view changed that `draw` can't see: the next frames are drawn. */
  changed(): void {
    this.left = SETTLE;
  }

  /**
   * Whether to draw this frame (call it once a frame, before drawing, the camera posed): false while
   * the view is held. `wanted`: something waits for this frame to be drawn.
   */
  draw(wanted = false): boolean {
    this.holding = false;
    if (!this.enabled || !this.active) {
      this.left = SETTLE;
      return true;
    }
    this.sample(this.now);
    if (!same(this.now, this.last)) {
      this.last.set(this.now);
      this.left = SETTLE;
    }
    if (this.left > 0) {
      this.left--;
      return true;
    }
    if (wanted) return true;
    this.holding = true;
    return false;
  }

  /** The numbers whose change changes the view (see the file's header). */
  private sample(out: Float64Array): void {
    let k = 0;
    const m = this.camera.entity.getWorldTransform().data;
    for (let i = 0; i < 16; i++) out[k++] = m[i];
    const c = this.camera;
    out[k++] = c.fov;
    out[k++] = c.nearClip;
    out[k++] = c.farClip;
    const cc = c.clearColor;
    out[k++] = cc.r;
    out[k++] = cc.g;
    out[k++] = cc.b;
    const ids = c.layers;
    let layerKey = 0;
    for (let i = 0; i < ids.length; i++) layerKey = layerKey * 31 + ids[i] + 1;
    out[k++] = layerKey;
    const device = this.app.graphicsDevice;
    out[k++] = device.width;
    out[k++] = device.height;
    const fog = this.app.scene.fog;
    out[k++] = fog.density;
    out[k++] = fog.color.r;
    out[k++] = fog.color.g;
    out[k++] = fog.color.b;
    const q = this.sun.getRotation();
    out[k++] = q.x;
    out[k++] = q.y;
    out[k++] = q.z;
    out[k++] = q.w;
    const light = this.sun.light;
    out[k++] = light ? light.color.r : 0;
    out[k++] = light ? light.color.g : 0;
    out[k++] = light ? light.color.b : 0;
    // Draws per layer (folded, so any number of layers fits).
    const layers = this.app.scene.layers.layerList;
    const start = k;
    for (let i = k; i < SAMPLES; i++) out[i] = 0;
    for (let i = 0; i < layers.length; i++) {
      const at = start + (i % (SAMPLES - start));
      out[at] = out[at] * 31 + layers[i].meshInstances.length + (layers[i].enabled ? 0 : 0.5);
    }
  }

  destroy(): void {
    for (const off of this.unlisten.splice(0)) off();
  }
}

function same(a: Float64Array, b: Float64Array): boolean {
  for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return false;
  return true;
}
