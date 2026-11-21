/**
 * The main menu's picture: a still of the last world made (no town is built for the menu; New
 * game makes the world). Taken by `menuScene.keepStill` from the menu's own shot (golden hour, the
 * town right of the menu panel) and kept in the Cache API, from build to build. A first visit (or
 * a browser that keeps no cache) gets the one shipped with the game (`FIRST_STILL`).
 */
import { openCache } from '../ml/assets';

/** A picture for the menu and the town it shows (the badge's name). */
export interface MenuStill {
  url: string;
  name: string;
}

const PREFIX = 'littlelives-menu-still-';
/** Bumped if what is kept stops matching the code that reads it. */
const CACHE = `${PREFIX}v1`;
/** The cache's one entry: a URL of its own (Cache Storage needs one; nothing is fetched from it). */
const ENTRY = new URL('/menu-still.jpg', typeof location === 'undefined' ? 'https://idyll.invalid/' : location.href).href;
/** The town's name, with the picture. */
const NAME = 'X-Town-Name';

/** The picture shipped with the game (`tools/bench/menu-still.mjs` takes it) and its town. */
export const FIRST_STILL: MenuStill = { url: `${import.meta.env.BASE_URL}menu/still.jpg`, name: 'Elmview' };

/** The entry a still is kept as. */
export function stillEntry(picture: Blob, name: string): Response {
  return new Response(picture, { headers: { 'Content-Type': picture.type || 'image/jpeg', [NAME]: encodeURIComponent(name) } });
}

/** The picture and name an entry holds, or null (not a picture, no name: not one of ours). */
export async function readStill(entry: Response): Promise<{ picture: Blob; name: string } | null> {
  const name = entry.headers.get(NAME);
  if (!name || !(entry.headers.get('Content-Type') ?? '').startsWith('image/')) return null;
  const picture = await entry.blob();
  return picture.size ? { picture, name: decodeURIComponent(name) } : null;
}

/** The still kept from the last world made (an object URL), or null (none yet, or not readable). */
export async function loadStill(): Promise<MenuStill | null> {
  if (typeof caches === 'undefined') return null;
  try {
    const cache = await openCache(PREFIX, CACHE);
    const hit = await cache?.match(ENTRY);
    const kept = hit ? await readStill(hit) : null;
    return kept ? { url: URL.createObjectURL(kept.picture), name: kept.name } : null;
  } catch {
    return null;
  }
}

/** Keeps `picture` (of the town `name`) for the menu, this visit's later ones and the next. */
export async function keepStill(picture: Blob, name: string): Promise<void> {
  if (typeof caches === 'undefined') return;
  try {
    const cache = await openCache(PREFIX, CACHE);
    await cache?.put(ENTRY, stillEntry(picture, name));
  } catch (err) {
    console.warn('[menu] could not keep its picture', err);
  }
}

/** What 0.8's menu kept of its showcase town (a 7 MB cache and its seed): gone, once. */
export async function forgetShowcase(): Promise<void> {
  try {
    localStorage.removeItem('littlelives.showcase');
  } catch {
    // (No storage: nothing kept either.)
  }
  if (typeof caches === 'undefined') return;
  try {
    for (const name of await caches.keys()) if (name.startsWith('littlelives-showcase-')) await caches.delete(name);
  } catch {
    // (Only space.)
  }
}
