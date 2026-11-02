/**
 * When the voices and the language model may work (both are ONNX Runtime in workers of their own,
 * each with its own WebGPU device), so they don't fight the game or each other:
 *
 * - their models start loading only once the page has booted (`startModelsAfter`), and not while
 *   the game holds them off;
 * - the game holds them off while it works hard (`holdInference`: a game loading, a focus switch
 *   rebuilding the scene): no model starts and no new line is made until it lets go;
 * - one job at a time on the graphics card across both (`inferenceSlot`): a line being spoken and
 *   one being written don't share the card with the game twice over. Jobs on the processor only
 *   wait for holds (the voice worker makes its lines one after another anyway).
 */

import { untrack } from 'svelte';
import { idleMoment } from '../render/shared/idle';

export type InferenceDevice = 'gpu' | 'cpu';

/** A hold the game forgot to let go of ends by itself after this long (a session that failed to start). */
const HOLD_MAX_MS = 120_000;

let nextHold = 1;
/** Holds not yet let go of, each with the timer that ends it if it's forgotten. */
const holds = new Map<number, ReturnType<typeof setTimeout>>();
/** When the last hold ended (performance clock): lines made across a hold aren't the engine's fault. */
let releasedAt = -Infinity;
/** The page has booted: models may start, holds permitting. */
let started = false;

interface Waiter {
  device: InferenceDevice | null;
  grant: () => void;
}
/** Jobs and model starts waiting, in the order they asked (`device` null: a model start). */
let waiting: Waiter[] = [];
/** A job is on the graphics card. */
let gpuBusy = false;

/** Lets waiting jobs and starts go, in order, as far as holds and the graphics card allow. */
function pump(): void {
  if (holds.size) return;
  const still: Waiter[] = [];
  for (const w of waiting) {
    if (w.device === null) {
      if (started) w.grant();
      else still.push(w);
    } else if (w.device === 'cpu') {
      w.grant();
    } else if (!gpuBusy) {
      gpuBusy = true;
      w.grant();
    } else {
      still.push(w);
    }
  }
  waiting = still;
}

/**
 * Holds inference and model starts off until the returned function is called (or `HOLD_MAX_MS`
 * passed: `reason` says in the console what forgot to let go).
 */
export function holdInference(reason: string): () => void {
  const id = nextHold++;
  const release = () => {
    if (!holds.has(id)) return;
    clearTimeout(holds.get(id));
    holds.delete(id);
    releasedAt = performance.now();
    pump();
  };
  const forgotten = () => {
    console.warn(`[ml] inference was held off (${reason}) and never let go: letting go`);
    release();
  };
  holds.set(id, setTimeout(forgotten, HOLD_MAX_MS));
  return release;
}

/** Holds inference off until `work` settles. */
export function holdInferenceWhile(reason: string, work: Promise<unknown>): void {
  const release = holdInference(reason);
  void work.finally(release).catch(() => {});
}

/** Whether something holds inference off now. */
export function inferenceHeld(): boolean {
  return holds.size > 0;
}

/** Whether inference was held off at any time since `since` (performance clock). */
export function heldSince(since: number): boolean {
  return holds.size > 0 || releasedAt >= since;
}

/**
 * Waits for a turn to run one job on `device`; call the returned function when it's done. Null
 * when `signal` aborted while waiting.
 */
export function inferenceSlot(device: InferenceDevice, signal?: AbortSignal): Promise<(() => void) | null> {
  return new Promise((resolve) => {
    if (signal?.aborted) return resolve(null);
    let done = false;
    const free = () => {
      if (done) return;
      done = true;
      if (device === 'gpu') {
        gpuBusy = false;
        pump();
      }
    };
    const w: Waiter = {
      device,
      grant: () => {
        signal?.removeEventListener('abort', abort);
        resolve(free);
      },
    };
    const abort = () => {
      waiting = waiting.filter((x) => x !== w);
      resolve(null);
    };
    signal?.addEventListener('abort', abort, { once: true });
    waiting.push(w);
    pump();
  });
}

/** A job's entry taken off `pending` (`id`'s), its turn on the device (`inferenceSlot`) given back. */
export function settle<P extends { free: () => void }>(pending: Map<number, P>, id: number): P | undefined {
  const p = pending.get(id);
  pending.delete(id);
  p?.free();
  return p;
}

/** Resolves once models may start loading: the page has booted and nothing holds them off. */
export function whenModelsMayStart(): Promise<void> {
  return new Promise((resolve) => {
    waiting.push({ device: null, grant: resolve });
    pump();
  });
}

/** Models may start (as soon as nothing holds them off): `startModelsAfter`'s, once the page has booted; idempotent. */
export function startModels(): void {
  if (started) return;
  started = true;
  pump();
}

/**
 * Starts the models in the first idle moment once `booting()` turns false: the voices and the
 * language model don't download and set up while the game boots and fetches its first town.
 */
export function startModelsAfter(booting: () => boolean): void {
  const stop = $effect.root(() => {
    $effect(() => {
      if (booting()) return;
      untrack(() => {
        queueMicrotask(stop);
        void idleMoment(3000).then(startModels);
      });
    });
  });
}
