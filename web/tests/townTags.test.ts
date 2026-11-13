/**
 * The town tags (docs/design/living-town.md, Town tags): ranked on screen and laid out as the menus' tags are
 * (`game/tagLayout.ts`), the household's own always shown, places thinned by zoom and a hamlet's only while looked into,
 * workers shown at a place their category works at, stably by id (`ui/town/workplace.ts`), places from the public
 * lots before worldgen's, opening hours, homes for sale, the household's next events and the day bar.
 */
import { describe, expect, test } from 'bun:test';
import type { PlotInfo, RelationshipView, SimView, TownPlace } from '../src/core/protocol';
import { dayBar } from '../src/ui/town/dayBar';
import { layoutTags, tagOrder, type Box, type TagSpot } from '../src/game/tagLayout';
import { TAG_ORDER } from '../src/ui/town/kinds';
import { homesForSale, nameLabels, openState, openText, townPlaces } from '../src/ui/town/places';
import { HAMLET_REACH, homeTag, mostPlaces, mostSales, SALE_MOST, SALE_REACH, nearestFew, placesInView, placeTags, TAG_RANK, tagRank, whereabouts, yourPeople, type TagWorld, type TownTag } from '../src/ui/town/tags';
import { nextEvents, waitText } from '../src/ui/town/today';
import { schoolOf, workplaceOf } from '../src/ui/town/workplace';
import { mineFade, tagFade } from '../src/ui/town/fade';

const SCREEN: Box = { l: 8, t: 8, r: 1912, b: 1072 };

/** A town tag's spot in the layout as `TownTags.svelte` makes it: a pill 120 × 30 px (its stem in it) over (`sx`, `sy`). */
const spot = (t: Pick<TownTag, 'kind' | 'tone' | 'ours'>, sx: number, sy: number, wins: number | null = t.kind === 'household' ? 1 : null): TagSpot => ({
  sx,
  sy,
  w: 120,
  h: 30,
  rank: tagRank(t),
  capped: false,
  wins,
  x: 0,
  y: 0,
  crowded: false,
});
const MINE = { kind: 'household', tone: 'mine' } as const;
const PERSON = { kind: 'people', tone: 'person' } as const;
const PLACE = { kind: 'places', tone: 'place' } as const;

describe('the town tags on screen', () => {
  test('the tag menu’s kinds: household, people, places, for sale, names', () => {
    expect(TAG_ORDER).toEqual(['household', 'people', 'places', 'sale', 'names']);
  });

  test('ranks: the household’s own, your people, places one of yours is at, areas, other places, homes for sale, streets', () => {
    const ranks = [MINE, PERSON, { ...PLACE, ours: true }, { kind: 'names', tone: 'area' }, PLACE, { kind: 'sale', tone: 'sale' }, { kind: 'names', tone: 'street' }].map((t) => tagRank(t as TownTag));
    expect(ranks).toEqual([0, TAG_RANK.people, TAG_RANK.ours, TAG_RANK.area, TAG_RANK.places, TAG_RANK.sale, TAG_RANK.street]);
    expect([...ranks].sort((a, b) => a - b)).toEqual(ranks);
  });

  test('the household’s own always show, clear of each other where they stand together; a place there gives way', () => {
    // Four of the household and a place at one spot (all at home, or at work together).
    const mine = Array.from({ length: 4 }, () => spot(MINE, 900, 500));
    const place = spot(PLACE, 905, 500);
    layoutTags([place, ...mine], SCREEN, []);
    expect(mine.every((t) => !t.crowded)).toBe(true);
    const boxes = mine.map((t) => ({ l: t.x - 60, t: t.y - 30, r: t.x + 60, b: t.y }));
    for (let i = 0; i < boxes.length; i++) for (let j = i + 1; j < boxes.length; j++) expect(boxes[i].l < boxes[j].r && boxes[j].l < boxes[i].r && boxes[i].t < boxes[j].b && boxes[j].t < boxes[i].b).toBe(false);
    expect(place.crowded).toBe(true);
  });

  test('a tag at the screen’s edge comes in whole: the household’s pulled in, the others only within reach of their point', () => {
    const mine = spot(MINE, 1900, 400);
    const person = spot(PERSON, 1900, 600);
    const place = spot(PLACE, 1870, 800);
    layoutTags([mine, person, place], SCREEN, []);
    expect(mine.crowded).toBe(false);
    expect(mine.x + 60).toBeLessThanOrEqual(SCREEN.r);
    // (The person’s pill would have to move 48 px sideways, 67 counted: too far to say whose it is; the place's 18.)
    expect(person.crowded).toBe(true);
    expect(place.crowded).toBe(false);
    expect(place.x + 60).toBeLessThanOrEqual(SCREEN.r);
  });

  test('your people take the room before the places, and a place one of yours is at before the others', () => {
    const other = spot(PLACE, 500, 500);
    const ours = spot({ ...PLACE, ours: true }, 505, 502);
    const person = spot(PERSON, 1000, 500);
    const placeByPerson = spot(PLACE, 1004, 498);
    layoutTags([other, ours, placeByPerson, person], SCREEN, []);
    expect([ours.crowded, other.crowded, person.crowded, placeByPerson.crowded]).toEqual([false, true, false, true]);
  });

  test('a tag shown last frame keeps its place before one of its kind nearer the camera', () => {
    const near = spot(PLACE, 500, 502);
    const held = { ...spot(PLACE, 505, 500), held: true };
    layoutTags([near, held], SCREEN, []);
    expect([held.crowded, near.crowded]).toEqual([false, true]);
    expect(tagOrder(held)).toBeLessThan(tagOrder(near));
  });

  test('the winners in their order: the clicked tag, the household’s own, the pointer’s', () => {
    const order = [spot(PLACE, 0, 900, 2), spot(MINE, 0, 100), spot(PERSON, 0, 50, 0), spot(MINE, 0, 700), spot(PERSON, 0, 1000)]
      .sort((a, b) => tagOrder(a) - tagOrder(b))
      .map((t) => [t.wins, t.sy]);
    expect(order).toEqual([
      [0, 50],
      [1, 700],
      [1, 100],
      [2, 900],
      [null, 1000],
    ]);
  });
});

describe('places by zoom', () => {
  const areas = [
    { x: 0, z: 0, w: 300, d: 300 },
    { x: 600, z: 0, w: 200, d: 150 },
    { x: 0, z: 600, w: 200, d: 150 },
  ];
  const place = (i: number, x: number, z: number, area: number, ours = false) => ({ key: `pl:${i}`, kind: 'places' as const, anchor: { x, z }, area, ours });
  // Thirty in the main town along a line out from (0, 0), two in each hamlet.
  const tags = [...Array.from({ length: 30 }, (_, i) => place(i, i * 10, 0, 0)), place(30, 700, 50, 1), place(31, 720, 60, 1), place(32, 100, 650, 2), place(33, 120, 700, 2, true)];

  test('fewer as the camera goes out: a street’s worth close in, a dozen or so at 400 m, six farthest out', () => {
    expect(mostPlaces(200)).toBe(27);
    expect(mostPlaces(400)).toBe(14);
    expect(mostPlaces(900)).toBe(6);
    for (let r = 200; r < 900; r += 50) expect(mostPlaces(r + 50)).toBeLessThanOrEqual(mostPlaces(r));
    // Homes for sale likewise: twelve close in, six at 400 m (none past `SALE_REACH`: `saleTags`).
    expect([mostSales(150), mostSales(200), mostSales(400), mostSales(SALE_REACH)]).toEqual([SALE_MOST, SALE_MOST, 6, 5]);
  });

  test('the main town’s nearest the middle of the view; a hamlet’s only while the camera looks into it; one of yours is at always', () => {
    const over = placesInView(tags, areas, { x: 0, z: 0 }, 400);
    expect(over.size).toBe(mostPlaces(400) + 1);
    expect([...Array.from({ length: mostPlaces(400) }, (_, i) => `pl:${i}`), 'pl:33'].every((k) => over.has(k))).toBe(true);
    expect(over.has('pl:30') || over.has('pl:32')).toBe(false);
    // Looking into the first hamlet (just off its edge counts): its places, nearest first, then the main town's.
    const hamlet = placesInView(tags, areas, { x: 600 - HAMLET_REACH + 1, z: 50 }, 900);
    expect(['pl:30', 'pl:31', 'pl:33'].every((k) => hamlet.has(k))).toBe(true);
    expect(hamlet.has('pl:32')).toBe(false);
    expect(hamlet.size).toBe(mostPlaces(900) + 1);
  });
});

describe('the few of a crowded kind', () => {
  test('the nearest stay, the first by key among equals; all of them when they are few', () => {
    const tags = [
      { key: 's:4', far: 90 },
      { key: 's:1', far: 20 },
      { key: 's:3', far: 55 },
      { key: 's:2', far: 20 },
    ];
    expect([...nearestFew(tags, 2)].sort()).toEqual(['s:1', 's:2']);
    expect([...nearestFew(tags, 3)].sort()).toEqual(['s:1', 's:2', 's:3']);
    expect(nearestFew(tags, 9).size).toBe(4);
    expect(nearestFew(tags, 0).size).toBe(0);
  });
});

describe('workplaces', () => {
  const places: Pick<TownPlace, 'work' | 'kind'>[] = [
    { kind: 'shop', work: ['retail'] },
    { kind: 'cafe', work: ['culinary', 'hospitality'] },
    { kind: 'restaurant', work: ['culinary'] },
    { kind: 'park' },
    { kind: 'office', work: ['it', 'finance'] },
    { kind: 'school', work: ['education'] },
  ];

  test('a worker shows at a place that lists their category', () => {
    expect(workplaceOf(3, 'retail', places)).toBe(0);
    expect(workplaceOf(3, 'finance', places)).toBe(4);
    for (let id = 0; id < 50; id++) expect([1, 2]).toContain(workplaceOf(id, 'culinary', places)!);
  });

  test('no place for the category, or no category: none ("at work")', () => {
    expect(workplaceOf(3, 'military', places)).toBeNull();
    expect(workplaceOf(3, null, places)).toBeNull();
    expect(workplaceOf(3, undefined, places)).toBeNull();
    expect(workplaceOf(3, 'retail', [])).toBeNull();
  });

  test('stable by resident id: the same answer every time, and with the places in another state of the game', () => {
    for (let id = 0; id < 50; id++) {
      const first = workplaceOf(id, 'culinary', places);
      expect(workplaceOf(id, 'culinary', places)).toBe(first);
      expect(workplaceOf(id, 'culinary', places.map((p) => ({ ...p })))).toBe(first);
    }
  });

  test('several places: residents spread over them', () => {
    const offices = Array.from({ length: 4 }, () => ({ work: ['it'] }));
    const counts = [0, 0, 0, 0];
    for (let id = 0; id < 200; id++) counts[workplaceOf(id, 'it', offices)!]++;
    for (const n of counts) expect(n).toBeGreaterThan(25);
  });

  test('pupils at a school of the town, stably; none without one', () => {
    expect(schoolOf(9, places)).toBe(5);
    expect(schoolOf(9, places.slice(0, 5))).toBeNull();
  });
});

const plot = (id: number, name: string, isPublic: boolean, extra: Partial<PlotInfo> = {}): PlotInfo => ({ id, name, x: id * 30, z: 0, w: 20, d: 30, public: isPublic, entry: null, house: null, ...extra });

describe('places', () => {
  test('without the town file’s places, the public lots are places, by name', () => {
    const plots = [plot(0, '1 Mill Lane', false), plot(1, 'The Bakery', true, { entry: [40, 31] }), plot(2, 'Corner Café', true), plot(3, 'Town Park', true), plot(4, 'Lookout', true)];
    const places = townPlaces({ pads: { 4: 12 } }, plots);
    expect(places.map((p) => [p.name, p.kind, p.plot])).toEqual([
      ['The Bakery', 'shop', 1],
      ['Corner Café', 'cafe', 2],
      ['Town Park', 'park', 3],
      ['Lookout', 'outdoor', 4],
    ]);
    expect([places[0].x, places[0].z]).toEqual([40, 31]);
    expect([places[2].x, places[2].z]).toEqual([100, 15]);
    expect(places[0].work).toEqual(['retail']);
    expect(places[1].work).toContain('culinary');
    expect(places[2].hours).toBeUndefined();
  });

  test('the town file’s places come as they are', () => {
    const own: TownPlace[] = [{ name: 'Mill Café', kind: 'cafe', x: 1, z: 2, area: 0 }];
    expect(townPlaces({ places: own }, [plot(1, 'The Bakery', true)])).toBe(own);
  });

  test('opening hours: open till, closed till, past midnight, other days', () => {
    const t = (m: number) => `${String(Math.floor(m / 60)).padStart(2, '0')}:${String(m % 60).padStart(2, '0')}`;
    const days = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
    const weekdays = { open: 9 * 60, close: 17 * 60, days: 0b0011111 };
    expect(openText(openState(weekdays, 10 * 60, 0), 0, t, days)).toBe('open till 17:00');
    expect(openText(openState(weekdays, 8 * 60, 0), 0, t, days)).toBe('closed · opens 09:00');
    expect(openText(openState(weekdays, 18 * 60, 0), 0, t, days)).toBe('closed · opens tomorrow 09:00');
    expect(openText(openState(weekdays, 18 * 60, 4), 4, t, days)).toBe('closed · opens Mon 09:00');
    expect(openState(weekdays, 12 * 60, 5).open).toBe(false);
    const late = { open: 18 * 60, close: 2 * 60, days: 0b1111111 };
    expect(openState(late, 23 * 60, 2)).toEqual({ open: true, until: 120 });
    expect(openState(late, 60, 2)).toEqual({ open: true, until: 120 });
    expect(openState(late, 3 * 60, 2).open).toBe(false);
    // (Friday only, late: open early on Saturday, not on Sunday.)
    const friday = { open: 20 * 60, close: 3 * 60, days: 0b0010000 };
    expect(openState(friday, 60, 5).open).toBe(true);
    expect(openState(friday, 60, 6).open).toBe(false);
    expect(openText(openState(undefined, 0, 0), 0, t, days)).toBe('');
  });

  test('homes for sale: lots nobody lives on, priced from land and house', () => {
    const plots = [plot(0, 'A', false, { house: [2, 2, 12, 10], storeys: 2 }), plot(1, 'B', false), plot(2, 'Park', true), plot(3, 'C', false, { house: [0, 0, 8, 8] })];
    const rates = { perTile: 10, perFloorTile: 100, round: 100 };
    const homes = homesForSale(plots, [{ plot: 3 }], rates);
    expect(homes.map((h) => h.plot)).toEqual([0, 1]);
    expect(homes[0]).toEqual({ plot: 0, price: 22000, floor: 160, storeys: 2, w: 20, d: 30 });
    expect(homes[1]).toEqual({ plot: 1, price: 6000, floor: 0, storeys: 0, w: 20, d: 30 });
    expect(homesForSale(plots, [], undefined)[0].price).toBe(0);
  });

  test('names: each street once, at its longest stretch’s middle; hamlets, at their centre', () => {
    const labels = nameLabels({
      roads: [
        { points: [[0, 0], [100, 0]], width: 8, ends: ['out', 'join'], name: 'High Street' },
        { points: [[0, 10], [10, 10]], width: 8, ends: ['join', 'join'], name: 'High Street' },
        { points: [[0, 50], [0, 90]], width: 2, ends: ['dead', 'dead'], name: 'Old Trail', kind: 'trail' },
        { points: [[0, 0], [0, 40]], width: 6, ends: ['dead', 'join'] },
      ],
      areas: [
        { name: 'Millbrook', kind: 'hamlet', x: 0, z: 0, w: 200, d: 200 },
        { name: 'The Fells', kind: 'countryside', x: 200, z: 0, w: 200, d: 200 },
      ],
      centre: [{ area: 0, poly: [[10, 10], [30, 10], [30, 30], [10, 30]] }],
    });
    expect(labels).toEqual([
      { key: 'street:High Street', text: 'High Street', x: 50, z: 0, area: false },
      { key: 'area:0', text: 'Millbrook', x: 20, z: 20, area: true },
    ]);
  });
});

describe('who and where', () => {
  const time = (m: number) => `${String(Math.floor(m / 60)).padStart(2, '0')}:${String(m % 60).padStart(2, '0')}`;
  const sim = (id: number, extra: Partial<SimView> = {}): SimView =>
    ({ id, detail: false, name: `S${id}`, household: 0, traits: [], perks: [], plot: null, awayUntil: null, visiting: null, job: null, needs: [], mood: 0, emotion: null, feelings: [], actions: [], skills: [], ...extra }) as SimView;
  const places: TownPlace[] = [{ name: 'Bakery', kind: 'shop', x: 5, z: 5, work: ['culinary'], area: 0, plot: 2 }];
  const world = (sims: SimView[], relationships: RelationshipView[] = []): TagWorld => ({
    sims,
    roster: sims.map((s) => ({ id: s.id, gender: 'female' })),
    households: [
      { id: 0, name: 'Hart', plot: 0, player: true },
      { id: 1, name: 'Lowe', plot: 1, player: false },
    ],
    plots: [plot(0, '1 Mill Lane', false), plot(1, '2 Mill Lane', false), plot(2, 'Bakery', true)],
    relationships,
    places,
    placeOfPlot: new Map([[2, 0]]),
    minute: 600,
    weekday: 0,
    time,
    days: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
    pupil: (stage) => stage === 'child',
  });
  const job = { category: 'culinary', days: [0, 1, 2, 3, 4], startHour: 9 } as SimView['job'];

  test('“Mara · at work · Bakery · home 17:10”', () => {
    const mara = sim(1, { name: 'Mara', awayUntil: 17 * 60 + 10, job });
    expect(whereabouts(mara, world([mara]))).toEqual({ place: 0, text: 'at work · Bakery · home 17:10', unplaced: false });
  });

  test('at work with no place for the career: “at work”, nowhere to stand', () => {
    const jo = sim(1, { awayUntil: 17 * 60, job: { ...job!, category: 'military' } });
    expect(whereabouts(jo, world([jo]))).toEqual({ place: null, text: 'at work · home 17:00', unplaced: true });
  });

  test('at home, at a public place, visiting, at school', () => {
    const w = world([]);
    expect(whereabouts(sim(1, { plot: 0 }), w).text).toBe('at home');
    expect(whereabouts(sim(1, { plot: 2 }), w)).toEqual({ place: 0, text: 'at Bakery', unplaced: false });
    expect(whereabouts(sim(1, { plot: 1, visiting: 1 }), w).text).toBe('visiting the Lowes');
    expect(whereabouts(sim(1, { stage: 'child', awayUntil: 15 * 60 }), w)).toEqual({ place: null, text: 'at school · home 15:00', unplaced: true });
  });

  test('a hamlet’s place whose name another has says which hamlet; the main town’s and a name of its own don’t', () => {
    const same: TownPlace[] = [
      { name: 'Bakery', kind: 'shop', x: 5, z: 5, area: 0 },
      { name: 'Bakery', kind: 'shop', x: 400, z: 5, area: 1 },
      { name: 'Mill Café', kind: 'cafe', x: 410, z: 5, area: 1 },
      { name: 'Bakery', kind: 'shop', x: 900, z: 5, area: 2 },
    ];
    const w = { ...world([]), places: same, placeOfPlot: new Map<number, number>(), areas: [{ name: 'Brightfield', kind: 'hamlet' as const }, { name: 'Millbrook', kind: 'hamlet' as const }, { name: 'Far Fields', kind: 'countryside' as const }] };
    const tags = placeTags(w, new Map());
    expect(tags.map((t) => t.detail)).toEqual(['', 'Millbrook', '', '']);
    expect(tags.map((t) => t.area)).toEqual([0, 1, 1, 2]);
  });

  test('your people: family, partners and friends, closest first; not acquaintances, not the household', () => {
    const rel = (a: number, b: number, extra: Partial<RelationshipView>): RelationshipView => ({ a, b, friendship: 0, romance: 0, partners: false, chemistry: 0, ...extra });
    const sims = [sim(1), sim(2), sim(3, { household: 1 }), sim(4, { household: 1 }), sim(5, { household: 1 }), sim(6, { household: 1 })];
    const w = world(sims, [rel(1, 2, { friendship: 90 }), rel(1, 3, { friendship: 40 }), rel(1, 4, { friendship: 10 }), rel(2, 5, { kin: 'sibling', friendship: 5 }), rel(1, 6, { romance: 70, partners: true, friendship: 60 })]);
    const people = yourPeople(new Set([1, 2]), w);
    expect(people.map((p) => [p.sim, p.label])).toEqual([
      [6, 'Partner'],
      [5, 'Sister'],
      [3, 'Friend'],
    ]);
  });
});

describe('today', () => {
  const rules = { school: { start: 8, days: [0, 1, 2, 3, 4] }, visits: new Set(['visit']), householdRoutines: [] };

  test('the next shift, school and planned visit, soonest first', () => {
    const members = [
      { id: 1, name: 'Mara', job: { days: [0, 1, 2], startHour: 9 }, pupil: false, plan: { routines: [{ id: 1, activity: 'visit', days: 0b1, start: 18 * 60, minutes: 60 }], skipHousehold: [] } },
      { id: 2, name: 'Tom', job: null, pupil: true },
    ];
    const events = nextEvents(members, rules, 7 * 60, 0);
    expect(events.map((e) => [e.name, e.kind, e.inDays, e.minute])).toEqual([
      ['Tom', 'school', 0, 480],
      ['Mara', 'shift', 0, 540],
      ['Mara', 'visit', 0, 1080],
    ]);
    // Past nine on a Wednesday: Mara's next shift is Monday's.
    const later = nextEvents(members, rules, 10 * 60, 2);
    expect(later.find((e) => e.kind === 'shift')).toMatchObject({ inDays: 5, minute: 540 });
  });

  test('household routine blocks count unless skipped', () => {
    const r = { ...rules, householdRoutines: [{ id: 7, activity: 'visit', days: 0b1111111, start: 20 * 60, minutes: 60 }] };
    const keep = { id: 1, name: 'A', job: null, pupil: false, plan: { routines: [], skipHousehold: [] } };
    const skip = { ...keep, id: 2, plan: { routines: [], skipHousehold: [7] } };
    expect(nextEvents([keep, skip], r, 600, 3).map((e) => e.sim)).toEqual([1]);
  });

  test('how long till', () => {
    const days = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
    expect(waitText({ wait: 40, inDays: 0 }, 0, days)).toBe('in 40 min');
    expect(waitText({ wait: 130, inDays: 0 }, 0, days)).toBe('in 2 h 10');
    expect(waitText({ wait: 600, inDays: 1 }, 0, days)).toBe('tomorrow');
    expect(waitText({ wait: 3000, inDays: 2 }, 5, days)).toBe('on Mon');
  });
});

describe('the day bar and the fade', () => {
  test('sunrise to sunset', () => {
    expect(dayBar(5 * 60)).toMatchObject({ share: 0, day: false });
    expect(dayBar(13 * 60)).toMatchObject({ share: 0.5, day: true });
    expect(dayBar(21 * 60)).toMatchObject({ share: 1, day: false });
  });

  test('the home has a tag of its own while nobody of the household is in', () => {
    const home = plot(0, '1 Mill Lane', false, { house: [2, 2, 12, 10], storeys: 2 });
    const s = (id: number, extra: Partial<SimView>) => ({ id, plot: null, awayUntil: null, ...extra }) as SimView;
    // Someone in: their own tag marks it.
    expect(homeTag(home, [s(1, { plot: 0 }), s(2, { awayUntil: 17 * 60 })], 0)).toBeNull();
    // Everyone out or at work: "Home", over the house, and a click glides down to it.
    const t = homeTag(home, [s(1, { plot: 3 }), s(2, { awayUntil: 17 * 60 })], 0)!;
    expect(t).toMatchObject({ kind: 'household', title: 'Home', tone: 'mine', anchor: { x: 7, z: 6 }, action: { kind: 'home' } });
    expect(t.lift).toBeGreaterThan(6);
    expect(homeTag(null, [], null)).toBeNull();
  });

  test('the tags fade in from a town blend of 0.2', () => {
    expect(tagFade(0)).toBe(0);
    expect(tagFade(0.2)).toBe(0);
    expect(tagFade(0.35)).toBeCloseTo(0.5);
    expect(tagFade(0.5)).toBe(1);
    expect(tagFade(1)).toBe(1);
    // The household's own are in from the town view's start: whole by 150 m, where the rest have not begun.
    expect(mineFade(0)).toBe(0);
    expect(mineFade(0.015)).toBeCloseTo(0.5);
    expect(mineFade(0.03)).toBe(1);
    expect(mineFade(1)).toBe(1);
    expect(tagFade(0.03)).toBe(0);
  });
});
