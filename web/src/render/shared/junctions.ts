/**
 * The town's junctions as drawn (engine-neutral): where three roads for cars or more meet, each
 * junction's arms (the way out along each, its carriageway, where its crosswalk and stop line are,
 * whether it has a zebra and lights)
 * as the simulation has them (`WorldStructure.junctions`, sim-core `traffic::Signal`), or worked out
 * from the roads the way the simulation joins them up (`deriveJunctions`: a structure without them,
 * an older sim, the tests), and from them what the renderer draws (`planJunctions`): the pad of
 * asphalt joining the arms with a kerb return of `KERB_RETURN` m round each corner between two of
 * them, the sidewalk turning that corner, and where the roads' own sidewalks stop for it
 * (`JunctionCuts`). The meshes: `render/shared/roads.ts` `roadMeshes`.
 *
 * Roundabouts keep their own look (a ring round a fountain): no pad, no markings, no corners.
 */

import type { Junction, RoadPlan, WorldStructure } from '../../core/protocol';
import { lengths, pointAt, SIDEWALK, type Pt } from '../../game/roads';

export type { Junction };

/** The radius of a kerb return (m: from the kerb's line round the corner between two arms). */
export const KERB_RETURN = 3;
/** A crosswalk's width along its arm, and how far past it the stop line is (m; sim-core `traffic.rs`). */
export const CROSSWALK = 3;
export const STOP_PAST = 0.5;
/** The box round a junction (sim-core `traffic::boxes`): how far out along its arms (m) at least and at most, and how far apart two arms' lanes are at its edge. */
const BOX_LEAST = 6;
const BOX_MOST = 14;
const ARMS_APART = 4.6;
/** How far along an arm's road its way out is taken (m, sim-core `ARM_LOOK`). */
const ARM_LOOK = 5;
/** Two roads' meeting places this near are one junction (m). */
const SAME_JUNCTION = 1.5;
/** A junction is lit where this many arms meet, or this many of streets and country roads (sim-core `traffic::LIT_ARMS`, `LIT_MAIN_ARMS`). */
const LIT_ARMS = 4;
const LIT_MAIN_ARMS = 3;
/** Two arms whose ways out are further apart than 150° (the cosine) are across from each other (sim-core `traffic::OPPOSITE`). */
const OPPOSITE = -0.866;
/** A corner between two arms gets a kerb return only when they are this far apart, and no further (radians): straight on there's no corner, and a sliver of one would reach far out. */
const CORNER_LEAST = (20 * Math.PI) / 180;
const CORNER_MOST = (160 * Math.PI) / 180;
/** How far past a sidewalk's outer edge its cut by a corner reaches (m; as `sidewalkCut`'s margin). */
const CUT_MARGIN = 0.2;
/** Where along an arm its road's middle is measured from the way out (m), and how far off it may be taken to be (m). */
const OFF_AT = 6;
const OFF_MOST = 1.5;

/** Whether cars use a road (not a trail), and whether it has sidewalks (a town street or lane). */
const drivable = (r: RoadPlan | undefined) => !!r && r.kind !== 'trail' && r.points.length >= 2;
export const hasSidewalks = (r: RoadPlan | undefined) => !!r && r.kind !== 'country' && r.kind !== 'trail';
/** A road's carriageway, kerb to kerb (m): a country road has no sidewalks. */
export const carriageway = (r: RoadPlan) => (hasSidewalks(r) ? r.width - 2 * SIDEWALK : r.width);

// ---- the junctions from the roads ---------------------------------------------------------

/** Each set of roads' junctions, worked out once. */
const derived = new WeakMap<readonly RoadPlan[], Junction[]>();

/**
 * The junctions of `roads` as the simulation joins them up (sim-core `travel::Graph::build`): a
 * road's end on another road's middle line (within half the wider one's width and a metre), two
 * roads crossing, places this near one another one junction; those where three arms or more meet.
 * Its middle is on the road met (a lane begins at that road's kerb: `trimJoins`). Each arm's
 * crosswalk is just out past the junction's box (sim-core `traffic::boxes`), its stop line past
 * that. A junction is lit as the simulation lights them (sim-core `traffic::signal_at`: a big one,
 * where four arms or more meet or three of streets and country roads, and a roundabout's), its
 * heads numbered in the order worked out, not the snapshot's; any other gives way. Its zebras as
 * the simulation has them too: a lit one's on its streets' and lanes' arms, one giving way's on a
 * street's arm, one of two through it (the first round from the way out along z, as the
 * simulation goes round them).
 */
export function deriveJunctions(roads: readonly RoadPlan[]): Junction[] {
  const had = derived.get(roads);
  if (had) return had;
  const lens = roads.map((r) => lengths(r.points));
  const bounds = roads.map((r) => boundsOf(r.points));
  const ring = (r: RoadPlan) => r.ends[0] === 'loop';
  /** Where two roads meet: the point, and where along each. */
  const touches: { p: Pt; on: [road: number, s: number][] }[] = [];
  const near = (q: number, x: number, z: number) => nearestOn(roads[q].points, lens[q], x, z);
  roads.forEach((r, i) => {
    if (!drivable(r) || ring(r)) return;
    for (const e of [0, 1] as const) {
      const p = e === 0 ? r.points[0] : r.points[r.points.length - 1];
      const se = e === 0 ? 0 : lens[i][lens[i].length - 1];
      roads.forEach((o, q) => {
        if (q === i || !drivable(o)) return;
        const snap = Math.max(o.width, r.width) / 2 + 1;
        if (!boxNear(bounds[q], p, snap)) return;
        const { dist, s } = near(q, p[0], p[1]);
        if (dist > snap) return;
        const len = lens[q][lens[q].length - 1];
        // (Near the other's end: their ends meet there.)
        const sq = !ring(o) && s < snap ? 0 : !ring(o) && s > len - snap ? len : s;
        touches.push({ p: pointAt(o.points, sq, lens[q]).p, on: [[i, se], [q, sq]] });
      });
    }
  });
  for (const c of crossings(roads, lens)) touches.push(c);
  // Touches this near each other: one junction.
  const groups: (typeof touches)[] = [];
  for (const t of touches) {
    const g = groups.find((g) => g.some((u) => Math.hypot(u.p[0] - t.p[0], u.p[1] - t.p[1]) < SAME_JUNCTION));
    if (g) g.push(t);
    else groups.push([t]);
  }
  const out: Junction[] = [];
  let light = 0;
  for (const g of groups) {
    const x = g.reduce((n, t) => n + t.p[0], 0) / g.length;
    const z = g.reduce((n, t) => n + t.p[1], 0) / g.length;
    // Each road's arms: one out along it from its end, two from its middle.
    const along = new Map<number, number[]>();
    for (const t of g) for (const [road, s] of t.on) along.set(road, [...(along.get(road) ?? []), s]);
    const ways: { road: number; dir: Pt }[] = [];
    for (const [road, ss] of along) {
      const r = roads[road];
      const s = ss.reduce((n, v) => n + v, 0) / ss.length;
      const len = lens[road][lens[road].length - 1];
      for (const k of [1, -1]) {
        let at = s + k * ARM_LOOK;
        if (ring(r)) at = ((at % len) + len) % len;
        else if (at < 0 || at > len) {
          if (Math.abs(s - (k > 0 ? len : 0)) < 0.5) continue;
          at = Math.max(0, Math.min(len, at));
        }
        const q = pointAt(r.points, at, lens[road]).p;
        const l = Math.hypot(q[0] - x, q[1] - z);
        if (l < 0.5) continue;
        const dir: Pt = [(q[0] - x) / l, (q[1] - z) / l];
        if (ways.some((w) => w.dir[0] * dir[0] + w.dir[1] * dir[1] > 0.995)) continue;
        ways.push({ road, dir });
      }
    }
    if (ways.length < 3) continue;
    const roundabout = ways.some((w) => roads[w.road].island);
    ways.sort((a, b) => Math.atan2(a.dir[1], a.dir[0]) - Math.atan2(b.dir[1], b.dir[0]));
    const kind = (road: number) => roads[road].kind ?? 'street';
    const main = ways.filter((w) => kind(w.road) === 'street' || kind(w.road) === 'country').length;
    const lit = roundabout || ways.length >= LIT_ARMS || main >= LIT_MAIN_ARMS;
    // (Giving way, of a street's two arms through it the first round as the simulation goes, by
    // the angle from z towards x.)
    const round = (d: Pt) => Math.atan2(d[0], d[1]);
    const across = (w: (typeof ways)[number]) =>
      ways.some((o) => o !== w && kind(o.road) === 'street' && o.dir[0] * w.dir[0] + o.dir[1] * w.dir[1] < OPPOSITE && round(o.dir) < round(w.dir));
    const zebra = (w: (typeof ways)[number]) => !roundabout && (kind(w.road) === 'street' ? lit || !across(w) : kind(w.road) === 'lane' && lit);
    out.push({
      x,
      z,
      ...(roundabout ? { roundabout: true as const } : {}),
      arms: ways.map((w) => {
        const { road, dir } = w;
        const walkFrom = boxReach(dir, ways.map((w) => w.dir));
        return { road, dir, width: carriageway(roads[road]), walkFrom, walkTo: walkFrom + CROSSWALK, stop: walkFrom + CROSSWALK + STOP_PAST, crosswalk: zebra(w), light: lit && !roundabout ? light++ : null };
      }),
    });
  }
  derived.set(roads, out);
  return out;
}

/** How far out along arm `a` a junction's box reaches (m, sim-core `traffic::boxes`): as far as its lane is `ARMS_APART` from another's going the same way. */
function boxReach(a: Pt, all: readonly Pt[]): number {
  let r = BOX_LEAST;
  for (const b of all) {
    if (b === a) continue;
    const cos = a[0] * b[0] + a[1] * b[1];
    if (cos > 0) r = Math.max(r, ARMS_APART / Math.max(1e-3, Math.sqrt(Math.max(0, 1 - cos * cos))));
  }
  return Math.min(BOX_MOST, r);
}

/** A polyline's bounding box (x0, z0, x1, z1). */
function boundsOf(pts: readonly Pt[]): [number, number, number, number] {
  const b: [number, number, number, number] = [Infinity, Infinity, -Infinity, -Infinity];
  for (const [x, z] of pts) {
    if (x < b[0]) b[0] = x;
    if (x > b[2]) b[2] = x;
    if (z < b[1]) b[1] = z;
    if (z > b[3]) b[3] = z;
  }
  return b;
}

/** Whether `p` is within `m` m of a bounding box. */
const boxNear = ([x0, z0, x1, z1]: readonly number[], p: Pt, m: number) => p[0] > x0 - m && p[0] < x1 + m && p[1] > z0 - m && p[1] < z1 + m;

/** The nearest point of a polyline (its distance and arc length). */
function nearestOn(pts: readonly Pt[], ls: readonly number[], x: number, z: number): { dist: number; s: number } {
  let best = { dist: Infinity, s: 0 };
  for (let i = 0; i + 1 < pts.length; i++) {
    const [[ax, az], [bx, bz]] = [pts[i], pts[i + 1]];
    const [dx, dz] = [bx - ax, bz - az];
    const l2 = dx * dx + dz * dz || 1;
    const t = Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / l2));
    const dist = Math.hypot(x - (ax + dx * t), z - (az + dz * t));
    if (dist < best.dist) best = { dist, s: ls[i] + t * (ls[i + 1] - ls[i]) };
  }
  return best;
}

/** Where two roads for cars cross away from their ends (sim-core `travel::crossings`): segments filed by 8 m cells, each pair crossing once. */
function crossings(roads: readonly RoadPlan[], lens: readonly number[][]): { p: Pt; on: [number, number][] }[] {
  const CELL = 8;
  const cells = new Map<number, [number, number][]>();
  roads.forEach((r, i) => {
    if (!drivable(r)) return;
    for (let k = 0; k + 1 < r.points.length; k++) {
      const [[ax, az], [bx, bz]] = [r.points[k], r.points[k + 1]];
      for (let cz = Math.floor(Math.min(az, bz) / CELL); cz <= Math.floor(Math.max(az, bz) / CELL); cz++) {
        for (let cx = Math.floor(Math.min(ax, bx) / CELL); cx <= Math.floor(Math.max(ax, bx) / CELL); cx++) {
          const key = cx * 65536 + cz;
          const list = cells.get(key);
          if (list) list.push([i, k]);
          else cells.set(key, [[i, k]]);
        }
      }
    }
  });
  const out: { p: Pt; on: [number, number][] }[] = [];
  const seen = new Set<string>();
  for (const list of cells.values()) {
    for (let u = 0; u < list.length; u++) {
      for (let v = u + 1; v < list.length; v++) {
        const [[i, k], [j, l]] = [list[u], list[v]];
        if (i === j) continue;
        const key = i < j ? `${i}:${k}:${j}:${l}` : `${j}:${l}:${i}:${k}`;
        if (seen.has(key)) continue;
        seen.add(key);
        const [a0, a1, b0, b1] = [roads[i].points[k], roads[i].points[k + 1], roads[j].points[l], roads[j].points[l + 1]];
        const hit = cross(a0, a1, b0, b1);
        if (!hit) continue;
        const sa = lens[i][k] + (lens[i][k + 1] - lens[i][k]) * hit[0];
        const sb = lens[j][l] + (lens[j][l + 1] - lens[j][l]) * hit[1];
        const snap = Math.max(roads[i].width, roads[j].width) / 2 + 1;
        const inside = (r: number, s: number) => roads[r].ends[0] === 'loop' || (s >= snap && s <= lens[r][lens[r].length - 1] - snap);
        if (!inside(i, sa) || !inside(j, sb)) continue;
        out.push({ p: [a0[0] + (a1[0] - a0[0]) * hit[0], a0[1] + (a1[1] - a0[1]) * hit[0]], on: [[i, sa], [j, sb]] });
      }
    }
  }
  return out;
}

/** Where segments a0–a1 and b0–b1 cross (how far along each, 0..1), or null. */
function cross(a0: Pt, a1: Pt, b0: Pt, b1: Pt): [number, number] | null {
  const [ux, uz, vx, vz] = [a1[0] - a0[0], a1[1] - a0[1], b1[0] - b0[0], b1[1] - b0[1]];
  const d = ux * vz - uz * vx;
  if (Math.abs(d) < 1e-9) return null;
  const [wx, wz] = [b0[0] - a0[0], b0[1] - a0[1]];
  const t = (wx * vz - wz * vx) / d;
  const s = (wx * uz - wz * ux) / d;
  return t >= 0 && t < 1 && s >= 0 && s < 1 ? [t, s] : null;
}

// ---- the junctions as drawn ---------------------------------------------------------------

/**
 * An arm as drawn: its way out (a unit vector), how far its road's middle runs to the anticlockwise
 * side of that way from the junction's (`off`: a lane begun at a street's kerb meets it a little to
 * one side, its way out taken to a point along it), half its carriageway, whether its road has
 * sidewalks, its crosswalk and stop line (m from the middle), whether there's a zebra on it
 * (`crosswalk`), whether it's lit.
 */
export interface ArmPlan {
  road: number;
  dir: Pt;
  off: number;
  half: number;
  walks: boolean;
  walkFrom: number;
  walkTo: number;
  stop: number;
  crosswalk: boolean;
  lit: boolean;
  /** How far out the pad reaches along it (m). */
  reach: number;
  /** How far out its road goes (m: a short one ends before its crosswalk, which then isn't drawn). */
  room: number;
}

/**
 * The kerb return between arm `a` and the next round from it (`b`, anticlockwise as x turns to z):
 * the middle of its arc, and how far along each arm its kerb meets the arc. `walk`: a sidewalk turns
 * the corner (one of the two has them).
 */
export interface CornerPlan {
  a: number;
  b: number;
  o: Pt;
  ta: number;
  tb: number;
  walk: boolean;
}

/** A junction as drawn (not a roundabout's): its middle, its arms anticlockwise, its corners. */
export interface JunctionPlan {
  x: number;
  z: number;
  arms: ArmPlan[];
  corners: CornerPlan[];
}

/** A direction's left and right a quarter turn anticlockwise (x towards z) and clockwise. */
export const turnedOn = ([x, z]: Pt): Pt => [-z, x];
export const turnedBack = ([x, z]: Pt): Pt => [z, -x];

/** Each list of junctions as drawn, worked out once. */
const plans = new WeakMap<readonly Junction[], JunctionPlan[]>();

/** The junctions in `list` as drawn (`roads`: the town's, the arms' `road` indexing them), roundabouts left out. */
export function planJunctions(list: readonly Junction[], roads: readonly RoadPlan[]): JunctionPlan[] {
  const had = plans.get(list);
  if (had) return had;
  const out: JunctionPlan[] = [];
  for (const j of list) {
    if (j.roundabout || j.arms.length < 3) continue;
    const arms: ArmPlan[] = j.arms
      .map((a) => {
        const l = Math.hypot(a.dir[0], a.dir[1]) || 1;
        const { dir, off, room } = armLine(j, [a.dir[0] / l, a.dir[1] / l], roads[a.road]);
        const walks = hasSidewalks(roads[a.road]);
        return { road: a.road, dir, off, half: a.width / 2, walks, walkFrom: a.walkFrom, walkTo: a.walkTo, stop: a.stop, crosswalk: a.crosswalk ?? walks, lit: a.light !== null, reach: Math.min(a.walkTo, room), room };
      })
      .sort((a, b) => Math.atan2(a.dir[1], a.dir[0]) - Math.atan2(b.dir[1], b.dir[0]));
    const corners: CornerPlan[] = [];
    arms.forEach((A, a) => {
      const b = (a + 1) % arms.length;
      const B = arms[b];
      let turn = Math.atan2(B.dir[1], B.dir[0]) - Math.atan2(A.dir[1], A.dir[0]);
      if (turn <= 0) turn += Math.PI * 2;
      if (turn < CORNER_LEAST || turn > CORNER_MOST) return;
      // The arc's middle: `KERB_RETURN` out from both kerbs, on the corner's side of each.
      const o = offsetMeet(A.dir, A.off + A.half + KERB_RETURN, B.dir, B.half - B.off + KERB_RETURN);
      if (!o) return;
      const ta = o[0] * A.dir[0] + o[1] * A.dir[1];
      const tb = o[0] * B.dir[0] + o[1] * B.dir[1];
      // (Not round to an arm whose road ends before the corner would.)
      if (ta <= 0 || tb <= 0 || ta + 0.5 > A.room || tb + 0.5 > B.room) return;
      corners.push({ a, b, o, ta, tb, walk: A.walks || B.walks });
      A.reach = Math.max(A.reach, ta + 0.5);
      B.reach = Math.max(B.reach, tb + 0.5);
    });
    out.push({ x: j.x, z: j.z, arms, corners });
  }
  plans.set(list, out);
  return out;
}

/**
 * An arm's line as its road runs `OFF_AT` m out along the way `dir` out of junction `j`: the way it
 * runs there, and how far to the anticlockwise side of the junction's middle (as given, no offset:
 * its road isn't there).
 */
function armLine(j: Junction, dir: Pt, r: RoadPlan | undefined): { dir: Pt; off: number; room: number } {
  const as = { dir, off: 0, room: Infinity };
  if (!r || r.points.length < 2) return as;
  const [x, z] = [j.x + dir[0] * OFF_AT, j.z + dir[1] * OFF_AT];
  const ls = lengths(r.points);
  const { dist, s } = nearestOn(r.points, ls, x, z);
  const end = r.ends[0] === 'loop' ? Infinity : 0;
  const { p, t } = pointAt(r.points, s, ls);
  const k = t[0] * dir[0] + t[1] * dir[1] < 0 ? -1 : 1;
  // (How far on its road goes past there, the way out.)
  const room = Math.max(OFF_AT, OFF_AT + (end || (k > 0 ? ls[ls.length - 1] - s : s)));
  if (dist > OFF_MOST) return { ...as, room: dist > OFF_AT ? Infinity : room };
  const way: Pt = [t[0] * k, t[1] * k];
  if (way[0] * dir[0] + way[1] * dir[1] < Math.cos((30 * Math.PI) / 180)) return { ...as, room };
  const n = turnedOn(way);
  return { dir: way, off: (p[0] - j.x) * n[0] + (p[1] - j.z) * n[1], room };
}

/**
 * The point (from a junction's middle) `da` m to the anticlockwise side of the line along `a`
 * and `db` m to the clockwise side of the one along `b` (null: the two run the same way).
 */
export function offsetMeet(a: Pt, da: number, b: Pt, db: number): Pt | null {
  // p · turnedOn(a) = da, p · turnedBack(b) = db.
  const [n1, n2] = [turnedOn(a), turnedBack(b)];
  const det = n1[0] * n2[1] - n1[1] * n2[0];
  if (Math.abs(det) < 1e-6) return null;
  return [(da * n2[1] - db * n1[1]) / det, (n1[0] * db - n2[0] * da) / det];
}

/** The town's junctions as drawn: the simulation's (`WorldStructure.junctions`), else worked out from its roads. */
export function junctionsOf(world: WorldStructure, roads: readonly RoadPlan[]): JunctionPlan[] {
  return planJunctions(world.junctions ?? deriveJunctions(roads), roads);
}

// ---- where the roads' sidewalks stop for them -----------------------------------------------

/** A stretch of an arm's side where the roads' sidewalks stop for its corner: from the junction's middle out along `dir` to `to`, on `side` (+1 anticlockwise) out to `wide` m. */
interface Cut {
  x: number;
  z: number;
  dir: Pt;
  side: 1 | -1;
  to: number;
  wide: number;
}

/** Each set of junctions' cuts, filed once. */
const cutsOf = new WeakMap<readonly JunctionPlan[], JunctionCuts>();

/**
 * Where the roads' own sidewalks stop for the junctions' corners: along each arm on a corner's
 * side, out from the junction's middle to where its kerb meets the corner's arc (the corner's
 * sidewalk goes on round from there). Filed by 16 m cells.
 */
export class JunctionCuts {
  private readonly cells = new Map<number, Cut[]>();
  static of(list: readonly JunctionPlan[]): JunctionCuts {
    let c = cutsOf.get(list);
    if (!c) cutsOf.set(list, (c = new JunctionCuts(list)));
    return c;
  }

  private constructor(list: readonly JunctionPlan[]) {
    for (const j of list) {
      for (const c of j.corners) {
        if (!c.walk) continue;
        const [A, B] = [j.arms[c.a], j.arms[c.b]];
        this.file({ x: j.x, z: j.z, dir: A.dir, side: 1, to: c.ta, wide: A.off + A.half + SIDEWALK + CUT_MARGIN });
        this.file({ x: j.x, z: j.z, dir: B.dir, side: -1, to: c.tb, wide: B.half - B.off + SIDEWALK + CUT_MARGIN });
      }
    }
  }

  private file(c: Cut): void {
    const [ex, ez] = [c.x + c.dir[0] * c.to, c.z + c.dir[1] * c.to];
    const r = c.wide;
    for (let cz = Math.floor((Math.min(c.z, ez) - r) / 16); cz <= Math.floor((Math.max(c.z, ez) + r) / 16); cz++) {
      for (let cx = Math.floor((Math.min(c.x, ex) - r) / 16); cx <= Math.floor((Math.max(c.x, ex) + r) / 16); cx++) {
        const key = cx * 65536 + cz;
        const list = this.cells.get(key);
        if (list) list.push(c);
        else this.cells.set(key, [c]);
      }
    }
  }

  /** Whether a sidewalk stops at (x, z) for a junction's corner. */
  cut(x: number, z: number): boolean {
    return this.near(x, z, 0);
  }

  /** Whether a sidewalk stops for a junction's corner within `r` m of (x, z). */
  near(x: number, z: number, r: number): boolean {
    for (let cz = Math.floor((z - r) / 16); cz <= Math.floor((z + r) / 16); cz++) {
      for (let cx = Math.floor((x - r) / 16); cx <= Math.floor((x + r) / 16); cx++) {
        const list = this.cells.get(cx * 65536 + cz);
        if (!list) continue;
        for (const c of list) {
          const [dx, dz] = [x - c.x, z - c.z];
          const along = dx * c.dir[0] + dz * c.dir[1];
          // (Anticlockwise of the way out: `turnedOn`.)
          const across = (-dx * c.dir[1] + dz * c.dir[0]) * c.side;
          const out = Math.hypot(Math.max(0, -along, along - c.to), Math.max(0, -across, across - c.wide));
          if (out <= r) return true;
        }
      }
    }
    return false;
  }
}
