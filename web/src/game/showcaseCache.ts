/**
 * The main menu's showcase, kept from visit to visit: its land, the houses it generated and the menus' preview are the same for a build, so a
 * player who comes back sees the same valley again without waiting for the town worker to lay it
 * out. The seed is kept with the build's id: a new build makes a new valley, and only its cache is
 * kept (an older build's is dropped).
 *
 * Kept as JSON in the Cache API (`ml/assets.ts` `openCache`): 7 MB for a Medium world, a few
 * hundredths of a second to read, against seconds to make. `keepShowcase` is called in the town worker
 * (the preview job), so the stringifying and the write cost the page nothing; a browser that keeps
 * no cache simply makes the town again next visit.
 */
import type { WorldStructure } from '../core/protocol';
import { BUILD_ID as BUILD } from '../assets/build';
import { openCache } from '../ml/assets';
import type { HouseTemplate, NeighbourhoodDraft } from './town';
import { EMPTY_MESH } from './townPreview';

const PREFIX = 'littlelives-showcase-';
const SEED_KEY = 'littlelives.showcase';
/** Bumped if what is kept stops matching the code that reads it (a build's id already names the cache). */
const VERSION = 1;

const CACHE = `${PREFIX}${BUILD}`;

/** The cache's one entry: a URL of its own (Cache Storage needs one; nothing is fetched from it). */
const ENTRY = new URL('/showcase-cache.json', typeof location === 'undefined' ? 'https://idyll.invalid/' : location.href).href;

export interface Showcase {
  /** The town as the menu shows it, people and all (the same pick of neighbours every visit). */
  draft: NeighbourhoodDraft;
  /** The houses the land was made with (no residents), as the town worker generated them. */
  houses: [string, HouseTemplate | undefined][];
  /** The menus' 3D preview of `draft`. */
  preview: WorldStructure;
}

/** What is kept: the preview without its (empty) meshes, which JSON can't carry. */
type Kept = Omit<Showcase, 'preview'> & { version: number; preview: Omit<WorldStructure, 'meshes'> };

/** The entry a showcase is kept as (JSON: a house the land didn't generate goes as null). */
export function keepEntry(showcase: Showcase): Kept {
  const { meshes: _meshes, ...preview } = showcase.preview;
  return { version: VERSION, ...showcase, preview };
}

/** The showcase an entry holds, or null (another build's, an older version's, not one of ours). */
export function readEntry(kept: unknown): Showcase | null {
  const entry = kept as Kept | null;
  if (entry?.version !== VERSION || !entry.draft || !entry.preview) return null;
  const houses = (entry.houses ?? []) as [string, HouseTemplate | null | undefined][];
  return {
    draft: entry.draft,
    houses: houses.map(([id, house]) => [id, house ?? undefined]),
    preview: { ...entry.preview, meshes: { walls: EMPTY_MESH, wallsLow: EMPTY_MESH, floors: EMPTY_MESH } },
  };
}

/** The showcase kept for this build, or null (none yet, another build's, or not readable). */
export async function loadShowcase(): Promise<Showcase | null> {
  if (typeof caches === 'undefined') return null;
  try {
    const cache = await openCache(PREFIX, CACHE);
    const hit = await cache?.match(ENTRY);
    if (!hit) return null;
    return readEntry(await hit.json());
  } catch {
    return null;
  }
}

/** Keeps `showcase` for this build's later visits (in the town worker, where the cost is off the page). */
export async function keepShowcase(showcase: Showcase): Promise<void> {
  if (typeof caches === 'undefined') return;
  try {
    // (Stringified now, before any await: on the page, where no worker runs, the draft may change meanwhile.)
    const body = JSON.stringify(keepEntry(showcase));
    const cache = await openCache(PREFIX, CACHE);
    if (!cache) return;
    await cache.put(ENTRY, new Response(body, { headers: { 'Content-Type': 'application/json' } }));
  } catch (err) {
    console.warn('[showcase] could not keep it', CACHE, ENTRY, err);
  }
}

/** The seed this build's showcase is laid out from, kept from visit to visit (a new build: a new valley). */
export function showcaseSeed(): number {
  const roll = () => (Math.random() * 2 ** 31) >>> 0;
  try {
    const saved = JSON.parse(localStorage.getItem(SEED_KEY) ?? 'null') as { build?: string; seed?: number } | null;
    if (saved?.build === BUILD && typeof saved.seed === 'number') return saved.seed >>> 0;
    const seed = roll();
    localStorage.setItem(SEED_KEY, JSON.stringify({ build: BUILD, seed }));
    return seed;
  } catch {
    return roll();
  }
}
