// The classic worlds (the eight kinds a world was before places and options, `support/classicWorlds.ts`)
// come out byte for byte as they did: each world's draft, as JSON, hashes to what it did when the
// generator read the kinds' own tables (`CHARACTER` and `WORLDS`, 2026-11-15), re-pinned when the town's named
// places became lots of their own (`macro.ts` `namedLots`, 2026-11-17), when a village's hall
// got a look of its own (`Civic.kinds`, 2026-11-18), and for the real town (2026-11-20: every building a
// lot, laid by the dense pass, with buildings of flats, car parks and bays; the country road's corner rule;
// 679 family names), and when the generator's math became the same on every machine and engine (2026-11-21:
// `game/pmath.ts` for `Math.sin`, `Math.hypot` and the like, whose last bits Bun takes from the system's libm, so
// Linux's made other towns than macOS's). Seeds 11 and 22 of
// each kind, and the worlds the pinned tests name (continuedPorts, countryBends, countryCorridors,
// driveways, gardenFences), and when a lane became a lane's 6 m (2026-11-23: every area road but its first, the
// lots along it half a metre nearer its middle), and for the motorway (2026-11-23: one in every world where it fits,
// edge to edge past the main town, the country roads crossing under it at its gates; in the others, the note why
// not in their diagnostics), for traffic on the right as the game shows it (2026-11-24: each motorway's
// carriageways on the other sides of its middle line, its ramps with them), and when the motorway went down before
// the villages (2026-11-24: its course chosen with the main town alone, none costing a village; the villages kept
// clear of it, its gates chosen by them; no side T where another area's street comes up end to end). A change that
// moves one on purpose re-pins it here.
import { afterAll, describe, expect, test } from 'bun:test';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { CLASSIC, type ClassicKind } from './support/classicWorlds';

const WEB = new URL('../', import.meta.url).pathname;

/** Each world: its kind, its seed, and the first 16 hex digits of its draft's SHA-256. */
const GOLDEN: [ClassicKind, number, string][] = [
  ['small', 11, '935f849f8d210083'],
  ['small', 22, '198a2962563c309b'],
  ['medium', 11, '6c628ef9b9de4a81'],
  ['medium', 22, '8b60a0e2273abce6'],
  ['dense', 11, 'a86b065a4a241ee5'],
  ['dense', 22, 'a1f12d810c0d21d6'],
  ['farm', 11, '3d71714cfaed9283'],
  ['farm', 22, '285c197fcc8c6cdf'],
  ['seaside', 11, 'fc75918d0a7935a5'],
  ['seaside', 22, '03c0921f639277b3'],
  ['mountain', 11, 'dc32305f7d9571d4'],
  ['mountain', 22, 'b348ade4a333fcea'],
  ['lakeside', 11, 'd5c4daf4aa681807'],
  ['lakeside', 22, '5cbd3cd998ace625'],
  ['forest', 11, '7108db26efa7884a'],
  ['forest', 22, '9d08c592c0abcaea'],
  ['dense', 808114082, '554cd3b8fac06c75'],
  ['small', 199762, '2bb56668555e71c5'],
  ['forest', 223520, 'b2452fe4d7502993'],
  ['medium', 104735, '5b4b40a57b63d198'],
  ['mountain', 231441, '80e7d3702b01f75c'],
  ['lakeside', 239360, 'ae4d4bc7cac4c5ce'],
  ['farm', 271032, '33646024a240c17d'],
  ['small', 278952, 'ff9c32b3436dac89'],
  ['forest', 278953, '8c80a1791edd801f'],
  ['forest', 425059, '34ecc934ed0002c8'],
  ['lakeside', 215603, '9bdce2a77f948759'],
  ['forest', 320330, '4e3fca3dfb104ed2'],
  ['dense', 320329, 'a1ea14dad80e08bf'],
  ['seaside', 215602, 'da6d5c4a0d2b3690'],
  ['forest', 104735, 'b571e76878c569e6'],
  ['forest', 112654, '0cb3d7a68c5c5dab'],
  ['lakeside', 225304, 'ab0044b37f6da7ae'],
  ['small', 314192, '0a00f97fe85b39d9'],
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
