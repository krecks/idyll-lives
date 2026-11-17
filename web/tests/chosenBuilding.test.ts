/**
 * A building clicked in the play view (`ui/town/chosen.ts`): what it is (a place, a lot's house, a backdrop building),
 * its icon and pin, and who is inside now, as the town card and its pin say it.
 */
import { describe, expect, test } from 'bun:test';
import type { BackdropBuilding, PlotInfo, SimView, TownPlace } from '../src/core/protocol';
import { chosenAnchor, chosenIcon, chosenOf, insideText, whoIsIn, type ChosenTown } from '../src/ui/town/chosen';
import type { TagWorld } from '../src/ui/town/tags';

const plot = (id: number, name: string, pub: boolean, extra: Partial<PlotInfo> = {}): PlotInfo => ({ id, name, x: id * 20, z: 0, w: 16, d: 20, public: pub, entry: null, house: null, ...extra });
const places: TownPlace[] = [
  { name: 'Bakery', kind: 'shop', x: 45, z: 5, work: ['culinary'], area: 0, plot: 2 },
  { name: 'Town Offices', kind: 'office', x: 100, z: 100, work: ['business'], area: 0 },
];
const box = (x: number, z: number): [number, number][] => [
  [x, z],
  [x, z + 10],
  [x + 10, z + 10],
  [x + 10, z],
];
const backdrop: BackdropBuilding[] = [
  { poly: box(95, 95), y: 0, storeys: 4, roof: 'hip', use: 'office', look: 0, place: 1, area: 0 },
  { poly: box(200, 0), y: 0, storeys: 3, roof: 'gable', use: 'flats', look: 0, area: 0 },
];
const town: ChosenTown = {
  places,
  backdrop,
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
  test("a backdrop place: the place's card, nothing to look inside; another building: what it is", () => {
    const office = chosenOf({ kind: 'building', index: 0 }, town)!;
    expect(office).toEqual({ kind: 'place', place: 1, plot: null, title: 'Town Offices', detail: '' });
    expect(chosenIcon(office, town)).toBe('icon.career.business');
    const flats = chosenOf({ kind: 'building', index: 1 }, town)!;
    expect(flats).toEqual({ kind: 'building', building: 1, plot: null, title: 'Flats', detail: '' });
    expect(chosenAnchor(flats, town)).toEqual({ x: 205, z: 5, lift: 12 });
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
  test('a building nobody is simulated in says nothing; an empty one, nobody', () => {
    expect(whoIsIn({ kind: 'building', building: 1, plot: null, title: '', detail: '' }, world([]), new Map())).toBeNull();
    expect(insideText({ names: [], others: 0 })).toBe('Nobody inside now');
    expect(insideText({ names: ['Mika', 'Ines'], others: 0 })).toBe('Mika and Ines inside');
  });
});
