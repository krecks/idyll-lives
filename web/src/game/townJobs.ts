/**
 * Making a world and its town file off the main thread (`town.worker.ts`). Both are seeded and
 * pure, but a big world takes seconds (its layout, then a house for every lot) and assembling
 * its file most of a second: on the page that froze the screen. `runTownJob` is what the worker
 * runs (and the page itself, where workers can't start), so the two give the same result.
 */

import type { Content } from '../content/content';
import type { WorldOptions, WorldStructure } from '../core/protocol';
import { residentsOf, type HouseholdDraft } from './household';
import type { Residents } from './housegen';
import { flatsTemplate } from './flats';
import { encodeLand, type Land } from './land';
import { settleEdit, settleLand } from './landFollow';
import { editedProblems } from './edit/checked';
import { assembleTown, generatedHouses, generateWorld, houseTemplate, loadTemplates, primeHouses, type AssembleOptions, type HouseTemplate, type NeighbourhoodDraft, type Templates } from './town';
import { previewWorld } from './townPreview';
import { landDraft, layOn } from './macro';
import type { Pt } from './roads';
import { proposalOf } from './proposal';
import { buildFromUses } from './build';
import type { WorldStep } from './world';
import { LazyWorker, Superseded, type WorkerAnswer, type WorkerAsk } from '../core/lazyWorker';
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
   * bridged as a new world's; else the town goes on a piece of dry land and, with villages, a village
   * on every other piece a bridge could reach, bridged to it). `macro.ts` `layOn`.
   */
  | { kind: 'town'; land: NeighbourhoodDraft; seed: number; name?: string; options: Partial<WorldOptions>; site?: Pt; waterKept: boolean }
  /**
   * Step 2's proposed layout, from what the `town` job is given: the generator's town laid and settled as that job
   * lays it (`built`), and the layout step 2 starts from (`plan`: its roads, areas, centre and uses, with no lots,
   * places or parking). The result's town (`LaidTown`) is what step 2 shows: the plan's streets settled.
   */
  | { kind: 'layout'; land: NeighbourhoodDraft; seed: number; name?: string; options: Partial<WorldOptions>; site?: Pt; waterKept: boolean }
  /**
   * Step 3's town built from step 2's layout (`plan`, as the player left it) with `seed` (the build's own, which
   * "Build again" changes): a layout whose hash is `proposal`'s builds the proposal's town as it is.
   */
  | { kind: 'build'; plan: NeighbourhoodDraft; proposal: Proposal; seed: number }
  /**
   * Step 2's edits made: the town's plan draft (`game/edit/`; its land the land as laid, not graded:
   * `LaidTown.plan`) settled on its land (`landFollow.ts` `settleEdit`), after `ops` edits. `steps`:
   * the plans the batch went through, the first the plan the town on screen was settled from, the
   * last `plan` (`ops + 1` of them): with them an edit that takes the land over a slope limit is
   * refused, and the plan before it settled instead (without, nothing is refused). `before`: the last
   * settle's `marks` (not given, the first step is settled too, to find them). `check`: whether the
   * settled town is checked (`problems`; yes unless false: half a second on a city, M3).
   */
  | { kind: 'edit'; plan: NeighbourhoodDraft; ops: number; steps?: readonly NeighbourhoodDraft[]; before?: readonly string[]; check?: boolean }
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
  | { kind: 'furnish'; town: NeighbourhoodDraft; households: readonly { slot: number; flat?: number; household: HouseholdDraft }[] };

/**
 * A town laid out: its draft, the menus' preview of it, and the houses made for its lots (by template
 * id). `plan`: a town laid on a land whose town follows its slopes (`town`, `world`): the draft with
 * its land as laid, not yet graded under the town (step 2's edits start from it; settling it again,
 * `edit`, gives `draft`); `marks`: what in `draft` is over a slope limit (the first `edit`'s `before`).
 */
export interface LaidTown {
  draft: NeighbourhoodDraft;
  preview: WorldStructure;
  houses: [string, HouseTemplate | undefined][];
  plan?: NeighbourhoodDraft;
  marks?: string[];
}

/**
 * Step 2's proposed layout (the `layout` job's): the layout it starts from (`plan`: the generator's roads, areas,
 * centre and uses, with no lots, places or parking), the town the generator built for it (`built`, with its
 * uses: what Build gives for the layout untouched) and the plan's `layoutHash` (`hash`). `builtPlan` and `builtMarks`:
 * the built town's own `LaidTown.plan` and `marks`, which step 3's edits start from.
 */
export interface Proposal {
  plan: NeighbourhoodDraft;
  built: NeighbourhoodDraft;
  hash: string;
  builtPlan?: NeighbourhoodDraft;
  builtMarks?: string[];
}

/**
 * Step 2's edits settled (`edit`): the town settled (`draft`, its `preview`; `houses`: those its lots
 * use that the first step's didn't, all of them without steps); the plan that was settled (`step`: its
 * index in the job's `steps`, the last unless an edit was refused); the edits refused (`op`: an edit's
 * index in the batch, from 0) with why, as a clause; what in it is over a slope limit (`marks`, the
 * next job's `before`); and what is wrong with it (`townCheck.ts`, `hard` blocks Next), if checked.
 */
export interface EditedTown extends Omit<LaidTown, 'plan' | 'marks'> {
  step: number;
  refused: { op: number; why: string }[];
  marks: string[];
  problems?: { hard: string[]; soft: string[] };
}

export type TownResult =
  | { kind: 'land'; draft: NeighbourhoodDraft; preview: WorldStructure }
  | ({ kind: 'town' } & LaidTown)
  | ({ kind: 'layout'; proposal: Proposal } & LaidTown)
  | ({ kind: 'build' } & LaidTown)
  | ({ kind: 'edit' } & EditedTown)
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
      const lay = { townSeed: job.seed, waterKept: job.waterKept, ...(job.site ? { site: job.site } : {}) };
      return { kind: 'town', ...laidOut(content, t, layOn(content, t, job.land, job.options, lay, job.name ?? job.land.name, step), step) };
    }
    case 'layout':
      return { kind: 'layout', ...proposed(content, t, job, step) };
    case 'build':
      return { kind: 'build', ...built(content, t, job) };
    case 'edit':
      return { kind: 'edit', ...edited(content, t, job) };
    case 'assemble':
      return { kind: 'assemble', lot: assembleTown(content, t, job.town, job.player, job.slot, job.options, mulberry32(job.random)) };
    case 'furnish':
      return { kind: 'furnish', ...furnish(content, t, job.town, job.households) };
  }
}

/** The houses of `town` furnished for `households` (as `layoutTown` furnishes them), kept with the houses made: how many, and in how long (ms). */
function furnish(content: Content, t: Templates, town: NeighbourhoodDraft, households: readonly { slot: number; flat?: number; household: HouseholdDraft }[]): { houses: number; ms: number } {
  const t0 = performance.now();
  let houses = 0;
  // (A building's flats furnished together, for every household in them, as `layoutTown` asks for them.)
  const flats = new Map<number, (Residents | undefined)[]>();
  for (const { slot, flat, household } of households) {
    const s = town.slots[slot];
    if (flat !== undefined) (flats.get(slot) ?? flats.set(slot, []).get(slot)!)[flat] = residentsOf(content, household);
    else if (s?.kind === 'house' && houseTemplate(t, s.template, residentsOf(content, household))) houses++;
  }
  for (const [slot, residents] of flats) {
    const s = town.slots[slot];
    const id = s?.kind === 'flats' ? s.template : s?.upper;
    if (id && flatsTemplate(t, id, Array.from(residents))) houses++;
  }
  return { houses, ms: performance.now() - t0 };
}

/** A town just laid (`laid`) with its land settled under it (`settleLand`), its preview, its houses and its plan (`LaidTown.plan`). */
function laidOut(content: Content, t: Templates, laid: NeighbourhoodDraft, step?: (done: WorldStep) => void): LaidTown {
  const settled = settleLand(laid, (d) => previewWorld(content, t, d, null), undefined, content.careerCategories);
  const draft = settled.draft;
  const preview = settled.world;
  if (draft.open?.land && draft !== laid) preview.meta = { ...preview.meta, land: encodeLand(draft.open.land) };
  step?.('grade');
  const houses = generatedHouses(t, draft.slots.map((s) => s.template));
  step?.('houses');
  // (The land as laid: what was given up gone from the town, but its land not graded.)
  const land = laid.open?.land;
  const plan = draft.open && land?.seeded === 'follow' && draft.open.land !== land ? { ...draft, open: { ...draft.open, land } } : undefined;
  return { draft, preview, houses, ...(plan ? { plan, marks: settled.marks } : {}) };
}

/**
 * Step 2's proposed layout (`layout`): the town laid and settled as the `town` job lays it, its proposal
 * (`proposal.ts` `proposalOf`: the town built and the layout step 2 starts from), and that layout's streets
 * settled on the land as an edit is (`settleEdit`, giving nothing up): what step 2 shows, with no lots and so
 * no houses (Build's job has the town's). Its `plan` is the layout, `marks` what in it is over a slope limit.
 */
function proposed(content: Content, t: Templates, job: Extract<TownJob, { kind: 'layout' }>, step?: (done: WorldStep) => void): { proposal: Proposal } & LaidTown {
  const lay = { townSeed: job.seed, waterKept: job.waterKept, ...(job.site ? { site: job.site } : {}) };
  const town = laidOut(content, t, layOn(content, t, job.land, job.options, lay, job.name ?? job.land.name, step), step);
  const proposal = proposalOf(town, t);
  const settled = settleEdit(proposal.plan, (d) => previewWorld(content, t, d, null));
  const { draft, world: preview } = settled;
  if (draft.open?.land && draft !== proposal.plan) preview.meta = { ...preview.meta, zones: draft.open.zones, land: encodeLand(draft.open.land) };
  return { proposal, draft, preview, houses: [], plan: proposal.plan, marks: settled.marks };
}

/**
 * Step 3's town built from a layout (`build`: `build.ts` `buildFromUses`): the proposal's town as it is for an
 * untouched layout and the proposal's seed, else built again from the layout's streets and uses; its preview,
 * houses, plan and marks.
 */
function built(content: Content, t: Templates, job: Extract<TownJob, { kind: 'build' }>): LaidTown {
  return buildFromUses(job.proposal, job.plan, t, content, job.seed);
}

/** Step 2's edits settled (`EditedTown`): `settleEdit`, the settled land coded into the preview as `laidOut` codes it, the new houses, the checks. */
function edited(content: Content, t: Templates, job: Extract<TownJob, { kind: 'edit' }>): EditedTown {
  const settled = settleEdit(job.plan, (d) => previewWorld(content, t, d, null), job.before, job.steps);
  const { draft, world: preview } = settled;
  if (draft.open?.land && draft !== job.plan) preview.meta = { ...preview.meta, zones: draft.open.zones, land: encodeLand(draft.open.land) };
  // (The page has the houses of the town it settled last: `keepHouses`.)
  const had = new Set(job.steps && job.steps.length > 1 ? job.steps[0].slots.map((s) => s.template) : []);
  const houses = generatedHouses(t, draft.slots.map((s) => s.template).filter((id) => !had.has(id)));
  return { draft, preview, houses, step: settled.step, refused: settled.refused, marks: settled.marks, ...(job.check === false ? {} : { problems: editedProblems(draft, t, content) }) };
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

/** The `edit` jobs asked (`editTown`): the newest's number, and the one the worker is on. */
let editsAsked = 0;
let editRunning: Promise<unknown> | null = null;

/**
 * Settles step 2's edits in the town worker (the `edit` job; on the page if no worker can run), a
 * newer ask replacing an older one: an ask waiting behind the one under way is dropped when a newer
 * one comes (rejected with `Superseded`), and so is the answer to one under way, which is let run to
 * its end (stopping it would stop the worker, and the houses it has made would go with it). An
 * aborted `signal` drops the ask the same way. Resolves to the newest ask's answer.
 */
export async function editTown(content: Content, job: Omit<Extract<TownJob, { kind: 'edit' }>, 'kind'>, signal?: AbortSignal): Promise<Extract<TownResult, { kind: 'edit' }>> {
  const asked = ++editsAsked;
  const dropped = () => asked !== editsAsked || !!signal?.aborted;
  const aborted = new Promise<never>((_, reject) => signal?.addEventListener('abort', () => reject(new Superseded()), { once: true }));
  aborted.catch(() => {});
  while (editRunning) {
    await Promise.race([editRunning.catch(() => {}), aborted]);
    if (dropped()) throw new Superseded();
  }
  if (dropped()) throw new Superseded();
  const run = townJob(content, { kind: 'edit', ...job });
  editRunning = run;
  try {
    const answer = await Promise.race([run, aborted]);
    if (dropped()) throw new Superseded();
    return answer;
  } finally {
    if (editRunning === run) editRunning = null;
  }
}

/**
 * Keeps the town worker running (not stopped for being idle) until the returned function is called:
 * for step 2's editor while it is open, since a fresh worker has none of the town's houses made and
 * its first settle makes them all again (a city's: about 1.7 s rather than 0.5, M3).
 */
export function holdTownWorker(): () => void {
  return towns.hold();
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
