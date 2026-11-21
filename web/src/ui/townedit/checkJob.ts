/**
 * The Problems list's checks, run in their own worker (`check.worker.ts`): a city's take seconds
 * (`townCheck.ts`'s road and lot checks), too long for the page after every edit. Where no worker
 * can run, the page checks the town itself.
 */

import type { Content } from '../../content/content';
import { LazyWorker, type WorkerAnswer, type WorkerAsk } from '../../core/lazyWorker';
import { loadTemplates, type NeighbourhoodDraft } from '../../game/town';
import { townIssues, type TownIssues } from './problems';

/** A town to check (its plan draft), and the town as it was laid's problems (they block nothing). */
export interface CheckJob {
  draft: NeighbourhoodDraft;
  base?: TownIssues;
}
export type ToCheckWorker = WorkerAsk<CheckJob>;
export type FromCheckWorker = WorkerAnswer<TownIssues>;

/** The worker; it goes once it has had nothing to do for a minute. */
const checks = new LazyWorker<CheckJob, TownIssues>({
  start: () => new Worker(new URL('./check.worker.ts', import.meta.url), { type: 'module' }),
  idleMs: 60_000,
  broke: (why) => console.warn('[town] the checks worker failed; checking towns on the page:', why),
});

/** What is wrong with `draft` (`problems.ts` `townIssues`), checked in the worker (on the page if none can run). */
export function checkTown(content: Content, draft: NeighbourhoodDraft, base?: TownIssues): Promise<TownIssues> {
  return checks.ask({ draft, ...(base ? { base } : {}) }, async () => townIssues(draft, await loadTemplates(), content, base));
}
