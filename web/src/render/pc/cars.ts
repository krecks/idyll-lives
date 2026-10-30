/**
 * Households' cars in the game on PlayCanvas (sim-core `cars.rs`): parked on their driveways, and
 * driving off to work and back. Each is an entity of its own (its body from `shared/cars.ts`, one
 * mesh per body and paint, shared, and a soft contact shadow) with its four doors as entities on
 * their hinges, placed every frame where the sim's snapshot says it is (`SnapshotLayout.car`),
 * between two snapshots; hidden while it's out of town or its parking spot has been built over,
 * and while it stands on the viewed lot with its basement in view (`hideOn`: the lot's ground is
 * hidden then).
 * A door swings open while someone gets in or out by it, and shut after (`DOOR_SECONDS`, in game
 * time).
 */

import * as pc from './engine';
import { MaterialLibrary } from './materials';
import { drawMesh, meshFrom } from './meshes';
import type { FrameState } from '../../core/bridge';
import type { Seat } from '../shared/characters/seats';
import type { WorldStructure } from '../../core/protocol';
import { CARS, CAR_PAINT, carDoor, DOORS, paintCar, paintDoor } from '../shared/cars';
import { Geo } from '../shared/geometry';
import { GAME_RATE } from '../shared/gameSpeed';
import { blob, Painter } from '../shared/street';


/** How wide a door opens (degrees), and how long it takes to swing open or shut (seconds of game time at 1×: about 9 sim ticks, which getting in and out leave it, sim-core `cars::DOOR_OPENS`). */
const DOOR_OPEN = 62;
const DOOR_SECONDS = 0.45;

/** A car's seat (sim-core `Car::seat_at`): lower than a chair's, so a grown-up's head stays under the roof. */
const CAR_SEAT: Seat = { h: 0.3, fwd: 0 };

interface Drawn {
  entity: pc.Entity;
  /** Its size, and where it stands now (whether it's parked there, and whether it's in town at all). */
  len: number;
  wid: number;
  at: { x: number; z: number; yaw: number; shown: boolean; out: boolean };
  /** Its doors on their hinges, which way each opens (+1 or -1), and how far open each is (0..1). */
  doors: { entity: pc.Entity; side: number; open: number }[];
}

export class Cars {
  private readonly lib: MaterialLibrary;
  private readonly root: pc.Entity;
  private key = '';
  private cars: Drawn[] = [];
  private lastNow = 0;
  /** Body meshes by body and paint, and the contact shadow by body: shared by the cars of a build that look alike (they go with them). */
  private readonly meshes = new Map<string, pc.Mesh>();
  /** Whether a car moved in the last frame (the sun's map is to be drawn again). */
  moved = false;
  /** Cars standing in here (x0, z0, x1, z1) aren't drawn (`hideOn`). */
  private hidden: readonly [number, number, number, number] | null = null;

  constructor(
    private readonly app: pc.AppBase,
    parent: pc.Entity,
    private readonly shadows: boolean,
  ) {
    this.lib = MaterialLibrary.for(app);
    this.root = new pc.Entity('cars');
    parent.addChild(this.root);
  }

  /** The world's cars (built again only when they change: another town, another save). */
  build(world: WorldStructure): void {
    const list = world.cars ?? [];
    const key = JSON.stringify(list.map((c) => [c.plot, c.body, c.paint]));
    if (key === this.key) return;
    this.clear();
    this.key = key;
    for (const info of list) {
      const entity = new pc.Entity('car');
      this.root.addChild(entity);
      const shape = CARS[info.body % CARS.length];
      const paint = CAR_PAINT[info.paint % CAR_PAINT.length];
      const body = this.mesh(`body ${info.body} ${info.paint}`, () => {
        const p = new Painter(new Geo());
        paintCar(p.at(0, 0, 0, 0.015), shape, paint, true);
        return p.geo;
      });
      const shadow = this.mesh(`shadow ${info.body}`, () => {
        const g = new Geo().color('#000000');
        const b = CARS[info.body % CARS.length];
        blob(g, 0, 0, b.wid * 0.62, b.len * 0.56, 0, 0.55, 0.03);
        return g;
      });
      if (body) drawMesh(entity, 'carBody', body, this.lib.plainFinish(), { castShadows: this.shadows });
      if (shadow) drawMesh(entity, 'carShadow', shadow, this.lib.contactShadow(false));
      const doors: Drawn['doors'] = [];
      for (let d = 0; d < DOORS; d++) {
        const { side, hinge } = carDoor(shape, d);
        const mesh = this.mesh(`door ${info.body} ${info.paint} ${d}`, () => {
          const p = new Painter(new Geo());
          paintDoor(p.at(0, 0, 0, 0.015), shape, paint, d);
          return p.geo;
        });
        const door = new pc.Entity('carDoor');
        entity.addChild(door);
        door.setLocalPosition(side * (shape.wid / 2), 0, hinge);
        if (mesh) drawMesh(door, 'carDoorPanel', mesh, this.lib.plainFinish(), { castShadows: this.shadows });
        doors.push({ entity: door, side, open: 0 });
      }
      // Out of the house's lamps, as the street is.
      for (const r of entity.findComponents('render') as pc.RenderComponent[]) this.lib.excludeFromLights([], r.meshInstances);
      this.place(entity, info.x, info.z, info.yaw);
      this.cars.push({ entity, doors, len: shape.len, wid: shape.wid, at: { x: info.x, z: info.z, yaw: info.yaw, shown: false, out: false } });
    }
  }

  /** Each car where the snapshot has it (between the last two). */
  update(frame: FrameState): void {
    const { prev, curr, alpha, layout } = frame;
    const base = layout.carBase;
    const stride = layout.carStride;
    const f = layout.car;
    const count = layout.header.carCount === undefined ? 0 : curr[layout.header.carCount];
    // (Doors swing in game time, as the sim has them: faster at higher speeds, still while paused.)
    const rate = GAME_RATE[curr[layout.header.speed] | 0] ?? 1;
    const dt = Math.min(0.1, Math.max(0, (frame.now - this.lastNow) / 1000)) * rate;
    this.lastNow = frame.now;
    this.moved = false;
    if (base === undefined || stride === undefined || !f) return;
    this.cars.forEach(({ entity, doors, at }, i) => {
      const o = base + i * stride;
      const shown = i < count ? curr[o + f.shown] : 0;
      entity.enabled = shown > 0;
      at.shown = shown === 1;
      at.out = !shown;
      if (!shown) return;
      const k = prev[o + f.shown] > 0 ? Math.min(1, Math.max(0, alpha)) : 1;
      const lerp = (j: number) => prev[o + j] + (curr[o + j] - prev[o + j]) * k;
      let dy = curr[o + f.yaw] - prev[o + f.yaw];
      dy -= Math.round(dy / (Math.PI * 2)) * Math.PI * 2;
      const yaw = k < 1 ? prev[o + f.yaw] + dy * k : curr[o + f.yaw];
      this.place(entity, lerp(f.x), lerp(f.z), yaw);
      [at.x, at.z, at.yaw] = [lerp(f.x), lerp(f.z), yaw];
      if (this.hidden) entity.enabled = !this.inHidden(at.x, at.z);
      if (shown > 1) this.moved = true;
      const open = f.doors === undefined ? 0 : curr[o + f.doors];
      doors.forEach((door, d) => {
        const target = open & (1 << d) ? 1 : 0;
        if (door.open === target) return;
        const step = dt / DOOR_SECONDS;
        door.open = target > door.open ? Math.min(1, door.open + step) : Math.max(0, door.open - step);
        // Eased; the rear edge swings out (`paintDoor`).
        const k = door.open * door.open * (3 - 2 * door.open);
        door.entity.setLocalEulerAngles(0, -door.side * DOOR_OPEN * k, 0);
        this.moved = true;
      });
    });
  }

  /**
   * Cars standing in `rect` (x0, z0, x1, z1) aren't drawn, or none are hidden (null): the viewed lot
   * while its basement is in view, as its ground is. A car driving off it shows as it leaves.
   */
  hideOn(rect: readonly [number, number, number, number] | null): void {
    this.hidden = rect;
    for (const { entity, at } of this.cars) entity.enabled = !at.out && !this.inHidden(at.x, at.z);
  }

  /** The seat of a parked car at (x, z) (someone sat in it), else null: for `Residents.setCarSeat`. */
  seatAt(x: number, z: number): Seat | null {
    for (const { len, wid, at } of this.cars) {
      if (!at.shown) continue;
      const [dx, dz] = [x - at.x, z - at.z];
      const along = dx * Math.sin(at.yaw) + dz * Math.cos(at.yaw);
      const across = dx * Math.cos(at.yaw) - dz * Math.sin(at.yaw);
      if (Math.abs(along) < len / 2 && Math.abs(across) < wid / 2) return CAR_SEAT;
    }
    return null;
  }

  clear(): void {
    for (const c of this.cars) c.entity.destroy();
    this.cars = [];
    this.meshes.clear();
    this.key = '';
  }

  destroy(): void {
    this.clear();
    this.root.destroy();
  }

  private inHidden(x: number, z: number): boolean {
    const r = this.hidden;
    return !!r && x > r[0] && x < r[2] && z > r[1] && z < r[3];
  }

  private place(entity: pc.Entity, x: number, z: number, yaw: number): void {
    entity.setLocalPosition(x, 0, z);
    entity.setLocalEulerAngles(0, (yaw * 180) / Math.PI, 0);
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
