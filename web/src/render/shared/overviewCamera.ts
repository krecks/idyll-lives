/**
 * The menus' town overview camera: frames a shot's rectangle into its part of the screen, glides
 * between shots, drifts and sways, takes the player's orbit and zoom on top, and keeps above the
 * land. Engine-neutral: it writes an orbit pose (the game's orbit conventions, `shared/orbit.ts`,
 * with a view offset that moves the picture by `targetScreenOffset` view units) into any camera
 * with those fields.
 */

import type { TownShot } from '../types';
import { angleDelta, ease } from './angles';

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
  private aspect = 16 / 9;
  private viewW = 1920;
  private viewH = 1080;
  private onArrive: (() => void) | null = null;

  /** Height of the land round the town (null: flat), so the camera stays above it with the town in sight. */
  ground: ((x: number, z: number) => number) | null = null;

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

  /** Frames `shot`; with `duration` 0 (or `cut`) it jumps there. */
  setShot(shot: TownShot, cut = false): void {
    const alphaChanged = shot.alpha !== undefined && shot.alpha !== this.shot?.alpha;
    if (alphaChanged) this.userAlpha = 0;
    if (shot.zoom !== this.shot?.zoom || alphaChanged) this.userZoom = 1;
    this.shot = shot;
    this.drift = shot.drift ?? 0;
    this.sway = shot.sway ?? 0;
    this.swayTime = 0;
    this.fitGoal(shot);
    this.swayBase = this.goal.alpha;
    this.startGlide(cut ? 0 : (shot.duration ?? 1.2));
  }

  /** Glides to an exact camera pose (the game's view of a lot) and calls back on arrival. */
  glideTo(target: { alpha: number; beta: number; radius: number; x: number; z: number }, seconds: number, done: () => void): void {
    this.shot = null;
    this.drift = 0;
    this.sway = 0;
    this.userAlpha = 0;
    this.userZoom = 1;
    Object.assign(this.goal, { alpha: target.alpha, beta: target.beta, radius: target.radius, tx: target.x, tz: target.z, ox: 0, oy: 0 });
    this.onArrive = done;
    this.startGlide(seconds);
  }

  nudge(dAlpha: number, zoom: number): void {
    this.userAlpha += dAlpha;
    this.userZoom = Math.min(2.6, Math.max(0.6, this.userZoom * zoom));
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
    if (this.t < this.duration) {
      this.t = Math.min(this.duration, this.t + dt);
      const k = ease(this.t / this.duration);
      const f = this.from;
      c.alpha = f.alpha + angleDelta(f.alpha, ga) * k;
      c.beta = f.beta + (g.beta - f.beta) * k;
      // Zooms glide in log space (a long pull-back doesn't rush at the start).
      c.radius = Math.exp(Math.log(f.radius) + (Math.log(g.radius / zoom) - Math.log(f.radius)) * k);
      c.tx = f.tx + (g.tx - f.tx) * k;
      c.tz = f.tz + (g.tz - f.tz) * k;
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
      c.beta += (g.beta - c.beta) * k;
      c.radius += (g.radius / zoom - c.radius) * k;
      c.tx += (g.tx - c.tx) * k;
      c.tz += (g.tz - c.tz) * k;
      c.ox += ((g.ox * c.radius) / g.radius - c.ox) * k;
      c.oy += ((g.oy * c.radius) / g.radius - c.oy) * k;
      // Within a hair of the goal: there (a camera that keeps creeping is never still: held frames).
      if (Math.abs(angleDelta(c.alpha, ga)) + Math.abs(g.beta - c.beta) < SNAP && Math.abs(g.radius / zoom - c.radius) + Math.abs(g.tx - c.tx) + Math.abs(g.tz - c.tz) < SNAP * c.radius) {
        c.alpha = ga;
        c.beta = g.beta;
        c.radius = g.radius / zoom;
        c.tx = g.tx;
        c.tz = g.tz;
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
    // Looking from higher up while the land (a mountain) would stand in the way.
    let beta = c.beta;
    for (let k = 0; k < 20 && beta > 0.2 && this.hidden(c.alpha, beta, c.radius, c.tx, c.tz); k++) beta -= 0.04;
    cam.beta = beta;
    cam.radius = c.radius;
    cam.target.x = c.tx;
    cam.target.y = 0;
    cam.target.z = c.tz;
    cam.targetScreenOffset.x = c.ox;
    cam.targetScreenOffset.y = c.oy;
  }

  /** Whether the camera at this pose is in the ground or has the land between it and the target. */
  private hidden(alpha: number, beta: number, radius: number, tx: number, tz: number): boolean {
    if (!this.ground) return false;
    const [dx, dy, dz] = [Math.cos(alpha) * Math.sin(beta) * radius, Math.cos(beta) * radius, Math.sin(alpha) * Math.sin(beta) * radius];
    for (const t of [0, 0.12, 0.25, 0.4, 0.55, 0.7]) {
      const [x, y, z] = [tx + dx * (1 - t), dy * (1 - t), tz + dz * (1 - t)];
      // Clear of the treetops too.
      if (y < this.ground(x, z) + (t === 0 ? 12 : 14)) return true;
    }
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
    let hi = 900;
    for (let i = 0; i < 28; i++) {
      const mid = (lo + hi) / 2;
      if (fits(mid)) hi = mid;
      else lo = mid;
    }
    g.radius = hi;
    [g.ox, g.oy] = offsets(hi);
  }
}
