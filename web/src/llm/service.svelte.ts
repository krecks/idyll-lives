/**
 * The language model for the whole app: one worker with LFM2.5 350M on the GPU, started and
 * loaded by `ensureLlm` (when the game starts, if residents' lines are the model's), and a
 * status for settings screens. Replies are written one after another; each has a deadline
 * after which it's no longer wanted.
 *
 * Knows nothing about the game: callers pass chat messages and get text back (`talk/` writes
 * residents' lines with it).
 */

import type { ChatMessage } from './chat';
import type { GenParams, GenResult } from './generate';
import { MODEL } from './models';
import type { FromLlmWorker, ToLlmWorker } from './protocol';

export type LlmState = 'off' | 'loading' | 'ready' | 'error';

// Models players added from Hugging Face (until 2026-10-12): no longer offered.
try {
  localStorage.removeItem('littlelives.llmModels');
} catch {
  // Storage unavailable.
}

export const llmStatus = $state({
  state: 'off' as LlmState,
  /** Download progress 0..1 while loading. */
  progress: 0,
  error: '',
});

/** How the LLM worker is doing (for the debug overlay). */
export interface LlmStats {
  state: LlmState;
  model: string;
  loadMs: number;
  /** Replies written, and those skipped (late or aborted). */
  replies: number;
  skipped: number;
  queued: number;
  /** Time to read the last prompt, ms, and the share of its tokens whose state was cached. */
  lastPrefillMs: number;
  lastCachedShare: number;
  /** Tokens per second writing, last reply and on average. */
  lastTokPerS: number;
  avgTokPerS: number;
  /** Share of the last few seconds the worker spent writing, 0..1. */
  busy: number;
}

let worker: Worker | null = null;
let nextId = 1;
const pending = new Map<number, { resolve: (r: GenResult | null) => void; reject: (e: Error) => void }>();
let waiters: { resolve: () => void; reject: (e: Error) => void }[] = [];
const BUSY_WINDOW_MS = 5000;
const counters = { loadMs: 0, replies: 0, skipped: 0, lastPrefillMs: 0, lastCachedShare: 0, lastTokPerS: 0, tokens: 0, decodeMs: 0, recent: [] as { at: number; ms: number }[] };

function post(msg: ToLlmWorker): void {
  worker?.postMessage(msg);
}

function onMessage(msg: FromLlmWorker): void {
  if (msg.type === 'progress') {
    llmStatus.progress = msg.total ? msg.loaded / msg.total : 0;
  } else if (msg.type === 'ready') {
    counters.loadMs = msg.ms;
    llmStatus.progress = 1;
    llmStatus.state = 'ready';
    waiters.forEach((w) => w.resolve());
    waiters = [];
  } else if (msg.type === 'done') {
    const r = msg.result;
    counters.replies++;
    counters.lastPrefillMs = r.prefillMs;
    counters.lastCachedShare = r.promptTokens ? r.cachedTokens / r.promptTokens : 0;
    counters.lastTokPerS = r.decodeMs > 0 ? r.ids.length / (r.decodeMs / 1000) : 0;
    counters.tokens += r.ids.length;
    counters.decodeMs += r.decodeMs;
    const now = performance.now();
    counters.recent.push({ at: now, ms: r.prefillMs + r.decodeMs });
    while (counters.recent.length && counters.recent[0].at < now - BUSY_WINDOW_MS) counters.recent.shift();
    pending.get(msg.id)?.resolve(r);
    pending.delete(msg.id);
  } else if (msg.type === 'skipped') {
    counters.skipped++;
    pending.get(msg.id)?.resolve(null);
    pending.delete(msg.id);
  } else if (msg.type === 'error') {
    if (msg.id !== undefined) {
      pending.get(msg.id)?.reject(new Error(msg.message));
      pending.delete(msg.id);
      return;
    }
    llmStatus.error = msg.message;
    waiters.forEach((w) => w.reject(new Error(msg.message)));
    waiters = [];
    stop();
    llmStatus.state = 'error';
  }
}

/** Starts the worker and the model's download if needed; resolves when it can write. */
export function ensureLlm(): Promise<void> {
  if (llmStatus.state === 'ready' && worker) return Promise.resolve();
  if (!worker) {
    worker = new Worker(new URL('./llm.worker.ts', import.meta.url), { type: 'module' });
    worker.onmessage = (e: MessageEvent<FromLlmWorker>) => onMessage(e.data);
    worker.onerror = (e) => onMessage({ type: 'error', message: e.message || 'language model worker failed' });
  }
  if (llmStatus.state !== 'loading') {
    llmStatus.progress = 0;
    llmStatus.error = '';
    llmStatus.state = 'loading';
    post({ type: 'load', base: import.meta.env.BASE_URL, model: MODEL });
  }
  return new Promise((resolve, reject) => waiters.push({ resolve, reject }));
}

/** Whether a reply can be started now. */
export function llmReady(): boolean {
  return llmStatus.state === 'ready' && !!worker;
}

/**
 * A reply to `messages` (the model's defaults for any parameter not given), or null if it was
 * skipped: still waiting when `deadline` (a `Date.now()` time) passed, or `signal` aborted it (one
 * the deadline cut short keeps what it wrote: `stop: 'deadline'`).
 * `reply`: its first words, written for the model (not part of the text returned); `lead`: the
 * start of the last message, whose state is kept too (`chat.ts`). Waits for the model to load.
 */
export async function generateText(
  messages: ChatMessage[],
  params: Partial<GenParams> = {},
  options: { reply?: string; lead?: string; deadline?: number; signal?: AbortSignal } = {},
): Promise<GenResult | null> {
  await ensureLlm();
  const id = nextId++;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    if (options.signal) {
      if (options.signal.aborted) post({ type: 'abort', id });
      else options.signal.addEventListener('abort', () => post({ type: 'abort', id }), { once: true });
    }
    post({ type: 'generate', id, messages, reply: options.reply ?? '', lead: options.lead ?? '', params: { ...MODEL.defaults, ...params }, deadline: options.deadline ?? Number.MAX_SAFE_INTEGER });
  });
}

export function llmStats(): LlmStats {
  return {
    state: llmStatus.state,
    model: MODEL.label,
    loadMs: counters.loadMs,
    replies: counters.replies,
    skipped: counters.skipped,
    queued: pending.size,
    lastPrefillMs: counters.lastPrefillMs,
    lastCachedShare: counters.lastCachedShare,
    lastTokPerS: counters.lastTokPerS,
    avgTokPerS: counters.decodeMs > 0 ? counters.tokens / (counters.decodeMs / 1000) : 0,
    busy: Math.min(1, counters.recent.filter((r) => r.at >= performance.now() - BUSY_WINDOW_MS).reduce((n, r) => n + r.ms, 0) / BUSY_WINDOW_MS),
  };
}

/** Stops the worker and frees the model's memory (files stay in the browser cache). */
export function unloadLlm(): void {
  stop();
  llmStatus.state = 'off';
  llmStatus.progress = 0;
}

function stop(): void {
  worker?.terminate();
  worker = null;
  Object.assign(counters, { loadMs: 0, replies: 0, skipped: 0, lastPrefillMs: 0, lastCachedShare: 0, lastTokPerS: 0, tokens: 0, decodeMs: 0, recent: [] });
  for (const p of pending.values()) p.reject(new Error('language model unloaded'));
  pending.clear();
  waiters.forEach((w) => w.reject(new Error('language model unloaded')));
  waiters = [];
}
