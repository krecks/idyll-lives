/**
 * Footprints on the town's ground: polygons in town metres (x, z), counter-clockwise seen from above
 * (+y up, x right: z runs down the screen, so the signed area x·z' − x'·z is negative), the tiles
 * they cover, whether they overlap, and whether one faces the road nearest it. Shared by the
 * generator's passes (the centre, the dense pass, the parking) and the checks of a laid town.
 */

import { inside, lengths, pointAt, type RoadIndex, type Pt, type RoadPlan } from './roads';
import type { Rect } from './town';

/** A building's side runs within this (radians) of the way the road nearest it runs, or square to it. */
export const FACING = (15 * Math.PI) / 180;

const add = (a: Pt, b: Pt, k = 1): Pt => [a[0] + b[0] * k, a[1] + b[1] * k];
const dist = (a: Pt, b: Pt) => Math.hypot(a[0] - b[0], a[1] - b[1]);

/** Twice the signed area of a polygon (x·z' − x'·z summed): negative when it goes counter-clockwise seen from above. */
export function signedArea2(poly: readonly Pt[]): number {
  let n = 0;
  for (let i = 0; i < poly.length; i++) {
    const [a, b] = [poly[i], poly[(i + 1) % poly.length]];
    n += a[0] * b[1] - b[0] * a[1];
  }
  return n;
}

/** `poly` wound counter-clockwise seen from above (x right, z down the screen: its signed area negative), turned in place if it was not. */
export function ccw(poly: Pt[]): Pt[] {
  return signedArea2(poly) > 0 ? poly.reverse() : poly;
}

/** A rectangle `w` along `t` and `d` along `n` (unit, square to each other) from the middle of its front edge `c`, counter-clockwise seen from above. */
export function rectFrom(c: Pt, t: Pt, n: Pt, w: number, d: number): Pt[] {
  const a = add(c, t, -w / 2);
  const b = add(c, t, w / 2);
  const poly: Pt[] = [a, b, add(b, n, d), add(a, n, d)];
  return signedArea2(poly) > 0 ? poly.reverse() : poly;
}

/** Whether two convex polygons overlap by more than `slack` m (separating axes): touching is no overlap. */
export function overlaps(a: readonly Pt[], b: readonly Pt[], slack = 0.05): boolean {
  for (const poly of [a, b]) {
    for (let i = 0; i < poly.length; i++) {
      const [p, q] = [poly[i], poly[(i + 1) % poly.length]];
      const l = Math.hypot(q[0] - p[0], q[1] - p[1]) || 1;
      const ax: Pt = [-(q[1] - p[1]) / l, (q[0] - p[0]) / l];
      const span = (ps: readonly Pt[]) => {
        let [lo, hi] = [Infinity, -Infinity];
        for (const v of ps) {
          const d = v[0] * ax[0] + v[1] * ax[1];
          [lo, hi] = [Math.min(lo, d), Math.max(hi, d)];
        }
        return [lo, hi];
      };
      const [[a0, a1], [b0, b1]] = [span(a), span(b)];
      if (a1 - b0 <= slack || b1 - a0 <= slack) return false;
    }
  }
  return true;
}

/** Distance from a point to a polygon's outline. */
export function toOutline(poly: readonly Pt[], x: number, z: number): number {
  let best = Infinity;
  for (let i = 0; i < poly.length; i++) {
    const [[ax, az], [bx, bz]] = [poly[i], poly[(i + 1) % poly.length]];
    const [dx, dz] = [bx - ax, bz - az];
    const t = Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / (dx * dx + dz * dz || 1)));
    best = Math.min(best, Math.hypot(x - ax - dx * t, z - az - dz * t));
  }
  return best;
}

/** The tiles a polygon covers or comes within `grow` m of (each tile's middle within `grow` + half its diagonal), as `fn(x, z)` calls; true if one returns true. */
export function tilesOf(poly: readonly Pt[], grow: number, fn: (x: number, z: number) => boolean | void): boolean {
  const reach = grow + 0.71;
  const xs = poly.map((p) => p[0]);
  const zs = poly.map((p) => p[1]);
  if (poly.length === 4) {
    // (A rectangle: in its own frame, within `reach` of it each way, a little more at its corners.)
    const [a, b, , d] = poly;
    const [ul, vl] = [dist(a, b), dist(a, d)];
    if (ul > 1e-6 && vl > 1e-6) {
      const u: Pt = [(b[0] - a[0]) / ul, (b[1] - a[1]) / ul];
      const v: Pt = [(d[0] - a[0]) / vl, (d[1] - a[1]) / vl];
      if (Math.abs(u[0] * v[0] + u[1] * v[1]) < 1e-6) {
        for (let z = Math.floor(Math.min(...zs) - reach); z <= Math.ceil(Math.max(...zs) + reach); z++) {
          for (let x = Math.floor(Math.min(...xs) - reach); x <= Math.ceil(Math.max(...xs) + reach); x++) {
            const [px, pz] = [x + 0.5 - a[0], z + 0.5 - a[1]];
            const [pu, pv] = [px * u[0] + pz * u[1], px * v[0] + pz * v[1]];
            if (pu > -reach && pu < ul + reach && pv > -reach && pv < vl + reach && fn(x, z)) return true;
          }
        }
        return false;
      }
    }
  }
  for (let z = Math.floor(Math.min(...zs) - reach); z <= Math.ceil(Math.max(...zs) + reach); z++) {
    for (let x = Math.floor(Math.min(...xs) - reach); x <= Math.ceil(Math.max(...xs) + reach); x++) {
      const [cx, cz] = [x + 0.5, z + 0.5];
      if ((inside(poly as Pt[], cx, cz) || toOutline(poly, cx, cz) < reach) && fn(x, z)) return true;
    }
  }
  return false;
}

/**
 * Whether a footprint faces the road nearest its middle (not a trail): one of its sides runs
 * within `FACING` of the road's way there, or square to it (`FACING` from that too). tools/worlds/sample.ts asks the same.
 */
export function facesRoad(poly: readonly Pt[], roads: readonly RoadPlan[], index: RoadIndex, slack = 0): boolean {
  const c = centroidOf(poly);
  const near = [...index.around(c[0], c[1], 90)].filter(([r]) => roads[r].kind !== 'trail');
  if (!near.length) return false;
  const least = Math.min(...near.map(([, hit]) => hit.dist));
  // (`slack`: every road about as near, as well: the roads rounded in the file may have another nearest.)
  const lens = (lengthsOf.get(roads) ?? lengthsOf.set(roads, roads.map((r) => lengths(r.points))).get(roads))!;
  /** Whether a side runs along road `r` at `s` (the way it runs over 4 m there: one segment's way at a bend's corner may be off it). */
  const along = (r: number, s: number) => {
    const L = lens[r][lens[r].length - 1];
    const [a, b] = [pointAt(roads[r].points, Math.max(0, s - 2), lens[r]).p, pointAt(roads[r].points, Math.min(L, s + 2), lens[r]).p];
    const l = dist(a, b) || 1;
    const t: Pt = [(b[0] - a[0]) / l, (b[1] - a[1]) / l];
    return poly.some((p, k) => {
      const q = poly[(k + 1) % poly.length];
      const e = Math.hypot(q[0] - p[0], q[1] - p[1]) || 1;
      const cos = Math.abs(((q[0] - p[0]) * t[0] + (q[1] - p[1]) * t[1]) / e);
      return cos >= Math.cos(FACING) || cos <= Math.sin(FACING);
    });
  };
  return near
    .filter(([, hit]) => hit.dist <= least + slack)
    .every(([r, hit]) => {
      if (!along(r, hit.s)) return false;
      // (Inside a bend, as near to the road at more than one place along it: along it at each.)
      if (!slack) return true;
      const pts = roads[r].points;
      return pts.every((p, i) => dist(p, c) > least + slack || along(r, lens[r][i]));
    });
}
/** Each set of roads' arc lengths (`facesRoad`). */
const lengthsOf = new WeakMap<readonly RoadPlan[], number[][]>();

/** Calls `fn` with the index of every tile (of `W` × `D`) whose middle is inside `poly`. */
export function fillPoly(poly: readonly Pt[], W: number, D: number, fn: (i: number) => void): void {
  const zs = poly.map((p) => p[1]);
  // (A row at a time: where the outline crosses the row's middle, inside between each pair, as `inside` says.)
  for (let z = Math.max(0, Math.floor(Math.min(...zs))); z < Math.min(D, Math.ceil(Math.max(...zs))); z++) {
    const zc = z + 0.5;
    const cuts: number[] = [];
    for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
      const [[xi, zi], [xj, zj]] = [poly[i], poly[j]];
      if (zi > zc !== zj > zc) cuts.push(((xj - xi) * (zc - zi)) / (zj - zi) + xi);
    }
    cuts.sort((a, b) => a - b);
    for (let k = 0; k + 1 < cuts.length; k += 2) {
      for (let x = Math.max(0, Math.ceil(cuts[k] - 0.5)); x < Math.min(W, Math.ceil(cuts[k + 1] - 0.5)); x++) fn(z * W + x);
    }
  }
}

/** A rectangle's corners, counter-clockwise seen from above. */
export function rectPoly(r: Rect): Pt[] {
  return [
    [r.x, r.z],
    [r.x, r.z + r.d],
    [r.x + r.w, r.z + r.d],
    [r.x + r.w, r.z],
  ];
}

/** A polygon's centroid (the mean of its corners). */
export function centroidOf(poly: readonly Pt[]): Pt {
  return [poly.reduce((s, p) => s + p[0], 0) / poly.length, poly.reduce((s, p) => s + p[1], 0) / poly.length];
}
