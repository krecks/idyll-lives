/**
 * Households' cars as the street draws them: their bodies (made of the street's plain-finish
 * primitives, `paintCar`), their paints, and where a house's car parks on its driveway
 * (`parkCar`), the same for the town as it is made (`game/town.ts`, the sim's `cars`), the menus'
 * town and the game, so a car stands where the town showed it.
 *
 * Engine-neutral: the street (`street.ts`) and the moving cars (`pc/cars.ts`) draw them.
 */

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
}

export const CARS: CarBody[] = [
  // Saloon.
  { len: 4.4, wid: 1.78, wheelR: 0.32, wheelBase: 2.65, low: [[-2.2, 0.17], [2.2, 0.17], [2.24, 0.52], [2.14, 0.74], [0.95, 0.88], [-1.35, 0.93], [-2.12, 0.88], [-2.24, 0.58]], cabin: [-1.4, 0.95, -0.8, 0.16, 0.9, 1.42] },
  // Hatchback.
  { len: 4.0, wid: 1.74, wheelR: 0.31, wheelBase: 2.5, low: [[-1.98, 0.17], [2.0, 0.17], [2.04, 0.52], [1.96, 0.72], [0.85, 0.87], [-1.9, 0.93], [-2.02, 0.6]], cabin: [-1.9, 0.88, -1.72, 0.08, 0.9, 1.46] },
  // SUV.
  { len: 4.6, wid: 1.88, wheelR: 0.37, wheelBase: 2.75, low: [[-2.3, 0.23], [2.3, 0.23], [2.34, 0.66], [2.24, 0.92], [1.25, 1.02], [-2.25, 1.06], [-2.34, 0.7]], cabin: [-2.24, 1.28, -2.14, 0.48, 1.02, 1.76] },
  // Estate.
  { len: 4.6, wid: 1.8, wheelR: 0.32, wheelBase: 2.7, low: [[-2.3, 0.17], [2.3, 0.17], [2.34, 0.52], [2.24, 0.74], [1.05, 0.88], [-2.25, 0.93], [-2.33, 0.6]], cabin: [-2.24, 1.05, -2.1, 0.25, 0.9, 1.45] },
];

/** Doors (as sim-core `cars::DOORS`): 0 front left (the driver's, on the car's +x side), 1 front right, 2 back left, 3 back right. */
export const DOORS = 4;
/** Paint of the inside of a car, where an open door shows it. */
const INSIDE = '#26282B';

/** Where door `d` of `body` is: its side (+1: +x), its hinge (front edge, z), its rear edge at the belt line and at the sill, its sill's height, and the part of the side window it carries (`u0`..`u1`, front to back). */
export function carDoor(body: CarBody, d: number): { side: 1 | -1; hinge: number; rearBelt: number; rearLow: number; sill: number; u0: number; u1: number } {
  const [br, bf] = body.cabin;
  const side = d % 2 === 0 ? 1 : -1;
  const at = (u: number) => bf + (br - bf) * u;
  const arch = body.wheelR + 0.1;
  const sill = Math.max(...body.low.slice(0, 2).map((q) => q[1])) + 0.1;
  if (d < 2) return { side, hinge: Math.min(bf, body.wheelBase / 2 - arch), rearBelt: at(0.505), rearLow: at(0.505), sill, u0: 0, u1: 0.505 };
  return { side, hinge: at(0.535), rearBelt: at(0.97), rearLow: Math.max(at(0.97), -body.wheelBase / 2 + arch), sill, u0: 0.535, u1: 0.97 };
}

/**
 * A parked car in `p`'s frame (front towards +z), sitting on the ground at `y`. `apart`: without
 * its doors, the inside showing where they go (each drawn on its own, `paintDoor`, to open).
 */
export function paintCar(p: Painter, body: CarBody, paint: string, apart = false): void {
  const { wid, low } = body;
  const hw = wid / 2;
  const inner = hw - 0.045;
  const [cz, cy] = [low.reduce((a, q) => a + q[0], 0) / low.length, low.reduce((a, q) => a + q[1], 0) / low.length];
  const shrunk = low.map(([z, y]): [number, number] => {
    const [dz, dy] = [cz - z, cy - y];
    const l = Math.hypot(dz, dy) || 1;
    return [z + (dz / l) * 0.035, y + (dy / l) * 0.035];
  });
  p.paint(paint, 0.2, 0);
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
    if (ny < -0.9) p.paint(paint, 0.2, 0);
  }
  // Greenhouse: painted frame and roof, glass inset into it.
  const [br, bf, rr, rf, yb, yr] = body.cabin;
  const hb = hw - 0.07;
  const hr = hw - 0.25;
  const B = (x: number, z: number): V3 => [x, yb, z];
  const R = (x: number, z: number): V3 => [x, yr, z];
  const quad = (q: V3[], out: V3, glass: [number, number, number, number] | null, pillar = false) => {
    p.paint(paint, 0.2, 0).poly(q, out);
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
    if (pillar) {
      const mid = 0.52;
      const lift = (v: V3): V3 => {
        const len = Math.hypot(...out) || 1;
        return [v[0] + (out[0] / len) * 0.004, v[1] + (out[1] / len) * 0.004, v[2] + (out[2] / len) * 0.004];
      };
      p.paint(paint, 0.2, 0).poly([lift(at(mid - 0.025, 0)), lift(at(mid + 0.025, 0)), lift(at(mid + 0.025, 1)), lift(at(mid - 0.025, 1))], out);
    }
  };
  const ws = facingOut([B(-hb, bf), B(hb, bf), R(hr, rf)], [0, 0.6, 1]);
  quad([B(-hb, bf), B(hb, bf), R(hr, rf), R(-hr, rf)], ws, [0.05, 0.95, 0.06, 0.9]);
  const rw = facingOut([B(hb, br), B(-hb, br), R(-hr, rr)], [0, 0.6, -1]);
  quad([B(hb, br), B(-hb, br), R(-hr, rr), R(hr, rr)], rw, [0.06, 0.94, 0.1, 0.88]);
  for (const side of [-1, 1]) {
    const q: V3[] = side > 0 ? [B(hb, bf), B(hb, br), R(hr, rr), R(hr, rf)] : [B(-hb, br), B(-hb, bf), R(-hr, rf), R(-hr, rr)];
    if (!apart) {
      quad(q, facingOut(q, [side, 0.3, 0]), [0.05, 0.95, 0.1, 0.86], true);
      continue;
    }
    // The doors carry the side windows: the inside behind them.
    const n = facingOut(q, [side, 0.3, 0]);
    p.paint(paint, 0.2, 0).poly(q, n);
    p.paint(INSIDE, 0.9, 0).poly(q.map(([x, y, z]): V3 => [x + n[0] * 0.003, y + n[1] * 0.003, z + n[2] * 0.003]), n);
  }
  if (apart) {
    p.paint(INSIDE, 0.9, 0);
    for (let d = 0; d < DOORS; d++) {
      const { side, hinge, rearBelt, rearLow, sill } = carDoor(body, d);
      const x = side * (hw + 0.002);
      p.poly([[x, sill, hinge], [x, yb, hinge], [x, yb, rearBelt], [x, sill, rearLow]], [side, 0, 0]);
    }
  }
  p.paint(paint, 0.2, 0).poly([R(-hr, rr), R(hr, rr), R(hr, rf), R(-hr, rf)], [0, 1, 0]);
  // Wheels in dark arches, hubcaps.
  const r = body.wheelR;
  for (const wz of [body.wheelBase / 2, -body.wheelBase / 2]) {
    for (const side of [-1, 1]) {
      p.paint('#141517', 0.9, 0).tube('x', side * (hw + 0.002), side * (hw + 0.003), r, wz, r + 0.07, r + 0.07, 8, true, 0, Math.PI);
      p.paint('#1A1A1C', 0.85, 0).tube('x', side * (hw - 0.22), side * (hw + 0.012), r, wz, r, r, 12);
      p.paint('#B4B8BC', 0.3, 1).tube('x', side * (hw + 0.012), side * (hw + 0.02), r, wz, r * 0.6, r * 0.5, 10);
    }
  }
  // Bumpers, lights, grille, plates, mirrors.
  const zf = Math.max(...low.map((q) => q[0]));
  const zb = Math.min(...low.map((q) => q[0]));
  // Bumpers and lights by the wheels' height, not the sills'.
  const yl = body.wheelR - 0.02;
  p.paint('#2A2C2E', 0.6, 0);
  p.box(-hw + 0.04, yl - 0.02, zf - 0.12, hw - 0.04, yl + 0.16, zf + 0.04);
  p.box(-hw + 0.04, yl - 0.02, zb - 0.04, hw - 0.04, yl + 0.16, zb + 0.12);
  p.box(-0.42, yl + 0.26, zf - 0.08, 0.42, yl + 0.36, zf + 0.0);
  p.paint('#F5F3EA', 0.1, 0);
  for (const s of [-1, 1]) p.box(s * (hw - 0.34), yl + 0.24, zf - 0.1, s * (hw - 0.08), yl + 0.36, zf - 0.02);
  p.paint('#A3161E', 0.2, 0);
  for (const s of [-1, 1]) p.box(s * (hw - 0.36), yl + 0.28, zb + 0.02, s * (hw - 0.06), yl + 0.42, zb + 0.1);
  p.paint('#E9E6D8', 0.4, 0);
  p.box(-0.26, yl + 0.02, zf + 0.04, 0.26, yl + 0.13, zf + 0.06);
  p.box(-0.26, yl + 0.2, zb - 0.02, 0.26, yl + 0.31, zb);
  p.paint(paint, 0.2, 0);
  for (const s of [-1, 1]) p.box(Math.min(s * hw, s * (hw + 0.12)), yb + 0.04, bf - 0.32, Math.max(s * hw, s * (hw + 0.12)), yb + 0.14, bf - 0.2);
}

/**
 * Door `d` of `body` (`carDoor`) in `p`'s frame with its hinge at the origin (the door's front edge
 * on the car's side, on the ground at `y`), shut: the panel, its window in a painted frame, the
 * inside facing in. Turned about its hinge (outwards: rear edge away from the car), it opens.
 */
export function paintDoor(p: Painter, body: CarBody, paint: string, d: number): void {
  const { side, hinge, rearBelt, rearLow, sill, u0, u1 } = carDoor(body, d);
  const [br, bf, rr, rf, yb, yr] = body.cabin;
  const hw = body.wid / 2;
  const out = 0.008;
  // (Relative to the hinge: x outwards from the car's side, z back from the hinge.)
  const X = (x: number) => side * (x - hw + out);
  const Z = (z: number) => z - hinge;
  const n: V3 = [side, 0, 0];
  const panel: V3[] = [[X(hw), sill, Z(hinge)], [X(hw), yb, Z(hinge)], [X(hw), yb, Z(rearBelt)], [X(hw), sill, Z(rearLow)]];
  p.paint(paint, 0.2, 0).poly(panel, n);
  p.paint(INSIDE, 0.9, 0).poly(panel.map(([x, y, z]): V3 => [x - side * 0.035, y, z]), [-side, 0, 0]);
  // The window: the side window's part between `u0` and `u1`, sloping in to the roof.
  const hb = hw - 0.07;
  const hr = hw - 0.25;
  const at = (u: number, v: number): V3 => {
    const z = bf + (br - bf) * u + (rf + (rr - rf) * u - (bf + (br - bf) * u)) * v;
    return [X(hb + (hr - hb) * v), yb + (yr - yb) * v, Z(z)];
  };
  const frame: V3[] = [at(u0, 0), at(u1, 0), at(u1, 1), at(u0, 1)];
  const fn = facingOut(frame, [side, 0.3, 0]);
  const face = (q: V3[], normal: V3, lift: number) => p.poly(q.map(([x, y, z]): V3 => [x + normal[0] * lift, y + normal[1] * lift, z + normal[2] * lift]), normal);
  p.paint(paint, 0.2, 0);
  face(frame, fn, 0);
  const glass = (u: number, v: number) => at(u0 + (u1 - u0) * u, v);
  p.paint('#1C252D', 0.06, 0);
  face([glass(0.08, 0.1), glass(0.92, 0.1), glass(0.92, 0.86), glass(0.08, 0.86)], fn, 0.006);
  const back: V3 = [-fn[0], -fn[1], -fn[2]];
  face([glass(0.08, 0.1), glass(0.92, 0.1), glass(0.92, 0.86), glass(0.08, 0.86)], back, 0.03);
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
 * agree.
 */
export function parkCar(plot: Rect, spot: DriveEnd): ParkedCar | null {
  if (hash(plot.x, plot.z, 20) >= 0.8) return null;
  const { front, street } = spot;
  const body = Math.floor(hash(plot.x, plot.z, 21) * CARS.length) % CARS.length;
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
