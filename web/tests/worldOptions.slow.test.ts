// A place's world follows its options (`world.ts` `specOf`, `macro.ts` `generateFromSpec`): the same
// place, seed and options make the same world twice; each option changes it, the way it says (the
// main town's size, how many villages, how much land, how high it stands, how much of it is wooded) while
// the world keeps its name and its look; the options are kept with it, in the draft and the town
// file; the land's relief comes with it; a river's bridges and an island's causeways span their
// water, their decks dry to walk on; and the kinds from before places (small, medium, dense) are
// the countryside at their town sizes.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('world options', async () => {
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
  const { assembleTown, generateWorld, populate, useTownContent, vacantSlots } = await import('../src/game/town');
  const { defaultsOf, expectations, placeOf, specOf } = await import('../src/game/world');
  const macro = await import('../src/game/macro');
  const { inside, lengths } = await import('../src/game/roads');
  const lengthOf = (r: { points: [number, number][] }) => lengths(r.points).at(-1)!;
  type Templates = import('../src/game/town').Templates;
  type WorldOptions = import('../src/game/world').WorldOptions;
  type Draft = import('../src/game/town').NeighbourhoodDraft;
  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  /** A heath hamlet with no villages round it (a small world, quick to make), with `options` over that. */
  const SMALL: Partial<WorldOptions> = { town: 'hamlet', villages: 'none' };
  const SEED = 4242;
  const made = new Map<string, Draft>();
  const world = (options: Partial<WorldOptions> = {}, kind = 'heath', seed = SEED) => {
    const key = JSON.stringify([kind, seed, options]);
    let draft = made.get(key);
    if (!draft) made.set(key, (draft = generateWorld(content, t, kind, seed, undefined, undefined, { ...SMALL, ...options })));
    return draft;
  };
  const share = (draft: Draft, kind: string) => {
    const polys = draft.open!.zones.filter((z) => z.kind === kind).map((z) => z.poly);
    let area = 0;
    for (const poly of polys) area += Math.abs(poly.reduce((n, p, i) => n + p[0] * poly[(i + 1) % poly.length][1] - poly[(i + 1) % poly.length][0] * p[1], 0)) / 2;
    return area / (draft.width * draft.depth);
  };
  const highest = (draft: Draft) => Math.max(...draft.open!.relief!.h);

  test('the same place, seed and options make the same world', () => {
    const again = generateWorld(content, t, 'heath', SEED, undefined, undefined, SMALL);
    expect(JSON.stringify(again)).toBe(JSON.stringify(world()));
  }, 60_000);

  test('each option changes the world, which keeps its name and its look', () => {
    const base = world();
    const changed: [Partial<WorldOptions>, (d: Draft) => void][] = [
      // A village: more lots in a bigger main town (on as much land: a village alone fits a small land too).
      [{ town: 'village' }, (d) => {
        expect(d.slots.filter((s) => s.area === 0).length).toBeGreaterThan(base.slots.filter((s) => s.area === 0).length);
        expect(d.width).toBe(base.width);
      }],
      // A bigger land, chosen: the same town on more of it.
      [{ size: 'large' }, (d) => {
        expect(d.width).toBeGreaterThan(base.width);
        expect(d.world!.options!.size).toBe('large');
      }],
      // Villages round it: more areas, a bigger world (they want a medium land).
      [{ villages: 'few' }, (d) => {
        expect(d.open!.areas.length).toBeGreaterThan(1);
        expect(d.width).toBeGreaterThan(base.width);
      }],
      // Mountains: the land stands far higher.
      [{ hills: 'mountains' }, (d) => {
        expect(highest(d)).toBeGreaterThan(highest(base) * 3);
        expect(d.world!.landscape.mountains).toBeGreaterThanOrEqual(120);
        expect(d.world!.landscape.slope).toBe(0.75);
      }],
      // Flat: lower.
      [{ hills: 'flat' }, (d) => expect(highest(d)).toBeLessThan(highest(base))],
      // Wooded: far more woods.
      [{ woods: 'wooded' }, (d) => {
        expect(share(d, 'woods')).toBeGreaterThan(share(base, 'woods') * 3);
        expect(d.world!.landscape.scatter).toBeGreaterThan(base.world!.landscape.scatter!);
      }],
    ];
    for (const [options, check] of changed) {
      const d = world(options);
      expect(JSON.stringify(d)).not.toBe(JSON.stringify(base));
      expect(d.name).toBe(base.name);
      // (The look from its own stream: the same grass and trees.)
      expect(d.world!.landscape.grass).toEqual(base.world!.landscape.grass);
      expect(d.world!.landscape.conifers).toBe(base.world!.landscape.conifers);
      check(d);
    }
  }, 120_000);

  test('the options are kept with the world, and the relief with its land', () => {
    const d = world({ hills: 'hilly' });
    expect(d.world!.options).toEqual({ ...defaultsOf('heath'), ...SMALL, hills: 'hilly', size: 'small' });
    const relief = d.open!.relief!;
    expect(relief.cell).toBe(16);
    expect(relief.cols).toBe(Math.ceil(d.width / 16));
    expect(relief.rows).toBe(Math.ceil(d.depth / 16));
    expect(relief.h.length).toBe(relief.cols * relief.rows);
    expect(relief.h.every((h) => Number.isInteger(h) && h >= 0)).toBe(true);
    // (Over the level the town is built at: much of the land at it, some well above.)
    expect(relief.h.filter((h) => h === 0).length).toBeGreaterThan(0);
    expect(highest(d)).toBeGreaterThan(50);
    // In the town file: the options with its look, the relief with its land.
    const town = populate(content, assets, t, d);
    const file = JSON.parse(assembleTown(content, t, town, randomHousehold(content, assets, 2), vacantSlots(town)[0].index));
    expect(file.meta.world.options).toEqual(d.world!.options);
    expect(file.meta.relief).toEqual(relief);
  }, 60_000);

  test('the kinds from before places are the countryside at their town sizes', () => {
    for (const [kind, town] of [['small', 'village'], ['medium', 'town'], ['dense', 'city']] as const) {
      expect(placeOf(kind)).toBe('country');
      const spec = specOf('country', { town });
      expect(spec.names).toBe(kind);
      expect(spec.options!.town).toBe(town);
    }
    expect(placeOf('nowhere')).toBe('country');
    const legacy = generateWorld(content, t, 'small', 77);
    expect(JSON.stringify(legacy)).toBe(JSON.stringify(generateWorld(content, t, 'country', 77, undefined, undefined, { town: 'village' })));
    expect(legacy.world!.kind).toBe('country');
  }, 60_000);

  test("a river's bridge spans it, its deck dry to walk on; an island's causeways are its ways out", () => {
    const { unrle } = macro;
    for (const [kind, options, crossings] of [['river', { town: 'village' }, 1], ['island', { town: 'village' }, 1], ['island', { town: 'town' }, 2]] as const) {
      const d = world(options, kind, 31);
      const bridges = d.roads.filter((r) => r.bridge);
      expect(bridges.length).toBe(crossings);
      const walk = unrle(d.open!.water, d.width * d.depth);
      const sea = d.open!.zones.filter((z) => z.kind === 'water');
      for (const r of bridges) {
        const [s0, s1] = r.bridge!;
        expect(s1 - s0).toBeGreaterThan(4);
        expect(s1 - s0).toBeLessThanOrEqual(kind === 'island' ? 140 : 45);
        expect(r.ends.includes('out')).toBe(kind === 'island');
        // Over the water (a zone), but its deck and verges not in the water the sim walks round.
        const mid = r.points[Math.round(r.points.length * ((s0 + s1) / 2 / lengthOf(r)))] ?? r.points[0];
        expect(sea.some((z) => inside(z.poly, mid[0], mid[1]))).toBe(true);
        for (const p of r.points) expect(walk[Math.min(d.depth - 1, Math.floor(p[1])) * d.width + Math.min(d.width - 1, Math.floor(p[0]))]).toBe(0);
      }
      if (kind === 'island') {
        // The sea reaches every edge of the world; the causeways leave it, one per way out.
        const corners = [[2, 2], [d.width - 2, 2], [d.width - 2, d.depth - 2], [2, d.depth - 2]];
        for (const [x, z] of corners) expect(sea.some((zone) => inside(zone.poly, x, z))).toBe(true);
        expect(d.roads.filter((r) => r.ends.includes('out')).length).toBe(crossings);
      }
    }
  }, 120_000);

  test('what a world is expected to have follows its options', () => {
    const small = expectations('country', { town: 'hamlet', villages: 'none', woods: 'wooded' });
    expect(small.homes).toBe(14);
    expect(small.empty).toBeNull();
    expect(small.built).toBeCloseTo(0.08 * 0.6);
    expect(small.careers).toBe(false);
    const big = expectations('country', { town: 'city', villages: 'many' });
    expect(big.homes).toBe(150 + 70);
    expect(big.careers).toBe(true);
    expect(expectations('heath').meadow).toBe(true);
  });
});
