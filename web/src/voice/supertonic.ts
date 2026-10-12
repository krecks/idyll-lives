/**
 * One line of speech with Supertone's Supertonic 3, the same way everywhere: the voice worker
 * (`tts.worker.ts`) and the Node tools (`tools/voice/check.mjs`, `samples.mjs`) call this, so the
 * checks hear exactly what the game plays. Plain TypeScript with type-only imports (Node runs it
 * by stripping the types). Ported from Supertone's reference code (`web/helper.js`,
 * github.com/supertone-inc/supertonic).
 *
 * Supertonic speaks in one of ten voice styles (`F1`…`M5`) at a given speed, a whole line at a
 * time: the duration predictor tells how long the line lasts, the text encoder reads it, then
 * `STEPS` passes of the vector estimator turn noise into the line's latents (flow matching) and
 * the vocoder turns them into 44.1 kHz audio. A few large passes, so it suits the GPU. The noise
 * is ours, from a hash of the voice and the text (`noise`), so the same line in the same voice
 * gives the same samples (the clip cache relies on it). Our WASM then brings the line to 24 kHz,
 * trims it, gives it the resident's own pitch and size and finds its mouth shapes.
 */

import type { VoiceParams } from './voices';

export interface OrtTensor {
  readonly data: unknown;
  readonly dims: readonly number[];
}
/** The pieces of ONNX Runtime used here (the worker's `onnxruntime-web/webgpu`, or Node's `onnxruntime-web`). */
export interface Ort {
  Tensor: new (type: 'int64' | 'float32', data: BigInt64Array | Float32Array, dims: readonly number[]) => OrtTensor;
}
export interface Session {
  run(feeds: Record<string, OrtTensor>): Promise<Record<string, OrtTensor>>;
}
/** `Line` from the phonemizer WASM (`crates/voice-wasm`). */
interface Line {
  readonly text: string;
  finish(samples: Float32Array, rate: number): Float32Array;
  /** After `finish`: the clip's mouth shapes, flat (`parseVisemes`). */
  readonly visemes: Float32Array;
  free(): void;
}
export interface Phonemizer {
  line(text: string, speed: number, pitch: number, depth: number): Line;
}

/** `tts.json`, the parts used here. */
export interface Config {
  ae: { sample_rate: number; base_chunk_size: number };
  ttl: { chunk_compress_factor: number; latent_dim: number };
}
/** A voice style file (`F1.json`…): two tensors, as nested arrays. */
export interface StyleFile {
  style_ttl: { dims: number[]; data: unknown[] };
  style_dp: { dims: number[]; data: unknown[] };
}
/** A voice style, flat. */
export interface Style {
  ttl: { data: Float32Array; dims: readonly number[] };
  dp: { data: Float32Array; dims: readonly number[] };
}

export interface Engine {
  ort: Ort;
  sessions: { duration: Session; text: Session; vector: Session; vocoder: Session };
  config: Config;
  /** Unicode code point to the model's text id (`unicode_indexer.json`). */
  indexer: number[];
  /** Voice styles by id (`F1`…`M5`). */
  styles: Map<string, Style>;
  phonemizer: Phonemizer;
}

/** The voice styles Supertonic 3 comes with, as its files name them. */
export const STYLE_IDS = ['F1', 'F2', 'F3', 'F4', 'F5', 'M1', 'M2', 'M3', 'M4', 'M5'] as const;
/** Denoising passes per line (the reference's default: 4 is about twice as fast, a little rougher). */
export const STEPS = 8;
/** The reference's speaking rate; a voice's `speed` is a factor on it. */
const BASE_SPEED = 1.05;

/** A style file as flat tensors. */
export function parseStyle(file: StyleFile): Style {
  const flat = (t: { dims: number[]; data: unknown[] }) => ({ data: Float32Array.from(t.data.flat(Infinity) as number[]), dims: t.dims });
  return { ttl: flat(file.style_ttl), dp: flat(file.style_dp) };
}

/**
 * The reference's clean-up of English text (`preprocessText`): Unicode decomposed, dashes,
 * quotes and stray symbols made plain, spaces before punctuation dropped, a full stop at the end
 * if there's none, and the language tags around it.
 */
export function prepareText(text: string): string {
  let t = text.normalize('NFKD');
  t = t.replace(/[\u{1F300}-\u{1FAFF}\u{2600}-\u{27BF}\u{1F1E6}-\u{1F1FF}]+/gu, '');
  t = t.replace(/[–‑—]/g, '-').replace(/[“”]/g, '"').replace(/[‘’´`]/g, "'").replace(/[_[\]|/#→←]/g, ' ');
  t = t.replace(/[♥☆♡©\\]/g, '').replaceAll('@', ' at ');
  t = t.replace(/ ([,.!?;:'])/g, '$1');
  t = t.replace(/"{2,}/g, '"').replace(/'{2,}/g, "'");
  t = t.replace(/\s+/g, ' ').trim();
  if (!/[.!?;:,'")\]}…]$/.test(t)) t += '.';
  return `<en>${t}</en>`;
}

/** 32-bit FNV-1a of a string. */
function hashString(s: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 0x01000193);
  return h >>> 0;
}

/** A uniform (0, 1) value for element `i` of stream `seed`. */
function uniform(seed: number, i: number): number {
  let h = (Math.imul(seed ^ 0x9e3779b9, 0x85ebca6b) ^ Math.imul(i + 1, 0x27d4eb2f)) >>> 0;
  h = Math.imul(h ^ (h >>> 16), 0x7feb352d);
  h = Math.imul(h ^ (h >>> 15), 0x846ca68b);
  h ^= h >>> 16;
  return ((h >>> 0) + 0.5) / 4294967296;
}

/** Standard normal noise (Box–Muller on a hashed stream). */
function noise(seed: number, n: number): Float32Array {
  const out = new Float32Array(n);
  for (let i = 0; i < n; i++) out[i] = Math.sqrt(-2 * Math.log(uniform(seed, 2 * i))) * Math.cos(2 * Math.PI * uniform(seed, 2 * i + 1));
  return out;
}

/**
 * 24 kHz mono for one line and its mouth shapes (flat). `finishMs`: bringing it to 24 kHz,
 * trimming, pitch and size, and mouth shapes.
 */
export async function synthesizeLine(engine: Engine, text: string, voice: VoiceParams): Promise<{ samples: Float32Array; visemes: Float32Array; finishMs: number }> {
  // The model speaks at the voice's speed itself; our WASM gives the pitch and size.
  const line = engine.phonemizer.line(text, 1, voice.pitch, voice.depth);
  try {
    const plain = line.text;
    if (!plain) return { samples: new Float32Array(0), visemes: Float32Array.of(0, 0), finishMs: 0 };
    const style = engine.styles.get(voice.style);
    if (!style) throw new Error(`unknown voice style ${voice.style}`);
    const { ort, sessions, config } = engine;
    const speed = BASE_SPEED * voice.speed;
    const prepared = prepareText(plain);
    const ids: bigint[] = [];
    for (const ch of prepared) {
      const c = ch.codePointAt(0)!;
      ids.push(BigInt(c < engine.indexer.length ? engine.indexer[c] : -1));
    }
    const n = ids.length;
    const textIds = new ort.Tensor('int64', BigInt64Array.from(ids), [1, n]);
    const textMask = new ort.Tensor('float32', new Float32Array(n).fill(1), [1, 1, n]);
    const styleTtl = new ort.Tensor('float32', style.ttl.data, style.ttl.dims);
    const styleDp = new ort.Tensor('float32', style.dp.data, style.dp.dims);
    const seconds = ((await sessions.duration.run({ text_ids: textIds, style_dp: styleDp, text_mask: textMask })).duration.data as Float32Array)[0] / speed;
    const textEmb = (await sessions.text.run({ text_ids: textIds, style_ttl: styleTtl, text_mask: textMask })).text_emb;
    const rate = config.ae.sample_rate;
    const chunk = config.ae.base_chunk_size * config.ttl.chunk_compress_factor;
    const dim = config.ttl.latent_dim * config.ttl.chunk_compress_factor;
    const frames = Math.max(1, Math.ceil(Math.floor(seconds * rate) / chunk));
    let latent = noise(hashString(`${voice.style}|${speed.toFixed(3)}|${plain}`), dim * frames);
    const latentMask = new ort.Tensor('float32', new Float32Array(frames).fill(1), [1, 1, frames]);
    const total = new ort.Tensor('float32', Float32Array.of(STEPS), [1]);
    for (let step = 0; step < STEPS; step++) {
      const out = await sessions.vector.run({
        noisy_latent: new ort.Tensor('float32', latent, [1, dim, frames]),
        text_emb: textEmb,
        style_ttl: styleTtl,
        latent_mask: latentMask,
        text_mask: textMask,
        current_step: new ort.Tensor('float32', Float32Array.of(step), [1]),
        total_step: total,
      });
      latent = out.denoised_latent.data as Float32Array;
    }
    const wav = (await sessions.vocoder.run({ latent: new ort.Tensor('float32', latent, [1, dim, frames]) })).wav_tts.data as Float32Array;
    const start = performance.now();
    // New arrays of our own, so their buffers can be transferred to the main thread.
    const samples = line.finish(wav, rate);
    const visemes = line.visemes;
    return { samples, visemes, finishMs: performance.now() - start };
  } finally {
    line.free();
  }
}
