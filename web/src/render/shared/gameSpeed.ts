/**
 * How fast the residents' animation time runs at each game speed (engine-neutral): the residents,
 * the objects' clips and the doors keep to it.
 */

/** Sim ticks per frame step at each game speed (`clock::TICKS_PER_STEP`). */
export const GAME_RATE = [0, 1, 3, 10, 30, 60];
/** Speed while time skips through quiet hours (`clock::AUTO_FAST_SPEED`). */
export const QUIET_SPEED = 4;
