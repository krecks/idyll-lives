/**
 * Save slots in IndexedDB (browser storage, per device). One record per slot in `saves`, its
 * `data` field the simulation's own save format (never parsed here); the slot's details alone
 * are kept in `meta` too, so lists (the menu, Load game) don't read every game.
 */

import type { SaveProblem } from '../core/protocol';

export interface SaveMeta {
  id: string;
  /** Player-facing slot name. */
  name: string;
  household: string;
  members: string[];
  day: number;
  minute: number;
  savedAt: number;
  /** Small JPEG data URL, if one could be captured. */
  thumbnail: string | null;
  autosave: boolean;
  /**
   * The game the save belongs to: made at New game and carried through loading and saving
   * (its autosaves rotate together). Saves from before there were games are each their own.
   */
  game?: string;
  /** This version of the game (`__APP_VERSION__`) found the save to be from a newer one: kept, not offered. */
  tooNewFor?: string;
  /** This version of the game failed while loading the save: kept (it can be exported), not offered; a later version tries again. */
  failedIn?: string;
}

export interface SaveRecord extends SaveMeta {
  data: string;
}

/** A save the game can't open: `message` says so in plain words (and whether it was kept). */
export class UnloadableSave extends Error {
  constructor(
    readonly saveId: string,
    readonly problem: SaveProblem,
    message: string,
  ) {
    super(message);
  }
}

// Legacy key from the project's old name; kept so saves survive.
const DB_NAME = 'open-sims-wasm';
/** 1: whole records in `saves`; 2: their details also in `meta`. */
const DB_VERSION = 2;
const STORE = 'saves';
const META = 'meta';

let dbPromise: Promise<IDBDatabase> | null = null;

function db(): Promise<IDBDatabase> {
  if (dbPromise) return dbPromise;
  const opening = new Promise<IDBDatabase>((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onupgradeneeded = () => upgrade(req.result, req.transaction!);
    req.onsuccess = () => {
      const conn = req.result;
      // The browser closed it (storage cleared), or another tab needs a newer version: open again next time.
      const lost = () => {
        if (dbPromise === opening) dbPromise = null;
      };
      conn.onclose = lost;
      conn.onversionchange = () => {
        conn.close();
        lost();
      };
      resolve(conn);
    };
    req.onerror = () => reject(req.error ?? new Error('could not open save storage'));
    // An open tab of an older version holds the saves (it never lets go): ask again next time.
    req.onblocked = () => reject(new Error('Your saves are open in another tab of Idyll Lives. Close it to see them here.'));
  });
  dbPromise = opening;
  opening.catch(() => {
    if (dbPromise === opening) dbPromise = null;
  });
  return opening;
}

/** Creates the stores; from version 1, copies every record's details into `meta` (the records stay as they are). */
function upgrade(conn: IDBDatabase, tx: IDBTransaction): void {
  if (!conn.objectStoreNames.contains(STORE)) conn.createObjectStore(STORE, { keyPath: 'id' });
  if (conn.objectStoreNames.contains(META)) return;
  const meta = conn.createObjectStore(META, { keyPath: 'id' });
  tx.objectStore(STORE).openCursor().onsuccess = (e) => {
    const cursor = (e.target as IDBRequest<IDBCursorWithValue | null>).result;
    if (!cursor) return;
    meta.put(metaOf(cursor.value as SaveRecord));
    cursor.continue();
  };
}

function metaOf(record: SaveRecord): SaveMeta {
  const { data: _data, ...meta } = record;
  return meta;
}

/**
 * Runs `op` in one transaction over `stores` and resolves with its request's result once the
 * transaction has committed (a write that fails to commit, e.g. storage full, rejects).
 */
async function run<T>(mode: IDBTransactionMode, stores: string[], op: (tx: IDBTransaction) => IDBRequest<T> | void, retry = true): Promise<T> {
  const conn = await db();
  let tx: IDBTransaction;
  try {
    tx = conn.transaction(stores, mode);
  } catch (err) {
    // The connection closed under us: open it again, once.
    if (dbPromise) dbPromise = null;
    if (retry) return run(mode, stores, op, false);
    throw err;
  }
  return new Promise<T>((resolve, reject) => {
    const req = op(tx);
    tx.oncomplete = () => resolve(req ? req.result : (undefined as T));
    tx.onerror = () => reject(tx.error ?? req?.error ?? new Error('save storage error'));
    tx.onabort = () => reject(tx.error ?? new Error('saving was cancelled'));
  });
}

/** Saves found this session to be unloadable: not offered to Continue or prepared again. */
const refused = new Set<string>();
const listeners = new Set<() => void>();

/** Calls `fn` whenever saves were written, removed or set aside. Returns an unsubscribe function. */
export function onSavesChanged(fn: () => void): () => void {
  listeners.add(fn);
  return () => listeners.delete(fn);
}

function changed(): void {
  for (const fn of listeners) fn();
}

/** Newest first, without the (large) game data. */
export async function listSaves(): Promise<SaveMeta[]> {
  const all = await run('readonly', [META], (tx) => tx.objectStore(META).getAll() as IDBRequest<SaveMeta[]>);
  return all.sort((a, b) => b.savedAt - a.savedAt);
}

export function readSave(id: string): Promise<SaveRecord | undefined> {
  return run('readonly', [STORE], (tx) => tx.objectStore(STORE).get(id) as IDBRequest<SaveRecord | undefined>);
}

export async function writeSave(record: SaveRecord): Promise<void> {
  keepStorage();
  await run('readwrite', [STORE, META], (tx) => {
    tx.objectStore(STORE).put(record);
    tx.objectStore(META).put(metaOf(record));
  });
  refused.delete(record.id);
  changed();
}

export async function deleteSave(id: string): Promise<void> {
  await run('readwrite', [STORE, META], (tx) => {
    tx.objectStore(STORE).delete(id);
    tx.objectStore(META).delete(id);
  });
  changed();
}

/** Whether Continue may offer the save (it isn't known to be unloadable). */
export function playable(save: SaveMeta): boolean {
  return !refused.has(save.id) && save.tooNewFor !== __APP_VERSION__ && save.failedIn !== __APP_VERSION__;
}

/** The newest save Continue can open. */
export async function latestSave(): Promise<SaveMeta | undefined> {
  return (await listSaves()).find(playable);
}

/** Each game keeps its two newest autosaves: the slot to write next is the empty or the older one. */
export async function nextAutosaveId(game: string): Promise<string> {
  const slots = [0, 1].map((i) => `autosave:${game}:${i}`);
  const savedAt = new Map((await listSaves()).map((s) => [s.id, s.savedAt]));
  return slots.reduce((a, b) => ((savedAt.get(b) ?? 0) < (savedAt.get(a) ?? 0) ? b : a));
}

/**
 * Sets aside a save the game couldn't open: one from a newer version, or one the simulation failed
 * to load, is kept (and no longer offered by this version); a too old or damaged one is removed.
 * Returns the notice for players.
 */
export async function refuseSave(save: SaveMeta, problem: SaveProblem): Promise<string> {
  refused.add(save.id);
  try {
    if (problem === 'tooNew') await run('readwrite', [META], (tx) => tx.objectStore(META).put({ ...metaOf(save as SaveRecord), tooNewFor: __APP_VERSION__ }));
    else if (problem === 'failed') await run('readwrite', [META], (tx) => tx.objectStore(META).put({ ...metaOf(save as SaveRecord), failedIn: __APP_VERSION__ }));
    else await deleteSave(save.id);
  } catch (err) {
    console.warn('[saves] could not set aside an unloadable save', err);
  }
  changed();
  return problemNotice(problem, save.name);
}

function problemNotice(problem: SaveProblem, name: string): string {
  if (problem === 'tooNew') return `“${name}” is from a newer version of Idyll Lives, so this version can't open it. It's been kept.`;
  if (problem === 'tooOld') return `“${name}” is from a version of Idyll Lives too old to open, so it was removed.`;
  if (problem === 'failed') return `This version of Idyll Lives couldn't open “${name}”. It's been kept: you can export it from Load game.`;
  return `“${name}” is damaged and can't be opened, so it was removed.`;
}

/** What's wrong with a save file someone tried to import. */
export function importProblemText(problem: SaveProblem): string {
  if (problem === 'tooNew') return "This save is from a newer version of Idyll Lives, so this version can't open it.";
  if (problem === 'tooOld') return 'This save is from a version of Idyll Lives too old to open.';
  if (problem === 'failed') return "This version of Idyll Lives couldn't open this save.";
  return "This save is damaged and can't be opened.";
}

/** Marks an exported save file (and its format). */
const FILE_KIND = 'idyll-lives-save';

/** A save as a file to keep elsewhere: the whole record, as JSON. */
export async function exportSave(id: string): Promise<{ name: string; text: string }> {
  const record = await readSave(id);
  if (!record) throw new Error('That save no longer exists.');
  const name = `${record.name.replace(/[^\p{L}\p{N} ._-]+/gu, ' ').trim() || 'save'}.json`;
  return { name, text: JSON.stringify({ kind: FILE_KIND, format: 1, ...record }) };
}

/**
 * Reads an exported save file into a new record (a new slot, so nothing is overwritten); null if
 * the file isn't one. Whether the game can open it is for the simulation to say.
 */
export function parseSaveFile(text: string): SaveRecord | null {
  let file: Record<string, unknown>;
  try {
    file = JSON.parse(text);
  } catch {
    return null;
  }
  if (!file || typeof file !== 'object' || file.kind !== FILE_KIND || typeof file.data !== 'string') return null;
  const str = (v: unknown, fallback: string) => (typeof v === 'string' ? v : fallback);
  const num = (v: unknown, fallback: number) => (typeof v === 'number' && Number.isFinite(v) ? v : fallback);
  return {
    id: crypto.randomUUID(),
    name: str(file.name, 'Imported save').slice(0, 80),
    household: str(file.household, ''),
    members: Array.isArray(file.members) ? file.members.filter((m): m is string => typeof m === 'string') : [],
    day: num(file.day, 1),
    minute: num(file.minute, 0),
    savedAt: num(file.savedAt, Date.now()),
    // Only a picture carried in the file itself.
    thumbnail: typeof file.thumbnail === 'string' && file.thumbnail.startsWith('data:image/') ? file.thumbnail : null,
    autosave: false,
    game: typeof file.game === 'string' ? file.game : undefined,
    data: file.data,
  };
}

let storageKept = false;

/** Asks the browser once to keep the saves when it runs short of space (granted silently, or not). */
function keepStorage(): void {
  if (storageKept) return;
  storageKept = true;
  navigator.storage?.persist?.().catch(() => false);
}
