/**
 * What the cars draw with on PlayCanvas (`cars.ts`): which of them are drawn
 * (near, streamed in, not hidden: `hideOn`, `setView`), whether one moving draws the sun's map
 * again (`setShadowTest`), how their wheels turn and steer as they go (`rollWheels`), and every one's
 * wheels in one instanced draw and their soft contact shadows in another.
 */

import * as pc from './engine';
import { Instances, type InstanceStream } from './instancing';
import { MaterialLibrary } from './materials';
import { meshFrom } from './meshes';
import { paintWheel } from '../shared/cars';
import { chunkAt } from '../shared/chunks';
import { wrapAngle } from '../shared/characters/rig';
import { Geo } from '../shared/geometry';
import { compose, quatYawPitchRoll, type Quat } from '../shared/math';
import { blob, Painter } from '../shared/street';

/** Vehicles further than this from the camera's target aren't drawn (m): past the fog, a dot at most. */
const FAR = 650;
/**
 * Further than this in a frame (m), a vehicle was put there (a load, back into town), not driven: its wheels don't
 * turn for it. At least `JUMP`; at speed, as far as `FASTEST` m a second of drive time goes in the frame (the layout's
 * `driveScale` times the game speed's playback rate, `jumpOver`).
 */
const JUMP = 10;
const FASTEST = 30;

/** `JUMP` for a frame `real` seconds long, `drive` seconds of drive time a real second (`driveScale` × playback rate). */
export function jumpOver(real: number, drive: number): number {
  return Math.max(JUMP, FASTEST * drive * real);
}
/** Height of the body and wheels over the ground (as the street's `paintCar`). */
export const LIFT = 0.015;

/** Where a vehicle is drawn (`shown`: 0 out of town or not drawn at all). */
export interface Spot {
  x: number;
  z: number;
  yaw: number;
  shown: number;
}

/** A wheel: which side (+1: +x), where (m from the middle), and whether it steers. */
export interface Wheel {
  side: 1 | -1;
  x: number;
  z: number;
  front: boolean;
}

/** Where a vehicle stands on the ground (`Vehicles.standing`), and where it stood and when it was worked out. */
export interface Stand {
  y: number;
  pitch: number;
  x: number;
  z: number;
  yaw: number;
  wheelBase: number;
  stamp: number;
}

/** A `Stand` not worked out yet. */
export const newStand = (): Stand => ({ y: 0, pitch: 0, x: NaN, z: NaN, yaw: NaN, wheelBase: NaN, stamp: -1 });

/** The wheels' turn (rad, about their axle) and the front ones' steer (rad, +: towards +x). */
export interface Rolling {
  spin: number;
  steer: number;
}

/** How the front wheels steer: at most (rad), and how quickly they follow the way it turns (per second). */
export interface Steering {
  max: number;
  rate: number;
}

/**
 * A vehicle moved from `last` to `at` in `real` seconds (`drive` seconds of drive time each: `jumpOver`): its wheels
 * turn by the way it went, the front ones steer by how it turned. The way it went along itself (m), or null when it
 * was put there (further than `jumpOver`).
 */
export function rollWheels(v: Rolling, at: Spot, last: Pick<Spot, 'x' | 'z' | 'yaw'>, wheelR: number, wheelBase: number, steer: Steering, real: number, drive = 1): number | null {
  const dx = at.x - last.x;
  const dz = at.z - last.z;
  if (Math.hypot(dx, dz) > jumpOver(real, drive)) return null;
  const along = dx * Math.sin(at.yaw) + dz * Math.cos(at.yaw);
  v.spin = (v.spin + along / wheelR) % (Math.PI * 2);
  // Turning by `turn` over `along` metres: a circle the front wheels steer round (a bicycle's, the wheelbase long).
  if (Math.abs(along) > 1e-3 && real > 0) {
    const turn = wrapAngle(at.yaw - last.yaw);
    const goal = Math.max(-steer.max, Math.min(steer.max, Math.atan((turn * wheelBase) / along)));
    v.steer += (goal - v.steer) * (1 - Math.exp(-real * steer.rate));
  }
  return along;
}

export abstract class Vehicles {
  protected readonly lib: MaterialLibrary;
  protected readonly root: pc.Entity;
  /** Meshes by key, the wheel and the contact shadow among them (they go with the build). */
  protected readonly meshes = new Map<string, pc.Mesh>();
  /** Every drawn vehicle's wheels; its contact shadow. */
  protected wheels: Instances | null = null;
  protected shadowDraw: Instances | null = null;
  /** Whether one moved in the last frame where it casts a shadow into the sun's map (the map is to be drawn again). */
  moved = false;

  /**
   * Adds to `out` the draws of what drives (the wheels: every drawn vehicle's in one draw, whose box grows and shrinks
   * as any of them goes), drawn again at every redraw of the sun's map over the still casters' picture
   * (`SunShadows.movers`): one driving anywhere doesn't make that picture be drawn again.
   */
  movingCasters(out: pc.MeshInstance[]): void {
    const wheels = this.wheels;
    if (this.shadows && wheels?.enabled) out.push(wheels.meshInstance);
  }
  /** Those standing in here (x0, z0, x1, z1) aren't drawn (`hideOn`). */
  protected hidden: readonly [number, number, number, number] | null = null;
  /** The chunks streamed in (null: all of them) and the camera's target: those outside them, or further than `FAR` from it, aren't drawn (`setView`). */
  protected chunks: ReadonlySet<number> | null = null;
  protected eye: { x: number; z: number } | null = null;
  /** Whether one at (x, z) could cast into the sun's map (`SunShadows.reaches`; null: anywhere): only one moving there draws it again. */
  protected inMap: ((x: number, z: number) => boolean) | null = null;
  /** Scratch: a matrix (game space), and a turn. */
  protected readonly m = new Float32Array(16);
  protected readonly q: Quat = [0, 0, 0, 1];

  constructor(
    protected readonly app: pc.AppBase,
    parent: pc.Entity,
    protected readonly shadows: boolean,
    name: string,
  ) {
    this.lib = MaterialLibrary.for(app);
    this.root = new pc.Entity(name);
    parent.addChild(this.root);
  }

  /** Those standing in `rect` (x0, z0, x1, z1) aren't drawn (the viewed lot while its basement is in view, as its ground is), or none are hidden (null). */
  hideOn(rect: readonly [number, number, number, number] | null): void {
    this.hidden = rect;
  }

  /** The chunks streamed in (null: the whole town is drawn) and the camera's target (x, z): those outside the chunks, or further than `FAR` from the target, aren't drawn. */
  setView(chunks: ReadonlySet<number> | null, x: number, z: number): void {
    this.chunks = chunks;
    this.eye = { x, z };
  }

  /** Whether a spot can cast into the sun's map (`SunShadows.reaches`; null: anywhere): one moving elsewhere doesn't draw it again. */
  setShadowTest(inMap: ((x: number, z: number) => boolean) | null): void {
    this.inMap = inMap;
  }

  /** The ground they stand on (game space, m: `BuiltSurface.at` on a land the town follows), or 0 everywhere (null). */
  setGround(ground: ((x: number, z: number) => number) | null): void {
    this.ground = ground;
  }
  protected ground: ((x: number, z: number) => number) | null = null;

  /**
   * Where a vehicle at `at`, `wheelBase` m between its axles, stands: the ground under its middle (`y`) and its pitch
   * from the ground under its two axles (`pitch`, rad, nose up), into `out` (0 and 0 without a ground). Worked out once
   * an update (`stamp`) while it stays put: placing its body and writing its contact shadow ask it the same frame, and
   * each look at the ground walks the roads round it.
   */
  protected standing(at: { x: number; z: number; yaw: number }, wheelBase: number, out: Stand): Stand {
    if (out.stamp === this.stamp && out.x === at.x && out.z === at.z && out.yaw === at.yaw && out.wheelBase === wheelBase) return out;
    out.stamp = this.stamp;
    out.x = at.x;
    out.z = at.z;
    out.yaw = at.yaw;
    out.wheelBase = wheelBase;
    const g = this.ground;
    if (!g) {
      out.y = 0;
      out.pitch = 0;
      return out;
    }
    // (Its nose along (sin yaw, cos yaw).)
    const fx = Math.sin(at.yaw) * (wheelBase / 2);
    const fz = Math.cos(at.yaw) * (wheelBase / 2);
    const front = g(at.x + fx, at.z + fz);
    const back = g(at.x - fx, at.z - fz);
    out.y = (front + back) / 2;
    out.pitch = wheelBase > 0 ? Math.atan2(front - back, wheelBase) : 0;
    return out;
  }
  /** Moved on at each update: a `Stand` from an update before is worked out again (the land may have changed under it). */
  protected stamp = 0;

  abstract clear(): void;

  destroy(): void {
    this.clear();
    this.root.destroy();
  }

  /** Whether one at (x, z) can cast into the sun's map (`setShadowTest`). */
  protected inShadows(x: number, z: number): boolean {
    return !this.inMap || this.inMap(x, z);
  }

  /** Whether one is drawn: in town, near, and not standing where they're hidden (`hideOn`). */
  protected visible(at: Spot): boolean {
    if (!at.shown) return false;
    const r = this.hidden;
    if (r && at.x > r[0] && at.x < r[2] && at.z > r[1] && at.z < r[3]) return false;
    if (this.chunks && !this.chunks.has(chunkAt(at.x, at.z))) return false;
    const e = this.eye;
    return !e || Math.hypot(at.x - e.x, at.z - e.z) <= FAR;
  }

  /** The wheels' and contact shadows' draws (`name`Wheels, `name`Shadows), made if they aren't: room for `most` vehicles, their shadows blobs with an edge `edge` soft. */
  protected ensureAround(name: string, most: number, edge: number): void {
    if (!this.wheels) {
      const wheel = this.mesh('wheel', () => {
        const p = new Painter(new Geo());
        paintWheel(p);
        return p.geo;
      });
      if (wheel) this.wheels = this.instanced(`${name}Wheels`, wheel, this.lib.plainFinish(), this.shadows, most * 4);
    }
    if (!this.shadowDraw) {
      // (A blob of radius 1, stretched to each one's size.)
      const shadow = this.mesh('shadow', () => {
        const g = new Geo().color('#000000');
        blob(g, 0, 0, 1, 1, 0, edge, 0.03);
        return g;
      });
      if (shadow) this.shadowDraw = this.instanced(`${name}Shadows`, shadow, this.lib.contactShadow(false), false, most);
    }
  }

  /** A drawn vehicle's four wheels (`wide`: their width, as a wheel's) and its contact shadow, at `slot` in their draws, standing as `stand` (`standing`). */
  protected writeAt(slot: number, at: Spot, size: { wid: number; len: number; wheelR: number }, four: readonly Wheel[], v: Rolling, stand: Stand, wide = 1): void {
    const { m, q } = this;
    const s = Math.sin(at.yaw);
    const c = Math.cos(at.yaw);
    const wheels = this.wheels;
    // (On a land the town follows, each wheel on the ground under it, the shadow under the body's middle, pitched as it.)
    const g = this.ground;
    if (wheels) {
      const r = size.wheelR;
      for (let w = 0; w < 4; w++) {
        const { side, x, z, front } = four[w];
        // The left ones turned round (their hubs face out), turning the other way for it.
        const yaw = at.yaw + (front ? v.steer : 0) + (side < 0 ? Math.PI : 0);
        const spin = side < 0 ? -v.spin : v.spin;
        // Turned about y (`yaw`), then about x (`spin`), scaled by its radius across y and z.
        const [wx, wz] = [at.x + x * c + z * s, at.z - x * s + z * c];
        compose(wide, r, r, quatYawPitchRoll(yaw, spin, 0, q), wx, r + LIFT + (g ? g(wx, wz) : 0), wz, m);
        wheels.setMatrix(slot * 4 + w, m);
      }
    }
    // (A blob of radius 1, stretched to its size.)
    this.shadowDraw?.setMatrix(slot, compose(size.wid * 0.62, 1, size.len * 0.56, quatYawPitchRoll(at.yaw, -stand.pitch, 0, q), at.x, stand.y, at.z, m));
  }

  /** The wheels and contact shadows written (`writeAt`) sent to the GPU, drawn while there are any. */
  protected sendAround(): void {
    const { wheels, shadowDraw: shadows } = this;
    if (wheels) {
      wheels.update();
      wheels.enabled = wheels.count > 0;
    }
    if (shadows) {
      shadows.update();
      shadows.enabled = shadows.count > 0;
    }
  }

  /** The wheels' and contact shadows' draws let go, and the meshes nothing holds. */
  protected clearDraws(): void {
    this.wheels?.destroy();
    this.shadowDraw?.destroy();
    this.wheels = this.shadowDraw = null;
    // (The instanced draws hold their meshes without taking them; the entities' are let go as they went.)
    for (const m of this.meshes.values()) if (m.refCount === 0) m.destroy();
    this.meshes.clear();
  }

  /**
   * An instanced draw of `mesh` under the root, out of the house's lamps (`streams` beside each copy's matrix,
   * `castKeyed`: `instancing.ts`).
   */
  protected instanced(name: string, mesh: pc.Mesh, material: pc.Material, castShadows: boolean, capacity: number, streams: readonly InstanceStream[] = [], castKeyed = false): Instances {
    const draw = new Instances(this.app.graphicsDevice, this.root, mesh, material, { name, castShadows, capacity: Math.max(1, capacity), streams, castKeyed });
    this.lib.excludeFromLights([], [draw.meshInstance]);
    draw.enabled = false;
    return draw;
  }

  protected mesh(key: string, make: () => Geo): pc.Mesh | null {
    let m = this.meshes.get(key);
    if (!m) {
      const made = meshFrom(this.app.graphicsDevice, make().meshData({ colors: true }));
      if (!made) return null;
      this.meshes.set(key, (m = made));
    }
    return m;
  }
}
