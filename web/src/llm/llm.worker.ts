/**
 * Language model worker: writes replies to chat messages, one at a time, off the main thread.
 * ONNX Runtime Web runs the model on the GPU (WebGPU); the tokenizer is
 * `@huggingface/tokenizers`; `generate.ts` writes each reply. Knows nothing about the game.
 *
 * Everything is self-hosted, fetched once and kept in the Cache API, so the next session starts
 * without downloading.
 */

import { Tokenizer } from '@huggingface/tokenizers';
import { fetchOnce, forget, openCache, prune } from '../ml/assets';
import { configureOrt, importOrt, ORT_WASM, sessionOptions } from '../ml/ort';
import { formatChat, type ChatMessage } from './chat';
import { generate, PrefixCache, type GenParams, type LlmEngine, type LlmSession, type TensorCtor } from './generate';
import type { ModelSpec } from './models';
import type { FromLlmWorker, ToLlmWorker } from './protocol';

interface WorkerScope {
  onmessage: ((e: MessageEvent<ToLlmWorker>) => void) | null;
  postMessage(message: FromLlmWorker, transfer?: Transferable[]): void;
}
const scope = self as WorkerScope;

/** Bumped when the files change in place; older caches are deleted. (New files under new names don't need it.) */
const CACHE_PREFIX = 'littlelives-llm-';
const CACHE = `${CACHE_PREFIX}v1`;

let engine: LlmEngine | null = null;
let spec: ModelSpec | null = null;
let loading: Promise<void> | null = null;
/** Replies are written one after another: a run must finish before the next starts. */
let queue: Promise<void> = Promise.resolve();
const aborted = new Set<number>();

scope.onmessage = (e) => {
  const msg = e.data;
  if (msg.type === 'load') {
    if (!loading) {
      loading = load(msg.base, msg.model).catch((err) => {
        loading = null;
        scope.postMessage({ type: 'error', message: String(err) });
        throw err;
      });
      loading.catch(() => {});
    }
  } else if (msg.type === 'generate') {
    queue = queue.then(() => write(msg.id, msg.messages, msg.reply, msg.lead, msg.params, msg.deadline));
  } else if (msg.type === 'abort') {
    aborted.add(msg.id);
  }
};

async function write(id: number, messages: ChatMessage[], reply: string, lead: string, params: GenParams, deadline: number): Promise<void> {
  try {
    if (!loading) throw new Error('language model not loaded');
    await loading;
    if (aborted.has(id) || Date.now() > deadline) {
      scope.postMessage({ type: 'skipped', id });
      return;
    }
    const result = await generate(engine!, formatChat(spec!.chat, messages, reply, lead), params, () => aborted.has(id) || Date.now() > deadline);
    // Out of time with something written: what's there (the caller keeps finished sentences).
    if (result.stop === 'aborted' && !aborted.has(id) && result.ids.length) scope.postMessage({ type: 'done', id, result: { ...result, stop: 'deadline' } });
    else if (result.stop === 'aborted') scope.postMessage({ type: 'skipped', id });
    else scope.postMessage({ type: 'done', id, result });
  } catch (err) {
    scope.postMessage({ type: 'error', id, message: String(err) });
  } finally {
    aborted.delete(id);
  }
}

/** Downloads (or takes from the cache) the runtime, tokenizer and model, and starts its session. */
async function load(base: string, model: ModelSpec): Promise<void> {
  const start = performance.now();
  const dir = new URL(model.url, new URL(base, location.href)).href;
  const { graph, data } = model;
  // `built`: imported through Vite. In dev their URLs aren't hashed, so a rebuilt runtime would
  // otherwise come back stale from the cache; those are fetched fresh there.
  const files: { url: string; size: number; sha256?: string; built?: boolean }[] = [
    { url: ORT_WASM.url, size: ORT_WASM.bytes, built: true },
    ...[model.tokenizer, model.tokenizerConfig, graph, ...(data ? [data] : [])].map(([name, size, sha256]) => ({ url: dir + name, size, sha256 })),
  ];
  const total = files.reduce((n, f) => n + f.size, 0);
  const done = files.map(() => 0);
  const report = () => scope.postMessage({ type: 'progress', loaded: done.reduce((a, b) => a + b, 0), total });
  const cache = await openCache(CACHE_PREFIX, CACHE);
  const bytes = await Promise.all(
    files.map((f, i) =>
      fetchOnce(
        f.built && import.meta.env.DEV ? null : cache,
        f.url,
        (n) => {
          done[i] = Math.min(n, f.size);
          report();
        },
        f.sha256,
      ),
    ),
  );
  void prune(cache, files.map((f) => f.url));
  const [ortWasm, tokenizerJson, tokenizerConfig, graphBytes, dataBytes] = bytes;
  const ort = await importOrt();
  configureOrt(ort, ortWasm);
  const decode = (b: ArrayBuffer) => JSON.parse(new TextDecoder().decode(b)) as object;
  const tokenizer = new Tokenizer(decode(tokenizerJson), decode(tokenizerConfig));
  const session = await ort.InferenceSession.create(new Uint8Array(graphBytes), {
    ...sessionOptions(),
    // The weights file is named in the graph by its name alone (a packed one without its `z`).
    ...(data ? { externalData: [{ path: data[0].split('/').pop()!.replace(/(\.onnx_data)z$/, '$1'), data: new Uint8Array(dataBytes) }] } : {}),
    // The model's state stays on the GPU between steps; only the logits come back.
    preferredOutputLocation: 'gpu-buffer' as const,
  });
  // The session has its own copy of the weights.
  if (data) forget(dir + data[0]);
  forget(dir + graph[0]);
  const stopIds = new Set(model.stop.map((t) => tokenizer.token_to_id(t)).filter((id): id is number => id !== undefined));
  engine = {
    ort: ort as unknown as { Tensor: TensorCtor },
    session: session as unknown as LlmSession,
    tokenizer,
    stopIds,
    prefixes: new PrefixCache(8),
  };
  spec = model;
  scope.postMessage({ type: 'ready', ms: performance.now() - start });
}
