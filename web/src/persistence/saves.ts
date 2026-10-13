/**
 * Save slots in IndexedDB (browser storage, per device). One record per slot; the
 * `data` field is the simulation's own save format and is never parsed here.
 */

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
}

export interface SaveRecord extends SaveMeta {
  data: string;
}

export const AUTOSAVE_ID = 'autosave';

// Legacy key from the project's old name; kept so saves survive.
const DB_NAME = 'open-sims-wasm';
const STORE = 'saves';

let dbPromise: Promise<IDBDatabase> | null = null;

function db(): Promise<IDBDatabase> {
  if (dbPromise) return dbPromise;
  const opening = new Promise<IDBDatabase>((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, 1);
    req.onupgradeneeded = () => req.result.createObjectStore(STORE, { keyPath: 'id' });
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
  });
  dbPromise = opening;
  opening.catch(() => {
    if (dbPromise === opening) dbPromise = null;
  });
  return opening;
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

/** Newest first, without the (large) game data. */
export async function listSaves(): Promise<SaveMeta[]> {
  const all = await run('readonly', [STORE], (tx) => tx.objectStore(STORE).getAll() as IDBRequest<SaveRecord[]>);
  return all.map(({ data: _data, ...meta }) => meta).sort((a, b) => b.savedAt - a.savedAt);
}

export function readSave(id: string): Promise<SaveRecord | undefined> {
  return run('readonly', [STORE], (tx) => tx.objectStore(STORE).get(id) as IDBRequest<SaveRecord | undefined>);
}

export async function writeSave(record: SaveRecord): Promise<void> {
  await run('readwrite', [STORE], (tx) => {
    tx.objectStore(STORE).put(record);
  });
}

export async function deleteSave(id: string): Promise<void> {
  await run('readwrite', [STORE], (tx) => {
    tx.objectStore(STORE).delete(id);
  });
}

export async function latestSave(): Promise<SaveMeta | undefined> {
  return (await listSaves())[0];
}
