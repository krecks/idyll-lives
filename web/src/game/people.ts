/**
 * The town's people: the households that live in its homes, drawn from seeded streams so the world
 * code's people seed makes the same people every time (`peopleStreams`).
 *
 * They come in two waves. `populate` (on entering step 3) fills the homes as the town has always
 * been filled, leaving a quarter of them for the player to choose from. `offered` picks the
 * residents step 3 offers as households to play. `fillTown` (on Move in) gives every home still for
 * sale a household, but for a few of the cheapest (`keepForSale`), up to `FILL_CAP` neighbours, or
 * fewer homes on a machine that holds fewer (`FillTier`); `fillPlan` says ahead of time who those
 * households are, so their houses can be furnished early (`townJobs.ts` `furnishAhead`).
 * `neighbourBonds` rolls who among the neighbours already know each other. `rehouse` finds another
 * household for one home. Moved here from `town.ts`, which still exports what it did.
 */

import type { AssetRegistry } from '../assets/registry';
import type { Content } from '../content/content';
import { bedroomsNeeded, randomHousehold, residentsOf, type HouseholdDraft } from './household';
import { mulberry32 } from './noise';
import { homePrice } from './price';
import { houseTemplate, shuffled, type NeighbourhoodDraft, type PlotSlot, type Templates } from './town';
import { savingsOf } from './wealth';

type Rand = () => number;

/**
 * Residents the neighbours number at most, when someone asks for a number (the sim holds 768
 * residents, sim-core `MAX_SIMS`; room left for the player's household and babies).
 */
export const NEIGHBOUR_CAP = 720;

/**
 * Neighbours a big world starts with when nobody asks for a number (~120
 * residents; the rest of its homes are for sale, and newcomers fill the town from there). Tried at 168
 * in the living-town run and put back: with ~170 from the first morning, a sample world's centre jammed.
 */
const BIG_START = 118;

/** Homes the first wave leaves for sale at the least, for the player to choose from (and a quarter of them if that is more). */
const LEFT_TO_CHOOSE = 6;

/**
 * The most households the first wave moves into a town of `homes` homes: all of them but those it
 * leaves for the player to choose from (a quarter, six at least). `populate` clamps a count to it,
 * and step 3's Neighbours slider stops at it.
 */
export function householdsAtMost(homes: number): number {
  return Math.max(0, homes - Math.min(homes, Math.max(LEFT_TO_CHOOSE, Math.ceil(homes / 4))));
}

/** The residents the simulation holds at most (sim-core `MAX_SIMS`): what a town filled in full may grow to. */
export const SIM_RESIDENTS = 768;

/**
 * Neighbours a world starts with at most once the player has moved in (`fillTown`, in full). The
 * biggest worlds have about 390 homes, which their natural households would fill with about 850
 * people; the smallest homes then take one or two each. It leaves 88 of the sim's 768 for the
 * player's household, babies and newcomers.
 */
export const FILL_CAP = 680;

/**
 * How full the second wave fills the town, for the machine the game runs on (`tierFor`). Each home's
 * household comes from its own lot's stream, so a smaller tier is the same people in fewer homes:
 * - `full`: every home but the few kept for sale, up to `FILL_CAP` neighbours;
 * - `half`: the main town and the player's own village in full, every other village's homes half
 *   lived in, up to 500;
 * - `near`: the homes within 250 m of the player's in full, a third of the others in each area,
 *   up to 340.
 */
export type FillTier = 'full' | 'half' | 'near';

/** Each tier's neighbours at most (`fill`), and the most residents its town grows to by births and newcomers (`most`: the town file's `residentsMost`). */
const TIERS: Record<FillTier, { fill: number; most: number }> = {
  full: { fill: FILL_CAP, most: SIM_RESIDENTS },
  half: { fill: 500, most: 576 },
  near: { fill: 340, most: 384 },
};

/** The most residents a town filled at `tier` grows to by births and newcomers (the town file's `residentsMost`). */
export function residentsMost(tier: FillTier = 'full'): number {
  return TIERS[tier].most;
}

/**
 * The tier each graphics preset fills the town at (`render/quality.ts` `GraphicsPreset`). A preset
 * gets a tier when the sim's wasm build, with the camera's 320-tile square in full detail as the
 * game streams it, runs at least 4,250 ticks a second on the slowest machine the preset is for
 * (the fastest speed needs 3,600); with none that does, the smallest. Measured on 2026-11-17 on an
 * M3 Pro over a game day of the seaside, river and forest cities (`tools/m1/run.sh`,
 * `tools/bench/simtick.mjs`, Node), the wasm ran at most about 2,800 ticks a second of CPU time
 * near, 1,100 half and 800 full; an M1 (low and medium's floor) at 0.72 of that is about 2,000,
 * 800 and 580. No tier reaches 4,250 on either machine, so every preset fills the town near the
 * player's home; the fastest speed runs slow on all of them, the full town most (the machine was
 * shared and busy: a quiet run of `tools/m1/run.sh` may move these, not past 4,250).
 */
const TIER_FOR: Record<string, FillTier> = { low: 'near', medium: 'near', high: 'near' };

/** The tier to fill the town at on a machine with the graphics preset `preset` (full for one this does not know). */
export function tierFor(preset: string | undefined): FillTier {
  return (preset && TIER_FOR[preset]) || 'full';
}

/**
 * Births and newcomers' room over the residents a town starts with, in a town not filled by
 * `fillTown` (`defaultResidentsMost`): the room the full fill leaves.
 */
const GROWTH_ROOM = SIM_RESIDENTS - FILL_CAP;
/** The most residents every town grew to before the sim's limit was raised (0.8.2 and earlier). */
const OLD_LIMIT = 240;

/**
 * The most residents a town grows to when nothing filled it (`fillTown` sets its own): those it
 * starts with and room for births and newcomers, but never under the 240 every town could grow to
 * before, so a town of the first wave alone (the tools, Creation's path, older code) still fills
 * its empty homes with newcomers as it always did, and never swells past it unasked.
 */
export function defaultResidentsMost(residents: number): number {
  return Math.min(SIM_RESIDENTS, Math.max(OLD_LIMIT, residents + GROWTH_ROOM));
}

/** The homes left for sale once the town is filled (`fillTown`), of `homes` in all: one in twenty, two at least. */
export function keepForSale(homes: number): number {
  return Math.max(2, Math.round(homes * 0.05));
}

/**
 * The people's random streams, all from one seed (the world code's people seed). Each stream is
 * made afresh when asked for, from its start, so asking twice gives the same numbers twice.
 */
export interface PeopleStreams {
  /** The seed they come from. */
  seed: number;
  /**
   * Where the lot at (`x`, `z`) comes in the order the first wave picks homes in (lower first).
   * A number of the lot's own, so a lot added or taken away elsewhere moves no other in it.
   */
  order(x: number, z: number): number;
  /**
   * The household that lives on the lot at (`x`, `z`), whichever wave fills it. Keyed by the lot's
   * position to the metre, not its number, so a lot added or taken away elsewhere leaves it alone.
   */
  lot(x: number, z: number): Rand;
  /** Which residents step 3 offers, and in which order. */
  offer(): Rand;
  /** The seed of the neighbours' bonds (the assemble job's `random`). */
  bonds: number;
  /** The seed of the simulation's own dice, unless the page's `?seed=` gives one. */
  sim: number;
}

/** A 32-bit hash's last mixing step (MurmurHash3's), so nearby inputs land far apart. */
function scramble(h: number): number {
  h ^= h >>> 16;
  h = Math.imul(h, 0x85ebca6b);
  h ^= h >>> 13;
  h = Math.imul(h, 0xc2b2ae35);
  h ^= h >>> 16;
  return h >>> 0;
}

/** One 32-bit number from several whole numbers, their order counting. */
function mix(...parts: number[]): number {
  let h = 0x9e3779b9;
  for (const p of parts) h = scramble((h ^ scramble(p | 0)) + 0x6d2b79f5);
  return h;
}

/** What each stream is keyed by beside the seed. */
const ORDER = 1;
const LOT = 2;
const OFFER = 3;
const BONDS = 4;
const SIM = 5;

/** The people's streams from `seed` (see `PeopleStreams`). */
export function peopleStreams(seed: number): PeopleStreams {
  const base = seed >>> 0;
  return {
    seed: base,
    order: (x, z) => mix(base, ORDER, Math.round(x), Math.round(z)),
    lot: (x, z) => mulberry32(mix(base, LOT, Math.round(x), Math.round(z))),
    offer: () => mulberry32(mix(base, OFFER)),
    bonds: mix(base, BONDS),
    // (Never 0: the page takes 0 for "none given".)
    sim: mix(base, SIM) >>> 1 || 1,
  };
}

/** The people's streams, or `rand` standing in for all of them (Creation's dice: `Math.random`). */
type People = PeopleStreams | Rand;

/** The order and lot streams of `people`. */
function streamsOf(people: People): Pick<PeopleStreams, 'order' | 'lot'> {
  return typeof people === 'function' ? { order: () => people(), lot: () => people } : people;
}

/** Tries at a household for a home: one that fits it and can buy it, not too rich for it. */
const HOUSEHOLD_TRIES = 36;
/** Savings beyond this many times a home's price make a household too well off for it (taken only when nothing closer turns up). */
const RICH_FOR = 4;

/** A home's bedrooms (one for an empty lot). */
const bedroomsOf = (t: Templates, s: PlotSlot) => houseTemplate(t, s.template)?.bedrooms ?? 1;

/**
 * The first wave of neighbours: `count` households (default: as many as the town's homes allow,
 * up to `BIG_START` residents), each in a home that has room for it and that it could buy
 * (`wealth.ts` `savingsOf` covers `price.ts` `homePrice`). A quarter of the homes, six at least,
 * are left for sale for the player to choose from. The houses are picked in `people`'s order (each
 * lot's own place in it) and filled dearest first; one no household turns up for stays for sale. Each home's household comes
 * from its lot's own stream (or from `people` itself when it is a plain `Rand`: Creation's dice).
 */
export function populate(content: Content, assets: AssetRegistry, t: Templates, town: NeighbourhoodDraft, count?: number, people: People = Math.random): NeighbourhoodDraft {
  const streams = streamsOf(people);
  // (A big world starts with about 120 residents unless asked for more, up to its cap.)
  const MAX_NEIGHBOURS = count === undefined ? BIG_START : NEIGHBOUR_CAP;
  const houses = town.slots.filter((s) => s.kind === 'house');
  const wanted = Math.max(0, Math.min(count ?? houses.length, householdsAtMost(houses.length)));
  const price = new Map(houses.map((s) => [s.index, homePrice(content, t, s)]));
  const dearestFirst = (list: PlotSlot[]) => [...list].sort((a, b) => price.get(b.index)! - price.get(a.index)!);
  // The houses picked first; the others stand in for any of them nobody could buy. The town's
  // cheapest home stays for sale (if it can): a starter home within reach of a young household.
  const cheapest = houses.reduce<PlotSlot | undefined>((a, s) => (!a || price.get(s.index)! < price.get(a.index)! ? s : a), undefined);
  const rank = new Map(houses.map((s) => [s.index, streams.order(s.x, s.z)]));
  const picked = houses.filter((s) => s !== cheapest).sort((a, b) => rank.get(a.index)! - rank.get(b.index)! || a.x - b.x || a.z - b.z);
  const order = [...picked, ...(cheapest ? [cheapest] : [])];
  const queue = [...dearestFirst(order.slice(0, wanted)), ...dearestFirst(order.slice(wanted))];
  const taken = new Set<string>();
  const households: NeighbourhoodDraft['households'] = [];
  let residents = 0;
  for (const slot of queue) {
    if (households.length >= wanted || residents >= MAX_NEIGHBOURS) break;
    const household = householdFor(content, assets, bedroomsOf(t, slot), price.get(slot.index)!, MAX_NEIGHBOURS - residents, [...taken], streams.lot(slot.x, slot.z));
    if (!household) continue;
    taken.add(household.name);
    residents += household.members.length;
    households.push({ household, slot: slot.index });
  }
  return { ...town, households };
}

/** Residents step 3 offers at the least (all of them if there are fewer) and at the most. */
const OFFER_LEAST = 10;
const OFFER_MOST = 30;

/**
 * The slots of the resident households step 3 offers to play, in the order it shows them: all of
 * them if there are ten or fewer, else six in ten of them, between 10 and 30 (and `most` at most).
 * Chosen for variety: each next one is the household least like those already chosen (in its
 * size, whether it is a couple, has children or is siblings, how old its eldest is, the size of
 * its lot, and whether it lives in the main town), the offer's stream breaking ties.
 */
export function offered(town: NeighbourhoodDraft, streams: PeopleStreams, most = OFFER_MOST): number[] {
  const all = town.households;
  const wanted = Math.min(most, all.length <= OFFER_LEAST ? all.length : Math.max(OFFER_LEAST, Math.min(OFFER_MOST, Math.round(all.length * 0.6))));
  const traits = (h: (typeof all)[number]): string[] => {
    const { members, bonds } = h.household;
    const eldest = Math.max(0, ...members.map((m) => m.age ?? 0));
    const slot = town.slots[h.slot];
    return [
      `size ${members.length}`,
      `couple ${bonds.some((b) => b.preset === 'partners')}`,
      `children ${bonds.some((b) => b.preset === 'parent')}`,
      `siblings ${bonds.some((b) => b.preset === 'siblings')}`,
      `age ${eldest < 35 ? 'young' : eldest < 60 ? 'settled' : 'elder'}`,
      `lot ${slot?.lot ?? 'medium'}`,
      `town ${(slot?.area ?? 0) === 0}`,
    ];
  };
  const pool = shuffled(
    all.map((h) => ({ slot: h.slot, traits: traits(h) })),
    streams.offer(),
  );
  const seen = new Map<string, number>();
  const chosen: number[] = [];
  while (chosen.length < wanted && pool.length) {
    // (The first of the best, so the stream's order breaks ties.)
    let best = 0;
    let bestScore = -1;
    pool.forEach((p, i) => {
      const score = p.traits.reduce((sum, k) => sum + 1 / (1 + (seen.get(k) ?? 0)), 0);
      if (score > bestScore) [best, bestScore] = [i, score];
    });
    const [pick] = pool.splice(best, 1);
    for (const k of pick.traits) seen.set(k, (seen.get(k) ?? 0) + 1);
    chosen.push(pick.slot);
  }
  return chosen;
}

/**
 * How `fillPlan` and `fillTown` fill: at which tier (`FillTier`, full by default); the neighbours
 * at most in full (`FILL_CAP` by default; a smaller tier stops at its own limit when that is fewer);
 * and family names none of their households takes (the player's).
 */
export interface FillOptions {
  tier?: FillTier;
  cap?: number;
  avoid?: readonly string[];
}

/** Homes this near the player's (m), in the same area, are filled first: the player's own street, roughly. */
const STREET_NEAR = 80;
/** Households each village gets before the main town's homes are filled. */
const VILLAGE_FIRST = 2;
/** The `near` tier fills every home this near the player's (m), or the main town's middle before the player has chosen. */
const TIER_NEAR = 250;

/**
 * Who the second wave would move in, home by home (see `fillTown`): for every home still for sale
 * but the `keepForSale` cheapest and the player's (`playerSlot`; none yet when furnishing ahead,
 * before the player has chosen), a household from its lot's stream, while the neighbours number no
 * more than the cap. In the order they move in: the player's street first, then two in each
 * village, then the main town from the player's home (or its middle) outwards, then the villages'
 * and the countryside's other homes. Where the households the homes would naturally have would
 * pass the cap, the smallest homes take one or two people instead (smallest first, then the
 * cheapest), until they fit: every home lit counts for more than every bed taken. A home nobody
 * turns up for stays for sale. A tier below full then leaves some of those homes for sale (see
 * `FillTier`; which, by each lot's own place in the order) and stops at its own limit: the
 * households are drawn and sized as for the full fill first, so the people in the homes it fills
 * are the full fill's.
 */
export function fillPlan(
  content: Content,
  assets: AssetRegistry,
  t: Templates,
  town: NeighbourhoodDraft,
  playerSlot: number | undefined,
  streams: PeopleStreams,
  { tier = 'full', cap = FILL_CAP, avoid = [] }: FillOptions = {},
): { slot: number; household: HouseholdDraft }[] {
  const houses = town.slots.filter((s) => s.kind === 'house');
  const lived = new Set(town.households.map((h) => h.slot));
  const price = new Map(houses.map((s) => [s.index, homePrice(content, t, s)]));
  // (Ties by position, not number: a lot elsewhere coming or going changes no order here.)
  const byPlace = (a: PlotSlot, b: PlotSlot) => a.x - b.x || a.z - b.z;
  const forSale = houses.filter((s) => !lived.has(s.index) && s.index !== playerSlot);
  const kept = new Set(
    [...forSale]
      .sort((a, b) => price.get(a.index)! - price.get(b.index)! || byPlace(a, b))
      .slice(0, keepForSale(houses.length))
      .map((s) => s.index),
  );
  const homes = forSale.filter((s) => !kept.has(s.index));
  // The order they move in.
  const areas = town.open?.areas ?? [];
  const middle = (k: number): [number, number] => {
    const a = areas[k];
    return a ? [a.x + a.w / 2, a.z + a.d / 2] : [town.width / 2, town.depth / 2];
  };
  const player = playerSlot === undefined ? undefined : town.slots[playerSlot];
  const from: [number, number] = player ? [player.x, player.z] : middle(0);
  const far = (s: PlotSlot, [x, z]: [number, number]) => Math.hypot(s.x - x, s.z - z);
  const areaOf = (s: PlotSlot) => s.area ?? 0;
  const street = new Set(player ? homes.filter((s) => areaOf(s) === areaOf(player) && far(s, from) <= STREET_NEAR).map((s) => s.index) : []);
  const villageFirst = new Set<number>();
  areas.forEach((a, k) => {
    if (k === 0 || a.kind !== 'hamlet') return;
    const at = middle(k);
    const own = homes.filter((s) => areaOf(s) === k && !street.has(s.index)).sort((p, q) => far(p, at) - far(q, at) || byPlace(p, q));
    for (const s of own.slice(0, VILLAGE_FIRST)) villageFirst.add(s.index);
  });
  const band = (s: PlotSlot) => (street.has(s.index) ? 0 : villageFirst.has(s.index) ? 1 : areaOf(s) === 0 ? 2 : 3);
  const order = [...homes].sort((a, b) => band(a) - band(b) || far(a, from) - far(b, from) || byPlace(a, b));

  // Each home's household as it would naturally be, in that order (the family names taken in it).
  const taken = new Set<string>([...avoid, ...town.households.map((h) => h.household.name)]);
  const roll = (s: PlotSlot, room: number) => householdFor(content, assets, bedroomsOf(t, s), price.get(s.index)!, room, [...taken], streams.lot(s.x, s.z));
  const drawn = new Map<number, HouseholdDraft>();
  for (const s of order) {
    const household = roll(s, 4);
    if (!household) continue;
    drawn.set(s.index, household);
    taken.add(household.name);
  }
  // Smaller households in the smallest homes while the total would pass the cap: one or two
  // people first, then one alone.
  const already = town.households.filter((h) => h.slot !== playerSlot).reduce((n, h) => n + h.household.members.length, 0);
  let total = already + [...drawn.values()].reduce((n, h) => n + h.members.length, 0);
  const smallest = homes.filter((s) => drawn.has(s.index)).sort((a, b) => bedroomsOf(t, a) - bedroomsOf(t, b) || price.get(a.index)! - price.get(b.index)! || byPlace(a, b));
  for (const most of [2, 1]) {
    for (const s of smallest) {
      if (total <= cap) break;
      const before = drawn.get(s.index)!;
      if (before.members.length <= most) continue;
      taken.delete(before.name);
      const household = roll(s, most) ?? before;
      taken.add(household.name);
      drawn.set(s.index, household);
      total -= before.members.length - household.members.length;
    }
  }
  // The homes the tier fills: all of them in full; else those it fills in full, and of the others in
  // each area the first half (or third) in each lot's own order.
  const fills = (() => {
    if (tier === 'full') return () => true;
    const whole = (s: PlotSlot) => (tier === 'half' ? areaOf(s) === 0 || (player !== undefined && areaOf(s) === areaOf(player)) : far(s, from) <= TIER_NEAR);
    const share = tier === 'half' ? 1 / 2 : 1 / 3;
    const rest = new Map<number, PlotSlot[]>();
    for (const s of order) if (drawn.has(s.index) && !whole(s)) rest.set(areaOf(s), [...(rest.get(areaOf(s)) ?? []), s]);
    const some = new Set<number>();
    for (const list of rest.values()) {
      const ranked = [...list].sort((a, b) => streams.order(a.x, a.z) - streams.order(b.x, b.z) || byPlace(a, b));
      for (const s of ranked.slice(0, Math.ceil(list.length * share))) some.add(s.index);
    }
    return (s: PlotSlot) => whole(s) || some.has(s.index);
  })();
  const limit = tier === 'full' ? cap : Math.min(cap, TIERS[tier].fill);
  // Moved in in order, while there is room.
  const plan: { slot: number; household: HouseholdDraft }[] = [];
  let residents = already;
  for (const s of order) {
    const household = drawn.get(s.index);
    if (!household || !fills(s) || residents + household.members.length > limit) continue;
    residents += household.members.length;
    plan.push({ slot: s.index, household });
  }
  return plan;
}

/**
 * `town` once the player has moved into `playerSlot`: the second wave (`fillPlan`) moved into the
 * homes still for sale, so the town is lived in but for a few of its cheapest homes (fewer at a
 * tier below full), and the most residents it grows to set for its town file (`residentsMost`).
 */
export function fillTown(
  content: Content,
  assets: AssetRegistry,
  t: Templates,
  town: NeighbourhoodDraft,
  playerSlot: number,
  streams: PeopleStreams,
  options?: FillOptions,
): NeighbourhoodDraft {
  return { ...town, households: [...town.households, ...fillPlan(content, assets, t, town, playerSlot, streams, options)], residentsMost: residentsMost(options?.tier) };
}

/** Ties a neighbour has with the others, on average (about what three in ten of a town of ~120 gave each). */
const TIES = 35;
/** In a smaller town, the share of the others a neighbour knows (as every town had before `TIES`). */
const TIES_SHARE = 0.3;
/** How quickly knowing someone falls off with how far apart they live (m): half as likely this far away as next door. */
const TIES_NEAR = 60;

/**
 * Who among the neighbours already know each other: friends, rivals and acquaintances, as pairs of
 * residents (indexes into `sims`) with the bond's preset. Each resident from `from` on (those before
 * are a newly arrived player household) has about `TIES` ties, or three in ten of the others in a
 * smaller town, likelier with people living near (`homes`: each household's home, by the household
 * numbers in `sims`). Never two of one household. One roll of `rand` a pair. Of the ties, one in
 * ten is a rivalry and two in ten a friendship; a preset not among `presets` is left out.
 */
export function neighbourBonds(
  sims: readonly { household: number }[],
  homes: readonly (readonly [number, number])[],
  rand: Rand,
  { from = 0, presets }: { from?: number; presets?: readonly string[] } = {},
): { a: number; b: number; preset: string }[] {
  const out: { a: number; b: number; preset: string }[] = [];
  const n = sims.length;
  const ties = Math.min(TIES, TIES_SHARE * (n - from - 1));
  if (ties <= 0) return out;
  // How likely two households' people are to know each other, by how far apart they live.
  const H = homes.length;
  const near = new Float64Array(H * H);
  for (let i = 0; i < H; i++) {
    for (let j = i + 1; j < H; j++) {
      const w = TIES_NEAR / (TIES_NEAR + Math.hypot(homes[i][0] - homes[j][0], homes[i][1] - homes[j][1]));
      near[i * H + j] = near[j * H + i] = w;
    }
  }
  // (Each resident's sum of them over everyone else: a pair's chance is its share of both sums.)
  const weight = new Float64Array(n);
  for (let a = from; a < n; a++) {
    for (let b = from; b < n; b++) if (sims[a].household !== sims[b].household) weight[a] += near[sims[a].household * H + sims[b].household];
  }
  for (let a = from; a < n; a++) {
    for (let b = a + 1; b < n; b++) {
      if (sims[a].household === sims[b].household) continue;
      const roll = rand();
      const p = Math.min(1, ((ties * near[sims[a].household * H + sims[b].household]) / 2) * (1 / weight[a] + 1 / weight[b]));
      if (!(roll < p)) continue;
      const kind = roll / p;
      const preset = kind < 0.1 ? 'rivals' : kind < 0.3 ? 'friends' : 'acquaintances';
      if (!presets || presets.includes(preset)) out.push({ a, b, preset });
    }
  }
  return out;
}

/**
 * A household for a home with `bedrooms` that costs `price`, of `room` residents at most: it fits
 * (a couple shares a bedroom, two children may, anyone else needs one), has saved enough to buy it,
 * and preferably isn't many times richer than it needs to be. A dear home looks mostly among older
 * households, who have climbed higher and put more by. Every roll comes from `rand`. Null if none
 * turns up.
 */
export function householdFor(
  content: Content,
  assets: AssetRegistry,
  bedrooms: number,
  price: number,
  room: number,
  avoidNames: readonly string[],
  rand: Rand = Math.random,
): HouseholdDraft | null {
  let size = Math.max(1, Math.min(4, bedrooms + Math.floor(rand() * 2), room));
  let best: { household: HouseholdDraft; savings: number } | null = null;
  // A home dearer than two young grown-ups between jobs could buy goes mostly to an older household.
  const young = (content.adultStages[0]?.from ?? 18) + 7;
  const dear = price > savingsOf(content, { members: [{ age: young }, { age: young }] });
  for (let tries = 0, misfits = 0; tries < HOUSEHOLD_TRIES; tries++) {
    const household = randomHousehold(content, assets, size, avoidNames, { older: dear ? rand() < 0.8 : rand() < 0.2 }, rand);
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
 * them), with a name none of `avoidNames` has; any household of `size` if none turns up. Its rolls
 * come from `rand`.
 */
export function rehouse(content: Content, assets: AssetRegistry, t: Templates, town: NeighbourhoodDraft, slot: number, size: number, avoidNames: readonly string[], rand: Rand = Math.random): HouseholdDraft {
  const s = town.slots[slot];
  const bedrooms = (s && houseTemplate(t, s.template)?.bedrooms) ?? 1;
  return (s && householdFor(content, assets, bedrooms, homePrice(content, t, s), NEIGHBOUR_CAP, avoidNames, rand)) ?? randomHousehold(content, assets, size, avoidNames, {}, rand);
}
