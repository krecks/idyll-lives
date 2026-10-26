/**
 * Resolves asset keys to entries. Manifests stack: later packs override earlier keys,
 * so new art can be dropped in without changing code or the base manifest.
 *
 * Production builds also ship compressed copies of the base assets (KTX2 textures, meshopt
 * models, gzipped character data; see `tools/assets/optimize.mjs`) and an index of them,
 * `optimized.json` next to the base manifest. Loaders ask `optimizedUrl` for the file to fetch;
 * files without a copy (the dev server, a modder's pack) load as they are.
 */

import { withBuild, withoutBuild } from './build';
import { bundled } from './bundle';
import type { AssetEntry, AssetType, EntryOf, IconEntry, Manifest } from './types';

const URL_FIELDS = ['url', 'far', 'texture'] as const;

export class AssetRegistry {
  private readonly entries = new Map<string, AssetEntry>();
  private readonly warned = new Set<string>();

  static async load(manifestUrl: string, extraPacks: readonly string[] = []): Promise<AssetRegistry> {
    const registry = new AssetRegistry();
    const base = new URL(manifestUrl, location.href);
    // Every manifest downloads at once; entries apply in order (later packs override earlier keys).
    const [manifests] = await Promise.all([
      Promise.all([fetchManifests(base), ...extraPacks.map((pack) => fetchManifests(new URL(pack, base)))]),
      loadOptimized(base),
    ]);
    for (const [url, manifest] of manifests.flat()) {
      for (const [key, entry] of Object.entries(manifest.entries)) registry.entries.set(key, extended(registry.entries.get(key), resolveUrls(entry, url)));
    }
    // A model without a file draws nothing (a placed object, a box): said once per key, here.
    for (const [key, entry] of registry.entries) if (entry.type === 'model' && !entry.url) console.warn(`[assets] '${key}' has no model file`);
    return registry;
  }

  get<T extends AssetType>(key: string, type: T): EntryOf<T> | undefined {
    const entry = this.entries.get(key);
    if (entry?.type === type) return entry as EntryOf<T>;
    this.warnOnce(key, entry ? `is a ${entry.type}, expected ${type}` : 'is missing');
    return undefined;
  }

  /** Whether `key` exists with type `type` (no warning; for optional variants such as `model.sofa@modern`). */
  has(key: string, type: AssetType): boolean {
    return this.entries.get(key)?.type === type;
  }

  /** Every key with type `type` (the dev benches list models with it). */
  keys(type: AssetType): string[] {
    return [...this.entries].flatMap(([key, entry]) => (entry.type === type ? [key] : []));
  }

  /** Icons never fail: a missing key yields a neutral placeholder so the UI stays intact. */
  icon(key: string): IconEntry {
    return this.get(key, 'icon') ?? { type: 'icon', url: FALLBACK_ICON, mode: 'mask' };
  }

  /** Says `problem` about `key` once (a lookup made every frame would otherwise fill the console). */
  private warnOnce(key: string, problem: string): void {
    if (this.warned.has(key)) return;
    this.warned.add(key);
    console.warn(`[assets] '${key}' ${problem}`);
  }
}

/** A manifest followed by the packs it lists (depth first, in order), fetched in parallel. */
async function fetchManifests(url: URL): Promise<[URL, Manifest][]> {
  const manifest = await readJson<Manifest>(url, (res) => {
    if (!res.ok) throw new Error(`failed to load asset manifest ${url}: ${res.status}`);
  });
  const packs = await Promise.all((manifest.packs ?? []).map((pack) => fetchManifests(new URL(pack, url))));
  return [[url, manifest], ...packs.flat()];
}

/** Absolute source URL -> absolute URL of its compressed copy (both without the build query). */
const optimized = new Map<string, string>();

/** Reads the build's index of compressed copies (there is none on the dev server). */
async function loadOptimized(base: URL): Promise<void> {
  try {
    const index = await readJson<{ files?: Record<string, string> }>(new URL('optimized.json', base), (res) => {
      // (The dev server answers a missing file with its index page.)
      if (!res.ok || !res.headers.get('content-type')?.includes('json')) throw new Error('no index');
    });
    for (const [file, copy] of Object.entries(index.files ?? {})) optimized.set(new URL(file, base).href, new URL(copy, base).href);
  } catch {
    // No index: everything loads from its source file.
  }
}

/** A JSON file from the build's bundle (`bundle.ts`), else fetched; `check` vets the response. */
async function readJson<T>(url: URL, check: (res: Response) => void): Promise<T> {
  const text = await bundled(url.href);
  if (text !== null) return JSON.parse(text) as T;
  const res = await fetch(withBuild(url.href));
  check(res);
  return (await res.json()) as T;
}

/** The file to fetch for an asset URL: its compressed copy when the build made one. */
export function optimizedUrl(url: string): string {
  const copy = optimized.get(withoutBuild(new URL(url, location.href).href));
  return copy ? withBuild(copy) : url;
}

/**
 * A pack's model entry over the one it overrides: it keeps that one's `vary` unless it gives its
 * own, so packs needn't copy it (or let it go stale).
 */
function extended(below: AssetEntry | undefined, entry: AssetEntry): AssetEntry {
  if (below?.type !== 'model' || entry.type !== 'model') return entry;
  return { ...entry, ...((entry.vary ?? below.vary) !== undefined ? { vary: entry.vary ?? below.vary } : {}) };
}

/** The file references an entry has: every kind of entry has one or two of `URL_FIELDS`. */
type UrlFields = Partial<Record<(typeof URL_FIELDS)[number], string>>;

/** Makes file references absolute relative to the manifest that declared them, with this build's query. */
function resolveUrls(entry: AssetEntry, base: URL): AssetEntry {
  const out = { ...entry } as AssetEntry & UrlFields;
  for (const field of URL_FIELDS) {
    const url = out[field];
    // (A hand-written pack can put anything there: `null` would become a file called `null`.)
    if (typeof url === 'string') out[field] = withBuild(new URL(url, base).href);
  }
  return out;
}

const FALLBACK_ICON =
  'data:image/svg+xml,' +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="7" fill="none" stroke="#000" stroke-width="2"/></svg>',
  );
