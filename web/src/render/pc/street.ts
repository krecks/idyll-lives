/**
 * The street on PlayCanvas: the whole town's as `shared/street.ts` lays it out (`TownStreet`: the
 * neighbours' houses, sidewalks, streetlights and their pools of light, kerbside furniture, cars,
 * fences and hedges, gardens, other lots' things; in the game also the road, garden paths and
 * fields), planned per 64 m chunk and drawn per draw group (a 128 m superchunk, `shared/chunks.ts`):
 * one mesh per material for each group, so a group off the screen (or out of the sun's shadow
 * square) costs nothing, and a chunk planned again (the focus lot changed beside it, a thing moved
 * in it) builds only its group's meshes again. The instanced models (`model.street.*`, the
 * foliage, the town's things) are one `Instances` per model over every chunk drawn (`instancing.ts`:
 * one draw per model mesh), their matrices written again only when the chunks drawn change.
 *
 * View-independent: the town is laid out once; the focus lot (the viewed one) is the game's own, so
 * its chunk is planned without its house and garden, and the lots beside it keep their fences clear
 * of it. `setChunks` limits what is drawn to the chunks round the camera (null: all of them). The
 * models are kept (hidden) between builds in the street's own template cache. The neighbours'
 * window glass glows at night only while someone is up in there (`setUp`), redrawn at most once a
 * second, group by group. The focus lot's garage hangs apart (`lotRoot`), hidden with the lot's
 * ground while its basement is in view (`showLot`).
 */

import * as pc from './engine';
import { Instances } from './instancing';
import type { HouseBuilder, HouseMesh } from './house';
import { MaterialLibrary, GAME_FRONT_FACE } from './materials';
import { drawMesh, meshFrom, type DrawOptions } from './meshes';
import { buildModel, withGlows, type ModelMesh, type ModelTemplate } from './models';
import { windStill } from './nature';
import { TemplateCache } from './templateCache';
import type { AssetRegistry } from '../../assets/registry';
import type { ModelEntry } from '../../assets/types';
import type { FenceStyleDef } from '../../content/content';
import type { WorldStructure } from '../../core/protocol';
import type { ViewRect } from '../types';
import type { RGB } from '../shared/environment';
import type { Geo, MeshData } from '../shared/geometry';
import type { HousePart } from '../shared/house';
import type { StandIn } from '../shared/modelEntry';
import { chunkCoords, groupOf, SUPERCHUNK } from '../shared/chunks';
import { castsShadow, FOLIAGE, instanceMatrix, NO_FOCUS, TownStreet, type ChunkPlan, type Inst, type ObjectLook } from '../shared/street';
import { perfEnd, perfStart, perfTimeAsync } from '../../debug/perf';

export interface StreetOptions {
  /** Layer ids of everything this street draws (default: the world layer). */
  layers?: number[];
  /** Neighbours' windows glow only while someone is up in there (`setUp`; the game's street, not the menus' town). */
  lived?: boolean;
  /** Draws the outdoor things of the lots outside the viewed one, public places' and neighbours' gardens' (the game, not the overview, which draws every lot's). */
  look?: ObjectLook;
  /** 1: the street's models' far versions, if they have one (`QualitySettings.streetLod`); 0: full detail. */
  lod?: 0 | 1;
  /** Whether the street casts the sun's shadows (`QualitySettings.streetShadows`). */
  shadows?: boolean;
  /** Also draws the road, the garden paths and the fields off the viewed lot (the game's street; `ChunkPlan.ground`). */
  ground?: boolean;
  /** The content's fence styles: the street draws the fences on other lots (`TownStreet`). */
  fenceStyles?: readonly FenceStyleDef[];
}

/** A model the street places: its template, a draw per mesh, and its glows' (with the street's copy of their material). */
interface Placed {
  template: ModelTemplate;
  draws: Instances[];
  glows: { draw: Instances; material: pc.StandardMaterial }[];
}

/** A placed thing in the game: what the birds find in the street (`PlantPlacement` in the landscape). */
export interface StreetPlant {
  key: string;
  x: number;
  y: number;
  z: number;
  sx: number;
  sy: number;
  sz: number;
  yaw: number;
}

/** How often (ms) at most the neighbours' window glass is redrawn as they go to bed or get up. */
const WINDOWS_EVERY_MS = 1000;
/** The far LOD's houses past this distance (m) from the camera's target aren't drawn. */
const FAR_CULL = 650;
/** Plans kept per chunk (`planned`): the focus lot's chunks with it and without it, and one more. */
const PLANS_KEPT = 3;

/** A chunk as built: its plan, and each of its meshes' vertex arrays (by draw name), made once. */
interface BuiltChunk {
  key: string;
  plan: ChunkPlan;
  data: Map<string, MeshData>;
}

/** A neighbour's house whose glass glows while someone is up: its wall box, its panes, its group. */
interface Home {
  box: [number, number, number, number];
  panes: Geo;
  up: boolean;
  group: number;
}

/** A draw group's meshes (one per material), and its lit and dark window glass. */
interface Group {
  meshes: pc.Entity[];
  windows: HouseMesh[];
  /** Its windows wait to be redrawn (someone went to bed or got up there). */
  windowsDue: boolean;
}

/** What a mesh of the street is drawn as: its material and how. */
interface Draw {
  material: () => pc.Material;
  colors: boolean;
  cut?: boolean;
  cast: boolean;
  receive: boolean;
  alphaIndex?: number;
  /** In the viewed house's lamps' light (the ground layers, as the lot's); the rest is kept out of them. */
  lit?: boolean;
  /** A house part (drawn by the house builder, in its surface). */
  part?: HousePart;
}

export class Street {
  private readonly lib: MaterialLibrary;
  private readonly device: pc.GraphicsDevice;
  /** Everything the street draws hangs here (game space); the viewed lot's own (its garage, `StreetPlan.lot`) under `lotRoot`. */
  readonly root: pc.Entity;
  private readonly lotRoot: pc.Entity;
  /** The town laid out (planned chunk by chunk), and what it was laid out from. */
  private town: TownStreet | null = null;
  private layoutKey = '';
  private townWorld: WorldStructure | null = null;
  private focus = NO_FOCUS;
  /** The chunks to draw (null: every one the town has). */
  private wanted: ReadonlySet<number> | null = null;
  private readonly chunks = new Map<number, BuiltChunk>();
  /** Every chunk planned so far (drawn or not), by id, its last `PLANS_KEPT` plans, newest first: a chunk streamed in again (or planned again for a focus lot just left) with the same key is taken from here. */
  private readonly planned = new Map<number, BuiltChunk[]>();
  /** Draw groups whose chunks changed, waiting for their meshes. */
  private readonly dirtyGroups = new Set<number>();
  /** Whether every chunk of the town has been planned (the first build while streaming does it). */
  private prepared = false;
  private readonly groups = new Map<number, Group>();
  /** The focus lot's own (its garage), as built, and from which chunk key. */
  private lotMeshes: pc.Entity[] = [];
  private lotKey = '';
  /** Instanced models by key (foliage, props, public places' things), kept hidden between builds. */
  private readonly templates: TemplateCache<ModelTemplate>;
  private readonly placed = new Map<string, Placed>();
  /** Keys of the templates the last build draws (see `invalidate`). */
  private used = new Set<string>();
  /** The models' matrices wait to be written again (the chunks drawn changed, or a file came). */
  private instancesDue = true;
  private readonly glowMat: pc.StandardMaterial;
  private readonly poolMat: pc.StandardMaterial;
  private readonly poolTexture: pc.Texture;
  private lamps = -1;
  private readonly glow: RGB = { r: 1, g: 0.8, b: 0.5 };
  /** Neighbours' houses (wall box, grid lines) and their window glass, lit while someone is `up` in there. */
  private homes: Home[] = [];
  private boxes: (readonly [number, number, number, number])[] = [];
  private windowsDrawn = 0;
  private windowsDue = false;
  private readonly matrices = new Float32Array(16);
  private destroyed = false;
  /** Builds are counted: a newer one makes an older one stop where it is. */
  private builds = 0;

  /** See `StreetOptions`. */
  private readonly layers?: number[];
  private readonly lived: boolean;
  private readonly look?: ObjectLook;
  private readonly lod: 0 | 1;
  private readonly shadows: boolean;
  private readonly ground: boolean;
  private readonly fenceStyles: readonly FenceStyleDef[];

  constructor(
    private readonly app: pc.AppBase,
    parent: pc.Entity,
    private readonly assets: AssetRegistry,
    private readonly house: HouseBuilder,
    { layers, lived = false, look, lod = 0, shadows = true, ground = false, fenceStyles = [] }: StreetOptions = {},
  ) {
    this.fenceStyles = fenceStyles;
    this.layers = layers;
    this.lived = lived;
    this.look = look;
    this.lod = lod;
    this.shadows = shadows;
    this.ground = ground;
    this.lib = MaterialLibrary.for(app);
    this.device = app.graphicsDevice;
    this.templates = new TemplateCache(app);
    this.root = new pc.Entity('street');
    parent.addChild(this.root);
    this.lotRoot = new pc.Entity('streetLot');
    this.root.addChild(this.lotRoot);
    // Streetlights' glass: unlit, its colour by the time of day (`setNight`). (Its colours, and the
    // pools' and the lenses' strengths, are sRGB as PlayCanvas takes them, as they were tuned;
    // clamped to 1.)
    const glow = (this.glowMat = new pc.StandardMaterial());
    glow.name = 'streetLampGlass';
    glow.frontFace = GAME_FRONT_FACE;
    glow.useLighting = false;
    glow.useSkybox = false;
    glow.diffuse.set(0, 0, 0);
    glow.emissive.set(0.6, 0.6, 0.56);
    glow.update();
    // Pools of lamplight on the ground: added, unlit, no fog (fog would add up to boxes).
    this.poolTexture = gradientTexture(this.device, 'streetLightPool', [
      [0, 255],
      [0.35, 150],
      [0.7, 40],
      [1, 0],
    ]);
    const pool = (this.poolMat = new pc.StandardMaterial());
    pool.name = 'streetLightPools';
    pool.frontFace = GAME_FRONT_FACE;
    pool.useLighting = false;
    pool.useSkybox = false;
    pool.useFog = false;
    pool.diffuse.set(0, 0, 0);
    // Added: colour x the gradient's falloff (as opacity), so black by day adds nothing.
    pool.emissive.set(0, 0, 0);
    pool.opacityMap = this.poolTexture;
    pool.opacityMapChannel = 'r';
    pool.blendType = pc.BLEND_ADDITIVEALPHA;
    pool.depthWrite = false;
    pool.slopeDepthBias = -4;
    pool.update();
  }

  /** The town as the street has laid it out (null before the first build): which lot is where. */
  get layout(): TownStreet | null {
    return this.town;
  }

  /**
   * Draws only these chunks (`shared/chunks.ts` ids; null: all of the town's) from the next `build`
   * on. True when that changes what is drawn.
   */
  setChunks(ids: ReadonlySet<number> | null): boolean {
    const same = ids === this.wanted || (!!ids && !!this.wanted && ids.size === this.wanted.size && [...ids].every((id) => this.wanted!.has(id)));
    if (same) return false;
    this.wanted = ids;
    return true;
  }

  /**
   * Builds (or keeps) the street for this world with the viewed lot (`view`) as the focus (the menus'
   * town: none, its player's lot `WorldMeta.home`). Only the chunks whose plan changed are built
   * again: one or two when the focus moves to the next lot or a thing outdoors moves.
   */
  async build(world: WorldStructure, view: ViewRect | null, pace?: () => Promise<void>): Promise<void> {
    if (this.destroyed) return;
    const build = ++this.builds;
    const stale = () => this.destroyed || build !== this.builds;
    // (No lot in view: the town as the menus show it, the player's lot dressed as a shell.)
    const mode = view ? 'game' : 'town';
    const layoutKey = TownStreet.layoutKey(world, mode);
    const span = perfStart('street: plan');
    if (!this.town || layoutKey !== this.layoutKey) {
      this.dropChunks();
      const entry = (k: string) => (this.assets.has(k, 'model') ? (this.assets.get(k, 'model') as ModelEntry) : undefined);
      this.town = new TownStreet(this.house.planner, world, { has: (k) => this.assets.has(k, 'model'), entry, lived: this.lived, look: this.look, ground: this.ground, fenceStyles: this.fenceStyles }, mode);
      this.layoutKey = layoutKey;
      this.townWorld = world;
    } else if (world !== this.townWorld) {
      this.town.setWorld(world);
      this.townWorld = world;
    }
    const town = this.town;
    this.focus = view ? (world.plots.find((p) => p.x < view.x + view.w - 0.01 && p.x + p.w > view.x + 0.01 && p.z < view.z + view.d - 0.01 && p.z + p.d > view.z + 0.01)?.id ?? NO_FOCUS) : town.mode === 'town' ? (world.meta?.home ?? NO_FOCUS) : NO_FOCUS;
    const focus = this.focus;
    const all = town.chunkIds();
    const wanted = this.wanted ? all.filter((id) => this.wanted!.has(id)) : all;
    const dirty = this.dirtyGroups;
    const wantedSet = new Set(wanted);
    // Chunks no longer drawn go (their plans are kept: streamed in again, they cost only their group's meshes).
    for (const id of [...this.chunks.keys()]) {
      if (wantedSet.has(id)) continue;
      this.chunks.delete(id);
      dirty.add(groupOf(id));
    }
    let planned = 0;
    const planOf = (id: number, key: string): BuiltChunk => {
      // (The last few plans of each chunk kept: back on a lot the camera just left, its chunks are planned already.)
      const kept = this.planned.get(id) ?? [];
      const at = kept.findIndex((k) => k.key === key);
      const built = at >= 0 ? kept[at] : (() => {
        const plan = town.chunk(id, focus);
        return { key, plan, data: chunkData(plan) };
      })();
      if (at !== 0) this.planned.set(id, [built, ...kept.filter((k) => k !== built)].slice(0, PLANS_KEPT));
      return built;
    };
    for (const id of wanted) {
      const key = town.chunkKey(id, focus);
      if (this.chunks.get(id)?.key !== key) {
        this.chunks.set(id, planOf(id, key));
        dirty.add(groupOf(id));
        planned++;
      }
      await pace?.();
      if (stale()) return;
    }
    // Streaming: every other chunk planned now too (at load, between frames), so the camera
    // roaming later only merges and uploads them.
    if (this.wanted && !this.prepared) {
      for (const id of all) {
        if (wantedSet.has(id)) continue;
        planOf(id, town.chunkKey(id, focus));
        await pace?.();
        if (stale()) return;
      }
      this.prepared = true;
    }
    perfEnd(span);
    if (planned || dirty.size) this.instancesDue = true;

    const meshesSpan = perfStart('street: meshes');
    // (A group left by a build a newer one stopped stays dirty for that one.)
    const rebuilt = dirty.size > 0;
    for (const g of [...dirty]) {
      await this.buildGroup(g, pace, stale);
      if (stale()) return;
      dirty.delete(g);
    }
    // The focus lot's garage (apart: it goes with the lot's ground).
    const focusChunk = town.focusChunk(focus);
    const lotPlan = focusChunk !== null ? this.chunks.get(focusChunk) : undefined;
    const lotKey = lotPlan ? `${focus}|${lotPlan.key}` : '';
    if (lotKey !== this.lotKey) {
      this.lotKey = lotKey;
      for (const e of this.lotMeshes) e.destroy();
      this.lotMeshes = [];
      const lot = lotPlan?.plan.lot;
      if (lot) {
        const parent = this.lotRoot;
        for (const part of lot.silhouettes.parts) {
          const m = this.house.drawData(part, part.geo.meshData({ colors: true, cut: part.cut }), parent);
          if (!m) continue;
          this.dress(m.entity, part.cast);
          this.lotMeshes.push(m.entity);
        }
        const add = (geo: Geo, name: string, material: pc.Material, opts: { cast?: boolean; colors?: boolean } & DrawOptions = {}) => {
          const mesh = meshFrom(this.device, geo.meshData({ colors: opts.colors ?? true }));
          if (!mesh) return;
          const entity = drawMesh(parent, name, mesh, material, { ...opts, castShadows: !!opts.cast && this.shadows, ...(this.layers ? { layers: this.layers } : {}) });
          this.dress(entity, !!opts.cast);
          this.lotMeshes.push(entity);
        };
        add(lot.props, 'streetLotProps', this.lib.plainFinish(), { cast: true });
        add(lot.glow, 'streetLotLamps', this.glowMat, { colors: false });
        add(lot.pools, 'streetLotPools', this.poolMat, { colors: false, receiveShadows: false, alphaIndex: 5 });
      }
    }
    if (rebuilt) this.gatherHomes();
    this.buildFar(town, all, focus);
    perfEnd(meshesSpan);

    if (!this.instancesDue) return;
    this.instancesDue = false;
    const batchesSpan = perfStart('street: batches');
    // Every chunk's instances of a model, one list a model (chunk by chunk, in id order).
    const order = [...this.chunks.keys()].sort((a, b) => a - b);
    const batches = new Map<string, Inst[]>((Object.values(FOLIAGE) as string[]).map((k) => [k, []]));
    for (const id of order) {
      for (const [k, items] of this.chunks.get(id)!.plan.batches) {
        let list = batches.get(k);
        if (!list) batches.set(k, (list = []));
        for (const i of items) list.push(i);
      }
    }
    // Models from the last build that aren't used now are hidden.
    for (const [k, p] of this.placed) if (!batches.get(k)?.length) this.hide(p);
    this.used = new Set([...batches].filter(([, items]) => items.length).map(([k]) => k));
    this.plants = [...batches].flatMap(([k, items]) => items.map(([x, z, s, sy, yaw, y = 0]): StreetPlant => ({ key: k, x, y, z, sx: s, sy, sz: k === FOLIAGE.hedge ? 1 : s, yaw })));
    for (const [k, items] of batches) {
      if (!items.length) continue;
      await pace?.();
      if (stale()) {
        this.instancesDue = true;
        return;
      }
      await perfTimeAsync(`street: batch ${k}`, () => this.instances(k, items, k === FOLIAGE.hedge, castsShadow(k), town.footprints.get(k), town.standIns.get(k), pace));
    }
    perfEnd(batchesSpan);
    // What files that came replace (the boxes) is drawn no more.
    this.disposeStale();
    const lamps = this.lamps;
    this.lamps = -1;
    if (lamps >= 0) this.setNight(lamps, this.glow);
  }

  /**
   * The models the last build drew from instances, as placed (a hedge isn't scaled along its depth):
   * where the birds find the street's bushes, hedges and other lots' trees. A new array only when
   * the street is built again.
   */
  plants: readonly StreetPlant[] = [];

  /**
   * A model file that failed has come: what was built without it (a box, or nothing) is built afresh
   * when next asked for. If the street draws one, the next `build` places it again (true); otherwise
   * the old ones just go. Called between builds.
   */
  invalidate(url: string): boolean {
    const keys = this.templates.invalidate(url);
    // (Those not drawn are hidden already.)
    this.disposeStale(keys.filter((k) => !this.used.has(k)));
    if (!keys.some((k) => this.used.has(k))) return false;
    this.instancesDue = true;
    return true;
  }

  /** Removes what was built (the models are kept, hidden, for the next build). */
  clear(): void {
    this.builds++;
    this.used.clear();
    this.plants = [];
    this.dropChunks();
    this.town = null;
    this.townWorld = null;
    this.layoutKey = '';
    for (const p of this.placed.values()) this.hide(p);
    // Nothing drawn any more: the templates built again for files that came can go.
    this.disposeStale();
  }

  /** Lets go of everything (the street's models too). */
  destroy(): void {
    this.clear();
    this.destroyed = true;
    for (const p of this.placed.values()) this.drop(p);
    this.placed.clear();
    for (const [, t] of this.templates) void t.then((template) => this.disposeTemplate(template)).catch(() => {});
    this.templates.clear();
    this.root.destroy();
    this.glowMat.destroy();
    this.poolMat.destroy();
    this.poolTexture.destroy();
  }

  /** What is drawn now: chunks in detail, draw groups, far groups (for the bench). */
  stats(): { chunks: number; groups: number; far: number; streamed: boolean } {
    return { chunks: this.chunks.size, groups: this.groups.size, far: [...this.farGroups.values()].filter((g) => g.entity?.enabled).length, streamed: this.wanted !== null };
  }

  /** Shows or hides the viewed lot's own (its garage): it stands on the lot's ground, hidden while its basement is in view. */
  showLot(on: boolean): void {
    this.lotRoot.enabled = on;
  }

  /** Neighbours' houses: their wall boxes (x0, z0, x1, z1 on the ground's grid lines). */
  get houses(): readonly (readonly [number, number, number, number])[] {
    return this.boxes;
  }

  /**
   * Which neighbours' houses have someone up in them (as `houses`): only theirs glow at night.
   * Redrawn at most once a second (a street going to bed is many changes), only the groups where
   * someone did; true when it was.
   */
  setUp(up: readonly boolean[]): boolean {
    for (let i = 0; i < this.homes.length; i++) {
      const h = this.homes[i];
      const now = up[i] ?? true;
      if (h.up === now) continue;
      h.up = now;
      this.windowsDue = true;
      const g = this.groups.get(h.group);
      if (g) g.windowsDue = true;
    }
    if (!this.windowsDue || performance.now() - this.windowsDrawn < WINDOWS_EVERY_MS) return false;
    this.windowsDrawn = performance.now();
    this.windowsDue = false;
    for (const [id, g] of this.groups) if (g.windowsDue) this.drawWindows(id, g);
    return true;
  }

  /** Streetlights, porch lamps and window glass follow the lighting key's lamp level (`glow`: the lamps' colour, linear). */
  setNight(lamps: number, glow: RGB): void {
    this.house.setNight(lamps, glow);
    this.glow.r = glow.r;
    this.glow.g = glow.g;
    this.glow.b = glow.b;
    if (Math.abs(lamps - this.lamps) < 1e-3) return;
    this.lamps = lamps;
    const g = Math.min(1, Math.max(0, (lamps - 0.3) / 0.45));
    // Pale glass by day, a warm glow at night.
    this.glowMat.emissive.set(Math.min(1, 0.62 + g * 1.7), Math.min(1, 0.62 + g * 0.95), Math.min(1, 0.58 + g * 0.2));
    this.glowMat.update();
    this.poolMat.emissive.set(0.62 * g, 0.46 * g, 0.26 * g);
    this.poolMat.update();
    // The models' glows (the path lights' lenses) with the streetlights: nothing by day.
    for (const [key, p] of this.placed) {
      for (const { draw, material } of p.glows) {
        material.emissive.set(g, g, g);
        material.update();
        draw.enabled = g > 0 && draw.count > 0 && this.used.has(key);
      }
    }
  }

  /**
   * Each frame (the game's): far groups past `FAR_CULL` m of the camera's target (`x`, `z`) are
   * left out (the fog has them by then).
   */
  frame(x: number, z: number): void {
    for (const g of this.farGroups.values()) {
      if (!g.entity) continue;
      g.entity.enabled = Math.hypot(g.x - x, g.z - z) <= FAR_CULL + SUPERCHUNK * 0.71;
    }
  }

  /**
   * The far LOD: the houses of the chunks not streamed in (`setChunks`), as boxes with their roofs,
   * one mesh a draw group (a 128 m superchunk), built again only when which of its chunks are far
   * changes. None while every chunk is drawn.
   */
  private buildFar(town: TownStreet, all: readonly number[], focus: number): void {
    const parts = new Map<number, { keys: string[]; data: MeshData[] }>();
    if (this.wanted) {
      for (const id of all) {
        if (this.wanted.has(id)) continue;
        const far = town.farChunk(id, focus);
        if (!far) continue;
        const g = groupOf(id);
        let p = parts.get(g);
        if (!p) parts.set(g, (p = { keys: [], data: [] }));
        p.keys.push(far.key);
        p.data.push((this.farData.get(far.key) ?? this.farData.set(far.key, far.geo.meshData({ colors: true })).get(far.key))!);
      }
    }
    for (const [g, old] of this.farGroups) {
      if (parts.has(g)) continue;
      old.entity?.destroy();
      this.farGroups.delete(g);
    }
    for (const [g, p] of parts) {
      const key = p.keys.join('|');
      const old = this.farGroups.get(g);
      if (old?.key === key) continue;
      old?.entity?.destroy();
      const data = concatMeshData(p.data);
      const mesh = data && meshFrom(this.device, data);
      const entity = mesh ? drawMesh(this.root, 'streetFar', mesh, this.lib.plainFinish(), { castShadows: false, ...(this.layers ? { layers: this.layers } : {}) }) : null;
      if (entity) this.dress(entity, false);
      const [gx, gz] = chunkCoords(g);
      this.farGroups.set(g, { key, entity, x: (gx + 0.5) * SUPERCHUNK, z: (gz + 0.5) * SUPERCHUNK });
    }
    // (Vertex arrays of far chunks no group draws any more go.)
    if (this.farData.size > 4 * all.length) this.farData.clear();
  }
  /** Far LOD meshes by draw group, where it is, and what it was made from; far chunks' vertex arrays by key. */
  private readonly farGroups = new Map<number, { key: string; entity: pc.Entity | null; x: number; z: number }>();
  private readonly farData = new Map<string, MeshData>();

  // ---- helpers ----------------------------------------------------------------------------

  /** How each of a chunk's meshes is drawn, by name. */
  private draws(plan: ChunkPlan): Map<string, Draw> {
    const out = new Map<string, Draw>();
    for (const part of plan.silhouettes.parts) out.set(part.name, { material: () => this.lib.plainFinish(), colors: true, cut: part.cut, cast: part.cast, receive: !part.shadowOnly, part });
    out.set('streetSidewalks', { material: () => this.lib.surface('material.sidewalk', { vertexColors: true }), colors: true, cast: false, receive: true });
    out.set('streetProps', { material: () => this.lib.plainFinish(), colors: true, cast: true, receive: true });
    out.set('streetLamps', { material: () => this.glowMat, colors: false, cast: false, receive: true });
    out.set('streetLightPools', { material: () => this.poolMat, colors: false, cast: false, receive: false, alphaIndex: 5 });
    out.set('streetShadows', { material: () => this.lib.contactShadow(false), colors: true, cast: false, receive: true });
    // The ground layers as the lot draws its own: double-sided, in the house's lamps' light.
    for (const g of plan.ground) out.set(g.name, { material: () => this.lib.surface(g.material, { doubleSided: true }), colors: false, cast: false, receive: true, lit: true });
    return out;
  }

  /** A draw group's meshes again, from its chunks' vertex arrays (one mesh a material), and its window glass. */
  private async buildGroup(id: number, pace?: () => Promise<void>, stale?: () => boolean): Promise<void> {
    const members = [...this.chunks.entries()].filter(([c]) => groupOf(c) === id).sort(([a], [b]) => a - b);
    let g = this.groups.get(id);
    if (!members.length) {
      if (g) for (const e of [...g.meshes, ...g.windows.map((w) => w.entity)]) e.destroy();
      this.groups.delete(id);
      return;
    }
    // The new meshes built hidden, a material at a time between frames, then put in place of the old at once.
    const fresh: pc.Entity[] = [];
    const draws = new Map<string, Draw>();
    for (const [, c] of members) for (const [k, d] of this.draws(c.plan)) if (!draws.has(k)) draws.set(k, d);
    for (const [name, d] of draws) {
      const data = concatMeshData(members.map(([, c]) => c.data.get(name)).filter((m) => m !== undefined));
      if (!data) continue;
      let entity: pc.Entity | null = null;
      if (d.part) entity = this.house.drawData(d.part, data, this.root)?.entity ?? null;
      else {
        const mesh = meshFrom(this.device, data);
        if (mesh) entity = drawMesh(this.root, name, mesh, d.material(), { castShadows: d.cast && this.shadows, receiveShadows: d.receive, alphaIndex: d.alphaIndex, ...(this.layers ? { layers: this.layers } : {}) });
      }
      if (!entity) continue;
      if (d.lit) {
        if (this.layers && entity.render) entity.render.layers = this.layers;
      } else this.dress(entity, d.cast);
      entity.enabled = false;
      fresh.push(entity);
      await pace?.();
      if (stale?.()) {
        for (const e of fresh) e.destroy();
        return;
      }
    }
    g = this.groups.get(id);
    if (g) for (const e of g.meshes) e.destroy();
    else this.groups.set(id, (g = { meshes: [], windows: [], windowsDue: false }));
    for (const e of fresh) e.enabled = true;
    g.meshes = fresh;
    g.windowsDue = true;
  }

  /** The neighbours' houses whose glass glows, over every chunk drawn (who was up stays so: the lights don't flick on for a frame). */
  private gatherHomes(): void {
    const wasUp = new Map(this.homes.map((h) => [`${h.box[0]},${h.box[1]}`, h.up]));
    const homes: Home[] = [];
    for (const id of [...this.chunks.keys()].sort((a, b) => a - b)) {
      for (const { spec, panes } of this.chunks.get(id)!.plan.silhouettes.homes) {
        homes.push({ box: [spec.x0, spec.z0, spec.x1, spec.z1], panes, up: wasUp.get(`${spec.x0},${spec.z0}`) ?? true, group: groupOf(id) });
      }
    }
    this.homes = homes;
    this.boxes = homes.map((h) => h.box);
    for (const [gid, g] of this.groups) if (g.windowsDue) this.drawWindows(gid, g);
    this.windowsDrawn = performance.now();
  }

  /** A group's neighbours' glass again: those with someone up in one mesh, glowing; the rest in another, dark. */
  private drawWindows(id: number, g: Group): void {
    for (const m of g.windows) m.entity.destroy();
    g.windows = [];
    g.windowsDue = false;
    const mine = this.homes.filter((h) => h.group === id);
    for (const lit of [true, false]) {
      const panes = mine.filter((h) => h.up === lit).map((h) => h.panes);
      const m = panes.length ? this.house.neighbourWindows(panes, lit, this.root) : null;
      if (!m) continue;
      this.dress(m.entity, false);
      g.windows.push(m);
    }
  }

  /** Forgets every chunk built, its groups' meshes and the focus lot's own. */
  private dropChunks(): void {
    for (const g of this.groups.values()) {
      for (const e of g.meshes) e.destroy();
      for (const w of g.windows) w.entity.destroy();
    }
    this.groups.clear();
    this.chunks.clear();
    this.planned.clear();
    this.prepared = false;
    this.dirtyGroups.clear();
    for (const g of this.farGroups.values()) g.entity?.destroy();
    this.farGroups.clear();
    this.farData.clear();
    for (const e of this.lotMeshes) e.destroy();
    this.lotMeshes = [];
    this.lotKey = '';
    this.homes = [];
    this.boxes = [];
    this.instancesDue = true;
  }

  /** A street mesh's own: kept out of the viewed house's lamps (`excludeFromLights`; its shaders stay simple). */
  private dress(entity: pc.Entity, cast: boolean): void {
    const mis = entity.render?.meshInstances ?? [];
    for (const mi of mis) mi.castShadow = cast && this.shadows;
    this.lib.excludeFromLights([], mis);
    if (this.layers && entity.render) entity.render.layers = this.layers;
  }

  private hide(p: Placed): void {
    for (const d of p.draws) d.enabled = false;
    for (const g of p.glows) g.draw.enabled = false;
  }

  /** Stops drawing a model (its draws go; the template is the cache's). */
  private drop(p: Placed): void {
    for (const d of p.draws) d.destroy();
    for (const g of p.glows) {
      g.draw.destroy();
      g.material.destroy();
    }
  }

  private disposeTemplate(t: ModelTemplate): void {
    for (const m of withGlows(t)) m.mesh.destroy();
  }

  /** Disposes the templates set aside (or those under `keys`), with the street's draws of them. */
  private disposeStale(keys?: readonly string[]): void {
    for (const [k, p] of this.placed) {
      if (keys && !keys.includes(k)) continue;
      // A key whose template was set aside (no longer the cache's): its draws go with it.
      const now = this.templates.built(k);
      if (now === p.template) continue;
      this.drop(p);
      this.placed.delete(k);
    }
    this.templates.disposeStale(keys);
  }

  /**
   * Instances of a model (`flat`: not scaled along z, hedges stretched along their run). Another
   * lot's thing without its file is its box (`standIn`); foliage and kerbside props are left out.
   */
  private async instances(key: string, items: Inst[], flat: boolean, cast: boolean, footprint: [number, number] = [1, 1], standIn?: StandIn, pace?: () => Promise<void>): Promise<void> {
    const template = this.templates.get(key, () => buildModel(this.app, `street:${key}`, this.assets.has(key, 'model') ? (this.assets.get(key, 'model') as ModelEntry) : undefined, footprint, { far: this.layers !== undefined, lod: this.lod, standIn }));
    const t = await perfTimeAsync(`street: model ${key}`, () => template);
    await pace?.();
    if (this.destroyed) return;
    let p = this.placed.get(key);
    if (p && p.template !== t) {
      // Built again (a file came): the old draws go with the old template (`disposeStale`), hidden meanwhile.
      this.hide(p);
      p = undefined;
    }
    if (!p) {
      p = { template: t, draws: [], glows: [] };
      this.placed.set(key, p);
      for (const m of t.meshes) p.draws.push(this.draw(m, m.material, cast));
      for (const { mesh } of t.glows ?? []) {
        // (Every lens glows alike: the street's copy of their material carries the strength.)
        const material = (mesh.material as pc.StandardMaterial).clone();
        material.update();
        p.glows.push({ draw: this.draw(mesh, material, false), material });
      }
    }
    const span = perfStart(`street: matrices ${key}`);
    for (const d of [...p.draws, ...p.glows.map((g) => g.draw)]) {
      d.setCount(items.length);
      for (let i = 0; i < items.length; i++) {
        instanceMatrix(items[i], flat, this.matrices, 0);
        d.setMatrix(i, this.matrices);
      }
      d.update();
    }
    for (const d of p.draws) d.enabled = items.length > 0;
    // Its glow, off until `setNight` lights it.
    for (const g of p.glows) g.draw.enabled = false;
    perfEnd(span);
    // (In place of a box set aside for a file that came, if there was one: never both at once.)
    this.templates.hideStale(key);
  }

  private draw(m: ModelMesh, material: pc.Material, cast: boolean): Instances {
    const d = new Instances(this.device, this.root, m.mesh, material, { name: m.name, castShadows: cast && this.shadows, ...(this.layers ? { layers: this.layers } : {}) });
    // (Foliage sways in the wind, `nature.ts`: a part grown from nothing without fluttering.)
    windStill(d.meshInstance, m);
    this.lib.excludeFromLights([], [d.meshInstance]);
    return d;
  }
}

/** A chunk's meshes as vertex arrays, by draw name (the house parts with their cutaway attribute). */
function chunkData(plan: ChunkPlan): Map<string, MeshData> {
  const out = new Map<string, MeshData>();
  const put = (name: string, geo: Geo, colors: boolean, cut = false) => {
    if (!geo.empty) out.set(name, geo.meshData({ colors, cut }));
  };
  for (const part of plan.silhouettes.parts) put(part.name, part.geo, true, part.cut);
  put('streetSidewalks', plan.walk, true);
  put('streetProps', plan.props, true);
  put('streetLamps', plan.glow, false);
  put('streetLightPools', plan.pools, false);
  put('streetShadows', plan.shadows, true);
  for (const g of plan.ground) put(g.name, g.geo, false);
  return out;
}

/** Vertex arrays one after the other as one mesh's (null: nothing). */
export function concatMeshData(list: readonly MeshData[]): MeshData | null {
  if (!list.length) return null;
  if (list.length === 1) return list[0];
  let verts = 0;
  let tris = 0;
  for (const m of list) {
    verts += m.positions.length / 3;
    tris += m.indices.length;
  }
  const positions = new Float32Array(verts * 3);
  const normals = new Float32Array(verts * 3);
  const uvs = new Float32Array(verts * 2);
  const colors = list[0].colors ? new Float32Array(verts * 4) : undefined;
  const cut = list[0].cut ? new Float32Array(verts * 4) : undefined;
  const indices = verts > 65535 ? new Uint32Array(tris) : new Uint16Array(tris);
  let v = 0;
  let t = 0;
  for (const m of list) {
    positions.set(m.positions, v * 3);
    normals.set(m.normals, v * 3);
    uvs.set(m.uvs, v * 2);
    if (colors && m.colors) colors.set(m.colors, v * 4);
    if (cut && m.cut) cut.set(m.cut, v * 4);
    for (let i = 0; i < m.indices.length; i++) indices[t + i] = m.indices[i] + v;
    v += m.positions.length / 3;
    t += m.indices.length;
  }
  return { positions, normals, uvs, indices, colors, cut, vertexAlpha: list.some((m) => m.vertexAlpha) };
}

/**
 * A radial gradient (white in the middle to black at the rim, `stops`: where and how bright, 0..255),
 * 128 texels square, linear: the pools of lamplight's falloff.
 */
export function gradientTexture(device: pc.GraphicsDevice, name: string, stops: readonly [number, number][]): pc.Texture {
  const size = 128;
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = size;
  const ctx = canvas.getContext('2d')!;
  const g = ctx.createRadialGradient(size / 2, size / 2, 0, size / 2, size / 2, size / 2);
  for (const [at, v] of stops) g.addColorStop(at, `rgb(${v},${v},${v})`);
  ctx.fillStyle = g;
  ctx.fillRect(0, 0, size, size);
  const texture = new pc.Texture(device, { name, width: size, height: size, format: pc.PIXELFORMAT_RGBA8, mipmaps: true, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE });
  texture.setSource(canvas);
  return texture;
}
