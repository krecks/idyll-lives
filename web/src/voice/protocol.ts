/** Messages between the main thread and the voice worker (`tts.worker.ts`). */

import type { LineTiming } from './supertonic';
import type { VoiceParams } from './voices';

/** Where the voice model runs: the graphics card (WebGPU), or the processor (WASM) as Auto's fallback. */
export type VoiceDevice = 'cpu' | 'gpu';

export type ToVoiceWorker =
  /** `base`: the app's base URL, where `voice/` lives. */
  | { type: 'load'; base: string; language: string; device: VoiceDevice }
  | { type: 'speak'; id: number; text: string; voice: VoiceParams };

export type FromVoiceWorker =
  /** Download progress (bytes). */
  | { type: 'progress'; loaded: number; total: number }
  | { type: 'ready'; ms: number }
  /** 24 kHz mono and its mouth shapes (flat, `parseVisemes`); `ms`: how long the engine took. */
  | { type: 'audio'; id: number; samples: Float32Array; visemes: Float32Array; ms: number; timing: LineTiming }
  /**
   * A line that failed (`id`), or the engine couldn't load or the worker failed. `restart`: the
   * engine broke while making a line (a WebAssembly trap); a new worker can take over.
   */
  | { type: 'error'; id?: number; message: string; restart?: boolean };

export const SAMPLE_RATE = 24_000;
