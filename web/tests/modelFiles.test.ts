/**
 * A renderer's model files (`render/shared/modelFiles.ts`): each imported once; one nothing built
 * uses is kept among the last few such files and then let go of (`evict`), never while a model built
 * from it is about, and imported again when it's wanted again; `settled` waits for the imports.
 */
import { describe, expect, test } from 'bun:test';
import { ModelFiles } from '../src/render/shared/modelFiles';

function files(limit: number) {
  const imported: string[] = [];
  const evicted: string[] = [];
  const f = new ModelFiles<{ url: string }>(
    async (url) => {
      imported.push(url);
      return { url };
    },
    () => false,
    (file) => evicted.push(file.url),
    limit,
  );
  return { f, imported, evicted };
}

/** Lets the imports' handlers run (they're promise callbacks). */
const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

describe('model files', () => {
  test('a file is imported once while it is kept', async () => {
    const { f, imported } = files(2);
    const [a, b] = await Promise.all([f.load('a'), f.load('a')]);
    expect(a).toBe(b);
    expect(imported).toEqual(['a']);
  });

  test('files nothing uses go, the least recently wanted first; files in use stay', async () => {
    const { f, imported, evicted } = files(2);
    for (const url of ['a', 'b', 'c']) {
      await f.load(url);
      f.use(url);
    }
    // Nothing is idle yet: nothing goes.
    await settle();
    expect(evicted).toEqual([]);
    f.release('a');
    f.release('b');
    // `a` wanted again (a build of it) before `c` is let go of: `b` is the least recently wanted.
    await f.load('a');
    f.release('c');
    await settle();
    expect(evicted).toEqual(['b']);
    // Wanted again after it went: imported again.
    await f.load('b');
    expect(imported).toEqual(['a', 'b', 'c', 'b']);
  });

  test('a file used twice stays until both let go of it', async () => {
    const { f, evicted } = files(0);
    await f.load('a');
    f.use('a');
    f.use('a');
    f.release('a');
    await settle();
    expect(evicted).toEqual([]);
    expect(f.has('a')).toBe(true);
    f.release('a');
    await settle();
    expect(evicted).toEqual(['a']);
    expect(f.has('a')).toBe(false);
  });

  test('a file read ahead of time is idle until a model is built from it', async () => {
    const { f, evicted } = files(1);
    await f.load('a', false);
    await f.load('b', false);
    await settle();
    expect(evicted).toEqual(['a']);
    f.use('b');
    await f.load('c', false);
    await settle();
    expect(evicted).toEqual(['a']);
  });

  test('settled waits for every import under way', async () => {
    let finish!: () => void;
    const f = new ModelFiles<string>(
      (url) => new Promise((resolve) => (finish = () => resolve(url))),
      () => false,
    );
    void f.load('slow');
    let done = false;
    const settled = f.settled().then(() => (done = true));
    await settle();
    expect(done).toBe(false);
    finish();
    await settled;
    expect(done).toBe(true);
  });
});
