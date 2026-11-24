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

/**
 * Super speed (sim-core `clock::SUPER_SPEED`): as fast as the machine goes, up to 30×, and up to 60× while
 * the household is all asleep or out. With the setting the simulation goes to it on its own as the household
 * falls quiet, and back to the speed before it as that ends; picked by the player, it stays.
 */
export const SUPER_SPEED = 4;
/** How long after a speed is sent the simulation's views may still be from before it (ms). */
const SETTLE_MS = 300;

export class GameSpeed {
  /** The speed the player chose (0: paused). */
  chosen: number;
  /** The last speed above 0 (Space resumes at it). */
  last: number;
  /** Whether the super speed running is the player's pick (`choose`), not the simulation's own (`follow`). */
  superPicked = false;
  private readonly holds = new Set<SpeedHold>(['loading']);
  /** When a speed was last sent (ms, performance clock). */
  private sentAt = -Infinity;

  /**
   * `send`: runs the simulation at a speed (and shows it), each time that changes. Loading holds
   * time from the start (the simulation starts paused: `SimBridge.start`). `saved`: the speed the
   * game was saved at, which play resumes at (a game saved paused, under a menu or in Build: at 1).
   */
  constructor(
    private readonly send: (speed: number) => void,
    saved = 1,
  ) {
    this.chosen = this.last = saved > 0 ? saved : 1;
  }

  /** The speed the simulation runs at. */
  get running(): number {
    return this.holds.size ? 0 : this.chosen;
  }

  /**
   * The player's speed (applied at once, or when nothing holds time any more). Super speed picked while the
   * simulation is at it by itself is sent all the same: from then on it is the player's, and stays when the
   * household wakes (sim-core `apply_set_speed`; unsent, it dropped back to the speed before it).
   */
  choose(speed: number): void {
    const was = this.running;
    this.chosen = speed;
    if (speed > 0) this.last = speed;
    this.superPicked = speed === SUPER_SPEED;
    this.changed(was, speed === SUPER_SPEED);
  }

  /**
   * The simulation changed speed on its own (`speed`, from its UI view): super speed by itself started as
   * the household fell quiet, or ended as someone woke or came home. The player's speed follows it without
   * sending it back; under a hold, the speed given back stays as it was (the simulation leaves super
   * speed itself if it isn't quiet then). Only super speed's comings and goings, and not just after a
   * speed was sent: any other difference is a view from before the player's last pick. `now`: ms.
   */
  follow(speed: number, now = performance.now()): void {
    if (this.holds.size || speed === this.chosen || now - this.sentAt < SETTLE_MS) return;
    if (speed !== SUPER_SPEED && this.chosen !== SUPER_SPEED) return;
    this.chosen = speed;
    if (speed > 0) this.last = speed;
    this.superPicked = false;
  }

  /** Holds time still for `reason` (`on`), or lets go of it. */
  hold(reason: SpeedHold, on: boolean): void {
    const was = this.running;
    if (on) this.holds.add(reason);
    else this.holds.delete(reason);
    this.changed(was);
  }

  /** Sends the running speed if it changed from `was` (or `again`, unless something holds time). */
  private changed(was: number, again = false): void {
    if (this.running === was && !(again && !this.holds.size)) return;
    this.sentAt = performance.now();
    this.send(this.running);
  }
}
