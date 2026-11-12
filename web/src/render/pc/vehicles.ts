/**
 * What the cars draw with on PlayCanvas (`cars.ts`): which of them are drawn
 * (near, streamed in, not hidden: `hideOn`, `setView`), whether one moving draws the sun's map
 * again (`setShadowTest`), how their wheels turn and steer as they go (`rollWheels`), and every one's
 * wheels in one instanced draw and their soft contact shadows in another.
 */

import * as pc from './engine';
import { Instances } from './instancing';
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

  /** A drawn vehicle's four wheels (`wide`: their width, as a wheel's) and its contact shadow, at `slot` in their draws. */
  protected writeAt(slot: number, at: Spot, size: { wid: number; len: number; wheelR: number }, four: readonly Wheel[], v: Rolling, wide = 1): void {
    const { m, q } = this;
    const s = Math.sin(at.yaw);
    const c = Math.cos(at.yaw);
    const wheels = this.wheels;
    if (wheels) {
      const r = size.wheelR;
      for (let w = 0; w < 4; w++) {
        const { side, x, z, front } = four[w];
        // The left ones turned round (their hubs face out), turning the other way for it.
        const yaw = at.yaw + (front ? v.steer : 0) + (side < 0 ? Math.PI : 0);
        const spin = side < 0 ? -v.spin : v.spin;
        // Turned about y (`yaw`), then about x (`spin`), scaled by its radius across y and z.
        compose(wide, r, r, quatYawPitchRoll(yaw, spin, 0, q), at.x + x * c + z * s, r + LIFT, at.z - x * s + z * c, m);
        wheels.setMatrix(slot * 4 + w, m);
      }
    }
    // (A blob of radius 1, stretched to its size.)
    this.shadowDraw?.setMatrix(slot, compose(size.wid * 0.62, 1, size.len * 0.56, quatYawPitchRoll(at.yaw, 0, 0, q), at.x, 0, at.z, m));
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

  /** An instanced draw of `mesh` under the root, out of the house's lamps. */
  protected instanced(name: string, mesh: pc.Mesh, material: pc.Material, castShadows: boolean, capacity: number): Instances {
    const draw = new Instances(this.app.graphicsDevice, this.root, mesh, material, { name, castShadows, capacity: Math.max(1, capacity) });
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
