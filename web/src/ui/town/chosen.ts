/**
 * A building clicked in the play view (`Renderer.buildingAt`) as the town card's choice (`townHud.chosen`): what it is,
 * its icon, where its pin stands, who is inside now, and a building's flats (a building of flats', those over a
 * place), each with who lives there or its price. Pure: `chosenView.svelte.ts` gives it the game's views.
 */

import { lotOf, type FlatInfo, type HouseholdInfo, type PlotInfo, type TownPlace } from '../../core/protocol';
import { CIVIC, civicOf } from '../../game/backdrop';
import type { BuildingHit } from '../../render/types';
import { flatsIn, lotTitle } from '../place';
import { openState, openText, type HomeForSale } from './places';
import { whereabouts, type TagWorld } from './tags';
import type { Chosen } from './townHud.svelte';

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

export interface ChosenTown {
  places: readonly TownPlace[];
  plots: readonly PlotInfo[];
  /** The town's flats (`WorldStructure.flats`). */
  flats?: readonly FlatInfo[];
  households: readonly Pick<HouseholdInfo, 'name' | 'plot'>[];
}

/** What a click on building `hit` chooses: its lot's place (a public lot's), else a building of flats, else the lot's house. */
export function chosenOf(hit: BuildingHit, t: ChosenTown): Chosen | null {
  const p = t.plots[hit.plot];
  if (!p) return null;
  const place = t.places.findIndex((pl) => pl.plot === hit.plot);
  if (place >= 0) return { kind: 'place', place, plot: hit.plot, title: t.places[place].name, detail: '' };
  if (p.kind === 'flats' || flatsIn(t, p.id).length) return { kind: 'flats', plot: hit.plot, title: p.name, detail: '' };
  return { kind: 'home', plot: hit.plot, title: lotTitle(p, t.households, 'Empty house'), detail: '' };
}

/** A flat on its building's card: its name there ("Flat 3"), its storey, who lives there ("The Coles"; null: nobody) and, for sale, its price. */
export interface FlatRow {
  plot: number;
  name: string;
  storey: number;
  who: string | null;
  /** Its price while it is for sale (`places.ts` `homesForSale`), else null. */
  price: number | null;
}

/** The flats of lot `lot` as its card lists them, by storey (`sale`: the homes for sale, priced). */
export function flatRows(lot: number, t: ChosenTown, sale: readonly HomeForSale[]): FlatRow[] {
  const building = t.plots[lot]?.name;
  return flatsIn(t, lot).map((f) => {
    const hh = t.households.find((h) => h.plot === f.id);
    // (Its building's name goes without saying on its building's card.)
    const name = building && f.name.endsWith(`, ${building}`) ? f.name.slice(0, -building.length - 2) : f.name;
    return { plot: f.id, name, storey: f.storey, who: hh ? `The ${hh.name}s` : null, price: hh ? null : (sale.find((s) => s.plot === f.id)?.price ?? null) };
  });
}

/** A building of flats' card line: "4 flats · 1 for sale". */
export function flatsDetail(rows: readonly FlatRow[]): string {
  const free = rows.filter((r) => r.who === null).length;
  return [`${rows.length} ${rows.length === 1 ? 'flat' : 'flats'}`, free ? `${free} for sale` : ''].filter(Boolean).join(' · ');
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
export function chosenIcon(c: Chosen, t: Pick<ChosenTown, 'places'>): string {
  if (c.kind === 'place') return KIND_ICON[t.places[c.place]?.kind] ?? 'icon.ui.town';
  if (c.kind === 'home' || c.kind === 'flats') return 'icon.ui.home';
  return 'icon.ui.face';
}

/** Where the choice's pin stands: over its building's middle (town metres), `lift` m over the ground; null for a person. */
export function chosenAnchor(c: Chosen, t: Pick<ChosenTown, 'places' | 'plots'>): { x: number; z: number; lift: number } | null {
  if (c.kind === 'place') {
    const p = t.places[c.place];
    return p ? { x: p.x, z: p.z, lift: 7 } : null;
  }
  if (c.kind === 'home' || c.kind === 'flats') {
    const p = t.plots[c.plot];
    if (!p) return null;
    const [x0, z0, x1, z1] = p.house ?? [p.x, p.z, p.x + p.w, p.z + p.d];
    return { x: (x0 + x1) / 2, z: (z0 + z1) / 2, lift: 3 + 3 * (p.storeys ?? 1) };
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
 * them), in the house (on its lot, not away), or in a building of flats (in its flats, its hall or its yard). Null
 * for a person.
 */
export function whoIsIn(c: Chosen, w: TagWorld, yours: ReadonlyMap<number, string>): Inside | null {
  if (c.kind === 'person') return null;
  const out: Inside = { names: [], others: 0 };
  for (const s of w.sims) {
    const at = c.kind === 'flats' && s.plot !== null ? lotOf(w, s.plot) : s.plot;
    const there = c.kind === 'place' ? whereabouts(s, w).place === c.place : at === c.plot && s.awayUntil === null;
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
