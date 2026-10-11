/**
 * The language model in Node, as the browser's LLM worker runs it: its 4-bit build from
 * `web/public/llm/`, the tokenizer (`@huggingface/tokenizers`), and every reply written by
 * `web/src/llm/generate.ts` (the worker's own code). Node has no WebGPU, so ONNX Runtime Web
 * runs it on the CPU here (its WASM build: slow with 4-bit weights, but the same loop and the
 * same text). Used by `check.mjs`. Needs `pnpm llm` first.
 */

import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';

const web = new URL('../../web/', import.meta.url);
const require = createRequire(new URL('package.json', web));
export const ort = require('onnxruntime-web');
// Its ES module (`require.resolve` finds the CommonJS build next to it).
const { Tokenizer } = await import(require.resolve('@huggingface/tokenizers').replace(/\.cjs$/, '.mjs'));
export const { generate, PrefixCache, countTokens } = await import(new URL('src/llm/generate.ts', web).href);
export const { formatChat } = await import(new URL('src/llm/chat.ts', web).href);

/** The default model, as `web/src/llm/models.ts` has it. */
export const SPEC = {
  dir: new URL('public/llm/lfm2.5-350m/', web),
  graph: 'model_q4.onnx',
  data: 'model_q4.onnx_data',
  chat: { kind: 'chatml', bos: '<|startoftext|>' },
  stop: ['<|im_end|>', '<|endoftext|>'],
};

const json = (name) => JSON.parse(readFileSync(new URL(name, SPEC.dir), 'utf8'));
export const tokenizer = new Tokenizer(json('tokenizer.json'), json('tokenizer_config.json'));
ort.env.logLevel = 'error';

/** A new session, as the worker makes one (`threads`: 1 unless asked). */
export async function engine(threads = 1) {
  ort.env.wasm.numThreads = threads;
  const session = await ort.InferenceSession.create(readFileSync(new URL(SPEC.graph, SPEC.dir)), {
    executionProviders: ['wasm'],
    graphOptimizationLevel: 'all',
    logSeverityLevel: 3,
    externalData: [{ path: SPEC.data, data: readFileSync(new URL(SPEC.data, SPEC.dir)) }],
  });
  const stopIds = new Set(SPEC.stop.map((t) => tokenizer.token_to_id(t)).filter((id) => id !== undefined));
  return { ort, session, tokenizer, stopIds, prefixes: new PrefixCache(4) };
}
