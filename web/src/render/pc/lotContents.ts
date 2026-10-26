/**
 * What grows and stands on the lot, for the PlayCanvas renderer (`PlayCanvasRenderer.ts` calls it
 * from `setWorld`, `clear`, `update` and the rest): the landscape round the town and the sky
 * (`nature.ts`), the placed objects with their moving parts (`objectMotion.ts`), contact shadows
 * and glows, the chairs pulled up to tables (`tableTuck.ts`), and the food and dishes (`items.ts`).
 * The house, fences, roads, street and garden pools are the renderer's (`house.ts` and the rest).
 *
 * - Landscapes are kept by town (`Landscape.key`) and shared by the game world and the menus'
 *   overview when they show the same town (`useNature`; the overview asks for its layer with
 *   `useNature('town', world)`, drawn on the town layer, `townLayer`). Plants drawn in groups show
 *   only in the game, those drawn whole only in the town; the woods near the town cast the sun's
 *   shadow in the game from their whole draws, on the shadow-only layer (`layers.ts`), on presets
 *   with `landscapeShadows`.
 * - Objects: one instanced draw per mesh of each model's template (a template per model, size and
 *   paint), kept from build to build (`TemplateCache`) and drawn with the copies of the world being
 *   built; a moving part's meshes draw from buffers of their own. Templates the world no longer uses
 *   are switched off, those in a player's colours let go of.
 *
 * Per frame (`update`): the object clips, the screens' glow and the sky following the camera; no
 * allocations. The lamps' light is `lamps.ts`'s, which drives the shades' glow here (`applyGlow`:
 * each lamp's fade, each room's lights; `setLampsOff` records which are off, for the clips).
 * The residents use `tables` and `pullChairs`, picking and build mode `placed`, the mirrors
 * `mirrors`.
 */

import * as pc from './engine';
import { Instances, INSTANCE_COLOR } from './instancing';
import { ItemLayer } from './items';
import { shadowOnlyLayer } from './layers';
import { MaterialLibrary } from './materials';
import { buildModel, disposeTemplate, glowByInstance, type MirrorSurface, type ModelTemplate } from './models';
import type { GrassPusher } from './grass';
import { installNature, Landscape, NATURE_DIAGS, setWind, Sky, windStill, type RoadBuilder } from './nature';
import { ObjectMotion, writeInstance, type MovingObject, type PopTarget } from './objectMotion';
import { ObjectShadows, type ShadowSource } from './objectShadows';
import { toGame } from './space';
import { tableRooms } from './tableTuck';
import { TemplateCache } from './templateCache';
import { benchDiag } from '../../debug/benchFlags';
import { perfTime, perfTimeAsync } from '../../debug/perf';
import { colorsKey, objectCentre, storeyShown, type ItemPlacement, type ObjectPlacement, type PartColors, type WorldStructure } from '../../core/protocol';
import type { FrameState } from '../../core/bridge';
import type { Lighting } from '../shared/environment';
import { LOOK as CUT } from '../shared/geometry';
import { compose, quatYawPitchRoll, type Quat } from '../shared/math';
import { motionsOf, paintColors, painted, placementVariation, standInOf, type Motions } from '../shared/modelEntry';
import { restPoses, slotOf } from '../shared/objectMotion';
import type { TableRoom } from '../shared/tableTuck';
import { WALL_HEIGHT } from '../shared/walls';
import type { RenderQuality } from '../quality';
import { resizeKey, type ObjectBackgroundClips, type ObjectClipInfo, type ObjectResize, type RendererDeps } from '../types';

/** Radius of the sky dome (m): inside the camera's far plane. */
const SKY_RADIUS = 700;

/** The lot as the renderer has it now: the storey in view, where each storey's rows are, and what's in view. */
export interface LotView {
  /** The storey in view (-1: the basement). */
  storey(): number;
  /** The storey a lot row is on. */
  storeyOf(z: number): number;
  /** The first lot row of storey `k`. */
  rowOf(k: number): number;
  /** Whether a ground point (tiles) is in the lot being viewed, `margin` round it. */
  inView(x: number, z: number, margin: number): boolean;
}

/** What the renderer does for the lot's contents. */
export interface LotHooks {
  /** Gives the frames the build runs behind a turn (the renderer's `pace`). */
  pace(): Promise<void>;
  /** The sun's shadows draw again (`urgent`: now, not at the shadow rate). */
  shadowsMoved(urgent: boolean): void;
  /** The landscapes drawn changed (their draws are `LotContents.landscapeDraws`: the lamps leave them out). */
  landscapeChanged?(): void;
}

/** A landscape, shared by the game world and the overview when they are the same town. */
interface NatureSet {
  /** Its town's (`Landscape.key`). */
  key: string;
  /** Kept to refresh the lawn when rooms or objects change (build mode), and to build plants again when a file comes. */
  landscape: Landscape;
  draws: pc.Entity[];
  casters: pc.Entity[];
  only: Map<pc.Entity, 'game' | 'town'>;
  ready: Promise<void>;
  users: Set<'world' | 'town'>;
  pending: number;
  /** Its plants being built again with the files that came (`refreshNature`), one file after another. */
  refreshing: Promise<void>;
}

/** A placed object as drawn (picking, build mode's hand, the chairs pulled in). */
export interface PlacedObject {
  id: number;
  /** Where it stands, on the storey it's on (`y0`: that storey's floor). */
  y0: number;
  minX: number;
  minZ: number;
  maxX: number;
  maxZ: number;
  height: number;
  /** Bottom of the model above `y0` (things on a wall or the ceiling hang higher). */
  bottom: number;
  /** Things on a wall: the cutaway look bits that drop their wall (`wallCutBits`), to hide them with it. */
  wallCut?: number;
  /** Its instance in its template's draws. */
  pop: PopTarget;
  /** Its contact shadow's instance. */
  shadow?: PopTarget;
  /** Things on a top: the object whose top it is (they're in hand with it). */
  host?: number;
  /** What it was built from: to place it again (its picking box) without a build (`patch`). */
  built?: { template: ModelTemplate; motions?: Motions };
}

/** A mirror's glass on an instance (`mirrors.ts` draws the reflections): its faces, and the buffer and index of the matrix it moves with. */
export interface MirrorInstance {
  surfaces: MirrorSurface[];
  matrices: Float32Array;
  index: number;
}

/** What glows of one model (its shades, a screen): the draw, each instance's room, lamp and object, and whether it's a screen. */
interface Glow {
  draw: Instances;
  colors: Float32Array;
  rooms: number[];
  lamps: number[];
  ids: number[];
  screen: boolean;
}

/** One model's draws: a draw per template mesh (and glow), and the buffers they draw from. */
interface ModelDraws {
  template: ModelTemplate;
  /** Per template mesh, then per glow. */
  draws: Instances[];
  glows: Instances[];
}

/** How lamps and rooms are lit now (`lamps.ts` drives it): each lamp's level, each room's windows' glow and whether it's kept dark. */
export interface GlowLight {
  /** How far the shades glow (0 by day .. 1 at night). */
  level: number;
  /** A lamp's level as it fades on or off (1: on). */
  lampOn(id: number): number;
  /** A room's lights as they go out (1: on). */
  roomOn(room: number): number;
  /** A room without windows (its shades glow by day too). */
  dimmed(room: number): boolean;
}

export class LotContents {
  readonly sky: Sky;
  readonly items: ItemLayer;
  private readonly lib: MaterialLibrary;
  private readonly motion = new ObjectMotion();
  private readonly diag = benchDiag();
  private readonly shadowLayer: pc.Layer;

  /** Landscapes by town (seed and size), and which one each layer shows. */
  private readonly natures = new Map<string, NatureSet>();
  private readonly natureIn: Record<'world' | 'town', NatureSet | null> = { world: null, town: null };
  private readonly natureTicket = { world: 0, town: 0 };
  /** The overview's layer for the town's draws (`overview.ts` `townLayer`; without it the town's are drawn on the world layer). */
  townLayer: number | null = null;

  /** Object templates by key, kept from build to build, and the ones the world being built uses. */
  private readonly templates: TemplateCache<ModelTemplate>;
  private readonly templatesInUse = new Set<string>();
  /** Templates in a player's colours: let go of once no world draws them. */
  private readonly paintedTemplates = new Set<string>();
  private readonly modelDraws = new Map<string, ModelDraws>();
  /** The objects as drawn by the last build. */
  placed: PlacedObject[] = [];
  /** Mirror glass on the objects drawn (`mirrors.ts` reflects in it). */
  mirrors: MirrorInstance[] = [];
  /** The tables the seats face, by seat id, for the residents pulling their chairs up. */
  tables = new Map<number, TableRoom>();
  private seatTurns = new Map<number, number>();
  private pulledIn = new Map<number, { by: number; saved: { target: PopTarget; matrix: Float32Array }[] }>();
  private readonly wallItemsHidden = new Map<number, Float32Array>();
  /** Whether a moving part casts the sun's shadow (then its moves draw the shadows again). */
  private movingPartsCast = false;
  private glows: Glow[] = [];
  private glowLight: GlowLight = { level: 0, lampOn: (id) => (this.lampsOff.has(id) ? 0 : 1), roomOn: () => 1, dimmed: () => false };
  /** The objects residents use this frame (screens are lit by it) and when that was. */
  private readonly screenOn = new Set<number>();
  private screenTime = 0;
  /** Contact shadows under the objects: the blob and this world's draw of it. */
  private readonly blob: { mesh: pc.Mesh; material: pc.StandardMaterial };
  private blobs: Instances | null = null;
  /** Low's one sun-shadow caster for the objects' bodies (`QualitySettings.mergedObjectShadows`, `objectShadows.ts`); null on the others. */
  private readonly objectShadows: ObjectShadows | null;
  /** Lamps (and fires) switched off, by object id (the sim's `lampsOff`). */
  private lampsOff: ReadonlySet<number> = new Set();
  private reducedMotion = false;
  private readonly vTmp = new Float32Array(16);
  private readonly qTmp: Quat = [0, 0, 0, 1];
  private readonly eye = { x: 0, y: 0, z: 0 };

  constructor(
    private readonly app: pc.AppBase,
    private readonly root: pc.Entity,
    sun: pc.Entity,
    /** The camera (its eye: the sky's middle, and how near the ambient clips are). */
    private readonly camera: pc.Entity,
    private readonly deps: RendererDeps,
    private readonly quality: RenderQuality,
    private readonly view: LotView,
    private readonly hooks: LotHooks,
  ) {
    this.lib = MaterialLibrary.for(app);
    // Wind sway and tint variation for the plant finishes (hooks them as the models load).
    installNature(app);
    this.shadowLayer = shadowOnlyLayer(app, sun);
    this.objectShadows = quality.mergedObjectShadows ? new ObjectShadows(app.graphicsDevice, root, this.shadowLayer.id) : null;
    const clouds = deps.assets.has('image.clouds', 'image') ? deps.assets.get('image.clouds', 'image')!.url : undefined;
    this.sky = new Sky(app, root, SKY_RADIUS, clouds);
    this.items = new ItemLayer(app, root, deps.assets, deps.content);
    this.templates = new TemplateCache(app);
    this.motion.setAmbientRange(quality.ambientRange);
    this.blob = contactBlob(app);
  }

  // --- the landscape -------------------------------------------------------------------

  /** The game world's landscape (built with `buildNature`), for the overview's camera, the birds and the haze. */
  get landscape(): Landscape | null {
    return this.natureIn.world?.landscape ?? null;
  }

  /** The landscape `layer` shows. */
  landscapeOf(layer: 'world' | 'town'): Landscape | null {
    return this.natureIn[layer]?.landscape ?? null;
  }

  /**
   * The game world's landscape for `world` (shared with the overview's when it's the same town),
   * and its lawn hidden where rooms and furniture are now (rooms built, furniture bought).
   */
  async buildNature(world: WorldStructure, buildRoads?: RoadBuilder): Promise<void> {
    await perfTimeAsync('render: nature', async () => {
      const n = await this.useNature('world', world, buildRoads);
      perfTime('nature: refreshLawn', () => n.landscape.refreshLawn(world));
    });
  }

  /**
   * The landscape of `world` for a layer: shared with the other layer when it is the same town
   * (seed and size), otherwise built; one no layer shows any more is destroyed.
   */
  async useNature(layer: 'world' | 'town', world: WorldStructure, buildRoads?: RoadBuilder): Promise<NatureSet> {
    const key = Landscape.key(world);
    const ticket = ++this.natureTicket[layer];
    let set = this.natures.get(key);
    if (!set) {
      const landscape = new Landscape(world, this.quality.lawnDensity, this.quality.streetLod, this.quality.woodsCell, this.quality.grass);
      const created: NatureSet = { key, landscape, draws: [], casters: [], only: new Map(), users: new Set(), pending: 0, ready: Promise.resolve(), refreshing: Promise.resolve() };
      created.ready = landscape.build(this.app, this.root, this.deps.assets, world, () => this.hooks.pace(), buildRoads).then((nature) => {
        perfTime('nature: meshes', () => {
          created.draws = nature.draws;
          created.casters = nature.casters;
          created.only = nature.only;
          for (const e of nature.draws) this.prepareNature(e);
        });
      });
      this.natures.set(key, (set = created));
    }
    set.pending++;
    try {
      await set.ready;
    } finally {
      set.pending--;
    }
    if (ticket === this.natureTicket[layer]) {
      const old = this.natureIn[layer];
      if (old !== set) old?.users.delete(layer);
      set.users.add(layer);
      this.natureIn[layer] = set;
    }
    // Drop landscapes nobody shows; draw each on the layers that use it.
    for (const [k, n] of this.natures) {
      if (!n.users.size && !n.pending) {
        n.landscape.dispose();
        this.natures.delete(k);
        continue;
      }
      this.showNature(n);
    }
    this.hooks.landscapeChanged?.();
    this.hooks.shadowsMoved(true);
    return set;
  }

  /** The landscape's draws (the lamps leave them out: `MaterialLibrary.excludeFromLights`). */
  landscapeDraws(): pc.MeshInstance[] {
    const out: pc.MeshInstance[] = [];
    for (const n of this.natures.values()) for (const e of n.draws) for (const r of e.findComponents('render') as pc.RenderComponent[]) out.push(...r.meshInstances);
    for (const r of this.sky.dome.findComponents('render') as pc.RenderComponent[]) out.push(...r.meshInstances);
    return out;
  }

  /** A landscape draw just built: switched off until shown (`showNature`), dug away over a basement in view. */
  private prepareNature(entity: pc.Entity): void {
    entity.enabled = false;
    for (const r of entity.findComponents('render') as pc.RenderComponent[]) for (const mi of r.meshInstances) this.lib.digInto(mi);
  }

  /**
   * Draws a landscape on the layers that use it (see the file's header): what both draw on the world
   * layer and the town's (each camera draws one of them, and the sun lights the one on screen), the
   * whole plants on the town layer, and on the shadow-only layer too while the game casts from them.
   */
  private showNature(n: NatureSet): void {
    const world = n.users.has('world');
    const town = n.users.has('town');
    const townLayer = this.townLayer ?? pc.LAYERID_WORLD;
    const casters = new Set(this.quality.landscapeShadows ? n.casters : []);
    // `?diag=nonature`: without the woods, rocks and lawn (the terrain, water and roads stay); `nowoods`,
    // `nofarwoods`, `noflowers` and `nograss` leave out one part of them (`NaturePart`).
    const plantsOff = this.diag.has('nonature');
    const partsOff = new Set(NATURE_DIAGS.filter(([diag]) => this.diag.has(diag)).map(([, part]) => part));
    const parts = partsOff.size ? n.landscape.parts() : null;
    for (const e of n.draws) {
      const render = e.render;
      const plant = !!render?.meshInstances[0]?.instancingData;
      const only = n.only.get(e);
      let first: number = pc.LAYERID_WORLD;
      let second = -1;
      let on: boolean;
      let casts = casters.has(e);
      if (!only) {
        on = world || town;
        if (town) [first, second] = world && townLayer !== pc.LAYERID_WORLD ? [pc.LAYERID_WORLD, townLayer] : [townLayer, -1];
      } else if (only === 'game') {
        on = world;
        casts = false;
      } else if (town) {
        on = true;
        first = townLayer;
        // (The game casts its shadows from them too.)
        if (world && casts) second = this.shadowLayer.id;
      } else {
        // The game draws these in groups: whole, they only cast its shadow.
        on = world && casts;
        first = this.shadowLayer.id;
      }
      const part = parts?.get(e);
      e.enabled = on && !(plant && plantsOff) && !(part && partsOff.has(part));
      if (render) {
        const layers = render.layers;
        if (layers.length !== (second < 0 ? 1 : 2) || layers[0] !== first || (second >= 0 && layers[1] !== second)) render.layers = second < 0 ? [first] : [first, second];
        if (render.castShadows !== casts) render.castShadows = casts;
      }
    }
  }

  /** A model file that failed has come: each landscape's plants built without it are built again (`rebuildNature`). */
  refreshNature(url: string, gone: () => boolean): void {
    for (const set of this.natures.values()) {
      set.refreshing = set.refreshing.then(() => this.rebuildNature(set, url, gone)).catch((err) => console.error(`[render] building the landscape again with ${url} failed`, err));
    }
  }

  /**
   * Builds again the plants of `set` built without the file at `url` (nothing, where it failed;
   * the full model, where a far version did), in slices between frames and unseen, and swaps them
   * in: the rest of the landscape stays as it is, and what they replace is drawn until then and
   * destroyed after. No world is built again for it. (PlayCanvas compiles a shader as it's first
   * drawn: nothing waits for the new draws' shaders before they are swapped in.)
   */
  private async rebuildNature(set: NatureSet, url: string, gone: () => boolean): Promise<void> {
    try {
      await set.ready;
    } catch {
      // (Its first build failed: there's nothing to build again into.)
      return;
    }
    const lost = () => gone() || this.natures.get(set.key) !== set;
    if (lost()) return;
    const built = await set.landscape.rebuild(this.app, this.root, this.deps.assets, url, () => this.hooks.pace());
    if (!built) return;
    if (lost()) {
      built.discard();
      return;
    }
    for (const e of built.draws) this.prepareNature(e);
    built.commit();
    const replaced = new Set(built.replaced);
    set.draws = [...set.draws.filter((e) => !replaced.has(e)), ...built.draws];
    set.casters = [...set.casters.filter((e) => !replaced.has(e)), ...built.casters];
    for (const e of replaced) set.only.delete(e);
    for (const [e, only] of built.only) set.only.set(e, only);
    this.showNature(set);
    this.hooks.landscapeChanged?.();
    this.hooks.shadowsMoved(true);
  }

  /** Lets go of a layer's landscape (the overview left; the game's stays with its world). */
  dropNature(layer: 'world' | 'town'): void {
    const set = this.natureIn[layer];
    if (!set) return;
    set.users.delete(layer);
    this.natureIn[layer] = null;
    if (!set.users.size && !set.pending) {
      set.landscape.dispose();
      this.natures.delete(set.key);
    } else this.showNature(set);
    this.hooks.landscapeChanged?.();
  }

  // --- the sky -------------------------------------------------------------------------

  /**
   * The time of day (each lighting update): the sky, the night clips (curtains drawn), and the
   * shades' glow at the lamps' level (from dusk; the rooms' own state is `lamps.ts`'s, `applyGlow`).
   */
  applyLighting(l: Lighting): void {
    this.sky.update(l);
    this.motion.setLamps(l.lamps);
    const level = Math.min(1, Math.max(0, (l.lamps - 0.35) / 0.5));
    if (level !== this.glowLight.level) this.applyGlow({ ...this.glowLight, level });
  }

  /** How strongly the wind moves the plants (reduce motion: still), and whether the clouds drift. */
  setReducedMotion(on: boolean): void {
    this.reducedMotion = on;
    setWind(this.app, on ? 0 : 1);
  }

  /** Each frame, game or overview, once the camera is placed: the sky round it, the clouds' drift (`dt` ms), the grass in view. */
  frame(dt: number): void {
    const eye = toGame(this.camera.getPosition(), this.eye);
    this.sky.follow(eye.x, eye.y, eye.z);
    if (!this.reducedMotion) this.sky.drift(dt);
    const camera = this.camera.camera;
    if (!camera) return;
    const { world, town } = this.natureIn;
    world?.landscape.grassFrame(eye, camera);
    if (town && town !== world) town.landscape.grassFrame(eye, camera);
  }

  /** The residents pushing the game world's grass aside (the nearest the eye first). */
  pushGrass(pushers: readonly GrassPusher[]): void {
    this.natureIn.world?.landscape.pushGrass(pushers);
  }

  // --- objects -------------------------------------------------------------------------

  /** Model key for an object. */
  private objectModel(o: ObjectPlacement): string {
    return this.deps.content.object(o.def)?.model ?? `model.${o.def}`;
  }

  /** How the player resized an object (undefined: as its kind comes). */
  private objectResize(o: ObjectPlacement): ObjectResize | undefined {
    const def = this.deps.content.object(o.def);
    const from = def?.footprint ?? [1, 1];
    const tiles: [number, number] = o.rot % 2 === 0 ? [o.w, o.d] : [o.d, o.w];
    const size: [number, number] = o.size ? [o.size[0] / 100, o.size[1] / 100] : tiles;
    const resize = { from: [from[0], from[1]] as [number, number], size, tall: o.tall ?? 0, ceiling: def?.layer === 'ceiling' };
    return resizeKey(resize) ? resize : undefined;
  }

  /** The colours an object's parts are drawn in: the player's, on the parts its model paints. */
  private objectColors(o: ObjectPlacement): PartColors | undefined {
    return paintColors(this.deps.assets.get(this.objectModel(o), 'model'), o.colors);
  }

  /** An object's template key, and how to build it. */
  objectTemplate(o: ObjectPlacement): [string, () => Promise<ModelTemplate>] {
    const key = this.objectModel(o);
    const def = this.deps.content.object(o.def);
    const resize = this.objectResize(o);
    const colors = this.objectColors(o);
    const id = `object:${key}${resizeKey(resize)}${colorsKey(colors)}`;
    if (colors && colorsKey(colors)) this.paintedTemplates.add(id);
    // Without its file, a box in its place, so it can still be seen and clicked, until the file comes
    // (`refreshModels`). Shaped by the first kind built with this model; no two kinds share one today.
    return [id, () => buildModel(this.app, key, painted(this.deps.assets.get(key, 'model'), colors), def?.footprint ?? [1, 1], { resize, standIn: standInOf(this.deps.content, def) })];
  }

  /** Cached template by key; marks it as used by the world being built. */
  template(key: string, build: () => Promise<ModelTemplate>): Promise<ModelTemplate> {
    this.templatesInUse.add(key);
    return this.templates.get(key, build);
  }

  /**
   * Where a click on `o` lands, in its storey's row space: its own tiles, or what's drawn of it (see
   * `buildObjects`). Something held open (a wall bed folded down) by its whole tiles.
   * Returns the near corner and its size `[x, z, w, d]`, not the far corner.
   */
  private pickBox(o: ObjectPlacement, template: ModelTemplate, motions: Motions | undefined): [number, number, number, number] {
    const def = this.deps.content.object(o.def);
    const layer = def?.layer;
    const centre = objectCentre(o);
    const D = this.view.rowOf(this.view.storeyOf(o.z));
    const z = o.z - D;
    const [ox, oz] = [centre.x - o.x - o.w / 2, centre.z - o.z - o.d / 2];
    const { turn, size } = placementVariation(this.deps.assets.get(this.objectModel(o), 'model')?.vary ?? 0, o.x, o.z);
    // Something small on a top is picked by its own disc, not the whole tile (the table around it stays
    // clickable). Anything else by what's drawn of it (a pot on a rug leaves the rug around it clickable),
    // never past its tiles; something held open (a wall bed folded down) by its whole tiles.
    const radius = this.deps.content.radius(def);
    let [px, pz] = layer === 'surface' ? [centre.x - radius * 1.2, centre.z - D - radius * 1.2] : [o.x + ox, z + oz];
    let [pw, pd] = layer === 'surface' ? [radius * 2.4, radius * 2.4] : [o.w, o.d];
    const shape = o.open && motions?.open ? undefined : template.outline;
    if (layer !== 'surface' && shape) {
      const yaw = (o.rot * Math.PI) / 2 + ((o.turn ?? 0) * Math.PI) / 180 + turn;
      const [c, s] = [Math.cos(yaw), Math.sin(yaw)];
      // Model x and z turned by the yaw (as `RotationYawPitchRoll`: x -> (c, -s), z -> (s, c)).
      const xs: number[] = [];
      const zs: number[] = [];
      for (const mx of [shape.minX, shape.maxX]) {
        for (const mz of [shape.minZ, shape.maxZ]) {
          xs.push(centre.x + (mx * c + mz * s) * size);
          zs.push(centre.z - D + (-mx * s + mz * c) * size);
        }
      }
      const [x0, x1] = [Math.max(o.x + ox, Math.min(...xs) - 0.03), Math.min(o.x + ox + o.w, Math.max(...xs) + 0.03)];
      const [z0, z1] = [Math.max(z + oz, Math.min(...zs) - 0.03), Math.min(z + oz + o.d, Math.max(...zs) + 0.03)];
      if (x1 > x0 && z1 > z0) [px, pz, pw, pd] = [x0, z0, x1 - x0, z1 - z0];
    }
    return [px, pz, pw, pd];
  }

  /**
   * The cutaway look bits (geometry.ts `LOOK`) that drop the wall a thing on a wall hangs on, as
   * house.ts `edgeMask` gives them: looking towards a room on either side of it (along any of the
   * tiles it hangs across), and the way it faces (its wall then stands between the camera and it,
   * even with no room behind).
   */
  private wallCutBits(world: WorldStructure, o: ObjectPlacement): number {
    const W = world.width;
    const k = this.view.storeyOf(o.z);
    // A room on the thing's own storey (lot rows run on into the next storey's).
    const room = (x: number, z: number) => x >= 0 && z >= 0 && x < W && z < world.depth && this.view.storeyOf(z) === k && world.rooms[z * W + x] > 0;
    const r = o.rot % 4;
    let bits: number = [CUT.PosZ, CUT.PosX, CUT.NegZ, CUT.NegX][r];
    if (r % 2 === 0) {
      // Its back on the wall along grid line `z`, between rows z - 1 and z.
      const z = r === 0 ? o.z : o.z + o.d;
      for (let x = o.x; x < o.x + o.w; x++) bits |= (room(x, z) ? CUT.PosZ : 0) | (room(x, z - 1) ? CUT.NegZ : 0);
    } else {
      const x = r === 1 ? o.x : o.x + o.w;
      for (let z = o.z; z < o.z + o.d; z++) bits |= (room(x, z) ? CUT.PosX : 0) | (room(x - 1, z) ? CUT.NegX : 0);
    }
    return bits;
  }

  /** The draws of a model's template (made the first time it's drawn, and again for a template built anew). */
  private drawsOf(key: string, template: ModelTemplate): ModelDraws {
    let d = this.modelDraws.get(key);
    if (d?.template === template) return d;
    if (d) for (const draw of [...d.draws, ...d.glows]) draw.destroy();
    const device = this.app.graphicsDevice;
    d = {
      template,
      draws: template.meshes.map((m) => {
        const draw = new Instances(device, this.root, m.mesh, m.material, { name: m.name, castShadows: true, receiveShadows: true });
        windStill(draw.meshInstance, m);
        return draw;
      }),
      // (Glows: unlit, added on top, their strength per instance in its colour: `applyGlow`.)
      glows: (template.glows ?? []).map((g) => new Instances(device, this.root, g.mesh.mesh, glowByInstance(this.app, g.mesh.material), { name: g.mesh.name, streams: [INSTANCE_COLOR], castShadows: false, receiveShadows: false })),
    };
    this.modelDraws.set(key, d);
    return d;
  }

  /**
   * Builds the objects of `world` on the storeys up to the one in view (each lifted to its storey,
   * from its lot rows; from the ground up, stairs down to the basement show in their stairwell too),
   * their contact shadows, the moving parts' clips, the tables the seats face; then switches off
   * the templates no longer used. After the lot (the house) and the street.
   */
  async buildObjects(world: WorldStructure): Promise<void> {
    const v = this.view;
    const storey = v.storey();
    this.templatesInUse.clear();
    this.placed = [];
    this.wallItemsHidden.clear();
    const down = (o: ObjectPlacement) => storey >= 0 && v.storeyOf(o.z) === -1 && !!this.deps.content.object(o.def)?.stairs;
    const shown = world.objects.filter((o) => (storeyShown(v.storeyOf(o.z), storey) || down(o)) && v.inView(o.x + o.w / 2, o.z - v.rowOf(v.storeyOf(o.z)) + o.d / 2, 0));
    // One template per model and size (things the player resized get their own).
    const byModel = Map.groupBy(shown, (o) => `${o.def}|${this.objectModel(o)}${resizeKey(this.objectResize(o))}${colorsKey(this.objectColors(o))}`);
    const lists = [...byModel.values()];
    const keys: string[] = [];
    const templates = await perfTimeAsync('render: object templates', async () => {
      // Generated models build as they're asked for: one at a time, so a build behind the overview can pause between them.
      const pending: Promise<ModelTemplate>[] = [];
      for (const list of lists) {
        await this.hooks.pace();
        const [key, build] = this.objectTemplate(list[0]);
        keys.push(key);
        pending.push(this.template(key, build));
      }
      return Promise.all(pending);
    });
    // What each tile's top belongs to, for the things standing on tops.
    const tops = new Map<string, number>();
    for (const o of shown) {
      if (!this.deps.content.object(o.def)?.surface) continue;
      for (let z = o.z; z < o.z + o.d; z++) for (let x = o.x; x < o.x + o.w; x++) tops.set(`${x}:${z}`, o.id);
    }
    const mirrors: MirrorInstance[] = [];
    const moving: MovingObject[] = [];
    const glows: Glow[] = [];
    const drawnKeys = new Set<string>();
    let partsCast = false;
    // (Low: the opaque bodies cast from one merged mesh, `objectShadows.ts`.)
    const shadowSources: ShadowSource[] = [];
    const m = this.vTmp;
    lists.forEach((list, n) => {
      const template = templates[n];
      const key = keys[n];
      drawnKeys.add(key);
      const d = this.drawsOf(key, template);
      const matrices = new Float32Array(list.length * 16);
      const entry = this.deps.assets.get(this.objectModel(list[0]), 'model');
      // Moving parts (a fridge's doors) draw from buffers of their own, each instance posed on its own.
      const motion = template.motion;
      const motions = motion && motionsOf(entry);
      const partCount = motion ? Math.max(-1, ...motion.parts) + 1 : 0;
      // (A part's glow, if any, moves with it.)
      const glowOf = (k: number) => (template.glows ?? []).flatMap((g, i) => (g.part === k ? [d.glows[i]] : []));
      const parts = Array.from({ length: partCount }, (_, k) => ({ meshes: [...d.draws.filter((_, i) => motion!.parts[i] === k), ...glowOf(k)], matrices: new Float32Array(list.length * 16) }));
      const vary = entry?.vary ?? 0;
      list.forEach((o, i) => {
        const { turn, size } = placementVariation(vary, o.x, o.z);
        const k = v.storeyOf(o.z);
        const D = v.rowOf(k);
        // Things on a top stand on it.
        const y0 = k * WALL_HEIGHT + (o.lift ?? 0);
        // Things that stand anywhere on their tile are moved off its middle (picking and the shadow follow).
        const centre = objectCentre(o);
        quatYawPitchRoll((o.rot * Math.PI) / 2 + ((o.turn ?? 0) * Math.PI) / 180 + turn, 0, 0, this.qTmp);
        matrices.set(compose(size, size, size, this.qTmp, centre.x, y0, centre.z - D, m), i * 16);
        const def = this.deps.content.object(o.def);
        const layer = def?.layer;
        const bottom = layer === 'wall' || layer === 'ceiling' ? Math.max(0, Math.min(template.bottom ?? 0, template.height - 0.05)) : 0;
        const [px, pz, pw, pd] = this.pickBox(o, template, motions);
        const pop: PopTarget = { meshes: [...d.draws.filter((_, j) => !motion || motion.parts[j] < 0), ...glowOf(-1)], matrices, index: i, ...(partCount ? { moving: { parts, poses: restPoses(partCount) } } : {}) };
        this.placed.push({
          id: o.id,
          y0,
          minX: px,
          minZ: pz,
          maxX: px + pw,
          maxZ: pz + pd,
          height: template.height,
          bottom,
          wallCut: layer === 'wall' ? this.wallCutBits(world, o) : undefined,
          host: layer === 'surface' ? tops.get(`${o.x}:${o.z}`) : undefined,
          pop,
          built: { template, motions },
        });
        if (motion && motions) {
          // A lamp's fan turns while it's on; a fire's flames (its uses' clips) burn while it's lit.
          const light = def?.light;
          const burns = light?.fire ? [...new Set(Object.values(motions.uses).flatMap((clips) => clips.map((c) => c.clip)))] : undefined;
          moving.push({ id: o.id, target: pop, motion, motions, interactions: (def?.interactions ?? []).map((it) => it.id), open: !!o.open, stage: o.growth, slotOf: slotOf(o, def), lamp: !!light && light.switch !== false, burns });
        }
        // Mirrors on a moving part (a mirror cabinet's door) follow that part's buffer.
        for (const [part, surfaces] of Map.groupBy(template.mirrors ?? [], (s) => s.part ?? -1)) {
          mirrors.push({ surfaces, matrices: part >= 0 && parts[part] ? parts[part].matrices : matrices, index: i });
        }
      });
      // At rest, the parts are where the body is.
      for (const part of parts) part.matrices.set(matrices);
      d.draws.forEach((draw, j) => {
        const part = motion ? motion.parts[j] : -1;
        draw.setMatrices(part < 0 ? matrices : parts[part].matrices, list.length);
        draw.update();
        draw.enabled = true;
        // On Low an opaque mesh casts from the merged mesh (its body) or not at all (a moving part);
        // something see-through casts from its own copies.
        const material = draw.meshInstance.material;
        const merged = !!this.objectShadows && !!material && material.blendType === pc.BLEND_NONE && !(material.alphaTest > 0);
        const render = draw.entity.render;
        if (render && render.castShadows === merged) render.castShadows = !merged;
        if (merged && part < 0) shadowSources.push({ mesh: draw.mesh, matrices, count: list.length });
        // (A moving part casting makes its moves draw the shadows again.)
        if (part >= 0 && render?.castShadows) partsCast = true;
      });
      (template.glows ?? []).forEach(({ part, screen }, j) => {
        // Strengths per instance (`applyGlow`), by the room each stands in; on a moving part, its buffer.
        const draw = d.glows[j];
        const still = part < 0 || !parts[part];
        draw.setMatrices(still ? matrices : parts[part].matrices, list.length);
        const colors = new Float32Array(list.length * 4);
        draw.setAll(INSTANCE_COLOR, colors);
        draw.update();
        draw.enabled = false;
        const W = world.width;
        const roomOf = (o: ObjectPlacement) => world.rooms[Math.floor(o.z + o.d / 2) * W + Math.floor(o.x + o.w / 2)] ?? 0;
        // A lamp's shade glows with the lamp (switched on or off), anything else's with its room's lights.
        const light = this.deps.content.object(list[0].def)?.light;
        glows.push({ draw, colors, rooms: list.map(roomOf), lamps: list.map((o) => (light && light.switch !== false ? o.id : -1)), ids: list.map((o) => o.id), screen: !!screen });
      });
    });
    this.movingPartsCast = partsCast;
    this.objectShadows?.build(shadowSources);
    // Models no longer drawn: their draws go off (their templates stay for the next build).
    for (const [key, d] of this.modelDraws) {
      if (drawnKeys.has(key)) continue;
      for (const draw of [...d.draws, ...d.glows]) {
        draw.setCount(0);
        draw.update();
        draw.enabled = false;
      }
    }
    this.glows = glows;
    // A night clip that lights a thing glowing with its room (the candles' flames) stands only while the room's lights are on.
    const glowRoom = new Map<number, number>();
    for (const g of glows) if (!g.screen) g.ids.forEach((id, k) => g.lamps[k] < 0 && g.rooms[k] && glowRoom.set(id, g.rooms[k]));
    this.motion.setUnlit((id) => {
      const room = glowRoom.get(id);
      return room !== undefined && this.glowLight.roomOn(room) < 0.5;
    });
    this.applyGlow(this.glowLight);
    this.mirrors = mirrors;
    this.motion.set(moving);
    this.buildContactShadows(shown);
    // What replaces the boxes of files that came is placed now: they're hidden (and go after the build).
    this.templates.hideStale();
    this.pullUpChairs(world);
    this.retireTemplates();
    this.showItems();
    this.hooks.shadowsMoved(true);
  }

  /** Soft contact shadows under every object, sized to its footprint (one instanced draw). */
  private buildContactShadows(shown: readonly ObjectPlacement[]): void {
    this.blobs?.destroy();
    this.blobs = null;
    if (!shown.length) return;
    const v = this.view;
    const blobs = new Float32Array(shown.length * 16);
    shown.forEach((o, i) => {
      const k = i * 16;
      const s = v.storeyOf(o.z);
      const D = v.rowOf(s);
      // Nothing on a wall, the ceiling or the floor's surface (rugs) sits on a contact shadow;
      // things on a top get a small one there, the size of what they take on it.
      const def = this.deps.content.object(o.def);
      const layer = def?.layer ?? 'floor';
      const flat = layer === 'floor' || layer === 'surface' ? 1 : 0;
      const [sw, sd] = layer === 'surface' ? [this.deps.content.radius(def) * 2.3, this.deps.content.radius(def) * 2.3] : [o.w * 1.15, o.d * 1.15];
      blobs[k] = sw * flat;
      blobs[k + 5] = flat;
      blobs[k + 10] = sd * flat;
      const centre = objectCentre(o);
      blobs[k + 12] = centre.x;
      blobs[k + 13] = 0.018 + s * WALL_HEIGHT + (o.lift ?? 0);
      blobs[k + 14] = centre.z - D;
      blobs[k + 15] = 1;
    });
    const draw = (this.blobs = new Instances(this.app.graphicsDevice, this.root, this.blob.mesh, this.blob.material, { name: 'objectShadows', castShadows: false, receiveShadows: false, capacity: shown.length }));
    draw.setMatrices(blobs, shown.length);
    draw.update();
    const index = new Map(shown.map((o, i) => [o.id, i]));
    for (const p of this.placed) {
      const i = index.get(p.id);
      if (i !== undefined) p.shadow = { meshes: [draw], matrices: blobs, index: i };
    }
  }

  /** Switches off the cached templates the world doesn't use, and lets go of those in a player's colours (their materials are shared and stay). */
  private retireTemplates(): void {
    for (const [key, t] of this.templates) {
      if (this.templatesInUse.has(key)) continue;
      void t.then((template) => {
        if (this.templatesInUse.has(key)) return;
        if (!this.paintedTemplates.has(key) || this.templates.peek(key) !== t) return;
        this.templates.delete(key);
        this.paintedTemplates.delete(key);
        const d = this.modelDraws.get(key);
        if (d?.template === template) {
          for (const draw of [...d.draws, ...d.glows]) draw.destroy();
          this.modelDraws.delete(key);
        }
        disposeTemplate(template);
      });
    }
  }

  /**
   * A world that only changed what's drawn of its objects (an opening clip, a crop's stage), applied
   * in place: their picking boxes, and the clips follow on from here.
   */
  patch(world: WorldStructure): void {
    const byId = new Map(world.objects.map((o) => [o.id, o]));
    for (const p of this.placed) {
      const o = byId.get(p.id);
      if (!o || !p.built) continue;
      // What's drawn of it can change shape (an opening clip); its tiles never do.
      const [px, pz, pw, pd] = this.pickBox(o, p.built.template, p.built.motions);
      [p.minX, p.minZ, p.maxX, p.maxZ] = [px, pz, px + pw, pz + pd];
    }
    // The motion reads both live every frame: a door swings, a crop grows, without a build.
    this.motion.patch(world.objects);
    this.showItems();
  }

  /**
   * A model file that failed has come: the templates built without it are set aside (still drawn
   * until the next build), those the world doesn't draw let go of now. Whether the world draws one
   * (then it's built again: `prebuild`, then the renderer's `setWorld`).
   */
  invalidate(url: string): boolean {
    const keys = this.templates.invalidate(url);
    this.templates.disposeStale(keys.filter((k) => !this.templatesInUse.has(k)));
    return keys.some((k) => this.templatesInUse.has(k));
  }

  /**
   * Builds the templates the objects of `world` need a frame at a time (the old ones are still
   * drawn), so the world build after it finds them ready instead of making them all in one frame.
   */
  async prebuild(world: WorldStructure): Promise<void> {
    const built = new Set<string>();
    for (const o of world.objects) {
      const [key, build] = this.objectTemplate(o);
      if (built.has(key) || this.templates.peek(key)) continue;
      built.add(key);
      await this.hooks.pace();
      await this.template(key, build).catch(() => {});
    }
  }

  /** After a world build: the templates set aside for files that came go (nothing draws them now). */
  disposeStale(): boolean {
    let any = false;
    for (const [key, d] of this.modelDraws) {
      if (this.templates.built(key) === d.template) continue;
      for (const draw of [...d.draws, ...d.glows]) draw.destroy();
      this.modelDraws.delete(key);
      any = true;
    }
    return this.templates.disposeStale() || any;
  }

  /** The lot's things go (the session ended): the templates built without their files are built afresh next time. */
  clear(): void {
    for (const d of this.modelDraws.values()) {
      for (const draw of [...d.draws, ...d.glows]) {
        draw.setCount(0);
        draw.update();
        draw.enabled = false;
      }
    }
    this.blobs?.destroy();
    this.blobs = null;
    this.objectShadows?.clear();
    this.templatesInUse.clear();
    this.retireTemplates();
    this.templates.dropFailed();
    this.disposeStale();
    this.placed = [];
    this.mirrors = [];
    this.glows = [];
    this.tables = new Map();
    this.pulledIn.clear();
    this.wallItemsHidden.clear();
    this.motion.clear();
    this.items.clear();
  }

  /** Food and dishes for the lot and storeys in view (after a build, a storey changed). */
  showItems(): void {
    const v = this.view;
    const storey = v.storey();
    this.items.show({ storey, storeyOf: (z) => v.storeyOf(z), shown: (k) => storeyShown(k, storey), groundZ: (z) => z - v.rowOf(v.storeyOf(z)), storeyHeight: WALL_HEIGHT, inView: (x, z) => v.inView(x, z, 0) });
  }

  /** New items from the simulation. */
  setItems(items: readonly ItemPlacement[]): void {
    this.items.set(items);
  }

  /** The lamps (and fires) switched off, by object id: a fan's blades stop, a fire's flames die down, a shade goes dark. */
  setLampsOff(off: ReadonlySet<number>): void {
    this.lampsOff = off;
    this.motion.setLampsOff(off);
  }

  /**
   * Things on a wall go with their wall: all of them with walls down (`down`), and in cutaway those
   * on the walls that drop (`bits`, from either side: `wallCutBits`). `keep`: one that stays shown
   * (held in build mode).
   */
  cutWallItems(bits: number, down: boolean, keep: number | null = null): void {
    for (const p of this.placed) {
      if (p.wallCut === undefined || p.id === keep) continue;
      const hide = down || (bits & p.wallCut) !== 0;
      const saved = this.wallItemsHidden.get(p.id);
      if (hide && !saved) {
        const m = p.pop.matrices.slice(p.pop.index * 16, p.pop.index * 16 + 16);
        this.wallItemsHidden.set(p.id, m);
        const gone = m.slice();
        gone.fill(0, 0, 12);
        this.write(p.pop, gone);
        this.hooks.shadowsMoved(true);
      } else if (!hide && saved) {
        this.wallItemsHidden.delete(p.id);
        this.write(p.pop, saved);
        this.hooks.shadowsMoved(true);
      }
    }
  }

  /** The tables the seats in the new world face (`tableRooms`), for the residents to pull their chairs up to; none pulled in yet. */
  private pullUpChairs(world: WorldStructure): void {
    this.pulledIn.clear();
    const byId = new Map(this.placed.map((p) => [p.id, p]));
    this.tables = tableRooms(world, this.deps.content, (id) => byId.get(id)?.pop, (o) => this.view.storeyOf(o.z) * WALL_HEIGHT);
    this.seatTurns = new Map(world.objects.filter((o) => this.tables.has(o.id)).map((o) => [o.id, o.rot]));
  }

  /** Draws the chairs residents sit pulled up to a table (`pulled`: by chair, how far in, m), and those let go back where they stand (the residents call it each frame). */
  pullChairs(pulled: ReadonlyMap<number, number>, held: number | null = null): void {
    for (const [id, was] of this.pulledIn) {
      if (pulled.has(id)) continue;
      for (const { target, matrix } of was.saved) this.write(target, matrix);
      this.pulledIn.delete(id);
      this.hooks.shadowsMoved(true);
    }
    for (const [id, by] of pulled) {
      const was = this.pulledIn.get(id);
      if ((was && Math.abs(was.by - by) < 0.001) || held === id) continue;
      const p = this.placed.find((o) => o.id === id);
      if (!p) continue;
      const targets = p.shadow ? [p.pop, p.shadow] : [p.pop];
      const saved = was?.saved ?? targets.map((target) => ({ target, matrix: target.matrices.slice(target.index * 16, target.index * 16 + 16) }));
      this.pulledIn.set(id, { by, saved });
      // Along its facing (the turn of its placement: the model's z).
      const yaw = ((this.seatTurns.get(id) ?? 0) * Math.PI) / 2;
      for (const { target, matrix } of saved) {
        const m = matrix.slice();
        m[12] += Math.sin(yaw) * by;
        m[14] += Math.cos(yaw) * by;
        this.write(target as PopTarget, m);
      }
      this.hooks.shadowsMoved(true);
    }
  }

  /** How object `id`'s seat has moved from rest (`ObjectMotion.ride`), for the residents on it. */
  ride(id: number): Float32Array | null {
    return this.motion.ride(id);
  }

  /** How far round object `id`'s loop is (a bike's pedals), for the rider's feet. */
  loopPhase(id: number): number | null {
    return this.motion.loopPhase(id);
  }

  // --- glows ---------------------------------------------------------------------------

  /**
   * The shades' and fires' glow (`lamps.ts` lights the rooms: `light` says how): each instance as
   * bright as its lamp, or its room's lights, at the lamps' level (by day too in a room without
   * windows); a glow with nothing lit isn't drawn. Screens are lit by use (`update`).
   */
  applyGlow(light: GlowLight): void {
    this.glowLight = light;
    for (const g of this.glows) {
      if (g.screen) {
        this.writeScreenGlow(g);
        continue;
      }
      let lit = false;
      g.rooms.forEach((room, i) => {
        const lamp = g.lamps[i];
        const lampOn = lamp >= 0 ? light.lampOn(lamp) : room ? light.roomOn(room) : 1;
        const f = (room && light.dimmed(room) ? 1 : light.level) * lampOn;
        g.colors.fill(f, i * 4, i * 4 + 3);
        g.colors[i * 4 + 3] = 1;
        lit ||= f > 0;
      });
      if (g.draw.destroyed) continue;
      g.draw.setAll(INSTANCE_COLOR, g.colors);
      g.draw.update(false);
      g.draw.enabled = lit;
    }
  }

  /**
   * Screens (a TV, a monitor) are lit only while a resident uses them, and flicker a little while
   * they run; the glow mesh's vertex colour is the screen's, its instance colour the brightness.
   */
  private writeScreenGlow(g: Glow): void {
    const level = this.glowLight.level;
    const t = this.screenTime / 1000;
    let changed = false;
    let on = false;
    for (let i = 0; i < g.ids.length; i++) {
      const id = g.ids[i];
      let f = 0;
      if (this.screenOn.has(id)) {
        on = true;
        const flicker = this.reducedMotion ? 1 : 0.9 + 0.1 * (0.5 + 0.5 * Math.sin(t * 17.3 + id * 1.7)) * (0.6 + 0.4 * Math.sin(t * 41.1 + id * 2.3));
        f = Math.fround((0.55 + 0.45 * level) * flicker);
      }
      if (g.colors[i * 4] === f) continue;
      g.colors.fill(f, i * 4, i * 4 + 3);
      g.colors[i * 4 + 3] = 1;
      changed = true;
    }
    if (g.draw.destroyed) return;
    // Uploaded only when a brightness changed; off (nobody watching) it isn't drawn at all.
    if (changed) {
      g.draw.setAll(INSTANCE_COLOR, g.colors);
      g.draw.update(false);
    }
    if (g.draw.enabled !== on) g.draw.enabled = on;
  }

  /** Which screens are on this frame: the objects residents are using (not just walking to). */
  private animateScreens(frame: FrameState): void {
    let any = false;
    for (const g of this.glows) any ||= g.screen;
    if (!any) return;
    const { curr, layout } = frame;
    const k = layout.sim;
    this.screenOn.clear();
    const count = curr[layout.header.simCount] | 0;
    for (let i = 0; i < count; i++) {
      const o = layout.headerLen + i * layout.simStride;
      if ((curr[o + k.action] | 0) < 0) continue;
      const id = curr[o + k.object] | 0;
      if (id >= 0) this.screenOn.add(id);
    }
    this.screenTime = frame.now;
    for (const g of this.glows) if (g.screen) this.writeScreenGlow(g);
  }

  // --- each frame ----------------------------------------------------------------------

  /**
   * The game's frame (not the overview's): doors that open while residents use what they're on,
   * ambient clips near the camera (the sun's shadows aren't drawn again for those), the screens.
   * Whether anything a use moves moved.
   */
  update(frame: FrameState): boolean {
    toGame(this.camera.getPosition(), this.eye);
    const moved = this.motion.update(frame, frame.layout.actions ?? [], this.eye);
    if (moved.moved && this.movingPartsCast) this.hooks.shadowsMoved(false);
    this.animateScreens(frame);
    return moved.moved;
  }

  // --- dev tools (`RendererDebugHooks`) ------------------------------------------------

  objectClips(id: number): ReadonlyMap<string, ObjectClipInfo> | null {
    return this.motion.clipsOf(id);
  }

  objectMotions(id: number): ObjectBackgroundClips | null {
    return this.motion.motionsOf(id);
  }

  objectPoints(id: number, max = 50_000): Float32Array | null {
    const pop = this.placed.find((p) => p.id === id)?.pop;
    if (!pop) return null;
    const m = pop.matrices.subarray(pop.index * 16, pop.index * 16 + 16);
    const all: number[] = [];
    for (const draw of pop.meshes) {
      const v: number[] = [];
      if (!draw.mesh.getPositions(v)) continue;
      const step = 3 * Math.max(1, Math.floor(v.length / 3 / max));
      for (let i = 0; i < v.length; i += step) {
        const [x, y, z] = [v[i], v[i + 1], v[i + 2]];
        all.push(x * m[0] + y * m[4] + z * m[8] + m[12], x * m[1] + y * m[5] + z * m[9] + m[13], x * m[2] + y * m[6] + z * m[10] + m[14]);
      }
    }
    return all.length ? Float32Array.from(all) : null;
  }

  /** Whether the thing on a wall `id` is hidden with its wall now (`cutWallItems`: not picked then). */
  wallItemHidden(id: number): boolean {
    return this.wallItemsHidden.has(id);
  }

  dispose(): void {
    this.clear();
    for (const n of this.natures.values()) n.landscape.dispose();
    this.natures.clear();
    this.natureIn.world = this.natureIn.town = null;
    for (const d of this.modelDraws.values()) for (const draw of [...d.draws, ...d.glows]) draw.destroy();
    this.modelDraws.clear();
    for (const [, t] of this.templates) void t.then((template) => disposeTemplate(template));
    this.templates.clear();
    this.items.dispose();
    this.sky.destroy();
    this.objectShadows?.destroy();
  }

  /** Writes an object's copy anew (`write`): build mode's hand hides one and shows it again (`placing.ts`). */
  rewrite(target: PopTarget, matrix: ArrayLike<number>): void {
    this.write(target, matrix);
  }

  /** Writes an object's copy anew (`writeInstance`), and on Low its body in the merged shadow caster. */
  private write(target: PopTarget, matrix: ArrayLike<number>): void {
    writeInstance(target, matrix);
    this.objectShadows?.bake(target);
  }
}

const blobs = new WeakMap<pc.AppBase, { mesh: pc.Mesh; material: pc.StandardMaterial }>();

/**
 * Soft round contact shadows (objects per world; residents per frame) from one radial
 * texture: a flat square, black, its alpha the texture's, drawn over the floor without writing depth.
 */
export function contactBlob(app: pc.AppBase): { mesh: pc.Mesh; material: pc.StandardMaterial } {
  const known = blobs.get(app);
  if (known) return known;
  const device = app.graphicsDevice;
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = 64;
  const ctx = canvas.getContext('2d')!;
  const g = ctx.createRadialGradient(32, 32, 0, 32, 32, 32);
  g.addColorStop(0, 'rgba(0,0,0,0.55)');
  g.addColorStop(0.55, 'rgba(0,0,0,0.3)');
  g.addColorStop(1, 'rgba(0,0,0,0)');
  ctx.fillStyle = g;
  ctx.fillRect(0, 0, 64, 64);
  const texture = new pc.Texture(device, { name: 'blob', width: 64, height: 64, format: pc.PIXELFORMAT_RGBA8, mipmaps: true, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE });
  texture.setSource(canvas);
  const material = new pc.StandardMaterial();
  material.name = 'blobShadow';
  material.useLighting = false;
  material.useSkybox = false;
  material.diffuse.set(0, 0, 0);
  material.emissive.set(0, 0, 0);
  material.opacityMap = texture;
  material.opacityMapChannel = 'a';
  material.blendType = pc.BLEND_NORMAL;
  material.depthWrite = false;
  // (Its depth pulled towards the camera, off the floor it lies on.)
  material.slopeDepthBias = -3;
  material.cull = pc.CULLFACE_NONE;
  material.update();
  // A 1 m square on the floor, u along +x and v along +z.
  const mesh = new pc.Mesh(device);
  mesh.setPositions([-0.5, 0, 0.5, 0.5, 0, 0.5, -0.5, 0, -0.5, 0.5, 0, -0.5]);
  mesh.setNormals([0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0]);
  mesh.setUvs(0, [0, 1, 1, 1, 0, 0, 1, 0]);
  mesh.setIndices([3, 1, 0, 2, 3, 0]);
  mesh.update(pc.PRIMITIVE_TRIANGLES);
  const made = { mesh, material };
  blobs.set(app, made);
  return made;
}
