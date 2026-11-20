/**
 * The town editor's words for what it shows: what a lot is (a home, a building of flats, a place with
 * flats over it...), what a removal takes with it (the street card lists it first; the Remove tool
 * says it as the pointer goes over a street).
 */

import { CIVIC, civicLabel, draftAreaSize } from '../../game/places';
import { placeOfTemplate } from '../../game/placeLots';
import type { Goes } from '../../game/edit/streets';
import { homesOn, type NeighbourhoodDraft, type PlotSlot } from '../../game/town';

/** What a named place's lot of another kind than a civic place is (`places.ts` `ZONE`). */
const ZONE_KIND: Record<string, string> = { shop: 'Shop', cafe: 'Café', restaurant: 'Restaurant', office: 'Offices', workshop: 'Workshop', farm: 'Farm shop' };

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

/**
 * What a lot is, in a word or two: a home, a farm, a building of flats, a public place's or an outdoor spot's kind, a named
 * place's (a civic place's called as one of its area's size in `draft`: a village's hall its village hall; with flats over
 * it, said so).
 */
export function lotKind(s: NeighbourhoodDraft['slots'][number], draft?: NeighbourhoodDraft): string {
  const what = s.kind === 'shop' ? placeOfTemplate(s.template) : null;
  if (what) {
    const kind = 'civic' in what ? civicLabel(CIVIC[what.civic], draft ? draftAreaSize(draft, s.area ?? 0) : 'town') : (ZONE_KIND[what.zone] ?? 'Shop');
    return s.upper?.startsWith('over:') ? `${kind}, flats over it` : kind;
  }
  if (s.kind === 'flats') return 'Flats';
  if (s.kind === 'house') return s.lot === 'farm' ? 'Farm' : s.lot === 'terrace' ? 'Terraced home' : s.garden === 'tight' ? 'Town house' : `${s.lot ? s.lot.charAt(0).toUpperCase() + s.lot.slice(1) : 'A'} home`;
  if (s.kind === 'outdoor') return 'Outdoor spot';
  return s.kind === 'shop' ? 'Shop' : 'Public place';
}

/** The flats lot `s` holds, in words ("its 6 flats", "the 2 flats over it"); null: none (a house, a place with nothing over it). */
export function flatsLine(s: Pick<PlotSlot, 'kind' | 'template' | 'upper' | 'lot'>): string | null {
  if (s.kind === 'house') return null;
  const n = homesOn(s, { farms: true });
  if (!n) return null;
  return s.kind === 'flats' ? `its ${count(n, 'flat')}` : `the ${count(n, 'flat')} over it`;
}

/**
 * What a removal takes besides the thing itself (`streets.ts` `whatGoes`): other roads, bridges left leading nowhere,
 * homes (a house, each flat of a building of flats or over a place), other lots, parking spaces (their car parks whole).
 */
export function whatGoesLine(draft: NeighbourhoodDraft, goes: Pick<Goes, 'roads' | 'lots' | 'parking'> & Partial<Pick<Goes, 'bridges'>>, road: number): string[] {
  const bridged = new Set(goes.bridges ?? []);
  const name = (r: number) => draft.roads[r]?.name ?? 'a road';
  const roads = goes.roads.filter((r) => r !== road && !bridged.has(r)).map(name);
  const bridges = [...bridged].filter((r) => r !== road).map(name);
  const homes = goes.lots.reduce((n, i) => n + (draft.slots[i] ? homesOn(draft.slots[i], { farms: true }) : 0), 0);
  const others = goes.lots.filter((i) => draft.slots[i] && draft.slots[i].kind !== 'house' && draft.slots[i].kind !== 'flats').length;
  return [
    ...(roads.length ? [`${count(roads.length, 'road')} left joined to nothing (${listed(roads)})`] : []),
    ...(bridges.length ? [`${count(bridges.length, 'bridge')} left leading nowhere (${listed(bridges)})`] : []),
    ...(homes ? [count(homes, 'home')] : []),
    ...(others ? [count(others, 'other lot')] : []),
    ...(goes.parking.length ? [count(goes.parking.length, 'parking space')] : []),
  ];
}

/** What a removal makes of the country roads it leaves a long dead end (`Goes.streets`), as a sentence; null: none. */
export function madeStreetsLine(draft: NeighbourhoodDraft, goes: Pick<Goes, 'streets'>): string | null {
  const names = goes.streets.map((r) => draft.roads[r]?.name ?? 'a country road');
  if (!names.length) return null;
  return `${listed(names)}, left ending past a court, ${names.length === 1 ? 'becomes a street' : 'become streets'} (no taxi turns round in a country road).`;
}
