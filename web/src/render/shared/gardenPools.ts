/**
 * Where light lands on the lawn at night (engine-neutral; the renderers' `GardenPools` draw the pools): a garden
 * lamp's round its foot, as bright in its middle as its light there (`gardenPool`), and a lit room's through its
 * windows, a soft patch on the ground outside each (`windowPools`).
 */

import type { WorldStructure } from '../../core/protocol';
import type { RoomLight } from './house';
import { tuned } from '../tune';

/**
 * A pool of light on the ground: where it lands (m, its middle), how far it reaches (along x; `depth` along z, for
 * one that isn't round) and how bright its middle is (0..1), and what it follows on and off: a lamp, or a room's
 * lights (a window's). `half`: only the half of it on that side of its middle, towards +x, -x, +z or -z (a
 * window's: its middle is at the foot of the wall, and the house stands on the other half).
 */
export interface GardenPool {
  x: number;
  y: number;
  z: number;
  radius: number;
  depth?: number;
  half?: 'x+' | 'x-' | 'z+' | 'z-';
  strength: number;
  lamp?: number;
  room?: number;
}

/** A garden lamp's pool of light on the lawn (`gardenPool`): its middle's brightness per `intensity / height²` (the street lamp's 0.3, the street's own up to 0.6), at most this, and at least this wide (m). */
const POOL_GAIN = 1.2;
const POOL_MAX = 0.4;
const POOL_RADIUS_MIN = 1.5;

/**
 * Where a garden lamp's light lands on the lawn (`GardenPools`): round its foot, as far as twice
 * its height (within its reach; a wall light aimed out, a smaller one in front of its wall, its
 * edge at the wall), as bright in the middle as its light there, `intensity / height²`, scaled to
 * the pools' look and capped (a fire pit's). `row`: the ground storey's first lot row.
 */
export function gardenPool(l: RoomLight, row: number): GardenPool {
  const h = Math.max(l.y, 0.5);
  const out = !!l.dir && l.dir[1] < 0 && Math.hypot(l.dir[0], l.dir[2]) > 0.1;
  const radius = Math.min(Math.max((out ? 1.2 : 2) * h, POOL_RADIUS_MIN), l.range ?? 4.5);
  const along = out ? (radius * 0.9) / Math.hypot(l.dir![0], l.dir![2]) : 0;
  const strength = Math.min(((l.power ?? 1) * POOL_GAIN) / (h * h), POOL_MAX);
  return { x: l.x + (l.dir?.[0] ?? 0) * along, y: 0, z: l.z - row + (l.dir?.[2] ?? 0) * along, radius, strength, lamp: l.lamp };
}

/**
 * A lit room's window on the garden: half a pool against the wall under it, brightest at the wall's foot (`near`
 * m out from the wall's line: just off its face) and gone `far` m out, `spread` m wider than the window on either
 * side, as bright there as `strength` (a garden lamp's pool: 0.3 to 0.4) in a room wholly lamplit
 * (`Lamps.windowLight`). Against the wall it reads as the room's light spilling out; a whole pool further out,
 * where a lamp indoors would really send its light over the sill, read as a patch of its own on the lawn.
 */
export const WINDOW_POOL = tuned('windowPool', { near: 0.1, far: 3.4, spread: 0.7, strength: 0.32 });

/**
 * The pools of light the viewed house's lit rooms throw on the garden through their ground-floor windows
 * (`GardenPools`): one outside each run of window along an outside wall (a wide window is one pool), following the
 * room behind it (`GardenPool.room`). `ground`: whether a lot row is on the ground storey; `row`: that storey's
 * first lot row; `inView`: whether a tile (in lot rows) is the viewed lot's. Windows in diagonal walls, on upper
 * storeys and between two rooms throw none.
 */
export function windowPools(world: Pick<WorldStructure, 'openings' | 'rooms' | 'width' | 'depth'>, ground: (z: number) => boolean, row: number, inView: (x: number, z: number) => boolean): GardenPool[] {
  const W = world.width;
  // (Rows of another storey are not this storey's rooms: outdoors, as far as its windows go.)
  const roomAt = (x: number, z: number) => (x >= 0 && z >= 0 && x < W && z < world.depth && ground(z) ? (world.rooms[z * W + x] ?? 0) : 0);
  // Runs of window along one side of one wall line, by the room behind them: where each edge starts.
  const runs = new Map<string, { axis: 'h' | 'v'; line: number; out: number; room: number; at: number[] }>();
  for (const o of world.openings) {
    if (o.kind !== 'window' || !ground(o.z) || !inView(o.x, o.z)) continue;
    // (An 'h' edge lies on the line z, between rows z - 1 and z, from x to x + 1; a 'v' edge on the line x.)
    const a = o.axis === 'h' ? roomAt(o.x, o.z - 1) : roomAt(o.x - 1, o.z);
    const b = roomAt(o.x, o.z);
    if ((a === 0) === (b === 0)) continue;
    const out = a === 0 ? -1 : 1;
    const room = a === 0 ? b : a;
    const line = o.axis === 'h' ? o.z : o.x;
    const key = `${o.axis}:${line}:${out}:${room}`;
    let run = runs.get(key);
    if (!run) runs.set(key, (run = { axis: o.axis, line, out, room, at: [] }));
    run.at.push(o.axis === 'h' ? o.x : o.z);
  }
  const { near, far, spread, strength } = WINDOW_POOL;
  const pools: GardenPool[] = [];
  for (const run of runs.values()) {
    run.at.sort((p, q) => p - q);
    for (let i = 0; i < run.at.length; ) {
      let j = i;
      while (j + 1 < run.at.length && run.at[j + 1] === run.at[j] + 1) j++;
      const along = (run.at[i] + run.at[j] + 1) / 2;
      const wide = (run.at[j] + 1 - run.at[i]) / 2 + spread;
      const foot = run.line + run.out * near;
      const out = run.out > 0 ? '+' : '-';
      pools.push(run.axis === 'h' ? { x: along, y: 0, z: foot - row, radius: wide, depth: far, half: `z${out}`, strength, room: run.room } : { x: foot, y: 0, z: along - row, radius: far, depth: wide, half: `x${out}`, strength, room: run.room });
      i = j + 1;
    }
  }
  return pools;
}
