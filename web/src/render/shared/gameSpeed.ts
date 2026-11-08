/**
 * How fast the game plays at each speed (engine-neutral): the residents, the objects' clips and the
 * doors keep to it.
 */

/**
 * Seconds of the sim's real time a real second at each game speed: the ticks a step runs (sim-core
 * `clock::TICKS_PER_STEP`), a step being 1 / `TICKS_PER_SECOND` s, so the same whatever the tick
 * rate. What moves in real time (walking, cars, doors, clips) goes this many times as fast; the
 * clock's game minutes go this times `gameMinutesPerSecond` (`core/snapshot.ts`) a real second.
 */
export const PLAYBACK_RATE = [0, 1, 3, 10, 30, 60];
/** Speed while time skips through quiet hours (`clock::AUTO_FAST_SPEED`). */
export const QUIET_SPEED = 4;
