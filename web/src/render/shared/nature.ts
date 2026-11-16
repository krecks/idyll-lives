/**
 * The engine-neutral half of the landscape round the town (`pc/nature.ts` draws
 * it): the land's shape (rolling hills, a valley's rim, mountains, fields, a sea or a lake, levelled
 * under the country roads), the terrain's colours, where the woods and the lawn's flowers grow, the
 * lawn hidden indoors, the garden dressing split into species, and the wind's numbers as the
 * plant shaders sway them. Everything is generated deterministically from the town's seed, so a
 * loaded game looks the same, with the same plants in the same order.
 *
 * Placements are matrices in game space (the layout of `math.ts`): 16 floats each.
 */

import { DEFAULT_LANDSCAPE, groundDepth, type LandFile, type LandscapeParams, type WorldStructure, type WorldZone } from '../../core/protocol';
import { COVER, COVER_CELL, decodeLand, heightAt as landHeight, LAND_CELL, type Land, type LandRect } from '../../game/land';
import { coverAt, landTop, landWater, wetDistance, wetEdgeAt } from './landBake';
import { CountryRoads, ROAD_VERGE, type RoadLand } from './countryRoads';
import { roadIndex, roadTiles, townExits, townRoads, type TownExit } from './roads';
import { compose, quatAxisAngle } from './math';
import { fbm, hash, mulberry32, ridged, smoothstep, valueNoise } from '../../game/noise';
import { tuned } from '../tune';
import { shareAt, ZONE_CELL, ZoneField, type ZoneKind } from './zones';

/**
 * How far the landscape extends beyond the town (metres): towns of older code, and worlds (which
 * reach further, to a farther horizon). The valley's rim rises over `RING` either way.
 */
export const RING = 150;
const WORLD_RING = 260;
/** How far (m) `world`'s landscape reaches past its land's edge (the layout's own ring, known before one is made). */
export function landscapeRing(world: { meta?: { world?: unknown } | null }): number {
  return world.meta?.world ? WORLD_RING : RING;
}
/** Flat apron around the town before the hills start. */
export const APRON = 6;
/** A big world's distance-from-built grid (m a cell; `distanceToTown`). */
const BUILT_CELL = 2;
/** A big world's woods are drawn in squares this wide (m), those far off left out (`pc/nature.ts` `frameTiles`); the preset's own (`OpenWorldQuality.woodsCell`) where it gives one. */
export const BIG_WOODS_CELL = 256;
/** A big world's lawn flowers are drawn in squares this wide (m), those far off left out (`pc/nature.ts` `frameTiles`). */
export const BIG_FLOWERS_CELL = 128;
/** A big world's orchards: fruit trees in rows this far apart (m), and this far apart along a row. */
const ORCHARD_ROWS = 6;
const ORCHARD_STEP = 5;
/** A big world's hill zones rise this high (m) at most (between what is built: the land there stays flat). */
const BIG_HILLS = 16;
const TERRAIN_STEP = 2.5;
/** A big world's ground tiles (m; `pc/nature.ts` `terrainTiles`), laid from the terrain's corner. */
export const GROUND_TILE = 128;
/** At most this many terrain quads across (a wider landscape gets coarser ones). */
const TERRAIN_SEGMENTS = 260;
/** Trees within this distance of the town cast shadows; further ones don't. */
const SHADOW_RANGE = 28;
/**
 * A land drawn from its height field (`LandscapeLayout.setLand`): past the world's edge the landscape's own land,
 * moved by how far the field's edge stands off it there, the move fading out over this far (m).
 */
const LAND_EDGE_FADE = 60;
/** A height field whose top stands over this (m) has rock, snow and a treeline on it, as mountains do. */
const LAND_PEAKS = 40;
/**
 * A land drawn from its height field grows its woods and flowers in squares this wide (m), each from a stream of
 * its own (`mulberry32(hash(bx, bz, seed))`): a stroke of the brushes grows only the squares it reached again.
 */
export const LAND_BLOCK = 32;
/** A square's woods on a grid of this many points across (`LAND_BLOCK` / 8 = 4 m apart, not the 4.2 of one stream: as dense, less this share). */
const BLOCK_GRID = 8;
/** The cover's kinds as zones (`COVER`). */
const COVER_ZONES: readonly [ZoneKind, number][] = [
  ['woods', COVER.woods],
  ['meadow', COVER.meadow],
  ['fields', COVER.fields],
];
/** Mountains: no trees above about this height (m). */
const TREELINE = 12;
/** Mountains: bare rock and then snow from these shares of their height up to these (a relief grid's: higher up, its massif is all of it). */
const ROCK_SNOW: [[number, number], [number, number]] = [
  [0.16, 0.28],
  [0.33, 0.45],
];
const RELIEF_ROCK_SNOW: [[number, number], [number, number]] = [
  [0.45, 0.6],
  [0.76, 0.92],
];
/** A world with a relief grid among mountains: no trees above this share of the mountains' height. */
const RELIEF_TREELINE = 0.3;
/** How smoothly (m) a relief grid's land meets the most it may rise from what is built (`smoothMin`). */
const RELIEF_BLEND = 6;
/** A relief grid's land rises at most this steeply from what is built where the world doesn't say (`LandscapeParams.slope`). */
const RELIEF_SLOPE = 0.5;
/** The mountains past a relief world's edge rise over this far (m) out from it. */
const EDGE_MOUNTAINS: [number, number] = [10, 150];
/** The random draws a lawn tuft took (its scale, turn and spot), still taken so the flowers stand where they did. */
const TUFT_DRAWS = 4;
/** No lawn this close to a road's edge (m): the grass's blades lean, and would reach over it. */
const LAWN_CLEAR = 0.6;
/** Height of the sea or a lake (m); the shore is where the land meets it. */
export const WATER_Y = -0.35;
/** Sand from the shoreline up the bank (m). */
const BEACH = 8;
/**
 * In the game, plants are drawn in groups the camera leaves out when they're out of view (a model
 * drawn as one mesh over the whole landscape never is): the woods in this many sectors round the
 * town. A model part with fewer triangles in all than `SPLIT_TRIANGLES` (bark, flowers) stays
 * one mesh: the draw calls more would cost the main thread more than the triangles they save. Each
 * group is a mesh of its own, and every mesh costs the main thread a little each frame whether
 * drawn or not, so the sectors are wide. A sector reaches from the town to the horizon, though, and
 * one in view draws all of it: in the wide view about fourteen times the far trees on screen. Low draws
 * the woods in 100 m squares instead (`QualitySettings.woodsCell`), more meshes that leave out more
 * trees. The menus' town, which has the whole landscape in view, draws each part whole.
 */
const FOREST_SECTORS = 8;
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

// (The seeded generator, lattice hash, value noise, fractal noise, ridged noise and smooth step: the world generator's own, `game/noise.ts`, so the land it lays out and the land drawn agree.)
export { fbm, hash, mulberry32, ridged, smoothstep, valueNoise };

/**
 * The smaller of `a` and `b` (both 0 or more), rounded off where they come within `k` of each other (a
 * polynomial smooth minimum), and never below half the smaller: two that are both 0 stay 0.
 */
export function smoothMin(a: number, b: number, k: number): number {
  const m = Math.min(a, b);
  const t = Math.max(k - Math.abs(a - b), 0) / k;
  return m - Math.min(t * t * k * 0.25, m * 0.5);
}

// ---- Colours ------------------------------------------------------------------------

/** A colour, 0..1 per channel. */
interface Rgb {
  r: number;
  g: number;
  b: number;
}

/** An sRGB hex colour (`#rrggbb`) as 0..1 channels (each byte / 255). */
function hex(c: string): Rgb {
  return { r: parseInt(c.substring(1, 3), 16) / 255, g: parseInt(c.substring(3, 5), 16) / 255, b: parseInt(c.substring(5, 7), 16) / 255 };
}

/** The colour `t` of the way from `a` to `b`, per channel (`out` may be `a`). */
function lerp(a: Rgb, b: Rgb, t: number, out: Rgb): void {
  out.r = a.r + (b.r - a.r) * t;
  out.g = a.g + (b.g - a.g) * t;
  out.b = a.b + (b.b - a.b) * t;
}

/** The terrain's colours: a world's four greens (`LandscapeParams.grass`), and the rest. */
function terrainPalette(greens: readonly string[]) {
  const [lush, grass, meadow, hill] = greens.map(hex);
  return {
    lush,
    grass,
    meadow,
    hill,
    dirt: hex('#7C6748'),
    sand: hex('#D8C89A'),
    wetSand: hex('#A99A72'),
    rock: hex('#7A7064'),
    snow: hex('#F1F3F4'),
    alpine: hex('#8FA266'),
    crops: ['#C9B45C', '#7E6142', '#7A9C44', '#B5A35A', '#6C8A3C'].map(hex),
    furrow: hex('#5F4733'),
  };
}
type TerrainPalette = ReturnType<typeof terrainPalette>;

// ---- Placements ---------------------------------------------------------------------

/** A placement's matrix: scaled, turned `yaw` about up, at (x, y, z) (`compose`, `math.ts`). */
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
  /** Which group a point is in, the game drawing the woods by group: one of the `FOREST_SECTORS` round the town, or a square (`woodsCell`). */
  sector: (x: number, z: number) => string;
}

/** The lawn's flowers as placed (`LandscapeLayout.lawns`): each model's matrices, and the town tile under each (-1 outside it). */
export interface LawnPlacement {
  key: string;
  matrices: Float32Array[];
  tiles: number[];
  /** Which group an instance at (x, z) is drawn in (a big world's: squares the game leaves out past a distance); none: one draw. */
  cell?: (x: number, z: number) => string;
  /** They keep a tile's distance from what hides the lawn (`LawnLayer.clear`). */
  clear?: boolean;
}

/** Lawn flowers as drawn: each instance's original matrix and town tile (-1 outside the town), its matrix as drawn (`shown`) and whether it's hidden. */
export interface LawnLayer<D> {
  /** What draws them (each engine's own). */
  draws: D[];
  base: Float32Array;
  shown: Float32Array;
  tiles: Int32Array;
  hidden: Uint8Array;
  /**
   * They are hidden on the tiles next to a hidden one too (a house's walls, a thing in the garden): a patch of
   * flowers is a tile wide and spills over its tile's edge, onto a doorstep or against a wall's foot.
   */
  clear?: boolean;
}

/** Where the lawn is hidden (`refreshLawn`): per town tile, and the split tiles' diagonals. */
export interface LawnMask {
  width: number;
  /** 1 indoors or under furniture; on a tile split by a diagonal wall, 2 or 3: only its half 0 or 1 indoors. */
  hidden: Uint8Array;
  splitAxis: Map<number, 'dp' | 'dn'>;
}

/** Which half of a tile split by a diagonal wall (`axis`) a point (`fx`, `fz` within the tile) is in: a 'dp' wall's half 0 is fz < fx, a 'dn' wall's fx + fz < 1. */
export function splitHalf(axis: 'dp' | 'dn', fx: number, fz: number): 0 | 1 {
  return axis === 'dp' ? (fz < fx ? 0 : 1) : fx + fz < 1 ? 0 : 1;
}

/**
 * Per tile of the town's ground, where no lawn grows now: `lawnMask`'s hidden tiles (2 or 3 on a
 * split tile: only that half), and 1 on its streets, paths and fields (`unlawned`) and its patios
 * (floor coverings outdoors). The grass's mask (`shared/grass.ts`) and the birds' lawn read it.
 */
export function bareTiles(world: WorldStructure, mask?: LawnMask): Uint8Array {
  // (Asked again for a world whose lawn is as it was, the birds' after a focus lot shown: the same answer.)
  if (!mask) {
    const inputs = lawnInputs(world);
    if (lastBare && sameLawnInputs(lastBare.inputs, inputs)) return lastBare.bare;
    const bare = bareTilesOf(world, lawnMask(world));
    lastBare = { inputs, bare };
    return bare;
  }
  return bareTilesOf(world, mask);
}
let lastBare: { inputs: LawnInputs; bare: Uint8Array } | null = null;

/**
 * `bareTiles` for a world whose lawn inputs (`lawnInputs`) and mask the caller has made already (the
 * landscape's `refreshLawn`): kept as the answer `bareTiles` gives for those inputs (the birds' next).
 */
export function keepBareTiles(world: WorldStructure, inputs: LawnInputs, mask: LawnMask): Uint8Array {
  const bare = bareTilesOf(world, mask);
  lastBare = { inputs, bare };
  return bare;
}

function bareTilesOf(world: WorldStructure, mask: LawnMask): Uint8Array {
  const w = world.width;
  const d = groundDepth(world);
  const bare = mask.hidden.slice(0, w * d);
  const paved = unlawned(world);
  for (let i = 0; i < bare.length; i++) if (paved[i]) bare[i] = 1;
  for (const [x, z] of world.floors ?? []) if (x >= 0 && z >= 0 && x < w && z < d) bare[z * w + x] = 1;
  return bare;
}

/**
 * Per tile of the town's ground (1), where no lawn grows at all: its roads (or streets), paths and
 * fields (the birds forage on the lawn). `any`: the tiles any of them reaches into; else only those
 * whose middle they cover.
 */
export function unlawned(world: WorldStructure, any = true): Uint8Array {
  const w = world.width;
  const d = groundDepth(world);
  const out = world.meta?.roads ? roadTiles(world.meta.roads, w, d).slice() : new Uint8Array(w * d);
  const span = (a: number, len: number, n: number): [number, number] =>
    any ? [Math.max(0, Math.floor(a)), Math.min(n, Math.ceil(a + len))] : [Math.max(0, Math.ceil(a - 0.5)), Math.min(n, Math.ceil(a + len - 0.5))];
  for (const r of [...(world.meta?.streets ?? []), ...(world.meta?.paths ?? []), ...(world.meta?.world?.fields ?? [])]) {
    const [[x0, x1], [z0, z1]] = [span(r.x, r.w, w), span(r.z, r.d, d)];
    for (let z = z0; z < z1; z++) out.fill(1, z * w + x0, z * w + Math.max(x0, x1));
  }
  return out;
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
 * Hides the flowers of `layers` on the tiles `mask` hides them on (`refreshLawn`), and shows the
 * rest; `upload` is called with each layer whose matrices changed. On a tile split by a diagonal
 * wall, each follows the half it stands in.
 */
export function hideLawn<D>(layers: readonly LawnLayer<D>[], { width, hidden, splitAxis }: LawnMask, upload: (layer: LawnLayer<D>) => void): void {
  // (For the layers that keep clear: whether a tile or one of the eight round it is hidden, at all.)
  let near: Uint8Array | undefined;
  const nearHidden = (): Uint8Array => {
    if (near) return near;
    near = new Uint8Array(hidden.length);
    const depth = Math.floor(hidden.length / width);
    for (let i = 0; i < hidden.length; i++) {
      if (!hidden[i]) continue;
      const [x, z] = [i % width, Math.floor(i / width)];
      for (let dz = -1; dz <= 1; dz++) for (let dx = -1; dx <= 1; dx++) if (x + dx >= 0 && x + dx < width && z + dz >= 0 && z + dz < depth) near[(z + dz) * width + x + dx] = 1;
    }
    return near;
  };
  for (const layer of layers) {
    let changed = false;
    for (let k = 0; k < layer.tiles.length; k++) {
      const t = layer.tiles[k];
      let hide = t >= 0 ? (layer.clear ? nearHidden()[t] : hidden[t]) : 0;
      if (hide > 1) {
        // The flower's position within its tile decides its half.
        const fx = layer.base[k * 16 + 12] - (t % width);
        const fz = layer.base[k * 16 + 14] - Math.floor(t / width);
        const axis = splitAxis.get(t) ?? 'dp';
        const half = splitHalf(axis, fx, fz);
        // Flowers right at the wall would poke through it.
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
export function lawnLayer<D>(draws: D[], matrices: readonly ArrayLike<number>[], tiles: readonly number[], indices: readonly number[], clear = false): LawnLayer<D> {
  const base = new Float32Array(indices.length * 16);
  indices.forEach((k, i) => base.set(matrices[k], i * 16));
  return { draws, base, shown: new Float32Array(base), tiles: Int32Array.from(indices, (k) => tiles[k]), hidden: new Uint8Array(indices.length), ...(clear ? { clear } : {}) };
}

/** A model's copies grouped by `group` (each group's indices into `items`, in their order): the woods' and the street's draw groups. */
export function groupInstances<T, K>(items: readonly T[], group: (item: T) => K): Map<K, number[]> {
  const groups = new Map<K, number[]>();
  items.forEach((it, i) => {
    const k = group(it);
    const list = groups.get(k);
    if (list) list.push(i);
    else groups.set(k, [i]);
  });
  return groups;
}

// ---- Landscape ----------------------------------------------------------------------

/**
 * The land round a town and what grows on it, as the renderer builds it (`pc/nature.ts`'s `Landscape`
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
  /** How far the sand reaches from the shoreline (m; `LandscapeParams.beach`). */
  private readonly beach: number;
  /**
   * A big world's relief grid (`WorldMeta.relief`) in metres over the built level, and how steeply the land
   * may rise from what is built (`LandscapeParams.slope`). Null: none (older saves, the classic specs).
   */
  private readonly relief: { cell: number; cols: number; rows: number; h: Float32Array; slope: number } | null;
  /** Where the roads leave town (the mountains part there). */
  private readonly passes: TownExit[];
  /** The roads out of town (the ground is levelled under them). */
  readonly roads: R;
  /** The woods' trees and bushes near the town (full models), where birds may live (`woodsNear`). */
  private nearWoods: PlantPlacement[] = [];
  /** `woodsNear`'s last answer, and where it was asked (the same array while that's unchanged). */
  private woodsAsked: { at: string; plants: readonly PlantPlacement[] } | null = null;

  /**
   * `lawnDensity`: the share of the lawn's flowers drawn (Low: half, and none of the patches in their drifts); `lawnLod` 1 draws
   * them in their far versions (Low, as the street's models: `QualitySettings.streetLod`); `woodsCell`:
   * the side (m) of the squares the game draws the woods in (0: the `FOREST_SECTORS`;
   * `QualitySettings.woodsCell`); `bigWoodsCell`: a big world's (`OpenWorldQuality.woodsCell`).
   */
  constructor(
    world: WorldStructure,
    makeRoads: (land: RoadLand, world: WorldStructure, name: string) => R,
    protected readonly lawnDensity = 1,
    protected readonly lawnLod: 0 | 1 = 0,
    protected readonly woodsCell = 0,
    protected readonly bigWoodsCell = BIG_WOODS_CELL,
  ) {
    this.seed = world.meta?.seed ?? 1;
    this.w = world.width;
    this.d = groundDepth(world);
    this.land = { ...DEFAULT_LANDSCAPE, ...world.meta?.world?.landscape };
    this.ring = landscapeRing(world);
    this.beach = this.land.beach ?? BEACH;
    this.built = world.meta?.world?.size === 'big' ? LandscapeLayout.builtField(world, this.w, this.d) : null;
    const relief = world.meta?.relief;
    this.relief =
      this.built && relief && relief.cell > 0 && relief.cols > 0 && relief.rows > 0 && relief.h.length >= relief.cols * relief.rows
        ? { cell: relief.cell, cols: relief.cols, rows: relief.rows, h: Float32Array.from(relief.h, (v) => v / 10), slope: this.land.slope ?? RELIEF_SLOPE }
        : null;
    this.zones = this.built && world.meta?.zones?.length ? new ZoneField(world.meta.zones, this.w, this.d) : null;
    this.zonesToday = this.zones;
    this.zoneList = world.meta?.zones ?? [];
    this.drawnWatered = !!this.land.water || !!this.zones?.has('water');
    this.orchards = this.built ? this.zoneList.filter((z) => z.kind === 'orchard') : [];
    // (A world with a height field is drawn from it from the start, its roads laid on it.)
    this.metaLand = world.meta?.land;
    if (this.metaLand) this.adoptLand(decodeLand(this.metaLand));
    this.passes = townExits(world);
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

  /** Whether there is a sea or a lake (a big world's: water zones; with a height field, its water too). */
  get watered(): boolean {
    return this.drawnWatered || (!!this.field && this.fieldWater.any);
  }

  /** Whether the water sheet at the sea's level is drawn: the landscape's own water, or a height field's at that level. */
  get seaSheet(): boolean {
    return this.drawnWatered || (!!this.field && this.fieldWater.sea);
  }

  /** The height field the land is drawn from (`setLand`), or null. */
  get heightField(): Land | null {
    return this.field;
  }

  /**
   * Draws the land from `land` from now on (kept: the brushes change it in place, then `landChanged`); null: from
   * the world's own (`meta.land`) if it has one, else from the landscape. What is already built isn't drawn again
   * here (the engine's side does that).
   */
  setLand(land: Land | null): void {
    this.field = null;
    this.wetSd = null;
    this.coverSeen = null;
    this.zones = this.zonesToday;
    if (land) this.adoptLand(land);
    else if (this.metaLand) this.adoptLand(decodeLand(this.metaLand));
    this.landVersion++;
  }

  /**
   * The samples in `rect` of the height field changed (a stroke done): its water's edge, its highest point and the
   * zones of the cover kinds that changed made again.
   */
  landChanged(rect: LandRect): void {
    if (!this.field) return;
    this.measureLand(rect);
    this.landVersion++;
  }

  /** A height field taken on (`setLand`): its water, top and zones. */
  private adoptLand(land: Land): void {
    this.field = land;
    this.coverSeen = null;
    this.measureLand(null);
  }

  /** The height field's water, top and (a big world's) zones from its cover: the cover kinds changed in `rect`, or all. */
  private measureLand(rect: LandRect | null): void {
    const land = this.field!;
    this.wetSd = wetDistance(land);
    this.fieldWater = landWater(land);
    this.fieldTop = landTop(land);
    if (!this.built) return;
    const seen = this.coverSeen;
    const changed = new Set<number>();
    if (seen && seen.length === land.cover.length && rect) {
      const cw = Math.ceil((land.cols * LAND_CELL) / COVER_CELL);
      const cd = land.cover.length / cw;
      const [ci0, cj0] = [Math.max(0, Math.floor((rect.i0 * LAND_CELL) / COVER_CELL) - 1), Math.max(0, Math.floor((rect.j0 * LAND_CELL) / COVER_CELL) - 1)];
      const [ci1, cj1] = [Math.min(cw, Math.ceil((rect.i1 * LAND_CELL) / COVER_CELL) + 1), Math.min(cd, Math.ceil((rect.j1 * LAND_CELL) / COVER_CELL) + 1)];
      for (let j = cj0; j < cj1; j++) {
        for (let i = ci0; i < ci1; i++) {
          const k = j * cw + i;
          if (seen[k] === land.cover[k]) continue;
          changed.add(seen[k]);
          changed.add(land.cover[k]);
        }
      }
      if (!changed.size) return;
    }
    this.coverSeen = land.cover.slice();
    const masks = new Map<ZoneKind, Uint8Array>();
    const gw = Math.max(1, Math.ceil(this.w / ZONE_CELL));
    const gd = Math.max(1, Math.ceil(this.d / ZONE_CELL));
    for (const [kind, value] of COVER_ZONES) {
      if (seen && rect && !changed.has(value) && this.zones !== this.zonesToday) continue;
      const mask = new Uint8Array(gw * gd);
      for (let j = 0; j < gd; j++) for (let i = 0; i < gw; i++) if (coverAt(land, (i + 0.5) * ZONE_CELL, (j + 0.5) * ZONE_CELL) === value) mask[j * gw + i] = 1;
      masks.set(kind, mask);
    }
    if (this.zones && this.zones !== this.zonesToday) for (const [kind, mask] of masks) this.zones.setMask(kind, mask);
    else this.zones = new ZoneField(this.zoneList, this.w, this.d, masks);
  }

  /** What covers the ground at (x, z) (`COVER`): the height field's cover, else the zones' (fields, then woods, then meadows). */
  coverAt(x: number, z: number): number {
    if (this.field) return coverAt(this.field, x, z);
    const zones = this.zonesToday;
    if (!zones) return COVER.open;
    if (zones.edge('fields', x, z) > 0) return COVER.fields;
    if (zones.edge('woods', x, z) > 0) return COVER.woods;
    if (zones.edge('meadow', x, z) > 0) return COVER.meadow;
    return COVER.open;
  }

  /** The height the rock, the snow and the treeline go by: the mountains', or a height field's top where it stands over `LAND_PEAKS`. */
  private get peaks(): number {
    const top = this.land.mountains ?? 0;
    return this.field && this.fieldTop > LAND_PEAKS ? Math.max(top, this.fieldTop) : top;
  }

  /**
   * Identity of a town's landscape (built once per town and kept while it is shown). A world with a height field
   * (`meta.land`) is told by `:L`, not by its zones or its field's contents: those change with every stroke of the
   * brushes (a new `meta` each, its zones derived from the field), drawn in place (`landChanged`).
   */
  static key(world: WorldStructure): string {
    return world.meta?.land ? `${LandscapeLayout.baseKey(world)}:L` : `${LandscapeLayout.baseKey(world)}${world.meta?.zones ? `:z${world.meta.zones.length}` : ''}`;
  }

  /** `key` without the zones and the height field: a landscape drawn from a field (`setLand`) is still the one for its world once that world carries the field (`LotContents.useNature`). */
  static baseKey(world: WorldStructure): string {
    const look = world.meta?.world;
    const streets = [...(world.meta?.streets ?? []).map((s) => `${s.z}`), ...(world.meta?.roads ?? []).map((r) => `${r.points[0]}:${r.points.length}`)].join(',');
    // (A backdrop is built land too, `builtField`: a town with one has a landscape of its own.)
    const backdrop = world.meta?.backdrop?.length ? `:b${world.meta.backdrop.length}` : '';
    // (The options, the zones and the relief too: a world made again with only its woods or its hills changed
    // keeps its seed, name and roads, and must not keep the landscape it had.)
    const options = look?.options ? `:o${JSON.stringify(look.options)}` : '';
    const relief = world.meta?.relief ? `:r${reliefSum(world.meta.relief)}` : '';
    return `${world.meta?.seed ?? 1}:${world.width}x${groundDepth(world)}:${world.meta?.name ?? ''}:${streets}${look ? `:${look.kind}:${JSON.stringify(look.landscape)}` : ''}${backdrop}${options}${relief}`;
  }

  /**
   * Metres past the shoreline of the sea or the lake (above 0 in the water, below 0 on land);
   * -Infinity without water. The shore wanders a little but never comes into the town.
   */
  waterEdge(x: number, z: number): number {
    // (A height field's: the edge of its water.)
    return this.field ? wetEdgeAt(this.field, this.wetSd, x, z) : this.drawnWaterEdge(x, z);
  }

  /** `waterEdge` as the landscape lays it out (without a height field). */
  private drawnWaterEdge(x: number, z: number): number {
    const water = this.land.water;
    // (A big world's water is its zones'.)
    if (!water) return this.zonesToday ? this.zonesToday.edge('water', x, z) - (fbm(x * 0.03, z * 0.03, this.seed + 61, 3) - 0.45) * 6 : -Infinity;
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
    if (!this.built) return Math.hypot(dx, dz);
    // A big world: from what is built (its lots, roads and trails), the land between rising as round a town.
    const b = this.built;
    const cx = Math.max(0, Math.min(b.gw - 1, Math.floor(Math.min(this.w, Math.max(0, x)) / BUILT_CELL)));
    const cz = Math.max(0, Math.min(b.gd - 1, Math.floor(Math.min(this.d, Math.max(0, z)) / BUILT_CELL)));
    return Math.hypot(dx, dz) + b.dist[cz * b.gw + cx];
  }

  /**
   * A big world's distance (m) from anything built, on a `BUILT_CELL` m grid over it: its lots, and
   * its roads and trails with a verge, so every hamlet gets the flat apron a town has and the land
   * rises between them (two chamfer passes, ~2 % off the true distance). Null: not a big world.
   */
  protected readonly built: { gw: number; gd: number; dist: Float32Array } | null;

  /**
   * A big world's zones (`meta.zones`): its water, its hills, and where woods, meadows, fields and orchards are. Null
   * elsewhere. With a height field (`setLand`), its woods, meadows and fields are the field's cover.
   */
  protected zones: ZoneField | null;
  /** The zones as `meta.zones` has them (the land as the landscape lays it out: past the edge of a height field's). */
  private readonly zonesToday: ZoneField | null;
  /** The world's zones as given (`meta.zones`). */
  private readonly zoneList: readonly WorldZone[];
  /** Whether the landscape has water of its own (a sea or a lake; a big world's water zones): `watered` without a height field. */
  private readonly drawnWatered: boolean;

  /**
   * The land as a height field (`game/land.ts`), once there is one: the world's `meta.land`, or given (`setLand`;
   * the brushes change it in place, `landChanged`). The land is drawn from it inside the world. Null: from the landscape.
   */
  protected field: Land | null = null;
  /** The world's own height field (`meta.land`), what `setLand(null)` goes back to. */
  private readonly metaLand: LandFile | undefined;
  /** The field's water: the signed distance from its edge (`wetDistance`), whether it has any and any at the sea's level, and its highest sample. */
  private wetSd: Float32Array | null = null;
  private fieldWater = { any: false, sea: false };
  private fieldTop = 0;
  /** The field's cover as its zones were last made from it (`landChanged`: only the kinds that changed again). */
  private coverSeen: Uint8Array | null = null;
  /** Counts the field's changes (the town view's ground map is made again for a new one). */
  landVersion = 0;
  /** A big world's orchards (`meta.zones`), each a rectangle. */
  private readonly orchards: readonly WorldZone[];

  private static builtField(world: WorldStructure, w: number, d: number): { gw: number; gd: number; dist: Float32Array } {
    const gw = Math.ceil(w / BUILT_CELL);
    const gd = Math.ceil(d / BUILT_CELL);
    const dist = new Float32Array(gw * gd).fill(1e9);
    const mark = (x0: number, z0: number, x1: number, z1: number) => {
      for (let cz = Math.max(0, Math.floor(z0 / BUILT_CELL)); cz <= Math.min(gd - 1, Math.floor(z1 / BUILT_CELL)); cz++) {
        for (let cx = Math.max(0, Math.floor(x0 / BUILT_CELL)); cx <= Math.min(gw - 1, Math.floor(x1 / BUILT_CELL)); cx++) dist[cz * gw + cx] = 0;
      }
    };
    for (const p of world.plots) mark(p.x, p.z, p.x + p.w, p.z + p.d);
    // (The living town's backdrop buildings too, and a metre round them: no tree grows in one.)
    for (const b of world.meta?.backdrop ?? []) {
      const xs = b.poly.map((q) => q[0]);
      const zs = b.poly.map((q) => q[1]);
      mark(Math.min(...xs) - 1, Math.min(...zs) - 1, Math.max(...xs) + 1, Math.max(...zs) + 1);
    }
    for (const r of world.meta?.roads ?? []) {
      const half = r.width / 2 + 2;
      for (let i = 0; i + 1 < r.points.length; i++) {
        const [[ax, az], [bx, bz]] = [r.points[i], r.points[i + 1]];
        const n = Math.max(1, Math.ceil(Math.hypot(bx - ax, bz - az) / BUILT_CELL));
        for (let k = 0; k <= n; k++) {
          const [x, z] = [ax + ((bx - ax) * k) / n, az + ((bz - az) * k) / n];
          mark(x - half, z - half, x + half, z + half);
        }
      }
    }
    const c = BUILT_CELL;
    const diag = c * Math.SQRT2;
    const relax = (i: number, j: number, cost: number) => {
      if (dist[j] + cost < dist[i]) dist[i] = dist[j] + cost;
    };
    for (let z = 0; z < gd; z++) {
      for (let x = 0; x < gw; x++) {
        const i = z * gw + x;
        if (x > 0) relax(i, i - 1, c);
        if (z > 0) {
          relax(i, i - gw, c);
          if (x > 0) relax(i, i - gw - 1, diag);
          if (x < gw - 1) relax(i, i - gw + 1, diag);
        }
      }
    }
    for (let z = gd - 1; z >= 0; z--) {
      for (let x = gw - 1; x >= 0; x--) {
        const i = z * gw + x;
        if (x < gw - 1) relax(i, i + 1, c);
        if (z < gd - 1) {
          relax(i, i + gw, c);
          if (x < gw - 1) relax(i, i + gw + 1, diag);
          if (x > 0) relax(i, i + gw - 1, diag);
        }
      }
    }
    return { gw, gd, dist };
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
    // (A world with a relief grid has its mountains in it: these rise past its edge, out to the horizon.)
    const ramp = this.relief ? smoothstep(EDGE_MOUNTAINS[0], EDGE_MOUNTAINS[1], Math.hypot(Math.max(0 - x, 0, x - this.w), Math.max(0 - z, 0, z - this.d))) : smoothstep(APRON + 10, APRON + 130, dist);
    if (ramp <= 0) return 0;
    const open = this.openness(z);
    // A pass where each road leaves town: lower along its way out (`past` the town, `aside` from that way).
    let pass = 1;
    for (const e of this.passes) {
      const past = (x - e.x) * e.dx + (z - e.z) * e.dz;
      if (past <= 0) continue;
      const aside = Math.abs((x - e.x) * e.dz - (z - e.z) * e.dx);
      pass = Math.min(pass, 1 - smoothstep(0, 50, past) * (1 - smoothstep(14, 70, aside)) * 0.8);
    }
    const crest = ridged(x * 0.0075, z * 0.0075, this.seed + 211);
    return top * ramp * open * pass * (0.25 + 0.75 * crest);
  }

  /**
   * Height of the land before the roads are cut in. With a height field (`setLand`): the field's inside the world;
   * past its edge the landscape's own, moved by how far the field's edge stands off it there, that fading out over
   * `LAND_EDGE_FADE` m (no shore bank of the landscape's: the field has its own).
   */
  ground(x: number, z: number): number {
    const field = this.field;
    if (!field) return this.drawnGround(x, z);
    if (x >= 0 && z >= 0 && x <= this.w && z <= this.d) return landHeight(field, x, z);
    const cx = Math.min(this.w, Math.max(0, x));
    const cz = Math.min(this.d, Math.max(0, z));
    const today = this.drawnGround(x, z);
    const fade = 1 - smoothstep(0, LAND_EDGE_FADE, Math.hypot(x - cx, z - cz));
    return fade > 0 ? today + (landHeight(field, cx, cz) - this.drawnGround(cx, cz)) * fade : today;
  }

  /** `ground` as the landscape lays it out (without a height field). */
  private drawnGround(x: number, z: number): number {
    const dist = this.distanceToTown(x, z);
    const L = this.land;
    const water = this.drawnWatered;
    let h = 0;
    if (dist > APRON) {
      const rise = smoothstep(APRON, APRON + L.rise, dist);
      // Among mountains the near side of the valley (towards the menus' camera) stays low.
      const open = L.mountains ? this.openness(z) : 1;
      const hills = (fbm(x * 0.018, z * 0.018, this.seed) * L.hills - (L.hills * 4) / 14) * (0.15 + 0.85 * open);
      // (A big world's rim rises round the world, past its edge; between its hamlets its hill zones do.)
      const rimFrom = this.zonesToday ? Math.hypot(Math.max(0 - x, 0, x - this.w), Math.max(0 - z, 0, z - this.d)) : dist - L.rimFrom;
      const rim = Math.pow(Math.max(0, rimFrom) / RING, 2) * L.rim * open;
      // By water the land stays above it (no puddles in the hollows).
      if (this.relief) {
        // (The mountains past the edge rise from the grid's land there, not on top of all of it: peaks of the
        // grid's height and the mountains' together would wall the world in.)
        const r = this.relieved(x, z, dist, open);
        h = Math.max(water ? 0 : -0.4, rise * hills + rim + r) + Math.max(0, this.mountain(x, z, dist) - 0.5 * r);
      } else {
        const zoned = this.zonesToday ? this.zonesToday.share('hill', x, z, 40) * BIG_HILLS * (0.55 + 0.9 * fbm(x * 0.008 + 5, z * 0.008, this.seed + 303, 3)) : 0;
        h = Math.max(water ? 0 : -0.4, rise * (hills + zoned) + rim) + this.mountain(x, z, dist);
      }
    }
    if (!water || dist === 0) return h;
    // Down a sandy bank to the shore, then under the water.
    const e = this.drawnWaterEdge(x, z);
    const beach = this.beach;
    if (e < -beach * 1.5) return h;
    if (e > 0) return WATER_Y - Math.min(4, e * 0.22 + e * e * 0.004);
    const bank = WATER_Y - e * 0.09;
    return h + (Math.min(h, bank) - h) * smoothstep(-beach * 1.5, -beach * 0.3, e);
  }

  /** The relief grid's height (m) at (x, z), read between the cells' middles, the edge's past the world's edge. */
  reliefAt(x: number, z: number): number {
    const R = this.relief;
    if (!R) return 0;
    const gx = Math.min(R.cols - 1, Math.max(0, x / R.cell - 0.5));
    const gz = Math.min(R.rows - 1, Math.max(0, z / R.cell - 0.5));
    const x0 = Math.floor(gx);
    const z0 = Math.floor(gz);
    const x1 = Math.min(R.cols - 1, x0 + 1);
    const z1 = Math.min(R.rows - 1, z0 + 1);
    const u = gx - x0;
    const v = gz - z0;
    const h = R.h;
    const a = h[z0 * R.cols + x0];
    const b = h[z0 * R.cols + x1];
    const c = h[z1 * R.cols + x0];
    const d = h[z1 * R.cols + x1];
    return (a + (b - a) * u) * (1 - v) + (c + (d - c) * u) * v;
  }

  /**
   * What a relief grid raises the land by at (x, z), `dist` m from what is built (`open`: `openness` among
   * mountains): the grid's height, roughened a little by noise (and among mountains by ridges above 20 m, more on
   * the valley's far side), and no more than the world's `slope` lets it rise from what is built, the two met
   * smoothly (`smoothMin`).
   */
  private relieved(x: number, z: number, dist: number, open: number): number {
    const r = this.reliefAt(x, z);
    if (r <= 0) return 0;
    let g = r * (0.85 + 0.3 * fbm(x * 0.012, z * 0.012, this.seed + 517, 3));
    if (this.land.mountains && g > 15) g += g * 0.35 * (ridged(x * 0.0075, z * 0.0075, this.seed + 211) - 0.5) * smoothstep(15, 25, g) * (0.4 + 0.6 * open);
    return smoothMin(g, this.relief!.slope * Math.max(0, dist - APRON), RELIEF_BLEND);
  }

  /** No trees above this (m) among mountains, give or take (`woods`), and the alpine meadow from about here. */
  private get treeTop(): number {
    const top = this.peaks;
    return this.relief && top ? top * RELIEF_TREELINE : TREELINE;
  }

  /** The terrain's square (m across, centred on the town) and its quads across. */
  protected terrainGrid(): { size: number; segments: number } {
    const size = Math.max(this.w, this.d) + this.ring * 2;
    return { size, segments: Math.min(TERRAIN_SEGMENTS, Math.round(size / TERRAIN_STEP)) };
  }

  /**
   * The sheet of the sea or the lake: its size, at the water line under the whole landscape. A big world's
   * ground is drawn in `GROUND_TILE` m tiles from the terrain's corner, the last ones reaching past its square:
   * the sheet reaches as far (else the sea round an island stops short of the ground's edge there, and the sea
   * floor shows past it).
   */
  protected waterSheet(): { size: number; x: number; y: number; z: number } {
    const size = Math.max(this.w, this.d) + this.ring * 2;
    if (!this.built) return { size, x: this.w / 2, y: WATER_Y, z: this.d / 2 };
    const tiled = Math.ceil(size / GROUND_TILE) * GROUND_TILE;
    return { size: tiled, x: this.w / 2 + (tiled - size) / 2, y: WATER_Y, z: this.d / 2 + (tiled - size) / 2 };
  }

  /**
   * The terrain's heights and colours: each vertex of `positions` (x, y, z; in place) lifted to the
   * land, and its colour (RGBA, sRGB, converted on the GPU; it multiplies the grass detail texture)
   * into `colors`, in alpha how much of it is lawn (the lawn's tone on the ground past the grass's
   * field, `GAME_GRASS_GROUND`).
   */
  protected async shadeTerrain(positions: Float32Array | number[], colors: Float32Array, pace?: () => Promise<void>): Promise<void> {
    const tmp: Rgb = { r: 0, g: 0, b: 0 };
    let seen = 0;
    for (let i = 0, c = 0; i < positions.length; i += 3, c += 4) {
      if (++seen % 4096 === 0) await pace?.();
      const x = positions[i];
      const z = positions[i + 2];
      const h = this.heightAt(x, z);
      positions[i + 1] = h;
      const lawn = this.terrainColour(x, z, h, tmp);
      colors[c] = tmp.r;
      colors[c + 1] = tmp.g;
      colors[c + 2] = tmp.b;
      colors[c + 3] = lawn;
    }
  }

  /** The terrain's palette (`terrainPalette`), made once. */
  private palette: TerrainPalette | null = null;

  /**
   * The terrain's colour at (x, z), `h` high, into `out` (sRGB; it multiplies the grass detail
   * texture), and how much of it is lawn, 0..1 (the grass grows there: not on worn patches,
   * fields, rock, snow, road verges or sand; `shared/grass.ts`).
   */
  terrainColour(x: number, z: number, h: number, out: Rgb): number {
    // Lush, slightly yellow-green lawn (the classic life-sim look); greener on the hills. Each
    // world has its own greens.
    const P = (this.palette ??= terrainPalette(this.land.grass));
    // (Asked first, right after the caller's `heightAt` here: the mountains' slope below asks other points, and `near` keeps only its last answer.)
    const road = this.roads.near(x, z);
    let lawn = 1;
    const patch = fbm(x * 0.06, z * 0.06, this.seed + 99, 3);
    const fine = fbm(x * 0.21, z * 0.21, this.seed + 31, 2);
    lerp(P.lush, P.grass, smoothstep(0.3, 0.55, fine), out);
    lerp(out, P.meadow, smoothstep(0.5, 0.75, patch), out);
    lerp(out, P.hill, smoothstep(1, 10, h), out);
    // A big world's meadows paler and flowery, its woods' floor the hills' deeper green.
    if (this.zones) {
      lerp(out, P.meadow, this.zones.share('meadow', x, z, 8) * 0.75, out);
      lerp(out, P.hill, this.zones.share('woods', x, z, 10) * 0.6, out);
    }
    // Worn, earthy patches: sparse, and more common on the flat ground near the town.
    const bare = smoothstep(0.66, 0.8, fbm(x * 0.09 + 13, z * 0.09 - 7, this.seed + 5, 3)) * (1 - smoothstep(2, 8, h));
    lerp(out, P.dirt, bare * 0.85, out);
    lawn *= 1 - bare;
    // Farmland: a patchwork of fields (wheat, ploughed earth, green crops) round the town.
    const cell = this.fieldCell(x, z);
    if (cell) {
      const crop = P.crops[Math.floor(cell.kind * P.crops.length)];
      lerp(out, crop, cell.weight * 0.9, out);
      // Ploughed and planted fields show their furrows.
      if (crop !== P.crops[0]) lerp(out, P.furrow, cell.weight * 0.35 * Math.max(0, Math.sin(cell.v * Math.PI * 1.4)), out);
      lawn *= 1 - cell.weight;
    }
    // Mountains: bare rock above the trees and on steep faces, snow on the tops.
    const top = this.peaks;
    if (top) {
      const steep = Math.hypot(this.heightAt(x + 1.5, z) - h, this.heightAt(x, z + 1.5) - h) / 1.5;
      // Pale alpine meadow above the trees, then rock, then snow.
      const line = this.treeTop;
      lerp(out, P.alpine, smoothstep(line * 0.8, line * 1.25, h + fine * 6) * 0.8, out);
      // (A relief grid's massif is the world's own, its whole height: its rock and snow begin higher up it.)
      const [rock, snow] = this.relief ? RELIEF_ROCK_SNOW : ROCK_SNOW;
      const rocky = Math.max(smoothstep(top * rock[0], top * rock[1], h + fine * 12), smoothstep(0.7, 1.2, steep) * smoothstep(8, 18, h)) * 0.92;
      lerp(out, P.rock, rocky, out);
      const snowy = smoothstep(top * snow[0], top * snow[1], h + patch * 20) * (1 - smoothstep(1.5, 2.6, steep) * 0.5);
      lerp(out, P.snow, snowy, out);
      lawn *= (1 - rocky) * (1 - snowy);
    }
    // A worn verge of dirt and gravel along the roads (no grass on them, nor leaning over their edge).
    if (road) {
      const verge = 1 - smoothstep(road.width / 2, road.width / 2 + 2.2, road.dist);
      lerp(out, P.dirt, 0.45 * verge, out);
      lawn *= 1 - verge;
      if (road.dist < road.width / 2 + LAWN_CLEAR) lawn = 0;
    }
    // The side roads joining the streets' ends, flat on the apron.
    if (this.roads.onLink(x, z, LAWN_CLEAR)) lawn = 0;
    // Sand along the water, darker where it is wet.
    const e = this.waterEdge(x, z);
    const beach = this.beach;
    if (e > -beach) {
      const sandy = smoothstep(-beach, -beach * 0.45, e);
      lerp(out, e > -1.5 ? P.wetSand : P.sand, sandy, out);
      lawn *= 1 - sandy;
    }
    return lawn;
  }

  /**
   * The farm field a point lies in (farm worlds, outside the town): the field's kind (0..1),
   * how far across it the point is along the furrows (`v`, m), how strongly it shows (fading out
   * with distance and up the hills) and whether the point is by a hedgerow at its edge. Null elsewhere.
   */
  private fieldCell(x: number, z: number): { kind: number; v: number; weight: number; hedge: boolean } | null {
    // (A big world's fields are its field zones: the patchwork in them only, wherever they are.)
    const zones = this.zones;
    const f = zones ? (zones.has('fields') ? (this.land.fields ?? { size: 34, angle: hash(3, 5, this.seed) * Math.PI }) : null) : this.land.fields;
    if (!f) return null;
    const dist = this.distanceToTown(x, z);
    if (dist < APRON + 3) return null;
    // (How far inside the field zones: their share here, and a hedgerow along their edge.)
    const inside = zones ? zones.edge('fields', x, z) : 0;
    const weight = (zones ? shareAt(inside, 4) : 1 - smoothstep(this.ring * 0.6, this.ring * 0.85, dist)) * smoothstep(APRON + 3, APRON + 6, dist);
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
    // (A hedgerow along the patchwork's lines, and along a big world's field zones' edges.)
    return { kind: kind / 0.82, v: (v - cv) * f.size * 0.7, weight, hedge: edge < 1.6 || (!!zones && Math.abs(inside) < 1.2) };
  }

  /**
   * A big world's farmland for the town view's colour map (`pc/groundTiles.ts`), as the terrain colours it
   * (`terrainColour`, `fieldCell`): the patchwork's grid (cells `size` m along the direction `angle` and `across` m
   * across it), each cell's colour (`crop`: sRGB into `out`, its furrows' share of it evened out; false for a cell
   * left as meadow), and how strongly the fields show at a point (`weight`, 0..1: nothing off the field zones or
   * by the town). Null: no farmland. The ground's own colours are a vertex apart, so from above a field's edge
   * steps along the grid; the map draws each where it is.
   */
  farmland(): { size: number; across: number; angle: number; crop(cu: number, cv: number, out: Rgb): boolean; weight(x: number, z: number): number; box?: [number, number, number, number] } | null {
    const zones = this.zones;
    if (!zones?.has('fields')) return null;
    // (A height field's fields are its cover's: where they are, as the zones' outlines no longer say.)
    const box = this.field ? zones.extent('fields') : null;
    const f = this.land.fields ?? { size: 34, angle: hash(3, 5, this.seed) * Math.PI };
    const P = (this.palette ??= terrainPalette(this.land.grass));
    return {
      size: f.size,
      across: f.size * 0.7,
      angle: f.angle,
      ...(box ? { box } : {}),
      crop: (cu, cv, out) => {
        const kind = hash(cu, cv, this.seed + 77);
        if (kind > 0.82) return false;
        const crop = P.crops[Math.min(P.crops.length - 1, Math.floor((kind / 0.82) * P.crops.length))];
        // (The furrows' sine over a field, on average: 0.35 × 1/π.)
        lerp(crop, P.furrow, crop !== P.crops[0] ? 0.11 : 0, out);
        return true;
      },
      weight: (x, z) => {
        const dist = this.distanceToTown(x, z);
        return dist < APRON + 3 ? 0 : shareAt(zones.edge('fields', x, z), 4) * smoothstep(APRON + 3, APRON + 6, dist) * 0.9;
      },
    };
  }

  /**
   * Woods outside the town: mixed deciduous / birch / conifer stands with bushes and rocks. Trees
   * near the town are the full models and cast shadows; further ones use the low level of detail
   * (a file's own far version), which don't. The game draws them in groups by sector round the town,
   * or by square (`woodsCell`).
   */
  protected async woods(pace?: () => Promise<void>): Promise<Woods> {
    this.nearWoods = [];
    this.woodsAsked = null;
    const near = new Map<string, Float32Array[]>();
    const far = new Map<string, Float32Array[]>();
    const rules = this.woodsRules();
    const size = Math.max(this.w, this.d) + this.ring * 2;
    const x0 = this.w / 2 - size / 2;
    const z0 = this.d / 2 - size / 2;
    // (The woods' stream carries on into the orchards without a height field: theirs as they always were. With
    // one, the woods' are the squares' own, and the orchards have one of their own.)
    let rand = mulberry32(this.seed ^ 0x5eed);
    if (this.field) {
      // A height field's woods in squares, each from its own stream (`woodsBlocks`).
      this.woodBlocks = null;
      await this.woodsBlocks(null, pace);
      this.gatherWoods(near, far);
      rand = mulberry32(this.seed ^ 0x0c4a2d);
    } else {
      const spacing = 4.2;
      for (let gx = 0; gx * spacing < size; gx++) {
        if (gx % 4 === 0) await pace?.();
        for (let gz = 0; gz * spacing < size; gz++) {
          const x = x0 + (gx + rand()) * spacing;
          const z = z0 + (gz + rand()) * spacing;
          this.plant(x, z, rand, rules, 1, near, far, this.nearWoods);
        }
      }
    }
    // A big world's orchards: fruit trees in rows across each (its own draws, so the woods above are as they were).
    // (With a height field, from a stream of their own: the woods' are the squares'.)
    this.plantOrchards(rand, far);
    return { near, far, sector: this.woodsSectors() };
  }

  /** How the woods' squares are named (`Woods.sector`). */
  protected woodsSectors(): (x: number, z: number) => string {
    const [cx, cz] = [this.w / 2, this.d / 2];
    // (A big world's woods in squares, whatever the preset: sectors from its middle would each reach a kilometre.)
    const cell = this.built ? this.bigWoodsCell : this.woodsCell;
    return cell > 0 ? (x: number, z: number) => `${Math.floor(x / cell)}:${Math.floor(z / cell)}` : (x: number, z: number) => `${Math.floor((Math.atan2(z - cz, x - cx) / (Math.PI * 2) + 1) * FOREST_SECTORS) % FOREST_SECTORS}`;
  }

  /** The woods' numbers as the landscape has them: the beach, the treeline, the trees' scatter and the bush roll's shares. */
  private woodsRules(): { beach: number; treeline: number; scatter: number; bushTo: number; smallTo: number; top: number } {
    // The bush roll: rocks below 0.08, then bushes, then small ones (`scrub` of the roll in all, two to one).
    const scrub = this.land.scrub === undefined ? null : Math.min(0.92, Math.max(0, this.land.scrub));
    const [bushTo, smallTo] = scrub === null ? [0.2, 0.26] : [0.08 + (scrub * 2) / 3, 0.08 + scrub];
    return { beach: this.beach, treeline: this.treeTop, scatter: this.land.scatter ?? 1, bushTo, smallTo, top: this.peaks };
  }

  /**
   * Whether a tree, bush or rock of the woods grows at (x, z) and which, by the draws of `rand` (always in the same
   * order: the same woods from the same stream), into `near` or `far` (and `plants`, those near the town). `thin`:
   * the share of the roll kept (a square's grid is a little denser than one stream's).
   */
  private plant(x: number, z: number, rand: () => number, rules: ReturnType<LandscapeLayout['woodsRules']>, thin: number, near: Map<string, Float32Array[]>, far: Map<string, Float32Array[]>, plants: PlantPlacement[]): void {
    const { beach, treeline, scatter, bushTo, smallTo, top } = rules;
    const dist = this.distanceToTown(x, z);
    const L = this.land;
    if (dist < APRON + L.treeline) return;
    const forest = fbm(x * 0.025, z * 0.025, this.seed + 7, 3);
    let density = smoothstep(0.42 - L.forest, 0.62 - L.forest, forest) * 0.85 + 0.06 + L.forest * 0.25 - (dist < 14 ? 0.05 : 0);
    // A big world's woods are its woods zones (trees dotted about the open land, fewer on its
    // meadows), and its orchards their own (rows of fruit trees, below).
    if (this.zones) {
      if (this.zones.edge('orchard', x, z) > -3) return;
      const wood = this.zones.share('woods', x, z, 8);
      density = wood * (0.6 + 0.3 * smoothstep(0.3, 0.6, forest)) + (1 - wood) * (0.12 + 0.08 * smoothstep(0.42, 0.7, forest)) * scatter;
      density *= 1 - 0.6 * this.zones.share('meadow', x, z, 6);
    }
    // Fields are open, with hedgerows and the odd tree along their edges; nothing grows in the water or on the beach.
    const cell = this.fieldCell(x, z);
    if (cell) density = cell.hedge ? 0.55 : density * (1 - cell.weight);
    if (this.watered && this.waterEdge(x, z) > -beach * 0.8) return;
    // Bare tops on mountains.
    if (top && this.heightAt(x, z) > treeline + forest * 10) return;
    // Nothing on the roads or right by them.
    const road = this.roads.near(x, z);
    if (road && road.dist < road.width / 2 + 2.5) return;
    // The far woods of a wide world thinner (seen through haze; fewer trees to draw).
    if (this.ring > RING && dist > RING && rand() < 0.45) return;
    if (rand() > density * thin) return;
    // Species: conifers in the dense cores, birches in groves at the edges, broadleaves elsewhere.
    const grove = fbm(x * 0.06 + 40, z * 0.06 - 15, this.seed + 3, 2);
    const roll = rand();
    let kind: string;
    if (roll < 0.08) kind = 'model.rock';
    // (A hedgerow mostly bushes.)
    else if (roll < bushTo || (cell?.hedge && roll < 0.55)) kind = 'model.bush';
    else if (roll < smallTo) kind = 'model.bush.small';
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
    // (A big world's: all far versions, none casting. Its roads and trails run everywhere, and every tree
    // along them would be a full model in the sun's shadows.)
    const close = !this.built && dist < SHADOW_RANGE && kind !== 'model.rock';
    if (close) plants.push({ key: kind, x, y, z, sx: scale, sy: up, sz: scale, yaw });
    const bucket = close ? near : far;
    let list = bucket.get(kind);
    if (!list) bucket.set(kind, (list = []));
    list.push(m);
  }

  /** A big world's orchards into `far`: fruit trees in rows across each, by the draws of `draw`. */
  private plantOrchards(draw: () => number, far: Map<string, Float32Array[]>): void {
    for (const zone of this.zones ? this.orchards : []) {
      const xs = zone.poly.map((p) => p[0]);
      const zs = zone.poly.map((p) => p[1]);
      const [x0o, x1o, z0o, z1o] = [Math.min(...xs), Math.max(...xs), Math.min(...zs), Math.max(...zs)];
      // Rows along the orchard's longer side.
      const along = x1o - x0o >= z1o - z0o;
      const [len, across] = along ? [x1o - x0o, z1o - z0o] : [z1o - z0o, x1o - x0o];
      const rows = Math.max(1, Math.floor((across - 2) / ORCHARD_ROWS));
      const per = Math.max(1, Math.floor((len - 2) / ORCHARD_STEP));
      for (let r = 0; r < rows; r++) {
        for (let k = 0; k < per; k++) {
          const u = (len - (per - 1) * ORCHARD_STEP) / 2 + k * ORCHARD_STEP + (draw() - 0.5) * 0.6;
          const v = (across - (rows - 1) * ORCHARD_ROWS) / 2 + r * ORCHARD_ROWS + (draw() - 0.5) * 0.6;
          const [x, z] = along ? [x0o + u, z0o + v] : [x0o + v, z0o + u];
          const scale = 0.85 + draw() * 0.3;
          const m = placed(scale, scale * (0.95 + draw() * 0.1), scale, draw() * Math.PI * 2, x, this.heightAt(x, z) - 0.05, z);
          let list = far.get('model.tree.fruit');
          if (!list) far.set('model.tree.fruit', (list = []));
          list.push(m);
        }
      }
    }
  }

  // ---- A height field's woods and flowers, square by square ------------------------------

  /** The landscape's square (m: its corner and side) and its `LAND_BLOCK` squares: the first's indices and how many across. */
  private blockGrid(): { bx0: number; bz0: number; n: number } {
    const size = Math.max(this.w, this.d) + this.ring * 2;
    const x0 = this.w / 2 - size / 2;
    const z0 = this.d / 2 - size / 2;
    const bx0 = Math.floor(x0 / LAND_BLOCK);
    const bz0 = Math.floor(z0 / LAND_BLOCK);
    return { bx0, bz0, n: Math.max(Math.floor((x0 + size) / LAND_BLOCK) - bx0, Math.floor((z0 + size) / LAND_BLOCK) - bz0) + 1 };
  }

  /** The squares (`blockGrid` indices) a box (m) reaches, or all of them. */
  private blocksIn(box: [number, number, number, number] | null): number[] {
    const { bx0, bz0, n } = this.blockGrid();
    const [i0, j0, i1, j1] = box ? [Math.floor(box[0] / LAND_BLOCK) - bx0, Math.floor(box[1] / LAND_BLOCK) - bz0, Math.floor(box[2] / LAND_BLOCK) - bx0, Math.floor(box[3] / LAND_BLOCK) - bz0] : [0, 0, n - 1, n - 1];
    const out: number[] = [];
    for (let j = Math.max(0, j0); j <= Math.min(n - 1, j1); j++) for (let i = Math.max(0, i0); i <= Math.min(n - 1, i1); i++) out.push(j * n + i);
    return out;
  }

  /** A height field's woods, by square (`blockGrid` index): what grows in each (null before the first `woods`). */
  private woodBlocks: ({ near: Map<string, Float32Array[]>; far: Map<string, Float32Array[]>; plants: PlantPlacement[] } | undefined)[] | null = null;

  /**
   * Grows the woods of the squares a box (m) reaches again (all: null) from each square's own stream: the same
   * draws as one stream's (`plant`), on a grid of `BLOCK_GRID` points a side. Says which models' trees changed.
   */
  protected async woodsBlocks(box: [number, number, number, number] | null, pace?: () => Promise<void>): Promise<Set<string>> {
    const { bx0, bz0, n } = this.blockGrid();
    const blocks = (this.woodBlocks ??= new Array(n * n));
    const rules = this.woodsRules();
    const spacing = LAND_BLOCK / BLOCK_GRID;
    // (One stream's grid was 4.2 m apart: as many trees a hectare.)
    const thin = (spacing / 4.2) ** 2;
    const changed = new Set<string>();
    const size = Math.max(this.w, this.d) + this.ring * 2;
    const [x0, z0] = [this.w / 2 - size / 2, this.d / 2 - size / 2];
    let done = 0;
    for (const b of this.blocksIn(box)) {
      if (++done % 8 === 0) await pace?.();
      const [bx, bz] = [bx0 + (b % n), bz0 + Math.floor(b / n)];
      const rand = mulberry32(Math.floor(hash(bx, bz, this.seed ^ 0x5eed) * 4294967296));
      const near = new Map<string, Float32Array[]>();
      const far = new Map<string, Float32Array[]>();
      const plants: PlantPlacement[] = [];
      for (let gz = 0; gz < BLOCK_GRID; gz++) {
        for (let gx = 0; gx < BLOCK_GRID; gx++) {
          const x = bx * LAND_BLOCK + (gx + rand()) * spacing;
          const z = bz * LAND_BLOCK + (gz + rand()) * spacing;
          // (Only on the landscape's square, as one stream's.)
          if (x < x0 || z < z0 || x >= x0 + size || z >= z0 + size) continue;
          this.plant(x, z, rand, rules, thin, near, far, plants);
        }
      }
      for (const map of [blocks[b]?.near, blocks[b]?.far, near, far]) for (const key of map?.keys() ?? []) changed.add(key);
      blocks[b] = { near, far, plants };
    }
    return changed;
  }

  /** The squares' woods gathered into `near` and `far` (square by square, in order), and those near the town for the birds. */
  private gatherWoods(near: Map<string, Float32Array[]>, far: Map<string, Float32Array[]>): void {
    this.nearWoods = [];
    this.woodsAsked = null;
    for (const block of this.woodBlocks ?? []) {
      if (!block) continue;
      for (const [from, into] of [
        [block.near, near],
        [block.far, far],
      ] as const) {
        for (const [key, list] of from) {
          const all = into.get(key);
          if (all) all.push(...list);
          else into.set(key, [...list]);
        }
      }
      this.nearWoods.push(...block.plants);
    }
  }

  /**
   * A height field's woods grown again where a box (m) reaches (a stroke's, done): the squares there from their own
   * streams, the orchards on the land as it is now; all of the woods as placed, and which models' trees changed.
   */
  protected async woodsAgain(box: [number, number, number, number] | null, pace?: () => Promise<void>): Promise<{ woods: Woods; changed: Set<string> }> {
    const changed = await this.woodsBlocks(box, pace);
    const near = new Map<string, Float32Array[]>();
    const far = new Map<string, Float32Array[]>();
    this.gatherWoods(near, far);
    this.plantOrchards(mulberry32(this.seed ^ 0x0c4a2d), far);
    if (this.orchards.length) changed.add('model.tree.fruit');
    return { woods: { near, far, sector: this.woodsSectors() }, changed };
  }

  /**
   * Wildflowers on the town's tiles and round it (the grass itself is drawn on the GPU,
   * `shared/grass.ts`). The ones indoors or under furniture are hidden by `refreshLawn` (rooms and
   * furniture change in build mode). The draws are the ones the lawn's tufts were placed with, in
   * their order, so the flowers stand where they always have.
   */
  protected async lawns(world: WorldStructure, pace?: () => Promise<void>): Promise<LawnPlacement[]> {
    if (this.field) {
      this.flowerBlocks = null;
      return (await this.lawnsAgain(world, null, pace)).lawns;
    }
    const rand = mulberry32(this.seed ^ 0x1a3d);
    // (Tiles whose middle is paved.)
    const paved = unlawned(world, false);
    const flowers: Float32Array[] = [];
    const flowerTiles: number[] = [];
    for (let z = 0; z < groundDepth(world); z++) {
      if (z % 4 === 0) await pace?.();
      for (let x = 0; x < world.width; x++) {
        const i = z * world.width + x;
        if (paved[i]) continue;
        // (Where the tufts were, before the grass: one or two a tile.)
        const n = rand() < 0.5 ? 2 : 1;
        for (let k = 0; k < n * TUFT_DRAWS; k++) rand();
        // (A big world's water is on its tiles: no flowers in it.)
        // In drifts a few metres across, as daisies and clover grow, with a single one here and there between them.
        const drift = smoothstep(FLOWER_DRIFTS.from, FLOWER_DRIFTS.full, valueNoise(x / FLOWER_DRIFTS.span, z / FLOWER_DRIFTS.span, this.seed + 421));
        if (rand() < FLOWER_DRIFTS.single + FLOWER_DRIFTS.dense * drift && !(this.zones && this.waterEdge(x + 0.5, z + 0.5) > -this.beach * 0.6)) {
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
      if (dist === 0 || rand() < dist / 40 || this.waterEdge(x, z) > -this.beach * 0.6) continue;
      const road = this.roads.near(x, z);
      if (road && road.dist < road.width / 2 + 0.3) continue;
      // (In this order, as ever: the scale, the turn, the spot, then whether it's a flower; nine in ten were tufts.)
      const scale = 0.8 + rand() * 0.7;
      const yaw = rand() * Math.PI * 2;
      const px = x - 0.5 + rand();
      const pz = z - 0.5 + rand();
      if (rand() >= 0.1 || this.roads.onLink(px, pz)) continue;
      flowers.push(placed(scale, scale, scale, yaw, px, this.heightAt(x, z), pz));
      flowerTiles.push(-1);
    }
    // Where they gather, patches of one kind of small flower, so a drift reads as one from the game's camera: daisies
    // in most, buttercups or violets in others, more of them towards a drift's middle. (Draws of their own: the
    // flowers above stand where they always have.)
    const patchRand = mulberry32(this.seed ^ 0x5d21f);
    const patches = FLOWER_PATCHES.kinds.map((kind) => ({ kind, matrices: [] as Float32Array[], tiles: [] as number[] }));
    // (On the plots' mown lawns: the rough grass between them is over their heads. And a tile clear of paving: a patch
    // spills over its tile's edge, and stood on a path or a drive.)
    const pavedNear = unlawned(world, true);
    const clearOfPaving = (x: number, z: number): boolean => {
      for (let dz = -1; dz <= 1; dz++) {
        for (let dx = -1; dx <= 1; dx++) {
          const [nx, nz] = [x + dx, z + dz];
          if (nx >= 0 && nx < world.width && nz >= 0 && nz < groundDepth(world) && pavedNear[nz * world.width + nx]) return false;
        }
      }
      return true;
    };
    const mown = new Uint8Array(paved.length);
    for (const p of world.plots ?? []) {
      for (let z = Math.max(0, p.z); z < Math.min(groundDepth(world), p.z + p.d); z++) mown.fill(1, z * world.width + Math.max(0, p.x), z * world.width + Math.min(world.width, p.x + p.w));
    }
    // (None on a lighter lawn, Low's: they measured 0.08 ms of GPU a frame there, on WebGL2.)
    for (let z = 0; z < (this.lawnDensity >= 1 ? groundDepth(world) : 0); z++) {
      if (z % 8 === 0) await pace?.();
      for (let x = 0; x < world.width; x++) {
        const i = z * world.width + x;
        if (paved[i] || !mown[i] || !clearOfPaving(x, z)) continue;
        const drift = smoothstep(FLOWER_PATCHES.from, FLOWER_PATCHES.full, valueNoise(x / FLOWER_DRIFTS.span, z / FLOWER_DRIFTS.span, this.seed + 421));
        if (drift <= 0 || (this.zones && this.waterEdge(x + 0.5, z + 0.5) > -this.beach * 0.6)) continue;
        const which = valueNoise(x / FLOWER_PATCHES.kindSpan, z / FLOWER_PATCHES.kindSpan, this.seed + 433);
        const into = patches[which < FLOWER_PATCHES.split[0] ? 0 : which < FLOWER_PATCHES.split[1] ? 1 : 2];
        const n = Math.floor(drift * FLOWER_PATCHES.dense + patchRand());
        for (let k = 0; k < n; k++) {
          into.matrices.push(scatter(x, z, patchRand, 0.85 + patchRand() * 0.5));
          into.tiles.push(i);
        }
      }
    }
    // A lighter lawn keeps every so many of them, the same ones each time (the placement above is unchanged).
    const d = this.lawnDensity;
    const thin = <T>(list: T[]) => (d >= 1 ? list : list.filter((_, k) => Math.floor((k + 1) * d) > Math.floor(k * d)));
    // (Flowers are sparse: too few triangles for groups to pay; but a big world's are a kilometre of them, in squares.)
    const cell = this.built ? (x: number, z: number) => `${Math.floor(x / BIG_FLOWERS_CELL)}:${Math.floor(z / BIG_FLOWERS_CELL)}` : undefined;
    // (The patches in squares the camera leaves out when out of view, in a town too.)
    const patchCell = cell ?? ((x: number, z: number) => `${Math.floor(x / FLOWER_PATCHES.cell)}:${Math.floor(z / FLOWER_PATCHES.cell)}`);
    return [
      { key: 'model.flowers', matrices: thin(flowers), tiles: thin(flowerTiles), ...(cell ? { cell } : {}) },
      ...patches.filter((p) => p.matrices.length).map((p) => ({ key: p.kind, matrices: thin(p.matrices), tiles: thin(p.tiles), cell: patchCell, clear: true })),
    ];
  }

  /** A height field's lawn flowers, by square (`blockGrid` index): each model's matrices and town tiles (null before the first `lawns`). */
  private flowerBlocks: (Map<string, { matrices: Float32Array[]; tiles: number[] }> | undefined)[] | null = null;

  /**
   * A height field's lawn flowers grown again where a box (m) reaches (all: null), square by square from each one's
   * own streams, standing on the land (`lawns` as one stream grows them, on the ground's height): all of them as
   * placed, and which models' changed.
   */
  protected async lawnsAgain(world: WorldStructure, box: [number, number, number, number] | null, pace?: () => Promise<void>): Promise<{ lawns: LawnPlacement[]; changed: Set<string> }> {
    const { bx0, bz0, n } = this.blockGrid();
    const blocks = (this.flowerBlocks ??= new Array(n * n));
    const W = world.width;
    const D = groundDepth(world);
    const paved = unlawned(world, false);
    const pavedNear = unlawned(world, true);
    const mown = new Uint8Array(paved.length);
    for (const p of world.plots ?? []) for (let z = Math.max(0, p.z); z < Math.min(D, p.z + p.d); z++) mown.fill(1, z * W + Math.max(0, p.x), z * W + Math.min(W, p.x + p.w));
    const clearOfPaving = (x: number, z: number): boolean => {
      for (let dz = -1; dz <= 1; dz++) for (let dx = -1; dx <= 1; dx++) if (x + dx >= 0 && x + dx < W && z + dz >= 0 && z + dz < D && pavedNear[(z + dz) * W + x + dx]) return false;
      return true;
    };
    const wet = (x: number, z: number) => this.watered && this.waterEdge(x, z) > -this.beach * 0.6;
    // (Round the town as one stream's 5000 tries over the town and its apron, as many a square.)
    const [ax0, az0, ax1, az1] = [-APRON - 30, -APRON - 30, W + APRON + 30, D + APRON + 30];
    const tries = Math.round((5000 * LAND_BLOCK * LAND_BLOCK) / ((ax1 - ax0) * (az1 - az0)) * 10) / 10;
    const d = this.lawnDensity;
    const thin = <T>(list: T[]) => (d >= 1 ? list : list.filter((_, k) => Math.floor((k + 1) * d) > Math.floor(k * d)));
    const changed = new Set<string>();
    let done = 0;
    for (const b of this.blocksIn(box)) {
      if (++done % 8 === 0) await pace?.();
      const [bx, bz] = [bx0 + (b % n), bz0 + Math.floor(b / n)];
      const rand = mulberry32(Math.floor(hash(bx, bz, this.seed ^ 0x1a3d) * 4294967296));
      const patchRand = mulberry32(Math.floor(hash(bx, bz, this.seed ^ 0x5d21f) * 4294967296));
      const out = new Map<string, { matrices: Float32Array[]; tiles: number[] }>();
      const put = (key: string, m: Float32Array, tile: number) => {
        let at = out.get(key);
        if (!at) out.set(key, (at = { matrices: [], tiles: [] }));
        at.matrices.push(m);
        at.tiles.push(tile);
      };
      // (A flower's turn, then its spot in the tile, then it stands on the land there.)
      const standing = (x: number, z: number, r: () => number, scale: number) => {
        const yaw = r() * Math.PI * 2;
        const [px, pz] = [x + r(), z + r()];
        return placed(scale, scale, scale, yaw, px, this.heightAt(px, pz), pz);
      };
      const [tx0, tz0] = [Math.max(0, bx * LAND_BLOCK), Math.max(0, bz * LAND_BLOCK)];
      const [tx1, tz1] = [Math.min(W, (bx + 1) * LAND_BLOCK), Math.min(D, (bz + 1) * LAND_BLOCK)];
      for (let z = tz0; z < tz1; z++) {
        for (let x = tx0; x < tx1; x++) {
          const i = z * W + x;
          if (paved[i]) continue;
          // In drifts a few metres across, as daisies and clover grow, with a single one here and there between them.
          const drift = smoothstep(FLOWER_DRIFTS.from, FLOWER_DRIFTS.full, valueNoise(x / FLOWER_DRIFTS.span, z / FLOWER_DRIFTS.span, this.seed + 421));
          if (rand() < FLOWER_DRIFTS.single + FLOWER_DRIFTS.dense * drift && !wet(x + 0.5, z + 0.5)) put('model.flowers', standing(x, z, rand, 0.8 + rand() * 0.5), i);
          // The patches of one kind of small flower where they gather, on the plots' mown lawns, clear of paving.
          if (this.lawnDensity < 1 || !mown[i] || !clearOfPaving(x, z)) continue;
          const many = smoothstep(FLOWER_PATCHES.from, FLOWER_PATCHES.full, valueNoise(x / FLOWER_DRIFTS.span, z / FLOWER_DRIFTS.span, this.seed + 421));
          if (many <= 0 || wet(x + 0.5, z + 0.5)) continue;
          const which = valueNoise(x / FLOWER_PATCHES.kindSpan, z / FLOWER_PATCHES.kindSpan, this.seed + 433);
          const kind = FLOWER_PATCHES.kinds[which < FLOWER_PATCHES.split[0] ? 0 : which < FLOWER_PATCHES.split[1] ? 1 : 2];
          const copies = Math.floor(many * FLOWER_PATCHES.dense + patchRand());
          for (let k = 0; k < copies; k++) put(kind, standing(x, z, patchRand, 0.85 + patchRand() * 0.5), i);
        }
      }
      // Lawn detail round the town too, thinning out with distance.
      const count = Math.floor(tries) + (rand() < tries % 1 ? 1 : 0);
      for (let t = 0; t < count; t++) {
        const x = bx * LAND_BLOCK + rand() * LAND_BLOCK;
        const z = bz * LAND_BLOCK + rand() * LAND_BLOCK;
        if (x < ax0 || z < az0 || x >= ax1 || z >= az1) continue;
        const dist = this.distanceToTown(x, z);
        if (dist === 0 || rand() < dist / 40 || wet(x, z)) continue;
        const road = this.roads.near(x, z);
        if (road && road.dist < road.width / 2 + 0.3) continue;
        if (rand() >= 0.1) continue;
        const m = standing(x - 0.5, z - 0.5, rand, 0.8 + rand() * 0.7);
        if (this.roads.onLink(m[12], m[14])) continue;
        put('model.flowers', m, -1);
      }
      for (const at of out.values()) {
        at.matrices = thin(at.matrices);
        at.tiles = thin(at.tiles);
      }
      for (const map of [blocks[b], out]) for (const key of map?.keys() ?? []) changed.add(key);
      blocks[b] = out;
    }
    const all = new Map<string, { matrices: Float32Array[]; tiles: number[] }>();
    for (const key of ['model.flowers', ...FLOWER_PATCHES.kinds]) all.set(key, { matrices: [], tiles: [] });
    for (const block of blocks) {
      for (const [key, at] of block ?? []) {
        const into = all.get(key)!;
        into.matrices.push(...at.matrices);
        into.tiles.push(...at.tiles);
      }
    }
    const cell = this.built ? (x: number, z: number) => `${Math.floor(x / BIG_FLOWERS_CELL)}:${Math.floor(z / BIG_FLOWERS_CELL)}` : (x: number, z: number) => `${Math.floor(x / FLOWER_PATCHES.cell)}:${Math.floor(z / FLOWER_PATCHES.cell)}`;
    const lawns = [...all]
      .filter(([, at]) => at.matrices.length)
      .map(([key, at]) => ({ key, matrices: at.matrices, tiles: at.tiles, cell, ...(key === 'model.flowers' ? {} : { clear: true }) }));
    return { lawns, changed };
  }
}

/** A relief grid's checksum (for `LandscapeLayout.key`), once per grid. */
function reliefSum(relief: { cols: number; rows: number; h: readonly number[] }): string {
  let sum = reliefSums.get(relief);
  if (sum === undefined) {
    let h = 0x811c9dc5;
    for (const v of relief.h) h = Math.imul(h ^ v, 0x01000193) >>> 0;
    reliefSums.set(relief, (sum = `${relief.cols}x${relief.rows}:${h.toString(36)}`));
  }
  return sum;
}
const reliefSums = new WeakMap<object, string>();

// ---- Garden dressing ------------------------------------------------------------------

export type Decor = [number, number, number, number][];

/**
 * The lawn's flowers in drifts: the noise they follow (`span` m a cell), its values where a drift begins and is
 * full, the share of tiles with a flower in a full drift and between drifts. A little fewer in all than when
 * six tiles in a hundred had one, wherever they were (the noise is over `from` on about a third of the land).
 */
const FLOWER_DRIFTS = tuned('flowers', { span: 6, from: 0.6, full: 0.76, dense: 0.22, single: 0.006 });

/**
 * The patches of small flowers where the lawn's flowers gather (`lawns`): the drifts' noise from where there are
 * any to where there are `dense` of them a tile; their models, and the values of a wider noise (`kindSpan` m a
 * cell) that part the first kind's drifts from the second's and the third's; `cell`: the squares (m) they are drawn in.
 *
 * A patch or two a garden, at the drifts' cores (`from` 0.74): as first drawn (from 0.6, 2.2 a tile) they were the
 * busiest thing outside the house and sparkled as the camera moved (a tenth more of a play picture's pixels
 * changed under a quarter-pixel move; now a fiftieth more), and they stood on paths, at doorsteps and against the
 * foot of walls: now a tile clear of paving, of the house and of what stands in the garden (`LawnLayer.clear`).
 */
const FLOWER_PATCHES = tuned('patches', {
  from: 0.74,
  full: 0.92,
  dense: 1.6,
  kinds: ['model.flowers.daisies', 'model.flowers.buttercups', 'model.flowers.violets'],
  kindSpan: 17,
  split: [0.52, 0.64],
  cell: 32,
});

/** Clear distances for lot trees (trunk to edge, metres). */
const TREE_CLEAR = { street: 3.0, path: 1.3, house: 2.2, object: 1.0 };

/**
 * Moves a lot tree (by up to 2.5 m, staying on the viewed lot) until its trunk is clear of the
 * roads (road, sidewalk, kerb furniture and streetlights all stand within their width, see
 * street.ts), garden paths and driveways, the house and placed objects; null when no spot is.
 */
function clearTreeSpot(x: number, z: number, world: WorldStructure, view: { x: number; z: number; w: number; d: number } | null): [number, number] | null {
  const roads = roadIndex(townRoads(world));
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
    !roads.onRoad(px, pz, TREE_CLEAR.street) &&
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

/** Fallen petals under a lot's tree in blossom (`natureDecor`): no plant (no bird lives in it, and it casts no shadow). */
export const PETALS = 'model.flowers.petals';
/** How far they lie, of the model's 2.3 m at a tree's scale 1. */
const PETALS_SPREAD = 1.05;

/** The garden dressing (`natureDecor`'s) as placed, for the birds: its plants (not the petals fallen from them). */
export function decorPlants(decor: readonly [string, Decor][]): PlantPlacement[] {
  return decor.flatMap(([key, items]) => (key === PETALS ? [] : items.map(([x, z, s, yaw]) => ({ key, x, y: 0, z, sx: s, sy: s, sz: s, yaw }))));
}

/**
 * Splits the house builder's garden dressing (`model.tree` lot trees, `model.bush` foundation
 * shrubs; items are [x, z, scale, yaw]) into species. Lot trees mix oaks, maples, blossom, fruit,
 * birch, magnolia and Japanese maple, with fallen petals under the two in blossom; each house gets either a clipped hedge along its walls or a
 * mixed border of shrubs, hydrangeas, roses, lavender, clipped balls and beds of lupins and marigolds. Deterministic per position.
 */
export function natureDecor(key: string, items: Decor, world?: WorldStructure, view?: { x: number; z: number; w: number; d: number } | null): [string, Decor][] {
  const out = new Map<string, Decor>();
  const put = (k: string, item: [number, number, number, number]) => {
    const list = out.get(k);
    if (list) list.push(item);
    else out.set(k, [item]);
  };
  if (key === 'model.tree') {
    for (const item of items) {
      const h = hash(Math.floor(item[0] * 7), Math.floor(item[1] * 7), 91);
      // Crowns are ~2 m in radius: keep trunks well off the sidewalk, its streetlights and props.
      const spot = world ? clearTreeSpot(item[0], item[1], world, view ?? null) : [item[0], item[1]];
      if (!spot) continue;
      const species =
        h < 0.24 ? 'model.tree' : h < 0.4 ? 'model.tree.maple' : h < 0.55 ? 'model.tree.blossom' : h < 0.68 ? 'model.tree.fruit' : h < 0.8 ? 'model.tree.birch' : h < 0.9 ? 'model.tree.magnolia' : 'model.tree.japaneseMaple';
      put(species, [spot[0], spot[1], item[2], item[3]]);
      // A tree in blossom has shed some of it: petals on the lawn under its crown.
      if (species === 'model.tree.blossom' || species === 'model.tree.magnolia') put(PETALS, [spot[0], spot[1], item[2] * PETALS_SPREAD, item[3]]);
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
        // A mixed border: green shrubs with hydrangeas, roses, lavender and clipped balls among them, and a bed of
        // lupins or marigolds here and there (the plain green shrub was three in ten of it, and most of what showed).
        const kind =
          h < 0.18 ? 'model.bush' : h < 0.31 ? 'model.bush.flowering' : h < 0.4 ? 'model.bush.small' : h < 0.52 ? 'model.garden.hydrangea' : h < 0.63 ? 'model.garden.rosebush' : h < 0.73 ? 'model.garden.lavender' : h < 0.81 ? 'model.garden.boxwood' : h < 0.91 ? 'model.garden.lupins' : 'model.garden.marigolds';
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
  // (A tree's leaf cards, as the leaves they are.)
  'material.finish.leafcards': [WIND.bend, 0.012, 3.1, 1.5],
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

/** What the lawn grows round (`lawnMask`, `bareTiles`, the grass's walls, the lawn's map): rooms, things, walls, diagonals, floors, fences, doors, plots, paths and fields. */
export interface LawnInputs {
  width: number;
  rooms: ArrayLike<number>;
  things: Int32Array;
  defs: string;
  edges: string;
  meta: unknown;
}

export function lawnInputs(world: WorldStructure): LawnInputs {
  const things = new Int32Array(world.objects.length * 5);
  world.objects.forEach((o, i) => things.set([o.x, o.z, o.w, o.d, o.rot], i * 5));
  return {
    width: world.width,
    rooms: world.rooms,
    things,
    defs: world.objects.map((o) => o.def).join(),
    edges: JSON.stringify([world.walls.length, world.diagonals, world.floors, world.walls.map((e) => `${e.axis}${e.x},${e.z}`).join(), lawnEdges(world)]),
    meta: [world.meta?.paths, world.meta?.streets, world.meta?.roads, world.meta?.world?.fields],
  };
}

/** What the lawn's map reads besides (`shared/lawn.ts`): the fences and gates, the doors and the plots. */
function lawnEdges(world: WorldStructure): string {
  const fences = (world.fences ?? []).map((e) => `${e.axis}${e.x},${e.z}${e.kind === 'gate' ? 'g' : ''}`).join();
  const doors = world.openings.filter((o) => o.kind === 'door').map((o) => `${o.axis}${o.x},${o.z}`).join();
  return `${fences}|${doors}|${world.plots.map((p) => `${p.x},${p.z},${p.w},${p.d}`).join()}`;
}

export function sameLawnInputs(a: LawnInputs, b: LawnInputs): boolean {
  if (a.width !== b.width || a.rooms.length !== b.rooms.length || a.things.length !== b.things.length || a.defs !== b.defs || a.edges !== b.edges) return false;
  const [am, bm] = [a.meta as unknown[], b.meta as unknown[]];
  if (am.some((v, i) => v !== bm[i] && JSON.stringify(v) !== JSON.stringify(bm[i]))) return false;
  for (let i = 0; i < a.things.length; i++) if (a.things[i] !== b.things[i]) return false;
  if (a.rooms !== b.rooms) for (let i = 0; i < a.rooms.length; i++) if (a.rooms[i] !== b.rooms[i]) return false;
  return true;
}
