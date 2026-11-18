// Step 4's Move in (`ui/newGame.ts`): the request a chosen home makes, for a household made in
// Creation and for one of the town's own (staying in its home, or moving to one for sale, its old
// home then for sale too), and the town the game starts in: every home still for sale lived in but
// for the few cheapest kept (`people.ts` `keepForSale`), no more neighbours than the cap, the same
// people from the same people seed, and `Math.random` never called when the request has one.
import { afterAll, beforeAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('moving in', async () => {
  // --- the page's world in bun: content from web/public (put back for whatever runs next) ---
  const realLocation = globalThis.location;
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url, 'http://localhost/');
    const path = `${WEB}public${decodeURIComponent(url.pathname)}`;
    return existsSync(path) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
  }) as typeof fetch;
  const realRandom = Math.random;
  afterAll(() => {
    globalThis.fetch = realFetch;
    Object.assign(globalThis, { location: realLocation });
    Math.random = realRandom;
  });
  const { Content } = await import('../src/content/content');
  const { AssetRegistry } = await import('../src/assets/registry');
  const { objectInfos } = await import('../src/game/housegen');
  const { randomHousehold } = await import('../src/game/household');
  const { EMPTY_LOT, generateWorld, useTownContent, vacantSlots } = await import('../src/game/town');
  const { keepForSale, peopleStreams, populate, FILL_CAP } = await import('../src/game/people');
  const { mulberry32 } = await import('../src/game/noise');
  const { householdsAhead, moveInRequest, peopleSeedOf, startCode, townOnMoveIn } = await import('../src/ui/newGame');
  const { residentsOf } = await import('../src/game/household');
  type Templates = import('../src/game/town').Templates;
  type NeighbourhoodDraft = import('../src/game/town').NeighbourhoodDraft;
  type PlotSlot = import('../src/game/town').PlotSlot;
  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  /** A main town of `size` shipped houses on a grid and a village of twelve 400 m off (nothing the generator makes). */
  const gridTown = (size: number): NeighbourhoodDraft => {
    const kinds = [
      ['maple', 'small'],
      ['willow', 'medium'],
      ['oakridge', 'large'],
    ] as const;
    const slots: PlotSlot[] = [];
    const add = (x: number, z: number, area: number) => {
      const [template, lot] = kinds[slots.length % kinds.length];
      slots.push({ index: slots.length, kind: 'house', template, lot, name: `${slots.length + 1} Grid Road`, x, z, facing: 0, area });
    };
    for (let i = 0; i < size; i++) add(40 + (i % 10) * 30, 40 + Math.floor(i / 10) * 40, 0);
    for (let i = 0; i < 12; i++) add(640 + (i % 4) * 30, 80 + Math.floor(i / 4) * 40, 1);
    const areas = [
      { name: 'Grid', kind: 'hamlet' as const, x: 0, z: 0, w: 320, d: 600 },
      { name: 'Outlying', kind: 'hamlet' as const, x: 600, z: 40, w: 160, d: 160 },
    ];
    return { name: 'Grid', width: 800, depth: 640, slots, roads: [], greens: [], households: [], open: { areas } as NeighbourhoodDraft['open'] };
  };
  const hamlet = generateWorld(content, t, 'country', 4242, undefined, undefined, { town: 'hamlet', villages: 'none' });
  const homes = (town: NeighbourhoodDraft) => town.slots.filter((s) => s.kind === 'house').length;
  const residents = (town: NeighbourhoodDraft, but?: number) => town.households.filter((h) => h.slot !== but).reduce((n, h) => n + h.household.members.length, 0);
  const player = () => randomHousehold(content, assets, 2, [], {}, mulberry32(1));
  const SEED = 2026;
  const firstWave = (town: NeighbourhoodDraft) => populate(content, assets, t, town, undefined, peopleStreams(SEED));
  const common = { emptyLot: false, mode: 'living' as const, lifespan: 'normal' as const, peopleSeed: SEED };

  // From here on nothing may roll Creation's dice.
  beforeAll(() => {
    Math.random = () => {
      throw new Error('Math.random was called');
    };
  });

  test("the people seed is the code's own, else drawn from its town's seed (or its land's)", () => {
    const code = startCode('river', 1234);
    expect(peopleSeedOf({ ...code, peopleSeed: 5 })).toBe(5);
    expect(peopleSeedOf({ ...code, townSeed: 77 })).toBe(peopleSeedOf({ ...code, townSeed: 77 }));
    expect(peopleSeedOf({ ...code, townSeed: 77 })).not.toBe(peopleSeedOf(code));
  });

  test('a household made in Creation moves into the home chosen, as it stands or on an empty lot', () => {
    const town = firstWave(hamlet);
    const home = vacantSlots(town)[0].index;
    const household = player();
    const request = moveInRequest({ town, slot: home, household, from: null, ...common });
    expect(request).toMatchObject({ kind: 'new', slot: home, household, fill: true, peopleSeed: SEED });
    expect(request.existing).toBeUndefined();
    expect(request.town).toBe(town);
    const empty = moveInRequest({ town, slot: home, household, from: null, ...common, emptyLot: true });
    expect(empty.town.slots[home].template).toBe(EMPTY_LOT);
  });

  test("one of the town's households stays in its home (taken over), or moves and leaves its home for sale", () => {
    const town = firstWave(hamlet);
    const own = town.households[0];
    const stay = moveInRequest({ town, slot: own.slot, household: own.household, from: own.slot, ...common, emptyLot: true });
    expect(stay).toMatchObject({ kind: 'new', slot: own.slot, existing: true, household: own.household, fill: true });
    expect(stay.town).toBe(town);
    const elsewhere = vacantSlots(town)[0].index;
    const moved = moveInRequest({ town, slot: elsewhere, household: own.household, from: own.slot, ...common });
    expect(moved.existing).toBeUndefined();
    expect(moved.town.households.some((h) => h.slot === own.slot)).toBe(false);
    expect(moved.town.households.length).toBe(town.households.length - 1);
    expect(vacantSlots(moved.town).map((s) => s.index)).toContain(own.slot);
    // Their old home is filled with the rest (it isn't among the few cheapest kept here, or it is for sale).
    const filled = townOnMoveIn(content, assets, t, moved);
    expect(filled.households.some((h) => h.household === own.household)).toBe(false);
    expect(filled.households.every((h) => h.household.name !== own.household.name)).toBe(true);
  });

  test('Move in fills every home but the few kept for sale, the same people twice', () => {
    const town = firstWave(hamlet);
    const home = vacantSlots(town)[0].index;
    const request = moveInRequest({ town, slot: home, household: player(), from: null, ...common });
    const filled = townOnMoveIn(content, assets, t, request);
    // (The player's home is not among the town's households: the game moves them in.)
    expect(vacantSlots(filled).filter((s) => s.index !== home).length).toBe(keepForSale(homes(hamlet)));
    expect(filled.households.slice(0, town.households.length)).toEqual(town.households);
    // The newcomers take no household's family name as the player's (the first wave was drawn before the player was made).
    expect(filled.households.slice(town.households.length).every((h) => h.household.name !== request.household.name)).toBe(true);
    expect(JSON.stringify(townOnMoveIn(content, assets, t, request).households)).toBe(JSON.stringify(filled.households));
    // Another people seed: other newcomers.
    const other = townOnMoveIn(content, assets, t, { ...request, peopleSeed: SEED + 1 });
    expect(JSON.stringify(other.households)).not.toBe(JSON.stringify(filled.households));
    // A household taking over its own home: the town is filled round it too.
    const own = town.households[0];
    const stayed = townOnMoveIn(content, assets, t, moveInRequest({ town, slot: own.slot, household: own.household, from: own.slot, ...common }));
    expect(vacantSlots(stayed).length).toBe(keepForSale(homes(hamlet)));
  });

  test('a big town is filled up to the cap and no further, every home but the kept lived in', () => {
    const big = gridTown(480);
    const town = firstWave(big);
    const home = vacantSlots(town).find((s) => (s.area ?? 0) === 0)!.index;
    const request = moveInRequest({ town, slot: home, household: player(), from: null, ...common });
    const filled = townOnMoveIn(content, assets, t, request);
    expect(residents(filled)).toBeLessThanOrEqual(FILL_CAP);
    expect(residents(filled)).toBeGreaterThan(FILL_CAP - 4);
    expect(vacantSlots(filled).filter((s) => s.index !== home).length).toBe(keepForSale(homes(big)));
    // The village has its households.
    expect(filled.households.some((h) => (filled.slots[h.slot].area ?? 0) === 1)).toBe(true);
    // A smaller tier: fewer homes.
    expect(townOnMoveIn(content, assets, t, request, Math.random, 'near').households.length).toBeLessThan(filled.households.length);
  });

  test('the houses furnished ahead in step 4 are for the households Move in moves into them', () => {
    const town = firstWave(gridTown(180));
    const household = player();
    const ahead = new Map(householdsAhead(content, assets, t, town, SEED, household).map((h) => [h.slot, h.household]));
    const home = vacantSlots(town).find((s) => (s.area ?? 0) === 0)!.index;
    const filled = townOnMoveIn(content, assets, t, moveInRequest({ town, slot: home, household, from: null, ...common }));
    // (A house is furnished for who lives in it: the same residents, the same furniture.)
    const same = filled.households.filter((h) => ahead.has(h.slot) && JSON.stringify(residentsOf(content, ahead.get(h.slot)!)) === JSON.stringify(residentsOf(content, h.household)));
    expect(same.length).toBeGreaterThan(filled.households.length * 0.95);
  });

  test('a request without the fill (or with no people seed: then a seed of its own) starts as it is', () => {
    const town = firstWave(hamlet);
    const home = vacantSlots(town)[0].index;
    const request = moveInRequest({ town, slot: home, household: player(), from: null, ...common });
    expect(townOnMoveIn(content, assets, t, { ...request, fill: undefined })).toBe(town);
    const { peopleSeed: _, ...unseeded } = request;
    const filled = townOnMoveIn(content, assets, t, unseeded, mulberry32(9));
    expect(vacantSlots(filled).filter((s) => s.index !== home).length).toBe(keepForSale(homes(hamlet)));
  });
});
