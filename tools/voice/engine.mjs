/**
 * The voice engine in Node, as the browser's voice worker runs it: our voice WASM
 * (`web/src/voice/wasm-pkg`), ONNX Runtime Web (its WASM build) and the files in
 * `web/public/voice/`, with every line made by `web/src/voice/pocket.ts` (the worker's own code).
 * Used by `check.mjs` and `samples.mjs`. Needs `pnpm voice` first.
 */

import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { gunzipSync } from 'node:zlib';

const web = new URL('../../web/', import.meta.url);
const require = createRequire(new URL('package.json', web));
export const ort = require('onnxruntime-web');
const { Tokenizer } = await import(new URL('node_modules/@huggingface/tokenizers/dist/tokenizers.mjs', web).href);
const wasm = await import(new URL('src/voice/wasm-pkg/voice_wasm.js', web).href);
export const { synthesizeLine, prepareVoice, voiceKey } = await import(new URL('src/voice/pocket.ts', web).href);
export const voices = await import(new URL('src/voice/voices.ts', web).href);
export const { PRESETS } = voices;

export const SAMPLE_RATE = 24000;
/** The model's folder and files, as the worker names them. */
export const MODEL_DIR = 'pocket-en-2026-04/';
export const MODELS = { encoder: 'mimi_encoder_int8.onnx', text: 'text_conditioner_int8.onnx', main: 'flow_lm_main_int8.onnx', flow: 'flow_lm_flow_int8.onnx', decoder: 'mimi_decoder_int8.onnx' };

const dir = new URL('public/voice/', web);
wasm.initSync({ module: readFileSync(new URL('src/voice/wasm-pkg/voice_wasm_bg.wasm', web)) });
export const { clip, reference, median_pitch: medianPitch } = wasm;
export const phonemizer = new wasm.Voice(gunzipSync(readFileSync(new URL('en-us.lexz', dir))).toString());

/** Threads for the runtime: `VOICE_THREADS` (default 1, as the worker runs without cross-origin isolation). Set before the first session. */
ort.env.wasm.numThreads = Number(process.env.VOICE_THREADS ?? 1);
ort.env.logLevel = 'error';

const file = (name) => readFileSync(new URL(MODEL_DIR + name, dir));

/** `bos_before_voice.npy` (float32, little-endian) as a flat array. */
function npy(buf) {
  const header = buf.readUInt16LE(8);
  return new Float32Array(buf.buffer.slice(buf.byteOffset + 10 + header, buf.byteOffset + buf.length));
}

/** The bundled voices by clip name (`preset:<name>`), as the worker unpacks them. */
function presetClips() {
  const pcm = readFileSync(new URL('presets-2.pcm', dir));
  const index = JSON.parse(readFileSync(new URL('presets-2.json', dir), 'utf8'));
  const clips = new Map();
  for (const { name, offset, length } of index) {
    const c = new Float32Array(length);
    for (let i = 0; i < length; i++) c[i] = pcm.readInt16LE((offset + i) * 2) / 32768;
    clips.set(`preset:${name}`, c);
  }
  return clips;
}

/** An engine with the model loaded (fresh sessions, nothing kept). */
export async function engine() {
  const options = { executionProviders: ['wasm'], graphOptimizationLevel: 'all' };
  const sessions = {};
  for (const [k, name] of Object.entries(MODELS)) sessions[k] = await ort.InferenceSession.create(file(name), options);
  return {
    ort,
    sessions,
    tokenizer: new Tokenizer(JSON.parse(file('tokenizer.json').toString()), {}),
    bundle: JSON.parse(file('bundle.json').toString()),
    bos: npy(file('bos_before_voice.npy')),
    phonemizer,
    reference,
    clips: presetClips(),
    voices: new Map(),
  };
}

/** 24 kHz samples of one line in a voice (`VoiceParams`). */
export async function say(e, text, voice) {
  return (await synthesizeLine(e, text, voice)).samples;
}
