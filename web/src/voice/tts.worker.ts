/**
 * Voice worker: turns lines of text into 24 kHz audio, one at a time, off the main thread.
 * Our Rust/WASM phonemizer (`crates/voice-wasm`) turns text into KittenTTS nano's input ids, and
 * ONNX Runtime Web runs the model (its WASM build with SIMD, on this one thread; WebGPU only
 * when the device test tries it). `synth.ts` makes each line.
 *
 * Everything is self-hosted, fetched once and kept in the Cache API, so the next session starts
 * without downloading.
 */

import { fetchOnce, gunzipText, openCache, prune } from '../ml/assets';
import { configureOrt, importOrt, ORT_WASM, sessionOptions, type Device } from '../ml/ort';
import init, { Voice } from './wasm-pkg/voice_wasm.js';
import wasmUrl from './wasm-pkg/voice_wasm_bg.wasm?url';
import type { FromVoiceWorker, ToVoiceWorker } from './protocol';
import { synthesizeLine, type Engine, type Session } from './synth';
import type { VoiceParams } from './voices';

interface WorkerScope {
  onmessage: ((e: MessageEvent<ToVoiceWorker>) => void) | null;
  postMessage(message: FromVoiceWorker, transfer?: Transferable[]): void;
}
const scope = self as unknown as WorkerScope;

/** Bumped when the files change; older caches are deleted. (New files under new names don't need it.) */
const CACHE_PREFIX = 'littlelives-voice-';
const CACHE = `${CACHE_PREFIX}v2`;

/** Files under `voice/` (made by `tools/voice/fetch.mjs`), with sizes in bytes for the progress bar. */
type File = [name: string, bytes: number];
const LEXICON: File = ['en-us.lexz', 1_320_518];
const MODEL: File = ['kitten-nano-0.8-edit2.onnx', 15_450_474];
const VOICES: File = ['kitten-nano-0.8-voices.f32', 1_048_576];
const VOICE_WASM_BYTES = 152_378;

const engine: Engine = { ort: null as unknown as Engine['ort'], phonemizer: null as unknown as Engine['phonemizer'], session: null, kittenVoices: null };
let loading: Promise<void> | null = null;
/** Lines are made one after another: a run must finish before the next starts. */
let queue: Promise<void> = Promise.resolve();

scope.onmessage = (e) => {
  const msg = e.data;
  if (msg.type === 'load') {
    if (!loading) {
      loading = load(msg.base, msg.device ?? 'cpu').catch((err) => {
        loading = null;
        scope.postMessage({ type: 'error', message: String(err) });
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
    const start = performance.now();
    const { samples, visemes, resampleMs } = await synthesizeLine(engine, text, voice);
    scope.postMessage({ type: 'audio', id, samples, visemes, ms: performance.now() - start, resampleMs }, [samples.buffer, visemes.buffer]);
  } catch (err) {
    scope.postMessage({ type: 'error', id, message: String(err) });
  }
}

/** Downloads (or takes from the cache) the runtime, phonemizer, dictionary and model, and starts its session. */
async function load(base: string, device: Device): Promise<void> {
  const start = performance.now();
  // `built`: imported through Vite. In dev their URLs aren't hashed, so a rebuilt wasm would
  // otherwise come back stale from the cache; those are fetched fresh there.
  const files: { url: string; size: number; built?: boolean }[] = [
    { url: ORT_WASM[device].url, size: ORT_WASM[device].bytes, built: true },
    { url: wasmUrl, size: VOICE_WASM_BYTES, built: true },
    ...[LEXICON, MODEL, VOICES].map(([name, size]) => ({ url: `${base}voice/${name}`, size })),
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
  // Keeps the other device's runtime: the device test switches back and forth.
  void prune(cache, [...files.map((f) => f.url), ORT_WASM[device === 'gpu' ? 'cpu' : 'gpu'].url]);
  const [ortWasm, wasm, lexicon, model, voices] = bytes;
  await init({ module_or_path: wasm });
  engine.phonemizer = new Voice(await gunzipText(lexicon));
  // One thread, no proxy worker (this is the worker); the runtime comes from our own download.
  const ort = await importOrt(device);
  configureOrt(ort, device, 1, ortWasm);
  engine.ort = ort as unknown as Engine['ort'];
  engine.session = (await ort.InferenceSession.create(new Uint8Array(model), {
    ...sessionOptions(device),
  })) as unknown as Session;
  engine.kittenVoices = new Float32Array(voices);
  scope.postMessage({ type: 'ready', ms: performance.now() - start });
}
