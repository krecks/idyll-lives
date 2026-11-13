// Worlds and town files made in the town worker (`game/townJobs.ts`, `town.worker.ts`) are the ones
// the page would make: the job and its result go through a structured clone (as `postMessage` sends
// them), and the world, its preview and its houses come out as `generateWorld`, `previewWorld` and
// `houseTemplate` make them for the same seed (the main menu's showcase too: its land, and its preview
// once peopled on the page); a town file as `assembleTown` makes it, its dice (friends, rivals) from
// the job's own seed.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('town jobs', async () => {
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
  const { generateWorld, houseTemplate, populate, useTownContent, vacantSlots } = await import('../src/game/town');
  const { previewWorld } = await import('../src/game/townPreview');
  const { runTownJob } = await import('../src/game/townJobs');
  type Templates = import('../src/game/town').Templates;
  type TownJob = import('../src/game/townJobs').TownJob;
  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };
  /** A job as the worker runs it: sent, run, and its result sent back. */
  const inWorker = (job: TownJob) => structuredClone(runTownJob(content, t, structuredClone(job)));

  const worlds = [
    { kind: 'medium', seed: 7 },
    { kind: 'farm', seed: 3141 },
    { kind: 'seaside', seed: 271828, name: 'Typed In' },
    { kind: 'forest', seed: 99 },
  ] as const;

  for (const w of worlds) {
    test(
      `${w.kind} ${w.seed}: the worker's world is the page's`,
      () => {
        const name = 'name' in w ? w.name : undefined;
        const made = inWorker({ kind: 'world', world: w.kind, seed: w.seed, name });
        if (made.kind !== 'world') throw new Error('not a world');
        const draft = generateWorld(content, t, w.kind, w.seed, name);
        expect(made.draft).toEqual(draft);
        expect(made.preview).toEqual(previewWorld(content, t, draft, null));
        // Every generated house its lots use, as the page makes it.
        const ids = new Set(draft.slots.map((s) => s.template).filter((id) => id.startsWith('gen:')));
        expect(made.houses.map(([id]) => id).sort()).toEqual([...ids].sort());
        for (const [id, house] of made.houses) expect(house).toEqual(houseTemplate(t, id));
      },
      30_000,
    );
  }

  test(
    "the main menu's showcase: its land, peopled on the page, and its preview are the page's",
    () => {
      const land = inWorker({ kind: 'land', world: 'medium', seed: 2032020577 });
      if (land.kind !== 'land') throw new Error('not a land');
      const draft = generateWorld(content, t, 'medium', 2032020577);
      expect(land.draft).toEqual(draft);
      const ids = new Set(draft.slots.map((s) => s.template).filter((id) => id.startsWith('gen:')));
      expect(land.houses.map(([id]) => id).sort()).toEqual([...ids].sort());
      // (Peopled on the page as the menu does, with the page's dice.)
      const town = populate(content, assets, t, land.draft);
      expect(town.households.length).toBeGreaterThan(0);
      const made = inWorker({ kind: 'preview', town });
      if (made.kind !== 'preview') throw new Error('not a preview');
      expect(made.preview).toEqual(previewWorld(content, t, town, null));
    },
    30_000,
  );

  test('the worker\'s town file is the page\'s, its dice from the job\'s seed', () => {
    const town = populate(content, assets, t, generateWorld(content, t, 'small', 4242));
    const player = randomHousehold(content, assets, 2);
    const job: TownJob = { kind: 'assemble', town, player, slot: vacantSlots(town)[0].index, options: { mode: 'living', lifespan: 'normal' }, random: 12345 };
    const a = inWorker(job);
    const b = runTownJob(content, t, job);
    if (a.kind !== 'assemble' || b.kind !== 'assemble') throw new Error('not a town file');
    expect(a.lot).toBe(b.lot);
    const other = runTownJob(content, t, { ...job, random: 54321 });
    // (Another seed: other bonds between the neighbours, the same town.)
    if (other.kind !== 'assemble') throw new Error('not a town file');
    expect(JSON.parse(other.lot).plots).toEqual(JSON.parse(a.lot).plots);
    // (A whole Big town made, peopled and put together three times: 7 s on CI's runner.)
  }, 30_000);
});
