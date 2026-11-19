/**
 * A town's public parking: marked bays in rows (car parks) off a straight stretch of street. A car parks in a bay
 * nose in (the way the sim turns into a driveway: sim-core `cars::parking_bay`) and backs out of it; a lone pair by
 * the verge is a car park of its own. `carParkAt` lays one out; the generator puts one by the square, one by each big
 * place and pairs along the verges (`macro.ts`), and keeps every bay clear of every driveway (`clearOfDrives`).
 */

import type { CarPark, ParkingBay } from '../core/protocol';
import { flatsTemplate } from './flats';
import { rectFrom, tilesOf } from './footprints';
import { leftOf, lengths, nearest, pointAt, roadIndex, straightAround, type Pt, type RoadPlan } from './roads';
import { DRIVE_REACH, driveMeets, driveways, houseTemplate, onLot, slotDims, turner, type Frontage, type Ground, type PlotSlot, type Rect, type Templates } from './town';

// ---- The simulation's parking rules (sim-core `cars/parking.rs`, held to it by `web/tests/parkingRules.test.ts`) ----

/**
 * How near a driveway's mouth (where it meets its road, on the road's middle line) a bay's middle may not be (m): the
 * simulation parks no trip's car in a bay nearer (`by_a_mouth`), and a generated town lays none so.
 */
export const MOUTH_CLEAR = 7.0;
/** How near a driveway (the line from its inner end to its mouth) a bay's middle may not be (m), as `MOUTH_CLEAR`. */
export const DRIVE_CLEAR = 2.5;
/**
 * A bay: 3 m along the road by 6 m deep, side by side 3 m apart in a car park's row (m). A car parked in one stands at
 * its middle facing its yaw (nose in, away from the road), the pose the town file gives it.
 */
export const BAY_WIDE = 3.0;
export const BAY_DEEP = 6.0;
/** The lines painted round a bay, across and along it (m, about its middle): every car body parked in it is inside them. */
export const BAY_LINES: readonly [number, number] = [2.8, 5.6];
// ---- Laying car parks out ----

/** How far a car park's stretch of road may turn (radians over its length and a bay either side): each bay faces away from the road at its own nearest point. */
const PARK_BEND = 0.3;
/** Off the world's edge (m), as the lots keep. */
const EDGE = 8;

const add = (a: Pt, b: Pt, k = 1): Pt => [a[0] + b[0] * k, a[1] + b[1] * k];
const r2 = (v: number) => Math.round(v * 100) / 100;
const r4 = (v: number) => Math.round(v * 1e4) / 1e4;

/** A car park laid out (`carParkAt`): the car park, its bays, and its asphalt with `MOUTH_CLEAR` m more along the road either end (what it keeps clear). */
export interface CarParkLaid {
  park: CarPark;
  bays: ParkingBay[];
  clear: Pt[];
}

/**
 * A car park of `n` bays (area `area`) on the side of road `f.road` that `f` is on, its row's middle at `f.s`:
 * bay `k`'s middle `p + t·3·(k − (n − 1)/2) + n·(width/2 + 0.5 + 3)` (`t` along the road, `n` away from it), each
 * facing away from the road at its own nearest point (nose in); its asphalt from the verge's edge back the bays'
 * depth; its P sign a metre past its far end, by the pavement, facing the road.
 */
export function carParkAt(roads: readonly RoadPlan[], f: Pick<Frontage, 'road' | 's' | 'side'>, n: number, area: number, also: Pick<CarPark, 'serves' | 'square'> = {}): CarParkLaid {
  const road = roads[f.road];
  const ls = lengths(road.points);
  const { p, t } = pointAt(road.points, f.s, ls);
  const out = leftOf(t).map((v) => v * f.side) as Pt;
  const reach = road.width / 2 + 0.5;
  const bays = Array.from({ length: n }, (_, k): ParkingBay => {
    const mid = add(add(p, t, BAY_WIDE * (k - (n - 1) / 2)), out, reach + BAY_DEEP / 2);
    const q = pointAt(road.points, nearest(road.points, mid[0], mid[1], ls).s, ls).p;
    return { x: r2(mid[0]), z: r2(mid[1]), yaw: r4(Math.atan2(mid[0] - q[0], mid[1] - q[1])), area };
  });
  const front = add(p, out, reach);
  const poly = rectFrom(front, t, out, BAY_WIDE * n, BAY_DEEP).map(([x, z]): [number, number] => [r2(x), r2(z)]);
  const sign = add(add(p, t, (BAY_WIDE * n) / 2 + 1), out, reach + 0.4);
  const park: CarPark = { area, poly, sign: [r2(sign[0]), r2(sign[1]), r4(Math.atan2(-out[0], -out[1]))], ...also };
  return { park, bays, clear: rectFrom(front, t, out, BAY_WIDE * n + 2 * MOUTH_CLEAR, BAY_DEEP) };
}

/**
 * Whether a car park of `n` bays fits at frontage `f` (`carParkAt`): its stretch of road straight, its ends
 * `PARK_CORNER` m from a junction (`offCorners`, `macro.ts` `cornersOf`), its asphalt and `MOUTH_CLEAR` m either
 * end of it on land nothing takes (`ground`) inside the world's edge, every bay clear of every driveway of `drives`
 * (`clearOf`) and nearer its own street than any other road.
 */
export function carParkFits(roads: readonly RoadPlan[], ground: Ground, f: Frontage, n: number, offCorners: (f: Frontage) => boolean, drives: readonly DriveLine[] = []): CarParkLaid | null {
  const road = roads[f.road];
  const ls = lengths(road.points);
  const L = ls[ls.length - 1];
  const half = (BAY_WIDE * n) / 2;
  if (f.s - half < 4 || f.s + half > L - 4) return null;
  if (!straightAround(road.points, f.s, half + BAY_WIDE, PARK_BEND, ls)) return null;
  for (const s of [f.s - half, f.s + half]) if (!offCorners({ ...f, s, p: pointAt(road.points, s, ls).p })) return null;
  const laid = carParkAt(roads, f, n, 0);
  // (Every tile it touches but its own road's, which reach a little past its verge: `roads.ts` `roadMask`.)
  const roads0 = ground.roadTiles();
  const own = (x: number, z: number) => roads0[z * ground.w + x] === 1 && nearest(road.points, x + 0.5, z + 0.5, ls).dist < road.width / 2 + 1.5;
  const bad = tilesOf(laid.clear, 0, (x, z) => x < EDGE || z < EDGE || x >= ground.w - EDGE || z >= ground.d - EDGE || (!own(x, z) && ground.takenAt(x + 0.5, z + 0.5)));
  if (bad) return null;
  // (No bay by a driveway, and every bay nearer its own street than any other road.)
  if (!clearOf(laid.bays, drives)) return null;
  for (const b of laid.bays) if (nearestRoad(roads, b.x, b.z) !== f.road) return null;
  return laid;
}

/**
 * Pairs of bays by the verge (a single one for the last) along the first of `frontage` each fits (`carParkFits`), of
 * area `area`, till they make `spaces` bays; each car park's land taken on `ground` as it goes down. The car parks laid.
 */
export function vergeParking(roads: readonly RoadPlan[], ground: Ground, frontage: readonly Frontage[], spaces: number, area: number, offCorners: (f: Frontage) => boolean = () => true, drives: readonly DriveLine[] = []): CarParkLaid[] {
  const out: CarParkLaid[] = [];
  for (let n = spaces; n > 0; n = spaces - out.reduce((k, p) => k + p.bays.length, 0)) {
    const want = Math.min(2, n);
    let laid: CarParkLaid | null = null;
    for (const f of frontage) if ((laid = carParkFits(roads, ground, f, want, offCorners, drives))) break;
    if (!laid) break;
    const own: CarParkLaid = { ...laid, park: { ...laid.park, area }, bays: laid.bays.map((b) => ({ ...b, area })) };
    takeCarPark(ground, own);
    out.push(own);
  }
  return out;
}

/** Takes a car park's land on `ground`: its asphalt and `MOUTH_CLEAR` m along the road either end (`CarParkLaid.clear`), so no lot's driveway comes out by it. */
export function takeCarPark(ground: Ground, laid: CarParkLaid): void {
  // (Every tile it touches: a lot beside it keeps off its asphalt's corners too.)
  tilesOf(laid.clear, 0, (x, z) => {
    ground.take({ x, z, w: 1, d: 1 });
  });
}

/** The box (town tiles) a car park's asphalt covers (what edits keep clear of it, as of a lot). */
export function carParkRect(p: Pick<CarPark, 'poly'>): Rect {
  const xs = p.poly.map((q) => q[0]);
  const zs = p.poly.map((q) => q[1]);
  const [x0, z0] = [Math.floor(Math.min(...xs)), Math.floor(Math.min(...zs))];
  return { x: x0, z: z0, w: Math.ceil(Math.max(...xs)) - x0, d: Math.ceil(Math.max(...zs)) - z0 };
}

/** The box (town tiles) a bay covers: its `BAY_WIDE` × `BAY_DEEP` turned to its yaw. */
export function bayRect(b: Pick<ParkingBay, 'x' | 'z' | 'yaw'>): Rect {
  const [ax, az] = [Math.abs(Math.sin(b.yaw)), Math.abs(Math.cos(b.yaw))];
  const hw = (BAY_WIDE * az + BAY_DEEP * ax) / 2;
  const hd = (BAY_WIDE * ax + BAY_DEEP * az) / 2;
  const [x0, z0] = [Math.floor(b.x - hw), Math.floor(b.z - hd)];
  return { x: x0, z: z0, w: Math.ceil(b.x + hw) - x0, d: Math.ceil(b.z + hd) - z0 };
}

/**
 * The driveways of the houses of `slots` (and the bays of its flats buildings) as `layoutTown` lays them (the side that meets its road squarely, else the
 * first: `town.ts` `driveMeets`), each as `DriveLine`.
 */
export function driveLines(t: Templates, slots: readonly PlotSlot[], roads: readonly RoadPlan[], footprint: (def: string) => [number, number]): DriveLine[] {
  const index = roadIndex(roads);
  const out: DriveLine[] = [];
  for (const s of slots) {
    const { width, depth } = slotDims(t, s);
    const turn = turner(s.x, s.z, width, depth, s.facing);
    // (A flats building's bays are its flats' driveways.)
    const f = s.kind === 'flats' ? flatsTemplate(t, s.template) : undefined;
    for (const { rect: r } of f?.bays ?? []) {
      const [a, b] = [turn.point([r.x + r.w / 2, 0]), turn.point([r.x + r.w / 2, -1])];
      const meets = driveMeets(roads, index, a, [b[0] - a[0], b[1] - a[1]], DRIVE_REACH);
      if (meets) out.push({ inner: turn.point([r.x + r.w / 2, r.d]), mouth: pointAt(roads[meets.road].points, meets.s).p });
    }
    const h = s.kind === 'house' ? houseTemplate(t, s.template) : undefined;
    if (!h) continue;
    const sides = driveways(onLot(t, h, s, footprint), width, depth, footprint).map((r) => {
      const [a, b] = [turn.point([r.x + r.w / 2, 0]), turn.point([r.x + r.w / 2, -1])];
      return { r, meets: driveMeets(roads, index, a, [b[0] - a[0], b[1] - a[1]], DRIVE_REACH) };
    });
    const side = sides.find((o) => o.meets?.square) ?? sides[0];
    if (!side?.meets) continue;
    out.push({ inner: turn.point([side.r.x + side.r.w / 2, side.r.d]), mouth: pointAt(roads[side.meets.road].points, side.meets.s).p });
  }
  return out;
}

/** A driveway as the simulation measures a bay against it: its inner end and its mouth (where it meets its road, on the road's middle line). */
export interface DriveLine {
  inner: Pt;
  mouth: Pt;
}

/** How far a point is from a segment. */
function offSegment([x, z]: Pt, [ax, az]: Pt, [bx, bz]: Pt): number {
  const [dx, dz] = [bx - ax, bz - az];
  const l2 = dx * dx + dz * dz;
  const k = l2 ? Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / l2)) : 0;
  return Math.hypot(x - ax - dx * k, z - az - dz * k);
}

/** Whether every bay's middle is `MOUTH_CLEAR` m from every driveway's mouth and `DRIVE_CLEAR` m from its line (sim-core `by_a_mouth`). */
export function clearOf(bays: readonly Pick<ParkingBay, 'x' | 'z'>[], drives: readonly DriveLine[]): boolean {
  return bays.every((b) => drives.every((d) => Math.hypot(b.x - d.mouth[0], b.z - d.mouth[1]) >= MOUTH_CLEAR && offSegment([b.x, b.z], d.inner, d.mouth) >= DRIVE_CLEAR));
}

/** The road (not a trail) whose middle line is nearest a point, by index (-1: none). */
function nearestRoad(roads: readonly RoadPlan[], x: number, z: number): number {
  let best = { r: -1, d: Infinity };
  roads.forEach((r, i) => {
    if (r.kind === 'trail') return;
    const d = nearest(r.points, x, z).dist;
    if (d < best.d) best = { r: i, d };
  });
  return best.r;
}

/**
 * The car parks of `parks` with every bay by a driveway of `drives` taken out (`clearOf`: a lot laid after a car park
 * may have its driveway by it, one across the street too); a car park left with no bay goes. Its bays, numbered by their
 * car parks (`park`), and the car parks.
 */
export function clearOfDrives(parks: readonly CarParkLaid[], drives: readonly DriveLine[]): { bays: ParkingBay[]; carParks: CarPark[] } {
  const bays: ParkingBay[] = [];
  const carParks: CarPark[] = [];
  for (const p of parks) {
    const kept = p.bays.filter((b) => clearOf([b], drives));
    if (!kept.length) continue;
    bays.push(...kept.map((b) => ({ ...b, park: carParks.length })));
    carParks.push(p.park);
  }
  return { bays, carParks };
}
