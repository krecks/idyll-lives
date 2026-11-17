/**
 * The landscape's water flowing (`shaders/water.ts`): the ripples' texture (made at run time, one per device) and a
 * landscape's flow map, a texel a sample of its height field (`shared/waterFlow.ts`), worked out again where the land
 * changes (a brush's stamp, a stroke done) and sent up whole. A landscape with no height field has a still map (its
 * sea, its lake, only ripple).
 */

import * as pc from './engine';
import './shaders/water';
import { benchDiag } from '../../debug/benchFlags';
import { LAND_CELL, type Land, type LandRect } from '../../game/land';
import { mulberry32 } from '../../game/noise';
import { sameLand } from '../shared/landBake';
import { flowReach, flowTexels, riverCourse, stillFlow, waterFlow, type RiverCourse, type WaterFlow } from '../shared/waterFlow';

/** The ripples' texture: texels a side (one repeat), and how many waves make it. */
const RIPPLE_SIZE = 128;
const RIPPLE_WAVES = 28;

const ripples = new WeakMap<pc.GraphicsDevice, pc.Texture>();

/**
 * The ripples (`game_waterRipple`): waves of whole numbers of wavelengths across the texture (so it repeats without a
 * seam), shorter ones lower, at random ways and phases; RG their slope along x and z (128 level), B their height, A
 * 255. With mips, so far-off water reads them averaged out (calm), not as noise.
 */
export function rippleTexels(size = RIPPLE_SIZE): Uint8Array {
  const rand = mulberry32(0x5eed);
  const waves: { kx: number; kz: number; a: number; p: number }[] = [];
  while (waves.length < RIPPLE_WAVES) {
    const [kx, kz] = [Math.round((rand() * 2 - 1) * 9), Math.round((rand() * 2 - 1) * 9)];
    const k = Math.hypot(kx, kz);
    if (k < 2 || k > 9) continue;
    waves.push({ kx, kz, a: 1 / k ** 1.6, p: rand() * Math.PI * 2 });
  }
  const height = new Float32Array(size * size);
  const sx = new Float32Array(size * size);
  const sz = new Float32Array(size * size);
  for (let j = 0; j < size; j++) {
    for (let i = 0; i < size; i++) {
      let [h, gx, gz] = [0, 0, 0];
      for (const w of waves) {
        const t = ((w.kx * i + w.kz * j) / size) * Math.PI * 2 + w.p;
        h += w.a * Math.sin(t);
        const c = w.a * Math.cos(t) * Math.PI * 2;
        gx += c * w.kx;
        gz += c * w.kz;
      }
      height[j * size + i] = h;
      sx[j * size + i] = gx;
      sz[j * size + i] = gz;
    }
  }
  let [lo, hi, steep] = [Infinity, -Infinity, 0];
  for (let k = 0; k < height.length; k++) {
    [lo, hi] = [Math.min(lo, height[k]), Math.max(hi, height[k])];
    steep = Math.max(steep, Math.abs(sx[k]), Math.abs(sz[k]));
  }
  const out = new Uint8Array(size * size * 4);
  for (let k = 0; k < height.length; k++) {
    out[k * 4] = Math.round(128 + (127 * sx[k]) / steep);
    out[k * 4 + 1] = Math.round(128 + (127 * sz[k]) / steep);
    out[k * 4 + 2] = Math.round((255 * (height[k] - lo)) / (hi - lo));
    out[k * 4 + 3] = 255;
  }
  return out;
}

function rippleTexture(device: pc.GraphicsDevice): pc.Texture {
  let tex = ripples.get(device);
  if (!tex) {
    tex = new pc.Texture(device, { name: 'waterRipple', width: RIPPLE_SIZE, height: RIPPLE_SIZE, format: pc.PIXELFORMAT_RGBA8, mipmaps: true, minFilter: pc.FILTER_LINEAR_MIPMAP_LINEAR, magFilter: pc.FILTER_LINEAR, addressU: pc.ADDRESS_REPEAT, addressV: pc.ADDRESS_REPEAT, levels: [rippleTexels()] });
    ripples.set(device, tex);
  }
  return tex;
}

/** `?diag=noflow`: the water as it was before it flowed (the plain material), to measure what the flowing costs. */
export const flowingWater = () => !benchDiag().has('noflow');

/**
 * A landscape's flow map, on its water's material (`use`): from its height field (`setLand`), worked out again over
 * what changed (`changed`).
 */
export class WaterFlowMap {
  private texture: pc.Texture;
  private texels: Uint8Array;
  private flow: WaterFlow | null = null;
  private course: RiverCourse | null = null;
  private land: Land | null = null;
  private readonly area = new Float32Array([1, 1, 0.5, 0.5]);
  private readonly materials = new Set<pc.StandardMaterial>();

  constructor(private readonly device: pc.GraphicsDevice) {
    this.texels = Uint8Array.from([128, 128, 0, 255]);
    this.texture = this.made(1, 1);
  }

  /** Draws `material` (the water's) flowing, from this map; `still`: ripples only (the Low preset). */
  use(material: pc.StandardMaterial, still: boolean): void {
    material.setDefine('GAME_WATER', '');
    if (still) material.setDefine('GAME_WATER_STILL', '');
    // Out of doors: no room's light or windowless mask read (texture units, `samplerCheck.ts`).
    material.setDefine('GAME_OUTDOORS', '');
    material.setParameter('game_waterRipple', rippleTexture(this.device));
    this.materials.add(material);
    this.bind(material);
    material.update();
  }

  /** The land its water flows on, worked out whole (null: none, all still). */
  setLand(land: Land | null): void {
    // (The same land again, a copy of it: taken on as it is.)
    if (land && this.land && this.flow && land !== this.land && sameLand(land, this.land)) {
      this.land = land;
      return;
    }
    this.land = land;
    if (!land) {
      this.flow = null;
      this.course = null;
      this.resize(1, 1);
      this.texels.set([128, 128, 0, 255]);
      this.upload();
      return;
    }
    this.course = riverCourse(land);
    this.flow = waterFlow(land, this.course, undefined, stillFlow(land.cols, land.rows));
    this.resize(land.cols, land.rows);
    flowTexels(this.flow, { i0: 0, j0: 0, i1: land.cols, j1: land.rows }, this.texels);
    this.upload();
  }

  /**
   * The land's samples in `rect` changed (a stamp; `done`, a stroke done: its river's course looked for again): the
   * flow worked out again as far as they reach, and sent up.
   */
  changed(rect: LandRect, done: boolean): void {
    const land = this.land;
    if (!land || !this.flow || this.flow.cols !== land.cols || this.flow.rows !== land.rows) {
      this.setLand(land);
      return;
    }
    if (done) this.course = riverCourse(land);
    const reach = flowReach(land, rect);
    waterFlow(land, this.course, reach, this.flow);
    flowTexels(this.flow, reach, this.texels);
    this.upload();
  }

  destroy(): void {
    this.texture.destroy();
    this.materials.clear();
  }

  private made(width: number, height: number): pc.Texture {
    return new pc.Texture(this.device, { name: 'waterFlow', width, height, format: pc.PIXELFORMAT_RGBA8, mipmaps: false, minFilter: pc.FILTER_LINEAR, magFilter: pc.FILTER_LINEAR, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE, levels: [this.texels] });
  }

  /** The map at `cols` × `rows` texels (a new texture if its size changed), on every material using it. */
  private resize(cols: number, rows: number): void {
    this.area.set([1 / (cols * LAND_CELL), 1 / (rows * LAND_CELL), 0.5 / cols, 0.5 / rows]);
    if (this.texture.width !== cols || this.texture.height !== rows) {
      this.texels = new Uint8Array(cols * rows * 4);
      const old = this.texture;
      this.texture = this.made(cols, rows);
      old.destroy();
    }
    for (const m of this.materials) this.bind(m);
  }

  private bind(material: pc.StandardMaterial): void {
    material.setParameter('game_waterFlow', this.texture);
    material.setParameter('game_waterArea', this.area);
  }

  private upload(): void {
    const data = this.texture.lock() as Uint8Array;
    if (data !== this.texels) data.set(this.texels);
    this.texture.unlock();
  }
}
