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

import { groundDepth, type RoadPlan, type WorldStructure } from '../../core/protocol';
import { COVER, LAND_CELL } from '../../game/land';
import { SKIRT_MOST, type BuiltSurface } from './builtSurface';
import { roadTiles } from './roads';
import { compose, quatAxisAngle } from './math';
import { fbm, hash, mulberry32, ridged, smoothstep, valueNoise } from '../../game/noise';
import { type ZoneKind } from './zones';
import { layBackdropGrounds, wayCorners } from './townBuildings';

/**
 * How far the landscape extends beyond the town (metres): towns of older code, and worlds (which
 * reach further, to a farther horizon). The valley's rim rises over `RING` either way.
 */
export const RING = 150;
export const WORLD_RING = 260;
/** How far (m) `world`'s landscape reaches past its land's edge (the layout's own ring, known before one is made). */
export function landscapeRing(world: { meta?: { world?: unknown } | null }): number {
  return world.meta?.world ? WORLD_RING : RING;
}
/** Flat apron around the town before the hills start. */
export const APRON = 6;
/** A big world's distance-from-built grid (m a cell; `distanceToTown`). */
export const BUILT_CELL = 2;
/** A big world's woods are drawn in squares this wide (m), those far off left out (`pc/nature.ts` `frameTiles`); the preset's own (`OpenWorldQuality.woodsCell`) where it gives one. */
export const BIG_WOODS_CELL = 256;
/** A big world's lawn flowers are drawn in squares this wide (m), those far off left out (`pc/nature.ts` `frameTiles`). */
export const BIG_FLOWERS_CELL = 128;
/** A big world's orchards: fruit trees in rows this far apart (m), and this far apart along a row. */
export const ORCHARD_ROWS = 6;
export const ORCHARD_STEP = 5;
/** A big world's hill zones rise this high (m) at most (between what is built: the land there stays flat). */
export const BIG_HILLS = 16;
export const TERRAIN_STEP = 2.5;
/** A big world's ground tiles (m; `pc/nature.ts` `terrainTiles`), laid from the terrain's corner. */
export const GROUND_TILE = 128;
/** At most this many terrain quads across (a wider landscape gets coarser ones). */
export const TERRAIN_SEGMENTS = 260;
/** Trees within this distance of the town cast shadows; further ones don't. */
export const SHADOW_RANGE = 28;
/**
 * A land drawn from its height field (`LandscapeLayout.setLand`): past the world's edge the landscape's own land,
 * moved by how far the field's edge stands off it there, the move fading out over this far (m).
 */
export const LAND_EDGE_FADE = 60;
/**
 * Past the world's edge only a height field's sea runs on: its other water (a pond, a river) reaching the edge ends
 * there, behind a bank this far over its level (m, as the brushes' `BANK`), and the landscape's water past the edge is
 * left out within this far (m) along the edge of it (its sandy bank reaches `BEACH * 1.5` from its shore, give or take
 * the zones' wandering edge).
 */
export const END_BANK = 0.25;
export const END_REACH = 20;
/** Lots are filed for `padAt` by cells this wide (m). */
export const PAD_CELL = 16;

/**
 * `values` (a `cols` × `rows` grid of `LAND_CELL` m samples) at (`x`, `z`) on the triangles the terrain draws them
 * as (`pc/nature.ts` `tileMesh`: each cell split from its corner at (x, z + cell) to the one at (x + cell, z)), the
 * edge's past the edge: what stands on the land stands on the terrain drawn, not a hair above or below it.
 */
export function between(values: Float32Array, cols: number, rows: number, x: number, z: number): number {
  const fx = Math.min(cols - 1, Math.max(0, x / LAND_CELL));
  const fz = Math.min(rows - 1, Math.max(0, z / LAND_CELL));
  const i = Math.min(Math.floor(fx), Math.max(0, cols - 2));
  const j = Math.min(Math.floor(fz), Math.max(0, rows - 2));
  const [i1, j1] = [Math.min(i + 1, cols - 1), Math.min(j + 1, rows - 1)];
  // (u across from the cell's west side, v down from its north (larger z) side: the split is u = v.)
  const [u, v] = [fx - i, 1 - (fz - j)];
  const [h00, h10, h01, h11] = [values[j1 * cols + i], values[j1 * cols + i1], values[j * cols + i], values[j * cols + i1]];
  return u >= v ? h00 + u * (h10 - h00) + v * (h11 - h10) : h00 + v * (h01 - h00) + u * (h11 - h01);
}

/**
 * The surface's graded land (`BuiltSurface.conformed`) with the samples inside the first `count` of its plots (the
 * world's lots, not its backdrop's buildings, which stand on the land) carried in from the land round them: ring by
 * ring from their edges, each the mean of its neighbours reached already (the terrain beside a lot is the land's, not
 * a slope to its pad: `LandscapeLayout.terrainField`). A sample of a lot beside another lot (a step or less apart:
 * the narrow strip between them, which has no sample of its own) keeps its pad, so the strip lies between the two
 * pads, not down at the land beyond them. So does one beside land within `SKIRT_MOST` of its pad: there the land
 * slopes to the pad and meets it at its edge (as the surface grades it round a lot whose edge is off the land's
 * grain), not standing a few centimetres over it or under it behind a retaining edge, which went up the slope in
 * metre-long steps.
 */
export function lotsFilled(surface: BuiltSurface, count: number): Float32Array {
  const { cols, rows, conformed } = surface;
  const out = conformed.slice();
  // Each sample's lot (-1: none; on a shared edge, the last).
  const owner = new Int32Array(cols * rows).fill(-1);
  for (let n = 0; n < Math.min(count, surface.plots.length); n++) {
    const p = surface.plots[n];
    for (let j = Math.max(0, Math.ceil(p.z / LAND_CELL)); j <= Math.min(rows - 1, Math.floor((p.z + p.d) / LAND_CELL)); j++) {
      for (let i = Math.max(0, Math.ceil(p.x / LAND_CELL)); i <= Math.min(cols - 1, Math.floor((p.x + p.w) / LAND_CELL)); i++) owner[j * cols + i] = n;
    }
  }
  const around = (k: number, visit: (n: number) => void) => {
    const [i, j] = [k % cols, Math.floor(k / cols)];
    if (i > 0) visit(k - 1);
    if (i < cols - 1) visit(k + 1);
    if (j > 0) visit(k - cols);
    if (j < rows - 1) visit(k + cols);
  };
  // (1: still to fill. A lot's sample beside another lot's keeps its pad: none to fill.)
  const todo = new Uint8Array(cols * rows);
  let ring: number[] = [];
  for (let k = 0; k < owner.length; k++) {
    if (owner[k] < 0) continue;
    const pad = surface.padY(owner[k]);
    let beside = false;
    around(k, (q) => (beside ||= (owner[q] >= 0 && owner[q] !== owner[k]) || (owner[q] < 0 && Math.abs(conformed[q] - pad) <= SKIRT_MOST)));
    if (beside) continue;
    todo[k] = 1;
    ring.push(k);
  }
  while (ring.length) {
    const reached: number[] = [];
    const values: number[] = [];
    const later: number[] = [];
    for (const k of ring) {
      let [sum, n] = [0, 0];
      around(k, (q) => {
        if (todo[q]) return;
        sum += out[q];
        n++;
      });
      if (n) {
        reached.push(k);
        values.push(sum / n);
      } else later.push(k);
    }
    // (A lot no land reaches, all of the world: its pad stays.)
    if (!reached.length) break;
    reached.forEach((k, m) => {
      out[k] = values[m];
      todo[k] = 0;
    });
    ring = later;
  }
  return out;
}

/** A height field whose top stands over this (m) has rock, snow and a treeline on it, as mountains do. */
export const LAND_PEAKS = 40;
/**
 * A land drawn from its height field grows its woods and flowers in squares this wide (m), each from a stream of
 * its own (`mulberry32(hash(bx, bz, seed))`): a stroke of the brushes grows only the squares it reached again.
 */
export const LAND_BLOCK = 32;
/** A square's woods on a grid of this many points across (`LAND_BLOCK` / 8 = 4 m apart, not the 4.2 of one stream: as dense, less this share). */
export const BLOCK_GRID = 8;
/** The cover's kinds as zones (`COVER`). */
export const COVER_ZONES: readonly [ZoneKind, number][] = [
  ['woods', COVER.woods],
  ['meadow', COVER.meadow],
  ['fields', COVER.fields],
];
/** Mountains: no trees above about this height (m). */
export const TREELINE = 12;
/** Mountains: bare rock and then snow from these shares of their height up to these (a relief grid's: higher up, its massif is all of it). */
export const ROCK_SNOW: [[number, number], [number, number]] = [
  [0.16, 0.28],
  [0.33, 0.45],
];
export const RELIEF_ROCK_SNOW: [[number, number], [number, number]] = [
  [0.45, 0.6],
  [0.76, 0.92],
];
/** A world with a relief grid among mountains: no trees above this share of the mountains' height. */
export const RELIEF_TREELINE = 0.3;
/** How smoothly (m) a relief grid's land meets the most it may rise from what is built (`smoothMin`). */
export const RELIEF_BLEND = 6;
/** A relief grid's land rises at most this steeply from what is built where the world doesn't say (`LandscapeParams.slope`). */
export const RELIEF_SLOPE = 0.5;
/** The mountains past a relief world's edge rise over this far (m) out from it. */
export const EDGE_MOUNTAINS: [number, number] = [10, 150];
/** The random draws a lawn tuft took (its scale, turn and spot), still taken so the flowers stand where they did. */
export const TUFT_DRAWS = 4;
/** No lawn this close to a road's edge (m): the grass's blades lean, and would reach over it. */
export const LAWN_CLEAR = 0.6;
/** Height of the sea or a lake (m); the shore is where the land meets it. */
export const WATER_Y = -0.35;
/**
 * Past the world's edge a height field's water is rounded (`roundedWater`): from the edge's own carried straight out at
 * the edge to wholly rounded this far (m) past it.
 */
export const ROUND_IN = 8;
/**
 * Inside the world's edge, within this far (m) of it, a height field's shore is drawn into the water rounded past the
 * edge where that is wetter (`shoreIn`): from wholly at the edge to not at all this far in.
 */
export const BLEND_IN = 8;
/** Past the world's edge, behind the sandy bank of a height field's water the land rises no steeper than this (19°). */
export const SHORE_RISE = 0.35;
/** Past the world's edge a height field's edge's stand off the landscape is carried out over this (m), not its banks and beds. */
export const BANK_TOP = WATER_Y + 1;
/** How far (m) round a road, lot or building that came or went the land's plants and lawn are grown again (`relayout`): past what clears them. */
export const RELAY_REACH = 12;
/** Sand from the shoreline up the bank (m). */
export const BEACH = 8;
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
export const FOREST_SECTORS = 8;
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
export interface Rgb {
  r: number;
  g: number;
  b: number;
}

/** An sRGB hex colour (`#rrggbb`) as 0..1 channels (each byte / 255). */
export function hex(c: string): Rgb {
  return { r: parseInt(c.substring(1, 3), 16) / 255, g: parseInt(c.substring(3, 5), 16) / 255, b: parseInt(c.substring(5, 7), 16) / 255 };
}

/** The colour `t` of the way from `a` to `b`, per channel (`out` may be `a`). */
export function lerp(a: Rgb, b: Rgb, t: number, out: Rgb): void {
  out.r = a.r + (b.r - a.r) * t;
  out.g = a.g + (b.g - a.g) * t;
  out.b = a.b + (b.b - a.b) * t;
}

/** The terrain's colours: a world's four greens (`LandscapeParams.grass`), and the rest. */
export function terrainPalette(greens: readonly string[]) {
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
export type TerrainPalette = ReturnType<typeof terrainPalette>;

// ---- Placements ---------------------------------------------------------------------

/** A placement's matrix: scaled, turned `yaw` about up, at (x, y, z) (`compose`, `math.ts`). */
export function placed(sx: number, sy: number, sz: number, yaw: number, x: number, y: number, z: number): Float32Array {
  return compose(sx, sy, sz, quatAxisAngle(0, 1, 0, yaw), x, y, z, new Float32Array(16)) as Float32Array;
}

export function scatter(x: number, z: number, rand: () => number, scale: number, y = 0): Float32Array {
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
export let lastBare: { inputs: LawnInputs; bare: Uint8Array } | null = null;

/**
 * `bareTiles` for a world whose lawn inputs (`lawnInputs`) and mask the caller has made already (the
 * landscape's `refreshLawn`): kept as the answer `bareTiles` gives for those inputs (the birds' next).
 */
export function keepBareTiles(world: WorldStructure, inputs: LawnInputs, mask: LawnMask): Uint8Array {
  const bare = bareTilesOf(world, mask);
  lastBare = { inputs, bare };
  return bare;
}

export function bareTilesOf(world: WorldStructure, mask: LawnMask): Uint8Array {
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
 * fields, and what the backdrop's buildings stand on (their forecourts, yards and ways to the road, as the street lays
 * them: `layBackdropGrounds`) (the birds forage on the lawn). `any`: the tiles any of them reaches into; else only those
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
  const backdrop = world.meta?.backdrop;
  if (backdrop?.length) {
    // (Not on the lots: their gardens are theirs, as the street lays it.)
    const lots = new Uint8Array(w * d);
    for (const p of world.plots) for (let z = Math.max(0, Math.floor(p.z)); z < Math.min(d, Math.ceil(p.z + p.d)); z++) lots.fill(1, z * w + Math.max(0, Math.floor(p.x)), z * w + Math.min(w, Math.ceil(p.x + p.w)));
    const grounds = new Uint8Array(w * d);
    const driven = (world.meta?.roads ?? []).filter((r) => r.kind !== 'trail');
    for (const { court, way } of layBackdropGrounds(backdrop, driven, world.plots)) {
      if (court) fillQuad(grounds, w, d, court, any);
      if (way) fillQuad(grounds, w, d, wayCorners(way), any);
    }
    for (let i = 0; i < out.length; i++) if (grounds[i] && !lots[i]) out[i] = 1;
  }
  return out;
}

/** Marks (1) the tiles of a `w` × `d` grid a convex quadrilateral `q` covers the middle of (`any`: reaches into at all). */
export function fillQuad(out: Uint8Array, w: number, d: number, q: readonly (readonly [number, number])[], any: boolean): void {
  // (Its sides' signs: the same for every side at a point inside, whichever way round it is wound.)
  const side = (k: number, x: number, z: number) => {
    const [a, b] = [q[k], q[(k + 1) % q.length]];
    return (b[0] - a[0]) * (z - a[1]) - (b[1] - a[1]) * (x - a[0]);
  };
  const inside = (x: number, z: number) => {
    let pos = false;
    let neg = false;
    for (let k = 0; k < q.length; k++) {
      const v = side(k, x, z);
      if (v > 0) pos = true;
      else if (v < 0) neg = true;
    }
    return !(pos && neg);
  };
  // (Reaching into a tile: its middle within half a tile's diagonal of the quad, near enough.)
  const grow = any ? 0.71 : 0;
  const xs = q.map((p) => p[0]);
  const zs = q.map((p) => p[1]);
  const [x0, x1] = [Math.max(0, Math.floor(Math.min(...xs) - grow)), Math.min(w - 1, Math.ceil(Math.max(...xs) + grow))];
  const [z0, z1] = [Math.max(0, Math.floor(Math.min(...zs) - grow)), Math.min(d - 1, Math.ceil(Math.max(...zs) + grow))];
  const [cx, cz] = [xs.reduce((a, v) => a + v, 0) / q.length, zs.reduce((a, v) => a + v, 0) / q.length];
  for (let z = z0; z <= z1; z++) {
    for (let x = x0; x <= x1; x++) {
      let [px, pz] = [x + 0.5, z + 0.5];
      // (Grown: the tile's middle pulled towards the quad's by up to `grow`.)
      if (grow) {
        const l = Math.hypot(cx - px, cz - pz);
        if (l > 0) [px, pz] = [px + ((cx - px) / l) * Math.min(grow, l), pz + ((cz - pz) / l) * Math.min(grow, l)];
      }
      if (inside(px, pz)) out[z * w + x] = 1;
    }
  }
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

/** The most samples a side of each part `LandscapeLayout.takeRoads` says to draw again (128 m): where the land moved, not the box round it all. */
export const TAKEN_BLOCK = 64;

/**
 * `v` as JSON with its objects' keys sorted: two worlds whose look or options are the same but were
 * made in another order (the menus' draft's, the sim's) keep the same landscape (`baseKey`).
 */
export function canonicalJson(v: unknown): string {
  if (Array.isArray(v)) return `[${v.map(canonicalJson).join(',')}]`;
  // (A typed array's keys are its indices, which must keep their order: only plain objects are sorted.)
  if (v && typeof v === 'object' && !ArrayBuffer.isView(v)) {
    const o = v as Record<string, unknown>;
    const keys = Object.keys(o).filter((k) => o[k] !== undefined).sort();
    return `{${keys.map((k) => `${JSON.stringify(k)}:${canonicalJson(o[k])}`).join(',')}}`;
  }
  return JSON.stringify(v) ?? 'null';
}

/** The same road laid out again (not just another object of it): its middle, width, kind, ends and the decks the land's water made. */
export function sameRoad(a: RoadPlan, b: RoadPlan): boolean {
  if (a === b) return true;
  if (a.width !== b.width || a.kind !== b.kind || a.ends[0] !== b.ends[0] || a.ends[1] !== b.ends[1] || a.points.length !== b.points.length) return false;
  if (a.points.some(([x, z], i) => x !== b.points[i][0] || z !== b.points[i][1])) return false;
  const decks = (r: RoadPlan) => `${r.bridge ?? ''}|${(r.bridges ?? []).map(([s, e]) => `${s}:${e}`).join(',')}`;
  return decks(a) === decks(b);
}

/**
 * The land round a town and what grows on it, as the renderer builds it (`pc/nature.ts`'s `Landscape`
 * draws it). `R`: the engine's country roads (their layout is `CountryRoads`'s).
 */

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
    meta: [world.meta?.paths, world.meta?.streets, world.meta?.roads, world.meta?.world?.fields, world.meta?.backdrop],
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
