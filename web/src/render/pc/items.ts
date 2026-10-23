/**
 * Food and dishes on tables, counters and floors on PlayCanvas (the port of `babylon/items.ts`;
 * sim-core `items.rs`, `ItemPlacement`): a pot of stew on the counter, plates in front of the
 * diners, dirty plates waiting for the sink.
 *
 * They change far more often than furniture (every serving taken, every plate put down), so they
 * are drawn apart from the world's objects: one instanced draw per mesh of each item model's
 * template (content `items[].model`, `item.*` in the manifest), written again when the simulation
 * sends new items. Food that has gone off gets a grey-green film on top. Items in a fridge aren't
 * drawn. Items are picked by a small box each (the pie menu: eat, clear away).
 */

import * as pc from './engine';
import { Instances } from './instancing';
import { GAME_FRONT_FACE } from './materials';
import { buildModel, onModelFileArrived, uploadData, type ModelTemplate } from './models';
import { TemplateCache } from './templateCache';
import type { AssetRegistry } from '../../assets/registry';
import type { Content } from '../../content/content';
import type { ItemPlacement } from '../../core/protocol';
import { buildAhead } from '../shared/idle';
import { compose, quatYawPitchRoll, type Quat } from '../shared/math';
import { ITEM_STAND_IN } from '../shared/modelEntry';
import { sphereData } from '../shared/modelParts';

/** Where items show: the storey in view, the storey of a lot row and how high each stands. */
export interface ItemView {
  storey: number;
  /** The storey a lot row is on (-1: the basement). */
  storeyOf(z: number): number;
  /** Whether storey `k` shows with `storey` in view. */
  shown(k: number): boolean;
  /** The same row on the ground's rows. */
  groundZ(z: number): number;
  storeyHeight: number;
  /** Whether a ground point (tiles) is in the lot being viewed. */
  inView(x: number, z: number): boolean;
}

/** A drawn item's pick box (metres; ground rows). */
export interface PlacedItem {
  id: number;
  minX: number;
  minZ: number;
  maxX: number;
  maxZ: number;
  y0: number;
  height: number;
}

/** Pick boxes are at least this wide (m): plates are small at game distance. */
const MIN_PICK = 0.3;

export class ItemLayer {
  private readonly templates: TemplateCache<ModelTemplate>;
  /** Per model in use: its instanced draws (one per template mesh) and their matrices. */
  private readonly draws = new Map<string, { template: ModelTemplate; draws: Instances[] }>();
  private readonly mold: Instances;
  private items: readonly ItemPlacement[] = [];
  private view: ItemView | null = null;
  /** Bumped per rebuild, so a slow one never overwrites a newer one. */
  private generation = 0;
  placed: PlacedItem[] = [];
  /** Called after the drawn items change. */
  onChange: (() => void) | null = null;
  private readonly unlisten: () => void;
  private readonly m = new Float32Array(16);
  private readonly q: Quat = [0, 0, 0, 1];

  constructor(
    private readonly app: pc.AppBase,
    private readonly parent: pc.Entity,
    private readonly assets: AssetRegistry,
    private readonly content: Content,
  ) {
    const mat = new pc.StandardMaterial();
    mat.name = 'itemMoldMat';
    // (Babylon's standard material, in its own gamma space: as PlayCanvas takes colours.)
    mat.diffuse.set(0.42, 0.5, 0.32);
    mat.specular.set(0, 0, 0);
    mat.opacity = 0.85;
    mat.blendType = pc.BLEND_NORMAL;
    mat.frontFace = GAME_FRONT_FACE;
    mat.update();
    this.mold = new Instances(app.graphicsDevice, parent, uploadData(app.graphicsDevice, sphereData(1, 1, 1, 6)), mat, { name: 'itemMold', castShadows: false });
    this.mold.enabled = false;
    this.templates = new TemplateCache(app);
    // A model file that failed has come: the items drawn without it (a block, or nothing) are built again.
    this.unlisten = onModelFileArrived(app, (url) => {
      if (this.templates.invalidate(url).length) void this.rebuild();
    });
  }

  /** New items from the simulation. */
  set(items: readonly ItemPlacement[]): void {
    this.items = items;
    void this.rebuild();
  }

  /** The lot or storey in view changed. */
  show(view: ItemView): void {
    this.view = view;
    void this.rebuild();
  }

  /** Nothing to draw (the session ended). */
  clear(): void {
    this.items = [];
    void this.rebuild();
    // Those built without their files are built (and their files tried) afresh in the next session.
    this.templates.dropFailed();
  }

  private modelOf(def: string): string {
    return this.content.item(def)?.model ?? `item.${def}`;
  }

  private template(key: string): Promise<ModelTemplate> {
    // Without its file, a small block where it stands.
    return this.templates.get(key, () => buildModel(this.app, `item:${key}`, this.assets.get(key, 'model'), [1, 1], { standIn: ITEM_STAND_IN }));
  }

  /** The draws of model `key` for `template` (made the first time, and again for a template built anew). */
  private drawsOf(key: string, template: ModelTemplate): Instances[] {
    let d = this.draws.get(key);
    if (d?.template !== template) {
      for (const draw of d?.draws ?? []) draw.destroy();
      d = { template, draws: template.meshes.map((m) => new Instances(this.app.graphicsDevice, this.parent, m.mesh, m.material, { name: m.name, castShadows: true, receiveShadows: true })) };
      this.draws.set(key, d);
    }
    return d.draws;
  }

  private async rebuild(): Promise<void> {
    const generation = ++this.generation;
    const view = this.view;
    if (!view) {
      // Nothing is drawn yet: the blocks of files that came can go.
      this.disposeStale();
      return;
    }
    const shown = this.items.filter((it) => it.state !== 'stored' && view.shown(view.storeyOf(it.z)) && view.inView(it.x, view.groundZ(it.z)));
    const byModel = Map.groupBy(shown, (it) => this.modelOf(it.def));
    this.buildElsewhere(generation);
    const keys = [...byModel.keys()];
    const templates = await Promise.all(keys.map((k) => this.template(k)));
    if (generation !== this.generation) return;
    const placed: PlacedItem[] = [];
    const molds: number[] = [];
    const m = this.m;
    keys.forEach((key, n) => {
      const list = byModel.get(key)!;
      const template = templates[n];
      const draws = this.drawsOf(key, template);
      for (const draw of draws) draw.setCount(list.length);
      list.forEach((it, i) => {
        const [x, y, z] = [it.x, view.storeyOf(it.z) * view.storeyHeight + it.y, view.groundZ(it.z)];
        compose(1, 1, 1, quatYawPitchRoll(it.yaw, 0, 0, this.q), x, y, z, m);
        for (const draw of draws) draw.setMatrix(i, m);
        const half = Math.max(MIN_PICK, this.content.item(it.def)?.radius ?? 0.12) / 2 + 0.05;
        placed.push({ id: it.id, minX: x - half, minZ: z - half, maxX: x + half, maxZ: z + half, y0: y, height: Math.max(0.12, template.height) });
        if (it.state === 'spoiled') molds.push(x, y + template.height * 0.85 + 0.01, z, half * 1.4);
      });
      for (const draw of draws) {
        draw.update();
        draw.enabled = true;
      }
    });
    // Models no longer drawn.
    for (const [key, d] of this.draws) {
      if (byModel.has(key)) continue;
      for (const draw of d.draws) {
        draw.setCount(0);
        draw.update();
        draw.enabled = false;
      }
    }
    // A film of mould over food gone off.
    const count = molds.length / 4;
    this.mold.setCount(count);
    for (let i = 0; i < count; i++) {
      const [x, y, z, r] = [molds[i * 4], molds[i * 4 + 1], molds[i * 4 + 2], molds[i * 4 + 3]];
      this.mold.setMatrix(i, compose(r, 0.025, r, [0, 0, 0, 1], x, y, z, m));
    }
    this.mold.update();
    this.mold.enabled = count > 0;
    this.placed = placed;
    // Built since without the blocks of files that came (set aside just before this rebuild began, `onModelFileArrived`): they go.
    this.disposeStale();
    this.onChange?.();
  }

  /** The templates set aside go, and the draws of them (a rebuild drew their replacements). */
  private disposeStale(): void {
    for (const [key, d] of this.draws) {
      if (this.templates.built(key) === d.template) continue;
      for (const draw of d.draws) draw.destroy();
      this.draws.delete(key);
    }
    this.templates.disposeStale();
  }

  /**
   * Generated models of the items on other lots, built ahead in idle moments and kept hidden
   * (the camera may follow someone there). Models from files build when shown.
   */
  private buildElsewhere(generation: number): void {
    const missing = new Set<string>();
    for (const it of this.items) {
      const key = this.modelOf(it.def);
      if (!this.templates.has(key) && !(this.assets.has(key, 'model') && this.assets.get(key, 'model')?.url)) missing.add(key);
    }
    const jobs = [...missing].map((key) => () => {
      if (this.templates.has(key)) return;
      // (Only built: a template has no draws until it's shown.)
      void this.template(key);
    });
    if (jobs.length) buildAhead(jobs, () => generation === this.generation);
  }

  dispose(): void {
    this.unlisten();
    this.mold.destroy();
    this.mold.mesh.destroy();
    for (const d of this.draws.values()) for (const draw of d.draws) draw.destroy();
    this.draws.clear();
    this.templates.clear();
    this.templates.disposeStale();
  }
}
