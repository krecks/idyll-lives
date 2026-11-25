// The language model's worker lost its graphics device (`llm/llm.worker.ts` says so with `restart`): the service
// stops it and starts a new one on the same device (`llm/service.svelte.ts`). No model needed.
import { expect, test } from 'bun:test';
import { startModels } from '../src/ml/schedule.svelte';

// Svelte's `$state` outside a compiled component: the plain object is enough here.
(globalThis as { $state?: unknown }).$state = <T>(v: T) => v;
const { ensureLlm, llmStatus, unloadLlm } = await import('../src/llm/service.svelte');

class FakeWorker {
  static made: FakeWorker[] = [];
  readonly sent: unknown[] = [];
  terminated = false;
  onmessage: ((e: { data: unknown }) => void) | null = null;
  onerror: unknown = null;
  constructor() {
    FakeWorker.made.push(this);
  }
  postMessage(m: unknown): void {
    this.sent.push(m);
  }
  terminate(): void {
    this.terminated = true;
  }
}

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

test('a lost graphics device starts the model again on the same device', async () => {
  const realWorker = globalThis.Worker;
  globalThis.Worker = FakeWorker as unknown as typeof Worker;
  try {
    startModels();
    void ensureLlm('gpu').catch(() => {});
    await tick();
    const [first] = FakeWorker.made;
    expect(first.sent).toEqual([expect.objectContaining({ type: 'load', device: 'gpu' })]);
    first.onmessage!({ data: { type: 'error', message: 'graphics device lost', restart: true } });
    await tick();
    expect(first.terminated).toBe(true);
    expect(FakeWorker.made.length).toBe(2);
    expect(FakeWorker.made[1].sent).toEqual([expect.objectContaining({ type: 'load', device: 'gpu' })]);
    expect(llmStatus.state).toBe('loading');
  } finally {
    unloadLlm();
    globalThis.Worker = realWorker;
  }
});
