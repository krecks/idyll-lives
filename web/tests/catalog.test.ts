// The furnish catalog (`ui/buy/catalog.ts`) over the shipped content: every item for sale is summed up and its
// lovers found without throwing, decor with no interactions (a field wall) among them, as sim-core reads such an
// object (its interactions left out: none).
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';

const WEB = new URL('../', import.meta.url).pathname;

describe('the furnish catalog', async () => {
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
  const { loversOf, summarize } = await import('../src/ui/buy/catalog');
  const content = await Content.load('http://localhost/content/base.json');

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
