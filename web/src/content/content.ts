/**
 * Gameplay content as the main thread sees it (labels, icons, descriptions).
 * The simulation receives the same JSON, ignores the visual fields, and is the
 * authority on rules; `validateCharacter` mirrors its checks for instant UI feedback.
 */

import { withBuild } from '../assets/build';
import { bundled } from '../assets/bundle';

export interface NeedDef {
  id: string;
  label: string;
  decayPerHour: number;
  /** The level residents like to keep it at: below it they see to it when convenient. */
  comfortable?: number;
  /** Asset key. */
  icon: string;
}

export interface InteractionDef {
  id: string;
  label: string;
  minutes: number;
  /** Need changes over the whole interaction (absent: none, like a crib's care, which is for the baby). */
  effects?: Record<string, number>;
  /** Needs it fills for the baby lying in the object (cribs). */
  care?: Record<string, number>;
  pose?: 'stand' | 'sit' | 'lie';
  /** Sit on a free seat within this many metres of the object, if there is one (watching TV from the sofa). */
  seatNear?: number;
  /** Only a seat that looks at the object: from in front of it (`front`, a screen) or from any side (`around`, a fire). */
  seatFacing?: 'front' | 'around';
  /** Share (0..1) of the use's need gains that nearby residents get too (hearing the TV while eating). */
  broadcast?: number;
  /** How far (m) that carries outside the same room. */
  broadcastRange?: number;
  /** Used standing on the object, not in front of it (a shower): in on its centre, facing out. */
  inside?: boolean;
  autonomous?: boolean;
  tags?: string[];
  /** Skill levels gained per hour. */
  skills?: Record<string, number>;
  /** Money charged from household funds each time it's used. */
  cost?: number;
  /** Skill that makes this interaction better (bigger need gains per level). */
  skill?: string;
  /** Feeling granted when finished (if the Sim has at least `feelingMinSkill` in `skill`). */
  feeling?: string;
  feelingMinSkill?: number;
  /**
   * Animation tag (one of the content's `animations`) the renderer shows while a Sim uses it.
   * Without one the simulation derives a tag from `tags`, `pose` and `effects`.
   */
  anim?: string;
  /** Only offered while the seat faces a table it tucks under (`seatAt`): dining at a table. */
  needsTable?: boolean;
  /** Things a working object on the same plot must provide (`ObjectDef.provides`): coffee needs a coffee machine. */
  requires?: string[];
  /** Takes every slot of the object: only started with nobody else on it, and nobody else gets on while it lasts. */
  whole?: boolean;
  /**
   * Starts this recipe (`recipes` id): the resident goes round its stations (fridge, counter,
   * stove) instead of using this object alone, and a dish is served. `effects` are then what
   * cooking does for the cook; `minutes` aren't needed (the steps take their own).
   */
  recipe?: string;
  /**
   * Food to sit down with: `minutes` is then the fetching, and the resident eats `item` (an
   * `items` id) at a table if there is one, for `eat.minutes`.
   */
  eat?: { item: string; minutes: number };
  /** Only offered while the object is open or closed (a wall bed's "Fold up" while it's down), or its crop ripe. */
  when?: 'open' | 'closed' | 'ripe';
  /** Opens or closes the object as the use starts, and leaves it so (sleeping in a wall bed folds it down); or picks its crop. */
  leaves?: 'open' | 'closed' | 'picked';
  /** A tag: what follows when it finishes (toilets: `handwash`, washing hands at a sink). */
  after?: string;
  /** A spare bed's night (a sofa bed's): it doesn't make the room a bedroom, and is slept in with no other bed free. */
  spare?: boolean;
  /** The baby's place in a crib: only a baby takes it. */
  baby?: boolean;
  /** Grade points for a full use: only pupils do homework. */
  homework?: number;
  /**
   * The life stages (`life.stages` ids) that may take it, else its recipe's, else its object's
   * (default every stage but a baby's): the kids' things are a child's, wine a grown-up's.
   */
  stages?: string[];
  /** Only started between these hours of the day (`[from, to]`, past midnight when `from` > `to`): stargazing after dark. */
  hours?: [number, number];
  /** Only under open sky: the object stands outside. */
  sky?: boolean;
  /** Only with something to work on: dirty dishes, washing to do, a washed load to dry (sim-core `content::WorkOn`). */
  workOn?: 'dishes' | 'laundry' | 'washed';
  /** A game for two, at an object with two places: played with someone free nearby, fetched to the other place (sim-core `partners.rs`). */
  partner?: boolean;
  /** With `partner`: the use (an id of the same object's) the player's order does instead with nobody free to play. */
  alone?: string;
  /**
   * What it sounds like while it lasts: a world sound by key (`audio/world.ts` `LOOPS`: `chop` for
   * a cheese board), or `none`; absent: by the object's `sounds`, else by `anim`. Not for a
   * `recipe`, whose steps sound by the station they're done at.
   */
  sound?: string;
  /** A sound played once as it ends (`audio/water.ts` `ONCE`: a toilet's `flush`). */
  endSound?: string;
}

/** How cooking, eating and washing up go (content `cooking`; sim-core `cooking::CookingRules`). */
export interface CookingRules {
  /** The kinds of station (objects' `stations` and recipe steps' `at` name them). */
  stations?: string[];
  spoilHours?: number;
  storedSpoilHours?: number;
  eatMinutes?: number;
  dishesMinutes?: number;
  carryDishes?: number;
  leftoverPreference?: number;
  feedFamily?: number;
  hungryBelow?: number;
  invitedBoost?: number;
  inviteMinutes?: number;
  stationWaitMinutes?: number;
  dirtyDirtPerHour?: number;
  skipPenalty?: number;
  seatTags?: string[];
  seatAvoid?: string[];
  /** Height (m) of a station's top without a `surface` (where the board or pan is drawn). */
  defaultSurfaceHeight?: number;
  serveRange?: number;
  eatLabel?: string;
  eatTags?: string[];
  eatAnim?: string;
  dishesLabel?: string;
  dishesTags?: string[];
  dishesAnim?: string;
  dishesInterest?: number;
  /** Clearing away: scraping food off one thing (into a `bin`, else at the sink). */
  scrapeMinutes?: number;
  scrapeAnim?: string;
  /** Putting an armful of clean things away, at the nearest of `putAwayAt` (stations, the first listed on a tie; default `prep`). */
  putAwayMinutes?: number;
  putAwayAnim?: string;
  putAwayAt?: string[];
}

/** Something residents carry or put down (content `items`): ingredients, a pot of stew, a plate. */
export interface ItemDef {
  id: string;
  label: string;
  /** Asset key of its model (`item.*`). */
  model?: string;
  /** Carried in one hand (a plate) or both (a pot). */
  hold?: 'hand' | 'both';
  /** Taking a serving gives one of these (a pot's servings are plates). */
  serving?: string;
  /** What's left once it's eaten up (a dirty pot). */
  emptyAs?: string;
  /** Needs washing up. */
  dirty?: boolean;
  radius?: number;
}

/** One step of a recipe: at one of the stations `at`, for `minutes`. */
export interface RecipeStep {
  label?: string;
  at: string[];
  minutes: number;
  anim?: string;
  /** Another `anim` at some of its stations (`{oven: 'bake'}`), for a station object whose first kind in `at` it is. */
  animAt?: Record<string, string>;
  /** Done without when the home has no such station. */
  optional?: boolean;
  /** What the cook carries away from it (an item id). */
  holds?: string;
  /** What it sounds like, over the station's (`ObjectDef.sounds`; a world sound key or `none`): a bake in the oven of a cooker is no frying pan. */
  sound?: string;
}

/** Something to cook (content `recipes`; sim-core `cooking::RecipeDef`). */
export interface RecipeDef {
  id: string;
  label: string;
  /** The dish it serves (an item id). */
  item: string;
  servings?: [number, number];
  costPerServing?: number;
  /** What eating one serving gives. */
  effects?: Record<string, number>;
  eatFeeling?: string;
  /** The cook eats it straight away (a sandwich). */
  eatNow?: boolean;
  eatMinutes?: number;
  /** Cooked and served outdoors (a barbecue). */
  outdoor?: boolean;
  steps: RecipeStep[];
  /** The life stages that may cook it, for interactions without `stages` of their own (at a hot stove: teens and up). */
  stages?: string[];
}

/** Where a lamp's shade sends its light (content `light.aim`; see `ObjectDef.light`). */
export type LightAim = 'down' | 'up' | 'out' | 'ceiling';

/** What kind of top a surface is (sim-core `content::SurfaceKind`): which seats tuck under it. */
export type SurfaceKind = 'table' | 'bar' | 'counter' | 'desk' | 'low' | 'shelf';

export interface ObjectDef {
  id: string;
  name: string;
  /** Asset key of the 3D model. */
  model: string;
  /** Asset key of the catalog icon. */
  icon?: string;
  footprint?: [number, number];
  interactions: InteractionDef[];
  /** Buy-mode price; objects without one aren't sold. */
  price?: number;
  /** Buy-mode category id. */
  category?: string;
  /** Decor points it gives its room (default: by category, `roomRules.decorByCategory`). */
  decor?: number;
  /** Group within the category (one of its `groups`), e.g. `trees` in the garden. */
  group?: string;
  description?: string;
  /** Sims that can use it at once. */
  slots?: number;
  /**
   * Each slot's place (sim-core `ObjectInstance::slot_position`, `slot_yaw`; `slotPlace`): metres off the
   * footprint's middle to its right and front (before turning, for its own size), and the way its user faces,
   * degrees from its front towards its right. Absent: spread across its width, facing its front.
   */
  places?: { at: [number, number]; facing?: number }[];
  /** Things it provides while it works (interactions `requires`): a coffee machine provides coffee. */
  provides?: string[];
  /** Can only be placed outdoors (trees, flower beds, ponds). */
  outdoors?: boolean;
  /** A flight of stairs to the storey above (sim-core `storeys.rs`). */
  stairs?: boolean;
  /**
   * Gives light at night (lamps, lanterns): reach in metres, relative brightness, height of the bulb,
   * where the bulb is off the middle (`at`: metres, x and z in the object's own frame, before
   * turning; default the middle), and the floor it lights fully as its room is scored (tiles;
   * default `roomRules.lampTiles`). `aim`: where its shade sends the light (default all round):
   * `down` (a pendant's dome, a desk lamp), `up` (a sconce's open cup: a wash up the wall), `out`
   * (a wall light: away from its wall and down, none behind it) or `ceiling` (an uplighter: up
   * at the ceiling, which lights the room). `switch: false`: no switch, lit every night; `fire`:
   * a fire, lit to sit by (sim-core `lights.rs`), its flames burning while it's lit.
   */
  light?: { range?: number; intensity?: number; height?: number; at?: [number, number]; tiles?: number; switch?: boolean; fire?: boolean; aim?: LightAim };
  /**
   * What's done at it sounds like this, by animation tag (world sound keys or `none`, as an
   * interaction's `sound`), for its uses without a `sound` and the cooking steps at it as a
   * station: an oven's `stir` is silent, not a frying pan.
   */
  sounds?: Record<string, string>;
  /** A world sound it makes by itself, used or not, while its lot is looked at (a fountain's water). */
  ambient?: string;
  /** Turns freely (at any angle, for looks); default: 1×1 objects of `objectRules.freeRotation` categories. */
  freeRotation?: boolean;
  /**
   * Where it goes (sim-core `content::Layer`): furniture on the floor (default), on a `wall` (its
   * back against one), from the `ceiling` (indoors) or flat on the floor as a `rug`. Off the floor
   * layer it takes no floor space; only things on the same layer can't overlap. On a `surface`
   * it stands on another object's top (a vase on a table).
   */
  layer?: 'floor' | 'wall' | 'ceiling' | 'rug' | 'surface';
  /** On a wall: it belongs over a window (curtains, blinds); everything else on a wall needs plain wall behind it. */
  overWindow?: boolean;
  /** On the floor: as tall as the wall (a wardrobe, a fireplace): nothing hangs on the wall or from the ceiling over it. */
  fillsWall?: boolean;
  /** From the ceiling: hangs low enough to walk into (a hanging plant): it takes the floor under it. */
  hangsLow?: boolean;
  /** From the ceiling: hangs at head height (a chandelier, a low pendant), to hang over a table: nobody walks under it, but furniture goes under it. */
  headHeight?: boolean;
  /** On the floor: built to stand back to a wall (a wall-hung toilet, a wall bed, a leaning ladder): it needs plain wall all along its back. */
  againstWall?: boolean;
  /** Bunk beds: the second slot lies on the upper bunk. */
  bunk?: boolean;
  /** Bunk beds: the life stages that may take the upper bunk (default everyone who may use the bed). */
  upperStages?: string[];
  /** The life stages that may use it, for interactions without `stages` of their own. */
  stages?: string[];
  /**
   * A top small things stand on: its height (m), kind, and the margin kept clear along its edges
   * (m, default 0.04); its width and depth (m, default the footprint's) and where its middle is off
   * the footprint's middle (m, to the object's right and front; default [0, 0]).
   */
  surface?: { height: number; kind: SurfaceKind; inset?: number; size?: [number, number]; at?: [number, number] };
  /** Things on the `surface` layer: the radius (m) of the disc they take on their host's top (default 0.12). */
  radius?: number;
  /**
   * Where it is used from: the tile in front (default), any free tile `around` it (chairs pushed under a table);
   * sat or lain on from beside the slot (`side`: beds, bikes) or from behind it (`behind`: a desk's chair), in
   * front of it in build mode's rule and used standing (sim-core's `seat_approach`).
   */
  approach?: 'front' | 'around' | 'side' | 'behind';
  /** Seats: the kinds of surface they tuck under; facing one, they are a seat at that table. */
  seatAt?: SurfaceKind[];
  /**
   * Seats (renderer only): height of the seat (m) and how far forward of the object's middle the hips sit (m);
   * `facing: 'back'` turns the sitter to face the object's back (a pianist at an upright with its back to the
   * wall), `legs: 'out'` stretches the legs out along the seat (a chaise), `recline` is how far back (degrees,
   * from sitting up) the sitter leans relaxing, napping or watching (against a slanted back: an Adirondack
   * chair; default a little, about 8), and `pedals` are what the feet turn (an exercise bike: the crank axle's
   * height and forward offset, and the cranks' length, m).
   */
  seat?: { height: number; forward?: number; facing?: 'front' | 'back'; legs?: 'down' | 'out'; recline?: number; pedals?: { height: number; forward?: number; crank: number } };
  /**
   * Used standing up on it (renderer only): the height (m) of what residents stand on (a treadmill's belt, a
   * trampoline's mat), who step up onto its middle from the tile in front; `motion` bounces them on it or
   * buzzes them.
   */
  stand?: { height: number; motion?: 'bounce' | 'buzz' };
  /**
   * What residents hold while they use it (renderer only): a moving part of its model (by node name; the
   * model hides it while used, `motions`), held by `grip`: a `bar` across both hands (a barbell), a `pair`
   * (two parts, one in each hand: dumbbells), a `guitar` (its neck in the left hand, its body against
   * the belly, strummed) or a `pole` (its foot on the floor ahead, its top in the hands: a mop).
   */
  held?: { part: string | string[]; grip: 'bar' | 'pair' | 'guitar' | 'pole' };
  /** Cooking stations it is (`cooking.stations`: `ingredients`, `prep`, `stove`, `oven`, `grill`, `microwave`, `dishes`, `bin`, `cupboard`). */
  stations?: string[];
  /** Keeps leftovers (fridges). */
  stores?: boolean;
}

export interface SkillDef {
  id: string;
  label: string;
  icon: string;
  description: string;
}

/** A colour the player can give an object (the parts its model paints). Visual only: price and effects stay the same. */
export interface ObjectColorDef {
  id: string;
  label: string;
  color: string;
}

export interface BuyCategory {
  id: string;
  label: string;
  /** Icon asset key for the catalog tab. */
  icon?: string;
  /** Sub-groups shown as chips when the category is open; objects name theirs in `group`. */
  groups?: { id: string; label: string; outside?: boolean }[];
  /** Its things belong in the garden (wishes for a room don't offer them); a group can say otherwise. */
  outside?: boolean;
}

/** A wall covering (Build mode's paint): a finish (texture) tinted with a colour. Price per face. */
export interface WallCoveringDef {
  id: string;
  label: string;
  finish: 'plaster' | 'wallpaper' | 'siding' | 'brick' | 'wood' | 'stone' | 'tile';
  color: string;
  price?: number;
}

/** What happens when a need runs out (sim-core `AccidentDef`); `story` reads "{a} …". */
export interface AccidentDef {
  id: string;
  need: string;
  story?: string;
  feeling?: string;
}

/** A stage of life (content `life.stages`): from `from` years until the next stage's. */
export interface LifeStageDef {
  id: string;
  label: string;
  from: number;
  /** The story's words when someone reaches it ("is an elder now"). */
  story?: string;
  /** How far hair greys at this stage (0..1). */
  greyHair?: number;
  /** Grown up (default: unless `baby` or `school`); a baby (lives in a crib); goes to school. */
  adult?: boolean;
  baby?: boolean;
  school?: boolean;
  /** Body size and head size for the stage (renderer). */
  scale?: number;
  head?: number;
  /** How much they stoop (0..1; elders). */
  stoop?: number;
}

/** What a room is for, from the tags of what stands in it (sim-core `RoomKind`). */
export interface RoomKindDef {
  id: string;
  label?: string;
  /** Asset key of its icon (thought bubbles, Our home). */
  icon?: string;
  tags: string[];
  essentials?: string[][];
  size?: [number, number];
  exclusive?: boolean;
  /** A room no kind's tags are offered in is of this kind if people pass through it (two doors or more, or stairs): a hallway. */
  passage?: boolean;
}

/** A roof style (Build mode's Roof tool): the shape and, for pitched roofs, the pitch in degrees. */
export interface RoofStyleDef {
  id: string;
  label: string;
  shape: 'gable' | 'hip' | 'flat';
  pitch?: number;
}

/** A roof colour (shingles). */
export interface RoofColorDef {
  id: string;
  label: string;
  color: string;
}

/** A fence style (Build mode's Fence and Gate tools). Price per metre; a gate adds `build.gate`. */
export interface FenceStyleDef {
  id: string;
  label: string;
  kind: 'picket' | 'rails' | 'slats' | 'iron' | 'stone';
  color: string;
  /** Height in metres. */
  height?: number;
  price?: number;
}

/** A floor covering (Build mode's Floor tool): a finish (texture) tinted with a colour. Price per tile. */
export interface FloorCoveringDef {
  id: string;
  label: string;
  finish: 'wood' | 'whitewash' | 'tile' | 'carpet' | 'stone';
  color: string;
  price?: number;
}

/** A door style: the leaf (`panel` painted, `oak` natural wood, `halfGlass`, `glass`) and its paint. */
export interface DoorStyleDef {
  id: string;
  label: string;
  leaf: 'panel' | 'oak' | 'halfGlass' | 'glass';
  /** Paint (or frame) colour; without one, front doors take the house's accent colour. */
  color?: string;
  price?: number;
}

/** A window style: opening heights (m), glazing bars and shutters outside (`house`: if the house's style has them). */
export interface WindowStyleDef {
  id: string;
  label: string;
  panes: 'cross' | 'grid' | 'bar' | 'transom' | 'none';
  sill: number;
  head: number;
  /** Wall left either side of the glazing (m). */
  inset: number;
  shutters: boolean | 'house';
  price?: number;
}

export interface CareerCategory {
  id: string;
  label: string;
  icon: string;
  description: string;
  tracks: { id: string; label: string; titles: string[] }[];
}

export interface Economy {
  startingFunds: number;
  /** Extra money to build with when a household starts on an empty lot (Living games). */
  emptyLotFunds?: number;
  currency: string;
  rent?: { weekday: number; hour: number; base: number; perTile: number; billsBase?: number; billsRate?: number };
}

export interface TraitDef {
  id: string;
  label: string;
  description: string;
  icon: string;
  conflicts?: string[];
}

export interface PerkDef {
  id: string;
  label: string;
  description: string;
  icon: string;
  cost: number;
}

export interface GenderDef {
  id: string;
  label: string;
  pronouns: { subject: string; object: string; possessive: string };
}

export interface EmotionDef {
  id: string;
  label: string;
  icon: string;
}

export interface SocialDef {
  id: string;
  label: string;
  category: string;
  icon: string;
}

/** Something to do on the phone (`phone.actions`; sim-core `phone.rs`). */
export interface PhoneActionDef {
  id: string;
  label: string;
  icon?: string;
  /** With someone (texting, calling, inviting); without: on their own (scrolling). */
  contact?: boolean;
  /** After a yes: they come over (`here`) or the caller goes over (`there`). */
  meet?: 'here' | 'there';
}

/** What a routine block can be for (`activities`). */
export interface ActivityDef {
  id: string;
  label: string;
  icon: string;
  tags?: string[];
  /** Blocks name a skill to train. */
  skill?: boolean;
  social?: boolean;
  visit?: boolean;
  sleep?: boolean;
}

export interface GoalDef {
  id: string;
  /** With `{skill}`, `{n}`, `{category}` and `{object}`. */
  label: string;
  icon: string;
  kind: 'hasJob' | 'jobLevel' | 'promoted' | 'skill' | 'friends' | 'partner' | 'funds' | 'own';
}

export interface PlannerRules {
  maxMinutes: number;
  maxSleepMinutes: number;
  maxGoals: number;
}

export interface Names {
  /** Every first name (used for genders without their own list). */
  first: string[];
  /** First names per gender id. */
  byGender?: Record<string, string[]>;
  last: string[];
  towns?: string[];
  streets?: string[];
  /** Names for each kind of world (`game/world.ts`), by its id. */
  worlds?: Record<string, WorldNames>;
}

/** A kind of world's names: a town is a `start` and an `end`, a street a `streets` word and a `ways` word. */
export interface WorldNames {
  start: string[];
  end: string[];
  streets: string[];
  ways: string[];
  /** One line about the place, shown when the world is made. */
  taglines: string[];
}

export interface CharacterRules {
  minTraits: number;
  maxTraits: number;
  perkPoints: number;
  maxHousehold: number;
}

interface ContentFile {
  needs: NeedDef[];
  rules: CharacterRules;
  traits: TraitDef[];
  perks: PerkDef[];
  names: Names;
  genders: GenderDef[];
  emotions: EmotionDef[];
  socials: SocialDef[];
  careerCategories?: CareerCategory[];
  bondPresets: Record<string, unknown>;
  events: Record<string, string>;
  objects: ObjectDef[];
  skills?: SkillDef[];
  /** The palette of object colours; any other colour can be chosen too. */
  objectColors?: ObjectColorDef[];
  buyCategories?: BuyCategory[];
  wallCoverings?: WallCoveringDef[];
  floorCoverings?: FloorCoveringDef[];
  doorStyles?: DoorStyleDef[];
  windowStyles?: WindowStyleDef[];
  fenceStyles?: FenceStyleDef[];
  roofStyles?: RoofStyleDef[];
  roofColors?: RoofColorDef[];
  roomKinds?: RoomKindDef[];
  accidents?: AccidentDef[];
  roomRules?: { decorByCategory?: Record<string, number>; lampTiles?: number; windowTiles?: number };
  life?: { daysPerYear?: number; startAge?: [number, number]; stages?: LifeStageDef[]; adoption?: { cost: number } };
  economy?: Partial<Economy>;
  objectRules?: { freeRotation?: string[] };
  /** Animation tags interactions can use as `anim` (the snapshot layout's `actions`). */
  animations?: string[];
  activities?: ActivityDef[];
  goals?: GoalDef[];
  planner?: Partial<PlannerRules>;
  /** Spoken lines per language, laid out like `content/voice/<lang>.json` (see `voice/lines.ts`). */
  voice?: Record<string, Json>;
  /** How the language model is asked for lines, per language, laid out like `content/talk/<lang>.json` (see `talk/prompt.ts`). */
  talk?: Record<string, Json>;
  cooking?: CookingRules;
  items?: ItemDef[];
  recipes?: RecipeDef[];
  phone?: { actions?: PhoneActionDef[] };
  dayRhythm?: { sleepTags?: string[] };
  /** Further content files (relative to this one), merged key by key. */
  include?: string[];
}

export class Content {
  readonly needs: readonly NeedDef[];
  readonly traits: readonly TraitDef[];
  readonly perks: readonly PerkDef[];
  readonly rules: CharacterRules;
  readonly names: Names;
  readonly genders: readonly GenderDef[];
  readonly emotions: readonly EmotionDef[];
  readonly socials: readonly SocialDef[];
  readonly careerCategories: readonly CareerCategory[];
  readonly skills: readonly SkillDef[];
  readonly objectColors: readonly ObjectColorDef[];
  readonly buyCategories: readonly BuyCategory[];
  /** Build mode looks; a wall face's covering is an index into `wallCoverings` + 1 (0: automatic). */
  readonly wallCoverings: readonly WallCoveringDef[];
  /** A floor tile's covering is an index into `floorCoverings` + 1 (0: automatic, by room). */
  readonly floorCoverings: readonly FloorCoveringDef[];
  readonly doorStyles: readonly DoorStyleDef[];
  readonly fenceStyles: readonly FenceStyleDef[];
  readonly roofStyles: readonly RoofStyleDef[];
  readonly roofColors: readonly RoofColorDef[];
  readonly roomKinds: readonly RoomKindDef[];
  readonly accidents: readonly AccidentDef[];
  /** Life stages, youngest first. */
  readonly lifeStages: readonly LifeStageDef[];
  /** What adopting costs (null: no adoption). */
  readonly adoptionCost: number | null;
  private readonly decorByCategory: Readonly<Record<string, number>>;
  /** Tiles one lamp lights fully at night (`roomRules.lampTiles`, as the simulation scores rooms). */
  readonly lampTiles: number;
  /** Tiles one window lights fully by day (`roomRules.windowTiles`). */
  readonly windowTiles: number;
  readonly windowStyles: readonly WindowStyleDef[];
  readonly economy: Economy;
  /** Animation tags, in the order the snapshot's `sim.action` indexes them. */
  readonly animations: readonly string[];
  /** Objects for sale, in content order. */
  readonly shop: readonly ObjectDef[];
  readonly bondPresets: readonly string[];
  /** Story-feed templates with `{a}`, `{b}`, `{c}` placeholders. */
  readonly eventTexts: Readonly<Record<string, string>>;
  /** What routine blocks can be for; life goals; planner limits. */
  readonly activities: readonly ActivityDef[];
  readonly goals: readonly GoalDef[];
  readonly planner: PlannerRules;
  private readonly objects: Map<string, ObjectDef>;
  /** Every object, in the order the simulation numbers them (story events name them by index). */
  readonly objectList: readonly ObjectDef[];
  /** Spoken lines from content packs per language, laid over `content/voice/<lang>.json`. */
  readonly voice: Readonly<Record<string, Json>>;
  /** The language model's words from content packs per language, laid over `content/talk/<lang>.json`. */
  readonly talk: Readonly<Record<string, Json>>;
  /** Cooking rules, and the items and recipes in the order the snapshot's `sim.held` / `sim.recipe` index them. */
  readonly cooking: CookingRules;
  readonly items: readonly ItemDef[];
  readonly recipes: readonly RecipeDef[];
  /** What residents do on their phones, in the order phone commands index them. */
  readonly phoneActions: readonly PhoneActionDef[];
  /** Interaction tags that count as sleep (`dayRhythm.sleepTags`; sim-core `DayRhythm::sleep_tags`): a bed is what has one. */
  readonly sleepTags: readonly string[];
  private readonly itemById: Map<string, ItemDef>;

  private constructor(
    /** Raw JSON, forwarded unchanged to the simulation. */
    readonly json: string,
  ) {
    const file = JSON.parse(json) as ContentFile;
    this.needs = file.needs;
    this.traits = file.traits;
    this.perks = file.perks;
    this.rules = file.rules;
    this.names = file.names;
    this.genders = file.genders ?? [];
    this.emotions = file.emotions ?? [];
    this.socials = file.socials ?? [];
    this.careerCategories = file.careerCategories ?? [];
    this.skills = file.skills ?? [];
    this.objectColors = file.objectColors ?? [];
    this.buyCategories = file.buyCategories ?? [];
    this.wallCoverings = file.wallCoverings ?? [];
    this.floorCoverings = file.floorCoverings ?? [];
    this.doorStyles = file.doorStyles ?? [];
    this.windowStyles = file.windowStyles ?? [];
    this.fenceStyles = file.fenceStyles ?? [];
    this.roofStyles = file.roofStyles ?? [];
    this.roofColors = file.roofColors ?? [];
    this.roomKinds = file.roomKinds ?? [];
    this.accidents = file.accidents ?? [];
    this.lifeStages = file.life?.stages ?? [];
    this.adoptionCost = file.life?.adoption?.cost ?? null;
    this.decorByCategory = file.roomRules?.decorByCategory ?? {};
    this.lampTiles = file.roomRules?.lampTiles ?? 10;
    this.windowTiles = file.roomRules?.windowTiles ?? 6;
    this.economy = { startingFunds: 0, currency: '$', ...file.economy };
    this.animations = [...new Set(file.animations ?? [])];
    this.shop = file.objects.filter((o) => o.price !== undefined);
    this.bondPresets = Object.keys(file.bondPresets ?? {});
    this.eventTexts = file.events ?? {};
    this.activities = file.activities ?? [];
    this.goals = file.goals ?? [];
    this.planner = { maxMinutes: 240, maxSleepMinutes: 720, maxGoals: 10, ...file.planner };
    this.objects = new Map(file.objects.map((o) => [o.id, o]));
    this.objectList = file.objects;
    this.voice = file.voice ?? {};
    this.talk = file.talk ?? {};
    this.cooking = file.cooking ?? {};
    this.items = file.items ?? [];
    this.recipes = file.recipes ?? [];
    this.phoneActions = file.phone?.actions ?? [];
    this.sleepTags = file.dayRhythm?.sleepTags ?? [];
    this.itemById = new Map(this.items.map((i) => [i.id, i]));
    this.freeRotation = new Set(file.objectRules?.freeRotation ?? []);
  }

  private readonly freeRotation: Set<string>;

  /** Whether an object turns freely (sim-core `ObjectDef::turns`): at any angle past its facing. */
  turns(def: ObjectDef): boolean {
    const [w, d] = def.footprint ?? [1, 1];
    return def.freeRotation ?? (def.layer === 'surface' || (def.layer !== 'wall' && w === 1 && d === 1 && def.category !== undefined && this.freeRotation.has(def.category)));
  }

  /** Things on the `surface` layer: the radius (m) of the disc they take on a top (sim-core `ObjectDef::radius`). */
  radius(def: ObjectDef | undefined): number {
    return Math.max(0.02, Math.min(0.45, def?.radius ?? 0.12));
  }

  item(id: string): ItemDef | undefined {
    return this.itemById.get(id);
  }

  /** Its layer (`floor` for furniture): only things on the same layer take each other's place. */
  layer(def: ObjectDef | undefined): NonNullable<ObjectDef['layer']> {
    return def?.layer ?? 'floor';
  }

  /** Loads a content file and merges the files it includes (content packs); see `mergeContent`. */
  static async load(url: string): Promise<Content> {
    const base = new URL(url, location.href);
    const baseText = await fetchText(base.href);
    const includes = ((JSON.parse(baseText) as { include?: unknown }).include as string[] | undefined) ?? [];
    const parts = await Promise.all(includes.map((name) => fetchText(new URL(name, base).href)));
    const name = base.pathname.split('/').pop() || url;
    return new Content(JSON.stringify(mergeContent([[name, baseText], ...includes.map((n, i): [string, string] => [n, parts[i]])])));
  }

  activity(id: string): ActivityDef | undefined {
    return this.activities.find((a) => a.id === id);
  }

  /** The life stage at `age` (the last one reached). */
  stageOf(age: number): LifeStageDef | undefined {
    let stage: LifeStageDef | undefined;
    for (const s of this.lifeStages) if (age >= s.from) stage = s;
    return stage ?? this.lifeStages[0];
  }

  /** Whether someone at `age` is grown up (may work, fall in love, live without a parent). */
  isAdult(age: number): boolean {
    const s = this.stageOf(age);
    return !s || (s.adult ?? (!s.baby && !s.school));
  }

  /** The grown-up stages, youngest first. */
  get adultStages(): LifeStageDef[] {
    return this.lifeStages.filter((s) => s.adult ?? (!s.baby && !s.school));
  }

  /** Ages a stage spans, `[from, to]` (the last one: fifteen years). */
  stageAges(id: string): [number, number] {
    const i = this.lifeStages.findIndex((s) => s.id === id);
    const s = this.lifeStages[i];
    if (!s) return [25, 50];
    const next = this.lifeStages[i + 1];
    return [s.from, next ? next.from - 1 : s.from + 15];
  }

  /**
   * Whether someone in life stage `stage` (an id) may take interaction `it` of `def`, as the
   * simulation keeps it (sim-core `world::unfit`): its `stages`, homework only for pupils, a
   * baby's care only for grown-ups.
   */
  mayUse(def: ObjectDef, it: InteractionDef, stage: string | null | undefined): boolean {
    const s = this.lifeStages.find((x) => x.id === stage);
    if (!s) return true;
    const stages = it.stages ?? (it.recipe ? this.recipes.find((r) => r.id === it.recipe)?.stages : undefined) ?? def.stages;
    const babies = this.lifeStages.some((x) => x.baby);
    if (stages ? !stages.includes(s.id) : it.baby ? babies && !s.baby : !!s.baby) return false;
    if (it.homework && !s.school) return false;
    return !Object.values(it.care ?? {}).some((g) => g > 0) || (s.adult ?? (!s.baby && !s.school));
  }

  /** Whether an object belongs in the garden rather than a room (outdoor-only, or its category or group says so). */
  belongsOutside(def: ObjectDef): boolean {
    if (def.outdoors) return true;
    const category = this.buyCategories.find((c) => c.id === def.category);
    return category?.groups?.find((g) => g.id === def.group)?.outside ?? category?.outside ?? false;
  }

  /** Decor points an object gives the room it stands in (sim-core `ObjectDef::decor`). */
  decorOf(def: ObjectDef): number {
    return def.decor ?? (def.category ? this.decorByCategory[def.category] : undefined) ?? 0;
  }

  /** Whether an object's interactions offer any of `tags`. */
  offersTags(def: ObjectDef, tags: readonly string[]): boolean {
    return def.interactions.some((it) => (it.tags ?? []).some((t) => tags.includes(t)));
  }

  /** Whether an object's interactions offer an activity (with a skill to train, if given). */
  offers(def: ObjectDef, activity: ActivityDef, skill?: string | null): boolean {
    return def.interactions.some((it) =>
      activity.skill && skill ? (it.skills?.[skill] ?? 0) > 0 : (it.tags ?? []).some((t) => activity.tags?.includes(t)),
    );
  }

  skill(id: string): SkillDef | undefined {
    return this.skills.find((s) => s.id === id);
  }

  careerCategory(id: string | null | undefined): CareerCategory | undefined {
    return id ? this.careerCategories.find((c) => c.id === id) : undefined;
  }

  object(id: string): ObjectDef | undefined {
    return this.objects.get(id);
  }

  trait(id: string): TraitDef | undefined {
    return this.traits.find((t) => t.id === id);
  }

  gender(id: string): GenderDef | undefined {
    return this.genders.find((g) => g.id === id);
  }

  /** First names that suit a gender (all first names when the content has no list for it). */
  firstNames(gender: string): readonly string[] {
    const list = this.names.byGender?.[gender];
    return list && list.length ? list : this.names.first;
  }

  emotion(id: string | null): EmotionDef | undefined {
    return id ? this.emotions.find((e) => e.id === id) : undefined;
  }

  social(index: number): SocialDef | undefined {
    return this.socials[index];
  }

  perk(id: string): PerkDef | undefined {
    return this.perks.find((p) => p.id === id);
  }

  perkCost(perks: readonly string[]): number {
    return perks.reduce((sum, id) => sum + (this.perk(id)?.cost ?? 0), 0);
  }

  /** Why `traitId` can't be added to `traits`, or null if it can. */
  traitBlocker(traits: readonly string[], traitId: string): string | null {
    if (traits.includes(traitId)) return null;
    const conflict = traits.find((t) => this.trait(t)?.conflicts?.includes(traitId) || this.trait(traitId)?.conflicts?.includes(t));
    if (conflict) return `Conflicts with ${this.trait(conflict)?.label ?? conflict}`;
    if (traits.length >= this.rules.maxTraits) return `Up to ${this.rules.maxTraits} traits`;
    return null;
  }

  /** Mirrors `Content::character_modifiers` in sim-core. Returns a list of problems. */
  validateCharacter(traits: readonly string[], perks: readonly string[]): string[] {
    const problems: string[] = [];
    const { minTraits, maxTraits, perkPoints } = this.rules;
    if (traits.length < minTraits) problems.push(`Pick at least ${minTraits} trait${minTraits === 1 ? '' : 's'}`);
    if (traits.length > maxTraits) problems.push(`Pick at most ${maxTraits} traits`);
    for (const t of traits) {
      const blocker = this.traitBlocker(traits.filter((x) => x !== t), t);
      if (blocker?.startsWith('Conflicts')) problems.push(`${this.trait(t)?.label}: ${blocker.toLowerCase()}`);
    }
    if (this.perkCost(perks) > perkPoints) problems.push(`Perks cost more than ${perkPoints} points`);
    return problems;
  }
}

type Json = Record<string, unknown>;

const isJsonObject = (v: unknown): v is Json => typeof v === 'object' && v !== null && !Array.isArray(v);
const jsonKind = (v: unknown) => (Array.isArray(v) ? 'array' : isJsonObject(v) ? 'object' : 'value');
/**
 * Content keys renamed when moodlets became feelings: [old, new]. Kept so content packs written
 * before the rename still load. Same list as `LEGACY_KEYS` in sim-core's `pack.rs`.
 */
const LEGACY_KEYS: [string, string][] = [
  ['moodlets', 'feelings'],
  ['moodlet', 'feeling'],
  ['moodletMinSkill', 'feelingMinSkill'],
  ['targetMoodlet', 'targetFeeling'],
  ['winnerMoodlet', 'winnerFeeling'],
  ['loserMoodlet', 'loserFeeling'],
  ['jealousyMoodlet', 'jealousyFeeling'],
  ['heartbreakMoodlet', 'heartbreakFeeling'],
  ['promotionMoodlet', 'promotionFeeling'],
  ['missedMoodlet', 'missedFeeling'],
  ['debtMoodlet', 'debtFeeling'],
];

/** Renames `LEGACY_KEYS` anywhere in `value`, unless the new key is already there. */
function upgradeLegacyKeys(value: unknown): void {
  if (Array.isArray(value)) {
    value.forEach(upgradeLegacyKeys);
  } else if (isJsonObject(value)) {
    for (const [old, key] of LEGACY_KEYS) {
      if (old in value && !(key in value)) {
        value[key] = value[old];
        delete value[old];
      }
    }
    Object.values(value).forEach(upgradeLegacyKeys);
  }
}

/** `traitPatches` effect maps whose entries are set per key. */
const PATCH_MAPS = ['needDecay', 'needGain', 'tagPreference', 'tagAcceptance', 'tagSuccess', 'skillGain'];

/**
 * Merges content files in order (base first, then its includes). Mirrors
 * `sim_core::pack::merge_named` exactly; see docs/content-packs.md:
 * top-level arrays are appended (a repeated `id` is an error), objects are merged
 * shallowly, other values replaced, `traitPatches` are applied to the merged traits
 * last, and `include` / `$comment` are dropped. Keys from before the moodlet → feeling rename
 * are renamed first (`LEGACY_KEYS`). Only here: `voice` (spoken lines) and `talk` (the language
 * model's words), which sim-core drops, are merged at every depth, so packs add or replace
 * single lines (`mergeDeep`).
 */
export function mergeContent(files: [name: string, text: string][]): Json {
  const out: Json = {};
  const origin = new Map<string, string>();
  const patches: [string, unknown][] = [];
  for (const [name, text] of files) {
    let file: unknown;
    try {
      file = JSON.parse(text);
    } catch (e) {
      throw new Error(`${name}: invalid JSON: ${(e as Error).message}`);
    }
    if (!isJsonObject(file)) throw new Error(`${name}: a content file must be a JSON object`);
    upgradeLegacyKeys(file);
    for (const [key, value] of Object.entries(file)) {
      if (key === 'include' || key === '$comment') continue;
      if (key === 'traitPatches') {
        patches.push([name, value]);
        continue;
      }
      if (key === 'voice' || key === 'talk') {
        if (!isJsonObject(value)) throw new Error(`${name}: ${key} must be an object`);
        const prev = out[key];
        out[key] = mergeDeep(isJsonObject(prev) ? prev : {}, value);
        continue;
      }
      const prev = out[key];
      if (prev !== undefined && jsonKind(prev) !== jsonKind(value)) {
        throw new Error(`${name}: '${key}' has a different type than in the files before it`);
      }
      if (Array.isArray(value)) {
        for (const item of value) {
          if (!isJsonObject(item) || typeof item.id !== 'string') continue;
          const first = origin.get(`${key}\u0000${item.id}`);
          if (first !== undefined) throw new Error(`duplicate ${key} id '${item.id}' in ${first} and ${name}`);
          origin.set(`${key}\u0000${item.id}`, name);
        }
        out[key] = [...((prev as unknown[] | undefined) ?? []), ...value];
      } else if (isJsonObject(value)) {
        out[key] = { ...((prev as Json | undefined) ?? {}), ...value };
      } else {
        out[key] = value;
      }
    }
  }
  const setEntries = (target: Json, key: string, value: unknown, ctx: string) => {
    if (!isJsonObject(value)) throw new Error(`${ctx}.${key} must be an object`);
    if (!isJsonObject(target[key])) target[key] = {};
    Object.assign(target[key] as Json, value);
  };
  for (const [name, value] of patches) {
    if (!isJsonObject(value)) throw new Error(`${name}: traitPatches must be an object`);
    const traits = Array.isArray(out.traits) ? (out.traits as unknown[]) : [];
    for (const [id, patch] of Object.entries(value)) {
      const ctx = `${name}: traitPatches.${id}`;
      const target = traits.find((t) => isJsonObject(t) && t.id === id);
      if (!isJsonObject(target)) throw new Error(`${name}: traitPatches: unknown trait '${id}'`);
      if (!isJsonObject(patch)) throw new Error(`${ctx} must be an object`);
      for (const [k, v] of Object.entries(patch)) {
        if (k === 'startingSkills') {
          setEntries(target, k, v, ctx);
        } else if (k === 'effects') {
          if (!isJsonObject(v)) throw new Error(`${ctx}.effects must be an object`);
          if (!isJsonObject(target.effects)) target.effects = {};
          const effects = target.effects as Json;
          for (const [ek, ev] of Object.entries(v)) {
            if (PATCH_MAPS.includes(ek)) {
              setEntries(effects, ek, ev, `${ctx}.effects`);
            } else if (ek === 'mood' || ek === 'walkSpeed') {
              if (typeof ev !== 'number') throw new Error(`${ctx}.effects.${ek} must be a number`);
              const old = typeof effects[ek] === 'number' ? (effects[ek] as number) : ek === 'mood' ? 0 : 1;
              effects[ek] = ek === 'mood' ? old + ev : old * ev;
            } else {
              throw new Error(`${ctx}.effects: unknown key '${ek}'`);
            }
          }
        } else {
          throw new Error(`${ctx}: unknown key '${k}' (use effects or startingSkills)`);
        }
      }
    }
  }
  return out;
}

/**
 * `b` laid over `a`: objects merge key by key at every depth, anything else (a list of lines)
 * is replaced. Neither input changes.
 */
export function mergeDeep(a: Json, b: Json): Json {
  const out: Json = { ...a };
  for (const [key, value] of Object.entries(b)) {
    const prev = out[key];
    out[key] = isJsonObject(prev) && isJsonObject(value) ? mergeDeep(prev, value) : value;
  }
  return out;
}

/** A file's text: from the build's bundle (`assets/bundle.ts`) when it's there, else fetched. */
export async function fetchText(url: string): Promise<string> {
  const text = await bundled(url);
  if (text !== null) return text;
  const res = await fetch(withBuild(url));
  if (!res.ok) throw new Error(`failed to load ${url}: ${res.status}`);
  return res.text();
}
