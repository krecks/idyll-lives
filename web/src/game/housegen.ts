/**
 * Generated houses (0.18): every neighbour's home is laid out from `content/housegen.json`.
 *
 * A house class is picked by lot size (bedrooms, size, whether it has a living room, a second
 * bathroom or a second storey), then:
 * 1. the footprint is split into rooms (a guillotine plan: each cut divides the rooms left into
 *    two groups by the floor they want), so every room is a rectangle and the plan stays connected;
 * 2. doors join the rooms (a tree from the entrance, the living room its hub where it can be,
 *    no way through a bedroom, no bathroom off the room the front door opens into), the front
 *    door faces the street and a garden door leads out of a shared room at the back (else at a
 *    side); kitchen and living room may share an open space;
 * 3. two-storey houses get stairs along an outside wall of the living room, an upstairs hall over
 *    them, and the bedrooms and a bathroom off that hall; some houses get a basement: stairs down
 *    along a wall of the living room (or the entrance hall, the dining room, the kitchen), a hall
 *    at their foot and a room or two off it (a games room, a gym, a workshop...), under the house;
 * 4. windows go along the outside walls, daylight enough for every room that has one; kitchens
 *    get a counter run along one wall (sink, stove); furniture goes against the walls (doors and
 *    stairs kept clear, every piece reachable), tables stand free with chairs round them, small
 *    things go on tops (counters, tables, shelves), floor lamps and such keep to corners and out
 *    of doorways, every room gets ceiling lights enough for all of it at night, and a few things
 *    (a patio set) go into the garden, by the garden door. The furniture is for whoever lives
 *    there (`Residents`): a couple's bed for two, a bed each for everyone else, toys only where
 *    there are children, and spare bedrooms (and playrooms) as guest rooms, studies and such.
 *
 * Seeded: the same seed, lot size and config always give the same house (laid out the same for
 * any residents), so a town keeps only `gen:<lot size>:<seed>` per lot (see `town.ts`). Output
 * is a `HouseTemplate` in the layouts' frame (drawn for a medium lot, front to the street at
 * z = 0), with `upper` for storey 1 and `basement` for storey -1 (both on the same rows as the ground).
 */

import type { ObjectDef, SurfaceKind } from '../content/content';
import { approachCandidates, covers, freeSpots, frontTile, middleOf, overhangTiles, topOf, type Shape, type Shapes, type Top } from './surfaces';
import type { Door, HouseTemplate, Placement, PlannedRoom, Wall } from './town';

export interface HouseGenConfig {
  classes: Record<string, HouseClass>;
  /** Class weights per lot size. */
  lots: Record<string, Record<string, number>>;
  margins: { front: [number, number]; back: number; side: number };
  rooms: Record<Kind, RoomSpec>;
  /** Rooms besides bedrooms, kitchen, bathroom and living room, where the house has more floor than its rooms may take. */
  extras?: ExtraRoom[];
  openPlan: number;
  /** `daylight`: the share of each room's floor its windows should light by day (`windowTiles` each). */
  windows: { spacing: [number, number]; daylight?: number };
  stairs: string;
  /** `washingLine`: put in the garden of a home that washes with nowhere to dry the washing (`washesWithoutDrying`). */
  garden: { objects: string[]; count: [number, number]; sets?: TableSet[]; washingLine?: string };
  /** Ceiling lights for rooms whose own `ceiling` slots have none (the upstairs hall, the basement's). */
  lights?: string[];
  basement?: BasementSpec;
  /** How a house is furnished for the people living in it (see `Residents`). */
  residents?: ResidentsSpec;
  names: { first: string[]; second: Record<string, string[]> };
}

/**
 * Who lives in a house: `couples` (a bed for two), `adults` on their own and `children` (a bed
 * each), `babies` (a crib, which the simulation brings). The layout never depends on it, only
 * the furniture: undefined is an empty house.
 */
export interface Residents {
  couples: number;
  adults: number;
  children: number;
  babies: number;
}

/** A key for `residents` (empty: undefined). */
export function residentsKey(r: Residents | undefined): string {
  return r ? `${r.couples}.${r.adults}.${r.children}.${r.babies}` : '';
}

/**
 * Furnishing for the residents: the beds bedrooms get (`couple`, `single` for a room two share,
 * `child`, `bunk` for two children short of floor), what goes into a child's room besides
 * (`childRoom`), what a bedroom nobody sleeps in becomes (`spareRoom`, by weight: `guest`, a
 * bedroom with a single bed, else a room kind), and what only a home with children has (`kidsOnly`:
 * content categories and groups; a playroom without children becomes a spare room too).
 */
interface ResidentsSpec {
  beds: { couple: string[]; single: string[]; child: string[]; bunk?: string[] };
  childRoom?: Slot[];
  spareRoom: Record<string, number>;
  kidsOnly: { categories?: string[]; groups?: string[] };
}

interface HouseClass {
  label: string;
  bedrooms: [number, number];
  /** Chance of a living room. */
  living: number;
  /** Chance of a second bathroom. */
  extraBathroom: number;
  storeys: { one: number; two: number };
  width: [number, number];
  depth: [number, number];
}

type RoomKind = Kind | 'hall';
/** Rooms with a recipe in the config (`bedroom`, `kitchen`, `bathroom`, `living` and extras; halls have none). */
type Kind = string;

/**
 * A room a house may get on top of the ones it must have: only while its floor is bigger than
 * its rooms may take (`maxSide`), if `chance` (a number, or per lot size) says so, with at least
 * `minBedrooms`. `floor: "upper"` puts it upstairs in a two-storey house; `label` names it in the
 * house's description.
 */
interface ExtraRoom {
  kind: Kind;
  chance: number | Record<string, number>;
  minBedrooms?: number;
  floor?: 'ground' | 'upper';
  label?: string;
}

/**
 * Basements: a share of houses (`chance`, a number or per lot size) get one, with `count` rooms
 * of the kinds in `rooms` (picked by `weight`, none the house has already; `label` names them in
 * the house's description), off a hall at the foot of the stairs down.
 */
interface BasementSpec {
  chance: number | Record<string, number>;
  count: [number, number];
  rooms: { kind: Kind; weight?: number; label?: string }[];
}

interface RoomSpec {
  weight: number;
  minSide: number;
  /** Per lot size, a narrowest width instead of `minSide` (a large house's kitchen, room for an island). */
  minSideOn?: Record<string, number>;
  /** The most a room may measure (tiles: its short side, its long side); bigger floors get more rooms. */
  maxSide?: [number, number];
  /** Kitchens: a counter run along one wall, placed before the furniture. */
  run?: CounterRun;
  furniture: Slot[];
  /** Tables with chairs round them (or desks with a chair), after the furniture that must come. */
  dining?: TableSet[];
  /** Small things on the tops of what's in the room (content `layer: "surface"`), after the furniture. */
  surfaces?: TopSlot[];
  /**
   * Things that take no floor space (content `layer`), after the furniture: on the walls (their
   * back on a wall or a window), from the ceiling and rugs on the floor. Only things nobody uses
   * (pictures, lamps, rugs), so nothing needs to reach them.
   */
  wall?: Slot[];
  ceiling?: Slot[];
  rug?: Slot[];
}

interface Slot {
  one: string[];
  chance?: number;
  count?: [number, number];
  opposite?: string;
  /** On the same wall as another piece if there's room (`"run"`: the kitchen's counter run; a fridge at its end). */
  beside?: string;
  /** Things on walls: on the wall behind one of these if the room has one (a mirror over the basin). */
  over?: string[];
}

/** Counter pieces side by side along a wall: each of `with` (one id per slot) among plain `segments`, `length` tiles in all. */
interface CounterRun {
  segments: string[];
  with: Slot[];
  length: [number, number];
  /** Rows of room it leaves in front of it where a wall allows (an island and its stools). */
  clear?: number;
}

/**
 * A table and chairs facing it, all the same chair (`seats`: how many, as many as fit up to the
 * most; fewer than the least and the set isn't placed). It stands free in the room, unless
 * `wall` (desks): then against a wall with its chairs along its front.
 */
interface TableSet {
  tables: string[];
  chairs: string[];
  seats: [number, number];
  chance?: number;
  wall?: boolean;
  /** Chairs (stools at an island) along one long side only (the back of a table someone uses); a square one, two sides across from each other. */
  oneSide?: boolean;
  /** Only in a house with (`ifRoom`) or without (`unlessRoom`) a room of this kind. */
  ifRoom?: Kind;
  unlessRoom?: Kind;
}

/** Things to put on tops of the given kinds (`minSide`: only tops at least this wide and deep, m). */
interface TopSlot {
  on: SurfaceKind[];
  /** Only on these objects (an island's top). */
  of?: string[];
  one: string[];
  count?: [number, number];
  chance?: number;
  minSide?: number;
}

/** What the generator needs to know of an object type (from the game's content). */
export interface ObjectInfo extends Shape {
  /** `[w, d]` (along x and z, facing +z). */
  footprint: [number, number];
  /** The disc it takes on a top (m), its default and limits applied (`Content.radius`). */
  radius: number;
  /** Whether it has interactions or cooking stations: someone must be able to get to it. */
  used: boolean;
  /** Whether it gives light (content `light`): it counts for its room's light at night. */
  lamp: boolean;
  /** Tiles it lights at night (content `light.tiles`; undefined: `roomRules.lampTiles`). */
  lightTiles?: number;
  /** As tall as the wall: nothing hangs on the wall or from the ceiling over it (content `fillsWall`). */
  fillsWall?: boolean;
  /** Belongs over a window (content `overWindow`): only such things hang on one. */
  overWindow?: boolean;
  /** Built to stand back to a wall (content `againstWall`): only with plain wall all along its back. */
  againstWall?: boolean;
  /** Hangs from the ceiling at head height (content `headHeight`): nobody walks under it (its tiles are taken from the floor's way through). */
  headHeight?: boolean;
  /** Hangs from the ceiling low enough to walk into (content `hangsLow`). */
  hangsLow?: boolean;
  /** Tiles beside its own it reaches over at head height (content `overhang`: an arc lamp's shade over the next tile). */
  overhang?: [number, number][];
  /** A staircase. */
  stairs?: boolean;
  /** Sat or lain on (an interaction with pose `sit` or `lie`): a sofa, a chair, a bed. */
  seated?: boolean;
  /** A hob or a grill (stations `stove`, `grill`) or a sink (`dishes`, where hands are washed): no small things on its top. */
  keepClear: boolean;
  /** Content category and group (`residents.kidsOnly`: things only for a home with children). */
  category?: string;
  group?: string;
}

/** Whether object type `o` is a hob, a grill or a sink (see `ObjectInfo.keepClear`). */
function cooksOrWashes(o: ObjectDef): boolean {
  const stations = o.stations ?? [];
  const sink = stations.includes('dishes') && (o.interactions ?? []).some((i) => (i.tags ?? []).includes('handwash'));
  return sink || stations.includes('stove') || stations.includes('grill');
}

/** An object type's details, or undefined if it doesn't exist. */
export type ObjectInfos = (def: string) => ObjectInfo | undefined;

/** The generator's view of the game's content (`Content`, or the content files read by a tool). */
export function objectInfos(content: { object(id: string): ObjectDef | undefined; radius(def: ObjectDef | undefined): number }): ObjectInfos {
  const known = new Map<string, ObjectInfo | undefined>();
  return (def) => {
    if (!known.has(def)) {
      const o = content.object(def);
      known.set(def, o && { ...o, footprint: o.footprint ?? [1, 1], radius: content.radius(o), used: (o.interactions ?? []).length > 0 || (o.stations ?? []).length > 0, lamp: !!o.light, lightTiles: o.light?.tiles, keepClear: cooksOrWashes(o), seated: (o.interactions ?? []).some((i) => i.pose === 'sit' || i.pose === 'lie') });
    }
    return known.get(def);
  };
}

interface Rect {
  x: number;
  z: number;
  w: number;
  d: number;
}

interface Room extends Rect {
  kind: RoomKind;
}

/** One storey's plan while it's being made (lot coordinates). */
interface Floor {
  rooms: Room[];
  /** Unit wall edges: `h:x:z` (along x on line z) and `v:x:z` (along z on line x) → what stands there. */
  edges: Map<string, 'wall' | 'door' | 'window'>;
  objects: Placement[];
  /**
   * Where each of `objects` is used from: its front tile, or (chairs) any of the tiles it may be
   * approached from; null: nowhere that matters (garden things, things on walls and tops).
   */
  fronts: ([number, number][] | null)[];
  /** Tiles nothing may stand on (by doors, stairs, the landing). */
  keep: Set<string>;
  /** Of `keep`, the tiles along an open kitchen and living room's missing wall: an island's stools may stand on them. */
  open?: Set<string>;
  /** Tiles taken by furniture. */
  taken: Set<string>;
  /** Tiles taken on the wall, ceiling and rug layers (`wall:x:z`, ...). */
  layers?: Set<string>;
  /** Tiles open to the stairs down to the basement (the stairwell): nothing hangs or lies there either. */
  well?: Set<string>;
  /** Tiles a shade reaches over from beside (content `overhang`, an arc lamp's): nothing hangs low or at head height there. */
  shaded?: Set<string>;
}

const key = (x: number, z: number) => `${x}:${z}`;

/** A small seeded generator (mulberry32). */
export function random(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/**
 * `lampTiles`: tiles a lamp lights at night unless its `light.tiles` says (content
 * `roomRules.lampTiles`); every room gets lights enough for all of it. `windowTiles`: tiles one window lights by day (`roomRules.windowTiles`).
 */
export function generateHouse(cfg: HouseGenConfig, info: ObjectInfos, lot: { width: number; depth: number }, medium: { width: number }, lotSize: string, seed: number, lampTiles = 10, windowTiles = 6, residents?: Residents): HouseTemplate | null {
  // Rooms as narrow as their kind may be on this size of lot.
  const sized = (s: RoomSpec): RoomSpec => (s.minSideOn?.[lotSize] === undefined ? s : { ...s, minSide: s.minSideOn[lotSize] });
  cfg = { ...cfg, rooms: Object.fromEntries(Object.entries(cfg.rooms).map(([k, s]) => [k, sized(s)])) };
  // A few tries: some draws don't fit (too many rooms for the floor, no wall for the stairs).
  for (let attempt = 0; attempt < 12; attempt++) {
    // The basement draws numbers of its own: a house without one is laid out as it was before
    // there were any (as is one whose stairs down leave no room for its furniture, without them).
    const n = seed * 31 + attempt;
    let house = attemptHouse(cfg, info, lot, lotSize, random(n), random(n ^ 0x5bd1e995), lampTiles, windowTiles, residents);
    if (house === 'basement') house = attemptHouse(cfg, info, lot, lotSize, random(n), null, lampTiles, windowTiles, residents);
    if (!house || house === 'basement') continue;
    // Into the layouts' frame (centred on lots of other widths by `onLot`).
    const dx = -Math.floor((lot.width - medium.width) / 2);
    const shift = (h: { walls: Wall[]; doors: Door[]; windows?: Door[]; objects: Placement[]; rooms?: PlannedRoom[] }) => {
      h.walls = h.walls.map(([x0, z0, x1, z1]) => [x0 + dx, z0, x1 + dx, z1]);
      h.doors = h.doors.map((d) => ({ ...d, x: d.x + dx }));
      h.windows = (h.windows ?? []).map((d) => ({ ...d, x: d.x + dx }));
      h.objects = h.objects.map((o) => ({ ...o, x: o.x + dx }));
      h.rooms = (h.rooms ?? []).map((r) => ({ ...r, x: r.x + dx }));
    };
    shift(house);
    if (house.upper) shift(house.upper);
    if (house.basement) shift(house.basement);
    house.spawns = house.spawns.map(([x, z]) => [x + dx, z]);
    house.id = `gen:${lotSize}:${seed}`;
    return house;
  }
  return null;
}

/**
 * `list` shuffled in place (Fisher-Yates). Not `sort(() => rng() - 0.5)`: what a sort does with such
 * answers is up to the engine, and JavaScriptCore's gave one seed other houses from call to call.
 */
function shuffle<T>(rng: () => number, list: T[]): T[] {
  for (let i = list.length - 1; i > 0; i--) {
    const j = Math.floor(rng() * (i + 1));
    [list[i], list[j]] = [list[j], list[i]];
  }
  return list;
}

function pick<T>(rng: () => number, list: readonly T[]): T {
  return list[Math.floor(rng() * list.length)];
}

function between(rng: () => number, [a, b]: readonly [number, number]): number {
  return a + Math.floor(rng() * (b - a + 1));
}

function weighted(rng: () => number, weights: Record<string, number>): string | null {
  const entries = Object.entries(weights).filter(([, w]) => w > 0);
  const total = entries.reduce((n, [, w]) => n + w, 0);
  let r = rng() * total;
  for (const [k, w] of entries) if ((r -= w) <= 0) return k;
  return entries[entries.length - 1]?.[0] ?? null;
}

/**
 * `belowRng`: the random numbers for the basement (null: no basement). `'basement'`: there's no
 * room for the house's furniture with the stairs down (it may do without a basement).
 */
function attemptHouse(cfg: HouseGenConfig, info: ObjectInfos, lot: { width: number; depth: number }, lotSize: string, rng: () => number, belowRng: (() => number) | null, lampTiles: number, windowTiles: number, residents: Residents | undefined): HouseTemplate | null | 'basement' {
  const className = weighted(rng, cfg.lots[lotSize] ?? cfg.lots.medium ?? {});
  const cls = className ? cfg.classes[className] : undefined;
  if (!cls) return null;
  const bedrooms = between(rng, cls.bedrooms);
  const living = rng() < cls.living;
  const extraBath = rng() < cls.extraBathroom;
  const two = (weighted(rng, { one: cls.storeys.one, two: cls.storeys.two }) ?? 'one') === 'two';

  // Footprint, within the lot's garden margins.
  const { margins } = cfg;
  const front = between(rng, margins.front);
  const maxW = lot.width - 2 * margins.side;
  const maxD = lot.depth - front - margins.back;
  let W = Math.min(maxW, between(rng, cls.width));
  let D = Math.min(maxD, between(rng, cls.depth));
  if (W < 6 || D < 6) return null;

  // Who goes where.
  const ground: Kind[] = ['kitchen'];
  const upstairs: Kind[] = [];
  if (living || two) ground.push('living');
  if (two) {
    upstairs.push(...Array<Kind>(bedrooms).fill('bedroom'), 'bathroom');
    if (extraBath) ground.push('bathroom');
  } else {
    ground.push(...Array<Kind>(bedrooms).fill('bedroom'), 'bathroom');
    if (extraBath) ground.push('bathroom');
  }
  // Rooms have a most they may measure: a big floor gets more rooms (a dining room, a study...),
  // not bigger ones, and if there are none left to add, the house is smaller.
  const [baseGround, baseUp] = [ground.length, upstairs.length];
  const lists = two ? [ground, upstairs] : [ground];
  const floorOf = (kinds: Kind[], w = W, d = D) => (kinds === upstairs ? (w - 2) * (d - 1) : w * d);
  const most = (kinds: Kind[]) => kinds.reduce((n, k) => n + capArea(cfg, k), 0);
  const least = (kinds: Kind[]) => kinds.reduce((n, k) => n + cfg.rooms[k].minSide ** 2, 0);
  const stretched = (kinds: Kind[]) => most(kinds) < floorOf(kinds) * ROOM_SLACK;
  const add = (kind: Kind, list: Kind[]) => {
    if (stretched(list) && (kind === 'bathroom' || !list.includes(kind)) && (least(list) + cfg.rooms[kind].minSide ** 2) * LEAST_SLACK <= floorOf(list)) list.push(kind);
  };
  const chanceOf = (chance: number | Record<string, number>) => (typeof chance === 'number' ? chance : (chance[lotSize] ?? 0));
  const extras = (cfg.extras ?? []).filter((e) => cfg.rooms[e.kind] && bedrooms >= (e.minBedrooms ?? 0) && chanceOf(e.chance) > 0);
  for (const e of extras.filter((e) => rng() < chanceOf(e.chance))) add(e.kind, two && e.floor === 'upper' ? upstairs : ground);
  // Rooms need some floor besides their own (their shapes don't fit together perfectly).
  while (lists.some(stretched)) {
    const [w, d] = W >= D ? [W - 1, D] : [W, D - 1];
    if (w < 6 || d < 6 || lists.some((l) => least(l) * LEAST_SLACK > floorOf(l, w, d))) break;
    [W, D] = [w, d];
  }
  // Still stretched (a two-storey house's floors are as big as each other): more of the rooms
  // houses here may get, on whichever floor wants them.
  for (const e of extras) for (const list of lists) add(e.kind, list);
  // Extras a smaller house no longer needs.
  for (const list of lists) {
    const base = list === ground ? baseGround : baseUp;
    while (list.length > base && most(list) - capArea(cfg, list[list.length - 1]) >= floorOf(list) * ROOM_SLACK) list.pop();
  }
  // A floor that doesn't fall into rooms of their sizes: an extra room fewer, else a smaller house.
  // Doors that don't work out (the bathroom only off the room the front door opens into): the same.
  let g: Floor | null = null;
  let house: Rect = { x: 0, z: front, w: W, d: D };
  const openPlan = rng() < cfg.openPlan;
  for (let step = 0; step < 8; step++) {
    const slack = lot.width - 2 * margins.side - W;
    house = { x: margins.side + Math.floor(rng() * (slack + 1)), z: front, w: W, d: D };
    const at = house;
    g = floorPlan(cfg, rng, house, ground, (f) => frontDoor(f, at, rng) && connect(f, rng, openPlan));
    if (g) break;
    g = null;
    if (ground.length > baseGround && step % 2 === 0) ground.pop();
    else if (W >= D && W > 6) W--;
    else if (D > 6) D--;
    else break;
  }
  if (!g) return null;

  let up: Floor | null = null;
  if (two) {
    up = upperFloor(cfg, info, rng, house, g, upstairs);
    while (!up && upstairs.length > baseUp) {
      upstairs.pop();
      up = upperFloor(cfg, info, rng, house, g, upstairs);
    }
    if (!up) return null;
  }
  const has = new Set([...ground, ...upstairs]);
  // What the rooms become for the people living here (a draw of its own: the rest of the house is the same for anyone).
  const fit = fitFor(cfg, info, random((rng() * 2 ** 32) >>> 0), [...g.rooms, ...(up?.rooms ?? [])], residents);
  // A basement, if the house gets one and it fits (fewer rooms down there if they don't); with
  // `belowRng` null, none.
  let basement: Floor | null = null;
  const below: Kind[] = [];
  const spec = cfg.basement;
  if (spec && belowRng && belowRng() < chanceOf(spec.chance)) {
    // The kinds in an order drawn by weight: the first `count`, fewer if they don't fit, else one of the others alone.
    const n = between(belowRng, spec.count);
    const order: Kind[] = [];
    for (const options = spec.rooms.filter((r) => cfg.rooms[r.kind] && !has.has(r.kind)); options.length; ) {
      const kind = weighted(belowRng, Object.fromEntries(options.map((o) => [o.kind, o.weight ?? 1])))!;
      order.push(kind);
      options.splice(options.findIndex((o) => o.kind === kind), 1);
    }
    const plans = [...Array.from({ length: n }, (_, i) => order.slice(0, n - i)), ...order.slice(n).map((k) => [k])].filter((p) => p.length);
    for (const kinds of plans) {
      basement = basementFloor(cfg, info, belowRng, house, g, kinds, new Set([...has, ...kinds]), lampTiles, fit);
      if (!basement) continue;
      below.push(...kinds);
      break;
    }
    for (const k of below) has.add(k);
  }
  gardenDoor(g, house, rng);
  windows(cfg, rng, g, house, windowTiles);
  if (up) windows(cfg, rng, up, house, windowTiles);
  // A stairwell that leaves the ground floor without room for what must be there: try the house without a basement.
  if (!furnish(cfg, info, rng, g, has, lampTiles, fit)) return basement ? 'basement' : null;
  if (up && !furnish(cfg, info, rng, up, has, lampTiles, fit)) return null;
  garden(cfg, info, rng, g, house, lot, washesWithoutDrying(info, [g, up, basement]));

  // Spawns in front of the front door.
  const door = [...g.edges].find(([k, v]) => v === 'door' && k === `h:${k.split(':')[1]}:${house.z}`);
  const doorX = door ? Number(door[0].split(':')[1]) : house.x + Math.floor(W / 2);
  const spawns: [number, number][] = [
    [doorX + 0.5, house.z - 1.5],
    [doorX + 1.5, house.z - 1.5],
    [doorX - 0.5, house.z - 1.5],
    [doorX + 2.5, house.z - 1.5],
  ];

  const name = `${pick(rng, cfg.names.first)} ${pick(rng, cfg.names.second[cls.label] ?? ['House'])}`;
  const words = ['no', 'one', 'two', 'three', 'four', 'five', 'six'];
  const parts = [
    `${words[bedrooms] ?? bedrooms} bedroom${bedrooms === 1 ? '' : 's'}`,
    ground.includes('living') ? (hasOpenPlan(g) ? 'an open kitchen and living room' : 'a living room') : null,
    extraBath ? 'two bathrooms' : null,
    ...new Set([...ground.slice(baseGround), ...upstairs.slice(baseUp)].map((k) => cfg.extras?.find((e) => e.kind === k)?.label)),
    below.length ? `a basement with ${listOf(below.map((k) => spec?.rooms.find((r) => r.kind === k)?.label ?? `a ${k}`))}` : null,
  ].filter((p): p is string => !!p);
  const listed = listOf(parts);
  const description = `A ${two ? 'two-storey ' : ''}${cls.label} with ${listed}.`;

  const out = (f: Floor) => {
    const e = edgesOut(f);
    // The front door first (the garden path and driveway start from it).
    e.doors.sort((a, b) => Number(b.axis === 'x' && b.z === house.z) - Number(a.axis === 'x' && a.z === house.z));
    return { ...e, objects: f.objects, rooms: f.rooms.map(({ kind, x, z, w, d }) => ({ kind, x, z, w, d })) };
  };
  return {
    id: '',
    name,
    description,
    bedrooms,
    ...out(g),
    spawns,
    ...(up ? { upper: out(up) } : {}),
    ...(basement ? { basement: out(basement) } : {}),
  };
}

/** `a, b and c`. */
function listOf(parts: string[]): string {
  return parts.length > 1 ? `${parts.slice(0, -1).join(', ')} and ${parts[parts.length - 1]}` : parts[0];
}

// ---- floor plans ------------------------------------------------------------------------

/** How much more floor the rooms may take than a storey has: room for the plan to fall into rectangles. */
const ROOM_SLACK = 1.2;
/** How much more floor than its rooms' least a storey needs to fall into rooms at all. */
const LEAST_SLACK = 1.4;

/** The most a room of this kind may measure, `[short side, long side]`. */
function cap(cfg: HouseGenConfig, k: Kind): [number, number] {
  return cfg.rooms[k]?.maxSide ?? [Infinity, Infinity];
}

function capArea(cfg: HouseGenConfig, k: Kind): number {
  const [a, b] = cap(cfg, k);
  return a * b;
}

/** Whether a room of kind `k` may measure `w` by `d`. */
function fits(cfg: HouseGenConfig, k: Kind, w: number, d: number): boolean {
  const [a, b] = cap(cfg, k);
  return Math.min(w, d) >= cfg.rooms[k].minSide && Math.min(w, d) <= a && Math.max(w, d) <= b;
}

/** Splits `rect` among `kinds` (a guillotine plan) so that `doors` can join the rooms; null if they don't fit. */
function floorPlan(cfg: HouseGenConfig, rng: () => number, rect: Rect, kinds: Kind[], doors: (f: Floor) => boolean): Floor | null {
  for (let tries = 0; tries < 60; tries++) {
    const order = shuffle(rng, [...kinds]);
    const rooms = split(cfg, rng, rect, order);
    if (rooms) {
      const floor: Floor = { rooms, edges: new Map(), objects: [], fronts: [], keep: new Set(), taken: new Set() };
      walls(floor);
      if (doors(floor)) return floor;
    }
  }
  return null;
}

function split(cfg: HouseGenConfig, rng: () => number, r: Rect, kinds: Kind[]): Room[] | null {
  const spec = (k: Kind) => cfg.rooms[k];
  if (kinds.length === 1) return fits(cfg, kinds[0], r.w, r.d) ? [{ ...r, kind: kinds[0] }] : null;
  // Two groups, as even by wanted floor as a random cut allows.
  const at = 1 + Math.floor(rng() * (kinds.length - 1));
  const a = kinds.slice(0, at);
  const b = kinds.slice(at);
  const weight = (ks: Kind[]) => ks.reduce((n, k) => n + spec(k).weight, 0);
  const share = weight(a) / (weight(a) + weight(b));
  const most = (ks: Kind[]) => ks.reduce((n, k) => n + capArea(cfg, k), 0);
  // How long a group's side of the cut may be, `across` the other way: a room alone, as long as it may measure.
  const span = (ks: Kind[], across: number, len: number): [number, number] => {
    const lo = Math.max(...ks.map((k) => spec(k).minSide));
    if (ks.length > 1) return [lo, Math.min(len, Math.floor(most(ks) / across))];
    const [c0, c1] = cap(cfg, ks[0]);
    return across < lo || across > c1 ? [1, 0] : [lo, across > c0 ? c0 : c1];
  };
  // Cut across the longer side mostly; the other way if that can't work.
  const first = r.w > r.d ? rng() < 0.8 : rng() < 0.2;
  for (const alongX of [first, !first]) {
    const len = alongX ? r.w : r.d;
    const across = alongX ? r.d : r.w;
    const [aLo, aHi] = span(a, across, len);
    const [bLo, bHi] = span(b, across, len);
    const lo = Math.max(aLo, len - bHi);
    const hi = Math.min(aHi, len - bLo);
    if (lo > hi) continue;
    const cut = Math.max(lo, Math.min(hi, Math.round(len * share + (rng() - 0.5) * 1.5)));
    const [ra, rb]: [Rect, Rect] = alongX
      ? [{ ...r, w: cut }, { ...r, x: r.x + cut, w: r.w - cut }]
      : [{ ...r, d: cut }, { ...r, z: r.z + cut, d: r.d - cut }];
    const left = split(cfg, rng, ra, a);
    const right = left && split(cfg, rng, rb, b);
    if (left && right) return [...left, ...right];
  }
  return null;
}

/** Walls around every room (shared walls once). */
function walls(f: Floor): void {
  for (const r of f.rooms) {
    for (let x = r.x; x < r.x + r.w; x++) {
      f.edges.set(`h:${x}:${r.z}`, 'wall');
      f.edges.set(`h:${x}:${r.z + r.d}`, 'wall');
    }
    for (let z = r.z; z < r.z + r.d; z++) {
      f.edges.set(`v:${r.x}:${z}`, 'wall');
      f.edges.set(`v:${r.x + r.w}:${z}`, 'wall');
    }
  }
}

/** Edges two rooms share (unit edges, as keys). */
function shared(a: Rect, b: Rect): string[] {
  const out: string[] = [];
  if (a.x + a.w === b.x || b.x + b.w === a.x) {
    const x = a.x + a.w === b.x ? b.x : a.x;
    for (let z = Math.max(a.z, b.z); z < Math.min(a.z + a.d, b.z + b.d); z++) out.push(`v:${x}:${z}`);
  }
  if (a.z + a.d === b.z || b.z + b.d === a.z) {
    const z = a.z + a.d === b.z ? b.z : a.z;
    for (let x = Math.max(a.x, b.x); x < Math.min(a.x + a.w, b.x + b.w); x++) out.push(`h:${x}:${z}`);
  }
  return out;
}

/** The tiles on both sides of an edge. */
function sides(edge: string): [[number, number], [number, number]] {
  const [axis, xs, zs] = edge.split(':');
  const [x, z] = [Number(xs), Number(zs)];
  return axis === 'h' ? [[x, z - 1], [x, z]] : [[x - 1, z], [x, z]];
}

/** Puts a door on one of `edges` (not at a corner where possible) and keeps its tiles clear. */
function door(f: Floor, rng: () => number, edges: string[]): boolean {
  if (!edges.length) return false;
  const inner = edges.length > 2 ? edges.slice(1, -1) : edges;
  const e = pick(rng, inner);
  f.edges.set(e, 'door');
  for (const [x, z] of sides(e)) f.keep.add(key(x, z));
  return true;
}

/**
 * The front door, from the street into the entrance hall (false if the house has one that isn't
 * on the street) or the living room, else the kitchen or the dining room; never a bedroom or a bathroom.
 */
function frontDoor(f: Floor, house: Rect, rng: () => number): boolean {
  const facing = (r: Room) => r.z === house.z && r.w >= 2;
  if (f.rooms.some((r) => r.kind === 'entry' && !facing(r))) return false;
  const entry = ['entry', 'living', 'kitchen', 'dining'].map((k) => f.rooms.find((r) => r.kind === k && facing(r))).find((r) => r);
  if (!entry) return false;
  const edges = Array.from({ length: entry.w }, (_, i) => `h:${entry.x + i}:${house.z}`);
  return door(f, rng, edges);
}

const PUBLIC: RoomKind[] = ['entry', 'living', 'kitchen', 'dining'];

/**
 * Doors so every room can be reached: a tree from the entrance, the living room its hub where
 * possible (else the other shared rooms), nobody walking through a bathroom, a laundry or a
 * bedroom (but into its own bathroom), a bathroom never off the room the front door opens into and off the
 * kitchen only if nothing else will do. False if some room can't be reached that way.
 */
function connect(f: Floor, rng: () => number, openPlan: boolean): boolean {
  const rooms = f.rooms;
  const entry = rooms.find((r) => sides([...f.edges].find(([, v]) => v === 'door')?.[0] ?? '').some(([x, z]) => inside(r, x, z)))!;
  const reached = new Set<Room>([entry]);
  // Open plan: kitchen and living room as one space (their shared wall goes).
  if (openPlan) {
    const k = rooms.find((r) => r.kind === 'kitchen');
    const l = rooms.find((r) => r.kind === 'living');
    const wall = k && l ? shared(k, l) : [];
    if (wall.length >= 2) {
      // The way between them stays clear of furniture (but for stools pulled up to an island).
      f.open = new Set();
      for (const e of wall) {
        f.edges.delete(e);
        for (const [x, z] of sides(e)) {
          f.keep.add(key(x, z));
          f.open.add(key(x, z));
        }
      }
    }
  }
  while (reached.size < rooms.length) {
    let best: { from: Room; to: Room; edges: string[]; score: number } | null = null;
    for (const from of reached) {
      for (const to of rooms) {
        if (reached.has(to)) continue;
        const edges = shared(from, to);
        if (edges.length === 0) continue;
        const open = edges.every((e) => !f.edges.has(e));
        // Nobody walks through a bathroom or a laundry, nor a bedroom but into its own bathroom.
        if ((to.kind === 'bathroom' && from === entry) || from.kind === 'bathroom' || from.kind === 'laundry' || (from.kind === 'bedroom' && to.kind !== 'bathroom')) continue;
        // Rooms open to each other need no door; the others open off the living room, else the other shared rooms.
        const hub = from.kind === 'living' ? 5 : PUBLIC.includes(from.kind) ? 2 : 0;
        const kitchen = to.kind === 'bathroom' && from.kind === 'kitchen' ? -4 : 0;
        const score = (open ? 10 : 0) + hub + kitchen + rng();
        if (!best || score > best.score) best = { from, to, edges: edges.filter((e) => f.edges.has(e)), score };
      }
    }
    if (!best) return false;
    if (best.edges.length) door(f, rng, best.edges);
    reached.add(best.to);
  }
  return true;
}

/**
 * A second way out, into the back garden: a door in the back wall of the living room (else the
 * dining room, else the kitchen), clear of the stairs and the corners, leaving the room outside
 * wall for two windows at least (else one, the door at a corner if need be). With no such room at
 * the back, in one's side wall (its back half if it can be). None if no shared room will do.
 */
function gardenDoor(f: Floor, house: Rect, rng: () => number): void {
  const back = house.z + house.d;
  const kept = (e: string) => f.edges.get(e) === 'wall' && !sides(e).some(([x, z]) => f.taken.has(key(x, z)));
  // A room's outside walls (towards the street, the back, the left, the right), unit edges in order.
  const walls = (r: Room): [string[], string[], string[], string[]] => [
    r.z === house.z ? Array.from({ length: r.w }, (_, i) => `h:${r.x + i}:${house.z}`) : [],
    r.z + r.d === back ? Array.from({ length: r.w }, (_, i) => `h:${r.x + i}:${back}`) : [],
    r.x === house.x ? Array.from({ length: r.d }, (_, i) => `v:${house.x}:${r.z + i}`) : [],
    r.x + r.w === house.x + house.w ? Array.from({ length: r.d }, (_, i) => `v:${house.x + house.w}:${r.z + i}`) : [],
  ];
  // Where a door may go along `wall` (not at its ends unless `corners`; along a side, its back half only if `half`).
  const spots = (wall: string[], corners: boolean, half: boolean) =>
    wall.filter((e, i) => kept(e) && (corners || wall.length <= 2 || (i > 0 && i < wall.length - 1)) && (!half || Number(e.split(':')[2]) >= house.z + house.d / 2));
  const passes = [
    { corners: false, windows: 2, side: false, half: false },
    { corners: true, windows: 1, side: false, half: false },
    { corners: false, windows: 1, side: true, half: true },
    { corners: false, windows: 1, side: true, half: false },
  ];
  for (const { corners, windows, side, half } of passes) {
    for (const kind of ['living', 'dining', 'kitchen']) {
      for (const r of f.rooms.filter((r) => r.kind === kind)) {
        const all = walls(r);
        const free = all.flat().filter(kept);
        const edges = (side ? [all[2], all[3]] : [all[1]]).flatMap((w) => spots(w, corners, half));
        if (free.length - 1 < windows || !edges.length) continue;
        door(f, rng, [pick(rng, edges)]);
        return;
      }
    }
  }
}

function inside(r: Rect, x: number, z: number): boolean {
  return x >= r.x && z >= r.z && x < r.x + r.w && z < r.z + r.d;
}

function hasOpenPlan(f: Floor): boolean {
  const k = f.rooms.find((r) => r.kind === 'kitchen');
  const l = f.rooms.find((r) => r.kind === 'living');
  return !!k && !!l && shared(k, l).length > 0 && shared(k, l).every((e) => !f.edges.has(e));
}

/** Rooms the stairs may go up from. */
const STAIR_ROOMS: RoomKind[] = ['living', 'entry', 'dining'];
/** Rooms the stairs down to the basement may start in (a kitchen too, if none of those has room). */
const DOWN_ROOMS: RoomKind[] = [...STAIR_ROOMS, 'kitchen'];

/**
 * Stairs along an outside wall of the living room, the entrance hall or the dining room
 * (climbing away from the street), and the
 * storey above: a hall over the stairs, the other rooms in a row off it, each with a door; on a
 * floor too wide for that, a corridor from the hall across the floor with rooms either side.
 */
function upperFloor(cfg: HouseGenConfig, info: ObjectInfos, rng: () => number, house: Rect, g: Floor, kinds: Kind[]): Floor | null {
  const [, len] = info(cfg.stairs)?.footprint ?? [1, 4];
  // In the living room, the entrance hall or the dining room: a column on the left or right
  // outside wall, with room for the stairs and their foot.
  const spots: [number, Room][] = [];
  for (const room of shuffle(rng, g.rooms.filter((r) => STAIR_ROOMS.includes(r.kind)))) {
    for (const c of shuffle(rng, [house.x, house.x + house.w - 1])) if (c >= room.x && c < room.x + room.w) spots.push([c, room]);
  }
  for (const [c, room] of spots) {
    for (let z = room.z + room.d - len; z >= room.z + 1; z--) {
      const tiles = Array.from({ length: len }, (_, i) => key(c, z + i));
      const foot = key(c, z - 1);
      if ([...tiles, foot].some((t) => g.keep.has(t))) continue;
      const landing = z + len;
      // The hall upstairs: the stairs' column and the one beside it, the whole depth.
      const left = c === house.x;
      const hall: Rect = { x: left ? c : c - 1, z: house.z, w: 2, d: house.d };
      if (landing >= house.z + house.d) continue;
      const rest: Rect = left ? { x: c + 2, z: house.z, w: house.w - 2, d: house.d } : { x: house.x, z: house.z, w: house.w - 2, d: house.d };
      const up = offHall(cfg, rng, hall, rest, kinds);
      if (!up) continue;
      // Stairs below, their stairwell and landing above.
      g.objects.push({ def: cfg.stairs, x: c, z, rot: 2 });
      g.fronts.push([[c, z - 1]]);
      for (const t of [...tiles, foot]) g.keep.add(t);
      for (const t of tiles) g.taken.add(t);
      for (let i = 0; i < len; i++) up.taken.add(key(c, z + i));
      up.keep.add(key(c, landing));
      for (let zz = house.z; zz < house.z + house.d; zz++) up.keep.add(key(left ? c + 1 : c - 1, zz));
      return up;
    }
  }
  return null;
}

/**
 * A floor of `hall` with rooms of `kinds` in `rest` beside it, each with a door off the hall (or
 * off a corridor from it across `rest`, if they need one: see `upperRooms`); the hall and the
 * corridor are one space, with no furniture of their own. `across`: the hall runs along x, `rest`
 * before or behind it (upstairs, it runs along z with `rest` to a side). Null if the rooms don't fit.
 */
function offHall(cfg: HouseGenConfig, rng: () => number, hall: Rect, rest: Rect, kinds: Kind[], across = false): Floor | null {
  // Planned turned a quarter (x for z) if the hall runs along x.
  const turn = <T extends Rect>(r: T): T => (across ? { ...r, x: r.z, z: r.x, w: r.d, d: r.w } : r);
  const plan = upperRooms(cfg, rng, turn(rest), kinds);
  if (!plan) return null;
  const rooms = plan.rooms.map(turn);
  const corridor = plan.corridor && turn(plan.corridor);
  const halls: Room[] = [{ ...hall, kind: 'hall' }, ...(corridor ? [{ ...corridor, kind: 'hall' as const }] : [])];
  const f: Floor = { rooms: [...halls, ...rooms], edges: new Map(), objects: [], fronts: [], keep: new Set(), taken: new Set() };
  walls(f);
  if (corridor) for (const e of shared(hall, corridor)) f.edges.delete(e);
  for (const r of rooms) {
    const edges = (corridor && shared(corridor, r).length ? shared(corridor, r) : null) ?? shared(hall, r);
    if (!door(f, rng, edges)) return null;
  }
  return f;
}

/** How many basement plans (`offHall`) are tried at most: for one place of the stairs, and in all. */
const BASEMENT_TRIES = [40, 400];

/**
 * A basement under the house: stairs down from a ground-floor room (the living room, the
 * entrance hall or the dining room; else the kitchen), along one of its walls (its only outside
 * wall, which its windows need, if nothing else will do; never that of a room open to the next),
 * climbing either way, their top and landing in the room (which stays walkable round the
 * stairwell). Below, a hall two tiles wide holding the stairs, at least as long as they and
 * their foot, and the rooms of `kinds` off it (as upstairs) on the side away from the stairs,
 * under the house; the basement as big as they allow. They're furnished here (`has`: the rooms the house has); only
 * once it all fits does the ground keep the stairwell and landing clear. Null if it doesn't.
 */
function basementFloor(cfg: HouseGenConfig, info: ObjectInfos, rng: () => number, house: Rect, g: Floor, kinds: Kind[], has: Set<Kind>, lampTiles: number, fit: Fit): Floor | null {
  const [, len] = info(cfg.stairs)?.footprint ?? [1, 4];
  // Sizes the rooms fill side by side along the hall, each across the whole width (`row`).
  const longest = (k: Kind, w: number) => {
    const [a, b] = cap(cfg, k);
    return w < cfg.rooms[k].minSide || w > b ? 0 : w > a ? a : b;
  };
  const could = (w: number, d: number) => kinds.every((k) => longest(k, w) >= cfg.rooms[k].minSide) && d >= kinds.reduce((n, k) => n + cfg.rooms[k].minSide, 0) && d <= kinds.reduce((n, k) => n + longest(k, w), 0);
  // Worked out along z, the stairs in column `u` climbing along `v`; stairs along x (`across`)
  // the same with x and z swapped.
  // `dark`: along the room's only outside wall, which its windows need.
  type Spot = { across: boolean; u: number; first: boolean; room: Room; dark: boolean };
  const spots: Spot[] = [];
  for (const room of shuffle(rng, g.rooms.filter((r) => DOWN_ROOMS.includes(r.kind)))) {
    // Its outside walls (towards the street, then clockwise).
    const out = [room.z === house.z, room.x + room.w === house.x + house.w, room.z + room.d === house.z + house.d, room.x === house.x];
    const only = out.filter(Boolean).length === 1;
    // Never there in a kitchen or living room open to the other (its windows light both).
    const opened = [...(g.open ?? [])].some((t) => inside(room, ...(t.split(':').map(Number) as [number, number])));
    for (const across of [false, true]) {
      const [u0, n] = across ? [room.z, room.d] : [room.x, room.w];
      // Against the room's first wall (the hall past the stairs, the rooms past it), or its last.
      const [low, high] = [only && out[across ? 0 : 3], only && out[across ? 2 : 1]];
      if (!(low && opened)) spots.push({ across, u: u0, first: true, room, dark: low });
      if (!(high && opened)) spots.push({ across, u: u0 + n - 1, first: false, room, dark: high });
    }
  }
  // The kitchen last (it wants its floor for an island), and a room's only outside wall only if nothing else will do.
  const last = (s: Spot) => 2 * Number(s.dark) + Number(s.room.kind === 'kitchen');
  shuffle(rng, spots).sort((a, b) => last(a) - last(b));
  let total = 0;
  for (const { across, u, first, room } of spots) {
    const at = (a: number, b: number): [number, number] => (across ? [b, a] : [a, b]);
    const turn = (r: Rect): Rect => (across ? { x: r.z, z: r.x, w: r.d, d: r.w } : r);
    const [R, H] = [turn(room), turn(house)];
    const starts = shuffle(rng, Array.from({ length: Math.max(0, R.d - len + 1) }, (_, i) => R.z + i));
    for (const v of starts) {
      for (const up of shuffle(rng, [true, false])) {
        // Climbing towards higher `v` (the foot before the steps, the landing past them), or lower.
        const lv = up ? v + len : v - 1;
        const foot = up ? v - 1 : v + len;
        if (!inside(R, u, lv) || foot < H.z || foot >= H.z + H.d) continue;
        const well = Array.from({ length: len }, (_, i) => key(...at(u, v + i)));
        const landing = key(...at(u, lv));
        if ([...well, landing].some((t) => g.keep.has(t) || g.taken.has(t))) continue;
        // The room above stays walkable round the stairwell, to its landing.
        for (const t of well) g.taken.add(t);
        g.keep.add(landing);
        const open = reachable(g, room);
        for (const t of well) g.taken.delete(t);
        g.keep.delete(landing);
        if (!open) continue;
        // The rooms beside the hall, under the house (the biggest that work); the hall the stairs'
        // column and the one beside it (kept clear), along the steps and their foot and the rooms.
        const wide = first ? H.x + H.w - u - 2 : u - 1 - H.x;
        const [top, bottom] = [Math.min(v, foot), Math.max(v + len - 1, foot)];
        const sizes: { z: number; d: number; w: number; hz: number; hd: number; score: number }[] = [];
        for (let z0 = H.z; z0 < H.z + H.d; z0++) {
          for (let z1 = z0 + 1; z1 <= H.z + H.d; z1++) {
            const [hz, hd] = [Math.min(z0, top), Math.max(z1, bottom + 1) - Math.min(z0, top)];
            for (let w = 2; w <= wide; w++) if (could(w, z1 - z0)) sizes.push({ z: z0, d: z1 - z0, w, hz, hd, score: w * (z1 - z0) - (hd - (z1 - z0)) + rng() });
          }
        }
        sizes.sort((a, b) => b.score - a.score);
        let tries = 0;
        for (const size of sizes) {
          if (tries++ >= BASEMENT_TRIES[0] || total >= BASEMENT_TRIES[1]) break;
          total++;
          const hall = turn({ x: first ? u : u - 1, z: size.hz, w: 2, d: size.hd });
          const rest = turn({ x: first ? u + 2 : u - 1 - size.w, z: size.z, w: size.w, d: size.d });
          const b = offHall(cfg, rng, hall, rest, kinds, across);
          if (!b) continue;
          // The stairs as placed: their corner, turned to climb the right way (they climb away from their front).
          const [x, z] = at(u, v);
          b.objects.push({ def: cfg.stairs, x, z, rot: across ? (up ? 3 : 1) : up ? 2 : 0 });
          b.fronts.push([at(u, foot)]);
          for (const t of well) b.taken.add(t);
          b.keep.add(key(...at(u, foot)));
          for (let s = size.hz; s < size.hz + size.hd; s++) b.keep.add(key(...at(first ? u + 1 : u - 1, s)));
          if (!furnish(cfg, info, rng, b, has, lampTiles, fit)) continue;
          // The stairwell above, and its landing.
          for (const t of well) g.taken.add(t);
          g.keep.add(landing);
          g.well = new Set(well);
          return b;
        }
        if (total >= BASEMENT_TRIES[1]) return null;
      }
    }
  }
  return null;
}

/**
 * The rooms upstairs besides the hall: a row of rooms the width of the floor, each off the hall;
 * else (the floor is wider than its rooms may be) a corridor across the floor from the hall,
 * a row of rooms on either side of it.
 */
function upperRooms(cfg: HouseGenConfig, rng: () => number, rest: Rect, kinds: Kind[]): { rooms: Room[]; corridor: Rect | null } | null {
  const plain = row(cfg, rng, rest, kinds, false);
  if (plain) return { rooms: plain, corridor: null };
  for (const cw of [1, 2, 3]) {
    for (let tries = 0; tries < 8; tries++) {
      const da = Math.floor((rest.d - cw) / 2) + (tries % 2);
      const a: Rect = { ...rest, d: da };
      const b: Rect = { ...rest, z: rest.z + da + cw, d: rest.d - da - cw };
      if (a.d < 2 || b.d < 2) continue;
      // Rooms to the two sides by the floor they want.
      const sides: [Kind[], Kind[]] = [[], []];
      const want = [0, 0];
      for (const k of shuffle(rng, [...kinds])) {
        const i = want[0] <= want[1] ? 0 : 1;
        sides[i].push(k);
        want[i] += cfg.rooms[k].weight;
      }
      if (!sides[0].length || !sides[1].length) continue;
      const ra = row(cfg, rng, a, sides[0], true);
      const rb = ra && row(cfg, rng, b, sides[1], true);
      if (ra && rb) return { rooms: [...ra, ...rb], corridor: { ...rest, z: rest.z + da, d: cw } };
    }
  }
  return null;
}

/** Rooms side by side in `r` (along x, else along z), each across the whole of it; null if they don't fit. */
function row(cfg: HouseGenConfig, rng: () => number, r: Rect, kinds: Kind[], alongX: boolean): Room[] | null {
  const len = alongX ? r.w : r.d;
  const across = alongX ? r.d : r.w;
  // The most of the row a room may take, being `across` the other way.
  const longest = (k: Kind) => {
    const [a, b] = cap(cfg, k);
    return across > b ? 0 : across > a ? a : b;
  };
  if (kinds.some((k) => across < cfg.rooms[k].minSide || longest(k) < cfg.rooms[k].minSide)) return null;
  for (let tries = 0; tries < 6; tries++) {
    const order = shuffle(rng, [...kinds]);
    const weight = order.reduce((n, k) => n + cfg.rooms[k].weight, 0);
    const rooms: Room[] = [];
    let at = 0;
    for (let i = 0; i < order.length; i++) {
      const s = cfg.rooms[order[i]];
      const after = order.slice(i + 1);
      const left = len - at;
      const restLeast = after.reduce((n, k) => n + cfg.rooms[k].minSide, 0);
      const restMost = after.reduce((n, k) => n + longest(k), 0);
      let n = i === order.length - 1 ? left : Math.round((len * s.weight) / weight);
      n = Math.max(s.minSide, left - restMost, Math.min(n, longest(order[i]), left - restLeast));
      if (n < s.minSide || n > longest(order[i]) || n > left - restLeast) break;
      rooms.push(alongX ? { x: r.x + at, z: r.z, w: n, d: r.d, kind: order[i] } : { x: r.x, z: r.z + at, w: r.w, d: n, kind: order[i] });
      at += n;
    }
    if (rooms.length === order.length && at === len) return rooms;
  }
  return null;
}

// ---- windows, furniture, garden ---------------------------------------------------------

/**
 * Windows along each room's outside walls: one every so many tiles (`windows.spacing`), a
 * bathroom one. A room they leave short of daylight (`windows.daylight`: the share of its floor
 * its windows light, `windowTiles` tiles each as the simulation scores rooms) gets more on its
 * outside walls, away from the others first, at a corner of the house or beside a door if need
 * be. Never in a door's place, nor where the stairs run along the wall.
 */
function windows(cfg: HouseGenConfig, rng: () => number, f: Floor, house: Rect, windowTiles: number): void {
  const outside = (e: string) => {
    const [axis, xs, zs] = e.split(':');
    const [x, z] = [Number(xs), Number(zs)];
    return axis === 'h' ? (z === house.z || z === house.z + house.d) && x >= house.x && x < house.x + house.w : (x === house.x || x === house.x + house.w) && z >= house.z && z < house.z + house.d;
  };
  const along = (e: string) => {
    const [axis, xs, zs] = e.split(':');
    const [x, z] = [Number(xs), Number(zs)];
    return axis === 'h' ? [`h:${x - 1}:${z}`, `h:${x + 1}:${z}`] : [`v:${x}:${z - 1}`, `v:${x}:${z + 1}`];
  };
  // A plain wall not along the stairs (or the stairwell), nor (`loose` 0) next to a door or at a
  // corner of the house; `loose` 1 allows a corner, 2 a door beside it too.
  const free = (e: string, loose: number) =>
    f.edges.get(e) === 'wall' && !sides(e).some(([x, z]) => f.taken.has(key(x, z))) && !along(e).some((n) => (loose < 2 && f.edges.get(n) === 'door') || (loose < 1 && !f.edges.has(n)));
  for (const r of f.rooms) {
    const spacing = between(rng, cfg.windows.spacing);
    const edges = [...f.edges.keys()].filter((e) => f.edges.get(e) === 'wall' && outside(e) && sides(e).some(([x, z]) => inside(r, x, z)));
    let placed = 0;
    edges.forEach((e, i) => {
      if (r.kind === 'bathroom' && placed >= 1) return;
      if ((i + Math.floor(spacing / 2)) % spacing !== 0) return;
      if (!free(e, 0)) return;
      f.edges.set(e, 'window');
      placed++;
    });
    // Daylight enough.
    const want = Math.ceil(((cfg.windows.daylight ?? 0) * r.w * r.d) / Math.max(windowTiles, 0.1));
    for (const loose of [0, 1, 2]) {
      while (placed < want) {
        const spare = edges.filter((e) => free(e, loose));
        const e = spare.find((e) => !along(e).some((n) => f.edges.get(n) === 'window')) ?? spare[0];
        if (!e) break;
        f.edges.set(e, 'window');
        placed++;
      }
    }
  }
}

/**
 * What the rooms of a house become for the people living in it (`Residents`): per room, the
 * recipes to try in turn before its own (`plans`; the house's planned rooms keep the kinds they
 * were laid out as). `keep`: whether a thing belongs in the house (only a home with children has toys).
 */
interface Fit {
  plans: Map<Room, Plan[]>;
  keep: (o: ObjectInfo) => boolean;
}

/** A room furnished from `spec` as a room of `kind`. */
interface Plan {
  kind: Kind;
  spec: RoomSpec;
}

/**
 * Who sleeps where (config `residents`): the biggest bedrooms first, a couple's with a bed for
 * two, an adult's with any bed, a child's with a child's bed and toys; short of bedrooms,
 * children share one (two beds, else a bunk bed), then adults (two single beds, else a bunk bed); a bedroom
 * nobody sleeps in is a guest room or a room of another kind (a study), as is a playroom in a
 * home without children. An empty house (`residents` undefined) keeps its bedrooms as they are.
 */
function fitFor(cfg: HouseGenConfig, info: ObjectInfos, rng: () => number, rooms: Room[], residents: Residents | undefined): Fit {
  const spec = cfg.residents;
  const kids = !!residents && residents.children + residents.babies > 0;
  const only = spec?.kidsOnly;
  const keep = (o: ObjectInfo) => kids || !only || !(only.categories?.includes(o.category ?? '') || only.groups?.includes(o.group ?? ''));
  const fit: Fit = { plans: new Map(), keep };
  if (!spec) return fit;
  const bedroom = cfg.rooms.bedroom;
  const bedSlot = bedroom?.furniture.findIndex((s) => s.one.some((d) => info(d)?.group === 'beds')) ?? -1;
  // A bedroom with `n` of these beds (and `more` things besides).
  const withBeds = (beds: string[] | undefined, n = 1, more: Slot[] = []): Plan | null =>
    beds?.length && bedSlot >= 0 ? { kind: 'bedroom', spec: { ...bedroom, furniture: [...bedroom.furniture.map((s, i) => (i === bedSlot ? { one: beds, count: [n, n] as [number, number] } : s)), ...more] } } : null;
  const spare = (guest: boolean): Plan | null => {
    const k = weighted(rng, Object.fromEntries(Object.entries(spec.spareRoom).filter(([k]) => (k === 'guest' ? guest : !!cfg.rooms[k]))));
    return !k ? null : k === 'guest' ? withBeds(spec.beds.single) : { kind: k, spec: cfg.rooms[k] };
  };
  const set = (r: Room, plans: (Plan | null)[]) => fit.plans.set(r, plans.filter((p): p is Plan => !!p));
  if (!kids) for (const r of rooms.filter((r) => r.kind === 'playroom')) set(r, [spare(false)]);
  if (!residents) return fit;
  type Sleeper = 'couple' | 'adults' | 'children' | 'adult' | 'child';
  const sleepers: Sleeper[] = [...Array<Sleeper>(residents.couples).fill('couple'), ...Array<Sleeper>(residents.adults).fill('adult'), ...Array<Sleeper>(residents.children).fill('child')];
  const bedrooms = rooms.filter((r) => r.kind === 'bedroom').sort((a, b) => b.w * b.d - a.w * a.d);
  // Two of `one` into a room for two.
  const pair = (one: Sleeper, two: Sleeper) => {
    const i = sleepers.indexOf(one);
    const j = sleepers.indexOf(one, i + 1);
    if (j < 0) return false;
    sleepers.splice(j, 1);
    sleepers[i] = two;
    return true;
  };
  while (sleepers.length > bedrooms.length && (pair('child', 'children') || pair('adult', 'adults')));
  const order: Sleeper[] = ['couple', 'adults', 'children', 'adult', 'child'];
  sleepers.sort((a, b) => order.indexOf(a) - order.indexOf(b));
  const toys = spec.childRoom ?? [];
  bedrooms.forEach((r, i) => {
    const who = sleepers[i];
    if (who === 'couple') set(r, [withBeds(spec.beds.couple)]);
    else if (who === 'adults') set(r, [withBeds(spec.beds.single, 2), withBeds(spec.beds.bunk)]);
    else if (who === 'children') set(r, [withBeds(spec.beds.child, 2, toys), withBeds(spec.beds.bunk, 1, toys), withBeds(spec.beds.single, 2)]);
    else if (who === 'child') set(r, [withBeds(spec.beds.child, 1, toys)]);
    else if (!who) set(r, [spare(true)]);
  });
  return fit;
}

/** What `furnish` may take back: everything placed on the floor and the tiles it took. */
function snapshot(f: Floor): () => void {
  const [objects, fronts, taken, keep, layers] = [[...f.objects], [...f.fronts], new Set(f.taken), new Set(f.keep), new Set(f.layers ?? [])];
  return () => {
    f.objects.splice(0, f.objects.length, ...objects);
    f.fronts.splice(0, f.fronts.length, ...fronts);
    f.taken = new Set(taken);
    f.keep = new Set(keep);
    f.layers = new Set(layers);
  };
}

/**
 * Furnishes each room (`furnishRoom`), each with random numbers of its own. Whether everything
 * that must be in the rooms fits is up to their own recipes, so a house works out the same for
 * anyone; then it's furnished again for who lives there (`fit`: each room from the first of its
 * plans that fits, else its own recipe without what doesn't belong in the house). If a room
 * won't do either way (rooms are not quite apart: a chair is reached from beside it, through an
 * open wall), the floor as its own recipes had it, without what doesn't belong (nor what stood
 * on that). False if a piece that must come doesn't fit.
 */
function furnish(cfg: HouseGenConfig, info: ObjectInfos, rng: () => number, f: Floor, has: Set<Kind>, lampTiles: number, fit: Fit): boolean {
  const seed = (rng() * 2 ** 32) >>> 0;
  const empty = snapshot(f);
  for (const [i, r] of f.rooms.entries()) {
    if (r.kind === 'hall') lights(cfg, info, f, r, lampTiles);
    else if (!furnishRoom(cfg, info, random(seed + i), f, r, cfg.rooms[r.kind], has, lampTiles)) return false;
  }
  if (f.rooms.every((r) => !fit.plans.get(r)?.length) && f.objects.every((o) => fit.keep(info(o.def)!))) return true;
  const own = snapshot(f);
  empty();
  const kept: ObjectInfos = (d) => {
    const o = info(d);
    return o && fit.keep(o) ? o : undefined;
  };
  for (const [i, r] of f.rooms.entries()) {
    if (r.kind === 'hall') {
      lights(cfg, info, f, r, lampTiles);
      continue;
    }
    const back = snapshot(f);
    let done = false;
    for (const plan of [...(fit.plans.get(r) ?? []), { kind: r.kind, spec: cfg.rooms[r.kind] }]) {
      back();
      if (furnishRoom(cfg, kept, random(seed + i), f, { ...r, kind: plan.kind }, plan.spec, has, lampTiles)) {
        done = true;
        break;
      }
    }
    if (!done) {
      own();
      strip(f, info, fit);
      return true;
    }
  }
  return true;
}

/** Takes what doesn't belong in the house (`fit.keep`) off floor `f`, and the small things on its top. */
function strip(f: Floor, info: ObjectInfos, fit: Fit): void {
  const gone = f.objects.filter((o) => !fit.keep(info(o.def)!));
  const under = new Set<string>();
  for (const o of gone) {
    const [fw, fd] = info(o.def)!.footprint;
    const [w, d] = o.rot % 2 === 0 ? [fw, fd] : [fd, fw];
    for (const t of tilesOf(o.x, o.z, w, d)) under.add(t);
  }
  for (let i = f.objects.length - 1; i >= 0; i--) {
    const o = f.objects[i];
    if (gone.includes(o) || (info(o.def)?.layer === 'surface' && under.has(key(o.x, o.z)))) {
      f.objects.splice(i, 1);
      f.fronts.splice(i, 1);
    }
  }
}

/**
 * Furnishes room `r` from `spec`: its counter run, the furniture that must come, its tables and
 * chairs (some only in houses with or without a room of some kind: `has`), the rest of its
 * furniture, small things on the tops, then things on the walls, ceiling and floor, and lights
 * enough. False if a piece that must come doesn't fit.
 */
function furnishRoom(cfg: HouseGenConfig, info: ObjectInfos, rng: () => number, f: Floor, r: Room, spec: RoomSpec, has: Set<Kind>, lampTiles: number): boolean {
  const shapes: Shapes = { object: info, radius: (d) => d?.radius ?? 0.12 };
  const sideOf = new Map<string, number>();
  if (spec.run) {
    const side = counterRun(f, r, info, rng, spec.run);
    if (side === false) return false;
    if (side !== undefined) sideOf.set('run', side);
  }
  const required = (slot: Slot) => slot.chance === undefined || slot.chance >= 1;
  const furniture = (slot: Slot) => {
    const count = slot.count ? between(rng, slot.count) : 1;
    for (let n = 0; n < count; n++) {
      if (!required(slot) && rng() >= (slot.chance ?? 1)) continue;
      const options = shuffle(rng, slot.one.filter((d) => info(d)));
      let done = false;
      for (const def of options) {
        const across = slot.opposite ? sideOf.get(slot.opposite) : undefined;
        const prefer = across === undefined ? (slot.beside ? sideOf.get(slot.beside) : undefined) : (across + 2) % 4;
        // Things nobody uses (a floor lamp, a vase on a stand) keep out of the way in doorways;
        // one that must come goes wherever it fits if it has to.
        const aside = !info(def)!.used;
        const side = place(f, r, def, info(def)!, rng, prefer, aside, info) ?? (aside && required(slot) ? place(f, r, def, info(def)!, rng, prefer, false, info) : null);
        if (side !== null) {
          sideOf.set(def, side);
          for (const o of slot.one) sideOf.set(o, side);
          done = true;
          break;
        }
      }
      if (!done && required(slot)) return false;
    }
    return true;
  };
  // What must come first, then the tables (they want the middle of the room), then the rest.
  for (const slot of spec.furniture) if (required(slot) && !furniture(slot)) return false;
  for (const set of spec.dining ?? []) {
    if ((set.ifRoom && !has.has(set.ifRoom)) || (set.unlessRoom && has.has(set.unlessRoom))) continue;
    if (rng() < (set.chance ?? 1)) tableSet(f, r, info, rng, set);
  }
  for (const slot of spec.furniture) if (!required(slot)) furniture(slot);
  for (const slot of spec.surfaces ?? []) onTops(f, r, shapes, info, rng, slot);
  for (const layer of ['wall', 'ceiling', 'rug'] as const) {
    for (const slot of spec[layer] ?? []) {
      const count = slot.count ? between(rng, slot.count) : 1;
      for (let n = 0; n < count; n++) {
        if (rng() >= (slot.chance ?? 1)) continue;
        const def = pick(rng, slot.one);
        // (Curtains that don't fit the room's windows: the slot's others, as wide as one is.)
        const others = info(def)?.overWindow ? slot.one.filter((d) => d !== def && info(d)?.overWindow) : [];
        for (const d of [def, ...others]) {
          const fp = info(d)?.footprint;
          if (fp && mount(cfg, info, f, r, d, fp, layer, rng, slot.over)) break;
        }
      }
    }
  }
  lights(cfg, info, f, r, lampTiles);
  return true;
}

/**
 * Lights from the ceiling until room `r` is lit right through at night, as the simulation scores
 * rooms: each lamp lights its `light.tiles` (else `lampTiles`), and lamps already in it (on the
 * floor, on tops, on walls) count. As few as do it, all alike and spread evenly, each in the
 * middle of its share of the room: one big light sized to the room rather than a ceiling full of
 * small ones. They're the kind that takes fewest of the room's own ceiling lights (else of
 * `lights`; the one hanging there already if it's as good, else whichever the room's place
 * picks), and what hung there already comes down to hang with them. Draws nothing from the
 * house's random numbers.
 */
function lights(cfg: HouseGenConfig, info: ObjectInfos, f: Floor, r: Room, lampTiles: number): void {
  const tiles = r.w * r.d;
  const cover = (d: string) => (info(d)?.lamp ? Math.max(info(d)!.lightTiles ?? lampTiles, 0.1) : 0);
  const lit = () => f.objects.reduce((n, o) => n + (inside(r, o.x, o.z) ? cover(o.def) : 0), 0);
  if (lit() >= tiles) return;
  const hanging = (d: string) => cover(d) > 0 && info(d)?.layer === 'ceiling';
  const own = (cfg.rooms[r.kind]?.ceiling ?? []).flatMap((s) => s.one).filter(hanging);
  const options = own.length ? own : (cfg.lights ?? []).filter(hanging);
  // What hangs there already comes down: it hangs again with the rest, all spread evenly.
  const hung = f.objects.filter((o) => inside(r, o.x, o.z) && hanging(o.def));
  for (const o of hung) unhang(f, info, o);
  const short = tiles - lit();
  const kinds = [...new Set([...hung.map((o) => o.def), ...options])];
  if (!kinds.length) return;
  const need = (d: string) => Math.ceil(short / cover(d) - 1e-6);
  const want = Math.min(...kinds.map(need));
  const best = kinds.filter((d) => need(d) === want);
  const def = best.find((d) => hung.some((o) => o.def === d)) ?? best[(r.x + r.z) % best.length];
  // Hanging from the ceiling draws no random numbers.
  const rng = () => 0.5;
  // The room in `want` shares, as near square as can be.
  let grid: [number, number] = [want, 1];
  let most = Infinity;
  for (let cols = 1; cols <= want; cols++) {
    const rows = Math.ceil(want / cols);
    const score = cols * rows * 100 + Math.abs(r.w / cols - r.d / rows);
    if (score < most) [most, grid] = [score, [cols, rows]];
  }
  const [cols, rows] = grid;
  for (let j = 0; j < rows; j++) {
    // A last row with fewer shares spreads them over the room's width.
    const across = Math.min(cols, want - j * cols);
    for (let i = 0; i < across; i++) {
      if (lit() >= tiles) return;
      mount(cfg, info, f, r, def, info(def)!.footprint, 'ceiling', rng, undefined, [r.x + ((i + 0.5) * r.w) / across - 0.5, r.z + ((j + 0.5) * r.d) / rows - 0.5]);
    }
  }
  // Shares with no free ceiling: wherever there's room (and where a lamp at head height would be
  // in the way everywhere, another kind).
  const others = kinds.filter((d) => d !== def && !info(d)!.headHeight).sort((a, b) => need(a) - need(b));
  while (lit() < tiles) if (![def, ...others].some((d) => mount(cfg, info, f, r, d, info(d)!.footprint, 'ceiling', rng))) return;
}

/** The floor tiles a lamp at head height took from walking (`mount`), freed when it comes down. */
const hungLow = new WeakMap<Placement, string[]>();

/** Takes a ceiling light down again (and frees its ceiling tiles, and the floor a lamp at head height took). */
function unhang(f: Floor, info: ObjectInfos, o: Placement): void {
  const at = f.objects.indexOf(o);
  f.objects.splice(at, 1);
  f.fronts.splice(at, 1);
  for (const t of hungLow.get(o) ?? []) f.taken.delete(t);
  const [fw, fd] = info(o.def)?.footprint ?? [1, 1];
  const [w, d] = o.rot % 2 === 0 ? [fw, fd] : [fd, fw];
  for (let dz = 0; dz < d; dz++) for (let dx = 0; dx < w; dx++) f.layers?.delete(`ceiling:${key(o.x + dx, o.z + dz)}`);
}

/** Tiles of a footprint (min corner, rotated size). */
function tilesOf(x: number, z: number, w: number, d: number): string[] {
  const tiles: string[] = [];
  for (let dz = 0; dz < d; dz++) for (let dx = 0; dx < w; dx++) tiles.push(key(x + dx, z + dz));
  return tiles;
}

/** Takes back everything placed on `f` since it had `n` objects (their tiles too). */
function undo(f: Floor, n: number, tiles: string[]): void {
  f.objects.length = n;
  f.fronts.length = n;
  for (const t of tiles) f.taken.delete(t);
}

/**
 * A kitchen's counter run: the pieces of `run.with` (one of each slot that comes; a sink, a
 * stove, maybe a dishwasher) in a random order among plain segments, side by side along one wall,
 * each used from the row in front (kept clear). Along a wall with `run.clear` rows of room in
 * front of it if there's one (an island and its stools), shorter if the walls are short; if no
 * wall has room, the pieces go against the walls one by one. The side it went on (0 the side
 * towards the street, then clockwise; undefined one by one), false if a piece that must come
 * doesn't fit.
 */
function counterRun(f: Floor, r: Room, info: ObjectInfos, rng: () => number, run: CounterRun): number | undefined | false {
  const pieces: { def: string; w: number; required: boolean }[] = [];
  for (const slot of run.with) {
    if (slot.chance !== undefined && rng() >= slot.chance) continue;
    const def = shuffle(rng, slot.one.filter((d) => info(d)?.footprint[1] === 1))[0];
    if (def) pieces.push({ def, w: info(def)!.footprint[0], required: slot.chance === undefined || slot.chance >= 1 });
  }
  const segments = run.segments.filter((d) => info(d)?.footprint.every((n) => n === 1));
  const segment = segments.length ? pick(rng, segments) : null;
  const least = pieces.reduce((n, p) => n + p.w, 0);
  const longest = Math.max(least, between(rng, run.length));
  // The walls with room enough in front first, then any.
  const roomy = (side: number) => (side % 2 === 0 ? r.d : r.w) >= (run.clear ?? 0);
  for (const first of [true, false]) for (let len = longest; len >= Math.max(least, 1); len--) {
    if (len > least && !segment) continue;
    const order = shuffle(rng, [...pieces.map((p) => p.def), ...Array<string>(len - least).fill(segment ?? '')]);
    for (const side of shuffle(rng, [0, 1, 2, 3])) {
      if (roomy(side) !== first) continue;
      // Back to the wall on `side` (0 the side towards the street, then clockwise), as in `place`.
      const rot = [0, 3, 2, 1][side];
      const along = side % 2 === 0;
      if ((along ? r.d : r.w) < 2) continue;
      const starts = shuffle(rng, Array.from({ length: (along ? r.w : r.d) - len + 1 }, (_, i) => i));
      for (const s of starts) {
        const n = f.objects.length;
        const tiles: string[] = [];
        let at = s;
        let ok = true;
        for (const def of order) {
          const [fw] = info(def)!.footprint;
          const [w, d] = along ? [fw, 1] : [1, fw];
          const x = along ? r.x + at : side === 3 ? r.x : r.x + r.w - 1;
          const z = along ? (side === 0 ? r.z : r.z + r.d - 1) : r.z + at;
          at += fw;
          const mine = tilesOf(x, z, w, d);
          const front = frontTile(x, z, w, d, rot);
          if (mine.some((t) => f.taken.has(t) || f.keep.has(t)) || !inside(r, front[0], front[1]) || f.taken.has(key(front[0], front[1]))) {
            ok = false;
            break;
          }
          for (const t of mine) f.taken.add(t);
          tiles.push(...mine);
          f.objects.push({ def, x, z, rot });
          f.fronts.push([front]);
        }
        if (ok && reachable(f, r)) {
          for (const fr of f.fronts.slice(n)) for (const [x, z] of fr ?? []) f.keep.add(key(x, z));
          return side;
        }
        undo(f, n, tiles);
      }
    }
  }
  // No wall long enough: one by one.
  if (!pieces.every((p) => place(f, r, p.def, info(p.def)!, rng) !== null || !p.required)) return false;
  return undefined;
}

/** Room a chair takes along a table's edge (m). */
const SEAT_WIDTH = 0.6;

/**
 * The tiles round a table (at `x`, `z`, turned `rot`) where chairs go, each with the way a chair
 * there faces the table, best first: down its long sides from the middle out, across from each
 * other, then its ends (a square table: one on each side first). As many along an edge as its
 * top has room for (a small round table: two, across from each other). `sides`: only these
 * (0: the -z side, 1: +x, 2: +z, 3: -x).
 */
function seatTiles(table: ObjectInfo, x: number, z: number, rot: number, sides = [0, 1, 2, 3]): { x: number; z: number; rot: number }[] {
  const [w, d] = rot % 2 === 0 ? table.footprint : [table.footprint[1], table.footprint[0]];
  const top = table.surface?.size ?? table.footprint;
  const [tw, td] = rot % 2 === 0 ? top : [top[1], top[0]];
  let [nw, nd] = [Math.min(w, Math.floor(tw / SEAT_WIDTH)), Math.min(d, Math.floor(td / SEAT_WIDTH))];
  if (nw === 0 && nd === 0) nw = 1;
  const middleOut = (n: number, most: number) => Array.from({ length: n }, (_, i) => i).sort((a, b) => Math.abs(a - (n - 1) / 2) - Math.abs(b - (n - 1) / 2) || a - b).slice(0, most);
  // Per side: the chair tiles, facing the table (a chair at rot 0 faces +z).
  const along: { x: number; z: number; rot: number }[][] = [
    middleOut(w, nw).map((i) => ({ x: x + i, z: z - 1, rot: 0 })),
    middleOut(d, nd).map((i) => ({ x: x + w, z: z + i, rot: 3 })),
    middleOut(w, nw).map((i) => ({ x: x + i, z: z + d, rot: 2 })),
    middleOut(d, nd).map((i) => ({ x: x - 1, z: z + i, rot: 1 })),
  ].map((list, i) => (sides.includes(i) ? list : []));
  const groups = w === d ? [[0, 2, 1, 3]] : w > d ? [[0, 2], [1, 3]] : [[1, 3], [0, 2]];
  const out: { x: number; z: number; rot: number }[] = [];
  for (const group of groups) {
    const most = Math.max(...group.map((s) => along[s].length));
    for (let k = 0; k < most; k++) for (const s of group) if (along[s][k]) out.push(along[s][k]);
  }
  return out;
}

/**
 * A table and its chairs in room `r`: the table free in the room (off the walls where it can
 * be, near the middle; a desk with `wall` against one), chairs of one kind on its free sides,
 * each facing it and with a way to it. False if it doesn't fit with the fewest seats.
 */
function tableSet(f: Floor, r: Room, info: ObjectInfos, rng: () => number, set: TableSet): boolean {
  const chairs = set.chairs.filter((d) => info(d));
  const chair = chairs.length ? pick(rng, chairs) : null;
  const want = between(rng, set.seats);
  const [least] = set.seats;
  if (!chair && least > 0) return false;
  for (const def of shuffle(rng, set.tables.filter((d) => info(d)))) {
    const table = info(def)!;
    const [fw, fd] = table.footprint;
    const spots: { x: number; z: number; rot: number; score: number }[] = [];
    if (set.wall) {
      // Back to a wall, front into the room (as `place` puts furniture).
      for (const side of [0, 1, 2, 3]) {
        const rot = [0, 3, 2, 1][side];
        const [w, d] = rot % 2 === 0 ? [fw, fd] : [fd, fw];
        const line = side % 2 === 0 ? r.w - w : r.d - d;
        for (let i = 0; i <= line; i++) {
          const x = side === 0 || side === 2 ? r.x + i : side === 3 ? r.x : r.x + r.w - w;
          const z = side === 1 || side === 3 ? r.z + i : side === 0 ? r.z : r.z + r.d - d;
          spots.push({ x, z, rot, score: rng() });
        }
      }
    } else {
      const [cx, cz] = [r.x + r.w / 2, r.z + r.d / 2];
      for (const turned of [0, 1]) {
        const [w, d] = turned ? [fd, fw] : [fw, fd];
        for (let z = r.z; z + d <= r.z + r.d; z++) {
          for (let x = r.x; x + w <= r.x + r.w; x++) {
            // Facing into the room (a table nobody uses still needs its front open), either way if
            // both are (an island's cook stands at the front, its stools behind).
            const rots = [turned, turned + 2].filter((rot) => inside(r, ...frontTile(x, z, w, d, rot)));
            // Room for chairs and a way round them on every side; else as near the middle as can be.
            const gap = Math.min(x - r.x, z - r.z, r.x + r.w - x - w, r.z + r.d - z - d);
            for (const rot of rots) spots.push({ x, z, rot, score: Math.min(gap, 2) * 4 - Math.hypot(x + w / 2 - cx, z + d / 2 - cz) + rng() });
          }
        }
      }
    }
    spots.sort((a, b) => b.score - a.score);
    for (const { x, z, rot } of spots.slice(0, 48)) {
      const [w, d] = rot % 2 === 0 ? [fw, fd] : [fd, fw];
      const mine = tilesOf(x, z, w, d);
      const front = frontTile(x, z, w, d, rot);
      if (mine.some((t) => f.taken.has(t) || f.keep.has(t)) || !inside(r, front[0], front[1])) continue;
      if (table.used && f.taken.has(key(front[0], front[1]))) continue;
      // Chairs along a desk's front only; stools along one side of an island (its back, if someone uses it).
      const [fs, bs] = [[2, 1, 0, 3][rot % 4], [0, 3, 2, 1][rot % 4]];
      const one = shuffle(rng, w === d ? [[0, 2], [1, 3]] : w > d ? [[0], [2]] : [[1], [3]]);
      const choices = set.wall ? [[fs]] : set.oneSide ? (table.used ? [[bs]] : one) : [undefined];
      // Stools may stand on the kitchen's side of the way into an open living room (it stays open on the other).
      const free = (s: { x: number; z: number }) => inside(r, s.x, s.z) && !f.taken.has(key(s.x, s.z)) && (!f.keep.has(key(s.x, s.z)) || (!!set.oneSide && !!f.open?.has(key(s.x, s.z))));
      const options = choices.map((sides) => seatTiles(table, x, z, rot, sides).filter(free)).filter((seats) => seats.length >= least);
      if (!options.length) continue;
      const n = f.objects.length;
      for (const t of mine) f.taken.add(t);
      f.objects.push({ def, x, z, rot });
      f.fronts.push(table.used ? [front] : null);
      if (!reachable(f, r)) {
        undo(f, n, mine);
        continue;
      }
      for (const seats of options) {
        const tiles: string[] = [];
        for (const s of seats) {
          if (tiles.length >= want || !chair) break;
          const t = key(s.x, s.z);
          f.taken.add(t);
          f.objects.push({ def: chair, x: s.x, z: s.z, rot: s.rot });
          f.fronts.push(approachCandidates(info(chair), s.x, s.z, s.rot).map((c) => c.tile));
          if (reachable(f, r)) tiles.push(t);
          else undo(f, f.objects.length - 1, [t]);
        }
        if (tiles.length >= least) {
          if (table.used) f.keep.add(key(front[0], front[1]));
          return true;
        }
        undo(f, n + 1, tiles);
      }
      undo(f, n, mine);
    }
  }
  return false;
}

/**
 * Small things on tops in room `r`: each on the emptiest top of a kind it may go on (a table's
 * in its middle, elsewhere anywhere free; never a hob's or a sink's), facing about the way its
 * host does, listed right after its host (the simulation places it on a top that's already there).
 */
function onTops(f: Floor, r: Room, shapes: Shapes, info: ObjectInfos, rng: () => number, slot: TopSlot): void {
  const count = slot.count ? between(rng, slot.count) : 1;
  for (let n = 0; n < count; n++) {
    if (rng() >= (slot.chance ?? 1)) continue;
    // Something not on a top in the room yet, if there's anything left (one toaster is plenty).
    const here = new Set(f.objects.filter((o) => info(o.def)?.layer === 'surface' && inside(r, o.x, o.z)).map((o) => o.def));
    const fresh = slot.one.filter((d) => !here.has(d));
    const def = pick(rng, fresh.length ? fresh : slot.one);
    const thing = info(def);
    if (thing?.layer !== 'surface') continue;
    const tops = f.objects
      .map((o) => topOf(shapes, o))
      .filter((t): t is Top => !!t && inside(r, t.host.x, t.host.z) && slot.on.includes(t.kind) && !info(t.host.def)?.keepClear && (!slot.of || slot.of.includes(t.host.def)) && 2 * Math.min(t.hw, t.hd) >= (slot.minSide ?? 0));
    const riders = (top: Top) => f.objects.filter((o) => info(o.def)?.layer === 'surface' && covers(top, ...middleOf(shapes, o))).length;
    const order = tops.map((t) => ({ t, n: riders(t) + rng() })).sort((a, b) => a.n - b.n);
    for (const { t } of order) {
      const spots = freeSpots(shapes, f.objects, t, thing.radius);
      if (!spots.length) continue;
      const spot = t.kind === 'table' ? spots[0] : spots[Math.floor(rng() * spots.length)];
      // About the way its host faces, a little turned.
      const angle = (((t.host.rot * 90 + Math.round(rng() * 60) - 30) % 360) + 360) % 360;
      const rot = Math.floor(angle / 90);
      const at = f.objects.indexOf(t.host as Placement) + 1;
      f.objects.splice(at, 0, { def, x: spot.x, z: spot.z, rot, ...(angle % 90 ? { turn: angle % 90 } : {}), ...(spot.offset[0] || spot.offset[1] ? { offset: spot.offset } : {}) });
      f.fronts.splice(at, 0, null);
      break;
    }
  }
}

/**
 * One thing on the wall, ceiling or rug layer of room `r` (it never blocks anything): on a wall,
 * its back on wall edges (or window ones, for what goes over a window), not over furniture as tall
 * as the wall; from the ceiling, over a free spot (not over such furniture either); a rug, in the middle of
 * the room. Never on the stairs, and its front (where the simulation checks for walls) inside the room.
 * `near`: from the ceiling, as near this tile as there's room (else near the middle).
 */
function mount(cfg: HouseGenConfig, info: ObjectInfos, f: Floor, r: Room, def: string, [fw, fd]: [number, number], layer: 'wall' | 'ceiling' | 'rug', rng: () => number, over?: string[], near?: [number, number]): boolean {
  const layers = (f.layers ??= new Set());
  const used = !!info(def)?.used;
  const stairs = new Set<string>();
  for (const o of f.objects) {
    if (o.def !== cfg.stairs) continue;
    const [sw, sd] = info(o.def)?.footprint ?? [1, 4];
    const [w, d] = o.rot % 2 === 0 ? [sw, sd] : [sd, sw];
    for (let dz = 0; dz < d; dz++) for (let dx = 0; dx < w; dx++) stairs.add(key(o.x + dx, o.z + dz));
  }
  // Nor over the stairwell upstairs (all a hall has taken), nor over the one down to the basement.
  if (r.kind === 'hall') for (const t of f.taken) stairs.add(t);
  for (const t of f.well ?? []) stairs.add(t);
  // Nor on the wall or from the ceiling over furniture as tall as the wall (content `fillsWall`).
  if (layer !== 'rug') {
    for (const o of f.objects) {
      const thing = info(o.def);
      if (!thing?.fillsWall) continue;
      const [w, d] = o.rot % 2 === 0 ? thing.footprint : [thing.footprint[1], thing.footprint[0]];
      for (let dz = 0; dz < d; dz++) for (let dx = 0; dx < w; dx++) stairs.add(key(o.x + dx, o.z + dz));
    }
  }
  // A lamp at head height takes the floor under it from walking: never a doorway's or a front's.
  const low = layer === 'ceiling' && !!info(def)?.headHeight;
  const fits = (x: number, z: number, w: number, d: number, rot: number) => {
    for (let dz = 0; dz < d; dz++) {
      for (let dx = 0; dx < w; dx++) {
        const t = key(x + dx, z + dz);
        if (!inside(r, x + dx, z + dz) || stairs.has(t) || layers.has(`${layer}:${t}`) || (low && f.keep.has(t))) return false;
        // (Nor into an arc lamp's shade reaching over from beside.)
        if ((low || (layer === 'ceiling' && info(def)?.hangsLow)) && f.shaded?.has(t)) return false;
      }
    }
    const [frx, frz] = frontTile(x, z, w, d, rot);
    // One with a use (the mirror cabinet's "Brush your teeth") has its front tile free to stand on.
    return inside(r, frx, frz) && !(used && f.taken.has(key(frx, frz)));
  };
  const take = (x: number, z: number, w: number, d: number, rot: number) => {
    for (let dz = 0; dz < d; dz++) for (let dx = 0; dx < w; dx++) layers.add(`${layer}:${key(x + dx, z + dz)}`);
    const placed = { def, x, z, rot };
    f.objects.push(placed);
    f.fronts.push(used ? [frontTile(x, z, w, d, rot)] : null);
    const under = low ? tilesOf(x, z, w, d).filter((t) => !f.taken.has(t)) : [];
    for (const t of under) f.taken.add(t);
    if (under.length) hungLow.set(placed, under);
    // ...and a way to it (and round what hangs at head height).
    if ((used || under.length) && !reachable(f, r)) {
      f.objects.pop();
      f.fronts.pop();
      for (const t of under) f.taken.delete(t);
      for (let dz = 0; dz < d; dz++) for (let dx = 0; dx < w; dx++) layers.delete(`${layer}:${key(x + dx, z + dz)}`);
      return false;
    }
    return true;
  };
  if (layer === 'wall') {
    // Only what belongs over a window (content `overWindow`: curtains) hangs on one.
    const window = !!info(def)?.overWindow;
    const backed = (e: string) => f.edges.get(e) === 'wall' || (window && f.edges.get(e) === 'window');
    // ...and that only over one (with no free window in the room, none), just as wide as the
    // window: a pair a tile wide over a one-edge window, the wide ones over a two-edge window
    // (two pairs on one window would hang rail over rail).
    const glass = (e: string) => f.edges.get(e) === 'window';
    const beyond = (backs: string[]) => {
      const [first, last] = [backs[0], backs[backs.length - 1]].map((e) => e.split(':'));
      return first[0] === 'h' ? [`h:${Number(first[1]) - 1}:${first[2]}`, `h:${Number(last[1]) + 1}:${last[2]}`] : [`v:${first[1]}:${Number(first[2]) - 1}`, `v:${last[1]}:${Number(last[2]) + 1}`];
    };
    const hangs = (backs: string[]) => backs.every(backed) && (!window || (backs.every(glass) && !beyond(backs).some(glass)));
    // Over a piece in the room: turned as it is, centred on it, its back on the same wall.
    for (const o of over ? f.objects.filter((o) => over.includes(o.def) && inside(r, o.x, o.z)) : []) {
      const [ow, od] = info(o.def)?.footprint ?? [1, 1];
      const [pw, pd] = o.rot % 2 === 0 ? [ow, od] : [od, ow];
      const [w, d] = o.rot % 2 === 0 ? [fw, fd] : [fd, fw];
      const x = o.x + Math.floor((pw - w) / 2);
      const z = o.z + Math.floor((pd - d) / 2);
      const backs =
        o.rot === 0 ? Array.from({ length: w }, (_, i) => `h:${x + i}:${o.z}`)
        : o.rot === 2 ? Array.from({ length: w }, (_, i) => `h:${x + i}:${o.z + pd}`)
        : o.rot === 3 ? Array.from({ length: d }, (_, i) => `v:${o.x + pw}:${z + i}`)
        : Array.from({ length: d }, (_, i) => `v:${o.x}:${z + i}`);
      const [wx, wz] = o.rot === 2 ? [x, o.z + pd - d] : o.rot === 3 ? [o.x + pw - w, z] : o.rot === 0 ? [x, o.z] : [o.x, z];
      if (hangs(backs) && fits(wx, wz, w, d, o.rot) && take(wx, wz, w, d, o.rot)) return true;
    }
    for (const side of shuffle(rng, [0, 1, 2, 3])) {
      const rot = [0, 3, 2, 1][side];
      const [w, d] = rot % 2 === 0 ? [fw, fd] : [fd, fw];
      const spots: [number, number, string[]][] = [];
      if (side === 0) for (let x = r.x; x + w <= r.x + r.w; x++) spots.push([x, r.z, Array.from({ length: w }, (_, i) => `h:${x + i}:${r.z}`)]);
      if (side === 2) for (let x = r.x; x + w <= r.x + r.w; x++) spots.push([x, r.z + r.d - d, Array.from({ length: w }, (_, i) => `h:${x + i}:${r.z + r.d}`)]);
      if (side === 3) for (let z = r.z; z + d <= r.z + r.d; z++) spots.push([r.x, z, Array.from({ length: d }, (_, i) => `v:${r.x}:${z + i}`)]);
      if (side === 1) for (let z = r.z; z + d <= r.z + r.d; z++) spots.push([r.x + r.w - w, z, Array.from({ length: d }, (_, i) => `v:${r.x + r.w}:${z + i}`)]);
      shuffle(rng, spots);
      for (const [x, z, backs] of spots) if (hangs(backs) && fits(x, z, w, d, rot) && take(x, z, w, d, rot)) return true;
    }
    return false;
  }
  if (layer === 'ceiling') {
    // Turned whichever way has its front in the room (along a corridor a tile wide, too).
    const spots: [number, number, number][] = [];
    for (const rot of [0, 1, 2, 3]) {
      const [w, d] = rot % 2 === 0 ? [fw, fd] : [fd, fw];
      for (let z = r.z; z + d <= r.z + r.d; z++) for (let x = r.x; x + w <= r.x + r.w; x++) spots.push([x, z, rot]);
    }
    // Near the middle of the room (or of the share a light is for).
    const [cx, cz] = near ?? [r.x + (r.w - fw) / 2, r.z + (r.d - fd) / 2];
    spots.sort((a, b) => Math.hypot(a[0] - cx, a[1] - cz) - Math.hypot(b[0] - cx, b[1] - cz));
    for (const [x, z, rot] of spots) {
      const [w, d] = rot % 2 === 0 ? [fw, fd] : [fd, fw];
      if (!fits(x, z, w, d, rot)) continue;
      if (take(x, z, w, d, rot)) return true;
      // (Only a lamp at head height that would block the way looks further.)
      if (!low) return false;
    }
    return false;
  }
  for (const rot of [0, 1]) {
    const [w, d] = rot % 2 === 0 ? [fw, fd] : [fd, fw];
    const x = r.x + Math.floor((r.w - w) / 2);
    const z = r.z + Math.floor((r.d - d) / 2);
    if (fits(x, z, w, d, rot)) return take(x, z, w, d, rot);
  }
  return false;
}

/**
 * One piece against a wall of room `r`, facing into it (`prefer`: a side first, 0 the side
 * towards the street, then clockwise). Returns the side, or null if there's no place for it.
 * `aside` (things nobody uses: a floor lamp): in a corner if it can be, and never next to the
 * tile inside a door (it would make the way in a squeeze). One that reaches over the tiles beside
 * it (content `overhang`: an arc lamp) hangs its shade over a seat there if it can (`info`: a sofa
 * or a chair), else over free floor, which it takes from walking; never over a doorway, the
 * stairwell, another shade or another room.
 */
function place(f: Floor, r: Room, def: string, { footprint: [fw, fd], againstWall, overhang }: ObjectInfo, rng: () => number, prefer?: number, aside = false, info?: ObjectInfos): number | null {
  // Tiles round the tiles inside the room's doors.
  const doorway = new Set<string>();
  if (aside) {
    for (const [e, v] of f.edges) {
      if (v !== 'door') continue;
      for (const [x, z] of sides(e)) {
        if (!inside(r, x, z)) continue;
        for (let dz = -1; dz <= 1; dz++) for (let dx = -1; dx <= 1; dx++) doorway.add(key(x + dx, z + dz));
      }
    }
  }
  const sideList = shuffle(rng, [0, 1, 2, 3]);
  if (prefer !== undefined) sideList.sort((a, b) => (a === prefer ? -1 : b === prefer ? 1 : 0));
  type Spot = { side: number; rot: number; w: number; d: number; x: number; z: number };
  // Side by side (each side's spots shuffled only once it's reached).
  function* walls(): Generator<Spot> {
    for (const side of sideList) {
      // Back to the wall on `side`, front into the room: rot 0 faces +z (back to the street-side wall).
      const rot = [0, 3, 2, 1][side];
      const [w, d] = rot % 2 === 0 ? [fw, fd] : [fd, fw];
      const along: [number, number][] = [];
      if (side === 0) for (let x = r.x; x + w <= r.x + r.w; x++) along.push([x, r.z]);
      if (side === 2) for (let x = r.x; x + w <= r.x + r.w; x++) along.push([x, r.z + r.d - d]);
      if (side === 3) for (let z = r.z; z + d <= r.z + r.d; z++) along.push([r.x, z]);
      if (side === 1) for (let z = r.z; z + d <= r.z + r.d; z++) along.push([r.x + r.w - w, z]);
      shuffle(rng, along);
      for (const [x, z] of along) yield { side, rot, w, d, x, z };
    }
  }
  // Out of the way: corners first, whichever side they're on.
  const corner = (s: Spot) => Number((s.x === r.x || s.x + s.w === r.x + r.w) && (s.z === r.z || s.z + s.d === r.z + r.d));
  // What is on a tile a shade would reach over: 'free' floor, 'under' (a seat it hangs over), or null (it can't).
  const beneath = (t: [number, number]): 'free' | 'under' | null => {
    const k = key(t[0], t[1]);
    if (!inside(r, t[0], t[1]) || f.keep.has(k) || f.well?.has(k) || f.shaded?.has(k) || (aside && doorway.has(k))) return null;
    if (!f.taken.has(k)) return 'free';
    const o = info && f.objects.find((p) => {
      if ((info(p.def)?.layer ?? 'floor') !== 'floor') return false;
      const [pw, pd] = info(p.def)?.footprint ?? [1, 1];
      const [w, d] = p.rot % 2 === 0 ? [pw, pd] : [pd, pw];
      return inside({ x: p.x, z: p.z, w, d }, t[0], t[1]);
    });
    const thing = o && info?.(o.def);
    return thing?.seated && !thing.fillsWall && !thing.stairs ? 'under' : null;
  };
  const reach = (s: Spot) => overhangTiles(overhang, s.x, s.z, s.rot);
  // An arc lamp's shade over a sofa or a chair first.
  const over = (s: Spot) => Number(overhang !== undefined && reach(s).every((t) => beneath(t) === 'under'));
  const spots = aside || overhang ? [...walls()].sort((a, b) => over(b) - over(a) || corner(b) - corner(a)) : walls();
  // What's built to stand against a wall (content `againstWall`) has plain wall all along its back.
  const walled = (side: number, x: number, z: number, w: number, d: number) =>
    (side === 0 || side === 2 ? Array.from({ length: w }, (_, i) => `h:${x + i}:${side === 0 ? z : z + d}`) : Array.from({ length: d }, (_, i) => `v:${side === 3 ? x : x + w}:${z + i}`)).every((e) => f.edges.get(e) === 'wall');
  for (const { side, rot, w, d, x, z } of spots) {
    const tiles = tilesOf(x, z, w, d);
    const front = frontTile(x, z, w, d, rot);
    if (!inside(r, front[0], front[1])) continue;
    if (againstWall && !walled(side, x, z, w, d)) continue;
    if (aside && tiles.some((t) => doorway.has(t))) continue;
    // A shade over the tiles beside: nobody needs its front, which may be the furniture under it.
    const shade = overhang ? reach({ side, rot, w, d, x, z }) : [];
    const kinds = shade.map(beneath);
    if (kinds.some((k) => k === null)) continue;
    if (tiles.some((t) => f.taken.has(t) || f.keep.has(t)) || (!overhang && f.taken.has(key(front[0], front[1])))) continue;
    const n = f.objects.length;
    // (The free floor under its shade is taken from walking, as under a lamp at head height.)
    const under = shade.filter((_, i) => kinds[i] === 'free').map(([tx, tz]) => key(tx, tz));
    for (const t of [...tiles, ...under]) f.taken.add(t);
    f.objects.push({ def, x, z, rot });
    f.fronts.push(overhang ? null : [front]);
    if (reachable(f, r)) {
      if (overhang) for (const [tx, tz] of shade) (f.shaded ??= new Set()).add(key(tx, tz));
      else f.keep.add(key(front[0], front[1]));
      return side;
    }
    undo(f, n, [...tiles, ...under]);
  }
  return null;
}

/**
 * Every free tile of the room the doors need is still connected, and so is a way to everything
 * in it: its front tile (which must be free), or for chairs one of the free tiles round them;
 * no free floor is shut off behind furniture.
 */
function reachable(f: Floor, r: Room): boolean {
  const free = (x: number, z: number) => inside(r, x, z) && !f.taken.has(key(x, z));
  const needed: string[] = [];
  for (let z = r.z; z < r.z + r.d; z++) for (let x = r.x; x < r.x + r.w; x++) if (f.keep.has(key(x, z)) && free(x, z)) needed.push(key(x, z));
  // Per object in the room, the free tiles it can be used from (one must be reached).
  const ways: string[][] = [];
  for (const tiles of f.fronts) {
    const mine = (tiles ?? []).filter(([x, z]) => inside(r, x, z));
    if (!mine.length) continue;
    const open = mine.filter(([x, z]) => free(x, z)).map(([x, z]) => key(x, z));
    if (!open.length) return false;
    ways.push(open);
  }
  const start = needed[0] ?? ways[0]?.[0];
  if (!start) return true;
  const seen = new Set<string>([start]);
  const queue = [start];
  while (queue.length) {
    const [x, z] = queue.pop()!.split(':').map(Number);
    for (const [nx, nz] of [[x + 1, z], [x - 1, z], [x, z + 1], [x, z - 1]]) {
      const k = key(nx, nz);
      if (!seen.has(k) && free(nx, nz)) {
        seen.add(k);
        queue.push(k);
      }
    }
  }
  if (!needed.every((k) => seen.has(k)) || !ways.every((w) => w.some((k) => seen.has(k)))) return false;
  // No free floor shut off behind furniture.
  for (let z = r.z; z < r.z + r.d; z++) for (let x = r.x; x < r.x + r.w; x++) if (free(x, z) && !seen.has(key(x, z))) return false;
  return true;
}

/**
 * Whether a home washes with nowhere to dry the washing: something on its floors leaves a washed load
 * (content `makes`: the washer's *Take out the washing*) and nothing works on one (`workOn`: a dryer).
 */
function washesWithoutDrying(info: ObjectInfos, floors: (Floor | null)[]): boolean {
  const uses = floors.flatMap((f) => f?.objects ?? []).flatMap((o) => info(o.def)?.interactions ?? []);
  return uses.some((i) => i.makes === 'washed') && !uses.some((i) => i.workOn === 'washed');
}

/**
 * Things outside the house: the washing line first, for a home that washes with nowhere to dry
 * (`line`: behind the house or to its sides, out on the lawn with a clear tile all round, by the
 * garden door if it fits), then tables with chairs round them, then a few things.
 */
function garden(cfg: HouseGenConfig, info: ObjectInfos, rng: () => number, g: Floor, house: Rect, lot: { width: number; depth: number }, line = false): void {
  const clear = (x: number, z: number) => x >= 0 && z >= 0 && x < lot.width && z < lot.depth && (x < house.x - 1 || x > house.x + house.w || z < house.z - 1 || z > house.z + house.d) && !g.taken.has(key(x, z));
  // Behind the house or to its sides (the front garden is left for the path and drive).
  const spot = (w: number, d: number): [number, number] => [Math.floor(rng() * (lot.width - w)), house.z + Math.floor(rng() * (lot.depth - house.z - d))];
  // Just outside the garden door, if there is one (else anywhere).
  const back = house.z + house.d;
  const gardenDoor = [...g.edges].find(([e, v]) => v === 'door' && e.startsWith('h:') && Number(e.split(':')[2]) === back)?.[0];
  const doorX = gardenDoor ? Number(gardenDoor.split(':')[1]) : null;
  const near = (w: number, d: number): [number, number] => (doorX === null ? spot(w, d) : [doorX - Math.floor(w / 2) + Math.floor(rng() * 5) - 2, back + 1 + Math.floor(rng() * 2)]);
  const washingLine = line ? cfg.garden.washingLine : undefined;
  const lineSize = washingLine ? info(washingLine)?.footprint : undefined;
  if (washingLine && lineSize) {
    // Along the lot (its front to the back or to the house), a clear tile all round it.
    const [w, d] = lineSize;
    for (let tries = 0; tries < 60; tries++) {
      const [x, z] = tries < 20 ? near(w + 2, d + 2) : spot(w + 2, d + 2);
      const tiles = tilesOf(x, z, w + 2, d + 2);
      if (!tiles.every((t) => clear(...(t.split(':').map(Number) as [number, number])))) continue;
      for (const t of tiles) g.taken.add(t);
      g.objects.push({ def: washingLine, x: x + 1, z: z + 1, rot: rng() < 0.5 ? 0 : 2 });
      g.fronts.push(null);
      break;
    }
  }
  // A table with chairs round it, with a clear tile round the chairs (by the garden door if it fits).
  for (const set of cfg.garden.sets ?? []) {
    if (rng() >= (set.chance ?? 1)) continue;
    const chairs = set.chairs.filter((d) => info(d));
    const tables = set.tables.filter((d) => info(d));
    if (!chairs.length || !tables.length) continue;
    const chair = pick(rng, chairs);
    const def = pick(rng, tables);
    const rot = rng() < 0.5 ? 0 : 1;
    const [fw, fd] = info(def)!.footprint;
    const [w, d] = rot ? [fd, fw] : [fw, fd];
    const seats = seatTiles(info(def)!, 0, 0, rot).slice(0, between(rng, set.seats));
    for (let tries = 0; tries < 30; tries++) {
      const [x, z] = tries < 15 ? near(w + 4, d + 4) : spot(w + 4, d + 4);
      const tiles = tilesOf(x, z, w + 4, d + 4);
      if (!tiles.every((t) => clear(...(t.split(':').map(Number) as [number, number])))) continue;
      for (const t of tiles) g.taken.add(t);
      g.objects.push({ def, x: x + 2, z: z + 2, rot });
      g.fronts.push(null);
      for (const s of seats) {
        g.objects.push({ def: chair, x: x + 2 + s.x, z: z + 2 + s.z, rot: s.rot });
        g.fronts.push(null);
      }
      break;
    }
  }
  const count = between(rng, cfg.garden.count);
  for (let n = 0, tries = 0; n < count && tries < 40; tries++) {
    const def = pick(rng, cfg.garden.objects);
    const fp = info(def)?.footprint;
    if (!fp) continue;
    const [w, d] = fp;
    const [x, z] = spot(w, d);
    const tiles: [number, number][] = [];
    for (let dz = -1; dz <= d; dz++) for (let dx = -1; dx <= w; dx++) tiles.push([x + dx, z + dz]);
    if (!tiles.every(([tx, tz]) => clear(tx, tz))) continue;
    for (const [tx, tz] of tiles) g.taken.add(key(tx, tz));
    g.objects.push({ def, x, z, rot: Math.floor(rng() * 4) & 2 });
    g.fronts.push(null);
    n++;
  }
}

/** Unit edges merged into wall runs, and the doors and windows in them. */
function edgesOut(f: Floor): { walls: Wall[]; doors: Door[]; windows: Door[] } {
  const walls: Wall[] = [];
  const doors: Door[] = [];
  const windows: Door[] = [];
  const h = new Map<number, number[]>();
  const v = new Map<number, number[]>();
  for (const [e, kind] of f.edges) {
    const [axis, xs, zs] = e.split(':');
    const [x, z] = [Number(xs), Number(zs)];
    if (axis === 'h') (h.get(z) ?? h.set(z, []).get(z)!).push(x);
    else (v.get(x) ?? v.set(x, []).get(x)!).push(z);
    if (kind === 'door') doors.push({ x, z, axis: axis === 'h' ? 'x' : 'z' });
    if (kind === 'window') windows.push({ x, z, axis: axis === 'h' ? 'x' : 'z' });
  }
  const runs = (list: number[], emit: (a: number, b: number) => void) => {
    list.sort((a, b) => a - b);
    let start = list[0];
    for (let i = 1; i <= list.length; i++) {
      if (i < list.length && list[i] === list[i - 1] + 1) continue;
      emit(start, list[i - 1] + 1);
      start = list[i];
    }
  };
  for (const [z, xs] of h) runs(xs, (a, b) => walls.push([a, z, b, z]));
  for (const [x, zs] of v) runs(zs, (a, b) => walls.push([x, a, x, b]));
  return { walls, doors, windows };
}
