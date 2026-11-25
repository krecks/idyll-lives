// `persistence/saves.ts`, its decisions without the browser's storage: which unloadable saves are
// removed, which autosave slot is written next, which game a loaded save goes on as, and what an
// imported file keeps.
import { describe, expect, test } from 'bun:test';
import { gameOnLoading, keptAs, nextAutosaveId, openedSave, parseSaveFile, problemNotice, type SaveMeta } from '../src/persistence/saves';

const meta = (id: string, savedAt: number, game?: string): SaveMeta => ({
  id,
  name: id,
  household: 'Park',
  members: [],
  day: 1,
  minute: 0,
  savedAt,
  thumbnail: null,
  autosave: id.startsWith('autosave:'),
  game,
});

describe('an unloadable save', () => {
  test('from a newer version, or one this version failed to load, is kept; a too old or damaged one is removed', () => {
    expect(keptAs('tooNew')).toBe('tooNewFor');
    expect(keptAs('failed')).toBe('failedIn');
    expect(keptAs('tooOld')).toBeNull();
    expect(keptAs('corrupt')).toBeNull();
  });

  test('the notice says which', () => {
    expect(problemNotice('tooNew', 'A')).toContain('kept');
    expect(problemNotice('failed', 'A')).toContain('kept');
    expect(problemNotice('tooOld', 'A')).toContain('removed');
    expect(problemNotice('corrupt', 'A')).toContain('removed');
  });
});

describe('nextAutosaveId', () => {
  test('fills the empty slot, then writes over the older one', () => {
    expect(nextAutosaveId('g', [])).toBe('autosave:g:0');
    expect(nextAutosaveId('g', [meta('autosave:g:0', 10, 'g')])).toBe('autosave:g:1');
    const both = [meta('autosave:g:0', 10, 'g'), meta('autosave:g:1', 20, 'g')];
    expect(nextAutosaveId('g', both)).toBe('autosave:g:0');
    expect(nextAutosaveId('g', [meta('autosave:g:0', 30, 'g'), meta('autosave:g:1', 20, 'g')])).toBe('autosave:g:1');
    // (Another game's slots are its own.)
    expect(nextAutosaveId('h', both)).toBe('autosave:h:0');
  });
});

describe('gameOnLoading', () => {
  const day5 = meta('manual', 5, 'g');
  const saves = [day5, meta('autosave:g:0', 19, 'g'), meta('autosave:g:1', 20, 'g'), meta('other', 99, 'h')];

  test('the newest save of its game goes on as that game', () => {
    expect(gameOnLoading(saves[2], saves)).toBe('g');
    // (One from before there were games is its own.)
    expect(gameOnLoading(meta('old', 1), [meta('old', 1)])).toBe('old');
  });

  test('an older one starts a game of its own, so the newer autosaves are kept', () => {
    expect(gameOnLoading(day5, saves)).not.toBe('g');
    expect(gameOnLoading(saves[1], saves)).not.toBe('g');
  });
});

describe('parseSaveFile', () => {
  const file = { kind: 'idyll-lives-save', format: 1, ...meta('x', 7, 'g'), data: '{"version":43}' };

  test('makes a new slot of its own game', () => {
    const record = parseSaveFile(JSON.stringify(file));
    expect(record).not.toBeNull();
    expect(record!.id).not.toBe('x');
    expect(record!.game).toBeUndefined();
    expect(record!.data).toBe(file.data);
    expect(record!.savedAt).toBe(7);
  });

  test("refuses what isn't a save file", () => {
    expect(parseSaveFile('not json')).toBeNull();
    expect(parseSaveFile('null')).toBeNull();
    expect(parseSaveFile(JSON.stringify({ ...file, kind: 'other' }))).toBeNull();
    expect(parseSaveFile(JSON.stringify({ ...file, data: 3 }))).toBeNull();
  });
});

/** Just enough of IndexedDB for `saves.ts`: one store, `meta`; each request answers on a later turn, the transaction completes once none is left. */
function fakeMetaStore(rows: Map<string, SaveMeta>) {
  const transaction = () => {
    let pending = 0;
    const tx = {
      oncomplete: null as null | (() => void),
      objectStore: () => ({ get: (id: string) => ask(() => rows.get(id)), put: (v: SaveMeta) => ask(() => void rows.set(v.id, v)) }),
    };
    const ask = (fn: () => unknown) => {
      pending++;
      const req = { result: undefined as unknown, onsuccess: null as null | (() => void) };
      setTimeout(() => {
        req.result = fn();
        req.onsuccess?.();
        if (--pending === 0) setTimeout(() => tx.oncomplete?.());
      });
      return req;
    };
    return tx;
  };
  (globalThis as { indexedDB?: unknown }).indexedDB = {
    open: () => {
      const req = { result: { transaction }, onsuccess: null as null | (() => void) };
      setTimeout(() => req.onsuccess?.());
      return req;
    },
  };
}

describe('openedSave', () => {
  test("clears a set-aside save's mark, and never writes back details another tab has saved since", async () => {
    const rows = new Map<string, SaveMeta>();
    fakeMetaStore(rows);
    const read = meta('autosave:g:0', 10, 'g');
    // (Another tab autosaved into the slot after this one read it.)
    rows.set(read.id, { ...read, day: 5, savedAt: 50 });
    await openedSave(read);
    expect(rows.get(read.id)).toEqual({ ...read, day: 5, savedAt: 50 });
    rows.set(read.id, { ...read, day: 5, savedAt: 50, failedIn: '0.8.3' });
    await openedSave(read);
    expect(rows.get(read.id)).toEqual({ ...read, day: 5, savedAt: 50 });
  });
});
