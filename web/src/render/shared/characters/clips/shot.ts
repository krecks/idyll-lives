/**
 * Shooting hoops (`basketballHoop`, characters.ts `STEP_IN`: out on the court, a lane each for two): from
 * `SHOT.to` m out from the hoop's middle, its rim 2.94 m up and 0.27 m out (`playground.py` `RIM`). A round of
 * a few dribbles, the ball gathered to the chest, the dip and a jump shot, the right hand under the ball and the
 * left at its side, let go at the top with the wrist flicked through, a landing on bent knees and the follow-through
 * held while the ball drops in, a look up at it. A child heaves it up two-handed from the chest instead, with
 * a little hop. In sizes like the street's (`street.ts`): the rim's height and distance divided by the size give
 * the clip's, which set where the arms point and where the head looks. The ball is mimed (props in hand are a
 * later item); its net swishes 1.4 s into the use and every 3.2 s after (`HOOP` in `playground.py`), the
 * round's length.
 */

import { cycle, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { arms, body, track, type Key } from './games';
import { TAU } from './idle';
import { wrist } from './ik';
import { SHARES_177, sizes, type Fit, type Size } from './street';

/** The rim's height and how far out it is from the hoop's middle (m). */
const RIM = { y: 2.94, z: 0.27 };
type ShotFit = Fit & { push: boolean };
/** Three sizes; the smallest shoots two-handed from the chest. */
export const SHOT: Size<ShotFit>[] = sizes('hoopShot', SHARES_177, (s) => ({ s, to: 2.5, push: s < 0.7 }));
const SHOT_S = 3.2;
/** When (s into the round) the ball is gathered, the dip, the jump, the release and the landing. */
const T = { gather: 1.05, dip: 1.35, jump: 1.55, release: 1.72, land: 1.95, settle: 2.3 };

/**
 * A round at the hoop: three dribbles at the right hip, the ball gathered to the chest, the knees dipping,
 * up off the floor with the ball to the set point over the forehead, let go at the top towards the rim, down
 * onto bent knees with the follow-through held, and the arms down as the ball drops.
 */
function shotPose(fit: ShotFit): (p: number) => PoseSpec {
  const { s, to, push } = fit;
  // The rim from the eyes' place, in the clip's frame: its height and how far ahead.
  const rim: Vec = [0, RIM.y / s, (to - RIM.z) / s];
  const aim = (Math.atan2(rim[1] - 1.6, rim[2]) * 180) / Math.PI;
  // The pelvis: low for the dip, up off the floor in the jump (a child's hop smaller), and the knees bent on landing.
  const up = push ? 0.06 : 0.13;
  const Y: Key<number>[] = [[0, -0.04], [T.gather, -0.03], [T.dip, -0.12], [T.jump, 0.0, 'out'], [T.release, up, 'out'], [T.land, -0.02, 'in'], [T.land + 0.12, -0.1, 'out'], [T.settle, -0.03], [SHOT_S, -0.04]];
  // The feet: off the floor while the pelvis is up past standing (toes pointed), back where they were.
  const LIFT: Key<number>[] = [[0, 0], [T.jump, 0], [T.release, up * 0.9, 'out'], [T.land, 0, 'in'], [SHOT_S, 0]];
  const BEND: Key<number>[] = [[0, 14], [T.gather, 10], [T.dip, 18], [T.jump, 4], [T.release, -2], [T.land + 0.12, 14], [T.settle, 6], [SHOT_S, 14]];
  // The right hand: the dribbles at the hip (down, up), to the chest, the set point, let go up towards the rim, the follow-through, down.
  const bounce = (t: number): Vec => [0.25, 0.85 - 0.18 * (0.5 - 0.5 * Math.cos((t / 0.3) * TAU)), 0.3];
  const chest: Vec = [0.06, 1.2, 0.3];
  const set: Vec = push ? [0.08, 1.28, 0.32] : [0.12, 1.75, 0.16];
  const out: Vec = push ? [0.08, 1.6, 0.6] : [0.1, 2.05, 0.45];
  const R: Key<Vec>[] = [[T.gather - 0.15, [0.25, 0.85, 0.3]], [T.gather + 0.1, chest], [T.dip, chest], [T.jump + 0.05, set], [T.release, set], [T.release + 0.1, out, 'out'], [T.settle, out], [SHOT_S - 0.35, [0.22, 1.0, 0.28]], [SHOT_S, [0.25, 0.85, 0.3]]];
  const L: Key<Vec>[] = [[0, [-0.2, 0.95, 0.2]], [T.gather - 0.1, [-0.15, 1.0, 0.3]], [T.gather + 0.1, [-0.1, 1.2, 0.33]], [T.dip, [-0.1, 1.2, 0.33]], [T.jump + 0.05, push ? [-0.08, 1.28, 0.32] : [-0.04, 1.78, 0.22]], [T.release, push ? [-0.08, 1.28, 0.32] : [-0.04, 1.78, 0.22]], [T.release + 0.1, push ? [-0.08, 1.6, 0.6] : [-0.12, 1.85, 0.3], 'out'], [T.settle, push ? [-0.08, 1.55, 0.58] : [-0.14, 1.8, 0.28]], [SHOT_S - 0.3, [-0.2, 0.95, 0.2]], [SHOT_S, [-0.2, 0.95, 0.2]]];
  // The wrist: cocked back under the ball, flicked through at the release.
  const FLICK: Key<number>[] = [[T.gather - 0.15, 0], [T.jump, -45], [T.release, -50], [T.release + 0.1, 55, 'out'], [T.settle, 50], [SHOT_S - 0.3, 0]];
  // The palm down on the dribble, turned up under the ball from the gather.
  const PRONATE: Key<number>[] = [[T.gather - 0.15, 70], [T.gather + 0.1, push ? -60 : -30], [T.settle, push ? -60 : -30], [SHOT_S - 0.2, 70]];
  return (p) => {
    const t = p * SHOT_S;
    const lift = track(LIFT, t);
    const y = track(Y, t);
    // Looking down at the dribble, up at the rim from the gather.
    const look = Math.min(1, Math.max(0, (t - 0.85) / 0.4)) * (1 - Math.min(1, Math.max(0, (t - (SHOT_S - 0.4)) / 0.4)));
    const nod = 22 * (1 - look) - look * Math.min(40, aim * 0.9);
    const b = body({
      breath: p,
      at: [0.005, y, -0.02 - 0.06 * Math.max(0, -y)],
      hip: 8 + 50 * Math.max(0, -y),
      bend: track(BEND, t),
      head: [nod, 0, 0],
      l: [-0.02, lift, 0.03],
      r: [0.03, lift, -0.04],
      fl: [lift * 160, -8, 0],
      fr: [lift * 160, 6, 0],
    });
    const r = t < T.gather - 0.15 ? bounce(t) : track(R, t);
    const flick = track(FLICK, t);
    return arms(
      b,
      { at: track(L, t), palm: true, pole: [-0.8, -0.5, -0.4], pronate: push ? -60 : -80, hand: wrist('l', -10, 0) },
      { at: r, palm: true, pole: [0.6, -0.7, -0.3], pronate: track(PRONATE, t), hand: wrist('r', t < T.gather - 0.15 ? 20 * Math.sin((t / 0.3) * TAU) : flick, 0) },
    );
  };
}

export function shotClips(): ClipDef[] {
  return SHOT.map(({ clip, fit }) => cycle(clip, { seconds: SHOT_S, loop: true }, shotPose(fit)));
}
