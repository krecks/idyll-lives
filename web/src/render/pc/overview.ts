/**
 * The town overview behind the menus (main menu, new-game steps) on PlayCanvas, the port of
 * `babylon/overview.ts`: the whole neighbourhood at once, drawn with the game's own builders so it
 * looks like the game. Every house is a dressed shell (`HouseBuilder.silhouetteMeshes` through
 * `Street`: porches, chimneys, garages, cars, fences, flower beds; the player's house,
 * `WorldMeta.home`, dressed as the game builds it), with its models' far versions, plus the road,
 * garden paths and fields and every outdoor object (garden and park trees, benches, flower beds).
 * No interiors, no residents.
 *
 * Also owns the overview's lot highlights: glowing outlines, a soft fill and a light wall round the
 * selected lot, an instance per plot whose colours are rewritten as they fade (`frame`).
 *
 * Everything here is drawn on the town layer (`townLayer`: the camera draws it in place of the world
 * layer while the menus show the town, and the sun lights it then, `PlayCanvasRenderer.enterTown`),
 * so it can be built and kept while the game world is on screen, and the other way round. The
 * landscape round it is `LotContents`' (`useNature('town', …)`), shared with the game's when it is
 * the same town; the camera rig is the renderer's (`shared/overviewCamera.ts`).
 */

import * as pc from './engine';
import type { HouseBuilder } from './house';
import { Instances, INSTANCE_COLOR } from './instancing';
import { GAME_FRONT_FACE, MaterialLibrary } from './materials';
import { drawMesh, meshFrom } from './meshes';
import { buildModel, disposeTemplate, glowByInstance, type ModelTemplate } from './models';
import { windStill } from './nature';
import { Street } from './street';
import { TemplateCache } from './templateCache';
import type { AssetRegistry } from '../../assets/registry';
import type { Content } from '../../content/content';
import { groundDepth, objectCentre, type ObjectPlacement, type PlotInfo, type WorldStructure } from '../../core/protocol';
import type { LotHighlight } from '../types';
import type { RGB } from '../shared/environment';
import { Geo, type V3 } from '../shared/geometry';
import { compose, quatYawPitchRoll, type Quat } from '../shared/math';
import { placementVariation, standInOf } from '../shared/modelEntry';
import { fieldLayers } from '../shared/street';

/** Height that labels and fits allow for (roofs, chimneys, crowns). */
const HIGHLIGHT_INSET = 0.55;
const HIGHLIGHT_CORNER = 1.6;
/** Per-plot highlight intensities ease towards their targets with this time constant (s). */
const FADE = 0.12;
/** How far (Babylon's `zOffset`: depth-buffer steps per slope) the highlights are drawn in front of what they lie on. */
const HIGHLIGHT_DEPTH_SLOPE = -6;

/**
 * Room above 1 for the highlights' colours (the selected lot's blue is 1.25, and the strength goes to
 * 1.2): PlayCanvas clamps a vertex colour where it's read (`emissivePS`, `opacityPS`), Babylon took
 * them as they were. The material's emissive is this much stronger and the instance colours this
 * much weaker (linear), and an alpha over 1 goes into the colour (the glow is added by its alpha).
 */
const GLOW_HEADROOM = 2;
const HEADROOM_SRGB = 1 / Math.pow(GLOW_HEADROOM, 1 / 2.2);

/** The highlights' colours (sRGB, as Babylon's standard material took them), outline, hover, selected and marked. */
const COLORS: Record<'outline' | 'hover' | 'selected' | 'marked', RGB> = {
  outline: { r: 1, g: 0.97, b: 0.9 },
  hover: { r: 1, g: 0.98, b: 0.92 },
  selected: { r: 0.42, g: 0.62, b: 1.25 },
  marked: { r: 1.0, g: 0.78, b: 0.38 },
};

const townLayers = new WeakMap<pc.AppBase, pc.Layer>();

/**
 * The overview's layer (Babylon's `LAYER_TOWN` camera mask), right after the world layer in both
 * its opaque and blended halves; made once per app. No light is on it but the sun while the town is
 * shown (the lamps stay on the world layer).
 */
export function townLayer(app: pc.AppBase): pc.Layer {
  let layer = townLayers.get(app);
  if (!layer) {
    layer = new pc.Layer({ name: 'Town' });
    const layers = app.scene.layers;
    const world = layers.getLayerById(pc.LAYERID_WORLD)!;
    layers.insertOpaque(layer, layers.getOpaqueIndex(world) + 1);
    layers.insertTransparent(layer, layers.getTransparentIndex(world) + 1);
    townLayers.set(app, layer);
  }
  return layer;
}

/** A model drawn at the outdoor objects of every lot: its template and a draw per mesh. */
interface Placed {
  template: ModelTemplate;
  draws: Instances[];
}

/** One of the three highlight draws (ring, fill, wall): an instance per plot. */
interface Highlight {
  draw: Instances;
}

export class TownOverview {
  /** Everything the overview draws hangs here (game space, under the world root). */
  readonly root: pc.Entity;
  private readonly layers: number[];
  private readonly lib: MaterialLibrary;
  private readonly device: pc.GraphicsDevice;
  /** Two streets used in turn: the next one builds (hidden) while the current one stays on screen. */
  private readonly streets: [Street, Street];
  private current = 0;
  private ground: pc.Entity[] = [];
  private readonly templates: TemplateCache<ModelTemplate>;
  private readonly placed = new Map<string, Placed>();
  private readonly templatesInUse = new Set<string>();
  private key = '';
  plots: PlotInfo[] = [];
  width = 0;
  depth = 0;

  // Lot highlights: ring (outline), fill (inner glow) and wall (light curtain), one instance per plot.
  private readonly glowMaterial: pc.Material;
  private hl: Highlight[] = [];
  private hlSize = '';
  private state: LotHighlight = { hover: null, selected: null, marked: [], outlines: false };
  /** Per plot: current and target intensity of [outline, hover, selected, marked]. */
  private level = new Float32Array(0);
  private target = new Float32Array(0);
  /** Colours must be written even though nothing is fading (after a rebuild). */
  private dirty = false;
  private time = 0;
  private readonly rgba = new Float32Array(4);
  private readonly m = new Float32Array(16);
  private readonly q: Quat = [0, 0, 0, 1];

  constructor(
    private readonly app: pc.AppBase,
    parent: pc.Entity,
    private readonly assets: AssetRegistry,
    private readonly content: Content,
    house: HouseBuilder,
  ) {
    this.lib = MaterialLibrary.for(app);
    this.device = app.graphicsDevice;
    this.layers = [townLayer(app).id];
    this.root = new pc.Entity('townOverview');
    parent.addChild(this.root);
    this.templates = new TemplateCache(app);
    this.streets = [new Street(app, this.root, assets, house, { layers: this.layers }), new Street(app, this.root, assets, house, { layers: this.layers })];
    // Unlit: the output is the vertex colour times the instance colour (`glowByInstance`), added by its alpha.
    const mat = new pc.StandardMaterial();
    mat.name = 'lotGlow';
    mat.frontFace = GAME_FRONT_FACE;
    mat.useLighting = false;
    mat.useSkybox = false;
    mat.useFog = false;
    mat.diffuse.set(0, 0, 0);
    mat.emissive.set(1, 1, 1);
    mat.emissiveIntensity = GLOW_HEADROOM;
    mat.emissiveVertexColor = true;
    // (sRGB, linearised: the Babylon renderer's standard material's output was taken as sRGB.)
    (mat as pc.StandardMaterial & { vertexColorGamma: boolean }).vertexColorGamma = true;
    mat.opacityVertexColor = true;
    mat.opacityVertexColorChannel = 'a';
    mat.blendType = pc.BLEND_ADDITIVEALPHA;
    mat.depthWrite = false;
    mat.cull = pc.CULLFACE_NONE;
    mat.slopeDepthBias = HIGHLIGHT_DEPTH_SLOPE;
    mat.update();
    this.glowMaterial = glowByInstance(app, mat);
  }

  /** Every mesh instance the overview draws (the house's lamps leave them out: `Lamps.exclude`). */
  meshInstances(): pc.MeshInstance[] {
    const out: pc.MeshInstance[] = [];
    for (const r of this.root.findComponents('render') as pc.RenderComponent[]) out.push(...r.meshInstances);
    return out;
  }

  /**
   * A model file that failed has come: what was built without it (a box, or nothing) is built afresh
   * when next asked for, and if the overview draws one, the next `build` builds the town again (even
   * the same one; true). Called between builds.
   */
  invalidate(url: string): boolean {
    const keys = this.templates.invalidate(url);
    // Those the overview doesn't draw (hidden) go now; it's built again only for those it does.
    this.disposeStale(keys.filter((k) => !this.templatesInUse.has(k)));
    let drawn = keys.some((k) => this.templatesInUse.has(k));
    for (const street of this.streets) drawn = street.invalidate(url) || drawn;
    if (drawn) this.key = '';
    return drawn;
  }

  /** Builds the overview of `world` (kept when it is the same town); the previous one stays until it is done. True when it was built. */
  async build(world: WorldStructure, pace?: () => Promise<void>): Promise<boolean> {
    const key = townKey(world);
    if (key === this.key) return false;
    const next = this.streets[1 - this.current];
    // (Hidden while it builds: the current one is on screen.)
    next.root.enabled = false;
    await next.build(world, null, pace);
    const ground = await this.buildGround(world, pace);
    const objects = await this.buildObjects(world, pace);
    await pace?.();

    // Swap: drop the previous overview, show the new one.
    this.streets[this.current].clear();
    this.current = 1 - this.current;
    next.root.enabled = true;
    for (const e of this.ground) e.destroy();
    this.ground = ground;
    for (const e of ground) e.enabled = true;
    for (const [k, p] of this.placed) if (!objects.has(k)) for (const d of p.draws) d.enabled = false;
    this.templatesInUse.clear();
    for (const [k, { template, matrices }] of objects) {
      this.templatesInUse.add(k);
      const p = this.placedOf(k, template);
      const count = matrices.length / 16;
      for (const d of p.draws) {
        d.setMatrices(matrices, count);
        d.update();
        d.enabled = count > 0;
      }
      // (In place of a box set aside for a file that came, if there was one: never both at once.)
      this.templates.hideStale(k);
    }
    // The previous overview's boxes for files that have come since are drawn no more.
    this.disposeStale();
    this.key = key;
    this.plots = world.plots;
    this.width = world.width;
    this.depth = groundDepth(world);
    this.buildHighlights(world.plots);
    return true;
  }

  /** Streetlights, porch lamps and window glass follow the lighting key's lamp level (`glow`: the lamps' colour, linear). */
  setNight(lamps: number, glow: RGB): void {
    this.streets[this.current].setNight(lamps, glow);
  }

  /** Which lots glow (see `Renderer.setLotHighlight`). */
  setHighlight(h: LotHighlight): void {
    this.state = h;
    this.updateTargets();
  }

  /** Per frame (`dt` in s): eases the highlights; true while their colours changed. Allocation-free. */
  frame(dt: number): boolean {
    this.time += dt;
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
    const c = this.rgba;
    for (let i = 0; i < n; i++) {
      const outline = this.level[i * 4];
      const hover = this.level[i * 4 + 1];
      const sel = this.level[i * 4 + 2] * beat;
      const marked = this.level[i * 4 + 3];
      // Ring: the strongest of the states, blended by colour.
      ring.draw.set(INSTANCE_COLOR, i, writeColor(c, outline * 0.32, hover * 1.15, sel * 1.1, marked * 0.6));
      fill.draw.set(INSTANCE_COLOR, i, writeColor(c, 0, hover * 0.4, sel * 0.34, marked * 0.05));
      wall.draw.set(INSTANCE_COLOR, i, writeColor(c, 0, hover * 0.2, sel * 0.32, 0));
    }
    for (const h of this.hl) h.draw.update(false);
    return true;
  }

  destroy(): void {
    for (const street of this.streets) street.destroy();
    for (const e of this.ground) e.destroy();
    this.ground = [];
    this.dropHighlights();
    for (const p of this.placed.values()) for (const d of p.draws) d.destroy();
    this.placed.clear();
    for (const [, t] of this.templates) void t.then((template) => disposeTemplate(template)).catch(() => {});
    this.templates.clear();
    this.root.destroy();
  }

  // ---- building ---------------------------------------------------------------------

  /** Road and garden paths / driveways and fields, one flat mesh each (hidden until the swap). */
  private async buildGround(world: WorldStructure, pace?: () => Promise<void>): Promise<pc.Entity[]> {
    const out: pc.Entity[] = [];
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
      const mesh = meshFrom(this.device, g.meshData());
      if (!mesh) continue;
      const entity = drawMesh(this.root, name, mesh, this.lib.surface(material, { doubleSided: true }), { receiveShadows: true, layers: this.layers });
      entity.enabled = false;
      out.push(entity);
    }
    return out;
  }

  /** Outdoor objects of every lot (garden and park trees, benches, flower beds), one template per model, and their matrices. */
  private async buildObjects(world: WorldStructure, pace?: () => Promise<void>): Promise<Map<string, { template: ModelTemplate; matrices: Float32Array }>> {
    const W = world.width;
    const outdoor = (o: ObjectPlacement) => {
      for (let z = o.z; z < o.z + o.d; z++) for (let x = o.x; x < o.x + o.w; x++) if (world.rooms[z * W + x]) return false;
      return true;
    };
    const groups = Map.groupBy(world.objects.filter(outdoor), (o) => this.content.object(o.def)?.model ?? `model.${o.def}`);
    const out = new Map<string, { template: ModelTemplate; matrices: Float32Array }>();
    for (const [key, list] of groups) {
      await pace?.();
      const template = await this.templates.get(key, () => {
        const def = this.content.object(list[0].def);
        // Seen from afar: a file's far version; without a file, the box the lot draws.
        return buildModel(this.app, `town:${key}`, this.assets.get(key, 'model'), def?.footprint ?? [1, 1], { far: true, standIn: standInOf(this.content, def) });
      });
      const matrices = new Float32Array(list.length * 16);
      const vary = this.assets.get(key, 'model')?.vary ?? 0;
      list.forEach((o, i) => {
        const { turn, size } = placementVariation(vary, o.x, o.z);
        const centre = objectCentre(o);
        compose(size, size, size, quatYawPitchRoll((o.rot * Math.PI) / 2 + ((o.turn ?? 0) * Math.PI) / 180 + turn, 0, 0, this.q), centre.x, 0, centre.z, this.m);
        matrices.set(this.m, i * 16);
      });
      out.set(key, { template, matrices });
      await pace?.();
    }
    return out;
  }

  /** The draws of a model's template (made the first time, again for a template built anew). */
  private placedOf(key: string, template: ModelTemplate): Placed {
    let p = this.placed.get(key);
    if (p && p.template !== template) {
      // Built again (a file came): the old draws go with the old template (`disposeStale`), hidden meanwhile.
      for (const d of p.draws) d.enabled = false;
      p = undefined;
    }
    if (!p) {
      p = { template, draws: [] };
      for (const m of template.meshes) {
        const d = new Instances(this.device, this.root, m.mesh, m.material, { name: m.name, castShadows: true, receiveShadows: true, layers: this.layers });
        // (Foliage sways in the wind, `nature.ts`: a part grown from nothing without fluttering.)
        windStill(d.meshInstance, m);
        p.draws.push(d);
      }
      this.placed.set(key, p);
    }
    return p;
  }

  /** Disposes the templates set aside (or those under `keys`), with the overview's draws of them. */
  private disposeStale(keys?: readonly string[]): void {
    for (const [k, p] of this.placed) {
      if (keys && !keys.includes(k)) continue;
      if (this.templates.built(k) === p.template) continue;
      for (const d of p.draws) d.destroy();
      this.placed.delete(k);
    }
    this.templates.disposeStale(keys);
  }

  /** Takes down the highlight draws and their meshes (a draw never destroys its mesh: they're the overview's). */
  private dropHighlights(): void {
    for (const { draw } of this.hl) {
      draw.destroy();
      draw.mesh.destroy();
    }
    this.hl = [];
  }

  /** Highlight meshes for the plot size (built again only when it changes) and one instance per plot. */
  private buildHighlights(plots: PlotInfo[]): void {
    const first = plots[0];
    if (!first) return;
    const size = `${first.w}x${first.d}`;
    if (size !== this.hlSize) {
      this.dropHighlights();
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
      this.hl = (
        [
          [ring, 'lotRing'],
          [fill, 'lotFill'],
          [wall, 'lotWall'],
        ] as const
      ).map(([geo, name]) => {
        const mesh = meshFrom(this.device, geo.meshData({ colors: true }))!;
        const draw = new Instances(this.device, this.root, mesh, this.glowMaterial, { name, streams: [INSTANCE_COLOR], castShadows: false, receiveShadows: false, layers: this.layers, cull: false });
        return { draw };
      });
    }
    const n = plots.length;
    const m = this.m;
    for (const h of this.hl) {
      h.draw.setCount(n);
      plots.forEach((p, i) => {
        m.fill(0);
        m[0] = p.w / first.w;
        m[5] = 1;
        m[10] = p.d / first.d;
        m[12] = p.x;
        m[14] = p.z;
        m[15] = 1;
        h.draw.setMatrix(i, m);
        h.draw.set(INSTANCE_COLOR, i, ZERO);
      });
      h.draw.update(false);
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

const ZERO = [0, 0, 0, 0];

/** Identity of a town layout: the overview builds again only when it changes. */
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

/** A highlight's colour (into `out`): the states mixed by intensity (an added glow, alpha its strength). */
function writeColor(out: Float32Array, outline: number, hover: number, selected: number, marked: number): Float32Array {
  const a = outline + hover + selected + marked;
  if (a < 1e-4) {
    out.fill(0);
    return out;
  }
  const c = COLORS;
  // (The strength over 1 in the colour, and the colour within the headroom: see `GLOW_HEADROOM`.)
  const alpha = Math.min(1.2, a);
  const k = (HEADROOM_SRGB * Math.pow(Math.max(1, alpha), 1 / 2.2)) / a;
  out[0] = (c.outline.r * outline + c.hover.r * hover + c.selected.r * selected + c.marked.r * marked) * k;
  out[1] = (c.outline.g * outline + c.hover.g * hover + c.selected.g * selected + c.marked.g * marked) * k;
  out[2] = (c.outline.b * outline + c.hover.b * hover + c.selected.b * selected + c.marked.b * marked) * k;
  out[3] = Math.min(1, alpha);
  return out;
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
