// Step 2's lot edits (`game/edit/lots.ts`) on a generated hamlet: a street filled with homes from
// the world's lot mix, a lot added only where the generator would put one (it fits, and a car can
// turn out of its driveway onto the road), a lot moved with its house, the houses along the streets
// touched numbered again, the town's last home and last park kept, the same seed the same houses,
// the pointer's check cheap, the garden, house, public place and a civic place's lot changed; and a town
// edited so has no problem the generated town hadn't (`townCheck.ts` `townProblems`).
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('lot edits', async () => {
  // --- the page's world in bun: content from web/public (put back for whatever runs next) ---
  const realLocation = globalThis.location;
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url, 'http://localhost/');
    const path = `${WEB}public${decodeURIComponent(url.pathname)}`;
    return existsSync(path) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
  }) as typeof fetch;
  afterAll(() => {
    globalThis.fetch = realFetch;
    Object.assign(globalThis, { location: realLocation });
  });
  const { Content } = await import('../src/content/content');
  const { AssetRegistry } = await import('../src/assets/registry');
  const { objectInfos } = await import('../src/game/housegen');
  const { randomHousehold } = await import('../src/game/household');
  const { mulberry32 } = await import('../src/game/noise');
  const { unrle } = await import('../src/game/landMasks');
  const { lengths } = await import('../src/game/roads');
  const { townProblems } = await import('../src/game/townCheck');
  const { assembleTown, boundsOf, frontageOf, generatedId, generateWorld, homesOn: homesOnLot, houseFor, LotPlacer, plotRect, useTownContent, vacantSlots } = await import('../src/game/town');
  const { entriesOf, groundOf, townGround } = await import('../src/game/edit/ground');
  const lots = await import('../src/game/edit/lots');
  type Templates = import('../src/game/town').Templates;
  type NeighbourhoodDraft = import('../src/game/town').NeighbourhoodDraft;
  type Edited = import('../src/game/edit/edited').Edited;
  type RoadSpot = import('../src/game/edit/ground').RoadSpot;
  type LotChoice = import('../src/game/edit/lots').LotChoice;
  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  /** A small generated world (a hamlet, no villages). */
  const hamlet = generateWorld(content, t, 'country', 4245, undefined, undefined, { town: 'hamlet', villages: 'none' });

  /** The draft an edit made (failing the test with its reason if it was refused). */
  const made = (e: Edited): NeighbourhoodDraft => {
    if ('refused' in e) throw new Error(`refused: ${e.refused}`);
    return e.draft;
  };
  /** Every spot along road `road` (or every road), every `step` m, both sides. */
  const spots = (draft: NeighbourhoodDraft, road?: number, step = 2): RoadSpot[] =>
    draft.roads.flatMap((r, i) => {
      if (road !== undefined && i !== road) return [];
      const ls = lengths(r.points);
      const out: RoadSpot[] = [];
      for (let s = 0; s < ls[ls.length - 1]; s += step) out.push({ road: i, s, side: 1 }, { road: i, s, side: -1 });
      return out;
    });
  /** The first spot (along `road`, or any) where `what` fits. */
  const fitting = (draft: NeighbourhoodDraft, what: LotChoice | { slot: number }, seed = 0, road?: number): RoadSpot => {
    const at = spots(draft, road).find((p) => !('refused' in lots.canPlace(draft, t, p, what, seed)));
    if (!at) throw new Error('nowhere fits');
    return at;
  };
  const homesOn = (draft: NeighbourhoodDraft, road: number) => {
    const e = entriesOf(draft, t);
    return draft.slots.filter((s, i) => s.kind === 'house' && s.lot !== 'farm' && e[i].road === road);
  };
  const overlap = (a: { x: number; z: number; w: number; d: number }, b: typeof a) => a.x < b.x + b.w && b.x < a.x + a.w && a.z < b.z + b.d && b.z < a.z + a.d;
  /** The houses along `road` numbered as the generator numbers them: odd on its left, even on its right, in turn along it. */
  const numberedInTurn = (draft: NeighbourhoodDraft, road: number) => {
    const e = entriesOf(draft, t);
    const name = draft.roads[road].name!;
    for (const side of [1, -1] as const) {
      const along = draft.slots.flatMap((s, i) => (s.kind === 'house' && s.lot !== 'farm' && e[i].road === road && e[i].side === side ? [{ s, at: e[i].s }] : [])).sort((a, b) => a.at - b.at);
      along.forEach(({ s }, k) => expect(s.name).toBe(`${k * 2 + (side > 0 ? 1 : 2)} ${name}`));
    }
  };

  test("a lot's key is where it stands to the whole metre, as the people's streams key its household", () => {
    expect(lots.lotKey({ x: 10.4, z: 20.6 })).toBe('10,21');
    expect(lots.lotKey({ x: -3.5, z: 7.5 })).toBe(`${Math.round(-3.5)},${Math.round(7.5)}`);
    const s = hamlet.slots[5];
    expect(lots.slotOf(hamlet, lots.lotKey(s))).toBe(5);
  });

  test('the ground has every lot, the car parks, the water with its shore and the roads on it', () => {
    const ground = townGround(hamlet, t);
    for (const s of hamlet.slots) expect(ground.free(plotRect(t, s), 0)).toBe(false);
    for (const p of hamlet.open!.carParks ?? []) expect(ground.free(boundsOf(p.poly), 0)).toBe(false);
    const wet = unrle(hamlet.open!.water, hamlet.width * hamlet.depth).indexOf(1);
    expect(wet).toBeGreaterThanOrEqual(0);
    expect(ground.takenAt((wet % hamlet.width) + 0.5, Math.floor(wet / hamlet.width) + 0.5)).toBe(true);
    const [x, z] = hamlet.roads[0].points[10];
    expect(ground.takenAt(x, z)).toBe(true);
    // A new one each time (the caller's to take from); the pointer's kept once per draft.
    expect(townGround(hamlet, t)).not.toBe(ground);
    expect(groundOf(hamlet, t)).toBe(groundOf(hamlet, t));
  });

  test('each home meets the road its address names', () => {
    const e = entriesOf(hamlet, t);
    const homes = hamlet.slots.filter((s) => s.kind === 'house' && s.lot !== 'farm');
    expect(homes.length).toBeGreaterThan(10);
    for (const s of homes) expect(s.name.endsWith(` ${hamlet.roads[e[s.index].road].name}`)).toBe(true);
    // Numbered again, a generated town's names stay as they were.
    expect(lots.renumber(hamlet, t, hamlet.roads.map((_, i) => i))).toBe(hamlet);
  });

  test('a street is filled with homes from the lot mix, off its corners, numbered, the same for the same seed', () => {
    const road = hamlet.roads.map((_, i) => i).sort((a, b) => homesOn(hamlet, b).length - homesOn(hamlet, a).length)[0];
    const gone = new Set(homesOn(hamlet, road).map((s) => s.index));
    expect(gone.size).toBeGreaterThan(3);
    const bare = lots.removeLots(hamlet, gone);
    expect(homesOn(bare, road).length).toBe(0);
    const filled = made(lots.fillStreet(bare, t, content, road, 11));
    const added = filled.slots.slice(bare.slots.length);
    expect(added.length).toBeGreaterThan(3);
    const ground = townGround(bare, t);
    const e = entriesOf(filled, t);
    for (const s of added) {
      expect(s.kind).toBe('house');
      expect(['small', 'medium', 'large']).toContain(s.lot!);
      expect(s.template.startsWith(`gen:${s.lot}:`)).toBe(true);
      expect(s.area).toBe(0);
      expect(e[s.index].road).toBe(road);
      // On free ground, apart from each other.
      expect(ground.free(plotRect(t, s), 0)).toBe(true);
      for (const o of added) if (o !== s) expect(overlap(plotRect(t, s), plotRect(t, o))).toBe(false);
    }
    numberedInTurn(filled, road);
    // The same seed, the same houses; another, others.
    expect(JSON.stringify(made(lots.fillStreet(bare, t, content, road, 11)))).toBe(JSON.stringify(filled));
    const other = made(lots.fillStreet(bare, t, content, road, 12)).slots.slice(bare.slots.length);
    expect(other.map((s) => s.template).join()).not.toBe(added.map((s) => s.template).join());
    // A footpath is not filled.
    const trail = hamlet.roads.findIndex((r) => r.kind === 'trail');
    expect(lots.fillStreet(hamlet, t, content, trail, 1)).toEqual({ refused: 'A footpath has no lots along it.' });
  });

  test("a lot is added only where it fits and a car can drive out, just where the generator's placer would put it", () => {
    const choice: LotChoice = { kind: 'house', size: 'medium', garden: 'roomy' };
    const u = { kind: 'house' as const, name: '', lot: 'medium' as const, ...houseFor(t, 'medium', 'roomy', mulberry32(7)) };
    const said: Record<string, number> = {};
    // At each place along every road the generator would try a lot (`frontageOf`), as `canPlace` and as `LotPlacer.place` with its driveway test.
    for (const [k, f] of frontageOf(hamlet.roads, null, [0, 0], () => 0).entries()) {
      if (k % 3) continue;
      const fit = lots.canPlace(hamlet, t, { road: f.road, s: f.s, side: f.side }, choice, 7);
      const why = 'refused' in fit ? fit.refused : 'fits';
      said[why] = (said[why] ?? 0) + 1;
      // (Off the corners, the town centre's frontage and the ends: the generator's frontage is filtered so before it places.)
      if (why !== 'fits' && !/room|driveway/.test(why)) continue;
      const placer = new LotPlacer(content, t, townGround(hamlet, t), hamlet.roads, [0, 0], mulberry32(1), true);
      placer.place([u], [f], [], [0, 0], undefined, true);
      expect(placer.slots.length).toBe(why === 'fits' ? 1 : 0);
      if ('refused' in fit) continue;
      const r = plotRect(t, placer.slots[0]);
      expect({ x: fit.x, z: fit.z, w: fit.w, d: fit.d }).toEqual(r);
    }
    expect(said.fits).toBeGreaterThan(5);
    expect(said['There is no room for it here.']).toBeGreaterThan(5);
    // Added where it fits: the lot `canPlace` showed, its house the one the seed draws, numbered on its road.
    const at = fitting(hamlet, choice, 7);
    const fit = lots.canPlace(hamlet, t, at, choice, 7);
    const town = made(lots.addLot(hamlet, t, content, at, choice, 7));
    const slot = town.slots[hamlet.slots.length];
    expect(plotRect(t, slot)).toEqual({ x: (fit as { x: number }).x, z: (fit as { z: number }).z, w: (fit as { w: number }).w, d: (fit as { d: number }).d });
    expect(slot.template).toBe(u.template);
    expect(slot.name).toMatch(new RegExp(` ${town.roads[at.road].name}$`));
    numberedInTurn(town, at.road);
    // Refused where it doesn't fit (on a lot already there), with the reason.
    const busy = entriesOf(hamlet, t)[hamlet.slots.findIndex((s) => s.kind === 'house' && s.lot === 'medium')];
    expect(lots.addLot(hamlet, t, content, busy, choice, 7)).toEqual({ refused: 'There is no room for it here.' });
  });

  test('farms, public places and outdoor spots are added with what goes with them', () => {
    const farm = made(lots.addLot(hamlet, t, content, fitting(hamlet, { kind: 'farm' }, 3), { kind: 'farm' }, 3));
    const f = farm.slots.at(-1)!;
    expect(f.lot).toBe('farm');
    expect(f.name).toMatch(/ Farm/);
    expect(f.farm?.objects.length).toBeGreaterThan(0);
    const cafe: LotChoice = { kind: 'shop', template: 'cafe' };
    const withCafe = made(lots.addLot(hamlet, t, content, fitting(hamlet, cafe), cafe, 1));
    const c = withCafe.slots.at(-1)!;
    const place = withCafe.open!.places!.find((p) => p.plot === c.index)!;
    expect(place.kind).toBe('cafe');
    expect(place.work).toEqual(['culinary', 'hospitality']);
    expect(place.hours).toBeDefined();
    const spot: LotChoice = { kind: 'outdoor', template: 'lookout' };
    const withSpot = made(lots.addLot(hamlet, t, content, fitting(hamlet, spot), spot, 1));
    const o = withSpot.slots.at(-1)!;
    expect(o.kind).toBe('outdoor');
    expect(withSpot.open!.places!.find((p) => p.plot === o.index)?.kind).toBe('outdoor');
    expect(withSpot.open!.pads[o.index]).toBeDefined();
    // A short trail into it from its road.
    const trail = withSpot.roads.at(-1)!;
    expect(trail.kind).toBe('trail');
    expect(withSpot.roads.length).toBe(hamlet.roads.length + 1);
    expect(lots.addLot(hamlet, t, content, fitting(hamlet, cafe), { kind: 'park', template: 'cafe' }, 1)).toHaveProperty('refused');
  });

  test('a lot moved keeps its house, garden, kind and household', () => {
    const e = entriesOf(hamlet, t);
    const from = hamlet.slots.find((s) => s.kind === 'house' && s.lot !== 'farm' && s.lot !== 'terrace')!;
    const left = e[from.index].road;
    // (A household living there goes with it.)
    const lived = { ...hamlet, households: [{ household: { name: 'Test' } as never, slot: from.index }] };
    const road = hamlet.roads.findIndex((r, i) => i !== left && r.kind !== 'trail' && spots(lived, i).some((p) => !('refused' in lots.canPlace(lived, t, p, { slot: from.index }))));
    const at = fitting(lived, { slot: from.index }, 0, road);
    const fit = lots.canPlace(lived, t, at, { slot: from.index });
    const town = made(lots.moveLot(lived, t, from.index, at));
    const moved = town.slots[from.index];
    for (const k of ['template', 'garden', 'lot', 'kind', 'w', 'd', 'index'] as const) expect(moved[k]).toEqual(from[k]);
    expect(lots.lotKey(moved)).not.toBe(lots.lotKey(from));
    expect(plotRect(t, moved)).toEqual({ x: (fit as { x: number }).x, z: (fit as { z: number }).z, w: (fit as { w: number }).w, d: (fit as { d: number }).d });
    expect(town.households).toEqual(lived.households);
    expect(entriesOf(town, t)[from.index].road).toBe(road);
    // The houses along the road it left and the one it went to numbered again.
    numberedInTurn(town, road);
    numberedInTurn(town, left);
    expect(moved.name).toMatch(new RegExp(` ${town.roads[road].name}$`));
    expect(homesOn(town, left).length).toBe(homesOn(hamlet, left).length - 1);
    // Not onto another lot: refused at every other house's door, for want of room where nothing else forbids it.
    const onto = hamlet.slots.filter((s) => s !== from && s.kind === 'house').map((s) => lots.moveLot(lived, t, from.index, e[s.index]));
    expect(onto.every((v) => 'refused' in v)).toBe(true);
    expect(onto.some((v) => 'refused' in v && v.refused === 'There is no room for it here.')).toBe(true);
  });

  test("the town's last home and last park are kept", () => {
    // (Its homes: houses, buildings of flats and the places with flats over them.)
    const homes = new Set(hamlet.slots.filter((s) => homesOnLot(s, { farms: true }) > 0).map((s) => s.index));
    expect(lots.removeLotsEdit(hamlet, homes)).toEqual({ refused: 'The town keeps its last home.' });
    const allButOne = new Set([...homes].slice(1));
    const one = made(lots.removeLotsEdit(hamlet, allButOne));
    expect(one.slots.filter((s) => homesOnLot(s, { farms: true }) > 0).length).toBe(1);
    const parks = hamlet.slots.filter((s) => s.kind === 'park' && s.template !== 'market');
    expect(parks.length).toBe(1);
    expect(lots.removeLotsEdit(hamlet, new Set([parks[0].index]))).toEqual({ refused: 'The town keeps its last park.' });
    expect(lots.swapPlace(hamlet, t, content, parks[0].index, 'grocer')).toEqual({ refused: 'The town keeps its last park.' });
    // The market is no park: it may go, and its place with it; the slope limits' removal refuses nothing.
    const market = hamlet.slots.find((s) => s.template === 'market')!;
    const noMarket = lots.removeLotsEdit(hamlet, new Set([market.index]));
    expect(made(noMarket).open!.places!.some((p) => p.name === 'Market Square')).toBe(false);
    expect(noMarket).toHaveProperty('cleared', { lots: [market.index] });
    expect(lots.removeLots(hamlet, homes).slots.some((s) => s.kind === 'house')).toBe(false);
  });

  test('the same seed draws the same house, another seed another', () => {
    const choice: LotChoice = { kind: 'house', size: 'large' };
    const at = fitting(hamlet, choice, 5);
    const [a, b] = [made(lots.addLot(hamlet, t, content, at, choice, 5)), made(lots.addLot(hamlet, t, content, at, choice, 5))];
    expect(JSON.stringify(b)).toBe(JSON.stringify(a));
    const c = lots.addLot(hamlet, t, content, at, choice, 6);
    if (!('refused' in c)) expect(c.draft.slots.at(-1)!.template).not.toBe(a.slots.at(-1)!.template);
  });

  test("a lot's garden, house, public place and a civic place's lot changed", () => {
    // The garden: the same house, its lot cut again with its front where it was.
    const home = hamlet.slots.find((s) => s.kind === 'house' && s.template.startsWith('gen:') && s.lot !== 'farm' && s.lot !== 'terrace' && s.garden !== 'snug' && s.garden !== 'tight')!;
    const snug = made(lots.setGarden(hamlet, t, home.index, 'snug'));
    const cut = snug.slots[home.index];
    expect(cut.garden).toBe('snug');
    expect(cut.template).toBe(lots.withGardenId(home.template, 'snug'));
    expect(cut.anchor).toBe(home.anchor);
    expect(lots.setGarden(hamlet, t, home.index, home.garden!)).toHaveProperty('refused');
    // Another house, where it fits.
    const smaller = generatedId(home.lot!, 123, 'snug');
    const swapped = lots.anotherHouse(hamlet, t, home.index, smaller);
    if (!('refused' in swapped)) expect(swapped.draft.slots[home.index].template).toBe(smaller);
    const farm = hamlet.slots.find((s) => s.lot === 'farm')!;
    expect(lots.anotherHouse(hamlet, t, farm.index, smaller)).toEqual({ refused: 'A farm keeps its farmhouse.' });
    // A public place, among the six of its lot's size.
    expect(lots.publicChoices(t).sort()).toEqual(['bakery', 'cafe', 'grocer', 'market', 'park', 'playground']);
    const market = hamlet.slots.find((s) => s.template === 'market')!;
    const bakery = made(lots.swapPlace(hamlet, t, content, market.index, 'bakery'));
    expect(bakery.slots[market.index]).toMatchObject({ kind: 'shop', template: 'bakery', name: 'The Bakery' });
    expect(bakery.open!.places!.find((p) => p.plot === market.index)).toMatchObject({ name: 'The Bakery', kind: 'shop', work: ['culinary', 'retail'] });
    expect(lots.swapPlace(hamlet, t, content, market.index, 'lookout')).toHaveProperty('refused');
    // A civic place is a lot of its own: the hamlet's hall, its parish hall.
    const places = hamlet.open!.places!;
    const hall = hamlet.slots.find((s) => lots.civicOfLot(s) === 0)!;
    expect(hall).toBeDefined();
    expect(hall.template).toBe('parishHall');
    const hallPlace = places.find((p) => p.plot === hall.index)!;
    expect(hallPlace.name).toBe(`${hamlet.open!.areas[0].name} Parish Hall`);
    expect(hall.name).toBe(hallPlace.name);
    // Not another public place: its lot is built for it.
    expect(lots.swapPlace(hamlet, t, content, hall.index, 'bakery')).toHaveProperty('refused');
    // A hamlet is offered a hamlet's (no school, a village's) until it has a village's homes.
    const school = lots.CIVIC_PLACES.findIndex((c) => c.label === 'School');
    expect(lots.civicSwaps(hamlet, hall.index).some((c) => c.label === 'School')).toBe(false);
    expect(lots.swapCivic(hamlet, t, content, hall.index, school, 1)).toHaveProperty('refused');
    const grown = { ...hamlet, slots: [...hamlet.slots, ...Array.from({ length: 60 }, (_, k) => ({ ...home, index: hamlet.slots.length + k }))] };
    const choices = lots.civicSwaps(grown, hall.index);
    expect(choices.some((c) => c.label === 'School')).toBe(true);
    expect(choices.some((c) => c.label === 'Church')).toBe(false);
    // The school's layout is on a much bigger lot than the hall's: by the square, it has no room there.
    expect(lots.swapCivic(grown, t, content, hall.index, school, 1)).toEqual({ refused: 'There is no room for the school here.' });
    // The inn made studios, a layout on a lot of another size: the inn taken up, the studios a lot of its own where it met its road.
    // (A garage's yard has no room there: its neighbours are too near.)
    const inn = hamlet.slots.find((s) => s.template === 'inn')!;
    const innPlace = places.find((p) => p.plot === inn.index)!;
    const [garage, studios] = ['Garage', 'Studios'].map((label) => lots.CIVIC_PLACES.findIndex((c) => c.label === label));
    expect(lots.civicSwaps(grown, inn.index).some((c) => c.civic === garage)).toBe(true);
    expect(lots.swapCivic(grown, t, content, inn.index, garage, 1)).toEqual({ refused: 'There is no room for the garage here.' });
    expect(lots.civicSwaps(grown, inn.index).some((c) => c.civic === studios)).toBe(true);
    const toStudios = lots.swapCivic(grown, t, content, inn.index, studios, 1);
    const town = made(toStudios);
    expect(toStudios).toHaveProperty('cleared', { lots: [inn.index] });
    expect(town.slots.length).toBe(grown.slots.length);
    expect(town.slots.some((s) => s.template === 'inn')).toBe(false);
    const lot = town.slots.at(-1)!;
    expect(lot).toMatchObject({ kind: 'shop', template: 'studios', area: 0, index: town.slots.length - 1 });
    expect(lot.dress).toBeDefined();
    expect(entriesOf(town, t).at(-1)!.road).toBe(entriesOf(grown, t)[inn.index].road);
    for (const s of town.slots.slice(0, -1)) if (s.kind !== 'house' || s.lot === 'farm') expect(overlap(plotRect(t, s), plotRect(t, lot))).toBe(false);
    const p = town.open!.places!.find((q) => q.plot === lot.index)!;
    expect(p.kind).toBe('workshop');
    expect(p.name).toMatch(/ Studios$/);
    expect(lot.name).toBe(p.name);
    expect(town.open!.places!.filter((q) => q.plot === undefined)).toEqual([]);
    expect(town.open!.places!.some((q) => q.name === innPlace.name)).toBe(false);
    // (The careers the inn took on for the town, if any, go with it.)
    const extra = (innPlace.work ?? []).filter((w) => !lots.CIVIC_PLACES[lots.civicOfLot(inn)].work.includes(w));
    expect(p.work![0]).toBe('arts');
    for (const w of extra) expect(p.work).toContain(w);
    expect(p.work).not.toContain('music');
    const church = lots.CIVIC_PLACES.findIndex((c) => c.label === 'Church');
    expect(lots.swapCivic(hamlet, t, content, hall.index, church)).toHaveProperty('refused');
    expect(lots.swapCivic(hamlet, t, content, market.index, school)).toEqual({ refused: 'That is not a civic place.' });
  });

  test('the check on every pointer move is under 5 ms', () => {
    const choice: LotChoice = { kind: 'house', size: 'small', garden: 'big' };
    const home = hamlet.slots.find((s) => s.kind === 'house' && s.lot === 'medium')!.index;
    const along = spots(hamlet, undefined, 1).filter((_, k) => k % 7 === 0);
    // (The first makes the draft's ground and the house: once per draft and seed.)
    lots.canPlace(hamlet, t, along[0], choice, 9);
    lots.canPlace(hamlet, t, along[0], { slot: home });
    const t0 = performance.now();
    for (const at of along) {
      lots.canPlace(hamlet, t, at, choice, 9);
      lots.canPlace(hamlet, t, at, { slot: home });
    }
    const each = (performance.now() - t0) / (along.length * 2);
    expect(along.length).toBeGreaterThan(100);
    expect(each).toBeLessThan(5);
  });

  test('a town with lots removed, added and moved has no problem the generated town has not', () => {
    let town: NeighbourhoodDraft = hamlet;
    const homes = town.slots.filter((s) => s.kind === 'house' && s.lot !== 'farm');
    town = made(lots.removeLotsEdit(town, new Set([homes[1].index, homes[4].index])));
    const house: LotChoice = { kind: 'house', size: 'small' };
    town = made(lots.addLot(town, t, content, fitting(town, house, 21), house, 21));
    const cafe: LotChoice = { kind: 'shop', template: 'cafe' };
    town = made(lots.addLot(town, t, content, fitting(town, cafe), cafe, 22));
    const spot: LotChoice = { kind: 'outdoor', template: 'campfire' };
    town = made(lots.addLot(town, t, content, fitting(town, spot), spot, 23));
    const mover = town.slots.find((s) => s.kind === 'house' && s.lot === 'medium')!.index;
    town = made(lots.moveLot(town, t, mover, fitting(town, { slot: mover })));
    const outdoor = town.slots.find((s) => s.kind === 'outdoor')!.index;
    town = made(lots.moveLot(town, t, outdoor, fitting(town, { slot: outdoor })));
    const country = town.roads.findIndex((r) => r.kind === 'country' && !r.bridge && !('refused' in lots.fillStreet(town, t, content, town.roads.indexOf(r), 24)));
    town = made(lots.fillStreet(town, t, content, country, 24));
    const market = town.slots.find((s) => s.template === 'market')!.index;
    town = made(lots.swapPlace(town, t, content, market, 'grocer'));
    expect(town.slots.length).toBeGreaterThan(hamlet.slots.length);

    const careers = content.careerCategories.map((c) => c.id);
    const problems = (draft: NeighbourhoodDraft) => {
      const home = vacantSlots(draft)[0].index;
      const file = JSON.parse(assembleTown(content, t, draft, randomHousehold(content, assets, 2, [], {}, mulberry32(1)), home, undefined, mulberry32(2)));
      const found = townProblems(file, undefined, { careers });
      // (Lots and places are named by index, which edits move: the kinds of problem are compared.)
      return new Set([...found.hard, ...found.soft].map((p) => p.replace(/[\d.]+/g, '#')));
    };
    const before = problems(hamlet);
    const after = problems(town);
    expect([...after].filter((p) => !before.has(p))).toEqual([]);
  });

});
