/**
 * The town's people: the households that live in its homes (`populate`), each in a home with room
 * for it that it could buy, another for a home on request (`rehouse`); and, to come, the seeded
 * streams they are drawn from (`peopleStreams`, from the world code's people seed), the residents
 * step 3 offers as households to play (`offered`), and the homes still for sale filled once the
 * player has moved in (`fillTown`). Moved here from `town.ts`, which still exports what it did.
 */

import type { AssetRegistry } from '../assets/registry';
import type { Content } from '../content/content';
import { bedroomsNeeded, randomHousehold, residentsOf, type HouseholdDraft } from './household';
import { homePrice } from './price';
import { houseTemplate, shuffled, type NeighbourhoodDraft, type PlotSlot, type Templates } from './town';
import { savingsOf } from './wealth';

type Rand = () => number;

/**
 * Residents the neighbours number at most (the sim holds 240 Sims in
 * a big world; room left for the player's household and babies).
 */
export const NEIGHBOUR_CAP = 220;

/**
 * Neighbours a big world starts with when nobody asks for a number (~120
 * residents; the rest of its homes are for sale, and newcomers fill the town from there). Tried at 168
 * in the living-town run and put back: with ~170 from the first morning, a sample world's centre jammed.
 */
const BIG_START = 118;

/** Neighbours a world starts with once the player has moved in (`fillTown`): room left under the sim's 240 for the player's household, babies and newcomers. */
export const START_CAP = 150;

/** The homes left for sale once the town is filled (`fillTown`), of `homes` in all: one in twenty, two at least. */
export function keepForSale(homes: number): number {
  return Math.max(2, Math.round(homes * 0.05));
}

/**
 * The people's random streams, all from one `seed`: the order homes are filled in, one stream per
 * lot keyed by its position (whichever wave fills it, its household is the same), the residents
 * step 3 offers, and the seeds of the neighbours' bonds and the sim's own dice. Only its shape is
 * here so far.
 */
export function peopleStreams(_seed: number): { order: Rand; lot(x: number, z: number): Rand; offer: Rand; bonds: number; sim: number } {
  throw new Error('peopleStreams: not built yet');
}

/** The slots of the resident households step 3 offers to play (`most` at most), in the order it shows them. Only its shape is here so far. */
export function offered(_town: NeighbourhoodDraft, _streams: ReturnType<typeof peopleStreams>, _most?: number): number[] {
  throw new Error('offered: not built yet');
}

/** `town` with the homes still for sale filled once the player has moved into `playerSlot` (up to `START_CAP` neighbours, `keepForSale` of them left). Only its shape is here so far. */
export function fillTown(_content: Content, _assets: AssetRegistry, _t: Templates, _town: NeighbourhoodDraft, _playerSlot: number, _streams: ReturnType<typeof peopleStreams>): NeighbourhoodDraft {
  throw new Error('fillTown: not built yet');
}

/** Tries at a household for a home: one that fits it and can buy it, not too rich for it. */
const HOUSEHOLD_TRIES = 36;
/** Savings beyond this many times a home's price make a household too well off for it (taken only when nothing closer turns up). */
const RICH_FOR = 4;

/**
 * Neighbours for a town: `count` households (default: all but two houses empty), each in a home
 * that has room for it and that it could buy (`wealth.ts` `savingsOf` covers `price.ts`
 * `homePrice`), no more than `MAX_NEIGHBOURS` residents in all. The houses are picked at random
 * and filled dearest first; one no household turns up for stays for sale.
 */
export function populate(content: Content, assets: AssetRegistry, t: Templates, town: NeighbourhoodDraft, count?: number): NeighbourhoodDraft {
  // (A big world starts with about 120 residents unless asked for more, up to its cap.)
  const MAX_NEIGHBOURS = count === undefined ? BIG_START : NEIGHBOUR_CAP;
  const houses = town.slots.filter((s) => s.kind === 'house');
  const wanted = Math.max(0, Math.min(count ?? houses.length - 2, houses.length - 1));
  const price = new Map(houses.map((s) => [s.index, homePrice(content, t, s)]));
  const dearestFirst = (list: PlotSlot[]) => [...list].sort((a, b) => price.get(b.index)! - price.get(a.index)!);
  // The houses picked first; the others stand in for any of them nobody could buy. The town's
  // cheapest home stays for sale (if it can): a starter home within reach of a young household.
  const cheapest = houses.reduce<PlotSlot | undefined>((a, s) => (!a || price.get(s.index)! < price.get(a.index)! ? s : a), undefined);
  const order = [...shuffled(houses.filter((s) => s !== cheapest)), ...(cheapest ? [cheapest] : [])];
  const queue = [...dearestFirst(order.slice(0, wanted)), ...dearestFirst(order.slice(wanted))];
  const taken = new Set<string>();
  const households: NeighbourhoodDraft['households'] = [];
  let residents = 0;
  for (const slot of queue) {
    if (households.length >= wanted || residents >= MAX_NEIGHBOURS) break;
    const bedrooms = houseTemplate(t, slot.template)?.bedrooms ?? 1;
    const household = householdFor(content, assets, bedrooms, price.get(slot.index)!, MAX_NEIGHBOURS - residents, [...taken]);
    if (!household) continue;
    taken.add(household.name);
    residents += household.members.length;
    households.push({ household, slot: slot.index });
  }
  return { ...town, households };
}

/**
 * A household for a home with `bedrooms` that costs `price`: it fits (a couple shares a bedroom,
 * two children may, anyone else needs one), has saved enough to buy it, and preferably isn't
 * many times richer than it needs to be. A dear home looks mostly among older households, who
 * have climbed higher and put more by. Null if none turns up.
 */
export function householdFor(
  content: Content,
  assets: AssetRegistry,
  bedrooms: number,
  price: number,
  room: number,
  avoidNames: readonly string[],
): HouseholdDraft | null {
  let size = Math.max(1, Math.min(4, bedrooms + Math.floor(Math.random() * 2), room));
  let best: { household: HouseholdDraft; savings: number } | null = null;
  // A home dearer than two young grown-ups between jobs could buy goes mostly to an older household.
  const young = (content.adultStages[0]?.from ?? 18) + 7;
  const dear = price > savingsOf(content, { members: [{ age: young }, { age: young }] });
  for (let tries = 0, misfits = 0; tries < HOUSEHOLD_TRIES; tries++) {
    const household = randomHousehold(content, assets, size, avoidNames, { older: dear ? Math.random() < 0.8 : Math.random() < 0.2 });
    if (bedroomsNeeded(residentsOf(content, household)) > bedrooms) {
      if (++misfits % 6 === 0 && size > 1) size--;
      continue;
    }
    const savings = savingsOf(content, household);
    if (savings < price) continue;
    if (savings <= Math.max(price, 1) * RICH_FOR) return household;
    if (!best || savings < best.savings) best = { household, savings };
  }
  return best?.household ?? null;
}

/**
 * Another household for the home on `slot` (one that fits it and can buy it, as `populate` finds
 * them), with a name none of `avoidNames` has; any household of `size` if none turns up.
 */
export function rehouse(content: Content, assets: AssetRegistry, t: Templates, town: NeighbourhoodDraft, slot: number, size: number, avoidNames: readonly string[]): HouseholdDraft {
  const s = town.slots[slot];
  const bedrooms = (s && houseTemplate(t, s.template)?.bedrooms) ?? 1;
  return (s && householdFor(content, assets, bedrooms, homePrice(content, t, s), NEIGHBOUR_CAP, avoidNames)) ?? randomHousehold(content, assets, size, avoidNames);
}
