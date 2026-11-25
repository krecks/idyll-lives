// Worlds and town files made in the town worker (`game/townJobs.ts`, `town.worker.ts`) are the ones
// the page would make: the job and its result go through a structured clone (as `postMessage` sends
// them), and the world, its preview and its houses come out as `generateWorld`, `previewWorld` and
// `houseTemplate` make them for the same seed (its land graded under its town: `landFollow.ts` `settleLand`), its steps told as it goes (step 1's progress); a town
// file as `assembleTown` makes it, its dice (friends, rivals) from the job's own seed.
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';

describe('town jobs', async () => {
  // --- the page's world in bun: content from web/public (put back for whatever runs next) ---
  usePublicFetch();
  const { AssetRegistry } = await import('../src/assets/registry');
  const { randomHousehold } = await import('../src/game/household');
  const { generateWorld, houseTemplate, populate, useTownContent, vacantSlots } = await import('../src/game/town');
  const { previewWorld } = await import('../src/game/townPreview');
  const { runTownJob } = await import('../src/game/townJobs');
  const { settleLand } = await import('../src/game/landFollow');
  const { defaultsOf } = await import('../src/game/world');
  type TownJob = import('../src/game/townJobs').TownJob;
  const content = await loadContent();
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const t = await templatesOf(content);
  /** A job as the worker runs it: sent, run, and its result sent back. */
  const inWorker = (job: TownJob) => structuredClone(runTownJob(content, t, structuredClone(job)));

  const worlds = [
    { kind: 'country', seed: 7 },
    { kind: 'farm', seed: 3141 },
    { kind: 'seaside', seed: 271828, name: 'Typed In', options: { town: 'town', hills: 'hilly' } },
    { kind: 'forest', seed: 99 },
  ] as const;

  for (const w of worlds) {
    test(
      `${w.kind} ${w.seed}: the worker's world is the page's`,
      () => {
        const name = 'name' in w ? w.name : undefined;
        const options = 'options' in w ? w.options : undefined;
        const made = inWorker({ kind: 'world', world: w.kind, seed: w.seed, name, options });
        if (made.kind !== 'world') throw new Error('not a world');
        const laid = generateWorld(content, t, w.kind, w.seed, name, undefined, options);
        const draft = settleLand(laid, (d) => previewWorld(content, t, d, null), undefined, content.careerCategories).draft;
        expect(draft.open?.land?.seeded).toBe('follow');
        expect(made.draft).toEqual(draft);
        // (Its options kept with it: the place's own, but for those chosen.)
        expect(draft.world?.options).toEqual({ ...defaultsOf(w.kind), ...options });
        expect(made.preview).toEqual(previewWorld(content, t, draft, null));
        // Every generated house its lots use, as the page makes it.
        const ids = new Set(draft.slots.map((s) => s.template).filter((id) => id.startsWith('gen:')));
        expect(made.houses.map(([id]) => id).sort()).toEqual([...ids].sort());
        for (const [id, house] of made.houses) expect(house).toEqual(houseTemplate(t, id));
      },
      30_000,
    );
  }

  test('a world tells its steps in turn, and is the same world for it', () => {
    const steps: string[] = [];
    const made = runTownJob(content, t, { kind: 'world', world: 'country', seed: 909 }, (step) => steps.push(step));
    expect(steps).toEqual(['land', 'roads', 'homes', 'grade', 'houses']);
    if (made.kind !== 'world') throw new Error('not a world');
    const laid = generateWorld(content, t, 'country', 909);
    expect(made.draft).toEqual(settleLand(laid, (d) => previewWorld(content, t, d, null), undefined, content.careerCategories).draft);
  }, 30_000);

  test('the worker\'s town file is the page\'s, its dice from the job\'s seed', () => {
    const town = populate(content, assets, t, generateWorld(content, t, 'country', 4242));
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
