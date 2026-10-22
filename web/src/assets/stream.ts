/**
 * The order in which assets download. Loads someone is waiting for (the viewed lot, the model in
 * hand, catalog pictures) run at once; files fetched ahead of time (`prefetch`: the rest of the
 * town; `background`: a model file that failed, tried again) wait until none of those is running
 * and go a couple at a time, so they never hold up what's on screen.
 *
 * A plan (`planStream`) says what this session will eventually fetch, so the HUD can show how much
 * of the game's data is already streamed and cached. `prefetchNow` is the same download in the
 * foreground (the loading screens wait for it); `prefetch` is the background stream after Play.
 */

import { withBuild, withoutBuild } from './build';

/** Background downloads at once. */
const BACKGROUND_PARALLEL = 2;
/** Foreground downloads at once (a loading screen's plan). */
const FOREGROUND_PARALLEL = 4;

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

/** A URL without this build's query: one key per file across releases. */
function key(url: string): string {
  return withoutBuild(url);
}

const requested = new Set<string>();
const finished = new Set<string>();
const inflight = new Map<string, Promise<void>>();

/** Downloads `url` once per page (shared by `prefetch` and `prefetchNow`). */
function load(url: string, priority: 'high' | 'low'): Promise<void> {
  const existing = inflight.get(url);
  if (existing) return existing;
  const p = (async () => {
    const res = await fetch(withBuild(url), { priority });
    if (!res.ok) throw new Error(`${url}: ${res.status}`);
    await res.arrayBuffer();
    finished.add(url);
  })();
  inflight.set(url, p);
  const forget = () => {
    if (inflight.get(url) === p) inflight.delete(url);
  };
  void p.then(forget, forget);
  return p;
}

/* The plan: the files this session will fetch, and how many are done. */

const planned = new Set<string>();
const settled = new Set<string>();
let done = 0;
let planLabel = '';

export interface StreamSnapshot {
  /** Files in the plan. */
  total: number;
  /** Files fetched (or failed, so the indicator does not hang). */
  done: number;
  label: string;
  active: boolean;
}

const listeners = new Set<() => void>();

/** Called whenever the plan or its progress changes. */
export function onStreamChange(fn: () => void): () => void {
  listeners.add(fn);
  return () => listeners.delete(fn);
}

export function streamSnapshot(): StreamSnapshot {
  return { total: planned.size, done, label: planLabel, active: planned.size > 0 && done < planned.size };
}

function notify(): void {
  for (const fn of listeners) fn();
}

function settle(url: string, ok: boolean): void {
  if (settled.has(url)) return;
  settled.add(url);
  if (ok) finished.add(url);
  if (planned.has(url)) done++;
  notify();
}

/**
 * Sets what this session will fetch: the files the house needs now, then the rest of the town.
 * Files already cached (from the menu, a previous session) start out done.
 */
export function planStream(urls: Iterable<string>, label: string): void {
  planned.clear();
  settled.clear();
  done = 0;
  planLabel = label;
  for (const url of urls) {
    const k = key(url);
    planned.add(k);
    if (finished.has(k)) {
      settled.add(k);
      done++;
    }
  }
  notify();
}

/** Resolves once the plan has nothing left to fetch. */
export function whenStreamSettled(): Promise<void> {
  if (!streamSnapshot().active) return Promise.resolve();
  return new Promise((resolve) => {
    const off = onStreamChange(() => {
      if (!streamSnapshot().active) {
        off();
        resolve();
      }
    });
  });
}

/**
 * Downloads files in the background so they come from the browser's cache when they're needed
 * (each URL once per page, in the order asked).
 */
export function prefetch(urls: Iterable<string>): void {
  for (const url of urls) {
    const k = key(url);
    if (requested.has(k) || finished.has(k)) continue;
    requested.add(k);
    void background(() => load(k, 'low'))
      .then(() => settle(k, true))
      .catch(() => {
        requested.delete(k);
        settle(k, false);
      });
  }
}

/**
 * Downloads `urls` now, a few at a time, pausing the background stream. Used by a loading screen
 * (the menu's assets, then the viewed house) so nothing it needs is left to stream later.
 */
export async function prefetchNow(urls: Iterable<string>, onProgress?: (done: number, total: number) => void): Promise<void> {
  const list = [...new Set([...urls].map(key))];
  const total = list.length;
  let completed = 0;
  const report = () => onProgress?.(completed, total);
  report();
  let at = 0;
  const worker = async () => {
    while (at < list.length) {
      const url = list[at++];
      if (finished.has(url)) {
        settle(url, true);
      } else {
        requested.add(url);
        try {
          // A background load may already be queued for this file. Wait for it without counting
          // as a foreground load: the queue only starts when `foregroundLoads` is zero, so
          // wrapping a queued background load would wait on itself.
          await (inflight.has(url) ? load(url, 'high') : foreground(() => load(url, 'high')));
          settle(url, true);
        } catch {
          requested.delete(url);
          settle(url, false);
        }
      }
      completed++;
      report();
    }
  };
  await Promise.all(Array.from({ length: Math.min(FOREGROUND_PARALLEL, list.length) }, worker));
}
