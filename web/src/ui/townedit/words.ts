/**
 * The town editor's words for what it shows: what a lot or building is, what a removal takes with
 * it (the street card lists it first; the Remove tool says it as the pointer goes over a street).
 */

import type { NeighbourhoodDraft } from '../../game/town';

/** A sentence from an edit's reason (they are written as clauses: "it bends tighter than 9 m"). */
export const sentence = (why: string) => {
  const s = why.trim();
  return s ? `${s.charAt(0).toUpperCase()}${s.slice(1)}${/[.!?]$/.test(s) ? '' : '.'}` : s;
};

/** How many of a thing, in words ("1 home", "3 homes"). */
export const count = (n: number, one: string, many = `${one}s`) => `${n} ${n === 1 ? one : many}`;

/** A list in words: "a", "a and b", "a, b and c". */
export function listed(items: readonly string[]): string {
  return items.length < 2 ? (items[0] ?? '') : `${items.slice(0, -1).join(', ')} and ${items[items.length - 1]}`;
}

/** What a lot is, in a word or two: a home, a farm, a public place's or an outdoor spot's kind. */
export function lotKind(s: NeighbourhoodDraft['slots'][number]): string {
  if (s.kind === 'house') return s.lot === 'farm' ? 'Farm' : s.lot === 'terrace' ? 'Terraced home' : `${s.lot ? s.lot.charAt(0).toUpperCase() + s.lot.slice(1) : 'A'} home`;
  if (s.kind === 'outdoor') return 'Outdoor spot';
  return s.kind === 'shop' ? 'Shop' : 'Public place';
}

/** What a removal takes besides the thing itself: other roads, homes, other lots, buildings, parking (`streets.ts` `whatGoes`). */
export function whatGoesLine(draft: NeighbourhoodDraft, goes: { roads: number[]; lots: number[]; backdrop: number[]; parking: number[] }, road: number): string[] {
  const roads = goes.roads.filter((r) => r !== road).map((r) => draft.roads[r]?.name ?? 'a road');
  const homes = goes.lots.filter((i) => draft.slots[i]?.kind === 'house').length;
  const others = goes.lots.length - homes;
  return [
    ...(roads.length ? [`${count(roads.length, 'road')} left joined to nothing (${listed(roads)})`] : []),
    ...(homes ? [count(homes, 'home')] : []),
    ...(others ? [count(others, 'other lot')] : []),
    ...(goes.backdrop.length ? [count(goes.backdrop.length, 'building')] : []),
    ...(goes.parking.length ? [count(goes.parking.length, 'parking space')] : []),
  ];
}
