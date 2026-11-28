/**
 * A main town's centre (`chooseCentre`): its high street, the stretch of its street nearest its
 * middle, and a square off it, kept clear of lots before they go down; its outline
 * (`WorldMeta.centre`), and how built over it is against the rest of the town (`centreDensity`).
 */

import { dist, inside, leftOf, lengths, nearest, pointAt, roadMask, type Pt, type RoadPlan } from './roads';
import type { Ground, Rect } from './town';
import { add, ccw, fillPoly, r2, rectFrom, rectPoly, tilesOf } from './footprints';

/** How long a main town's high street is (m, at most), and how far its centre reaches either side of its middle line. */
const HIGH_STREET = 180;
const BAND = 32;
/** The bands a centre's outline is tried at, widest first (`outline`): a narrower one where the wider crosses itself, or isn't the densest. */
export const BANDS = [BAND, 24, 18, 12];
/** What a city's avenue counts against being its high street (m, as nearer the middle): its through road, the shops on a street off it. */
const AVENUE_HIGH_STREET = 400;
/** A centre's side narrowed to its kerb (`narrowCentre`): the road's half-width and its pavement, about (m). */
const KERB_BAND = 6;
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
 * A main town's centre (area `area`): its high street, the stretch of its street (else a lane; a
 * city's avenue last of all, its through road) nearest its middle, `HIGH_STREET` m long at most, and a square off it on the side towards the
 * middle if it has room there (else the other), the largest of `SQUARES` that fits on land
 * nothing takes yet (no road, green or water). Of the stretches, the one with the most land free to
 * build on round it counts for more (nothing on it, and not `steep`: a hillside its lots would be
 * given up on). The square's tiles are taken on `ground`, so no
 * lot goes there. The outline reaches `BAND` m either side of the street, further round the
 * square. Null: no street long enough.
 */
export function chooseCentre(roads: readonly RoadPlan[], areaRoads: readonly number[], middle: Pt, ground: Ground, area = 0, steep: (x: number, z: number) => boolean = () => false): Centre | null {
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
        const [x, z] = [(i % ground.w) + 0.5, Math.floor(i / ground.w) + 0.5];
        if (!ground.takenAt(x, z) && !steep(x, z)) free++;
      });
      const score = dist(p, middle) + (road.kind === 'lane' ? 40 : road.kind === 'avenue' ? AVENUE_HIGH_STREET : 0) + (1 - free / Math.max(1, all)) * 200;
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

/**
 * Centre `c` made the densest: its outline narrowed to its high street, either side on its own (the narrower
 * `BANDS`, down to `KERB_BAND` just past the pavement, where the land on that side is the emptier: a shore, a
 * hillside) until its open land is built over `DENSEST` times as densely as the rest of its area's box `box`
 * (`centreDensity` over `taken`, `polys`), the widest outline that is; else the one that comes nearest. Its square
 * stays inside it (`outline` goes round it).
 */
export function narrowCentre(roads: readonly RoadPlan[], c: Centre, W: number, D: number, taken: Uint8Array, box: Rect, polys: readonly (readonly Pt[])[]): void {
  const ratio = densityRatio(W, D, taken, box, polys);
  let best = { poly: c.poly, r: ratio(c.poly), width: 2 * c.band };
  if (best.r >= DENSEST) return;
  const bands = [...BANDS.filter((b) => b <= c.band), KERB_BAND];
  for (const left of bands) {
    for (const right of bands) {
      if (left === c.band && right === c.band) continue;
      const poly = outline(roads, c, left, right);
      if (selfCrossing(poly)) continue;
      const r = ratio(poly);
      const better = r >= DENSEST ? best.r < DENSEST || left + right > best.width : r > best.r;
      if (better) best = { poly, r, width: left + right };
    }
  }
  c.poly = best.poly;
}

/**
 * `centreDensity`'s centre against the rest of the box, for one outline after another over the same land: the open
 * land of the box and the buildings' tiles worked out once (each tile counted for the first building over it, as
 * `centreDensity` counts them), one buffer marking each outline's tiles.
 */
function densityRatio(W: number, D: number, taken: Uint8Array, box: Rect, polys: readonly (readonly Pt[])[]): (centre: readonly Pt[]) => number {
  const inBox = (x: number, z: number) => x >= box.x && z >= box.z && x < box.x + box.w && z < box.z + box.d;
  let open = 0;
  for (let z = Math.max(0, box.z); z < Math.min(D, box.z + box.d); z++) for (let x = Math.max(0, box.x); x < Math.min(W, box.x + box.w); x++) if (!taken[z * W + x]) open++;
  const built: number[] = [];
  const seen = new Uint8Array(W * D);
  for (const poly of polys) {
    fillPoly(poly, W, D, (i) => {
      if (seen[i] || taken[i] || !inBox(i % W, Math.floor(i / W))) return;
      seen[i] = 1;
      built.push(i);
    });
  }
  const mark = new Int32Array(W * D);
  let stamp = 0;
  return (centre) => {
    stamp++;
    let openIn = 0;
    fillPoly(centre, W, D, (i) => {
      mark[i] = stamp;
      if (!taken[i] && inBox(i % W, Math.floor(i / W))) openIn++;
    });
    let builtIn = 0;
    for (const i of built) if (mark[i] === stamp) builtIn++;
    const inside = builtIn / Math.max(1, openIn);
    const rest = (built.length - builtIn) / Math.max(1, open - openIn);
    return inside / Math.max(1e-6, rest);
  };
}

/**
 * Centre `c` of a town of `W` × `D` made the densest (`narrowCentre`) over its land as laid: its open land is no road
 * of `roads` (but a trail), water (`water`, one per tile, if any), green (`greens`), its square or lot but a building's (`lots`:
 * each lot's rectangle, and whether it is a building's, built over: a named place's or a building of flats'; a home's
 * lot is mostly its garden), against the rest of its area's box `box`.
 */
export function densestCentre(c: Centre, roads: readonly RoadPlan[], W: number, D: number, water: Uint8Array | null, greens: readonly (readonly Pt[])[], lots: readonly { rect: Rect; built: boolean }[], box: Rect): void {
  const taken = roadMask(roads.filter((r) => r.kind !== 'trail'), W, D, 0);
  if (water) for (let i = 0; i < taken.length; i++) if (water[i]) taken[i] = 1;
  for (const g of greens) fillPoly(g, W, D, (i) => (taken[i] = 1));
  // (Its square kept open, as a green is: never built on.)
  if (c.square) fillPoly(c.square, W, D, (i) => (taken[i] = 1));
  const built: Pt[][] = [];
  for (const { rect: r, built: building } of lots) {
    if (building) built.push(rectPoly(r));
    else for (let z = Math.max(0, r.z); z < Math.min(D, r.z + r.d); z++) for (let x = Math.max(0, r.x); x < Math.min(W, r.x + r.w); x++) taken[z * W + x] = 1;
  }
  narrowCentre(roads, c, W, D, taken, box, built);
}

/** A centre's outline `band` m either side of its high street (`right` m on its right, if given), out round its square: counter-clockwise seen from above. */
export function outline(roads: readonly RoadPlan[], c: Centre, band: number, right = band): Pt[] {
  const pts = roads[c.road].points;
  const ls = lengths(pts);
  const { s0, s1, at } = c;
  const lefts: Pt[] = [];
  const rights: Pt[] = [];
  const steps = Math.ceil((s1 - s0) / 4);
  for (let k = 0; k <= steps; k++) {
    const s = s0 + ((s1 - s0) * k) / steps;
    const { p, t } = pointAt(pts, s, ls);
    const n = leftOf(t);
    // (Out round the square, on its side.)
    const near = !!at && Math.abs(s - at.s) <= at.w / 2 + 2;
    const round = at ? at.front + at.d + 4 : 0;
    lefts.push(add(p, n, near && at?.side === 1 ? Math.max(band, round) : band));
    rights.push(add(p, n, -(near && at?.side === -1 ? Math.max(right, round) : right)));
  }
  const poly = [...lefts, ...rights.reverse()].map(([x, z]): Pt => [r2(x), r2(z)]);
  return ccw(poly);
}

/**
 * How built over a main town's centre is against the rest of its box: of each one's open land
 * (`taken`: the tiles that are a road, a lot, a green or water, one per tile over `W` × `D`), the
 * share under a building (`polys`: the named places' and the buildings of flats' lots) whose tiles' middles are in it. `per`: each
 * building's tiles counted towards the rest of the box. tools/worlds/sample.ts checks the centre is the denser.
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
