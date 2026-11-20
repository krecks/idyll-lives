/**
 * The homes of a new game's town as steps 4 and 5 see them. A home is a house's lot, or one flat of a
 * building of flats or of the flats over a place (`HomeRef`: the building's slot and the flat's index in
 * its template's `units`, as `NeighbourhoodDraft.households` and the town file's `AssembleOptions.flat`
 * name it). Which homes are for sale, what each is called and costs, and the list Choose Home shows,
 * a building's flats together. Pure, so the rules can be tested without a page.
 */

import type { Content } from '../content/content';
import { flatName, flatsTemplate } from '../game/flats';
import { homesOf } from '../game/people';
import { flatsIdOf, type NeighbourhoodDraft, type PlotSlot, type Templates } from '../game/town';

/** A home in a draft: a house's lot (`slot`), or flat `flat` of the building on it. */
export interface HomeRef {
  slot: number;
  flat?: number;
}

/** A home's key, one string a home: "12" for a house's lot, "12:3" for flat 3 of the building on lot 12. */
export function homeKey(h: HomeRef): string {
  return h.flat === undefined ? `${h.slot}` : `${h.slot}:${h.flat}`;
}

/** Whether `a` and `b` are the same home (a flat is not its building's lot). */
export function sameHome(a: HomeRef | null | undefined, b: HomeRef | null | undefined): boolean {
  return !!a && !!b && a.slot === b.slot && a.flat === b.flat;
}

/** A home as Choose Home offers it: where it is, what it is called, which storey a flat is on, its bedrooms and its price (`people.ts` `homesOf`). */
export interface HomeOffer extends HomeRef {
  key: string;
  /** The lot it is on (a flat's building's). */
  lot: PlotSlot;
  /** "Flat 2, Mill Court" for a flat, as the game names it; a house's lot's own name. */
  name: string;
  /** A flat's storey (0: the ground floor). */
  storey?: number;
  bedrooms: number;
  price: number;
}

/** A storey in words: "Ground floor", "Floor 1". */
export function storeyName(storey: number): string {
  return storey <= 0 ? 'Ground floor' : `Floor ${storey}`;
}

/** Every home of `town` (`people.ts` `homesOf`): each house's lot and each flat, with its name and a flat's storey. */
export function homeOffers(content: Content, t: Templates, town: NeighbourhoodDraft): HomeOffer[] {
  return homesOf(content, t, town).map((h) => {
    const ref: HomeRef = { slot: h.slot.index, ...(h.flat === undefined ? {} : { flat: h.flat }) };
    if (h.flat === undefined) return { ...ref, key: homeKey(ref), lot: h.slot, name: h.slot.name, bedrooms: h.bedrooms, price: h.price };
    const storey = flatsTemplate(t, flatsIdOf(h.slot))?.units[h.flat]?.storey ?? 0;
    return { ...ref, key: homeKey(ref), lot: h.slot, name: flatName(h.slot.name, h.flat), storey, bedrooms: h.bedrooms, price: h.price };
  });
}

/** Whether a household lives in home `h` (a household of a flat lives in that flat, not on its building's lot). */
export function livedIn(town: Pick<NeighbourhoodDraft, 'households'>, h: HomeRef): boolean {
  return town.households.some((x) => sameHome(x, h));
}

/** The homes of `town` nobody lives in: houses and flats for sale (`homeOffers`). */
export function homesForSale(content: Content, t: Templates, town: NeighbourhoodDraft): HomeOffer[] {
  const lived = new Set(town.households.map(homeKey));
  return homeOffers(content, t, town).filter((h) => !lived.has(h.key));
}

/**
 * Choose Home's list: the household's own home first (`from`), then the main town's homes, then each village's,
 * cheapest first; a building's flats together, where its cheapest flat would come, in storey order.
 */
export function homeRows<H extends HomeOffer>(homes: readonly H[], from: HomeRef | null): H[] {
  const cheapest = new Map<number, number>();
  for (const h of homes) if (h.flat !== undefined && !sameHome(h, from)) cheapest.set(h.slot, Math.min(cheapest.get(h.slot) ?? Infinity, h.price));
  const own = (h: H) => Number(sameHome(h, from));
  const price = (h: H) => (h.flat === undefined || own(h) ? h.price : cheapest.get(h.slot)!);
  return [...homes].sort(
    (a, b) =>
      own(b) - own(a) ||
      (a.lot.area ?? 0) - (b.lot.area ?? 0) ||
      price(a) - price(b) ||
      a.slot - b.slot ||
      (a.storey ?? 0) - (b.storey ?? 0) ||
      (a.flat ?? 0) - (b.flat ?? 0),
  );
}

/**
 * The home a click on lot `slot` chooses among `homes` (those to choose from): the one chosen already if it is on that
 * lot, else the household's own (`from`), else the cheapest the household can afford (`dear` says which it can't),
 * else the cheapest. Null if none of them is on the lot.
 */
export function homeOnLot<H extends HomeOffer>(homes: readonly H[], slot: number, chosen: HomeRef | null, from: HomeRef | null, dear: (h: H) => boolean): H | null {
  const here = homes.filter((h) => h.slot === slot);
  if (!here.length) return null;
  const pick = here.find((h) => sameHome(h, chosen)) ?? here.find((h) => sameHome(h, from));
  if (pick) return pick;
  const byPrice = [...here].sort((a, b) => a.price - b.price || (a.flat ?? 0) - (b.flat ?? 0));
  return byPrice.find((h) => !dear(h)) ?? byPrice[0];
}
