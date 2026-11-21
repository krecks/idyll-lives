/**
 * What a home costs: paid once, from the household's savings, when it moves in (`wealth.ts`
 * `savingsOf` says what it has). The land (by the lot's size), the house (by its floor, every
 * storey and the basement) and what's in it (the things' prices), as `economy.homePrice` in
 * base.json sets them. An empty lot is the land alone; public places aren't for sale. A flat
 * (`flats.ts`) is its share of its building's land, its own floor at a flat's rate a tile
 * (`FLAT_FLOOR`) and what's in it: flats are the cheapest homes.
 */

import type { Content, Economy } from '../content/content';
import { flatsTemplate, type FlatsTemplate } from './flats';
import { houseTemplate, slotDims, type HouseTemplate, type PlotSlot, type Templates, type Wall } from './town';
import * as pmath from './pmath';

/** A home's price and what it's made of ($). */
export interface PriceParts {
  land: number;
  house: number;
  furniture: number;
  total: number;
}

type Rates = NonNullable<Economy['homePrice']>;
/** What `landAndBuilt` and `roundPrice` read of the rates (the views' homes for sale have no furniture rate). */
type LandRates = Pick<Rates, 'perTile' | 'perFloorTile' | 'round' | 'sizeRef' | 'sizePremium'>;
type Floor = { walls: Wall[]; objects: HouseTemplate['objects']; rooms?: HouseTemplate['rooms'] };

/**
 * Tiles of floor on one storey: its rooms as planned (generated houses), else the box its walls
 * enclose (the fixed layouts have no rooms listed; their walls stand on the tiles' edges).
 */
function floorTiles(f: Floor): number {
  if (f.rooms?.length) return f.rooms.reduce((n, r) => n + r.w * r.d, 0);
  if (!f.walls.length) return 0;
  const xs = f.walls.flatMap(([x0, , x1]) => [x0, x1]);
  const zs = f.walls.flatMap(([, z0, , z1]) => [z0, z1]);
  return (Math.max(...xs) - Math.min(...xs)) * (Math.max(...zs) - Math.min(...zs));
}

/**
 * The price of `house` (none: an empty lot) on a `width` × `depth` lot, from `rates`
 * (`economy.homePrice`; missing: homes are free). Takes the things' prices as a function, so the
 * house generator's sampler (which has object defs, not a `Content`) prices homes the same way.
 */
export function priceOf(objectPrice: (def: string) => number, rates: Rates | undefined, house: HouseTemplate | undefined, width: number, depth: number): PriceParts {
  if (!rates) return { land: 0, house: 0, furniture: 0, total: 0 };
  const floors: Floor[] = house ? [house, house.upper, house.basement].flatMap((f) => (f ? [f] : [])) : [];
  return partsOf(objectPrice, rates, width * depth, floors, 1);
}

/**
 * A flat's floor costs this share of a house's a tile: no roof, garden or outside walls of its own
 * but the ones it shares, and its stairs and hall the building's.
 */
export const FLAT_FLOOR = 0.25;

/**
 * The price of flat `unit` of building `b` (as `flatsTemplate` gives it: for sale, furnished for
 * nobody in particular) from `rates`: its share of the building's land (the lot shared evenly by
 * its flats, and the place under them over a place), its floor (`FLAT_FLOOR`) and what's in it.
 */
export function flatPriceOf(objectPrice: (def: string) => number, rates: Rates | undefined, b: FlatsTemplate, unit: number): PriceParts {
  const flat = b.units[unit];
  if (!rates || !flat) return { land: 0, house: 0, furniture: 0, total: 0 };
  return partsOf(objectPrice, rates, flatLandShare(b.lot.width * b.lot.depth, b.units.length, !!b.place), [flat.layout], FLAT_FLOOR);
}

/** A flat's share of its building's lot (`lotTiles`): the lot shared evenly by its `flats`, and the place under them if they are over one. */
export function flatLandShare(lotTiles: number, flats: number, overPlace: boolean): number {
  return lotTiles / (flats + (overPlace ? 1 : 0));
}

/**
 * The land (`tiles` of it) and the house (`floor` tiles of floor, `share` of a house's rate a tile) at `rates`, not
 * rounded. A bigger house costs more a tile of floor too (a villa isn't a cottage three times over): `sizePremium`
 * against `sizeRef` tiles.
 */
export function landAndBuilt(rates: LandRates, tiles: number, floor: number, share: number): { land: number; built: number } {
  return { land: rates.perTile * tiles, built: share * rates.perFloorTile * floor * pmath.pow(floor / (rates.sizeRef ?? 100), rates.sizePremium ?? 0) };
}

/** A price rounded to the rates' step (`round`). */
export function roundPrice(rates: Pick<Rates, 'round'>, price: number): number {
  const step = Math.max(1, rates.round);
  return Math.round(price / step) * step;
}

/** The land (`tiles` of it), the floor of `floors` (`share` of a house's a tile) and what's in them, priced at `rates`. */
function partsOf(objectPrice: (def: string) => number, rates: Rates, tiles: number, floors: Floor[], share: number): PriceParts {
  const parts = landAndBuilt(rates, tiles, floors.reduce((n, f) => n + floorTiles(f), 0), share);
  const land = Math.round(parts.land);
  const built = Math.round(parts.built);
  const furniture = Math.round(rates.furniture * floors.reduce((n, f) => n + f.objects.reduce((m, o) => m + objectPrice(o.def), 0), 0));
  return { land, house: built, furniture, total: roundPrice(rates, land + built + furniture) };
}

/**
 * The price of plot `slot`'s home (`empty`: its lot alone, the house cleared away); null for a
 * public place. A farm's fields and farmyard are part of its lot, so they're priced as land.
 */
export function priceParts(content: Content, t: Templates, slot: PlotSlot, empty = false): PriceParts | null {
  if (slot.kind !== 'house') return null;
  const { width, depth } = slotDims(t, slot);
  // The house as it stands for sale: furnished for nobody in particular (the layout is the same for anyone).
  // (An empty lot has no template: `houseTemplate` finds nothing for it.)
  const house = empty ? undefined : houseTemplate(t, slot.template);
  return priceOf((def) => content.object(def)?.price ?? 0, content.economy.homePrice, house, width, depth);
}

/**
 * The price of flat `unit` (its index in its building's `units`) of plot `slot`, a building of flats
 * or a place with flats over it (`slot.template` a `flats:` or `over:` id); null if it has no such flat.
 */
export function flatPriceParts(content: Content, t: Templates, slot: Pick<PlotSlot, 'template'>, unit: number): PriceParts | null {
  const b = flatsTemplate(t, slot.template);
  if (!b?.units[unit]) return null;
  return flatPriceOf((def) => content.object(def)?.price ?? 0, content.economy.homePrice, b, unit);
}

/** The price of flat `unit` of plot `slot` (0 if it has none). */
export function flatPrice(content: Content, t: Templates, slot: Pick<PlotSlot, 'template'>, unit: number): number {
  return flatPriceParts(content, t, slot, unit)?.total ?? 0;
}

/** The price of plot `slot`'s home (0 for a public place). */
export function homePrice(content: Content, t: Templates, slot: PlotSlot, empty = false): number {
  return priceParts(content, t, slot, empty)?.total ?? 0;
}
