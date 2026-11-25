// Step 2's build-up (`game/edit/buildup.ts`) on hilly land, a town laid as step 2 lays one (the `land` and
// `town` jobs): the settle takes every lot a street is built up with, those that would break a slope limit
// left out first. (The rest of the build-up: `editBuildup.test.ts`.)
import { describe, expect, test } from 'bun:test';
import { inWorkerOf, loadContent, templatesOf, usePublicFetch } from './support/content';

describe('build-up on hilly land', async () => {
  usePublicFetch();
  const { useTownContent } = await import('../src/game/town');
  const { lengths } = await import('../src/game/roads');
  const { removeLots } = await import('../src/game/edit/lots');
  const { entriesOf } = await import('../src/game/edit/ground');
  const { buildUp } = await import('../src/game/edit/buildup');
  type NeighbourhoodDraft = import('../src/game/town').NeighbourhoodDraft;
  type WorldOptions = import('../src/game/world').WorldOptions;
  const content = await loadContent();
  useTownContent(content);
  const t = await templatesOf(content);

  const inWorker = await inWorkerOf(content, t);
  /** A town's plan as step 2 edits it (the land as laid under it). */
  const planOf = (world: string, seed: number, options: Partial<WorldOptions>): NeighbourhoodDraft => {
    const land = inWorker({ kind: 'land', world, seed, options });
    const town = inWorker({ kind: 'town', land: land.draft, seed, options, waterKept: true });
    if (!town.plan) throw new Error('the town has no plan');
    return town.plan;
  };
  /** `draft` with road `road` stripped bare: every lot fronting it gone. */
  const bare = (draft: NeighbourhoodDraft, road: number): NeighbourhoodDraft => {
    const e = entriesOf(draft, t);
    return removeLots(draft, new Set(draft.slots.filter((_, i) => e[i].road === road).map((s) => s.index)));
  };
  /** The roads of area `area` (streets and lanes) by length, longest first. */
  const streetsOf = (draft: NeighbourhoodDraft, area: number) =>
    draft.roads
      .map((r, i) => ({ r, i, L: lengths(r.points).at(-1)! }))
      .filter(({ r }) => r.area === area && (r.kind === 'street' || r.kind === 'lane') && !r.bridge)
      .sort((a, b) => b.L - a.L)
      .map(({ i }) => i);

  test('on hilly land the settle takes every lot a street is built up with: those that would break a slope limit are left out first', () => {
    const settle = (steps: readonly NeighbourhoodDraft[]) => inWorker({ kind: 'edit', plan: steps[steps.length - 1], ops: steps.length - 1, steps, check: false });
    const hilly = planOf('heath', 5, { town: 'town', villages: 'few', size: 'medium', hills: 'hilly' });
    let [built, left] = [0, 0];
    for (const road of streetsOf(hilly, 0).slice(0, 3)) {
      const b = bare(hilly, road);
      const e = buildUp(b, t, content, { road, mode: 'both', seed: 1 });
      if ('refused' in e) continue;
      built += e.draft.slots.length - b.slots.length;
      left += e.leftOut ?? 0;
      expect(settle([b, e.draft]).refused).toEqual([]);
    }
    expect(built).toBeGreaterThan(10);
    console.log(`hilly: ${built} lots, ${left} left out`);
  }, 120_000);
});
