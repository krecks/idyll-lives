/**
 * The town's buses in the game on PlayCanvas (sim-core `bus.rs`): placed every frame where the
 * sim's snapshot says each is (`SnapshotLayout.bus`), between two snapshots (and on past the
 * newest while it drives, as the cars go: `FrameState.alpha`); hidden outside the chunks streamed
 * in, far off (`setView`), and on the viewed lot with its basement in view (`hideOn`).
 *
 * A bus is an entity of its own (`shared/bus.ts` `paintBus`): its body in one piece while its
 * doors are shut (one draw); at a stop, the body with its doorways open and the doors' four glass
 * leaves sliding apart (`DOOR_SECONDS` in game time), shut again before it drives off. Every bus's
 * wheels are one instanced draw (they turn as it goes and the front ones steer), their soft
 * contact shadows another. Riders aren't drawn in it: on board, they're out of sight.
 */

import * as pc from './engine';
import { Instances } from './instancing';
import { MaterialLibrary } from './materials';
import { drawMesh, meshFrom } from './meshes';
import type { FrameState } from '../../core/bridge';
import { BUS, BUS_DOORS, busLeaf, busWheelAt, paintBus } from '../shared/bus';
import { paintWheel } from '../shared/cars';
import { chunkAt } from '../shared/chunks';
import { lerpAngle, wrapAngle } from '../shared/characters/rig';
import { ease } from '../shared/doors';
import { Geo } from '../shared/geometry';
import { GAME_RATE } from '../shared/gameSpeed';
import { compose, quatYawPitchRoll, type Quat } from '../shared/math';
import { blob, Painter } from '../shared/street';

/** Buses further than this from the camera's target aren't drawn (m). */
const FAR = 650;
/** How long a door takes to open or shut (s of game time at 1×: the sim's doors open about 12 ticks after it stops, `bus::DOOR_TICKS`). */
const DOOR_SECONDS = 0.55;
/** How far a door's leaves slide apart (a share of a leaf's width), and out from the side (m). */
const SLIDE = 0.92;
const OUT = 0.05;
/** How far the front wheels steer at most (rad) and how quickly (per second); further than this in a frame (m), it was put there. */
const STEER_MAX = 0.45;
const STEER_RATE = 6;
const JUMP = 10;
/** Height of the body over the ground. */
const LIFT = 0.015;
/** The bus's four wheels (`busWheelAt`). */
const WHEELS = [0, 1, 2, 3].map(busWheelAt);
/** Most buses (sim-core `bus::MAX_BUSES`). */
const MOST = 128;

interface Drawn {
  id: number;
  line: number;
  entity: pc.Entity;
  /** Its body in one piece (doors shut), and apart (doorways open) with the leaves. */
  whole: pc.Entity;
  apart: pc.Entity;
  leaves: pc.Entity[];
  at: { x: number; z: number; yaw: number; shown: number };
  last: { x: number; z: number; yaw: number };
  /** How far open its doors are (0..1), the wheels' turn and steer. */
  open: number;
  spin: number;
  steer: number;
  /** It was in the newest snapshot (else it's gone: let go). */
  seen: boolean;
  drawn: boolean;
}

export class Buses {
  private readonly lib: MaterialLibrary;
  private readonly root: pc.Entity;
  private readonly buses = new Map<number, Drawn>();
  /** Buses gone out of town, kept for the next of their line (their meshes stay held). */
  private readonly spare: Drawn[] = [];
  private readonly meshes = new Map<string, pc.Mesh>();
  private wheels: Instances | null = null;
  private shadowDraw: Instances | null = null;
  private lastNow = 0;
  /** Whether a bus moved in the last frame where it casts into the sun's map. */
  moved = false;
  private hidden: readonly [number, number, number, number] | null = null;
  private chunks: ReadonlySet<number> | null = null;
  private eye: { x: number; z: number } | null = null;
  private inMap: ((x: number, z: number) => boolean) | null = null;
  private readonly m = new Float32Array(16);
  private readonly q: Quat = [0, 0, 0, 1];

  constructor(
    private readonly app: pc.AppBase,
    parent: pc.Entity,
    private readonly shadows: boolean,
  ) {
    this.lib = MaterialLibrary.for(app);
    this.root = new pc.Entity('buses');
    parent.addChild(this.root);
  }

  /** Each bus where the snapshot has it (between the last two), its doors and wheels. */
  update(frame: FrameState): void {
    const { prev, curr, alpha, layout } = frame;
    const f = layout.bus;
    const base = layout.busBase;
    const stride = layout.busStride;
    const countAt = layout.header.busCount;
    const rate = GAME_RATE[curr[layout.header.speed] | 0] ?? 1;
    const real = Math.min(0.1, Math.max(0, (frame.now - this.lastNow) / 1000));
    this.lastNow = frame.now;
    this.moved = false;
    if (!f || base === undefined || stride === undefined || countAt === undefined) return;
    const count = Math.min(MOST, curr[countAt] | 0);
    if (count === 0 && this.buses.size === 0) return;
    // Where each was in the snapshot before, by which bus it is (its row may have moved up).
    const before = new Map<number, number>();
    for (let k = 0; k < Math.min(MOST, prev[countAt] | 0); k++) before.set(prev[base + k * stride + f.id], base + k * stride);
    for (const b of this.buses.values()) b.seen = false;
    let changed = false;
    for (let k = 0; k < count; k++) {
      const o = base + k * stride;
      const shown = curr[o + f.shown];
      if (!shown) continue;
      const id = curr[o + f.id];
      const bus = this.buses.get(id) ?? this.make(id, curr[o + f.line] | 0, curr[o + f.x], curr[o + f.z], curr[o + f.yaw]);
      bus.seen = true;
      const p = before.get(id);
      // Driving in both, on past the newest snapshot while the next is late; standing, not past it.
      const was = p === undefined ? 0 : prev[p + f.shown];
      const t = !was ? 1 : shown === 2 && was === 2 ? Math.max(0, alpha) : Math.min(1, Math.max(0, alpha));
      const [px, pz, pyaw] = p === undefined ? [curr[o + f.x], curr[o + f.z], curr[o + f.yaw]] : [prev[p + f.x], prev[p + f.z], prev[p + f.yaw]];
      bus.at.x = px + (curr[o + f.x] - px) * t;
      bus.at.z = pz + (curr[o + f.z] - pz) * t;
      bus.at.yaw = lerpAngle(pyaw, curr[o + f.yaw], t);
      bus.at.shown = shown;
      // Its doors open while the sim has them open, sliding in game time.
      const target = (curr[o + f.doors] | 0) !== 0 ? 1 : 0;
      if (bus.open !== target && rate > 0) {
        const step = (real * rate) / DOOR_SECONDS;
        bus.open = target > bus.open ? Math.min(1, bus.open + step) : Math.max(0, bus.open - step);
      }
      if (this.place(bus, real)) changed = true;
    }
    // Gone out of town (or not in the snapshot): put by for the next one.
    for (const [id, b] of this.buses) {
      if (b.seen) continue;
      b.entity.enabled = false;
      b.drawn = false;
      this.buses.delete(id);
      this.spare.push(b);
      changed = true;
    }
    if (changed) this.writeAround();
  }

  /** Buses standing in `rect` (x0, z0, x1, z1) aren't drawn (the viewed lot while its basement is in view), or none are hidden (null). */
  hideOn(rect: readonly [number, number, number, number] | null): void {
    this.hidden = rect;
  }

  /** The chunks streamed in (null: the whole town is drawn) and the camera's target: buses outside them, or further than `FAR`, aren't drawn. */
  setView(chunks: ReadonlySet<number> | null, x: number, z: number): void {
    this.chunks = chunks;
    this.eye = { x, z };
  }

  /** Whether a spot can cast into the sun's map (`SunShadows.reaches`; null: anywhere). */
  setShadowTest(inMap: ((x: number, z: number) => boolean) | null): void {
    this.inMap = inMap;
  }

  clear(): void {
    for (const b of [...this.buses.values(), ...this.spare]) b.entity.destroy();
    this.buses.clear();
    this.spare.length = 0;
    this.wheels?.destroy();
    this.shadowDraw?.destroy();
    this.wheels = this.shadowDraw = null;
    for (const m of this.meshes.values()) if (m.refCount === 0) m.destroy();
    this.meshes.clear();
  }

  destroy(): void {
    this.clear();
    this.root.destroy();
  }

  /** A bus's entity (the first time it's in the snapshot): one of its line's put by, else a new one, its body whole and apart, its doors' leaves. */
  private make(id: number, line: number, x: number, z: number, yaw: number): Drawn {
    const k = this.spare.findIndex((b) => b.line === line);
    if (k >= 0) {
      const bus = this.spare.splice(k, 1)[0];
      Object.assign(bus, { id, at: { x, z, yaw, shown: 0 }, last: { x, z, yaw }, open: 0, spin: 0, steer: 0, seen: true, drawn: false });
      bus.whole.enabled = true;
      bus.apart.enabled = false;
      this.buses.set(id, bus);
      return bus;
    }
    const entity = new pc.Entity('bus');
    this.root.addChild(entity);
    const whole = new pc.Entity('busWhole');
    const apart = new pc.Entity('busApart');
    entity.addChild(whole);
    entity.addChild(apart);
    apart.enabled = false;
    const body = (open: boolean) =>
      this.mesh(`body ${line} ${open}`, () => {
        const p = new Painter(new Geo());
        paintBus(p.at(0, 0, 0, LIFT), line, open);
        return p.geo;
      });
    const one = body(false);
    if (one) drawMesh(whole, 'busBody', one, this.lib.plainFinish(), { castShadows: this.shadows });
    const bare = body(true);
    if (bare) drawMesh(apart, 'busBody', bare, this.lib.plainFinish(), { castShadows: this.shadows });
    const leaves: pc.Entity[] = [];
    BUS_DOORS.forEach((_, d) => {
      const mesh = this.mesh(`leaf ${d}`, () => {
        const p = new Painter(new Geo());
        busLeaf(p.at(0, 0, 0, LIFT), d);
        return p.geo;
      });
      for (let k = 0; k < 2; k++) {
        const leaf = new pc.Entity('busDoor');
        apart.addChild(leaf);
        if (mesh) drawMesh(leaf, 'busDoorLeaf', mesh, this.lib.plainFinish(), { castShadows: this.shadows });
        leaves.push(leaf);
      }
    });
    for (const r of entity.findComponents('render') as pc.RenderComponent[]) this.lib.excludeFromLights([], r.meshInstances);
    this.ensureDraws();
    const bus: Drawn = { id, line, entity, whole, apart, leaves, at: { x, z, yaw, shown: 0 }, last: { x, z, yaw }, open: 0, spin: 0, steer: 0, seen: true, drawn: false };
    this.buses.set(id, bus);
    return bus;
  }

  /** A bus placed this frame (`real` seconds since the last): drawn or not, turned, its wheels rolled, its doors slid. Whether it changed what the wheels and shadows draw show. */
  private place(bus: Drawn, real: number): boolean {
    const { at, last } = bus;
    const drawn = this.visible(at);
    let changed = drawn !== bus.drawn;
    bus.drawn = drawn;
    if (bus.entity.enabled !== drawn) bus.entity.enabled = drawn;
    const [dx, dz] = [at.x - last.x, at.z - last.z];
    const moved = dx !== 0 || dz !== 0 || at.yaw !== last.yaw;
    if (moved) {
      changed = true;
      if (Math.hypot(dx, dz) <= JUMP) {
        const along = dx * Math.sin(at.yaw) + dz * Math.cos(at.yaw);
        bus.spin = (bus.spin + along / BUS.wheelR) % (Math.PI * 2);
        const turn = wrapAngle(at.yaw - last.yaw);
        if (Math.abs(along) > 1e-3 && real > 0) {
          const wheelBase = BUS.axles[0] - BUS.axles[1];
          const goal = Math.max(-STEER_MAX, Math.min(STEER_MAX, Math.atan((turn * wheelBase) / along)));
          bus.steer += (goal - bus.steer) * (1 - Math.exp(-real * STEER_RATE));
        }
      }
      [last.x, last.z, last.yaw] = [at.x, at.z, at.yaw];
      if (this.shadows && (!this.inMap || this.inMap(at.x, at.z))) this.moved = true;
    }
    if (!drawn) return changed;
    bus.entity.setLocalPosition(at.x, 0, at.z);
    bus.entity.setLocalEulerAngles(0, (at.yaw * 180) / Math.PI, 0);
    const shut = bus.open <= 0;
    if (bus.whole.enabled !== shut) {
      bus.whole.enabled = shut;
      bus.apart.enabled = !shut;
      if (this.shadows) this.moved = true;
    }
    if (!shut) {
      const k = ease(bus.open);
      BUS_DOORS.forEach((d, i) => {
        const half = d.w / 4;
        for (let s = 0; s < 2; s++) {
          // The front leaf slides forward, the back one back, out from the side as they go.
          const way = s === 0 ? 1 : -1;
          const z = d.z + way * half * (1 + SLIDE * 2 * k);
          bus.leaves[i * 2 + s].setLocalPosition(-BUS.wid / 2 - OUT * Math.min(1, k * 4), 0, z);
        }
      });
    }
    return changed;
  }

  /** Whether a bus is drawn: near, streamed in, not hidden. */
  private visible(at: Drawn['at']): boolean {
    if (!at.shown) return false;
    const r = this.hidden;
    if (r && at.x > r[0] && at.x < r[2] && at.z > r[1] && at.z < r[3]) return false;
    if (this.chunks && !this.chunks.has(chunkAt(at.x, at.z))) return false;
    const e = this.eye;
    return !e || Math.hypot(at.x - e.x, at.z - e.z) <= FAR;
  }

  /** The wheels' and contact shadows' draws, made the first time a bus comes. */
  private ensureDraws(): void {
    if (!this.wheels) {
      const wheel = this.mesh('wheel', () => {
        const p = new Painter(new Geo());
        paintWheel(p);
        return p.geo;
      });
      if (wheel) this.wheels = this.instanced('busWheels', wheel, this.lib.plainFinish(), this.shadows, MOST * 4);
    }
    if (!this.shadowDraw) {
      const shadow = this.mesh('shadow', () => {
        const g = new Geo().color('#000000');
        blob(g, 0, 0, 1, 1, 0, 0.6, 0.03);
        return g;
      });
      if (shadow) this.shadowDraw = this.instanced('busShadows', shadow, this.lib.contactShadow(false), false, MOST);
    }
  }

  /** Every drawn bus's wheels and contact shadow, written again. */
  private writeAround(): void {
    const { m, q } = this;
    const drawn = [...this.buses.values()].filter((b) => b.drawn);
    const wheels = this.wheels;
    const shadows = this.shadowDraw;
    wheels?.setCount(drawn.length * 4);
    shadows?.setCount(drawn.length);
    drawn.forEach((bus, slot) => {
      const { at } = bus;
      const [s, c] = [Math.sin(at.yaw), Math.cos(at.yaw)];
      const r = BUS.wheelR;
      for (let w = 0; w < 4 && wheels; w++) {
        const { side, x, z, front } = WHEELS[w];
        // (The left ones, +x, as the cars': hubs facing out.)
        const yaw = at.yaw + (front ? bus.steer : 0) + (side < 0 ? Math.PI : 0);
        const spin = side < 0 ? -bus.spin : bus.spin;
        compose(1.25, r, r, quatYawPitchRoll(yaw, spin, 0, q), at.x + x * c + z * s, r + LIFT, at.z - x * s + z * c, m);
        wheels.setMatrix(slot * 4 + w, m);
      }
      shadows?.setMatrix(slot, compose(BUS.wid * 0.62, 1, BUS.len * 0.56, quatYawPitchRoll(at.yaw, 0, 0, q), at.x, 0, at.z, m));
    });
    if (wheels) {
      wheels.update();
      wheels.enabled = wheels.count > 0;
    }
    if (shadows) {
      shadows.update();
      shadows.enabled = shadows.count > 0;
    }
  }

  private instanced(name: string, mesh: pc.Mesh, material: pc.Material, castShadows: boolean, capacity: number): Instances {
    const draw = new Instances(this.app.graphicsDevice, this.root, mesh, material, { name, castShadows, capacity: Math.max(1, capacity) });
    this.lib.excludeFromLights([], [draw.meshInstance]);
    draw.enabled = false;
    return draw;
  }

  private mesh(key: string, make: () => Geo): pc.Mesh | null {
    let m = this.meshes.get(key);
    if (!m) {
      const made = meshFrom(this.app.graphicsDevice, make().meshData({ colors: true }));
      if (!made) return null;
      this.meshes.set(key, (m = made));
    }
    return m;
  }
}
