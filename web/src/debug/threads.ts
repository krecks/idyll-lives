/**
 * What each thread is doing, for the debug overlay and debug reports: the main thread (UI and
 * drawing), the sim worker, the voice worker and the language model's worker, and the sum of them.
 */

import type { SimThreadStats } from '../core/protocol';
import { llmStats, type LlmStats } from '../llm';
import type { RenderStats } from '../render/types';
import type { VoiceDirectorStats } from '../voice';
import { voiceThreadStats, type VoiceThreadStats } from '../voice/service.svelte';

export interface ThreadStats {
  overall: {
    /** Logical cores the browser reports. */
    cores: number;
    /** Work across all the threads, in cores (1 = one core fully busy). */
    busyCores: number;
    /** Main-thread JS heap plus the sim's WASM memory, bytes (null when the browser won't say). */
    memoryBytes: number | null;
    /** Shared memory between the threads (needs cross-origin isolation). */
    isolated: boolean;
  };
  main: {
    /**
     * Share of the time spent on frames, 0..1: the game's frame callback and the browser's style,
     * layout and paint after it (`FrameMeter`; until it has measured, the callback alone).
     */
    busy: number;
    /** Share of the time in the game's frame callback alone. */
    frameWork: number;
    /** Tasks over 50 ms in the last ten seconds, and the longest, ms (null: not supported). */
    longTasks: number | null;
    longestTaskMs: number;
    heapBytes: number | null;
  };
  /** Null until the worker's first report. */
  sim: SimThreadStats | null;
  voice: VoiceThreadStats & VoiceDirectorStats;
  llm: LlmStats;
}

const LONG_TASK_WINDOW_MS = 10_000;
const longTasks: { at: number; ms: number }[] = [];
const longTasksSupported = typeof PerformanceObserver !== 'undefined' && PerformanceObserver.supportedEntryTypes?.includes('longtask');
if (longTasksSupported) {
  new PerformanceObserver((list) => {
    for (const e of list.getEntries()) longTasks.push({ at: e.startTime + e.duration, ms: e.duration });
    // Also when nobody reads them (the overlay closed): only the last ten seconds are kept.
    forgetLongTasks(performance.now());
  }).observe({ type: 'longtask', buffered: true });
}

function forgetLongTasks(now: number): void {
  while (longTasks.length && longTasks[0].at < now - LONG_TASK_WINDOW_MS) longTasks.shift();
}

/** The meter stops this long (ms) after the last reading (the overlay closed). */
const METER_IDLE_MS = 2000;

/**
 * What the browser does for each frame after its frame callbacks (style, layout, paint, handing
 * the picture over), which the game's own frame timing leaves out: a message posted from the last
 * callback of each frame arrives once the browser has finished the frame. Measured as a share of
 * the time, only while someone reads it (the overlay is open).
 */
class FrameMeter {
  private running = false;
  private lastRead = 0;
  /** Time (ms) since `since`, counted until `counted` (a frame held up past the next one counts once). */
  private spent = 0;
  private since = 0;
  private counted = 0;
  private readonly done = new MessageChannel();

  constructor() {
    this.done.port1.onmessage = (e: MessageEvent<number>) => {
      const now = performance.now();
      this.spent += Math.max(0, now - Math.max(e.data, this.counted));
      this.counted = now;
    };
  }

  /** Share of the time since the last reading, or null until frames have been measured. */
  read(): number | null {
    const now = performance.now();
    this.lastRead = now;
    const span = now - this.since;
    const share = this.running && this.counted > this.since && span > 0 ? Math.min(1, this.spent / span) : null;
    this.spent = 0;
    this.since = now;
    if (!this.running) {
      this.running = true;
      requestAnimationFrame(this.frame);
    }
    return share;
  }

  /**
   * Each frame, after the game's callback (asked for later, it runs later): a message that arrives
   * once the browser has finished the frame.
   */
  private readonly frame = () => {
    if (performance.now() - this.lastRead > METER_IDLE_MS) {
      this.running = false;
      return;
    }
    this.done.port2.postMessage(performance.now());
    requestAnimationFrame(this.frame);
  };
}

const frameMeter = new FrameMeter();

declare global {
  interface Performance {
    /** Chrome only (not in the DOM's types): the page's JavaScript heap. */
    readonly memory?: { usedJSHeapSize: number };
  }
}

export function threadStats(
  render: RenderStats,
  sim: SimThreadStats | null,
  director: VoiceDirectorStats,
): ThreadStats {
  forgetLongTasks(performance.now());
  const heapBytes = performance.memory?.usedJSHeapSize ?? null;
  const frameWork = Math.min(1, (render.cpuMs * render.fps) / 1000);
  const mainBusy = Math.min(1, frameWork + (frameMeter.read() ?? 0));
  const voice = { ...voiceThreadStats(), ...director };
  const llm = llmStats();
  // On the GPU the model keeps about one core busy while it writes.
  const llmCores = llm.busy;
  return {
    overall: {
      cores: navigator.hardwareConcurrency ?? 0,
      busyCores: mainBusy + (sim?.busy ?? 0) + voice.busy + llmCores,
      memoryBytes: heapBytes === null ? null : heapBytes + (sim?.memoryBytes ?? 0),
      isolated: self.crossOriginIsolated,
    },
    main: {
      busy: mainBusy,
      frameWork,
      longTasks: longTasksSupported ? longTasks.length : null,
      longestTaskMs: longTasks.reduce((n, t) => Math.max(n, t.ms), 0),
      heapBytes,
    },
    sim,
    voice,
    llm,
  };
}
