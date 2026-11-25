// The furnish catalog (`ui/buy/catalog.ts`) over the shipped content: every item for sale is summed up and its
// lovers found without throwing, decor with no interactions (a field wall) among them, as sim-core reads such an
// object (its interactions left out: none).
import { describe, expect, test } from 'bun:test';
import { loadContent, usePublicFetch } from './support/content';

describe('the furnish catalog', async () => {
  usePublicFetch();
  const { loversOf, summarize } = await import('../src/ui/buy/catalog');
  const content = await loadContent();

  test('an object without interactions has none', () => {
    expect(content.object('countryside.dryStoneWall')?.interactions).toEqual([]);
    for (const def of content.objectList) expect(Array.isArray(def.interactions)).toBe(true);
  });

  test('every item for sale is summed up and its lovers found', () => {
    const resident = { name: 'Ada Lovelace', traits: content.traits.map((t) => t.id) };
    for (const def of content.shop) {
      expect(() => summarize(content, def)).not.toThrow();
      expect(() => loversOf(content, def, [resident])).not.toThrow();
    }
  });
});
