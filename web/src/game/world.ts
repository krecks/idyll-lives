/**
 * Kinds of world a new game can be set in: towns of three sizes, a farm, a seaside village, a
 * mountain hamlet, a lakeside town and a clearing in the woods. A recipe says how the town is laid
 * out (its roads, how many lots, public places, farms; `generateWorld` in `town.ts` lays it out)
 * and what the land round it is like; the world's seed rolls the rest, so no two worlds of a kind
 * are alike: its name, the way its roads wind, its tagline, how hilly and wooded it is, where the water lies.
 */

import type { Content, WorldNames } from '../content/content';
import { DEFAULT_LANDSCAPE, type LandscapeParams } from '../core/protocol';
import type { GardenSize } from './housegen';
import { branch, centroid, crossing, cutAround, lane, lengths, loop, range, ring, roundabout, through, type Box, type Pt, type RoadPlan } from './roads';

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

export interface WorldRecipe {
  label: string;
  blurb: string;
  /** The area its roads are laid out in (m; the town is cut to what it takes, at most 256 m). */
  box: Box;
  /** Lots in all (public places and farms among them); fewer if the roads haven't room. */
  lots: number;
  /** Chances of a gap of a metre or two after a lot, and of a stretch of meadow. */
  gaps: [number, number];
  /** The town's roads. */
  roads(rand: Rand, box: Box): RoadLayout;
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

/** One winding high street, a short cul-de-sac off it. */
function highStreet(rand: Rand, box: Box): RoadLayout {
  const main = through(rand, box, { amp: range(rand, 6, 11), bends: 2 });
  const L = lengthOf(main);
  const culDeSac = branch(main, L * range(rand, 0.4, 0.6), side(rand), range(rand, 28, 40), range(rand, -0.7, 0.7));
  return { roads: [main, culDeSac] };
}

/** A through road with a crescent round a green, and lanes off it. */
function crescentTown(rand: Rand, box: Box): RoadLayout {
  const main = through(rand, box, { at: 0.55, amp: range(rand, 6, 10), bends: 2 });
  const L = lengthOf(main);
  const mid = L * range(rand, 0.42, 0.55);
  const s = side(rand);
  const crescent = loop(main, mid - range(rand, 24, 30), mid + range(rand, 24, 30), s, range(rand, 34, 42));
  const lanes = [
    branch(main, mid + range(rand, -10, 10), -s as 1 | -1, range(rand, 50, 65), range(rand, -0.8, 0.8)),
    branch(main, L * range(rand, 0.17, 0.24), side(rand), range(rand, 40, 55), range(rand, -0.7, 0.7)),
    branch(main, L * range(rand, 0.76, 0.83), side(rand), range(rand, 40, 55), range(rand, -0.7, 0.7)),
  ];
  return { roads: [main, crescent.road, ...lanes], greens: [crescent.green], centre: centroid(crescent.green) };
}

/** A crossroads with a roundabout (a fountain in the middle), crooked lanes off both roads either side. */
function crossroads(rand: Rand, box: Box): RoadLayout {
  const main = through(rand, box, { amp: range(rand, 4, 8), bends: 2 });
  const cross = through(rand, box, { along: 'z', amp: range(rand, 4, 8), bends: 2 });
  const [L, C] = [lengthOf(main), lengthOf(cross)];
  const at = crossing(main, cross) ?? ([box.w / 2, box.d / 2] as Pt);
  const R = 6.5;
  const roads: RoadPlan[] = [...cutAround(main, at, R), ...cutAround(cross, at, R), roundabout(at[0], at[1], R)];
  for (const [parent, len, at] of [[main, L, 0.24], [main, L, 0.76], [cross, C, 0.24], [cross, C, 0.76]] as const) {
    roads.push(branch(parent, len * (at + range(rand, -0.04, 0.04)), 1, range(rand, 45, 60), range(rand, -1.1, 1.1)));
    roads.push(branch(parent, len * (at + range(rand, -0.06, 0.06)), -1, range(rand, 40, 55), range(rand, -1.1, 1.1)));
  }
  return { roads, centre: at };
}

/** A long country lane winding through the fields, a fork off it leading out of town. */
function countryLane(rand: Rand, box: Box): RoadLayout {
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

/** A road along the shore (homes on its landward side only), a lane climbing inland, a crescent up the hill. */
function shoreRoad(rand: Rand, box: Box): RoadLayout {
  const { main, water, land, centre, L } = waterfront(rand, box, 0.12, [3, 6]);
  const climb = branch(main, L * range(rand, 0.42, 0.58), land, range(rand, 55, 70), range(rand, -0.8, 0.8));
  // A crescent up the hill at one end of the village.
  const at = rand() < 0.5 ? L * 0.22 : L * 0.78;
  const crescent = loop(main, at - 22, at + 22, land, range(rand, 30, 38));
  return { roads: [main, climb, crescent.road], water, centre };
}

/** A lane winding up the valley in tight bends, tracks off it. */
function switchback(rand: Rand, box: Box): RoadLayout {
  const main = through(rand, box, { amp: range(rand, 16, 24), bends: 3, width: 6 });
  const L = lengthOf(main);
  const tracks = [branch(main, L * range(rand, 0.3, 0.45), side(rand), range(rand, 24, 34), range(rand, -0.8, 0.8)), branch(main, L * range(rand, 0.55, 0.7), side(rand), range(rand, 24, 34), range(rand, -0.8, 0.8))];
  return { roads: [main, ...tracks], centre: [box.w / 2, box.d / 2] };
}

/** A road round the lake's end of town (homes on its landward side), a crescent and lanes off it inland. */
function lakeRoad(rand: Rand, box: Box): RoadLayout {
  const { main, water, land, centre, L } = waterfront(rand, box, 0.14, [5, 9]);
  const crescent = loop(main, L * 0.5 - 24, L * 0.5 + 24, land, range(rand, 34, 42));
  const lanes = [0.22, 0.78].map((at) => branch(main, L * (at + range(rand, -0.04, 0.04)), land, range(rand, 75, 95), range(rand, -0.9, 0.9)));
  return { roads: [main, crescent.road, ...lanes], water, centre };
}

/** A lane round a clearing in the woods, a way out of it and a track into the trees. */
function clearing(rand: Rand, box: Box): RoadLayout {
  const [cx, cz] = [box.w / 2, box.d / 2];
  const round = ring(rand, cx, cz, range(rand, 30, 36), range(rand, 22, 28));
  // Out of the clearing from its southern edge (the way the menus look at it from), and a track into the trees.
  const bottom = round.points.reduce((a, b) => (b[1] > a[1] ? b : a));
  const wander = range(rand, -10, 10);
  const out = lane([bottom, [bottom[0] + wander * 0.4, bottom[1] + 22], [bottom[0] + wander, box.d + 24]], ['join', 'out'], 6);
  const top = round.points.reduce((a, b) => (b[1] < a[1] ? b : a));
  const track = lane([top, [top[0] + range(rand, -14, 14), top[1] - 20], [top[0] + range(rand, -24, 24), top[1] - 34]], ['join', 'dead']);
  return { roads: [round, out, track], greens: [round.points], centre: [cx, cz] };
}

export const WORLDS: Record<WorldKind, WorldRecipe> = {
  small: {
    label: 'Small town',
    blurb: 'Homes along one winding street, a park and a shop or two.',
    box: { w: 150, d: 110 },
    lots: 10,
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
    box: { w: 200, d: 190 },
    lots: 20,
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
    box: { w: 230, d: 230 },
    lots: 42,
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
    box: { w: 240, d: 170 },
    lots: 10,
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
    box: { w: 220, d: 130 },
    lots: 14,
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
    box: { w: 170, d: 150 },
    lots: 8,
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
    box: { w: 210, d: 170 },
    lots: 20,
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
    box: { w: 170, d: 150 },
    lots: 10,
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

/** `count` different street names for a town of this kind. */
export function streetNames(content: Content, kind: WorldKind, rand: Rand, count: number): string[] {
  const names = namesOf(content, kind);
  const out: string[] = [];
  for (let tries = 0; out.length < count && tries < 40; tries++) {
    const name = names ? `${pickOf(names.streets, rand)} ${pickOf(names.ways, rand)}` : pickOf(content.names.streets ?? ['Main Street'], rand);
    if (!out.some((n) => n.split(' ')[0] === name.split(' ')[0])) out.push(name);
  }
  while (out.length < count) out.push(`${out[0] ?? 'Main Street'} ${out.length + 1}`);
  return out;
}

/** One line about a town of this kind. */
export function worldTagline(content: Content, kind: WorldKind, rand: Rand): string {
  const lines = content.names.worlds?.[kind]?.taglines ?? [];
  return lines.length ? pickOf(lines, rand) : WORLDS[kind].blurb;
}
