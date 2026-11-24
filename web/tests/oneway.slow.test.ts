// A town's and a city's one-way pairs and a city's avenue (`game/oneway.ts`, `world.ts` `mainTown`), over a sweep of
// generated worlds, every place at each main town size: every lot can be driven to from the way out of town and back
// (`cutOffRoads`, the town's check `townCheck.ts` alike); each one-way road a street of the main town, both its
// ends joined, none where a country road goes on (a way out of town), and no more than the size's pairs; none in a
// hamlet's or a village's main town. A city whose shape has a through road has it as an avenue, 14 m wide, out of
// town (on to the country roads; on an island, one way out may end short); no other place has one.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import type { RoadPlan } from '../src/core/protocol';
import { isSeparated } from '../src/core/roadKinds';
import { cutOffRoads } from '../src/game/oneway';

const WEB = new URL('../', import.meta.url).pathname;
const SEEDS = [7, 3141, 271828];

describe('one-way pairs and avenues in generated worlds', async () => {
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
  const { WORLD_KINDS } = await import('../src/game/world');
  type Templates = import('../src/game/town').Templates;
  const content = await Content.load('http://localhost/content/base.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  /** Whether road `i`'s end `e` is where another road (a country road) goes on from it, end to end. */
  const goesOn = (roads: readonly RoadPlan[], i: number, e: 0 | 1) => {
    const p = e === 0 ? roads[i].points[0] : roads[i].points[roads[i].points.length - 1];
    return roads.some((o, j) => j !== i && o.kind === 'country' && [o.points[0], o.points[o.points.length - 1]].some((q) => Math.hypot(q[0] - p[0], q[1] - p[1]) < 1));
  };

  let pairs = 0;
  for (const kind of WORLD_KINDS) {
    test(
      kind,
      () => {
        for (const size of ['village', 'town', 'city'] as const) {
          for (const seed of size === 'village' ? SEEDS.slice(0, 1) : SEEDS) {
            const town = generateWorld(content, t, kind, seed, undefined, undefined, { town: size });
            const tag = `${kind} ${size} ${seed}`;
            const roads = town.roads;
            // Every lot driven to and back: no car road cut off, by the roads alone (as the simulation routes a trip
            // in town) nor from the world beyond the ways out.
            expect(cutOffRoads(roads, false, true), `${tag}: cut off in town`).toEqual([]);
            expect(cutOffRoads(roads), `${tag}: cut off`).toEqual([]);
            // (The motorway's carriageways and ramps are one-way too: theirs are the motorway's own tests, `motorway.test.ts` and `motorwayFit.slow.test.ts`.)
            const ways = roads.flatMap((r, i) => (r.oneway && !isSeparated(r.kind) ? [i] : []));
            if (size === 'village') expect(ways, `${tag}: one-way roads`).toEqual([]);
            expect(ways.length, `${tag}: one-way roads`).toBeLessThanOrEqual(size === 'city' ? 6 : size === 'town' ? 4 : 0);
            pairs += ways.length / 2;
            for (const i of ways) {
              const r = roads[i];
              expect(r.kind, `${tag}: ${r.name}`).toBe('street');
              expect(r.area, `${tag}: ${r.name}`).toBe(0);
              expect(r.ends, `${tag}: ${r.name}`).toEqual(['join', 'join']);
              expect(goesOn(roads, i, 0) || goesOn(roads, i, 1), `${tag}: ${r.name} a way out of town`).toBe(false);
            }
            // A city's avenue: its main town's, 14 m wide, out of town (or into the country road that goes on from
            // it); a forest's clearing has no through road.
            const avenues = roads.flatMap((r, i) => (r.kind === 'avenue' ? [i] : []));
            if (size !== 'city' || kind === 'forest') {
              expect(avenues, `${tag}: avenues`).toEqual([]);
              continue;
            }
            expect(avenues.length, `${tag}: avenues`).toBe(1);
            const av = roads[avenues[0]];
            expect(av.area, tag).toBe(0);
            expect(av.width, tag).toBe(14);
            // (On an island a way out with nowhere to go ends short: a dead end.)
            for (const e of [0, 1] as const) expect(av.ends[e] === 'out' || av.ends[e] === 'dead' || goesOn(roads, avenues[0], e), `${tag}: the avenue's end ${e}`).toBe(true);
            expect([0, 1].some((e) => av.ends[e] === 'out' || goesOn(roads, avenues[0], e as 0 | 1)), `${tag}: the avenue leads out of town`).toBe(true);
          }
        }
      },
      600_000,
    );
  }

  test('one-way pairs are laid at all', () => {
    // (Over the sweep: most towns and cities have a few.)
    expect(pairs).toBeGreaterThan(WORLD_KINDS.length * SEEDS.length);
  });
});
