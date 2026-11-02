// A worker started when first asked and stopped once idle (`core/lazyWorker.ts`, the town and render
// data workers'): answers matched to asks by id; where no worker can start, or once one broke, asks go
// to the page's fallback (those waiting too); an aborted ask the worker is on stops it, and the asks
// behind it go to a fresh one.
import { describe, expect, jest, test } from 'bun:test';
import { LazyWorker, Superseded, type WorkerAnswer, type WorkerAsk } from '../src/core/lazyWorker';

class FakeWorker {
  readonly sent: WorkerAsk<string>[] = [];
  terminated = false;
  onmessage: ((e: { data: WorkerAnswer<string> }) => void) | null = null;
  onerror: ((e: { message: string; preventDefault(): void }) => void) | null = null;
  postMessage(message: WorkerAsk<string>): void {
    this.sent.push(message);
  }
  terminate(): void {
    this.terminated = true;
  }
  answer(a: WorkerAnswer<string>): void {
    this.onmessage?.({ data: a });
  }
}

function lazy(start: () => FakeWorker = () => new FakeWorker()) {
  const workers: FakeWorker[] = [];
  const broke: string[] = [];
  const w = new LazyWorker<string, string>({
    start: () => {
      const made = start();
      workers.push(made);
      return made as unknown as Worker;
    },
    idleMs: 1000,
    broke: (why) => broke.push(why),
  });
  return { w, workers, broke };
}

const page = (what: string) => () => Promise.resolve(`page ${what}`);

describe('lazy worker', () => {
  test('started on the first ask, answers by id, stopped once idle', async () => {
    jest.useFakeTimers();
    try {
      const { w, workers } = lazy();
      const a = w.ask('a', page('a'));
      const b = w.ask('b', page('b'));
      expect(workers.length).toBe(1);
      const [worker] = workers;
      expect(worker.sent.map((s) => s.message)).toEqual(['a', 'b']);
      worker.answer({ id: worker.sent[1].id, answer: 'B' });
      worker.answer({ id: worker.sent[0].id, error: 'no' });
      expect(await b).toBe('B');
      await expect(a).rejects.toThrow('no');
      jest.advanceTimersByTime(999);
      expect(worker.terminated).toBe(false);
      jest.advanceTimersByTime(1);
      expect(worker.terminated).toBe(true);
      expect(w.run).toBe(0);
    } finally {
      jest.useRealTimers();
    }
  });

  test('a hold keeps it running while idle', () => {
    jest.useFakeTimers();
    try {
      const { w, workers } = lazy();
      const run = w.post('open');
      const release = w.hold();
      expect(run).toBe(1);
      jest.advanceTimersByTime(5000);
      expect(workers[0].terminated).toBe(false);
      release();
      jest.advanceTimersByTime(1000);
      expect(workers[0].terminated).toBe(true);
    } finally {
      jest.useRealTimers();
    }
  });

  test('no worker can start: every ask goes to the page', async () => {
    const { w } = lazy(() => {
      throw new Error('no workers here');
    });
    expect(await w.ask('a', page('a'))).toBe('page a');
    expect(w.post('open')).toBe(0);
  });

  test('a broken worker hands the asks waiting, and later ones, to the page', async () => {
    const { w, workers, broke } = lazy();
    const a = w.ask('a', page('a'));
    const b = w.ask('b', page('b'));
    workers[0].answer({ id: workers[0].sent[0].id, error: 'no content', broken: true });
    expect([await a, await b]).toEqual(['page a', 'page b']);
    expect(broke).toEqual(['no content']);
    expect(workers[0].terminated).toBe(true);
    expect(await w.ask('c', page('c'))).toBe('page c');
    expect(workers.length).toBe(1);
  });

  test('aborting the ask it is on starts a fresh worker for those behind; one waiting just goes', async () => {
    const { w, workers } = lazy();
    const first = new AbortController();
    const waiting = new AbortController();
    const a = w.ask('a', page('a'), { signal: first.signal });
    const b = w.ask('b', page('b'), { signal: waiting.signal });
    const c = w.ask('c', page('c'));
    waiting.abort();
    await expect(b).rejects.toBeInstanceOf(Superseded);
    expect(workers.length).toBe(1);
    first.abort();
    await expect(a).rejects.toBeInstanceOf(Superseded);
    expect(workers[0].terminated).toBe(true);
    expect(workers.length).toBe(2);
    const resent = workers[1].sent;
    expect(resent.map((s) => s.message)).toEqual(['c']);
    workers[1].answer({ id: resent[0].id, answer: 'C' });
    expect(await c).toBe('C');
    // (Already aborted: never sent.)
    const gone = new AbortController();
    gone.abort();
    await expect(w.ask('d', page('d'), { signal: gone.signal })).rejects.toBeInstanceOf(Superseded);
  });
});
