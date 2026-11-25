// Step 2's Problems list (`ui/townedit/problems.ts` `townIssues`) on a laid city with many villages: it blocks
// nothing. (The town editor's other tests: `townEditor.test.ts`.)
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';

describe('the Problems list, on a generated city', async () => {
  usePublicFetch();
  const { generateWorld, useTownContent } = await import('../src/game/town');
  const { townIssues } = await import('../src/ui/townedit/problems');
  const content = await loadContent();
  useTownContent(content);
  const t = await templatesOf(content);

  // (A city's checks take seconds, which is why they run in their own worker, `checkJob.ts`: printed here to keep an eye on.)
  test('a laid city blocks nothing', () => {
    const city = generateWorld(content, t, 'country', 77, undefined, undefined, { town: 'city', villages: 'many' });
    const t0 = performance.now();
    const found = townIssues(city, t, content);
    console.log(`townIssues on a city with many villages (${city.slots.length} lots, ${city.open?.carParks?.length ?? 0} car parks): ${(performance.now() - t0).toFixed(0)} ms`);
    expect(found.blocking).toEqual([]);
  }, 60_000);
});
