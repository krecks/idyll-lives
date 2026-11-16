/**
 * Making a world and its town file off the main thread (`town.worker.ts`). Both are seeded and
 * pure, but a big world takes seconds (its layout, then a house for every lot) and assembling
 * its file most of a second: on the page that froze the screen. `runTownJob` is what the worker
 * runs (and the page itself, where workers can't start), so the two give the same result.
 */

import type { Content } from '../content/content';
import type { WorldOptions, WorldStructure } from '../core/protocol';
import { residentsOf, type HouseholdDraft } from './household';
import { encodeLand, type Land } from './land';
import { settleLand } from './landFollow';
import { assembleTown, generatedHouses, generateWorld, houseTemplate, loadTemplates, primeHouses, type AssembleOptions, type HouseTemplate, type NeighbourhoodDraft, type Templates } from './town';
import { previewWorld } from './townPreview';
import { landDraft } from './macro';
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
   * Step 2's town, laid on `land` (step 1's) from `seed` with `options` (over the land's own), its
   * land settled under it as a new world's is (the `world` job's); named `name`, else as the land is. `site`: where its main town goes
   * (else where the land's plan has it); `waterKept`: the land's water is as its plan has it. For
   * now it is the `world` job for the land's place and options (its shaped land, if it has one);
   * the site and the plan are not read yet.
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
  | { kind: 'assemble'; town: NeighbourhoodDraft; player: HouseholdDraft; slot: number; options: AssembleOptions; random: number }
  /**
   * The houses of `town` furnished for the households about to live in them (`slot`: each one's
   * home), ahead of the `assemble` job that would otherwise furnish them all at Move in. They are
   * kept where the job ran (the worker's houses, or the page's where no worker runs), so the
   * `assemble` after it finds them made. Sent by `furnishAhead`.
   */
  | { kind: 'furnish'; town: NeighbourhoodDraft; households: readonly { slot: number; household: HouseholdDraft }[] };

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
  | { kind: 'assemble'; lot: string }
  /** How many houses were furnished (`furnish`), and how long that took (ms). */
  | { kind: 'furnish'; houses: number; ms: number };

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
      const options = { ...job.land.world?.options, ...job.options };
      return { kind: 'town', ...laidOut(content, t, generateWorld(content, t, job.land.world?.kind ?? 'country', job.seed, job.name ?? job.land.name, step, options, job.land.open?.land), step) };
    }
    case 'edit':
      throw new Error('The edit job is not built yet.');
    case 'assemble':
      return { kind: 'assemble', lot: assembleTown(content, t, job.town, job.player, job.slot, job.options, mulberry32(job.random)) };
    case 'furnish':
      return { kind: 'furnish', ...furnish(content, t, job.town, job.households) };
  }
}

/** The houses of `town` furnished for `households` (as `layoutTown` furnishes them), kept with the houses made: how many, and in how long (ms). */
function furnish(content: Content, t: Templates, town: NeighbourhoodDraft, households: readonly { slot: number; household: HouseholdDraft }[]): { houses: number; ms: number } {
  const t0 = performance.now();
  let houses = 0;
  for (const { slot, household } of households) {
    const s = town.slots[slot];
    if (s?.kind === 'house' && houseTemplate(t, s.template, residentsOf(content, household))) houses++;
  }
  return { houses, ms: performance.now() - t0 };
}

/** A town just laid (`laid`) with its land settled under it (`settleLand`), its preview and its houses. */
function laidOut(content: Content, t: Templates, laid: NeighbourhoodDraft, step?: (done: WorldStep) => void): LaidTown {
  const settled = settleLand(laid, (d) => previewWorld(content, t, d, null));
  const draft = settled.draft;
  const preview = settled.world;
  if (draft.open?.land && draft !== laid) preview.meta = { ...preview.meta, land: encodeLand(draft.open.land) };
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
  const asked = towns.ask(job, () => onPage(content, job, step), { signal, onNote: step });
  // (The houses furnished ahead are of no more use once the town file is made.)
  if (job.kind === 'assemble') asked.then(letFurnishedGo, letFurnishedGo);
  return asked;
}

/**
 * How long the worker is kept for the houses furnished ahead, at most, when no `assemble` job comes
 * (ms): the player may stay in step 4 a while, and an idle worker goes after a minute with them.
 */
const FURNISHED_KEPT_MS = 15 * 60_000;
/** Letting go of the worker kept for the houses furnished ahead, while it is kept. */
let furnishedHold: (() => void) | null = null;

function letFurnishedGo(): void {
  furnishedHold?.();
  furnishedHold = null;
}

/**
 * Furnishes ahead, in the town worker, the houses of `town` for the households about to live in
 * them (`households`: their homes' slots), so Move in's `assemble` job need not (on the biggest
 * worlds that is most of its time). Meant for step 4, before the player has chosen a home: the households of both
 * waves, the first wave's (`town.households`) and the second's as `people.ts` `fillPlan` has them
 * with no home chosen yet (a few may come out differently once one is: those are furnished at Move
 * in). The worker is kept running until the next `assemble` job is done (or `FURNISHED_KEPT_MS`
 * passes), since a fresh one would have lost them: so leave the job running at Move in (the
 * `assemble` waits behind it), as aborting it stops the worker. Another call replaces this one's
 * hold; an aborted `signal` drops the job. Resolves to how many houses were furnished and in how long.
 */
export async function furnishAhead(
  content: Content,
  town: NeighbourhoodDraft,
  households: readonly { slot: number; household: HouseholdDraft }[],
  signal?: AbortSignal,
): Promise<{ houses: number; ms: number }> {
  letFurnishedGo();
  const release = towns.hold();
  const hold = () => {
    clearTimeout(timer);
    release();
  };
  const timer = setTimeout(() => furnishedHold === hold && letFurnishedGo(), FURNISHED_KEPT_MS);
  furnishedHold = hold;
  try {
    const { houses, ms } = await townJob(content, { kind: 'furnish', town, households }, signal);
    return { houses, ms };
  } catch (err) {
    // (Dropped or failed: nothing furnished worth keeping the worker for.)
    if (furnishedHold === hold) letFurnishedGo();
    throw err;
  }
}

/** A world's houses the worker made, kept on the page too (the menu and the screens after need them). */
export function keepHouses(result: LaidTown): void {
  primeHouses(result.houses);
}
