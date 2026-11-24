/**
 * The lines painted along the roads (engine-neutral), into the road markings' mesh (`RoadMeshes.marks`: vertex
 * colours in the road paint, `MaterialLibrary.roadPaint`) with the junctions' stop lines and zebras (`roads.ts`
 * `junctionMarks`):
 *
 * - an avenue's and a motorway's lanes going the same way parted by dashed lines;
 * - a motorway's and a ramp's edges by solid lines (a motorway's lanes from its inside, by the barrier, its shoulder
 *   outside them: `laneLayout`), broken where a ramp leaves it or joins it (the gore);
 * - a one-way road's way by arrows down each lane, every `ARROW_EVERY` m and just past each junction (a motorway's
 *   further apart).
 *
 * None across a junction (`junctionStretches`). Streets, lanes and country roads going both ways stay unmarked.
 */

import type { RoadPlan } from '../../core/protocol';
import { kindInfo, kindOf, onewayOf } from '../../core/roadKinds';
import { lengths, nearest, RoadIndex } from '../../game/roads';
import { encodeFinish, type Geo, type V3 } from './geometry';
import { carriageway, type JunctionPlan } from './junctions';
import { aside, insideOf, MOTORWAY, Walk } from './motorway';
import type { RoadLevels } from './roads';

/** How high the markings lie over the road (m: `roads.ts` `MARK_Y`, which this module can't import without a cycle). */
const MARK_UP = 0.016;
/**
 * The markings' paint (these lines', and the junctions' stop lines and zebras: `roads.ts` `junctionMarks`): a vertex colour
 * (worn white: no brighter than the sidewalks) with the plain finish's roughness in its alpha (`geometry.ts` `encodeFinish`).
 */
export const PAINT = '#D6D6D0';
export const PAINT_FINISH = encodeFinish(0.7, 0);
/** A line's width (m): a dashed lane line, a solid edge line. */
const DASH_WIDE = 0.15;
const EDGE_WIDE = 0.2;
/** Dashes (m): a town road's and a motorway's, each its dash and the gap after it. */
const DASH = { town: [3, 6], motorway: [6, 12] } as const;
/** How far apart a one-way road's arrows are (m), a motorway's, and how far past a junction the first is. */
export const ARROW_EVERY = 40;
const ARROW_EVERY_MOTORWAY = 100;
const ARROW_PAST = 6;
/** An arrow's measures (m): its length, its shaft's width, its head's length and width. */
const ARROW = { long: 5, shaft: 0.18, head: 1.6, wide: 0.9 } as const;
/** How far apart along a line its pieces are taken (m). */
const STEP = 2;
/** How far a ramp's edge line keeps in from its edge (m). */
const RAMP_EDGE = 0.35;

/**
 * A lane: its middle (m from the road's middle, + towards (t.z, -t.x) of the way its points run: its left on the town map,
 * which draws z down the page, so its right as the game shows it, `render/pc/space.ts`) and the way its cars go (+1: along
 * its points).
 */
export interface Lane {
  off: number;
  way: 1 | -1;
}

/** A road's lines across (m from its middle, + as `Lane.off`): its lanes, the dashed lines between lanes going the same way, its solid edge lines. */
export interface LaneLayout {
  lanes: Lane[];
  dividers: number[];
  edges: number[];
  /** Its dashes: a town road's or a motorway's. */
  dash: 'town' | 'motorway';
}

/**
 * Road `ri`'s lines across (null: none, a two-way street, lane or country road, a trail): an avenue's two lanes each
 * way between its kerbs and its median (traffic keeping to the right); a motorway carriageway's lanes from its inside
 * (`insideOf`: towards its pair), an edge line each side of them, its shoulder outside; a ramp's one lane between two
 * edge lines; a one-way street's one lane.
 */
export function laneLayout(roads: readonly RoadPlan[], ri: number): LaneLayout | null {
  const road = roads[ri];
  const info = kindInfo(road);
  const way = onewayOf(road);
  const half = carriageway(road) / 2;
  switch (kindOf(road)) {
    case 'avenue': {
      const m = info.median / 2;
      const w = (half - m) / info.lanesEachWay;
      const lanes: Lane[] = [];
      const dividers: number[] = [];
      for (let k = 0; k < info.lanesEachWay; k++) {
        // (Traffic on the right as the game shows it (sim-core `cars::right_of`): going along its points, +; against
        // them, -.)
        lanes.push({ off: m + w * (k + 0.5), way: 1 }, { off: -(m + w * (k + 0.5)), way: -1 });
        if (k > 0) dividers.push(-(m + w * k), m + w * k);
      }
      return { lanes, dividers, edges: [], dash: 'town' };
    }
    case 'motorway': {
      const inside = insideOf(roads, ri);
      const from = (d: number) => inside * (half - d);
      const M = MOTORWAY;
      const n = info.lanesEachWay;
      const lanes = Array.from({ length: n }, (_, k): Lane => ({ off: from(M.inner + M.lane * (k + 0.5)), way: way < 0 ? -1 : 1 }));
      const dividers = Array.from({ length: n - 1 }, (_, k) => from(M.inner + M.lane * (k + 1)));
      return { lanes, dividers, edges: [from(M.inner - EDGE_WIDE / 2 - 0.05), from(M.inner + M.lane * n + EDGE_WIDE / 2 + 0.05)], dash: 'motorway' };
    }
    case 'ramp':
      return { lanes: [{ off: 0, way: way < 0 ? -1 : 1 }], dividers: [], edges: [half - RAMP_EDGE, -(half - RAMP_EDGE)], dash: 'town' };
    default:
      return way ? { lanes: [{ off: 0, way }], dividers: [], edges: [], dash: 'town' } : null;
  }
}

/** Each set of junctions' stretches of each road (`junctionStretches`): a road's points are never changed in place. */
const stretchesOf = new WeakMap<readonly JunctionPlan[], WeakMap<RoadPlan, readonly [number, number][]>>();

/**
 * The stretches of `road` (arc lengths, m, each start and end) its junctions cover: round each junction on it (its
 * middle within its half width of the road's), out along its arms as far as their pads and stop lines reach. Once per
 * road and set of junctions.
 */
export function junctionStretches(road: RoadPlan, junctions: readonly JunctionPlan[], ls?: readonly number[]): readonly [number, number][] {
  let byRoad = stretchesOf.get(junctions);
  if (!byRoad) stretchesOf.set(junctions, (byRoad = new WeakMap()));
  let had = byRoad.get(road);
  if (!had) byRoad.set(road, (had = stretchesAt(road, junctions, ls ?? lengths(road.points))));
  return had;
}

function stretchesAt(road: RoadPlan, junctions: readonly JunctionPlan[], ls: readonly number[]): [number, number][] {
  const out: [number, number][] = [];
  const most = road.width / 2 + 0.5;
  // (Only the junctions near its box are measured: a metre to spare.)
  let [x0, z0, x1, z1] = [Infinity, Infinity, -Infinity, -Infinity];
  for (const [x, z] of road.points) [x0, z0, x1, z1] = [Math.min(x0, x), Math.min(z0, z), Math.max(x1, x), Math.max(z1, z)];
  const [bx0, bz0, bx1, bz1] = [x0 - most - 1, z0 - most - 1, x1 + most + 1, z1 + most + 1];
  for (const j of junctions) {
    if (j.x < bx0 || j.x > bx1 || j.z < bz0 || j.z > bz1) continue;
    const near = nearest(road.points, j.x, j.z, ls);
    if (near.dist > most) continue;
    let reach = 0;
    for (const A of j.arms) reach = Math.max(reach, A.reach, Math.min(A.stop, A.room));
    out.push([near.s - reach - 0.3, near.s + reach + 0.3]);
  }
  return out.sort((a, b) => a[0] - b[0]);
}

/** Whether `s` is in one of `stretches`, `m` m round each. */
const within = (stretches: readonly [number, number][], s: number, m = 0) => stretches.some(([a, b]) => s > a - m && s < b + m);

/** Each set of roads' indexes of its motorways and its ramps (where their lines break for each other: the gore). */
const goresOf = new WeakMap<readonly RoadPlan[], { motorways: RoadIndex; ramps: RoadIndex }>();

function gores(roads: readonly RoadPlan[]): { motorways: RoadIndex; ramps: RoadIndex } {
  let g = goresOf.get(roads);
  if (!g) goresOf.set(roads, (g = { motorways: new RoadIndex(roads.filter((r) => r.kind === 'motorway')), ramps: new RoadIndex(roads.filter((r) => r.kind === 'ramp')) }));
  return g;
}

/**
 * The lines along `roads` (`laneLayout`) into `marks`, in the road paint, at the roads' heights (`levels`; none: at 0),
 * none across the `junctions` (`junctionStretches`), only near `near` if given (a chunk's square, grown).
 */
export function roadMarkings(roads: readonly RoadPlan[], marks: Geo, junctions: readonly JunctionPlan[], levels?: RoadLevels | null, near?: (x: number, z: number) => boolean): void {
  const up: V3 = [0, 1, 0];
  roads.forEach((road, ri) => {
    if (road.points.length < 2) return;
    const lay = laneLayout(roads, ri);
    if (!lay) return;
    const ls = lengths(road.points);
    const total = ls[ls.length - 1];
    const cut = junctionStretches(road, junctions, ls);
    const walk = new Walk(road.points, ls);
    const kind = kindOf(road);
    // (A motorway's lines break where a ramp runs on it, a ramp's where it runs on the motorway: the gore between them.)
    const g = kind === 'motorway' || kind === 'ramp' ? gores(roads) : null;
    const gore = (x: number, z: number) => (kind === 'motorway' ? g!.ramps.onRoad(x, z, -0.2) : kind === 'ramp' ? g!.motorways.onRoad(x, z, -0.1) : false);
    const Y = (s: number) => (levels ? levels.road(road, s) : 0) + MARK_UP;
    const point = (s: number, off: number): V3 => {
      const { p, t } = walk.at(s);
      const [x, z] = aside(p, t, off);
      return [x, Y(s), z];
    };
    marks.color(PAINT, PAINT_FINISH);
    /** A line `wide` across at `off` from `s0` to `s1`, in pieces, none in a junction nor in the gore. */
    const line = (off: number, s0: number, s1: number, wide: number) => {
      const n = Math.max(1, Math.ceil((s1 - s0) / STEP));
      for (let k = 0; k < n; k++) {
        const [a, b] = [s0 + ((s1 - s0) * k) / n, s0 + ((s1 - s0) * (k + 1)) / n];
        const m = (a + b) / 2;
        if (within(cut, m)) continue;
        const mid = point(m, off);
        if ((near && !near(mid[0], mid[2])) || gore(mid[0], mid[2])) continue;
        marks.poly([point(a, off - wide / 2), point(b, off - wide / 2), point(b, off + wide / 2), point(a, off + wide / 2)], up);
      }
    };
    for (const off of lay.edges) line(off, 0, total, EDGE_WIDE);
    const [dash, gap] = DASH[lay.dash];
    for (const off of lay.dividers) for (let s = gap / 2; s + dash <= total; s += dash + gap) line(off, s, s + dash, DASH_WIDE);
    // Arrows down a one-way road's lanes.
    if (!onewayOf(road)) return;
    const every = kind === 'motorway' ? ARROW_EVERY_MOTORWAY : ARROW_EVERY;
    const at: number[] = [];
    const room = (s: number) => s > ARROW.long && s < total - ARROW.long && !within(cut, s, ARROW.long) && !at.some((o) => Math.abs(o - s) < every / 3);
    // (Just past each junction, the way its cars go; then along it.)
    const way = onewayOf(road);
    for (const [a, b] of cut) {
      const s = way > 0 ? b + ARROW_PAST : a - ARROW_PAST;
      if (room(s)) at.push(s);
    }
    for (let s = every / 2; s < total; s += every) if (room(s)) at.push(s);
    for (const s of at) {
      for (const lane of lay.lanes) {
        const mid = point(s, lane.off);
        if ((near && !near(mid[0], mid[2])) || gore(mid[0], mid[2])) continue;
        arrow(marks, (u, v) => point(s + u * lane.way, lane.off - v * lane.way), up);
      }
    }
  });
}

/**
 * An arrow pointing along a lane, drawn by `at(u, v)`: `u` m along the way it points from its middle, `v` m across it (either way: it is symmetric).
 * Its shaft, then its head.
 */
function arrow(marks: Geo, at: (u: number, v: number) => V3, up: V3): void {
  const A = ARROW;
  const [tail, neck, tip] = [-A.long / 2, A.long / 2 - A.head, A.long / 2];
  const s = A.shaft / 2;
  marks.poly([at(tail, -s), at(neck, -s), at(neck, s), at(tail, s)], up);
  marks.poly([at(neck, -A.wide / 2), at(tip, 0), at(neck, A.wide / 2)], up);
}
