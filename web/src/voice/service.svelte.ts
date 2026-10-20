/**
 * The voice engine for the whole app (settings preview, benchmark, creator, the game): one
 * worker with Supertone's Supertonic 3 on the GPU (WebGPU) or, in Auto's fallback, on the
 * processor (WASM), loaded when the game starts (while voices are on), and a status the settings
 * screen shows.
 */

import { untrack } from 'svelte';
import { hasWebGpu } from '../ml/ort';
import { settings } from '../settings/settings.svelte';
import { SAMPLE_RATE, type FromVoiceWorker, type ToVoiceWorker, type VoiceDevice } from './protocol';
import type { LineTiming } from './supertonic';
import { parseVisemes, type VisemeTrack } from './visemes';
import type { VoiceParams } from './voices';

/** One line of speech: 24 kHz mono and its mouth shapes. */
export interface Clip {
  samples: Float32Array;
  visemes: VisemeTrack;
}

export interface BenchResult {
  /** Median generation time ÷ audio length (lower is faster; 0.25 = four times real time). */
  rtf: number;
  /** Median time to make one typical line, ms. */
  lineMs: number;
  /** Worst frame while speech was being made, ms (the 3D scene keeps drawing during the test). */
  worstFrameMs: number;
  cores: number;
  model: string;
  /** Where the test ran: the graphics card or the processor. */
  device: VoiceDevice;
  date: string;
}

export type BenchVerdict = 'good' | 'ok' | 'slow';
export type ModelState = 'off' | 'loading' | 'ready' | 'error';

/** How the voice worker is doing (for the debug overlay). */
export interface VoiceThreadStats {
  state: ModelState;
  /** Where the engine runs, while loaded (or starting). */
  device: VoiceDevice | null;
  /** Time to load the engine, ms (0 until ready). */
  loadMs: number;
  /** Lines made since the engine started, and lines waiting to be made. */
  lines: number;
  queued: number;
  /** Time to make a line, last and average, ms. */
  lastMs: number;
  avgMs: number;
  /** The last line's parts, ms (duration, text, vector, vocoder, shaping). */
  lastTiming: LineTiming;
  /** Generation time ÷ audio length over all lines (lower is faster). */
  rtf: number;
  /** Share of the last few seconds the worker spent making speech, 0..1. */
  busy: number;
  /** Clips kept for reuse, their size in bytes, and lines served from them. */
  cachedClips: number;
  cacheBytes: number;
  cacheHits: number;
}

// Before `voiceStatus`, which reads stored results with them.
const BENCH_KEY = 'littlelives.voiceBench';
/** The model, its voices and the runtime: a new id discards old speed-test results and cached clips. */
export const MODEL_ID = 'supertonic-3@3cadd1e+fp16w+6steps/ort-web@1.30.0';

export const voiceStatus = $state({
  state: 'off' as ModelState,
  /** Where the engine runs: the graphics card or the processor; null while unloaded. */
  device: null as VoiceDevice | null,
  /** Download progress 0..1 while loading. */
  progress: 0,
  error: '',
  benchmarking: false,
  bench: readBench(),
});

/** The download, MB: the runtime's WebGPU build, our WASM and dictionary (about 28.3 MB), the model (185.2 MB: 16-bit weights, gzipped) and its voices and text files (3.2 MB). */
export const DOWNLOAD_MB = 216.7;
const BENCH_LINES = [
  'Hello there!',
  "I'm so hungry, I could eat a whole pizza by myself.",
  'What a lovely garden. I should water the roses before it gets dark.',
];
/** The voice the speed test uses: one of the bundled men's (Arlo), as he is. */
const BENCH_VOICE: VoiceParams = { style: 'M1', pitch: 1, depth: 1, speed: 1 };

let worker: Worker | null = null;
/** The device to use next; null: Auto picks one when voices start. */
let wantedDevice: VoiceDevice | null = null;
/** Whether Auto chose the current device (so switching back to Auto keeps it); a manual choice clears it. */
let autoChoice = false;
let nextId = 1;
const pending = new Map<number, { resolve: (c: Clip) => void; reject: (e: Error) => void }>();
let waiters: { resolve: () => void; reject: (e: Error) => void }[] = [];
/** Counters behind `voiceThreadStats` (not reactive: read by the overlay's timer). */
const BUSY_WINDOW_MS = 5000;
const counters = {
  loadMs: 0,
  lines: 0,
  totalMs: 0,
  lastMs: 0,
  lastTiming: { duration: 0, text: 0, vector: 0, vocoder: 0, finish: 0 },
  audioSeconds: 0,
  recent: [] as { at: number; ms: number }[],
};
/** Times the engine broke and was started again this session (see `onMessage`). */
const MAX_RESTARTS = 3;
let restarts = 0;
/** Times the engine was stopped on purpose (voices off, paused): an Auto start waiting on its graphics-card check is then void. */
let stops = 0;
/** The game paused the voices for its session: they don't start by themselves until it ends (a preview still starts them). */
let held = $state(false);

function post(msg: ToVoiceWorker): void {
  worker?.postMessage(msg);
}

function onMessage(msg: FromVoiceWorker): void {
  if (msg.type === 'progress') {
    voiceStatus.progress = msg.total ? msg.loaded / msg.total : 0;
  } else if (msg.type === 'ready') {
    counters.loadMs = msg.ms;
    voiceStatus.progress = 1;
    voiceStatus.state = 'ready';
    waiters.forEach((w) => w.resolve());
    waiters = [];
  } else if (msg.type === 'audio') {
    const now = performance.now();
    counters.lines++;
    counters.totalMs += msg.ms;
    counters.lastMs = msg.ms;
    counters.lastTiming = msg.timing;
    counters.audioSeconds += msg.samples.length / SAMPLE_RATE;
    counters.recent.push({ at: now, ms: msg.ms });
    while (counters.recent.length && counters.recent[0].at < now - BUSY_WINDOW_MS) counters.recent.shift();
    pending.get(msg.id)?.resolve({ samples: msg.samples, visemes: parseVisemes(msg.visemes) });
    pending.delete(msg.id);
  } else if (msg.type === 'error') {
    if (msg.id !== undefined) {
      pending.get(msg.id)?.reject(new Error(msg.message));
      pending.delete(msg.id);
      return;
    }
    const failedDevice = voiceStatus.device;
    voiceStatus.error = msg.message;
    waiters.forEach((w) => w.reject(new Error(msg.message)));
    waiters = [];
    unload();
    voiceStatus.state = 'error';
    // The engine broke making a line: a new worker takes over (the game only speaks while the
    // engine is ready; a preview starts one itself). A few times a session at most.
    if (msg.restart && settings.voices && !held && restarts < MAX_RESTARTS) {
      restarts++;
      console.warn('[voice] the voice engine failed; starting it again:', msg.message);
      voiceStatus.state = 'off';
      void ensureVoice().catch(() => {});
    } else if (failedDevice === 'gpu' && settings.voices && !held && settings.voiceDevice === 'auto') {
      // Auto: the graphics card could not start the voices, so use the processor instead.
      wantedDevice = 'cpu';
      autoChoice = true;
      voiceStatus.state = 'off';
      void ensureVoice('cpu').catch(() => {});
    }
  }
}

/**
 * Starts the worker and the model's download if needed; resolves when it can speak. `device`
 * overrides the chosen one (Auto's fallback, or a manual choice); otherwise Auto picks the
 * graphics card when WebGPU is there and the processor when it is not.
 */
export async function ensureVoice(device?: VoiceDevice): Promise<void> {
  let target = device ?? chosenDevice();
  if (!target) {
    // Auto asks whether there is a graphics card. Meanwhile the voices may be switched off or
    // paused (this start is then void), or another call may start the engine (join it).
    const stopped = stops;
    const gpu = await hasWebGpu();
    if (stops !== stopped) throw new Error('voice unloaded');
    target = chosenDevice() ?? (gpu ? 'gpu' : 'cpu');
  }
  if (worker && voiceStatus.device === target) {
    if (voiceStatus.state === 'ready') return;
    if (voiceStatus.state === 'loading') return new Promise((resolve, reject) => waiters.push({ resolve, reject }));
  }
  if (worker) unload();
  wantedDevice = target;
  worker = new Worker(new URL('./tts.worker.ts', import.meta.url), { type: 'module' });
  worker.onmessage = (e: MessageEvent<FromVoiceWorker>) => onMessage(e.data);
  worker.onerror = (e) => onMessage({ type: 'error', message: e.message || 'voice worker failed' });
  voiceStatus.progress = 0;
  voiceStatus.error = '';
  voiceStatus.state = 'loading';
  voiceStatus.device = target;
  post({ type: 'load', base: import.meta.env.BASE_URL, language: settings.voiceLanguage, device: target });
  return new Promise((resolve, reject) => waiters.push({ resolve, reject }));
}

/** The device already decided on (Auto's pick or fallback, or a manual choice); null: Auto has yet to pick. */
function chosenDevice(): VoiceDevice | null {
  return wantedDevice ?? (settings.voiceDevice === 'auto' ? null : settings.voiceDevice);
}

/** Whether the engine can speak now. */
export function voiceReady(): boolean {
  return voiceStatus.state === 'ready' && !!worker;
}

/**
 * Clips already made, by language, model, voice and text. The engine is deterministic (seeded
 * noise), so a resident saying the same line again sounds exactly the same: reuse the clip
 * instead of making it again. Least recently used first out; survives the engine unloading.
 */
const CACHE_BYTES = 24 * 2 ** 20;
const clips = new Map<string, Clip>();
const making = new Map<string, Promise<Clip>>();
const cache = { bytes: 0, hits: 0 };

function clipKey(text: string, voice: VoiceParams): string {
  const v = [voice.pitch, voice.depth, voice.speed].map((x) => x.toFixed(3)).join('|');
  return `${settings.voiceLanguage}|${MODEL_ID}|${voice.style}|${v}|${text}`;
}

/** The clip for this line and voice if it was made before (and keeps it fresh), else null. */
function cachedClip(text: string, voice: VoiceParams): Clip | null {
  const key = clipKey(text, voice);
  const clip = clips.get(key);
  if (!clip) return null;
  clips.delete(key);
  clips.set(key, clip);
  cache.hits++;
  return clip;
}

/** Whether the clip for this line and voice was made before (without counting a hit). */
export function hasClip(text: string, voice: VoiceParams): boolean {
  return clips.has(clipKey(text, voice));
}

/** Audio for one line, from the cache or made (once, however often it's asked for meanwhile). */
export function speak(text: string, voice: VoiceParams): Promise<Clip> {
  const cached = cachedClip(text, voice);
  if (cached) return Promise.resolve(cached);
  const key = clipKey(text, voice);
  const inFlight = making.get(key);
  if (inFlight) return inFlight;
  const clip = synthesize(text, voice)
    .then((c) => {
      keep(key, c);
      return c;
    })
    .finally(() => making.delete(key));
  making.set(key, clip);
  return clip;
}

function keep(key: string, clip: Clip): void {
  const bytes = clipBytes(clip);
  if (clips.has(key) || bytes > CACHE_BYTES / 8) return;
  clips.set(key, clip);
  cache.bytes += bytes;
  for (const [k, old] of clips) {
    if (cache.bytes <= CACHE_BYTES) break;
    clips.delete(k);
    cache.bytes -= clipBytes(old);
  }
}

/** The samples and the whole flat track the parsed one views. */
function clipBytes(c: Clip): number {
  return c.samples.byteLength + c.visemes.times.buffer.byteLength + c.visemes.visemes.byteLength;
}

/** Audio for one line (24 kHz mono) and its mouth shapes, always made fresh (the speed test). Lines are made one after another in the worker. */
export async function synthesize(text: string, voice: VoiceParams): Promise<Clip> {
  await ensureVoice();
  const id = nextId++;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    post({ type: 'speak', id, text, voice: { ...voice } });
  });
}

export function voiceThreadStats(): VoiceThreadStats {
  const now = performance.now();
  const recentMs = counters.recent.filter((r) => r.at >= now - BUSY_WINDOW_MS).reduce((n, r) => n + r.ms, 0);
  return {
    state: voiceStatus.state,
    device: voiceStatus.device,
    loadMs: counters.loadMs,
    lines: counters.lines,
    queued: pending.size,
    lastMs: counters.lastMs,
    avgMs: counters.lines ? counters.totalMs / counters.lines : 0,
    lastTiming: counters.lastTiming,
    rtf: counters.audioSeconds ? counters.totalMs / 1000 / counters.audioSeconds : 0,
    busy: Math.min(1, recentMs / BUSY_WINDOW_MS),
    cachedClips: clips.size,
    cacheBytes: cache.bytes,
    cacheHits: cache.hits,
  };
}

/** Stops the worker and frees the model's memory (files stay in the browser cache). */
export function unloadVoice(): void {
  unload();
  wantedDevice = null;
  stops++;
}

/** The game paused the voices for its session: stops the engine and keeps it from starting by itself. */
export function pauseVoice(): void {
  held = true;
  unloadVoice();
}

/** That session ended: the engine starts again if voices are on. */
export function resumeVoice(): void {
  held = false;
}

/** Stops the worker; the state is 'off' until the caller says otherwise. */
function unload(): void {
  worker?.terminate();
  worker = null;
  voiceStatus.state = 'off';
  voiceStatus.progress = 0;
  voiceStatus.device = null;
  Object.assign(counters, {
    loadMs: 0,
    lines: 0,
    totalMs: 0,
    lastMs: 0,
    lastTiming: { duration: 0, text: 0, vector: 0, vocoder: 0, finish: 0 },
    audioSeconds: 0,
    recent: [],
  });
  for (const p of pending.values()) p.reject(new Error('voice unloaded'));
  pending.clear();
  waiters.forEach((w) => w.reject(new Error('voice unloaded')));
  waiters = [];
}

/** Whether the engine keeps up: lines in time (a 2-second line within about 1.5 s) and smooth frames. */
export function benchVerdict(b: BenchResult): BenchVerdict {
  if (b.lineMs <= 1500 && b.worstFrameMs < 50) return 'good';
  if (b.lineMs <= 3000) return 'ok';
  return 'slow';
}

/**
 * "Test this computer": makes three typical lines while the scene keeps drawing, and measures
 * how fast speech is made and whether frames stay smooth.
 */
export async function runBenchmark(): Promise<BenchResult> {
  voiceStatus.benchmarking = true;
  try {
    const result = await bench();
    voiceStatus.bench = result;
    try {
      localStorage.setItem(BENCH_KEY, JSON.stringify(result));
    } catch {
      // Storage unavailable; the result still shows for this session.
    }
    return result;
  } finally {
    voiceStatus.benchmarking = false;
  }
}

async function bench(): Promise<BenchResult> {
  await ensureVoice();
  await synthesize('Ready.', BENCH_VOICE); // warm-up (the GPU sets up its kernels)
  let worstFrameMs = 0;
  let last = performance.now();
  let watching = true;
  const watch = (t: number) => {
    worstFrameMs = Math.max(worstFrameMs, t - last);
    last = t;
    if (watching) requestAnimationFrame(watch);
  };
  requestAnimationFrame((t) => {
    last = t;
    requestAnimationFrame(watch);
  });
  const runs: { ms: number; rtf: number }[] = [];
  try {
    for (const line of BENCH_LINES) {
      const start = performance.now();
      const { samples } = await synthesize(line, BENCH_VOICE);
      const ms = performance.now() - start;
      runs.push({ ms, rtf: ms / 1000 / (samples.length / SAMPLE_RATE) });
    }
  } finally {
    watching = false;
  }
  const median = <T>(xs: T[], key: (x: T) => number) => xs.map(key).sort((a, b) => a - b)[Math.floor(xs.length / 2)];
  return {
    rtf: median(runs, (r) => r.rtf),
    lineMs: median(runs, (r) => r.ms),
    worstFrameMs,
    cores: navigator.hardwareConcurrency ?? 0,
    model: MODEL_ID,
    device: voiceStatus.device ?? 'gpu',
    date: new Date().toISOString(),
  };
}

/** The stored result if still valid for this model and this computer (older ones, per model, are dropped). */
function readBench(): BenchResult | null {
  try {
    const b = JSON.parse(localStorage.getItem(BENCH_KEY) ?? 'null') as BenchResult | null;
    if (!b || b.model !== MODEL_ID || b.cores !== (navigator.hardwareConcurrency ?? 0)) return null;
    // Results from before 0.3.0 ran on the graphics card (the only device then).
    return { ...b, device: b.device === 'cpu' ? 'cpu' : 'gpu' };
  } catch {
    return null;
  }
}

/**
 * Loaded from the start while voices are on (the game launching, or switching them on); unloaded
 * when they're switched off. A manual device choice restarts the engine on that device; Auto
 * picks one when it starts and otherwise keeps a working engine (it steps down by itself if the
 * card gets too slow, `stepDownVoice`). While the game has paused the voices for its session, nothing starts the engine here.
 */
$effect.root(() => {
  $effect(() => {
    const on = settings.voices;
    const mode = settings.voiceDevice;
    const paused = held;
    untrack(() => {
      if (!on) {
        // (Also voids an Auto start still asking about the graphics card.)
        unloadVoice();
        return;
      }
      if (paused) return;
      if (mode !== 'auto') {
        autoChoice = false;
        if (voiceStatus.device !== mode || voiceStatus.state === 'off' || voiceStatus.state === 'error') {
          void ensureVoice(mode).catch(() => {});
        }
        return;
      }
      if (voiceStatus.state === 'off' || voiceStatus.state === 'error') {
        wantedDevice = null;
        autoChoice = true;
        void ensureVoice().catch(() => {});
      } else if (!autoChoice) {
        // Coming back to Auto from a manual device: let Auto choose again (and keep a card it
        // already stepped down from, if Auto chose it).
        wantedDevice = null;
        autoChoice = true;
        unload();
        void ensureVoice().catch(() => {});
      }
    });
  });
});

/**
 * Auto's slow-machine step: stop the graphics-card engine and start the processor one. False if
 * this is not an Auto graphics-card session (a manual choice is never overridden), or if the
 * processor cannot start either (the caller then pauses the voices as before).
 */
export async function stepDownVoice(): Promise<boolean> {
  if (settings.voiceDevice !== 'auto' || voiceStatus.device !== 'gpu') return false;
  wantedDevice = 'cpu';
  autoChoice = true;
  try {
    // (It stops the graphics card's engine first.)
    await ensureVoice('cpu');
    return true;
  } catch {
    // The processor's engine failed to start: stop it, unless something took over since (a
    // restart, or the voices switched off).
    if (voiceStatus.state === 'error') unload();
    return false;
  }
}
