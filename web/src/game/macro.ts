/**
 * A big world (docs/design/open-world.md, size `big`): 896 to 1024 m of land with a main town (the
 * kind's own shape, larger than a Normal town, its streets reaching all of its box), five to seven
 * hamlets 170 m or more from each other and spread over the whole world, country roads between
 * them and out of the world with farmsteads and cottages along them, and countryside: woods in
 * copses, meadows, hills, water, farmland and orchards round the hamlets and farmsteads, and
 * outdoor places (a picnic meadow, a fishing jetty, a lookout, a campfire, an orchard, a hammock
 * grove) reached by trails from the nearest hamlet.
 *
 * The land is a cost grid of 8 m cells rolled from noise (how steep, how wooded, where the water
 * is); the areas go where it is flat and dry, far enough apart (a Poisson disc); country roads
 * are A* paths over the grid, smoothed into curves; trails are A* paths on a finer grid round
 * the lots. One `Ground` covers the whole world, and each area's lots go along its own roads,
 * nearest its middle first (`town.ts` `LotPlacer`). Every area has a bus stop or two, so every
 * house is within ~120 m of a way out of town for people on foot.
 */

import type { Content } from '../content/content';
import type { Rle, TownArea, WorldZone } from '../core/protocol';
import { random } from './housegen';
import { fbm } from './noise';
import { centroid, cropRoads, curve, inside, leftOf, lengths, nearest, pointAt, RoadIndex, roadEnds, type Box, type Pt, type RoadPlan } from './roads';
import { BIG_COURTS, COURTS, frontageOf, Ground, houseFor, joinUp, LotPlacer, plotRect, settleRoads, slotDims, turner, wantedLots, type Facing, type Frontage, type NeighbourhoodDraft, type PlotSlot, type PublicTemplate, type Rect, type Templates } from './town';
import { clearing, countryLane, crescentTown, crossroads, highStreet, streetNames, WORLDS, worldName, worldTagline, type RoadLayout, type SizeSpec, type WorldKind } from './world';

type Rand = () => number;
type Shape = (rand: Rand, box: Box, scale?: number) => RoadLayout;

/** A big world's areas, land, stops and parking, as the town file carries them (`assembleTown`). */
export interface OpenWorldDraft {
  /** Area 0 is the main town. */
  areas: TownArea[];
  zones: WorldZone[];
  /** The water's tiles (1 m, row-major over the world), run-length (`Rle`). */
  water: Rle;
  /** The height (m) each outdoor plot is levelled to, by plot index. */
  pads: Record<number, number>;
  /** Bus stops: a way out of town for people on foot, in each area. */
  stops: { x: number; z: number; area: number }[];
  parking: { x: number; z: number; rot: number; area: number }[];
}

/** What a kind of world is like when it is big. */
interface Character {
  /** The world's size (m, both ways; a multiple of 64 in this range). */
  world: [number, number];
  /** Hamlets besides the main town. */
  hamlets: [number, number];
  /** A hamlet: its shapes (each hamlet one of them), its box, lots, how far its lanes reach (`SizeSpec.scale`), its streets out into its box and cross streets (`fill`, `extend`). */
  hamlet: { shapes: Shape[]; box: Box; lots: [number, number]; farms?: [number, number]; places: [number, number]; scale: number; fill?: number; extend?: number };
  /** The main town's shape, if not the kind's own. */
  main?: Shape;
  water: 'sea' | 'lake' | 'ponds';
  /** Share of the land in woods, and how hilly it is (m). */
  woods: number;
  hills: number;
  /** A valley down the middle, mountains either side. */
  valley?: boolean;
  /** Farmland round the areas and the farmsteads, as far as this past an area's box (m). */
  fields?: number;
  /** Outdoor places, in the order areas take them (`PublicTemplate` ids), and how many each area has. */
  spots: string[];
  spotsPerArea: [number, number];
}

/** A hamlet: a high street with a back lane, or a crescent round a green, its streets reaching all of its box. */
const HAMLET: Character['hamlet'] = { shapes: [highStreet, crescentTown], box: { w: 220, d: 160 }, lots: [16, 24], places: [1, 1], scale: 1.1, fill: 2, extend: 1 };

const CHARACTER: Record<WorldKind, Character> = {
  small: { world: [896, 960], hamlets: [5, 6], hamlet: HAMLET, water: 'ponds', woods: 0.42, hills: 14, fields: 100, spots: ['picnicMeadow', 'lookout', 'orchard', 'hammockGrove', 'fishingJetty'], spotsPerArea: [1, 1] },
  medium: { world: [896, 960], hamlets: [5, 6], hamlet: HAMLET, water: 'ponds', woods: 0.42, hills: 14, fields: 100, spots: ['picnicMeadow', 'fishingJetty', 'lookout', 'orchard', 'hammockGrove'], spotsPerArea: [1, 1] },
  dense: { world: [960, 1024], hamlets: [5, 7], hamlet: HAMLET, water: 'ponds', woods: 0.34, hills: 10, fields: 100, spots: ['picnicMeadow', 'orchard', 'lookout', 'fishingJetty'], spotsPerArea: [1, 1] },
  farm: {
    world: [960, 1024],
    hamlets: [5, 7],
    hamlet: { shapes: [countryLane], box: { w: 250, d: 180 }, lots: [16, 20], farms: [2, 3], places: [0, 0], scale: 1, fill: 2 },
    main: crossroads,
    water: 'ponds',
    woods: 0.26,
    hills: 7,
    fields: 150,
    spots: ['orchard', 'picnicMeadow', 'fishingJetty', 'lookout'],
    spotsPerArea: [1, 1],
  },
  seaside: { world: [960, 1024], hamlets: [5, 6], hamlet: HAMLET, water: 'sea', woods: 0.32, hills: 14, fields: 90, spots: ['fishingJetty', 'lookout', 'picnicMeadow', 'hammockGrove'], spotsPerArea: [1, 1] },
  mountain: { world: [960, 1024], hamlets: [5, 6], hamlet: HAMLET, water: 'ponds', woods: 0.54, hills: 10, valley: true, spots: ['lookout', 'campfire', 'hammockGrove', 'fishingJetty'], spotsPerArea: [1, 1] },
  lakeside: { world: [960, 1024], hamlets: [5, 6], hamlet: HAMLET, water: 'lake', woods: 0.4, hills: 10, fields: 90, spots: ['fishingJetty', 'picnicMeadow', 'hammockGrove', 'lookout'], spotsPerArea: [1, 1] },
  forest: {
    world: [960, 1024],
    hamlets: [5, 6],
    hamlet: { shapes: [clearing], box: { w: 220, d: 190 }, lots: [16, 20], places: [1, 1], scale: 1.2, fill: 2 },
    water: 'ponds',
    woods: 0.78,
    hills: 12,
    spots: ['campfire', 'hammockGrove', 'lookout', 'fishingJetty', 'picnicMeadow'],
    spotsPerArea: [1, 2],
  },
};

/** The land's cells (m). */
const CELL = 8;
/** Areas this far apart at least (their middles, m), and as far as this from the one they join; best this far from the nearest. */
const AREA_APART = 170;
const AREA_REACH = 440;
const AREA_AIM = 230;
/** No area nearer the world's edge than this (m), nor nearer another's box. */
const EDGE = 50;
const BOX_GAP = 24;
/** Country roads' width (m), trails'. */
const COUNTRY_WIDTH = 6;
const TRAIL_WIDTH = 2;
/** Houses as far as this from a stop get one of their own (m). */
const STOP_REACH = 105;
/** Houses with no room in any area go along a country road, as far as this from their own area's box (m). */
const SPILL = 140;
/** Along a country road: a farmstead every so far (m), and none nearer an area's box than this. */
const FARM_EVERY: [number, number] = [150, 200];
const FARM_OFF = 25;
/** Cottages by the country roads: so many where each leaves an area or meets another, within this of the spot (m). */
const COTTAGES: [number, number] = [2, 4];
const COTTAGE_REACH = 70;
/** Land steeper than this (rise per m) is no place for a farmstead or a cottage. */
const STEEP = 0.25;
/** An orchard beside a farmstead: this deep from the lot's side (m), a gap of this from it. */
const ORCHARD = 26;
const ORCHARD_GAP = 3;
/** A home by a country road stands where it turns less than `BEND_MOST` (radians) over `BEND` m either side. */
const BEND = 16;
const BEND_MOST = 0.3;

const range = (rand: Rand, lo: number, hi: number) => lo + (hi - lo) * rand();
const between = (rand: Rand, [lo, hi]: [number, number]) => lo + Math.floor(rand() * (hi - lo + 1));
const dist = (a: Pt, b: Pt) => Math.hypot(a[0] - b[0], a[1] - b[1]);
const round2 = (v: number) => Math.round(v * 100) / 100;

/** The land: per 8 m cell, its height (m), whether it is water, whether it is wooded. */
class Land {
  readonly gw: number;
  readonly gd: number;
  readonly height: Float32Array;
  readonly water: Uint8Array;
  readonly woods: Uint8Array;

  constructor(
    readonly W: number,
    readonly D: number,
  ) {
    this.gw = Math.ceil(W / CELL);
    this.gd = Math.ceil(D / CELL);
    this.height = new Float32Array(this.gw * this.gd);
    this.water = new Uint8Array(this.gw * this.gd);
    this.woods = new Uint8Array(this.gw * this.gd);
  }

  cell(x: number, z: number): number {
    const cx = Math.max(0, Math.min(this.gw - 1, Math.floor(x / CELL)));
    const cz = Math.max(0, Math.min(this.gd - 1, Math.floor(z / CELL)));
    return cz * this.gw + cx;
  }

  centre(i: number): Pt {
    return [((i % this.gw) + 0.5) * CELL, (Math.floor(i / this.gw) + 0.5) * CELL];
  }

  at(x: number, z: number): number {
    return this.height[this.cell(x, z)];
  }

  /** How steep the land is at a cell (rise per metre). */
  slope(i: number): number {
    const [x, z] = [i % this.gw, Math.floor(i / this.gw)];
    const h = (dx: number, dz: number) => this.height[Math.max(0, Math.min(this.gd - 1, z + dz)) * this.gw + Math.max(0, Math.min(this.gw - 1, x + dx))];
    return Math.hypot(h(1, 0) - h(-1, 0), h(0, 1) - h(0, -1)) / (2 * CELL);
  }

  /** Whether any cell within `r` m of (x, z) is water. */
  wet(x: number, z: number, r: number): boolean {
    for (let cz = Math.floor((z - r) / CELL); cz <= Math.floor((z + r) / CELL); cz++) {
      for (let cx = Math.floor((x - r) / CELL); cx <= Math.floor((x + r) / CELL); cx++) {
        if (cx >= 0 && cz >= 0 && cx < this.gw && cz < this.gd && this.water[cz * this.gw + cx]) return true;
      }
    }
    return false;
  }
}

/** A binary heap of cell indices by priority. */
class Heap {
  private readonly items: number[] = [];
  private readonly keys: number[] = [];
  get size(): number {
    return this.items.length;
  }
  push(item: number, key: number): void {
    const { items, keys } = this;
    let i = items.push(item) - 1;
    keys.push(key);
    while (i > 0) {
      const p = (i - 1) >> 1;
      if (keys[p] <= key) break;
      [items[i], items[p], keys[i], keys[p]] = [items[p], items[i], keys[p], keys[i]];
      i = p;
    }
  }
  pop(): number {
    const { items, keys } = this;
    const top = items[0];
    const lastItem = items.pop()!;
    const lastKey = keys.pop()!;
    if (items.length) {
      items[0] = lastItem;
      keys[0] = lastKey;
      let i = 0;
      for (;;) {
        const [l, r] = [i * 2 + 1, i * 2 + 2];
        let m = i;
        if (l < items.length && keys[l] < keys[m]) m = l;
        if (r < items.length && keys[r] < keys[m]) m = r;
        if (m === i) break;
        [items[i], items[m], keys[i], keys[m]] = [items[m], items[i], keys[m], keys[i]];
        i = m;
      }
    }
    return top;
  }
}

/**
 * The cheapest way over a `gw` × `gd` grid (8 neighbours, cost per cell `cost`: Infinity where
 * blocked) from any of `starts` to a cell `goal` accepts, `h` guessing the rest: the cells, start first.
 */
function astar(gw: number, gd: number, cost: (i: number) => number, starts: readonly number[], goal: (i: number) => boolean, h: (i: number) => number, limit = 400000): number[] | null {
  const g = new Float32Array(gw * gd).fill(Infinity);
  const from = new Int32Array(gw * gd).fill(-1);
  const open = new Heap();
  for (const s of starts) {
    if (g[s] === 0) continue;
    g[s] = 0;
    open.push(s, h(s));
  }
  const closed = new Uint8Array(gw * gd);
  for (let steps = 0; open.size && steps < limit; steps++) {
    const i = open.pop();
    if (closed[i]) continue;
    closed[i] = 1;
    if (goal(i)) {
      const path = [i];
      while (from[path[path.length - 1]] >= 0) path.push(from[path[path.length - 1]]);
      return path.reverse();
    }
    const [x, z] = [i % gw, Math.floor(i / gw)];
    const here = cost(i);
    for (let dz = -1; dz <= 1; dz++) {
      for (let dx = -1; dx <= 1; dx++) {
        if (!dx && !dz) continue;
        const [nx, nz] = [x + dx, z + dz];
        if (nx < 0 || nz < 0 || nx >= gw || nz >= gd) continue;
        const j = nz * gw + nx;
        if (closed[j]) continue;
        const c = cost(j);
        if (c === Infinity) continue;
        // (No cutting a blocked corner.)
        if (dx && dz && (cost(z * gw + nx) === Infinity || cost(nz * gw + x) === Infinity)) continue;
        const ng = g[i] + ((here + c) / 2) * (dx && dz ? Math.SQRT2 : 1);
        if (ng < g[j]) {
          g[j] = ng;
          from[j] = i;
          open.push(j, ng + h(j));
        }
      }
    }
  }
  return null;
}

/** A polyline with the points that hardly change its course taken out (Douglas-Peucker, `tol` m). */
function simplify(pts: readonly Pt[], tol: number): Pt[] {
  if (pts.length < 3) return [...pts];
  const keep = new Uint8Array(pts.length);
  keep[0] = keep[pts.length - 1] = 1;
  const stack: [number, number][] = [[0, pts.length - 1]];
  while (stack.length) {
    const [a, b] = stack.pop()!;
    const [[ax, az], [bx, bz]] = [pts[a], pts[b]];
    const l = Math.hypot(bx - ax, bz - az) || 1;
    let [far, at] = [0, -1];
    for (let i = a + 1; i < b; i++) {
      const d = Math.abs((pts[i][0] - ax) * (bz - az) - (pts[i][1] - az) * (bx - ax)) / l;
      if (d > far) [far, at] = [d, i];
    }
    if (far > tol && at > 0) {
      keep[at] = 1;
      stack.push([a, at], [at, b]);
    }
  }
  return pts.filter((_, i) => keep[i]);
}

/** A closed polygon simplified as `simplify` does a polyline: in two halves, from its first point and the point furthest from it. */
function simplifyRing(ring: readonly Pt[], tol: number): Pt[] {
  if (ring.length < 4) return [...ring];
  let far = 0;
  for (let i = 1; i < ring.length; i++) if (dist(ring[i], ring[0]) > dist(ring[far], ring[0])) far = i;
  const a = simplify(ring.slice(0, far + 1), tol);
  const b = simplify([...ring.slice(far), ring[0]], tol);
  return [...a, ...b.slice(1, -1)];
}

/** A closed polygon with its corners cut (Chaikin), once. */
function chaikin(poly: readonly Pt[]): Pt[] {
  const out: Pt[] = [];
  for (let i = 0; i < poly.length; i++) {
    const [a, b] = [poly[i], poly[(i + 1) % poly.length]];
    out.push([a[0] * 0.75 + b[0] * 0.25, a[1] * 0.75 + b[1] * 0.25], [a[0] * 0.25 + b[0] * 0.75, a[1] * 0.25 + b[1] * 0.75]);
  }
  return out;
}

const area2 = (poly: readonly Pt[]) => poly.reduce((n, p, i) => n + p[0] * poly[(i + 1) % poly.length][1] - poly[(i + 1) % poly.length][0] * p[1], 0);

/**
 * The outlines of a cell mask as polygons (m), smoothed: each piece of land one ring, its holes
 * joined to it by a seam (so filled even-odd, as `inside` does, the holes stay open).
 */
function outlines(mask: Uint8Array, gw: number, gd: number, minCells = 6): Pt[][] {
  const on = (x: number, z: number) => x >= 0 && z >= 0 && x < gw && z < gd && mask[z * gw + x] === 1;
  // Edges round each set cell where its neighbour isn't, clockwise (z down the page): corner keys.
  const key = (x: number, z: number) => z * (gw + 1) + x;
  const next = new Map<number, number[]>();
  const edge = (ax: number, az: number, bx: number, bz: number) => {
    const k = key(ax, az);
    const list = next.get(k);
    if (list) list.push(key(bx, bz));
    else next.set(k, [key(bx, bz)]);
  };
  let cells = 0;
  for (let z = 0; z < gd; z++) {
    for (let x = 0; x < gw; x++) {
      if (!on(x, z)) continue;
      cells++;
      if (!on(x, z - 1)) edge(x, z, x + 1, z);
      if (!on(x + 1, z)) edge(x + 1, z, x + 1, z + 1);
      if (!on(x, z + 1)) edge(x + 1, z + 1, x, z + 1);
      if (!on(x - 1, z)) edge(x, z + 1, x, z);
    }
  }
  if (!cells) return [];
  const loops: Pt[][] = [];
  for (const start of [...next.keys()]) {
    while (next.get(start)?.length) {
      const loop: Pt[] = [];
      let k = start;
      do {
        loop.push([(k % (gw + 1)) * CELL, Math.floor(k / (gw + 1)) * CELL]);
        const list = next.get(k)!;
        const n = list.pop()!;
        if (!list.length) next.delete(k);
        k = n;
      } while (k !== start && next.has(k));
      // Straight runs as one edge.
      const corners = loop.filter((p, i) => {
        const [a, b] = [loop[(i + loop.length - 1) % loop.length], loop[(i + 1) % loop.length]];
        return (b[0] - p[0]) * (p[1] - a[1]) !== (b[1] - p[1]) * (p[0] - a[0]);
      });
      if (corners.length >= 3 && Math.abs(area2(corners)) / 2 >= minCells * CELL * CELL) loops.push(chaikin(chaikin(corners)));
    }
  }
  const outer = loops.filter((l) => area2(l) > 0).sort((a, b) => area2(a) - area2(b));
  const holes = loops.filter((l) => area2(l) < 0);
  const rings = outer.map((l) => [...l]);
  for (const hole of holes) {
    const o = rings.findIndex((ring, i) => inside(outer[i], hole[0][0], hole[0][1]) && ring.length);
    if (o < 0) continue;
    const ring = rings[o];
    let [bi, bj, bd] = [0, 0, Infinity];
    for (let i = 0; i < ring.length; i += 2) {
      for (let j = 0; j < hole.length; j += 2) {
        const d = dist(ring[i], hole[j]);
        if (d < bd) [bi, bj, bd] = [i, j, d];
      }
    }
    rings[o] = [...ring.slice(0, bi + 1), ...hole.slice(bj), ...hole.slice(0, bj + 1), ...ring.slice(bi)];
  }
  return rings.map((r) => simplifyRing(r, 0.6).map(([x, z]): Pt => [round2(x), round2(z)]));
}

/** Tiles (1 m) of a `W` × `D` world inside any of `polys` (even-odd, at the tiles' middles). */
function rasterize(polys: readonly Pt[][], W: number, D: number): Uint8Array {
  const mask = new Uint8Array(W * D);
  for (const poly of polys) {
    for (let z = 0; z < D; z++) {
      const zc = z + 0.5;
      const xs: number[] = [];
      for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
        const [[xi, zi], [xj, zj]] = [poly[i], poly[j]];
        if (zi > zc !== zj > zc) xs.push(((xj - xi) * (zc - zi)) / (zj - zi) + xi);
      }
      xs.sort((a, b) => a - b);
      for (let k = 0; k + 1 < xs.length; k += 2) {
        for (let x = Math.max(0, Math.ceil(xs[k] - 0.5)); x < Math.min(W, Math.ceil(xs[k + 1] - 0.5)); x++) mask[z * W + x] ^= 1;
      }
    }
  }
  return mask;
}

/** A 0/1 mask, run-length (`Rle`): [run, value, run, value, ...] from tile 0. */
export function rle(mask: Uint8Array): Rle {
  const out: Rle = [];
  let [run, value] = [0, mask[0] ?? 0];
  for (const v of mask) {
    if (v === value) run++;
    else {
      out.push(run, value);
      [run, value] = [1, v];
    }
  }
  if (run) out.push(run, value);
  return out;
}

/** A run-length mask (`Rle`) back to one value per tile. */
export function unrle(runs: Rle, size: number): Uint8Array {
  const mask = new Uint8Array(size);
  let at = 0;
  for (let i = 0; i + 1 < runs.length; i += 2) {
    if (runs[i + 1]) mask.fill(1, at, Math.min(size, at + runs[i]));
    at += runs[i];
  }
  return mask;
}

/** An area as it is laid out: its box on the world, its roads (indices into the world's), its middle. */
interface Plan {
  name: string;
  rect: Rect;
  centre: Pt;
  roads: number[];
  greens: Pt[][];
  spec: SizeSpec;
  /** Its ways on: road ends that led out of its box. */
  ports: { road: number; end: 0 | 1; p: Pt; dir: Pt; used: boolean }[];
}

/**
 * Lays out a big world of `kind` from `seed` (the same kind and seed always give the same world).
 * See the top of this file.
 */
export function generateBigWorld(content: Content, t: Templates, kind: WorldKind, seed: number, name?: string): NeighbourhoodDraft {
  const recipe = WORLDS[kind];
  const ch = CHARACTER[kind];
  const rand = random(seed);
  const rolledName = worldName(content, kind, rand);
  const tagline = worldTagline(content, kind, rand);
  const W = Math.round(range(rand, ...ch.world) / 64) * 64;
  const D = W;
  const land = new Land(W, D);
  const { gw, gd } = land;
  const nseed = (seed % 100000) + 17;

  // ---- the land: hills (a valley in the mountains), woods ------------------------------------
  const valleyZ = (x: number) => D / 2 + (fbm(x / 420, 3.7, nseed + 3, 3) - 0.47) * 260;
  for (let i = 0; i < gw * gd; i++) {
    const [x, z] = land.centre(i);
    let h = fbm(x / 230, z / 230, nseed, 4) * ch.hills * 2;
    if (ch.valley) h += 120 * Math.min(1, Math.max(0, (Math.abs(z - valleyZ(x)) - 70) / 260)) ** 1.4;
    land.height[i] = h;
  }

  // ---- the main town: its roads first (which side its water is on), then where it goes -------
  const mainSpec = recipe.sizes.big;
  const mainShape = ch.main ?? recipe.roads;
  const layout = mainShape(rand, mainSpec.box, mainSpec.scale);
  const mainRoads = settleRoads(joinUp(rand, layout, mainSpec, BIG_COURTS), mainSpec.box);
  const box = mainSpec.box;
  const waterSide = layout.water ?? (rand() < 0.5 ? 'zmin' : 'zmax');

  // The sea along one edge, a lake in the middle.
  if (ch.water === 'sea') {
    const deep = range(rand, 120, 160);
    for (let i = 0; i < gw * gd; i++) {
      const [x, z] = land.centre(i);
      const shore = deep + (fbm(x / 160, 9.1, nseed + 5, 3) - 0.47) * 90;
      if (waterSide === 'zmax' ? z > D - shore : z < shore) land.water[i] = 1;
    }
  }
  const lake: Pt = [W / 2 + range(rand, -50, 50), D / 2 + range(rand, -40, 40)];
  if (ch.water === 'lake') {
    const [rx, rz] = [range(rand, 150, 185), range(rand, 95, 120)];
    for (let i = 0; i < gw * gd; i++) {
      const [x, z] = land.centre(i);
      const r = ((x - lake[0]) / rx) ** 2 + ((z - lake[1]) / rz) ** 2;
      if (r < 1 + (fbm(x / 90, z / 90, nseed + 6, 3) - 0.47) * 0.7) land.water[i] = 1;
    }
  }
  /** The first dry z going from `from` towards `dir` across x0..x1 (the shore nearest the town). */
  const shoreAt = (x0: number, x1: number, dir: 1 | -1, from: number): number => {
    let best = dir > 0 ? Infinity : -Infinity;
    for (let x = Math.max(0, x0); x <= Math.min(W - 1, x1); x += CELL) {
      for (let z = from; z >= 0 && z < D; z += dir * CELL) {
        if (land.water[land.cell(x, z)]) {
          best = dir > 0 ? Math.min(best, z) : Math.max(best, z);
          break;
        }
      }
    }
    return best;
  };

  // Where the main town's box goes.
  let ox = Math.round(W / 2 - box.w / 2 + range(rand, -0.08, 0.08) * W);
  let oz = Math.round(D / 2 - box.d / 2 + range(rand, -0.08, 0.08) * D);
  ox = Math.max(EDGE, Math.min(W - EDGE - box.w, ox));
  if (ch.water === 'sea' || ch.water === 'lake') {
    // Its water side on the shore (its first road runs along it): looking for the water from the land beyond the town.
    const dir = waterSide === 'zmax' ? 1 : -1;
    const from = Math.max(0, Math.min(D - 1, (ch.water === 'sea' ? D / 2 : lake[1]) - dir * 260));
    const shore = shoreAt(ox, ox + box.w, dir, from);
    if (Number.isFinite(shore)) oz = Math.round(dir > 0 ? shore - 4 - box.d : shore + 4);
  } else if (ch.valley) oz = Math.round(valleyZ(ox + box.w / 2) - box.d / 2);
  oz = Math.max(EDGE / 2, Math.min(D - EDGE / 2 - box.d, oz));

  const roads: RoadPlan[] = [];
  const plans: Plan[] = [];
  const takenNames: string[] = [];
  /** An area's roads onto the world: moved to (`x`, `z`), turned across (`swap`), kept as the area's. */
  const addArea = (areaName: string, laid: readonly RoadPlan[], local: Box, x: number, z: number, swap: boolean, spec: SizeSpec, greens: Pt[][], centre: Pt) => {
    const onto = ([px, pz]: Pt): Pt => (swap ? [x + pz, z + px] : [x + px, z + pz]);
    const index = plans.length;
    const plan: Plan = {
      name: areaName,
      rect: swap ? { x, z, w: local.d, d: local.w } : { x, z, w: local.w, d: local.d },
      centre: onto(centre),
      roads: [],
      greens: greens.map((g) => g.map(onto)),
      spec,
      ports: [],
    };
    laid.forEach((r, k) => {
      const points = r.points.map(onto);
      const road: RoadPlan = { ...r, points, kind: k === 0 ? 'street' : 'lane', area: index, ...(r.lots && swap ? { lots: (-r.lots) as 1 | -1 } : {}) };
      const at = roads.push(road) - 1;
      plan.roads.push(at);
      for (const e of [0, 1] as const) {
        if (r.ends[e] !== 'out') continue;
        const [p, q] = e === 0 ? [points[0], points[1]] : [points[points.length - 1], points[points.length - 2]];
        const l = dist(p, q) || 1;
        plan.ports.push({ road: at, end: e, p, dir: [(p[0] - q[0]) / l, (p[1] - q[1]) / l], used: false });
      }
    });
    plans.push(plan);
  };
  addArea(name ?? rolledName, mainRoads, box, ox, oz, false, mainSpec, layout.greens ?? [], layout.centre ?? mainRoads[0].points[Math.floor(mainRoads[0].points.length / 2)]);

  // ---- hamlets: where it's dry and flat, far enough from the others, near enough to one --------
  const mainCentre = plans[0].centre;
  const hamlets = between(rand, ch.hamlets);
  const candidates: Pt[] = Array.from({ length: 700 }, (): Pt => [range(rand, EDGE + 80, W - EDGE - 80), range(rand, EDGE + 60, D - EDGE - 60)]);
  /** Which quarter of the world a point is in: the hamlets go first to those without an area. */
  const quarter = (p: Pt) => (p[0] < W / 2 ? 0 : 1) + (p[1] < D / 2 ? 0 : 2);
  for (let n = 0, relax = 0; n < hamlets; n++) {
    const held = new Set(plans.map((a) => quarter(a.centre)));
    const swap = rand() < 0.35;
    const hb = ch.hamlet.box;
    const [bw, bd] = swap ? [hb.d, hb.w] : [hb.w, hb.d];
    let best: { p: Pt; score: number } | null = null;
    // (No room where it should be: a little further, on steeper land.)
    for (const p of relax ? [...candidates].reverse() : candidates) {
      const rect = { x: Math.round(p[0] - bw / 2), z: Math.round(p[1] - bd / 2), w: bw, d: bd };
      if (rect.x < EDGE || rect.z < EDGE || rect.x + bw > W - EDGE || rect.z + bd > D - EDGE) continue;
      const near = Math.min(...plans.map((a) => dist(a.centre, p)));
      if (near < AREA_APART || near > AREA_REACH + relax * 90) continue;
      // Clear of the other areas' boxes, dry, not too steep.
      if (plans.some((a) => rect.x < a.rect.x + a.rect.w + BOX_GAP && a.rect.x < rect.x + rect.w + BOX_GAP && rect.z < a.rect.z + a.rect.d + BOX_GAP && a.rect.z < rect.z + rect.d + BOX_GAP)) continue;
      if (land.wet(p[0], p[1], Math.max(bw, bd) / 2 + 30)) continue;
      let steep = 0;
      for (let z = rect.z; z < rect.z + bd; z += CELL * 2) for (let x = rect.x; x < rect.x + bw; x += CELL * 2) steep = Math.max(steep, land.slope(land.cell(x, z)));
      if (steep > 0.35 - relax * 0.1 && !relax) continue;
      let score = Math.abs(near - AREA_AIM) / 40 + steep * 40 + land.at(p[0], p[1]) * (ch.valley ? 0.08 : 0.02) + (held.has(quarter(p)) ? 0 : -5);
      // A fishing cove on the shore; a hamlet across the lake from the town.
      if (n === 0 && ch.water === 'sea') score += land.wet(p[0], p[1], Math.max(bw, bd) / 2 + 70) ? -6 : 6;
      if (n === 0 && ch.water === 'lake') score += ((p[0] - lake[0]) * (mainCentre[0] - lake[0]) + (p[1] - lake[1]) * (mainCentre[1] - lake[1]) < 0 ? -6 : 6) + (land.wet(p[0], p[1], Math.max(bw, bd) / 2 + 60) ? -3 : 3);
      score += rand() * 0.5;
      if (!best || score < best.score) best = { p, score };
    }
    if (!best) {
      if (relax++ < 1) n--;
      continue;
    }
    const { shapes, scale: hs, fill, extend } = ch.hamlet;
    const local = shapes[(plans.length - 1) % shapes.length](rand, hb, hs);
    const laid = settleRoads(joinUp(rand, local, { box: hb, lots: 0, scale: hs, fill, extend }, COURTS), hb);
    let areaName = worldName(content, kind, rand);
    for (let k = 0; k < 12 && [...plans.map((a) => a.name)].includes(areaName); k++) areaName = worldName(content, kind, rand);
    if (plans.some((a) => a.name === areaName)) areaName = `${areaName} End`;
    const spec: SizeSpec = { box: hb, lots: between(rand, ch.hamlet.lots), scale: hs, places: ch.hamlet.places, farms: ch.hamlet.farms ?? [0, 0] };
    addArea(areaName, laid, hb, Math.round(best.p[0] - bw / 2), Math.round(best.p[1] - bd / 2), swap, spec, local.greens ?? [], local.centre ?? [hb.w / 2, hb.d / 2]);
  }

  // Fewer hamlets than a world of this kind has at least: their houses go to the others (the main town first; `LotPlacer` spills them on).
  const missing = ch.hamlets[0] - (plans.length - 1);
  if (missing > 0) plans[0].spec = { ...mainSpec, lots: mainSpec.lots + missing * ch.hamlet.lots[0] };

  // ---- country roads: between the areas (a tree of them and one more for a loop), and out ----
  const areaBlock = new Uint8Array(gw * gd);
  for (const [a, plan] of plans.entries()) {
    const r = plan.rect;
    for (let z = Math.floor((r.z - 6) / CELL); z <= Math.floor((r.z + r.d + 6) / CELL); z++) {
      for (let x = Math.floor((r.x - 6) / CELL); x <= Math.floor((r.x + r.w + 6) / CELL); x++) if (x >= 0 && z >= 0 && x < gw && z < gd) areaBlock[z * gw + x] = a + 1;
    }
  }
  const rim = new Uint8Array(gw * gd);
  for (let i = 0; i < gw * gd; i++) if (i % gw < 2 || i % gw >= gw - 2 || i < gw * 2 || i >= gw * (gd - 2)) rim[i] = 1;
  const wetNear = new Uint8Array(gw * gd);
  for (let i = 0; i < gw * gd; i++) if (land.water[i]) for (const d of [0, -1, 1, -gw, gw, -gw - 1, -gw + 1, gw - 1, gw + 1]) if (i + d >= 0 && i + d < gw * gd) wetNear[i + d] = 1;
  const roadCost = (from: Pt | null, to: Pt | null) => (i: number) => {
    if (wetNear[i]) return Infinity;
    if (areaBlock[i]) {
      const c = land.centre(i);
      if (!((from && dist(c, from) < 26) || (to && dist(c, to) < 26))) return Infinity;
    }
    // (Not along the world's edge: smoothed, it would wander out of the world and back.)
    return 1 + land.slope(i) * 40 + land.woods[i] * 0.3 + (rim[i] ? 12 : 0);
  };
  /** As `roadCost`, but along the water's edge too (not over it), and less put off by the slope: for when that finds no way. */
  const relaxedCost = (from: Pt | null, to: Pt | null) => {
    const strict = roadCost(from, to);
    return (i: number) => (land.water[i] ? Infinity : wetNear[i] ? 30 + land.slope(i) * 15 : Math.min(strict(i), 1 + land.slope(i) * 15 + land.woods[i] * 0.3));
  };
  /** The way `astar` finds with `roadCost`; if none, with `relaxedCost`. */
  const route = (from: Pt | null, to: Pt | null, starts: number[], goal: (i: number) => boolean, h: (i: number) => number) =>
    astar(gw, gd, roadCost(from, to), starts, goal, h) ?? astar(gw, gd, relaxedCost(from, to), starts, goal, h);
  /** Country roads the layout found no way for. */
  const failed: string[] = [];
  /** A country road from `a` (a port) along `cells`, to `b`: smoothed into a curve, leaving and arriving straight. */
  const countryRoad = (a: { p: Pt; dir: Pt }, cells: number[], b: { p: Pt; dir: Pt } | null, ends: RoadPlan['ends'], roadName: string): RoadPlan => {
    const mid = simplify(cells.map((i) => land.centre(i)), 5).slice(1, b ? -1 : undefined);
    const lead = (q: { p: Pt; dir: Pt }, k: number): Pt => [q.p[0] + q.dir[0] * k, q.p[1] + q.dir[1] * k];
    // Near an end, only the way's points out in front of it (no doubling back to it: a kink).
    const before = (q: { p: Pt; dir: Pt }, p: Pt, near: number) => dist(p, q.p) > near && (dist(p, q.p) > 48 || q.dir[0] * (p[0] - q.p[0]) + q.dir[1] * (p[1] - q.p[1]) > dist(p, q.p) * 0.5);
    // (No lead out over the water: a port on the shore is come to from the land.)
    const leads = (q: { p: Pt; dir: Pt }, k: number): Pt[] => (land.wet(...lead(q, k), 4) ? [] : [lead(q, k)]);
    const make = (k: number) => {
      const tail: Pt[] = b ? [...leads(b, k), b.p] : [];
      const ctrl = [a.p, ...leads(a, k), ...mid.filter((p) => before(a, p, k + 8) && (!b || before(b, p, k + 8))), ...tail];
      return curve(ctrl);
    };
    // (Shorter leads if the longer ones swing it out over the water.)
    let points = make(14);
    for (const k of [7, 3]) if (points.some(([x, z]) => land.wet(x, z, 2))) points = make(k);
    return { points, width: COUNTRY_WIDTH, ends, kind: 'country', name: roadName };
  };
  const portOf = (from: number, to: Pt) =>
    plans[from].ports
      .map((q) => ({ q, score: dist(q.p, to) - (q.dir[0] * (to[0] - q.p[0]) + q.dir[1] * (to[1] - q.p[1])) / (dist(q.p, to) || 1) * 60 + (q.used ? 120 : 0) }))
      .sort((x, y) => x.score - y.score)[0]?.q;
  /**
   * Where a country road leaves port `q` towards `toward`: the port itself; one another country
   * road leaves already, off that road 30 m out from it, square to it (rather than both from the
   * port, running together a while).
   */
  const via = (q: { p: Pt; dir: Pt; used: boolean }, toward: Pt): { p: Pt; dir: Pt } => {
    if (!q.used) return q;
    const r = roads.find((o) => o.kind === 'country' && [o.points[0], o.points[o.points.length - 1]].some((e) => dist(e, q.p) < 1));
    if (!r) return q;
    const ls = lengths(r.points);
    const L = ls[ls.length - 1];
    if (L < 70) return q;
    const s = dist(r.points[0], q.p) < 1 ? 30 : L - 30;
    const { p, t } = pointAt(r.points, s, ls);
    const n = leftOf(t);
    const k = n[0] * (toward[0] - p[0]) + n[1] * (toward[1] - p[1]) >= 0 ? 1 : -1;
    return { p, dir: [n[0] * k, n[1] * k] };
  };
  // Prim's tree over the areas, then the shortest edge it left out.
  const inTree = [0];
  const edges: [number, number][] = [];
  while (inTree.length < plans.length) {
    let best: [number, number, number] | null = null;
    for (const a of inTree) for (let b = 0; b < plans.length; b++) if (!inTree.includes(b) && (!best || dist(plans[a].centre, plans[b].centre) < best[2])) best = [a, b, dist(plans[a].centre, plans[b].centre)];
    edges.push([best![0], best![1]]);
    inTree.push(best![1]);
  }
  // (A loop or two: more of them in a world of more areas.)
  for (let loops = plans.length >= 6 ? 2 : plans.length >= 3 ? 1 : 0; loops > 0; loops--) {
    let extra: [number, number, number] | null = null;
    for (let a = 0; a < plans.length; a++) {
      for (let b = a + 1; b < plans.length; b++) {
        if (edges.some(([x, y]) => (x === a && y === b) || (x === b && y === a))) continue;
        const d = dist(plans[a].centre, plans[b].centre);
        if (d < 560 && (!extra || d < extra[2])) extra = [a, b, d];
      }
    }
    if (extra) edges.push([extra[0], extra[1]]);
  }
  const roadNames = new Set<string>();
  const uniqueName = (base: string, alt: string) => {
    const pick = !roadNames.has(base) ? base : !roadNames.has(alt) ? alt : `${alt} ${roadNames.size}`;
    roadNames.add(pick);
    return pick;
  };
  for (const [a, b] of edges) {
    const qa = portOf(a, plans[b].centre);
    const qb = portOf(b, plans[a].centre);
    if (!qa || !qb) continue;
    const [pa, pb] = [via(qa, qb.p), via(qb, qa.p)];
    const goal = land.cell(pb.p[0] + pb.dir[0] * 8, pb.p[1] + pb.dir[1] * 8);
    const [gx, gz] = land.centre(goal);
    const cells = route(qa.p, qb.p, [land.cell(pa.p[0] + pa.dir[0] * 8, pa.p[1] + pa.dir[1] * 8)], (i) => i === goal, (i) => dist(land.centre(i), [gx, gz]) / CELL);
    if (!cells) {
      failed.push(`no country road between ${plans[a].name} and ${plans[b].name}`);
      continue;
    }
    const far = b === 0 ? a : a === 0 ? b : dist(plans[a].centre, mainCentre) > dist(plans[b].centre, mainCentre) ? a : b;
    const near = far === a ? b : a;
    roads.push(countryRoad(pa, cells, pb, ['join', 'join'], uniqueName(`${plans[far].name} Road`, `${plans[near].name} Lane`)));
    qa.used = qb.used = true;
  }
  // Two or three ways out of the world: from the ports left over (else any), to the nearest edge.
  const outs = 2 + Math.floor(rand() * 2);
  const edgeName = ['North', 'East', 'South', 'West'];
  const usedEdges = new Set<number>();
  /** Ways out tried that found no way (a port, an edge: across the sea, say). */
  const noWay = new Map<object, Set<number>>();
  let made = 0;
  /** Each edge of the world (north, east, south, west): whether any of it is dry land. */
  const dryEdge = [0, 1, 2, 3].map((side) => {
    for (let k = 0; k < (side % 2 ? gd : gw); k++) {
      const i = [k, k * gw + gw - 1, (gd - 1) * gw + k, k * gw][side];
      if (!wetNear[i]) return true;
    }
    return false;
  });
  for (let tries = 0; made < outs && tries < outs + 12; tries++) {
    const ports = plans.flatMap((plan) => plan.ports).sort((x, y) => Number(x.used) - Number(y.used));
    let best: { q: (typeof ports)[number]; side: number; d: number } | null = null;
    for (const q of ports) {
      for (const [side, d] of [q.p[1], W - q.p[0], D - q.p[1], q.p[0]].entries()) {
        if (!dryEdge[side] || noWay.get(q)?.has(side)) continue;
        const facing = [[0, -1], [1, 0], [0, 1], [-1, 0]][side];
        const score = d - (q.dir[0] * facing[0] + q.dir[1] * facing[1]) * 80 + (q.used ? 400 : 0) + (usedEdges.has(side) ? 500 : 0);
        if (!best || score < best.d) best = { q, side, d: score };
      }
    }
    if (!best) break;
    const { q, side } = best;
    const from = via(q, [q.p[0] + [0, 400, 0, -400][side], q.p[1] + [-400, 0, 400, 0][side]]);
    const onEdge = (i: number) => [Math.floor(i / gw) === 0, i % gw === gw - 1, Math.floor(i / gw) === gd - 1, i % gw === 0][side];
    const cells = route(q.p, null, [land.cell(from.p[0] + from.dir[0] * 8, from.p[1] + from.dir[1] * 8)], onEdge, (i) => [Math.floor(i / gw), gw - 1 - (i % gw), gd - 1 - Math.floor(i / gw), i % gw][side]);
    if (!cells) {
      // (Another port or edge, then.)
      noWay.set(q, (noWay.get(q) ?? new Set()).add(side));
      continue;
    }
    const last = land.centre(cells[cells.length - 1]);
    const beyond: Pt = [last[0] + [0, 40, 0, -40][side], last[1] + [-40, 0, 40, 0][side]];
    const road = countryRoad(from, [...cells], null, ['join', 'out'], uniqueName(`${edgeName[side]} Road`, `${edgeName[side]} Lane`));
    road.points = curve([...simplify(road.points, 0.3), beyond]);
    roads.push(road);
    q.used = true;
    usedEdges.add(side);
    made++;
  }
  if (made < 2) failed.push(`${made} ways out of the world (two or more wanted)`);
  // Ports nobody took: a country road on from each, to the nearest country road, another area's
  // port nobody took or the edge of the world, whichever is nearest (a dead end only if none is).
  const plain = (p: Pt) => plans.every((a) => p[0] < a.rect.x - 30 || p[1] < a.rect.z - 30 || p[0] > a.rect.x + a.rect.w + 30 || p[1] > a.rect.z + a.rect.d + 30);
  const edgeOf = (i: number) => [Math.floor(i / gw) === 0, i % gw === gw - 1, Math.floor(i / gw) === gd - 1, i % gw === 0].indexOf(true);
  for (const [k, plan] of plans.entries()) {
    for (const q of plan.ports) {
      if (q.used) continue;
      const country = new Map<number, number>();
      roads.forEach((r, i) => {
        if (r.kind === 'country') for (const p of r.points) if (plain(p)) country.set(land.cell(p[0], p[1]), i);
      });
      const others = plans.flatMap((o, a) => (a === k ? [] : o.ports.filter((x) => !x.used)));
      const portAt = new Map(others.map((x) => [land.cell(x.p[0] + x.dir[0] * 8, x.p[1] + x.dir[1] * 8), x]));
      const goal = (i: number) => country.has(i) || portAt.has(i) || edgeOf(i) >= 0;
      // (Into the other areas only by their ports.)
      const cost = (strict: boolean) => {
        const c = (strict ? roadCost : relaxedCost)(q.p, null);
        return (i: number) => (areaBlock[i] && others.some((x) => dist(land.centre(i), x.p) < 26) ? 1 : c(i));
      };
      const start = [land.cell(q.p[0] + q.dir[0] * 8, q.p[1] + q.dir[1] * 8)];
      const cells = astar(gw, gd, cost(true), start, goal, () => 0) ?? astar(gw, gd, cost(false), start, goal, () => 0);
      const end = cells?.[cells.length - 1];
      if (!cells || end === undefined) {
        failed.push(`no country road on from ${plan.name}`);
        continue;
      }
      const roadName = uniqueName(`${plan.name} Road`, `${plan.name} Lane`);
      const to = portAt.get(end);
      const other = country.get(end);
      if (to) {
        roads.push(countryRoad(q, cells, to, ['join', 'join'], roadName));
        to.used = true;
      } else if (other !== undefined) {
        // Into the other road square to it, ending on its middle.
        // (Met from the side the way comes from: the last cell but one or two before it.)
        const c = cells.length >= 3 ? land.centre(cells[cells.length - 3]) : q.p;
        const pts = roads[other].points;
        const at = nearest(pts, c[0], c[1]);
        const T = pointAt(pts, at.s).p;
        const n = leftOf(pointAt(pts, at.s).t);
        roads.push(countryRoad(q, cells.slice(0, -1), { p: T, dir: [n[0] * at.side, n[1] * at.side] }, ['join', 'join'], roadName));
      } else {
        const side = edgeOf(end);
        const last = land.centre(end);
        const beyond: Pt = [last[0] + [0, 40, 0, -40][side], last[1] + [-40, 0, 40, 0][side]];
        const road = countryRoad(q, cells, null, ['join', 'out'], roadName);
        road.points = curve([...simplify(road.points, 0.3), beyond]);
        roads.push(road);
      }
      q.used = true;
    }
  }
  // Ports still nobody's (no way on from them): their roads end there.
  for (const plan of plans) {
    for (const q of plan.ports) {
      const road = roads[q.road];
      road.ends = q.end === 0 ? [q.used ? 'join' : 'dead', road.ends[1]] : [road.ends[0], q.used ? 'join' : 'dead'];
      if (q.used) continue;
      // A dead end stops inside its area (as a town's do), not out in the fields.
      const r = plan.rect;
      const within = ([x, z]: Pt) => x > r.x + 10 && z > r.z + 10 && x < r.x + r.w - 10 && z < r.z + r.d - 10;
      const pts = q.end === 0 ? [...road.points].reverse() : road.points;
      let keep = pts.length;
      while (keep > 2 && !within(pts[keep - 1])) keep--;
      if (keep < pts.length) road.points = q.end === 0 ? pts.slice(0, keep).reverse() : pts.slice(0, keep);
    }
  }

  // Ponds, out in the countryside.
  if (ch.water === 'ponds') {
    const ponds = kind === 'farm' ? 3 : kind === 'forest' ? 2 : 1 + Math.floor(rand() * 2);
    for (let n = 0, tries = 0; n < ponds && tries < 200; tries++) {
      const r = range(rand, 14, 24);
      const c: Pt = [range(rand, 60, W - 60), range(rand, 60, D - 60)];
      if (plans.some((a) => c[0] > a.rect.x - 50 - r && c[0] < a.rect.x + a.rect.w + 50 + r && c[1] > a.rect.z - 50 - r && c[1] < a.rect.z + a.rect.d + 50 + r)) continue;
      if (Math.min(...plans.map((a) => dist(a.centre, c))) > 200) continue;
      // (Clear of the roads.)
      if (roads.some((road) => nearest(road.points, c[0], c[1]).dist < r * 1.4 + road.width / 2 + 6)) continue;
      for (let i = 0; i < gw * gd; i++) {
        const [x, z] = land.centre(i);
        if (dist([x, z], c) < r * (0.85 + fbm(x / 20, z / 20, nseed + 8 + n, 2) * 0.5)) land.water[i] = 1;
      }
      n++;
    }
  }

  // ---- water and woods: zones, the mask the lots keep off -------------------------------------
  const waterPolys = outlines(land.water, gw, gd, 2);
  const waterMask = rasterize(waterPolys, W, D);
  // Woods: the most wooded share of the land by noise, not on the areas, the water or the roads.
  const woodsNoise = new Float32Array(gw * gd);
  for (let i = 0; i < gw * gd; i++) {
    const [x, z] = land.centre(i);
    woodsNoise[i] = fbm(x / 105, z / 105, nseed + 7, 4) + (ch.valley ? Math.min(0.25, land.height[i] / 400) : 0);
  }
  const cut = [...woodsNoise].sort((a, b) => b - a)[Math.floor(ch.woods * gw * gd)] ?? 1;
  for (let i = 0; i < gw * gd; i++) land.woods[i] = woodsNoise[i] > cut && !land.water[i] ? 1 : 0;

  // ---- the lots: one ground over the world, each area's along its own roads -------------------
  const worldRoads = roads;
  const allGreens = plans.flatMap((p) => p.greens);
  const ground = new Ground(W, D, worldRoads, allGreens);
  ground.block(waterMask);
  const lots = new LotPlacer(content, t, ground, worldRoads, recipe.gaps, rand, true);
  const names: (string | undefined)[] = worldRoads.map((r) => r.name);
  const fronts = new Map<number, ReturnType<typeof frontageOf>>();
  const left: { u: Parameters<typeof lots.place>[0][number]; from: number }[] = [];
  for (const [a, plan] of plans.entries()) {
    const wanted = wantedLots(content, t, kind, plan.spec, rand, { park: a === 0, avoid: a === 0 ? [] : ['market'] });
    fronts.set(a, frontageOf(worldRoads, plan.roads, plan.centre, rand));
    for (const u of lots.place(wanted, fronts.get(a)!, plan.greens, plan.centre, a, true)) left.push({ u, from: a });
    const streets = streetNames(content, kind, rand, plan.roads.length, [...takenNames, ...roadNames], plan.name);
    takenNames.push(...streets);
    plan.roads.forEach((r, k) => (names[r] = streets[k]));
  }
  // Houses with no room in their own area go to the nearest other that has room (homes, not shops).
  let country: ReturnType<typeof frontageOf> | undefined;
  for (const { u, from } of left) {
    const others = plans.map((_, a) => a).filter((a) => a !== from && u.kind === 'house').sort((x, y) => dist(plans[x].centre, plans[from].centre) - dist(plans[y].centre, plans[from].centre));
    const placed = others.some((a) => !lots.place([u], fronts.get(a)!, [], plans[a].centre, a, true).length);
    if (placed) continue;
    // Else along a country road, near its own area's end of it.
    country ??= frontageOf(worldRoads, worldRoads.flatMap((r, i) => (r.kind === 'country' ? [i] : [])), plans[0].centre, rand).filter((f) => straight(worldRoads[f.road].points, f.s, lengths(worldRoads[f.road].points)));
    const r = plans[from].rect;
    const off = (p: Pt) => Math.hypot(Math.max(r.x - p[0], 0, p[0] - r.x - r.w), Math.max(r.z - p[1], 0, p[1] - r.z - r.d));
    const near = country.filter((f) => off(f.p) < SPILL).sort((x, y) => dist(x.p, plans[from].centre) - dist(y.p, plans[from].centre));
    if (u.kind === 'house' && !lots.place([u], near, [], plans[from].centre, from, true).length) continue;
    lots.dropped.push({ name: u.name, kind: u.kind, lot: u.lot });
  }
  const farmsteads = alongCountryRoads(content, t, lots, plans, worldRoads, land, rand);
  lots.number(names);
  const slots = lots.slots;
  const rectOf = (s: PlotSlot) => plotRect(t, s);

  // ---- stops and parking in each area -------------------------------------------------------
  const stops: OpenWorldDraft['stops'] = [];
  const parking: OpenWorldDraft['parking'] = [];
  const index = new RoadIndex(worldRoads);
  /** The kerb of road `r` nearest (x, z): on its sidewalk. */
  const kerb = (r: number, x: number, z: number): Pt => {
    const road = worldRoads[r];
    const n = nearest(road.points, x, z);
    const ls = lengths(road.points);
    let i = 0;
    while (i < ls.length - 1 && ls[i] < n.s) i++;
    const [p, q] = [road.points[Math.max(0, i - 1)], road.points[Math.min(road.points.length - 1, i + 1)]];
    const l = dist(p, q) || 1;
    const left: Pt = [(q[1] - p[1]) / l, -(q[0] - p[0]) / l];
    const c = road.points[i];
    const off = road.width / 2 - 0.6;
    return [round2(c[0] + left[0] * off * n.side), round2(c[1] + left[1] * off * n.side)];
  };
  for (const [a, plan] of plans.entries()) {
    const mine = slots.filter((s) => (s.area ?? 0) === a);
    const centreRoad = plan.roads.map((r) => ({ r, d: index.distance(r, plan.centre[0], plan.centre[1], 400) })).sort((x, y) => x.d - y.d)[0];
    const here: Pt[] = [];
    if (centreRoad) here.push(kerb(centreRoad.r, plan.centre[0], plan.centre[1]));
    for (const s of mine) {
      if (s.kind !== 'house') continue;
      const f = lots.placed.get(s);
      const r = rectOf(s);
      const mid: Pt = [r.x + r.w / 2, r.z + r.d / 2];
      if (here.some((p) => dist(p, mid) <= STOP_REACH) || !f) continue;
      here.push(kerb(f.road, f.p[0], f.p[1]));
    }
    stops.push(...here.map(([x, z]) => ({ x, z, area: a })));
    // Parking: a few spaces by the verge nearest the middle.
    const frontage = frontageOf(worldRoads, plan.roads, plan.centre, rand);
    for (const f of frontage) {
      if (parking.filter((p) => p.area === a).length >= (a === 0 ? 4 : 2)) break;
      const facing = (Math.abs(f.n[1]) >= Math.abs(f.n[0]) ? (f.n[1] > 0 ? 0 : 2) : f.n[0] > 0 ? 1 : 3) as Facing;
      const reach = worldRoads[f.road].width / 2 + 0.5;
      const [qx, qz] = [f.p[0] + f.n[0] * (reach + 3), f.p[1] + f.n[1] * (reach + 3)];
      const r = facing % 2 ? { x: Math.round(qx - 3), z: Math.round(qz - 1.5), w: 6, d: 3 } : { x: Math.round(qx - 1.5), z: Math.round(qz - 3), w: 3, d: 6 };
      if (!ground.free(r, 8) || parking.some((p) => Math.hypot(p.x - r.x, p.z - r.z) < 12)) continue;
      ground.take(r, 1);
      parking.push({ x: r.x + r.w / 2, z: r.z + r.d / 2, rot: facing, area: a });
    }
  }

  // ---- outdoor places, a trail to each from its area's roads ----------------------------------
  const outdoor = new Map(t.parks.filter((p) => p.kind === 'outdoor').map((p) => [p.id, p]));
  const pads: Record<number, number> = {};
  let next = 0;
  for (const [a, plan] of plans.entries()) {
    const count = between(rand, ch.spotsPerArea) + (a === 0 ? 1 : 0);
    for (let n = 0; n < count; n++) {
      const id = ch.spots[next++ % ch.spots.length];
      const place = outdoor.get(id);
      if (!place) continue;
      const spot = siteFor(place, plan, land, ground, waterMask, t, rand, W, D);
      if (!spot) continue;
      const slot = lots.put({ kind: 'outdoor', template: place.id, name: `${plan.name} ${place.name}`, lot: 'medium', w: spot.w, d: spot.d, area: a }, spot, null);
      const r = rectOf(slot);
      pads[slot.index] = Math.round(land.at(r.x + r.w / 2, r.z + r.d / 2) * 10) / 10;
      const trail = trailTo(slot, plan, worldRoads, slots, waterMask, land, t, W, D);
      // (No way to it: no place there, then. It is the last lot put down.)
      if (!trail) {
        slots.pop();
        delete pads[slot.index];
        continue;
      }
      trail.name = uniqueName(`${slot.name} Trail`, `${slot.name} Path`);
      trail.area = a;
      worldRoads.push(trail);
      // (Nothing else goes down on it.)
      for (const [x, z] of trail.points) ground.take({ x: Math.floor(x) - 1, z: Math.floor(z) - 1, w: 3, d: 3 });
    }
  }

  // ---- orchards: rows of fruit trees beside each farm (behind it, else to one side) ------------
  const orchards: Rect[] = [];
  const wetIn = (r: Rect) => [0, 0.5, 1].some((u) => [0, 0.5, 1].some((v) => land.wet(r.x + r.w * u, r.z + r.d * v, 0)));
  for (const s of slots) {
    if (s.lot !== 'farm') continue;
    const r = rectOf(s);
    const [g, o] = [ORCHARD_GAP, ORCHARD];
    // Behind the lot (away from its road: the front is its min z, min x, max z, max x as it faces 0..3), then its sides.
    const back = [{ x: r.x, z: r.z + r.d + g, w: r.w, d: o }, { x: r.x + r.w + g, z: r.z, w: o, d: r.d }, { x: r.x, z: r.z - g - o, w: r.w, d: o }, { x: r.x - g - o, z: r.z, w: o, d: r.d }];
    const tries = [back[s.facing], back[(s.facing + 1) % 4], back[(s.facing + 3) % 4]];
    const at = tries.find((q) => ground.free(q, 8) && !wetIn(q));
    if (!at) continue;
    ground.take(at, 2);
    orchards.push(at);
  }

  // ---- zones -------------------------------------------------------------------------------
  const clearAround = (mask: Uint8Array, r: Rect, grow: number) => {
    for (let z = Math.floor((r.z - grow) / CELL); z <= Math.floor((r.z + r.d + grow) / CELL); z++) for (let x = Math.floor((r.x - grow) / CELL); x <= Math.floor((r.x + r.w + grow) / CELL); x++) if (x >= 0 && z >= 0 && x < gw && z < gd) mask[z * gw + x] = 0;
  };
  const woods = land.woods.slice();
  for (const plan of plans) clearAround(woods, plan.rect, 10);
  for (const s of slots) clearAround(woods, rectOf(s), 4);
  for (const r of orchards) clearAround(woods, r, 4);
  for (const road of worldRoads) for (let i = 0; i < road.points.length; i += 3) clearAround(woods, { x: road.points[i][0], z: road.points[i][1], w: 0, d: 0 }, road.width / 2 + 3);
  const heights = [...land.height].sort((x, y) => x - y);
  const hillCut = heights[Math.floor(heights.length * (ch.valley ? 0.62 : 0.82))];
  const hills = new Uint8Array(gw * gd);
  const meadow = new Uint8Array(gw * gd);
  const fields = new Uint8Array(gw * gd);
  /** Farmland: round each area's box (as far as `ch.fields` past it) and each farmstead (a little less from its middle). */
  const reach = ch.fields ?? 0;
  const farmMiddles = farmsteads.map((s): Pt => {
    const r = rectOf(s);
    return [r.x + r.w / 2, r.z + r.d / 2];
  });
  const offRect = (r: Rect, [x, z]: Pt) => Math.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d));
  const nearFarmland = (p: Pt) =>
    plans.some((a) => offRect(a.rect, p) < reach) || farmMiddles.some((m) => dist(m, p) < reach * 1.1);
  for (let i = 0; i < gw * gd; i++) {
    if (land.water[i]) continue;
    const [x, z] = land.centre(i);
    if (land.height[i] > hillCut) hills[i] = 1;
    if (!woods[i] && !areaBlock[i] && fbm(x / 70, z / 70, nseed + 11, 3) > 0.58) meadow[i] = 1;
    if (ch.fields && !woods[i] && !areaBlock[i] && !meadow[i] && nearFarmland([x, z]) && fbm(x / 110, z / 110, nseed + 13, 3) > (kind === 'farm' ? 0.32 : 0.4)) fields[i] = 1;
  }
  for (const plan of plans) clearAround(hills, plan.rect, 0);
  for (const road of worldRoads) {
    for (let i = 0; i < road.points.length; i += 3) {
      const at = { x: road.points[i][0], z: road.points[i][1], w: 0, d: 0 };
      clearAround(fields, at, road.width / 2 + 2);
      clearAround(meadow, at, road.width / 2 + 1);
    }
  }
  for (const r of [...slots.map(rectOf), ...orchards]) {
    clearAround(fields, r, 2);
    clearAround(meadow, r, 2);
  }
  const zones: WorldZone[] = [
    ...waterPolys.map((poly): WorldZone => ({ kind: 'water', poly })),
    ...outlines(hills, gw, gd, 12).map((poly): WorldZone => ({ kind: 'hill', poly })),
    ...outlines(fields, gw, gd, 10).map((poly): WorldZone => ({ kind: 'fields', poly })),
    ...outlines(meadow, gw, gd, 6).map((poly): WorldZone => ({ kind: 'meadow', poly })),
    ...outlines(woods, gw, gd, 8).map((poly): WorldZone => ({ kind: 'woods', poly })),
    ...orchards.map((r): WorldZone => ({ kind: 'orchard', poly: [[r.x, r.z], [r.x + r.w, r.z], [r.x + r.w, r.z + r.d], [r.x, r.z + r.d]] })),
  ];

  const areas: TownArea[] = plans.map((p) => ({ name: p.name, kind: 'hamlet', x: p.rect.x, z: p.rect.z, w: p.rect.w, d: p.rect.d }));
  // (The landscape round the world: its sea or lake is in the zones now.)
  const { water: _water, ...landscape } = recipe.landscape(rand, W, D, waterSide);
  return {
    name: name ?? rolledName,
    width: W,
    depth: D,
    slots,
    roads: cropRoads(worldRoads, 0, 0, W, D),
    greens: allGreens.map((g) => g.map(([x, z]): Pt => [round2(x), round2(z)])),
    households: [],
    seed,
    world: { kind, size: 'big', tagline, landscape },
    diagnostics: { dropped: lots.dropped, ...(failed.length ? { roads: failed } : {}) },
    open: { areas, zones, water: rle(waterMask), pads, stops, parking },
  };
}

/** A country road runs nearly straight round arc length `s` (within `BEND` m either way): a car turns in off it without swinging off the road. */
function straight(pts: readonly Pt[], s: number, ls: number[]): boolean {
  const L = ls[ls.length - 1];
  const [a, b] = [pointAt(pts, Math.max(0, s - BEND), ls).t, pointAt(pts, Math.min(L, s + BEND), ls).t];
  return a[0] * b[0] + a[1] * b[1] > Math.cos(BEND_MOST);
}

/**
 * Homes along the country roads, on purpose: 2 to 4 cottages where each road leaves an area and
 * where it meets another out in the country, then a farmstead (a farm lot: its fields, its barn
 * and yard) every 150 to 200 m of road, each clear of the areas, the water and steep land, and of
 * the area it is nearest. The farmsteads placed.
 */
function alongCountryRoads(content: Content, t: Templates, lots: LotPlacer, plans: readonly Plan[], roads: readonly RoadPlan[], land: Land, rand: Rand): PlotSlot[] {
  const country = roads.flatMap((r, i) => (r.kind === 'country' ? [i] : []));
  const byRoad = new Map<number, Frontage[]>();
  const lens = new Map(country.map((r) => [r, lengths(roads[r].points)]));
  for (const f of frontageOf(roads, country, plans[0].centre, rand)) {
    if (!straight(roads[f.road].points, f.s, lens.get(f.road)!)) continue;
    const list = byRoad.get(f.road);
    if (list) list.push(f);
    else byRoad.set(f.road, [f]);
  }
  /** Clear of every area's box by `by` m. */
  const outside = (p: Pt, by: number) => plans.every((a) => p[0] < a.rect.x - by || p[1] < a.rect.z - by || p[0] > a.rect.x + a.rect.w + by || p[1] > a.rect.z + a.rect.d + by);
  /** Well inside the world (a road out leaves it there). */
  const within = (p: Pt) => Math.min(p[0], p[1], land.W - p[0], land.D - p[1]) > 30;
  const homeArea = (p: Pt) => plans.reduce((k, a, i) => (dist(a.centre, p) < dist(plans[k].centre, p) ? i : k), 0);
  /** Dry and not steep within `r` m of `p`. */
  const buildable = (p: Pt, r: number) => {
    if (land.wet(p[0], p[1], r)) return false;
    for (let dz = -r; dz <= r; dz += CELL * 2) for (let dx = -r; dx <= r; dx += CELL * 2) if (land.slope(land.cell(p[0] + dx, p[1] + dz)) > STEEP) return false;
    return true;
  };
  const house = (lot: 'small' | 'farm', name: string, area: number) => ({ kind: 'house' as const, name, lot, area, ...houseFor(t, lot, lot === 'farm' ? 'roomy' : rand() < 0.5 ? 'snug' : 'roomy', rand) });

  // Cottages: where each country road leaves an area (a little way out along it), and where one meets another.
  const spots: { p: Pt; roads: number[] }[] = [];
  for (const r of country) {
    const pts = roads[r].points;
    const L = lengths(pts).at(-1)!;
    for (const e of [0, 1] as const) {
      if (roads[r].ends[e] === 'out') continue;
      const end = e === 0 ? pts[0] : pts[pts.length - 1];
      if (!outside(end, 20)) {
        const s = range(rand, 45, 80);
        if (L > s + 30) spots.push({ p: pointAt(pts, e === 0 ? s : L - s).p, roads: [r] });
      } else {
        const other = country.find((o) => o !== r && nearest(roads[o].points, end[0], end[1]).dist < roads[o].width / 2 + 2);
        spots.push({ p: end, roads: other === undefined ? [r] : [r, other] });
      }
    }
  }
  for (const spot of spots) {
    if (!buildable(spot.p, 24)) continue;
    const fronts = spot.roads
      .flatMap((r) => byRoad.get(r) ?? [])
      .filter((f) => dist(f.p, spot.p) > 10 && dist(f.p, spot.p) < COTTAGE_REACH && outside(f.p, 14) && within(f.p))
      .sort((a, b) => dist(a.p, spot.p) - dist(b.p, spot.p));
    if (!fronts.length) continue;
    const area = homeArea(spot.p);
    for (let k = between(rand, COTTAGES); k > 0; k--) if (lots.place([house('small', '', area)], fronts, [], spot.p, area, true).length) break;
  }

  // Farmsteads along the way, each named for a word of the farmland's (none twice).
  const words = content.names.worlds?.farm?.start ?? ['Meadow', 'Barley', 'Clover'];
  const named = new Set(lots.slots.filter((s) => s.lot === 'farm').map((s) => s.name));
  const farmName = () => {
    for (let k = 0; k < 24; k++) {
      const n = `${words[Math.floor(rand() * words.length)]} Farm`;
      if (!named.has(n)) return n;
    }
    return `${words[named.size % words.length]} Farm ${named.size + 1}`;
  };
  const farms: PlotSlot[] = [];
  for (const r of country) {
    const pts = roads[r].points;
    const L = lengths(pts).at(-1)!;
    const fronts = byRoad.get(r) ?? [];
    for (let s = range(rand, 50, 110); s < L - 30; ) {
      const p = pointAt(pts, s).p;
      if (!outside(p, FARM_OFF) || !within(p) || !buildable(p, 36) || farms.some((f) => dist([f.x, f.z], p) < 110)) {
        s += 16;
        continue;
      }
      // (Farms go on the frontage furthest down the list first: the nearest last here.)
      const near = fronts.filter((f) => Math.abs(f.s - s) < 24).sort((a, b) => Math.abs(b.s - s) - Math.abs(a.s - s));
      const area = homeArea(p);
      const name = farmName();
      if (lots.place([house('farm', name, area)], near, [], p, area, true).length) {
        s += 16;
        continue;
      }
      named.add(name);
      farms.push(lots.slots[lots.slots.length - 1]);
      s += range(rand, ...FARM_EVERY);
    }
  }
  return farms;
}

/**
 * Where an outdoor place of area `plan` goes: on its kind of land (by the water, its back to it;
 * up a hill; in the woods; in a meadow), 50 to 140 m from the area's middle, on land nothing else
 * takes. Its rectangle and which way it faces (towards the area: its trail comes from there).
 */
function siteFor(place: PublicTemplate, plan: Plan, land: Land, ground: Ground, water: Uint8Array, t: Templates, rand: Rand, W: number, D: number): { x: number; z: number; facing: Facing; w: number; d: number } | null {
  const frame = place.lot ?? t.plot;
  let best: { at: { x: number; z: number; facing: Facing; w: number; d: number }; score: number } | null = null;
  const wetAt = (x: number, z: number) => x >= 0 && z >= 0 && x < W && z < D && water[Math.floor(z) * W + Math.floor(x)] === 1;
  for (let k = 0; k < 260; k++) {
    const angle = rand() * Math.PI * 2;
    const r = range(rand, 50, 140) + Math.max(plan.rect.w, plan.rect.d) * 0.3;
    const c: Pt = [plan.centre[0] + Math.cos(angle) * r, plan.centre[1] + Math.sin(angle) * r];
    // From the area towards the place (its trail comes from there), or towards the water behind it.
    let n: Pt = [(c[0] - plan.centre[0]) / r, (c[1] - plan.centre[1]) / r];
    let score = 0;
    if (place.site === 'water') {
      let [sx, sz, hits] = [0, 0, 0];
      for (let a = 0; a < 16; a++) {
        const [dx, dz] = [Math.cos((a / 16) * Math.PI * 2), Math.sin((a / 16) * Math.PI * 2)];
        if (wetAt(c[0] + dx * 16, c[1] + dz * 16)) [sx, sz, hits] = [sx + dx, sz + dz, hits + 1];
      }
      if (!hits || hits > 9) continue;
      const l = Math.hypot(sx, sz) || 1;
      n = [sx / l, sz / l];
      score = -hits;
    } else if (place.site === 'hill') score = -land.at(c[0], c[1]) + land.slope(land.cell(c[0], c[1])) * 60;
    else if (place.site === 'woods') {
      let trees = 0;
      for (let dz = -1; dz <= 1; dz++) for (let dx = -1; dx <= 1; dx++) trees += land.woods[land.cell(c[0] + dx * 16, c[1] + dz * 16)];
      if (trees < 5) continue;
      score = -trees + land.slope(land.cell(c[0], c[1])) * 60;
    } else score = land.woods[land.cell(c[0], c[1])] * 4 + land.slope(land.cell(c[0], c[1])) * 60;
    const facing = (Math.abs(n[1]) >= Math.abs(n[0]) ? (n[1] > 0 ? 0 : 2) : n[0] > 0 ? 1 : 3) as Facing;
    const [w, d] = facing % 2 ? [frame.depth, frame.width] : [frame.width, frame.depth];
    const rect = { x: Math.round(c[0] - w / 2), z: Math.round(c[1] - d / 2), w, d };
    if (!ground.free({ x: rect.x - 3, z: rect.z - 3, w: rect.w + 6, d: rect.d + 6 }, 8)) continue;
    // Keeps a little way back from the water.
    if (land.wet(c[0], c[1], 0) || [0, 0.5, 1].some((u) => [0, 0.5, 1].some((v) => wetAt(rect.x - 2 + (rect.w + 4) * u, rect.z - 2 + (rect.d + 4) * v)))) continue;
    if (!best || score < best.score) best = { at: { x: rect.x, z: rect.z, facing, w: frame.width, d: frame.depth }, score };
  }
  return best?.at ?? null;
}

/** Trails' grid (m). */
const TRAIL_CELL = 2;

/**
 * A trail to outdoor place `slot` from the nearest of its area's roads, round the lots and the
 * water: A* on a 2 m grid from every road cell of the area to the place's front, smoothed where
 * that keeps it off the lots. Null if there's no way.
 */
function trailTo(slot: PlotSlot, plan: Plan, roads: readonly RoadPlan[], slots: readonly PlotSlot[], water: Uint8Array, land: Land, t: Templates, W: number, D: number): RoadPlan | null {
  const rect = plotRect(t, slot);
  const { width, depth } = slotDims(t, slot);
  const turn = turner(slot.x, slot.z, width, depth, slot.facing);
  const front = turn.point([width / 2, 0]);
  const before = turn.point([width / 2, -3]);
  // A window round the area and the place.
  const x0 = Math.max(0, Math.floor(Math.min(plan.rect.x, rect.x) - 40));
  const z0 = Math.max(0, Math.floor(Math.min(plan.rect.z, rect.z) - 40));
  const x1 = Math.min(W, Math.ceil(Math.max(plan.rect.x + plan.rect.w, rect.x + rect.w) + 40));
  const z1 = Math.min(D, Math.ceil(Math.max(plan.rect.z + plan.rect.d, rect.z + rect.d) + 40));
  const gw = Math.ceil((x1 - x0) / TRAIL_CELL);
  const gd = Math.ceil((z1 - z0) / TRAIL_CELL);
  const blocked = new Uint8Array(gw * gd);
  const mark = (r: Rect, grow: number) => {
    for (let z = Math.max(0, Math.floor((r.z - grow - z0) / TRAIL_CELL)); z <= Math.min(gd - 1, Math.floor((r.z + r.d + grow - z0) / TRAIL_CELL)); z++) {
      for (let x = Math.max(0, Math.floor((r.x - grow - x0) / TRAIL_CELL)); x <= Math.min(gw - 1, Math.floor((r.x + r.w + grow - x0) / TRAIL_CELL)); x++) blocked[z * gw + x] = 1;
    }
  };
  for (const s of slots) mark(plotRect(t, s), 0.4);
  for (let z = z0; z < z1; z++) for (let x = x0; x < x1; x++) if (water[z * W + x]) blocked[Math.floor((z - z0) / TRAIL_CELL) * gw + Math.floor((x - x0) / TRAIL_CELL)] = 1;
  const cellOf = ([x, z]: Pt) => Math.max(0, Math.min(gd - 1, Math.floor((z - z0) / TRAIL_CELL))) * gw + Math.max(0, Math.min(gw - 1, Math.floor((x - x0) / TRAIL_CELL)));
  const centre = (i: number): Pt => [x0 + ((i % gw) + 0.5) * TRAIL_CELL, z0 + (Math.floor(i / gw) + 0.5) * TRAIL_CELL];
  const onRoad = new Uint8Array(gw * gd);
  const starts: number[] = [];
  const startPoint = new Map<number, Pt>();
  for (const r of plan.roads) {
    for (const p of roads[r].points) {
      const i = cellOf(p);
      onRoad[i] = 1;
      if (!blocked[i] && !startPoint.has(i)) {
        starts.push(i);
        startPoint.set(i, p);
      }
    }
  }
  const goal = cellOf(before);
  blocked[goal] = 0;
  const cells = astar(gw, gd, (i) => (blocked[i] ? Infinity : onRoad[i] ? 4 : 1 + land.woods[land.cell(...centre(i))] * 0.4), starts, (i) => i === goal, (i) => dist(centre(i), before) / TRAIL_CELL);
  if (!cells) return null;
  const raw: Pt[] = [startPoint.get(cells[0])!, ...cells.slice(1).map(centre), before, front];
  const clear = (pts: readonly Pt[]) =>
    pts.every(([x, z]) => !slots.some((s) => {
      const r = plotRect(t, s);
      return s !== slot && x > r.x + 0.05 && x < r.x + r.w - 0.05 && z > r.z + 0.05 && z < r.z + r.d - 0.05;
    }) && !(x >= 0 && z >= 0 && x < W && z < D && water[Math.floor(z) * W + Math.floor(x)]));
  const smooth = curve(simplify(raw, 0.9));
  const points = clear(smooth) ? smooth : densify(raw);
  return { points: points.map(([x, z]): Pt => [round2(x), round2(z)]), width: TRAIL_WIDTH, ends: ['join', 'dead'], kind: 'trail' };
}

/** A polyline with a point at least every metre. */
function densify(pts: readonly Pt[]): Pt[] {
  const out: Pt[] = [];
  for (let i = 0; i + 1 < pts.length; i++) {
    const n = Math.max(1, Math.ceil(dist(pts[i], pts[i + 1])));
    for (let k = 0; k < n; k++) out.push([pts[i][0] + ((pts[i + 1][0] - pts[i][0]) * k) / n, pts[i][1] + ((pts[i + 1][1] - pts[i][1]) * k) / n]);
  }
  out.push(pts[pts.length - 1]);
  return out;
}

/** The middles of a big world's areas, and a centroid helper for the menus. */
export function areaCentre(a: TownArea): Pt {
  return centroid([[a.x, a.z], [a.x + a.w, a.z + a.d]]);
}

/** Where the roads lead out of a big world: their `out` ends (on its edge). */
export function worldExits(roads: readonly RoadPlan[]): Pt[] {
  return roadEnds(roads.filter((r) => r.kind !== 'trail')).map(({ x, z }): Pt => [x, z]);
}
