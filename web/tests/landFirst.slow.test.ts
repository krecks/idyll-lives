// The land first, then the town on it (the `land` and `town` jobs, `townJobs.ts`; `macro.ts` `landDraft`, `layOn`):
// a town laid on every place's bare land, one road network with no lot in the water, the land's plan and name
// carried; a new world (`generateWorld`) is that land and that town; a river kept as planned bridged, one
// painted over not; the land as step 1 left it, and its level, kept away from the town (no jump in height
// between the steps); and a land the player shaped still taking a town, its town file carrying that land. `SLOW_TESTS=1 bun test tests/landFirst.slow.test.ts`.
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

describe('the land first, then the town', async () => {
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
  const { assembleTown, generateWorld, plotRect, populate, useTownContent, vacantSlots } = await import('../src/game/town');
  const { AssetRegistry } = await import('../src/assets/registry');
  const { randomHousehold } = await import('../src/game/household');
  const { runTownJob } = await import('../src/game/townJobs');
  const { LandEditor, builtOf, Protection, wetTile } = await import('../src/game/landEdit');
  const { unrle } = await import('../src/game/landMasks');
  const { decodeLand, LAND_CELL, landHash } = await import('../src/game/land');
  const { WORLD_KINDS } = await import('../src/game/world');
  type Templates = import('../src/game/town').Templates;
  type NeighbourhoodDraft = import('../src/game/town').NeighbourhoodDraft;
  type TownResult = import('../src/game/townJobs').TownResult;
  type WorldOptions = import('../src/game/world').WorldOptions;
  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  const landOf = (world: string, seed: number, options?: Partial<WorldOptions>): NeighbourhoodDraft => {
    const made = runTownJob(content, t, { kind: 'land', world, seed, ...(options ? { options } : {}) });
    if (made.kind !== 'land') throw new Error('not a land');
    return made.draft;
  };
  const townOn = (land: NeighbourhoodDraft, more: { seed?: number; waterKept?: boolean; site?: [number, number] } = {}): Extract<TownResult, { kind: 'town' }> => {
    const made = runTownJob(content, t, { kind: 'town', land, seed: more.seed ?? land.seed!, options: {}, waterKept: more.waterKept ?? true, ...(more.site ? { site: more.site } : {}) });
    if (made.kind !== 'town') throw new Error('not a town');
    return made;
  };
  /** What is wrong with a town as the sim would take it: more than one road network, a lot in the water. */
  const faults = (draft: NeighbourhoodDraft): string[] => {
    const out: string[] = [];
    if (networks(draft.roads.filter((r) => r.kind !== 'trail')) !== 1) out.push('more than one road network');
    const W = draft.width;
    const water = unrle(draft.open!.water, W * draft.depth);
    for (const s of draft.slots) {
      const r = plotRect(t, s);
      for (let z = r.z; z < r.z + r.d; z += 2) for (let x = r.x; x < r.x + r.w; x += 2) if (water[z * W + x]) return [...out, `${s.name} in the water`];
    }
    return out;
  };

  test('a town on the bare land of every place', () => {
    for (const kind of WORLD_KINDS) {
      const land = landOf(kind, 31337);
      const { draft } = townOn(land);
      const tag = `${kind}: ${draft.name}`;
      expect(faults(draft), tag).toEqual([]);
      expect(draft.open!.areas.length, tag).toBeGreaterThan(1);
      expect(draft.slots.filter((s) => s.kind === 'house').length, tag).toBeGreaterThan(20);
      // The land's name, size and plan; its land not shaped by anyone; the town on its site.
      expect(draft.name).toBe(land.name);
      expect(draft.width).toBe(land.width);
      expect(JSON.stringify(draft.open!.plan)).toBe(JSON.stringify(land.open!.plan));
      expect(draft.open!.land!.edited).toBe(false);
      expect(draft.world?.townSeed).toBeUndefined();
      const [sx, sz] = draft.open!.site!;
      const plan = land.open!.plan!;
      expect(Math.abs(sx - plan.site[0]), tag).toBeLessThan(40);
      expect(Math.abs(sz - plan.site[1]), tag).toBeLessThan(140);
    }
  }, 600_000);

  test('a new world is its land and its town laid on it', () => {
    const options: Partial<WorldOptions> = { town: 'village', villages: 'few' };
    const land = landOf('heath', 4242, options);
    const laid = runTownJob(content, t, { kind: 'town', land, seed: 4242, options, waterKept: true });
    const world = runTownJob(content, t, { kind: 'world', world: 'heath', seed: 4242, options });
    if (laid.kind !== 'town' || world.kind !== 'world') throw new Error('not a town');
    expect(JSON.stringify(world.draft)).toBe(JSON.stringify(laid.draft));
    expect(generateWorld(content, t, 'heath', 4242, undefined, undefined, options).slots.length).toBeGreaterThanOrEqual(laid.draft.slots.length);
    // Another town on the same land: another seed, the land's own kept where nothing is built.
    const other = townOn(land, { seed: 99 }).draft;
    expect(other.world?.townSeed).toBe(99);
    expect(JSON.stringify(other.slots)).not.toBe(JSON.stringify(laid.draft.slots));
  }, 300_000);

  test('a river kept as planned is bridged; one painted over is not', () => {
    const land = landOf('river', 7, { town: 'village', villages: 'few' });
    const kept = townOn(land).draft;
    const decks = (d: NeighbourhoodDraft) => d.roads.filter((r) => r.bridge && r.bridge[1] > r.bridge[0]);
    expect(decks(kept).length).toBeGreaterThan(0);
    expect(faults(kept)).toEqual([]);
    expect(kept.diagnostics?.roads ?? []).toEqual([]);
    // (The town's own bridge, from its waterfront street.)
    expect(decks(kept).some((r) => r.name === `${kept.name} Bridge`)).toBe(true);
    // Its water changed (not as planned): no bridge, the town on the largest piece of dry land.
    const changed = townOn(land, { waterKept: false }).draft;
    expect(decks(changed)).toEqual([]);
    expect(faults(changed)).toEqual([]);
  }, 300_000);

  test('the land kept as step 1 left it, and its level: no jump in height between the steps', () => {
    for (const [kind, options] of [
      ['heath', { hills: 'hilly' }],
      ['seaside', {}],
      ['mountain', {}],
    ] as const) {
      const land = landOf(kind, 2024, options);
      const { draft, preview } = townOn(land);
      const [bare, built] = [land.open!.land!, draft.open!.land!];
      const { cols, rows } = bare;
      const dist = new Protection(cols, rows, builtOf(preview)).dist;
      // Far from what is built (past where the land eases under the town), the land as it was.
      let far = 0;
      let moved = 0;
      const under: number[] = [];
      for (let k = 0; k < cols * rows; k++) {
        if (dist[k] > 200) {
          far++;
          if (bare.h[k] !== built.h[k]) moved++;
        }
        if (dist[k] === 0) under.push((built.h[k] - bare.h[k]) / 100);
      }
      expect(far, kind).toBeGreaterThan(1000);
      expect(moved, `${kind}: samples moved far from the town`).toBe(0);
      // Under the town, eased and graded, but not lifted or lowered as a whole.
      under.sort((a, b) => a - b);
      expect(Math.abs(under[Math.floor(under.length / 2)]), `${kind}: the land under the town moved`).toBeLessThan(1.5);
    }
  }, 300_000);

  test('a land the player shaped still takes a town', () => {
    const land = landOf('country', 5150, { town: 'village', villages: 'few', size: 'medium' });
    const shaped = { ...land.open!.land!, h: land.open!.land!.h.slice(), wl: land.open!.land!.wl.slice(), cover: land.open!.land!.cover.slice() };
    const editor = new LandEditor(shaped, () => 1);
    // A lake where the town would go, and a hill beside it.
    const [sx, sz] = land.open!.plan!.site;
    editor.begin({ tool: 'lake', radius: 50, strength: 0.6 }, sx, sz);
    editor.end();
    editor.begin({ tool: 'raise', radius: 60, strength: 1 }, sx + 160, sz);
    editor.end();
    expect(shaped.edited).toBe(true);
    expect(wetTile(shaped, sx, sz)).toBe(true);
    const relaid = townOn({ ...land, open: { ...land.open!, land: shaped } }, { waterKept: false }).draft;
    expect(faults(relaid)).toEqual([]);
    expect(relaid.world?.shaped).toBe(true);
    expect(relaid.open!.land!.edited).toBe(true);
    expect(relaid.slots.length).toBeGreaterThan(40);
    // The lake is still there, and the water the sim walks is the land's.
    const W = relaid.width;
    const water = unrle(relaid.open!.water, W * relaid.depth);
    expect(water[Math.floor(sz) * W + Math.floor(sx)]).toBe(1);
    let differ = 0;
    for (let z = 0; z < relaid.depth; z += 4) for (let x = 0; x < W; x += 4) if (wetTile(relaid.open!.land!, x, z) !== !!water[z * W + x]) differ++;
    expect(differ).toBe(0);
    // (The player's own land is left as it was.)
    expect(landHash(shaped)).not.toBe(landHash(relaid.open!.land));
    expect(shaped.cols * LAND_CELL).toBe(W);
    // The game takes the land as the town has it, sample for sample, the town following it (what is saved and continued).
    const town = populate(content, assets, t, relaid);
    const file = JSON.parse(assembleTown(content, t, town, randomHousehold(content, assets, 2), vacantSlots(town)[0].index));
    const played = decodeLand(file.meta.land);
    expect(played.seeded).toBe('follow');
    expect(landHash(played)).toBe(landHash(relaid.open!.land));
  }, 300_000);
});
