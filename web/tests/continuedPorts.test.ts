// No country road leaves a street's end that another area's street comes up to (`macro.ts`
// `continued`): the two meet end to end, one street on through, and a country road from there ran
// on along the other street a few metres off before turning away across it, a junction knotted
// into two a few metres apart (Canalside's Queen'ston Lane, from Foundry Street's end by Union
// Street, the dense world of the play-test, meta.seed 808114082).
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import { nearest } from '../src/game/roads';

const WEB = new URL('../', import.meta.url).pathname;

describe('country roads by streets meeting end to end', async () => {
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
  type Templates = import('../src/game/town').Templates;
  const content = await Content.load('http://localhost/content/base.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  test(
    'big dense 808114082 (Canalside): the two streets meet, and no country road leaves where they do',
    () => {
      const town = generateWorld(content, t, 'dense', 808114082);
      expect(town.name).toBe('Canalside');
      const streets = town.roads.filter((r) => r.kind !== 'country' && r.kind !== 'trail' && r.area !== undefined);
      // (Each pair of other areas' streets whose ends meet: no country road ends there.)
      const met: [number, number][] = [];
      for (const a of streets) {
        for (const b of streets) {
          if (a === b || a.area === b.area) continue;
          for (const p of [a.points[0], a.points[a.points.length - 1]]) if (nearest(b.points, p[0], p[1]).dist <= Math.max(a.width, b.width) / 2 + 1) met.push(p);
        }
      }
      expect(met.some(([x, z]) => Math.hypot(x - 653, z - 227) < 2)).toBe(true);
      const country = town.roads.filter((r) => r.kind === 'country');
      const at = country.flatMap((r) => [r.points[0], r.points[r.points.length - 1]].filter(([x, z]) => met.some(([mx, mz]) => Math.hypot(x - mx, z - mz) < 5)).map(() => r.name));
      expect(at).toEqual([]);
    },
    60_000,
  );
});
