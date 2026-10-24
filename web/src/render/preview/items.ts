/**
 * Catalog pictures of furniture: a small PlayCanvas app of its own (beside the game's, on a device
 * of its own: WebGPU, or WebGL2 as the game's setting says) that builds each model with the game's
 * own pipeline (`buildModel`: the same files and finishes as in play), lights it like a product
 * shot (`itemScene.ts`), draws it into a multisampled target of the picture's size and reads it
 * back into a cached image URL.
 *
 * Requests are queued and drawn one model at a time: what's on screen first, in order (cards ask
 * when they scroll into view and withdraw when they leave), then pictures drawn ahead of time in
 * idle moments. Encoding a picture overlaps building the next model. Pictures are kept for the
 * page and, in Cache Storage, between visits (keyed by app version and the model's manifest
 * entry). Turntables (a model seen from `frames` angles around it) feed the detail pane's
 * preview. The app is released a few seconds after the queue empties.
 *
 * The game's frames come first: pictures drawn ahead of time wait for an idle moment, one model
 * each, and a picture's read-back waits for the GPU without holding the main thread (`read`).
 */

import * as pc from '../pc/engine';
import { withoutBuild } from '../../assets/build';
import type { AssetRegistry } from '../../assets/registry';
import { openCache } from '../../ml/assets';
import { colorsKey, type PartColors } from '../../core/protocol';
import { buildModel } from '../pc/models';
import { shadingFromSettings } from '../pc/shaders/softLook';
import { toEngine } from '../pc/space';
import { painted } from '../shared/modelEntry';
import type { ItemPreviews, ItemStage } from '../types';
import { boxCorners, contactShadow, PICTURE_STAND_IN, placeModel, shadersCompiling, studioScene, texturesReady, type StudioScene } from './itemScene';
import { ItemStageController } from './itemStage';

/** Picture sizes in pixels (cards show ~80 CSS px; the detail's turntable ~200). */
const THUMB_PX = 192;
const TURN_PX = 320;
/** Give up waiting for a model's shaders (ms): drawn then, without what isn't ready. */
const SHADERS_TIMEOUT = 8000;
/** Idle time (ms) before the app is released. */
const RELEASE_AFTER = 5000;
const TURNTABLE_CACHE = 24;
/** A picture of a box (its model's file failed) is handed out again for this long (ms) before the model is drawn again, its file tried again. */
const BOX_AGAIN_MS = 30_000;
/** Three-quarter view from the front, a little to the side and above (radians). */
const YAW = -0.62;
const PITCH = 0.5;
/**
 * Smallest half-height of the view (m), so a pot plant doesn't fill the frame like a bed; small
 * enough that a mug on a top still shows (about a third of the picture).
 */
const MIN_HALF = 0.3;

/** `now`: on screen (drawn first, in request order); `idle`: drawn ahead of time when nothing else waits. */
export type Priority = 'now' | 'idle';

interface Job {
  key: string;
  model: string;
  footprint: [number, number];
  /** Its parts' colours (turntables of a placed object). */
  colors?: PartColors;
  frames: number;
  size: number;
  priority: Priority;
  waiters: ((urls: string[] | null) => void)[];
}

const thumbs = new Map<string, string>();
const turntables = new Map<string, string[]>();
/**
 * Pictures kept between visits (Cache Storage), in a cache per app version: older versions' caches
 * are deleted when it opens. Bump `v3` to drop old pictures after a look change. Pictures are kept
 * per shading (Settings, Performance, Shading), here and in memory.
 */
const STORE_PREFIX = 'littlelives-item-pictures-';
const STORE = `${STORE_PREFIX}v3-${__APP_VERSION__}`;

/** A model's picture in the shading the setting asks for now (`thumbs`' key). */
function thumbKey(model: string): string {
  return `${shadingFromSettings()}|${model}`;
}

/** `backend`: `webgl` forces WebGL2 (as the game's renderer setting does; `?renderer=webgl` too). */
export function createItemPreviews(assets: AssetRegistry, backend: () => 'auto' | 'webgl' = () => 'auto'): ItemPreviews {
  return new ItemThumbnails(assets, backend);
}

class ItemThumbnails implements ItemPreviews {
  private studio: Promise<ItemStudio | null> | null = null;
  private live: ItemStudio | null = null;
  private unavailable = false;
  private readonly queue: Job[] = [];
  private running = false;
  private releaseTimer = 0;
  private store: Promise<Cache | null> | null = null;
  /**
   * Pictures of a box (the model's file failed), by job key: not kept, but the latest is handed out
   * again for a while (`BOX_AGAIN_MS`); every one handed out is let go once the real one comes.
   */
  private readonly boxed = new Map<string, { urls: string[]; at: number; all: string[] }>();

  constructor(
    private readonly assets: AssetRegistry,
    private readonly backend: () => 'auto' | 'webgl',
  ) {}

  cachedThumbnail(model: string): string | null {
    return thumbs.get(thumbKey(model)) ?? null;
  }

  thumbnail(model: string, footprint: [number, number], priority: Priority = 'now', signal?: AbortSignal): Promise<string | null> {
    const key = thumbKey(model);
    const hit = thumbs.get(key);
    if (hit) return Promise.resolve(hit);
    return this.stored(model).then((url) => {
      if (url) {
        thumbs.set(key, url);
        return url;
      }
      if (signal?.aborted) return null;
      return this.request(key, model, footprint, 1, THUMB_PX, priority, signal).then((urls) => urls?.[0] ?? null);
    });
  }

  prefetch(models: readonly { model: string; footprint: [number, number] }[]): void {
    for (const m of models) void this.thumbnail(m.model, m.footprint, 'idle');
  }

  cachedTurntable(model: string, frames: number, colors?: PartColors): string[] | null {
    return turntables.get(`${thumbKey(model)}|${frames}|${colorsKey(colors)}`) ?? null;
  }

  turntable(model: string, footprint: [number, number], frames: number, colors?: PartColors): Promise<string[] | null> {
    const key = `${thumbKey(model)}|${frames}|${colorsKey(colors)}`;
    const hit = turntables.get(key);
    if (hit) {
      turntables.delete(key);
      turntables.set(key, hit);
      return Promise.resolve(hit);
    }
    return this.request(key, model, footprint, frames, TURN_PX, 'now', undefined, colors);
  }

  async stage(host: HTMLElement, model: string, footprint: [number, number]): Promise<ItemStage | null> {
    if (this.unavailable) return null;
    try {
      return await ItemStageController.create(host, this.assets, model, footprint, this.backend());
    } catch (err) {
      console.warn('[preview] item stage unavailable', err);
      return null;
    }
  }

  /**
   * Pictures drawn for `key`: of a box (`boxed`), handed out again for a while, or the real one, which
   * lets go of every box picture handed out before it (whoever showed one asks again, and now gets
   * the real one).
   */
  private boxPictures(key: string, urls: string[], boxed: boolean): void {
    const before = this.boxed.get(key);
    if (boxed) {
      this.boxed.set(key, { urls, at: performance.now(), all: [...(before?.all ?? []), ...urls] });
      return;
    }
    for (const u of before?.all ?? []) URL.revokeObjectURL(u);
    this.boxed.delete(key);
  }

  /** Keeps what was drawn, even if whoever asked has gone meanwhile. */
  private remember(job: Job, urls: string[]): void {
    if (job.frames === 1) {
      if (thumbs.has(job.key)) return;
      thumbs.set(job.key, urls[0]);
      void this.keep(job.model, urls[0]);
      return;
    }
    turntables.set(job.key, urls);
    while (turntables.size > TURNTABLE_CACHE) {
      const [old, oldUrls] = turntables.entries().next().value!;
      turntables.delete(old);
      for (const u of oldUrls) URL.revokeObjectURL(u);
    }
  }

  private request(
    key: string,
    model: string,
    footprint: [number, number],
    frames: number,
    size: number,
    priority: Priority,
    signal?: AbortSignal,
    colors?: PartColors,
  ): Promise<string[] | null> {
    if (this.unavailable) return Promise.resolve(null);
    // A box drawn a moment ago is handed out again rather than drawn once more for each card.
    const box = this.boxed.get(key);
    if (box && performance.now() - box.at < BOX_AGAIN_MS) return Promise.resolve(box.urls);
    return new Promise((resolve) => {
      let job = this.queue.find((j) => j.key === key);
      if (!job) {
        job = { key, model, footprint, colors, frames, size, priority, waiters: [] };
        this.queue.push(job);
      } else if (priority === 'now' && job.priority === 'idle') {
        // Wanted on screen now: ahead of everything drawn ahead of time.
        job.priority = 'now';
        this.queue.splice(this.queue.indexOf(job), 1);
        this.queue.push(job);
      }
      const waiter = (urls: string[] | null) => resolve(urls);
      job.waiters.push(waiter);
      signal?.addEventListener('abort', () => {
        const j = job!;
        j.waiters = j.waiters.filter((w) => w !== waiter);
        resolve(null);
        // Nobody wants it any more (scrolled away, another category): don't draw it.
        const i = this.queue.indexOf(j);
        if (!j.waiters.length && j.priority === 'now' && i >= 0) this.queue.splice(i, 1);
      }, { once: true });
      void this.pump();
    });
  }

  /** The next job: on-screen requests in order, then (when the page is idle) the rest. */
  private next(): Job | undefined {
    const i = this.queue.findIndex((j) => j.priority === 'now');
    return i >= 0 ? this.queue.splice(i, 1)[0] : this.queue.shift();
  }

  private async pump(): Promise<void> {
    if (this.running) return;
    this.running = true;
    clearTimeout(this.releaseTimer);
    try {
      const studio = await this.open();
      while (this.queue.length) {
        // Drawing ahead of time yields to the game: one model per idle moment.
        if (!this.queue.some((j) => j.priority === 'now')) await idle();
        const job = this.next();
        if (!job) continue;
        let encoded: Promise<string[] | null> = Promise.resolve(null);
        let boxed = false;
        if (studio) {
          try {
            // The images are encoded while the next model is built.
            const drawn = await studio.draw(job.model, this.assets, job.footprint, job.frames, job.size, job.colors);
            encoded = drawn?.encoded ?? encoded;
            boxed = !!drawn?.boxed;
          } catch (err) {
            console.warn(`[preview] thumbnail of ${job.model} failed`, err);
          }
        }
        void encoded.then(
          (urls) => {
            // A box drawn for a file that failed is shown but not kept (here or between visits): the
            // next request draws the thing again, with its file if it has come.
            if (urls) this.boxPictures(job.key, urls, boxed);
            if (urls && !boxed) this.remember(job, urls);
            for (const w of job.waiters) w(urls);
          },
          () => job.waiters.forEach((w) => w(null)),
        );
      }
    } finally {
      this.running = false;
      this.idleCheck();
    }
  }

  // ---- pictures kept between visits ----------------------------------------------------

  private cache(): Promise<Cache | null> {
    this.store ??= typeof caches === 'undefined' ? Promise.resolve(null) : openCache(STORE_PREFIX, STORE);
    return this.store;
  }

  /** Cache key: changes with the app version, the shading and the model's manifest entry (its file, fit, finishes; not the build). */
  private storeKey(model: string): string {
    const entry = this.assets.has(model, 'model') ? withoutBuild(JSON.stringify(this.assets.get(model, 'model'))) : '';
    return `${location.origin}/__item-pictures/${__APP_VERSION__}/${shadingFromSettings()}/${THUMB_PX}/${hash(model + entry)}.webp`;
  }

  private async stored(model: string): Promise<string | null> {
    try {
      const res = await (await this.cache())?.match(this.storeKey(model));
      return res ? URL.createObjectURL(await res.blob()) : null;
    } catch {
      return null;
    }
  }

  private async keep(model: string, url: string): Promise<void> {
    try {
      const store = await this.cache();
      if (!store) return;
      const blob = await (await fetch(url)).blob();
      await store.put(this.storeKey(model), new Response(blob, { headers: { 'Content-Type': 'image/webp' } }));
    } catch {
      // Storage full or unavailable: pictures are drawn again next time.
    }
  }

  private open(): Promise<ItemStudio | null> {
    this.studio ??= ItemStudio.create(this.assets, this.backend()).then(
      (studio) => (this.live = studio),
      (err: unknown) => {
        console.warn('[preview] 3D item pictures unavailable', err);
        this.unavailable = true;
        this.studio = null;
        return null;
      },
    );
    return this.studio;
  }

  private idleCheck(): void {
    clearTimeout(this.releaseTimer);
    this.releaseTimer = window.setTimeout(() => {
      const studio = this.live;
      if (!studio || this.running || this.queue.length) return;
      this.live = null;
      this.studio = null;
      studio.dispose();
    }, RELEASE_AFTER);
  }
}

/** Resolves when the browser has a quiet moment (or after a short while regardless). */
function idle(): Promise<void> {
  return new Promise((resolve) => {
    if ('requestIdleCallback' in window) requestIdleCallback(() => resolve(), { timeout: 400 });
    else setTimeout(resolve, 30);
  });
}

/** FNV-1a, as hex. */
function hash(text: string): string {
  let h = 0x811c9dc5;
  for (let i = 0; i < text.length; i++) h = Math.imul(h ^ text.charCodeAt(i), 0x01000193);
  return (h >>> 0).toString(16);
}

class ItemStudio {
  private readonly camera: pc.CameraComponent;
  private readonly shadow: pc.Entity;
  /** A target per picture size (multisampled, as Babylon's canvas was), made on first use. */
  private readonly targets = new Map<number, pc.RenderTarget>();
  private last = performance.now();

  private constructor(private readonly scene: StudioScene) {
    const cam = (this.camera = scene.camera);
    cam.projection = pc.PROJECTION_ORTHOGRAPHIC;
    cam.aspectRatioMode = pc.ASPECT_MANUAL;
    cam.aspectRatio = 1;
    cam.nearClip = 0.05;
    cam.farClip = 40;
    this.shadow = contactShadow(scene, scene.root);
  }

  static async create(assets: AssetRegistry, backend: 'auto' | 'webgl'): Promise<ItemStudio> {
    // (Drawn into targets of its own: the canvas is only the device's.)
    const canvas = document.createElement('canvas');
    canvas.id = 'item-pictures';
    canvas.width = canvas.height = 16;
    return new ItemStudio(await studioScene(canvas, assets, { backend, antialias: false, alpha: false }));
  }

  /**
   * Builds `model` and draws it from `frames` angles around it. Resolves once drawn; `encoded`
   * (the image URLs, one per angle) settles later, so the next model can be built meanwhile.
   * `boxed`: its file failed (to load or to build), and the picture is of a plain box in its place.
   */
  async draw(model: string, assets: AssetRegistry, footprint: [number, number], frames: number, size: number, colors?: PartColors): Promise<{ encoded: Promise<string[]>; boxed: boolean } | null> {
    const { app, lib, root } = this.scene;
    lib.setShading(shadingFromSettings());
    const entry = assets.has(model, 'model') ? assets.get(model, 'model') : undefined;
    const template = await buildModel(app, `thumb:${model}`, painted(entry, colors), footprint, { standIn: entry?.url ? PICTURE_STAND_IN : undefined });
    const placed = placeModel(this.scene, root, template);
    try {
      const corners = boxCorners(template.meshes);
      if (!corners.length || !(await texturesReady(lib))) return null;

      // The contact shadow is the model's own ground size, under its middle (it turns with the
      // model), not its footprint's: a mug's tile-sized shadow ran off the picture's edges.
      let [x0, x1, z0, z1] = [Infinity, -Infinity, Infinity, -Infinity];
      for (const v of corners) [x0, x1, z0, z1] = [Math.min(x0, v.x), Math.max(x1, v.x), Math.min(z0, v.z), Math.max(z1, v.z)];
      const [sx, sz] = [(x0 + x1) / 2, (z0 + z1) / 2];
      this.shadow.setLocalScale((x1 - x0) * 1.3 + 0.08, 1, (z1 - z0) * 1.3 + 0.08);

      const angles = Array.from({ length: frames }, (_, i) => (i * Math.PI * 2) / frames);
      this.frame(corners, angles);
      const target = this.target(size);
      this.camera.renderTarget = target;
      // Its shaders compiled first, between the page's frames (WebGL2: `asyncShaders`).
      for (const started = performance.now(); ; ) {
        this.render();
        if (!shadersCompiling(this.scene.device) || performance.now() - started > SHADERS_TIMEOUT) break;
        await new Promise((r) => setTimeout(r, 16));
      }
      const turned = new Float32Array(16);
      const urls: Promise<string>[] = [];
      for (const angle of angles) {
        // Turned about y in game space, as Babylon's rotation.y (+x towards -z).
        const c = Math.cos(angle);
        const s = Math.sin(angle);
        turned.set([c, 0, -s, 0, 0, 1, 0, 0, s, 0, c, 0, 0, 0, 0, 1]);
        for (const draw of placed.draws) {
          draw.setMatrices(turned, 1);
          draw.update();
        }
        // (A local turn about y under the world root is the same turn in game space.)
        this.shadow.setLocalEulerAngles(0, (angle * 180) / Math.PI, 0);
        this.shadow.setLocalPosition(sx * c + sz * s, 0.004, -sx * s + sz * c);
        this.render();
        urls.push(this.capture(target, size));
      }
      return { encoded: Promise.all(urls), boxed: !!template.boxed };
    } finally {
      placed.destroy();
    }
  }

  dispose(): void {
    for (const target of this.targets.values()) {
      target.destroyTextureBuffers();
      target.destroy();
    }
    this.targets.clear();
    this.scene.destroy();
  }

  /** One frame of the scene (the wind's clock moving on by the time since the last). */
  private render(): void {
    const now = performance.now();
    const app = this.scene.app;
    app.update(Math.min(0.1, (now - this.last) / 1000));
    this.last = now;
    app.render();
  }

  private target(size: number): pc.RenderTarget {
    let target = this.targets.get(size);
    if (!target) {
      const texture = new pc.Texture(this.scene.device, {
        name: `itemPicture${size}`,
        width: size,
        height: size,
        format: pc.PIXELFORMAT_RGBA8,
        mipmaps: false,
        minFilter: pc.FILTER_LINEAR,
        magFilter: pc.FILTER_LINEAR,
        addressU: pc.ADDRESS_CLAMP_TO_EDGE,
        addressV: pc.ADDRESS_CLAMP_TO_EDGE,
      });
      target = new pc.RenderTarget({ name: `itemPicture${size}`, colorBuffer: texture, depth: true, samples: 4 });
      this.targets.set(size, target);
    }
    return target;
  }

  /** Points the camera so the model fills the picture at every angle drawn (game space, as Babylon's camera stood). */
  private frame(corners: pc.Vec3[], angles: number[]): void {
    const dir = new pc.Vec3(Math.sin(YAW) * Math.cos(PITCH), Math.sin(PITCH), Math.cos(YAW) * Math.cos(PITCH));
    const forward = dir.clone().mulScalar(-1);
    const right = new pc.Vec3().cross(pc.Vec3.UP, forward).normalize();
    const up = new pc.Vec3().cross(forward, right).normalize();
    let minR = Infinity;
    let maxR = -Infinity;
    let minU = Infinity;
    let maxU = -Infinity;
    const p = new pc.Vec3();
    for (const a of angles) {
      const c = Math.cos(a);
      const s = Math.sin(a);
      for (const v of corners) {
        // Babylon's rotation.y turns +x towards -z.
        p.set(v.x * c + v.z * s, v.y, -v.x * s + v.z * c);
        const r = p.dot(right);
        const u = p.dot(up);
        minR = Math.min(minR, r);
        maxR = Math.max(maxR, r);
        minU = Math.min(minU, u);
        maxU = Math.max(maxU, u);
      }
    }
    const half = Math.max(MIN_HALF, ((maxR - minR) / 2) * 1.14, ((maxU - minU) / 2) * 1.14);
    const centre = right.clone().mulScalar((minR + maxR) / 2).add(up.clone().mulScalar((minU + maxU) / 2));
    const eye = centre.clone().add(dir.clone().mulScalar(12));
    const entity = this.camera.entity;
    entity.setPosition(toEngine(eye.x, eye.y, eye.z, new pc.Vec3()));
    entity.lookAt(toEngine(centre.x, centre.y, centre.z, new pc.Vec3()));
    this.camera.orthoHeight = half;
  }

  /**
   * Reads the frame just drawn back (recorded now, before the next one is drawn into the same
   * target) and encodes it in the background. The rows come back bottom-up on WebGL2
   * (`readPixels`) and top-down on WebGPU (`copyTextureToBuffer`); premultiplied, as drawn over
   * the clear transparent black.
   */
  private async capture(target: pc.RenderTarget, size: number): Promise<string> {
    // (From the texture itself, which the multisampled target resolves into at the end of its pass: WebGL2 can't read a multisampled framebuffer.)
    const read = target.colorBuffer.read(0, 0, size, size, { immediate: true }) as Promise<Uint8Array> | undefined;
    if (!read) throw new Error('read-back unavailable');
    const pixels = await read;
    const img = new ImageData(size, size);
    const row = size * 4;
    const bottomUp = !this.scene.device.isWebGPU;
    const out = img.data;
    for (let y = 0; y < size; y++) {
      const from = (bottomUp ? size - 1 - y : y) * row;
      out.set(pixels.subarray(from, from + row), y * row);
    }
    // Straight alpha for the canvas (the edges and the shade were drawn over transparent black).
    for (let i = 0; i < out.length; i += 4) {
      const a = out[i + 3];
      if (a === 0 || a === 255) continue;
      const k = 255 / a;
      out[i] = Math.min(255, out[i] * k);
      out[i + 1] = Math.min(255, out[i + 1] * k);
      out[i + 2] = Math.min(255, out[i + 2] * k);
    }
    const canvas = document.createElement('canvas');
    canvas.width = canvas.height = size;
    canvas.getContext('2d')!.putImageData(img, 0, 0);
    const blob = await new Promise<Blob>((resolve, reject) => canvas.toBlob((b) => (b ? resolve(b) : reject(new Error('toBlob failed'))), 'image/webp', 0.9));
    return URL.createObjectURL(blob);
  }
}
