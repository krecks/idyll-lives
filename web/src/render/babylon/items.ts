/**
 * Food and dishes on tables, counters and floors (sim-core `items.rs`; `ItemPlacement`): a pot
 * of stew on the counter, plates in front of the diners, dirty plates waiting for the sink.
 *
 * They change far more often than furniture (every serving taken, every plate put down), so
 * they are drawn apart from the world's objects: one thin-instanced template per item model
 * (content `items[].model`, `item.*` in the manifest), rebuilt when the simulation sends new
 * items. Food that has gone off gets a grey-green film on top. Items in a fridge aren't drawn.
 * Items are picked by a small box each (the pie menu: eat, clear away).
 */

import { Color3, Matrix, Mesh, MeshBuilder, Quaternion, StandardMaterial, Vector3, type Scene } from './core';
import type { AssetRegistry } from '../../assets/registry';
import type { Content } from '../../content/content';
import type { ItemPlacement } from '../../core/protocol';
import { buildModel, onModelFileArrived, type ModelTemplate } from './models';
import { buildAhead } from '../shared/idle';
import { ITEM_STAND_IN } from '../shared/modelEntry';
import { TemplateCache } from './templateCache';

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
  /** Per model in use: its instance matrices. */
  private readonly buffers = new Map<string, Float32Array>();
  private readonly mold: Mesh;
  private moldMatrices = new Float32Array(16);
  private items: readonly ItemPlacement[] = [];
  private view: ItemView | null = null;
  /** Bumped per rebuild, so a slow one never overwrites a newer one. */
  private generation = 0;
  /** Models drawn by the last rebuild. */
  private drawn = new Set<string>();
  placed: PlacedItem[] = [];
  /** Called after the drawn items change (the renderer redraws a recorded frame). */
  onChange: (() => void) | null = null;

  constructor(
    private readonly scene: Scene,
    private readonly assets: AssetRegistry,
    private readonly content: Content,
  ) {
    const mold = MeshBuilder.CreateSphere('itemMold', { diameter: 1, segments: 6 }, scene);
    const mat = new StandardMaterial('itemMoldMat', scene);
    mat.diffuseColor = new Color3(0.42, 0.5, 0.32);
    mat.specularColor = Color3.Black();
    mat.alpha = 0.85;
    mold.material = mat;
    mold.isPickable = false;
    mold.thinInstanceSetBuffer('matrix', this.moldMatrices, 16, false);
    mold.thinInstanceCount = 0;
    this.mold = mold;
    this.templates = new TemplateCache(scene);
    // A model file that failed has come: the items drawn without it (a block, or nothing) are built again.
    this.unlisten = onModelFileArrived(scene, (url) => {
      if (this.templates.invalidate(url).length) void this.rebuild();
    });
  }

  private readonly unlisten: () => void;

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
    return this.templates.get(key, () => buildModel(this.scene, `item:${key}`, this.assets.get(key, 'model'), [1, 1], { standIn: ITEM_STAND_IN }));
  }

  private async rebuild(): Promise<void> {
    const generation = ++this.generation;
    const view = this.view;
    if (!view) {
      // Nothing is drawn yet: the blocks of files that came can go.
      this.templates.disposeStale();
      return;
    }
    const shown = this.items.filter((it) => it.state !== 'stored' && view.shown(view.storeyOf(it.z)) && view.inView(it.x, view.groundZ(it.z)));
    const byModel = Map.groupBy(shown, (it) => this.modelOf(it.def));
    this.buildElsewhere(generation);
    const keys = [...byModel.keys()];
    const templates = await Promise.all(keys.map((k) => this.template(k)));
    if (generation !== this.generation) return;
    const placed: PlacedItem[] = [];
    const q = new Quaternion();
    const m = new Matrix();
    const one = Vector3.One();
    const at = new Vector3();
    const molds: number[] = [];
    keys.forEach((key, n) => {
      const list = byModel.get(key)!;
      const template = templates[n];
      let matrices = this.buffers.get(key);
      if (!matrices || matrices.length < list.length * 16) {
        matrices = new Float32Array(Math.max(4, list.length * 2) * 16);
        this.buffers.set(key, matrices);
      }
      list.forEach((it, i) => {
        const [x, y, z] = [it.x, view.storeyOf(it.z) * view.storeyHeight + it.y, view.groundZ(it.z)];
        Quaternion.RotationYawPitchRollToRef(it.yaw, 0, 0, q);
        Matrix.ComposeToRef(one, q, at.set(x, y, z), m);
        m.copyToArray(matrices, i * 16);
        const half = Math.max(MIN_PICK, this.content.item(it.def)?.radius ?? 0.12) / 2 + 0.05;
        placed.push({ id: it.id, minX: x - half, minZ: z - half, maxX: x + half, maxZ: z + half, y0: y, height: Math.max(0.12, template.height) });
        if (it.state === 'spoiled') molds.push(x, y + template.height * 0.85 + 0.01, z, half * 1.4);
      });
      for (const mesh of template.meshes) {
        mesh.setEnabled(true);
        mesh.isPickable = false;
        mesh.receiveShadows = true;
        mesh.thinInstanceSetBuffer('matrix', matrices, 16, false);
        mesh.thinInstanceCount = list.length;
        mesh.thinInstanceBufferUpdated('matrix');
        mesh.thinInstanceRefreshBoundingInfo(false);
      }
    });
    // Models no longer drawn.
    for (const [key, t] of this.templates) {
      if (byModel.has(key)) continue;
      void t.then((template) => {
        if (generation !== this.generation || byModel.has(key)) return;
        for (const mesh of template.meshes) {
          mesh.thinInstanceCount = 0;
          mesh.setEnabled(false);
        }
      });
    }
    // A film of mould over food gone off.
    const count = molds.length / 4;
    if (this.moldMatrices.length < count * 16) this.moldMatrices = new Float32Array(Math.max(4, count * 2) * 16);
    for (let i = 0; i < count; i++) {
      const [x, y, z, r] = molds.slice(i * 4, i * 4 + 4);
      Matrix.ComposeToRef(at.set(r, 0.025, r), Quaternion.Identity(), new Vector3(x, y, z), m);
      m.copyToArray(this.moldMatrices, i * 16);
    }
    this.mold.thinInstanceSetBuffer('matrix', this.moldMatrices, 16, false);
    this.mold.thinInstanceCount = count;
    this.mold.thinInstanceBufferUpdated('matrix');
    this.mold.thinInstanceRefreshBoundingInfo(false);
    this.mold.setEnabled(count > 0);
    this.drawn = new Set(keys);
    this.placed = placed;
    // Built since without the blocks of files that came (set aside just before this rebuild began, `onModelFileArrived`): they go.
    this.templates.disposeStale();
    this.onChange?.();
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
      void this.template(key).then((template) => {
        if (!this.drawn.has(key)) for (const mesh of template.meshes) mesh.setEnabled(false);
      });
    });
    if (jobs.length) buildAhead(jobs, () => generation === this.generation);
  }

  dispose(): void {
    this.unlisten();
    this.mold.dispose();
    for (const [, t] of this.templates) void t.then((template) => template.meshes.forEach((mesh) => mesh.dispose()));
    this.templates.clear();
    this.templates.disposeStale();
  }
}
