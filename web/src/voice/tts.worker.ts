/**
 * Voice worker: turns lines of text into 24 kHz audio, one at a time, off the main thread.
 * Supertone's Supertonic 3 runs on ONNX Runtime Web, on the GPU (WebGPU: a whole line in a few
 * large passes, about 30 times faster than it plays) or, in Auto's fallback, on the processor
 * (WASM: about 2.5 times faster than it plays). Our Rust/WASM
 * (`crates/voice-wasm`) shapes the voices, finds the mouth shapes and reads the text plain.
 * `supertonic.ts` makes each line.
 *
 * Everything is self-hosted, fetched once and kept in the Cache API, so the next session starts
 * without downloading.
 */

import { fetchOnce, forget, gunzipText, openCache, openRuntimeCache, prune } from '../ml/assets';
import { gpuBuffers, trackGpuBuffers } from '../ml/gpuMemory';
import { configureOrt, hasWebGpu, importOrt, ORT_WASM, sessionOptions } from '../ml/ort';
import init, { Voice } from './wasm-pkg/voice_wasm.js';
import wasmUrl from './wasm-pkg/voice_wasm_bg.wasm?url';
import { parseStyle, STYLE_IDS, synthesizeLine, type Config, type Engine, type Session, type StyleFile } from './supertonic';
import type { FromVoiceWorker, ToVoiceWorker, VoiceDevice } from './protocol';
import type { VoiceParams } from './voices';

interface WorkerScope {
  onmessage: ((e: MessageEvent<ToVoiceWorker>) => void) | null;
  postMessage(message: FromVoiceWorker, transfer?: Transferable[]): void;
  /** Called by the wasm runtime just before a Rust panic traps; set below, where the runtime looks for it. */
  __wasmPanic?(message: string): void;
}
const scope = self as WorkerScope;

/** Bumped when the files change; older caches are deleted. (New files under new names don't need it.) */
const CACHE_PREFIX = 'littlelives-voice-';
// v7: the matrix weights read in 8 bits on the GPU (`MatMulNBits`, 0.8.3), the same names.
const CACHE = `${CACHE_PREFIX}v7`;
/** Pocket TTS's encoded voices, no longer used: deleted. */
const OLD_VOICES_PREFIX = 'littlelives-voices-';

/**
 * Files under `voice/` (made by `cargo fetch-voice`), with sizes in bytes for the progress
 * bar and the SHA-256 `crates/tools/src/voicefetch.rs` pins (a graph's of its unpacked bytes).
 * The graphs come with their weights in 8-bit integers (the duration predictor's in 16-bit
 * floats), gzipped (`.onnxz`: unpacked by `fetchOnce`). The matrix weights, nearly all of them,
 * stay 8-bit on the GPU (`crates/tools/src/nbits.rs`): about 130 MB of GPU memory, not 400.
 * The dictionary has none: gzipped as it is made, its bytes may differ with another version of the gzip.
 */
type File = [name: string, bytes: number, sha256?: string];
const MODEL_DIR = 'supertonic-3/';
const MODELS = {
  duration: ['duration_predictor.onnxz', 1_633_681, '622de4d364dcb42259986c32d5ca4a0f521fddfaed09ffb155aa8cf3d7e4d864'],
  text: ['text_encoder.onnxz', 9_142_040, 'cd962e8655ce06da6551eb82dbaeb53d390a124ff552aba32b98bd135ae21892'],
  vector: ['vector_estimator.onnxz', 63_712_058, 'cdde82f0571c9bb31e0d2ef8f4ed7f0688710b28658d8d9626ad97e78e4b3c8e'],
  vocoder: ['vocoder.onnxz', 24_824_344, 'd230d59f057405246e25e2a6c011316e439bda6c3832627f7f05d46cdd3c63cf'],
} satisfies Record<keyof Engine['sessions'], File>;
/** A voice style file is about 291 KB. */
const STYLE_BYTES = 291_500;
const STYLE_SHA256: Record<(typeof STYLE_IDS)[number], string> = {
  F1: 'bbdec6ee00231c2c742ad05483df5334cab3b52fda3ba38e6a07059c4563dbc2',
  F2: '7c722c6a72707b1a77f035d67f0d1351ba187738e06f7683e8c72b1df3477fc6',
  F3: '12f6ef2573baa2defa1128069cb59f203e3ab67c92af77b42df8a0e3a2f7c6ab',
  F4: 'c2fa764c1225a76dfc3e2c73e8aa4f70d9ee48793860eb34c295fff01c2e032b',
  F5: '45966e73316415626cf41a7d1c6f3b4c70dbc1ba2bee5c1978ef0ce33244fc8d',
  M1: 'e35604687f5d23694b8e91593a93eec0e4eca6c0b02bb8ed69139ab2ea6b0a5b',
  M2: 'b76cbf62bac707c710cf0ae5aba5e31eea1a6339a9734bfae33ab98499534a50',
  M3: 'ea1ac35ccb91b0d7ecad533a2fbd0eec10c91513d8951e3b25fbba99954e159b',
  M4: 'ca8eefad4fcd989c9379032ff3e50738adc547eeb5e221b82593a6d7b3bac303',
  M5: 'dd22b92740314321f8ae11c5e87f8dd60d060f15dd3a632b5adf77f471f77af2',
};
const SMALL: File[] = [
  [`${MODEL_DIR}tts.json`, 8_253, '42078d3aef1cd43ab43021f3c54f47d2d75ceb4e75f627f118890128b06a0d09'],
  [`${MODEL_DIR}unicode_indexer.json`, 277_676, '9bf7346e43883a81f8645c81224f786d43c5b57f3641f6e7671a7d6c493cb24f'],
  ['en-us.lexz', 1_310_821],
];
const VOICE_WASM_BYTES = 162_000;

/** `GPUBufferUsage` STORAGE | COPY_SRC | COPY_DST: an input ONNX Runtime can read (and download). */
const INPUT_USAGE = 0x80 | 0x04 | 0x08;

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
    if (msg.measure) trackGpuBuffers();
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
  } else if (msg.type === 'memory') {
    const { live, peak } = gpuBuffers();
    scope.postMessage({ type: 'memory', gpuLive: live, gpuPeak: peak });
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
      const said = panicked;
      panicked = null;
      scope.postMessage({ type: 'error', message: said ?? String(err), restart: true });
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
  const files: { url: string; size: number; sha256?: string; built?: boolean; runtime?: boolean }[] = [
    { url: ORT_WASM.url, size: ORT_WASM.bytes, built: true, runtime: true },
    { url: wasmUrl, size: VOICE_WASM_BYTES, built: true },
    ...names.map((k) => ({ url: `${base}voice/${MODEL_DIR}${MODELS[k][0]}`, size: MODELS[k][1], sha256: MODELS[k][2] })),
    ...SMALL.map(([name, size, sha256]) => ({ url: `${base}voice/${name}`, size, sha256 })),
    ...STYLE_IDS.map((id) => ({ url: `${base}voice/${MODEL_DIR}${id}.json`, size: STYLE_BYTES, sha256: STYLE_SHA256[id] })),
  ];
  const total = files.reduce((n, f) => n + f.size, 0);
  const done = files.map(() => 0);
  const report = () => scope.postMessage({ type: 'progress', loaded: done.reduce((a, b) => a + b, 0), total });
  // The runtime in the cache both workers share (`openRuntimeCache`), the rest in this one's.
  const [cache, runtime] = await Promise.all([openCache(CACHE_PREFIX, CACHE), openRuntimeCache()]);
  const bytes = await Promise.all(
    files.map((f, i) =>
      fetchOnce(
        f.built && import.meta.env.DEV ? null : f.runtime ? runtime : cache,
        f.url,
        (n) => {
          done[i] = Math.min(n, f.size);
          report();
        },
        f.sha256,
      ),
    ),
  );
  void prune(cache, files.filter((f) => !f.runtime).map((f) => f.url));
  void prune(runtime, [ORT_WASM.url]);
  const [ortWasm, wasm, ...rest] = bytes;
  const models = rest.slice(0, names.length);
  const [config, indexer, lexicon, ...styles] = rest.slice(names.length);
  await init({ module_or_path: wasm });
  const json = <T>(b: ArrayBuffer) => JSON.parse(new TextDecoder().decode(b)) as T;
  // No proxy worker (this is the worker); the runtime comes from our own download.
  const ort = await importOrt();
  configureOrt(ort, ortWasm, device);
  const sessions = {} as Record<keyof typeof MODELS, Session>;
  // The runtime creates one session at a time. On the GPU the text encoder's output and the vector
  // estimator's latent stay there: the text into every pass, the latent from pass to pass and
  // straight into the vocoder, with no download and upload between the graphs. The duration
  // predictor and the vocoder hand back what we need on the CPU; on the processor everything
  // comes back as CPU tensors.
  for (const [i, k] of names.entries()) {
    sessions[k] = await ort.InferenceSession.create(new Uint8Array(models[i]), sessionOptions(device, k === 'vector' || k === 'text' ? 'gpu-buffer' : undefined));
  }
  // What every pass reads goes up to the GPU once a line (`Engine.upload`).
  const gpu = device === 'gpu' ? await ort.env.webgpu.device : null;
  engine = {
    ort,
    sessions,
    config: json<Config>(config),
    indexer: json<number[]>(indexer),
    styles: new Map(STYLE_IDS.map((id, i) => [id, parseStyle(json<StyleFile>(styles[i]))])),
    phonemizer: new Voice(await gunzipText(lexicon)),
    upload: gpu
      ? (data, dims) => {
          // Sized up to 16 bytes: the runtime's kernels read four floats at a time.
          const buffer = gpu.createBuffer({ size: Math.ceil(data.byteLength / 16) * 16, usage: INPUT_USAGE });
          gpu.queue.writeBuffer(buffer, 0, data);
          return ort.Tensor.fromGpuBuffer(buffer, { dataType: 'float32', dims, dispose: () => buffer.destroy() });
        }
      : undefined,
  };
  // The sessions and the engine have their own copies.
  for (const f of files) forget(f.url);
  // A lost graphics device (a driver reset, the GPU's memory gone) never comes back here: a new worker takes over.
  void gpu?.lost.then((info) => scope.postMessage({ type: 'error', message: `graphics device lost: ${info.message}`, restart: true }));
  scope.postMessage({ type: 'ready', ms: performance.now() - start });
}
