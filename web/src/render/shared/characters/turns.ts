/**
 * Turning on the spot (`residents.ts`): a resident standing still who has to face a way well off the one they face
 * (a thing they've come to use, someone who talks to them) steps round to it with a turn clip
 * (`clips/locomotion.ts` `turnClip`), the size nearest the turn, rather than pivoting on planted feet. The turn
 * itself is the runtime's: the Sim's yaw follows the clip's steps (`turnShare`) from where it started to the way
 * wanted, whatever the size played. Smaller turns pivot, as before.
 */

import { bake } from './clips/dsl';
import { TURN_ANGLES, turnClip, turnDef, turnShare } from './clips/locomotion';
import type { CharacterSet, Clip } from './data';
import { type SimRig, wrapAngle } from './rig';

/** Turns smaller than this (rad) pivot on the planted feet; from this on they step round. */
export const TURN_MIN = (35 * Math.PI) / 180;
/** The crossfade into a turn and out of it (s). */
export const TURN_FADE = 0.2;

/** Whether `key` is a turn on the spot. */
export function isTurn(key: string): boolean {
  return /^turn[LR]\d/.test(key);
}

/** The turn clip `key` (`turnClip`'s name for one of `TURN_ANGLES`), baked into `set` if it isn't there yet; undefined for any other name. */
export function turnClipIn(set: CharacterSet, key: string): Clip | undefined {
  const have = set.clips.get(key);
  if (have) return have;
  const m = /^turn([LR])(\d+)(Child)?$/.exec(key);
  if (!m || !TURN_ANGLES.includes(Number(m[2]))) return undefined;
  const clip = bake([turnDef(Number(m[2]), m[1] === 'R', !!m[3])], set).get(key)!;
  set.clips.set(key, clip);
  return clip;
}

/**
 * The yaw of a resident turning on the spot this frame, or null when they aren't (they pivot as before). `still`:
 * standing free, nothing else turning them (residents.ts `update`); `face`: the way they should face; `child`: a
 * child's turn clips (`turnClip`), baked into `set` the first time one is needed. Starts a turn when `face` is
 * `TURN_MIN` or more off, and drops one no longer `still` (a walk takes over, from the yaw it had got to).
 */
export function turnOnSpot(set: CharacterSet, rig: SimRig, still: boolean, face: number, child: boolean): number | null {
  if (!still) {
    rig.turnKey = '';
    return null;
  }
  if (!rig.turnKey) {
    const by = wrapAngle(face - rig.yaw);
    if (Math.abs(by) < TURN_MIN) return null;
    const deg = (Math.abs(by) * 180) / Math.PI;
    rig.turnDeg = TURN_ANGLES.reduce((best, a) => (Math.abs(a - deg) < Math.abs(best - deg) ? a : best), TURN_ANGLES[0]);
    // (+ yaw turns the body towards its +x, its right.)
    rig.turnKey = turnClip(rig.turnDeg, by > 0, child);
    turnClipIn(set, rig.turnKey);
    rig.turnFrom = rig.yaw;
    rig.turnBy = by;
    rig.turnStarted = false;
  }
  return turnYaw(rig);
}

/** The yaw of a resident turning on the spot, where their turn clip has got to (again once it has advanced this frame: the feet stay put only with the yaw of the frame they're drawn at). */
export function turnYaw(rig: SimRig): number {
  const clip = rig.turnStarted && rig.cur.key === rig.turnKey ? rig.cur.clip : null;
  const p = clip ? Math.min(1, rig.cur.time / clip.duration) : 0;
  return wrapAngle(rig.turnFrom + rig.turnBy * turnShare(rig.turnDeg, p));
}

/** Whether the turn `rig` is making has played to its end (`choose` then goes on with what they do). */
export function turnDone(rig: SimRig): boolean {
  return rig.turnStarted && rig.cur.key === rig.turnKey && !!rig.cur.clip && rig.cur.time >= rig.cur.clip.duration;
}
