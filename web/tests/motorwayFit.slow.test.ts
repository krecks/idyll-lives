/**
 * How often the motorway fits (`game/motorway.ts`), laid before the villages, and how many villages the worlds keep
 * beside it: every place at each land size, two worlds of each (the seeds `tools/worlds/map.ts` draws at seed base 0,
 * laid the way a new game lays them), and the classic worlds' seeds 11 and 22. Per place (and per classic kind), at
 * least `FLOOR`'s worlds have one and at least its villages are placed, one fewer than there were (2026-11-24, the
 * motorway before the villages and costing none of them, the lake's villages along its shore: 47 of the 66 worlds,
 * from 31 when it came after them, none among mountains; their villages 185, from 180). Each world with one has nothing wrong
 * with it (`motorwayProblems`) and every crossing on both roads under a deck. `bun test tests/motorwayFit.slow.test.ts` by hand; the slow set is `bun run test:slow` in web/ (`SLOW_TESTS=1 tools/ci-local.sh web`; CI on v* tags and by hand); `bun tools/worlds/map.ts <dir> 2 --classic` draws the same worlds' maps.
 */
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';
import type { RoadPlan } from '../src/core/protocol';
import { motorwayProblems } from '../src/game/motorway';
import { deckSpans } from '../src/game/roads';

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
  usePublicFetch();
  const { useTownContent } = await import('../src/game/town');
  const { generateFromSpec, landDraft, layOn } = await import('../src/game/macro');
  const { CLASSIC, WORLD_KINDS } = await import('../src/game/world');
  type Draft = import('../src/game/town').NeighbourhoodDraft;
  const content = await loadContent();
  useTownContent(content);
  const t = await templatesOf(content);

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
          } catch (e) {
            // (A small lakeside's or river's land with no room for its town: no world, as before. Anything else fails.)
            if (!String((e as Error)?.message).startsWith('There is no dry land big enough for the town')) throw e;
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
