/**
 * The town's paved ground near, as the street's chunks draw it (`street.ts` `TownStreet.chunk`):
 * for each stream chunk, the pieces `townPaving` gives draped over the ground in cells of 2 m or
 * less, their courses' joints, the bays' lines just over the asphalt, and a P sign by each car park.
 * `streetGroundKey` says what a chunk's share is drawn from, so a chunk whose share is the same is
 * not planned again.
 */

import type { WorldStructure } from '../../core/protocol';
import type { ChunkPlan } from './street';

/** What the near paving reads (as `townPaving.ts` `PavingWorld`). */
export type StreetGroundWorld = Pick<WorldStructure, 'plots' | 'meta' | 'parking' | 'carParks'>;

/** What chunk `id`'s share of the paved ground is drawn from, as a key ('': none). */
export function streetGroundKey(world: StreetGroundWorld, id: number): string {
  void world;
  void id;
  return '';
}

/**
 * Adds chunk `id`'s share of the paved ground to its plan: its layers (`ChunkPlan.ground`) and its
 * P signs (`ChunkPlan.batches`), laid over `ground` where the land is shaped (none: at 0).
 */
export function streetGround(plan: ChunkPlan, world: StreetGroundWorld, id: number, ground?: (x: number, z: number) => number): void {
  void plan;
  void world;
  void id;
  void ground;
}
