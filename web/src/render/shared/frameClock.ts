/**
 * The renderer's clock for what is drawn at a rate of its own (the sun's and the lamps' shadow maps, the reflections,
 * the lit windows, a frame held while nothing moves, the lot the camera settles on): the wall clock
 * (`performance.now()`), or in fixed-step playback (`Renderer.framesOnDemand`) the time of the frame being drawn, so
 * that the same frames draw the same whatever each of them cost. Time measured for its own sake (a build's slice, what a
 * frame takes) stays the wall clock's.
 */

import { mulberry32 } from '../../game/noise';

let fixed: number | null = null;
/** The latest time the clock has been fixed at (fixed-step playback runs ahead of the wall clock). */
let latest = 0;

/** Now (ms, the `performance.now()` clock): the frame's time in fixed-step playback. */
export function frameNow(): number {
  return fixed ?? performance.now();
}

/** Fixed-step playback: the clock reads `time` until it is set again (null: the wall clock again). */
export function fixFrameClock(time: number | null): void {
  fixed = time;
  if (time !== null) latest = Math.max(latest, time);
}

/**
 * Where a new stretch of fixed-step playback starts its clock: a whole millisecond a second past the wall clock and
 * past every frame drawn before (a run before this one ran ahead of the wall clock), so whatever was held back by its
 * rate then is due at its first frame, however long the page has been open.
 */
export function fixedStart(): number {
  return Math.ceil(Math.max(performance.now(), latest)) + 1000;
}

/** Fixed-step playback's dice (`restartDice`), or none: `Math.random`. */
let dice: (() => number) | null = null;

/**
 * Dice for what the eye alone sees (a bird's glance, its hop): `Math.random`, or in fixed-step playback a stream that
 * starts again with it (`restartDice`), so the same frames draw the same birds.
 */
export function frameRandom(): number {
  return dice ? dice() : Math.random();
}

/** Fixed-step playback: its dice from the start (`on`), or `Math.random` again. */
export function restartDice(on: boolean): void {
  dice = on ? mulberry32(1) : null;
}
