/**
 * The sounds, made: in Rust (`crates/sound`), here through its WebAssembly (`crates/sound-wasm`, built into
 * `wasm-pkg/` by `bun run sound`). It loads once on the page, as the game boots (`main.ts`), and once in the sound
 * worker (`water.worker.ts`). Until it has, nothing can be made (`synthReady`): a blip or a step asked for then
 * is skipped, a loop made a moment later. Each sound is samples at a sample rate, the same every time.
 */

import init, * as wasm from './wasm-pkg/sound_wasm.js';
import type { Blip, Footwear, MonoKind, RoomSize, Stereo, Surface, Voice, WorkerKind } from './kinds';

let loading: Promise<void> | null = null;
let loaded = false;

/**
 * Loads the synthesis (once; again after a failed load): from `source`, the .wasm's bytes (the tools and the
 * tests, under Bun), or fetched from beside this module.
 */
export function loadSynth(source?: BufferSource): Promise<void> {
  loading ??= init({ module_or_path: source ?? new URL('./wasm-pkg/sound_wasm_bg.wasm', import.meta.url) }).then(
    () => {
      loaded = true;
    },
    (err: unknown) => {
      loading = null;
      throw err;
    },
  );
  return loading;
}

/** Whether sounds can be made now. */
export function synthReady(): boolean {
  return loaded;
}

function made<T>(what: string, value: T | undefined): T {
  if (value === undefined) throw new Error(`synth: no sound ${what}`);
  return value;
}

function both(s: wasm.Sound): Stereo | Float32Array {
  try {
    const left = s.takeLeft();
    const right = s.takeRight();
    return right ? { left, right } : left;
  } finally {
    s.free();
  }
}

/** A sound made by kind: the water's and the things' loops, the outdoors', the pieces of music (two channels, or one). */
export function makeSound(kind: WorkerKind, rate: number): Stereo | Float32Array {
  return both(made(kind, wasm.make(kind, rate)));
}

/** A sound of one channel (the sizzle, the TV through a wall, the knock at the door…), its peak 1. */
export function monoSamples(kind: MonoKind, rate: number): Float32Array {
  return made(kind, wasm.mono(kind, rate));
}

/** An interface blip. */
export function blipSamples(name: Blip, rate: number): Float32Array {
  return made(name, wasm.blip(name, rate));
}

/** One footstep on `surface` in `wear`, version `version` (0 to `STEP_VERSIONS` - 1), its peak 1. */
export function stepSamples(surface: Surface, wear: Footwear, version: number, rate: number): Float32Array {
  return made(`${surface} ${wear}`, wasm.step(surface, wear, version, rate));
}

/** One version of a bird's song (0 to `VERSIONS` - 1). */
export function songSamples(bird: Voice, version: number, rate: number): Float32Array {
  return made(bird.species, wasm.song(bird.species, bird.seed, bird.pitch, bird.pace, bird.near, version, rate));
}

/** A room's impulse response (two channels, its energy 1 a channel on average). */
export function roomResponse(size: RoomSize, rate: number): Stereo {
  return both(made(size, wasm.room(size, rate))) as Stereo;
}

/**
 * `x` through a low-pass filter as a Web Audio `BiquadFilterNode` computes it (its `Q` in dB). `loop`: `x` is a
 * loop, and the filter has run over its end before its start, so the result loops as smoothly as `x` did.
 */
export function lowpass(x: Float32Array, rate: number, frequency: number, q: number, loop: boolean): Float32Array {
  return wasm.lowpass(x, rate, frequency, q, loop);
}
