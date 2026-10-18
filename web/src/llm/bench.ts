/**
 * The language model benchmark: the game's own worker (`llm.worker.ts`), model and prompts
 * (`talk/prompt.ts`, `content/talk/en.json`), driven from a page with no game around it, so what
 * is measured is the model alone. `tools/llm/bench.mjs` starts Vite, opens this page in Chromium
 * with WebGPU and prints the report.
 *
 * Everything is timed by the worker's own clock (`GenResult`), so the numbers are the same the
 * game's F3 overlay shows:
 *
 * - loading: from starting the worker to the session being ready (download from the dev server,
 *   gunzip, WebGPU upload), with the download's progress;
 * - warm-up: the first generate, which compiles the kernels;
 * - reading a prompt: the game's first line (nothing cached) and the next one (the shared and
 *   persona parts cached), in ms and cached share;
 * - writing: tokens a second over `decodeTokens`, `decodeRuns` times (median, min, max);
 * - the game's five test lines end to end, from asking to finished text.
 *
 * Loaded only by `llm-bench.html`; the game never imports this.
 */

import { buildMessages, type LineEvent, type Persona, type Situation, type TalkText } from '../talk/prompt';
import { loadTalk } from '../talk/text';
import type { ChatMessage } from './chat';
import type { GenParams, GenResult } from './generate';
import { MODEL } from './models';
import type { FromLlmWorker, ToLlmWorker } from './protocol';

/** The game's longest line (talk/writer.ts `LINE_TOKENS`). */
const LINE_TOKENS = 160;
/** The game's line endings: the closing quote is not said. */
const LINE_STOPS = ['"', '“', '”'];

export interface PerfSample {
  name: string;
  promptTokens: number;
  cachedTokens: number;
  prefillMs: number;
  decodeMs: number;
  tokens: number;
  /** Written tokens over decode time. */
  tokPerS: number;
  /** Asking to finished text (worker round trip included). */
  totalMs: number;
  stop: GenResult['stop'];
  text: string;
}

export interface PerfReport {
  adapter: Record<string, string>;
  isolated: boolean;
  cores: number;
  userAgent: string;
  /** Worker start to ready: fetch, gunzip, session and WebGPU upload. */
  loadMs: number;
  loadProgress: { atMs: number; loaded: number; total: number }[];
  /** The first generate (kernel compilation). */
  warmupMs: number;
  /** Game line 1 (nothing cached) and line 2 (shared and persona cached). */
  cold: PerfSample;
  cached: PerfSample;
  /** Repeated long writes: every run, the median, the fastest and the slowest. */
  decode: { runs: PerfSample[]; tokPerS: number; minTokPerS: number; maxTokPerS: number; prefillMs: number };
  /** The game's five test lines. */
  lines: PerfSample[];
}

export interface PerfOptions {
  /** Tokens a decode run may write (default 96). */
  decodeTokens?: number;
  /** Decode runs (default 3). */
  decodeRuns?: number;
  /** Also run the game's five test lines (default true). */
  lines?: boolean;
}

declare global {
  interface Window {
    __llmPerf?: (options?: PerfOptions) => Promise<PerfReport>;
  }
}

const median = (xs: number[]) => (xs.length ? [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)] : 0);

/** The GPU's name and description, as the device test keeps it (`talk/devices.svelte.ts`). */
async function adapterInfo(): Promise<Record<string, string>> {
  const gpu = (navigator as Navigator & { gpu?: { requestAdapter(): Promise<unknown> } }).gpu;
  try {
    const adapter = (await gpu?.requestAdapter()) as { info?: Record<string, unknown>; requestAdapterInfo?: () => Promise<Record<string, unknown>> } | null;
    if (!adapter) return { adapter: 'none' };
    const info = adapter.info ?? (adapter.requestAdapterInfo ? await adapter.requestAdapterInfo() : undefined);
    if (!info) return { adapter: 'unknown' };
    const out: Record<string, string> = {};
    for (const key of ['vendor', 'architecture', 'device', 'description']) if (info[key]) out[key] = String(info[key]);
    return Object.keys(out).length ? out : { adapter: 'unknown' };
  } catch (err) {
    return { error: String(err) };
  }
}

/** The game's test moments (`talk/devices.svelte.ts` `testPrompts`): one persona, five moments, 1–5 sentences. */
function gameAsks(text: TalkText): { built: { messages: ChatMessage[]; lead: string; reply: string }; sentences: number }[] {
  const persona: Persona = {
    first: 'Mia',
    person: text.people.youngAdult?.female ?? 'a young woman',
    job: 'Line cook (Culinary)',
    personality: [text.traits.cheerful, text.traits.foodie].filter(Boolean),
  };
  const base: Situation = { when: 'Tuesday evening', place: 'at home, in the kitchen', feeling: ['happy'], mind: [], nearby: ['fridge', 'dining table'] };
  const moments: [Partial<Situation>, LineEvent][] = [
    [{}, { kind: 'use', action: 'cook a meal (stove)' }],
    [{ place: 'in the garden', partner: { name: 'Tom', relation: 'good friend' } }, { kind: 'social', social: 'chat', phase: 'start', other: 'Tom' }],
    [{ partner: { name: 'Tom', relation: 'friend' } }, { kind: 'social', social: 'joke', phase: 'good', other: 'Tom' }],
    [{ feeling: ['tired'] }, { kind: 'thought', thought: 'skipped', subject: 'work out' }],
    [{ place: 'at home, in the living room' }, { kind: 'emotion', emotion: 'inspired' }],
  ];
  return moments.map(([situation, event], i) => ({ built: buildMessages(text, persona, { ...base, ...situation }, event, [], i + 1)!, sentences: i + 1 }));
}

/** A prompt that keeps writing for a while: the decode benchmark (the game's own lines stop after 1–5). */
const LONG: ChatMessage[] = [
  { role: 'system', content: 'You write detailed, vivid diary entries in plain English, without headings or lists.' },
  { role: 'user', content: 'Write a long diary entry about a busy day working as a line cook. Write at least ten sentences, one after another, and do not stop early.' },
];

function sample(name: string, g: GenResult, totalMs: number): PerfSample {
  return {
    name,
    promptTokens: g.promptTokens,
    cachedTokens: g.cachedTokens,
    prefillMs: g.prefillMs,
    decodeMs: g.decodeMs,
    tokens: g.ids.length,
    tokPerS: g.decodeMs > 0 ? g.ids.length / (g.decodeMs / 1000) : 0,
    totalMs,
    stop: g.stop,
    text: g.text,
  };
}

/** Runs the benchmark once; rejects if the model can't load or run here. */
export async function perf(options: PerfOptions = {}): Promise<PerfReport> {
  const decodeTokens = options.decodeTokens ?? 96;
  const decodeRuns = Math.max(1, options.decodeRuns ?? 3);
  const text = await loadTalk('en');
  if (!text) throw new Error('no talk text: is web/public/content/talk/en.json served?');
  const asks = gameAsks(text);
  const adapter = await adapterInfo();

  const started = performance.now();
  const progress: { atMs: number; loaded: number; total: number }[] = [];
  const pending = new Map<number, { resolve: (r: GenResult | null) => void; reject: (e: Error) => void }>();
  let nextId = 1;
  let ready: { resolve: (ms: number) => void; reject: (e: Error) => void } | null = null;

  const worker = new Worker(new URL('./llm.worker.ts', import.meta.url), { type: 'module' });
  worker.onmessage = (e: MessageEvent<FromLlmWorker>) => {
    const msg = e.data;
    if (msg.type === 'progress') progress.push({ atMs: performance.now() - started, loaded: msg.loaded, total: msg.total });
    else if (msg.type === 'ready') ready?.resolve(performance.now() - started);
    else if (msg.type === 'done') {
      pending.get(msg.id)?.resolve(msg.result);
      pending.delete(msg.id);
    } else if (msg.type === 'skipped') {
      pending.get(msg.id)?.resolve(null);
      pending.delete(msg.id);
    } else if (msg.type === 'error') {
      if (msg.id !== undefined) {
        pending.get(msg.id)?.reject(new Error(msg.message));
        pending.delete(msg.id);
      } else {
        ready?.reject(new Error(msg.message));
      }
    }
  };
  const load = new Promise<number>((resolve, reject) => (ready = { resolve, reject }));
  worker.postMessage({ type: 'load', base: import.meta.env.BASE_URL, model: MODEL } satisfies ToLlmWorker);
  const loadMs = await load;

  /** One reply through the worker, timed end to end. */
  async function ask(messages: ChatMessage[], params: Partial<GenParams>, reply = '', lead = ''): Promise<PerfSample> {
    const id = nextId++;
    const at = performance.now();
    const result = await new Promise<GenResult | null>((resolve, reject) => {
      pending.set(id, { resolve, reject });
      worker.postMessage({ type: 'generate', id, messages, reply, lead, params: { ...MODEL.defaults, ...params }, deadline: Number.MAX_SAFE_INTEGER } satisfies ToLlmWorker);
    });
    const totalMs = performance.now() - at;
    if (!result) throw new Error('the worker skipped the reply');
    return sample(`#${id}`, result, totalMs);
  }

  try {
    // Kernels compile on the first run; a tiny one keeps it off the real measurements.
    const warmAt = performance.now();
    await ask([{ role: 'user', content: 'Hi' }], { maxTokens: 2, temperature: 0 });
    const warmupMs = performance.now() - warmAt;

    const lines: PerfSample[] = [];
    if (options.lines !== false) {
      for (const [i, { built, sentences }] of asks.entries()) {
        const run = await ask(built.messages, { temperature: 0, maxTokens: LINE_TOKENS, stopAt: LINE_STOPS, sentences }, built.reply, built.lead);
        run.name = `line ${i + 1} (${sentences} sentence${sentences > 1 ? 's' : ''})`;
        lines.push(run);
      }
    }

    const decode: PerfSample[] = [];
    for (let i = 0; i < decodeRuns; i++) {
      const run = await ask(LONG, { temperature: 0, maxTokens: decodeTokens, stopAtNewline: false, stopAt: [] });
      run.name = `decode ${i + 1}`;
      decode.push(run);
    }
    const speeds = decode.map((r) => r.tokPerS);

    const fallback = (name: string): PerfSample => sample(name, { text: '', ids: [], promptTokens: 0, cachedTokens: 0, prefillMs: 0, decodeMs: 0, stop: 'length', nonFinite: false }, 0);
    return {
      adapter,
      isolated: crossOriginIsolated,
      cores: navigator.hardwareConcurrency,
      userAgent: navigator.userAgent,
      loadMs,
      loadProgress: progress,
      warmupMs,
      cold: lines[0] ?? fallback('line 1'),
      cached: lines[1] ?? fallback('line 2'),
      decode: { runs: decode, tokPerS: median(speeds), minTokPerS: Math.min(...speeds), maxTokPerS: Math.max(...speeds), prefillMs: median(decode.map((r) => r.prefillMs)) },
      lines,
    };
  } finally {
    worker.terminate();
  }
}

if (typeof window !== 'undefined') {
  window.__llmPerf = perf;
  // Opened by hand (`llm-bench.html?auto`): run with the defaults and show the report.
  if (new URLSearchParams(location.search).has('auto')) {
    void perf()
      .then((report) => {
        const out = document.getElementById('out');
        if (out) out.textContent = JSON.stringify(report, null, 2);
      })
      .catch((err) => {
        const out = document.getElementById('out');
        if (out) out.textContent = `benchmark failed: ${String(err)}`;
      });
  }
}
