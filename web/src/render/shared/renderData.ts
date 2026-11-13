/**
 * The landscape's data built off the main thread (`renderData.worker.ts`): the grass field
 * (`grassField`, a texel a metre over the land: seconds of work round a big world), the terrain's
 * heights and colours (`LandscapeLayout.shadeTerrain`), and the town's road mesh (`roadMeshes`,
 * the menu overview's streets). The worker lays the land out again from the same world
 * (`LandscapeLayout`, deterministic: the same numbers as the page's) and sends back typed arrays;
 * the page only builds meshes and textures from them. Where no worker runs (or one fails), each
 * call says so and the caller builds on the page as before.
 */

import type { WorldStructure } from '../../core/protocol';
import { CountryRoads } from './countryRoads';
import { grassField, townGrass, type GrassField } from './grass';
import { LandscapeLayout } from './nature';
import type { RoadMeshData } from './roads';
import { LazyWorker } from '../../core/lazyWorker';

/**
 * What of a world the land is laid out from (`LandscapeLayout`, `CountryRoads`, the grass field's
 * land, the town's roads): its size, its lots, its meta (seed, landscape, roads, zones) and its
 * junctions (the road mesh's pads and markings). The town's own contents (rooms, walls, furniture:
 * millions of tiles in a big world) stay on the page: the town's lawn mask is made there
 * (`townGrass`).
 */
export function landWorld(world: WorldStructure): WorldStructure {
  const { version, lotVersion, mode, width, depth, storeyDepth, plots, meta, junctions } = world;
  return {
    version,
    lotVersion,
    mode,
    width,
    depth,
    ...(storeyDepth === undefined ? {} : { storeyDepth }),
    plots,
    meta,
    ...(junctions === undefined ? {} : { junctions }),
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
  | { type: 'roads'; world: WorldStructure }
  | { type: 'shade'; land: number; positions: Float32Array };

export type RenderDataAnswer =
  | { type: 'grass'; field: LandGrass }
  | { type: 'roads'; roads: RoadMeshData }
  | { type: 'shade'; positions: Float32Array; colors: Float32Array };

/** A render data worker (`renderData.worker.ts`); it goes once it has had nothing to do for half a minute. */
const dataWorker = () =>
  new LazyWorker<ToRenderData, RenderDataAnswer>({
    start: () => new Worker(new URL('./renderData.worker.ts', import.meta.url), { type: 'module' }),
    idleMs: 30_000,
    broke: (why) => console.warn('[render] the render data worker failed; building on the page:', why),
  });
/**
 * Two of them: one shades the terrain, the other makes the grass field (a second or more round a big
 * world: in one worker the terrain's tiles waited behind it, or it behind them). Each lays the land out.
 */
const shading = dataWorker();
const grassing = dataWorker();

let nextLand = 1;
/** No answer from the worker: the caller builds it on the page. */
const none = () => Promise.resolve(null);

/** One of the workers with a world's land open in it: the `run` it was sent to (another, after one broke, doesn't have it). */
class Opened {
  private constructor(
    private readonly worker: LazyWorker<ToRenderData, RenderDataAnswer>,
    private readonly run: number,
    private readonly release: () => void,
  ) {}

  static in(worker: LazyWorker<ToRenderData, RenderDataAnswer>, id: number, world: WorldStructure): Opened | null {
    const run = worker.post({ type: 'open', land: id, world });
    return run ? new Opened(worker, run, worker.hold()) : null;
  }

  get there(): boolean {
    return this.worker.run === this.run;
  }

  ask(message: ToRenderData, transfer: Transferable[] = []): Promise<RenderDataAnswer | null> {
    return this.worker.ask(message, none, { transfer }).catch(none);
  }

  close(id: number): void {
    if (this.there) this.worker.post({ type: 'close', land: id });
    this.release();
  }
}

/**
 * A world's land in the workers, for one build (`close` when done; the workers stay meanwhile). Each
 * call resolves to null when a worker can't give the answer: the caller builds it on the page.
 */
export class OffThreadLand {
  private closed = false;

  private constructor(
    private readonly id: number,
    private readonly shading: Opened,
    private readonly grassing: Opened | null,
  ) {}

  /** The world's land sent to the workers; null where no worker can run. */
  static open(world: WorldStructure): OffThreadLand | null {
    const id = nextLand++;
    const land = landWorld(world);
    const shade = Opened.in(shading, id, land);
    return shade ? new OffThreadLand(id, shade, Opened.in(grassing, id, land)) : null;
  }

  /** The grass field's land part, `margin` m round the town (`grassField`). */
  async grass(margin: number): Promise<LandGrass | null> {
    // (In the shading worker where a second one can't run.)
    const there = this.grassing?.there ? this.grassing : this.shading;
    if (this.closed || !there.there) return null;
    const reply = await there.ask({ type: 'grass', land: this.id, margin });
    return reply?.type === 'grass' ? reply.field : null;
  }

  /**
   * The terrain's heights and colours for the grid `positions` (x, y, z: y set;
   * `LandscapeLayout.shadeTerrain`). `positions` goes over to the worker (the caller's copy is
   * empty after): the arrays come back with the answer.
   */
  async shade(positions: Float32Array): Promise<{ positions: Float32Array; colors: Float32Array } | null> {
    if (this.closed || !this.shading.there) return null;
    const reply = await this.shading.ask({ type: 'shade', land: this.id, positions }, [positions.buffer]);
    return reply?.type === 'shade' ? reply : null;
  }

  /** Done with this world: the workers forget its land (and go once idle). */
  close(): void {
    if (this.closed) return;
    this.shading.close(this.id);
    this.grassing?.close(this.id);
    this.closed = true;
  }
}

/**
 * The town's road mesh (`roads.ts` `townRoadMesh`), made in the shading worker from the world's land
 * part while the page plans the street (the same arrays as the page's, in the same order). Null where
 * no worker could make it: the caller builds it on the page.
 */
export async function townRoadMeshOffThread(world: WorldStructure): Promise<RoadMeshData | null> {
  const reply = await shading.ask({ type: 'roads', world: landWorld(world) }, none).catch(none);
  return reply?.type === 'roads' ? reply.roads : null;
}
