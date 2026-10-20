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
  | { type: 'adopt'; household: number; child: boolean }
  /** "Skip quiet hours": time-lapse while the player's household sleeps or is at work. */
  | { type: 'setAutoFast'; enabled: boolean }
  | { type: 'use'; sim: number; object: number; interaction: number }
  /** Eat a serving of food (an item), or clear it away to the sink. */
  | { type: 'useItem'; sim: number; item: number; action: 'eat' | 'clear' }
  /** A lamp switched on or off (sim-core `lights.rs`): at once, nobody walks over. */
  | { type: 'switchLamp'; object: number; on: boolean }
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
  | { type: 'declinePurchase'; sim: number };

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
}

/** Someone whose slot a newcomer took; story events name them with `FORMER | index`. */
export interface FormerResident {
  name: string;
  household: string;
}

/** Event references with this bit name a former resident (sim-core `social::FORMER`). */
export const FORMER = 2 ** 31;

export interface WorldMeta {
  kind?: string;
  name?: string;
  streets?: { x: number; z: number; w: number; d: number }[];
  paths?: { x: number; z: number; w: number; d: number }[];
  seed?: number;
  /** What kind of world the town is in and how its land looks (missing: the classic valley). */
  world?: WorldLook;
  /** The menus' town preview only: the plot drawn as the player's house, as the game shows it (no garage, car or garden dressing). */
  home?: number | null;
}

/** The world a town stands in (`game/world.ts`): its kind, its land, and farm fields in town. */
export interface WorldLook {
  kind: string;
  landscape: LandscapeParams;
  /** Tilled fields on farm lots (town tiles): drawn as soil, fenced in. */
  fields?: { x: number; z: number; w: number; d: number }[];
}

/**
 * How the land round a town looks (see `render/babylon/nature.ts`). `DEFAULT_LANDSCAPE` is the
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
}

/** Sent when walls or objects change (rare). */
/** Living: the residents earn the money for building. Creative: building is free. */
export type GameKind = 'living' | 'creative';

/** The parts of `WorldStructure` that are the lot itself: sent only when `lotVersion` changes. */
export type LotPart = 'rooms' | 'walls' | 'openings' | 'diagonals' | 'fences' | 'floors' | 'meshes';
export type LeanWorld = Omit<WorldStructure, LotPart>;

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
  plots: PlotInfo[];
  exits: [number, number][];
  rooms: number[];
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
  /** Plot the Sim is on (null on the street or at work). */
  plot: number | null;
  /** Minute of day they come back from work, if away. */
  awayUntil: number | null;
  visiting: number | null;
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
  build: { wall: number; door: number; window: number; remove: number; diagonalWall?: number; fence?: number; gate?: number };
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
  speed: number;
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
  | { type: 'init'; content: string; source: GameSource }
  /** Only checks that a save opens (an imported file); answered with `checked`. */
  | { type: 'check'; content: string; save: string }
  | { type: 'command'; command: Command }
  | { type: 'save'; requestId: number }
  | { type: 'socialOptions'; requestId: number; actor: number; target: number }
  /** The whole story log. */
  | { type: 'events'; requestId: number }
  /** The household's home as a blueprint. */
  | { type: 'blueprint'; requestId: number; household: number }
  /** The resident whose panel is open (their details and relationships are sent too); null: none. */
  | { type: 'inspect'; sim: number | null }
  /** The next UI update sends everything. */
  | { type: 'resync' }
  /** Which part of the town to build geometry for (tile rectangle); null = everything. */
  | { type: 'view'; region: [number, number, number, number] | null }
  /** `?debug`'s `__crash`: the next step panics, an error nobody catches, or the worker hangs. */
  | { type: 'debugCrash'; what: 'panic' | 'throw' | 'hang' };

/** How the sim worker kept up over the last second (for the debug overlay). */
export interface SimThreadStats {
  /** Steps run in the last second, and the rate the sim aims for. */
  stepsPerSecond: number;
  targetPerSecond: number;
  /** Time per step (advance, snapshot, UI view), average and worst, ms. */
  stepMs: number;
  stepMaxMs: number;
  /** Share of the second the worker spent working, 0..1. */
  busy: number;
  /** Size of the simulation's WASM memory, bytes. */
  memoryBytes: number;
}

/** How far a game's start has come: `loading` the simulation (repeated as it downloads), then `building` its world or opening its save. */
export type StartPhase = 'loading' | 'building';

export type FromWorker =
  /** A start is under way (before `ready`). */
  | { type: 'starting'; phase: StartPhase }
  | { type: 'ready'; layout: SnapshotLayout; shared: SharedArrayBuffer | null; catalog: Catalog }
  /** `full`: with the lot; otherwise a `LeanWorld` (the lot is as last sent). */
  | { type: 'world'; world: WorldStructure | LeanWorld; full: boolean }
  /** Food and dishes, whenever they change (`ItemPlacement`). */
  | { type: 'items'; items: ItemPlacement[] }
  /** The lamps switched off (object ids), whenever a lamp is switched or the objects change. */
  | { type: 'lamps'; off: number[] }
  | { type: 'ui'; ui: UiUpdate }
  | { type: 'saved'; requestId: number; data: string }
  | { type: 'socialOptions'; requestId: number; options: SocialOption[] }
  | { type: 'events'; requestId: number; events: SocialEvent[] }
  /** JSON, or null with `error`. */
  | { type: 'blueprint'; requestId: number; data: string | null; error?: string }
  /** Fallback transport when SharedArrayBuffer is unavailable. */
  | { type: 'snapshot'; data: Float32Array; step: number }
  /** About once a second. */
  | { type: 'stats'; stats: SimThreadStats }
  /** A request (`requestId`) that couldn't be answered. */
  | { type: 'failed'; requestId: number; message: string }
  /** `fatal`: the simulation stopped for good (a trap, or a step that failed). `problem`: the game couldn't start because of its save. */
  | { type: 'error'; message: string; fatal: boolean; problem?: SaveProblem }
  /** A save `check`ed: `problem` null if it opens. */
  | { type: 'checked'; problem: SaveProblem | null; message: string };

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
