/**
 * Voice worker: turns lines of text into 24 kHz audio, one at a time, off the main thread.
 * Kyutai's Pocket TTS runs on ONNX Runtime Web (its WASM build with SIMD; WebGPU only when the
 * device test tries it), and our Rust/WASM (`crates/voice-wasm`) shapes the voices, finds the
 * mouth shapes and reads the text plain. `pocket.ts` makes each line.
 *
 * Everything is self-hosted, fetched once and kept in the Cache API, so the next session starts
 * without downloading; encoded voices are kept there too, so a voice is encoded once.
 */

import { Tokenizer } from '@huggingface/tokenizers';
import { fetchOnce, openCache, prune, gunzipText } from '../ml/assets';
import { configureOrt, importOrt, ORT_WASM, sessionOptions, type Device } from '../ml/ort';
import init, { clip, reference, Voice } from './wasm-pkg/voice_wasm.js';
import wasmUrl from './wasm-pkg/voice_wasm_bg.wasm?url';
import { prepareVoice, synthesizeLine, type Bundle, type EmbeddingStore, type Engine, type Phonemizer, type Session } from './pocket';
import type { FromVoiceWorker, ToVoiceWorker } from './protocol';
import type { VoiceParams } from './voices';

interface WorkerScope {
  onmessage: ((e: MessageEvent<ToVoiceWorker>) => void) | null;
  postMessage(message: FromVoiceWorker, transfer?: Transferable[]): void;
}
const scope = self as unknown as WorkerScope;

/** Bumped when the files change; older caches are deleted. (New files under new names don't need it.) */
const CACHE_PREFIX = 'littlelives-voice-';
const CACHE = `${CACHE_PREFIX}v3`;
/** Encoded voices, per model and clip shaping: bumped when either changes (older ones are deleted). */
const VOICES_PREFIX = 'littlelives-voices-';
const VOICES_CACHE = `${VOICES_PREFIX}pocket-en-2026-04-shape2`;

/** Files under `voice/` (made by `tools/voice/fetch.mjs`), with sizes in bytes for the progress bar. */
type File = [name: string, bytes: number];
const MODEL_DIR = 'pocket-en-2026-04/';
const MODELS = {
  encoder: ['mimi_encoder_int8.onnx', 20_779_616],
  text: ['text_conditioner_int8.onnx', 16_388_384],
  main: ['flow_lm_main_int8.onnx', 76_341_079],
  flow: ['flow_lm_flow_int8.onnx', 9_962_530],
  decoder: ['mimi_decoder_int8.onnx', 22_684_077],
} satisfies Record<keyof Engine['sessions'], File>;
const SMALL: File[] = [
  [`${MODEL_DIR}bundle.json`, 24_381],
  [`${MODEL_DIR}bos_before_voice.npy`, 4_224],
  [`${MODEL_DIR}tokenizer.json`, 245_020],
  ['presets-2.json', 1_188],
  ['presets-2.pcm', 11_329_820],
  ['en-us.lexz', 1_320_518],
];
const VOICE_WASM_BYTES = 158_000;

let engine: Engine | null = null;
let loading: Promise<void> | null = null;
/** Lines are made one after another: a run must finish before the next starts. */
let queue: Promise<void> = Promise.resolve();

scope.onmessage = (e) => {
  const msg = e.data;
  if (msg.type === 'load') {
    if (!loading) {
      loading = load(msg.base, msg.device ?? 'cpu', msg.threads ?? 1).catch((err) => {
        loading = null;
        scope.postMessage({ type: 'error', message: String(err) });
        throw err;
      });
      loading.catch(() => {});
    }
  } else if (msg.type === 'clip') {
    // After loading (the WASM shapes it), before any line asks for it.
    queue = queue.then(async () => {
      await loading?.catch(() => {});
      if (engine) engine.clips.set(`rec:${msg.sha}`, clip(msg.samples, msg.rate));
    });
  } else if (msg.type === 'prepare') {
    queue = queue.then(() => prepare(msg.voice));
  } else if (msg.type === 'speak') {
    queue = queue.then(() => speak(msg.id, msg.text, msg.voice));
  }
};

async function prepare(voice: VoiceParams): Promise<void> {
  try {
    await loading;
    if (engine) await prepareVoice(engine, voice);
  } catch {
    // Said again when the line comes; its error goes with it.
  }
}

async function speak(id: number, text: string, voice: VoiceParams): Promise<void> {
  try {
    if (!loading) throw new Error('voice model not loaded');
    await loading;
    if (!engine) throw new Error('voice model not loaded');
    const start = performance.now();
    const { samples, visemes, prepMs } = await synthesizeLine(engine, text, voice);
    scope.postMessage({ type: 'audio', id, samples, visemes, ms: performance.now() - start, prepMs }, [samples.buffer, visemes.buffer]);
  } catch (err) {
    scope.postMessage({ type: 'error', id, message: String(err) });
  }
}

/** Encoded voices in the Cache API (a few hundred KB each), so the next session doesn't encode them again. */
async function embeddingStore(): Promise<EmbeddingStore | null> {
  try {
    const cache = await openCache(VOICES_PREFIX, VOICES_CACHE);
    if (!cache) return null;
    const url = (key: string) => `https://voices.invalid/${encodeURIComponent(key)}`;
    return {
      async get(key) {
        const hit = await cache.match(url(key));
        return hit ? new Float32Array(await hit.arrayBuffer()) : null;
      },
      async put(key, embeddings) {
        await cache.put(url(key), new Response(embeddings.slice().buffer));
      },
    };
  } catch {
    return null;
  }
}

/** The bundled voices, as clips by name. */
function presetClips(pcm: ArrayBuffer, index: { name: string; offset: number; length: number }[]): Map<string, Float32Array> {
  const all = new Int16Array(pcm);
  const clips = new Map<string, Float32Array>();
  for (const { name, offset, length } of index) clips.set(`preset:${name}`, Float32Array.from(all.subarray(offset, offset + length), (v) => v / 32768));
  return clips;
}

/** `bos_before_voice.npy` (float32, little-endian) as a flat array. */
function npy(bytes: ArrayBuffer): Float32Array {
  const header = new DataView(bytes).getUint16(8, true);
  return new Float32Array(bytes.slice(10 + header));
}

/** Downloads (or takes from the cache) the runtime, our WASM, the model and the voices, and starts its sessions. */
async function load(base: string, device: Device, threads: number): Promise<void> {
  const start = performance.now();
  const names = Object.keys(MODELS) as (keyof typeof MODELS)[];
  // `built`: imported through Vite. In dev their URLs aren't hashed, so a rebuilt wasm would
  // otherwise come back stale from the cache; those are fetched fresh there.
  const files: { url: string; size: number; built?: boolean }[] = [
    { url: ORT_WASM[device].url, size: ORT_WASM[device].bytes, built: true },
    { url: wasmUrl, size: VOICE_WASM_BYTES, built: true },
    ...names.map((k) => ({ url: `${base}voice/${MODEL_DIR}${MODELS[k][0]}`, size: MODELS[k][1] })),
    ...SMALL.map(([name, size]) => ({ url: `${base}voice/${name}`, size })),
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
  const [ortWasm, wasm, ...rest] = bytes;
  const models = rest.slice(0, names.length);
  const [bundleJson, bos, tokenizerJson, presetIndex, presetPcm, lexicon] = rest.slice(names.length);
  await init({ module_or_path: wasm });
  const text = (b: ArrayBuffer) => new TextDecoder().decode(b);
  // No proxy worker (this is the worker); the runtime comes from our own download.
  const ort = await importOrt(device);
  configureOrt(ort, device, threads, ortWasm);
  const sessions = {} as Record<keyof typeof MODELS, Session>;
  for (const [i, k] of names.entries()) {
    sessions[k] = (await ort.InferenceSession.create(new Uint8Array(models[i]), sessionOptions(device))) as unknown as Session;
  }
  engine = {
    ort: ort as unknown as Engine['ort'],
    sessions,
    tokenizer: new Tokenizer(JSON.parse(text(tokenizerJson)), {}),
    bundle: JSON.parse(text(bundleJson)) as Bundle,
    bos: npy(bos),
    phonemizer: new Voice(await gunzipText(lexicon)) as unknown as Phonemizer,
    reference,
    clips: presetClips(presetPcm, JSON.parse(text(presetIndex))),
    voices: new Map(),
    store: await embeddingStore(),
  };
  scope.postMessage({ type: 'ready', ms: performance.now() - start });
}
