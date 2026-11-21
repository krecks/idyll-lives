/** Messages between the main thread and the sim worker. */

import type { Appearance, SimSpawn } from '../game/household';
import type { SnapshotLayout } from './snapshot';

/** Mirrors `sim-core/src/command.rs`. */
export type Command =
  | { type: 'setSpeed'; speed: number }
  /** Free will for a household (default: the player's). */
  | { type: 'setAutonomy'; enabled: boolean; household?: number }
  | { type: 'setLifespan'; lifespan: Lifespan }
  | { type: 'setPlayerMoves'; enabled: boolean }
  /** Tests and `?debug` only (`__addFunds`): money for a household (negative takes it away, down to 0). */
  | { type: 'addFunds'; household: number; amount: number }
  /** Tests and `?debug` only (`check.mjs play --skills`): a resident's skill set to a level. */
  | { type: 'setSkill'; sim: number; skill: string; level: number }
  | { type: 'setNeed'; sim: number; need: string; value: number }
  | { type: 'setBond'; a: number; b: number; friendship?: number; romance?: number; chemistry?: number; partners?: boolean }
  | { type: 'adopt'; household: number; child: boolean }
  /** Super speed by itself: the game goes to super speed as the player's household all falls asleep or goes out, and back to the speed before it as that ends. */
  | { type: 'setAutoFast'; enabled: boolean }
  | { type: 'use'; sim: number; object: number; interaction: number }
  /** Eat a serving of food (an item), or clear it away to the sink. */
  | { type: 'useItem'; sim: number; item: number; action: 'eat' | 'clear' }
  /** A lamp switched on or off (sim-core `lights.rs`): at once, nobody walks over. */
  | { type: 'switchLamp'; object: number; on: boolean }
  /** Curtains drawn shut or opened (sim-core `curtains.rs`): at once, nobody walks over. */
  | { type: 'drawCurtains'; object: number; shut: boolean }
  | { type: 'moveTo'; sim: number; x: number; z: number }
  | { type: 'social'; sim: number; target: number; social: number }
  | { type: 'joinCareer'; sim: number; career: number; level: number }
  | { type: 'quitCareer'; sim: number }
  | { type: 'visit'; sim: number; plot: number }
  | { type: 'goHome'; sim: number }
  /** Use the phone: `action` indexes content `phone.actions`; `contact` for texting, calling, inviting over or asking to come over. */
  | { type: 'phone'; sim: number; action: number; contact?: number }
  | { type: 'cancel'; sim: number; index: number }
  /** Build and buy are addressed to a household: its home may have nobody living in it yet. */
  /**
   * `turn`: degrees past the facing, for objects that turn freely (0..89); `offset`: centimetres
   * (x, z) off the middle of the footprint (up to `MAX_OFFSET` each way; things on a wall only
   * along it); `size`: its own size (centimetres across and deep before turning, `MIN_SIZE` to
   * `MAX_SIZE` tiles; it takes them rounded up to whole tiles); `tall`: centimetres taller
   * (negative: shorter) than its kind.
   */
  | { type: 'buy'; household: number; object: string; at?: [number, number, number]; colors?: Record<string, number>; turn?: number; offset?: [number, number]; size?: [number, number]; tall?: number }
  | { type: 'sell'; household: number; object: number }
  /** The paid quick fix for something worn or broken. */
  | { type: 'repair'; household: number; object: number }
  | { type: 'moveObject'; household: number; object: number; x: number; z: number; rot: number; turn?: number; offset?: [number, number]; size?: [number, number]; tall?: number }
  /** `color`: 0xRRGGBB on the parts its model paints; absent: as made. */
  | { type: 'recolor'; household: number; object: number; colors: Record<string, number> }
  | { type: 'upgrade'; household: number; object: number }
  | { type: 'build'; household: number; edits: EdgeEdit[] }
  | { type: 'paint'; household: number; faces: FacePaint[] }
  | { type: 'paintFloor'; household: number; tiles: FloorPaint[] }
  | { type: 'undo'; household: number }
  | { type: 'redo'; household: number }
  /** The roof over the home: a roof style and colour (free). */
  | { type: 'setRoof'; household: number; style: number; color: number }
  /** A saved house (`blueprint` as `World::blueprint` wrote it), built on the household's empty lot. */
  | { type: 'buildBlueprint'; household: number; blueprint: unknown }
  /** Moves the room around tile (x, z) by (dx, dz) tiles, with what stands in it. */
  | { type: 'moveRoom'; household: number; x: number; z: number; dx: number; dz: number }
  /** A garage on the home lot: its walls' centre lines, its door facing `rot` (as a thing's: 0 towards +z, 1 +x, 2 -z, 3 -x); content `build.garage`. */
  | { type: 'buildGarage'; household: number; x0: number; z0: number; x1: number; z1: number; rot: number }
  /** Takes down the home lot's garage (sold back for `objectRules.resale` of its price). */
  | { type: 'sellGarage'; household: number }
  /** A porch over a door of the home (its edge, `h` or `v`, on the ground storey), on the door's outdoor side; content `build.porch`. */
  | { type: 'buildPorch'; household: number; axis: 'h' | 'v'; x: number; z: number }
  /** Takes down the porch over a door of the home (sold back for `objectRules.resale` of its price). */
  | { type: 'removePorch'; household: number; axis: 'h' | 'v'; x: number; z: number }
  /** A porch piece on the home lot: tiles `x .. x + w` × `z .. z + d` (as turned), facing `rot`; content `build.porchPost`, or `porchCanopy` / `porchStep` a tile. */
  | { type: 'buildPorchPiece'; household: number; kind: PorchPieceKind; x: number; z: number; w: number; d: number; rot: number }
  /** Takes down the porch piece of `kind` on tile (x, z) of the home lot (sold back for `objectRules.resale` of its price). */
  | { type: 'removePorchPiece'; household: number; kind: PorchPieceKind; x: number; z: number }
  /** New residents move into the household's home (`bonds` index into `sims`); `name` renames the household. */
  | { type: 'moveIn'; household: number; name?: string; sims: SimSpawn[]; bonds: { a: number; b: number; preset: string }[] }
  /** The resident's own routine blocks (the whole list). */
  | { type: 'setRoutines'; sim: number; routines: RoutineIn[] }
  /** The routine template of the resident's household. */
  | { type: 'setHouseholdRoutines'; sim: number; routines: RoutineIn[] }
  | { type: 'skipHouseholdRoutine'; sim: number; routine: number; skip: boolean }
  | { type: 'addGoal'; sim: number; goal: GoalIn }
  | { type: 'removeGoal'; sim: number; index: number }
  /** Higher on the list steers more. */
  | { type: 'moveGoal'; sim: number; index: number; to: number }
  | { type: 'acceptSuggestion'; sim: number; index: number }
  | { type: 'dismissSuggestion'; sim: number; index: number }
  /** Yes or no to what a resident asked to buy (a yes buys and places it). */
  | { type: 'acceptPurchase'; sim: number }
  | { type: 'declinePurchase'; sim: number }
  /**
   * The land shaped (Build mode's Land tab): `cost` charged to the household (Living; free in
   * Creative; negative: a refund, an undone stroke). `meta.land` merges into the world's
   * `meta.land` tile by tile (`h` and `wl` by tile index, the rest replaced); `meta.zones`, when
   * given, replaces the world's. `water`: the world's whole water mask (1 m tiles, `Rle`), when
   * the stroke changed the water. Not a home edit: the land's own undo sends the reverse.
   */
  | { type: 'shapeLand'; household: number; cost: number; meta: ShapeLandMeta; water?: Rle };

/**
 * What a `shapeLand` command changes in the world's `meta`: the land's touched tiles, the zones derived from it, and
 * the bridges its water made (`roads`: by road index, each road's whole list, `RoadPlan.bridges`; an empty one none).
 */
export interface ShapeLandMeta {
  land?: Partial<Omit<LandFile, 'h' | 'wl' | 'flow'>> & { h?: Record<number, string>; wl?: Record<number, string>; flow?: Record<number, string> };
  zones?: WorldZone[];
  roads?: Record<number, { bridges: [number, number][] }>;
}

/**
 * A `shapeLand` command, as `SimBridge.shapeLand` sends it (answered, unlike `send`'s commands).
 * The first one of a world without `meta.land` carries the land's whole header (`cols` and `rows`
 * at least); a tile it doesn't send is empty (flat at 0 and dry). A tile index past the land's, a
 * cost over the household's funds (Living), and new water on a lot, on a road (its width with the
 * sidewalks), on a resident or where something stands are refused whole, with the reason.
 */
export type ShapeLandCommand = Extract<Command, { type: 'shapeLand' }>;

/**
 * `meta` with a `shapeLand` stroke's `patch` merged in as the simulation merges it: `land.h`,
 * `land.wl` and `land.flow` (once either has one) tile by tile, the land's other fields and `zones` replaced, a road's `bridges` replaced
 * (none: taken away). A new object (and a new `land`, new `roads` and new road objects where they
 * changed), `meta` left as it is; null with nothing to merge.
 */
export function mergeShapeLand(meta: WorldMeta | null, patch: ShapeLandMeta): WorldMeta | null {
  if (!patch.land && !patch.zones && !patch.roads) return meta;
  const out: WorldMeta = { ...(meta ?? {}) };
  if (patch.land) {
    const { h, wl, flow, ...header } = patch.land;
    const land = { ...out.land, ...header } as LandFile;
    const tiles = Math.ceil(land.cols / (land.tile ?? 64)) * Math.ceil(land.rows / (land.tile ?? 64));
    const merged = (old: string[] | undefined, by: Record<number, string> | undefined) => {
      const codes = Array.from({ length: tiles }, (_, t) => old?.[t] ?? '');
      for (const [t, code] of Object.entries(by ?? {})) codes[Number(t)] = code;
      return codes;
    };
    land.h = merged(out.land?.h, h);
    land.wl = merged(out.land?.wl, wl);
    if (out.land?.flow || flow) land.flow = merged(out.land?.flow, flow);
    out.land = land;
  }
  if (patch.zones) out.zones = patch.zones;
  if (patch.roads && out.roads) {
    const roads = out.roads.slice();
    for (const [i, { bridges }] of Object.entries(patch.roads)) {
      const road = roads[Number(i)];
      if (!road) continue;
      const { bridges: _old, ...rest } = road;
      roads[Number(i)] = bridges.length ? { ...rest, bridges } : rest;
    }
    out.roads = roads;
  }
  return out;
}

/** A routine block: an activity (content `activities` id) on weekdays (bit 0 = Monday) from `start` (minute of the day) for `minutes`. */
export interface RoutineIn {
  id?: number;
  activity: string;
  skill?: string;
  days: number;
  start: number;
  minutes: number;
}

export interface Routine extends RoutineIn {
  id: number;
}

export interface GoalIn {
  def: string;
  skill?: string;
  category?: string;
  /** Object id, for an `own` goal (its price is the target). */
  object?: string;
  target?: number;
}

/** Why a block wasn't kept. */
export interface ReasonView {
  code: 'trait' | 'need' | 'mood' | 'away' | 'noPlace';
  trait?: string;
  need?: string;
}

/** A running or past block. */
export interface BlockView {
  routine: number;
  household: boolean;
  activity: string;
  skill: string | null;
  /** Day it started (from 1). */
  day: number;
  start: number;
  minutes: number;
  status: 'active' | 'kept' | 'cut' | 'skipped' | 'noPlace';
  reason: ReasonView | null;
  /** Minutes spent on it, in steps of 10. */
  done: number;
}

export type GoalKind = 'hasJob' | 'jobLevel' | 'promoted' | 'skill' | 'friends' | 'partner' | 'funds' | 'own';

/** What a goal still takes: a skill level a career goal needs (reached ones drop off), the
 * performance bar (of 100), or the money for something they want. */
export interface GoalStep {
  kind: 'skill' | 'performance' | 'funds';
  skill: string | null;
  have: number;
  need: number;
}

/** A goal; its text comes from the content's goal template (see `goalText`), so it can be translated. */
export interface GoalView {
  def: string;
  kind: GoalKind;
  icon: string;
  /** 0..1 in 1 % steps. */
  progress: number;
  /** Change over the last days (positive: getting there), in 0.01 steps rounded away from zero. */
  trend: number;
  skill: string | null;
  category: string | null;
  /** Object id, for things they want (`target` is its price). */
  object: string | null;
  target: number;
  sinceDay: number;
  steps: GoalStep[];
}

/** Something a resident asks to buy, waiting for a yes or no. */
export interface PurchaseView {
  object: string;
  price: number;
  /** Times they were told no for it before (from the second no on, they're cross). */
  declines: number;
}

export interface HomeWish {
  room?: string;
  factor?: string;
  fix?: string;
  another?: string;
}

/** A resident's planner (the player's household only). */
export interface PlanView {
  routines: Routine[];
  /** Household template blocks this resident doesn't follow. */
  skipHousehold: number[];
  current: BlockView | null;
  /** The last week's blocks, oldest first. */
  history: BlockView[];
  goals: GoalView[];
  suggestions: GoalView[];
  /** `[activity, skill]`: places they'd like (from blocks with nowhere to do them). */
  wishes: [string, string | null][];
  /** What they'd like for the home: a better room (`room`: kind id, "garden", or none for a room
   * nothing marks; `factor`: size, light, decor, clean, function), a fix (`fix`: object id) or
   * another room of a kind (`another`). */
  homeWishes: HomeWish[];
  /** How well they stick to plans (1 = average), in 0.05 steps rounded down. */
  adherence: number;
  purchase: PurchaseView | null;
}

/** A floor tile to cover: `covering` 0 for the automatic floor, else a floor covering + 1. */
export interface FloorPaint {
  x: number;
  z: number;
  covering: number;
}

/** One wall edge: `h` runs from (x, z) to (x + 1, z); `v` from (x, z) to (x, z + 1). */
export interface WallEdge {
  axis: 'h' | 'v';
  x: number;
  z: number;
  /** Covering of each face (0: automatic, else `wallCoverings` index + 1); face 0 looks towards -z (h) or -x (v). Absent: both automatic. */
  faces?: [number, number];
  /** 1: half wall. */
  form?: number;
}

/**
 * Diagonal wall directions across tile (x, z): `dp` ("/") from (x, z) to (x + 1, z + 1),
 * `dn` ("\") from (x, z + 1) to (x + 1, z).
 */
export type DiagonalAxis = 'dp' | 'dn';

/** A wall edge (or a diagonal across a tile) to set: a plain wall, a door or window in a wall, or nothing (`open`). */
export interface EdgeEdit {
  axis: WallEdge['axis'] | DiagonalAxis;
  x: number;
  z: number;
  kind: 'wall' | 'door' | 'window' | 'open' | 'fence' | 'gate';
  /** New walls: covering of both faces. */
  cover?: number;
  /** Walls: 0 full height, 1 half wall. */
  form?: number;
  /** Doors and windows: style index; fences and gates: fence style index. */
  style?: number;
  /** Windows: sill; doors and windows: top (centimetres above the floor, `MAX_HEAD` at most). Absent keeps a standing one's. */
  sill?: number;
  head?: number;
}

/** Doors and windows (sim-core `MAX_HEAD`, `MIN_DOOR_HEAD`, `MIN_WINDOW`): the highest top, a door's lowest top, a window's least height (centimetres). */
export const MAX_HEAD = 270;
export const MIN_DOOR_HEAD = 180;
export const MIN_WINDOW = 30;

/** A fence or gate on a grid edge (as `WallEdge`); not a wall: it makes no rooms. */
export interface FenceEdge {
  axis: 'h' | 'v';
  x: number;
  z: number;
  kind: 'fence' | 'gate';
  /** Index into the content's fence styles. */
  style?: number;
}

/** One wall face to cover (`side` 0 looks towards -z / -x / the tile's half 0). */
export interface FacePaint {
  axis: EdgeEdit['axis'];
  x: number;
  z: number;
  side: 0 | 1;
  covering: number;
}

/** A diagonal wall across tile (x, z). `rooms`: rooms of the tile's halves (0 touches its -z side, 1 its +z side). */
export interface DiagonalWall {
  axis: DiagonalAxis;
  x: number;
  z: number;
  kind: 'wall' | 'door' | 'window';
  rooms: [number, number];
  /** Covering of each face (face 0 towards half 0), half wall (`form` 1), door or window style. */
  faces?: [number, number];
  form?: number;
  style?: number;
}

/** A door or window in a wall (its edge is also listed in `WorldStructure.walls`). */
export interface Opening extends WallEdge {
  kind: 'door' | 'window';
  /** Door or window style index. */
  style?: number;
  /** Resized by the player: the sill (windows) and the top, centimetres above the floor; absent: the style's. */
  sill?: number;
  head?: number;
}

export interface MeshArrays {
  positions: Float32Array;
  normals: Float32Array;
  uvs: Float32Array;
  indices: Uint32Array;
}

/**
 * Food and dishes (`sim-core/src/items.rs`, `view::items_json`): sent on their own (`items`
 * message) whenever they change, which is far more often than the furniture.
 */
export interface ItemPlacement {
  /** Stable for the item's life. */
  id: number;
  /** Content `items` id; its `model` is drawn. */
  def: string;
  /** Where it stands, in tiles (`z` on the lot's rows, like objects). */
  x: number;
  z: number;
  /** Height above its storey's floor (m): the top it stands on. */
  y: number;
  /** Turned like what it stands on (radians). */
  yaw: number;
  servings: number;
  /** `stored` items are in a fridge (not drawn). */
  state: 'fresh' | 'spoiled' | 'dirty' | 'stored';
  household: number;
}

export interface ObjectPlacement {
  id: number;
  def: string;
  x: number;
  z: number;
  rot: number;
  /** Degrees past `rot`, for objects that turn freely (0..89). */
  turn?: number;
  /** Centimetres (x, z) its middle stands off the middle of its footprint. */
  offset?: [number, number];
  /**
   * Resized by the player: its own size, centimetres across and deep before turning (it stands
   * in the middle of its tiles `w` × `d`, moved by `offset`); absent: its kind's footprint.
   */
  size?: [number, number];
  /** Centimetres taller (negative: shorter) than its kind. */
  tall?: number;
  /** Things on the `surface` layer: the height (m) of the top they stand on. */
  lift?: number;
  /** Rotated footprint in tiles (its own: resized, it differs from its kind's). */
  w: number;
  d: number;
  /** Upgrade level (0 = as bought). */
  quality: number;
  /** The colours the player chose for its parts; absent: as made. */
  colors?: PartColors;
  /** What selling returns; null if it can't be sold. */
  sellValue: number | null;
  /** Wear 0..1 (broken at 1). */
  wear?: number;
  /** Open (a wall bed folded down): set by interactions that open or close it (`leaves`). */
  open?: boolean;
  /** A crop's stage (content `grows`): 0 planted, 1 growing, 2 ripe, 3 picked; absent for other things. */
  growth?: number;
  /** A machine running on its own (content `runs`: the washer's drum turning, its sound); sim-core `ObjectInstance::run`. */
  running?: boolean;
  /** What the quick fix costs (worn things). */
  repairCost?: number;
}

export interface SimInfo {
  id: number;
  name: string;
  household: number;
  gender: string;
  attractedTo: string[];
  appearance: Appearance;
  traits: string[];
  perks: string[];
  /** Died or moved away: the slot (and its snapshot row) stays, but they're not listed or drawn. */
  gone?: { why: 'died' | 'movedAway'; day: number };
  /** Bumped when someone new takes the slot. */
  generation?: number;
  /** Life stage id (how they look: elders' hair greys). */
  stage?: string;
  /** The outfit they changed into at a wardrobe (absent: 0, the creator's; characters/outfit.ts `spareLook`). */
  outfit?: number;
}

/** Someone whose slot a newcomer took; story events name them with `FORMER | index`. */
export interface FormerResident {
  name: string;
  household: string;
}

/** Event references with this bit name a former resident (sim-core `social::FORMER`). */
export const FORMER = 2 ** 31;

/** What is at the end of a road: the way out of town, a dead end (a turning circle), another road it joins, or itself (a ring). */
export type RoadEnd = 'out' | 'dead' | 'join' | 'loop';

/** A town's road (`game/roads.ts`): its middle, a point about every metre (town tiles); its width with the sidewalks. */
export interface RoadPlan {
  points: [number, number][];
  width: number;
  ends: [RoadEnd, RoadEnd];
  name?: string;
  /** The side its lots are on, if only one (+1: its left, looking along it; -1: its right): the other looks out over the water. */
  lots?: 1 | -1;
  /** What stands in the middle of a ring this small (a roundabout). */
  island?: 'fountain';
  /** What kind of way it is (default `street`). */
  kind?: RoadKind;
  /** The area (`OpenWorldTownFields.areas` index) it belongs to; absent for ways between areas. */
  area?: number;
  /** A bridge or a causeway: the arc lengths (m) along it where its deck over the water begins and ends. */
  bridge?: [number, number];
  /**
   * Bridges the land's brushes made where water came across it (Build mode's Land tab: `game/landBridges.ts`), each
   * as `bridge`, along the road as it is (not made straight nor split); the simulation keeps them
   * (`Command::ShapeLand`'s `meta.roads`). A road's decks are both (`render/shared/roads.ts` `bridgeSpans`).
   */
  bridges?: [number, number][];
}

/** `street` (default) · `lane` · `country` (between areas) · `trail` (2 m, on foot only). */
export type RoadKind = 'street' | 'lane' | 'country' | 'trail';

/** How big a world's main town is: a hamlet, a village, a town or a city. */
export type TownSize = 'hamlet' | 'village' | 'town' | 'city';

/**
 * How much land a new world has, both ways (`game/world.ts` `LAND_SIZE`): small, medium or large.
 * The bigger towns and more villages need the bigger lands (`townFits`).
 */
export type LandSize = 'small' | 'medium' | 'large';

/**
 * What the player chose for a world besides its place (`game/world.ts` `WorldKind`): how big its
 * main town is, how many villages round it, how hilly and how wooded its land, and (`size`) how
 * much land it has (none: the place's own, or for another town and villages the smallest land they
 * fit on: `world.ts` `optionsFor`, `sizeFor`).
 */
export interface WorldOptions {
  town: TownSize;
  villages: 'none' | 'few' | 'many';
  hills: 'flat' | 'rolling' | 'hilly' | 'mountains';
  woods: 'open' | 'some' | 'wooded';
  size?: LandSize;
}

/** How big a world is: every world is a Big one (cosy and normal were dropped in 0.8.0; a save that still says so reads as `big`). */
export type WorldSize = 'big';

/** Run-length mask over the ground grid, row-major from tile (0, 0): `[run, value(0|1), run, value, ...]`. */
export type Rle = number[];

/** A hamlet (area 0: the main town) or a stretch of countryside. */
export interface TownArea {
  name: string;
  kind: 'hamlet' | 'countryside';
  x: number;
  z: number;
  w: number;
  d: number;
}

/** Where people leave town: a road out (`[x, z]`, or typed `kind: 'road'`). */
export type TownExit = [number, number] | { x: number; z: number; kind: 'road' };

/** Land the renderer draws and the generator keeps building off. */
export interface WorldZone {
  /** What the land is (an orchard: fruit trees in rows beside a farmstead). */
  kind: 'woods' | 'meadow' | 'water' | 'hill' | 'fields' | 'orchard';
  poly: [number, number][];
}

/** The town file's open-world fields (all optional; absent: today's behaviour). */
export interface OpenWorldTownFields {
  size?: WorldSize;
  /** The roads for the sim (the same objects as `meta.roads`). */
  roads?: RoadPlan[];
  areas?: TownArea[];
  exits?: TownExit[];
  terrain?: { water?: Rle; blocked?: Rle };
  /** Public parking: marked bays, each where a car parked in it stands (the simulation parks at exactly this pose). */
  parking?: ParkingBay[];
  /** The car parks the bays are grouped into (`ParkingBay.park`). */
  carParks?: CarPark[];
}

export interface WorldMeta {
  kind?: string;
  name?: string;
  /** Older towns' streets: straight, along x (newer ones have `roads`). */
  streets?: { x: number; z: number; w: number; d: number }[];
  /** The town's roads (missing in older saves: their `streets`). */
  roads?: RoadPlan[];
  /** Village greens and the like: grass the roads go round (polygons, town tiles). */
  greens?: [number, number][][];
  paths?: { x: number; z: number; w: number; d: number }[];
  /** Where each plot's street is (by plot index; `game/town.ts` `Facing`): 0 north, 1 west, 2 south, 3 east. Missing in older saves. */
  facing?: number[];
  seed?: number;
  /** What kind of world the town is in and how its land looks (missing: the classic valley). */
  world?: WorldLook;
  /** The menus' town preview only: the plot drawn as the player's house, as the game shows it (its car on the driveway, no garage or garden dressing). */
  home?: number | null;
  /**
   * The town put its garden fences, hedges and farm field fences up as real fences (`fences`, see
   * `game/gardenFences.ts`): the street draws none of its own. Missing in older towns and saves,
   * whose street still dresses neighbours' gardens and fields with fences of its own.
   */
  gardenFences?: boolean;
  /** Big worlds: woods, meadows, water, hills and fields across the world. */
  zones?: WorldZone[];
  /** Big worlds: the height (m) each outdoor plot off the flat town land is levelled to, by plot index. */
  pads?: Record<number, number>;
  /** The areas (hamlets) as the generator laid them out, for the map and the camera. */
  areas?: TownArea[];
  // pin:living-town-meta begin
  /** Living town: the places the town tags name (each a lot's). */
  places?: TownPlace[];
  /** Living town: each main town's centre, the square and the high street (`poly`, paved but for its lots and greens), and the square itself. */
  centre?: { area: number; poly: [number, number][]; square?: [number, number][] }[];
  /** Real town: paved ground the town lays besides its roads, paths and car parks (courts, yards), the one source near and far draw. */
  paving?: PavedPiece[];
  /** Living town: how the lots of the town's named places and flats buildings look from outside, by plot index (`PlotSlot.dress`). */
  lotLooks?: Record<number, LotLook>;
  // pin:living-town-meta end
  /**
   * Big worlds: how high the land stands over the level the towns are built at, a grid of `cell` m
   * squares (`cols` × `rows`, row-major from the world's corner), in decimetres (0 on the water).
   * Missing in older saves: the land is drawn from the landscape alone.
   */
  relief?: { cell: number; cols: number; rows: number; h: number[] };
  /**
   * The land as a height field (`game/land.ts`), once there is one: shaped by the player, or
   * surveyed for a new world. Missing: the land is drawn from `relief` and the landscape.
   */
  land?: LandFile;
}

/**
 * The land as the town file and saves carry it (`game/land.ts` `encodeLand`): heights and water
 * levels every `cell` m over `cols` × `rows` samples from the world's corner, cut into tiles of
 * `tile` × `tile` samples (row-major, the last row and column of tiles cut short at the edge).
 */
export interface LandFile {
  v: 1;
  cell: 2;
  cols: number;
  rows: number;
  tile: 64;
  /**
   * The heights (cm) by tile: each the base64 of zigzag varints of every sample less the one left
   * of it (the first of a row less 0), a run of zeros as a 0x00 then the run's length.
   */
  h: string[];
  /** The water levels (cm; `NONE` where dry) by tile, coded as `h`; '' for a tile with no water. */
  wl: string[];
  /**
   * Which way a drawn river runs (`Land.flow`: 0 still, else an angle) by tile, coded as `h`; '' for a tile with
   * none. Missing on a land no river was drawn on.
   */
  flow?: string[];
  /** The cover every 4 m (row-major), run-length `[run, value, ...]`: 0 open, 1 woods, 2 meadow, 3 fields. */
  cover: number[];
  /** How the field was first filled: `drawn` from the land as the renderer drew it, `follow` with the town on its slopes. */
  seeded: 'drawn' | 'follow';
}

// pin:living-town-types begin

/** What a building is, for how it looks closed (a lot's `LotLook`). */
export type BuildingUse =
  | 'flats'
  | 'terrace'
  | 'house'
  | 'shop'
  | 'cafe'
  | 'restaurant'
  | 'office'
  | 'school'
  | 'church'
  | 'hall'
  | 'workshop'
  | 'farm';

/** Somewhere the town tags name: a public lot. */
export interface TownPlace {
  name: string;
  kind: 'shop' | 'cafe' | 'restaurant' | 'office' | 'school' | 'church' | 'park' | 'outdoor' | 'workshop' | 'farm';
  /** Where its tag stands (town metres). */
  x: number;
  z: number;
  /** Its lot: the plot index. */
  plot: number;
  /** Opening hours: minutes of the day [open, close) and the days (bit 0 = Monday). Absent: always open. */
  hours?: { open: number; close: number; days: number };
  /** Career categories (`JobView.category`) whose workers are shown as working here. */
  work?: string[];
  area: number;
}

/**
 * How a named place's or a building of flats' lot (a town hall, a school, a shop on the high street:
 * `game/placeLots.ts`) looks from outside, closed: its use, its storeys (3 m each, as many as it really
 * has, at most the town's), its roof, and its wall and roof colour pair (an index into the renderer's
 * palette, taken modulo its length).
 */
export interface LotLook {
  use: BuildingUse;
  storeys: number;
  roof: 'gable' | 'hip' | 'flat' | 'mansard';
  look: number;
}

/**
 * A marked parking bay (`OpenWorldTownFields.parking`, `WorldStructure.parking`): its middle (m),
 * the way a car parked in it faces (rad, as the simulation's yaw: nose in, away from the road), its
 * area (`TownArea` index) and its car park (`carParks` index; absent: a bay on its own by the verge).
 */
export interface ParkingBay {
  x: number;
  z: number;
  yaw: number;
  area: number;
  park?: number;
}

/**
 * A car park: a row of nose-in bays off a straight stretch of street on one apron. Its asphalt (4
 * corners, counter-clockwise seen from above), where its P sign stands (x, z and the way it faces,
 * rad), the plot of the place it is by (`serves`), or that it is the square's (`square`).
 */
export interface CarPark {
  area: number;
  poly: [number, number][];
  sign: [number, number, number];
  serves?: number;
  square?: true;
}

/**
 * Paved ground the town lays besides its roads and paths (`WorldMeta.paving`): a centre's, a
 * square, the open ground of a place's or a flats building's lot (`court`), a workshop's yard. A
 * polygon in town metres, counter-clockwise seen from above; `plot`: the lot it is part of.
 */
export interface PavedPiece {
  kind: 'centre' | 'square' | 'court' | 'yard';
  poly: [number, number][];
  plot?: number;
}

/** What a lot is (`PlotInfo.kind`, the town file's `plots[].kind`; a flat in the file is `flat`). */
export type PlotKind = 'home' | 'farm' | 'public' | 'shop' | 'countryside' | 'flats';

/**
 * A flat (`WorldStructure.flats`): a plot of its own inside a building's lot (`of`: a flats lot,
 * or a shop's, café's or restaurant's with flats over it), on one storey (`storey`). Its id follows
 * the last ground lot's (at or past `plots.length`); its rectangle (`x`, `z`, `w`, `d`) is on the
 * ground's rows, as a lot's; `door` is the landing tile before its own door, on its storey's rows.
 */
export interface FlatInfo {
  id: number;
  of: number;
  storey: number;
  x: number;
  z: number;
  w: number;
  d: number;
  name: string;
  door: [number, number];
}
// pin:living-town-types end

/** The world a town stands in (`game/world.ts`): its kind, its land, and farm fields in town. */
export interface WorldLook {
  kind: string;
  /** How big the world is (always `big` now). */
  size?: WorldSize;
  landscape: LandscapeParams;
  /** What the player chose for the world besides its place (missing in older saves). */
  options?: WorldOptions;
  /** Tilled fields on farm lots (town tiles): drawn as soil, fenced in. */
  fields?: { x: number; z: number; w: number; d: number }[];
  /** The seed its town was laid from, where it differs from the land's (`game/worldCode.ts`). */
  townSeed?: number;
  /** The seed its people were drawn from (`game/people.ts` `peopleStreams`). */
  peopleSeed?: number;
  /** Its land was shaped by hand (step 1's brushes): the world code no longer makes it. */
  shaped?: true;
  /** Its town was edited by hand (step 2's tools): the world code no longer makes it. */
  edited?: true;
}

/**
 * How the land round a town looks (see `render/shared/nature.ts`). `DEFAULT_LANDSCAPE` is the
 * classic valley every town had before worlds had kinds.
 */
export interface LandscapeParams {
  /** Height of the hills (m) and how far they take to rise from the town's edge (m). */
  hills: number;
  rise: number;
  /** The valley's rim: how high it climbs (m) and how far from the town it starts (m). */
  rim: number;
  rimFrom: number;
  /** More (above 0) or fewer (below) trees in the woods, and how close they come (m past the flat apron). */
  forest: number;
  treeline: number;
  /** Conifers and birches among the trees, 0..1 (0.5: the classic mix). */
  conifers: number;
  birches: number;
  /** Grass: lush, lawn, meadow and hillside green (hex). */
  grass: [string, string, string, string];
  /** The sea past one side of the town (z side, its shore this far out), or a lake beside it (town metres). */
  water?: { kind: 'sea'; side: 'zmin' | 'zmax'; shore: number } | { kind: 'lake'; x: number; z: number; rx: number; rz: number };
  /** Farmland round the town: a patchwork of fields (cell size m, grid turned by `angle` rad). */
  fields?: { size: number; angle: number };
  /**
   * A range of mountains round the valley, its peaks this high (m): ridges and pinnacles with
   * bare rock above the trees and snow on the tops. Lower on the low-z side (the menus look at
   * the town from there) and parted by a pass where the road leaves town.
   */
  mountains?: number;
  /** The steepest the land rises from anything built (rise per m); missing: as steep as the hills make it. */
  slope?: number;
  /** Trees out on open land, times the usual (default 1). */
  scatter?: number;
  /** The share of the trees that are bushes (default: the usual share). */
  scrub?: number;
  /** How far the sand reaches from the shoreline (m; default 8). */
  beach?: number;
}

export const DEFAULT_LANDSCAPE: LandscapeParams = {
  hills: 14,
  rise: 45,
  rim: 30,
  rimFrom: 60,
  forest: 0,
  treeline: 2,
  conifers: 0.5,
  birches: 0.5,
  grass: ['#6F9A3E', '#86A84A', '#A2AE5A', '#638A3E'],
};

export interface HouseholdInfo {
  id: number;
  name: string;
  /** Its home: a lot, or a flat (at or past `plots.length`: `flatOf`, `lotOf`). */
  plot: number | null;
  player: boolean;
}

export interface PlotInfo {
  id: number;
  name: string;
  x: number;
  z: number;
  w: number;
  d: number;
  public: boolean;
  entry: [number, number] | null;
  /** Wall bounding box `[x0, z0, x1, z1]`, for house silhouettes. */
  house: [number, number, number, number] | null;
  /** The roof the player chose: `[style, colour]` (indices into the content's roof styles and colours). */
  roof?: [number, number];
  /** Storeys the house on it has (absent: one). */
  storeys?: number;
  /**
   * The hours the place on it keeps (sim-core `world::PlotHours`, `Plot::open_at`): minutes of the day [open, close)
   * and the days (bit 0 = Monday), a `close` at or before `open` running past midnight. Absent: always open.
   */
  hours?: { open: number; close: number; days: number };
  /** What the lot is (the town file's `plots[].kind`). Absent: a town file without kinds. */
  kind?: PlotKind;
}

/** What `lotOf` and `flatOf` read: the ground lots and the flats. */
type LotsAndFlats = { plots: readonly PlotInfo[]; flats?: readonly FlatInfo[] };

/** The flat plot id `plot` is (a household's or a resident's may be one), or undefined for a ground lot. */
export function flatOf(world: LotsAndFlats, plot: number): FlatInfo | undefined {
  if (plot < world.plots.length) return undefined;
  const flats = world.flats ?? [];
  const at = flats[plot - world.plots.length];
  return at?.id === plot ? at : flats.find((f) => f.id === plot);
}

/** The ground lot plot id `plot` stands on: a flat's building, else the lot itself. */
export function lotOf(world: LotsAndFlats, plot: number): number {
  return flatOf(world, plot)?.of ?? plot;
}

/** Sent when walls or objects change (rare). */
/** Living: the residents earn the money for building. Creative: building is free. */
export type GameKind = 'living' | 'creative';

/**
 * A `world` message's structure (sim-core `view::StructureSync`): what changed since the last
 * message, the first time everything. The parts it has replace the last ones, those `unset` names
 * are gone (the last garage sold). The things: `objectSplice` (`[start, deleted, things]`, as
 * `Array.splice`) the ones bought, sold or set up, then `objectChanges` the others that changed, by
 * their index in the list after the splice (a door opened, a crop grown). The lot's walls,
 * openings, diagonals, fences and floors come only when the lot changed, and only those that did;
 * its rooms (when they changed) and meshes come beside it (typed arrays, transferred).
 */
export type WorldUpdate = Partial<Omit<WorldStructure, 'rooms' | 'meshes' | 'staffSims'>> & {
  version: number;
  lotVersion: number;
  objectSplice?: [number, number, ObjectPlacement[]];
  objectChanges?: [number, ObjectPlacement][];
  unset?: (keyof WorldStructure)[];
};

export interface WorldStructure {
  version: number;
  /** Changes whenever the lot (walls, floors, fences, coverings) does; not with furniture. */
  lotVersion: number;
  mode: GameKind;
  width: number;
  /** Rows of the lot, every storey included (see `storeyDepth`). */
  depth: number;
  /**
   * Storeys (sim-core `storeys.rs`): storey `k` is rows `k * storeyDepth ..` of the lot, laid out
   * like the ground (`storeys`: the ground and those above it). Absent in older structures: one
   * storey. Use `storeyOfRow` and `storeyRow` rather than multiplying: the basement has rows too.
   */
  storeys?: number;
  storeyDepth?: number;
  /**
   * The block of `storeyDepth` rows holding the basement (storey -1); blocks after it hold the
   * storeys above `basement - 1` (one up each). Absent: no basement.
   */
  basement?: number;
  objects: ObjectPlacement[];
  /** Every slot, gone residents included (ids index this list); `roster` in the UI leaves them out. */
  sims: SimInfo[];
  former?: FormerResident[];
  households: HouseholdInfo[];
  /** The ground lots, in slot order (ids index this list; `meta.facing`, `pads` and `lotLooks` too). Flats are in `flats`. */
  plots: PlotInfo[];
  exits: [number, number][];
  /** The room of each tile, row by row (`width × depth`; 0 outdoors): a `Uint16Array` from the simulation. */
  rooms: Uint16Array | number[];
  /** Every edge with a wall on it (plain, or with a door or window), in grid order. */
  walls: WallEdge[];
  /** Doors and windows. */
  openings: Opening[];
  /** Diagonal walls (absent from older structures). */
  diagonals?: DiagonalWall[];
  /** Fences and gates (absent from older structures). */
  fences?: FenceEdge[];
  /** Floor coverings: `[x, z, covering]` per tile that has one (absent: automatic floors). */
  floors?: [number, number, number][];
  /** Presentation data carried by the simulation unchanged (see `game/town.ts`). */
  meta: WorldMeta | null;
  meshes: { walls: MeshArrays; wallsLow: MeshArrays; floors: MeshArrays };
  /**
   * The town's vendors and shopkeepers (sim-core `staff.rs`), in their snapshot rows' order (after the
   * residents'): what each looks like (`seed`) and the posts they serve at (stable object ids). Absent: none.
   */
  staff?: StaffInfo[];
  /** The servers as residents for the renderer (rows `sims.length + k`), given their looks by the session (`game/staff.ts`). */
  staffSims?: SimInfo[];
  /**
   * Households' cars (sim-core `cars.rs`), in their snapshot rows' order (`SnapshotLayout.car`): where each
   * parks. Absent: a game from before cars (the street draws some by the neighbours' driveways, parked).
   */
  cars?: CarInfo[];
  /** Garages at the inner ends of driveways (sim-core `cars::Garage`: wall centre lines, by plot). Absent: a game from before them (the street places neighbours' own). */
  garages?: GarageInfo[];
  /** Porches over doors (sim-core `porches::Porch`): the town's over its front doors, and the ones the player built. Absent: none. */
  porches?: PorchInfo[];
  /** Porch pieces standing on their own (sim-core `porches::PorchPiece`): posts, canopies and step platforms the player built. Absent: none. */
  porchPieces?: PorchPieceInfo[];
  /**
   * The lanes where the kerbside furniture keeps back, for cars to pull over onto the pavement to
   * pass: every lane too narrow for two cars to pass in their lanes (a 6 m lane: sim-core
   * `cars::pull_over_streets`). Absent: none (the furniture stands where it always did).
   */
  pullOverStreets?: PullOverStreetInfo[];
  /**
   * Where the traffic lights' poles stand on the pavement and which way their heads face (x, z, yaw: sim-core `traffic::SignalArm`, in the snapshot's signal
   * head rows' order): the kerbside furniture keeps clear of them. Absent: none.
   */
  lightPoles?: [number, number, number][];
  /**
   * Every junction where three or more roads for cars meet (sim-core `traffic::Signal`), for its
   * pad and markings: the lit ones first, in the snapshot's signal head rows' order, then those
   * giving way (no heads). Absent: none.
   */
  junctions?: JunctionInfo[];
  /**
   * Public parking (sim-core `travel::Parking`): every marked bay, where a car parked in it stands
   * and faces (the simulation parks at exactly this pose), in the town file's order. Absent: none.
   */
  parking?: ParkingBay[];
  /** The car parks the bays are grouped into (`ParkingBay.park`), as the town file has them. Absent: none. */
  carParks?: CarPark[];
  /** The flats (each a plot of its own, ids after the last of `plots`), in id order. Absent: none. */
  flats?: FlatInfo[];
}

/**
 * A junction (`WorldStructure.junctions`, sim-core `traffic::Signal`): its node's middle (m), and
 * its arms. `roundabout`: a node on a roundabout's ring (the renderer leaves out its pad and
 * markings; its lights work as before, an arm at a time).
 */
export interface JunctionInfo {
  x: number;
  z: number;
  roundabout?: true;
  arms: JunctionArmInfo[];
}

/**
 * A junction's arm (sim-core `traffic::SignalArm`). Distances are m out from the junction's middle
 * along `dir`: the crosswalk from `walkFrom` (the corner, where the kerb's return round from the
 * next arm ends, about `traffic::KERB_RETURN` = 3 m in radius, or the box's edge) to `walkTo`
 * (`traffic::CROSSWALK` = 3 m on), and the stop line at `stop` (`traffic::STOP_GAP` = 0.5 m past
 * it): cars on red stop with their noses there, and at a zebra for those on foot on it.
 */
export interface JunctionArmInfo {
  /** The arm's road (`meta.roads` index). */
  road: number;
  /** The way out along the arm from the middle (a unit vector: x, z). */
  dir: [number, number];
  /** The road's carriageway, kerb to kerb (m). */
  width: number;
  walkFrom: number;
  walkTo: number;
  stop: number;
  /**
   * Whether its crosswalk is there, a zebra (sim-core `SignalArm::crosswalk`): at a lit junction
   * on a street's or a lane's arm, at one giving way on one of a street's two arms through it.
   * Absent (an older simulation): on any arm with sidewalks.
   */
  crosswalk?: boolean;
  /** Its signal head's row in the snapshot (`SnapshotLayout.light`); null: not lit. */
  light: number | null;
}

/** A junction, as the renderer's pads and markings take it (`render/shared/junctions.ts`). */
export type Junction = JunctionInfo;

/**
 * A lane cars pull over on (`WorldStructure.pullOverStreets`): the town's road (`meta.roads`
 * index) and how far in from its edges (m, either side) its kerbside furniture may reach (sim-core
 * `cars::kerbside`, `cars::LANE_KERBSIDE`): a car passing another pulled over comes no nearer.
 */
export interface PullOverStreetInfo {
  road: number;
  keep: number;
}

/**
 * A garage (`WorldStructure.garages`): its walls' centre lines, and for one the player built (Build
 * mode's Garage tool) the way its door faces (`rot`, as a thing's: 0 towards +z, 1 +x, 2 -z, 3 -x).
 * No `rot`: its door faces the street, as the town placed it.
 */
export interface GarageInfo {
  plot: number;
  x0: number;
  z0: number;
  x1: number;
  z1: number;
  rot?: number;
}

/**
 * A porch over a door (`WorldStructure.porches`): the door's edge on the ground storey (`h` along x
 * on grid line z, `v` along z on grid line x) and the side it stands on (`out`: 1 towards larger z
 * or x, -1 the other way). Landing, posts, a gabled canopy and a lamp (`HouseBuilder.porch`).
 */
export interface PorchInfo {
  plot: number;
  axis: 'h' | 'v';
  x: number;
  z: number;
  out: number;
}

/** What a porch piece is: a post (it blocks its tile), a gabled canopy, a step platform. */
export type PorchPieceKind = 'post' | 'canopy' | 'step';

/** A porch piece (`WorldStructure.porchPieces`): the tiles it covers, `x .. x + w` × `z .. z + d` (as turned), and the way it faces (`rot`, as a thing's: 0 towards +z, 1 +x, 2 -z, 3 -x). */
export interface PorchPieceInfo {
  plot: number;
  kind: PorchPieceKind;
  x: number;
  z: number;
  w: number;
  d: number;
  rot: number;
}

/** A household's car (`WorldStructure.cars`): where it parks (its middle, which way it faces: 0 towards +z) and its look (`shared/cars.ts`: its size is its body's). */
export interface CarInfo {
  plot: number;
  x: number;
  z: number;
  yaw: number;
  body: number;
  paint: number;
  /** How old it is (`shared/cars.ts` `CarAge`: 0 old, 1 used, 2 new; absent: used). */
  age?: number;
}

/** One of the town's servers (`WorldStructure.staff`). */
export interface StaffInfo {
  seed: number;
  posts: number[];
}

export interface ActionView {
  label: string;
  /** The object it's done at (its stable id). */
  object: number | null;
  /** The other Sim, for social actions. */
  target: number | null;
  progress: number;
  directed: boolean;
  active: boolean;
}

/** How fast residents age: off, or 1, 2 or 4 game days a year (short, normal, long). */
export type Lifespan = 'off' | 'short' | 'normal' | 'long';

/**
 * A resident as the HUD sees them. Everyone gets the summary (who, where, which job); the
 * details (from `grade` on: needs, feelings, actions, skills, plan) are kept up to date only
 * while `detail` is set: for the player's household and the resident being inspected. Others
 * have empty details.
 */
export interface SimView {
  id: number;
  detail: boolean;
  name: string;
  household: number;
  traits: string[];
  perks: string[];
  /** Age in whole years and the life stage id (absent from older workers). */
  age?: number;
  stage?: string | null;
  /** Retired: the weekly pension. */
  pension?: number;
  /** Plot the Sim is on (null on the street or at work): a lot, or a flat (`flatOf`, `lotOf`). */
  plot: number | null;
  /** Minute of day they come back from work, if away. */
  awayUntil: number | null;
  visiting: number | null;
  /**
   * On their way somewhere (a visit, home, home from work by car): the plot, and about when they're
   * there (minute of day, in 5-minute steps). `at` only with `detail` (it takes a search along the
   * roads), or home from work by car. Absent when they aren't (and from older workers).
   */
  trip?: { to: number; at?: number };
  /**
   * On a bike or an e-scooter of theirs (sim-core `ride.rs`), off to work, to school, out (a visit
   * or an outing: where to is the `trip`) or home on it (not while away at work with it). Absent
   * when they aren't (and from older workers).
   */
  ride?: { kind: 'bike' | 'scooter'; to: 'work' | 'school' | 'out' | 'home' };
  /** The work taxi (sim-core `taxi::status`): waiting for it at the kerb, in it to work or school (`at`: the minute of the day they're there), or home in it (or waiting at work for it). Absent: none of those. */
  taxi?: { state: 'wait' | 'work' | 'home'; at?: number };
  job: JobView | null;
  /** A pupil's school grade (0..100, in tenths). */
  grade?: number;
  /** 0..1 in 1 % steps (the bars' resolution), as is `mood`. */
  needs: number[];
  mood: number;
  emotion: string | null;
  /** `minutesLeft` in whole minutes. */
  feelings: { id: string; label: string; mood: number; minutesLeft: number }[];
  /** `progress` in 1 % steps. */
  actions: ActionView[];
  /** Skill levels in content order (whole number = level; in 1 % steps, rounded down). */
  skills: number[];
  /** Their phone (with content `phone`): why it's out of use, and who they can reach. */
  phone?: PhoneView;
  /** Why they can't be sent out (visiting) right now ("Nobody to mind the baby"), as the sim would refuse it. */
  homebound?: string;
  /** Routines, goals and wishes (the player's household only). */
  plan?: PlanView;
}

/** A resident's phone (`phone.rs`). */
export interface PhoneView {
  /** Why they can't use it now ("Too young for a phone", "At work"), or null. */
  off: string | null;
  /** Everyone they've met who has a phone too. */
  contacts: ContactView[];
}

/** Someone in a resident's phone. */
export interface ContactView {
  id: number;
  /** What keeps them from the phone ("Asleep", "At work", "Busy"...), or null when free. */
  status: string | null;
  /** Per phone action (content order): why it's out with them right now (the sim would refuse the order), or null. */
  why: (string | null)[];
}

/**
 * A resident's `id` and the fields that changed. New residents come with every field; optional
 * fields that went away are `null`; `plan` carries only its parts that changed. `detail: false`
 * drops the details.
 */
export type SimPatch = { id: number } & { [K in Exclude<keyof SimView, 'id' | 'plan'>]?: SimView[K] | null } & { plan?: Partial<PlanView> | null };

export interface Requirement {
  skill: string;
  level: number;
  /** The Sim's current whole level. */
  have: number;
}

export interface JobView {
  /** Index into the career catalog. */
  index: number;
  career: string;
  careerLabel: string;
  category: string | null;
  title: string;
  level: number;
  levels: number;
  grade: string | null;
  gradeLabel: string | null;
  performance: number;
  /** What a shift adds to the bar at this pace (the mood they last left for work in). */
  shiftPerformance: number;
  /** Shifts at this pace still to fill the bar; null at the top, or when shifts don't fill it. */
  shiftsToFullBar: number | null;
  /** The mood they last left for work in (0..1). */
  shiftMood: number;
  /** Per shift on this Sim's schedule. */
  pay: number;
  payPerHour: number;
  weeklyPay: number;
  startHour: number;
  hours: number;
  /** This Sim's workdays (0 = Monday). */
  days: number[];
  standardDays: number[];
  workweek: string | null;
  /** Skill levels above (+) or below (-) the requirements. */
  fit: number;
  requires: Requirement[];
  next: { title: string; grade: string | null; payPerHour: number; requires: Requirement[] } | null;
  nextTitle: string | null;
}

/** Every career level and the build/buy rules, sent once per game. */
export interface Catalog {
  grades: { id: string; label: string; payPerHour: number }[];
  categories: string[];
  careers: CareerEntry[];
  workweek: { fit: number; days: number; label: string }[];
  probationLevels: number;
  maxSkill: number;
  objectRules: {
    maxQuality: number;
    qualityBonus: number;
    upgradeCost: number;
    upgradeMinutes: number;
    upgradeSkill: string | null;
    upgradeSkillPerLevel: number;
    resale: number;
  };
  build: {
    wall: number;
    door: number;
    window: number;
    remove: number;
    diagonalWall?: number;
    fence?: number;
    gate?: number;
    garage?: number;
    porch?: number;
    porchPost?: number;
    porchCanopy?: number;
    porchStep?: number;
    /** Shaping the land (Build mode's Land tab; content `build.land`). */
    land?: LandPrices;
  };
}

/**
 * What shaping the land costs (content `build.land`): `earth` a cubic metre raised or lowered by
 * the brushes, `mountain` one by the peak, range and level tools (volume: Σ|Δh| × 4 m² over the
 * 2 m samples); the rest a square metre, of water made and of cover planted or cleared.
 */
export interface LandPrices {
  earth: number;
  mountain: number;
  water: number;
  woods: number;
  meadow: number;
  fields: number;
  clear: number;
}

export interface CareerEntry {
  id: string;
  label: string;
  category: string | null;
  skills: string[];
  levels: {
    title: string;
    grade: string | null;
    /** Per shift on the standard week. */
    pay: number;
    start: number;
    hours: number;
    days: number[];
    /** `[skill id, level]`. */
    requires: [string, number][];
  }[];
}

/** A room (or the garden) of the player's home, with its scores (sim-core `rooms.rs`). */
export interface RoomView {
  /** Lot room id (0 for the garden); changes when walls do. */
  id: number;
  garden: boolean;
  /** Index into the content's room kinds; absent: nothing tells what it's for. */
  kind?: number;
  /** Things of two exclusive kinds stand in it (a bed in the kitchen). */
  mixed: boolean;
  /** The essential of its kind nothing in it offers (index into the kind's `essentials`). */
  missing?: number;
  tiles: number;
  windows: number;
  doors: number;
  lamps: number;
  /** Mean dirt, 0..1 in 1 % steps (as are the scores). */
  dirt: number;
  centre: [number, number];
  /** Size, light, decor, cleanliness, function, overall (0..1). */
  scores: [number, number, number, number, number, number];
}

/** A household's money and home. `undo`: build and buy edits the household can take back. */
export interface FundsView {
  id: number;
  funds: number;
  rent: number | null;
  bills: number | null;
  undo?: number;
  redo?: number;
  /** The routine template (the player's household only). */
  routines?: Routine[];
  /** A baby on the way: the parents and the game day it's due. */
  expecting?: { parents: [number, number]; due: number };
}

/**
 * The HUD's view of the game, ~10 times a second, as changes (sim-core `view::UiSync`): each
 * update carries only what changed since the one before. Residents, households, relationship
 * pairs and rooms are keyed by id and come whole when changed (residents: their changed fields);
 * story events by id. Values that drift every tick are rounded to what the HUD shows, so changed
 * means visibly changed. A `full` update carries everything and replaces what the main thread
 * had: the first one, and the next one after a `resync` request. Lists are absent when nothing
 * in them changed.
 */
export interface UiUpdate {
  full: boolean;
  day: number;
  /** 0 = Monday. */
  weekday: number;
  minute: number;
  /** 0 paused, 1 normal, 2 fast, 3 faster (10×), 4 super speed (`SUPER_SPEED`: up to 30×, up to 60× while `quiet`). */
  speed: number;
  /** Whether it is quiet at home: everyone asleep or out, nothing notable there for an hour (sim-core `World::calm`); super speed runs up to 60× then. */
  quiet: boolean;
  autonomy: boolean;
  /** How fast residents age. */
  lifespan: Lifespan;
  /** Whether the player's residents move in with partners and out of home on their own. */
  playerMoves: boolean;
  /** Every resident in town (summary), with details for some (see `SimView`). */
  sims?: SimPatch[];
  /** Residents no longer listed (died, moved away). */
  simsGone?: number[];
  households?: FundsView[];
  /** Pairs with a resident whose details are sent (`SimView.detail`), both ways: nobody else's are shown. */
  relationships?: RelationshipView[];
  /** `[a, b]` pairs no longer sent (forgotten, or neither of them is followed in detail any more). */
  relationshipsGone?: [number, number][];
  /** New story events, oldest first (a full update: the recent ones). The whole log: `events` request. */
  events?: SocialEvent[];
  /** The rooms and garden of the player's home. */
  rooms?: RoomView[];
  roomsGone?: number[];
}

/** Directional: how `a` feels about `b`. Only pairs that have met. */
export interface RelationshipView {
  a: number;
  b: number;
  friendship: number;
  romance: number;
  partners: boolean;
  chemistry: number;
  /** Family: what b is to a. */
  kin?: Kin;
}

/** A family link: what one resident is to another. */
export type Kin = 'parent' | 'child' | 'sibling';

export interface SocialEvent {
  id: number;
  tick: number;
  kind: string;
  a: number;
  b: number;
  c?: number;
  /** A level or an amount of money. */
  n?: number;
  /** A one-off payment that came with the event (a promotion bonus). Where an event has both, `{money}` in its text is this and `{n}` is `n`. */
  pay?: number;
  /** Skill index. */
  skill?: number;
  /** Career index (catalog `careers`), for job events; `n` is then the level. Goal events: career category index. */
  career?: number;
  /** Goal definition index (content `goals`), for goal events (`n` the target, `skill`). */
  goal?: number;
  /** Object definition index, for things residents want (and their goals). */
  object?: number;
  /** 0 everyday, 1 notable, 2 a milestone. */
  importance: number;
}

export interface SocialOption {
  index: number;
  id: string;
  label: string;
  category: string;
  chance: number;
}

/** How a session starts: a fresh lot (with the household placed) or a save file. */
export type GameSource = { lot: string; seed: number } | { save: string };

/**
 * Why a save can't be opened: it's from a newer version of the game (kept), from a version
 * too old to read, or damaged (both removed), or the simulation failed while loading it (kept:
 * that may be the loader's fault rather than the save's, and a later version may open it).
 */
export type SaveProblem = 'tooNew' | 'tooOld' | 'corrupt' | 'failed';

export type ToWorker =
  /**
   * `speed`: the clock's speed from the first step (0: the game starts paused). `module`: the
   * simulation's wasm, compiled once a page (`game/session.ts` `compileSim`); else the worker fetches it.
   */
  | { type: 'init'; content: string; source: GameSource; speed?: number; module?: WebAssembly.Module }
  /** Only checks that a save opens (an imported file); answered with `checked`. */
  | { type: 'check'; content: string; save: string }
  | { type: 'command'; command: Command }
  | { type: 'save'; requestId: number }
  | { type: 'socialOptions'; requestId: number; actor: number; target: number }
  /** The whole story log. */
  | { type: 'events'; requestId: number }
  /** The household's home as a blueprint. */
  | { type: 'blueprint'; requestId: number; household: number }
  /** The land shaped (`SimBridge.shapeLand`): answered with `shaped`, or `failed` with why it was refused. */
  | { type: 'shapeLand'; requestId: number; command: ShapeLandCommand }
  /** The water mask as it is now (`SimBridge.waterMask`): answered with `waterMask`. */
  | { type: 'waterMask'; requestId: number }
  /** The resident whose panel is open (their details and relationships are sent too); null: none. */
  | { type: 'inspect'; sim: number | null }
  /** The next UI update sends everything. */
  | { type: 'resync' }
  /** Which part of the town to build geometry for (tile rectangle); null = everything. `seq` comes back with the worlds built for it. */
  | { type: 'view'; region: [number, number, number, number] | null; seq: number }
  /** The camera's stream rectangle: the sim's full-detail area. */
  | { type: 'watch'; region: [number, number, number, number] | null }
  /** `?debug`'s `__crash`: the next step panics, an error nobody catches, or the worker hangs. */
  | { type: 'debugCrash'; what: 'panic' | 'throw' | 'hang' }
  /**
   * Fixed-step playback (benchmarks, `SimBridge.pace`): `frames` stops the worker's own clock, and only `advance`
   * runs ticks from then on; false starts its clock again.
   */
  | { type: 'pace'; frames: boolean }
  /** Fixed-step playback: runs `ticks` ticks now, publishes what they changed, and answers with `advanced`. */
  | { type: 'advance'; requestId: number; ticks: number };

/** How the sim worker kept up over the last second (for the debug overlay). */
export interface SimThreadStats {
  /** Steps run in the last second, and the rate the sim aims for. */
  stepsPerSecond: number;
  targetPerSecond: number;
  /** Ticks run in the last second (at super speed, as many as fitted in the steps' time). */
  ticksPerSecond: number;
  /** Time per step (advance, snapshot, UI view), average and worst, ms. */
  stepMs: number;
  stepMaxMs: number;
  /** Share of the second the worker spent working, 0..1. */
  busy: number;
  /** Size of the simulation's WASM memory, bytes. */
  memoryBytes: number;
}

export type FromWorker =
  /** A start is under way and making progress (before `ready`; repeated while the simulation downloads). */
  | { type: 'starting' }
  /**
   * `speed`: the game's own speed as it opened (a save's: the one it was saved at), before the
   * `init` speed held it: what play resumes at.
   */
  | { type: 'ready'; layout: SnapshotLayout; shared: SharedArrayBuffer | null; catalog: Catalog; speed: number }
  /**
   * What changed of the world structure (`WorldUpdate`), with the lot's rooms when the lot changed
   * and its meshes when the lot or the view did. `view`: the `seq` of the view it was built for.
   */
  | { type: 'world'; view: number; world: WorldUpdate; rooms?: Uint16Array; meshes?: WorldStructure['meshes'] }
  /** Food and dishes, whenever they change (`ItemPlacement`). */
  | { type: 'items'; items: ItemPlacement[] }
  /** The lamps switched off (object ids), whenever a lamp is switched or the objects change. */
  | { type: 'lamps'; off: number[] }
  /** The curtains drawn shut (object ids), whenever some are drawn or opened or the objects change. */
  | { type: 'curtains'; shut: number[] }
  | { type: 'ui'; ui: UiUpdate }
  | { type: 'saved'; requestId: number; data: string }
  | { type: 'socialOptions'; requestId: number; options: SocialOption[] }
  | { type: 'events'; requestId: number; events: SocialEvent[] }
  /** JSON, or null with `error`. */
  | { type: 'blueprint'; requestId: number; data: string | null; error?: string }
  /** A `shapeLand` done (charged, and its meta and water changed). */
  | { type: 'shaped'; requestId: number }
  /** The water mask the simulation walks (1 m tiles over the ground, `width` × `depth`), as it is now. */
  | { type: 'waterMask'; requestId: number; water: Rle; width: number; depth: number }
  /** Fallback transport when SharedArrayBuffer is unavailable. */
  | { type: 'snapshot'; data: Float32Array; step: number }
  /** About once a second. */
  | { type: 'stats'; stats: SimThreadStats }
  /** A request (`requestId`) that couldn't be answered. */
  | { type: 'failed'; requestId: number; message: string }
  /** `fatal`: the simulation stopped for good (a trap, or a step that failed). `problem`: the game couldn't start because of its save. */
  | { type: 'error'; message: string; fatal: boolean; problem?: SaveProblem }
  /** A save `check`ed: `problem` null if it opens. */
  | { type: 'checked'; problem: SaveProblem | null; message: string }
  /** An `advance` done, after the snapshot and what the ticks changed: the ticks it ran (fewer once the game is paused). */
  | { type: 'advanced'; requestId: number; ticks: number };

/**
 * The newest thing (highest object id) a resident in `snapshot` uses or walks to, -1 if none. Ids
 * are never reused (sim-core `next_object_id`), so one above the highest the main thread was sent
 * is a thing it hasn't heard of yet (`sim.worker.ts` then sends the world at once).
 */
export function newestObjectUsed(snapshot: ArrayLike<number>, layout: SnapshotLayout): number {
  let newest = -1;
  const count = snapshot[layout.header.simCount] | 0;
  for (let i = 0; i < count; i++) newest = Math.max(newest, snapshot[layout.headerLen + i * layout.simStride + layout.sim.object] | 0);
  return newest;
}

/** Rows of the town itself (the ground storey; see `WorldStructure.storeyDepth`). */
export function groundDepth(world: { depth: number; storeyDepth?: number }): number {
  return world.storeyDepth ?? world.depth;
}

/** What `storeyOfRow` and `storeyRow` need of a world. */
export type StoreyLayout = { depth: number; storeyDepth?: number; storeys?: number; basement?: number };

/** Whether the lot holds more than the ground's rows (storeys above, or a basement). */
function layered(world: StoreyLayout): boolean {
  return ((world.storeys ?? 1) > 1 || world.basement !== undefined) && groundDepth(world) > 0;
}

/** The storey a lot row is on (0: the ground, -1: the basement). */
export function storeyOfRow(world: StoreyLayout, z: number): number {
  if (!layered(world)) return 0;
  const k = Math.max(0, Math.floor(z / groundDepth(world)));
  const b = world.basement;
  return b === undefined || k < b ? k : k === b ? -1 : k - 1;
}

/** The first lot row of storey `k` (the basement: -1). */
export function storeyRow(world: StoreyLayout, k: number): number {
  if (!layered(world)) return 0;
  const b = world.basement;
  const block = k < 0 ? (b ?? 0) : b !== undefined && k >= b ? k + 1 : k;
  return block * groundDepth(world);
}

/**
 * The colours a player gave an object's parts: `#rrggbb` by the name of the model's material it
 * goes on (`body`, `stone`); a part left out is as made.
 */
export type PartColors = Record<string, string>;

/** `colors` as one string, the same for the same colours (for template and cache keys); '' for none. */
export function colorsKey(colors: PartColors | undefined): string {
  return colors
    ? Object.keys(colors)
        .sort()
        .map((part) => `${part}:${colors[part]}`)
        .join(',')
    : '';
}

/** The lowest storey: -1 with a basement, else 0 (the ground). */
export function lowestStorey(world: { basement?: number }): number {
  return world.basement !== undefined ? -1 : 0;
}

/**
 * Whether storey `k` shows while storey `view` is in view: the ground up to it, or (viewing the
 * basement) the basement alone.
 */
export function storeyShown(k: number, view: number): boolean {
  return view < 0 ? k === view : k >= 0 && k <= view;
}

/** How far an object can stand off the middle of its footprint (centimetres per axis; sim-core `MAX_OFFSET`). */
export const MAX_OFFSET = 50;
/** Resizing (sim-core `MAX_SIZE`, `MIN_SIZE`, `MAX_TALLER`, `MAX_SHORTER`): tiles each way at most, centimetres each way at least, centimetres taller and shorter. */
export const MAX_SIZE = 6;
export const MIN_SIZE = 10;
export const MAX_TALLER = 200;
export const MAX_SHORTER = 100;

/** Where an object's middle stands (tiles): the middle of its footprint, moved by its `offset`. */
export function objectCentre(o: { x: number; z: number; w: number; d: number; offset?: [number, number] }): { x: number; z: number } {
  return { x: o.x + o.w / 2 + (o.offset?.[0] ?? 0) / 100, z: o.z + o.d / 2 + (o.offset?.[1] ?? 0) / 100 };
}
