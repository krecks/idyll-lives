/**
 * The country roads' router, moved out of `macro.ts` `generateFromSpec` so a village placed by
 * hand is joined to the roads the way a generated one is. It works over the land's 8 m cells
 * (`Cells`: their heights, water and woods) and the areas' boxes, and reads the world's roads as
 * they are laid (the list it is given is the one the caller adds to, read afresh on every call).
 *
 * A country road leaves an area by a street's end straight on or by a T square off a street, and
 * is a path over the grid that turns as a road does (`steer`: wide bends, never back on itself),
 * smoothed into a curve with no hairpins (`COUNTRY_TURN`); one whose path follows a road laid
 * already shares it, joining it and leaving it in a T, rather than running beside it (`shared`).
 * It draws nothing at random: the same cells, areas and roads give the same road.
 */

import type { RoadPlan } from '../core/protocol';
import { simplify } from './landMasks';
import { curve, evenly, leftOf, lengths, nearest, pointAt, RoadIndex, sharpestTurn, tangent, type Pt } from './roads';
import type { Rect } from './town';
import type { WorldSpec } from './world';

/** The land's cells (m). */
export const CELL = 8;
/**
 * Every road in a big world, the country roads and every area's streets and lanes, is this wide at
 * least (m): two cars pass on it in their lanes (the sim's `cars::lane_on`), so all of them are laid
 * this wide before any lot is placed.
 */
export const ROAD_WIDTH = 7;
/** Country roads' width (m). */
export const COUNTRY_WIDTH = ROAD_WIDTH;
/** A T off a street costs this much more than a port to leave by (m; `macro.ts` `sideT`). */
export const SIDE_COST = 30;
/** Leaving off a country road that leaves a port already (`via`) costs this much more (m): a port nobody has taken yet goes first. */
const VIA_COST = 40;
/** Leaving from a port another country road leaves already, both from it together, costs this much more (m). */
const RETAKE_COST = 240;
/** A road on from a port nobody took goes to what lies within this (radians) of straight on from it, rather than what lies nearer off to the side. */
const AHEAD = Math.PI / 3;
/** What going on to something off to the side of such a port or behind it costs, more than to what lies ahead (cells of flat open land). */
const ASIDE = 30;
/** A country road's course, before it is smoothed into a curve, has a point at least this often (m): see `evenly`. */
export const CTRL_APART = 16;
/**
 * A country road's course that comes within `SHARED_REACH` cells of a road laid already (not a
 * trail) for `SHARED_RUN` cells or more (in a row, give or take `SHARED_GAP`), heading its way give
 * or take `SHARED_ANGLE`, follows it there: it is laid only up to it and on from it, joining it in
 * a T (`shared`), and that road carries it over the stretch, rather than the two running
 * side by side a few metres apart (cars on them meet nose to nose, each pulling over to its right).
 */
const SHARED_REACH = 2;
const SHARED_RUN = 3;
const SHARED_GAP = 2;
const SHARED_ANGLE = Math.PI / 4;
/** A country road tries this many ways for one that doesn't wind round further than `WINDING` (radians) within `WINDING_OVER` m (a loop). */
export const TRIES = 4;
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
/** The longer straight leads (m) a country road tries out of a way or into one it meets at a slant (`road`). */
const LONG_LEADS = [24, 34, 44];
/** The steepest a country road climbs (`builtSurface.ts` `GRADE.country`). */
export const COUNTRY_GRADE = 0.12;
/** A country road keeps under this grade where it can in a place's world (a little under its limit, `builtSurface.ts` `GRADE`), a step steeper costing it `ROUTE_STEEP` cells of flat open land for each 1 of grade over. */
export const ROUTE_GRADE = 0.1;
const ROUTE_STEEP = 300;
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
/** The world's edges (north, east, south, west): outwards. */
export const FACING: readonly Pt[] = [
  [0, -1],
  [1, 0],
  [0, 1],
  [-1, 0],
];

const dist = (a: Pt, b: Pt) => Math.hypot(a[0] - b[0], a[1] - b[1]);
const round2 = (v: number) => Math.round(v * 100) / 100;

/** The land as the generator reads it: per 8 m cell, its height (m), whether it is water, whether it is wooded. */
export class Cells {
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

  /** Whether any of the land within `by` m of a box is water (looked at every 4 m). */
  wetNear(r: Rect, by: number): boolean {
    for (let z = r.z - by; z <= r.z + r.d + by; z += CELL / 2) for (let x = r.x - by; x <= r.x + r.w + by; x += CELL / 2) if (this.water[this.cell(x, z)]) return true;
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

/** What a step from one cell to a neighbour costs more than its cells do (a climb steeper than a road may make). */
type Rise = (from: number, to: number) => number;

/**
 * The cheapest way over a `gw` × `gd` grid (8 neighbours, cost per cell `cost`: Infinity where
 * blocked, and `rise` more a step from cell to cell where given) from any of `starts` to a cell
 * `goal` accepts, `h` guessing the rest: the cells, start first.
 */
export function astar(gw: number, gd: number, cost: (i: number) => number, starts: readonly number[], goal: (i: number) => boolean, h: (i: number) => number, limit = 400000, rise?: Rise): number[] | null {
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

/** The eight ways a cell steps to its neighbours, in turn round the compass, an eighth of a turn apart. */
const STEPS: readonly Pt[] = [
  [1, 0],
  [1, 1],
  [0, 1],
  [-1, 1],
  [-1, 0],
  [-1, -1],
  [0, -1],
  [1, -1],
];
/** The step nearest heading `d`. */
const octant = ([x, z]: Pt) => (((Math.round(Math.atan2(z, x) / (Math.PI / 4)) % 8) + 8) % 8);
/** How nearly step `h` heads along `d` (a unit vector): the cosine between them. */
export const along = (h: number, d: Pt) => (STEPS[h][0] * d[0] + STEPS[h][1] * d[1]) / Math.hypot(...STEPS[h]);

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

/** Whether `p` is clear of every area's box by more than `by` m. */
export const clearOfBoxes = (areas: readonly { rect: Rect }[], p: Pt, by: number) =>
  areas.every((a) => p[0] < a.rect.x - by || p[1] < a.rect.z - by || p[0] > a.rect.x + a.rect.w + by || p[1] > a.rect.z + a.rect.d + by);

/** A road end that led out of an area's box (`AreaPlan.ports`): where, which way out, and whether a country road goes on from it yet. */
export interface Port {
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
export interface Way {
  p: Pt;
  dir: Pt;
  reach?: number;
  port?: Port;
}

/** A way found over the grid: its cells, which of the ways it was asked from it leaves by, the heading it ends on, and whether it was steered (else found by A*). */
export interface Route {
  cells: number[];
  from: number;
  h?: number;
  steered: boolean;
}

/** A stretch of a country road's course: its ends, its cells, and whether it is the course's first or last (`shared`). */
export interface Piece {
  a: Way;
  cells: number[];
  b: Way | null;
  first: boolean;
  last: boolean;
}

/** What a cell the route might end on costs to end there (-1: it may not), heading `h`, more loosely if `loose`. */
export type Goal = (i: number, h?: number, loose?: boolean) => number;

/** What a cell costs a country road: where it may go (Infinity: not), and where it may go when that finds no way. */
export type Costs = readonly [(i: number) => number, (i: number) => number];

/** The country road on from a port (`CountryRouter.onward`), as it would be laid: its road whole, the port it ends at (if one), how bad it is, its course, its far end (null: out of the world over edge `side`); and the way it leaves by. */
export interface Onward {
  made: { road: RoadPlan; to?: Port; bad: number; cells: number[]; b: Way | null; side: number };
  from: Way;
}

export class CountryRouter {
  /** Per cell, the area (its index + 1) whose box (and 6 m round it) covers it; 0 none. */
  readonly areaBlock: Uint8Array;
  /** The cells along the world's edge (two deep). */
  readonly rim: Uint8Array;
  /** The cells that are water or touch it. */
  readonly wetNear: Uint8Array;
  /** The ports left without a way on because every way on was too steep (`tooSteep`). */
  readonly steepOn = new Set<Port>();
  private readonly climb: number;
  /**
   * A place's land (its town follows its slopes: `landFollow.ts`): what a step costs a country road
   * where it climbs more steeply than `ROUTE_GRADE`, so it keeps within a country road's grade
   * where the land lets it (along the valleys, through the passes, round a steep bank, rather than
   * over the massif). None in a classic world.
   */
  private readonly rise: Rise | undefined;

  /**
   * A router over `cells` (the land as it stands: its woods and water read as they are when it
   * routes), keeping out of `areas`' boxes but by their ways in and out, sharing `roads` (read
   * afresh each time: the caller adds the roads it lays to the same list), as `spec`'s world has it
   * (a classic world's or a place's; how much height counts against a cell).
   */
  constructor(
    readonly cells: Cells,
    readonly areas: readonly { rect: Rect }[],
    readonly roads: readonly RoadPlan[],
    readonly spec: Pick<WorldSpec, 'classic' | 'climb'>,
  ) {
    const { gw, gd } = cells;
    this.areaBlock = new Uint8Array(gw * gd);
    for (const [a, plan] of areas.entries()) {
      const r = plan.rect;
      for (let z = Math.floor((r.z - 6) / CELL); z <= Math.floor((r.z + r.d + 6) / CELL); z++) {
        for (let x = Math.floor((r.x - 6) / CELL); x <= Math.floor((r.x + r.w + 6) / CELL); x++) if (x >= 0 && z >= 0 && x < gw && z < gd) this.areaBlock[z * gw + x] = a + 1;
      }
    }
    this.rim = new Uint8Array(gw * gd);
    this.climb = spec.climb ?? 0;
    for (let i = 0; i < gw * gd; i++) if (i % gw < 2 || i % gw >= gw - 2 || i < gw * 2 || i >= gw * (gd - 2)) this.rim[i] = 1;
    this.wetNear = new Uint8Array(gw * gd);
    for (let i = 0; i < gw * gd; i++) if (cells.water[i]) for (const d of [0, -1, 1, -gw, gw, -gw - 1, -gw + 1, gw - 1, gw + 1]) if (i + d >= 0 && i + d < gw * gd) this.wetNear[i + d] = 1;
    this.rise = spec.classic
      ? undefined
      : (a, b) => {
          const g = Math.abs(cells.height[b] - cells.height[a]) / (a % gw !== b % gw && Math.floor(a / gw) !== Math.floor(b / gw) ? CELL * Math.SQRT2 : CELL);
          return g > ROUTE_GRADE ? (g - ROUTE_GRADE) * ROUTE_STEEP : 0;
        };
  }

  /** What a cell costs a country road (Infinity where it may not go: on or by the water, in an area but within 26 m of `ends`, its ways in and out). */
  cost(ends: readonly Pt[]): (i: number) => number {
    const { cells: land, areaBlock, wetNear, rim, climb } = this;
    return (i: number) => {
      if (wetNear[i]) return Infinity;
      if (areaBlock[i]) {
        const c = land.centre(i);
        if (!ends.some((e) => dist(c, e) < 26)) return Infinity;
      }
      // (Not along the world's edge: smoothed, it would wander out of the world and back. In hilly
      // land and the mountains, along the valleys.)
      return 1 + land.slope(i) * 40 + land.woods[i] * 0.3 + (rim[i] ? 12 : 0) + (climb ? land.height[i] * climb : 0);
    };
  }

  /** As `cost`, but along the water's edge too (not over it), and less put off by the slope: for when that finds no way. */
  relaxed(ends: readonly Pt[]): (i: number) => number {
    const { cells: land, wetNear } = this;
    const strict = this.cost(ends);
    return (i: number) => (land.water[i] ? Infinity : wetNear[i] ? 30 + land.slope(i) * 15 : Math.min(strict(i), 1 + land.slope(i) * 15 + land.woods[i] * 0.3));
  }

  /** `cost` and `relaxed` for a road whose ways in and out are `ends`. */
  costs(ends: readonly Pt[]): Costs {
    return [this.cost(ends), this.relaxed(ends)];
  }

  /**
   * The way a country road goes from any of `from` (a port, or a T off a road), heading its way,
   * to a cell `goal` accepts (and a heading it accepts, when it is given one, more loosely if
   * asked: what ending there costs, -1 if it doesn't), `h` guessing how far is left: as `steer`
   * finds it over the first of `costs` (`costs([])` if not given), or where that finds no way over
   * the second (each such cell `DETOUR` more), never heading back against `toward`, else coming in
   * more loosely, else heading any way; else as `astar` does (`steered` false). Its cells, which of
   * `from` it leaves by and the heading it ends on; null if there is none.
   */
  route(from: readonly Way[], goal: Goal, h: (i: number) => number, toward?: Pt, [cost, fallback]: Costs = this.costs([])): Route | null {
    const { gw, gd } = this.cells;
    const starts = from.map((w) => ({ cell: this.outCell(w), h: octant(w.dir), g: this.wayCost(w) }));
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
    const found = (toward && steer(gw, gd, grid, starts, either, h, toward, undefined, this.rise)) || steer(gw, gd, grid, starts, goal, h, undefined, LOOSE_LIMIT, this.rise);
    if (found) return { cells: found.cells, from: found.start, h: found.h, steered: true };
    for (const c of [cost, fallback]) {
      const cells = astar(gw, gd, c, [starts[0].cell], (i) => goal(i) >= 0, h, undefined, this.rise);
      if (cells) return { cells, from: 0, steered: false };
    }
    return null;
  }

  /** Leaving by a port taken already: from the port itself (`RETAKE_COST`), or off the road on from it (`VIA_COST`). */
  takenCost(w: Way): number {
    return w.port?.used ? (w.p === w.port.p ? RETAKE_COST : VIA_COST) : 0;
  }

  /** What leaving by a way costs `steer` (in cells) before it is out over the land: its lead, and more for a T than a port, and for a port taken already. */
  wayCost(w: Way): number {
    return ((w.reach ?? CELL) + (w.reach ? SIDE_COST : 0) + this.takenCost(w)) / CELL;
  }

  /** The cell a country road's way over the land starts from, leaving by `w` (or ends on, coming in by it): a cell out in front of it, or where its reach ends. */
  outCell(w: Way): number {
    return this.cells.cell(w.p[0] + w.dir[0] * (w.reach ?? CELL), w.p[1] + w.dir[1] * (w.reach ?? CELL));
  }

  /** A way's way in, from its `outCell`: heading back into it, give or take a twelfth of a turn (`loose`: three sixteenths). */
  into(q: Way): (h?: number, loose?: boolean) => boolean {
    return (h?: number, loose = false) => h === undefined || along(h, [-q.dir[0], -q.dir[1]]) > (loose ? 0.38 : 0.85);
  }

  /** A country road from `a` along `cells` to `b` (else a dead end): smoothed into a curve, leaving and arriving straight (on across an area's land from a T). */
  road(a: Way, cells: readonly number[], b: Way | null, ends: RoadPlan['ends'], name: string): RoadPlan {
    const land = this.cells;
    const mid = simplify(
      cells.map((i) => land.centre(i)),
      5,
    ).slice(1, b ? -1 : undefined);
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
    return { points, width: COUNTRY_WIDTH, ends, kind: 'country', name };
  }

  /** A country road from `a` along `cells` out of the world over edge `side` (`FACING`), 40 m on past its last cell. */
  toEdge(a: Way, cells: readonly number[], side: number, name: string): RoadPlan {
    const last = this.cells.centre(cells[cells.length - 1]);
    const road = this.road(a, cells, null, ['join', 'out'], name);
    road.points = curve(evenly([...simplify(road.points, 0.3), [last[0] + FACING[side][0] * 40, last[1] + FACING[side][1] * 40]], CTRL_APART));
    return road;
  }

  /**
   * A country road's course from `a` along `cells` to `b` (null: on out of the world), in pieces
   * where it runs along a road laid already (`SHARED_RUN`): the stretches it would run alongside
   * that road are left to it, and each piece between joins it square in a T (merged into it,
   * rather than side by side a few metres off); from an end right by one it runs along (`shortcut`),
   * straight onto it, or off it straight into the end; each T facing the side the piece goes. Each
   * piece's ends (`a`, `b`: the course's own, or such a T) and its cells, and whether it is the
   * course's first or last stretch; one piece, the whole course, if it runs alongside none.
   */
  shared(a: Way, cells: readonly number[], b: Way | null): Piece[] {
    const { cells: land, roads } = this;
    const { W, D, gw, gd } = land;
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
    const out: Piece[] = [];
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
  }

  /** A piece of a country road (`shared`): straight from its start onto the road beside it if it has no cells, out of the world over edge `side` if it is the last and goes out. */
  piece(part: Piece, side = -1): RoadPlan {
    return !part.cells.length
      ? { points: curve(evenly([part.a.p, part.b!.p], CTRL_APART)), width: COUNTRY_WIDTH, ends: ['join', 'join'], kind: 'country', name: '' }
      : part.last && side >= 0
        ? this.toEdge(part.a, part.cells, side, '')
        : this.road(part.a, part.cells, part.b, ['join', 'join'], '');
  }

  /**
   * Whether the roads laid so far and `extra` (not trails), joined as the simulation joins them (an
   * end within half the wider one's width and a metre of the other's middle), join `p` to `q`.
   */
  joins(extra: readonly RoadPlan[], p: Pt, q: Pt): boolean {
    const all = [...this.roads, ...extra].filter((r) => r.kind !== 'trail');
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
  }

  /**
   * A country road (`whole`, laid from `a` along `cells` to `b`, or out of the world over edge
   * `side`) as it goes down: in pieces where it runs along a road laid already (`shared`), if they
   * and the roads join its ends still and none turns or winds worse than it would; else whole. The
   * roads (the whole one not yet `settle`d), and whether they leave its first end and come to its last.
   */
  course(whole: RoadPlan, a: Way, cells: readonly number[] | undefined, b: Way | null, side = -1): { laid: RoadPlan[]; first: boolean; last: boolean } {
    const parts = cells ? this.shared(a, cells, b) : null;
    if (!parts || (parts.length === 1 && parts[0].first && parts[0].last)) return { laid: [whole], first: true, last: true };
    // (Not a stub where a port touches the road beside it already: the two join as they are.)
    const laid = parts.map((part) => this.settle(this.piece(part, side))).filter((r) => lengths(r.points)[r.points.length - 1] > COUNTRY_WIDTH / 2 + 1);
    // (Nor pieces that wind or turn worse than the road would whole: a short one between two Ts
    // close together can double back.)
    const worst = Math.max(0, ...laid.map((r) => this.fault(r)));
    if (worst > this.fault(this.settle(whole)) || !this.joins(laid, whole.points[0], whole.points[whole.points.length - 1])) return { laid: [whole], first: true, last: true };
    return { laid, first: !!parts[0]?.first, last: !!parts[parts.length - 1]?.last };
  }

  /**
   * Where a country road leaves port `q` towards `toward`: the port itself; one another country
   * road leaves already, off that road 30 m out from it, square to it (rather than both from the
   * port, running together a while).
   */
  via(q: Port, toward: Pt): Way {
    const self: Way = { p: q.p, dir: q.dir, port: q };
    if (!q.used) return self;
    // (Not its own road: a bridge's far end is a port.)
    const r = this.roads.find((o, i) => i !== q.road && o.kind === 'country' && [o.points[0], o.points[o.points.length - 1]].some((e) => dist(e, q.p) < 1));
    if (!r) return self;
    const ls = lengths(r.points);
    const L = ls[ls.length - 1];
    if (L < 70) return self;
    const s = dist(r.points[0], q.p) < 1 ? 30 : L - 30;
    const { p, t } = pointAt(r.points, s, ls);
    const n = leftOf(t);
    const k = n[0] * (toward[0] - p[0]) + n[1] * (toward[1] - p[1]) >= 0 ? 1 : -1;
    return { p, dir: [n[0] * k, n[1] * k], port: q };
  }

  /**
   * What is wrong with a country road's course (where it meets another road aside), worst first:
   * 3 if it runs over the water, 2 if it turns sharper than `COUNTRY_TURN` over 40 m anywhere, 1
   * if it winds round further than `WINDING` within `WINDING_OVER` m (a loop), else 0.
   */
  fault(road: RoadPlan): number {
    const land = this.cells;
    return road.points.some(([x, z]) => land.wet(x, z, 1)) ? 3 : sharpestTurn(road.points, 20).angle > COUNTRY_TURN - TURN_SPARE ? 2 : winding(road.points, WINDING_OVER) > WINDING ? 1 : 0;
  }

  /** A country road that turns too sharply (`fault` 2) relaxed till it doesn't (`unkink`), if that keeps it off the water; else as it was. */
  settle(road: RoadPlan): RoadPlan {
    if (this.fault(road) !== 2) return road;
    const land = this.cells;
    const points = unkink(road.points, COUNTRY_TURN - TURN_SPARE);
    return points && !points.some(([x, z]) => land.wet(x, z, 1)) ? { ...road, points } : road;
  }

  /**
   * Whether a place's country road along `pts` would have to stand off its land by more than
   * `STEEP_OFF` m to keep within `COUNTRY_GRADE` (the land's heights along it, the least the most a
   * road within its grade must leave them by: half the gap between its grade-limited envelopes).
   * Never in a classic world.
   */
  tooSteep(pts: readonly Pt[]): boolean {
    if (this.spec.classic || pts.length < 2) return false;
    const land = this.cells;
    const line = evenly(pts, CELL / 2);
    const ls = lengths(line);
    const up = line.map(([x, z]) => land.at(x, z));
    const down = up.slice();
    for (const [a, b] of [
      [1, 0],
      [-1, line.length - 1],
    ] as const) {
      for (let i = b + a; i >= 0 && i < line.length; i += a) {
        const g = COUNTRY_GRADE * Math.abs(ls[i] - ls[i - a]);
        up[i] = Math.min(up[i], up[i - a] + g);
        down[i] = Math.max(down[i], down[i - a] - g);
      }
    }
    return up.some((u, i) => (down[i] - u) / 2 > STEEP_OFF);
  }

  /** Which edge of the world cell `i` is on (`FACING`: north, east, south, west), -1 none. */
  edgeOf(i: number): number {
    const { gw, gd } = this.cells;
    return [Math.floor(i / gw) === 0, i % gw === gw - 1, Math.floor(i / gw) === gd - 1, i % gw === 0].indexOf(true);
  }

  /**
   * The country road on from port `q`, as it would be laid: to the nearest country road (not its
   * own) out in the country (more than 30 m from every area's box), one of `others` (other areas'
   * ports nobody took: into their areas only by them) or the edge of the world; if the road there
   * turns too sharply or winds round, the nearest somewhere else. Its course, where it ends, and
   * the way it leaves by. Null if there is none, or if every way on was too steep (`tooSteep`: the
   * port is then in `steepOn`).
   */
  onward(q: Port, others: readonly Port[]): Onward | null {
    const { cells: land, roads, areaBlock } = this;
    const { gw, gd } = land;
    const plain = (p: Pt) => clearOfBoxes(this.areas, p, 30);
    /** Country roads' cells out in the country, and which way the road runs there. */
    const country = new Map<number, { road: number; t: Pt }>();
    roads.forEach((r, i) => {
      if (r.kind === 'country' && i !== q.road) r.points.forEach((p, j) => plain(p) && country.set(land.cell(p[0], p[1]), { road: i, t: tangent(r.points, j) }));
    });
    const portAt = new Map(others.map((x) => [land.cell(x.p[0] + x.dir[0] * CELL, x.p[1] + x.dir[1] * CELL), x]));
    // A country road met square to it (give or take an eighth of a turn), a port head on, the
    // world's edge heading straight out of it; what lies off to the side or behind the port
    // costs more (a road swinging round to it).
    const meets = (i: number, h?: number) => {
      const on = country.get(i);
      if (on) return h === undefined || Math.abs(along(h, on.t)) < 0.75;
      const port = portAt.get(i);
      if (port) return this.into(port)(h);
      const side = this.edgeOf(i);
      return side >= 0 && (h === undefined || along(h, FACING[side]) > 0.9);
    };
    const inFront = (w: Way, p: Pt, angle: number) => w.dir[0] * (p[0] - w.p[0]) + w.dir[1] * (p[1] - w.p[1]) >= dist(p, w.p) * Math.cos(angle);
    const banned = new Set<number>();
    const goal = (i: number, h?: number) => (banned.has(i) || !meets(i, h) ? -1 : inFront(q, land.centre(i), AHEAD) ? 0 : ASIDE);
    // (Into the other areas only by their ports.)
    const cost = (strict: boolean) => {
      const c = strict ? this.cost([q.p]) : this.relaxed([q.p]);
      return (i: number) => (areaBlock[i] && others.some((x) => dist(land.centre(i), x.p) < 26) ? 1 : c(i));
    };
    const from: Way = { p: q.p, dir: q.dir, port: q };
    // The nearest; if the road there turns too sharply or winds round, the nearest somewhere else.
    let made: Onward['made'] | null = null;
    for (let tries = 0; tries < TRIES && (!made || made.bad > 0); tries++) {
      const cells = this.route([from], goal, () => 0, undefined, [cost(true), cost(false)])?.cells;
      const end = cells?.[cells.length - 1];
      if (!cells || end === undefined) break;
      const to = portAt.get(end);
      const other = country.get(end)?.road;
      let road: RoadPlan;
      // (Its course and where it ends: a port, a T, out of the world over edge `side`.)
      let [course, b, side] = [cells, null as Way | null, -1];
      if (to) {
        b = { p: to.p, dir: to.dir, port: to };
        road = this.road(q, cells, b, ['join', 'join'], '');
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
        road = this.road(q, course, b, ['join', 'join'], '');
      } else {
        side = this.edgeOf(end);
        road = this.toEdge(q, cells, side, '');
      }
      const bad = this.fault(road) + (this.tooSteep(road.points) ? STEEP_FAULT : 0);
      if (!made || bad < made.bad) made = { road, to, bad, cells: course, b, side };
      // (Not there again.)
      const [ex, ez] = [end % gw, Math.floor(end / gw)];
      for (let dz = -3; dz <= 3; dz++) for (let dx = -3; dx <= 3; dx++) if (ex + dx >= 0 && ez + dz >= 0 && ex + dx < gw && ez + dz < gd) banned.add((ez + dz) * gw + ex + dx);
    }
    // (A place's street end with no way on a country road could climb: it ends there.)
    if (made && made.bad >= STEEP_FAULT) {
      this.steepOn.add(q);
      return null;
    }
    return made && { made, from };
  }

  /**
   * The roads the country road on from a port (`onward`) is laid as (`course`, each `settle`d, not
   * yet named), and whether the first leaves the port and the last comes to where it goes.
   */
  layOnward({ made, from }: Onward): { laid: RoadPlan[]; first: boolean; last: boolean } {
    const { laid, first, last } = this.course(made.road, from, made.cells, made.b, made.side);
    return { laid: laid.map((r) => this.settle(r)), first, last };
  }
}
