/**
 * Where the time goes: named spans through start-up, menus and session preparation, the browser's
 * long tasks, and the frames it missed. Kept always (a few hundred numbers a session, no work per
 * frame beyond one comparison) and printed to the console with `?profile`.
 *
 * `?profile` also logs each span as it ends and a table of everything over 20 ms, so a slow start
 * says which step it was. `window.__perfReport()` returns the same data for a test harness.
 */

export interface PerfEntry {
  name: string;
  /** When it started, ms since the page loaded. */
  start: number;
  /** How long it took, ms. */
  ms: number;
  /** Extra context (what was loaded, how much of it). */
  detail?: string;
}

export interface LongTask {
  start: number;
  ms: number;
  /** The span that was running (or had just ended), if one was. */
  phase: string;
}

export interface DroppedFrame {
  /** When the long frame began, ms since the page loaded. */
  at: number;
  /** Time since the previous frame, ms. */
  gap: number;
  phase: string;
}

export interface PerfReport {
  entries: PerfEntry[];
  longTasks: LongTask[];
  droppedFrames: DroppedFrame[];
  /** Spans over 20 ms, longest first. */
  slowest: PerfEntry[];
}

const params = new URLSearchParams(location.search);
const LOG = params.has('profile');
const MAX_ENTRIES = 500;
const MAX_TASKS = 200;
const MAX_DROPS = 200;
/**
 * A frame longer than this is a hitch worth naming. 20 ms: a frame that takes a fifth of a 60 Hz
 * frame's time (or half a 120 Hz one) is already felt as a stutter, and at 50 ms most of them went
 * unrecorded.
 */
const DROP_MS = 20;

const entries: PerfEntry[] = [];
const longTasks: LongTask[] = [];
const droppedFrames: DroppedFrame[] = [];
let phase = '';

/** Names the work about to happen: spans and hitches are reported under it. */
export function perfPhase(name: string): void {
  phase = name;
}

/** A span: `const t = perfStart('name'); … perfEnd(t)`. */
export function perfStart(name: string, detail?: string): PerfEntry {
  return { name, start: performance.now(), ms: 0, detail };
}

export function perfEnd(entry: PerfEntry, detail?: string): number {
  entry.ms = performance.now() - entry.start;
  if (detail !== undefined) entry.detail = detail;
  record(entry);
  return entry.ms;
}

/** Times `fn` (sync) and records it. */
export function perfTime<T>(name: string, fn: () => T, detail?: string): T {
  const entry = perfStart(name, detail);
  try {
    return fn();
  } finally {
    perfEnd(entry);
  }
}

/** Times `fn` (async) and records it. */
export async function perfTimeAsync<T>(name: string, fn: () => Promise<T>, detail?: string): Promise<T> {
  const entry = perfStart(name, detail);
  try {
    return await fn();
  } finally {
    perfEnd(entry);
  }
}

function record(entry: PerfEntry): void {
  entries.push(entry);
  if (entries.length > MAX_ENTRIES) entries.shift();
  if (LOG && entry.ms >= 10) {
    const detail = entry.detail ? ` (${entry.detail})` : '';
    console.info(`[perf] ${entry.name}: ${entry.ms.toFixed(1)} ms${detail}`);
  }
}

/** The spans, long tasks and dropped frames so far. */
export function perfReport(): PerfReport {
  return {
    entries: entries.slice(),
    longTasks: longTasks.slice(),
    droppedFrames: droppedFrames.slice(),
    slowest: [...entries].filter((e) => e.ms >= 20).sort((a, b) => b.ms - a.ms),
  };
}

/** Logs the slowest spans and hitches as a table (the `?profile` summary). */
export function perfSummary(): void {
  const report = perfReport();
  const rows = report.slowest.map((e) => ({ step: e.name, ms: Math.round(e.ms), at: Math.round(e.start), detail: e.detail ?? '' }));
  if (rows.length) console.table(rows);
  const tasks = report.longTasks.filter((t) => t.ms >= 50);
  if (tasks.length) {
    console.info(
      '[perf] long tasks:',
      tasks.map((t) => `${Math.round(t.ms)} ms at ${Math.round(t.start)}${t.phase ? ` (${t.phase})` : ''}`).join(', '),
    );
  }
  const drops = report.droppedFrames;
  if (drops.length) {
    const worst = [...drops].sort((a, b) => b.gap - a.gap).slice(0, 10);
    console.info('[perf] worst frames:', worst.map((d) => `${Math.round(d.gap)} ms at ${Math.round(d.at)} (${d.phase})`).join(', '));
  }
  console.info('[perf] use __perfReport() for the raw data');
}

if (typeof PerformanceObserver !== 'undefined') {
  try {
    new PerformanceObserver((list) => {
      for (const e of list.getEntries()) {
        // A long task's entry is reported when it ends; the span running then is the one to blame
        // (usually still the same step, which may have just recorded itself).
        const at = e.startTime;
        const task: LongTask = { start: at, ms: e.duration, phase };
        longTasks.push(task);
        if (longTasks.length > MAX_TASKS) longTasks.shift();
        if (LOG) console.info(`[perf] long task: ${Math.round(e.duration)} ms at ${Math.round(at)} (${task.phase})`);
      }
    }).observe({ entryTypes: ['longtask'] });
  } catch {
    // Not supported (Safari, or an older browser): spans still work.
  }
}

// Frames the display showed that took much longer than the one before: a hitch, named by phase.
// One more `requestAnimationFrame` loop is cheap; only the comparison runs each frame.
let lastFrame = 0;
const watchFrames = (now: number) => {
  if (lastFrame && now - lastFrame >= DROP_MS && !document.hidden) {
    // A span that just ended is a better name for a hitch than the idle phase after it.
    const recent = entries[entries.length - 1];
    const named = recent && now - (recent.start + recent.ms) < DROP_MS ? recent.name : phase;
    droppedFrames.push({ at: lastFrame, gap: now - lastFrame, phase: named });
    if (droppedFrames.length > MAX_DROPS) droppedFrames.shift();
  }
  lastFrame = now;
  requestAnimationFrame(watchFrames);
};
requestAnimationFrame(watchFrames);

/** `?profile`: a summary once start-up settles (and on demand from the console). */
if (LOG) {
  window.addEventListener('load', () => setTimeout(perfSummary, 4000));
}

declare global {
  interface Window {
    __perfReport?: () => PerfReport;
  }
}
window.__perfReport = perfReport;
