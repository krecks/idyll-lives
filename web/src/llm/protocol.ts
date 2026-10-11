/** Messages between the main thread and the language model worker (`llm.worker.ts`). */

import type { ChatMessage } from './chat';
import type { GenParams, GenResult } from './generate';
import type { Device } from '../ml/ort';
import type { ModelSpec } from './models';

export type ToLlmWorker =
  /** `base`: the app's base URL (a model's `url` may be under it); `model`: what to load; `keep`: other models' files to keep in the cache. */
  | { type: 'load'; base: string; model: ModelSpec; device: Device; keep: string[] }
  /**
   * `reply`: the reply's first words, written for the model, and `lead` the last message's
   * start, kept as its own part (`formatChat`); `deadline`:
   * `Date.now()` time after which the reply is no longer wanted (dropped unstarted, or cut short).
   */
  | { type: 'generate'; id: number; messages: ChatMessage[]; reply: string; lead: string; params: GenParams; deadline: number }
  | { type: 'abort'; id: number };

export type FromLlmWorker =
  /** Download progress (bytes). */
  | { type: 'progress'; loaded: number; total: number }
  /** `threads`: the runtime's threads (on the CPU; 1 on the GPU). */
  | { type: 'ready'; ms: number; device: Device; threads: number }
  | { type: 'done'; id: number; result: GenResult }
  /** A reply not written: its deadline passed before it started, or it was aborted. */
  | { type: 'skipped'; id: number }
  /** A reply that failed (`id`), or the model couldn't load or the worker failed. */
  | { type: 'error'; id?: number; message: string };
