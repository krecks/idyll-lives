/**
 * The gameplay benchmark's window into the renderer (`tools/bench/gameplay.mjs`): `?gpustats` has the game
 * measure GPU time for the whole session and answers `window.__gameStats()` with the renderer's own numbers, on
 * demand, so a harness never has to wait for the 5 s dev play log sample. Off by default, and only where the debug
 * hooks are (`DEBUG_HOOKS`: the dev server and a build made for the benchmark, `tools/bench/lib/build.mjs`); a
 * release leaves the hook out.
 */

import type { MemoryCensus, RenderStats } from '../render/types';
import { DEBUG_HOOKS } from './hooks';

const params = new URLSearchParams(location.search);

/** Whether the gameplay benchmark asked for the renderer's stats (`?gpustats`, where the debug hooks are). */
export const GAMEPLAY_STATS = DEBUG_HOOKS && params.has('gpustats');

/**
 * What `window.__gameStats()` answers: the renderer's stats, the GPU passes it timed, and the
 * simulation's WASM memory (the sim worker reports its own; the main thread can't read it).
 */
type GameplayStats = RenderStats & {
  gpuPasses: { name: string; ms: number; frames: number }[];
  /** The sim worker's WASM linear memory, bytes; null until its first 1 Hz report. */
  simMemoryBytes: number | null;
};

declare global {
  interface Window {
    /** The renderer's stats on demand (`?gpustats`; `tools/bench/gameplay.mjs`). */
    __gameStats?: () => GameplayStats;
    /** Where the renderer's bytes are, on demand (`?gpustats`; `Renderer.memoryCensus`). */
    __memCensus?: () => MemoryCensus | null;
  }
}

/** Answers `window.__gameStats()` with `stats` while `?gpustats` is set; does nothing otherwise. */
export function installGameplayStats(stats: () => GameplayStats): void {
  if (GAMEPLAY_STATS) window.__gameStats = stats;
}

/** Answers `window.__memCensus()` with the renderer's census while `?gpustats` is set (in the menus too). */
export function installMemoryCensus(census: () => MemoryCensus | null): void {
  if (GAMEPLAY_STATS) window.__memCensus = census;
}
