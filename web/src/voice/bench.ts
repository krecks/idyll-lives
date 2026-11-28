/**
 * The voice benchmark: the game's own voice worker (`tts.worker.ts`, Supertonic 3) on the GPU
 * (WebGPU) or the processor, driven from a page with no game around it, so what is measured is
 * the voices alone. `tools/voice/bench.mjs` starts Vite, opens `voice-bench.html` in Chromium and
 * prints the report:
 *
 * - loading: from starting the worker to its sessions being ready;
 * - memory: bytes in the worker's GPU buffers once loaded and at their peak (`ml/gpuMemory.ts`),
 *   and the worker's JavaScript and WebAssembly memory (`measureUserAgentSpecificMemory`, where
 *   the page is cross-origin isolated);
 * - each test line in its voice, `runs` times: the parts' times (`LineTiming`, the worker's own
 *   clock), the whole line, and how many times faster than it plays it was made;
 * - the samples of each line's first run (24 kHz mono), for listening and comparing.
 *
 * Loaded only by `voice-bench.html`; the game never imports this.
 */

import type { FromVoiceWorker, ToVoiceWorker, VoiceDevice } from './protocol';
import { SAMPLE_RATE } from './protocol';
import type { LineTiming } from './supertonic';
import type { VoiceParams } from './voices';

/** Lines like residents' (one to five sentences), each in a voice of its own. */
const BENCH_LINES: { name: string; text: string; voice: VoiceParams }[] = [
  { name: 'arlo-short', text: 'Hello there!', voice: { style: 'M1', pitch: 1, depth: 1, speed: 1 } },
  { name: 'ada-hungry', text: "I'm so hungry, I could eat a whole pizza by myself.", voice: { style: 'F1', pitch: 1, depth: 1, speed: 1 } },
  { name: 'child-garden', text: 'What a lovely garden. I should water the roses before it gets dark.', voice: { style: 'F1', pitch: 1.3, depth: 1.08, speed: 1.04 } },
  { name: 'ben-work', text: "Another long day at the office. The printer jammed twice, and my boss wanted the report by noon. I just want to sit down.", voice: { style: 'M2', pitch: 1, depth: 1, speed: 1 } },
  { name: 'cleo-joke', text: "Ha! That's the funniest thing I've heard all week, Tom.", voice: { style: 'F3', pitch: 1, depth: 1, speed: 1 } },
  { name: 'dex-cook', text: "The sauce needs more garlic. Maybe a little basil too. Let's see how it tastes now. Perfect, if I say so myself!", voice: { style: 'M4', pitch: 0.95, depth: 1, speed: 0.95 } },
  { name: 'elsie-long', text: "I met the new neighbours this morning. They have two dogs and a little boy who loves trains. We talked about the weather, the bakery on the corner and the school. I think we'll get along just fine. Maybe I'll bring them a cake on Sunday.", voice: { style: 'F5', pitch: 1, depth: 1, speed: 1.05 } },
  { name: 'ezra-tired', text: "I'm exhausted. Bed sounds wonderful right now.", voice: { style: 'M5', pitch: 1, depth: 1, speed: 0.9 } },
];

interface LineSample {
  name: string;
  /** Seconds of speech made. */
  seconds: number;
  /** Each run: the worker's time for the whole line, and its parts, ms. */
  runs: { ms: number; timing: LineTiming }[];
  /** Median of the runs' whole times, ms, and how many times faster than it plays. */
  ms: number;
  speed: number;
  /** Every run gave the same samples. */
  stable: boolean;
  /** The first run's samples, float32 little-endian in base64. */
  samples: string;
}

interface VoiceReport {
  device: VoiceDevice;
  adapter: Record<string, string>;
  isolated: boolean;
  userAgent: string;
  /** Worker start to ready (download from the dev server or the cache, sessions, WebGPU upload). */
  loadMs: number;
  /** The first line (shaders compiled), ms. */
  warmupMs: number;
  /** GPU buffer bytes once loaded, and at the peak over all the lines. */
  gpuLoaded: number;
  gpuPeak: number;
  gpuAfter: number;
  /** The worker's JS and WebAssembly memory once loaded and after the lines (0: not measurable here). */
  workerLoaded: number;
  workerAfter: number;
  lines: LineSample[];
  /** All lines' speech over all their median times: times faster than it plays. */
  speed: number;
  /** Medians over all lines and runs of each part's time per second of speech, ms. */
  perSecond: LineTiming;
}

interface VoiceBenchOptions {
  device?: VoiceDevice;
  /** Runs per line (default 3). */
  runs?: number;
}

declare global {
  interface Window {
    __voicePerf?: (options?: VoiceBenchOptions) => Promise<VoiceReport>;
  }
  interface Performance {
    measureUserAgentSpecificMemory?(): Promise<{ breakdown: { bytes: number; attribution: { scope?: string }[] }[] }>;
  }
}

const median = (xs: number[]) => (xs.length ? [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)] : 0);

async function adapterInfo(): Promise<Record<string, string>> {
  const gpu = (navigator as Navigator & { gpu?: { requestAdapter(): Promise<unknown> } }).gpu;
  try {
    const adapter = (await gpu?.requestAdapter()) as { info?: Record<string, unknown> } | null;
    const info = adapter?.info;
    if (!info) return { adapter: adapter ? 'unknown' : 'none' };
    const out: Record<string, string> = {};
    for (const key of ['vendor', 'architecture', 'device', 'description']) if (info[key]) out[key] = String(info[key]);
    return out;
  } catch (err) {
    return { error: String(err) };
  }
}

/** The dedicated workers' memory (JS heap and WebAssembly), bytes; 0 where it can't be measured. */
async function workerMemory(): Promise<number> {
  if (!crossOriginIsolated || !performance.measureUserAgentSpecificMemory) return 0;
  try {
    const m = await performance.measureUserAgentSpecificMemory();
    return m.breakdown.filter((b) => b.attribution.some((a) => a.scope === 'DedicatedWorkerGlobalScope')).reduce((n, b) => n + b.bytes, 0);
  } catch {
    return 0;
  }
}

function base64(samples: Float32Array): string {
  const bytes = new Uint8Array(samples.buffer, samples.byteOffset, samples.byteLength);
  let s = '';
  for (let i = 0; i < bytes.length; i += 0x8000) s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(s);
}

const same = (a: Float32Array, b: Float32Array) => a.length === b.length && a.every((x, i) => Object.is(x, b[i]));

/** Runs the benchmark once; rejects if the voices can't load or run here. */
export async function perf(options: VoiceBenchOptions = {}): Promise<VoiceReport> {
  const device = options.device ?? 'gpu';
  const runs = Math.max(1, options.runs ?? 3);
  const adapter = await adapterInfo();
  const worker = new Worker(new URL('./tts.worker.ts', import.meta.url), { type: 'module' });
  let ready: { resolve: () => void; reject: (e: Error) => void } | null = null;
  let memory: ((m: { gpuLive: number; gpuPeak: number }) => void) | null = null;
  const pending = new Map<number, { resolve: (m: { samples: Float32Array; ms: number; timing: LineTiming }) => void; reject: (e: Error) => void }>();
  worker.onmessage = (e: MessageEvent<FromVoiceWorker>) => {
    const msg = e.data;
    if (msg.type === 'ready') ready?.resolve();
    else if (msg.type === 'memory') memory?.(msg);
    else if (msg.type === 'audio') {
      pending.get(msg.id)?.resolve(msg);
      pending.delete(msg.id);
    } else if (msg.type === 'error') {
      const err = new Error(msg.message);
      if (msg.id !== undefined) {
        pending.get(msg.id)?.reject(err);
        pending.delete(msg.id);
      } else ready?.reject(err);
    }
  };
  const gpu = () =>
    new Promise<{ gpuLive: number; gpuPeak: number }>((resolve) => {
      memory = resolve;
      worker.postMessage({ type: 'memory' } satisfies ToVoiceWorker);
    });
  let nextId = 1;
  const speak = (text: string, voice: VoiceParams) =>
    new Promise<{ samples: Float32Array; ms: number; timing: LineTiming }>((resolve, reject) => {
      const id = nextId++;
      pending.set(id, { resolve, reject });
      worker.postMessage({ type: 'speak', id, text, voice } satisfies ToVoiceWorker);
    });

  try {
    const started = performance.now();
    const loaded = new Promise<void>((resolve, reject) => (ready = { resolve, reject }));
    worker.postMessage({ type: 'load', base: import.meta.env.BASE_URL, device, measure: true } satisfies ToVoiceWorker);
    await loaded;
    const loadMs = performance.now() - started;
    const gpuLoaded = (await gpu()).gpuLive;
    const workerLoaded = await workerMemory();

    const warmAt = performance.now();
    await speak('Warming up.', BENCH_LINES[0].voice);
    const warmupMs = performance.now() - warmAt;

    const lines: LineSample[] = [];
    const parts: Record<keyof LineTiming, number[]> = { duration: [], text: [], vector: [], vocoder: [], finish: [] };
    for (const line of BENCH_LINES) {
      const results: { samples: Float32Array; ms: number; timing: LineTiming }[] = [];
      for (let r = 0; r < runs; r++) results.push(await speak(line.text, line.voice));
      const seconds = results[0].samples.length / SAMPLE_RATE;
      const ms = median(results.map((x) => x.ms));
      for (const x of results) for (const k of Object.keys(parts) as (keyof LineTiming)[]) parts[k].push(x.timing[k] / Math.max(seconds, 1e-3));
      lines.push({
        name: line.name,
        seconds,
        runs: results.map((x) => ({ ms: x.ms, timing: x.timing })),
        ms,
        speed: ms > 0 ? (seconds * 1000) / ms : 0,
        stable: results.every((x) => same(x.samples, results[0].samples)),
        samples: base64(results[0].samples),
      });
    }
    const after = await gpu();
    const totalSeconds = lines.reduce((n, l) => n + l.seconds, 0);
    const totalMs = lines.reduce((n, l) => n + l.ms, 0);
    return {
      device,
      adapter,
      isolated: crossOriginIsolated,
      userAgent: navigator.userAgent,
      loadMs,
      warmupMs,
      gpuLoaded,
      gpuPeak: after.gpuPeak,
      gpuAfter: after.gpuLive,
      workerLoaded,
      workerAfter: await workerMemory(),
      lines,
      speed: totalMs > 0 ? (totalSeconds * 1000) / totalMs : 0,
      perSecond: { duration: median(parts.duration), text: median(parts.text), vector: median(parts.vector), vocoder: median(parts.vocoder), finish: median(parts.finish) },
    };
  } finally {
    worker.terminate();
  }
}

if (typeof window !== 'undefined') window.__voicePerf = perf;
