/**
 * What the Street tool draws, as its card (`StreetCard.svelte`) sets it: the kind of road, whether
 * what is in its way is cleared, and what is put along it. Kept while the tool is put down and
 * taken up again.
 */

import type { RoadKind } from '../../core/protocol';

/** The kind asked for; `auto`: as `drawStreet` chooses by where it runs (a street of the town or village it runs in, else a country road). */
export type StreetKindChoice = 'auto' | 'street' | 'lane' | 'country' | 'trail';
/** What goes along a new street: homes from the area's lot mix, the town's other buildings, both, or nothing. */
export type Along = 'homes' | 'buildings' | 'both' | 'none';

export const streetChoice = $state({
  kind: 'auto' as StreetKindChoice,
  clear: true,
  /** Homes for now: town buildings come with building streets up (`buildup.ts`), and then the default is both in the main town. */
  along: 'homes' as Along,
  /** What the street under the pointer would be (the card shows `auto` as it). */
  shown: 'street' as RoadKind,
});

/** The kind `drawStreet` is asked for (`street` for `auto`: it makes a street a country road outside the town and villages). */
export const strokeKind = (k: StreetKindChoice): RoadKind => (k === 'auto' ? 'street' : k);

/** Whether homes are put along it (`drawStreet`'s `fill`). */
export const fillsHomes = (a: Along) => a === 'homes' || a === 'both';
