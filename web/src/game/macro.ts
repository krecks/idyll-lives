/**
 * A big world, laid out from its `WorldSpec` (`world.ts`): its land (896 to 1024 m of it in the
 * classic worlds) with a main town (the spec's shape, its streets reaching all of its box), its
 * hamlets (five to seven in the classic worlds) 170 m or more from each other and spread over the
 * whole world, country roads between
 * them and out of the world with farmsteads and cottages along them, and countryside: woods in
 * copses, meadows, hills, water, farmland and orchards round the hamlets and farmsteads, and
 * outdoor places (a picnic meadow, a fishing jetty, a lookout, a campfire, an orchard, a hammock
 * grove) reached by trails from the nearest hamlet.
 *
 * A place's water may be a river across the world (its town on a bank, a bridge from the town's
 * waterfront street, one or two more where it runs straight; country roads keep to one bank but
 * over a bridge) or the sea round an island (causeways out over it are the world's ways out). A
 * bridge or a causeway is a straight country road whose deck over the water is `RoadPlan.bridge`;
 * the sim's walkers walk its deck dry (`deckOf`). A place's world carries its relief (`reliefOf`).
 *
 * The land is a cost grid of 8 m cells rolled from noise (how steep, how wooded, where the water
 * is); the areas go where it is flat and dry, far enough apart (a Poisson disc); country roads
 * leave an area by a street's end straight on or by a T square off a street, and are paths over
 * the grid that turn as a road does (`steer`: wide bends, never back on themselves), smoothed
 * into curves with no hairpins (`COUNTRY_TURN`); one whose path follows a road laid already shares
 * it, joining it and leaving it in a T, rather than running beside it (`alongExisting`); trails
 * are A* paths on a finer grid round the lots. One `Ground` covers the whole world, and each
 * area's lots go along its own roads, nearest its middle first (`town.ts` `LotPlacer`).
 */

import type { Content } from '../content/content';
import type { BackdropBuilding, Rle, TownArea, TownPlace, WorldMeta, WorldZone } from '../core/protocol';
import { distanceToRect } from '../render/shared/chunks';
import { chooseCentre, layBackdrop, reserveCentre } from './backdrop';
import { random } from './housegen';
import { COVER, COVER_CELL, coverSpan, LAND_CELL, NONE, type Land as LandField } from './land';
import { APRON, landZones, Protection, wetTiles, type BuiltLand } from './landEdit';
import { chaikin, outlines, rasterize, rle, simplify, simplifyRing, unrle } from './landMasks';
import { fbm } from './noise';
import { centroid, cropRoads, curve, inside, leftOf, lengths, nearest, pointAt, RoadIndex, roadEnds, sharpestTurn, straightAround, tangent, type Box, type Pt, type RoadPlan } from './roads';
import { BIG_COURTS, boundsOf, COURTS, frontageOf, Ground, houseFor, joinUp, LotPlacer, plotRect, settleRoads, slotDims, turner, wantedLots, type Facing, type Frontage, type NeighbourhoodDraft, type PlotSlot, type PublicTemplate, type Rect, type Templates } from './town';
import { streetNames, worldName, worldTagline, type SizeSpec, type WorldSpec, type WorldStep } from './world';

/** The mask helpers moved to `landMasks.ts`; still here for those that import them from the generator. */
export { chaikin, outlines, rasterize, rle, simplifyRing, unrle };

type Rand = () => number;

/** A big world's areas, land and parking, as the town file carries them (`assembleTown`). */
export interface OpenWorldDraft {
  /** Area 0 is the main town. */
  areas: TownArea[];
  zones: WorldZone[];
  /** The water's tiles (1 m, row-major over the world), run-length (`Rle`). */
  water: Rle;
  /** The height (m) each outdoor plot is levelled to, by plot index. */
  pads: Record<number, number>;
  parking: { x: number; z: number; rot: number; area: number }[];
  /** The living town (`backdrop.ts`): buildings nobody lives in, the places the tags name, the main town's centre. */
  backdrop?: BackdropBuilding[];
  places?: TownPlace[];
  centre?: { area: number; poly: Pt[] }[];
  /** The centres' squares (kept clear of lots: `withGarden`); not in the town file. */
  squares?: Pt[][];
  /** How high the land stands over the level the towns are built at (`reliefOf`); none in a classic world. */
  relief?: WorldMeta['relief'];
  /**
   * The land's height (m) per 8 m cell, read at the cells' middles, over the level the towns are
   * built at (`heightsOf`: below it too, unlike `relief`), row-major from the world's corner. What
   * a place's land as a height field is seeded from (`landFollow.ts` `withLand`); none in a classic
   * world or one laid again on a shaped land, and not in the town file.
   */
  heights?: { cell: number; cols: number; rows: number; h: Float32Array };
  /**
   * The land as a height field (`land.ts`), once the player has shaped it (step 1's brushes,
   * `landEdit.ts` `applyLand`) or the town was laid again on it (`generateFromSpec`'s `shaped`):
   * the town file carries it as `meta.land` once it is `edited`.
   */
  land?: LandField;
}

/** The land's cells (m). */
const CELL = 8;
/** Areas this far apart at least (their middles, m), and as far as this from the one they join; best this far from the nearest. */
export const AREA_APART = 170;
const AREA_REACH = 440;
const AREA_AIM = 230;
/** No area nearer the world's edge than this (m), nor nearer another's box. */
const EDGE = 50;
const BOX_GAP = 24;
/**
 * An area's road out of it ends this far inside the world's edge at least (m), where the country
 * road on from it starts: an area's box keeps `EDGE` off it, but a lane out of it can reach further
 * than that (a farm hamlet's fork, its box's depth long), past the edge, and a country road on
 * from beyond the edge is cut off from it when the roads are cut to the world.
 */
const PORT_IN = EDGE / 2;
/**
 * Every road in a big world, the country roads and every area's streets and lanes, is this wide at
 * least (m): two cars pass on it in their lanes (the sim's `cars::lane_on`), so all of them are laid
 * this wide before any lot is placed.
 */
const ROAD_WIDTH = 7;
/** Country roads' width (m), trails'. */
const COUNTRY_WIDTH = ROAD_WIDTH;
const TRAIL_WIDTH = 2;
/** Houses with no room in any area go along a country road, as far as this from their own area's box (m). */
const SPILL = 140;
/** The main town's terrace rows with no room in it go along the country roads leaving it, as far as this from its box (m). */
const TERRACE_SPILL = 110;
/** Along a country road: no farmstead nearer an area's box than this (m; how often one comes: `WorldSpec.farmEvery`). */
const FARM_OFF = 25;
/** Cottages by the country roads: so many where each leaves an area or meets another, within this of the spot (m). */
const COTTAGES: [number, number] = [2, 4];
const COTTAGE_REACH = 70;
/** Land steeper than this (rise per m) is no place for a farmstead or a cottage; in a place's world (its towns follow its slopes) nor for any lot out along a country road, within `SPILL_LOOK` m of where it goes. */
const STEEP = 0.25;
const PLACE_STEEP = 0.12;
const SPILL_LOOK = 12;
/** An orchard beside a farmstead: this deep from the lot's side (m), a gap of this from it. */
const ORCHARD = 26;
const ORCHARD_GAP = 3;
/**
 * A country road leaves an area by a port within `SWING` (radians) of the way it is going, else
 * swinging round costs it `SWING_COST` m a radian more; a T off a street facing within
 * `SIDE_FACING` of the way costs `SIDE_COST` m more than a port, and keeps `SIDE_APART` m from the
 * area's other roads where it leaves its street and `SIDE_DEPTH` m (a lot's depth) as it crosses.
 */
const SWING = Math.PI / 6;
const SWING_COST = 120;
const SIDE_FACING = Math.PI / 4;
const SIDE_COST = 30;
const SIDE_APART = 24;
const SIDE_DEPTH = 16;
/** Leaving off a country road that leaves a port already (`via`) costs this much more (m): a port nobody has taken yet goes first. */
const VIA_COST = 40;
/** Leaving from a port another country road leaves already, both from it together, costs this much more (m). */
const RETAKE_COST = 240;
/** A road on from a port nobody took goes to what lies within this (radians) of straight on from it, rather than what lies nearer off to the side. */
const AHEAD = Math.PI / 3;
/** What going on to something off to the side of such a port or behind it costs, more than to what lies ahead (cells of flat open land). */
const ASIDE = 30;
/** How many of the likeliest ways out of each area (`waysOut`) a country road between two looks at. */
const WAYS = 5;
/** A country road's course, before it is smoothed into a curve, has a point at least this often (m): see `evenly`. */
const CTRL_APART = 16;
/**
 * A country road's course that comes within `SHARED_REACH` cells of a road laid already (not a
 * trail) for `SHARED_RUN` cells or more (in a row, give or take `SHARED_GAP`), heading its way give
 * or take `SHARED_ANGLE`, follows it there: it is laid only up to it and on from it, joining it in
 * a T (`alongExisting`), and that road carries it over the stretch, rather than the two running
 * side by side a few metres apart (cars on them meet nose to nose, each pulling over to its right).
 */
const SHARED_REACH = 2;
const SHARED_RUN = 3;
const SHARED_GAP = 2;
const SHARED_ANGLE = Math.PI / 4;
/** A country road between two areas goes straight on from a way out of one to a street of the other this near (m), where that street bends less than `STRAIGHT_BEND` (radians) over 28 m. */
const STRAIGHT_ON = 160;
const STRAIGHT_BEND = 0.6;
/** A country road between two areas tries this many pairs of ways for one that doesn't wind round further than `WINDING` (radians) within `WINDING_OVER` m (a loop). */
const TRIES = 4;
const WINDING = Math.PI;
const WINDING_OVER = 150;
/** A country road turns no sharper than this (radians) over any 40 m of it: as a steady bend, one of 16 m radius or more; no hairpins. */
export const COUNTRY_TURN = (70 * Math.PI) / 180;
/**
 * What a country road is laid to keep short of `COUNTRY_TURN` (radians): its points are rounded to
 * the centimetre in the file, and a bend near its end is measured over a span that ends where the
 * road does, so a hair either way moves it.
 */
const TURN_SPARE = 0.05;
/** The longer straight leads (m) a country road tries out of a way or into one it meets at a slant (`countryRoad`). */
const LONG_LEADS = [24, 34, 44];
/** An area's lots keep their middles this far along their road from where it meets another (m), and off where it turns more than `TIGHT_BEND` (radians) over as far (a bend of 20 m radius or less). */
const CORNER = 18;
const TIGHT_BEND = 0.9;
/** A home by a country road stands where it turns less than `BEND_MOST` (radians) over `BEND` m either side. */
const BEND = 16;
const BEND_MOST = 0.3;
/** A T leaves a street where it turns less than this (radians) over 14 m either side. */
const T_BEND = 0.3;

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
 * blocked, and `rise` more a step from cell to cell where given) from any of `starts` to a cell
 * `goal` accepts, `h` guessing the rest: the cells, start first.
 */
function astar(gw: number, gd: number, cost: (i: number) => number, starts: readonly number[], goal: (i: number) => boolean, h: (i: number) => number, limit = 400000, rise?: Rise): number[] | null {
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
        const ng = g[i] + ((here + c) / 2) * (dx && dz ? Math.SQRT2 : 1) + (rise ? rise(i, j) : 0);
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

/** What a step from one cell to a neighbour costs more than its cells do (a climb steeper than a road may make). */
type Rise = (from: number, to: number) => number;

/** The eight ways a cell steps to its neighbours, in turn round the compass, an eighth of a turn apart. */
const STEPS: readonly Pt[] = [[1, 0], [1, 1], [0, 1], [-1, 1], [-1, 0], [-1, -1], [0, -1], [1, -1]];
/** The step nearest heading `d`. */
const octant = ([x, z]: Pt) => (((Math.round(Math.atan2(z, x) / (Math.PI / 4)) % 8) + 8) % 8);
/** How nearly step `h` heads along `d` (a unit vector): the cosine between them. */
const along = (h: number, d: Pt) => (STEPS[h][0] * d[0] + STEPS[h][1] * d[1]) / Math.hypot(...STEPS[h]);

/**
 * The cheapest way over a `gw` × `gd` grid as a road runs, not as a walker does: heading one of
 * eight ways, turning an eighth of a turn at a time, two cells or more straight on between turns
 * the same way (a curve of 25 m radius or more, never back on itself) and one between turns back
 * the other way (a bend round something). From whichever of `starts` (a cell, the way it heads,
 * and what starting there costs already) to a cell and heading `goal` accepts (what it costs to
 * end there; -1 if it is no end). Cost per cell `cost` (Infinity where blocked), plus `TURN_COST`
 * a turn and `TIGHT_COST` more for one only a cell after the last, and `rise` a step where given,
 * `h` guessing the rest. Given `toward`, it never heads back the other way (so never round in a
 * loop). The cells, start first; which start; the heading it ends on.
 */
function steer(gw: number, gd: number, cost: Float32Array, starts: readonly { cell: number; h: number; g: number }[], goal: (i: number, h: number) => number, h: (i: number) => number, toward?: Pt, limit = 400000, rise?: Rise): { cells: number[]; start: number; h: number } | null {
  // A state: a cell, a heading and how it came there: (0, 1) by a turn one way or the other, (2, 3)
  // a cell straight on after one, (4) two or more (or from the start).
  const K = 5;
  const n = gw * gd * 8 * K;
  const g = new Float32Array(n).fill(Infinity);
  const from = new Int32Array(n).fill(-1);
  const closed = new Uint8Array(n);
  const open = new Heap();
  for (const s of starts) {
    const k = (s.cell * 8 + s.h) * K + 4;
    if (cost[s.cell] === Infinity || g[k] <= s.g) continue;
    g[k] = s.g;
    open.push(k, s.g + h(s.cell));
  }
  // (The best end yet, with what ending there costs: none better once the open states all cost more.)
  let done = { k: -1, g: Infinity };
  for (let steps = 0; open.size && steps < limit; steps++) {
    const k = open.pop();
    if (closed[k]) continue;
    closed[k] = 1;
    if (g[k] + h(Math.floor(k / K / 8)) >= done.g) break;
    const c = k % K;
    const hd = Math.floor(k / K) % 8;
    const i = Math.floor(k / K / 8);
    const end = goal(i, hd);
    if (end >= 0 && g[k] + end < done.g) done = { k, g: g[k] + end };
    const [x, z] = [i % gw, Math.floor(i / gw)];
    for (const turn of c === 4 ? [0, 1, -1] : c === 2 ? [0, -1] : c === 3 ? [0, 1] : [0]) {
      const nh = (hd + turn + 8) % 8;
      if (toward && along(nh, toward) < -0.8) continue;
      const [dx, dz] = STEPS[nh];
      const [nx, nz] = [x + dx, z + dz];
      if (nx < 0 || nz < 0 || nx >= gw || nz >= gd) continue;
      const j = nz * gw + nx;
      if (cost[j] === Infinity) continue;
      // (No cutting a blocked corner.)
      if (dx && dz && (cost[z * gw + nx] === Infinity || cost[nz * gw + x] === Infinity)) continue;
      const nk = (j * 8 + nh) * K + (turn ? (turn > 0 ? 0 : 1) : c < 2 ? c + 2 : 4);
      if (closed[nk]) continue;
      const ng = g[k] + ((cost[i] + cost[j]) / 2) * (dx && dz ? Math.SQRT2 : 1) + (turn ? TURN_COST + (c < 4 ? TIGHT_COST : 0) : 0) + (rise ? rise(i, j) : 0);
      if (ng < g[nk]) {
        g[nk] = ng;
        from[nk] = k;
        open.push(nk, ng + h(j));
      }
    }
  }
  if (done.k < 0) return null;
  const path = [done.k];
  while (from[path[path.length - 1]] >= 0) path.push(from[path[path.length - 1]]);
  path.reverse();
  const first = path[0];
  const start = starts.findIndex((s) => (s.cell * 8 + s.h) * K + 4 === first);
  return { cells: path.map((k) => Math.floor(k / K / 8)), start, h: Math.floor(done.k / K) % 8 };
}
/** The steepest a country road climbs (`builtSurface.ts` `GRADE.country`); among a place's mountains, a hamlet's box no higher or lower than the area it joins by more than this share of that over the gap between them; and how much a metre of height counts against a site there (the valley floor first). */
const COUNTRY_GRADE = 0.12;
const REACHABLE = 0.7;
const MOUNTAIN_LOW = 0.3;
/** A country road keeps under this grade where it can in a place's world (a little under its limit, `builtSurface.ts` `GRADE`), a step steeper costing it `ROUTE_STEEP` cells of flat open land for each 1 of grade over. */
const ROUTE_GRADE = 0.1;
const ROUTE_STEEP = 300;
/** A road out of a place's world looks for the lowest of an edge this far (m) either way of straight out; a metre more than it may climb counts this much against that edge (m). */
const EDGE_WINDOW = 200;
const OVER_CLIMB = 40;
/** A place's country road that would stand further off its land than this (m) to keep within its grade is too steep to lay (`tooSteep`); what that counts against a way on. */
const STEEP_OFF = 3;
const STEEP_FAULT = 1000;
/** What `steer` adds for a turn, and for one soon after the last (a cell's worth of flat open land is 1): a road keeps straight where it may, and to wide curves. */
const TURN_COST = 1;
const TIGHT_COST = 2;
/** What a cell a country road may only go through when there is no other way costs it more (cells of flat open land). */
const DETOUR = 20;
/** How far `route` looks (states) for a way that may head any way, once one that doesn't head back has found none. */
const LOOSE_LIMIT = 200000;
/** What coming into a way more loosely (`route`) costs more (cells of flat open land). */
const LOOSE_COST = 12;

/**
 * A polyline with points put in along its longer stretches, none longer than `most` m: what
 * `curve` makes of it keeps to it (a long stretch beside a short one swings the curve out past
 * the short one's end and back, a kink).
 */
function evenly(pts: readonly Pt[], most: number): Pt[] {
  const out: Pt[] = pts.length ? [pts[0]] : [];
  for (let i = 1; i < pts.length; i++) {
    const n = Math.ceil(dist(pts[i - 1], pts[i]) / most);
    for (let k = 1; k < n; k++) out.push([pts[i - 1][0] + ((pts[i][0] - pts[i - 1][0]) * k) / n, pts[i - 1][1] + ((pts[i][1] - pts[i - 1][1]) * k) / n]);
    out.push(pts[i]);
  }
  return out;
}

/** How far `p` is from rectangle `r` (0 inside it). */
const offRect = (r: Rect, [x, z]: Pt) => distanceToRect(x, z, r.x, r.z, r.w, r.d);

/** Whether `p` is clear of every area's box by more than `by` m. */
const clearOf = (plans: readonly { rect: Rect }[], p: Pt, by: number) => plans.every((a) => p[0] < a.rect.x - by || p[1] < a.rect.z - by || p[0] > a.rect.x + a.rect.w + by || p[1] > a.rect.z + a.rect.d + by);

/**
 * A polyline relaxed until it turns no sharper than `most` (radians) over any two spans of 20 m
 * (`sharpestTurn`), its ends where they were: points every 2 m, each pulled towards the middle of
 * its neighbours, again and again (as far as `tries` times). Null if it still turns too sharply.
 */
function unkink(pts: readonly Pt[], most: number, tries = 600): Pt[] | null {
  const ls = lengths(pts);
  const L = ls[ls.length - 1];
  const n = Math.max(2, Math.ceil(L / 2));
  let line = Array.from({ length: n + 1 }, (_, k): Pt => pointAt(pts, (L * k) / n, ls).p);
  // (Measured as it will be filed: to the centimetre.)
  const filed = (pts: readonly Pt[]) => pts.map(([x, z]): Pt => [round2(x), round2(z)]);
  for (let k = 0; k < tries; k++) {
    if (k % 20 === 0 && sharpestTurn(filed(line), 20).angle <= most) return filed(line);
    line = line.map((p, i): Pt => (i === 0 || i === n ? p : [(p[0] * 2 + line[i - 1][0] + line[i + 1][0]) / 4, (p[1] * 2 + line[i - 1][1] + line[i + 1][1]) / 4]));
  }
  return sharpestTurn(filed(line), 20).angle <= most ? filed(line) : null;
}

/** The most a polyline's heading (along it 10 m at a time) changes, one way, within `over` m of it (radians). */
function winding(pts: readonly Pt[], over: number): number {
  const ls = lengths(pts);
  const L = ls[ls.length - 1];
  const heads: number[] = [];
  for (let s = 0; s + 10 <= L; s += 10) {
    const [a, b] = [pointAt(pts, s, ls).p, pointAt(pts, s + 10, ls).p];
    let h = Math.atan2(b[1] - a[1], b[0] - a[0]);
    // (Unwrapped: each within half a turn of the last.)
    if (heads.length) h += Math.round((heads[heads.length - 1] - h) / (2 * Math.PI)) * 2 * Math.PI;
    heads.push(h);
  }
  let most = 0;
  for (let i = 0; i < heads.length; i++) for (let j = i + 1; j < heads.length && j - i <= over / 10; j++) most = Math.max(most, Math.abs(heads[j] - heads[i]));
  return most;
}

/** A `W` × `D` mask of tiles grown by `by` tiles every way (a square round each). */
function grown(mask: Uint8Array, W: number, D: number, by: number): Uint8Array {
  const across = new Uint8Array(mask.length);
  for (let z = 0; z < D; z++) {
    for (let x = 0, last = -Infinity; x < W; x++) {
      if (mask[z * W + x]) last = x;
      // (The nearest set tile at or before x, within `by`; then the one after.)
      if (x - last <= by) across[z * W + x] = 1;
    }
    for (let x = W - 1, next = Infinity; x >= 0; x--) {
      if (mask[z * W + x]) next = x;
      if (next - x <= by) across[z * W + x] = 1;
    }
  }
  const out = new Uint8Array(mask.length);
  for (let x = 0; x < W; x++) {
    for (let z = 0, last = -Infinity; z < D; z++) {
      if (across[z * W + x]) last = z;
      if (z - last <= by) out[z * W + x] = 1;
    }
    for (let z = D - 1, next = Infinity; z >= 0; z--) {
      if (across[z * W + x]) next = z;
      if (next - z <= by) out[z * W + x] = 1;
    }
  }
  return out;
}

/** A road end that led out of an area's box (`Plan.ports`): where, which way out, and whether a country road goes on from it yet. */
interface Port {
  road: number;
  end: 0 | 1;
  area: number;
  p: Pt;
  dir: Pt;
  used: boolean;
}

/**
 * Where a country road leaves an area or comes into it: from `p` heading `dir`, straight on for
 * `reach` m (across the area's own land, from a T off one of its streets) before it may turn; the
 * port it goes on from (or off the country road from), if any.
 */
interface Way {
  p: Pt;
  dir: Pt;
  reach?: number;
  port?: Port;
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
  ports: Port[];
}

/** A stream of draws of its own for the massif's height (so a spec without one draws as it did). */
const MASSIF = 0x6d617373;
/** A stream of its own for how a place's land looks (`WorldSpec.landscape`): its palette and trees stay as the options change. */
const LOOK = 0x6c6f6f6b;
/** The mountains rise this far from the valley, the shore, the lake (m), past this much of it flat (m); the low-z side (the menus' camera's) to this share of their height. */
const MASSIF_RISE = 260;
const MASSIF_FLAT = 70;
const MASSIF_NEAR = 0.45;
/** The size of world (m) a lake is drawn for: in a smaller one (a place's), smaller in proportion. */
const LAKE_WORLD = 992;
/** A place's outdoor places no further than this from their area's box (m: the sample's check is 150). */
const SPOT_REACH = 130;
/** A place's lots keep this far from the water (m). */
const SHORE_GAP = 3;
/** Streams of their own for a river's course and its bridges (a place's). */
const RIVER = 0x72697672;
/** A river's width (m), how far its meanders swing (m), how far its mountains stand back from it (m). */
const RIVER_WIDTH: [number, number] = [14, 22];
const MEANDER: [number, number] = [70, 110];
const RIVER_FLAT = 90;
/** An island's coast: a rounded square (`ISLAND` of the world's size from its middle to the coast along each axis), its points round it. */
const ISLAND = 0.44;
const COAST_POINTS = 180;
/** A bridge's ends stand this far inland of the banks (m); a causeway's inland end this far from the coast. Its deck: the water, and this much either side (m). */
const BRIDGE_IN = 12;
const CAUSEWAY_IN = 30;
const DECK_PAST = 2;
/** The longest a bridge's deck may be (m); more bridges over a river (few villages: one, many: two), this far from each other at least and from the areas' boxes (m). */
const BRIDGE_SPAN = 45;
const BRIDGE_APART = 220;
const BRIDGE_CLEAR = 30;
/** The world's edges (north, east, south, west): which way is out over each, and their names. */
const EDGE_OUT: readonly Pt[] = [[0, -1], [1, 0], [0, 1], [-1, 0]];
const EDGE_NAMES = ['North', 'East', 'South', 'West'];

/** How many, in words (the ways out the world wants at least). */
const WORDS = ['no', 'one', 'two', 'three', 'four'];

/**
 * Lays out a big world from `spec` and `seed` (the same spec and seed always give the same world).
 * See the top of this file. `step`: told as each part of it is done (it changes nothing).
 *
 * `shaped`: the town laid again on a land the player shaped (`land.ts`; step 1's "Lay the town
 * again"). The world is the land's size, and its 8 m cells are filled from it (`fillFrom`: the
 * mean height, wet if any sample is, wooded if most of it is); the place's own water and
 * mountains are not laid, nor any bridge over a river; the areas go only on the largest piece of
 * dry land (`dryRegion`). The world keeps its name, its towns' shapes and their lot mix. The draft
 * carries a copy of the land (`open.land`), graded flat under what is built and its cover cleared
 * there, and its water and zones come from it.
 */
export function generateFromSpec(content: Content, t: Templates, spec: WorldSpec, seed: number, name?: string, step?: (done: WorldStep) => void, shaped?: LandField): NeighbourhoodDraft {
  const rand = random(seed);
  const rolledName = worldName(content, spec.names, rand);
  const tagline = worldTagline(content, spec, rand);
  const rolledW = Math.round(range(rand, ...spec.world) / 64) * 64;
  const W = shaped ? shaped.cols * LAND_CELL : rolledW;
  const D = shaped ? shaped.rows * LAND_CELL : W;
  const land = new Land(W, D);
  const { gw, gd } = land;
  const nseed = (seed % 100000) + 17;
  /** Country roads the layout found no way for (and bridges and causeways). */
  const failed: string[] = [];

  // ---- the land: hills (a valley between the massif's mountains), woods -----------------------
  const valley = !!spec.massif;
  const massif = spec.massif ? range(random((seed ^ MASSIF) >>> 0), ...spec.massif) : 0;
  const valleyZ = (x: number) => D / 2 + (fbm(x / 420, 3.7, nseed + 3, 3) - 0.47) * 260;
  const clamp01 = (v: number) => Math.min(1, Math.max(0, v));
  for (let i = 0; i < gw * gd; i++) {
    const [x, z] = land.centre(i);
    let h = fbm(x / 230, z / 230, nseed, 4) * spec.hills * 2;
    if (valley && spec.classic) h += massif * clamp01((Math.abs(z - valleyZ(x)) - MASSIF_FLAT) / MASSIF_RISE) ** 1.4;
    land.height[i] = h;
  }
  if (shaped) fillFrom(land, shaped);
  /** On a shaped land, its largest piece of dry land (the areas go only there), with a test that a box stands on it. */
  const region = shaped ? dryRegion(land) : null;
  /**
   * A place's mountains (not a classic world's): the massif's height times `m(x, z)`^1.4 (0 where
   * the land stays low, 1 where the mountains stand full height), lower on the low-z side, once
   * the water they rise from is laid.
   */
  const raise = (m: (x: number, z: number) => number) => {
    if (!valley || spec.classic || shaped) return;
    for (let i = 0; i < gw * gd; i++) {
      const [x, z] = land.centre(i);
      land.height[i] += massif * clamp01(m(x, z)) ** 1.4 * (MASSIF_NEAR + (1 - MASSIF_NEAR) * clamp01(z / (0.6 * D)));
    }
  };

  // ---- the main town: its roads first (which side its water is on), then where it goes -------
  const mainSpec = spec.town;
  const layout = spec.shape(rand, mainSpec.box, mainSpec.scale);
  const mainRoads = settleRoads(joinUp(rand, layout, mainSpec, BIG_COURTS), mainSpec.box);
  const box = mainSpec.box;
  const waterSide = layout.water ?? (rand() < 0.5 ? 'zmin' : 'zmax');

  // The sea along one edge, a lake in the middle; the mountains (a place's) inland of them, else either side of the valley.
  if (spec.water === 'sea') {
    const deep = range(rand, 120, 160);
    const shoreOf = (x: number) => deep + (fbm(x / 160, 9.1, nseed + 5, 3) - 0.47) * 90;
    for (let i = 0; i < gw * gd && !shaped; i++) {
      const [x, z] = land.centre(i);
      const shore = shoreOf(x);
      if (waterSide === 'zmax' ? z > D - shore : z < shore) land.water[i] = 1;
    }
    raise((x, z) => ((waterSide === 'zmax' ? D - shoreOf(x) - z : z - shoreOf(x)) - 160) / MASSIF_RISE);
  }
  const lake: Pt = [W / 2 + range(rand, -50, 50), D / 2 + range(rand, -40, 40)];
  if (spec.water === 'lake') {
    let [rx, rz] = [range(rand, 150, 185), range(rand, 95, 120)];
    // (A place's: a lake in proportion to its world, and room for the town between it and the world's edge, on its side of it.)
    if (!spec.classic) {
      [rx, rz] = [rx * Math.min(1, W / LAKE_WORLD), rz * Math.min(1, W / LAKE_WORLD)];
      const room = EDGE / 2 + box.d + 4 + rz * 1.4;
      lake[1] = waterSide === 'zmin' ? Math.min(lake[1], D - room) : Math.max(lake[1], room);
    }
    for (let i = 0; i < gw * gd && !shaped; i++) {
      const [x, z] = land.centre(i);
      const r = ((x - lake[0]) / rx) ** 2 + ((z - lake[1]) / rz) ** 2;
      if (r < 1 + (fbm(x / 90, z / 90, nseed + 6, 3) - 0.47) * 0.7) land.water[i] = 1;
    }
    raise((x, z) => ((Math.hypot((x - lake[0]) / rx, (z - lake[1]) / rz) - 1.9) * Math.min(rx, rz)) / 200);
  }
  // An island (a place's): a rounded square with a ragged coast, the sea all round it to the world's edges; its mountains in its middle.
  const coast: Pt[] = [];
  if (spec.water === 'island') {
    const [ax, az] = [ISLAND * W, ISLAND * D];
    for (let k = 0; k < COAST_POINTS; k++) {
      const a = (k / COAST_POINTS) * Math.PI * 2;
      const [c, s] = [Math.cos(a), Math.sin(a)];
      const r0 = 1 / (Math.abs(c / ax) ** 3 + Math.abs(s / az) ** 3) ** (1 / 3);
      const r = r0 * (1 + (fbm((W / 2 + c * r0) / 140, (D / 2 + s * r0) / 140, nseed + 23, 3) - 0.47) * 0.14);
      coast.push([round2(W / 2 + c * r), round2(D / 2 + s * r)]);
    }
    for (let i = 0; i < gw * gd && !shaped; i++) if (!inside(coast, ...land.centre(i))) land.water[i] = 1;
    raise((x, z) => (0.75 - (Math.abs((x - W / 2) / ax) ** 3 + Math.abs((z - D / 2) / az) ** 3) ** (1 / 3)) / 0.45);
  }
  if (spec.water === 'ponds') raise((x, z) => (Math.abs(z - valleyZ(x)) - MASSIF_FLAT) / MASSIF_RISE);
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
  // (Room for the town between the river and the world's edge, on its side of it.)
  const room = EDGE / 2 + box.d + 40;
  const river = spec.water === 'river' && !shaped ? riverAcross(random((seed ^ RIVER) >>> 0), W, D, [ox, ox + box.w], nseed, waterSide === 'zmax' ? [room, D] : [0, D - room]) : null;
  if (river) {
    for (let i = 0; i < gw * gd; i++) {
      const [x, z] = land.centre(i);
      // (Every cell the river touches.)
      if (Math.abs(z - river.z(x)) < river.half(x) + CELL / 2) land.water[i] = 1;
    }
    raise((x, z) => (Math.abs(z - river.z(x)) - RIVER_FLAT) / MASSIF_RISE);
  }
  /** Which side of the river a point is on (-1 north, 1 south; 0 with no river). */
  const bank = (p: Pt) => (river ? (p[1] < river.z(p[0]) ? -1 : 1) : 0);
  if (region) {
    // On a shaped land: as near where it would go as it fits on the largest piece of dry land.
    const at = region.nearest({ x: ox, z: oz, w: box.w, d: box.d }, [EDGE, EDGE / 2, W - EDGE - box.w, D - EDGE / 2 - box.d]);
    if (!at) throw new Error('There is no dry land big enough for the town.');
    [ox, oz] = at;
  } else if (spec.water === 'sea' || spec.water === 'lake' || spec.water === 'river' || spec.water === 'island') {
    // Its water side on the shore (its first road runs along it): looking for the water from the land beyond the town.
    const dir = waterSide === 'zmax' ? 1 : -1;
    const from = Math.max(0, Math.min(D - 1, spec.water === 'island' ? D / 2 : (spec.water === 'sea' ? D / 2 : spec.water === 'lake' ? lake[1] : river!.z(ox + box.w / 2)) - dir * 260));
    const shore = shoreAt(ox, ox + box.w, dir, from);
    if (Number.isFinite(shore)) oz = Math.round(dir > 0 ? shore - 4 - box.d : shore + 4);
  } else if (valley) oz = Math.round(valleyZ(ox + box.w / 2) - box.d / 2);
  if (!region) oz = Math.max(EDGE / 2, Math.min(D - EDGE / 2 - box.d, oz));

  const roads: RoadPlan[] = [];
  const plans: Plan[] = [];
  const takenNames: string[] = [];
  /** An area's roads onto the world: moved to (`x`, `z`), turned across (`swap`), kept as the area's. */
  const addArea = (areaName: string, laid: readonly RoadPlan[], local: Box, x: number, z: number, swap: boolean, size: SizeSpec, greens: Pt[][], centre: Pt) => {
    const onto = ([px, pz]: Pt): Pt => (swap ? [x + pz, z + px] : [x + px, z + pz]);
    const index = plans.length;
    const plan: Plan = {
      name: areaName,
      rect: swap ? { x, z, w: local.d, d: local.w } : { x, z, w: local.w, d: local.d },
      centre: onto(centre),
      roads: [],
      greens: greens.map((g) => g.map(onto)),
      spec: size,
      ports: [],
    };
    /** Whether a point is `PORT_IN` or further inside the world. */
    const inWorld = ([px, pz]: Pt) => Math.min(px, pz, W - px, D - pz) >= PORT_IN;
    laid.forEach((r, k) => {
      // (Each way out cut back from the world's edge.)
      const all = r.points.map(onto);
      let [from, to] = [0, all.length];
      if (r.ends[1] === 'out') while (to - from > 2 && !inWorld(all[to - 1])) to--;
      if (r.ends[0] === 'out') while (to - from > 2 && !inWorld(all[from])) from++;
      const points = all.slice(from, to);
      const road: RoadPlan = { ...r, points, width: Math.max(r.width, ROAD_WIDTH), kind: k === 0 ? 'street' : 'lane', area: index, ...(r.lots && swap ? { lots: (-r.lots) as 1 | -1 } : {}) };
      const at = roads.push(road) - 1;
      plan.roads.push(at);
      for (const e of [0, 1] as const) {
        if (r.ends[e] !== 'out') continue;
        const [p, q] = e === 0 ? [points[0], points[1]] : [points[points.length - 1], points[points.length - 2]];
        const l = dist(p, q) || 1;
        plan.ports.push({ road: at, end: e, area: index, p, dir: [(p[0] - q[0]) / l, (p[1] - q[1]) / l], used: false });
      }
    });
    plans.push(plan);
  };
  addArea(name ?? rolledName, mainRoads, box, ox, oz, false, mainSpec, layout.greens ?? [], layout.centre ?? mainRoads[0].points[Math.floor(mainRoads[0].points.length / 2)]);

  // ---- a river's town bridge, an island's causeways -------------------------------------------
  /** Bridges and causeways laid (their ends, for the hamlets to keep clear of). */
  const crossings: Pt[][] = [];
  /** A straight road from `a` to `b` over the water (`bridge`: its deck, set once the roads are cut to the world). */
  const crossing = (a: Pt, b: Pt, ends: RoadPlan['ends'], roadName: string): RoadPlan => ({
    points: evenly([a, b], 1).map(([x, z]): Pt => [round2(x), round2(z)]),
    width: ROAD_WIDTH,
    ends,
    kind: 'country',
    name: roadName,
    bridge: [0, 0],
  });
  if (river) {
    // From a straight stretch of the waterfront street nearest the town's middle, a T square across the river,
    // on 12 m past its far bank: a way on from the town, as a port.
    const at = plans[0].roads[0];
    const pts = roads[at].points;
    const ls = lengths(pts);
    const L = ls[ls.length - 1];
    const tries: number[] = [];
    for (let s = 30; s <= L - 30; s += 4) if (straightAround(pts, s, 14, T_BEND, ls)) tries.push(s);
    tries.sort((x, y) => Math.abs(pointAt(pts, x, ls).p[0] - plans[0].centre[0]) - Math.abs(pointAt(pts, y, ls).p[0] - plans[0].centre[0]));
    for (const s of tries) {
      const p = pointAt(pts, s, ls).p;
      const span = across(river, p, 160);
      if (!span || span.wet[1] - span.wet[0] + 2 * DECK_PAST > BRIDGE_SPAN || land.wet(...span.far, 2)) continue;
      const laid = roads.push(crossing(p, span.far, ['join', 'join'], `${plans[0].name} Bridge`)) - 1;
      plans[0].ports.unshift({ road: laid, end: 1, area: 0, p: span.far, dir: span.dir, used: false });
      crossings.push([p, span.far]);
      break;
    }
    if (!crossings.length) failed.push(`no bridge from ${plans[0].name} over the river`);
  }
  /** An island's causeways: the inland end of each (where a country road comes to it), the way inland, the edge it leaves by (`FACING`). */
  const causeways: { p: Pt; dir: Pt; side: number; road: number }[] = [];
  if (spec.water === 'island') {
    // On the side of the island nearest the main town but its own shore's; a second (a town's, a city's) opposite.
    const c = plans[0].centre;
    const near = [c[1], W - c[0], D - c[1], c[0]];
    const own = waterSide === 'zmin' ? 0 : 2;
    const first = [0, 1, 2, 3].filter((k) => k !== own).sort((x, y) => near[x] - near[y])[0];
    for (const side of [first, (first + 2) % 4].slice(0, spec.exits[1])) {
      const out = EDGE_OUT[side];
      // Along the middle of that side, as near the town as it may be.
      const along = side % 2 ? Math.max(0.3 * D, Math.min(0.7 * D, c[1])) : Math.max(0.3 * W, Math.min(0.7 * W, c[0]));
      const edge: Pt = side % 2 ? [side === 1 ? W : 0, along] : [along, side === 2 ? D : 0];
      // In from the edge to the coast, then on inland.
      let d = 0;
      while (d < W / 2 && land.water[land.cell(edge[0] - out[0] * (d + 0.5), edge[1] - out[1] * (d + 0.5))]) d += 1;
      const inland: Pt = [round2(edge[0] - out[0] * (d + CAUSEWAY_IN)), round2(edge[1] - out[1] * (d + CAUSEWAY_IN))];
      const beyond: Pt = [edge[0] + out[0] * 40, edge[1] + out[1] * 40];
      const road = roads.push(crossing(inland, beyond, ['join', 'out'], `${EDGE_NAMES[side]} Causeway`)) - 1;
      causeways.push({ p: inland, dir: [-out[0], -out[1]], side, road });
      crossings.push([inland, edge]);
    }
  }

  // ---- hamlets: where it's dry and flat, far enough from the others, near enough to one --------
  const mainCentre = plans[0].centre;
  /** Whether any of the land within `by` m of a box is water. */
  const wetNearRect = (r: Rect, by: number) => {
    for (let z = r.z - by; z <= r.z + r.d + by; z += CELL / 2) for (let x = r.x - by; x <= r.x + r.w + by; x += CELL / 2) if (land.water[land.cell(x, z)]) return true;
    return false;
  };
  const hamlets = between(rand, spec.hamlets);
  /**
   * A place among mountains (its towns follow its slopes): a hamlet only where a country road from
   * the area nearest it can reach it within its grade, the two boxes' heights (their land's mean)
   * no further apart than `REACHABLE` of a country road's climb over the gap between them; and the
   * valley floor before the slopes (`MOUNTAIN_LOW`). Where none can go, it is left out (its houses
   * go to the others: `missing`).
   */
  const reachable = !spec.classic && spec.options?.hills === 'mountains';
  const boxHeight = (r: Rect) => {
    let [sum, n] = [0, 0];
    for (let z = Math.max(0, Math.floor(r.z / CELL)); z <= Math.min(gd - 1, Math.floor((r.z + r.d) / CELL)); z++) {
      for (let x = Math.max(0, Math.floor(r.x / CELL)); x <= Math.min(gw - 1, Math.floor((r.x + r.w) / CELL)); x++) {
        sum += land.height[z * gw + x];
        n++;
      }
    }
    return n ? sum / n : 0;
  };
  const rectGap = (a: Rect, b: Rect) => Math.hypot(Math.max(0, a.x - (b.x + b.w), b.x - (a.x + a.w)), Math.max(0, a.z - (b.z + b.d), b.z - (a.z + a.d)));
  const candidates: Pt[] = Array.from({ length: 700 }, (): Pt => [range(rand, EDGE + 80, W - EDGE - 80), range(rand, EDGE + 60, D - EDGE - 60)]);
  /** Which quarter of the world a point is in: the hamlets go first to those without an area. */
  const quarter = (p: Pt) => (p[0] < W / 2 ? 0 : 1) + (p[1] < D / 2 ? 0 : 2);
  for (let n = 0, relax = 0; n < hamlets; n++) {
    const held = new Set(plans.map((a) => quarter(a.centre)));
    const swap = rand() < 0.35;
    const hb = spec.hamlet.box;
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
      if (spec.water === 'river' || spec.water === 'island' ? wetNearRect(rect, 24) : land.wet(p[0], p[1], Math.max(bw, bd) / 2 + 30)) continue;
      if (region && (wetNearRect(rect, 24) || !region.holds(rect, 24))) continue;
      // (Clear of the bridges and causeways.)
      if (crossings.some(([a, b]) => evenly([a, b], 4).some((q) => offRect(rect, q) < 50))) continue;
      let steep = 0;
      for (let z = rect.z; z < rect.z + bd; z += CELL * 2) for (let x = rect.x; x < rect.x + bw; x += CELL * 2) steep = Math.max(steep, land.slope(land.cell(x, z)));
      if (steep > 0.35 - relax * 0.1 && !relax) continue;
      if (reachable) {
        const joins = plans.reduce((a, b) => (dist(b.centre, p) < dist(a.centre, p) ? b : a));
        if (Math.abs(boxHeight(rect) - boxHeight(joins.rect)) > REACHABLE * COUNTRY_GRADE * rectGap(rect, joins.rect)) continue;
      }
      let score = Math.abs(near - AREA_AIM) / 40 + steep * 40 + land.at(p[0], p[1]) * (reachable ? MOUNTAIN_LOW : valley ? 0.08 : 0.02) + (held.has(quarter(p)) ? 0 : -5);
      // A fishing cove on the shore; a hamlet across the lake from the town.
      if (n === 0 && spec.water === 'sea') score += land.wet(p[0], p[1], Math.max(bw, bd) / 2 + 70) ? -6 : 6;
      if (n === 0 && river) score += bank(p) !== bank(mainCentre) ? -6 : 6;
      if (n === 0 && spec.water === 'lake') score += ((p[0] - lake[0]) * (mainCentre[0] - lake[0]) + (p[1] - lake[1]) * (mainCentre[1] - lake[1]) < 0 ? -6 : 6) + (land.wet(p[0], p[1], Math.max(bw, bd) / 2 + 60) ? -3 : 3);
      score += rand() * 0.5;
      if (!best || score < best.score) best = { p, score };
    }
    if (!best) {
      if (relax++ < 1) n--;
      continue;
    }
    const { shapes, scale: hs, fill, extend } = spec.hamlet;
    const local = shapes[(plans.length - 1) % shapes.length](rand, hb, hs);
    const laid = settleRoads(joinUp(rand, local, { box: hb, lots: 0, scale: hs, fill, extend }, COURTS), hb);
    let areaName = worldName(content, spec.names, rand);
    for (let k = 0; k < 12 && [...plans.map((a) => a.name)].includes(areaName); k++) areaName = worldName(content, spec.names, rand);
    if (plans.some((a) => a.name === areaName)) areaName = `${areaName} End`;
    const own: SizeSpec = { box: hb, lots: between(rand, spec.hamlet.lots), scale: hs, places: spec.hamlet.places, farms: spec.hamlet.farms ?? [0, 0] };
    addArea(areaName, laid, hb, Math.round(best.p[0] - bw / 2), Math.round(best.p[1] - bd / 2), swap, own, local.greens ?? [], local.centre ?? [hb.w / 2, hb.d / 2]);
  }

  // Fewer hamlets than the world has at least: their houses go to the others (the main town first; `LotPlacer` spills them on).
  const missing = spec.hamlets[0] - (plans.length - 1);
  if (missing > 0) plans[0].spec = { ...mainSpec, lots: mainSpec.lots + missing * spec.hamlet.lots[0] };

  step?.('land');
  // ---- country roads: between the areas (a tree of them and one more for a loop), and out ----
  const areaBlock = new Uint8Array(gw * gd);
  for (const [a, plan] of plans.entries()) {
    const r = plan.rect;
    for (let z = Math.floor((r.z - 6) / CELL); z <= Math.floor((r.z + r.d + 6) / CELL); z++) {
      for (let x = Math.floor((r.x - 6) / CELL); x <= Math.floor((r.x + r.w + 6) / CELL); x++) if (x >= 0 && z >= 0 && x < gw && z < gd) areaBlock[z * gw + x] = a + 1;
    }
  }
  const rim = new Uint8Array(gw * gd);
  const climb = spec.climb ?? 0;
  for (let i = 0; i < gw * gd; i++) if (i % gw < 2 || i % gw >= gw - 2 || i < gw * 2 || i >= gw * (gd - 2)) rim[i] = 1;
  const wetNear = new Uint8Array(gw * gd);
  for (let i = 0; i < gw * gd; i++) if (land.water[i]) for (const d of [0, -1, 1, -gw, gw, -gw - 1, -gw + 1, gw - 1, gw + 1]) if (i + d >= 0 && i + d < gw * gd) wetNear[i + d] = 1;
  /** What a cell costs a country road (Infinity where it may not go: on or by the water, in an area but within 26 m of `ends`, its ways in and out). */
  const roadCost = (ends: readonly Pt[]) => (i: number) => {
    if (wetNear[i]) return Infinity;
    if (areaBlock[i]) {
      const c = land.centre(i);
      if (!ends.some((e) => dist(c, e) < 26)) return Infinity;
    }
    // (Not along the world's edge: smoothed, it would wander out of the world and back. In hilly
    // land and the mountains, along the valleys.)
    return 1 + land.slope(i) * 40 + land.woods[i] * 0.3 + (rim[i] ? 12 : 0) + (climb ? land.height[i] * climb : 0);
  };
  /**
   * A place's land (its town follows its slopes: `landFollow.ts`): what a step costs a country road
   * where it climbs more steeply than `ROUTE_GRADE`, so it keeps within a country road's grade
   * where the land lets it (along the valleys, through the passes, round a steep bank, rather than
   * over the massif). None in a classic world.
   */
  const rise: Rise | undefined = spec.classic
    ? undefined
    : (a, b) => {
        const g = Math.abs(land.height[b] - land.height[a]) / (a % gw !== b % gw && Math.floor(a / gw) !== Math.floor(b / gw) ? CELL * Math.SQRT2 : CELL);
        return g > ROUTE_GRADE ? (g - ROUTE_GRADE) * ROUTE_STEEP : 0;
      };
  /** Whether the land within `SPILL_LOOK` m of `p` is gentle enough for a lot out along a country road (any, in a classic world). */
  const gentleBy = (p: Pt) => {
    if (spec.classic) return true;
    for (let dz = -SPILL_LOOK; dz <= SPILL_LOOK; dz += CELL) for (let dx = -SPILL_LOOK; dx <= SPILL_LOOK; dx += CELL) if (land.slope(land.cell(p[0] + dx, p[1] + dz)) > PLACE_STEEP) return false;
    return true;
  };
  /** As `roadCost`, but along the water's edge too (not over it), and less put off by the slope: for when that finds no way. */
  const relaxedCost = (ends: readonly Pt[]) => {
    const strict = roadCost(ends);
    return (i: number) => (land.water[i] ? Infinity : wetNear[i] ? 30 + land.slope(i) * 15 : Math.min(strict(i), 1 + land.slope(i) * 15 + land.woods[i] * 0.3));
  };
  /**
   * The way a country road goes from any of `from` (a port, or a T off a road), heading its way,
   * to a cell `goal` accepts (and a heading it accepts, when it is given one, more loosely if
   * asked: what ending there costs, -1 if it doesn't): as `steer` finds it over `cost`, or where
   * that finds no way over `fallback` (each such cell `DETOUR` more), never heading back against
   * `toward`, else coming in more loosely, else heading any way; else as `astar` does (`steered`
   * false). Its cells, which of `from` it leaves by and the heading it ends on.
   */
  const route = (cost: (i: number) => number, fallback: (i: number) => number, from: readonly Way[], goal: (i: number, h?: number, loose?: boolean) => number, h: (i: number) => number, toward?: Pt): { cells: number[]; from: number; h?: number; steered: boolean } | null => {
    const starts = from.map((w) => ({ cell: outCell(w), h: octant(w.dir), g: wayCost(w) }));
    const grid = Float32Array.from({ length: gw * gd }, (_, i) => {
      const c = cost(i);
      return c < Infinity ? c : fallback(i) + DETOUR;
    });
    // (Coming in loosely costs it more.)
    const either = (i: number, hd: number) => {
      const e = goal(i, hd);
      if (e >= 0) return e;
      const l = goal(i, hd, true);
      return l >= 0 ? l + LOOSE_COST : -1;
    };
    const found = (toward && steer(gw, gd, grid, starts, either, h, toward, undefined, rise)) || steer(gw, gd, grid, starts, goal, h, undefined, LOOSE_LIMIT, rise);
    if (found) return { cells: found.cells, from: found.start, h: found.h, steered: true };
    for (const c of [cost, fallback]) {
      const cells = astar(gw, gd, c, [starts[0].cell], (i) => goal(i) >= 0, h, undefined, rise);
      if (cells) return { cells, from: 0, steered: false };
    }
    return null;
  };
  /** Leaving by a port taken already: from the port itself (`RETAKE_COST`), or off the road on from it (`VIA_COST`). */
  const takenCost = (w: Way) => (w.port?.used ? (w.p === w.port.p ? RETAKE_COST : VIA_COST) : 0);
  /** What leaving by a way costs `steer` (in cells) before it is out over the land: its lead, and more for a T than a port, and for a port taken already. */
  const wayCost = (w: Way) => ((w.reach ?? CELL) + (w.reach ? SIDE_COST : 0) + takenCost(w)) / CELL;
  /** The cell a country road's way over the land starts from, leaving by `w` (or ends on, coming in by it): a cell out in front of it, or where its reach ends. */
  const outCell = (w: Way) => land.cell(w.p[0] + w.dir[0] * (w.reach ?? CELL), w.p[1] + w.dir[1] * (w.reach ?? CELL));
  /** A way's way in, from its `outCell`: heading back into it, give or take a twelfth of a turn (`loose`: three sixteenths). */
  const into = (q: Way) => (h?: number, loose = false) => h === undefined || along(h, [-q.dir[0], -q.dir[1]]) > (loose ? 0.38 : 0.85);
  /** The world's edges (north, east, south, west): outwards. */
  const FACING: readonly Pt[] = [[0, -1], [1, 0], [0, 1], [-1, 0]];
  /** A country road from `a` along `cells`, to `b`: smoothed into a curve, leaving and arriving straight (on across an area's land from a T). */
  const countryRoad = (a: Way, cells: number[], b: Way | null, ends: RoadPlan['ends'], roadName: string): RoadPlan => {
    const mid = simplify(cells.map((i) => land.centre(i)), 5).slice(1, b ? -1 : undefined);
    const lead = (q: Way, k: number): Pt => [q.p[0] + q.dir[0] * k, q.p[1] + q.dir[1] * k];
    const len = (q: Way, k: number) => Math.max(k, q.reach ?? 0);
    // Near an end, only the way's points out in front of it (no doubling back to it: a kink).
    const before = (q: Way, p: Pt, near: number) => dist(p, q.p) > near && (dist(p, q.p) > 48 || q.dir[0] * (p[0] - q.p[0]) + q.dir[1] * (p[1] - q.p[1]) > dist(p, q.p) * 0.5);
    // (No lead out over the water: a port on the shore is come to from the land.)
    const leads = (q: Way, k: number): Pt[] => (land.wet(...lead(q, len(q, k)), 4) ? [] : [lead(q, len(q, k))]);
    const make = (k: number) => {
      const tail: Pt[] = b ? [...leads(b, k), b.p] : [];
      const ctrl = [a.p, ...leads(a, k), ...mid.filter((p) => before(a, p, len(a, k) + 8) && (!b || before(b, p, len(b, k) + 8))), ...tail];
      return curve(evenly(ctrl, CTRL_APART));
    };
    // (Shorter leads if the longer ones swing it out over the water.)
    const wet = (pts: readonly Pt[]) => pts.some(([x, z]) => land.wet(x, z, 2));
    let points = make(14);
    for (const k of [7, 3]) if (wet(points)) points = make(k);
    // (Longer leads where it comes into a way, or leaves one, at a slant and turns too sharply
    // just before it: the bend in further out, where it has room to be a road's.)
    for (const k of LONG_LEADS) {
      if (sharpestTurn(points, 20).angle <= COUNTRY_TURN - TURN_SPARE) break;
      const longer = make(k);
      if (!wet(longer) && sharpestTurn(longer, 20).angle < sharpestTurn(points, 20).angle) points = longer;
    }
    return { points, width: COUNTRY_WIDTH, ends, kind: 'country', name: roadName };
  };
  /** A country road from `a` along `cells` out of the world over edge `side` (`FACING`), 40 m on past its last cell. */
  const toEdge = (a: Way, cells: number[], side: number, roadName: string): RoadPlan => {
    const last = land.centre(cells[cells.length - 1]);
    const road = countryRoad(a, cells, null, ['join', 'out'], roadName);
    road.points = curve(evenly([...simplify(road.points, 0.3), [last[0] + FACING[side][0] * 40, last[1] + FACING[side][1] * 40]], CTRL_APART));
    return road;
  };
  /**
   * A country road's course from `a` along `cells` to `b` (null: on out of the world), in pieces
   * where it runs along a road laid already (`SHARED_RUN`): the stretches it would run alongside
   * that road are left to it, and each piece between joins it square in a T (merged into it,
   * rather than side by side a few metres off); from an end right by one it runs along (`shortcut`),
   * straight onto it, or off it straight into the end; each T facing the side the piece goes. Each
   * piece's ends (`a`, `b`: the course's own, or such a T) and its cells, and whether it is the
   * course's first or last stretch; one piece, the whole course, if it runs alongside none.
   */
  const alongExisting = (a: Way, cells: readonly number[], b: Way | null): { a: Way; cells: number[]; b: Way | null; first: boolean; last: boolean }[] => {
    // The roads (not trails) passing within `SHARED_REACH` cells of each cell, but for the streets
    // its own ends go on from where they are their ports (it leaves a port in line with its
    // street). (An end off a country road from a port, `via`, is out beside that street: it may
    // run along it.)
    const atPort = (w: Way | null) => (w?.port && w.p === w.port.p ? w.port.road : undefined);
    const own = new Set([atPort(a), atPort(b)]);
    const by = new Map<number, number[]>();
    const R = SHARED_REACH;
    roads.forEach((r, i) => {
      if (r.kind === 'trail' || own.has(i)) return;
      // (Each cell it passes over once; off the world, none.)
      const under = new Set(r.points.map(([x, z]) => (x < 0 || z < 0 || x >= W || z >= D ? -1 : Math.floor(z / CELL) * gw + Math.floor(x / CELL))));
      under.delete(-1);
      for (const u of under) {
        const [cx, cz] = [u % gw, Math.floor(u / gw)];
        for (let dz = -R; dz <= R; dz++) {
          for (let dx = -R; dx <= R; dx++) {
            if (cx + dx < 0 || cz + dz < 0 || cx + dx >= gw || cz + dz >= gd) continue;
            const at = (cz + dz) * gw + cx + dx;
            const list = by.get(at);
            if (!list) by.set(at, [i]);
            else if (!list.includes(i)) list.push(i);
          }
        }
      }
    });
    // Per cell, those it runs along there (within `SHARED_ANGLE` of their way: not crossing them).
    const on = cells.map((c, i) => {
      const [p, q] = [land.centre(cells[Math.max(0, i - 1)]), land.centre(cells[Math.min(cells.length - 1, i + 1)])];
      const l = dist(p, q) || 1;
      const c0 = land.centre(c);
      return (by.get(c) ?? []).filter((road) => {
        const { t } = pointAt(roads[road].points, nearest(roads[road].points, c0[0], c0[1]).s);
        return Math.abs(((q[0] - p[0]) * t[0] + (q[1] - p[1]) * t[1]) / l) >= Math.cos(SHARED_ANGLE);
      });
    });
    const runs: { from: number; to: number; roads: Set<number> }[] = [];
    for (let i = 0; i < cells.length; i++) {
      if (!on[i].length) continue;
      let j = i;
      for (let k = i + 1; k < cells.length && k <= j + SHARED_GAP + 1; k++) if (on[k].length) j = k;
      if (j - i + 1 >= SHARED_RUN) runs.push({ from: i, to: j, roads: new Set(on.slice(i, j + 1).flat()) });
      i = j;
    }
    if (!runs.length) return [{ a, cells: [...cells], b, first: true, last: true }];
    const centre = (i: number) => land.centre(cells[i]);
    // A T on the nearest of `list` to `cell` (or a point), facing `side` (a point; else it).
    const tee = (list: Iterable<number>, cell: number | Pt, side?: Pt): Way => {
      const c = typeof cell === 'number' ? land.centre(cell) : cell;
      const [road, at] = [...list].map((r) => [r, nearest(roads[r].points, c[0], c[1])] as const).sort((x, y) => x[1].dist - y[1].dist)[0];
      const { p, t } = pointAt(roads[road].points, at.s);
      const n = leftOf(t);
      const k = side ? (n[0] * (side[0] - p[0]) + n[1] * (side[1] - p[1]) >= 0 ? 1 : -1) : at.side;
      return { p, dir: [n[0] * k, n[1] * k] };
    };
    // (The way a piece goes off a T, or comes into one: a few cells on from it, not the cell by it,
    // which may lie over the road where the course crosses it at a slant.)
    const ahead = (from: number, to: number) => centre(Math.min(to, from + SHARED_RUN));
    const behind = (from: number, to: number) => centre(Math.max(from, to - SHARED_RUN));
    /**
     * Where the course's end `w` (`into`: its start; else its end) lies by a road it runs along in
     * `run`, and that road from by `w` to where the course first runs along it from there is no
     * further than the course's own way: a T on it by `w`, to go straight onto it (the course's
     * cells before dropped: a stub, or a loop round back onto the road beside it). Else null.
     */
    const shortcut = (w: Way | null, run: (typeof runs)[number], into: boolean): Way | null => {
      if (!w) return null;
      const fits = [...run.roads].filter((r) => {
        const ls = lengths(roads[r].points);
        const m = nearest(roads[r].points, w.p[0], w.p[1], ls);
        if (m.dist > SHARED_REACH * CELL) return false;
        // (The cell it is along it from, and the course's way from `w` to there.)
        let at = into ? run.from : run.to;
        while (!on[at].includes(r)) at += into ? 1 : -1;
        let way = dist(w.p, centre(into ? 0 : cells.length - 1));
        for (let i = into ? 0 : at; i < (into ? at : cells.length - 1); i++) way += dist(centre(i), centre(i + 1));
        const c = centre(at);
        return Math.abs(m.s - nearest(roads[r].points, c[0], c[1], ls).s) <= way + CELL;
      });
      return fits.length ? tee(fits, w.p) : null;
    };
    // (From a port alongside one only if the port is near it: straight onto it, a short way.)
    const fromPort = atPort(a) !== undefined;
    if (runs[0].from === 0 && fromPort && !shortcut(a, runs[0], true)) runs.shift();
    if (!runs.length) return [{ a, cells: [...cells], b, first: true, last: true }];
    const end = runs[runs.length - 1];
    const [onto, off] = [shortcut(a, runs[0], true), shortcut(b, end, false)];
    const out: { a: Way; cells: number[]; b: Way | null; first: boolean; last: boolean }[] = [];
    const add = (from: number, to: number, pa: Way, pb: Way | null, first: boolean, last: boolean) => {
      if (to >= from) out.push({ a: pa, cells: cells.slice(from, to + 1), b: pb, first, last });
    };
    // (By it from its start, or by it from a port on: straight onto it, no cells.)
    if (onto) out.push({ a, cells: [], b: onto, first: true, last: false });
    else add(0, runs[0].from - 1, a, tee(runs[0].roads, cells[Math.max(0, runs[0].from - 1)], behind(0, runs[0].from - 1)), true, false);
    for (let k = 0; k + 1 < runs.length; k++) {
      const [r, n] = [runs[k], runs[k + 1]];
      // (Not off a road and back onto it: it goes on along it.)
      if ([...r.roads].some((x) => n.roads.has(x))) continue;
      add(r.to + 1, n.from - 1, tee(r.roads, cells[r.to + 1], ahead(r.to + 1, n.from - 1)), tee(n.roads, cells[n.from - 1], behind(r.to + 1, n.from - 1)), false, false);
    }
    // (By it to its end: off it straight to the end.)
    if (off) out.push({ a: off, cells: [], b, first: false, last: true });
    else add(end.to + 1, cells.length - 1, tee(end.roads, cells[Math.min(cells.length - 1, end.to + 1)], ahead(end.to + 1, cells.length - 1)), b, false, true);
    return out;
  };
  /** A piece of a country road (`alongExisting`): straight from its start onto the road beside it if it has no cells, out of the world over edge `side` if it is the last and goes out. */
  const layPiece = (part: ReturnType<typeof alongExisting>[number], side = -1): RoadPlan =>
    !part.cells.length
      ? { points: curve(evenly([part.a.p, part.b!.p], CTRL_APART)), width: COUNTRY_WIDTH, ends: ['join', 'join'], kind: 'country', name: '' }
      : part.last && side >= 0
        ? toEdge(part.a, part.cells, side, '')
        : countryRoad(part.a, part.cells, part.b, ['join', 'join'], '');
  /**
   * Whether the roads laid so far and `extra` (not trails), joined as the simulation joins them (an
   * end within half the wider one's width and a metre of the other's middle), join `p` to `q`.
   */
  const joins = (extra: readonly RoadPlan[], p: Pt, q: Pt): boolean => {
    const all = [...roads, ...extra].filter((r) => r.kind !== 'trail');
    const index = new RoadIndex(all);
    const parent = all.map((_, i) => i);
    const find = (i: number): number => (parent[i] === i ? i : (parent[i] = find(parent[i])));
    all.forEach((r, i) => {
      for (const [x, z] of [r.points[0], r.points[r.points.length - 1]]) {
        for (const [j, hit] of index.around(x, z, 12)) if (j !== i && hit.dist <= Math.max(r.width, all[j].width) / 2 + 1) parent[find(i)] = find(j);
      }
    });
    const at = ([x, z]: Pt) => [...index.around(x, z, 1).keys()].map(find);
    const from = at(p);
    return at(q).some((r) => from.includes(r));
  };
  /**
   * A country road (`whole`, laid from `a` along `cells` to `b`, or out of the world over edge
   * `side`) as it goes down: in pieces where it runs along a road laid already (`alongExisting`),
   * if they and the roads join its ends still and none turns or winds worse than it would; else whole. The roads, and whether they leave
   * its first end and come to its last.
   */
  const layCourse = (whole: RoadPlan, a: Way, cells: readonly number[] | undefined, b: Way | null, side = -1): { laid: RoadPlan[]; first: boolean; last: boolean } => {
    const parts = cells ? alongExisting(a, cells, b) : null;
    if (!parts || (parts.length === 1 && parts[0].first && parts[0].last)) return { laid: [whole], first: true, last: true };
    // (Not a stub where a port touches the road beside it already: the two join as they are.)
    const laid = parts.map((part) => settle(layPiece(part, side))).filter((r) => lengths(r.points)[r.points.length - 1] > COUNTRY_WIDTH / 2 + 1);
    // (Nor pieces that wind or turn worse than the road would whole: a short one between two Ts
    // close together can double back.)
    const worst = Math.max(0, ...laid.map(fault));
    if (worst > fault(settle(whole)) || !joins(laid, whole.points[0], whole.points[whole.points.length - 1])) return { laid: [whole], first: true, last: true };
    return { laid, first: !!parts[0]?.first, last: !!parts[parts.length - 1]?.last };
  };
  /**
   * Where a country road leaves port `q` towards `toward`: the port itself; one another country
   * road leaves already, off that road 30 m out from it, square to it (rather than both from the
   * port, running together a while).
   */
  const via = (q: Port, toward: Pt): Way => {
    const self: Way = { p: q.p, dir: q.dir, port: q };
    if (!q.used) return self;
    // (Not its own road: a bridge's far end is a port.)
    const r = roads.find((o, i) => i !== q.road && o.kind === 'country' && [o.points[0], o.points[o.points.length - 1]].some((e) => dist(e, q.p) < 1));
    if (!r) return self;
    const ls = lengths(r.points);
    const L = ls[ls.length - 1];
    if (L < 70) return self;
    const s = dist(r.points[0], q.p) < 1 ? 30 : L - 30;
    const { p, t } = pointAt(r.points, s, ls);
    const n = leftOf(t);
    const k = n[0] * (toward[0] - p[0]) + n[1] * (toward[1] - p[1]) >= 0 ? 1 : -1;
    return { p, dir: [n[0] * k, n[1] * k], port: q };
  };
  /** How far a way heading `dir` from `p` is from heading for `toward`, as metres of road: nothing within `SWING`, then more the more it would have to swing round. */
  const swing = (p: Pt, dir: Pt, toward: Pt) => {
    const l = dist(p, toward) || 1;
    const cos = (dir[0] * (toward[0] - p[0]) + dir[1] * (toward[1] - p[1])) / l;
    return Math.max(0, Math.acos(Math.max(-1, Math.min(1, cos))) - SWING) * SWING_COST;
  };
  /**
   * Which of `list` (roads) pass within `R` m of a point round `rect` (to 40 m out): -1 none, the
   * road if only one does, -2 if more.
   */
  const roadsNear = (list: readonly number[], rect: Rect, R: number) => {
    const G = 2;
    const [x0, z0] = [rect.x - 40, rect.z - 40];
    const [w, d] = [Math.ceil((rect.w + 80) / G), Math.ceil((rect.d + 80) / G)];
    const grid = new Int32Array(w * d).fill(-1);
    const reach = Math.ceil(R / G);
    const add = (i: number) => {
      const pts = roads[i].points;
      for (let k = 0; k + 1 < pts.length; k++) {
        const steps = Math.max(1, Math.ceil(dist(pts[k], pts[k + 1]) / G));
        for (let j = 0; j <= steps; j++) {
          const [px, pz] = [pts[k][0] + ((pts[k + 1][0] - pts[k][0]) * j) / steps, pts[k][1] + ((pts[k + 1][1] - pts[k][1]) * j) / steps];
          const [cx, cz] = [Math.floor((px - x0) / G), Math.floor((pz - z0) / G)];
          for (let z = Math.max(0, cz - reach); z <= Math.min(d - 1, cz + reach); z++) {
            const dz = z0 + (z + 0.5) * G - pz;
            for (let x = Math.max(0, cx - reach); x <= Math.min(w - 1, cx + reach); x++) {
              const at = z * w + x;
              const dx = x0 + (x + 0.5) * G - px;
              if (grid[at] === i || grid[at] === -2 || dx * dx + dz * dz >= R * R) continue;
              grid[at] = grid[at] === -1 ? i : -2;
            }
          }
        }
      }
    };
    list.forEach(add);
    const at = ([px, pz]: Pt) => {
      const [x, z] = [Math.floor((px - x0) / G), Math.floor((pz - z0) / G)];
      return x < 0 || z < 0 || x >= w || z >= d ? -1 : grid[z * w + x];
    };
    return { at, add };
  };
  /** Per area, `roadsNear` its own roads, and how many of the world's roads it has been told about. */
  const sideGrids = new Map<number, { roads: number; apart: ReturnType<typeof roadsNear>; depth: ReturnType<typeof roadsNear> }>();
  /** Which of area `a`'s roads (and the country roads into it) pass within `SIDE_APART` and `SIDE_DEPTH` of a point. */
  const gridsOf = (a: number) => {
    const r = plans[a].rect;
    let grids = sideGrids.get(a);
    if (!grids) sideGrids.set(a, (grids = { roads: roads.length, apart: roadsNear(plans[a].roads, r, SIDE_APART), depth: roadsNear(plans[a].roads, r, SIDE_DEPTH) }));
    // (Country roads into the area since: from a T.)
    for (; grids.roads < roads.length; grids.roads++) {
      if (roads[grids.roads].kind === 'country' && roads[grids.roads].points.some(([x, z]) => x > r.x - 30 && z > r.z - 30 && x < r.x + r.w + 30 && z < r.z + r.d + 30)) grids.depth.add(grids.roads);
    }
    return { apart: grids.apart.at, depth: grids.depth.at };
  };
  /**
   * Where way `w` (out of another area), straight on, comes to one of area `b`'s roads within
   * `STRAIGHT_ON` m: square to it give or take `SIDE_FACING`, where it runs straight and not by a
   * junction, over dry land and nobody else's area, clear of `b`'s other roads by a lot's depth.
   * The point (on the road's middle) and how far; null if it doesn't.
   */
  const straightOn = (w: Way, b: number): { p: Pt; len: number } | null => {
    const { apart, depth } = gridsOf(b);
    let to = -1;
    for (let d = 2; d <= STRAIGHT_ON; d += 2) {
      const q: Pt = [w.p[0] + w.dir[0] * d, w.p[1] + w.dir[1] * d];
      const owner = areaBlock[land.cell(q[0], q[1])];
      if (land.wet(q[0], q[1], 3) || (owner && owner !== b + 1 && owner !== (w.port?.area ?? -2) + 1)) return null;
      const by = depth(q);
      if (by === -1) continue;
      // (The first of its roads it comes near is the one it meets, and nothing else on the way.)
      if (by === -2 || !plans[b].roads.includes(by) || (to >= 0 && by !== to)) return null;
      to = by;
      const pts = roads[by].points;
      const ls = lengths(pts);
      const at = nearest(pts, q[0], q[1], ls);
      if (at.dist > 2) continue;
      const { p, t } = pointAt(pts, at.s, ls);
      const ok = Math.abs(t[0] * w.dir[0] + t[1] * w.dir[1]) <= Math.sin(SIDE_FACING) && straightAround(pts, at.s, 14, STRAIGHT_BEND, ls) && [-1, by].includes(apart(p)) && at.s > 20 && at.s < ls[ls.length - 1] - 20;
      return ok ? { p, len: dist(w.p, p) } : null;
    }
    return null;
  };
  /**
   * A way out of area `a` towards `toward` by a T off one of its streets or lanes: where it runs
   * straight (and not by a junction), square to it on the side facing `toward` (within
   * `SIDE_FACING`), straight on across the area's land clear of its other roads, its lots' depth
   * from them, and of the water. The nearest such to `toward` with the least to cross first, none
   * two within `SIDE_APART` of each other.
   */
  const sideT = (a: number, toward: Pt): (Way & { score: number })[] => {
    const plan = plans[a];
    const r = plan.rect;
    const near = (p: Pt, m: number) => p[0] > r.x - m && p[1] > r.z - m && p[0] < r.x + r.w + m && p[1] < r.z + r.d + m;
    const { apart, depth } = gridsOf(a);
    const found: (Way & { score: number })[] = [];
    for (const ri of plan.roads) {
      const pts = roads[ri].points;
      const ls = lengths(pts);
      const L = ls[ls.length - 1];
      for (let s = 20; s <= L - 20; s += 4) {
        if (!straightAround(pts, s, 14, T_BEND, ls)) continue;
        const { p, t } = pointAt(pts, s, ls);
        const l = dist(p, toward) || 1;
        const u: Pt = [(toward[0] - p[0]) / l, (toward[1] - p[1]) / l];
        const left = leftOf(t);
        const k = left[0] * u[0] + left[1] * u[1] >= 0 ? 1 : -1;
        const n: Pt = [left[0] * k, left[1] * k];
        if (n[0] * u[0] + n[1] * u[1] < Math.cos(SIDE_FACING)) continue;
        if (![-1, ri].includes(apart(p))) continue;
        let reach = 0;
        for (let d = 4; d < 200; d += 4) {
          const q: Pt = [p[0] + n[0] * d, p[1] + n[1] * d];
          const by = depth(q);
          if (land.wet(q[0], q[1], 4) || (by !== -1 && (by !== ri || d > SIDE_DEPTH))) break;
          const cell = land.cell(q[0], q[1]);
          if (!near(q, 0) && !areaBlock[cell]) {
            // (Over the land beyond, first.)
            if (roadCost([])(cell) < Infinity) reach = d;
            break;
          }
        }
        if (!reach) continue;
        const out: Pt = [p[0] + n[0] * reach, p[1] + n[1] * reach];
        found.push({ p, dir: n, reach, score: reach + dist(out, toward) + swing(out, n, toward) + SIDE_COST });
      }
    }
    found.sort((x, y) => x.score - y.score);
    return found.filter((w, i) => found.slice(0, i).every((o) => dist(o.p, w.p) >= SIDE_APART));
  };
  /**
   * Whether port `q` is no road end any more: another area's street or lane comes up to it (its
   * middle within half the wider one's width and a metre: the simulation joins them, one street
   * on through). No country road leaves it: it ran on along that street a few metres off before
   * turning away, crossing it (Canalside's Queen'ston Lane, by Foundry Street's end).
   */
  const continued = (q: Port) =>
    roads.some((o) => o.area !== undefined && o.area !== q.area && o.kind !== 'trail' && o.kind !== 'country' && nearest(o.points, q.p[0], q.p[1]).dist <= Math.max(o.width, roads[q.road].width) / 2 + 1);
  /**
   * The way a country road leaves area `a` towards `toward` (or comes into it from there): the
   * port nearest it that faces it (a port with a country road on from it already, off that road),
   * else a T off one of its streets facing it (`sideT`) if that swings round less.
   */
  const waysOut = (a: number, toward: Pt): (Way & { score: number })[] => {
    const ports = plans[a].ports.filter((q) => !continued(q)).map((q) => {
      const w = via(q, toward);
      return { ...w, score: dist(w.p, toward) + swing(w.p, w.dir, toward) + takenCost(w) };
    });
    return [...ports, ...sideT(a, toward).slice(0, 3)].sort((x, y) => x.score - y.score);
  };
  /**
   * What is wrong with a country road's course (where it meets another road aside), worst first:
   * 3 if it runs over the water, 2 if it turns sharper than `COUNTRY_TURN` over 40 m anywhere, 1
   * if it winds round further than `WINDING` within `WINDING_OVER` m (a loop), else 0.
   */
  const fault = (road: RoadPlan) =>
    road.points.some(([x, z]) => land.wet(x, z, 1)) ? 3 : sharpestTurn(road.points, 20).angle > COUNTRY_TURN - TURN_SPARE ? 2 : winding(road.points, WINDING_OVER) > WINDING ? 1 : 0;
  /** A country road that turns too sharply (`fault` 2) relaxed till it doesn't (`unkink`), if that keeps it off the water; else as it was. */
  const settle = (road: RoadPlan): RoadPlan => {
    if (fault(road) !== 2) return road;
    const points = unkink(road.points, COUNTRY_TURN - TURN_SPARE);
    return points && !points.some(([x, z]) => land.wet(x, z, 1)) ? { ...road, points } : road;
  };
  /** Whether `p` lies ahead of way `w`, within `angle` (radians) of straight on. */
  const inFront = (w: Way, p: Pt, angle: number) => w.dir[0] * (p[0] - w.p[0]) + w.dir[1] * (p[1] - w.p[1]) >= dist(p, w.p) * Math.cos(angle);
  // (Two areas' streets meeting end to end: their ports are a way on already, kept as they are.)
  for (const q of plans.flatMap((plan) => plan.ports)) if (continued(q)) q.used = true;
  // Prim's tree over the areas, then the shortest edge it left out. (Across a river only the main
  // town and an area on the other bank: over its bridge.)
  const apart = (a: number, b: number) => (river && a && b && bank(plans[a].centre) !== bank(plans[b].centre) ? Infinity : dist(plans[a].centre, plans[b].centre));
  const inTree = [0];
  const edges: [number, number][] = [];
  while (inTree.length < plans.length) {
    let best: [number, number, number] | null = null;
    for (const a of inTree) for (let b = 0; b < plans.length; b++) if (!inTree.includes(b) && (!best || apart(a, b) < best[2])) best = [a, b, apart(a, b)];
    edges.push([best![0], best![1]]);
    inTree.push(best![1]);
  }
  // (A loop or two: more of them in a world of more areas.)
  for (let loops = plans.length >= 6 ? 2 : plans.length >= 3 ? 1 : 0; loops > 0; loops--) {
    let extra: [number, number, number] | null = null;
    for (let a = 0; a < plans.length; a++) {
      for (let b = a + 1; b < plans.length; b++) {
        if (edges.some(([x, y]) => (x === a && y === b) || (x === b && y === a))) continue;
        const d = apart(a, b);
        if (d < 560 && (!extra || d < extra[2])) extra = [a, b, d];
      }
    }
    if (extra) edges.push([extra[0], extra[1]]);
  }
  const roadNames = new Set<string>();
  // (The bridges and causeways laid already have theirs.)
  for (const r of roads) if (r.bridge && r.name) roadNames.add(r.name);
  const uniqueName = (base: string, alt: string) => {
    const pick = !roadNames.has(base) ? base : !roadNames.has(alt) ? alt : `${alt} ${roadNames.size}`;
    roadNames.add(pick);
    return pick;
  };
  for (const [a, b] of edges) {
    // The likeliest ways out of each towards the other, the cheapest way between any of them;
    // if it turns too sharply (its ends, smoothed), again without the way it came in by, then without the one it left by.
    // (By a river, the road keeps to the bank the area other than the main town is on: the main
    // town's way to the far bank is over its bridge.)
    const side = bank(plans[a === 0 ? b : a].centre);
    const onBank = (list: (Way & { score: number })[]) => (river ? list.filter((w) => bank(w.p) === side) : list);
    let outs = onBank(waysOut(a, plans[b].centre)).slice(0, WAYS);
    let ins = onBank(waysOut(b, plans[a].centre)).slice(0, WAYS);
    type Made = { road: RoadPlan; pa: Way; pb: Way; bad: number; cells?: number[] };
    let made = null as Made | null;
    // Near enough, a way out of one straight on to a street of the other, meeting it in a T.
    const direct = [...outs.map((w) => ({ w, to: b })), ...ins.map((w) => ({ w, to: a }))]
      .map(({ w, to }) => ({ w, hit: straightOn(w, to) }))
      .flatMap(({ w, hit }) => (hit ? [{ w, hit, cost: hit.len + wayCost(w) * CELL }] : []))
      .sort((x, y) => x.cost - y.cost)[0];
    if (direct) {
      const pts = curve(evenly([direct.w.p, direct.hit.p], CTRL_APART));
      made = { road: { points: pts, width: COUNTRY_WIDTH, ends: ['join', 'join'], kind: 'country', name: '' }, pa: direct.w, pb: { p: direct.hit.p, dir: [-direct.w.dir[0], -direct.w.dir[1]] }, bad: 0 };
    }
    for (let tries = 0; tries < TRIES && outs.length && ins.length && (!made || made.bad > 0); tries++) {
      // (Into the areas by their ports, or off a road by one, if it comes to it.)
      const ends = [...outs, ...ins].flatMap((w) => (w.port ? [w.port.p, w.p] : []));
      const goals = ins.map((w) => ({ w, cell: outCell(w), into: into(w), extra: wayCost(w) }));
      // (Coming in by the cheapest of them that ends there, heading in.)
      const goal = (i: number, h?: number, loose?: boolean) => {
        let best = -1;
        for (const o of goals) if (o.cell === i && o.into(h, loose) && (best < 0 || o.extra < best)) best = o.extra;
        return best;
      };
      const goalAt = goals.map((o) => land.centre(o.cell));
      const guess = (i: number) => {
        const c = land.centre(i);
        let near = Infinity;
        for (const e of goalAt) near = Math.min(near, dist(c, e));
        return near / CELL;
      };
      const span = dist(plans[a].centre, plans[b].centre) || 1;
      const found = route(roadCost(ends), relaxedCost(ends), outs, goal, guess, [(plans[b].centre[0] - plans[a].centre[0]) / span, (plans[b].centre[1] - plans[a].centre[1]) / span]);
      if (!found) break;
      const end = found.cells[found.cells.length - 1];
      const pb = goals.filter((o) => o.cell === end && o.into(found.h, true)).sort((x, y) => x.extra - y.extra)[0]?.w ?? goals.find((o) => o.cell === end)!.w;
      const pa = outs[found.from];
      const road = countryRoad(pa, found.cells, pb, ['join', 'join'], '');
      const bad = fault(road);
      if (!made || bad < made.bad) made = { road, pa, pb, bad, cells: found.cells };
      // (No way it steers between any of them: none without them either.)
      if (!bad || !found.steered) break;
      if (tries % 2 === 0) ins = ins.filter((w) => w !== pb);
      else outs = outs.filter((w) => w !== pa);
    }
    if (!made) {
      failed.push(`no country road between ${plans[a].name} and ${plans[b].name}`);
      continue;
    }
    const far = b === 0 ? a : a === 0 ? b : dist(plans[a].centre, mainCentre) > dist(plans[b].centre, mainCentre) ? a : b;
    const near = far === a ? b : a;
    const name = () => uniqueName(`${plans[far].name} Road`, `${plans[near].name} Lane`);
    // (In pieces where it runs along a road laid already; a straight road onto a street, whole.)
    const { laid, first, last } = layCourse(made.road, made.pa, made.cells, made.pb);
    for (const road of laid) roads.push({ ...settle(road), name: name() });
    // (Its ends' ports, where a piece of it leaves them.)
    if (first && made.pa.port) made.pa.port.used = true;
    if (last && made.pb.port) made.pb.port.used = true;
  }
  // Ways out of the world (two or three in the classic worlds): from the ports left over (else any), to the nearest edge.
  // (An island's ways out are its causeways: the country roads to them are laid below.)
  const outs = spec.water === 'island' ? 0 : between(rand, spec.exits);
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
  /**
   * A place's land: how much more a road from `p` to edge `side` (`d` m off) would climb than a
   * country road may (m, from the lowest of that edge within `EDGE_WINDOW` m either way of straight
   * out), times `OVER_CLIMB`: the ways out go down the valley or through a pass, not over the
   * massif. 0 in a classic world.
   */
  const overClimb = (p: Pt, side: number, d: number) => {
    if (spec.classic) return 0;
    const along = side % 2 ? p[1] : p[0];
    let low = Infinity;
    for (let t = Math.max(0, along - EDGE_WINDOW); t <= Math.min(side % 2 ? D : W, along + EDGE_WINDOW); t += CELL) {
      const [x, z] = side % 2 ? [side === 1 ? W - 1 : 0, t] : [t, side === 2 ? D - 1 : 0];
      low = Math.min(low, land.at(x, z));
    }
    return Math.max(0, Math.abs(low - land.at(p[0], p[1])) - ROUTE_GRADE * d) * OVER_CLIMB;
  };
  for (let tries = 0; made < outs && tries < outs + 12; tries++) {
    const ports = plans
      .flatMap((plan) => plan.ports)
      .filter((q) => !continued(q))
      .sort((x, y) => Number(x.used) - Number(y.used));
    let best: { q: (typeof ports)[number]; side: number; d: number } | null = null;
    for (const q of ports) {
      for (const [side, d] of [q.p[1], W - q.p[0], D - q.p[1], q.p[0]].entries()) {
        if (!dryEdge[side] || noWay.get(q)?.has(side)) continue;
        // (Not across the river to its far edge.)
        if (river && side % 2 === 0 && bank(q.p) !== (side === 0 ? -1 : 1)) continue;
        const facing = FACING[side];
        const score = d - (q.dir[0] * facing[0] + q.dir[1] * facing[1]) * 80 + (q.used ? 400 : 0) + (usedEdges.has(side) ? 500 : 0) + overClimb(q.p, side, d);
        if (!best || score < best.d) best = { q, side, d: score };
      }
    }
    if (!best) break;
    const { q, side } = best;
    const toward: Pt = [q.p[0] + FACING[side][0] * 400, q.p[1] + FACING[side][1] * 400];
    // (A T off a street facing the edge, if the port doesn't.)
    const tee = q.dir[0] * FACING[side][0] + q.dir[1] * FACING[side][1] < Math.cos(SIDE_FACING) ? sideT(q.area, toward)[0] : undefined;
    const from: Way = tee ?? via(q, toward);
    const onEdge = (i: number) => [Math.floor(i / gw) === 0, i % gw === gw - 1, Math.floor(i / gw) === gd - 1, i % gw === 0][side];
    const ends = from.port ? [from.port.p, from.p] : [];
    const found = route(roadCost(ends), relaxedCost(ends), [from], (i, h) => (onEdge(i) && (h === undefined || along(h, FACING[side]) > 0.9) ? 0 : -1), (i) => [Math.floor(i / gw), gw - 1 - (i % gw), gd - 1 - Math.floor(i / gw), i % gw][side], FACING[side]);
    if (!found) {
      // (Another port or edge, then.)
      noWay.set(q, (noWay.get(q) ?? new Set()).add(side));
      continue;
    }
    // (In pieces where it runs along a road laid already.)
    const name = () => uniqueName(`${edgeName[side]} Road`, `${edgeName[side]} Lane`);
    const { laid, first } = layCourse(toEdge(from, found.cells, side, ''), from, found.cells, null, side);
    for (const road of laid) roads.push({ ...settle(road), name: name() });
    if (from.port && first) q.used = true;
    usedEdges.add(side);
    made++;
  }
  // An island's causeways: a country road to each from the nearest port (one nobody took first).
  for (const c of causeways) {
    const way: Way = { p: c.p, dir: c.dir };
    const goalCell = outCell(way);
    const comesIn = into(way);
    const ports = plans
      .flatMap((plan) => plan.ports)
      .filter((q) => !continued(q))
      .sort((x, y) => Number(x.used) - Number(y.used) || dist(x.p, c.p) - dist(y.p, c.p));
    let laid = false;
    for (const q of ports.slice(0, 4)) {
      const from = via(q, c.p);
      const ends = [q.p, from.p, c.p];
      const l = dist(from.p, c.p) || 1;
      const found = route(roadCost(ends), relaxedCost(ends), [from], (i, h, loose) => (i === goalCell && comesIn(h, loose) ? 0 : -1), (i) => dist(land.centre(i), c.p) / CELL, [(c.p[0] - from.p[0]) / l, (c.p[1] - from.p[1]) / l]);
      if (!found) continue;
      const name = () => uniqueName(`${plans[q.area].name} Road`, `${plans[q.area].name} Lane`);
      const { laid: pieces, first } = layCourse(countryRoad(from, found.cells, way, ['join', 'join'], ''), from, found.cells, way);
      for (const road of pieces) roads.push({ ...settle(road), name: name() });
      if (first && from.port) q.used = true;
      laid = true;
      made++;
      break;
    }
    if (!laid) failed.push(`no country road to the ${roads[c.road].name}`);
  }
  if (made < spec.exits[0]) failed.push(`${made} ways out of the world (${WORDS[spec.exits[0]] ?? spec.exits[0]} or more wanted)`);
  // Ports nobody took: a country road on from each, to the nearest country road, another area's
  // port nobody took or the edge of the world, whichever is nearest (a dead end only if none is).
  const plain = (p: Pt) => clearOf(plans, p, 30);
  const edgeOf = (i: number) => [Math.floor(i / gw) === 0, i % gw === gw - 1, Math.floor(i / gw) === gd - 1, i % gw === 0].indexOf(true);
  /**
   * The country road on from port `q` (of area `k`; -1: a bridge's end, no area's), as it would be
   * laid: to the nearest country road (not its own), another area's port nobody took or the edge of
   * the world; if the road there turns too sharply or winds round, the nearest somewhere else. Its
   * course, where it ends, and the way it leaves by. Null if there is none.
   */
  /**
   * Whether a place's country road along `pts` would have to stand off its land by more than
   * `STEEP_OFF` m to keep within `COUNTRY_GRADE` (the land's heights along it, the least the most a
   * road within its grade must leave them by: half the gap between its grade-limited envelopes).
   * Never in a classic world.
   */
  const tooSteep = (pts: readonly Pt[]) => {
    if (spec.classic || pts.length < 2) return false;
    const along = evenly(pts, CELL / 2);
    const ls = lengths(along);
    const up = along.map(([x, z]) => land.at(x, z));
    const down = up.slice();
    for (const [a, b] of [
      [1, 0],
      [-1, along.length - 1],
    ] as const) {
      for (let i = b + a; i >= 0 && i < along.length; i += a) {
        const g = COUNTRY_GRADE * Math.abs(ls[i] - ls[i - a]);
        up[i] = Math.min(up[i], up[i - a] + g);
        down[i] = Math.max(down[i], down[i - a] - g);
      }
    }
    return up.some((u, i) => (down[i] - u) / 2 > STEEP_OFF);
  };
  /** The ports left without a way on because every way on was too steep (`tooSteep`). */
  const steepOn = new Set<Port>();
  const wayOn = (q: Port, k: number) => {
    /** Country roads' cells out in the country, and which way the road runs there. */
    const country = new Map<number, { road: number; t: Pt }>();
    roads.forEach((r, i) => {
      if (r.kind === 'country' && i !== q.road) r.points.forEach((p, j) => plain(p) && country.set(land.cell(p[0], p[1]), { road: i, t: tangent(r.points, j) }));
    });
    const others = plans.flatMap((o, a) => (a === k ? [] : o.ports.filter((x) => !x.used && !continued(x))));
    const portAt = new Map(others.map((x) => [land.cell(x.p[0] + x.dir[0] * CELL, x.p[1] + x.dir[1] * CELL), x]));
    // A country road met square to it (give or take an eighth of a turn), a port head on, the
    // world's edge heading straight out of it; what lies off to the side or behind the port
    // costs more (a road swinging round to it).
    const meets = (i: number, h?: number) => {
      const on = country.get(i);
      if (on) return h === undefined || Math.abs(along(h, on.t)) < 0.75;
      const port = portAt.get(i);
      if (port) return into(port)(h);
      const side = edgeOf(i);
      return side >= 0 && (h === undefined || along(h, FACING[side]) > 0.9);
    };
    const banned = new Set<number>();
    const goal = (i: number, h?: number) => (banned.has(i) || !meets(i, h) ? -1 : inFront(q, land.centre(i), AHEAD) ? 0 : ASIDE);
    // (Into the other areas only by their ports.)
    const cost = (strict: boolean) => {
      const c = (strict ? roadCost : relaxedCost)([q.p]);
      return (i: number) => (areaBlock[i] && others.some((x) => dist(land.centre(i), x.p) < 26) ? 1 : c(i));
    };
    const from: Way = { p: q.p, dir: q.dir, port: q };
    // The nearest; if the road there turns too sharply or winds round, the nearest somewhere else.
    let made: { road: RoadPlan; to?: Port; bad: number; cells: number[]; b: Way | null; side: number } | null = null;
    for (let tries = 0; tries < TRIES && (!made || made.bad > 0); tries++) {
      const cells = route(cost(true), cost(false), [from], goal, () => 0)?.cells;
      const end = cells?.[cells.length - 1];
      if (!cells || end === undefined) break;
      const to = portAt.get(end);
      const other = country.get(end)?.road;
      let road: RoadPlan;
      // (Its course and where it ends: a port, a T, out of the world over edge `side`.)
      let [course, b, side] = [cells, null as Way | null, -1];
      if (to) {
        b = { p: to.p, dir: to.dir, port: to };
        road = countryRoad(q, cells, b, ['join', 'join'], '');
      } else if (other !== undefined) {
        // Into the other road square to it, ending on its middle.
        // (Met from the side the way comes from: the last cell but one or two before it.)
        const c = cells.length >= 3 ? land.centre(cells[cells.length - 3]) : q.p;
        const pts = roads[other].points;
        const at = nearest(pts, c[0], c[1]);
        const T = pointAt(pts, at.s).p;
        const n = leftOf(pointAt(pts, at.s).t);
        b = { p: T, dir: [n[0] * at.side, n[1] * at.side] };
        course = cells.slice(0, -1);
        road = countryRoad(q, course, b, ['join', 'join'], '');
      } else {
        side = edgeOf(end);
        road = toEdge(q, cells, side, '');
      }
      const bad = fault(road) + (tooSteep(road.points) ? STEEP_FAULT : 0);
      if (!made || bad < made.bad) made = { road, to, bad, cells: course, b, side };
      // (Not there again.)
      const [ex, ez] = [end % gw, Math.floor(end / gw)];
      for (let dz = -3; dz <= 3; dz++) for (let dx = -3; dx <= 3; dx++) if (ex + dx >= 0 && ez + dz >= 0 && ex + dx < gw && ez + dz < gd) banned.add((ez + dz) * gw + ex + dx);
    }
    // (A place's street end with no way on a country road could climb: it ends there.)
    if (made && made.bad >= STEEP_FAULT) {
      steepOn.add(q);
      return null;
    }
    return made && { made, from };
  };
  /** Lays the country road on from a port (`wayOn`), named for `place`: whether its first piece leaves the port. */
  const layOn = ({ made, from }: NonNullable<ReturnType<typeof wayOn>>, place: string) => {
    // (In pieces where it runs along a road laid already.)
    const name = () => uniqueName(`${place} Road`, `${place} Lane`);
    const { laid, first, last } = layCourse(made.road, from, made.cells, made.b, made.side);
    for (const road of laid) roads.push({ ...settle(road), name: name() });
    if (made.to && last) made.to.used = true;
    return first;
  };
  for (const [k, plan] of plans.entries()) {
    for (const q of plan.ports) {
      if (q.used || continued(q)) continue;
      const on = wayOn(q, k);
      if (!on) {
        // (A place's street running out to the shore, with no way round from there: it ends by the water; or up to
        // land no country road could climb: it ends there.)
        const shore = !spec.classic && [8, 16, 24, 32, 40, 48, 56].some((d) => land.wet(q.p[0] + q.dir[0] * d, q.p[1] + q.dir[1] * d, 4));
        if (!shore && !steepOn.has(q)) failed.push(`no country road on from ${plan.name}`);
        continue;
      }
      if (layOn(on, plan.name)) q.used = true;
    }
  }
  // More bridges over a river (a few villages round the town: one; many: two): where it runs
  // straight, well apart from the other bridges and clear of the areas and the roads, the most
  // useful first (the nearest the roads either side); each laid only if a country road goes on
  // from both its ends.
  if (river && spec.options) {
    let more = { none: 0, few: 1, many: 2 }[spec.options.villages];
    const index = new RoadIndex(roads);
    const heading = (x: number) => Math.atan2(river.z(x + 4) - river.z(x - 4), 8);
    const sites: { a: Pt; b: Pt; score: number }[] = [];
    for (let x = 80; x <= W - 80 && more > 0; x += 8) {
      if (Math.abs(heading(x - 24) - heading(x + 24)) > 0.2) continue;
      const slope = (river.z(x + 1) - river.z(x - 1)) / 2;
      const l = Math.hypot(1, slope);
      const back = river.half(x) / l + BRIDGE_IN;
      const a: Pt = [round2(x + (slope / l) * back), round2(river.z(x) - back / l)];
      const span = across(river, a, BRIDGE_IN + 4);
      if (!span || span.wet[1] - span.wet[0] + 2 * DECK_PAST > BRIDGE_SPAN) continue;
      const b = span.far;
      const pts = evenly([a, b], 2);
      if ([a, b].some((p) => Math.min(p[0], p[1], W - p[0], D - p[1]) < 40 || land.wet(p[0], p[1], 2))) continue;
      if (pts.some((p) => plans.some((plan) => offRect(plan.rect, p) < BRIDGE_CLEAR) || index.nearest(p[0], p[1], 12))) continue;
      const near = (p: Pt) => index.nearest(p[0], p[1], 300)?.dist ?? 300;
      sites.push({ a, b, score: near(a) + near(b) });
    }
    sites.sort((x, y) => x.score - y.score);
    const mid = ([a, b]: readonly Pt[]): Pt => [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2];
    for (const site of sites) {
      if (more <= 0) break;
      if (crossings.some((c) => dist(mid(c), mid([site.a, site.b])) < BRIDGE_APART)) continue;
      const at = roads.length;
      const u: Pt = [(site.b[0] - site.a[0]) / dist(site.a, site.b), (site.b[1] - site.a[1]) / dist(site.a, site.b)];
      const ends: Port[] = [
        { road: at, end: 0, area: -1, p: site.a, dir: [-u[0], -u[1]], used: false },
        { road: at, end: 1, area: -1, p: site.b, dir: u, used: false },
      ];
      const ons = ends.map((q) => wayOn(q, -1));
      if (ons.some((o) => !o)) continue;
      const placeNear = (p: Pt) => plans.filter((plan) => bank(plan.centre) === bank(p)).sort((x, y) => dist(x.centre, p) - dist(y.centre, p))[0] ?? plans[0];
      const named = placeNear(site.a).name;
      roads.push(crossing(site.a, site.b, ['join', 'join'], uniqueName(`${named} Bridge`, `${named} Old Bridge`)));
      crossings.push([site.a, site.b]);
      ons.forEach((o, e) => {
        if (!layOn(o!, placeNear(ends[e].p).name)) failed.push(`no country road on from ${roads[at].name}`);
      });
      more--;
    }
  }
  // Ports still nobody's (no way on from them): their roads end there.
  for (const plan of plans) {
    for (const q of plan.ports) {
      const road = roads[q.road];
      road.ends = q.end === 0 ? [q.used ? 'join' : 'dead', road.ends[1]] : [road.ends[0], q.used ? 'join' : 'dead'];
      if (q.used) continue;
      // (A bridge stays whole: a bridge to nowhere is a fault.)
      if (road.bridge) {
        failed.push(`no country road on from ${road.name}`);
        continue;
      }
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
  if (spec.water === 'ponds') {
    // (A fixed number draws nothing.)
    const ponds = spec.ponds[0] === spec.ponds[1] ? spec.ponds[0] : between(rand, spec.ponds);
    for (let n = 0, tries = 0; n < ponds && tries < 200; tries++) {
      const r = range(rand, 14, 24);
      const c: Pt = [range(rand, 60, W - 60), range(rand, 60, D - 60)];
      if (plans.some((a) => c[0] > a.rect.x - 50 - r && c[0] < a.rect.x + a.rect.w + 50 + r && c[1] > a.rect.z - 50 - r && c[1] < a.rect.z + a.rect.d + 50 + r)) continue;
      if (Math.min(...plans.map((a) => dist(a.centre, c))) > 200) continue;
      // (Clear of the roads.)
      if (roads.some((road) => nearest(road.points, c[0], c[1]).dist < r * 1.4 + road.width / 2 + 6)) continue;
      for (let i = 0; i < gw * gd && !shaped; i++) {
        const [x, z] = land.centre(i);
        if (dist([x, z], c) < r * (0.85 + fbm(x / 20, z / 20, nseed + 8 + n, 2) * 0.5)) land.water[i] = 1;
      }
      n++;
    }
  }

  step?.('roads');
  // ---- water and woods: zones, the mask the lots keep off -------------------------------------
  // (A river's and an island's sea their own outlines: the river smooth, the sea out to the world's edges with the island a hole in it.)
  // (A shaped land's water is its own: its tiles, its outlines.)
  const waterPolys = shaped ? [] : river ? [river.poly] : coast.length ? [islandSea(coast, W, D)] : outlines(land.water, gw, gd, 2);
  const waterMask = shaped ? wetTiles(shaped, W, D) : rasterize(waterPolys, W, D);
  // Woods: the most wooded share of the land by noise, not on the areas, the water or the roads.
  const woodsNoise = new Float32Array(gw * gd);
  for (let i = 0; i < gw * gd; i++) {
    const [x, z] = land.centre(i);
    woodsNoise[i] = fbm(x / 105, z / 105, nseed + 7, 4) + (valley ? Math.min(0.25, land.height[i] / 400) : 0);
  }
  const cut = [...woodsNoise].sort((a, b) => b - a)[Math.floor(spec.woods * gw * gd)] ?? 1;
  // (A shaped land's woods are its own, from `fillFrom`.)
  for (let i = 0; i < gw * gd && !shaped; i++) land.woods[i] = woodsNoise[i] > cut && !land.water[i] ? 1 : 0;

  // ---- the lots: one ground over the world, each area's along its own roads -------------------
  const worldRoads = roads;
  // No road out over the world's edge but by an end out of it (`PORT_IN`): cut to the world, it would come apart.
  for (const r of roads) {
    const off = r.points.map(([x, z]) => x < 0 || z < 0 || x > W || z > D);
    const [first, last] = [off.indexOf(false), off.lastIndexOf(false)];
    if (first < 0 || (first > 0 && r.ends[0] !== 'out') || (last < off.length - 1 && r.ends[1] !== 'out') || off.slice(first, last).includes(true)) failed.push(`${r.name ?? 'a road'} runs out over the edge of the world`);
  }
  const allGreens = plans.flatMap((p) => p.greens);
  const ground = new Ground(W, D, worldRoads, allGreens);
  // (A place's lots and buildings keep a strip of bank between them and the water: a way round behind the gardens.)
  ground.block(spec.classic ? waterMask : grown(waterMask, W, D, SHORE_GAP));
  // The living town's own draws (the centre, the terraces, the backdrop), apart from the layout's:
  // the rest of the world is laid out as it was.
  const dense = random((seed ^ 0x2545f491) >>> 0);
  // The main town's centre: its high street and a square off it, kept clear of lots.
  const centre = chooseCentre(worldRoads, plans[0].roads, plans[0].centre, ground);
  const inCentre = (f: Frontage) => !!centre && inside(centre.poly, f.p[0] + f.n[0] * 10, f.p[1] + f.n[1] * 10);
  const lots = new LotPlacer(content, t, ground, worldRoads, spec.gaps, rand, true);
  const names: (string | undefined)[] = worldRoads.map((r) => r.name);
  const fronts = new Map<number, ReturnType<typeof frontageOf>>();
  const left: { u: Parameters<typeof lots.place>[0][number]; from: number }[] = [];
  // Where each area road meets another (arc lengths along it): no lot's middle within `CORNER` m of
  // one, so no driveway comes out right at a junction (a car turning out of it there swings across the corner).
  const meets = new Map<number, number[]>();
  const areaRoads = new Set(plans.flatMap((plan) => plan.roads));
  for (const r of areaRoads) {
    const ls = lengths(worldRoads[r].points);
    meets.set(r, worldRoads[r].ends.flatMap((e, k) => (e === 'join' ? [k ? ls[ls.length - 1] : 0] : [])));
  }
  // (Where another road's end joins it, or another road runs by it, close enough to touch.)
  const byRoad = new RoadIndex(worldRoads);
  worldRoads.forEach((o, j) => {
    for (const [x, z] of evenly(o.points, 2)) {
      for (const [r, hit] of byRoad.around(x, z, 10)) {
        if (r !== j && areaRoads.has(r) && hit.dist <= (o.width + worldRoads[r].width) / 2 + 2) meets.get(r)!.push(hit.s);
      }
    }
  });
  // (Nor on a tight bend: out of a driveway there a car swings off the road.)
  const offCorners = (f: Frontage) => {
    if (!(meets.get(f.road) ?? []).every((s) => Math.abs(f.s - s) >= CORNER)) return false;
    return straightAround(worldRoads[f.road].points, f.s, CORNER / 2, TIGHT_BEND);
  };
  let reserved: Uint8Array | undefined;
  for (const [a, plan] of plans.entries()) {
    const wanted = wantedLots(content, t, spec, plan.spec, rand, { park: a === 0, avoid: a === 0 ? [] : ['market'] });
    const all = frontageOf(worldRoads, plan.roads, plan.centre, rand);
    fronts.set(a, all.filter(offCorners));
    if (a === 0) {
      // The main town: its public places round its middle, on the high street too; then its
      // terraces off the high street, nearest the middle; then its houses, beyond them.
      const homes = all.filter((f) => !inCentre(f));
      const [places, houses] = [wanted.filter((u) => u.kind !== 'house'), wanted.filter((u) => u.kind === 'house')];
      for (const u of lots.place(places, fronts.get(a)!, plan.greens, plan.centre, a, true)) left.push({ u, from: a });
      reserved = centre ? reserveCentre(centre, ground) : undefined;
      // (Out of room in the town: on along the country roads leaving it, where they run straight.)
      const countryWays = worldRoads.flatMap((r, i) => (r.kind === 'country' && !r.bridge ? [i] : []));
      const lens = new Map(countryWays.map((r) => [r, lengths(worldRoads[r].points)]));
      const out = frontageOf(worldRoads, countryWays, plan.centre, dense).filter((f) => offRect(plan.rect, f.p) < TERRACE_SPILL && straight(worldRoads[f.road].points, f.s, lens.get(f.road)!) && gentleBy(f.p));
      terraceRows(t, lots, [homes, out], spec, dense);
      fronts.set(a, homes.filter(offCorners));
      for (const u of lots.place(houses, fronts.get(a)!, [], plan.centre, a, true)) left.push({ u, from: a });
    } else for (const u of lots.place(wanted, fronts.get(a)!, plan.greens, plan.centre, a, true)) left.push({ u, from: a });
    const streets = streetNames(content, spec.names, rand, plan.roads.length, [...takenNames, ...roadNames], plan.name);
    takenNames.push(...streets);
    plan.roads.forEach((r, k) => (names[r] = streets[k]));
  }
  // Houses with no room in their own area go to the nearest other that has room (homes, not shops).
  // (In a place's world, out along the country roads only where the land by the road is gentle: `PLACE_STEEP`.)
  let country: ReturnType<typeof frontageOf> | undefined;
  for (const { u, from } of left) {
    const others = plans.map((_, a) => a).filter((a) => a !== from && u.kind === 'house').sort((x, y) => dist(plans[x].centre, plans[from].centre) - dist(plans[y].centre, plans[from].centre));
    const placed = others.some((a) => !lots.place([u], fronts.get(a)!, [], plans[a].centre, a, true).length);
    if (placed) continue;
    // Else along a country road, near its own area's end of it.
    country ??= frontageOf(worldRoads, worldRoads.flatMap((r, i) => (r.kind === 'country' && !r.bridge ? [i] : [])), plans[0].centre, rand).filter((f) => straight(worldRoads[f.road].points, f.s, lengths(worldRoads[f.road].points)) && gentleBy(f.p));
    const r = plans[from].rect;
    const off = (p: Pt) => offRect(r, p);
    const near = country.filter((f) => off(f.p) < SPILL).sort((x, y) => dist(x.p, plans[from].centre) - dist(y.p, plans[from].centre));
    if (u.kind === 'house' && !lots.place([u], near, [], plans[from].centre, from, true).length) continue;
    lots.dropped.push({ name: u.name, kind: u.kind, lot: u.lot });
  }
  const farmsteads = alongCountryRoads(content, t, lots, plans, worldRoads, land, spec.farmEvery, rand, spec.classic ? STEEP : PLACE_STEEP);
  lots.number(names);
  const slots = lots.slots;
  const rectOf = (s: PlotSlot) => plotRect(t, s);

  // ---- parking in each area ------------------------------------------------------------------
  const parking: OpenWorldDraft['parking'] = [];
  for (const [a, plan] of plans.entries()) {
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
    const count = between(rand, spec.spotsPerArea) + (a === 0 ? 1 : 0);
    for (let n = 0; n < count; n++) {
      const id = spec.spots[next++ % spec.spots.length];
      const place = outdoor.get(id);
      if (!place) continue;
      const spot = siteFor(place, plan, land, ground, waterMask, t, rand, W, D, spec.classic ? Infinity : SPOT_REACH);
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

  // ---- the living town: the backdrop, and the places the tags name --------------------------
  const { backdrop, places } = layBackdrop({
    content,
    W,
    D,
    roads: worldRoads,
    areas: plans.map((p) => ({ name: p.name, rect: p.rect, centre: p.centre, roads: p.roads })),
    slots,
    rectOf,
    ground,
    centres: centre ? [centre] : [],
    reserved,
    greens: allGreens,
    water: waterMask,
    density: spec.density,
    least: spec.mainLeast,
    civic: spec.civic,
    rand: dense,
  });

  // ---- zones -------------------------------------------------------------------------------
  const clearAround = (mask: Uint8Array, r: Rect, grow: number) => {
    for (let z = Math.floor((r.z - grow) / CELL); z <= Math.floor((r.z + r.d + grow) / CELL); z++) for (let x = Math.floor((r.x - grow) / CELL); x <= Math.floor((r.x + r.w + grow) / CELL); x++) if (x >= 0 && z >= 0 && x < gw && z < gd) mask[z * gw + x] = 0;
  };
  const woods = land.woods.slice();
  // (Wooded: round a hamlet the woods come up to its roads and lots.)
  for (const [a, plan] of plans.entries()) if (!spec.wildBoxes || (a === 0 && spec.wildBoxes === 'hamlets')) clearAround(woods, plan.rect, 10);
  for (const s of slots) clearAround(woods, rectOf(s), 4);
  for (const r of orchards) clearAround(woods, r, 4);
  const footprints = backdrop.map((b) => boundsOf(b.poly));
  for (const r of footprints) clearAround(woods, r, 3);
  for (const road of worldRoads) for (let i = 0; i < road.points.length; i += 3) clearAround(woods, { x: road.points[i][0], z: road.points[i][1], w: 0, d: 0 }, road.width / 2 + 3);
  const heights = [...land.height].sort((x, y) => x - y);
  const hillCut = heights[Math.floor(heights.length * (valley ? 0.62 : 0.82))];
  const hills = new Uint8Array(gw * gd);
  const meadow = new Uint8Array(gw * gd);
  const fields = new Uint8Array(gw * gd);
  /** Farmland: round each area's box (as far as `spec.fields` past it) and each farmstead (a little less from its middle). */
  const reach = spec.fields ?? 0;
  const farmMiddles = farmsteads.map((s): Pt => {
    const r = rectOf(s);
    return [r.x + r.w / 2, r.z + r.d / 2];
  });
  const nearFarmland = (p: Pt) =>
    plans.some((a) => offRect(a.rect, p) < reach) || farmMiddles.some((m) => dist(m, p) < reach * 1.1);
  for (let i = 0; i < gw * gd; i++) {
    if (land.water[i]) continue;
    const [x, z] = land.centre(i);
    if (land.height[i] > hillCut) hills[i] = 1;
    if (!woods[i] && !areaBlock[i] && fbm(x / 70, z / 70, nseed + 11, 3) > (spec.meadowCut ?? 0.58)) meadow[i] = 1;
    if (spec.fields && !woods[i] && !areaBlock[i] && !meadow[i] && nearFarmland([x, z]) && fbm(x / 110, z / 110, nseed + 13, 3) > spec.fieldsCut) fields[i] = 1;
  }
  for (const plan of plans) clearAround(hills, plan.rect, 0);
  for (const road of worldRoads) {
    for (let i = 0; i < road.points.length; i += 3) {
      const at = { x: road.points[i][0], z: road.points[i][1], w: 0, d: 0 };
      clearAround(fields, at, road.width / 2 + 2);
      clearAround(meadow, at, road.width / 2 + 1);
    }
  }
  for (const r of [...slots.map(rectOf), ...orchards, ...footprints]) {
    clearAround(fields, r, 2);
    clearAround(meadow, r, 2);
  }
  let zones: WorldZone[] = [
    ...waterPolys.map((poly): WorldZone => ({ kind: 'water', poly })),
    ...outlines(hills, gw, gd, 12).map((poly): WorldZone => ({ kind: 'hill', poly })),
    ...outlines(fields, gw, gd, 10).map((poly): WorldZone => ({ kind: 'fields', poly })),
    ...outlines(meadow, gw, gd, 6).map((poly): WorldZone => ({ kind: 'meadow', poly })),
    ...outlines(woods, gw, gd, 8).map((poly): WorldZone => ({ kind: 'woods', poly })),
    ...orchards.map((r): WorldZone => ({ kind: 'orchard', poly: [[r.x, r.z], [r.x + r.w, r.z], [r.x + r.w, r.z + r.d], [r.x, r.z + r.d]] })),
  ];

  // A shaped land: graded and cleared under what is built, its water and cover the zones (the hills and orchards still the layout's).
  let laid: LandField | undefined;
  if (shaped) {
    laid = settleTown(shaped, { width: W, depth: D, plots: [...slots.map(rectOf), ...orchards], roads: worldRoads, backdrop });
    const own = landZones(laid);
    zones = (['water', 'hill', 'fields', 'meadow', 'woods', 'orchard'] as const).flatMap((kind) => (kind === 'hill' || kind === 'orchard' ? zones : own).filter((z) => z.kind === kind));
  }

  const areas: TownArea[] = plans.map((p) => ({ name: p.name, kind: 'hamlet', x: p.rect.x, z: p.rect.z, w: p.rect.w, d: p.rect.d }));
  // (The landscape round the world: its sea or lake is in the zones now. A place's from its own
  // stream; its mountains as high as the massif.)
  const { water: _water, ...landscape } = spec.landscape(spec.classic ? rand : random((seed ^ LOOK) >>> 0), W, D, waterSide);
  if (!spec.classic && massif) landscape.mountains = Math.round(massif);
  const relief = spec.classic ? undefined : reliefOf(land, [...slots.map(rectOf), ...footprints]);
  const signed = spec.classic || shaped ? undefined : heightsOf(land, [...slots.map(rectOf), ...footprints], worldRoads);
  // The roads cut to the world; each bridge's and causeway's deck over its water (and a little
  // past it), and the water the sim walks round without the decks.
  const filed = cropRoads(worldRoads, 0, 0, W, D);
  let walkWater = waterMask;
  if (filed.some((r) => r.bridge)) {
    walkWater = waterMask.slice();
    for (const r of filed) if (r.bridge) r.bridge = deckOf(r, waterMask, walkWater, W, D);
  }
  step?.('homes');
  return {
    name: name ?? rolledName,
    width: W,
    depth: D,
    slots,
    roads: filed,
    greens: allGreens.map((g) => g.map(([x, z]): Pt => [round2(x), round2(z)])),
    households: [],
    seed,
    world: { kind: spec.kind, size: 'big', tagline, landscape, ...(spec.options ? { options: spec.options } : {}) },
    diagnostics: { dropped: lots.dropped, ...(failed.length ? { roads: failed } : {}), ...(reachable && missing > 0 ? { villages: { left: missing, homes: missing * spec.hamlet.lots[0] } } : {}) },
    open: {
      areas,
      zones,
      water: rle(walkWater),
      pads,
      parking,
      backdrop,
      places,
      ...(centre ? { centre: [{ area: centre.area, poly: centre.poly }], squares: centre.square ? [centre.square] : [] } : {}),
      ...(relief ? { relief } : {}),
      ...(signed ? { heights: signed } : {}),
      ...(laid ? { land: laid } : {}),
    },
  };
}

/** A shaped land's 8 m cells (`generateFromSpec`'s `shaped`): each the mean height of its samples, water if any of them is wet, wooded if most of its cover is woods. */
function fillFrom(land: Land, field: LandField): void {
  const k = CELL / LAND_CELL;
  const c = CELL / COVER_CELL;
  const cw = coverSpan(field.cols);
  const cd = coverSpan(field.rows);
  for (let cz = 0; cz < land.gd; cz++) {
    for (let cx = 0; cx < land.gw; cx++) {
      let [sum, n, wet] = [0, 0, 0];
      for (let j = cz * k; j < Math.min(field.rows, cz * k + k); j++) {
        for (let i = cx * k; i < Math.min(field.cols, cx * k + k); i++) {
          const s = j * field.cols + i;
          sum += field.h[s];
          n++;
          if (field.wl[s] !== NONE && field.wl[s] > field.h[s]) wet = 1;
        }
      }
      let [woods, cells] = [0, 0];
      for (let r = cz * c; r < Math.min(cd, cz * c + c); r++) {
        for (let q = cx * c; q < Math.min(cw, cx * c + c); q++) {
          cells++;
          if (field.cover[r * cw + q] === COVER.woods) woods++;
        }
      }
      const i = cz * land.gw + cx;
      land.height[i] = n ? sum / n / 100 : 0;
      land.water[i] = wet;
      land.woods[i] = woods * 2 > cells && !wet ? 1 : 0;
    }
  }
}

/**
 * A shaped land's largest piece of dry land (8 m cells joined side to side), the areas' to go on:
 * whether a box and the land `by` m round it stand all on it (`holds`), and the place nearest a
 * box's (8 m steps) where it does, within bounds (`nearest`: [x0, z0, x1, z1] of its corner).
 */
function dryRegion(land: Land): { holds: (r: Rect, by: number) => boolean; nearest: (r: Rect, bounds: [number, number, number, number]) => Pt | null } {
  const { gw, gd } = land;
  const piece = new Int32Array(gw * gd).fill(-1);
  const sizes: number[] = [];
  for (let start = 0; start < gw * gd; start++) {
    if (land.water[start] || piece[start] >= 0) continue;
    const id = sizes.length;
    const stack = [start];
    piece[start] = id;
    let n = 0;
    while (stack.length) {
      const i = stack.pop()!;
      n++;
      const [x, z] = [i % gw, Math.floor(i / gw)];
      for (const [nx, nz] of [[x - 1, z], [x + 1, z], [x, z - 1], [x, z + 1]] as const) {
        const j = nz * gw + nx;
        if (nx < 0 || nz < 0 || nx >= gw || nz >= gd || land.water[j] || piece[j] >= 0) continue;
        piece[j] = id;
        stack.push(j);
      }
    }
    sizes.push(n);
  }
  const main = sizes.indexOf(Math.max(-1, ...sizes));
  // Cells off it, summed from the corner.
  const sat = new Int32Array((gw + 1) * (gd + 1));
  for (let z = 0; z < gd; z++) {
    let row = 0;
    for (let x = 0; x < gw; x++) {
      row += piece[z * gw + x] === main ? 0 : 1;
      sat[(z + 1) * (gw + 1) + x + 1] = sat[z * (gw + 1) + x + 1] + row;
    }
  }
  const holds = (r: Rect, by: number) => {
    const [x0, z0] = [Math.floor((r.x - by) / CELL), Math.floor((r.z - by) / CELL)];
    const [x1, z1] = [Math.floor((r.x + r.w + by) / CELL) + 1, Math.floor((r.z + r.d + by) / CELL) + 1];
    if (main < 0 || x0 < 0 || z0 < 0 || x1 > gw || z1 > gd) return false;
    return sat[z1 * (gw + 1) + x1] - sat[z0 * (gw + 1) + x1] - sat[z1 * (gw + 1) + x0] + sat[z0 * (gw + 1) + x0] === 0;
  };
  const nearest = (r: Rect, [bx0, bz0, bx1, bz1]: [number, number, number, number]): Pt | null => {
    let best: Pt | null = null;
    let far = Infinity;
    for (let z = bz0; z <= bz1; z += CELL) {
      for (let x = bx0; x <= bx1; x += CELL) {
        const d = Math.hypot(x - r.x, z - r.z);
        if (d >= far || !holds({ x, z, w: r.w, d: r.d }, CELL)) continue;
        [best, far] = [[Math.round(x), Math.round(z)], d];
      }
    }
    return best;
  };
  return { holds, nearest };
}

/** The ground round what is built rises back to a shaped land's own height over this far (m) past the flat apron. */
const GRADE_RISE = 40;
/** A bank kept this far over the water beside it (cm) where the grading lowers the land. */
const GRADE_BANK = 25;

/**
 * A shaped land with a town laid on it (`generateFromSpec`'s `shaped`): a copy, flat at 0 under
 * what is built and rising back to its own height round it (as the renderer draws a world's
 * land round its towns), but never below a bank over the water beside it; and its cover open
 * within 2 m of anything built. Marked `edited`. A land whose town follows its slopes (`seeded`
 * `'follow'`) keeps its heights: the town is graded into it afterwards (`landFollow.ts` `withLand`).
 */
function settleTown(field: LandField, built: BuiltLand): LandField {
  const { cols, rows } = field;
  const out: LandField = { ...field, h: field.h.slice(), wl: field.wl.slice(), cover: field.cover.slice(), edited: true };
  const guard = new Protection(cols, rows, built);
  const wet = (k: number) => out.wl[k] !== NONE && out.wl[k] > out.h[k];
  for (let k = 0; k < cols * rows && field.seeded !== 'follow'; k++) {
    if (wet(k)) continue;
    const d = guard.dist[k];
    if (d >= APRON + GRADE_RISE) continue;
    const t = Math.min(1, Math.max(0, (d - APRON) / GRADE_RISE));
    out.h[k] = Math.round(field.h[k] * t * t * (3 - 2 * t));
  }
  // (Beside water, no lower than a bank over it, nor than the land was.)
  for (let j = 0; j < rows && field.seeded !== 'follow'; j++) {
    for (let i = 0; i < cols; i++) {
      const k = j * cols + i;
      if (wet(k) || out.h[k] === field.h[k]) continue;
      let top = -Infinity;
      for (const [ni, nj] of [[i - 1, j], [i + 1, j], [i, j - 1], [i, j + 1]] as const) if (ni >= 0 && nj >= 0 && ni < cols && nj < rows && wet(nj * cols + ni)) top = Math.max(top, out.wl[nj * cols + ni] + GRADE_BANK);
      if (top > out.h[k]) out.h[k] = Math.min(field.h[k], top);
    }
  }
  const cw = coverSpan(cols);
  const c = COVER_CELL / LAND_CELL;
  for (let r = 0; r < coverSpan(rows); r++) for (let q = 0; q < cw; q++) if (guard.dist[Math.min(rows - 1, r * c + 1) * cols + Math.min(cols - 1, q * c + 1)] <= 2) out.cover[r * cw + q] = COVER.open;
  return out;
}

/** A river across a world (`riverAcross`). */
interface River {
  /** Its middle line, from west to east, and its half-width there (m, along z: wider where it runs slantwise). */
  z: (x: number) => number;
  half: (x: number) => number;
  /** Its outline, a water zone (on past the world's edges). */
  poly: Pt[];
}

/**
 * A river across a `W` × `D` world from west to east: meandering 70 to 110 m either way, but a
 * fifth as much along `reach` (the main town's stretch of it, from x to x) and a little past it,
 * 14 to 22 m wide, its middle near the world's (within `middle`, from z to z, where the town has room).
 */
function riverAcross(rand: Rand, W: number, D: number, reach: [number, number], nseed: number, middle: [number, number]): River {
  const c = Math.max(middle[0], Math.min(middle[1], D / 2 + range(rand, -0.04, 0.04) * D));
  const tilt = range(rand, -0.05, 0.05);
  const amp = range(rand, ...MEANDER);
  const k = rand() * 50;
  const calm = (x: number) => {
    const t = Math.min(1, Math.max(reach[0] - 60 - x, x - reach[1] - 60, 0) / 120);
    return 0.2 + 0.8 * t * t * (3 - 2 * t);
  };
  const z = (x: number) => c + tilt * (x - W / 2) + (fbm(x / 260, k, nseed + 21, 3) - 0.5) * 2.5 * amp * calm(x);
  const width = (x: number) => RIVER_WIDTH[0] + (RIVER_WIDTH[1] - RIVER_WIDTH[0]) * Math.min(1, Math.max(0, fbm(x / 120, k + 3, nseed + 22, 2) * 1.6 - 0.3));
  const half = (x: number) => (width(x) / 2) * Math.hypot(1, (z(x + 1) - z(x - 1)) / 2);
  const xs: number[] = [];
  for (let x = -60; x <= W + 60; x += 4) xs.push(x);
  const poly = [...xs.map((x): Pt => [x, round2(z(x) - half(x))]), ...[...xs].reverse().map((x): Pt => [x, round2(z(x) + half(x))])];
  return { z, half, poly };
}

/**
 * Across `river` from `p`, square to its course: how far along that way the water begins and
 * ends (m), the point `BRIDGE_IN` m past its far bank, and the way on from there (away from the
 * water). Null if the river is more than `max` m off.
 */
function across(river: River, p: Pt, max: number): { wet: [number, number]; far: Pt; dir: Pt } | null {
  const toward = river.z(p[0]) > p[1] ? 1 : -1;
  const t: Pt = [1, (river.z(p[0] + 4) - river.z(p[0] - 4)) / 8];
  const l = Math.hypot(...t);
  const n: Pt = [(-t[1] / l) * toward, (t[0] / l) * toward];
  let [first, last] = [-1, -1];
  for (let d = 0; d <= max + 80; d += 0.5) {
    const q: Pt = [p[0] + n[0] * d, p[1] + n[1] * d];
    if (Math.abs(q[1] - river.z(q[0])) < river.half(q[0])) {
      if (first < 0) first = d;
      last = d;
    } else if (first >= 0) break;
    if (first < 0 && d > max) return null;
  }
  if (first < 0) return null;
  const far: Pt = [round2(p[0] + n[0] * (last + BRIDGE_IN)), round2(p[1] + n[1] * (last + BRIDGE_IN))];
  return { wet: [first, last], far, dir: n };
}

/**
 * An island's sea: the world's square with the island's `coast` a hole in it, joined to it by a
 * seam (filled even-odd, as `inside` does), touching every edge of the world.
 */
function islandSea(coast: readonly Pt[], W: number, D: number): Pt[] {
  const west = coast.reduce((k, p, i) => (p[0] < coast[k][0] ? i : k), 0);
  const seam: Pt = [0, coast[west][1]];
  return [seam, [0, 0], [W, 0], [W, D], [0, D], seam, ...coast.slice(west), ...coast.slice(0, west + 1)];
}

/**
 * A bridge's or a causeway's deck (`RoadPlan.bridge`): its arc lengths from the first point over
 * `water` (1 m tiles) less `DECK_PAST` to the last plus as much. The tiles of its deck
 * (`deckTiles`) are cleared in `walk` (the water the sim's walkers keep off), so a walker on its
 * verge walks dry.
 */
function deckOf(road: RoadPlan, water: Uint8Array, walk: Uint8Array, W: number, D: number): [number, number] {
  const ls = lengths(road.points);
  const L = ls[ls.length - 1];
  const wetAt = ([x, z]: Pt) => water[Math.min(D - 1, Math.max(0, Math.floor(z))) * W + Math.min(W - 1, Math.max(0, Math.floor(x)))] === 1;
  const first = road.points.findIndex(wetAt);
  if (first < 0) return [0, 0];
  let last = first;
  for (let i = first; i < road.points.length; i++) if (wetAt(road.points[i])) last = i;
  const span: [number, number] = [round2(Math.max(0, ls[first] - DECK_PAST)), round2(Math.min(L, ls[last] + DECK_PAST))];
  deckTiles(road, span, W, D, (tile) => (walk[tile] = 0));
  return span;
}

/**
 * The 1 m tiles (indices into a `W` × `D` world) of a bridge's or a causeway's deck over `span`
 * (arc lengths along `road`, as `RoadPlan.bridge`): those within half its width and `DECK_PAST`
 * of its middle line there, each passed to `each`. The water under them is walked dry.
 */
export function deckTiles(road: RoadPlan, span: readonly [number, number], W: number, D: number, each: (tile: number) => void): void {
  const ls = lengths(road.points);
  const [a, b] = [pointAt(road.points, span[0], ls).p, pointAt(road.points, span[1], ls).p];
  const R = road.width / 2 + DECK_PAST;
  const [dx, dz] = [b[0] - a[0], b[1] - a[1]];
  const len2 = dx * dx + dz * dz || 1;
  for (let z = Math.max(0, Math.floor(Math.min(a[1], b[1]) - R)); z < Math.min(D, Math.ceil(Math.max(a[1], b[1]) + R)); z++) {
    for (let x = Math.max(0, Math.floor(Math.min(a[0], b[0]) - R)); x < Math.min(W, Math.ceil(Math.max(a[0], b[0]) + R)); x++) {
      const t = Math.max(0, Math.min(1, ((x + 0.5 - a[0]) * dx + (z + 0.5 - a[1]) * dz) / len2));
      if (Math.hypot(x + 0.5 - a[0] - dx * t, z + 0.5 - a[1] - dz * t) <= R) each(z * W + x);
    }
  }
}

/** The relief's cells (m). */
const RELIEF = 16;

/**
 * How high the land stands over the level the towns are built at (`WorldMeta.relief`): the land's
 * height averaged over 16 m cells, less the middle height (the median) of the cells anything
 * stands on (`built`: the lots and the backdrop), in decimetres, never below 0, and 0 where the
 * cell is mostly water. The renderer draws its hills from it.
 */
function reliefOf(land: Land, built: readonly Rect[]): NonNullable<WorldMeta['relief']> {
  const k = RELIEF / CELL;
  const [cols, rows] = [Math.ceil(land.W / RELIEF), Math.ceil(land.D / RELIEF)];
  const mean = new Float32Array(cols * rows);
  const wet = new Uint8Array(cols * rows);
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      let [sum, n, water] = [0, 0, 0];
      for (let dz = 0; dz < k; dz++) {
        for (let dx = 0; dx < k; dx++) {
          const [x, z] = [c * k + dx, r * k + dz];
          if (x >= land.gw || z >= land.gd) continue;
          sum += land.height[z * land.gw + x];
          water += land.water[z * land.gw + x];
          n++;
        }
      }
      mean[r * cols + c] = n ? sum / n : 0;
      wet[r * cols + c] = water * 2 >= n && n ? 1 : 0;
    }
  }
  const on = new Uint8Array(cols * rows);
  for (const b of built) {
    for (let r = Math.max(0, Math.floor(b.z / RELIEF)); r <= Math.min(rows - 1, Math.floor((b.z + b.d) / RELIEF)); r++) {
      for (let c = Math.max(0, Math.floor(b.x / RELIEF)); c <= Math.min(cols - 1, Math.floor((b.x + b.w) / RELIEF)); c++) on[r * cols + c] = 1;
    }
  }
  const levels = [...mean.filter((_, i) => on[i] && !wet[i])].sort((a, b) => a - b);
  const level = levels[Math.floor(levels.length / 2)] ?? 0;
  return { cell: RELIEF, cols, rows, h: Array.from(mean, (h, i) => (wet[i] ? 0 : Math.max(0, Math.round((h - level) * 10)))) };
}

/** The share of the built cells lower than the level the towns are built at (`heightsOf`). */
const BUILT_LOW = 0.05;

/**
 * The land's signed heights (`OpenWorldDraft.heights`): each 8 m cell's height less the level the
 * towns are built at, the height `BUILT_LOW` of the cells anything is built on stand lower than
 * (the lots, the backdrop and the roads but the trails; not water). Mostly the built land is a
 * little over 0, and the sea (`land.ts` `SEA_CM`) a little under it, as the renderer has always
 * drawn them.
 */
function heightsOf(land: Land, built: readonly Rect[], roads: readonly RoadPlan[]): NonNullable<OpenWorldDraft['heights']> {
  const { gw, gd } = land;
  const on = new Uint8Array(gw * gd);
  for (const b of built) {
    for (let z = Math.max(0, Math.floor(b.z / CELL)); z <= Math.min(gd - 1, Math.floor((b.z + b.d) / CELL)); z++) {
      for (let x = Math.max(0, Math.floor(b.x / CELL)); x <= Math.min(gw - 1, Math.floor((b.x + b.w) / CELL)); x++) on[z * gw + x] = 1;
    }
  }
  for (const r of roads) if (r.kind !== 'trail') for (const [x, z] of r.points) if (x >= 0 && z >= 0 && x < land.W && z < land.D) on[land.cell(x, z)] = 1;
  const levels: number[] = [];
  for (let i = 0; i < gw * gd; i++) if (on[i] && !land.water[i]) levels.push(land.height[i]);
  levels.sort((a, b) => a - b);
  const level = levels[Math.floor(levels.length * BUILT_LOW)] ?? 0;
  return { cell: CELL, cols: gw, rows: gd, h: Float32Array.from(land.height, (h) => h - level) };
}

/**
 * The main town's terraces (`WorldSpec.terraces` lots, in rows of `WorldSpec.rows`: two, a
 * semi-detached pair), each row along the first stretch of `frontages` it fits on (the town's
 * nearest its middle first, then the next list's: `LotPlacer.placeRow`); a row that fits nowhere is
 * tried shorter, down to a pair. Their houses are drawn from `rand` (the world's own draws are left as
 * they were). The rows placed.
 */
function terraceRows(t: Templates, lots: LotPlacer, frontages: readonly (readonly Frontage[])[], spec: Pick<WorldSpec, 'terraces' | 'rows'>, rand: Rand): PlotSlot[][] {
  const out: PlotSlot[][] = [];
  let left = between(rand, spec.terraces);
  // (The longest row that went down last: one longer fits nowhere.)
  let longest = spec.rows[1];
  // (Nowhere for a pair: no more rows.)
  for (let tries = 0; left >= 2 && tries < 400; tries++) {
    let n = Math.min(left, longest, between(rand, spec.rows));
    if (left - n === 1) n = left === 3 && spec.rows[1] >= 3 ? 3 : n - 1;
    const units = Array.from({ length: Math.max(2, n) }, () => rowHouse(t, rand))
      .filter((h) => !!h)
      .map((h) => ({ kind: 'house' as const, name: '', lot: 'terrace' as const, area: 0, ...h }));
    // (No terraced house to be had: none generated, the content has no row class.)
    if (units.length < 2) break;
    let placed: PlotSlot[] = [];
    for (let k = units.length; k >= 2 && !placed.length; k--) {
      for (const frontage of frontages) if (!placed.length) placed = lots.placeRow(units.slice(0, k), frontage, 0);
    }
    if (!placed.length) break;
    if (placed.length < units.length) longest = placed.length;
    out.push(placed);
    left -= placed.length;
  }
  return out;
}

/**
 * A terraced house and its lot (`houseFor` on a terrace lot): one generated (a draw or two in a
 * hundred lays none out; `houseFor` would put a fixed layout, too wide for the lot, in its place).
 * Null: none in 40 draws.
 */
function rowHouse(t: Templates, rand: Rand): ReturnType<typeof houseFor> | null {
  for (let k = 0; k < 40; k++) {
    const h = houseFor(t, 'terrace', 'snug', rand);
    if (h.template.startsWith('gen:')) return h;
  }
  return null;
}

/** A country road runs nearly straight round arc length `s` (within `BEND` m either way): a car turns in off it without swinging off the road. */
const straight = (pts: readonly Pt[], s: number, ls: number[]) => straightAround(pts, s, BEND, BEND_MOST, ls);

/**
 * Homes along the country roads, on purpose: 2 to 4 cottages where each road leaves an area and
 * where it meets another out in the country, then a farmstead (a farm lot: its fields, its barn
 * and yard) every `every` m of road (150 to 200 in the classic worlds), each clear of the areas,
 * the water and steep land, and of the area it is nearest. The farmsteads placed.
 */
function alongCountryRoads(content: Content, t: Templates, lots: LotPlacer, plans: readonly Plan[], roads: readonly RoadPlan[], land: Land, every: [number, number], rand: Rand, steepest = STEEP): PlotSlot[] {
  const country = roads.flatMap((r, i) => (r.kind === 'country' && !r.bridge ? [i] : []));
  const byRoad = new Map<number, Frontage[]>();
  const lens = new Map(country.map((r) => [r, lengths(roads[r].points)]));
  for (const f of frontageOf(roads, country, plans[0].centre, rand)) {
    if (!straight(roads[f.road].points, f.s, lens.get(f.road)!)) continue;
    const list = byRoad.get(f.road);
    if (list) list.push(f);
    else byRoad.set(f.road, [f]);
  }
  const outside = (p: Pt, by: number) => clearOf(plans, p, by);
  /** Well inside the world (a road out leaves it there). */
  const within = (p: Pt) => Math.min(p[0], p[1], land.W - p[0], land.D - p[1]) > 30;
  const homeArea = (p: Pt) => plans.reduce((k, a, i) => (dist(a.centre, p) < dist(plans[k].centre, p) ? i : k), 0);
  /** Dry and not steep within `r` m of `p`. */
  const buildable = (p: Pt, r: number) => {
    if (land.wet(p[0], p[1], r)) return false;
    for (let dz = -r; dz <= r; dz += CELL * 2) for (let dx = -r; dx <= r; dx += CELL * 2) if (land.slope(land.cell(p[0] + dx, p[1] + dz)) > steepest) return false;
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
      s += range(rand, ...every);
    }
  }
  return farms;
}

/**
 * Where an outdoor place of area `plan` goes: on its kind of land (by the water, its back to it;
 * up a hill; in the woods; in a meadow), 50 to 140 m from the area's middle, on land nothing else
 * takes, no further than `reach` from the area's box. Its rectangle and which way it faces
 * (towards the area: its trail comes from there).
 */
function siteFor(place: PublicTemplate, plan: Plan, land: Land, ground: Ground, water: Uint8Array, t: Templates, rand: Rand, W: number, D: number, reach = Infinity): { x: number; z: number; facing: Facing; w: number; d: number } | null {
  const frame = place.lot ?? t.plot;
  let best: { at: { x: number; z: number; facing: Facing; w: number; d: number }; score: number } | null = null;
  const wetAt = (x: number, z: number) => x >= 0 && z >= 0 && x < W && z < D && water[Math.floor(z) * W + Math.floor(x)] === 1;
  for (let k = 0; k < 260; k++) {
    const angle = rand() * Math.PI * 2;
    const r = range(rand, 50, 140) + Math.max(plan.rect.w, plan.rect.d) * 0.3;
    const c: Pt = [plan.centre[0] + Math.cos(angle) * r, plan.centre[1] + Math.sin(angle) * r];
    if (offRect(plan.rect, c) > reach) continue;
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
  const points = clear(smooth) ? smooth : evenly(raw, 1);
  return { points: points.map(([x, z]): Pt => [round2(x), round2(z)]), width: TRAIL_WIDTH, ends: ['join', 'dead'], kind: 'trail' };
}

/** The middles of a big world's areas, and a centroid helper for the menus. */
export function areaCentre(a: TownArea): Pt {
  return centroid([[a.x, a.z], [a.x + a.w, a.z + a.d]]);
}

/** Where the roads lead out of a big world: their `out` ends (on its edge). */
export function worldExits(roads: readonly RoadPlan[]): Pt[] {
  return roadEnds(roads.filter((r) => r.kind !== 'trail')).map(({ x, z }): Pt => [x, z]);
}
