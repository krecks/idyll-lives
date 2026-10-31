/**
 * Two clips of a resident on their own at a piece of furniture: brushing the hair at the dressing table's
 * mirror (`brushHair`, the seated upper body, characters.ts `ACTIONS.wash`), dozing curled up on the side
 * (`lieCurl`, characters.ts `CURLED_BEDS`: a seat too short to lie on flat, the window seat) and lying on the
 * side at full length (`lieSide`, `SIDE_BEDS`: a daybed). All are body-relative: one size.
 */

import { cycle, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { bump, stand, TAU } from './idle';
import { armIK, footIK, lerpV, pointOn, wrist } from './ik';

// --- brushing the hair --------------------------------------------------------------------------

/** A stroke of the brush (mimed, in the right fist) from the crown down the right side of the hair to below the ear: the palm's place from the head's joint (m, reference skeleton, the head's frame), a hand's width off the hair. */
const BRUSH: readonly Vec[] = [
  [0.03, 0.25, -0.02],
  [0.12, 0.2, -0.05],
  [0.16, 0.08, -0.07],
  [0.15, -0.04, -0.05],
];
/** Strokes a round and their share of it; the rest of the round the left hand smooths the left side, the head turned to the mirror. */
const STROKES = 4;
const BRUSHING = 0.7;
const BRUSH_S = 5;

/** Where along a stroke the brush is (0 at the crown, 1 at its end), the head's place. */
function strokeAt(k: number): Vec {
  const n = BRUSH.length - 1;
  const i = Math.min(n - 1, Math.floor(k * n));
  return lerpV(BRUSH[i], BRUSH[i + 1], k * n - i);
}

/**
 * Brushing the hair in the mirror: four strokes from the crown down the right side, the head tilting into
 * each, the brush lifted out and back up between them; then the left hand smooths the left side down and the
 * head turns to look at it in the mirror. Made standing; the runtime plays it as the seated upper body.
 */
function brushPose(p: number): PoseSpec {
  const brushing = p < BRUSHING;
  const k = brushing ? (p / BRUSHING) * STROKES : 0;
  const n = k % 1;
  // Down the hair in the first 70 % of a stroke, back up off it in the rest.
  const down = Math.min(1, n / 0.7);
  const off = n < 0.7 ? 0 : Math.sin(Math.PI * ((n - 0.7) / 0.3));
  const on = bump(p, 0.0, BRUSHING + 0.06, 0.06);
  const smooth = bump(p, BRUSHING + 0.02, 0.98, 0.08);
  const tilt = 9 * on * Math.sin(Math.PI * down) * (n < 0.7 ? 1 : 0.4);
  const body = stand({ breath: p, depth: 0.5, lean: 4, head: [4 + 4 * on, -6 * on + 14 * smooth, -tilt + 6 * smooth] });
  const head = (v: Vec): Vec => pointOn(body, 'Head', v);
  const at = strokeAt(n < 0.7 ? down : 1 - (n - 0.7) / 0.3);
  const brush = head([at[0] + 0.05 * off, at[1], at[2] + 0.03 * off]);
  const lap: Vec = [0.16, 1.02, 0.25];
  // The left hand: resting at the front, then smoothing the left side from the temple down to the shoulder.
  const s = (p - BRUSHING) / (1 - BRUSHING);
  const side = head([-0.14, 0.12 - 0.18 * Math.min(1, Math.max(0, s * 1.6 - 0.2)), -0.02]);
  const rest: Vec = [-0.14, 1.04, 0.24];
  return {
    ...body,
    ...armIK('r', lerpV(lap, brush, on), body, { palm: true, pole: [0.9, -0.3, -0.2], pronate: -20, hand: wrist('r', -10, 0) }),
    ...armIK('l', lerpV(rest, side, smooth), body, { palm: true, pole: [-0.9, -0.3, -0.2], pronate: -60 * smooth + 60 * (1 - smooth), hand: wrist('l', 10, 0) }),
  };
}

// --- curled up on the side ------------------------------------------------------------------------

/**
 * Lying curled up on the left side, facing the room (made standing like `lie`: the runtime turns it flat, the
 * front up; here the body is turned a quarter about its length, so the front faces the side, -x): the knees
 * drawn up, the back rounded, the head bowed onto the pillow, the left forearm under it and the right hand on
 * the cushion in front of the chest. The body is moved `CURL.along` towards the feet (it is shorter than lying
 * flat, about 1.1 m for a grown-up's: centred on where a flat one would lie), `CURL.back` m back (the knees in
 * on the cushion) and `CURL.up` off the mattress (the hip's side is under the pelvis' joint, not its back).
 */
const CURL = { along: -0.25, back: 0.12, up: 0.07, turn: -90 };

function curlPose(p: number): PoseSpec {
  const b = Math.sin(p * TAU * 2);
  const body: PoseSpec = {
    move: [CURL.back, CURL.along, CURL.up],
    pelvis: [-18, CURL.turn, 0],
    spine_01: [10, 0, 0],
    spine_02: [10 - 1.2 * b, 0, 0],
    spine_03: [8 - 0.8 * b, 0, 0],
    neck_01: [12, 0, -8],
    Head: [10 + 0.5 * b, -6, -8],
  };
  // The knees drawn up in front (-x once turned), the heels back under them, the soles to the back; the upper
  // (right) leg a little further up and forward. Places from the ankles' rest places (model space).
  const legs = {
    ...footIK('r', [CURL.back - 0.36, 0.17, 0.14], body, { foot: [80, CURL.turn, 0], pole: [-1, 0.2, 0.1] }),
    ...footIK('l', [CURL.back - 0.12, 0.16, -0.06], body, { foot: [80, CURL.turn, 0], pole: [-1, 0.1, -0.1] }),
  };
  const head = (v: Vec) => pointOn(body, 'Head', v);
  return {
    ...body,
    ...legs,
    // (The left hand under the cheek on the pillow, the right in front of the chest on the cushion: model space,
    // the cushion a little under the pelvis' joint, `CURL.up`.)
    ...armIK('l', head([-0.09, 0.02, 0.1]), body, { palm: true, pole: [-0.2, -1, -0.3], pronate: 0, hand: wrist('l', 20, 0) }),
    ...armIK('r', [CURL.back - 0.3, 1.02 + 0.003 * b, -0.03], body, { palm: true, pole: [0.2, 0.2, 1], pronate: 80, hand: wrist('r', 20, 0) }),
  };
}

/**
 * Lying on the right side at full length (a daybed lounged on; made standing like `lie`, the runtime turning it
 * flat): the body turned a quarter about its length so its front faces +x and its back -x (an object's back rail,
 * its front in front of the sleeper), the legs along the mattress, the right one straight and the left drawn up a
 * little over it, the right forearm under the cheek on the pillow and the left hand resting in front of the chest.
 * The pelvis is lifted `SIDE.up` off the mattress (its side is under the joint, not its back).
 */
const SIDE = { back: -0.05, up: 0.005, turn: 90 };

function sidePose(p: number): PoseSpec {
  const b = Math.sin(p * TAU * 2);
  const body: PoseSpec = {
    move: [SIDE.back, 0, SIDE.up],
    pelvis: [-4, SIDE.turn, 0],
    spine_01: [2, 0, 0],
    spine_02: [-1.4 * b, 0, 0],
    spine_03: [-1.0 * b, 0, 0],
    neck_01: [9, 4, 0],
    Head: [7 + 0.6 * b, 7, 4],
  };
  const head = (v: Vec) => pointOn(body, 'Head', v);
  return {
    ...body,
    ...footIK('r', [0.02, 0.005, -0.14], body, { foot: [30, 16, 0], pole: [1, 0, 0.2] }),
    ...footIK('l', [0.09, 0.07, -0.11], body, { foot: [30, -10, 0], pole: [1, 0, 0.4] }),
    // (The right hand under the cheek on the pillow, the left in front of the chest on the mattress.)
    ...armIK('r', head([0.09, 0.02, 0.1]), body, { palm: true, pole: [0.2, -1, -0.3], pronate: 0, hand: wrist('r', 20, 0) }),
    ...armIK('l', [0.2, 1.05, -0.07], body, { palm: true, pole: [-0.6, 0.2, 0.1], pronate: 80, hand: wrist('l', 20, 0) }),
  };
}

export function selfClips(): ClipDef[] {
  return [cycle('brushHair', { seconds: BRUSH_S, loop: true }, brushPose), cycle('lieCurl', { seconds: 2.4, loop: true }, curlPose), cycle('lieSide', { seconds: 2.4, loop: true }, sidePose)];
}
