/**
 * `?memtrace` (dev server only): records every typed array and ArrayBuffer of 64 KB or more the page's
 * own code makes, by the stack that made it, and how much of it is still alive (a finalizer per buffer).
 * `window.__memtrace()` answers the sites, the largest first; `tools/bench/gameplay.mjs --memtrace`
 * reads it at the entry and at the end. It sees the main thread only (a worker has its own globals), and
 * only what is made through a constructor (`new`, `from`, `of`, `slice`, `map`, `subarray` makes none),
 * plus `Response`/`Blob` `arrayBuffer()`: what the browser makes on its own side is not seen.
 *
 * It wraps the constructors in proxies, so it costs a stack trace per large allocation and is never in
 * a build: `import.meta.env.DEV` is false there and the module does nothing.
 */

/** The smallest allocation recorded (bytes). */
const LARGE = 64 * 1024;
/** Frames of a stack kept for its key (after the tracer's own). */
const FRAMES = 6;

interface Site {
  /** Bytes and allocations made, all told. */
  bytes: number;
  count: number;
  /** Bytes still alive (not yet collected). */
  live: number;
  /** The largest one. */
  largest: number;
}

interface MemtraceSite extends Site {
  stack: string;
}

declare global {
  interface Window {
    /** The large allocations by site, the most bytes alive first (`?memtrace`, dev only). */
    __memtrace?: (top?: number) => { sites: MemtraceSite[]; bytes: number; live: number; count: number };
  }
}

const params = new URLSearchParams(location.search);

if (import.meta.env.DEV && params.has('memtrace')) install();

function install(): void {
  const sites = new Map<string, Site>();
  const freed = new FinalizationRegistry<{ site: Site; bytes: number }>(({ site, bytes }) => {
    site.live -= bytes;
  });
  /** The buffers seen already (a view on one isn't a new allocation). */
  const seen = new WeakSet<ArrayBufferLike>();
  let busy = false;

  const record = (buffer: ArrayBufferLike) => {
    const bytes = buffer.byteLength;
    if (bytes < LARGE || busy || seen.has(buffer)) return;
    busy = true;
    try {
      seen.add(buffer);
      const stack = (new Error().stack ?? '')
        .split('\n')
        .slice(1)
        .filter((l) => !l.includes('memtrace'))
        .slice(0, FRAMES)
        .map((l) => l.trim().replace(/^at /, '').replace(/https?:\/\/[^/]+/, '').replace(/\?[^:)]*/, ''))
        .join(' < ');
      let site = sites.get(stack);
      if (!site) sites.set(stack, (site = { bytes: 0, count: 0, live: 0, largest: 0 }));
      site.bytes += bytes;
      site.count++;
      site.live += bytes;
      site.largest = Math.max(site.largest, bytes);
      freed.register(buffer, { site, bytes });
    } finally {
      busy = false;
    }
  };

  const wrap = <C extends abstract new (...args: never[]) => object>(ctor: C, buffer: (made: InstanceType<C>) => ArrayBufferLike): C =>
    new Proxy(ctor, {
      construct(target, args, newTarget) {
        const made = Reflect.construct(target, args, newTarget) as InstanceType<C>;
        // (A view on a buffer that exists already makes nothing new.)
        const first = args[0] as unknown;
        if (!(first instanceof ArrayBuffer) && !(typeof SharedArrayBuffer !== 'undefined' && first instanceof SharedArrayBuffer)) record(buffer(made));
        return made;
      },
    });
  const g = globalThis as unknown as Record<string, unknown>;
  for (const name of ['Float32Array', 'Float64Array', 'Int8Array', 'Uint8Array', 'Uint8ClampedArray', 'Int16Array', 'Uint16Array', 'Int32Array', 'Uint32Array', 'BigInt64Array', 'BigUint64Array']) {
    const ctor = g[name] as (new (...args: never[]) => ArrayBufferView) | undefined;
    if (!ctor) continue;
    const wrapped = wrap(ctor, (made) => made.buffer);
    g[name] = wrapped;
    // (`slice`, `map` and `filter` make their copies through the constructor they find here.)
    Object.defineProperty(ctor.prototype, 'constructor', { value: wrapped, writable: true, configurable: true });
  }
  {
    const ctor = ArrayBuffer;
    const wrapped = wrap(ctor, (made) => made);
    g.ArrayBuffer = wrapped;
    Object.defineProperty(ctor.prototype, 'constructor', { value: wrapped, writable: true, configurable: true });
    const slice = ctor.prototype.slice;
    ctor.prototype.slice = function (this: ArrayBuffer, ...args: Parameters<ArrayBuffer['slice']>) {
      const made = slice.apply(this, args);
      record(made);
      return made;
    };
  }
  for (const owner of [Response.prototype, Blob.prototype]) {
    const read = owner.arrayBuffer;
    owner.arrayBuffer = async function (this: Response & Blob) {
      const made = await read.call(this);
      record(made);
      return made;
    };
  }

  window.__memtrace = (top = 40) => {
    const all = [...sites].map(([stack, s]) => ({ stack, ...s }));
    return {
      sites: all.sort((a, b) => b.live - a.live || b.bytes - a.bytes).slice(0, top),
      bytes: all.reduce((a, s) => a + s.bytes, 0),
      live: all.reduce((a, s) => a + s.live, 0),
      count: all.reduce((a, s) => a + s.count, 0),
    };
  };
}
