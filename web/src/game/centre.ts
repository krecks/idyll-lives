/**
 * A main town's centre (`chooseCentre`): its high street, the stretch of its street nearest its
 * middle, and a square off it, kept clear of lots before they go down; its outline
 * (`WorldMeta.centre`), and how built over it is against the rest of the town (`centreDensity`).
 */

import { inside, leftOf, lengths, nearest, pointAt, type Pt, type RoadPlan } from './roads';
import type { Ground, Rect } from './town';
import { fillPoly, rectFrom, signedArea2, tilesOf } from './footprints';

/** How long a main town's high street is (m, at most), and how far its centre reaches either side of its middle line. */
const HIGH_STREET = 180;
const BAND = 32;
/** The bands a centre's outline is tried at, widest first (`outline`): a narrower one where the wider crosses itself, or isn't the densest. */
export const BANDS = [BAND, 24, 18, 12];
/** A main town's square (m along its street, m deep), the larger first. */
const SQUARES: readonly [number, number][] = [
  [26, 18],
  [20, 15],
  [16, 12],
];
/** A main town's centre is built over at least this many times as densely as the rest of its box (`centreDensity`). */
export const DENSEST = 1.15;
/** Off the land's edge (m), as the lots keep. */
export const EDGE = 8;

/** A main town's centre: its outline (`WorldMeta.centre`), its square, its high street (`road` from `s0` to `s1` m along it). */
export interface Centre {
  area: number;
  poly: Pt[];
  square: Pt[] | null;
  road: number;
  s0: number;
  s1: number;
  /** How far the outline reaches either side of the street (m), and the square along it (`outline`). */
  band: number;
  at?: { s: number; w: number; d: number; side: 1 | -1; front: number };
}

const add = (a: Pt, b: Pt, k = 1): Pt => [a[0] + b[0] * k, a[1] + b[1] * k];
const dist = (a: Pt, b: Pt) => Math.hypot(a[0] - b[0], a[1] - b[1]);
const r2 = (v: number) => Math.round(v * 100) / 100;

/** Whether a polyline crosses itself (two segments not next to each other meeting). */
export function selfCrossing(poly: readonly Pt[]): boolean {
  const n = poly.length;
  const cross = (o: Pt, a: Pt, b: Pt) => (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
  for (let i = 0; i < n; i++) {
    for (let j = i + 2; j < n; j++) {
      if (i === 0 && j === n - 1) continue;
      const [a, b, c, d] = [poly[i], poly[(i + 1) % n], poly[j], poly[(j + 1) % n]];
      if (cross(a, b, c) * cross(a, b, d) < 0 && cross(c, d, a) * cross(c, d, b) < 0) return true;
    }
  }
  return false;
}

/**
 * A main town's centre (area `area`): its high street, the stretch of its street (else a lane)
 * nearest its middle, `HIGH_STREET` m long at most, and a square off it on the side towards the
 * middle if it has room there (else the other), the largest of `SQUARES` that fits on land
 * nothing takes yet (no road, green or water). The square's tiles are taken on `ground`, so no
 * lot goes there. The outline reaches `BAND` m either side of the street, further round the
 * square. Null: no street long enough.
 */
export function chooseCentre(roads: readonly RoadPlan[], areaRoads: readonly number[], middle: Pt, ground: Ground, area = 0): Centre | null {
  // Stretches of the area's streets and lanes round its middle: the one nearest it, straight
  // enough for its outline not to cross itself, with the most land nothing takes yet (no green,
  // no water, no other road) to build on.
  let best: { road: number; s0: number; s1: number; side: 1 | -1; score: number } | null = null;
  for (const r of areaRoads) {
    const road = roads[r];
    if (road.kind === 'country' || road.kind === 'trail') continue;
    const ls = lengths(road.points);
    const L = ls[ls.length - 1];
    const half = Math.min(HIGH_STREET / 2, L / 2 - 6);
    if (half < 20) continue;
    const near = nearest(road.points, middle[0], middle[1], ls);
    for (const shift of [0, -50, 50, -100, 100]) {
      const mid = Math.max(6 + half, Math.min(L - 6 - half, near.s + shift));
      const { p } = pointAt(road.points, mid, ls);
      const probe: Centre = { area, poly: [], square: null, road: r, s0: mid - half, s1: mid + half, band: BAND };
      // (The widest band whose outline doesn't cross itself: round a tight bend, a narrower one.)
      const band = BANDS.find((b) => !selfCrossing(outline(roads, probe, b)));
      if (band === undefined) continue;
      const poly = outline(roads, probe, band);
      let [free, all] = [0, 0];
      fillPoly(poly, ground.w, ground.d, (i) => {
        all++;
        if (!ground.takenAt((i % ground.w) + 0.5, Math.floor(i / ground.w) + 0.5)) free++;
      });
      const score = dist(p, middle) + (road.kind === 'lane' ? 40 : 0) + (1 - free / Math.max(1, all)) * 200 + (BAND - band) * 3;
      if (!best || score < best.score) best = { road: r, s0: mid - half, s1: mid + half, side: near.side, score };
    }
  }
  if (!best) return null;
  const road = roads[best.road];
  const pts = road.points;
  const ls = lengths(pts);
  const { s0, s1 } = best;
  const mid = (s0 + s1) / 2;
  const hw = road.width / 2;
  const front = hw + 1.5;
  // The square: the largest that fits, nearest the middle, on the middle's side first.
  let square: { poly: Pt[]; side: 1 | -1; s: number; w: number; d: number } | null = null;
  for (const [w, d] of SQUARES) {
    for (const side of [best.side, -best.side as 1 | -1]) {
      for (let k = 0; k <= 12 && !square; k++) {
        const s = mid + (k % 2 ? 1 : -1) * Math.ceil(k / 2) * 6;
        if (s - w / 2 < s0 || s + w / 2 > s1) continue;
        const { p, t } = pointAt(pts, s, ls);
        const n = leftOf(t).map((v) => v * side) as Pt;
        const poly = rectFrom(add(p, n, front), t, n, w, d);
        let clear = true;
        tilesOf(poly, 0, (x, z) => {
          if (!inside(poly, x + 0.5, z + 0.5)) return false;
          clear = !ground.takenAt(x + 0.5, z + 0.5) && x >= EDGE && z >= EDGE && x < ground.w - EDGE && z < ground.d - EDGE;
          return !clear;
        });
        if (clear) square = { poly, side, s, w, d };
      }
      if (square) break;
    }
    if (square) break;
  }
  if (square) {
    const sq = square.poly;
    tilesOf(sq, 0, (x, z) => {
      if (inside(sq, x + 0.5, z + 0.5)) ground.take({ x, z, w: 1, d: 1 });
    });
  }
  const centre: Centre = { area, poly: [], square: square?.poly.map(([x, z]): Pt => [r2(x), r2(z)]) ?? null, road: best.road, s0, s1, band: BAND, ...(square ? { at: { s: square.s, w: square.w, d: square.d, side: square.side, front } } : {}) };
  for (const band of BANDS) {
    centre.poly = outline(roads, centre, band);
    centre.band = band;
    if (!selfCrossing(centre.poly)) break;
  }
  return centre;
}

/** A centre's outline `band` m either side of its high street, out round its square: counter-clockwise seen from above. */
export function outline(roads: readonly RoadPlan[], c: Centre, band: number): Pt[] {
  const pts = roads[c.road].points;
  const ls = lengths(pts);
  const { s0, s1, at } = c;
  const left: Pt[] = [];
  const right: Pt[] = [];
  const steps = Math.ceil((s1 - s0) / 4);
  for (let k = 0; k <= steps; k++) {
    const s = s0 + ((s1 - s0) * k) / steps;
    const { p, t } = pointAt(pts, s, ls);
    const n = leftOf(t);
    const round = at && Math.abs(s - at.s) <= at.w / 2 + 2 ? Math.max(band, at.front + at.d + 4) : band;
    left.push(add(p, n, at?.side === 1 ? round : band));
    right.push(add(p, n, -(at?.side === -1 ? round : band)));
  }
  const poly = [...left, ...right.reverse()].map(([x, z]): Pt => [r2(x), r2(z)]);
  return signedArea2(poly) > 0 ? poly.reverse() : poly;
}

/**
 * How built over a main town's centre is against the rest of its box: of each one's open land
 * (`taken`: the tiles that are a road, a lot, a green or water, one per tile over `W` × `D`), the
 * share under a backdrop building (`polys`) whose tiles' middles are in it. `per`: each
 * building's tiles counted towards the rest of the box (to take it away again: `layBackdrop`).
 * tools/worlds/sample.ts checks the centre is the denser.
 */
export function centreDensity(W: number, D: number, taken: Uint8Array, box: Rect, centre: readonly Pt[], polys: readonly (readonly Pt[])[]): { centre: number; rest: number; open: [number, number]; built: [number, number]; per: number[] } {
  const inC = new Uint8Array(W * D);
  fillPoly(centre, W, D, (i) => (inC[i] = 1));
  const open: [number, number] = [0, 0];
  for (let z = Math.max(0, box.z); z < Math.min(D, box.z + box.d); z++) for (let x = Math.max(0, box.x); x < Math.min(W, box.x + box.w); x++) if (!taken[z * W + x]) open[inC[z * W + x] ? 0 : 1]++;
  const built: [number, number] = [0, 0];
  const seen = new Uint8Array(W * D);
  const per = polys.map((poly) => {
    let rest = 0;
    fillPoly(poly, W, D, (i) => {
      const [x, z] = [i % W, Math.floor(i / W)];
      if (seen[i] || taken[i] || x < box.x || z < box.z || x >= box.x + box.w || z >= box.z + box.d) return;
      seen[i] = 1;
      built[inC[i] ? 0 : 1]++;
      if (!inC[i]) rest++;
    });
    return rest;
  });
  return { centre: built[0] / Math.max(1, open[0]), rest: built[1] / Math.max(1, open[1]), open, built, per };
}
