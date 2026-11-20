/**
 * Households' cars as the street draws them: their bodies (made of the street's plain-finish
 * primitives, `paintCar`), their paints, and where a house's car parks on its driveway
 * (`parkCar`), the same for the town as it is made (`game/town.ts`, the sim's `cars`), the menus'
 * town and the game, so a car stands where the town showed it. The work taxi's look too
 * (`paintTaxi`: no household's, so its own), and which of the snapshot's car rows is which
 * (`carRowAt`, `carRowIsTaxi`).
 *
 * Engine-neutral: the street (`street.ts`) and the moving cars (`pc/cars.ts`) draw them.
 */

import type { SnapshotLayout } from '../../core/snapshot';
import { wrapAngle } from './characters/rig';
import { facing, type V3 } from './geometry';
import type { Painter, Rect } from './street';

/** Car paints. */
export const CAR_PAINT = ['#C9CDD2', '#F0F0EC', '#1E2328', '#2B4A7A', '#9E2A2B', '#56684C', '#7C8288', '#C8B48E', '#6E8FB0', '#5B2333'];

/** A number 0..1 from a spot and a salt: the same each time (what the street and the cars look like). */
export function hash(x: number, z: number, salt: number): number {
  let h = (Math.imul(Math.round(x * 10) | 0, 374761393) + Math.imul(Math.round(z * 10) | 0, 668265263) + Math.imul(salt, 2147483647)) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

/** One of `list`, by `hash`. */
export function pick<T>(list: readonly T[], x: number, z: number, salt: number): T {
  return list[Math.floor(hash(x, z, salt) * list.length) % list.length];
}

// ---- cars ------------------------------------------------------------------------------

export interface CarBody {
  /** As content's `carBodies` (`careers.json`), in the same order: a car's body is its index. */
  id: string;
  len: number;
  wid: number;
  wheelR: number;
  wheelBase: number;
  /**
   * Side profile of the lower body (z forward, y up), convex, counter-clockwise; its first two
   * points the sills, at the ground clearance (a saloon's 0.13–0.16 m, a compact SUV's 0.2 m, and
   * a little for the sills).
   */
  low: [number, number][];
  /** Greenhouse: base rear z, base front z, roof rear z, roof front z, belt y, roof y. */
  cabin: [number, number, number, number, number, number];
  /** How far in from the sides the roof is (m; absent 0.25): a boxy van's or off-roader's less. */
  roofIn?: number;
  /** Along the side window (0 its front, 1 its back): where the front door ends, the back door begins and ends (absent: about halves, `DOOR_SPLIT`). */
  doors?: [number, number, number];
  /** The side windows glazed only so far back (0..1 of the side window): a van's back is panelled. */
  sideGlass?: number;
  /** No back window (a van's back doors are panelled). */
  blindRear?: boolean;
  /** An open load bed from z0 to z1 behind the cab (a pickup's). */
  bed?: [number, number];
  /** A spare wheel on its back (an off-roader's) and a roof rack. */
  spare?: boolean;
  rack?: boolean;
  /** Bright trim (an executive saloon's): a chrome line under its windows and a chrome grille, whatever its age. */
  chrome?: boolean;
  /** The headlights' foot (m; absent: a little over the wheels' tops), set into its nose and tail wherever they are at that height. */
  lights?: number;
  /** A dark rubbing strip along its sides from this height (m), between the wheels' arches (a van's, an off-roader's). */
  strip?: number;
}

/** `CarBody.doors` when a body says none. */
const DOOR_SPLIT: [number, number, number] = [0.505, 0.535, 0.97];

/**
 * How old a car is (sim-core `content::CarAge`, by the grade of its owner's job): 0 old (its paint faded and dull,
 * its bumpers grey), 1 used (as every car was before), 2 new (glossy, bumpers in its paint, a chrome line under its
 * windows).
 */
export type CarAge = 0 | 1 | 2;
export const CAR_AGES = 3;
/** A car's age as the structure gives it (`CarInfo.age`; absent or out of range: used). */
export function carAge(age: number | undefined): CarAge {
  return age === 0 || age === 2 ? age : 1;
}

/** The paint's roughness by age, and how far an old car's has faded (towards `FADED`). */
const AGE_GLOSS = [0.46, 0.2, 0.09];
const FADE = 0.2;
const FADED = '#B9B6AC';
const CHROME = '#C9CED3';

export const CARS: CarBody[] = [
  // Saloon.
  { id: 'saloon', len: 4.4, wid: 1.78, wheelR: 0.32, wheelBase: 2.65, low: [[-2.2, 0.17], [2.2, 0.17], [2.24, 0.52], [2.14, 0.74], [0.95, 0.88], [-1.35, 0.93], [-2.12, 0.88], [-2.24, 0.58]], cabin: [-1.4, 0.95, -0.8, 0.16, 0.9, 1.42] },
  // Hatchback.
  { id: 'hatchback', len: 4.0, wid: 1.74, wheelR: 0.31, wheelBase: 2.5, low: [[-1.98, 0.17], [2.0, 0.17], [2.04, 0.52], [1.96, 0.72], [0.85, 0.87], [-1.9, 0.93], [-2.02, 0.6]], cabin: [-1.9, 0.88, -1.72, 0.08, 0.9, 1.46] },
  // SUV.
  { id: 'suv', len: 4.6, wid: 1.88, wheelR: 0.37, wheelBase: 2.75, low: [[-2.3, 0.23], [2.3, 0.23], [2.34, 0.66], [2.24, 0.92], [1.25, 1.02], [-2.25, 1.06], [-2.34, 0.7]], cabin: [-2.24, 1.28, -2.14, 0.48, 1.02, 1.76] },
  // Estate.
  { id: 'estate', len: 4.6, wid: 1.8, wheelR: 0.32, wheelBase: 2.7, low: [[-2.3, 0.17], [2.3, 0.17], [2.34, 0.52], [2.24, 0.74], [1.05, 0.88], [-2.25, 0.93], [-2.33, 0.6]], cabin: [-2.24, 1.05, -2.1, 0.25, 0.9, 1.45] },
  // Van: a short bonnet, a tall box, glass only in the cab and at the front, panelled behind (a trade's).
  {
    id: 'van',
    len: 5.0,
    wid: 1.98,
    wheelR: 0.34,
    wheelBase: 2.95,
    low: [[-2.44, 0.2], [2.4, 0.2], [2.5, 0.5], [2.46, 0.86], [1.5, 1.08], [-2.46, 1.12], [-2.5, 0.55]],
    cabin: [-2.46, 1.5, -2.42, 0.75, 1.08, 1.98],
    roofIn: 0.12,
    doors: [0.39, 0.41, 0.7],
    sideGlass: 0.39,
    blindRear: true,
    lights: 0.66,
    strip: 0.42,
  },
  // Pickup: a double cab and an open load bed behind it, high on its wheels.
  {
    id: 'pickup',
    len: 5.2,
    wid: 1.86,
    wheelR: 0.38,
    wheelBase: 3.08,
    low: [[-2.55, 0.3], [2.5, 0.3], [2.6, 0.6], [2.55, 0.95], [1.35, 1.1], [-2.58, 1.1], [-2.6, 0.62]],
    cabin: [-0.95, 1.3, -0.9, 0.55, 1.1, 1.82],
    roofIn: 0.2,
    bed: [-2.5, -1.0],
    lights: 0.7,
  },
  // Off-roader: boxy and upright, high off the ground, a spare wheel on its back and a rack on its roof.
  {
    id: 'offRoader',
    len: 4.6,
    wid: 1.9,
    wheelR: 0.39,
    wheelBase: 2.75,
    low: [[-2.05, 0.34], [2.18, 0.34], [2.3, 0.58], [2.28, 0.96], [1.3, 1.1], [-2.08, 1.12], [-2.1, 0.6]],
    cabin: [-2.08, 1.3, -2.06, 1.0, 1.12, 1.95],
    roofIn: 0.1,
    doors: [0.42, 0.45, 0.78],
    spare: true,
    rack: true,
    lights: 0.72,
    strip: 0.44,
  },
  // Executive saloon: long and low, a long bonnet, a sloping back, bright trim.
  {
    id: 'executive',
    len: 4.9,
    wid: 1.86,
    wheelR: 0.34,
    wheelBase: 2.97,
    low: [[-2.4, 0.16], [2.38, 0.16], [2.45, 0.5], [2.36, 0.7], [1.2, 0.85], [-1.6, 0.9], [-2.36, 0.86], [-2.45, 0.56]],
    cabin: [-1.75, 1.15, -0.95, 0.12, 0.88, 1.43],
    roofIn: 0.26,
    chrome: true,
    lights: 0.54,
  },
];

/** The bodies a town gives a household's car before anyone's job says otherwise (`parkCar`): the first four, the family cars. */
export const FAMILY_CARS = 4;

/** `hex` mixed with `with` by `t` (0: `hex`). */
function mix(hex: string, withHex: string, t: number): string {
  const a = parseInt(hex.slice(1), 16);
  const b = parseInt(withHex.slice(1), 16);
  const c = [16, 8, 0].map((sh) => Math.round(((a >> sh) & 255) * (1 - t) + ((b >> sh) & 255) * t));
  return `#${c.map((v) => v.toString(16).padStart(2, '0')).join('')}`;
}

/** A car's paint as its age shows it: its colour and roughness. */
function aged(paint: string, age: CarAge): [string, number] {
  return [age === 0 ? mix(paint, FADED, FADE) : paint, AGE_GLOSS[age] ?? AGE_GLOSS[1]];
}

/** Doors (as sim-core `cars::DOORS`): 0 front left (the driver's, on the car's +x side), 1 front right, 2 back left, 3 back right. */
export const DOORS = 4;
/**
 * How wide a door opens (degrees), wide open; the room (m) beside a car's side it needs for that (sim-core
 * `cars::SWING`); and how far out from the side (m) a door opened only part way reaches, where a driver squeezes in or
 * out by it (`cars::DOOR_OUT`: `Car::door_in_bay`, a car in a full row of bays).
 */
export const DOOR_OPEN = 62;
export const DOOR_SWING_ROOM = 1.2;
export const DOOR_OUT = 0.45;
/** The simulation asks its tiles (a metre each) clear as far out as `DOOR_SWING_ROOM`: a car this much further off is still in the way (m). */
const SWING_TILE = 0.3;

/** A car standing somewhere: its middle, its yaw (rad, as the sim's: its front along (sin, cos)), its body's size. */
export interface CarAt {
  x: number;
  z: number;
  yaw: number;
  len: number;
  wid: number;
}

/**
 * How wide (degrees) door `d` of `car` (its body `body`) opens with `others` standing round it: wide open
 * (`DOOR_OPEN`), unless another car beside it on that door's side leaves too little room there for that (`DOOR_SWING_ROOM`
 * as the simulation's tiles have it: a full row of bays, 3 m apart); then only part way, its rear edge `DOOR_OUT` out from the side as the sim has it, and never as
 * far as the other car.
 */
export function doorSwing(body: CarBody, d: number, car: CarAt, others: Iterable<CarAt>): number {
  const { side, hinge, rearBelt, rearLow } = carDoor(body, d);
  const long = hinge - Math.min(rearBelt, rearLow);
  const [s, c] = [Math.sin(car.yaw), Math.cos(car.yaw)];
  let gap = Infinity;
  for (const o of others) {
    if (o === car) continue;
    const [dx, dz] = [o.x - car.x, o.z - car.z];
    if (Math.abs(dx) > 8 || Math.abs(dz) > 8) continue;
    // (Its corners in this car's frame: across it, +x on the door's side, and along it.)
    const [os, oc] = [Math.sin(o.yaw), Math.cos(o.yaw)];
    let [near, v0, v1] = [Infinity, Infinity, -Infinity];
    for (const [u, v] of [[-1, -1], [1, -1], [1, 1], [-1, 1]]) {
      const [cx, cz] = [dx + (u * o.wid * oc) / 2 + (v * o.len * os) / 2, dz - (u * o.wid * os) / 2 + (v * o.len * oc) / 2];
      const across = side * (cx * c - cz * s);
      const along = cx * s + cz * c;
      near = Math.min(near, across);
      [v0, v1] = [Math.min(v0, along), Math.max(v1, along)];
    }
    // (Beside the door: on its side, and reaching along the car past it.)
    if (near > body.wid / 2 - 0.2 && v0 < hinge && v1 > hinge - long) gap = Math.min(gap, near - body.wid / 2);
  }
  if (gap >= DOOR_SWING_ROOM + SWING_TILE) return DOOR_OPEN;
  const out = Math.max(0, Math.min(DOOR_OUT, gap - 0.1));
  return Math.min(DOOR_OPEN, (Math.asin(Math.min(1, out / long)) * 180) / Math.PI);
}
/** Paint of the inside of a car, where an open door shows it. */
const INSIDE = '#26282B';

/** Where door `d` of `body` is: its side (+1: +x), its hinge (front edge, z), its rear edge at the belt line and at the sill, its sill's height, and the part of the side window it carries (`u0`..`u1`, front to back). */
export function carDoor(body: CarBody, d: number): { side: 1 | -1; hinge: number; rearBelt: number; rearLow: number; sill: number; u0: number; u1: number } {
  const [br, bf] = body.cabin;
  const [f1, r0, r1] = body.doors ?? DOOR_SPLIT;
  const side = d % 2 === 0 ? 1 : -1;
  const at = (u: number) => bf + (br - bf) * u;
  const arch = body.wheelR + 0.1;
  const sill = Math.max(...body.low.slice(0, 2).map((q) => q[1])) + 0.1;
  if (d < 2) return { side, hinge: Math.min(bf, body.wheelBase / 2 - arch), rearBelt: at(f1), rearLow: at(f1), sill, u0: 0, u1: f1 };
  return { side, hinge: at(r0), rearBelt: at(r1), rearLow: Math.max(at(r1), -body.wheelBase / 2 + arch), sill, u0: r0, u1: r1 };
}

/**
 * A parked car in `p`'s frame (front towards +z), sitting on the ground at `y`. `apart`: without
 * its doors, the inside showing where they go (each drawn on its own, `paintDoor`, to open).
 * `wheels`: with its wheels (the street's); without, their arches only, the wheels drawn on their
 * own to turn and steer (`paintWheel`, `wheelAt`: the game's cars). `age`: how old it looks (`CarAge`).
 */
export function paintCar(p: Painter, body: CarBody, paint: string, apart = false, wheels = true, age: CarAge = 1): void {
  const { wid, low } = body;
  const [colour, gloss] = aged(paint, age);
  const painted = () => p.paint(colour, gloss, 0);
  const hw = wid / 2;
  const inner = hw - 0.045;
  const [cz, cy] = [low.reduce((a, q) => a + q[0], 0) / low.length, low.reduce((a, q) => a + q[1], 0) / low.length];
  const shrunk = low.map(([z, y]): [number, number] => {
    const [dz, dy] = [cz - z, cy - y];
    const l = Math.hypot(dz, dy) || 1;
    return [z + (dz / l) * 0.035, y + (dy / l) * 0.035];
  });
  painted();
  // Sides, chamfers and the perimeter.
  for (const side of [-1, 1]) {
    p.poly(shrunk.map(([z, y]): V3 => [side * hw, y, z]), [side, 0, 0]);
  }
  for (let i = 0; i < low.length; i++) {
    const [z0, y0] = low[i];
    const [z1, y1] = low[(i + 1) % low.length];
    const l = Math.hypot(z1 - z0, y1 - y0) || 1;
    const [ny, nz] = [-(z1 - z0) / l, (y1 - y0) / l];
    if (ny < -0.9) p.paint('#202224', 0.8, 0);
    p.poly([[-inner, y0, z0], [inner, y0, z0], [inner, y1, z1], [-inner, y1, z1]], [0, ny, nz]);
    for (const side of [-1, 1]) {
      const [s0, s1] = [shrunk[i], shrunk[(i + 1) % low.length]];
      const n: V3 = [side * 0.7, ny * 0.7, nz * 0.7];
      p.poly([[side * hw, s0[1], s0[0]], [side * hw, s1[1], s1[0]], [side * inner, y1, z1], [side * inner, y0, z0]], n);
    }
    if (ny < -0.9) painted();
  }
  // Greenhouse: painted frame and roof, glass inset into it.
  const [br, bf, rr, rf, yb, yr] = body.cabin;
  const hb = hw - 0.07;
  const hr = hw - (body.roofIn ?? 0.25);
  const B = (x: number, z: number): V3 => [x, yb, z];
  const R = (x: number, z: number): V3 => [x, yr, z];
  const [f1, r0] = body.doors ?? DOOR_SPLIT;
  const sideGlass = body.sideGlass ?? 1;
  const quad = (q: V3[], out: V3, glass: [number, number, number, number] | null, pillar: number | null = null) => {
    painted().poly(q, out);
    if (!glass) return;
    const [u0, u1, v0, v1] = glass;
    // Bilinear sub-rectangle of the quad (q0..q1 along the bottom, q3..q2 along the top).
    const at = (u: number, v: number): V3 => {
      const b = [0, 1, 2].map((k) => q[0][k] + (q[1][k] - q[0][k]) * u);
      const t = [0, 1, 2].map((k) => q[3][k] + (q[2][k] - q[3][k]) * u);
      const len = Math.hypot(...out) || 1;
      const n = [0, 1, 2].map((k) => b[k] + (t[k] - b[k]) * v + (out[k] / len) * 0.006);
      return [n[0], n[1], n[2]];
    };
    p.paint('#1C252D', 0.06, 0).poly([at(u0, v0), at(u1, v0), at(u1, v1), at(u0, v1)], out);
    if (pillar !== null) {
      const mid = pillar;
      const lift = (v: V3): V3 => {
        const len = Math.hypot(...out) || 1;
        return [v[0] + (out[0] / len) * 0.004, v[1] + (out[1] / len) * 0.004, v[2] + (out[2] / len) * 0.004];
      };
      painted().poly([lift(at(mid - 0.025, 0)), lift(at(mid + 0.025, 0)), lift(at(mid + 0.025, 1)), lift(at(mid - 0.025, 1))], out);
    }
  };
  const ws = facingOut([B(-hb, bf), B(hb, bf), R(hr, rf)], [0, 0.6, 1]);
  quad([B(-hb, bf), B(hb, bf), R(hr, rf), R(-hr, rf)], ws, [0.05, 0.95, 0.06, 0.9]);
  const rw = facingOut([B(hb, br), B(-hb, br), R(-hr, rr)], [0, 0.6, -1]);
  quad([B(hb, br), B(-hb, br), R(-hr, rr), R(hr, rr)], rw, body.blindRear ? null : [0.06, 0.94, 0.1, 0.88]);
  for (const side of [-1, 1]) {
    const q: V3[] = side > 0 ? [B(hb, bf), B(hb, br), R(hr, rr), R(hr, rf)] : [B(-hb, br), B(-hb, bf), R(-hr, rf), R(-hr, rr)];
    if (!apart) {
      // (Glazed from the front as far as `sideGlass`: on the left side the quad runs back to front.)
      // (Its B-pillar between the doors, where the glass runs past it: mirrored on the left, where the quad runs back
      // to front, for a body with doors of its own; the family cars' as they always were, 4 cm off on that side.)
      const [g0, g1] = side > 0 ? [0.05, Math.min(0.95, sideGlass)] : [Math.max(0.05, 1 - sideGlass), 0.95];
      const split = (f1 + r0) / 2;
      quad(q, facingOut(q, [side, 0.3, 0]), [g0, g1, 0.1, 0.86], sideGlass > split ? (side < 0 && body.doors ? 1 - split : split) : null);
      continue;
    }
    // The doors carry the side windows: the inside behind them.
    const n = facingOut(q, [side, 0.3, 0]);
    painted().poly(q, n);
    p.paint(INSIDE, 0.9, 0).poly(q.map(([x, y, z]): V3 => [x + n[0] * 0.003, y + n[1] * 0.003, z + n[2] * 0.003]), n);
    // (Panelled behind the doors' glass: a van's back, which no door carries.)
    const back = body.doors?.[2] ?? DOOR_SPLIT[2];
    if (sideGlass < 1 && back < 1) {
      const at = (u: number, v: number): V3 => {
        const z = bf + (br - bf) * u + (rf + (rr - rf) * u - (bf + (br - bf) * u)) * v;
        return [side * (hb + (hr - hb) * v + 0.004), yb + (yr - yb) * v, z];
      };
      painted().poly(side > 0 ? [at(back, 0), at(1, 0), at(1, 1), at(back, 1)] : [at(1, 0), at(back, 0), at(back, 1), at(1, 1)], n);
    }
  }
  if (apart) {
    p.paint(INSIDE, 0.9, 0);
    for (let d = 0; d < DOORS; d++) {
      const { side, hinge, rearBelt, rearLow, sill } = carDoor(body, d);
      const x = side * (hw + 0.002);
      p.poly([[x, sill, hinge], [x, yb, hinge], [x, yb, rearBelt], [x, sill, rearLow]], [side, 0, 0]);
    }
  }
  painted().poly([R(-hr, rr), R(hr, rr), R(hr, rf), R(-hr, rf)], [0, 1, 0]);
  // Wheels in dark arches, hubcaps.
  const r = body.wheelR;
  for (const wz of [body.wheelBase / 2, -body.wheelBase / 2]) {
    for (const side of [-1, 1]) {
      p.paint('#141517', 0.9, 0).tube('x', side * (hw + 0.002), side * (hw + 0.003), r, wz, r + 0.07, r + 0.07, 8, true, 0, Math.PI);
      if (!wheels) continue;
      p.paint('#1A1A1C', 0.85, 0).tube('x', side * (hw - 0.22), side * (hw + 0.012), r, wz, r, r, 12);
      p.paint('#B4B8BC', 0.3, 1).tube('x', side * (hw + 0.012), side * (hw + 0.02), r, wz, r * 0.6, r * 0.5, 10);
    }
  }
  // Bumpers, lights, grille, plates, mirrors.
  const zf = Math.max(...low.map((q) => q[0]));
  const zb = Math.min(...low.map((q) => q[0]));
  // Bumpers and lights by the wheels' height, not the sills'. A new car's in its paint, an old one's weathered grey.
  const yl = body.wheelR - 0.02;
  // (Lights at a height of their own stand on the nose and the tail where they are at that height.)
  const ly = body.lights ?? yl + 0.24;
  const [lf, lb] = body.lights === undefined ? [zf - 0.02, zb + 0.1] : [Math.max(faceZ(low, ly, 1), faceZ(low, ly + 0.18, 1)) + 0.02, Math.min(faceZ(low, ly, -1), faceZ(low, ly + 0.18, -1)) + 0.06];
  if (age === 2) painted();
  else p.paint(age === 0 ? '#45474A' : '#2A2C2E', age === 0 ? 0.85 : 0.6, 0);
  p.box(-hw + 0.04, yl - 0.02, zf - 0.12, hw - 0.04, yl + 0.16, zf + 0.04);
  p.box(-hw + 0.04, yl - 0.02, zb - 0.04, hw - 0.04, yl + 0.16, zb + 0.12);
  const bright = body.chrome || age === 2;
  if (bright) p.paint(CHROME, 0.18, 1);
  else p.paint('#2A2C2E', 0.6, 0);
  p.box(-0.42, ly + 0.02, lf - 0.06, 0.42, ly + 0.12, lf + 0.02);
  p.paint('#F5F3EA', 0.1, 0);
  for (const s of [-1, 1]) p.box(s * (hw - 0.34), ly, lf - 0.08, s * (hw - 0.08), ly + 0.12, lf);
  p.paint('#A3161E', 0.2, 0);
  for (const s of [-1, 1]) p.box(s * (hw - 0.36), ly + 0.04, lb - 0.08, s * (hw - 0.06), ly + 0.18, lb);
  p.paint('#E9E6D8', 0.4, 0);
  p.box(-0.26, yl + 0.02, zf + 0.04, 0.26, yl + 0.13, zf + 0.06);
  p.box(-0.26, yl + 0.2, zb - 0.02, 0.26, yl + 0.31, zb);
  painted();
  for (const s of [-1, 1]) p.box(Math.min(s * hw, s * (hw + 0.12)), yb + 0.04, bf - 0.32, Math.max(s * hw, s * (hw + 0.12)), yb + 0.14, bf - 0.2);
  // A chrome line under the side windows (bright trim, a new car).
  if (bright && !apart) {
    p.paint(CHROME, 0.18, 1);
    for (const s of [-1, 1]) p.box(Math.min(s * hw, s * (hw + 0.008)), yb - 0.035, br + 0.05, Math.max(s * hw, s * (hw + 0.008)), yb - 0.005, bf - 0.05, s > 0 ? 'nx' : 'px');
  }
  extras(p, body, zb, zf, yr, hr);
}

/** How far out its side profile `low` reaches at height `y`, at its nose (`end` 1: the most z) or its tail (-1: the least). */
function faceZ(low: CarBody['low'], y: number, end: 1 | -1): number {
  let out = end > 0 ? -Infinity : Infinity;
  for (let i = 0; i < low.length; i++) {
    const [[z0, y0], [z1, y1]] = [low[i], low[(i + 1) % low.length]];
    if ((y < Math.min(y0, y1)) || y > Math.max(y0, y1) || y0 === y1) continue;
    const z = z0 + ((z1 - z0) * (y - y0)) / (y1 - y0);
    out = end > 0 ? Math.max(out, z) : Math.min(out, z);
  }
  return Number.isFinite(out) ? out : end > 0 ? Math.max(...low.map((q) => q[0])) : Math.min(...low.map((q) => q[0]));
}

/** What some bodies have besides the rest (`CarBody`): a pickup's load bed, an off-roader's spare wheel and roof rack, a van's back doors' seam. */
function extras(p: Painter, body: CarBody, zb: number, zf: number, yr: number, hr: number): void {
  const hw = body.wid / 2;
  if (body.strip !== undefined) {
    // Between the arches and beyond them, a hand's breadth high.
    const [y0, y1] = [body.strip, body.strip + 0.09];
    const [a, r] = [body.wheelBase / 2, body.wheelR + 0.1];
    const runs: [number, number][] = [[zb + 0.18, -a - r], [-a + r, a - r], [a + r, zf - 0.18]];
    p.paint('#232527', 0.85, 0);
    for (const s of [-1, 1]) {
      for (const [z0, z1] of runs) if (z1 - z0 > 0.1) p.box(Math.min(s * hw, s * (hw + 0.012)), y0, z0, Math.max(s * hw, s * (hw + 0.012)), y1, z1, s > 0 ? 'nx' : 'px');
    }
  }
  if (body.bed) {
    // The bed's floor and its inner sides, dark, sunk into the body's top; the tailgate's top edge painted.
    const [z0, z1] = body.bed;
    const top = Math.max(...body.low.filter(([z]) => z <= z1 + 0.05).map((q) => q[1]));
    const [x0, x1] = [-hw + 0.09, hw - 0.09];
    p.paint('#2B2D2F', 0.9, 0);
    p.poly([[x0, top + 0.004, z0], [x1, top + 0.004, z0], [x1, top + 0.004, z1], [x0, top + 0.004, z1]], [0, 1, 0]);
    p.paint('#1E2022', 0.95, 0);
    p.poly([[x0 + 0.03, top + 0.006, z0 + 0.06], [x1 - 0.03, top + 0.006, z0 + 0.06], [x1 - 0.03, top + 0.006, z1 - 0.03], [x0 + 0.03, top + 0.006, z1 - 0.03]], [0, 1, 0]);
    // (A ribbed floor, seen from above as lines along it.)
    p.paint('#34373A', 0.8, 0);
    for (let x = x0 + 0.2; x < x1 - 0.12; x += 0.22) p.poly([[x, top + 0.008, z0 + 0.1], [x + 0.05, top + 0.008, z0 + 0.1], [x + 0.05, top + 0.008, z1 - 0.06], [x, top + 0.008, z1 - 0.06]], [0, 1, 0]);
    // The seam between the cab and the bed down each side, and the tailgate's across the back.
    p.paint('#1E2022', 0.8, 0);
    const sill = body.low[0][1] + 0.12;
    for (const s of [-1, 1]) p.box(Math.min(s * hw, s * (hw + 0.006)), sill, z1 + 0.02, Math.max(s * hw, s * (hw + 0.006)), top - 0.02, z1 + 0.05, s > 0 ? 'nx' : 'px');
    const tail = faceZ(body.low, top - 0.15, -1);
    p.box(-hw + 0.1, top - 0.1, tail - 0.006, hw - 0.1, top - 0.08, tail + 0.01, 'pz');
  }
  if (body.spare) {
    // On the back door, at the belt's height: a tyre and its cover.
    const y = body.cabin[4] - 0.08;
    const r = body.wheelR * 0.92;
    p.paint('#1A1A1C', 0.85, 0).tube('z', zb - 0.2, zb + 0.02, 0, y, r, r, 14);
    p.paint('#2A2C2E', 0.6, 0).tube('z', zb - 0.215, zb - 0.2, 0, y, r * 0.62, r * 0.62, 12);
  }
  if (body.rack) {
    // Two rails along the roof on feet, and three bars across.
    const [rr, rf] = [body.cabin[2], body.cabin[3]];
    p.paint('#1E2022', 0.7, 0);
    for (const s of [-1, 1]) {
      const x = s * (hr - 0.06);
      p.box(x - 0.025, yr + 0.06, rr + 0.12, x + 0.025, yr + 0.1, rf - 0.12, 'ny');
      for (const z of [rr + 0.18, rf - 0.18]) p.box(x - 0.02, yr, z - 0.03, x + 0.02, yr + 0.06, z + 0.03, 'ny');
    }
    for (let k = 0; k < 3; k++) {
      const z = rr + 0.3 + ((rf - rr - 0.6) * k) / 2;
      p.box(-(hr - 0.06), yr + 0.07, z - 0.025, hr - 0.06, yr + 0.1, z + 0.025, 'ny');
    }
  }
  if (body.blindRear) {
    // The back doors' seam down the middle, and their handles.
    const [br, , rr, , yb] = body.cabin;
    p.paint('#1E2022', 0.8, 0);
    p.box(-0.006, body.low[0][1] + 0.2, Math.min(br, rr) - 0.008, 0.006, body.cabin[5] - 0.08, Math.min(br, rr) - 0.002, 'pz');
    p.paint('#2A2C2E', 0.5, 0);
    for (const s of [-1, 1]) p.box(s * 0.05 - 0.03, yb - 0.02, Math.min(br, rr) - 0.02, s * 0.05 + 0.03, yb + 0.02, Math.min(br, rr), 'pz');
  }
}

// ---- the work taxi ----------------------------------------------------------------------

/** The work taxi (sim-core `taxi.rs`, a car row whose `kind` is a taxi): a saloon (`CARS`) in the taxi's yellow. */
export const TAXI_BODY = 0;
export const TAXI_PAINT = '#F2B81C';
/** Its chequered band along its sides: its squares' size (m), two rows of them, their top this far under the belt line. */
const CHECK = 0.07;
const CHECK_DROP = 0.07;
/** Its roof sign: width, height, depth (m), and its colours. */
const SIGN = { w: 0.62, h: 0.17, d: 0.15 };
const SIGN_FACE = '#FFF4C8';
const INK = '#16181A';

/**
 * The work taxi in `p`'s frame, as `paintCar` paints a household's (`apart`, `wheels` as there):
 * a saloon in the taxi's yellow with a black-and-white chequered band along each side (on the
 * doors where they're apart: `paintTaxiDoor`) and a lit-looking TAXI sign across its roof,
 * lettered front and back, so it's a taxi at a glance from any side.
 */
export function paintTaxi(p: Painter, apart = false, wheels = true): void {
  const body = CARS[TAXI_BODY];
  paintCar(p, body, TAXI_PAINT, apart, wheels);
  const hw = body.wid / 2;
  const [, bf, rr, rf, yb, yr] = body.cabin;
  const zb = Math.min(...body.low.map((q) => q[0]));
  // The band from the rear quarter to the front wing, past the doors (their own when apart).
  const [from, to] = [zb + 0.3, bf + 0.25];
  for (const side of [-1, 1] as const) {
    const x = side * (hw + 0.004);
    const face = (z: number, y: number): V3 => [x, y, z];
    if (!apart) {
      checks(p, from, to, yb, face, [side, 0, 0]);
      continue;
    }
    // (Apart, the doors' spans are theirs: the band behind and in front of them.)
    const spans = [0, 1, 2, 3].filter((d) => carDoor(body, d).side === side).map((d) => doorSpan(body, d));
    const lo = Math.min(...spans.map((sp) => sp[0]));
    const hi = Math.max(...spans.map((sp) => sp[1]));
    checks(p, from, lo, yb, face, [side, 0, 0]);
    checks(p, hi, to, yb, face, [side, 0, 0]);
  }
  // The roof sign: a light box on a dark foot across the roof, TAXI on its front and back.
  const zc = (rr + rf) / 2;
  const { w, h, d } = SIGN;
  p.paint(INK, 0.6, 0).box(-w / 2 + 0.04, yr, zc - d / 2 + 0.02, w / 2 - 0.04, yr + 0.03, zc + d / 2 - 0.02, 'ny');
  p.paint(SIGN_FACE, 0.3, 0).box(-w / 2, yr + 0.03, zc - d / 2, w / 2, yr + 0.03 + h, zc + d / 2, 'ny');
  for (const end of [-1, 1] as const) {
    const z = zc + end * (d / 2 + 0.003);
    // (Read from in front of the car, its +x is on the left, as the world is drawn mirrored in z;
    // from behind, on the right: the word runs from -x at the front, +x at the back.)
    lettering(p, (u, v): V3 => [-end * u, yr + 0.03 + v, z], [0, 0, end], h);
  }
}

/** Door `d` of the taxi (`paintDoor` with its paint), its part of the chequered band on it. */
export function paintTaxiDoor(p: Painter, d: number): void {
  const body = CARS[TAXI_BODY];
  paintDoor(p, body, TAXI_PAINT, d);
  const { side, hinge } = carDoor(body, d);
  const [rear, front] = doorSpan(body, d);
  // (In the door's frame, as `paintDoor`'s: x out from the car's side, z back from the hinge.)
  const x = side * (0.008 + 0.004);
  checks(p, rear, front, body.cabin[4], (z, y): V3 => [x, y, z - hinge], [side, 0, 0]);
}

/** Where door `d`'s panel runs (z, rear to front) at the band's height. */
function doorSpan(body: CarBody, d: number): [number, number] {
  const { hinge, rearBelt, rearLow, sill } = carDoor(body, d);
  const yb = body.cabin[4];
  const t = Math.min(1, Math.max(0, (yb - CHECK_DROP - CHECK - sill) / (yb - sill)));
  return [rearLow + (rearBelt - rearLow) * t, hinge];
}

/**
 * The chequered band from `z0` to `z1` (car z), two rows of squares on a white strip under the
 * belt line `yb`, its squares on the car's own grid (a door's line up with the body's), placed by
 * `at(z, y)` facing `n`.
 */
function checks(p: Painter, z0: number, z1: number, yb: number, at: (z: number, y: number) => V3, n: V3): void {
  if (z1 - z0 < 0.01) return;
  const top = yb - CHECK_DROP;
  const bottom = top - 2 * CHECK;
  const lift = (q: V3, k: number): V3 => [q[0] + n[0] * k, q[1] + n[1] * k, q[2] + n[2] * k];
  p.paint('#F4F2EC', 0.3, 0).poly([at(z0, bottom), at(z1, bottom), at(z1, top), at(z0, top)], n);
  p.paint(INK, 0.4, 0);
  for (let k = Math.floor(z0 / CHECK); k * CHECK < z1; k++) {
    const a = Math.max(z0, k * CHECK);
    const b = Math.min(z1, (k + 1) * CHECK);
    if (b - a < 0.005) continue;
    // (Black on alternate squares, each row the other way: `k` may be negative.)
    const row = ((k % 2) + 2) % 2;
    const y0 = row === 0 ? bottom : bottom + CHECK;
    p.poly([lift(at(a, y0), 0.002), lift(at(b, y0), 0.002), lift(at(b, y0 + CHECK), 0.002), lift(at(a, y0 + CHECK), 0.002)], n);
  }
}

/** TAXI in block letters on a sign `h` high, placed by `at(u, v)` (u across from its middle, v up from its foot) facing `n`. */
function lettering(p: Painter, at: (u: number, v: number) => V3, n: V3, h: number): void {
  const quad = (a: [number, number], b: [number, number], c: [number, number], d: [number, number]) => p.poly([at(...a), at(...b), at(...c), at(...d)], n);
  const [lw, gap, s] = [0.085, 0.032, 0.021];
  const [v0, v1] = [h * 0.2, h * 0.8];
  const vm = (v0 + v1) / 2;
  const total = 4 * lw + 3 * gap;
  p.paint(INK, 0.4, 0);
  ['T', 'A', 'X', 'I'].forEach((ch, i) => {
    const u0 = -total / 2 + i * (lw + gap);
    const [u1, uc] = [u0 + lw, u0 + lw / 2];
    if (ch === 'T') {
      quad([u0, v1 - s], [u1, v1 - s], [u1, v1], [u0, v1]);
      quad([uc - s / 2, v0], [uc + s / 2, v0], [uc + s / 2, v1 - s], [uc - s / 2, v1 - s]);
    } else if (ch === 'A') {
      quad([u0, v0], [u0 + s, v0], [uc + s / 2, v1], [uc - s / 2, v1]);
      quad([u1 - s, v0], [u1, v0], [uc + s / 2, v1], [uc - s / 2, v1]);
      quad([u0 + lw * 0.27, vm - s * 0.7], [u1 - lw * 0.27, vm - s * 0.7], [u1 - lw * 0.32, vm + s * 0.3], [u0 + lw * 0.32, vm + s * 0.3]);
    } else if (ch === 'X') {
      quad([u0, v0], [u0 + s, v0], [u1, v1], [u1 - s, v1]);
      quad([u1 - s, v0], [u1, v0], [u0 + s, v1], [u0, v1]);
    } else quad([uc - s / 2, v0], [uc + s / 2, v0], [uc + s / 2, v1], [uc - s / 2, v1]);
  });
}

// ---- the snapshot's car rows -------------------------------------------------------------

/** Where car row `row` starts in a snapshot laid out as `layout` (-1: the layout has no cars). */
export function carRowAt(layout: SnapshotLayout, row: number): number {
  return layout.carBase === undefined || layout.carStride === undefined || !layout.car ? -1 : layout.carBase + row * layout.carStride;
}

/**
 * Whether car row `row` of `snap` is a work taxi's (`car.kind`, sim-core `snapshot::car::KIND`):
 * none is in a layout without kinds (every car a household's), nor past the rows there are.
 */
export function carRowIsTaxi(snap: ArrayLike<number>, layout: SnapshotLayout, row: number): boolean {
  const f = layout.car;
  const o = carRowAt(layout, row);
  if (o < 0 || f?.kind === undefined || !f.kinds || layout.header.carCount === undefined) return false;
  if (row < 0 || row >= (snap[layout.header.carCount] | 0)) return false;
  return snap[o + f.kind] === f.kinds.taxi;
}

/**
 * Door `d` of `body` (`carDoor`) in `p`'s frame with its hinge at the origin (the door's front edge
 * on the car's side, on the ground at `y`), shut: the panel, its window in a painted frame, the
 * inside facing in. Turned about its hinge (outwards: rear edge away from the car), it opens.
 */
export function paintDoor(p: Painter, body: CarBody, paint: string, d: number, age: CarAge = 1): void {
  const { side, hinge, rearBelt, rearLow, sill, u0, u1 } = carDoor(body, d);
  const [br, bf, rr, rf, yb, yr] = body.cabin;
  const [colour, gloss] = aged(paint, age);
  const hw = body.wid / 2;
  const out = 0.008;
  // (Relative to the hinge: x outwards from the car's side, z back from the hinge.)
  const X = (x: number) => side * (x - hw + out);
  const Z = (z: number) => z - hinge;
  const n: V3 = [side, 0, 0];
  const panel: V3[] = [[X(hw), sill, Z(hinge)], [X(hw), yb, Z(hinge)], [X(hw), yb, Z(rearBelt)], [X(hw), sill, Z(rearLow)]];
  p.paint(colour, gloss, 0).poly(panel, n);
  p.paint(INSIDE, 0.9, 0).poly(panel.map(([x, y, z]): V3 => [x - side * 0.035, y, z]), [-side, 0, 0]);
  // The window: the side window's part between `u0` and `u1`, sloping in to the roof (glazed as far back as the body's `sideGlass`).
  const hb = hw - 0.07;
  const hr = hw - (body.roofIn ?? 0.25);
  const at = (u: number, v: number): V3 => {
    const z = bf + (br - bf) * u + (rf + (rr - rf) * u - (bf + (br - bf) * u)) * v;
    return [X(hb + (hr - hb) * v), yb + (yr - yb) * v, Z(z)];
  };
  const frame: V3[] = [at(u0, 0), at(u1, 0), at(u1, 1), at(u0, 1)];
  const fn = facingOut(frame, [side, 0.3, 0]);
  const face = (q: V3[], normal: V3, lift: number) => p.poly(q.map(([x, y, z]): V3 => [x + normal[0] * lift, y + normal[1] * lift, z + normal[2] * lift]), normal);
  p.paint(colour, gloss, 0);
  face(frame, fn, 0);
  const glass = (u: number, v: number) => at(u0 + (u1 - u0) * u, v);
  const end = Math.min(0.92, ((body.sideGlass ?? 1) - u0) / (u1 - u0));
  if (end <= 0.2) return;
  p.paint('#1C252D', 0.06, 0);
  face([glass(0.08, 0.1), glass(end, 0.1), glass(end, 0.86), glass(0.08, 0.86)], fn, 0.006);
  const back: V3 = [-fn[0], -fn[1], -fn[2]];
  face([glass(0.08, 0.1), glass(end, 0.1), glass(end, 0.86), glass(0.08, 0.86)], back, 0.03);
}

/** Wheel `w` of `body` (0 front right, 1 front left, 2 back right, 3 back left: front ones steer): its side (+1: +x), and where its hub is (x, z; y is its radius). */
export function wheelAt(body: CarBody, w: number): { side: 1 | -1; x: number; z: number; front: boolean } {
  const side = w % 2 === 0 ? 1 : -1;
  const front = w < 2;
  return { side, x: side * (body.wid / 2 + 0.012), z: front ? body.wheelBase / 2 : -body.wheelBase / 2, front };
}

/** Where a car is: its middle and which way it faces (0: towards +z). */
export interface CarPose {
  x: number;
  z: number;
  yaw: number;
}

/** Where the point `inCar` last looked at is in the car (m): along it (+: towards its front) and across it (+: towards +x at yaw 0). */
const inside = { along: 0, across: 0 };

/** Whether (x, z) is in `body` standing at `pose` (its box seen from above): someone sat in it (`pc/cars.ts` `seatAt`, `carriedAlong`). */
export function inCar(body: CarBody, pose: CarPose, x: number, z: number): boolean {
  const [dx, dz] = [x - pose.x, z - pose.z];
  const [s, c] = [Math.sin(pose.yaw), Math.cos(pose.yaw)];
  inside.along = dx * s + dz * c;
  inside.across = dx * c - dz * s;
  return Math.abs(inside.along) < body.len / 2 && Math.abs(inside.across) < body.wid / 2;
}

/**
 * Someone at (x, z) facing `yaw` with `body` at `from`: where they are with it at `to` instead
 * (`out`), carried along rigidly, turned with it; false (`out` untouched) when they aren't in it.
 * Those riding in a car that drives: sat where the snapshot has them in it, drawn where the car is
 * drawn (`pc/cars.ts` `ride`).
 */
export function carriedAlong(body: CarBody, from: CarPose, to: CarPose, x: number, z: number, yaw: number, out: CarPose): boolean {
  if (!inCar(body, from, x, z)) return false;
  const { along, across } = inside;
  const [s1, c1] = [Math.sin(to.yaw), Math.cos(to.yaw)];
  out.x = to.x + across * c1 + along * s1;
  out.z = to.z - across * s1 + along * c1;
  out.yaw = wrapAngle(to.yaw + wrapAngle(yaw - from.yaw));
  return true;
}

/**
 * A wheel of radius 1 (scaled to a car's `wheelR` across y and z: `wheelAt`) in `p`'s frame, turning
 * about x through its hub at the origin: the tyre in from the hub's face (x 0, facing +x, out from
 * the car), a hubcap with five holes, so it can be seen to turn. The left wheels are turned round.
 */
export function paintWheel(p: Painter): void {
  p.paint('#1A1A1C', 0.85, 0).tube('x', -0.232, 0, 0, 0, 1, 1, 12);
  p.paint('#B4B8BC', 0.3, 1).tube('x', 0, 0.008, 0, 0, 0.6, 0.5, 10);
  p.paint('#2A2C2F', 0.6, 0);
  for (let k = 0; k < 5; k++) {
    const a = (k / 5) * Math.PI * 2;
    const [y, z] = [Math.cos(a) * 0.3, Math.sin(a) * 0.3];
    p.box(0.008, y - 0.07, z - 0.07, 0.011, y + 0.07, z + 0.07, 'nx');
  }
}

/** `q`'s normal (its first three corners), turned to face `towards`. */
function facingOut(q: V3[], towards: V3): V3 {
  return facing(q[0], q[1], q[2], towards);
}

/** Clearance from a driveway's inner end to the car parked on it (m; sim-core `cars::PARK_GAP`). */
const PARK_GAP = 0.45;

/** A driveway's inner end (its middle, `x`, `z`), which way it runs from there to the street (`front`: +1 towards +z) and the middle of that street (`street`, z): where a car parks (sim-core `cars::Drive`). */
export interface DriveEnd {
  x: number;
  z: number;
  front: 1 | -1;
  street: number;
}

/** The inner end of the driveway `drive` on `plot`, by the street the plot faces (one of its z edges lies on a street's edge); null when it faces none. */
export function driveEnd(plot: Rect, drive: Rect, streets: readonly Rect[]): DriveEnd | null {
  for (const st of streets) {
    if (st.w < st.d || plot.x + plot.w <= st.x || plot.x >= st.x + st.w) continue;
    if (Math.abs(plot.z + plot.d - st.z) < 0.05) return { x: drive.x + drive.w / 2, z: drive.z, front: 1, street: st.z + st.d / 2 };
    if (Math.abs(plot.z - (st.z + st.d)) < 0.05) return { x: drive.x + drive.w / 2, z: drive.z + drive.d, front: -1, street: st.z + st.d / 2 };
  }
  return null;
}

/** Where a house's car parks: its middle, which way it faces (0: towards +z), the middle of the street its driveway leads to, its body and paint (indices into `CARS`, `CAR_PAINT`) and size. */
export interface ParkedCar {
  x: number;
  z: number;
  yaw: number;
  street: number;
  body: number;
  paint: number;
  len: number;
  wid: number;
}

/**
 * The car of the house on `plot`, on its driveway (at its inner end `spot`, `driveEnd`: beside
 * the house, not in the front yard Sims cross), facing the house or the street; null when the
 * house has none (about one in five). From the plot's place, so the town, the menus and the game
 * agree: one of the family cars (`FAMILY_CARS`), which the sim changes for the one its household's
 * best earner's job calls for (sim-core `cars/looks.rs`; a town's own game draws the sim's).
 */
export function parkCar(plot: Rect, spot: DriveEnd, always = false): ParkedCar | null {
  if (!always && hash(plot.x, plot.z, 20) >= 0.8) return null;
  const { front, street } = spot;
  const body = Math.floor(hash(plot.x, plot.z, 21) * FAMILY_CARS) % FAMILY_CARS;
  const { len, wid } = CARS[body];
  const yaw = (hash(plot.x, plot.z, 22) < 0.65 ? (front > 0 ? Math.PI : 0) : front > 0 ? 0 : Math.PI) + (hash(plot.x, plot.z, 23) - 0.5) * 0.06;
  return {
    x: spot.x,
    z: spot.z + front * (PARK_GAP + len / 2),
    yaw,
    street,
    body,
    paint: Math.floor(hash(plot.x, plot.z, 24) * CAR_PAINT.length) % CAR_PAINT.length,
    len,
    wid,
  };
}

/**
 * The car of the house on `plot` with the driveway `drive` (both on the town; the plot's street
 * on its side `facing`, as `TownMeta.facing`: 0 north, 1 west, 2 south, 3 east), on the town:
 * `parkCar` with the plot laid out as the street lays it out (`street.ts` `layout`: turned a
 * quarter when its street is east or west of it), so the town's cars (`game/town.ts`, the sim's
 * `cars`) stand where the street drew them, look as it drew them and go on the same houses (about
 * four in five). `always`: a car even on a house the street would leave without one.
 */
export function houseCar(plot: Rect, facing: number, drive: Rect, always = false): ParkedCar | null {
  const turned = facing % 2 === 1;
  // (As `street.ts` `localRect`, `townPoint` and `townYaw`.)
  const laid = (r: Rect): Rect => (turned ? { x: r.z, z: -(r.x + r.w), w: r.d, d: r.w } : r);
  const front: 1 | -1 = facing === 2 || facing === 1 ? 1 : -1;
  const d = laid(drive);
  const car = parkCar(laid(plot), { x: d.x + d.w / 2, z: front > 0 ? d.z : d.z + d.d, front, street: 0 }, always);
  if (!car || !turned) return car;
  return { ...car, x: -car.z, z: car.x, yaw: car.yaw - Math.PI / 2 };
}

// ---- garages ---------------------------------------------------------------------------

/** A garage's depth, and its least width between its wall lines: 3 m inside, a single garage's (its door 2.4 m). */
export const GARAGE_DEPTH = 5.4;
export const GARAGE_WIDTH = 3.14;
/** Half a wall's thickness (as in `house.ts`). */
const WALL_HALF = 0.07;
/** How far a garage keeps from the things on the lot (m): a tile, to use them from and walk round them by. */
const GARAGE_CLEAR = 1;

/** A garage: its wall centre lines (sim-core `cars::Garage`). */
export interface Garage {
  x0: number;
  z0: number;
  x1: number;
  z1: number;
}

/**
 * The garage of the house (`h`: its wall box) on `plot` at the inner end of its driveway `d` (the
 * street on its `front` side), when it has one (about seven in ten) and it fits clear of
 * `blocked` (things on the lot, by `GARAGE_CLEAR`): wall centre lines. The town as it's made decides it
 * (`game/town.ts`, the sim's `garages`), and so the menus and the game agree.
 */
export function garageFor(plot: Rect, h: { x0: number; z0: number; x1: number; z1: number }, d: Rect, front: 1 | -1, blocked: readonly Rect[]): Garage | null {
  if (hash(plot.x, plot.z, 50) > 0.7) return null;
  return garageAt(plot, h, d, front, blocked);
}

/**
 * The garage `garageFor` gives a house that has one, on any house (the player builds it: Build
 * mode's Garage tool, at the end of their driveway): null when it doesn't fit there.
 */
export function garageAt(plot: Rect, h: { x0: number; z0: number; x1: number; z1: number }, d: Rect, front: 1 | -1, blocked: readonly Rect[]): Garage | null {
  // As wide as the drive and a little more, at least a single garage's `GARAGE_WIDTH`, moved
  // (off the drive's middle if it must) to stand clear of the house and the plot's edge.
  const toRight = d.x >= h.x1;
  const [lo, hi] = toRight ? [h.x1 + 0.35, plot.x + plot.w - 0.45] : [plot.x + 0.45, h.x0 - 0.35];
  const w = Math.max(d.w + 0.3, GARAGE_WIDTH);
  if (hi - lo < w) return null;
  const x0 = Math.min(Math.max(d.x + d.w / 2 - w / 2, lo), hi - w);
  const x1 = x0 + w;
  const inner = front > 0 ? d.z : d.z + d.d;
  const [z0, z1] = front > 0 ? [inner - GARAGE_DEPTH, inner] : [inner, inner + GARAGE_DEPTH];
  if (z0 < plot.z + 0.8 || z1 > plot.z + plot.d - 0.8) return null;
  const r = { x: x0 - WALL_HALF, z: z0 - WALL_HALF, w: x1 - x0 + 2 * WALL_HALF, d: z1 - z0 + 2 * WALL_HALF };
  const m = GARAGE_CLEAR;
  if (blocked.some((b) => r.x - m < b.x + b.w && r.x + r.w + m > b.x && r.z - m < b.z + b.d && r.z + r.d + m > b.z)) return null;
  return { x0, z0, x1, z1 };
}
