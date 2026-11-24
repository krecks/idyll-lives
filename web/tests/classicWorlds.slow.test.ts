// The classic worlds (the eight kinds a world was before places and options, `support/classicWorlds.ts`)
// come out byte for byte as they did: each world's draft, as JSON, hashes to what it did when the
// generator read the kinds' own tables (`CHARACTER` and `WORLDS`, 2026-11-15), re-pinned when the town's named
// places became lots of their own (`macro.ts` `namedLots`, 2026-11-17), when a village's hall
// got a look of its own (`Civic.kinds`, 2026-11-18), and for the real town (2026-11-20: every building a
// lot, laid by the dense pass, with buildings of flats, car parks and bays; the country road's corner rule;
// 679 family names), and when the generator's math became the same on every machine and engine (2026-11-21:
// `game/pmath.ts` for `Math.sin`, `Math.hypot` and the like, whose last bits Bun takes from the system's libm, so
// Linux's made other towns than macOS's), and when a road whose name and its lane's were taken got the street's other
// words before a number (2026-11-24: "Elmfield Way", not "Elmfield Lane 21"). Seeds 11 and 22 of
// each kind, and the worlds the pinned tests name (continuedPorts, countryBends, countryCorridors,
// driveways, gardenFences). A change that moves one on purpose re-pins it here.
import { afterAll, describe, expect, test } from 'bun:test';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { CLASSIC, type ClassicKind } from './support/classicWorlds';

const WEB = new URL('../', import.meta.url).pathname;

/** Each world: its kind, its seed, and the first 16 hex digits of its draft's SHA-256. */
const GOLDEN: [ClassicKind, number, string][] = [
  ['small', 11, '860b033e0ef01381'],
  ['small', 22, '6a347d81b289f7b6'],
  ['medium', 11, '21ebe4df22ed2d0c'],
  ['medium', 22, '1d77f639d23e69f7'],
  ['dense', 11, '5c4c927ff8f8ddb2'],
  ['dense', 22, '7775dd7e37329843'],
  ['farm', 11, '75c8d346d3565167'],
  ['farm', 22, 'f76862e49e228027'],
  ['seaside', 11, 'af50ded43e9a852c'],
  ['seaside', 22, '82ad4deff7b97ebf'],
  ['mountain', 11, '1a17adbc98a9ccb9'],
  ['mountain', 22, '60469f472e8f4e7c'],
  ['lakeside', 11, 'a3c664eeadc90aa9'],
  ['lakeside', 22, '7eaa2fd5082cdd72'],
  ['forest', 11, '5baed11c166e1a99'],
  ['forest', 22, '8e0d89d8f2e9bc65'],
  ['dense', 808114082, '73592bc1ea8f4b44'],
  ['small', 199762, '8cb4cf8d2baf2335'],
  ['forest', 223520, '54f69f8756da4c61'],
  ['medium', 104735, 'faeeaebfac73398d'],
  ['mountain', 231441, 'b60c8a80ef212a40'],
  ['lakeside', 239360, 'd870f5d0915420f6'],
  ['farm', 271032, 'b3961b2df99d7fed'],
  ['small', 278952, '7c7fff78378056b1'],
  ['forest', 278953, '1bf87f859eb80bc7'],
  ['forest', 425059, 'b8a6f6a1344ffe05'],
  ['lakeside', 215603, '05553121ad55d979'],
  ['forest', 320330, 'e0b54ada73f9312b'],
  ['dense', 320329, '63758c1381d6fb79'],
  ['seaside', 215602, 'bf997a89bc47be53'],
  ['forest', 104735, 'b3fb5c185f8816f0'],
  ['forest', 112654, '9a4b3d81959411d6'],
  ['lakeside', 225304, 'cf657bba1980c9bc'],
  ['small', 314192, 'c28578030175681e'],
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
