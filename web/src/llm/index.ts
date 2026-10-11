/**
 * A small language model in the browser, on ONNX Runtime Web (CPU or GPU), for anything that
 * wants short text written: chat messages in, text out. Nothing here knows about the game;
 * `talk/` builds residents' lines on it.
 *
 * - `service.svelte.ts`: the app-wide worker, its status and `generateText`;
 * - `models.ts`: the models it can run; `chat.ts`: chat formats; `generate.ts`: the
 *   generation loop (shared with `tools/llm/check.mjs`).
 */

export type { ChatMessage } from './chat';
export type { GenParams, GenResult } from './generate';
export { BUILT_IN, DEFAULT_MODEL, downloadMb, type ModelSpec } from './models';
export { addModel, allModels, library, modelById, removeModel } from './library.svelte';
export { describeHubModel, repoOf } from './hub';
export { ensureLlm, generateText, llmReady, llmStats, llmStatus, unloadLlm, type LlmState, type LlmStats } from './service.svelte';
