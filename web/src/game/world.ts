/**
 * Kinds of world a new game can be set in: towns of three sizes, a farm, a seaside village, a
 * mountain hamlet, a lakeside town and a clearing in the woods. A recipe says how the town is laid
 * out (streets, lots, public places, farms; `generateWorld` in `town.ts` lays it out) and what the
 * land round it is like; the world's seed rolls the rest, so no two worlds of a kind are alike: its
 * name, its streets, its tagline, how hilly and wooded it is, where the water lies.
 */

import type { Content, WorldNames } from '../content/content';
import { DEFAULT_LANDSCAPE, type LandscapeParams } from '../core/protocol';

export type WorldKind = 'small' | 'medium' | 'dense' | 'farm' | 'seaside' | 'mountain' | 'lakeside' | 'forest';
export const WORLD_KINDS: readonly WorldKind[] = ['small', 'medium', 'dense', 'farm', 'seaside', 'mountain', 'lakeside', 'forest'];

type Rand = () => number;

export interface WorldRecipe {
  label: string;
  blurb: string;
  /** Streets, one behind the other, each with a row of lots on both sides. */
  streets: number;
  /** Lots on each side of a street. */
  columns: number;
  /** Chances of a small and a large lot (the rest are medium). */
  small: number;
  large: number;
  /** Public places besides the town park (fewest, most), and the ones it would rather have. */
  places: [number, number];
  prefer?: string[];
  /** Farm lots (fewest, most). */
  farms?: [number, number];
  /** The land round a town of this size (`width` × `depth` m). */
  landscape(rand: Rand, width: number, depth: number): LandscapeParams;
}

/** `lo..hi`, evenly. */
const range = (rand: Rand, lo: number, hi: number) => lo + rand() * (hi - lo);

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

export const WORLDS: Record<WorldKind, WorldRecipe> = {
  small: {
    label: 'Small town',
    blurb: 'Homes along one long street, a park and a shop or two.',
    streets: 1,
    columns: 5,
    small: 0.25,
    large: 0.2,
    places: [1, 2],
    landscape: valley,
  },
  medium: {
    label: 'Medium town',
    blurb: 'Two streets of homes round a little centre.',
    streets: 2,
    columns: 5,
    small: 0.25,
    large: 0.2,
    places: [3, 3],
    landscape: valley,
  },
  dense: {
    label: 'Dense town',
    blurb: 'Three streets of small lots, shops on every corner.',
    streets: 3,
    columns: 7,
    small: 0.6,
    large: 0.05,
    places: [5, 5],
    prefer: ['cafe', 'grocer', 'bakery', 'market'],
    landscape: (rand) => ({ ...valley(rand), hills: range(rand, 6, 11), forest: range(rand, -0.18, -0.06), treeline: range(rand, 6, 12) }),
  },
  farm: {
    label: 'Farm',
    blurb: 'A country road, farmsteads with fields and barns, open land all round.',
    streets: 1,
    columns: 5,
    small: 0.1,
    large: 0.35,
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
    streets: 1,
    columns: 7,
    small: 0.4,
    large: 0.1,
    places: [3, 3],
    prefer: ['cafe', 'market', 'bakery'],
    landscape: (rand) => ({
      ...valley(rand),
      hills: range(rand, 8, 14),
      forest: range(rand, -0.15, -0.02),
      conifers: range(rand, 0.4, 0.7),
      birches: range(rand, 0.1, 0.3),
      grass: grass(rand, COAST, [0, 0.06], [1, 1.06]),
      water: { kind: 'sea', side: rand() < 0.5 ? 'zmin' : 'zmax', shore: range(rand, 4, 7) },
    }),
  },
  mountain: {
    label: 'Mountain hamlet',
    blurb: 'A few big gardens in a high valley, pines on steep slopes all round.',
    streets: 1,
    columns: 4,
    small: 0.05,
    large: 0.55,
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
    streets: 2,
    columns: 5,
    small: 0.2,
    large: 0.25,
    places: [2, 3],
    prefer: ['cafe', 'market'],
    landscape: (rand, width, depth) => {
      const base = valley(rand);
      const rx = range(rand, 24, 38);
      const rz = range(rand, 16, 26);
      // Behind the town or in front of it (where the view across the town shows it), past its flat apron.
      const behind = rand() < 0.5;
      const water = { kind: 'lake' as const, x: width * range(rand, 0.3, 0.7), z: behind ? -(10 + rz) : depth + 10 + rz, rx, rz };
      return { ...base, hills: range(rand, 7, 12), birches: range(rand, 0.55, 0.8), water };
    },
  },
  forest: {
    label: 'Forest clearing',
    blurb: 'A clearing in deep woods: big gardens, few neighbours, birdsong.',
    streets: 1,
    columns: 5,
    small: 0,
    large: 0.6,
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
