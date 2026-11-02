/**
 * Makes worlds and town files off the main thread (`townJobs.ts`): loads the game's content and
 * house templates itself once, then runs the jobs it is sent in turn.
 */

import { Content } from '../content/content';
import { loadTemplates, useTownContent } from './town';
import { runTownJob, type FromTownWorker, type ToTownWorker } from './townJobs';

interface WorkerScope {
  onmessage: ((e: MessageEvent<ToTownWorker>) => void) | null;
  postMessage(message: FromTownWorker): void;
}
const scope = self as unknown as WorkerScope;

/** The content and templates, as the page loads them (`session.ts` `loadGameData`). */
const ready = Content.load(`${import.meta.env.BASE_URL}content/base.json`).then(async (content) => {
  useTownContent(content);
  return { content, templates: await loadTemplates() };
});

// Jobs wait for the content, then run in the order they came. Content that can't load breaks
// the worker (the page then makes its towns itself).
scope.onmessage = (e) => {
  const { id, job } = e.data;
  ready.then(
    ({ content, templates }) => {
      try {
        scope.postMessage({ id, result: runTownJob(content, templates, job) });
      } catch (err) {
        scope.postMessage({ id, error: err instanceof Error ? err.message : String(err) });
      }
    },
    (err: unknown) => scope.postMessage({ id, error: err instanceof Error ? err.message : String(err), broken: true }),
  );
};
