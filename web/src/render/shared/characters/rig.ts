// `compose` and `rotationOf` are ported from Babylon.js 9.30 (`Matrix.ComposeToRef`, `Matrix.decompose`,
// `Quaternion.FromRotationMatrixToRef`). Changed for Idyll Lives: on the rig's own records.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
/**
 * A resident's animation state (`SimRig`, one per resident row) and its clip players, with the small helpers the
 * residents' code shares (`residents.ts`).
 */

import { Pose } from '../../../core/snapshot';
import type { VisemeTrack } from '../../../voice/visemes';
import type { StepIn } from './actions';
import type { BodyData, Clip } from './data';
import type { StageDirection } from './expressions';
import type { Face } from './face';
import type { OutfitParams, Wear } from './outfit';
import type { Placement } from './pose';
import { type Bed, LIE_BACK, LIE_CENTRE, type PlacedObject } from './seats';
import { EMBRACE } from './socials';

/** Rows in the pose texture (= Sim capacity). */
export const MAX_CHARACTERS = 64;
/** How far the feet reach past the slot centre for a lying Sim, and their back's height above the feet line: babies (much smaller) lie centred in the crib, on the mattress. */
export function lieCentre(rig: SimRig): number {
  // (A smaller body than its family's base body, a child's, lies with its head where a grown-up's would be, on the pillow.)
  return (rig.body.stage === 'baby' ? rig.body.height * 0.5 : LIE_CENTRE - (rig.body.height / rig.size) * (1 - rig.size)) * rig.scale;
}
export function lieBack(rig: SimRig): number {
  return (rig.body.stage === 'baby' ? LIE_BACK * 0.45 : LIE_BACK * rig.size) * rig.scale;
}
/** How far in front of its middle a body's chest stands out (m, `EMBRACE`). */
export function chestDepth(rig: SimRig): number {
  return EMBRACE.chest * rig.body.height * rig.scale * rig.build;
}
/** How far (m) each of two stepping in to hug or kiss steps (`EMBRACE`), from the distance between them: until their chests meet (`gap` m apart). */
export function embraceStep(rig: SimRig, partner: SimRig | null, gap: number): number {
  if (!partner) return 0;
  const d = Math.hypot(partner.x - rig.x, partner.z - rig.z);
  return clamp((d - chestDepth(rig) - chestDepth(partner) - gap) / 2, 0, EMBRACE.most);
}
/** How far (m) the other's shoulders are below a hugger's (0 when level or higher, `EMBRACE`). */
export function shoulderDrop(rig: SimRig, partner: SimRig | null): number {
  if (!partner) return 0;
  return Math.max(0, (rig.body.height * rig.scale - partner.body.height * partner.scale) * EMBRACE.shoulder);
}
export interface Player {
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
  /** Placed in this frame (world: a point on the floor and a facing) instead of at the Sim: the bunk's ladder (`BUNK_LADDER`). */
  frame: Frame | null;
}
/** A place on the floor and a facing (world, the Sim's convention: `yaw` 0 faces +z). */
export interface Frame {
  x: number;
  z: number;
  yaw: number;
}
export function player(): Player {
  return {
    clip: null,
    key: '',
    time: 0,
    rate: 1,
    repeat: false,
    rootY: 0,
    rootFwd: 0,
    rootPitch: 0,
    frame: null,
  };
}
export function copyPlayer(from: Player, to: Player): void {
  to.clip = from.clip;
  to.key = from.key;
  to.time = from.time;
  to.rate = from.rate;
  to.repeat = from.repeat;
  to.rootY = from.rootY;
  to.rootFwd = from.rootFwd;
  to.rootPitch = from.rootPitch;
  to.frame = from.frame;
}
export interface SimRig {
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
  /** Sat against a tree in its shade (`Seat.shade`): the way they face, chosen as they sit down (NaN: not yet). */
  shadeYaw: number;
  /** Sitting (or last sat) on the floor (`floorSit`: `Floor`, `Ground`; '' on a seat): its own clips. */
  floorSat: string;
  /**
   * Seated, the legs' turns that put the feet on the floor (`seatLegs`, `benchLegs`: each thigh's, calf's and foot's
   * about x and the thigh's out to the side, left then right), faded out standing up.
   */
  seatTurn: Float32Array;
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
  /** Extra forward step towards a partner (hugs, kisses); slid forward in a seat to put the feet up (`feet`). */
  approach: number;
  /**
   * Feet up on a footstool this frame (`footstoolFor`, `feetUp`): whether, and where the ankles go (m: height,
   * forward of and to the right of the Sim's place) and how far forward the Sim slides in the seat for it.
   */
  feetUp: boolean;
  feet: Float32Array;
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
  /**
   * Carrying a baby (`CARRY`): seconds of lifting them out (-1: not carrying) and of putting them in
   * (-1: not yet), at `carryRate`.
   */
  carryT: number;
  putT: number;
  carryRate: number;
  /** Changing the baby (`CHANGE`): as the cuddle's `cuddleT`, `cuddleOut`, `cuddleMin`, `cuddleRate`. */
  changeT: number;
  changeOut: number;
  changeMin: number;
  changeRate: number;
  /** Where they turn to while lifting a baby out of the crib (NaN: as the sim faces them). */
  faceX: number;
  faceZ: number;
  /** Its clothes for each `Wear` (`wearFor`; null: a ghost), and the ones written to its row (null: none yet). */
  outfits: Record<Wear, OutfitParams> | null;
  worn: OutfitParams | null;
  /** Finger openness (see HANDS), smoothed. */
  hands: number;
  /** Seconds in the current step of a use sequence. */
  stepTime: number;
  /** Emotion code (index + 1 into the content's emotions; 0 = none). */
  emotion: number;
  /** The social they're in (content `socials` id; '' when none or unknown), for its gestures (`GESTURES`). */
  socialId: string;
  /** Bed while lying (for the blanket) and how far the blanket is pulled up (0..1). */
  bed: PlacedObject | null;
  bedH: number;
  /** How far forward of the slot it lies (`BEDS` `fwd`). */
  bedFwd: number;
  /** Lying (or last lain) in the upper bunk (`BUNKS`): up and down its ladder (`BUNK_LADDER`). */
  upperBunk: boolean;
  /** Where the newest snapshot has them (the drawn place, `x` and `z`, eases there): which bunk they are given. */
  tx: number;
  tz: number;
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
  /** The way they lay (kept while lying), for turning out of bed to the side sim-core turned them (`outOfBed`). */
  lieYaw: number;
  /**
   * Getting into bed from its side (sim-core brings them to the tile beside it, content `approach: side`):
   * the way they sit on its edge first, facing out (NaN: from its foot, as `LIE_ENTRY`), and how far
   * out from the slot the hips sit (m).
   */
  sideYaw: number;
  sideOut: number;
  /** Where they stood just before sitting or lying down (world), to tell the side they came from. */
  fromX: number;
  fromZ: number;
  /**
   * Getting up from a seat: sim-core puts the Sim on the tile they came from at once; they rise where
   * the seat is (`riseX`, `riseZ`: the seated root as last drawn) and step to the tile while rising
   * (`rising`).
   */
  rising: boolean;
  riseX: number;
  riseZ: number;
  /** Rising off a built-in chair to a tile behind it: the corner beside the chair the step goes round (`cornerOf`; NaN: straight). */
  riseViaX: number;
  riseViaZ: number;
  /**
   * Walking in (`WALK_IN`): sim-core moves a standing resident onto a seat, into a shower or up to the place it
   * stands them at in one step; drawn, they walk there from where they were (`walkX`, `walkZ`; round the corner
   * `viaX`, `viaZ` beside a built-in chair, NaN without), after `walkWait` s standing (a shower's door opening),
   * then turn to the way sim-core faces them (`walkYaw`). `walkT`: the seconds since it began (-1: not walking);
   * `walkS`: the metres walked of the way, `walkL` its length when last drawn, `walkEX`, `walkEZ` its end and
   * `walkPX`, `walkPZ` where they were last drawn; `walkTurn`: the seconds turned there; `walkDown`: sitting down
   * still to come once there.
   */
  walkT: number;
  walkS: number;
  walkL: number;
  walkX: number;
  walkZ: number;
  viaX: number;
  viaZ: number;
  walkEX: number;
  walkEZ: number;
  walkPX: number;
  walkPZ: number;
  walkWait: number;
  walkYaw: number;
  walkTurn: number;
  walkDown: boolean;
  /**
   * Out of the shower in the towel (`wearFor`): the seconds since the use in it ended (-1: not), the towel kept while
   * they step out (`walkT`) and `TOWEL_OUT` s more without a step.
   */
  toweled: number;
  /** The shower they were last under (`toweled`), for its door as they step out (`startWalk`). */
  shower: PlacedObject | null;
  sitX: number;
  sitZ: number;
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
  /** Its lane (`lanes`), the way they face it (`stepFacing`), and the step in's sideways part to it (world). */
  inLane: number;
  inFacing: number;
  inDX: number;
  inDZ: number;
  outT: number;
  /** Seconds after the use before stepping out (the entry's `out`). */
  outWait: number;
  /** Over a rim on the way (`StepIn.over`): where it is (a share of the way out), how high, and the way's length (m); [0, 0, 1]: none. */
  inOver: Float32Array;
  /** Pulled up to a table (`TUCK`): how far the chair is in now (m), and which seat (-1: none). */
  tuck: number;
  tuckSeat: number;
  /** The top still further off than the desk-hands clip reaches, pulled in as far as it goes (`sitHandsFar`). */
  tuckFar: boolean;
  /** The top lower over the seat than the desk-hands clip's hands (`LOW_TOP`, `sitHandsLow`): a kids' chair at a coffee table. */
  tuckLow: boolean;
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
export function advance(p: Player, dt: number): void {
  if (!p.clip) return;
  p.time += dt * p.rate;
  if (p.clip.loop && p.time > p.clip.duration * 64) p.time %= p.clip.duration;
}
/** Head-top marker (speech bubbles, the selection marker, the creator's framing) above the crown (m, body scale). */
export const HEAD_MARGIN = 0.04;
/** The eyes' height below a Sim's head-top marker (m). */
export function eyesBelowTop(rig: SimRig): number {
  const b = rig.body;
  return ((b.eyeY !== null ? b.height - b.eyeY : b.headAbove * 0.7) + HEAD_MARGIN) * rig.scale * rig.head;
}
/** Mouth shapes for talk without a voice line (viseme indices: aa, E, O, I, U, PP, DD, SS, nn). */
export const BABBLE = [10, 11, 13, 12, 14, 1, 4, 7, 8, 10, 11];
/** Whose turn it is in a conversation: the starter (role 1) and the other Sim alternate; those listening in (role 3) only listen. */
export function speaking(pair: number, role: number, time: number): boolean {
  if (role === 0 || role === 3) return false;
  const turn = Math.floor((time + (pair % 5) * 0.53) / 2.8);
  return (turn % 2 === 0) === (role === 1);
}
export function splitKey(key: string): [string, string] {
  const at = key.indexOf(':');
  return [key.slice(0, at), key.slice(at + 1)];
}
export function hash(n: number): number {
  let h = Math.imul((n | 0) ^ 0x9e3779b9, 0x85ebca6b);
  h = Math.imul(h ^ (h >>> 13), 0xc2b2ae35);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}
export function clamp(v: number, lo: number, hi: number): number {
  return v < lo ? lo : v > hi ? hi : v;
}
/** (x, y, z) made 1 long, into `UNIT`. */
export function unit(x: number, y: number, z: number): Float32Array {
  const l = Math.hypot(x, y, z) || 1;
  UNIT[0] = x / l;
  UNIT[1] = y / l;
  UNIT[2] = z / l;
  return UNIT;
}
export const UNIT = new Float32Array(3);
/** Sets a frame (`Characters.placeFrame`): its x, y and z axes, then its origin. */
export function setFrame(f: Float32Array, xx: number, xy: number, xz: number, yx: number, yy: number, yz: number, zx: number, zy: number, zz: number, ox: number, oy: number, oz: number): void {
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
export function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}
/** Getting into bed: sitting down on its edge (until the crossfade to lying), as `choose` plays it. */
export function gettingIn(rig: SimRig): boolean {
  return (rig.lastPose === Pose.Stand && rig.cur.key !== 'sitDown') || (rig.cur.key === 'sitDown' && rig.cur.time < rig.cur.clip!.duration - 0.35);
}
/** 0 before `from`, easing to 1 over `length` after it. */
export function ramp(t: number, from: number, length: number): number {
  return smooth(clamp((t - from) / length, 0, 1));
}
export function smooth(t: number): number {
  return t * t * (3 - 2 * t);
}
export function wrapAngle(a: number): number {
  a %= Math.PI * 2;
  if (a > Math.PI) a -= Math.PI * 2;
  if (a < -Math.PI) a += Math.PI * 2;
  return a;
}
export function lerpAngle(a: number, b: number, t: number): number {
  return a + wrapAngle(b - a) * t;
}
export function clampHeight(h: number | undefined): number {
  return typeof h === 'number' && Number.isFinite(h) ? Math.min(1.15, Math.max(0.85, h)) : 1;
}
/** Build (appearance `build`, slim 0.85 .. broad 1.2; missing in older saves: 1). */
export function clampBuild(b: number | undefined): number {
  return typeof b === 'number' && Number.isFinite(b) ? Math.min(1.2, Math.max(0.85, b)) : 1;
}
/** Puts `pl` `k` (0..1) of the way from (`ax`, `az`) to (`cx`, `cz`), round the corner (`bx`, `bz`) on the way (NaN: straight). */
export function alongWay(pl: Placement, ax: number, az: number, bx: number, bz: number, cx: number, cz: number, k: number): void {
  if (Number.isNaN(bx)) {
    pl.x = lerp(ax, cx, k);
    pl.z = lerp(az, cz, k);
    return;
  }
  const a = Math.hypot(bx - ax, bz - az);
  const b = Math.hypot(cx - bx, cz - bz);
  const s = k * (a + b);
  if (s <= a) {
    const u = a > 0 ? s / a : 1;
    pl.x = lerp(ax, bx, u);
    pl.z = lerp(az, bz, u);
  } else {
    const v = b > 0 ? (s - a) / b : 1;
    pl.x = lerp(bx, cx, v);
    pl.z = lerp(bz, cz, v);
  }
}

// The ported matrix maths as the residents' code uses it, number for number (the same float32 matrices, the same
// order of operations), so residents stand where they did up to 0.3.2: a placement carried by a ride
// (`Characters.riding`) and a bone's height at rest (`bindY`).

/** The original's `Matrix.ComposeToRef` of scale (`sx`, `sy`, `sz`), rotation (`qx` .. `qw`) and translation into `out`. */
export function compose(sx: number, sy: number, sz: number, x: number, y: number, z: number, w: number, tx: number, ty: number, tz: number, out: Float32Array): void {
  const x2 = x + x;
  const y2 = y + y;
  const z2 = z + z;
  const xx = x * x2;
  const xy = x * y2;
  const xz = x * z2;
  const yy = y * y2;
  const yz = y * z2;
  const zz = z * z2;
  const wx = w * x2;
  const wy = w * y2;
  const wz = w * z2;
  out[0] = (1 - (yy + zz)) * sx;
  out[1] = (xy + wz) * sx;
  out[2] = (xz - wy) * sx;
  out[3] = 0;
  out[4] = (xy - wz) * sy;
  out[5] = (1 - (xx + zz)) * sy;
  out[6] = (yz + wx) * sy;
  out[7] = 0;
  out[8] = (xz + wy) * sz;
  out[9] = (yz - wx) * sz;
  out[10] = (1 - (xx + yy)) * sz;
  out[11] = 0;
  out[12] = tx;
  out[13] = ty;
  out[14] = tz;
  out[15] = 1;
}

/** The original's `a.multiplyToRef(b, out)` (`a` applied first, then `b`; `out` may be `a` or `b`). */
export function multiply(a: ArrayLike<number>, b: ArrayLike<number>, out: Float32Array): void {
  const a0 = a[0];
  const a1 = a[1];
  const a2 = a[2];
  const a3 = a[3];
  const a4 = a[4];
  const a5 = a[5];
  const a6 = a[6];
  const a7 = a[7];
  const a8 = a[8];
  const a9 = a[9];
  const a10 = a[10];
  const a11 = a[11];
  const a12 = a[12];
  const a13 = a[13];
  const a14 = a[14];
  const a15 = a[15];
  const b0 = b[0];
  const b1 = b[1];
  const b2 = b[2];
  const b3 = b[3];
  const b4 = b[4];
  const b5 = b[5];
  const b6 = b[6];
  const b7 = b[7];
  const b8 = b[8];
  const b9 = b[9];
  const b10 = b[10];
  const b11 = b[11];
  const b12 = b[12];
  const b13 = b[13];
  const b14 = b[14];
  const b15 = b[15];
  out[0] = a0 * b0 + a1 * b4 + a2 * b8 + a3 * b12;
  out[1] = a0 * b1 + a1 * b5 + a2 * b9 + a3 * b13;
  out[2] = a0 * b2 + a1 * b6 + a2 * b10 + a3 * b14;
  out[3] = a0 * b3 + a1 * b7 + a2 * b11 + a3 * b15;
  out[4] = a4 * b0 + a5 * b4 + a6 * b8 + a7 * b12;
  out[5] = a4 * b1 + a5 * b5 + a6 * b9 + a7 * b13;
  out[6] = a4 * b2 + a5 * b6 + a6 * b10 + a7 * b14;
  out[7] = a4 * b3 + a5 * b7 + a6 * b11 + a7 * b15;
  out[8] = a8 * b0 + a9 * b4 + a10 * b8 + a11 * b12;
  out[9] = a8 * b1 + a9 * b5 + a10 * b9 + a11 * b13;
  out[10] = a8 * b2 + a9 * b6 + a10 * b10 + a11 * b14;
  out[11] = a8 * b3 + a9 * b7 + a10 * b11 + a11 * b15;
  out[12] = a12 * b0 + a13 * b4 + a14 * b8 + a15 * b12;
  out[13] = a12 * b1 + a13 * b5 + a14 * b9 + a15 * b13;
  out[14] = a12 * b2 + a13 * b6 + a14 * b10 + a15 * b14;
  out[15] = a12 * b3 + a13 * b7 + a14 * b11 + a15 * b15;
}

/** The 4x4 cofactors of the first row (the original's `determinant`, `InvertMatrixToArray`), into `COFACTORS`. */
function cofactors(m: ArrayLike<number>): void {
  const m10 = m[4];
  const m11 = m[5];
  const m12 = m[6];
  const m13 = m[7];
  const m20 = m[8];
  const m21 = m[9];
  const m22 = m[10];
  const m23 = m[11];
  const m30 = m[12];
  const m31 = m[13];
  const m32 = m[14];
  const m33 = m[15];
  const det_22_33 = m22 * m33 - m32 * m23;
  const det_21_33 = m21 * m33 - m31 * m23;
  const det_21_32 = m21 * m32 - m31 * m22;
  const det_20_33 = m20 * m33 - m30 * m23;
  const det_20_32 = m20 * m32 - m22 * m30;
  const det_20_31 = m20 * m31 - m30 * m21;
  COFACTORS[0] = +(m11 * det_22_33 - m12 * det_21_33 + m13 * det_21_32);
  COFACTORS[1] = -(m10 * det_22_33 - m12 * det_20_33 + m13 * det_20_32);
  COFACTORS[2] = +(m10 * det_21_33 - m11 * det_20_33 + m13 * det_20_31);
  COFACTORS[3] = -(m10 * det_21_32 - m11 * det_20_32 + m12 * det_20_31);
}
const COFACTORS = new Float64Array(4);

/** The rotation of `m` (the original's `Matrix.decompose` with no scaling node: the scale taken out, `y` negated when mirrored) into `out`'s `qx` .. `qw`. */
export function rotationOf(m: Float32Array, out: { qx: number; qy: number; qz: number; qw: number }): void {
  let sx = Math.sqrt(m[0] * m[0] + m[1] * m[1] + m[2] * m[2]);
  let sy = Math.sqrt(m[4] * m[4] + m[5] * m[5] + m[6] * m[6]);
  let sz = Math.sqrt(m[8] * m[8] + m[9] * m[9] + m[10] * m[10]);
  cofactors(m);
  if (m[0] * COFACTORS[0] + m[1] * COFACTORS[1] + m[2] * COFACTORS[2] + m[3] * COFACTORS[3] <= 0) sy *= -1;
  if (sx === 0 || sy === 0 || sz === 0) {
    out.qx = 0;
    out.qy = 0;
    out.qz = 0;
    out.qw = 1;
    return;
  }
  sx = 1 / sx;
  sy = 1 / sy;
  sz = 1 / sz;
  // (Through a float32 matrix, as the original's `MathTmp.Matrix[0]`.)
  const r = ROTATION;
  r[0] = m[0] * sx;
  r[1] = m[1] * sx;
  r[2] = m[2] * sx;
  r[4] = m[4] * sy;
  r[5] = m[5] * sy;
  r[6] = m[6] * sy;
  r[8] = m[8] * sz;
  r[9] = m[9] * sz;
  r[10] = m[10] * sz;
  // The rotation matrix's quaternion (the original's `Quaternion.FromRotationMatrixToRef`).
  const m11 = r[0];
  const m12 = r[4];
  const m13 = r[8];
  const m21 = r[1];
  const m22 = r[5];
  const m23 = r[9];
  const m31 = r[2];
  const m32 = r[6];
  const m33 = r[10];
  const trace = m11 + m22 + m33;
  let s: number;
  if (trace > 0) {
    s = 0.5 / Math.sqrt(trace + 1.0);
    out.qw = 0.25 / s;
    out.qx = (m32 - m23) * s;
    out.qy = (m13 - m31) * s;
    out.qz = (m21 - m12) * s;
  } else if (m11 > m22 && m11 > m33) {
    s = 2.0 * Math.sqrt(1.0 + m11 - m22 - m33);
    out.qw = (m32 - m23) / s;
    out.qx = 0.25 * s;
    out.qy = (m12 + m21) / s;
    out.qz = (m13 + m31) / s;
  } else if (m22 > m33) {
    s = 2.0 * Math.sqrt(1.0 + m22 - m11 - m33);
    out.qw = (m13 - m31) / s;
    out.qx = (m12 + m21) / s;
    out.qy = 0.25 * s;
    out.qz = (m23 + m32) / s;
  } else {
    s = 2.0 * Math.sqrt(1.0 + m33 - m11 - m22);
    out.qw = (m21 - m12) / s;
    out.qx = (m13 + m31) / s;
    out.qy = (m23 + m32) / s;
    out.qz = 0.25 * s;
  }
}
const ROTATION = new Float32Array(16);

/**
 * The height of bone `b`'s joint at rest, from a body's inverse bind matrices (`m`, 16 floats a bone): the inverse's
 * translation y, as the original's `Matrix.FromArray(m, b * 16).invert().m[13]` (float32).
 */
export function bindY(m: ArrayLike<number>, b: number): number {
  const o = b * 16;
  const a = BIND;
  for (let k = 0; k < 16; k++) a[k] = m[o + k];
  cofactors(a);
  const det = a[0] * COFACTORS[0] + a[1] * COFACTORS[1] + a[2] * COFACTORS[2] + a[3] * COFACTORS[3];
  // (Not invertible: the matrix is kept as it was, as the original's `invert` does.)
  if (det === 0) return a[13];
  return Math.fround(bindCofactor13(a) * (1 / det));
}
const BIND = new Float32Array(16);

/** The original's `cofact_13` of `InvertMatrixToArray`: `+(m00 * det_21_32 - m01 * det_20_32 + m02 * det_20_31)`. */
function bindCofactor13(m: ArrayLike<number>): number {
  const m20 = m[8];
  const m21 = m[9];
  const m22 = m[10];
  const m30 = m[12];
  const m31 = m[13];
  const m32 = m[14];
  const det_21_32 = m21 * m32 - m31 * m22;
  const det_20_32 = m20 * m32 - m22 * m30;
  const det_20_31 = m20 * m31 - m30 * m21;
  return +(m[0] * det_21_32 - m[1] * det_20_32 + m[2] * det_20_31);
}
