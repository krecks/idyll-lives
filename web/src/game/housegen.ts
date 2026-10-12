/**
 * Generated houses (0.18): every neighbour's home is laid out from `content/housegen.json`.
 *
 * A house class is picked by lot size (bedrooms, size, whether it has a living room, a second
 * bathroom or a second storey), then:
 * 1. the footprint is split into rooms (a guillotine plan: each cut divides the rooms left into
 *    two groups by the floor they want), so every room is a rectangle and the plan stays connected;
 * 2. doors join the rooms (a tree from the entrance, preferring doors onto the living room or
 *    kitchen), the front door faces the street; kitchen and living room may share an open space;
 * 3. two-storey houses get stairs along an outside wall of the living room, an upstairs hall over
 *    them, and the bedrooms and a bathroom off that hall;
 * 4. windows go along the outside walls, daylight enough for every room that has one; kitchens
 *    get a counter run along one wall (sink, stove); furniture goes against the walls (doors and
 *    stairs kept clear, every piece reachable), tables stand free with chairs round them, small
 *    things go on tops (counters, tables, shelves), floor lamps and such keep to corners and out
 *    of doorways, every room gets ceiling lights enough for all of it at night, and a few things
 *    (a patio set) go into the garden.
 *
 * Seeded: the same seed, lot size and config always give the same house, so a town keeps only
 * `gen:<lot size>:<seed>` per lot (see `town.ts`). Output is a `HouseTemplate` in the layouts'
 * frame (drawn for a medium lot, front to the street at z = 0), with `upper` for storey 1.
 */

import type { ObjectDef, SurfaceKind } from '../content/content';
import { approachCandidates, covers, freeSpots, frontTile, middleOf, topOf, type Shape, type Shapes, type Top } from './surfaces';
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
  garden: { objects: string[]; count: [number, number]; sets?: TableSet[] };
  /** Ceiling lights for rooms whose own `ceiling` slots have none (the upstairs hall). */
  lights?: string[];
  names: { first: string[]; second: Record<string, string[]> };
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

interface RoomSpec {
  weight: number;
  minSide: number;
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
  /** Things on walls: on the wall behind one of these if the room has one (a mirror over the basin). */
  over?: string[];
}

/** Counter pieces side by side along a wall: each of `with` (one id per slot) among plain `segments`, `length` tiles in all. */
interface CounterRun {
  segments: string[];
  with: Slot[];
  length: [number, number];
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
  /** Whether it has interactions: someone must be able to get to it. */
  used: boolean;
  /** Whether it gives light (content `light`): it counts for its room's light at night. */
  lamp: boolean;
}

/** An object type's details, or undefined if it doesn't exist. */
export type ObjectInfos = (def: string) => ObjectInfo | undefined;

/** The generator's view of the game's content (`Content`, or the content files read by a tool). */
export function objectInfos(content: { object(id: string): ObjectDef | undefined; radius(def: ObjectDef | undefined): number }): ObjectInfos {
  const known = new Map<string, ObjectInfo | undefined>();
  return (def) => {
    if (!known.has(def)) {
      const o = content.object(def);
      known.set(def, o && { ...o, footprint: o.footprint ?? [1, 1], radius: content.radius(o), used: (o.interactions ?? []).length > 0, lamp: !!o.light });
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
  /** Tiles taken by furniture. */
  taken: Set<string>;
  /** Tiles taken on the wall, ceiling and rug layers (`wall:x:z`, ...). */
  layers?: Set<string>;
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
 * `lampTiles`: tiles one lamp lights at night (content `roomRules.lampTiles`); every room gets
 * lights enough for all of it. `windowTiles`: tiles one window lights by day (`roomRules.windowTiles`).
 */
export function generateHouse(cfg: HouseGenConfig, info: ObjectInfos, lot: { width: number; depth: number }, medium: { width: number }, lotSize: string, seed: number, lampTiles = 10, windowTiles = 6): HouseTemplate | null {
  // A few tries: some draws don't fit (too many rooms for the floor, no wall for the stairs).
  for (let attempt = 0; attempt < 12; attempt++) {
    const house = attemptHouse(cfg, info, lot, lotSize, random(seed * 31 + attempt), lampTiles, windowTiles);
    if (!house) continue;
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
    house.spawns = house.spawns.map(([x, z]) => [x + dx, z]);
    house.id = `gen:${lotSize}:${seed}`;
    return house;
  }
  return null;
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

function attemptHouse(cfg: HouseGenConfig, info: ObjectInfos, lot: { width: number; depth: number }, lotSize: string, rng: () => number, lampTiles: number, windowTiles: number): HouseTemplate | null {
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
  const chanceOf = (e: ExtraRoom) => (typeof e.chance === 'number' ? e.chance : (e.chance[lotSize] ?? 0));
  const extras = (cfg.extras ?? []).filter((e) => cfg.rooms[e.kind] && bedrooms >= (e.minBedrooms ?? 0) && chanceOf(e) > 0);
  for (const e of extras.filter((e) => rng() < chanceOf(e))) add(e.kind, two && e.floor === 'upper' ? upstairs : ground);
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
  let g: Floor | null = null;
  let house: Rect = { x: 0, z: front, w: W, d: D };
  for (let step = 0; step < 8; step++) {
    const slack = lot.width - 2 * margins.side - W;
    house = { x: margins.side + Math.floor(rng() * (slack + 1)), z: front, w: W, d: D };
    g = floorPlan(cfg, rng, house, ground);
    if (g && frontDoor(g, house, rng)) break;
    g = null;
    if (ground.length > baseGround && step % 2 === 0) ground.pop();
    else if (W >= D && W > 6) W--;
    else if (D > 6) D--;
    else break;
  }
  if (!g) return null;
  connect(g, rng, rng() < cfg.openPlan);

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
  windows(cfg, rng, g, house, windowTiles);
  if (up) windows(cfg, rng, up, house, windowTiles);
  if (!furnish(cfg, info, rng, g, has, lampTiles)) return null;
  if (up && !furnish(cfg, info, rng, up, has, lampTiles)) return null;
  garden(cfg, info, rng, g, house, lot);

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
  ].filter((p): p is string => !!p);
  const listed = parts.length > 1 ? `${parts.slice(0, -1).join(', ')} and ${parts[parts.length - 1]}` : parts[0];
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
  };
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

/** Splits `rect` among `kinds` (a guillotine plan); null if they don't fit. */
function floorPlan(cfg: HouseGenConfig, rng: () => number, rect: Rect, kinds: Kind[]): Floor | null {
  for (let tries = 0; tries < 60; tries++) {
    const order = [...kinds].sort(() => rng() - 0.5);
    const rooms = split(cfg, rng, rect, order);
    if (rooms) {
      const floor: Floor = { rooms, edges: new Map(), objects: [], fronts: [], keep: new Set(), taken: new Set() };
      walls(floor);
      return floor;
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

/** The front door, from the street into the entry or the living room (else the kitchen, else any room but a bathroom). */
function frontDoor(f: Floor, house: Rect, rng: () => number): boolean {
  const facing = (r: Room) => r.z === house.z && r.w >= 2;
  const entry = ['entry', 'living', 'kitchen'].map((k) => f.rooms.find((r) => r.kind === k && facing(r))).find((r) => r) ?? f.rooms.find((r) => !['bathroom', 'laundry'].includes(r.kind) && facing(r));
  if (!entry) return false;
  const edges = Array.from({ length: entry.w }, (_, i) => `h:${entry.x + i}:${house.z}`);
  return door(f, rng, edges);
}

const PUBLIC: RoomKind[] = ['entry', 'living', 'kitchen', 'dining'];

/** Doors so every room can be reached: a tree from the entrance, through the living room and kitchen where possible. */
function connect(f: Floor, rng: () => number, openPlan: boolean): void {
  const rooms = f.rooms;
  const entry = rooms.find((r) => sides([...f.edges].find(([, v]) => v === 'door')?.[0] ?? '').some(([x, z]) => inside(r, x, z)))!;
  const reached = new Set<Room>([entry]);
  // Open plan: kitchen and living room as one space (their shared wall goes).
  if (openPlan) {
    const k = rooms.find((r) => r.kind === 'kitchen');
    const l = rooms.find((r) => r.kind === 'living');
    const wall = k && l ? shared(k, l) : [];
    if (wall.length >= 2) {
      // The way between them stays clear of furniture.
      for (const e of wall) {
        f.edges.delete(e);
        for (const [x, z] of sides(e)) f.keep.add(key(x, z));
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
        // Rooms open to each other need no door; bathrooms and bedrooms open off the shared rooms.
        const score = (open ? 10 : 0) + (PUBLIC.includes(from.kind) ? 3 : 0) + (to.kind === 'bathroom' && from.kind === 'bedroom' ? -2 : 0) + rng();
        if (!best || score > best.score) best = { from, to, edges: edges.filter((e) => f.edges.has(e)), score };
      }
    }
    if (!best) break;
    if (best.edges.length) door(f, rng, best.edges);
    reached.add(best.to);
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
  for (const room of g.rooms.filter((r) => STAIR_ROOMS.includes(r.kind)).sort(() => rng() - 0.5)) {
    for (const c of [house.x, house.x + house.w - 1].sort(() => rng() - 0.5)) if (c >= room.x && c < room.x + room.w) spots.push([c, room]);
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
      const plan = upperRooms(cfg, rng, rest, kinds);
      if (!plan) continue;
      const { rooms, corridor } = plan;
      // The hall and corridor have no furniture of their own; they're one space.
      const halls: Room[] = [{ ...hall, kind: 'hall' }, ...(corridor ? [{ ...corridor, kind: 'hall' as const }] : [])];
      const up: Floor = { rooms: [...halls, ...rooms], edges: new Map(), objects: [], fronts: [], keep: new Set(), taken: new Set() };
      walls(up);
      if (corridor) for (const e of shared(hall, corridor)) up.edges.delete(e);
      for (const r of rooms) {
        const edges = (corridor && shared(corridor, r).length ? shared(corridor, r) : null) ?? shared(hall, r);
        if (!door(up, rng, edges)) return null;
      }
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
      for (const k of [...kinds].sort(() => rng() - 0.5)) {
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
    const order = [...kinds].sort(() => rng() - 0.5);
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
 * Furnishes each room: its counter run, the furniture that must come, its tables and chairs
 * (some only in houses with or without a room of some kind: `has`), the rest of its furniture,
 * small things on the tops, then things on the walls, ceiling and floor, and lights enough
 * (halls get only those). False if a piece that must come doesn't fit.
 */
function furnish(cfg: HouseGenConfig, info: ObjectInfos, rng: () => number, f: Floor, has: Set<Kind>, lampTiles: number): boolean {
  const shapes: Shapes = { object: info, radius: (d) => d?.radius ?? 0.12 };
  for (const r of f.rooms) {
    if (r.kind === 'hall') {
      lights(cfg, info, f, r, lampTiles);
      continue;
    }
    const spec = cfg.rooms[r.kind];
    if (spec.run && !counterRun(f, r, info, rng, spec.run)) return false;
    const sideOf = new Map<string, number>();
    const required = (slot: Slot) => slot.chance === undefined || slot.chance >= 1;
    const furniture = (slot: Slot) => {
      const count = slot.count ? between(rng, slot.count) : 1;
      for (let n = 0; n < count; n++) {
        if (!required(slot) && rng() >= (slot.chance ?? 1)) continue;
        const options = slot.one.filter((d) => info(d)).sort(() => rng() - 0.5);
        let done = false;
        for (const def of options) {
          const across = slot.opposite ? sideOf.get(slot.opposite) : undefined;
          const prefer = across === undefined ? undefined : (across + 2) % 4;
          // Things nobody uses (a floor lamp, a vase on a stand) keep out of the way in doorways;
          // one that must come goes wherever it fits if it has to.
          const aside = !info(def)!.used;
          const side = place(f, r, def, info(def)!, rng, prefer, aside) ?? (aside && required(slot) ? place(f, r, def, info(def)!, rng, prefer) : null);
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
          const fp = info(def)?.footprint;
          if (fp) mount(cfg, info, f, r, def, fp, layer, rng, slot.over);
        }
      }
    }
    lights(cfg, info, f, r, lampTiles);
  }
  return true;
}

/**
 * Lights from the ceiling until room `r` is lit right through at night: one lamp lights
 * `lampTiles` tiles (as the simulation scores rooms), and lamps already in it (on the floor,
 * on tops, on walls, from the ceiling) count. They spread over the room, each in the middle of
 * its share of it, all alike: the light already hanging there, else one of the room's own
 * ceiling lights (else of `lights`). Draws nothing from the house's random numbers.
 */
function lights(cfg: HouseGenConfig, info: ObjectInfos, f: Floor, r: Room, lampTiles: number): void {
  const want = Math.ceil((r.w * r.d) / Math.max(lampTiles, 0.1));
  const lamps = () => f.objects.filter((o) => inside(r, o.x, o.z) && info(o.def)?.lamp);
  if (lamps().length >= want) return;
  const hanging = (d: string) => !!info(d)?.lamp && info(d)?.layer === 'ceiling';
  const own = (cfg.rooms[r.kind]?.ceiling ?? []).flatMap((s) => s.one).filter(hanging);
  const options = own.length ? own : (cfg.lights ?? []).filter(hanging);
  const def = lamps().find((o) => hanging(o.def))?.def ?? options[(r.x + r.z) % options.length];
  if (!def) return;
  // Hanging from the ceiling draws no random numbers.
  const rng = () => 0.5;
  // The room in `want` shares (or a few more), as near square as can be.
  let grid: [number, number] = [want, 1];
  let best = Infinity;
  for (let cols = 1; cols <= want; cols++) {
    const rows = Math.ceil(want / cols);
    const score = cols * rows * 100 + Math.abs(r.w / cols - r.d / rows);
    if (score < best) [best, grid] = [score, [cols, rows]];
  }
  const [cols, rows] = grid;
  const spots: [number, number][] = [];
  for (let j = 0; j < rows; j++) for (let i = 0; i < cols; i++) spots.push([r.x + ((i + 0.5) * r.w) / cols - 0.5, r.z + ((j + 0.5) * r.d) / rows - 0.5]);
  for (const [sx, sz] of spots) {
    if (lamps().length >= want) return;
    // A share with a ceiling light over it already (one hung above) is lit.
    if (lamps().some((o) => info(o.def)?.layer === 'ceiling' && Math.hypot(o.x - sx, o.z - sz) < 1.5)) continue;
    mount(cfg, info, f, r, def, info(def)!.footprint, 'ceiling', rng, undefined, [sx, sz]);
  }
  // Shares that were lit already, or had no free ceiling: wherever there's room.
  for (let n = lamps().length; n < want; n++) {
    if (!mount(cfg, info, f, r, def, info(def)!.footprint, 'ceiling', rng)) return;
  }
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
 * each used from the row in front (kept clear). Shorter if the walls are short; if no wall has
 * room, the pieces go against the walls one by one. False if a piece that must come doesn't fit.
 */
function counterRun(f: Floor, r: Room, info: ObjectInfos, rng: () => number, run: CounterRun): boolean {
  const pieces: { def: string; w: number; required: boolean }[] = [];
  for (const slot of run.with) {
    if (slot.chance !== undefined && rng() >= slot.chance) continue;
    const def = slot.one.filter((d) => info(d)?.footprint[1] === 1).sort(() => rng() - 0.5)[0];
    if (def) pieces.push({ def, w: info(def)!.footprint[0], required: slot.chance === undefined || slot.chance >= 1 });
  }
  const segments = run.segments.filter((d) => info(d)?.footprint.every((n) => n === 1));
  const segment = segments.length ? pick(rng, segments) : null;
  const least = pieces.reduce((n, p) => n + p.w, 0);
  for (let len = Math.max(least, between(rng, run.length)); len >= Math.max(least, 1); len--) {
    if (len > least && !segment) continue;
    const order = [...pieces.map((p) => p.def), ...Array<string>(len - least).fill(segment ?? '')].sort(() => rng() - 0.5);
    for (const side of [0, 1, 2, 3].sort(() => rng() - 0.5)) {
      // Back to the wall on `side` (0 the side towards the street, then clockwise), as in `place`.
      const rot = [0, 3, 2, 1][side];
      const along = side % 2 === 0;
      if ((along ? r.d : r.w) < 2) continue;
      const starts = Array.from({ length: (along ? r.w : r.d) - len + 1 }, (_, i) => i).sort(() => rng() - 0.5);
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
          return true;
        }
        undo(f, n, tiles);
      }
    }
  }
  // No wall long enough: one by one.
  return pieces.every((p) => place(f, r, p.def, info(p.def)!, rng) !== null || !p.required);
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
  for (const def of set.tables.filter((d) => info(d)).sort(() => rng() - 0.5)) {
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
            // Facing into the room (a table nobody uses still needs its front open).
            const rot = inside(r, ...frontTile(x, z, w, d, turned)) ? turned : turned + 2;
            // Room for chairs and a way round them on every side; else as near the middle as can be.
            const gap = Math.min(x - r.x, z - r.z, r.x + r.w - x - w, r.z + r.d - z - d);
            spots.push({ x, z, rot, score: Math.min(gap, 2) * 4 - Math.hypot(x + w / 2 - cx, z + d / 2 - cz) + rng() });
          }
        }
      }
    }
    spots.sort((a, b) => b.score - a.score);
    for (const { x, z, rot } of spots.slice(0, 24)) {
      const [w, d] = rot % 2 === 0 ? [fw, fd] : [fd, fw];
      const mine = tilesOf(x, z, w, d);
      const front = frontTile(x, z, w, d, rot);
      if (mine.some((t) => f.taken.has(t) || f.keep.has(t)) || !inside(r, front[0], front[1])) continue;
      if (table.used && f.taken.has(key(front[0], front[1]))) continue;
      // Chairs along a desk's front only; stools along one side of an island (its back, if someone uses it).
      const [fs, bs] = [[2, 1, 0, 3][rot % 4], [0, 3, 2, 1][rot % 4]];
      const one = (w === d ? [[0, 2], [1, 3]] : w > d ? [[0], [2]] : [[1], [3]]).sort(() => rng() - 0.5);
      const choices = set.wall ? [[fs]] : set.oneSide ? (table.used ? [[bs]] : one) : [undefined];
      const free = (s: { x: number; z: number }) => inside(r, s.x, s.z) && !f.taken.has(key(s.x, s.z)) && !f.keep.has(key(s.x, s.z));
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
 * in its middle, elsewhere anywhere free), facing about the way its host does, listed right
 * after its host (the simulation places it on a top that's already there).
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
      .filter((t): t is Top => !!t && inside(r, t.host.x, t.host.z) && slot.on.includes(t.kind) && (!slot.of || slot.of.includes(t.host.def)) && 2 * Math.min(t.hw, t.hd) >= (slot.minSide ?? 0));
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
 * its back on wall or window edges; from the ceiling, over a free spot; a rug, in the middle of
 * the room. Never on the stairs, and its front (where the simulation checks for walls) inside the room.
 * `near`: from the ceiling, as near this tile as there's room (else near the middle).
 */
function mount(cfg: HouseGenConfig, info: ObjectInfos, f: Floor, r: Room, def: string, [fw, fd]: [number, number], layer: 'wall' | 'ceiling' | 'rug', rng: () => number, over?: string[], near?: [number, number]): boolean {
  const layers = (f.layers ??= new Set());
  const stairs = new Set<string>();
  for (const o of f.objects) {
    if (o.def !== cfg.stairs) continue;
    const [sw, sd] = info(o.def)?.footprint ?? [1, 4];
    const [w, d] = o.rot % 2 === 0 ? [sw, sd] : [sd, sw];
    for (let dz = 0; dz < d; dz++) for (let dx = 0; dx < w; dx++) stairs.add(key(o.x + dx, o.z + dz));
  }
  // Nor over the stairwell upstairs (all a hall has taken).
  if (r.kind === 'hall') for (const t of f.taken) stairs.add(t);
  const fits = (x: number, z: number, w: number, d: number, rot: number) => {
    for (let dz = 0; dz < d; dz++) {
      for (let dx = 0; dx < w; dx++) {
        const t = key(x + dx, z + dz);
        if (!inside(r, x + dx, z + dz) || stairs.has(t) || layers.has(`${layer}:${t}`)) return false;
      }
    }
    const [frx, frz] = frontTile(x, z, w, d, rot);
    return inside(r, frx, frz);
  };
  const take = (x: number, z: number, w: number, d: number, rot: number) => {
    for (let dz = 0; dz < d; dz++) for (let dx = 0; dx < w; dx++) layers.add(`${layer}:${key(x + dx, z + dz)}`);
    f.objects.push({ def, x, z, rot });
    f.fronts.push(null);
    return true;
  };
  if (layer === 'wall') {
    const backed = (e: string) => f.edges.get(e) === 'wall' || f.edges.get(e) === 'window';
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
      if (backs.every(backed) && fits(wx, wz, w, d, o.rot)) return take(wx, wz, w, d, o.rot);
    }
    for (const side of [0, 1, 2, 3].sort(() => rng() - 0.5)) {
      const rot = [0, 3, 2, 1][side];
      const [w, d] = rot % 2 === 0 ? [fw, fd] : [fd, fw];
      const spots: [number, number, string[]][] = [];
      if (side === 0) for (let x = r.x; x + w <= r.x + r.w; x++) spots.push([x, r.z, Array.from({ length: w }, (_, i) => `h:${x + i}:${r.z}`)]);
      if (side === 2) for (let x = r.x; x + w <= r.x + r.w; x++) spots.push([x, r.z + r.d - d, Array.from({ length: w }, (_, i) => `h:${x + i}:${r.z + r.d}`)]);
      if (side === 3) for (let z = r.z; z + d <= r.z + r.d; z++) spots.push([r.x, z, Array.from({ length: d }, (_, i) => `v:${r.x}:${z + i}`)]);
      if (side === 1) for (let z = r.z; z + d <= r.z + r.d; z++) spots.push([r.x + r.w - w, z, Array.from({ length: d }, (_, i) => `v:${r.x + r.w}:${z + i}`)]);
      spots.sort(() => rng() - 0.5);
      // Curtains go over windows; everything else prefers a plain wall.
      const window = def.startsWith('curtains');
      spots.sort((a, b) => Number(b[2].some((e) => f.edges.get(e) === 'window') === window) - Number(a[2].some((e) => f.edges.get(e) === 'window') === window));
      for (const [x, z, backs] of spots) if (backs.every(backed) && fits(x, z, w, d, rot)) return take(x, z, w, d, rot);
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
      if (fits(x, z, w, d, rot)) return take(x, z, w, d, rot);
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
 * tile inside a door (it would make the way in a squeeze).
 */
function place(f: Floor, r: Room, def: string, { footprint: [fw, fd] }: ObjectInfo, rng: () => number, prefer?: number, aside = false): number | null {
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
  const sideList = [0, 1, 2, 3].sort(() => rng() - 0.5);
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
      along.sort(() => rng() - 0.5);
      for (const [x, z] of along) yield { side, rot, w, d, x, z };
    }
  }
  // Out of the way: corners first, whichever side they're on.
  const corner = (s: Spot) => Number((s.x === r.x || s.x + s.w === r.x + r.w) && (s.z === r.z || s.z + s.d === r.z + r.d));
  const spots = aside ? [...walls()].sort((a, b) => corner(b) - corner(a)) : walls();
  for (const { side, rot, w, d, x, z } of spots) {
    const tiles = tilesOf(x, z, w, d);
    const front = frontTile(x, z, w, d, rot);
    if (!inside(r, front[0], front[1])) continue;
    if (aside && tiles.some((t) => doorway.has(t))) continue;
    if (tiles.some((t) => f.taken.has(t) || f.keep.has(t)) || f.taken.has(key(front[0], front[1]))) continue;
    const n = f.objects.length;
    for (const t of tiles) f.taken.add(t);
    f.objects.push({ def, x, z, rot });
    f.fronts.push([front]);
    if (reachable(f, r)) {
      f.keep.add(key(front[0], front[1]));
      return side;
    }
    undo(f, n, tiles);
  }
  return null;
}

/**
 * Every free tile of the room the doors need is still connected, and so is a way to everything
 * in it: its front tile (which must be free), or for chairs one of the free tiles round them.
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
  return needed.every((k) => seen.has(k)) && ways.every((w) => w.some((k) => seen.has(k)));
}

function garden(cfg: HouseGenConfig, info: ObjectInfos, rng: () => number, g: Floor, house: Rect, lot: { width: number; depth: number }): void {
  const clear = (x: number, z: number) => x >= 0 && z >= 0 && x < lot.width && z < lot.depth && (x < house.x - 1 || x > house.x + house.w || z < house.z - 1 || z > house.z + house.d) && !g.taken.has(key(x, z));
  // Behind the house or to its sides (the front garden is left for the path and drive).
  const spot = (w: number, d: number): [number, number] => [Math.floor(rng() * (lot.width - w)), house.z + Math.floor(rng() * (lot.depth - house.z - d))];
  // A table with chairs round it, with a clear tile round the chairs.
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
      const [x, z] = spot(w + 4, d + 4);
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
