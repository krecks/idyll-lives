// The page's world in bun: a `location` at http://localhost/ and a `fetch` that reads web/public,
// then the shipped content and house templates through it. The src modules are imported only
// inside the helpers, so a test's own `await import(...)` of src still runs after the stub is in.
import { afterAll } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import type { Content } from '../../src/content/content';
import type { Templates } from '../../src/game/town';
import type { TownJob, TownResult } from '../../src/game/townJobs';

const PUBLIC = new URL('../../public', import.meta.url).pathname;

/** Fakes `location` and swaps `fetch` for one reading files under web/public (404 'missing' otherwise); both put back after the enclosing describe for whatever runs next. */
export function usePublicFetch(): void {
  const realLocation = globalThis.location;
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url, 'http://localhost/');
    const path = `${PUBLIC}${decodeURIComponent(url.pathname)}`;
    return existsSync(path) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
  }) as typeof fetch;
  afterAll(() => {
    globalThis.fetch = realFetch;
    Object.assign(globalThis, { location: realLocation });
  });
}

// Memoized per process: no test writes to the loaded content or templates, so every file shares one parse.
let contentP: Promise<Content> | undefined;
/** The shipped content (content/base.json), through the stubbed fetch. */
export function loadContent(): Promise<Content> {
  return (contentP ??= import('../../src/content/content').then(({ Content }) => Content.load('http://localhost/content/base.json')));
}

const templatesCache = new WeakMap<Content, Promise<Templates>>();
/** The shipped house templates (houses.json, housegen.json) with `content`'s object infos and tiles. */
export function templatesOf(content: Content): Promise<Templates> {
  let p = templatesCache.get(content);
  if (!p) {
    p = import('../../src/game/housegen').then(({ objectInfos }) => {
      const json = <T>(file: string) => JSON.parse(readFileSync(`${PUBLIC}/content/${file}`, 'utf8')) as T;
      return { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };
    });
    templatesCache.set(content, p);
  }
  return p;
}

/** A town job as the worker runs it: sent, run, and its result sent back. */
export async function inWorkerOf(content: Content, t: Templates) {
  const { runTownJob } = await import('../../src/game/townJobs');
  return <K extends TownJob['kind']>(job: Extract<TownJob, { kind: K }>) => structuredClone(runTownJob(content, t, structuredClone(job))) as Extract<TownResult, { kind: K }>;
}
