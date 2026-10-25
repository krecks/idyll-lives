/**
 * Voice worker: turns lines of text into 24 kHz audio, one at a time, off the main thread.
 * Supertone's Supertonic 3 runs on ONNX Runtime Web, on the GPU (WebGPU: a whole line in a few
 * large passes, about 30 times faster than it plays) or, in Auto's fallback, on the processor
 * (WASM: about 2.5 times faster than it plays; docs/design/voices.md). Our Rust/WASM
 * (`crates/voice-wasm`) shapes the voices, finds the mouth shapes and reads the text plain.
 * `supertonic.ts` makes each line.
 *
 * Everything is self-hosted, fetched once and kept in the Cache API, so the next session starts
 * without downloading.
 */

import { fetchOnce, forget, gunzipText, openCache, prune } from '../ml/assets';
import { configureOrt, hasWebGpu, importOrt, ORT_WASM, sessionOptions } from '../ml/ort';
import init, { Voice } from './wasm-pkg/voice_wasm.js';
import wasmUrl from './wasm-pkg/voice_wasm_bg.wasm?url';
import { parseStyle, STYLE_IDS, synthesizeLine, type Config, type Engine, type Phonemizer, type Session, type StyleFile } from './supertonic';
import type { FromVoiceWorker, ToVoiceWorker, VoiceDevice } from './protocol';
import type { VoiceParams } from './voices';

interface WorkerScope {
  onmessage: ((e: MessageEvent<ToVoiceWorker>) => void) | null;
  postMessage(message: FromVoiceWorker, transfer?: Transferable[]): void;
  /** Set by the wasm runtime: it calls this just before a Rust panic traps. */
  __wasmPanic?(message: string): void;
}
const scope = self as WorkerScope;

/** Bumped when the files change; older caches are deleted. (New files under new names don't need it.) */
const CACHE_PREFIX = 'littlelives-voice-';
const CACHE = `${CACHE_PREFIX}v5`;
/** Pocket TTS's encoded voices, no longer used: deleted. */
const OLD_VOICES_PREFIX = 'littlelives-voices-';

/**
 * Files under `voice/` (made by `cargo fetch-voice`), with sizes in bytes for the progress
 * bar. The graphs come with their weights in 16-bit floats, gzipped (`.onnxz`: unpacked by
 * `fetchOnce`).
 */
type File = [name: string, bytes: number];
const MODEL_DIR = 'supertonic-3/';
const MODELS = {
  duration: ['duration_predictor.onnxz', 1_633_681],
  text: ['text_encoder.onnxz', 16_783_293],
  vector: ['vector_estimator.onnxz', 119_556_677],
  vocoder: ['vocoder.onnxz', 47_191_753],
} satisfies Record<keyof Engine['sessions'], File>;
/** A voice style file is about 291 KB. */
const STYLE_BYTES = 291_500;
const SMALL: File[] = [
  [`${MODEL_DIR}tts.json`, 8_253],
  [`${MODEL_DIR}unicode_indexer.json`, 277_676],
  ['en-us.lexz', 1_310_821],
];
const VOICE_WASM_BYTES = 162_000;

let engine: Engine | null = null;
let loading: Promise<void> | null = null;
/** Lines are made one after another: a run must finish before the next starts. */
let queue: Promise<void> = Promise.resolve();
/** What our WebAssembly said as it panicked (its panic hook calls `__wasmPanic` just before the trap). */
let panicked: string | null = null;

scope.__wasmPanic = (message) => {
  panicked = message;
  console.error(`[voice] ${message}`);
};

scope.onmessage = (e) => {
  const msg = e.data;
  if (msg.type === 'load') {
    if (!loading) {
      loading = load(msg.base, msg.device).catch((err) => {
        loading = null;
        scope.postMessage({ type: 'error', message: String(err instanceof Error ? err.message : err) });
        throw err;
      });
      loading.catch(() => {});
    }
  } else if (msg.type === 'speak') {
    queue = queue.then(() => speak(msg.id, msg.text, msg.voice));
  }
};

async function speak(id: number, text: string, voice: VoiceParams): Promise<void> {
  try {
    if (!loading) throw new Error('voice model not loaded');
    await loading;
    if (!engine) throw new Error('voice model not loaded');
    const start = performance.now();
    const { samples, visemes, timing } = await synthesizeLine(engine, text, voice);
    scope.postMessage({ type: 'audio', id, samples, visemes, ms: performance.now() - start, timing }, [samples.buffer, visemes.buffer]);
  } catch (err) {
    scope.postMessage({ type: 'error', id, message: String(err) });
    // A trap (ours, or ONNX Runtime's) leaves that WebAssembly broken: its stack and memory are
    // wherever the call stopped. Nothing here uses it again; the main thread starts a new worker.
    if (err instanceof WebAssembly.RuntimeError) {
      engine = null;
      loading = null;
      scope.postMessage({ type: 'error', message: panicked ?? String(err), restart: true });
    }
  }
}

/** Deletes caches an older voice model left (Pocket TTS's encoded voices). */
async function dropOldCaches(): Promise<void> {
  try {
    for (const name of await caches.keys()) if (name.startsWith(OLD_VOICES_PREFIX)) await caches.delete(name);
  } catch {
    // Only space.
  }
}

/** Downloads (or takes from the cache) the runtime, our WASM, the model and its voices, and starts its sessions on `device`. */
async function load(base: string, device: VoiceDevice): Promise<void> {
  const start = performance.now();
  if (device === 'gpu' && !(await hasWebGpu())) throw new Error('voices need a graphics card that supports WebGPU');
  void dropOldCaches();
  const names = Object.keys(MODELS) as (keyof typeof MODELS)[];
  // `built`: imported through Vite. In dev their URLs aren't hashed, so a rebuilt wasm would
  // otherwise come back stale from the cache; those are fetched fresh there.
  const files: { url: string; size: number; built?: boolean }[] = [
    { url: ORT_WASM.url, size: ORT_WASM.bytes, built: true },
    { url: wasmUrl, size: VOICE_WASM_BYTES, built: true },
    ...names.map((k) => ({ url: `${base}voice/${MODEL_DIR}${MODELS[k][0]}`, size: MODELS[k][1] })),
    ...SMALL.map(([name, size]) => ({ url: `${base}voice/${name}`, size })),
    ...STYLE_IDS.map((id) => ({ url: `${base}voice/${MODEL_DIR}${id}.json`, size: STYLE_BYTES })),
  ];
  const total = files.reduce((n, f) => n + f.size, 0);
  const done = files.map(() => 0);
  const report = () => scope.postMessage({ type: 'progress', loaded: done.reduce((a, b) => a + b, 0), total });
  const cache = await openCache(CACHE_PREFIX, CACHE);
  const bytes = await Promise.all(
    files.map((f, i) =>
      fetchOnce(f.built && import.meta.env.DEV ? null : cache, f.url, (n) => {
        done[i] = Math.min(n, f.size);
        report();
      }),
    ),
  );
  void prune(cache, files.map((f) => f.url));
  const [ortWasm, wasm, ...rest] = bytes;
  const models = rest.slice(0, names.length);
  const [config, indexer, lexicon, ...styles] = rest.slice(names.length);
  await init({ module_or_path: wasm });
  const json = <T>(b: ArrayBuffer) => JSON.parse(new TextDecoder().decode(b)) as T;
  // No proxy worker (this is the worker); the runtime comes from our own download.
  const ort = await importOrt();
  configureOrt(ort, ortWasm, device);
  const sessions = {} as Record<keyof typeof MODELS, Session>;
  // The runtime creates one session at a time. On the GPU the vector estimator's latent stays there
  // between its passes and goes straight into the vocoder, and the other graphs hand back the few
  // numbers we need on the CPU; on the processor everything comes back as CPU tensors.
  for (const [i, k] of names.entries()) {
    sessions[k] = (await ort.InferenceSession.create(
      new Uint8Array(models[i]),
      sessionOptions(device, k === 'vector' ? 'gpu-buffer' : undefined),
    )) as unknown as Session;
  }
  engine = {
    ort: ort as unknown as Engine['ort'],
    sessions,
    config: json<Config>(config),
    indexer: json<number[]>(indexer),
    styles: new Map(STYLE_IDS.map((id, i) => [id, parseStyle(json<StyleFile>(styles[i]))])),
    phonemizer: new Voice(await gunzipText(lexicon)) as unknown as Phonemizer,
  };
  // The sessions and the engine have their own copies.
  for (const f of files) forget(f.url);
  scope.postMessage({ type: 'ready', ms: performance.now() - start });
}
