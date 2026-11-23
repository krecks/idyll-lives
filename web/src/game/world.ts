/**
 * The places a new game can be set in (the countryside, farmland, the seaside, a lake, a river
 * valley, the mountains, a forest, a heath, an island) and the options the player picks besides
 * (`WorldOptions`: how big the main town is, how many villages round it, how hilly and wooded the
 * land). A place and its options make one `WorldSpec`, everything the generator reads (`specOf`;
 * `macro.ts` `generateFromSpec` lays it out): how its towns are laid out (their roads, how many
 * lots, public places, farms) and what the land round them is like. The world's seed rolls the
 * rest, so no two worlds of a place are alike: its name, the way its roads wind, its tagline, how
 * hilly and wooded it is, where the water lies.
 */

import type { Content, WorldNames } from '../content/content';
import { DEFAULT_LANDSCAPE, type LandSize, type LandscapeParams, type TownSize, type WorldOptions } from '../core/protocol';
import { ROAD_KIND } from '../core/roadKinds';
import type { GardenSize } from './housegen';
import { branch, centroid, clearOf, crossing, crossStreet, cutAround, inside, lane, leftOf, lengths, loop, nearest, pointAt, range, ring, roadDistance, RoadIndex, roomAt, roundabout, through, type Box, type Pt, type RoadPlan } from './roads';
import * as pmath from './pmath';

export type { LandSize, TownSize, WorldOptions };

/** A place a world is set in. */
export type WorldKind = 'country' | 'farm' | 'seaside' | 'lakeside' | 'river' | 'mountain' | 'forest' | 'heath' | 'island';
/** The places, in the order step 1 lists them. */
export const WORLD_KINDS: readonly WorldKind[] = ['country', 'farm', 'seaside', 'lakeside', 'river', 'mountain', 'forest', 'heath', 'island'];

/**
 * A part of making a world done, in order (step 1's progress, `CreateWorld.svelte`): the land and its
 * towns' streets, the country roads between them, the lots, then the houses and the menus' preview
 * (`townJobs.ts`), and the land graded under the town (`grade`). Told as each ends; what it says
 * doesn't change the world.
 */
export type WorldStep = 'land' | 'roads' | 'homes' | 'houses' | 'grade';

type Rand = () => number;

/** A town's roads as its shape lays them out (`box`'s coordinates). */
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
 * reach. Every world is a Big one (cosy and normal were dropped in
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
  /** Public places besides the park, and farm lots, if not the world's (`LotMix`). */
  places?: [number, number];
  farms?: [number, number];
  /** Its main through road an avenue (a city's: `Shape`'s `avenue`), laid so before anything else is laid off it. */
  avenue?: boolean;
  /** Pairs of streets in its dense middle made one-way at most (`oneway.ts` `oneWayPairs`; a town's and a city's). */
  oneway?: number;
}

/**
 * A town's roads laid out in `box` (`scale`: as `SizeSpec.scale`). `avenue`: its main through road an avenue
 * (`ROAD_KIND.avenue`, 14 m), the first of its roads, laid so before the streets off it (where a crossroads' cross
 * road meets it, a crossroads with lights, not a roundabout); a shape with no through road (a clearing's ring) has none.
 */
export type Shape = (rand: Rand, box: Box, scale?: number, avenue?: boolean) => RoadLayout;

/** Road `r` an avenue if `avenue` (as wide as one), else as it is. */
const avenueIf = (r: RoadPlan, avenue = false): RoadPlan => (avenue ? { ...r, kind: 'avenue', width: ROAD_KIND.avenue.width } : r);

/**
 * What a town's lots are (`town.ts` `wantedLots`, and `LotPlacer` for the gaps): how big its
 * houses, how much garden, which public places and how many, its farms.
 */
export interface LotMix {
  /** Chances of a gap of a metre or two after a lot, and of a stretch of meadow. */
  gaps: [number, number];
  /** Chances of a small and a large house (the rest are medium). */
  small: number;
  large: number;
  /** How much garden the houses get round them, by weight (each lot is cut to fit its house; `housegen.ts` `fitLot`). */
  gardens: Partial<Record<GardenSize, number>>;
  /** Public places besides the town park (fewest, most), if not the town's own (`SizeSpec.places`), and the ones it would rather have. */
  places: [number, number];
  prefer?: string[];
  /** Farm lots (fewest, most), if not the town's own (`SizeSpec.farms`). */
  farms?: [number, number];
}

/** A hamlet round the main town: its shapes (each hamlet one of them), its box, lots, how far its lanes reach (`SizeSpec.scale`), its streets out into its box and cross streets (`fill`, `extend`). */
export interface HamletSpec {
  shapes: Shape[];
  box: Box;
  lots: [number, number];
  farms?: [number, number];
  places: [number, number];
  scale: number;
  fill?: number;
  extend?: number;
}

/**
 * A world as the generator lays it out (`macro.ts` `generateFromSpec`): a place with its options
 * (`specOf`), or one of the classic worlds (`CLASSIC`). Ranges (`[fewest, most]`) are rolled from
 * the world's seed.
 */
export interface WorldSpec extends LotMix {
  /** What the world says it is (`WorldInfo.kind`): a place (`WorldKind`), or a classic world's kind. */
  kind: string;
  /** The options it was made with (a place's; a classic world has none). */
  options?: WorldOptions;
  /** Which of the content's name lists its towns, streets and taglines come from (`content.names.worlds`). */
  names: string;
  /** One line about it, if the content has no taglines for `names`. */
  blurb: string;

  /** The main town: its box, lots and lanes, how its roads are laid out, and the land round it (`width` × `depth` m; `water`: the side the roads left for it). */
  town: SizeSpec;
  shape: Shape;
  landscape(rand: Rand, width: number, depth: number, water?: 'zmin' | 'zmax'): LandscapeParams;

  /** The world's size (m, both ways; rounded to a multiple of 64). */
  world: [number, number];
  /** Hamlets besides the main town, and what they are like. */
  hamlets: [number, number];
  hamlet: HamletSpec;
  /**
   * The main town's terraced houses: so many lots in rows of `rows` houses (two: a
   * semi-detached pair), each row on one lot's frontage (`LotPlacer.placeRow`), off the high
   * street; and how built up the town is, 0..1 (`dense.ts`: the fewer gaps along its streets the more).
   */
  terraces: [number, number];
  rows: [number, number];
  density: number;
  /** The biggest town size whose civic places the main town has (`places.ts` `CIVIC`). */
  civic: TownSize;
  /** Country roads out of the world. */
  exits: [number, number];

  /** Where the water is: ponds out in the country, the sea along one edge, a lake, a river across, the sea all round. */
  water: 'ponds' | 'sea' | 'lake' | 'river' | 'island';
  /** Ponds (with `water: 'ponds'`). */
  ponds: [number, number];
  /** Share of the land in woods, and how hilly it is (m). */
  woods: number;
  hills: number;
  /** A valley down the middle, mountains either side this high (m). */
  massif?: [number, number];
  /** Farmland round the areas and the farmsteads, as far as this past an area's box (m); how much of it is fields (the noise's cut: lower, more). */
  fields?: number;
  fieldsCut: number;
  /** A farmstead along a country road every so far (m). */
  farmEvery: [number, number];
  /** Outdoor places, in the order areas take them (`PublicTemplate` ids), and how many each area has. */
  spots: string[];
  spotsPerArea: [number, number];
  /** One of the worlds from before places and options (`CLASSIC`). */
  classic?: true;

  // What a place with options has besides (none of it in a classic world).
  /** Meadow where the noise is over this (0.58 if not given: lower, more; a heath's heather). */
  meadowCut?: number;
  /** What a metre of height costs a country road, in cells of flat open land (hilly land and mountains: the roads keep to the valleys). */
  climb?: number;
  /** Woods grow into the hamlets' boxes up to their roads, lots and buildings (Wooded), the main town's too (`all`) when it is a hamlet; else every box is cleared. */
  wildBoxes?: 'hamlets' | 'all';
}

/** A place as step 1 lists it, and the options it starts with. */
export interface Place {
  label: string;
  blurb: string;
  defaults: WorldOptions;
}


/** A green moved a little in hue and brightness (so each world's grass is its own). */
function tint(hex: string, rand: Rand, warm: number, light: number): string {
  const n = parseInt(hex.slice(1), 16);
  const c = [(n >> 16) & 255, (n >> 8) & 255, n & 255];
  const out = [c[0] * (1 + warm) * light, c[1] * light, c[2] * (1 - warm) * light].map((v) => Math.max(0, Math.min(255, Math.round(v * (0.97 + rand() * 0.06)))));
  return `#${out.map((v) => v.toString(16).padStart(2, '0')).join('')}`;
}

/** Grass for a world: a base palette, warmer or cooler, lighter or darker. */
export function grass(rand: Rand, base: LandscapeParams['grass'], warm: [number, number], light: [number, number]): LandscapeParams['grass'] {
  const w = range(rand, ...warm);
  const l = range(rand, ...light);
  return base.map((g) => tint(g, rand, w, l)) as LandscapeParams['grass'];
}

/** Grass palettes: the classic valley's, the mountains', the coast's, the heath's (heather where the meadows are, bracken on the hills). */
export const GREEN = DEFAULT_LANDSCAPE.grass;
export const ALPINE: LandscapeParams['grass'] = ['#557F3A', '#6E9142', '#8C9A55', '#4F7238'];
export const COAST: LandscapeParams['grass'] = ['#7AA046', '#93AD55', '#B2B46A', '#6C9142'];
export const HEATH: LandscapeParams['grass'] = ['#76783F', '#7F9146', '#86687F', '#937046'];

/** The classic valley, rolled a little differently for each world. */
export const valley = (rand: Rand): LandscapeParams => ({
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
  const gap = (r: RoadPlan) => pmath.hypot(r.points[r.points.length - 1][0] - target[0], r.points[r.points.length - 1][1] - target[1]);
  return gap(lanes[0]) <= gap(lanes[1]) ? lanes[0] : lanes[1];
}

/** One winding high street, a back lane looping off it and back, a short lane off the other side. */
export function highStreet(rand: Rand, box: Box, scale = 1, avenue = false): RoadLayout {
  const main = avenueIf(through(rand, box, { amp: range(rand, 6, 11), bends: 2 }), avenue);
  const L = lengthOf(main);
  const s = side(rand);
  const s0 = L * range(rand, 0.28, 0.4);
  const back = loop(main, s0, Math.min(L - 30, s0 + range(rand, 70, 90) * scale), s, range(rand, 30, 38) * scale);
  const lane = branch(main, L * range(rand, 0.45, 0.65), -s as 1 | -1, range(rand, 28, 40) * scale, range(rand, -0.7, 0.7));
  return { roads: [main, back.road, lane] };
}

/** A through road with a crescent round a green; lanes off it joined up into a back street and to the crescent. */
export function crescentTown(rand: Rand, box: Box, scale = 1, avenue = false): RoadLayout {
  const main = avenueIf(through(rand, box, { at: 0.55, amp: range(rand, 6, 10), bends: 2 }), avenue);
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
 * lanes off each road for `connect` to join up. A city's (`avenue`): its main road an avenue straight
 * through, the cross road meeting it from either side (a crossroads with lights), no roundabout.
 */
export function crossroads(rand: Rand, box: Box, scale = 1, avenue = false): RoadLayout {
  const main = avenueIf(through(rand, box, { amp: range(rand, 4, 8), bends: 2 }), avenue);
  const cross = through(rand, box, { along: 'z', amp: range(rand, 4, 8), bends: 2 });
  const at = crossing(main, cross) ?? ([box.w / 2, box.d / 2] as Pt);
  const R = 6.5;
  // (The cross road cut right by the avenue's middle: `settleRoads` takes it back to its kerb.)
  const roads: RoadPlan[] = avenue ? [main, ...cutAround(cross, at, AVENUE_CUT)] : [...cutAround(main, at, R), ...cutAround(cross, at, R), roundabout(at[0], at[1], R)];
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

/** Where a road meets a city's avenue across it, it is cut this near (m) the avenue's middle (`crossroads`). */
const AVENUE_CUT = 2;

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
function waterfront(rand: Rand, box: Box, at: number, amp: [number, number], avenue = false) {
  const water = rand() < 0.5 ? 'zmin' : 'zmax';
  const main = avenueIf(through(rand, box, { at: water === 'zmin' ? at : 1 - at, amp: range(rand, ...amp), bends: 2 }), avenue);
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
export function shoreRoad(rand: Rand, box: Box, scale = 1, avenue = false): RoadLayout {
  const { main, water, land, centre, L } = waterfront(rand, box, 0.12, [3, 6], avenue);
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
export function switchback(rand: Rand, box: Box, scale = 1, avenue = false): RoadLayout {
  const main = avenueIf(through(rand, box, { amp: range(rand, 16, 24), bends: 3, width: 6 }), avenue);
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
export function lakeRoad(rand: Rand, box: Box, scale = 1, avenue = false): RoadLayout {
  const { main, water, land, centre, L } = waterfront(rand, box, 0.14, [5, 9], avenue);
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
  // (The roads' lengths and their index, worked out again only when a street is added.)
  const length = new Map<RoadPlan, number>();
  const lengthOfRoad = (r: RoadPlan) => length.get(r) ?? length.set(r, lengthOf(r)).get(r)!;
  let index = new RoadIndex(out);
  for (let n = 0, tries = 0; n < count && tries < count * 40; tries++) {
    const parents = out.filter((r) => !r.island && r.kind !== 'trail' && lengthOfRoad(r) > 40);
    if (!parents.length) break;
    const parent = parents[Math.floor(rand() * parents.length)];
    const from = out.indexOf(parent);
    const { p, t } = pointAt(parent.points, lengthOfRoad(parent) * range(rand, 0.1, 0.9));
    const sd = parent.lots ?? side(rand);
    const n0 = leftOf(t);
    const a = range(rand, -0.3, 0.3);
    const dir: Pt = [(n0[0] * pmath.cos(a) - n0[1] * pmath.sin(a)) * sd, (n0[0] * pmath.sin(a) + n0[1] * pmath.cos(a)) * sd];
    const street = crossStreet(out, index, from, p, dir, fit, 110 * scale);
    if (!street) continue;
    out.push(street);
    index = new RoadIndex(out);
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
  // Points 10 m apart over the box, each as far from the nearest road's middle as it is (200 m at
  // most; 0 within `EMPTY` of one, for good: roads are only ever added). Worked out over the roads
  // once, then brought down by each street laid since (`measured`: the roads they count).
  const points: { x: number; z: number; d: number }[] = [];
  for (let z = 34; z <= box.d - 34; z += 10) for (let x = 34; x <= box.w - 34; x += 10) points.push({ x, z, d: 0 });
  let measured = 0;
  for (let n = 0, tries = 0; n < count && tries < count * 4; tries++) {
    const index = new RoadIndex(out);
    for (const q of points) {
      if (!measured) {
        // (Most points are near a road: asked how near only when not.)
        q.d = index.nearest(q.x, q.z, EMPTY) ? 0 : (index.nearest(q.x, q.z, 200)?.dist ?? 200);
        continue;
      }
      for (let r = measured; r < out.length && q.d > EMPTY; r++) q.d = Math.min(q.d, roadDistance(out[r], q.x, q.z));
    }
    measured = out.length;
    let best: { p: Pt; d: number } | null = null;
    for (const { x, z, d } of points) {
      if (tried.some((t) => pmath.hypot(t[0] - x, t[1] - z) < 40) || greens.some((g) => inside(g, x, z)) || d <= EMPTY) continue;
      if (!best || d > best.d) best = { p: [x, z], d };
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

/** The places, as step 1 lists them, and the options each starts with. */
export const WORLDS: Record<WorldKind, Place> = {
  country: {
    label: 'Countryside',
    blurb: 'Rolling country, hedged lanes and ponds, villages strung along the roads.',
    defaults: { town: 'town', villages: 'many', hills: 'rolling', woods: 'some', size: 'large' },
  },
  farm: {
    label: 'Farmland',
    blurb: 'Flat open fields, farmsteads with barns and orchards along every road.',
    defaults: { town: 'village', villages: 'few', hills: 'flat', woods: 'open', size: 'medium' },
  },
  seaside: {
    label: 'Seaside',
    blurb: 'Homes along a sandy shore, the sea down one side of the world.',
    defaults: { town: 'village', villages: 'few', hills: 'rolling', woods: 'some', size: 'medium' },
  },
  lakeside: {
    label: 'Lakeside',
    blurb: 'A broad lake in the middle of the land, homes along its shore.',
    defaults: { town: 'town', villages: 'few', hills: 'rolling', woods: 'some', size: 'medium' },
  },
  river: {
    label: 'River valley',
    blurb: 'A winding river across the land, homes on its banks, bridges over it.',
    defaults: { town: 'town', villages: 'few', hills: 'rolling', woods: 'some', size: 'medium' },
  },
  mountain: {
    label: 'Mountains',
    blurb: 'A hamlet in a high valley, pines on steep slopes and peaks all round.',
    defaults: { town: 'hamlet', villages: 'few', hills: 'mountains', woods: 'wooded', size: 'medium' },
  },
  forest: {
    label: 'Forest',
    blurb: 'A clearing in deep woods, big gardens and birdsong.',
    defaults: { town: 'hamlet', villages: 'few', hills: 'rolling', woods: 'wooded', size: 'medium' },
  },
  heath: {
    label: 'Heath',
    blurb: 'Open moorland, heather and bracken on high bare hills, few trees.',
    defaults: { town: 'hamlet', villages: 'few', hills: 'hilly', woods: 'open', size: 'medium' },
  },
  island: {
    label: 'Island',
    blurb: 'An island with the sea all round, a causeway to the mainland.',
    defaults: { town: 'village', villages: 'few', hills: 'rolling', woods: 'some', size: 'medium' },
  },
};

/**
 * World kinds from before places and options, and the place and town size each is now: a save
 * or a tool that names one (`generateWorld`) gets that place.
 */
export const LEGACY_KINDS: Record<string, { kind: WorldKind; town?: TownSize }> = {
  small: { kind: 'country', town: 'village' },
  medium: { kind: 'country', town: 'town' },
  dense: { kind: 'country', town: 'city' },
};

/** The place a kind names: a place as it is, a legacy kind's place (`LEGACY_KINDS`), else the countryside. */
export function placeOf(kind: string): WorldKind {
  if (Object.hasOwn(WORLDS, kind)) return kind as WorldKind;
  return Object.hasOwn(LEGACY_KINDS, kind) ? LEGACY_KINDS[kind].kind : 'country';
}

/** The options a place starts with. */
export function defaultsOf(kind: WorldKind): WorldOptions {
  return { ...WORLDS[kind].defaults };
}

/** The worlds from before places and options: the kinds a world was, each with its own tables. */
export type ClassicKind = 'small' | 'medium' | 'dense' | 'farm' | 'seaside' | 'mountain' | 'lakeside' | 'forest';

/** A classic world's hamlet: a high street with a back lane, or a crescent round a green, its streets reaching all of its box. */
const HAMLET: HamletSpec = { shapes: [highStreet, crescentTown], box: { w: 220, d: 160 }, lots: [16, 24], places: [1, 1], scale: 1.1, fill: 2, extend: 1 };

/** What every classic world has alike: every civic place, two or three ways out, a farmstead every 150 to 200 m. */
const CLASSIC_BUILT: Pick<WorldSpec, 'civic' | 'exits' | 'farmEvery' | 'fieldsCut' | 'ponds' | 'classic'> = {
  civic: 'town',
  exits: [2, 3],
  farmEvery: [150, 200],
  fieldsCut: 0.4,
  ponds: [1, 2],
  classic: true,
};

/**
 * The classic worlds, as they were laid out before places and options: the pinned tests' worlds
 * (`tests/support/classicWorlds.ts`) and, for now, the places' (`specOf`).
 */
export const CLASSIC: Record<ClassicKind, WorldSpec> = {
  small: {
    ...CLASSIC_BUILT,
    kind: 'small',
    names: 'small',
    blurb: 'Homes along one winding street, a park and a shop or two.',
    town: { box: { w: 330, d: 256 }, lots: 70, scale: 1.5, extend: 5, fill: 4, places: [2, 3] },
    shape: highStreet,
    landscape: valley,
    gaps: [0.4, 0.1],
    small: 0.25,
    large: 0.2,
    gardens: { snug: 1, roomy: 3, big: 1 },
    places: [1, 2],
    world: [896, 960],
    hamlets: [5, 6],
    hamlet: HAMLET,
    terraces: [95, 110],
    rows: [2, 5],
    density: 0.6,
    water: 'ponds',
    woods: 0.42,
    hills: 14,
    fields: 100,
    spots: ['picnicMeadow', 'lookout', 'orchard', 'hammockGrove', 'fishingJetty'],
    spotsPerArea: [1, 1],
  },
  medium: {
    ...CLASSIC_BUILT,
    kind: 'medium',
    names: 'medium',
    blurb: 'A crescent of homes round the village green, lanes off the main road.',
    town: { box: { w: 366, d: 340 }, lots: 82, scale: 1.45, extend: 6, fill: 5, places: [4, 4] },
    shape: crescentTown,
    landscape: valley,
    gaps: [0.35, 0.08],
    small: 0.25,
    large: 0.2,
    gardens: { snug: 1, roomy: 3, big: 1 },
    places: [3, 3],
    world: [896, 960],
    hamlets: [5, 6],
    hamlet: HAMLET,
    terraces: [105, 120],
    rows: [2, 5],
    density: 0.65,
    water: 'ponds',
    woods: 0.42,
    hills: 14,
    fields: 100,
    spots: ['picnicMeadow', 'fishingJetty', 'lookout', 'orchard', 'hammockGrove'],
    spotsPerArea: [1, 1],
  },
  dense: {
    ...CLASSIC_BUILT,
    kind: 'dense',
    names: 'dense',
    blurb: 'A crossroads and crooked lanes of small lots, shops on every corner.',
    town: { box: { w: 390, d: 390 }, lots: 110, scale: 1.3, extend: 7, fill: 6, places: [6, 6] },
    shape: crossroads,
    landscape: (rand) => ({ ...valley(rand), hills: range(rand, 6, 11), forest: range(rand, -0.18, -0.06), treeline: range(rand, 6, 12) }),
    gaps: [0.12, 0.02],
    small: 0.6,
    large: 0.05,
    gardens: { snug: 4, roomy: 1 },
    places: [5, 5],
    prefer: ['cafe', 'grocer', 'bakery', 'market'],
    world: [960, 1024],
    hamlets: [5, 7],
    hamlet: HAMLET,
    terraces: [125, 140],
    rows: [3, 6],
    density: 0.85,
    water: 'ponds',
    woods: 0.34,
    hills: 10,
    fields: 100,
    spots: ['picnicMeadow', 'orchard', 'lookout', 'fishingJetty'],
    spotsPerArea: [1, 1],
  },
  farm: {
    ...CLASSIC_BUILT,
    kind: 'farm',
    names: 'farm',
    blurb: 'A country road, farmsteads with fields and barns, open land all round.',
    // (Its main town is a crossroads; its hamlets are the country lanes.)
    town: { box: { w: 330, d: 330 }, lots: 60, scale: 1.15, extend: 5, fill: 4, places: [2, 3], farms: [0, 0] },
    shape: crossroads,
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
    gaps: [0.5, 0.3],
    small: 0.1,
    large: 0.35,
    gardens: { roomy: 1, big: 3 },
    places: [1, 2],
    prefer: ['market', 'grocer'],
    farms: [3, 4],
    world: [960, 1024],
    hamlets: [5, 7],
    hamlet: { shapes: [countryLane], box: { w: 250, d: 180 }, lots: [16, 20], farms: [2, 3], places: [0, 0], scale: 1, fill: 2 },
    terraces: [95, 110],
    rows: [2, 3],
    density: 0.45,
    water: 'ponds',
    ponds: [3, 3],
    woods: 0.26,
    hills: 7,
    fields: 150,
    fieldsCut: 0.32,
    spots: ['orchard', 'picnicMeadow', 'fishingJetty', 'lookout'],
    spotsPerArea: [1, 1],
  },
  seaside: {
    ...CLASSIC_BUILT,
    kind: 'seaside',
    names: 'seaside',
    blurb: 'Homes above a sandy shore, a café and the smell of the sea.',
    town: { box: { w: 410, d: 280 }, lots: 82, scale: 1.45, extend: 8, fill: 4, places: [4, 4] },
    shape: shoreRoad,
    landscape: (rand, _width, _depth, side) => ({
      ...valley(rand),
      hills: range(rand, 8, 14),
      forest: range(rand, -0.15, -0.02),
      conifers: range(rand, 0.4, 0.7),
      birches: range(rand, 0.1, 0.3),
      grass: grass(rand, COAST, [0, 0.06], [1, 1.06]),
      water: { kind: 'sea', side: side ?? (rand() < 0.5 ? 'zmin' : 'zmax'), shore: range(rand, 4, 7) },
    }),
    gaps: [0.3, 0.08],
    small: 0.4,
    large: 0.1,
    gardens: { snug: 2, roomy: 2 },
    places: [3, 3],
    prefer: ['cafe', 'market', 'bakery'],
    world: [960, 1024],
    hamlets: [5, 6],
    hamlet: HAMLET,
    terraces: [110, 125],
    rows: [2, 5],
    density: 0.65,
    water: 'sea',
    woods: 0.32,
    hills: 14,
    fields: 90,
    spots: ['fishingJetty', 'lookout', 'picnicMeadow', 'hammockGrove'],
    spotsPerArea: [1, 1],
  },
  mountain: {
    ...CLASSIC_BUILT,
    kind: 'mountain',
    names: 'mountain',
    blurb: 'A few big gardens in a high valley, pines on steep slopes all round.',
    town: { box: { w: 354, d: 305 }, lots: 72, scale: 1.45, extend: 8, fill: 4, places: [2, 3] },
    shape: switchback,
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
    gaps: [0.45, 0.25],
    small: 0.05,
    large: 0.55,
    gardens: { roomy: 1, big: 3 },
    places: [1, 2],
    prefer: ['cafe', 'bakery'],
    world: [960, 1024],
    hamlets: [5, 6],
    hamlet: HAMLET,
    terraces: [85, 95],
    rows: [2, 3],
    density: 0.45,
    water: 'ponds',
    woods: 0.54,
    hills: 10,
    massif: [120, 120],
    spots: ['lookout', 'campfire', 'hammockGrove', 'fishingJetty'],
    spotsPerArea: [1, 1],
  },
  lakeside: {
    ...CLASSIC_BUILT,
    kind: 'lakeside',
    names: 'lakeside',
    blurb: 'A town by still water, reeds and a sandy bank, gentle hills.',
    town: { box: { w: 390, d: 316 }, lots: 84, scale: 1.4, extend: 8, fill: 5, places: [3, 4] },
    shape: lakeRoad,
    landscape: (rand, width, depth, side) => {
      const base = valley(rand);
      const rx = range(rand, 24, 38);
      const rz = range(rand, 16, 26);
      // Behind the town or in front of it (where the view across the town shows it, and its road runs), past its flat apron.
      const behind = side ? side === 'zmin' : rand() < 0.5;
      const water = { kind: 'lake' as const, x: width * range(rand, 0.3, 0.7), z: behind ? -(10 + rz) : depth + 10 + rz, rx, rz };
      return { ...base, hills: range(rand, 7, 12), birches: range(rand, 0.55, 0.8), water };
    },
    gaps: [0.35, 0.1],
    small: 0.2,
    large: 0.25,
    gardens: { roomy: 3, big: 1 },
    places: [2, 3],
    prefer: ['cafe', 'market'],
    world: [960, 1024],
    hamlets: [5, 6],
    hamlet: HAMLET,
    terraces: [105, 120],
    rows: [2, 5],
    density: 0.6,
    water: 'lake',
    woods: 0.4,
    hills: 10,
    fields: 90,
    spots: ['fishingJetty', 'picnicMeadow', 'hammockGrove', 'lookout'],
    spotsPerArea: [1, 1],
  },
  forest: {
    ...CLASSIC_BUILT,
    kind: 'forest',
    names: 'forest',
    blurb: 'A clearing in deep woods: big gardens, few neighbours, birdsong.',
    town: { box: { w: 340, d: 305 }, lots: 66, scale: 1.4, extend: 8, fill: 4, places: [2, 2] },
    shape: clearing,
    landscape: (rand) => ({
      ...valley(rand),
      hills: range(rand, 8, 14),
      forest: range(rand, 0.28, 0.4),
      treeline: range(rand, 0, 1),
      conifers: range(rand, 0.45, 0.75),
      birches: range(rand, 0.5, 0.8),
      grass: grass(rand, GREEN, [-0.05, 0.02], [0.9, 0.98]),
    }),
    gaps: [0.45, 0.2],
    small: 0,
    large: 0.6,
    gardens: { roomy: 1, big: 4 },
    places: [1, 1],
    prefer: ['bakery', 'cafe'],
    world: [960, 1024],
    hamlets: [5, 6],
    hamlet: { shapes: [clearing], box: { w: 220, d: 190 }, lots: [16, 20], places: [1, 1], scale: 1.2, fill: 2 },
    terraces: [85, 100],
    rows: [2, 3],
    density: 0.45,
    water: 'ponds',
    ponds: [2, 2],
    woods: 0.78,
    hills: 12,
    spots: ['campfire', 'hammockGrove', 'lookout', 'fishingJetty', 'picnicMeadow'],
    spotsPerArea: [1, 2],
  },
};

/**
 * A main town of each size (`specOf`): the area of its box (m², the shape of the place's own; none:
 * the place's own box, lots and lanes), its lots (a place with a bigger town of its own takes the
 * more), how far its lanes reach, its cross streets and outer streets, its public places besides
 * the park; its terraced houses (lots, rows), and how built up it is (`dense.ts`).
 */
interface SizeTable {
  area?: number;
  lots?: [number, number];
  scale?: number;
  extend?: number;
  fill?: number;
  places?: [number, number];
  terraces: [number, number];
  rows: [number, number];
  density: number;
}
const SIZES: Record<TownSize, SizeTable> = {
  hamlet: { area: 42_000, lots: [20, 24], scale: 1.1, extend: 1, fill: 2, places: [1, 1], terraces: [0, 0], rows: [2, 2], density: 0.25 },
  village: { area: 70_000, lots: [46, 54], scale: 1.25, extend: 3, fill: 3, places: [2, 2], terraces: [18, 30], rows: [2, 3], density: 0.4 },
  town: { terraces: [90, 120], rows: [2, 5], density: 0.6 },
  city: { area: 152_000, lots: [104, 110], scale: 1.3, extend: 8, fill: 6, places: [6, 6], terraces: [125, 140], rows: [3, 6], density: 0.85 },
};
/** The town sizes, smallest first. */
export const TOWN_SIZES: readonly TownSize[] = ['hamlet', 'village', 'town', 'city'];
/** A main town at town size has its place's lots, between these (`mainTown`). */
const TOWN_LOTS: [number, number] = [66, 84];

/**
 * The fewest homes (its houses' lots, the dense pass's terraced houses and homes on tight gardens among them) a
 * main town of each size has by its homes (`sizeByHomes`): a few more than the most the size below lays. Laid by
 * the generator, a main town of each size measured 28–74, 48–115, 71–191 and 123–257 homes over the nine places
 * and six seeds, 2026-11-20. The sizes overlap since the dense pass lays homes besides the houses' lots: a main
 * town is its place's size first (`places.ts` `draftAreaSize`), and its homes only ever make it bigger.
 */
const GROWN_FROM: Record<TownSize, number> = { hamlet: 0, village: 80, town: 120, city: 200 };

/** How big a main town of `homes` homes is: the biggest size whose fewest homes it has (`GROWN_FROM`), so a town the generator laid is its own size or below. */
export function sizeByHomes(homes: number): TownSize {
  return TOWN_SIZES.reduce<TownSize>((out, size) => (homes >= GROWN_FROM[size] ? size : out), 'hamlet');
}

/** The land a new world has (m, both ways) by its size (`WorldOptions.size`). */
export const LAND_SIZE: Record<LandSize, number> = { small: 640, medium: 832, large: 960 };
/** The land sizes, smallest first. */
const LAND_SIZES: readonly LandSize[] = ['small', 'medium', 'large'];

/**
 * The smallest land a main town and the villages round it fit on, as the generator lays them
 * (measured over the places, 2026-11-16): a small land holds a hamlet or a village alone (a town
 * by a lake has no room on it); a medium one any town with a few villages; many villages want a
 * large one (on a medium land only three or four of five are placed).
 */
const LEAST_LAND: Record<WorldOptions['villages'], Record<TownSize, LandSize>> = {
  none: { hamlet: 'small', village: 'small', town: 'medium', city: 'medium' },
  few: { hamlet: 'medium', village: 'medium', town: 'medium', city: 'medium' },
  many: { hamlet: 'large', village: 'large', town: 'large', city: 'large' },
};

/** Whether a main town of `town` and `villages` round it fit on land of `size` (`LEAST_LAND`). */
export function townFits(size: LandSize, town: TownSize, villages: WorldOptions['villages']): boolean {
  return LAND_SIZES.indexOf(size) >= LAND_SIZES.indexOf(LEAST_LAND[villages][town]);
}

/** The smallest land a main town of `town` and `villages` round it fit on (`townFits`); the largest if none does. */
export function sizeFor(town: TownSize, villages: WorldOptions['villages']): LandSize {
  return LAND_SIZES.find((s) => townFits(s, town, villages)) ?? 'large';
}

/**
 * How hilly (`specOf`): the generator's hills (m), the mountains' height either side of the
 * valley (m), the steepest the drawn land rises from anything built (`LandscapeParams.slope`), the
 * landscape's own small hills times this (`swell`), and what a metre of height costs a road (`climb`).
 */
const HILLS: Record<WorldOptions['hills'], { hills: number; massif?: [number, number]; slope: number; swell: number; climb?: number }> = {
  flat: { hills: 3, slope: 0.15, swell: 0.4 },
  rolling: { hills: 9, slope: 0.25, swell: 1 },
  hilly: { hills: 20, slope: 0.4, swell: 1.2, climb: 0.05 },
  mountains: { hills: 12, massif: [120, 160], slope: 0.75, swell: 1, climb: 0.04 },
};

/** How wooded (`specOf`): the share of the land in woods before the towns and roads are cleared out of it, and trees out on open land (`LandscapeParams.scatter`). */
const WOODS: Record<WorldOptions['woods'], { share: number; scatter: number }> = {
  open: { share: 0.1, scatter: 0.5 },
  some: { share: 0.4, scatter: 1 },
  wooded: { share: 0.72, scatter: 1.2 },
};

/** A value by town size, or the same at every size. */
type BySize<T> = T | Record<TownSize, T>;
const bySize = <T>(v: BySize<T>, size: TownSize): T => (v && typeof v === 'object' && 'hamlet' in v ? (v as Record<TownSize, T>)[size] : (v as T));

/**
 * A place (`specOf`): the classic world a main town of each size starts from (its box's shape; at
 * town size its box, lots and lanes; its lot mix), the town's shape and its name list by size, how
 * its land looks (drawn from the look's own stream), farm lots in the main town by size, its land
 * (water, ponds, farmland, outdoor places, hamlets), trees out on open land times the Woods
 * option's, and more for the landscape.
 */
interface PlaceTable {
  from: BySize<ClassicKind>;
  shape: BySize<Shape>;
  names: BySize<string>;
  look: WorldSpec['landscape'];
  farms?: Partial<Record<TownSize, [number, number]>>;
  mix?: Partial<LotMix>;
  land: Pick<WorldSpec, 'water' | 'ponds' | 'spots' | 'spotsPerArea' | 'hamlet'> & Partial<Pick<WorldSpec, 'fields' | 'fieldsCut' | 'farmEvery' | 'meadowCut'>>;
  scatter?: number;
  more?: Partial<LandscapeParams>;
}

/** A river valley's land: the classic valley, lusher and with more birches by the water. */
const riverLook = (rand: Rand): LandscapeParams => ({
  ...valley(rand),
  hills: range(rand, 8, 13),
  forest: range(rand, -0.06, 0.06),
  birches: range(rand, 0.6, 0.85),
  grass: grass(rand, GREEN, [-0.05, 0.02], [0.98, 1.06]),
});

/** A heath's land: heather and bracken, few trees and those mostly birches and pines, bushes among them. */
const heathLook = (rand: Rand): LandscapeParams => ({
  ...valley(rand),
  hills: range(rand, 10, 16),
  forest: range(rand, -0.25, -0.12),
  treeline: range(rand, 4, 8),
  conifers: range(rand, 0.55, 0.8),
  birches: range(rand, 0.55, 0.8),
  grass: grass(rand, HEATH, [-0.02, 0.05], [0.95, 1.04]),
});

/** A heath's hamlets: a high street with a back lane, now and then a crescent round a green. */
const HEATH_HAMLET: HamletSpec = { shapes: [highStreet, highStreet, crescentTown], box: { w: 220, d: 160 }, lots: [14, 20], places: [1, 1], scale: 1.1, fill: 2, extend: 1 };

const PLACES: Record<WorldKind, PlaceTable> = {
  country: {
    from: { hamlet: 'small', village: 'small', town: 'medium', city: 'dense' },
    shape: { hamlet: highStreet, village: highStreet, town: crescentTown, city: crossroads },
    names: { hamlet: 'small', village: 'small', town: 'medium', city: 'dense' },
    look: valley,
    land: { water: 'ponds', ponds: [1, 2], fields: 100, spots: ['picnicMeadow', 'fishingJetty', 'lookout', 'orchard', 'hammockGrove'], spotsPerArea: [1, 1], hamlet: HAMLET },
  },
  farm: {
    from: 'farm',
    shape: { hamlet: countryLane, village: countryLane, town: crossroads, city: crossroads },
    names: 'farm',
    look: (rand, w, d, side) => CLASSIC.farm.landscape(rand, w, d, side),
    farms: { hamlet: [2, 3], village: [3, 4], town: [1, 2], city: [0, 1] },
    land: {
      water: 'ponds',
      ponds: [3, 3],
      fields: 220,
      fieldsCut: 0.32,
      farmEvery: [110, 150],
      spots: ['orchard', 'picnicMeadow', 'fishingJetty', 'lookout'],
      spotsPerArea: [1, 1],
      hamlet: CLASSIC.farm.hamlet,
    },
  },
  seaside: {
    from: 'seaside',
    shape: shoreRoad,
    names: 'seaside',
    look: (rand, w, d, side) => CLASSIC.seaside.landscape(rand, w, d, side),
    land: { water: 'sea', ponds: [0, 0], fields: 90, spots: ['fishingJetty', 'lookout', 'picnicMeadow', 'hammockGrove'], spotsPerArea: [1, 1], hamlet: HAMLET },
  },
  lakeside: {
    from: 'lakeside',
    shape: lakeRoad,
    names: 'lakeside',
    look: (rand, w, d, side) => CLASSIC.lakeside.landscape(rand, w, d, side),
    land: { water: 'lake', ponds: [0, 0], fields: 90, spots: ['fishingJetty', 'picnicMeadow', 'hammockGrove', 'lookout'], spotsPerArea: [1, 1], hamlet: HAMLET },
  },
  river: {
    from: 'lakeside',
    shape: lakeRoad,
    names: 'river',
    look: riverLook,
    land: { water: 'river', ponds: [0, 0], fields: 90, spots: ['fishingJetty', 'picnicMeadow', 'lookout', 'hammockGrove'], spotsPerArea: [1, 1], hamlet: HAMLET },
    more: { beach: 3 },
  },
  mountain: {
    from: 'mountain',
    shape: switchback,
    names: 'mountain',
    look: (rand, w, d, side) => CLASSIC.mountain.landscape(rand, w, d, side),
    land: { water: 'ponds', ponds: [1, 2], spots: ['lookout', 'campfire', 'hammockGrove', 'fishingJetty'], spotsPerArea: [1, 1], hamlet: HAMLET },
  },
  forest: {
    from: 'forest',
    shape: clearing,
    names: 'forest',
    look: (rand, w, d, side) => CLASSIC.forest.landscape(rand, w, d, side),
    land: { water: 'ponds', ponds: [2, 2], spots: ['campfire', 'hammockGrove', 'lookout', 'fishingJetty', 'picnicMeadow'], spotsPerArea: [1, 2], hamlet: CLASSIC.forest.hamlet },
  },
  heath: {
    from: 'small',
    shape: highStreet,
    names: 'heath',
    look: heathLook,
    mix: { gaps: [0.45, 0.2], small: 0.2, large: 0.3, gardens: { roomy: 2, big: 2 }, places: [1, 2], prefer: ['cafe', 'bakery'] },
    land: { water: 'ponds', ponds: [2, 3], fields: 60, meadowCut: 0.45, spots: ['lookout', 'picnicMeadow', 'campfire'], spotsPerArea: [1, 1], hamlet: HEATH_HAMLET },
    scatter: 0.4,
    more: { scrub: 0.6 },
  },
  island: {
    from: 'seaside',
    shape: shoreRoad,
    names: 'island',
    look: (rand, w, d, side) => CLASSIC.seaside.landscape(rand, w, d, side),
    land: { water: 'island', ponds: [0, 0], fields: 90, spots: ['fishingJetty', 'lookout', 'picnicMeadow', 'hammockGrove'], spotsPerArea: [1, 1], hamlet: HAMLET },
  },
};

/**
 * The main town of a place at town size `size`, from the classic world's at town size (`base`):
 * as it is at town size; else a box of the size's area in the same shape, the size's lots (the more
 * the bigger the place's own town), lanes and streets; farm lots `farms` if given. A city's main
 * through road is an avenue; a town and a city have a few one-way pairs (`ONE_WAY_PAIRS`).
 */
function mainTown(base: SizeSpec, size: TownSize, farms?: [number, number]): SizeSpec {
  const s = SIZES[size];
  const roads = { ...(size === 'city' ? { avenue: true } : {}), ...(ONE_WAY_PAIRS[size] ? { oneway: ONE_WAY_PAIRS[size] } : {}) };
  if (!s.area || !s.lots) return { ...base, ...roads, ...(farms ? { farms } : {}) };
  const w = Math.round(Math.sqrt((s.area * base.box.w) / base.box.d));
  const k = Math.max(0, Math.min(1, (base.lots - TOWN_LOTS[0]) / (TOWN_LOTS[1] - TOWN_LOTS[0])));
  return {
    box: { w, d: Math.round(s.area / w) },
    lots: Math.round(s.lots[0] + (s.lots[1] - s.lots[0]) * k),
    scale: s.scale ?? base.scale,
    extend: s.extend,
    fill: s.fill,
    places: s.places,
    ...(farms ?? base.farms ? { farms: farms ?? base.farms } : {}),
    ...roads,
  };
}

/** Pairs of streets made one-way in a main town's dense middle, at most, by its size (`SizeSpec.oneway`): none in a hamlet or a village. */
const ONE_WAY_PAIRS: Record<TownSize, number> = { hamlet: 0, village: 0, town: 2, city: 3 };

/** Undefined options taken out (so they don't hide the place's own). */
const given = (options: Partial<WorldOptions>) => Object.fromEntries(Object.entries(options).filter(([, v]) => v !== undefined)) as Partial<WorldOptions>;

/**
 * A place's options in full: `options` over the place's own (`defaultsOf`), and its land's size as
 * chosen; else the place's own with its own town and villages, or the smallest land the town and
 * villages chosen fit on (`sizeFor`).
 */
export function optionsFor(kind: WorldKind, options: Partial<WorldOptions> = {}): WorldOptions & { size: LandSize } {
  const chosen = given(options);
  const own = defaultsOf(kind);
  const o = { ...own, ...chosen };
  const size = chosen.size ?? (o.town === own.town && o.villages === own.villages && own.size ? own.size : sizeFor(o.town, o.villages));
  return { ...o, size };
}

/**
 * A place with its options as the generator lays it out: `options` over the place's own
 * (`defaultsOf`), from the tables above. The main town's size sets its box, lots and lanes, its
 * terraced houses, how built over it is and its civic places; the villages how many hamlets and how
 * big the world; the hills how high the land rises (mountains either side of a valley, or round the
 * water) and how steep it is drawn; the woods how much of the land is wooded and how many trees
 * stand out in the open. The place sets the rest: its town's shape, its names, its water and how its
 * land looks (drawn from a stream of its own, so the palette and the trees stay as an option changes).
 */
export function specOf(kind: WorldKind, options: Partial<WorldOptions> = {}): WorldSpec {
  const o = optionsFor(kind, options);
  const p = PLACES[kind];
  const size = o.town;
  const from = CLASSIC[bySize(p.from, size)];
  const s = SIZES[size];
  const hills = HILLS[o.hills];
  const woods = WOODS[o.woods];
  const island = p.land.water === 'island';
  const small = size === 'hamlet' || size === 'village';
  return {
    kind,
    options: o,
    names: bySize(p.names, size),
    blurb: WORLDS[kind].blurb,
    town: mainTown(from.town, size, p.farms?.[size]),
    shape: bySize(p.shape, size),
    landscape: (rand, width, depth, side) => {
      // (The massif's height is the generator's: `macro.ts` sets `mountains` from it.)
      const { mountains: _mountains, ...look } = p.look(rand, width, depth, side);
      return { ...look, hills: look.hills * hills.swell, slope: hills.slope, scatter: woods.scatter * (p.scatter ?? 1), ...p.more };
    },
    gaps: from.gaps,
    small: from.small,
    large: from.large,
    gardens: from.gardens,
    places: from.places,
    ...(from.prefer ? { prefer: from.prefer } : {}),
    ...(from.farms ? { farms: from.farms } : {}),
    ...p.mix,
    world: [LAND_SIZE[o.size], LAND_SIZE[o.size]],
    hamlets: o.villages === 'none' ? [0, 0] : o.villages === 'few' ? [2, 3] : island ? [3, 4] : [5, 7],
    hamlet: p.land.hamlet,
    terraces: s.terraces,
    rows: s.rows,
    density: s.density,
    civic: size,
    exits: island ? (small ? [1, 1] : [2, 2]) : [2, 3],
    water: p.land.water,
    ponds: p.land.ponds,
    woods: woods.share,
    hills: hills.hills,
    ...(hills.massif ? { massif: hills.massif } : {}),
    ...(p.land.fields ? { fields: p.land.fields } : {}),
    fieldsCut: p.land.fieldsCut ?? 0.4,
    farmEvery: p.land.farmEvery ?? [150, 200],
    spots: p.land.spots,
    spotsPerArea: p.land.spotsPerArea,
    ...(p.land.meadowCut ? { meadowCut: p.land.meadowCut } : {}),
    ...(hills.climb ? { climb: hills.climb } : {}),
    ...(o.woods === 'wooded' ? { wildBoxes: size === 'hamlet' ? ('all' as const) : ('hamlets' as const) } : {}),
  };
}

/**
 * What a world of a place with its options has at least (`tools/worlds/sample.ts` checks it): the
 * share of its land (less the water) built and farmed (the areas' boxes, the lots, the fields and
 * orchards), the most share of its 100 m squares with nothing in them (null: not checked; on a
 * heath its meadows, the heather, count) and how many of its quarters hold an area or homes; its
 * fewest homes; whether every career works in the main town
 * (a town's or a city's); an island's ways out of the world, its causeways (fewest, most; null:
 * not checked, a world with land at its edges has as many as its roads find).
 */
export interface Expected {
  built: number;
  empty: number | null;
  meadow: boolean;
  quarters: number;
  homes: number;
  careers: boolean;
  exits: [number, number] | null;
}
export function expectations(kind: WorldKind, options: Partial<WorldOptions> = {}): Expected {
  const o = optionsFor(kind, options);
  const v = o.villages;
  return {
    // (Wooded: the woods come up to the hamlets; among mountains, the towns keep to the valleys.)
    built: { none: 0.08, few: 0.15, many: 0.25 }[v] * (o.woods === 'wooded' ? 0.6 : 1) * (o.hills === 'mountains' ? 0.8 : 1),
    empty: { none: null, few: 0.2, many: 0.1 }[v],
    meadow: kind === 'heath',
    // (Among mountains, along the valley: across a half of the world, it may be.)
    quarters: Math.min({ none: 0, few: 3, many: 4 }[v], o.hills === 'mountains' ? 2 : 4),
    homes: { hamlet: 14, village: 40, town: 110, city: 150 }[o.town] + { none: 0, few: 25, many: 70 }[v],
    careers: o.town === 'town' || o.town === 'city',
    exits: kind === 'island' ? specOf(kind, o).exits : null,
  };
}

const pickOf = <T>(list: readonly T[], rand: Rand): T => list[Math.floor(rand() * list.length)];

/** A world's names from the content's list `names` (none: the town and street lists). */
function namesOf(content: Content, names: string): WorldNames | null {
  const list = content.names.worlds?.[names];
  return list && list.start.length && list.end.length && list.streets.length && list.ways.length ? list : null;
}

/** A name for a town from the content's list `names` (`WorldSpec.names`): a start and an end ("Gull" + "cove"). */
export function worldName(content: Content, names: string, rand: Rand): string {
  const list = namesOf(content, names);
  if (!list) return pickOf(content.names.towns ?? ['Willowbrook'], rand);
  return `${pickOf(list.start, rand)}${pickOf(list.end, rand)}`.replace(/(.)\1\1/, '$1$1');
}

/**
 * `count` different street names for a town from the content's list `names`, none of `taken`:
 * each with a first word of its own ("Mill Lane", "Orchard Way"), then other pairs of words, then
 * (a world of several places) ones with the place's name (`area`). The first 40 tries are rolled,
 * as they always were; the rest are taken in turn without rolling, so the town's later rolls stay
 * as they were.
 */
export function streetNames(content: Content, names: string, rand: Rand, count: number, taken: readonly string[] = [], area?: string): string[] {
  const list = namesOf(content, names);
  const out: string[] = [];
  const firstWord = (name: string) => name.split(' ')[0];
  const used = (name: string) => taken.includes(name) || out.includes(name);
  const firstUsed = (name: string) => taken.some((n) => firstWord(n) === firstWord(name)) || out.some((n) => firstWord(n) === firstWord(name));
  for (let tries = 0; out.length < count && tries < 40; tries++) {
    const name = list ? `${pickOf(list.streets, rand)} ${pickOf(list.ways, rand)}` : pickOf(content.names.streets ?? ['Main Street'], rand);
    if (!firstUsed(name)) out.push(name);
  }
  if (out.length < count) {
    // Every pair of words, in an order of this town's own.
    const starts = list ? list.streets : (content.names.streets ?? ['Main Street']).map(firstWord);
    const ways = list ? list.ways : ['Street', 'Lane', 'Road', 'Way', 'Close', 'Row'];
    const turn = taken.length * 7 + count;
    const pairs = starts.flatMap((_, i) => ways.map((_, j) => `${starts[(i + turn) % starts.length]} ${ways[(i + j) % ways.length]}`));
    for (const name of pairs) if (out.length < count && !firstUsed(name)) out.push(name);
    for (const name of pairs) if (out.length < count && !used(name)) out.push(name);
    for (const name of pairs) if (out.length < count && area && !used(`${name}, ${area}`)) out.push(`${name}, ${area}`);
  }
  while (out.length < count) out.push(`${out[0] ?? 'Main Street'} ${out.length + 1}`);
  return out;
}

/** Words in a tagline that say how big a place is, and the town sizes each fits. */
const SIZE_WORDS: readonly [RegExp, readonly TownSize[]][] = [
  [/\b(hamlet|handful)\b/i, ['hamlet']],
  [/\bvillage\b/i, ['hamlet', 'village']],
  [/\btown\b/i, ['town', 'city']],
  [/\bcity\b/i, ['city']],
];

/**
 * One line about a world: from the content's taglines for its `names`, else its `blurb`. A place's
 * world (one with options) leaves out the lines that name a town of another size ("A hamlet that
 * hears its own echo." is no city's); a classic world takes any.
 */
export function worldTagline(content: Content, spec: Pick<WorldSpec, 'names' | 'blurb' | 'options'>, rand: Rand): string {
  const lines = content.names.worlds?.[spec.names]?.taglines ?? [];
  if (!lines.length) return spec.blurb;
  const at = Math.floor(rand() * lines.length);
  const size = spec.options?.town;
  if (!size) return lines[at];
  // (The one drawn if it fits, else the next that does: a world keeps its line as its town grows, where it can.)
  const fits = (line: string) => SIZE_WORDS.every(([words, sizes]) => !words.test(line) || sizes.includes(size));
  for (let k = 0; k < lines.length; k++) if (fits(lines[(at + k) % lines.length])) return lines[(at + k) % lines.length];
  return spec.blurb;
}
