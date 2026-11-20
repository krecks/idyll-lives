/**
 * A building clicked in the play view (`ui/town/chosen.ts`): what it is (a place, a building of flats, a lot's house),
 * its icon and pin, a building's flats, and who is inside now, as the town card and its pin say it.
 */
import { describe, expect, test } from 'bun:test';
import type { PlotInfo, SimView, TownPlace } from '../src/core/protocol';
import { FLAT_FLOOR } from '../src/game/price';
import { chosenAnchor, chosenIcon, chosenOf, flatRows, flatsDetail, insideText, placeDetail, placeLabel, whoIsIn, type ChosenTown } from '../src/ui/town/chosen';
import { FLAT_FLOOR } from '../src/game/price';
import { homesForSale } from '../src/ui/town/places';
import type { TagWorld } from '../src/ui/town/tags';
import { BUILDING, FLATS, realTown } from './fixtures/realTown';

const plot = (id: number, name: string, pub: boolean, extra: Partial<PlotInfo> = {}): PlotInfo => ({ id, name, x: id * 20, z: 0, w: 16, d: 20, public: pub, entry: null, house: null, ...extra });
const places: TownPlace[] = [
  { name: 'Bakery', kind: 'shop', x: 45, z: 5, work: ['culinary'], area: 0, plot: 2 },
  { name: 'Town Offices', kind: 'office', x: 100, z: 100, work: ['business'], area: 0 },
];
const town: ChosenTown = {
  places,
  plots: [plot(0, '1 Mill Lane', false, { house: [2, 2, 12, 14], storeys: 2 }), plot(1, '2 Mill Lane', false), plot(2, 'Bakery', true)],
  households: [
    { name: 'Hart', plot: 0 },
    { name: 'Lowe', plot: 1 },
  ],
};

describe('what a click on a building chooses', () => {
  test("a neighbour's house: the household's lot, its icon a house, its pin over the house", () => {
    const c = chosenOf({ kind: 'lot', plot: 1 }, town)!;
    expect(c).toEqual({ kind: 'home', plot: 1, title: 'The Lowes', detail: '' });
    expect(chosenIcon(c, town)).toBe('icon.ui.home');
    const home = chosenOf({ kind: 'lot', plot: 0 }, town)!;
    expect(chosenAnchor(home, town)).toEqual({ x: 7, z: 8, lift: 9 });
  });
  test("a public lot: its place's card, with its lot to look inside", () => {
    expect(chosenOf({ kind: 'lot', plot: 2 }, town)).toEqual({ kind: 'place', place: 0, plot: 2, title: 'Bakery', detail: '' });
  });
});

describe("a building of flats' card (the real town: Mill Court, flats 1 to 4)", () => {
  const world = realTown();
  const t: ChosenTown = { places: [], plots: world.plots, flats: world.flats, households: world.households };

  test('the building: its card, its icon a home, its pin over its walls', () => {
    const c = chosenOf({ kind: 'lot', plot: BUILDING }, t)!;
    expect(c).toEqual({ kind: 'flats', plot: BUILDING, title: 'Mill Court', detail: '' });
    expect(chosenIcon(c, t)).toBe('icon.ui.home');
    expect(chosenAnchor(c, t)).toEqual({ x: 13.5, z: 12.5, lift: 9 });
  });

  test('its flats by storey, each with who lives there, or for sale and its price', () => {
    const households = world.households.filter((h) => h.plot !== FLATS[3]);
    const sale = homesForSale(world.plots, households, { perTile: 10, perFloorTile: 100, round: 10 }, world.flats);
    expect(sale.map((h) => h.plot)).toEqual([FLATS[3]]);
    const rows = flatRows(BUILDING, { ...t, households }, sale);
    expect(rows.map((r) => [r.plot, r.name, r.storey, r.who])).toEqual([
      [1, 'Flat 1', 0, 'The Ashs'],
      [2, 'Flat 2', 0, 'The Birchs'],
      [3, 'Flat 3', 1, 'The Coles'],
      [4, 'Flat 4', 1, null],
    ]);
    // (The lot of 520 m² shared by four flats, and its own 63 m² at a flat's rate.)
    expect(rows[3].price).toBe(Math.round((10 * 130 + FLAT_FLOOR * 100 * 63) / 10) * 10);
    expect(rows.slice(0, 3).every((r) => r.price === null)).toBe(true);
    expect(flatsDetail(rows)).toBe('4 flats · 1 for sale');
    expect(flatsDetail(rows.slice(0, 3))).toBe('3 flats');
  });

  test("a place with flats over it is the place's card, its flats listed alike", () => {
    const shop: TownPlace = { name: 'Corner Shop', kind: 'shop', x: 13, z: 30, work: ['retail'], area: 0, plot: BUILDING };
    const mixed: ChosenTown = { ...t, places: [shop], plots: [{ ...world.plots[0], public: true, kind: 'shop' }] };
    expect(chosenOf({ kind: 'lot', plot: BUILDING }, mixed)).toEqual({ kind: 'place', place: 0, plot: BUILDING, title: 'Corner Shop', detail: '' });
    expect(flatRows(BUILDING, mixed, []).length).toBe(4);
    // (Over a place the land is shared with the place too: five shares of the 520 m².)
    const sale = homesForSale(mixed.plots, world.households.filter((h) => h.plot !== FLATS[3]), { perTile: 10, perFloorTile: 100, round: 10 }, world.flats);
    expect(sale.map((h) => [h.plot, h.price])).toEqual([[FLATS[3], Math.round((10 * 104 + FLAT_FLOOR * 100 * 63) / 10) * 10]]);
  });

  test('inside: those in its flats, its hall and its yard', () => {
    const sim = (id: number, plot: number | null) => ({ id, detail: false, name: `S${id}`, household: 9, plot, awayUntil: null, visiting: null }) as unknown as SimView;
    const w = { sims: [sim(1, FLATS[0]), sim(2, BUILDING), sim(3, FLATS[2]), sim(4, null)], flats: world.flats, plots: world.plots } as unknown as TagWorld;
    expect(whoIsIn({ kind: 'flats', plot: BUILDING, title: '', detail: '' }, w, new Map([[3, 'Cara']]))).toEqual({ names: ['Cara'], others: 2 });
    expect(whoIsIn({ kind: 'home', plot: FLATS[0], title: '', detail: '' }, w, new Map())).toEqual({ names: [], others: 1 });
  });
});

describe("a place's card line", () => {
  const t = (m: number) => `${String(Math.floor(m / 60)).padStart(2, '0')}:${String(m % 60).padStart(2, '0')}`;
  const days = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
  const fire: TownPlace = { name: 'Ashby Fire Station', kind: 'workshop', x: 0, z: 0, work: ['firefighter'], area: 0, plot: 4 };
  const hall: TownPlace = { name: 'Millbrook Village Hall', kind: 'office', x: 0, z: 0, work: ['politics'], area: 1, plot: 5 };
  const school: TownPlace = { name: 'Ashby School', kind: 'school', x: 0, z: 0, work: ['education'], area: 0, plot: 6 };

  test("a civic place says what it is (a village's hall its village hall); a shop or an office its name alone", () => {
    expect(placeLabel(fire)).toBe('Fire station');
    expect(placeLabel(hall)).toBe('Village hall');
    expect(placeLabel({ ...hall, name: 'Ashby Town Hall', area: 0 })).toBe('Town hall');
    expect(placeLabel(places[0])).toBe('');
    expect(placeLabel(places[1])).toBe('');
    expect(placeLabel(undefined)).toBe('');
  });

  test('then whether it is open, and till when or when it opens', () => {
    const shift = { open: 7 * 60, close: 17 * 60, days: 0b1111111 };
    expect(placeDetail(fire, shift, 10 * 60, 0, t, days)).toBe('Fire station · open till 17:00');
    const term = { open: 8 * 60, close: 16 * 60, days: 0b0011111 };
    expect(placeDetail(school, term, 12 * 60, 5, t, days)).toBe('School · closed · opens Mon 08:00');
    expect(placeDetail(places[0], { open: 9 * 60, close: 18 * 60, days: 0b1111111 }, 10 * 60, 0, t, days)).toBe('open till 18:00');
    expect(placeDetail(places[0], undefined, 10 * 60, 0, t, days)).toBe('');
  });

  test('a place lot is a place: Visit and Look inside go to its lot', () => {
    const t2: ChosenTown = { ...town, places: [...places, school], plots: [...town.plots, plot(3, 'x', false), plot(4, 'x', false), plot(5, 'x', false), plot(6, 'Ashby School', true)] };
    expect(chosenOf({ kind: 'lot', plot: 6 }, t2)).toEqual({ kind: 'place', place: 2, plot: 6, title: 'Ashby School', detail: '' });
    expect(chosenIcon(chosenOf({ kind: 'lot', plot: 6 }, t2)!, t2)).toBe('icon.career.education');
  });
});

describe('who is inside', () => {
  const sim = (id: number, extra: Partial<SimView> = {}): SimView =>
    ({ id, detail: false, name: `S${id}`, household: 1, traits: [], perks: [], plot: null, awayUntil: null, visiting: null, job: null, needs: [], mood: 0, emotion: null, feelings: [], actions: [], skills: [], ...extra }) as SimView;
  const world = (sims: SimView[]): TagWorld => ({
    sims,
    roster: sims.map((s) => ({ id: s.id, gender: 'female' })),
    households: [
      { id: 0, name: 'Hart', plot: 0, player: true },
      { id: 1, name: 'Lowe', plot: 1, player: false },
    ],
    plots: town.plots,
    relationships: [],
    places,
    placeOfPlot: new Map([[2, 0]]),
    minute: 600,
    weekday: 0,
    time: (m) => String(m),
    days: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
    pupil: () => false,
  });
  const job = { category: 'business', days: [0, 1, 2, 3, 4], startHour: 9 } as SimView['job'];

  test("in a house: those on its lot, yours by name; not those away at work", () => {
    const sims = [sim(1, { plot: 1 }), sim(2, { plot: 1 }), sim(3, { plot: 1, household: 0, name: 'Mika' }), sim(4, { plot: null, awayUntil: 1000, job })];
    const inside = whoIsIn({ kind: 'home', plot: 1, title: '', detail: '' }, world(sims), new Map([[3, 'Mika']]))!;
    expect(inside).toEqual({ names: ['Mika'], others: 2 });
    expect(insideText(inside)).toBe('Mika and 2 others inside');
  });
  test('at an office: those working there, as the place tags count them', () => {
    const sims = [sim(4, { awayUntil: 1000, job }), sim(5)];
    const inside = whoIsIn({ kind: 'place', place: 1, plot: null, title: '', detail: '' }, world(sims), new Map())!;
    expect(inside).toEqual({ names: [], others: 1 });
    expect(insideText(inside)).toBe('1 person inside');
  });
  test('at a shop with flats over it: those in the shop, and those at home in its flats', () => {
    const flat = (id: number) => ({ id, of: 2, storey: 1, x: 40, z: 0, w: 8, d: 10, name: `Flat ${id - 2}, Bakery`, door: [44, 30] as [number, number] });
    const w = { ...world([sim(1, { plot: 2 }), sim(2, { plot: 3, household: 2 }), sim(3, { plot: 4, household: 0, name: 'Mika' }), sim(4, { plot: 3, household: 2, awayUntil: 900, job }), sim(5, { plot: 1 })]), flats: [flat(3), flat(4)] };
    w.households = [...w.households, { id: 2, name: 'Upton', plot: 3, player: false }];
    const inside = whoIsIn({ kind: 'place', place: 0, plot: 2, title: '', detail: '' }, w, new Map([[3, 'Mika']]))!;
    expect(inside).toEqual({ names: ['Mika'], others: 2 });
  });
  test('a person says nothing; an empty building, nobody', () => {
    expect(whoIsIn({ kind: 'person', sim: 1, plot: null, title: '', detail: '' }, world([]), new Map())).toBeNull();
    expect(insideText({ names: [], others: 0 })).toBe('Nobody inside now');
    expect(insideText({ names: ['Mika', 'Ines'], others: 0 })).toBe('Mika and Ines inside');
  });
});
