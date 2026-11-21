/**
 * The menus' town overview camera: frames a shot's rectangle into its part of the screen, glides
 * between shots, drifts and sways, takes the player's orbit and zoom on top, and keeps above the
 * land. Engine-neutral: it writes an orbit pose (the game's orbit conventions, `shared/orbit.ts`,
 * with a view offset that moves the picture by `targetScreenOffset` view units) into any camera
 * with those fields.
 */

import { groundDepth } from '../../core/protocol';
import type { TownShot } from '../types';
import { angleDelta, ease } from './angles';

/**
 * How far (m, game space) a view offset (`ox`, `oy`: the picture moved by that much to the right and up, as
 * `targetScreenOffset` has it) moves an orbit camera at `alpha`, `beta` and its target: along the view's right and up
 * (a left-handed look-at, up y), into `out`. The camera is drawn from there (`pc/camera.ts` `setPose`), so what keeps
 * it clear of the land looks from there too.
 */
export function viewShift(alpha: number, beta: number, ox: number, oy: number, out: [number, number, number]): [number, number, number] {
  if (ox === 0 && oy === 0) {
    out[0] = out[1] = out[2] = 0;
    return out;
  }
  const [sa, ca, sb, cb] = [Math.sin(alpha), Math.cos(alpha), Math.sin(beta), Math.cos(beta)];
  const [fx, fy, fz] = [-ca * sb, -cb, -sa * sb];
  const rl = Math.hypot(fz, fx) || 1;
  const [rx, rz] = [fz / rl, -fx / rl];
  const [ux, uy, uz] = [fy * rz, fz * rx - fx * rz, -fy * rx];
  out[0] = -(ox * rx + oy * ux);
  out[1] = -(oy * uy);
  out[2] = -(ox * rz + oy * uz);
  return out;
}

/** The camera the overview drives: any with these fields. */
export interface OverviewCameraTarget {
  alpha: number;
  beta: number;
  radius: number;
  readonly target: { x: number; y: number; z: number };
  /** Moves the picture by this much in view units (the camera and its target slid along the view's right and up). */
  readonly targetScreenOffset: { x: number; y: number };
  /** Vertical field of view (radians). */
  readonly fov: number;
}

/** How tall the town stands (m), framed with it. */
export const OVERVIEW_TOWN_HEIGHT = 7;

/** How near (radians, and parts of the radius) a settled camera comes to its goal before it's put there. */
const SNAP = 1e-4;
/** The player's zoom comes in as near as this (m from the target), however far out the shot is: the menus' own. */
export const OVERVIEW_NEAREST = 50;
/** The nearest of the new game's land, streets and build steps (`OverviewCamera.setNearest`): down among the houses. */
export const OVERVIEW_CLOSE = 10;
/** The player's tilt (`nudge`'s `dBeta`) stays within these (radians from straight down). */
export const TILT_MIN = 0.15;
export const TILT_MAX = 1.35;
/**
 * The camera's target is lifted onto the land under it as the camera comes in: not at all from this far out (m)
 * (a shot is framed about the land's datum), wholly from `LIFT_NEAR` in (a close look on a hill).
 */
const LIFT_FAR = 260;
const LIFT_NEAR = 80;
/**
 * No shot is framed from further out than this (m from the target), and the player's zoom goes out no further: the
 * whole of the largest land (960 m) fits the narrowest stage beside the side panel well within it.
 */
export const OVERVIEW_FARTHEST = 2400;
/** Where along the way from the camera (0) to its target (1) `hidden` looks for the land in the way. */
const HIDDEN_AT = [0, 0.12, 0.25, 0.4, 0.55, 0.7] as const;
/** The ways round the eye `hidden` looks for land over it (`clear` m off each way). */
const AROUND = [
  [1, 0],
  [-1, 0],
  [0, 1],
  [0, -1],
] as const;
/** How far (m) the player may pan the camera off the shot's focus when no land is given to keep it over (`bounds`). */
const PAN_REACH = 1500;
/**
 * With a landscape round the land (`setBounds`'s `reach`), the least share of the picture's height the land keeps as the
 * camera tilts low (`reachTilt`): from far out, low, the camera looked across the haze past the landscape, the ring's
 * hills before the land standing in it, at the land as a strip a fifth of the picture high.
 */
export const LAND_TALL = 0.4;
/** Or this share of the picture's height the land has at the shot's own tilt, if that is less (`reachTilt`). */
export const LAND_KEEP = 0.8;
/**
 * The land's sides the eye stands past are seen over the landscape's ring (`overRing`) at this many points each, its
 * ground read this many times along the way to each, from where the way comes into the landscape to the land's edge.
 */
const RING_POINTS = 5;
const RING_STEPS = 20;
/** How far (m) the way to the land's edge passes over the ring's ground at least (`overRing`; the curtain's top is 0.3 m over it). */
const RING_CLEAR = 2;

/**
 * The part of the way from `e` on along `d` (0 at `e`, 1 at its end) at which it comes into `a`..`b` (0 from inside; past
 * 1, or infinite, if it never does). One axis of a box.
 */
function enter(e: number, d: number, a: number, b: number): number {
  if (e >= a && e <= b) return 0;
  if (d === 0) return Infinity;
  const s = ((e < a ? a : b) - e) / d;
  return s >= 0 ? s : Infinity;
}

/** A rectangle of the world (m) the camera's target is kept over. */
export interface OverviewBounds {
  x0: number;
  z0: number;
  x1: number;
  z1: number;
}

interface Pose {
  alpha: number;
  beta: number;
  radius: number;
  tx: number;
  tz: number;
  /** View-space offset (`targetScreenOffset`), frames the focus off-centre. */
  ox: number;
  oy: number;
}

const pose = (): Pose => ({ alpha: 0, beta: 0.9, radius: 60, tx: 0, tz: 0, ox: 0, oy: 0 });

/**
 * Drives the camera in the overview: shots framed into a screen rectangle (fit by projecting
 * the focus box), glides between them (ease in-out), a slow drift and user nudges. Per frame it
 * only does arithmetic on preallocated poses.
 */
export class OverviewCamera {
  private readonly cur = pose();
  private readonly from = pose();
  private readonly goal = pose();
  private shot: TownShot | null = null;
  private t = 0;
  private duration = 0;
  private drift = 0;
  private sway = 0;
  private swayTime = 0;
  private swayBase = 0;
  /** User orbit and zoom on top of the shot. */
  private userAlpha = 0;
  private userZoom = 1;
  /** The player's tilt (radians from straight down) in place of the shot's; null: the shot's own. */
  private tilt: number | null = null;
  /** How near (m) the player's zoom comes in (`setNearest`). */
  private nearest = OVERVIEW_NEAREST;
  /** The player's pan (m) on top of the shot's focus. */
  private panX = 0;
  private panZ = 0;
  private aspect = 16 / 9;
  private viewW = 1920;
  private viewH = 1080;
  private onArrive: (() => void) | null = null;

  /** Height of the land round the town (null: flat), so the camera stays above it with the town in sight. */
  ground: ((x: number, z: number) => number) | null = null;
  /** The land the player's pan keeps the camera's target over (null: within `PAN_REACH` of the shot's focus). */
  private bounds: OverviewBounds | null = null;
  /**
   * The landscape round the land (null: none): the camera's tilt is kept to what keeps the land more than a strip, and
   * its edge in sight over the landscape's ring (`reachTilt`).
   */
  private reach: OverviewBounds | null = null;
  /** `reachTilt`'s last pose and what it gave (the camera at rest asks the same again every frame). */
  private readonly tiltAsked = [NaN, NaN, NaN, NaN, NaN, NaN, NaN, NaN, NaN];
  private tiltGiven = 0;
  /** `hidden`'s view offset, worked out in place. */
  private readonly shift: [number, number, number] = [0, 0, 0];

  constructor(private readonly camera: OverviewCameraTarget) {}

  /** Copies the camera's pose (entering the overview from the game). */
  syncFromCamera(): void {
    const c = this.camera;
    Object.assign(this.cur, { alpha: c.alpha, beta: c.beta, radius: c.radius, tx: c.target.x, tz: c.target.z, ox: c.targetScreenOffset.x, oy: c.targetScreenOffset.y });
  }

  setViewport(width: number, height: number): void {
    if (width === this.viewW && height === this.viewH) return;
    this.viewW = width;
    this.viewH = height;
    this.refit();
  }

  /** Re-fits the current shot (viewport changed), keeping the drift/sway phase. */
  private refit(): void {
    if (!this.shot) return;
    const alpha = this.goal.alpha;
    const base = this.swayBase;
    this.fitGoal({ ...this.shot, alpha: this.shot.alpha === undefined ? undefined : alpha });
    this.goal.alpha = alpha;
    this.swayBase = base;
  }

  /**
   * The land the camera's target is kept over while the player pans (null: none); the pan already made is pulled in.
   * `reach`: the landscape drawn round it (its box, the land's within it), and the camera tilts no lower than keeps the
   * land more than a strip, and its edge in sight over the landscape's ring (`reachTilt`).
   */
  setBounds(bounds: OverviewBounds | null, reach: OverviewBounds | null = null): void {
    this.bounds = bounds;
    this.reach = reach;
    this.tiltAsked[0] = NaN;
    this.clampPan();
  }

  /**
   * How near (m from the target) the player's zoom comes in from now on: `OVERVIEW_NEAREST` in the menus,
   * `OVERVIEW_CLOSE` in the new game's steps that shape the town. A zoom already nearer is pulled back out.
   */
  setNearest(metres: number): void {
    this.nearest = Math.max(1, metres);
    this.nudge(0, 1);
  }

  /** Frames `shot`; with `duration` 0 (or `cut`) it jumps there. */
  setShot(shot: TownShot, cut = false): void {
    const alphaChanged = shot.alpha !== undefined && shot.alpha !== this.shot?.alpha;
    if (alphaChanged) this.userAlpha = 0;
    if (shot.zoom !== this.shot?.zoom || alphaChanged) this.userZoom = 1;
    if ((shot.beta ?? 0.9) !== (this.shot?.beta ?? 0.9) || alphaChanged) this.tilt = null;
    // (Another part of the world framed: the player's pan is let go.)
    const f = this.shot?.focus;
    if (!f || f.x !== shot.focus.x || f.z !== shot.focus.z || f.w !== shot.focus.w || f.d !== shot.focus.d) this.panX = this.panZ = 0;
    this.shot = shot;
    this.drift = shot.drift ?? 0;
    this.sway = shot.sway ?? 0;
    this.swayTime = 0;
    this.fitGoal(shot);
    this.swayBase = this.goal.alpha;
    this.clampPan();
    this.startGlide(cut ? 0 : (shot.duration ?? 1.2));
  }

  /** The player's orbit, zoom and pan let go: the camera glides back to the shot as it frames it (`seconds`; 0 cuts). */
  reframe(seconds = 1.2): void {
    if (!this.shot) return;
    this.userAlpha = 0;
    this.userZoom = 1;
    this.tilt = null;
    this.panX = this.panZ = 0;
    // (The shot's drift and sway back, from the way the camera faces now.)
    this.drift = this.shot.drift ?? 0;
    this.sway = this.shot.sway ?? 0;
    this.swayTime = 0;
    this.startGlide(seconds);
  }

  /** Glides to an exact camera pose (the game's view of a lot) and calls back on arrival. */
  glideTo(target: { alpha: number; beta: number; radius: number; x: number; z: number }, seconds: number, done: () => void): void {
    this.shot = null;
    this.drift = 0;
    this.sway = 0;
    this.userAlpha = 0;
    this.userZoom = 1;
    this.tilt = null;
    this.panX = this.panZ = 0;
    Object.assign(this.goal, { alpha: target.alpha, beta: target.beta, radius: target.radius, tx: target.x, tz: target.z, ox: 0, oy: 0 });
    this.onArrive = done;
    this.startGlide(seconds);
  }

  /**
   * The player's orbit (radians), zoom (a factor) and tilt (radians, more is lower) on top of the shot. The zoom comes
   * in to the nearest (`setNearest`) at most; the tilt stays within `TILT_MIN`–`TILT_MAX`. Says by how much the zoom
   * changed (a factor: 1 where it was already at its limit).
   */
  nudge(dAlpha: number, zoom: number, dBeta = 0): number {
    if (dAlpha || dBeta || zoom !== 1) this.taken();
    this.userAlpha += dAlpha;
    if (dBeta) this.tilt = Math.min(TILT_MAX, Math.max(TILT_MIN, (this.tilt ?? this.goal.beta) + dBeta));
    // (Out to the shot's distance over 0.6, and never past `OVERVIEW_FARTHEST`.)
    const out = Math.min(1, Math.max(0.6, this.goal.radius / OVERVIEW_FARTHEST));
    const before = this.userZoom;
    this.userZoom = Math.min(Math.max(2.6, this.goal.radius / this.nearest), Math.max(out, this.userZoom * zoom));
    return this.userZoom / before;
  }

  /**
   * The player's zoom by `zoom` towards the ground point (`x`, `z`) (the one under the pointer): the camera's target
   * moves towards it as the camera comes in, so it stays under the pointer.
   */
  zoomAt(x: number, z: number, zoom: number): void {
    const k = this.nudge(0, zoom);
    if (k === 1) return;
    const [tx, tz] = [this.goal.tx + this.panX, this.goal.tz + this.panZ];
    this.pan((x - tx) * (1 - 1 / k), (z - tz) * (1 - 1 / k));
  }

  /**
   * The player's pan: the camera's target moved (`dx`, `dz`) m over the ground, on top of the shot's focus. It stays
   * over the land (`setBounds`), or within `PAN_REACH` of the focus without one.
   */
  pan(dx: number, dz: number): void {
    if (dx || dz) this.taken();
    this.panX += dx;
    this.panZ += dz;
    this.clampPan();
  }

  /** The player's pan along the view: `right` m to the picture's right and `forward` m into it, over the ground. */
  panView(right: number, forward: number): void {
    // (The view's right and its way into the picture, flat on the ground: as `fitGoal`'s basis.)
    const [ca, sa] = [Math.cos(this.cur.alpha), Math.sin(this.cur.alpha)];
    this.pan(-sa * right - ca * forward, ca * right - sa * forward);
  }

  /** The player moved the camera: the shot's drift and sway stop where they are (until the shot is framed again). */
  private taken(): void {
    if (!this.drift && !this.sway) return;
    this.swayBase = this.goal.alpha;
    this.drift = this.sway = 0;
  }

  /** The player's pan pulled in so the target (the shot's, moved by it) stays over the land. */
  private clampPan(): void {
    const b = this.bounds;
    if (!b || !this.shot) {
      this.panX = Math.min(PAN_REACH, Math.max(-PAN_REACH, this.panX));
      this.panZ = Math.min(PAN_REACH, Math.max(-PAN_REACH, this.panZ));
      return;
    }
    const { tx, tz } = this.goal;
    this.panX = Math.min(Math.max(this.panX, Math.min(0, b.x0 - tx)), Math.max(0, b.x1 - tx));
    this.panZ = Math.min(Math.max(this.panZ, Math.min(0, b.z0 - tz)), Math.max(0, b.z1 - tz));
  }

  /** Where the camera's target is now (m), and its distance from it: for a drag over the land (`PlayCanvasRenderer.dragTown`). */
  get pose(): { readonly tx: number; readonly tz: number; readonly radius: number } {
    return this.cur;
  }

  /** Advances and applies the pose. */
  frame(dt: number, aspect: number): void {
    if (Math.abs(aspect - this.aspect) > 1e-3) {
      this.aspect = aspect;
      this.refit();
    }
    this.swayBase += this.drift * dt;
    if (this.sway) {
      this.swayTime += dt;
      this.goal.alpha = this.swayBase + this.sway * Math.sin((this.swayTime * Math.PI * 2) / 70);
    } else this.goal.alpha = this.swayBase;
    const g = this.goal;
    const c = this.cur;
    const zoom = this.userZoom;
    const ga = g.alpha + this.userAlpha;
    const gb = this.tilt ?? g.beta;
    const [gx, gz] = [g.tx + this.panX, g.tz + this.panZ];
    if (this.t < this.duration) {
      this.t = Math.min(this.duration, this.t + dt);
      const k = ease(this.t / this.duration);
      const f = this.from;
      c.alpha = f.alpha + angleDelta(f.alpha, ga) * k;
      c.beta = f.beta + (gb - f.beta) * k;
      // Zooms glide in log space (a long pull-back doesn't rush at the start).
      c.radius = Math.exp(Math.log(f.radius) + (Math.log(g.radius / zoom) - Math.log(f.radius)) * k);
      c.tx = f.tx + (gx - f.tx) * k;
      c.tz = f.tz + (gz - f.tz) * k;
      c.ox = f.ox + ((g.ox * c.radius) / g.radius - f.ox) * k;
      c.oy = f.oy + ((g.oy * c.radius) / g.radius - f.oy) * k;
      if (this.t >= this.duration && this.onArrive) {
        const done = this.onArrive;
        this.onArrive = null;
        done();
      }
    } else {
      // Settled: follow drift and nudges smoothly.
      const k = 1 - Math.exp(-dt / 0.25);
      c.alpha += angleDelta(c.alpha, ga) * k;
      c.beta += (gb - c.beta) * k;
      c.radius += (g.radius / zoom - c.radius) * k;
      c.tx += (gx - c.tx) * k;
      c.tz += (gz - c.tz) * k;
      c.ox += ((g.ox * c.radius) / g.radius - c.ox) * k;
      c.oy += ((g.oy * c.radius) / g.radius - c.oy) * k;
      // Within a hair of the goal: there (a camera that keeps creeping is never still: held frames).
      if (Math.abs(angleDelta(c.alpha, ga)) + Math.abs(gb - c.beta) < SNAP && Math.abs(g.radius / zoom - c.radius) + Math.abs(gx - c.tx) + Math.abs(gz - c.tz) < SNAP * c.radius) {
        c.alpha = ga;
        c.beta = gb;
        c.radius = g.radius / zoom;
        c.tx = gx;
        c.tz = gz;
        c.ox = (g.ox * c.radius) / g.radius;
        c.oy = (g.oy * c.radius) / g.radius;
      }
    }
    this.apply();
  }

  /** Writes the pose into the camera. */
  apply(): void {
    const cam = this.camera;
    const c = this.cur;
    cam.alpha = c.alpha;
    // (Near, the target on the land under it: on a hill the camera would be in it otherwise.)
    const lift = Math.min(1, Math.max(0, (LIFT_FAR - c.radius) / (LIFT_FAR - LIFT_NEAR)));
    const ty = lift > 0 && this.ground ? this.ground(c.tx, c.tz) * lift : 0;
    // Looking from higher up while the camera would be far out past the landscape, or the ring's hills or the land (a
    // mountain) would stand in the way.
    let beta = this.reachTilt(c.alpha, c.beta, c.radius, c.tx, ty, c.tz, c.ox, c.oy);
    for (let k = 0; k < 30 && beta > 0.2 && this.hidden(c.alpha, beta, c.radius, c.tx, ty, c.tz, c.ox, c.oy); k++) beta -= 0.04;
    cam.beta = beta;
    cam.radius = c.radius;
    cam.target.x = c.tx;
    cam.target.y = ty;
    cam.target.z = c.tz;
    cam.targetScreenOffset.x = c.ox;
    cam.targetScreenOffset.y = c.oy;
  }

  /**
   * The tilt (rad from straight down) at most `beta` from which the land (`bounds`, with a landscape round it: `reach`)
   * keeps `LAND_TALL` of the picture's height, or `LAND_KEEP` of what it has at the shot's own tilt if that is less (a
   * narrow window's, fitted by its width), and its edge is seen over the landscape's ring (`overRing`), for the camera
   * as drawn (moved by the view offset `ox`, `oy`), its target at (`tx`, `ty`, `tz`) `radius` m off along `alpha`: from
   * far out, a low camera saw the land as a strip behind the ring's hills, and on a mountain place the ring's near
   * mountains, wholly in the veil, hid half of it below a hard pale line. Eased as the camera zooms or pans (the most
   * that keeps it so, not steps). The same pose as last asked is answered as it was.
   */
  private reachTilt(alpha: number, beta: number, radius: number, tx: number, ty: number, tz: number, ox: number, oy: number): number {
    if (!this.bounds || !this.reach) return beta;
    const asked = this.tiltAsked;
    const goal = this.goal.beta;
    if (asked[0] === alpha && asked[1] === beta && asked[2] === radius && asked[3] === tx && asked[4] === ty && asked[5] === tz && asked[6] === ox && asked[7] === oy && asked[8] === goal) return this.tiltGiven;
    const least = Math.min(LAND_TALL, LAND_KEEP * this.landTall(alpha, Math.min(beta, goal), radius, tx, tz, ox, oy));
    const keeps = (b: number) => this.landTall(alpha, b, radius, tx, tz, ox, oy) >= least && this.overRing(alpha, b, radius, tx, ty, tz, ox, oy);
    let given = beta;
    if (!keeps(beta)) {
      // (Further out the lower it looks: the most that keeps it so, to a ten-thousandth of a radian.)
      let [lo, hi] = [0, beta];
      for (let k = 0; k < 14; k++) {
        const mid = (lo + hi) / 2;
        if (keeps(mid)) lo = mid;
        else hi = mid;
      }
      given = lo;
    }
    [asked[0], asked[1], asked[2], asked[3], asked[4], asked[5], asked[6], asked[7], asked[8]] = [alpha, beta, radius, tx, ty, tz, ox, oy, goal];
    this.tiltGiven = given;
    return given;
  }

  /**
   * Whether, from the camera as drawn out past the landscape (`reach`), the land's edge on each side it stands past is
   * seen over the landscape's ring (outside the land's `bounds`), by `RING_CLEAR` at least: at `RING_POINTS` along each
   * such side, at the land's height there, against the ring's ground on the way to it from where the way comes into the
   * landscape (its cut edge, the curtain closing it as high as the ground there) to the land's edge. From there what
   * stands in the way is the curtain and the ring by it, wholly in the veil: a hard pale line across the land. From
   * within the landscape it is (the ring's mountains near the eye are as clear as the veil leaves what is near it, and
   * stand before the land as they are), and without the land's height (`ground`). Allocation-free.
   */
  private overRing(alpha: number, beta: number, radius: number, tx: number, ty: number, tz: number, ox: number, oy: number): boolean {
    const ground = this.ground;
    const r = this.bounds!;
    const out = this.reach!;
    if (!ground) return true;
    const [sx, sy, sz] = viewShift(alpha, beta, ox, oy, this.shift);
    const flat = Math.sin(beta) * radius;
    const [ex, ey, ez] = [tx + Math.cos(alpha) * flat + sx, ty + Math.cos(beta) * radius + sy, tz + Math.sin(alpha) * flat + sz];
    if (ex >= out.x0 && ex <= out.x1 && ez >= out.z0 && ez <= out.z1) return true;
    for (let side = 0; side < 4; side++) {
      // (The sides the eye is past: x0, x1, z0, z1.)
      const alongX = side >= 2;
      const at = side === 0 ? r.x0 : side === 1 ? r.x1 : side === 2 ? r.z0 : r.z1;
      const past = side === 0 ? ex < r.x0 : side === 1 ? ex > r.x1 : side === 2 ? ez < r.z0 : ez > r.z1;
      if (!past) continue;
      for (let k = 0; k < RING_POINTS; k++) {
        const f = k / (RING_POINTS - 1);
        const [px, pz] = alongX ? [r.x0 + (r.x1 - r.x0) * f, at] : [at, r.z0 + (r.z1 - r.z0) * f];
        const [dx, dz] = [px - ex, pz - ez];
        const far = Math.hypot(dx, dz);
        if (far < 1) continue;
        // (Where the way to it comes into the landscape, and where it reaches the land: parts of the way.)
        const s0 = Math.max(0, enter(ex, dx, out.x0, out.x1), enter(ez, dz, out.z0, out.z1));
        const s1 = Math.max(enter(ex, dx, r.x0, r.x1), enter(ez, dz, r.z0, r.z1));
        if (!(s1 > s0)) continue;
        const down = (ground(px, pz) - ey) / far;
        for (let n = 0; n < RING_STEPS; n++) {
          const s = s0 + ((s1 - s0) * n) / RING_STEPS;
          if (s > 0 && (ground(ex + dx * s, ez + dz * s) + RING_CLEAR - ey) / (far * s) > down) return false;
        }
      }
    }
    return true;
  }

  /**
   * The share (0..1) of the picture's height the land (`bounds`, at its datum: its corners, the middles of its sides
   * and its middle) takes from the camera as drawn (moved by the view offset `ox`, `oy`) at `alpha`, `beta`, `radius`
   * m from (`tx`, `tz`), through its vertical field of view. Allocation-free.
   */
  private landTall(alpha: number, beta: number, radius: number, tx: number, tz: number, ox: number, oy: number): number {
    const r = this.bounds!;
    const [sx, sy, sz] = viewShift(alpha, beta, ox, oy, this.shift);
    const [sa, ca, sb, cb] = [Math.sin(alpha), Math.cos(alpha), Math.sin(beta), Math.cos(beta)];
    const [ex, ey, ez] = [tx + ca * sb * radius + sx, cb * radius + sy, tz + sa * sb * radius + sz];
    // (The view's way in and up, as `viewShift`'s.)
    const [fx, fy, fz] = [-ca * sb, -cb, -sa * sb];
    const rl = Math.hypot(fz, fx) || 1;
    const [rx, rz] = [fz / rl, -fx / rl];
    const [ux, uy, uz] = [fy * rz, fz * rx - fx * rz, -fy * rx];
    const tanY = Math.tan(this.camera.fov / 2);
    let [bottom, top] = [1, -1];
    for (let i = 0; i <= 2; i++) {
      for (let j = 0; j <= 2; j++) {
        const [dx, dy, dz] = [r.x0 + ((r.x1 - r.x0) * i) / 2 - ex, -ey, r.z0 + ((r.z1 - r.z0) * j) / 2 - ez];
        const depth = dx * fx + dy * fy + dz * fz;
        // (Behind the camera: below the picture, as it looks down.)
        const y = depth > 0 ? (dx * ux + dy * uy + dz * uz) / (depth * tanY) : -1;
        bottom = Math.min(bottom, Math.max(-1, y));
        top = Math.max(top, Math.min(1, y));
      }
    }
    return Math.max(0, (top - bottom) / 2);
  }

  /**
   * Whether the camera at this pose (its target at height `ty`) is in the ground or has the land between it and the
   * target. It keeps `min(12, radius × 0.35)` m over the land (and, as drawn, moved by the view offset `ox`, `oy`, over
   * the land under it and that far round it), and the line to the target clear of the treetops (14 m) or by
   * `radius × 0.35` for its share of the way, the less (so a camera come in close may still look low along a flat
   * street).
   */
  private hidden(alpha: number, beta: number, radius: number, tx: number, ty: number, tz: number, ox: number, oy: number): boolean {
    if (!this.ground) return false;
    const [dx, dy, dz] = [Math.cos(alpha) * Math.sin(beta) * radius, Math.cos(beta) * radius, Math.sin(alpha) * Math.sin(beta) * radius];
    const clear = Math.min(12, radius * 0.35);
    for (const t of HIDDEN_AT) {
      const [x, y, z] = [tx + dx * (1 - t), ty + dy * (1 - t), tz + dz * (1 - t)];
      // Clear of the treetops too.
      if (y < this.ground(x, z) + (t === 0 ? clear : Math.min(14, radius * 0.35 * (1 - t)))) return true;
    }
    // The camera as it is drawn, moved by the view offset (tens of metres from far out: `viewShift`), over the land
    // under it and `clear` m round it either way: not in the ground, nor down a cutting or a gorge under its sides (as
    // past the land's edge, where a river runs out between the hills).
    const [sx, sy, sz] = viewShift(alpha, beta, ox, oy, this.shift);
    const [ex, ey, ez] = [tx + dx + sx, ty + dy + sy, tz + dz + sz];
    if (ey < this.ground(ex, ez)) return true;
    for (const [ax, az] of AROUND) if (ey < this.ground(ex + ax * clear, ez + az * clear)) return true;
    return false;
  }

  private startGlide(seconds: number): void {
    Object.assign(this.from, this.cur);
    this.t = 0;
    this.duration = Math.max(0, seconds);
    if (this.duration === 0) {
      Object.assign(this.cur, this.goal);
      this.cur.alpha += this.userAlpha;
      this.cur.radius /= this.userZoom;
      this.cur.tx += this.panX;
      this.cur.tz += this.panZ;
      if (this.onArrive) {
        const done = this.onArrive;
        this.onArrive = null;
        done();
      }
    }
  }

  /** Goal pose for a shot: the radius and view offset that fit its focus into its frame. */
  private fitGoal(shot: TownShot): void {
    const g = this.goal;
    const f = shot.focus;
    g.alpha = shot.alpha ?? this.cur.alpha;
    g.beta = shot.beta ?? 0.9;
    g.tx = f.x + f.w / 2;
    g.tz = f.z + f.d / 2;
    const W = this.viewW;
    const H = this.viewH;
    const fr = shot.frame ?? { left: 0, top: 0, right: W, bottom: H };
    // Frame in normalised device coordinates.
    const nl = (fr.left / W) * 2 - 1;
    const nr = (fr.right / W) * 2 - 1;
    const nt = 1 - (fr.top / H) * 2;
    const nb = 1 - (fr.bottom / H) * 2;
    const ncx = (nl + nr) / 2;
    const ncy = (nt + nb) / 2;
    const zoom = shot.zoom ?? 1;
    const hx = ((nr - nl) / 2) * 0.94 * zoom;
    const hy = ((nt - nb) / 2) * 0.94 * zoom;
    const tanY = Math.tan(this.camera.fov / 2);
    const tanX = tanY * (W / H);
    // Camera basis (left-handed, as game space is).
    const sa = Math.sin(g.alpha);
    const ca = Math.cos(g.alpha);
    const sb = Math.sin(g.beta);
    const cb = Math.cos(g.beta);
    const ex = ca * sb;
    const ey = cb;
    const ez = sa * sb;
    // forward = -e; right = normalize(cross(up, forward)); up' = cross(forward, right)
    const rl = Math.hypot(ez, ex) || 1;
    const rx = -ez / rl;
    const rz = ex / rl;
    const ux = -ey * rz;
    const uy = -ez * rx + ex * rz;
    const uz = ey * rx;
    const corners: [number, number, number][] = [];
    for (const y of [0, OVERVIEW_TOWN_HEIGHT]) for (const x of [f.x, f.x + f.w]) for (const z of [f.z, f.z + f.d]) corners.push([x - g.tx, y, z - g.tz]);
    const project = (r: number, ox: number, oy: number) => {
      let minX = Infinity;
      let maxX = -Infinity;
      let minY = Infinity;
      let maxY = -Infinity;
      for (const [px, py, pz] of corners) {
        const dx = px - ex * r;
        const dy = py - ey * r;
        const dz = pz - ez * r;
        const depth = -(dx * ex + dy * ey + dz * ez);
        const x = (dx * rx + dz * rz + ox) / (Math.max(0.1, depth) * tanX);
        const y = (dx * ux + dy * uy + dz * uz + oy) / (Math.max(0.1, depth) * tanY);
        minX = Math.min(minX, x);
        maxX = Math.max(maxX, x);
        minY = Math.min(minY, y);
        maxY = Math.max(maxY, y);
      }
      return { minX, maxX, minY, maxY };
    };
    const offsets = (r: number) => {
      let ox = ncx * r * tanX;
      let oy = ncy * r * tanY;
      for (let i = 0; i < 4; i++) {
        const b = project(r, ox, oy);
        ox += (ncx - (b.minX + b.maxX) / 2) * r * tanX;
        oy += (ncy - (b.minY + b.maxY) / 2) * r * tanY;
      }
      return [ox, oy] as const;
    };
    const fits = (r: number) => {
      const [ox, oy] = offsets(r);
      const b = project(r, ox, oy);
      return (b.maxX - b.minX) / 2 <= hx && (b.maxY - b.minY) / 2 <= hy;
    };
    let lo = 4;
    let hi = OVERVIEW_FARTHEST;
    for (let i = 0; i < 28; i++) {
      const mid = (lo + hi) / 2;
      if (fits(mid)) hi = mid;
      else lo = mid;
    }
    g.radius = hi;
    [g.ox, g.oy] = offsets(hi);
  }
}

/** What `areaFocus` reads of a world (`WorldStructure`). */
export interface AreaWorld {
  width: number;
  depth: number;
  storeyDepth?: number;
  plots: readonly { x: number; z: number; w: number; d: number }[];
  meta?: { areas?: readonly { x: number; z: number; w: number; d: number }[] } | null;
}

/** Land kept round an area's lots in its shot (m): the street past the last house, a field. */
const AREA_MARGIN = 16;

/**
 * The part of the world an area's shot frames: the lots whose middles lie in its box, with a margin; the box itself
 * while it has none (a village just placed); the whole land for an area the world doesn't have (a bare land).
 */
export function areaFocus(world: AreaWorld, area: number): { x: number; z: number; w: number; d: number } {
  const box = world.meta?.areas?.[area];
  const depth = groundDepth(world);
  if (!box) return { x: 0, z: 0, w: world.width, d: depth };
  let [x0, z0, x1, z1] = [Infinity, Infinity, -Infinity, -Infinity];
  for (const p of world.plots) {
    const cx = p.x + p.w / 2;
    const cz = p.z + p.d / 2;
    if (cx < box.x || cx > box.x + box.w || cz < box.z || cz > box.z + box.d) continue;
    x0 = Math.min(x0, p.x);
    z0 = Math.min(z0, p.z);
    x1 = Math.max(x1, p.x + p.w);
    z1 = Math.max(z1, p.z + p.d);
  }
  if (x0 > x1) [x0, z0, x1, z1] = [box.x, box.z, box.x + box.w, box.z + box.d];
  else [x0, z0, x1, z1] = [x0 - AREA_MARGIN, z0 - AREA_MARGIN, x1 + AREA_MARGIN, z1 + AREA_MARGIN];
  // (Within the land.)
  x0 = Math.max(0, x0);
  z0 = Math.max(0, z0);
  x1 = Math.min(world.width, x1);
  z1 = Math.min(depth, z1);
  return { x: x0, z: z0, w: Math.max(1, x1 - x0), d: Math.max(1, z1 - z0) };
}
