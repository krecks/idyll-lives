/** Messages between the main thread and the language model worker (`llm.worker.ts`). */

import type { ChatMessage } from './chat';
import type { GenParams, GenResult } from './generate';
import type { OrtDevice } from '../ml/ort';
import type { ModelSpec } from './models';

export type ToLlmWorker =
  /** `base`: the app's base URL (the model's `url` is under it); `model`: what to load; `device`: where it runs. */
  | { type: 'load'; base: string; model: ModelSpec; device: OrtDevice }
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
  | { type: 'ready'; ms: number }
  | { type: 'done'; id: number; result: GenResult }
  /** A reply not written: its deadline passed before it started, or it was aborted. */
  | { type: 'skipped'; id: number }
  /** A reply that failed (`id`), or the model couldn't load or the worker failed. `restart`: the graphics device was lost; a new worker can take over. */
  | { type: 'error'; id?: number; message: string; restart?: boolean };
