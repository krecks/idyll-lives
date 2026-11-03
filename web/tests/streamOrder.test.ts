/**
 * The background stream's order (`assets/stream.ts`): in the order asked, until the game ranks the
 * waiting files by where the camera is (`reprioritize`); files it doesn't rank come last. And a
 * loading screen's downloads (`prefetchNow`): a file whose download fails is tried once more, one
 * the server hasn't got isn't, and either way the loading goes on.
 */
import { afterEach, describe, expect, test } from 'bun:test';
import { background, fetched, foreground, prefetchNow, reprioritize } from '../src/assets/stream';

describe('streaming by priority', () => {
  test('waiting files start by rank, the unranked last, ties in the order asked', async () => {
    const started: string[] = [];
    let release!: () => void;
    // A load someone waits for holds the background back while the queue fills.
    const held = foreground(() => new Promise<void>((r) => (release = r)));
    const loads = ['far', 'near', 'unknown', 'focus', 'near2'].map((url) => background(async () => void started.push(url), url));
    const rank: Record<string, number> = { far: 2.5, near: 1.1, focus: 0, near2: 1.1 };
    reprioritize((url) => rank[url] ?? Infinity);
    release();
    await held;
    await Promise.all(loads);
    reprioritize(null);
    expect(started).toEqual(['focus', 'near', 'near2', 'far', 'unknown']);
  });
});

describe('downloads for a loading screen', () => {
  const realFetch = globalThis.fetch;
  afterEach(() => {
    globalThis.fetch = realFetch;
  });

  /** A fetch answering by URL: `fail` network errors first for each, then the file (or a 404 for `missing`). */
  function fakeFetch(fail: Record<string, number>, missing: string[] = []) {
    const asked: string[] = [];
    globalThis.fetch = (async (input: RequestInfo | URL) => {
      const url = String(input);
      asked.push(url);
      if ((fail[url] ?? 0) > 0) {
        fail[url]--;
        throw new TypeError('network error');
      }
      return missing.includes(url) ? new Response('', { status: 404 }) : new Response(new Uint8Array(8));
    }) as typeof fetch;
    return asked;
  }

  test('a download that fails is tried once more; a missing file is not; the rest carries on', async () => {
    const asked = fakeFetch({ 'https://x/once.glb': 1, 'https://x/twice.glb': 2 }, ['https://x/missing.glb']);
    const seen: number[] = [];
    await prefetchNow(['https://x/once.glb', 'https://x/twice.glb', 'https://x/missing.glb', 'https://x/fine.glb'], (done) => seen.push(done));
    expect(fetched('https://x/once.glb')).toBe(true);
    expect(fetched('https://x/twice.glb')).toBe(false);
    expect(fetched('https://x/missing.glb')).toBe(false);
    expect(fetched('https://x/fine.glb')).toBe(true);
    expect(asked.filter((u) => u === 'https://x/once.glb').length).toBe(2);
    expect(asked.filter((u) => u === 'https://x/twice.glb').length).toBe(2);
    expect(asked.filter((u) => u === 'https://x/missing.glb').length).toBe(1);
    expect(seen.at(-1)).toBe(4);
  });
});
