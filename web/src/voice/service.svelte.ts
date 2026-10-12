/**
 * The voice engine for the whole app (settings preview, benchmark, creator, the game): one
 * worker with Kyutai's Pocket TTS, started and loaded the first time a voice is needed, and a
 * status the settings screen shows. Players' recordings are decoded here and handed to the
 * worker as clips (`useRecording`).
 */

import type { Device } from '../ml/ort';
import { settings } from '../settings/settings.svelte';
import { SAMPLE_RATE, type FromVoiceWorker, type ToVoiceWorker } from './protocol';
import { parseVisemes, type VisemeTrack } from './visemes';
import type { Recording, VoiceParams } from './voices';

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
  date: string;
}

export type BenchVerdict = 'good' | 'ok' | 'slow';
export type ModelState = 'off' | 'loading' | 'ready' | 'error';

/** How the voice worker is doing (for the debug overlay). */
export interface VoiceThreadStats {
  state: ModelState;
  /** Where the model runs and on how many threads. */
  device: Device;
  threads: number;
  /** Time to load the engine, ms (0 until ready). */
  loadMs: number;
  /** Lines made since the engine started, and lines waiting to be made. */
  lines: number;
  queued: number;
  /** Time to make a line, last and average, ms. */
  lastMs: number;
  avgMs: number;
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
export const MODEL_ID = 'pocket-tts-en-2026-04@58a6d00+int8+presets2/ort-web@1.30.0';

export const voiceStatus = $state({
  state: 'off' as ModelState,
  /** Download progress 0..1 while loading. */
  progress: 0,
  error: '',
  benchmarking: false,
  bench: readBench(),
});

/** The download, MB: runtime, our WASM and dictionary (about 15.7 MB), the model (146.5 MB with its tokenizer) and its voices (11.3 MB). */
export const DOWNLOAD_MB = 173.5;
const BENCH_LINES = [
  'Hello there!',
  "I'm so hungry, I could eat a whole pizza by myself.",
  'What a lovely garden. I should water the roses before it gets dark.',
];
/** The voice the speed test uses: one of the bundled men's (Charles), as he is. */
const BENCH_VOICE: VoiceParams = { clip: 'preset:Charles', clipPitch: 1, clipDepth: 1, pitch: 1, depth: 1, speed: 1, temperature: 0.7 };

let worker: Worker | null = null;
/** Counts workers started: what one knew (prepared voices), the next doesn't. */
let generation = 0;
export const voiceGeneration = () => generation;
/** Where the model runs: the CPU, unless the device test found the GPU better (`useVoiceDevice`). */
let device: Device = 'cpu';
/**
 * The runtime's threads: two where the page is cross-origin isolated and the computer has six
 * cores or more (about 1.6 times as fast as one; the game, the simulation and a language model
 * keep the rest), else one.
 */
export function voiceThreads(): number {
  return self.crossOriginIsolated && (navigator.hardwareConcurrency ?? 0) >= 6 ? 2 : 1;
}
let nextId = 1;
const pending = new Map<number, { resolve: (c: Clip) => void; reject: (e: Error) => void }>();
let waiters: { resolve: () => void; reject: (e: Error) => void }[] = [];
/** Counters behind `voiceThreadStats` (not reactive: read by the overlay's timer). */
const BUSY_WINDOW_MS = 5000;
const counters = { loadMs: 0, lines: 0, totalMs: 0, lastMs: 0, audioSeconds: 0, recent: [] as { at: number; ms: number }[] };

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
    voiceStatus.error = msg.message;
    waiters.forEach((w) => w.reject(new Error(msg.message)));
    waiters = [];
    unload();
    voiceStatus.state = 'error';
  }
}

/** Starts the worker and the model's download if needed; resolves when it can speak. */
export function ensureVoice(): Promise<void> {
  if (voiceStatus.state === 'ready' && worker) return Promise.resolve();
  if (!worker) {
    generation++;
    worker = new Worker(new URL('./tts.worker.ts', import.meta.url), { type: 'module' });
    worker.onmessage = (e: MessageEvent<FromVoiceWorker>) => onMessage(e.data);
    worker.onerror = (e) => onMessage({ type: 'error', message: e.message || 'voice worker failed' });
  }
  if (voiceStatus.state !== 'loading') {
    voiceStatus.progress = 0;
    voiceStatus.error = '';
    voiceStatus.state = 'loading';
    post({ type: 'load', base: import.meta.env.BASE_URL, language: settings.voiceLanguage, device, threads: voiceThreads() });
    // A new worker knows no recordings yet.
    for (const [sha, r] of recordings) void r.then((pcm) => post({ type: 'clip', sha, samples: pcm.samples.slice(), rate: pcm.rate })).catch(() => {});
  }
  return new Promise((resolve, reject) => waiters.push({ resolve, reject }));
}

/** Runs the model on another device from the next load on (unloading it if loaded on another). */
export function useVoiceDevice(next: Device): void {
  if (next === device) return;
  device = next;
  if (worker) unloadVoice();
}

/** The device the model runs on (or will, when loaded). */
export function voiceDevice(): Device {
  return device;
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
  const v = [voice.clipPitch, voice.clipDepth, voice.pitch, voice.depth, voice.speed, voice.temperature].map((x) => x.toFixed(3)).join('|');
  // The GPU's arithmetic differs a little from the CPU's: clips are kept per device.
  return `${settings.voiceLanguage}|${MODEL_ID}|${device}|${voice.clip}|${v}|${text}`;
}

/** The clip for this line and voice if it was made before (and keeps it fresh), else null. */
export function cachedClip(text: string, voice: VoiceParams): Clip | null {
  const key = clipKey(text, voice);
  const clip = clips.get(key);
  if (!clip) return null;
  clips.delete(key);
  clips.set(key, clip);
  cache.hits++;
  return clip;
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
  await recordingFor(voice);
  const id = nextId++;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    post({ type: 'speak', id, text, voice: { ...voice } });
  });
}

/**
 * Gets a voice ready ahead of its first line (when a resident is selected, or a voice is chosen
 * in the creator): about two seconds the first time a voice is heard in this browser, a fraction
 * of that after.
 */
export function prepareVoice(voice: VoiceParams): void {
  if (!worker || voiceStatus.state === 'off' || voiceStatus.state === 'error') return;
  void recordingFor(voice)
    .then(() => post({ type: 'prepare', voice: { ...voice } }))
    .catch(() => {});
}

/** Players' recordings by SHA: decoded to 24 kHz once, given to every worker started. */
const recordings = new Map<string, Promise<{ samples: Float32Array; rate: number }>>();
/** Recordings this browser couldn't decode (made in another browser, say): those residents use their generated voice. */
export const brokenRecordings = new Set<string>();

/** Decodes a recording (once) and hands it to the worker as clip `rec:<sha>`; rejects if this browser can't read it. */
export function useRecording(rec: Recording): Promise<void> {
  let r = recordings.get(rec.sha);
  if (!r) {
    r = decodeRecording(rec);
    recordings.set(rec.sha, r);
    r.catch(() => {
      recordings.delete(rec.sha);
      brokenRecordings.add(rec.sha);
    });
    void r.then((pcm) => post({ type: 'clip', sha: rec.sha, samples: pcm.samples.slice(), rate: pcm.rate })).catch(() => {});
  }
  return r.then(() => {});
}

/** Waits for a recording a voice uses (it must have been handed over with `useRecording`). */
function recordingFor(voice: VoiceParams): Promise<void> {
  if (!voice.clip.startsWith('rec:')) return Promise.resolve();
  const r = recordings.get(voice.clip.slice(4));
  return r ? r.then(() => {}) : Promise.reject(new Error('recording not loaded'));
}

/** A recording's sound at 24 kHz (the browser decodes and resamples it). */
async function decodeRecording(rec: Recording): Promise<{ samples: Float32Array; rate: number }> {
  const bin = atob(rec.audio);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  const ctx = new OfflineAudioContext(1, SAMPLE_RATE, SAMPLE_RATE);
  const audio = await ctx.decodeAudioData(bytes.buffer);
  return { samples: audio.getChannelData(0).slice(), rate: audio.sampleRate };
}

export function voiceThreadStats(): VoiceThreadStats {
  const now = performance.now();
  const recentMs = counters.recent.filter((r) => r.at >= now - BUSY_WINDOW_MS).reduce((n, r) => n + r.ms, 0);
  return {
    state: voiceStatus.state,
    device,
    threads: voiceThreads(),
    loadMs: counters.loadMs,
    lines: counters.lines,
    queued: pending.size,
    lastMs: counters.lastMs,
    avgMs: counters.lines ? counters.totalMs / counters.lines : 0,
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
  voiceStatus.state = 'off';
  voiceStatus.progress = 0;
}

function unload(): void {
  worker?.terminate();
  worker = null;
  Object.assign(counters, { loadMs: 0, lines: 0, totalMs: 0, lastMs: 0, audioSeconds: 0, recent: [] });
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
 * how fast speech is made and whether frames stay smooth (on the CPU unless the device test
 * found the GPU better).
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
  await synthesize('Ready.', BENCH_VOICE); // warm-up (and the voice prepared)
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
    date: new Date().toISOString(),
  };
}

/** The stored result if still valid for this model and this computer (older ones, per model, are dropped). */
function readBench(): BenchResult | null {
  try {
    const b = JSON.parse(localStorage.getItem(BENCH_KEY) ?? 'null') as BenchResult | null;
    return b && b.model === MODEL_ID && b.cores === (navigator.hardwareConcurrency ?? 0) ? b : null;
  } catch {
    return null;
  }
}

/** Unload when voices are switched off. */
$effect.root(() => {
  $effect(() => {
    if (!settings.voices && worker) unloadVoice();
  });
});
