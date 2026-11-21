/**
 * A town's public parking: marked bays in rows (car parks) off a straight stretch of street. A car parks in a bay
 * nose in (the way the sim turns into a driveway: sim-core `cars::parking_bay`) and backs out of it; a lone pair by
 * the verge is a car park of its own. `carParkAt` lays one out; the generator puts one by the square, one by each big
 * place and pairs along the verges (`macro.ts`), and keeps every bay clear of every driveway (`clearOfDrives`).
 */

import type { CarPark, ParkingBay } from '../core/protocol';
import { EDGE } from './centre';
import { flatsTemplate } from './flats';
import { add, r2, rectFrom, tilesOf } from './footprints';
import { leftOf, nearest, pointAt, RoadIndex, roadIndex, roadLengths, straightAround, type Pt, type RoadPlan } from './roads';
import { boundsOf, houseDrive, houseTemplate, lineMeets, onLot, turnOnto, type Frontage, type Ground, type PlotSlot, type Rect, type Templates } from './town';
import * as pmath from './pmath';

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
export const PARK_BEND = 0.3;
/** How far a single bay's stretch may turn where an area has no straighter one for its share (`vergeParking`'s `bend`). */
export const LOOSE_BEND = 0.6;
/** How far off the grid's axes (radians) a row of more than two bays may run (`carParkFits`). */
const ROW_SLANT = 0.35;
/** A main town's car park by its square goes on the streets this near it (m, `bySquare`), one by a big place on its road this far past its lot's half-size at most (`besidePlace`). */
const SQUARE_REACH = 120;
const PLACE_REACH = 40;

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
  const ls = roadLengths(road);
  const { p, t } = pointAt(road.points, f.s, ls);
  const out = leftOf(t).map((v) => v * f.side) as Pt;
  const reach = road.width / 2 + 0.5;
  const bays = Array.from({ length: n }, (_, k): ParkingBay => {
    const mid = add(add(p, t, BAY_WIDE * (k - (n - 1) / 2)), out, reach + BAY_DEEP / 2);
    const q = pointAt(road.points, nearest(road.points, mid[0], mid[1], ls).s, ls).p;
    return { x: r2(mid[0]), z: r2(mid[1]), yaw: r4(pmath.atan2(mid[0] - q[0], mid[1] - q[1])), area };
  });
  const front = add(p, out, reach);
  const poly = rectFrom(front, t, out, BAY_WIDE * n, BAY_DEEP).map(([x, z]): [number, number] => [r2(x), r2(z)]);
  const sign = add(add(p, t, (BAY_WIDE * n) / 2 + 1), out, reach + 0.4);
  const park: CarPark = { area, poly, sign: [r2(sign[0]), r2(sign[1]), r4(pmath.atan2(-out[0], -out[1]))], ...also };
  return { park, bays, clear: rectFrom(front, t, out, BAY_WIDE * n + 2 * MOUTH_CLEAR, BAY_DEEP) };
}

/** What a car park is checked against and laid as (`carParkFits`): the driveways its bays keep clear of, how far its stretch may bend (`PARK_BEND` unless given), its area (0 unless given) and what else it is (`CarPark.serves`, `square`). */
export interface ParkAs {
  drives?: readonly DriveLine[];
  bend?: number;
  area?: number;
  also?: Pick<CarPark, 'serves' | 'square'>;
}

/**
 * Whether a car park of `n` bays fits at frontage `f` (`carParkAt`, of `as`'s area and `also`): its stretch of road
 * straight (`as.bend`), its ends `PARK_CORNER` m from a junction (`offCorners`, `macro.ts` `cornersOf`), its asphalt
 * and `MOUTH_CLEAR` m either end of it on land nothing takes (`ground`) inside the world's edge, every bay clear of
 * every driveway of `as.drives` (`clearOf`) and nearer its own street than any other road.
 */
export function carParkFits(roads: readonly RoadPlan[], ground: Ground, f: Frontage, n: number, offCorners: (f: Frontage) => boolean, { drives = [], bend = PARK_BEND, area = 0, also = {} }: ParkAs = {}): CarParkLaid | null {
  const road = roads[f.road];
  const ls = roadLengths(road);
  const L = ls[ls.length - 1];
  const half = (BAY_WIDE * n) / 2;
  if (f.s - half < 4 || f.s + half > L - 4) return null;
  if (!straightAround(road.points, f.s, half + BAY_WIDE, bend, ls)) return null;
  // (A row of three or more only along a street running near enough along x or z: the cars' door stands between the
  // bays are tiles, and a row across the grid leaves its middle cars' doors none a driver can get to.)
  if (n > 2) {
    const { t } = pointAt(road.points, f.s, ls);
    const a = pmath.atan2(Math.abs(t[1]), Math.abs(t[0]));
    if (Math.min(a, Math.PI / 2 - a) > ROW_SLANT) return null;
  }
  for (const s of [f.s - half, f.s + half]) if (!offCorners({ ...f, s, p: pointAt(road.points, s, ls).p })) return null;
  const laid = carParkAt(roads, f, n, area, also);
  // (Every tile it touches but its own road's, which reach a little past its verge: `roads.ts` `roadMask`.)
  const roads0 = ground.roadTiles();
  const { p: p0, t: t0 } = pointAt(road.points, f.s, ls);
  const out0 = leftOf(t0).map((v) => v * f.side) as Pt;
  const own = (x: number, z: number) => roads0[z * ground.w + x] === 1 && (x + 0.5 - p0[0]) * out0[0] + (z + 0.5 - p0[1]) * out0[1] < road.width / 2 + 1.5;
  const bad = tilesOf(laid.clear, 0, (x, z) => x < EDGE || z < EDGE || x >= ground.w - EDGE || z >= ground.d - EDGE || (!own(x, z) && ground.takenAt(x + 0.5, z + 0.5)));
  if (bad) return null;
  // (No bay by a driveway, and every bay nearer its own street than any other road.)
  if (!clearOf(laid.bays, drives)) return null;
  const index = indexOf(roads);
  for (const b of laid.bays) if (nearestRoad(roads, index, b.x, b.z, nearest(road.points, b.x, b.z).dist + 1) !== f.road) return null;
  return laid;
}

/**
 * A car park of `few` to `most` bays (the most that fit) along the first of `frontage` it fits (`carParkFits`, laid as
 * `as` says), its land taken on `ground`: the car park, or null.
 */
export function parkAlong(roads: readonly RoadPlan[], ground: Ground, frontage: readonly Frontage[], [few, most]: readonly [number, number], offCorners: (f: Frontage) => boolean, as: ParkAs = {}): CarParkLaid | null {
  for (let n = most; n >= few; n--) {
    for (const f of frontage) {
      const laid = carParkFits(roads, ground, f, n, offCorners, as);
      if (!laid) continue;
      takeCarPark(ground, laid);
      return laid;
    }
  }
  return null;
}

/** Of `frontage`, that on the streets near a main town's square (its middle `sq`; no country road), where its car park goes. */
export function bySquare(roads: readonly RoadPlan[], frontage: readonly Frontage[], sq: Pt): Frontage[] {
  return frontage.filter((f) => pmath.hypot(f.p[0] - sq[0], f.p[1] - sq[1]) < SQUARE_REACH && roads[f.road].kind !== 'country');
}

/** Of `frontage` (along a big place's road), that on the place's side of it near its lot `r` (`Frontage.far` from its middle), where its car park goes. */
export function besidePlace(frontage: readonly Frontage[], side: 1 | -1, r: Rect): Frontage[] {
  return frontage.filter((g) => g.side === side && g.far < Math.max(r.w, r.d) / 2 + PLACE_REACH);
}

/**
 * Pairs of bays by the verge (a single one for the last) along the first of `frontage` each fits (`carParkFits`), of
 * area `area`, till they make `spaces` bays; each car park's land taken on `ground` as it goes down. The car parks laid.
 */
export function vergeParking(roads: readonly RoadPlan[], ground: Ground, frontage: readonly Frontage[], spaces: number, area: number, offCorners: (f: Frontage) => boolean = () => true, drives: readonly DriveLine[] = [], bend = PARK_BEND): CarParkLaid[] {
  const out: CarParkLaid[] = [];
  for (let n = spaces; n > 0; n = spaces - out.reduce((k, p) => k + p.bays.length, 0)) {
    const want = Math.min(2, n);
    let laid: CarParkLaid | null = null;
    for (const f of frontage) if ((laid = carParkFits(roads, ground, f, want, offCorners, { drives, bend, area }))) break;
    if (!laid) break;
    takeCarPark(ground, laid);
    out.push(laid);
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
  return boundsOf(p.poly);
}

/** The box (town tiles) a bay covers: its `BAY_WIDE` × `BAY_DEEP` turned to its yaw. */
export function bayRect(b: Pick<ParkingBay, 'x' | 'z' | 'yaw'>): Rect {
  const [ax, az] = [Math.abs(pmath.sin(b.yaw)), Math.abs(pmath.cos(b.yaw))];
  const hw = (BAY_WIDE * az + BAY_DEEP * ax) / 2;
  const hd = (BAY_WIDE * ax + BAY_DEEP * az) / 2;
  const [x0, z0] = [Math.floor(b.x - hw), Math.floor(b.z - hd)];
  return { x: x0, z: z0, w: Math.ceil(b.x + hw) - x0, d: Math.ceil(b.z + hd) - z0 };
}

/**
 * The driveways of the houses of `slots` (and the bays of its flats buildings) as `layoutTown` lays them (`town.ts`
 * `houseDrive` and `lineMeets`), each as `DriveLine`.
 */
export function driveLines(t: Templates, slots: readonly PlotSlot[], roads: readonly RoadPlan[], footprint: (def: string) => [number, number]): DriveLine[] {
  const index = roadIndex(roads);
  const out: DriveLine[] = [];
  for (const s of slots) {
    const turn = turnOnto(t, s);
    const line = (r: Rect, meets: { road: number; s: number } | null) => {
      if (meets) out.push({ inner: turn.point([r.x + r.w / 2, r.d]), mouth: pointAt(roads[meets.road].points, meets.s).p });
    };
    // (A flats building's bays are its flats' driveways.)
    const f = s.kind === 'flats' ? flatsTemplate(t, s.template) : undefined;
    for (const { rect: r } of f?.bays ?? []) line(r, lineMeets(roads, index, turn, r));
    const h = s.kind === 'house' ? houseTemplate(t, s.template) : undefined;
    const side = h && houseDrive(t, s, onLot(t, h, s, footprint), roads, index, footprint);
    if (side) line(side.r, side.meets);
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
  return pmath.hypot(x - ax - dx * k, z - az - dz * k);
}

/** Grid cells (m) the driveways are filed under for `clearOf`. */
const DRIVE_CELL = 16;
/** Each list of driveways filed by grid cell (`DRIVE_CELL`): each under every cell its line comes within `MOUTH_CLEAR` m of, about. */
const driveCells = new WeakMap<readonly DriveLine[], Map<number, DriveLine[]>>();
function drivesNear(drives: readonly DriveLine[], x: number, z: number): readonly DriveLine[] {
  let cells = driveCells.get(drives);
  if (!cells) {
    cells = new Map();
    for (const d of drives) {
      const [x0, x1] = [Math.min(d.inner[0], d.mouth[0]) - MOUTH_CLEAR, Math.max(d.inner[0], d.mouth[0]) + MOUTH_CLEAR];
      const [z0, z1] = [Math.min(d.inner[1], d.mouth[1]) - MOUTH_CLEAR, Math.max(d.inner[1], d.mouth[1]) + MOUTH_CLEAR];
      for (let cz = Math.floor(z0 / DRIVE_CELL); cz <= Math.floor(z1 / DRIVE_CELL); cz++) {
        for (let cx = Math.floor(x0 / DRIVE_CELL); cx <= Math.floor(x1 / DRIVE_CELL); cx++) {
          const key = cx * 65536 + cz;
          (cells.get(key) ?? cells.set(key, []).get(key)!).push(d);
        }
      }
    }
    driveCells.set(drives, cells);
  }
  return cells.get(Math.floor(x / DRIVE_CELL) * 65536 + Math.floor(z / DRIVE_CELL)) ?? [];
}

/**
 * Whether every bay's middle is `MOUTH_CLEAR` m from every driveway's mouth and `DRIVE_CLEAR` m from its line (sim-core
 * `by_a_mouth`). Only the driveways filed near a bay are measured (`drivesNear`): one further off is clear of it.
 */
export function clearOf(bays: readonly Pick<ParkingBay, 'x' | 'z'>[], drives: readonly DriveLine[]): boolean {
  return bays.every((b) => drivesNear(drives, b.x, b.z).every((d) => pmath.hypot(b.x - d.mouth[0], b.z - d.mouth[1]) >= MOUTH_CLEAR && offSegment([b.x, b.z], d.inner, d.mouth) >= DRIVE_CLEAR));
}

/** Each list of roads' index for `nearestRoad`, and how many roads it was made for (made again once more are added). */
const indexes = new WeakMap<readonly RoadPlan[], { n: number; index: RoadIndex }>();
function indexOf(roads: readonly RoadPlan[]): RoadIndex {
  const had = indexes.get(roads);
  if (had && had.n === roads.length) return had.index;
  const index = new RoadIndex(roads);
  indexes.set(roads, { n: roads.length, index });
  return index;
}

/**
 * The road (not a trail) whose middle line is nearest a point, by index (-1: none within `reach` m): of those the index
 * finds within `reach`, the nearest as `nearest` measures them, the first of those as near.
 */
function nearestRoad(roads: readonly RoadPlan[], index: RoadIndex, x: number, z: number, reach: number): number {
  let best = { r: -1, d: Infinity };
  for (const i of index.roadsNear(x, z, reach)) {
    if (roads[i].kind === 'trail') continue;
    const d = nearest(roads[i].points, x, z).dist;
    if (d < best.d) best = { r: i, d };
  }
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
