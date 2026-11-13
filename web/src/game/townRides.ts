/**
 * The bikes and e-scooters a new town gives its households (content `vehicle`, sim-core `ride.rs`):
 * what each household has (`ridesFor`: about a bicycle a grown-up or teen, up to three, more often
 * with no car; a kids' bike a child; an e-scooter now and then) and where they stand on its lot
 * (`placeRides`): outdoors by the house or the garage, their backs to it, clear of the driveway
 * and the car on it, the paths, the doors and what's used in the garden, each reached from the
 * street (round the porch's posts) and leaving everything else reached as before.
 *
 * Plot coordinates throughout (the street along z = 0, as `town.ts` lays a house out).
 */

import type { Content } from '../content/content';
import { hash } from '../render/shared/cars';
import type { HouseholdDraft } from './household';
import { frontTile, sizeOf } from './surfaces';
import type { Door, Placement, Rect, Wall } from './town';

/** What a household rides (content ids). */
export const RIDES = { bike: 'bicycle', kids: 'bicycleChild', scooter: 'eScooter' } as const;
/** The bicycles a household may have (content ids), a city bike twice so it stays the common one: a
 *  household's own bikes need not all be alike, and a town's need not either. */
export const BIKES = ['bicycle', 'bicycle', 'mountainBike', 'eBike', 'roadBike', 'cargoBike'] as const;
/** Bicycles a household gets at most. */
const MOST_BIKES = 3;
/** The chance a teen or grown-up gets a bicycle, with a car at home and without one. */
const BIKE_CHANCE = { car: 0.55, none: 0.9 };
/** The chance a household with someone old enough gets an e-scooter too. */
const SCOOTER_CHANCE = 1 / 6;
/** Children this old or older get a kids' bike (younger ones are carried or walk). */
const KIDS_BIKE_AGE = 4;

/** A number 0..1 for plot `at`, draw `k`: the same each time the town is laid out. */
function draw(at: { x: number; z: number }, k: number): number {
  return hash(at.x, at.z, 70 + k);
}

/** The `k`th bicycle of the household at `at`: one of `BIKES` (leaving out any this content hasn't),
 *  the same each time the town is laid out. */
function bikeFor(content: Content, at: { x: number; z: number }, k: number): string {
  const have = BIKES.filter((def) => content.object(def));
  return have[Math.floor(draw(at, 40 + k) * have.length)] ?? RIDES.bike;
}

/** The rides of `household`, living on the plot at `at` (its corner on the town: what decides), with a car at home or not. */
export function ridesFor(content: Content, household: HouseholdDraft, car: boolean, at: { x: number; z: number }): string[] {
  const stages = household.members.map((m) => content.stageOf(m.age)?.id ?? '');
  const riders = stages.filter((s) => s !== 'baby' && s !== 'child').length;
  const out: string[] = [];
  let k = 0;
  for (let i = 0; i < riders && out.length < MOST_BIKES; i++) {
    if (draw(at, k++) < (car ? BIKE_CHANCE.car : BIKE_CHANCE.none)) out.push(bikeFor(content, at, i));
  }
  if (riders > 0 && draw(at, 20) < SCOOTER_CHANCE) out.push(RIDES.scooter);
  for (const m of household.members) if (content.stageOf(m.age)?.id === 'child' && m.age >= KIDS_BIKE_AGE) out.push(RIDES.kids);
  return out.filter((def) => content.object(def));
}

/** What `placeRides` keeps clear and goes by, in plot coordinates. */
export interface RideLot {
  width: number;
  depth: number;
  walls: readonly Wall[];
  doors: readonly Door[];
  objects: readonly Placement[];
  /** Garden paths and the driveway (the car parks on it), and anything else not to stand on (a farm's fields). */
  paths: readonly Rect[];
  drive: Rect | null;
  /** The garage, and where the player could build one (Build mode's Garage tool): wall boxes. */
  garages: readonly Rect[];
  keep?: readonly Rect[];
  /** Where the posts of the porch over the front door stand (`frontPorchPosts`): nobody walks through them. */
  posts?: readonly [number, number][];
}

/** Tiles kept clear round a door, out from it (its doorstep and the tile past it). */
const DOORSTEP = 2;

/** The tiles right behind a thing, along its back (away from its front tile). */
function backTiles(x: number, z: number, w: number, d: number, rot: number): [number, number][] {
  return frontRow(x, z, w, d, rot + 2);
}

/**
 * The tiles right in front of a thing, along its front: its front tile is one of them (which one, for
 * a thing two wide, depends on how the plot is turned onto the town: all of them are kept free).
 */
function frontRow(x: number, z: number, w: number, d: number, rot: number): [number, number][] {
  const out: [number, number][] = [];
  switch ((rot + 2) % 4) {
    case 0:
      for (let i = 0; i < w; i++) out.push([x + i, z - 1]);
      break;
    case 1:
      for (let i = 0; i < d; i++) out.push([x - 1, z + i]);
      break;
    case 2:
      for (let i = 0; i < w; i++) out.push([x + i, z + d]);
      break;
    default:
      for (let i = 0; i < d; i++) out.push([x + w, z + i]);
  }
  return out;
}

const overlaps = (r: Rect, x: number, z: number, m = 0) => x < r.x + r.w + m && x + 1 > r.x - m && z < r.z + r.d + m && z + 1 > r.z - m;

/**
 * Where `defs` stand on `lot` (placements, in plot coordinates; one that finds no room is left
 * out): each outdoors with its back to the house or the garage (or as near as it goes), near the
 * front door and the others, clear of the driveway by a tile (the car's doors), of the paths,
 * the doorsteps and the tiles things are used from, a tile in from the plot's edge (its fence);
 * its own front tile reached from the street, and every tile reached before still reached.
 * `rand` (0..1, by draw) breaks ties the same way each time.
 */
export function placeRides(content: Content, lot: RideLot, defs: readonly string[], rand: (k: number) => number): Placement[] {
  const { width: W, depth: D } = lot;
  const key = (x: number, z: number) => z * W + x;
  const inLot = (x: number, z: number) => x >= 0 && z >= 0 && x < W && z < D;
  // What stands: the house (its walls' box), the garages and the things on the ground.
  const blocked = new Uint8Array(W * D);
  const block = (x: number, z: number) => inLot(x, z) && (blocked[key(x, z)] = 1);
  const xs = lot.walls.flatMap(([x0, , x1]) => [x0, x1]);
  const zs = lot.walls.flatMap(([, z0, , z1]) => [z0, z1]);
  const house = xs.length ? { x0: Math.min(...xs), z0: Math.min(...zs), x1: Math.max(...xs), z1: Math.max(...zs) } : null;
  const inHouse = (x: number, z: number) => !!house && x >= house.x0 && x < house.x1 && z >= house.z0 && z < house.z1;
  const inGarage = (x: number, z: number) => lot.garages.some((g) => overlaps(g, x, z));
  for (let z = 0; z < D; z++) for (let x = 0; x < W; x++) if (inHouse(x, z) || inGarage(x, z)) block(x, z);
  for (const [x, z] of lot.posts ?? []) block(x, z);
  // Not to stand on: the paths, the driveway and a tile round it, doorsteps, where things are used from.
  const keep = new Uint8Array(W * D);
  const keepAt = (x: number, z: number) => inLot(x, z) && (keep[key(x, z)] = 1);
  for (const o of lot.objects) {
    const def = content.object(o.def);
    const [w, d] = sizeOf(def, o.rot);
    const ground = !def?.layer || def.layer === 'floor';
    for (let dz = 0; dz < d; dz++) for (let dx = 0; dx < w; dx++) (ground ? block : keepAt)(Math.floor(o.x) + dx, Math.floor(o.z) + dz);
    if (def?.interactions.length) keepAt(...frontTile(Math.floor(o.x), Math.floor(o.z), w, d, o.rot));
  }
  for (let z = 0; z < D; z++) {
    for (let x = 0; x < W; x++) {
      if ([...lot.paths, ...(lot.keep ?? [])].some((r) => overlaps(r, x, z)) || (lot.drive && overlaps(lot.drive, x, z, 1))) keepAt(x, z);
    }
  }
  for (const door of lot.doors) {
    // The two tiles either side of the doorway; out from the one outside the house.
    const [a, b]: [number, number][] = door.axis === 'x' ? [[door.x, door.z - 1], [door.x, door.z]] : [[door.x - 1, door.z], [door.x, door.z]];
    const [out, inside] = inHouse(...a) ? [b, a] : [a, b];
    const [sx, sz] = [out[0] - inside[0], out[1] - inside[1]];
    for (let k = 0; k < DOORSTEP; k++) for (const side of [-1, 0, 1]) keepAt(out[0] + sx * k + (sz ? side : 0), out[1] + sz * k + (sx ? side : 0));
  }
  // Where the street gets in: the paths' and the driveway's tiles on the front row.
  const gates: number[] = [];
  for (let x = 0; x < W; x++) if ([...lot.paths, ...(lot.drive ? [lot.drive] : [])].some((r) => overlaps(r, x, 0))) gates.push(key(x, 0));
  const front = lot.doors[0];
  const doorstep: [number, number] = front ? (front.axis === 'x' ? [front.x, front.z - 1] : [front.x - 1, front.z]) : [Math.floor(W / 2), 1];
  if (!gates.length) gates.push(key(...doorstep));
  const reach = (): Uint8Array => {
    const seen = new Uint8Array(W * D);
    const queue = gates.filter((k) => !blocked[k]);
    for (const k of queue) seen[k] = 1;
    for (let i = 0; i < queue.length; i++) {
      const [x, z] = [queue[i] % W, Math.floor(queue[i] / W)];
      for (const [nx, nz] of [[x + 1, z], [x - 1, z], [x, z + 1], [x, z - 1]]) {
        if (!inLot(nx, nz) || blocked[key(nx, nz)] || seen[key(nx, nz)]) continue;
        seen[key(nx, nz)] = 1;
        queue.push(key(nx, nz));
      }
    }
    return seen;
  };
  const before = reach();
  const out: Placement[] = [];
  const taken: [number, number][] = [];
  let n = 0;
  for (const def of defs) {
    const shape = content.object(def);
    const spots: { x: number; z: number; rot: number; score: number }[] = [];
    for (let rot = 0; rot < 4; rot++) {
      const [w, d] = sizeOf(shape, rot);
      for (let z = 1; z + d <= D - 1; z++) {
        for (let x = 1; x + w <= W - 1; x++) {
          const fronts = frontRow(x, z, w, d, rot);
          if (fronts.some(([fx, fz]) => fx < 1 || fz < 1 || fx >= W - 1 || fz >= D - 1 || blocked[key(fx, fz)] || (lot.drive && overlaps(lot.drive, fx, fz)))) continue;
          const [fx, fz] = frontTile(x, z, w, d, rot);
          let free = true;
          for (let dz = 0; dz < d && free; dz++) for (let dx = 0; dx < w && free; dx++) free = !blocked[key(x + dx, z + dz)] && !keep[key(x + dx, z + dz)];
          if (!free) continue;
          // Its back against the house or the garage (or a tile off), near the front door and the others.
          const back = backTiles(x, z, w, d, rot);
          const against = back.every(([bx, bz]) => inHouse(bx, bz) || inGarage(bx, bz));
          const near = back.some(([bx, bz]) => [[1, 0], [-1, 0], [0, 1], [0, -1]].some(([ox, oz]) => inHouse(bx + ox, bz + oz) || inGarage(bx + ox, bz + oz)));
          if (!against && !near) continue;
          const door = Math.abs(fx - doorstep[0]) + Math.abs(fz - doorstep[1]);
          const others = taken.length ? Math.min(...taken.map(([tx, tz]) => Math.abs(tx - x) + Math.abs(tz - z))) : 0;
          spots.push({ x, z, rot, score: door * 0.5 + others + (against ? 0 : 4) + rand(n++) * 3 });
        }
      }
    }
    spots.sort((a, b) => a.score - b.score);
    for (const s of spots.slice(0, 12)) {
      const [w, d] = sizeOf(shape, s.rot);
      for (let dz = 0; dz < d; dz++) for (let dx = 0; dx < w; dx++) blocked[key(s.x + dx, s.z + dz)] = 1;
      const after = reach();
      const ok = frontRow(s.x, s.z, w, d, s.rot).every(([fx, fz]) => after[key(fx, fz)]) && before.every((v, k) => !v || after[k] || blocked[k]);
      if (ok) {
        out.push({ def, x: s.x, z: s.z, rot: s.rot });
        taken.push([s.x, s.z]);
        for (const [fx, fz] of frontRow(s.x, s.z, w, d, s.rot)) keepAt(fx, fz);
        break;
      }
      for (let dz = 0; dz < d; dz++) for (let dx = 0; dx < w; dx++) blocked[key(s.x + dx, s.z + dz)] = 0;
    }
  }
  return out;
}
