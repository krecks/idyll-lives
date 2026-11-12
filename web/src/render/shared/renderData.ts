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
import { LazyWorker } from '../../core/lazyWorker';

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
export type LandGrass = Omit<GrassField, 'town' | 'townW' | 'townD' | 'shade' | 'lawn' | 'patches'>;

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
  const field: GrassField = { ...land, townW: 0, townD: 0, town: new Uint8Array(0), shade: new Uint8Array(0), lawn: new Uint8Array(0), patches: new Uint8Array(land.pw * land.pd) };
  townGrass(field, world);
  return field;
}

export type ToRenderData =
  | { type: 'open'; land: number; world: WorldStructure }
  | { type: 'close'; land: number }
  | { type: 'grass'; land: number; margin: number }
  | { type: 'shade'; land: number; positions: Float32Array };

export type RenderDataAnswer = { type: 'grass'; field: LandGrass } | { type: 'shade'; positions: Float32Array; colors: Float32Array };

/** The render data worker (`renderData.worker.ts`); it goes once it has had nothing to do for half a minute. */
const data = new LazyWorker<ToRenderData, RenderDataAnswer>({
  start: () => new Worker(new URL('./renderData.worker.ts', import.meta.url), { type: 'module' }),
  idleMs: 30_000,
  broke: (why) => console.warn('[render] the render data worker failed; building on the page:', why),
});

let nextLand = 1;
/** No answer from the worker: the caller builds it on the page. */
const none = () => Promise.resolve(null);

/**
 * A world's land in the worker, for one build (`close` when done; the worker stays meanwhile). Each
 * call resolves to null when the worker can't give the answer: the caller builds it on the page.
 */
export class OffThreadLand {
  private closed = false;

  private constructor(
    private readonly id: number,
    /** The worker it was sent to (`LazyWorker.run`): another (after one broke) doesn't have it. */
    private readonly run: number,
    private readonly release: () => void,
  ) {}

  /** The world's land sent to the worker; null where no worker can run. */
  static open(world: WorldStructure): OffThreadLand | null {
    const id = nextLand++;
    const run = data.post({ type: 'open', land: id, world: landWorld(world) });
    return run ? new OffThreadLand(id, run, data.hold()) : null;
  }

  private get there(): boolean {
    return !this.closed && data.run === this.run;
  }

  /** The grass field's land part, `margin` m round the town (`grassField`). */
  async grass(margin: number): Promise<LandGrass | null> {
    if (!this.there) return null;
    const reply = await data.ask({ type: 'grass', land: this.id, margin }, none).catch(none);
    return reply?.type === 'grass' ? reply.field : null;
  }

  /**
   * The terrain's heights and colours for the grid `positions` (x, y, z: y set;
   * `LandscapeLayout.shadeTerrain`). `positions` goes over to the worker (the caller's copy is
   * empty after): the arrays come back with the answer.
   */
  async shade(positions: Float32Array): Promise<{ positions: Float32Array; colors: Float32Array } | null> {
    if (!this.there) return null;
    const reply = await data.ask({ type: 'shade', land: this.id, positions }, none, { transfer: [positions.buffer] }).catch(none);
    return reply?.type === 'shade' ? reply : null;
  }

  /** Done with this world: the worker forgets its land (and goes once idle). */
  close(): void {
    if (this.closed) return;
    if (this.there) data.post({ type: 'close', land: this.id });
    this.closed = true;
    this.release();
  }
}
