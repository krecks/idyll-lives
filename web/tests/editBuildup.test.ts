// Step 2's build-up (`game/edit/buildup.ts`) on a town laid as step 2 lays one (the `land` and `town`
// jobs: its plan, the land as laid): a street built up the same way twice for the same seed; nothing
// built off the street's frontage and nothing standing moved, every new building a real lot (a named
// place called as one and given its place, a building of flats called as its template names it); a
// street through the centre gets no homes of the lot mix there but the centre's buildings, flats over
// its shops among them, a village's street a village's kinds; the homes the same whichever mode; undo
// gives the draft's bytes back; a civic place offered until the town has it, put down a lot of its own,
// and a civic place's lot made another; a street built up until it is full, then refused with why; its
// buildings cleared with their flats and places; on hilly land the settle takes every lot built (those
// that would break a slope limit left out first); a street drawn with the town's buildings along it in
// one edit; and a town built up so has no hard problem it hadn't (`edit/checked.ts` `editedProblems`).
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('build-up', async () => {
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
  const { objectInfos } = await import('../src/game/housegen');
  const { plotRect, useTownContent } = await import('../src/game/town');
  const { runTownJob } = await import('../src/game/townJobs');
  const { inside, lengths, RoadIndex } = await import('../src/game/roads');
  const { CIVIC, civicChoices, civicOf, draftAreaSize } = await import('../src/game/places');
  const { sizeByHomes } = await import('../src/game/world');
  const { civicOfLot, removeLots, swapCivic } = await import('../src/game/edit/lots');
  const { entriesOf, frontOf } = await import('../src/game/edit/ground');
  const { TownHistory } = await import('../src/game/edit/history');
  const { editedProblems } = await import('../src/game/edit/checked');
  const { addCivic, areaOfRoad, buildingsOn, builtAlong, buildUp, clearBuildings, civicOffered, isTownBuilding } = await import('../src/game/edit/buildup');
  const { homesOn } = await import('../src/game/town');
  const { flatsTemplate } = await import('../src/game/flats');
  const { drawStreet, removeRoad } = await import('../src/game/edit/streets');
  type Templates = import('../src/game/town').Templates;
  type NeighbourhoodDraft = import('../src/game/town').NeighbourhoodDraft;
  type TownJob = import('../src/game/townJobs').TownJob;
  type TownResult = import('../src/game/townJobs').TownResult;
  type WorldOptions = import('../src/game/world').WorldOptions;
  type Edited = import('../src/game/edit/edited').Edited;
  const content = await Content.load('http://localhost/content/base.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  const inWorker = <K extends TownJob['kind']>(job: Extract<TownJob, { kind: K }>) => structuredClone(runTownJob(content, t, structuredClone(job))) as Extract<TownResult, { kind: K }>;
  /** A town's plan as step 2 edits it (the land as laid under it). */
  const planOf = (world: string, seed: number, options: Partial<WorldOptions>): NeighbourhoodDraft => {
    const land = inWorker({ kind: 'land', world, seed, options });
    const town = inWorker({ kind: 'town', land: land.draft, seed, options, waterKept: true });
    if (!town.plan) throw new Error('the town has no plan');
    return town.plan;
  };
  const bytes = (v: unknown) => JSON.stringify(v, (_, x) => (ArrayBuffer.isView(x) ? Array.from(x as Int16Array).join(',') : x));
  const made = (e: Edited): NeighbourhoodDraft => {
    if ('refused' in e) throw new Error(`refused: ${e.refused}`);
    return e.draft;
  };
  /** `draft` with road `road` stripped bare: every lot fronting it gone. */
  const bare = (draft: NeighbourhoodDraft, road: number): NeighbourhoodDraft => {
    const e = entriesOf(draft, t);
    return removeLots(draft, new Set(draft.slots.filter((_, i) => e[i].road === road).map((s) => s.index)));
  };
  /** The roads of area `area` (streets and lanes) by length, longest first. */
  const streetsOf = (draft: NeighbourhoodDraft, area: number) =>
    draft.roads
      .map((r, i) => ({ r, i, L: lengths(r.points).at(-1)! }))
      .filter(({ r }) => r.area === area && (r.kind === 'street' || r.kind === 'lane') && !r.bridge)
      .sort((a, b) => b.L - a.L)
      .map(({ i }) => i);

  const town = planOf('country', 11, { town: 'town', villages: 'few', size: 'medium' });
  const centre = town.open!.centre![0].poly;
  /** A main-town street through none of the centre, stripped bare. */
  const plainRoad = streetsOf(town, 0).find((i) => !town.roads[i].points.some(([x, z]) => inside(centre, x, z)))!;
  const plain = bare(town, plainRoad);
  /** The street the centre is on, stripped bare. */
  const highRoad = streetsOf(town, 0).sort((a, b) => town.roads[b].points.filter(([x, z]) => inside(centre, x, z)).length - town.roads[a].points.filter(([x, z]) => inside(centre, x, z)).length)[0];
  const high = bare(town, highRoad);

  test('the same build-up twice gives the same buildings; the draft it was made on is untouched', () => {
    const before = bytes(plain);
    const a = buildUp(plain, t, content, { road: plainRoad, mode: 'both', seed: 7 });
    const b = buildUp(plain, t, content, { road: plainRoad, mode: 'both', seed: 7 });
    expect(bytes(made(a))).toBe(bytes(made(b)));
    expect(bytes(plain)).toBe(before);
    // Another seed, another street's worth.
    const c = made(buildUp(plain, t, content, { road: plainRoad, mode: 'both', seed: 8 }));
    expect(bytes(c.slots)).not.toBe(bytes(made(a).slots));
  });

  test('nothing is built off the street, nothing standing moves, and every building is a real lot', () => {
    const e = buildUp(plain, t, content, { road: plainRoad, mode: 'both', seed: 7 });
    const d = made(e);
    const fresh = d.slots.slice(plain.slots.length);
    const buildings = fresh.filter(isTownBuilding);
    expect(buildings.length).toBeGreaterThan(1);
    expect(d.slots.slice(0, plain.slots.length).map((s) => [s.x, s.z, s.facing, s.template])).toEqual(plain.slots.map((s) => [s.x, s.z, s.facing, s.template]));
    expect(d.open!.places!.slice(0, plain.open!.places!.length)).toEqual(plain.open!.places!);
    // Every new lot meets the street, and the town's buildings fronting it are the new ones.
    const entries = entriesOf(d, t);
    for (let i = plain.slots.length; i < d.slots.length; i++) expect(entries[i].road).toBe(plainRoad);
    expect(buildingsOn(d, t, plainRoad)).toEqual(buildings.map((s) => s.index));
    // Every new public lot has its place, called as it is, none twice; a building of flats is called as its template names it.
    const places = d.open!.places!;
    expect(places.length - plain.open!.places!.length).toBe(fresh.filter((s) => s.kind === 'shop').length);
    for (const s of fresh.filter((q) => q.kind === 'shop')) expect(places.find((p) => p.plot === s.index)?.name).toBe(s.name);
    expect(new Set(places.map((p) => p.name)).size).toBe(places.length);
    for (const s of fresh.filter((q) => q.kind === 'flats')) expect(s.name).toBe(flatsTemplate(t, s.template)!.name);
    const homes = fresh.reduce((n, s) => n + homesOn(s, { farms: true }), 0);
    console.log(`plain street: ${fresh.length} lots (${homes} homes: ${fresh.filter((s) => s.kind === 'flats').length} buildings of flats, ${fresh.filter((s) => s.kind === 'shop' && s.upper?.startsWith('over:')).length} places with flats over them), ${buildings.length} of them the town's buildings, ${places.length - plain.open!.places!.length} places, left out ${'leftOut' in e ? e.leftOut : 0}`);
  });

  test("a street through the centre gets no homes of the lot mix there, but the centre's buildings, flats over its shops among them", () => {
    const d = made(buildUp(high, t, content, { road: highRoad, mode: 'both', seed: 3 }));
    const fresh = d.slots.slice(high.slots.length);
    // (The lot mix's homes outside it; in it, the centre's kinds: shops, cafés, restaurants, offices, flats, terraces.)
    const inCentre = fresh.filter((s) => inside(centre, ...frontOf(t, s)));
    expect(inCentre.length).toBeGreaterThan(3);
    for (const s of inCentre) expect(isTownBuilding(s)).toBe(true);
    expect(inCentre.some((s) => s.kind === 'house' && s.garden === 'tight')).toBe(false);
    const uses = inCentre.map((s) => s.dress?.use ?? (s.lot === 'terrace' ? 'terrace' : s.kind));
    for (const u of uses) expect(['shop', 'flats', 'cafe', 'restaurant', 'office', 'terrace']).toContain(u);
    // (Most of its shops, cafés and restaurants with flats over them: `dense.ts` `raisePlaces`.)
    const over = inCentre.filter((s) => s.upper?.startsWith('over:')).length;
    expect(over + inCentre.filter((s) => s.kind === 'flats').length).toBeGreaterThan(0);
    console.log(`high street: ${inCentre.length} lots in the centre (${[...new Set(uses)].join(', ')}; ${over} with flats over them), ${fresh.length - inCentre.length} lots outside it`);
  });

  test("a village's street gets a village's kinds and homes", () => {
    const area = town.open!.areas.findIndex((_, a) => a > 0 && streetsOf(town, a).length > 0);
    expect(area).toBeGreaterThan(0);
    const road = streetsOf(town, area)[0];
    expect(areaOfRoad(town, road)).toBe(area);
    const v = bare(town, road);
    const d = made(buildUp(v, t, content, { road, mode: 'both', seed: 5 }));
    const fresh = d.slots.slice(v.slots.length);
    expect(fresh.length).toBeGreaterThan(0);
    // (A hamlet's kinds: homes, terraces, workshops, a shop or a café; no flats, offices or restaurants.)
    const uses = fresh.map((s) => s.dress?.use ?? (s.lot === 'terrace' ? 'terrace' : s.kind));
    for (const [k, s] of fresh.entries()) {
      expect(['house', 'terrace', 'workshop', 'shop', 'cafe']).toContain(uses[k]);
      expect(s.area ?? 0).toBe(area);
    }
    console.log(`village street: ${fresh.length} lots (${[...new Set(uses)].join(', ')})`);
  });

  test('a country road out of every town and village is not built up, and says why', () => {
    const road = town.roads.findIndex((_, i) => town.roads[i].kind === 'country' && !town.roads[i].bridge && areaOfRoad(town, i) < 0);
    expect(road).toBeGreaterThanOrEqual(0);
    const e = buildUp(town, t, content, { road, mode: 'both', seed: 1 });
    expect('refused' in e && e.refused).toMatch(/country road out of the town and its villages/);
  });

  test('the homes stay the same whichever mode', () => {
    // (The lot mix's homes: those that are none of the town's buildings.)
    const sig = (d: NeighbourhoodDraft) => d.slots.slice(plain.slots.length).filter((s) => !isTownBuilding(s)).map((s) => JSON.stringify(plotRect(t, s)) + s.template);
    const homes = sig(made(buildUp(plain, t, content, { road: plainRoad, mode: 'homes', seed: 9 })));
    const both = sig(made(buildUp(plain, t, content, { road: plainRoad, mode: 'both', seed: 9 })));
    const buildings = made(buildUp(plain, t, content, { road: plainRoad, mode: 'buildings', seed: 9 }));
    expect(both.length).toBeGreaterThan(0);
    // (Both: the main town's homes on a share of the frontage, the first the lot fill puts down.)
    expect(both.length).toBeLessThan(homes.length);
    expect(homes.slice(0, both.length)).toEqual(both);
    // Buildings alone: none of the lot mix's homes, only the town's buildings.
    const fresh = buildings.slots.slice(plain.slots.length);
    expect(fresh.length).toBeGreaterThan(0);
    expect(fresh.every(isTownBuilding)).toBe(true);
  });

  test("undo gives the draft's bytes back", () => {
    const history = new TownHistory(plain);
    const before = bytes(history.draft);
    history.push(made(buildUp(plain, t, content, { road: plainRoad, mode: 'both', seed: 2 })));
    expect(bytes(history.draft)).not.toBe(before);
    expect(bytes(history.undo())).toBe(before);
  });

  test('a street is built up until it is full, then refused with why; its buildings cleared', () => {
    let d = plain;
    let refused = '';
    for (let k = 0; k < 12 && !refused; k++) {
      const e = buildUp(d, t, content, { road: plainRoad, mode: 'both', seed: 100 + k });
      if ('refused' in e) refused = e.refused;
      else d = e.draft;
    }
    expect(refused).toBe(`nothing more fits along ${plain.roads[plainRoad].name}`);
    const on = buildingsOn(d, t, plainRoad);
    expect(on.length).toBeGreaterThan(0);
    const cleared = clearBuildings(d, t, plainRoad);
    const c = made(cleared);
    expect('cleared' in cleared && cleared.cleared.lots).toEqual(on);
    expect(buildingsOn(c, t, plainRoad)).toEqual([]);
    expect(c.slots.length).toBe(d.slots.length - on.length);
    // The places of those that were places go with them; the homes of the lot mix stay.
    const gone = new Set(on);
    expect(c.open!.places!.length).toBe(d.open!.places!.filter((p) => p.plot === undefined || !gone.has(p.plot)).length);
    expect(c.slots.map((s) => [s.x, s.z, s.template])).toEqual(d.slots.filter((s) => !gone.has(s.index)).map((s) => [s.x, s.z, s.template]));
    expect('refused' in clearBuildings(c, t, plainRoad) && (clearBuildings(c, t, plainRoad) as { refused: string }).refused).toMatch(/nothing is built along/);
  });

  /** The lot of civic place `civic` (an index into `CIVIC`) in area `area` of `draft`, by slot index; -1 if none. */
  const civicLot = (draft: NeighbourhoodDraft, area: number, civic: number) => draft.slots.findIndex((s) => (s.area ?? 0) === area && civicOfLot(s) === civic);

  test('a civic place is offered until the town has it, then refused; put down by hand, it is a lot of its own', () => {
    // The town's civic places are lots; its church taken away, offered, put back by hand.
    const church = civicLot(town, 0, 1);
    expect(church).toBeGreaterThanOrEqual(0);
    expect(civicOf(town.open!.places!.find((p) => p.plot === church))).toBe(1);
    const without = removeLots(town, new Set([church]));
    expect(civicChoices(town, 0).some((c) => c.index === 1)).toBe(false);
    expect(civicOffered(without, 0).filter((c) => c.civic === 1).length).toBe(1);
    // About where it stood (the town is built up close: elsewhere it may not fit), the spot found to the metre
    // (the Lot tool's fit, not the generator's, may want it a few metres along: 7 m with 6 m lanes).
    const door = town.open!.places!.find((p) => p.plot === church)!;
    const near = new RoadIndex(without.roads).nearest(door.x, door.z, 60)!;
    let put: NeighbourhoodDraft | null = null;
    for (let s = Math.floor(near.s) - 10; s <= near.s + 10 && !put; s++) {
      for (const side of [1, -1] as const) {
        const e = addCivic(without, t, content, { civic: 1, road: near.road, s, side, seed: 4 });
        if ('draft' in e) {
          put = e.draft;
          // The same op asked again: the same edit (the Lot tool asks on every pointer move).
          expect(addCivic(without, t, content, { civic: 1, road: near.road, s, side, seed: 4 })).toBe(e);
          break;
        }
      }
    }
    expect(put).not.toBeNull();
    const d = put!;
    expect(d.slots.length).toBe(without.slots.length + 1);
    const lot = d.slots.at(-1)!;
    expect(lot).toMatchObject({ kind: 'shop', template: 'church', area: 0, index: d.slots.length - 1 });
    expect(lot.dress).toMatchObject({ use: 'church' });
    // (Its lot the layout's own, not the frame the sim's shops share.)
    const p = t.parks.find((q) => q.id === 'church')!;
    expect([lot.w, lot.d]).toEqual([p.lot!.width, p.lot!.depth]);
    const place = d.open!.places!.find((q) => q.plot === lot.index)!;
    expect(civicOf(place)).toBe(1);
    expect(place).toMatchObject({ kind: 'church', area: 0, name: lot.name });
    expect(place.name).toMatch(/^St .+'s Church$/);
    expect(place.hours).toBeDefined();
    expect(civicOffered(d, 0).some((c) => c.civic === 1)).toBe(false);
    const again = addCivic(d, t, content, { civic: 1, road: streetsOf(d, 0)[0], s: 40, side: 1, seed: 1 });
    expect('refused' in again && again.refused).toMatch(/has a church already/);
  });

  test("a village is offered a village's civic places, by a village's names: its hall a village hall, no police station", () => {
    const village = town.open!.areas.findIndex((_, a) => a > 0 && streetsOf(town, a).length > 0 && civicLot(town, a, 0) >= 0);
    expect(village).toBeGreaterThan(0);
    // Its own hall, a village hall on the parish hall's layout; taken away, offered again.
    const name = town.open!.areas[village].name;
    const hall = civicLot(town, village, 0);
    expect(town.slots[hall].template).toBe('parishHall');
    expect(town.open!.places!.find((p) => p.plot === hall)!.name).toBe(`${name} Village Hall`);
    const noHall = removeLots(town, new Set([hall]));
    expect(civicOffered(town, village).some((c) => c.civic === 0)).toBe(false);
    const offered = civicOffered(noHall, village);
    expect(offered.find((c) => c.civic === 0)?.label).toBe('Village hall');
    expect(offered.some((c) => /Police|Fire|Bank|Hotel/.test(c.label))).toBe(false);
    // (The main town is offered as before: a town's hall is its town hall.)
    expect(civicChoices(town, 0).every((c) => c.index !== 0 || c.name === 'Town hall')).toBe(true);
    // Put down on one of its streets, it is named as the village's hall.
    let put: NeighbourhoodDraft | null = null;
    for (const road of streetsOf(noHall, village)) {
      const L = lengths(noHall.roads[road].points).at(-1)!;
      for (let s = 8; s < L - 8 && !put; s += 4) {
        for (const side of [1, -1] as const) {
          const e = addCivic(bare(noHall, road), t, content, { civic: 0, road, s, side, seed: 2 });
          if ('draft' in e) {
            put = e.draft;
            break;
          }
        }
      }
      if (put) break;
    }
    expect(put).not.toBeNull();
    const lot = put!.slots.at(-1)!;
    expect(lot).toMatchObject({ template: 'parishHall', area: village, name: `${name} Village Hall` });
    expect(put!.open!.places!.find((p) => p.plot === lot.index)!.name).toBe(`${name} Village Hall`);
    // One a village is too small for is refused, and says so.
    const police = addCivic(town, t, content, { civic: 3, road: streetsOf(town, village)[0], s: 20, side: 1, seed: 1 });
    expect('refused' in police && police.refused).toMatch(/too small for a police station/);
  });

  test("the main town is offered its size's civic places, the bigger of its place's and its homes', those for careers nobody there employs first", () => {
    // A main town's size by its homes (as the generator lays each size's).
    expect([20, 70, 90, 110, 150, 190, 240].map(sizeByHomes)).toEqual(['hamlet', 'hamlet', 'village', 'village', 'town', 'town', 'city']);
    // A hamlet is offered no police station, nor a village's school, until it has grown.
    const hamlet = planOf('country', 11, { town: 'hamlet', villages: 'none', size: 'small' });
    const homes = hamlet.slots.filter((s) => s.kind === 'house' && (s.area ?? 0) === 0);
    expect(draftAreaSize(hamlet, 0)).toBe('hamlet');
    expect(civicOffered(hamlet, 0).some((c) => /Police|Fire|Bank|Hotel|School/.test(c.label))).toBe(false);
    const road = streetsOf(hamlet, 0)[0];
    const police = addCivic(hamlet, t, content, { civic: 3, road, s: 20, side: 1, seed: 1 });
    expect('refused' in police && police.refused).toMatch(/too small for a police station/);
    const grown = { ...hamlet, slots: [...hamlet.slots, ...Array.from({ length: 140 - homes.length }, (_, k) => ({ ...homes[0], index: hamlet.slots.length + k }))] };
    expect(draftAreaSize(grown, 0)).toBe('town');
    expect(civicOffered(grown, 0).some((c) => c.label === 'Police station')).toBe(true);
    // The town without its church and its police station: the police station (a detective's work, nobody else's)
    // offered before the church (nobody's work).
    const without = removeLots(town, new Set([civicLot(town, 0, 1), civicLot(town, 0, 3)]));
    const order = civicOffered(without, 0).map((c) => c.civic);
    expect(order).toContain(1);
    expect(order).toContain(3);
    expect(order.indexOf(3)).toBeLessThan(order.indexOf(1));
  });

  test("a civic place's lot made another whose layout is on a lot of its size: the same lot, built from the other layout", () => {
    const [police, bank] = ['Police station', 'Bank'].map((label) => CIVIC.findIndex((c) => c.label === label));
    expect(t.parks.find((p) => p.id === 'police')!.lot).toEqual(t.parks.find((p) => p.id === 'bank')!.lot);
    expect(civicLot(town, 0, police)).toBeGreaterThanOrEqual(0);
    // The town has a bank: refused, and said why.
    const had = swapCivic(town, t, content, civicLot(town, 0, police), bank, 1);
    expect('refused' in had && had.refused).toMatch(/has a bank already/);
    const noBank = removeLots(town, new Set([civicLot(town, 0, bank)]));
    const at = civicLot(noBank, 0, police);
    const e = swapCivic(noBank, t, content, at, bank, 1);
    const d = made(e);
    expect(e).toHaveProperty('cleared', { lots: [] });
    const [was, now] = [noBank.slots[at], d.slots[at]];
    expect([now.x, now.z, now.facing, now.w, now.d, now.anchor]).toEqual([was.x, was.z, was.facing, was.w, was.d, was.anchor]);
    expect(now.template).toBe('bank');
    expect(d.slots.length).toBe(noBank.slots.length);
    const place = d.open!.places!.find((p) => p.plot === at)!;
    expect(civicOf(place)).toBe(bank);
    expect(place.name).toBe(now.name);
    expect(place.name).toMatch(/ Bank$/);
    expect(place.work!.slice(0, 2)).toEqual(['finance', 'business']);
    expect(d.open!.places!.length).toBe(noBank.open!.places!.length);
    expect(civicOffered(d, 0).some((c) => c.civic === police)).toBe(true);
  });

  test('on hilly land the settle takes every lot a street is built up with: those that would break a slope limit are left out first', () => {
    const settle = (steps: readonly NeighbourhoodDraft[]) => inWorker({ kind: 'edit', plan: steps[steps.length - 1], ops: steps.length - 1, steps, check: false });
    const hilly = planOf('heath', 5, { town: 'town', villages: 'few', size: 'medium', hills: 'hilly' });
    let [built, left] = [0, 0];
    for (const road of streetsOf(hilly, 0).slice(0, 3)) {
      const b = bare(hilly, road);
      const e = buildUp(b, t, content, { road, mode: 'both', seed: 1 });
      if ('refused' in e) continue;
      built += e.draft.slots.length - b.slots.length;
      left += e.leftOut ?? 0;
      expect(settle([b, e.draft]).refused).toEqual([]);
    }
    expect(built).toBeGreaterThan(10);
    console.log(`hilly: ${built} lots, ${left} left out`);
  }, 120_000);

  test('a street drawn with the town buildings along it is one edit: the street, its homes and its buildings', () => {
    // A street of the main town joined to a road at both ends taken up, and drawn again as the Street tool draws one
    // (a dead end reaches 45 m at most: its homes, a share of the frontage in the main town, would take all of it).
    let found: { gone: NeighbourhoodDraft; points: [number, number][]; drawn: Edited } | null = null;
    for (const road of streetsOf(town, 0)) {
      const r = town.roads[road];
      if (r.ends[0] !== 'join' || r.ends[1] !== 'join') continue;
      const e = removeRoad(town, t, road);
      if (!('draft' in e)) continue;
      // (Its line again, or its ends and middle: drawn over, its curve is made again.)
      const pts = r.points.map(([x, z]): [number, number] => [x, z]);
      for (const points of [pts, [pts[0], pts[Math.floor(pts.length / 2)], pts[pts.length - 1]]]) {
        const drawn = drawStreet(e.draft, t, content, { points, kind: 'street', clear: true, fill: false, seed: 6 });
        if ('draft' in drawn) found = { gone: e.draft, points, drawn };
        if (found) break;
      }
      if (found) break;
    }
    expect(found).not.toBeNull();
    const { gone, points, drawn } = found!;
    const d = made(builtAlong(gone, drawn, t, content, 'both', 6));
    const road = d.roads.findIndex((r) => !gone.roads.includes(r));
    expect(road).toBeGreaterThanOrEqual(0);
    expect(areaOfRoad(d, road)).toBe(0);
    expect(d.slots.length).toBeGreaterThan(gone.slots.length);
    expect(buildingsOn(d, t, road).length).toBeGreaterThan(0);
    expect(made(builtAlong(gone, drawn, t, content, 'both', 6))).toEqual(d);
    // Homes alone: `drawStreet`'s own fill, none of the town's buildings.
    const homes = made(drawStreet(gone, t, content, { points, kind: 'street', clear: true, fill: true, seed: 6 }));
    expect(homes.slots.slice(gone.slots.length).some(isTownBuilding)).toBe(false);
  });

  test('a town built up has no hard problem it had not', () => {
    const was = new Set(editedProblems(town, t, content).hard);
    let d = plain;
    d = made(buildUp(d, t, content, { road: plainRoad, mode: 'both', seed: 7 }));
    const highUp = buildUp(bare(d, highRoad), t, content, { road: highRoad, mode: 'both', seed: 7 });
    d = made(highUp);
    const fresh = editedProblems(d, t, content).hard.filter((p) => !was.has(p));
    expect(fresh).toEqual([]);
  });
});
