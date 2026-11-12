/**
 * The town tags (docs/design/living-town.md, Town tags): kept from overlapping by importance (`ui/town/overlap.ts`),
 * workers shown at a place their category works at, stably by id (`ui/town/workplace.ts`), places from the public
 * lots before worldgen's, opening hours, homes for sale, the household's next events and the day bar.
 */
import { describe, expect, test } from 'bun:test';
import type { PlotInfo, RelationshipView, SimView, TownPlace } from '../src/core/protocol';
import { dayBar } from '../src/ui/town/dayBar';
import { keepApart, TAG_ORDER, type TagBox, type TagKind } from '../src/ui/town/overlap';
import { homesForSale, nameLabels, openState, openText, townPlaces } from '../src/ui/town/places';
import { whereabouts, yourPeople, type TagWorld } from '../src/ui/town/tags';
import { nextEvents, waitText } from '../src/ui/town/today';
import { schoolOf, workplaceOf } from '../src/ui/town/workplace';
import { tagFade } from '../src/ui/town/fade';

const box = (key: string, kind: TagKind, x: number, y: number, w = 100, h = 20, depth = 100): TagBox => ({ key, kind, x, y, w, h, depth });
const screen = { width: 1920, height: 1080 };

describe('keeping tags apart', () => {
  test('the order of importance is household, people, places, for sale, names', () => {
    expect(TAG_ORDER).toEqual(['household', 'people', 'places', 'sale', 'names']);
  });

  test('when two meet, the more important stays, whatever order they come in', () => {
    const kinds = [...TAG_ORDER];
    for (let i = 0; i < kinds.length; i++) {
      for (let j = i + 1; j < kinds.length; j++) {
        const a = box('a', kinds[i], 100, 100);
        const b = box('b', kinds[j], 150, 110);
        expect([...keepApart([b, a], screen).keys()]).toEqual(['a']);
        expect([...keepApart([a, b], screen).keys()]).toEqual(['a']);
      }
    }
  });

  test('tags that don’t meet all stay, and the gap between them counts', () => {
    const tags = [box('a', 'names', 0, 0), box('b', 'names', 104, 0), box('c', 'names', 0, 24)];
    expect(keepApart(tags, screen).size).toBe(3);
    // (Two pixels apart, under the three-pixel gap: they meet.)
    expect(keepApart([box('a', 'places', 0, 0), box('b', 'places', 102, 0)], screen).size).toBe(1);
    expect(keepApart([box('a', 'places', 0, 0), box('b', 'places', 102, 0)], { ...screen, gap: 0 }).size).toBe(2);
  });

  test('of a kind: the one shown last frame stays, else the nearer, else the first by key', () => {
    const near = box('near', 'places', 100, 100, 100, 20, 50);
    const far = box('far', 'places', 120, 100, 100, 20, 300);
    expect([...keepApart([far, near], screen).keys()]).toEqual(['near']);
    expect([...keepApart([far, near], { ...screen, shown: new Set(['far']) }).keys()]).toEqual(['far']);
    const x = box('x', 'sale', 0, 0, 100, 20, 80);
    const y = box('y', 'sale', 10, 0, 100, 20, 80);
    expect([...keepApart([y, x], screen).keys()]).toEqual(['x']);
  });

  test('the shown-last-frame rule never beats a more important kind', () => {
    const name = box('name', 'names', 100, 100);
    const home = box('home', 'household', 120, 105);
    expect([...keepApart([name, home], { ...screen, shown: new Set(['name']) }).keys()]).toEqual(['home']);
  });

  test('a pinned tag (the one clicked) stays over anything', () => {
    const chosen = { ...box('chosen', 'places', 100, 100), pinned: true };
    const home = box('home', 'household', 120, 105);
    expect([...keepApart([home, chosen], screen).keys()]).toEqual(['chosen']);
  });

  test('a tag wholly off the screen is dropped, one partly on it kept', () => {
    const kept = keepApart([box('left', 'places', -100, 10), box('edge', 'places', -50, 50), box('below', 'places', 10, 1080)], screen);
    expect([...kept.keys()]).toEqual(['edge']);
  });

  test('a chain: the first keeps its place, the second goes, the third (clear of the first) stays', () => {
    const tags = [box('a', 'household', 0, 0), box('b', 'people', 60, 0), box('c', 'places', 120, 0)];
    expect([...keepApart(tags, screen).keys()].sort()).toEqual(['a', 'c']);
  });

  test('two of the household in one spot: the second steps up over the first instead of going', () => {
    const a = { ...box('a', 'household', 100, 100), climb: 3 };
    const b = { ...box('b', 'household', 105, 100), climb: 3 };
    const kept = keepApart([a, b], screen);
    expect(kept.get('a')).toBe(0);
    expect(kept.get('b')).toBe(-23);
    // (Without the climb, the second goes.)
    expect([...keepApart([box('a', 'household', 100, 100), box('b', 'household', 105, 102)], screen).keys()]).toEqual(['a']);
    // Three steps at most.
    const many = Array.from({ length: 6 }, (_, i) => ({ ...box(`m${i}`, 'household', 100, 500), climb: 3 }));
    expect(keepApart(many, screen).size).toBe(4);
  });

  test('the HUD’s panels are kept clear: a tag under one goes, or steps up clear of it', () => {
    const panel = { x: 0, y: 0, w: 400, h: 60 };
    expect(keepApart([box('a', 'household', 50, 40)], { ...screen, avoid: [panel] }).size).toBe(0);
    expect(keepApart([box('a', 'household', 50, 100)], { ...screen, avoid: [panel] }).get('a')).toBe(0);
    const under = { x: 0, y: 200, w: 400, h: 60 };
    expect(keepApart([{ ...box('a', 'household', 50, 230), climb: 3 }], { ...screen, avoid: [under] }).get('a')).toBe(-69);
  });

  test('hundreds of tags across the grid’s cells match a plain pairwise check', () => {
    let seed = 7;
    const rand = () => ((seed = (Math.imul(seed, 1103515245) + 12345) >>> 0) / 2 ** 32);
    const tags: TagBox[] = [];
    for (let i = 0; i < 400; i++) tags.push(box(`t${i}`, TAG_ORDER[i % 5], rand() * 1900, rand() * 1060, 40 + rand() * 200, 18 + rand() * 10, rand() * 900));
    const kept = keepApart(tags, screen);
    const list = tags.filter((t) => kept.has(t.key));
    for (let i = 0; i < list.length; i++) {
      for (let j = i + 1; j < list.length; j++) {
        const a = list[i];
        const b = list[j];
        const meet = a.x < b.x + b.w + 3 && b.x < a.x + a.w + 3 && a.y < b.y + b.h + 3 && b.y < a.y + a.h + 3;
        expect(meet).toBe(false);
      }
    }
    // Every dropped tag meets a kept one at least as important.
    for (const t of tags.filter((t) => !kept.has(t.key))) {
      const blocked = list.some((k) => TAG_ORDER.indexOf(k.kind) <= TAG_ORDER.indexOf(t.kind) && t.x < k.x + k.w + 3 && k.x < t.x + t.w + 3 && t.y < k.y + k.h + 3 && k.y < t.y + t.h + 3);
      expect(blocked).toBe(true);
    }
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

  test('the tags fade in from a town blend of 0.2', () => {
    expect(tagFade(0)).toBe(0);
    expect(tagFade(0.2)).toBe(0);
    expect(tagFade(0.35)).toBeCloseTo(0.5);
    expect(tagFade(0.5)).toBe(1);
    expect(tagFade(1)).toBe(1);
  });
});
