/**
 * Neighbourhood generation and assembly. A town is one or more streets, one behind the other,
 * each with a row of plots on both sides; each plot has a small, medium or large house (or a farm)
 * and gets a house layout (or a public place: the park, a playground, a square, a shop) from
 * `content/houses.json`. A generated house comes with its lot, cut to fit it with a snug, roomy
 * or big garden (`housegen.ts` `fitLot`); the fixed layouts are drawn for a medium lot and sit
 * centred on lots of the other sizes, with the front door towards the street, a small lot only
 * taking a house that fits. A farm is a large house with fields beside it and a farmyard behind.
 * Plots on the south side of a street are rotated 180° so every front door faces the street.
 * What kind of world the town is in (`world.ts`) and its seed decide the layout (`generateWorld`);
 * the households come after (`populate`). The result is one town file the simulation loads like any lot.
 */

import type { AssetRegistry } from '../assets/registry';
import type { Content } from '../content/content';
import type { GameKind, LandscapeParams, Lifespan, WorldLook } from '../core/protocol';
import { fetchText } from '../content/content';
import { bedroomsNeeded, householdBonds, householdSpawns, randomHousehold, residentsOf, type HouseholdDraft, type SimSpawn } from './household';
import { generateHouse, objectInfos, random, residentsKey, type GardenSize, type HouseGenConfig, type ObjectInfos, type Residents } from './housegen';
import { streetNames, WORLDS, worldName, worldTagline, type WorldKind } from './world';

export type Wall = [number, number, number, number];
export interface Door {
  x: number;
  z: number;
  axis: 'x' | 'z';
}
export interface Placement {
  def: string;
  x: number;
  z: number;
  rot: number;
  /** Degrees past `rot` (0..90), for things that turn freely (sim-core `check_turn`). */
  turn?: number;
  /** Off the middle of its footprint in hundredths of a tile ([x, z], up to ±50), for things that turn freely. */
  offset?: [number, number];
}

export interface HouseTemplate {
  id: string;
  name: string;
  description: string;
  bedrooms: number;
  walls: Wall[];
  doors: Door[];
  /** Windows in walls, placed like doors. */
  windows?: Door[];
  objects: Placement[];
  spawns: [number, number][];
  /** The storey above (generated two-storey houses), laid out like the ground. */
  upper?: { walls: Wall[]; doors: Door[]; windows?: Door[]; objects: Placement[]; rooms?: PlannedRoom[] };
  /** The basement (some generated houses), laid out like the ground under it: no windows, stairs standing in it come up inside the house. */
  basement?: { walls: Wall[]; doors: Door[]; windows?: Door[]; objects: Placement[]; rooms?: PlannedRoom[] };
  /** Generated houses: the rooms as planned (for checking them; the game finds rooms from the walls). */
  rooms?: PlannedRoom[];
  /** Generated houses with a garden size: the lot cut to fit the house, the layout in its frame (see `housegen.ts` `fitLot`). */
  lot?: { width: number; depth: number };
}

/** A room of a generated house: what it's for and the rectangle it takes. */
export interface PlannedRoom {
  kind: string;
  x: number;
  z: number;
  w: number;
  d: number;
}

/**
 * A public place (`parks` in `content/houses.json`): a park or square laid out in the open, or a
 * shop, a small shell with walls, doors and windows like a house's. Everyone in town may use what
 * is there. The first one is the town park every town has.
 */
export interface PublicTemplate {
  id: string;
  name: string;
  description: string;
  /** `park` (default) or `shop` (labels, and which ones a town gets). */
  kind?: 'park' | 'shop';
  walls?: Wall[];
  doors?: Door[];
  windows?: Door[];
  objects: Placement[];
  /** Paved ground (plot coordinates, any size): walks, squares, terraces. Only drawn. */
  paths?: Rect[];
  /** Where visitors arrive (plot coordinates); default: the middle of the street front. */
  entry?: [number, number];
}

/** `farm`: a large lot's house with fields and a farmyard (farm worlds only; not offered to resize to). */
export type LotSize = 'small' | 'medium' | 'large' | 'farm';
export const LOT_SIZES: readonly LotSize[] = ['small', 'medium', 'large'];

/** The lot size a lot's house is made for (a farmhouse is a large lot's house). */
const houseLot = (size: LotSize | undefined): LotSize => (size === 'farm' ? 'large' : (size ?? 'medium'));

export interface Templates {
  /** The lot every layout is drawn for (a medium lot). */
  plot: { width: number; depth: number };
  /** Lot sizes (missing in older files: every lot is `plot`). */
  lotSizes?: Partial<Record<LotSize, { width: number; depth: number }>>;
  houses: HouseTemplate[];
  parks: PublicTemplate[];
  /** How houses are generated (`content/housegen.json`; absent: only the fixed layouts above). */
  gen?: HouseGenConfig;
  /** Object types from the game's content (for furnishing generated houses). */
  info?: ObjectInfos;
  /** Tiles one lamp lights at night (content `roomRules.lampTiles`): generated rooms get lights enough. */
  lampTiles?: number;
  /** Tiles one window lights by day (content `roomRules.windowTiles`): generated rooms get daylight enough. */
  windowTiles?: number;
}

/** Town sizes of drafts from older code (worlds have a `WorldKind` now). */
export type TownSize = 'small' | 'medium' | 'large';

export interface PlotSlot {
  index: number;
  /** `park` and `shop` are public places (see `PublicTemplate`). */
  kind: 'house' | 'park' | 'shop';
  template: string;
  /** Address or place name. */
  name: string;
  x: number;
  z: number;
  /** Rotated 180° (south side of the street). */
  rotated: boolean;
  /** The size of its house (and of its lot, unless `w` and `d` say otherwise). Missing in drafts from older code: medium. */
  lot?: LotSize;
  /** Its lot's own width and depth: a generated house's lot cut to fit it (and a farm's round that). Missing: `lot`'s. */
  w?: number;
  d?: number;
  /** How much garden a generated house's lot has round it (see `housegen.ts` `fitLot`). */
  garden?: GardenSize;
  /** The street it is on (index into the draft's `streets`; missing: the first). */
  street?: number;
  /** A farm lot's fields and farmyard, in plot coordinates like the house layout. */
  farm?: { objects: Placement[]; fields: Rect[] };
}

export interface Rect {
  x: number;
  z: number;
  w: number;
  d: number;
}

export interface NeighbourhoodDraft {
  name: string;
  /** Drafts from older code; worlds say what kind they are in `world`. */
  size?: TownSize;
  width: number;
  depth: number;
  slots: PlotSlot[];
  streets: Rect[];
  /** Neighbour households and the slot each lives in. */
  households: { household: HouseholdDraft; slot: number }[];
  /**
   * Landscape seed (hills, woods, lawn detail). Kept while the draft is edited, so the 3D preview
   * in the menus and the game itself show the same valley. Missing in drafts from older code.
   * In a world it is also the seed its layout was made from.
   */
  seed?: number;
  /** The world the town is in (missing in drafts from older code: the classic valley). */
  world?: WorldInfo;
}

/** What kind of world a town is in, one line about it, and how its land looks. */
export interface WorldInfo {
  kind: WorldKind;
  tagline: string;
  landscape: LandscapeParams;
}

/** Presentation data the renderer reads back from the world (it survives save/load). */
export interface TownMeta {
  kind: 'town';
  name: string;
  streets: Rect[];
  paths: Rect[];
  seed: number;
  world?: WorldLook;
  /** The menus' preview only (`previewWorld`): the plot drawn as the player's house. */
  home?: number | null;
}

const MARGIN = 8;
/** Tiles a town may measure each way (sim-core `Lot::from_file`). */
const MAX_TOWN = 256;
const STREET = 6;

let templates: Promise<Templates> | null = null;
let giveContent: (content: Content) => void = () => {};
const townContent = new Promise<Content>((resolve) => (giveContent = resolve));

/** The game's content, once loaded (generated houses are furnished from it). */
export function useTownContent(content: Content): void {
  giveContent(content);
}

export function loadTemplates(): Promise<Templates> {
  const base = import.meta.env.BASE_URL;
  templates ??= Promise.all([
    fetchText(`${base}content/houses.json`).then((t) => JSON.parse(t) as Templates),
    fetchText(`${base}content/housegen.json`)
      .then((t) => JSON.parse(t) as HouseGenConfig)
      .catch(() => undefined),
    townContent,
  ]).then(([t, gen, content]) => ({ ...t, gen, info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles }));
  return templates;
}

const generated = new Map<string, HouseTemplate | undefined>();

/** A generated house's id: its size, seed and garden (the same id always gives the same house and lot). */
export function generatedId(size: LotSize, seed: number, garden?: GardenSize): string {
  return `gen:${size}:${seed >>> 0}${garden ? `:${garden}` : ''}`;
}

type Rand = () => number;
const pick = <T>(list: readonly T[], rand: Rand = Math.random): T => list[Math.floor(rand() * list.length)];
/** One of the keys of `weights`, as likely as its weight. */
function pickWeighted<K extends string>(weights: Partial<Record<K, number>>, rand: Rand): K {
  const entries = Object.entries(weights) as [K, number][];
  let r = rand() * entries.reduce((n, [, w]) => n + w, 0);
  for (const [k, w] of entries) if ((r -= w) < 0) return k;
  return entries[entries.length - 1][0];
}
/** A shuffled copy (Fisher-Yates). */
function shuffled<T>(list: readonly T[], rand: Rand = Math.random): T[] {
  const out = [...list];
  for (let i = out.length - 1; i > 0; i--) {
    const j = Math.floor(rand() * (i + 1));
    [out[i], out[j]] = [out[j], out[i]];
  }
  return out;
}

/** A house layout by id; a generated one furnished for `residents` (the layout is the same for anyone; undefined: nobody lives there). */
export function houseTemplate(t: Templates, id: string, residents?: Residents): HouseTemplate | undefined {
  if (id.startsWith('gen:')) {
    const at = `${id}#${residentsKey(residents)}`;
    if (!generated.has(at)) {
      const [, size, seed, garden] = id.split(':');
      const house = t.gen && t.info ? generateHouse(t.gen, t.info, lotDims(t, size as LotSize), t.plot, size, Number(seed), t.lampTiles, t.windowTiles, residents, garden as GardenSize | undefined) : null;
      generated.set(at, house ?? undefined);
    }
    return generated.get(at);
  }
  return t.houses.find((h) => h.id === id);
}

/** Width and depth of a lot of this size. */
export function lotDims(t: Templates, size: LotSize | undefined): { width: number; depth: number } {
  return t.lotSizes?.[size ?? 'medium'] ?? t.plot;
}

/** Width and depth of a plot's lot: its own (cut to fit its house), else its size's. */
export function slotDims(t: Templates, s: Pick<PlotSlot, 'lot' | 'w' | 'd'>): { width: number; depth: number } {
  return s.w && s.d ? { width: s.w, depth: s.d } : lotDims(t, s.lot);
}

/** Farm lots: fields this wide beside the farmhouse's own lot, the farmyard this deep behind it. */
const FARM_FIELDS = 18;
const FARM_YARD = 16;

/**
 * A layout moved onto plot `s`'s lot: centred across it, front kept at the street (a farmhouse
 * stands at its west end, its fields beside it). Objects that would end up off the lot (garden
 * things on a small lot) are left out.
 */
export function onLot(t: Templates, h: HouseTemplate, s: Pick<PlotSlot, 'lot' | 'w' | 'd'>, footprint: (def: string) => [number, number]): HouseTemplate {
  const { width, depth } = slotDims(t, s);
  const frame = h.lot ?? t.plot;
  // A fixed layout's farmhouse stands where it would on a large lot.
  const dx = s.lot !== 'farm' ? Math.floor((width - frame.width) / 2) : h.lot ? 0 : Math.floor((lotDims(t, 'large').width - t.plot.width) / 2);
  if (dx === 0 && depth >= frame.depth) return h;
  const inside = (o: Placement) => {
    const [fw, fd] = footprint(o.def);
    const [w, d] = o.rot % 2 === 0 ? [fw, fd] : [fd, fw];
    return o.x >= 0 && o.z >= 0 && o.x + w <= width && o.z + d <= depth;
  };
  const move = <T extends { walls: Wall[]; doors: Door[]; windows?: Door[]; objects: Placement[] }>(f: T): T => ({
    ...f,
    walls: f.walls.map(([x0, z0, x1, z1]) => [x0 + dx, z0, x1 + dx, z1]),
    doors: f.doors.map((d) => ({ ...d, x: d.x + dx })),
    windows: f.windows?.map((d) => ({ ...d, x: d.x + dx })),
    objects: f.objects.map((o) => ({ ...o, x: o.x + dx })).filter(inside),
  });
  return { ...move(h), spawns: h.spawns.map(([x, z]) => [x + dx, z]), upper: h.upper && move(h.upper), basement: h.basement && move(h.basement) };
}

/** A public place moved onto plot `s`'s lot like a house layout (`onLot`): centred across it, front kept at the street. */
export function placeOnLot(t: Templates, p: PublicTemplate, s: Pick<PlotSlot, 'lot' | 'w' | 'd'>): PublicTemplate {
  const dx = Math.floor((slotDims(t, s).width - t.plot.width) / 2);
  if (dx === 0) return p;
  const door = (d: Door): Door => ({ ...d, x: d.x + dx });
  return {
    ...p,
    walls: p.walls?.map(([x0, z0, x1, z1]) => [x0 + dx, z0, x1 + dx, z1]),
    doors: p.doors?.map(door),
    windows: p.windows?.map(door),
    objects: p.objects.map((o) => ({ ...o, x: o.x + dx })),
    paths: p.paths?.map((r) => ({ ...r, x: r.x + dx })),
    entry: p.entry && [p.entry[0] + dx, p.entry[1]],
  };
}

/** Whether a layout's house fits on a lot of `size` (with a metre to spare around its walls). */
export function fitsLot(t: Templates, h: HouseTemplate, size: LotSize): boolean {
  const { width, depth } = lotDims(t, houseLot(size));
  const dx = Math.floor((width - t.plot.width) / 2);
  const xs = h.walls.flatMap(([x0, , x1]) => [x0 + dx, x1 + dx]);
  const zs = h.walls.flatMap(([, z0, , z1]) => [z0, z1]);
  return Math.min(...xs) >= 1 && Math.max(...xs) <= width - 1 && Math.max(...zs) <= depth - 1;
}

/**
 * Places every lot along its street by its size, and sizes the town around them. Streets lie one
 * behind the other, the back gardens of one street's far row touching those of the next one's near row.
 */
function placeLots(t: Templates, slots: PlotSlot[]): { slots: PlotSlot[]; width: number; depth: number; streets: Rect[] } {
  const count = Math.max(1, ...slots.map((s) => (s.street ?? 0) + 1));
  // Each street's rotated (south) row lies before it, the other after it; each row's lots touch the street.
  const rowDepth = (street: number, rotated: boolean) =>
    Math.max(0, ...slots.filter((s) => (s.street ?? 0) === street && s.rotated === rotated).map((s) => slotDims(t, s).depth));
  const streetZ: number[] = [];
  let z = MARGIN;
  for (let i = 0; i < count; i++) {
    streetZ.push(z + rowDepth(i, true));
    z = streetZ[i] + STREET + rowDepth(i, false);
  }
  const x = new Map<string, number>();
  const placed = slots.map((s) => {
    const { width, depth } = slotDims(t, s);
    const street = s.street ?? 0;
    const row = `${street}:${s.rotated}`;
    const at = { ...s, x: x.get(row) ?? MARGIN, z: s.rotated ? streetZ[street] - depth : streetZ[street] + STREET };
    x.set(row, at.x + width);
    return at;
  });
  const width = Math.max(MARGIN, ...x.values()) + MARGIN;
  return { slots: placed, width, depth: z + MARGIN, streets: streetZ.map((sz) => ({ x: 0, z: sz, w: width, d: STREET })) };
}

/**
 * A house for a new lot of `size` with a `garden` round it, and the lot's own width and depth: a
 * generated house (see `housegen.ts`) on a lot cut to fit it, else a fixed layout that fits.
 */
export function houseFor(t: Templates, size: LotSize, garden: GardenSize, rand: Rand = Math.random): Pick<PlotSlot, 'template' | 'w' | 'd' | 'garden'> {
  if (t.gen) {
    const id = generatedId(houseLot(size), (rand() * 2 ** 31) >>> 0, garden);
    const h = houseTemplate(t, id);
    if (h) return { template: id, garden, ...lotOf(h, size) };
  }
  const fitting = t.houses.filter((h) => fitsLot(t, h, size));
  return { template: pick(fitting.length ? fitting : t.houses, rand).id };
}

/** A generated house's lot (a farm's: its fields beside it, its farmyard behind). */
function lotOf(h: HouseTemplate, size: LotSize): Pick<PlotSlot, 'w' | 'd'> {
  if (!h.lot) return {};
  return size === 'farm' ? { w: h.lot.width + FARM_FIELDS, d: h.lot.depth + FARM_YARD } : { w: h.lot.width, d: h.lot.depth };
}

/** The draft with the lot of `slot` cut for a `garden` of another size round its house (the same house); the town is laid out again round it. */
export function withGarden(t: Templates, town: NeighbourhoodDraft, slot: number, garden: GardenSize): NeighbourhoodDraft {
  const old = town.slots[slot];
  if (!old || old.garden === garden || !old.template.startsWith('gen:')) return town;
  const [, size, seed] = old.template.split(':');
  const template = generatedId(size as LotSize, Number(seed), garden);
  const h = houseTemplate(t, template);
  if (!h) return town;
  const placed = placeLots(t, town.slots.map((s) => (s.index === slot ? { ...s, template, garden, ...lotOf(h, s.lot ?? 'medium') } : s)));
  return { ...town, ...placed };
}

/** Template id of a lot with nothing on it yet (no house template has it): build from the ground up. */
export const EMPTY_LOT = 'empty';

/** The draft with the house on `slot` cleared away (an empty lot), or put back (`template`). */
export function withLot(town: NeighbourhoodDraft, slot: number, template: string): NeighbourhoodDraft {
  if (town.slots[slot]?.template === template) return town;
  return { ...town, slots: town.slots.map((s) => (s.index === slot ? { ...s, template } : s)) };
}

/** Residents the neighbours number at most (sim-core holds 64 Sims: room for the player's household and babies). */
const MAX_NEIGHBOURS = 48;

/**
 * Lays out a world of `kind` from `seed` (the same kind and seed always give the same town): its
 * streets, lots, houses, public places and farms, its name, tagline and land. Nobody lives there
 * yet (see `populate`).
 */
export function generateWorld(content: Content, t: Templates, kind: WorldKind, seed = (Math.random() * 2 ** 31) >>> 0, name?: string): NeighbourhoodDraft {
  const recipe = WORLDS[kind];
  const rand = random(seed);
  // Rolled whether used or not, so a name typed in leaves the rest of the world as it was.
  const rolledName = worldName(content, kind, rand);
  const tagline = worldTagline(content, kind, rand);
  const names = streetNames(content, kind, rand, recipe.streets);
  const cols = recipe.columns;
  const half = Math.floor(cols / 2);
  /** Lot index of `col` in `row` (0: the rotated south row) of street `street`. */
  const at = (street: number, row: number, col: number) => (street * 2 + row) * cols + col;

  // The park in the middle of the first street's north row; the other public places gather round
  // it (across the street first, then beside it) and round the middle of the other streets, so
  // every street has a little centre.
  const parkIndex = at(0, 1, half);
  const near = (street: number) =>
    [[1, half], [0, half], [1, half + 1], [0, half + 1], [1, half - 1], [0, half - 1]]
      .filter(([, col]) => col >= 0 && col < cols)
      .map(([row, col]) => at(street, row, col))
      .filter((i) => i !== parkIndex);
  const ranked = Array.from({ length: recipe.streets }, (_, i) => near(i));
  const spots = [0, 1, 2, 3, 4, 5].flatMap((k) => ranked.map((list) => list[k]).filter((i) => i !== undefined));
  const [fewest, most] = recipe.places;
  const count = Math.min(spots.length, fewest + Math.floor(rand() * (most - fewest + 1)));
  // The places this kind of world would rather have first, in no particular order, then the rest.
  const prefer = recipe.prefer ?? [];
  const others = shuffled(t.parks.slice(1), rand)
    .sort((a, b) => Number(!prefer.includes(a.id)) - Number(!prefer.includes(b.id)))
    .slice(0, count);
  const places = new Map<number, PublicTemplate>([[parkIndex, t.parks[0]], ...others.map((p, i) => [spots[i], p] as [number, PublicTemplate])]);

  // Farms on the ends of the rows, out where the town gives way to the fields.
  const [fewFarms, mostFarms] = recipe.farms ?? [0, 0];
  const ends = Array.from({ length: recipe.streets * 2 }, (_, r) => [r * cols, r * cols + cols - 1]).flat();
  const farmCount = t.lotSizes?.farm ? fewFarms + Math.floor(rand() * (mostFarms - fewFarms + 1)) : 0;
  const farms = new Set(shuffled(ends.filter((i) => !places.has(i)), rand).slice(0, farmCount));
  const farmWords = shuffled(content.names.worlds?.farm?.start ?? ['Meadow', 'Barley', 'Clover'], rand);

  const lotSize = (): LotSize => {
    if (!t.lotSizes) return 'medium';
    const r = rand();
    return r < recipe.small ? 'small' : r < 1 - recipe.large ? 'medium' : 'large';
  };
  const gardenSize = () => pickWeighted(recipe.gardens, rand);
  const unplaced: PlotSlot[] = [];
  let farmNo = 0;
  for (let street = 0; street < recipe.streets; street++) {
    for (let row = 0; row < 2; row++) {
      let number = 1;
      for (let col = 0; col < cols; col++) {
        const index = unplaced.length;
        const place = places.get(index);
        const farm = farms.has(index);
        const lot: LotSize = place ? 'medium' : farm ? 'farm' : lotSize();
        const slot: PlotSlot = {
          index,
          kind: place ? (place.kind ?? 'park') : 'house',
          ...(place ? { template: place.id } : houseFor(t, lot, gardenSize(), rand)),
          name: place ? place.name : farm ? `${farmWords[farmNo++ % farmWords.length]} Farm` : `${number++ * 2 - row} ${names[street]}`,
          x: 0,
          z: 0,
          rotated: row === 0,
          lot,
          street,
        };
        if (farm) slot.farm = farmstead(content, t, slot, rand);
        unplaced.push(slot);
      }
    }
  }
  // A town is at most 256 m across: a row too long for that gives up its farms first, then its biggest lots.
  const smaller: Partial<Record<LotSize, LotSize>> = { farm: 'large', large: 'medium', medium: 'small' };
  for (let row = 0; row < recipe.streets * 2; row++) {
    const lots = unplaced.slice(row * cols, row * cols + cols);
    const across = () => lots.reduce((sum, s) => sum + slotDims(t, s).width, 0);
    while (across() > MAX_TOWN - 2 * MARGIN) {
      const order: LotSize[] = ['farm', 'large', 'medium'];
      const slot = order.map((size) => lots.find((s) => s.kind === 'house' && s.lot === size)).find((s) => s);
      if (!slot?.lot) break;
      const lot = smaller[slot.lot]!;
      Object.assign(slot, { lot, w: undefined, d: undefined, ...houseFor(t, lot, slot.garden ?? gardenSize(), rand), farm: undefined });
    }
  }
  const { slots, width, depth, streets } = placeLots(t, unplaced);
  const landscape = recipe.landscape(rand, width, depth);
  return { name: name ?? rolledName, width, depth, slots, streets, households: [], seed, world: { kind, tagline, landscape } };
}

/**
 * Neighbours for a town: `count` households (default: all but two houses empty) in houses picked
 * at random, each sized to its home, no more than `MAX_NEIGHBOURS` residents in all.
 */
export function populate(content: Content, assets: AssetRegistry, t: Templates, town: NeighbourhoodDraft, count?: number): NeighbourhoodDraft {
  const houses = town.slots.filter((s) => s.kind === 'house');
  const wanted = Math.max(0, Math.min(count ?? houses.length - 2, houses.length - 1));
  const taken = new Set<string>();
  const households: NeighbourhoodDraft['households'] = [];
  let residents = 0;
  for (const slot of shuffled(houses).slice(0, wanted)) {
    if (residents >= MAX_NEIGHBOURS) break;
    const bedrooms = houseTemplate(t, slot.template)?.bedrooms ?? 1;
    let size = Math.max(1, Math.min(4, bedrooms + Math.floor(Math.random() * 2), MAX_NEIGHBOURS - residents));
    // A household that fits: a couple shares a bedroom, two children may, anyone else needs one.
    let household = randomHousehold(content, assets, size, [...taken]);
    for (let tries = 1; bedroomsNeeded(residentsOf(content, household)) > bedrooms && size > 1; tries++) {
      if (tries % 6 === 0) size--;
      household = randomHousehold(content, assets, size, [...taken]);
    }
    taken.add(household.name);
    residents += household.members.length;
    households.push({ household, slot: slot.index });
  }
  return { ...town, households };
}

/** A town of one of the older sizes with its neighbours (a large one is a dense town now). */
export function generateNeighbourhood(
  content: Content,
  assets: AssetRegistry,
  t: Templates,
  size: TownSize,
  name?: string,
  neighbours?: number,
  seed?: number,
): NeighbourhoodDraft {
  return populate(content, assets, t, generateWorld(content, t, size === 'large' ? 'dense' : size, seed, name), neighbours);
}

/** What a farm's fields grow: two or three of these, a pair of rows each in turn. */
const CROPS = ['garden.vegPatch', 'garden.pumpkins', 'garden.sunflowers', 'garden.strawberries', 'garden.tomatoes'];
/** The farmyard behind a farmhouse, biggest first: [object, fewest, most, turned (rot) or null: any way]. */
const FARMYARD: [string, number, number, number | null][] = [
  ['farm.barn', 1, 1, 2],
  ['farm.silo', 1, 1, 2],
  ['farm.tractor', 0, 1, null],
  ['farm.coop', 1, 1, 2],
  ['farm.trough', 1, 1, null],
  ['farm.hayStack', 1, 2, null],
  ['farm.hayBale', 2, 4, null],
];

/**
 * A farm lot's fields and farmyard (plot coordinates, street at z = 0), round a farmhouse that
 * stands as on a large lot: a field of crops in rows beside the house (a path between each pair
 * of rows), a fallow field with hay behind it, and the barn, silo, coop and the rest in a yard
 * behind the house. Clear of the house, its garden and its driveway.
 */
function farmstead(content: Content, t: Templates, slot: PlotSlot, rand: Rand): { objects: Placement[]; fields: Rect[] } {
  const footprint = (def: string): [number, number] => content.object(def)?.footprint ?? [1, 1];
  const has = (def: string) => !!content.object(def);
  const { width: pw, depth: pd } = slotDims(t, slot);
  const template = houseTemplate(t, slot.template);
  // The farmhouse's own lot (a fixed layout's: a large one).
  const large = template?.lot ?? lotDims(t, 'large');
  const drive = template ? driveway(onLot(t, template, slot, footprint), pw, pd, footprint) : null;
  const east = Math.ceil(Math.max(large.width, drive ? drive.x + drive.w : 0)) + 1;
  const objects: Placement[] = [];
  const taken: Rect[] = [];
  const free = (r: Rect) => !taken.some((o) => r.x < o.x + o.w + 1 && r.x + r.w + 1 > o.x && r.z < o.z + o.d + 1 && r.z + r.d + 1 > o.z);
  const put = (def: string, x: number, z: number, rot: number) => {
    const [fw, fd] = footprint(def);
    const [w, d] = rot % 2 === 0 ? [fw, fd] : [fd, fw];
    objects.push({ def, x, z, rot });
    taken.push({ x, z, w, d });
  };

  // Crops beside the house.
  const crops = shuffled(CROPS.filter(has), rand).slice(0, 2 + Math.floor(rand() * 2));
  // An odd number of tiles across, so its rows of crops line up with the furrows the renderer draws (street.ts `fieldLayers`).
  const across = pw - 1 - east;
  const cropField: Rect = { x: east + 1 - (across % 2), z: 2, w: across - 1 + (across % 2), d: Math.min(22, Math.floor(pd * 0.5)) };
  const fallow: Rect = { x: east, z: cropField.z + cropField.d + 2, w: cropField.w, d: pd - 2 - (cropField.z + cropField.d + 2) };
  if (crops.length && cropField.w >= 3) {
    // A scarecrow in one of the rows, halfway back (not on a path: those lead to the plants).
    const rows = Math.floor((cropField.w - 1) / 2);
    const scarecrow = has('farm.scarecrow') ? { x: cropField.x + 1 + 2 * Math.floor(rand() * rows), z: cropField.z + Math.floor(cropField.d / 2) } : null;
    for (let i = 0, x = cropField.x + 1; x < cropField.x + cropField.w - 1; i++, x += 2) {
      const crop = crops[Math.floor(i / 2) % crops.length];
      // Facing the paths either side (the next plant in the row stands in front otherwise).
      for (let z = cropField.z + 1; z < cropField.z + cropField.d - 1; z++) {
        if (scarecrow?.x === x && scarecrow.z === z) put('farm.scarecrow', x, z, 2);
        else put(crop, x, z, i % 2 === 0 ? 1 : 3);
      }
    }
  }
  // Hay on the fallow field.
  const [bw, bd] = footprint('farm.hayBale');
  if (has('farm.hayBale') && fallow.d >= bd + 2) {
    for (let k = 0, n = 3 + Math.floor(rand() * 4); k < n; k++) {
      const x = fallow.x + 1 + Math.floor(rand() * Math.max(1, fallow.w - 1 - bw));
      const z = fallow.z + 1 + Math.floor(rand() * Math.max(1, fallow.d - 1 - bd));
      if (free({ x, z, w: bw, d: bd })) put('farm.hayBale', x, z, Math.floor(rand() * 4));
    }
  }

  // The farmyard behind the house.
  const yard: Rect = { x: 1, z: large.depth + 1, w: east - 2, d: pd - large.depth - 2 };
  for (const [def, fewest, most, turned] of FARMYARD) {
    if (!has(def)) continue;
    for (let k = 0, n = fewest + Math.floor(rand() * (most - fewest + 1)); k < n; k++) {
      for (let tries = 0; tries < 40; tries++) {
        const rot = turned ?? Math.floor(rand() * 4);
        const [fw, fd] = footprint(def);
        const [w, d] = rot % 2 === 0 ? [fw, fd] : [fd, fw];
        if (w > yard.w || d > yard.d) break;
        const x = yard.x + Math.floor(rand() * (yard.w - w + 1));
        // Big buildings stand at the back of the yard, facing the house.
        const z = turned === 2 ? yard.z + yard.d - d : yard.z + Math.floor(rand() * (yard.d - d + 1));
        if (!free({ x, z, w, d })) continue;
        put(def, x, z, rot);
        break;
      }
    }
  }
  return { objects, fields: [cropField, ...(fallow.d >= 3 ? [fallow] : [])] };
}

/** Driveways are paths wider than this (garden paths are 1 m); see `driveway`. */
export const DRIVEWAY_MIN_WIDTH = 1.8;
/** Driveway length past the front wall of the house: room to park a car beside it. */
const DRIVEWAY_PAST_FRONT = 5.4;

/**
 * Driveway of a house layout in plot coordinates (front edge at z = 0): from the street to
 * beside the house, on the side with more room (away from the front door when even), clear
 * of garden objects. Null when neither side has room.
 */
export function driveway(h: HouseTemplate, pw: number, pd: number, footprint: (def: string) => [number, number]): Rect | null {
  const xs = h.walls.flatMap(([x0, , x1]) => [x0, x1]);
  const zs = h.walls.flatMap(([, z0, , z1]) => [z0, z1]);
  const [hx0, hx1, hz0] = [Math.min(...xs), Math.max(...xs), Math.min(...zs)];
  const door = h.doors[0]?.x ?? (hx0 + hx1) / 2;
  const d = Math.min(hz0 + DRIVEWAY_PAST_FRONT, pd - 1);
  const sides = [
    { gap: pw - hx1, far: door < (hx0 + hx1) / 2, rect: (w: number, gap: number) => ({ x: hx1 + Math.min(0.6, gap - w - 0.3), z: 0, w, d }) },
    { gap: hx0, far: door >= (hx0 + hx1) / 2, rect: (w: number, gap: number) => ({ x: hx0 - Math.min(0.6, gap - w - 0.3) - w, z: 0, w, d }) },
  ].sort((a, b) => b.gap - a.gap || Number(b.far) - Number(a.far));
  const blocked = (r: Rect) =>
    h.objects.some((o) => {
      const [fw, fd] = footprint(o.def);
      const [w, dd] = o.rot % 2 === 0 ? [fw, fd] : [fd, fw];
      return o.x < r.x + r.w && o.x + w > r.x && o.z < r.z + r.d && o.z + dd > r.z;
    });
  for (const side of sides) {
    const w = Math.min(3, side.gap - 0.6);
    if (w < 2.3) continue;
    const r = side.rect(w, side.gap);
    if (!blocked(r)) return r;
  }
  return null;
}

export function vacantSlots(town: NeighbourhoodDraft): PlotSlot[] {
  const occupied = new Set(town.households.map((h) => h.slot));
  return town.slots.filter((s) => s.kind === 'house' && !occupied.has(s.index));
}

/** Town geometry in world tiles: what `assembleTown` writes and the menu preview draws. */
export interface TownLayout {
  walls: Wall[];
  doors: Door[];
  windows: Door[];
  objects: Placement[];
  /** Garden paths (1 m) and driveways (wider than `DRIVEWAY_MIN_WIDTH`). */
  paths: Rect[];
  /** Farm fields (see `WorldLook.fields`). */
  fields: Rect[];
  plots: { name: string; x: number; z: number; w: number; d: number; public: boolean; entry: [number, number]; storeys: number }[];
  /** Storeys of the tallest house: storey `k`'s walls and objects are on rows `k * depth ..` (see `WorldStructure.storeys`). */
  storeys: number;
  /** Whether a house has a basement: then basements are on the block of rows after the storeys', `storeys * depth ..`. */
  basement: boolean;
}

/**
 * Stamps every plot's house layout (or its public place: a park, a square, a shop) onto the town. `playerSlot` is left without a
 * driveway (it stays open for building); null gives every house one. Each generated house is
 * furnished for its household (`player`: a new one moving in at `playerSlot`).
 */
export function layoutTown(content: Content, t: Templates, town: NeighbourhoodDraft, playerSlot: number | null, player?: HouseholdDraft): TownLayout {
  const walls: Wall[] = [];
  const doors: Door[] = [];
  const windows: Door[] = [];
  const objects: Placement[] = [];
  const paths: Rect[] = [];
  const fields: Rect[] = [];
  const footprint = (def: string): [number, number] => content.object(def)?.footprint ?? [1, 1];

  /** A rectangle in plot coordinates onto the town. */
  const area = (s: PlotSlot, r: Rect): Rect => {
    const { width: pw, depth: pd } = slotDims(t, s);
    return s.rotated ? { x: s.x + pw - r.x - r.w, z: s.z + pd - r.z - r.d, w: r.w, d: r.d } : { x: s.x + r.x, z: s.z + r.z, w: r.w, d: r.d };
  };
  const point = (s: PlotSlot, [x, z]: [number, number]): [number, number] => {
    const { width: pw, depth: pd } = slotDims(t, s);
    return s.rotated ? [s.x + pw - x, s.z + pd - z] : [s.x + x, s.z + z];
  };
  /** `lift`: rows to the storey (see `TownLayout.storeys`). */
  const placeAll = (s: PlotSlot, list: Placement[], lift = 0) => {
    const { width: pw, depth: pd } = slotDims(t, s);
    for (const o of list) {
      const [fw, fd] = footprint(o.def);
      const [w, d] = o.rot % 2 === 0 ? [fw, fd] : [fd, fw];
      // Turned round with the plot, a thing set off its tile's middle stands off the other way.
      const extra = { ...(o.turn ? { turn: o.turn } : {}), ...(o.offset ? { offset: (s.rotated ? [-o.offset[0], -o.offset[1]] : o.offset) as [number, number] } : {}) };
      objects.push(
        s.rotated
          ? { def: o.def, x: s.x + pw - o.x - w, z: s.z + pd - o.z - d + lift, rot: (o.rot + 2) % 4, ...extra }
          : { def: o.def, x: s.x + o.x, z: s.z + o.z + lift, rot: o.rot, ...extra },
      );
    }
  };
  /** Walls, doors and windows of a layout (a house or a shop), `lift` rows up (see `TownLayout.storeys`). */
  const shell = (s: PlotSlot, f: { walls?: Wall[]; doors?: Door[]; windows?: Door[] }, lift = 0) => {
    const { width: pw, depth: pd } = slotDims(t, s);
    for (const [x0, z0, x1, z1] of f.walls ?? []) {
      const [a, b] = [point(s, [x0, z0]), point(s, [x1, z1])];
      walls.push([a[0], a[1] + lift, b[0], b[1] + lift]);
    }
    // Doors and windows sit on wall edges; rotating the plot moves an edge's start corner.
    const edge = (d: Door): Door =>
      !s.rotated
        ? { x: s.x + d.x, z: s.z + d.z + lift, axis: d.axis }
        : d.axis === 'x'
          ? { x: s.x + pw - 1 - d.x, z: s.z + pd - d.z + lift, axis: 'x' }
          : { x: s.x + pw - d.x, z: s.z + pd - 1 - d.z + lift, axis: 'z' };
    doors.push(...(f.doors ?? []).map(edge));
    windows.push(...(f.windows ?? []).map(edge));
  };
  /** A garden path from a front door (plot coordinates) to the street. */
  const pathTo = (s: PlotSlot, front: Door) => {
    const { depth: pd } = slotDims(t, s);
    const [px, pz] = point(s, [front.x, front.z]);
    const doorX = s.rotated ? px - 1 : px;
    paths.push(s.rotated ? { x: doorX, z: pz, w: 1, d: s.z + pd - pz } : { x: doorX, z: s.z, w: 1, d: pz - s.z });
  };
  const living = new Map(town.households.map((h) => [h.slot, h.household]));
  if (player && playerSlot !== null) living.set(playerSlot, player);
  const layoutOf = (s: PlotSlot) => {
    const who = living.get(s.index);
    const h = houseTemplate(t, s.template, who && residentsOf(content, who));
    return h && onLot(t, h, s, footprint);
  };
  const placeOf = (s: PlotSlot) => {
    const place = s.kind === 'house' ? undefined : t.parks.find((p) => p.id === s.template);
    return place && placeOnLot(t, place, s);
  };

  const plots = town.slots.map((s) => {
    const { width: pw, depth: pd } = slotDims(t, s);
    // Visitors arrive in front of the door (the first spawn point of the house layout).
    const spawn = layoutOf(s)?.spawns[0] ?? placeOf(s)?.entry;
    const entry: [number, number] = spawn ? point(s, spawn) : [s.x + pw / 2, s.rotated ? s.z + pd - 1.5 : s.z + 1.5];
    return { name: s.name, x: s.x, z: s.z, w: pw, d: pd, public: s.kind !== 'house', entry, storeys: layoutOf(s)?.upper ? 2 : 1 };
  });
  const storeys = Math.max(1, ...plots.map((p) => p.storeys));
  let basement = false;
  for (const s of town.slots) {
    if (s.kind !== 'house') {
      const place = placeOf(s);
      if (!place) continue;
      shell(s, place);
      placeAll(s, place.objects);
      if (place.doors?.[0]) pathTo(s, place.doors[0]);
      for (const r of place.paths ?? []) paths.push(area(s, r));
      continue;
    }
    // A farm's fields and farmyard (an empty farm lot keeps them too).
    if (s.farm) {
      placeAll(s, s.farm.objects);
      fields.push(...s.farm.fields.map((r) => area(s, r)));
    }
    const h = layoutOf(s);
    if (!h) continue;
    const { width: pw, depth: pd } = slotDims(t, s);
    shell(s, h);
    placeAll(s, h.objects);
    // The storey above, on its own rows of the lot.
    if (h.upper) {
      shell(s, h.upper, town.depth);
      placeAll(s, h.upper.objects, town.depth);
    }
    // The basement, on the rows after every storey's.
    if (h.basement) {
      shell(s, h.basement, storeys * town.depth);
      placeAll(s, h.basement.objects, storeys * town.depth);
      basement = true;
    }
    // A garden path from the front door to the street.
    if (h.doors[0]) pathTo(s, h.doors[0]);
    // Neighbours get a driveway beside the house (the renderer parks a car on it); the player's
    // plot stays open for building.
    const drive = s.index === playerSlot ? null : driveway(h, pw, pd, footprint);
    if (drive) paths.push(area(s, drive));
  }
  return { walls, doors, windows, objects, paths, fields, plots, storeys, basement };
}

/** What the renderer needs to know about a town's world (it survives save/load in `meta`). */
export function worldLook(town: NeighbourhoodDraft, fields: Rect[]): WorldLook | undefined {
  return town.world && { kind: town.world.kind, landscape: town.world.landscape, ...(fields.length ? { fields } : {}) };
}

export interface AssembleOptions {
  /** `player` is the neighbour household already living at the player's slot. */
  existing?: boolean;
  mode?: GameKind;
  lifespan?: Lifespan;
}

/**
 * Builds the simulation's town file. A new player household moves in as household 0, before the
 * neighbours. With `existing`, `player` is the neighbour household already
 * living at `playerSlot`: it stays where it is in the list (with its jobs and history) and only
 * becomes the player's. A new household on an empty lot gets money to build with in Living games.
 */
export function assembleTown(
  content: Content,
  t: Templates,
  town: NeighbourhoodDraft,
  player: HouseholdDraft,
  playerSlot: number,
  { existing = false, mode = 'living', lifespan = 'normal' }: AssembleOptions = {},
): string {
  const { walls, doors, windows, objects, paths, fields, plots, storeys, basement } = layoutTown(content, t, town, playerSlot, existing ? undefined : player);
  const footprint = (def: string): [number, number] => content.object(def)?.footprint ?? [1, 1];
  const point = (s: PlotSlot, [x, z]: [number, number]): [number, number] => {
    const { width: pw, depth: pd } = slotDims(t, s);
    return s.rotated ? [s.x + pw - x, s.z + pd - z] : [s.x + x, s.z + z];
  };

  if (existing && !town.households.some((h) => h.slot === playerSlot)) throw new Error('Nobody lives in that house.');
  // A new household first, then the neighbours (one of whom may be the player's).
  const neighbours = town.households.map((h) => ({ ...h, player: existing && h.slot === playerSlot }));
  const home = town.slots[playerSlot];
  const residents: { household: HouseholdDraft; slot: number; player: boolean }[] = existing
    ? neighbours
    : [{ household: player, slot: playerSlot, player: true }, ...neighbours];
  const { startingFunds, emptyLotFunds = 0 } = content.economy;
  const emptyLot = !houseTemplate(t, home.template);
  const households = residents.map((r, i) => ({
    name: r.household.name.trim(),
    plot: r.slot,
    player: r.player,
    ...(!existing && i === 0 && emptyLot && mode === 'living' ? { funds: startingFunds + emptyLotFunds } : {}),
  }));
  const sims: (SimSpawn & { job?: { career: string; level: number } })[] = [];
  const relationships: { a: number; b: number; preset: string }[] = [];
  residents.forEach((r, index) => {
    const slot = town.slots[r.slot];
    const template = houseTemplate(t, slot.template);
    const spawns = ((template && onLot(t, template, slot, footprint).spawns) ?? [[slotDims(t, slot).width / 2, 1.5]]).map((p) => point(slot, p));
    relationships.push(...householdBonds(r.household, sims.length));
    for (const spawn of householdSpawns(r.household, index, spawns)) {
      // Most neighbours already work somewhere (an existing household the player takes over too);
      // a newly arrived household finds jobs in game. Neighbours start somewhere on the ladder
      // (mostly the lower grades) with the skills for it.
      const tracks = content.careerCategories.flatMap((c) => c.tracks);
      const settled = !r.player || existing;
      const track = settled && tracks.length && Math.random() < 0.75 ? pick(tracks) : null;
      const level = Math.floor(Math.random() ** 2 * 6);
      sims.push(track ? { ...spawn, job: { career: track.id, level } } : spawn);
    }
  });

  // Neighbours have history with each other; a newly arrived player household doesn't.
  const newcomers = existing ? 0 : player.members.length;
  for (let a = newcomers; a < sims.length; a++) {
    for (let b = a + 1; b < sims.length; b++) {
      if (sims[a].household === sims[b].household) continue;
      const roll = Math.random();
      const preset = roll < 0.03 ? 'rivals' : roll < 0.09 ? 'friends' : roll < 0.3 ? 'acquaintances' : null;
      if (preset && content.bondPresets.includes(preset)) relationships.push({ a, b, preset });
    }
  }

  const meta: TownMeta = { kind: 'town', name: town.name, streets: town.streets, paths, seed: town.seed ?? (Math.random() * 2 ** 31) >>> 0, world: worldLook(town, fields) };
  // Residents leave town (for work) at both ends of every street.
  const exits = town.streets.flatMap((st) => [[0.5, st.z + st.d / 2], [town.width - 0.5, st.z + st.d / 2]]);
  return JSON.stringify({ mode, lifespan, width: town.width, depth: town.depth, storeys, ...(basement ? { basement } : {}), walls, doors, windows, objects, plots, households, sims, relationships, exits, meta });
}
