/**
 * What the zone brush paints, as its card (`ZoneCard.svelte`) sets it: a use (`uses.ts`) or Erase,
 * the brush's size, and whether it paints only beside the streets. Kept while the tool is put down
 * and taken up again.
 */

import type { UseKind } from '../../game/uses';

/** The brush's sizes (m across). */
export const ZONE_SIZES = [8, 16, 32, 64] as const;
export type ZoneSize = (typeof ZONE_SIZES)[number];

export const zoneChoice = $state({
  /** The use painted; null: Erase. */
  kind: 'homes' as UseKind | null,
  size: 16 as ZoneSize,
  /** Paints only within `layoutApi.ts` `BESIDE_STREETS` m of a road (erasing goes anywhere). */
  beside: true,
});
