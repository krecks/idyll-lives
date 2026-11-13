/**
 * The town's junctions as drawn (engine-neutral): where three roads for cars or more meet, each
 * junction's arms (the way out along each, its carriageway, where its crosswalk and stop line are,
 * whether it has a zebra and lights) as the simulation has them (`WorldStructure.junctions`,
 * sim-core `traffic::Signal`), and from them what the renderer draws (`planJunctions`): the pad of
 * asphalt joining the arms with a kerb return of `KERB_RETURN` m round each corner between two of
 * them, the sidewalk turning that corner, and where the roads' own sidewalks stop for it
 * (`JunctionCuts`). The meshes: `render/shared/roads.ts` `roadMeshes`.
 *
 * Roundabouts keep their own look (a ring round a fountain): no pad, no markings, no corners.
 */

import type { Junction, RoadPlan, WorldStructure } from '../../core/protocol';
import { lengths, nearest, pointAt, SIDEWALK, type Pt } from '../../game/roads';

export type { Junction };

/** The radius of a kerb return (m: from the kerb's line round the corner between two arms). */
export const KERB_RETURN = 3;
/** A crosswalk's width along its arm, and how far past it the stop line is (m; sim-core `traffic.rs`). */
export const CROSSWALK = 3;
export const STOP_PAST = 0.5;

/** Whether a road has sidewalks (a town street or lane). */
export const hasSidewalks = (r: RoadPlan | undefined) => !!r && r.kind !== 'country' && r.kind !== 'trail';
/** A road's carriageway, kerb to kerb (m): a country road has no sidewalks. */
export const carriageway = (r: RoadPlan) => (hasSidewalks(r) ? r.width - 2 * SIDEWALK : r.width);

/** A corner between two arms gets a kerb return only when they are this far apart, and no further (radians): straight on there's no corner, and a sliver of one would reach far out. */
const CORNER_LEAST = (20 * Math.PI) / 180;
const CORNER_MOST = (160 * Math.PI) / 180;
/** How far past a sidewalk's outer edge its cut by a corner reaches (m; as `sidewalkCut`'s margin). */
const CUT_MARGIN = 0.2;
/** Where along an arm its road's middle is measured from the way out (m), and how far off it may be taken to be (m). */
const OFF_AT = 6;
const OFF_MOST = 1.5;

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
  const { dist, s } = nearest(r.points, x, z, ls);
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

/** The town's junctions as drawn: the simulation's (`WorldStructure.junctions`). */
export function junctionsOf(world: WorldStructure, roads: readonly RoadPlan[]): JunctionPlan[] {
  return planJunctions(world.junctions ?? [], roads);
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
