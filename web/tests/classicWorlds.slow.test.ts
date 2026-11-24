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
// not in their diagnostics), and for traffic on the right as the game shows it (2026-11-24: each motorway's
// carriageways on the other sides of its middle line, its ramps with them). A change that moves one on purpose
// re-pins it here.
import { afterAll, describe, expect, test } from 'bun:test';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { CLASSIC, type ClassicKind } from './support/classicWorlds';

const WEB = new URL('../', import.meta.url).pathname;

/** Each world: its kind, its seed, and the first 16 hex digits of its draft's SHA-256. */
const GOLDEN: [ClassicKind, number, string][] = [
  ['small', 11, '93df4323760d2619'],
  ['small', 22, '2ed86794329ed91f'],
  ['medium', 11, 'b83a866c67c569e5'],
  ['medium', 22, '3bf4c8ed4309a2b2'],
  ['dense', 11, '22c0fe2ea106fa6c'],
  ['dense', 22, '55f04793ce33813d'],
  ['farm', 11, '9eb491d37063f6b6'],
  ['farm', 22, '5a7abad0018483cb'],
  ['seaside', 11, '7864295262d20929'],
  ['seaside', 22, 'd02c6338ad3f1fec'],
  ['mountain', 11, '16b8bd99ff146eb6'],
  ['mountain', 22, '43de42b0f3edb49f'],
  ['lakeside', 11, '5a21d6efeeb2e338'],
  ['lakeside', 22, '56d2f4ce778e13ef'],
  ['forest', 11, '76a8ebe6e0940013'],
  ['forest', 22, 'e3d0a96c266c18a3'],
  ['dense', 808114082, '12805b57a1b311ee'],
  ['small', 199762, 'ff2bd176035ed1f0'],
  ['forest', 223520, 'd20689f685de74e7'],
  ['medium', 104735, '92967efb1e0a8dfe'],
  ['mountain', 231441, '37220e054cad2ece'],
  ['lakeside', 239360, 'bcd3aeeaf364b180'],
  ['farm', 271032, 'f7581f2346a18310'],
  ['small', 278952, '764ce1233cc43de8'],
  ['forest', 278953, 'f1c1b2f67ee5cf5c'],
  ['forest', 425059, 'caf8a24d6c71e6fc'],
  ['lakeside', 215603, '94e70241f1ebe00f'],
  ['forest', 320330, '31f5d72234db5f2f'],
  ['dense', 320329, '9a65de94c0f54f6b'],
  ['seaside', 215602, '360b2019c057c708'],
  ['forest', 104735, '253f9bd018698ad3'],
  ['forest', 112654, '77733808b2760704'],
  ['lakeside', 225304, '1930a40a8ad7dfc3'],
  ['small', 314192, 'aee40bd123c11592'],
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
