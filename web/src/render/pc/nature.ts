/**
 * The landscape round the town on PlayCanvas: terrain with rolling hills, the sea or a lake, the
 * country roads (drawn by whoever is handed them, `buildRoads`), forests, lawn details, and the sky
 * dome with its band of clouds. Everything is static and instanced (`instancing.ts`: a draw call
 * per model type, and in the game per area: the groups of `shared/nature.ts`), generated
 * deterministically from the town's seed by the engine-neutral layout (`LandscapeLayout`), so a
 * town always grows the same plants in the same places.
 *
 * The lawn's grass is built on the GPU (`grass.ts`), drawn near the eye; past it the ground
 * carries the lawn, its gusts too (`grassGround`), and on past the town into the landscape.
 *
 * Trees, bushes, hedges, the lawn's flowers and the garden catalog's plants are Blender-built files
 * (tools/art/blender/families/plants.py, trees.py): crowns on the leafy `foliage` finish over `bark`
 * limbs, each tree and shrub with a far version for the distant woods. `installNature` gives those
 * two finishes the wind (`NATURE_WIND`, the port of `NatureWindPlugin`) and the foliage its
 * per-instance tint.
 *
 * Draws are entities under the world root (game space), made switched off: their owner shows them
 * (`lotContents.ts`'s landscapes, shared between the game and the overview), digs them away over a
 * basement and gives the woods near the town their shadows.
 */

import { daylightByDefault } from './daylight';
import * as pc from './engine';
import { withoutContact } from './post';
import { Grass, grassGround, type GrassPusher } from './grass';
import { disposeTiles, type GroundTile } from './groundTiles';
import { Instances, INSTANCE_COLOR } from './instancing';
import { gpuOnly, meshFrom } from './meshes';
import { GAME_FRONT_FACE, MaterialLibrary, textureWithFallback } from './materials';
import { buildModel, disposeTemplate, type ModelMesh, type ModelTemplate } from './models';
import { codeFor, float, registerShaderFeature, type ShaderFeature } from './shaders/chunks';
import type { AssetRegistry } from '../../assets/registry';
import type { WorldStructure } from '../../core/protocol';
import { LookedFrom } from '../shared/chunks';
import { Geo } from '../shared/geometry';
import { CountryRoads, ROAD_VERGE } from '../shared/countryRoads';
import type { Lighting } from '../shared/environment';
import { GRASS, grassField, gustAt, gustNoise, markGrassPatches, recolourGrass, regroundGrass, type GrassQuality } from '../shared/grass';
import { OffThreadLand, withTownGrass } from '../shared/renderData';
import { pondQuads, rectBox, ROUND_REACH } from '../shared/landBake';
import { flowingWater, WaterFlowMap } from './water';
import { padGeometry, padMask, padsOf } from '../shared/pads';
import { LAND_CELL, type Land, type LandRect } from '../../game/land';
import { toLinear } from '../shared/math';
import { computeNormals, sphereData } from '../shared/modelParts';
import { GROUND_TILE, groupInstances, hideLawn, keepBareTiles, LandscapeLayout, lawnInputs, lawnLayer, lawnMask, sameLawnInputs, smoothstep, SPLIT_TRIANGLES, WIND, WIND_KIND, windSwayAt, type LawnInputs, type LawnLayer, type LawnMask, type LawnPlacement, type Woods } from '../shared/nature';
import type { OpenWorldQuality } from '../quality';
import type { BenchDiag } from '../../debug/benchFlags';
import type { TownBuildStep } from '../types';
import { tuned } from '../tune';
import { both } from '../../core/async';
import { perfEnd, perfStart, perfTime, perfTimeAsync } from '../../debug/perf';

export { decorPlants, natureDecor, type PlantPlacement } from '../shared/nature';

/** What the landscape draws: entities under the world root, switched off until their owner shows them. */
export interface NatureResult {
  draws: pc.Entity[];
  /**
   * Draws that should cast shadows: the woods near the town, whole in the game too (its shadow map
   * round the viewed lot would draw each group reaching into it, a draw each).
   */
  casters: pc.Entity[];
  /** Plants drawn only in the game (in groups) or only in the menus' town (whole); the rest are drawn in both. */
  only: Map<pc.Entity, 'game' | 'town'>;
}

/**
 * A landscape's plants built again (`Landscape.rebuild`): the new draws, the ones they replace, the
 * swap (`commit`: the replaced ones and their templates go), or the new ones dropped (`discard`).
 */
export interface NatureRebuild extends NatureResult {
  replaced: pc.Entity[];
  commit: () => void;
  discard: () => void;
}

/**
 * The landscape's plants by kind (`Landscape.parts`, for `?diag=nowoods,nofarwoods,noflowers,nograss`):
 * the woods near the town (the full models), the woods further out (their far versions), the lawn's
 * flowers and its grass.
 */
export type NaturePart = 'woods' | 'farWoods' | 'flowers' | 'grass';

/** The `?diag=` names that leave out each part. */
export const NATURE_DIAGS = [
  ['nowoods', 'woods'],
  ['nofarwoods', 'farWoods'],
  ['noflowers', 'flowers'],
  ['nograss', 'grass'],
] as const satisfies readonly (readonly [BenchDiag, NaturePart])[];

/** Draws the country roads (`pc/roads.ts`): their entities, under `parent`. */
export type RoadBuilder = (roads: CountryRoads, parent: pc.Entity) => pc.Entity[];

/** A part of the landscape built (`Landscape.build`'s `onStep`). */
export type NatureStep = Exclude<TownBuildStep, 'town'>;

// ---- Landscape ----------------------------------------------------------------------

/** The landscape as PlayCanvas draws it (its layout and growth: `shared/nature.ts`). */
export class Landscape extends LandscapeLayout<CountryRoads> {
  /** Lawn flowers, whole and in groups. */
  private lawn: LawnLayer<Instances>[] = [];
  /** The lawn's grass (built with the rest; none before). */
  private grass: Grass | null = null;
  /** What the lawn was last fitted to (`refreshLawn`). */
  private lawnInputs: LawnInputs | null = null;
  /** What `refreshLawn` last hid, for the lawn built again since (`rebuild`). */
  private lawnMask: LawnMask | null = null;
  /** Each model's plants as built, to build again when a file they were built without comes (`rebuild`). */
  private batches: Batch[] = [];
  /** Its other draws (the terrain, the water, the roads). */
  private others: pc.Entity[] = [];
  /** A big world's far woods as crown clusters, shown in the town view past the woods drawn in full (`FarCrowns`). */
  private crowns: FarCrowns | null = null;
  /** How the woods' squares are named (`Woods.sector`), as they were last laid out. */
  private woodsSector: ((x: number, z: number) => string) | null = null;
  /** The water's material and the library that adopted it (its own: it goes with the landscape). */
  private waterMaterial: { material: pc.StandardMaterial; lib: MaterialLibrary } | null = null;
  /** The water's flow map (`pc/water.ts`), made with its material (none: `?diag=noflow`). */
  private flowMap: WaterFlowMap | null = null;

  /** The land in the render data worker while `build` runs: the terrain's shading and the grass field are made there. */
  private offThread: OffThreadLand | null = null;
  /** The world it was built for (as last fitted, `refreshLawn`): its lawns grown again from it (`regrow`). */
  private world: WorldStructure | null = null;
  /** The world it was built for (as last fitted), null before a build. */
  get builtFor(): WorldStructure | null {
    return this.world;
  }

  /** The app and the entity its draws hang under (`build`'s). */
  private host: { app: pc.AppBase; parent: pc.Entity } | null = null;
  /** The water sheet at the sea's level (`water`), while it is drawn. */
  private sheet: pc.Entity | null = null;
  /** A height field's water above the sea's level, by ground tile (`pondQuads`). */
  private readonly ponds = new Map<number, pc.Entity>();
  /**
   * Its draws changed outside a build (a height field's water came or went: `landChanged`, `redrawLand`): the
   * owner shows the `added` (switched off, as a build's) and lets go of the `removed` (destroyed after it's told).
   */
  onDraws: ((added: pc.Entity[], removed: pc.Entity[]) => void) | null = null;

  /** `grassQuality`: how the grass is drawn (the preset's, `QualitySettings.grass`); `open`: a big world's distances (`QualitySettings.openWorld`). */
  constructor(world: WorldStructure, lawnDensity: number, lawnLod: 0 | 1, woodsCell: number, private readonly grassQuality: GrassQuality, private readonly open: OpenWorldQuality) {
    super(world, (land, w, name) => new CountryRoads(land, w, name), lawnDensity, lawnLod, woodsCell, open.woodsCell);
  }

  /** `onStep`: told as its ground, its woods and lawns and its grass are each built. */
  async build(app: pc.AppBase, parent: pc.Entity, assets: AssetRegistry, world: WorldStructure, pace?: () => Promise<void>, buildRoads?: RoadBuilder, onStep?: (done: NatureStep) => void): Promise<NatureResult> {
    this.world = world;
    this.host = { app, parent };
    this.offThread = OffThreadLand.open(world);
    try {
      return await this.buildAll(app, parent, assets, world, pace, buildRoads, onStep);
    } finally {
      this.offThread?.close();
      this.offThread = null;
    }
  }

  private async buildAll(app: pc.AppBase, parent: pc.Entity, assets: AssetRegistry, world: WorldStructure, pace?: () => Promise<void>, buildRoads?: RoadBuilder, onStep?: (done: NatureStep) => void): Promise<NatureResult> {
    const draws: pc.Entity[] = [];
    // (The grass field's land is made in a worker of its own meanwhile, its town mask here after the
    // plants: the page has the terrain's meshes and the plants to make first.)
    const grassLand = this.offThread?.grass(this.ring) ?? Promise.resolve(null);
    this.batches = [];
    // A big world's ground in 128 m tiles of three levels of detail; a town of today's sizes one mesh, as ever.
    const ground =
      world.meta?.world?.size === 'big'
        ? perfTimeAsync('nature: terrain tiles', () => this.terrainTiles(app, parent, pace), `${Math.ceil(this.terrainGrid().size / TILE) ** 2} tiles`)
        : perfTimeAsync('nature: terrain', () => this.terrain(app, parent, pace)).then((e) => [e]);
    void ground.then(() => onStep?.('hills'), () => {});
    // The woods and the lawn's flowers meanwhile: the terrain waits on the worker's shading for much of its time.
    const plants = (async () => {
      await perfTimeAsync('nature: forests', () => this.forests(app, parent, assets, pace));
      await perfTimeAsync('nature: lawns', () => this.plantLawns(app, parent, assets, world, pace));
      onStep?.('woods');
    })();
    const [grounds] = await both(ground, plants);
    draws.push(...grounds);
    // (On a land the town follows, each lot's pad a slab of its own, with its retaining edges: `shared/pads.ts`.)
    const pads = this.surface ? perfTime('nature: pads', () => this.padDraws(app, parent, grounds)) : { slabs: [], edges: [] };
    draws.push(...pads.slabs, ...pads.edges);
    this.grounds = grounds;
    this.pads = pads;
    if (this.seaSheet) draws.push((this.sheet = perfTime('nature: water', () => this.water(app, parent))));
    // (A height field's water above the sea's level: its own, tile by tile.)
    if (this.heightField) for (let k = 0; k < this.tiles.length; k++) draws.push(...this.pond(k));
    this.roadBuilder = buildRoads ?? null;
    this.roadDraws = buildRoads ? perfTime('nature: roads', () => buildRoads(this.roads, parent)) : [];
    draws.push(...this.roadDraws);
    for (const e of draws) e.enabled = false;
    this.others = draws;
    this.findSquares();
    const field = await perfTimeAsync('nature: grass field', async () => {
      const land = await grassLand;
      return land ? withTownGrass(land, world) : grassField(world, this, pace, this.ring);
    });
    const grass = (this.grass = new Grass(app, parent, field, this.grassQuality));
    onStep?.('grass');
    grass.onGround([...grounds, ...pads.slabs].map((g) => g.render!.meshInstances[0]));
    const built = gather(this.batches);
    return { draws: [...draws, ...built.draws, ...grass.draws.map((d) => d.entity)], casters: built.casters, only: built.only };
  }

  /**
   * The terrain's heights and colours for the grid `grid` lays out (x and z; y is set): made in the
   * worker while it runs (the same numbers; the grid goes over to it and comes back), else here.
   */
  private async shaded(grid: () => Float32Array, pace?: () => Promise<void>): Promise<{ positions: Float32Array; colors: Float32Array }> {
    const made = await this.offThread?.shade(grid());
    if (made) return made;
    const positions = grid();
    const colors = new Float32Array((positions.length / 3) * 4);
    await this.shadeTerrain(positions, colors, pace);
    return { positions, colors };
  }

  /** The terrain's draws (a big world's tiles, else the one), the pads' and the country roads' (and what draws those), as built: what `relay` draws again. */
  private grounds: pc.Entity[] = [];
  private pads: { slabs: pc.Entity[]; edges: pc.Entity[] } = { slabs: [], edges: [] };
  private roadDraws: pc.Entity[] = [];
  private roadBuilder: RoadBuilder | null = null;

  /** Whether `world` can be taken on in place (`relay`): built, in ground tiles, another town on the same land (`canRelay`). */
  relayable(world: WorldStructure): boolean {
    return !!this.host && this.tiles.length > 0 && this.canRelay(world);
  }

  /**
   * Takes on another town on the same land in place (`canRelay`; step 2's edits): laid out again (`relayout`), then
   * the ground tiles where the land as drawn changed shaded again, the pads and the country roads drawn again, the
   * water as the land now has it and the grass there. Says the box (m) whose woods and flowers are to be grown again
   * (`regrow`), or null when nothing changed. Between `pace`s; not while `build` runs.
   */
  async relay(world: WorldStructure, pace?: () => Promise<void>): Promise<[number, number, number, number] | null> {
    if (!this.relayable(world)) return null;
    const host = this.host!;
    const { app, parent } = host;
    // (After the town's own planning started: a frame between.)
    await pace?.();
    if (!this.host || !this.relayable(world)) return null;
    const changed = perfTime('nature: relay layout', () => this.relayout(world));
    this.world = world;
    await pace?.();
    if (!this.host) return null;
    // The pads, in the ground tiles where a lot came, went or moved, or the land changed (the rest kept as drawn),
    // and the country roads, whole (a few draws).
    const dirty = this.padsChanged(changed?.rect ?? null);
    const pads = perfTime('nature: pads', () => this.padDraws(app, parent, this.grounds, dirty), dirty ? `${dirty.size} tiles` : 'all');
    const old = [...pads.gone, ...this.roadDraws];
    this.pads = pads;
    await pace?.();
    if (!this.host) {
      for (const e of pads.made) e.destroy();
      return null;
    }
    this.roadDraws = this.roadBuilder ? perfTime('nature: roads', () => this.roadBuilder!(this.roads, parent)) : [];
    const fresh = [...pads.made, ...this.roadDraws];
    const gone = new Set(old);
    this.others = [...this.others.filter((e) => !gone.has(e)), ...fresh];
    this.drawsChanged(fresh, old);
    this.grass?.onGround([...this.grounds, ...pads.slabs].map((g) => g.render!.meshInstances[0]));
    if (!changed) return null;
    const box = this.landBox(changed.rect, LAND_DONE_PAD);
    const tiles = this.tilesIn(box);
    for (const k of tiles) {
      await pace?.();
      const t = this.tiles[k];
      if (!t || !this.host) return null;
      const positions = this.tileGrid(k);
      const colors = new Float32Array((positions.length / 3) * 4);
      const span = perfStart('land: shade tile');
      await this.shadeTerrain(positions, colors);
      perfEnd(span);
      t.kept = bytesOf(colors);
      this.redrawTile(t, positions, colors);
    }
    this.pondsIn(tiles);
    this.sheetAsNeeded();
    const grass = this.grass?.grassField;
    if (grass) {
      for (let z = box[1]; z < box[3]; z += LAND_GRASS_STRIP) {
        await pace?.();
        recolourGrass(grass, this, box[0], z, box[2], Math.min(box[3], z + LAND_GRASS_STRIP));
      }
      regroundGrass(grass, this, box[0], box[1], box[2], box[3]);
      this.grass?.landChanged(true);
    }
    const [x0, z0, x1, z1] = changed.box;
    return [Math.min(box[0], x0), Math.min(box[1], z0), Math.max(box[2], x1), Math.max(box[3], z1)];
  }

  /**
   * Builds again the plants built without the file at `url`, now that it has come: the new draws
   * (switched off, like a first build's), the ones they replace, and `commit`, which puts the new
   * in the old ones' place here (the lawn's hiding goes over to them) and takes the old ones down;
   * the caller shows the new draws as it commits, or `discard`s them. Null when none was built
   * without it. Not while `build` or another `rebuild` runs.
   */
  async rebuild(app: pc.AppBase, parent: pc.Entity, assets: AssetRegistry, url: string, pace?: () => Promise<void>): Promise<NatureRebuild | null> {
    const stale = this.batches.filter((b) => b.again && b.built.failed?.includes(url));
    if (!stale.length) return null;
    const fresh: Batch[] = [];
    for (const { again, casts, part } of stale) {
      const { key, matrices, lod, cell } = again!;
      fresh.push(batch(again!, casts, part, await instanced(app, parent, assets, key, matrices, lod, cell, pace)));
    }
    const replaced = gather(stale).draws;
    const commit = () => {
      for (const b of stale) release(b.built);
      this.batches = this.batches.map((b) => fresh[stale.indexOf(b)] ?? b);
      const going = new Set(replaced);
      this.lawn = this.lawn.filter((layer) => !layer.draws.some((d) => going.has(d.entity)));
      stale.forEach(({ again }, i) => {
        if (!again!.tiles) return;
        const layers = lawnLayers(fresh[i].built, again!.matrices, again!.tiles, again!.clear);
        this.lawn.push(...layers);
        if (this.lawnMask) hideLawn(layers, this.lawnMask, uploadLawn);
      });
      this.findSquares();
    };
    return { ...gather(fresh), replaced, commit, discard: () => fresh.forEach((b) => release(b.built)) };
  }

  /** Takes down everything it draws (the plants' templates with their draws). */
  dispose(): void {
    for (const b of this.batches) release(b.built);
    this.crowns?.destroy();
    this.crowns = null;
    // A ground tile's levels not drawn now are no draw's: they go here (the one drawn goes with its entity).
    disposeTiles(this.tiles);
    this.tiles = [];
    this.groundMats = null;
    this.padTexture?.destroy();
    this.padTexture = null;
    this.slabMaterial = null;
    this.woodsSquares = [];
    this.flowerSquares = [];
    for (const e of this.others) e.destroy();
    this.grounds = [];
    this.pads = { slabs: [], edges: [] };
    this.padGroups.clear();
    this.padList = [];
    this.roadDraws = [];
    for (const e of this.ponds.values()) e.destroy();
    this.ponds.clear();
    this.sheet = null;
    // (The land's work still to come, `landDone`, `redrawLand`, `regrow`, finds nothing to draw into.)
    this.host = null;
    this.world = null;
    if (this.waterMaterial) {
      this.waterMaterial.lib.release(this.waterMaterial.material);
      this.waterMaterial.material.destroy();
      this.waterMaterial = null;
    }
    this.flowMap?.destroy();
    this.flowMap = null;
    this.grass?.dispose();
    this.grass = null;
    this.batches = [];
    this.others = [];
    this.lawn = [];
  }

  /** Each of its plants' draws by kind (see `NaturePart`); the terrain, the water and the roads aren't in it. */
  parts(): Map<pc.Entity, NaturePart> {
    const out = new Map<pc.Entity, NaturePart>();
    for (const b of this.batches) for (const e of gather([b]).draws) out.set(e, b.part);
    for (const d of this.grass?.draws ?? []) out.set(d.entity, 'grass');
    return out;
  }

  /** Each frame: the grass near the eye (game space) in view of the camera (`Grass.frame`). */
  grassFrame(eye: { x: number; y: number; z: number }, camera: pc.CameraComponent, datum = 0): void {
    this.grass?.frame(eye, camera.frustum, datum);
  }

  /** The residents pushing the grass aside now (`Grass.setPushers`). */
  pushGrass(pushers: readonly GrassPusher[]): void {
    this.grass?.setPushers(pushers);
  }

  /**
   * Hides the lawn on tiles that are now indoors or under furniture, and shows it again elsewhere:
   * the flowers (on a tile split by a diagonal wall, each follows the half it stands in) and the grass.
   */
  refreshLawn(world: WorldStructure): void {
    this.world = world;
    // (A world message that changed nothing the lawn grows round, a focus lot shown: nothing to do.)
    const inputs = perfTime('lawn: inputs', () => lawnInputs(world));
    if (this.lawnInputs && sameLawnInputs(this.lawnInputs, inputs)) return;
    this.lawnInputs = inputs;
    const mask = (this.lawnMask = perfTime('lawn: mask', () => lawnMask(world)));
    perfTime('lawn: flowers', () => hideLawn(this.lawn, mask, uploadLawn));
    // (Its bare tiles kept for the birds, who ask next.)
    const bare = perfTime('lawn: bare tiles', () => keepBareTiles(world, inputs, mask));
    perfTime('lawn: grass', () => this.grass?.refresh(world, mask, bare));
  }

  /**
   * A big world's ground: the landscape's square in `TILE` m tiles, each with three levels (64, 32
   * and 16 quads across) and a skirt hanging from its edges (no cracks where neighbours differ), one
   * draw each, the level it draws chosen by distance from the camera's target (`frameTiles`). Only
   * the finest grid is shaded (heights and colours, a ring past the tile for its normals; both kept,
   * the colours as bytes); the coarser take every second and fourth of its points. The finest level
   * is made only for the tiles near enough to draw it (`frameTiles`), and let go again far off.
   */
  private async terrainTiles(app: pc.AppBase, parent: pc.Entity, pace?: () => Promise<void>): Promise<pc.Entity[]> {
    const { count } = this.tileFrame();
    const material = this.groundMaterial(app);
    const out: pc.Entity[] = [];
    this.tiles = [];
    // The worker shades a few tiles ahead while one's meshes are made here (its round trips overlap);
    // on the page, one at a time.
    const ahead = this.offThread ? SHADE_AHEAD : 0;
    const shading: Promise<{ positions: Float32Array; colors: Float32Array }>[] = [];
    for (let k = 0; k < count * count; k++) {
      while (shading.length <= ahead && k + shading.length < count * count) {
        const at = k + shading.length;
        shading.push(this.shaded(() => this.tileGrid(at), pace));
      }
      const { positions, colors } = await shading.shift()!;
      const [i, j, tx, tz] = this.tileCorner(k);
      const t: GroundTile = { mi: null!, meshes: [null, null, null], x: tx + TILE / 2, z: tz + TILE / 2, lod: 1, kept: bytesOf(colors), heights: heightsOf(positions), stale: 0 };
      t.meshes[1] = this.tileLevel(t, k, 1, false);
      t.meshes[2] = this.tileLevel(t, k, 2, false);
      const mi = (t.mi = new pc.MeshInstance(t.meshes[1]!, material));
      const entity = new pc.Entity(`terrain${i},${j}`, app);
      entity.addComponent('render', { meshInstances: [mi], castShadows: false, receiveShadows: true });
      entity.enabled = false;
      parent.addChild(entity);
      out.push(entity);
      this.tiles.push(t);
    }
    return out;
  }

  /** The ground tiles' layout: the landscape's square's corner and side, tiles across, and the finest level's quads across (`S`), their size (`c`) and its grid's points across with a ring (`n`). */
  private tileFrame(): { x0: number; z0: number; size: number; count: number; S: number; c: number; n: number } {
    const { size } = this.terrainGrid();
    const S = TILE_LODS[0];
    return { x0: this.w / 2 - size / 2, z0: this.d / 2 - size / 2, size, count: Math.ceil(size / TILE), S, c: TILE / S, n: S + 3 };
  }

  /**
   * The ground as drawn now at (`x`, `z`), for the benches: the terrain's height between the points of its tile's
   * finest grid (null: no tile there), and the grass's ground there (null: no grass grows in that patch).
   */
  drawnAt(x: number, z: number): { terrain: number | null; grass: number | null } {
    let terrain: number | null = null;
    const { x0, z0, count, S, c, n } = this.tileFrame();
    const [ti, tj] = [Math.floor((x - x0) / TILE), Math.floor((z - z0) / TILE)];
    const h = ti >= 0 && tj >= 0 && ti < count && tj < count ? this.tiles[tj * count + ti]?.heights : undefined;
    if (h) {
      const [, , tx, tz] = this.tileCorner(tj * count + ti);
      const [fc, fr] = [(x - tx) / c, S - (z - tz) / c];
      const [c0, r0] = [Math.min(S - 1, Math.floor(fc)), Math.min(S - 1, Math.floor(fr))];
      const [u, v] = [fc - c0, fr - r0];
      const at = (cc: number, rr: number) => h[(rr + 1) * n + cc + 1];
      terrain = (at(c0, r0) * (1 - u) + at(c0 + 1, r0) * u) * (1 - v) + (at(c0, r0 + 1) * (1 - u) + at(c0 + 1, r0 + 1) * u) * v;
    }
    let grass: number | null = null;
    const g = this.grass?.grassField;
    if (g) {
      const p = GRASS.patch;
      const [i, j] = [Math.floor((x - g.x0) / p), Math.floor((z - g.z0) / p)];
      if (i >= 0 && j >= 0 && i < g.pw && j < g.pd && g.patches[j * g.pw + i]) {
        const k = (j * g.pw + i) * 3;
        grass = g.ground[k] + g.ground[k + 1] * (x - g.x0 - i * p) + g.ground[k + 2] * (z - g.z0 - j * p);
      }
    }
    return { terrain, grass };
  }

  /** Tile `k`'s place: across, down, and its corner (m). */
  private tileCorner(k: number): [number, number, number, number] {
    const { x0, z0, count } = this.tileFrame();
    return [k % count, Math.floor(k / count), x0 + (k % count) * TILE, z0 + Math.floor(k / count) * TILE];
  }

  /**
   * Tile `k`'s finest grid (x and z; y to be set), a ring of one past it for the normals: (col, row) from -1 to S + 1,
   * row 0 at its far (max z) edge. The last tiles across and down reach past the landscape's square (its side is
   * seldom a whole number of tiles): their points past it are drawn in to its edge, so the ground stops there. Out
   * there the rim kept climbing, and the menus' haze, which hangs over the land up to the square's edge, let a strip
   * of it show above itself along the far sides.
   */
  private tileGrid(k: number): Float32Array {
    const { S, c, n, x0, z0, size } = this.tileFrame();
    const [, , tx, tz] = this.tileCorner(k);
    const [x1, z1] = [x0 + size, z0 + size];
    const positions = new Float32Array(n * n * 3);
    for (let row = -1, v = 0; row <= S + 1; row++) for (let col = -1; col <= S + 1; col++, v++) [positions[v * 3], positions[v * 3 + 2]] = [Math.min(x1, tx + col * c), Math.min(z1, tz + (S - row) * c)];
    return positions;
  }

  /** Tile `k`'s finest grid as `writeTileLevel` reads it. */
  private gridOf(k: number): TileGrid {
    const { S, c, n, x0, z0, size } = this.tileFrame();
    const [, , tx, tz] = this.tileCorner(k);
    return { S, c, n, tx, tz, x1: x0 + size, z1: z0 + size };
  }

  /** Tile `t`'s (`k`'s) level `q` made from its heights and colours as kept (`live`: to be written into). */
  private tileLevel(t: GroundTile, k: number, q: number, live: boolean): pc.Mesh {
    return tileLevelMesh(this.host!.app.graphicsDevice, this.gridOf(k), TILE_LODS[q], t.heights!, t.kept!, live);
  }

  /**
   * Tile `t`'s (`k`'s) level `q` from its heights and colours as kept now: written into its mesh where it keeps its
   * vertices, else made anew (live: a brush has reached the tile, it will be written into again) and the old one
   * destroyed; where the tile draws it, drawn.
   */
  private refreshLevel(t: GroundTile, k: number, q: number): void {
    const mesh = t.meshes[q];
    t.stale &= ~(1 << q);
    if (mesh && rewriteTileLevel(mesh, this.gridOf(k), TILE_LODS[q], t.heights!, t.kept!)) return;
    const made = this.tileLevel(t, k, q, true);
    t.meshes[q] = made;
    if (t.lod === q) t.mi.mesh = made;
    if (mesh) mesh.destroy();
  }

  /**
   * A tile's grid shaded again (`positions` with their heights, `colors`): its heights and colours kept, and its
   * levels drawn again from them (the finest only if it has one).
   */
  private redrawTile(t: GroundTile, positions: Float32Array, colors: Float32Array): void {
    if (!this.host) return;
    const k = this.tiles.indexOf(t);
    t.kept = bytesOf(colors);
    t.heights = heightsOf(positions, t.heights);
    for (let q = 0; q < TILE_LODS.length; q++) if (t.meshes[q]) this.refreshLevel(t, k, q);
  }

  /**
   * The ground tiles a box (m) reaches (their grids' rings too), by index. With a height field, the box of the samples
   * in `rect` (out to the ones beside them, and `pad` m more; to `LAND_EDGE_FADE` past the world's edge where it
   * reaches it: the land past it follows the edge).
   */
  private tilesIn(box: [number, number, number, number]): number[] {
    const { x0, z0, count, c } = this.tileFrame();
    const out: number[] = [];
    const [i0, j0] = [Math.max(0, Math.floor((box[0] - c - x0) / TILE)), Math.max(0, Math.floor((box[1] - c - z0) / TILE))];
    const [i1, j1] = [Math.min(count - 1, Math.floor((box[2] + c - x0) / TILE)), Math.min(count - 1, Math.floor((box[3] + c - z0) / TILE))];
    for (let j = j0; j <= j1; j++) for (let i = i0; i <= i1; i++) out.push(j * count + i);
    return out;
  }

  /**
   * The box (m) round the samples in `rect` the land's drawing leans on, `pad` m out, and on past the world's edge where
   * it reaches it (and `along` m more each way there: the water past the edge as rounded again once a stroke is done,
   * `roundedWater`).
   */
  landBox(rect: LandRect, pad = 0, along = 0): [number, number, number, number] {
    const field = this.heightField!;
    const box = rectBox(rect, pad);
    const fade = LAND_EDGE_FADE_M + pad;
    if (rect.i0 <= 0 || rect.j0 <= 0 || rect.i1 >= field.cols || rect.j1 >= field.rows) [box[0], box[1], box[2], box[3]] = [box[0] - along, box[1] - along, box[2] + along, box[3] + along];
    if (rect.i0 <= 0) box[0] = Math.min(box[0], -fade);
    if (rect.j0 <= 0) box[1] = Math.min(box[1], -fade);
    if (rect.i1 >= field.cols) box[2] = Math.max(box[2], Math.max(this.w, field.cols * LAND_CELL) + fade);
    if (rect.j1 >= field.rows) box[3] = Math.max(box[3], Math.max(this.d, field.rows * LAND_CELL) + fade);
    return box;
  }

  /**
   * A height field's samples in `rect` changed (a brush's stamp): the ground tiles there drawn again from it, with the
   * colours they had (`kept`; shaded again once the stroke is done, `landDone`), the grass's heights there and the
   * water above the sea's level. Cheap, every stamp: only the heights of the tiles' points in the stamp's box are
   * worked out again (each tile keeps its own), and only the level each tile draws is written, the rows of it they
   * are in (the others are written when the tile next draws them, `frameTiles`, or when the stroke is done). Says how
   * many tiles it drew again.
   */
  reground(rect: LandRect): number {
    if (!this.heightField || !this.host) return 0;
    // (On a land its town follows, drawn from the field there until the stroke is done: `landChanged`.)
    this.landStamped(rect);
    const box = this.landBox(rect, LAND_CELL);
    const tiles = this.tilesIn(box);
    const span = perfStart('land: stamp heights');
    const rows: [number, number][] = [];
    for (const k of tiles) rows.push(this.tileHeights(k, box));
    perfEnd(span);
    const write = perfStart('land: stamp meshes');
    tiles.forEach((k, at) => {
      const t = this.tiles[k];
      if (!t?.kept || !t.heights) return;
      // (The rows of the level drawn whose points, or their neighbours for the normals, moved.)
      const step = TILE_LODS[0] / TILE_LODS[t.lod];
      const [r0, r1] = rows[at];
      const lo = Math.max(0, Math.ceil((r0 - 1) / step));
      const hi = Math.min(TILE_LODS[t.lod], Math.floor((r1 + 1) / step));
      const mesh = t.meshes[t.lod];
      if (!mesh || hi < lo || !rewriteTileLevel(mesh, this.gridOf(k), TILE_LODS[t.lod], t.heights, null, lo, hi)) this.refreshLevel(t, k, t.lod);
      for (let q = 0; q < TILE_LODS.length; q++) if (q !== t.lod && t.meshes[q]) t.stale |= 1 << q;
    });
    perfEnd(write);
    perfTime('land: stamp ponds', () => this.pondsIn(tiles));
    perfTime('land: stamp flow', () => this.flowMap?.changed(rect, false));
    const grass = this.grass?.grassField;
    if (grass) {
      perfTime('land: stamp grass', () => {
        regroundGrass(grass, this, box[0], box[1], box[2], box[3]);
        this.grass?.landChanged(false);
      });
    }
    return tiles.length;
  }

  /**
   * Tile `k`'s heights at the points of its finest grid in `box` (m) worked out again from the land as it is (all of
   * them the first time); says the rows of its grid (from 0, its ring's -1 to `S + 1`) they are in.
   */
  private tileHeights(k: number, box: readonly [number, number, number, number]): [number, number] {
    const t = this.tiles[k];
    const { S, c, n, tx, tz, x1, z1 } = this.gridOf(k);
    let [c0, c1, r0, r1] = [-1, S + 1, -1, S + 1];
    if (t.heights) {
      // (A point past the landscape's square stands on its edge: in the box if the edge is.)
      c0 = Math.max(-1, Math.ceil((box[0] - tx) / c));
      c1 = box[2] >= x1 ? S + 1 : Math.min(S + 1, Math.floor((box[2] - tx) / c));
      r0 = box[3] >= z1 ? -1 : Math.max(-1, Math.ceil(S - (box[3] - tz) / c));
      r1 = Math.min(S + 1, Math.floor(S - (box[1] - tz) / c));
    } else t.heights = new Float32Array(n * n);
    if (c1 < c0 || r1 < r0) return [1, 0];
    const h = t.heights;
    // (`heightAt`, the roads' levelling kept by point: the roads don't move as the land is shaped.)
    const levels = this.surface ? null : (t.levels ??= this.roadLevels(this.tileGrid(k)));
    for (let row = r0; row <= r1; row++) {
      const z = Math.min(z1, tz + (S - row) * c);
      for (let col = c0; col <= c1; col++) {
        const x = Math.min(x1, tx + col * c);
        const p = (row + 1) * n + col + 1;
        if (!levels) h[p] = this.terrainAt(x, z);
        else {
          const g = this.ground(x, z);
          h[p] = g + (levels[p * 2 + 1] - g) * levels[p * 2];
        }
      }
    }
    return [r0 + 1, r1 + 1];
  }

  /**
   * How the roads level the land at each point of a tile's grid (`heightAt`): how much of the way (0..1) to the
   * height it is levelled to (a pair a point).
   */
  private roadLevels(positions: Float32Array): Float32Array {
    const out = new Float32Array((positions.length / 3) * 2);
    for (let v = 0, q = 0; v < positions.length; v += 3, q += 2) {
      const road = this.roads.near(positions[v], positions[v + 2]);
      if (!road) continue;
      out[q] = 1 - smoothstep(road.width / 2 + 0.4, road.width / 2 + ROAD_VERGE, road.dist);
      out[q + 1] = road.h - 0.02;
    }
    return out;
  }

  /** The water above the sea's level of the tiles `tiles` drawn again (`pond`), the owner told what came and went. */
  private pondsIn(tiles: readonly number[]): void {
    const added: pc.Entity[] = [];
    const removed: pc.Entity[] = [];
    for (const k of tiles) {
      const old = this.ponds.get(k);
      // (A tile with water before and after keeps its draw: its mesh is swapped.)
      if (old && this.pondMesh(k, old)) continue;
      if (old) removed.push(old);
      this.ponds.delete(k);
      added.push(...this.pond(k));
    }
    this.drawsChanged(added, removed);
  }

  /** The owner told of draws come and gone (`onDraws`), and the gone destroyed. */
  private drawsChanged(added: pc.Entity[], removed: pc.Entity[]): void {
    if (!added.length && !removed.length) return;
    this.onDraws?.(added, removed);
    for (const e of removed) e.destroy();
  }

  /** Tile `k`'s water above the sea's level (`pondQuads`) written into `into` where it fits (`pondMeshOf`), or null without any. */
  private pondData(k: number, into: pc.Mesh | null = null): pc.Mesh | null {
    const field = this.heightField;
    if (!field || !this.host) return null;
    const [, , tx, tz] = this.tileCorner(k);
    const quads = pondQuads(field, tx, tz, tx + TILE, tz + TILE);
    return quads ? pondMeshOf(this.host.app.graphicsDevice, quads, into) : null;
  }

  /** Tile `k`'s water drawn again by its draw `entity` (into its mesh where it fits, else a new one and the old destroyed); false if it has none now. */
  private pondMesh(k: number, entity: pc.Entity): boolean {
    const mi = entity.render!.meshInstances[0];
    const old = mi.mesh;
    const mesh = this.pondData(k, old);
    if (!mesh) return false;
    if (mesh !== old) {
      mi.mesh = mesh;
      old?.destroy();
    }
    return true;
  }

  /** Tile `k`'s water above the sea's level (`pondQuads`), as a draw in the sheet's material (switched off), kept by tile; none without any. */
  private pond(k: number): pc.Entity[] {
    const mesh = this.pondData(k);
    if (!mesh || !this.host) return [];
    const [i, j] = this.tileCorner(k);
    const { app, parent } = this.host;
    const entity = drawOf(app, parent, `pond${i},${j}`, mesh, this.waterMat(app));
    this.ponds.set(k, entity);
    return [entity];
  }

  /** The water sheet drawn if the land has water at the sea's level (or the landscape its own), taken away if not. */
  private sheetAsNeeded(): void {
    if (!this.host) return;
    if (this.seaSheet && !this.sheet) {
      this.sheet = this.water(this.host.app, this.host.parent);
      this.others.push(this.sheet);
      this.drawsChanged([this.sheet], []);
    } else if (!this.seaSheet && this.sheet) {
      const gone = this.sheet;
      this.sheet = null;
      this.others = this.others.filter((e) => e !== gone);
      this.drawsChanged([], [gone]);
    }
  }

  /**
   * A stroke of the brushes done over the samples in `rect`: the land's water, top and zones measured again
   * (`LandscapeLayout.landChanged`), then the ground tiles there shaded again (their colours too, and kept), the
   * sheet as the water now needs it, and the grass's colours and ground there, a tile or a strip of the grass at a time between
   * `pace`s. (The woods and flowers: `regrow`.) Says the box (m) it reached.
   */
  async landDone(rect: LandRect, pace?: () => Promise<void>): Promise<[number, number, number, number] | null> {
    if (!this.heightField || !this.host) return null;
    // (On a land its town follows, what the surface solved again moved too.)
    const exits = this.landExits;
    const changed = this.landChanged(rect);
    const box = this.landBox(changed, LAND_DONE_PAD, ROUND_REACH);
    // (A river drawn out to the edge, or away from it: the valley past the edge came, went or moved with it.)
    const valleys = this.valleysMoved(exits);
    if (valleys) [box[0], box[1], box[2], box[3]] = [Math.min(box[0], valleys[0]), Math.min(box[1], valleys[1]), Math.max(box[2], valleys[2]), Math.max(box[3], valleys[3])];
    perfTime('land: flow', () => this.flowMap?.changed(changed, true));
    for (const k of this.tilesIn(box)) {
      await pace?.();
      const t = this.tiles[k];
      if (!t || !this.host) return null;
      const positions = this.tileGrid(k);
      const colors = new Float32Array((positions.length / 3) * 4);
      // (Whole before its meshes are made: it gives a turn every 4096 points, `pace` or not.)
      const span = perfStart('land: shade tile');
      await this.shadeTerrain(positions, colors);
      perfEnd(span);
      t.kept = bytesOf(colors);
      this.redrawTile(t, positions, colors);
    }
    this.sheetAsNeeded();
    // (Roads or lots the land moved: the pads and the country roads drawn again on them.)
    if (this.builtMoved) {
      this.builtMoved = false;
      await pace?.();
      if (!this.host) return null;
      perfTime('land: pads and roads', () => this.redrawBuilt());
    }
    const grass = this.grass?.grassField;
    if (grass) {
      for (let z = box[1]; z < box[3]; z += LAND_GRASS_STRIP) {
        await pace?.();
        perfTime('land: grass strip', () => recolourGrass(grass, this, box[0], z, box[2], Math.min(box[3], z + LAND_GRASS_STRIP)), `${Math.round(box[2] - box[0])} m`);
      }
      // (Its ground too: the stamps laid it on the field as the brush left it, but on a land its town follows the land
      // is settled and graded round what is built at the stroke's end. Left there, the grass stood off the ground, by a
      // road the land raised beside it some 0.3 m over its edge.)
      await pace?.();
      perfTime('land: grass ground', () => regroundGrass(grass, this, box[0], box[1], box[2], box[3]));
      perfTime('land: grass upload', () => this.grass?.landChanged(true));
    }
    return box;
  }

  /** The pads and the country roads drawn again, whole, on the surface as it is now (the land moved them: `landDone`). */
  private redrawBuilt(): void {
    if (!this.host || !this.surface) return;
    const { app, parent } = this.host;
    const old = [...this.pads.slabs, ...this.pads.edges, ...this.roadDraws];
    const pads = (this.pads = this.padDraws(app, parent, this.grounds));
    this.roadDraws = this.roadBuilder ? this.roadBuilder(this.roads, parent) : [];
    const fresh = [...pads.slabs, ...pads.edges, ...this.roadDraws];
    const gone = new Set(old);
    this.others = [...this.others.filter((e) => !gone.has(e)), ...fresh];
    this.drawsChanged(fresh, old);
    this.grass?.onGround([...this.grounds, ...pads.slabs].map((g) => g.render!.meshInstances[0]));
  }

  /**
   * The land drawn again whole (a height field taken on or let go of, `setLand`): every ground tile shaded again and
   * its colours kept, the water above the sea's level, the sheet, and the grass field, made in the render data worker
   * where one runs (sent the field), else here between `pace`s. (The woods and flowers: `regrow`.)
   */
  async redrawLand(pace?: () => Promise<void>): Promise<void> {
    const world = this.world;
    if (!world || !this.host || !this.tiles.length) return;
    const off = OffThreadLand.open(world);
    off?.setField(this.heightField);
    this.offThread = off;
    try {
      const grassLand = off?.grass(this.ring) ?? Promise.resolve(null);
      const ahead = off ? SHADE_AHEAD : 0;
      const shading: Promise<{ positions: Float32Array; colors: Float32Array }>[] = [];
      const count = this.tiles.length;
      for (let k = 0; k < count; k++) {
        while (shading.length <= ahead && k + shading.length < count) {
          const at = k + shading.length;
          shading.push(this.shaded(() => this.tileGrid(at), pace));
        }
        const { positions, colors } = await shading.shift()!;
        await pace?.();
        const t = this.tiles[k];
        if (!t || !this.host) return;
        t.kept = bytesOf(colors);
        t.heights = heightsOf(positions, t.heights);
        t.stale = 0;
        // (Made anew, kept off the CPU: most of them no brush will reach.)
        for (let q = 0; q < TILE_LODS.length; q++) {
          const old = t.meshes[q];
          if (!old) continue;
          const made = this.tileLevel(t, k, q, false);
          t.meshes[q] = made;
          if (t.lod === q) t.mi.mesh = made;
          old.destroy();
        }
      }
      this.pondsIn([...this.tiles.keys()]);
      this.sheetAsNeeded();
      const grass = this.grass?.grassField;
      if (grass) {
        const made = await grassLand;
        if (made && made.colour.length === grass.colour.length && made.ground.length === grass.ground.length) {
          grass.colour.set(made.colour);
          grass.ground.set(made.ground);
          markGrassPatches(grass);
        } else {
          const [x1, z1] = [grass.x0 + grass.w, grass.z0 + grass.d];
          for (let z = grass.z0; z < z1; z += LAND_GRASS_STRIP) {
            await pace?.();
            recolourGrass(grass, this, grass.x0, z, x1, Math.min(z1, z + LAND_GRASS_STRIP));
          }
          regroundGrass(grass, this, grass.x0, grass.z0, x1, z1);
        }
        this.grass?.landChanged(true);
      }
    } finally {
      off?.close();
      this.offThread = null;
    }
  }

  /**
   * The woods and the lawn's flowers grown again: with a height field, the squares a box (m) reaches (all: null)
   * from their own streams (`woodsAgain`, `lawnsAgain`), else all of them from the one stream, as built. The models
   * whose plants changed are drawn anew (their templates kept), as `rebuild` does: the new draws, the ones they
   * replace, and `commit` or `discard`. Null before a build.
   */
  async regrow(assets: AssetRegistry, box: [number, number, number, number] | null, pace?: () => Promise<void>): Promise<NatureRebuild | null> {
    const world = this.world;
    if (!world || !this.host) return null;
    const { app, parent } = this.host;
    let woods: Woods;
    let woodsChanged: Set<string> | null = null;
    let lawns: LawnPlacement[];
    let lawnsChanged: Set<string> | null = null;
    if (this.heightField) {
      ({ woods, changed: woodsChanged } = await perfTimeAsync('land: woods', () => this.woodsAgain(box, pace)));
      ({ lawns, changed: lawnsChanged } = await perfTimeAsync('land: flowers', () => this.lawnsAgain(world, box, pace)));
    } else {
      woods = await this.woods(pace);
      lawns = await this.lawns(world, pace);
    }
    const isWoods = (b: Batch) => b.part === 'woods' || b.part === 'farWoods';
    const stale = this.batches.filter((b) => (isWoods(b) ? !woodsChanged || woodsChanged.has(b.key) : !lawnsChanged || lawnsChanged.has(b.key)));
    const templates = new Map<string, ModelTemplate>();
    for (const b of stale) if (b.built.template && !b.built.failed) templates.set(`${b.key}|${b.lod}`, b.built.template);
    const fresh: Batch[] = [];
    const freshLawns: { at: number; matrices: Float32Array[]; tiles: readonly number[]; clear?: boolean }[] = [];
    for (const [bucket, casts] of [
      [woods.near, true],
      [woods.far, false],
    ] as const) {
      for (const [key, list] of bucket) {
        if (woodsChanged && !woodsChanged.has(key)) continue;
        const lod = casts ? 0 : 1;
        const built = await instanced(app, parent, assets, key, list, lod, woods.sector, pace, templates.get(`${key}|${lod}`));
        fresh.push(batch({ key, matrices: list, lod, cell: woods.sector }, casts, casts ? 'woods' : 'farWoods', built));
      }
    }
    for (const { key, matrices, tiles, cell, clear } of lawns) {
      if (lawnsChanged && !lawnsChanged.has(key)) continue;
      const lod = this.lawnLod;
      const built = await instanced(app, parent, assets, key, matrices, lod, cell, pace, templates.get(`${key}|${lod}`));
      freshLawns.push({ at: fresh.length, matrices, tiles, clear });
      fresh.push(batch({ key, matrices, lod, cell, tiles, clear }, false, 'flowers', built));
    }
    // A big world's crown clusters, gathered again from every tree.
    let crowns: FarCrowns | null = null;
    if (this.built && (!woodsChanged || woodsChanged.size)) {
      const trees: { kind: string; m: Float32Array }[] = [];
      for (const map of [woods.near, woods.far]) for (const [kind, list] of map) for (const m of list) trees.push({ kind, m });
      crowns = new FarCrowns(app, parent, trees, woods.sector);
    }
    const kept = new Set(templates.values());
    const replaced = gather(stale).draws;
    const commit = () => {
      const used = new Set(fresh.map((b) => b.built.template));
      for (const b of stale) release(b.built, used);
      const going = new Set(stale);
      this.batches = [...this.batches.filter((b) => !going.has(b)), ...fresh];
      const gone = new Set(replaced);
      this.lawn = this.lawn.filter((layer) => !layer.draws.some((d) => gone.has(d.entity)));
      for (const { at, matrices, tiles, clear } of freshLawns) {
        const layers = lawnLayers(fresh[at].built, matrices, tiles, clear);
        this.lawn.push(...layers);
        if (this.lawnMask) hideLawn(layers, this.lawnMask, uploadLawn);
      }
      if (crowns) {
        this.crowns?.destroy();
        this.crowns = crowns;
      }
      this.woodsSector = woods.sector;
      this.findSquares();
    };
    const discard = () => {
      for (const b of fresh) release(b.built, kept);
      crowns?.destroy();
    };
    return { ...gather(fresh), replaced, commit, discard };
  }

  /** A big world's woods and flower squares, by where they are: those far from the camera are left out (`frameTiles`). */
  private findSquares(): void {
    const sector = this.woodsSector;
    const of = (parts: readonly NaturePart[]) =>
      this.built ? this.batches.filter((b) => parts.includes(b.part)).flatMap((b) => b.built.cells.flatMap((c) => c.draws.map((d) => d.meshInstance))).map((mi) => ({ mi, x: mi.aabb.center.x, z: -mi.aabb.center.z, key: sector?.(mi.aabb.center.x, -mi.aabb.center.z) ?? '' })) : [];
    this.woodsSquares = of(['woods', 'farWoods']);
    this.flowerSquares = of(['flowers']);
    this.framedAt.forget();
  }

  /** A big world's woods squares: each one's draw and where its middle is (game space). */
  private woodsSquares: { mi: pc.MeshInstance; x: number; z: number; key: string }[] = [];
  /** A big world's lawn flowers' squares, as the woods'. */
  private flowerSquares: { mi: pc.MeshInstance; x: number; z: number }[] = [];
  /** A big world's ground tiles: each one's draw, its meshes by level, where its middle is and the level drawn. */
  private tiles: GroundTile[] = [];
  /** Where `frameTiles` last looked from, and how far into the town view (the radius in 10 m steps; -1 out of it). */
  private readonly framedAt = new LookedFrom();
  private framedTown = -1;

  /**
   * Each frame (the game's): each ground tile draws the level for its distance from (`x`, `z`), the camera's target.
   * In the town view (`town`, its blend, over 0; `radius`, the orbit's) the woods draw their own trees only in the
   * squares near the target (nearer the further out it is: `TOWN_WOODS_NEAR` at the full view), crown clusters in the
   * rest (`FarCrowns`), and a ground tile's level goes by the eye's distance too (its altitude): culling by altitude
   * and the frustum, not the preset's distances. Out of it, as ever.
   */
  frameTiles(x: number, z: number, town = 0, radius = 0): void {
    // (The camera's target still, as far out: nothing changes.)
    const level = town > 0 ? Math.round(radius / 10) : -1;
    if (!this.framedAt.moved(x, z, 0.5) && level === this.framedTown) return;
    this.framedTown = level;
    const { woodsFar, flowersFar, tiles: [near, mid] } = this.open;
    if (town > 0) {
      const reach = woodsFar + (TOWN_WOODS_NEAR - woodsFar) * town;
      const cell = this.bigWoodsCell;
      const near = (key: string) => {
        const at = key.indexOf(':');
        return Math.hypot((Number(key.slice(0, at)) + 0.5) * cell - x, (Number(key.slice(at + 1)) + 0.5) * cell - z) < reach;
      };
      for (const w of this.woodsSquares) w.mi.visible = near(w.key);
      this.crowns?.show((key) => !near(key));
    } else {
      for (const w of this.woodsSquares) w.mi.visible = Math.hypot(w.x - x, w.z - z) < woodsFar;
      this.crowns?.show(null);
    }
    for (const f of this.flowerSquares) f.mi.visible = Math.hypot(f.x - x, f.z - z) < flowersFar;
    const lift = town * radius;
    this.tiles.forEach((t, k) => {
      const d = town > 0 ? Math.hypot(t.x - x, t.z - z, lift) : Math.hypot(t.x - x, t.z - z);
      const lod = d < near ? 0 : d < mid ? 1 : 2;
      if (lod === t.lod) return;
      t.lod = lod;
      // (Its finest level made as it comes near, and let go once it is far off; a level a brush changed while
      // another was drawn, written again.)
      if (!t.meshes[lod] && t.heights && t.kept) t.meshes[lod] = this.tileLevel(t, k, lod, false);
      else if (t.stale & (1 << lod)) this.refreshLevel(t, k, lod);
      t.mi.mesh = t.meshes[lod] ?? t.mi.mesh;
      if (lod === 2 && t.meshes[0]) {
        t.meshes[0].destroy();
        t.meshes[0] = null;
        t.stale &= ~1;
      }
    });
  }

  /** The ground: a regular grid over the landscape's square, lifted to the land and coloured. */
  private async terrain(app: pc.AppBase, parent: pc.Entity, pace?: () => Promise<void>): Promise<pc.Entity> {
    const { size, segments } = this.terrainGrid();
    const n = segments + 1;
    // (Rows from the far edge, moved to the town's middle.)
    const uvs = new Float32Array(n * n * 2);
    for (let row = 0, v = 0; row <= segments; row++) {
      for (let col = 0; col <= segments; col++, v++) {
        uvs[v * 2] = col / segments;
        uvs[v * 2 + 1] = 1 - row / segments;
      }
    }
    const grid = () => {
      const positions = new Float32Array(n * n * 3);
      for (let row = 0, v = 0; row <= segments; row++) {
        for (let col = 0; col <= segments; col++, v++) {
          positions[v * 3] = (col * size) / segments - size / 2 + this.w / 2;
          positions[v * 3 + 2] = ((segments - row) * size) / segments - size / 2 + this.d / 2;
        }
      }
      return positions;
    };
    const indices = new Uint32Array(segments * segments * 6);
    for (let row = 0, k = 0; row < segments; row++) {
      for (let col = 0; col < segments; col++) {
        indices[k++] = col + 1 + (row + 1) * n;
        indices[k++] = col + 1 + row * n;
        indices[k++] = col + row * n;
        indices[k++] = col + (row + 1) * n;
        indices[k++] = col + 1 + (row + 1) * n;
        indices[k++] = col + row * n;
      }
    }
    // sRGB vertex colours (converted on the GPU) multiply the grass detail texture.
    const { positions, colors } = await this.shaded(grid, pace);
    const normals = new Float32Array(positions.length);
    computeNormals(positions, indices, normals);
    const mesh = new pc.Mesh(app.graphicsDevice);
    mesh.setPositions(positions);
    mesh.setNormals(normals);
    mesh.setUvs(0, uvs);
    mesh.setColors(colors);
    mesh.setIndices(indices);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    gpuOnly(mesh);
    // (A copy of its own: the gusts' define is the landscape's.)
    return drawOf(app, parent, 'terrain', mesh, this.groundMaterial(app));
  }

  /**
   * The sea or the lake: one flat sheet at the water line under the whole landscape (the land
   * covers it everywhere but in the water), reflecting the sky.
   */
  private water(app: pc.AppBase, parent: pc.Entity): pc.Entity {
    const at = this.waterSheet();
    const h = at.size / 2;
    const mesh = new pc.Mesh(app.graphicsDevice);
    // (Wound as the ground: its front up.)
    mesh.setPositions([at.x - h, at.y, at.z + h, at.x + h, at.y, at.z + h, at.x - h, at.y, at.z - h, at.x + h, at.y, at.z - h]);
    mesh.setNormals([0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0]);
    mesh.setUvs(0, [0, 1, 1, 1, 0, 0, 1, 0]);
    mesh.setIndices([3, 1, 0, 2, 3, 0]);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    return drawOf(app, parent, 'water', mesh, this.waterMat(app));
  }

  /**
   * Draws the land from `land` from now on (`LandscapeLayout.setLand`), its water flowing as the land has it (the
   * flow map worked out whole).
   */
  override setLand(land: Land | null): void {
    super.setLand(land);
    this.flowMap?.setLand(this.heightField);
  }

  /**
   * Whether its water moves in the view of `frustum` (world space): drawn (shown), flowing (not `?diag=noflow`), and a
   * square of the land's water in view; with no height field, the sheet in view.
   */
  waterInView(frustum: pc.Frustum): boolean {
    if (!this.flowMap || !this.host) return false;
    let shown = !!this.sheet?.enabled;
    for (const e of this.ponds.values()) shown ||= e.enabled;
    if (!shown) return false;
    const seen = this.flowMap.inView(frustum, this.host.parent.getWorldTransform());
    if (seen !== null) return seen;
    const aabb = this.sheet?.render?.meshInstances[0]?.aabb;
    if (!aabb) return false;
    waterSphere.center.copy(aabb.center);
    waterSphere.radius = aabb.halfExtents.length();
    return frustum.containsSphere(waterSphere) > 0;
  }

  /**
   * The water's material (the sheet's and a height field's ponds'), made once: its own, it goes with the landscape.
   * Flowing (`pc/water.ts`) as its height field has it.
   */
  private waterMat(app: pc.AppBase): pc.StandardMaterial {
    if (this.waterMaterial) return this.waterMaterial.material;
    const mat = new pc.StandardMaterial();
    mat.name = 'water';
    mat.frontFace = GAME_FRONT_FACE;
    mat.useMetalness = true;
    mat.diffuse.fromString('#2F6F86');
    mat.metalness = 0;
    mat.glossInvert = true;
    mat.gloss = 0.12;
    mat.opacity = 0.9;
    mat.blendType = pc.BLEND_NORMAL;
    const lib = MaterialLibrary.for(app);
    lib.adopt(mat);
    if (flowingWater()) {
      this.flowMap = new WaterFlowMap(app.graphicsDevice);
      this.flowMap.use(mat);
      perfTime('nature: water flow', () => this.flowMap!.setLand(this.heightField));
    }
    mat.update();
    this.waterMaterial = { material: mat, lib };
    return mat;
  }

  /** The ground tiles' materials (the town view lays its colour map over them: `groundTiles.ts` `TownGroundMap`), the pads' slabs' too. */
  groundMaterials(): readonly pc.StandardMaterial[] {
    // (Worked out once: the tiles keep their material.)
    return (this.groundMats ??= [...new Set([...this.tiles.map((t) => t.mi.material as pc.StandardMaterial), ...(this.slabMaterial ? [this.slabMaterial] : [])])]);
  }

  /**
   * The terrain's material: the ground's, with the gusts (a copy of its own: their define is the landscape's), and on
   * a land the town follows another copy that draws nothing inside a lot (`GAME_PAD_MASK`: the lots' pads are slabs).
   */
  private groundMaterial(app: pc.AppBase): pc.StandardMaterial {
    const material = MaterialLibrary.for(app).surface('material.ground', { vertexColors: true, variant: this.surface ? 'grassPads' : 'grass' });
    grassGround(app.graphicsDevice, material);
    if (this.surface) padMasked(app.graphicsDevice, material);
    return material;
  }

  /**
   * On a land the town follows: the lots the terrain leaves out (the pad mask, a byte a metre: `grounds`' material), and each lot's slab at its pad (the
   * ground's own material, its colours the terrain's there) and its retaining edges (the foundations' stone, tinted) in a draw
   * of each per ground tile.
   */
  /**
   * The ground tiles whose pads are to be drawn again for the surface as it is now (`relay`): those within reach of
   * a lot whose pad came, went or moved since they were drawn, and those over `land` (land rect: where the land
   * changed). Null: all of them (none drawn yet).
   */
  private padsChanged(land: LandRect | null): Set<number> | null {
    if (!this.padGroups.size || !this.surface) return null;
    const pads = padsOf(this.surface, this.world?.plots.length ?? 0);
    const key = (p: { x: number; z: number; w: number; d: number; y: number }) => `${p.x},${p.z},${p.w},${p.d},${p.y}`;
    const [now, was] = [new Set(pads.map(key)), new Set(this.padList.map(key))];
    // (A lot's retaining edges are drawn by the higher of two lots side by side: a pad moved changes its neighbours' too.)
    const reach = 2 + Math.max(0, ...pads.map((p) => Math.max(p.w, p.d)));
    const out = new Set<number>();
    for (const p of [...pads.filter((q) => !was.has(key(q))), ...this.padList.filter((q) => !now.has(key(q)))]) {
      for (const k of this.tilesIn([p.x - reach, p.z - reach, p.x + p.w + reach, p.z + p.d + reach])) out.add(k);
    }
    if (land) for (const k of this.tilesIn(this.landBox(land, LAND_DONE_PAD))) out.add(k);
    return out;
  }
  /** The pads as last drawn (`padDraws`), and each ground tile's draws of them. */
  private padList: ReturnType<typeof padsOf> = [];
  private readonly padGroups = new Map<number, { slab: pc.Entity | null; edge: pc.Entity | null }>();

  /**
   * The pads' draws: the slabs and the edges (all of them), those made now and those they replace (`gone`, not yet
   * destroyed). `dirty`: only the ground tiles to draw again (`padsChanged`), the others' kept; null: all.
   */
  private padDraws(app: pc.AppBase, parent: pc.Entity, grounds: readonly pc.Entity[], dirty: Set<number> | null = null): { slabs: pc.Entity[]; edges: pc.Entity[]; made: pc.Entity[]; gone: pc.Entity[] } {
    const surface = this.surface!;
    const device = app.graphicsDevice;
    const all = padsOf(surface, this.world?.plots.length ?? 0);
    this.padList = all;
    const { x0, z0, size } = this.tileFrame();
    const count = Math.ceil(size / TILE);
    // The terrain leaves out the lots: a byte a metre over the world.
    const [w, d] = [Math.ceil(this.w), Math.ceil(this.d)];
    this.padTexture?.destroy();
    const mask = (this.padTexture = new pc.Texture(device, { name: 'padMask', width: w, height: d, format: pc.PIXELFORMAT_R8, mipmaps: false, minFilter: pc.FILTER_NEAREST, magFilter: pc.FILTER_NEAREST, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE, levels: [padMask(all, w, d)] }));
    const area = new Float32Array([0, 0, w, d]);
    for (const g of grounds) {
      const mi = g.render!.meshInstances[0];
      mi.setParameter('game_padMask', mask);
      mi.setParameter('game_padArea', area);
    }
    const groupOf = (x: number, z: number) => Math.max(0, Math.min(count - 1, Math.floor((z - z0) / TILE))) * count + Math.max(0, Math.min(count - 1, Math.floor((x - x0) / TILE)));
    // (For some tiles: the pads within reach of them, whose slabs, edges and neighbours' edges may fall in them.)
    let pads = all;
    if (dirty) {
      const reach = 2 + Math.max(0, ...all.map((p) => Math.max(p.w, p.d)));
      const boxes = [...dirty].map((k) => this.tileCorner(k)).map(([, , tx, tz]) => [tx - reach, tz - reach, tx + TILE + reach, tz + TILE + reach]);
      pads = all.filter((p) => boxes.some(([bx0, bz0, bx1, bz1]) => p.x < bx1 && p.x + p.w > bx0 && p.z < bz1 && p.z + p.d > bz0));
    }
    const geos = padGeometry(
      (x, z) => this.terrainAt(x, z),
      pads,
      groupOf,
      (x, z, y, out) => this.terrainColour(x, z, y, out),
      (x, z) => [(x - x0) / size, (z - z0) / size],
    );
    const lib = MaterialLibrary.for(app);
    const slab = (this.slabMaterial = lib.surface('material.ground', { vertexColors: true, variant: 'grass' }));
    grassGround(device, slab);
    // (The retaining edges in the houses' foundation stone: the soil's and the paving's textures are laid along xz,
    // in streaks up a wall.)
    const stone = lib.surface('material.wall.stone', { vertexColors: true, doubleSided: true });
    const made: pc.Entity[] = [];
    const gone: pc.Entity[] = [];
    for (const k of dirty ?? [...this.padGroups.keys(), ...geos.keys()]) {
      const had = this.padGroups.get(k);
      if (had) for (const e of [had.slab, had.edge]) if (e) gone.push(e);
      this.padGroups.delete(k);
      const g = geos.get(k);
      if (!g) continue;
      const slabs = meshFrom(device, g.slabs.meshData({ colors: true }));
      const edges = meshFrom(device, g.edges.meshData({ colors: true }));
      const drawn = { slab: slabs ? drawOf(app, parent, `pads${k}`, slabs, slab) : null, edge: edges ? drawOf(app, parent, `padEdges${k}`, edges, stone) : null };
      for (const e of [drawn.slab, drawn.edge]) if (e) made.push(e);
      this.padGroups.set(k, drawn);
    }
    const groups = [...this.padGroups.entries()].sort((a, b) => a[0] - b[0]).map(([, d]) => d);
    return { slabs: groups.flatMap((d) => (d.slab ? [d.slab] : [])), edges: groups.flatMap((d) => (d.edge ? [d.edge] : [])), made, gone };
  }
  /** The slabs' material (`padDraws`). */
  private slabMaterial: pc.StandardMaterial | null = null;
  /** The pad mask (`padDraws`). */
  private padTexture: pc.Texture | null = null;
  private groundMats: pc.StandardMaterial[] | null = null;

  /** Crown clusters gathered from the woods (for the report), 0 without them. */
  get crownCount(): number {
    return this.crowns?.count ?? 0;
  }

  /** The woods (`woods`), each model instanced, in groups by sector round the town. */
  private async forests(app: pc.AppBase, parent: pc.Entity, assets: AssetRegistry, pace?: () => Promise<void>): Promise<void> {
    const { near, far, sector } = await this.woods(pace);
    this.woodsSector = sector;
    // A big world's woods at altitude: crown clusters over every tree (`FarCrowns`), shown only in the town view.
    this.crowns?.destroy();
    this.crowns = null;
    if (this.built) {
      const trees: { kind: string; m: Float32Array }[] = [];
      for (const bucket of [near, far]) for (const [kind, list] of bucket) for (const m of list) trees.push({ kind, m });
      this.crowns = perfTime('nature: far crowns', () => new FarCrowns(app, parent, trees, sector));
    }
    for (const [bucket, casts] of [
      [near, true],
      [far, false],
    ] as const) {
      for (const [key, list] of bucket) {
        const lod = casts ? 0 : 1;
        this.batches.push(batch({ key, matrices: list, lod, cell: sector }, casts, casts ? 'woods' : 'farWoods', await instanced(app, parent, assets, key, list, lod, sector, pace)));
      }
    }
  }

  /** The lawn's flowers (`lawns`), instanced. */
  private async plantLawns(app: pc.AppBase, parent: pc.Entity, assets: AssetRegistry, world: WorldStructure, pace?: () => Promise<void>): Promise<void> {
    this.lawn = [];
    for (const { key, matrices, tiles, cell, clear } of await this.lawns(world, pace)) {
      const lod = this.lawnLod;
      const built = await instanced(app, parent, assets, key, matrices, lod, cell, pace);
      this.batches.push(batch({ key, matrices, lod, cell, tiles, clear }, false, 'flowers', built));
      this.lawn.push(...lawnLayers(built, matrices, tiles, clear));
    }
    this.refreshLawn(world);
  }
}

/** A big world's ground tiles (m), and their levels' quads across (the distances they change at are the preset's, `OpenWorldQuality.tiles`). */
const TILE = GROUND_TILE;
const TILE_LODS = [64, 32, 16];
/**
 * Ground tiles the render data worker is asked to shade ahead of the one whose meshes are being made
 * (a big world has about 144): enough that it goes on shading while the page is busy with the rest
 * of the build.
 */
const SHADE_AHEAD = 16;
/** How far (m) past a stroke's samples the ground is shaded again once it's done: the zones' soft edges and the beach lean on them. */
const LAND_DONE_PAD = 16;
/** The grass's colours made again this many metres (rows of texels) at a time once a stroke is done. */
const LAND_GRASS_STRIP = 12;
/** How far past the world's edge a height field's edge carries (`LandscapeLayout.ground`). */
const LAND_EDGE_FADE_M = 60;

/** A grid's heights (the y of its `positions`), into `into` if it is the size. */
function heightsOf(positions: Float32Array, into?: Float32Array): Float32Array {
  const out = into && into.length === positions.length / 3 ? into : new Float32Array(positions.length / 3);
  for (let v = 0; v < out.length; v++) out[v] = positions[v * 3 + 1];
  return out;
}

/** Colours 0..1 as bytes (a ground tile's kept colours: 18 KB a tile, not the 72 of floats). */
function bytesOf(colors: Float32Array): Uint8Array {
  const out = new Uint8Array(colors.length);
  for (let q = 0; q < colors.length; q++) out[q] = Math.max(0, Math.min(255, Math.round(colors[q] * 255)));
  return out;
}

/**
 * Where a ground tile's finest grid lies (`Landscape.tileFrame`, `tileCorner`): `S` quads across, `c` m apart, `n`
 * points across with the ring of one past it, its corner (`tx`, `tz`), and the landscape's square's far edges (`x1`,
 * `z1`), where its points stop (`tileGrid`). Point (col, row), each from -1, is `(row + 1) * n + col + 1`; row 0 at the
 * tile's far (max z) edge.
 */
interface TileGrid {
  S: number;
  c: number;
  n: number;
  tx: number;
  tz: number;
  x1: number;
  z1: number;
}

/**
 * A ground tile level's vertices, interleaved: position and normal (floats), colour (RGBA bytes, sRGB, as shaded) and
 * four bytes of nothing for the texture coordinates, 32 bytes a vertex. The ground's material lays its textures from
 * the position (`GAME_PLANAR_UV`), but its shader still declares the coordinates: without the stream WebGPU drew it
 * otherwise.
 */
const tileFormats = new WeakMap<pc.GraphicsDevice, pc.VertexFormat>();
function tileFormat(device: pc.GraphicsDevice): pc.VertexFormat {
  let format = tileFormats.get(device);
  if (!format) {
    format = new pc.VertexFormat(device, [
      { semantic: pc.SEMANTIC_POSITION, components: 3, type: pc.TYPE_FLOAT32 },
      { semantic: pc.SEMANTIC_NORMAL, components: 3, type: pc.TYPE_FLOAT32 },
      { semantic: pc.SEMANTIC_COLOR, components: 4, type: pc.TYPE_UINT8, normalize: true },
      { semantic: pc.SEMANTIC_TEXCOORD0, components: 4, type: pc.TYPE_UINT8, normalize: true },
    ]);
    tileFormats.set(device, format);
  }
  return format;
}

/** A level `segs` quads across: its grid's points and then each edge's again for the skirt. */
const levelVertices = (segs: number) => (segs + 1) * (segs + 1) + 4 * (segs + 1);
const levelIndices = (segs: number) => segs * segs * 6 + 4 * segs * 12;

/**
 * The indices of a level `segs` quads across (the same for every tile): one buffer per level and device, shared by
 * every tile's mesh of it, so never destroyed with one (however it goes: a level let go, a tile's draw with its
 * entity); kept for the device's life.
 */
const tileIndices = new WeakMap<pc.GraphicsDevice, Map<number, pc.IndexBuffer>>();
function tileIndexBuffer(device: pc.GraphicsDevice, segs: number): pc.IndexBuffer {
  let byLevel = tileIndices.get(device);
  if (!byLevel) tileIndices.set(device, (byLevel = new Map()));
  let ib = byLevel.get(segs);
  if (ib) return ib;
  const m = segs + 1;
  const idx = new Uint16Array(levelIndices(segs));
  let k = 0;
  for (let row = 0; row < segs; row++) {
    for (let cc = 0; cc < segs; cc++) {
      idx[k++] = cc + 1 + (row + 1) * m;
      idx[k++] = cc + 1 + row * m;
      idx[k++] = cc + row * m;
      idx[k++] = cc + (row + 1) * m;
      idx[k++] = cc + 1 + (row + 1) * m;
      idx[k++] = cc + row * m;
    }
  }
  // The skirt: each edge's points again (from `m * m`, an edge after another), joined to the edge both ways round.
  const edges = [(q: number) => q, (q: number) => segs * m + q, (q: number) => q * m, (q: number) => q * m + segs];
  edges.forEach((edge, e) => {
    const low = m * m + e * m;
    for (let q = 0; q + 1 < m; q++) {
      const [a, b, la, lb] = [edge(q), edge(q + 1), low + q, low + q + 1];
      idx.set([a, b, lb, a, lb, la, a, lb, b, a, la, lb], k);
      k += 12;
    }
  });
  ib = new pc.IndexBuffer(device, pc.INDEXFORMAT_UINT16, idx.length, pc.BUFFER_STATIC, idx.buffer);
  ib.destroy = () => {};
  byLevel.set(segs, ib);
  return ib;
}

/**
 * Writes level `segs` of a tile (every `S / segs`th point of its finest grid `g`, its `heights` and its `colours`)
 * into `data` (its vertex buffer's bytes, `tileFormat`): the rows of its grid from `r0` to `r1` (all, by default),
 * then its skirt, a cell and a metre down from its edges. Normals from the finest heights round each point. Colours
 * only with `colours` (else the buffer's kept).
 */
function writeTileLevel(data: ArrayBuffer, g: TileGrid, segs: number, heights: Float32Array, colours: Uint8Array | null, r0 = 0, r1 = segs): void {
  const { S, c, n, tx, tz, x1, z1 } = g;
  const step = S / segs;
  const m = segs + 1;
  const f = new Float32Array(data);
  const b = new Uint8Array(data);
  const inv = 1 / (2 * c);
  const write = (v: number, p: number, x: number, y: number, z: number) => {
    const o = v * 8;
    f[o] = x;
    f[o + 1] = y;
    f[o + 2] = z;
    // (Rows run towards -z: the slope along z is the other way round.)
    const dx = (heights[p + 1] - heights[p - 1]) * inv;
    const dz = (heights[p - n] - heights[p + n]) * inv;
    const l = 1 / Math.sqrt(dx * dx + 1 + dz * dz);
    f[o + 3] = -dx * l;
    f[o + 4] = l;
    f[o + 5] = -dz * l;
    if (colours) {
      const cb = v * 32 + 24;
      const p4 = p * 4;
      b[cb] = colours[p4];
      b[cb + 1] = colours[p4 + 1];
      b[cb + 2] = colours[p4 + 2];
      b[cb + 3] = colours[p4 + 3];
    }
  };
  for (let row = r0; row <= r1; row++) {
    const fr = row * step;
    const z = Math.min(z1, tz + (S - fr) * c);
    for (let cc = 0, v = row * m; cc < m; cc++, v++) {
      const fc = cc * step;
      write(v, (fr + 1) * n + fc + 1, Math.min(x1, tx + fc * c), heights[(fr + 1) * n + fc + 1], z);
    }
  }
  // The skirt: each edge's points again, lower.
  const drop = c * step + 1;
  const edges = [(q: number) => q, (q: number) => segs * m + q, (q: number) => q * m, (q: number) => q * m + segs];
  edges.forEach((edge, e) => {
    for (let q = 0, v = m * m + e * m; q < m; q++, v++) {
      const o = edge(q) * 8;
      f.copyWithin(v * 8, o, o + 6);
      f[v * 8 + 1] -= drop;
      b.copyWithin(v * 32 + 24, o * 4 + 24, o * 4 + 28);
    }
  });
}

/** A tile level's box: its square, from its lowest height less the skirt's drop to its highest. */
function tileBox(g: TileGrid, segs: number, heights: Float32Array): pc.BoundingBox {
  let [lo, hi] = [Infinity, -Infinity];
  for (let k = 0; k < heights.length; k++) {
    const h = heights[k];
    if (h < lo) lo = h;
    if (h > hi) hi = h;
  }
  lo -= g.c * (g.S / segs) + 1;
  const [x0, x1] = [g.tx, Math.min(g.x1, g.tx + g.S * g.c)];
  const [z0, z1] = [g.tz, Math.min(g.z1, g.tz + g.S * g.c)];
  const box = new pc.BoundingBox();
  box.setMinMax(new pc.Vec3(x0, lo, z0), new pc.Vec3(x1, hi, z1));
  return box;
}

/**
 * Level `segs` of a ground tile as a mesh (`writeTileLevel`), its indices the level's shared ones. `live`: its vertex
 * buffer keeps its bytes on the CPU, to be written into (`rewriteTileLevel`); else they go once uploaded (`gpuOnly`).
 */
function tileLevelMesh(device: pc.GraphicsDevice, g: TileGrid, segs: number, heights: Float32Array, colours: Uint8Array, live: boolean): pc.Mesh {
  const count = levelVertices(segs);
  const data = new ArrayBuffer(count * 32);
  writeTileLevel(data, g, segs, heights, colours);
  const vb = new pc.VertexBuffer(device, tileFormat(device), count, { usage: live ? pc.BUFFER_DYNAMIC : pc.BUFFER_STATIC, data });
  const mesh = new pc.Mesh(device);
  mesh.vertexBuffer = vb;
  mesh.indexBuffer[0] = tileIndexBuffer(device, segs);
  mesh.primitive[0] = { type: pc.PRIMITIVE_TRIANGLES, base: 0, baseVertex: 0, count: levelIndices(segs), indexed: true };
  mesh.aabb = tileBox(g, segs, heights);
  return live ? mesh : gpuOnly(mesh);
}

/**
 * Writes level `segs` of a ground tile into its live `mesh` (`tileLevelMesh`): the rows `r0` to `r1` of its grid and
 * its skirt (colours too with `colours`), only those bytes sent up. False (nothing written) if its vertices aren't
 * kept on the CPU.
 */
function rewriteTileLevel(mesh: pc.Mesh, g: TileGrid, segs: number, heights: Float32Array, colours: Uint8Array | null, r0 = 0, r1 = segs): boolean {
  const vb = mesh.vertexBuffer;
  const data = vb?.storage;
  if (!vb || !(data instanceof ArrayBuffer) || data.byteLength !== levelVertices(segs) * 32) return false;
  writeTileLevel(data, g, segs, heights, colours, r0, r1);
  const m = segs + 1;
  if (r0 === 0 && r1 === segs) vb.unlock();
  else {
    vb.unlock(r0 * m * 32, (r1 - r0 + 1) * m * 32);
    vb.unlock(m * m * 32, 4 * m * 32);
  }
  mesh.aabb = tileBox(g, segs, heights);
  return true;
}

/** `Landscape.waterInView`'s sphere round the sheet. */
const waterSphere = new pc.BoundingSphere();

/**
 * A tile's water above the sea's level (`pondQuads`) written into `into` where it fits (its buffers kept on the CPU,
 * room enough), else a mesh made for it with room to spare (a power of two), kept on the CPU: a brush's stamps over a
 * pond write into the one mesh.
 */
function pondMeshOf(device: pc.GraphicsDevice, quads: { positions: Float32Array; normals: Float32Array; indices: Uint32Array }, into: pc.Mesh | null): pc.Mesh {
  const verts = quads.positions.length / 3;
  const count = quads.indices.length;
  const vb = into?.vertexBuffer;
  const ib = into?.indexBuffer[0];
  let mesh = into;
  if (!mesh || !vb || !ib || !(vb.storage instanceof ArrayBuffer) || !(ib.storage instanceof ArrayBuffer) || vb.numVertices < verts || ib.numIndices < count) {
    const room = (k: number) => 2 ** Math.ceil(Math.log2(Math.max(64, k)));
    const format = pondFormats.get(device) ?? new pc.VertexFormat(device, [
      { semantic: pc.SEMANTIC_POSITION, components: 3, type: pc.TYPE_FLOAT32 },
      { semantic: pc.SEMANTIC_NORMAL, components: 3, type: pc.TYPE_FLOAT32 },
    ]);
    pondFormats.set(device, format);
    mesh = new pc.Mesh(device);
    mesh.vertexBuffer = new pc.VertexBuffer(device, format, room(verts), { usage: pc.BUFFER_DYNAMIC });
    mesh.indexBuffer[0] = new pc.IndexBuffer(device, pc.INDEXFORMAT_UINT32, room(count), pc.BUFFER_DYNAMIC);
  }
  const v = mesh.vertexBuffer!;
  const i = mesh.indexBuffer[0]!;
  const f = new Float32Array(v.storage as ArrayBuffer);
  for (let k = 0; k < verts; k++) {
    f[k * 6] = quads.positions[k * 3];
    f[k * 6 + 1] = quads.positions[k * 3 + 1];
    f[k * 6 + 2] = quads.positions[k * 3 + 2];
    f[k * 6 + 3] = quads.normals[k * 3];
    f[k * 6 + 4] = quads.normals[k * 3 + 1];
    f[k * 6 + 5] = quads.normals[k * 3 + 2];
  }
  new Uint32Array(i.storage as ArrayBuffer).set(quads.indices);
  v.unlock(0, verts * 24);
  i.unlock(0, count * 4);
  mesh.primitive[0] = { type: pc.PRIMITIVE_TRIANGLES, base: 0, baseVertex: 0, count, indexed: true };
  const box = new pc.BoundingBox();
  box.compute(quads.positions, verts);
  mesh.aabb = box;
  return mesh;
}
const pondFormats = new WeakMap<pc.GraphicsDevice, pc.VertexFormat>();

/** The pad mask's define on the terrain's material (`padDraws` sets each draw's own mask): an empty one until a draw has one. */
const padMasks = new WeakSet<pc.StandardMaterial>();
const noPads = new WeakMap<pc.GraphicsDevice, pc.Texture>();
function padMasked(device: pc.GraphicsDevice, material: pc.StandardMaterial): void {
  if (padMasks.has(material)) return;
  padMasks.add(material);
  let none = noPads.get(device);
  if (!none) noPads.set(device, (none = new pc.Texture(device, { name: 'noPads', width: 1, height: 1, format: pc.PIXELFORMAT_R8, mipmaps: false, levels: [new Uint8Array(1)] })));
  material.setDefine('GAME_PAD_MASK', '');
  material.setParameter('game_padMask', none);
  material.setParameter('game_padArea', new Float32Array(4));
  material.update();
}

/** An entity of `app` drawing `mesh` with `material`, switched off, under `parent`; it receives shadows and casts none. */
function drawOf(app: pc.AppBase, parent: pc.Entity, name: string, mesh: pc.Mesh, material: pc.Material): pc.Entity {
  const entity = new pc.Entity(name, app);
  entity.addComponent('render', { meshInstances: [new pc.MeshInstance(mesh, material)], castShadows: false, receiveShadows: true });
  entity.enabled = false;
  parent.addChild(entity);
  return entity;
}

/** A lawn layer's matrices as drawn now, up to its draws. */
function uploadLawn(layer: LawnLayer<Instances>): void {
  for (const draw of layer.draws) {
    draw.setMatrices(layer.shown);
    draw.update();
  }
}

/**
 * A model's meshes drawn instanced: the parts drawn whole in the game and the menus' town
 * (`shared`), and those also drawn in groups (see `SPLIT_TRIANGLES`): whole in the town (`whole`),
 * in groups in the game (`cells`; each keeps its instances in their order, `indices` into `matrices`).
 */
interface Instanced {
  shared: Instances[];
  whole: Instances[];
  cells: { draws: Instances[]; indices: number[] }[];
  /** The model's files that failed to load (`ModelTemplate.failed`): it's built again when one comes (`Landscape.rebuild`). */
  failed?: string[];
  /** The template they draw (theirs alone: it goes with them, `release`). */
  template?: ModelTemplate;
}

/** Takes down a model's plants: their draws, then the template's meshes. */
function release(built: Instanced, keep?: ReadonlySet<ModelTemplate | undefined>): void {
  for (const d of [...built.shared, ...built.whole, ...built.cells.flatMap((c) => c.draws)]) d.destroy();
  if (built.template && !keep?.has(built.template)) disposeTemplate(built.template);
}

/** One model's plants, as the landscape keeps them. */
interface Batch {
  /** Its model, and the level drawn (`instanced`'s). */
  key: string;
  lod: 0 | 1;
  built: Instanced;
  /** Its parts drawn whole cast shadows (the woods near the town). */
  casts: boolean;
  part: Exclude<NaturePart, 'grass'>;
  /** What it was built from, kept only while it's built without a file (`built.failed`), to build it again when that comes (`rebuild`). */
  again?: BatchSource;
}

/** `instanced`'s arguments for a model's plants. */
interface BatchSource {
  key: string;
  matrices: Float32Array[];
  lod: 0 | 1;
  cell?: (x: number, z: number) => string;
  /** The lawn's: each instance's town tile (-1 outside the town), for `refreshLawn`. */
  tiles?: readonly number[];
  /** The lawn's: they keep a tile clear of what hides the lawn (`LawnLayer.clear`). */
  clear?: boolean;
}

/** A batch of plants built from `source`, which it keeps only if it was built without a file (all of them are several MB a landscape). */
function batch(source: BatchSource, casts: boolean, part: Batch['part'], built: Instanced): Batch {
  return { key: source.key, lod: source.lod, built, casts, part, ...(built.failed ? { again: source } : {}) };
}

/** The draws of `batches`: which cast shadows, and which are drawn only in the game or only in the menus' town (see `NatureResult`). */
function gather(batches: readonly Batch[]): NatureResult {
  const draws: pc.Entity[] = [];
  const casters: pc.Entity[] = [];
  const only = new Map<pc.Entity, 'game' | 'town'>();
  for (const { built, casts } of batches) {
    const groups = built.cells.flatMap((cell) => cell.draws.map((d) => d.entity));
    const shared = built.shared.map((d) => d.entity);
    const whole = built.whole.map((d) => d.entity);
    draws.push(...shared, ...whole, ...groups);
    for (const e of whole) only.set(e, 'town');
    for (const e of groups) only.set(e, 'game');
    if (casts) casters.push(...shared, ...whole);
  }
  return { draws, casters, only };
}

/** The lawn layers of a model's tufts or flowers: every instance, in the parts drawn whole; then each group's. */
function lawnLayers(built: Instanced, matrices: readonly Float32Array[], tiles: readonly number[], clear = false): LawnLayer<Instances>[] {
  const layers: LawnLayer<Instances>[] = [];
  for (const { draws, indices } of [{ draws: [...built.shared, ...built.whole], indices: matrices.map((_, k) => k) }, ...built.cells]) {
    if (!draws.length) continue;
    layers.push(lawnLayer(draws, matrices, tiles, indices, clear));
  }
  return layers;
}

/**
 * One template per model key, drawn instanced; static, so the boxes are worked out once. `cell`
 * groups the instances by where they stand: the parts with enough triangles get a draw per group
 * (sharing the template's mesh and material), which the game's camera leaves out when out of view.
 */
async function instanced(
  app: pc.AppBase,
  parent: pc.Entity,
  assets: AssetRegistry,
  key: string,
  matrices: Float32Array[],
  lod: 0 | 1 = 0,
  cell?: (x: number, z: number) => string,
  pace?: () => Promise<void>,
  made?: ModelTemplate,
): Promise<Instanced> {
  if (matrices.length === 0) return { shared: [], whole: [], cells: [] };
  await pace?.();
  // Without its file, nothing, until the file comes (`Landscape.rebuild`). (`made`: the template built already, its plants grown again.)
  const template = made ?? (await perfTimeAsync(`nature: model ${key}`, () => buildModel(app, key, assets.get(key, 'model'), [1, 1], { lod })));
  await pace?.();
  const groups = cell ? groupInstances(matrices, (m) => cell(m[12], m[14])) : new Map<string, number[]>();
  const split = groups.size > 1 ? template.meshes.filter((m) => splitsByGroup(m, matrices.length)) : [];
  const span = perfStart(`nature: instances ${key}`);
  const draw = (m: ModelMesh, indices: readonly number[], name: string): Instances => {
    const d = new Instances(app.graphicsDevice, parent, m.mesh, m.material, { name, castShadows: false, receiveShadows: true, capacity: indices.length });
    d.setCount(indices.length);
    for (let i = 0; i < indices.length; i++) d.setMatrix(i, matrices[indices[i]]);
    d.update();
    windStill(d.meshInstance, m);
    d.enabled = false;
    return d;
  };
  const all = matrices.map((_, k) => k);
  const whole = template.meshes.map((m) => draw(m, all, m.name));
  await pace?.();
  const cells: Instanced['cells'] = [];
  if (split.length) {
    let c = 0;
    for (const indices of groups.values()) {
      cells.push({ draws: split.map((m) => draw(m, indices, `${m.name}#${c}`)), indices });
      c++;
      await pace?.();
    }
  }
  perfEnd(span);
  return {
    shared: whole.filter((_, i) => !split.includes(template.meshes[i])),
    whole: whole.filter((_, i) => split.includes(template.meshes[i])),
    cells,
    template,
    ...(template.failed ? { failed: template.failed } : {}),
  };
}

/** Whether a mesh drawn `copies` times is drawn per draw group (where there are groups): `SPLIT_TRIANGLES` or more triangles over them all. As the street's (`pc/street.ts`). */
export function splitsByGroup(m: ModelMesh, copies: number): boolean {
  // (Its indices drawn, three a triangle.)
  return ((m.mesh.primitive[0]?.count ?? 0) / 3) * copies >= SPLIT_TRIANGLES;
}

// ---- Wind and foliage shading -----------------------------------------------------------

/**
 * The wind (the port of `NatureWindPlugin`), on parts on the `bark` and `foliage` finishes: a slow
 * height-weighted bend with gusts plus leaf flutter, phased per instance by where it stands (its
 * matrix's translation; an instance's scale and turn are taken out of the flutter and the bend's
 * direction), and, on foliage, a small per-instance colour
 * variation so repeated trees don't look stamped. Short plants hardly move. The vertex's local
 * position (`vsLocal`), so the shadow pass could take it too; it doesn't: trees cast still shadows
 * (the sun's map is drawn again only when something in it changes, `sunShadows.ts`).
 *
 * - `GAME_WIND`: `game_wind` (global: time, strength, direction x, z), `game_windKind` (the
 *   material's: `WIND_KIND`), `game_windFlutter` (1, and 0 on a draw of a part grown from nothing,
 *   `windStill`: flat at rest, it stays hidden).
 * - `GAME_WIND_TINT`: the foliage's tint, on its vertex colour (an sRGB factor, so here its 2.2 power on the linear colour).
 *
 * No instance stream of its own: it reads only the instance's matrix (`instance_line1..4`).
 */
/**
 * The leaves' colour from plant to plant and from their tops to their undersides (sRGB factors on the foliage's
 * colour, `GAME_WIND_TINT`): `warm` to `cool`, the two ends a plant's hue is drawn between by where it stands, and
 * `value`, how light it is (from, to); `top` and `under`, what faces the sky and what faces the ground (`facing`:
 * the normal's height where it is all `under` and all `top`; `petal`: the linear colour from which a part is a
 * petal, not a leaf, and to which it is all petal, which keeps its colour from plant to plant and under `top`;
 * `needle`: the blue of a green (linear blue over green) from which it is a conifer's and to which it is all one, which
 * takes `needles` less of `top`; `night`: what `top` is by night, a grey, `game_daylight` 0: the warm tops were the
 * one thing the night's grade, which spares what is warm for the lamps' sake, left yellow-green); `petalNight`: how
 * much of a petal's colour goes to grey by night, for the same reason (a patch of buttercups stayed bright yellow
 * under the moon). From the play camera a crown, a shrub and a hedge
 * are seen from above: lit only as the sun fell on them they were one dark green, top and side.
 */
const LEAVES = tuned('leaves', {
  warm: [1.14, 1.04, 0.76],
  cool: [0.86, 0.98, 1.07],
  value: [0.86, 1.1],
  top: [1.42, 1.38, 1.1],
  under: 0.8,
  facing: [-0.5, 0.9],
  petal: [0.3, 0.7],
  needle: [0.24, 0.36],
  needles: 0.75,
  night: 1.2,
  petalNight: 0.85,
});

function windCode(lang: 'glsl' | 'wgsl'): ShaderFeature['glsl'] {
  const { wgsl, v2, v3, u, decl } = codeFor(lang);
  const f = float;
  const { gust: g, phase, sway } = WIND;
  const local = `
#if defined(GAME_WIND) && !defined(SHADOW_PASS)
	{
		${decl('windInst', 'vec3', true)} = ${v3}(0.0);
		${decl('windLw', 'vec3', true)} = ${v3}(${u('game_wind')}.z, 0.0, ${u('game_wind')}.w);
		// The instance's scale along each axis: flutter is in world metres, however far a moving part is scaled
		// (a part grown from nothing is 1000x along its height once grown).
		${decl('windScl', 'vec3', true)} = ${v3}(1.0);
		#ifdef INSTANCING
			windInst = instance_line4.xyz;
			${decl('windS2', 'float')} = max(dot(instance_line1.xyz, instance_line1.xyz), 0.0001);
			windLw = ${v3}(dot(windLw, instance_line1.xyz), 0.0, dot(windLw, instance_line3.xyz)) / windS2;
			windScl = max(${v3}(length(instance_line1.xyz), length(instance_line2.xyz), length(instance_line3.xyz)), ${v3}(0.0001));
		#endif
		${decl('windT', 'float')} = ${u('game_wind')}.x;
		${decl('windPh', 'float')} = dot(windInst.xz, ${v2}(${f(phase[0])}, ${f(phase[1])}));
		${decl('windH', 'float')} = max(localPos.y, 0.0);
		${decl('windGust', 'float')} = (${f(g.base)} + ${f(g.amp)} * sin(windT * ${f(g.rate)} + windPh * ${f(g.phase)})) * ${u('game_wind')}.y;
		${decl('windSway', 'float')} = (${sway.map((s) => `sin(windT * ${f(s.rate)} + windPh * ${f(s.phase)}) * ${f(s.weight)}`).join(' + ')}) * windGust;
		${decl('windKind', 'vec4')} = ${u('game_windKind')};
		${decl('windFp', 'float')} = dot(localPos, ${v3}(3.1, 1.7, 2.3)) + windPh * 3.0;
		${decl('windFl', 'float')} = windKind.y * ${u('game_windFlutter')} * min(windH * windKind.w, 1.0) * windGust;
		localPos = localPos + windLw * (windSway * windKind.x * windH * windH) + ${v3}(sin(windT * windKind.z + windFp), 0.6 * sin(windT * windKind.z * 1.3 + windFp * 1.3), cos(windT * windKind.z * 0.9 + windFp)) * windFl / windScl;
	}
#endif`;
  const color = wgsl ? 'output.vVertexColor' : 'vVertexColor';
  const end = `
#if defined(GAME_WIND_TINT) && defined(VERTEX_COLOR) && !defined(SHADOW_PASS)
	{
		${decl('tintInst', 'vec3', true)} = ${v3}(0.0);
		#ifdef INSTANCING
			tintInst = instance_line4.xyz;
		#endif
		${decl('tintRnd', 'float')} = fract(sin(dot(tintInst.xz, ${v2}(12.9898, 78.233))) * 43758.5453);
		// (Leaves only: a petal, light already, keeps its colour.)
		${decl('tintLeaf', 'float')} = 1.0 - smoothstep(${f(LEAVES.petal[0])}, ${f(LEAVES.petal[1])}, max(${color}.r, max(${color}.g, ${color}.b)));
		${decl('tint', 'vec3', true)} = mix(${v3}(1.0), mix(${v3}(${LEAVES.warm.map(f).join(', ')}), ${v3}(${LEAVES.cool.map(f).join(', ')}), tintRnd) * (${f(LEAVES.value[0])} + ${f(LEAVES.value[1] - LEAVES.value[0])} * fract(tintRnd * 7.31)), tintLeaf);
		#ifdef NORMALS
			// Lit from above: what faces the sky lighter and a little warmer, what faces the ground darker.
			${decl('tintUp', 'float')} = smoothstep(${f(LEAVES.facing[0])}, ${f(LEAVES.facing[1])}, normalize(${wgsl ? 'output.vNormalW' : 'vNormalW'}).y);
			// (Less on a conifer's blue-green needles, which it turned to mint: by how blue the green is.)
			${decl('tintNeedle', 'float')} = smoothstep(${f(LEAVES.needle[0])}, ${f(LEAVES.needle[1])}, ${color}.b / max(${color}.g, 0.0001));
			tint = tint * mix(${v3}(${f(LEAVES.under)}), mix(${v3}(1.0), mix(${v3}(${f(LEAVES.night)}), ${v3}(${LEAVES.top.map(f).join(', ')}), ${u('game_daylight')}), tintLeaf * (1.0 - ${f(LEAVES.needles)} * tintNeedle)), tintUp);
		#endif
		${color} = ${wgsl ? 'vec4f' : 'vec4'}(${color}.rgb * pow(tint, ${v3}(2.2)), ${color}.a);
		// (By night a petal's colour goes most of the way to its grey: the night's grade spares what is warm.)
		${color} = ${wgsl ? 'vec4f' : 'vec4'}(mix(${color}.rgb, ${v3}(dot(${color}.rgb, ${v3}(0.2126, 0.7152, 0.0722))), (1.0 - tintLeaf) * (1.0 - ${u('game_daylight')}) * ${f(LEAVES.petalNight)}), ${color}.a);
	}
#endif`;
  const vsDecl = wgsl
    ? `
#ifdef GAME_WIND
	uniform game_wind: vec4f;
	uniform game_windKind: vec4f;
	uniform game_windFlutter: f32;
	uniform game_daylight: f32;
#endif`
    : `
#ifdef GAME_WIND
	uniform vec4 game_wind;
	uniform vec4 game_windKind;
	uniform float game_windFlutter;
	uniform float game_daylight;
#endif`;
  return { vsDecl, vsLocal: local, vsEnd: end };
}

/** The plants' wind and tint (see `windCode`), at its place among the shader features (`chunks.ts`: lower runs first). */
export const NATURE_WIND: ShaderFeature = { name: 'natureWind', priority: 230, glsl: windCode('glsl'), wgsl: windCode('wgsl') };

// (Before any device's chunks are built: they're built with every feature registered, once per device.)
registerShaderFeature(NATURE_WIND);

/** The wind of an app's plants: its clock (seconds, advancing with the frames) and strength (1 as made; 0 still, for reduce motion). */
interface WindState {
  time: number;
  strength: number;
  /** The water's clock (s, `shaders/water.ts`), and how fast it goes (`WATER_CALM` with Reduce motion). */
  water: number;
  waterRate: number;
}

/** With Reduce motion the water runs at this share of its speed, and its second still layer doesn't drift. */
const WATER_CALM = 0.25;

const windStates = new WeakMap<pc.AppBase, WindState>();

/**
 * Hooks the plant finishes as the library makes them (whoever builds the first plant): the wind,
 * and the tint on foliage. Also advances the wind clock with the app's frames and hands it to the
 * shaders. Call once per app, before any model loads.
 */
export function installNature(app: pc.AppBase): void {
  if (windStates.has(app)) return;
  const state: WindState = { time: 0, strength: 1, water: 0, waterRate: 1 };
  windStates.set(app, state);
  daylightByDefault(app.graphicsDevice);
  MaterialLibrary.for(app).onCreate((key, material) => {
    const kind = WIND_KIND[key];
    if (!kind) return;
    material.setDefine('GAME_WIND', '');
    if (key !== 'material.finish.bark') material.setDefine('GAME_WIND_TINT', '');
    // No contact shading among leaves and twigs, and none of them in the depth it is made from (`post.ts`).
    withoutContact(material);
    material.setParameter('game_windKind', new Float32Array(kind));
    material.setParameter('game_windFlutter', 1);
  });
  const id = app.graphicsDevice.scope.resolve('game_wind');
  const value = new Float32Array([0, 1, WIND.dir[0], WIND.dir[1]]);
  const waterId = app.graphicsDevice.scope.resolve('game_waterClock');
  const water = new Float32Array(4);
  app.on('update', (dt: number) => {
    state.time += Math.min(0.1, dt);
    state.water += Math.min(0.1, dt) * state.waterRate;
  });
  app.on('prerender', () => {
    value[0] = state.time;
    value[1] = state.strength;
    id.setValue(value);
    water[0] = state.water;
    water[1] = state.waterRate === 1 ? 1 : 0;
    waterId.setValue(water);
  });
}

/** The water calmed (Reduce motion: `WATER_CALM` of its speed, one still layer drifting) or not. */
export function setWaterCalm(app: pc.AppBase, calm: boolean): void {
  const state = windStates.get(app);
  if (state) state.waterRate = calm ? WATER_CALM : 1;
}

/** A draw of model mesh `m` (on a plant finish) bends without fluttering when it's a part grown from nothing (`ModelMesh.windStill`). */
export function windStill(meshInstance: pc.MeshInstance, m: ModelMesh): void {
  if (m.windStill) meshInstance.setParameter('game_windFlutter', 0);
}

/** How strongly the wind moves an app's plants (1 as made; 0 still, for reduce motion and `?still`). */
export function setWind(app: pc.AppBase, strength: number): void {
  const state = windStates.get(app);
  if (state) state.strength = strength;
}

/** The gusts' noise for `windGust` (`gustNoise`: the texture the grass reads, on the CPU), made the first time it is asked for. */
let gusts: Uint8Array | null = null;

/**
 * The gust over the ground at (`x`, `z`) now, 0 (calm) .. 1 (a strong gust), as the grass shows it (`gustAt`
 * at the wind's own time, times its strength): what the wind in the leaves sounds by. Zero before the app's
 * wind exists, and with the wind still.
 */
export function windGust(app: pc.AppBase, x: number, z: number): number {
  const state = windStates.get(app);
  if (!state || state.strength <= 0) return 0;
  return gustAt((gusts ??= gustNoise()), state.time, x, z) * state.strength;
}

/**
 * How far the wind bends a plant standing at (`px`, `pz`) now, per m² of height up its model (m,
 * along `windDirection`): what sits on a branch sways with it. Zero before the app's wind exists.
 */
export function windSway(app: pc.AppBase, px: number, pz: number): number {
  const state = windStates.get(app);
  return state ? windSwayAt(state.time, state.strength, px, pz) : 0;
}

// ---- Sky ------------------------------------------------------------------------------

/** The dome's sphere: 32 segments (`sphereData`). */
const SKY_SEGMENTS = 32;

/**
 * Gradient sky dome with a glow around the sun (colours follow the time of day via vertex
 * colours) and a band of drifting clouds (`image.clouds`). Both follow the camera (`follow`), so
 * they are always as far away, unlit and out of the fog.
 *
 * Colours as the look was tuned with: the sky's colour is taken for gamma and linearised before the
 * image processing, so the dome's linear vertex colours are linearised once more here, and the
 * clouds' emissive colour is handed over as an sRGB one (PlayCanvas linearises it).
 */
export class Sky {
  readonly dome: pc.Entity;
  readonly band: pc.Entity | null = null;
  private readonly mesh: pc.Mesh;
  private readonly dirs: Float32Array;
  private readonly colors: Float32Array;
  private readonly cloudMaterial: pc.StandardMaterial | null = null;
  private readonly tmp = { r: 0, g: 0, b: 0 };
  private readonly glow = { r: 0, g: 0, b: 0 };
  private spin = 0;

  constructor(app: pc.AppBase, parent: pc.Entity, private readonly radius: number, cloudsUrl?: string) {
    const device = app.graphicsDevice;
    const { positions, indices } = sphereData(radius * 2, radius * 2, radius * 2, SKY_SEGMENTS);
    this.dirs = new Float32Array(positions.length);
    for (let i = 0; i < positions.length; i++) this.dirs[i] = positions[i] / radius;
    this.colors = new Float32Array((positions.length / 3) * 4);
    const mesh = (this.mesh = new pc.Mesh(device));
    mesh.setPositions(positions);
    mesh.setColors(this.colors);
    mesh.setIndices(indices);
    mesh.update(pc.PRIMITIVE_TRIANGLES);
    const mat = new pc.StandardMaterial();
    mat.name = 'sky';
    mat.useLighting = false;
    mat.useSkybox = false;
    mat.useFog = false;
    mat.diffuse.set(0, 0, 0);
    mat.emissive.set(1, 1, 1);
    mat.emissiveVertexColor = true;
    mat.cull = pc.CULLFACE_NONE;
    mat.update();
    this.dome = new pc.Entity('sky', app);
    this.dome.addComponent('render', { meshInstances: [new pc.MeshInstance(mesh, mat)], castShadows: false, receiveShadows: false });
    // (It moves with the camera, which is always inside it.)
    this.dome.render!.meshInstances[0].cull = false;
    parent.addChild(this.dome);

    if (cloudsUrl) {
      // A slightly tapered open cylinder spanning ~3..32 degrees above the horizon: 48 sides, its
      // faces doubled (each drawn twice, uncull'd, so the clouds blend in twice).
      const r = radius * 0.94;
      const [bottom, top, height, sides] = [r, r * 0.85, r * 0.55, 48];
      const p: number[] = [];
      const uv: number[] = [];
      for (let i = 0; i <= 1; i++) {
        const rr = i ? top : bottom;
        for (let j = 0; j <= sides; j++) {
          const a = (j * Math.PI * 2) / sides;
          p.push(Math.cos(-a) * rr, -height / 2 + i * height + r * 0.31, Math.sin(-a) * rr);
          // (`uScale` 2 baked in; the band's v runs up it, PlayCanvas's from the image's top.)
          uv.push((j / sides) * 2, 1 - i);
        }
      }
      const idx: number[] = [];
      for (let j = 0; j < sides; j++) {
        const [a, b, c, d] = [j, j + 1, j + sides + 1, j + sides + 2];
        idx.push(c, b, a, c, d, b);
      }
      const n = p.length / 3;
      const band = new pc.Mesh(device);
      band.setPositions([...p, ...p]);
      band.setUvs(0, [...uv, ...uv]);
      band.setIndices([...idx, ...idx.map((k) => k + n)]);
      band.update(pc.PRIMITIVE_TRIANGLES);
      const cm = (this.cloudMaterial = new pc.StandardMaterial());
      cm.name = 'clouds';
      cm.useLighting = false;
      cm.useSkybox = false;
      cm.useFog = false;
      cm.diffuse.set(0, 0, 0);
      cm.emissive.set(0, 0, 0);
      cm.blendType = pc.BLEND_NORMAL;
      cm.depthWrite = false;
      cm.cull = pc.CULLFACE_NONE;
      cm.opacity = 0;
      cm.update();
      void textureWithFallback(app, cloudsUrl).then((tex) => {
        if (!tex) return;
        tex.addressU = pc.ADDRESS_REPEAT;
        tex.addressV = pc.ADDRESS_CLAMP_TO_EDGE;
        cm.emissiveMap = tex;
        cm.opacityMap = tex;
        cm.opacityMapChannel = 'a';
        cm.opacity = 1;
        cm.update();
      });
      const entity = (this.band = new pc.Entity('clouds', app));
      entity.addComponent('render', { meshInstances: [new pc.MeshInstance(band, cm)], castShadows: false, receiveShadows: false });
      entity.render!.meshInstances[0].cull = false;
      this.dome.addChild(entity);
    }
  }

  /** Puts the sky round the camera (game space: its eye), so it never comes nearer. */
  follow(x: number, y: number, z: number): void {
    this.dome.setLocalPosition(x, y, z);
  }

  /**
   * The dome's radius (m; the clouds' band with it): it looks the same at any (its colours go by direction, out
   * of the fog), and hides whatever lies further off, so the town view, which sees further, moves it out.
   */
  reach(radius: number): void {
    const scale = radius / this.radius;
    if (Math.abs(this.dome.getLocalScale().x - scale) > 1e-4) this.dome.setLocalScale(scale, scale, scale);
  }

  /** Slow cloud drift; call per frame with the frame's milliseconds. */
  drift(dt: number): void {
    if (!this.band) return;
    this.spin += dt * 0.000004;
    // (A turn about y in game space: degrees, the world root's mirror taken by the parent.)
    this.band.setLocalEulerAngles(0, (this.spin * 180) / Math.PI, 0);
  }

  /** Horizon = fog/clear colour, zenith = deeper sky colour, plus a glow around the sun. */
  update(l: Lighting): void {
    const sx = -l.sunDirection.x;
    const sy = -l.sunDirection.y;
    const sz = -l.sunDirection.z;
    // Strongest at golden hour, faint at noon, a dim halo around the moon at night.
    const strength = Math.min(1, l.sunIntensity / 2.5) * (0.35 + 0.65 * (1 - l.sunUp));
    const n = this.dirs.length / 3;
    const tmp = this.tmp;
    const glow = this.glow;
    const z = l.zenithColor;
    const c = l.clearColor;
    for (let i = 0; i < n; i++) {
      const dx = this.dirs[i * 3];
      const dy = this.dirs[i * 3 + 1];
      const dz = this.dirs[i * 3 + 2];
      const t = smoothstep(0.02, 0.75, dy);
      tmp.r = c.r + (z.r - c.r) * t;
      tmp.g = c.g + (z.g - c.g) * t;
      tmp.b = c.b + (z.b - c.b) * t;
      const d = Math.max(0, dx * sx + dy * sy + dz * sz);
      const g = (Math.pow(d, 6) * 0.55 + Math.pow(d, 48) * 0.6) * strength * (dy > -0.05 ? 1 : 0);
      glow.r = l.sunColor.r * g;
      glow.g = l.sunColor.g * g;
      glow.b = l.sunColor.b * g;
      // The linear colour, linearised again (see the class).
      this.colors[i * 4] = toLinear(toLinear(tmp.r + glow.r));
      this.colors[i * 4 + 1] = toLinear(toLinear(tmp.g + glow.g));
      this.colors[i * 4 + 2] = toLinear(toLinear(tmp.b + glow.b));
      this.colors[i * 4 + 3] = 1;
    }
    this.mesh.setColors(this.colors);
    this.mesh.update(pc.PRIMITIVE_TRIANGLES, false);
    const cm = this.cloudMaterial;
    if (cm) {
      // Lit side of the clouds: sky-bright by day, sunset-tinted at golden hour, dim at night.
      const k = 0.35 + 0.25 * (1 - l.sunUp);
      const s = Math.min(1.15, 0.25 + l.sunIntensity * 0.28);
      cm.emissive.set(toLinear((c.r + (l.sunColor.r - c.r) * k) * s), toLinear((c.g + (l.sunColor.g - c.g) * k) * s), toLinear((c.b + (l.sunColor.b - c.b) * k) * s));
      cm.update();
    }
  }

  destroy(): void {
    this.dome.destroy();
  }
}


// ---- the far woods at altitude (crown clusters past the near woods) ----

/** The trees gathered into one crown cluster per square this wide (m). */
const CROWN_CELL = 9;
/** In the full town view the woods draw their own trees only in squares within this of the camera's target (m); crown clusters elsewhere. */
const TOWN_WOODS_NEAR = 260;
/** The crowns' colours (sRGB): broadleaves, birches, conifers and fruit trees, as the woods' leaves read from afar. */
const CROWN = { leaf: '#4F6E33', birch: '#6E8A42', needle: '#36503C', fruit: '#5C7A36' } as const;

/** A crown cluster's mesh: a low dome over a trunk's gap (unit: radius 1, from 0.3 to 1 up), smooth-shaded; a conifer's a cone. */
function crownMesh(cone: boolean): Geo {
  // (White at 0.45 alpha, the cutaway's keep-everything: what its vertices were given one by one.)
  const g = new Geo().color('#FFFFFF', 0.45);
  const n = 10;
  // Rings up the crown (height, radius), and the dome's middle its normals point out from.
  const rings: [number, number][] = cone ? [[0.22, 0.6], [0.5, 0.42], [0.78, 0.2], [1, 0]] : [[0.3, 0.7], [0.48, 0.97], [0.7, 0.88], [0.88, 0.55], [1, 0]];
  const mid = cone ? 0.35 : 0.5;
  const squash = cone ? 0.5 : 1.4;
  const vertex = (x: number, y: number, z: number, nx: number, ny: number, nz: number) => {
    const l = Math.hypot(nx, ny, nz) || 1;
    return g.point([x, y, z], [nx / l, ny / l, nz / l], 0, 0);
  };
  const ids = rings.map(([y, r]) =>
    Array.from({ length: n }, (_, k) => {
      const a = (k / n) * Math.PI * 2;
      const [x, z] = [Math.cos(a) * r, Math.sin(a) * r];
      return vertex(x, y, z, Math.cos(a) * Math.max(r, 0.3), (y - mid) * squash + (r === 0 ? 1 : 0), Math.sin(a) * Math.max(r, 0.3));
    }),
  );
  // (Wound as `Geo` winds a face towards its normal, `(a - b) × (c - b)`: out.)
  for (let i = 0; i + 1 < ids.length; i++) {
    for (let k = 0; k < n; k++) {
      const [a, b, c, d] = [ids[i][k], ids[i][(k + 1) % n], ids[i + 1][(k + 1) % n], ids[i + 1][k]];
      g.face(a, b, c).face(a, c, d);
    }
  }
  // Its underside, flat.
  const [yb] = rings[0];
  const centre = vertex(0, yb, 0, 0, -1, 0);
  for (let k = 0; k < n; k++) g.face(centre, ids[0][(k + 1) % n], ids[0][k]);
  return g;
}

/** A crown cluster as gathered: where it stands, how wide and tall, its colour; by woods square. */
interface Crown {
  key: string;
  cone: boolean;
  m: Float32Array;
  color: [number, number, number, number];
}

/**
 * The far woods at altitude: every woods tree gathered into crown clusters (`CROWN_CELL` m squares, one
 * cluster each: its trees' middle, as wide as they spread, as tall as they are on average, broadleaf or
 * conifer by most of them), one instanced draw each of a dome and a cone. Shown only in the town view, in
 * the woods' squares that don't draw their own trees then (`show`).
 */
export class FarCrowns {
  /** Its draws, made the first time it shows any (nothing of it before the town view: `TownBuildings.made`). */
  private draws: [Instances, Instances] | null = null;
  private readonly meshes: pc.Mesh[] = [];
  private readonly byKey = new Map<string, Crown[]>();
  private shownKeys = '';

  constructor(
    private readonly app: pc.AppBase,
    private readonly parent: pc.Entity,
    trees: Iterable<{ kind: string; m: Float32Array }>,
    sector: (x: number, z: number) => string,
  ) {
    // Gathered by square.
    const cells = new Map<string, { n: number; x: number; y: number; z: number; s: number; x0: number; x1: number; z0: number; z1: number; cone: number; birch: number; fruit: number }>();
    for (const { kind, m } of trees) {
      if (kind === 'model.rock' || kind.startsWith('model.bush')) continue;
      const [x, y, z] = [m[12], m[13], m[14]];
      const s = Math.hypot(m[0], m[1], m[2]);
      const key = `${Math.floor(x / CROWN_CELL)},${Math.floor(z / CROWN_CELL)}`;
      let c = cells.get(key);
      if (!c) cells.set(key, (c = { n: 0, x: 0, y: 0, z: 0, s: 0, x0: x, x1: x, z0: z, z1: z, cone: 0, birch: 0, fruit: 0 }));
      c.n++;
      c.x += x;
      c.y += y;
      c.z += z;
      c.s += s;
      [c.x0, c.x1, c.z0, c.z1] = [Math.min(c.x0, x), Math.max(c.x1, x), Math.min(c.z0, z), Math.max(c.z1, z)];
      if (kind === 'model.pine' || kind === 'model.spruce') c.cone++;
      else if (kind === 'model.tree.birch') c.birch++;
      else if (kind === 'model.tree.fruit') c.fruit++;
    }
    let i = 0;
    for (const c of cells.values()) {
      const [x, y, z, s] = [c.x / c.n, c.y / c.n, c.z / c.n, c.s / c.n];
      const cone = c.cone * 2 > c.n;
      // As wide as its trees spread and a crown more; as tall as a tree of their size (a conifer taller, slimmer).
      const h = (cone ? 10.5 : 8.5) * s;
      const rx = Math.min(CROWN_CELL * 0.75, (c.x1 - c.x0) / 2 + (cone ? 1.8 : 2.6) * s);
      const rz = Math.min(CROWN_CELL * 0.75, (c.z1 - c.z0) / 2 + (cone ? 1.8 : 2.6) * s);
      const yaw = ((i++ * 2.399) % (Math.PI * 2));
      const [cs, sn] = [Math.cos(yaw), Math.sin(yaw)];
      const m = new Float32Array(16);
      m[0] = cs * rx;
      m[2] = -sn * rx;
      m[5] = h;
      m[8] = sn * rz;
      m[10] = cs * rz;
      m[12] = x;
      m[13] = y;
      m[14] = z;
      m[15] = 1;
      const base = cone ? CROWN.needle : c.birch * 2 > c.n ? CROWN.birch : c.fruit * 2 > c.n ? CROWN.fruit : CROWN.leaf;
      const v = parseInt(base.slice(1), 16);
      const k = 0.9 + ((i * 0.6180339) % 1) * 0.2;
      const color: Crown['color'] = [Math.min(1, (((v >> 16) & 255) / 255) * k), Math.min(1, (((v >> 8) & 255) / 255) * k), Math.min(1, ((v & 255) / 255) * k), 1];
      const key = sector(x, z);
      let list = this.byKey.get(key);
      if (!list) this.byKey.set(key, (list = []));
      list.push({ key, cone, m, color });
    }
  }

  /** Crowns in total (for the report). */
  get count(): number {
    let n = 0;
    for (const l of this.byKey.values()) n += l.length;
    return n;
  }

  private made(): [Instances, Instances] {
    if (this.draws) return this.draws;
    const { app, parent } = this;
    const lib = MaterialLibrary.for(app);
    const material = lib.instanceColored(lib.plainFinish());
    return (this.draws = [false, true].map((cone) => {
      const mesh = meshFrom(app.graphicsDevice, crownMesh(cone).meshData({ colors: true }))!;
      mesh.incRefCount();
      this.meshes.push(mesh);
      const d = new Instances(app.graphicsDevice, parent, mesh, material, { name: cone ? 'farCrowns.cone' : 'farCrowns.dome', streams: [INSTANCE_COLOR], castShadows: false, receiveShadows: true, capacity: 1024 });
      lib.excludeFromLights([], [d.meshInstance]);
      d.enabled = false;
      return d;
    }) as [Instances, Instances]);
  }

  /** Shows the crowns of the squares `far` says (none: hidden), written again only when that set changes. */
  show(far: ((key: string) => boolean) | null): void {
    const keys = far ? [...this.byKey.keys()].filter(far).sort() : [];
    const sig = keys.join('|');
    if (sig === this.shownKeys) return;
    this.shownKeys = sig;
    const [dome, cone] = this.made();
    const lists = keys.map((k) => this.byKey.get(k)!);
    let nd = 0;
    let nc = 0;
    for (const l of lists) for (const c of l) c.cone ? nc++ : nd++;
    dome.setCount(nd);
    cone.setCount(nc);
    let [id, ic] = [0, 0];
    for (const l of lists) {
      for (const c of l) {
        const d = c.cone ? cone : dome;
        const i = c.cone ? ic++ : id++;
        d.setMatrix(i, c.m);
        d.set(INSTANCE_COLOR, i, c.color);
      }
    }
    for (const d of [dome, cone]) {
      d.update();
      d.enabled = d.count > 0;
    }
  }

  destroy(): void {
    for (const d of this.draws ?? []) d.destroy();
    for (const mesh of this.meshes) {
      mesh.decRefCount();
      if (mesh.refCount === 0) mesh.destroy();
    }
  }
}
