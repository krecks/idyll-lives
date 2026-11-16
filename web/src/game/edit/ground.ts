/**
 * The generator's `Ground` (`town.ts`: what is taken on a world, where a lot may go) made again
 * from a draft, for the town edits: its roads, greens, water with its shore strip, every lot, the
 * backdrop, the parking and the squares. Only its shape is here so far.
 */

import type { Ground, NeighbourhoodDraft, Templates } from '../town';

/** A place along a road: which road, how far along it (m), and which side (+1: its left, looking along it; -1: its right). */
export interface RoadSpot {
  road: number;
  s: number;
  side: 1 | -1;
}

/** The ground of `draft` as the generator would have it with everything on it laid already. */
export function townGround(_draft: NeighbourhoodDraft, _t: Templates): Ground {
  throw new Error('townGround: not built yet');
}

/** Where each lot of `draft` meets its road (by slot index). */
export function entriesOf(_draft: NeighbourhoodDraft, _t: Templates): RoadSpot[] {
  throw new Error('entriesOf: not built yet');
}
