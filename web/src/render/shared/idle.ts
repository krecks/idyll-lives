/** Work done in the page's idle moments: models built ahead without holding up a frame. */

/** Idle time (ms) left below which `buildAhead` waits for the next idle moment. */
const IDLE_SLICE_MS = 6;

/**
 * Runs `jobs` in idle moments, as many as fit in each, until they are done or `current()` turns
 * false: models built ahead without holding up a frame.
 */
export function buildAhead(jobs: (() => void)[], current: () => boolean): void {
  const more = () => {
    if (jobs.length) idle(step);
  };
  const step = (deadline?: IdleDeadline) => {
    if (!current()) return;
    while (jobs.length) {
      jobs.shift()!();
      if (!deadline || deadline.timeRemaining() < IDLE_SLICE_MS) break;
    }
    more();
  };
  idle(step);
}

/** Runs `fn` when the page is idle (soon anyway; a short timeout where idle callbacks are missing). */
function idle(fn: (deadline?: IdleDeadline) => void, timeout = 2000): void {
  if ('requestIdleCallback' in window) requestIdleCallback(fn, { timeout });
  else setTimeout(fn, 50);
}
