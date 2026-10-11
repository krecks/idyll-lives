/**
 * Model files for workers that run ONNX models (`voice/tts.worker.ts`, `llm/llm.worker.ts`):
 * downloaded once with progress and kept in the Cache API, so the next session starts without
 * downloading. Each worker has its own versioned cache (`prefix` + version).
 */

/** This version's cache (`name`); caches of earlier versions (same `prefix`) are deleted. */
export async function openCache(prefix: string, name: string): Promise<Cache | null> {
  try {
    for (const old of await caches.keys()) {
      if (old.startsWith(prefix) && old !== name) await caches.delete(old);
    }
    return await caches.open(name);
  } catch {
    return null;
  }
}

/** Drops cached files this version no longer uses (an older build's wasm, models no longer used). */
export async function prune(cache: Cache | null, keep: string[]): Promise<void> {
  if (!cache) return;
  const wanted = new Set(keep.map((url) => new URL(url, location.href).href));
  try {
    for (const req of await cache.keys()) if (!wanted.has(req.url)) await cache.delete(req);
  } catch {
    // Only space: the files still load.
  }
}

/** Unpacks a gzipped file, or reads it as text if a server already unpacked it. */
export async function gunzipText(bytes: ArrayBuffer): Promise<string> {
  const head = new Uint8Array(bytes, 0, 2);
  if (head[0] !== 0x1f || head[1] !== 0x8b) return new TextDecoder().decode(bytes);
  return new Response(new Blob([bytes]).stream().pipeThrough(new DecompressionStream('gzip'))).text();
}

/** Each file once per worker (a load tried again after failing only fetches what it's missing). */
const fetched = new Map<string, Promise<ArrayBuffer>>();
export function fetchOnce(cache: Cache | null, url: string, progress: (bytes: number) => void, sha256?: string): Promise<ArrayBuffer> {
  let p = fetched.get(url);
  if (!p) {
    p = fetchCached(cache, url, progress, sha256);
    p.catch(() => fetched.delete(url));
    fetched.set(url, p);
    return p;
  }
  return p.then((b) => {
    progress(b.byteLength);
    return b;
  });
}

/** Forgets a file `fetchOnce` holds (its bytes went to a session that keeps its own copy). */
export function forget(url: string): void {
  fetched.delete(url);
}

/** The file's bytes from the cache, or downloaded (with progress), checked against `sha256` if given, and cached. */
export async function fetchCached(cache: Cache | null, url: string, progress: (bytes: number) => void, sha256?: string): Promise<ArrayBuffer> {
  const hit = await cache?.match(url);
  if (hit) {
    const bytes = await hit.arrayBuffer();
    progress(bytes.byteLength);
    return bytes;
  }
  const res = await fetch(url);
  if (!res.ok || !res.body) throw new Error(`${url}: HTTP ${res.status}`);
  const chunks: Uint8Array[] = [];
  let loaded = 0;
  const reader = res.body.getReader();
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    loaded += value.byteLength;
    progress(loaded);
  }
  const bytes = new Uint8Array(loaded);
  let at = 0;
  for (const c of chunks) {
    bytes.set(c, at);
    at += c.byteLength;
  }
  if (sha256 && (await sha256Hex(bytes.buffer)) !== sha256) throw new Error(`${url}: not the file expected (SHA-256 differs)`);
  await cache?.put(url, new Response(bytes, { headers: { 'Content-Type': res.headers.get('Content-Type') ?? 'application/octet-stream' } })).catch(() => {});
  return bytes.buffer;
}

/** SHA-256 of some bytes, as hex. */
export async function sha256Hex(bytes: ArrayBuffer): Promise<string> {
  const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
  return Array.from(digest, (b) => b.toString(16).padStart(2, '0')).join('');
}
