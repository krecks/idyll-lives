/**
 * Bikes and e-scooters (content `vehicle`, sim-core `ride.rs`) standing on lots other than the one viewed, on
 * PlayCanvas. The viewed lot's are its own objects (`lotContents.ts`); the street draws the rest of what stands
 * in other gardens, but not these: they go off with their riders, and aren't drawn where they stand while
 * ridden (`hide`: the residents draw them under their riders, `shared/characters/vehicles.ts`).
 *
 * One instanced draw per mesh of each model in the player's colours for it, shared by every copy: the model
 * near the camera's target (`NEAR`), its far version further off; none past `FAR`, nor outside the chunks
 * streamed in (`setView`), nor on a lot hidden while its basement is in view (`hideOn`).
 */

import * as pc from './engine';
import { Instances } from './instancing';
import { buildModel, disposeTemplate, type ModelTemplate } from './models';
import type { AssetRegistry } from '../../assets/registry';
import type { Content } from '../../content/content';
import { colorsKey, objectCentre, type ObjectPlacement, type WorldStructure } from '../../core/protocol';
import { chunkAt } from '../shared/chunks';
import { compose, quatYawPitchRoll, type Quat } from '../shared/math';
import { painted, placementVariation } from '../shared/modelEntry';

/** Within this far of the camera's target (m), the whole model; further, its far version. */
const NEAR = 45;
/** Further than this from the camera's target (m), not drawn. */
const FAR = 320;

interface Parked {
  id: number;
  x: number;
  z: number;
  /** Its placement (game space), worked out once. */
  matrix: ArrayLike<number>;
  look: Look;
}

/** A model in the player's colours for it: its near and far templates' draws, built once. */
interface Look {
  near: Instances[];
  far: Instances[];
  templates: ModelTemplate[];
}

export class ParkedRides {
  private readonly root: pc.Entity;
  private parked: Parked[] = [];
  private readonly looks = new Map<string, Look>();
  private hidden: ReadonlySet<number> = new Set();
  private chunks: ReadonlySet<number> | null = null;
  private eye: { x: number; z: number } | null = null;
  private pit: readonly [number, number, number, number] | null = null;
  /** Bumped by each build: templates that come after another build has begun are let go of. */
  private generation = 0;
  private readonly q: Quat = [0, 0, 0, 1];

  constructor(
    private readonly app: pc.AppBase,
    parent: pc.Entity,
    private readonly assets: AssetRegistry,
    private readonly content: Content,
    private readonly shadows: boolean,
  ) {
    this.root = new pc.Entity('parkedRides');
    parent.addChild(this.root);
  }

  /** Whether the street leaves `def` to this (a thing to ride). */
  static rides(content: Content, def: string): boolean {
    return !!content.object(def)?.vehicle;
  }

  /** The world's things to ride that stand off the viewed lot (`viewed`: whether a spot is on it), outdoors on the ground storey. */
  async build(world: WorldStructure, viewed: (x: number, z: number) => boolean): Promise<void> {
    const generation = ++this.generation;
    const ground = world.storeyDepth ?? world.depth;
    const list = world.objects.filter((o) => ParkedRides.rides(this.content, o.def) && o.z + o.d <= ground && !viewed(o.x + o.w / 2, o.z + o.d / 2));
    const keyOf = (o: ObjectPlacement) => `${o.def}${colorsKey(o.colors)}`;
    const wanted = new Map(list.map((o) => [keyOf(o), o]));
    for (const [key, look] of this.looks) {
      if (wanted.has(key)) continue;
      this.dropLook(look);
      this.looks.delete(key);
    }
    // The looks not built yet, all at once.
    await Promise.all(
      [...wanted].map(async ([key, o]) => {
        if (this.looks.has(key)) return;
        const look = await this.makeLook(key, o);
        if (generation !== this.generation) this.dropLook(look);
        else this.looks.set(key, look);
      }),
    );
    if (generation !== this.generation) return;
    this.parked = list.flatMap((o) => {
      const look = this.looks.get(keyOf(o));
      if (!look) return [];
      const c = objectCentre(o);
      const model = this.content.object(o.def)?.model ?? '';
      const { turn, size } = placementVariation(this.assets.get(model, 'model')?.vary ?? 0, o.x, o.z);
      const yaw = (o.rot * Math.PI) / 2 + ((o.turn ?? 0) * Math.PI) / 180 + turn;
      return [{ id: o.id, x: c.x, z: c.z, matrix: compose(size, size, size, quatYawPitchRoll(yaw, 0, 0, this.q), c.x, 0, c.z, new Float32Array(16)), look }];
    });
    this.write();
  }

  /** Those ridden now (stable ids): not drawn where they stand. */
  hide(ids: ReadonlySet<number>): void {
    this.hidden = ids;
    this.write();
  }

  /** The chunks streamed in (null: all) and the camera's target (x, z). */
  setView(chunks: ReadonlySet<number> | null, x: number, z: number): void {
    this.chunks = chunks;
    this.eye = { x, z };
    this.write();
  }

  /** Those standing in here (x0, z0, x1, z1) aren't drawn (a lot whose basement is in view), or none (null). */
  hideOn(rect: readonly [number, number, number, number] | null): void {
    this.pit = rect;
    this.write();
  }

  clear(): void {
    this.generation++;
    this.parked = [];
    for (const look of this.looks.values()) this.dropLook(look);
    this.looks.clear();
  }

  destroy(): void {
    this.clear();
    this.root.destroy();
  }

  /** Every copy written again where it's drawn (near or far) or left out. */
  private write(): void {
    const near = new Map<Look, Parked[]>();
    const far = new Map<Look, Parked[]>();
    for (const p of this.parked) {
      if (this.hidden.has(p.id) || (this.chunks && !this.chunks.has(chunkAt(p.x, p.z)))) continue;
      const r = this.pit;
      if (r && p.x > r[0] && p.x < r[2] && p.z > r[1] && p.z < r[3]) continue;
      const d = this.eye ? Math.hypot(p.x - this.eye.x, p.z - this.eye.z) : 0;
      if (d > FAR) continue;
      const into = d <= NEAR ? near : far;
      const list = into.get(p.look);
      if (list) list.push(p);
      else into.set(p.look, [p]);
    }
    for (const look of this.looks.values()) {
      for (const [draws, list] of [
        [look.near, near.get(look) ?? []],
        [look.far, far.get(look) ?? []],
      ] as const) {
        for (const draw of draws) {
          draw.setCount(list.length);
          list.forEach((p, i) => draw.setMatrix(i, p.matrix));
          draw.update();
          draw.enabled = list.length > 0;
        }
      }
    }
  }

  /** A model's near and far draws in the colours of `o`. */
  private async makeLook(key: string, o: ObjectPlacement): Promise<Look> {
    const def = this.content.object(o.def);
    const entry = painted(this.assets.get(def?.model ?? '', 'model'), o.colors);
    const footprint = def?.footprint ?? [1, 1];
    const [near, far] = await Promise.all([buildModel(this.app, `ride:${key}`, entry, footprint), buildModel(this.app, `ride:${key}:far`, entry, footprint, { far: true })]);
    const draws = (t: ModelTemplate) =>
      t.meshes.map((m) => {
        const draw = new Instances(this.app.graphicsDevice, this.root, m.mesh, m.material, { name: m.name, castShadows: this.shadows, receiveShadows: true, capacity: 4 });
        draw.enabled = false;
        return draw;
      });
    return { near: draws(near), far: draws(far), templates: [near, far] };
  }

  private dropLook(look: Look): void {
    for (const draw of [...look.near, ...look.far]) draw.destroy();
    for (const t of look.templates) disposeTemplate(t);
  }
}
