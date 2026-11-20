// Step 3's households come from the world code (`ui/newGame.ts`): the same code lives in the same
// town and offers the same cards, with `Math.random` never called (it throws here); a code without a
// people seed of its own draws one from the town's seed; "Show others" offers households not shown
// yet, the same ones for the same presses; and the Neighbours slider ends where `populate` stops.
import { afterAll, beforeAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe("step 3's households from the world code", async () => {
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
  const { generateWorld, useTownContent } = await import('../src/game/town');
  const { homesOf, householdsAtMost, offered, peopleStreams, populate } = await import('../src/game/people');
  const { offerPage, peopleKey, peopleSeedOf, startCode, townKey, withSeed } = await import('../src/ui/newGame');
  const { homeKey } = await import('../src/ui/homes');
  type Templates = import('../src/game/town').Templates;
  type NeighbourhoodDraft = import('../src/game/town').NeighbourhoodDraft;
  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };
  /** A village-sized world with a few villages: enough households for more than one offer. */
  const world = generateWorld(content, t, 'country', 5150, undefined, undefined, { town: 'village', villages: 'few' });

  beforeAll(() => {
    Math.random = () => {
      throw new Error('Math.random was called');
    };
  });

  /** What step 3 makes of `code` on `town`: its households and its first offer, as the screen makes them. */
  const step3 = (town: NeighbourhoodDraft, code: ReturnType<typeof startCode>, count?: number) => {
    const streams = peopleStreams(peopleSeedOf(code));
    const lived = populate(content, assets, t, town, count, streams);
    return { lived, offer: offerPage(lived, streams, 0), streams };
  };

  test('the same code gives the same households and the same offer; New neighbours others', () => {
    const code = startCode('country', 5150);
    const [a, b] = [step3(world, code), step3(world, code)];
    expect(JSON.stringify(b.lived.households)).toBe(JSON.stringify(a.lived.households));
    expect(b.offer).toEqual(a.offer);
    expect(a.offer.length).toBeGreaterThanOrEqual(10);
    expect(a.offer.length).toBeLessThanOrEqual(30);
    // (The first offer is the people's own, `offered`, which names each household by its lot.)
    expect(a.offer.map((h) => h.slot)).toEqual(offered(a.lived, a.streams));
    // A people seed of its own (the dice), carried in the code: other households.
    const other = step3(world, { ...code, peopleSeed: 77 });
    expect(JSON.stringify(other.lived.households)).not.toBe(JSON.stringify(a.lived.households));
    expect(step3(world, { ...code, peopleSeed: 77 }).offer).toEqual(other.offer);
  });

  test("a code without a people seed draws one from the town's seed; the land's seed takes both away", () => {
    const code = startCode('river', 1234);
    expect(peopleSeedOf(code)).toBe(peopleSeedOf({ ...code }));
    expect(peopleSeedOf({ ...code, townSeed: 9 })).not.toBe(peopleSeedOf(code));
    expect(peopleSeedOf({ ...code, townSeed: 9 })).toBe(peopleSeedOf({ ...code, townSeed: 9 }));
    expect(peopleSeedOf({ ...code, peopleSeed: 42 })).toBe(42);
    expect(peopleSeedOf(withSeed({ ...code, peopleSeed: 42 }, 99))).toBe(peopleSeedOf({ ...code, seed: 99 }));
  });

  test('the town is lived in again only when the town or the people seed changed', () => {
    const code = startCode('river', 1234);
    const town = townKey('land', code, null);
    const key = peopleKey(town, peopleSeedOf(code));
    expect(peopleKey(townKey('land', code, null), peopleSeedOf(code))).toBe(key);
    expect(peopleKey(town, peopleSeedOf({ ...code, peopleSeed: 5 }))).not.toBe(key);
    expect(peopleKey(townKey('land', { ...code, townSeed: 3 }, null), peopleSeedOf(code))).not.toBe(key);
  });

  test('Show others offers households not shown yet, the same for the same presses, then starts again', () => {
    const { lived, streams } = step3(world, startCode('country', 5150));
    const all = lived.households.length;
    expect(all).toBeGreaterThan(30);
    const pages = [0, 1, 2, 3, 4, 5].map((p) => offerPage(lived, streams, p));
    expect([0, 1, 2, 3, 4, 5].map((p) => offerPage(lived, streams, p))).toEqual(pages);
    const seen = new Set<string>();
    for (const homes of pages) {
      // (A household by its home: a house's lot, or a flat of a building, whose households share its lot.)
      const page = homes.map(homeKey);
      expect(page.length).toBeGreaterThan(0);
      expect(page.length).toBeLessThanOrEqual(30);
      expect(new Set(page).size).toBe(page.length);
      const fresh = page.filter((s) => !seen.has(s));
      // (Each press shows only households not shown yet, until fewer than ten are left: then all again.)
      if (seen.size + 10 <= all) expect(fresh).toEqual(page);
      else seen.clear();
      for (const s of page) seen.add(s);
      for (const s of page) expect(lived.households.some((h) => homeKey(h) === s)).toBe(true);
    }
  });

  test('the Neighbours slider ends where the first wave stops', () => {
    // (Every home: a house's lot, or a flat.)
    const homes = homesOf(content, t, world).length;
    const most = householdsAtMost(homes);
    expect(most).toBe(homes - Math.max(6, Math.ceil(homes / 4)));
    const code = startCode('country', 5150);
    // (A count past it moves in no more.)
    expect(step3(world, code, most + 5).lived.households.length).toBeLessThanOrEqual(most);
    expect(householdsAtMost(4)).toBe(0);
    expect(householdsAtMost(0)).toBe(0);
  });
});
