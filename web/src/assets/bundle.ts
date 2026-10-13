/**
 * The game data every start needs, in one file the build writes (`bundle.json`, see `dataBundle`
 * in vite.config.ts): `content/base.json` and the files it includes, the house templates, the
 * asset manifests with their packs and the index of compressed copies, each file's text by its
 * path below the site's base. One request instead of about 60 in three waves, each waiting for
 * the one before. The files also ship one by one: the dev server has no bundle and serves them,
 * and anything not in the bundle (a `?pack=`) is fetched as before.
 */

import { withBuild, withoutBuild } from './build';

let files: Promise<Record<string, string> | null> | null = null;

/** A file's text from the bundle; null when it isn't in one (or there is no bundle). */
export async function bundled(url: string): Promise<string | null> {
  // (None outside Vite's builds: the dev server, and tools/worlds/sample.ts in Node.)
  if (!import.meta.env?.PROD) return null;
  const base = new URL(import.meta.env.BASE_URL, location.href);
  const file = new URL(withoutBuild(url), base);
  if (file.origin !== base.origin || file.search || !file.pathname.startsWith(base.pathname)) return null;
  files ??= fetch(withBuild(new URL('bundle.json', base).href))
    .then((res) => (res.ok ? (res.json() as Promise<Record<string, string>>) : null))
    .catch(() => null);
  return (await files)?.[file.pathname.slice(base.pathname.length)] ?? null;
}
