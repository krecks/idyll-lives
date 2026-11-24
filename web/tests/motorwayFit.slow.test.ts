/**
 * How often the motorway fits (`game/motorway.ts`), laid before the villages, and how many villages the worlds keep
 * beside it: every place at each land size, two worlds of each (the seeds `tools/worlds/map.ts` draws at seed base 0,
 * laid the way a new game lays them), and the classic worlds' seeds 11 and 22. Per place (and per classic kind), at
 * least `FLOOR`'s worlds have one and at least its villages are placed, one fewer than there were (2026-11-24, the
 * motorway before the villages and costing none of them, the lake's villages along its shore: 47 of the 66 worlds,
 * from 31 when it came after them, none among mountains; their villages 185, from 180). Each world with one has nothing wrong
 * with it (`motorwayProblems`) and every crossing on both roads under a deck. `SLOW_TESTS=1 bun test
 * tests/motorwayFit.slow.test.ts`; `bun tools/worlds/map.ts <dir> 2 --classic` draws the same worlds' maps.
 */
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import type { RoadPlan } from '../src/core/protocol';
import { motorwayProblems } from '../src/game/motorway';
import { deckSpans } from '../src/game/roads';

const WEB = new URL('../', import.meta.url).pathname;

/** Per place (a classic kind as `classic <kind>`): the worlds with a motorway at least, and their villages at least. */
const FLOOR: Record<string, [number, number]> = {
  country: [6, 15],
  farm: [6, 9],
  seaside: [4, 10],
  lakeside: [3, 11],
  river: [4, 9],
  mountain: [0, 5],
  forest: [6, 10],
  heath: [6, 8],
  island: [2, 9],
  'classic small': [2, 10],
  'classic medium': [2, 10],
  'classic dense': [2, 12],
  'classic farm': [2, 9],
  'classic seaside': [1, 10],
  'classic mountain': [0, 11],
  'classic lakeside': [0, 11],
  'classic forest': [1, 9],
};

describe('the motorway, before the villages', async () => {
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
  const { generateFromSpec, landDraft, layOn } = await import('../src/game/macro');
  const { CLASSIC, WORLD_KINDS } = await import('../src/game/world');
  type Templates = import('../src/game/town').Templates;
  type Draft = import('../src/game/town').NeighbourhoodDraft;
  const content = await Content.load('http://localhost/content/base.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  /** A world's motorway (whether it has one) and its villages; what is wrong with its motorway. */
  const judged = (d: Draft) => {
    const roads: RoadPlan[] = d.roads;
    const problems = motorwayProblems(roads);
    roads.forEach((w, wi) => {
      if (w.kind !== 'motorway') return;
      for (const c of w.crosses ?? []) {
        if (!deckSpans(w).some(([a, b]) => c.s > a && c.s < b)) problems.push(`${w.name}: crossing at ${c.s.toFixed(0)} m not under a deck`);
        if (!roads[c.road]?.crosses?.some((x) => x.road === wi)) problems.push(`${roads[c.road]?.name} doesn't list ${w.name}`);
      }
    });
    return { fits: roads.some((r) => r.kind === 'motorway'), villages: (d.open?.areas.length ?? 1) - 1, problems };
  };

  /** Each place's worlds: its own, at each size; each classic kind's, seeds 11 and 22. */
  const places: [string, (() => Draft)[]][] = [
    ...WORLD_KINDS.map((kind): [string, (() => Draft)[]] => [
      kind,
      (['small', 'medium', 'large'] as const).flatMap((size) => [0, 1].map((i) => () => layOn(content, t, landDraft(content, t, kind, { size }, (i + 1) * 104729 + kind.length), {}, { waterKept: true }))),
    ]),
    ...Object.keys(CLASSIC).map((kind): [string, (() => Draft)[]] => [`classic ${kind}`, [11, 22].map((seed) => () => generateFromSpec(content, t, CLASSIC[kind as keyof typeof CLASSIC], seed))]),
  ];
  for (const [place, worlds] of places) {
    test(
      `${place}: ${FLOOR[place][0]} worlds with one at least, ${FLOOR[place][1]} villages`,
      () => {
        let [fits, villages] = [0, 0];
        for (const make of worlds) {
          let d: Draft;
          try {
            d = make();
          } catch {
            // (A small lakeside's or river's land with no room for its town: no world, as before.)
            continue;
          }
          const w = judged(d);
          expect(w.problems).toEqual([]);
          fits += w.fits ? 1 : 0;
          villages += w.villages;
        }
        expect(fits).toBeGreaterThanOrEqual(FLOOR[place][0]);
        expect(villages).toBeGreaterThanOrEqual(FLOOR[place][1]);
      },
      300_000,
    );
  }
});
