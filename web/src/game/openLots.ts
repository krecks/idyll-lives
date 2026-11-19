/**
 * Which lots may be drawn open (their roof off, their rooms and things shown): the household's
 * home, a lot one of the household is on now, and a lot the player chose to look inside (the
 * building card's "Look inside"). Another household's house, a shop with none of yours in it,
 * stays a closed house as the camera roams over it (`session.ts` `onFocusLot`); following
 * someone still opens the lot they are on.
 *
 * A flat is a plot of its own: it opens as a home does (yours, one of yours in it, looked inside).
 * Its building opens if any of its flats may, or one of yours is in its hall or yard (on the
 * building's own plot), or it is looked inside; the renderer then asks of each of its flats, and
 * those that may not stay lidded.
 */

import type { FlatInfo, SimView } from '../core/protocol';

export interface OpenLots {
  /** The household's home (a lot, or a flat), if it has one. */
  home: number | null;
  /** The plot the player chose to look inside (a lot, or a flat), if any. */
  lookInside: number | null;
  sims: readonly Pick<SimView, 'id' | 'plot' | 'awayUntil'>[];
  /** The household's members. */
  mine: ReadonlySet<number>;
  /** The town's flats (`WorldStructure.flats`): a building opens with them. */
  flats?: readonly Pick<FlatInfo, 'id' | 'of'>[];
}

/** Whether plot `plot` (a lot, or a flat) may be shown open now. */
export function mayOpen(plot: number, s: OpenLots): boolean {
  if (plot === s.home || plot === s.lookInside) return true;
  for (const sim of s.sims) if (sim.plot === plot && sim.awayUntil === null && s.mine.has(sim.id)) return true;
  for (const f of s.flats ?? []) if (f.of === plot && mayOpen(f.id, s)) return true;
  return false;
}
