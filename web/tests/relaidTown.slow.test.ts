// "Lay the town again" (`macro.ts` `generateFromSpec`'s `shaped`, step 1): a lake painted over a world's main
// town, the town laid again on that land, gives one road network as the sim joins its roads, no lot or road
// in the water, no bridge over a river that is no longer the place's, the areas on the one piece of dry land,
// and the world keeps its name and its main town's shape; the draft carries the land (graded flat under the
// town) and its water and zones come from it. Unshaped worlds are as they were (the classic and world-option
// goldens). `SLOW_TESTS=1 bun test tests/relaidTown.slow.test.ts`.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import type { RoadPlan } from '../src/core/protocol';
import { nearest } from '../src/game/roads';

const WEB = new URL('../', import.meta.url).pathname;

/** Per road, its network, as the sim joins them (an end within half the wider road's width and a metre of the other's middle). */
function networks(roads: readonly RoadPlan[]): number {
  const parent = roads.map((_, i) => i);
  const find = (i: number): number => (parent[i] === i ? i : (parent[i] = find(parent[i])));
  roads.forEach((r, i) => {
    for (const end of [r.points[0], r.points[r.points.length - 1]]) {
      roads.forEach((o, j) => {
        if (j !== i && nearest(o.points, end[0], end[1]).dist <= Math.max(o.width, r.width) / 2 + 1) parent[find(i)] = find(j);
      });
    }
  });
  return new Set(roads.map((_, i) => find(i))).size;
}

describe('the town laid again on a shaped land', async () => {
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
  const { generateWorld, plotRect, useTownContent } = await import('../src/game/town');
  const { LandEditor, surveyDraft, wetTile } = await import('../src/game/landEdit');
  const { unrle } = await import('../src/game/landMasks');
  const { LAND_CELL } = await import('../src/game/land');
  type Templates = import('../src/game/town').Templates;
  type WorldOptions = import('../src/game/world').WorldOptions;
  const content = await Content.load('http://localhost/content/base.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  const WORLDS: [string, number, Partial<WorldOptions>][] = [
    ['country', 4242, { town: 'village', villages: 'few' }],
    ['river', 7, { town: 'village', villages: 'few' }],
  ];

  for (const [kind, seed, options] of WORLDS) {
    test(
      `${kind} ${seed}: a lake over the town, the town laid again`,
      () => {
        const draft = generateWorld(content, t, kind, seed, undefined, undefined, options);
        const land = surveyDraft(draft);
        const editor = new LandEditor(land, () => 1);
        // A lake 120 m across over the main town's middle.
        const a = draft.open!.areas[0];
        const [cx, cz] = [a.x + a.w / 2, a.z + a.d / 2];
        editor.begin({ tool: 'lake', radius: 60, strength: 0.6 }, cx, cz);
        editor.end();
        expect(wetTile(land, Math.floor(cx), Math.floor(cz))).toBe(true);
        expect(land.edited).toBe(true);

        const relaid = generateWorld(content, t, kind, seed, undefined, undefined, options, land);
        const tag = `${kind} ${seed}`;
        // Its name, its size, its main town's shape (as many streets), a town.
        expect(relaid.name).toBe(draft.name);
        expect(relaid.width).toBe(land.cols * LAND_CELL);
        expect(relaid.roads.filter((r) => r.area === 0 && r.kind !== 'trail' && r.kind !== 'country').length, tag).toBe(draft.roads.filter((r) => r.area === 0 && r.kind !== 'trail' && r.kind !== 'country').length);
        expect(relaid.slots.length).toBeGreaterThan(draft.slots.length * 0.7);
        // One road network.
        expect(networks(relaid.roads), `${tag}: networks`).toBe(1);
        // No bridge (the river is the land's now, and no bridge is laid over it).
        expect(relaid.roads.some((r) => r.bridge && r.bridge[1] > r.bridge[0]), `${tag}: bridges`).toBe(false);
        // No lot, and no road, in the water; the sim's water is the land's.
        const shaped = relaid.open!.land!;
        expect(shaped.edited).toBe(true);
        const W = relaid.width;
        const water = unrle(relaid.open!.water, W * relaid.depth);
        for (const s of relaid.slots) {
          const r = plotRect(t, s);
          for (let z = r.z; z < r.z + r.d; z++) for (let x = r.x; x < r.x + r.w; x++) expect(water[z * W + x], `${tag}: ${s.name} in the water`).toBe(0);
        }
        for (const r of relaid.roads) for (const [x, z] of r.points) if (x >= 0 && z >= 0 && x < W && z < relaid.depth) expect(water[Math.floor(z) * W + Math.floor(x)], `${tag}: ${r.name} in the water`).toBe(0);
        let wet = 0;
        for (let z = 0; z < relaid.depth; z += 4) for (let x = 0; x < W; x += 4) if (wetTile(shaped, x, z) !== !!water[z * W + x]) wet++;
        expect(wet).toBe(0);
        // The lake is a water zone; the land is flat under the lots.
        expect(relaid.open!.zones.some((z) => z.kind === 'water')).toBe(true);
        for (const s of relaid.slots.slice(0, 20)) {
          const r = plotRect(t, s);
          const k = Math.round((r.z + r.d / 2) / LAND_CELL) * shaped.cols + Math.round((r.x + r.w / 2) / LAND_CELL);
          expect(shaped.h[k]).toBe(0);
        }
        // The player's land is not changed by it.
        expect(land.h.some((h, k) => h !== shaped.h[k])).toBe(true);
      },
      180_000,
    );
  }
});
