/**
 * The landscape's data built off the main thread (`renderData.worker.ts`): the grass field
 * (`grassField`, a texel a metre over the land: seconds of work round a big world) and the
 * terrain's heights and colours (`LandscapeLayout.shadeTerrain`). The worker lays the land out
 * again from the same world (`LandscapeLayout`, deterministic: the same numbers as the page's)
 * and sends back typed arrays; the page only builds meshes and textures from them. Where no worker
 * runs (or one fails), each call says so and the caller builds on the page as before.
 */

import type { WorldStructure } from '../../core/protocol';
import { CountryRoads } from './countryRoads';
import { grassField, townGrass, type GrassField } from './grass';
import { LandscapeLayout } from './nature';

/**
 * What of a world the land is laid out from (`LandscapeLayout`, `CountryRoads`, the grass field's
 * land): its size, its lots and its meta (seed, landscape, roads, zones). The town's own contents
 * (rooms, walls, furniture: millions of tiles in a big world) stay on the page: the town's lawn
 * mask is made there (`townGrass`).
 */
export function landWorld(world: WorldStructure): WorldStructure {
  const { version, lotVersion, mode, width, depth, storeyDepth, plots, meta } = world;
  return {
    version,
    lotVersion,
    mode,
    width,
    depth,
    ...(storeyDepth === undefined ? {} : { storeyDepth }),
    plots,
    meta,
    objects: [],
    sims: [],
    households: [],
    exits: [],
    rooms: [],
    walls: [],
    openings: [],
    meshes: {} as WorldStructure['meshes'],
  };
}

/** The land part of a grass field: all of it but the town's mask (`townGrass` makes it, and the patches, on the page). */
export type LandGrass = Omit<GrassField, 'town' | 'townW' | 'townD' | 'patches'>;

/**
 * The land as the worker lays it out: `pc/nature.ts`'s `Landscape` without what it draws (the
 * lawn's density and the woods' squares change neither the ground nor its colours).
 */
export class Land extends LandscapeLayout {
  constructor(world: WorldStructure) {
    super(world, (land, w, name) => new CountryRoads(land, w, name));
  }

  /** The grass field's land part (`grassField`; its town mask is left to the page). */
  async grass(world: WorldStructure, margin: number): Promise<LandGrass> {
    const { x0, z0, margin: m, w, d, colour, pw, pd, ground } = await grassField(world, this, undefined, margin);
    return { x0, z0, margin: m, w, d, colour, pw, pd, ground };
  }

  /** The terrain's heights (into `positions`) and colours (`shadeTerrain`). */
  async shade(positions: Float32Array): Promise<Float32Array> {
    const colors = new Float32Array((positions.length / 3) * 4);
    await this.shadeTerrain(positions, colors);
    return colors;
  }
}

/** A grass field from its land part (the worker's) and the town's mask made here, as `grassField` makes it whole. */
export function withTownGrass(land: LandGrass, world: WorldStructure): GrassField {
  const field: GrassField = { ...land, townW: 0, townD: 0, town: new Uint8Array(0), patches: new Uint8Array(land.pw * land.pd) };
  townGrass(field, world);
  return field;
}

export type ToRenderData =
  | { type: 'open'; land: number; world: WorldStructure }
  | { type: 'close'; land: number }
  | { type: 'grass'; id: number; land: number; margin: number }
  | { type: 'shade'; id: number; land: number; positions: Float32Array };

export type FromRenderData =
  | { id: number; type: 'grass'; field: LandGrass }
  | { id: number; type: 'shade'; positions: Float32Array; colors: Float32Array }
  | { id: number; type: 'error'; message: string };

/** The worker goes once it has had nothing to do for this long. */
const IDLE_MS = 30_000;

let worker: Worker | null = null;
/** No worker can run here (or one failed): everything is built on the page. */
let broken = false;
let nextId = 1;
let nextLand = 1;
let open = 0;
const pending = new Map<number, (reply: FromRenderData | null) => void>();
let idleTimer: ReturnType<typeof setTimeout> | undefined;

function stop(): void {
  worker?.terminate();
  worker = null;
  clearTimeout(idleTimer);
  for (const answer of pending.values()) answer(null);
  pending.clear();
}

function start(): Worker | null {
  if (worker || broken) return worker;
  try {
    worker = new Worker(new URL('./renderData.worker.ts', import.meta.url), { type: 'module' });
  } catch {
    broken = true;
    return null;
  }
  worker.onmessage = (e: MessageEvent<FromRenderData>) => {
    const answer = pending.get(e.data.id);
    pending.delete(e.data.id);
    answer?.(e.data.type === 'error' ? null : e.data);
  };
  worker.onerror = (e) => {
    e.preventDefault();
    console.warn('[render] the render data worker failed; building on the page:', e.message);
    broken = true;
    stop();
  };
  return worker;
}

function ask(w: Worker, message: ToRenderData & { id: number }, transfer: Transferable[] = []): Promise<FromRenderData | null> {
  return new Promise((resolve) => {
    pending.set(message.id, resolve);
    w.postMessage(message, transfer);
  });
}

/**
 * A world's land in the worker, for one build (`close` when done). Each call resolves to null
 * when the worker can't give the answer: the caller builds it on the page.
 */
export class OffThreadLand {
  private readonly id = nextLand++;
  private closed = false;

  private constructor(private readonly w: Worker) {}

  /** The world's land sent to the worker; null where no worker can run. */
  static open(world: WorldStructure): OffThreadLand | null {
    const w = start();
    if (!w) return null;
    clearTimeout(idleTimer);
    open++;
    const land = new OffThreadLand(w);
    w.postMessage({ type: 'open', land: land.id, world: landWorld(world) } satisfies ToRenderData);
    return land;
  }

  /** The grass field's land part, `margin` m round the town (`grassField`). */
  async grass(margin: number): Promise<LandGrass | null> {
    if (this.closed || worker !== this.w) return null;
    const reply = await ask(this.w, { type: 'grass', id: nextId++, land: this.id, margin });
    return reply?.type === 'grass' ? reply.field : null;
  }

  /**
   * The terrain's heights and colours for `positions` (x, y, z: y set in place) into `colors`
   * (`LandscapeLayout.shadeTerrain`); false if they weren't made (nothing is changed then).
   */
  async shade(positions: Float32Array, colors: Float32Array): Promise<boolean> {
    if (this.closed || worker !== this.w) return false;
    // (A copy goes: the caller's arrays are its own until the answer is in.)
    const sent = positions.slice();
    const reply = await ask(this.w, { type: 'shade', id: nextId++, land: this.id, positions: sent }, [sent.buffer]);
    if (reply?.type !== 'shade' || reply.positions.length !== positions.length || reply.colors.length !== colors.length) return false;
    positions.set(reply.positions);
    colors.set(reply.colors);
    return true;
  }

  /** Done with this world: the worker forgets its land (and goes once idle). */
  close(): void {
    if (this.closed) return;
    this.closed = true;
    if (worker !== this.w) return;
    this.w.postMessage({ type: 'close', land: this.id } satisfies ToRenderData);
    if (--open > 0) return;
    clearTimeout(idleTimer);
    idleTimer = setTimeout(() => {
      if (!open && !pending.size) stop();
    }, IDLE_MS);
  }
}
