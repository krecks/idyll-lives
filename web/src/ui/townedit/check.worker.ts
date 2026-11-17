/**
 * Step 2's Problems list, checked off the main thread (`checkJob.ts`): the content and the house
 * templates loaded once, then each town sent checked in turn (`problems.ts` `townIssues`). A big
 * town's checks take seconds; the page stays free to draw the edits meanwhile.
 */

import { Content } from '../../content/content';
import { loadTemplates, useTownContent } from '../../game/town';
import { townIssues } from './problems';
import type { CheckJob, FromCheckWorker, ToCheckWorker } from './checkJob';

interface WorkerScope {
  onmessage: ((e: MessageEvent<ToCheckWorker>) => void) | null;
  postMessage(message: FromCheckWorker): void;
}
const scope = self as unknown as WorkerScope;

const ready = Content.load(`${import.meta.env.BASE_URL}content/base.json`).then(async (content) => {
  useTownContent(content);
  return { content, templates: await loadTemplates() };
});

scope.onmessage = (e) => {
  const { id, message } = e.data;
  const job: CheckJob = message;
  ready.then(
    ({ content, templates }) => {
      try {
        scope.postMessage({ id, answer: townIssues(job.draft, templates, content, job.base) });
      } catch (err) {
        scope.postMessage({ id, error: err instanceof Error ? err.message : String(err) });
      }
    },
    (err: unknown) => scope.postMessage({ id, error: err instanceof Error ? err.message : String(err), broken: true }),
  );
};
