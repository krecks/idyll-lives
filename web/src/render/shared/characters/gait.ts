/**
 * Which gait a walker plays and how fast (`residents.ts` `choose`): the walk, jog and run cycles matched to the
 * ground speed, each up to its most strides a second (`GAIT_MAX_RATE`, the run's `runMaxRate`), the slowest gait
 * that needn't go faster than that, a slower one than now only well within it (`JOG_BACK`).
 */

import { GAIT_MAX_RATE, GEARS, JOG_BACK, RUN_MAX_RATE } from './actions';

/**
 * The run's most strides a second for legs `legs` long (the body's `pelvisScale` times its height factor; 1 is the
 * reference young man's): `RUN_MAX_RATE`, faster on shorter legs as the square root of their length, as legs swing
 * (a pendulum's beat). A child's (0.55–0.57) goes to 5.3–5.4, so at game speed 3 (15 m/s) their feet stay down
 * (they slid 11–14 %, at most 4); a teen's 4.3–4.4. Never slower than the reference's: tall legs don't need it.
 */
export function runMaxRate(legs: number): number {
  return RUN_MAX_RATE / Math.sqrt(Math.min(1, Math.max(0.25, legs)));
}

/**
 * The gait (an index into `GEARS`) and its rate for a walker at `speed` m/s on legs `legs` long, playing gear `now`
 * (-1: none of them); `speedOf` is each gait clip's ground speed at rate 1 on the reference legs (`Clip.speed`),
 * undefined for one the set doesn't have.
 */
export function gaitFor(speed: number, legs: number, now: number, speedOf: (key: string) => number | undefined): { gear: number; rate: number } {
  let gear = 0;
  while (gear < GEARS.length - 1) {
    const up = speedOf(GEARS[gear + 1]);
    if (up === undefined || speed / (speedOf(GEARS[gear])! * legs) <= GAIT_MAX_RATE * (gear < now ? JOG_BACK : 1)) break;
    gear++;
  }
  const cap = gear === GEARS.length - 1 ? runMaxRate(legs) : GAIT_MAX_RATE;
  return { gear, rate: Math.min(Math.max(speed / (speedOf(GEARS[gear])! * legs), 0.35), cap) };
}

/** How much of the way the feet slide (0..1) at `speed` m/s for the gait `gaitFor` picks: what its cap leaves short. */
export function gaitSlide(speed: number, legs: number, speedOf: (key: string) => number | undefined): number {
  const { gear, rate } = gaitFor(speed, legs, -1, speedOf);
  return Math.max(0, 1 - (rate * speedOf(GEARS[gear])! * legs) / speed);
}
