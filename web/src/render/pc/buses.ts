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
import { drawMesh } from './meshes';
import { LIFT, rollWheels, Vehicles } from './vehicles';
import type { FrameState } from '../../core/bridge';
import { BUS, BUS_DOORS, busLeaf, busWheelAt, paintBus } from '../shared/bus';
import { lerpAngle } from '../shared/characters/rig';
import { ease } from '../shared/doors';
import { Geo } from '../shared/geometry';
import { GAME_RATE } from '../shared/gameSpeed';
import { Painter } from '../shared/street';

/** How long a door takes to open or shut (s of game time at 1×: the sim's doors open about 12 ticks after it stops, `bus::DOOR_TICKS`). */
const DOOR_SECONDS = 0.55;
/** How far a door's leaves slide apart (a share of a leaf's width), and out from the side (m). */
const SLIDE = 0.92;
const OUT = 0.05;
/** How far the front wheels steer at most (rad) and how quickly (per second). */
const STEER = { max: 0.45, rate: 6 };
/** The distance between its axles (m). */
const WHEEL_BASE = BUS.axles[0] - BUS.axles[1];
/** The bus's four wheels (`busWheelAt`), a little wider than a car's. */
const WHEELS = [0, 1, 2, 3].map(busWheelAt);
const WHEEL_WIDE = 1.25;
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

export class Buses extends Vehicles {
  private readonly buses = new Map<number, Drawn>();
  /** Buses gone out of town, kept for the next of their line (their meshes stay held). */
  private readonly spare: Drawn[] = [];
  private lastNow = 0;
  /** Where each bus was in the snapshot before (its row), by which bus it is: filled again each frame. */
  private readonly before = new Map<number, number>();

  constructor(app: pc.AppBase, parent: pc.Entity, shadows: boolean) {
    super(app, parent, shadows, 'buses');
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
    const before = this.before;
    before.clear();
    for (let k = 0; k < Math.min(MOST, prev[countAt] | 0); k++) before.set(prev[base + k * stride + f.id], base + k * stride);
    for (const b of this.buses.values()) b.seen = false;
    let changed = false;
    for (let k = 0; k < count; k++) {
      const o = base + k * stride;
      const shown = curr[o + f.shown];
      if (!shown) continue;
      const id = curr[o + f.id];
      const x = curr[o + f.x];
      const z = curr[o + f.z];
      const yaw = curr[o + f.yaw];
      const bus = this.buses.get(id) ?? this.make(id, curr[o + f.line] | 0, x, z, yaw);
      bus.seen = true;
      const p = before.get(id);
      // Driving in both, on past the newest snapshot while the next is late; standing, not past it.
      const was = p === undefined ? 0 : prev[p + f.shown];
      const t = !was ? 1 : shown === 2 && was === 2 ? Math.max(0, alpha) : Math.min(1, Math.max(0, alpha));
      const px = p === undefined ? x : prev[p + f.x];
      const pz = p === undefined ? z : prev[p + f.z];
      const pyaw = p === undefined ? yaw : prev[p + f.yaw];
      bus.at.x = px + (x - px) * t;
      bus.at.z = pz + (z - pz) * t;
      bus.at.yaw = lerpAngle(pyaw, yaw, t);
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

  clear(): void {
    for (const b of this.buses.values()) b.entity.destroy();
    for (const b of this.spare) b.entity.destroy();
    this.buses.clear();
    this.spare.length = 0;
    this.clearDraws();
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
    this.ensureAround('bus', MOST, 0.6);
    const bus: Drawn = { id, line, entity, whole, apart, leaves, at: { x, z, yaw, shown: 0 }, last: { x, z, yaw }, open: 0, spin: 0, steer: 0, seen: true, drawn: false };
    this.buses.set(id, bus);
    return bus;
  }

  /** A bus placed this frame (`real` seconds since the last): drawn or not, turned, its wheels rolled, its doors slid. Whether it changed what the wheels and shadows draw show. */
  private place(bus: Drawn, real: number): boolean {
    const { at, last } = bus;
    const drawn = this.visible(at);
    const came = drawn && !bus.drawn;
    let changed = drawn !== bus.drawn;
    bus.drawn = drawn;
    if (bus.entity.enabled !== drawn) bus.entity.enabled = drawn;
    const moved = at.x !== last.x || at.z !== last.z || at.yaw !== last.yaw;
    if (moved) {
      changed = true;
      rollWheels(bus, at, last, BUS.wheelR, WHEEL_BASE, STEER, real);
      last.x = at.x;
      last.z = at.z;
      last.yaw = at.yaw;
      if (this.shadows && this.inShadows(at.x, at.z)) this.moved = true;
    }
    if (!drawn) return changed;
    // (Only when it moved, or it shows again: it may have moved while it wasn't drawn.)
    if (moved || came) {
      bus.entity.setLocalPosition(at.x, 0, at.z);
      bus.entity.setLocalEulerAngles(0, (at.yaw * 180) / Math.PI, 0);
    }
    const shut = bus.open <= 0;
    if (bus.whole.enabled !== shut) {
      bus.whole.enabled = shut;
      bus.apart.enabled = !shut;
      if (this.shadows) this.moved = true;
    }
    if (!shut) {
      const k = ease(bus.open);
      for (let i = 0; i < BUS_DOORS.length; i++) {
        const d = BUS_DOORS[i];
        const half = d.w / 4;
        for (let s = 0; s < 2; s++) {
          // The front leaf slides forward, the back one back, out from the side as they go.
          const way = s === 0 ? 1 : -1;
          const z = d.z + way * half * (1 + SLIDE * 2 * k);
          bus.leaves[i * 2 + s].setLocalPosition(-BUS.wid / 2 - OUT * Math.min(1, k * 4), 0, z);
        }
      }
    }
    return changed;
  }

  /** Every drawn bus's wheels and contact shadow, written again. */
  private writeAround(): void {
    let n = 0;
    for (const bus of this.buses.values()) if (bus.drawn) n++;
    this.wheels?.setCount(n * 4);
    this.shadowDraw?.setCount(n);
    let slot = 0;
    // (The left ones, +x, as the cars': hubs facing out.)
    for (const bus of this.buses.values()) if (bus.drawn) this.writeAt(slot++, bus.at, BUS, WHEELS, bus, WHEEL_WIDE);
    this.sendAround();
  }
}
