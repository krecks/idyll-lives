/**
 * The game camera when the watching director drives it (`Renderer.setGameShot`): glides to a
 * shot, then follows its target (a point, or residents as they move) with a slow drift. Player
 * input takes the camera back at once (`release`). Per frame it only does arithmetic on
 * preallocated poses.
 */

import type { GameShot } from '../types';
import { angleDelta, ease } from './angles';

/** The camera the rig drives: an orbit pose (any camera with these fields). */
export interface RigCamera {
  alpha: number;
  beta: number;
  radius: number;
  readonly target: { x: number; y: number; z: number };
}

interface Pose {
  alpha: number;
  beta: number;
  radius: number;
  tx: number;
  tz: number;
}

const pose = (): Pose => ({ alpha: 0, beta: 0.9, radius: 24, tx: 0, tz: 0 });

/** Default glide time (seconds). */
const GLIDE = 2.6;
/** How quickly a settled shot follows moving residents (seconds to close most of the gap). */
const FOLLOW_LAG = 0.9;
/**
 * How far (m) a camera that has caught up with whoever it follows may fall behind them again. Its easing alone trails a
 * walker by their speed times its lag: under a metre at game speed 1, 22 m at speed 4 (45 m/s), off the screen.
 */
export const FOLLOW_GAP = 1.2;

/**
 * Keeps `target` (x, z) within `FOLLOW_GAP` of (x, z) once it has caught up (`caught`: within it already, the last
 * frame); returns whether it is within it now. Before that it eases in as it will (a glide to someone far off).
 */
export function keepUp(target: { x: number; z: number }, x: number, z: number, caught: boolean): boolean {
  const dx = target.x - x;
  const dz = target.z - z;
  const gap = Math.hypot(dx, dz);
  if (gap <= FOLLOW_GAP) return true;
  if (!caught) return false;
  target.x = x + (dx / gap) * FOLLOW_GAP;
  target.z = z + (dz / gap) * FOLLOW_GAP;
  return true;
}

export class GameCameraRig {
  private readonly cur = pose();
  private readonly from = pose();
  private readonly goal = pose();
  private shot: GameShot | null = null;
  private t = 0;
  private duration = 0;
  private readonly head = { x: 0, y: 0, z: 0 };
  /** Whether the settled shot has caught up with the residents it follows (`keepUp`). */
  private caught = false;
  private readonly at = { x: 0, z: 0 };

  constructor(
    private readonly camera: RigCamera,
    private readonly limits: { radius: readonly [number, number]; beta: readonly [number, number] },
    /** Interpolated head position of resident `index`; false if not drawn. */
    private readonly simHead: (index: number, out: { x: number; y: number; z: number }) => boolean,
    /** Stops what is left of the player's last drag (`OrbitControls.stopInertia`). */
    private readonly stopInput: () => void,
  ) {}

  get active(): boolean {
    return this.shot !== null;
  }

  /** Starts a shot from wherever the camera is now; `cut` jumps there. */
  setShot(shot: GameShot, cut = false): void {
    const c = this.camera;
    if (!this.shot) Object.assign(this.cur, { alpha: c.alpha, beta: c.beta, radius: c.radius, tx: c.target.x, tz: c.target.z });
    this.shot = shot;
    const g = this.goal;
    g.alpha = shot.alpha ?? this.cur.alpha;
    g.beta = clamp(shot.beta ?? this.cur.beta, this.limits.beta);
    g.radius = clamp(shot.radius ?? this.cur.radius, this.limits.radius);
    this.aim(g);
    Object.assign(this.from, this.cur);
    this.t = 0;
    this.caught = false;
    this.duration = cut ? 0 : Math.max(0, shot.duration ?? GLIDE);
    if (this.duration === 0) Object.assign(this.cur, g);
  }

  /** Hands the camera back to the player where it is, without a jolt. */
  release(): void {
    this.shot = null;
    this.stopInput();
  }

  /** Advances and applies the pose. */
  frame(dt: number): void {
    const shot = this.shot;
    if (!shot) return;
    const g = this.goal;
    g.alpha += (shot.drift ?? 0) * dt;
    this.aim(g);
    const c = this.cur;
    if (this.t < this.duration) {
      this.t = Math.min(this.duration, this.t + dt);
      const k = ease(this.t / this.duration);
      const f = this.from;
      c.alpha = f.alpha + angleDelta(f.alpha, g.alpha) * k;
      c.beta = f.beta + (g.beta - f.beta) * k;
      c.radius = Math.exp(Math.log(f.radius) + (Math.log(g.radius) - Math.log(f.radius)) * k);
      c.tx = f.tx + (g.tx - f.tx) * k;
      c.tz = f.tz + (g.tz - f.tz) * k;
    } else {
      const k = 1 - Math.exp(-dt / FOLLOW_LAG);
      c.alpha += angleDelta(c.alpha, g.alpha) * k;
      c.beta += (g.beta - c.beta) * k;
      c.radius += (g.radius - c.radius) * k;
      c.tx += (g.tx - c.tx) * k;
      c.tz += (g.tz - c.tz) * k;
      // (Following residents, never far behind them, however fast the game runs.)
      if (shot.follow?.length) {
        this.at.x = c.tx;
        this.at.z = c.tz;
        this.caught = keepUp(this.at, g.tx, g.tz, this.caught);
        c.tx = this.at.x;
        c.tz = this.at.z;
      }
    }
    const cam = this.camera;
    cam.alpha = c.alpha;
    cam.beta = c.beta;
    cam.radius = c.radius;
    cam.target.x = c.tx;
    cam.target.y = 0;
    cam.target.z = c.tz;
    // The director owns the camera: no leftover spin from the player's last drag.
    this.stopInput();
  }

  /** The shot's target: its point, or the middle of the residents it follows (when drawn). */
  private aim(g: Pose): void {
    const shot = this.shot;
    if (!shot) return;
    if (shot.follow?.length) {
      let x = 0;
      let z = 0;
      let n = 0;
      for (const i of shot.follow) {
        if (!this.simHead(i, this.head)) continue;
        x += this.head.x;
        z += this.head.z;
        n++;
      }
      if (n) {
        g.tx = x / n;
        g.tz = z / n;
        return;
      }
    }
    if (shot.target) {
      g.tx = shot.target.x;
      g.tz = shot.target.z;
    }
  }
}

function clamp(v: number, [lo, hi]: readonly [number, number]): number {
  return Math.min(hi, Math.max(lo, v));
}
