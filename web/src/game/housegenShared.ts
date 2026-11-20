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
 *    things go on tops (counters, tables, shelves) a resident can get at (free floor beside the
 *    top, else a seat pulled up to it, as the game's load check has it), floor lamps and such keep to corners and out
 *    of doorways, every room gets ceiling lights enough for all of it at night, and a few things
 *    (a patio set) go into the garden, by the garden door. The furniture is for whoever lives
 *    there (`Residents`): a couple's bed for two, a bed each for everyone else (a child's for
 *    their age: no race car bed for a teenager), toys only where there are children young enough
 *    for them, and spare bedrooms (and playrooms) as guest rooms, studies and such.
 *
 * Seeded: the same seed, lot size and config always give the same house (laid out the same for
 * any residents), so a town keeps only `gen:<lot size>:<seed>[:<garden>]` per lot (see `town.ts`).
 * With a `GardenSize` the lot is cut to fit the house (`fitLot`): a front yard, a garden behind
 * and beside it in proportion to the house, room for a driveway on one side; the house's `lot`
 * says how big it came out, and the layout is in that lot's frame. Without one (older ids) it is
 * in the layouts' frame (drawn for a medium lot). Front to the street at z = 0, with `upper` for
 * storey 1 and `basement` for storey -1 (both on the same rows as the ground).
 */
import type { ObjectDef, SurfaceKind } from '../content/content';
import { type Shape } from './surfaces';
import type { Placement } from './town';
import { mulberry32 as random } from './noise';
export interface HouseGenConfig {
  classes: Record<string, HouseClass>;
  /** Class weights per lot size. */
  lots: Record<string, Record<string, number>>;
  margins: { front: [number, number]; back: number; side: number };
  /** How much garden a lot cut to fit its house gets (`fitLot`). */
  gardens?: Record<GardenSize, GardenSpec>;
  rooms: Record<Kind, RoomSpec>;
  /** Rooms besides bedrooms, kitchen, bathroom and living room, where the house has more floor than its rooms may take. */
  extras?: ExtraRoom[];
  openPlan: number;
  /** `daylight`: the share of each room's floor its windows should light by day (`windowTiles` each). */
  windows: { spacing: [number, number]; daylight?: number };
  stairs: string;
  /** `washingLine`: put in the garden of a home that washes with nowhere to dry the washing (`washesWithoutDrying`). */
  garden: { objects: string[]; count: [number, number]; sets?: TableSet[]; washingLine?: string; area?: number };
  /** Ceiling lights for rooms whose own `ceiling` slots have none (the upstairs hall, the basement's). */
  lights?: string[];
  basement?: BasementSpec;
  /** How a house is furnished for the people living in it (see `Residents`). */
  residents?: ResidentsSpec;
  names: { first: string[]; second: Record<string, string[]> };
}
/**
 * Who lives in a house: `couples` (a bed for two), `adults` on their own and `children` (a bed
 * each), `babies` (a crib, which the simulation brings); `stages`, the children's life stages
 * (content `life.stages` ids, one each: their beds and toys are for them; absent: any child's).
 * The layout never depends on it, only the furniture: undefined is an empty house.
 */
export interface Residents {
  couples: number;
  adults: number;
  children: number;
  babies: number;
  stages?: string[];
}
/** How much garden a house's lot has round it (see `fitLot`). */
export type GardenSize = 'snug' | 'roomy' | 'big';
export const GARDEN_SIZES: readonly GardenSize[] = ['snug', 'roomy', 'big'];
/**
 * A lot cut to fit its house: `back`, the garden behind it as a share of the house's depth (a
 * range, and at least and at most `backMin`..`backMax` m); `side`, the garden beside it (m, a
 * range); `drive`, the least on the driveway's side.
 */
export interface GardenSpec {
  back: [number, number];
  backMin: number;
  backMax: number;
  side: [number, number];
  drive: number;
}
/** A key for `residents` (empty: undefined). */
export function residentsKey(r: Residents | undefined): string {
  return r ? `${r.couples}.${r.adults}.${r.children}.${r.babies}${r.stages?.length ? `.${[...r.stages].sort().join('+')}` : ''}` : '';
}
/**
 * Furnishing for the residents: the beds bedrooms get (`couple`, `single` for a room two share,
 * `child`, `bunk` for two children short of floor), what goes into a child's room besides
 * (`childRoom`), what a bedroom nobody sleeps in becomes (`spareRoom`, by weight: `guest`, a
 * bedroom with a single bed, else a room kind), and what only a home with children has (`kidsOnly`:
 * content categories and groups; a playroom without children becomes a spare room too).
 */
export interface ResidentsSpec {
  beds: { couple: string[]; single: string[]; child: string[]; bunk?: string[] };
  childRoom?: Slot[];
  spareRoom: Record<string, number>;
  kidsOnly: { categories?: string[]; groups?: string[] };
}
export interface HouseClass {
  label: string;
  bedrooms: [number, number];
  /** Chance of a living room. */
  living: number;
  /** Chance of a second bathroom. */
  extraBathroom: number;
  storeys: { one: number; two: number };
  width: [number, number];
  depth: [number, number];
  /**
   * A house in a row (a terrace's or a semi-detached pair's): its lot cut to it with `ROW_GAP` m
   * beside it on one side (its +x) and none on the other, no room for a driveway, and the front
   * yard `margins.front`'s least, so the fronts of a row line up. Next door's house, its own wall
   * on its lot's edge, stands `ROW_GAP` m off.
   */
  row?: boolean;
}
/** The gap between two houses in a row (m): see `HouseClass.row`. */
export const ROW_GAP = 1;
export type RoomKind = Kind | 'hall';
/** Rooms with a recipe in the config (`bedroom`, `kitchen`, `bathroom`, `living` and extras; halls have none). */
export type Kind = string;
/**
 * A room a house may get on top of the ones it must have: only while its floor is bigger than
 * its rooms may take (`maxSide`), if `chance` (a number, or per lot size) says so, with at least
 * `minBedrooms`. `floor: "upper"` puts it upstairs in a two-storey house; `label` names it in the
 * house's description.
 */
export interface ExtraRoom {
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
export interface BasementSpec {
  chance: number | Record<string, number>;
  count: [number, number];
  rooms: { kind: Kind; weight?: number; label?: string }[];
}
export interface RoomSpec {
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
export interface Slot {
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
export interface CounterRun {
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
export interface TableSet {
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
export interface TopSlot {
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
  /** The life stages (content `stages` ids) that may use it, by any of its uses (undefined: anyone): the race car bed and the toy box are a child's. */
  usedBy?: string[];
  /** How far a tree's crown reaches out from its middle (m, content `crown`): no wall closer (`crownClear`). */
  crown?: number;
}
/**
 * Whether no wall of floor `g` (with a door or window or not) stands closer than `crown` m to the
 * point (`x`, `z`): a tree's crown there doesn't grow into the house (as the game's `home.rs`
 * `crown_meets_wall` refuses).
 */
export function crownClear(g: { edges: Map<string, string> }, x: number, z: number, crown: number): boolean {
  for (const e of g.edges.keys()) {
    const [axis, xs, zs] = e.split(':');
    const [ex, ez] = [Number(xs), Number(zs)];
    const [dx, dz] = axis === 'h' ? [x - Math.min(Math.max(x, ex), ex + 1), z - ez] : [x - ex, z - Math.min(Math.max(z, ez), ez + 1)];
    if (Math.hypot(dx, dz) < crown) return false;
  }
  return true;
}
/** Whether object type `o` is a hob, a grill or a sink (see `ObjectInfo.keepClear`). */
export function cooksOrWashes(o: ObjectDef): boolean {
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
      known.set(def, o && { ...o, footprint: o.footprint ?? [1, 1], radius: content.radius(o), used: (o.interactions ?? []).length > 0 || (o.stations ?? []).length > 0, lamp: !!o.light, lightTiles: o.light?.tiles, keepClear: cooksOrWashes(o), seated: (o.interactions ?? []).some((i) => i.pose === 'sit' || i.pose === 'lie'), usedBy: usedBy(o) });
    }
    return known.get(def);
  };
}
/** The life stages that may use `o` (`ObjectInfo.usedBy`): its uses' `stages`, else the object's; undefined if one is anyone's. */
export function usedBy(o: ObjectDef): string[] | undefined {
  const each = (o.interactions ?? []).filter((i) => !i.baby).map((i) => i.stages ?? o.stages);
  if (!each.length) return o.stages;
  return each.some((s) => !s) ? undefined : [...new Set(each.flatMap((s) => s ?? []))];
}
/** Whether someone of life stage `stage` may use `o` (`ObjectInfo.usedBy`; no stage: anyone may). */
export function usableAt(o: ObjectInfo | undefined, stage: string | undefined): boolean {
  return !o?.usedBy || !stage || o.usedBy.includes(stage);
}
export interface Rect {
  x: number;
  z: number;
  w: number;
  d: number;
}
export interface Room extends Rect {
  kind: RoomKind;
}
/** One storey's plan while it's being made (lot coordinates). */
export interface Floor {
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
export const key = (x: number, z: number) => `${x}:${z}`;
// (The small seeded generator, `noise.ts`'s mulberry32.)
export { random };
/**
 * `lampTiles`: tiles a lamp lights at night unless its `light.tiles` says (content
 * `roomRules.lampTiles`); every room gets lights enough for all of it. `windowTiles`: tiles one window lights by day (`roomRules.windowTiles`).
 */
