/**
 * The town's named places: its civic places (`CIVIC`) and the other kinds (`ZONE`), each a lot
 * (`placeLots.ts`), what kind of building each looks like from outside (`Kind`), their opening
 * hours and the careers working there, their names (`Namer`, `placeName`), and how big a place
 * each area of a town is for them (`areaSize`, `draftAreaSize`).
 */

import type { CareerCategory } from '../content/content';
import type { BuildingUse, LotLook, TownPlace, TownSize } from '../core/protocol';
import type { NeighbourhoodDraft } from './town';
import { pick } from './noise';
import { placeOfTemplate } from './placeLots';
import { placeOf, sizeByHomes, specOf } from './world';

type Rand = () => number;

/** What a building is, and how big a kind of it is: along the road (m), deep (m), storeys, roofs. */
export interface Kind {
  use: BuildingUse;
  w: [number, number];
  d: [number, number];
  storeys: [number, number];
  roofs: readonly LotLook['roof'][];
}
/** A kind of building: its use, how long along the road and how deep (m), its storeys and roofs. */
export const K = (use: BuildingUse, w: [number, number], d: [number, number], storeys: [number, number], roofs: readonly LotLook['roof'][]): Kind => ({ use, w, d, storeys, roofs });

/**
 * A town's civic places, each a lot (`placeLots.ts`), a main town's laid first, nearest its square (its
 * high street's middle: `macro.ts`): what it is called on a card, its kind (how it looks from outside,
 * and how big a building set by hand is), its name, the careers working there, and the smallest town
 * that has it; and what its name looks like (to know one in a town: `civicOf`).
 */
export interface Civic {
  label: string;
  kind: Kind;
  /** How it looks from outside in a place of a size where it differs from `kind` (a hamlet's or a village's hall is a low hall, not a town hall). */
  kinds?: Partial<Record<TownSize, Kind>>;
  place: TownPlace['kind'];
  name: (n: Namer) => string;
  work: string[];
  least: TownSize;
  /** The layout its lot is built from (`houses.json` `parks` of kind `place`) in a place of each size: `placeLots.ts` `placeTemplate`. */
  templates: Partial<Record<TownSize, string>>;
  /** What a card calls it in a smaller place than a town (a village's hall is its village hall): `civicLabel`. */
  labels?: Partial<Record<TownSize, string>>;
  /** Its gable end to the road (a church): deeper than wide, the ridge running back from the road. */
  endOn?: boolean;
  known: RegExp;
}
/** The town sizes, smallest first. */
const SIZES: readonly TownSize[] = ['hamlet', 'village', 'town', 'city'];
/** The same layout in a place of any size (`Civic.templates`). */
const every = (id: string): Record<TownSize, string> => ({ hamlet: id, village: id, town: id, city: id });
/** The civic places, in the order a main town lays them (an index into this is a civic place's in step 2's choices: `civicChoices`). */
export const CIVIC: readonly Civic[] = [
  // (A hamlet's hall is its parish's, a village's its own: the careers no other place takes work there.)
  { label: 'Town hall', kind: K('office', [16, 20], [12, 14], [3, 3], ['hip', 'mansard']), place: 'office', name: (n) => `${n.town} ${{ hamlet: 'Parish Hall', village: 'Village Hall', town: 'Town Hall', city: 'Town Hall' }[n.size]}`, work: ['politics'], least: 'hamlet', templates: { hamlet: 'parishHall', village: 'parishHall', town: 'townHall', city: 'townHall' }, kinds: { hamlet: K('hall', [12, 12], [10, 10], [1, 1], ['gable']), village: K('hall', [12, 12], [10, 10], [1, 1], ['gable']) }, labels: { hamlet: 'Parish hall', village: 'Village hall' }, known: / (Parish|Village|Town) Hall$/ },
  { label: 'Church', kind: K('church', [11, 13], [22, 28], [3, 3], ['gable']), place: 'church', name: (n) => `St ${pick(n.rand, SAINTS)}'s Church`, work: [], least: 'hamlet', templates: every('church'), endOn: true, known: /^St .+'s Church$/ },
  { label: 'School', kind: K('school', [28, 36], [14, 18], [2, 2], ['hip', 'flat']), place: 'school', name: (n) => `${n.town} School`, work: ['education'], least: 'village', templates: every('school'), known: / School$/ },
  { label: 'Police station', kind: K('office', [12, 16], [11, 13], [2, 3], ['flat', 'hip']), place: 'office', name: (n) => `${n.town} Police Station`, work: ['detective'], least: 'town', templates: every('police'), known: / Police Station$/ },
  { label: 'Fire station', kind: K('workshop', [16, 20], [14, 16], [2, 2], ['flat']), place: 'workshop', name: (n) => `${n.town} Fire Station`, work: ['firefighter'], least: 'town', templates: every('fireStation'), known: / Fire Station$/ },
  { label: 'Medical centre', kind: K('office', [14, 18], [11, 13], [2, 2], ['flat', 'hip']), place: 'office', name: (n) => `${n.street()} Medical Centre`, work: ['medicine'], least: 'town', templates: every('medical'), known: / Medical Centre$/ },
  { label: 'Leisure centre', kind: K('workshop', [22, 28], [16, 20], [2, 2], ['flat']), place: 'workshop', name: (n) => `${n.town} Leisure Centre`, work: ['sports'], least: 'town', templates: every('leisure'), known: / Leisure Centre$/ },
  { label: 'Bank', kind: K('office', [12, 16], [11, 13], [3, 4], ['flat', 'mansard']), place: 'office', name: (n) => `${n.last()} Bank`, work: ['finance', 'business'], least: 'town', templates: every('bank'), known: / Bank$/ },
  { label: 'Newspaper', kind: K('office', [12, 16], [11, 13], [3, 4], ['flat']), place: 'office', name: (n) => `The ${n.town} Gazette`, work: ['journalism', 'tv', 'influencer'], least: 'town', templates: every('newspaper'), known: /^The .+ Gazette$/ },
  { label: 'Works', kind: K('office', [16, 22], [12, 15], [3, 5], ['flat']), place: 'office', name: (n) => `${n.street()} Works`, work: ['it', 'science'], least: 'town', templates: every('works'), known: / Works$/ },
  { label: 'Solicitors', kind: K('office', [9, 12], [10, 12], [3, 3], ['mansard', 'gable']), place: 'office', name: (n) => `${n.last()} & ${n.last()} Solicitors`, work: ['law'], least: 'town', templates: every('solicitors'), known: / Solicitors$/ },
  { label: 'Army reserve centre', kind: K('office', [12, 16], [11, 13], [2, 2], ['flat']), place: 'office', name: () => 'Army Reserve Centre', work: ['military'], least: 'town', templates: every('armyReserve'), known: /^Army Reserve Centre/ },
  { label: 'Garage', kind: K('workshop', [14, 18], [12, 14], [1, 2], ['flat', 'gable']), place: 'workshop', name: (n) => `${n.last()}'s Garage`, work: ['mechanics', 'transport'], least: 'village', templates: every('garage'), known: /'s Garage$/ },
  { label: 'Builders', kind: K('workshop', [14, 18], [12, 14], [1, 2], ['gable']), place: 'workshop', name: (n) => `${n.last()} Builders`, work: ['trades'], least: 'village', templates: every('builders'), known: / Builders$/ },
  { label: 'Studios', kind: K('workshop', [10, 14], [10, 12], [2, 2], ['gable']), place: 'workshop', name: (n) => `${n.street()} Studios`, work: ['arts'], least: 'village', templates: every('studios'), known: / Studios$/ },
  { label: 'Inn', kind: K('restaurant', [12, 16], [12, 14], [2, 3], ['gable', 'hip']), place: 'restaurant', name: (n) => n.inn(), work: ['culinary', 'hospitality', 'music'], least: 'hamlet', templates: every('inn'), known: /^The \S+ \S+$/ },
  { label: 'Hotel', kind: K('restaurant', [16, 22], [12, 14], [3, 4], ['mansard', 'hip']), place: 'restaurant', name: (n) => `The ${n.town} Hotel`, work: ['hospitality'], least: 'town', templates: every('hotel'), known: / Hotel$/ },
  { label: 'Boutique', kind: K('shop', [8, 10], [10, 12], [2, 3], ['gable', 'mansard']), place: 'shop', name: (n) => (n.size === 'hamlet' ? `${n.last()}'s Village Shop` : `${n.last()}'s Boutique`), work: ['fashion', 'retail'], least: 'hamlet', templates: { hamlet: 'villageShop', village: 'boutique', town: 'boutique', city: 'boutique' }, labels: { hamlet: 'Village shop' }, known: /'s (Village Shop|Boutique)$/ },
  { label: 'Garden centre', kind: K('shop', [14, 20], [12, 16], [1, 1], ['flat', 'gable']), place: 'shop', name: (n) => `${n.town} Garden Centre`, work: ['nature', 'retail'], least: 'village', templates: every('gardenCentre'), known: / Garden Centre$/ },
];
/** What a named place of the other kinds is (one that is no civic one: `placeOf`). */
export type ZoneKind = 'shop' | 'cafe' | 'restaurant' | 'office' | 'workshop' | 'farm';
/**
 * The named places besides the civic ones, by kind, as lots (`placeLots.ts`): the layouts a lot of
 * the kind is built from (`houses.json` `parks` of kind `place`, one drawn for each), its compact
 * layouts (`compact`: a narrow lot of a street built up shoulder to shoulder, `dense.ts`), the kind of
 * building it looks like from outside (its roof: `LotLook`), and how many a main town of
 * each size has (`lots`, fewest and most) and every other area (`around`): as many as a town named
 * of the kind when these were buildings nobody went into (measured over the places' sample worlds,
 * 2026-11-17). A farm shop is a farmstead's own, one by each (`macro.ts`), none of these.
 */
export const ZONE: Record<ZoneKind, { templates: readonly string[]; compact: readonly string[]; kind: Kind; lots: Record<TownSize, [number, number]>; around: [number, number] }> = {
  shop: { templates: ['shopSmall', 'shopGoods'], compact: ['shopTiny'], kind: K('shop', [10, 12], [10, 12], [2, 3], ['gable', 'flat', 'mansard']), lots: { hamlet: [8, 12], village: [10, 14], town: [8, 13], city: [10, 14] }, around: [0, 2] },
  cafe: { templates: ['cafeSmall'], compact: ['cafeTiny'], kind: K('cafe', [10, 12], [10, 12], [2, 3], ['gable', 'mansard']), lots: { hamlet: [3, 5], village: [4, 6], town: [4, 6], city: [5, 7] }, around: [0, 2] },
  restaurant: { templates: ['restaurant'], compact: ['restaurantSmall'], kind: K('restaurant', [13, 15], [12, 14], [2, 3], ['gable', 'hip']), lots: { hamlet: [1, 2], village: [1, 2], town: [1, 2], city: [2, 3] }, around: [0, 0] },
  office: { templates: ['officeSuite'], compact: ['officeSmall'], kind: K('office', [15, 17], [12, 14], [2, 4], ['flat', 'mansard', 'hip']), lots: { hamlet: [1, 3], village: [3, 5], town: [2, 4], city: [3, 5] }, around: [0, 0] },
  workshop: { templates: ['workshopUnit'], compact: ['workshopSmall'], kind: K('workshop', [15, 17], [12, 14], [1, 2], ['gable', 'flat']), lots: { hamlet: [0, 2], village: [2, 4], town: [2, 4], city: [3, 5] }, around: [0, 2] },
  farm: { templates: ['farmShop'], compact: [], kind: K('farm', [9, 11], [9, 11], [1, 2], ['gable']), lots: { hamlet: [0, 0], village: [0, 0], town: [0, 0], city: [0, 0] }, around: [0, 0] },
};
const SAINTS = ['Mary', 'John', 'Peter', 'Michael', 'Andrew', 'James', 'Anne', 'Catherine', 'Luke', 'Margaret', 'Martin', 'Nicholas'];
const SHOPS = ['Bakery', 'Butcher', 'Greengrocer', 'Bookshop', 'Hardware', 'Chemist', 'Florist', 'Newsagent', 'Toy Shop', 'Deli', 'Tailor', 'Cobbler', 'Fishmonger', 'Gift Shop', 'Bike Shop', 'Record Shop', 'Sweet Shop', 'Wool Shop', 'Antiques', 'Opticians'];
const CAFES = ['Tea Rooms', 'Coffee House', 'Café', 'Kitchen', 'Espresso Bar', 'Tea Shop'];
const INN_A = ['Red', 'Golden', 'Black', 'White', 'Green', 'Old', 'Royal', 'Blue', 'Silver', 'Merry'];
const INN_B = ['Lion', 'Hart', 'Swan', 'Bell', 'Crown', 'Anchor', 'Plough', 'Fox', 'Stag', 'Oak', 'Boar', 'Hound'];
const OFFICES = ['House', 'Chambers', 'Court', 'Buildings'];
const WORKSHOPS = ['Joinery', 'Pottery', 'Forge', 'Printworks', 'Upholstery', 'Glassworks', 'Bike Repairs', 'Smithy'];

/** What a kind of place is open (minutes of the day, [open, close)) and on which days (bit 0 Monday) before its workers' shifts widen it. */
export const HOURS: Record<Exclude<TownPlace['kind'], 'park' | 'outdoor'>, NonNullable<TownPlace['hours']>> = {
  shop: { open: 9 * 60, close: 18 * 60, days: 0b0111111 },
  cafe: { open: 7 * 60 + 30, close: 18 * 60, days: 0b1111111 },
  restaurant: { open: 12 * 60, close: 23 * 60, days: 0b1111111 },
  office: { open: 8 * 60, close: 18 * 60, days: 0b0011111 },
  school: { open: 8 * 60, close: 16 * 60, days: 0b0011111 },
  church: { open: 8 * 60, close: 20 * 60, days: 0b1111111 },
  workshop: { open: 7 * 60 + 30, close: 17 * 60, days: 0b0111111 },
  farm: { open: 6 * 60, close: 20 * 60, days: 0b1111111 },
};
/** The sim's own shops (public lots): open as long as an outing may go there (content `visits.outingHomeBy`, 23:00). */
export const LOT_HOURS: NonNullable<TownPlace['hours']> = { open: 7 * 60, close: 23 * 60, days: 0b1111111 };
/** The careers working at the sim's public places, by template. */
export const LOT_WORK: Record<string, string[]> = { cafe: ['culinary', 'hospitality'], grocer: ['retail'], bakery: ['culinary', 'retail'], market: ['retail', 'nature'] };
/** The careers working at a named place of a kind besides the civic ones. */
export const KIND_WORK: Partial<Record<TownPlace['kind'], string[]>> = { shop: ['retail'], cafe: ['culinary', 'hospitality'], restaurant: ['culinary', 'hospitality'], office: ['business'], workshop: ['trades'], farm: ['nature'] };
/** The kinds of named place a building may be (not a park or the outdoors). */
export const PLACE_KINDS: readonly TownPlace['kind'][] = ['shop', 'cafe', 'restaurant', 'office', 'school', 'church', 'workshop', 'farm'];

/** Opening hours for a place of `kind` where `work`'s careers work: the kind's, widened to every shift of theirs (a career's own and its tracks'). */
export function hoursFor(base: NonNullable<TownPlace['hours']>, work: readonly string[], categories: readonly CareerCategory[]): NonNullable<TownPlace['hours']> {
  let { open, close, days } = base;
  for (const id of work) {
    const c = categories.find((k) => k.id === id);
    for (const shift of [c?.shift, ...(c?.tracks ?? []).map((t) => t.shift)]) {
      if (!shift) continue;
      open = Math.min(open, Math.round(shift.start * 60));
      close = Math.max(close, Math.min(24 * 60, Math.round((shift.start + shift.hours) * 60)));
      for (const d of shift.days) days |= 1 << d;
    }
  }
  return { open, close, days };
}

/** Names for places, none twice. */
export interface Namer {
  rand: Rand;
  town: string;
  /** How big the main town is (a hamlet's area: as its main town). */
  size: TownSize;
  last(): string;
  street(): string;
  inn(): string;
  /** A name `draw` gives that no place has yet. */
  unique(draw: () => string): string;
}

/**
 * The names for places in `town` (an area's name), drawn from `rand`: a family name of `lastNames`,
 * a street's own word ("Mill" of "Mill Lane, Ashby": one of `streets`, else the town's name), an
 * inn's, and a name nobody in `taken` has (drawn again a few times, then numbered; then taken).
 */
export function namerFor(rand: Rand, town: string, size: TownSize, lastNames: readonly string[], streets: readonly string[], taken: Set<string>): Namer {
  return {
    rand,
    town,
    size,
    last: () => pick(rand, lastNames),
    // (A street's own word: "Mill" of "Mill Lane, Ashby".)
    street: () => (streets.length ? pick(rand, streets).split(',')[0].replace(/ (Street|Road|Lane|Way|Avenue|Row|Close|Drive|Hill|Walk|Crescent|Place|Gardens|Terrace|Parade|Square|Rise|Green|End|Mews|Court)$/, '') : town),
    inn: () => `The ${pick(rand, INN_A)} ${pick(rand, INN_B)}`,
    unique: (draw: () => string) => {
      // (Drawn again a few times before it gets a number.)
      let name = draw();
      for (let k = 0; k < 8 && taken.has(name); k++) name = draw();
      let out = name;
      for (let k = 2; taken.has(out); k++) out = `${name} ${k}`;
      taken.add(out);
      return out;
    },
  };
}

/** What a card calls civic place `c` in a place of `size` (its own label, else the size's: `Civic.labels`). */
export function civicLabel(c: Civic, size: TownSize): string {
  return c.labels?.[size] ?? c.label;
}

/**
 * How big area `area` of a town whose main town is `main` is, for its civic places and their names: the main town
 * (area 0) its own size; a village (any other area) a village, and no bigger than its main town (round a hamlet,
 * a hamlet). A village is offered a village's civic places (`Civic.least`) and its hall is a village hall.
 */
export function areaSize(main: TownSize, area: number): TownSize {
  return area === 0 ? main : SIZES[Math.min(SIZES.indexOf(main), SIZES.indexOf('village'))];
}

/**
 * How big area `area` of `draft` is, for the civic places it is offered by hand and their names: a village as
 * `areaSize` has it; the main town the bigger of its place's size (`draftSize`) and the size its homes make it
 * (`world.ts` `sizeByHomes`), so a hamlet built out to a village's homes is offered a village's places.
 */
export function draftAreaSize(draft: NeighbourhoodDraft, area: number): TownSize {
  const main = draftSize(draft);
  if (area !== 0) return areaSize(main, area);
  let homes = 0;
  for (const s of draft.slots) if (s.kind === 'house' && (s.area ?? 0) === 0) homes++;
  const grown = sizeByHomes(homes);
  return SIZES.indexOf(grown) > SIZES.indexOf(main) ? grown : main;
}

/**
 * Whether civic place `c` is for a bigger place than one of `size` (`Civic.least`, `draftAreaSize`): a village is given
 * a village's by hand, and the main town its own size's (a hamlet has no police station until it has grown).
 */
export function civicTooBig(c: Civic, size: TownSize): boolean {
  return SIZES.indexOf(c.least) > SIZES.indexOf(size);
}

/**
 * The civic places `options` (indices into `CIVIC`) in the order they are offered to an area whose places employ
 * `employed` careers: first those whose careers nobody there employs, then those with some of theirs nobody does,
 * then the rest (a church employs nobody); each group in `CIVIC`'s order.
 */
export function byNeed(options: readonly number[], employed: ReadonlySet<string>): number[] {
  const need = (i: number) => {
    const free = CIVIC[i].work.filter((w) => !employed.has(w)).length;
    return free && free === CIVIC[i].work.length ? 0 : free ? 1 : 2;
  };
  return [...options].sort((a, b) => need(a) - need(b));
}

/** Which civic place `place` is (an index into `CIVIC`), or -1: its kind, the careers it was made for first, and its name's look. */
export function civicOf(place: TownPlace | undefined): number {
  if (!place) return -1;
  return CIVIC.findIndex((c) => c.place === place.kind && c.work.every((w, i) => place.work?.[i] === w) && c.known.test(place.name));
}

/**
 * The civic places area `area` of `draft` (the main town unless given) does not have yet, of its size
 * (`draftAreaSize`, `civicTooBig`: a village has no police station), for step 2's Lot tool to set by hand, those for
 * careers nobody there employs first (`byNeed`): each one's index into `CIVIC`, what it is called on a card
 * (`civicLabel`), and how big its building may be (m along the road, m deep).
 */
export function civicChoices(draft: NeighbourhoodDraft, area = 0): { index: number; name: string; w: [number, number]; d: [number, number] }[] {
  const places = (draft.open?.places ?? []).filter((p) => p.area === area);
  // (Its places' and its lots': a civic place's lot is known by its layout, `placeOfTemplate`.)
  const has = new Set(places.map(civicOf));
  for (const s of draft.slots) {
    const what = (s.area ?? 0) === area && s.kind === 'shop' ? placeOfTemplate(s.template) : null;
    if (what && 'civic' in what) has.add(what.civic);
  }
  const size = draftAreaSize(draft, area);
  const open = CIVIC.flatMap((c, index) => (has.has(index) || civicTooBig(c, size) ? [] : [index]));
  return byNeed(open, new Set(places.flatMap((p) => p.work ?? []))).map((index) => {
    const c = CIVIC[index];
    return { index, name: civicLabel(c, size), w: c.kind.w, d: c.kind.d };
  });
}

/** How big `draft`'s main town is (its place's and options' `civic`). */
export function draftSize(draft: NeighbourhoodDraft): TownSize {
  return specOf(placeOf(draft.world?.kind ?? 'country'), draft.world?.options).civic;
}

/** A named place's name (not a civic one's). */
export function placeName(kind: TownPlace['kind'], n: Namer): string {
  switch (kind) {
    case 'shop':
      return n.rand() < 0.5 ? `${n.last()}'s ${pick(n.rand, SHOPS)}` : `${n.street()} ${pick(n.rand, SHOPS)}`;
    case 'cafe':
      return n.rand() < 0.5 ? `${n.last()}'s ${pick(n.rand, CAFES)}` : `The ${n.street()} ${pick(n.rand, CAFES)}`;
    case 'restaurant':
      return n.inn();
    case 'office':
      return `${n.street()} ${pick(n.rand, OFFICES)}`;
    case 'workshop':
      return `${n.last()}'s ${pick(n.rand, WORKSHOPS)}`;
    case 'school':
      return `${n.town} School`;
    case 'church':
      return `St ${pick(n.rand, SAINTS)}'s Church`;
    default:
      return `${n.town} Farm`;
  }
}
