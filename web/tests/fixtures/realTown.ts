/**
 * The real town's fixture as the renderer and the UI get it: the world's structure of the
 * simulation's `tests/towns/flats.json` (a flats building of two storeys on lot 0, its four flats
 * as plots 1 to 4, a household in each, two bays in its front strip, a car park of four bays down
 * the street), with the meta a generated town carries (its road, centre and square, a court paved
 * in front of the building, the building's look). For tests of what reads flats, bays, car parks
 * and the paving; the walls and rooms are the file's, on the ground and on storey 1.
 */

import type { CarPark, FlatInfo, Opening, ParkingBay, RoadPlan, WallEdge, WorldMeta, WorldStructure } from '../../src/core/protocol';

/** The town's size (m), and rows a storey (storey `k` is rows `k * STOREY ..`; the basement's block after the three above). */
export const W = 40;
export const STOREY = 40;
/** The flats building's lot (plot 0) and its flats' ids. */
export const BUILDING = 0;
export const FLATS = [1, 2, 3, 4] as const;

/** A wall from (x0, z0) to (x1, z1) along a grid line, as its edges. */
function run(x0: number, z0: number, x1: number, z1: number): WallEdge[] {
  const out: WallEdge[] = [];
  if (z0 === z1) for (let x = Math.min(x0, x1); x < Math.max(x0, x1); x++) out.push({ axis: 'h', x, z: z0 });
  else for (let z = Math.min(z0, z1); z < Math.max(z0, z1); z++) out.push({ axis: 'v', x: x0, z });
  return out;
}

/** The building's walls, doors and windows on storey `k`, and its rooms there (the flats, their bathrooms, the hall). */
function storey(k: number, rooms: Uint16Array): { walls: WallEdge[]; openings: Opening[] } {
  const o = k * STOREY;
  const segments: [number, number, number, number][] = [
    [5, 8, 22, 8], [5, 17, 22, 17], [5, 8, 5, 17], [22, 8, 22, 17],
    [12, 8, 12, 17], [15, 8, 15, 17],
    [5, 11, 8, 11], [8, 8, 8, 11], [19, 11, 22, 11], [19, 8, 19, 11],
  ];
  const walls = segments.flatMap(([x0, z0, x1, z1]) => run(x0, z0 + o, x1, z1 + o));
  const openings: Opening[] = [
    { axis: 'h', x: 6, z: 11 + o, kind: 'door' },
    { axis: 'h', x: 20, z: 11 + o, kind: 'door' },
    { axis: 'v', x: 12, z: 15 + o, kind: 'door' },
    { axis: 'v', x: 15, z: 15 + o, kind: 'door' },
    { axis: 'h', x: 9, z: 17 + o, kind: 'window' },
    { axis: 'h', x: 17, z: 17 + o, kind: 'window' },
    { axis: 'v', x: 5, z: 14 + o, kind: 'window' },
    { axis: 'v', x: 22, z: 14 + o, kind: 'window' },
    ...(k === 0
      ? ([
          { axis: 'h', x: 13, z: 17, kind: 'door' },
          { axis: 'h', x: 13, z: 8, kind: 'door' },
        ] as Opening[])
      : []),
  ];
  // (Rooms by number: each flat's main room and bathroom, the hall; a storey's own numbers.)
  const room = (x0: number, z0: number, x1: number, z1: number, n: number) => {
    for (let z = z0; z < z1; z++) for (let x = x0; x < x1; x++) rooms[(z + o) * W + x] = n;
  };
  const n = k * 5;
  room(5, 8, 12, 17, n + 1);
  room(5, 8, 8, 11, n + 2);
  room(15, 8, 22, 17, n + 3);
  room(19, 8, 22, 11, n + 4);
  room(12, 8, 15, 17, n + 5);
  return { walls, openings };
}

/** The street along the front, as the town file has it. */
const ROAD: RoadPlan = { points: [[0.5, 34.5], [39.5, 34.5]], width: 7, ends: ['out', 'out'], kind: 'street', name: 'Mill Lane' };

/** The flats: two a storey, the hall between them; each one's door off the hall, on its storey's rows. */
const flats: FlatInfo[] = FLATS.map((id, i) => {
  const [left, storeyOf] = [i % 2 === 0, Math.floor(i / 2)];
  return { id, of: BUILDING, storey: storeyOf, x: left ? 5 : 15, z: 8, w: 7, d: 9, name: `Flat ${id}, Mill Court`, door: [left ? 12 : 14, 15 + storeyOf * STOREY] };
});

/** The car park's bays (nose in, away from the street) and the car park. */
const parking: ParkingBay[] = [27.5, 30.5, 33.5, 36.5].map((x) => ({ x, z: 27.5, yaw: 3.1416, area: 0, park: 0 }));
const carParks: CarPark[] = [{ area: 0, poly: [[26, 24.5], [26, 30.5], [38, 30.5], [38, 24.5]], sign: [25.5, 30.5, 0] }];

/** The real town's structure (a fresh copy each call). */
export function realTown(): WorldStructure {
  const rooms = new Uint16Array(W * STOREY * 4);
  const [ground, up] = [storey(0, rooms), storey(1, rooms)];
  const meta: WorldMeta = {
    kind: 'town',
    name: 'Mill End',
    seed: 7,
    roads: [ROAD],
    facing: [2],
    centre: [{ area: 0, poly: [[0, 18], [0, 40], [40, 40], [40, 18]], square: [[26, 18], [26, 24], [38, 24], [38, 18]] }],
    paving: [{ kind: 'court', poly: [[4, 17], [4, 30], [24, 30], [24, 17]], plot: BUILDING }],
    lotLooks: { [BUILDING]: { use: 'flats', storeys: 2, roof: 'flat', look: 3 } },
  };
  return {
    version: 1,
    lotVersion: 1,
    mode: 'living',
    width: W,
    depth: STOREY * 4,
    storeys: 3,
    storeyDepth: STOREY,
    basement: 3,
    objects: [],
    sims: [],
    households: [
      { id: 0, name: 'Ash', plot: 1, player: true },
      { id: 1, name: 'Birch', plot: 2, player: false },
      { id: 2, name: 'Cole', plot: 3, player: false },
      { id: 3, name: 'Dale', plot: 4, player: false },
    ],
    plots: [{ id: BUILDING, name: 'Mill Court', x: 4, z: 4, w: 20, d: 26, public: false, entry: [13.5, 17.5], house: [5, 8, 22, 17], storeys: 2, kind: 'flats' }],
    flats: flats.map((f) => ({ ...f, door: [...f.door] })),
    exits: [[0.5, 34.5], [39.5, 34.5]],
    rooms,
    walls: [...ground.walls, ...up.walls],
    openings: [...ground.openings, ...up.openings],
    meta,
    meshes: {} as WorldStructure['meshes'],
    parking: parking.map((b) => ({ ...b })),
    carParks: carParks.map((c) => ({ ...c, poly: c.poly.map((p) => [...p]) })),
  };
}
