/**
 * The traffic lights in the game on PlayCanvas (sim-core `traffic.rs`): a pole with its three-lamp
 * head (`model.trafficLight`, tools/art/blender/families/street_furniture.py) at each signal head
 * the snapshot has (`SnapshotLayout.light`, read by `shared/signals.ts`), standing where its row
 * says, its head turned to the traffic coming in; hidden outside the chunks streamed in and far off
 * (`setView`), as the cars are.
 *
 * Every head is one instanced draw of the model (and one of its far version for the heads past
 * `FAR_MODELS`; on Low, the far version for all, as the street's props). Its lenses are dark glass
 * in the model; the lamp a head shows glows on top of its lens, each colour an instanced draw of
 * that lens's glow (`BuildOptions.glowApart`) holding the heads showing it: at most three more
 * draws for every light in town, by day as at night, brighter as the lamps come up (`setNight`).
 *
 * Below the traffic head each near pole has a pedestrians' head facing across the road
 * (`shared/signals.ts` `pedestrianHead`): its housing one instanced draw, its two figures' glows
 * one each (walk while the junction's walk phase is on, don't walk otherwise): three draws more.
 *
 * Per frame (`update`) the rows are read into arrays made once and compared; the draws are written
 * again only when a head's lamp changes, the heads change (a new world), or the view does. Nothing
 * is allocated per frame.
 */

import * as pc from './engine';
import { Instances } from './instancing';
import { MaterialLibrary } from './materials';
import { meshFrom } from './meshes';
import { buildModel, disposeTemplate, type ModelMesh, type ModelTemplate } from './models';
import type { AssetRegistry } from '../../assets/registry';
import type { ModelEntry } from '../../assets/types';
import type { FrameState } from '../../core/bridge';
import { chunkAt } from '../shared/chunks';
import { compose, quatYawPitchRoll, type Quat } from '../shared/math';
import type { Geo } from '../shared/geometry';
import { pedestrianHead, SIGNAL_LAMPS, SignalRows } from '../shared/signals';

/** The model's key in the manifests. */
export const SIGNAL_MODEL = 'model.trafficLight';
/** Heads further than this from the camera's target aren't drawn (m), as the cars (`vehicles.ts` `FAR`). */
const FAR = 650;
/** Heads further than this (m) draw the far version (the street's models' `FAR_MODELS`). */
const FAR_MODELS = 100;
/** The view is worked out again once the camera's target has moved this far (m). */
const REFRAME = 4;
/** How bright a lit lens glows by day and at night (the glow material's emissive intensity). */
const GLOW_DAY = 1.6;
const GLOW_NIGHT = 1.1;

export interface SignalOptions {
  /** Whether the poles cast the sun's shadows (`QualitySettings.streetShadows`). */
  shadows: boolean;
  /** 1: the far version for every head (`QualitySettings.streetLod`, Low). */
  lod: 0 | 1;
}

/** The model, its far version and the draws of each. */
interface Drawn {
  near: ModelTemplate;
  far: ModelTemplate | null;
  body: Instances[];
  farBody: Instances[];
  /** Per lamp (`SIGNAL_LAMPS`), its lens's glow (null: the model has none). */
  glows: (Instances | null)[];
  /** The pedestrians' heads (near poles only): their housings, and their don't-walk and walk figures' glows (null: no glow material). */
  ped: { housing: Instances; dontWalk: Instances | null; walk: Instances | null; meshes: pc.Mesh[] } | null;
}

export class Signals {
  private readonly lib: MaterialLibrary;
  private readonly root: pc.Entity;
  private readonly rows = new SignalRows();
  private drawn: Drawn | null = null;
  /** The templates are being built (a load in flight), and how many builds there have been (a newer one wins). */
  private loading = false;
  private builds = 0;
  /** The lens glows' material: the model's glow material, its strength the time of day's (`setNight`). */
  private glowMat: pc.StandardMaterial | null = null;
  private glowLevel = GLOW_DAY;
  /** The draws are to be written again (the heads, a lamp or the view changed). */
  private due = false;
  /** The chunks streamed in (null: all of them) and the camera's target where the view was last worked out (NaN: none yet, every head near). */
  private chunks: ReadonlySet<number> | null = null;
  private eyeX = NaN;
  private eyeZ = NaN;
  /** The view changed since the draws were written (which heads are drawn, and how). */
  private reframed = false;
  /** Whether the poles changed where they cast into the sun's map (the map is to be drawn again). */
  moved = false;
  /** Scratch: a matrix, a turn, each draw's count as it's written. */
  private readonly m = new Float32Array(16);
  private readonly q: Quat = [0, 0, 0, 1];
  private readonly lit = [0, 0, 0];
  private destroyed = false;

  constructor(
    private readonly app: pc.AppBase,
    parent: pc.Entity,
    private readonly assets: AssetRegistry,
    private readonly options: SignalOptions,
  ) {
    this.lib = MaterialLibrary.for(app);
    this.root = new pc.Entity('signals');
    parent.addChild(this.root);
  }

  /** The heads where the newest snapshot has them, each showing its lamp. */
  update(frame: FrameState): void {
    this.moved = false;
    const change = this.rows.read(frame.curr, frame.layout);
    if (change.placed || change.lit) this.due = true;
    if (!this.due) return;
    if (!this.drawn) {
      // (The model the first time there are lights: a town without any never loads it.)
      if (this.rows.count > 0 && !this.loading) void this.load();
      return;
    }
    this.write(change.placed);
  }

  /** The chunks streamed in (null: the whole town) and the camera's target (x, z): heads outside them, or further than `FAR`, aren't drawn. */
  setView(chunks: ReadonlySet<number> | null, x: number, z: number): void {
    const moved = !(Math.hypot(x - this.eyeX, z - this.eyeZ) < REFRAME);
    if (chunks === this.chunks && !moved) return;
    this.chunks = chunks;
    if (moved) {
      this.eyeX = x;
      this.eyeZ = z;
    }
    this.reframed = true;
    if (this.rows.count > 0) this.due = true;
  }

  /** The lit lenses brighter as the lamps come up (`Lighting.lamps`, 0 by day to 1 at night). */
  setNight(lamps: number): void {
    const g = Math.min(1, Math.max(0, (lamps - 0.3) / 0.45));
    this.glowLevel = GLOW_DAY + (GLOW_NIGHT - GLOW_DAY) * g;
    const mat = this.glowMat;
    if (mat && Math.abs(mat.emissiveIntensity - this.glowLevel) > 1e-3) {
      mat.emissiveIntensity = this.glowLevel;
      mat.update();
    }
  }

  /** A model file that failed has come: the model is built again with it. True when it was this one's. */
  fileArrived(url: string): boolean {
    const d = this.drawn;
    if (!d || !(d.near.failed?.includes(url) || d.far?.failed?.includes(url))) return false;
    this.drop();
    this.due = true;
    return true;
  }

  /** No heads (a new world is built: the next snapshot's are placed). */
  clear(): void {
    this.rows.clear();
    const d = this.drawn;
    if (d) for (const draw of [...d.body, ...d.farBody, ...d.glows, ...pedDraws(d)]) if (draw) hide(draw);
    this.due = false;
  }

  destroy(): void {
    this.destroyed = true;
    this.drop();
    this.glowMat?.destroy();
    this.glowMat = null;
    this.root.destroy();
  }

  /** Builds the model (and its far version) and their draws, then draws the heads. */
  private async load(): Promise<void> {
    const build = ++this.builds;
    this.loading = true;
    try {
      const entry = this.assets.has(SIGNAL_MODEL, 'model') ? (this.assets.get(SIGNAL_MODEL, 'model') as ModelEntry) : undefined;
      const { lod, shadows } = this.options;
      // (On Low the far version is the model: `lod` 1 builds it from the far file.)
      const [near, far] = await Promise.all([
        buildModel(this.app, 'signal', entry, [1, 1], { lod, glowApart: true }),
        lod === 0 && entry?.far ? buildModel(this.app, 'signal:far', entry, [1, 1], { far: true }) : Promise.resolve(null),
      ]);
      if (this.destroyed || build !== this.builds) {
        disposeTemplate(near);
        if (far) disposeTemplate(far);
        return;
      }
      const body = near.meshes.map((m) => this.draw(m, m.material, shadows));
      const farBody = far && !far.boxed ? far.meshes.map((m) => this.draw(m, m.material, shadows)) : [];
      const glows = SIGNAL_LAMPS.map((name) => {
        const glow = near.glows?.find((g) => g.material === `lens_${name}`);
        if (!glow) return null;
        if (!this.glowMat) {
          // (The model's glow material, with a strength of its own.)
          const mat = (glow.mesh.material as pc.StandardMaterial).clone();
          mat.name = 'signalGlow';
          mat.emissiveIntensity = this.glowLevel;
          mat.update();
          this.glowMat = mat;
        }
        return this.draw(glow.mesh, this.glowMat, false);
      });
      this.drawn = { near, far, body, farBody, glows, ped: this.pedestrians(shadows) };
      this.write(true);
    } finally {
      if (build === this.builds) this.loading = false;
    }
  }

  /** Every head drawn again where it is: near ones in detail, far ones in their far version, each lit lens in its lamp's glow. */
  private write(placed: boolean): void {
    const d = this.drawn;
    if (!d) return;
    this.due = false;
    // (Which heads are drawn and how changed: the poles' draws and shadows with them.)
    const moved = placed || this.reframed;
    this.reframed = false;
    const { rows, m, q, lit } = this;
    const far = d.farBody.length > 0;
    if (moved) {
      for (const draw of d.body) draw.setCount(rows.count);
      for (const draw of d.farBody) draw.setCount(rows.count);
    }
    for (const draw of d.glows) draw?.setCount(rows.count);
    const ped = d.ped;
    if (ped) {
      if (moved) ped.housing.setCount(rows.count);
      ped.dontWalk?.setCount(rows.count);
      ped.walk?.setCount(rows.count);
    }
    let near = 0;
    let away = 0;
    let walking = 0;
    let waiting = 0;
    lit[0] = lit[1] = lit[2] = 0;
    const looked = !Number.isNaN(this.eyeX);
    for (let k = 0; k < rows.count; k++) {
      const x = rows.x[k];
      const z = rows.z[k];
      if (this.chunks && !this.chunks.has(chunkAt(x, z))) continue;
      const dist = looked ? Math.hypot(x - this.eyeX, z - this.eyeZ) : 0;
      if (dist > FAR) continue;
      compose(1, 1, 1, quatYawPitchRoll(rows.yaw[k], 0, 0, q), x, 0, z, m);
      if (far && dist > FAR_MODELS) {
        if (moved) for (const draw of d.farBody) draw.setMatrix(away, m);
        away++;
      } else {
        if (moved) for (const draw of d.body) draw.setMatrix(near, m);
        if (ped) {
          if (moved) ped.housing.setMatrix(near, m);
          if (rows.walk[k]) ped.walk?.setMatrix(walking++, m);
          else ped.dontWalk?.setMatrix(waiting++, m);
        }
        near++;
      }
      const lamp = rows.lamp[k];
      const glow = d.glows[lamp];
      if (glow) glow.setMatrix(lit[lamp]++, m);
    }
    // (Only the lamps changed: the poles stand as they stood.)
    if (moved) {
      for (const draw of d.body) show(draw, near);
      for (const draw of d.farBody) show(draw, away);
      if (ped) show(ped.housing, near);
    }
    if (ped?.walk) show(ped.walk, walking);
    if (ped?.dontWalk) show(ped.dontWalk, waiting);
    for (let lamp = 0; lamp < d.glows.length; lamp++) {
      const draw = d.glows[lamp];
      if (draw) show(draw, lit[lamp]);
    }
    if (moved && this.options.shadows) this.moved = true;
  }

  /** An instanced draw of `mesh` under the root, out of the house's lamps. */
  private draw(mesh: ModelMesh, material: pc.Material, cast: boolean): Instances {
    const draw = new Instances(this.app.graphicsDevice, this.root, mesh.mesh, material, { name: mesh.name, castShadows: cast, receiveShadows: true, capacity: Math.max(1, this.rows.count) });
    this.lib.excludeFromLights([], [draw.meshInstance]);
    draw.enabled = false;
    return draw;
  }

  /** The pedestrians' heads' draws: the housing in the plain finish, the figures in the lenses' glow (none without it). */
  private pedestrians(shadows: boolean): Drawn['ped'] {
    const head = pedestrianHead();
    const upload = (g: Geo) => meshFrom(this.app.graphicsDevice, g.meshData({ colors: true }));
    const housing = upload(head.housing);
    if (!housing) return null;
    const meshes = [housing];
    const glow = (g: Geo, name: string) => {
      const mesh = this.glowMat && upload(g);
      if (!mesh) return null;
      meshes.push(mesh);
      return this.draw({ name, mesh, material: this.glowMat! }, this.glowMat!, false);
    };
    return {
      housing: this.draw({ name: 'signal|pedestrians', mesh: housing, material: this.lib.plainFinish() }, this.lib.plainFinish(), shadows),
      dontWalk: glow(head.dontWalk, 'signal|dontWalk'),
      walk: glow(head.walk, 'signal|walk'),
      meshes,
    };
  }

  /** The draws and the templates let go (built again when there are heads to draw). */
  private drop(): void {
    this.builds++;
    this.loading = false;
    const d = this.drawn;
    if (!d) return;
    for (const draw of [...d.body, ...d.farBody, ...d.glows, ...pedDraws(d)]) draw?.destroy();
    for (const mesh of d.ped?.meshes ?? []) mesh.destroy();
    disposeTemplate(d.near);
    if (d.far) disposeTemplate(d.far);
    this.drawn = null;
  }
}

/** The pedestrians' heads' draws. */
function pedDraws(d: Drawn): (Instances | null)[] {
  return d.ped ? [d.ped.housing, d.ped.dontWalk, d.ped.walk] : [];
}

/** A draw of `count` copies (written), drawn while there are any. */
function show(draw: Instances, count: number): void {
  draw.setCount(count);
  draw.update();
  draw.enabled = count > 0;
}

function hide(draw: Instances): void {
  draw.setCount(0);
  draw.update(false);
  draw.enabled = false;
}
