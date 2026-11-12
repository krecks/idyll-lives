/**
 * The living town's world data (docs/design/living-town.md, "World data"), laid out with a big
 * world (`macro.ts`):
 *
 * - each main town's centre (`chooseCentre`): its high street, the stretch of its street nearest
 *   its middle, and a square off it, kept clear of lots before they go down;
 * - the backdrop (`layBackdrop`): buildings nobody lives in that make the town dense, laid after
 *   the lots, the parking, the trails and the orchards: the main town's civic places first (a
 *   town hall, a church, a school, a police and a fire station, a medical centre, offices,
 *   workshops, a pub, shops: every career a resident may hold has a workplace there), then rows
 *   along every street (shops, cafés and flats of two to four storeys shoulder to shoulder in the
 *   centre; houses, terraces and flats further out), rows behind them, the blocks filled in, and
 *   barns and a farm building by every farmstead;
 * - the places the town tags name (`TownPlace`): every public lot (the sim's own) and every
 *   backdrop place, with its opening hours (the kind's, widened to the shifts of the careers that
 *   work there) and those careers' categories (`work`).
 *
 * The rules a backdrop building keeps (tools/worlds/sample.ts checks them): it never overlaps a
 * lot (and keeps `LOT_CLEAR` m off its boundary), a road with its sidewalks or a junction (it keeps
 * more than half a road's width off its middle line), the water, a trail, the parking, a square or
 * another backdrop building (two may touch: a row of shops). Its footprint is a rectangle turned to
 * the road nearest it (`FACING`: its long side or its gable end along the road). Every main town
 * has `MAIN_LEAST` or more, every hamlet `HAMLET_LEAST`; the centre is the densest.
 *
 * Footprints go counter-clockwise seen from above (+y up, x right: z runs down the screen, so the
 * signed area x·z' − x'·z is negative), in town metres. `y` is 0, the town's flat land: they stand
 * within reach of the roads and lots, where the land is levelled (the renderer may count them in
 * its built field as it does lots). `ridge` is the angle a turn round +y takes +x to the ridge's
 * way (right-handed, y up: the ridge runs along (cos r, −sin r) in x, z), in [0, π).
 */

import type { CareerCategory, Content } from '../content/content';
import type { BackdropBuilding, BackdropUse, TownPlace } from '../core/protocol';
import { inside, leftOf, lengths, nearest, pointAt, RoadIndex, type Pt, type RoadPlan } from './roads';
import type { Ground, PlotSlot, Rect } from './town';

type Rand = () => number;

/** How long a main town's high street is (m, at most), and how far its centre reaches either side of its middle line. */
const HIGH_STREET = 180;
const BAND = 32;
/** The bands a centre's outline is tried at, widest first (`outline`): a narrower one where the wider crosses itself, or isn't the densest. */
const BANDS = [BAND, 24, 18, 12];
/** A main town's square (m along its street, m deep), the larger first. */
const SQUARES: readonly [number, number][] = [
  [26, 18],
  [20, 15],
  [16, 12],
];
/** A backdrop building keeps this far off a lot's boundary (m). */
export const LOT_CLEAR = 1;
/** At least this many backdrop buildings in a main town, in a hamlet. */
export const MAIN_LEAST = 60;
export const HAMLET_LEAST = 8;
/** A main town's centre is built over at least this many times as densely as the rest of its box (`centreDensity`). */
const DENSEST = 1.15;
/** A backdrop building's side runs within this (radians) of the way the road nearest it runs, or square to it. */
export const FACING = (15 * Math.PI) / 180;
/** How far a building in a block's middle may stand from the nearest road's edge (m): the main town's, a hamlet's. */
const INFILL_REACH = [34, 24];
/** Off the land's edge (m), as the lots keep. */
const EDGE = 8;

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
const range = (rand: Rand, lo: number, hi: number) => lo + (hi - lo) * rand();
const between = (rand: Rand, lo: number, hi: number) => lo + Math.floor(rand() * (hi - lo + 1));
const pick = <T>(rand: Rand, list: readonly T[]): T => list[Math.floor(rand() * list.length)];

/** Twice the signed area of a polygon (x·z' − x'·z summed): negative when it goes counter-clockwise seen from above. */
export function signedArea2(poly: readonly Pt[]): number {
  let n = 0;
  for (let i = 0; i < poly.length; i++) {
    const [a, b] = [poly[i], poly[(i + 1) % poly.length]];
    n += a[0] * b[1] - b[0] * a[1];
  }
  return n;
}

/** A rectangle `w` along `t` and `d` along `n` (unit, square to each other) from the middle of its front edge `c`, counter-clockwise seen from above. */
function rectFrom(c: Pt, t: Pt, n: Pt, w: number, d: number): Pt[] {
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
function tilesOf(poly: readonly Pt[], grow: number, fn: (x: number, z: number) => boolean | void): boolean {
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

/** Whether a polyline crosses itself (two segments not next to each other meeting). */
function selfCrossing(poly: readonly Pt[]): boolean {
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
function outline(roads: readonly RoadPlan[], c: Centre, band: number): Pt[] {
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
 * Keeps a centre's land for the backdrop: every tile inside its outline nothing takes yet is taken
 * on `ground` (no house goes there: its public places went down first), and listed (one per tile,
 * row-major) for `layBackdrop` to build on.
 */
export function reserveCentre(centre: Centre, ground: Ground): Uint8Array {
  const out = new Uint8Array(ground.w * ground.d);
  const xs = centre.poly.map((p) => p[0]);
  const zs = centre.poly.map((p) => p[1]);
  for (let z = Math.max(0, Math.floor(Math.min(...zs))); z < Math.min(ground.d, Math.ceil(Math.max(...zs))); z++) {
    for (let x = Math.max(0, Math.floor(Math.min(...xs))); x < Math.min(ground.w, Math.ceil(Math.max(...xs))); x++) {
      if (!inside(centre.poly, x + 0.5, z + 0.5) || ground.takenAt(x + 0.5, z + 0.5)) continue;
      out[z * ground.w + x] = 1;
      ground.take({ x, z, w: 1, d: 1 });
    }
  }
  return out;
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

/** What a backdrop building is, and how big a kind of it is: along the road (m), deep (m), storeys, roofs. */
interface Kind {
  use: BackdropUse;
  w: [number, number];
  d: [number, number];
  storeys: [number, number];
  roofs: readonly BackdropBuilding['roof'][];
}
const K = (use: BackdropUse, w: [number, number], d: [number, number], storeys: [number, number], roofs: readonly BackdropBuilding['roof'][]): Kind => ({ use, w, d, storeys, roofs });
/** The kinds by where they stand: the centre (2 to 4 storeys, shoulder to shoulder), the rest of a main town, a hamlet. */
const KINDS = {
  centre: [
    [K('shop', [6, 10], [10, 14], [2, 3], ['gable', 'flat', 'mansard']), 34],
    [K('flats', [12, 20], [10, 14], [3, 4], ['flat', 'mansard', 'hip']), 20],
    [K('cafe', [6, 9], [9, 12], [2, 3], ['gable', 'mansard']), 10],
    [K('restaurant', [9, 13], [10, 14], [2, 3], ['gable', 'hip']), 7],
    [K('office', [14, 22], [11, 15], [3, 4], ['flat', 'mansard']), 12],
    [K('terrace', [16, 30], [9, 11], [2, 3], ['gable', 'mansard']), 12],
    [K('hall', [14, 18], [10, 14], [2, 2], ['gable', 'hip']), 3],
  ],
  town: [
    [K('house', [8, 11], [8, 10], [1, 2], ['gable', 'hip']), 30],
    [K('terrace', [18, 32], [9, 11], [2, 2], ['gable']), 24],
    [K('flats', [12, 20], [10, 14], [2, 4], ['flat', 'hip', 'mansard']), 16],
    [K('shop', [7, 10], [9, 12], [1, 2], ['gable', 'flat']), 7],
    [K('workshop', [12, 20], [10, 16], [1, 2], ['gable', 'flat']), 8],
    [K('office', [14, 20], [10, 14], [2, 3], ['flat', 'hip']), 5],
    [K('cafe', [6, 9], [9, 11], [1, 2], ['gable']), 3],
    [K('restaurant', [9, 12], [10, 13], [1, 2], ['gable', 'hip']), 2],
    [K('hall', [12, 16], [10, 12], [1, 2], ['gable']), 2],
  ],
  hamlet: [
    [K('house', [8, 11], [8, 10], [1, 2], ['gable', 'hip']), 48],
    [K('terrace', [16, 24], [9, 10], [2, 2], ['gable']), 12],
    [K('workshop', [10, 16], [10, 14], [1, 2], ['gable']), 10],
    [K('barn', [12, 18], [9, 12], [2, 2], ['gable']), 10],
    [K('shop', [7, 9], [9, 11], [1, 2], ['gable']), 6],
    [K('cafe', [6, 8], [9, 11], [1, 2], ['gable']), 4],
    [K('hall', [12, 15], [9, 12], [1, 2], ['gable']), 3],
  ],
} as const satisfies Record<string, readonly (readonly [Kind, number])[]>;
type Zone = keyof typeof KINDS;

/** A main town's civic places, laid first, nearest its square (its high street's middle): what, its kind, how big, its name, the careers working there. */
interface Civic {
  kind: Kind;
  place: TownPlace['kind'];
  name: (n: Namer) => string;
  work: string[];
  /** Its gable end to the road (a church): deeper than wide, the ridge running back from the road. */
  endOn?: boolean;
}
const CIVIC: readonly Civic[] = [
  { kind: K('office', [16, 20], [12, 14], [3, 3], ['hip', 'mansard']), place: 'office', name: (n) => `${n.town} Town Hall`, work: ['politics'] },
  { kind: K('church', [11, 13], [22, 28], [3, 3], ['gable']), place: 'church', name: (n) => `St ${pick(n.rand, SAINTS)}'s Church`, work: [], endOn: true },
  { kind: K('school', [28, 36], [14, 18], [2, 2], ['hip', 'flat']), place: 'school', name: (n) => `${n.town} School`, work: ['education'] },
  { kind: K('office', [12, 16], [11, 13], [2, 3], ['flat', 'hip']), place: 'office', name: (n) => `${n.town} Police Station`, work: ['detective'] },
  { kind: K('workshop', [16, 20], [14, 16], [2, 2], ['flat']), place: 'workshop', name: (n) => `${n.town} Fire Station`, work: ['firefighter'] },
  { kind: K('office', [14, 18], [11, 13], [2, 2], ['flat', 'hip']), place: 'office', name: (n) => `${n.street()} Medical Centre`, work: ['medicine'] },
  { kind: K('workshop', [22, 28], [16, 20], [2, 2], ['flat']), place: 'workshop', name: (n) => `${n.town} Leisure Centre`, work: ['sports'] },
  { kind: K('office', [12, 16], [11, 13], [3, 4], ['flat', 'mansard']), place: 'office', name: (n) => `${n.last()} Bank`, work: ['finance', 'business'] },
  { kind: K('office', [12, 16], [11, 13], [3, 4], ['flat']), place: 'office', name: (n) => `The ${n.town} Gazette`, work: ['journalism', 'tv', 'influencer'] },
  { kind: K('office', [16, 22], [12, 15], [3, 5], ['flat']), place: 'office', name: (n) => `${n.street()} Works`, work: ['it', 'science'] },
  { kind: K('office', [9, 12], [10, 12], [3, 3], ['mansard', 'gable']), place: 'office', name: (n) => `${n.last()} & ${n.last()} Solicitors`, work: ['law'] },
  { kind: K('office', [12, 16], [11, 13], [2, 2], ['flat']), place: 'office', name: () => 'Army Reserve Centre', work: ['military'] },
  { kind: K('workshop', [14, 18], [12, 14], [1, 2], ['flat', 'gable']), place: 'workshop', name: (n) => `${n.last()}'s Garage`, work: ['mechanics', 'transport'] },
  { kind: K('workshop', [14, 18], [12, 14], [1, 2], ['gable']), place: 'workshop', name: (n) => `${n.last()} Builders`, work: ['trades'] },
  { kind: K('workshop', [10, 14], [10, 12], [2, 2], ['gable']), place: 'workshop', name: (n) => `${n.street()} Studios`, work: ['arts'] },
  { kind: K('restaurant', [12, 16], [12, 14], [2, 3], ['gable', 'hip']), place: 'restaurant', name: (n) => n.inn(), work: ['culinary', 'hospitality', 'music'] },
  { kind: K('restaurant', [16, 22], [12, 14], [3, 4], ['mansard', 'hip']), place: 'restaurant', name: (n) => `The ${n.town} Hotel`, work: ['hospitality'] },
  { kind: K('shop', [8, 10], [10, 12], [2, 3], ['gable', 'mansard']), place: 'shop', name: (n) => `${n.last()}'s Boutique`, work: ['fashion', 'retail'] },
  { kind: K('shop', [14, 20], [12, 16], [1, 1], ['flat', 'gable']), place: 'shop', name: (n) => `${n.town} Garden Centre`, work: ['nature', 'retail'] },
];
const SAINTS = ['Mary', 'John', 'Peter', 'Michael', 'Andrew', 'James', 'Anne', 'Catherine', 'Luke', 'Margaret', 'Martin', 'Nicholas'];
const SHOPS = ['Bakery', 'Butcher', 'Greengrocer', 'Bookshop', 'Hardware', 'Chemist', 'Florist', 'Newsagent', 'Toy Shop', 'Deli', 'Tailor', 'Cobbler', 'Fishmonger', 'Gift Shop', 'Bike Shop', 'Record Shop', 'Sweet Shop', 'Wool Shop', 'Antiques', 'Opticians'];
const CAFES = ['Tea Rooms', 'Coffee House', 'Café', 'Kitchen', 'Espresso Bar', 'Tea Shop'];
const INN_A = ['Red', 'Golden', 'Black', 'White', 'Green', 'Old', 'Royal', 'Blue', 'Silver', 'Merry'];
const INN_B = ['Lion', 'Hart', 'Swan', 'Bell', 'Crown', 'Anchor', 'Plough', 'Fox', 'Stag', 'Oak', 'Boar', 'Hound'];
const OFFICES = ['House', 'Chambers', 'Court', 'Buildings'];
const WORKSHOPS = ['Joinery', 'Pottery', 'Forge', 'Printworks', 'Upholstery', 'Glassworks', 'Bike Repairs', 'Smithy'];
const FARM_BUILDINGS = ['Dairy', 'Stables', 'Farm Shop', 'Creamery', 'Granary'];

/** What a kind of place is open (minutes of the day, [open, close)) and on which days (bit 0 Monday) before its workers' shifts widen it. */
const HOURS: Record<Exclude<TownPlace['kind'], 'park' | 'outdoor'>, NonNullable<TownPlace['hours']>> = {
  shop: { open: 9 * 60, close: 18 * 60, days: 0b0111111 },
  cafe: { open: 7 * 60 + 30, close: 18 * 60, days: 0b1111111 },
  restaurant: { open: 12 * 60, close: 23 * 60, days: 0b1111111 },
  office: { open: 8 * 60, close: 18 * 60, days: 0b0011111 },
  school: { open: 8 * 60, close: 16 * 60, days: 0b0011111 },
  church: { open: 8 * 60, close: 20 * 60, days: 0b1111111 },
  workshop: { open: 7 * 60 + 30, close: 17 * 60, days: 0b0111111 },
  farm: { open: 6 * 60, close: 20 * 60, days: 0b1111111 },
};
/** The sim's own shops (public lots): open as long as an outing may go there (content `visits.outingHomeBy`, 23:00). */
const LOT_HOURS: NonNullable<TownPlace['hours']> = { open: 7 * 60, close: 23 * 60, days: 0b1111111 };
/** The careers working at the sim's public places, by template. */
const LOT_WORK: Record<string, string[]> = { cafe: ['culinary', 'hospitality'], grocer: ['retail'], bakery: ['culinary', 'retail'], market: ['retail', 'nature'] };
/** The careers working at a backdrop place of a kind besides the civic ones. */
const KIND_WORK: Partial<Record<TownPlace['kind'], string[]>> = { shop: ['retail'], cafe: ['culinary', 'hospitality'], restaurant: ['culinary', 'hospitality'], office: ['business'], workshop: ['trades'], farm: ['nature'] };
const PLACE_KINDS: readonly TownPlace['kind'][] = ['shop', 'cafe', 'restaurant', 'office', 'school', 'church', 'workshop', 'farm'];

/** Opening hours for a place of `kind` where `work`'s careers work: the kind's, widened to every shift of theirs (a career's own and its tracks'). */
export function hoursFor(base: NonNullable<TownPlace['hours']>, work: readonly string[], categories: readonly CareerCategory[]): NonNullable<TownPlace['hours']> {
  let { open, close, days } = base;
  for (const id of work) {
    const c = categories.find((k) => k.id === id);
    for (const shift of [c?.shift, ...(c?.tracks ?? []).map((t) => t.shift)]) {
      if (!shift) continue;
      open = Math.min(open, Math.round(shift.start * 60));
      close = Math.max(close, Math.min(24 * 60, Math.round((shift.start + shift.hours) * 60)));
      for (const d of shift.days) days |= 1 << d;
    }
  }
  return { open, close, days };
}

/** Names for places, none twice. */
interface Namer {
  rand: Rand;
  town: string;
  last(): string;
  street(): string;
  inn(): string;
  /** A name `draw` gives that no place has yet. */
  unique(draw: () => string): string;
}

export interface BackdropInput {
  content: Content;
  W: number;
  D: number;
  roads: readonly RoadPlan[];
  /** The areas (0: the main town): name, box, middle, roads (indices into `roads`). */
  areas: readonly { name: string; rect: Rect; centre: Pt; roads: readonly number[] }[];
  slots: readonly PlotSlot[];
  rectOf: (s: PlotSlot) => Rect;
  ground: Ground;
  centres: readonly Centre[];
  /** The centres' land kept for the backdrop (`reserveCentre`), if any. */
  reserved?: Uint8Array;
  /** The greens and the water (one per tile), for how built over the centre is (`centreDensity`). */
  greens: readonly Pt[][];
  water: Uint8Array;
  /** How much of a main town's land is built over, 0..1 (a dense town's most). */
  density: number;
  rand: Rand;
}

/** The backdrop and the places of a big world: see the top of this file. */
export function layBackdrop(input: BackdropInput): { backdrop: BackdropBuilding[]; places: TownPlace[] } {
  const { content, W, D, roads, areas, slots, rectOf, ground, centres, reserved, greens, water, density, rand } = input;
  const index = new RoadIndex(roads);
  const ways = roads.map((r) => r.kind !== 'trail');
  // What a building may not stand on (a tile each): what the ground takes (roads, greens, water,
  // lots and the gaps after them, the parking, the trails, the orchards, the squares), every lot
  // a metre round, and the land between a lot's front and its road (its driveway crosses it).
  const blocked = ground.mask();
  if (reserved) for (let i = 0; i < blocked.length; i++) if (reserved[i]) blocked[i] = 0;
  const roadTiles = ground.roadTiles();
  const mark = (r: Rect) => {
    for (let z = Math.max(0, r.z); z < Math.min(D, r.z + r.d); z++) for (let x = Math.max(0, r.x); x < Math.min(W, r.x + r.w); x++) blocked[z * W + x] = 1;
  };
  for (const s of slots) {
    const r = rectOf(s);
    mark({ x: r.x - LOT_CLEAR, z: r.z - LOT_CLEAR, w: r.w + 2 * LOT_CLEAR, d: r.d + 2 * LOT_CLEAR });
    // Out from its front to the road (a wedge on a bend), 14 m at most.
    const [dx, dz] = [[0, -1], [-1, 0], [0, 1], [1, 0]][s.facing];
    const along = s.facing % 2 ? r.d : r.w;
    for (let k = 0; k < along; k++) {
      let [x, z] = s.facing === 0 ? [r.x + k, r.z - 1] : s.facing === 1 ? [r.x - 1, r.z + k] : s.facing === 2 ? [r.x + k, r.z + r.d] : [r.x + r.w, r.z + k];
      for (let step = 0; step < 14 && x >= 0 && z >= 0 && x < W && z < D && !roadTiles[z * W + x]; step++, x += dx, z += dz) blocked[z * W + x] = 1;
    }
  }
  const built: BackdropBuilding[] = [];
  const buckets = new Map<number, number[]>();
  const CELL = 24;
  const key = (cx: number, cz: number) => cx * 4096 + cz;
  /** Files building `at` under the cells its footprint's box covers. */
  const bucket = (poly: readonly Pt[], at: number) => {
    const xs = poly.map((p) => p[0]);
    const zs = poly.map((p) => p[1]);
    for (let cz = Math.floor(Math.min(...zs) / CELL); cz <= Math.floor(Math.max(...zs) / CELL); cz++) {
      for (let cx = Math.floor(Math.min(...xs) / CELL); cx <= Math.floor(Math.max(...xs) / CELL); cx++) {
        const list = buckets.get(key(cx, cz));
        if (list) list.push(at);
        else buckets.set(key(cx, cz), [at]);
      }
    }
  };
  /** Whether a footprint may go down (see the top of this file). */
  const fits = (poly: Pt[]): boolean => {
    const bad = tilesOf(poly, 0, (x, z) => x < EDGE || z < EDGE || x >= W - EDGE || z >= D - EDGE || blocked[z * W + x] === 1);
    if (bad) return false;
    // (Off every road and trail by more than half its width, measured from its middle.)
    for (let i = 0; i < poly.length; i++) {
      const [a, b] = [poly[i], poly[(i + 1) % poly.length]];
      const n = Math.max(1, Math.ceil(dist(a, b)));
      for (let k = 0; k <= n; k++) {
        const p = add(a, [b[0] - a[0], b[1] - a[1]], k / n);
        if (index.onRoad(p[0], p[1], 0.5)) return false;
      }
    }
    const xs = poly.map((p) => p[0]);
    const zs = poly.map((p) => p[1]);
    for (let cz = Math.floor(Math.min(...zs) / CELL); cz <= Math.floor(Math.max(...zs) / CELL); cz++) {
      for (let cx = Math.floor(Math.min(...xs) / CELL); cx <= Math.floor(Math.max(...xs) / CELL); cx++) {
        // (Less than `overlaps`' slack: rounded to the centimetre in the file, it still holds.)
        for (const j of buckets.get(key(cx, cz)) ?? []) if (overlaps(poly, built[j].poly, 0.02)) return false;
      }
    }
    return true;
  };
  /** The road (not a trail) nearest a point, within `within` m: as `RoadIndex.nearest`. */
  const nearestWay = (x: number, z: number, within: number) => {
    let best: { road: number; dist: number; s: number; side: 1 | -1 } | null = null;
    for (const [road, hit] of index.around(x, z, within)) {
      if (!ways[road] || (best && best.dist <= hit.dist)) continue;
      // (+1: the road's left, as `RoadIndex.nearest` says.)
      const { p } = pointAt(roads[road].points, hit.s);
      const side: 1 | -1 = (x - p[0]) * hit.t[1] - (z - p[1]) * hit.t[0] >= 0 ? 1 : -1;
      best = { road, dist: hit.dist, s: hit.s, side };
    }
    return best;
  };
  const areaOf = (p: Pt) => {
    const at = areas.findIndex((a) => p[0] >= a.rect.x && p[1] >= a.rect.z && p[0] <= a.rect.x + a.rect.w && p[1] <= a.rect.z + a.rect.d);
    return at >= 0 ? at : areas.reduce((k, a, i) => (dist(a.centre, p) < dist(areas[k].centre, p) ? i : k), 0);
  };
  const centreAt = (p: Pt) => centres.some((c) => inside(c.poly, p[0], p[1]));
  /** Where each building's door is: the middle of its front, on the road's side. */
  const doors: Pt[] = [];
  /**
   * Lays a building of `kind` down, `w` along `t` and `d` back along `n` (away from the road) from
   * the middle of its front `c`, if it fits: its index, else -1.
   */
  const put = (c: Pt, t: Pt, n: Pt, w: number, d: number, kind: Kind, endOn = false): number => {
    const poly = rectFrom(c, t, n, w, d);
    if (!fits(poly) || !facesRoad(poly, roads, index, 0.3)) return -1;
    const roof = pick(rand, kind.roofs);
    // (The ridge along the longer side; a church's back from the road.)
    const way = endOn || d > w ? n : t;
    let ridge = Math.atan2(-way[1], way[0]);
    if (ridge < 0) ridge += Math.PI;
    if (ridge >= Math.PI) ridge -= Math.PI;
    const b: BackdropBuilding = {
      poly: poly.map(([x, z]): [number, number] => [r2(x), r2(z)]),
      y: 0,
      storeys: between(rand, ...kind.storeys),
      roof,
      ...(roof === 'gable' || roof === 'mansard' ? { ridge: Math.round(ridge * 1000) / 1000 } : {}),
      use: kind.use,
      look: Math.floor(rand() * 1024),
      area: areaOf(centroidOf(poly)),
    };
    const at = built.push(b) - 1;
    doors.push(add(c, n, -0.5));
    bucket(poly, at);
    return at;
  };
  /** Where a building's front goes from road `r` at `s` on `side`, set back `back` m from its sidewalk's edge: the middle of its front, the way along, the way back from the road. */
  const frontAt = (r: number, s: number, side: 1 | -1, back: number, ls?: number[]) => {
    const road = roads[r];
    const { p, t } = pointAt(road.points, s, ls);
    const n = leftOf(t).map((v) => v * side) as Pt;
    return { c: add(p, n, road.width / 2 + back), t, n };
  };
  const pickKind = (zone: Zone): Kind => {
    const list = KINDS[zone] as readonly (readonly [Kind, number])[];
    let r = rand() * list.reduce((n, [, w]) => n + w, 0);
    for (const [k, w] of list) if ((r -= w) <= 0) return k;
    return list[0][0];
  };
  // (Past the sidewalk: off the tiles the road's land takes, a metre and more past its edge.)
  const setback = (zone: Zone) => (zone === 'centre' ? range(rand, 2.3, 2.8) : zone === 'town' ? range(rand, 2.5, 5) : range(rand, 3, 7));

  // ---- the main towns' civic places, nearest their squares --------------------------------------
  const civic: { at: number; c: Civic }[] = [];
  for (const centre of centres) {
    const a = centre.area;
    const heart = centre.square ? centroidOf(centre.square) : pointAt(roads[centre.road].points, (centre.s0 + centre.s1) / 2).p;
    const fronts: { r: number; s: number; side: 1 | -1; far: number }[] = [];
    for (const r of areas[a].roads) {
      if (!ways[r] || roads[r].kind === 'country') continue;
      const ls = lengths(roads[r].points);
      for (let s = 6; s < ls[ls.length - 1] - 6; s += 3) for (const side of [1, -1] as const) fronts.push({ r, s, side, far: dist(pointAt(roads[r].points, s, ls).p, heart) });
    }
    fronts.sort((x, y) => x.far - y.far);
    for (const c of CIVIC) {
      for (const f of fronts) {
        const { c: mid, t, n } = frontAt(f.r, f.s, f.side, range(rand, 2.4, 4));
        const probe = add(mid, n, 2);
        if (blocked[Math.floor(probe[1]) * W + Math.floor(probe[0])]) continue;
        const at = put(mid, t, n, range(rand, ...c.kind.w), range(rand, ...c.kind.d), c.kind, c.endOn);
        if (at < 0) continue;
        civic.push({ at, c });
        break;
      }
    }
  }

  // ---- rows along the roads: the centres shoulder to shoulder, the rest with gaps -----------------
  const rows: { at: number; r: number; s: number; side: 1 | -1; back: number; zone: Zone }[] = [];
  areas.forEach((area, a) => {
    for (const r of area.roads) {
      if (!ways[r]) continue;
      const road = roads[r];
      const ls = lengths(road.points);
      const L = ls[ls.length - 1];
      for (const side of [1, -1] as const) {
        let s = 4 + rand() * 4;
        while (s < L - 4) {
          const probe = frontAt(r, s, side, 6, ls).c;
          const zone: Zone = centreAt(probe) ? 'centre' : a === 0 ? 'town' : 'hamlet';
          // (Now and then a gap: the more the further out.)
          const skip = zone === 'centre' ? 0.04 : zone === 'town' ? 0.22 * (1.2 - density) : 0.55;
          if (rand() < skip) {
            s += range(rand, 6, 16);
            continue;
          }
          const kind = pickKind(zone);
          let done = false;
          // (Smaller where it doesn't fit: narrower, and shallower in front of what stands behind.)
          for (const shrink of [1, 0.75, 0.55]) {
            const w = Math.max(kind.w[0] * 0.6, range(rand, ...kind.w) * shrink);
            const d = Math.max(6, range(rand, ...kind.d) * (shrink + 1) / 2);
            const back = setback(zone);
            const { c, t, n } = frontAt(r, s + w / 2, side, back, ls);
            const at = put(c, t, n, w, d, kind);
            if (at < 0) continue;
            rows.push({ at, r, s: s + w / 2, side, back: back + d, zone });
            s += w + (zone === 'centre' ? 0 : zone === 'town' ? range(rand, 1, 4) : range(rand, 4, 12));
            done = true;
            break;
          }
          if (!done) s += 3;
        }
      }
    }
  });

  // ---- a second row behind the first (the main towns): yards between, the road nearest still ------
  for (const row of [...rows]) {
    if (row.zone === 'hamlet' || rand() > (row.zone === 'centre' ? 0.9 : 0.55 * density + 0.2)) continue;
    const kind = pickKind(row.zone === 'centre' ? 'centre' : 'town');
    const w = range(rand, ...kind.w);
    const d = range(rand, ...kind.d);
    const { c, t, n } = frontAt(row.r, row.s, row.side, row.back + range(rand, 2, 5));
    const mid = add(c, n, d / 2);
    const near = nearestWay(mid[0], mid[1], 80);
    if (!near || near.road !== row.r) continue;
    put(c, t, n, w, d, kind);
  }

  // ---- the blocks filled in: on a jittered grid, turned to the road nearest ----------------------
  const infill = (a: number, chance: number, step: number, within?: Pt[]) => {
    const xs = within?.map((p) => p[0]);
    const zs = within?.map((p) => p[1]);
    const rect = xs && zs ? { x: Math.min(...xs) + 20, z: Math.min(...zs) + 20, w: Math.max(...xs) - Math.min(...xs) - 40, d: Math.max(...zs) - Math.min(...zs) - 40 } : areas[a].rect;
    const reach = INFILL_REACH[a === 0 ? 0 : 1];
    for (let z = rect.z - 20; z < rect.z + rect.d + 20; z += step) {
      for (let x = rect.x - 20; x < rect.x + rect.w + 20; x += step) {
        const p: Pt = [x + range(rand, -step / 3, step / 3), z + range(rand, -step / 3, step / 3)];
        if (within && !inside(within, p[0], p[1])) continue;
        if (p[0] < EDGE || p[1] < EDGE || p[0] >= W - EDGE || p[1] >= D - EDGE || blocked[Math.floor(p[1]) * W + Math.floor(p[0])]) continue;
        if (rand() > chance) continue;
        const near = nearestWay(p[0], p[1], reach + 10);
        if (!near || near.dist - roads[near.road].width / 2 > reach) continue;
        const { t } = pointAt(roads[near.road].points, near.s);
        const n = leftOf(t).map((v) => v * near.side) as Pt;
        const zone: Zone = centreAt(p) ? 'centre' : a === 0 ? 'town' : 'hamlet';
        const kind = pickKind(zone);
        // (In the centre smaller where it doesn't fit: it is built up close.)
        for (const shrink of zone === 'centre' ? [1, 0.75, 0.55] : [1]) {
          const w = Math.max(kind.w[0] * 0.6, range(rand, ...kind.w) * shrink);
          const d = Math.max(6, (range(rand, ...kind.d) * (shrink + 1)) / 2);
          if (put(add(p, n, -d / 2), t, n, w, d, kind) >= 0) break;
        }
      }
    }
  };
  const countIn = (a: number) => built.filter((b) => b.area === a).length;
  // (The centres first, closest: the densest.)
  for (const c of centres) infill(c.area, 1, 6, c.poly);
  infill(0, 0.3 + 0.45 * density, 12);
  for (let a = 1; a < areas.length; a++) infill(a, 0.25, 16);
  // (Short of the least: filled in more closely.)
  for (let a = 0; a < areas.length; a++) for (let k = 0; k < 3 && countIn(a) < (a === 0 ? MAIN_LEAST : HAMLET_LEAST); k++) infill(a, 1, 9 - k * 2);

  // ---- the centre the densest: houses and such taken away round it till it is --------------------
  // (Its square and what the centres' public places leave open count as open land: in a town whose
  // centre is mostly those, the rest is thinned.)
  for (const c of centres) {
    const box = areas[c.area].rect;
    // (The roads' tiles as the ground has them, a little wider than the check's: `DENSEST` spares it.)
    const taken = ground.roadTiles().slice();
    for (const s of slots) {
      const r = rectOf(s);
      for (let z = Math.max(0, r.z); z < Math.min(D, r.z + r.d); z++) for (let x = Math.max(0, r.x); x < Math.min(W, r.x + r.w); x++) taken[z * W + x] = 1;
    }
    for (const g of greens) fillPoly(g, W, D, (i) => (taken[i] = 1));
    for (let i = 0; i < taken.length; i++) if (water[i]) taken[i] = 1;
    let d = centreDensity(W, D, taken, box, c.poly, built.map((b) => b.poly));
    // (Not the densest: the outline narrowed to the street, where the land round it is the emptier.)
    for (const band of BANDS.filter((b) => b < c.band)) {
      if (d.centre >= d.rest * DENSEST) break;
      const poly = outline(roads, c, band);
      if (selfCrossing(poly)) continue;
      const n = centreDensity(W, D, taken, box, poly, built.map((b) => b.poly));
      if (n.centre / Math.max(1e-6, n.rest) <= d.centre / Math.max(1e-6, d.rest)) continue;
      [c.poly, c.band, d] = [poly, band, n];
    }
    let rest = d.built[1];
    // (Those not a place, outside the centre, in its box: the furthest from the centre first.)
    const mid = centroidOf(c.poly);
    const spare = built
      .map((b, i) => ({ i, far: dist(centroidOf(b.poly), mid) }))
      .filter(({ i }) => d.per[i] > 0 && built[i].area === c.area && !PLACE_KINDS.includes(built[i].use as TownPlace['kind']) && !centreAt(centroidOf(built[i].poly)))
      .sort((x, y) => y.far - x.far);
    const drop = new Set<number>();
    const want = d.centre / DENSEST;
    for (const { i } of spare) {
      if (rest / Math.max(1, d.open[1]) < want || built.length - drop.size <= 0 || countIn(c.area) - drop.size <= MAIN_LEAST) break;
      drop.add(i);
      rest -= d.per[i];
    }
    if (drop.size) {
      const keep = built.filter((_, i) => !drop.has(i));
      const doorsKept = doors.filter((_, i) => !drop.has(i));
      built.length = 0;
      built.push(...keep);
      doors.length = 0;
      doors.push(...doorsKept);
      civic.forEach((k) => (k.at -= [...drop].filter((j) => j < k.at).length));
      // (The buckets anew, by the buildings' new indices.)
      buckets.clear();
      built.forEach((b, at) => bucket(b.poly, at));
    }
  }

  // ---- by every farmstead: a farm building and a barn or two, behind it or beside ------------------
  const farmNames: string[] = [];
  const FARM = K('farm', [10, 14], [8, 11], [1, 2], ['gable']);
  const BARN = K('barn', [12, 18], [9, 12], [2, 2], ['gable']);
  for (const s of slots) {
    if (s.lot !== 'farm') continue;
    const r = rectOf(s);
    const mid: Pt = [r.x + r.w / 2, r.z + r.d / 2];
    const front = ([[0, -1], [-1, 0], [0, 1], [1, 0]] as Pt[])[s.facing];
    const aside: Pt = [-front[1], front[0]];
    const [depth, width] = s.facing % 2 ? [r.w, r.d] : [r.d, r.w];
    // (Turned to the road nearest, as every building is.)
    const near = nearestWay(mid[0], mid[1], 200);
    const t: Pt = near ? pointAt(roads[near.road].points, near.s).t : aside;
    const n: Pt = [-t[1], t[0]];
    const want = between(rand, 2, 3);
    let made = 0;
    for (const off of [4, 9, 15]) {
      for (const [u, v] of [[0, 1], [0.6, 1], [-0.6, 1], [1, 0], [-1, 0], [1, 0.6], [-1, 0.6]] as const) {
        if (made >= want) break;
        const kind = made === 0 ? FARM : BARN;
        const [w, d] = [range(rand, ...kind.w), range(rand, ...kind.d)];
        // Behind the lot (`v`) or beside it (`u`), `off` m clear of it and its own half again.
        const c = add(add(mid, front, -v * (depth / 2 + off + 7)), aside, u * (width / 2 + off + 9));
        const at = put(add(c, n, -d / 2), t, n, w, d, kind);
        if (at < 0) continue;
        if (made === 0) farmNames[at] = `${s.name} ${pick(rand, FARM_BUILDINGS)}`;
        made++;
      }
    }
  }

  // ---- the places: every public lot, then every backdrop place ------------------------------------
  const categories = content.careerCategories;
  const lastNames = content.names.last.length ? content.names.last : ['Smith'];
  const taken = new Set<string>(slots.map((s) => s.name));
  const streetNames = (a: number) => areas[a].roads.map((r) => roads[r].name).filter((n): n is string => !!n && !/Trail|Path/.test(n));
  const namer = (a: number): Namer => {
    const streets = streetNames(a);
    const self: Namer = {
      rand,
      town: areas[a].name,
      last: () => pick(rand, lastNames),
      // (A street's own word: "Mill" of "Mill Lane, Ashby".)
      street: () => (streets.length ? pick(rand, streets).split(',')[0].replace(/ (Street|Road|Lane|Way|Avenue|Row|Close|Drive|Hill|Walk|Crescent|Place|Gardens|Terrace|Parade|Square|Rise|Green|End|Mews|Court)$/, '') : areas[a].name),
      inn: () => `The ${pick(rand, INN_A)} ${pick(rand, INN_B)}`,
      unique: (draw: () => string) => {
        // (Drawn again a few times before it gets a number.)
        let name = draw();
        for (let k = 0; k < 8 && taken.has(name); k++) name = draw();
        let out = name;
        for (let k = 2; taken.has(out); k++) out = `${name} ${k}`;
        taken.add(out);
        return out;
      },
    };
    return self;
  };
  const places: TownPlace[] = [];
  const named = new Set<string>();
  for (const s of slots) {
    if (s.kind === 'house') continue;
    // (A hamlet's grocer or café is called as the main town's is: by its hamlet's name besides.)
    let name = s.name;
    if (named.has(name)) name = `${areas[s.area ?? 0].name} ${name.replace(/^The /, '')}`;
    for (let k = 2; named.has(name); k++) name = `${s.name} ${k}`;
    named.add(name);
    taken.add(name);
    const r = rectOf(s);
    const kind: TownPlace['kind'] = s.kind === 'outdoor' ? 'outdoor' : s.kind === 'park' ? (s.template === 'market' ? 'shop' : 'park') : s.template === 'cafe' ? 'cafe' : 'shop';
    // (A shop's tag at its door, on the street; a park's in its middle.)
    const front = ([[0.5, 0], [0, 0.5], [0.5, 1], [1, 0.5]] as const)[s.facing];
    const [x, z] = kind === 'park' || kind === 'outdoor' ? [r.x + r.w / 2, r.z + r.d / 2] : [r.x + r.w * front[0], r.z + r.d * front[1]];
    const work = LOT_WORK[s.template] ?? [];
    places.push({
      name,
      kind,
      x: r2(x),
      z: r2(z),
      plot: s.index,
      ...(kind === 'park' || kind === 'outdoor' ? {} : { hours: hoursFor(LOT_HOURS, work, categories) }),
      ...(work.length ? { work } : {}),
      area: s.area ?? 0,
    });
  }
  const civicAt = new Map(civic.map(({ at, c }) => [at, c]));
  built.forEach((b, i) => {
    const kind = b.use as TownPlace['kind'];
    if (!PLACE_KINDS.includes(kind)) return;
    const c = civicAt.get(i);
    const n = namer(b.area);
    const name = n.unique(() => (c ? c.name(n) : (farmNames[i] ?? placeName(kind, n))));
    const work = c ? c.work : (KIND_WORK[kind] ?? []);
    // (Its tag at its door: the middle of its front, on the street side.)
    b.place = places.length;
    places.push({ name, kind, x: r2(doors[i][0]), z: r2(doors[i][1]), hours: hoursFor(HOURS[kind as keyof typeof HOURS], work, categories), ...(work.length ? { work: [...work] } : {}), area: b.area });
  });
  // Every career a resident may hold works somewhere in each main town: any the civic places left
  // out (content added since) go to its first office.
  for (const centre of centres) {
    const here = places.filter((p) => p.area === centre.area);
    const office = here.find((p) => p.kind === 'office' && p.plot === undefined);
    if (!office) continue;
    for (const c of categories) {
      if (here.some((p) => p.work?.includes(c.id))) continue;
      office.work = [...(office.work ?? []), c.id];
      office.hours = hoursFor(office.hours ?? HOURS.office, [c.id], categories);
    }
  }
  return { backdrop: built, places };
}

/** A backdrop place's name (not a civic one's). */
function placeName(kind: TownPlace['kind'], n: Namer): string {
  switch (kind) {
    case 'shop':
      return n.rand() < 0.5 ? `${n.last()}'s ${pick(n.rand, SHOPS)}` : `${n.street()} ${pick(n.rand, SHOPS)}`;
    case 'cafe':
      return n.rand() < 0.5 ? `${n.last()}'s ${pick(n.rand, CAFES)}` : `The ${n.street()} ${pick(n.rand, CAFES)}`;
    case 'restaurant':
      return n.inn();
    case 'office':
      return `${n.street()} ${pick(n.rand, OFFICES)}`;
    case 'workshop':
      return `${n.last()}'s ${pick(n.rand, WORKSHOPS)}`;
    case 'school':
      return `${n.town} School`;
    case 'church':
      return `St ${pick(n.rand, SAINTS)}'s Church`;
    default:
      return `${n.town} Farm`;
  }
}

function centroidOf(poly: readonly Pt[]): Pt {
  return [poly.reduce((s, p) => s + p[0], 0) / poly.length, poly.reduce((s, p) => s + p[1], 0) / poly.length];
}
