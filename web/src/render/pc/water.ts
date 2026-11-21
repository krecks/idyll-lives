/**
 * The landscape's water flowing (`shaders/water.ts`): the ripples' texture (made at run time, one per device) and a
 * landscape's flow map, a texel a sample of its height field (`shared/waterFlow.ts`) and a still one round it (read
 * clamped, nothing flows past the land: only the still sea goes on there), worked out again where the land changes (a
 * brush's stamp, a stroke done) and sent up whole. A landscape with no height field has a still map (its sea, its
 * lake, only ripple).
 */

import * as pc from './engine';
import './shaders/water';
import { benchDiag } from '../../debug/benchFlags';
import { LAND_CELL, type Land, type LandRect } from '../../game/land';
import { mulberry32 } from '../../game/noise';
import { sameLand } from '../shared/landBake';
import { flowReach, flowTexels, riverCourse, stillFlow, waterFlow, type RiverCourse, type WaterFlow } from '../shared/waterFlow';

/** The still texels round the flow map. */
const BORDER = 1;
/** A still texel. */
const STILL = [128, 128, 0, 255];
/** Samples a side of the squares whose water is looked for in view (`inView`). */
const BLOCK = 32;

/** The ripples' texture: texels a side (one repeat), and how many waves make it. */
const RIPPLE_SIZE = 128;
const RIPPLE_WAVES = 28;

const ripples = new WeakMap<pc.GraphicsDevice, pc.Texture>();

/**
 * The ripples (`game_waterRipple`): waves of whole numbers of wavelengths across the texture (so it repeats without a
 * seam), shorter ones lower, at random ways and phases; RG their slope along x and z (128 level), B their height, A
 * a finer noise of the same kind, evened out (the foam's). With mips, so far-off water reads them averaged out
 * (calm), not as noise.
 */
export function rippleTexels(size = RIPPLE_SIZE): Uint8Array {
  const rand = mulberry32(0x5eed);
  const waves = wavesOf(rand, 2, 9, 1.6);
  const fine = wavesOf(rand, 4, 12, 1);
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
  const foam = new Float32Array(size * size);
  for (let j = 0; j < size; j++) {
    for (let i = 0; i < size; i++) {
      let h = 0;
      for (const w of fine) h += w.a * Math.sin(((w.kx * i + w.kz * j) / size) * Math.PI * 2 + w.p);
      foam[j * size + i] = h;
    }
  }
  // (Evened out by rank, so a share of the texels over a height is that share of the foam: `shaders/water.ts`.)
  const rank = new Float32Array(foam.length);
  const order = Array.from(foam.keys()).sort((a, b) => foam[a] - foam[b]);
  order.forEach((k, n) => (rank[k] = n / (order.length - 1)));
  const out = new Uint8Array(size * size * 4);
  for (let k = 0; k < height.length; k++) {
    out[k * 4] = Math.round(128 + (127 * sx[k]) / steep);
    out[k * 4 + 1] = Math.round(128 + (127 * sz[k]) / steep);
    out[k * 4 + 2] = Math.round((255 * (height[k] - lo)) / (hi - lo));
    out[k * 4 + 3] = Math.round(255 * rank[k]);
  }
  return out;
}

/** `RIPPLE_WAVES` waves of `from` to `to` wavelengths across the texture, `1 / k ^ fall` high, at random ways and phases. */
function wavesOf(rand: () => number, from: number, to: number, fall: number): { kx: number; kz: number; a: number; p: number }[] {
  const waves: { kx: number; kz: number; a: number; p: number }[] = [];
  while (waves.length < RIPPLE_WAVES) {
    const [kx, kz] = [Math.round((rand() * 2 - 1) * to), Math.round((rand() * 2 - 1) * to)];
    const k = Math.hypot(kx, kz);
    if (k < from || k > to) continue;
    waves.push({ kx, kz, a: 1 / k ** fall, p: rand() * Math.PI * 2 });
  }
  return waves;
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
  /** By square of `BLOCK` samples, the lowest and highest water in it (m; NaN: dry), for `inView`. */
  private blocks = new Float32Array(0);
  private readonly box = new pc.BoundingBox();
  private readonly seen = new pc.BoundingBox();
  private readonly sphere = new pc.BoundingSphere();

  constructor(private readonly device: pc.GraphicsDevice) {
    this.texels = Uint8Array.from(STILL);
    this.texture = this.made(1, 1);
  }

  /** Draws `material` (the water's) flowing, from this map. */
  use(material: pc.StandardMaterial): void {
    material.setDefine('GAME_WATER', '');
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
      this.blocks = new Float32Array(0);
      this.resize(1, 1);
      for (let t = 0; t < this.texels.length; t += 4) this.texels.set(STILL, t);
      this.upload();
      return;
    }
    this.course = riverCourse(land);
    this.flow = waterFlow(land, this.course, undefined, stillFlow(land.cols, land.rows));
    this.blocks = new Float32Array(Math.ceil(land.cols / BLOCK) * Math.ceil(land.rows / BLOCK) * 2);
    this.measure({ i0: 0, j0: 0, i1: land.cols, j1: land.rows });
    this.resize(land.cols, land.rows);
    flowTexels(this.flow, { i0: 0, j0: 0, i1: land.cols, j1: land.rows }, this.texels, BORDER);
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
    flowTexels(this.flow, reach, this.texels, BORDER);
    this.upload();
    this.measure(rect);
  }

  /**
   * Whether the land's water (null: it has none to look at: no land) has any square in the view of `frustum` (world
   * space), the water drawn under `toWorld` (its draws' parent: game space to the world's).
   */
  inView(frustum: pc.Frustum, toWorld: pc.Mat4): boolean | null {
    const land = this.land;
    if (!land) return null;
    const across = Math.ceil(land.cols / BLOCK);
    const half = (BLOCK * LAND_CELL) / 2;
    for (let b = 0; b * 2 < this.blocks.length; b++) {
      const lo = this.blocks[b * 2];
      if (Number.isNaN(lo)) continue;
      const hi = this.blocks[b * 2 + 1];
      this.box.center.set(((b % across) * BLOCK) * LAND_CELL + half, (lo + hi) / 2, Math.floor(b / across) * BLOCK * LAND_CELL + half);
      this.box.halfExtents.set(half, (hi - lo) / 2 + 0.5, half);
      this.seen.setFromTransformedAabb(this.box, toWorld);
      this.sphere.center.copy(this.seen.center);
      this.sphere.radius = this.seen.halfExtents.length();
      if (frustum.containsSphere(this.sphere) > 0) return true;
    }
    return false;
  }

  /** The squares `rect` (samples) reaches measured again: the lowest and highest of their water. */
  private measure(rect: LandRect): void {
    const land = this.land;
    if (!land) return;
    const { cols, rows, h, wl } = land;
    const across = Math.ceil(cols / BLOCK);
    for (let bj = Math.floor(rect.j0 / BLOCK); bj < Math.ceil(Math.min(rows, rect.j1) / BLOCK); bj++) {
      for (let bi = Math.floor(rect.i0 / BLOCK); bi < Math.ceil(Math.min(cols, rect.i1) / BLOCK); bi++) {
        let [lo, hi] = [Infinity, -Infinity];
        for (let j = bj * BLOCK; j < Math.min(rows, (bj + 1) * BLOCK); j++) {
          for (let k = j * cols + bi * BLOCK, end = j * cols + Math.min(cols, (bi + 1) * BLOCK); k < end; k++) {
            if (wl[k] <= h[k]) continue;
            [lo, hi] = [Math.min(lo, wl[k]), Math.max(hi, wl[k])];
          }
        }
        const b = (bj * across + bi) * 2;
        [this.blocks[b], this.blocks[b + 1]] = hi < lo ? [NaN, NaN] : [lo / 100, hi / 100];
      }
    }
  }

  destroy(): void {
    this.texture.destroy();
    this.materials.clear();
  }

  private made(width: number, height: number): pc.Texture {
    return new pc.Texture(this.device, { name: 'waterFlow', width, height, format: pc.PIXELFORMAT_RGBA8, mipmaps: false, minFilter: pc.FILTER_LINEAR, magFilter: pc.FILTER_LINEAR, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE, levels: [this.texels] });
  }

  /** The map for `cols` × `rows` samples, its still border round them (a new texture if its size changed), on every material using it. */
  private resize(cols: number, rows: number): void {
    const [w, h] = [cols + 2 * BORDER, rows + 2 * BORDER];
    // (Sample i at x = i × LAND_CELL is texel i + BORDER: its middle at (i + BORDER + 0.5) / w.)
    this.area.set([1 / (w * LAND_CELL), 1 / (h * LAND_CELL), (BORDER + 0.5) / w, (BORDER + 0.5) / h]);
    if (this.texture.width !== w || this.texture.height !== h) {
      this.texels = new Uint8Array(w * h * 4);
      for (let t = 0; t < this.texels.length; t += 4) this.texels.set(STILL, t);
      const old = this.texture;
      this.texture = this.made(w, h);
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
