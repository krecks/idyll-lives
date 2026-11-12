/**
 * A big world's ground tiles (`nature.ts` `terrainTiles`): each has three levels of detail and draws
 * one of them, chosen by distance (`frameTiles`). Apart, so it can be checked on the engine's null
 * device without the rest of the landscape. And the town view's colour map over them (`TownGroundMap`).
 */

import * as pc from './engine';
import { groundDepth, type RoadPlan, type WorldStructure } from '../../core/protocol';
import { carriageway, hasSidewalks } from '../shared/junctions';

/** A ground tile as `terrainTiles` builds it: its draw, its meshes by level, where its middle is and the level drawn. */
export interface GroundTile {
  mi: pc.MeshInstance;
  meshes: pc.Mesh[];
  x: number;
  z: number;
  lod: number;
}

/**
 * Destroys the ground tiles' levels no draw holds (all but the one each draws now, which goes with its
 * entity): left to themselves, about 40 MB of a big world's vertex buffers with every new game or load.
 */
export function disposeTiles(tiles: readonly GroundTile[]): void {
  for (const t of tiles) for (const m of t.meshes) if (m !== t.mi.mesh) m.destroy();
}

// ---- the town view's colour map (docs/design/living-town.md: a ground colour map at altitude) ----

/** Texels a metre at most, and the map's side at most (2048: half a metre a texel over a Big world). */
const MAP_PER_M = 2;
const MAP_MAX = 2048;
/** The map's colours (sRGB): the asphalt and pavements as the street's textures and tones come out, the centre's paving, water, soil, paths. */
const MAP = {
  asphalt: '#55544F',
  pavement: '#C4C0B6',
  paving: '#B3A58E',
  water: '#3E6E80',
  soil: '#6B4E37',
  furrow: '#5A412E',
  path: '#A99C86',
} as const;

/**
 * The ground's colour map at altitude: what the town lays on its land (streets and pavements, the junctions'
 * pads, the centre's paving, its water, its farm fields and paths) over the whole world in one texture, baked
 * once per town (`bake`; again only for another town), and laid over the ground tiles' colour by the town
 * view's blend (`shaders/townGround.ts`). The tiles' material takes its define only while the blend is over 0
 * (`apply`): up to `TOWN_FROM` the ground draws with the shader it always had.
 */
export class TownGroundMap {
  private texture: pc.Texture | null = null;
  private key = '';
  private area = new Float32Array(4);
  private on = false;

  constructor(private readonly device: pc.GraphicsDevice) {}

  /** Bakes the map for `world` (nothing again for the same town; nothing without a 2D canvas: the tests' engine). */
  bake(world: WorldStructure): void {
    const meta = world.meta;
    const W = world.width;
    const D = groundDepth(world);
    const key = `${meta?.seed ?? 0}|${W}x${D}|${meta?.roads?.length ?? 0}|${world.plots.length}|${meta?.centre?.length ?? 0}|${meta?.backdrop?.length ?? 0}`;
    if (key === this.key || typeof OffscreenCanvas === 'undefined' || !meta) return;
    this.key = key;
    const size = Math.min(MAP_MAX, 2 ** Math.ceil(Math.log2(Math.max(W, D) * MAP_PER_M)));
    const sx = size / W;
    const sz = size / D;
    const canvas = new OffscreenCanvas(size, size);
    const g = canvas.getContext('2d', { willReadFrequently: true })!;
    g.setTransform(sx, 0, 0, sz, 0, 0);
    g.lineCap = 'round';
    g.lineJoin = 'round';
    const poly = (p: readonly (readonly [number, number])[]) => {
      g.beginPath();
      p.forEach(([x, z], i) => (i ? g.lineTo(x, z) : g.moveTo(x, z)));
      g.closePath();
    };
    // Farm fields on farm lots: soil in furrows.
    for (const f of meta.world?.fields ?? []) {
      g.fillStyle = MAP.soil;
      g.fillRect(f.x, f.z, f.w, f.d);
      g.fillStyle = MAP.furrow;
      const along = f.w >= f.d;
      for (let k = 0.5; k < (along ? f.d : f.w); k += 1.2) (along ? g.fillRect(f.x, f.z + k, f.w, 0.45) : g.fillRect(f.x + k, f.z, 0.45, f.d));
    }
    // Water (a big world's water zones: its river and ponds).
    g.fillStyle = MAP.water;
    for (const z of meta.zones ?? []) {
      if (z.kind !== 'water') continue;
      poly(z.poly);
      g.fill();
    }
    // The centre's paving, round the lots (their gardens stay theirs) and the greens.
    if (meta.centre?.length) {
      g.fillStyle = MAP.paving;
      for (const c of meta.centre) {
        poly(c.poly);
        g.fill();
      }
      g.globalCompositeOperation = 'destination-out';
      g.fillStyle = '#000';
      for (const p of world.plots) g.fillRect(p.x, p.z, p.w, p.d);
      for (const green of meta.greens ?? []) {
        poly(green);
        g.fill();
      }
      g.globalCompositeOperation = 'source-over';
    }
    // Paths, then the roads: their pavements (a road's width has them), then their carriageways, trails as paths.
    g.fillStyle = MAP.path;
    for (const p of meta.paths ?? []) g.fillRect(p.x, p.z, p.w, p.d);
    const roads = meta.roads ?? [];
    const stroke = (r: RoadPlan, width: number, colour: string) => {
      g.strokeStyle = colour;
      g.lineWidth = width;
      g.beginPath();
      r.points.forEach(([x, z], i) => (i ? g.lineTo(x, z) : g.moveTo(x, z)));
      g.stroke();
    };
    for (const r of roads) if (hasSidewalks(r)) stroke(r, r.width, MAP.pavement);
    for (const r of roads) if (r.kind === 'trail') stroke(r, r.width, MAP.path);
    for (const r of roads) if (r.kind !== 'trail') stroke(r, carriageway(r), MAP.asphalt);
    // The junctions' pads: where the roads' ends meet, asphalt as wide as the widest carriageway there.
    g.fillStyle = MAP.asphalt;
    for (const j of world.junctions ?? []) {
      let half = 0;
      for (const r of roads) for (const [x, z] of [r.points[0], r.points[r.points.length - 1]]) if (Math.hypot(x - j.x, z - j.z) < 2) half = Math.max(half, carriageway(r) / 2);
      if (!half) continue;
      g.beginPath();
      g.arc(j.x, j.z, half + 1, 0, Math.PI * 2);
      g.fill();
    }
    // Premultiplied (so the mips stay true at an edge), up to the GPU with its mips.
    const data = g.getImageData(0, 0, size, size).data;
    for (let i = 0; i < data.length; i += 4) {
      const a = data[i + 3] / 255;
      data[i] = Math.round(data[i] * a);
      data[i + 1] = Math.round(data[i + 1] * a);
      data[i + 2] = Math.round(data[i + 2] * a);
    }
    this.texture?.destroy();
    const texture = new pc.Texture(this.device, { name: 'townGroundMap', width: size, height: size, format: pc.PIXELFORMAT_RGBA8, mipmaps: true, minFilter: pc.FILTER_LINEAR_MIPMAP_LINEAR, magFilter: pc.FILTER_LINEAR, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE, anisotropy: 8, levels: [new Uint8Array(data.buffer)] });
    this.texture = texture;
    this.area.set([0, 0, 1 / W, 1 / D]);
  }

  /** Whether it is laid over the ground now. */
  get active(): boolean {
    return this.on;
  }

  /** Laid over `materials` (the ground tiles') by `blend` (0: not at all, and not in their shader either). */
  apply(materials: Iterable<pc.StandardMaterial>, blend: number): void {
    const on = blend > 0 && !!this.texture;
    for (const m of materials) {
      if (on !== this.on) {
        m.setDefine('GAME_TOWN_GROUND', on ? '' : false);
        m.update();
      }
      if (on) {
        m.setParameter('game_townMap', this.texture!);
        m.setParameter('game_townMapArea', this.area);
        m.setParameter('game_townGround', blend);
      }
    }
    this.on = on;
  }

  dispose(): void {
    this.texture?.destroy();
    this.texture = null;
    this.key = '';
  }
}
