/**
 * The order in which assets download. Loads someone is waiting for (the viewed lot, the model in
 * hand, catalog pictures) run at once; files fetched ahead of time (`prefetch`: the other lots'
 * furniture; `background`: a model file that failed, tried again) wait until none of those is
 * running and go a couple at a time, so they never hold up what's on screen.
 */

import { withBuild } from './build';

/** Background downloads at once. */
const BACKGROUND_PARALLEL = 2;

let foregroundLoads = 0;
let backgroundLoads = 0;
const waiting: (() => void)[] = [];

/** Runs a load someone is waiting for; background downloads hold back while it runs. */
export async function foreground<T>(load: () => Promise<T>): Promise<T> {
  foregroundLoads++;
  try {
    return await load();
  } finally {
    foregroundLoads--;
    next();
  }
}

function next(): void {
  while (foregroundLoads === 0 && backgroundLoads < BACKGROUND_PARALLEL && waiting.length) {
    backgroundLoads++;
    waiting.shift()!();
  }
}

/**
 * Runs a load nobody is waiting for (a file fetched ahead of time, one that failed tried again): it
 * waits until none of the loads someone is waiting for is running, a couple at a time.
 */
export function background<T>(load: () => Promise<T>): Promise<T> {
  return new Promise<void>((resolve) => {
    waiting.push(resolve);
    next();
  })
    .then(load)
    .finally(() => {
      backgroundLoads--;
      next();
    });
}

const requested = new Set<string>();

/**
 * Downloads files in the background so they come from the browser's cache when they're needed
 * (each URL once per page, in the order asked).
 */
export function prefetch(urls: Iterable<string>): void {
  for (const url of urls) {
    if (requested.has(url)) continue;
    requested.add(url);
    void background(async () => {
      const res = await fetch(withBuild(url), { priority: 'low' });
      await res.arrayBuffer();
    }).catch(() => requested.delete(url));
  }
}
