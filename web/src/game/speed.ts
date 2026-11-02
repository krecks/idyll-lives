/**
 * The game's speed: the one the player chose, and what holds time still over it. The simulation
 * runs at the player's speed when nothing holds it, else at 0; a speed chosen while held is the
 * one it comes back at. (Holds kept apart gave back each other's 0: Build left behind the pause
 * menu ran the world under "Paused", and Live came back stuck at a standstill.)
 */

/**
 * Why time stands still: the game is still loading (until `reveal`), a menu or window is open
 * (`setMenuPause`), or the player is in Build or Create.
 */
export type SpeedHold = 'loading' | 'menu' | 'build';

export class GameSpeed {
  /** The speed the player chose (0: paused). */
  chosen = 1;
  /** The last speed above 0 (Space resumes at it). */
  last = 1;
  private readonly holds = new Set<SpeedHold>(['loading']);

  /**
   * `send`: runs the simulation at a speed (and shows it), each time that changes. Loading holds
   * time from the start (the simulation starts paused: `SimBridge.start`).
   */
  constructor(private readonly send: (speed: number) => void) {}

  /** The speed the simulation runs at. */
  get running(): number {
    return this.holds.size ? 0 : this.chosen;
  }

  /** The player's speed (applied at once, or when nothing holds time any more). */
  choose(speed: number): void {
    const was = this.running;
    this.chosen = speed;
    if (speed > 0) this.last = speed;
    this.changed(was);
  }

  /** Holds time still for `reason` (`on`), or lets go of it. */
  hold(reason: SpeedHold, on: boolean): void {
    const was = this.running;
    if (on) this.holds.add(reason);
    else this.holds.delete(reason);
    this.changed(was);
  }

  private changed(was: number): void {
    if (this.running !== was) this.send(this.running);
  }
}
