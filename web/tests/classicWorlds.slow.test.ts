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
  ['small', 11, '2dd709596b4378ae'],
  ['small', 22, '083745ac0fba6841'],
  ['medium', 11, '11bd80352eb79fdd'],
  ['medium', 22, '64f86ed7e7d53446'],
  ['dense', 11, '890f4da75136af38'],
  ['dense', 22, 'f91df8ad22089afe'],
  ['farm', 11, '1717d65a2c2b5301'],
  ['farm', 22, '45d7b4ab9368e341'],
  ['seaside', 11, 'ff559a9d6b6fcc70'],
  ['seaside', 22, '8e2be8f03cc89e7d'],
  ['mountain', 11, '861f41015c2c4985'],
  ['mountain', 22, 'f859466e339960c9'],
  ['lakeside', 11, '64df52b9edbfc847'],
  ['lakeside', 22, '746c4faa2afec301'],
  ['forest', 11, 'aa808aea1ec3997e'],
  ['forest', 22, '30aa17736a4fbcee'],
  ['dense', 808114082, 'aa74853331500d77'],
  ['small', 199762, '9186dab0bfb29a75'],
  ['forest', 223520, '97cd228b1512578e'],
  ['medium', 104735, '4b8e73cb8e477a0b'],
  ['mountain', 231441, '9cd7a60bf615e77e'],
  ['lakeside', 239360, '93a98109b80d1f7d'],
  ['farm', 271032, 'b1b9aeb91218f190'],
  ['small', 278952, '40204b0c6986c2f0'],
  ['forest', 278953, 'b7260415f246e38d'],
  ['forest', 425059, 'f9a73a8993ff5607'],
  ['lakeside', 215603, '4d436060d55c10f9'],
  ['forest', 320330, '9e081f17322e9436'],
  ['dense', 320329, '7123db8867f199ea'],
  ['seaside', 215602, '1cadf5fe3e9c9d5f'],
  ['forest', 104735, '6e566519b20124c9'],
  ['forest', 112654, '97542cffbea03e00'],
  ['lakeside', 225304, 'b11845113a71b297'],
  ['small', 314192, '1e83cbff5b3e3eb9'],
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
