/**
 * Language models the LLM worker can run, with everything that differs between them: where
 * their files are, chat format, stop tokens and sampling defaults. Built-in models are listed
 * here (the self-hosted default fetched by `tools/llm/fetch.mjs`, the others from Hugging Face);
 * players add others from Hugging Face (`hub.ts`, kept by `library.svelte.ts`). The generation
 * loop (`generate.ts`) reads each model's cache inputs from the model itself.
 *
 * Every model is used as its 4-bit build, as it is: small to download, fast on the GPU (WebGPU).
 * On the CPU (WASM) 4-bit runs slowly, so there the device test decides whether it's usable.
 */

import type { ChatFormat } from './chat';
import type { GenParams } from './generate';

/** A file: its name under the model's `url`, size in bytes (progress bars) and SHA-256 if known (checked on download). */
export type ModelFile = [name: string, bytes: number, sha256?: string];

export interface ModelSpec {
  id: string;
  /** Shown in settings. */
  label: string;
  /** A line about it for settings. */
  about: string;
  /** Changes when any file does: older device-test results are dropped. */
  version: string;
  /** Where its files are: a folder under the app's base URL, or a full URL (Hugging Face, pinned to a revision). */
  url: string;
  tokenizer: ModelFile;
  tokenizerConfig: ModelFile;
  /** The 4-bit graph, and its weights file if they're outside it. */
  graph: ModelFile;
  data?: ModelFile;
  chat: ChatFormat;
  /** Tokens that end a reply. */
  stop: string[];
  defaults: GenParams;
  licence: string;
  /** Added by the player (not built in). */
  added?: boolean;
}

const DEFAULTS: GenParams = { maxTokens: 32, temperature: 0.7, topK: 40, topP: 0.9, repetitionPenalty: 1.15, seed: 1, stopAtNewline: true };

export const LFM25_350M: ModelSpec = {
  id: 'lfm2.5-350m',
  label: 'LFM2.5 350M',
  about: 'Liquid AI; the default. Fast, and good at keeping to the moment.',
  version: 'lfm2.5-350m@7dd4999',
  url: 'llm/lfm2.5-350m/',
  tokenizer: ['tokenizer.json', 3_297_793, '29d43b4be8e8a896fefd7cd836ca6d6b4eedd249f823866ce0453b368e646f49'],
  tokenizerConfig: ['tokenizer_config.json', 3_825, 'e70cd070c2696bf1b0f1ad303c34502753670cca6f844637821cb1c37e6bea8d'],
  graph: ['model_q4.onnx', 183_442, 'd1a705712e93aafaba1346b32245fa59e7857a46e2272003c0e8c524977e0de8'],
  data: ['model_q4.onnx_data', 293_629_952, '71ec6ad38a4c463dcb3dba671d06a1d9861be3a23e51290d818b95c0b7d2a5db'],
  chat: { kind: 'chatml', bos: '<|startoftext|>' },
  stop: ['<|im_end|>', '<|endoftext|>'],
  defaults: DEFAULTS,
  licence: 'LFM Open License v1.0',
};

export const SMOLLM2_360M: ModelSpec = {
  id: 'smollm2-360m',
  label: 'SmolLM2 360M',
  about: 'Hugging Face; downloaded from Hugging Face.',
  version: 'smollm2-360m@a10cc15',
  url: 'https://huggingface.co/HuggingFaceTB/SmolLM2-360M-Instruct/resolve/a10cc1512eabd3dde888204e902eca88bddb4951/',
  tokenizer: ['tokenizer.json', 2_104_556, '9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c'],
  tokenizerConfig: ['tokenizer_config.json', 3_764, '4ec77d44f62efeb38d7e044a1db318f6a939438425312dfa333b8382dbad98df'],
  graph: ['onnx/model_q4.onnx', 387_943_246, 'a4ffda96e65beafc6f6cef0cbcf9fdc1cbdd79c230906bf3897d190547c7a596'],
  chat: { kind: 'chatml', bos: '' },
  stop: ['<|im_end|>', '<|endoftext|>'],
  defaults: DEFAULTS,
  licence: 'Apache-2.0',
};

export const GEMMA3_270M: ModelSpec = {
  id: 'gemma-3-270m',
  label: 'Gemma 3 270M',
  about: 'Google; the smallest. Downloaded from Hugging Face.',
  version: 'gemma-3-270m@2dbbfdb',
  url: 'https://huggingface.co/onnx-community/gemma-3-270m-it-ONNX/resolve/2dbbfdb1b59bd034eb959428c6a7da9dd7ea27f0/',
  tokenizer: ['tokenizer.json', 20_323_013, '55da1312bdf1d7d8fe8d9d1b3eed04086261149e6034e0ac3f8c633b67f5aac8'],
  tokenizerConfig: ['tokenizer_config.json', 2_313, '8ae5c9297e0ca85077c0f4952f85251b4773af7abc4673b62b54bc0b609a9667'],
  graph: ['onnx/model_q4.onnx', 242_522, 'cb3615746a988839308507933d4e0501ff695e747a0a0ce15c0ef53083f8ec54'],
  data: ['onnx/model_q4.onnx_data', 322_933_248, '25c4a14af32fc1011a90504076024b0e1f65f915e9690b44a5484200d2e20851'],
  chat: { kind: 'gemma', bos: '<bos>' },
  stop: ['<end_of_turn>', '<eos>'],
  defaults: DEFAULTS,
  licence: 'Gemma Terms of Use',
};

export const BUILT_IN: readonly ModelSpec[] = [LFM25_350M, SMOLLM2_360M, GEMMA3_270M];
export const DEFAULT_MODEL = LFM25_350M;

/** Download size in MB: tokenizer, graph and weights (the runtime not included). */
export function downloadMb(spec: ModelSpec): number {
  const files = [spec.tokenizer, spec.tokenizerConfig, spec.graph, ...(spec.data ? [spec.data] : [])];
  return files.reduce((n, [, bytes]) => n + bytes, 0) / 1e6;
}
