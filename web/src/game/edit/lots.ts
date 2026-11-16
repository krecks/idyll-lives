/**
 * Step 2's lot edits: a lot added along a road (a home, a farm, a public place, an outdoor spot),
 * moved, a new street filled with homes, a lot's house, garden or public place changed, a civic
 * building's use swapped, lots removed, and the houses along streets numbered again. Each keeps
 * the generator's own rules (`town.ts` `lotAt`, `LotPlacer.place`, `withGarden`). Only
 * `removeLots` is built so far (the slope limits' last resort uses it too: `landFollow.ts`
 * `settleLand`); the rest have their shape.
 */

import type { Content } from '../../content/content';
import type { GardenSize } from '../housegen';
import type { LotSize, NeighbourhoodDraft, Templates } from '../town';
import type { Edited } from './edited';
import type { RoadSpot } from './ground';

/** What goes on a lot being added: a home of a size (with a garden), a farm, or a public place or outdoor spot (its template). */
export type LotChoice = { kind: 'house'; size: Exclude<LotSize, 'farm'>; garden?: GardenSize } | { kind: 'farm' } | { kind: 'park' | 'shop' | 'outdoor'; template: string };

/** `draft` with a lot of `choice` at `at`, its house drawn from `seed`; refused where it doesn't fit or no car can drive out. */
export function addLot(_draft: NeighbourhoodDraft, _t: Templates, _content: Content, _at: RoadSpot, _choice: LotChoice, _seed: number): Edited {
  throw new Error('addLot: not built yet');
}

/** `draft` with the lot on `slot` moved to `at`, keeping its house, garden and kind. */
export function moveLot(_draft: NeighbourhoodDraft, _t: Templates, _slot: number, _at: RoadSpot): Edited {
  throw new Error('moveLot: not built yet');
}

/** `draft` with homes from its area's lot mix along road `road`, drawn from `seed`. */
export function fillStreet(_draft: NeighbourhoodDraft, _t: Templates, _content: Content, _road: number, _seed: number): Edited {
  throw new Error('fillStreet: not built yet');
}

/** `draft` with another house (`template`) on `slot`, where it fits as `withGarden` fits one. */
export function anotherHouse(_draft: NeighbourhoodDraft, _t: Templates, _slot: number, _template: string): Edited {
  throw new Error('anotherHouse: not built yet');
}

/** `draft` with the garden on `slot` made `garden`. */
export function setGarden(_draft: NeighbourhoodDraft, _t: Templates, _slot: number, _garden: GardenSize): Edited {
  throw new Error('setGarden: not built yet');
}

/** `draft` with the public lot on `slot` made another public place of its size (`template`). */
export function swapPlace(_draft: NeighbourhoodDraft, _t: Templates, _slot: number, _template: string): Edited {
  throw new Error('swapPlace: not built yet');
}

/** `draft` with backdrop building `building` made another civic place (`use`), its footprint kept. */
export function swapCivic(_draft: NeighbourhoodDraft, _building: number, _use: string): Edited {
  throw new Error('swapCivic: not built yet');
}

/** `draft` with the houses along `roads` numbered again (odd on a road's left, even on its right). */
export function renumber(_draft: NeighbourhoodDraft, _roads: readonly number[]): NeighbourhoodDraft {
  throw new Error('renumber: not built yet');
}

/**
 * `draft` without the lots on `lots` (slot indices) and the backdrop buildings `backdrop` (their
 * indices in `open.backdrop`): the slots left numbered again from 0, and the pads, places and
 * backdrop buildings that name them with them; a place goes with its lot or its building.
 */
export function removeLots(draft: NeighbourhoodDraft, lots: ReadonlySet<number>, backdrop: ReadonlySet<number> = new Set()): NeighbourhoodDraft {
  const open = draft.open;
  const kept = draft.slots.filter((s) => !lots.has(s.index));
  const renumber = new Map(kept.map((s, i) => [s.index, i]));
  const slots = kept.map((s, i) => ({ ...s, index: i }));
  if (!open) return { ...draft, slots };
  const pads: Record<number, number> = {};
  for (const [k, v] of Object.entries(open.pads)) if (renumber.has(Number(k))) pads[renumber.get(Number(k))!] = v;
  const places = open.places ?? [];
  const placeGone = new Set<number>();
  places.forEach((q, i) => {
    if (q.plot !== undefined && !renumber.has(q.plot)) placeGone.add(i);
  });
  (open.backdrop ?? []).forEach((b, i) => {
    if (backdrop.has(i) && b.place !== undefined) placeGone.add(b.place);
  });
  const placeNumber = new Map<number, number>();
  places.forEach((_, i) => !placeGone.has(i) && placeNumber.set(i, placeNumber.size));
  const keptPlaces = places.filter((_, i) => !placeGone.has(i)).map((q) => (q.plot === undefined ? q : { ...q, plot: renumber.get(q.plot)! }));
  const keptBackdrop = (open.backdrop ?? []).filter((_, i) => !backdrop.has(i)).map((b) => (b.place === undefined ? b : { ...b, place: placeNumber.get(b.place)! }));
  return { ...draft, slots, open: { ...open, pads, ...(open.places ? { places: keptPlaces } : {}), ...(open.backdrop ? { backdrop: keptBackdrop } : {}) } };
}
