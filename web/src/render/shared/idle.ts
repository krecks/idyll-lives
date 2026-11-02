/** Work done in the page's idle moments: models built ahead without holding up a frame. */

/** Idle time (ms) left below which `buildAhead` waits for the next idle moment. */
const IDLE_SLICE_MS = 6;
/** Where idle callbacks are missing: the wait (ms) standing in for an idle moment, and the time (ms) it is taken to have. */
const NO_IDLE_WAIT_MS = 50;
const NO_IDLE_SLICE_MS = 8;

/**
 * The page's next idle moment, with how much of it is left (`timeRemaining`, ms); `timeout`: the
 * longest wait for one (none: as long as it takes). Where idle callbacks are missing, a short wait
 * and a short slice instead.
 */
export function idleMoment(timeout?: number): Promise<{ timeRemaining(): number }> {
  return new Promise((resolve) => {
    if (typeof requestIdleCallback === 'function') {
      requestIdleCallback(resolve, { timeout });
      return;
    }
    setTimeout(() => {
      const end = performance.now() + NO_IDLE_SLICE_MS;
      resolve({ timeRemaining: () => Math.max(0, end - performance.now()) });
    }, NO_IDLE_WAIT_MS);
  });
}

/**
 * Runs `jobs` in idle moments, as many as fit in each, until they are done or `current()` turns
 * false: models built ahead without holding up a frame.
 */
export function buildAhead(jobs: (() => void)[], current: () => boolean): void {
  void (async () => {
    while (jobs.length) {
      const moment = await idleMoment(2000);
      if (!current()) return;
      do jobs.shift()!();
      while (jobs.length && moment.timeRemaining() >= IDLE_SLICE_MS);
    }
  })();
}
