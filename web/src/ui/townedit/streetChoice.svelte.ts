/**
 * What the Street tool draws, as its card (`StreetCard.svelte`) sets it: the kind of road, whether
 * what is in its way is cleared, and what is put along it. Kept while the tool is put down and
 * taken up again.
 */

import type { RoadKind } from '../../core/protocol';

/** The kind asked for; `auto`: as `drawStreet` chooses by where it runs (a street of the town or village it runs in, else a country road). */
export type StreetKindChoice = 'auto' | 'street' | 'lane' | 'country' | 'trail';
/** What goes along a new street: homes from the area's lot mix, the town's other buildings, both, or nothing; `auto`: by where it runs (`alongFor`). */
export type Along = 'auto' | 'homes' | 'buildings' | 'both' | 'none';

export const streetChoice = $state({
  kind: 'auto' as StreetKindChoice,
  clear: true,
  along: 'auto' as Along,
  /** What the street under the pointer would be (the card shows `auto` as it), and the area it is a street of (-1: none). */
  shown: 'street' as RoadKind,
  shownArea: -1,
  /** The roads the last street drawn was made as (split at its bridges) and their names: the card lists them while the draft has them (`streetLive.ts` `drawnStretches`). */
  drawn: [] as { road: number; name: string }[],
});

/** The kind `drawStreet` is asked for (`street` for `auto`: it makes a street a country road outside the town and villages). */
export const strokeKind = (k: StreetKindChoice): RoadKind => (k === 'auto' ? 'street' : k);

/** What goes along a street of area `area` (-1: none): `auto` is both homes and the town's buildings in the main town, homes elsewhere. */
export const alongFor = (a: Along, area: number): Exclude<Along, 'auto'> => (a !== 'auto' ? a : area === 0 ? 'both' : 'homes');

/** Whether homes are put along it by `drawStreet` itself (its `fill`): homes alone (with the town's buildings, build-up lays both: `buildup.ts` `builtAlong`). */
export const fillsHomes = (a: Exclude<Along, 'auto'>) => a === 'homes';
