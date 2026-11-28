/**
 * The language model in the browser (LFM2.5 350M on ONNX Runtime Web, WebGPU or WASM), for anything
 * that wants short text written: chat messages in, text out. Nothing here knows about the game;
 * `talk/` builds residents' lines on it.
 *
 * - `service.svelte.ts`: the app-wide worker, its status and `generateText`;
 * - `models.ts`: the model; `chat.ts`: its chat format; `generate.ts`: the generation loop
 *   (shared with `tools/llm/check.mjs`); `cpuGraph.ts`: its graph made runnable on the processor.
 */

export type { GenResult } from './generate';
export { downloadMb, MODEL } from './models';
export { ensureLlm, generateText, llmReady, llmStats, llmStatus, unloadLlm, type LlmStats } from './service.svelte';
