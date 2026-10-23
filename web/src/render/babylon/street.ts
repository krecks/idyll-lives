/**
 * The neighbourhood around the viewed lot, as a lived-in street: the neighbour houses
 * (`HouseBuilder.silhouettes`, with porches, chimneys and garages), sidewalks with kerbs and
 * a dashed centre line, streetlights that light up at night (with pools of light on the
 * ground), mailboxes, hydrants and bins along the kerb, cars parked on neighbours' driveways,
 * picket fences or hedges around neighbours' back gardens, and flower beds along their house
 * fronts, with pots by the door and little lights along the garden path. The town's public places
 * seen from another lot: shops as silhouettes with an awning, and the park, playground and market
 * things standing in the open. At the ends of the street a bus stop and street signs; benches,
 * planters and bins along the kerb in front of the public places.
 *
 * Rules:
 * - Visual only, and kept off where Sims walk: front lawns (Sims cut across them on their way
 *   out of town), garden paths, driveway mouths, the road and the middle of the sidewalk. Kerb
 *   furniture stands on the road side of the sidewalk, fences only run from the house front line
 *   backwards, cars park beside the houses.
 * - Nothing on the viewed lot: the player builds there (the mailbox stands on the sidewalk). In
 *   the menus' town the player's lot (`WorldMeta.home`) gets its house as a shell dressed as the
 *   game builds it (no porch, chimney, garage, car or garden dressing), and nothing else.
 * - Static: generated once per town layout and kept across structure versions of the viewed
 *   lot (build-mode edits don't rebuild it), merged into a handful of meshes (one per material)
 *   plus thin-instanced foliage and props (manifest models: `model.street.*` and the town's
 *   things, see `assets/packs/street/manifest.json`), a few dozen draw calls for the whole street.
 * - Models built in Blender (`.glb` files, downloaded like the rest), plus a CC0 concrete texture for the sidewalks.
 *
 * The layout and the geometry are engine-neutral (`shared/street.ts` `planStreet`); this draws them on Babylon.
 */

import { Color3, Constants, DynamicTexture, StandardMaterial, type Light, type Material, type Mesh, type Scene } from './core';
import type { AssetRegistry } from '../../assets/registry';
import type { ModelEntry } from '../../assets/types';
import type { WorldStructure } from '../../core/protocol';
import type { ViewRect } from '../types';
import type { Geo } from '../shared/geometry';
import { toMesh } from './meshes';
import type { HouseBuilder } from './house';
import { MaterialLibrary } from './materials';
import { buildModel, withGlows, type ModelTemplate } from './models';
import type { StandIn } from '../shared/modelEntry';
import type { PlantPlacement } from './nature';
import { FOLIAGE, instanceMatrix, planStreet, streetKey, type Inst, type ObjectLook } from '../shared/street';
import { TemplateCache } from './templateCache';
import { perfEnd, perfStart, perfTimeAsync } from '../../debug/perf';

export { fieldLayers, Painter, type ObjectLook } from '../shared/street';


export interface StreetOptions {
  /** Camera layer of everything this street draws (see layers.ts); default: the game world. */
  layerMask?: number;
  /** Neighbours' windows glow only while someone is up in there (`setUp`; the game's street, not the menus' town). */
  lived?: boolean;
  /** Draws the outdoor things of public places outside the viewed lot (the game, not the overview, which draws every lot's). */
  look?: ObjectLook;
  /** 1: the street's models' far versions, if they have one (`QualitySettings.streetLod`); 0: full detail. */
  lod?: 0 | 1;
}

/** How often (ms) at most the neighbours' window glass is redrawn as they go to bed or get up. */
const WINDOWS_EVERY_MS = 1000;

export class Street {
  private readonly lib: MaterialLibrary;
  private key = '';
  private meshes: Mesh[] = [];
  private casters: Mesh[] = [];
  /** Thin-instanced models by key (foliage, props, public places' things), kept hidden between builds. */
  private readonly templates: TemplateCache<ModelTemplate>;
  /** Footprints of public places' things by model key, and the boxes they're drawn as without their files (see `look`). */
  private readonly footprints = new Map<string, [number, number]>();
  private readonly standIns = new Map<string, StandIn>();
  /** Keys of the templates the last build draws (see `invalidate`). */
  private used = new Set<string>();
  private readonly glowMat: StandardMaterial;
  private readonly poolMat: StandardMaterial;
  /** What glows at night on the models drawn (a path light's lens: a template's `glows`), and each instance's strength. */
  private glows: { mesh: Mesh; colors: Float32Array }[] = [];
  private lamps = -1;
  /** Neighbours' houses (wall box, grid lines) and their window glass, lit while someone is `up` in there. */
  private homes: { box: [number, number, number, number]; panes: Geo; up: boolean }[] = [];
  private boxes: (readonly [number, number, number, number])[] = [];
  /** Their glass as drawn: one mesh lit, one dark (see `drawWindows`); when, and whether someone has gone to bed or got up since. */
  private windowMeshes: Mesh[] = [];
  private windowsDrawn = 0;
  private windowsDue = false;

  /** See `StreetOptions`. */
  private readonly layerMask?: number;
  private readonly lived: boolean;
  private readonly look?: ObjectLook;
  private readonly lod: 0 | 1;

  constructor(
    private readonly scene: Scene,
    private readonly assets: AssetRegistry,
    private readonly house: HouseBuilder,
    { layerMask, lived = false, look, lod = 0 }: StreetOptions = {},
  ) {
    this.layerMask = layerMask;
    this.lived = lived;
    this.look = look;
    this.lod = lod;
    this.lib = MaterialLibrary.for(scene);
    this.templates = new TemplateCache(scene);
    const glow = (this.glowMat = new StandardMaterial('streetLampGlass', scene));
    glow.disableLighting = true;
    glow.diffuseColor = Color3.Black();
    glow.specularColor = Color3.Black();
    glow.emissiveColor = new Color3(0.6, 0.6, 0.56);
    // Pools of lamplight on the ground: additive, unlit, no fog (fog would add up to boxes).
    const tex = new DynamicTexture('streetLightPool', 128, scene, true);
    const ctx = tex.getContext();
    const g = ctx.createRadialGradient(64, 64, 0, 64, 64, 64);
    g.addColorStop(0, 'rgb(255,255,255)');
    g.addColorStop(0.35, 'rgb(150,150,150)');
    g.addColorStop(0.7, 'rgb(40,40,40)');
    g.addColorStop(1, 'rgb(0,0,0)');
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, 128, 128);
    tex.update();
    const pool = (this.poolMat = new StandardMaterial('streetLightPools', scene));
    pool.disableLighting = true;
    pool.diffuseColor = Color3.Black();
    pool.specularColor = Color3.Black();
    // Additive: colour x the gradient's falloff (as opacity), so black by day adds nothing.
    tex.getAlphaFromRGB = true;
    pool.opacityTexture = tex;
    pool.emissiveColor = Color3.Black();
    pool.alphaMode = Constants.ALPHA_ADD;
    pool.disableDepthWrite = true;
    pool.fogEnabled = false;
    pool.zOffset = -4;
  }

  /**
   * Builds (or keeps) the street for this world and view; returns the shadow casters. Only a
   * change of the town layout outside the viewed lot rebuilds it.
   */
  async build(world: WorldStructure, view: ViewRect | null, pace?: () => Promise<void>): Promise<Mesh[]> {
    const key = streetKey(world, view);
    if (key === this.key) return this.casters;
    for (const mesh of this.meshes) mesh.dispose(false, false);
    this.meshes = [];
    this.casters = [];
    this.key = key;
    const wasUp = new Map(this.homes.map((h) => [`${h.box[0]},${h.box[1]}`, h.up]));
    this.dropWindows();

    const plan = await planStreet(this.house.planner, world, view, { has: (k) => this.assets.has(k, 'model'), entry: (k) => (this.assets.has(k, 'model') ? (this.assets.get(k, 'model') as ModelEntry) : undefined), lived: this.lived, look: this.look }, pace);
    for (const [k, f] of plan.footprints) this.footprints.set(k, f);
    for (const [k, s] of plan.standIns) this.standIns.set(k, s);
    const sil = this.house.silhouetteMeshes(plan.silhouettes);
    if (this.layerMask !== undefined) for (const mesh of sil.meshes) mesh.layerMask = this.layerMask;
    this.meshes.push(...sil.meshes);
    this.casters.push(...sil.casters);
    // Who was up stays so through a rebuild (the lights don't flick on for a frame).
    this.homes = sil.homes.map(({ spec, panes }) => ({ box: [spec.x0, spec.z0, spec.x1, spec.z1], panes, up: wasUp.get(`${spec.x0},${spec.z0}`) ?? true }));
    this.boxes = this.homes.map((h) => h.box);
    this.drawWindows();
    // The dark glass is first drawn when someone goes to bed: compiled now, not then.
    const lit = this.windowMeshes.find((m) => m.name === 'silhouetteWindowsLit');
    if (lit) void this.house.neighbourGlass(false).forceCompilationAsync(lit).catch(() => {});
    const { walk, pools, shadows } = plan;

    // ---- meshes ---------------------------------------------------------------------------
    const meshesSpan = perfStart('street: meshes');
    const add = (geo: Geo, name: string, material: Material, opts: { cast?: boolean; plain?: boolean; colors?: boolean } = {}) => {
      const mesh = toMesh(geo, name, this.scene, { colors: opts.colors ?? true, cut: false });
      if (!mesh) return;
      if (opts.plain) mesh.hasVertexAlpha = false;
      mesh.material = material;
      mesh.receiveShadows = true;
      mesh.isPickable = false;
      if (this.layerMask !== undefined) mesh.layerMask = this.layerMask;
      this.meshes.push(mesh);
      if (opts.cast) this.casters.push(mesh);
    };
    add(walk, 'streetSidewalks', this.lib.surface('material.sidewalk', { vertexColors: true }));
    add(plan.props, 'streetProps', this.lib.plainFinish(), { cast: true, plain: true });
    add(plan.glow, 'streetLamps', this.glowMat, { colors: false });
    const poolMesh = toMesh(pools, 'streetLightPools', this.scene, { colors: false });
    if (poolMesh) {
      poolMesh.material = this.poolMat;
      poolMesh.isPickable = false;
      poolMesh.alphaIndex = 5;
      if (this.layerMask !== undefined) poolMesh.layerMask = this.layerMask;
      this.meshes.push(poolMesh);
    }
    add(shadows, 'streetShadows', this.lib.contactShadow());
    // Interior lamps of the viewed house never reach the street (keeps its shaders simple), left
    // out as soon as the meshes exist.
    const interior = this.interiorLights();
    this.lib.excludeFromLights(interior, new Set(this.meshes));
    perfEnd(meshesSpan);
    await pace?.();
    const batchesSpan = perfStart('street: batches');
    const foliage: Mesh[] = [];
    const batches = plan.batches;
    // Models from the last build that aren't used now are hidden.
    for (const [key, template] of this.templates) if (!batches.some(([k]) => k === key)) void template.then((t) => withGlows(t).forEach((m) => m.setEnabled(false)));
    this.glows = [];
    this.used = new Set(batches.filter(([, items]) => items.length).map(([key]) => key));
    this.plants = batches.flatMap(([key, items]) => items.map(([x, z, s, sy, yaw]): PlantPlacement => ({ key, x, y: 0, z, sx: s, sy, sz: key === FOLIAGE.hedge ? 1 : s, yaw })));
    for (const [key, items, cast] of batches) {
      await pace?.();
      const meshes = await perfTimeAsync(`street: batch ${key}`, () => this.instances(key, items, key === FOLIAGE.hedge, this.footprints.get(key), this.standIns.get(key), pace));
      this.lib.excludeFromLights(interior, new Set(meshes));
      foliage.push(...meshes);
      if (cast) this.casters.push(...meshes);
    }
    perfEnd(batchesSpan);
    // What files that came replace (the boxes) is drawn no more.
    this.templates.disposeStale();
    for (const mesh of this.meshes) mesh.freezeWorldMatrix();
    this.drawn = [...this.meshes, ...this.windowMeshes, ...foliage];
    const lamps = this.lamps;
    this.lamps = -1;
    if (lamps >= 0) this.setNight(lamps, this.glow);
    return this.casters;
  }

  private glow = new Color3(1, 0.8, 0.5);

  /**
   * The models the last build drew from thin instances, as placed (a hedge isn't scaled along its
   * depth): where the birds find the street's bushes, hedges and public places' trees. A new array
   * only when the street is built again.
   */
  plants: readonly PlantPlacement[] = [];

  /** Every mesh currently drawn (including the foliage templates in use). */
  get all(): readonly Mesh[] {
    return this.drawn;
  }
  private drawn: Mesh[] = [];

  /**
   * A model file that failed has come: what was built without it (a box, or nothing) is built afresh
   * when next asked for. If the street draws one, the next `build` rebuilds it (true); otherwise the
   * old ones just go. Called between builds (one running holds the templates it was given).
   */
  invalidate(url: string): boolean {
    const keys = this.templates.invalidate(url);
    // (Those not drawn are hidden already.)
    this.templates.disposeStale(keys.filter((k) => !this.used.has(k)));
    if (!keys.some((k) => this.used.has(k))) return false;
    this.key = '';
    return true;
  }

  /** Removes what was built (the foliage templates are kept, hidden, for the next build). */
  clear(): void {
    this.used.clear();
    this.plants = [];
    for (const mesh of this.meshes) mesh.dispose(false, false);
    this.meshes = [];
    this.casters = [];
    this.drawn = [];
    this.key = '';
    this.glows = [];
    this.dropWindows();
    for (const [, template] of this.templates) {
      void template.then((t) => {
        if (this.key === '') for (const mesh of withGlows(t)) mesh.setEnabled(false);
      });
    }
    // Nothing drawn any more: the templates built again for files that came can go.
    this.templates.disposeStale();
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
    this.homes.forEach((h, i) => {
      const now = up[i] ?? true;
      if (h.up === now) return;
      h.up = now;
      this.windowsDue = true;
    });
    if (!this.windowsDue || performance.now() - this.windowsDrawn < WINDOWS_EVERY_MS) return false;
    this.drawWindows();
    return true;
  }

  /** The neighbours' glass again: those with someone up in one mesh, glowing; the rest in another, dark. */
  private drawWindows(): void {
    for (const mesh of this.windowMeshes) mesh.dispose(false, false);
    this.windowMeshes = [];
    this.windowsDrawn = performance.now();
    this.windowsDue = false;
    for (const lit of [true, false]) {
      const mesh = this.house.neighbourWindows(this.homes.filter((h) => h.up === lit).map((h) => h.panes), lit);
      if (!mesh) continue;
      if (this.layerMask !== undefined) mesh.layerMask = this.layerMask;
      mesh.freezeWorldMatrix();
      this.windowMeshes.push(mesh);
    }
    // Kept out of the viewed house's interior lights, as the rest of the street.
    this.lib.excludeFromLights(this.interiorLights(), new Set(this.windowMeshes));
    if (this.drawn.length) this.drawn = [...this.drawn.filter((m) => !m.isDisposed()), ...this.windowMeshes];
  }

  /** The viewed house's interior lights (the room lights, or the clustered container holding them), which never reach the street. */
  private interiorLights(): Light[] {
    return this.scene.lights.filter((l) => l.name.startsWith('room') || l.name === 'lamps');
  }

  /** Forgets the neighbours' houses and their glass. */
  private dropWindows(): void {
    for (const mesh of this.windowMeshes) mesh.dispose(false, false);
    this.windowMeshes = [];
    this.homes = [];
    this.boxes = [];
  }

  /** Streetlights, porch lamps and window glass follow the lighting key's lamp level. */
  setNight(lamps: number, glow: Color3): void {
    this.house.setNight(lamps, glow);
    this.glow.copyFrom(glow);
    if (Math.abs(lamps - this.lamps) < 1e-3) return;
    this.lamps = lamps;
    const g = Math.min(1, Math.max(0, (lamps - 0.3) / 0.45));
    // Pale glass by day, a warm glow at night.
    this.glowMat.emissiveColor.set(0.62 + g * 1.7, 0.62 + g * 0.95, 0.58 + g * 0.2);
    this.poolMat.emissiveColor.set(0.62 * g, 0.46 * g, 0.26 * g);
    // The models' glows (the path lights' lenses) with the streetlights: nothing by day.
    for (const { mesh, colors } of this.glows) {
      for (let i = 0; i < colors.length; i += 4) colors.fill(g, i, i + 3);
      mesh.thinInstanceBufferUpdated('color');
      mesh.setEnabled(g > 0);
    }
  }

  // ---- helpers ----------------------------------------------------------------------------

  /**
   * Thin instances of a model (`flat`: not scaled along z, hedges stretched along their run). A public
   * place's thing without its file is its box (`standIn`); foliage and kerbside props are left out.
   */
  private async instances(key: string, items: Inst[], flat = false, footprint: [number, number] = [1, 1], standIn?: StandIn, pace?: () => Promise<void>): Promise<Mesh[]> {
    // The overview's street (on a layer of its own) is seen from afar: files' far versions.
    const template = this.templates.get(key, () => buildModel(this.scene, `street:${key}`, this.assets.has(key, 'model') ? (this.assets.get(key, 'model') as ModelEntry) : undefined, footprint, { far: this.layerMask !== undefined, lod: this.lod, standIn }));
    const t = await perfTimeAsync(`street: model ${key}`, () => template);
    await pace?.();
    const buf = new Float32Array(Math.max(1, items.length) * 16);
    const matrices = perfStart(`street: matrices ${key}`);
    items.forEach((item, i) => instanceMatrix(item, flat, buf, i * 16));
    perfEnd(matrices);
    const meshSpan = perfStart(`street: instance meshes ${key}`);
    for (const mesh of t.meshes) {
      mesh.isPickable = false;
      mesh.receiveShadows = true;
      if (this.layerMask !== undefined) mesh.layerMask = this.layerMask;
      mesh.thinInstanceSetBuffer('matrix', buf, 16, true);
      mesh.thinInstanceCount = items.length;
      if (items.length) mesh.thinInstanceRefreshBoundingInfo(false);
      mesh.setEnabled(items.length > 0);
      mesh.freezeWorldMatrix();
    }
    // Its glow (a still model's: one mesh), off until `setNight` lights it.
    for (const { mesh } of t.glows ?? []) {
      if (this.layerMask !== undefined) mesh.layerMask = this.layerMask;
      mesh.thinInstanceSetBuffer('matrix', buf, 16, true);
      mesh.thinInstanceCount = items.length;
      mesh.setEnabled(false);
      if (!items.length) continue;
      const colors = new Float32Array(items.length * 4).fill(1);
      mesh.thinInstanceSetBuffer('color', colors, 4, false);
      mesh.thinInstanceRefreshBoundingInfo(false);
      mesh.freezeWorldMatrix();
      this.glows.push({ mesh, colors });
    }
    // (In place of a box set aside for a file that came, if there was one: never both at once.)
    this.templates.hideStale(key);
    perfEnd(meshSpan);
    await pace?.();
    return items.length ? t.meshes : [];
  }
}

