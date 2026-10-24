/**
 * What residents sit and lie on (`residents.ts`): seats and beds (`furniture.json`, read by `cargo check-glb` and
 * `cargo review` too; what their entries mean and why some are as they are: docs/characters-format.md "Seats and
 * beds"), footstools and footrests, bunk beds, hammocks and quilts, chairs pulled up to tables, and the objects
 * placed in the world as the residents' code sees them (`PlacedObject`).
 */

import type { ObjectDef } from '../../../content/content';
import { FLOOR_SITS } from './clips/seated';
import FURNITURE from './furniture.json';
import type { Wear } from './outfit';
import type { LegSide } from './seatFit';

/**
 * Seat surface height (m) and forward offset of the hips from the object's centre, per object def;
 * `back`: the sitter faces the object's back (a dressing table's mirror against the wall), `fwd` still
 * towards its front; `recline`: leant back against a slanted back (rad, `Seat.recline`); `along`, `legsOut`:
 * as `Seat`'s.
 */
export const SEATS = FURNITURE.seats as Record<string, Seat>;
export const DEFAULT_SEAT: Seat = FURNITURE.defaultSeat;
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
export const TUCK = { edge: 0.24, far: 0.3, knees: 0.56, kneeUp: 0.11, toes: 0.62, toeUp: 0.12, clear: 0.02, max: 0.45, rise: 0.45, rate: 3, back: 4 };
/** The height (m) the seated clips are made for (their reaches scale with the body). */
export const CLIP_HEIGHT = 1.77;
/**
 * Where a Sim sits on an object (`PlacedObject.seat`): the table above, else the content's `seat`
 * (with how they sit: facing the object's back, legs out along it, pedalling), else the default.
 */
export function seatOf(def: ObjectDef | undefined, id: string): Seat {
  const own = def?.seat;
  if (SEATS[id] || !own) return SEATS[id] ?? DEFAULT_SEAT;
  const pedals = own.pedals ? { h: own.pedals.height, fwd: own.pedals.forward ?? 0, crank: own.pedals.crank } : undefined;
  const recline = own.recline !== undefined ? (own.recline * Math.PI) / 180 : undefined;
  return { h: own.height, fwd: own.forward ?? 0, back: own.facing === 'back', legsOut: own.legs === 'out', pedals, recline };
}
/** A seat (see `seatOf`): its height and the hips' forward offset (m, the object's frame), and how they sit. */
export interface Seat {
  h: number;
  fwd: number;
  /** Facing the object's back (a dressing table's mirror against the wall, a pianist at an upright); `fwd` is still towards its front. */
  back?: boolean;
  /** Legs stretched out along the seat (a chaise). */
  legsOut?: boolean;
  /** With `legsOut`: the room (m) ahead of the hips for the legs (a short tub): the heels stop there, the knees drawn up. */
  room?: number;
  /**
   * Facing along the object, a quarter turn from its front: towards its right (1) or its left (-1), `fwd`
   * then being along that way (a bath, sat in lengthways).
   */
  along?: 1 | -1;
  /** How far back (rad, from sitting up) the back leans just sitting, relaxing, napping or watching: against a slanted back (an Adirondack chair). Default: upright sitting, a little relaxing (`activityPose`). */
  recline?: number;
  /** Pedals the feet turn (an exercise bike): the crank's axle (height, forward of the middle) and its length (m). */
  pedals?: { h: number; fwd: number; crank: number };
  /**
   * Where its top ends in front (m forward of the footprint's middle, as `fwd`; `cargo check-glb` measures it): on a
   * deep seat the hips move forward until the knees clear it (`perch`), not the shins down inside the cushion.
   */
  front?: number;
  /**
   * Sat against a tree's trunk on the ground, in its shade: facing out from it on whichever of its front and its
   * sides is in the crown's shadow then (`shadeSide`), `fwd` from the trunk's middle.
   */
  shade?: boolean;
  /**
   * A footrest the feet go on where the floor is out of their reach (a bar stool's ring): `[h, fwd]`, its top's height
   * (m) and how far forward of the footprint's middle its front is (m); the balls of the feet on it (`seatLegs`).
   */
  rest?: readonly number[];
}
/**
 * Mattress / bench top height (m) for lying, and how far forward of the footprint's middle the
 * sleepers lie (`fwd`, m): a sofa bed's mattress unfolds out in front of the sofa. `across`: lain on
 * along its width, head to its left, in its middle (a hammock slung side to side on a wide footprint).
 */
export const BEDS: Record<string, Bed> = FURNITURE.beds;
export const DEFAULT_BED: Bed = FURNITURE.defaultBed;
/** What residents lie on (`BEDS`). */
export interface Bed {
  h: number;
  fwd?: number;
  /** How far short of the footprint's foot end the quilt ends (m, default 0.05): a bed whose mattress stops short of it. */
  foot?: number;
  /**
   * A bed with sides round the mattress (a back, arms, a rail: the daybed): the quilt is tucked in at the mattress's
   * edges, `[width, right]`: its width over the sleeper (m, the side drops then inside the mattress) and how far (m)
   * to the bed's right of the footprint's middle the mattress's open middle is (clear of the back's cushions).
   */
  tuck?: readonly number[];
  across?: boolean;
  /** A bench lifted on (the weight bench): the feet down on the floor either side of it, this far (m) out from its middle (`benchLegs`). */
  feet?: number;
  /**
   * A hammock's sling, along the sleeper from its middle (m): its top's height there (`low`) and at its
   * ends (`end`, `half` each way, at the spreader bars; a parabola between), and how far towards the head
   * end the pillow is. The sleeper lies in it (`inSling`): hips in the dip, head raised onto the
   * pillow, knees up and the feet inside the spreader bar.
   */
  sling?: { low: number; end: number; half: number; head: number };
}
/** Bunk beds (content `bunk`): mattress tops of the lower and the upper bunk, `[lower, upper]`. */
export const BUNKS: Record<string, readonly number[]> = FURNITURE.bunks;
/**
 * Footstools: the feet go up on them from a seat in front (`Characters.footstoolFor`). Each its top's width
 * and depth (m, its own frame, as made); the top is its seat (`Seat.h`).
 */
export const FOOTSTOOLS: Record<string, readonly number[]> = FURNITURE.footstools;
/** On a footrest (`Seat.rest`): the ankle this far (m, for the reference height) behind its front, the ball of the foot on it. */
export const REST_BACK = 0.09;
/**
 * Feet up on a footstool (`Characters.feetUp`): the heels `heel` m in from its near edge (at most to its middle),
 * else as far in as the legs reach, at least `edge` m; the ankles `ankle` over its top (body scale 1), at most
 * `side` m to one side of the hips. The legs reach `reach` of their length, no nearer than `near` of it (the
 * knees not drawn right up), slid forward in the seat up to `slide` m for it, slouching back `slouch` rad a metre
 * slid. Not for what is done with the hands or the legs on a seat (`NO_FEET_UP`).
 */
export const FEET_UP = { heel: 0.14, edge: 0.03, ankle: 0.075, side: 0.3, reach: 0.995, near: 0.6, slide: 0.18, slouch: 1.4 };
export const NO_FEET_UP = new Set(['music', 'meditate', 'toilet', 'bath', 'wash']);
/** Beds whose sleepers get a blanket (pack hammocks, sofas and nap pods don't). */
export const BLANKET_BEDS = new Set(['bed', 'lazy.snoozecloudBed', 'bedSingle', 'bedBunk', 'bedFuton', 'bedKing', 'bedDouble', 'bedQueenUpholstered', 'bedCanopy', 'kidsBed', 'kidsBedCar', 'wallBed', 'bedDaybed']);
/**
 * The part of a bed whose colour its sleepers' quilts take (`PlacedObject.quilt`): the one the player
 * paints as its "Bedding", else this (the futon's own quilt, a throw).
 */
export const QUILT_PART: Record<string, string> = { bedFuton: 'quilt_oat', wallBed: 'throw', 'lazy.snoozecloudBed': 'throw' };
/**
 * The thighs' turn forward from the hips (rad) in `sit` (clips/seated.ts), that the shells' seated corrective is
 * made for (crates/tools/src/characters/clothes.rs `SIT_FLEX`, 76°): `writeSit` weighs it by the thighs' own.
 */
export const SIT_FLEX = (76 * Math.PI) / 180;
/** The thighs' turn (rad, their mean) below which the shells keep their standing shape (`writeSit`). */
export const SIT_FROM = (15 * Math.PI) / 180;
/** A quilt's colour until its bed's file says (sRGB). */
export const QUILT_COLOUR = '#E7E0D4';
/**
 * What is worn in the water (`wearFor`) where it isn't a towel (every shower, bath and tub is): swimwear in
 * the hot tubs and the cold plunge; and a towel where the use isn't a bath or a shower (the sauna).
 */
export const BATH_WEAR: Record<string, Wear> = { hotTubCedar: 'swim', whirlpoolTub: 'swim', 'hotHeaded.coldPlunge': 'swim', 'loner.soloSauna': 'towel' };
/**
 * Two sleepers' quilts in a wide bed (`writeBlanket`): each this much (m) wider on each side than half the bed
 * and moved this much towards its middle, so they overlap by four times it over the middle.
 */
export const BLANKET_MEET = 0.06;
/**
 * Lying: height of the back above the feet line (body scale 1, for a grown-up's body: a smaller body's,
 * a child's, is thinner by its `size`), and how far the feet reach past the slot centre.
 */
export const LIE_BACK = 0.1;
export const LIE_CENTRE = 0.86;
/**
 * In a hammock (`Bed.sling`): the pillow's thickness (m), the ankles' height over the sling (m, body
 * scale 1), how far inside the spreader bar they stay (m), the most the upper body rises (rad), how far
 * the back lies above the sling's top (m, its thickness), and how far the head's joint lies short of the
 * pillow's middle (m: the back of the head on it).
 */
export const SLING = { pillow: 0.07, ankle: 0.07, feet: 0.12, lift: 0.7, back: 0.02, crown: 0.08 };
/** A sling's top `d` m from its middle along the sleeper (a parabola from `low` to `end` at the bars). */
export function slingTop(sling: NonNullable<Bed['sling']>, d: number): number {
  return sling.low + (sling.end - sling.low) * Math.min(1, (d / sling.half) ** 2);
}
/** Getting into bed over its foot (its sides blocked): where the hips sit down, forward of the slot centre (body scale 1). */
export const LIE_ENTRY = 0.55;
/** Getting into bed from its side (`sideIn`): how far in from its edge the hips sit down (body scale 1). */
export const SIDE_IN = 0.2;
/** Beds too short to lie on flat (a window seat's 1.3 m between its pillows): dozed on curled up on the side (`lieCurl`, clips/self.ts). */
export const CURLED_BEDS = new Set(['gloomy.windowSeat']);
/**
 * Pedalling (`Characters.pedal`): a turn's length (s) when the object's loop isn't playing (the bike's
 * `pedals_turn`), and the ankle's place over the pedal's axle (m up and forward, body scale 1).
 */
export const PEDAL = { turn: 1.2, ankle: [0.08, -0.09] as const };
/** On the weight bench (`Characters.benchLegs`): the knee this share of the thigh's length on from the hip, towards the feet. */
export const BENCH_KNEE = 0.92;
/** The hips' joints this far (m, the reference body) out from the middle. */
export const BENCH_HIP = 0.09;
/** Legs out along a seat (`Characters.legsOut`): the ankles this far below the hips' joints (m, body scale 1). */
export const LEGS_OUT_DROP = 0.04;
/** How a held part is held (content `held.grip`; see `Characters.writeHeld`). */
export type Grip = 'bar' | 'pair' | 'guitar' | 'pole';
/** Where the legs are in a clip (`measureLegs`; body space, scale 1; `LegSide`, characters/seatFit.ts). */
export interface LegFit {
  l: LegSide;
  r: LegSide;
  /** The ankle joints' height standing (the `idle` clip's), for feet put on the floor (`seatLegs`). */
  ankle: number;
  /** The spine's base (spine_01) and the head's joint: height and depth. */
  spine: readonly [number, number];
  head: readonly [number, number];
}
/** Sitting down and getting up (from a chair or the floor): no held prop drawn meanwhile. */
export const RISING = new Set(['sitDown', 'standUp', ...FLOOR_SITS.flatMap((f) => [`sitDown${f.name}`, `standUp${f.name}`])]);
/** Sitting on the floor (`floorSit`), and sitting down there. */
export const ON_FLOOR = new Set(FLOOR_SITS.flatMap((f) => [`sit${f.name}`, `sitDown${f.name}`]));
/**
 * How a seat `h` m over the floor under it is sat on: on a chair's clips (''), or on the floor by the floor clips
 * (clips/seated.ts `FLOOR_SITS`, each with its own sitting down and getting up): under 0.2 m cross-legged on a
 * floor cushion or a hearth's (`Floor`), under 0.07 m on the ground against a tree's trunk (`Ground`).
 */
export function floorSit(h: number): string {
  return h < 0.07 ? 'Ground' : h < 0.2 ? 'Floor' : '';
}
/** The clips of each way of sitting (`floorSit`): sitting, sitting down and getting up. */
export const SIT_CLIPS: Record<string, { sit: string; down: string; up: string }> = Object.fromEntries(
  ['', ...FLOOR_SITS.map((f) => f.name)].map((n) => [n, { sit: `sit${n}`, down: `sitDown${n}`, up: `standUp${n}` }]),
);
/** Clips whose legs are placed by `legTo`: sitting (pedalling, legs out along a chaise) and lying (in a hammock). */
export const LEG_CLIPS = ['sit', 'sitHands', 'lie'] as const;
/** Pelvis offset of the seated clip for one body (scale 1). */
export interface SeatFit {
  y: number;
  z: number;
}
export interface PlacedObject {
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
  /** A top sat at by a use of its own (a dining set, a desk with its chair built in: `seatsItself`): its chairs don't move (`cornerOf`). */
  builtIn: boolean;
  /**
   * A bed's bedding (`QUILT_PART`), whose colour its sleepers' quilts take: the colour the player gave
   * it, else `key` (`<model>#<part>`) for the file's own (`beddingColours`); null for anything else.
   */
  quilt: { key: string; colour?: string } | null;
  /** A footstool (`FOOTSTOOLS`): its top's half width and depth (m, resized with it) and its facing (rad); null for anything else. */
  footstool: { hw: number; hd: number; yaw: number } | null;
  cx: number;
  cz: number;
  minX: number;
  minZ: number;
  maxX: number;
  maxZ: number;
}
