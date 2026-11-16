/**
 * Step 2's village edits: a village placed by hand (a hamlet of the place's own kind, its streets
 * laid by the generator's functions, its road by `countryRouter.ts`, its lots by `LotPlacer`),
 * moved, or removed with everything in it. Only their shape is here so far.
 */

import type { Content } from '../../content/content';
import type { NeighbourhoodDraft, Templates } from '../town';
import type { Edited } from './edited';

/** Where a village is placed (its box's middle, m), whether its shape is mirrored, and the seed its streets, lots and names are drawn from. */
export interface VillageAt {
  x: number;
  z: number;
  swap: boolean;
  seed: number;
}

/** `draft` with a village placed `at`, joined to the roads where a road can be found. */
export function placeVillage(_draft: NeighbourhoodDraft, _t: Templates, _content: Content, _at: VillageAt): Edited {
  throw new Error('placeVillage: not built yet');
}

/** `draft` with village `area` moved by whole metres (`dx`, `dz`), routed to the roads again. */
export function moveVillage(_draft: NeighbourhoodDraft, _area: number, _dx: number, _dz: number): Edited {
  throw new Error('moveVillage: not built yet');
}

/** `draft` without village `area`: its streets, lots, backdrop, places and its area. */
export function removeVillage(_draft: NeighbourhoodDraft, _area: number): Edited {
  throw new Error('removeVillage: not built yet');
}

/** `draft` without area `area`, the area indexes on everything else numbered again. */
export function removeArea(_draft: NeighbourhoodDraft, _area: number): NeighbourhoodDraft {
  throw new Error('removeArea: not built yet');
}
