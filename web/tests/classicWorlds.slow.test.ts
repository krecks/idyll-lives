// The classic worlds (the eight kinds a world was before places and options, `support/classicWorlds.ts`)
// come out byte for byte as they did: each world's draft, as JSON, hashes to what it did when the
// generator read the kinds' own tables (`CHARACTER` and `WORLDS`, 2026-11-15), re-pinned when the town's named
// places became lots of their own (`macro.ts` `namedLots`, 2026-11-17). Seeds 11 and 22 of
// each kind, and the worlds the pinned tests name (continuedPorts, countryBends, countryCorridors,
// driveways, gardenFences). A change that moves one on purpose re-pins it here.
import { afterAll, describe, expect, test } from 'bun:test';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { CLASSIC, type ClassicKind } from './support/classicWorlds';

const WEB = new URL('../', import.meta.url).pathname;

/** Each world: its kind, its seed, and the first 16 hex digits of its draft's SHA-256. */
const GOLDEN: [ClassicKind, number, string][] = [
  ['small', 11, '32f4b83f112ff33b'],
  ['small', 22, '205f627dbe1d32de'],
  ['medium', 11, 'e72096547a878c43'],
  ['medium', 22, 'b18313f728294bf1'],
  ['dense', 11, '8564bd8e78157f29'],
  ['dense', 22, '2a1f70ba7b3b2c29'],
  ['farm', 11, 'e83f8cace05e947b'],
  ['farm', 22, '650ef9cdc5200203'],
  ['seaside', 11, 'd3cdef4f970c0032'],
  ['seaside', 22, '74b59fc9a7c8b833'],
  ['mountain', 11, '606c8fdd226a74d8'],
  ['mountain', 22, 'c4964d8a39e4db71'],
  ['lakeside', 11, 'db3172dacdd291f5'],
  ['lakeside', 22, 'f1712457e20a2cda'],
  ['forest', 11, '72275ff526442e53'],
  ['forest', 22, '771e8cb8fcdf07e0'],
  ['dense', 808114082, '6aac33ed993a7677'],
  ['small', 199762, 'afac7df5f56e4990'],
  ['forest', 223520, '0edbb43e92864f35'],
  ['medium', 104735, 'f3cb987553ecc36d'],
  ['mountain', 231441, '2de25e72e9b93bd8'],
  ['lakeside', 239360, '56654468bbb8d0f2'],
  ['farm', 271032, '1da649c9dcb56f65'],
  ['small', 278952, 'c7966027c6233769'],
  ['forest', 278953, '5971273518443fa2'],
  ['forest', 425059, '349c2d222d2f17ec'],
  ['lakeside', 215603, 'be0c8f28de85b373'],
  ['forest', 320330, '84433c3efd062967'],
  ['dense', 320329, '6fd637a9761d718c'],
  ['seaside', 215602, 'fd96b725aaf5621c'],
  ['forest', 104735, '19a18bd83db3f52f'],
  ['forest', 112654, 'd689d9b4fec191e6'],
  ['lakeside', 225304, 'ab794081b72a644c'],
  ['small', 314192, 'e5e79bbf77fce750'],
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
