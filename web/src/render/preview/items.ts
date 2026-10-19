/**
 * Catalog pictures of furniture: a small WebGL engine of its own that builds each model with the
 * game's own pipeline (`buildModel`: the same files and finishes as in play),
 * lights it like a product shot and reads the picture back into a cached image URL.
 *
 * Requests are queued and drawn one model at a time: what's on screen first, in order (cards ask
 * when they scroll into view and withdraw when they leave), then pictures drawn ahead of time in
 * idle moments. Encoding a picture overlaps building the next model. Pictures are kept for the
 * page and, in Cache Storage, between visits (keyed by app version and the model's manifest
 * entry). Turntables (a model seen from `frames` angles around it) feed the detail pane's
 * preview. The engine is released a few seconds after the queue empties.
 */

import { Camera, Engine, FreeCamera, Matrix, TransformNode, Vector3, type Mesh, type Scene } from '../babylon/core';
import { withoutBuild } from '../../assets/build';
import type { AssetRegistry } from '../../assets/registry';
import { openCache } from '../../ml/assets';
import { colorsKey, type PartColors } from '../../core/protocol';
import { buildModel, painted } from '../babylon/models';
import { shadingFromSettings } from '../babylon/softLook';
import type { ItemPreviews, ItemStage } from '../types';
import { boxCorners, contactShadow, studioScene } from './itemScene';
import { ItemStageController } from './itemStage';

/** Picture sizes in pixels (cards show ~80 CSS px; the detail's turntable ~200). */
const THUMB_PX = 192;
const TURN_PX = 320;
/** Idle time (ms) before the engine is released. */
const RELEASE_AFTER = 5000;
/** Give up waiting for a model's materials (ms). */
const READY_TIMEOUT = 8000;
const TURNTABLE_CACHE = 24;
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

export function createItemPreviews(assets: AssetRegistry): ItemPreviews {
  return new ItemThumbnails(assets);
}

class ItemThumbnails implements ItemPreviews {
  private studio: Promise<ItemStudio | null> | null = null;
  private live: ItemStudio | null = null;
  private unavailable = false;
  private readonly queue: Job[] = [];
  private running = false;
  private releaseTimer = 0;
  private store: Promise<Cache | null> | null = null;

  constructor(private readonly assets: AssetRegistry) {}

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
      return await ItemStageController.create(host, this.assets, model, footprint);
    } catch (err) {
      console.warn('[preview] item stage unavailable', err);
      return null;
    }
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
        if (studio) {
          try {
            // The images are encoded while the next model is built.
            encoded = (await studio.draw(job.model, this.assets, job.footprint, job.frames, job.size, job.colors))?.encoded ?? encoded;
          } catch (err) {
            console.warn(`[preview] thumbnail of ${job.model} failed`, err);
          }
        }
        void encoded.then(
          (urls) => {
            if (urls) this.remember(job, urls);
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
    this.studio ??= ItemStudio.create(this.assets).then(
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
  private readonly root: TransformNode;
  private readonly camera: FreeCamera;
  private readonly shadow: Mesh;

  private constructor(
    private readonly engine: Engine,
    private readonly scene: Scene,
    private readonly canvas: HTMLCanvasElement,
  ) {
    this.root = new TransformNode('itemRoot', scene);
    const cam = (this.camera = new FreeCamera('itemCamera', new Vector3(0, 2, 5), scene, false));
    cam.mode = Camera.ORTHOGRAPHIC_CAMERA;
    cam.minZ = 0.05;
    cam.maxZ = 40;
    cam.inputs.clear();
    scene.activeCamera = cam;
    this.shadow = contactShadow(scene);
  }

  static async create(assets: AssetRegistry): Promise<ItemStudio> {
    const canvas = document.createElement('canvas');
    canvas.width = canvas.height = TURN_PX;
    const engine = new Engine(canvas, true, { preserveDrawingBuffer: true, premultipliedAlpha: true, alpha: true, stencil: false, adaptToDeviceRatio: false });
    try {
      return new ItemStudio(engine, await studioScene(engine, assets), canvas);
    } catch (err) {
      engine.dispose();
      throw err;
    }
  }

  /**
   * Builds `model` and draws it from `frames` angles around it. Resolves once drawn; `encoded`
   * (the image URLs, one per angle) settles later, so the next model can be built meanwhile.
   */
  async draw(model: string, assets: AssetRegistry, footprint: [number, number], frames: number, size: number, colors?: PartColors): Promise<{ encoded: Promise<string[]> } | null> {
    const entry = assets.get(model, 'model');
    const template = await buildModel(this.scene, `thumb:${model}`, painted(entry, colors), footprint);
    const meshes = template.meshes;
    try {
      // A file that failed to load gets no picture (asked for again later), never a saved one of what stood in.
      if (template.failed) return null;
      const identity = Matrix.Identity().asArray() as unknown as Float32Array;
      for (const mesh of meshes) {
        mesh.parent = this.root;
        mesh.isPickable = false;
        mesh.thinInstanceSetBuffer('matrix', identity, 16, true);
        mesh.alwaysSelectAsActiveMesh = true;
      }
      this.root.rotation.y = 0;
      const corners = boxCorners(meshes);
      if (!corners.length || !(await this.whenReady(meshes))) return null;

      // The contact shadow is the model's own ground size, under its middle (it turns with the
      // model), not its footprint's: a mug's tile-sized shadow ran off the picture's edges.
      let [x0, x1, z0, z1] = [Infinity, -Infinity, Infinity, -Infinity];
      for (const v of corners) [x0, x1, z0, z1] = [Math.min(x0, v.x), Math.max(x1, v.x), Math.min(z0, v.z), Math.max(z1, v.z)];
      const [sx, sz] = [(x0 + x1) / 2, (z0 + z1) / 2];
      this.shadow.scaling.set((x1 - x0) * 1.3 + 0.08, 1, (z1 - z0) * 1.3 + 0.08);

      const angles = Array.from({ length: frames }, (_, i) => (i * Math.PI * 2) / frames);
      this.frame(corners, angles);
      this.engine.setSize(size, size);
      const urls: Promise<string>[] = [];
      for (const angle of angles) {
        this.root.rotation.y = angle;
        this.shadow.rotation.y = angle;
        // Babylon's rotation.y turns +x towards -z.
        this.shadow.position.x = sx * Math.cos(angle) + sz * Math.sin(angle);
        this.shadow.position.z = -sx * Math.sin(angle) + sz * Math.cos(angle);
        this.scene.render();
        urls.push(this.capture(size));
      }
      return { encoded: Promise.all(urls) };
    } finally {
      for (const mesh of meshes) mesh.dispose(false, false);
    }
  }

  dispose(): void {
    this.scene.dispose();
    this.engine.dispose();
  }

  /** Points the camera so the model fills the picture at every angle drawn. */
  private frame(corners: Vector3[], angles: number[]): void {
    const dir = new Vector3(Math.sin(YAW) * Math.cos(PITCH), Math.sin(PITCH), Math.cos(YAW) * Math.cos(PITCH));
    const forward = dir.scale(-1);
    const right = Vector3.Cross(Vector3.Up(), forward).normalize();
    const up = Vector3.Cross(forward, right).normalize();
    let minR = Infinity;
    let maxR = -Infinity;
    let minU = Infinity;
    let maxU = -Infinity;
    const p = new Vector3();
    for (const a of angles) {
      const c = Math.cos(a);
      const s = Math.sin(a);
      for (const v of corners) {
        // Babylon's rotation.y turns +x towards -z.
        p.set(v.x * c + v.z * s, v.y, -v.x * s + v.z * c);
        const r = Vector3.Dot(p, right);
        const u = Vector3.Dot(p, up);
        minR = Math.min(minR, r);
        maxR = Math.max(maxR, r);
        minU = Math.min(minU, u);
        maxU = Math.max(maxU, u);
      }
    }
    const half = Math.max(MIN_HALF, ((maxR - minR) / 2) * 1.14, ((maxU - minU) / 2) * 1.14);
    const centre = right.scale((minR + maxR) / 2).add(up.scale((minU + maxU) / 2));
    const cam = this.camera;
    cam.position.copyFrom(centre.add(dir.scale(12)));
    cam.setTarget(centre);
    cam.orthoLeft = -half;
    cam.orthoRight = half;
    cam.orthoBottom = -half;
    cam.orthoTop = half;
  }

  /** Waits until every mesh can be drawn (shaders compiled, textures loaded). */
  private async whenReady(meshes: Mesh[]): Promise<boolean> {
    const started = performance.now();
    while (performance.now() - started < READY_TIMEOUT) {
      if (meshes.every((m) => m.isReady(true)) && this.shadow.isReady(true)) return true;
      await new Promise((r) => setTimeout(r, 16));
    }
    return false;
  }

  /** Copies the frame just drawn (synchronously) and encodes it in the background. */
  private async capture(size: number): Promise<string> {
    const out = document.createElement('canvas');
    out.width = out.height = size;
    out.getContext('2d')!.drawImage(this.canvas, 0, 0, size, size, 0, 0, size, size);
    const blob = await new Promise<Blob>((resolve, reject) => out.toBlob((b) => (b ? resolve(b) : reject(new Error('toBlob failed'))), 'image/webp', 0.9));
    return URL.createObjectURL(blob);
  }
}
