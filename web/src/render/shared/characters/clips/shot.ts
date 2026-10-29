/**
 * Shooting hoops (`basketballHoop`, characters.ts `STEP_IN`: out on the court, a lane each for two): from
 * `SHOT.to` m out from the hoop's middle, its rim 2.94 m up and 0.27 m out (`playground.py` `RIM`); and at the
 * streetball hoop (`energetic.streetballHoop`, `STREET_SHOT`), its rim 2.4 m up and 0.22 m out
 * (`pack_energetic.py` `RIM`), from 2.2 m out, where its two players stand (content `places`). A round of
 * a few dribbles, the ball gathered to the chest, the dip and a jump shot, the right hand under the ball and the
 * left at its side, let go at the top with the wrist flicked through, a landing on bent knees and the follow-through
 * held while the ball drops in, a look up at it. A child heaves it up two-handed from the chest instead, with
 * a little hop. In sizes like the street's (`street.ts`): the rim's height and distance divided by the size give
 * the clip's, which set where the arms point and where the head looks. The ball goes with the round (`ballAt`): bounced
 * on the dribbles, in both hands from the gather, in an arc to the rim and back; its net swishes 1.4 s into the use and every 3.2 s after (`HOOP` in `playground.py`), the
 * round's length.
 */

import { cycle, type ClipDef, type PoseSpec, type Vec } from './dsl';
import { arms, body, track, type Key } from './games';
import { TAU } from './idle';
import { wrist } from './ik';
import { SHARES_177, sizes, type Fit, type Size } from './street';

/** The rim's height and how far out it is from the hoop's middle (m): the park's hoop and the streetball hoop. */
const RIM = { y: 2.94, z: 0.27 };
const STREET_RIM = { y: 2.4, z: 0.22 };
type ShotFit = Fit & { push: boolean; rim: { y: number; z: number } };
/** Three sizes; the smallest shoots two-handed from the chest. */
export const SHOT: Size<ShotFit>[] = sizes('hoopShot', SHARES_177, (s) => ({ s, to: 2.5, push: s < 0.7, rim: RIM }));
export const STREET_SHOT: Size<ShotFit>[] = sizes('streetShot', SHARES_177, (s) => ({ s, to: 2.2, push: s < 0.7, rim: STREET_RIM }));
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
  const rim: Vec = [0, fit.rim.y / s, (to - fit.rim.z) / s];
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

/** The basketball's radius (m; held.py `basketball`, its middle that far up from its origin). */
export const BALL_R = 0.1254;
/** The round's fits by clip, for the ball's path (`ballAt`). */
const FITS = new Map([...SHOT, ...STREET_SHOT].map(({ clip, fit }) => [clip, fit]));
/** The rim (its height and how far out from the hoop's middle, m) of clip `clip`'s round, if it is one. */
export function rimOf(clip: string): { y: number; z: number } | undefined {
  return FITS.get(clip)?.rim;
}
/** When (s into the round) the ball reaches the rim, and the floor under it after dropping through the net. */
const RIM_AT = T.release + 0.5;
const DOWN_AT = RIM_AT + 0.28;

function ease(u: number): number {
  const v = Math.min(1, Math.max(0, u));
  return v * v * (3 - 2 * v);
}

/**
 * Where the basketball's middle is `t` s into a round of clip `clip` at the hoop, into `out` (x, y, z): bounced under
 * the right palm on the dribbles (to the floor and back up to the hand every 0.3 s), taken into both hands at the
 * gather and held up to the set point, then let go in an arc to the rim (`rim`: its middle, in the world), down through
 * the net to the floor and back to the hand in two bounces for the next round. `palms`: the right's, then the left's,
 * as just posed; `feet` (x, z) and `yaw` the shooter's; `floor` the floor's height; `scale` the body's, `size` the
 * ball's (the hands'). False for a clip that isn't a round at a hoop.
 */
export function ballAt(clip: string, t: number, palms: Float32Array, rim: ArrayLike<number>, feet: ArrayLike<number>, yaw: number, floor: number, scale: number, size: number, out: Float32Array): boolean {
  const fit = FITS.get(clip);
  if (!fit) return false;
  const r = BALL_R * size;
  const fx = Math.sin(yaw);
  const fz = Math.cos(yaw);
  // Under the right palm; between the two, a little up from them (the right hand under the ball, the left at its side).
  const under = (y: number): void => {
    out[0] = palms[0];
    out[1] = y;
    out[2] = palms[2];
  };
  const held = (o: Float32Array, k: number): void => {
    o[k] = (palms[0] + palms[3]) / 2;
    o[k + 1] = (palms[1] + palms[4]) / 2 + 0.8 * r;
    o[k + 2] = (palms[2] + palms[5]) / 2;
  };
  const low = floor + r;
  if (t < T.gather - 0.15) {
    // To the floor and back: at the hand on the hand's top, on the floor half way.
    under(low + (palms[1] - r - low) * Math.abs(Math.cos((Math.PI * t) / 0.3)));
    return true;
  }
  if (t < T.release) {
    // From under the right hand into both, gathered to the chest, up to the set point.
    const w = ease((t - (T.gather - 0.15)) / 0.25);
    held(out, 0);
    out[0] = palms[0] + (out[0] - palms[0]) * w;
    out[1] = palms[1] - r + (out[1] - (palms[1] - r)) * w;
    out[2] = palms[2] + (out[2] - palms[2]) * w;
    return true;
  }
  if (t < RIM_AT) {
    // Let go from the set point (the clip's hands there, on this body) in an arc to the rim, a little over it.
    const set = fit.push ? [0, 1.28, 0.32] : [0.04, 1.77, 0.19];
    const sx = feet[0] + (fz * set[0] + fx * set[2]) * scale;
    const sy = floor + set[1] * scale + 0.8 * r;
    const sz = feet[1] + (-fx * set[0] + fz * set[2]) * scale;
    const u = (t - T.release) / (RIM_AT - T.release);
    out[0] = sx + (rim[0] - sx) * u;
    out[2] = sz + (rim[2] - sz) * u;
    out[1] = sy + (rim[1] + r * 0.3 - sy) * u + 4 * 0.55 * u * (1 - u);
    return true;
  }
  if (t < DOWN_AT) {
    // Through the net, down to the floor under the rim.
    const u = (t - RIM_AT) / (DOWN_AT - RIM_AT);
    out[0] = rim[0];
    out[2] = rim[2];
    out[1] = rim[1] + r * 0.3 + (low - rim[1] - r * 0.3) * u * u;
    return true;
  }
  // Back to the shooter in two bounces, up into the right hand for the next round's first dribble.
  const u = (t - DOWN_AT) / (SHOT_S - DOWN_AT);
  const bounce = 0.45 * Math.abs(Math.sin(2 * Math.PI * Math.min(u, 0.75) / 0.75)) * (1 - u);
  const lift = ease((u - 0.75) / 0.25);
  out[0] = rim[0] + (palms[0] - rim[0]) * ease(u);
  out[2] = rim[2] + (palms[2] - rim[2]) * ease(u);
  out[1] = low + bounce + (palms[1] - r - low) * lift;
  return true;
}

export function shotClips(): ClipDef[] {
  return [...SHOT, ...STREET_SHOT].map(({ clip, fit }) => cycle(clip, { seconds: SHOT_S, loop: true }, shotPose(fit)));
}
