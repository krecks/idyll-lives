/**
 * Keeps one release's files apart from the last one's in the browser's cache. GitHub Pages caches
 * everything for 10 minutes under the same names, and a deploy replaces the whole site, so every
 * runtime fetch from `public/` (content, asset manifests, art) carries this build's id as a query:
 * new code never meets old content or manifests from the cache. Not for `voice/` and `llm/`: their
 * files are pinned by SHA-256 (a new model gets a new folder), so they stay cached across releases.
 */

/** None outside Vite (tools/worlds/sample.ts runs the loaders in Node). */
const QUERY = typeof __BUILD_ID__ === 'string' ? `v=${__BUILD_ID__}` : '';

/** `url` with this build's query (once; `data:` and `blob:` URLs stay as they are). */
export function withBuild(url: string): string {
  if (!QUERY || url.startsWith('data:') || url.startsWith('blob:') || url.includes(QUERY)) return url;
  return `${url}${url.includes('?') ? '&' : '?'}${QUERY}`;
}

/** `text` (a URL, or JSON holding URLs) without this build's query: for keys that outlive a build. */
export function withoutBuild(text: string): string {
  return QUERY ? text.replaceAll(`?${QUERY}`, '').replaceAll(`&${QUERY}`, '') : text;
}
