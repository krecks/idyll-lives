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
// lots along it half a metre nearer its middle). A change that moves one on purpose re-pins it here.
import { afterAll, describe, expect, test } from 'bun:test';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { CLASSIC, type ClassicKind } from './support/classicWorlds';

const WEB = new URL('../', import.meta.url).pathname;

/** Each world: its kind, its seed, and the first 16 hex digits of its draft's SHA-256. */
const GOLDEN: [ClassicKind, number, string][] = [
  ['small', 11, '07b88856165f9b1e'],
  ['small', 22, 'e0f0fe9d6c85a4ab'],
  ['medium', 11, '5c19db5ae9af8eb5'],
  ['medium', 22, '8a562db6bf56739f'],
  ['dense', 11, '17d10e5878e77ea1'],
  ['dense', 22, 'e100fec2c5f30940'],
  ['farm', 11, 'bd808b9828aaaaff'],
  ['farm', 22, '0037f8b77a2f2d61'],
  ['seaside', 11, '8b72fe5d245e8b85'],
  ['seaside', 22, '1410367b926bbff6'],
  ['mountain', 11, 'a4dd6e5d018043c5'],
  ['mountain', 22, 'bc583e55d48cc9b2'],
  ['lakeside', 11, '84e8654a78c69428'],
  ['lakeside', 22, 'd51e568bfbd79849'],
  ['forest', 11, '04fff2fc623a83b9'],
  ['forest', 22, 'cc70c7dafd372569'],
  ['dense', 808114082, '2cf7075f6168fe6a'],
  ['small', 199762, 'f0d0570365b940b7'],
  ['forest', 223520, 'e999deeb73df4661'],
  ['medium', 104735, '1133e075c1f0e1ab'],
  ['mountain', 231441, 'f98dc63a9d60d84c'],
  ['lakeside', 239360, '45c90d0953a32bd3'],
  ['farm', 271032, '5fc8b9aae9ae4740'],
  ['small', 278952, '80b93db5a9271f04'],
  ['forest', 278953, '12cb391948ad602b'],
  ['forest', 425059, '2c5d6f841b881ae9'],
  ['lakeside', 215603, '05de7b80da481753'],
  ['forest', 320330, '03316b99844403f3'],
  ['dense', 320329, 'cb48dd10a8682932'],
  ['seaside', 215602, '7d8b56a18308f887'],
  ['forest', 104735, '556f018715231ee0'],
  ['forest', 112654, 'e0d2706f13e5f6e4'],
  ['lakeside', 225304, 'df5dad0f9586780f'],
  ['small', 314192, 'a05d3592400e1df5'],
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
