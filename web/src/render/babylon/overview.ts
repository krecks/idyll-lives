/**
 * The town overview behind the menus (main menu, new-game steps): the whole neighbourhood at
 * once, drawn with the game's own builders so it looks like the game. Every house is a dressed
 * shell (`HouseBuilder.silhouettes` through `Street`: porches, chimneys, garages, cars, fences,
 * flower beds; the player's house, `WorldMeta.home`, dressed as the game builds it), plus the
 * road, garden paths and every outdoor object (garden and park trees, benches, flower beds). No
 * interiors, no Sims.
 *
 * Also owns the overview's lot highlights (glowing outlines, a soft fill and a light wall around
 * the selected lot; thin instances whose colours are rewritten, never re-recorded) and its
 * camera rig (framed shots, glides, a slow drift).
 *
 * Everything here is on the `LAYER_TOWN` camera layer, so it can be built and kept while the
 * game world is on screen, and the other way round (see `BabylonRenderer.showTown`).
 */

import { ArcRotateCamera, Color3, Constants, Matrix, Quaternion, StandardMaterial, Vector3, type Mesh, type Scene } from './core';
import type { AssetRegistry } from '../../assets/registry';
import type { Content } from '../../content/content';
import { groundDepth, objectCentre, type ObjectPlacement, type PlotInfo, type WorldStructure } from '../../core/protocol';
import type { LotHighlight } from '../types';
import { Geo, type V3 } from '../shared/geometry';
import { toMesh } from './meshes';
import type { HouseBuilder } from './house';
import { LAYER_TOWN } from '../shared/layers';
import type { MaterialLibrary } from './materials';
import { buildModel, type ModelTemplate } from './models';
import { placementVariation, standInOf } from '../shared/modelEntry';
import { fieldLayers, Street } from './street';
import { TemplateCache } from './templateCache';
import { OverviewCamera } from '../shared/overviewCamera';

/** Height that labels and fits allow for (roofs, chimneys, crowns). */
const HIGHLIGHT_INSET = 0.55;
const HIGHLIGHT_CORNER = 1.6;
/** Per-plot highlight intensities ease towards their targets with this time constant (s). */
const FADE = 0.12;

const COLORS = {
  outline: new Color3(1, 0.97, 0.9),
  hover: new Color3(1, 0.98, 0.92),
  selected: new Color3(0.42, 0.62, 1.25),
  marked: new Color3(1.0, 0.78, 0.38),
};

export class TownOverview {
  /** Two streets used in turn: the next one builds while the current one stays on screen. */
  private readonly streets: [Street, Street];
  private current = 0;
  private ground: Mesh[] = [];
  private readonly templates: TemplateCache<ModelTemplate>;
  private readonly templatesInUse = new Set<string>();
  private key = '';
  /** Shadow casters of the overview (without the landscape). */
  casters: Mesh[] = [];
  /** Everything on the town layer (for the room-light exclusions). */
  meshes: Mesh[] = [];
  plots: PlotInfo[] = [];
  readonly centre = new Vector3();
  width = 0;
  depth = 0;

  // Lot highlights: ring (outline), fill (inner glow) and wall (light curtain), one instance per plot.
  private readonly glowMaterial: StandardMaterial;
  private hl: { mesh: Mesh; colors: Float32Array; matrices: Float32Array }[] = [];
  private hlSize = '';
  private state: LotHighlight = { hover: null, selected: null, marked: [], outlines: false };
  /** Per plot: current and target intensity of [outline, hover, selected, marked]. */
  private level = new Float32Array(0);
  private target = new Float32Array(0);
  /** Colours must be written even though nothing is fading (after a rebuild). */
  private dirty = false;
  private time = 0;

  readonly rig: OverviewCamera;

  constructor(
    private readonly scene: Scene,
    private readonly assets: AssetRegistry,
    private readonly content: Content,
    house: HouseBuilder,
    private readonly lib: MaterialLibrary,
    camera: ArcRotateCamera,
  ) {
    this.templates = new TemplateCache(scene);
    this.streets = [new Street(scene, assets, house, { layerMask: LAYER_TOWN }), new Street(scene, assets, house, { layerMask: LAYER_TOWN })];
    const mat = (this.glowMaterial = new StandardMaterial('lotGlow', scene));
    // Unlit: the output is emissive x vertex colour x instance colour.
    mat.disableLighting = true;
    mat.diffuseColor = Color3.Black();
    mat.emissiveColor = Color3.White();
    mat.specularColor = Color3.Black();
    mat.alphaMode = Constants.ALPHA_ADD;
    mat.disableDepthWrite = true;
    mat.backFaceCulling = false;
    mat.fogEnabled = false;
    mat.zOffset = -6;
    this.rig = new OverviewCamera(camera);
  }

  /**
   * A model file that failed has come: what was built without it (a box, or nothing) is built afresh
   * when next asked for, and if the overview draws one, the next `build` builds the town again (even
   * the same one; true). Called between builds.
   */
  invalidate(url: string): boolean {
    const keys = this.templates.invalidate(url);
    // Those the overview doesn't draw (hidden) go now; it's built again only for those it does.
    this.templates.disposeStale(keys.filter((k) => !this.templatesInUse.has(k)));
    let drawn = keys.some((k) => this.templatesInUse.has(k));
    for (const street of this.streets) drawn = street.invalidate(url) || drawn;
    if (drawn) this.key = '';
    return drawn;
  }

  /** Builds the overview of `world` (kept when it is the same town); the previous one stays until it is done. */
  async build(world: WorldStructure, pace?: () => Promise<void>): Promise<boolean> {
    const key = townKey(world);
    if (key === this.key) return false;
    const next = this.streets[1 - this.current];
    const streetCasters = await next.build(world, null, pace);
    const ground = await this.buildGround(world, pace);
    const objects = await this.buildObjects(world, pace);
    await pace?.();

    // Swap: drop the previous overview, show the new one.
    this.streets[this.current].clear();
    this.current = 1 - this.current;
    for (const mesh of this.ground) mesh.dispose(false, false);
    this.ground = ground;
    for (const [key, t] of this.templates) {
      if (objects.has(key)) continue;
      void t.then((template) => {
        if (!this.templatesInUse.has(key)) for (const mesh of template.meshes) mesh.setEnabled(false);
      });
    }
    // The previous overview's boxes for files that have come since are drawn no more.
    this.templates.disposeStale();
    this.templatesInUse.clear();
    const objectMeshes: Mesh[] = [];
    for (const [key, { template, matrices }] of objects) {
      this.templatesInUse.add(key);
      for (const mesh of template.meshes) {
        mesh.thinInstanceSetBuffer('matrix', matrices, 16, true);
        mesh.thinInstanceRefreshBoundingInfo(false);
        mesh.setEnabled(true);
        objectMeshes.push(mesh);
      }
    }
    this.key = key;
    this.plots = world.plots;
    this.width = world.width;
    this.depth = groundDepth(world);
    this.centre.set(world.width / 2, 0, groundDepth(world) / 2);
    this.casters = [...streetCasters, ...objectMeshes];
    this.buildHighlights(world.plots);
    this.meshes = [...next.all, ...ground, ...objectMeshes, ...this.hl.map((h) => h.mesh)];
    for (const mesh of this.meshes) mesh.layerMask = LAYER_TOWN;
    return true;
  }

  setNight(lamps: number, glow: Color3): void {
    this.streets[this.current].setNight(lamps, glow);
  }

  /** Which lots glow (see `Renderer.setLotHighlight`). */
  setHighlight(h: LotHighlight): void {
    this.state = h;
    this.updateTargets();
  }

  /**
   * Per frame: eases the highlights (true while the colour buffers changed) and the camera.
   * Allocation-free.
   */
  frame(dt: number, aspect: number): boolean {
    this.time += dt;
    this.rig.frame(dt, aspect);
    const n = this.plots.length;
    if (!this.hl.length || !n) return false;
    let moving = false;
    const k = 1 - Math.exp(-dt / FADE);
    for (let i = 0; i < n * 4; i++) {
      const d = this.target[i] - this.level[i];
      if (Math.abs(d) < 0.002) this.level[i] = this.target[i];
      else {
        this.level[i] += d * k;
        moving = true;
      }
    }
    const pulse = this.state.selected !== null;
    if (!moving && !pulse && !this.dirty) return false;
    this.dirty = false;
    const beat = 0.82 + 0.18 * Math.sin(this.time * 3.2);
    const [ring, fill, wall] = this.hl;
    for (let i = 0; i < n; i++) {
      const outline = this.level[i * 4];
      const hover = this.level[i * 4 + 1];
      const sel = this.level[i * 4 + 2] * beat;
      const marked = this.level[i * 4 + 3];
      // Ring: the strongest of the states, blended by colour.
      writeColor(ring.colors, i, outline * 0.32, hover * 1.15, sel * 1.1, marked * 0.6);
      writeColor(fill.colors, i, 0, hover * 0.4, sel * 0.34, marked * 0.05);
      writeColor(wall.colors, i, 0, hover * 0.2, sel * 0.32, 0);
    }
    for (const h of this.hl) h.mesh.thinInstanceBufferUpdated('color');
    return true;
  }

  dispose(): void {
    for (const street of this.streets) street.clear();
    for (const mesh of this.ground) mesh.dispose(false, false);
    for (const h of this.hl) h.mesh.dispose(false, false);
  }

  // ---- building ---------------------------------------------------------------------

  /** Road and garden paths / driveways, one flat mesh each. */
  private async buildGround(world: WorldStructure, pace?: () => Promise<void>): Promise<Mesh[]> {
    const out: Mesh[] = [];
    const layers: [string, { x: number; z: number; w: number; d: number }[] | undefined, string, number][] = [
      ['townStreets', world.meta?.streets, 'material.road', 0.012],
      ['townPaths', world.meta?.paths, 'material.path', 0.014],
      ...fieldLayers(world.meta?.world?.fields, 'town'),
    ];
    for (const [name, rects, material, y] of layers) {
      if (!rects?.length) continue;
      await pace?.();
      const g = new Geo();
      for (const r of rects) g.poly([[r.x, y, r.z], [r.x + r.w, y, r.z], [r.x + r.w, y, r.z + r.d], [r.x, y, r.z + r.d]], [0, 1, 0]);
      const mesh = toMesh(g, name, this.scene);
      if (!mesh) continue;
      mesh.material = this.lib.surface(material, { doubleSided: true });
      mesh.receiveShadows = true;
      mesh.isPickable = false;
      mesh.freezeWorldMatrix();
      out.push(mesh);
    }
    return out;
  }

  /** Outdoor objects of every lot (garden and park trees, benches, flower beds), one template per model. */
  private async buildObjects(world: WorldStructure, pace?: () => Promise<void>): Promise<Map<string, { template: ModelTemplate; matrices: Float32Array }>> {
    const W = world.width;
    const outdoor = (o: ObjectPlacement) => {
      for (let z = o.z; z < o.z + o.d; z++) for (let x = o.x; x < o.x + o.w; x++) if (world.rooms[z * W + x]) return false;
      return true;
    };
    const groups = Map.groupBy(world.objects.filter(outdoor), (o) => this.content.object(o.def)?.model ?? `model.${o.def}`);
    const out = new Map<string, { template: ModelTemplate; matrices: Float32Array }>();
    const m = new Matrix();
    const q = new Quaternion();
    const scale = new Vector3();
    const at = new Vector3();
    for (const [key, list] of groups) {
      await pace?.();
      const template = await this.templates.get(key, () => {
        const def = this.content.object(list[0].def);
        // Seen from afar: a file's far version; without a file, the box the lot draws.
        return buildModel(this.scene, `town:${key}`, this.assets.get(key, 'model'), def?.footprint ?? [1, 1], { far: true, standIn: standInOf(this.content, def) }).then((template) => {
          for (const mesh of template.meshes) {
            mesh.layerMask = LAYER_TOWN;
            mesh.isPickable = false;
            mesh.receiveShadows = true;
          }
          return template;
        });
      });
      const matrices = new Float32Array(list.length * 16);
      const vary = this.assets.get(key, 'model')?.vary ?? 0;
      list.forEach((o, i) => {
        const { turn, size } = placementVariation(vary, o.x, o.z);
        const centre = objectCentre(o);
        Quaternion.RotationYawPitchRollToRef((o.rot * Math.PI) / 2 + ((o.turn ?? 0) * Math.PI) / 180 + turn, 0, 0, q);
        Matrix.ComposeToRef(scale.setAll(size), q, at.set(centre.x, 0, centre.z), m);
        m.copyToArray(matrices, i * 16);
      });
      out.set(key, { template, matrices });
      await pace?.();
    }
    return out;
  }

  /** Highlight meshes for the plot size (rebuilt only when it changes) and one instance per plot. */
  private buildHighlights(plots: PlotInfo[]): void {
    const first = plots[0];
    if (!first) return;
    const size = `${first.w}x${first.d}`;
    if (size !== this.hlSize) {
      for (const h of this.hl) h.mesh.dispose(false, false);
      this.hlSize = size;
      const path = roundedRect(first.w, first.d, HIGHLIGHT_INSET, HIGHLIGHT_CORNER, 7);
      const ring = new Geo().color('#FFFFFF');
      const fill = new Geo().color('#FFFFFF');
      const wall = new Geo().color('#FFFFFF');
      const up: V3 = [0, 1, 0];
      for (let i = 0; i < path.length; i++) {
        const [ax, az, anx, anz] = path[i];
        const [bx, bz, bnx, bnz] = path[(i + 1) % path.length];
        const at = (x: number, z: number, nx: number, nz: number, d: number, y: number): V3 => [x + nx * d, y, z + nz * d];
        // Ring: feathered band either side of the outline.
        for (const [d0, d1, a0, a1] of [
          [-0.42, -0.07, 0, 1],
          [-0.07, 0.07, 1, 1],
          [0.07, 0.42, 1, 0],
        ] as const) {
          // (Lifted over the lawn tufts so they don't hide it.)
          ring.poly([at(ax, az, anx, anz, d0, 0.22), at(bx, bz, bnx, bnz, d0, 0.22), at(bx, bz, bnx, bnz, d1, 0.22), at(ax, az, anx, anz, d1, 0.22)], up, undefined, [a0, a0, a1, a1]);
        }
        // Fill: a glow fading inwards from the outline.
        fill.poly([at(ax, az, anx, anz, -3.2, 0.2), at(bx, bz, bnx, bnz, -3.2, 0.2), at(bx, bz, bnx, bnz, 0, 0.2), at(ax, az, anx, anz, 0, 0.2)], up, undefined, [0, 0, 1, 1]);
        // Wall: a curtain of light standing on the outline, fading upwards.
        wall.poly([at(ax, az, anx, anz, 0, 0.05), at(bx, bz, bnx, bnz, 0, 0.05), at(bx, bz, bnx, bnz, 0, 2.2), at(ax, az, anx, anz, 0, 2.2)], [anx, 0, anz], undefined, [1, 1, 0, 0]);
      }
      this.hl = [
        [ring, 'lotRing'],
        [fill, 'lotFill'],
        [wall, 'lotWall'],
      ].map(([geo, name]) => {
        const mesh = toMesh((geo as Geo), name as string, this.scene, { colors: true })!;
        mesh.hasVertexAlpha = true;
        mesh.material = this.glowMaterial;
        mesh.isPickable = false;
        mesh.alwaysSelectAsActiveMesh = true;
        mesh.layerMask = LAYER_TOWN;
        return { mesh, colors: new Float32Array(0), matrices: new Float32Array(0) };
      });
    }
    const n = plots.length;
    for (const h of this.hl) {
      h.matrices = new Float32Array(n * 16);
      h.colors = new Float32Array(n * 4);
      plots.forEach((p, i) => {
        const o = i * 16;
        h.matrices[o] = p.w / first.w;
        h.matrices[o + 5] = 1;
        h.matrices[o + 10] = p.d / first.d;
        h.matrices[o + 12] = p.x;
        h.matrices[o + 14] = p.z;
        h.matrices[o + 15] = 1;
      });
      h.mesh.thinInstanceSetBuffer('matrix', h.matrices, 16, true);
      h.mesh.thinInstanceSetBuffer('color', h.colors, 4, false);
    }
    this.level = new Float32Array(n * 4);
    this.target = new Float32Array(n * 4);
    this.updateTargets();
    // Start at the targets (no fade-in after a rebuild), then write the colours once.
    this.level.set(this.target);
    this.dirty = true;
  }

  private updateTargets(): void {
    const n = this.plots.length;
    if (this.target.length !== n * 4) return;
    const { hover, selected, marked, outlines } = this.state;
    for (let i = 0; i < n; i++) {
      const id = this.plots[i].id;
      this.target[i * 4] = outlines ? 1 : 0;
      this.target[i * 4 + 1] = hover === id && selected !== id ? 1 : 0;
      this.target[i * 4 + 2] = selected === id ? 1 : 0;
      this.target[i * 4 + 3] = marked.includes(id) && selected !== id && hover !== id ? 1 : 0;
    }
  }
}

/** Identity of a town layout: the overview rebuilds only when it changes. */
function townKey(world: WorldStructure): string {
  return [
    world.meta?.seed ?? 0,
    world.width,
    world.depth,
    world.plots.map((p) => `${p.house?.join(',') ?? '-'}`).join(';'),
    world.openings.length,
    world.objects.length,
    world.objects.reduce((a, o) => a + o.x * 31 + o.z * 7 + o.rot, 0),
    (world.meta?.paths ?? []).map((r) => `${r.x},${r.z},${r.w}`).join(';'),
    JSON.stringify(world.meta?.world ?? null),
    world.meta?.home ?? '-',
  ].join('|');
}

/** Ring colour of one plot: the states mixed by intensity (additive glow, alpha = strength). */
function writeColor(buf: Float32Array, i: number, outline: number, hover: number, selected: number, marked: number): void {
  const a = outline + hover + selected + marked;
  const o = i * 4;
  if (a < 1e-4) {
    buf[o] = buf[o + 1] = buf[o + 2] = buf[o + 3] = 0;
    return;
  }
  const c = COLORS;
  buf[o] = (c.outline.r * outline + c.hover.r * hover + c.selected.r * selected + c.marked.r * marked) / a;
  buf[o + 1] = (c.outline.g * outline + c.hover.g * hover + c.selected.g * selected + c.marked.g * marked) / a;
  buf[o + 2] = (c.outline.b * outline + c.hover.b * hover + c.selected.b * selected + c.marked.b * marked) / a;
  buf[o + 3] = Math.min(1.2, a);
}

/** Closed rounded rectangle inset into a w x d plot: points [x, z, outward nx, nz]. */
function roundedRect(w: number, d: number, inset: number, r: number, seg: number): [number, number, number, number][] {
  const pts: [number, number, number, number][] = [];
  const [x0, z0, x1, z1] = [inset + r, inset + r, w - inset - r, d - inset - r];
  const corners: [number, number, number][] = [
    [x1, z1, 0],
    [x0, z1, Math.PI / 2],
    [x0, z0, Math.PI],
    [x1, z0, Math.PI * 1.5],
  ];
  for (const [cx, cz, start] of corners) {
    for (let i = 0; i <= seg; i++) {
      const t = start + (i / seg) * (Math.PI / 2);
      const nx = Math.cos(t);
      const nz = Math.sin(t);
      pts.push([cx + nx * r, cz + nz * r, nx, nz]);
    }
  }
  return pts;
}

// ---- camera -------------------------------------------------------------------------------

