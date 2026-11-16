/**
 * Making a world and its town file off the main thread (`town.worker.ts`). Both are seeded and
 * pure, but a big world takes seconds (its layout, then a house for every lot) and assembling
 * its file most of a second: on the page that froze the screen. `runTownJob` is what the worker
 * runs (and the page itself, where workers can't start), so the two give the same result.
 */

import type { Content } from '../content/content';
import type { WorldOptions, WorldStructure } from '../core/protocol';
import type { HouseholdDraft } from './household';
import { encodeLand, type Land } from './land';
import { settleLand } from './landFollow';
import { assembleTown, generatedHouses, generateWorld, loadTemplates, primeHouses, type AssembleOptions, type HouseTemplate, type NeighbourhoodDraft, type Templates } from './town';
import { previewWorld } from './townPreview';
import { landDraft, layOn } from './macro';
import type { Pt } from './roads';
import type { WorldStep } from './world';
import { LazyWorker, type WorkerAnswer, type WorkerAsk } from '../core/lazyWorker';
import { mulberry32 } from './noise';

export type TownJob =
  /**
   * Step 1's land: place `world` with `options` (the place's own where not given) from `seed`, with
   * no town on it (`macro.ts` `landDraft`), and its preview.
   */
  | { kind: 'land'; world: string; seed: number; options?: Partial<WorldOptions> }
  /**
   * Step 2's town, laid on `land` (step 1's, with its plan) from `seed` (the town's own: the land's
   * by default) with `options` over the land's own, its land settled under it as a new world's is
   * (the `world` job's); named `name`, else as the land is. `site`: the middle of its main town
   * (else the plan's site); `waterKept`: the land's water is as its plan has it (its river is
   * bridged as a new world's; else the town goes on the largest piece of dry land and no bridge is
   * laid). `macro.ts` `layOn`.
   */
  | { kind: 'town'; land: NeighbourhoodDraft; seed: number; name?: string; options: Partial<WorldOptions>; site?: Pt; waterKept: boolean }
  /**
   * Step 2's edits made: the town's plan draft (`game/edit/`) settled on its land
   * (`landFollow.ts` `settleEdit`), after `ops` edits. Not built yet: it fails.
   */
  | { kind: 'edit'; plan: NeighbourhoodDraft; ops: number }
  /**
   * A new world (`generateWorld`), with the menu's preview of it (`previewWorld`) and, a place's,
   * its land graded under its town (`landFollow.ts` `settleLand`, from the preview's lots and paths,
   * giving up what still breaks the slope limits):
   * `world` is its place (`WorldKind`, or a kind from before places: `LEGACY_KINDS`), `options` the
   * player's (the place's own where not given). `land`: the town laid again on a land the player
   * shaped (step 1's "Lay the town again", or Town or Villages changed on a shaped land); it
   * crosses to the worker copied (its typed arrays cloned), so the page keeps its own.
   */
  | { kind: 'world'; world: string; seed: number; name?: string; options?: Partial<WorldOptions>; land?: Land }
  /** The simulation's town file (`assembleTown`); `random`: the seed for its dice (neighbours' bonds). */
  | { kind: 'assemble'; town: NeighbourhoodDraft; player: HouseholdDraft; slot: number; options: AssembleOptions; random: number };

/** A town laid out: its draft, the menus' preview of it, and the houses made for its lots (by template id). */
export interface LaidTown {
  draft: NeighbourhoodDraft;
  preview: WorldStructure;
  houses: [string, HouseTemplate | undefined][];
}

export type TownResult =
  | { kind: 'land'; draft: NeighbourhoodDraft; preview: WorldStructure }
  | ({ kind: 'town' } & LaidTown)
  /** The settled town, what is wrong with it (`townCheck.ts`: `hard` blocks Next), and the edits refused (by their index among the `ops`) with why. */
  | ({ kind: 'edit'; problems: { hard: string[]; soft: string[] }; refused: { op: number; why: string }[] } & LaidTown)
  | ({ kind: 'world' } & LaidTown)
  | { kind: 'assemble'; lot: string };

export type ToTownWorker = WorkerAsk<TownJob>;
/** `broken`: the worker couldn't load the content (every job fails there). A note: a world's step done (`WorldStep`). */
export type FromTownWorker = WorkerAnswer<TownResult, WorldStep>;

/**
 * Runs a job. `assembleTown` rolls some dice (who are friends, rivals): they come from the job's
 * own seed, so the worker and the page give the same file for the same job. `step`: a world's
 * parts as they are done (the last, `houses`, once its houses and preview are made).
 */
export function runTownJob(content: Content, t: Templates, job: TownJob, step?: (done: WorldStep) => void): TownResult {
  switch (job.kind) {
    case 'world':
      return { kind: 'world', ...laidOut(content, t, generateWorld(content, t, job.world, job.seed, job.name, step, job.options, job.land), step) };
    case 'land': {
      const draft = landDraft(content, t, job.world, job.options, job.seed);
      return { kind: 'land', draft, preview: previewWorld(content, t, draft, null) };
    }
    case 'town': {
      const lay = { townSeed: job.seed, waterKept: job.waterKept, ...(job.site ? { site: job.site } : {}) };
      return { kind: 'town', ...laidOut(content, t, layOn(content, t, job.land, job.options, lay, job.name ?? job.land.name, step), step) };
    }
    case 'edit':
      throw new Error('The edit job is not built yet.');
    case 'assemble':
      return { kind: 'assemble', lot: assembleTown(content, t, job.town, job.player, job.slot, job.options, mulberry32(job.random)) };
  }
}

/** A town just laid (`laid`) with its land settled under it (`settleLand`), its preview and its houses. */
function laidOut(content: Content, t: Templates, laid: NeighbourhoodDraft, step?: (done: WorldStep) => void): LaidTown {
  const settled = settleLand(laid, (d) => previewWorld(content, t, d, null));
  const draft = settled.draft;
  const preview = settled.world;
  if (draft.open?.land && draft !== laid) preview.meta = { ...preview.meta, land: encodeLand(draft.open.land) };
  step?.('grade');
  const houses = generatedHouses(t, draft.slots.map((s) => s.template));
  step?.('houses');
  return { draft, preview, houses };
}

/** Raised for a job a newer one replaced (or whose signal aborted it). */
export { Superseded } from '../core/lazyWorker';

/** The town worker (`town.worker.ts`); it goes once it has had nothing to do for a minute (its content and houses are freed). */
const towns = new LazyWorker<TownJob, TownResult, WorldStep>({
  start: () => new Worker(new URL('./town.worker.ts', import.meta.url), { type: 'module' }),
  idleMs: 60_000,
  broke: (why) => console.warn('[town] the town worker failed; making towns on the page:', why),
});

async function onPage(content: Content, job: TownJob, step?: (done: WorldStep) => void): Promise<TownResult> {
  return runTownJob(content, await loadTemplates(), job, step);
}

/**
 * Runs `job` in the town worker (on the page if no worker can run; `content` is for that). An
 * aborted `signal` drops the job: one already running stops with its worker, and the jobs waiting
 * behind it go to a fresh one, so a player rolling world after world waits only for the last.
 * `step`: a world's parts as they are done (in the worker or on the page).
 */
export function townJob<J extends TownJob>(content: Content, job: J, signal?: AbortSignal, step?: (done: WorldStep) => void): Promise<Extract<TownResult, { kind: J['kind'] }>>;
export function townJob(content: Content, job: TownJob, signal?: AbortSignal, step?: (done: WorldStep) => void): Promise<TownResult> {
  return towns.ask(job, () => onPage(content, job, step), { signal, onNote: step });
}

/** A world's houses the worker made, kept on the page too (the menu and the screens after need them). */
export function keepHouses(result: LaidTown): void {
  primeHouses(result.houses);
}
