/**
 * Keeps one release's files apart from the last one's in the browser's cache. GitHub Pages caches
 * everything for 10 minutes under the same names, and a deploy replaces the whole site, so every
 * runtime fetch from `public/` (content, asset manifests, art) carries a query naming what it is:
 * new code never meets old content or manifests from the cache.
 *
 * - A file the build knows the contents of (its bundle's `hashes`, `bundle.ts`: every model, texture,
 *   character file, icon and content file it ships) is asked for by a hash of them, `?h=<hash>`: the
 *   same URL from release to release while the file doesn't change, so the isolation service worker
 *   keeps it across releases and a release downloads only the files it changed.
 * - Anything else (`bundle.json` itself, a file asked for before the bundle arrived, a modder's pack)
 *   carries this build's id, `?v=<build>`.
 *
 * Not for `voice/` and `llm/`: their files are pinned by SHA-256 (a new model gets a new folder), so
 * they stay cached across releases in the workers' own caches.
 */

/** This build's id; 'dev' outside Vite (tools/worlds/sample.ts runs the loaders in Node). */
export const BUILD_ID = typeof __BUILD_ID__ === 'string' ? __BUILD_ID__ : 'dev';
/** None outside Vite. */
const QUERY = typeof __BUILD_ID__ === 'string' ? `v=${__BUILD_ID__}` : '';
/** A content hash's query: what `vite.config.ts` (`dataHashes`) writes, and the service worker keeps across builds. */
const HASH_QUERY = /[?&]h=[0-9a-f]{12}(?![0-9a-z])/;
const HASH_QUERIES = new RegExp(HASH_QUERY.source, 'g');

/** File path below the site's base -> its content hash (the build's bundle); null until it's read, or without one. */
let hashes: ReadonlyMap<string, string> | null = null;
let site: URL | null = null;

/** The build's content hashes (`bundle.ts`, once it has the bundle): paths below `base`, the site's root. */
export function useHashes(index: Record<string, string>, base: URL): void {
  hashes = new Map(Object.entries(index));
  site = base;
}

/** The content hash the build gave the file at `url`, if it has one (a URL with a query of its own has none). */
function hashOf(url: string): string | undefined {
  if (!hashes || !site) return undefined;
  const file = new URL(url, location.href);
  if (file.search || file.hash || file.origin !== site.origin || !file.pathname.startsWith(site.pathname)) return undefined;
  return hashes.get(file.pathname.slice(site.pathname.length));
}

/**
 * `url` as this build fetches it (once): by its content hash where the build knows it, else with this
 * build's query (`data:` and `blob:` URLs stay as they are).
 */
export function withBuild(url: string): string {
  if (!QUERY || url.startsWith('data:') || url.startsWith('blob:') || url.includes(QUERY)) return url;
  if (HASH_QUERY.test(url)) return url;
  const hash = hashOf(url);
  return `${url}${url.includes('?') ? '&' : '?'}${hash ? `h=${hash}` : QUERY}`;
}

/** `text` (a URL, or JSON holding URLs) without the queries `withBuild` adds: for keys that outlive a build. */
export function withoutBuild(text: string): string {
  if (!QUERY) return text;
  return text.replaceAll(`?${QUERY}`, '').replaceAll(`&${QUERY}`, '').replace(HASH_QUERIES, '');
}
