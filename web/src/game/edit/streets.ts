/**
 * Step 2's street edits: a street, lane, avenue, country road or footpath drawn on the town, one-way
 * or not (snapped to the roads it starts and ends at, joining them in a T, bridged over water:
 * `bridges.ts`), a road's kind or way changed (`changeKind`), a road removed with what stands on it
 * (the motorway whole), a road, place, village or the town renamed. Each keeps the
 * generator's own rules (`roads.ts` `clearOf`, `roomAt`, `bends`).
 *
 * A drawn street:
 * - is the player's points simplified (`SIMPLIFY` m), its corners rounded (`rounded`), and made a
 *   curve as country roads are (`curve(evenly(...))`);
 * - begins and ends on a road within `SNAP` m of its ends, in a T that begins at that road's kerb
 *   (`trimJoins`), meeting it at 35° or more and square if it comes within 15° of square, no nearer
 *   the road's ends than `NEAR_END` m and slid along it where another junction is too near; or at a
 *   dead end within `SNAP` m, which it then carries on; or at the world's edge (a way out), or in a
 *   dead end of its own;
 * - ends at the first road it reaches (no crossroads: the player goes on from 14 m along), but a
 *   street runs across a footpath (`Meeting.crossable`), which it cuts (`cutTrails`);
 * - is a lane, an avenue or a street of an area where it runs inside its box or within `NEAR_AREA` m
 *   of it, else a country road, unless the player asked for a street, a lane or an avenue
 *   (`StreetStroke.asked`), which it then stays, its area given at the Build step; a footpath is a
 *   trail; an avenue, 14 m wide, keeps that much further from other roads (`clearOf`, `Meeting.touch`);
 * - one-way (`StreetStroke.oneway`), goes the way it is drawn or against it, ends on a road at both
 *   ends, and is refused where it would leave somewhere a car can't reach and come back from
 *   (`oneway.ts` `cutOffRoads`);
 * - never meets nor crosses the motorway or its ramps (they are the generator's: `CROSS_MOTORWAY`);
 * - is refused, with the reason, where it bends tighter than 9 m, runs closer than 18 m alongside
 *   another road (`clearOf`), leaves the world other than by its end, meets a road within 14 m of
 *   another junction (`roomAt`), or joins no road;
 * - crosses water on a bridge (`bridges.ts` `crossWater`: straight, one deck of `DRAWN_SPAN` m at
 *   most, laid through the water's middle and turned squarer where the line drawn won't make one,
 *   its deck carried on over the dip a river lies in, the street split round it) where it may, and
 *   is refused where it may not;
 * - clears the lots under it (`clear`; else it is refused), and the car parks (each whole);
 * - is named, and the roads it is split into at its bridges, with names nobody in town has.
 *
 * The dead ends (the player's choice): a drawn dead end may reach as far past the last road it
 * meets as it is drawn. The sim drives a long one in a street or a lane (the renderer paves a
 * turning circle at every dead end and the taxis turn round in any of a street's or a lane's,
 * `sim-core taxi.rs` `dead_end`), but no taxi turns round in a country road: a drawn country road's
 * piece that ends more than `COURT` m past the last road it meets is made a street of no area (the
 * Build step gives it one). The town's checks of an edited town (`townCheck.ts`, not `generated`)
 * leave a street's or a lane's courts be.
 */

import type { Content } from '../../content/content';
import type { RoadKind, RoadPlan } from '../../core/protocol';
import { onewayOf, ROAD_KIND } from '../../core/roadKinds';
import { cutOffRoads } from '../oneway';
import { bayRect, carParkRect } from '../parking';
import { surfacePlots } from '../../render/shared/builtSurface';
import { crossWater, DRAWN_SPAN, wetSpans, withBridge, withoutDeck } from '../bridges';
import { CTRL_APART, ROAD_WIDTH } from '../countryRouter';
import { rle, simplify, unrle } from '../landMasks';
import { mulberry32 } from '../noise';
import { BRIDGE_IN } from '../macro';
import { bends, clearOf, connector, COURT, cropRoads, curve, endDir, evenly, inside, leftOf, lengths, MEET, nearest, pointAt, RoadIndex, ROOM, roomAt, SIDEWALK, trimJoins, type Fit, type Pt } from '../roads';
import { plotRect, type NeighbourhoodDraft, type Rect, type Templates } from '../town';
import { placeOf, specOf, streetNames } from '../world';
import type { Edited } from './edited';
import { wholeParks, withoutBays } from './ground';
import { fillStreet, removalRefused, removeLots } from './lots';
import { withoutEmptyAreas } from './villages';
import * as pmath from '../pmath';

/**
 * A street being drawn: its points as the player set them, its kind, whether what is in its way is cleared (else it
 * is refused there) and whether it is filled with homes, and the seed its homes and names are drawn from. `asked`:
 * the kind is the player's own choice, so a street or a lane stays one outside the town's and the villages' boxes
 * (with no area: the Build step gives it one); else it is a country road there.
 */
export interface StreetStroke {
  points: readonly Pt[];
  kind: RoadKind;
  asked?: boolean;
  /** A one-way street: cars go the way it is drawn (+1) or against it (-1); absent: both ways. (A dead end it carries on keeps its own way.) */
  oneway?: 1 | -1;
  clear: boolean;
  fill: boolean;
  seed: number;
}

/** What a name is given to: a road, a place (`OpenWorldDraft.places`), a village (an area), or the town. */
export type Named = { kind: 'road'; road: number } | { kind: 'place'; place: number } | { kind: 'area'; area: number } | { kind: 'town' };

/** An end of a drawn street snaps to a road, or a dead end, this near (m, from the road's edge; from the dead end's last point). */
export const SNAP = 10;
/** The Street tool's magnet pulls the pointer to this far past a road's edge (m), well within `SNAP`: where a click by the kerb would be. */
const PULL_IN = 2;
/** A street meets a road at `roads.ts` `MEET` at least, and is made square within this of it. Junctions on one road keep `ROOM` apart (`roomAt`). */
const SQUARE = (15 * Math.PI) / 180;
/** (`roads.ts` `ROOM`, for the street tool's live checks.) */
export { ROOM } from '../roads';
/** A footpath keeps this far from a road alongside it (m); roads keep `clearOf`'s 18 m. */
export const TRAIL_APART = 6;
/** A street drawn this near an area's box (m) is one of its streets. */
const NEAR_AREA = 30;
/** A drawn line is simplified to this (m), and has a point every `CTRL_APART` m (a country road's) before it is made a curve. Widths: `streetWidth`. */
const SIMPLIFY = 5;
/** How much of a drawn line (m) is turned to meet a road, or to carry on a dead end's way. */
const APPROACH = 16;
/** A street's last metres into a junction run straight (m). */
const STRAIGHT_IN = 4;
/** The shortest street drawn (m). */
const SHORTEST = 16;
/** Lots and parking keep this far from a new road's edge (m), else they are in its way. */
const CORRIDOR = 1.5;
/** A street joins a road no nearer its end than this (m): one drawn nearer meets it this far in. */
const NEAR_END = 6;
/** A junction with another within `ROOM` m of it slides along its road by the least of these (m) that leaves room, either way. */
const SLIDES = [4, 8, 12, 16];
/** Wet stretches of a line closer than this (m) are crossed on one bridge (each bridge's straight approaches and the road's turns onto them would overlap). */
export const ONE_CROSSING = 56;
/** An outdoor place stands this near the end of its trail (m: `townCheck.ts` checks 6). */
const TRAIL_END = 6;

const dot = (a: Pt, b: Pt) => a[0] * b[0] + a[1] * b[1];
const unit = (x: number, z: number): Pt => {
  const l = pmath.hypot(x, z) || 1;
  return [x / l, z / l];
};
const lengthOf = (pts: readonly Pt[]) => lengths(pts)[pts.length - 1];
const refused = (why: string): Edited => ({ refused: why });

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

/**
 * Whether the roads but those `without` are one network (as the sim joins them: an end within half
 * the wider road's width and a metre of the other's middle) with a road out of the world on it.
 * (`landFollow.ts` has its own `oneNetwork`, not exported.)
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
const offRect = (r: Rect, x: number, z: number) => pmath.hypot(Math.max(r.x - x, 0, x - r.x - r.w), Math.max(r.z - z, 0, z - r.z - r.d));

/** How far a point is from a polygon (0 inside it). */
function offPoly(poly: readonly Pt[], x: number, z: number): number {
  if (inside(poly, x, z)) return 0;
  let best = Infinity;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const [[ax, az], [bx, bz]] = [poly[j], poly[i]];
    const [dx, dz] = [bx - ax, bz - az];
    const l2 = dx * dx + dz * dz || 1;
    const f = Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / l2));
    best = Math.min(best, pmath.hypot(x - ax - dx * f, z - az - dz * f));
  }
  return best;
}

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
export type End = { kind: 'free' } | { kind: 'out' } | { kind: 'join'; road: number } | { kind: 'extend'; road: number; end: 0 | 1 };

const endKind = (e: End): RoadPlan['ends'][0] => (e.kind === 'free' ? 'dead' : e.kind === 'out' ? 'out' : 'join');
const reversed = (pts: readonly Pt[]): Pt[] => [...pts].reverse();

/** What the Street tool's magnet pulls the pointer to (`Meeting.magnet`): a point on a road's middle, a road's dead end, or the world's edge (`road` -1). */
export interface Magnet {
  at: Pt;
  road: number;
  kind: 'road' | 'dead' | 'edge';
}

/**
 * Whether `draft` has no road a car drives (none at all, or footpaths only), as after Clear all: the first street
 * drawn then has to make the town's way out on its own, from the world's edge to the world's edge (or to a dead end
 * no longer than a court), and the streets after it run from road to road.
 */
export const streetsBare = (draft: Pick<NeighbourhoodDraft, 'roads'>): boolean => !draft.roads.some((r) => r.kind !== 'trail');

/**
 * What a street drawn along `line` is made, of the kind asked (`kind`, `asked`: see `StreetStroke`), not carrying a
 * dead end on: a footpath; a street or a lane of the area it runs in (`areaOf`); outside them a country road, or the
 * street or lane asked for with no area.
 */
export function madeKind(draft: NeighbourhoodDraft, line: readonly Pt[], kind: RoadKind, asked = false): Pick<RoadPlan, 'kind' | 'area'> {
  if (kind === 'trail') return { kind: 'trail' };
  const area = areaOf(draft, line);
  if (area >= 0) return { kind: kind === 'lane' || kind === 'avenue' ? kind : 'street', area };
  return asked && TOWN_KINDS.has(kind) ? { kind } : { kind: 'country' };
}

/** The kinds a street of a town or a village is (with an area, or given one at the Build step). */
const TOWN_KINDS: ReadonlySet<RoadKind> = new Set(['street', 'lane', 'avenue']);

/** Whether a road is the motorway's: a carriageway or a ramp (the generator's alone: `ROAD_KIND.drawable`). */
export const isMotorway = (r: Pick<RoadPlan, 'kind'>) => r.kind === 'motorway' || r.kind === 'ramp';

/** Why a street drawn into the motorway is refused (crossing under an overpass may come later). */
export const CROSS_MOTORWAY = "can't cross the motorway";

/**
 * The town's roads as a drawn street meets them: what it may join (a car's road a car's road, a
 * footpath any; never a roundabout) and what its ends snap to.
 */
export class Meeting {
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
    return !road.island && !isMotorway(road) && (this.trail || road.kind !== 'trail');
  }

  /** Whether the street runs across road `r` rather than meeting it: a footpath, for a street (`cutTrails` cuts it there). */
  crossable(r: number): boolean {
    return !this.trail && this.roads[r].kind === 'trail';
  }

  /** `crossable`, as `roads.ts` `clearOf` and `roomAt` take it (the roads they leave out). */
  readonly skip = (r: number): boolean => this.crossable(r);

  /** How near a road's middle the street's middle comes where it touches it (m: `clearOf`'s). */
  touch(r: number): number {
    return this.roads[r].width / 2 + this.width / 2 + 1;
  }

  /**
   * Where the pointer at `p` is pulled to (the Street tool's magnet), within `reach` m: beside the nearest road it may
   * join (off a bridge's deck), `PULL_IN` m past its edge, where a street's end snaps to it as one set there would
   * (`snap`); a dead end of its own kind (which wins over a road less than `SNAP` m nearer, as `snap` prefers it); or
   * the world's edge (`W` × `D` m). Null: nothing so near, or `p` snaps to a road already. `keep`: whether a point on
   * a road (-1: the edge) may pull at all (the tool leaves out the road a street starts on, near its start).
   */
  magnet(p: Pt, reach: number, W: number, D: number, keep: (at: Pt, road: number) => boolean = () => true): Magnet | null {
    if (this.snap(p).kind !== 'free') return null;
    let best: Magnet | null = null;
    let near = Infinity;
    const take = (at: Pt, road: number, kind: Magnet['kind'], d: number) => {
      if (d < near && keep(at, road)) [best, near] = [{ at, road, kind }, d];
    };
    for (const [r, m] of this.index.around(p[0], p[1], reach)) {
      const road = this.roads[r];
      if (!this.joinable(r) || (road.bridge && m.s > road.bridge[0] - BRIDGE_IN && m.s < road.bridge[1] + BRIDGE_IN)) continue;
      const q = pointAt(road.points, m.s).p;
      const off = road.width / 2 + PULL_IN;
      const k = m.dist > off ? off / m.dist : 1;
      take([q[0] + (p[0] - q[0]) * k, q[1] + (p[1] - q[1]) * k], r, 'road', m.dist);
    }
    for (const [r, road] of this.roads.entries()) {
      if (!this.joinable(r) || road.bridge || (road.kind === 'trail') !== this.trail) continue;
      for (const e of [0, 1] as const) {
        if (road.ends[e] !== 'dead') continue;
        const q = e ? road.points[road.points.length - 1] : road.points[0];
        const d = pmath.hypot(q[0] - p[0], q[1] - p[1]);
        if (d <= reach) take([q[0], q[1]], r, 'dead', d - SNAP);
      }
    }
    const edges: [number, Pt][] = [
      [p[0], [0, p[1]]],
      [W - p[0], [W, p[1]]],
      [p[1], [p[0], 0]],
      [D - p[1], [p[0], D]],
    ];
    for (const [d, at] of edges) if (d >= 0 && d <= reach) take([Math.max(0, Math.min(W, at[0])), Math.max(0, Math.min(D, at[1]))], -1, 'edge', d);
    return best;
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
        const d = pmath.hypot(q[0] - p[0], q[1] - p[1]);
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

  /** The first point of `line` (its index) that touches a road, not counting road `from` near the line's start (`free` m) nor a road it crosses (`crossable`); and that road. */
  contact(line: readonly Pt[], from: number): { i: number; road: number } | null {
    const ls = lengths(line);
    const free = from < 0 ? 0 : this.touch(from) * 1.8 + 1 + SNAP;
    for (let i = 0; i < line.length; i++) {
      for (const [r, m] of this.index.around(line[i][0], line[i][1], this.widest / 2 + this.width / 2 + 1)) {
        if (m.dist >= this.touch(r) || (r === from && ls[i] < free) || this.crossable(r)) continue;
        return { i, road: r };
      }
    }
    return null;
  }

  /**
   * `line` with its last stretch turned to meet road `r` in a T: on the road's middle, nearest
   * where the line ends (but no nearer its ends than `NEAR_END` m), at 35° or more (square within
   * 15° of it), over the last `APPROACH` m; the junction too (`at`); or why not. Where another
   * junction is within `ROOM` m of it, the junction slides along the road, the least of `SLIDES`
   * (16 m at most) either way, over a longer approach, that leaves room and has the line turn no
   * tighter than it may; else it stays, and `drawStreet` refuses it for the room.
   */
  joinTo(line: readonly Pt[], r: number): { pts: Pt[]; at: Pt } | string {
    const o = this.roads[r];
    const name = o.name ?? 'the road';
    const lo = lengths(o.points);
    const Lo = lo[lo.length - 1];
    const E = line[line.length - 1];
    let sT = this.index.along(r, E[0], E[1]);
    if (o.ends[0] !== 'loop') {
      if (Lo < NEAR_END * 2) return `it meets ${name} too near its end`;
      sT = Math.max(NEAR_END, Math.min(Lo - NEAR_END, sT));
    }
    const onDeck = (s: number) => !!o.bridge && s > o.bridge[0] - BRIDGE_IN && s < o.bridge[1] + BRIDGE_IN;
    if (onDeck(sT)) return `it meets ${name} on its bridge`;
    const first = this.meetAt(line, r, sT, APPROACH, lo);
    if (roomAt(this.index, first.at, [r], this.skip)) return first;
    const loop = o.ends[0] === 'loop';
    const slid = SLIDES.flatMap((d) => [sT - d, sT + d])
      .map((s) => (loop ? ((s % Lo) + Lo) % Lo : s))
      .filter((s) => (loop || (s >= NEAR_END && s <= Lo - NEAR_END)) && !onDeck(s));
    for (const s of slid) {
      const shift = Math.abs(s - sT);
      const tried = this.meetAt(line, r, s, APPROACH + shift, lo);
      if (roomAt(this.index, tried.at, [r], this.skip) && !bends(tried.pts.slice(-2 * (APPROACH + shift))).tight) return tried;
    }
    return first;
  }

  /** `line` turned to meet road `r` at `sT` m along it (`lo`: its arc lengths) over its last `approach` m (at most 40 % of it; 60 % for a slid junction): see `joinTo`. */
  private meetAt(line: readonly Pt[], r: number, sT: number, approach: number, lo: number[]): { pts: Pt[]; at: Pt } {
    const o = this.roads[r];
    const { p: T, t: tq } = pointAt(o.points, sT, lo);
    const ls = lengths(line);
    const L = ls[ls.length - 1];
    const R = Math.min(approach, L * (approach > APPROACH ? 0.6 : 0.4));
    const back = pointAt(line, L - R, ls);
    const n = leftOf(tq);
    const sgn = dot([back.p[0] - T[0], back.p[1] - T[1]], n) >= 0 ? 1 : -1;
    const into: Pt = [-n[0] * sgn, -n[1] * sgn];
    const v = endDir(line);
    const along = dot(v, tq);
    const meet = pmath.atan2(Math.max(0, dot(v, into)), Math.abs(along));
    let a = into;
    if (meet < Math.PI / 2 - SQUARE) {
      const k = Math.max(meet, MEET);
      const way = Math.sign(along) || 1;
      a = unit(into[0] * pmath.sin(k) + tq[0] * way * pmath.cos(k), into[1] * pmath.sin(k) + tq[1] * way * pmath.cos(k));
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
export function areaOf(draft: NeighbourhoodDraft, line: readonly Pt[]): number {
  const areas = draft.open?.areas ?? [];
  let [best, most] = [-1, 0];
  areas.forEach((a, k) => {
    const n = line.filter(([x, z]) => offRect(a, x, z) <= NEAR_AREA).length;
    if (n > most) [best, most] = [k, n];
  });
  return most * 2 >= line.length ? best : -1;
}

/**
 * The lots (slot indices) under `roads` (within their width and `CORRIDOR` m), and the parking bays
 * (`open.parking` indices) of every car park whose asphalt or one of whose bays is under them (a car park
 * goes whole: `ground.ts` `wholeParks`).
 */
function inTheWay(draft: NeighbourhoodDraft, t: Templates, roads: readonly RoadPlan[]): { lots: number[]; parking: number[] } {
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
  const open = draft.open;
  if (!open) return { lots: lots.map((s) => s.index), parking: [] };
  const parks = new Set((open.carParks ?? []).flatMap((p, k) => {
    const r = carParkRect(p);
    return under(r, (x, z) => offPoly(p.poly, x, z)) ? [k] : [];
  }));
  const bays = open.parking.flatMap((q, i) => {
    if (q.park !== undefined && parks.has(q.park)) return [i];
    const r = bayRect(q);
    return under(r, (x, z) => offRect(r, x, z)) ? [i] : [];
  });
  return { lots: lots.map((s) => s.index), parking: wholeParks(open, bays) };
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

/** A drawn street's line: the player's points simplified, rounded and made a curve, carried out past the world's edge where an end nears it, and cut to the world; with its ends (`out` where it leaves the world). */
export interface StreetLine {
  points: Pt[];
  ends: RoadPlan['ends'];
}

/** The line `points` make for a road `width` wide on a `W` × `D` m world (see the top of this file), or why there is none. */
export function streetLine(points: readonly Pt[], W: number, D: number, width: number): StreetLine | string {
  const raw = points.filter((p, i) => Number.isFinite(p[0]) && Number.isFinite(p[1]) && (i === 0 || pmath.hypot(p[0] - points[i - 1][0], p[1] - points[i - 1][1]) > 0.5));
  if (raw.length < 2) return 'a street is drawn with two points or more';
  const cut = cropRoads([{ points: toEdge(curve(evenly(rounded(simplify(raw, SIMPLIFY)), CTRL_APART)), W, D), width, ends: ['dead', 'dead'] }], 0, 0, W, D);
  if (cut.length !== 1) return cut.length ? 'it leaves the world other than by its end' : 'it is off the world';
  if (lengthOf(cut[0].points) < SHORTEST) return `it is too short (${SHORTEST} m at least)`;
  return { points: cut[0].points, ends: cut[0].ends };
}

/** How a drawn street's line meets the town's roads (`streetCourse`). */
export interface StreetCourse {
  start: End;
  end: End;
  junctions: { at: Pt; road: number }[];
  part: Pt[];
  head: Pt[];
  foot: Pt[];
  ends: RoadPlan['ends'];
}

/**
 * Where a drawn street's line meets the town's roads (`meet`): what it starts and ends at (the
 * first road it reaches ending it; a dead end it carries on its start), the line turned to meet the
 * roads it joins in a T (`junctions`), and the stretch that is new (`part`) between the dead ends
 * it carries on before it (`head`, its dead end last) and after it (`foot`, its dead end first),
 * with the made road's ends; or why not.
 */
export function streetCourse(meet: Meeting, shaped: StreetLine): StreetCourse | string {
  const roads = meet.roads;
  const named = (r: number) => roads[r].name ?? 'a road';
  let line = shaped.points;

  // ---- its ends: where it starts, the first road it reaches, where it ends ----
  let start: End = shaped.ends[0] === 'out' ? { kind: 'out' } : meet.snap(line[0]);
  const hit = meet.contact(line, start.kind === 'join' || start.kind === 'extend' ? start.road : -1);
  let end: End;
  if (hit) {
    if (isMotorway(roads[hit.road])) return CROSS_MOTORWAY;
    if (!meet.joinable(hit.road)) return 'it runs into the roundabout';
    line = line.slice(0, hit.i + 1);
    const snapped = meet.snap(line[line.length - 1]);
    end = snapped.kind === 'free' ? { kind: 'join', road: hit.road } : snapped;
  } else end = shaped.ends[1] === 'out' ? { kind: 'out' } : meet.snap(line[line.length - 1]);
  if (lengthOf(line) < SHORTEST) return `it is too short (${SHORTEST} m at least)`;
  // (A dead end it carries on is its start.)
  if (end.kind === 'extend' && start.kind !== 'extend') {
    [start, end] = [end, start];
    line = reversed(line);
  }
  const base = start.kind === 'extend' ? start.road : -1;
  if (base >= 0 && (end.kind === 'extend' || end.kind === 'join') && end.road === base) return `it would join ${named(base)} to itself`;

  // ---- the ends shaped: T-joins turned to meet their road, a dead end's way carried on ----
  const junctions: { at: Pt; road: number }[] = [];
  for (const [e, j] of [start, end].entries()) {
    if (j.kind !== 'join') continue;
    const joined = meet.joinTo(e ? line : reversed(line), j.road);
    if (typeof joined === 'string') return joined;
    line = e ? joined.pts : reversed(joined.pts);
    junctions.push({ at: joined.at, road: j.road });
  }
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
  return { start, end, junctions, part, head, foot, ends };
}

/** The width of a drawn road of `kind` (m): a footpath's, or its kind's (`ROAD_KIND`) and no narrower than every road's in a big world. */
export const streetWidth = (kind: RoadKind) => (kind === 'trail' ? ROAD_KIND.trail.width : Math.max(ROAD_KIND[kind].width, ROAD_WIDTH));

/** Why a one-way street is refused where it would leave somewhere a car can't get back from (`oneway.ts` `cutOffRoads`). */
export const CUT_OFF = 'some homes could no longer be reached by car';
/** Why a one-way street is refused with a dead end (no car could turn back out of it); while drawing, a street not finished yet (`streetLive.ts` `unfinished`). */
export const ONE_WAY_ENDS = 'a one-way street ends on a road at both ends';

/** Whether road `points` runs the way the player drew `drawn` (+1) or against it (-1): which of its ends the drawing began nearer. */
function drawnWay(drawn: readonly Pt[], points: readonly Pt[]): 1 | -1 {
  const ls = lengths(points);
  const [a, b] = [drawn[0], drawn[drawn.length - 1]];
  return nearest(points, a[0], a[1], ls).s <= nearest(points, b[0], b[1], ls).s ? 1 : -1;
}

/**
 * `roads` (kept from a list by their old indices, `from`) with what each says it passes over or under (`crosses`)
 * numbered as they now are, those of roads gone left out.
 */
function withCrosses(roads: RoadPlan[], from: readonly number[]): RoadPlan[] {
  if (!roads.some((r) => r.crosses)) return roads;
  const now = new Map(from.map((old, i) => [old, i]));
  return roads.map((r) => {
    if (!r.crosses) return r;
    const crosses = r.crosses.flatMap((c) => (now.has(c.road) ? [{ ...c, road: now.get(c.road)! }] : []));
    if (crosses.length === r.crosses.length && crosses.every((c, k) => c.road === r.crosses![k].road)) return r;
    const { crosses: _gone, ...rest } = r;
    return crosses.length ? { ...rest, crosses } : rest;
  });
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
  const width = streetWidth(stroke.kind);
  const roads = draft.roads;
  const meet = new Meeting(roads, trail, width);
  const named = (r: number) => roads[r].name ?? 'a road';

  const shaped = streetLine(stroke.points, W, D, width);
  if (typeof shaped === 'string') return refused(shaped);
  const course = streetCourse(meet, shaped);
  if (typeof course === 'string') return refused(course);
  const { start, end, junctions, part, head, foot, ends } = course;
  const base = start.kind === 'extend' ? start.road : -1;
  const made = [...head, ...part.slice(head.length ? 1 : 0), ...foot.slice(1)];

  // ---- the generator's rules ----
  const fromRoad = start.kind === 'join' || start.kind === 'extend' ? start.road : -1;
  const toRoad = end.kind === 'join' || end.kind === 'extend' ? end.road : -1;
  // (Its bends with a little of the roads it carries on.)
  if (bends([...head.slice(-20, -1), ...part, ...foot.slice(1, 20)]).tight) return refused('it bends tighter than 9 m');
  for (const j of junctions) if (!roomAt(meet.index, j.at, [j.road], meet.skip)) return refused(`it meets ${named(j.road)} within ${ROOM} m of another junction`);
  if (junctions.length === 2 && pmath.hypot(junctions[0].at[0] - junctions[1].at[0], junctions[0].at[1] - junctions[1].at[1]) < ROOM) return refused(`its two junctions are within ${ROOM} m of each other`);
  const fit: Fit = { box: { w: W, d: D }, margin: -0.5, greens: draft.greens };
  const apart = trail ? TRAIL_APART : undefined;
  if (!clearOf(part, width, roads, meet.index, fit, fromRoad, toRoad, apart, meet.skip)) {
    const ls = lengths(part);
    const L = ls[ls.length - 1];
    if (part.some(([x, z], i) => ls[i] > 4 && L - ls[i] > 4 && draft.greens.some((g) => inside(g, x, z)))) return refused('it runs across a green');
    if (!clearOf(part, width, roads, meet.index, fit, fromRoad, toRoad, 0, meet.skip)) return refused('it runs into another road');
    return refused(`it runs closer than ${apart ?? 18} m alongside another road`);
  }

  // ---- the road it makes: its kind, its area and its name ----
  const was = base >= 0 ? roads[base] : null;
  const merged = start.kind === 'extend' && end.kind === 'extend' ? end.road : -1;
  const kind: Pick<RoadPlan, 'kind' | 'area'> = was ? { kind: was.kind ?? 'street', ...(was.area === undefined ? {} : { area: was.area }) } : madeKind(draft, part, stroke.kind, stroke.asked);
  const taken = namesIn(draft);
  const rand = mulberry32(stroke.seed);
  const areaName = kind.area === undefined ? undefined : draft.open?.areas[kind.area]?.name;
  const name = was?.name ?? freshNames(draft, content, rand, 1, taken, areaName)[0];
  // (One-way the way it is drawn: its points may run the other way, from a dead end it carries on.)
  const oneWay = was || trail || !stroke.oneway ? null : ((stroke.oneway * drawnWay(stroke.points, made)) as 1 | -1);
  const road: RoadPlan = { ...(was ?? {}), points: made, width: was?.width ?? width, ends, ...kind, name, ...(oneWay ? { oneway: oneWay } : {}) };
  if (onewayOf(road) && ends.includes('dead')) return refused(ONE_WAY_ENDS);
  // (Where it begins at a road's kerb, not its middle; the roads it is made of left out.)
  const trimmed = trimJoins([...roads.filter((_, i) => i !== base && i !== merged), road]);
  const laid: RoadPlan = { ...road, points: trimmed[trimmed.length - 1].points.map(([x, z]): Pt => [Math.round(x * 100) / 100, Math.round(z * 100) / 100]) };
  const at = base >= 0 ? base : roads.length;
  let next: RoadPlan[] = base >= 0 ? roads.map((r, i) => (i === at ? laid : r)) : [...roads, laid];

  // What is in its way (before the water: a bridge's own check is for what stands there now).
  const way = inTheWay(draft, t, [{ ...laid, points: base >= 0 ? part : laid.points }]);
  if (!stroke.clear && way.lots.length) return refused(`${way.lots.length} lot${way.lots.length > 1 ? 's are' : ' is'} in its way`);
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
    // (Not the footpaths: one it crosses passes under its deck or is cut, `cutTrails`.)
    const others = roads.filter((_, i) => i !== base && i !== merged && !meet.crossable(i));
    for (const span of crossingsOf(spans).reverse()) {
      const r = next[at];
      const crossed = crossWater(r.points, r.ends, span, { wet, W, D, roads: others, plots, land: open.land, most: DRAWN_SPAN });
      if (typeof crossed === 'string') return refused(crossed);
      const [after] = crossed.after ? freshNames(draft, content, rand, 1, taken, areaName) : [undefined];
      const had = next.length;
      const bridged = withBridge(next, at, span, walk ?? wet, W, D, { bridge: bridgeName(name, taken), after }, crossed.fit);
      [next, walk] = [bridged.roads, bridged.walk];
      for (let k = had; k < next.length; k++) pieces.push(k);
    }
    // Nothing of it over the water but its decks, and onto and off them no tighter than it may bend.
    for (const i of pieces) {
      if (next[i].bridge) continue;
      if (wetSpans(next[i].points, wet, W, D).length) return refused('it runs over the water where it would not be a bridge');
      if (pieces.length > 1 && bends(next[i].points).tight) return refused('it would turn onto its bridge tighter than 9 m');
    }
  }
  // (Its bridges one-way with it: their decks run the way it does.)
  if (oneWay) for (const i of pieces) if (next[i].oneway !== oneWay) next[i] = { ...next[i], oneway: oneWay };
  if (merged >= 0) next = withCrosses(next.filter((_, i) => i !== merged), next.flatMap((_, i) => (i === merged ? [] : [i])));
  const fresh = pieces.map((i) => (merged >= 0 && i > merged ? i - 1 : i));

  // ---- a dead end anywhere: a street's or a lane's as it is (the sim turns round in any of theirs); a country road's past a court a street of no area (no taxi turns round in a country road) ----
  for (const i of fresh) {
    const r = next[i];
    if (r.kind !== 'country' || r.bridge) continue;
    const court = longestCourt(next, [i]);
    if (court && court.stub > COURT + 0.5) {
      const { area: _area, ...rest } = r;
      next[i] = { ...rest, kind: 'street' };
    }
  }
  if (!oneNetwork(next, new Set())) return refused('it joins none of the roads');
  if (fresh.some((i) => onewayOf(next[i])) && cutOffRoads(next).length > cutOffRoads(roads).length) return refused(CUT_OFF);

  // ---- the footpaths it crosses, cut to meet it in a T either side (one it crosses on a bridge's deck passes under it) ----
  if (!trail) {
    const kept = draft.slots.filter((s) => !gone.has(s.index) && s.kind === 'outdoor').map((s) => plotRect(t, s));
    next = cutTrails(next, fresh, kept, taken);
  }

  // ---- the town with it ----
  let drawn: NeighbourhoodDraft = {
    ...draft,
    roads: next,
    ...(open ? { open: { ...open, ...(walk ? { water: rle(walk) } : {}), ...withoutBays(open, way.parking) } } : {}),
  };
  drawn = removeLots(drawn, gone);
  if (stroke.fill && !trail) {
    for (const [k, i] of fresh.entries()) {
      const r = drawn.roads[i];
      if (r.bridge || r.area === undefined) continue;
      const filled = fillStreet(drawn, t, content, i, (stroke.seed + k * 0x9e3779b1) >>> 0);
      if ('draft' in filled) drawn = filled.draft;
    }
  }
  return { draft: drawn, cleared: { lots: way.lots }, made: fresh };
}

/** The kinds a road may be made on its card (`changeKind`): the car roads the Street tool draws. */
export const CHANGE_KINDS: readonly RoadKind[] = ['street', 'lane', 'avenue', 'country'];

/**
 * `draft` with road `road` made a `kind` (`CHANGE_KINDS`), one-way (`oneway`: +1 the way its points run, -1 against
 * them, 0 both ways; absent: as it is). Made wider (an avenue), it has to be clear of every road but those meeting
 * or crossing it (`clearOf`), and the lots under it (within its width and `CORRIDOR` m) are cleared (`clear`) or it
 * is refused; its car parks in the way go. Made narrower, it just narrows. Either way the roads that join it in a T
 * begin at its new kerb. A street, a lane or an avenue is of the area it is in (or none: the Build step gives it
 * one), a country road of none. Refused: on the motorway and its ramps, a footpath made a road or a road a footpath,
 * a bridge made another kind, a road under the motorway made wider, a country road left a dead end past a court, a
 * one-way road with a dead end, or one-way roads that leave somewhere a car can no longer get to and back from
 * (`oneway.ts` `cutOffRoads`).
 */
export function changeKind(draft: NeighbourhoodDraft, t: Templates, road: number, kind: RoadKind, oneway?: 1 | -1 | 0, clear = false): Edited {
  const roads = draft.roads;
  const r = roads[road];
  if (!r) return refused('there is no such road');
  if (isMotorway(r)) return refused('the motorway stays as it is: rename it or remove it');
  if (!CHANGE_KINDS.includes(kind) && kind !== 'trail') return refused('only the world lays a motorway');
  const was = r.kind ?? 'street';
  const way = oneway === undefined ? (r.oneway ?? 0) : oneway;
  if ((was === 'trail') !== (kind === 'trail')) return refused(was === 'trail' ? 'a footpath stays a footpath: draw a street instead' : 'a road stays a road: draw a footpath instead');
  if (kind === 'trail' && way) return refused('a footpath has no one way');
  if (kind === was && way === (r.oneway ?? 0)) return { draft, cleared: { lots: [] } };
  if (r.bridge && kind !== was) return refused('a bridge stays as it is built');
  if (way && r.ends.includes('dead')) return refused(ONE_WAY_ENDS);
  const width = r.bridge ? r.width : streetWidth(kind);
  const wider = width > r.width + 0.01;
  const name = r.name ?? 'it';
  if (wider && r.crosses?.length) return refused(`${name} passes under the motorway, where it can't be made wider`);

  // ---- the roads meeting it or crossing it (its own new width is theirs to begin at), and room to be wider ----
  const own = new RoadIndex([r]);
  const meets = new Set(roads.flatMap((o, j) => (j !== road && o.points.some(([x, z]) => own.distance(0, x, z, r.width / 2 + 1) <= r.width / 2 + 1) ? [j] : [])));
  for (const [e, p] of [r.points[0], r.points[r.points.length - 1]].entries()) {
    if (r.ends[e] !== 'join') continue;
    const near = new RoadIndex(roads).nearest(p[0], p[1], ROAD_WIDTH);
    if (near) meets.add(near.road);
  }
  if (wider) {
    const fit: Fit = { box: { w: draft.width, d: draft.depth }, margin: -0.5, greens: draft.greens };
    // (Only its new kerbs: it kept its roads apart as it was. Looked for as far off as the widest road's kerb comes.)
    if (!clearOf(r.points, width, roads, new RoadIndex(roads), fit, -1, -1, width / 2 + 1, (j) => j === road || meets.has(j))) return refused(`there is no room for ${name} to be ${width} m wide: another road or a green is too near`);
  }
  const area = TOWN_KINDS.has(kind) ? (r.area ?? areaOf(draft, r.points)) : -1;
  const { area: _area, oneway: _way, ...plain } = r;
  const made: RoadPlan = { ...plain, kind, width, ...(area >= 0 ? { area } : {}), ...(way ? { oneway: way } : {}) };
  const next = roads.map((o, j) => (j === road ? made : meets.has(j) ? atKerb(o, made) : o));
  if (kind === 'country' && r.ends.includes('dead')) {
    const court = longestCourt(next, [road]);
    if (court && court.stub > COURT + 0.5) return refused(`a country road's dead end reaches ${COURT} m at most, and ${name} ends ${Math.round(court.stub)} m past the last road it meets`);
  }
  if (way !== (r.oneway ?? 0)) {
    const before = new Set(cutOffRoads(roads));
    if (cutOffRoads(next).some((i) => !before.has(i))) return refused(CUT_OFF);
  }

  // ---- what is in the way of it wider ----
  const inWay = wider ? inTheWay(draft, t, [made]) : { lots: [], parking: [] };
  if (!clear && inWay.lots.length) return refused(`${inWay.lots.length} lot${inWay.lots.length > 1 ? 's are' : ' is'} in its way`);
  const gone = new Set(inWay.lots);
  const kept = gone.size ? removalRefused(draft, gone) : null;
  if (kept) return refused(`${kept.replace(/\.$/, '')}: it would be in the way`);
  const open = draft.open;
  return { draft: removeLots({ ...draft, roads: next, ...(open && inWay.parking.length ? { open: { ...open, ...withoutBays(open, inWay.parking) } } : {}) }, gone), cleared: { lots: inWay.lots } };
}

/**
 * Road `o` with each end that joins road `to` in a T begun at its kerb again (as `roads.ts` `trimJoins` begins it):
 * carried on along its way onto `to`'s middle, then cut back to its kerb (a footpath's to just inside its edge, as
 * `cutTrails` cuts one).
 */
function atKerb(o: RoadPlan, to: RoadPlan): RoadPlan {
  const index = new RoadIndex([to]);
  const dist = ([x, z]: Pt) => index.distance(0, x, z, to.width + ROAD_WIDTH);
  const kerb = o.kind === 'trail' ? to.width / 2 - TRAIL_IN : to.width / 2 - SIDEWALK;
  const L = lengthOf(to.points);
  let pts = o.points;
  for (const e of [0, 1] as const) {
    if (o.ends[e] !== 'join') continue;
    const line = e ? [...pts] : reversed(pts);
    const end = line[line.length - 1];
    // (A T: by its middle, not end to end with it, as a bridge's pieces are.)
    const s = index.along(0, end[0], end[1]);
    if (!(dist(end) <= to.width / 2 + ROAD_WIDTH) || (to.ends[0] !== 'loop' && (s < 1 || s > L - 1))) continue;
    // (On along its way to the middle, a point every half metre, while it nears it.)
    const d = endDir(line);
    let near = dist(end);
    for (let k = 1; k <= 60 && near > 0.1; k++) {
      const q: Pt = [end[0] + d[0] * k * 0.5, end[1] + d[1] * k * 0.5];
      const n = dist(q);
      if (!(n < near)) break;
      line.push(q);
      near = n;
    }
    while (line.length > 2 && dist(line[line.length - 2]) < kerb) line.pop();
    if (dist(line[line.length - 1]) < kerb - 0.01) {
      // (The kerb between its last two points: its end there.)
      const [a, b] = [line[line.length - 2], line[line.length - 1]];
      const [da, db] = [dist(a), dist(b)];
      const f = da > db ? (da - kerb) / (da - db) : 1;
      line[line.length - 1] = [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f];
    }
    const rounded = line.map(([x, z]): Pt => [Math.round(x * 100) / 100, Math.round(z * 100) / 100]);
    pts = e ? rounded : reversed(rounded);
  }
  return pts === o.points ? o : { ...o, points: pts };
}

/** A line's wet stretches (`wetSpans`) as the crossings bridged: those closer than `ONE_CROSSING` m one. */
export function crossingsOf(spans: readonly (readonly [number, number])[]): [number, number][] {
  const out: [number, number][] = [];
  for (const s of spans) {
    const last = out[out.length - 1];
    if (last && s[0] - last[1] < ONE_CROSSING) last[1] = s[1];
    else out.push([s[0], s[1]]);
  }
  return out;
}

/** A footpath cut by a street ends this far (m) inside the street's edge (where the sim joins it: within a metre of its edge); a piece of it shorter than `TRAIL_LEAST` m goes, but the one at an outdoor place. */
const TRAIL_IN = 0.5;
const TRAIL_LEAST = 4;
/** A footpath is looked at every this far (m) along it for where a street runs over it. */
const TRAIL_STEP = 0.25;

/**
 * `roads` with each footpath that the roads `made` (a street just drawn, its pieces) run over cut
 * there: split into pieces, each ending `TRAIL_IN` m inside the street's edge (a T: the sim joins it
 * there), a piece shorter than `TRAIL_LEAST` m left out unless an outdoor place (`outdoor`, their
 * rectangles) is at its end. The piece with the footpath's last point (its outdoor place's end) keeps
 * its index and name; the others go at the end of the list, named after it with a number nobody in
 * town has (`taken`, added to). Where the street crosses it on a bridge's deck it passes under it, whole.
 */
export function cutTrails(roads: readonly RoadPlan[], made: readonly number[], outdoor: readonly Rect[], taken: Set<string>): RoadPlan[] {
  const streets = made.map((i) => roads[i]).filter((r) => r.kind !== 'trail');
  if (!streets.length) return [...roads];
  const index = new RoadIndex(streets);
  const reach = Math.max(...streets.map((r) => r.width)) / 2;
  const out = [...roads];
  const isMade = new Set(made);
  roads.forEach((r, j) => {
    if (r.kind !== 'trail' || isMade.has(j) || r.points.length < 2) return;
    const ls = lengths(r.points);
    const L = ls[ls.length - 1];
    // Where along it a street runs over it (within its width less `TRAIL_IN`), and whether on a deck.
    const runs: { from: number; to: number; deck: boolean; closest: number }[] = [];
    for (let s = 0; s <= L + 1e-6; s += TRAIL_STEP) {
      const p = pointAt(r.points, Math.min(s, L), ls).p;
      let hit: { dist: number; deck: boolean } | null = null;
      for (const [k, m] of index.around(p[0], p[1], reach)) {
        const st = streets[k];
        if (m.dist >= st.width / 2 - TRAIL_IN || (hit && hit.dist <= m.dist)) continue;
        hit = { dist: m.dist, deck: !!st.bridge && m.s >= st.bridge[0] && m.s <= st.bridge[1] };
      }
      if (!hit) continue;
      const last = runs[runs.length - 1];
      if (last && s - last.to <= TRAIL_STEP * 1.5) {
        last.to = s;
        if (hit.dist < last.closest) [last.closest, last.deck] = [hit.dist, hit.deck];
      } else runs.push({ from: s, to: s, deck: hit.deck, closest: hit.dist });
    }
    const cuts = runs.filter((c) => !c.deck);
    if (!cuts.length) return;
    // The pieces between the cuts, each end at a cut a join.
    const end = r.points[r.points.length - 1];
    const atPlace = outdoor.some((q) => pmath.hypot(Math.max(q.x - end[0], 0, end[0] - q.x - q.w), Math.max(q.z - end[1], 0, end[1] - q.z - q.d)) <= TRAIL_END);
    const bounds: [number, number][] = [];
    let from = 0;
    for (const c of cuts) {
      bounds.push([from, Math.max(0, c.from - TRAIL_STEP)]);
      from = Math.min(L, c.to + TRAIL_STEP);
    }
    bounds.push([from, L]);
    const pieces = bounds
      .map(([a, b], k) => ({ a, b, k }))
      .filter(({ a, b }) => b - a >= TRAIL_LEAST || (b >= L - 1e-6 && b > a && atPlace));
    if (!pieces.length) return;
    const lastPiece = pieces[pieces.length - 1];
    const cut: RoadPlan[] = pieces.map(({ a, b, k }) => {
      const pts = trailBetween(r.points, a, b, ls);
      const ends: RoadPlan['ends'] = [k === 0 ? r.ends[0] : 'join', k === bounds.length - 1 ? r.ends[1] : 'join'];
      return { ...r, points: pts, ends };
    });
    out[j] = cut[pieces.indexOf(lastPiece)];
    let n = 2;
    for (const piece of cut) {
      if (piece === out[j]) continue;
      while (taken.has(`${r.name ?? 'Footpath'} ${n}`)) n++;
      const name = `${r.name ?? 'Footpath'} ${n}`;
      taken.add(name);
      out.push({ ...piece, name });
    }
  });
  return out;
}

/** The points of a footpath (`pts`) between arc lengths `a` and `b`, rounded as a drawn road's are. */
function trailBetween(pts: readonly Pt[], a: number, b: number, ls: number[]): Pt[] {
  const out: Pt[] = [pointAt(pts, a, ls).p];
  for (let i = 0; i < pts.length; i++) if (ls[i] > a + 0.01 && ls[i] < b - 0.01) out.push(pts[i]);
  out.push(pointAt(pts, b, ls).p);
  return out.map(([x, z]): Pt => [Math.round(x * 100) / 100, Math.round(z * 100) / 100]);
}

/** What removing a road takes with it (`whatGoes`). */
export interface Goes {
  /** The roads that go (the road itself among them): those left joined to no way out of the world, and the bridges left leading nowhere (`bridges`). */
  roads: number[];
  /** Of `roads`, the bridges that go because they would be left ending in a dead end: a deck to nowhere. */
  bridges: number[];
  /** The country roads that led onto a bridge that goes, left ending there in a dead end longer than `COURT`: made streets of no area (as a drawn one is: `drawStreet`). */
  streets: number[];
  lots: number[];
  /** The parking bays that go (`open.parking` indices): those by a road that goes, and the rest of their car parks with them. */
  parking: number[];
}

/**
 * What removing road `road` from `draft` takes with it (`Goes`): the roads left joined to no way out
 * of the world (with `road` itself), and a bridge left ending in a dead end (it led onto `road`: a
 * bridge has a road on at each end, as a drawn one must), the lots whose way in is on one of them and
 * the outdoor places at a gone trail's end (slot indices), the car parks by them (`ground.ts`
 * `wholeParks`); and the country roads that led onto such a bridge, left ending there more than
 * `COURT` m past the last road they meet, which become streets of no area (no taxi turns round in a
 * country road: the rule a drawn one is made by, `drawStreet`; the Build step gives each an area). Or
 * why it may not go: no road out of the world would be left, the roads left would be two networks
 * each with its own way out, another country road would be left a dead end longer than `COURT`, or
 * its lots are the town's last homes or its last park (`lots.ts` `removalRefused`).
 */
export function whatGoes(draft: NeighbourhoodDraft, t: Templates, road: number): Goes | { refused: string } {
  const roads = draft.roads;
  if (!Number.isInteger(road) || road < 0 || road >= roads.length) return { refused: 'there is no such road' };
  // (The motorway goes whole: both its carriageways and its ramps.)
  const start = motorwayOf(roads, road);
  const named = start.length > 1 ? 'the motorway' : (roads[road].name ?? 'it');
  const nets = networks(roads, new Set(start));
  const leaves = new Set([...nets.entries()].filter(([i]) => roads[i].ends.includes('out')).map(([, n]) => n));
  if (!leaves.size) return { refused: `without ${named} no road would lead out of the world` };
  if (leaves.size > 1) return { refused: `without ${named} the roads would be cut in two` };
  const gone = new Set([...start, ...[...nets.entries()].filter(([, n]) => !leaves.has(n)).map(([i]) => i)]);
  // A bridge it leaves ending in a dead end goes too, and so on (a bridge has a road on at each end).
  const bridges: number[] = [];
  let left = withoutRoads(roads, gone);
  for (;;) {
    const dangling = left.ended.filter((i) => left.roads[i].bridge && left.roads[i].ends.includes('dead')).map((i) => left.from[i]);
    if (!dangling.length) break;
    for (const i of dangling) {
      gone.add(i);
      bridges.push(i);
    }
    left = withoutRoads(roads, gone);
  }
  // A country road it leaves ending in a dead end past a court: refused (no taxi turns round in a country road), but one
  // that led onto a bridge that goes with it, which is made a street of no area, as a drawn one ending there would be
  // (the stretch of a street drawn over the water before its bridge). A street or a lane may be left so, the sim
  // turning round in any of theirs. (Not every country road made a street: one taken up and drawn again would not be
  // a country road again.)
  const bridgeEnds = bridges.flatMap((i) => [roads[i].points[0], roads[i].points[roads[i].points.length - 1]]);
  const atBridge = (r: RoadPlan) => r.ends.some((e, k) => e === 'dead' && bridgeEnds.some(([x, z]) => pmath.hypot(x - r.points[k ? r.points.length - 1 : 0][0], z - r.points[k ? r.points.length - 1 : 0][1]) <= r.width / 2 + 1));
  const streets: number[] = [];
  for (const i of left.ended) {
    const r = left.roads[i];
    if (r.kind !== 'country' || r.bridge) continue;
    const court = longestCourt(left.roads, [i]);
    if (!court || court.stub <= COURT + 0.5) continue;
    if (!atBridge(r)) return { refused: `without ${named}, ${r.name ?? 'a road'} would end ${Math.round(court.stub)} m past the last road it meets (a dead end reaches ${COURT} m at most)` };
    streets.push(left.from[i]);
  }
  const open = draft.open;
  const rects = draft.slots.map((s) => plotRect(t, s));
  const entries = surfacePlots({ plots: rects, meta: { roads, facing: draft.slots.map((s) => s.facing) } });
  const trailEnds = [...gone].filter((i) => roads[i].kind === 'trail').flatMap((i) => [roads[i].points[0], roads[i].points[roads[i].points.length - 1]]);
  const lots = draft.slots.filter((s, n) => (entries[n].entry && gone.has(entries[n].entry!.road)) || (s.kind === 'outdoor' && trailEnds.some(([x, z]) => offRect(rects[n], x, z) < TRAIL_END))).map((s) => s.index);
  // (The town keeps its last home and its last park, as a lot removed on its own does.)
  const kept = removalRefused(draft, new Set(lots));
  if (kept) return { refused: `${kept.replace(/\.$/, '')}: without ${named} it would go` };
  const index = new RoadIndex(roads);
  const bays = (open?.parking ?? []).flatMap((p, i) => {
    const near = index.nearest(p.x, p.z, ROAD_WIDTH / 2 + 6);
    return near && gone.has(near.road) ? [i] : [];
  });
  const parking = open ? wholeParks(open, bays) : [];
  return { roads: [...gone].sort((a, b) => a - b), bridges: bridges.sort((a, b) => a - b), streets: streets.sort((a, b) => a - b), lots, parking };
}

/**
 * The roads that go with road `road`: the motorway whole (its carriageways and its ramps) if it is one of them, else
 * the road alone.
 */
export function motorwayOf(roads: readonly RoadPlan[], road: number): number[] {
  return roads[road] && isMotorway(roads[road]) ? roads.flatMap((r, i) => (isMotorway(r) ? [i] : [])) : [road];
}

/**
 * `roads` without those in `gone`; a join left with nothing to join is a dead end (`ended`: those roads, by their index
 * in what is left; `from`: each road's index in `roads`). What a road passes over or under (`crosses`) is numbered as
 * the roads now are; one that passed over or under the motorway, gone, lies on the land (no `raised`).
 */
function withoutRoads(roads: readonly RoadPlan[], gone: ReadonlySet<number>): { roads: RoadPlan[]; ended: number[]; from: number[] } {
  const from = roads.flatMap((_, i) => (gone.has(i) ? [] : [i]));
  const flat = new Set(from.filter((i) => roads[i].crosses?.some((c) => gone.has(c.road) && isMotorway(roads[c.road]))));
  const kept = withCrosses(
    from.map((i) => {
      if (!flat.has(i)) return roads[i];
      const { raised: _raised, ...rest } = roads[i];
      return rest;
    }),
    from,
  );
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
  return { roads: left, ended, from };
}

/**
 * `draft` without road `road`, its lots and parking, and any road it leaves joined to nothing; refused
 * if no road out of the world would be left, or it would take the town's last home or park (see `whatGoes`). A road that met it in a T and stays
 * now ends there in a dead end (a bridge left so goes too; a country road that led onto it, left so past a court, is
 * made a street of no area); the water under a bridge that goes is water again; a village left with
 * no road and no lot goes too (`villages.ts` `withoutEmptyAreas`).
 */
export function removeRoad(draft: NeighbourhoodDraft, t: Templates, road: number): Edited {
  const goes = whatGoes(draft, t, road);
  if ('refused' in goes) return goes;
  // (A country road left a long dead end, a street of no area: see `whatGoes`.)
  const made = new Set(goes.streets);
  const { roads: left, from } = withoutRoads(draft.roads, new Set(goes.roads));
  const roads = left.map((r, i) => {
    if (!made.has(from[i])) return r;
    const { area: _area, ...rest } = r;
    return { ...rest, kind: 'street' as const };
  });
  const open = draft.open;
  let next: NeighbourhoodDraft = { ...draft, roads };
  if (open) {
    const [W, D] = [draft.width, draft.depth];
    let water = open.water;
    const bridges = goes.roads.filter((i) => draft.roads[i].bridge);
    if (bridges.length) {
      const lakes = open.zones.filter((z) => z.kind === 'water').map((z) => z.poly);
      const wetAt = (x: number, z: number) => lakes.some((poly) => inside(poly, x, z));
      let walk = unrle(water, W * D);
      for (const i of bridges) walk = withoutDeck(draft.roads[i], walk, wetAt, W, D);
      water = rle(walk);
    }
    next = { ...next, open: { ...open, water, ...withoutBays(open, goes.parking) } };
  }
  // (A village whose only road it was goes with it: its streets and lots went, so its area does too, the others numbered again.)
  return { draft: withoutEmptyAreas(removeLots(next, new Set(goes.lots))), cleared: { lots: goes.lots } };
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
  if (wanted === old) return { draft, cleared: { lots: [] } };
  const taken = namesIn(draft);
  if (taken.has(wanted)) return refused(`${wanted} is taken`);
  if (what.kind === 'road') {
    const roads = draft.roads.map((r, i) => (i === what.road ? { ...r, name: wanted } : r));
    // (Its houses' addresses: their number, then its name.)
    const slots = old ? draft.slots.map((s) => (s.name.endsWith(` ${old}`) && /^\d+$/.test(s.name.slice(0, -old.length - 1)) ? { ...s, name: `${s.name.slice(0, -old.length)}${wanted}` } : s)) : draft.slots;
    return { draft: { ...draft, roads, slots }, cleared: { lots: [] } };
  }
  if (!open) return { draft: { ...draft, name: wanted }, cleared: { lots: [] } };
  if (what.kind === 'place') {
    const place = places[what.place];
    const slots = draft.slots.map((s) => (s.index === place.plot && s.name === old ? { ...s, name: wanted } : s));
    return { draft: { ...draft, slots, open: { ...open, places: places.map((p, i) => (i === what.place ? { ...p, name: wanted } : p)) } }, cleared: { lots: [] } };
  }
  const k = town ? 0 : (what as { area: number }).area;
  const renamed = areas.map((a, i) => (i === k ? { ...a, name: wanted } : a));
  return { draft: { ...draft, ...(town ? { name: wanted } : {}), open: { ...open, areas: renamed } }, cleared: { lots: [] } };
}
