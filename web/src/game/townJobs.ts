/**
 * Making a world and its town file off the main thread (`town.worker.ts`). Both are seeded and
 * pure, but a big world takes seconds (its layout, then a house for every lot) and assembling
 * its file most of a second: on the page that froze the screen. `runTownJob` is what the worker
 * runs (and the page itself, where workers can't start), so the two give the same result.
 */

import type { Content } from '../content/content';
import type { WorldStructure, WorldSize } from '../core/protocol';
import type { HouseholdDraft } from './household';
import { assembleTown, generatedHouses, generateWorld, loadTemplates, primeHouses, type AssembleOptions, type HouseTemplate, type NeighbourhoodDraft, type Templates } from './town';
import { previewWorld } from './townPreview';
import type { WorldKind } from './world';
import { LazyWorker, type WorkerAnswer, type WorkerAsk } from '../core/lazyWorker';
import { mulberry32 } from './noise';

export type TownJob =
  /** A new world (`generateWorld`), with the menu's preview of it (`previewWorld`). */
  | { kind: 'world'; world: WorldKind; seed: number; name?: string; size: WorldSize }
  /** The simulation's town file (`assembleTown`); `random`: the seed for its dice (neighbours' bonds). */
  | { kind: 'assemble'; town: NeighbourhoodDraft; player: HouseholdDraft; slot: number; options: AssembleOptions; random: number };

export type TownResult =
  | { kind: 'world'; draft: NeighbourhoodDraft; preview: WorldStructure; houses: [string, HouseTemplate | undefined][] }
  | { kind: 'assemble'; lot: string };

export type ToTownWorker = WorkerAsk<TownJob>;
/** `broken`: the worker couldn't load the content (every job fails there). */
export type FromTownWorker = WorkerAnswer<TownResult>;

/**
 * Runs a job. `assembleTown` rolls some dice (who are friends, rivals): they come from the job's
 * own seed, so the worker and the page give the same file for the same job.
 */
export function runTownJob(content: Content, t: Templates, job: TownJob): TownResult {
  if (job.kind === 'world') {
    const draft = generateWorld(content, t, job.world, job.seed, job.name, job.size);
    const preview = previewWorld(content, t, draft, null);
    return { kind: 'world', draft, preview, houses: generatedHouses(t, draft.slots.map((s) => s.template)) };
  }
  return { kind: 'assemble', lot: assembleTown(content, t, job.town, job.player, job.slot, job.options, mulberry32(job.random)) };
}

/** Raised for a job a newer one replaced (or whose signal aborted it). */
export { Superseded } from '../core/lazyWorker';

/** The town worker (`town.worker.ts`); it goes once it has had nothing to do for a minute (its content and houses are freed). */
const towns = new LazyWorker<TownJob, TownResult>({
  start: () => new Worker(new URL('./town.worker.ts', import.meta.url), { type: 'module' }),
  idleMs: 60_000,
  broke: (why) => console.warn('[town] the town worker failed; making towns on the page:', why),
});

async function onPage(content: Content, job: TownJob): Promise<TownResult> {
  return runTownJob(content, await loadTemplates(), job);
}

/**
 * Runs `job` in the town worker (on the page if no worker can run; `content` is for that). An
 * aborted `signal` drops the job: one already running stops with its worker, and the jobs waiting
 * behind it go to a fresh one, so a player rolling world after world waits only for the last.
 */
export function townJob(content: Content, job: TownJob, signal?: AbortSignal): Promise<TownResult> {
  return towns.ask(job, () => onPage(content, job), { signal });
}

/** A world's houses the worker made, kept on the page too (the menu and the screens after need them). */
export function keepHouses(result: Extract<TownResult, { kind: 'world' }>): void {
  primeHouses(result.houses);
}
