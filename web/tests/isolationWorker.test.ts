// `public/isolation-sw.js`: which requests its caches keep for good (`built`), and which kept files a
// new build lets go (`sweep`). A mistake here leaves players on stale files the network is never asked for again.
import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';

const ROOT = 'https://example.github.io/game/';
const BUILD = 'abc123';

/** The worker as a build stamps it (`vite.config.ts` `isolationWorker`), its functions handed out; `kept` stands in for its data cache. */
function worker(kept: Request[] = []) {
  const source = readFileSync(new URL('../public/isolation-sw.js', import.meta.url), 'utf8');
  const code = source.replace("const BUILD = 'dev';", `const BUILD = '${BUILD}';`);
  expect(code).not.toBe(source);
  const self = { location: { href: `${ROOT}isolation-sw.js` }, addEventListener() {} };
  const cache = { keys: async () => [...kept], delete: async (r: Request) => (kept.splice(kept.indexOf(r), 1), true) };
  const caches = { open: async () => cache };
  return new Function('self', 'caches', `${code}\nreturn { built, sweep };`)(self, caches) as {
    built(request: Request, url: URL): string | null;
    sweep(bundle: Response): Promise<void>;
  };
}

const cacheOf = (path: string, init?: RequestInit) => {
  const url = new URL(path, ROOT);
  return worker().built(new Request(url, init), url);
};

describe('built', () => {
  test('a file asked for by its content hash, and a hashed build file, go in the data cache kept across builds', () => {
    expect(cacheOf('assets/models/held/mug.glb?h=0123456789ab')).toBe('littlelives-data');
    expect(cacheOf('assets/index-Ab_d1-34.js')).toBe('littlelives-data');
  });

  test("this build's query goes in this build's cache; another build's, or none, is not kept", () => {
    expect(cacheOf(`content/base.json?v=${BUILD}`)).toBe(`littlelives-build-${BUILD}`);
    expect(cacheOf('content/base.json?v=older')).toBeNull();
    expect(cacheOf('content/base.json')).toBeNull();
    expect(cacheOf('index.html')).toBeNull();
  });

  test('not kept: a range, a POST, a hash with another query, the voice and model runtimes, a bad hash', () => {
    expect(cacheOf('assets/models/held/mug.glb?h=0123456789ab', { headers: { range: 'bytes=0-9' } })).toBeNull();
    expect(cacheOf('assets/index-Ab_d1-34.js', { method: 'POST' })).toBeNull();
    expect(cacheOf('assets/models/held/mug.glb?h=0123456789ab&x=1')).toBeNull();
    expect(cacheOf('assets/ort-wasm-Ab_d1-34.wasm')).toBeNull();
    expect(cacheOf('assets/models/held/mug.glb?h=0123456789AB')).toBeNull();
  });
});

describe('sweep', () => {
  test('lets go of a file whose hash changed or that the build no longer ships, keeps the rest', async () => {
    const at = (path: string) => new Request(new URL(path, ROOT));
    const kept = [at('assets/a.glb?h=000000000001'), at('assets/b.glb?h=000000000002'), at('assets/index-AAAAAAAA.js'), at('assets/index-BBBBBBBB.js')];
    const bundle = Response.json({ hashes: { 'assets/a.glb': '000000000001', 'assets/b.glb': '0000000000ff' }, built: ['assets/index-AAAAAAAA.js'] });
    await worker(kept).sweep(bundle);
    expect(kept.map((r) => r.url.slice(ROOT.length))).toEqual(['assets/a.glb?h=000000000001', 'assets/index-AAAAAAAA.js']);
  });
});
