/**
 * The game data every start needs, in one file the build writes (`bundle.json`, see `dataBundle`
 * in vite.config.ts): `content/base.json` and the files it includes, the house templates, the
 * asset manifests with their packs and the index of compressed copies, each file's text by its
 * path below the site's base. One request instead of about 60 in three waves, each waiting for
 * the one before. The files also ship one by one: the dev server has no bundle and serves them,
 * and anything not in the bundle (a `?pack=`) is fetched as before.
 *
 * It also carries the content hash of every data file the build ships, by which they're fetched
 * (`build.ts`): its `hashes`.
 */

import { useHashes, withBuild, withoutBuild } from './build';

/** `bundle.json`: each file's text by its path, and the content hash of every data file the build ships (`build.ts`). */
interface Bundle {
  files: Record<string, string>;
  hashes?: Record<string, string>;
  /** (The build's hashed files, which the isolation service worker keeps while a build lists them.) */
  built?: string[];
}

let bundle: Promise<Bundle | null> | null = null;

/** The build's bundle (fetched once), or null outside a build or when it can't be had. */
function load(): Promise<Bundle | null> {
  // (None outside Vite's builds: the dev server, and tools/worlds/sample.ts in Node.)
  if (!import.meta.env?.PROD) return Promise.resolve(null);
  const base = new URL(import.meta.env.BASE_URL, location.href);
  return (bundle ??= fetch(withBuild(new URL('bundle.json', base).href))
    .then((res) => (res.ok ? (res.json() as Promise<Bundle>) : null))
    .then((read) => {
      if (read?.hashes) useHashes(read.hashes, base);
      return read?.files ? read : null;
    })
    .catch(() => null));
}

/** A file's text from the bundle; null when it isn't in one (or there is no bundle). */
export async function bundled(url: string): Promise<string | null> {
  if (!import.meta.env?.PROD) return null;
  const base = new URL(import.meta.env.BASE_URL, location.href);
  const file = new URL(withoutBuild(url), base);
  if (file.origin !== base.origin || file.search || !file.pathname.startsWith(base.pathname)) return null;
  return (await load())?.files[file.pathname.slice(base.pathname.length)] ?? null;
}

/** `url` as `withBuild` gives it once the bundle's content hashes are known: for files fetched before anything else needed the bundle. */
export async function dataUrl(url: string): Promise<string> {
  await load();
  return withBuild(url);
}
