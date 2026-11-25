// Step 4's Move in (`ui/newGame.ts`): the request a chosen home makes (a house's lot, or a flat), for a
// household made in Creation and for one of the town's own (staying in its home, or moving to one for
// sale, its old home then for sale too), and the town the game starts in: every home still for sale
// lived in but for the few cheapest kept (`people.ts` `keepForSale`), no more neighbours than the cap,
// the same people from the same people seed, and `Math.random` never called when the request has one;
// and the town file of a household in a flat (`assembleTown`), which has it at home in that flat.
import { afterAll, beforeAll, describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';

describe('moving in', async () => {
  // --- the page's world in bun: content from web/public (put back for whatever runs next) ---
  usePublicFetch();
  const realRandom = Math.random;
  afterAll(() => {
    Math.random = realRandom;
  });
  const { AssetRegistry } = await import('../src/assets/registry');
  const { randomHousehold } = await import('../src/game/household');
  const { EMPTY_LOT, assembleTown, generateWorld, useTownContent } = await import('../src/game/town');
  const { homesOf, keepForSale, peopleStreams, populate, FILL_CAP } = await import('../src/game/people');
  const { homeKey, homesForSale, sameHome } = await import('../src/ui/homes');
  const { mulberry32 } = await import('../src/game/noise');
  const { householdsAhead, moveInRequest, townOnMoveIn } = await import('../src/ui/newGame');
  const { residentsOf } = await import('../src/game/household');
  type NeighbourhoodDraft = import('../src/game/town').NeighbourhoodDraft;
  type PlotSlot = import('../src/game/town').PlotSlot;
  type HomeRef = import('../src/ui/homes').HomeRef;
  const content = await loadContent();
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const t = await templatesOf(content);

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
  /** Every home (houses' lots and flats), and those for sale but `but` (the player's). */
  const homes = (town: NeighbourhoodDraft) => homesOf(content, t, town).length;
  const forSale = (town: NeighbourhoodDraft, but?: HomeRef) => homesForSale(content, t, town).filter((h) => !sameHome(h, but));
  /** A household's home as the request names it. */
  const ref = (h: HomeRef): HomeRef => (h.flat === undefined ? { slot: h.slot } : { slot: h.slot, flat: h.flat });
  /** A house's lot for sale, and a flat for sale. */
  const house = (town: NeighbourhoodDraft) => ref(forSale(town).find((h) => h.flat === undefined)!);
  const flat = (town: NeighbourhoodDraft) => ref(forSale(town).find((h) => h.flat !== undefined)!);
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

  test('a household made in Creation moves into the home chosen, as it stands or on an empty lot', () => {
    const town = firstWave(hamlet);
    const home = house(town).slot;
    const household = player();
    const request = moveInRequest({ town, slot: home, household, from: null, ...common });
    expect(request).toMatchObject({ kind: 'new', slot: home, household, fill: true, peopleSeed: SEED });
    expect(request.existing).toBeUndefined();
    expect(request.flat).toBeUndefined();
    expect(request.town).toBe(town);
    const empty = moveInRequest({ town, slot: home, household, from: null, ...common, emptyLot: true });
    expect(empty.town.slots[home].template).toBe(EMPTY_LOT);
  });

  test('a household made in Creation moves into a flat: its building and its flat there, never an empty lot', () => {
    const town = firstWave(hamlet);
    const home = flat(town);
    const household = player();
    const request = moveInRequest({ town, ...home, household, from: null, ...common, emptyLot: true });
    expect(request).toMatchObject({ kind: 'new', slot: home.slot, flat: home.flat, household, fill: true });
    expect(request.town).toBe(town);
    // The rest are filled round them; their flat is theirs (nobody else moves in), the building's others are filled or for sale.
    const filled = townOnMoveIn(content, assets, t, request);
    expect(filled.households.some((h) => sameHome(h, home))).toBe(false);
    expect(forSale(filled, home).length).toBe(keepForSale(homes(hamlet)));
    expect((filled.shells ?? []).some((s) => sameHome(s, home))).toBe(false);
  });

  test("one of the town's households stays in its home (taken over), or moves and leaves its home for sale", () => {
    const town = firstWave(hamlet);
    const own = town.households.find((h) => h.flat === undefined)!;
    const stay = moveInRequest({ town, slot: own.slot, household: own.household, from: ref(own), ...common, emptyLot: true });
    expect(stay).toMatchObject({ kind: 'new', slot: own.slot, existing: true, household: own.household, fill: true });
    expect(stay.town).toBe(town);
    const elsewhere = house(town).slot;
    const moved = moveInRequest({ town, slot: elsewhere, household: own.household, from: ref(own), ...common });
    expect(moved.existing).toBeUndefined();
    expect(moved.town.households.some((h) => h.slot === own.slot)).toBe(false);
    expect(moved.town.households.length).toBe(town.households.length - 1);
    expect(forSale(moved.town).map((h) => h.key)).toContain(homeKey(own));
    // Their old home is filled with the rest (it isn't among the few cheapest kept here, or it is for sale).
    const filled = townOnMoveIn(content, assets, t, moved);
    expect(filled.households.some((h) => h.household === own.household)).toBe(false);
    // (No newcomer takes their family name.)
    expect(filled.households.every((h) => h.household.name !== own.household.name)).toBe(true);
  });

  test("one of the town's households in a flat stays in it (taken over), or moves and leaves only its flat for sale", () => {
    const town = firstWave(hamlet);
    const own = town.households.find((h) => h.flat !== undefined)!;
    expect(own).toBeDefined();
    const stay = moveInRequest({ town, ...ref(own), household: own.household, from: ref(own), ...common, emptyLot: true });
    expect(stay).toMatchObject({ kind: 'new', slot: own.slot, flat: own.flat, existing: true, household: own.household });
    expect(stay.town).toBe(town);
    const moved = moveInRequest({ town, ...house(town), household: own.household, from: ref(own), ...common });
    expect(moved.existing).toBeUndefined();
    expect(moved.flat).toBeUndefined();
    // (Only their flat: the building's other households stay.)
    expect(moved.town.households.length).toBe(town.households.length - 1);
    expect(moved.town.households.filter((h) => h.slot === own.slot).length).toBe(town.households.filter((h) => h.slot === own.slot).length - 1);
    expect(forSale(moved.town).map((h) => h.key)).toContain(homeKey(own));
  });

  test('Move in fills every home but the few kept for sale, the same people twice', () => {
    const town = firstWave(hamlet);
    const home = house(town).slot;
    const request = moveInRequest({ town, slot: home, household: player(), from: null, ...common });
    const filled = townOnMoveIn(content, assets, t, request);
    // (The player's home is not among the town's households: the game moves them in.)
    expect(forSale(filled, { slot: home }).length).toBe(keepForSale(homes(hamlet)));
    expect(filled.households.slice(0, town.households.length)).toEqual(town.households);
    // The newcomers take no household's family name as the player's (the first wave was drawn before the player was made).
    expect(filled.households.slice(town.households.length).every((h) => h.household.name !== request.household.name)).toBe(true);
    expect(JSON.stringify(townOnMoveIn(content, assets, t, request).households)).toBe(JSON.stringify(filled.households));
    // Another people seed: other newcomers.
    const other = townOnMoveIn(content, assets, t, { ...request, peopleSeed: SEED + 1 });
    expect(JSON.stringify(other.households)).not.toBe(JSON.stringify(filled.households));
    // A household taking over its own home: the town is filled round it too.
    const own = town.households[0];
    const stayed = townOnMoveIn(content, assets, t, moveInRequest({ town, ...ref(own), household: own.household, from: ref(own), ...common }));
    expect(forSale(stayed).length).toBe(keepForSale(homes(hamlet)));
  });

  test('a big town is filled up to the cap and no further, every home but the kept lived in', () => {
    const big = gridTown(480);
    const town = firstWave(big);
    const home = forSale(town).find((h) => (h.lot.area ?? 0) === 0)!.slot;
    const request = moveInRequest({ town, slot: home, household: player(), from: null, ...common });
    const filled = townOnMoveIn(content, assets, t, request);
    expect(residents(filled)).toBeLessThanOrEqual(FILL_CAP);
    expect(residents(filled)).toBeGreaterThan(FILL_CAP - 4);
    expect(forSale(filled, { slot: home }).length).toBe(keepForSale(homes(big)));
    // A family name to every household: the names outlast a town filled to the cap.
    expect(new Set(filled.households.map((h) => h.household.name)).size).toBe(filled.households.length);
    // The village has its households.
    expect(filled.households.some((h) => (filled.slots[h.slot].area ?? 0) === 1)).toBe(true);
    // A smaller tier: fewer homes.
    expect(townOnMoveIn(content, assets, t, request, Math.random, 'near').households.length).toBeLessThan(filled.households.length);
  });

  test('the houses furnished ahead in step 4 are for the households Move in moves into them', () => {
    const town = firstWave(gridTown(180));
    const household = player();
    const ahead = new Map(householdsAhead(content, assets, t, town, SEED, household).map((h) => [homeKey(h), h.household]));
    const home = forSale(town).find((h) => (h.lot.area ?? 0) === 0)!.slot;
    const filled = townOnMoveIn(content, assets, t, moveInRequest({ town, slot: home, household, from: null, ...common }));
    // (A house is furnished for who lives in it: the same residents, the same furniture.)
    const same = filled.households.filter((h) => ahead.has(homeKey(h)) && JSON.stringify(residentsOf(content, ahead.get(homeKey(h))!)) === JSON.stringify(residentsOf(content, h.household)));
    expect(same.length).toBeGreaterThan(filled.households.length * 0.95);
  });

  test('a request without the fill (or with no people seed: then a seed of its own) starts as it is', () => {
    const town = firstWave(hamlet);
    const home = house(town).slot;
    const request = moveInRequest({ town, slot: home, household: player(), from: null, ...common });
    expect(townOnMoveIn(content, assets, t, { ...request, fill: undefined })).toBe(town);
    const { peopleSeed: _, ...unseeded } = request;
    const filled = townOnMoveIn(content, assets, t, unseeded, mulberry32(9));
    expect(forSale(filled, { slot: home }).length).toBe(keepForSale(homes(hamlet)));
  });

  test('the game starts with the household at home in its flat, the one it chose (a new one, or one of the town\'s staying)', () => {
    type FilePlot = { kind?: string; of?: number; storey?: number; x: number; z: number; w: number; d: number };
    type TownFile = { depth: number; plots: FilePlot[]; households: { name: string; plot: number; player: boolean }[]; sims: { household: number; x: number; z: number }[] };
    const town = firstWave(hamlet);
    const home = flat(town);
    const household = player();
    const request = moveInRequest({ town, ...home, household, from: null, ...common });
    const filled = townOnMoveIn(content, assets, t, request);
    // As the session hands it to the town job (`session.ts`): the flat in the options.
    const file = JSON.parse(assembleTown(content, t, filled, household, request.slot, { flat: request.flat, mode: 'living' }, mulberry32(3))) as TownFile;
    const mine = file.households.findIndex((h) => h.player);
    expect(file.households[mine].name).toBe(household.name);
    const plot = file.plots[file.households[mine].plot];
    expect(plot).toMatchObject({ kind: 'flat', of: home.slot });
    // (At home behind its own door: on its storey's rows, inside its rectangle.)
    const sims = file.sims.filter((s) => s.household === mine);
    expect(sims.length).toBe(household.members.length);
    const lift = (plot.storey ?? 0) * file.depth;
    for (const s of sims) {
      expect(s.x).toBeGreaterThanOrEqual(plot.x);
      expect(s.x).toBeLessThanOrEqual(plot.x + plot.w);
      expect(s.z - lift).toBeGreaterThanOrEqual(plot.z);
      expect(s.z - lift).toBeLessThanOrEqual(plot.z + plot.d);
    }
    // One of the town's households taking over its flat: the same flat is the player's.
    const own = town.households.find((h) => h.flat !== undefined)!;
    const stay = moveInRequest({ town, ...ref(own), household: own.household, from: ref(own), ...common });
    const theirs = JSON.parse(assembleTown(content, t, townOnMoveIn(content, assets, t, stay), own.household, stay.slot, { existing: true, flat: stay.flat }, mulberry32(3))) as TownFile;
    const player2 = theirs.households.find((h) => h.player)!;
    expect(player2.name).toBe(own.household.name.trim());
    expect(theirs.plots[player2.plot]).toMatchObject({ kind: 'flat', of: own.slot });
  });
});
