/**
 * The dev play log (dev server only): what happens while the game is played, so it can be read
 * afterwards. Every page load is one log, `<repo>/dev-logs/<id>.jsonl` (one JSON object a line),
 * and `dev-logs/latest.txt` names the newest. Off in production builds and benchmarks (`?bench`),
 * and with `?devlog=0`.
 *
 * What it keeps:
 * - `start`: the build, the browser, the screen (size, device pixel ratio), the GPU, the settings;
 * - `log`: every console warning and error, uncaught error and rejected promise (`debug/log.ts`);
 * - `screen`, `settings`, `visibility`, `window`: the menu or the game shown, a setting changed (old and
 *   new), the tab hidden or shown, the window resized or moved to another screen;
 * - `session`: a game opened (what: new, a save) and closed;
 * - `sample` (every `SAMPLE_MS` while a game is open): frames (fps, frame time, its slowest, the main
 *   thread's work), the resolution drawn, draw calls, GPU time when measured, the simulation's steps,
 *   memory, long tasks, the game's day, time and speed;
 * - `resolution`: the size drawn changed (and from what);
 * - `slowFrame`: a gap between frames over `SLOW_FRAME_MS` (a stall), with the game's time.
 * - `perf`: every span `debug/perf.ts` times (start-up, the menus' town, portrait batches, a game's
 *   preparation and warm-up), with its start and length in ms since the page loaded.
 *
 * Lines go to the dev server every `FLUSH_MS` (`vite.config.ts`, POST `/__debug/devlog`), and once more
 * when the page is left.
 */

import type { RenderStats } from '../render/types';
import { watchLog } from './log';

/** How often the game's state is sampled while a game is open, and how often lines go to the dev server (ms). */
export const SAMPLE_MS = 5000;
const FLUSH_MS = 2000;
/** A flush this long or longer (characters) isn't sent `keepalive` (the browser's limit is 64 kB in flight). */
const KEEPALIVE_MAX = 32_000;
/** A gap between drawn frames this long is a stall worth a line (ms). */
export const SLOW_FRAME_MS = 100;
/** Stall lines kept per sample period at most (a long hitch shouldn't flood the log). */
const SLOW_FRAMES_PER_SAMPLE = 20;

const params = new URLSearchParams(location.search);
/** Whether this page keeps a dev play log. */
export const DEVLOG = import.meta.env.DEV && !params.has('bench') && params.get('devlog') !== '0';

const pad = (n: number) => String(n).padStart(2, '0');
const now = new Date();
/** This page's log: its start, to the second, and a few letters so two tabs don't share one. */
const ID = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}_${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}_${Math.random().toString(36).slice(2, 6)}`;

let queue: string[] = [];
let slowFrames = 0;

/** Adds a line: `kind` and its data, with the wall-clock time and ms since the page loaded. */
export function devlog(kind: string, data: Record<string, unknown> = {}): void {
  if (!DEVLOG) return;
  try {
    queue.push(JSON.stringify({ at: new Date().toISOString(), t: Math.round(performance.now()), kind, ...data }));
  } catch {
    queue.push(JSON.stringify({ at: new Date().toISOString(), t: Math.round(performance.now()), kind, unserialisable: true }));
  }
}

/** A gap of `ms` between two drawn frames (`SLOW_FRAME_MS` or more), with whatever helps place it. */
export function devlogSlowFrame(ms: number, data: Record<string, unknown>): void {
  if (!DEVLOG || slowFrames >= SLOW_FRAMES_PER_SAMPLE) return;
  slowFrames++;
  devlog('slowFrame', { ms: Math.round(ms), ...data });
}

let pausedUntil = 0;

/**
 * Whether the gap before this frame may hold a pause on purpose: a menu over the game or a hidden tab
 * (nothing is drawn then), opened or closed within the last second. Such gaps aren't stalls.
 */
export function devlogPaused(): boolean {
  return performance.now() < pausedUntil;
}

/** A menu opened or closed, or the tab hidden or shown: the next frame's gap isn't a stall (`devlogPaused`). */
export function devlogPause(): void {
  pausedUntil = performance.now() + 1000;
}

/** A new sample period: stalls count again. */
export function devlogNextSample(): void {
  slowFrames = 0;
}

function flush(leaving = false): void {
  if (!queue.length) return;
  const body = JSON.stringify({ id: ID, lines: queue });
  queue = [];
  if (leaving && navigator.sendBeacon) {
    navigator.sendBeacon('/__debug/devlog', new Blob([body], { type: 'application/json' }));
    return;
  }
  // (A request kept alive past the page may carry 64 kB at most, all of them together: a bigger one is
  // refused, and its lines were lost. A busy load's spans come to more than that in one flush.)
  void fetch('/__debug/devlog', { method: 'POST', body, headers: { 'content-type': 'application/json' }, keepalive: body.length < KEEPALIVE_MAX }).catch(() => {});
}

/** The screen the page is on: CSS size, device pixel ratio, device pixels. */
export function screenInfo(): Record<string, unknown> {
  const dpr = window.devicePixelRatio || 1;
  return {
    window: [window.innerWidth, window.innerHeight],
    dpr,
    devicePixels: [Math.round(window.innerWidth * dpr), Math.round(window.innerHeight * dpr)],
    screen: [screen.width, screen.height],
  };
}

let started = false;

/** Starts the log (once): the page's set-up, the console's warnings and errors, and the page's comings and goings. */
export function startDevLog(context: Record<string, unknown> = {}): void {
  if (!DEVLOG || started) return;
  started = true;
  devlog('start', {
    id: ID,
    url: location.href,
    userAgent: navigator.userAgent,
    cores: navigator.hardwareConcurrency,
    isolated: crossOriginIsolated,
    webgpu: 'gpu' in navigator,
    ...screenInfo(),
    ...context,
  });
  watchLog((e) => devlog('log', { level: e.level, text: e.text }));
  document.addEventListener('visibilitychange', () => {
    devlogPause();
    devlog('visibility', { hidden: document.hidden });
  });
  let resizeTimer = 0;
  window.addEventListener('resize', () => {
    clearTimeout(resizeTimer);
    resizeTimer = window.setTimeout(() => devlog('window', screenInfo()), 300);
  });
  setInterval(() => flush(), FLUSH_MS);
  window.addEventListener('pagehide', () => {
    devlog('leave');
    flush(true);
  });
}

/** What changed between two plain objects, one level deep: `{ key: [before, after] }`. */
export function changes(before: Record<string, unknown>, after: Record<string, unknown>): Record<string, [unknown, unknown]> {
  const out: Record<string, [unknown, unknown]> = {};
  for (const key of new Set([...Object.keys(before), ...Object.keys(after)])) {
    const a = JSON.stringify(before[key]);
    const b = JSON.stringify(after[key]);
    if (a !== b) out[key] = [before[key], after[key]];
  }
  return out;
}

let drawnSize = '';

/**
 * The renderer's part of a sample (`RenderStats`, rounded), and a `resolution` line whenever the size
 * drawn changed since the last sample (`setting`: the resolution setting it was drawn for).
 */
export function renderSample(s: RenderStats, setting: unknown): Record<string, unknown> {
  const size = `${s.renderWidth}x${s.renderHeight}`;
  if (size !== drawnSize) devlog('resolution', { from: drawnSize || null, to: size, scale: +s.renderScale.toFixed(3), setting, ...screenInfo() });
  drawnSize = size;
  return {
    backend: s.backend,
    preset: s.preset,
    fps: +s.fps.toFixed(1),
    frameMs: +s.frameMs.toFixed(2),
    cpuMs: +s.cpuMs.toFixed(2),
    gpuMs: s.gpuMs === null ? null : +s.gpuMs.toFixed(2),
    residentsMs: +s.residentsMs.toFixed(2),
    drawCalls: s.drawCalls,
    held: s.holdStillFrames,
    size,
    scale: +s.renderScale.toFixed(3),
    refreshHz: Math.round(s.refreshHz),
    divisor: s.refreshDivisor,
  };
}
