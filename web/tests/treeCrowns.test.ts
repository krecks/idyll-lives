// Trees keep their crowns out of the houses (content `crown`): a town's gardens plant none whose
// crown would grow into a house wall, its own lot's or a neighbour's (`town.ts` `dropCrowdedTrees`,
// `housegen.ts` `crownClear`), as the game refuses to plant one there (sim-core `home.rs`).
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('tree crowns', async () => {
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
  const { crownClear, objectInfos } = await import('../src/game/housegen');
  const { generateWorld, layoutTown, useTownContent } = await import('../src/game/town');
  type Templates = import('../src/game/town').Templates;

  const content = await Content.load('http://localhost/content/base.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  test('the trees have crowns', () => {
    for (const id of ['tree', 'pine', 'garden.maple', 'garden.spruce']) expect(content.object(id)?.crown ?? 0, id).toBeGreaterThan(1);
  });

  test('crownClear measures to the nearest wall edge', () => {
    const g = { edges: new Map([['v:10:5', 'wall']]) };
    expect(crownClear(g, 7.5, 5.5, 3)).toBe(false);
    expect(crownClear(g, 7, 5.5, 3)).toBe(true);
    expect(crownClear(g, 10.5, 1.5, 3)).toBe(true);
    expect(crownClear(g, 10.5, 3.5, 3)).toBe(false);
  });

  test("no town tree's crown grows into a house", () => {
    let trees = 0;
    for (const kind of ['small', 'medium', 'dense', 'farm', 'seaside', 'mountain', 'lakeside', 'forest'] as const) {
      for (const size of ['cosy', 'normal'] as const) {
        for (const seed of [11, 4242, 90210]) {
          const town = generateWorld(content, t, kind, seed, undefined, size);
          const layout = layoutTown(content, t, town, null);
          const edges = new Map<string, string>();
          for (const [x0, z0, x1, z1] of layout.walls) {
            if (Math.max(z0, z1) > town.depth) continue;
            if (z0 === z1) for (let x = Math.min(x0, x1); x < Math.max(x0, x1); x++) edges.set(`h:${x}:${z0}`, 'wall');
            else for (let z = Math.min(z0, z1); z < Math.max(z0, z1); z++) edges.set(`v:${x0}:${z}`, 'wall');
          }
          for (const o of layout.objects) {
            const def = content.object(o.def);
            const crown = def?.crown ?? 0;
            if (crown <= 0 || o.z >= town.depth) continue;
            trees++;
            const [fw, fd] = def?.footprint ?? [1, 1];
            const [w, d] = (o.rot ?? 0) % 2 ? [fd, fw] : [fw, fd];
            expect(crownClear({ edges }, o.x + w / 2, o.z + d / 2, crown), `${kind} ${size} ${seed}: ${o.def} at ${o.x}, ${o.z}`).toBe(true);
          }
        }
      }
    }
    // Gardens still have trees.
    expect(trees).toBeGreaterThan(100);
  }, 300_000);
});
