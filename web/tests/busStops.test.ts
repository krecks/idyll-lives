// A cosy or normal town as `assembleTown` makes it has a bus stop or two (sim-core `bus.rs`): ways
// out of town (`kind: 'stop'`) at the kerb of its main road near its middle, on one side of it,
// with room behind the pavement for a shelter clear of the lots; its roads out are ways out too.
// A big world's stops are its areas' (`macro.ts`), as they were.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('bus stops in new towns', async () => {
  // --- the page's world in bun: content from web/public (put back for whatever runs next) ---
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
  const { AssetRegistry } = await import('../src/assets/registry');
  const { objectInfos } = await import('../src/game/housegen');
  const { randomHousehold } = await import('../src/game/household');
  const { assembleTown, generateWorld, populate, useTownContent, vacantSlots } = await import('../src/game/town');
  const { nearest, pointAt, roadIndex } = await import('../src/game/roads');
  type Templates = import('../src/game/town').Templates;
  type RoadPlan = import('../src/core/protocol').RoadPlan;
  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  type Exit = [number, number] | { x: number; z: number; kind: 'road' | 'stop' };
  type Plot = { x: number; z: number; w: number; d: number };

  const towns = [
    { kind: 'medium', seed: 7, size: 'cosy' },
    { kind: 'small', seed: 4242, size: 'cosy' },
    { kind: 'farm', seed: 3141, size: 'normal' },
    { kind: 'seaside', seed: 271828, size: 'normal' },
  ] as const;

  for (const w of towns) {
    test(
      `${w.kind} ${w.size} ${w.seed}: a stop or two on the main road near the middle`,
      () => {
        const town = populate(content, assets, t, generateWorld(content, t, w.kind, w.seed, undefined, w.size));
        const file = JSON.parse(assembleTown(content, t, town, randomHousehold(content, assets, 2), vacantSlots(town)[0].index, {}, () => 0.5)) as { width: number; depth: number; exits: Exit[]; roads: RoadPlan[]; plots: Plot[] };
        const typed = file.exits.map((e) => (Array.isArray(e) ? { x: e[0], z: e[1], kind: 'road' as const } : e));
        const stops = typed.filter((e) => e.kind === 'stop');
        expect(stops.length).toBeGreaterThanOrEqual(1);
        expect(stops.length).toBeLessThanOrEqual(2);
        // The roads out are there too.
        expect(typed.filter((e) => e.kind === 'road').length).toBeGreaterThanOrEqual(1);
        const index = roadIndex(file.roads);
        const sides = new Set<number>();
        for (const s of stops) {
          // On the pavement of a road a bus drives, by its kerb.
          const on = index.nearest(s.x, s.z, 8);
          expect(on).not.toBeNull();
          const road = file.roads[on!.road];
          expect(road.kind ?? 'street').not.toBe('trail');
          expect(on!.dist).toBeGreaterThan(road.width / 2 - 1.35);
          expect(on!.dist).toBeLessThan(road.width / 2);
          sides.add(on!.road * 10 + on!.side);
          // Nearer the town's middle than its edges.
          const [mx, mz] = [file.width / 2, file.depth / 2];
          expect(Math.hypot(s.x - mx, s.z - mz)).toBeLessThan(Math.hypot(file.width, file.depth) / 2.5);
          // Room behind the pavement for the shelter: off every lot.
          const n = nearest(road.points, s.x, s.z);
          const at = pointAt(road.points, n.s);
          const [ox, oz] = [(s.x - at.p[0]) / n.dist, (s.z - at.p[1]) / n.dist];
          const out = road.width / 2 + 0.7;
          for (const [a, b] of [[-1.5, -0.6], [1.5, -0.6], [-1.5, 0.6], [1.5, 0.6], [0, 0]]) {
            const [x, z] = [at.p[0] + ox * (out + b) + at.t[0] * a, at.p[1] + oz * (out + b) + at.t[1] * a];
            for (const p of file.plots) expect(x > p.x && x < p.x + p.w && z > p.z && z < p.z + p.d).toBe(false);
          }
        }
        // Both on one side of one road: the bus passes them the same way.
        expect(sides.size).toBe(1);
      },
      60_000,
    );
  }

  test('a big world keeps its areas\' stops', () => {
    const town = populate(content, assets, t, generateWorld(content, t, 'forest', 99, undefined, 'big'));
    const file = JSON.parse(assembleTown(content, t, town, randomHousehold(content, assets, 2), vacantSlots(town)[0].index, {}, () => 0.5)) as { exits: Exit[] };
    const stops = file.exits.filter((e) => !Array.isArray(e) && e.kind === 'stop');
    expect(stops.length).toBe(town.open!.stops.length);
  }, 120_000);
});
