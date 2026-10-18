/**
 * Text generation with a decoder-only ONNX language model, the same way everywhere: the LLM
 * worker (`llm.worker.ts`) and the Node check (`tools/llm/check.mjs`) call this. Plain
 * TypeScript without runtime imports (Node runs it by stripping the types).
 *
 * The model's state between steps is read from the model itself: every input named `past_…`
 * (attention keys and values, `past_key_values.N.key`, or a convolution's state, `past_conv.N`)
 * is fed back from the output of the same name with `present` (`present.N.key`,
 * `present_conv.N`). It starts empty: sequence lengths 0, fixed-size states zero. So models with
 * attention only and hybrids (LFM2: convolutions and attention) run alike.
 *
 * A prompt comes in parts (`chat.ts`, `formatChat`): the state after each part but the last is
 * kept in a small cache, keyed by the text up to there, and the next prompt that starts the same
 * reads only from its deepest cached part on. Residents' lines: the worked examples are read once
 * for everyone, a resident's persona once for them, and each line costs only the moment.
 */

/** The pieces of ONNX Runtime used here (`onnxruntime-web`, CPU or WebGPU build). */
export interface Tensor {
  readonly dims: readonly number[];
  readonly type: string;
  readonly data: unknown;
  /** Where the data is (`cpu`, or `gpu-buffer` for outputs kept on the GPU). */
  readonly location?: string;
  getData?(): Promise<unknown>;
  dispose?(): void;
}
export type TensorCtor = new (type: 'int64' | 'float32' | 'float16', data: BigInt64Array | Float32Array | Uint16Array, dims: readonly number[]) => Tensor;
export interface ValueMetadata {
  name: string;
  isTensor: boolean;
  type?: string;
  shape?: readonly (number | string)[];
}
export interface LlmSession {
  readonly inputNames: readonly string[];
  readonly outputNames: readonly string[];
  readonly inputMetadata: readonly ValueMetadata[];
  run(feeds: Record<string, Tensor>): Promise<Record<string, Tensor>>;
}
/** The pieces of `@huggingface/tokenizers` used here. */
export interface LlmTokenizer {
  encode(text: string, options: { add_special_tokens: boolean }): { ids: number[] };
  decode(ids: number[], options: { skip_special_tokens: boolean }): string;
}

export interface GenParams {
  /** Tokens to write at most. */
  maxTokens: number;
  /** 0: always the likeliest token (greedy). */
  temperature: number;
  /** Sample among the `topK` likeliest tokens, and of those the fewest that make up `topP`. */
  topK: number;
  topP: number;
  /** Tokens already written are this much less likely again (1: off). */
  repetitionPenalty: number;
  /** Same seed, prompt and model: the same text. */
  seed: number;
  /** End at the first line break after some text (replies of one line). */
  stopAtNewline?: boolean;
  /** End where any of these is written (left out of the text): a closing quote after `Mia: "`. */
  stopAt?: string[];
  /**
   * The reply may not start the way any of these does (their first token is barred): example
   * lines in the prompt show the tone without being copied.
   */
  avoidStart?: string[];
  /**
   * Exactly this many sentences of plain speech: until the last one is finished only plain
   * text can be written (`PLAIN`: nothing that would end the reply, like the end of the turn, a
   * quote or a line break, and no emoji or other scripts), and the reply ends with it. Small
   * models asked for more than two sentences often stop at two, or drift off once they can't.
   */
  sentences?: number;
}

export interface GenResult {
  /** The reply, special tokens left out. */
  text: string;
  /** Token ids written. */
  ids: number[];
  promptTokens: number;
  /** Prompt tokens whose state came from the cache. */
  cachedTokens: number;
  /** Time to read the prompt (to the first token's choice), and to write the rest, ms. */
  prefillMs: number;
  decodeMs: number;
  /** `deadline`: time ran out with some text written (`aborted` until the worker tells them apart). */
  stop: 'eos' | 'newline' | 'text' | 'length' | 'aborted' | 'deadline';
  /** Some logits weren't finite numbers (a broken model or runtime: the text is garbage). */
  nonFinite: boolean;
}

/** Model state: tensors by input name (`past_…`). */
type State = Map<string, Tensor>;

interface Cached {
  state: State;
  length: number;
}

/** States after prompt prefixes, least recently used first out. */
export class PrefixCache {
  private readonly entries = new Map<string, Cached>();
  /** Tensors held by entries (never disposed while held). */
  readonly held = new Set<Tensor>();
  hits = 0;
  misses = 0;
  private readonly max: number;

  constructor(max = 8) {
    this.max = max;
  }

  get(key: string): Cached | undefined {
    const e = this.entries.get(key);
    if (e) {
      this.entries.delete(key);
      this.entries.set(key, e);
    }
    return e;
  }

  set(key: string, e: Cached): void {
    this.entries.set(key, e);
    for (const t of e.state.values()) this.held.add(t);
    while (this.entries.size > this.max) {
      const [oldest, old] = this.entries.entries().next().value as [string, Cached];
      this.entries.delete(oldest);
      for (const t of old.state.values()) {
        this.held.delete(t);
        t.dispose?.();
      }
    }
  }

  clear(): void {
    for (const e of this.entries.values()) for (const t of e.state.values()) t.dispose?.();
    this.entries.clear();
    this.held.clear();
  }

  get size(): number {
    return this.entries.size;
  }
}

export interface LlmEngine {
  ort: { Tensor: TensorCtor };
  session: LlmSession;
  tokenizer: LlmTokenizer;
  /** Token ids that end a reply. */
  stopIds: ReadonlySet<number>;
  prefixes: PrefixCache;
}

/** A finished sentence: its closing punctuation (and a closing quote or bracket), then a space or the end. */
const SENTENCE_END = /[.!?…]+["'”’)\]]*(?=\s|$)/g;

/** How many sentences `text` has finished. */
export function sentencesIn(text: string): number {
  return text.match(SENTENCE_END)?.length ?? 0;
}

/** Plain speech: Latin letters (with accents), digits, spaces and plain punctuation (as `talk/clean.ts` keeps). */
const PLAIN = /^[A-Za-zÀ-ÖØ-öø-ÿ0-9 .,!?'’\-—…:;]+$/;

/** A sentence's end with a closing quote or bracket in one token (`."`, `!”`): the punctuation alone stands in for it. */
const END_AND_QUOTE = /^(\s?[.!?…]+)["'”’」』)\]]+\s*$/;

/**
 * For writing plain speech only, per engine (one pass over the vocabulary, about 70 ms, once):
 * the tokens that aren't plain (special tokens, quotes, line breaks, emoji, other scripts), and
 * those that end a sentence with a quote, each with the plain punctuation token written instead.
 */
interface PlainOnly {
  barred: number[];
  ends: Map<number, number>;
}
const plainOnly = new WeakMap<LlmEngine, PlainOnly>();
function plainTokens(engine: LlmEngine, vocab: number): PlainOnly {
  let p = plainOnly.get(engine);
  if (!p) {
    const { tokenizer } = engine;
    p = { barred: [], ends: new Map() };
    for (let id = 0; id < vocab; id++) {
      if (engine.stopIds.has(id)) {
        p.barred.push(id);
        continue;
      }
      const t = tokenizer.decode([id], { skip_special_tokens: true });
      if (PLAIN.test(t)) continue;
      const end = END_AND_QUOTE.exec(t);
      const plain = end ? tokenizer.encode(end[1], { add_special_tokens: false }).ids : [];
      if (plain.length === 1) p.ends.set(id, plain[0]);
      else p.barred.push(id);
    }
    plainOnly.set(engine, p);
  }
  return p;
}

/** The output that carries an input's next state (`past_key_values.2.key` → `present.2.key`, `past_conv.0` → `present_conv.0`). */
export function presentName(past: string): string {
  return past.replace(/^past_key_values\./, 'present.').replace(/^past_/, 'present_');
}

/** The model's empty state: zero-length sequences, zeroed fixed-size states. */
function emptyState(engine: LlmEngine): State {
  const state: State = new Map();
  for (const m of engine.session.inputMetadata) {
    if (!m.isTensor || !m.name.startsWith('past_')) continue;
    // The first axis is the batch (one); other named axes grow with the sequence (empty yet).
    const dims = (m.shape ?? []).map((d, i) => (typeof d === 'number' ? d : i === 0 ? 1 : 0));
    const n = dims.reduce((a, b) => a * b, 1);
    const half = m.type === 'float16';
    state.set(m.name, new engine.ort.Tensor(half ? 'float16' : 'float32', half ? new Uint16Array(n) : new Float32Array(n), dims));
  }
  return state;
}

/** Frees a state's tensors, except those the prefix cache holds. */
function release(engine: LlmEngine, state: State): void {
  for (const t of state.values()) if (!engine.prefixes.held.has(t)) t.dispose?.();
}

/**
 * Runs the model on `ids` after `past` tokens in `state`: the next state, and the logits for
 * the token after the last (if wanted). The old state is not freed here.
 */
async function step(engine: LlmEngine, ids: readonly number[], state: State, past: number, wantLogits: boolean): Promise<{ state: State; logits: Float32Array | null }> {
  const { Tensor } = engine.ort;
  const names = engine.session.inputNames;
  const n = ids.length;
  const feeds: Record<string, Tensor> = {
    input_ids: new Tensor('int64', BigInt64Array.from(ids, (id) => BigInt(id)), [1, n]),
    attention_mask: new Tensor('int64', new BigInt64Array(past + n).fill(1n), [1, past + n]),
  };
  if (names.includes('num_logits_to_keep')) feeds.num_logits_to_keep = new Tensor('int64', BigInt64Array.of(1n), []);
  if (names.includes('position_ids')) feeds.position_ids = new Tensor('int64', BigInt64Array.from({ length: n }, (_, i) => BigInt(past + i)), [1, n]);
  for (const [name, t] of state) feeds[name] = t;
  const out = await engine.session.run(feeds);
  const next: State = new Map();
  for (const name of state.keys()) {
    const t = out[presentName(name)];
    if (!t) throw new Error(`model has no output ${presentName(name)} for ${name}`);
    next.set(name, t);
  }
  let logits: Float32Array | null = null;
  const raw = out.logits;
  if (wantLogits) {
    const data = raw.location && raw.location !== 'cpu' && raw.getData ? await raw.getData() : raw.data;
    const vocab = raw.dims[raw.dims.length - 1];
    logits = lastRow(data as Float32Array | Uint16Array, vocab, raw.type === 'float16');
  }
  raw.dispose?.();
  return { state: next, logits };
}

/** The last `vocab` logits as float32 (copied: the tensor is freed). */
function lastRow(data: Float32Array | Uint16Array, vocab: number, half: boolean): Float32Array {
  const start = data.length - vocab;
  if (!half) return (data as Float32Array).slice(start);
  const out = new Float32Array(vocab);
  for (let i = 0; i < vocab; i++) out[i] = halfToFloat((data as Uint16Array)[start + i]);
  return out;
}

function halfToFloat(h: number): number {
  const s = h & 0x8000 ? -1 : 1;
  const e = (h >> 10) & 0x1f;
  const f = h & 0x3ff;
  if (e === 0) return s * 2 ** -14 * (f / 1024);
  if (e === 31) return f ? NaN : s * Infinity;
  return s * 2 ** (e - 15) * (1 + f / 1024);
}

/**
 * The reply to a prompt (its `parts`, see above). `aborted` is asked between tokens;
 * `useCache: false` reads the whole prompt in one go and keeps nothing (the checks compare both).
 */
export async function generate(
  engine: LlmEngine,
  prompt: { parts: readonly string[] },
  params: GenParams,
  aborted: () => boolean = () => false,
  useCache = true,
): Promise<GenResult> {
  const { tokenizer } = engine;
  const t0 = performance.now();
  const parts = prompt.parts.length ? prompt.parts : [''];
  const partIds = parts.map((p) => tokenizer.encode(p, { add_special_tokens: false }).ids);
  const promptTokens = partIds.reduce((n, x) => n + x.length, 0);
  let state: State = emptyState(engine);
  let past = 0;
  let cachedTokens = 0;
  let first: number[];
  if (!useCache) {
    first = partIds.flat();
  } else {
    // The deepest part already read: start after it.
    const keys = parts.slice(0, -1).map((_, i) => parts.slice(0, i + 1).join(''));
    let from = 0;
    for (let i = keys.length - 1; i >= 0; i--) {
      const hit = engine.prefixes.get(keys[i]);
      if (!hit) continue;
      release(engine, state);
      state = hit.state;
      past = cachedTokens = hit.length;
      from = i + 1;
      break;
    }
    if (from > 0) engine.prefixes.hits++;
    if (from < keys.length) engine.prefixes.misses++;
    // Read the rest of the kept parts, keeping the state after each.
    for (let i = from; i < keys.length; i++) {
      if (!partIds[i].length) continue;
      const next = (await step(engine, partIds[i], state, past, false)).state;
      release(engine, state);
      state = next;
      past += partIds[i].length;
      engine.prefixes.set(keys[i], { state, length: past });
    }
    first = partIds[partIds.length - 1];
  }
  let ran = await step(engine, first, state, past, true);
  release(engine, state);
  state = ran.state;
  past += first.length;
  const prefillMs = performance.now() - t0;

  const t1 = performance.now();
  const rand = mulberry32(params.seed);
  const barred = new Set((params.avoidStart ?? []).map((t) => tokenizer.encode(t.trim(), { add_special_tokens: false }).ids[0]).filter((id) => id !== undefined));
  const ids: number[] = [];
  let text = '';
  let stop: GenResult['stop'] = 'length';
  let nonFinite = false;
  const wanted = params.sentences ?? 0;
  for (;;) {
    const logits = ran.logits!;
    if (!ids.length) for (const id of barred) logits[id] = -Infinity;
    // Not done yet: only plain speech, so it can't end (or drift off) here.
    const plain = wanted > 0 && sentencesIn(text) < wanted ? plainTokens(engine, logits.length) : null;
    if (plain) for (const id of plain.barred) logits[id] = -Infinity;
    let id = sample(logits, params, ids, rand);
    // Only the chosen token has to be a number: `sample` skips the ones that aren't, and a broken
    // model shows here (the text is garbage, and the caller throws it away).
    nonFinite ||= !Number.isFinite(logits[id]);
    // A sentence's end with a closing quote: the sentence ends, the quote doesn't come.
    if (plain) id = plain.ends.get(id) ?? id;
    if (engine.stopIds.has(id)) {
      stop = 'eos';
      break;
    }
    ids.push(id);
    text = tokenizer.decode(ids, { skip_special_tokens: true });
    if (params.stopAtNewline) {
      const start = text.length - text.trimStart().length;
      const nl = text.indexOf('\n', start);
      if (nl >= 0) {
        text = text.slice(0, nl);
        stop = 'newline';
        break;
      }
    }
    const at = firstOf(text, params.stopAt);
    if (at >= 0) {
      text = text.slice(0, at);
      stop = 'text';
      break;
    }
    // The last sentence wanted is finished: that's the reply.
    if (wanted > 0 && /[.!?…]["'”’)\]]*$/.test(text.trimEnd()) && sentencesIn(text.trimEnd()) >= wanted) {
      stop = 'text';
      break;
    }
    if (ids.length >= params.maxTokens) break;
    if (aborted()) {
      stop = 'aborted';
      break;
    }
    ran = await step(engine, [id], state, past, true);
    release(engine, state);
    state = ran.state;
    past++;
  }
  release(engine, state);
  return {
    text: text.trim(),
    ids,
    promptTokens,
    cachedTokens,
    prefillMs,
    decodeMs: performance.now() - t1,
    stop,
    nonFinite,
  };
}

/** Token count of some text (prompt budgets). */
export function countTokens(tokenizer: LlmTokenizer, text: string): number {
  return tokenizer.encode(text, { add_special_tokens: false }).ids.length;
}

/** Where the first of `stops` is in `text`, or -1. */
function firstOf(text: string, stops: readonly string[] | undefined): number {
  let first = -1;
  for (const s of stops ?? []) {
    const at = s ? text.indexOf(s) : -1;
    if (at >= 0 && (first < 0 || at < first)) first = at;
  }
  return first;
}

/** A small seeded random number generator (0..1). */
export function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/**
 * The next token: repetition penalty on tokens already written, then the likeliest (temperature
 * 0) or a draw among the top-k, top-p tokens at the temperature.
 */
export function sample(logits: Float32Array, p: GenParams, written: readonly number[], rand: () => number): number {
  const scores = logits;
  if (p.repetitionPenalty !== 1) {
    for (const id of new Set(written)) {
      const s = scores[id];
      scores[id] = s > 0 ? s / p.repetitionPenalty : s * p.repetitionPenalty;
    }
  }
  if (p.temperature <= 0) return argmax(scores);
  // The top k in a min-heap (its root is the weakest kept token), then best first.
  const k = Math.max(1, Math.min(p.topK, scores.length));
  const top: number[] = [];
  for (let i = 0; i < scores.length; i++) {
    const s = scores[i];
    if (!Number.isFinite(s)) continue;
    if (top.length === k) {
      if (s <= scores[top[0]]) continue;
      top[0] = i;
      // Sift the new token down to where it belongs.
      let at = 0;
      for (;;) {
        const left = at * 2 + 1;
        const right = left + 1;
        let small = at;
        if (left < k && scores[top[left]] < scores[top[small]]) small = left;
        if (right < k && scores[top[right]] < scores[top[small]]) small = right;
        if (small === at) break;
        [top[at], top[small]] = [top[small], top[at]];
        at = small;
      }
    } else {
      top.push(i);
      // Sift it up.
      let at = top.length - 1;
      while (at > 0) {
        const parent = (at - 1) >> 1;
        if (scores[top[parent]] <= scores[top[at]]) break;
        [top[at], top[parent]] = [top[parent], top[at]];
        at = parent;
      }
    }
  }
  if (!top.length) return argmax(scores);
  top.sort((a, b) => scores[b] - scores[a]);
  const max = scores[top[0]];
  const weights = top.map((i) => Math.exp((scores[i] - max) / p.temperature));
  const total = weights.reduce((a, b) => a + b, 0);
  // Of those, the fewest that together are at least `topP` likely.
  let keep = 0;
  for (let acc = 0; keep < top.length && acc < p.topP * total; keep++) acc += weights[keep];
  keep = Math.max(1, keep);
  const kept = weights.slice(0, keep).reduce((a, b) => a + b, 0);
  let r = rand() * kept;
  for (let j = 0; j < keep; j++) {
    r -= weights[j];
    if (r <= 0) return top[j];
  }
  return top[keep - 1];
}

function argmax(xs: Float32Array): number {
  let best = 0;
  for (let i = 1; i < xs.length; i++) if (xs[i] > xs[best]) best = i;
  return best;
}
