// The classic worlds (the eight kinds a world was before places and options, `support/classicWorlds.ts`)
// come out byte for byte as they did: each world's draft, as JSON, hashes to what it did when the
// generator read the kinds' own tables (`CHARACTER` and `WORLDS`, 2026-11-15), re-pinned when the town's named
// places became lots of their own (`macro.ts` `namedLots`, 2026-11-17) and when a village's hall
// got a look of its own (`Civic.kinds`, 2026-11-18). Seeds 11 and 22 of
// each kind, and the worlds the pinned tests name (continuedPorts, countryBends, countryCorridors,
// driveways, gardenFences). A change that moves one on purpose re-pins it here.
import { afterAll, describe, expect, test } from 'bun:test';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { CLASSIC, type ClassicKind } from './support/classicWorlds';

const WEB = new URL('../', import.meta.url).pathname;

/** Each world: its kind, its seed, and the first 16 hex digits of its draft's SHA-256. */
const GOLDEN: [ClassicKind, number, string][] = [
  ['small', 11, 'd0e7c82381e75cad'],
  ['small', 22, '46c623224bcdda1b'],
  ['medium', 11, '05809e01803237e9'],
  ['medium', 22, '936ed5916c13f4c4'],
  ['dense', 11, '4501e35be0935355'],
  ['dense', 22, '328f8e4769d1469e'],
  ['farm', 11, '7f99767cbc1b8e70'],
  ['farm', 22, '8df989700c88f710'],
  ['seaside', 11, '6087af459e4e8b9b'],
  ['seaside', 22, 'ba8ce3648cea7b88'],
  ['mountain', 11, '88a64f1f0a43179c'],
  ['mountain', 22, 'c4b009964424f3b8'],
  ['lakeside', 11, '9c614b995c06b97b'],
  ['lakeside', 22, '65f513b223e9fcb2'],
  ['forest', 11, '3ad173cb405e0ec7'],
  ['forest', 22, '638981e1e1bf2292'],
  ['dense', 808114082, '269bbf8fe369ad4e'],
  ['small', 199762, 'a4f3e8e9ab188bd3'],
  ['forest', 223520, '27a47ae7ca4212fe'],
  ['medium', 104735, '51df84f531c65cdb'],
  ['mountain', 231441, '3aca7bce3d3df61f'],
  ['lakeside', 239360, 'fb0af91373e6ade1'],
  ['farm', 271032, '660122595f179933'],
  ['small', 278952, 'f5e4929505d646b4'],
  ['forest', 278953, '2d916ca8293d3c6a'],
  ['forest', 425059, '0fad02a75c7d98c4'],
  ['lakeside', 215603, '5fa8ed17c931e956'],
  ['forest', 320330, '3b07a329b450820c'],
  ['dense', 320329, 'e141e24aebdbbaa6'],
  ['seaside', 215602, 'da215999c1eb36de'],
  ['forest', 104735, '4e48b6d0f840ee86'],
  ['forest', 112654, '539b4ac246683c3a'],
  ['lakeside', 225304, '1c07bbf4df0272f0'],
  ['small', 314192, 'ac263e1fa6b51ad6'],
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
