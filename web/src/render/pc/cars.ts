/**
 * Households' cars in the game on PlayCanvas (sim-core `cars.rs`): parked on their driveways, and
 * driving off to work and back, placed every frame where the sim's snapshot says it is
 * (`SnapshotLayout.car`), between two snapshots (and on past the newest while it drives, as the
 * residents go: `FrameState.alpha`); hidden while it's out of town or its parking spot has been
 * built over, while it stands on the viewed lot with its basement in view (`hideOn`: the lot's
 * ground is hidden then), and while it's outside the chunks streamed in or far off (`setView`).
 *
 * Drawn in as few draws as will do: the parked ones with their doors shut one instanced draw per
 * body and paint (a mesh each, shared); those driving, or with a door open, an entity of their own
 * (the body, tilted a little as it speeds up and slows down, and its four doors on their hinges,
 * drawn in one piece with it while they're all shut); every car's wheels one instanced draw (they
 * turn as it goes and the front ones steer: `paintWheel`), and their soft contact shadows another.
 * A door swings open while someone gets in or out by it, and shut after (`DOOR_SECONDS`, in game
 * time). Those riding in a car that drives sit in it where it is drawn (`ride`).
 */

import * as pc from './engine';
import { Instances } from './instancing';
import { MaterialLibrary } from './materials';
import { drawMesh, meshFrom } from './meshes';
import type { FrameState } from '../../core/bridge';
import type { Seat } from '../shared/characters/seats';
import type { WorldStructure } from '../../core/protocol';
import { CARS, CAR_PAINT, carDoor, carriedAlong, DOORS, paintCar, paintDoor, paintWheel, wheelAt, type CarBody, type CarPose } from '../shared/cars';
import { chunkAt } from '../shared/chunks';
import { lerpAngle, wrapAngle } from '../shared/characters/rig';
import { DOOR_SWING, ease } from '../shared/doors';
import { Geo } from '../shared/geometry';
import { GAME_RATE } from '../shared/gameSpeed';
import { blob, Painter } from '../shared/street';

/** How wide a door opens (degrees). It swings as a house's door does (`DOOR_SWING`, in game time: about 9 sim ticks, which getting in and out leave it, sim-core `cars::DOOR_OPENS`). */
const DOOR_OPEN = 62;

/** A car's seat (sim-core `Car::seat_at`): lower than a chair's, so a grown-up's head stays under the roof. */
const CAR_SEAT: Seat = { h: 0.3, fwd: 0 };

/** Cars further than this from the camera's target aren't drawn (m): past the fog, a dot at most. */
const FAR = 650;
/** How far the front wheels steer at most (rad), and how quickly they follow the way the car turns (per second). */
const STEER_MAX = 0.5;
const STEER_RATE = 8;
/** How far the body tilts at most as it speeds up or slows down (rad, nose up speeding up), per m/s² of it, and how quickly it settles (per second). */
const PITCH_MAX = 0.03;
const PITCH_PER_ACCEL = 0.006;
const PITCH_RATE = 5;
/** How quickly the speed the tilt goes by follows the car's (per second): the frames between snapshots go in steps. */
const SPEED_RATE = 4;
/** Further than this in a frame (m), a car was put there (a load, back into town), not driven: its wheels don't turn for it. */
const JUMP = 10;
/** Height of the body and wheels over the ground (as the street's `paintCar`). */
const LIFT = 0.015;

/** A car's own entity, while it drives or has a door open. */
interface Own {
  entity: pc.Entity;
  /** The body, tilted about the car's middle on the ground (its pitch). */
  tilt: pc.Entity;
  /** Drawn in one piece while its doors are shut (`whole`), apart (`apart`: the body without its doors, and the doors) while one is open. */
  whole: pc.Entity;
  apart: pc.Entity;
  doors: pc.Entity[];
}

interface Drawn {
  shape: CarBody;
  /** Its body and paint (`body paint`): the instanced draw it's in while parked. */
  look: string;
  body: number;
  paint: number;
  /** Where it's drawn now (`shown`: the snapshot's, 0 out of town, 1 parked, 2 moving). */
  at: { x: number; z: number; yaw: number; shown: number };
  /** Where it was drawn last frame (its wheels turn by the way between). */
  last: { x: number; z: number; yaw: number };
  /** Where the newest snapshot has it, and whether it drives in either snapshot: those riding in it sit in that frame (`ride`). */
  snap: { x: number; z: number; yaw: number; driving: boolean };
  /** How far open each door is (0..1), and which way each opens (+1 or -1). */
  open: number[];
  sides: number[];
  /** Its own entity, made the first time it drives or opens a door (null till then). */
  own: Own | null;
  /** Whether it is drawn (in town, near, not hidden), and whether by its own entity (else the parked draw for its look). */
  drawn: boolean;
  owned: boolean;
  /** The wheels' turn (rad, about their axle), the front ones' steer (rad, +: towards +x), the body's tilt (rad, nose up) and its speed (m/s, smoothed). */
  spin: number;
  steer: number;
  pitch: number;
  speed: number;
}

/** The parked cars of a look (body and paint), one instanced draw. */
interface Parked {
  draw: Instances;
  /** Its cars changed (or one moved): its copies are written again. */
  dirty: boolean;
}

export class Cars {
  private readonly lib: MaterialLibrary;
  private readonly root: pc.Entity;
  private key = '';
  private cars: Drawn[] = [];
  private lastNow = 0;
  /** Body meshes by body and paint, the wheel and the contact shadow: shared by the cars of a build that look alike (they go with them). */
  private readonly meshes = new Map<string, pc.Mesh>();
  /** The parked cars by look; every car's wheels; every car's contact shadow. */
  private readonly parked = new Map<string, Parked>();
  private wheels: Instances | null = null;
  private shadowDraw: Instances | null = null;
  /** The wheels and contact shadows are written again (a car drawn moved, or came or went). */
  private around = true;
  /** Whether a car moved in the last frame where it casts a shadow into the sun's map (the map is to be drawn again). */
  moved = false;
  /** Cars standing in here (x0, z0, x1, z1) aren't drawn (`hideOn`). */
  private hidden: readonly [number, number, number, number] | null = null;
  /** The chunks streamed in (null: all of them) and the camera's target: cars outside them, or further than `FAR` from it, aren't drawn (`setView`). */
  private chunks: ReadonlySet<number> | null = null;
  private eye: { x: number; z: number } | null = null;
  /** Whether a car at (x, z) could cast into the sun's map (`SunShadows.reaches`; null: anywhere): only one moving there draws it again. */
  private inMap: ((x: number, z: number) => boolean) | null = null;
  /** The frame the cars were last placed for (`pose`: the residents riding in them ask first). */
  private posedAt = NaN;
  private posedCurr: Float32Array | null = null;
  /** Scratch: a matrix (game space). */
  private readonly m = new Float32Array(16);

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
      const body = info.body % CARS.length;
      const paint = info.paint % CAR_PAINT.length;
      const shape = CARS[body];
      const at = { x: info.x, z: info.z, yaw: info.yaw, shown: 0 };
      this.cars.push({
        shape,
        look: `${body} ${paint}`,
        body,
        paint,
        at,
        last: { x: at.x, z: at.z, yaw: at.yaw },
        snap: { x: at.x, z: at.z, yaw: at.yaw, driving: false },
        open: new Array<number>(DOORS).fill(0),
        sides: Array.from({ length: DOORS }, (_, d) => carDoor(shape, d).side),
        own: null,
        drawn: false,
        owned: false,
        spin: 0,
        steer: 0,
        pitch: 0,
        speed: 0,
      });
    }
    const wheel = this.mesh('wheel', () => {
      const p = new Painter(new Geo());
      paintWheel(p);
      return p.geo;
    });
    if (wheel) this.wheels = this.instanced('carWheels', wheel, this.lib.plainFinish(), this.shadows, list.length * 4);
    // (A blob of radius 1, stretched to each car's size.)
    const shadow = this.mesh('shadow', () => {
      const g = new Geo().color('#000000');
      blob(g, 0, 0, 1, 1, 0, 0.55, 0.03);
      return g;
    });
    if (shadow) this.shadowDraw = this.instanced('carShadows', shadow, this.lib.contactShadow(false), false, list.length);
    this.around = true;
  }

  /** Each car where the snapshot has it (between the last two), its doors, wheels and tilt, and how it's drawn. */
  update(frame: FrameState): void {
    const { curr, layout } = frame;
    const f = layout.car;
    // (Doors swing in game time, as the sim has them: faster at higher speeds, still while paused.)
    const rate = GAME_RATE[curr[layout.header.speed] | 0] ?? 1;
    const real = Math.min(0.1, Math.max(0, (frame.now - this.lastNow) / 1000));
    const dt = real * rate;
    this.lastNow = frame.now;
    this.moved = false;
    if (layout.carBase === undefined || layout.carStride === undefined || !f) return;
    this.pose(frame);
    const step = dt / DOOR_SWING;
    const doorsAt = f.doors;
    for (let i = 0; i < this.cars.length; i++) {
      const car = this.cars[i];
      const { at, last } = car;
      if (!at.shown) continue;
      const o = layout.carBase + i * layout.carStride;
      let changed = at.x !== last.x || at.z !== last.z || at.yaw !== last.yaw;
      // (Paused, nothing goes on settling.)
      if (changed) this.roll(car, real);
      else if (rate > 0) changed = this.settle(car, real);
      [last.x, last.z, last.yaw] = [at.x, at.z, at.yaw];
      const open = doorsAt === undefined ? 0 : curr[o + doorsAt];
      for (let d = 0; d < DOORS; d++) {
        const target = open & (1 << d) ? 1 : 0;
        if (car.open[d] === target) continue;
        car.open[d] = target > car.open[d] ? Math.min(1, car.open[d] + step) : Math.max(0, car.open[d] - step);
        changed = true;
      }
      if (!changed || !car.drawn) continue;
      this.around = true;
      if (this.shadows && this.inShadows(at.x, at.z)) this.moved = true;
    }
    this.apply();
  }

  /**
   * Cars standing in `rect` (x0, z0, x1, z1) aren't drawn, or none are hidden (null): the viewed lot
   * while its basement is in view, as its ground is. A car driving off it shows as it leaves.
   */
  hideOn(rect: readonly [number, number, number, number] | null): void {
    this.hidden = rect;
    this.apply();
  }

  /**
   * The chunks streamed in (null: the whole town is drawn) and the camera's target (x, z): cars
   * outside the chunks, or further than `FAR` from the target, aren't drawn.
   */
  setView(chunks: ReadonlySet<number> | null, x: number, z: number): void {
    this.chunks = chunks;
    this.eye = { x, z };
    this.apply();
  }

  /** Whether a spot can cast into the sun's map (`SunShadows.reaches`; null: anywhere): a car moving elsewhere doesn't draw it again. */
  setShadowTest(inMap: ((x: number, z: number) => boolean) | null): void {
    this.inMap = inMap;
  }

  /** The seat of a car at (x, z) where it's drawn (someone sat in it, parked or driving), else null: for `Residents.setCarSeat`. */
  seatAt(x: number, z: number): Seat | null {
    for (const { shape, at } of this.cars) {
      if (!at.shown) continue;
      const [dx, dz] = [x - at.x, z - at.z];
      const along = dx * Math.sin(at.yaw) + dz * Math.cos(at.yaw);
      const across = dx * Math.cos(at.yaw) - dz * Math.sin(at.yaw);
      if (Math.abs(along) < shape.len / 2 && Math.abs(across) < shape.wid / 2) return CAR_SEAT;
    }
    return null;
  }

  /**
   * Someone the newest snapshot has at (x, z), facing `yaw`, in a car that drives (in either
   * snapshot; or, `aboard`, one they rode in last frame: it may still be coming to a stop between the
   * snapshots): where they sit in it as it is drawn this frame (`out`: x, z, yaw), so they go with it
   * rigidly, turns and all, not easing after it. False when they aren't in one. For
   * `Residents.setCarRide`.
   */
  ride(frame: FrameState, x: number, z: number, yaw: number, aboard: boolean, out: CarPose): boolean {
    if (!this.cars.length) return false;
    this.pose(frame);
    for (const { shape, snap, at } of this.cars) {
      if ((snap.driving || aboard) && at.shown && carriedAlong(shape, snap, at, x, z, yaw, out)) return true;
    }
    return false;
  }

  clear(): void {
    for (const c of this.cars) c.own?.entity.destroy();
    this.cars = [];
    for (const p of this.parked.values()) p.draw.destroy();
    this.parked.clear();
    this.wheels?.destroy();
    this.shadowDraw?.destroy();
    this.wheels = this.shadowDraw = null;
    // (The instanced draws hold their meshes without taking them; the entities' are let go as they went.)
    for (const m of this.meshes.values()) if (m.refCount === 0) m.destroy();
    this.meshes.clear();
    this.key = '';
    this.posedAt = NaN;
  }

  destroy(): void {
    this.clear();
    this.root.destroy();
  }

  /** Where each car is drawn this frame (between the last two snapshots), once a frame: the residents riding in them ask before `update`. */
  private pose(frame: FrameState): void {
    const { prev, curr, alpha, layout, now } = frame;
    if (now === this.posedAt && curr === this.posedCurr) return;
    this.posedAt = now;
    this.posedCurr = curr;
    const base = layout.carBase;
    const stride = layout.carStride;
    const f = layout.car;
    if (base === undefined || stride === undefined || !f) return;
    const count = layout.header.carCount === undefined ? 0 : curr[layout.header.carCount];
    const before = layout.header.carCount === undefined ? 0 : prev[layout.header.carCount];
    for (let i = 0; i < this.cars.length; i++) {
      const { at, snap } = this.cars[i];
      const o = base + i * stride;
      at.shown = i < count ? curr[o + f.shown] : 0;
      if (!at.shown) continue;
      const was = i < before ? prev[o + f.shown] : 0;
      // Driving in both, on past the newest snapshot while the next is late (as far as the residents
      // go: the bridge's `MAX_EXTRAPOLATION`), or it would stall and then jump; parking, not past its spot.
      const k = !was ? 1 : at.shown === 2 && was === 2 ? Math.max(0, alpha) : Math.min(1, Math.max(0, alpha));
      const [cx, cz, cyaw] = [curr[o + f.x], curr[o + f.z], curr[o + f.yaw]];
      at.x = prev[o + f.x] + (cx - prev[o + f.x]) * k;
      at.z = prev[o + f.z] + (cz - prev[o + f.z]) * k;
      at.yaw = lerpAngle(prev[o + f.yaw], cyaw, k);
      [snap.x, snap.z, snap.yaw] = [cx, cz, cyaw];
      snap.driving = at.shown === 2 || was === 2;
    }
  }

  /** A car moved since last frame (`real` seconds): its wheels turn by the way it went, the front ones steer by how it turned, the body tilts by how its speed changed. */
  private roll(car: Drawn, real: number): void {
    const { at, last, shape } = car;
    const [dx, dz] = [at.x - last.x, at.z - last.z];
    if (Math.hypot(dx, dz) > JUMP) {
      car.speed = 0;
      return;
    }
    const along = dx * Math.sin(at.yaw) + dz * Math.cos(at.yaw);
    car.spin = (car.spin + along / shape.wheelR) % (Math.PI * 2);
    if (real <= 0) return;
    // Turning by `turn` over `along` metres: a circle the front wheels steer round (a bicycle's, the wheelbase long).
    const turn = wrapAngle(at.yaw - last.yaw);
    if (Math.abs(along) > 1e-3) {
      const goal = Math.max(-STEER_MAX, Math.min(STEER_MAX, Math.atan((turn * shape.wheelBase) / along)));
      car.steer += (goal - car.steer) * (1 - Math.exp(-real * STEER_RATE));
    }
    this.tilt(car, along / real, real);
  }

  /** A car standing still this frame: its tilt settles, its wheels straighten once parked. Whether either changed. */
  private settle(car: Drawn, real: number): boolean {
    let changed = false;
    if (car.at.shown === 1 && car.steer !== 0) {
      car.steer = Math.abs(car.steer) < 0.01 ? 0 : car.steer * Math.exp(-real * STEER_RATE);
      changed = true;
    }
    if (car.speed !== 0 || car.pitch !== 0) changed = this.tilt(car, 0, real) || changed;
    return changed;
  }

  /** The body's tilt for its speed now (`speed`, m/s): nose up speeding up, down slowing down. Whether it changed. */
  private tilt(car: Drawn, speed: number, real: number): boolean {
    if (real <= 0) return false;
    const was = car.speed;
    car.speed += (speed - car.speed) * (1 - Math.exp(-real * SPEED_RATE));
    if (Math.abs(car.speed) < 0.01 && speed === 0) car.speed = 0;
    const accel = (Math.abs(car.speed) - Math.abs(was)) / real;
    const goal = Math.max(-PITCH_MAX, Math.min(PITCH_MAX, accel * PITCH_PER_ACCEL));
    const pitch = car.pitch;
    car.pitch += (goal - car.pitch) * (1 - Math.exp(-real * PITCH_RATE));
    if (Math.abs(car.pitch) < 1e-4 && goal === 0) car.pitch = 0;
    return car.pitch !== pitch;
  }

  /** Whether a car at (x, z) can cast into the sun's map (`setShadowTest`). */
  private inShadows(x: number, z: number): boolean {
    return !this.inMap || this.inMap(x, z);
  }

  /** Whether a car is drawn: in town, near, and not standing where cars are hidden (`hideOn`). */
  private visible(at: Drawn['at']): boolean {
    if (!at.shown) return false;
    const r = this.hidden;
    if (r && at.x > r[0] && at.x < r[2] && at.z > r[1] && at.z < r[3]) return false;
    if (this.chunks && !this.chunks.has(chunkAt(at.x, at.z))) return false;
    const e = this.eye;
    return !e || Math.hypot(at.x - e.x, at.z - e.z) <= FAR;
  }

  /** How each car is drawn now (by its own entity, in its look's parked draw, or not at all), and the draws written again where that changed. */
  private apply(): void {
    for (const car of this.cars) {
      const drawn = this.visible(car.at);
      const owned = drawn && (car.at.shown === 2 || car.pitch !== 0 || car.open.some((o) => o > 0));
      if (drawn !== car.drawn || owned !== car.owned) {
        // (In or out of a parked draw, and the wheels and shadows with it.)
        const parked = (car.drawn && !car.owned) || (drawn && !owned) ? this.parkedFor(car) : null;
        if (parked) parked.dirty = true;
        this.around = true;
        if (this.shadows && this.inShadows(car.at.x, car.at.z)) this.moved = true;
        car.drawn = drawn;
        car.owned = owned;
      }
      if (owned) this.placeOwn(car);
      else if (car.own?.entity.enabled) car.own.entity.enabled = false;
    }
    for (const [look, p] of this.parked) {
      if (!p.dirty) continue;
      p.dirty = false;
      let n = 0;
      for (const car of this.cars) if (car.look === look && car.drawn && !car.owned) n++;
      p.draw.setCount(n);
      n = 0;
      for (const car of this.cars) {
        if (car.look !== look || !car.drawn || car.owned) continue;
        p.draw.setMatrix(n++, this.carMatrix(car.at, 0));
      }
      p.draw.update();
      p.draw.enabled = n > 0;
    }
    if (this.around) this.writeAround();
  }

  /** Every drawn car's wheels and contact shadow, written again. */
  private writeAround(): void {
    this.around = false;
    const wheels = this.wheels;
    const shadows = this.shadowDraw;
    let n = 0;
    for (const car of this.cars) if (car.drawn) n++;
    wheels?.setCount(n * 4);
    shadows?.setCount(n);
    const m = this.m;
    let j = 0;
    for (const car of this.cars) {
      if (!car.drawn) continue;
      const { at, shape } = car;
      const [s, c] = [Math.sin(at.yaw), Math.cos(at.yaw)];
      if (wheels) {
        const r = shape.wheelR;
        for (let w = 0; w < 4; w++) {
          const { side, x, z, front } = wheelAt(shape, w);
          // The left ones turned round (their hubs face out), turning the other way for it.
          const yaw = at.yaw + (front ? car.steer : 0) + (side < 0 ? Math.PI : 0);
          const spin = side < 0 ? -car.spin : car.spin;
          const [sy, cy, sp, cp] = [Math.sin(yaw), Math.cos(yaw), Math.sin(spin), Math.cos(spin)];
          // Turned about y (`yaw`), then about x (`spin`), scaled by its radius across y and z.
          m[0] = cy;
          m[1] = 0;
          m[2] = -sy;
          m[3] = 0;
          m[4] = sy * sp * r;
          m[5] = cp * r;
          m[6] = cy * sp * r;
          m[7] = 0;
          m[8] = sy * cp * r;
          m[9] = -sp * r;
          m[10] = cy * cp * r;
          m[11] = 0;
          m[12] = at.x + x * c + z * s;
          m[13] = r + LIFT;
          m[14] = at.z - x * s + z * c;
          m[15] = 1;
          wheels.setMatrix(j * 4 + w, m);
        }
      }
      if (shadows) {
        const [rx, rz] = [shape.wid * 0.62, shape.len * 0.56];
        m.fill(0);
        m[0] = c * rx;
        m[2] = -s * rx;
        m[5] = 1;
        m[8] = s * rz;
        m[10] = c * rz;
        m[12] = at.x;
        m[14] = at.z;
        m[15] = 1;
        shadows.setMatrix(j, m);
      }
      j++;
    }
    if (wheels) {
      wheels.update();
      wheels.enabled = n > 0;
    }
    if (shadows) {
      shadows.update();
      shadows.enabled = n > 0;
    }
  }

  /** A car's body in game space (`pitch`: nose up, about its middle on the ground). */
  private carMatrix(at: Drawn['at'], pitch: number): Float32Array {
    const m = this.m;
    const [s, c, sp, cp] = [Math.sin(at.yaw), Math.cos(at.yaw), Math.sin(-pitch), Math.cos(-pitch)];
    // Turned about y, then (nose up: back about x) tilted.
    m[0] = c;
    m[1] = 0;
    m[2] = -s;
    m[3] = 0;
    m[4] = s * sp;
    m[5] = cp;
    m[6] = c * sp;
    m[7] = 0;
    m[8] = s * cp;
    m[9] = -sp;
    m[10] = c * cp;
    m[11] = 0;
    m[12] = at.x;
    m[13] = 0;
    m[14] = at.z;
    m[15] = 1;
    return m;
  }

  /** A car drawn by its own entity, placed, tilted, its doors swung, in one piece while they're shut. */
  private placeOwn(car: Drawn): void {
    const own = (car.own ??= this.makeOwn(car));
    if (!own.entity.enabled) own.entity.enabled = true;
    own.entity.setLocalPosition(car.at.x, 0, car.at.z);
    own.entity.setLocalEulerAngles(0, (car.at.yaw * 180) / Math.PI, 0);
    // (Nose up: turned back about x.)
    own.tilt.setLocalEulerAngles((-car.pitch * 180) / Math.PI, 0, 0);
    let shut = true;
    for (let d = 0; d < DOORS; d++) {
      const open = car.open[d];
      if (open > 0) shut = false;
      // The rear edge swings out (`paintDoor`).
      own.doors[d].setLocalEulerAngles(0, -car.sides[d] * DOOR_OPEN * ease(open), 0);
    }
    if (own.whole.enabled !== shut) {
      own.whole.enabled = shut;
      own.apart.enabled = !shut;
    }
  }

  /** A car's own entity (the first time it drives or opens a door): its body in one piece, and apart with its doors. */
  private makeOwn(car: Drawn): Own {
    const { shape, body, paint } = car;
    const hex = CAR_PAINT[paint];
    const entity = new pc.Entity('car');
    this.root.addChild(entity);
    const tilt = new pc.Entity('carTilt');
    entity.addChild(tilt);
    const whole = new pc.Entity('carWhole');
    const apart = new pc.Entity('carApart');
    tilt.addChild(whole);
    tilt.addChild(apart);
    apart.enabled = false;
    const one = this.wholeMesh(car);
    if (one) drawMesh(whole, 'carBody', one, this.lib.plainFinish(), { castShadows: this.shadows });
    const bare = this.mesh(`body ${body} ${paint}`, () => {
      const p = new Painter(new Geo());
      paintCar(p.at(0, 0, 0, LIFT), shape, hex, true, false);
      return p.geo;
    });
    if (bare) drawMesh(apart, 'carBody', bare, this.lib.plainFinish(), { castShadows: this.shadows });
    const doors: pc.Entity[] = [];
    for (let d = 0; d < DOORS; d++) {
      const { side, hinge } = carDoor(shape, d);
      const mesh = this.mesh(`door ${body} ${paint} ${d}`, () => {
        const p = new Painter(new Geo());
        paintDoor(p.at(0, 0, 0, LIFT), shape, hex, d);
        return p.geo;
      });
      const door = new pc.Entity('carDoor');
      apart.addChild(door);
      door.setLocalPosition(side * (shape.wid / 2), 0, hinge);
      if (mesh) drawMesh(door, 'carDoorPanel', mesh, this.lib.plainFinish(), { castShadows: this.shadows });
      doors.push(door);
    }
    // Out of the house's lamps, as the street is.
    for (const r of entity.findComponents('render') as pc.RenderComponent[]) this.lib.excludeFromLights([], r.meshInstances);
    return { entity, tilt, whole, apart, doors };
  }

  /** A car's body in one piece, its doors shut, without its wheels (shut, as most are most of the time, one draw instead of five). */
  private wholeMesh(car: Drawn): pc.Mesh | null {
    return this.mesh(`whole ${car.body} ${car.paint}`, () => {
      const p = new Painter(new Geo());
      paintCar(p.at(0, 0, 0, LIFT), car.shape, CAR_PAINT[car.paint], false, false);
      return p.geo;
    });
  }

  /** The parked draw for a car's look (made the first time a car of it parks in view; null: its body has no mesh). */
  private parkedFor(car: Drawn): Parked | null {
    let p = this.parked.get(car.look);
    if (!p) {
      const mesh = this.wholeMesh(car);
      if (!mesh) return null;
      this.parked.set(car.look, (p = { draw: this.instanced('carsParked', mesh, this.lib.plainFinish(), this.shadows, 4), dirty: true }));
    }
    return p;
  }

  /** An instanced draw of `mesh` under the cars, out of the house's lamps. */
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
