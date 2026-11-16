/**
 * Step 2's street edits: a street, lane, country road or footpath drawn on the town (snapped to
 * the roads it starts and ends at, joining them in a T, bridged over water: `bridges.ts`), a road
 * removed with what stands on it, a road, place, village or the town renamed. Each keeps the
 * generator's own rules (`roads.ts` `clearOf`, `roomAt`, `bends`). Only their shape is here so far.
 */

import type { RoadKind } from '../../core/protocol';
import type { Pt } from '../roads';
import type { NeighbourhoodDraft, Templates } from '../town';
import type { Edited } from './edited';

/** A street being drawn: its points as the player set them, its kind, whether what is in its way is cleared (else it is refused there) and whether it is filled with homes, and the seed its homes are drawn from. */
export interface StreetStroke {
  points: readonly Pt[];
  kind: RoadKind;
  clear: boolean;
  fill: boolean;
  seed: number;
}

/** What a name is given to: a road, a place (`OpenWorldDraft.places`), a village (an area), or the town. */
export type Named = { kind: 'road'; road: number } | { kind: 'place'; place: number } | { kind: 'area'; area: number } | { kind: 'town' };

/** `draft` with the street `stroke` draws. */
export function drawStreet(_draft: NeighbourhoodDraft, _t: Templates, _stroke: StreetStroke): Edited {
  throw new Error('drawStreet: not built yet');
}

/** `draft` without road `road`, its lots and parking, and any road it leaves joined to nothing; refused if no road out of the world would be left. */
export function removeRoad(_draft: NeighbourhoodDraft, _road: number): Edited {
  throw new Error('removeRoad: not built yet');
}

/** `draft` with `what` called `name` (a road's houses numbered along it again). */
export function rename(_draft: NeighbourhoodDraft, _what: Named, _name: string): Edited {
  throw new Error('rename: not built yet');
}
