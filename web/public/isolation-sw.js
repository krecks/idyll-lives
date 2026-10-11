/**
 * Cross-origin isolation where the host can't send the headers (GitHub Pages): this service
 * worker adds them to the site's own responses, so the page gets SharedArrayBuffer (the
 * simulation's zero-copy snapshots) and threads for ONNX Runtime (the language model on the
 * CPU). Registered by `index.html` only when the page isn't isolated already (the dev server
 * and `vite preview` send the headers themselves: see `vite.config.ts`). Caches nothing.
 *
 * Cross-origin downloads (models from Hugging Face) are fetched with CORS, which
 * `require-corp` accepts.
 */

self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', (event) => event.waitUntil(self.clients.claim()));

self.addEventListener('fetch', (event) => {
  const request = event.request;
  // A request the browser may only answer from its cache, across origins: not ours to fetch.
  if (request.cache === 'only-if-cached' && request.mode !== 'same-origin') return;
  // Only the site's own files get the headers; other origins answer for themselves.
  if (new URL(request.url).origin !== self.location.origin) return;
  event.respondWith(
    fetch(request).then((response) => {
      if (response.status === 0) return response;
      const headers = new Headers(response.headers);
      headers.set('Cross-Origin-Opener-Policy', 'same-origin');
      headers.set('Cross-Origin-Embedder-Policy', 'require-corp');
      headers.set('Cross-Origin-Resource-Policy', 'same-origin');
      return new Response(response.body, { status: response.status, statusText: response.statusText, headers });
    }),
  );
});
