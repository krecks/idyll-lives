// The furnish catalog (`ui/buy/catalog.ts`) over the shipped content: every item for sale is summed up and its
// lovers found without throwing, decor with no interactions (a field wall) among them, as sim-core reads such an
// object (its interactions left out: none).
import { describe, expect, test } from 'bun:test';
import { loadContent, usePublicFetch } from './support/content';

describe('the furnish catalog', async () => {
  usePublicFetch();
  const { appeal, LOVES, loversOf, summarize } = await import('../src/ui/buy/catalog');
  const content = await loadContent();

  test('an object without interactions has none', () => {
    expect(content.object('countryside.dryStoneWall')?.interactions).toEqual([]);
    for (const def of content.objectList) expect(Array.isArray(def.interactions)).toBe(true);
  });

  test("a foodie loves a stove: the traits' preferences multiply over its tags, as in the sim", () => {
    const stove = content.objectList.find((d) => d.interactions.some((it) => it.tags?.length === 2 && it.tags.includes('food') && it.tags.includes('cooking')));
    expect(stove).toBeDefined();
    // (food 1.6 × cooking 1.8: loved, where the larger of the two alone is not.)
    expect(appeal(content, stove!, ['foodie'])).toBeGreaterThanOrEqual(LOVES);
  });

  test('every item for sale is summed up and its lovers found', () => {
    const resident = { name: 'Ada Lovelace', traits: content.traits.map((t) => t.id) };
    for (const def of content.shop) {
      expect(() => summarize(content, def)).not.toThrow();
      expect(() => loversOf(content, def, [resident])).not.toThrow();
    }
  });
});
