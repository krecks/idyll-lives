/**
 * What building a ray from the camera meets first (a click on a lot's house or building): each one
 * as a prism, its footprint raised from `y0` to `y1` (its roof included). The footprint may be any
 * simple polygon.
 */

import { MOST_STOREYS } from './townBuildings';

type V3 = { readonly x: number; readonly y: number; readonly z: number };
type Footprint = readonly (readonly [number, number])[];

/** Whether (`x`, `z`) is inside `poly` (even-odd). */
function insidePoly(poly: Footprint, x: number, z: number): boolean {
  let inside = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const [xi, zi] = poly[i];
    const [xj, zj] = poly[j];
    if (zi > z !== zj > z && x < ((xj - xi) * (z - zi)) / (zj - zi) + xi) inside = !inside;
  }
  return inside;
}

/**
 * How tall a lot's house stands to its walls' top (m): its own storeys (`wall` m each), or a lot with a look closed
 * (`WorldMeta.lotLooks`: a named place's, a flats building's) as the street draws it, its real storeys (`storey` m each,
 * the town's, at most `MOST_STOREYS`: `townBuildings.ts` `lotLookBuilding`).
 */
export function lotWallsHeight(storeys: number | undefined, look: object | undefined, wall: number, storey: number): number {
  return look ? Math.max(1, Math.min(MOST_STOREYS, Math.round(storeys ?? 1))) * storey : (storeys ?? 1) * wall;
}

/** A rectangle `[x0, z0, x1, z1]` as a footprint. */
export function rectFootprint(x0: number, z0: number, x1: number, z1: number): Footprint {
  return [
    [x0, z0],
    [x1, z0],
    [x1, z1],
    [x0, z1],
  ];
}

/**
 * The distance along the ray (origin `o`, direction `d`) to where it enters the prism over `poly`
 * from `y0` to `y1` (0 when it starts inside), or Infinity when it misses.
 */
export function rayPrism(o: V3, d: V3, poly: Footprint, y0: number, y1: number): number {
  if (poly.length < 3) return Infinity;
  // (The stretch of the ray between the two heights.)
  let t0 = 0;
  let t1 = Infinity;
  if (Math.abs(d.y) < 1e-9) {
    if (o.y < y0 || o.y > y1) return Infinity;
  } else {
    let a = (y0 - o.y) / d.y;
    let b = (y1 - o.y) / d.y;
    if (a > b) [a, b] = [b, a];
    t0 = Math.max(0, a);
    t1 = b;
    if (t1 < t0) return Infinity;
  }
  // (Already over the footprint where that stretch starts: in through the top, or the bottom.)
  if (insidePoly(poly, o.x + d.x * t0, o.z + d.z * t0)) return t0;
  // (Else in through a wall: the nearest edge crossed within the stretch.)
  let best = Infinity;
  for (let i = 0; i < poly.length; i++) {
    const [ax, az] = poly[i];
    const [bx, bz] = poly[(i + 1) % poly.length];
    const ex = bx - ax;
    const ez = bz - az;
    const den = d.x * ez - d.z * ex;
    if (Math.abs(den) < 1e-12) continue;
    const px = ax - o.x;
    const pz = az - o.z;
    const t = (px * ez - pz * ex) / den;
    const s = (px * d.z - pz * d.x) / den;
    if (s >= 0 && s <= 1 && t >= t0 && t <= t1 && t < best) best = t;
  }
  return best;
}
