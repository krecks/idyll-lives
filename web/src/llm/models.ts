/**
 * The language model that writes residents' lines, with what the worker needs to run it: where
 * its files are, its chat format, stop tokens and sampling defaults. Liquid AI's LFM2.5 350M,
 * self-hosted (fetched by `cargo fetch-llm`), its upstream 4-bit build repacked smaller without
 * changing what it computes (`crates/tools/src/llmslim.rs`): small to download, fast on the GPU
 * (WebGPU). It runs only there (on the CPU 4-bit was too slow for
 * lines); without WebGPU residents say hand-written lines. The generation loop (`generate.ts`)
 * reads the model's cache inputs from the model itself.
 */

import type { ChatFormat } from './chat';
import type { GenParams } from './generate';

/**
 * A file: its name under the model's `url`, size in bytes (progress bars) and SHA-256 if known
 * (checked on download). A name ending in `z` is gzipped (`ml/assets.ts`, `packed`): the size is
 * the download's, the SHA-256 the unpacked file's.
 */
export type ModelFile = [name: string, bytes: number, sha256?: string];

export interface ModelSpec {
  id: string;
  /** Shown in settings. */
  label: string;
  /** A line about it for settings. */
  about: string;
  /** Changes when any file does: older device-test results are dropped. */
  version: string;
  /** Where its files are: a folder under the app's base URL. */
  url: string;
  tokenizer: ModelFile;
  tokenizerConfig: ModelFile;
  /** The 4-bit graph, and its weights file if they're outside it. */
  graph: ModelFile;
  data?: ModelFile;
  /** Positions it can read: a prompt and its reply together (its rotary tables' rows). */
  positions: number;
  chat: ChatFormat;
  /** Tokens that end a reply. */
  stop: string[];
  defaults: GenParams;
  licence: string;
}

const DEFAULTS: GenParams = { maxTokens: 32, temperature: 0.7, topK: 40, topP: 0.9, repetitionPenalty: 1.15, seed: 1, stopAtNewline: true };

export const MODEL: ModelSpec = {
  id: 'lfm2.5-350m',
  label: 'LFM2.5 350M',
  about: 'Liquid AI. Fast, and good at keeping to the moment.',
  version: 'lfm2.5-350m@7dd4999-slim',
  url: 'llm/lfm2.5-350m/',
  tokenizer: ['tokenizer.json', 3_297_793, '29d43b4be8e8a896fefd7cd836ca6d6b4eedd249f823866ce0453b368e646f49'],
  tokenizerConfig: ['tokenizer_config.json', 3_825, 'e70cd070c2696bf1b0f1ad303c34502753670cca6f844637821cb1c37e6bea8d'],
  graph: ['model_q4.onnx', 183_345, 'e17d03a87346295631d71358fd98abad930172bbf52819a26c34ca4d1dc4d04c'],
  data: ['model_q4.onnx_dataz', 195_523_654, '3b7570c237498b56b6276ceae327a11dd7f3e96dee8a4eaa3dad27548ee6a9de'],
  // `llmslim.rs`, POSITIONS.
  positions: 4096,
  chat: { bos: '<|startoftext|>' },
  stop: ['<|im_end|>', '<|endoftext|>'],
  defaults: DEFAULTS,
  licence: 'LFM Open License v1.0',
};

/** Download size in MB: tokenizer, graph and weights (the runtime not included). */
export function downloadMb(spec: ModelSpec): number {
  const files = [spec.tokenizer, spec.tokenizerConfig, spec.graph, ...(spec.data ? [spec.data] : [])];
  return files.reduce((n, [, bytes]) => n + bytes, 0) / 1e6;
}
