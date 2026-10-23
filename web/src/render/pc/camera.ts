/**
 * The game camera as PlayCanvas draws with it: an orbit pose in game space (Babylon's orbit
 * conventions: the eye at `target + radius * (cos α sin β, cos β, sin α sin β)`, y up) placed on a
 * PlayCanvas camera through `space.ts`, and points projected back onto the screen.
 */

import * as pc from './engine';
import { toEngine } from './space';

export interface Lens {
  /** Vertical field of view (radians), near and far planes (m). */
  fov: number;
  near: number;
  far: number;
}

export class GameCamera {
  readonly entity: pc.Entity;
  private readonly eye = new pc.Vec3();
  private readonly aim = new pc.Vec3();
  private readonly view = new pc.Mat4();
  private readonly viewProj = new pc.Mat4();
  private readonly clip = new pc.Vec4();

  constructor(parent: pc.Entity, lens: Lens, name = 'camera') {
    this.entity = new pc.Entity(name);
    this.entity.addComponent('camera', { clearColor: new pc.Color(0, 0, 0, 1) });
    parent.addChild(this.entity);
    this.setLens(lens);
  }

  get camera(): pc.CameraComponent {
    return this.entity.camera!;
  }

  setLens(lens: Lens): void {
    const c = this.camera;
    c.fov = (lens.fov * 180) / Math.PI;
    c.nearClip = lens.near;
    c.farClip = lens.far;
  }

  /**
   * Places the camera at the orbit pose (game space): looking at `target` from `alpha`, `beta`,
   * `radius`, the picture moved by (`ox`, `oy`) view units (Babylon's `targetScreenOffset`: the
   * camera and its target both slid along the view's right and up).
   */
  setPose(alpha: number, beta: number, radius: number, tx: number, ty: number, tz: number, ox = 0, oy = 0): void {
    const sa = Math.sin(alpha);
    const ca = Math.cos(alpha);
    const sb = Math.sin(beta);
    const cb = Math.cos(beta);
    let ex = tx + radius * ca * sb;
    let ey = ty + radius * cb;
    let ez = tz + radius * sa * sb;
    if (ox !== 0 || oy !== 0) {
      // The view's right and up in game space (Babylon's left-handed look-at, up y).
      const fx = -ca * sb;
      const fy = -cb;
      const fz = -sa * sb;
      const rl = Math.hypot(fz, fx) || 1;
      const rx = fz / rl;
      const rz = -fx / rl;
      const ux = fy * rz;
      const uy = fz * rx - fx * rz;
      const uz = -fy * rx;
      const dx = -(ox * rx + oy * ux);
      const dy = -(oy * uy);
      const dz = -(ox * rz + oy * uz);
      ex += dx;
      ey += dy;
      ez += dz;
      tx += dx;
      ty += dy;
      tz += dz;
    }
    toEngine(ex, ey, ez, this.eye);
    toEngine(tx, ty, tz, this.aim);
    this.entity.setPosition(this.eye);
    this.entity.lookAt(this.aim, pc.Vec3.UP);
  }

  /** The camera's eye in game space. */
  eyeInGame(out: { x: number; y: number; z: number }): { x: number; y: number; z: number } {
    const p = this.entity.getPosition();
    out.x = p.x;
    out.y = p.y;
    out.z = -p.z;
    return out;
  }

  /**
   * Where the game-space point (x, y, z) shows on a canvas `width` × `height` CSS pixels: false when
   * it is behind the camera, past its far plane or not a point (as Babylon's `Vector3.Project`).
   */
  project(x: number, y: number, z: number, width: number, height: number, out: { x: number; y: number }): boolean {
    // (From the entity as it stands now: the camera's own view matrix is brought up to date when it draws.)
    this.view.copy(this.entity.getWorldTransform()).invert();
    const vp = this.viewProj.mul2(this.camera.camera.projectionMatrix, this.view).data;
    // (Game space to PlayCanvas: z turned round.)
    const wz = -z;
    const c = this.clip;
    c.x = vp[0] * x + vp[4] * y + vp[8] * wz + vp[12];
    c.y = vp[1] * x + vp[5] * y + vp[9] * wz + vp[13];
    c.z = vp[2] * x + vp[6] * y + vp[10] * wz + vp[14];
    c.w = vp[3] * x + vp[7] * y + vp[11] * wz + vp[15];
    if (!(c.w > 0)) return false;
    const nz = c.z / c.w;
    const sx = ((c.x / c.w + 1) / 2) * width;
    const sy = ((1 - c.y / c.w) / 2) * height;
    if (!(nz >= -1 && nz <= 1 && Number.isFinite(sx) && Number.isFinite(sy))) return false;
    out.x = sx;
    out.y = sy;
    return true;
  }
}
