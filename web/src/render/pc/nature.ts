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
import { CountryRoads } from '../shared/countryRoads';
import type { Lighting } from '../shared/environment';
import { grassField, gustAt, gustNoise, type GrassQuality } from '../shared/grass';
import { OffThreadLand, withTownGrass } from '../shared/renderData';
import { toLinear } from '../shared/math';
import { computeNormals, sphereData } from '../shared/modelParts';
import { groupInstances, hideLawn, LandscapeLayout, lawnInputs, lawnLayer, lawnMask, sameLawnInputs, smoothstep, SPLIT_TRIANGLES, WIND, WIND_KIND, windSwayAt, type LawnInputs, type LawnLayer, type LawnMask } from '../shared/nature';
import type { OpenWorldQuality } from '../quality';
import type { BenchDiag } from '../../debug/benchFlags';
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

  /** The land in the render data worker while `build` runs: the terrain's shading and the grass field are made there. */
  private offThread: OffThreadLand | null = null;

  /** `grassQuality`: how the grass is drawn (the preset's, `QualitySettings.grass`); `open`: a big world's distances (`QualitySettings.openWorld`). */
  constructor(world: WorldStructure, lawnDensity: number, lawnLod: 0 | 1, woodsCell: number, private readonly grassQuality: GrassQuality, private readonly open: OpenWorldQuality) {
    super(world, (land, w, name) => new CountryRoads(land, w, name), lawnDensity, lawnLod, woodsCell, open.woodsCell);
  }

  async build(app: pc.AppBase, parent: pc.Entity, assets: AssetRegistry, world: WorldStructure, pace?: () => Promise<void>, buildRoads?: RoadBuilder): Promise<NatureResult> {
    this.offThread = OffThreadLand.open(world);
    try {
      return await this.buildAll(app, parent, assets, world, pace, buildRoads);
    } finally {
      this.offThread?.close();
      this.offThread = null;
    }
  }

  private async buildAll(app: pc.AppBase, parent: pc.Entity, assets: AssetRegistry, world: WorldStructure, pace?: () => Promise<void>, buildRoads?: RoadBuilder): Promise<NatureResult> {
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
    // The woods and the lawn's flowers meanwhile: the terrain waits on the worker's shading for much of its time.
    const plants = (async () => {
      await perfTimeAsync('nature: forests', () => this.forests(app, parent, assets, pace));
      await perfTimeAsync('nature: lawns', () => this.plantLawns(app, parent, assets, world, pace));
    })();
    const [grounds] = await both(ground, plants);
    draws.push(...grounds);
    if (this.watered) draws.push(perfTime('nature: water', () => this.water(app, parent)));
    if (buildRoads) draws.push(...perfTime('nature: roads', () => buildRoads(this.roads, parent)));
    for (const e of draws) e.enabled = false;
    this.others = draws;
    this.findSquares();
    const field = await perfTimeAsync('nature: grass field', async () => {
      const land = await grassLand;
      return land ? withTownGrass(land, world) : grassField(world, this, pace, this.ring);
    });
    const grass = (this.grass = new Grass(app, parent, field, this.grassQuality));
    grass.onGround(grounds.map((g) => g.render!.meshInstances[0]));
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
    this.woodsSquares = [];
    this.flowerSquares = [];
    for (const e of this.others) e.destroy();
    if (this.waterMaterial) {
      this.waterMaterial.lib.release(this.waterMaterial.material);
      this.waterMaterial.material.destroy();
      this.waterMaterial = null;
    }
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
  grassFrame(eye: { x: number; y: number; z: number }, camera: pc.CameraComponent): void {
    this.grass?.frame(eye, camera.frustum);
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
    // (A world message that changed nothing the lawn grows round, a focus lot shown: nothing to do.)
    const inputs = lawnInputs(world);
    if (this.lawnInputs && sameLawnInputs(this.lawnInputs, inputs)) return;
    this.lawnInputs = inputs;
    this.lawnMask = lawnMask(world);
    hideLawn(this.lawn, this.lawnMask, uploadLawn);
    this.grass?.refresh(world);
  }

  /**
   * A big world's ground: the landscape's square in `TILE` m tiles, each with three meshes (64, 32
   * and 16 quads across) and a skirt hanging from its edges (no cracks where neighbours differ), one
   * draw each, the mesh it draws chosen by distance from the camera's target (`frameTiles`). Only
   * the finest grid is shaded (heights and colours, a ring past the tile for its normals); the
   * coarser take every second and fourth of its points.
   */
  private async terrainTiles(app: pc.AppBase, parent: pc.Entity, pace?: () => Promise<void>): Promise<pc.Entity[]> {
    const { size } = this.terrainGrid();
    const x0 = this.w / 2 - size / 2;
    const z0 = this.d / 2 - size / 2;
    const count = Math.ceil(size / TILE);
    const material = MaterialLibrary.for(app).surface('material.ground', { vertexColors: true, variant: 'grass' });
    grassGround(app.graphicsDevice, material);
    const out: pc.Entity[] = [];
    this.tiles = [];
    const S = TILE_LODS[0];
    const c = TILE / S;
    const n = S + 3; // (a ring of one past the tile for the normals)
    // Tile k (i across, j down) and its finest level's grid point (col, row), col and row from -1 to
    // S + 1; row 0 at the tile's far (max z) edge.
    const corner = (k: number) => [k % count, Math.floor(k / count), x0 + (k % count) * TILE, z0 + Math.floor(k / count) * TILE];
    const grid = (k: number) => () => {
      const [, , tx, tz] = corner(k);
      const positions = new Float32Array(n * n * 3);
      for (let row = -1, v = 0; row <= S + 1; row++) for (let col = -1; col <= S + 1; col++, v++) [positions[v * 3], positions[v * 3 + 2]] = [tx + col * c, tz + (S - row) * c];
      return positions;
    };
    const at = (col: number, row: number) => (row + 1) * n + col + 1;
    // The worker shades a few tiles ahead while one's meshes are made here (its round trips overlap);
    // on the page, one at a time.
    const ahead = this.offThread ? SHADE_AHEAD : 0;
    const shading: Promise<{ positions: Float32Array; colors: Float32Array }>[] = [];
    for (let k = 0; k < count * count; k++) {
      while (shading.length <= ahead && k + shading.length < count * count) shading.push(this.shaded(grid(k + shading.length), pace));
      const { positions, colors } = await shading.shift()!;
      const [i, j, tx, tz] = corner(k);
      const meshes = TILE_LODS.map((segs) => tileMesh(app, positions, colors, at, segs, S / segs, c, x0, z0, size));
      const mi = new pc.MeshInstance(meshes[1], material);
      const entity = new pc.Entity(`terrain${i},${j}`, app);
      entity.addComponent('render', { meshInstances: [mi], castShadows: false, receiveShadows: true });
      entity.enabled = false;
      parent.addChild(entity);
      out.push(entity);
      this.tiles.push({ mi, meshes, x: tx + TILE / 2, z: tz + TILE / 2, lod: 1 });
    }
    return out;
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
    for (const t of this.tiles) {
      const d = town > 0 ? Math.hypot(t.x - x, t.z - z, lift) : Math.hypot(t.x - x, t.z - z);
      const lod = d < near ? 0 : d < mid ? 1 : 2;
      if (lod === t.lod) continue;
      t.lod = lod;
      t.mi.mesh = t.meshes[lod];
    }
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
    const material = MaterialLibrary.for(app).surface('material.ground', { vertexColors: true, variant: 'grass' });
    grassGround(app.graphicsDevice, material);
    return drawOf(app, parent, 'terrain', mesh, material);
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
    mat.update();
    this.waterMaterial = { material: mat, lib };
    return drawOf(app, parent, 'water', mesh, mat);
  }

  /** The ground tiles' materials (the town view lays its colour map over them: `groundTiles.ts` `TownGroundMap`). */
  groundMaterials(): readonly pc.StandardMaterial[] {
    // (Worked out once: the tiles keep their material.)
    return (this.groundMats ??= [...new Set(this.tiles.map((t) => t.mi.material as pc.StandardMaterial))]);
  }
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
const TILE = 128;
const TILE_LODS = [64, 32, 16];
/**
 * Ground tiles the render data worker is asked to shade ahead of the one whose meshes are being made
 * (a big world has about 144): enough that it goes on shading while the page is busy with the rest
 * of the build.
 */
const SHADE_AHEAD = 16;

/**
 * One level of a ground tile from the finest level's shaded points (`at(col, row)` into `positions`
 * and `colors`, `c` m apart): every `step`th of them, `segs` quads across, normals from the
 * finest heights round each point, and a skirt down from the edges. UVs as the one-mesh ground's
 * (over the landscape's square from `x0`, `z0`, `size` across).
 */
function tileMesh(app: pc.AppBase, positions: Float32Array, colors: Float32Array, at: (col: number, row: number) => number, segs: number, step: number, c: number, x0: number, z0: number, size: number): pc.Mesh {
  const m = segs + 1;
  const pos: number[] = [];
  const nor: number[] = [];
  const uv: number[] = [];
  const col: number[] = [];
  const idx: number[] = [];
  const vertex = (k: number, drop = 0) => {
    const [x, y, z] = [positions[k * 3], positions[k * 3 + 1], positions[k * 3 + 2]];
    pos.push(x, y - drop, z);
    uv.push((x - x0) / size, (z - z0) / size);
    col.push(colors[k * 4], colors[k * 4 + 1], colors[k * 4 + 2], colors[k * 4 + 3]);
    return pos.length / 3 - 1;
  };
  const normal = (cc: number, rr: number) => {
    const h = (a: number, b: number) => positions[at(a, b) * 3 + 1];
    // (Rows run towards -z: the slope along z is the other way round.)
    const dx = (h(cc + 1, rr) - h(cc - 1, rr)) / (2 * c);
    const dz = (h(cc, rr - 1) - h(cc, rr + 1)) / (2 * c);
    const l = Math.hypot(dx, 1, dz);
    nor.push(-dx / l, 1 / l, -dz / l);
  };
  for (let row = 0; row < m; row++) {
    for (let cc = 0; cc < m; cc++) {
      vertex(at(cc * step, row * step));
      normal(cc * step, row * step);
    }
  }
  for (let row = 0; row < segs; row++) {
    for (let cc = 0; cc < segs; cc++) {
      idx.push(cc + 1 + (row + 1) * m, cc + 1 + row * m, cc + row * m, cc + (row + 1) * m, cc + 1 + (row + 1) * m, cc + row * m);
    }
  }
  // The skirt: each edge's points again, a cell and a metre lower, joined to the edge both ways round.
  const drop = c * step + 1;
  const edges = [Array.from({ length: m }, (_, k) => k), Array.from({ length: m }, (_, k) => segs * m + k), Array.from({ length: m }, (_, k) => k * m), Array.from({ length: m }, (_, k) => k * m + segs)];
  for (const edge of edges) {
    const low = edge.map((v) => {
      const k = vertex(0);
      // (A copy of the edge point, lowered: overwrite what `vertex(0)` wrote.)
      pos[k * 3] = pos[v * 3];
      pos[k * 3 + 1] = pos[v * 3 + 1] - drop;
      pos[k * 3 + 2] = pos[v * 3 + 2];
      uv[k * 2] = uv[v * 2];
      uv[k * 2 + 1] = uv[v * 2 + 1];
      for (let q = 0; q < 4; q++) col[k * 4 + q] = col[v * 4 + q];
      nor.push(nor[v * 3], nor[v * 3 + 1], nor[v * 3 + 2]);
      return k;
    });
    for (let k = 0; k + 1 < edge.length; k++) {
      const [a, b, la, lb] = [edge[k], edge[k + 1], low[k], low[k + 1]];
      idx.push(a, b, lb, a, lb, la, a, lb, b, a, la, lb);
    }
  }
  const mesh = new pc.Mesh(app.graphicsDevice);
  mesh.setPositions(pos);
  mesh.setNormals(nor);
  mesh.setUvs(0, uv);
  mesh.setColors(col);
  mesh.setIndices(pos.length / 3 > 65535 ? new Uint32Array(idx) : new Uint16Array(idx));
  mesh.update(pc.PRIMITIVE_TRIANGLES);
  return gpuOnly(mesh);
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
function release(built: Instanced): void {
  for (const d of [...built.shared, ...built.whole, ...built.cells.flatMap((c) => c.draws)]) d.destroy();
  if (built.template) disposeTemplate(built.template);
}

/** One model's plants, as the landscape keeps them. */
interface Batch {
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
  return { built, casts, part, ...(built.failed ? { again: source } : {}) };
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
): Promise<Instanced> {
  if (matrices.length === 0) return { shared: [], whole: [], cells: [] };
  await pace?.();
  // Without its file, nothing, until the file comes (`Landscape.rebuild`).
  const template = await perfTimeAsync(`nature: model ${key}`, () => buildModel(app, key, assets.get(key, 'model'), [1, 1], { lod }));
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
}

const windStates = new WeakMap<pc.AppBase, WindState>();

/**
 * Hooks the plant finishes as the library makes them (whoever builds the first plant): the wind,
 * and the tint on foliage. Also advances the wind clock with the app's frames and hands it to the
 * shaders. Call once per app, before any model loads.
 */
export function installNature(app: pc.AppBase): void {
  if (windStates.has(app)) return;
  const state: WindState = { time: 0, strength: 1 };
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
  app.on('update', (dt: number) => {
    state.time += Math.min(0.1, dt);
  });
  app.on('prerender', () => {
    value[0] = state.time;
    value[1] = state.strength;
    id.setValue(value);
  });
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
  const g = new Geo();
  const n = 10;
  // Rings up the crown (height, radius), and the dome's middle its normals point out from.
  const rings: [number, number][] = cone ? [[0.22, 0.6], [0.5, 0.42], [0.78, 0.2], [1, 0]] : [[0.3, 0.7], [0.48, 0.97], [0.7, 0.88], [0.88, 0.55], [1, 0]];
  const mid = cone ? 0.35 : 0.5;
  const squash = cone ? 0.5 : 1.4;
  const vertex = (x: number, y: number, z: number, nx: number, ny: number, nz: number) => {
    const l = Math.hypot(nx, ny, nz) || 1;
    g.pos.push(x, y, z);
    g.nor.push(nx / l, ny / l, nz / l);
    g.uv.push(0, 0);
    g.col.push(1, 1, 1, 0.45);
    g.cut.push(15, 0, 15, 0);
    return g.pos.length / 3 - 1;
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
      g.idx.push(a, b, c, a, c, d);
    }
  }
  // Its underside, flat.
  const [yb] = rings[0];
  const centre = vertex(0, yb, 0, 0, -1, 0);
  for (let k = 0; k < n; k++) g.idx.push(centre, ids[0][(k + 1) % n], ids[0][k]);
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
