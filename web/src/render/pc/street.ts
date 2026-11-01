/**
 * The neighbourhood around the viewed lot on PlayCanvas: the street as `shared/street.ts` lays it
 * out (`planStreet`: the neighbours' houses, sidewalks, streetlights and their pools of light,
 * kerbside furniture, cars, fences and hedges, gardens, other lots' things), drawn as a handful
 * of meshes (one per material) and instanced models (`model.street.*`, the foliage, the town's
 * things: one draw per model mesh, `instancing.ts`).
 *
 * Static: built once per town layout round the viewed lot (`streetKey`) and kept across structure
 * versions; the models are kept (hidden) between builds in the street's own template cache. The
 * neighbours' window glass glows at night only while someone is up in there (`setUp`), redrawn
 * at most once a second. The viewed lot's garage hangs apart (`lotRoot`), hidden with the lot's
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
import type { WorldStructure } from '../../core/protocol';
import type { ViewRect } from '../types';
import type { RGB } from '../shared/environment';
import type { Geo } from '../shared/geometry';
import type { StandIn } from '../shared/modelEntry';
import { FOLIAGE, instanceMatrix, planStreet, streetKey, type Inst, type ObjectLook } from '../shared/street';
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

export class Street {
  private readonly lib: MaterialLibrary;
  private readonly device: pc.GraphicsDevice;
  /** Everything the street draws hangs here (game space); the viewed lot's own (its garage, `StreetPlan.lot`) under `lotRoot`. */
  readonly root: pc.Entity;
  private readonly lotRoot: pc.Entity;
  private key = '';
  private meshes: pc.Entity[] = [];
  /** Instanced models by key (foliage, props, public places' things), kept hidden between builds. */
  private readonly templates: TemplateCache<ModelTemplate>;
  private readonly placed = new Map<string, Placed>();
  /** Footprints of other lots' things by model key, and the boxes they're drawn as without their files (see `look`). */
  private readonly footprints = new Map<string, [number, number]>();
  private readonly standIns = new Map<string, StandIn>();
  /** Keys of the templates the last build draws (see `invalidate`). */
  private used = new Set<string>();
  private readonly glowMat: pc.StandardMaterial;
  private readonly poolMat: pc.StandardMaterial;
  private readonly poolTexture: pc.Texture;
  private lamps = -1;
  private readonly glow: RGB = { r: 1, g: 0.8, b: 0.5 };
  /** Neighbours' houses (wall box, grid lines) and their window glass, lit while someone is `up` in there. */
  private homes: { box: [number, number, number, number]; panes: Geo; up: boolean }[] = [];
  private boxes: (readonly [number, number, number, number])[] = [];
  /** Their glass as drawn: one mesh lit, one dark (see `drawWindows`); when, and whether someone has gone to bed or got up since. */
  private windowMeshes: HouseMesh[] = [];
  private windowsDrawn = 0;
  private windowsDue = false;
  private readonly matrices = new Float32Array(16);
  private destroyed = false;

  /** See `StreetOptions`. */
  private readonly layers?: number[];
  private readonly lived: boolean;
  private readonly look?: ObjectLook;
  private readonly lod: 0 | 1;
  private readonly shadows: boolean;

  constructor(
    private readonly app: pc.AppBase,
    parent: pc.Entity,
    private readonly assets: AssetRegistry,
    private readonly house: HouseBuilder,
    { layers, lived = false, look, lod = 0, shadows = true }: StreetOptions = {},
  ) {
    this.layers = layers;
    this.lived = lived;
    this.look = look;
    this.lod = lod;
    this.shadows = shadows;
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

  /** Builds (or keeps) the street for this world and view. Only a change of the town layout outside the viewed lot rebuilds it. */
  async build(world: WorldStructure, view: ViewRect | null, pace?: () => Promise<void>): Promise<void> {
    const key = streetKey(world, view);
    if (key === this.key || this.destroyed) return;
    for (const e of this.meshes) e.destroy();
    this.meshes = [];
    this.key = key;
    const wasUp = new Map(this.homes.map((h) => [`${h.box[0]},${h.box[1]}`, h.up]));
    this.dropWindows();

    const entry = (k: string) => (this.assets.has(k, 'model') ? (this.assets.get(k, 'model') as ModelEntry) : undefined);
    const plan = await planStreet(this.house.planner, world, view, { has: (k) => this.assets.has(k, 'model'), entry, lived: this.lived, look: this.look }, pace);
    if (this.destroyed || this.key !== key) return;
    for (const [k, f] of plan.footprints) this.footprints.set(k, f);
    for (const [k, s] of plan.standIns) this.standIns.set(k, s);
    const sil = this.house.silhouetteMeshes(plan.silhouettes, this.root);
    const garage = plan.lot ? this.house.silhouetteMeshes(plan.lot.silhouettes, this.lotRoot).meshes : [];
    for (const m of [...sil.meshes, ...garage]) this.dress(m.entity, m.part.cast);
    this.meshes.push(...[...sil.meshes, ...garage].map((m) => m.entity));
    // Who was up stays so through a rebuild (the lights don't flick on for a frame).
    this.homes = sil.homes.map(({ spec, panes }) => ({ box: [spec.x0, spec.z0, spec.x1, spec.z1], panes, up: wasUp.get(`${spec.x0},${spec.z0}`) ?? true }));
    this.boxes = this.homes.map((h) => h.box);
    this.drawWindows();

    const meshesSpan = perfStart('street: meshes');
    const add = (geo: Geo, name: string, material: pc.Material, opts: { cast?: boolean; colors?: boolean; parent?: pc.Entity } & DrawOptions = {}) => {
      const mesh = meshFrom(this.device, geo.meshData({ colors: opts.colors ?? true }));
      if (!mesh) return;
      const entity = drawMesh(opts.parent ?? this.root, name, mesh, material, { ...opts, castShadows: !!opts.cast && this.shadows, ...(this.layers ? { layers: this.layers } : {}) });
      this.dress(entity, !!opts.cast);
      this.meshes.push(entity);
    };
    add(plan.walk, 'streetSidewalks', this.lib.surface('material.sidewalk', { vertexColors: true }));
    add(plan.props, 'streetProps', this.lib.plainFinish(), { cast: true });
    add(plan.glow, 'streetLamps', this.glowMat, { colors: false });
    add(plan.pools, 'streetLightPools', this.poolMat, { colors: false, receiveShadows: false, alphaIndex: 5 });
    add(plan.shadows, 'streetShadows', this.lib.contactShadow(false));
    if (plan.lot) {
      const parent = this.lotRoot;
      add(plan.lot.props, 'streetLotProps', this.lib.plainFinish(), { cast: true, parent });
      add(plan.lot.glow, 'streetLotLamps', this.glowMat, { colors: false, parent });
      add(plan.lot.pools, 'streetLotPools', this.poolMat, { colors: false, receiveShadows: false, alphaIndex: 5, parent });
    }
    perfEnd(meshesSpan);
    await pace?.();
    if (this.destroyed || this.key !== key) return;

    const batchesSpan = perfStart('street: batches');
    const batches = plan.batches;
    // Models from the last build that aren't used now are hidden.
    for (const [k, p] of this.placed) if (!batches.some(([b]) => b === k)) this.hide(p);
    this.used = new Set(batches.filter(([, items]) => items.length).map(([k]) => k));
    this.plants = batches.flatMap(([k, items]) => items.map(([x, z, s, sy, yaw, y = 0]): StreetPlant => ({ key: k, x, y, z, sx: s, sy, sz: k === FOLIAGE.hedge ? 1 : s, yaw })));
    for (const [k, items, cast] of batches) {
      await pace?.();
      if (this.destroyed || this.key !== key) return;
      await perfTimeAsync(`street: batch ${k}`, () => this.instances(k, items, k === FOLIAGE.hedge, cast, this.footprints.get(k), this.standIns.get(k), pace));
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
   * when next asked for. If the street draws one, the next `build` rebuilds it (true); otherwise the
   * old ones just go. Called between builds.
   */
  invalidate(url: string): boolean {
    const keys = this.templates.invalidate(url);
    // (Those not drawn are hidden already.)
    this.disposeStale(keys.filter((k) => !this.used.has(k)));
    if (!keys.some((k) => this.used.has(k))) return false;
    this.key = '';
    return true;
  }

  /** Removes what was built (the models are kept, hidden, for the next build). */
  clear(): void {
    this.used.clear();
    this.plants = [];
    for (const e of this.meshes) e.destroy();
    this.meshes = [];
    this.key = '';
    this.dropWindows();
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
   * Redrawn at most once a second (a street going to bed is many changes); true when it was.
   */
  setUp(up: readonly boolean[]): boolean {
    for (let i = 0; i < this.homes.length; i++) {
      const h = this.homes[i];
      const now = up[i] ?? true;
      if (h.up === now) continue;
      h.up = now;
      this.windowsDue = true;
    }
    if (!this.windowsDue || performance.now() - this.windowsDrawn < WINDOWS_EVERY_MS) return false;
    this.drawWindows();
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

  // ---- helpers ----------------------------------------------------------------------------

  /** A street mesh's own: kept out of the viewed house's lamps (`excludeFromLights`; its shaders stay simple). */
  private dress(entity: pc.Entity, cast: boolean): void {
    const mis = entity.render?.meshInstances ?? [];
    for (const mi of mis) mi.castShadow = cast && this.shadows;
    this.lib.excludeFromLights([], mis);
    if (this.layers && entity.render) entity.render.layers = this.layers;
  }

  /** The neighbours' glass again: those with someone up in one mesh, glowing; the rest in another, dark. */
  private drawWindows(): void {
    for (const m of this.windowMeshes) m.entity.destroy();
    this.windowMeshes = [];
    this.windowsDrawn = performance.now();
    this.windowsDue = false;
    for (const lit of [true, false]) {
      const panes = this.homes.filter((h) => h.up === lit).map((h) => h.panes);
      const m = panes.length ? this.house.neighbourWindows(panes, lit, this.root) : null;
      if (!m) continue;
      this.dress(m.entity, false);
      this.windowMeshes.push(m);
    }
  }

  /** Forgets the neighbours' houses and their glass. */
  private dropWindows(): void {
    for (const m of this.windowMeshes) m.entity.destroy();
    this.windowMeshes = [];
    this.homes = [];
    this.boxes = [];
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
