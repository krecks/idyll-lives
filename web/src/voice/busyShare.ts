/**
 * Whether making speech makes something worse (`index.ts`: the frames, the simulation keeping up):
 * moments (frames, the simulation's seconds) counted while speech is being made and while it isn't,
 * and how many of each went badly (slow, behind).
 */
export class BusyShare {
  private busy = 0;
  private busyBad = 0;
  private idle = 0;
  private idleBad = 0;

  /** Judged every `window` busy moments: worse when over `share` of them went badly. */
  constructor(
    private readonly window: number,
    private readonly share: number,
  ) {}

  /**
   * Counts a moment. True once `window` busy ones say speech makes it worse: over `share` of them
   * bad, and over twice the idle ones' share (a game slow anyway isn't the voices' fault); the
   * count then starts again.
   */
  add(busy: boolean, bad: boolean): boolean {
    if (busy) {
      this.busy++;
      if (bad) this.busyBad++;
    } else {
      this.idle++;
      if (bad) this.idleBad++;
    }
    if (this.busy < this.window) return false;
    const busyShare = this.busyBad / this.busy;
    const idleShare = this.idle ? this.idleBad / this.idle : 0;
    this.reset();
    return busyShare > this.share && busyShare > idleShare * 2;
  }

  reset(): void {
    this.busy = this.busyBad = this.idle = this.idleBad = 0;
  }
}
