/**
 * Which lots may be drawn open (their roof off, their rooms and things shown): the household's
 * home, a lot one of the household is on now, and a lot the player chose to look inside (the
 * building card's "Look inside"). Another household's house, a shop with none of yours in it,
 * stays a closed house as the camera roams over it (`session.ts` `onFocusLot`); following
 * someone still opens the lot they are on.
 */

import type { SimView } from '../core/protocol';

export interface OpenLots {
  /** The household's home lot, if it has one. */
  home: number | null;
  /** The lot the player chose to look inside, if any. */
  lookInside: number | null;
  sims: readonly Pick<SimView, 'id' | 'plot' | 'awayUntil'>[];
  /** The household's members. */
  mine: ReadonlySet<number>;
}

/** Whether lot `plot` may be shown open now. */
export function mayOpen(plot: number, s: OpenLots): boolean {
  if (plot === s.home || plot === s.lookInside) return true;
  for (const sim of s.sims) if (sim.plot === plot && sim.awayUntil === null && s.mine.has(sim.id)) return true;
  return false;
}
