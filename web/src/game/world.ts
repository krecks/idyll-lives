/**
 * Kinds of world a new game can be set in: towns of three kinds, a farm, a seaside village, a
 * mountain hamlet, a lakeside town and a clearing in the woods. A recipe says how the town is laid
 * out (its roads, how many lots, public places, farms; `generateWorld` in `town.ts` lays it out)
 * and what the land round it is like; the world's seed rolls the rest, so no two worlds of a kind
 * are alike: its name, the way its roads wind, its tagline, how hilly and wooded it is, where the water lies.
 */

import type { Content, WorldNames } from '../content/content';
import { DEFAULT_LANDSCAPE, type LandscapeParams } from '../core/protocol';
import type { GardenSize } from './housegen';
import { branch, centroid, clearOf, crossing, crossStreet, cutAround, inside, lane, leftOf, lengths, loop, nearest, pointAt, range, ring, RoadIndex, roomAt, roundabout, through, type Box, type Pt, type RoadPlan } from './roads';

export type WorldKind = 'small' | 'medium' | 'dense' | 'farm' | 'seaside' | 'mountain' | 'lakeside' | 'forest';
export const WORLD_KINDS: readonly WorldKind[] = ['small', 'medium', 'dense', 'farm', 'seaside', 'mountain', 'lakeside', 'forest'];

type Rand = () => number;

/** A town's roads as a recipe lays them out (`box`'s coordinates). */
export interface RoadLayout {
  roads: RoadPlan[];
  /** Greens the roads go round: no lots there, the park on one if it fits. */
  greens?: Pt[][];
  /** Where the town's middle is (its lots gather round it); default: the middle of the first road. */
  centre?: Pt;
  /** The side of the town the water is on (the sea, the lake). */
  water?: 'zmin' | 'zmax';
}

/**
 * The world's main town: the area its roads are laid out in, how many lots, and how far its lanes
 * reach (docs/design/open-world.md). Every world is a Big one (cosy and normal were dropped in
 * 0.8.0): its main town is the heart of a bigger world (`macro.ts`).
 */
export interface SizeSpec {
  /** The area its roads are laid out in (m; the town is cut to what it takes, at most 256 m but in a Big world). */
  box: Box;
  /** Lots in all (public places and farms among them); fewer if the roads haven't room. */
  lots: number;
  /** Lengths of lanes, crescents and the like (not widths) times this. */
  scale: number;
  /** Cross streets between the roads (`extend`): a bigger town's blocks. */
  extend?: number;
  /** Streets out into the parts of the box the roads leave empty (`fill`): a big town's outer blocks. */
  fill?: number;
  /** Public places besides the park, and farm lots, if not the recipe's. */
  places?: [number, number];
  farms?: [number, number];
}

export interface WorldRecipe {
  label: string;
  blurb: string;
  /** The world's main town. */
  town: SizeSpec;
  /** Chances of a gap of a metre or two after a lot, and of a stretch of meadow. */
  gaps: [number, number];
  /** The town's roads (`scale`: as `SizeSpec.scale`). */
  roads(rand: Rand, box: Box, scale?: number): RoadLayout;
  /** Chances of a small and a large house (the rest are medium). */
  small: number;
  large: number;
  /** How much garden the houses get round them, by weight (each lot is cut to fit its house; `housegen.ts` `fitLot`). */
  gardens: Partial<Record<GardenSize, number>>;
  /** Public places besides the town park (fewest, most), and the ones it would rather have. */
  places: [number, number];
  prefer?: string[];
  /** Farm lots (fewest, most). */
  farms?: [number, number];
  /** The land round a town of this size (`width` × `depth` m; `water`: the side the roads left for it). */
  landscape(rand: Rand, width: number, depth: number, water?: 'zmin' | 'zmax'): LandscapeParams;
}


/** A green moved a little in hue and brightness (so each world's grass is its own). */
function tint(hex: string, rand: Rand, warm: number, light: number): string {
  const n = parseInt(hex.slice(1), 16);
  const c = [(n >> 16) & 255, (n >> 8) & 255, n & 255];
  const out = [c[0] * (1 + warm) * light, c[1] * light, c[2] * (1 - warm) * light].map((v) => Math.max(0, Math.min(255, Math.round(v * (0.97 + rand() * 0.06)))));
  return `#${out.map((v) => v.toString(16).padStart(2, '0')).join('')}`;
}

/** Grass for a world: a base palette, warmer or cooler, lighter or darker. */
function grass(rand: Rand, base: LandscapeParams['grass'], warm: [number, number], light: [number, number]): LandscapeParams['grass'] {
  const w = range(rand, ...warm);
  const l = range(rand, ...light);
  return base.map((g) => tint(g, rand, w, l)) as LandscapeParams['grass'];
}

const GREEN = DEFAULT_LANDSCAPE.grass;
const ALPINE: LandscapeParams['grass'] = ['#557F3A', '#6E9142', '#8C9A55', '#4F7238'];
const COAST: LandscapeParams['grass'] = ['#7AA046', '#93AD55', '#B2B46A', '#6C9142'];

/** The classic valley, rolled a little differently for each world. */
const valley = (rand: Rand): LandscapeParams => ({
  ...DEFAULT_LANDSCAPE,
  hills: range(rand, 9, 18),
  rise: range(rand, 38, 52),
  rim: range(rand, 22, 36),
  forest: range(rand, -0.08, 0.08),
  conifers: range(rand, 0.35, 0.6),
  birches: range(rand, 0.35, 0.65),
  grass: grass(rand, GREEN, [-0.04, 0.05], [0.95, 1.05]),
});

const side = (rand: Rand): 1 | -1 => (rand() < 0.5 ? 1 : -1);
const lengthOf = (r: RoadPlan) => lengths(r.points).at(-1)!;

/**
 * A lane off `parent` at arc length `s` on its `side`, `length` m long, curling `curl` radians
 * whichever way brings its end nearer `target` (a dead end, for `connect` to take on).
 */
function toward(parent: RoadPlan, s: number, side: 1 | -1, length: number, curl: number, target: Pt): RoadPlan {
  const lanes = [branch(parent, s, side, length, curl), branch(parent, s, side, length, -curl)];
  const gap = (r: RoadPlan) => Math.hypot(r.points[r.points.length - 1][0] - target[0], r.points[r.points.length - 1][1] - target[1]);
  return gap(lanes[0]) <= gap(lanes[1]) ? lanes[0] : lanes[1];
}

/** One winding high street, a back lane looping off it and back, a short lane off the other side. */
export function highStreet(rand: Rand, box: Box, scale = 1): RoadLayout {
  const main = through(rand, box, { amp: range(rand, 6, 11), bends: 2 });
  const L = lengthOf(main);
  const s = side(rand);
  const s0 = L * range(rand, 0.28, 0.4);
  const back = loop(main, s0, Math.min(L - 30, s0 + range(rand, 70, 90) * scale), s, range(rand, 30, 38) * scale);
  const lane = branch(main, L * range(rand, 0.45, 0.65), -s as 1 | -1, range(rand, 28, 40) * scale, range(rand, -0.7, 0.7));
  return { roads: [main, back.road, lane] };
}

/** A through road with a crescent round a green; lanes off it joined up into a back street and to the crescent. */
export function crescentTown(rand: Rand, box: Box, scale = 1): RoadLayout {
  const main = through(rand, box, { at: 0.55, amp: range(rand, 6, 10), bends: 2 });
  const L = lengthOf(main);
  const mid = L * range(rand, 0.42, 0.55);
  const s = side(rand);
  const half = range(rand, 24, 30) * scale;
  const crescent = loop(main, mid - half, mid + half, s, range(rand, 34, 42) * scale);
  const top = pointAt(crescent.road.points, lengthOf(crescent.road) / 2).p;
  // Across the road from the crescent, two lanes curling towards each other (a back street, once joined).
  const apart = range(rand, 30, 40) * scale;
  const behind: Pt = (() => {
    const { p, t } = pointAt(main.points, mid);
    const n = leftOf(t);
    return [p[0] - n[0] * s * 60 * scale, p[1] - n[1] * s * 60 * scale];
  })();
  const lanes = [
    toward(main, mid - apart, -s as 1 | -1, range(rand, 50, 62) * scale, range(rand, 0.6, 0.9), behind),
    toward(main, mid + apart, -s as 1 | -1, range(rand, 50, 62) * scale, range(rand, 0.6, 0.9), behind),
    // Further out on the crescent's side, lanes curling in towards it.
    toward(main, L * range(rand, 0.17, 0.24), s, range(rand, 40, 55) * scale, range(rand, 0.4, 0.8), top),
    toward(main, L * range(rand, 0.76, 0.83), s, range(rand, 40, 55) * scale, range(rand, 0.4, 0.8), top),
  ];
  return { roads: [main, crescent.road, ...lanes], greens: [crescent.green], centre: centroid(crescent.green) };
}

/**
 * A crossroads with a roundabout (a fountain in the middle) and a loose grid of blocks round it:
 * in three or four of its quarters a street from one road round to the other, and further out
 * lanes off each road for `connect` to join up.
 */
export function crossroads(rand: Rand, box: Box, scale = 1): RoadLayout {
  const main = through(rand, box, { amp: range(rand, 4, 8), bends: 2 });
  const cross = through(rand, box, { along: 'z', amp: range(rand, 4, 8), bends: 2 });
  const at = crossing(main, cross) ?? ([box.w / 2, box.d / 2] as Pt);
  const R = 6.5;
  const roads: RoadPlan[] = [...cutAround(main, at, R), ...cutAround(cross, at, R), roundabout(at[0], at[1], R)];
  const sm = nearest(main.points, at[0], at[1]).s;
  const sc = nearest(cross.points, at[0], at[1]).s;
  const quarters: [1 | -1, 1 | -1][] = [[1, 1], [1, -1], [-1, 1], [-1, -1]];
  for (let i = quarters.length - 1; i > 0; i--) {
    const j = Math.floor(rand() * (i + 1));
    [quarters[i], quarters[j]] = [quarters[j], quarters[i]];
  }
  const blocks = rand() < 0.5 ? 3 : 4;
  for (const [qx, qz] of quarters.slice(0, blocks)) {
    // From the main road `ra` m out from the middle, round the corner, to the cross road `rb` m out.
    const [ra, rb] = [range(rand, 44, 60) * scale, range(rand, 44, 60) * scale];
    const a = pointAt(main.points, sm + qx * ra).p;
    const b = pointAt(cross.points, sc + qz * rb).p;
    const corner: Pt = [a[0] + (at[0] - a[0]) * 0.1, b[1] + (at[1] - b[1]) * 0.1];
    const mid = (p: Pt, k: number): Pt => [p[0] + (corner[0] - p[0]) * k, p[1] + (corner[1] - p[1]) * k];
    roads.push(lane([a, mid(a, 0.5), corner, mid(b, 0.5), b], ['join', 'join']));
  }
  // Further out: a lane off each arm, for `connect` to take round to the next.
  for (const [r, s0] of [[main, sm], [cross, sc]] as const) {
    const len = lengthOf(r);
    for (const k of [1, -1] as const) {
      const s = s0 + k * range(rand, 0.6, 0.75) * (k > 0 ? len - s0 : s0);
      if (s < 20 || s > len - 20) continue;
      roads.push(toward(r, s, side(rand), range(rand, 45, 60) * scale, range(rand, 0.5, 1), at));
    }
  }
  return { roads, centre: at };
}

/** A long country lane winding through the fields, a fork off it leading out of town. */
export function countryLane(rand: Rand, box: Box, _scale = 1): RoadLayout {
  const main = through(rand, box, { amp: range(rand, 14, 22), bends: 3, width: 6 });
  const L = lengthOf(main);
  const fork = branch(main, L * range(rand, 0.4, 0.6), side(rand), box.d, range(rand, -0.5, 0.5), { end: 'out' });
  return { roads: [main, fork], centre: [box.w / 2, box.d / 2] };
}

/**
 * A road along the water's side of town, `at` across the box from that edge, winding `amp` m:
 * its lots on its landward side only (`land`, +1: its left); the town's middle a little inland.
 */
function waterfront(rand: Rand, box: Box, at: number, amp: [number, number]) {
  const water = rand() < 0.5 ? 'zmin' : 'zmax';
  const main = through(rand, box, { at: water === 'zmin' ? at : 1 - at, amp: range(rand, ...amp), bends: 2 });
  // Inland: the left of a road along +x is towards smaller z.
  const land: 1 | -1 = water === 'zmin' ? -1 : 1;
  main.lots = land;
  const centre: Pt = [box.w / 2, water === 'zmin' ? box.d * 0.3 : box.d * 0.7];
  return { main, water, land, centre, L: lengthOf(main) } as const;
}

/**
 * A road along the shore (homes on its landward side only), a crescent up the hill at one end of
 * the village, two lanes climbing inland for `connect` to join into an upper street.
 */
function shoreRoad(rand: Rand, box: Box, scale = 1): RoadLayout {
  const { main, water, land, centre, L } = waterfront(rand, box, 0.12, [3, 6]);
  const end = rand() < 0.5 ? 0.22 : 0.78;
  const crescent = loop(main, L * end - 22 * scale, L * end + 22 * scale, land, range(rand, 30, 38) * scale);
  const climbs = [0.5, 1 - end].map((at) => L * (at + range(rand, -0.04, 0.04)));
  const up = inland(main, (climbs[0] + climbs[1]) / 2, land, 75 * scale);
  const lanes = climbs.map((s) => toward(main, s, land, range(rand, 55, 66) * scale, range(rand, 0.2, 0.45), up));
  return { roads: [main, crescent.road, ...lanes], water, centre };
}

/** The point `depth` m inland of road `r` at arc length `s` (on its `land` side). */
function inland(r: RoadPlan, s: number, land: 1 | -1, depth: number): Pt {
  const { p, t } = pointAt(r.points, s);
  const n = leftOf(t);
  return [p[0] + n[0] * land * depth, p[1] + n[1] * land * depth];
}

/** A lane winding up the valley in tight bends, tracks off it (round to it again where the land allows). */
function switchback(rand: Rand, box: Box, scale = 1): RoadLayout {
  const main = through(rand, box, { amp: range(rand, 16, 24), bends: 3, width: 6 });
  const L = lengthOf(main);
  const tracks = [0.3, 0.6].map((at) => {
    const s = L * (at + range(rand, 0, 0.12));
    const ahead = pointAt(main.points, Math.min(L, s + 70 * scale)).p;
    return toward(main, s, side(rand), range(rand, 26, 36) * scale, range(rand, 0.5, 0.9), ahead);
  });
  return { roads: [main, ...tracks], centre: [box.w / 2, box.d / 2] };
}

/**
 * A road round the lake's end of town (homes on its landward side), a crescent off it, and lanes
 * either side of it climbing inland for `connect` to join into an upper street over the crescent.
 */
function lakeRoad(rand: Rand, box: Box, scale = 1): RoadLayout {
  const { main, water, land, centre, L } = waterfront(rand, box, 0.14, [5, 9]);
  const crescent = loop(main, L * 0.5 - 24 * scale, L * 0.5 + 24 * scale, land, range(rand, 34, 42) * scale);
  const up = inland(main, L * 0.5, land, 95 * scale);
  const lanes = [0.3, 0.7].map((at) => toward(main, L * (at + range(rand, -0.03, 0.03)), land, range(rand, 68, 80) * scale, range(rand, 0.25, 0.5), up));
  return { roads: [main, crescent.road, ...lanes], water, centre };
}

/** A lane round a clearing in the woods, a way out of it and a track into the trees. */
export function clearing(rand: Rand, box: Box, scale = 1): RoadLayout {
  const [cx, cz] = [box.w / 2, box.d / 2];
  const round = ring(rand, cx, cz, range(rand, 30, 36) * scale, range(rand, 22, 28) * scale);
  // Out of the clearing from its southern edge (the way the menus look at it from), and a track into the trees.
  const bottom = round.points.reduce((a, b) => (b[1] > a[1] ? b : a));
  const wander = range(rand, -10, 10);
  const out = lane([bottom, [bottom[0] + wander * 0.4, bottom[1] + 22], [bottom[0] + wander, box.d + 24]], ['join', 'out'], 6);
  const top = round.points.reduce((a, b) => (b[1] < a[1] ? b : a));
  const track = lane([top, [top[0] + range(rand, -14, 14) * scale, top[1] - 20 * scale], [top[0] + range(rand, -24, 24) * scale, top[1] - 34 * scale]], ['join', 'dead']);
  // A bigger clearing: a lane off its side too, curling round (to the clearing again, where it can).
  const lanes: RoadPlan[] = [];
  if (scale > 1) {
    const ls = lengths(round.points);
    const east = rand() < 0.5;
    const at = round.points.reduce((k, q, i) => ((east ? q[0] > round.points[k][0] : q[0] < round.points[k][0]) ? i : k), 0);
    const { p, t } = pointAt(round.points, ls[at], ls);
    const n = leftOf(t);
    const away: 1 | -1 = n[0] * (p[0] - cx) + n[1] * (p[1] - cz) > 0 ? 1 : -1;
    const ahead = pointAt(round.points, (ls[at] + ls[ls.length - 1] * 0.22) % ls[ls.length - 1], ls).p;
    lanes.push(toward(round, ls[at], away, range(rand, 40, 50) * scale, range(rand, 1.2, 1.6), ahead));
  }
  return { roads: [round, out, track, ...lanes], greens: [round.points], centre: [cx, cz] };
}

/**
 * `count` cross streets, each from a road across to the next one along, 30 to 110 m (times
 * `scale`) away: blocks between them. Each keeps clear of the other roads, 18 m from those it runs
 * alongside, inside the box and off the greens (a few tries each; fewer if none fit).
 */
export function extend(rand: Rand, roads: readonly RoadPlan[], box: Box, count: number, scale = 1, greens: readonly Pt[][] = []): RoadPlan[] {
  const out = [...roads];
  const fit = { box, greens };
  for (let n = 0, tries = 0; n < count && tries < count * 40; tries++) {
    const parents = out.filter((r) => !r.island && r.kind !== 'trail' && lengthOf(r) > 40);
    if (!parents.length) break;
    const parent = parents[Math.floor(rand() * parents.length)];
    const from = out.indexOf(parent);
    const { p, t } = pointAt(parent.points, lengthOf(parent) * range(rand, 0.1, 0.9));
    const sd = parent.lots ?? side(rand);
    const n0 = leftOf(t);
    const a = range(rand, -0.3, 0.3);
    const dir: Pt = [(n0[0] * Math.cos(a) - n0[1] * Math.sin(a)) * sd, (n0[0] * Math.sin(a) + n0[1] * Math.cos(a)) * sd];
    const street = crossStreet(out, new RoadIndex(out), from, p, dir, fit, 110 * scale);
    if (!street) continue;
    out.push(street);
    n++;
  }
  return out;
}

/** A part of a town's box this far (m) from every road is empty: room for a street and its lots each side. */
const EMPTY = 58;

/**
 * Up to `count` streets out into the parts of `box` the roads leave empty (a big town's outer
 * blocks, so its streets reach all of its box), the emptiest first: a crescent off the road
 * nearest each, bowing out into it, where one fits as `connect`'s lanes do (inside the box, off
 * the greens, clear of the other roads, room at both its junctions); else a lane towards it for
 * `connect` to join up.
 */
export function fill(rand: Rand, roads: readonly RoadPlan[], box: Box, count: number, scale = 1, greens: readonly Pt[][] = []): RoadPlan[] {
  const out = [...roads];
  const fit = { box, greens };
  const tried: Pt[] = [];
  for (let n = 0, tries = 0; n < count && tries < count * 4; tries++) {
    const index = new RoadIndex(out);
    let best: { p: Pt; d: number } | null = null;
    for (let z = 34; z <= box.d - 34; z += 10) {
      for (let x = 34; x <= box.w - 34; x += 10) {
        if (tried.some((t) => Math.hypot(t[0] - x, t[1] - z) < 40) || greens.some((g) => inside(g, x, z))) continue;
        // (Most points are near a road: asked how near only when not.)
        if (index.nearest(x, z, EMPTY)) continue;
        const d = index.nearest(x, z, 200)?.dist ?? 200;
        if (!best || d > best.d) best = { p: [x, z], d };
      }
    }
    if (!best) break;
    const p = best.p;
    tried.push(p);
    // The nearest road it can come off (not a roundabout, nor a short one).
    const near = [...index.around(p[0], p[1], best.d + 40)].filter(([r]) => !out[r].island && out[r].kind !== 'trail' && lengthOf(out[r]) > 70).sort((a, b) => a[1].dist - b[1].dist)[0];
    if (!near) continue;
    const [r, at] = near;
    const parent = out[r];
    const L = lengthOf(parent);
    const sideOf: 1 | -1 = (() => {
      const q = pointAt(parent.points, at.s).p;
      const nn = leftOf(at.t);
      return (p[0] - q[0]) * nn[0] + (p[1] - q[1]) * nn[1] >= 0 ? 1 : -1;
    })();
    const depth = Math.min(at.dist * range(rand, 0.8, 0.95), 85 * scale);
    let made: RoadPlan | null = null;
    for (const span of [range(rand, 70, 95) * scale, range(rand, 50, 65) * scale]) {
      const [s0, s1] = [at.s - span / 2, at.s + span / 2];
      if (s0 < 12 || s1 > L - 12) continue;
      const { road } = loop(parent, s0, s1, sideOf, depth);
      const [a, b] = [road.points[0], road.points[road.points.length - 1]];
      if (!roomAt(index, a, [r]) || !roomAt(index, b, [r])) continue;
      if (!clearOf(road.points, road.width, out, index, fit, r, r, 26)) continue;
      made = road;
      break;
    }
    // Else a lane towards it (`connect` takes it on, or cuts it back).
    if (!made) {
      const lane = branch(parent, Math.min(L - 15, Math.max(15, at.s)), sideOf, Math.min(at.dist + 20, 110 * scale), range(rand, -0.5, 0.5));
      if (roomAt(index, lane.points[0], [r]) && clearOf(lane.points, lane.width, out, index, fit, r, -1)) made = lane;
    }
    if (!made) continue;
    out.push(made);
    n++;
  }
  return out;
}

export const WORLDS: Record<WorldKind, WorldRecipe> = {
  small: {
    label: 'Small town',
    blurb: 'Homes along one winding street, a park and a shop or two.',
    town: { box: { w: 330, d: 256 }, lots: 70, scale: 1.5, extend: 5, fill: 4, places: [2, 3] },
    gaps: [0.4, 0.1],
    roads: highStreet,
    small: 0.25,
    large: 0.2,
    gardens: { snug: 1, roomy: 3, big: 1 },
    places: [1, 2],
    landscape: valley,
  },
  medium: {
    label: 'Medium town',
    blurb: 'A crescent of homes round the village green, lanes off the main road.',
    town: { box: { w: 366, d: 340 }, lots: 82, scale: 1.45, extend: 6, fill: 5, places: [4, 4] },
    gaps: [0.35, 0.08],
    roads: crescentTown,
    small: 0.25,
    large: 0.2,
    gardens: { snug: 1, roomy: 3, big: 1 },
    places: [3, 3],
    landscape: valley,
  },
  dense: {
    label: 'Dense town',
    blurb: 'A crossroads and crooked lanes of small lots, shops on every corner.',
    town: { box: { w: 390, d: 390 }, lots: 110, scale: 1.3, extend: 7, fill: 6, places: [6, 6] },
    gaps: [0.12, 0.02],
    roads: crossroads,
    small: 0.6,
    large: 0.05,
    gardens: { snug: 4, roomy: 1 },
    places: [5, 5],
    prefer: ['cafe', 'grocer', 'bakery', 'market'],
    landscape: (rand) => ({ ...valley(rand), hills: range(rand, 6, 11), forest: range(rand, -0.18, -0.06), treeline: range(rand, 6, 12) }),
  },
  farm: {
    label: 'Farm',
    blurb: 'A country road, farmsteads with fields and barns, open land all round.',
    town: { box: { w: 330, d: 330 }, lots: 60, scale: 1.15, extend: 5, fill: 4, places: [2, 3], farms: [0, 0] },
    gaps: [0.5, 0.3],
    roads: countryLane,
    small: 0.1,
    large: 0.35,
    gardens: { roomy: 1, big: 3 },
    places: [1, 2],
    prefer: ['market', 'grocer'],
    farms: [3, 4],
    landscape: (rand) => ({
      ...valley(rand),
      hills: range(rand, 4, 8),
      rise: range(rand, 55, 75),
      rim: range(rand, 14, 24),
      rimFrom: range(rand, 80, 100),
      forest: range(rand, -0.2, -0.1),
      grass: grass(rand, GREEN, [0.02, 0.09], [1, 1.08]),
      fields: { size: range(rand, 18, 30), angle: range(rand, -0.5, 0.5) },
    }),
  },
  seaside: {
    label: 'Seaside village',
    blurb: 'Homes above a sandy shore, a café and the smell of the sea.',
    town: { box: { w: 410, d: 280 }, lots: 82, scale: 1.45, extend: 8, fill: 4, places: [4, 4] },
    gaps: [0.3, 0.08],
    roads: shoreRoad,
    small: 0.4,
    large: 0.1,
    gardens: { snug: 2, roomy: 2 },
    places: [3, 3],
    prefer: ['cafe', 'market', 'bakery'],
    landscape: (rand, _width, _depth, side) => ({
      ...valley(rand),
      hills: range(rand, 8, 14),
      forest: range(rand, -0.15, -0.02),
      conifers: range(rand, 0.4, 0.7),
      birches: range(rand, 0.1, 0.3),
      grass: grass(rand, COAST, [0, 0.06], [1, 1.06]),
      water: { kind: 'sea', side: side ?? (rand() < 0.5 ? 'zmin' : 'zmax'), shore: range(rand, 4, 7) },
    }),
  },
  mountain: {
    label: 'Mountain hamlet',
    blurb: 'A few big gardens in a high valley, pines on steep slopes all round.',
    town: { box: { w: 354, d: 305 }, lots: 72, scale: 1.45, extend: 8, fill: 4, places: [2, 3] },
    gaps: [0.45, 0.25],
    roads: switchback,
    small: 0.05,
    large: 0.55,
    gardens: { roomy: 1, big: 3 },
    places: [1, 2],
    prefer: ['cafe', 'bakery'],
    landscape: (rand) => ({
      ...DEFAULT_LANDSCAPE,
      hills: range(rand, 10, 16),
      rise: range(rand, 18, 28),
      rim: range(rand, 30, 50),
      rimFrom: range(rand, 40, 60),
      forest: range(rand, 0.05, 0.15),
      treeline: range(rand, 1, 3),
      conifers: range(rand, 0.85, 1),
      birches: range(rand, 0.2, 0.45),
      grass: grass(rand, ALPINE, [-0.06, 0], [0.92, 1.02]),
      mountains: range(rand, 120, 160),
    }),
  },
  lakeside: {
    label: 'Lakeside',
    blurb: 'A town by still water, reeds and a sandy bank, gentle hills.',
    town: { box: { w: 390, d: 316 }, lots: 84, scale: 1.4, extend: 8, fill: 5, places: [3, 4] },
    gaps: [0.35, 0.1],
    roads: lakeRoad,
    small: 0.2,
    large: 0.25,
    gardens: { roomy: 3, big: 1 },
    places: [2, 3],
    prefer: ['cafe', 'market'],
    landscape: (rand, width, depth, side) => {
      const base = valley(rand);
      const rx = range(rand, 24, 38);
      const rz = range(rand, 16, 26);
      // Behind the town or in front of it (where the view across the town shows it, and its road runs), past its flat apron.
      const behind = side ? side === 'zmin' : rand() < 0.5;
      const water = { kind: 'lake' as const, x: width * range(rand, 0.3, 0.7), z: behind ? -(10 + rz) : depth + 10 + rz, rx, rz };
      return { ...base, hills: range(rand, 7, 12), birches: range(rand, 0.55, 0.8), water };
    },
  },
  forest: {
    label: 'Forest clearing',
    blurb: 'A clearing in deep woods: big gardens, few neighbours, birdsong.',
    town: { box: { w: 340, d: 305 }, lots: 66, scale: 1.4, extend: 8, fill: 4, places: [2, 2] },
    gaps: [0.45, 0.2],
    roads: clearing,
    small: 0,
    large: 0.6,
    gardens: { roomy: 1, big: 4 },
    places: [1, 1],
    prefer: ['bakery', 'cafe'],
    landscape: (rand) => ({
      ...valley(rand),
      hills: range(rand, 8, 14),
      forest: range(rand, 0.28, 0.4),
      treeline: range(rand, 0, 1),
      conifers: range(rand, 0.45, 0.75),
      birches: range(rand, 0.5, 0.8),
      grass: grass(rand, GREEN, [-0.05, 0.02], [0.9, 0.98]),
    }),
  },
};

const pickOf = <T>(list: readonly T[], rand: Rand): T => list[Math.floor(rand() * list.length)];

/** A world's names from the content (the town and street lists when a kind has none). */
function namesOf(content: Content, kind: WorldKind): WorldNames | null {
  const names = content.names.worlds?.[kind];
  return names && names.start.length && names.end.length && names.streets.length && names.ways.length ? names : null;
}

/** A name for a town of this kind: a start and an end ("Gull" + "cove"). */
export function worldName(content: Content, kind: WorldKind, rand: Rand): string {
  const names = namesOf(content, kind);
  if (!names) return pickOf(content.names.towns ?? ['Willowbrook'], rand);
  return `${pickOf(names.start, rand)}${pickOf(names.end, rand)}`.replace(/(.)\1\1/, '$1$1');
}

/**
 * `count` different street names for a town of this kind, none of `taken`: each with a first word
 * of its own ("Mill Lane", "Orchard Way"), then other pairs of words, then (a world of several
 * places) ones with the place's name (`area`). The first 40 tries are rolled, as they always were;
 * the rest are taken in turn without rolling, so the town's later rolls stay as they were.
 */
export function streetNames(content: Content, kind: WorldKind, rand: Rand, count: number, taken: readonly string[] = [], area?: string): string[] {
  const names = namesOf(content, kind);
  const out: string[] = [];
  const firstWord = (name: string) => name.split(' ')[0];
  const used = (name: string) => taken.includes(name) || out.includes(name);
  const firstUsed = (name: string) => taken.some((n) => firstWord(n) === firstWord(name)) || out.some((n) => firstWord(n) === firstWord(name));
  for (let tries = 0; out.length < count && tries < 40; tries++) {
    const name = names ? `${pickOf(names.streets, rand)} ${pickOf(names.ways, rand)}` : pickOf(content.names.streets ?? ['Main Street'], rand);
    if (!firstUsed(name)) out.push(name);
  }
  if (out.length < count) {
    // Every pair of words, in an order of this town's own.
    const starts = names ? names.streets : (content.names.streets ?? ['Main Street']).map(firstWord);
    const ways = names ? names.ways : ['Street', 'Lane', 'Road', 'Way', 'Close', 'Row'];
    const turn = taken.length * 7 + count;
    const pairs = starts.flatMap((_, i) => ways.map((_, j) => `${starts[(i + turn) % starts.length]} ${ways[(i + j) % ways.length]}`));
    for (const name of pairs) if (out.length < count && !firstUsed(name)) out.push(name);
    for (const name of pairs) if (out.length < count && !used(name)) out.push(name);
    for (const name of pairs) if (out.length < count && area && !used(`${name}, ${area}`)) out.push(`${name}, ${area}`);
  }
  while (out.length < count) out.push(`${out[0] ?? 'Main Street'} ${out.length + 1}`);
  return out;
}

/** One line about a town of this kind. */
export function worldTagline(content: Content, kind: WorldKind, rand: Rand): string {
  const lines = content.names.worlds?.[kind]?.taglines ?? [];
  return lines.length ? pickOf(lines, rand) : WORLDS[kind].blurb;
}
