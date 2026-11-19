/**
 * Flats: a building of several homes on one lot, each a plot of its own (`FlatInfo`, the town
 * file's `plots[].of`) on one storey. A walk-up of 2 or 3 storeys round a stair hall, its flats
 * generated from the house generator's `flat` class, its lot a front strip with a bay for some of
 * its flats and a yard behind. The same model makes the flats over a shop, a café or a restaurant.
 */

import type { PlotSlot, Templates } from './town';

type Rand = () => number;

/** A flats building as a slot takes it: its template (`flats:<seed>:<storeys>:<wide>`), its lot's size and its look. */
export type FlatsSlot = Pick<PlotSlot, 'template' | 'w' | 'd' | 'dress'>;

/**
 * A flats building for a lot in `zone` (the centre or the rest of a town), drawn from `rand`; none
 * until the generator makes flats (`flats.ts` is filled in by the flats stream).
 */
export function flatsFor(t: Templates, zone: 'centre' | 'town', rand: Rand): FlatsSlot | null {
  void t;
  void zone;
  void rand;
  return null;
}
