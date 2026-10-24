/**
 * How residents move and what they play for what they do (`residents.ts` picks from these): the gaits and walking
 * in, the clips per action tag (`ACTIONS`, read by `cargo review` too) and the hands' poses, the things used from
 * inside or up against them (`STEP_IN`), rides, cuddles and carrying a baby, held instruments, a station's tools
 * and what is put down there. The data of the residents' state machine, engine-neutral.
 */

import { BOULDER, FOOS, FOOS_LANES, POOL, SMASH, YOGA_AT, foosClip } from './clips/games';
import { KNEEL_LANES, KNEEL_TABLE } from './clips/low';
import { BALL_PIT, PLAY_TENT } from './clips/play';
import { BOARD, CLOCK, CUPBOARD_OF, FEEDER, PUT_AWAY } from './clips/reach';
import { BUNK_DOWN, BUNK_UP, RIDE_HORSE, RIDE_ROUND, RIDE_SEESAW, RIDE_SPRING, RIDE_SWING, SLIDE } from './clips/rides';
import { SHOT } from './clips/shot';
import { CLIMB, CLIMB_LANES, FOUNTAIN, type Size, climbClip } from './clips/street';
import { EYEPIECE, PICK_APPLE, PICK_LEMON, PLATE_HOLD, SHOWER_CHILD } from './clips/uses';

/** Hip roll (rad) while walking, per body. */
export const GAIT_SWAY: Record<string, number> = { female: 0.07, male: 0.015 };
/** Standing heights (metres) before a Sim's own height factor. */
export const BASE_HEIGHT: Record<string, number> = { male: 1.77, female: 1.67 };
/** Larger jumps between frames snap instead of sliding. */
export const SNAP_DISTANCE = 1.6;
/**
 * A frame this long after the last (s): back from a hidden tab (it draws no frames) or a long stall.
 * The residents went on meanwhile, so each is drawn as if just come into view: one who sat down
 * meanwhile doesn't sit down again, and no walking speed is estimated over the gap (walkers would
 * run on the spot).
 */
export const STALL = 0.5;
/**
 * Walkers glide from snapshot to snapshot as far as they can walk between them (at speeds 4 and 5
 * a step covers metres; cutting corners of their path is the price): metres walked per tick
 * (`world::WALK_SPEED`), the fastest walkers' trait and perk factors (1.15 x 1.2 in the base game,
 * with room for packs) and some slack. Further than that, or not walking in both, is a jump.
 */
export const WALK_PER_TICK = 0.075;
export const WALK_FASTEST = 1.5;
export const WALK_SLACK = 1.2;
/**
 * The walk, jog and run cycles play matched to the ground speed, up to this many times their own pace
 * (faster, they would only flicker; the feet slide instead). A walker who would need the walk faster
 * jogs along (`jogAlong`), one who would need the jog faster runs (`runAlong`, game speeds 3 and up,
 * children from 2), each going back to the slower gait below `JOG_BACK` of its switch. The run, the
 * last, goes on to `RUN_MAX_RATE` (about six strides a second): adults and teens keep their feet on
 * the ground at speed 3 (15 m/s; children run at up to 13 m/s there), and at speeds 4 and 5 (45 and
 * 90 m/s) no gait could.
 */
export const GAIT_MAX_RATE = 3;
export const RUN_MAX_RATE = 4;
export const JOG_BACK = 0.8;
/** Locomotion cycles, slowest first (they play in real time at the matched rate, and change into each other in step). */
export const GEARS = ['walk', 'jogAlong', 'runAlong'];
export const GAITS = new Set(GEARS);
export const DEFAULT_FADE = 0.25;
/**
 * How far back (m) a Sim stands from the middle of the tile in front while using an object whose
 * clip opens something across that tile: a dishwasher's door drops 0.44 m into it; a sofa bed's
 * mattress, pulled out or folded away, reaches half a metre past it. Negative: steps in to a thing
 * smaller than its tile (the herb planters, 0.4–0.46 m deep, are picked from as from a counter), or to
 * the part used: the toes at a basin or a sink's cabinet, the hands over the water; up to the toaster and
 * the blender on a counter, the chemistry bench pushed back on its tile, onto the dumbbell rack's mat, the clock's trunk door, the
 * giving jar. Back from the statue, to look up at it. In beside the high chair, to feed the baby in it.
 */
export const STAND_BACK: Record<string, number> = {
  highChair: -0.5,
  dishwasher: 0.15,
  dishwasherCompact: 0.15,
  sofaBed: 1,
  'garden.herbs': -0.5,
  'foodie.herbGarden': -0.5,
  sink: -0.35,
  sinkPedestal: -0.5,
  sinkConsole: -0.45,
  sinkVessel: -0.45,
  vanityDouble: -0.45,
  farmhouseSink: -0.45,
  counterSinkDouble: -0.45,
  breakfastCounter: -0.3,
  chemistrySet: -0.35,
  dumbbells: -0.6,
  grandfatherClock: -0.6,
  'kind.givingJar': -0.4,
  townStatue: 0.8,
};
/**
 * Things hugged (the giant teddy, sitting on the floor): the hugger crouches down to it this far (m) from
 * the middle of the tile in front and wraps the arms round it
 * (the social hug's, `hugArms`), raised as far as their shoulders come down below where the arms go round
 * (`at`, m: the bear's neck).
 */
export const HUGGED: Record<string, { reach: number; at: number }> = { teddyBear: { reach: 0.58, at: 0.6 } };
/**
 * Things talked to, looked at or listened to (`talk`, `look`, `listen`) with the head level, not tipped down to
 * them: the telescope (its eyepiece clips), and the mirrors, looked into at eye height (a speech practised).
 */
export const LEVEL_GAZE = new Set(['telescope', 'mirror', 'mirrorWall', 'mirrorRound', 'mirrorFloor']);
/** Crouched (the `crouch` clip): the shoulders' height for a body's standing height. */
export const CROUCH_SHOULDER = 0.55;
/**
 * Things used from inside or right up against them, while sim-core stands their users on the tile in front
 * (`stepIn`): they step up to the object (walking) and use it there, and step back out to the tile when
 * done (`stepOut`). `by`: by the user's height (m, the tallest first), the clip played there instead of
 * the action's (none: the action's own, up at the part used), and where: the feet `to` m from the object's
 * middle (along its facing), `up` m above the ground (the clips in `street.ts` are fitted to the drinking
 * fountain and the climbing frame for each size), and at one of `lanes`, m to the object's right of its
 * middle: the one nearest where sim-core stood them (each in front of their own slot) no one else using it
 * has taken; with all of them taken, none: they use it from there. `sides`: used from its front or its back
 * (sim-core's `places`: the pool and foosball players across the table), `to` and `lanes` measured from the
 * side they stand on. `wait`: seconds into the use before stepping in (the phone box's door opening first; its
 * clip `delay`s so that they stand `back` m back from the tile's middle first, clear of the door), and
 * `out` seconds after the use before stepping
 * out (the door opening again, `out` in its motions). `perLane`: the clips are one per lane, not a sequence (each
 * lane's round goes its own way: the climbing frame's, round to its pole and back). `tag`: only for that use
 * (its action tag), the object's other uses as usual. `over`: a rim stepped over on the way in and out (`at` m
 * from the middle, `h` m high: the ball pit's).
 * `square`: they face the way the object faces, squared to it
 * at their lane, not turned to its middle as sim-core stands them (off to one side that turned the step in and the
 * hands at a foosball's handles 27° off).
 */
export interface StepIn {
  by: readonly (readonly [height: number, clip: readonly string[], at: { to: number; up?: number }])[];
  lanes?: readonly number[];
  sides?: boolean;
  wait?: number;
  back?: number;
  out?: number;
  perLane?: boolean;
  tag?: string;
  over?: { at: number; h: number };
  square?: boolean;
}
/** `StepIn.by` from a clip's sizes, each from its shortest resident up: the size's clip, or the clips `clips` plays for it. */
export const bySize = (sizes: readonly Size[], clips: (clip: string) => readonly string[] = (clip) => [clip]): StepIn['by'] => sizes.map(({ clip, min, fit }) => [min, clips(clip), fit] as const);
/** Eating a picked fruit (after the pick), bite by bite. */
export const EAT_FRUIT = ['eat', 'idle', 'eat', 'idle', 'eat', 'idle'];
export const STEP_IN: Record<string, StepIn> = {
  drinkingFountain: { by: bySize(FOUNTAIN) },
  // Onto its floor (the plinth, 12 cm), facing the phone at the back.
  phoneBooth: { by: [[0, ['phone'], { to: -0.04, up: 0.12 }]], wait: 1, back: 0.25, out: 0.25 },
  // Side by side on the net (0.9 m wide, from -0.75 m), each round over the deck and down the pole.
  climbingFrame: { by: bySize(CLIMB, (clip) => CLIMB_LANES.map((_, lane) => climbClip(clip, lane))), lanes: CLIMB_LANES, perLane: true },
  // A round from the run-out's foot: round to the ladder, up it, down the chute and back (`clips/rides.ts`).
  slide: { by: bySize(SLIDE) },
  // Games and sport up against the thing (`clips/games.ts`): bent over the pool table's long rail (two players, one
  // on each long side), at a pair of the foosball rods (one each side), at the plate stack and the crate, up on the bouldering wall's crash
  // mat, and on the yoga mat's front end, facing along it.
  poolTable: { by: bySize(POOL), lanes: [-0.55, 0.55], sides: true },
  foosball: { by: bySize(FOOS, (clip) => FOOS_LANES.map((_, lane) => foosClip(clip, lane))), lanes: FOOS_LANES, perLane: true, square: true, sides: true },
  'hotHeaded.smashBin': { by: bySize(SMASH) },
  'energetic.boulderingWall': { by: bySize(BOULDER), lanes: [0] },
  yogaMat: { by: [[0, ['yoga'], YOGA_AT]] },
  // Up to the board on the wall (its face 0.42 m behind the middle), writing on it at shoulder to head height
  // (clips/reach.ts); up to the wall cupboards over the worktop and the china cabinet, the doors opened and the
  // clean things put away up in them; to the clock's hood, winding it at the dial (a child at the trunk's chains).
  whiteboard: { by: bySize(BOARD) },
  ...Object.fromEntries(Object.entries(CUPBOARD_OF).map(([def, name]): [string, StepIn] => [def, { by: bySize(PUT_AWAY[name]), tag: 'fetch' }])),
  grandfatherClock: { by: bySize(CLOCK), tag: 'tinker' },
  // Out on the court (2.5 m from the hoop's middle, a lane each for two), shooting (clips/shot.ts).
  basketballHoop: { by: bySize(SHOT), lanes: [-0.45, 0.45], tag: 'exercise' },
  // Kneeling at the low train table's edge, the hands on its track and its little town, a lane each (clips/low.ts).
  trainTable: { by: bySize(KNEEL_TABLE), lanes: KNEEL_LANES, tag: 'play' },
  // Into the shelter at the timetable's end, on the back glass past the bench.
  busStop: { by: [[0, [], { to: 0.05 }]], lanes: [-0.95] },
  // At the feeder's post, at its back left, clear of the bird bath: its house filled from a scoop (clips/reach.ts).
  'kind.songbirdFeeder': { by: bySize(FEEDER), lanes: [-0.2], tag: 'garden' },
  // At the microwave on the counter's right, the hands at its door.
  microwave: { by: [[0, [], { to: 0.55 }]], lanes: [-0.2] },
  // At the mic stand, singing into it.
  'cheerful.karaokeMachine': { by: [[0, [], { to: 0.45 }]], lanes: [0.18] },
  // Up on the stage, each at a mic stand.
  'outgoing.karaokeStage': { by: [[0, [], { to: 0.36, up: 0.146 }]], lanes: [-0.36, 0.36] },
  // Bent down to the eyepiece of the tube aimed up (clips/uses.ts), from nearer than the tile.
  telescope: { by: bySize(EYEPIECE), tag: 'look' },
  // Picking the fruit hanging at the front (clips/uses.ts, fitted to where it hangs), then eating it; lemons are
  // picked for the kitchen (G-68), one after another.
  'garden.apple': { by: bySize(PICK_APPLE, (clip) => [clip, ...EAT_FRUIT]), tag: 'eat' },
  'garden.lemonTree': { by: bySize(PICK_LEMON), tag: 'harvest' },
  // Children's play (`clips/play.ts`): over the ball pit's rim into its middle, wading, jumping and sitting in the
  // balls; down at the play tent's door, crawling in to play on its mat and back out.
  ballPit: { by: bySize(BALL_PIT), over: BALL_PIT[0].fit.over },
  playTent: { by: bySize(PLAY_TENT) },
};
/** One clip played alone (`StepIn.perLane`), by name. */
export const LANE_SEQ = new Map<string, readonly string[]>();
/** How far (s) into someone else's round on the same thing one stepped in may still start theirs (`inTurn`). */
export const LANE_SYNC = 0.4;
/** Stepping in (m/s, game time) and out again (s). */
export const STEP_IN_SPEED = 1.05;
export const STEP_OUT_S = 1.1;
/**
 * Walking in (`Characters.walksIn`): a step of more than `min` m that sim-core makes a standing resident take in one
 * tick (onto a seat, into a shower, up to someone to talk to or the spot it stands them at) is walked, at `pace` of
 * the walk's own speed for the body, then they turn to the way sim-core faces them (up to `turn` s). Round a built-in
 * chair (`cornerOf`), to or from a tile behind it, by a corner `side` m to its side.
 */
export const WALK_IN = { min: 0.25, pace: 0.8, turn: 0.8, side: 0.55 };
/**
 * Showers with a door: seconds of standing while it slides open before stepping in (`in`), and when the use ends
 * before stepping out (`out`; `motions` `door_open`), through the opening (`door`: its middle, m to the shower's
 * right of its middle and ahead of it).
 */
export const WALK_WAIT: Record<string, { in: number; out: number; door: readonly [number, number] }> = { shower: { in: 0.7, out: 0.7, door: [0.21, 0.45] } };
/** Over a rim (`StepIn.over`): up on it from this far (m) either side. */
export const STEP_OVER = 0.3;
/**
 * Rides, sat astride with the hands on the grips whatever the use (`clips/rides.ts`): the clip for the rider's
 * height (`sizes`), in time with the ride's loop (`sync`: the swing's pumping legs). Where each rider sits and
 * faces is its slot's place (content `places`: the seesaw's two at its ends facing each other, the
 * merry-go-round's four round it facing in); each ride carries its rider with it (`ObjectMotion.ride`).
 */
export interface Ride {
  sizes: readonly { clip: string; min: number }[];
  sync?: boolean;
}
/** Rides sat astride, in a frame (`walksIn`: not walked onto). */
export const STRADDLED = new Set(['rockingHorse', 'springRider', 'seesaw']);
export const RIDES: Record<string, Ride> = {
  rockingHorse: { sizes: RIDE_HORSE },
  springRider: { sizes: RIDE_SPRING },
  seesaw: { sizes: RIDE_SEESAW },
  merryGoRound: { sizes: RIDE_ROUND },
  swingSet: { sizes: RIDE_SWING, sync: true },
};
/**
 * The upper bunk's ladder (`BUNKS`): getting into the upper bunk the sleeper climbs up it from the tile in front
 * of the foot end and lies down (`up`), and getting up climbs back down it to the tile (`down`: sim-core keeps
 * them there meanwhile, `LADDER_DOWN_TICKS`), the clip for their height (`clips/rides.ts`). Both are made in the
 * bed's frame, its upper slot facing its foot end (`Player.frame`), as sim-core puts the Sim in the bed, and
 * back on the tile, at once. Faded in from standing over `fade` s; to and from lying (the clips end and begin
 * in the lying pose) over `lie` s.
 */
export const BUNK_LADDER = { up: BUNK_UP, down: BUNK_DOWN, fade: 0.3, lie: 0.12 };
/** A baby in the crib plays its own lying clips (clips/baby.ts: legs drawn up, hands by the head), asleep or awake, at their own speed; crossfaded over `BABY_LIE_FADE` s when the baby falls asleep or wakes. */
export const BABY_LIE = { asleep: 'babyLieAsleep', awake: 'babyLieAwake' } as const;
export const BABY_LIE_FADE = 1.6;
/** Every lying clip (`lie`, and the baby's). */
export const LYING = new Set<string>(['lie', 'lieCurl', BABY_LIE.asleep, BABY_LIE.awake]);
/**
 * A cuddle at the crib (action `cuddle`): the grown-up steps up to the rail, leans over it and lifts
 * the baby out, holds them cradled in the arms (head in the crook of the left arm) rocking gently,
 * and near the end lays them back down; the sim keeps the baby in the crib throughout (render only).
 * `pick` / `put`: seconds of animation for picking up and putting back (a cuddle too short for them,
 * at speed, plays them faster); `rail`: the rail's distance from the baby lying in the crib; `reach`:
 * the feet's distance from it while reaching in, `clear` while holding the baby; `lean`: the bend
 * over the rail (rad, for a grown-up of `height` m; taller ones bend further), `arms` the arms'
 * angle forward of it.
 */
export const CUDDLE = { pick: 2.6, put: 2.4, rail: 0.25, reach: 0.22, clear: 0.52, lean: 0.9, height: 1.7, arms: 0.4 };
/**
 * Carrying a baby (sim-core `carry.rs`, the layout's `carry`): the grown-up lifts them out of the crib
 * as for a cuddle (`CUDDLE.pick`, fitted into the sim's `lift` game minutes), carries them cradled in
 * the arms on the way, and lays them from the arms into the high chair (or back in the crib) as the
 * cuddle puts them back (`CUDDLE.put`), in the next `lift` minutes.
 */
export const CARRY = { lift: 1.5 };
/**
 * Changing the baby at the crib (action `change`): the carer, standing at the changing table beside
 * the cot, lifts the baby out of the cot (as `CUDDLE`), lays them on the changing pad across it, the
 * head to the right, works at them there (the `change` clips) and near the end lifts them back into the
 * cot; render only, as the cuddle (the sim keeps the baby in the cot's slot). `pad`: the pad's top (m;
 * the cot's mattress is `BEDS.crib`), `back`: its middle behind the crib's middle (m, along its facing),
 * `turn`: the baby turned on the pad (rad, about the vertical).
 */
export const CHANGE = { pad: 0.865, back: -0.19, turn: -Math.PI / 2 };
/**
 * The held baby: its middle forward of the forearms' (body scale of the one holding them) and its
 * back above them (its own); its head end raised (rad) and turned towards the one holding them.
 */
export const CRADLE = { forward: 0.08, lift: -0.03, tilt: 0.22, roll: -0.55 };
/**
 * A held guitar (`writeHeld`): where its body rests, from the spine's base (m, body scale 1: out to the
 * front, to the right, up; `upShort` more for a short one, held higher), and how far up the instrument
 * that is (its share of the instrument's height). Instruments `short` m long and shorter (for the
 * player's size: a ukulele) are held as short, `long` and longer (a guitar) as long, between them in
 * between (`guitarReach`). The strumming clips are fitted to it (clips/uses.ts `GUITAR_BODY`): its neck goes
 * from there to the left hand, out to the left across the body.
 */
export const GUITAR = { out: 0.17, right: 0.08, up: -0.03, upShort: 0.12, body: 0.27, short: 0.7, long: 0.9 };
/**
 * Playing a guitar or a ukulele (clips/uses.ts: held across the body, the neck to the left; `strumShort`
 * for a short one, `guitarReach` under a half).
 */
export const STRUM: readonly string[] = ['strum'];
export const STRUM_SHORT: readonly string[] = ['strumShort'];
/** A use sequence of the idle alone (curling dumbbells on top, `activityPose`), and of pushing (a mop). */
export const IDLE_ONLY: readonly string[] = ['idle'];
export const PUSH_ONLY: readonly string[] = ['push'];
/** Seconds the towel stays on after a shower without a step out of it (`toweled`). */
export const TOWEL_OUT = 0.8;
/** Things stood on holding their grips (content `stand`): the vibration plate, its clip in a size per height (`PLATE_HOLD`). */
export const GRIPPED: Record<string, readonly { clip: string; min: number }[]> = { 'lazy.vibePlate': PLATE_HOLD };
/** Relaxing on foot (`ACTIONS.relax`): standing about, the arms folded now and then; at a fire (`WARM_AT`), the hands held out to it and rubbed. */
export const RELAX_STAND: readonly string[] = ['idle', 'foldArms'];
export const WARM_HANDS: readonly string[] = ['warmHands'];
export const WARM_AT = new Set(['firePit']);
/** Showering (`ACTIONS.shower`), a grown-up's and a child's. */
export const SHOWER_ONLY: readonly string[] = ['shower'];
export const SHOWER_SMALL: readonly string[] = ['showerChild'];
/** Crouched down (to hug the teddy, `HUGGED`). */
export const CROUCH_ONLY: readonly string[] = ['crouch'];
/**
 * Writing desks with their chair built in close, the top only about 0.3 m ahead of the hips (`SEATS`
 * sits the writer on that chair): written at with the hands nearer (`sitHandsNear`), on the top, not
 * through its front.
 */
export const NEAR_DESKS = new Set(['gloomy.journalDesk', 'neat.organizerDesk', 'romantic.letterBureau', 'hotHeaded.grudgeJournal']);
/** Ovens under a hob, near the floor: baking (`bake`) bends down to them. */
export const LOW_OVENS = new Set(['stoveElectric', 'stoveGas', 'rangeCooker', 'chefStove', 'kind.neighbourlyOven']);
/** Seated objects whose activity uses the hands on a desk / keys. */
export const SIT_HANDS = new Set(['computerDesk', 'piano']);
/** Standing object use: clip sequence (looped). */
export const USE: Record<string, readonly string[]> = {
  fridge: ['interact', 'eat', 'eat'],
  sink: ['interact'],
  shower: ['interact', 'idle'],
  tv: ['foldArms'],
  bookshelf: ['interact', 'phone', 'phone'],
  plant: ['water'],
  flowerbed: ['kneel'],
  treadmill: ['jog'],
  easel: ['interact'],
  mirror: ['talk'],
  workbench: ['interact', 'pickUp'],
  telescope: ['rail'],
  tree: ['idle'],
};
/**
 * Per finger and per joint (knuckle, middle, tip): share of the hand pose's opening (see `fingerMask`).
 * (`fingers` opens fully: the grip's fist is tight, and its open hands keep about the curl they had.)
 */
export const FINGER_OPEN: Record<string, number> = { index: 1, middle: 0.93, ring: 0.86, pinky: 0.8, thumb: 0.9, fingers: 1 };
export const JOINT_OPEN = [0.78, 0.9, 1];
/** Finger pose: 0 = the clip's grip, 1 = open hand (rest pose). */
export const HANDS = {
  grip: 0.1,
  hold: 0.35,
  relaxed: 0.6,
  /** Standing at rest: fingers loosely open along the thighs. */
  rest: 0.78,
  keys: 0.75,
  open: 0.85,
} as const;
/**
 * Animation per action tag (`layout.actions`, from the content's `animations`):
 * - `stand`: clip sequence while standing at the object (looped; looping clips play ~4 s per step);
 *   a function picks one by object def (and the resident's height);
 * - `sit`: seated clip (default `sit`), or a function of the object def;
 * - `upper`: upper-body layer while seated (`eat` and `drink` play now and then), or a function of the
 *   object def ('' for none);
 * - `hands`: finger pose, or a function of the object def.
 * Procedural layers on top (reclining, instruments, gazes, ...) are in `activityPose`.
 */
export interface ActionAnim {
  stand?: readonly string[] | ((def: string, height: number) => readonly string[]);
  sit?: string | ((def: string) => string);
  upper?: string | ((def: string) => string);
  hands?: number | ((def: string) => number);
}
/** Calls made to a screen (`phone`): talked to it, hands free, not with a phone at the ear. */
export const VIDEO_CALLS = new Set(['outgoing.videoCallNook', 'cheerful.partyBooth']);
/** Waist-high things tended standing, leaning in (the herb planter's `pick`): the greenhouse's staging, the herb trough, the compost bin. */
export const WAIST_HIGH = new Set(['greenhouse', 'foodie.herbGarden', 'compostBin']);
/** Crops picked down at the soil (strawberries, pumpkins, a meadow's flowers): the `plant` clip's crouch, not the bend over a bush. */
export const LOW_CROPS = new Set(['garden.strawberries', 'garden.pumpkins', 'garden.meadow']);
/** Fruit trees whose crop is picked reaching up into the crown (`pickHigh`): the apple tree's "Pick the apples" (G-68). */
export const FRUIT_TREES = new Set(['garden.apple']);
/** Flowers that bloom above about 0.7 m, smelt leaning in standing, a bloom held to the nose (`pick`), not crouched under. */
export const TALL_FLOWERS = new Set(['garden.cosmos', 'garden.flowerBox', 'garden.lavender', 'garden.lilies', 'garden.lupins', 'garden.roses', 'garden.wheelbarrow']);
/** Toys played with down on the floor at the box (`kneel`, as the play mat and the doll house). */
export const FLOOR_TOYS = new Set(['toyBox', 'toyChest']);
/** Wall cupboards (each with its putting away, `CUPBOARD_OF`): things put away up in them (no squat to the floor). */
export const WALL_CUPBOARDS = new Set(Object.keys(CUPBOARD_OF));
export const ACTIONS: Record<string, ActionAnim> = {
  eat: { stand: ['eat', 'idle'], upper: 'eat', hands: HANDS.hold },
  // Sips from a cup, spaced out.
  drink: { stand: ['drink', 'idle', 'idle'], upper: 'drink', hands: HANDS.hold },
  cook: {
    // (At an appliance on a counter, the microwave, the toaster: at it, not down at the floor.)
    stand: (def) => (def === 'microwave' || def === 'breakfastCounter' ? ['spell', 'interact'] : ['spell', 'interact', 'spell', 'pickUp']),
    sit: 'sitHands',
    hands: HANDS.hold,
  },
  // (Seated at the dressing table, getting ready: brushing the hair in the mirror, clips/self.ts.)
  wash: { stand: ['spell', 'interact'], upper: (def) => (def === 'dressingTable' ? 'brushHair' : ''), hands: (def) => (def === 'dressingTable' ? HANDS.grip : HANDS.relaxed) },
  // Washing the hair, turning round, soaping, rinsing (clips/uses.ts; a child turns only the chest).
  shower: { stand: (_, height) => (height < SHOWER_CHILD ? SHOWER_SMALL : SHOWER_ONLY), hands: HANDS.open },
  bath: { hands: HANDS.open },
  toilet: { hands: HANDS.relaxed },
  sleep: { hands: HANDS.open },
  nap: { hands: HANDS.open },
  sit: { hands: HANDS.open },
  // (At a fire on foot, warming the hands at it: `WARM_AT`.)
  relax: { stand: (def) => (WARM_AT.has(def) ? WARM_HANDS : RELAX_STAND), hands: HANDS.open },
  type: { stand: ['spell'], sit: 'sitHands', hands: HANDS.keys },
  write: { stand: ['interact', 'spell'], sit: (def) => (NEAR_DESKS.has(def) ? 'sitHandsNear' : 'sitHands'), hands: HANDS.hold },
  // (A notice board or a timetable is read looking at it, not at a book in the hands.)
  read: { stand: (def) => (def === 'noticeBoard' || def === 'busStop' ? ['idle'] : ['lantern']), upper: 'lantern', hands: HANDS.hold },
  watch: { stand: ['foldArms', 'idle'], hands: HANDS.open },
  // Darts thrown and the table-tennis bat swung (mimed, closed hands: `clips/games.ts`); the pool table and the foosball
  // rods have their clips in `STEP_IN`; toys down on the floor at the box; crouched at the low train table, the hands at
  // its track; other games (the arcade's panel) the hands busy at it.
  play: {
    stand: (def) =>
      def === 'dartboard' ? ['dartThrow'] : def === 'pingPongTable' ? ['pingPong'] : FLOOR_TOYS.has(def) ? ['kneel'] : def === 'trainTable' ? ['crouch'] : ['spell'],
    sit: 'sitHands',
    hands: (def) => (def === 'dartboard' || def === 'pingPongTable' || def === 'poolTable' || def === 'foosball' ? HANDS.grip : HANDS.keys),
  },
  // Keys and drums on the desk-hands clip; a cello is held upright between the knees.
  music: { stand: ['spell'], sit: (def) => (def.includes('cello') ? 'sit' : 'sitHands'), hands: HANDS.keys },
  paint: { stand: ['spell', 'interact', 'spell'], hands: HANDS.hold },
  // (The plate-smash bin, the bouldering wall and the yoga mat have their clips in `STEP_IN`.)
  exercise: {
    stand: (def) =>
      def.includes('Bag')
        ? ['punchJab', 'punchCross']
        : def.includes('fitnessGame')
          ? ['dance', 'punchJab', 'punchCross']
          : def.includes('streetball') || def === 'basketballHoop'
            ? ['pickUp', 'interact']
            : // Stretching (and, `choose`, bouncing and vibrating): the idle with `activityPose` on top.
              def.includes('sunrise')
              ? ['idle']
              : ['jog'],
    sit: 'sitHands',
    hands: (def) => (def === 'yogaMat' ? HANDS.open : HANDS.grip),
  },
  run: { stand: ['jog'], hands: HANDS.grip },
  lift: { stand: ['pickUp'], hands: HANDS.grip },
  dance: { stand: ['dance'], hands: HANDS.relaxed },
  talk: { stand: ['talk'], hands: HANDS.relaxed },
  // At the mic stand (`STEP_IN`), the hands free.
  sing: { stand: ['talk'], hands: HANDS.relaxed },
  clean: {
    stand: (def) => (def.includes('Mop') || def.includes('Vacuum') ? ['push'] : def.includes('laundry') ? ['spell', 'interact', 'pickUp'] : ['interact', 'spell']),
    hands: HANDS.hold,
  },
  // Gardening, by task (older content and packs: `garden`).
  water: { stand: ['water'], hands: HANDS.hold },
  // Kneeling at the soil; a waist-high thing leant over standing, the staked tomatoes tied up standing, a
  // raised bed's soil bent over.
  tend: {
    stand: (def) =>
      WAIST_HIGH.has(def) ? ['pick'] : def === 'garden.tomatoes' ? ['harvest', 'water'] : def === 'raisedBed' || def === 'natureLover.veggieBed' ? ['harvest'] : ['kneel'],
    hands: HANDS.relaxed,
  },
  // The baby picked up out of the crib and held (`CUDDLE`).
  cuddle: { stand: ['idle'], hands: HANDS.relaxed },
  // Spoon-feeding the baby in the high chair, from beside it: the hand out to the baby's mouth and back.
  feed: { stand: ['interact', 'idle', 'interact', 'spell'], hands: HANDS.hold },
  // At the changing table, the baby on the pad (`CHANGE`): the hands busy over them.
  change: { stand: ['interact', 'spell'], hands: HANDS.relaxed },
  // (From the waist-high herb planter, standing; low crops crouched down to the soil.)
  // (Fruit picked up in a tree's crown, `pickHigh`.)
  harvest: {
    stand: (def) => (def === 'garden.herbs' ? ['pick'] : FRUIT_TREES.has(def) ? ['pickHigh'] : LOW_CROPS.has(def) ? ['plant', 'kneel'] : ['harvest', 'plant']),
    hands: HANDS.hold,
  },
  prune: { stand: ['interact', 'spell', 'interact'], hands: HANDS.grip },
  garden: {
    // (The bird feeder's box on its post: reached for standing at it.)
    stand: (def) =>
      def === 'plant' || def.includes('Terrarium') || def.includes('bonsai') ? ['water'] : def === 'kind.songbirdFeeder' ? ['interact', 'spell'] : ['harvest', 'plant'],
    hands: HANDS.hold,
  },
  tinker: {
    // (The longcase clock wound at its trunk door, its weights' chains: no stoop to the floor.)
    stand: (def) => (def === 'grandfatherClock' ? ['interact', 'spell'] : ['interact', 'spell', 'pickUp']),
    sit: 'sitHands',
    hands: HANDS.grip,
  },
  look: {
    // (At the telescope bent down to its eyepiece: `STEP_IN`.)
    stand: ['idle', 'foldArms'],
    hands: HANDS.relaxed,
  },
  // Down to low flowers, leaning in to tall ones / up into a tree's crown (`activityPose` turns the head).
  smell: { stand: (def) => (TALL_FLOWERS.has(def) ? ['pick'] : ['crouch']), hands: HANDS.relaxed },
  admire: { stand: ['idle', 'foldArms'], hands: HANDS.relaxed },
  phone: { stand: ['phone'], sit: (def) => (VIDEO_CALLS.has(def) ? 'sitTalk' : 'sit'), upper: (def) => (VIDEO_CALLS.has(def) ? '' : 'phone'), hands: HANDS.hold },
  // Texting and scrolling (content `phone.actions[].anim`): the phone held in front, the head bowed over it, the thumbs moving.
  phoneScreen: { stand: ['phoneText'], sit: 'sit', upper: 'phoneText', hands: HANDS.hold },
  listen: { stand: ['idle', 'foldArms'], hands: HANDS.open },
  meditate: { stand: ['idle'], hands: HANDS.open },
  // (A coin into the giving jar.)
  idle: { stand: (def) => (def === 'kind.givingJar' ? ['interact', 'idle'] : ['idle']), hands: HANDS.relaxed },
  // A visitor at the front door: a knock, then waiting.
  knock: { stand: ['interact', 'idle'], hands: HANDS.relaxed },
  // Cooking, step by step (content `recipes`): ingredients from the fridge, chopping on the board,
  // stirring the pan, turning things on the grill, putting a dish down.
  fetch: { stand: (def) => (WALL_CUPBOARDS.has(def) ? ['interact', 'idle'] : ['interact', 'pickUp', 'idle']), hands: HANDS.hold },
  chop: { stand: ['chop', 'interact', 'chop'], hands: HANDS.grip },
  stir: { stand: ['stir', 'interact', 'stir'], hands: HANDS.hold },
  flip: { stand: ['flip', 'interact', 'flip'], hands: HANDS.grip },
  serve: { stand: ['pickUp', 'idle'], hands: HANDS.hold },
  // Batter stirred in a mixing bowl, pastry made (the bowl is drawn on the counter).
  mix: { stand: ['stir', 'interact', 'stir'], hands: HANDS.hold },
  // A dish put in the oven or the microwave, then watched, bending to look in now and then: down to
  // a cooker's oven, at chest height to an oven tower's, a pizza oven's mouth or a microwave.
  bake: { stand: (def) => (LOW_OVENS.has(def) ? ['pickUp', 'idle', 'foldArms', 'idle'] : ['interact', 'idle', 'foldArms', 'idle']), hands: HANDS.relaxed },
};
/** The prop drawn on a station's top while a Sim works there (by action tag; `tool.*` models). Nothing for a bake: the dish is in the oven. */
export const STATION_PROPS: Record<string, string> = { chop: 'tool.board', stir: 'tool.pan', flip: 'tool.grillFood', mix: 'tool.mixingBowl' };
/**
 * Where on a station its prop goes, by object def and action tag (else the top's middle, a little towards the
 * cook): m to its right and forward of its middle (its own frame), and the height it stands at. The pan on the
 * burner the cooker's clip lights (its flame or hot ring); the grill station's chopping on its free counter, off
 * the grate.
 */
export const STATION_SPOTS: Record<string, Partial<Record<string, readonly [x: number, z: number, y: number]>>> = {
  stoveGas: { stir: [-0.26, 0.02, 0.9] },
  // (No ring lights on it: the front one on the side of the knob that turns.)
  stoveElectric: { stir: [-0.16, -0.02, 0.9] },
  rangeCooker: { stir: [0, 0.03, 0.915] },
  cooktopCounter: { stir: [0.16, -0.22, 0.955] },
  grillStation: { chop: [0.6, -0.05, 0.905] },
};
/**
 * Station steps worked with the hands free (by action tag): what the cook carried in (the ingredients, the prepped
 * bowl) is put down on the station's top beside its tool while chopping, mixing, stirring or grilling, and picked up
 * again to move on; for a bake it's in the oven, not drawn.
 */
export const SET_DOWN: Record<string, 'top' | 'in'> = { chop: 'top', mix: 'top', stir: 'top', flip: 'top', bake: 'in' };
/** How far (m) to the side of the station's tool a thing put down beside it stands. */
export const SET_DOWN_SIDE = 0.3;
/** How far (m, body scale) below the palms' centres a carried thing's base rests. */
export const HOLD_DROP = 0.02;
/** The palm centre's offset from the wrist (left hand, body scale) for sets without `grip`. */
export const DEFAULT_GRIP: readonly [number, number, number] = [-0.023, -0.062, 0];
/** Looping clips in a use sequence play this long (s) per step. */
export const LOOP_STEP = 4;

