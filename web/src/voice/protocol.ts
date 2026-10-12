/** Messages between the main thread and the voice worker (`tts.worker.ts`). */

import type { Device } from '../ml/ort';
import type { VoiceParams } from './voices';

export type ToVoiceWorker =
  /** `base`: the app's base URL, where `voice/` lives; `device`: the CPU unless the device test asks for the GPU; `threads`: the runtime's. */
  | { type: 'load'; base: string; language: string; device?: Device; threads?: number }
  /** A player's recording (any rate, decoded on the main thread), as clip `rec:<sha>`. */
  | { type: 'clip'; sha: string; samples: Float32Array; rate: number }
  /** Get a voice ready ahead of its first line (shape, encode, condition); no reply. */
  | { type: 'prepare'; voice: VoiceParams }
  | { type: 'speak'; id: number; text: string; voice: VoiceParams };

export type FromVoiceWorker =
  /** Download progress (bytes). */
  | { type: 'progress'; loaded: number; total: number }
  | { type: 'ready'; ms: number }
  /**
   * 24 kHz mono and its mouth shapes (flat, `parseVisemes`); `ms`: how long the engine took,
   * `prepMs` of it for getting the voice ready (0 if it was).
   */
  | { type: 'audio'; id: number; samples: Float32Array; visemes: Float32Array; ms: number; prepMs: number }
  /** A line that failed (`id`), or the engine couldn't load or the worker failed. */
  | { type: 'error'; id?: number; message: string };

export const SAMPLE_RATE = 24_000;
