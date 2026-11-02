/**
 * Build mode's Garage tool, as far as it can be worked out without the simulation (which checks
 * everything again, and that nobody is shut in: sim-core `garages.rs`): where a garage goes under
 * the pointer and whether it fits there.
 *
 * A garage is a single one (`GARAGE_WIDTH` × `GARAGE_DEPTH` between its walls' centre lines, as the
 * town builds them), its door facing any of four ways (`rot`, as a thing's: 0 towards +z, 1 +x,
 * 2 -z, 3 -x). Placed freely it takes up `GARAGE_TILES` (turned with it), its walls in the middle
 * of them; at the end of the driveway it goes where the town puts garages (`drivewayGarage`).
 */

import { GARAGE_DEPTH, GARAGE_WIDTH } from '../render/shared/cars';

/** The tiles a garage placed freely takes up: across its door, and deep. */
export const GARAGE_TILES: readonly [number, number] = [4, 6];
/** Half a garage's wall (m), outside its centre lines (sim-core `GARAGE_WALL_HALF`). */
const WALL_HALF = 0.07;

/** A garage to build: its walls' centre lines and the way its door faces. */
export interface GaragePlan {
  x0: number;
  z0: number;
  x1: number;
  z1: number;
  rot: number;
}

interface Rect {
  x: number;
  z: number;
  w: number;
  d: number;
}

/** The garage facing `rot` whose tiles' middle is nearest (cx, cz), its walls in the middle of its tiles. */
export function freeGarage(cx: number, cz: number, rot: number): GaragePlan {
  const along = rot % 2 === 0;
  const [w, d] = along ? GARAGE_TILES : [GARAGE_TILES[1], GARAGE_TILES[0]];
  const [gw, gd] = along ? [GARAGE_WIDTH, GARAGE_DEPTH] : [GARAGE_DEPTH, GARAGE_WIDTH];
  const x = Math.floor(cx - w / 2 + 0.5);
  const z = Math.floor(cz - d / 2 + 0.5);
  const x0 = x + (w - gw) / 2;
  const z0 = z + (d - gd) / 2;
  return { x0, z0, x1: x0 + gw, z1: z0 + gd, rot };
}

/** The tiles a garage's walls touch (beyond a sliver), as the simulation counts them. */
export function garageTiles(g: Omit<GaragePlan, 'rot'>): [number, number][] {
  const lo = (v: number) => Math.floor(v - WALL_HALF + 0.02);
  const hi = (v: number) => Math.ceil(v + WALL_HALF - 0.02);
  const out: [number, number][] = [];
  for (let z = lo(g.z0); z < hi(g.z1); z++) for (let x = lo(g.x0); x < hi(g.x1); x++) out.push([x, z]);
  return out;
}

/** Whether a point is on a garage (within its walls). */
export function onGarage(g: Omit<GaragePlan, 'rot'>, x: number, z: number): boolean {
  return x >= g.x0 - WALL_HALF && x <= g.x1 + WALL_HALF && z >= g.z0 - WALL_HALF && z <= g.z1 + WALL_HALF;
}

/** What a garage on the home lot has to keep clear of (the lot as the view has it). */
export interface GarageSite {
  /** The home plot (its ground rows). */
  plot: Rect;
  /** The home has a garage already. */
  hasGarage: boolean;
  /** Whether a tile is indoors (in a room). */
  indoors(x: number, z: number): boolean;
  /** Whether a wall or a fence stands on an edge (`h`: along x at line z; `v`: along z at line x), or a diagonal (`d`) across tile (x, z). */
  walled(axis: 'h' | 'v' | 'd', x: number, z: number): boolean;
  /** The name (lower case) of what stands on a tile, if anything does. */
  thingAt(x: number, z: number): string | undefined;
  /** The driveway, where the car parks (null: none). */
  drive: Rect | null;
  /** Whether the household can pay for it. */
  affords: boolean;
}

/**
 * Whether a wall, a fence or a diagonal stands in garage `g` (its walls' thickness, and the wall's,
 * counted), as the simulation has it: one along its outline a little way off is no matter (a garage
 * beside the house, as the town builds them).
 */
function walledIn(g: Omit<GaragePlan, 'rot'>, site: GarageSite): boolean {
  const t = WALL_HALF;
  const [a0, b0, a1, b1] = [g.x0 - t, g.z0 - t, g.x1 + t, g.z1 + t];
  const eps = 0.01;
  const across = (lo: number, hi: number, a: number, b: number) => hi > a + eps && lo < b - eps;
  const [x0, z0, x1, z1] = [Math.floor(a0), Math.floor(b0), Math.ceil(a1), Math.ceil(b1)];
  for (let z = z0; z <= z1; z++) for (let x = x0; x < x1; x++) if (site.walled('h', x, z) && across(z - t, z + t, b0, b1) && across(x, x + 1, a0, a1)) return true;
  for (let z = z0; z < z1; z++) for (let x = x0; x <= x1; x++) if (site.walled('v', x, z) && across(x - t, x + t, a0, a1) && across(z, z + 1, b0, b1)) return true;
  return garageTiles(g).some(([x, z]) => site.walled('d', x, z));
}

/** Whether a wall, a fence, a door or a window on edge (`axis`, x, z) would stand in garage `g` (Build mode keeps them out: sim-core `garage_across`). */
export function edgeInGarage(g: Omit<GaragePlan, 'rot'>, axis: 'h' | 'v' | 'dp' | 'dn', x: number, z: number): boolean {
  const t = WALL_HALF;
  const [a0, b0, a1, b1] = [g.x0 - t, g.z0 - t, g.x1 + t, g.z1 + t];
  const eps = 0.01;
  const across = (lo: number, hi: number, a: number, b: number) => hi > a + eps && lo < b - eps;
  if (axis === 'h') return across(z - t, z + t, b0, b1) && across(x, x + 1, a0, a1);
  if (axis === 'v') return across(x - t, x + t, a0, a1) && across(z, z + 1, b0, b1);
  return across(x, x + 1, a0, a1) && across(z, z + 1, b0, b1);
}

/** Whether garage `g` can go on the home lot (`site`): true, or why not (a short hint). */
export function garageFits(g: GaragePlan, site: GarageSite): true | string {
  if (site.hasGarage) return 'Your home has a garage already';
  const p = site.plot;
  const [a0, b0, a1, b1] = [g.x0 - WALL_HALF, g.z0 - WALL_HALF, g.x1 + WALL_HALF, g.z1 + WALL_HALF];
  if (a0 < p.x - 0.01 || b0 < p.z - 0.01 || a1 > p.x + p.w + 0.01 || b1 > p.z + p.d + 0.01) return 'Has to go on your lot';
  const tiles = garageTiles(g);
  if (tiles.some(([x, z]) => site.indoors(x, z))) return 'Goes outdoors, not in a room';
  if (walledIn(g, site)) return 'A wall or a fence is in the way';
  for (const [x, z] of tiles) {
    const thing = site.thingAt(x, z);
    if (thing) return `The ${thing} is in the way`;
  }
  // (Its door may stand just on the driveway's end, where the town builds them.)
  const d = site.drive;
  const m = 0.1;
  if (d && a0 + m < d.x + d.w && a1 - m > d.x && b0 + m < d.z + d.d && b1 - m > d.z) return "That's where the car parks";
  if (!site.affords) return 'Not enough money';
  return true;
}
