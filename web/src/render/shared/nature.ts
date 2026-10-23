/**
 * The engine-neutral half of the landscape round the town (`babylon/nature.ts`, `pc/nature.ts` draw
 * it): the land's shape (rolling hills, a valley's rim, mountains, fields, a sea or a lake, levelled
 * under the country roads), the terrain's colours, where the woods and the lawn's tufts grow, the
 * lawn hidden indoors, the garden dressing split into species, and the wind's numbers as both
 * renderers' plant shaders sway them. Everything is generated deterministically from the town's seed,
 * so a loaded game looks the same, and both renderers place the same plants in the same order.
 *
 * Placements are matrices in game space (Babylon's layout, `math.ts`): 16 floats each.
 */

import { DEFAULT_LANDSCAPE, groundDepth, type LandscapeParams, type WorldStructure } from '../../core/protocol';
import { CountryRoads, ROAD_VERGE, type RoadLand } from './countryRoads';
import { compose, quatAxisAngle } from './math';

/**
 * How far the landscape extends beyond the town (metres): towns of older code, and worlds (which
 * reach further, to a farther horizon). The valley's rim rises over `RING` either way.
 */
export const RING = 150;
const WORLD_RING = 260;
/** Flat apron around the town before the hills start. */
export const APRON = 6;
const TERRAIN_STEP = 2.5;
/** At most this many terrain quads across (a wider landscape gets coarser ones). */
const TERRAIN_SEGMENTS = 260;
/** Trees within this distance of the town cast shadows; further ones don't. */
const SHADOW_RANGE = 28;
/** Mountains: no trees above about this height (m). */
const TREELINE = 12;
/** Height of the sea or a lake (m); the shore is where the land meets it. */
export const WATER_Y = -0.35;
/** Sand from the shoreline up the bank (m). */
const BEACH = 8;
/**
 * In the game, plants are drawn in groups the camera leaves out when they're out of view (a model
 * drawn as one mesh over the whole landscape never is): the woods in this many sectors round the
 * town, lawn tufts in squares of this size (m). A model part with fewer triangles in all than
 * `SPLIT_TRIANGLES` (bark, flowers) stays one mesh: the draw calls more would cost the main thread
 * more than the triangles they save. Each group is a mesh of its own, and every mesh costs the main
 * thread a little each frame whether drawn or not, so the squares are wide. The menus' town, which
 * has the whole landscape in view, draws each part whole.
 */
const FOREST_SECTORS = 8;
const TUFT_CELL = 64;
export const SPLIT_TRIANGLES = 36_000;

/** A tree, bush or hedge as placed (the garden's, the street's, the woods'): its model, where it stands, its scale (x, up, z) and turn. */
export interface PlantPlacement {
  key: string;
  x: number;
  y: number;
  z: number;
  sx: number;
  sy: number;
  sz: number;
  yaw: number;
}

// ---- Deterministic noise ------------------------------------------------------------

export function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export function hash(x: number, z: number, seed: number): number {
  let h = (Math.imul(x, 374761393) + Math.imul(z, 668265263) + Math.imul(seed, 2147483647)) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

function valueNoise(x: number, z: number, seed: number): number {
  const xi = Math.floor(x);
  const zi = Math.floor(z);
  const xf = x - xi;
  const zf = z - zi;
  const u = xf * xf * (3 - 2 * xf);
  const v = zf * zf * (3 - 2 * zf);
  const a = hash(xi, zi, seed);
  const b = hash(xi + 1, zi, seed);
  const c = hash(xi, zi + 1, seed);
  const d = hash(xi + 1, zi + 1, seed);
  return a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v;
}

/** Ridged noise, 0..1: sharp crests where plain noise crosses its middle (mountain ridges). */
function ridged(x: number, z: number, seed: number, octaves = 5): number {
  let sum = 0;
  let amp = 0.5;
  let freq = 1;
  let weight = 1;
  for (let i = 0; i < octaves; i++) {
    const r = Math.pow(1 - Math.abs(valueNoise(x * freq, z * freq, seed + i * 31) * 2 - 1), 2) * weight;
    sum += r * amp;
    // Detail mostly on the ridges, little in the valleys.
    weight = Math.min(1, r * 1.6);
    amp *= 0.5;
    freq *= 2.1;
  }
  return Math.min(1, sum / 0.9);
}

function fbm(x: number, z: number, seed: number, octaves = 4): number {
  let sum = 0;
  let amp = 0.5;
  let freq = 1;
  for (let i = 0; i < octaves; i++) {
    sum += amp * valueNoise(x * freq, z * freq, seed + i * 17);
    amp *= 0.5;
    freq *= 2;
  }
  return sum;
}

export const smoothstep = (a: number, b: number, x: number) => {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};

// ---- Colours ------------------------------------------------------------------------

/** A colour as Babylon's `Color3` holds one (0..1 per channel). */
interface Rgb {
  r: number;
  g: number;
  b: number;
}

/** An sRGB hex colour as Babylon's `Color3.FromHexString` reads it. */
function hex(c: string): Rgb {
  return { r: parseInt(c.substring(1, 3), 16) / 255, g: parseInt(c.substring(3, 5), 16) / 255, b: parseInt(c.substring(5, 7), 16) / 255 };
}

/** Babylon's `Color3.LerpToRef` (`out` may be `a`). */
function lerp(a: Rgb, b: Rgb, t: number, out: Rgb): void {
  out.r = a.r + (b.r - a.r) * t;
  out.g = a.g + (b.g - a.g) * t;
  out.b = a.b + (b.b - a.b) * t;
}

// ---- Placements ---------------------------------------------------------------------

/** A placement's matrix: scaled, turned `yaw` about up, at (x, y, z) (Babylon's `Matrix.Compose`). */
function placed(sx: number, sy: number, sz: number, yaw: number, x: number, y: number, z: number): Float32Array {
  return compose(sx, sy, sz, quatAxisAngle(0, 1, 0, yaw), x, y, z, new Float32Array(16)) as Float32Array;
}

function scatter(x: number, z: number, rand: () => number, scale: number, y = 0): Float32Array {
  // (In this order, as ever: the turn, then the spot within the tile.)
  const yaw = rand() * Math.PI * 2;
  const px = x + rand();
  const pz = z + rand();
  return placed(scale, scale, scale, yaw, px, y, pz);
}

/** The woods as placed (`LandscapeLayout.woods`): matrices per model key, near the town (full models, casting) and far (their far versions). */
export interface Woods {
  near: Map<string, Float32Array[]>;
  far: Map<string, Float32Array[]>;
  /** Which of the `FOREST_SECTORS` round the town a point is in (the game draws the woods in groups by it). */
  sector: (x: number, z: number) => string;
}

/** The lawn as placed (`LandscapeLayout.lawns`): each model's matrices, the town tile under each (-1 outside it), and the groups the game draws it in. */
export interface LawnPlacement {
  key: string;
  matrices: Float32Array[];
  tiles: number[];
  cell?: (x: number, z: number) => string;
}

/** Lawn tufts or flowers as drawn: each instance's original matrix and town tile (-1 outside the town), its matrix as drawn (`shown`) and whether it's hidden. */
export interface LawnLayer<D> {
  /** What draws them (each engine's own). */
  draws: D[];
  base: Float32Array;
  shown: Float32Array;
  tiles: Int32Array;
  hidden: Uint8Array;
}

/** Where the lawn is hidden (`refreshLawn`): per town tile, and the split tiles' diagonals. */
export interface LawnMask {
  width: number;
  /** 1 indoors or under furniture; on a tile split by a diagonal wall, 2 or 3: only its half 0 or 1 indoors. */
  hidden: Uint8Array;
  splitAxis: Map<number, 'dp' | 'dn'>;
}

/** The town's ground where no lawn grows at all: its streets, paths and fields (the birds forage on the lawn). */
export function unlawned(world: WorldStructure): readonly { x: number; z: number; w: number; d: number }[] {
  return [...(world.meta?.streets ?? []), ...(world.meta?.paths ?? []), ...(world.meta?.world?.fields ?? [])];
}

/** Where the lawn that grows is hidden now: the tiles indoors (or the half of a split tile that is) and under furniture. */
export function lawnMask(world: WorldStructure): LawnMask {
  const hidden = new Uint8Array(world.width * world.depth);
  for (let i = 0; i < hidden.length; i++) if (world.rooms[i] !== 0) hidden[i] = 1;
  // Split tiles: 2 = half 0 indoors, 3 = half 1 indoors, 4 = both (bit 1 marks "per half").
  const splitAxis = new Map<number, 'dp' | 'dn'>();
  for (const d of world.diagonals ?? []) {
    const i = d.z * world.width + d.x;
    const [a, b] = [d.rooms[0] !== 0, d.rooms[1] !== 0];
    hidden[i] = a && b ? 1 : a ? 2 : b ? 3 : 0;
    splitAxis.set(i, d.axis);
  }
  for (const o of world.objects) {
    for (let z = o.z; z < o.z + o.d; z++) for (let x = o.x; x < o.x + o.w; x++) hidden[z * world.width + x] = 1;
  }
  return { width: world.width, hidden, splitAxis };
}

/**
 * Hides the tufts and flowers of `layers` on the tiles `mask` hides them on (`refreshLawn`), and
 * shows the rest; `upload` is called with each layer whose matrices changed. On a tile split by a
 * diagonal wall, each tuft follows the half it stands in.
 */
export function hideLawn<D>(layers: readonly LawnLayer<D>[], { width, hidden, splitAxis }: LawnMask, upload: (layer: LawnLayer<D>) => void): void {
  for (const layer of layers) {
    let changed = false;
    for (let k = 0; k < layer.tiles.length; k++) {
      const t = layer.tiles[k];
      let hide = t >= 0 ? hidden[t] : 0;
      if (hide > 1) {
        // The tuft's position within its tile decides its half.
        const fx = layer.base[k * 16 + 12] - (t % width);
        const fz = layer.base[k * 16 + 14] - Math.floor(t / width);
        const axis = splitAxis.get(t) ?? 'dp';
        const half = axis === 'dp' ? (fz < fx ? 0 : 1) : fx + fz < 1 ? 0 : 1;
        // Tufts right at the wall would poke through it.
        const nearWall = (axis === 'dp' ? Math.abs(fx - fz) : Math.abs(fx + fz - 1)) < 0.17;
        hide = half === hide - 2 || nearWall ? 1 : 0;
      }
      if (layer.hidden[k] === hide) continue;
      layer.hidden[k] = hide;
      changed = true;
      // An all-zero matrix hides an instance; `base` brings it back once the tile is outdoors again.
      if (hide) layer.shown.fill(0, k * 16, k * 16 + 16);
      else layer.shown.set(layer.base.subarray(k * 16, k * 16 + 16), k * 16);
    }
    if (changed) upload(layer);
  }
}

/** A lawn layer of `indices` into `matrices` (and their tiles), drawn by `draws`. */
export function lawnLayer<D>(draws: D[], matrices: readonly ArrayLike<number>[], tiles: readonly number[], indices: readonly number[]): LawnLayer<D> {
  const base = new Float32Array(indices.length * 16);
  indices.forEach((k, i) => base.set(matrices[k], i * 16));
  return { draws, base, shown: new Float32Array(base), tiles: Int32Array.from(indices, (k) => tiles[k]), hidden: new Uint8Array(indices.length) };
}

/** A model's instances grouped by `cell` (each group's indices into `matrices`, in their order); empty without one. */
export function groupInstances(matrices: readonly ArrayLike<number>[], cell?: (x: number, z: number) => string): Map<string, number[]> {
  const groups = new Map<string, number[]>();
  if (!cell) return groups;
  matrices.forEach((m, i) => {
    const k = cell(m[12], m[14]);
    const group = groups.get(k);
    if (group) group.push(i);
    else groups.set(k, [i]);
  });
  return groups;
}

// ---- Landscape ----------------------------------------------------------------------

/**
 * The land round a town and what grows on it, as both renderers build it (each engine's `Landscape`
 * draws it). `R`: the engine's country roads (their layout is `CountryRoads`'s).
 */
export class LandscapeLayout<R extends CountryRoads = CountryRoads> {
  protected readonly seed: number;
  protected readonly w: number;
  protected readonly d: number;
  /** How the land looks (the world's kind rolled it; older towns: the classic valley). */
  protected readonly land: LandscapeParams;
  /** How far the land reaches past the town (m). */
  protected readonly ring: number;
  /** Where the road leaves town (z of the main street; the mountains part there). */
  private readonly passZ: number | null;
  /** The roads out of town (the ground is levelled under them). */
  readonly roads: R;
  /** The woods' trees and bushes near the town (full models), where birds may live (`woodsNear`). */
  private nearWoods: PlantPlacement[] = [];
  /** `woodsNear`'s last answer, and where it was asked (the same array while that's unchanged). */
  private woodsAsked: { at: string; plants: readonly PlantPlacement[] } | null = null;

  /**
   * `lawnDensity`: the share of the lawn's tufts and flowers drawn (Low: half); `lawnLod` 1 draws
   * them in their far versions (Low, as the street's models: `QualitySettings.streetLod`).
   */
  constructor(
    world: WorldStructure,
    makeRoads: (land: RoadLand, world: WorldStructure, name: string) => R,
    protected readonly lawnDensity = 1,
    protected readonly lawnLod: 0 | 1 = 0,
  ) {
    this.seed = world.meta?.seed ?? 1;
    this.w = world.width;
    this.d = groundDepth(world);
    this.land = { ...DEFAULT_LANDSCAPE, ...world.meta?.world?.landscape };
    this.ring = world.meta?.world ? WORLD_RING : RING;
    const streets = (world.meta?.streets ?? []).filter((s) => s.w >= s.d).sort((a, b) => a.z - b.z);
    const main = streets[Math.floor((streets.length - 1) / 2)];
    this.passZ = main ? main.z + main.d / 2 : null;
    this.roads = makeRoads(
      { seed: this.seed, w: this.w, d: this.d, ring: this.ring, ground: (x, z) => this.ground(x, z), waterEdge: (x, z) => this.waterEdge(x, z), distanceToTown: (x, z) => this.distanceToTown(x, z) },
      world,
      world.meta?.name ?? '',
    );
  }

  /** The woods' trees and bushes within `range` m of a point (those near the town only), for the birds: the same array when asked the same again. */
  woodsNear(x: number, z: number, range: number): readonly PlantPlacement[] {
    const at = `${x},${z},${range}`;
    if (this.woodsAsked?.at !== at) this.woodsAsked = { at, plants: this.nearWoods.filter((p) => Math.hypot(p.x - x, p.z - z) < range) };
    return this.woodsAsked.plants;
  }

  /** Whether mountains rise round the town (the haze is thinner, so they're seen). */
  get mountainous(): boolean {
    return !!this.land.mountains;
  }

  /** Whether there is a sea or a lake. */
  get watered(): boolean {
    return !!this.land.water;
  }

  /** Identity of a town's landscape (built once per town and kept while it is shown). */
  static key(world: WorldStructure): string {
    const look = world.meta?.world;
    const streets = (world.meta?.streets ?? []).map((s) => `${s.z}`).join(',');
    return `${world.meta?.seed ?? 1}:${world.width}x${groundDepth(world)}:${world.meta?.name ?? ''}:${streets}${look ? `:${look.kind}:${JSON.stringify(look.landscape)}` : ''}`;
  }

  /**
   * Metres past the shoreline of the sea or the lake (above 0 in the water, below 0 on land);
   * -Infinity without water. The shore wanders a little but never comes into the town.
   */
  waterEdge(x: number, z: number): number {
    const water = this.land.water;
    if (!water) return -Infinity;
    if (water.kind === 'sea') {
      const past = water.side === 'zmin' ? -z : z - this.d;
      return past - water.shore - fbm(x * 0.02, 3.7, this.seed + 61, 3) * 8;
    }
    const dx = (x - water.x) / water.rx;
    const dz = (z - water.z) / water.rz;
    const r = Math.hypot(dx, dz) * (1 + (fbm(Math.atan2(dz, dx) * 1.3 + 9, 0.5, this.seed + 67, 2) - 0.5) * 0.25);
    return (1 - r) * Math.min(water.rx, water.rz);
  }

  /** Distance from the town rectangle (0 inside). */
  distanceToTown(x: number, z: number): number {
    const dx = Math.max(0 - x, 0, x - this.w);
    const dz = Math.max(0 - z, 0, z - this.d);
    return Math.hypot(dx, dz);
  }

  /** Height of the land, levelled under the roads out of town. */
  heightAt(x: number, z: number): number {
    const h = this.ground(x, z);
    const road = this.roads.near(x, z);
    if (!road) return h;
    // Level with the road across it, then graded back into the land beside it.
    const t = 1 - smoothstep(road.width / 2 + 0.4, road.width / 2 + ROAD_VERGE, road.dist);
    return h + (road.h - 0.02 - h) * t;
  }

  /** How high the mountains rise on this side, 0..1: the valley opens out flat towards low z, where the menus look at the town from. */
  private openness(z: number): number {
    return smoothstep(-0.9, 0.1, (z - this.d / 2) / (this.d / 2 + 90));
  }

  /** Height the mountains (if any) add at a point `dist` from the town. */
  private mountain(x: number, z: number, dist: number): number {
    const top = this.land.mountains;
    if (!top) return 0;
    const ramp = smoothstep(APRON + 10, APRON + 130, dist);
    if (ramp <= 0) return 0;
    const open = this.openness(z);
    // A pass where the road leaves town at each end.
    const past = Math.max(-x, x - this.w);
    const pass = this.passZ === null || past <= 0 ? 1 : 1 - smoothstep(0, 50, past) * (1 - smoothstep(14, 70, Math.abs(z - this.passZ))) * 0.8;
    const crest = ridged(x * 0.0075, z * 0.0075, this.seed + 211);
    return top * ramp * open * pass * (0.25 + 0.75 * crest);
  }

  /** Height of the land before the roads are cut in. */
  ground(x: number, z: number): number {
    const dist = this.distanceToTown(x, z);
    const L = this.land;
    let h = 0;
    if (dist > APRON) {
      const rise = smoothstep(APRON, APRON + L.rise, dist);
      // Among mountains the near side of the valley (towards the menus' camera) stays low.
      const open = L.mountains ? this.openness(z) : 1;
      const hills = (fbm(x * 0.018, z * 0.018, this.seed) * L.hills - (L.hills * 4) / 14) * (0.15 + 0.85 * open);
      const rim = Math.pow(Math.max(0, dist - L.rimFrom) / RING, 2) * L.rim * open;
      // By water the land stays above it (no puddles in the hollows).
      h = Math.max(L.water ? 0 : -0.4, rise * hills + rim) + this.mountain(x, z, dist);
    }
    if (!L.water || dist === 0) return h;
    // Down a sandy bank to the shore, then under the water.
    const e = this.waterEdge(x, z);
    if (e < -BEACH * 1.5) return h;
    if (e > 0) return WATER_Y - Math.min(4, e * 0.22 + e * e * 0.004);
    const bank = WATER_Y - e * 0.09;
    return h + (Math.min(h, bank) - h) * smoothstep(-BEACH * 1.5, -BEACH * 0.3, e);
  }

  /** The terrain's square (m across, centred on the town) and its quads across. */
  protected terrainGrid(): { size: number; segments: number } {
    const size = Math.max(this.w, this.d) + this.ring * 2;
    return { size, segments: Math.min(TERRAIN_SEGMENTS, Math.round(size / TERRAIN_STEP)) };
  }

  /** The sheet of the sea or the lake: its size, at the water line under the whole landscape. */
  protected waterSheet(): { size: number; x: number; y: number; z: number } {
    return { size: Math.max(this.w, this.d) + this.ring * 2, x: this.w / 2, y: WATER_Y, z: this.d / 2 };
  }

  /**
   * The terrain's heights and colours: each vertex of `positions` (x, y, z; in place) lifted to the
   * land, and its colour (RGBA, sRGB, converted on the GPU; it multiplies the grass detail texture)
   * into `colors`.
   */
  protected async shadeTerrain(positions: Float32Array | number[], colors: Float32Array, pace?: () => Promise<void>): Promise<void> {
    // Lush, slightly yellow-green lawn (the classic life-sim look); greener on the hills. Each
    // world has its own greens.
    const [lush, grass, meadow, hill] = this.land.grass.map(hex);
    const dirt = hex('#7C6748');
    const sand = hex('#D8C89A');
    const wetSand = hex('#A99A72');
    const rock = hex('#7A7064');
    const snow = hex('#F1F3F4');
    const alpine = hex('#8FA266');
    const crops = ['#C9B45C', '#7E6142', '#7A9C44', '#B5A35A', '#6C8A3C'].map(hex);
    const furrow = hex('#5F4733');
    const tmp: Rgb = { r: 0, g: 0, b: 0 };
    let seen = 0;
    for (let i = 0, c = 0; i < positions.length; i += 3, c += 4) {
      if (++seen % 4096 === 0) await pace?.();
      const x = positions[i];
      const z = positions[i + 2];
      const h = this.heightAt(x, z);
      positions[i + 1] = h;
      const patch = fbm(x * 0.06, z * 0.06, this.seed + 99, 3);
      const fine = fbm(x * 0.21, z * 0.21, this.seed + 31, 2);
      lerp(lush, grass, smoothstep(0.3, 0.55, fine), tmp);
      lerp(tmp, meadow, smoothstep(0.5, 0.75, patch), tmp);
      lerp(tmp, hill, smoothstep(1, 10, h), tmp);
      // Worn, earthy patches: sparse, and more common on the flat ground near the town.
      const bare = smoothstep(0.66, 0.8, fbm(x * 0.09 + 13, z * 0.09 - 7, this.seed + 5, 3)) * (1 - smoothstep(2, 8, h));
      lerp(tmp, dirt, bare * 0.85, tmp);
      // Farmland: a patchwork of fields (wheat, ploughed earth, green crops) round the town.
      const cell = this.fieldCell(x, z);
      if (cell) {
        const crop = crops[Math.floor(cell.kind * crops.length)];
        lerp(tmp, crop, cell.weight * 0.9, tmp);
        // Ploughed and planted fields show their furrows.
        if (crop !== crops[0]) lerp(tmp, furrow, cell.weight * 0.35 * Math.max(0, Math.sin(cell.v * Math.PI * 1.4)), tmp);
      }
      // Mountains: bare rock above the trees and on steep faces, snow on the tops.
      const top = this.land.mountains;
      if (top) {
        const steep = Math.hypot(this.heightAt(x + 1.5, z) - h, this.heightAt(x, z + 1.5) - h) / 1.5;
        // Pale alpine meadow above the trees, then rock, then snow.
        lerp(tmp, alpine, smoothstep(TREELINE * 0.8, TREELINE * 1.25, h + fine * 6) * 0.8, tmp);
        lerp(tmp, rock, Math.max(smoothstep(top * 0.16, top * 0.28, h + fine * 12), smoothstep(0.7, 1.2, steep) * smoothstep(8, 18, h)) * 0.92, tmp);
        lerp(tmp, snow, smoothstep(top * 0.33, top * 0.45, h + patch * 20) * (1 - smoothstep(1.5, 2.6, steep) * 0.5), tmp);
      }
      // A worn verge of dirt and gravel along the roads.
      const road = this.roads.near(x, z);
      if (road) lerp(tmp, dirt, 0.45 * (1 - smoothstep(road.width / 2, road.width / 2 + 2.2, road.dist)), tmp);
      // Sand along the water, darker where it is wet.
      const e = this.waterEdge(x, z);
      if (e > -BEACH) {
        lerp(tmp, e > -1.5 ? wetSand : sand, smoothstep(-BEACH, -BEACH * 0.45, e), tmp);
      }
      colors[c] = tmp.r;
      colors[c + 1] = tmp.g;
      colors[c + 2] = tmp.b;
      colors[c + 3] = 1;
    }
  }

  /**
   * The farm field a point lies in (farm worlds, outside the town): the field's kind (0..1),
   * how far across it the point is along the furrows (`v`, m), how strongly it shows (fading out
   * with distance and up the hills) and whether the point is by a hedgerow at its edge. Null elsewhere.
   */
  private fieldCell(x: number, z: number): { kind: number; v: number; weight: number; hedge: boolean } | null {
    const f = this.land.fields;
    if (!f) return null;
    const dist = this.distanceToTown(x, z);
    if (dist < APRON + 3) return null;
    const weight = (1 - smoothstep(this.ring * 0.6, this.ring * 0.85, dist)) * smoothstep(APRON + 3, APRON + 6, dist);
    if (weight <= 0) return null;
    const c = Math.cos(f.angle);
    const s = Math.sin(f.angle);
    const u = (x * c + z * s) / f.size;
    const v = (-x * s + z * c) / (f.size * 0.7);
    const [cu, cv] = [Math.floor(u), Math.floor(v)];
    // Some cells are left as meadow.
    const kind = hash(cu, cv, this.seed + 77);
    if (kind > 0.82) return null;
    const edge = Math.min(u - cu, cu + 1 - u, v - cv, cv + 1 - v) * f.size * 0.7;
    return { kind: kind / 0.82, v: (v - cv) * f.size * 0.7, weight, hedge: edge < 1.6 };
  }

  /**
   * Woods outside the town: mixed deciduous / birch / conifer stands with bushes and rocks. Trees
   * near the town are the full models and cast shadows; further ones use the low level of detail
   * (a file's own far version), which don't.
   */
  protected async woods(pace?: () => Promise<void>): Promise<Woods> {
    const rand = mulberry32(this.seed ^ 0x5eed);
    this.nearWoods = [];
    this.woodsAsked = null;
    const near = new Map<string, Float32Array[]>();
    const far = new Map<string, Float32Array[]>();
    const spacing = 4.2;
    const size = Math.max(this.w, this.d) + this.ring * 2;
    const x0 = this.w / 2 - size / 2;
    const z0 = this.d / 2 - size / 2;
    for (let gx = 0; gx * spacing < size; gx++) {
      if (gx % 4 === 0) await pace?.();
      for (let gz = 0; gz * spacing < size; gz++) {
        const x = x0 + (gx + rand()) * spacing;
        const z = z0 + (gz + rand()) * spacing;
        const dist = this.distanceToTown(x, z);
        const L = this.land;
        if (dist < APRON + L.treeline) continue;
        const forest = fbm(x * 0.025, z * 0.025, this.seed + 7, 3);
        let density = smoothstep(0.42 - L.forest, 0.62 - L.forest, forest) * 0.85 + 0.06 + L.forest * 0.25 - (dist < 14 ? 0.05 : 0);
        // Fields are open, with hedgerows and the odd tree along their edges; nothing grows in the water or on the beach.
        const cell = this.fieldCell(x, z);
        if (cell) density = cell.hedge ? 0.55 : density * (1 - cell.weight);
        if (L.water && this.waterEdge(x, z) > -BEACH * 0.8) continue;
        // Bare tops on mountains.
        if (L.mountains && this.heightAt(x, z) > TREELINE + forest * 10) continue;
        // Nothing on the roads or right by them.
        const road = this.roads.near(x, z);
        if (road && road.dist < road.width / 2 + 2.5) continue;
        // The far woods of a wide world thinner (seen through haze; fewer trees to draw).
        if (this.ring > RING && dist > RING && rand() < 0.45) continue;
        if (rand() > density) continue;
        // Species: conifers in the dense cores, birches in groves at the edges, broadleaves elsewhere.
        const grove = fbm(x * 0.06 + 40, z * 0.06 - 15, this.seed + 3, 2);
        const roll = rand();
        let kind: string;
        if (roll < 0.08) kind = 'model.rock';
        else if (roll < 0.2) kind = 'model.bush';
        else if (roll < 0.26) kind = 'model.bush.small';
        else if ((forest > 0.56 && rand() < Math.min(1, (0.62 * L.conifers) / 0.5)) || (L.conifers > 0.5 && rand() < (L.conifers - 0.5) * 1.6)) kind = rand() < 0.4 ? 'model.spruce' : 'model.pine';
        else if (grove > 0.55 - Math.max(0, L.birches - 0.5) * 0.3 && rand() < Math.min(1, (0.7 * L.birches) / 0.5)) kind = 'model.tree.birch';
        else kind = rand() < 0.35 ? 'model.tree.maple' : 'model.tree';
        const tree = kind.startsWith('model.tree') || kind === 'model.pine' || kind === 'model.spruce';
        const scale = kind === 'model.rock' ? 0.6 + rand() * 0.7 : tree ? 0.75 + rand() * 0.6 : 0.8 + rand() * 0.6;
        // (Drawn in this order, as ever: the same woods from the same seed.)
        const up = scale * (0.9 + rand() * 0.25);
        const yaw = rand() * Math.PI * 2;
        const y = this.heightAt(x, z) - 0.05;
        const m = placed(scale, up, scale, yaw, x, y, z);
        const close = dist < SHADOW_RANGE && kind !== 'model.rock';
        if (close) this.nearWoods.push({ key: kind, x, y, z, sx: scale, sy: up, sz: scale, yaw });
        const bucket = close ? near : far;
        bucket.set(kind, [...(bucket.get(kind) ?? []), m]);
      }
    }
    const [cx, cz] = [this.w / 2, this.d / 2];
    const sector = (x: number, z: number) => `${Math.floor((Math.atan2(z - cz, x - cx) / (Math.PI * 2) + 1) * FOREST_SECTORS) % FOREST_SECTORS}`;
    return { near, far, sector };
  }

  /**
   * Grass tufts and wildflowers on the town's tiles. Every unpaved tile gets some; the ones
   * indoors or under furniture are hidden by `refreshLawn` (rooms and furniture change in build mode).
   */
  protected async lawns(world: WorldStructure, pace?: () => Promise<void>): Promise<LawnPlacement[]> {
    const rand = mulberry32(this.seed ^ 0x1a3d);
    const covered = unlawned(world);
    const onPavement = (x: number, z: number) => covered.some((r) => x >= r.x && x < r.x + r.w && z >= r.z && z < r.z + r.d);
    const tufts: Float32Array[] = [];
    const flowers: Float32Array[] = [];
    const tuftTiles: number[] = [];
    const flowerTiles: number[] = [];
    for (let z = 0; z < groundDepth(world); z++) {
      if (z % 4 === 0) await pace?.();
      for (let x = 0; x < world.width; x++) {
        const i = z * world.width + x;
        if (onPavement(x + 0.5, z + 0.5)) continue;
        // Tufts are a few thin blades each: a dense, soft lawn.
        const n = rand() < 0.5 ? 2 : 1;
        for (let k = 0; k < n; k++) {
          tufts.push(scatter(x, z, rand, 0.7 + rand() * 0.6));
          tuftTiles.push(i);
        }
        if (rand() < 0.06) {
          flowers.push(scatter(x, z, rand, 0.8 + rand() * 0.5));
          flowerTiles.push(i);
        }
      }
    }
    // Lawn detail around the town too, thinning out with distance.
    for (let n = 0; n < 5000; n++) {
      if (n % 1000 === 0) await pace?.();
      const x = -APRON - 30 + rand() * (world.width + APRON * 2 + 60);
      const z = -APRON - 30 + rand() * (groundDepth(world) + APRON * 2 + 60);
      const dist = this.distanceToTown(x, z);
      if (dist === 0 || rand() < dist / 40 || this.waterEdge(x, z) > -BEACH * 0.6) continue;
      const road = this.roads.near(x, z);
      if (road && road.dist < road.width / 2 + 0.3) continue;
      const m = scatter(x - 0.5, z - 0.5, rand, 0.8 + rand() * 0.7, this.heightAt(x, z));
      if (rand() < 0.1) {
        flowers.push(m);
        flowerTiles.push(-1);
      } else {
        tufts.push(m);
        tuftTiles.push(-1);
      }
    }
    // A lighter lawn keeps every so many of them, the same ones each time (the placement above is unchanged).
    const d = this.lawnDensity;
    const thin = <T>(list: T[]) => (d >= 1 ? list : list.filter((_, k) => Math.floor((k + 1) * d) > Math.floor(k * d)));
    // About `TUFT_CELL` squares over the town; the tufts scattered round it join the squares at its edge
    // (on their own they would be draws of a few tufts each).
    const [tw, td] = [world.width, groundDepth(world)];
    const [nx, nz] = [Math.max(1, Math.round(tw / TUFT_CELL)), Math.max(1, Math.round(td / TUFT_CELL))];
    const square = (x: number, z: number) => `${Math.min(nx - 1, Math.max(0, Math.floor((x / tw) * nx)))},${Math.min(nz - 1, Math.max(0, Math.floor((z / td) * nz)))}`;
    return [
      { key: 'model.grassTuft', matrices: thin(tufts), tiles: thin(tuftTiles), cell: square },
      // (Flowers are sparse: too few triangles for groups to pay.)
      { key: 'model.flowers', matrices: thin(flowers), tiles: thin(flowerTiles) },
    ];
  }
}

// ---- Garden dressing ------------------------------------------------------------------

export type Decor = [number, number, number, number][];

/** Clear distances for lot trees (trunk to edge, metres). */
const TREE_CLEAR = { street: 3.0, path: 1.3, house: 2.2, object: 1.0 };

/**
 * Moves a lot tree (by up to 2.5 m, staying on the viewed lot) until its trunk is clear of the
 * street (road, sidewalk, kerb furniture and streetlights all stand inside the street rects, see
 * street.ts), garden paths and driveways, the house and placed objects; null when no spot is.
 */
function clearTreeSpot(x: number, z: number, world: WorldStructure, view: { x: number; z: number; w: number; d: number } | null): [number, number] | null {
  const streets = world.meta?.streets ?? [];
  const paths = world.meta?.paths ?? [];
  const gap = (r: { x: number; z: number; w: number; d: number }, px: number, pz: number) =>
    Math.hypot(Math.max(r.x - px, 0, px - (r.x + r.w)), Math.max(r.z - pz, 0, pz - (r.z + r.d)));
  const indoorNear = (px: number, pz: number, r: number) => {
    for (let tz = Math.floor(pz - r); tz <= Math.floor(pz + r); tz++) {
      for (let tx = Math.floor(px - r); tx <= Math.floor(px + r); tx++) {
        if (tx < 0 || tz < 0 || tx >= world.width || tz >= world.depth || world.rooms[tz * world.width + tx] === 0) continue;
        if (gap({ x: tx, z: tz, w: 1, d: 1 }, px, pz) < r) return true;
      }
    }
    return false;
  };
  const ok = (px: number, pz: number) =>
    (!view || (px > view.x + 0.8 && px < view.x + view.w - 0.8 && pz > view.z + 0.8 && pz < view.z + view.d - 0.8)) &&
    streets.every((r) => gap(r, px, pz) >= TREE_CLEAR.street) &&
    paths.every((r) => gap(r, px, pz) >= TREE_CLEAR.path) &&
    world.objects.every((o) => gap(o, px, pz) >= TREE_CLEAR.object) &&
    !indoorNear(px, pz, TREE_CLEAR.house);
  for (let r = 0; r <= 2.5; r += 0.5) {
    const steps = r === 0 ? 1 : 12;
    for (let k = 0; k < steps; k++) {
      const a = (k / steps) * Math.PI * 2;
      const px = x + Math.cos(a) * r;
      const pz = z + Math.sin(a) * r;
      if (ok(px, pz)) return [px, pz];
    }
  }
  return null;
}

/** The garden dressing (`natureDecor`'s) as placed, for the birds. */
export function decorPlants(decor: readonly [string, Decor][]): PlantPlacement[] {
  return decor.flatMap(([key, items]) => items.map(([x, z, s, yaw]) => ({ key, x, y: 0, z, sx: s, sy: s, sz: s, yaw })));
}

/**
 * Splits the house builder's garden dressing (`model.tree` lot trees, `model.bush` foundation
 * shrubs; items are [x, z, scale, yaw]) into species. Lot trees mix oaks, maples, blossom, fruit,
 * birch, magnolia and Japanese maple; each house gets either a clipped hedge along its walls or a
 * mixed border of shrubs, hydrangeas, roses, lavender and clipped balls. Deterministic per position.
 */
export function natureDecor(key: string, items: Decor, world?: WorldStructure, view?: { x: number; z: number; w: number; d: number } | null): [string, Decor][] {
  const out = new Map<string, Decor>();
  const put = (k: string, item: [number, number, number, number]) => out.set(k, [...(out.get(k) ?? []), item]);
  if (key === 'model.tree') {
    for (const item of items) {
      const h = hash(Math.floor(item[0] * 7), Math.floor(item[1] * 7), 91);
      // Crowns are ~2 m in radius: keep trunks well off the sidewalk, its streetlights and props.
      const spot = world ? clearTreeSpot(item[0], item[1], world, view ?? null) : [item[0], item[1]];
      if (!spot) continue;
      const species =
        h < 0.24 ? 'model.tree' : h < 0.4 ? 'model.tree.maple' : h < 0.55 ? 'model.tree.blossom' : h < 0.68 ? 'model.tree.fruit' : h < 0.8 ? 'model.tree.birch' : h < 0.9 ? 'model.tree.magnolia' : 'model.tree.japaneseMaple';
      put(species, [spot[0], spot[1], item[2], item[3]]);
    }
  } else if (key === 'model.bush' && items.length) {
    // One border style per house (the first shrub's position stands in for the lot).
    const style = hash(Math.floor(items[0][0] / 8), Math.floor(items[0][1] / 8), 92);
    for (const item of items) {
      const [x, z, s] = item;
      const h = hash(Math.floor(x * 5), Math.floor(z * 5), 93);
      if (style < 0.4) {
        // Hedge segments run along the wall: shrubs sit at x + 0.5 (wall along x) or z + 0.5.
        const alongX = Math.abs(x - Math.floor(x) - 0.5) < 0.05;
        put('model.hedge', [x, z, 0.95 + (s - 0.55) * 0.2, alongX ? 0 : Math.PI / 2]);
      } else {
        // A mixed border: green shrubs with hydrangeas, roses, lavender and clipped balls among them.
        const kind =
          h < 0.3 ? 'model.bush' : h < 0.45 ? 'model.bush.flowering' : h < 0.55 ? 'model.bush.small' : h < 0.67 ? 'model.garden.hydrangea' : h < 0.78 ? 'model.garden.rosebush' : h < 0.89 ? 'model.garden.lavender' : 'model.garden.boxwood';
        put(kind, [x, z, s * (kind.startsWith('model.garden') ? 1.25 : 1.1), item[3]]);
      }
    }
  } else if (items.length) {
    out.set(key, items);
  }
  return [...out];
}

// ---- Wind ---------------------------------------------------------------------------

/**
 * The wind's bend, as the vertex shader draws it and `windSwayAt` follows it for what sits on a
 * branch: a sway of two sines along `dir`, in gusts, phased by where each plant stands, bending a
 * point by `bend` × its height in its model squared (m).
 */
export const WIND = {
  /** Which way it blows (x, z). */
  dir: [0.8, 0.6],
  /** Phase per metre (x, z), so neighbours sway out of step. */
  phase: [0.21, 0.17],
  /** Gusts: `base + amp · sin(t · rate + phase · ph)`. */
  gust: { base: 0.6, amp: 0.4, rate: 0.37, phase: 0.5 },
  /** The sway: `Σ weight · sin(t · rate + phase · ph)`. */
  sway: [
    { weight: 0.7, rate: 1.1, phase: 1 },
    { weight: 0.3, rate: 2.3, phase: 1.7 },
  ],
  /** Bend (m per m² of height, per unit of sway). */
  bend: 0.0022,
} as const;

/**
 * Per finish (primitive parts' `material`): x bend (per m^2 of height), y flutter amplitude (m),
 * z flutter speed, w flutter ramp (1/m). Foliage also gets its per-instance tint.
 */
export const WIND_KIND: Record<string, [number, number, number, number]> = {
  'material.finish.bark': [WIND.bend, 0, 0, 0],
  'material.finish.foliage': [WIND.bend, 0.012, 3.1, 1.5],
};

/**
 * How far the wind bends a plant standing at (`px`, `pz`) at wind time `t` and `strength`, per m²
 * of height up its model (m, along `windDirection`): a point `h` m up (before the plant's scale)
 * is where the vertex shader draws it (the bend, not the leaves' flutter) this times `h²` away, so
 * what sits on a branch sways with it.
 */
export function windSwayAt(t: number, strength: number, px: number, pz: number): number {
  const ph = px * WIND.phase[0] + pz * WIND.phase[1];
  const g = WIND.gust;
  const gust = (g.base + g.amp * Math.sin(t * g.rate + ph * g.phase)) * strength;
  let sway = 0;
  for (const s of WIND.sway) sway += Math.sin(t * s.rate + ph * s.phase) * s.weight;
  return sway * gust * WIND.bend;
}

/**
 * The way (x, z) the wind bends a plant instance turned `yaw` and scaled `sx` across and `sz` deep:
 * the wind's direction in the world, but for a plant scaled unevenly (a hedge), as the shader bends
 * it in the model's own space (its `lw`).
 */
export function windDirection(yaw: number, sx: number, sz: number, out: { x: number; z: number }): void {
  // The instance's x and z axes in the world (`Quaternion.RotationAxis(Up, yaw)`), and the wind along each.
  const [c, s] = [Math.cos(yaw), Math.sin(yaw)];
  const [dx, dz] = WIND.dir;
  const along = dx * c - dz * s;
  const across = (dx * s + dz * c) * (sz / sx) ** 2;
  out.x = along * c + across * s;
  out.z = -along * s + across * c;
}
