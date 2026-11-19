/**
 * Public parking: marked bays (`ParkingBay`) in car parks (`CarPark`). A car park is a row of bays square
 * to a straight stretch of street, behind its pavement, on one apron of asphalt, its P sign at one end; a
 * car parks in a bay nose in (the way the sim turns into a driveway: sim-core `cars::parking_bay`) and
 * backs out of it. A lone pair by the verge is a car park of its own. `carParkAt` lays one out; the
 * generator puts one by the square, one by each big place and pairs along the verges (`macro.ts`).
 */

import type { CarPark, ParkingBay } from '../core/protocol';
import { rectFrom, tilesOf } from './footprints';
import { leftOf, lengths, nearest, pointAt, straightAround, type Pt, type RoadPlan } from './roads';
import { driveways, houseTemplate, onLot, slotDims, turner, type Frontage, type Ground, type PlotSlot, type Rect, type Templates } from './town';

/** A bay keeps this far from a driveway's mouth (m, sim-core `cars::MOUTH_CLEAR`): a car turning out of one never meets a parked one. */
export const MOUTH_CLEAR = 7;
/** A bay keeps this far from a driveway's line (m, sim-core `cars::DRIVE_CLEAR`). */
export const DRIVE_CLEAR = 2.5;

/** A bay (m): along the road, and deep (a parked car's tiles and its doors' stands: today's spaces'). Its lines are drawn a little inside. */
export const BAY_W = 3;
export const BAY_D = 6;
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
    const mid = add(add(p, t, BAY_W * (k - (n - 1) / 2)), out, reach + BAY_D / 2);
    const q = pointAt(road.points, nearest(road.points, mid[0], mid[1], ls).s, ls).p;
    return { x: r2(mid[0]), z: r2(mid[1]), yaw: r4(Math.atan2(mid[0] - q[0], mid[1] - q[1])), area };
  });
  const front = add(p, out, reach);
  const poly = rectFrom(front, t, out, BAY_W * n, BAY_D).map(([x, z]): [number, number] => [r2(x), r2(z)]);
  const sign = add(add(p, t, (BAY_W * n) / 2 + 1), out, reach + 0.4);
  const park: CarPark = { area, poly, sign: [r2(sign[0]), r2(sign[1]), r4(Math.atan2(-out[0], -out[1]))], ...also };
  return { park, bays, clear: rectFrom(front, t, out, BAY_W * n + 2 * MOUTH_CLEAR, BAY_D) };
}

/**
 * Whether a car park of `n` bays fits at frontage `f` (`carParkAt`): its stretch of road straight, its ends
 * `PARK_CORNER` m from a junction (`offCorners`, `macro.ts` `cornersOf`), its asphalt and `MOUTH_CLEAR` m either
 * end of it on land nothing takes (`ground`) inside the world's edge, and no driveway's mouth of `mouths`
 * within `MOUTH_CLEAR` m of it.
 */
export function carParkFits(roads: readonly RoadPlan[], ground: Ground, f: Frontage, n: number, offCorners: (f: Frontage) => boolean, mouths: readonly Pt[] = []): CarParkLaid | null {
  const road = roads[f.road];
  const ls = lengths(road.points);
  const L = ls[ls.length - 1];
  const half = (BAY_W * n) / 2;
  if (f.s - half < 4 || f.s + half > L - 4) return null;
  if (!straightAround(road.points, f.s, half + BAY_W, PARK_BEND, ls)) return null;
  for (const s of [f.s - half, f.s + half]) if (!offCorners({ ...f, s, p: pointAt(road.points, s, ls).p })) return null;
  const laid = carParkAt(roads, f, n, 0);
  // (Every tile it touches but the road's own, which reach a little past its verge: `roads.ts` `roadMask`.)
  const roads0 = ground.roadTiles();
  const bad = tilesOf(laid.clear, 0, (x, z) => x < EDGE || z < EDGE || x >= ground.w - EDGE || z >= ground.d - EDGE || (!roads0[z * ground.w + x] && ground.takenAt(x + 0.5, z + 0.5)));
  if (bad) return null;
  const { p, t } = pointAt(road.points, f.s, ls);
  const out = leftOf(t).map((v) => v * f.side) as Pt;
  for (const m of mouths) {
    const [u, v] = [(m[0] - p[0]) * t[0] + (m[1] - p[1]) * t[1], (m[0] - p[0]) * out[0] + (m[1] - p[1]) * out[1]];
    const reach = road.width / 2 + 0.5;
    if (Math.hypot(Math.max(0, Math.abs(u) - half), Math.max(0, reach - v, v - reach - BAY_D)) < MOUTH_CLEAR) return null;
  }
  return laid;
}

/**
 * Pairs of bays by the verge (a single one for the last) along the first of `frontage` each fits (`carParkFits`), of
 * area `area`, till they make `spaces` bays; each car park's land taken on `ground` as it goes down. The car parks laid.
 */
export function vergeParking(roads: readonly RoadPlan[], ground: Ground, frontage: readonly Frontage[], spaces: number, area: number, offCorners: (f: Frontage) => boolean = () => true, mouths: readonly Pt[] = []): CarParkLaid[] {
  const out: CarParkLaid[] = [];
  for (let n = spaces; n > 0; n = spaces - out.reduce((k, p) => k + p.bays.length, 0)) {
    const want = Math.min(2, n);
    let laid: CarParkLaid | null = null;
    for (const f of frontage) if ((laid = carParkFits(roads, ground, f, want, offCorners, mouths))) break;
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

/** The box (town tiles) a bay covers: its `BAY_W` × `BAY_D` turned to its yaw. */
export function bayRect(b: Pick<ParkingBay, 'x' | 'z' | 'yaw'>): Rect {
  const [ax, az] = [Math.abs(Math.sin(b.yaw)), Math.abs(Math.cos(b.yaw))];
  const hw = (BAY_W * az + BAY_D * ax) / 2;
  const hd = (BAY_W * ax + BAY_D * az) / 2;
  const [x0, z0] = [Math.floor(b.x - hw), Math.floor(b.z - hd)];
  return { x: x0, z: z0, w: Math.ceil(b.x + hw) - x0, d: Math.ceil(b.z + hd) - z0 };
}

/** Where the houses of `slots` have their driveways' mouths: each driveway's end at its lot's front (town metres). */
export function driveMouths(t: Templates, slots: readonly PlotSlot[], footprint: (def: string) => [number, number]): Pt[] {
  const out: Pt[] = [];
  for (const s of slots) {
    const h = s.kind === 'house' ? houseTemplate(t, s.template) : undefined;
    if (!h) continue;
    const { width, depth } = slotDims(t, s);
    const turn = turner(s.x, s.z, width, depth, s.facing);
    for (const r of driveways(onLot(t, h, s, footprint), width, depth, footprint).slice(0, 1)) out.push(turn.point([r.x + r.w / 2, 0]));
  }
  return out;
}
