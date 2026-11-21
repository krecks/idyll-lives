// The classic worlds (the eight kinds a world was before places and options, `support/classicWorlds.ts`)
// come out byte for byte as they did: each world's draft, as JSON, hashes to what it did when the
// generator read the kinds' own tables (`CHARACTER` and `WORLDS`, 2026-11-15), re-pinned when the town's named
// places became lots of their own (`macro.ts` `namedLots`, 2026-11-17), when a village's hall
// got a look of its own (`Civic.kinds`, 2026-11-18), and for the real town (2026-11-20: every building a
// lot, laid by the dense pass, with buildings of flats, car parks and bays; the country road's corner rule;
// 679 family names). Seeds 11 and 22 of
// each kind, and the worlds the pinned tests name (continuedPorts, countryBends, countryCorridors,
// driveways, gardenFences). A change that moves one on purpose re-pins it here.
import { afterAll, describe, expect, test } from 'bun:test';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { CLASSIC, type ClassicKind } from './support/classicWorlds';

const WEB = new URL('../', import.meta.url).pathname;

/** Each world: its kind, its seed, and the first 16 hex digits of its draft's SHA-256. */
const GOLDEN: [ClassicKind, number, string][] = [
  ['small', 11, '3d9227655ebfe9dc'],
  ['small', 22, '6ad7f41082bd66a2'],
  ['medium', 11, '4909f41e7000bdd1'],
  ['medium', 22, '0d4b19adbf3af0a4'],
  ['dense', 11, '1f1a242101ee2f8e'],
  ['dense', 22, 'a817305645090a0d'],
  ['farm', 11, 'd708670f22d717f2'],
  ['farm', 22, 'b27f95101362f936'],
  ['seaside', 11, '720b3bbbaf2bc456'],
  ['seaside', 22, '540dedd887ef7818'],
  ['mountain', 11, 'f50fd689893f7d6f'],
  ['mountain', 22, '9ece85d3a0c95bcf'],
  ['lakeside', 11, 'f3fea687551341ef'],
  ['lakeside', 22, '04257be9db8b5047'],
  ['forest', 11, 'e11b78823849e74a'],
  ['forest', 22, '08057addf8b177d4'],
  ['dense', 808114082, 'ed93ae09db773b30'],
  ['small', 199762, '3fe1c61c744070fd'],
  ['forest', 223520, 'cf1a69f6e7c3475c'],
  ['medium', 104735, '66f24554280dd7cb'],
  ['mountain', 231441, '6c375fa9df540335'],
  ['lakeside', 239360, '3b7d1ee2ebdf83db'],
  ['farm', 271032, '9fb6c264e38584f9'],
  ['small', 278952, '673b977eeb7adb7a'],
  ['forest', 278953, '532c0f4a29e2f527'],
  ['forest', 425059, '5818192cca0c4606'],
  ['lakeside', 215603, '3c9efe3efe1496ee'],
  ['forest', 320330, 'fcdacfe9a70200cf'],
  ['dense', 320329, '9a8631f340ca23ee'],
  ['seaside', 215602, '74e909ddea32264a'],
  ['forest', 104735, 'ccca0ba61b7fab9f'],
  ['forest', 112654, '969d1ece389c9950'],
  ['lakeside', 225304, '36ffef092aca7789'],
  ['small', 314192, '6684f2b66654b198'],
];

describe('classic worlds', async () => {
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
  const { useTownContent } = await import('../src/game/town');
  const { generateFromSpec } = await import('../src/game/macro');
  type Templates = import('../src/game/town').Templates;
  const content = await Content.load('http://localhost/content/base.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };
  const hash = (text: string) => createHash('sha256').update(text).digest('hex').slice(0, 16);

  for (const [kind, seed, want] of GOLDEN) {
    test(
      `${kind} ${seed}`,
      () => {
        expect(hash(JSON.stringify(generateFromSpec(content, t, CLASSIC[kind], seed)))).toBe(want);
      },
      60_000,
    );
  }
});
