/**
 * The frame rate cap, paced in whole display refreshes. Babylon's own cap (`engine.maxFPS`) draws
 * once enough time has piled up since the last frame: at 60 on a 120 Hz display, after a frame
 * that came a refresh late it draws the next one a refresh later to catch up, so frames stay on
 * screen for 1, 2 or 3 refreshes and motion judders. The pacer instead draws every `divisor`-th
 * refresh counted from the last frame drawn, measured with the timestamps `requestAnimationFrame`
 * gives (the refresh each frame is for).
 *
 * A cap the refresh rate doesn't divide becomes the nearest rate it does, up to a quarter above
 * the cap: 60 is 72 fps on a 144 Hz display, 60 on 120 Hz, 75 on 75 Hz; 30 is 36 on 144 Hz.
 *
 * It plugs into Babylon's render loop as `engine.customAnimationFrameRequester`, and keeps the
 * timestamp of the refresh being drawn (`frameTime`) for the game's clocks. `pacingDivisor` is pure.
 */

/** A cap may round up this much (60 is 72 on 144 Hz, rather than 48). */
const ROUND_UP = 1.25;
/** Leeway for a measured rate a little off the screen's nominal one (75.4 Hz still shows a 60 cap as 75). */
const TOLERANCE = 1.02;
/** Refresh intervals kept to measure the display's rate, how many make a first estimate, and how often it is worked out again (refreshes). */
const SAMPLES = 64;
const FIRST_ESTIMATE = 16;
const ESTIMATE_EVERY = 32;
/** Gaps between refreshes outside this range (ms) aren't refreshes: a hidden tab, a stopped loop, a timer. */
const SHORTEST = 1000 / 500;
const LONGEST = 1000 / 20;

/** Every how many refreshes to draw, for a cap (frames per second; 0: none) on a display refreshing `hz` times a second. */
export function pacingDivisor(cap: number, hz: number): number {
  if (!(cap > 0) || !(hz > 0)) return 1;
  return Math.max(1, Math.ceil(hz / (cap * ROUND_UP * TOLERANCE)));
}

/**
 * The display's refresh interval from gaps between refreshes (ms, sorted): the usual short gap.
 * Longer gaps are refreshes a busy frame missed, so it's the average of the gaps around the
 * shortest usual one (the 10th percentile; one-off short gaps, as after a late frame, don't count).
 */
export function refreshInterval(sorted: Float64Array): number {
  const floor = sorted[Math.floor(sorted.length * 0.1)];
  let sum = 0;
  let count = 0;
  for (const gap of sorted) {
    if (gap < floor * 0.75 || gap > floor * 1.5) continue;
    sum += gap;
    count++;
  }
  return sum / count;
}

export class FramePacer {
  /** The display's refresh interval as measured (ms; 60 Hz until measured), and the refreshes per drawn frame. */
  interval = 1000 / 60;
  divisor = 1;
  /** The `requestAnimationFrame` timestamp of the frame being drawn (ms, the `performance.now()` clock). */
  frameTime = 0;
  /** Babylon's `ICustomAnimationFrameRequester`: the pending request (any non-zero id). */
  requestID?: number;
  private cap = 0;
  private callback: ((time: number) => void) | null = null;
  private raf = 0;
  private lastTick = 0;
  private lastDrawn = -Infinity;
  private readonly gaps = new Float64Array(SAMPLES);
  private readonly sorted = new Float64Array(SAMPLES);
  private gapCount = 0;

  /** Frames per second at most (0: every refresh). */
  setCap(fps: number): void {
    this.cap = fps;
    this.divisor = pacingDivisor(fps, 1000 / this.interval);
  }

  /** The display's refresh rate as measured (Hz). */
  get refreshHz(): number {
    return 1000 / this.interval;
  }

  // Arrow functions, so they work unbound (as Babylon's engine called them: `const { requestAnimationFrame } = requester`).

  readonly requestAnimationFrame = (fn: (time: number) => void): number => {
    this.callback = fn;
    if (!this.raf) this.raf = requestAnimationFrame(this.tick);
    return this.raf;
  };

  readonly cancelAnimationFrame = (): void => {
    this.callback = null;
    if (this.raf) cancelAnimationFrame(this.raf);
    this.raf = 0;
    this.lastTick = 0;
  };

  private readonly tick = (time: number): void => {
    this.raf = 0;
    if (this.lastTick) this.measure(time - this.lastTick);
    this.lastTick = time;
    const fn = this.callback;
    if (!fn) return;
    // Not this refresh: wait for the next (rounded: a refresh is never exactly on time).
    if (this.divisor > 1 && Math.round((time - this.lastDrawn) / this.interval) < this.divisor) {
      this.raf = requestAnimationFrame(this.tick);
      return;
    }
    this.callback = null;
    this.lastDrawn = this.frameTime = time;
    fn(time);
  };

  private measure(gap: number): void {
    if (gap < SHORTEST || gap > LONGEST) return;
    this.gaps[this.gapCount % SAMPLES] = gap;
    const count = ++this.gapCount;
    if (count !== FIRST_ESTIMATE && count % ESTIMATE_EVERY !== 0) return;
    const n = Math.min(count, SAMPLES);
    const sorted = n < SAMPLES ? this.sorted.subarray(0, n) : this.sorted;
    sorted.set(n < SAMPLES ? this.gaps.subarray(0, n) : this.gaps);
    sorted.sort();
    const interval = refreshInterval(sorted);
    // (Small drift isn't a new display: keep the divisor steady.)
    if (Math.abs(interval - this.interval) <= this.interval * 0.005) return;
    this.interval = interval;
    this.divisor = pacingDivisor(this.cap, 1000 / interval);
  }
}
