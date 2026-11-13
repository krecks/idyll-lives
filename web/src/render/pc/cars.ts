/**
 * Households' cars and the work taxis in the game on PlayCanvas (sim-core `cars.rs`, `taxi.rs`):
 * parked on their driveways, and driving off to work and back, placed every frame where the sim's snapshot says it is
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
 *
 * The snapshot's car rows are the households' first, in the structure's `cars` order (their looks),
 * then the work taxis' (`car.kind`): none in the structure, so each is drawn as the taxi
 * (`shared/cars.ts` `paintTaxi`), made the first time its row comes (a row stays for good; an
 * unused taxi's isn't shown).
 */

import * as pc from './engine';
import type { Instances } from './instancing';
import { drawMesh } from './meshes';
import { LIFT, rollWheels, Vehicles } from './vehicles';
import type { FrameState } from '../../core/bridge';
import type { Seat } from '../shared/characters/seats';
import type { WorldStructure } from '../../core/protocol';
import { CARS, CAR_PAINT, TAXI_PAINT, carDoor, carriedAlong, carRowIsTaxi, DOORS, inCar, paintCar, paintDoor, paintTaxi, paintTaxiDoor, TAXI_BODY, wheelAt, type CarBody, type CarPose } from '../shared/cars';
import { lerpAngle } from '../shared/characters/rig';
import { DOOR_SWING, ease } from '../shared/doors';
import { Geo } from '../shared/geometry';
import { PLAYBACK_RATE } from '../shared/gameSpeed';
import { compose, quatYawPitchRoll } from '../shared/math';
import { Painter } from '../shared/street';

/**
 * How wide a door opens (degrees). It swings as a house's door does, `DOOR_SWING` s of the sim's real time (9 ticks
 * at 20 a second), which getting in and out leave it (sim-core `cars::DOOR_OPENS`, 0.5 s).
 */
const DOOR_OPEN = 62;

/** A car's seat (sim-core `Car::seat_at`): lower than a chair's, so a grown-up's head stays under the roof. */
const CAR_SEAT: Seat = { h: 0.3, fwd: 0 };

/** How far the front wheels steer at most (rad), and how quickly they follow the way the car turns (per second). */
const STEER = { max: 0.5, rate: 8 };
/**
 * How far the body tilts at most as it speeds up or slows down (rad, nose up speeding up), per m/s² of it (of drive
 * time, as the sim's `ACCEL`: the same at any game speed), and how quickly it settles (per second).
 */
const PITCH_MAX = 0.03;
const PITCH_PER_ACCEL = 0.006;
const PITCH_RATE = 5;
/** How quickly the speed the tilt goes by follows the car's (per second): the frames between snapshots go in steps. */
const SPEED_RATE = 4;
/** Each body's four wheels (`wheelAt`), worked out once. */
const WHEELS = CARS.map((body) => [0, 1, 2, 3].map((w) => wheelAt(body, w)));

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
  /** Its row in the snapshot's cars (a household's car: its place in the structure's `cars`). */
  row: number;
  /** A work taxi (its look the taxi's: `paintTaxi`), else a household's car (`body`, `paint`). */
  taxi: boolean;
  shape: CarBody;
  /** Its body and paint (`body paint`, or `taxi`): the instanced draw it's in while parked. */
  look: string;
  body: number;
  paint: number;
  /** Where it's drawn now (`shown`: the snapshot's, 0 out of town, 1 parked, 2 moving). */
  at: { x: number; z: number; yaw: number; shown: number };
  /** Where it was drawn last frame (its wheels turn by the way between), and whether and how (`at.shown`). */
  last: { x: number; z: number; yaw: number; shown: number };
  /** Where the newest snapshot has it, and whether it drives in either snapshot: those riding in it sit in that frame (`ride`). */
  snap: { x: number; z: number; yaw: number; driving: boolean };
  /** Where it was drawn the frame before (`update`): its shadow in the sun's map was cast from there. */
  castFrom: { x: number; z: number };
  /** How far open each door is (0..1), which way each opens (+1 or -1), and whether any is open at all. */
  open: number[];
  sides: number[];
  ajar: boolean;
  /** Its own entity, made the first time it drives or opens a door (null till then). */
  own: Own | null;
  /** Whether it is drawn (in town, near, not hidden), and whether by its own entity (else the parked draw for its look). */
  drawn: boolean;
  owned: boolean;
  /** Its wheels' and contact shadow's place in their draws while drawn (as `writeAround` last wrote them). */
  slot: number;
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

export class Cars extends Vehicles {
  private key = '';
  private cars: Drawn[] = [];
  /** The snapshot's car rows looked at for a taxi so far (the households' and the taxis' come before). */
  private scanned = 0;
  private lastNow = 0;
  /** The parked cars by look (their body meshes, by body and paint, shared by the cars of a build that look alike). */
  private readonly parked = new Map<string, Parked>();
  /** The wheels and contact shadows are written again: every car's (one came or went), or only those of the cars drawn that moved (`stirred`). */
  private around = true;
  private readonly stirred: Drawn[] = [];
  /** The cars in town driving in either snapshot (`pose`): all a resident who wasn't riding can be riding in (`ride`). */
  private readonly driving: Drawn[] = [];
  /** The frame the cars were last placed for (`pose`: the residents riding in them ask first). */
  private posedAt = NaN;
  private posedCurr: Float32Array | null = null;

  constructor(app: pc.AppBase, parent: pc.Entity, shadows: boolean) {
    super(app, parent, shadows, 'cars');
  }

  /** The world's cars (built again only when they change: another town, another save). */
  build(world: WorldStructure): void {
    const list = world.cars ?? [];
    const key = JSON.stringify(list.map((c) => [c.plot, c.body, c.paint]));
    if (key === this.key) return;
    this.clear();
    this.key = key;
    list.forEach((info, row) => this.cars.push(drawnCar(row, false, info.body % CARS.length, info.paint % CAR_PAINT.length, info.x, info.z, info.yaw)));
    this.scanned = list.length;
    this.ensureAround('car', list.length, 0.55);
    this.around = true;
  }

  /** Each car where the snapshot has it (between the last two), its doors, wheels and tilt, and how it's drawn. */
  update(frame: FrameState): void {
    const { curr, layout } = frame;
    const f = layout.car;
    // (Doors swing in the sim's real time, as it has them: faster at higher speeds, still while paused.)
    const rate = PLAYBACK_RATE[curr[layout.header.speed] | 0] ?? 1;
    const real = Math.min(0.1, Math.max(0, (frame.now - this.lastNow) / 1000));
    const dt = real * rate;
    // (Seconds of drive time a real second: the cars go on the traffic's clock.)
    const drive = rate * layout.driveScale;
    this.lastNow = frame.now;
    this.moved = false;
    if (layout.carBase === undefined || layout.carStride === undefined || !f) return;
    this.pose(frame);
    const step = dt / DOOR_SWING;
    const doorsAt = f.doors;
    for (let i = 0; i < this.cars.length; i++) {
      const car = this.cars[i];
      const { at, last } = car;
      // (Where it was drawn last frame, its shadow in the sun's map with it: `castFrom`.)
      [car.castFrom.x, car.castFrom.z] = [last.x, last.z];
      // (Into town or out of it, parked or off driving: drawn another way, `restate`.)
      const shown = at.shown !== last.shown;
      last.shown = at.shown;
      let changed = false;
      if (at.shown) {
        const o = layout.carBase + car.row * layout.carStride;
        changed = at.x !== last.x || at.z !== last.z || at.yaw !== last.yaw;
        // (Paused, nothing goes on settling.)
        if (changed) this.roll(car, real, drive);
        else if (rate > 0) changed = this.settle(car, real, drive);
        [last.x, last.z, last.yaw] = [at.x, at.z, at.yaw];
        const open = doorsAt === undefined ? 0 : curr[o + doorsAt];
        let ajar = false;
        for (let d = 0; d < DOORS; d++) {
          const target = open & (1 << d) ? 1 : 0;
          if (car.open[d] !== target) {
            car.open[d] = target > car.open[d] ? Math.min(1, car.open[d] + step) : Math.max(0, car.open[d] - step);
            changed = true;
          }
          if (car.open[d] > 0) ajar = true;
        }
        car.ajar = ajar;
      }
      if (changed && car.drawn) {
        this.stirred.push(car);
        if (this.castsMoved(car)) this.moved = true;
      }
      if (changed || shown) this.restate(car);
    }
    this.flush();
  }

  /** As `Vehicles.hideOn`: a car driving off the lot shows as it leaves. */
  override hideOn(rect: readonly [number, number, number, number] | null): void {
    super.hideOn(rect);
    this.restateAll();
  }

  override setView(chunks: ReadonlySet<number> | null, x: number, z: number): void {
    super.setView(chunks, x, z);
    this.restateAll();
  }

  /**
   * Each car in town the near drawing leaves out (outside the chunks streamed in, or far off): where it is this frame,
   * which way it faces (rad, 0 along +z), its paint (sRGB) and whether it drives; for the town view's movers (`townView.ts`).
   */
  forEachFar(fn: (x: number, z: number, yaw: number, paint: string, taxi: boolean, driving: boolean) => void): void {
    for (const c of this.cars) if (c.at.shown && !c.drawn) fn(c.at.x, c.at.z, c.at.yaw, c.taxi ? TAXI_PAINT : CAR_PAINT[c.paint % CAR_PAINT.length], c.taxi, c.at.shown === 2);
  }

  /** The seat of a car at (x, z) where it's drawn (someone sat in it, parked or driving), else null: for `Residents.setCarSeat`. */
  seatAt(x: number, z: number): Seat | null {
    for (const { shape, at } of this.cars) if (at.shown && inCar(shape, at, x, z)) return CAR_SEAT;
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
    for (const { shape, snap, at } of aboard ? this.cars : this.driving) {
      if (at.shown && carriedAlong(shape, snap, at, x, z, yaw, out)) return true;
    }
    return false;
  }

  clear(): void {
    for (const c of this.cars) c.own?.entity.destroy();
    this.cars = [];
    this.stirred.length = 0;
    this.driving.length = 0;
    for (const p of this.parked.values()) p.draw.destroy();
    this.parked.clear();
    this.clearDraws();
    this.key = '';
    this.scanned = 0;
    this.posedAt = NaN;
  }

  /** Where each car is drawn this frame (between the last two snapshots), once a frame: the residents riding in them ask before `update`. */
  private pose(frame: FrameState): void {
    const { prev, curr, alpha, layout, now } = frame;
    if (now === this.posedAt && curr === this.posedCurr) return;
    this.posedAt = now;
    this.posedCurr = curr;
    this.driving.length = 0;
    const base = layout.carBase;
    const stride = layout.carStride;
    const f = layout.car;
    if (base === undefined || stride === undefined || !f) return;
    const count = layout.header.carCount === undefined ? 0 : curr[layout.header.carCount];
    const before = layout.header.carCount === undefined ? 0 : prev[layout.header.carCount];
    // A taxi's row come (after the households' and the taxis' before it): drawn as the taxi from now on.
    for (; this.scanned < count; this.scanned++) {
      const row = this.scanned;
      if (!carRowIsTaxi(curr, layout, row)) continue;
      const o = base + row * stride;
      this.cars.push(drawnCar(row, true, TAXI_BODY, -1, curr[o + f.x], curr[o + f.z], curr[o + f.yaw]));
      this.ensureAround('car', this.cars.length, 0.55);
      this.around = true;
    }
    for (let i = 0; i < this.cars.length; i++) {
      const { at, snap, row, taxi } = this.cars[i];
      const o = base + row * stride;
      // (A row not what it was taken for, as a household's car whose structure is a step behind: not shown.)
      at.shown = row < count && carRowIsTaxi(curr, layout, row) === taxi ? curr[o + f.shown] : 0;
      if (!at.shown) continue;
      const was = row < before ? prev[o + f.shown] : 0;
      // Driving in both, on past the newest snapshot while the next is late (as far as the residents
      // go: the bridge's `MAX_EXTRAPOLATION`), or it would stall and then jump; parking, not past its spot.
      const k = !was ? 1 : at.shown === 2 && was === 2 ? Math.max(0, alpha) : Math.min(1, Math.max(0, alpha));
      const [cx, cz, cyaw] = [curr[o + f.x], curr[o + f.z], curr[o + f.yaw]];
      at.x = prev[o + f.x] + (cx - prev[o + f.x]) * k;
      at.z = prev[o + f.z] + (cz - prev[o + f.z]) * k;
      at.yaw = lerpAngle(prev[o + f.yaw], cyaw, k);
      [snap.x, snap.z, snap.yaw] = [cx, cz, cyaw];
      snap.driving = at.shown === 2 || was === 2;
      if (snap.driving) this.driving.push(this.cars[i]);
    }
  }

  /**
   * A car moved since last frame (`real` seconds, `drive` seconds of drive time each): its wheels turn by the way it
   * went, the front ones steer by how it turned, the body tilts by how its speed changed.
   */
  private roll(car: Drawn, real: number, drive: number): void {
    const along = rollWheels(car, car.at, car.last, car.shape.wheelR, car.shape.wheelBase, STEER, real, drive);
    if (along === null) car.speed = 0;
    else if (real > 0 && drive > 0) this.tilt(car, along / (real * drive), real, drive);
  }

  /** A car standing still this frame: its tilt settles, its wheels straighten once parked. Whether either changed. */
  private settle(car: Drawn, real: number, drive: number): boolean {
    let changed = false;
    if (car.at.shown === 1 && car.steer !== 0) {
      car.steer = Math.abs(car.steer) < 0.01 ? 0 : car.steer * Math.exp(-real * STEER.rate);
      changed = true;
    }
    if (car.speed !== 0 || car.pitch !== 0) changed = this.tilt(car, 0, real, drive) || changed;
    return changed;
  }

  /** The body's tilt for its speed now (`speed`, m a second of drive time): nose up speeding up, down slowing down. Whether it changed. */
  private tilt(car: Drawn, speed: number, real: number, drive: number): boolean {
    if (real <= 0 || drive <= 0) return false;
    const was = car.speed;
    car.speed += (speed - car.speed) * (1 - Math.exp(-real * SPEED_RATE));
    if (Math.abs(car.speed) < 0.01 && speed === 0) car.speed = 0;
    const accel = (Math.abs(car.speed) - Math.abs(was)) / (real * drive);
    const goal = Math.max(-PITCH_MAX, Math.min(PITCH_MAX, accel * PITCH_PER_ACCEL));
    const pitch = car.pitch;
    car.pitch += (goal - car.pitch) * (1 - Math.exp(-real * PITCH_RATE));
    if (Math.abs(car.pitch) < 1e-4 && goal === 0) car.pitch = 0;
    return car.pitch !== pitch;
  }

  /** How a car is drawn now (by its own entity, in its look's parked draw, or not at all), its own entity placed. */
  private restate(car: Drawn): void {
    const drawn = this.visible(car.at);
    const owned = drawn && (car.at.shown === 2 || car.pitch !== 0 || car.ajar);
    if (drawn !== car.drawn || owned !== car.owned) {
      // (In or out of a parked draw; came or went, the wheels and shadows with it.)
      const parked = (car.drawn && !car.owned) || (drawn && !owned) ? this.parkedFor(car) : null;
      if (parked) parked.dirty = true;
      if (drawn !== car.drawn) this.around = true;
      if (this.castsMoved(car)) this.moved = true;
      car.drawn = drawn;
      car.owned = owned;
    }
    if (owned) this.placeOwn(car);
    else if (car.own?.entity.enabled) car.own.entity.enabled = false;
  }

  /**
   * Whether a car that moved, came or went changes the sun's map: where it is now casts into it, or where it was
   * (`castFrom`: its shadow is left there otherwise, as one at the fastest speed goes tens of metres in a frame, from
   * in the map to out of it).
   */
  private castsMoved(car: Drawn): boolean {
    return this.shadows && (this.inShadows(car.at.x, car.at.z) || this.inShadows(car.castFrom.x, car.castFrom.z));
  }

  /** Every car `restate`d (what's hidden or near changed), and the draws written again where that changed. */
  private restateAll(): void {
    for (const car of this.cars) this.restate(car);
    this.flush();
  }

  /** The parked draws, wheels and contact shadows written again where they changed. */
  private flush(): void {
    for (const [look, p] of this.parked) {
      if (!p.dirty) continue;
      p.dirty = false;
      let n = 0;
      for (const car of this.cars) if (car.look === look && car.drawn && !car.owned) n++;
      p.draw.setCount(n);
      n = 0;
      for (const car of this.cars) {
        if (car.look !== look || !car.drawn || car.owned) continue;
        p.draw.setMatrix(n++, this.carMatrix(car.at));
      }
      p.draw.update();
      p.draw.enabled = n > 0;
    }
    this.writeAround();
  }

  /** The drawn cars' wheels and contact shadows written again: all of them when one came or went (`around`), else those that moved (`stirred`). */
  private writeAround(): void {
    const wheels = this.wheels;
    const shadows = this.shadowDraw;
    if (this.around) {
      this.around = false;
      let n = 0;
      for (const car of this.cars) car.slot = car.drawn ? n++ : -1;
      wheels?.setCount(n * 4);
      shadows?.setCount(n);
      for (const car of this.cars) if (car.drawn) this.writeCar(car);
    } else if (this.stirred.length) {
      for (const car of this.stirred) this.writeCar(car);
    } else return;
    this.stirred.length = 0;
    this.sendAround();
  }

  /** A drawn car's four wheels and its contact shadow, at its `slot`. */
  private writeCar(car: Drawn): void {
    this.writeAt(car.slot, car.at, car.shape, WHEELS[car.body], car);
  }

  /** A parked car's body in game space (turned about y). */
  private carMatrix(at: Drawn['at']): Float32Array {
    compose(1, 1, 1, quatYawPitchRoll(at.yaw, 0, 0, this.q), at.x, 0, at.z, this.m);
    return this.m;
  }

  /** A car drawn by its own entity, placed, tilted, its doors swung, in one piece while they're shut. */
  private placeOwn(car: Drawn): void {
    const own = (car.own ??= this.makeOwn(car));
    if (!own.entity.enabled) own.entity.enabled = true;
    own.entity.setLocalPosition(car.at.x, 0, car.at.z);
    own.entity.setLocalEulerAngles(0, (car.at.yaw * 180) / Math.PI, 0);
    // (Nose up: turned back about x.)
    own.tilt.setLocalEulerAngles((-car.pitch * 180) / Math.PI, 0, 0);
    // The rear edge swings out (`paintDoor`).
    for (let d = 0; d < DOORS; d++) own.doors[d].setLocalEulerAngles(0, -car.sides[d] * DOOR_OPEN * ease(car.open[d]), 0);
    const shut = !car.ajar;
    if (own.whole.enabled !== shut) {
      own.whole.enabled = shut;
      own.apart.enabled = !shut;
    }
  }

  /** A car's own entity (the first time it drives or opens a door): its body in one piece, and apart with its doors. */
  private makeOwn(car: Drawn): Own {
    const { shape, paint } = car;
    const hex = car.taxi ? '' : CAR_PAINT[paint];
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
    const bare = this.mesh(`body ${car.look}`, () => {
      const p = new Painter(new Geo());
      if (car.taxi) paintTaxi(p.at(0, 0, 0, LIFT), true, false);
      else paintCar(p.at(0, 0, 0, LIFT), shape, hex, true, false);
      return p.geo;
    });
    if (bare) drawMesh(apart, 'carBody', bare, this.lib.plainFinish(), { castShadows: this.shadows });
    const doors: pc.Entity[] = [];
    for (let d = 0; d < DOORS; d++) {
      const { side, hinge } = carDoor(shape, d);
      const mesh = this.mesh(`door ${car.look} ${d}`, () => {
        const p = new Painter(new Geo());
        if (car.taxi) paintTaxiDoor(p.at(0, 0, 0, LIFT), d);
        else paintDoor(p.at(0, 0, 0, LIFT), shape, hex, d);
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
    return this.mesh(`whole ${car.look}`, () => {
      const p = new Painter(new Geo());
      if (car.taxi) paintTaxi(p.at(0, 0, 0, LIFT), false, false);
      else paintCar(p.at(0, 0, 0, LIFT), car.shape, CAR_PAINT[car.paint], false, false);
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

}

/** A car to draw: row `row` of the snapshot's cars, a household's (`body`, `paint`) or a work taxi, where it first stands. */
function drawnCar(row: number, taxi: boolean, body: number, paint: number, x: number, z: number, yaw: number): Drawn {
  const shape = CARS[body];
  return {
    row,
    taxi,
    shape,
    look: taxi ? 'taxi' : `${body} ${paint}`,
    body,
    paint,
    at: { x, z, yaw, shown: 0 },
    last: { x, z, yaw, shown: 0 },
    snap: { x, z, yaw, driving: false },
    castFrom: { x, z },
    open: new Array<number>(DOORS).fill(0),
    sides: Array.from({ length: DOORS }, (_, d) => carDoor(shape, d).side),
    ajar: false,
    own: null,
    drawn: false,
    owned: false,
    slot: -1,
    spin: 0,
    steer: 0,
    pitch: 0,
    speed: 0,
  };
}
