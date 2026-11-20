/**
 * The gameplay benchmark's window into the renderer (`tools/bench/gameplay.mjs`): `?gpustats` (dev
 * server only) has the game measure GPU time for the whole session and answers `window.__gameStats()`
 * with the renderer's own numbers, on demand, so a harness never has to wait for the 5 s dev play log
 * sample. Off by default, and absent from production builds: `import.meta.env.DEV` is false there, so
 * `GAMEPLAY_STATS` is `false` and the hook is never installed (and the flag is tree-shaken away). A
 * build made for the benchmark (`VITE_BENCH=1`, `tools/bench/lib/build.mjs`; `VITE_BENCH_STATS=1` keeps this hook alone) keeps it.
 */

import type { MemoryCensus, RenderStats } from '../render/types';
import { DEBUG_HOOKS } from './hooks';

const params = new URLSearchParams(location.search);

/** Whether the gameplay benchmark asked for the renderer's stats (`?gpustats`, dev only). */
export const GAMEPLAY_STATS = (DEBUG_HOOKS || import.meta.env.VITE_BENCH_STATS === '1') && params.has('gpustats');

/**
 * What `window.__gameStats()` answers: the renderer's stats, the GPU passes it timed, and the
 * simulation's WASM memory (the sim worker reports its own; the main thread can't read it).
 */
export type GameplayStats = RenderStats & {
  gpuPasses: { name: string; ms: number; frames: number }[];
  /** The sim worker's WASM linear memory, bytes; null until its first 1 Hz report. */
  simMemoryBytes: number | null;
};

declare global {
  interface Window {
    /** The renderer's stats on demand (`?gpustats`, dev only; `tools/bench/gameplay.mjs`). */
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
