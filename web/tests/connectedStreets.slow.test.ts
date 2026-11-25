// A town's streets join up (`roads.ts` `connect`): a lane's dead end goes on to the road it can
// reach, two dead ends facing each other become one lane, and every generated world (each kind,
// a few seeds) is one road network as the sim joins its roads (an end within half the
// wider road's width and a metre of the other's middle; sim-core `travel.rs` `Graph::build`),
// its one-way roads cutting no place off (`oneway.ts` `stronglyConnected`; a city's too), with no more courts than its areas may keep, none reaching more than `COURT` m once its land is
// settled under it as a new game's is (`landFollow.ts` `settleLand` cuts a longer one back).
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';
import type { RoadPlan } from '../src/core/protocol';
import { stronglyConnected } from '../src/game/oneway';
import { connect, COURT, lengths, nearest, type Pt } from '../src/game/roads';

const line = (x0: number, z0: number, x1: number, z1: number): Pt[] => {
  const n = Math.ceil(Math.hypot(x1 - x0, z1 - z0));
  return Array.from({ length: n + 1 }, (_, i): Pt => [x0 + ((x1 - x0) * i) / n, z0 + ((z1 - z0) * i) / n]);
};

/** Per road, its network (as the sim joins them), and per road where along it the others' ends meet it. */
function networks(roads: readonly RoadPlan[]): { net: number[]; joins: number[][] } {
  const parent = roads.map((_, i) => i);
  const find = (i: number): number => (parent[i] === i ? i : (parent[i] = find(parent[i])));
  const joins: number[][] = roads.map(() => []);
  roads.forEach((r, i) => {
    for (const end of [r.points[0], r.points[r.points.length - 1]]) {
      roads.forEach((o, j) => {
        const near = j === i ? null : nearest(o.points, end[0], end[1]);
        if (!near || near.dist > Math.max(o.width, r.width) / 2 + 1) return;
        parent[find(i)] = find(j);
        joins[j].push(near.s);
      });
    }
  });
  return { net: roads.map((_, i) => find(i)), joins };
}

describe('connect', () => {
  const box = { w: 300, d: 300 };
  const main: RoadPlan = { points: line(0, 150, 300, 150), width: 7, ends: ['out', 'out'] };

  test("a lane's dead end goes on to the road ahead of it", () => {
    // A lane off the main road going north, and a road across its way 70 m on.
    const across: RoadPlan = { points: line(60, 40, 260, 40), width: 6, ends: ['join', 'join'] };
    const lane: RoadPlan = { points: line(150, 150, 150, 100), width: 6, ends: ['join', 'dead'] };
    const roads = connect([main, across, lane], { box });
    expect(roads).toHaveLength(3);
    const joined = roads[2];
    expect(joined.ends).toEqual(['join', 'join']);
    const end = joined.points[joined.points.length - 1];
    expect(nearest(across.points, end[0], end[1]).dist).toBeLessThan(0.5);
    expect(new Set(networks(roads).net).size).toBe(1);
  });

  test('two dead ends facing each other become one lane', () => {
    const a: RoadPlan = { points: line(100, 150, 100, 90), width: 6, ends: ['join', 'dead'] };
    const b: RoadPlan = { points: line(180, 150, 180, 90), width: 6, ends: ['join', 'dead'] };
    const roads = connect([main, a, b], { box });
    expect(roads).toHaveLength(2);
    expect(roads[1].ends).toEqual(['join', 'join']);
    // From one lane's foot round to the other's, never crossing the main road.
    expect(roads[1].points.every(([, z]) => z <= 150.01)).toBe(true);
  });

  test("a lane with nowhere to go is a court, cut to the court's length; past the courts allowed it goes", () => {
    const lanes: RoadPlan[] = [40, 140, 240].map((x) => ({ points: line(x, 150, x, 230), width: 6, ends: ['join', 'dead'] }));
    // Nothing to reach: kept inside a box with no room for a way round.
    const roads = connect([main, ...lanes], { box: { w: 300, d: 250 }, margin: 14 }, 2);
    const courts = roads.filter((r) => r.ends.includes('dead'));
    expect(courts.length).toBeLessThanOrEqual(2);
    for (const r of courts) expect(lengths(r.points).at(-1)!).toBeLessThanOrEqual(COURT + 0.5);
  });
});

describe('generated worlds are one network', async () => {
  usePublicFetch();
  const { BIG_COURTS, COURTS, generateWorld, useTownContent } = await import('../src/game/town');
  const { WORLD_KINDS } = await import('../src/game/world');
  const { settleLand } = await import('../src/game/landFollow');
  const { previewWorld } = await import('../src/game/townPreview');
  const content = await loadContent();
  useTownContent(content);
  const t = await templatesOf(content);

  /** A world of `kind` (its place's own options but `town`) as a new game makes it, its land settled under it, checked. */
  const check = (kind: string, seed: number, town?: 'city') => {
    const draft = settleLand(generateWorld(content, t, kind, seed, undefined, undefined, town ? { town } : undefined), (d) => previewWorld(content, t, d, null)).draft;
    const tag = `${kind} ${town ?? ''} ${seed}`;
    const { net, joins } = networks(draft.roads);
    expect(new Set(net).size, `${tag}: networks`).toBe(1);
    // (Its one-way roads, a town's and a city's, cutting no place off: `oneway.ts`.)
    expect(stronglyConnected(draft.roads, true), `${tag}: one-way roads cut a place off`).toBe(true);
    expect(draft.diagnostics?.roads ?? [], `${tag}: country roads with no way`).toEqual([]);
    return { town: draft, tag, joins };
  };

  test(
    'big worlds',
    () => {
      for (const kind of WORLD_KINDS) {
        for (const seed of [7, 3141, 271828]) {
          const { town, tag, joins } = check(kind, seed);
          const courts = new Map<number, number>();
          town.roads.forEach((r, i) => {
            if (r.kind === 'trail') return;
            const L = lengths(r.points).at(-1)!;
            for (const e of [0, 1] as const) {
              if (r.ends[e] !== 'dead') continue;
              courts.set(r.area ?? 0, (courts.get(r.area ?? 0) ?? 0) + 1);
              const stub = L - Math.max(0, ...joins[i].map((s) => (e === 1 ? s : L - s)));
              expect(stub, `${tag}: ${r.name}'s court`).toBeLessThanOrEqual(COURT + 0.5);
            }
          });
          for (const [area, n] of courts) expect(n, `${tag}: courts in area ${area}`).toBeLessThanOrEqual(area === 0 ? BIG_COURTS : COURTS);
        }
      }
    },
    240_000,
  );

  test(
    'cities: their avenue and their one-way pairs joined up',
    () => {
      for (const kind of WORLD_KINDS) check(kind, 7, 'city');
    },
    240_000,
  );
});
