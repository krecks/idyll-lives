/**
 * Garden birds on PlayCanvas (the port of `babylon/birds.ts`): the engine-neutral birds
 * (`shared/birds.ts`: who lives where, how they perch, fly, forage, sing and sleep, and every matrix
 * drawn) as instanced draws (`instancing.ts`), one per mesh of each species' model, its matrices in
 * game space under the world root; only the parts written in a frame go up. They receive the sun's
 * shadows and cast none, and aren't culled (they're spread over the lot), as on Babylon.
 *
 * `GameBirds` is the renderer's side of them, as Babylon's renderer had it: where they live once the
 * lot is built (the viewed lot's garden dressing, the garden things outdoors, the street's plants,
 * the woods nearby, the viewed house's roof ridge), what they're told each frame (the game's clock
 * and speed, the hour, the camera, the residents' heads, reduce motion, the roof), hidden with the
 * garden while the basement is in view and in the town overview, and what the world's sound hears of
 * them. Nothing is allocated per frame.
 */

import type * as pc from './engine';
import { Instances } from './instancing';
import { buildModel, disposeTemplate, type ModelMesh, type ModelTemplate } from './models';
import { windSway } from './nature';
import { TemplateCache } from './templateCache';
import type { AssetRegistry } from '../../assets/registry';
import type { Content } from '../../content/content';
import type { FrameState } from '../../core/bridge';
import type { WorldStructure } from '../../core/protocol';
import type { SeenBird, ViewRect } from '../types';
import { BIRD_PARTS, birdLook, Birds as SharedBirds, noBirdLook, outdoorPlants, roofRidge, type BirdEntry, type BirdFrame, type BirdHabitat, type BirdLook, type BirdQuality, type BirdView } from '../shared/birds';
import { decorPlants, type Decor, type LandscapeLayout, type PlantPlacement } from '../shared/nature';

/** A species' look on PlayCanvas: its template's meshes (for `TemplateCache`), and each mesh's instanced draw, all and by part. */
interface Look extends BirdLook {
  meshes: ModelMesh[];
  draws: Instances[];
  parts: Instances[][];
}

/** The birds drawn by PlayCanvas under `root` (game space). */
export class Birds extends SharedBirds<Look> {
  constructor(app: pc.AppBase, root: pc.Entity, assets: AssetRegistry, debugging = false) {
    super(birdView(app, root, assets), assets, debugging);
  }
}

/** Draws `count` matrices of `buffer` (16 floats each) with `draw`. */
function upload(draw: Instances, buffer: Float32Array, count: number): void {
  draw.setCount(count);
  for (let i = 0; i < count; i++) draw.setMatrix(i, buffer, i * 16);
  draw.update(false);
}

/** The birds' engine side on a PlayCanvas app. */
function birdView(app: pc.AppBase, root: pc.Entity, assets: AssetRegistry): BirdView<Look> {
  const looks = new TemplateCache<Look>(app);
  /** Looks set aside (a file came), their draws to go with them (`disposeStale`). */
  let stale: Look[] = [];
  const destroy = (look: Look) => {
    for (const d of look.draws) d.destroy();
  };
  return {
    look: (id, key) =>
      looks.get(key, async () => {
        const entry = assets.get(key, 'model') as BirdEntry | undefined;
        const template = await buildModel(app, `bird:${id}`, entry, [1, 1], { parts: BIRD_PARTS });
        if (entry && template.meshes.length && template.motion) return lookFrom(app, root, entry, template);
        disposeTemplate(template);
        return { ...noBirdLook(template.failed), meshes: [], draws: [], parts: [] };
      }),
    invalidate: (url) => {
      looks.forEachBuilt((look) => {
        if (look.failed?.includes(url)) stale.push(look);
      });
      return looks.invalidate(url).length > 0;
    },
    disposeStale: () => {
      for (const look of stale) destroy(look);
      stale = [];
      looks.disposeStale();
    },
    show: (visible) =>
      looks.forEachBuilt((look) => {
        for (const d of look.draws) d.enabled = visible && look.count > 0;
      }),
    setBuffers: (look, visible) =>
      look.parts.forEach((list, k) => {
        for (const d of list) {
          upload(d, look.buffers[k], look.count);
          d.enabled = visible && look.count > 0;
        }
      }),
    updated: (look, part) => {
      const list = look.parts[part];
      for (let i = 0; i < list.length; i++) upload(list[i], look.buffers[part], look.count);
    },
    sway: (x, z) => windSway(app, x, z),
    dispose: () => {
      looks.forEachBuilt((look) => {
        destroy(look);
        disposeTemplate(look);
      });
      looks.clear();
      for (const look of stale) destroy(look);
      stale = [];
      looks.disposeStale();
    },
  };
}

/** A species' look from its model: an instanced draw per mesh, hidden until it has birds. */
function lookFrom(app: pc.AppBase, root: pc.Entity, entry: BirdEntry, template: ModelTemplate): Look {
  const motion = template.motion!;
  const parts: Instances[][] = BIRD_PARTS.map(() => []);
  const draws = template.meshes.map((m, k) => {
    const d = new Instances(app.graphicsDevice, root, m.mesh, m.material, { name: m.name, castShadows: false, receiveShadows: true, cull: false });
    d.enabled = false;
    parts[Math.max(0, motion.parts[k])].push(d);
    return d;
  });
  return { ...birdLook(entry, motion.rests), meshes: template.meshes, draws, parts };
}

/** What the birds need of the renderer (read when they're settled and each frame). */
export interface BirdsHost {
  /** The camera's position (game space). */
  eye(out: { x: number; y: number; z: number }): void;
  /** The storey in view (the garden is dug away over the basement: the birds hide). */
  storey(): number;
  /** The viewed house's roof is drawn (doves may sit on its ridge). */
  roofShown(): boolean;
  /** Reduce motion. */
  calm(): boolean;
}

/** The viewed lot's birds as the renderer keeps them (see above). */
export class GameBirds {
  readonly life: Birds;
  /** The viewed lot's garden dressing as placed (a new array as the lot is built again). */
  private decor: readonly PlantPlacement[] = [];
  private readonly frame: BirdFrame;
  private readonly eye = { x: 0, y: 0, z: 0 };
  private town = false;
  private storey = 0;

  constructor(
    app: pc.AppBase,
    root: pc.Entity,
    private readonly assets: AssetRegistry,
    private readonly content: Content,
    private readonly quality: BirdQuality,
    private readonly host: BirdsHost,
    debug = false,
  ) {
    this.life = new Birds(app, root, assets, debug);
    this.life.setQuality(quality);
    this.frame = { dt: 0, speed: 0, hour: 0, camera: this.eye, residents: new Float32Array(0), residentCount: 0, visible: new Uint8Array(0), calm: false, roof: false };
    // `?debug`: `__birds()` lists them (species, state, place) with what they did and the cost of a
    // frame; `__birdFly(id, where)` sends one off; `__birdTime(scale)` slows their clock.
    if (debug) Object.assign(window, { __birds: () => this.life.debug(), __birdFly: (id: number, where: 'plant' | 'ground' | 'roof' | 'home' = 'plant') => this.life.debugFly(id, where), __birdTime: (scale: number) => (this.life.timeScale = scale) });
  }

  /** The viewed lot's garden dressing as built (`natureDecor`'s, the hedges, shrubs and lot trees). */
  setDecor(decor: readonly [string, Decor][]): void {
    this.decor = decorPlants(decor);
  }

  /**
   * Where the birds live, now that the lot is built: the viewed lot's garden trees and shrubs, the
   * garden things outdoors (catalog trees, hedges, roses), the street's bushes, hedges and public
   * places' trees, the woods nearby, and the viewed house's roof ridge (`roofs`: its roof entities).
   */
  async settle(world: WorldStructure, view: ViewRect | null, minute: number, near: { landscape: LandscapeLayout | null; street: readonly PlantPlacement[]; roofs: readonly pc.Entity[]; houseTop: BirdHabitat['houseTop'] }): Promise<void> {
    const v = view;
    const woods = v && near.landscape?.woodsNear(v.x + v.w / 2, v.z + v.d / 2, Math.hypot(v.w, v.d) / 2 + this.quality.range);
    const { ridge, top } = roofRidge((fn) => eachVertex(near.roofs, fn));
    await this.life.setHabitat({ world, view: v, plants: [this.decor, outdoorPlants(world, this.content, this.assets), near.street, woods ?? []], ridge, roofTop: top, houseTop: near.houseTop, hour: (minute / 60) % 24 });
  }

  /**
   * The birds a frame on, with the game's clock (`dt` ms; calm hours at speed 4), the hour of `minute`,
   * the camera and the residents' heads (`count` of them, the ones drawn as `visible` says).
   */
  update(state: FrameState, minute: number, dt: number, residents: { heads: Float32Array; visible: Uint8Array }, count: number): boolean {
    // (Behind the overview they wait, hidden: `setTown`.)
    if (this.town) return false;
    const { curr, layout } = state;
    const h = layout.header;
    const calmHours = h.calm !== undefined && curr[h.calm] > 0;
    this.storey = this.host.storey();
    this.applyVisible();
    const f = this.frame;
    f.dt = dt / 1000;
    f.speed = calmHours ? 4 : curr[h.speed] | 0;
    f.hour = (minute / 60) % 24;
    this.host.eye(this.eye);
    f.residents = residents.heads;
    f.residentCount = count;
    f.visible = residents.visible;
    f.calm = this.host.calm();
    f.roof = this.host.roofShown() && this.storey >= 0;
    return this.life.update(f);
  }

  /** In the town overview (the game world isn't drawn): hidden, still, and none heard. (Called again with the same, nothing changes.) */
  setTown(on: boolean): void {
    this.town = on;
    this.applyVisible();
  }

  private applyVisible(): void {
    this.life.setVisible(!this.town && this.storey >= 0);
  }

  /** The birds as the world's sound hears them (none behind the overview). */
  heard(): readonly SeenBird[] {
    return this.town ? NONE : this.life.heard();
  }

  sings(id: number, seconds: number): void {
    this.life.sings(id, seconds);
  }

  fileArrived(url: string): void {
    this.life.fileArrived(url);
  }

  /** The world is gone (`Renderer.clear`). */
  clear(): void {
    this.life.clear();
    this.decor = [];
  }

  dispose(): void {
    this.life.dispose();
  }
}

const NONE: readonly SeenBird[] = [];

/** Every vertex of the meshes `entities` draw, in game space (the world root's mirror undone). */
function eachVertex(entities: readonly pc.Entity[], fn: (x: number, y: number, z: number) => void): void {
  const pos: number[] = [];
  for (const entity of entities) {
    const m = entity.getWorldTransform().data;
    for (const mi of entity.render?.meshInstances ?? []) {
      pos.length = 0;
      const n = mi.mesh?.getPositions(pos) ?? 0;
      for (let i = 0; i < n; i++) {
        const [x, y, z] = [pos[i * 3], pos[i * 3 + 1], pos[i * 3 + 2]];
        fn(x * m[0] + y * m[4] + z * m[8] + m[12], x * m[1] + y * m[5] + z * m[9] + m[13], -(x * m[2] + y * m[6] + z * m[10] + m[14]));
      }
    }
  }
}
