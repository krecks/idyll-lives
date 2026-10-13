/**
 * Cross-origin isolation where the host can't send the headers (GitHub Pages): this service
 * worker adds them to the site's own responses, so the page gets SharedArrayBuffer (the
 * simulation's zero-copy snapshots) and threads for ONNX Runtime (the language model on the
 * CPU). Registered by `index.html` only when the page isn't isolated already (the dev server
 * and `vite preview` send the headers themselves: see `vite.config.ts`).
 *
 * It also keeps the files the build made (`assets/<name>-<hash>.<ext>`: scripts, styles, the
 * simulation's wasm, fonts), which never change under their names, in a cache of this build's
 * own: a returning player loads them without asking the server again (Pages lets the browser
 * keep files for only 10 minutes). Everything else, the page itself included, comes from the
 * network as before. A new build is a new version of this file (the build writes its id below),
 * and the new worker deletes the old build's cache.
 *
 * Cross-origin downloads (models from Hugging Face) are fetched with CORS, which
 * `require-corp` accepts.
 */

/** This build's id (vite.config.ts writes it into the built copy). */
const BUILD = 'dev';
const PREFIX = 'littlelives-build-';
const CACHE = PREFIX + BUILD;
/** Where the build puts its files; a name with Vite's 8-character hash is one of them. */
const ASSETS = new URL('assets/', self.location.href).pathname;
const HASHED = /^[^/]+-[\w-]{8}\.(?:js|css|wasm|woff2)$/;
/** Kept by the voice's and language model's workers in caches of their own (ml/assets.ts). */
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
  event.respondWith(built(request, url) ? fromCache(event, request) : fetch(request).then(isolated));
});

/** One of this build's own files: never changes under its name. */
function built(request, url) {
  if (request.method !== 'GET' || request.headers.has('range') || url.search || !url.pathname.startsWith(ASSETS)) return false;
  const name = url.pathname.slice(ASSETS.length);
  return HASHED.test(name) && !KEPT_ELSEWHERE.test(name);
}

/** From this build's cache, else from the network (and then kept, headers and all). */
async function fromCache(event, request) {
  const cache = await caches.open(CACHE).catch(() => null);
  const hit = await cache?.match(request, { ignoreVary: true }).catch(() => undefined);
  if (hit) return hit;
  const response = await fetch(request);
  const out = isolated(response);
  if (cache && response.ok && response.status === 200 && response.type === 'basic') event.waitUntil(cache.put(request, out.clone()).catch(() => {}));
  return out;
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
