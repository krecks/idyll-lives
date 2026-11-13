/**
 * A big world's ground tiles (`nature.ts` `terrainTiles`): each has three levels of detail and draws
 * one of them, chosen by distance (`frameTiles`). Apart, so it can be checked on the engine's null
 * device without the rest of the landscape. And the town view's colour map over them (`TownGroundMap`).
 */

import * as pc from './engine';
import { groundDepth, type RoadPlan, type WorldStructure, type WorldZone } from '../../core/protocol';
import { carriageway, hasSidewalks } from '../shared/junctions';
import { backdropGround } from '../shared/townBuildings';

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

// ---- the town view's colour map (a ground colour map at altitude) ----

/** Texels a metre at most, and the map's side at most (2048: half a metre a texel over a Big world). */
const MAP_PER_M = 2;
const MAP_MAX = 2048;
/** The map's colours (sRGB): the asphalt and pavements as the street's textures and tones come out, the centre's paving, water, soil, paths, and what the backdrop's buildings stand on. */
const MAP = {
  asphalt: '#55544F',
  pavement: '#C4C0B6',
  paving: '#B3A58E',
  water: '#3E6E80',
  soil: '#6B4E37',
  furrow: '#5A412E',
  path: '#A99C86',
  forecourt: '#B9B3A6',
  yard: '#9A8C74',
  joint: '#A4967F',
} as const;

/** What the map asks of the land: a big world's farmland as the terrain colours it (`LandscapeLayout.farmland`). */
export interface TownGroundLand {
  farmland(): { size: number; across: number; angle: number; crop(cu: number, cv: number, out: { r: number; g: number; b: number }): boolean; weight(x: number, z: number): number } | null;
}

/** The centre's paving's courses (m apart). */
const COURSE = 5;
/** How wide a building's way to the road is drawn (m): a forecourt's or a yard's carried on to it, a garden's path. */
const WAY = { forecourt: 3, yard: 3, garden: 1.4 } as const;
/** How far apart (m) the farmland's strength is asked (drawn smoothed between), and how far round the field zones (their soft edge). */
const FIELD_MASK = 4;
const FIELD_SOFT = 8;

/**
 * The ground's colour map at altitude: what the town lays on its land (streets and pavements, the junctions'
 * pads, the centre's paving, its water, its farm fields and paths, a big world's farmland as its ground has it,
 * and what the backdrop's buildings stand on: forecourts, yards, a way to the road) over the whole
 * world in one texture, baked
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

  /**
   * Bakes the map for `world` (nothing again for the same town; nothing without a 2D canvas: the tests' engine).
   * `land`: the landscape, for a big world's farmland.
   */
  bake(world: WorldStructure, land: TownGroundLand | null = null): void {
    const meta = world.meta;
    const W = world.width;
    const D = groundDepth(world);
    const key = `${meta?.seed ?? 0}|${W}x${D}|${meta?.roads?.length ?? 0}|${world.plots.length}|${meta?.centre?.length ?? 0}|${meta?.backdrop?.length ?? 0}|${land ? 1 : 0}`;
    if (key === this.key || typeof OffscreenCanvas === 'undefined' || !meta) return;
    this.key = key;
    const size = Math.min(MAP_MAX, 2 ** Math.ceil(Math.log2(Math.max(W, D) * MAP_PER_M)));
    const sx = size / W;
    const sz = size / D;
    const canvas = new OffscreenCanvas(size, size);
    const g = canvas.getContext('2d', { willReadFrequently: true })!;
    // A big world's farmland first, under everything: the patchwork its ground has, each field's edge where it is.
    const farmland = land?.farmland();
    if (farmland && meta.zones) this.farmland(g, size, sx, sz, meta.zones, farmland);
    g.setTransform(sx, 0, 0, sz, 0, 0);
    g.lineCap = 'round';
    g.lineJoin = 'round';
    const poly = (p: readonly (readonly [number, number])[]) => {
      g.beginPath();
      p.forEach(([x, z], i) => (i ? g.lineTo(x, z) : g.moveTo(x, z)));
      g.closePath();
    };
    const roads = meta.roads ?? [];
    // Water (a big world's water zones: its river and ponds).
    g.fillStyle = MAP.water;
    for (const z of meta.zones ?? []) {
      if (z.kind !== 'water') continue;
      poly(z.poly);
      g.fill();
    }
    // The centre's paving, laid in courses (`COURSE` m apart, a joint's line each way): a square's stone, not a slab.
    for (const c of meta.centre ?? []) {
      g.fillStyle = MAP.paving;
      poly(c.poly);
      g.fill();
      let [x0, z0, x1, z1] = [Infinity, Infinity, -Infinity, -Infinity];
      for (const [x, z] of c.poly) [x0, z0, x1, z1] = [Math.min(x0, x), Math.min(z0, z), Math.max(x1, x), Math.max(z1, z)];
      g.save();
      poly(c.poly);
      g.clip();
      g.fillStyle = MAP.joint;
      for (let x = Math.ceil(x0 / COURSE) * COURSE; x < x1; x += COURSE) g.fillRect(x - 0.25, z0, 0.5, z1 - z0);
      for (let z = Math.ceil(z0 / COURSE) * COURSE; z < z1; z += COURSE) g.fillRect(x0, z - 0.25, x1 - x0, 0.5);
      g.restore();
    }
    // What the backdrop's buildings stand on (`backdropGround`: a forecourt, a yard; a garden's hedge is drawn with
    // the buildings, `gardenHedges`), and their ways to the road.
    const driven = roads.filter((r) => r.kind !== 'trail');
    const grounds = (meta.backdrop ?? []).filter((b) => b.poly.length >= 3).map((b) => backdropGround(b, driven));
    for (const way of [true, false]) {
      for (const b of grounds) {
        const colour = b.kind === 'garden' ? MAP.path : MAP[b.kind];
        if (way && b.to) {
          g.strokeStyle = colour;
          g.lineWidth = WAY[b.kind];
          g.beginPath();
          g.moveTo(b.from[0], b.from[1]);
          g.lineTo(b.to[0], b.to[1]);
          g.stroke();
        } else if (!way && b.kind !== 'garden') {
          g.fillStyle = colour;
          poly(b.round);
          g.fill();
        }
      }
    }
    // None of that on the lots (their gardens stay theirs) or the greens.
    g.globalCompositeOperation = 'destination-out';
    g.fillStyle = '#000';
    for (const p of world.plots) g.fillRect(p.x, p.z, p.w, p.d);
    for (const green of meta.greens ?? []) {
      poly(green);
      g.fill();
    }
    g.globalCompositeOperation = 'source-over';
    // Farm fields on farm lots: soil in furrows.
    for (const f of meta.world?.fields ?? []) {
      g.fillStyle = MAP.soil;
      g.fillRect(f.x, f.z, f.w, f.d);
      g.fillStyle = MAP.furrow;
      const along = f.w >= f.d;
      for (let k = 0.5; k < (along ? f.d : f.w); k += 1.2) (along ? g.fillRect(f.x, f.z + k, f.w, 0.45) : g.fillRect(f.x + k, f.z, 0.45, f.d));
    }
    // Paths, then the roads: their pavements (a road's width has them), then their carriageways, trails as paths.
    g.fillStyle = MAP.path;
    for (const p of meta.paths ?? []) g.fillRect(p.x, p.z, p.w, p.d);
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
    const texture = new pc.Texture(this.device, { name: 'townGroundMap', width: size, height: size, format: pc.PIXELFORMAT_RGBA8, mipmaps: true, minFilter: pc.FILTER_LINEAR_MIPMAP_LINEAR, magFilter: pc.FILTER_LINEAR, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE, levels: [new Uint8Array(data.buffer)] });
    this.texture = texture;
    this.area.set([0, 0, 1 / W, 1 / D]);
  }

  /**
   * A big world's farmland into the map (before anything else is drawn): the patchwork cell by cell, each a
   * rectangle in its crop's colour (the canvas draws its edges where they are), shown as strongly as the terrain
   * shows it (`land.weight`, asked `FIELD_MASK` m apart and smoothed between: the fields fade out at their zones'
   * edges and by the town).
   */
  private farmland(g: OffscreenCanvasRenderingContext2D, size: number, sx: number, sz: number, zones: readonly WorldZone[], land: NonNullable<ReturnType<TownGroundLand['farmland']>>): void {
    let [x0, z0, x1, z1] = [Infinity, Infinity, -Infinity, -Infinity];
    for (const zone of zones) {
      if (zone.kind !== 'fields') continue;
      for (const [x, z] of zone.poly) [x0, z0, x1, z1] = [Math.min(x0, x), Math.min(z0, z), Math.max(x1, x), Math.max(z1, z)];
    }
    const i0 = Math.max(0, Math.floor((x0 - FIELD_SOFT) * sx));
    const j0 = Math.max(0, Math.floor((z0 - FIELD_SOFT) * sz));
    const i1 = Math.min(size, Math.ceil((x1 + FIELD_SOFT) * sx));
    const j1 = Math.min(size, Math.ceil((z1 + FIELD_SOFT) * sz));
    if (i1 <= i0 || j1 <= j0) return;
    const [w, h] = [i1 - i0, j1 - j0];
    // (The box in metres, as the texels have it.)
    [x0, z0, x1, z1] = [i0 / sx, j0 / sz, i1 / sx, j1 / sz];
    const region = new OffscreenCanvas(w, h);
    const r = region.getContext('2d')!;
    r.setTransform(sx, 0, 0, sz, -i0, -j0);
    const [c, s] = [Math.cos(land.angle), Math.sin(land.angle)];
    const us = [x0 * c + z0 * s, x1 * c + z0 * s, x1 * c + z1 * s, x0 * c + z1 * s].map((u) => u / land.size);
    const vs = [-x0 * s + z0 * c, -x1 * s + z0 * c, -x1 * s + z1 * c, -x0 * s + z1 * c].map((v) => v / land.across);
    const at = (u: number, v: number) => [u * land.size * c - v * land.across * s, u * land.size * s + v * land.across * c] as const;
    // (Each cell a touch over its lines: no seam between two.)
    const [eu, ev] = [0.25 / land.size, 0.25 / land.across];
    const col = { r: 0, g: 0, b: 0 };
    for (let cu = Math.floor(Math.min(...us)); cu <= Math.floor(Math.max(...us)); cu++) {
      for (let cv = Math.floor(Math.min(...vs)); cv <= Math.floor(Math.max(...vs)); cv++) {
        if (!land.crop(cu, cv, col)) continue;
        r.fillStyle = `rgb(${Math.round(col.r * 255)}, ${Math.round(col.g * 255)}, ${Math.round(col.b * 255)})`;
        r.beginPath();
        [at(cu - eu, cv - ev), at(cu + 1 + eu, cv - ev), at(cu + 1 + eu, cv + 1 + ev), at(cu - eu, cv + 1 + ev)].forEach(([x, z], k) => (k ? r.lineTo(x, z) : r.moveTo(x, z)));
        r.closePath();
        r.fill();
      }
    }
    // How strongly it shows: a small picture of the strength, drawn over the patchwork smoothed (what it keeps of it).
    const mw = Math.ceil((x1 - x0) / FIELD_MASK) + 1;
    const mh = Math.ceil((z1 - z0) / FIELD_MASK) + 1;
    const mask = new OffscreenCanvas(mw, mh);
    const m = mask.getContext('2d')!;
    const img = m.createImageData(mw, mh);
    for (let j = 0; j < mh; j++) {
      for (let i = 0; i < mw; i++) {
        const o = (j * mw + i) * 4;
        img.data[o] = img.data[o + 1] = img.data[o + 2] = 255;
        img.data[o + 3] = Math.round(Math.min(1, Math.max(0, land.weight(x0 + (i + 0.5) * FIELD_MASK, z0 + (j + 0.5) * FIELD_MASK))) * 255);
      }
    }
    m.putImageData(img, 0, 0);
    r.setTransform(1, 0, 0, 1, 0, 0);
    r.globalCompositeOperation = 'destination-in';
    r.imageSmoothingEnabled = true;
    r.imageSmoothingQuality = 'high';
    r.drawImage(mask, 0, 0, mw, mh, 0, 0, mw * FIELD_MASK * sx, mh * FIELD_MASK * sz);
    g.drawImage(region, i0, j0);
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
