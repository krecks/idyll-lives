/**
 * Cross-origin isolation where the host can't send the headers (GitHub Pages): this service
 * worker adds them to the site's own responses, so the page gets SharedArrayBuffer (the
 * simulation's zero-copy snapshots) and threads for ONNX Runtime (the language model on the
 * CPU). Registered by `index.html` only when the page isn't isolated already (the dev server
 * and `vite preview` send the headers themselves: see `vite.config.ts`).
 *
 * It also keeps what never changes under its URL, so a returning player loads it without asking
 * the server again (Pages lets the browser keep files for only 10 minutes):
 *
 * - the files a build makes (`assets/<name>-<hash>.<ext>`: scripts, styles, the simulation's wasm,
 *   fonts) and the game's data asked for by its content hash (`?h=<hash>`, `src/assets/build.ts`:
 *   models, textures, the residents, icons, the content), in one cache for every build: a release
 *   downloads only the files it changed. Each time the page reads this build's `bundle.json`, which
 *   lists them (`built`, `hashes`), those no longer listed (changed or gone) are let go;
 * - the game's data fetched with this build's query (`?v=<build id>`: `bundle.json`, and anything
 *   the build has no hash for), in a cache of this build's own. A new build is a new version of this
 *   file (the build writes its id below), and the new worker deletes the old build's cache.
 *
 * Everything else, the page itself included, comes from the network as before.
 *
 * Cross-origin downloads (models from Hugging Face) are fetched with CORS, which
 * `require-corp` accepts.
 */

/** This build's id (vite.config.ts writes it into the built copy). */
const BUILD = 'dev';
const PREFIX = 'littlelives-build-';
const CACHE = PREFIX + BUILD;
/** Hashed build files and data by content hash, kept across builds (not under `PREFIX`, which a new build clears). */
const DATA = 'littlelives-data';
/** A content hash's query, the URL's only one (`src/assets/build.ts`). */
const HASH = /^[0-9a-f]{12}$/;
/** This build's `bundle.json`, which lists the hashed files and the data files it ships. */
const BUNDLE = new URL('bundle.json', self.location.href).pathname;
/** Where the build puts its files; a name with Vite's 8-character hash is one of them. */
const ASSETS = new URL('assets/', self.location.href).pathname;
const HASHED = /^[^/]+-[\w-]{8}\.(?:js|css|wasm|woff2)$/;
/** Kept by the voice's and language model's workers in caches of their own (ml/assets.ts; the runtime in one they share). */
const KEPT_ELSEWHERE = /^(?:ort-|voice_wasm_bg-)/;

self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', (event) =>
  event.waitUntil(
    (async () => {
      try {
        for (const name of await caches.keys()) if (name.startsWith(PREFIX) && name !== CACHE) await caches.delete(name);
      } catch {
        // Only space: the files still load.
      }
      await self.clients.claim();
    })(),
  ),
);

self.addEventListener('fetch', (event) => {
  const request = event.request;
  // A request the browser may only answer from its cache, across origins: not ours to fetch.
  if (request.cache === 'only-if-cached' && request.mode !== 'same-origin') return;
  // Only the site's own files get the headers; other origins answer for themselves.
  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;
  const cache = built(request, url);
  event.respondWith(cache ? fromCache(event, request, cache) : fetch(request).then(isolated));
});

/**
 * The cache for a file that never changes under its URL (null: not one): `DATA` for a hashed build
 * file or one asked for by its content hash, this build's own for game data with its id.
 */
function built(request, url) {
  if (request.method !== 'GET' || request.headers.has('range') || request.cache === 'no-store') return null;
  if (url.search) {
    const params = [...url.searchParams];
    if (params.length === 1 && params[0][0] === 'h' && HASH.test(params[0][1])) return DATA;
    // (Game data: only this build's. The dev server's 'dev' build never has the worker.)
    return url.searchParams.get('v') === BUILD && BUILD !== 'dev' ? CACHE : null;
  }
  if (!url.pathname.startsWith(ASSETS)) return null;
  const name = url.pathname.slice(ASSETS.length);
  return HASHED.test(name) && !KEPT_ELSEWHERE.test(name) ? DATA : null;
}

/** From `name`'s cache, else from the network (and then kept, headers and all). */
async function fromCache(event, request, name) {
  const cache = await caches.open(name).catch(() => null);
  const hit = await cache?.match(request, { ignoreVary: true }).catch(() => undefined);
  let out = hit;
  if (!out) {
    const response = await fetch(request);
    out = isolated(response);
    if (cache && response.ok && response.status === 200 && response.type === 'basic') event.waitUntil(cache.put(request, out.clone()).catch(() => {}));
  }
  if (!swept && name === CACHE && out.ok && new URL(request.url).pathname === BUNDLE) event.waitUntil(sweep(out.clone()));
  return out;
}

/** Whether the data cache has been swept since this worker started. */
let swept = false;

/**
 * Lets go of the files this build no longer ships (`bundle`: its `bundle.json`, whose `built` names
 * its hashed files and `hashes` each data file's content hash, by their paths below the site's root):
 * those changed or gone since they were kept. Once per start of the worker; the files this build asks
 * for are never among them.
 */
async function sweep(bundle) {
  if (swept) return;
  swept = true;
  try {
    const { hashes, built } = await bundle.json();
    if (!hashes || !built) return;
    const own = new Set(built);
    const root = new URL('./', self.location.href).pathname;
    const cache = await caches.open(DATA);
    for (const request of await cache.keys()) {
      const url = new URL(request.url);
      const path = url.pathname.startsWith(root) ? url.pathname.slice(root.length) : '';
      const kept = url.search ? hashes[path] === url.searchParams.get('h') : own.has(path);
      if (!kept) await cache.delete(request);
    }
  } catch {
    // Only space: the next start tries again.
    swept = false;
  }
}

/** `response` with the headers that isolate the page. */
function isolated(response) {
  if (response.status === 0) return response;
  const headers = new Headers(response.headers);
  headers.set('Cross-Origin-Opener-Policy', 'same-origin');
  headers.set('Cross-Origin-Embedder-Policy', 'require-corp');
  headers.set('Cross-Origin-Resource-Policy', 'same-origin');
  return new Response(response.body, { status: response.status, statusText: response.statusText, headers });
}
