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
// driveways, gardenFences). A change that moves one on purpose re-pins it here.
import { afterAll, describe, expect, test } from 'bun:test';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { CLASSIC, type ClassicKind } from './support/classicWorlds';

const WEB = new URL('../', import.meta.url).pathname;

/** Each world: its kind, its seed, and the first 16 hex digits of its draft's SHA-256. */
const GOLDEN: [ClassicKind, number, string][] = [
  ['small', 11, '3d9227655ebfe9dc'],
  ['small', 22, 'be37a72cc822986c'],
  ['medium', 11, 'd1723b45108e50ee'],
  ['medium', 22, '5f28f8b86e394b55'],
  ['dense', 11, 'f16cae4ea59eb808'],
  ['dense', 22, 'd4a1294953a3a667'],
  ['farm', 11, '1e52cad2d3a38868'],
  ['farm', 22, '6bce8a7b4b3c602c'],
  ['seaside', 11, '758f33bdb8ee0881'],
  ['seaside', 22, '0807701d4292ec96'],
  ['mountain', 11, 'f50fd689893f7d6f'],
  ['mountain', 22, '086edcd7be48a54d'],
  ['lakeside', 11, '4b3b6ac322b2d4e7'],
  ['lakeside', 22, 'b4e41d923e5798c5'],
  ['forest', 11, 'ad63c770db20f123'],
  ['forest', 22, '0d624b7829991e27'],
  ['dense', 808114082, '77cec6749f3d15c1'],
  ['small', 199762, '3fe1c61c744070fd'],
  ['forest', 223520, '54f69f8756da4c61'],
  ['medium', 104735, '8b06f388ce47a51c'],
  ['mountain', 231441, '4233db96a53e3c2b'],
  ['lakeside', 239360, 'd870f5d0915420f6'],
  ['farm', 271032, 'e3aa902bc82287fa'],
  ['small', 278952, '3af7fa218c890d3b'],
  ['forest', 278953, '1bf87f859eb80bc7'],
  ['forest', 425059, '7e70feb8488a2763'],
  ['lakeside', 215603, 'c74e67fbbd1f7ef2'],
  ['forest', 320330, '52c21b8291083a5b'],
  ['dense', 320329, '9f7f956be84788cb'],
  ['seaside', 215602, '4fe185a43361fd4e'],
  ['forest', 104735, 'b3fb5c185f8816f0'],
  ['forest', 112654, '25b498e133a87742'],
  ['lakeside', 225304, 'deea75a7b9afec5a'],
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
