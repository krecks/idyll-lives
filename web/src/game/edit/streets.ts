/**
 * Step 2's street edits: a street, lane, country road or footpath drawn on the town (snapped to
 * the roads it starts and ends at, joining them in a T, bridged over water: `bridges.ts`), a road
 * removed with what stands on it, a road, place, village or the town renamed. Each keeps the
 * generator's own rules (`roads.ts` `clearOf`, `roomAt`, `bends`).
 *
 * A drawn street:
 * - is the player's points simplified (`SIMPLIFY` m), its corners rounded (`rounded`), and made a
 *   curve as country roads are (`curve(evenly(...))`);
 * - begins and ends on a road within `SNAP` m of its ends, in a T that begins at that road's kerb
 *   (`trimJoins`), meeting it at 35° or more and square if it comes within 15° of square; or at a
 *   dead end within `SNAP` m, which it then carries on; or at the world's edge (a way out), or in a
 *   dead end of its own;
 * - ends at the first road it reaches (no crossroads: the player goes on from 14 m along);
 * - is a lane or a street of an area where it runs inside its box or within `NEAR_AREA` m of it,
 *   else a country road; a footpath is a trail;
 * - is refused, with the reason, where it bends tighter than 9 m, runs closer than 18 m alongside
 *   another road (`clearOf`), leaves the world other than by its end, meets a road within 14 m of
 *   another junction (`roomAt`), joins no road, or leaves a dead end reaching more than `COURT` m
 *   past the last road it meets;
 * - crosses water on a bridge (`bridges.ts`: straight, one deck, the street split round it) where
 *   it may, and is refused where it may not;
 * - clears the lots and backdrop buildings under it (`clear`; else it is refused), and the parking;
 * - is named, and the roads it is split into at its bridges, with names nobody in town has.
 *
 * The dead ends: a drawn dead end is held to `COURT` m past the last road it meets, as the
 * generator's are. The sim would drive a longer one in a street or a lane (the renderer paves a
 * turning circle at every dead end and the taxis turn round in any of a street's or a lane's,
 * `sim-core taxi.rs` `dead_end`), but not in a country road (no taxi turns round there), and the
 * town's checks (`townCheck.ts`) hold every court to `COURT`.
 */

import type { Content } from '../../content/content';
import type { RoadKind, RoadPlan } from '../../core/protocol';
import { surfacePlots } from '../../render/shared/builtSurface';
import { BRIDGE_IN, BRIDGE_SPAN, bridgeable, bridgeStretch, wetSpans, withBridge, withoutDeck } from '../bridges';
import { rle, simplify, unrle } from '../landMasks';
import { mulberry32 } from '../noise';
import { bends, clearOf, connector, COURT, cropRoads, curve, inside, leftOf, lengths, nearest, pointAt, RoadIndex, roomAt, trimJoins, type Fit, type Pt } from '../roads';
import { plotRect, type NeighbourhoodDraft, type Rect, type Templates } from '../town';
import { placeOf, specOf, streetNames } from '../world';
import type { Edited } from './edited';
import { fillStreet, removalRefused, removeLots } from './lots';

/** A street being drawn: its points as the player set them, its kind, whether what is in its way is cleared (else it is refused there) and whether it is filled with homes, and the seed its homes and names are drawn from. */
export interface StreetStroke {
  points: readonly Pt[];
  kind: RoadKind;
  clear: boolean;
  fill: boolean;
  seed: number;
}

/** What a name is given to: a road, a place (`OpenWorldDraft.places`), a village (an area), or the town. */
export type Named = { kind: 'road'; road: number } | { kind: 'place'; place: number } | { kind: 'area'; area: number } | { kind: 'town' };

/** An end of a drawn street snaps to a road, or a dead end, this near (m, from the road's edge; from the dead end's last point). */
const SNAP = 10;
/** The least angle a street meets a road at, and how near square it is made square. */
const MEET = (35 * Math.PI) / 180;
const SQUARE = (15 * Math.PI) / 180;
/** Junctions on one road this far apart at least (m: `roads.ts` `ROOM`, as `roomAt` keeps it). */
const ROOM = 14;
/** A footpath keeps this far from a road alongside it (m); roads keep `clearOf`'s 18 m. */
const TRAIL_APART = 6;
/** A street drawn this near an area's box (m) is one of its streets. */
const NEAR_AREA = 30;
/** A drawn line is simplified to this (m), and has a point this often before it is made a curve (m: `macro.ts` `CTRL_APART`). */
const SIMPLIFY = 5;
const CTRL_APART = 16;
/** Widths (m): every road in a big world (`macro.ts` `ROAD_WIDTH`), a footpath. */
const ROAD_WIDTH = 7;
const TRAIL_WIDTH = 2;
/** How much of a drawn line (m) is turned to meet a road, or to carry on a dead end's way. */
const APPROACH = 16;
/** A street's last metres into a junction run straight (m). */
const STRAIGHT_IN = 4;
/** The shortest street drawn (m). */
const SHORTEST = 16;
/** Lots, buildings and parking keep this far from a new road's edge (m), else they are in its way. */
const CORRIDOR = 1.5;
/** A street joins a road no nearer its end than this (m). */
const NEAR_END = 6;
/** Wet stretches of a line closer than this (m) are crossed on one bridge (each bridge's straight approaches and the road's turns onto them would overlap). */
const ONE_CROSSING = 56;
/** An outdoor place stands this near the end of its trail (m: `townCheck.ts` checks 6). */
const TRAIL_END = 6;

const dot = (a: Pt, b: Pt) => a[0] * b[0] + a[1] * b[1];
const unit = (x: number, z: number): Pt => {
  const l = Math.hypot(x, z) || 1;
  return [x / l, z / l];
};
const lengthOf = (pts: readonly Pt[]) => lengths(pts)[pts.length - 1];
const refused = (why: string): Edited => ({ refused: why });

/**
 * A polyline with points put in along its longer stretches, none longer than `most` m (copied from
 * `macro.ts` `evenly`, which is not exported): what `curve` makes of it keeps to it.
 */
function evenly(pts: readonly Pt[], most: number): Pt[] {
  const out: Pt[] = pts.length ? [pts[0]] : [];
  for (let i = 1; i < pts.length; i++) {
    const n = Math.ceil(Math.hypot(pts[i][0] - pts[i - 1][0], pts[i][1] - pts[i - 1][1]) / most);
    for (let k = 1; k < n; k++) out.push([pts[i - 1][0] + ((pts[i][0] - pts[i - 1][0]) * k) / n, pts[i - 1][1] + ((pts[i][1] - pts[i - 1][1]) * k) / n]);
    out.push(pts[i]);
  }
  return out;
}

/**
 * An open polyline with its corners cut (Chaikin, `times` over; its ends where they were): a
 * corner the player clicked is rounded by a share of the stretches either side of it, so a long
 * stretch turns widely and only a short zig-zag stays tight.
 */
function rounded(pts: readonly Pt[], times = 3): Pt[] {
  let out = [...pts];
  for (let k = 0; k < times && out.length > 2; k++) {
    const next: Pt[] = [out[0]];
    for (let i = 0; i + 1 < out.length; i++) {
      const [a, b] = [out[i], out[i + 1]];
      if (i > 0) next.push([a[0] * 0.75 + b[0] * 0.25, a[1] * 0.75 + b[1] * 0.25]);
      if (i + 2 < out.length) next.push([a[0] * 0.25 + b[0] * 0.75, a[1] * 0.25 + b[1] * 0.75]);
    }
    next.push(out[out.length - 1]);
    out = next;
  }
  return out;
}

/** The way a polyline leaves at its last point (a unit direction, from 3 m back; copied from `roads.ts` `endDir`, which is not exported). */
function endDir(pts: readonly Pt[]): Pt {
  const ls = lengths(pts);
  const back = pointAt(pts, ls[ls.length - 1] - 3, ls).p;
  const e = pts[pts.length - 1];
  return unit(e[0] - back[0], e[1] - back[1]);
}

/**
 * Whether the roads but those `without` are one network (as the sim joins them: an end within half
 * the wider road's width and a metre of the other's middle) with a road out of the world on it
 * (copied from `landFollow.ts` `oneNetwork`, which is not exported).
 */
function oneNetwork(roads: readonly RoadPlan[], without: ReadonlySet<number>): boolean {
  const nets = networks(roads, without);
  return new Set(nets.values()).size === 1 && roads.some((r, i) => !without.has(i) && r.ends.includes('out'));
}

/** Each road's network (a road's index stands for it), but those `without`: roads joined as the sim joins them. */
function networks(roads: readonly RoadPlan[], without: ReadonlySet<number>): Map<number, number> {
  const parent = roads.map((_, i) => i);
  const find = (i: number): number => (parent[i] === i ? i : (parent[i] = find(parent[i])));
  const index = new RoadIndex(roads);
  const widest = Math.max(0, ...roads.map((r) => r.width));
  roads.forEach((r, i) => {
    if (without.has(i)) return;
    for (const end of [r.points[0], r.points[r.points.length - 1]]) {
      for (const [j, m] of index.around(end[0], end[1], widest / 2 + 1)) {
        if (j !== i && !without.has(j) && m.dist <= Math.max(roads[j].width, r.width) / 2 + 1) parent[find(i)] = find(j);
      }
    }
  });
  return new Map(roads.flatMap((_, i) => (without.has(i) ? [] : [[i, find(i)] as [number, number]])));
}

/** How far a point is from a rectangle (0 inside it). */
const offRect = (r: Rect, x: number, z: number) => Math.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d));

/** How far a point is from a polygon (0 inside it). */
function offPoly(poly: readonly Pt[], x: number, z: number): number {
  if (inside(poly, x, z)) return 0;
  let best = Infinity;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const [[ax, az], [bx, bz]] = [poly[j], poly[i]];
    const [dx, dz] = [bx - ax, bz - az];
    const l2 = dx * dx + dz * dz || 1;
    const f = Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / l2));
    best = Math.min(best, Math.hypot(x - ax - dx * f, z - az - dz * f));
  }
  return best;
}

/** A parking space's ground (`macro.ts` lays them 3 × 6 m, turned with the verge). */
const spaceOf = (p: { x: number; z: number; rot: number }): Rect => (p.rot % 2 ? { x: p.x - 3, z: p.z - 1.5, w: 6, d: 3 } : { x: p.x - 1.5, z: p.z - 3, w: 3, d: 6 });

/** Every name in town: its roads', its places', its areas' and its own. */
function namesIn(draft: NeighbourhoodDraft): Set<string> {
  const open = draft.open;
  return new Set([draft.name, ...draft.roads.map((r) => r.name ?? ''), ...(open?.places ?? []).map((p) => p.name), ...(open?.areas ?? []).map((a) => a.name)].filter(Boolean));
}

/** `count` street names nobody in town has, from the place's own names (`world.ts` `streetNames`), drawn from `rand`. */
function freshNames(draft: NeighbourhoodDraft, content: Content, rand: () => number, count: number, taken: Set<string>, area?: string): string[] {
  if (!count) return [];
  const world = draft.world;
  const names = world ? specOf(placeOf(world.kind), world.options ?? {}).names : '';
  const out = streetNames(content, names, rand, count, [...taken], area);
  for (const n of out) taken.add(n);
  return out;
}

/** A bridge's name after the street it carries, nobody's in town. */
function bridgeName(street: string, taken: Set<string>): string {
  const word = street.split(' ')[0];
  const tries = [`${word} Bridge`, `${street} Bridge`, `${word} Old Bridge`];
  let pick = tries.find((n) => !taken.has(n));
  for (let k = 2; !pick; k++) if (!taken.has(`${word} Bridge ${k}`)) pick = `${word} Bridge ${k}`;
  taken.add(pick);
  return pick;
}

/** How a drawn street ends: on its own (a dead end), out of the world, joining a road in a T, or carrying on a road's dead end. */
type End = { kind: 'free' } | { kind: 'out' } | { kind: 'join'; road: number } | { kind: 'extend'; road: number; end: 0 | 1 };

const endKind = (e: End): RoadPlan['ends'][0] => (e.kind === 'free' ? 'dead' : e.kind === 'out' ? 'out' : 'join');
const reversed = (pts: readonly Pt[]): Pt[] => [...pts].reverse();

/**
 * The town's roads as a drawn street meets them: what it may join (a car's road a car's road, a
 * footpath any; never a roundabout) and what its ends snap to.
 */
class Meeting {
  readonly index: RoadIndex;
  private readonly widest: number;

  constructor(
    readonly roads: readonly RoadPlan[],
    readonly trail: boolean,
    readonly width: number,
  ) {
    this.index = new RoadIndex(roads);
    this.widest = Math.max(0, ...roads.map((r) => r.width));
  }

  joinable(r: number): boolean {
    const road = this.roads[r];
    return !road.island && (this.trail || road.kind !== 'trail');
  }

  /** How near a road's middle the street's middle comes where it touches it (m: `clearOf`'s). */
  touch(r: number): number {
    return this.roads[r].width / 2 + this.width / 2 + 1;
  }

  /** What the end `p` of a line snaps to: a dead end of a road of its own kind within `SNAP` m, else a road it may join within `SNAP` m of its edge, else nothing. */
  snap(p: Pt): End {
    let dead: End | null = null;
    let near = Infinity;
    for (const [k, r] of this.roads.entries()) {
      if (!this.joinable(k) || r.bridge || (r.kind === 'trail') !== this.trail) continue;
      for (const e of [0, 1] as const) {
        if (r.ends[e] !== 'dead') continue;
        const q = e ? r.points[r.points.length - 1] : r.points[0];
        const d = Math.hypot(q[0] - p[0], q[1] - p[1]);
        if (d <= SNAP && d < near) [dead, near] = [{ kind: 'extend', road: k, end: e }, d];
      }
    }
    if (dead) return dead;
    let best: End = { kind: 'free' };
    for (const [r, m] of this.index.around(p[0], p[1], SNAP + this.widest / 2)) {
      const edge = m.dist - this.roads[r].width / 2;
      if (edge <= SNAP && edge < near && this.joinable(r)) [best, near] = [{ kind: 'join', road: r }, edge];
    }
    return best;
  }

  /** The first point of `line` (its index) that touches a road, not counting road `from` near the line's start (`free` m); and that road. */
  contact(line: readonly Pt[], from: number): { i: number; road: number } | null {
    const ls = lengths(line);
    const free = from < 0 ? 0 : this.touch(from) * 1.8 + 1 + SNAP;
    for (let i = 0; i < line.length; i++) {
      for (const [r, m] of this.index.around(line[i][0], line[i][1], this.widest / 2 + this.width / 2 + 1)) {
        if (m.dist >= this.touch(r) || (r === from && ls[i] < free)) continue;
        return { i, road: r };
      }
    }
    return null;
  }

  /**
   * `line` with its last stretch turned to meet road `r` in a T: on the road's middle, nearest
   * where the line ends, at 35° or more (square within 15° of it), over the last `APPROACH` m;
   * the junction too (`at`); or why not.
   */
  joinTo(line: readonly Pt[], r: number): { pts: Pt[]; at: Pt } | string {
    const o = this.roads[r];
    const name = o.name ?? 'the road';
    const lo = lengths(o.points);
    const Lo = lo[lo.length - 1];
    const E = line[line.length - 1];
    const sT = this.index.along(r, E[0], E[1]);
    if (o.ends[0] !== 'loop' && (sT < NEAR_END || Lo - sT < NEAR_END)) return `it meets ${name} too near its end`;
    if (o.bridge && sT > o.bridge[0] - BRIDGE_IN && sT < o.bridge[1] + BRIDGE_IN) return `it meets ${name} on its bridge`;
    const { p: T, t: tq } = pointAt(o.points, sT, lo);
    const ls = lengths(line);
    const L = ls[ls.length - 1];
    const R = Math.min(APPROACH, L * 0.4);
    const back = pointAt(line, L - R, ls);
    const n = leftOf(tq);
    const sgn = dot([back.p[0] - T[0], back.p[1] - T[1]], n) >= 0 ? 1 : -1;
    const into: Pt = [-n[0] * sgn, -n[1] * sgn];
    const v = endDir(line);
    const along = dot(v, tq);
    const meet = Math.atan2(Math.max(0, dot(v, into)), Math.abs(along));
    let a = into;
    if (meet < Math.PI / 2 - SQUARE) {
      const k = Math.max(meet, MEET);
      const way = Math.sign(along) || 1;
      a = unit(into[0] * Math.sin(k) + tq[0] * way * Math.cos(k), into[1] * Math.sin(k) + tq[1] * way * Math.cos(k));
    }
    // (Its last `STRAIGHT_IN` m straight along that way.)
    const near: Pt = [T[0] - a[0] * STRAIGHT_IN, T[1] - a[1] * STRAIGHT_IN];
    const last = Array.from({ length: STRAIGHT_IN }, (_, k): Pt => [near[0] + a[0] * (k + 1), near[1] + a[1] * (k + 1)]);
    return { pts: [...line.filter((_, i) => ls[i] < L - R - 0.01), ...connector(back.p, back.t, near, a), ...last], at: T };
  }

  /** Road `r` with its dead end `end` last, and `line` (beginning by it) turned to carry on its way over its first `APPROACH` m. */
  extend(line: readonly Pt[], r: number, end: 0 | 1): { road: Pt[]; ends: RoadPlan['ends']; ext: Pt[] } {
    const o = this.roads[r];
    const road = end === 1 ? [...o.points] : reversed(o.points);
    const ends: RoadPlan['ends'] = end === 1 ? [...o.ends] : [o.ends[1], o.ends[0]];
    const E = road[road.length - 1];
    const ls = lengths(line);
    const R = Math.min(APPROACH, ls[ls.length - 1] * 0.4);
    const fwd = pointAt(line, R, ls);
    return { road, ends, ext: [...connector(E, endDir(road), fwd.p, fwd.t), ...line.filter((_, i) => ls[i] > R + 0.01)] };
  }
}

/** `line` with an end just short of the world's edge (`SNAP` m, heading out) carried on past it, so cutting it to the world leaves it a way out. */
function toEdge(line: Pt[], W: number, D: number): Pt[] {
  const out = [...line];
  for (const e of [0, 1] as const) {
    const pts = e ? out : reversed(out);
    const p = pts[pts.length - 1];
    if (p[0] < 0 || p[1] < 0 || p[0] > W || p[1] > D) continue;
    const d = endDir(pts);
    const hit = Math.min(d[0] > 0 ? (W - p[0]) / d[0] : d[0] < 0 ? -p[0] / d[0] : Infinity, d[1] > 0 ? (D - p[1]) / d[1] : d[1] < 0 ? -p[1] / d[1] : Infinity);
    if (hit > SNAP) continue;
    const past: Pt = [p[0] + d[0] * (hit + 2), p[1] + d[1] * (hit + 2)];
    if (e) out.push(past);
    else out.unshift(past);
  }
  return out;
}

/** The area whose box (and `NEAR_AREA` m round it) most of `line` runs in; -1: none. */
function areaOf(draft: NeighbourhoodDraft, line: readonly Pt[]): number {
  const areas = draft.open?.areas ?? [];
  let [best, most] = [-1, 0];
  areas.forEach((a, k) => {
    const n = line.filter(([x, z]) => offRect(a, x, z) <= NEAR_AREA).length;
    if (n > most) [best, most] = [k, n];
  });
  return most * 2 >= line.length ? best : -1;
}

/**
 * The lots (slot indices) and backdrop buildings (`open.backdrop` indices) under `roads` (within
 * their width and `CORRIDOR` m), and the parking spaces (`open.parking` indices).
 */
function inTheWay(draft: NeighbourhoodDraft, t: Templates, roads: readonly RoadPlan[]): { lots: number[]; backdrop: number[]; parking: number[] } {
  const pts = roads.flatMap((r) => evenly(r.points, 1).map((p) => ({ p, half: r.width / 2 + CORRIDOR })));
  const xs = pts.map((q) => q.p[0]);
  const zs = pts.map((q) => q.p[1]);
  const reach = Math.max(0, ...pts.map((q) => q.half));
  const box = [Math.min(...xs) - reach, Math.min(...zs) - reach, Math.max(...xs) + reach, Math.max(...zs) + reach];
  const near = (r: Rect) => r.x <= box[2] && r.x + r.w >= box[0] && r.z <= box[3] && r.z + r.d >= box[1];
  const under = (r: Rect, off: (x: number, z: number) => number) => near(r) && pts.some(({ p, half }) => off(p[0], p[1]) < half);
  const lots = draft.slots.filter((s) => {
    const r = plotRect(t, s);
    return under(r, (x, z) => offRect(r, x, z));
  });
  const backdrop = (draft.open?.backdrop ?? []).flatMap((b, i) => {
    const bx = b.poly.map((q) => q[0]);
    const bz = b.poly.map((q) => q[1]);
    const r = { x: Math.min(...bx), z: Math.min(...bz), w: Math.max(...bx) - Math.min(...bx), d: Math.max(...bz) - Math.min(...bz) };
    return under(r, (x, z) => offPoly(b.poly, x, z)) ? [i] : [];
  });
  const parking = (draft.open?.parking ?? []).flatMap((q, i) => {
    const r = spaceOf(q);
    return under(r, (x, z) => offRect(r, x, z)) ? [i] : [];
  });
  return { lots: lots.map((s) => s.index), backdrop, parking };
}

/** How far each dead end (not a trail's) of roads `which` reaches past the last road meeting it (m), the longest. */
function longestCourt(roads: readonly RoadPlan[], which: readonly number[]): { road: number; stub: number } | null {
  let worst: { road: number; stub: number } | null = null;
  for (const i of which) {
    const r = roads[i];
    if (r.kind === 'trail' || !r.ends.includes('dead')) continue;
    const met: number[] = [];
    roads.forEach((o, j) => {
      if (j === i) return;
      for (const end of [o.points[0], o.points[o.points.length - 1]]) {
        const near = nearest(r.points, end[0], end[1]);
        if (near.dist <= Math.max(o.width, r.width) / 2 + 1) met.push(near.s);
      }
      // (A road this one's own end meets: there, at that end.)
      for (const [e, p] of [r.points[0], r.points[r.points.length - 1]].entries()) {
        if (nearest(o.points, p[0], p[1]).dist <= Math.max(o.width, r.width) / 2 + 1) met.push(e ? lengthOf(r.points) : 0);
      }
    });
    const L = lengthOf(r.points);
    for (const e of [0, 1] as const) {
      if (r.ends[e] !== 'dead') continue;
      const stub = L - Math.max(0, ...met.map((s) => (e === 1 ? s : L - s)));
      if (!worst || stub > worst.stub) worst = { road: i, stub };
    }
  }
  return worst;
}

/**
 * `draft` with the street `stroke` draws (see the top of this file), its homes (`stroke.fill`, a
 * street or a lane: `lots.ts` `fillStreet`) and names drawn from `stroke.seed`; `content`, for the
 * place's street names and the homes. The new road is the last of the draft's roads (a dead end
 * it carries on keeps its index), and the roads it was split into at its bridges come after it.
 * What it cleared out of its way is in `cleared`.
 */
export function drawStreet(draft: NeighbourhoodDraft, t: Templates, content: Content, stroke: StreetStroke): Edited {
  const [W, D] = [draft.width, draft.depth];
  const trail = stroke.kind === 'trail';
  const width = trail ? TRAIL_WIDTH : ROAD_WIDTH;
  const roads = draft.roads;
  const meet = new Meeting(roads, trail, width);
  const named = (r: number) => roads[r].name ?? 'a road';

  // ---- the line, cut to the world ----
  const raw = stroke.points.filter((p, i) => Number.isFinite(p[0]) && Number.isFinite(p[1]) && (i === 0 || Math.hypot(p[0] - stroke.points[i - 1][0], p[1] - stroke.points[i - 1][1]) > 0.5));
  if (raw.length < 2) return refused('a street is drawn with two points or more');
  const cut = cropRoads([{ points: toEdge(curve(evenly(rounded(simplify(raw, SIMPLIFY)), CTRL_APART)), W, D), width, ends: ['dead', 'dead'] }], 0, 0, W, D);
  if (cut.length !== 1) return refused(cut.length ? 'it leaves the world other than by its end' : 'it is off the world');
  let line = cut[0].points;
  if (lengthOf(line) < SHORTEST) return refused(`it is too short (${SHORTEST} m at least)`);

  // ---- its ends: where it starts, the first road it reaches, where it ends ----
  let start: End = cut[0].ends[0] === 'out' ? { kind: 'out' } : meet.snap(line[0]);
  const hit = meet.contact(line, start.kind === 'join' || start.kind === 'extend' ? start.road : -1);
  let end: End;
  if (hit) {
    if (!meet.joinable(hit.road)) return refused(roads[hit.road].island ? 'it runs into the roundabout' : `it crosses ${roads[hit.road].name ?? 'a footpath'}`);
    line = line.slice(0, hit.i + 1);
    const snapped = meet.snap(line[line.length - 1]);
    end = snapped.kind === 'free' ? { kind: 'join', road: hit.road } : snapped;
  } else end = cut[0].ends[1] === 'out' ? { kind: 'out' } : meet.snap(line[line.length - 1]);
  if (lengthOf(line) < SHORTEST) return refused(`it is too short (${SHORTEST} m at least)`);
  // (A dead end it carries on is its start.)
  if (end.kind === 'extend' && start.kind !== 'extend') {
    [start, end] = [end, start];
    line = reversed(line);
  }
  const base = start.kind === 'extend' ? start.road : -1;
  if (base >= 0 && (end.kind === 'extend' || end.kind === 'join') && end.road === base) return refused(`it would join ${named(base)} to itself`);

  // ---- the ends shaped: T-joins turned to meet their road, a dead end's way carried on ----
  const junctions: { at: Pt; road: number }[] = [];
  for (const [e, j] of [start, end].entries()) {
    if (j.kind !== 'join') continue;
    const joined = meet.joinTo(e ? line : reversed(line), j.road);
    if (typeof joined === 'string') return refused(joined);
    line = e ? joined.pts : reversed(joined.pts);
    junctions.push({ at: joined.at, road: j.road });
  }
  // The new stretch of road (what is checked), and the road it makes: the dead ends it carries on
  // before it (`head`, its dead end last) and after it (`foot`, its dead end first).
  let part = line;
  let head: Pt[] = [];
  let foot: Pt[] = [];
  let ends: RoadPlan['ends'] = [endKind(start), endKind(end)];
  if (start.kind === 'extend') {
    const x = meet.extend(line, start.road, start.end);
    [part, head, ends] = [x.ext, x.road, [x.ends[0], endKind(end)]];
    if (end.kind === 'extend') {
      const y = meet.extend(reversed(part), end.road, end.end);
      [part, foot, ends] = [reversed(y.ext), reversed(y.road), [x.ends[0], y.ends[0]]];
    }
  }
  const made = [...head, ...part.slice(head.length ? 1 : 0), ...foot.slice(1)];

  // ---- the generator's rules ----
  const fromRoad = start.kind === 'join' || start.kind === 'extend' ? start.road : -1;
  const toRoad = end.kind === 'join' || end.kind === 'extend' ? end.road : -1;
  // (Its bends with a little of the roads it carries on.)
  if (bends([...head.slice(-20, -1), ...part, ...foot.slice(1, 20)]).tight) return refused('it bends tighter than 9 m');
  for (const j of junctions) if (!roomAt(meet.index, j.at, [j.road])) return refused(`it meets ${named(j.road)} within ${ROOM} m of another junction`);
  if (junctions.length === 2 && Math.hypot(junctions[0].at[0] - junctions[1].at[0], junctions[0].at[1] - junctions[1].at[1]) < ROOM) return refused(`its two junctions are within ${ROOM} m of each other`);
  const fit: Fit = { box: { w: W, d: D }, margin: -0.5, greens: draft.greens };
  const apart = trail ? TRAIL_APART : undefined;
  if (!clearOf(part, width, roads, meet.index, fit, fromRoad, toRoad, apart)) {
    const ls = lengths(part);
    const L = ls[ls.length - 1];
    if (part.some(([x, z], i) => ls[i] > 4 && L - ls[i] > 4 && draft.greens.some((g) => inside(g, x, z)))) return refused('it runs across a green');
    if (!clearOf(part, width, roads, meet.index, fit, fromRoad, toRoad, 0)) return refused('it runs into another road');
    return refused(`it runs closer than ${apart ?? 18} m alongside another road`);
  }

  // ---- the road it makes: its kind, its area and its name ----
  const was = base >= 0 ? roads[base] : null;
  const merged = start.kind === 'extend' && end.kind === 'extend' ? end.road : -1;
  let kind: Pick<RoadPlan, 'kind' | 'area'> = { kind: 'trail' };
  if (was) kind = { kind: was.kind ?? 'street', ...(was.area === undefined ? {} : { area: was.area }) };
  else if (!trail) {
    const area = areaOf(draft, part);
    kind = area < 0 ? { kind: 'country' } : { kind: stroke.kind === 'lane' ? 'lane' : 'street', area };
  }
  const taken = namesIn(draft);
  const rand = mulberry32(stroke.seed);
  const areaName = kind.area === undefined ? undefined : draft.open?.areas[kind.area]?.name;
  const name = was?.name ?? freshNames(draft, content, rand, 1, taken, areaName)[0];
  const road: RoadPlan = { ...(was ?? {}), points: made, width: was?.width ?? width, ends, ...kind, name };
  // (Where it begins at a road's kerb, not its middle; the roads it is made of left out.)
  const trimmed = trimJoins([...roads.filter((_, i) => i !== base && i !== merged), road]);
  const laid: RoadPlan = { ...road, points: trimmed[trimmed.length - 1].points.map(([x, z]): Pt => [Math.round(x * 100) / 100, Math.round(z * 100) / 100]) };
  const at = base >= 0 ? base : roads.length;
  let next: RoadPlan[] = base >= 0 ? roads.map((r, i) => (i === at ? laid : r)) : [...roads, laid];

  // What is in its way (before the water: a bridge's own check is for what stands there now).
  const way = inTheWay(draft, t, [{ ...laid, points: base >= 0 ? part : laid.points }]);
  if (!stroke.clear && (way.lots.length || way.backdrop.length)) {
    const what = [way.lots.length ? `${way.lots.length} lot${way.lots.length > 1 ? 's' : ''}` : '', way.backdrop.length ? `${way.backdrop.length} building${way.backdrop.length > 1 ? 's' : ''}` : ''].filter(Boolean).join(' and ');
    return refused(`${what} ${way.lots.length + way.backdrop.length > 1 ? 'are' : 'is'} in its way`);
  }
  const gone = new Set(way.lots);
  const plots = draft.slots.filter((s) => !gone.has(s.index)).map((s) => plotRect(t, s));

  // ---- the water: a bridge over each crossing, the last first (the road before keeps its index) ----
  const open = draft.open;
  let walk: Uint8Array | null = null;
  const pieces = [at];
  if (open) {
    const wet = unrle(open.water, W * D);
    const spans = wetSpans(laid.points, wet, W, D);
    if (spans.length && trail) return refused('a footpath does not cross the water');
    // (Crossings close together are one bridge.)
    const crossings: [number, number][] = [];
    for (const s of spans) {
      const last = crossings[crossings.length - 1];
      if (last && s[0] - last[1] < ONE_CROSSING) last[1] = s[1];
      else crossings.push([s[0], s[1]]);
    }
    const others = roads.filter((_, i) => i !== base && i !== merged);
    for (const span of crossings.reverse()) {
      const r = next[at];
      const why = bridgeable(r.points, span, others, plots);
      if (why) return refused(why);
      const L = lengthOf(r.points);
      const [a, b] = bridgeStretch(L, span);
      if ((a === 0 && r.ends[0] !== 'join') || (b === L && r.ends[1] !== 'join')) return refused('a bridge needs a road on at each end: carry it on past the water');
      const [after] = b < L ? freshNames(draft, content, rand, 1, taken, areaName) : [undefined];
      const had = next.length;
      const bridged = withBridge(next, at, span, walk ?? wet, W, D, { bridge: bridgeName(name, taken), after });
      [next, walk] = [bridged.roads, bridged.walk];
      for (let k = had; k < next.length; k++) pieces.push(k);
      const deck = next[a > 0 ? had : at].bridge!;
      if (deck[1] - deck[0] > BRIDGE_SPAN) return refused(`its bridge would be ${Math.round(deck[1] - deck[0])} m long (${BRIDGE_SPAN} m at most)`);
    }
    // Nothing of it over the water but its decks, and onto and off them no tighter than it may bend.
    for (const i of pieces) {
      if (next[i].bridge) continue;
      if (wetSpans(next[i].points, wet, W, D).length) return refused('it runs over the water where it would not be a bridge');
      if (pieces.length > 1 && bends(next[i].points).tight) return refused('it would turn onto its bridge tighter than 9 m');
    }
  }
  if (merged >= 0) next = next.filter((_, i) => i !== merged);
  const fresh = pieces.map((i) => (merged >= 0 && i > merged ? i - 1 : i));

  // ---- a court no longer than the generator's, and one network ----
  const court = longestCourt(next, fresh);
  if (court && court.stub > COURT + 0.5) return refused(`it ends ${Math.round(court.stub)} m past the last road it meets: a dead end reaches ${COURT} m at most, so end it on a road`);
  if (!oneNetwork(next, new Set())) return refused('it joins none of the roads');

  // ---- the town with it ----
  const parkingGone = new Set(way.parking);
  let drawn: NeighbourhoodDraft = {
    ...draft,
    roads: next,
    ...(open ? { open: { ...open, ...(walk ? { water: rle(walk) } : {}), parking: open.parking.filter((_, i) => !parkingGone.has(i)) } } : {}),
  };
  drawn = removeLots(drawn, gone, new Set(way.backdrop));
  if (stroke.fill && !trail) {
    for (const [k, i] of fresh.entries()) {
      const r = drawn.roads[i];
      if (r.bridge || r.area === undefined) continue;
      const filled = fillStreet(drawn, t, content, i, (stroke.seed + k * 0x9e3779b1) >>> 0);
      if ('draft' in filled) drawn = filled.draft;
    }
  }
  return { draft: drawn, cleared: { lots: way.lots, backdrop: way.backdrop } };
}

/**
 * What removing road `road` from `draft` takes with it: the roads left joined to no way out of the
 * world (`roads`, with `road` itself), the lots whose way in is on one of them and the outdoor
 * places at a gone trail's end (`lots`, slot indices), the backdrop buildings fronting them
 * (`backdrop`), the parking by them (`parking`); or why it may not go: no road out of the world
 * would be left, the roads left would be two networks each with its own way out, a country road
 * it met would be left a dead end longer than `COURT`, or its lots are the town's last homes or its
 * last park (`lots.ts` `removalRefused`).
 */
export function whatGoes(draft: NeighbourhoodDraft, t: Templates, road: number): { roads: number[]; lots: number[]; backdrop: number[]; parking: number[] } | { refused: string } {
  const roads = draft.roads;
  if (!Number.isInteger(road) || road < 0 || road >= roads.length) return { refused: 'there is no such road' };
  const nets = networks(roads, new Set([road]));
  const leaves = new Set([...nets.entries()].filter(([i]) => roads[i].ends.includes('out')).map(([, n]) => n));
  if (!leaves.size) return { refused: `without ${roads[road].name ?? 'it'} no road would lead out of the world` };
  if (leaves.size > 1) return { refused: `without ${roads[road].name ?? 'it'} the roads would be cut in two` };
  const gone = new Set([road, ...[...nets.entries()].filter(([, n]) => !leaves.has(n)).map(([i]) => i)]);
  // (A country road it leaves ending in a dead end reaches no further past the last road it meets
  // than a drawn one may: no taxi turns round in a country road. A street or a lane may be left
  // longer, the sim turning round in any of theirs; the Problems list says so.)
  const left = withoutRoads(roads, gone);
  const court = longestCourt(left.roads, left.ended.filter((i) => left.roads[i].kind === 'country'));
  if (court && court.stub > COURT + 0.5) return { refused: `without ${roads[road].name ?? 'it'}, ${left.roads[court.road].name ?? 'a road'} would end ${Math.round(court.stub)} m past the last road it meets (a dead end reaches ${COURT} m at most)` };
  const open = draft.open;
  const rects = draft.slots.map((s) => plotRect(t, s));
  const entries = surfacePlots({ plots: rects, meta: { roads, facing: draft.slots.map((s) => s.facing), backdrop: open?.backdrop ?? [] } });
  const trailEnds = [...gone].filter((i) => roads[i].kind === 'trail').flatMap((i) => [roads[i].points[0], roads[i].points[roads[i].points.length - 1]]);
  const lots = draft.slots.filter((s, n) => (entries[n].entry && gone.has(entries[n].entry!.road)) || (s.kind === 'outdoor' && trailEnds.some(([x, z]) => offRect(rects[n], x, z) < TRAIL_END))).map((s) => s.index);
  const backdrop = (open?.backdrop ?? []).flatMap((_, i) => {
    const e = entries[rects.length + i]?.entry;
    return e && gone.has(e.road) ? [i] : [];
  });
  // (The town keeps its last home and its last park, as a lot removed on its own does.)
  const kept = removalRefused(draft, new Set(lots));
  if (kept) return { refused: `${kept.replace(/\.$/, '')}: without ${roads[road].name ?? 'it'} it would go` };
  const index = new RoadIndex(roads);
  const parking = (open?.parking ?? []).flatMap((p, i) => {
    const near = index.nearest(p.x, p.z, ROAD_WIDTH / 2 + 6);
    return near && gone.has(near.road) ? [i] : [];
  });
  return { roads: [...gone].sort((a, b) => a - b), lots, backdrop, parking };
}

/** `roads` without those in `gone`; a join left with nothing to join is a dead end (`ended`: those roads, by their index in what is left). */
function withoutRoads(roads: readonly RoadPlan[], gone: ReadonlySet<number>): { roads: RoadPlan[]; ended: number[] } {
  const kept = roads.filter((_, i) => !gone.has(i));
  const ended: number[] = [];
  const left = kept.map((r, i) => {
    const ends = r.ends.map((e, k) => {
      if (e !== 'join') return e;
      const p = k ? r.points[r.points.length - 1] : r.points[0];
      return kept.some((o, j) => j !== i && nearest(o.points, p[0], p[1]).dist <= Math.max(o.width, r.width) / 2 + 1) ? e : 'dead';
    }) as RoadPlan['ends'];
    if (ends[0] === r.ends[0] && ends[1] === r.ends[1]) return r;
    ended.push(i);
    return { ...r, ends };
  });
  return { roads: left, ended };
}

/**
 * `draft` without road `road`, its lots and parking, and any road it leaves joined to nothing; refused
 * if no road out of the world would be left, or it would take the town's last home or park (see `whatGoes`). A road that met it in a T and stays
 * now ends there in a dead end; the water under a bridge that goes is water again.
 */
export function removeRoad(draft: NeighbourhoodDraft, t: Templates, road: number): Edited {
  const goes = whatGoes(draft, t, road);
  if ('refused' in goes) return goes;
  const { roads } = withoutRoads(draft.roads, new Set(goes.roads));
  const open = draft.open;
  let next: NeighbourhoodDraft = { ...draft, roads };
  if (open) {
    const [W, D] = [draft.width, draft.depth];
    const parking = new Set(goes.parking);
    let water = open.water;
    const bridges = goes.roads.filter((i) => draft.roads[i].bridge);
    if (bridges.length) {
      const lakes = open.zones.filter((z) => z.kind === 'water').map((z) => z.poly);
      const wetAt = (x: number, z: number) => lakes.some((poly) => inside(poly, x, z));
      let walk = unrle(water, W * D);
      for (const i of bridges) walk = withoutDeck(draft.roads[i], walk, wetAt, W, D);
      water = rle(walk);
    }
    next = { ...next, open: { ...open, water, parking: open.parking.filter((_, i) => !parking.has(i)) } };
  }
  return { draft: removeLots(next, new Set(goes.lots), new Set(goes.backdrop)), cleared: { lots: goes.lots, backdrop: goes.backdrop } };
}

/**
 * `draft` with `what` called `name` (spaces tidied): a road's houses keep their numbers along it
 * with the new name; a place's public lot is called as it is; the main town (area 0) is the town.
 * Refused for no name, a name over 40 letters, or one another road, place, village or the town has.
 */
export function rename(draft: NeighbourhoodDraft, what: Named, name: string): Edited {
  const wanted = name.trim().replace(/\s+/g, ' ');
  if (!wanted) return refused('a name needs a letter or two');
  if (wanted.length > 40) return refused('a name is 40 letters at most');
  const open = draft.open;
  const areas = open?.areas ?? [];
  const places = open?.places ?? [];
  const town = what.kind === 'town' || (what.kind === 'area' && what.area === 0);
  const old = town ? draft.name : what.kind === 'road' ? draft.roads[what.road]?.name : what.kind === 'place' ? places[what.place]?.name : areas[(what as { area: number }).area]?.name;
  if (old === undefined && !(what.kind === 'road' && draft.roads[what.road])) return refused('there is nothing there to name');
  if (wanted === old) return { draft, cleared: { lots: [], backdrop: [] } };
  const taken = namesIn(draft);
  if (taken.has(wanted)) return refused(`${wanted} is taken`);
  if (what.kind === 'road') {
    const roads = draft.roads.map((r, i) => (i === what.road ? { ...r, name: wanted } : r));
    // (Its houses' addresses: their number, then its name.)
    const slots = old ? draft.slots.map((s) => (s.name.endsWith(` ${old}`) && /^\d+$/.test(s.name.slice(0, -old.length - 1)) ? { ...s, name: `${s.name.slice(0, -old.length)}${wanted}` } : s)) : draft.slots;
    return { draft: { ...draft, roads, slots }, cleared: { lots: [], backdrop: [] } };
  }
  if (!open) return { draft: { ...draft, name: wanted }, cleared: { lots: [], backdrop: [] } };
  if (what.kind === 'place') {
    const place = places[what.place];
    const slots = place.plot === undefined ? draft.slots : draft.slots.map((s) => (s.index === place.plot && s.name === old ? { ...s, name: wanted } : s));
    return { draft: { ...draft, slots, open: { ...open, places: places.map((p, i) => (i === what.place ? { ...p, name: wanted } : p)) } }, cleared: { lots: [], backdrop: [] } };
  }
  const k = town ? 0 : (what as { area: number }).area;
  const renamed = areas.map((a, i) => (i === k ? { ...a, name: wanted } : a));
  return { draft: { ...draft, ...(town ? { name: wanted } : {}), open: { ...open, areas: renamed } }, cleared: { lots: [], backdrop: [] } };
}
