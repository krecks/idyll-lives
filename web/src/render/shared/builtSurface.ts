/**
 * Built land on a land that slopes (engine-neutral): the height of every road along it, of every
 * lot, and of the land round them graded to meet them, all worked out from a world's land
 * (`game/land.ts`), its roads, its plots and its junctions, the same way wherever it is asked (the
 * generator bakes it into the land it makes, `game/landFollow.ts` `withLand`; the renderer works it
 * out again when a world loads). Nothing of it is stored.
 *
 * - **Roads** climb with the land, smoothed and no steeper than their kind may be (`GRADE`); the
 *   trails, footpaths, lie on the land as it is (their `GRADE` only a measure), meet nothing at a
 *   junction and have nothing graded to them. The others are solved together: each starts as the land along its middle line averaged over
 *   `PROFILE_REACH` m either way, and the network as a whole is first brought within its grades
 *   the least it can be (`withinGrades`); then, `PASSES` times over, each road is smoothed, its grade clamped
 *   (moving both ends of a stretch too steep), every junction made one height (its roads' there,
 *   averaged) over each arm out to where its crosswalk ends (a road joining another without a
 *   junction, a trail or a road carried on, meets it at one height), and every bridge's deck lifted
 *   clear of its water. Last, the network is brought within its grades once more (the passes leave
 *   an arm between two junctions close together a little steep here and there).
 * - **Lots** are level pads. Each lot's way in (its driveway, else its path) meets its road at the
 *   *entry* (`entryY`), and its pad (`padY`) is within `ENTRY_MAX` of that: a short sloped drive or
 *   a step or two between them. Within that, a pad stands as near the middle of the land under it
 *   as it can (the least cut and fill), no further than `RETAIN_MAX` from its road along its front,
 *   and no further than that from the pads it touches, as far as the windows allow. A lot off any
 *   road stands at the middle of the land under it. So do the backdrop's buildings, their entry the
 *   road nearest them.
 * - **The land round them** is held between two cones from everything built, `bank` (the world's
 *   `LandscapeParams.slope`) steep: no higher than the built land nearest plus `bank` a metre away
 *   from it, no lower than it less as much. Under a road it is the road's height less `ROAD_SINK`
 *   out to its sidewalk's edge, and the road's height round the cells its edge crosses (the land
 *   drawn between the samples meets the road under its edge, never over it); inside a lot the
 *   pad's, and the pad's round the cells its edge crosses where the land there is within
 *   `SKIRT_MOST` of it (the land drawn between the samples meets the pad at its edge: a lawn runs
 *   on from it, not over its edge or under it); beside a lot within `RETAIN_MAX` of the pad (the
 *   tallest a retaining edge stands).
 *   Under a deck carried on over dry land well past its water, it is drawn `DECK_UNDER` under the
 *   deck's road at least (stored as graded: `graded`). Water is left as it is.
 *
 * The world's grain is the land's (`LAND_CELL` m samples). `at(x, z)` is the surface anything stands
 * on: a lot's pad, a road's height across it, else the graded land between its samples.
 */

import type { JunctionInfo, RoadKind, RoadPlan } from '../../core/protocol';
import { LAND_CELL, NONE, heightAt, type Land, type LandRect } from '../../game/land';
import { bridgeSpans, lengths, nearest, pointAt, RoadIndex } from '../../game/roads';
import { CHUNK, chunkAt, chunksIn } from './chunks';
import { carriageway, CROSSWALK, KERB_RETURN, planJunctions, type JunctionPlan } from './junctions';

/** The steepest each kind of way climbs (rise per m); a trail's is a measure only (it lies on the land). */
export const GRADE: Record<RoadKind, number> = { street: 0.08, lane: 0.08, country: 0.12, trail: 0.22 };
/** The tallest a lot's edge stands over the land beside it (m), the deepest a road or a lot is cut into the land or stands on it (m). */
export const RETAIN_MAX = 1.5;
export const CUT_MAX = 4;
/**
 * Round a lot, the land at the samples a cell or less outside it (the corners of the cells its edge crosses) is
 * graded to its pad where it lies within this (m) of it (`solveLand`). Lots don't keep to the land's 2 m grain, so
 * without it the land drawn across such a cell stood over the pad's edge or under it: on a gentle slope a lawn with a
 * crease and a small step at the lot's front, burying its path's first metre and its fence's foot. Further off, a
 * retaining edge stands as ever.
 */
export const SKIRT_MOST = 0.5;
/** Among mountains (a world whose Hills are `mountains`), a country road (a pass, a way out of the world) may be cut into the rock or stand over the land by this much (m). */
export const CUT_MAX_MOUNTAIN_ROAD = 10;
/** The graded land under a road lies this far below it (m), so it never shows through on a bend. */
export const ROAD_SINK = 0.06;
/** A bridge's deck stands at least this far over its water (m). */
export const BRIDGE_CLEAR = 1.2;
/**
 * Under a deck carried on over dry land past its water (a drawn street's, over a river's dip: `bridges.ts`
 * `grownDeck`; further than `DECK_DRY` m past the water, so never the generator's, whose deck ends `DECK_PAST`
 * m past it), out to its stone's edge (`DECK_APRON` m past the road's: `street.ts` `BRIDGE.apron`), the land
 * lies at least `DECK_UNDER` m under its road: under its stone (`BRIDGE.under`, 0.6 m), the bank going down
 * under the bridge as at an abutment, not graded up to the road the deck leaves.
 */
export const DECK_UNDER = 0.8;
const DECK_DRY = 4;
const DECK_APRON = 2;

/** A road's height is solved every this far along it (m). */
export const PROFILE_STEP = 2;
/** The land along a road is averaged over this far either way (m) for its first height. */
const PROFILE_REACH = 12;
/** How many times the roads are smoothed, clamped, joined and lifted over their water. */
const PASSES = 40;
/** A bridge's deck rises this much in its middle a metre of its span (m), and no more than `HUMP_MOST`. */
const HUMP = 0.01;
const HUMP_MOST = 0.6;
/** A road's end joins another road within half the wider's width and this (m) of the other's middle, as the sim joins them. */
const JOIN_MARGIN = 1;
/** Joins of roads this close (m) are one junction. */
const NODE_MERGE = 3;
/** A junction the sim reports is the one found here within this (m). */
const PLAN_MATCH = 6;
/** Where along a road (m from an end) counts as its end. */
const END_NEAR = 0.5;
/** The furthest (m) a town's streets are moved off their land for the country roads between towns (`tiered`). */
const TOWN_GIVE = 2;
/** A road is smoothed where its height bends by more than this (m) from one sample to the next (a grade changing by 1 % over `PROFILE_STEP`). */
const BEND = 0.02;
/** A height changed by more than this (m) is a change (`update`, `version`). */
const MOVED = 0.001;

/** A lot as the surface takes it: its rectangle, and where its way in meets its road (null: off any road). */
export interface SurfacePlot {
  x: number;
  z: number;
  w: number;
  d: number;
  entry: { road: number; s: number } | null;
  /** Along its entry's road, the stretch its front faces (arc lengths, the lesser first); none for a backdrop building. */
  front?: [number, number];
  /**
   * A turned backdrop building's footprint (the box above is its box): its pad is this, not its box, and gives
   * way to a road (a building turned by a junction would otherwise lift its box's corner over the junction).
   */
  poly?: readonly (readonly [number, number])[];
}

/** How far round a turned backdrop building's footprint its pad's land reaches (m): the widest of its forecourt, yard or garden. */
const GROUND_ROUND = 3.5;

/** Whether (x, z) is on `p`'s pad (or within `by` m of it): in its box, and in its ground if it is turned (`SurfacePlot.poly`). */
export function onPad(p: SurfacePlot, x: number, z: number, by = 0): boolean {
  if (x < p.x - by || x > p.x + p.w + by || z < p.z - by || z > p.z + p.d + by) return false;
  if (!p.poly) return true;
  let inside = false;
  let near = Infinity;
  const q = p.poly;
  for (let i = 0, j = q.length - 1; i < q.length; j = i++) {
    const [xi, zi] = q[i];
    const [xj, zj] = q[j];
    if (zi > z !== zj > z && x < ((xj - xi) * (z - zi)) / (zj - zi) + xi) inside = !inside;
    if (by > 0) {
      const [dx, dz] = [xj - xi, zj - zi];
      const l2 = dx * dx + dz * dz;
      const t = l2 > 0 ? Math.max(0, Math.min(1, ((x - xi) * dx + (z - zi) * dz) / l2)) : 0;
      near = Math.min(near, Math.hypot(x - xi - dx * t, z - zi - dz * t));
    }
  }
  return inside || near <= by;
}

/** A pad stands no further than this (m) above or below its road at its entry (a short drive, a few steps). */
export const ENTRY_MAX = 1;
/** How many times the pads are set again to meet their neighbours'. */
const PAD_PASSES = 8;
/** Pads touching each other (no more than the land's grain apart) are kept this near (m): a retaining edge, less a little. */
const PAD_STEP = RETAIN_MAX - 0.05;

/** A road's solved height: its samples (arc length, where they stand, the land there, the height). */
interface Profile {
  len: number;
  s: Float32Array;
  x: Float32Array;
  z: Float32Array;
  ground: Float32Array;
  y: Float32Array;
  /** The lowest its height may be (a bridge's deck), -Infinity elsewhere. */
  floor: Float32Array;
  /** Held by a junction (1): not smoothed, and its height the junction's. */
  pin: Uint8Array;
  grade: number;
}

/** A road at a junction or a join: where along it, and the samples held at the junction's height. */
interface Attach {
  road: number;
  s: number;
  /** The samples held (first and last, inclusive; -1: none). */
  from: number;
  to: number;
  /** Whether it is a way for cars (not a trail). */
  car: boolean;
  /** Whether it runs on through (else it ends here). */
  through: boolean;
}

/** Where roads meet: a junction (three or more arms for cars: `hold`) or a join (a road carried on, a trail). */
interface Node {
  x: number;
  z: number;
  atts: Attach[];
  hold: boolean;
  /** Its height (a junction's, after the last pass). */
  y: number;
}

/** A junction as solved: where it is, its height, and its roads with how far along each it is held there (arc lengths). */
export interface SurfaceJunction {
  x: number;
  z: number;
  y: number;
  arms: { road: number; from: number; to: number }[];
}

/** What a re-solve changed (`BuiltSurface.update`): the roads and pads whose heights moved, and the samples of the graded land that did (null: none). */
export interface SurfaceChange {
  roads: number[];
  plots: number[];
  rect: LandRect | null;
}

/**
 * The surface of a world's built land and the land graded round it (see the file's comment). It
 * reads `land` when made and on `update` (a land the brushes change in place).
 */
export class BuiltSurface {
  readonly cols: number;
  readonly rows: number;
  /** The graded land (m) at every sample: what the terrain is drawn from. */
  conformed: Float32Array;
  /** Under a deck's dry reach, the graded land held down under its stone (`DECK_UNDER`) was this (m; NaN elsewhere): what is stored (`graded`), so the roads are solved from the land as it was graded. */
  private held: Float32Array;
  private profiles: Profile[] = [];
  private pads: Float64Array;
  /** Each turned backdrop building's standing in water (none of its footprint dry, its middle wet): it grades no ground (`solveLand`). */
  private afloat: Uint8Array;
  /** Each lot's road's height at its entry (NaN: none). */
  private entries: Float64Array;
  /** Each lot's neighbours: the pads within the land's grain of it. */
  private readonly touching: number[][];
  private nodes: Node[] = [];
  /** Per sample: the built height there (NaN: nothing built), and what is built (0 nothing, 1 a road, 2 a lot, `BESIDE` the land beside a road, at its height, `SKIRT` the land round a lot, at its pad's). */
  private builtY: Float32Array;
  private kind: Uint8Array;
  private readonly index: RoadIndex;
  private readonly lens: number[][];
  private readonly lotGrid = new Map<number, number[]>();
  private readonly versions = new Map<number, number>();
  /** The widest road (m). */
  private readonly widest: number;
  /** Whether the country roads are solved after the towns' streets, each on its own (`tiered`). */
  private apart = false;
  /** How many places each road is joined at (its nodes). */
  private readonly joins: number[];

  constructor(
    readonly land: Land,
    readonly roads: readonly RoadPlan[],
    readonly plots: readonly SurfacePlot[],
    readonly junctions: readonly JunctionPlan[] = [],
    readonly bank = 0.5,
  ) {
    this.cols = land.cols;
    this.rows = land.rows;
    this.index = new RoadIndex(roads);
    this.widest = Math.max(0, ...roads.map((r) => r.width));
    this.lens = roads.map((r) => lengths(r.points));
    this.profiles = roads.map((r, i) => this.sampled(r, i));
    this.nodes = this.findNodes();
    this.joins = roads.map(() => 0);
    for (const node of this.nodes) for (const r of new Set(node.atts.map((a) => a.road))) this.joins[r]++;
    plots.forEach((p, i) => {
      for (let cz = Math.floor(p.z / LOT_CELL); cz <= Math.floor((p.z + p.d) / LOT_CELL); cz++) {
        for (let cx = Math.floor(p.x / LOT_CELL); cx <= Math.floor((p.x + p.w) / LOT_CELL); cx++) {
          const key = cx * 65536 + cz;
          const list = this.lotGrid.get(key);
          if (list) list.push(i);
          else this.lotGrid.set(key, [i]);
        }
      }
    });
    this.pads = new Float64Array(plots.length);
    this.afloat = new Uint8Array(plots.length);
    this.entries = new Float64Array(plots.length);
    this.touching = plots.map((p, i) => {
      const out = new Set<number>();
      for (let cz = Math.floor((p.z - LAND_CELL) / LOT_CELL); cz <= Math.floor((p.z + p.d + LAND_CELL) / LOT_CELL); cz++) {
        for (let cx = Math.floor((p.x - LAND_CELL) / LOT_CELL); cx <= Math.floor((p.x + p.w + LAND_CELL) / LOT_CELL); cx++) {
          for (const j of this.lotGrid.get(cx * 65536 + cz) ?? []) {
            const q = plots[j];
            if (j !== i && Math.max(q.x - (p.x + p.w), p.x - (q.x + q.w), 0) <= LAND_CELL && Math.max(q.z - (p.z + p.d), p.z - (q.z + q.d), 0) <= LAND_CELL) out.add(j);
          }
        }
      }
      return [...out].sort((a, b) => a - b);
    });
    this.builtY = new Float32Array(this.cols * this.rows);
    this.kind = new Uint8Array(this.cols * this.rows);
    this.conformed = new Float32Array(this.cols * this.rows);
    this.held = new Float32Array(this.cols * this.rows).fill(NaN);
    this.solve();
  }

  // ---- what it answers ----------------------------------------------------------------------

  /** Road `road`'s height (m) `s` m along it (its ends' past them). */
  roadY(road: number, s: number): number {
    const p = this.profiles[road];
    if (!p || p.y.length === 0) return 0;
    const n = p.y.length;
    if (n === 1 || s <= 0) return p.y[0];
    if (s >= p.len) return p.y[n - 1];
    const i = Math.min(n - 2, Math.floor(s / PROFILE_STEP));
    const t = (s - p.s[i]) / (p.s[i + 1] - p.s[i] || 1);
    return p.y[i] + (p.y[i + 1] - p.y[i]) * t;
  }

  /** Lot `plot`'s pad (m): the level its ground stands at (within `ENTRY_MAX` of `entryY`). */
  padY(plot: number): number {
    return this.pads[plot] ?? 0;
  }

  /**
   * The height (m) of lot `plot`'s road where its way in meets it (`SurfacePlot.entry`): its drive
   * or path runs from there, at the road's edge, to the pad (`padY`), sloping by no more than
   * `ENTRY_MAX`. A lot off any road: its pad.
   */
  entryY(plot: number): number {
    const e = this.entries[plot];
    return Number.isFinite(e) ? e : this.padY(plot);
  }

  /** The surface (m) at (`x`, `z`): a lot's pad in a lot (the highest where two touch), a road's height on a road (its deck on a bridge), a backdrop building's pad on its ground, else the graded land. */
  at(x: number, z: number): number {
    let [pad, ground] = [-Infinity, -Infinity];
    for (const i of this.lotGrid.get(Math.floor(x / LOT_CELL) * 65536 + Math.floor(z / LOT_CELL)) ?? []) {
      const p = this.plots[i];
      if (!onPad(p, x, z)) continue;
      if (p.poly) ground = Math.max(ground, this.pads[i]);
      else pad = Math.max(pad, this.pads[i]);
    }
    if (pad > -Infinity) return pad;
    const on = this.roadAt(x, z);
    if (on) return this.roadY(on.road, on.s);
    if (ground > -Infinity) return ground;
    return this.landAt(x, z);
  }

  /** The graded land (m) at (`x`, `z`), between the samples round it (under a road, `ROAD_SINK` below it; in a lot, its pad). */
  landAt(x: number, z: number): number {
    const { cols, rows, conformed } = this;
    const fx = Math.min(cols - 1, Math.max(0, x / LAND_CELL));
    const fz = Math.min(rows - 1, Math.max(0, z / LAND_CELL));
    const i = Math.min(Math.floor(fx), Math.max(0, cols - 2));
    const j = Math.min(Math.floor(fz), Math.max(0, rows - 2));
    const [tx, tz] = [fx - i, fz - j];
    const [i1, j1] = [Math.min(i + 1, cols - 1), Math.min(j + 1, rows - 1)];
    const top = conformed[j * cols + i] * (1 - tx) + conformed[j * cols + i1] * tx;
    const bottom = conformed[j1 * cols + i] * (1 - tx) + conformed[j1 * cols + i1] * tx;
    return top * (1 - tz) + bottom * tz;
  }

  /** The road (and how far along it) whose surface (out to its sidewalks' edges) holds (`x`, `z`): of those that do, the one whose middle is nearest. */
  roadAt(x: number, z: number): { road: number; s: number } | null {
    let best: { road: number; s: number; dist: number } | null = null;
    for (const [road, m] of this.index.around(x, z, this.widest / 2)) {
      if (m.dist > this.roads[road].width / 2 || (best && best.dist <= m.dist)) continue;
      best = { road, s: m.s, dist: m.dist };
    }
    return best && { road: best.road, s: best.s };
  }

  /** The junctions (three or more ways for cars meeting), each at its one height. */
  get junctionHeights(): SurfaceJunction[] {
    return this.nodes
      .filter((n) => n.hold)
      .map((n) => ({
        x: n.x,
        z: n.z,
        y: n.y,
        arms: n.atts.filter((a) => a.from >= 0).map((a) => ({ road: a.road, from: this.profiles[a.road].s[a.from], to: this.profiles[a.road].s[a.to] })),
      }));
  }

  /** The heights road `road` was solved at (m), every `PROFILE_STEP` m along it from its start, its end last. */
  profile(road: number): { s: Float32Array; y: Float32Array } {
    const p = this.profiles[road];
    return { s: p.s, y: p.y };
  }

  /** How many updates have moved a road or a pad (`update`): what a list of every lot's pad is kept by. */
  get moves(): number {
    return this.moved;
  }
  private moved = 0;

  /** How many times the roads or the pads in street chunk `chunk` (`chunks.ts` `chunkId`) have moved (`update`): a part of its key. */
  version(chunk: number): number {
    return this.versions.get(chunk) ?? 0;
  }

  /**
   * A hash of every height street chunk `chunk` (`chunks.ts` `chunkId`) stands on, to the millimetre: the roads'
   * samples within reach of it, the pads of the lots reaching into it and the graded land under it. Two surfaces
   * (of two towns on one land: a lot removed, a street added) give a chunk the same one where nothing in it moved,
   * so its street need not be planned again. Worked out for every chunk at once, the first time one is asked for,
   * and again after `update`.
   */
  chunkSig(chunk: number): number {
    this.sigs ??= this.chunkSigs();
    return this.sigs.get(chunk) ?? 0;
  }
  private sigs: Map<number, number> | null = null;

  /** A hash of every road's heights (to the millimetre): two surfaces with the same roads and this the same draw them alike. */
  roadsSig(): number {
    let h = 0x811c9dc5;
    for (const p of this.profiles) {
      h = Math.imul(h ^ p.y.length, 16777619);
      for (let i = 0; i < p.y.length; i++) h = Math.imul(h ^ Math.round(p.y[i] * 1000), 16777619);
    }
    return h >>> 0;
  }

  private chunkSigs(): Map<number, number> {
    const out = new Map<number, number>();
    const add = (c: number, v: number) => out.set(c, Math.imul((out.get(c) ?? 0x811c9dc5) ^ Math.round(v * 1000), 16777619));
    // (Each height into every chunk within reach of it: what stands at a chunk's edge reads them across it.)
    const reach = this.widest / 2 + PROFILE_STEP;
    for (const p of this.profiles) for (let i = 0; i < p.y.length; i++) for (const c of chunksIn(p.x[i] - reach, p.z[i] - reach, p.x[i] + reach, p.z[i] + reach)) add(c, p.y[i]);
    this.plots.forEach((p, i) => {
      for (const c of chunksIn(p.x - LAND_CELL, p.z - LAND_CELL, p.x + p.w + LAND_CELL, p.z + p.d + LAND_CELL)) add(c, this.pads[i]);
    });
    const { cols, rows, conformed } = this;
    const per = CHUNK / LAND_CELL;
    for (let cj = 0; cj * per < rows; cj++) {
      for (let ci = 0; ci * per < cols; ci++) {
        const c = chunkAt(ci * CHUNK, cj * CHUNK);
        let h = out.get(c) ?? 0x811c9dc5;
        for (let j = Math.max(0, cj * per - 1); j <= Math.min(rows - 1, (cj + 1) * per + 1); j++) {
          for (let i = Math.max(0, ci * per - 1), k = j * cols + i; i <= Math.min(cols - 1, (ci + 1) * per + 1); i++, k++) h = Math.imul(h ^ Math.round(conformed[k] * 1000), 16777619);
        }
        out.set(c, h);
      }
    }
    for (const [c, h] of out) out.set(c, h >>> 0);
    return out;
  }

  /**
   * The land with the built land graded into it (cm, as `Land.h`): the road's own height under a
   * road (not sunk), the pad in a lot, the graded land round them; water left as it is. What the
   * generator stores (`withLand`), so a world's land agrees with its town.
   */
  graded(): Int16Array {
    const out = new Int16Array(this.cols * this.rows);
    const { land, builtY, kind, conformed, held } = this;
    for (let k = 0; k < out.length; k++) {
      if (wet(land, k)) out[k] = land.h[k];
      else out[k] = cm(kind[k] ? builtY[k] : Number.isNaN(held[k]) ? conformed[k] : held[k]);
    }
    return out;
  }

  /**
   * Solves it all again from the land as it is now (after a stroke of the land brushes touched
   * `rect`): the roads, the pads and the graded land. What moved: the roads and pads whose heights
   * changed and the rectangle of samples whose graded land did; each street chunk they are in
   * counts one more `version`. The same as a surface made afresh on this land.
   */
  update(_rect: LandRect): SurfaceChange {
    const oldY = this.profiles.map((p) => p.y.slice());
    const oldPads = this.pads.slice();
    const oldLand = this.conformed.slice();
    this.solve();
    this.sigs = null;
    const bumped = new Set<number>();
    const roads: number[] = [];
    this.profiles.forEach((p, r) => {
      let moved = false;
      for (let i = 0; i < p.y.length; i++) {
        if (Math.abs(p.y[i] - oldY[r][i]) <= MOVED) continue;
        moved = true;
        bumped.add(chunkAt(p.x[i], p.z[i]));
      }
      if (moved) roads.push(r);
    });
    const plots: number[] = [];
    this.pads.forEach((y, i) => {
      if (Math.abs(y - oldPads[i]) <= MOVED) return;
      plots.push(i);
      const p = this.plots[i];
      for (const c of chunksIn(p.x, p.z, p.x + p.w, p.z + p.d)) bumped.add(c);
    });
    for (const c of bumped) this.versions.set(c, this.version(c) + 1);
    if (roads.length || plots.length) this.moved++;
    let rect: LandRect | null = null;
    for (let k = 0; k < oldLand.length; k++) {
      if (Math.abs(oldLand[k] - this.conformed[k]) <= MOVED) continue;
      const [i, j] = [k % this.cols, Math.floor(k / this.cols)];
      rect = rect ? { i0: Math.min(rect.i0, i), j0: Math.min(rect.j0, j), i1: Math.max(rect.i1, i + 1), j1: Math.max(rect.j1, j + 1) } : { i0: i, j0: j, i1: i + 1, j1: j + 1 };
    }
    return { roads, plots, rect };
  }

  /**
   * The land under and round what is built kept within reach of it after the earth brushes changed it in `rect` (a
   * land its town follows, shaped in the game: its roads and lots move with it, within their grades): the surface
   * solved again (`update`), then round the stroke where the land is cut into or filled over by more than `CUT_MAX`
   * (a road's cut, a lot's pad) or stands more than `RETAIN_MAX` off a lot's pad beside it (or than it did `before` the stroke, if more), the land itself is brought
   * back within that, and solved again; `passes` times at most. Changes `land` in place. Says the samples it changed (null: none) and
   * what moved in all (`update`'s change, the passes' together).
   */
  settleCuts(rect: LandRect, passes = SETTLE_PASSES, before?: Int16Array): { rect: LandRect | null; change: SurfaceChange } {
    const { cols, rows, land, kind, conformed, held } = this;
    let changed: LandRect | null = null;
    let window = rect;
    const roads = new Set<number>();
    const plots = new Set<number>();
    let moved: LandRect | null = null;
    const join = (a: LandRect | null, b: LandRect | null): LandRect | null => (!a ? b : !b ? a : { i0: Math.min(a.i0, b.i0), j0: Math.min(a.j0, b.j0), i1: Math.max(a.i1, b.i1), j1: Math.max(a.j1, b.j1) });
    for (let pass = 0; pass <= passes; pass++) {
      const change = this.update(window);
      change.roads.forEach((r) => roads.add(r));
      change.plots.forEach((p) => plots.add(p));
      moved = join(moved, change.rect);
      if (pass === passes) break;
      // (Round the stroke, a margin past it: not wherever the roads it moved run.)
      const w = window;
      const [i0, j0, i1, j1] = [Math.max(0, w.i0 - SETTLE_MARGIN), Math.max(0, w.j0 - SETTLE_MARGIN), Math.min(cols, w.i1 + SETTLE_MARGIN), Math.min(rows, w.j1 + SETTLE_MARGIN)];
      let clamped: LandRect | null = null;
      for (let j = j0; j < j1; j++) {
        for (let i = i0; i < i1; i++) {
          const k = j * cols + i;
          if (wet(land, k)) continue;
          // (Beside a lot: within a retaining edge of the land graded to its pad; elsewhere within a cut or a fill.)
          let reach = CUT_MAX;
          if (!kind[k]) {
            for (let n = 0; n < 8 && reach > RETAIN_MAX; n++) {
              const [ni, nj] = [i + NEAR8[n][0], j + NEAR8[n][1]];
              if (ni >= 0 && nj >= 0 && ni < cols && nj < rows && kind[nj * cols + ni] === 2) reach = RETAIN_MAX;
            }
          }
          // (Under a deck's dry reach, the land as graded, not as held down under its stone.)
          const g = Number.isNaN(held[k]) ? conformed[k] : held[k];
          const h = land.h[k] / 100;
          // (A cut or fill the land had before the stroke (`before`) is left as it was, not made good: never past it.)
          const had = before ? Math.abs(before[k] / 100 - g) : 0;
          if (h <= g + Math.max(reach, had) + 0.005 && h >= g - Math.max(reach, had) - 0.005) continue;
          // (Brought back short of the limit: solved again, what is built comes some way after it.)
          const to = Math.max(reach * SETTLE_SHORT, had);
          land.h[k] = cm(Math.min(g + to, Math.max(g - to, h)));
          clamped = join(clamped, { i0: i, j0: j, i1: i + 1, j1: j + 1 });
        }
      }
      if (!clamped) break;
      changed = join(changed, clamped);
      window = join(window, clamped)!;
    }
    return { rect: changed, change: { roads: [...roads].sort((a, b) => a - b), plots: [...plots].sort((a, b) => a - b), rect: moved } };
  }

  // ---- solving ------------------------------------------------------------------------------

  /** The roads, the pads and the graded land, from the land as it is. */
  private solve(): void {
    this.solveRoads();
    this.solvePads();
    this.solveLand();
  }

  /** Every pad (see the file's comment). */
  private solvePads(): void {
    const { plots, land, pads, entries, touching } = this;
    const { cols, rows } = land;
    const c = LAND_CELL;
    const n = plots.length;
    const middle = new Float64Array(n);
    const window = new Float64Array(2 * n);
    plots.forEach((p, i) => {
      // The middle of the land under it (dry, inside its edges, which it may share with the lot beside it; the land at its middle if no sample is).
      let [lo, hi] = [Infinity, -Infinity];
      for (let j = Math.max(0, Math.floor(p.z / c) + 1); j <= Math.min(rows - 1, Math.ceil((p.z + p.d) / c) - 1); j++) {
        for (let k = j * cols + Math.max(0, Math.floor(p.x / c) + 1), end = j * cols + Math.min(cols - 1, Math.ceil((p.x + p.w) / c) - 1); k <= end; k++) {
          if (wet(land, k) || (p.poly && !onPad(p, (k - j * cols) * c, j * c))) continue;
          lo = Math.min(lo, land.h[k]);
          hi = Math.max(hi, land.h[k]);
        }
      }
      middle[i] = lo <= hi ? (lo + hi) / 200 : heightAt(land, p.x + p.w / 2, p.z + p.d / 2);
      // (A turned one in water, a river painted over it: its pad is the water's bed, no forecourt for the land round it.)
      this.afloat[i] = p.poly && lo > hi && wet(land, Math.min(rows - 1, Math.max(0, Math.round((p.z + p.d / 2) / c))) * cols + Math.min(cols - 1, Math.max(0, Math.round((p.x + p.w / 2) / c)))) ? 1 : 0;
      let [from, to] = [-Infinity, Infinity];
      entries[i] = NaN;
      if (p.entry) {
        const e = this.roadY(p.entry.road, p.entry.s);
        entries[i] = e;
        [from, to] = [e - ENTRY_MAX, e + ENTRY_MAX];
        if (p.front) {
          // Within a retaining edge of its road all along its front, if the window allows.
          let [low, high] = [Infinity, -Infinity];
          const [s0, s1] = p.front;
          for (let s = s0; ; s = Math.min(s1, s + PROFILE_STEP)) {
            const y = this.roadY(p.entry.road, s);
            [low, high] = [Math.min(low, y), Math.max(high, y)];
            if (s >= s1) break;
          }
          [from, to] = narrowed(from, to, high - RETAIN_MAX, low + RETAIN_MAX);
        }
      }
      window[2 * i] = from;
      window[2 * i + 1] = to;
      pads[i] = Math.min(to, Math.max(from, middle[i]));
    });
    // Near the pads they touch, as far as their windows let them.
    for (let pass = 0; pass < PAD_PASSES; pass++) {
      for (let i = 0; i < n; i++) {
        let [from, to] = [window[2 * i], window[2 * i + 1]];
        for (const j of touching[i]) [from, to] = narrowed(from, to, pads[j] - PAD_STEP, pads[j] + PAD_STEP);
        pads[i] = Math.min(to, Math.max(from, middle[i]));
      }
    }
  }

  /** Road `r`'s samples, every `PROFILE_STEP` m from its start and its end. */
  private sampled(r: RoadPlan, road: number): Profile {
    return sampleLine(r.points, this.lens[road], GRADE[r.kind ?? 'street']);
  }

  /**
   * Where the roads meet: each road's end within half the wider's width and `JOIN_MARGIN` of
   * another's middle, two roads crossing, a ring's ends, those within `NODE_MERGE` of each other
   * one; and the junctions the sim reports (`junctions`), whose arms are held as far as it draws them.
   */
  private findNodes(): Node[] {
    const { roads, lens } = this;
    const found: { x: number; z: number; atts: [number, number][] }[] = [];
    const widest = this.widest;
    roads.forEach((r, i) => {
      if (r.points.length < 2) return;
      const L = lens[i][lens[i].length - 1];
      const ends: [number, number][] = [
        [0, 0],
        [r.points.length - 1, L],
      ];
      if (r.ends[0] === 'loop') found.push({ x: r.points[0][0], z: r.points[0][1], atts: [[i, 0], [i, L]] });
      for (const [k, s] of ends) {
        if (r.ends[0] === 'loop') break;
        const [x, z] = r.points[k];
        for (const [q, m] of this.index.around(x, z, widest / 2 + JOIN_MARGIN)) {
          if (q === i || m.dist > Math.max(roads[q].width, r.width) / 2 + JOIN_MARGIN) continue;
          const p = pointAt(roads[q].points, m.s, lens[q]).p;
          found.push({ x: p[0], z: p[1], atts: [[i, s], [q, m.s]] });
        }
      }
    });
    for (const c of this.crossings()) found.push(c);
    // The joins close together, one node.
    const parent = found.map((_, i) => i);
    const root = (i: number): number => (parent[i] === i ? i : (parent[i] = root(parent[i])));
    for (let a = 0; a < found.length; a++) {
      for (let b = a + 1; b < found.length; b++) if (Math.hypot(found[a].x - found[b].x, found[a].z - found[b].z) <= NODE_MERGE) parent[root(b)] = root(a);
    }
    const groups = new Map<number, (typeof found)[number][]>();
    found.forEach((f, i) => {
      const g = groups.get(root(i));
      if (g) g.push(f);
      else groups.set(root(i), [f]);
    });
    const raw: { x: number; z: number; atts: [number, number][]; reach: Map<number, number> }[] = [];
    for (const list of groups.values()) {
      const atts: [number, number][] = [];
      for (const f of list) {
        for (const [road, s] of f.atts) {
          const L = lens[road][lens[road].length - 1];
          const had = atts.findIndex(([r, t]) => r === road && Math.abs(t - s) <= NODE_MERGE);
          if (had < 0) atts.push([road, s]);
          else if (s < END_NEAR || s > L - END_NEAR) atts[had] = [road, s];
        }
      }
      raw.push({ x: list.reduce((a, f) => a + f.x, 0) / list.length, z: list.reduce((a, f) => a + f.z, 0) / list.length, atts, reach: new Map() });
    }
    // The sim's junctions: their arms held as far as the renderer draws their pads.
    for (const j of this.junctions) {
      let node = raw.find((n) => Math.hypot(n.x - j.x, n.z - j.z) <= PLAN_MATCH);
      if (!node) {
        const atts = [...new Set(j.arms.map((a) => a.road))].filter((r) => roads[r]?.points.length >= 2).map((r): [number, number] => [r, nearest(roads[r].points, j.x, j.z, lens[r]).s]);
        raw.push((node = { x: j.x, z: j.z, atts, reach: new Map() }));
      }
      for (const a of j.arms) node.reach.set(a.road, Math.max(node.reach.get(a.road) ?? 0, a.reach));
    }
    // (A trail meets nothing: it lies on the land.)
    return raw.map((n) => ({ ...n, atts: n.atts.filter(([r]) => !this.draped(r)) })).filter((n) => n.atts.length >= 2).map((n) => this.nodeOf(n.x, n.z, n.atts, n.reach));
  }

  /** Where two roads' middles cross away from their ends (each road's segments filed by 8 m cell). */
  private crossings(): { x: number; z: number; atts: [number, number][] }[] {
    const cells = new Map<number, [number, number, number][]>();
    this.roads.forEach((r, road) => {
      const ls = this.lens[road];
      for (let k = 0; k + 1 < r.points.length; k++) {
        const [[ax, az], [bx, bz]] = [r.points[k], r.points[k + 1]];
        for (let cz = Math.floor(Math.min(az, bz) / 8); cz <= Math.floor(Math.max(az, bz) / 8); cz++) {
          for (let cx = Math.floor(Math.min(ax, bx) / 8); cx <= Math.floor(Math.max(ax, bx) / 8); cx++) {
            const key = cx * 65536 + cz;
            const list = cells.get(key);
            const seg: [number, number, number] = [road, k, ls[k]];
            if (list) list.push(seg);
            else cells.set(key, [seg]);
          }
        }
      }
    });
    const out: { x: number; z: number; atts: [number, number][] }[] = [];
    const seen = new Set<string>();
    for (const list of cells.values()) {
      for (let a = 0; a < list.length; a++) {
        for (let b = a + 1; b < list.length; b++) {
          const [ra, ka, sa] = list[a];
          const [rb, kb, sb] = list[b];
          if (ra === rb) continue;
          const key = ra < rb ? `${ra}:${ka}:${rb}:${kb}` : `${rb}:${kb}:${ra}:${ka}`;
          if (seen.has(key)) continue;
          seen.add(key);
          const [p0, p1] = [this.roads[ra].points[ka], this.roads[ra].points[ka + 1]];
          const [q0, q1] = [this.roads[rb].points[kb], this.roads[rb].points[kb + 1]];
          const [dx, dz, ex, ez] = [p1[0] - p0[0], p1[1] - p0[1], q1[0] - q0[0], q1[1] - q0[1]];
          const det = dx * ez - dz * ex;
          if (Math.abs(det) < 1e-9) continue;
          const t = ((q0[0] - p0[0]) * ez - (q0[1] - p0[1]) * ex) / det;
          const u = ((q0[0] - p0[0]) * dz - (q0[1] - p0[1]) * dx) / det;
          if (t < 0 || t > 1 || u < 0 || u > 1) continue;
          const [s1, s2] = [sa + t * Math.hypot(dx, dz), sb + u * Math.hypot(ex, ez)];
          const [L1, L2] = [this.lens[ra].at(-1)!, this.lens[rb].at(-1)!];
          // (A road's end on another is a join, found already.)
          if (s1 < END_NEAR || s1 > L1 - END_NEAR || s2 < END_NEAR || s2 > L2 - END_NEAR) continue;
          out.push({ x: p0[0] + dx * t, z: p0[1] + dz * t, atts: [[ra, s1], [rb, s2]] });
        }
      }
    }
    return out;
  }

  /** A node from its roads (and how far along them it is): which samples a junction holds, which ones decide its height. */
  private nodeOf(x: number, z: number, list: [number, number][], planned: ReadonlyMap<number, number>): Node {
    const { roads, profiles } = this;
    const atts: Attach[] = list.map(([road, s]) => {
      const r = roads[road];
      const L = profiles[road].len;
      const ring = r.ends[0] === 'loop';
      const through = !ring && s >= END_NEAR && s <= L - END_NEAR;
      return { road, s, from: -1, to: -1, car: r.kind !== 'trail', through };
    });
    const arms = atts.reduce((n, a) => n + (a.car ? (a.through ? 2 : 1) : 0), 0);
    const hold = arms >= 3;
    for (const a of atts) {
      const p = profiles[a.road];
      const n = p.y.length;
      if (!n) continue;
      const below = Math.max(0, Math.min(n - 1, Math.floor(a.s / PROFILE_STEP)));
      const above = Math.min(n - 1, a.s >= p.len ? n - 1 : below + 1);
      const at = a.s < END_NEAR ? 0 : a.s > p.len - END_NEAR ? n - 1 : -1;
      if (hold) {
        // Out past the far side of the other roads and their crosswalks (as far as the sim's own reaches, if further).
        const half = Math.max(0, ...atts.filter((o) => o !== a && o.car).map((o) => carriageway(roads[o.road]) / 2));
        const off = Math.hypot(p.x[at >= 0 ? at : below] - x, p.z[at >= 0 ? at : below] - z);
        const reach = Math.max(0, Math.max(half + KERB_RETURN + CROSSWALK, planned.get(a.road) ?? 0) - off);
        const fwd = a.through || at === 0;
        const back = a.through || at === n - 1;
        const lo = back ? a.s - reach : a.s;
        const hi = fwd ? a.s + reach : a.s;
        let from = n;
        let to = -1;
        for (let i = 0; i < n; i++) {
          if (p.s[i] < lo - 1e-4 || p.s[i] > hi + 1e-4) continue;
          from = Math.min(from, i);
          to = Math.max(to, i);
        }
        if (at >= 0) [from, to] = [Math.min(from, at), Math.max(to, at)];
        else [from, to] = [Math.min(from, below), Math.max(to, above)];
        a.from = from;
        a.to = to;
      } else if (at >= 0) a.from = a.to = at;
      else [a.from, a.to] = [below, above];
    }
    return { x, z, atts, hold, y: 0 };
  }

  /** Every road's heights (see the file's comment). */
  private solveRoads(): void {
    const { profiles, roads, land } = this;
    profiles.forEach((p, road) => {
      const n = p.y.length;
      p.pin.fill(0);
      p.floor.fill(-Infinity);
      if (this.draped(road)) {
        // A trail: the land under it, as it lies.
        for (let i = 0; i < n; i++) p.ground[i] = p.y[i] = heightAt(land, p.x[i], p.z[i]);
        return;
      }
      groundOf(p, land);
      // A bridge's deck clear of its water, a little higher in its middle.
      for (const span of bridgeSpans(roads[road])) {
        let level = -Infinity;
        for (let i = 0; i < n; i++) if (p.s[i] >= span[0] && p.s[i] <= span[1]) level = Math.max(level, waterAt(land, p.x[i], p.z[i]));
        if (level > -Infinity) {
          const hump = Math.min(HUMP_MOST, HUMP * (span[1] - span[0]));
          for (let i = 0; i < n; i++) {
            if (p.s[i] < span[0] || p.s[i] > span[1]) continue;
            p.floor[i] = level + BRIDGE_CLEAR + hump * Math.sin((Math.PI * (p.s[i] - span[0])) / (span[1] - span[0]));
          }
        }
      }
    });
    for (const node of this.nodes) for (const a of node.atts) if (a.from >= 0) profiles[a.road].pin.fill(1, a.from, a.to + 1);
    this.tiered();
    const scratch = new Float32Array(Math.max(0, ...profiles.map((p) => p.y.length)));
    const solid = profiles.filter((_, r) => !this.draped(r));
    for (let pass = 0; pass < PASSES; pass++) {
      for (const p of solid) settlePass(p, scratch);
      for (const node of this.nodes) this.join(node);
      for (const p of solid) lift(p);
    }
    // (The passes leave a junction's arm a little steep here and there, between two junctions close together: once more within the grades, as little as can be.)
    this.tiered();
    for (const p of solid) lift(p);
  }

  /** Whether road `road` is a footpath (a trail): it lies on the land as it is, no grade, no junction, nothing graded to it. */
  private draped(road: number): boolean {
    return this.roads[road].kind === 'trail';
  }

  /**
   * A road's rank: the ways between places first (the towns' streets and lanes, and the country
   * roads other roads join in two places or more, unless `apart`), then the other country roads; the
   * trails (lying on the land) none of them.
   */
  private tier(road: number): number {
    const r = this.roads[road];
    if (r.kind === 'trail') return 0;
    return r.kind === 'country' && (this.apart || this.joins[road] < 2) ? 2 : 1;
  }

  /**
   * Every road brought within its grade, moved as little as it can be, a rank at a time so a climb
   * no road could make stays on the road that has it: the ways between places all together
   * (`withinGrades`: where two villages stand further apart in height than the country road between
   * them can climb, each gives a little; but if a town's streets would give more than `TOWN_GIVE`,
   * they give that at most, and those country roads come after them), then each other country road
   * on its own (`withinRoad`: held where it meets a way solved already,
   * climbing between two such as steeply as it must); each junction at one height once the first
   * road through it is solved.
   */
  private tiered(): void {
    const { profiles, nodes } = this;
    const was = profiles.map((p) => p.y.slice());
    this.apart = false;
    this.withinGrades((r) => this.tier(r) === 1);
    // (Towns further apart in height than the country roads between them can climb: no town's streets dragged more
    // than `TOWN_GIVE` off their land for it; the country roads between them solved on their own after them.)
    const town = (r: number) => this.roads[r].kind !== 'country' && !this.draped(r);
    if (profiles.some((p, r) => town(r) && p.y.some((y, i) => Math.abs(y - was[r][i]) > TOWN_GIVE))) {
      this.apart = true;
      profiles.forEach((p, r) => {
        if (!town(r)) p.y.set(was[r]);
        else for (let i = 0; i < p.y.length; i++) p.y[i] = was[r][i] + Math.max(-TOWN_GIVE, Math.min(TOWN_GIVE, p.y[i] - was[r][i]));
      });
      this.withinGrades((r) => this.tier(r) === 1);
    }
    const fixed = profiles.map((p, r) => new Uint8Array(p.y.length).fill(this.tier(r) === 1 ? 1 : 0));
    const settled = new Uint8Array(nodes.length);
    const touching: number[][] = profiles.map(() => []);
    nodes.forEach((node, n) => {
      for (const a of node.atts) touching[a.road].push(n);
    });
    /** Node `n` at the height of road `road`'s held samples in it, averaged (`road` -1: its streets' and lanes'). */
    const settle = (n: number, road: number) => {
      const node = nodes[n];
      let [sum, count] = [0, 0];
      for (const a of node.atts) {
        if (a.from < 0 || (road >= 0 ? a.road !== road : this.tier(a.road) !== 1)) continue;
        for (let i = a.from; i <= a.to; i++) sum += profiles[a.road].y[i];
        count += a.to - a.from + 1;
      }
      if (!count) return;
      settled[n] = 1;
      node.y = sum / count;
      for (const a of node.atts) {
        if (a.from < 0) continue;
        profiles[a.road].y.fill(node.y, a.from, a.to + 1);
        fixed[a.road].fill(1, a.from, a.to + 1);
      }
    };
    nodes.forEach((_, n) => settle(n, -1));
    for (const rank of [2]) {
      profiles.forEach((p, r) => {
        if (this.tier(r) !== rank) return;
        withinRoad(p, fixed[r]);
        fixed[r].fill(1);
        for (const n of touching[r]) if (!settled[n]) settle(n, r);
      });
    }
  }

  /**
   * Every road's heights brought within its grade, all together, moved as little as they can be
   * (the most any moves is the least it can be): each sample midway between the highest and the
   * lowest it may stand at given every other's height and the grades between them along the roads,
   * a junction's held samples at one height (`grade-limited envelopes over the network, by
   * Dijkstra: the midpoint of the two is the closest such heights in the max norm).
   */
  private withinGrades(only: (road: number) => boolean = () => true): void {
    const { profiles, nodes } = this;
    const base: number[] = [];
    let total = 0;
    for (const p of profiles) {
      base.push(total);
      total += p.y.length;
    }
    // The network: each sample to the next along its road (its grade times the step apart), each junction's held samples to a hub of its own (free).
    const size = total + nodes.length;
    const pairs: number[] = [];
    profiles.forEach((p, r) => {
      if (only(r)) for (let i = 0; i + 1 < p.y.length; i++) pairs.push(base[r] + i, base[r] + i + 1, p.grade * (p.s[i + 1] - p.s[i]));
    });
    nodes.forEach((node, n) => {
      for (const a of node.atts) if (only(a.road)) for (let i = a.from; i >= 0 && i <= a.to; i++) pairs.push(total + n, base[a.road] + i, 0);
    });
    const edges = adjacency(size, pairs);
    const start = new Float64Array(size);
    profiles.forEach((p, r) => {
      for (let i = 0; i < p.y.length; i++) start[base[r] + i] = Math.max(p.y[i], p.floor[i]);
    });
    const hubOf = (sign: 1 | -1) => {
      nodes.forEach((node, n) => {
        let v = sign * Infinity;
        for (const a of node.atts) for (let i = a.from; i >= 0 && i <= a.to && only(a.road); i++) v = sign > 0 ? Math.min(v, start[base[a.road] + i]) : Math.max(v, start[base[a.road] + i]);
        start[total + n] = Number.isFinite(v) ? v : 0;
      });
    };
    // The highest each may stand: the least of every other's height plus the grades between (and the lowest, the same turned over).
    hubOf(1);
    const upper = envelope(start, edges, 1);
    hubOf(-1);
    const lower = envelope(start, edges, -1);
    profiles.forEach((p, r) => {
      if (only(r)) for (let i = 0; i < p.y.length; i++) p.y[i] = (upper[base[r] + i] + lower[base[r] + i]) / 2;
    });
  }

  /** A node's roads made to meet at one height: the mean of their held samples (a junction's ways for cars', not its trails'), every held sample set to it. */
  private join(node: Node): void {
    const { profiles } = this;
    // (The town's streets set it where they meet it, else the country roads, else the trails.)
    const first = Math.min(...node.atts.map((a) => this.tier(a.road)));
    let [sum, n] = [0, 0];
    for (const a of node.atts.filter((o) => this.tier(o.road) === first)) {
      if (a.from < 0) continue;
      const y = profiles[a.road].y;
      for (let i = a.from; i <= a.to; i++) sum += y[i];
      n += a.to - a.from + 1;
    }
    node.y = n ? sum / n : 0;
    for (const a of node.atts) if (a.from >= 0) profiles[a.road].y.fill(node.y, a.from, a.to + 1);
  }

  /** The built land on the samples, and the land graded round it. */
  private solveLand(): void {
    const { cols, rows, land, builtY, kind, roads, plots, pads } = this;
    builtY.fill(NaN);
    kind.fill(0);
    const near = new Float32Array(cols * rows).fill(Infinity);
    // (The least the land lies under a deck carried on over dry land: `DECK_UNDER`.)
    const ceiling = new Float32Array(cols * rows).fill(Infinity);
    const c = LAND_CELL;
    // Roads, out to their sidewalks' edges (where two overlap, the one whose middle is nearer); a bridge's deck is not on the land.
    roads.forEach((r, road) => {
      if (this.draped(road)) return;
      const ls = this.lens[road];
      const half = r.width / 2;
      // (Beside it out to a cell's diagonal past its edge: every corner of a cell its edge crosses.)
      const reach = half + c * Math.SQRT2;
      const spans = bridgeSpans(r);
      // Each deck's water along it: its dry reach past that is further than `DECK_DRY` m from it.
      const waters = spans.map((span): [number, number] => {
        let [from, to] = [Infinity, -Infinity];
        for (let s = span[0]; s <= span[1]; s += 1) {
          const { p } = pointAt(r.points, s, ls);
          const k = Math.min(rows - 1, Math.max(0, Math.round(p[1] / c))) * cols + Math.min(cols - 1, Math.max(0, Math.round(p[0] / c)));
          if (wet(land, k)) [from, to] = [Math.min(from, s), Math.max(to, s)];
        }
        return from <= to ? [from - DECK_DRY, to + DECK_DRY] : [-Infinity, Infinity];
      });
      for (let k = 0; k + 1 < r.points.length; k++) {
        const a = r.points[k];
        const b = r.points[k + 1];
        const ax = a[0];
        const az = a[1];
        const dx = b[0] - ax;
        const dz = b[1] - az;
        const l2 = dx * dx + dz * dz;
        const l = Math.sqrt(l2);
        const i0 = Math.max(0, Math.ceil((Math.min(ax, b[0]) - reach) / c));
        const i1 = Math.min(cols - 1, Math.floor((Math.max(ax, b[0]) + reach) / c));
        const j0 = Math.max(0, Math.ceil((Math.min(az, b[1]) - reach) / c));
        const j1 = Math.min(rows - 1, Math.floor((Math.max(az, b[1]) + reach) / c));
        for (let j = j0; j <= j1; j++) {
          for (let i = i0; i <= i1; i++) {
            const px = i * c - ax;
            const pz = j * c - az;
            const t = l2 > 0 ? Math.max(0, Math.min(1, (px * dx + pz * dz) / l2)) : 0;
            const ox = px - dx * t;
            const oz = pz - dz * t;
            const d = Math.sqrt(ox * ox + oz * oz);
            const at = j * cols + i;
            // (Under it; or beside it, at its height, the samples past its edge round the cells it crosses: the land
            // drawn between the samples meets it under its edge, not over it where the land beside stands higher (a
            // cut), and a narrow way's middle reads it back. Beside it, never in place of a road's own sample or a
            // deck's, nor of the nearer road's beside it.)
            const under = d <= half;
            if (under) {
              if (d >= near[at] && kind[at] !== BESIDE) continue;
            } else if (d > reach || (near[at] < Infinity && kind[at] !== BESIDE) || d >= near[at]) continue;
            const s = ls[k] + t * l;
            if (spans.some((span) => s >= span[0] && s <= span[1])) {
              if (d <= half + DECK_APRON && spans.some((span, n) => s >= span[0] && s <= span[1] && (s < waters[n][0] || s > waters[n][1]))) {
                ceiling[at] = Math.min(ceiling[at], this.roadY(road, s) - DECK_UNDER);
              }
              if (!under) continue;
              near[at] = d;
              builtY[at] = NaN;
              kind[at] = 0;
              continue;
            }
            near[at] = d;
            builtY[at] = this.roadY(road, s);
            kind[at] = under ? 1 : BESIDE;
          }
        }
      }
    });
    // Lots (the higher pad where two touch); a turned backdrop building's on its footprint and round it, out over
    // what it stands on (`GROUND_ROUND`) and a cell's diagonal past that (every corner of a cell its edge crosses,
    // as beside a road: the land drawn between the samples meets its forecourt under its edge), but not under a road,
    // and past its footprint not in a box nor beside one whose pad is more than a retaining edge from its own (a lot's
    // pad is its own and the land beside it within `RETAIN_MAX` of it: its ground gives way to it as to a road, and
    // a lot's own samples are its pad's); one standing in water none (`afloat`: its pad the water's bed, it would dig the banks round it out to that).
    const boxed = new Uint8Array(cols * rows);
    /** Whether sample (i, j) is in a box (a lot's, or a backdrop building's along x and z), or beside one whose pad is more than a retaining edge from `y`. */
    const byBox = (i: number, j: number, y: number) => {
      if (boxed[j * cols + i]) return true;
      for (let nj = Math.max(0, j - 1); nj <= Math.min(rows - 1, j + 1); nj++) {
        for (let ni = Math.max(0, i - 1); ni <= Math.min(cols - 1, i + 1); ni++) if (boxed[nj * cols + ni] && Math.abs(builtY[nj * cols + ni] - y) > RETAIN_MAX) return true;
      }
      return false;
    };
    plots.forEach((p, n) => {
      if (this.afloat[n]) return;
      const out = p.poly ? GROUND_ROUND + c * Math.SQRT2 : 0;
      for (let j = Math.max(0, Math.ceil((p.z - out) / c)); j <= Math.min(rows - 1, Math.floor((p.z + p.d + out) / c)); j++) {
        for (let i = Math.max(0, Math.ceil((p.x - out) / c)); i <= Math.min(cols - 1, Math.floor((p.x + p.w + out) / c)); i++) {
          const at = j * cols + i;
          // (Beside a road, only where its pad is no higher: the land meets the sidewalk's edge under it, and its ground's.)
          if (p.poly && (boxed[at] || kind[at] === 1 || (kind[at] === BESIDE && pads[n] > builtY[at]) || !onPad(p, i * c, j * c, out) || (!onPad(p, i * c, j * c) && byBox(i, j, pads[n])))) continue;
          builtY[at] = kind[at] === 2 ? Math.max(builtY[at], pads[n]) : pads[n];
          kind[at] = 2;
          if (!p.poly) boxed[at] = 1;
        }
      }
    });
    // Round a lot (not a turned backdrop building's: its ground reaches out already), the samples a cell or less
    // outside its box at its pad: the nearest lot's (the higher of two as near), on bare dry land within `SKIRT_MOST`
    // of that pad, not a road's nor beside one.
    const skirt = new Float32Array(cols * rows).fill(Infinity);
    const skirtY = new Float32Array(cols * rows);
    plots.forEach((p, n) => {
      if (p.poly || this.afloat[n]) return;
      const [i0, i1] = [Math.max(0, Math.floor((p.x - c) / c) + 1), Math.min(cols - 1, Math.ceil((p.x + p.w + c) / c) - 1)];
      const [j0, j1] = [Math.max(0, Math.floor((p.z - c) / c) + 1), Math.min(rows - 1, Math.ceil((p.z + p.d + c) / c) - 1)];
      for (let j = j0; j <= j1; j++) {
        for (let i = i0; i <= i1; i++) {
          const at = j * cols + i;
          if (boxed[at] || kind[at] || wet(land, at)) continue;
          const d = Math.hypot(Math.max(p.x - i * c, 0, i * c - p.x - p.w), Math.max(p.z - j * c, 0, j * c - p.z - p.d));
          if (d < skirt[at] - 1e-6 || (d <= skirt[at] + 1e-6 && pads[n] > skirtY[at])) {
            skirt[at] = d;
            skirtY[at] = pads[n];
          }
        }
      }
    });
    for (let k = 0; k < skirt.length; k++) {
      if (skirt[k] === Infinity || Math.abs(land.h[k] / 100 - skirtY[k]) > SKIRT_MOST) continue;
      builtY[k] = skirtY[k];
      kind[k] = SKIRT;
    }
    // The two cones from what is built, `bank` steep (two chamfer passes each way).
    const hi = new Float32Array(cols * rows);
    const lo = new Float32Array(cols * rows);
    for (let k = 0; k < hi.length; k++) {
      const b = kind[k] && !wet(land, k);
      hi[k] = b ? builtY[k] : Infinity;
      lo[k] = b ? builtY[k] : -Infinity;
    }
    cones(hi, lo, cols, rows, this.bank * c, this.bank * c * Math.SQRT2);
    // Beside a lot, no further from its pad than a retaining edge stands.
    for (let j = 0; j < rows; j++) {
      for (let i = 0; i < cols; i++) {
        const k = j * cols + i;
        if (kind[k] !== 2) continue;
        for (let dj = -1; dj <= 1; dj++) {
          for (let di = -1; di <= 1; di++) {
            const ni = i + di;
            const nj = j + dj;
            if (ni < 0 || nj < 0 || ni >= cols || nj >= rows) continue;
            const n = nj * cols + ni;
            if (kind[n]) continue;
            hi[n] = Math.min(hi[n], builtY[k] + RETAIN_MAX);
            lo[n] = Math.max(lo[n], builtY[k] - RETAIN_MAX);
          }
        }
      }
    }
    const out = this.conformed;
    const held = this.held;
    const { h: heights, wl } = land;
    for (let k = 0; k < out.length; k++) {
      const h = heights[k] / 100;
      if (wl[k] !== NONE && wl[k] > heights[k]) out[k] = h;
      else if (kind[k] === 1) out[k] = builtY[k] - ROAD_SINK;
      else if (kind[k]) out[k] = builtY[k];
      else if (lo[k] > hi[k]) out[k] = (lo[k] + hi[k]) / 2;
      else out[k] = Math.min(hi[k], Math.max(lo[k], h));
      // (Under a deck's dry reach, under its stone; a sample beside the road it leaves too. Drawn so, not stored: `graded`.)
      held[k] = NaN;
      if ((kind[k] === 0 || kind[k] === BESIDE) && out[k] > ceiling[k] && !wet(land, k)) {
        held[k] = out[k];
        out[k] = ceiling[k];
      }
    }
  }
}

/**
 * Two chamfer passes each way over a `cols` × `rows` grid: `hi` the least of every sample's plus
 * `side` a step across or along (`diag` a step aslant) to here, `lo` the greatest less as much.
 */
function cones(hi: Float32Array, lo: Float32Array, cols: number, rows: number, side: number, diag: number): void {
  const relax = (k: number, n: number, cost: number) => {
    const up = hi[n] + cost;
    if (up < hi[k]) hi[k] = up;
    const down = lo[n] - cost;
    if (down > lo[k]) lo[k] = down;
  };
  for (let j = 0; j < rows; j++) {
    const row = j * cols;
    for (let i = 0; i < cols; i++) {
      const k = row + i;
      if (i > 0) relax(k, k - 1, side);
      if (j > 0) {
        relax(k, k - cols, side);
        if (i > 0) relax(k, k - cols - 1, diag);
        if (i < cols - 1) relax(k, k - cols + 1, diag);
      }
    }
  }
  for (let j = rows - 1; j >= 0; j--) {
    const row = j * cols;
    for (let i = cols - 1; i >= 0; i--) {
      const k = row + i;
      if (i < cols - 1) relax(k, k + 1, side);
      if (j < rows - 1) {
        relax(k, k + cols, side);
        if (i < cols - 1) relax(k, k + cols + 1, diag);
        if (i > 0) relax(k, k + cols - 1, diag);
      }
    }
  }
}

/** A sample beside a road (a corner of a cell its edge crosses): graded to the road's height, not sunk (`solveLand`). */
const BESIDE = 3;
/** A sample round a lot (a corner of a cell its edge crosses): graded to its pad (`solveLand`, `SKIRT_MOST`). */
const SKIRT = 4;

/** [`from`, `to`] narrowed to [`lo`, `hi`] where they overlap (else as it was). */
function narrowed(from: number, to: number, lo: number, hi: number): [number, number] {
  const [a, b] = [Math.max(from, lo), Math.min(to, hi)];
  return a <= b ? [a, b] : [from, to];
}

/** The land settled under what is built this many times at most after a stroke (`settleCuts`), this many samples round what changed. */
const SETTLE_PASSES = 3;
const SETTLE_MARGIN = 6;
/** How far within its limit (a share of it) a sample past it is brought back. */
const SETTLE_SHORT = 0.4;
/** A sample's eight neighbours. */
const NEAR8: readonly [number, number][] = [[-1, -1], [0, -1], [1, -1], [-1, 0], [1, 0], [-1, 1], [0, 1], [1, 1]];

/** Lots are filed for `at` by cells this wide (m). */
const LOT_CELL = 16;

const cm = (m: number) => Math.max(-32767, Math.min(32767, Math.round(m * 100)));

/** Whether sample `k` is under water. */
const wet = (land: Land, k: number) => land.wl[k] !== NONE && land.wl[k] > land.h[k];

/** The land's surface (m) at (`x`, `z`) for a road's first height: its ground, or `BRIDGE_CLEAR` over the water where it is wet (a bridge's approaches climb to its deck), between the samples round it. */
function surfaceAt(land: Land, x: number, z: number): number {
  const { cols, rows, h, wl } = land;
  const fx = Math.min(cols - 1, Math.max(0, x / LAND_CELL));
  const fz = Math.min(rows - 1, Math.max(0, z / LAND_CELL));
  const i = Math.min(Math.floor(fx), Math.max(0, cols - 2));
  const j = Math.min(Math.floor(fz), Math.max(0, rows - 2));
  const [tx, tz] = [fx - i, fz - j];
  const [i1, j1] = [Math.min(i + 1, cols - 1), Math.min(j + 1, rows - 1)];
  const v = (k: number) => (wl[k] !== NONE && wl[k] > h[k] ? wl[k] + BRIDGE_CLEAR * 100 : h[k]);
  const top = v(j * cols + i) * (1 - tx) + v(j * cols + i1) * tx;
  const bottom = v(j1 * cols + i) * (1 - tx) + v(j1 * cols + i1) * tx;
  return (top * (1 - tz) + bottom * tz) / 100;
}

/** The highest water (m) under (`x`, `z`) among the four samples round it; -Infinity if none is wet. */
function waterAt(land: Land, x: number, z: number): number {
  const { cols, rows } = land;
  const [i, j] = [Math.floor(x / LAND_CELL), Math.floor(z / LAND_CELL)];
  let top = -Infinity;
  for (const [a, b] of [
    [i, j],
    [i + 1, j],
    [i, j + 1],
    [i + 1, j + 1],
  ]) {
    if (a < 0 || b < 0 || a >= cols || b >= rows) continue;
    const k = b * cols + a;
    if (wet(land, k)) top = Math.max(top, land.wl[k] / 100);
  }
  return top;
}

/** A network's links (`pairs`: a, b, cost, ...) by node: each node's neighbours from `first[v]` to `first[v + 1]` in `to`, at `cost`. */
interface Adjacency {
  first: Int32Array;
  to: Int32Array;
  cost: Float64Array;
}

function adjacency(size: number, pairs: readonly number[]): Adjacency {
  const first = new Int32Array(size + 1);
  for (let k = 0; k < pairs.length; k += 3) {
    first[pairs[k] + 1]++;
    first[pairs[k + 1] + 1]++;
  }
  for (let v = 0; v < size; v++) first[v + 1] += first[v];
  const fill = first.slice(0, size);
  const to = new Int32Array(first[size]);
  const cost = new Float64Array(first[size]);
  for (let k = 0; k < pairs.length; k += 3) {
    const [a, b, w] = [pairs[k], pairs[k + 1], pairs[k + 2]];
    to[fill[a]] = b;
    cost[fill[a]++] = w;
    to[fill[b]] = a;
    cost[fill[b]++] = w;
  }
  return { first, to, cost };
}

/**
 * One road brought within its grade, moved as little as it can be (the midpoint of its own
 * grade-limited envelopes, a deck no lower than its floor), and held to the samples `fixed` (where
 * it meets roads solved already): no further from them than its grade allows. Where two of those
 * stand further apart than its grade can climb, it climbs between them as steeply as it must.
 */
function withinRoad(p: Profile, fixed: Uint8Array): void {
  const { y, s, floor } = p;
  const n = y.length;
  if (!n) return;
  let grade = p.grade;
  for (let i = 0, last = -1; i < n; i++) {
    if (!fixed[i]) continue;
    if (last >= 0 && s[i] > s[last]) grade = Math.max(grade, Math.abs(y[i] - y[last]) / (s[i] - s[last]));
    last = i;
  }
  const up = new Float64Array(n);
  const down = new Float64Array(n);
  const hi = new Float64Array(n);
  const lo = new Float64Array(n);
  for (let i = 0; i < n; i++) {
    up[i] = down[i] = Math.max(y[i], floor[i]);
    hi[i] = fixed[i] ? y[i] : Infinity;
    lo[i] = fixed[i] ? y[i] : -Infinity;
  }
  const sweep = (i: number, j: number) => {
    const g = grade * Math.abs(s[i] - s[j]);
    up[i] = Math.min(up[i], up[j] + g);
    down[i] = Math.max(down[i], down[j] - g);
    hi[i] = Math.min(hi[i], hi[j] + g);
    lo[i] = Math.max(lo[i], lo[j] - g);
  };
  for (let i = 1; i < n; i++) sweep(i, i - 1);
  for (let i = n - 2; i >= 0; i--) sweep(i, i + 1);
  for (let i = 0; i < n; i++) {
    if (fixed[i]) continue;
    const mid = (up[i] + down[i]) / 2;
    y[i] = lo[i] <= hi[i] ? Math.min(hi[i], Math.max(lo[i], mid)) : (lo[i] + hi[i]) / 2;
  }
}

/**
 * The grade-limited envelope of `start` over a network: `sign` 1, each node's least
 * `start[u] + cost(u, v)` over every `u` (the highest it may stand); -1, the greatest
 * `start[u] - cost(u, v)` (the lowest). Dijkstra from every node at once.
 */
function envelope(start: Float64Array, edges: Adjacency, sign: 1 | -1): Float64Array {
  const n = start.length;
  const out = Float64Array.from(start, (v) => sign * v);
  const done = new Uint8Array(n);
  const heap = new MinHeap(n);
  for (let v = 0; v < n; v++) heap.push(v, out[v]);
  const { first, to, cost } = edges;
  while (heap.size) {
    const v = heap.pop();
    if (done[v]) continue;
    done[v] = 1;
    for (let k = first[v]; k < first[v + 1]; k++) {
      const u = to[k];
      const via = out[v] + cost[k];
      if (done[u] || via >= out[u]) continue;
      out[u] = via;
      heap.push(u, via);
    }
  }
  for (let v = 0; v < n; v++) out[v] *= sign;
  return out;
}

/** A binary heap of indices by key (an index may be pushed again with a smaller key; the stale entry is skipped by its taker). */
class MinHeap {
  private items: number[] = [];
  private keys: number[] = [];
  constructor(_hint: number) {}
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

/** `values` along a road (at arc lengths `s`), each averaged with those within `PROFILE_REACH` m of it. */
function averaged(s: Float32Array, values: Float32Array): Float32Array {
  const n = values.length;
  const out = new Float32Array(n);
  let [a, b, sum] = [0, 0, 0];
  for (let i = 0; i < n; i++) {
    while (b < n && s[b] <= s[i] + PROFILE_REACH) sum += values[b++];
    while (s[a] < s[i] - PROFILE_REACH) sum -= values[a++];
    out[i] = sum / (b - a);
  }
  return out;
}

/** A polyline's samples every `PROFILE_STEP` m from its start, its end last (`ls`: its arc lengths, `lengths`), as a profile of `grade`. */
function sampleLine(points: readonly (readonly [number, number])[], ls: readonly number[], grade: number): Profile {
  const len = ls[ls.length - 1] ?? 0;
  const n = points.length < 2 ? (points.length ? 1 : 0) : Math.ceil(len / PROFILE_STEP - 1e-6) + 1;
  const s = new Float32Array(n);
  const x = new Float32Array(n);
  const z = new Float32Array(n);
  let k = 0;
  for (let i = 0; i < n; i++) {
    const at = i === n - 1 ? len : i * PROFILE_STEP;
    while (k < ls.length - 2 && ls[k + 1] < at) k++;
    const [a, b] = [points[k], points[k + 1] ?? points[k]];
    const f = Math.max(0, Math.min(1, (at - ls[k]) / (ls[k + 1] - ls[k] || 1)));
    s[i] = at;
    x[i] = a[0] + (b[0] - a[0]) * f;
    z[i] = a[1] + (b[1] - a[1]) * f;
  }
  return { len, s, x, z, ground: new Float32Array(n), y: new Float32Array(n), floor: new Float32Array(n).fill(-Infinity), pin: new Uint8Array(n), grade };
}

/**
 * A road's first height (into `ground` and `y`): the land along its middle line (the water's top where it is wet),
 * averaged over `PROFILE_REACH` m either way. (Twice: what the first average left out, averaged and put back, so a
 * crest or a hollow along a road already smooth keeps its height, and the land graded to its roads gives the same
 * roads again.)
 */
function groundOf(p: Profile, land: Land): void {
  const n = p.y.length;
  const raw = new Float32Array(n);
  for (let i = 0; i < n; i++) raw[i] = surfaceAt(land, p.x[i], p.z[i]);
  const mean = averaged(p.s, raw);
  const rest = averaged(p.s, raw.map((v, i) => v - mean[i]));
  for (let i = 0; i < n; i++) p.ground[i] = mean[i] + rest[i];
  p.y.set(p.ground);
}

/** One of the solver's passes over a road on its own: smoothed where it bends, its deck up to its floor, its grade clamped both ways. */
function settlePass(p: Profile, scratch: Float32Array): void {
  smooth(p, scratch);
  lift(p);
  clampGrade(p, 1);
  clampGrade(p, -1);
}

/**
 * Three-tap smoothing of a road's heights where it bends (its grade changing by more than `BEND`
 * from one sample to the next; not where a junction holds it, nor at its ends): the kinks the
 * grades and the junctions leave rounded off, a road already smooth left as it is (so a land
 * graded to its roads gives the same roads again).
 */
function smooth(p: Profile, scratch: Float32Array): void {
  const n = p.y.length;
  if (n < 3) return;
  scratch.set(p.y.subarray(0, n));
  for (let i = 1; i < n - 1; i++) {
    if (p.pin[i] || Math.abs(scratch[i - 1] - 2 * scratch[i] + scratch[i + 1]) <= BEND) continue;
    p.y[i] = (scratch[i - 1] + 2 * scratch[i] + scratch[i + 1]) / 4;
  }
}

/** A bridge's deck up to its floor. */
function lift(p: Profile): void {
  for (let i = 0; i < p.y.length; i++) if (p.y[i] < p.floor[i]) p.y[i] = p.floor[i];
}

/**
 * A road's grade clamped to its kind's, sweeping along it (`dir` 1 from its start, -1 from its
 * end): a stretch too steep has both its ends moved towards each other, half each, but a deck not
 * below its floor (the other takes the rest).
 */
function clampGrade(p: Profile, dir: 1 | -1): void {
  const { y, s, floor, grade } = p;
  const n = y.length;
  for (let k = 0; k + 1 < n; k++) {
    const a = dir > 0 ? k : n - 1 - k;
    const b = a + dir;
    const most = grade * Math.abs(s[b] - s[a]);
    const d = y[b] - y[a];
    if (Math.abs(d) <= most) continue;
    const excess = Math.abs(d) - most;
    // (The higher one comes down, the lower one goes up.)
    const [up, down] = d > 0 ? [a, b] : [b, a];
    const lower = Math.min(Math.max(0, y[down] - floor[down]), excess / 2);
    const raise = excess - lower;
    y[down] -= lower;
    y[up] += raise;
  }
}

// ---- what a world gives it ----------------------------------------------------------------

/** What `surfacePlots` and `builtSurfaceOf` read of a world (a `WorldStructure`, a town file, a menu's preview). */
export interface SurfaceWorld {
  plots: readonly { x: number; z: number; w: number; d: number }[];
  meta?: {
    roads?: RoadPlan[];
    paths?: { x: number; z: number; w: number; d: number }[];
    facing?: number[];
    backdrop?: { poly: [number, number][] }[];
    world?: { landscape?: { slope?: number } };
  } | null;
  junctions?: JunctionInfo[];
}

/** How far out from a lot's front (m) its way in is looked for, without a path to show it. */
const ENTRY_REACH = 16;
/** A backdrop building takes the height of a road this near (m). */
const BACKDROP_REACH = 30;
/** How steeply the land may rise from what is built, in a world that doesn't say (the renderer's `RELIEF_SLOPE`). */
const DEFAULT_BANK = 0.5;

/**
 * A world's lots as the surface takes them, in its plots' order, then its backdrop's buildings
 * (each its footprint's box): where each lot's way in meets its road, and the stretch of that road
 * its front faces. That is the far end of the
 * widest of its paths across the verge from its front (a driveway's, else its front path's: the
 * town's `meta.paths` that touch its front edge from outside), else the first road straight out
 * from its front's middle within `ENTRY_REACH` m, else none. A backdrop building's is the road
 * nearest it within `BACKDROP_REACH` m (not a trail).
 */
export function surfacePlots(world: SurfaceWorld): SurfacePlot[] {
  const roads = world.meta?.roads ?? [];
  const index = new RoadIndex(roads);
  const paths = world.meta?.paths ?? [];
  // (The paths filed by cell, for those touching a lot's front.)
  const cells = new Map<number, number[]>();
  paths.forEach((r, n) => {
    for (let cz = Math.floor(r.z / LOT_CELL); cz <= Math.floor((r.z + r.d) / LOT_CELL); cz++) {
      for (let cx = Math.floor(r.x / LOT_CELL); cx <= Math.floor((r.x + r.w) / LOT_CELL); cx++) {
        const key = cx * 65536 + cz;
        const list = cells.get(key);
        if (list) list.push(n);
        else cells.set(key, [n]);
      }
    }
  });
  const widest = Math.max(0, ...roads.map((r) => r.width));
  const onRoad = (x: number, z: number, within = 0) => {
    let best: { road: number; s: number; dist: number } | null = null;
    for (const [road, m] of index.around(x, z, widest / 2 + within)) {
      if (m.dist > roads[road].width / 2 + within || (best && best.dist <= m.dist)) continue;
      best = { road, s: m.s, dist: m.dist };
    }
    return best && { road: best.road, s: best.s };
  };
  const out: SurfacePlot[] = world.plots.map((p, i) => {
    const facing = world.meta?.facing?.[i] ?? frontOf(p, onRoad);
    // Out from its front: which way, and its front edge's line.
    const [nx, nz] = OUT[facing & 3];
    const edge = nz < 0 ? p.z : nz > 0 ? p.z + p.d : nx < 0 ? p.x : p.x + p.w;
    const across = nz !== 0;
    const lo = across ? p.x : p.z;
    const hi = across ? p.x + p.w : p.z + p.d;
    let way: { r: (typeof paths)[number]; wide: number } | null = null;
    const near = new Set<number>();
    const [ex0, ez0, ex1, ez1] = across ? [lo, edge, hi, edge] : [edge, lo, edge, hi];
    for (let cz = Math.floor((ez0 - 0.1) / LOT_CELL); cz <= Math.floor((ez1 + 0.1) / LOT_CELL); cz++) {
      for (let cx = Math.floor((ex0 - 0.1) / LOT_CELL); cx <= Math.floor((ex1 + 0.1) / LOT_CELL); cx++) for (const n of cells.get(cx * 65536 + cz) ?? []) near.add(n);
    }
    for (const r of [...near].sort((a, b) => a - b).map((n) => paths[n])) {
      const side = nz < 0 ? r.z + r.d : nz > 0 ? r.z : nx < 0 ? r.x + r.w : r.x;
      if (Math.abs(side - edge) > 0.01) continue;
      const [a, b] = across ? [r.x, r.x + r.w] : [r.z, r.z + r.d];
      if (a < lo - 0.01 || b > hi + 0.01) continue;
      const wide = b - a;
      if (!way || wide > way.wide) way = { r, wide };
    }
    let entry: SurfacePlot['entry'] = null;
    if (way) {
      const r = way.r;
      const far = nz < 0 ? r.z : nz > 0 ? r.z + r.d : nx < 0 ? r.x : r.x + r.w;
      const mid = across ? r.x + r.w / 2 : r.z + r.d / 2;
      const [x, z] = across ? [mid, far] : [far, mid];
      entry = onRoad(x + nx * 0.5, z + nz * 0.5, 0.5);
    }
    if (!entry) {
      const mid = (lo + hi) / 2;
      for (let k = 0.5; k <= ENTRY_REACH && !entry; k += 0.5) {
        const [x, z] = across ? [mid, edge + nz * k] : [edge + nx * k, mid];
        entry = onRoad(x, z);
      }
    }
    if (!entry) return { x: p.x, z: p.z, w: p.w, d: p.d, entry };
    // The stretch of its road its front faces: from where each front corner is nearest it.
    const road = entry.road;
    const ends = (across ? [[lo, edge], [hi, edge]] : [[edge, lo], [edge, hi]]).map(([x, z]) => index.along(road, x, z));
    return { x: p.x, z: p.z, w: p.w, d: p.d, entry, front: [Math.min(...ends), Math.max(...ends)] };
  });
  for (const b of world.meta?.backdrop ?? []) {
    // (A turned one's pad is its footprint, not its box: a box along x and z is its footprint already.)
    const turned = b.poly.length >= 3 && b.poly.some((q, k) => {
      const n = b.poly[(k + 1) % b.poly.length];
      return Math.abs(n[0] - q[0]) > 0.01 && Math.abs(n[1] - q[1]) > 0.01;
    });
    const poly = turned ? b.poly : null;
    const xs = b.poly.map((q) => q[0]);
    const zs = b.poly.map((q) => q[1]);
    const [x0, z0, x1, z1] = [Math.min(...xs), Math.min(...zs), Math.max(...xs), Math.max(...zs)];
    let entry: SurfacePlot['entry'] = null;
    const [cx, cz] = [(x0 + x1) / 2, (z0 + z1) / 2];
    // (The nearest road, unless it is a trail: then the nearest that isn't.)
    const reach = Math.max(x1 - x0, z1 - z0) / 2;
    const first = index.nearest(cx, cz, reach + 6) ?? index.nearest(cx, cz, BACKDROP_REACH);
    if (first && roads[first.road].kind !== 'trail') entry = { road: first.road, s: first.s };
    else if (first) {
      let far = BACKDROP_REACH;
      for (const [road, m] of index.around(cx, cz, BACKDROP_REACH)) {
        if (roads[road].kind === 'trail' || m.dist >= far) continue;
        far = m.dist;
        entry = { road, s: m.s };
      }
    }
    out.push({ x: x0, z: z0, w: x1 - x0, d: z1 - z0, entry, ...(poly ? { poly } : {}) });
  }
  return out;
}

/** Out from a lot's front, by `Facing` (0: its street to smaller z, 1: smaller x, 2: larger z, 3: larger x). */
const OUT: readonly [number, number][] = [
  [0, -1],
  [-1, 0],
  [0, 1],
  [1, 0],
];

/** A lot's front where its world doesn't say (older saves): the side its nearest road is on. */
function frontOf(p: { x: number; z: number; w: number; d: number }, onRoad: (x: number, z: number, within?: number) => unknown): number {
  const mids: [number, number][] = [
    [p.x + p.w / 2, p.z - 1],
    [p.x - 1, p.z + p.d / 2],
    [p.x + p.w / 2, p.z + p.d + 1],
    [p.x + p.w + 1, p.z + p.d / 2],
  ];
  for (let k = 1; k <= ENTRY_REACH; k++) {
    for (let f = 0; f < 4; f++) {
      const [x, z] = [mids[f][0] + OUT[f][0] * (k - 1), mids[f][1] + OUT[f][1] * (k - 1)];
      if (onRoad(x, z)) return f;
    }
  }
  return 0;
}

/** How steeply a world's land may rise from what is built (`LandscapeParams.slope`): the surface's `bank`. */
export function bankOf(world: SurfaceWorld): number {
  return world.meta?.world?.landscape?.slope ?? DEFAULT_BANK;
}

/**
 * The surface of a world's built land on `land` (its `meta.land`, decoded): its roads, its lots
 * and backdrop (`surfacePlots`), the junctions the sim reports (`world.junctions`, none in a menu's
 * preview), and its `bank` (`bankOf`). How the renderer makes one when a world loads, and the
 * generator when it grades a new world's land.
 */
export function builtSurfaceOf(world: SurfaceWorld, land: Land): BuiltSurface {
  const roads = world.meta?.roads ?? [];
  return new BuiltSurface(land, roads, surfacePlots(world), planJunctions(world.junctions ?? [], roads), bankOf(world));
}

/** A road's profile as `profileOf` gives it: per sample, how far along (m), where, its height and its cut or fill. */
export interface RoadProfile {
  s: number[];
  x: number[];
  z: number[];
  y: number[];
  /**
   * How far the road there is cut into the land or stands over it (m, positive over it): its height less the land's
   * (`heightAt`). 0 over water, where it would be a bridge, and along a trail, which lies on the land.
   */
  cut: number[];
}

/**
 * The height a road of `kind` drawn along `points` would have on `land`, every `PROFILE_STEP` m along it (`s`, m):
 * what the surface's solver (`BuiltSurface`) gives a road of that kind on its own, its first height the land
 * averaged over `PROFILE_REACH` m either way (`groundOf`), brought within the kind's `GRADE` as little as it can be
 * (`withinRoad`), then the same passes of smoothing and clamping (`settlePass`). `ends`: the heights its first and
 * last samples are held at (the road it joins there, as a junction holds it), if any. `cut` is how far that is cut
 * into the land or stands over it (the street tool tints a stretch past `CUT_MAX`; among mountains a country road
 * may go to `CUT_MAX_MOUNTAIN_ROAD`, as `measureLand` judges it). A trail lies on the land as it is.
 *
 * Cheap enough for every pointer move (a few hundred samples for a long road). The settle has the last word: it
 * solves the road with the rest of the network, so a junction in the middle of it, or a road it is carried on
 * from, can move it a little.
 */
export function profileOf(points: readonly (readonly [number, number])[], kind: RoadKind, land: Land, ends?: { start?: number; end?: number }): RoadProfile {
  const p = sampleLine(points, lengths(points as [number, number][]), GRADE[kind]);
  const n = p.y.length;
  const out: RoadProfile = { s: Array.from(p.s), x: Array.from(p.x), z: Array.from(p.z), y: new Array<number>(n), cut: new Array<number>(n).fill(0) };
  if (kind === 'trail') {
    for (let i = 0; i < n; i++) out.y[i] = heightAt(land, p.x[i], p.z[i]);
    return out;
  }
  groundOf(p, land);
  // (Held at its ends as a junction holds a road's samples there: not smoothed, set back after each pass.)
  const fixed = new Uint8Array(n);
  const held: [number, number][] = [];
  if (n && ends?.start !== undefined) held.push([0, ends.start]);
  if (n > 1 && ends?.end !== undefined) held.push([n - 1, ends.end]);
  const hold = () => {
    for (const [i, y] of held) p.y[i] = y;
  };
  for (const [i] of held) fixed[i] = p.pin[i] = 1;
  hold();
  withinRoad(p, fixed);
  const scratch = new Float32Array(n);
  for (let pass = 0; pass < PASSES; pass++) {
    settlePass(p, scratch);
    hold();
    lift(p);
  }
  withinRoad(p, fixed);
  lift(p);
  for (let i = 0; i < n; i++) {
    out.y[i] = p.y[i];
    const [ci, cj] = [Math.min(land.cols - 1, Math.max(0, Math.round(p.x[i] / LAND_CELL))), Math.min(land.rows - 1, Math.max(0, Math.round(p.z[i] / LAND_CELL)))];
    out.cut[i] = wet(land, cj * land.cols + ci) ? 0 : p.y[i] - heightAt(land, p.x[i], p.z[i]);
  }
  return out;
}
