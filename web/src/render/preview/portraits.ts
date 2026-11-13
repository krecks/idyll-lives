/**
 * Head-and-shoulders portraits from the studio, cached by look.
 *
 * Requests are collected and drawn in batches: up to `BATCH` Sims stand in a row far off to the
 * side of the stage (rows 1..), held still with a friendly face and eye contact, all put there at
 * once (one build of the studio's residents a batch: posing them a couple a frame built them all
 * again each time, eight frames before the first picture). Once they are ready, a few portraits
 * are drawn per frame, each by a camera of its own into a multisampled target of its own (in the
 * studio's frame, `Studio.draw`), read back after it, un-premultiplied and encoded (WebP, PNG where
 * unsupported) into object URLs. A target per row lets the next few be drawn while the last ones
 * are still read back (they waited for them, a few frames each time).
 *
 * What's on screen comes first (`priority` `now`, in the order asked); the rest (`idle`: drawn ahead
 * of time, or a face scrolled out of view) fills the batches after it. A request whose asker has gone
 * (`signal`) is dropped if it isn't being drawn yet. The cache outlives the app, so the game asks for
 * each look once per page; `PortraitStore` keeps them between visits.
 */

import * as pc from '../pc/engine';
import { perfEnd, perfStart, type PerfEntry } from '../../debug/perf';
import { openCache } from '../../ml/assets';
import type { StageDirection } from '../shared/characters/expressions';
import { shadingFromSettings } from '../pc/shaders/softLook';
import type { SimLook } from '../types';
import { Studio } from './studio';

/** Portrait size in pixels (displayed at up to ~64 CSS px, so 2-3x for sharp HiDPI avatars). */
export const PORTRAIT_PX = 160;
/** Sims a batch stands in a row: about as many faces as step 2's list shows at once, so they come together. */
const BATCH = 32;
const PER_FRAME = 8;
/** Draws in a batch's first frame: those compile the portrait shaders, so they get their own task. */
const FIRST_DRAW = 1;
const FIRST_ROW = 1;
const ROW_X = 40;
const ROW_SPACING = 3;
/** Frames posed before drawing (heads settle; the camera and eye contact use them). */
const SETTLE_FRAMES = 3;
/** Give up on a batch whose materials never become ready (ms). */
const READY_TIMEOUT = 20_000;
const CACHE_MAX = 400;
const FOV = 0.3;
/** Camera a little to the side (towards the key light) and above the eyes. */
const ANGLE = 0.28;
/**
 * Framing for a Sim whose head top is at `REF_TOP` (m): half the height in view and how far below
 * the crown its centre is. Both scale with the Sim, so height doesn't change the picture.
 */
const FRAME_HALF = 0.22;
const FRAME_DROP = 0.176;
const REF_TOP = 1.78;

/** A friendly portrait face. */
const PORTRAIT_FACE = { smile: 0.6, wide: 0.3, lidL: 0.35, browUp: 0.15 };

/** `now`: on screen (drawn first, in the order asked); `idle`: drawn ahead of time, or out of view. */
export type PortraitPriority = 'now' | 'idle';

interface Job {
  key: string;
  look: SimLook;
  priority: PortraitPriority;
  waiters: ((url: string | null) => void)[];
  /** Draws that came back empty (a shader still compiling for the render target). */
  misses: number;
}
/** Retries for a portrait that came back empty. */
const MAX_MISSES = 3;

interface Batch {
  jobs: Job[];
  started: number;
  frames: number;
  next: number;
  pending: number;
  /** Its rows stand (`direct`, once). */
  posed: boolean;
  cams: { x: number; y: number; z: number }[];
  /** Its span (`debug/perf.ts`): from its start to its last portrait read back. */
  span: PerfEntry;
}

/** Cache key: everything that changes the picture (the look's own face is in its appearance), the shading included (not the height: framing follows the head; not the voice). */
export function lookKey(look: SimLook): string {
  const sorted = Object.entries(look.appearance)
    .filter(([k]) => k !== 'height' && k !== 'voice')
    .sort(([a], [b]) => (a < b ? -1 : 1));
  // Without garments the outfit comes from the Sim's id, and without a face of its own the face does (older saves).
  const legacy = !look.appearance.top || !look.appearance.face;
  return JSON.stringify([look.gender, sorted, legacy ? (look.id ?? 0) : null, shadingFromSettings()]);
}

/** Portrait URLs by look key, least recently used first (survives the app). */
const cache = new Map<string, string>();

export function cached(key: string): string | null {
  const url = cache.get(key);
  if (url === undefined) return null;
  cache.delete(key);
  cache.set(key, url);
  return url;
}

function remember(key: string, url: string): void {
  cache.set(key, url);
  while (cache.size > CACHE_MAX) {
    const [old, oldUrl] = cache.entries().next().value!;
    cache.delete(old);
    URL.revokeObjectURL(oldUrl);
  }
}

export class Portraits {
  /** `drawn`: each portrait drawn, as encoded (to keep between visits). */
  constructor(private readonly drawn?: (key: string, blob: Blob) => void) {}

  private readonly queue = new Map<string, Job>();
  private batch: Batch | null = null;
  /** A camera and its target per portrait drawn in a frame. */
  private slots: { camera: pc.CameraComponent; target: pc.RenderTarget }[] = [];
  /** The portraits this frame draws (`collect` reads them back). */
  private drawing: { job: Job; target: pc.RenderTarget }[] = [];
  private drawingBatch: Batch | null = null;
  private readonly target = { x: 0, y: 0, z: 0 };

  /**
   * The portrait of `look` (drawn in a coming batch). `signal`: the asker has gone (its face left the
   * page, or shows another look now): it gets null, and the look isn't drawn if nobody else waits for
   * it and its batch hasn't started.
   */
  request(key: string, look: SimLook, priority: PortraitPriority = 'now', signal?: AbortSignal): Promise<string | null> {
    if (signal?.aborted) return Promise.resolve(null);
    return new Promise((resolve) => {
      const inBatch = this.batch?.jobs.find((j) => j.key === key);
      let job = inBatch ?? this.queue.get(key);
      if (job) {
        job.waiters.push(resolve);
        if (priority === 'now') job.priority = 'now';
      } else {
        job = { key, look: { gender: look.gender, appearance: JSON.parse(JSON.stringify(look.appearance)), id: look.id }, priority, waiters: [resolve], misses: 0 };
        this.queue.set(key, job);
      }
      const asked = job;
      signal?.addEventListener(
        'abort',
        () => {
          asked.waiters = asked.waiters.filter((w) => w !== resolve);
          resolve(null);
          if (!asked.waiters.length && this.queue.get(key) === asked) this.queue.delete(key);
        },
        { once: true },
      );
    });
  }

  get busy(): boolean {
    return this.queue.size > 0 || this.batch !== null;
  }

  /** Fails every request (the studio is unavailable). */
  failAll(): void {
    for (const job of [...this.queue.values(), ...(this.batch?.jobs ?? [])]) for (const w of job.waiters) w(null);
    this.queue.clear();
    this.batch = null;
  }

  /** Before posing: starts the next batch, and points the portrait Sims' eyes at their cameras. */
  direct(studio: Studio, now: number): void {
    if (!this.batch && this.queue.size) this.startBatch(now);
    const b = this.batch;
    if (!b) return;
    if (!b.posed) this.pose(studio, b);
    const heads = studio.characters.heads;
    b.jobs.forEach((_, j) => {
      const row = FIRST_ROW + j;
      const cam = b.cams[j];
      const top = studio.characters.visible[row] ? heads[row * 3 + 1] : 1.75;
      const x = ROW_X + j * ROW_SPACING;
      // Frame: from the collarbones to just above the crown.
      const k = top / REF_TOP;
      const ty = top - FRAME_DROP * k;
      const dist = (FRAME_HALF * k) / Math.tan(FOV / 2);
      cam.x = x + Math.sin(ANGLE) * dist;
      cam.y = ty + 0.06;
      cam.z = Math.cos(ANGLE) * dist;
    });
  }

  /** After posing: turns on a camera for each portrait ready to draw in this frame (`Studio.draw`); whether any is. */
  prepare(studio: Studio, now: number): boolean {
    const b = this.batch;
    if (!b || !b.posed || b.next >= b.jobs.length) return false;
    if (!studio.ready()) {
      if (now - b.started > READY_TIMEOUT) this.finish(studio, true);
      return false;
    }
    if (++b.frames < SETTLE_FRAMES) return false;
    // The batch's first draws compile the portrait shaders: one a frame then, not eight in one task (~850 ms at boot).
    const limit = b.next === 0 ? FIRST_DRAW : PER_FRAME;
    while (b.next < b.jobs.length && this.drawing.length < limit) {
      const j = b.next++;
      // (A target per row: the ones drawn last frame may still be being read back.)
      const slot = this.slot(studio, j);
      const top = studio.characters.heads[(FIRST_ROW + j) * 3 + 1];
      this.target.x = ROW_X + j * ROW_SPACING;
      this.target.y = top - FRAME_DROP * (top / REF_TOP);
      Studio.aim(slot.camera, b.cams[j], this.target);
      slot.camera.entity.enabled = true;
      this.drawing.push({ job: b.jobs[j], target: slot.target });
    }
    this.drawingBatch = b;
    b.pending += this.drawing.length;
    return this.drawing.length > 0;
  }

  /**
   * After the frame: reads back what `prepare` drew, and turns its cameras off. Each read is sent at
   * once (`immediate`: on WebGPU the copy would otherwise wait in an encoder for the next frame, and
   * its buffer be mapped before, empty). From the resolved texture itself (a read through the
   * multisampled target is refused on WebGL2). The rows come back bottom-up on WebGL2
   * (`readPixels`), top-down on WebGPU.
   */
  collect(studio: Studio): void {
    const drawn = this.drawing;
    const b = this.drawingBatch;
    if (!drawn.length || !b) return;
    this.drawing = [];
    this.drawingBatch = null;
    for (const slot of this.slots) slot.camera.entity.enabled = false;
    const bottomUp = !studio.webgpu;
    for (const { job, target } of drawn) {
      void (async () => {
        let url: string | null = null;
        let empty = false;
        try {
          const pixels = (await target.colorBuffer.read(0, 0, PORTRAIT_PX, PORTRAIT_PX, { immediate: true })) as Uint8Array | null;
          if (pixels && coverage(pixels) > 0.02) {
            const blob = await encode(pixels, PORTRAIT_PX, PORTRAIT_PX, bottomUp);
            url = URL.createObjectURL(blob);
            this.drawn?.(job.key, blob);
          } else empty = true;
        } catch (err) {
          console.warn('[preview] portrait failed', err);
        }
        if (empty && ++job.misses < MAX_MISSES && !this.queue.has(job.key)) {
          // Drawn empty (its shader not ready for the target yet): draw it again.
          this.queue.set(job.key, job);
        } else {
          if (url) remember(job.key, url);
          for (const w of job.waiters) w(url);
          job.waiters = [];
        }
        if (this.batch === b && --b.pending === 0 && b.next >= b.jobs.length) this.finish(studio, false);
      })();
    }
  }

  /** Lets go of the cameras and targets (the studio goes: its app with them). */
  dispose(): void {
    for (const { camera, target } of this.slots) {
      camera.entity.destroy();
      target.destroyTextureBuffers();
      target.destroy();
    }
    this.slots = [];
    this.drawing = [];
    this.drawingBatch = null;
    this.failAll();
  }

  private startBatch(now: number): void {
    const queued = [...this.queue.values()];
    const jobs = [...queued.filter((j) => j.priority === 'now'), ...queued.filter((j) => j.priority !== 'now')].slice(0, BATCH);
    for (const job of jobs) this.queue.delete(job.key);
    const cams = jobs.map(() => ({ x: 0, y: 0, z: 0 }));
    this.batch = { jobs, started: now, frames: 0, next: 0, pending: 0, posed: false, cams, span: perfStart('portraits: batch') };
  }

  /** Puts the batch's Sims on their rows (built together), still, with a friendly face and eye contact. */
  private pose(studio: Studio, b: Batch): void {
    b.posed = true;
    studio.setRows(b.jobs.map((job, j) => [FIRST_ROW + j, { look: job.look, x: ROW_X + j * ROW_SPACING, z: 0, yaw: 0 }]));
    b.jobs.forEach((_, j) => {
      const dir: StageDirection = { still: true, time: 0.6, face: PORTRAIT_FACE, lookAt: b.cams[j], hands: 0.6 };
      studio.characters.directions[FIRST_ROW + j] = dir;
    });
  }

  /** Ends the batch and empties its rows; on failure, unanswered requests get null. */
  private finish(studio: Studio, failed: boolean): void {
    const b = this.batch;
    if (!b) return;
    if (failed) for (const job of b.jobs) for (const w of job.waiters.splice(0)) w(null);
    perfEnd(b.span, `${b.jobs.length} looks${failed ? ', failed' : ''}`);
    b.jobs.forEach((_, j) => (studio.characters.directions[FIRST_ROW + j] = null));
    studio.setRows(b.jobs.map((_, j) => [FIRST_ROW + j, null]));
    this.batch = null;
  }

  /** Portrait camera `i` of a frame, drawing into its own multisampled target (made on first use). */
  private slot(studio: Studio, i: number): { camera: pc.CameraComponent; target: pc.RenderTarget } {
    let slot = this.slots[i];
    if (!slot) {
      const texture = new pc.Texture(studio.device, {
        name: `portrait${i}`,
        width: PORTRAIT_PX,
        height: PORTRAIT_PX,
        format: pc.PIXELFORMAT_RGBA8,
        mipmaps: false,
        minFilter: pc.FILTER_LINEAR,
        magFilter: pc.FILTER_LINEAR,
        addressU: pc.ADDRESS_CLAMP_TO_EDGE,
        addressV: pc.ADDRESS_CLAMP_TO_EDGE,
      });
      const target = new pc.RenderTarget({ name: `portrait${i}`, colorBuffer: texture, depth: true, samples: 4 });
      // (Before the stage's camera, which draws onto the canvas.)
      const camera = studio.camera(`portraitCamera${i}`, FOV, 0.2, 4, -BATCH + i);
      camera.renderTarget = target;
      this.slots[i] = slot = { camera, target };
    }
    return slot;
  }
}

/** Fraction of opaque pixels. */
function coverage(data: Uint8Array): number {
  let n = 0;
  for (let k = 3; k < data.length; k += 16) if (data[k] > 128) n++;
  return (n * 4) / (data.length / 4);
}

/** RGBA (premultiplied by the MSAA resolve over a transparent clear) to an image. */
async function encode(data: Uint8Array, w: number, h: number, flipY: boolean): Promise<Blob> {
  const img = new ImageData(w, h);
  const out = img.data;
  for (let y = 0; y < h; y++) {
    const src = (flipY ? h - 1 - y : y) * w * 4;
    const dst = y * w * 4;
    for (let x = 0; x < w * 4; x += 4) {
      const a = data[src + x + 3];
      if (a === 0) continue;
      const k = a === 255 ? 1 : 255 / a;
      out[dst + x] = Math.min(255, data[src + x] * k);
      out[dst + x + 1] = Math.min(255, data[src + x + 1] * k);
      out[dst + x + 2] = Math.min(255, data[src + x + 2] * k);
      out[dst + x + 3] = a;
    }
  }
  let blob: Blob;
  if (typeof OffscreenCanvas !== 'undefined') {
    const canvas = new OffscreenCanvas(w, h);
    canvas.getContext('2d')!.putImageData(img, 0, 0);
    blob = await canvas.convertToBlob({ type: 'image/webp', quality: 0.92 });
  } else {
    const canvas = document.createElement('canvas');
    canvas.width = w;
    canvas.height = h;
    canvas.getContext('2d')!.putImageData(img, 0, 0);
    blob = await new Promise<Blob>((resolve, reject) => canvas.toBlob((b) => (b ? resolve(b) : reject(new Error('toBlob failed'))), 'image/webp', 0.92));
  }
  return blob;
}

/**
 * Portraits kept between visits (Cache Storage), as the catalog's pictures are (`items.ts`): each
 * under its look's key (`lookKey`: the look and the shading) and `version`, what the faces are drawn
 * from (the character files' contents; the app's version where the build has no content hashes).
 * Other caches of the prefix (older schemes) are deleted when it opens. Bump `v1` after a change to
 * how portraits are drawn (the studio's light, the framing, the face).
 */
const STORE_PREFIX = 'littlelives-portraits-';
const STORE = `${STORE_PREFIX}v1`;

export class PortraitStore {
  private store: Promise<Cache | null> | null = null;

  constructor(private readonly version: string) {}

  /** The kept portrait of `key` as an object URL (remembered for the page), or null. */
  async get(key: string): Promise<string | null> {
    try {
      const res = await (await this.cache())?.match(this.url(key));
      if (!res) return null;
      const url = URL.createObjectURL(await res.blob());
      remember(key, url);
      return url;
    } catch {
      return null;
    }
  }

  put(key: string, blob: Blob): void {
    void (async () => {
      try {
        await (await this.cache())?.put(this.url(key), new Response(blob, { headers: { 'Content-Type': blob.type || 'image/webp' } }));
      } catch {
        // Storage full or unavailable: portraits are drawn again next time.
      }
    })();
  }

  private cache(): Promise<Cache | null> {
    this.store ??= typeof caches === 'undefined' ? Promise.resolve(null) : openCache(STORE_PREFIX, STORE);
    return this.store;
  }

  private url(key: string): string {
    return `${location.origin}/__portraits/${this.version}/${PORTRAIT_PX}/${hash(key)}.webp`;
  }
}

/** FNV-1a, as hex (two rounds with different seeds: a look key is long, and two faces must not share a picture). */
function hash(text: string): string {
  let a = 0x811c9dc5;
  let b = 0x01000193;
  for (let i = 0; i < text.length; i++) {
    const c = text.charCodeAt(i);
    a = Math.imul(a ^ c, 0x01000193);
    b = Math.imul(b ^ c, 0x5bd1e995);
  }
  return (a >>> 0).toString(16).padStart(8, '0') + (b >>> 0).toString(16).padStart(8, '0');
}
