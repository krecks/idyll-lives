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
  ['small', 11, '65b1be97fe1e3191'],
  ['small', 22, '519d9e20d41c082c'],
  ['medium', 11, 'ddf6a581a6f0cb88'],
  ['medium', 22, '71e4489af28b3028'],
  ['dense', 11, '8564bd8e78157f29'],
  ['dense', 22, '7708dbddba985006'],
  ['farm', 11, 'ba128a3bee768e1f'],
  ['farm', 22, 'e6ee40b54d0591e4'],
  ['seaside', 11, 'd3cdef4f970c0032'],
  ['seaside', 22, 'aa0ee82e027927e0'],
  ['mountain', 11, '042a688e3de239d5'],
  ['mountain', 22, 'c1ea989290bd779d'],
  ['lakeside', 11, 'dd85229cfac37fd9'],
  ['lakeside', 22, '8c840701aef6e113'],
  ['forest', 11, 'e5c3dba15c0cccc6'],
  ['forest', 22, '8195b415cdba5465'],
  ['dense', 808114082, '6aac33ed993a7677'],
  ['small', 199762, '4c0c2d7f548ee0d1'],
  ['forest', 223520, 'a8570296d140adde'],
  ['medium', 104735, 'f9d3544a12595349'],
  ['mountain', 231441, '1177afd19ad20fb1'],
  ['lakeside', 239360, 'c494858a211c38ef'],
  ['farm', 271032, '204b8ace062d55ea'],
  ['small', 278952, 'c7966027c6233769'],
  ['forest', 278953, 'cee41c838bf3bbc5'],
  ['forest', 425059, '132cb9c2b8cd3979'],
  ['lakeside', 215603, 'be0c8f28de85b373'],
  ['forest', 320330, '927d8ffbc581dc17'],
  ['dense', 320329, '9d417f89bbd28191'],
  ['seaside', 215602, '9c03bad3d8d82013'],
  ['forest', 104735, '56e14fa9ddb6da53'],
  ['forest', 112654, 'a59f1ff875e569d3'],
  ['lakeside', 225304, 'e2af06347061df4b'],
  ['small', 314192, 'b4052fd5432c6d9d'],
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
