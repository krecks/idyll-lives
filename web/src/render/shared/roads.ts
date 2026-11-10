/**
 * A town's roads as they are drawn (engine-neutral): the road surface, and along both sides of a
 * street or lane a sidewalk with its kerb and a gutter's shade, following the road round its bends
 * (a country road has none). Where roads meet (`junctions.ts`) a pad of asphalt joins them with a
 * kerb return round each corner, the sidewalks turning the corners on it, white stop lines and
 * crosswalks on its arms; elsewhere a sidewalk stops at the edge of a road across it. A dead end
 * gets a turning circle with the sidewalk round it. `Kerb` walks a road's side by distance along it, for what
 * stands on the sidewalk (`street.ts`). Where roads are, how far, which is nearest: `game/roads.ts`
 * (`RoadIndex`), as the town was laid out with.
 *
 * Older towns have straight streets along x (`WorldMeta.streets`): drawn as roads the same way
 * (`townRoads`).
 */

import type { RoadPlan, WorldStructure } from '../../core/protocol';
import { rasterRoads, roadEnds, RoadIndex, roadIndex, SIDEWALK, trimJoins } from '../../game/roads';
import { Geo, type V3 } from './geometry';
import { deriveJunctions, hasSidewalks, JunctionCuts, KERB_RETURN, offsetMeet, planJunctions, turnedBack, turnedOn, type CornerPlan, type JunctionPlan } from './junctions';

export { RoadIndex, roadIndex, SIDEWALK };
/** The kerb's width, and the levels of the sidewalk, the kerb and the road. */
export const KERB = 0.14;
export const WALK_Y = 0.03;
export const KERB_Y = 0.045;
export const ROAD_Y = 0.012;
/** How high the road markings lie over the road (m: their material is drawn over it too, `MaterialLibrary.roadPaint`). */
export const MARK_Y = ROAD_Y + 0.004;
/** Road markings (m): a stop line's width, a crosswalk's stripes and the gaps between them, and how far in from the kerbs they keep. */
const STOP_LINE = 0.3;
const STRIPE = 0.5;
const STRIPE_GAP = 0.5;
const MARK_IN = 0.15;
/** The markings' paint: a vertex colour (worn white: no brighter than the sidewalks) with the plain finish's roughness in its alpha (`street.ts` `encodeFinish`: 0.7, not metal). */
const PAINT = '#D6D6D0';
const PAINT_FINISH = 0.35;
/** How far a dead end's turning circle reaches past the lane's kerbs (m). */
const TURNING = 2.4;

type Pt = [number, number];
export interface Box {
  x: number;
  z: number;
  w: number;
  d: number;
}

/** An older town's streets as roads, once per town. */
const streetRoads = new WeakMap<object, RoadPlan[]>();

/** The town's roads: its own, or an older town's straight streets as roads. */
export function townRoads(world: WorldStructure): RoadPlan[] {
  const meta = world.meta;
  if (!meta) return [];
  if (meta.roads) return meta.roads;
  let roads = streetRoads.get(meta);
  if (!roads) {
    roads = (meta.streets ?? []).filter((s) => s.w >= s.d).map((s): RoadPlan => ({ points: [[s.x, s.z + s.d / 2], [s.x + s.w, s.z + s.d / 2]], width: s.d, ends: ['out', 'out'] }));
    streetRoads.set(meta, roads);
  }
  return roads;
}

/** A way out of town: where a road leaves it, and the way out (a unit direction). */
export interface TownExit {
  x: number;
  z: number;
  dx: number;
  dz: number;
}

/**
 * Where the town's roads lead out of it, each with its way out: their `out` ends (an older town:
 * both ends of its middle street, where its country roads begin).
 */
export function townExits(world: WorldStructure): TownExit[] {
  if (!world.meta?.roads) {
    const streets = townRoads(world).sort((a, b) => a.points[0][1] - b.points[0][1]);
    const main = streets[Math.floor((streets.length - 1) / 2)];
    if (!main) return [];
    const z = main.points[0][1];
    return [{ x: 0, z, dx: -1, dz: 0 }, { x: world.width, z, dx: 1, dz: 0 }];
  }
  return roadEnds(world.meta.roads);
}

/** Tiles of a `w` × `d` town a road's surface or sidewalk covers (1), else 0 (each town's roads drawn once). */
const tilesOf = new WeakMap<readonly RoadPlan[], Uint8Array>();
export function roadTiles(roads: readonly RoadPlan[], w: number, d: number): Uint8Array {
  const had = tilesOf.get(roads);
  if (had && had.length === w * d) return had;
  const out = rasterRoads(roads, w, d, (r) => r.width / 2);
  for (const r of roads) {
    // A dead end's turning circle.
    for (const [cx, cz, rr] of deadEnds(r)) {
      for (let z = Math.max(0, Math.floor(cz - rr - SIDEWALK)); z <= Math.min(d - 1, Math.ceil(cz + rr + SIDEWALK)); z++) {
        for (let x = Math.max(0, Math.floor(cx - rr - SIDEWALK)); x <= Math.min(w - 1, Math.ceil(cx + rr + SIDEWALK)); x++) if (Math.hypot(x + 0.5 - cx, z + 0.5 - cz) < rr + SIDEWALK) out[z * w + x] = 1;
      }
    }
  }
  tilesOf.set(roads, out);
  return out;
}

/** A road's points, each with the direction along it and its left (x, z turned a quarter: (z, -x)), mitred at bends. */
interface Frame {
  p: Pt;
  t: Pt;
  /** The left, scaled so offset lines keep their distance through a bend. */
  n: Pt;
  /** Arc length. */
  s: number;
}

function framesOf(road: RoadPlan): Frame[] {
  const pts = road.points;
  const closed = road.ends[0] === 'loop' && pts.length > 2;
  const seg = (i: number): Pt => {
    const [a, b] = [pts[i], pts[i + 1]];
    const l = Math.hypot(b[0] - a[0], b[1] - a[1]) || 1;
    return [(b[0] - a[0]) / l, (b[1] - a[1]) / l];
  };
  let s = 0;
  return pts.map((p, i) => {
    if (i > 0) s += Math.hypot(p[0] - pts[i - 1][0], p[1] - pts[i - 1][1]);
    const last = pts.length - 1;
    const before = i > 0 ? seg(i - 1) : closed ? seg(last - 1) : seg(0);
    const after = i < last ? seg(i) : closed ? seg(0) : seg(last - 1);
    const [tx, tz] = [before[0] + after[0], before[1] + after[1]];
    const l = Math.hypot(tx, tz) || 1;
    const t: Pt = [tx / l, tz / l];
    // The mitre: 1 / cos(half the turn), kept sane at sharp turns.
    const k = Math.min(2, 1 / Math.max(0.5, t[0] * after[0] + t[1] * after[1]));
    return { p, t, n: [t[1] * k, -t[0] * k], s };
  });
}

/**
 * One side of a road, by distance along it (`s`) and in from the sidewalk's outer edge (`d`):
 * where things on the sidewalk stand, and which way they face the road.
 */
export class Kerb {
  readonly length: number;
  private readonly frames: Frame[];
  /** The road's turning circles (`deadEnds`). */
  private readonly ends: [number, number, number][];

  constructor(
    readonly road: RoadPlan,
    readonly index: number,
    /** +1: the road's left (looking along it), -1: its right. */
    readonly side: 1 | -1,
    /** The roads (this one at `index`) whose surfaces stop a sidewalk, country roads among them. */
    private readonly others: RoadIndex,
    /** Where the junctions' corners stop it (default: the junctions of `others`' roads as worked out from them). */
    private readonly cuts: JunctionCuts = defaultCuts(others.roads),
  ) {
    this.frames = framesOf(road);
    this.ends = deadEnds(road);
    this.length = this.frames[this.frames.length - 1].s;
  }

  private frameAt(s: number): { p: Pt; t: Pt } {
    const f = this.frames;
    // The last frame at or before `s` (arc lengths only grow).
    let [lo, hi] = [0, f.length - 2];
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1;
      if (f[mid].s <= s) lo = mid;
      else hi = mid - 1;
    }
    const i = lo;
    const u = Math.max(0, Math.min(1, (s - f[i].s) / (f[i + 1].s - f[i].s || 1)));
    const [a, b] = [f[i].p, f[i + 1].p];
    const [tx, tz] = [b[0] - a[0], b[1] - a[1]];
    const l = Math.hypot(tx, tz) || 1;
    return { p: [a[0] + tx * u, a[1] + tz * u], t: [tx / l, tz / l] };
  }

  /** The point `d` m in from the sidewalk's outer edge (towards the road's middle; less than 0: out on the verge) at `s`. */
  at(s: number, d: number): Pt {
    const { p, t } = this.frameAt(s);
    const off = (this.road.width / 2 - d) * this.side;
    return [p[0] + t[1] * off, p[1] - t[0] * off];
  }

  /** Yaw of a thing at `s` facing the road (its +z towards the road's middle). */
  yaw(s: number): number {
    const { t } = this.frameAt(s);
    // Towards the middle: the right of the way along on the left side, its left on the right.
    const [x, z] = [-t[1] * this.side, t[0] * this.side];
    return Math.atan2(x, z);
  }

  /** Arc length along the road nearest (x, z). */
  along(x: number, z: number): number {
    return this.others.along(this.index, x, z);
  }

  /** Whether the sidewalk runs at `s` (not cut by a road it meets, not in a turning circle). */
  open(s: number): boolean {
    const [x, z] = this.at(s, (SIDEWALK - KERB) / 2);
    return !sidewalkCut(this.others, this.index, this.ends, this.cuts, x, z);
  }

  /** Whether a point is in the turning circle at a dead end of this road. */
  inTurning(x: number, z: number): boolean {
    return inTurning(this.ends, x, z);
  }
}

/** Whether a point is in one of a road's turning circles (`deadEnds`), sidewalk and all (and `grow` m round). */
function inTurning(ends: readonly [number, number, number][], x: number, z: number, grow = 0): boolean {
  return ends.some(([cx, cz, r]) => Math.hypot(x - cx, z - cz) < r + SIDEWALK + grow);
}

/**
 * Whether road `ri`'s sidewalk is cut at a point across its middle: another road runs there (a
 * country road too), a junction's corner turns there (`cuts`), or it is in one of the road's turning
 * circles (`ends`).
 */
function sidewalkCut(index: RoadIndex, ri: number, ends: readonly [number, number, number][], cuts: JunctionCuts, x: number, z: number): boolean {
  return index.onRoad(x, z, 0.2, ri) || cuts.cut(x, z) || inTurning(ends, x, z);
}

/** Each town's roads that stop a sidewalk (`walkStops`), once. */
const stopsOf = new WeakMap<readonly RoadPlan[], RoadPlan[]>();

/**
 * Of a town's roads, those that stop a sidewalk where they meet or cross a street or lane: the
 * streets and lanes (those with sidewalks: the first, in their order), then its country roads, each
 * begun at the kerb of the road it joins (as the lanes are, `trimJoins`): one ending on a street's
 * middle line stops only the sidewalk on its own side. Trails stop none.
 */
export function walkStops(roads: readonly RoadPlan[]): RoadPlan[] {
  let out = stopsOf.get(roads);
  if (!out) {
    const cars = roads.filter((r) => r.kind !== 'trail');
    const trimmed = trimJoins(cars);
    stopsOf.set(roads, (out = [...cars.filter(hasSidewalks), ...trimmed.filter((_, i) => cars[i].kind === 'country')]));
  }
  return out;
}

/** The junctions of a set of roads as worked out from them, as drawn (`junctions.ts`). */
export function roadJunctions(roads: readonly RoadPlan[]): JunctionPlan[] {
  return planJunctions(deriveJunctions(roads), roads);
}

function defaultCuts(roads: readonly RoadPlan[]): JunctionCuts {
  return JunctionCuts.of(roadJunctions(roads));
}

/** A road's turning circles: their middles and radii (to the kerb). */
function deadEnds(road: RoadPlan): [number, number, number][] {
  const r = road.width / 2 - SIDEWALK + TURNING;
  const out: [number, number, number][] = [];
  if (road.ends[1] === 'dead') out.push([...road.points[road.points.length - 1], r]);
  if (road.ends[0] === 'dead') out.push([...road.points[0], r]);
  return out;
}

/** What is drawn of the roads: their surface, and the sidewalks and kerbs (vertex colours) with the gutters' shade; the markings (vertex colours: `PAINT`), if asked for. */
export interface RoadMeshes {
  surface: Geo;
  walk: Geo;
  shadows: Geo;
  marks?: Geo;
}

/**
 * The roads' surfaces, sidewalks, kerbs and gutters, only near `clip` (grown by `margin`) when
 * given: the road surface under the sidewalks too, the sidewalks on top (added to `into`'s meshes if given;
 * `parts`: only the surface, or only the sidewalks, kerbs and gutters). The `junctions` (default: as
 * worked out from `roads`): their pads with the surface, their corners' sidewalks with the sidewalks,
 * their markings with the surface into `into.marks` if given. Only streets and lanes have sidewalks:
 * a country road among `roads` only stops theirs.
 */
export function roadMeshes(roads: readonly RoadPlan[], clip: Box | null = null, margin = 0, into: Partial<RoadMeshes> = {}, parts: 'both' | 'surface' | 'sidewalks' = 'both', junctions: readonly JunctionPlan[] = roadJunctions(roads)): RoadMeshes {
  const [withSurface, withSidewalks] = [parts !== 'sidewalks', parts !== 'surface'];
  const surface = into.surface ?? new Geo();
  const walk = into.walk ?? new Geo();
  const shadows = into.shadows ?? new Geo().color('#000000');
  const index = roadIndex(roads);
  const cuts = JunctionCuts.of(junctions);
  const near = (x: number, z: number, r = 0) => !clip || (x > clip.x - margin - r && x < clip.x + clip.w + margin + r && z > clip.z - margin - r && z < clip.z + clip.d + margin + r);
  const up: V3 = [0, 1, 0];
  roads.forEach((road, ri) => {
    const walks = withSidewalks && hasSidewalks(road);
    const frames = framesOf(road);
    const half = road.width / 2;
    const ends = deadEnds(road);
    /** A point `off` m left of the middle (right when below 0) at frame `i`, `y` up. */
    const P = (i: number, off: number, y: number): V3 => [frames[i].p[0] + frames[i].n[0] * off, y, frames[i].p[1] + frames[i].n[1] * off];
    for (let i = 0; i + 1 < frames.length; i++) {
      const [mx, mz] = [(frames[i].p[0] + frames[i + 1].p[0]) / 2, (frames[i].p[1] + frames[i + 1].p[1]) / 2];
      if (!near(mx, mz, half)) continue;
      if (withSurface) surface.poly([P(i, half, ROAD_Y), P(i + 1, half, ROAD_Y), P(i + 1, -half, ROAD_Y), P(i, -half, ROAD_Y)], up);
      for (const side of walks ? ([1, -1] as const) : []) {
        // Offsets from the middle: the sidewalk's outer edge, the kerb's top, the road's edge, the gutter.
        const [edge, walkIn, kerb, gutter] = [half, half - (SIDEWALK - KERB), half - SIDEWALK, half - SIDEWALK - 0.4].map((o) => o * side);
        const out: V3 = [frames[i].n[0] * side, 0, frames[i].n[1] * side];
        const inward: V3 = [-out[0], 0, -out[2]];
        /** The point `off` m from the middle, `t` of the way from frame `i` to `i + 1` (`y` up). */
        const Q = (t: number, off: number, y: number): V3 => {
          const [a, b] = [frames[i], frames[i + 1]];
          const [nx, nz] = [a.n[0] + (b.n[0] - a.n[0]) * t, a.n[1] + (b.n[1] - a.n[1]) * t];
          const l = Math.hypot(nx, nz) || 1;
          return [a.p[0] + (b.p[0] - a.p[0]) * t + (nx / l) * off, y, a.p[1] + (b.p[1] - a.p[1]) * t + (nz / l) * off];
        };
        // A road it meets cuts the sidewalk square to its own kerb, not a ragged corner: its middle
        // is across another road there, or in a turning circle.
        const mid = (edge + walkIn) / 2;
        const cut = (t: number): boolean => {
          const [x, , z] = Q(t, mid, 0);
          return sidewalkCut(index, ri, ends, cuts, x, z);
        };
        // The stretch of the segment the sidewalk runs (`t0`..`t1`), its ends worked out where it's
        // cut: sampled every 0.5 m, then the boundary found between a clear and a cut sample.
        const runs: [number, number][] = [];
        const steps = Math.max(1, Math.ceil(Math.hypot(frames[i + 1].p[0] - frames[i].p[0], frames[i + 1].p[1] - frames[i].p[1]) / 0.5));
        // (Most of a sidewalk has no road near: one look round the segment's middle (`reach`, past
        // its ends with room for the bend) tells it runs the whole way, sparing the samples.)
        const [[ax, , az], [cx, , cz], [bx, , bz]] = [Q(0, mid, 0), Q(0.5, mid, 0), Q(1, mid, 0)];
        const reach = Math.max(Math.hypot(ax - cx, az - cz), Math.hypot(bx - cx, bz - cz)) + 0.5;
        const free = !index.onRoad(cx, cz, 0.2 + reach, ri) && !cuts.near(cx, cz, reach) && !inTurning(ends, cx, cz, reach);
        /** The boundary between a `blocked` sample and a `clear` one, worked out by halving. */
        const bound = (blocked: number, clear: number): number => {
          let [b, c] = [blocked, clear];
          for (let it = 0; it < 12; it++) {
            const m = (b + c) / 2;
            if (cut(m)) b = m;
            else c = m;
          }
          return b;
        };
        let from: number | null = free || !cut(0) ? 0 : null;
        for (let k = 1; k <= (free ? 0 : steps); k++) {
          const [prev, t] = [(k - 1) / steps, k / steps];
          if (cut(t)) {
            if (from !== null) runs.push([from, bound(t, prev)]);
            from = null;
          } else if (from === null) {
            from = bound(prev, t);
          }
        }
        if (from !== null) runs.push([from, 1]);
        if (runs.length === 0) continue;
        // The quad between two offsets over `t0`..`t1` (`alpha`: the shade's at the first, fading to none at the second).
        const strip = (g: Geo, t0: number, t1: number, a: number, ya: number, b: number, yb: number, n: V3, alpha?: number) =>
          g.poly([Q(t0, a, ya), Q(t1, a, ya), Q(t1, b, yb), Q(t0, b, yb)], n, undefined, alpha === undefined ? undefined : [alpha, alpha, 0, 0]);
        for (const [t0, t1] of runs) {
          walk.color('#E6E6E6');
          strip(walk, t0, t1, edge, WALK_Y, walkIn, WALK_Y, up);
          walk.color('#FFFFFF');
          strip(walk, t0, t1, walkIn, KERB_Y, kerb, KERB_Y, up);
          // Kerb faces: the step up from the walk and the drop to the road; the walk's own edge.
          strip(walk, t0, t1, walkIn, WALK_Y, walkIn, KERB_Y, out);
          strip(walk, t0, t1, kerb, KERB_Y, kerb, ROAD_Y, inward);
          walk.color('#E6E6E6');
          strip(walk, t0, t1, edge, WALK_Y, edge, 0, out);
          // Gutter: the road darkens along the kerb.
          strip(shadows, t0, t1, kerb, ROAD_Y + 0.003, gutter, ROAD_Y + 0.003, up, 0.3);
        }
      }
    }
    // Turning circles: road to the kerb, the sidewalk round it but where the lane comes in.
    for (const [ex, ez, r] of ends) {
      if (!near(ex, ez, r + SIDEWALK)) continue;
      const n = 28;
      const at = (a: number, rr: number, y: number): V3 => [ex + Math.cos(a) * rr, y, ez + Math.sin(a) * rr];
      const lane = road.points.filter((p) => Math.hypot(p[0] - ex, p[1] - ez) > r + SIDEWALK + 0.5);
      const laneIndex = new RoadIndex([{ ...road, points: lane.length >= 2 ? lane : road.points }]);
      const inLane = (x: number, z: number) => laneIndex.distance(0, x, z) < half;
      for (let k = 0; k < n; k++) {
        const [a0, a1] = [(k / n) * Math.PI * 2, ((k + 1) / n) * Math.PI * 2];
        if (withSurface) surface.poly([[ex, ROAD_Y, ez], at(a1, r + SIDEWALK, ROAD_Y), at(a0, r + SIDEWALK, ROAD_Y)], up);
        const am = (a0 + a1) / 2;
        if (!walks || inLane(ex + Math.cos(am) * (r + SIDEWALK / 2), ez + Math.sin(am) * (r + SIDEWALK / 2))) continue;
        const [edge, walkIn, kerb] = [r + SIDEWALK, r + KERB, r];
        const radial: V3 = [Math.cos(am), 0, Math.sin(am)];
        walk.color('#E6E6E6').poly([at(a0, walkIn, WALK_Y), at(a0, edge, WALK_Y), at(a1, edge, WALK_Y), at(a1, walkIn, WALK_Y)], up);
        walk.color('#FFFFFF').poly([at(a0, kerb, KERB_Y), at(a0, walkIn, KERB_Y), at(a1, walkIn, KERB_Y), at(a1, kerb, KERB_Y)], up);
        walk.poly([at(a0, kerb, ROAD_Y), at(a0, kerb, KERB_Y), at(a1, kerb, KERB_Y), at(a1, kerb, ROAD_Y)], [-radial[0], 0, -radial[2]]);
        walk.poly([at(a0, walkIn, KERB_Y), at(a0, walkIn, WALK_Y), at(a1, walkIn, WALK_Y), at(a1, walkIn, KERB_Y)], radial);
        walk.color('#E6E6E6').poly([at(a0, edge, 0), at(a1, edge, 0), at(a1, edge, WALK_Y), at(a0, edge, WALK_Y)], radial);
        shadows.poly([at(a0, kerb, ROAD_Y + 0.003), at(a1, kerb, ROAD_Y + 0.003), at(a1, kerb - 0.4, ROAD_Y + 0.003), at(a0, kerb - 0.4, ROAD_Y + 0.003)], up, undefined, [0.3, 0.3, 0, 0]);
      }
    }
  });
  for (const j of junctions) {
    if (!near(j.x, j.z, 20)) continue;
    if (withSurface) junctionPad(j, surface);
    if (withSidewalks) for (const c of j.corners) if (c.walk) junctionCorner(j, c, walk, shadows);
    if (withSurface && into.marks) junctionMarks(j, into.marks);
  }
  return { surface, walk, shadows, ...(into.marks ? { marks: into.marks } : {}) };
}

type Pt2 = readonly [number, number];
const UP: V3 = [0, 1, 0];

/** The points of an arc round `o` of radius `r` from the way `from` to the way `to` (unit vectors from `o`), the short way round, both ends included, a point every `step` radians at most. */
function arc(o: Pt2, r: number, from: Pt2, to: Pt2, step = Math.PI / 16): [number, number][] {
  const a0 = Math.atan2(from[1], from[0]);
  let sweep = Math.atan2(to[1], to[0]) - a0;
  while (sweep > Math.PI) sweep -= Math.PI * 2;
  while (sweep < -Math.PI) sweep += Math.PI * 2;
  const n = Math.max(2, Math.ceil(Math.abs(sweep) / step));
  return Array.from({ length: n + 1 }, (_, k) => {
    const a = a0 + (sweep * k) / n;
    return [o[0] + Math.cos(a) * r, o[1] + Math.sin(a) * r];
  });
}

/** A point `along` m out along arm `a` of junction `j` and `across` m to the anticlockwise side of its road's middle (less than 0: clockwise). */
function armPoint(j: JunctionPlan, a: number, along: number, across: number): [number, number] {
  const { dir, off } = j.arms[a];
  const n = turnedOn(dir);
  return [j.x + dir[0] * along + n[0] * (across + off), j.z + dir[1] * along + n[1] * (across + off)];
}

/**
 * A junction's pad, in the road's surface: its arms' carriageways out to their crosswalks (or past
 * their corners), the kerb returns' arcs between them, fanned from its middle (each corner's arc
 * seen whole from there).
 */
function junctionPad(j: JunctionPlan, surface: Geo): void {
  const ring: [number, number][] = [];
  const n = j.arms.length;
  for (let k = 0; k < n; k++) {
    const A = j.arms[k];
    const before = j.corners.find((c) => c.b === k);
    if (before) ring.push(armPoint(j, k, before.tb, -A.half));
    ring.push(armPoint(j, k, A.reach, -A.half), armPoint(j, k, A.reach, A.half));
    const after = j.corners.find((c) => c.a === k);
    if (after) {
      const B = j.arms[after.b];
      const o: Pt2 = [j.x + after.o[0], j.z + after.o[1]];
      // (From the kerb of this arm round to the next's, its ends left to the arms.)
      const pts = arc(o, KERB_RETURN, scale(turnedOn(A.dir), -1), scale(turnedBack(B.dir), -1));
      ring.push(...pts.slice(0, -1));
    }
  }
  if (ring.length < 3) return;
  surface.poly([[j.x, ROAD_Y, j.z], ...ring.map(([x, z]): V3 => [x, ROAD_Y, z]), [ring[0][0], ROAD_Y, ring[0][1]]], UP);
}

const scale = (v: Pt2, k: number): [number, number] => [v[0] * k, v[1] * k];

/**
 * The sidewalk turning a junction's corner `c`: from where the arms' sidewalks stop (`JunctionCuts`)
 * round the kerb return's arc, out to their sidewalks' outer edges (square at the corner of the land
 * behind); its kerb (top and faces) along the arc, and the gutter's shade below it.
 */
function junctionCorner(j: JunctionPlan, c: CornerPlan, walk: Geo, shadows: Geo): void {
  const [A, B] = [j.arms[c.a], j.arms[c.b]];
  const o: Pt2 = [j.x + c.o[0], j.z + c.o[1]];
  const r = KERB_RETURN;
  // The ways from the arc's middle to where it meets each arm's kerb.
  const [fa, fb] = [scale(turnedOn(A.dir), -1), scale(turnedBack(B.dir), -1)];
  const pa = armPoint(j, c.a, c.ta, A.half + SIDEWALK);
  const pb = armPoint(j, c.b, c.tb, -(B.half + SIDEWALK));
  const q = offsetMeet(A.dir, A.off + A.half + SIDEWALK, B.dir, B.half - B.off + SIDEWALK);
  if (!q) return;
  const qq: [number, number] = [j.x + q[0], j.z + q[1]];
  const inner = arc(o, r - KERB, fb, fa);
  const at = (p: Pt2, y: number): V3 => [p[0], y, p[1]];
  // The sidewalk: fanned from the land's corner (it sees the whole arc).
  walk.color('#E6E6E6').poly([at(qq, WALK_Y), at(pb, WALK_Y), ...inner.map((p) => at(p, WALK_Y)), at(pa, WALK_Y)], UP);
  // Its outer edges.
  walk.poly([at(pa, 0), at(qq, 0), at(qq, WALK_Y), at(pa, WALK_Y)], [...nv(turnedOn(A.dir))]);
  walk.poly([at(qq, 0), at(pb, 0), at(pb, WALK_Y), at(qq, WALK_Y)], [...nv(turnedBack(B.dir))]);
  // The kerb round the arc: its top, the drop to the road, the step up from the walk; the gutter.
  const outer = arc(o, r, fb, fa);
  const gutter = arc(o, r + 0.4, fb, fa);
  for (let k = 0; k + 1 < outer.length; k++) {
    const [i0, i1, o0, o1, g0, g1] = [inner[k], inner[k + 1], outer[k], outer[k + 1], gutter[k], gutter[k + 1]];
    const m: Pt2 = [(o0[0] + o1[0]) / 2 - o[0], (o0[1] + o1[1]) / 2 - o[1]];
    const l = Math.hypot(m[0], m[1]) || 1;
    const radial: V3 = [m[0] / l, 0, m[1] / l];
    walk.color('#FFFFFF').poly([at(i0, KERB_Y), at(o0, KERB_Y), at(o1, KERB_Y), at(i1, KERB_Y)], UP);
    walk.poly([at(o0, ROAD_Y), at(o0, KERB_Y), at(o1, KERB_Y), at(o1, ROAD_Y)], radial);
    walk.poly([at(i0, KERB_Y), at(i0, WALK_Y), at(i1, WALK_Y), at(i1, KERB_Y)], [-radial[0], 0, -radial[2]]);
    shadows.poly([at(o0, ROAD_Y + 0.003), at(o1, ROAD_Y + 0.003), at(g1, ROAD_Y + 0.003), at(g0, ROAD_Y + 0.003)], UP, undefined, [0.3, 0.3, 0, 0]);
  }
  // A short way on along each arm's sidewalk, under where it begins: no gap where its road bends
  // off the arm's way out (its sidewalk's end is square to its road there, the corner's to the arm).
  for (const [k, t, side] of [[c.a, c.ta, 1], [c.b, c.tb, -1]] as const) {
    // (A country road's arm has none of its own to meet.)
    if (!j.arms[k].walks) continue;
    const h = j.arms[k].half;
    const P = (along: number, off: number, y: number): V3 => {
      const [x, z] = armPoint(j, k, along, off * side);
      return [x, y, z];
    };
    const [t0, t1] = [t - 0.05, t + CORNER_ON];
    const n = turnedOn(j.arms[k].dir);
    const out: V3 = [n[0] * side, 0, n[1] * side];
    const inward: V3 = [-out[0], 0, -out[2]];
    walk.color('#E6E6E6').poly([P(t0, h + KERB, WALK_Y), P(t1, h + KERB, WALK_Y), P(t1, h + SIDEWALK, WALK_Y), P(t0, h + SIDEWALK, WALK_Y)], UP);
    walk.poly([P(t0, h + SIDEWALK, 0), P(t1, h + SIDEWALK, 0), P(t1, h + SIDEWALK, WALK_Y), P(t0, h + SIDEWALK, WALK_Y)], out);
    walk.color('#FFFFFF').poly([P(t0, h, KERB_Y), P(t1, h, KERB_Y), P(t1, h + KERB, KERB_Y), P(t0, h + KERB, KERB_Y)], UP);
    walk.poly([P(t0, h, ROAD_Y), P(t1, h, ROAD_Y), P(t1, h, KERB_Y), P(t0, h, KERB_Y)], inward);
    walk.poly([P(t0, h + KERB, WALK_Y), P(t1, h + KERB, WALK_Y), P(t1, h + KERB, KERB_Y), P(t0, h + KERB, KERB_Y)], out);
    shadows.poly([P(t0, h, ROAD_Y + 0.003), P(t1, h, ROAD_Y + 0.003), P(t1, h - 0.4, ROAD_Y + 0.003), P(t0, h - 0.4, ROAD_Y + 0.003)], UP, undefined, [0.3, 0.3, 0, 0]);
  }
}

/** How far on along an arm past its corner the corner's sidewalk reaches, under the arm's own (m). */
const CORNER_ON = 0.6;

const nv = (v: Pt2): V3 => [v[0], 0, v[1]];

/**
 * A junction's markings: on each arm into a street or lane, a zebra crosswalk across its
 * carriageway (`walkFrom`..`walkTo`) and a stop line across the half coming in (traffic keeps to
 * the right: the clockwise side of the way out) at `stop`; on a country road's arm, only the stop
 * line, and only when it's lit (no sidewalks to cross between).
 */
function junctionMarks(j: JunctionPlan, marks: Geo): void {
  marks.color(PAINT, PAINT_FINISH);
  const rect = (a: number, s0: number, s1: number, c0: number, c1: number) =>
    marks.poly([armPoint(j, a, s0, c0), armPoint(j, a, s1, c0), armPoint(j, a, s1, c1), armPoint(j, a, s0, c1)].map(([x, z]): V3 => [x, MARK_Y, z]), UP);
  j.arms.forEach((A, a) => {
    const half = A.half - MARK_IN;
    if (half <= 0.5) return;
    // (Not past the end of a road that ends there.)
    if ((A.walks || A.lit) && A.stop <= A.room) rect(a, A.stop - STOP_LINE, A.stop, -half, 0);
    if (!A.walks || A.walkTo - A.walkFrom < 1 || A.walkTo > A.room) return;
    // The stripes across the carriageway, as many as fit, the middle of the road between two or on one.
    const n = Math.max(1, Math.floor((2 * half + STRIPE_GAP) / (STRIPE + STRIPE_GAP)));
    const span = n * STRIPE + (n - 1) * STRIPE_GAP;
    for (let k = 0; k < n; k++) {
      const c0 = -span / 2 + k * (STRIPE + STRIPE_GAP);
      rect(a, A.walkFrom + 0.2, A.walkTo - 0.2, c0, c0 + STRIPE);
    }
  });
}
