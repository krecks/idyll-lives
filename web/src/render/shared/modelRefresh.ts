/**
 * When the world is built again with model files that came after failing (engine-neutral; the
 * Babylon renderer's `fileArrived` and `holdModelRefresh`): the files are gathered, and the
 * rebuild (`flush`, with every file that came) waits until no file has come for
 * `FILES_SETTLE_MS` (after an outage many come one after another, for one rebuild), but at most
 * `FILES_WAIT_MS` from the first. While held (`hold`: the game streams the rest of the town behind
 * the house) files are only gathered; let go, they're built at once. What can be built again
 * without the world (the ghost in hand, the landscape's plants) is the renderer's, as each file
 * comes.
 */

/** The wait (ms) after the last file came, and the longest from the first. */
export const FILES_SETTLE_MS = 1500;
export const FILES_WAIT_MS = 5000;

export class ModelRefresh {
  private readonly arrived = new Set<string>();
  private timer: ReturnType<typeof setTimeout> | null = null;
  private since = 0;
  private held = false;

  constructor(private readonly flush: (urls: string[]) => void) {}

  /** A model file that had failed has come. */
  fileArrived(url: string): void {
    const now = performance.now();
    if (!this.arrived.size) this.since = now;
    this.arrived.add(url);
    if (this.held) return;
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => this.run(), Math.max(0, Math.min(FILES_SETTLE_MS, this.since + FILES_WAIT_MS - now)));
  }

  /**
   * Holds the builds that streamed model files would set off, then lets them go at once. Left
   * alone, each settle (1.5 s, or 5 s at most after the first) built the whole world again.
   */
  hold(on: boolean): void {
    this.held = on;
    if (on || !this.arrived.size) return;
    this.run();
  }

  /** The files gathered go for nothing (the renderer went). */
  dispose(): void {
    if (this.timer) clearTimeout(this.timer);
    this.timer = null;
    this.arrived.clear();
  }

  private run(): void {
    if (this.timer) clearTimeout(this.timer);
    this.timer = null;
    const urls = [...this.arrived];
    this.arrived.clear();
    if (urls.length) this.flush(urls);
  }
}
