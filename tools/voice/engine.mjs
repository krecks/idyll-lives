/**
 * The voice engine in Node, as the browser's voice worker runs it: our voice WASM
 * (`web/src/voice/wasm-pkg`), ONNX Runtime Web (its WASM build: Node has no WebGPU, so here on
 * the CPU; the arithmetic differs from the GPU's only in rounding) and the files in
 * `web/public/voice/`, with every line made by `web/src/voice/supertonic.ts` (the worker's own
 * code). Used by `check.mjs` and `samples.mjs`. Needs `pnpm voice` first.
 */

import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { gunzipSync } from 'node:zlib';

const web = new URL('../../web/', import.meta.url);
const require = createRequire(new URL('package.json', web));
export const ort = require('onnxruntime-web');
const wasm = await import(new URL('src/voice/wasm-pkg/voice_wasm.js', web).href);
export const { synthesizeLine, parseStyle, STYLE_IDS } = await import(new URL('src/voice/supertonic.ts', web).href);
export const voices = await import(new URL('src/voice/voices.ts', web).href);
export const { PRESETS } = voices;

export const SAMPLE_RATE = 24000;
/** The model's folder and files, as the worker names them. */
export const MODEL_DIR = 'supertonic-3/';
/** The graphs as the site ships them: 16-bit weights, gzipped (`fetch.mjs`). */
export const MODELS = { duration: 'duration_predictor.onnxz', text: 'text_encoder.onnxz', vector: 'vector_estimator.onnxz', vocoder: 'vocoder.onnxz' };

const dir = new URL('public/voice/', web);
wasm.initSync({ module: readFileSync(new URL('src/voice/wasm-pkg/voice_wasm_bg.wasm', web)) });
export const { median_pitch: medianPitch } = wasm;
export const phonemizer = new wasm.Voice(gunzipSync(readFileSync(new URL('en-us.lexz', dir))).toString());

/** Threads for the runtime: `VOICE_THREADS` (default 4). Set before the first session. */
ort.env.wasm.numThreads = Number(process.env.VOICE_THREADS ?? 4);
ort.env.logLevel = 'error';

const file = (name) => readFileSync(new URL(MODEL_DIR + name, dir));
const json = (name) => JSON.parse(file(name).toString());

/** An engine with the model loaded (fresh sessions). */
export async function engine() {
  const options = { executionProviders: ['wasm'], graphOptimizationLevel: 'all' };
  const sessions = {};
  for (const [k, name] of Object.entries(MODELS)) sessions[k] = await ort.InferenceSession.create(gunzipSync(file(name)), options);
  return {
    ort,
    sessions,
    config: json('tts.json'),
    indexer: json('unicode_indexer.json'),
    styles: new Map(STYLE_IDS.map((id) => [id, parseStyle(json(`${id}.json`))])),
    phonemizer,
  };
}

/** 24 kHz samples of one line in a voice (`VoiceParams`). */
export async function say(e, text, voice) {
  return (await synthesizeLine(e, text, voice)).samples;
}
