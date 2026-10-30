/**
 * A town's roads as they are drawn (engine-neutral): the road surface, and along both sides a
 * sidewalk with its kerb and a gutter's shade, following the road round its bends. Where a road
 * meets or crosses another, the sidewalk stops at the other's edge; a dead end gets a turning
 * circle with the sidewalk round it. `Kerb` walks a road's side by distance along it, for what
 * stands on the sidewalk (`street.ts`).
 *
 * Older towns have straight streets along x (`WorldMeta.streets`): drawn as roads the same way
 * (`townRoads`).
 */

import type { RoadPlan, WorldStructure } from '../../core/protocol';
import { Geo, type V3 } from './geometry';

/** Sidewalk width (including the kerb) on each side of the road, and its levels. */
export const SIDEWALK = 1.35;
export const KERB = 0.14;
export const WALK_Y = 0.03;
export const KERB_Y = 0.045;
export const ROAD_Y = 0.012;
/** How far a dead end's turning circle reaches past the lane's kerbs (m). */
const TURNING = 2.4;

type Pt = [number, number];
export interface Box {
  x: number;
  z: number;
  w: number;
  d: number;
}

/** The town's roads: its own, or an older town's straight streets as roads. */
export function townRoads(world: WorldStructure): RoadPlan[] {
  if (world.meta?.roads) return world.meta.roads;
  return (world.meta?.streets ?? [])
    .filter((s) => s.w >= s.d)
    .map((s): RoadPlan => ({ points: [[s.x, s.z + s.d / 2], [s.x + s.w, s.z + s.d / 2]], width: s.d, ends: ['out', 'out'] }));
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
  return world.meta.roads.flatMap((r) =>
    ([0, 1] as const)
      .filter((e) => r.ends[e] === 'out')
      .map((e) => {
        const pts = r.points;
        const [end, before] = e === 0 ? [pts[0], pts[1]] : [pts[pts.length - 1], pts[pts.length - 2]];
        const l = Math.hypot(end[0] - before[0], end[1] - before[1]) || 1;
        return { x: end[0], z: end[1], dx: (end[0] - before[0]) / l, dz: (end[1] - before[1]) / l };
      }),
  );
}

/** Tiles of a `w` × `d` town a road's surface or sidewalk covers (1), else 0 (each town's roads drawn once). */
const tilesOf = new WeakMap<readonly RoadPlan[], Uint8Array>();
export function roadTiles(roads: readonly RoadPlan[], w: number, d: number): Uint8Array {
  const had = tilesOf.get(roads);
  if (had && had.length === w * d) return had;
  const out = new Uint8Array(w * d);
  for (const r of roads) {
    const reach = r.width / 2;
    for (let i = 0; i + 1 < r.points.length; i++) {
      const [[ax, az], [bx, bz]] = [r.points[i], r.points[i + 1]];
      const [dx, dz] = [bx - ax, bz - az];
      const l2 = dx * dx + dz * dz || 1;
      for (let z = Math.max(0, Math.floor(Math.min(az, bz) - reach)); z <= Math.min(d - 1, Math.ceil(Math.max(az, bz) + reach)); z++) {
        for (let x = Math.max(0, Math.floor(Math.min(ax, bx) - reach)); x <= Math.min(w - 1, Math.ceil(Math.max(ax, bx) + reach)); x++) {
          const t = Math.max(0, Math.min(1, ((x + 0.5 - ax) * dx + (z + 0.5 - az) * dz) / l2));
          if (Math.hypot(x + 0.5 - (ax + dx * t), z + 0.5 - (az + dz * t)) < reach) out[z * w + x] = 1;
        }
      }
    }
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

/** Roads near points, for "is this on a road": each road's bounding box, grown. */
export class RoadIndex {
  private readonly boxes: [number, number, number, number][];

  constructor(readonly roads: readonly RoadPlan[]) {
    this.boxes = roads.map((r) => {
      const xs = r.points.map((p) => p[0]);
      const zs = r.points.map((p) => p[1]);
      const m = r.width / 2 + 1;
      return [Math.min(...xs) - m, Math.min(...zs) - m, Math.max(...xs) + m, Math.max(...zs) + m];
    });
  }

  /** Distance from road `i`'s middle (Infinity well away from it). */
  distance(i: number, x: number, z: number): number {
    const [x0, z0, x1, z1] = this.boxes[i];
    if (x < x0 || x > x1 || z < z0 || z > z1) return Infinity;
    const pts = this.roads[i].points;
    let best = Infinity;
    for (let k = 0; k + 1 < pts.length; k++) {
      const [[ax, az], [bx, bz]] = [pts[k], pts[k + 1]];
      const [dx, dz] = [bx - ax, bz - az];
      const t = Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / (dx * dx + dz * dz || 1)));
      best = Math.min(best, Math.hypot(x - (ax + dx * t), z - (az + dz * t)));
    }
    return best;
  }

  /** Whether (x, z) is on a road other than `skip` (within its width and `margin`). */
  onRoad(x: number, z: number, margin = 0, skip = -1): boolean {
    return this.roads.some((r, i) => i !== skip && this.distance(i, x, z) < r.width / 2 + margin);
  }

  /** The road nearest (x, z), how far its middle is, the arc length there and which side (+1: its left). */
  nearest(x: number, z: number): { road: number; dist: number; s: number; side: 1 | -1 } | null {
    let best: { road: number; dist: number; s: number; side: 1 | -1 } | null = null;
    this.roads.forEach((r, i) => {
      let s = 0;
      for (let k = 0; k + 1 < r.points.length; k++) {
        const [[ax, az], [bx, bz]] = [r.points[k], r.points[k + 1]];
        const [dx, dz] = [bx - ax, bz - az];
        const l = Math.hypot(dx, dz) || 1;
        const t = Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / (l * l)));
        const dist = Math.hypot(x - (ax + dx * t), z - (az + dz * t));
        if (!best || dist < best.dist) best = { road: i, dist, s: s + t * l, side: (x - ax) * dz - (z - az) * dx >= 0 ? 1 : -1 };
        s += l;
      }
    });
    return best;
  }
}

/**
 * The roads as far as they cut into others' sidewalks: a lane that joins a road begins at that
 * road's middle; for this it begins at the road's kerb, so it doesn't cut the sidewalk across
 * the road from it too.
 */
export function sidewalkCuts(roads: readonly RoadPlan[]): RoadIndex {
  const all = new RoadIndex(roads);
  return new RoadIndex(
    roads.map((r, i) => {
      let pts = r.points;
      for (const e of [0, 1] as const) {
        if (r.ends[e] !== 'join') continue;
        const end = e === 0 ? pts[0] : pts[pts.length - 1];
        // The road it joins: the one whose middle is nearest its end.
        let parent = -1;
        roads.forEach((_, k) => {
          if (k !== i && (parent < 0 || all.distance(k, end[0], end[1]) < all.distance(parent, end[0], end[1]))) parent = k;
        });
        if (parent < 0) continue;
        const kerb = roads[parent].width / 2 - SIDEWALK;
        const keep = (p: Pt) => all.distance(parent, p[0], p[1]) >= kerb;
        const kept = e === 0 ? pts.slice(Math.max(0, pts.findIndex(keep))) : pts.slice(0, pts.length - Math.max(0, [...pts].reverse().findIndex(keep)));
        if (kept.length >= 2) pts = kept;
      }
      return { ...r, points: pts };
    }),
  );
}

/**
 * One side of a road, by distance along it (`s`) and in from the sidewalk's outer edge (`d`):
 * where things on the sidewalk stand, and which way they face the road.
 */
export class Kerb {
  readonly length: number;
  private readonly frames: Frame[];

  constructor(
    readonly road: RoadPlan,
    readonly index: number,
    /** +1: the road's left (looking along it), -1: its right. */
    readonly side: 1 | -1,
    private readonly others: RoadIndex,
  ) {
    this.frames = framesOf(road);
    this.length = this.frames[this.frames.length - 1].s;
  }

  private frameAt(s: number): { p: Pt; t: Pt } {
    const f = this.frames;
    let i = 0;
    while (i < f.length - 2 && f[i + 1].s < s) i++;
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
    let best = { dist: Infinity, s: 0 };
    const f = this.frames;
    for (let i = 0; i + 1 < f.length; i++) {
      const [[ax, az], [bx, bz]] = [f[i].p, f[i + 1].p];
      const [dx, dz] = [bx - ax, bz - az];
      const l = Math.hypot(dx, dz) || 1;
      const t = Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / (l * l)));
      const dist = Math.hypot(x - (ax + dx * t), z - (az + dz * t));
      if (dist < best.dist) best = { dist, s: f[i].s + t * l };
    }
    return best.s;
  }

  /** Whether the sidewalk runs at `s` (not cut by a road it meets, not in a turning circle). */
  open(s: number): boolean {
    const [x, z] = this.at(s, (SIDEWALK - KERB) / 2);
    return !this.others.onRoad(x, z, 0.2, this.index) && !this.inTurning(x, z);
  }

  /** Whether a point is in the turning circle at a dead end of this road. */
  inTurning(x: number, z: number): boolean {
    return deadEnds(this.road).some(([cx, cz, r]) => Math.hypot(x - cx, z - cz) < r + SIDEWALK);
  }
}

/** A road's turning circles: their middles and radii (to the kerb). */
function deadEnds(road: RoadPlan): [number, number, number][] {
  const r = road.width / 2 - SIDEWALK + TURNING;
  const out: [number, number, number][] = [];
  if (road.ends[1] === 'dead') out.push([...road.points[road.points.length - 1], r]);
  if (road.ends[0] === 'dead') out.push([...road.points[0], r]);
  return out;
}

/** What is drawn of the roads: their surface, and the sidewalks and kerbs (vertex colours) with the gutters' shade. */
export interface RoadMeshes {
  surface: Geo;
  walk: Geo;
  shadows: Geo;
}

/**
 * The roads' surfaces, sidewalks, kerbs and gutters, only near `clip` (grown by `margin`) when
 * given: the road surface under the sidewalks too, the sidewalks on top (added to `into`'s meshes if given).
 */
export function roadMeshes(roads: readonly RoadPlan[], clip: Box | null = null, margin = 0, into: Partial<RoadMeshes> = {}): RoadMeshes {
  const surface = into.surface ?? new Geo();
  const walk = into.walk ?? new Geo();
  const shadows = into.shadows ?? new Geo().color('#000000');
  const index = sidewalkCuts(roads);
  const near = (x: number, z: number, r = 0) => !clip || (x > clip.x - margin - r && x < clip.x + clip.w + margin + r && z > clip.z - margin - r && z < clip.z + clip.d + margin + r);
  const up: V3 = [0, 1, 0];
  roads.forEach((road, ri) => {
    const frames = framesOf(road);
    const half = road.width / 2;
    const ends = deadEnds(road);
    /** A point `off` m left of the middle (right when below 0) at frame `i`, `y` up. */
    const P = (i: number, off: number, y: number): V3 => [frames[i].p[0] + frames[i].n[0] * off, y, frames[i].p[1] + frames[i].n[1] * off];
    for (let i = 0; i + 1 < frames.length; i++) {
      const [mx, mz] = [(frames[i].p[0] + frames[i + 1].p[0]) / 2, (frames[i].p[1] + frames[i + 1].p[1]) / 2];
      if (!near(mx, mz, half)) continue;
      surface.poly([P(i, half, ROAD_Y), P(i + 1, half, ROAD_Y), P(i + 1, -half, ROAD_Y), P(i, -half, ROAD_Y)], up);
      for (const side of [1, -1] as const) {
        // Offsets from the middle: the sidewalk's outer edge, the kerb's top, the road's edge, the gutter.
        const [edge, walkIn, kerb, gutter] = [half, half - (SIDEWALK - KERB), half - SIDEWALK, half - SIDEWALK - 0.4].map((o) => o * side);
        const [cx, cz] = [mx + frames[i].n[0] * (edge + walkIn) / 2, mz + frames[i].n[1] * (edge + walkIn) / 2];
        // Not across another road, nor round a turning circle.
        if (index.onRoad(cx, cz, 0.2, ri) || ends.some(([ex, ez, r]) => Math.hypot(cx - ex, cz - ez) < r + SIDEWALK)) continue;
        const out: V3 = [frames[i].n[0] * side, 0, frames[i].n[1] * side];
        const inward: V3 = [-out[0], 0, -out[2]];
        // The quad between two offsets (`alpha`: the shade's at the first, fading to none at the second).
        const strip = (g: Geo, a: number, ya: number, b: number, yb: number, n: V3, alpha?: number) =>
          g.poly([P(i, a, ya), P(i + 1, a, ya), P(i + 1, b, yb), P(i, b, yb)], n, undefined, alpha === undefined ? undefined : [alpha, alpha, 0, 0]);
        walk.color('#E6E6E6');
        strip(walk, edge, WALK_Y, walkIn, WALK_Y, up);
        walk.color('#FFFFFF');
        strip(walk, walkIn, KERB_Y, kerb, KERB_Y, up);
        // Kerb faces: the step up from the walk and the drop to the road; the walk's own edge.
        strip(walk, walkIn, WALK_Y, walkIn, KERB_Y, out);
        strip(walk, kerb, KERB_Y, kerb, ROAD_Y, inward);
        walk.color('#E6E6E6');
        strip(walk, edge, WALK_Y, edge, 0, out);
        // Gutter: the road darkens along the kerb.
        strip(shadows, kerb, ROAD_Y + 0.003, gutter, ROAD_Y + 0.003, up, 0.3);
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
        surface.poly([[ex, ROAD_Y, ez], at(a1, r + SIDEWALK, ROAD_Y), at(a0, r + SIDEWALK, ROAD_Y)], up);
        const am = (a0 + a1) / 2;
        if (inLane(ex + Math.cos(am) * (r + SIDEWALK / 2), ez + Math.sin(am) * (r + SIDEWALK / 2))) continue;
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
  return { surface, walk, shadows };
}
