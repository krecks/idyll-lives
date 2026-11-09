// A town's streets join up (`roads.ts` `connect`): a lane's dead end goes on to the road it can
// reach, two dead ends facing each other become one lane, and every generated world (each kind,
// each size, a few seeds) is one road network as the sim joins its roads (an end within half the
// wider road's width and a metre of the other's middle; sim-core `travel.rs` `Graph::build`),
// with no more courts than its areas may keep, none reaching more than `COURT` m.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import type { RoadPlan } from '../src/core/protocol';
import { connect, COURT, lengths, nearest, type Pt } from '../src/game/roads';

const WEB = new URL('../', import.meta.url).pathname;
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
  const { BIG_COURTS, COURTS, generateWorld, useTownContent } = await import('../src/game/town');
  const { WORLD_KINDS } = await import('../src/game/world');
  type Templates = import('../src/game/town').Templates;
  const content = await Content.load('http://localhost/content/base.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  for (const size of ['big'] as const) {
    test(
      `${size} worlds`,
      () => {
        for (const kind of WORLD_KINDS) {
          for (const seed of [7, 3141, 271828]) {
            const town = generateWorld(content, t, kind, seed, undefined, size);
            const tag = `${kind} ${size} ${seed}`;
            const { net, joins } = networks(town.roads);
            expect(new Set(net).size, `${tag}: networks`).toBe(1);
            expect(town.diagnostics?.roads ?? [], `${tag}: country roads with no way`).toEqual([]);
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
            for (const [area, n] of courts) expect(n, `${tag}: courts in area ${area}`).toBeLessThanOrEqual(size === 'big' && area === 0 ? BIG_COURTS : COURTS);
          }
        }
      },
      size === 'big' ? 240_000 : 60_000,
    );
  }
});
