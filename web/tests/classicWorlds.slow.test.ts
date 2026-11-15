// The classic worlds (the eight kinds a world was before places and options, `support/classicWorlds.ts`)
// come out byte for byte as they did: each world's draft, as JSON, hashes to what it did when the
// generator read the kinds' own tables (`CHARACTER` and `WORLDS`, 2026-11-15). Seeds 11 and 22 of
// each kind, and the worlds the pinned tests name (continuedPorts, countryBends, countryCorridors,
// driveways, gardenFences). A change that moves one on purpose re-pins it here.
import { afterAll, describe, expect, test } from 'bun:test';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { CLASSIC, type ClassicKind } from './support/classicWorlds';

const WEB = new URL('../', import.meta.url).pathname;

/** Each world: its kind, its seed, and the first 16 hex digits of its draft's SHA-256. */
const GOLDEN: [ClassicKind, number, string][] = [
  ['small', 11, '029b7fb3e44d9f73'],
  ['small', 22, '168f437e76eb9363'],
  ['medium', 11, '00231126e719999f'],
  ['medium', 22, '711b3f3718522834'],
  ['dense', 11, '3b4970fe901eec6c'],
  ['dense', 22, '18d4d05028976692'],
  ['farm', 11, '4df1f233e1e40f56'],
  ['farm', 22, '12b84ca702e9479b'],
  ['seaside', 11, '76e202087b363a0f'],
  ['seaside', 22, '0ddcaa1080fc9f86'],
  ['mountain', 11, '34cce59780b11ad5'],
  ['mountain', 22, '640ba20bfe58c559'],
  ['lakeside', 11, '5241ff2bbceaabed'],
  ['lakeside', 22, '70f365b071d95064'],
  ['forest', 11, '71bc3fd78417a8a6'],
  ['forest', 22, '8d21a289056723d0'],
  ['dense', 808114082, '97c315fb700d01b7'],
  ['small', 199762, '1f43dd72ccaac745'],
  ['forest', 223520, '26b24620ab37ea89'],
  ['medium', 104735, '156fc7f2011e62d2'],
  ['mountain', 231441, '26b8f7930d5cc001'],
  ['lakeside', 239360, '3b52e589cef1edb4'],
  ['farm', 271032, 'fc347cbcff34e2e7'],
  ['small', 278952, '2565ced1789f41bf'],
  ['forest', 278953, 'f0260c1b1872dd86'],
  ['forest', 425059, '932abc484d26c0c8'],
  ['lakeside', 215603, 'b1a8d0b07288f2d4'],
  ['forest', 320330, 'db55e43df1f9eb71'],
  ['dense', 320329, 'eccea8c7037ab335'],
  ['seaside', 215602, 'c1bf368725cf4508'],
  ['mountain', 241142, '816f9a2b83b8cd68'],
  ['mountain', 207684, '7b252794364d65d4'],
  ['mountain', 128494, 'd0d3a26fea212950'],
  ['small', 314192, 'd954a1500ec7e16d'],
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
