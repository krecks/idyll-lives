/**
 * Sims as skinned, animated characters (bodies, faces, hair and clothes generated in code by
 * `tools/characters/prim`, clips made in code in `characters/clips`; format in
 * docs/characters-format.md).
 *
 * GPU / performance model:
 * - One mesh per (body, part) — body, eyes, brows, lashes, teeth, tongue, each hairstyle and each garment
 *   (`top.*`, `bottom.*`, `shoes.*`) — drawn with thin instances (one per Sim wearing it).
 * - Skinning runs on the GPU through Babylon's baked-vertex-animation path: a float texture holds
 *   one row per Sim (all bone matrices, plus appearance texels), and each instance reads "its"
 *   row (start frame = end frame = row, speed 0). The CPU poses every visible Sim once per frame
 *   (clip sampling, crossfades, retargeting, look-at, face) and uploads the texture in one write.
 *   Only buffer contents change per frame (bone texture, contact-shadow / ring / blanket instance
 *   matrices), never meshes. (WebGPU snapshot replays don't pick up the texture writes, so the game
 *   draws live; see `scheduleSnapshot` in BabylonRenderer.)
 * - Clothes are painted on the body by its shader (each garment's cut over the body's fields);
 *   shells (`top.*`, `bottom.*`, `shoes.*` parts) are added only where the silhouette needs them.
 * - Small face parts (teeth, tongue, lashes, brows) are drawn only on faces big enough on screen to
 *   show them, and faces too small to show expressions skip their morphs (`faceDetail`); only the
 *   instance lists change per frame.
 * - Far from the camera a Sim switches to the low-detail body and shells (`lod.*` parts; with a gap
 *   between switching down and up), and every Sim casts its sun shadow from shadow-only copies of
 *   those (`shadowMeshes`, on `LAYER_SHADOW_ONLY`).
 * - Life stages load the first time a Sim needs them (`loadStage`).
 * - No allocations per frame.
 *
 * Animation: a small state machine per Sim, driven by the snapshot: pose, moving, conversation
 * fields, and what the Sim is doing (`object` + `action` tag, mapped to clips, upper-body layers and
 * hand poses in `ACTIONS`; older layouts fall back to guessing from the object the Sim stands at).
 * The clips stand, sit and lie as residents should, so the runtime layers on only what depends on
 * the moment: idle with variations, walk (playback matched to the ground speed, so feet don't
 * slide), sit (fitted to each seat's height and depth, `SEATS`), lie (`lie` turned flat, under a
 * blanket in beds), talk / listen / laugh / argue / fight / hug / kiss, object use (cooking steps
 * have clips of their own), carrying (an upper-body clip, the thing on the palms), a baby picked up
 * out of the crib for a cuddle (posed in the arms of the one holding them, `cuddles`) and procedural
 * touches per activity (`activityPose`). States crossfade over ~0.25 s; turns are smoothed.
 *
 * Face: face morphs (ARKit face units and visemes, mixed per Sim in `characters/face.ts` and added
 * on the GPU before skinning) plus eye bones: irregular blinks, eyes on the conversation partner or
 * now and then the camera with the lids following, expressions from the snapshot `emotion`, and
 * the mouth speaking the voice line's visemes (`setSpeech`), or babbling without one (`animateFace`).
 */

import {
  Bone,
  Buffer,
  Color3,
  Constants,
  DynamicTexture,
  MeshBuilder,
  StandardMaterial,
  Mesh,
  PBRMaterial,
  RawTexture,
  Skeleton,
  Matrix,
  Quaternion,
  Vector3,
  VertexData,
  type AbstractMesh,
  type Engine,
  type Scene,
  type WebGPUEngine,
} from './core';
import { BakedVertexAnimationManager } from '@babylonjs/core/BakedVertexAnimation/bakedVertexAnimationManager';
import type { AssetRegistry } from '../../assets/registry';
import type { Content, ItemDef, ObjectDef } from '../../content/content';
import type { FrameState } from '../../core/bridge';
import { colorsKey, objectCentre, storeyShown, type ObjectPlacement, type PartColors, type SimInfo, type WorldStructure } from '../../core/protocol';
import { Pose } from '../../core/snapshot';
import { loadCharacterSet, loadStage, type BodyData, type CharacterSet, type Clip } from './characters/data';
import { APPEARANCE_TEXELS, CharacterPlugin, MORPH_TEXELS, type CharacterSurface, type MorphBinding } from './characters/material';
import { FACE_KEYS, FaceMixer, NEUTRAL, type Face } from './characters/face';
import { ENERGY_RATE, type VisemeTrack } from '../../voice/visemes';
import { outfitFor, writeOutfit, type OutfitParams } from './characters/outfit';
import { BLANKET_WIDTH, blanketMaterial, buildBlanket, type BlanketShape } from './characters/blanket';
import { CLIMB, CLIMB_LANES, climbClip, FOUNTAIN } from './characters/clips/street';
import { furthestAhead, nearestAhead, type TableRoom } from './tableTuck';
import { addLocal, addRotation, blendInto, makeAdditive, PoseScratch, qmul, resetAdditive, rotate, sampleClip, skin, type Additive, type Placement } from './characters/pose';
import type { MaterialLibrary } from './materials';
import { WALL_HEIGHT } from './house';
import { LAYER_SHADOW_ONLY } from './layers';
import { buildModel, ITEM_STAND_IN, onModelFileArrived, paintColors, painted, type ModelTemplate } from './models';
import { TemplateCache } from './templateCache';

/** Rows in the pose texture (= Sim capacity). */
export const MAX_CHARACTERS = 64;
/** Hip roll (rad) while walking, per body. */
const GAIT_SWAY: Record<string, number> = { female: 0.07, male: 0.015 };

/** Life stages without facial hair. */
const YOUNG_STAGES = new Set(['baby', 'child', 'teen']);
/** Parts every Sim has (besides the body's skin, hair and clothes). */
const FACE_PARTS: readonly string[] = ['eyes', 'brows', 'lashes', 'teeth', 'tongue'];

/** The body's skin chunks a Sim wearing `parts` shows (those no worn garment hides completely). */
function skinChunks(body: BodyData, parts: readonly string[]): string[] {
  return Object.entries(body.bodyChunks)
    .filter(([, hiders]) => !hiders.some((h) => parts.includes(h)))
    .map(([chunk]) => chunk);
}
/** Standing heights (metres) before a Sim's own height factor. */
const BASE_HEIGHT: Record<string, number> = { male: 1.77, female: 1.67 };
/** Snapshot animation codes (see `social::ANIMATIONS` in sim-core). */
const Anim = {
  None: 0,
  Talk: 1,
  Laugh: 2,
  Flirt: 3,
  Argue: 4,
  Fight: 5,
  Hug: 6,
  Kiss: 7,
  Wave: 8,
} as const;
/** Sim ticks per frame step at each game speed (`clock::TICKS_PER_STEP`). */
export const GAME_RATE = [0, 1, 3, 10, 30, 60];
/** Speed while time skips through quiet hours (`clock::AUTO_FAST_SPEED`). */
export const QUIET_SPEED = 4;
/** Larger jumps between frames snap instead of sliding. */
const SNAP_DISTANCE = 1.6;
/**
 * Walkers glide from snapshot to snapshot as far as they can walk between them (at speeds 4 and 5
 * a step covers metres; cutting corners of their path is the price): metres walked per tick
 * (`world::WALK_SPEED`), the fastest walkers' trait and perk factors (1.15 x 1.2 in the base game,
 * with room for packs) and some slack. Further than that, or not walking in both, is a jump.
 */
const WALK_PER_TICK = 0.075;
const WALK_FASTEST = 1.5;
const WALK_SLACK = 1.2;
/**
 * The walk and jog cycles play matched to the ground speed, up to this many times their own pace
 * (faster, they would only flicker; the feet slide instead). A walker who would need the walk
 * faster jogs along (`jogAlong`), and walks again below `JOG_BACK` of it.
 */
const GAIT_MAX_RATE = 3;
const JOG_BACK = 0.8;
/** Locomotion cycles (they play in real time at the matched rate, and change into each other in step). */
const GAITS = new Set(['walk', 'jogAlong']);
const DEFAULT_FADE = 0.25;

/**
 * Seat surface height (m) and forward offset of the hips from the object's centre, per object def;
 * `back`: the sitter faces the object's back (a dressing table's mirror against the wall), `fwd` still
 * towards its front; `recline`: leant back against a slanted back (rad, `Seat.recline`); `along`, `legsOut`:
 * as `Seat`'s.
 */
const SEATS: Record<string, Seat> = {
  sofa: { h: 0.44, fwd: -0.087 },
  armchair: { h: 0.42, fwd: -0.013 },
  bench: { h: 0.45, fwd: -0.03 },
  toilet: { h: 0.43, fwd: -0.13 },
  // Chairs at desks and tables sit in the back half of the footprint, facing the work surface.
  chessTable: { h: 0.46, fwd: -0.2 },
  computerDesk: { h: 0.44, fwd: -0.17 },
  diningTable: { h: 0.46, fwd: -0.19 },
  // Content packs.
  'gloomy.soakingTub': { h: 0.25, fwd: 0 },
  'romantic.heartTub': { h: 0.35, fwd: -0.36 },
  'hotHeaded.coldPlunge': { h: 0.35, fwd: 0 },
  'hotHeaded.coolDownCushion': { h: 0.4, fwd: 0 },
  // Their own chairs at the writing desks (`NEAR_DESKS`): the hands on the top or the flap, in front of the hutch.
  'gloomy.journalDesk': { h: 0.46, fwd: 0 },
  'neat.organizerDesk': { h: 0.45, fwd: -0.08 },
  'romantic.letterBureau': { h: 0.45, fwd: -0.06 },
  'hotHeaded.grudgeJournal': { h: 0.46, fwd: -0.03 },
  // The 0.20 catalogue (content/furniture.json).
  barStool: { h: 0.72, fwd: 0 },
  bistroSet: { h: 0.46, fwd: -0.19 },
  diningGlass: { h: 0.46, fwd: -0.22 },
  diningOak: { h: 0.46, fwd: -0.22 },
  diningChair: { h: 0.46, fwd: 0 },
  loveseat: { h: 0.44, fwd: -0.07 },
  sofaThree: { h: 0.44, fwd: -0.09 },
  sofaDesign: { h: 0.42, fwd: -0.07 },
  armchairLounge: { h: 0.42, fwd: -0.04 },
  armchairLeather: { h: 0.44, fwd: -0.02 },
  armchairDesign: { h: 0.45, fwd: -0.013 },
  pouf: { h: 0.46, fwd: 0 },
  // On the floor cushion in front of the fire, facing it.
  fireplaceElectric: { h: 0.14, fwd: 0.3, back: true },
  dressingTable: { h: 0.45, fwd: 0.22, back: true },
  toiletCompact: { h: 0.43, fwd: -0.13 },
  toiletSmart: { h: 0.44, fwd: -0.06 },
  // In the bath, along the tub: the back at the end without the taps, the legs out along it towards them.
  bathtub: { h: 0.25, fwd: -0.42, along: -1, legsOut: true },
  bathtubClawfoot: { h: 0.28, fwd: -0.38, along: -1, legsOut: true },
  bathtubFreestanding: { h: 0.27, fwd: -0.4, along: 1, legsOut: true },
  hotTubCedar: { h: 0.35, fwd: 0 },
  rockingHorse: { h: 0.56, fwd: 0 },
  kidsDesk: { h: 0.36, fwd: -0.2 },
  deskWriting: { h: 0.44, fwd: -0.17 },
  deskLaptop: { h: 0.44, fwd: -0.17 },
  computerGaming: { h: 0.44, fwd: -0.17 },
  officeChair: { h: 0.47, fwd: -0.04 },
  keyboardDigital: { h: 0.45, fwd: -0.2 },
  sewingTable: { h: 0.46, fwd: -0.22 },
  patioSet: { h: 0.44, fwd: -0.62 },
  picnicTable: { h: 0.4, fwd: -0.68 },
  // Against its back, slanted 22 degrees (as the Adirondack chair's, content `seat.recline`).
  gardenChair: { h: 0.4, fwd: -0.083, recline: (18 * Math.PI) / 180 },
  swingSet: { h: 0.5, fwd: 0 },
};
const DEFAULT_SEAT: Seat = { h: 0.45, fwd: 0 };

/**
 * How far back (m) a Sim stands from the middle of the tile in front while using an object whose
 * clip opens something across that tile: a dishwasher's door drops 0.44 m into it; a sofa bed's
 * mattress, pulled out or folded away, reaches half a metre past it. Negative: steps in to a thing
 * smaller than its tile (the herb planter, 0.46 m deep, is picked from as from a counter).
 */
const STAND_BACK: Record<string, number> = { dishwasher: 0.15, dishwasherCompact: 0.15, sofaBed: 1, 'garden.herbs': -0.5 };
/**
 * Things hugged (the giant teddy, sitting on the floor): the hugger crouches down to it this far (m) from
 * the middle of the tile in front and wraps the arms round it
 * (the social hug's, `hugArms`), raised as far as their shoulders come down below where the arms go round
 * (`at`, m: the bear's neck).
 */
const HUGGED: Record<string, { reach: number; at: number }> = { teddyBear: { reach: 0.58, at: 0.6 } };
/** Crouched (the `crouch` clip): the shoulders' height for a body's standing height. */
const CROUCH_SHOULDER = 0.55;

/**
 * Things used from inside or right up against them, while sim-core stands their users on the tile in front
 * (`stepIn`): they step up to the object (walking) and use it there, and step back out to the tile when
 * done (`stepOut`). `by`: by the user's height (m, the tallest first), the clip played there instead of
 * the action's, and where: the feet `to` m from the object's middle (along its facing), `up` m above the
 * ground (the clips in `street.ts` are fitted to the drinking fountain and the climbing frame for each
 * size), and at one of `lanes`, m to the object's right of its middle: the first no one else using it has
 * taken (sim-core stands everyone using it on the same tile in front, of a 2-wide thing the one with the
 * lower x or z: the climbing frame's net side or its pole's, by its facing). `wait`: seconds into the use
 * before stepping in (the phone box's door opening first; its clip `delay`s so that they stand `back` m
 * back from the tile's middle first, clear of the door), and `out` seconds after the use before stepping
 * out (the door opening again, `out` in its motions). `perLane`: the clips are one per lane, not a sequence (each
 * lane's round goes its own way: the climbing frame's, round to its pole and back).
 */
interface StepIn {
  by: readonly (readonly [height: number, clip: readonly string[], at: { to: number; up?: number }])[];
  lanes?: readonly number[];
  wait?: number;
  back?: number;
  out?: number;
  perLane?: boolean;
}
const STEP_IN: Record<string, StepIn> = {
  drinkingFountain: { by: FOUNTAIN.map(({ clip, min, fit }) => [min, [clip], fit] as const) },
  // Onto its floor (the plinth, 12 cm), facing the phone at the back.
  phoneBooth: { by: [[0, ['phone'], { to: -0.04, up: 0.12 }]], wait: 1, back: 0.25, out: 0.25 },
  // Side by side on the net (0.9 m wide, from -0.75 m), each round over the deck and down the pole.
  climbingFrame: { by: CLIMB.map(({ clip, min, fit }) => [min, CLIMB_LANES.map((_, lane) => climbClip(clip, lane)), fit] as const), lanes: CLIMB_LANES, perLane: true },
};
/** One clip played alone (`StepIn.perLane`), by name. */
const LANE_SEQ = new Map<string, readonly string[]>();
/** Stepping in (m/s, game time) and out again (s). */
const STEP_IN_SPEED = 1.05;
const STEP_OUT_S = 1.1;

/**
 * A chair pulled up to a table (`tableTuck.ts`): once seated, the sitter shuffles it in (and pushes it back
 * rising) until the top's near edge is `edge` m ahead of the hips (for a body of the reference height: the
 * desk-hands clip's hands then rest on the top), as far as the room under the top lets the knees (`knees` m
 * ahead of the hips, up to `kneeUp` m over the seat, and not past the top's middle: someone may sit across)
 * and the toes (`toes` m, up to `toeUp` m off the floor) go, `clear` m short of what is there, and `max` m at
 * most. Not for a top more than `rise` m over the seat (an office chair at a standing desk). `rate`, `back`:
 * the shuffle's pace in and out (1/s, game time). The top still more than `far` m ahead of the hips then (a
 * drawer pedestal where the knees would go), the hands reach further (`sitHandsFar`).
 */
const TUCK = { edge: 0.24, far: 0.3, knees: 0.56, kneeUp: 0.11, toes: 0.62, toeUp: 0.12, clear: 0.02, max: 0.45, rise: 0.45, rate: 3, back: 4 };
/** The height (m) the seated clips are made for (their reaches scale with the body). */
const CLIP_HEIGHT = 1.77;

/**
 * Where a Sim sits on an object (`PlacedObject.seat`): the table above, else the content's `seat`
 * (with how they sit: facing the object's back, legs out along it, pedalling), else the default.
 */
function seatOf(def: ObjectDef | undefined, id: string): Seat {
  const own = def?.seat;
  if (SEATS[id] || !own) return SEATS[id] ?? DEFAULT_SEAT;
  const pedals = own.pedals ? { h: own.pedals.height, fwd: own.pedals.forward ?? 0, crank: own.pedals.crank } : undefined;
  const recline = own.recline !== undefined ? (own.recline * Math.PI) / 180 : undefined;
  return { h: own.height, fwd: own.forward ?? 0, back: own.facing === 'back', legsOut: own.legs === 'out', pedals, recline };
}
/** A held part's name and its colours from the end of its prop key (`<part>@<colorsKey>`, see `PlacedObject.held`). */
function partColors(ref: string): [string, PartColors?] {
  const at = ref.indexOf('@');
  if (at < 0) return [ref];
  return [ref.slice(0, at), Object.fromEntries(ref.slice(at + 1).split(',').map((p) => [p.slice(0, p.indexOf(':')), p.slice(p.indexOf(':') + 1)]))];
}
/** A seat (see `seatOf`): its height and the hips' forward offset (m, the object's frame), and how they sit. */
interface Seat {
  h: number;
  fwd: number;
  /** Facing the object's back (a dressing table's mirror against the wall, a pianist at an upright); `fwd` is still towards its front. */
  back?: boolean;
  /** Legs stretched out along the seat (a chaise). */
  legsOut?: boolean;
  /**
   * Facing along the object, a quarter turn from its front: towards its right (1) or its left (-1), `fwd`
   * then being along that way (a bath, sat in lengthways).
   */
  along?: 1 | -1;
  /** How far back (rad, from sitting up) the back leans just sitting, relaxing, napping or watching: against a slanted back (an Adirondack chair). Default: upright sitting, a little relaxing (`activityPose`). */
  recline?: number;
  /** Pedals the feet turn (an exercise bike): the crank's axle (height, forward of the middle) and its length (m). */
  pedals?: { h: number; fwd: number; crank: number };
}
/**
 * Mattress / bench top height (m) for lying, and how far forward of the footprint's middle the
 * sleepers lie (`fwd`, m): a sofa bed's mattress unfolds out in front of the sofa. `across`: lain on
 * along its width, head to its left, in its middle (a hammock slung side to side on a wide footprint).
 */
const BEDS: Record<string, Bed> = {
  bed: { h: 0.58 },
  crib: { h: 0.56 },
  weightBench: { h: 0.5 },
  'gloomy.soakingTub': { h: 0.25 },
  'romantic.heartTub': { h: 0.35 },
  'hotHeaded.coldPlunge': { h: 0.35 },
  'hotHeaded.coolDownCushion': { h: 0.2 },
  bedSingle: { h: 0.5 },
  bedBunk: { h: 0.44 },
  bedFuton: { h: 0.4 },
  bedKing: { h: 0.59 },
  kidsBed: { h: 0.44 },
  // The race car's mattress ends at its bonnet, 34 cm short of the footprint's end.
  kidsBedCar: { h: 0.5, foot: 0.37 },
  sunLounger: { h: 0.35 },
  // Hammocks: lain in the sagging sling (`Bed.sling`, from their models' builds).
  hammock: { h: 0.62, sling: { low: 0.6, end: 0.86, half: 0.66, head: 0.5 } },
  'cheerful.rainbowHammock': { h: 0.5, across: true, sling: { low: 0.49, end: 0.86, half: 0.755, head: 0.52 } },
  'natureLover.gardenHammock': { h: 0.5, across: true, sling: { low: 0.475, end: 0.86, half: 0.72, head: 0.54 } },
  wallBed: { h: 0.46 },
  // The oak double, the upholstered queen and the four-poster (their mattress tops), and the daybed.
  bedDouble: { h: 0.52 },
  bedQueenUpholstered: { h: 0.52 },
  bedCanopy: { h: 0.52 },
  bedDaybed: { h: 0.51 },
  // Head on the seat, the rest on the mattress pulled out in front (`bed_out`).
  sofaBed: { h: 0.42, fwd: 0.6 },
  // The 2-wide window seat and settee: dozed on along the cushion, head to its left, whichever slot (the nap
  // takes both, content `whole`).
  'gloomy.windowSeat': { h: 0.45, across: true },
  'kind.hearthSettee': { h: 0.44, across: true },
};
const DEFAULT_BED: Bed = { h: 0.5 };
/** What residents lie on (`BEDS`). */
interface Bed {
  h: number;
  fwd?: number;
  /** How far short of the footprint's foot end the quilt ends (m, default 0.05): a bed whose mattress stops short of it. */
  foot?: number;
  across?: boolean;
  /**
   * A hammock's sling, along the sleeper from its middle (m): its top's height there (`low`) and at its
   * ends (`end`, `half` each way, at the spreader bars; a parabola between), and how far towards the head
   * end the pillow is. The sleeper lies in it (`inSling`): hips in the dip, head raised onto the
   * pillow, knees up and the feet inside the spreader bar.
   */
  sling?: { low: number; end: number; half: number; head: number };
}
/** Bunk beds (content `bunk`): mattress tops of the lower and the upper bunk. */
const BUNKS: Record<string, readonly [number, number]> = { bedBunk: [0.44, 1.38] };
/** Beds whose sleepers get a blanket (pack hammocks, sofas and nap pods don't). */
const BLANKET_BEDS = new Set(['bed', 'lazy.snoozecloudBed', 'bedSingle', 'bedBunk', 'bedFuton', 'bedKing', 'bedDouble', 'bedQueenUpholstered', 'bedCanopy', 'kidsBed', 'kidsBedCar', 'wallBed']);
/**
 * Two sleepers' quilts in a wide bed (`writeBlanket`): each this much (m) wider on each side than half the bed
 * and moved this much towards its middle, so they overlap by four times it over the middle.
 */
const BLANKET_MEET = 0.06;
/**
 * Lying: height of the back above the feet line (body scale 1, for a grown-up's body: a smaller body's,
 * a child's, is thinner by its `size`), and how far the feet reach past the slot centre.
 */
const LIE_BACK = 0.1;
const LIE_CENTRE = 0.86;
/**
 * In a hammock (`Bed.sling`): the pillow's thickness (m), the ankles' height over the sling (m, body
 * scale 1), how far inside the spreader bar they stay (m), the most the upper body rises (rad), how far
 * the back lies above the sling's top (m, its thickness), and how far the head's joint lies short of the
 * pillow's middle (m: the back of the head on it).
 */
const SLING = { pillow: 0.07, ankle: 0.07, feet: 0.12, lift: 0.7, back: 0.02, crown: 0.08 };
/** A sling's top `d` m from its middle along the sleeper (a parabola from `low` to `end` at the bars). */
function slingTop(sling: NonNullable<Bed['sling']>, d: number): number {
  return sling.low + (sling.end - sling.low) * Math.min(1, (d / sling.half) ** 2);
}
/** Getting into bed: where the hips sit down, forward of the slot centre (body scale 1). */
const LIE_ENTRY = 0.55;
/** How far the feet reach past the slot centre for a lying Sim, and their back's height above the feet line: babies (much smaller) lie centred in the crib, on the mattress. */
function lieCentre(rig: SimRig): number {
  // (A smaller body than its family's base body, a child's, lies with its head where a grown-up's would be, on the pillow.)
  return (rig.body.stage === 'baby' ? rig.body.height * 0.5 : LIE_CENTRE - (rig.body.height / rig.size) * (1 - rig.size)) * rig.scale;
}
function lieBack(rig: SimRig): number {
  return (rig.body.stage === 'baby' ? LIE_BACK * 0.45 : LIE_BACK * rig.size) * rig.scale;
}
/** A baby in the crib plays its own lying clips (clips/baby.ts: legs drawn up, hands by the head), asleep or awake, at their own speed; crossfaded over `BABY_LIE_FADE` s when the baby falls asleep or wakes. */
const BABY_LIE = { asleep: 'babyLieAsleep', awake: 'babyLieAwake' } as const;
const BABY_LIE_FADE = 1.6;
/** Every lying clip (`lie`, and the baby's). */
const LYING = new Set<string>(['lie', BABY_LIE.asleep, BABY_LIE.awake]);
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
const CUDDLE = { pick: 2.6, put: 2.4, rail: 0.44, reach: 0.22, clear: 0.52, lean: 0.9, height: 1.7, arms: 0.4 };
/**
 * The held baby: its middle forward of the forearms' (body scale of the one holding them) and its
 * back above them (its own); its head end raised (rad) and turned towards the one holding them.
 */
const CRADLE = { forward: 0.08, lift: -0.03, tilt: 0.22, roll: -0.55 };

/**
 * Hugs and kisses (`Anim.Hug`, `Anim.Kiss`): the two step in from where sim-core stands them (about 0.55 m
 * apart, middle to middle) until their fronts meet. `chest`: how far the chest stands out in front of the
 * body's middle, as a share of its height (grown-ups' bodies: 0.13 m of 1.77 m; times its build); `kiss`: the
 * gap a kiss keeps between the chests (m; the faces meet, the bodies lean in); `most`: the furthest one steps
 * (m). `shoulder`: the shoulders' height, as a share of the body's; `lower`: how much (rad) a hugger lowers the
 * arms for each metre the other's shoulders are below theirs (a grown-up's round a child), down to `low`, and
 * `bend`: how far (rad) they bend down over them per metre. `wrap`: the elbows' bend round the other's back
 * (rad; `wrapSmall` more per metre lower, round a child's), with the hands behind it, not out past it;
 * `aside`: how far (rad) each leans the head to their own right, past the other's.
 */
const EMBRACE = { chest: 0.074, kiss: 0.03, most: 0.45, shoulder: 0.79, lower: 1.4, low: 0.35, bend: 0.9, wrap: 1.45, wrapSmall: 1.2, aside: 0.1 };
/** How far in front of its middle a body's chest stands out (m, `EMBRACE`). */
function chestDepth(rig: SimRig): number {
  return EMBRACE.chest * rig.body.height * rig.scale * rig.build;
}
/** How far (m) each of two stepping in to hug or kiss steps (`EMBRACE`), from the distance between them: until their chests meet (`gap` m apart). */
function embraceStep(rig: SimRig, partner: SimRig | null, gap: number): number {
  if (!partner) return 0;
  const d = Math.hypot(partner.x - rig.x, partner.z - rig.z);
  return clamp((d - chestDepth(rig) - chestDepth(partner) - gap) / 2, 0, EMBRACE.most);
}
/** How far (m) the other's shoulders are below a hugger's (0 when level or higher, `EMBRACE`). */
function shoulderDrop(rig: SimRig, partner: SimRig | null): number {
  if (!partner) return 0;
  return Math.max(0, (rig.body.height * rig.scale - partner.body.height * partner.scale) * EMBRACE.shoulder);
}

/**
 * Pedalling (`Characters.pedal`): a turn's length (s) when the object's loop isn't playing (the bike's
 * `pedals_turn`), and the ankle's place over the pedal's axle (m up and forward, body scale 1).
 */
const PEDAL = { turn: 1.2, ankle: [0.08, -0.09] as const };
/** Legs out along a seat (`Characters.legsOut`): the ankles this far below the hips' joints (m, body scale 1). */
const LEGS_OUT_DROP = 0.04;
/** How a held part is held (content `held.grip`; see `Characters.writeHeld`). */
type Grip = 'bar' | 'pair' | 'guitar' | 'pole';
/** A prop's meshes (one thin instance per Sim row) and its top (the highest point of a part, its own frame). */
interface Prop {
  meshes: Mesh[];
  matrices: Float32Array;
  top: readonly [number, number, number];
  /** The files it was built without (it's a small block then, `ModelTemplate.failed`). */
  failed?: string[];
}
/**
 * A held guitar (`writeHeld`): where its body rests, from the spine's base (m, body scale 1: out to the
 * front, to the right, up; `upShort` more for a short one, held higher), and how far up the instrument
 * that is (its share of the instrument's height). Instruments `short` m long and shorter (for the
 * player's size: a ukulele) are held as short, `long` and longer (a guitar) as long, between them in
 * between (`guitarReach`).
 */
const GUITAR = { out: 0.17, right: 0.08, up: -0.03, upShort: 0.12, body: 0.27, short: 0.7, long: 1.15 };
/**
 * The player's arms (rad, `activityPose`), for a long instrument and a short one: the left upper arm out
 * and up, its elbow bent and the forearm turned in to the neck; the right the same over the body.
 */
const GUITAR_ARMS = {
  long: { leftOut: -0.55, leftUp: -0.55, leftBend: -1.1, leftIn: 0.5, rightOut: 0.2, rightUp: -0.2, rightBend: -0.75, rightIn: -0.9 },
  short: { leftOut: -0.45, leftUp: -0.3, leftBend: -1.25, leftIn: 0.9, rightOut: 0.2, rightUp: -0.25, rightBend: -0.95, rightIn: -0.9 },
};
/** A use sequence of the idle alone (curling dumbbells and playing a guitar on top, `activityPose`), and of pushing (a mop). */
const IDLE_ONLY: readonly string[] = ['idle'];
const PUSH_ONLY: readonly string[] = ['push'];
/** Crouched down (to hug the teddy, `HUGGED`). */
const CROUCH_ONLY: readonly string[] = ['crouch'];
/**
 * Writing desks with their chair built in close, the top only about 0.3 m ahead of the hips (`SEATS`
 * sits the writer on that chair): written at with the hands nearer (`sitHandsNear`), on the top, not
 * through its front.
 */
const NEAR_DESKS = new Set(['gloomy.journalDesk', 'neat.organizerDesk', 'romantic.letterBureau', 'hotHeaded.grudgeJournal']);
/** Seated objects whose activity uses the hands on a desk / keys. */
const SIT_HANDS = new Set(['computerDesk', 'piano']);
/** Standing object use: clip sequence (looped). */
const USE: Record<string, readonly string[]> = {
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

/** Per finger and per joint (knuckle, middle, tip): share of the hand pose's opening (see `fingerMask`). */
const FINGER_OPEN: Record<string, number> = { index: 1, middle: 0.93, ring: 0.86, pinky: 0.8, thumb: 0.9, fingers: 0.92 };
const JOINT_OPEN = [0.78, 0.9, 1];

/** Finger pose: 0 = the clip's grip, 1 = open hand (rest pose). */
const HANDS = {
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
 *   a function picks one by object def;
 * - `sit`: seated clip (default `sit`), or a function of the object def;
 * - `upper`: upper-body layer while seated (`eat` and `drink` play now and then);
 * - `hands`: finger pose.
 * Procedural layers on top (reclining, instruments, gazes, ...) are in `activityPose`.
 */
interface ActionAnim {
  stand?: readonly string[] | ((def: string) => readonly string[]);
  sit?: string | ((def: string) => string);
  upper?: string;
  hands?: number;
}
const ACTIONS: Record<string, ActionAnim> = {
  eat: { stand: ['eat', 'idle'], upper: 'eat', hands: HANDS.hold },
  // Sips from a cup, spaced out.
  drink: { stand: ['drink', 'idle', 'idle'], upper: 'drink', hands: HANDS.hold },
  cook: {
    stand: ['spell', 'interact', 'spell', 'pickUp'],
    sit: 'sitHands',
    hands: HANDS.hold,
  },
  wash: { stand: ['spell', 'interact'], hands: HANDS.relaxed },
  shower: { stand: ['spell', 'interact', 'idle'], hands: HANDS.open },
  bath: { hands: HANDS.open },
  toilet: { hands: HANDS.relaxed },
  sleep: { hands: HANDS.open },
  nap: { hands: HANDS.open },
  sit: { hands: HANDS.open },
  relax: { stand: ['idle', 'foldArms'], hands: HANDS.open },
  type: { stand: ['spell'], sit: 'sitHands', hands: HANDS.keys },
  write: { stand: ['interact', 'spell'], sit: (def) => (NEAR_DESKS.has(def) ? 'sitHandsNear' : 'sitHands'), hands: HANDS.hold },
  read: { stand: ['lantern'], upper: 'lantern', hands: HANDS.hold },
  watch: { stand: ['foldArms', 'idle'], hands: HANDS.open },
  play: { stand: ['spell'], sit: 'sitHands', hands: HANDS.keys },
  // Keys and drums on the desk-hands clip; a cello is held upright between the knees.
  music: { stand: ['spell'], sit: (def) => (def.includes('cello') ? 'sit' : 'sitHands'), hands: HANDS.keys },
  paint: { stand: ['spell', 'interact', 'spell'], hands: HANDS.hold },
  exercise: {
    stand: (def) =>
      def.includes('Bag') || def.includes('smash')
        ? ['punchJab', 'punchCross']
        : def.includes('fitnessGame')
          ? ['dance', 'punchJab', 'punchCross']
          : def.includes('streetball')
            ? ['pickUp', 'interact']
            : // Climbing and stretching (and, `choose`, bouncing and vibrating): the idle with `activityPose` on top.
              def.includes('boulder') || def.includes('sunrise')
              ? ['idle']
              : ['jog'],
    sit: 'sitHands',
    hands: HANDS.grip,
  },
  run: { stand: ['jog'], hands: HANDS.grip },
  lift: { stand: ['pickUp'], hands: HANDS.grip },
  dance: { stand: ['dance'], hands: HANDS.relaxed },
  talk: { stand: ['talk'], hands: HANDS.relaxed },
  // The phone clip's hand at the mouth reads as a microphone.
  sing: { stand: ['talk'], upper: 'phone', hands: HANDS.hold },
  clean: {
    stand: (def) => (def.includes('Mop') || def.includes('Vacuum') ? ['push'] : def.includes('laundry') ? ['spell', 'interact', 'pickUp'] : ['interact', 'spell']),
    hands: HANDS.hold,
  },
  // Gardening, by task (older content and packs: `garden`).
  water: { stand: ['water'], hands: HANDS.hold },
  tend: { stand: ['kneel'], hands: HANDS.relaxed },
  // The baby picked up out of the crib and held (`CUDDLE`).
  cuddle: { stand: ['idle'], hands: HANDS.relaxed },
  // (From the waist-high herb planter, standing.)
  harvest: { stand: (def) => (def === 'garden.herbs' ? ['pick'] : ['harvest', 'plant']), hands: HANDS.hold },
  prune: { stand: ['interact', 'spell', 'interact'], hands: HANDS.grip },
  garden: {
    stand: (def) => (def === 'plant' || def.includes('Terrarium') || def.includes('bonsai') ? ['water'] : ['harvest', 'plant']),
    hands: HANDS.hold,
  },
  tinker: {
    stand: ['interact', 'spell', 'pickUp'],
    sit: 'sitHands',
    hands: HANDS.grip,
  },
  look: {
    stand: (def) => (def === 'telescope' ? ['rail'] : ['idle', 'foldArms']),
    hands: HANDS.relaxed,
  },
  // Down to low flowers / up into a tree's crown (`activityPose` turns the head).
  smell: { stand: ['crouch'], hands: HANDS.relaxed },
  admire: { stand: ['idle', 'foldArms'], hands: HANDS.relaxed },
  phone: { stand: ['phone'], upper: 'phone', hands: HANDS.hold },
  listen: { stand: ['idle', 'foldArms'], hands: HANDS.open },
  meditate: { stand: ['idle'], hands: HANDS.open },
  idle: { stand: ['idle'], hands: HANDS.relaxed },
  // A visitor at the front door: a knock, then waiting.
  knock: { stand: ['interact', 'idle'], hands: HANDS.relaxed },
  // Cooking, step by step (content `recipes`): ingredients from the fridge, chopping on the board,
  // stirring the pan, turning things on the grill, putting a dish down.
  fetch: { stand: ['interact', 'pickUp', 'idle'], hands: HANDS.hold },
  chop: { stand: ['chop', 'interact', 'chop'], hands: HANDS.grip },
  stir: { stand: ['stir', 'interact', 'stir'], hands: HANDS.hold },
  flip: { stand: ['flip', 'interact', 'flip'], hands: HANDS.grip },
  serve: { stand: ['pickUp', 'idle'], hands: HANDS.hold },
};

/** The prop drawn on a station's top while a Sim works there (by action tag; `tool.*` models). */
const STATION_PROPS: Record<string, string> = { chop: 'tool.board', stir: 'tool.pan', flip: 'tool.grillFood' };
/** How far (m, body scale) below the palms' centres a carried thing's base rests. */
const HOLD_DROP = 0.02;
/** The palm centre's offset from the wrist (left hand, body scale) for sets without `grip`. */
const DEFAULT_GRIP: readonly [number, number, number] = [-0.023, -0.062, 0];
/** Looping clips in a use sequence play this long (s) per step. */
const LOOP_STEP = 4;

export type { Face } from './characters/face';
/** Expression per emotion id (content `emotions`, packs included); unknown ids stay neutral. */
export const EXPRESSIONS: Record<string, Partial<Face>> = {
  happy: { mood: 'happy' },
  flirty: { mood: 'flirty' },
  confident: { mood: 'confident' },
  focused: { mood: 'focused', gazeDown: 0.15 },
  inspired: { mood: 'inspired' },
  energized: { mood: 'energized' },
  playful: { mood: 'playful' },
  relaxed: { mood: 'relaxed' },
  embarrassed: { mood: 'embarrassed', gazeDown: 0.6 },
  sad: { mood: 'sad', gazeDown: 0.45 },
  tense: { mood: 'tense' },
  uncomfortable: { mood: 'uncomfortable' },
  bored: { mood: 'bored', gazeDown: 0.2 },
  angry: { mood: 'angry' },
  dazed: { mood: 'dazed' },
  surprised: { mood: 'surprised' },
  tired: { mood: 'tired', gazeDown: 0.3 },
  disgusted: { mood: 'disgusted' },
};
/**
 * The resting face every expression starts from: a soft closed-lip smile, relaxed brows lifted a
 * touch with soft inner ends, the lower lids a little raised as in a real smile. Scaled by mood
 * (gone when a Sim is very unhappy) and left out under emotions that frown or knit the brows, which
 * define the face themselves; the face returns to it when they pass.
 */
export const FRIENDLY: Partial<Face> = { smile: 0.35, lidL: 0.15, browUp: 0.15, browIn: 0.1 };

/** Emotions that set the whole face themselves (no resting smile under them). */
const COLD = new Set(['focused', 'embarrassed', 'sad', 'tense', 'uncomfortable', 'bored', 'angry', 'dazed', 'tired', 'disgusted']);
/** Mood (0..1) below which the resting smile fades out, and where it is full. */
const MOOD_COLD = 0.2;
const MOOD_WARM = 0.55;

/**
 * Relaxed standing: the clips stand that way themselves (feet under the hips, the weight drifting
 * from leg to leg, arms hanging loose). Over a skirt the hanging arms move out a little (radians,
 * model space; faded in on the clips below, see `stance`).
 */
export const STANCE = { skirt: -0.22 };
/** Clips whose arms hang at the sides. */
const RELAX_ARMS = new Set(['idle', 'yes', 'no']);
const ASLEEP: Partial<Face> = { mood: 'asleep', smile: 0 };
const LAUGH: Partial<Face> = { mood: 'laugh', smile: 0, frown: 0, browDown: 0, press: 0 };
const KISS: Partial<Face> = { mood: 'kiss', smile: 0, frown: 0, browDown: 0, press: 0 };
const HUG: Partial<Face> = { mood: 'hug', smile: 0, frown: 0, browDown: 0, press: 0 };
const NONE: Partial<Face> = {};

/**
 * Stage directions for one Sim row (character creator and portraits; never set in game): a clip to
 * play instead of the activity's, an expression on top of the emotion, a point to look at, an
 * extra procedural layer, and `still` (portraits: the clip held at `time`, no blinks or glances,
 * no smoothing). A directed Sim turns to the snapshot's yaw at once (the director smooths it).
 */
export interface StageDirection {
  clip?: string | null;
  /** With `still`: the clip time (s) to hold. */
  time?: number;
  /** Finger pose (0 = the clip's grip, 1 = open). */
  hands?: number;
  face?: Partial<Face> | null;
  /** World point the eyes (and partly the head) turn to. */
  lookAt?: { x: number; y: number; z: number } | null;
  /** Extra model-space rotations after the built-in layers (gestures). */
  pose?: ((add: Additive, bone: Record<string, number>, time: number) => void) | null;
  still?: boolean;
}

interface Player {
  clip: Clip | null;
  key: string;
  time: number;
  rate: number;
  /** Play non-looping clips in a loop (object use). */
  repeat: boolean;
  /** Root placement relative to the Sim: height, forward offset, pitch (lying). */
  rootY: number;
  rootFwd: number;
  rootPitch: number;
}

function player(): Player {
  return {
    clip: null,
    key: '',
    time: 0,
    rate: 1,
    repeat: false,
    rootY: 0,
    rootFwd: 0,
    rootPitch: 0,
  };
}

function copyPlayer(from: Player, to: Player): void {
  to.clip = from.clip;
  to.key = from.key;
  to.time = from.time;
  to.rate = from.rate;
  to.repeat = from.repeat;
  to.rootY = from.rootY;
  to.rootFwd = from.rootFwd;
  to.rootPitch = from.rootPitch;
}

interface SimRig {
  id: number;
  /** Who the row holds, in what body (see `build`: a rebuild keeps the rig only for the same). */
  who: string;
  /** Height of the floor under them (upper storeys, part way up the stairs). */
  floorY: number;
  body: BodyData;
  scale: number;
  /** Head size relative to the body (children's heads are bigger). */
  head: number;
  /** Its body's height as a share of its family's base body's (a child's body shaped for its stage: under 1). */
  size: number;
  /** How much they stoop (elders), 0..1. */
  stoop: number;
  /** Build (girth across the torso and limbs; 1 = as modelled). */
  build: number;
  seed: number;
  visible: boolean;
  /** Displayed position / yaw (smoothed). */
  x: number;
  z: number;
  yaw: number;
  speed: number;
  lastPose: number;
  /** Seconds in the current activity (for idle variations / sequences). */
  actTime: number;
  seq: number;
  useKey: string;
  useSeq: readonly string[] | null;
  /** Current action tag ('' = none / unknown) and whether it came from the snapshot. */
  tag: string;
  /** Def of the object in use ('' = none), for activity layers that depend on it, and the object. */
  useDef: string;
  useObj: PlacedObject | null;
  /** The snapshot's id of the object in use (-1: none). */
  useId: number;
  /** Lying in a hammock's sling (`inSling`), or null; the upper body's rise in it (rad). */
  sling: Bed['sling'] | null;
  slingLift: number;
  tagKnown: boolean;
  cur: Player;
  prev: Player;
  /** Weight of `cur` (1 = fully faded in). */
  w: number;
  fadeRate: number;
  /** Upper-body layer (eating while seated, hugging). */
  upper: Player;
  upperW: number;
  upperTarget: number;
  lookYaw: number;
  lookPitch: number;
  /** Extra forward step towards a partner (hugs, kisses). */
  approach: number;
  /**
   * A cuddle (`CUDDLE`): seconds since it began (-1: not cuddling) and since laying the baby back
   * down began (-1: not yet), game minutes so far, the pace of picking up / putting back, and the
   * baby's row (-1: none).
   */
  cuddleT: number;
  cuddleOut: number;
  cuddleMin: number;
  cuddleRate: number;
  cuddleBaby: number;
  /** Its poses' weights (0..1): reaching into the crib, holding the baby; and how far the baby is out of the crib, in the arms. */
  reachW: number;
  holdW: number;
  liftW: number;
  /** Where the feet step in towards the crib (m, along the facing). */
  cuddleStep: number;
  /** Day clothes and pyjamas (switched when the Sim lies down in bed). */
  day: OutfitParams | null;
  night: OutfitParams | null;
  asleep: boolean;
  /** Finger openness (see HANDS), smoothed. */
  hands: number;
  /** Seconds in the current step of a use sequence. */
  stepTime: number;
  /** Emotion code (index + 1 into the content's emotions; 0 = none). */
  emotion: number;
  /** Bed while lying (for the blanket) and how far the blanket is pulled up (0..1). */
  bed: PlacedObject | null;
  bedH: number;
  /** How far forward of the slot it lies (`BEDS` `fwd`). */
  bedFwd: number;
  /**
   * Lying along a bed lain on across (`BEDS` `across`), in its middle, from whichever slot (on a
   * 2-wide seat the nap takes both: content `whole`).
   */
  lieAcross: boolean;
  /**
   * Getting out of bed: sim-core puts the Sim on the tile in front at once; the lying body
   * (fading out) and the blanket stay where it lay (`lieX`, `lieZ`: its slot, kept while lying)
   * and the Sim stands up from the foot of the bed.
   */
  fromBed: boolean;
  lieX: number;
  lieZ: number;
  /**
   * Using a thing from inside or up against it (`STEP_IN`): which and how (null: not), the seconds since
   * the use began, and where they were last drawn (world); stepping back out to the tile in front after:
   * the seconds since the use ended (-1: not stepping out).
   */
  inside: StepIn | null;
  inT: number;
  inX: number;
  inY: number;
  inZ: number;
  /** Its lane (`lanes`), and the step in's sideways part to it (world). */
  inLane: number;
  inDX: number;
  inDZ: number;
  outT: number;
  /** Seconds after the use before stepping out (the entry's `out`). */
  outWait: number;
  /** Pulled up to a table (`TUCK`): how far the chair is in now (m), and which seat (-1: none). */
  tuck: number;
  tuckSeat: number;
  /** The top still further off than the desk-hands clip reaches, pulled in as far as it goes (`sitHandsFar`). */
  tuckFar: boolean;
  cover: number;
  /** Current (smoothed) facial expression. */
  face: Face;
  /** Blink: start time of the current blink and when the next one starts (s). */
  blinkAt: number;
  nextBlink: number;
  /** Eye gaze (rad, head-relative, smoothed) and the current gaze target (0 none, 1 partner, 2 camera, 3 glance). */
  eyeYaw: number;
  eyePitch: number;
  gazeMode: number;
  gazeUntil: number;
  glanceYaw: number;
  glancePitch: number;
  /** In a group conversation, someone listening in they glance at while speaking (head and eyes), instead of the one they talk with. */
  audience: SimRig | null;
  /** Stage directions for this frame (null in game). */
  dir: StageDirection | null;
  /** Resting-face warmth from mood (0..1). */
  warmth: number;
  /** Relaxed stance weight of the arms (smoothed 0..1). */
  relaxArms: number;
  /** Wears a skirt (the hands hang clear of it). */
  skirt: boolean;
  /** The face's morph weights (one per channel), settling towards the expression. */
  faceState: Float32Array;
  /** Face asymmetry: left / right side scales (~0.9..1.1, from the seed). */
  faceL: number;
  faceR: number;
  /** Voice line being spoken (lip sync): its visemes, start (s, `performance.now` clock) and segment. */
  speech: VisemeTrack | null;
  speechStart: number;
  speechSeg: number;
}

/**
 * A leg in a clip (`measureLegs`; body space, scale 1, in the y-z plane): the hip's height and depth,
 * the thigh's and the calf's lengths and their directions (rad from straight down towards the front).
 */
interface LegSide {
  hy: number;
  hz: number;
  l1: number;
  l2: number;
  t0: number;
  c0: number;
}
interface LegFit {
  l: LegSide;
  r: LegSide;
  /** The spine's base (spine_01) and the head's joint: height and depth. */
  spine: readonly [number, number];
  head: readonly [number, number];
}
/** Clips whose legs are placed by `legTo`: sitting (pedalling, legs out along a chaise) and lying (in a hammock). */
const LEG_CLIPS = ['sit', 'sitHands', 'lie'] as const;

/** Pelvis offset of the seated clip for one body (scale 1). */
interface SeatFit {
  y: number;
  z: number;
}

interface PlacedObject {
  def: string;
  /** Facing (0..3, as the simulation's `rot`). */
  rot: number;
  /** Where Sims sit on it (see `seatOf`; resized taller or shorter, the seat moves with it). */
  seat: Seat;
  /** What Sims stand up on to use it (content `stand`; resized, it moves too), or null. */
  stand: { h: number; motion?: 'bounce' | 'buzz' } | null;
  /**
   * What Sims hold while they use it (content `held`): its parts' prop keys (`<model key>#<part>`, then
   * `@<colours>` for an object the player coloured: drawn in its colours, as the object is) and how, or null.
   */
  held: { keys: string[]; grip: Grip } | null;
  /** Metres taller than its kind (the player's resizing): mattresses rise with it too. */
  lift: number;
  /** Something to sit or lie on (a seat or a bed, by its interactions or the tables above). */
  restful: boolean;
  cx: number;
  cz: number;
  minX: number;
  minZ: number;
  maxX: number;
  maxZ: number;
}

interface PartMesh {
  mesh: Mesh;
  /** Instance settings (the Sims' rows) currently drawn. */
  settings: Float32Array;
  /** Every Sim wearing it; small face parts draw only those close enough (`detail`). */
  all: Float32Array;
  /** Projected size (px) of a 5 cm face feature below which this part isn't drawn (0: always). */
  detail: number;
  /** Drawn on Sims near the camera, far from it (the low-detail body and clothes), or always. */
  range: Range;
}

const enum Range {
  Always,
  Near,
  Far,
}

/**
 * Sims switch to the low-detail body and clothes below this projected size (px per 5 cm of face)
 * and back above the second: the gap keeps a Sim at the threshold from flickering between them.
 */
const LOD_FAR_PX = 6;
const LOD_NEAR_PX = 7;

/**
 * Small face parts are skipped on faces too small on screen to show them: the projected size (px)
 * of a 5 cm feature (a mouth, an eye with its brow) below which each isn't drawn.
 */
const DETAIL_PX: Record<string, number> = { teeth: 8, tongue: 8, lashes: 3, brows: 2 };
/** Below this (same measure), a face doesn't run its morphs at all (expressions can't be seen). */
const MORPH_PX = 3;

export class Characters {
  private set: CharacterSet | null = null;
  private failed = false;
  private texture!: RawTexture;
  private data!: Float32Array;
  /** The pose texture as last uploaded, and which rows were drawn then (see `upload`). */
  private uploaded!: Float32Array;
  private readonly uploadedVisible = new Uint8Array(MAX_CHARACTERS);
  /** The last `update` changed how a drawn Sim looks (moved, breathed, blinked …): their shadows are drawn again. */
  posed = false;
  private width = 0;
  private vat!: BakedVertexAnimationManager;
  /** Face morphs (null for sets without them) and the per-Sim channel mixer. */
  private morph: MorphBinding | null = null;
  private mixer: FaceMixer | null = null;
  private readonly meshes = new Map<string, PartMesh>();
  /** The drawn part meshes by mesh (`rowsOf`). */
  private readonly parts = new WeakMap<object, PartMesh>();
  private readonly materials = new Map<string, PBRMaterial>();
  private readonly skeletons = new Map<string, Skeleton>();
  private sims: (SimRig | null)[] = [];
  private objects: PlacedObject[] = [];
  /** Rows per storey (0: one storey): residents' rows are mapped back onto the ground's. */
  private storeyDepth = 0;

  /** A lot row on the ground storey's rows (where it's drawn, lifted to its storey's height). */
  private groundRow(z: number): number {
    const d = this.storeyDepth;
    return d > 0 ? z - Math.floor(z / d) * d : z;
  }
  private readonly objectById = new Map<number, PlacedObject>();
  /** Pulled up to a table (`TUCK`): the tables the seats face, by seat id (`setTables`), and the seats' ids. */
  private tables = new Map<number, TableRoom>();
  private readonly seatIds = new Map<PlacedObject, number>();
  /** Where an object's moving seat has its riders now (`ObjectMotion.ride`): world space, from rest. */
  private rideOf: ((objectId: number) => Matrix | null) | null = null;
  private readonly rideM = new Matrix();
  private readonly rideQ = new Quaternion();
  private readonly rideS = new Vector3();
  private readonly rideT = new Vector3();
  /** Action tag per `sim.action` index (from the snapshot layout). */
  private actionTags: readonly string[] = [];
  private scratch!: PoseScratch;
  private qA!: Float32Array;
  private pA!: Float32Array;
  private qB!: Float32Array;
  private pB!: Float32Array;
  private add!: Additive;
  private upperMask!: Float32Array;
  private fingerMask!: Float32Array;
  private readonly place: Placement = {
    x: 0,
    y: 0,
    z: 0,
    qx: 0,
    qy: 0,
    qz: 0,
    qw: 1,
    scale: 1,
  };
  private readonly headTmp = new Float32Array(3);
  private readonly palm = new Float32Array(3);
  /** Per Sim row: the grown-up holding them (a baby picked up for a cuddle, `CUDDLE`), -1 none. */
  private readonly heldBy = new Int16Array(MAX_CHARACTERS).fill(-1);
  /** The rows in the order they're posed this frame: a baby in someone's arms after the one holding them. */
  private readonly order = new Int16Array(MAX_CHARACTERS);
  /** Per Sim row holding a baby: the middle of the cradling forearms (x, y, z) as last posed. */
  private readonly cradle = new Float32Array(MAX_CHARACTERS * 3);
  /** Scratch for placing a held baby: two orientations and a vector. */
  private readonly cradleQ = new Float32Array(12);
  /** How long a cuddle lasts (game minutes), per object def. */
  private readonly cuddleMinutes = new Map<string, number>();
  private readonly faceTmp: Face = { ...NEUTRAL };
  /** Gaze yaw / pitch (head-relative) from `gazeAt`. */
  private readonly gaze = new Float32Array(2);
  /**
   * Things Sims carry (content `items`, by model) and tools on the station they cook at (`tool.*`):
   * one template per model, one thin instance (matrix) per Sim row; null for a part its model doesn't have.
   */
  private readonly props: TemplateCache<Prop | null>;
  /** Stops listening for model files that come (`onModelFileArrived`). */
  private readonly unlisten: () => void;
  /** Per Sim row: the model of what it carries and of the tool in front of it, as last drawn. */
  private readonly heldProp: (string | null)[] = new Array(MAX_CHARACTERS).fill(null);
  private readonly toolProp: (string | null)[] = new Array(MAX_CHARACTERS).fill(null);
  /** Per Sim row: what it holds of the object it uses (content `held`), in the left and the right hand (or both). */
  private readonly leftProp: (string | null)[] = new Array(MAX_CHARACTERS).fill(null);
  private readonly rightProp: (string | null)[] = new Array(MAX_CHARACTERS).fill(null);
  /** Scratch for `writeHeld`: a frame's axes and origin (x, y, z each). */
  private readonly frame3 = new Float32Array(12);
  /**
   * Blankets: one quilt mesh per body (the base bodies', and each life stage's body as it loads, `addQuilt`),
   * one thin instance (matrix) per Sim row; `ext` holds two numbers a row (`writeBlanket`).
   */
  private readonly blankets = new Map<string, BlanketShape & { matrices: Float32Array; ext: Float32Array; base: boolean }>();
  /** The quilts' shared material, for the life stages' quilts made later. */
  private quilt: PBRMaterial | null = null;
  /** Per Sim row: head top (x, y, z) for speech bubbles and the selection marker. */
  readonly heads = new Float32Array(MAX_CHARACTERS * 3);
  /** Per Sim row: where they stand (x, z), drawn or not (NaN while away). */
  private readonly spots = new Float32Array(MAX_CHARACTERS * 2).fill(NaN);
  /** Per Sim row, for a conversation they started: how many listen in, and (up to) two of them. */
  private readonly groups = new Int16Array(MAX_CHARACTERS * 3);
  /** Per Sim row, for a conversation they started with others listening in: the middle of the group (x, z). */
  private readonly groupMiddle = new Float32Array(MAX_CHARACTERS * 2);
  /** Per Sim row: projected size (px) of 5 cm of its face, from the last frame (see `faceDetail`). */
  private readonly facePx = new Float32Array(MAX_CHARACTERS).fill(1e6);
  /** Per Sim row: drawn with the low-detail body and clothes. */
  private readonly far = new Uint8Array(MAX_CHARACTERS);
  /** The set has low-detail bodies. */
  private hasLod = false;
  /** Shadow-only copies of the low-detail meshes (all Sims cast shadows from those). */
  private readonly shadowMeshes = new Map<string, PartMesh>();
  readonly visible = new Uint8Array(MAX_CHARACTERS);
  private lastNow = 0;
  /** Seconds of animation played: stands still while the game is paused (Sims hold their pose). */
  private clock = 0;
  /** Sit / lie pelvis offsets per body (computed once from the clips). */
  private readonly sitPelvis = new Map<string, SeatFit>();
  /** Per body, then clip (`LEG_CLIPS`): where its legs are in the clip, for placing the feet (`legTo`). */
  private readonly legFits = new Map<string, Map<string, LegFit>>();
  /** Where a moving object's loop has got to (0..1 of a round; a bike's pedals), null while none plays. */
  private loopOf: ((objectId: number) => number | null) | null = null;
  /** Scratch: a leg's thigh and calf rotations from `legTo`, and the root's height and forward offset from `inSling`. */
  private readonly legAdd = new Float32Array(2);
  private readonly slingAt = new Float32Array(2);
  /** `?debug`: force a clip on every Sim (`__characters.force = 'sit'`). */
  force: string | null = null;
  /** `?debug`: keep the standing stance layers on a forced clip (as the clip looks in game). */
  forceStance = false;
  /**
   * `?debug`: expression override (`__characters.faceDebug = { smile: 1, jaw: 0.5, closed: 1 }`;
   * `channels: { jawOpen: 1 }` adds raw morph channels, `viseme: 10` holds a viseme).
   */
  faceDebug: (Partial<Face> & { closed?: number; talk?: boolean; channels?: Record<string, number>; viseme?: number }) | null = null;
  /** `?debug`: animation speed override (e.g. to keep animating while the game is paused). */
  debugRate: number | null = null;
  /** CPU time of the last `update` (ms, smoothed), for profiling. */
  updateMs = 0;
  /** Emotion id per snapshot emotion code - 1 (content `emotions`). */
  private emotionIds: readonly string[] = [];
  /** Stage directions per Sim row (character creator / portraits only; empty in game). */
  readonly directions: (StageDirection | null)[] = [];
  /** Selected Sim (selection ring), or null. */
  private selected: number | null = null;
  /** Reduce motion: the selection ring keeps still (no spin, no pulse). */
  reducedMotion = false;
  private ring: Mesh | null = null;
  private readonly ringMatrix = new Float32Array(16);

  constructor(
    private readonly scene: Scene,
    private readonly assets: AssetRegistry,
    private readonly lib: MaterialLibrary,
    /** For seats content describes (`seat`) and what can be sat or lain on. */
    private readonly content?: Content,
    /**
     * Builds again the props drawn without a model file that came (the game's): the scene then tries
     * failed files in the background (`onModelFileArrived`). Off for the preview studio and the debug pages.
     */
    refreshProps = false,
  ) {
    this.props = new TemplateCache(scene);
    // A model file that failed has come: the props drawn without it (small blocks) are built again,
    // and the blocks go once they are (hidden at once: the frame no longer moves them).
    this.unlisten = refreshProps
      ? onModelFileArrived(scene, (url) => {
          const keys = this.props.invalidate(url);
          if (!keys.length) return;
          this.props.hideStale();
          const dispose = () => this.props.disposeStale();
          void Promise.all(keys.map((key) => this.props.get(key, () => this.buildProp(key)))).then(dispose, dispose);
        })
      : () => {};
  }

  /** Loads the character set and creates the pose texture; false if unavailable. */
  async init(): Promise<boolean> {
    if (this.set) return true;
    if (this.failed) return false;
    const entry = this.assets.get('model.sim', 'character');
    if (!entry) {
      this.failed = true;
      return false;
    }
    try {
      this.set = await loadCharacterSet(entry.url);
    } catch (err) {
      console.warn('[render] characters unavailable', err);
      this.failed = true;
      return false;
    }
    const set = this.set;
    const NB = set.bones.length;
    this.width = NB * 4 + APPEARANCE_TEXELS + MORPH_TEXELS;
    this.data = new Float32Array(this.width * MAX_CHARACTERS * 4);
    this.uploaded = new Float32Array(this.data.length);
    this.texture = new RawTexture(
      this.data,
      this.width,
      MAX_CHARACTERS,
      Constants.TEXTUREFORMAT_RGBA,
      this.scene,
      false,
      false,
      Constants.TEXTURE_NEAREST_SAMPLINGMODE,
      Constants.TEXTURETYPE_FLOAT,
    );
    this.texture.name = 'simPoses';
    this.vat = new BakedVertexAnimationManager(this.scene);
    this.vat.texture = this.texture;
    this.vat.isEnabled = true;
    const morphs = set.morphs;
    if (morphs) {
      const texture = new RawTexture(
        morphs.data,
        morphs.width,
        morphs.height,
        Constants.TEXTUREFORMAT_RGBA,
        this.scene,
        false,
        false,
        Constants.TEXTURE_NEAREST_SAMPLINGMODE,
        Constants.TEXTURETYPE_HALF_FLOAT,
      );
      texture.name = 'simMorphs';
      this.morph = { texture, width: morphs.width, header: morphs.headerTexels };
      this.mixer = new FaceMixer(morphs);
    }
    this.scratch = new PoseScratch(NB);
    this.qA = new Float32Array(NB * 4);
    this.qB = new Float32Array(NB * 4);
    this.pA = new Float32Array(3);
    this.pB = new Float32Array(3);
    this.add = makeAdditive(NB);
    // Upper body: spine_02 and everything above it (arms, neck, head); spine_01 half.
    this.upperMask = new Float32Array(NB);
    for (let b = 0; b < NB; b++) {
      let p = b;
      let w = 0;
      while (p >= 0) {
        const n = set.bones[p];
        if (n === 'spine_02') {
          w = 1;
          break;
        }
        if (n === 'spine_01' && p === b) w = 0.5;
        p = set.parents[p];
      }
      this.upperMask[b] = w;
    }
    // How far each finger joint opens towards the rest pose (scaled by the hand pose): the knuckles
    // keep more of the clip's curl than the tips, and the curl grows from the index to the little
    // finger, so open hands rest softly curved instead of straight-knuckled with curled tips (a claw).
    this.fingerMask = new Float32Array(NB);
    set.bones.forEach((n, b) => {
      const m = /^(index|middle|ring|pinky|thumb|fingers)_0(\d)/.exec(n);
      if (m) this.fingerMask[b] = FINGER_OPEN[m[1]] * (JOINT_OPEN[Number(m[2]) - 1] ?? 1);
    });
    for (const body of set.bodies.values()) {
      this.sitPelvis.set(body.name, this.measureSit(body));
      this.legFits.set(body.name, this.measureLegs(body));
    }
    const lying = [...set.bodies.values()].map((body) => ({
      body,
      ...this.lyingPose(body),
    }));
    // One stretch range for both quilts (their toes are within a centimetre).
    const toeZ = Math.max(...lying.map((l) => l.feetZ)) + 0.03;
    this.quilt = blanketMaterial(this.scene, toeZ, toeZ + 0.29);
    this.lib.adopt(this.quilt);
    for (const l of lying) this.addQuilt(l.body, l);
    this.ring = this.makeRing();
    if (new URLSearchParams(location.search).has('debug')) Object.assign(window, { __characters: this, __stance: STANCE, __friendly: FRIENDLY });
    return true;
  }

  /** The mesh part of a hairstyle (manifest `model.hair.<style>` may name another part). */
  private hairPart(style = 'short'): string {
    const key = `model.hair.${style}`;
    return (this.assets.has(key, 'character') ? this.assets.get(key, 'character')!.part : undefined) ?? `hair.${style}`;
  }

  /** The base body (young adult) for a gender. */
  private baseBody(gender: string): BodyData {
    const set = this.set!;
    return set.bodies.get(gender === 'female' ? 'female' : 'male') ?? set.bodies.values().next().value!;
  }

  /** Emotion ids in snapshot order (`emotion` field = index + 1), for facial expressions. */
  setEmotions(ids: readonly string[]): void {
    this.emotionIds = ids;
  }

  /** Shows the selection ring under Sim `id` (null hides it). */
  select(id: number | null): void {
    this.selected = id;
  }

  /** A soft glowing ring on the floor around the selected Sim (one thin instance, moved per frame). */
  private makeRing(): Mesh {
    const size = 256;
    const tex = new DynamicTexture('simRing', size, this.scene, true);
    const ctx = tex.getContext() as CanvasRenderingContext2D;
    const g = ctx.createRadialGradient(size / 2, size / 2, size * 0.3, size / 2, size / 2, size / 2);
    g.addColorStop(0, 'rgba(255,255,255,0)');
    g.addColorStop(0.55, 'rgba(255,255,255,0.0)');
    g.addColorStop(0.72, 'rgba(255,255,255,0.85)');
    g.addColorStop(0.8, 'rgba(255,255,255,0.55)');
    g.addColorStop(1, 'rgba(255,255,255,0)');
    ctx.clearRect(0, 0, size, size);
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, size, size);
    tex.update();
    tex.hasAlpha = true;
    const mat = new StandardMaterial('simRing', this.scene);
    mat.disableLighting = true;
    mat.diffuseColor = Color3.Black();
    mat.specularColor = Color3.Black();
    mat.emissiveColor = Color3.FromHexString('#7BE38A');
    mat.opacityTexture = tex;
    mat.disableDepthWrite = true;
    mat.zOffset = -4;
    const ring = MeshBuilder.CreateGround('simRing', { width: 1, height: 1 }, this.scene);
    ring.material = mat;
    ring.isPickable = false;
    ring.alwaysSelectAsActiveMesh = true;
    ring.thinInstanceSetBuffer('matrix', this.ringMatrix, 16, false);
    return ring;
  }

  private writeRing(time: number): void {
    const m = this.ringMatrix;
    m.fill(0);
    const id = this.selected;
    const rig = id !== null ? this.sims[id] : null;
    if (rig && rig.visible && !LYING.has(rig.cur.key)) {
      const r = 0.95 * rig.scale * (1 + Math.sin(time * 2.4) * 0.03);
      const a = time * 0.5;
      m[0] = Math.cos(a) * r;
      m[2] = -Math.sin(a) * r;
      m[5] = 1;
      m[8] = Math.sin(a) * r;
      m[10] = Math.cos(a) * r;
      m[12] = rig.x;
      m[13] = 0.03;
      m[14] = rig.z;
      m[15] = 1;
    }
    this.ring?.thinInstanceBufferUpdated('matrix');
  }

  /** Character meshes in use (shadow casters; small details excluded). */
  casters(): Mesh[] {
    const out: Mesh[] = [];
    // The low-detail body and clothes' shadow copies; sets without them cast from the full meshes.
    for (const p of this.shadowMeshes.values()) if (p.mesh.isEnabled()) out.push(p.mesh);
    if (out.length) return out;
    for (const [key, p] of this.meshes) if (p.mesh.isEnabled() && !/:(eyes|brows|lashes|teeth|tongue|shoes\.)/.test(key)) out.push(p.mesh);
    return out;
  }

  /**
   * Builds instances for the world's Sims (meshes and materials are cached). `keep` keeps the
   * animation state (clips, blankets, pyjamas, a baby in someone's arms ...): `'rows'` of every
   * row that stays (previews restyling a Sim without restarting it), `'residents'` of rows that
   * still hold the same resident in the same body (the game: the world is rebuilt whenever
   * something in it changes; a slot taken by someone new, or a body grown into the next life
   * stage, starts afresh). A Sim may carry `outfitId`, the id its outfit is derived from when it
   * differs from its row.
   */
  async build(world: Pick<WorldStructure, 'sims' | 'objects'> & { storeys?: number; storeyDepth?: number; basement?: number }, keep: 'rows' | 'residents' | false = false): Promise<void> {
    if (!(await this.init())) return;
    const set = this.set!;
    // Life stages other than the base bodies' (children, teens, elders …) load on demand, the
    // first time a Sim needs them.
    await Promise.all(
      world.sims.map(async (s) => {
        if (s.gone || s.id >= MAX_CHARACTERS) return null;
        const base = this.baseBody(s.gender);
        const stage = s.appearance?.stage;
        if (stage && stage !== base.stage) {
          const b = await loadStage(set, base, stage).catch((err) => (console.warn('[render] life stage unavailable', base.name, stage, err), null));
          if (b && !this.sitPelvis.has(b.name)) {
            this.sitPelvis.set(b.name, this.measureSit(b));
            this.legFits.set(b.name, this.measureLegs(b));
          }
          // Its own quilt, draped over its own lying pose: a child's feet stand up higher for its size than a
          // grown-up's quilt made smaller covers (their shoes showed through it halfway down the bed).
          if (b && !this.blankets.has(b.name) && this.quilt) this.addQuilt(b, this.lyingPose(b));
        }
      }),
    );
    // Positions arrive in lot rows, which hold every storey (see `WorldStructure.storeys`).
    this.storeyDepth = (world.storeys ?? 1) > 1 || world.basement !== undefined ? (world.storeyDepth ?? 0) : 0;
    const kept = keep ? this.sims : [];
    this.sims = [];
    this.ghosts = [];
    this.data.fill(0);
    this.visible.fill(0);
    const groups = new Map<string, SimInfo[]>();
    const push = (key: string, s: SimInfo) => {
      const list = groups.get(key) ?? [];
      list.push(s);
      groups.set(key, list);
    };
    for (const s of world.sims) {
      if (s.id >= MAX_CHARACTERS || s.gone) continue;
      const base = this.baseBody(s.gender);
      const stage = s.appearance?.stage;
      // The body shaped for the Sim's life stage when there is one (else the base body, scaled).
      const body = stage && stage !== base.stage ? (set.bodies.get(`${base.name}.${stage}`) ?? base) : base;
      const shaped = !!stage && body.stage === stage;
      const hairKey = this.hairPart(s.appearance?.hairStyle);
      const outfitId = (s as SimInfo & { outfitId?: number }).outfitId ?? s.id;
      const day = outfitFor(outfitId, s.gender, s.appearance, body);
      // (Pyjamas are the same garments, so the same chunks show at night.)
      for (const part of skinChunks(body, day.parts)) if (body.parts.has(part)) push(`${body.name}:${part}`, s);
      for (const part of FACE_PARTS) if (body.parts.has(part)) push(`${body.name}:${part}`, s);
      if (body.parts.has(hairKey)) push(`${body.name}:${hairKey}`, s);
      if (s.appearance?.beard && !YOUNG_STAGES.has(body.stage) && body.parts.has('hair.beard')) push(`${body.name}:hair.beard`, s);
      for (const part of day.parts) if (body.parts.has(part)) push(`${body.name}:${part}`, s);
      // The low-detail body and clothes (drawn instead when the Sim is far from the camera).
      for (const part of ['lod.body', ...day.parts.map((p) => `lod.${p}`)]) if (body.parts.has(part)) push(`${body.name}:${part}`, s);
      const height = clampHeight(s.appearance?.height);
      // Life stage: a body shaped for it keeps its own size (at the base body's scale); otherwise
      // children are the base body made smaller with a bigger head (the session sets these).
      const stageScale = shaped ? 1 : (s.appearance?.stageScale ?? 1);
      const scale = ((BASE_HEIGHT[base.name] ?? 1.72) / base.height) * height * stageScale;
      const build = clampBuild(s.appearance?.build);
      const head = shaped ? 1 : (s.appearance?.stageHead ?? 1);
      const stoop = s.appearance?.stageStoop ?? 0;
      const size = body.height / base.height;
      const who = `${s.name}|${s.generation ?? 0}|${s.gender}|${stage ?? ''}|${body.name}|${outfitId}`;
      const old = kept[s.id];
      // The same rig carries on (pyjamas: `asleep` is worked out again, and the outfit for it written, when next posed).
      const rig: SimRig = old && (keep === 'rows' || old.who === who) ? Object.assign(old, { who, body, scale, size, head, stoop, build, asleep: false }) : {
        id: s.id,
        who,
        floorY: 0,
        body,
        scale,
        head,
        stoop,
        build,
        size,
        seed: hash(s.id + 17),
        visible: false,
        x: 0,
        z: 0,
        yaw: 0,
        speed: 0,
        lastPose: -1,
        actTime: 0,
        seq: 0,
        useKey: '',
        useSeq: null,
        tag: '',
        useDef: '',
        useObj: null,
        useId: -1,
        sling: null,
        slingLift: 0,
        tagKnown: false,
        cur: player(),
        prev: player(),
        w: 1,
        fadeRate: 4,
        upper: player(),
        upperW: 0,
        upperTarget: 0,
        lookYaw: 0,
        lookPitch: 0,
        approach: 0,
        cuddleT: -1,
        cuddleOut: -1,
        cuddleMin: 0,
        cuddleRate: 1,
        cuddleBaby: -1,
        reachW: 0,
        holdW: 0,
        liftW: 0,
        cuddleStep: 0,
        day: null,
        night: null,
        asleep: false,
        hands: HANDS.relaxed,
        stepTime: 0,
        emotion: 0,
        bed: null,
        bedH: 0,
        bedFwd: 0,
        lieAcross: false,
        fromBed: false,
        lieX: 0,
        lieZ: 0,
        inside: null,
        inT: 0,
        inX: 0,
        inY: 0,
        inZ: 0,
        inLane: 0,
        inDX: 0,
        inDZ: 0,
        outT: -1,
        outWait: 0,
        tuck: 0,
        tuckSeat: -1,
        tuckFar: false,
        cover: 0,
        face: { ...NEUTRAL },
        blinkAt: -1,
        nextBlink: 0,
        eyeYaw: 0,
        eyePitch: 0,
        gazeMode: 0,
        gazeUntil: 0,
        glanceYaw: 0,
        glancePitch: 0,
        audience: null,
        dir: null,
        warmth: 1,
        relaxArms: 0,
        skirt: false,
        faceState: new Float32Array(set.morphs?.channels.length ?? 0),
        faceL: 1 + (hash(s.id * 7 + 3) - 0.5) * 0.2,
        faceR: 1 + (hash(s.id * 7 + 5) - 0.5) * 0.2,
        speech: null,
        speechStart: 0,
        speechSeg: 0,
      };
      this.sims[s.id] = rig;
      rig.day = outfitFor(outfitId, s.gender, s.appearance, body);
      rig.night = outfitFor(outfitId, s.gender, s.appearance, body, true);
      rig.skirt = rig.day.parts.includes('bottom.skirt');
      writeOutfit(rig.day, this.data, this.row(s.id) + set.bones.length * 16);
    }
    for (const p of this.meshes.values()) p.mesh.setEnabled(false);
    for (const p of this.shadowMeshes.values()) p.mesh.setEnabled(false);
    this.hasLod = [...set.bodies.values()].some((b) => b.parts.has('lod.body'));
    for (const [key, sims] of groups) {
      const [bodyName, part] = splitKey(key);
      const body = set.bodies.get(bodyName)!;
      const pm = this.partMesh(body, part);
      if (!pm) continue;
      const ids = sims.map((x) => x.id);
      this.setInstances(pm, ids);
      pm.mesh.setEnabled(true);
      // Every Sim casts its sun shadow from the low-detail body and clothes (no hair, no face).
      if (part.startsWith('lod.')) {
        const sm = this.partMesh(body, part, true);
        if (sm) {
          this.setInstances(sm, ids);
          sm.mesh.setEnabled(true);
        }
      }
    }
    // Objects Sims sit, lie or stand at (for seat heights and object-use animations).
    this.objectById.clear();
    this.objects = world.objects.map((o: ObjectPlacement) => {
      const z = this.groundRow(o.z);
      // Its middle where it stands (things that stand anywhere on their tile); the bounds stay on tiles.
      const centre = objectCentre(o);
      const def = this.content?.object(o.def);
      const restful = !!(SEATS[o.def] || BEDS[o.def]) || !!def?.interactions.some((i) => i.pose === 'sit' || i.pose === 'lie');
      const lift = (o.tall ?? 0) / 100;
      const seat = seatOf(def, o.def);
      const stand = def?.stand ? { h: def.stand.height + lift, motion: def.stand.motion } : null;
      const paint = def?.held ? colorsKey(paintColors(this.assets.get(def.model, 'model'), o.colors)) : '';
      const held = def?.held ? { keys: [def.held.part].flat().map((part) => `${def.model}#${part}${paint ? `@${paint}` : ''}`), grip: def.held.grip } : null;
      return { def: o.def, rot: o.rot, seat: { ...seat, h: seat.h + lift }, stand, held, lift, restful, cx: centre.x, cz: z + centre.z - o.z, minX: o.x, minZ: z, maxX: o.x + o.w, maxZ: z + o.d };
    });
    world.objects.forEach((o, j) => this.objectById.set(o.id, this.objects[j]));
    this.seatIds.clear();
    world.objects.forEach((o, j) => this.tables.has(o.id) && this.seatIds.set(this.objects[j], o.id));
    const beds = world.objects.some((o) => BLANKET_BEDS.has(o.def));
    const bodies = new Set(this.sims.map((r) => r?.body.name));
    for (const [name, b] of this.blankets) {
      b.matrices.fill(0);
      b.mesh.thinInstanceBufferUpdated('matrix');
      // (A life stage's quilt only where someone has that body: no draw call for nobody.)
      b.mesh.setEnabled(beds && (b.base || bodies.has(name)));
    }
    this.texture.update(this.data);
    this.uploaded.set(this.data);
    this.uploadedVisible.set(this.visible);
    this.posed = true;
  }

  /** `?debug` stress test: `n` extra animated Sims in a grid around (x, z), cloned from Sim 0. */
  private ghosts: { x: number; z: number; clip: string }[] = [];
  stress(n: number, x: number, z: number): void {
    const set = this.set;
    const base = this.sims.find(Boolean);
    if (!set || !base) return;
    const count = this.sims.length;
    const clips = ['idle', 'walk', 'talk', 'foldArms', 'idle', 'walk'];
    // Young adults of both genders in a T-shirt, trousers and sneakers.
    const bodies = [this.baseBody('male'), this.baseBody('female')];
    const clothes = ['top.tee', 'bottom.trousers', 'shoes.sneakers'];
    const parts = [
      ...FACE_PARTS,
      'hair.short',
      ...clothes,
      ...new Set(bodies.flatMap((b) => skinChunks(b, clothes))),
      'lod.body',
      ...clothes.map((c) => `lod.${c}`),
    ];
    this.ghosts = [];
    for (let k = 0; k < n && count + k < MAX_CHARACTERS; k++) {
      const id = count + k;
      this.ghosts.push({
        x: x + (k % 5) * 1.1,
        z: z + Math.floor(k / 5) * 1.1,
        clip: clips[k % clips.length],
      });
      const body = bodies[k % 2];
      const rig: SimRig = {
        ...base,
        id,
        body,
        scale: (BASE_HEIGHT[body.name] ?? 1.72) / body.height,
        head: 1,
        stoop: 0,
        build: 1,
        size: 1,
        cur: player(),
        prev: player(),
        upper: player(),
        visible: true,
        x: 0,
        z: 0,
        yaw: k,
        seed: hash(id),
        face: { ...NEUTRAL },
        faceState: new Float32Array(set.morphs?.channels.length ?? 0),
        speech: null,
      };
      this.sims[id] = rig;
      const look = { body: '#5F86C9', skin: '#C08A64', hair: '#2B2320', hairStyle: 'short' as const, height: 1, top: 'top.tee', bottom: 'bottom.trousers', shoes: 'shoes.sneakers' };
      writeOutfit(outfitFor(id, body.name, look, body), this.data, this.row(id) + set.bones.length * 16);
    }
    // The ghosts' meshes (and the low-detail ones' shadow copies), even where nobody in the world wears them.
    for (const body of bodies) {
      for (const part of parts) {
        for (const shadow of part.startsWith('lod.') ? [false, true] : [false]) {
          const pm = this.partMesh(body, part, shadow);
          if (pm && !pm.mesh.isEnabled()) {
            pm.all = new Float32Array(0);
            pm.mesh.setEnabled(true);
          }
        }
      }
    }
    for (const [key, pm] of [...this.meshes, ...this.shadowMeshes]) {
      if (!pm.mesh.isEnabled()) continue;
      const [bodyName, part] = splitKey(key);
      const ids = [...pm.all.filter((_, j) => j % 4 === 0)];
      this.ghosts.forEach((_, k) => {
        if (bodies[k % 2].name === bodyName && parts.includes(part)) ids.push(count + k);
      });
      this.setInstances(pm, ids);
    }
  }

  private poseGhosts(count: number, dt: number, time: number): void {
    const set = this.set!;
    this.ghosts.forEach((g, k) => {
      const i = count + k;
      const rig = this.sims[i]!;
      if (rig.cur.key !== g.clip) this.start(rig, g.clip, 0.2, 1, true, 0, 0, 0);
      advance(rig.cur, dt);
      rig.w = 1;
      sampleClip(set, rig.cur.clip!, rig.cur.time, this.qA, this.pA);
      blendInto(this.qA, this.pA, set.sourceRestQ, this.pA, 0.6, this.fingerMask);
      resetAdditive(this.add);
      this.procedural(rig, i, 0, 0, null, Pose.Stand, false, time, dt);
      const pl = this.place;
      pl.qx = 0;
      pl.qy = Math.sin(rig.yaw / 2);
      pl.qz = 0;
      pl.qw = Math.cos(rig.yaw / 2);
      pl.x = g.x;
      pl.y = 0;
      pl.z = g.z;
      pl.scale = rig.scale;
      pl.head = rig.head;
      pl.build = rig.build;
      skin(set, rig.body, this.qA, this.pA, this.add, pl, this.scratch, this.data, this.row(i), this.headTmp);
      this.visible[i] = 1;
    });
  }

  /** Stops listening for model files (the renderer goes; the scene's meshes go with it). */
  dispose(): void {
    this.unlisten();
  }

  /** Removes the Sims (end of a session); meshes stay cached. */
  clear(): void {
    this.sims = [];
    this.ghosts = [];
    this.objects = [];
    this.objectById.clear();
    this.visible.fill(0);
    for (const p of this.meshes.values()) p.mesh.setEnabled(false);
    for (const b of this.blankets.values()) b.mesh.setEnabled(false);
    for (let i = 0; i < MAX_CHARACTERS; i++) {
      this.writeProp(this.heldProp, i, null);
      this.writeProp(this.toolProp, i, null);
      this.writeProp(this.leftProp, i, null);
      this.writeProp(this.rightProp, i, null);
    }
    // Props built without their files are built (and their files tried) afresh in the next session.
    this.props.dropFailed();
  }

  // --- carried things and tools ------------------------------------------------------------

  /**
   * A prop's template and matrices; starts loading it the first time (null until loaded, and for a part
   * its model doesn't have). `<model key>#<part>`: a moving part of an object's model, in its own frame
   * (`…@<colours>`: in those colours, `colorsKey`'s).
   */
  private prop(key: string): Prop | null {
    const known = this.props.built(key);
    if (known !== undefined) return known;
    void this.props.get(key, () => this.buildProp(key));
    return null;
  }

  /** Per frame: a prop nobody holds or works with isn't drawn (no draw call for its empty instances). */
  private readonly showProp = (p: Prop | null, key: string): void => {
    if (!p) return;
    const used = this.heldProp.includes(key) || this.toolProp.includes(key) || this.leftProp.includes(key) || this.rightProp.includes(key);
    for (const mesh of p.meshes) {
      mesh.isVisible = used;
      if (used) mesh.thinInstanceBufferUpdated('matrix');
    }
  };

  /** Builds prop `key` (`prop`); without its file, a small block (it's built again when the file comes). */
  private async buildProp(key: string): Promise<Prop | null> {
    const at = key.indexOf('#');
    const p =
      at < 0
        ? await buildModel(this.scene, `prop:${key}`, this.assets.get(key, 'model'), [1, 1], { standIn: ITEM_STAND_IN }).then((t) => ({ meshes: t.meshes, top: [0, 0, 0] as const, failed: t.failed }))
        : await this.partProp(key.slice(0, at), ...partColors(key.slice(at + 1)));
    if (!p) return null;
    const matrices = new Float32Array(MAX_CHARACTERS * 16);
    for (const mesh of p.meshes) {
      mesh.isPickable = false;
      mesh.thinInstanceSetBuffer('matrix', matrices, 16, false);
      mesh.alwaysSelectAsActiveMesh = true;
    }
    return { meshes: p.meshes, matrices, top: p.top, failed: p.failed };
  }

  /**
   * Moving part `part` of model `model` on its own (a held instrument), in the part's own frame (its node's
   * origin and axes at rest), painted in `colors` as the object is; and its top (the highest point: a mop's
   * handle); null if the model has no such part (nothing is drawn, and nothing is hidden on
   * the object either). Without the model's file, a small block in the hand.
   */
  private async partProp(model: string, part: string, colors?: PartColors): Promise<{ meshes: Mesh[]; top: readonly [number, number, number]; failed?: string[] } | null> {
    const t: ModelTemplate = await buildModel(this.scene, `prop:${model}#${part}`, painted(this.assets.get(model, 'model'), colors), [1, 1], { standIn: ITEM_STAND_IN });
    if (t.failed) return { meshes: t.meshes, top: [0, t.height, 0], failed: t.failed };
    const motion = t.motion;
    const j = motion?.names?.indexOf(part) ?? -1;
    const keep = t.meshes.filter((_, k) => j >= 0 && motion!.parts[k] === j);
    for (const mesh of t.meshes) if (!keep.includes(mesh)) mesh.dispose(false, false);
    for (const glow of t.glows ?? []) glow.mesh.dispose(false, false);
    if (j < 0 || !motion?.rests) return null;
    const local = motion.rests[j].clone().invert();
    const top: [number, number, number] = [0, -Infinity, 0];
    for (const mesh of keep) {
      mesh.bakeTransformIntoVertices(local);
      const p = mesh.getVerticesData('position') ?? [];
      for (let v = 1; v < p.length; v += 3) if (p[v] > top[1]) [top[0], top[1], top[2]] = [p[v - 1], p[v], p[v + 1]];
    }
    return { meshes: keep, top };
  }

  /**
   * What Sim row `i` holds of the object it uses (content `held`; its own part is hidden then, the
   * model's `motions`), placed by the hands as just posed: a barbell across both palms, a dumbbell in
   * each, a guitar's neck in the left hand with its body against the belly, a mop's handle in the hands
   * with its foot on the floor ahead.
   */
  private writeHeld(rig: SimRig, i: number, pose: number, moving: boolean): void {
    const held = !moving && rig.tagKnown && pose !== Pose.Sit && rig.cur.key !== 'sitDown' && rig.cur.key !== 'standUp' ? rig.useObj?.held : undefined;
    const grip = held && (held.grip === 'bar') === (pose === Pose.Lie) ? held.grip : null;
    const left = this.writeProp(this.leftProp, i, grip === 'pair' ? held!.keys[0] : null);
    const right = this.writeProp(this.rightProp, i, grip ? (held!.keys[grip === 'pair' ? 1 : 0] ?? null) : null);
    if (!grip || !this.set) return;
    const B = this.set.bone;
    const s = rig.scale;
    const size = Math.min(1, s);
    const f = this.frame3;
    const pl = this.palm;
    const fx = Math.sin(rig.yaw);
    const fz = Math.cos(rig.yaw);
    if (grip === 'pair') {
      // A dumbbell in each fist, lying across it (along the Sim's right), level.
      for (const [m, b, side] of [
        [left, B.hand_l, -1],
        [right, B.hand_r, 1],
      ] as const) {
        if (!m) continue;
        this.palmAt(rig, b, side, pl);
        setFrame(f, fz, 0, -fx, 0, 1, 0, fx, 0, fz, pl[0], pl[1], pl[2]);
        this.placeFrame(m, i, f, size);
      }
      return;
    }
    if (!right) return;
    this.palmAt(rig, B.hand_l, -1, pl);
    const lx = pl[0];
    const ly = pl[1];
    const lz = pl[2];
    this.palmAt(rig, B.hand_r, 1, pl);
    if (grip === 'bar') {
      // Across both palms, level.
      unit(pl[0] - lx, pl[1] - ly, pl[2] - lz);
      const x = UNIT[0];
      const y = UNIT[1];
      const z = UNIT[2];
      unit(-x * y, 1 - y * y, -z * y);
      const ux = UNIT[0];
      const uy = UNIT[1];
      const uz = UNIT[2];
      setFrame(f, x, y, z, ux, uy, uz, y * uz - z * uy, z * ux - x * uz, x * uy - y * ux, (pl[0] + lx) / 2, (pl[1] + ly) / 2, (pl[2] + lz) / 2);
      this.placeFrame(right, i, f, 1);
      return;
    }
    const top = this.props.built(held!.keys[0])?.top;
    if (!top) return;
    if (grip === 'guitar') {
      // Upright along its neck from where its body rests against the belly (in front of the spine's
      // base, a little to the right) to the left hand; its face to the front.
      const t = this.scratch.worldT;
      const g = GUITAR;
      const bx = t[B.spine_01 * 3] + (fx * g.out + fz * g.right) * s;
      const by = t[B.spine_01 * 3 + 1] + (g.up + g.upShort * (1 - this.guitarReach(rig))) * s;
      const bz = t[B.spine_01 * 3 + 2] + (fz * g.out - fx * g.right) * s;
      unit(lx - bx, ly - by, lz - bz);
      const yx = UNIT[0];
      const yy = UNIT[1];
      const yz = UNIT[2];
      const d = fx * yx + fz * yz;
      unit(fx - yx * d, -yy * d, fz - yz * d);
      const zx = UNIT[0];
      const zy = UNIT[1];
      const zz = UNIT[2];
      const along = top[1] * g.body * size;
      setFrame(f, yy * zz - yz * zy, yz * zx - yx * zz, yx * zy - yy * zx, yx, yy, yz, zx, zy, zz, bx - yx * along, by - yy * along, bz - yz * along);
      this.placeFrame(right, i, f, size);
      return;
    }
    // A pole: its top in the hands (between them, if both are on it), its foot on the floor ahead, as far
    // as it reaches; turned only up and down from facing the way the Sim faces.
    const both = Math.hypot(pl[0] - lx, pl[1] - ly, pl[2] - lz) < 0.35 * s;
    const gx = both ? (pl[0] + lx) / 2 : pl[0];
    const gy = both ? (pl[1] + ly) / 2 : pl[1];
    const gz = both ? (pl[2] + lz) / 2 : pl[2];
    const reach = Math.hypot(top[1], top[2]) * size;
    const high = Math.min(gy - rig.floorY, reach * 0.99);
    const ahead = Math.sqrt(reach * reach - high * high);
    // Its axis from the foot to the top: as built (up, and towards its back), and as held (up, and back to the hands).
    const turn = Math.atan2(-ahead, high) - Math.atan2(top[2], top[1]);
    const c = Math.cos(turn);
    const n = Math.sin(turn);
    setFrame(f, fz, 0, -fx, fx * n, c, fz * n, fx * c, -n, fz * c, gx + fx * ahead, rig.floorY, gz + fz * ahead);
    this.placeFrame(right, i, f, size);
  }

  /** How long the guitar Sim `rig` plays is for its size (`GUITAR`): 0 short (a ukulele) to 1 long (a guitar; also while it loads). */
  private guitarReach(rig: SimRig): number {
    const top = this.props.built(rig.useObj?.held?.keys[0] ?? '')?.top;
    return top ? clamp((top[1] * Math.min(1, rig.scale) / rig.scale - GUITAR.short) / (GUITAR.long - GUITAR.short), 0, 1) : 1;
  }

  /** Writes a prop placed by the frame `f` (its x, y and z axes, then its origin; scaled by `scale`) into row `i` of `m`. */
  private placeFrame(m: Float32Array, i: number, f: Float32Array, scale: number): void {
    const o = i * 16;
    for (let r = 0; r < 3; r++) {
      m[o + r * 4] = f[r * 3] * scale;
      m[o + r * 4 + 1] = f[r * 3 + 1] * scale;
      m[o + r * 4 + 2] = f[r * 3 + 2] * scale;
      m[o + r * 4 + 3] = 0;
    }
    m[o + 12] = f[9];
    m[o + 13] = f[10];
    m[o + 14] = f[11];
    m[o + 15] = 1;
  }

  /** Sim row `i` draws prop `key` (null: none) in `slot`; the old one's matrix is cleared. */
  private writeProp(slot: (string | null)[], i: number, key: string | null): Float32Array | null {
    const old = slot[i];
    if (old && old !== key) this.props.built(old)?.matrices.fill(0, i * 16, i * 16 + 16);
    slot[i] = key;
    return key ? (this.prop(key)?.matrices ?? null) : null;
  }

  /** Writes a level prop (turned to `yaw`) at a point into row `i` of `m`. */
  private placeProp(m: Float32Array, i: number, x: number, y: number, z: number, yaw: number, scale: number): void {
    const c = Math.cos(yaw) * scale;
    const s = Math.sin(yaw) * scale;
    const o = i * 16;
    m.fill(0, o, o + 16);
    m[o] = c;
    m[o + 2] = -s;
    m[o + 5] = scale;
    m[o + 8] = s;
    m[o + 10] = c;
    m[o + 12] = x;
    m[o + 13] = y;
    m[o + 14] = z;
    m[o + 15] = 1;
  }

  /**
   * What Sim row `i` carries (`item`, from the snapshot's `held`): on the right palm, or held in
   * front between both palms (content `hold`), level and facing the way they face.
   */
  private writeCarried(rig: SimRig, i: number, item: ItemDef | undefined): void {
    const key = item ? (item.model ?? `item.${item.id}`) : null;
    const m = this.writeProp(this.heldProp, i, key);
    if (!m || !item || !this.set) return;
    const B = this.set.bone;
    const palm = this.palm;
    this.palmAt(rig, B.hand_r, 1, palm);
    let x = palm[0];
    let y = palm[1];
    let z = palm[2];
    if (item.hold === 'both') {
      this.palmAt(rig, B.hand_l, -1, palm);
      x = (x + palm[0]) / 2;
      y = (y + palm[1]) / 2;
      z = (z + palm[2]) / 2;
    }
    // The thing rests on the palms (its base a little below their centres).
    this.placeProp(m, i, x, y - HOLD_DROP * rig.scale, z, rig.yaw, Math.min(1, rig.scale));
  }

  /** The palm centre of hand bone `b` (`side` 1 right, -1 left) as last posed, into `out`. */
  private palmAt(rig: SimRig, b: number, side: number, out: Float32Array): void {
    const t = this.scratch.worldT;
    const q = this.scratch.worldQ;
    const g = rig.body.grip ?? DEFAULT_GRIP;
    const s = rig.scale;
    // (The grip is the left palm's offset from the wrist in the hand's rest frame; the right mirrors x.)
    const gx = -g[0] * side * s;
    const gy = g[1] * s;
    const gz = g[2] * s;
    const qx = q[b * 4];
    const qy = q[b * 4 + 1];
    const qz = q[b * 4 + 2];
    const qw = q[b * 4 + 3];
    // Rotated by the hand: v + 2 w (q x v) + 2 q x (q x v).
    const cx = qy * gz - qz * gy;
    const cy = qz * gx - qx * gz;
    const cz = qx * gy - qy * gx;
    out[0] = t[b * 3] + gx + 2 * (qw * cx + qy * cz - qz * cy);
    out[1] = t[b * 3 + 1] + gy + 2 * (qw * cy + qz * cx - qx * cz);
    out[2] = t[b * 3 + 2] + gz + 2 * (qw * cz + qx * cy - qy * cx);
  }

  /**
   * Cuddles (`CUDDLE`): finds the baby each grown-up cuddling holds (lying in the crib they use),
   * advances the cuddle (picking up, holding, putting back as it nears its end; a cuddle cut short
   * lets the baby settle back into the crib) and orders the rows so a held baby is posed after the
   * one holding them.
   */
  private cuddles(frame: FrameState, count: number, dt: number, gameRate: number, animRate: number): void {
    const { curr, layout } = frame;
    const H = layout.headerLen;
    const S = layout.simStride;
    const k = layout.sim;
    const act = k.action !== undefined && k.object !== undefined ? this.actionTags.indexOf('cuddle') : -1;
    const step = dt * animRate;
    // Seconds of animation per game minute at this speed.
    const perMinute = gameRate > 0 ? animRate / gameRate : Infinity;
    this.heldBy.fill(-1);
    for (let j = 0; j < count; j++) {
      const rig = this.sims[j];
      if (!rig) continue;
      const o = H + j * S;
      const obj = curr[o + k.object] | 0;
      let baby = -1;
      if (act >= 0 && (curr[o + k.action] | 0) === act && curr[o + k.pose] === Pose.Stand && !(curr[o + k.moving] > 0) && !curr[o + k.away] && !this.force) {
        for (let b = 0; b < count && baby < 0; b++) {
          const ob = H + b * S;
          if (b !== j && this.sims[b]?.body.stage === 'baby' && curr[ob + k.pose] === Pose.Lie && (curr[ob + k.object] | 0) === obj && !curr[ob + k.away]) baby = b;
        }
      }
      if (baby >= 0) {
        // Game minutes it lasts.
        const minutes = this.cuddleLength(this.objectById.get(obj)?.def ?? '');
        if (rig.cuddleT < 0 || rig.cuddleBaby !== baby) {
          // Picking up and putting back fit in (faster at speed).
          rig.cuddleT = 0;
          rig.cuddleOut = -1;
          rig.cuddleMin = 0;
          rig.cuddleRate = clamp((CUDDLE.pick + CUDDLE.put + 1.5) / (minutes * perMinute), 1, 6);
        }
        rig.cuddleBaby = baby;
        rig.cuddleT += step * rig.cuddleRate;
        rig.cuddleMin += dt * gameRate;
        if (rig.cuddleOut >= 0) rig.cuddleOut += step * rig.cuddleRate;
        else if (rig.cuddleT >= CUDDLE.pick && (minutes - rig.cuddleMin) * perMinute * rig.cuddleRate <= CUDDLE.put) rig.cuddleOut = 0;
        const t = rig.cuddleT;
        const u = rig.cuddleOut;
        // The feet: at the rail to reach in, a step back from it holding the baby.
        const ob = H + baby * S;
        const d = Math.hypot(curr[ob + k.x] - curr[o + k.x], this.groundRow(curr[ob + k.z]) - this.groundRow(curr[o + k.z]));
        const atRail = Math.max(0, d - CUDDLE.rail - CUDDLE.reach);
        const clear = Math.max(0, d - CUDDLE.rail - CUDDLE.clear);
        if (u < 0) {
          // Picking up: leaning in over the rail, the hands under the baby, lifting them out into the arms.
          rig.reachW = ramp(t, 0.15, 0.9) * (1 - ramp(t, 1.5, 0.9));
          rig.holdW = ramp(t, 1.5, 0.9);
          rig.liftW = clamp((t - 1.35) / 1.05, 0, 1);
          rig.cuddleStep = lerp(atRail * ramp(t, 0, 0.7), clear, ramp(t, 1.9, 0.7));
        } else {
          // Putting back: the same the other way round.
          rig.reachW = ramp(u, 0.1, 0.9) * (1 - ramp(u, 1.4, 0.8));
          rig.holdW = 1 - ramp(u, 0.1, 0.9);
          rig.liftW = 1 - clamp((u - 0.1) / 1.05, 0, 1);
          rig.cuddleStep = lerp(lerp(clear, atRail, ramp(u, 0, 0.6)), 0, ramp(u, 1.5, 0.7));
        }
      } else {
        // Not (or no longer) cuddling: cut short, the arms come down and the baby settles back into the crib.
        rig.cuddleT = -1;
        rig.cuddleOut = -1;
        rig.cuddleStep = 0;
        rig.reachW = Math.max(0, rig.reachW - step * 3);
        rig.holdW = Math.max(0, rig.holdW - step * 1.6);
        rig.liftW = Math.max(0, rig.liftW - step * 1.6);
        // Walking off (given an order), or the baby no longer lying there as a baby: back in the crib at once.
        const held = rig.cuddleBaby >= 0 ? this.sims[rig.cuddleBaby] : null;
        if (curr[o + k.moving] > 0 || held?.body.stage !== 'baby' || curr[H + rig.cuddleBaby * S + k.pose] !== Pose.Lie) rig.liftW = 0;
        if (rig.liftW <= 0) rig.cuddleBaby = -1;
      }
      if (rig.cuddleBaby >= 0 && rig.liftW > 0) this.heldBy[rig.cuddleBaby] = j;
    }
    let n = 0;
    for (let i = 0; i < count; i++) if (this.heldBy[i] < 0) this.order[n++] = i;
    for (let i = 0; i < count; i++) if (this.heldBy[i] >= 0) this.order[n++] = i;
  }

  /** How long a cuddle at an object of def `def` lasts (game minutes; its interaction shown as `cuddle`). */
  private cuddleLength(def: string): number {
    let m = this.cuddleMinutes.get(def);
    if (m === undefined) {
      m = this.content?.object(def)?.interactions.find((it) => it.anim === 'cuddle')?.minutes ?? 15;
      this.cuddleMinutes.set(def, m);
    }
    return m;
  }

  /** Where Sim row `i` (picking up or holding a baby) has the baby, as just posed: in their hands, or cradled on their forearms. */
  private writeCradle(i: number): void {
    const B = this.set!.bone;
    const t = this.scratch.worldT;
    const c = this.cradle;
    const rig = this.sims[i]!;
    // Reaching into the crib: between the hands; holding: half way along the forearms, a little forward.
    const h = smooth(rig.holdW);
    for (let a = 0; a < 3; a++) {
      const hands = (t[B.hand_l * 3 + a] + t[B.hand_r * 3 + a]) / 2;
      c[i * 3 + a] = lerp(hands, (hands + (t[B.lowerarm_l * 3 + a] + t[B.lowerarm_r * 3 + a]) / 2) / 2, h);
    }
    c[i * 3] += Math.sin(rig.yaw) * CRADLE.forward * rig.scale * h;
    c[i * 3 + 2] += Math.cos(rig.yaw) * CRADLE.forward * rig.scale * h;
  }

  /**
   * A baby in the arms of Sim row `h` (`CUDDLE`), `e` of the way (0..1) from lying in the crib (the
   * placement as posed) to cradled across their forearms, on the back with the head in the crook of
   * the left arm, raised a little and turned towards them. Moves the body's middle (half way up its
   * back) with the hands and turns it about that.
   */
  private cradled(rig: SimRig, h: number, e: number): void {
    const pl = this.place;
    const q = this.cradleQ;
    const holder = this.sims[h]!;
    const s = rig.scale;
    const my = rig.body.height * 0.5 * s;
    const mz = -lieBack(rig);
    // Lying in the crib: the middle.
    rotate(pl.qx, pl.qy, pl.qz, pl.qw, 0, my, mz, q, 8);
    const x0 = pl.x + q[8];
    const y0 = pl.y + q[9];
    const z0 = pl.z + q[10];
    // In the arms: head towards the holder's left, along their front.
    const yaw = Math.atan2(Math.cos(holder.yaw), -Math.sin(holder.yaw));
    const pitch = -Math.PI / 2 + CRADLE.tilt;
    qmul(0, Math.sin(yaw / 2), 0, Math.cos(yaw / 2), Math.sin(pitch / 2), 0, 0, Math.cos(pitch / 2), q, 0);
    qmul(q[0], q[1], q[2], q[3], 0, Math.sin(CRADLE.roll / 2), 0, Math.cos(CRADLE.roll / 2), q, 4);
    // Blended with the crib's (the shorter way round).
    const dot = pl.qx * q[4] + pl.qy * q[5] + pl.qz * q[6] + pl.qw * q[7];
    const sg = dot < 0 ? -1 : 1;
    let qx = lerp(pl.qx, sg * q[4], e);
    let qy = lerp(pl.qy, sg * q[5], e);
    let qz = lerp(pl.qz, sg * q[6], e);
    let qw = lerp(pl.qw, sg * q[7], e);
    const len = Math.hypot(qx, qy, qz, qw) || 1;
    qx /= len;
    qy /= len;
    qz /= len;
    qw /= len;
    const c = this.cradle;
    // (In the hands soon after they start lifting, turned round over the whole lift; cut short, back
    // over the rail into the crib.)
    const at = smooth(clamp(e * 2.2, 0, 1));
    const x = lerp(x0, c[h * 3], at);
    const y = lerp(y0, c[h * 3 + 1] + CRADLE.lift * s, at) + Math.sin(at * Math.PI) * (holder.cuddleT < 0 ? 0.2 : 0.04);
    const z = lerp(z0, c[h * 3 + 2], at);
    rotate(qx, qy, qz, qw, 0, my, mz, q, 8);
    pl.x = x - q[8];
    pl.y = y - q[9];
    pl.z = z - q[10];
    pl.qx = qx;
    pl.qy = qy;
    pl.qz = qz;
    pl.qw = qw;
  }

  /**
   * The grown-up's side of a cuddle (`CUDDLE`), by its weights: bent over the crib's rail with the
   * arms reaching down into it; and holding the baby cradled in the arms, looking down at them,
   * rocking from side to side.
   */
  private cuddlePose(rig: SimRig, time: number): void {
    const B = this.set!.bone;
    const add = this.add;
    const r = smooth(rig.reachW);
    const h = smooth(rig.holdW);
    if (r > 0.001) {
      // Bent over the rail (the taller, the further), the arms down into the crib, a little in.
      const lean = CUDDLE.lean * clamp(1 + (rig.body.height * rig.scale - CUDDLE.height) * 1.2, 0.75, 1.4);
      addRotation(add, B.spine_01, 1, 0, 0, lean * 0.67 * r);
      addRotation(add, B.spine_02, 1, 0, 0, lean * 0.33 * r);
      addRotation(add, B.upperarm_l, 0, 0, 1, 0.1 * r);
      addRotation(add, B.upperarm_r, 0, 0, 1, -0.1 * r);
      addRotation(add, B.upperarm_l, 1, 0, 0, -(lean + CUDDLE.arms) * r);
      addRotation(add, B.upperarm_r, 1, 0, 0, -(lean + CUDDLE.arms) * r);
      addRotation(add, B.lowerarm_l, 1, 0, 0, -0.15 * r);
      addRotation(add, B.lowerarm_r, 1, 0, 0, -0.15 * r);
      addRotation(add, B.Head, 1, 0, 0, 0.2 * r);
    }
    if (h > 0.001) {
      // Rocking: slowly from side to side, the shoulders turning with it.
      const sway = Math.sin(time * 2.1 + rig.seed * 6) * h;
      addRotation(add, B.spine_01, 0, 0, 1, 0.035 * sway);
      addRotation(add, B.spine_02, 0, 1, 0, 0.14 * sway);
      // Cradling: elbows bent, the forearms across the front.
      addRotation(add, B.upperarm_l, 0, 0, 1, -0.15 * h);
      addRotation(add, B.upperarm_r, 0, 0, 1, 0.15 * h);
      addRotation(add, B.upperarm_l, 1, 0, 0, -0.15 * h);
      addRotation(add, B.upperarm_r, 1, 0, 0, -0.15 * h);
      addRotation(add, B.lowerarm_l, 1, 0, 0, -1.0 * h);
      addRotation(add, B.lowerarm_r, 1, 0, 0, -1.0 * h);
      addRotation(add, B.lowerarm_l, 0, 1, 0, 1.1 * h);
      addRotation(add, B.lowerarm_r, 0, 1, 0, -1.1 * h);
      // Looking down at the baby (the head on the left).
      addRotation(add, B.neck_01, 1, 0, 0, 0.15 * h);
      addRotation(add, B.Head, 1, 0, 0, 0.3 * h);
      addRotation(add, B.Head, 0, 1, 0, -0.2 * h);
    }
  }

  /**
   * The tool on the station Sim row `i` cooks at (a chopping board, a pan, food on the grill; by
   * action tag): on its top, a little towards the cook.
   */
  private writeTool(rig: SimRig, i: number, tag: string, station: PlacedObject | null | undefined): void {
    const key = station ? (STATION_PROPS[tag] ?? null) : null;
    const m = this.writeProp(this.toolProp, i, key);
    if (!m || !station) return;
    const def = this.content?.object(station.def);
    const top = def?.surface?.height ?? this.content?.cooking.defaultSurfaceHeight ?? 0.9;
    const dx = rig.x - station.cx;
    const dz = rig.z - station.cz;
    const d = Math.hypot(dx, dz) || 1;
    const toward = Math.min(0.15, d * 0.3);
    this.placeProp(m, i, station.cx + (dx / d) * toward, rig.floorY + top + 0.005, station.cz + (dz / d) * toward, rig.yaw, 1);
  }

  /** Head-top position of Sim `i` (false if not drawn). */
  head(i: number, out: { x: number; y: number; z: number }): boolean {
    if (i < 0 || i >= MAX_CHARACTERS || !this.visible[i]) return false;
    out.x = this.heads[i * 3];
    out.y = this.heads[i * 3 + 1];
    out.z = this.heads[i * 3 + 2];
    return true;
  }

  /** Where Sim `i` stands (x, z), also when not drawn (another lot); false while away or unknown. */
  spot(i: number, out: { x: number; z: number }): boolean {
    if (i < 0 || i >= MAX_CHARACTERS || Number.isNaN(this.spots[i * 2])) return false;
    out.x = this.spots[i * 2];
    out.z = this.spots[i * 2 + 1];
    return true;
  }

  /**
   * Poses every Sim for this frame. `inView(x, z)` decides visibility; contact-shadow matrices
   * (one 4x4 per Sim row) are written into `shadows`.
   */
  /** `storey`: the storey in view; residents above it (or, viewing the basement, not in it) aren't drawn. */
  update(frame: FrameState, inView: (x: number, z: number) => boolean, shadows: Float32Array, storey = Infinity): void {
    const set = this.set;
    if (!set) return;
    const started = performance.now();
    const { prev, curr, alpha, layout, now } = frame;
    let dt = this.lastNow ? (now - this.lastNow) / 1000 : 0;
    this.lastNow = now;
    if (dt > 0.1) dt = 0.1;
    if (dt < 0) dt = 0;
    const speed = curr[layout.header.speed] | 0;
    const quiet = layout.header.calm !== undefined && curr[layout.header.calm] > 0;
    const gameSpeed = quiet ? Math.max(speed, QUIET_SPEED) : speed;
    const gameRate = GAME_RATE[gameSpeed] ?? 1;
    // How far a walker can get from `prev` to `curr` (at the faster of their speeds, over the steps between them).
    const prevSpeed = prev[layout.header.speed] | 0;
    const prevGameSpeed = layout.header.calm !== undefined && prev[layout.header.calm] > 0 ? Math.max(prevSpeed, QUIET_SPEED) : prevSpeed;
    const reach = Math.max(SNAP_DISTANCE, WALK_PER_TICK * WALK_FASTEST * WALK_SLACK * Math.max(gameRate, GAME_RATE[prevGameSpeed] ?? 1) * frame.steps);
    const animRate = this.debugRate ?? Math.min(gameRate, 2.5);
    // Paused (the pause button, Buy and Build mode, menus): everyone freezes mid-motion, walk
    // cycles, breathing, blinks and glances included.
    if (gameSpeed === 0 && this.debugRate === null) dt = 0;
    this.clock += dt;
    const time = this.clock;
    const count = Math.min(curr[layout.header.simCount], MAX_CHARACTERS);
    const H = layout.headerLen;
    const S = layout.simStride;
    const k = layout.sim;
    const NB = set.bones.length;
    if (layout.actions && layout.actions !== this.actionTags) this.actionTags = layout.actions;
    const hasAction = k.action !== undefined && k.object !== undefined;

    // Group conversations: those listening in (role 3, the one who started it as partner), and
    // the middle of each group, everyone in it counted.
    const groups = this.groups;
    const middle = this.groupMiddle;
    groups.fill(0, 0, count * 3);
    for (let j = 0; j < count; j++) {
      const o = H + j * S;
      const l = curr[o + k.partner] | 0;
      if ((curr[o + k.role] | 0) !== 3 || l < 0 || l >= count || groups[l * 3] >= 2) continue;
      groups[l * 3 + 1 + groups[l * 3]++] = j;
    }
    for (let l = 0; l < count; l++) {
      const n = groups[l * 3];
      if (!n) continue;
      const t = curr[H + l * S + k.partner] | 0;
      // The one who started it, their partner and those listening in.
      let sx = 0;
      let sz = 0;
      let members = 0;
      for (let j = 0; j < n + 2; j++) {
        const m = j === 0 ? l : j === 1 ? t : groups[l * 3 + j - 1];
        if (m < 0 || m >= count) continue;
        sx += curr[H + m * S + k.x];
        sz += this.groundRow(curr[H + m * S + k.z]);
        members++;
      }
      middle[l * 2] = sx / members;
      middle[l * 2 + 1] = sz / members;
    }

    // Cuddles: who holds which baby (posed after them, in their arms).
    this.cuddles(frame, count, dt, gameRate, animRate);

    for (let oi = 0; oi < count; oi++) {
      const i = this.order[oi];
      const rig = this.sims[i];
      const o = H + i * S;
      const cx = curr[o + k.x];
      const cz = this.groundRow(curr[o + k.z]);
      const pz = this.groundRow(prev[o + k.z]);
      const ch = k.height !== undefined ? curr[o + k.height] : 0;
      let tx = cx;
      let tz = cz;
      let th = ch;
      let tyaw = curr[o + k.yaw];
      const walking = curr[o + k.moving] > 0 && prev[o + k.moving] > 0;
      if (walking ? Math.hypot(cx - prev[o + k.x], cz - pz) < reach : Math.abs(cx - prev[o + k.x]) + Math.abs(cz - pz) < SNAP_DISTANCE) {
        tx = prev[o + k.x] + (cx - prev[o + k.x]) * alpha;
        tz = pz + (cz - pz) * alpha;
        if (k.height !== undefined) th = prev[o + k.height] + (ch - prev[o + k.height]) * alpha;
        tyaw = lerpAngle(prev[o + k.yaw], tyaw, alpha);
      }
      const away = curr[o + k.away] !== 0;
      this.spots[i * 2] = away ? NaN : tx;
      this.spots[i * 2 + 1] = tz;
      const show = !!rig && !away && inView(tx, tz) && storeyShown(Math.floor(th + 0.01), storey);
      if (!rig || !show) {
        if (this.visible[i]) {
          this.data.fill(0, this.row(i), this.row(i) + NB * 16);
          shadows.fill(0, i * 16, i * 16 + 16);
          for (const b of this.blankets.values()) b.matrices.fill(0, i * 16, i * 16 + 16);
        }
        if (rig) rig.cover = 0;
        this.visible[i] = 0;
        if (rig) rig.visible = false;
        this.writeProp(this.heldProp, i, null);
        this.writeProp(this.toolProp, i, null);
        this.writeProp(this.leftProp, i, null);
        this.writeProp(this.rightProp, i, null);
        continue;
      }
      const dir = (rig.dir = this.directions[i] ?? null);
      rig.floorY = th * WALL_HEIGHT;
      const pose = curr[o + k.pose];
      const moving = curr[o + k.moving] > 0;
      // Calling someone over (sim-core's `call`): a wave, looking at them.
      const call = k.call === undefined ? 0 : curr[o + k.call] | 0;
      const social = (curr[o + k.anim] | 0) || (call > 0 ? Anim.Wave : 0);
      const role = curr[o + k.role] | 0;
      const partner = call > 0 ? call - 1 : curr[o + k.partner] | 0;
      const outcome = curr[o + k.outcome] | 0;
      rig.emotion = curr[o + k.emotion] | 0;
      const mood = k.mood !== undefined ? curr[o + k.mood] : 1;
      rig.warmth = clamp((mood - MOOD_COLD) / (MOOD_WARM - MOOD_COLD), 0, 1);
      // What the Sim is doing (sim-core's `object` / `action` fields; older layouts: guessed from position).
      let useObj: PlacedObject | null | undefined;
      let tag = '';
      let usingId = -1;
      if (hasAction) {
        const objId = curr[o + k.object];
        usingId = objId | 0;
        const act = curr[o + k.action];
        useObj = objId >= 0 ? (this.objectById.get(objId) ?? null) : null;
        tag = act >= 0 ? (this.actionTags[act | 0] ?? '') : '';
      }

      // Position / speed / facing.
      const appeared = !rig.visible;
      if (appeared) {
        rig.x = tx;
        rig.z = tz;
        rig.yaw = tyaw;
        rig.speed = 0;
        rig.lastPose = pose;
      }
      const dx = tx - rig.x;
      const dz = tz - rig.z;
      const dist = Math.hypot(dx, dz);
      if (dt > 0) {
        const inst = dist / dt;
        rig.speed += (Math.min(inst, 20) - rig.speed) * (1 - Math.exp(-dt * 10));
      }
      if (moving || dist > SNAP_DISTANCE || !rig.visible) {
        rig.x = tx;
        rig.z = tz;
      } else {
        // Settle into seats / beds / next to objects instead of popping.
        const f = 1 - Math.exp(-dt * 9);
        rig.x += dx * f;
        rig.z += dz * f;
      }
      let faceYaw = tyaw;
      // A bed lain on across (`BEDS` `across`): turned a quarter, feet to its right.
      rig.lieAcross = pose === Pose.Lie && !!BEDS[(useObj ?? this.objectAt(rig.x, rig.z))?.def ?? '']?.across;
      if (rig.lieAcross) faceYaw = wrapAngle(tyaw + Math.PI / 2);
      else if (pose === Pose.Sit) {
        const seat = this.seatUnder(rig, useObj)?.seat;
        // A seat facing the object's back (`SEATS` `back`, content `seat.facing`): turned round, facing into it (a
        // dressing table's mirror, an upright piano); or along it (`Seat.along`: a bath), turned a quarter.
        if (seat?.back) faceYaw = wrapAngle(tyaw + Math.PI);
        else if (seat?.along) faceYaw = wrapAngle(tyaw + (seat.along * Math.PI) / 2);
      }
      // Cuddling: turned to the baby in the crib.
      const cuddled = rig.cuddleT >= 0 && rig.cuddleBaby >= 0 ? this.sims[rig.cuddleBaby] : null;
      if (cuddled) faceYaw = Math.atan2(curr[H + cuddled.id * S + k.x] - rig.x, this.groundRow(curr[H + cuddled.id * S + k.z]) - rig.z);
      let partnerRig = partner >= 0 && partner < count ? this.sims[partner] : null;
      // Listening in (role 3, a group conversation): `partner` started it; they look at whichever
      // of the two is speaking, and stand the way sim-core turned them (to the middle of the group).
      const listening = role === 3;
      if (listening && partnerRig) {
        const other = curr[H + partner * S + k.partner] | 0;
        if (other >= 0 && other < count && !speaking(Math.min(partner, other), 1, time)) partnerRig = this.sims[other];
      }
      // The two talking face each other; with others listening in, they open up to the group: the
      // body faces its middle (as sim-core turns them), the head and eyes stay on the one speaking,
      // and while speaking themselves they glance round at those listening now and then.
      const leader = role === 1 ? i : role === 2 ? partner : -1;
      const listeners = leader >= 0 && leader < count ? groups[leader * 3] : 0;
      rig.audience = null;
      if (social && !listening && partnerRig && partnerRig.visible && !moving && pose === Pose.Stand) {
        if (listeners) {
          faceYaw = Math.atan2(middle[leader * 2] - rig.x, middle[leader * 2 + 1] - rig.z);
          if (speaking(Math.min(i, partner), role, time)) {
            const pick = Math.floor(hash(Math.floor(time / 1.4 + rig.seed * 3) * 7 + i) * (listeners + 1));
            if (pick > 0) rig.audience = this.sims[groups[leader * 3 + pick]] ?? null;
          }
        } else faceYaw = Math.atan2(partnerRig.x - rig.x, partnerRig.z - rig.z);
      }
      const dyaw = wrapAngle(faceYaw - rig.yaw);
      // Walkers turn as quickly as the game runs (up to 10x): sped up, the corners of their way come sooner.
      const turn = moving ? clamp(gameRate, 1, 10) : 1;
      const maxTurn = (moving ? 9 : 6) * dt * turn;
      rig.yaw = dir ? wrapAngle(faceYaw) : wrapAngle(rig.yaw + clamp(dyaw * (1 - Math.exp(-dt * 12 * turn)), -maxTurn, maxTurn));
      rig.visible = true;
      this.visible[i] = 1;

      // What it carries (`items` index + 1), and the arms that carry it.
      const held = k.held !== undefined ? curr[o + k.held] | 0 : 0;
      const item = held > 0 && layout.items ? this.content?.item(layout.items[held - 1] ?? '') : undefined;
      const carry = item ? (item.hold === 'both' ? 'carry' : 'carryOne') : '';

      // Activity → clip.
      rig.useId = usingId;
      this.choose(rig, i, pose, moving, social, role, outcome, partnerRig, time, dt * animRate, useObj, tag, carry, dt, appeared);
      rig.lastPose = pose;

      // Advance and blend.
      const step = dt * animRate;
      advance(rig.cur, GAITS.has(rig.cur.key) || rig.cur.key === 'jog' ? dt : step);
      advance(rig.prev, GAITS.has(rig.prev.key) || rig.prev.key === 'jog' ? dt : step);
      advance(rig.upper, step);
      // Carrying while walking: the arms bob in step with the feet.
      if (GAITS.has(rig.cur.key) && rig.upper.clip && rig.upper.key.startsWith('carry')) rig.upper.time = (rig.cur.time / rig.cur.clip!.duration) * rig.upper.clip.duration;
      rig.w = Math.min(1, rig.w + dt * rig.fadeRate * Math.max(1, animRate));
      rig.upperW += clamp(rig.upperTarget - rig.upperW, -dt * 4, dt * 4);
      if (dir?.still) {
        rig.cur.time = dir.time ?? rig.cur.time;
        rig.w = 1;
        rig.upperW = 0;
      }

      sampleClip(set, rig.cur.clip!, rig.cur.time, this.qA, this.pA);
      const w = smooth(rig.w);
      if (w < 1 && rig.prev.clip) {
        sampleClip(set, rig.prev.clip, rig.prev.time, this.qB, this.pB);
        blendInto(this.qB, this.pB, this.qA, this.pA, w, null);
        this.qA.set(this.qB);
        this.pA.set(this.pB);
      }
      if (rig.upperW > 0.001 && rig.upper.clip) {
        sampleClip(set, rig.upper.clip, rig.upper.time, this.qB, this.pB);
        blendInto(this.qA, this.pA, this.qB, this.pB, smooth(rig.upperW), this.upperMask);
      }

      if (this.force === 'rest') {
        this.qA.set(set.sourceRestQ);
        this.pA.set(set.sourcePelvis);
      }
      // Pyjamas in bed.
      const asleep = pose === Pose.Lie && LYING.has(rig.cur.key) && w > 0.5 && (!rig.tagKnown || rig.tag === 'sleep' || rig.tag === 'nap');
      if (asleep !== rig.asleep && rig.day && rig.night) {
        rig.asleep = asleep;
        writeOutfit(asleep ? rig.night : rig.day, this.data, this.row(i) + NB * 16);
      }
      // Hand pose: the clips clench fists; open them according to the activity.
      blendInto(this.qA, this.pA, set.sourceRestQ, this.pA, rig.hands, this.fingerMask);

      // Procedural layers.
      resetAdditive(this.add);
      this.procedural(rig, i, social, role, partnerRig, pose, moving, time, dt);
      // A baby in someone's arms: knees drawn up a little (the baby's lying clips have them drawn up already).
      const holder = this.heldBy[i] >= 0 ? this.sims[this.heldBy[i]] : null;
      const inArms = holder?.visible ? smooth(holder.liftW) : 0;
      if (inArms > 0) {
        const B = set.bone;
        const k = inArms * (rig.cur.key === 'lie' ? 1 : 0.3);
        addRotation(this.add, B.thigh_l, 1, 0, 0, -0.55 * k);
        addRotation(this.add, B.thigh_r, 1, 0, 0, -0.4 * k);
        addRotation(this.add, B.calf_l, 1, 0, 0, 0.8 * k);
        addRotation(this.add, B.calf_r, 1, 0, 0, 0.65 * k);
      }

      // Placement (root offsets blend with the clips).
      const rootY = lerp(rig.prev.rootY, rig.cur.rootY, w);
      const rootFwd = lerp(rig.prev.rootFwd, rig.cur.rootFwd, w) + rig.approach;
      const pitch = lerp(rig.prev.rootPitch, rig.cur.rootPitch, w);
      const sy = Math.sin(rig.yaw / 2);
      const cyw = Math.cos(rig.yaw / 2);
      const sp = Math.sin(pitch / 2);
      const cp = Math.cos(pitch / 2);
      // q = yaw(Y) * pitch(X)
      const pl = this.place;
      pl.qx = cyw * sp;
      pl.qy = sy * cp;
      pl.qz = -sy * sp;
      pl.qw = cyw * cp;
      pl.x = rig.x + Math.sin(rig.yaw) * rootFwd;
      pl.y = rootY + rig.floorY;
      pl.z = rig.z + Math.cos(rig.yaw) * rootFwd;
      this.stepPlace(rig, pl, step);
      pl.scale = rig.scale;
      pl.head = rig.head;
      pl.build = rig.build;
      // Sitting or lying on a seat that moves (a rocking chair, a swing): along with it.
      const ride = (pose === Pose.Sit || pose === Pose.Lie) && usingId >= 0 && this.rideOf ? this.rideOf(usingId) : null;
      if (ride) this.riding(ride);
      if (inArms > 0) this.cradled(rig, this.heldBy[i], inArms);
      skin(set, rig.body, this.qA, this.pA, this.add, pl, this.scratch, this.data, this.row(i), this.headTmp);
      if (rig.cuddleBaby >= 0 && rig.liftW > 0) this.writeCradle(i);
      this.writeBlanket(rig, i, pose, dt, appeared);
      this.writeCarried(rig, i, item);
      this.writeHeld(rig, i, pose, moving);
      this.writeTool(rig, i, moving ? '' : tag, useObj);

      // Head top (speech bubbles, selection marker): a little above the head joint.
      const hs = rig.scale * rig.head;
      this.heads[i * 3] = this.headTmp[0];
      this.heads[i * 3 + 1] = this.headTmp[1] + (rig.body.headAbove + HEAD_MARGIN) * hs;
      this.heads[i * 3 + 2] = this.headTmp[2];

      // Contact shadow under standing / sitting Sims.
      const r = pose === Pose.Lie ? 0 : 0.62 * hs;
      const m = i * 16;
      shadows.fill(0, m, m + 16);
      shadows[m] = r;
      shadows[m + 5] = 1;
      shadows[m + 10] = r;
      // (Under them inside a thing they use from inside, and on its floor: `STEP_IN`.)
      const stepped = !!rig.inside || rig.outT >= 0;
      shadows[m + 12] = stepped ? pl.x : rig.x;
      shadows[m + 13] = 0.02 + (stepped ? pl.y : rig.floorY);
      shadows[m + 14] = stepped ? pl.z : rig.z;
      shadows[m + 15] = 1;
    }
    if (this.ghosts.length) this.poseGhosts(count, dt, time);
    this.faceDetail(count + this.ghosts.length);
    this.writeRing(this.reducedMotion ? 0 : now * 0.001);
    for (let i = count + this.ghosts.length; i < MAX_CHARACTERS; i++) {
      if (!this.visible[i]) continue;
      this.visible[i] = 0;
      this.data.fill(0, this.row(i), this.row(i) + NB * 16);
      shadows.fill(0, i * 16, i * 16 + 16);
    }
    for (const b of this.blankets.values()) {
      b.mesh.thinInstanceBufferUpdated('matrix');
      b.mesh.thinInstanceBufferUpdated('blanketExt');
    }
    this.props.forEachBuilt(this.showProp);
    this.pullChairs();
    this.upload();
    this.updateMs += (performance.now() - started - this.updateMs) * 0.1;
  }

  // --- activity ------------------------------------------------------------------------

  /**
   * Picks the clip for this frame and starts a crossfade when it changes. `useObj` / `tag` come
   * from the snapshot (`undefined` with older layouts: the object is guessed from the position).
   */
  private choose(
    rig: SimRig,
    i: number,
    pose: number,
    moving: boolean,
    social: number,
    role: number,
    outcome: number,
    partner: SimRig | null,
    time: number,
    step: number,
    useObj: PlacedObject | null | undefined,
    tag: string,
    /** The upper-body clip for what the Sim carries ('' = nothing). */
    carry: string,
    /** Seconds since the last frame (0 while paused). */
    dt: number,
    /** Drawn again (back in view, on the storey shown, home from away): no fading in from what they did when last seen. */
    fresh: boolean,
  ): void {
    const pair = partner ? Math.min(i, partner.id) : i;
    const set = this.set!;
    const known = useObj !== undefined;
    const act = ACTIONS[tag];
    rig.tag = tag;
    rig.tagKnown = known;
    rig.actTime += step;
    rig.stepTime += step;
    let key = 'idle';
    let rootY = 0;
    let rootFwd = 0;
    let pitch = 0;
    let repeat = false;
    let fade = DEFAULT_FADE;
    let upper = '';
    let rate = 1;
    let hands: number = act?.hands ?? HANDS.relaxed;
    // Step towards a partner (hugs, kisses, fights): eased towards `reach` at `reachRate`.
    let reach = 0;
    let reachRate = 4;
    // Using a thing from inside or up against it this frame (`stepIn`).
    let inside = false;

    if (this.force) {
      key = this.force;
    } else if (pose === Pose.Lie) {
      const obj = (known ? useObj : null) ?? this.objectAt(rig.x, rig.z);
      const bed = (obj && BEDS[obj.def]) || DEFAULT_BED;
      rig.bed = obj;
      // A bunk's second sleeper (a hair to the right of its centre, see sim-core `slot_position`) is on top.
      const bunk = obj && BUNKS[obj.def];
      const across = obj ? (rig.x - obj.cx) * Math.cos((obj.rot * Math.PI) / 2) - (rig.z - obj.cz) * Math.sin((obj.rot * Math.PI) / 2) : 0;
      rig.bedH = (bunk ? bunk[across > 0 ? 1 : 0] : bed.h) + (obj?.lift ?? 0);
      // (Lain on across, they face along it: moved back to its middle from a slot to one side.)
      rig.bedFwd = (bed.fwd ?? 0) - (rig.lieAcross ? across : 0);
      rig.useDef = obj?.def ?? '';
      rig.useObj = obj ?? null;
      if ((rig.lastPose === Pose.Stand && rig.cur.key !== 'sitDown') || (rig.cur.key === 'sitDown' && rig.cur.time < rig.cur.clip!.duration - 0.35)) {
        // Getting in: sit down on the edge first; the crossfade to lying then leans the Sim back
        // (the root pitch blends with the clips).
        const sp = this.sitPelvis.get(rig.body.name)!;
        key = 'sitDown';
        rootY = rig.bedH - sp.y * rig.scale;
        rootFwd = LIE_ENTRY * rig.scale - sp.z * rig.scale + rig.bedFwd;
        fade = 0.35;
        if (rig.cur.key !== 'sitDown') rig.actTime = 0;
      } else if (rig.body.stage === 'baby') {
        // A baby: on the back in the middle of the crib, curled up asleep or kicking awake (its own
        // clips, made standing like `lie` and turned flat the same way).
        key = !known || tag === 'sleep' || tag === 'nap' ? BABY_LIE.asleep : BABY_LIE.awake;
        rootY = rig.bedH + lieBack(rig);
        rootFwd = lieCentre(rig);
        pitch = -Math.PI / 2;
        fade = LYING.has(rig.cur.key) ? BABY_LIE_FADE : 0.45;
        rig.lieX = rig.x;
        rig.lieZ = rig.z;
      } else {
        // On the back, head towards the head of the bed (local -z), centred on the slot: the
        // `lie` clip (made standing) turned flat and slowed down.
        key = 'lie';
        rootY = rig.bedH + lieBack(rig);
        rootFwd = lieCentre(rig) + rig.bedFwd;
        pitch = -Math.PI / 2;
        rate = 0.3;
        // In a hammock: down in the sling's dip, the head up on the pillow (`inSling`).
        const at = bed.sling ? this.inSling(rig, bed.sling, obj?.lift ?? 0, rootFwd) : null;
        rig.sling = at ? bed.sling! : null;
        if (at) [rootY, rootFwd] = at;
        fade = rig.cur.key === 'sitDown' ? 0.9 : 0.45;
        // (Where it lies, for getting up: by then the Sim has already moved off the bed.)
        rig.lieX = rig.x;
        rig.lieZ = rig.z;
      }
      hands = tag === 'lift' ? HANDS.grip : HANDS.open;
    } else if (pose === Pose.Sit) {
      const obj = known ? this.seatUnder(rig, useObj) : this.objectAt(rig.x, rig.z);
      const seat = obj?.seat ?? DEFAULT_SEAT;
      const sp = this.sitPelvis.get(rig.body.name)!;
      rootY = seat.h - sp.y * rig.scale;
      // (Facing the object's back, its forward is the Sim's back.)
      rootFwd = (seat.back ? -seat.fwd : seat.fwd) - sp.z * rig.scale;
      fade = 0.35;
      rig.useDef = obj?.def ?? '';
      rig.useObj = obj ?? null;
      if (social) key = speaking(pair, role, time) ? 'sitTalk' : 'sit';
      else if (act?.sit) key = typeof act.sit === 'function' ? act.sit(rig.useDef) : act.sit;
      else if (!known && obj && SIT_HANDS.has(obj.def)) key = 'sitHands';
      else key = 'sit';
      if (rig.lastPose === Pose.Stand && rig.cur.key !== 'sitDown') {
        key = 'sitDown';
        rig.actTime = 0;
      } else if (rig.cur.key === 'sitDown' && rig.cur.time < rig.cur.clip!.duration - 0.3) {
        key = 'sitDown';
      }
      rootFwd += this.tuckIn(rig, obj, seat, key !== 'sitDown', step, fresh);
      if (key === 'sitHands' && rig.tuckFar) key = 'sitHandsFar';
      if (!social) {
        const up = known ? act?.upper : obj?.def === 'diningTable' ? 'eat' : undefined;
        // Eating / drinking now and then; holding a book or phone throughout.
        if (up === 'eat' || up === 'drink') upper = Math.floor((rig.actTime + rig.seed * 9) / 4) % 3 === 0 ? up : '';
        else if (up) upper = up;
      }
    } else if (moving) {
      // Walking, the cycle matched to the ground speed; faster than the walk can step (a sped-up
      // game, short legs), jogging along.
      const legs = rig.body.pelvisScale * rig.scale;
      const walk = rig.speed / (set.clips.get('walk')!.speed * legs);
      const jog = set.clips.get('jogAlong');
      const jogging = !!jog && walk > GAIT_MAX_RATE * (rig.cur.key === 'jogAlong' ? JOG_BACK : 1);
      key = jogging ? 'jogAlong' : 'walk';
      rate = clamp(jogging ? rig.speed / (jog!.speed * legs) : walk, 0.35, GAIT_MAX_RATE);
      fade = GAITS.has(rig.cur.key) ? 0.3 : 0.2;
      hands = HANDS.relaxed;
      rig.useObj = null;
    } else if (social) {
      const me = speaking(pair, role, time);
      hands = HANDS.relaxed;
      // Done with the thing they used: its prop (a guitar, a mop) isn't in their hands while they talk.
      rig.useObj = null;
      switch (social) {
        case Anim.Talk:
          key = me
            ? 'talk'
            : outcome === 1 && hash(Math.floor(time / 3) + i) < 0.4
              ? 'yes'
              : outcome === 2 && hash(Math.floor(time / 3) + i) < 0.4
                ? 'no'
                : 'idle';
          break;
        case Anim.Laugh:
          key = 'talk';
          break;
        case Anim.Flirt:
          key = me ? 'talk' : 'idle';
          break;
        case Anim.Argue:
          key = me ? 'talk' : 'no';
          rate = 1.25;
          break;
        case Anim.Fight: {
          // The attacker steps in with each punch, the defender gives ground as it lands
          // (eased like the hug's approach).
          const beat = (time * 1.1) % 1;
          const lunge = Math.sin(beat * Math.PI);
          reach = role === 1 ? 0.04 + 0.1 * lunge : -0.03 - 0.09 * lunge;
          reachRate = 8;
          key = role === 1 ? (Math.floor(time * 1.1) % 2 ? 'punchCross' : 'punchJab') : 'hit';
          repeat = true;
          hands = HANDS.grip;
          break;
        }
        case Anim.Hug:
          // Arms around each other (`procedural`), stepped in until the chests meet.
          key = 'idle';
          reach = embraceStep(rig, partner, 0);
          reachRate = 6;
          break;
        case Anim.Kiss:
          key = 'idle';
          reach = embraceStep(rig, partner, EMBRACE.kiss);
          reachRate = 6;
          break;
        case Anim.Wave:
          // Goodbye: both wave, again and again while it lasts.
          key = 'wave';
          repeat = true;
          break;
        default:
          key = me ? 'talk' : 'idle';
      }
      if (social !== Anim.Talk && social !== Anim.Fight) rig.actTime = 0;
    } else {
      // Standing object use: the action's clip sequence (or, without action data, the object in front).
      const obj = known ? useObj : this.objectInFront(rig);
      rig.useDef = obj?.def ?? '';
      rig.useObj = obj ?? null;
      // Used from inside or up against it (`STEP_IN`): the clip for their height.
      const into = known && obj ? STEP_IN[obj.def] : undefined;
      // (Bouncing and vibrating on the equipment, content `stand`: the idle with `activityPose` on top.)
      const seq = into
        ? this.stepSeq(rig, into)
        : known
        ? obj && HUGGED[obj.def]
          ? CROUCH_ONLY
          : obj?.stand?.motion || obj?.held?.grip === 'pair' || obj?.held?.grip === 'guitar'
          ? IDLE_ONLY
          : obj?.held?.grip === 'pole'
            ? PUSH_ONLY
          : act?.stand
            ? typeof act.stand === 'function'
              ? act.stand(useObj?.def ?? '')
              : act.stand
            : undefined
        : obj
          ? USE[obj.def]
          : undefined;
      const useKey = seq ? `${tag}:${obj?.def ?? ''}` : '';
      if (seq && seq.length) {
        if (rig.useKey !== useKey) {
          rig.useKey = useKey;
          rig.useSeq = seq;
          rig.seq = 0;
          rig.actTime = 0;
          rig.stepTime = 0;
        }
        key = seq[rig.seq % seq.length];
        repeat = true;
        // Up to the crib's rail to pick the baby up, back from it holding them (`cuddles`).
        if (tag === 'cuddle') {
          reach = rig.cuddleStep;
          reachRate = 6;
        } else if (obj && STAND_BACK[obj.def]) {
          // Back from the door it opens (or in to a small thing), eased like the hug's approach.
          reach = -STAND_BACK[obj.def];
        } else if (obj && HUGGED[obj.def]) {
          // Up to the teddy, to hug it.
          reach = HUGGED[obj.def].reach;
        }
        const on = obj?.stand;
        if (on && obj && tag !== 'cuddle') {
          // Up on the equipment (content `stand`; Sims use it from the tile in front): forward to its
          // middle, on what they stand on (a treadmill's belt, facing the console).
          rootFwd = Math.hypot(obj.cx - rig.x, obj.cz - rig.z);
          rootY = on.h;
          if (on.motion === 'bounce') {
            // Bouncing on the mat: the feet leave it for a little under half of each bounce.
            const b = Math.sin(((rig.actTime * 1.6) % 1) * Math.PI);
            rootY += Math.max(0, b * 0.75 - 0.3) * 0.55;
          } else if (on.motion === 'buzz') {
            // On the vibration plate: a fine buzz.
            rootY += Math.sin(rig.actTime * 90) * 0.004;
          }
        }
        if (into && obj) {
          [rootFwd, rootY, key, rate] = this.stepIn(rig, obj, into, key, step, dt);
          inside = true;
        }
      } else {
        rig.useKey = '';
        rig.useSeq = null;
        // Idle with occasional variations.
        const phase = Math.floor((rig.actTime + rig.seed * 20) / 9);
        const r = hash(phase * 7 + i);
        key = rig.actTime < 6 ? 'idle' : r < 0.18 ? 'foldArms' : 'idle';
        if (!tag || tag === 'idle') hands = HANDS.rest;
      }
    }
    if (!inside && rig.inside) this.stepOut(rig, fresh);
    rig.approach += (reach - rig.approach) * Math.min(1, step * reachRate);
    const dir = rig.dir;
    if (dir?.clip) {
      key = dir.clip;
      repeat = false;
      fade = 0.3;
      rate = 1;
      upper = '';
    }
    if (dir?.hands !== undefined) hands = dir.hands;
    if (!moving && pose === Pose.Stand && (rig.lastPose === Pose.Sit || rig.lastPose === Pose.Lie) && rig.cur.key !== 'standUp' && !this.force) {
      // Rising from a seat (or sitting up out of bed: the crossfade from lying leans the Sim up).
      key = 'standUp';
      rig.fromBed = rig.lastPose === Pose.Lie && !!rig.bed;
      if (rig.fromBed) fade = 0.9;
    } else if (rig.cur.key === 'standUp' && !moving && pose === Pose.Stand && rig.cur.time < rig.cur.clip!.duration - 0.25) {
      key = 'standUp';
    }
    if (key !== 'standUp') rig.fromBed = false;
    if (pose !== Pose.Sit) rootFwd += this.tuckOut(rig, key, step);
    // Carrying something while walking or standing about (not while using a station with it).
    if (carry && pose === Pose.Stand && !upper && !dir?.clip && key !== 'standUp' && (moving || !rig.useSeq) && social !== Anim.Hug && social !== Anim.Fight) {
      upper = carry;
      hands = HANDS.hold;
      if (key === 'foldArms') key = 'idle';
    }
    const fx = Math.sin(rig.yaw);
    const fz = Math.cos(rig.yaw);
    if (rig.fromBed && rig.bed) {
      // Sitting on the foot of the bed, then stepping forward onto the Sim's tile while rising.
      const bed = rig.bed;
      const half = (Math.abs(fx) * (bed.maxX - bed.minX) + Math.abs(fz) * (bed.maxZ - bed.minZ)) / 2;
      const edge = (bed.cx - rig.x) * fx + (bed.cz - rig.z) * fz + half - 0.22 * rig.scale;
      const t = rig.cur.key === 'standUp' ? rig.cur.time / rig.cur.clip!.duration : 0;
      rootFwd = edge * (1 - smooth(clamp((t - 0.3) / 0.6, 0, 1)));
    }
    // (A little quicker at slow game speeds than the animation; still while paused.)
    rig.hands += (hands - rig.hands) * (1 - Math.exp(-(step * 5 + dt * 1.2)));

    // Sequenced object use: the next clip when a one-shot ends, or after LOOP_STEP for loops.
    const cur = rig.cur;
    const seq = rig.useSeq;
    const done = cur.clip && (cur.clip.loop ? rig.stepTime >= Math.max(LOOP_STEP, cur.clip.duration) : cur.time >= cur.clip.duration - 0.2);
    if (cur.key === key && repeat && seq && done) {
      rig.seq++;
      rig.stepTime = 0;
      key = seq[rig.seq % seq.length];
      // The same one-shot again restarts it (crossfaded); a loop simply continues.
      if (key !== cur.key || !cur.clip!.loop) this.start(rig, key, 0.3, rate, repeat, rootY, rootFwd, pitch);
    } else if (cur.key === key && repeat && done && !cur.clip!.loop) {
      // Repeated one-shots (fights).
      this.start(rig, key, 0.3, rate, repeat, rootY, rootFwd, pitch);
    } else if (cur.key !== key) {
      rig.stepTime = 0;
      this.start(rig, key, fade, rate, repeat, rootY, rootFwd, pitch);
    } else {
      cur.rate = rate;
      cur.rootY = rootY;
      cur.rootFwd = rootFwd;
      cur.rootPitch = pitch;
    }

    if (rig.fromBed && LYING.has(rig.prev.key)) rig.prev.rootFwd = (rig.lieX - rig.x) * fx + (rig.lieZ - rig.z) * fz + lieCentre(rig) + rig.bedFwd;

    // Upper-body layer.
    if (upper) {
      if (rig.upper.key !== upper) {
        rig.upper.key = upper;
        rig.upper.clip = this.clip(upper);
        rig.upper.time = 0;
        rig.upper.rate = 1;
      }
      rig.upperTarget = 1;
    } else {
      rig.upperTarget = 0;
    }

    if (fresh) {
      copyPlayer(rig.cur, rig.prev);
      rig.w = 1;
      rig.upperW = rig.upperTarget;
      rig.approach = reach;
      rig.hands = hands;
    }
  }

  /** The size of `STEP_IN` entry `into` for this resident's height: [height, clips, where]. */
  private stepFit(rig: SimRig, into: StepIn): StepIn['by'][number] {
    const h = rig.body.height * rig.scale;
    return into.by.find(([min]) => h >= min) ?? into.by[into.by.length - 1];
  }

  /** The first of `into`'s lanes no one else using the same thing has taken (its middle without lanes). */
  private freeLane(rig: SimRig, into: StepIn): number {
    const taken = this.sims.filter((r) => r && r !== rig && r.inside === into && r.useId === rig.useId).map((r) => r!.inLane);
    return into.lanes?.find((x) => !taken.includes(x)) ?? into.lanes?.[0] ?? 0;
  }

  /** The clips played using `into` (`STEP_IN`): its sequence for the resident's height, or (`perLane`) the one for their lane. */
  private stepSeq(rig: SimRig, into: StepIn): readonly string[] {
    const seq = this.stepFit(rig, into)[1];
    if (!into.perLane || !into.lanes) return seq;
    const lane = Math.max(0, into.lanes.indexOf(rig.inside === into ? rig.inLane : this.freeLane(rig, into)));
    const clip = seq[lane % seq.length];
    let one = LANE_SEQ.get(clip);
    if (!one) LANE_SEQ.set(clip, (one = [clip]));
    return one;
  }

  /**
   * Using a thing from inside or up against it (`STEP_IN`) from the tile in front: back a little while its
   * door opens (`back`, the first `wait` s), then walking in to where they use it, up onto its floor on the
   * way. The root's forward offset and height, the clip (standing about while waiting, walking, or `key`)
   * and its rate.
   */
  private stepIn(rig: SimRig, obj: PlacedObject, into: StepIn, key: string, step: number, dt: number): [number, number, string, number] {
    if (rig.inside !== into) {
      rig.inside = into;
      rig.inT = 0;
      rig.outT = -1;
      rig.inLane = this.freeLane(rig, into);
    } else rig.inT += step;
    const at = this.stepFit(rig, into)[2];
    const f = (obj.rot * Math.PI) / 2;
    // The tile's distance from the object's middle, along its facing; the way in from stood back.
    const ahead = (rig.x - obj.cx) * Math.sin(f) + (rig.z - obj.cz) * Math.cos(f);
    const back = into.back ?? 0;
    const dist = ahead - at.to + back;
    // And sideways to its lane (along the object's right).
    const across = into.lanes ? rig.inLane - ((rig.x - obj.cx) * Math.cos(f) - (rig.z - obj.cz) * Math.sin(f)) : 0;
    const way = Math.hypot(dist, across);
    const span = Math.max(0.3, way / STEP_IN_SPEED);
    const k = smooth(clamp((rig.inT - (into.wait ?? 0)) / span, 0, 1));
    const fwd = -back * smooth(clamp(rig.inT / 0.4, 0, 1)) + dist * k;
    const y = (at.up ?? 0) * smooth(clamp((k - 0.35) / 0.3, 0, 1));
    rig.inDX = Math.cos(f) * across * k;
    rig.inDZ = -Math.sin(f) * across * k;
    // (Waiting to step in, standing about.)
    if (k <= 0) return [fwd, y, into.wait ? 'idle' : key, 1];
    if (k >= 1) return [fwd, y, key, 1];
    // Walking, the cycle matched to the step's pace (the walk runs on real time).
    const legs = rig.body.pelvisScale * rig.scale;
    const rate = dt > 0 ? ((way / span) * (step / dt)) / (this.clip('walk').speed * legs) : 1;
    return [fwd, y, 'walk', clamp(rate, 0.35, GAIT_MAX_RATE)];
  }

  /** The use of a thing used from inside (`STEP_IN`) is over: back out to the tile in front after its `out` seconds (`stepPlace`); none when drawn afresh. */
  private stepOut(rig: SimRig, fresh: boolean): void {
    rig.outT = fresh ? -1 : 0;
    rig.outWait = rig.inside?.out ?? 0;
    rig.inside = null;
  }

  /** Where a resident using a thing from inside is drawn (kept for stepping out), and stepping out: from there to where they are now (`STEP_OUT_S`). */
  private stepPlace(rig: SimRig, pl: Placement, step: number): void {
    if (rig.inside) {
      pl.x += rig.inDX;
      pl.z += rig.inDZ;
      [rig.inX, rig.inY, rig.inZ] = [pl.x, pl.y, pl.z];
      return;
    }
    if (rig.outT < 0) return;
    const k = smooth(clamp((rig.outT - rig.outWait) / STEP_OUT_S, 0, 1));
    pl.x = lerp(rig.inX, pl.x, k);
    pl.y = lerp(rig.inY, pl.y, k);
    pl.z = lerp(rig.inZ, pl.z, k);
    rig.outT = k < 1 ? rig.outT + step : -1;
  }

  private clip(key: string): Clip {
    const set = this.set!;
    return set.clips.get(key) ?? set.clips.get('idle')!;
  }

  /** Length (s) of the clip played for `key` (0 before the set has loaded). */
  clipDuration(key: string): number {
    return this.set ? this.clip(key).duration : 0;
  }

  /** Moves this frame's placement (`place`) by `ride`, a seat's move from rest in world space. */
  private riding(ride: Matrix): void {
    const pl = this.place;
    this.rideQ.set(pl.qx, pl.qy, pl.qz, pl.qw);
    Matrix.ComposeToRef(this.rideS.setAll(1), this.rideQ, this.rideT.set(pl.x, pl.y, pl.z), this.rideM);
    this.rideM.multiplyToRef(ride, this.rideM);
    this.rideM.decompose(this.rideS, this.rideQ, this.rideT);
    [pl.x, pl.y, pl.z] = [this.rideT.x, this.rideT.y, this.rideT.z];
    [pl.qx, pl.qy, pl.qz, pl.qw] = [this.rideQ.x, this.rideQ.y, this.rideQ.z, this.rideQ.w];
  }

  /** The tables seats face, with what is in front of them (`tableRooms`), by seat id: set with each world. */
  setTables(tables: Map<number, TableRoom>): void {
    this.tables = tables;
    this.seatIds.clear();
    for (const [id] of tables) {
      const o = this.objectById.get(id);
      if (o) this.seatIds.set(o, id);
    }
  }

  /** How far each chair pulled up to a table is drawn pulled in now (m, along its facing), by seat id (`TUCK`). */
  readonly pulled = new Map<number, number>();

  /**
   * Seated on a chair that faces a table (`tables`): how far it is pulled in now (m), easing in once they
   * are `settled` (sat down), at once when drawn afresh (`TUCK`).
   */
  private tuckIn(rig: SimRig, obj: PlacedObject | null, seat: Seat, settled: boolean, step: number, fresh: boolean): number {
    const id = obj ? this.seatIds.get(obj) : undefined;
    const room = id === undefined ? undefined : this.tables.get(id);
    if (id !== rig.tuckSeat) rig.tuck = 0;
    rig.tuckSeat = id ?? -1;
    let goal = 0;
    rig.tuckFar = false;
    if (obj && room && settled && !seat.back && room.top > seat.h && room.top - seat.h <= TUCK.rise) {
      const s = (rig.body.height * rig.scale) / CLIP_HEIGHT;
      const yaw = (obj.rot * Math.PI) / 2;
      const [dx, dz] = [rig.x - obj.cx, rig.z - obj.cz];
      const right = dx * Math.cos(yaw) - dz * Math.sin(yaw);
      const hips = dx * Math.sin(yaw) + dz * Math.cos(yaw) + seat.fwd;
      const edge = nearestAhead(room, right, 0.12 * s, room.top - 0.04, room.top + 0.01);
      // (The knees no further than the middle of the top: someone may sit across it.)
      const middle = (edge + furthestAhead(room, right, 0.12 * s, room.top - 0.04, room.top + 0.01)) / 2;
      const knees = nearestAhead(room, right, 0.16 * s, TUCK.toeUp, seat.h + TUCK.kneeUp * s);
      const toes = nearestAhead(room, right, 0.16 * s, 0, TUCK.toeUp);
      const most = Math.min(edge - TUCK.edge * s, Math.min(knees, middle) - TUCK.knees * s - TUCK.clear, toes - TUCK.toes * s - TUCK.clear) - hips;
      if (Number.isFinite(edge)) goal = clamp(most, 0, TUCK.max);
      rig.tuckFar = Number.isFinite(edge) && edge - hips - goal > TUCK.far * s;
    }
    rig.tuck = fresh ? goal : rig.tuck + (goal - rig.tuck) * Math.min(1, step * TUCK.rate);
    return rig.tuck;
  }

  /** Off the chair pulled up to a table: it is pushed back as they rise (`TUCK`); how far in it still is while they do. */
  private tuckOut(rig: SimRig, key: string, step: number): number {
    if (rig.tuckSeat < 0) return 0;
    rig.tuck *= 1 - Math.min(1, step * TUCK.back);
    if (rig.tuck < 0.002) [rig.tuck, rig.tuckSeat] = [0, -1];
    return key === 'standUp' ? rig.tuck : 0;
  }

  /** The chairs pulled in now (`pulled`): each as far as the least pulled-in of those on it. */
  private pullChairs(): void {
    this.pulled.clear();
    for (const rig of this.sims) {
      if (!rig?.visible || rig.tuckSeat < 0 || rig.tuck <= 0.001) continue;
      const was = this.pulled.get(rig.tuckSeat);
      this.pulled.set(rig.tuckSeat, was === undefined ? rig.tuck : Math.min(was, rig.tuck));
    }
  }

  /** Where residents sitting or lying on an object are carried by its moving seat (`ObjectMotion.ride`). */
  setRide(rideOf: ((objectId: number) => Matrix | null) | null): void {
    this.rideOf = rideOf;
  }

  /** How far round an object's loop is (`ObjectMotion.loopPhase`: the bike's pedals the feet follow). */
  setLoop(loopOf: ((objectId: number) => number | null) | null): void {
    this.loopOf = loopOf;
  }

  private start(rig: SimRig, key: string, fade: number, rate: number, repeat: boolean, rootY: number, rootFwd: number, pitch: number): void {
    const clip = this.clip(key);
    // The fading-out player continues from the current blend.
    if (rig.w < 0.5 && rig.prev.clip) {
      // Mostly still the old one: keep it, replace the incoming.
    } else {
      copyPlayer(rig.cur, rig.prev);
    }
    const c = rig.cur;
    const sameLoop = c.key === key && clip.loop;
    // Walking into a jog and back: in step (both cycles start at the left heel's strike).
    const inStep = GAITS.has(key) && GAITS.has(c.key) && c.clip ? ((c.time / c.clip.duration) % 1) * clip.duration : -1;
    c.clip = clip;
    c.key = key;
    // Walk cycles of the previous walk keep their phase.
    c.time = sameLoop ? c.time : inStep >= 0 ? inStep : key === 'idle' || key === 'sit' || key === BABY_LIE.asleep || key === BABY_LIE.awake ? (rig.seed * 7) % clip.duration : 0;
    c.rate = rate;
    c.repeat = repeat;
    c.rootY = rootY;
    c.rootFwd = rootFwd;
    c.rootPitch = pitch;
    if (!rig.prev.clip) copyPlayer(c, rig.prev);
    rig.w = 0;
    rig.fadeRate = 1 / Math.max(0.05, fade);
  }

  /** Look-at, laughing, flirting and kissing on top of the clips. */
  private procedural(
    rig: SimRig,
    i: number,
    social: number,
    role: number,
    partner: SimRig | null,
    pose: number,
    moving: boolean,
    time: number,
    dt: number,
  ): void {
    const set = this.set!;
    const B = set.bone;
    const add = this.add;
    // Look at the conversation partner (head and neck turn, limited).
    let wantYaw = 0;
    let wantPitch = 0;
    const dir = rig.dir;
    if (dir?.lookAt) {
      // Directed: the head turns partway towards the point (the eyes do the rest).
      const dx = dir.lookAt.x - this.heads[i * 3];
      const dz = dir.lookAt.z - this.heads[i * 3 + 2];
      const dy = dir.lookAt.y - this.heads[i * 3 + 1];
      wantYaw = clamp(wrapAngle(Math.atan2(dx, dz) - rig.yaw), -1.1, 1.1) * 0.45;
      wantPitch = clamp(-Math.atan2(dy, Math.hypot(dx, dz)), -0.4, 0.4) * 0.45;
    } else if (dir?.still) {
      // Portraits: straight ahead.
    } else if (social && partner && partner.visible && pose !== Pose.Lie) {
      const ph = (rig.audience?.visible ? rig.audience : partner).id * 3;
      const dx = this.heads[ph] - this.heads[i * 3];
      const dz = this.heads[ph + 2] - this.heads[i * 3 + 2];
      const dy = this.heads[ph + 1] - this.heads[i * 3 + 1];
      wantYaw = clamp(wrapAngle(Math.atan2(dx, dz) - rig.yaw), -1.1, 1.1);
      wantPitch = clamp(-Math.atan2(dy, Math.hypot(dx, dz)), -0.4, 0.4);
    } else if (!moving && pose !== Pose.Lie && !this.force && rig.cuddleT < 0) {
      // Idle glances.
      const g = Math.floor(time / 3.7 + rig.seed * 5);
      wantYaw = (hash(g * 13 + i) - 0.5) * 0.9 * (hash(g * 5 + i) < 0.5 ? 1 : 0);
    }
    const f = dir?.still ? 1 : 1 - Math.exp(-dt * 5);
    rig.lookYaw += (wantYaw - rig.lookYaw) * f;
    rig.lookPitch += (wantPitch - rig.lookPitch) * f;
    if (Math.abs(rig.lookYaw) > 1e-3) {
      addRotation(add, B.neck_01, 0, 1, 0, rig.lookYaw * 0.4);
      addRotation(add, B.Head, 0, 1, 0, rig.lookYaw * 0.6);
    }
    if (Math.abs(rig.lookPitch) > 1e-3) addRotation(add, B.Head, 1, 0, 0, rig.lookPitch * 0.7);

    if (pose !== Pose.Lie && this.force !== 'rest') {
      // Elders stoop: the upper back rounds, the head lifts to look ahead.
      if (rig.stoop > 0) {
        addRotation(add, B.spine_03, 1, 0, 0, 0.2 * rig.stoop);
        addRotation(add, B.neck_01, 1, 0, 0, 0.08 * rig.stoop);
        addRotation(add, B.Head, 1, 0, 0, -0.12 * rig.stoop);
      }
      this.stance(rig, pose, moving, dt);
      // Walking: women's hips sway more (the pelvis rolls over each standing leg, the chest
      // counters it), as in real gaits; one roll each way per stride.
      const walkW = (rig.cur.key === 'walk' ? smooth(rig.w) : 0) * (GAIT_SWAY[rig.body.family] ?? 0);
      if (walkW > 0 && rig.cur.clip) {
        const roll = Math.sin((rig.cur.time / rig.cur.clip.duration) * Math.PI * 2) * walkW;
        addRotation(add, B.pelvis, 0, 0, 1, roll);
        addRotation(add, B.spine_02, 0, 0, 1, -roll * 0.7);
        addRotation(add, B.spine_03, 0, 0, 1, -roll * 0.4);
      }
      if (!dir?.still) {
        // Breathing (a slow lift of the chest, the head kept level; quicker when worked up) and a
        // faint sway of the head: nobody stands perfectly still.
        const rate = social === Anim.Argue || social === Anim.Fight || moving ? 2.2 : 1.55;
        const breath = Math.sin(time * rate + rig.seed * 6);
        addRotation(add, B.spine_02, 1, 0, 0, -0.014 * breath);
        addRotation(add, B.spine_03, 1, 0, 0, -0.008 * breath);
        addRotation(add, B.Head, 1, 0, 0, 0.012 * breath + 0.012 * Math.sin(time * 0.53 + rig.seed * 11));
        addRotation(add, B.Head, 0, 1, 0, 0.018 * Math.sin(time * 0.37 + rig.seed * 17));
      }
    }
    switch (social) {
      case Anim.Laugh: {
        // Natural laughter: body bounces, head tilts, shoulders shake.
        const bounce = Math.sin(time * 8 + rig.seed * 5) * 0.04;
        addRotation(add, B.spine_02, 1, 0, 0, -0.15 + bounce);
        addRotation(add, B.spine_01, 1, 0, 0, -0.08 + bounce * 0.5);
        addRotation(add, B.Head, 1, 0, 0, -0.2 + bounce * 0.7);
        // Shoulder shake.
        addRotation(add, B.upperarm_l, 1, 0, 0, Math.sin(time * 10 + rig.seed * 3) * 0.05);
        addRotation(add, B.upperarm_r, 1, 0, 0, -Math.sin(time * 10 + rig.seed * 3) * 0.05);
        break;
      }
      case Anim.Flirt:
        addRotation(add, B.Head, 0, 0, 1, 0.16 + Math.sin(time * 1.7) * 0.04);
        addRotation(add, B.pelvis, 0, 0, 1, Math.sin(time * 1.3) * 0.04);
        break;
      case Anim.Kiss:
        addRotation(add, B.spine_02, 1, 0, 0, 0.12);
        addRotation(add, B.Head, 0, 0, 1, role === 1 ? 0.22 : -0.22);
        addRotation(add, B.Head, 1, 0, 0, 0.1);
        break;
      case Anim.Hug: {
        // Bent down over a smaller one (a child), the arms lowered to go round them (`EMBRACE`).
        const drop = shoulderDrop(rig, partner);
        addRotation(add, B.spine_02, 1, 0, 0, 0.1 + drop * EMBRACE.bend);
        // Both heads turned and leant to their own right, so they go past each other (cheek by cheek), not into each other.
        addRotation(add, B.spine_03, 0, 0, 1, -EMBRACE.aside);
        addRotation(add, B.Head, 0, 1, 0, 0.5);
        // Arms around the other: one Sim over the shoulders, the other a little lower, around the back.
        this.hugArms(Math.max(EMBRACE.low, (role === 1 ? 1.3 : 1.05) - drop * EMBRACE.lower), time + role / 1.5, EMBRACE.wrap + drop * EMBRACE.wrapSmall);
        break;
      }
      case Anim.Argue:
        addRotation(add, B.spine_03, 1, 0, 0, 0.08);
        break;
      case Anim.Fight: {
        const isAttacker = role === 1;
        if (isAttacker) {
          // Attacker: body leans forward into punches.
          addRotation(add, B.spine_02, 1, 0, 0, 0.15);
          addRotation(add, B.spine_01, 1, 0, 0, 0.1);
        } else {
          // Defender: flinch back, forearms up to guard the face.
          addRotation(add, B.spine_02, 1, 0, 0, -0.12);
          addRotation(add, B.upperarm_l, 1, 0, 0, -0.45);
          addRotation(add, B.upperarm_r, 1, 0, 0, -0.45);
          addRotation(add, B.lowerarm_l, 1, 0, 0, -1.5);
          addRotation(add, B.lowerarm_r, 1, 0, 0, -1.5);
        }
        break;
      }
    }
    if (!social && !this.force && !dir?.clip) this.activityPose(rig, pose, time);
    if ((rig.reachW > 0 || rig.holdW > 0) && pose === Pose.Stand && !this.force) this.cuddlePose(rig, time);
    // Lying: a bench press, or resting (see the `lie` clip).
    if (pose === Pose.Lie && rig.cur.key === 'lie' && rig.tag === 'lift') {
      // Bench press: arms push the bar up and lower it to the chest.
      const press = 0.5 - 0.5 * Math.cos(time * 2.2 + rig.seed * 6);
      for (const [ua, la, sgn] of [
        [B.upperarm_l, B.lowerarm_l, 1],
        [B.upperarm_r, B.lowerarm_r, -1],
      ] as const) {
        addRotation(add, ua, 0, 0, 1, sgn * 0.55);
        addRotation(add, ua, 1, 0, 0, -1.2 - press * 0.1);
        addRotation(add, la, 1, 0, 0, -1.4 + press * 1.3);
      }
    } else if (pose === Pose.Lie && rig.cur.key === 'lie') {
      // (The clip breathes, with the hands on the belly.) The head turns slowly to one side, each
      // Sim its own way (asleep: deeper, less movement).
      const asleep = rig.tag === 'sleep' || rig.tag === 'nap';
      const headTurn = (rig.seed - 0.5) * 0.7 * (asleep ? 1 : 1.5) * (rig.sling ? 0.5 : 1);
      addRotation(add, B.Head, 0, 1, 0, headTurn + Math.sin(time * 0.2 + rig.seed * 8) * 0.02);
      if (rig.sling) this.slingPose(rig, rig.sling);
      // Subtle leg adjustment for more natural lying.
      else if (!asleep) {
        const legShift = Math.sin(time * 0.25 + rig.seed * 7) * 0.03;
        addRotation(add, B.thigh_l, 1, 0, 0, legShift);
        addRotation(add, B.thigh_r, 1, 0, 0, -legShift);
      }
    }
    dir?.pose?.(add, B, time);
    this.animateFace(rig, i, social, role, partner, pose, moving, time, dt);
  }

  /**
   * Arms round someone or something hugged: raised forward `raise` (rad), elbows wrapping in by `wrap` (rad;
   * the teddy's neck, the default, is held looser than a body is, `EMBRACE`); a slow squeeze.
   * (Model space: +z forward, the left arm on -x; x rotations raise / bend both sides alike. Spread first,
   * then raise: the other order would only roll the raised arm.)
   */
  private hugArms(raise: number, time: number, wrap = 0.95): void {
    const B = this.set!.bone;
    const add = this.add;
    const squeeze = Math.sin(time * 1.5) * 0.05;
    addRotation(add, B.upperarm_l, 0, 0, 1, -0.35);
    addRotation(add, B.upperarm_r, 0, 0, 1, 0.35);
    addRotation(add, B.upperarm_l, 1, 0, 0, -raise);
    addRotation(add, B.upperarm_r, 1, 0, 0, -raise);
    addRotation(add, B.lowerarm_l, 0, 1, 0, wrap + squeeze);
    addRotation(add, B.lowerarm_r, 0, 1, 0, -wrap - squeeze);
  }

  /**
   * Procedural layers for activities the clips only approximate: reclining, instruments,
   * meditation, gazes at plants and trees, and exercise equipment. Model space: +z is forward,
   * the left limbs are on -x; a negative x rotation raises an arm forward (or bends an elbow /
   * knee forward), on both sides alike.
   */
  private activityPose(rig: SimRig, pose: number, time: number): void {
    const B = this.set!.bone;
    const add = this.add;
    const def = rig.useDef;
    const key = rig.cur.key;
    const seed = rig.seed * 6;
    if (pose === Pose.Sit && key !== 'sitDown') {
      const seat = rig.useObj?.seat;
      if (seat?.pedals) this.pedal(rig, seat.pedals);
      else if (seat?.legsOut) this.legsOut(rig);
      switch (rig.tag) {
        case 'sit':
          // Just sitting: upright, but against a slanted back (`recline`).
          if (seat?.recline !== undefined) this.recline(seat.recline);
          break;
        case 'relax':
        case 'nap':
        case 'watch':
          // Sinking back into the seat (or against its slanted back, `recline`, split over the spine as `legsOut`
          // does, the head kept looking ahead), legs stretched out a little (or all the way, `legsOut`), arms on
          // the armrests.
          if (seat?.recline !== undefined) this.recline(seat.recline);
          else {
            addRotation(add, B.spine_01, 1, 0, 0, -0.14);
            addRotation(add, B.Head, 1, 0, 0, 0.1);
          }
          if (!seat?.legsOut) {
            addRotation(add, B.calf_l, 1, 0, 0, -0.35);
            addRotation(add, B.calf_r, 1, 0, 0, -0.3);
          }
          addRotation(add, B.upperarm_l, 0, 0, 1, -0.18);
          addRotation(add, B.upperarm_r, 0, 0, 1, 0.18);
          break;
        case 'bath': {
          // Leaning back against the end of the tub, arms along the rim.
          addRotation(add, B.spine_01, 1, 0, 0, -0.3);
          addRotation(add, B.Head, 1, 0, 0, 0.2);
          addRotation(add, B.upperarm_l, 0, 0, 1, -0.95);
          addRotation(add, B.upperarm_r, 0, 0, 1, 0.95);
          addRotation(add, B.lowerarm_l, 1, 0, 0, -0.15);
          addRotation(add, B.lowerarm_r, 1, 0, 0, -0.15);
          break;
        }
        case 'toilet':
          // Leaning forward, the hands sliding down the thighs (the clip rests them there).
          addRotation(add, B.spine_01, 1, 0, 0, 0.3);
          addRotation(add, B.Head, 1, 0, 0, -0.15);
          addRotation(add, B.upperarm_l, 1, 0, 0, -0.15);
          addRotation(add, B.upperarm_r, 1, 0, 0, -0.15);
          addRotation(add, B.lowerarm_l, 1, 0, 0, 0.1);
          addRotation(add, B.lowerarm_r, 1, 0, 0, 0.1);
          break;
        case 'meditate': {
          // Cross-legged and upright, hands resting on the knees, slow deep breaths.
          const breath = Math.sin(time * 0.8 + seed) * 0.025;
          addRotation(add, B.spine_01, 1, 0, 0, 0.18 + breath);
          addRotation(add, B.spine_02, 1, 0, 0, -breath);
          addRotation(add, B.Head, 1, 0, 0, 0.06);
          addRotation(add, B.thigh_l, 0, 1, 0, -0.75);
          addRotation(add, B.thigh_r, 0, 1, 0, 0.75);
          addRotation(add, B.calf_l, 0, 0, 1, 1.25);
          addRotation(add, B.calf_r, 0, 0, 1, -1.25);
          addRotation(add, B.upperarm_l, 0, 0, 1, -0.4);
          addRotation(add, B.upperarm_r, 0, 0, 1, 0.4);
          addRotation(add, B.upperarm_l, 1, 0, 0, -0.3);
          addRotation(add, B.upperarm_r, 1, 0, 0, -0.3);
          addRotation(add, B.lowerarm_l, 1, 0, 0, -0.1);
          addRotation(add, B.lowerarm_r, 1, 0, 0, -0.1);
          break;
        }
        case 'read':
          addRotation(add, B.Head, 1, 0, 0, 0.18);
          break;
        case 'music':
          if (def.includes('cello')) {
            // Knees apart around the body of the cello, the left hand up on its neck, the right
            // drawing the bow back and forth.
            const bow = Math.sin(time * 2.2 + seed);
            addRotation(add, B.thigh_l, 0, 1, 0, -0.35);
            addRotation(add, B.thigh_r, 0, 1, 0, 0.35);
            addRotation(add, B.upperarm_l, 1, 0, 0, -0.95);
            addRotation(add, B.upperarm_l, 0, 0, 1, -0.25);
            addRotation(add, B.lowerarm_l, 1, 0, 0, -1.3);
            addRotation(add, B.lowerarm_l, 0, 1, 0, 0.6);
            addRotation(add, B.upperarm_r, 1, 0, 0, -0.55);
            addRotation(add, B.upperarm_r, 0, 0, 1, 0.45 + bow * 0.12);
            addRotation(add, B.lowerarm_r, 1, 0, 0, -0.9);
            addRotation(add, B.lowerarm_r, 0, 1, 0, -0.7 - bow * 0.45);
            addRotation(add, B.Head, 0, 0, 1, 0.12);
          } else if (def.includes('drum')) {
            // Sticks: the forearms strike in turn.
            const l = Math.max(0, Math.sin(time * 9 + seed));
            const r = Math.max(0, Math.sin(time * 9 + seed + Math.PI));
            addRotation(add, B.lowerarm_l, 1, 0, 0, -0.55 * l);
            addRotation(add, B.lowerarm_r, 1, 0, 0, -0.55 * r);
            addRotation(add, B.Head, 1, 0, 0, Math.sin(time * 4.5) * 0.05);
          } else {
            // Keys: swaying with the music.
            addRotation(add, B.spine_02, 0, 0, 1, Math.sin(time * 1.1 + seed) * 0.04);
          }
          break;
      }
      return;
    }
    if (pose !== Pose.Stand) return;
    const grip = rig.useObj?.held?.grip;
    switch (rig.tag) {
      case 'lift':
        if (grip === 'pair') {
          // Curls: the upper arms at the sides, a dumbbell in each fist coming up in turn.
          const l = 0.5 - 0.5 * Math.cos(time * 2.4 + seed);
          const r = 0.5 - 0.5 * Math.cos(time * 2.4 + seed + Math.PI);
          addRotation(add, B.upperarm_l, 0, 0, 1, 0.12);
          addRotation(add, B.upperarm_r, 0, 0, 1, -0.12);
          addRotation(add, B.upperarm_l, 1, 0, 0, -0.1 - 0.15 * l);
          addRotation(add, B.upperarm_r, 1, 0, 0, -0.1 - 0.15 * r);
          addRotation(add, B.lowerarm_l, 1, 0, 0, -0.35 - 1.6 * l);
          addRotation(add, B.lowerarm_r, 1, 0, 0, -0.35 - 1.6 * r);
          addRotation(add, B.Head, 1, 0, 0, 0.12);
        }
        break;
      case 'music':
        if (grip === 'guitar') {
          // A guitar across the body (`writeHeld`): the left arm out to its neck, the right forearm over
          // its body, strumming; the head bent to the strings, nodding with the music.
          const { long: a, short: b } = GUITAR_ARMS;
          const k = this.guitarReach(rig);
          const strum = Math.sin(time * 7 + seed);
          addRotation(add, B.upperarm_l, 0, 0, 1, lerp(b.leftOut, a.leftOut, k));
          addRotation(add, B.upperarm_l, 1, 0, 0, lerp(b.leftUp, a.leftUp, k));
          addRotation(add, B.lowerarm_l, 1, 0, 0, lerp(b.leftBend, a.leftBend, k));
          addRotation(add, B.lowerarm_l, 0, 1, 0, lerp(b.leftIn, a.leftIn, k));
          addRotation(add, B.upperarm_r, 0, 0, 1, lerp(b.rightOut, a.rightOut, k));
          addRotation(add, B.upperarm_r, 1, 0, 0, lerp(b.rightUp, a.rightUp, k) + 0.06 * strum);
          addRotation(add, B.lowerarm_r, 1, 0, 0, lerp(b.rightBend, a.rightBend, k));
          addRotation(add, B.lowerarm_r, 0, 1, 0, lerp(b.rightIn, a.rightIn, k) + 0.12 * strum);
          addRotation(add, B.Head, 1, 0, 0, 0.25 + Math.sin(time * 3.5 + seed) * 0.04);
          addRotation(add, B.Head, 0, 1, 0, 0.15);
        }
        break;
      case 'relax': {
        const hugged = HUGGED[def];
        if (!hugged) break;
        // Hugging the teddy, crouched: the arms round its neck, the cheek on its head.
        const shoulder = rig.body.height * rig.scale * CROUCH_SHOULDER;
        addRotation(add, B.Head, 0, 1, 0, 0.5);
        this.hugArms(0.35 + 0.45 * clamp((hugged.at + 0.35 - shoulder) / 0.3, 0, 1), time);
        break;
      }
      case 'smell':
        // Nose down to the flowers.
        addRotation(add, B.spine_03, 1, 0, 0, 0.12);
        addRotation(add, B.Head, 1, 0, 0, 0.3);
        break;
      case 'admire':
        // Looking up into the crown.
        addRotation(add, B.spine_03, 1, 0, 0, -0.08);
        addRotation(add, B.neck_01, 1, 0, 0, -0.25);
        addRotation(add, B.Head, 1, 0, 0, -0.4 + Math.sin(time * 0.4 + seed) * 0.05);
        break;
      case 'talk':
      case 'look':
      case 'listen':
        // Talking to a plant, looking at a bird bath: eyes down to the object.
        if (def && def !== 'telescope' && key !== 'rail') addRotation(add, B.Head, 1, 0, 0, 0.2);
        break;
      case 'exercise':
        if (def.includes('boulder')) {
          // Climbing in place: the hands reach for holds overhead in turn, a knee comes up.
          const c = time * 1.4 + seed;
          const l = 0.5 + 0.5 * Math.sin(c);
          const r = 1 - l;
          addRotation(add, B.upperarm_l, 1, 0, 0, -2.2 - 0.5 * l);
          addRotation(add, B.upperarm_r, 1, 0, 0, -2.2 - 0.5 * r);
          addRotation(add, B.lowerarm_l, 1, 0, 0, -0.9 * r);
          addRotation(add, B.lowerarm_r, 1, 0, 0, -0.9 * l);
          addRotation(add, B.thigh_l, 1, 0, 0, -0.7 * r);
          addRotation(add, B.calf_l, 1, 0, 0, 1.0 * r);
          addRotation(add, B.thigh_r, 1, 0, 0, -0.7 * l);
          addRotation(add, B.calf_r, 1, 0, 0, 1.0 * l);
        } else if (rig.useObj?.stand?.motion === 'bounce') {
          // Arms swing up in the air, knees give on landing.
          const b = Math.sin(((rig.actTime * 1.6) % 1) * Math.PI);
          addRotation(add, B.upperarm_l, 0, 0, 1, -1.3 * b);
          addRotation(add, B.upperarm_r, 0, 0, 1, 1.3 * b);
          const land = Math.max(0, 0.45 - b);
          addRotation(add, B.thigh_l, 1, 0, 0, -0.8 * land);
          addRotation(add, B.thigh_r, 1, 0, 0, -0.8 * land);
          addRotation(add, B.calf_l, 1, 0, 0, 1.6 * land);
          addRotation(add, B.calf_r, 1, 0, 0, 1.6 * land);
          addRotation(add, B.foot_l, 1, 0, 0, -0.8 * land);
          addRotation(add, B.foot_r, 1, 0, 0, -0.8 * land);
        } else if (rig.useObj?.stand?.motion === 'buzz') {
          // Knees soft on the vibrating plate, a little buzz through the body.
          const buzz = Math.sin(time * 70) * 0.012;
          addRotation(add, B.thigh_l, 1, 0, 0, -0.3);
          addRotation(add, B.thigh_r, 1, 0, 0, -0.3);
          addRotation(add, B.calf_l, 1, 0, 0, 0.6);
          addRotation(add, B.calf_r, 1, 0, 0, 0.6);
          addRotation(add, B.foot_l, 1, 0, 0, -0.3);
          addRotation(add, B.foot_r, 1, 0, 0, -0.3);
          addRotation(add, B.Head, 1, 0, 0, buzz);
        } else if (def.includes('sunrise')) {
          // A slow morning stretch: arms up overhead and down again.
          const up = smooth(0.5 - 0.5 * Math.cos(time * 0.9 + seed));
          addRotation(add, B.upperarm_l, 1, 0, 0, -2.7 * up);
          addRotation(add, B.upperarm_r, 1, 0, 0, -2.7 * up);
          addRotation(add, B.spine_02, 1, 0, 0, -0.12 * up);
          addRotation(add, B.Head, 1, 0, 0, -0.2 * up);
        }
        break;
    }
  }

  /**
   * Pedalling (content `seat.pedals`): each foot on its pedal, the ball of the foot on it, in time with
   * the object's loop (the bike's `pedals_turn`; without one playing, at its pace from the use's start).
   */
  private pedal(rig: SimRig, pedals: NonNullable<Seat['pedals']>): void {
    const fits = this.legFits.get(rig.body.name)?.get(rig.cur.key);
    const obj = rig.useObj;
    if (!fits || !obj) return;
    const B = this.set!.bone;
    const s = rig.scale;
    const w = smooth(rig.w);
    // The axle ahead of the Sim (the object's middle, then the pedals' offset from it).
    const ahead = (obj.cx - rig.x) * Math.sin(rig.yaw) + (obj.cz - rig.z) * Math.cos(rig.yaw) + pedals.fwd;
    const phase = (rig.useId >= 0 ? this.loopOf?.(rig.useId) : null) ?? (rig.actTime / PEDAL.turn) % 1;
    for (const [fit, thigh, calf, foot, half] of [
      [fits.l, B.thigh_l, B.calf_l, B.foot_l, 0.5],
      [fits.r, B.thigh_r, B.calf_r, B.foot_r, 0],
    ] as const) {
      // Round from the bottom (the right pedal's at rest) to the back, the top and the front.
      const a = (phase + half) * Math.PI * 2;
      const y = pedals.h - pedals.crank * Math.cos(a) + PEDAL.ankle[0] * s;
      const z = ahead - pedals.crank * Math.sin(a) + PEDAL.ankle[1] * s;
      const turn = this.legTo(fit, (y - rig.cur.rootY) / s, (z - rig.cur.rootFwd) / s);
      addRotation(this.add, thigh, 1, 0, 0, turn[0] * w);
      addRotation(this.add, calf, 1, 0, 0, turn[1] * w);
      // The foot stays about as level as in the clip, its toes dipping as the pedal goes down.
      addRotation(this.add, foot, 1, 0, 0, (-(turn[0] + turn[1]) * 0.75 + 0.15 * Math.sin(a)) * w);
    }
  }

  /** Leaning back by `angle` (rad) against a seat's slanted back (`Seat.recline`), split over the spine as `legsOut` does, the head tipped forward to keep looking ahead. */
  private recline(angle: number): void {
    const B = this.set!.bone;
    addRotation(this.add, B.spine_01, 1, 0, 0, -0.71 * angle);
    addRotation(this.add, B.spine_02, 1, 0, 0, -0.29 * angle);
    addRotation(this.add, B.Head, 1, 0, 0, 0.64 * angle);
  }

  /**
   * Legs out along the seat (content `seat.legs`: a chaise): nearly straight, the heels on it, the toes up;
   * leant back against the back, the head up to look along the legs.
   */
  private legsOut(rig: SimRig): void {
    const fits = this.legFits.get(rig.body.name)?.get(rig.cur.key);
    if (!fits) return;
    const B = this.set!.bone;
    const w = smooth(rig.w);
    addRotation(this.add, B.spine_01, 1, 0, 0, -0.2 * w);
    addRotation(this.add, B.spine_02, 1, 0, 0, -0.08 * w);
    addRotation(this.add, B.Head, 1, 0, 0, 0.18 * w);
    for (const [fit, thigh, calf, foot, bend] of [
      [fits.l, B.thigh_l, B.calf_l, B.foot_l, 0.96],
      [fits.r, B.thigh_r, B.calf_r, B.foot_r, 0.93],
    ] as const) {
      // The ankles a heel's height above the seat, the hips' joints being higher (`seatDrop`).
      const turn = this.legTo(fit, fit.hy - LEGS_OUT_DROP, fit.hz + (fit.l1 + fit.l2) * bend);
      addRotation(this.add, thigh, 1, 0, 0, turn[0] * w);
      addRotation(this.add, calf, 1, 0, 0, turn[1] * w);
      addRotation(this.add, foot, 1, 0, 0, -(turn[0] + turn[1]) * 0.25 * w);
    }
  }

  /**
   * Relaxed standing (see `STANCE`): over a skirt the hanging arms move out to clear it, faded in
   * over a few tenths of a second on the clips whose arms hang and out again for everything else.
   */
  private stance(rig: SimRig, pose: number, moving: boolean, dt: number): void {
    const B = this.set!.bone;
    const standing = pose === Pose.Stand && !moving && (!this.force || this.forceStance);
    const arms = standing && rig.skirt && RELAX_ARMS.has(rig.cur.key) && !(rig.upperW > 0.05) && rig.cuddleT < 0 ? 1 : 0;
    rig.relaxArms += (arms - rig.relaxArms) * (rig.dir?.still ? 1 : Math.min(1, dt * 5));
    const a = smooth(clamp(rig.relaxArms, 0, 1));
    if (a > 1e-3) {
      addRotation(this.add, B.upperarm_l, 0, 0, 1, STANCE.skirt * a);
      addRotation(this.add, B.upperarm_r, 0, 0, 1, -STANCE.skirt * a);
    }
  }

  /**
   * Face: blinks at irregular intervals, eyes that look at the conversation partner (or now and
   * then at the camera / around), an expression from the Sim's emotion and conversation, and the
   * mouth moving while it speaks (to the voice line's visemes when there is one). The eyes are
   * bones (local rotations in the head's frame); the rest are face morphs, mixed by `FaceMixer` and
   * written into the Sim's row after the appearance texels.
   */
  private animateFace(
    rig: SimRig,
    i: number,
    social: number,
    role: number,
    partner: SimRig | null,
    pose: number,
    moving: boolean,
    time: number,
    dt: number,
  ): void {
    const set = this.set!;
    const B = set.bone;
    const mixer = this.mixer;
    if (!mixer || B.eye_l === undefined) return;
    const add = this.add;
    const pair = partner ? Math.min(i, partner.id) : i;
    const asleep = pose === Pose.Lie && LYING.has(rig.cur.key) && (!rig.tagKnown || rig.tag === 'sleep' || rig.tag === 'nap');

    // Target expression: the emotion, then the conversation / activity on top.
    const tgt = this.faceTmp;
    // The friendly resting face, then the emotion on top.
    const emotion = this.emotionIds[rig.emotion - 1] ?? '';
    Object.assign(tgt, NEUTRAL);
    const warmth = COLD.has(emotion) ? 0 : rig.warmth;
    if (warmth > 0) for (const key of FACE_KEYS) tgt[key] += (FRIENDLY[key] ?? 0) * warmth;
    Object.assign(tgt, EXPRESSIONS[emotion] ?? NONE);
    let eyesClosed = 0;
    let talking = false;
    if (social) {
      const me = speaking(pair, role, time);
      switch (social) {
        case Anim.Laugh:
          Object.assign(tgt, LAUGH);
          tgt.jaw = 0.1 + 0.2 * Math.abs(Math.sin(time * 11 + i));
          break;
        case Anim.Flirt:
          Object.assign(tgt, EXPRESSIONS.flirty);
          talking = me;
          break;
        case Anim.Argue:
        case Anim.Fight:
          Object.assign(tgt, EXPRESSIONS.angry);
          talking = me || social === Anim.Fight;
          break;
        case Anim.Kiss:
          Object.assign(tgt, KISS);
          eyesClosed = 1;
          break;
        case Anim.Hug:
          Object.assign(tgt, HUG);
          eyesClosed = 0.85;
          break;
        default:
          talking = me;
          // Listening: a friendly face (outcome 1 = going well, 2 = badly).
          if (!me) tgt.smile = Math.max(tgt.smile, 0.35);
      }
    } else if (rig.tag === 'sing' || (rig.tag === 'talk' && rig.cur.key === 'talk')) {
      talking = true;
      tgt.smile = Math.max(tgt.smile, 0.4);
    }
    if (rig.tag === 'meditate') eyesClosed = 1;
    if (rig.tag === 'read' || rig.tag === 'type' || rig.tag === 'write' || rig.tag === 'paint' || rig.tag === 'cook' || rig.tag === 'wash')
      tgt.gazeDown = Math.max(tgt.gazeDown, 0.55);
    if (asleep) {
      eyesClosed = 1;
      Object.assign(tgt, NEUTRAL, ASLEEP);
    }
    const dir = rig.dir;
    if (dir?.face) Object.assign(tgt, dir.face);
    const dbg = this.faceDebug;
    if (dbg) {
      Object.assign(tgt, NEUTRAL, dbg);
      eyesClosed = dbg.closed ?? 0;
      talking = !!dbg.talk;
    }
    const f = dir?.still ? 1 : 1 - Math.exp(-dt * 6);
    const face = rig.face;
    for (const k of FACE_KEYS) face[k] += (tgt[k] - face[k]) * f;

    // A voice line takes over from the conversation's turn-taking.
    const speech = rig.speech;
    const now = performance.now() / 1000;
    const voiced = !!speech && now - rig.speechStart < (speech.times[speech.times.length - 1] ?? 0) + 0.3;
    if (voiced) talking = true;

    // Blinks: every 1.5 - 6 s at rest, more often while talking (sometimes twice), ~0.17 s each.
    if (time >= rig.nextBlink) {
      rig.blinkAt = time;
      const r = hash(Math.floor(time * 10) + i * 7919);
      const gap = talking ? 1 + 2.6 * hash(Math.floor(time * 13) + i * 31) : 1.5 + 4.5 * hash(Math.floor(time * 13) + i * 31);
      rig.nextBlink = time + (r < 0.15 ? 0.32 : gap);
    }
    const bt = (time - rig.blinkAt) / 0.17;
    const blink = !dir?.still && bt >= 0 && bt < 1 ? (bt < 0.4 ? bt / 0.4 : 1 - (bt - 0.4) / 0.6) : 0;
    const closed = Math.max(eyesClosed, smooth(Math.min(1, blink)));

    // Gaze: the partner's eyes; otherwise, now and then, the camera or a glance around.
    let gy = 0;
    let gp = 0.35 * face.gazeDown;
    const ex = this.heads[i * 3];
    const eyeY = this.heads[i * 3 + 1] - eyesBelowTop(rig);
    const ez = this.heads[i * 3 + 2];
    if (time >= rig.gazeUntil) {
      const r = hash(Math.floor(time * 7) + i * 131);
      const mode = social && partner ? (r < 0.82 ? 1 : 3) : moving ? 0 : r < 0.3 ? 2 : r < 0.65 ? 3 : 0;
      // A big shift of gaze often comes with a blink.
      if (mode !== rig.gazeMode && hash(Math.floor(time * 11) + i * 3) < 0.4 && time - rig.blinkAt > 0.5) rig.nextBlink = time;
      rig.gazeMode = mode;
      rig.gazeUntil = time + 1.2 + 2.8 * hash(Math.floor(time * 5) + i * 17);
      rig.glanceYaw = (hash(Math.floor(time * 3) + i) - 0.5) * 0.7;
      rig.glancePitch = (hash(Math.floor(time * 3) + i + 5) - 0.6) * 0.3;
    }
    if (!asleep && closed < 0.99) {
      const look = rig.audience?.visible ? rig.audience : partner;
      if (rig.gazeMode === 1 && look?.visible) {
        this.gazeAt(rig, ex, eyeY, ez, this.heads[look.id * 3], this.heads[look.id * 3 + 1] - eyesBelowTop(look), this.heads[look.id * 3 + 2]);
        gy = this.gaze[0];
        gp = this.gaze[1];
      } else if (rig.gazeMode === 2) {
        const cam = this.scene.activeCamera;
        if (cam && this.gazeAt(rig, ex, eyeY, ez, cam.globalPosition.x, cam.globalPosition.y, cam.globalPosition.z)) {
          gy = this.gaze[0];
          gp = this.gaze[1];
        } else {
          gy = rig.glanceYaw;
          gp = rig.glancePitch;
        }
      } else if (rig.gazeMode === 3) {
        gy = rig.glanceYaw;
        gp = rig.glancePitch + 0.3 * face.gazeDown;
      }
    }
    if (dir?.lookAt && !asleep && closed < 0.99 && this.gazeAt(rig, ex, eyeY, ez, dir.lookAt.x, dir.lookAt.y, dir.lookAt.z)) {
      gy = this.gaze[0];
      gp = this.gaze[1];
    } else if (dir?.still) {
      gy = 0;
      gp = 0.35 * face.gazeDown;
    }
    gy = clamp(gy, -0.42, 0.42);
    gp = clamp(gp, -0.28, 0.32);
    // Saccades: quick, not smoothed over seconds.
    const fe = dir?.still ? 1 : 1 - Math.exp(-dt * 22);
    rig.eyeYaw += (gy - rig.eyeYaw) * fe;
    rig.eyePitch += (gp - rig.eyePitch) * fe;

    // Eye bones (local axes = the head's: x right, y up, z forward; +x rotation turns the front down).
    addLocal(add, B.eye_l, 0, 1, 0, rig.eyeYaw);
    addLocal(add, B.eye_l, 1, 0, 0, rig.eyePitch);
    addLocal(add, B.eye_r, 0, 1, 0, rig.eyeYaw);
    addLocal(add, B.eye_r, 1, 0, 0, rig.eyePitch);

    // Morphs: the expression (lids following the eyes), then speech. Faces too small on screen to
    // show them skip their morphs (in the vertex shader too).
    const morphRow = this.row(i) + set.bones.length * 16 + APPEARANCE_TEXELS * 4;
    if (this.facePx[i] < MORPH_PX && !dir && !dbg) {
      mixer.off(this.data, morphRow);
      return;
    }
    mixer.begin();
    mixer.mood(tgt.mood, 1, rig.faceL, rig.faceR);
    mixer.expression(tgt, rig.faceL, rig.faceR);
    if (!talking && !asleep && !dir?.still && !dbg) mixer.micro(time, rig.seed, 1);
    mixer.settle(rig.faceState, f);
    mixer.eyes(closed, rig.eyePitch);
    if (voiced) {
      mixer.speaking(1);
      this.speak(rig, speech!, now - rig.speechStart);
    } else if (talking) {
      mixer.speaking(1);
      this.babble(rig, i, time);
    } else if (dbg?.viseme) {
      mixer.viseme(dbg.viseme, 1, 1);
    }
    if (dbg?.channels) mixer.raw(dbg.channels);
    mixer.write(this.data, morphRow);
  }

  /**
   * Visemes of the voice line at `t` (s from its first sample): the mouth moves a little ahead of
   * the sound, and each shape blends in from the last over ~70 ms; quiet stretches open less.
   */
  private speak(rig: SimRig, track: VisemeTrack, t: number): void {
    const mixer = this.mixer!;
    const times = track.times;
    const n = track.visemes.length;
    if (!n) return;
    const tt = t + 0.04;
    let k = rig.speechSeg;
    if (k >= n || tt < times[k]) k = 0;
    while (k < n - 1 && tt >= times[k + 1]) k++;
    rig.speechSeg = k;
    if (tt < times[0] - 0.08 || tt > times[n] + 0.08) return;
    const e = track.energy;
    const energy = e.length ? e[Math.min(e.length - 1, Math.max(0, Math.floor(t * ENERGY_RATE)))] : 1;
    // Fade in before the first segment and out after the last.
    const edge = Math.min(1, (tt - times[0] + 0.08) / 0.08, (times[n] + 0.08 - tt) / 0.08);
    const a = smooth(clamp((tt - times[k]) / 0.07, 0, 1));
    mixer.viseme(track.visemes[k], a * edge, energy);
    if (k > 0 && a < 1) mixer.viseme(track.visemes[k - 1], (1 - a) * edge, energy);
  }

  /** Talking without a voice line: syllable-like mouth shapes, in phrases. */
  private babble(rig: SimRig, i: number, time: number): void {
    const rate = 5.5;
    const s = time * rate + rig.seed * 40;
    const k = Math.floor(s);
    const f = s - k;
    const phrase = Math.sin(time * 1.9 + rig.seed * 9);
    if (phrase < -0.55) return;
    const env = Math.sin(Math.PI * f) * (0.55 + 0.45 * Math.min(1, phrase + 0.55));
    const v = BABBLE[Math.floor(hash(k * 7 + i * 131) * BABBLE.length)];
    this.mixer!.viseme(v, env, 0.6);
  }

  /** Lip sync: `track` plays on Sim row `index` from `startTime` (s, `performance.now` clock); null stops it. */
  setSpeech(index: number, track: VisemeTrack | null, startTime: number): void {
    const rig = this.sims[index];
    if (!rig) return;
    rig.speech = track && track.visemes.length ? track : null;
    rig.speechStart = startTime;
    rig.speechSeg = 0;
  }

  // --- helpers --------------------------------------------------------------------------

  private row(i: number): number {
    return i * this.width * 4;
  }

  /**
   * Uploads the pose texture's rows that changed since the last upload (Sims drawn now, or just
   * hidden), as one band of rows: a few of the 64 most frames, none while the game is paused.
   */
  private upload(): void {
    const stride = this.width * 4;
    const data = this.data;
    const uploaded = this.uploaded;
    let lo = -1;
    let hi = -1;
    for (let i = 0; i < MAX_CHARACTERS; i++) {
      if (!this.visible[i] && !this.uploadedVisible[i]) continue;
      this.uploadedVisible[i] = this.visible[i];
      const end = (i + 1) * stride;
      let k = i * stride;
      while (k < end && data[k] === uploaded[k]) k++;
      if (k === end) continue;
      uploaded.set(data.subarray(k, end), k);
      if (lo < 0) lo = i;
      hi = i;
    }
    this.posed = lo >= 0;
    const texture = this.texture.getInternalTexture();
    if (lo < 0 || !texture) return;
    (this.scene.getEngine() as Engine | WebGPUEngine).updateTextureData(texture, data.subarray(lo * stride, (hi + 1) * stride), 0, lo, this.width, hi - lo + 1);
  }

  /** The seat a sitting Sim is on: the object they use if it's a seat, else the seat under them
   * (content `seatNear`: watching TV from the sofa). */
  private seatUnder(rig: SimRig, useObj: PlacedObject | null | undefined): PlacedObject | null {
    if (useObj && useObj.restful) return useObj;
    const under = this.objectAt(rig.x, rig.z);
    return under ?? useObj ?? null;
  }

  private objectAt(x: number, z: number): PlacedObject | null {
    let best: PlacedObject | null = null;
    let bestD = Infinity;
    for (const o of this.objects) {
      if (x < o.minX - 0.05 || x > o.maxX + 0.05 || z < o.minZ - 0.05 || z > o.maxZ + 0.05) continue;
      const d = Math.abs(x - o.cx) + Math.abs(z - o.cz);
      if (o.restful && d < bestD) {
        best = o;
        bestD = d;
      }
    }
    return best;
  }

  /** The object a standing Sim faces (within reach), if any. */
  private objectInFront(rig: SimRig): PlacedObject | null {
    const fx = rig.x + Math.sin(rig.yaw) * 0.55;
    const fz = rig.z + Math.cos(rig.yaw) * 0.55;
    let best: PlacedObject | null = null;
    let bestD = 0.6;
    for (const o of this.objects) {
      if (!USE[o.def]) continue;
      const qx = clamp(fx, o.minX, o.maxX);
      const qz = clamp(fz, o.minZ, o.maxZ);
      const d = Math.hypot(fx - qx, fz - qz);
      if (d < bestD) {
        best = o;
        bestD = d;
      }
    }
    return best;
  }

  /** Head-relative eye yaw / pitch towards a point (into `gaze`); false if it's behind the Sim. */
  private gazeAt(rig: SimRig, ex: number, ey: number, ez: number, tx: number, ty: number, tz: number): boolean {
    const dx = tx - ex;
    const dz = tz - ez;
    this.gaze[0] = wrapAngle(Math.atan2(dx, dz) - rig.yaw - rig.lookYaw);
    this.gaze[1] = -Math.atan2(ty - ey, Math.hypot(dx, dz)) - rig.lookPitch * 0.7;
    return Math.abs(this.gaze[0]) < 1.4;
  }

  /**
   * Blanket over a Sim asleep in a bed: fitted to the slot (half of a double bed, or a single bed)
   * and to the bed's foot end, pulled up from the foot end over ~0.5 s once the Sim lies flat (and
   * pushed back down as they get up); zero-scaled otherwise.
   */
  private writeBlanket(rig: SimRig, i: number, pose: number, dt: number, fresh: boolean): void {
    // The body's own quilt (a life stage's, draped over its own pose), else its family's base body's.
    const own = this.blankets.get(rig.body.name);
    const blanket = own ?? this.blankets.get(rig.body.family);
    if (!blanket) return;
    const bed = rig.bed;
    // Only once the body lies flat (the crossfade from sitting on the edge done): pulled up while the legs
    // still swing up onto the bed, it showed the shoes through it halfway down the bed.
    const flat = rig.w > 0.97 || rig.cover > 0;
    const under = pose === Pose.Lie && LYING.has(rig.cur.key) && flat && !!bed && BLANKET_BEDS.has(bed.def) && (rig.tagKnown ? rig.tag === 'sleep' || rig.tag === 'nap' : true);
    rig.cover = fresh ? (under ? 1 : 0) : clamp(rig.cover + (under ? dt * 2 : -dt * 3), 0, 1);
    const m = blanket.matrices;
    const o = i * 16;
    if (rig.cover <= 0 || !bed) {
      m.fill(0, o, o + 16);
      return;
    }
    const s = rig.scale;
    const fx = Math.sin(rig.yaw);
    const fz = Math.cos(rig.yaw);
    // Getting up: the Sim is already off the bed; the quilt stays on its slot as it slides off.
    const x = rig.fromBed ? rig.lieX : rig.x;
    const z = rig.fromBed ? rig.lieZ : rig.z;
    // Bed extent across the Sim (slot width) and the distance to its foot end.
    const across = Math.abs(fz) * (bed.maxX - bed.minX) + Math.abs(fx) * (bed.maxZ - bed.minZ);
    // Two to a wide bed: each quilt half the bed and a little more, moved a little towards its middle, so the
    // two meet over it (the one lifted a hair over the other) with no strip of the bed's own cover between.
    const wide = across >= 1.8;
    const slot = wide ? across / 2 + BLANKET_MEET * 2 : Math.min(across - 0.08, 1.0);
    const side = wide ? Math.sign((x - bed.cx) * fz - (z - bed.cz) * fx) * BLANKET_MEET : 0;
    const tx = fx > 1e-3 ? (bed.maxX - x) / fx : fx < -1e-3 ? (bed.minX - x) / fx : Infinity;
    const tz = fz > 1e-3 ? (bed.maxZ - z) / fz : fz < -1e-3 ? (bed.minZ - z) / fz : Infinity;
    const foot = Math.min(tx, tz) - (BEDS[bed.def]?.foot ?? 0.05);
    // A body smaller than the one the quilt was made for (a child's with no quilt of its own yet): the quilt
    // shrunk to it along the body and in height, from its feet, so its head is out (the width stays the bed's).
    const zs = own ? s : s * rig.size;
    const z0 = lieCentre(rig) - LIE_CENTRE * zs;
    // The quilt past the toes reaches the bed's foot end (local units, at the Sim's scale).
    blanket.ext[i * 2] = clamp((foot - z0) / zs - blanket.footZ, -0.22, 1.2);
    const c = smooth(rig.cover);
    // Pulled up: the quilt from its foot end to this line (local z) at full height, the rest folded away
    // under it (`BlanketPlugin`), so nothing shows through it on the way up or down.
    blanket.ext[i * 2 + 1] = c >= 1 ? -9 : blanket.footZ - (blanket.footZ - blanket.topZ + 0.02) * c;
    const sx = (slot / BLANKET_WIDTH) * (0.9 + 0.1 * c);
    // Two Sims in one bed: offset the quilts a hair so their tops don't z-fight where they meet.
    const lift = (i % 2) * 0.004;
    // Rows: x axis, y axis, z axis, translation (Babylon row-vector layout).
    m[o] = fz * sx;
    m[o + 1] = 0;
    m[o + 2] = -fx * sx;
    m[o + 3] = 0;
    m[o + 4] = 0;
    m[o + 5] = zs;
    m[o + 6] = 0;
    m[o + 7] = 0;
    m[o + 8] = fx * zs;
    m[o + 9] = 0;
    m[o + 10] = fz * zs;
    m[o + 11] = 0;
    m[o + 12] = x - fz * side + fx * z0;
    m[o + 13] = rig.bedH + lift;
    m[o + 14] = z + fx * side + fz * z0;
    m[o + 15] = 1;
  }

  /** A body's quilt (`blankets`), draped over its lying pose, every row's instance hidden. */
  private addQuilt(body: BodyData, lying: { positions: Float32Array; chestZ: number; feetZ: number }): void {
    const shape = buildBlanket(this.scene, `simBlanket:${body.name}`, lying.positions, lying.chestZ, lying.feetZ, this.quilt!);
    const matrices = new Float32Array(MAX_CHARACTERS * 16);
    const ext = new Float32Array(MAX_CHARACTERS * 2);
    shape.mesh.thinInstanceSetBuffer('matrix', matrices, 16, false);
    shape.mesh.thinInstanceSetBuffer('blanketExt', ext, 2, false);
    shape.mesh.setEnabled(false);
    this.blankets.set(body.name, { ...shape, matrices, ext, base: body.name === body.family });
  }

  /**
   * The lying pose of a body (scale 1, slot frame: x across, y above the mattress, z towards the
   * feet), skinned on the CPU once for the blanket: the `lie` clip sleeping Sims play. Also returns
   * the chest line and the toes' z.
   */
  private lyingPose(body: BodyData): {
    positions: Float32Array;
    chestZ: number;
    feetZ: number;
  } {
    const set = this.set!;
    const B = set.bone;
    const NB = set.bones.length;
    const mats = new Float32Array(NB * 16);
    const head = new Float32Array(3);
    sampleClip(set, this.clip('lie'), 0, this.qA, this.pA);
    resetAdditive(this.add);
    const sp = Math.sin(-Math.PI / 4);
    const cp = Math.cos(-Math.PI / 4);
    skin(
      set,
      body,
      this.qA,
      this.pA,
      this.add,
      {
        x: 0,
        // (As `lieBack`: a smaller body than its family's base body lies lower.)
        y: LIE_BACK * (body.height / (set.bodies.get(body.family)?.height ?? body.height)),
        z: LIE_CENTRE,
        qx: sp,
        qy: 0,
        qz: 0,
        qw: cp,
        scale: 1,
      },
      this.scratch,
      mats,
      0,
      head,
    );
    const chestZ = Math.max(this.scratch.worldT[B.upperarm_l * 3 + 2], this.scratch.worldT[B.upperarm_r * 3 + 2]) + 0.07;
    resetAdditive(this.add);
    // The body plus the clothes that stand off it most (skirts, boots).
    const parts = [...Object.keys(body.bodyChunks), 'bottom.skirt', 'shoes.boots'].map((k) => body.parts.get(k)).filter((p): p is NonNullable<typeof p> => !!p);
    const total = parts.reduce((n, p) => n + p.positions.length / 3, 0);
    const out = new Float32Array(total * 3);
    let feetZ = -Infinity;
    let base = 0;
    for (const part of parts) {
      const n = part.positions.length / 3;
      for (let v = 0; v < n; v++) {
        const px = part.positions[v * 3];
        const py = part.positions[v * 3 + 1];
        const pz = part.positions[v * 3 + 2];
        let x = 0;
        let y = 0;
        let z = 0;
        for (let k = 0; k < 4; k++) {
          const w = part.weights[v * 4 + k];
          if (w <= 0) continue;
          const m = part.joints[v * 4 + k] * 16;
          x += w * (mats[m] * px + mats[m + 4] * py + mats[m + 8] * pz + mats[m + 12]);
          y += w * (mats[m + 1] * px + mats[m + 5] * py + mats[m + 9] * pz + mats[m + 13]);
          z += w * (mats[m + 2] * px + mats[m + 6] * py + mats[m + 10] * pz + mats[m + 14]);
        }
        out[(base + v) * 3] = x;
        out[(base + v) * 3 + 1] = y;
        out[(base + v) * 3 + 2] = z;
        if (part === parts[0] && z > feetZ) feetZ = z;
      }
      base += n;
    }
    return { positions: out, chestZ, feetZ };
  }

  /** Where each leg is in the clips whose feet are placed (`LEG_CLIPS`), at their start (body space, scale 1). */
  private measureLegs(body: BodyData): Map<string, LegFit> {
    const set = this.set!;
    const B = set.bone;
    const tmp = new Float32Array(set.bones.length * 16);
    const head = new Float32Array(3);
    const t = this.scratch.worldT;
    const fits = new Map<string, LegFit>();
    for (const key of LEG_CLIPS) {
      const clip = set.clips.get(key);
      if (!clip) continue;
      sampleClip(set, clip, 0, this.qA, this.pA);
      resetAdditive(this.add);
      skin(set, body, this.qA, this.pA, this.add, { x: 0, y: 0, z: 0, qx: 0, qy: 0, qz: 0, qw: 1, scale: 1 }, this.scratch, tmp, 0, head);
      const side = (thigh: number, calf: number, foot: number) => {
        const [hy, hz, ky, kz, ay, az] = [t[thigh * 3 + 1], t[thigh * 3 + 2], t[calf * 3 + 1], t[calf * 3 + 2], t[foot * 3 + 1], t[foot * 3 + 2]];
        return { hy, hz, l1: Math.hypot(ky - hy, kz - hz), l2: Math.hypot(ay - ky, az - kz), t0: Math.atan2(kz - hz, hy - ky), c0: Math.atan2(az - kz, ky - ay) };
      };
      fits.set(key, {
        l: side(B.thigh_l, B.calf_l, B.foot_l),
        r: side(B.thigh_r, B.calf_r, B.foot_r),
        spine: [t[B.spine_01 * 3 + 1], t[B.spine_01 * 3 + 2]],
        head: [t[B.Head * 3 + 1], t[B.Head * 3 + 2]],
      });
    }
    return fits;
  }

  /**
   * Turns a leg (`fit`, its clip's) so its ankle reaches (`y`, `z`) (body space, scale 1; the knee bent
   * forward, out of reach straightened towards it): the thigh's and calf's rotations about x into
   * `legAdd`, added to the clip's (see `addRotation`: a positive x rotation swings a limb from +z
   * towards -y).
   */
  private legTo(fit: LegSide, y: number, z: number): Float32Array {
    const { hy, hz, l1, l2 } = fit;
    const r = clamp(Math.hypot(y - hy, z - hz), Math.abs(l1 - l2) + 0.01, l1 + l2 - 0.002);
    // Direction angles from straight down (-y) towards the front (+z).
    const d = Math.atan2(z - hz, hy - y);
    const thigh = d + Math.acos(clamp((l1 * l1 + r * r - l2 * l2) / (2 * l1 * r), -1, 1));
    const calf = d - Math.acos(clamp((l2 * l2 + r * r - l1 * l1) / (2 * l2 * r), -1, 1));
    this.legAdd[0] = fit.t0 - thigh;
    this.legAdd[1] = fit.c0 - calf - this.legAdd[0];
    return this.legAdd;
  }

  /**
   * Lying in a hammock's sling (`Bed.sling`; root `fwd0` as on a bed): the root's height and forward
   * offset that put the hips down in the dip with the back on the sling and, the upper body raised by
   * `rig.slingLift` (rad, about the spine's base), the head on the pillow. Null without the body's fit.
   */
  private inSling(rig: SimRig, sling: NonNullable<Bed['sling']>, lift: number, fwd0: number): Float32Array | null {
    const fit = this.legFits.get(rig.body.name)?.get('lie');
    if (!fit) return null;
    const s = rig.scale;
    const dy = (fit.head[0] - fit.spine[0]) * s;
    const dz = (fit.head[1] - fit.spine[1]) * s;
    const reach = Math.hypot(dy, dz);
    const lean = Math.atan2(dz, dy);
    // The head as far above the pillow as above a mattress.
    const want = slingTop(sling, SLING.crown - sling.head) + lift + lieBack(rig) + fit.head[1] * s + SLING.pillow;
    const out = this.slingAt;
    let fwd = fwd0;
    for (let k = 0; k < 3; k++) {
      // Along the sling (from its middle towards the feet): the spine's base, which the root follows down.
      const base = fwd - fit.spine[0] * s;
      out[0] = slingTop(sling, base) + lift + lieBack(rig) + SLING.back;
      const rise = clamp(Math.asin(clamp((want - out[0] - fit.spine[1] * s) / reach, -1, 1)) - lean, 0, SLING.lift);
      rig.slingLift = rise;
      // Moved along so the head, raised, is over the pillow.
      fwd += SLING.crown - sling.head - (base - (dy * Math.cos(rise) - dz * Math.sin(rise)));
    }
    out[1] = fwd;
    return out;
  }

  /**
   * The pose in a hammock's sling (`inSling`): the back curved up from the hips to the raised head, the
   * knees up and the heels on the sling inside the spreader bar.
   */
  private slingPose(rig: SimRig, sling: NonNullable<Bed['sling']>): void {
    const fits = this.legFits.get(rig.body.name)?.get('lie');
    if (!fits) return;
    const B = this.set!.bone;
    const add = this.add;
    const w = smooth(rig.w);
    const rise = rig.slingLift * w;
    addRotation(add, B.spine_01, 1, 0, 0, rise * 0.5);
    addRotation(add, B.spine_02, 1, 0, 0, rise * 0.3);
    addRotation(add, B.spine_03, 1, 0, 0, rise * 0.2);
    const s = rig.scale;
    const lift = rig.bed?.lift ?? 0;
    const { rootY, rootFwd } = rig.cur;
    for (const [fit, thigh, calf, foot, short] of [
      [fits.l, B.thigh_l, B.calf_l, B.foot_l, 0],
      [fits.r, B.thigh_r, B.calf_r, B.foot_r, 0.05],
    ] as const) {
      // Along the sling: the hip, and the ankle as far as the leg reaches, short of the spreader bar.
      const hip = rootFwd - fit.hy * s;
      const at = Math.min(sling.half - SLING.feet - short, hip + (fit.l1 + fit.l2) * s * 0.97);
      const turn = this.legTo(fit, (rootFwd - at) / s, (slingTop(sling, at) + lift + SLING.ankle * s - rootY) / s);
      addRotation(add, thigh, 1, 0, 0, turn[0] * w);
      addRotation(add, calf, 1, 0, 0, turn[1] * w);
      addRotation(add, foot, 1, 0, 0, -(turn[0] + turn[1]) * 0.4 * w);
    }
  }

  /** Pelvis height / forward offset of the seated and lying poses (body space, scale 1). */
  private measureSit(body: BodyData): SeatFit {
    const set = this.set!;
    const tmp = new Float32Array(set.bones.length * 16);
    const head = new Float32Array(3);
    const pose = (key: string, t: number) => {
      const clip = set.clips.get(key)!;
      sampleClip(set, clip, t, this.qA, this.pA);
      resetAdditive(this.add);
      skin(set, body, this.qA, this.pA, this.add, { x: 0, y: 0, z: 0, qx: 0, qy: 0, qz: 0, qw: 1, scale: 1 }, this.scratch, tmp, 0, head);
      const p = set.bone.pelvis * 3;
      return { y: this.scratch.worldT[p + 1], z: this.scratch.worldT[p + 2] };
    };
    const sit = pose('sit', 0);
    // The pelvis joint rests `seatDrop` above the seat surface (sets without it: ~11 cm).
    return { y: sit.y - (body.seatDrop ?? 0.11), z: sit.z };
  }

  /**
   * The mesh of a part of a body (`shadow`: its shadow-only copy, drawn into the sun's shadow map
   * but not by the camera).
   */
  private partMesh(body: BodyData, part: string, shadow = false): PartMesh | null {
    const key = `${body.name}:${part}`;
    const meshes = shadow ? this.shadowMeshes : this.meshes;
    const cached = meshes.get(key);
    if (cached) return cached;
    const data = body.parts.get(part);
    if (!data) return null;
    const mesh = new Mesh(`${shadow ? 'simShadow' : 'sim'}:${key}`, this.scene);
    const vd = new VertexData();
    vd.positions = data.positions;
    vd.normals = data.normals;
    vd.indices = data.indices;
    vd.matricesIndices = data.joints;
    vd.matricesWeights = data.weights;
    vd.applyToMesh(mesh, false);
    // Fields, morph slots and tone interleaved in one buffer: WebGPU allows 8 vertex buffers per
    // draw (here: position, normal, joints, weights, this, the instance matrices and settings).
    // Parts no morph moves get slot -1 (and no `simMorph` attribute).
    const n = data.fields.length / 4;
    const all = new Float32Array(n * 12);
    for (let v = 0; v < n; v++) {
      all.set(data.fields.subarray(v * 4, v * 4 + 4), v * 12);
      if (data.morph) all.set(data.morph.subarray(v * 4, v * 4 + 4), v * 12 + 4);
      else all[v * 12 + 4] = -1;
      all.set(data.tone.subarray(v * 4, v * 4 + 4), v * 12 + 8);
    }
    const buffer = new Buffer(this.scene.getEngine(), all, false, 12);
    mesh.setVerticesBuffer(buffer.createVertexBuffer('simField', 0, 4));
    if (data.morph && this.morph) mesh.setVerticesBuffer(buffer.createVertexBuffer('simMorph', 4, 4));
    mesh.setVerticesBuffer(buffer.createVertexBuffer('simTone', 8, 4));
    mesh.skeleton = this.skeleton(body);
    mesh.numBoneInfluencers = 4;
    mesh.bakedVertexAnimationManager = this.vat;
    mesh.material = this.material(part);
    mesh.isPickable = false;
    mesh.receiveShadows = !shadow;
    // Shadow copies: on a layer no camera draws (the shadow map draws its own list of meshes).
    if (shadow) mesh.layerMask = LAYER_SHADOW_ONLY;
    // Sims move every frame (bones carry the placement): skip culling and bounds work.
    mesh.alwaysSelectAsActiveMesh = true;
    mesh.doNotSyncBoundingInfo = true;
    mesh.freezeWorldMatrix();
    const range = shadow ? Range.Always : part.startsWith('lod.') ? Range.Far : part === 'body' || /^(body|top|bottom|shoes)\./.test(part) ? Range.Near : Range.Always;
    const pm: PartMesh = { mesh, settings: new Float32Array(0), all: new Float32Array(0), detail: shadow ? 0 : (DETAIL_PX[part] ?? 0), range };
    meshes.set(key, pm);
    if (!shadow) this.parts.set(mesh, pm);
    return pm;
  }

  /**
   * The Sim rows a part mesh draws now: one each four numbers of `rows`, `count` of them (its
   * thin instances); null for any other mesh. (The mirrors draw only the residents they show.)
   */
  rowsOf(mesh: AbstractMesh): { rows: Float32Array; count: number } | null {
    const pm = this.parts.get(mesh);
    return pm ? { rows: pm.settings, count: pm.mesh.thinInstanceCount } : null;
  }

  /** Draws `pm` for Sim rows `ids` (thin instances). */
  private setInstances(pm: PartMesh, ids: readonly number[]): void {
    const n = ids.length;
    const identity = new Float32Array(n * 16);
    const settings = new Float32Array(n * 4);
    ids.forEach((id, j) => {
      identity[j * 16] = identity[j * 16 + 5] = identity[j * 16 + 10] = identity[j * 16 + 15] = 1;
      settings[j * 4] = settings[j * 4 + 1] = id;
    });
    pm.mesh.thinInstanceSetBuffer('matrix', identity, 16, true);
    // Parts drawn by distance rewrite their rows each frame.
    pm.mesh.thinInstanceSetBuffer('bakedVertexAnimationSettingsInstanced', settings, 4, pm.detail === 0 && pm.range === Range.Always);
    pm.settings = settings;
    pm.all = settings.slice();
    pm.mesh.isVisible = n > 0;
  }

  /**
   * How big each Sim's face is on screen (`facePx`: px per 5 cm), and the small face parts (teeth,
   * tongue, lashes, brows) drawn only on the faces big enough to show them.
   */
  private faceDetail(count: number): void {
    const cam = this.scene.activeCamera;
    const px = this.facePx;
    if (!cam || cam.mode !== 0) {
      px.fill(1e6);
    } else {
      const k = (0.05 * this.scene.getEngine().getRenderHeight()) / (2 * Math.tan(cam.fov / 2));
      const c = cam.globalPosition;
      for (let i = 0; i < count; i++) {
        const rig = this.sims[i];
        if (!rig || !this.visible[i]) {
          px[i] = 0;
          continue;
        }
        // Directed Sims (the creator's stage, portraits drawn by their own cameras): full detail.
        if (rig.dir) {
          px[i] = 1e6;
          continue;
        }
        const d = Math.hypot(this.heads[i * 3] - c.x, this.heads[i * 3 + 1] - c.y, this.heads[i * 3 + 2] - c.z);
        px[i] = (k * rig.scale * rig.head) / Math.max(d, 0.05);
      }
    }
    // Low detail far from the camera (with a gap between switching down and up).
    const far = this.far;
    for (let i = 0; i < count; i++) {
      if (far[i] && px[i] > LOD_NEAR_PX) far[i] = 0;
      else if (!far[i] && px[i] < LOD_FAR_PX && this.hasLod) far[i] = 1;
    }
    for (const pm of this.meshes.values()) {
      if ((!pm.detail && pm.range === Range.Always) || !pm.mesh.isEnabled()) continue;
      const all = pm.all;
      const out = pm.settings;
      const near = pm.range === Range.Near;
      const ranged = pm.range !== Range.Always;
      let n = 0;
      let changed = false;
      for (let j = 0; j < all.length; j += 4) {
        const id = all[j];
        if (px[id] < pm.detail) continue;
        if (ranged && (far[id] === 1) === near) continue;
        if (out[n * 4] !== id) {
          out[n * 4] = out[n * 4 + 1] = id;
          changed = true;
        }
        n++;
      }
      if (changed || pm.mesh.thinInstanceCount !== n) {
        pm.mesh.thinInstanceCount = n;
        pm.mesh.thinInstanceBufferUpdated('bakedVertexAnimationSettingsInstanced');
      }
      // (With no thin instances Babylon would draw the mesh once, uninstanced: unposed and untinted.)
      pm.mesh.isVisible = n > 0;
    }
  }

  /** A skeleton only declares the bone count to the shaders; poses come from the texture. */
  private skeleton(body: BodyData): Skeleton {
    let sk = this.skeletons.get(body.family);
    if (sk) return sk;
    const set = this.set!;
    sk = new Skeleton(`sim:${body.family}`, `sim:${body.family}`, this.scene);
    const bones: Bone[] = [];
    set.bones.forEach((name, b) => {
      const p = set.parents[b];
      bones.push(new Bone(name, sk!, p >= 0 ? bones[p] : null, Matrix.Identity()));
    });
    this.skeletons.set(body.family, sk);
    return sk;
  }

  private material(part: string): PBRMaterial {
    const set = this.set!;
    const kind: CharacterSurface =
      part === 'body' || part === 'lod.body' || part.startsWith('body.')
        ? 'body'
        : part === 'eyes'
          ? 'eye'
          : part === 'brows'
            ? 'brow'
            : part === 'lashes'
              ? 'lash'
              : part === 'teeth' || part === 'tongue'
                ? 'mouth'
                : /^(lod\.)?(top|bottom|shoes)\./.test(part)
                  ? 'cloth'
                  : 'hair';
    const key = kind === 'mouth' ? `${kind}:${part}` : kind;
    const cached = this.materials.get(key);
    if (cached) return cached;
    const mat = new PBRMaterial(`sim:${key}`, this.scene);
    mat.metallic = 0;
    mat.maxSimultaneousLights = 6;
    // No textures: every colour comes from the Sim's appearance texels and the vertex tone (see the plugin).
    mat.albedoColor = new Color3(1, 1, 1);
    if (kind === 'body') {
      mat.roughness = 0.55;
    } else if (kind === 'cloth') {
      // As rough as the clothes painted on the body (so shells and paint match).
      mat.roughness = 0.85;
      // Open hems and collars show their inside.
      mat.backFaceCulling = false;
      mat.twoSidedLighting = true;
    } else if (kind === 'hair') {
      mat.roughness = 0.5;
      mat.sheen.isEnabled = true;
      mat.sheen.intensity = 0.4;
      mat.sheen.linkSheenWithAlbedo = true;
      mat.backFaceCulling = false;
      mat.twoSidedLighting = true;
    } else if (kind === 'brow' || kind === 'lash') {
      // Opaque: brows in the hair colour (see the plugin), lashes dark brown.
      if (kind === 'lash') mat.albedoColor = new Color3(0.16, 0.12, 0.1);
      mat.roughness = 0.6;
      mat.backFaceCulling = false;
      mat.twoSidedLighting = true;
    } else if (kind === 'mouth') {
      // Warm, soft and hardly reflecting the sky (the mouth hides most of it).
      mat.albedoColor = part === 'teeth' ? new Color3(1, 0.96, 0.9) : new Color3(0.85, 0.8, 0.8);
      mat.roughness = 0.5;
      mat.environmentIntensity = 0.35;
    } else {
      // Eyes: drawn by the plugin (sclera, iris, pupil); wet and glossy (a clear coat for the catchlight).
      mat.roughness = 0.25;
      mat.clearCoat.isEnabled = true;
      mat.clearCoat.intensity = 1;
      mat.clearCoat.roughness = 0.03;
    }
    new CharacterPlugin(mat, kind, set.bones.length * 4, this.texture, this.morph);
    this.lib.adopt(mat);
    this.materials.set(key, mat);
    return mat;
  }
}

function advance(p: Player, dt: number): void {
  if (!p.clip) return;
  p.time += dt * p.rate;
  if (p.clip.loop && p.time > p.clip.duration * 64) p.time %= p.clip.duration;
}

/** Head-top marker (speech bubbles, the selection marker, the creator's framing) above the crown (m, body scale). */
const HEAD_MARGIN = 0.04;

/** The eyes' height below a Sim's head-top marker (m). */
function eyesBelowTop(rig: SimRig): number {
  const b = rig.body;
  return ((b.eyeY !== null ? b.height - b.eyeY : b.headAbove * 0.7) + HEAD_MARGIN) * rig.scale * rig.head;
}

/** Mouth shapes for talk without a voice line (viseme indices: aa, E, O, I, U, PP, DD, SS, nn). */
const BABBLE = [10, 11, 13, 12, 14, 1, 4, 7, 8, 10, 11];

/** Whose turn it is in a conversation: the starter (role 1) and the other Sim alternate; those listening in (role 3) only listen. */
function speaking(pair: number, role: number, time: number): boolean {
  if (role === 0 || role === 3) return false;
  const turn = Math.floor((time + (pair % 5) * 0.53) / 2.8);
  return (turn % 2 === 0) === (role === 1);
}

function splitKey(key: string): [string, string] {
  const at = key.indexOf(':');
  return [key.slice(0, at), key.slice(at + 1)];
}

function hash(n: number): number {
  let h = Math.imul((n | 0) ^ 0x9e3779b9, 0x85ebca6b);
  h = Math.imul(h ^ (h >>> 13), 0xc2b2ae35);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

function clamp(v: number, lo: number, hi: number): number {
  return v < lo ? lo : v > hi ? hi : v;
}

/** (x, y, z) made 1 long, into `UNIT`. */
function unit(x: number, y: number, z: number): Float32Array {
  const l = Math.hypot(x, y, z) || 1;
  UNIT[0] = x / l;
  UNIT[1] = y / l;
  UNIT[2] = z / l;
  return UNIT;
}
const UNIT = new Float32Array(3);

/** Sets a frame (`Characters.placeFrame`): its x, y and z axes, then its origin. */
function setFrame(f: Float32Array, xx: number, xy: number, xz: number, yx: number, yy: number, yz: number, zx: number, zy: number, zz: number, ox: number, oy: number, oz: number): void {
  f[0] = xx;
  f[1] = xy;
  f[2] = xz;
  f[3] = yx;
  f[4] = yy;
  f[5] = yz;
  f[6] = zx;
  f[7] = zy;
  f[8] = zz;
  f[9] = ox;
  f[10] = oy;
  f[11] = oz;
}

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

/** 0 before `from`, easing to 1 over `length` after it. */
function ramp(t: number, from: number, length: number): number {
  return smooth(clamp((t - from) / length, 0, 1));
}

function smooth(t: number): number {
  return t * t * (3 - 2 * t);
}

function wrapAngle(a: number): number {
  a %= Math.PI * 2;
  if (a > Math.PI) a -= Math.PI * 2;
  if (a < -Math.PI) a += Math.PI * 2;
  return a;
}

function lerpAngle(a: number, b: number, t: number): number {
  return a + wrapAngle(b - a) * t;
}

function clampHeight(h: number | undefined): number {
  return typeof h === 'number' && Number.isFinite(h) ? Math.min(1.15, Math.max(0.85, h)) : 1;
}

/** Build (appearance `build`, slim 0.85 .. broad 1.2; missing in older saves: 1). */
function clampBuild(b: number | undefined): number {
  return typeof b === 'number' && Number.isFinite(b) ? Math.min(1.2, Math.max(0.85, b)) : 1;
}
