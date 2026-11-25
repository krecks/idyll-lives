/** Messages between the main thread and the voice worker (`tts.worker.ts`). */

import type { OrtDevice } from '../ml/ort';
import type { LineTiming } from './supertonic';
import type { VoiceParams } from './voices';

/** Where the voice model runs: the graphics card (WebGPU), or the processor (WASM) as Auto's fallback. */
export type VoiceDevice = OrtDevice;

export type ToVoiceWorker =
  /** `base`: the app's base URL, where `voice/` lives. */
  /** `measure`: count the model's GPU buffers (the benchmark, `bench.ts`; the game doesn't). */
  | { type: 'load'; base: string; language: string; device: VoiceDevice; measure?: boolean }
  | { type: 'speak'; id: number; text: string; voice: VoiceParams }
  /** The GPU buffers counted (`measure`): answered with `memory`. */
  | { type: 'memory' };

export type FromVoiceWorker =
  /** Download progress (bytes). */
  | { type: 'progress'; loaded: number; total: number }
  | { type: 'ready'; ms: number }
  /** 24 kHz mono and its mouth shapes (flat, `parseVisemes`); `ms`: how long the engine took. */
  | { type: 'audio'; id: number; samples: Float32Array; visemes: Float32Array; ms: number; timing: LineTiming }
  /**
   * A line that failed (`id`), or the engine couldn't load or the worker failed. `restart`: the
   * engine broke while making a line (a WebAssembly trap) or the graphics device was lost; a new worker can take over.
   */
  | { type: 'error'; id?: number; message: string; restart?: boolean }
  /** Bytes in GPU buffers now and at their peak (`measure`; 0 without it). */
  | { type: 'memory'; gpuLive: number; gpuPeak: number };

export const SAMPLE_RATE = 24_000;
