/**
 * How fast the game plays at each speed (engine-neutral): the residents, the objects' clips and the
 * doors keep to it.
 */

/**
 * Seconds of the sim's real time a real second at each game speed: the ticks a step runs (sim-core
 * `clock::TICKS_PER_STEP`), a step being 1 / `TICKS_PER_SECOND` s, so the same whatever the tick
 * rate. What moves in real time (walking, cars, doors, clips) goes this many times as fast; the
 * clock's game minutes go this times `gameMinutesPerSecond` (`core/snapshot.ts`) a real second.
 * Super speed's is its most: it runs as many ticks a step as the machine has time for, which the
 * snapshot says (`playbackRate`).
 */
export const PLAYBACK_RATE = [0, 1, 3, 10, 60];
/** Super speed (`clock::SUPER_SPEED`): up to 30×, up to 60× while everyone at home sleeps or is out. */
export const SUPER_SPEED = 4;

/**
 * The real rate a snapshot plays at: the ticks its step ran (the header's `rate`; at super speed
 * what the machine had time for), or for a layout without it the speed's `PLAYBACK_RATE`.
 */
export function playbackRate(snap: ArrayLike<number>, header: { speed: number; rate?: number }): number {
  if (header.rate !== undefined) return snap[header.rate] ?? 0;
  return PLAYBACK_RATE[snap[header.speed] | 0] ?? 1;
}
