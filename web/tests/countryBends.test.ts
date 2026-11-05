// No country road turns further than `COUNTRY_TURN` (70°) over any 40 m of it (`roads.ts`
// `sharpestTurn`, two spans of 20 m: the sampler's check), at its ends too: one that comes into a
// way (a street's end, a T) at a slant takes a longer straight lead into it, so the bend comes
// further out where it has room (`macro.ts` `countryRoad`), and one relaxed to keep within it is
// measured as filed, to the centimetre (`unkink`). From the big worlds whose roads turned 73-74°
// just before the street they came into: Elder Green (seed base 12) and Hollyshaw (seed base 15).
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import { sharpestTurn } from '../src/game/roads';

const WEB = new URL('../', import.meta.url).pathname;

describe('country road bends', async () => {
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
  const { generateWorld, useTownContent } = await import('../src/game/town');
  const { COUNTRY_TURN } = await import('../src/game/macro');
  type Templates = import('../src/game/town').Templates;
  const content = await Content.load('http://localhost/content/base.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };
  const deg = (a: number) => Math.round((a * 180) / Math.PI);

  for (const [kind, seed, name, road] of [
    ['small', 199762, 'Elder Green', 'Primrose End Road'],
    ['forest', 223520, 'Hollyshaw', 'Bluebellhurst Lane'],
  ] as const) {
    test(
      `big ${kind} ${seed} (${name}): no country road turns further than ${deg(COUNTRY_TURN)}° over 40 m (${road} did)`,
      () => {
        const town = generateWorld(content, t, kind, seed, undefined, 'big');
        expect(town.name).toBe(name);
        expect(town.roads.some((r) => r.name === road)).toBe(true);
        const sharp = town.roads.filter((r) => r.kind === 'country').flatMap((r) => {
          const { angle } = sharpestTurn(r.points, 20);
          return angle > COUNTRY_TURN + 1e-3 ? [`${r.name} turns ${deg(angle)}°`] : [];
        });
        expect(sharp).toEqual([]);
      },
      60_000,
    );
  }
});
