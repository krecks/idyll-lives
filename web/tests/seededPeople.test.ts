// The town's people come from the world code's people seed (`game/people.ts`): the same seed makes
// the same households in both waves, the same residents offered and the same bonds in the town
// file, byte for byte, with `Math.random` never called (it throws here). A home's household comes
// from its lot's own stream, so taking away a lot elsewhere leaves it be. After Move in every home
// is lived in but for the few cheapest kept for sale, up to the cap, with smaller households in the
// smallest homes when the cap is near; a smaller tier is the same people in fewer homes; each
// neighbour knows about 35 others, likelier those living near; the town file says the most
// residents the town grows to; and the houses furnished ahead are the ones Move in finds made.
import { afterAll, describe, expect, test } from 'bun:test';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('seeded people', async () => {
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
  const { assembleTown, generateWorld, useTownContent, vacantSlots } = await import('../src/game/town');
  const { defaultResidentsMost, FILL_CAP, fillPlan, fillTown, homesOf, keepForSale, neighbourBonds, offered, peopleStreams, populate, residentsMost, SIM_RESIDENTS, tierFor } = await import('../src/game/people');
  const { mulberry32 } = await import('../src/game/noise');
  const { runTownJob } = await import('../src/game/townJobs');
  type Templates = import('../src/game/town').Templates;
  type NeighbourhoodDraft = import('../src/game/town').NeighbourhoodDraft;
  type PlotSlot = import('../src/game/town').PlotSlot;
  type TownJob = import('../src/game/townJobs').TownJob;
  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  /**
   * A town of the shipped fixed houses on a grid, its own and nothing the generator makes, so the
   * pinned households below move only when the people's code or content does: a main town of
   * `size` homes and a village of twelve 400 m off.
   */
  const gridTown = (size = 48): NeighbourhoodDraft => {
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
    for (let i = 0; i < size; i++) add(40 + (i % 8) * 30, 40 + Math.floor(i / 8) * 40, 0);
    for (let i = 0; i < 12; i++) add(640 + (i % 4) * 30, 80 + Math.floor(i / 4) * 40, 1);
    const areas = [
      { name: 'Grid', kind: 'hamlet' as const, x: 0, z: 0, w: 320, d: 320 },
      { name: 'Outlying', kind: 'hamlet' as const, x: 600, z: 40, w: 160, d: 160 },
    ];
    return { name: 'Grid', width: 800, depth: 400, slots, roads: [], greens: [], households: [], open: { areas } as NeighbourhoodDraft['open'] };
  };
  /** A small generated world (a hamlet, no villages), for the town file. */
  const hamlet = generateWorld(content, t, 'country', 4242, undefined, undefined, { town: 'hamlet', villages: 'none' });

  // From here on nothing may roll Creation's dice: set now, not in a beforeAll, so the towns built while the
  // tests are collected (the big one below) are held to it too.
  Math.random = () => {
    throw new Error('Math.random was called');
  };

  /** Both waves of a town's people from `seed`, the player moving into the first home left, and whom step 3 offers. */
  const people = (town: NeighbourhoodDraft, seed: number, cap = FILL_CAP) => {
    const streams = peopleStreams(seed);
    const first = populate(content, assets, t, town, undefined, streams);
    const offer = offered(first, streams);
    const home = vacantSlots(first)[0].index;
    const filled = fillTown(content, assets, t, first, home, streams, { cap });
    return { first, offer, home, filled, streams };
  };
  const residents = (town: NeighbourhoodDraft, but?: number) => town.households.filter((h) => h.slot !== but).reduce((n, h) => n + h.household.members.length, 0);
  const player = () => randomHousehold(content, assets, 2, [], {}, mulberry32(1));

  test('the same seed makes the same households, offer and fill, twice; another seed others', () => {
    expect(() => Math.random()).toThrow();
    for (const town of [gridTown(), hamlet]) {
      const [a, b] = [people(town, 2026), people(town, 2026)];
      expect(JSON.stringify(b.first.households)).toBe(JSON.stringify(a.first.households));
      expect(JSON.stringify(b.offer)).toBe(JSON.stringify(a.offer));
      expect(JSON.stringify(b.filled.households)).toBe(JSON.stringify(a.filled.households));
      expect(a.filled.households.length).toBeGreaterThan(a.first.households.length);
      const other = people(town, 2027);
      expect(JSON.stringify(other.filled.households)).not.toBe(JSON.stringify(a.filled.households));
    }
  });

  test("the worker's town file is the page's, and the same twice, its bonds from the people seed", () => {
    const { filled, home, streams } = people(hamlet, 31);
    const job: TownJob = { kind: 'assemble', town: filled, player: player(), slot: home, options: { mode: 'living', lifespan: 'normal' }, random: streams.bonds };
    // (Sent and returned as `postMessage` does.)
    const inWorker = structuredClone(runTownJob(content, t, structuredClone(job)));
    const again = runTownJob(content, t, job);
    if (inWorker.kind !== 'assemble' || again.kind !== 'assemble') throw new Error('not a town file');
    const page = assembleTown(content, t, filled, job.player, home, job.options, mulberry32(streams.bonds));
    expect(inWorker.lot).toBe(page);
    expect(again.lot).toBe(page);
    const file = JSON.parse(page) as { sims: unknown[]; households: unknown[] };
    expect(file.households.length).toBe(filled.households.length + 1);
  });

  test("one seed's households are as pinned (a change in how people are drawn re-rolls everyone: re-pin on purpose)", () => {
    const { first, offer, filled } = people(gridTown(), 7);
    const pin = (v: unknown) => createHash('sha256').update(JSON.stringify(v)).digest('hex').slice(0, 16);
    // (What they are, to read when the pin moves.)
    expect(first.households.length).toBe(45);
    expect(filled.households.length).toBe(56);
    expect(offer.length).toBe(27);
    expect(filled.households[0].household.name).toBe('Lima');
    expect(pin(first.households)).toBe('77df76b764efcda3');
    expect(pin(filled.households)).toBe('848ce1fc0badd9fc');
    expect(pin(offer)).toBe('3795e135ec008d44');
  });

  test("taking away a lot elsewhere leaves a home's household alone", () => {
    const town = gridTown();
    const before = people(town, 99);
    // The village's last lot goes; the others are numbered again.
    const fewer: NeighbourhoodDraft = { ...town, slots: town.slots.slice(0, -1).map((s, index) => ({ ...s, index })) };
    const after = people(fewer, 99);
    const byPlace = (d: NeighbourhoodDraft) => new Map(d.households.map((h) => [`${d.slots[h.slot].x},${d.slots[h.slot].z}`, h.household]));
    const [was, now] = [byPlace(before.filled), byPlace(after.filled)];
    const both = [...was.keys()].filter((k) => now.has(k));
    expect(both.length).toBeGreaterThan(40);
    // Everyone in every home lived in both times is the same person. (A family name may move: there
    // are fewer of them than households, and one another household took first is passed over.)
    for (const k of both) expect(now.get(k)!.members).toEqual(was.get(k)!.members);
    // The first household to move in, which no other could take a name from, wholly the same.
    const firstIn = (d: NeighbourhoodDraft) => `${d.slots[d.households[0].slot].x},${d.slots[d.households[0].slot].z}`;
    expect(firstIn(after.first)).toBe(firstIn(before.first));
    expect(now.get(firstIn(after.first))).toEqual(was.get(firstIn(before.first))!);
  });

  test('after Move in every home but the kept cheapest is lived in, up to the cap', () => {
    for (const town of [gridTown(), hamlet]) {
      const { first, filled, home } = people(town, 5);
      // (Every home: a house's lot, or a flat.)
      const homes = homesOf(content, t, town);
      const key = (h: { slot: number | PlotSlot; flat?: number }) => `${typeof h.slot === 'number' ? h.slot : h.slot.index}:${h.flat ?? ''}`;
      const vacant = (d: NeighbourhoodDraft) => homes.filter((h) => !d.households.some((q) => key(q) === key(h)));
      // The first wave leaves a quarter of the homes, six at least, to choose from.
      expect(vacant(first).length).toBeGreaterThanOrEqual(Math.max(6, Math.ceil(homes.length / 4)));
      // After it, only the kept ones and the player's.
      const left = vacant(filled).filter((h) => key(h) !== `${home}:`);
      expect(left.length).toBe(keepForSale(homes.length));
      expect(residents(filled, home)).toBeLessThanOrEqual(FILL_CAP);
      // The cheapest of those for sale were kept.
      const priceOf = new Map(homes.map((h) => [key(h), h.price]));
      const moved = filled.households.slice(first.households.length).map((h) => priceOf.get(key(h))!);
      expect(Math.max(...left.map((h) => h.price))).toBeLessThanOrEqual(Math.min(...moved));
      // The village has its households.
      if (town === hamlet) continue;
      expect(filled.households.filter((h) => town.slots[h.slot].area === 1).length).toBe(12);
    }
  });

  test('near the cap, the fill stops there with smaller households, the village and the player\'s street first', () => {
    const town = gridTown(96);
    const { first, home } = people(town, 12);
    const cap = residents(first) + 20;
    const streams = peopleStreams(12);
    const plan = fillPlan(content, assets, t, first, home, streams, { cap });
    const total = residents(first) + plan.reduce((n, p) => n + p.household.members.length, 0);
    expect(total).toBeLessThanOrEqual(cap);
    expect(total).toBeGreaterThanOrEqual(cap - 1);
    // Every home lit counts for more than every bed taken: ones and twos.
    for (const p of plan) expect(p.household.members.length).toBeLessThanOrEqual(2);
    // Two in the village, whatever else.
    expect(plan.filter((p) => first.slots[p.slot].area === 1).length).toBeGreaterThanOrEqual(2);
    // The first in are the player's neighbours.
    const at = first.slots[home];
    const near = (slot: number) => Math.hypot(first.slots[slot].x - at.x, first.slots[slot].z - at.z) <= 80;
    expect(near(plan[0].slot)).toBe(true);
    // Furnishing ahead (no home chosen yet) asks for the same households in the homes it shares with it.
    const ahead = new Map(fillPlan(content, assets, t, first, undefined, streams, { cap: 10_000 }).map((p) => [p.slot, p.household]));
    const full = fillPlan(content, assets, t, first, home, streams, { cap: 10_000 });
    for (const p of full) expect(ahead.get(p.slot)?.members).toEqual(p.household.members);
  });

  test('each neighbour knows about 35 others, likelier those living near; a small town three in ten', () => {
    const ties = (households: number, per: number, spacing: number) => {
      const homes = Array.from({ length: households }, (_, i): [number, number] => [(i % 12) * spacing, Math.floor(i / 12) * spacing]);
      const sims = homes.flatMap((_, h) => Array.from({ length: per }, () => ({ household: h })));
      const bonds = neighbourBonds(sims, homes, mulberry32(3));
      const count = new Array<number>(sims.length).fill(0);
      for (const b of bonds) {
        expect(sims[b.a].household).not.toBe(sims[b.b].household);
        count[b.a]++;
        count[b.b]++;
      }
      const far = (b: { a: number; b: number }) => Math.hypot(homes[sims[b.a].household][0] - homes[sims[b.b].household][0], homes[sims[b.a].household][1] - homes[sims[b.b].household][1]);
      return { mean: count.reduce((a, b) => a + b, 0) / sims.length, bonds, far, sims };
    };
    // 118, 150, 560 and 680 residents.
    for (const [households, per] of [
      [59, 2],
      [75, 2],
      [280, 2],
      [340, 2],
    ]) {
      const { mean } = ties(households, per, 40);
      expect(mean).toBeGreaterThan(32);
      expect(mean).toBeLessThan(38);
    }
    // Nearer, likelier: the ties' mean distance is well under that of all pairs.
    const { bonds, far, sims } = ties(280, 2, 40);
    const pairs: number[] = [];
    for (let a = 0; a < sims.length; a += 7) for (let b = a + 1; b < sims.length; b += 5) if (sims[a].household !== sims[b].household) pairs.push(far({ a, b }));
    const meanOf = (v: number[]) => v.reduce((x, y) => x + y, 0) / v.length;
    expect(meanOf(bonds.map(far))).toBeLessThan(meanOf(pairs) * 0.8);
    // Of the ties, about a tenth rivals and a fifth friends.
    const share = (p: string) => bonds.filter((b) => b.preset === p).length / bonds.length;
    expect(share('rivals')).toBeGreaterThan(0.07);
    expect(share('rivals')).toBeLessThan(0.13);
    expect(share('friends')).toBeGreaterThan(0.16);
    expect(share('friends')).toBeLessThan(0.24);
    // Thirty residents: each knows about three in ten of the others.
    const small = ties(15, 2, 40).mean;
    expect(small).toBeGreaterThan(0.3 * 29 * 0.75);
    expect(small).toBeLessThan(0.3 * 29 * 1.25);
  });

  test('the town file holds about 35 ties a neighbour, none for a newly arrived household', () => {
    const { filled, home, streams } = people(gridTown(96), 8);
    const p = player();
    const file = JSON.parse(assembleTown(content, t, filled, p, home, {}, mulberry32(streams.bonds))) as { sims: { household: number }[]; relationships: { a: number; b: number; preset: string }[] };
    const neighbourly = new Set(['friends', 'rivals', 'acquaintances']);
    const count = new Array<number>(file.sims.length).fill(0);
    for (const r of file.relationships) {
      if (!neighbourly.has(r.preset)) continue;
      count[r.a]++;
      count[r.b]++;
    }
    expect(count.slice(0, p.members.length).every((n) => n === 0)).toBe(true);
    const neighbours = count.slice(p.members.length);
    const mean = neighbours.reduce((a, b) => a + b, 0) / neighbours.length;
    expect(neighbours.length).toBeGreaterThan(118);
    expect(mean).toBeGreaterThan(31);
    expect(mean).toBeLessThan(39);
  });

  /** A town too big for its natural households under `FILL_CAP` (612 homes of one or two bedrooms), both waves from seed 21. */
  const big = (() => {
    const town = gridTown(600);
    const streams = peopleStreams(21);
    const first = populate(content, assets, t, town, undefined, streams);
    const home = vacantSlots(first)[0].index;
    return { town, streams, first, home, filled: fillTown(content, assets, t, first, home, streams) };
  })();

  test('a full fill of a big town lights every home but the kept, under FILL_CAP, the smallest homes in ones and twos', () => {
    const { town, first, filled, home, streams } = big;
    const houses = town.slots.filter((s) => s.kind === 'house');
    expect(houses.length).toBe(612);
    const left = vacantSlots(filled).filter((s) => s.index !== home);
    expect(left.length).toBe(keepForSale(houses.length));
    const total = residents(filled, home);
    expect(total).toBeLessThanOrEqual(FILL_CAP);
    expect(total).toBeGreaterThan(FILL_CAP - 4);
    // The households the homes would naturally have passed the cap; the size rule made those of the
    // smallest homes ones and twos first: wherever a home's household was made smaller, every
    // smaller home's has one or two people.
    const natural = new Map(fillPlan(content, assets, t, first, home, streams, { cap: 10_000 }).map((p) => [p.slot, p.household]));
    expect(residents(first) + [...natural.values()].reduce((n, h) => n + h.members.length, 0)).toBeGreaterThan(FILL_CAP + 20);
    const moved = filled.households.slice(first.households.length);
    const bedrooms = (slot: number) => (town.slots[slot].template === 'oakridge' ? 2 : 1);
    const smaller = moved.filter((h) => h.household.members.length < natural.get(h.slot)!.members.length);
    expect(smaller.length).toBeGreaterThan(0);
    for (const c of smaller) for (const h of moved) if (bedrooms(h.slot) < bedrooms(c.slot)) expect(h.household.members.length).toBeLessThanOrEqual(2);
    expect(filled.residentsMost).toBe(SIM_RESIDENTS);
  });

  test("a smaller tier is the full fill's people in fewer homes, under its own limit", () => {
    // A big town, where each tier stops at its limit, and one where none does.
    const mid = gridTown(160);
    const midStreams = peopleStreams(22);
    const midFirst = populate(content, assets, t, mid, undefined, midStreams);
    for (const { town, streams, first, home } of [big, { town: mid, streams: midStreams, first: midFirst, home: vacantSlots(midFirst)[0].index }]) {
      const full = new Map(fillPlan(content, assets, t, first, home, streams).map((p) => [p.slot, JSON.stringify(p.household)]));
      const at = town.slots[home];
      const counts: number[] = [];
      for (const tier of ['half', 'near'] as const) {
        const plan = fillPlan(content, assets, t, first, home, streams, { tier });
        const slots = new Set(plan.map((p) => p.slot));
        counts.push(plan.length);
        expect(plan.length).toBeLessThan(full.size);
        for (const p of plan) expect(JSON.stringify(p.household)).toBe(full.get(p.slot)!);
        const filled = fillTown(content, assets, t, first, home, streams, { tier });
        expect(filled.residentsMost).toBe(residentsMost(tier));
        expect(residents(filled, home)).toBeLessThan(residentsMost(tier));
        if (town !== mid) continue;
        const inFull = [...full.keys()].map((slot) => town.slots[slot]);
        if (tier === 'half') {
          // The main town (the player's own) in full; the village half.
          for (const s of inFull.filter((s) => s.area === 0)) expect(slots.has(s.index)).toBe(true);
          const village = inFull.filter((s) => s.area === 1);
          expect(village.filter((s) => slots.has(s.index)).length).toBe(Math.ceil(village.length / 2));
        } else {
          // Every home near the player's lived in.
          for (const s of inFull.filter((s) => Math.hypot(s.x - at.x, s.z - at.z) <= 250)) expect(slots.has(s.index)).toBe(true);
        }
      }
      expect(counts[1]).toBeLessThan(counts[0]);
    }
  });

  test("the town file says the most residents the town grows to: its tier's, else what it starts with and room", () => {
    const { first, filled, home, streams } = people(hamlet, 31);
    const p = player();
    const fileOf = (town: NeighbourhoodDraft) => JSON.parse(assembleTown(content, t, town, p, home, {}, mulberry32(streams.bonds))) as { sims: unknown[]; residentsMost?: number };
    expect(fileOf(filled).residentsMost).toBe(SIM_RESIDENTS);
    expect(fileOf(fillTown(content, assets, t, first, home, streams, { tier: 'near' })).residentsMost).toBe(residentsMost('near'));
    // The first wave alone (no fill): what it starts with and room, but never under the old 240.
    const unfilled = fileOf(first);
    expect(unfilled.residentsMost).toBe(defaultResidentsMost(unfilled.sims.length));
    expect(defaultResidentsMost(120)).toBe(240);
    expect(defaultResidentsMost(400)).toBe(400 + SIM_RESIDENTS - FILL_CAP);
    expect(defaultResidentsMost(760)).toBe(SIM_RESIDENTS);
    // Every graphics preset fills the town in full (people.ts `TIER_FOR`), and so does a preset it does not know.
    expect(['low', 'medium', 'high', undefined].map(tierFor)).toEqual(['full', 'full', 'full', 'full']);
  });

  test('the furnish job furnishes the houses the households about to move in need, and the town file is the same after it', () => {
    const { first, home, streams } = people(hamlet, 44);
    const ahead = [...first.households, ...fillPlan(content, assets, t, first, undefined, streams)];
    const filled = fillTown(content, assets, t, first, home, streams);
    const job: TownJob = { kind: 'assemble', town: filled, player: player(), slot: home, options: {}, random: streams.bonds };
    const before = runTownJob(content, t, job);
    const done = runTownJob(content, t, structuredClone({ kind: 'furnish', town: first, households: ahead } satisfies TownJob));
    if (done.kind !== 'furnish' || before.kind !== 'assemble') throw new Error('not furnished');
    expect(done.houses).toBe(ahead.length);
    const after = runTownJob(content, t, job);
    if (after.kind !== 'assemble') throw new Error('not a town file');
    expect(after.lot).toBe(before.lot);
  });

  test('step 3 offers all of ten or fewer, else six in ten between 10 and 30, all different', () => {
    const streams = peopleStreams(4);
    const { first } = people(gridTown(96), 4);
    const take = (n: number) => ({ ...first, households: first.households.slice(0, n) });
    expect(offered(take(7), streams).sort((a, b) => a - b)).toEqual(take(7).households.map((h) => h.slot).sort((a, b) => a - b));
    expect(offered(take(10), streams).length).toBe(10);
    expect(offered(take(14), streams).length).toBe(10);
    expect(offered(take(30), streams).length).toBe(18);
    expect(first.households.length).toBeGreaterThan(50);
    const all = offered(first, streams);
    expect(all.length).toBe(30);
    expect(new Set(all).size).toBe(30);
    expect(offered(first, streams, 12).length).toBe(12);
    // Varied: households of more than one size among the first few.
    const sizes = new Set(all.slice(0, 4).map((slot) => first.households.find((h) => h.slot === slot)!.household.members.length));
    expect(sizes.size).toBeGreaterThan(1);
  });
});
