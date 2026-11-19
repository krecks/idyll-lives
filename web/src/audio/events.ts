/**
 * The rules for the sounds of things happening (`crates/sound/src/things/events.rs` makes them; `world.ts` and
 * `outside.ts` play them): how many doors may sound at once, what a car's row says happened since the last look,
 * and which curtains were drawn or opened. Plain functions, no DOM or audio context (`tests/events.test.ts`).
 */

import type { InteractionDef } from '../content/content';
import { EVENTS } from './catalog';

/** Whether one more door may sound at audio time `t` (s); if so it is counted in `recent`. */
export function doorMay(recent: number[], t: number): boolean {
  const d = EVENTS.door;
  let keep = 0;
  for (const at of recent) if (t - at < d.within) recent[keep++] = at;
  recent.length = keep;
  if (keep >= d.voices || (keep > 0 && t - recent[keep - 1] < d.apart)) return false;
  recent.push(t);
  return true;
}

/**
 * What a car's row says happened since the last look (`was`; `shown` 0 out of sight, 1 parked, 2 driving; `doors`
 * a bit per open door): a door shut (one open last look is shut now, the car in sight), or a parked car setting off.
 * Nothing on the first look at it, nor for a car just coming into sight (into town). A work taxi (`taxi`) stands
 * at each stop with its engine running: moving off again is no start.
 */
export function carEvents(was: { shown: number; doors: number } | undefined, shown: number, doors: number, taxi = false): { door: boolean; start: boolean } {
  if (!was || was.shown === 0) return { door: false, start: false };
  return { door: shown !== 0 && (was.doors & ~doors) !== 0, start: !taxi && was.shown === 1 && shown === 2 };
}

/** The ids that joined or left a set; none on the first sight of it (`was` null). */
export function changed(was: ReadonlySet<number> | null, now: ReadonlySet<number>): number[] {
  if (!was) return [];
  const out: number[] = [];
  for (const id of now) if (!was.has(id)) out.push(id);
  for (const id of was) if (!now.has(id)) out.push(id);
  return out;
}

/**
 * What a use says it sounds like (content `sound`, played while it lasts or once as it starts, and `endSound`, once
 * as it ends) done as animation `tag`. Done as its self-serve (`selfServe`: a household's own cart, the bar's drink
 * poured by the customer; another animation than the use's own) nobody serves or takes money: the use's own sounds
 * are not heard, only the self-serve's `endSound` (a shop counter's till, paid at on the way out).
 */
export function useSounds(use: Pick<InteractionDef, 'anim' | 'sound' | 'endSound' | 'selfServe'> | undefined, tag: string | undefined): { sound?: string; endSound?: string } {
  if (!use) return {};
  if (use.selfServe && tag !== use.anim) return { endSound: use.selfServe.endSound };
  return { sound: use.sound, endSound: use.endSound };
}
