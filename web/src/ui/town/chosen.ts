/**
 * A building clicked in the play view (`Renderer.buildingAt`) as the town card's choice (`townHud.chosen`): what it is,
 * its icon, where its pin stands, and who is inside now. Pure: `chosenView.svelte.ts` gives it the game's views.
 */

import type { BackdropBuilding, BuildingUse, HouseholdInfo, PlotInfo, TownPlace } from '../../core/protocol';
import { CIVIC, civicOf } from '../../game/places';
import type { BuildingHit } from '../../render/types';
import { lotTitle } from '../place';
import { openState, openText } from './places';
import { whereabouts, type TagWorld } from './tags';
import type { Chosen } from './townHud.svelte';

/** A backdrop building that is no place, in words. */
const USE_TITLE: Record<BuildingUse, string> = {
  flats: 'Flats',
  terrace: 'Terraced houses',
  house: 'House',
  shop: 'Shop',
  cafe: 'Café',
  restaurant: 'Restaurant',
  office: 'Offices',
  school: 'School',
  church: 'Church',
  hall: 'Hall',
  workshop: 'Workshop',
  farm: 'Farm',
  barn: 'Barn',
};

/** A place's icon, by its kind. */
const KIND_ICON: Record<TownPlace['kind'], string> = {
  shop: 'icon.ui.buy',
  cafe: 'icon.need.hunger',
  restaurant: 'icon.career.culinary',
  office: 'icon.career.business',
  school: 'icon.career.education',
  church: 'icon.category.community',
  park: 'icon.category.garden',
  outdoor: 'icon.category.outdoor',
  workshop: 'icon.career.trades',
  farm: 'icon.career.nature',
};

/** A backdrop building's icon, by its use. */
const USE_ICON: Record<BuildingUse, string> = {
  flats: 'icon.ui.home',
  terrace: 'icon.ui.home',
  house: 'icon.ui.home',
  shop: KIND_ICON.shop,
  cafe: KIND_ICON.cafe,
  restaurant: KIND_ICON.restaurant,
  office: KIND_ICON.office,
  school: KIND_ICON.school,
  church: KIND_ICON.church,
  hall: KIND_ICON.church,
  workshop: KIND_ICON.workshop,
  farm: KIND_ICON.farm,
  barn: KIND_ICON.farm,
};

export interface ChosenTown {
  places: readonly TownPlace[];
  backdrop: readonly BackdropBuilding[];
  plots: readonly PlotInfo[];
  households: readonly Pick<HouseholdInfo, 'name' | 'plot'>[];
}

/** What a click on building `hit` chooses: its place (a public lot's, a backdrop place's), else the lot's house or the building. */
export function chosenOf(hit: BuildingHit, t: ChosenTown): Chosen | null {
  if (hit.kind === 'lot') {
    const p = t.plots[hit.plot];
    if (!p) return null;
    const place = t.places.findIndex((pl) => pl.plot === hit.plot);
    if (place >= 0) return { kind: 'place', place, plot: hit.plot, title: t.places[place].name, detail: '' };
    return { kind: 'home', plot: hit.plot, title: lotTitle(p, t.households, 'Empty house'), detail: '' };
  }
  const b = t.backdrop[hit.index];
  if (!b) return null;
  const place = b.place === undefined ? undefined : t.places[b.place];
  if (place && b.place !== undefined) return { kind: 'place', place: b.place, plot: place.plot ?? null, title: place.name, detail: '' };
  return { kind: 'building', building: hit.index, plot: null, title: USE_TITLE[b.use] ?? 'Building', detail: '' };
}

/**
 * What a card calls a civic place (`game/backdrop.ts` `CIVIC`: "Fire station", "School"; a village's hall its
 * "Village hall", as its name says), '' for any other: a shop's or a café's name says what it is.
 */
export function placeLabel(place: TownPlace | undefined): string {
  const c = CIVIC[civicOf(place)];
  if (!c || !place) return '';
  const name = place.name.toLowerCase();
  return Object.values(c.labels ?? {}).find((l) => name.includes(l.toLowerCase())) ?? c.label;
}

/**
 * A place's card line: what it is (`placeLabel`) and whether it's open at `minute` of `weekday` with `hours` (its
 * lot's: `places.ts` `lotHours`): "Fire station · open till 17:00", "School · closed · opens Mon 08:00", "open till 18:00".
 */
export function placeDetail(place: TownPlace | undefined, hours: TownPlace['hours'] | undefined, minute: number, weekday: number, time: (minute: number) => string, days: readonly string[]): string {
  return [placeLabel(place), openText(openState(hours, minute, weekday), weekday, time, days)].filter(Boolean).join(' · ');
}

/** The choice's icon (`Icon` name). */
export function chosenIcon(c: Chosen, t: Pick<ChosenTown, 'places' | 'backdrop'>): string {
  if (c.kind === 'place') return KIND_ICON[t.places[c.place]?.kind] ?? 'icon.ui.town';
  if (c.kind === 'building') return USE_ICON[t.backdrop[c.building]?.use] ?? 'icon.ui.town';
  if (c.kind === 'home') return 'icon.ui.home';
  return 'icon.ui.face';
}

/** Where the choice's pin stands: over its building's middle (town metres), `lift` m over the ground; null for a person. */
export function chosenAnchor(c: Chosen, t: Pick<ChosenTown, 'places' | 'backdrop' | 'plots'>): { x: number; z: number; lift: number } | null {
  if (c.kind === 'place') {
    const p = t.places[c.place];
    return p ? { x: p.x, z: p.z, lift: 7 } : null;
  }
  if (c.kind === 'home') {
    const p = t.plots[c.plot];
    if (!p) return null;
    const [x0, z0, x1, z1] = p.house ?? [p.x, p.z, p.x + p.w, p.z + p.d];
    return { x: (x0 + x1) / 2, z: (z0 + z1) / 2, lift: 3 + 3 * (p.storeys ?? 1) };
  }
  if (c.kind === 'building') {
    const b = t.backdrop[c.building];
    if (!b?.poly.length) return null;
    let x = 0;
    let z = 0;
    for (const [px, pz] of b.poly) {
      x += px;
      z += pz;
    }
    return { x: x / b.poly.length, z: z / b.poly.length, lift: 3 * b.storeys + 3 };
  }
  return null;
}

/** Who is in it now: yours (household and your people: id → name) by name, how many others. */
export interface Inside {
  names: string[];
  others: number;
}

/**
 * Who is inside the choice now: at the place (on its lot, or working there: `whereabouts`, as the place tags count
 * them), or in the house (on its lot, not away). Null for a person, or a building nobody is simulated in.
 */
export function whoIsIn(c: Chosen, w: TagWorld, yours: ReadonlyMap<number, string>): Inside | null {
  if (c.kind !== 'place' && c.kind !== 'home') return null;
  const out: Inside = { names: [], others: 0 };
  for (const s of w.sims) {
    const there = c.kind === 'place' ? whereabouts(s, w).place === c.place : s.plot === c.plot && s.awayUntil === null;
    if (!there) continue;
    const name = yours.get(s.id);
    if (name) out.names.push(name);
    else out.others++;
  }
  return out;
}

/** `whoIsIn` in words: "Mika, Ines and 3 others inside", "Nobody inside now". */
export function insideText(inside: Inside): string {
  const { names, others } = inside;
  if (!names.length && !others) return 'Nobody inside now';
  const parts = [...names];
  if (others) parts.push(`${others} ${names.length ? (others === 1 ? 'other' : 'others') : others === 1 ? 'person' : 'people'}`);
  const list = parts.length > 1 ? `${parts.slice(0, -1).join(', ')} and ${parts[parts.length - 1]}` : parts[0];
  return `${list} inside`;
}
