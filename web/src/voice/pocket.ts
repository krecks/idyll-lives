/**
 * One line of speech with Kyutai's Pocket TTS, the same way everywhere: the voice worker
 * (`tts.worker.ts`) and the Node tools (`tools/voice/check.mjs`, `samples.mjs`) call this, so the
 * checks hear exactly what the game plays. Plain TypeScript with type-only imports (Node runs it
 * by stripping the types). Ported from the ONNX export's reference code
 * (`pocket_tts_onnx.py`, KevinAHM/pocket-tts-onnx).
 *
 * Pocket TTS speaks like a reference clip. The clip (a bundled voice or a player's recording) is
 * given the voice's clip pitch and size by our voice WASM (`reference`), turned into embeddings
 * by the Mimi encoder (kept in `store` when there is one, so a voice is encoded once), and fed to
 * the transformer once (`prepareVoice`), whose state is kept. Then
 * per line: the text's embeddings, then one 80 ms frame at a time the transformer gives a
 * conditioning vector and an end-of-speech score, the flow network turns noise into the frame's
 * latent (one Euler step), and the Mimi decoder turns the latents into 24 kHz audio. The noise is
 * ours, from a hash of the voice, the text and the frame (`noise`), so the same line in the same
 * voice gives the same samples (the clip cache relies on it). Our WASM then trims the clip,
 * gives it the resident's own pitch, size and speed and finds its mouth shapes.
 */

import type { VoiceParams } from './voices';

export interface OrtTensor {
  readonly data: unknown;
  readonly dims: readonly number[];
}
/** The pieces of ONNX Runtime used here (the worker's `onnxruntime-web/wasm`, or Node's `onnxruntime-web`). */
export interface Ort {
  Tensor: new (type: 'int64' | 'float32' | 'bool', data: BigInt64Array | Float32Array | Uint8Array, dims: readonly number[]) => OrtTensor;
}
export interface Session {
  readonly outputNames: readonly string[];
  run(feeds: Record<string, OrtTensor>): Promise<Record<string, OrtTensor>>;
}
export interface Tokenizer {
  encode(text: string): { ids: number[] };
}
/** `Line` from the phonemizer WASM (`crates/voice-wasm`). */
interface Line {
  readonly text: string;
  finish(samples: Float32Array): Float32Array;
  /** After `finish`: the clip's mouth shapes, flat (`parseVisemes`). */
  readonly visemes: Float32Array;
  free(): void;
}
export interface Phonemizer {
  line(text: string, speed: number, pitch: number, depth: number): Line;
}
/** Where encoded voices are kept between sessions (the worker's Cache API); keys include the model's version. */
export interface EmbeddingStore {
  get(key: string): Promise<Float32Array | null>;
  put(key: string, embeddings: Float32Array): Promise<void>;
}

/** One state tensor of a model, from `bundle.json`. */
interface StateEntry {
  input_name: string;
  output_name: string;
  dtype: 'float32' | 'int64' | 'bool';
  shape: number[];
  fill: 'nan' | 'ones' | 'zeros' | 'empty';
  module: string;
  key: string;
}
export interface Bundle {
  flow_lm_state_manifest: StateEntry[];
  mimi_state_manifest: StateEntry[];
  latent_dim: number;
  conditioning_dim: number;
  frame_rate: number;
  max_token_per_chunk: number;
}

/** A conditioned voice, kept small: each attention cache only up to the positions it uses. */
interface VoiceState {
  tensors: Record<string, { data: Float32Array | BigInt64Array | Uint8Array; dims: readonly number[]; used?: number }>;
}

export interface Engine {
  ort: Ort;
  sessions: { encoder: Session; text: Session; main: Session; flow: Session; decoder: Session };
  tokenizer: Tokenizer;
  bundle: Bundle;
  /** `bos_before_voice.npy`: embeddings put before a voice's, [frames × conditioning_dim]. */
  bos: Float32Array;
  phonemizer: Phonemizer;
  /** Our WASM's `reference`: a clip with a voice's pitch and size. */
  reference: (clip: Float32Array, pitch: number, depth: number) => Float32Array;
  /** Clips by name: `preset:<name>` (the bundled voices) and `rec:<sha>` (players' recordings). */
  clips: Map<string, Float32Array>;
  /** Conditioned voices by `voiceKey`, least recently used first. */
  voices: Map<string, VoiceState>;
  store?: EmbeddingStore | null;
}

/** Conditioned voices kept (about 6 MB each). */
const VOICES_KEPT = 12;
/** A frame ends the line once its end-of-speech score is above this (the reference's). */
const EOS_LOGIT = -4;
/** The reference's estimate for how long a line may take: 3 tokens a second, plus 2 s. */
const TOKENS_PER_SECOND = 3;
const PADDING_SECONDS = 2;
/** Latent frames decoded at a time. */
const DECODE_CHUNK = 15;

/** A clip's pitch and size are rounded to this, so near-identical voices share one. */
const STEP = 0.02;
const snap = (v: number) => Math.round(v / STEP) * STEP;

/** Which conditioned voice a resident's voice uses: the clip and the pitch and size put into it (the rest is per line). */
export function voiceKey(v: VoiceParams): string {
  return `${v.clip}|${snap(v.clipPitch).toFixed(2)}|${snap(v.clipDepth).toFixed(2)}`;
}

function fill(e: StateEntry): { data: Float32Array | BigInt64Array | Uint8Array; dims: number[] } {
  const n = e.shape.reduce((a, b) => a * b, 1);
  if (e.dtype === 'int64') return { data: new BigInt64Array(n).fill(e.fill === 'ones' ? 1n : 0n), dims: e.shape };
  if (e.dtype === 'bool') return { data: new Uint8Array(n).fill(e.fill === 'ones' ? 1 : 0), dims: e.shape };
  return { data: new Float32Array(n).fill(e.fill === 'nan' ? NaN : e.fill === 'ones' ? 1 : 0), dims: e.shape };
}

function initState(ort: Ort, manifest: StateEntry[]): Record<string, OrtTensor> {
  const state: Record<string, OrtTensor> = {};
  for (const e of manifest) {
    const { data, dims } = fill(e);
    state[e.input_name] = new ort.Tensor(e.dtype, data, dims);
  }
  return state;
}

function update(state: Record<string, OrtTensor>, out: Record<string, OrtTensor>, manifest: StateEntry[]): void {
  for (const e of manifest) state[e.input_name] = out[e.output_name];
}

/** A transformer state made small: attention caches cut to the positions their `step` says are used. */
function compact(state: Record<string, OrtTensor>, manifest: StateEntry[]): VoiceState {
  const steps = new Map<string, number>();
  for (const e of manifest) if (e.key === 'step') steps.set(e.module, Number((state[e.input_name].data as BigInt64Array)[0]));
  const tensors: VoiceState['tensors'] = {};
  for (const e of manifest) {
    const t = state[e.input_name];
    const data = t.data as Float32Array | BigInt64Array | Uint8Array;
    const used = steps.get(e.module);
    if (e.key === 'cache' && used !== undefined && e.shape.length === 5) {
      // [2 (keys, values), 1, positions, heads, size]: keep the first `used` positions of each.
      const [kv, , positions, heads, size] = e.shape;
      const row = heads * size;
      const out = new Float32Array(kv * used * row);
      for (let k = 0; k < kv; k++) out.set((data as Float32Array).subarray(k * positions * row, k * positions * row + used * row), k * used * row);
      tensors[e.input_name] = { data: out, dims: e.shape, used };
    } else {
      tensors[e.input_name] = { data: data.slice(), dims: t.dims };
    }
  }
  return { tensors };
}

function expand(ort: Ort, v: VoiceState, manifest: StateEntry[]): Record<string, OrtTensor> {
  const state: Record<string, OrtTensor> = {};
  for (const e of manifest) {
    const t = v.tensors[e.input_name];
    if (t.used !== undefined) {
      const [kv, , positions, heads, size] = e.shape;
      const row = heads * size;
      const data = new Float32Array(kv * positions * row).fill(NaN);
      for (let k = 0; k < kv; k++) data.set((t.data as Float32Array).subarray(k * t.used * row, (k + 1) * t.used * row), k * positions * row);
      state[e.input_name] = new ort.Tensor('float32', data, e.shape);
    } else {
      state[e.input_name] = new ort.Tensor(e.dtype, t.data, t.dims);
    }
  }
  return state;
}

/** The conditioned voice for `voice`, made (and kept) if needed; `ms`: time spent making it (0 if kept). */
export async function prepareVoice(engine: Engine, voice: VoiceParams): Promise<{ state: VoiceState; ms: number }> {
  const key = voiceKey(voice);
  const kept = engine.voices.get(key);
  if (kept) {
    engine.voices.delete(key);
    engine.voices.set(key, kept);
    return { state: kept, ms: 0 };
  }
  const start = performance.now();
  const clip = engine.clips.get(voice.clip);
  if (!clip) throw new Error(`voice clip ${voice.clip} not loaded`);
  const { ort, sessions, bundle } = engine;
  let emb = (await engine.store?.get(key).catch(() => null)) ?? null;
  if (!emb) {
    const ref = engine.reference(clip, snap(voice.clipPitch), snap(voice.clipDepth));
    const enc = await sessions.encoder.run({ audio: new ort.Tensor('float32', ref, [1, 1, ref.length]) });
    emb = enc[sessions.encoder.outputNames[0]].data as Float32Array;
    await engine.store?.put(key, emb).catch(() => {});
  }
  const dim = bundle.conditioning_dim;
  const frames = emb.length / dim;
  const bosFrames = engine.bos.length / dim;
  const prompt = new Float32Array((bosFrames + frames) * dim);
  prompt.set(engine.bos, 0);
  prompt.set(emb, bosFrames * dim);
  const manifest = bundle.flow_lm_state_manifest;
  const state = initState(ort, manifest);
  const out = await sessions.main.run({
    sequence: new ort.Tensor('float32', new Float32Array(0), [1, 0, bundle.latent_dim]),
    text_embeddings: new ort.Tensor('float32', prompt, [1, bosFrames + frames, dim]),
    ...state,
  });
  update(state, out, manifest);
  const v = compact(state, manifest);
  engine.voices.set(key, v);
  for (const k of engine.voices.keys()) {
    if (engine.voices.size <= VOICES_KEPT) break;
    engine.voices.delete(k);
  }
  return { state: v, ms: performance.now() - start };
}

/** 32-bit FNV-1a of a string. */
function hashString(s: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 0x01000193);
  return h >>> 0;
}

/** A uniform (0, 1) value for element `i` of frame `step` of stream `seed`. */
function uniform(seed: number, step: number, i: number): number {
  let h = (Math.imul(seed ^ 0x9e3779b9, 0x85ebca6b) ^ Math.imul(step + 1, 0xc2b2ae35) ^ Math.imul(i + 1, 0x27d4eb2f)) >>> 0;
  h = Math.imul(h ^ (h >>> 16), 0x7feb352d);
  h = Math.imul(h ^ (h >>> 15), 0x846ca68b);
  h ^= h >>> 16;
  return ((h >>> 0) + 0.5) / 4294967296;
}

/** Normal noise with deviation `std` for one frame (Box–Muller on two hashed streams). */
function noise(seed: number, step: number, n: number, std: number): Float32Array {
  const out = new Float32Array(n);
  if (std <= 0) return out;
  for (let i = 0; i < n; i++) out[i] = std * Math.sqrt(-2 * Math.log(uniform(seed, step, 2 * i))) * Math.cos(2 * Math.PI * uniform(seed, step, 2 * i + 1));
  return out;
}

/** The reference's clean-up: one line, a capital first, a full stop if it ends in a letter or digit; and how many frames to keep after the end. */
export function prepareText(text: string): { text: string; after: number } {
  let t = text.replace(/[\r\n]+/g, ' ').replace(/ {2,}/g, ' ').trim();
  const words = t.split(' ').filter(Boolean).length;
  if (t && t[0] !== t[0].toUpperCase()) t = t[0].toUpperCase() + t.slice(1);
  if (/[\p{L}\p{N}]$/u.test(t)) t += '.';
  return { text: t, after: (words <= 4 ? 3 : 1) + 2 };
}

/** Text in pieces the model takes at once (at most `max` tokens each), split between sentences. */
function chunks(tokenizer: Tokenizer, text: string, max: number): string[] {
  const sentences = text.match(/[^.!?…]+[.!?…]+["'”’)\]]*\s*|[^.!?…]+$/g) ?? [text];
  const out: string[] = [];
  let current = '';
  for (const s of sentences.map((x) => x.trim()).filter(Boolean)) {
    const joined = current ? `${current} ${s}` : s;
    if (current && tokenizer.encode(prepareText(joined).text).ids.length > max) {
      out.push(current);
      current = s;
    } else {
      current = joined;
    }
  }
  if (current) out.push(current);
  return out;
}

/** Latent frames for one piece of text from a conditioned voice. */
async function latents(engine: Engine, voice: VoiceState, text: string, seed: number, temperature: number): Promise<Float32Array[]> {
  const { ort, sessions, bundle } = engine;
  const manifest = bundle.flow_lm_state_manifest;
  const { text: prepared, after } = prepareText(text);
  const ids = engine.tokenizer.encode(prepared).ids;
  const state = expand(ort, voice, manifest);
  const te = await sessions.text.run({ token_ids: new ort.Tensor('int64', BigInt64Array.from(ids, (i) => BigInt(i)), [1, ids.length]) });
  let emb = te[sessions.text.outputNames[0]];
  if (emb.dims.length === 2) emb = new ort.Tensor('float32', emb.data as Float32Array, [1, ...emb.dims]);
  const latent = bundle.latent_dim;
  let out = await sessions.main.run({ sequence: new ort.Tensor('float32', new Float32Array(0), [1, 0, latent]), text_embeddings: emb, ...state });
  update(state, out, manifest);
  const noText = new ort.Tensor('float32', new Float32Array(0), [1, 0, bundle.conditioning_dim]);
  const s = new ort.Tensor('float32', Float32Array.of(0), [1, 1]);
  const t = new ort.Tensor('float32', Float32Array.of(1), [1, 1]);
  let current = new ort.Tensor('float32', new Float32Array(latent).fill(NaN), [1, 1, latent]);
  const limit = Math.ceil((ids.length / TOKENS_PER_SECOND + PADDING_SECONDS) * bundle.frame_rate);
  const [condName, eosName] = sessions.main.outputNames;
  const frames: Float32Array[] = [];
  let eos = -1;
  for (let step = 0; step < limit; step++) {
    out = await sessions.main.run({ sequence: current, text_embeddings: noText, ...state });
    update(state, out, manifest);
    if (eos < 0 && (out[eosName].data as Float32Array)[0] > EOS_LOGIT) eos = step;
    if (eos >= 0 && step >= eos + after) break;
    // One Euler step of the flow from noise (s = 0) to the frame's latent (t = 1).
    const x = noise(seed, step, latent, Math.sqrt(temperature));
    const flow = await sessions.flow.run({ c: out[condName], s, t, x: new ort.Tensor('float32', x, [1, latent]) });
    const v = flow[sessions.flow.outputNames[0]].data as Float32Array;
    for (let i = 0; i < latent; i++) x[i] += v[i];
    frames.push(x);
    current = new ort.Tensor('float32', x, [1, 1, latent]);
  }
  return frames;
}

/** 24 kHz audio for latent frames (a fresh decoder state). */
async function decode(engine: Engine, frames: Float32Array[]): Promise<Float32Array> {
  const { ort, sessions, bundle } = engine;
  const manifest = bundle.mimi_state_manifest;
  const state = initState(ort, manifest);
  const latent = bundle.latent_dim;
  const parts: Float32Array[] = [];
  for (let i = 0; i < frames.length; i += DECODE_CHUNK) {
    const chunk = frames.slice(i, i + DECODE_CHUNK);
    const data = new Float32Array(chunk.length * latent);
    chunk.forEach((f, k) => data.set(f, k * latent));
    const out = await sessions.decoder.run({ latent: new ort.Tensor('float32', data, [1, chunk.length, latent]), ...state });
    parts.push(out[sessions.decoder.outputNames[0]].data as Float32Array);
    update(state, out, manifest);
  }
  const samples = new Float32Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) {
    samples.set(p, at);
    at += p.length;
  }
  return samples;
}

/**
 * 24 kHz mono for one line and its mouth shapes (flat). `prepMs`: time spent conditioning the
 * voice (0 when it was kept); `finishMs`: trimming, speed and mouth shapes.
 */
export async function synthesizeLine(engine: Engine, text: string, voice: VoiceParams): Promise<{ samples: Float32Array; visemes: Float32Array; prepMs: number; finishMs: number }> {
  const line = engine.phonemizer.line(text, voice.speed, voice.pitch, voice.depth);
  try {
    const plain = line.text;
    if (!plain) return { samples: new Float32Array(0), visemes: Float32Array.of(0, 0), prepMs: 0, finishMs: 0 };
    const { state, ms: prepMs } = await prepareVoice(engine, voice);
    const temperature = Math.max(0, voice.temperature);
    const seed = hashString(`${voiceKey(voice)}|${temperature.toFixed(2)}|${plain}`);
    const pieces: Float32Array[] = [];
    for (const [k, piece] of chunks(engine.tokenizer, plain, engine.bundle.max_token_per_chunk).entries()) {
      pieces.push(await decode(engine, await latents(engine, state, piece, (seed + k * 0x9e3779b9) >>> 0, temperature)));
    }
    const raw = new Float32Array(pieces.reduce((n, p) => n + p.length, 0));
    let at = 0;
    for (const p of pieces) {
      raw.set(p, at);
      at += p.length;
    }
    const start = performance.now();
    // New arrays of our own, so their buffers can be transferred to the main thread.
    const samples = line.finish(raw);
    const visemes = line.visemes;
    return { samples, visemes, prepMs, finishMs: performance.now() - start };
  } finally {
    line.free();
  }
}
