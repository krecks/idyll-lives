/**
 * The language model for the whole app: one worker, started and loaded the first time a reply
 * is wanted, on the device asked for (`ensureLlm`), and a status for settings screens. Replies
 * are written one after another; each has a deadline after which it's no longer wanted.
 *
 * Knows nothing about the game: callers pass chat messages and get text back (`talk/` writes
 * residents' lines with it).
 */

import type { Device } from '../ml/ort';
import type { ChatMessage } from './chat';
import type { GenParams, GenResult } from './generate';
import { allModels, modelById } from './library.svelte';
import { DEFAULT_MODEL, type ModelSpec } from './models';
import type { FromLlmWorker, ToLlmWorker } from './protocol';

export type LlmState = 'off' | 'loading' | 'ready' | 'error';

export const llmStatus = $state({
  state: 'off' as LlmState,
  /** Download progress 0..1 while loading. */
  progress: 0,
  error: '',
  /** What runs now (once ready). */
  model: DEFAULT_MODEL.id,
  device: 'cpu' as Device,
  threads: 0,
});

/** How the LLM worker is doing (for the debug overlay). */
export interface LlmStats {
  state: LlmState;
  model: string;
  device: Device;
  threads: number;
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
let loaded = { model: '', device: 'cpu' as Device };
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
    llmStatus.device = msg.device;
    llmStatus.threads = msg.threads;
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

/**
 * Starts the worker and the model's download if needed; resolves when it can write. Asking for
 * another device or model than the one loaded unloads it first.
 */
export function ensureLlm(options: { device?: Device; model?: string } = {}): Promise<void> {
  const device = options.device ?? loaded.device;
  const model = modelById(options.model ?? DEFAULT_MODEL.id).id;
  if (worker && (loaded.device !== device || loaded.model !== model)) unloadLlm();
  if (llmStatus.state === 'ready' && worker) return Promise.resolve();
  if (!worker) {
    worker = new Worker(new URL('./llm.worker.ts', import.meta.url), { type: 'module' });
    worker.onmessage = (e: MessageEvent<FromLlmWorker>) => onMessage(e.data);
    worker.onerror = (e) => onMessage({ type: 'error', message: e.message || 'language model worker failed' });
    loaded = { model, device };
  }
  if (llmStatus.state !== 'loading') {
    llmStatus.progress = 0;
    llmStatus.error = '';
    llmStatus.state = 'loading';
    llmStatus.model = model;
    llmStatus.device = device;
    post({ type: 'load', base: import.meta.env.BASE_URL, model: $state.snapshot(modelById(model)), device, keep: allModels().flatMap(fileUrls) });
  }
  return new Promise((resolve, reject) => waiters.push({ resolve, reject }));
}

/** Whether a reply can be started now. */
export function llmReady(): boolean {
  return llmStatus.state === 'ready' && !!worker;
}

/**
 * A reply to `messages` (the model's defaults for any parameter not given), or null if it was
 * skipped: still waiting when `deadline` (a `Date.now()` time) passed, or `signal` aborted it.
 * `reply`: its first words, written for the model (not part of the text returned); `lead`: the
 * start of the last message, whose state is kept too (`chat.ts`). Waits for the model to load.
 */
export async function generateText(
  messages: ChatMessage[],
  params: Partial<GenParams> = {},
  options: { reply?: string; lead?: string; deadline?: number; signal?: AbortSignal } = {},
): Promise<GenResult | null> {
  await ensureLlm({ device: loaded.device, model: loaded.model || undefined });
  const id = nextId++;
  const spec = modelById(loaded.model);
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    if (options.signal) {
      if (options.signal.aborted) post({ type: 'abort', id });
      else options.signal.addEventListener('abort', () => post({ type: 'abort', id }), { once: true });
    }
    post({ type: 'generate', id, messages, reply: options.reply ?? '', lead: options.lead ?? '', params: { ...spec.defaults, ...params }, deadline: options.deadline ?? Number.MAX_SAFE_INTEGER });
  });
}

export function llmStats(): LlmStats {
  return {
    state: llmStatus.state,
    model: llmStatus.model,
    device: llmStatus.device,
    threads: llmStatus.threads,
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

/** Where a model's files are (to keep them in the cache). */
function fileUrls(m: ModelSpec): string[] {
  const dir = new URL(m.url, new URL(import.meta.env.BASE_URL, location.href)).href;
  return [m.tokenizer, m.tokenizerConfig, m.graph, ...(m.data ? [m.data] : [])].map(([name]) => dir + name);
}
