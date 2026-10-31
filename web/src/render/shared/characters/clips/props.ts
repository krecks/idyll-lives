/**
 * Clips for what is done with a prop in the hand (G-70; the prop is drawn by `residents.ts` `writeHand`, actions.ts
 * `PROPS`): a cup held before the chest between sips (`holdCup`, standing and the seated upper layer; the sip is
 * `drink`, clips/seated.ts), a coin tossed into the fountain (`toss`), seed scattered from a bag on the lap for the birds
 * (`scatter`, the seated upper layer) and a cocktail shaken in both hands (`shake`).
 */

import { clip, cycle, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { bump, hangArms, stand, TAU } from './idle';
import { addV, armIK, lerpV, norm, wrist } from './ik';
import { CUP_HELD, eatPose, seatedBody } from './seated';

// --- a coin into the fountain ---------------------------------------------------------------------

/** The toss: seconds, and when the coin leaves the hand (share of the clip: actions.ts shows the coin until then). */
export const TOSS_S = 3.2;
export const TOSS_LET_GO = 0.63;

/** The coin in the palm looked at, held to the chest for the wish, then the arm back and an underarm toss over the rim. */
function tossPose(hand: Vec, flex: number, pronate: number, lean: number, head: Vec, t: number): PoseSpec {
  const body = stand({ shift: -0.3, breath: t, lean, head });
  return {
    ...body,
    ...hangArms(4, 7, 14),
    ...armIK('r', hand, body, { palm: true, pole: [0.5, -1, -0.2], pronate, hand: wrist('r', flex, 0) }),
  };
}

// --- seed for the birds -----------------------------------------------------------------------------

/** The bag held on the left palm over the lap (m, reference skeleton, seated), and where the right hand dips in. */
const BAG: Vec = [-0.07, 0.75, 0.22];
const DIP: Vec = [-0.05, 0.8, 0.22];
const SCATTER_S = 3;

function scatterPose(p: number): PoseSpec {
  const out = bump(p, 0.3, 0.95, 0.12);
  const fling = bump(p, 0.45, 0.8, 0.1);
  const body = seatedBody({ breath: p, lean: 6 + 3 * fling, twist: -6 * fling, head: [16 - 4 * fling, -8 * fling, 0] });
  const throwAt: Vec = [0.3, 0.93, 0.47];
  const r = lerpV(lerpV(DIP, [0.08, 0.86, 0.28], out), throwAt, fling);
  return {
    ...body,
    ...armIK('l', BAG, body, { palm: true, pole: [-0.4, -1, -0.3], pronate: -60, hand: wrist('l', -5, 0) }),
    ...armIK('r', r, body, { palm: true, pole: [0.5, -1, -0.3], pronate: -20 - 40 * fling, hand: wrist('r', 10 - 30 * fling, 0) }),
  };
}

// --- shaking a cocktail -------------------------------------------------------------------------

/** The shaker held before the chest (m, reference skeleton): the right hand round its tin, the left over its cap, its top tipped back over the right shoulder. */
const SHAKER: Vec = [0.12, 1.24, 0.3];
const SHAKER_AXIS: Vec = norm([0.25, 0.8, -0.55]);
const SHAKE_S = 2;

function shakePose(p: number): PoseSpec {
  // Four hard shakes along the shaker's line each second, a pause to look at it once a loop.
  const look = bump(p, 0.78, 1, 0.08);
  const go = Math.sin(p * TAU * 8) * (1 - look);
  const body = stand({ shift: 0.3, breath: p, lean: 2, twist: -4 + 3 * go, head: [8 + 6 * look, -12, 0] });
  const r = addV(SHAKER, SHAKER_AXIS, 0.045 * go);
  const l = addV(r, SHAKER_AXIS, 0.13);
  return {
    ...body,
    ...armIK('r', r, body, { palm: true, pole: [0.6, -1, -0.2], pronate: 70, hand: wrist('r', 0, 0) }),
    ...armIK('l', l, body, { palm: true, pole: [-0.5, -1, -0.3], pronate: -10, hand: wrist('l', 20, 0) }),
  };
}

/** The cup held where the sip starts and ends, bobbing a little with the breath, the head turning now and then. */
function holdCupPose(p: number): PoseSpec {
  const b = 0.006 * Math.sin(p * TAU);
  return eatPose(p, [CUP_HELD[0], CUP_HELD[1] + b, CUP_HELD[2]], [2 * Math.sin(p * TAU * 2), 6 * Math.sin(p * TAU), 0], 0);
}

export function propClips(): ClipDef[] {
  const s = TOSS_S;
  const low: Vec = [0.13, 1.0, 0.24];
  const chest: Vec = [0.06, 1.22, 0.17];
  const back: Vec = [0.23, 0.86, -0.06];
  const letGo: Vec = [0.17, 1.24, 0.5];
  const follow: Vec = [0.15, 1.33, 0.52];
  const rest: Vec = [0.2, 0.9, 0.08];
  return [
    clip('toss', { seconds: s, loop: false }, [
      [0, () => tossPose(low, 0, -60, 3, [18, 0, 0], 0)],
      [0.55 / s, () => tossPose(chest, -10, -20, 1, [22, 0, 2], 0.2), 'smooth'],
      [1.3 / s, () => tossPose(chest, -10, -20, 1, [24, 0, 2], 0.4), 'smooth'],
      [1.75 / s, () => tossPose(back, 20, 20, 2, [14, 0, 0], 0.55), 'smooth'],
      [TOSS_LET_GO, () => tossPose(letGo, -25, -50, 6, [8, 0, 0], 0.65), 'out'],
      [2.4 / s, () => tossPose(follow, -20, -60, 7, [16, 0, 0], 0.75), 'smooth'],
      [1, () => tossPose(rest, 8, 0, 3, [14, 0, 0], 1), 'smooth'],
    ]),
    cycle('holdCup', { seconds: 2.5, loop: true }, holdCupPose),
    cycle('scatter', { seconds: SCATTER_S, loop: true }, scatterPose),
    cycle('shake', { seconds: SHAKE_S, loop: true }, shakePose),
  ];
}
