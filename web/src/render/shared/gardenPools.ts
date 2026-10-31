/**
 * Where a garden lamp's light lands on the lawn (engine-neutral; the renderers' `GardenPools` draw
 * the pools): round its foot, as bright in its middle as its light there.
 */

import type { RoomLight } from './house';

/** A garden lamp's pool: where it lands (m, its middle), how far it reaches and how bright its middle is (0..1), and the lamp it follows on and off. */
export interface GardenPool {
  x: number;
  y: number;
  z: number;
  radius: number;
  strength: number;
  lamp?: number;
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
