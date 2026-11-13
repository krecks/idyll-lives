/**
 * The living town's debug fixture (`?ltFixture`, docs/design/living-town.md): until worldgen writes
 * `WorldMeta.backdrop`, a Big world without one gets backdrop buildings here, in its empty land beside
 * its roads, as the contract's rules have them:
 *
 * - none overlaps a lot (each keeps 1 m off its boundary), a road with its pavements (a road's `width`
 *   has them), a junction's pad, water, or another backdrop building;
 * - they face the nearest road (the long side along it), set back a little from the pavement;
 * - the main town's centre (round the junction nearest its middle) is the densest: terraces, shops,
 *   cafés and flats of 2 to 4 storeys, close together; houses further out; farms and barns along the
 *   country roads.
 *
 * Each place-kind building gets a `TownPlace` (`places`), and each main town a `centre`. The same world
 * always gets the same fixture (seeded by `meta.seed`). Debug only: nothing outside `?ltFixture` calls it.
 */

import type { BackdropBuilding, BackdropUse, RoadPlan, TownPlace, WorldMeta, WorldStructure } from '../core/protocol';
import { groundDepth } from '../core/protocol';
import { mulberry32 } from '../game/noise';
import { lengths, pointAt } from '../game/roads';

/** What the fixture adds to a world's meta. */
export interface LtFixture {
  backdrop: BackdropBuilding[];
  places: TownPlace[];
  centre: NonNullable<WorldMeta['centre']>;
}

/** The centre's half side (m) round the junction nearest the main town's middle: built densest within it; its square (`centre`, paved) this much. */
const CENTRE_HALF = 110;
const SQUARE_HALF = 28;
/** Kept off a lot's boundary, the roads' edges and each other (m). */
const LOT_GAP = 1.2;
const ROAD_GAP = 0.6;
/** A junction's pad: this far past its widest arm's half width (m), its corners' kerbs and crossings. */
const PAD = 5;
/** Steps along a road (m) between tries. */
const STEP = 2;

type Pt = [number, number];

/** Seeded fixtures by world (its seed, size, plots and roads). */
const made = new Map<string, LtFixture>();

/** Adds the fixture to `world`'s meta where it has no backdrop (a Big world only); returns whether it did. */
export function applyLtFixture(world: WorldStructure): boolean {
  const meta = world.meta;
  if (!meta || meta.backdrop || meta.world?.size !== 'big') return false;
  const fx = ltFixture(world);
  meta.backdrop = fx.backdrop;
  meta.places ??= fx.places;
  meta.centre ??= fx.centre;
  return true;
}

/** The fixture for `world` (made once per world). */
export function ltFixture(world: WorldStructure): LtFixture {
  const meta = world.meta;
  const key = `${meta?.seed ?? 0}|${world.width}x${groundDepth(world)}|${world.plots.length}|${meta?.roads?.length ?? 0}`;
  let fx = made.get(key);
  if (!fx) made.set(key, (fx = build(world)));
  return fx;
}

/** A 1 m grid over the world: 1 where nothing may stand. */
class Blocked {
  readonly cells: Uint8Array;
  constructor(
    readonly w: number,
    readonly d: number,
  ) {
    this.cells = new Uint8Array(w * d);
  }
  rect(x0: number, z0: number, x1: number, z1: number): void {
    for (let z = Math.max(0, Math.floor(z0)); z < Math.min(this.d, Math.ceil(z1)); z++) this.cells.fill(1, z * this.w + Math.max(0, Math.floor(x0)), z * this.w + Math.min(this.w, Math.ceil(x1)));
  }
  /** Every cell whose middle is within `r` of segment a–b. */
  capsule(a: Pt, b: Pt, r: number): void {
    const [x0, x1] = [Math.min(a[0], b[0]) - r, Math.max(a[0], b[0]) + r];
    const [z0, z1] = [Math.min(a[1], b[1]) - r, Math.max(a[1], b[1]) + r];
    for (let z = Math.max(0, Math.floor(z0)); z < Math.min(this.d, Math.ceil(z1)); z++) {
      for (let x = Math.max(0, Math.floor(x0)); x < Math.min(this.w, Math.ceil(x1)); x++) {
        if (segDist(x + 0.5, z + 0.5, a, b) <= r) this.cells[z * this.w + x] = 1;
      }
    }
  }
  /** Every cell whose middle is in polygon `poly` (grown by `grow` m: its box's). */
  poly(poly: readonly Pt[], grow = 0): void {
    const xs = poly.map((p) => p[0]);
    const zs = poly.map((p) => p[1]);
    for (let z = Math.max(0, Math.floor(Math.min(...zs) - grow)); z < Math.min(this.d, Math.ceil(Math.max(...zs) + grow)); z++) {
      for (let x = Math.max(0, Math.floor(Math.min(...xs) - grow)); x < Math.min(this.w, Math.ceil(Math.max(...xs) + grow)); x++) {
        if (inPoly(x + 0.5, z + 0.5, poly) || (grow > 0 && polyDist(x + 0.5, z + 0.5, poly) <= grow)) this.cells[z * this.w + x] = 1;
      }
    }
  }
  /** Whether every cell polygon `poly` touches is free (and in the world). */
  free(poly: readonly Pt[]): boolean {
    const xs = poly.map((p) => p[0]);
    const zs = poly.map((p) => p[1]);
    const [x0, x1, z0, z1] = [Math.floor(Math.min(...xs)), Math.ceil(Math.max(...xs)), Math.floor(Math.min(...zs)), Math.ceil(Math.max(...zs))];
    if (x0 < 0 || z0 < 0 || x1 > this.w || z1 > this.d) return false;
    for (let z = z0; z < z1; z++) {
      for (let x = x0; x < x1; x++) {
        if (!this.cells[z * this.w + x]) continue;
        // (A cell the polygon only grazes: any of its corners or middle inside.)
        if (inPoly(x + 0.5, z + 0.5, poly) || polyDist(x + 0.5, z + 0.5, poly) < 0.71) return false;
      }
    }
    return true;
  }
}

function segDist(px: number, pz: number, a: Pt, b: Pt): number {
  const dx = b[0] - a[0];
  const dz = b[1] - a[1];
  const l2 = dx * dx + dz * dz;
  const t = l2 > 0 ? Math.max(0, Math.min(1, ((px - a[0]) * dx + (pz - a[1]) * dz) / l2)) : 0;
  return Math.hypot(px - (a[0] + t * dx), pz - (a[1] + t * dz));
}

function inPoly(x: number, z: number, poly: readonly Pt[]): boolean {
  let inside = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const [xi, zi] = poly[i];
    const [xj, zj] = poly[j];
    if (zi > z !== zj > z && x < ((xj - xi) * (z - zi)) / (zj - zi) + xi) inside = !inside;
  }
  return inside;
}

function polyDist(x: number, z: number, poly: readonly Pt[]): number {
  let d = Infinity;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) d = Math.min(d, segDist(x, z, poly[j], poly[i]));
  return d;
}

/** A rectangle's corners, counter-clockwise (x towards z), `along` its frontage direction, centred at (cx, cz). */
function rectPoly(cx: number, cz: number, ux: number, uz: number, front: number, depth: number): Pt[] {
  const [vx, vz] = [-uz, ux];
  const [a, b] = [front / 2, depth / 2];
  return [
    [cx - ux * a - vx * b, cz - uz * a - vz * b],
    [cx + ux * a - vx * b, cz + uz * a - vz * b],
    [cx + ux * a + vx * b, cz + uz * a + vz * b],
    [cx - ux * a + vx * b, cz - uz * a + vz * b],
  ];
}

/** Counter-clockwise in town metres (x towards z: a positive signed area). */
function ccw(poly: Pt[]): Pt[] {
  let s = 0;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) s += poly[j][0] * poly[i][1] - poly[i][0] * poly[j][1];
  return s >= 0 ? poly : [...poly].reverse();
}

/** What a stretch of road gets: its kind of building and how they're laid out. */
interface Style {
  front: [number, number];
  depth: [number, number];
  setback: [number, number];
  gap: [number, number];
  storeys: [number, number];
  /** How often a try is made (the rest of the road left open). */
  fill: number;
  /** Rows of buildings back from the road (the first along it, the rest behind it, each facing it), and the yard between rows (m). */
  rows: number;
  yard: [number, number];
  uses: readonly [BackdropUse, number][];
  roofs: readonly [BackdropBuilding['roof'], number][];
}

const CENTRE: Style = {
  rows: 3,
  yard: [2, 5],
  front: [6, 11],
  depth: [9, 14],
  setback: [0.3, 1],
  gap: [0.3, 0.8],
  storeys: [2, 4],
  fill: 1,
  uses: [['terrace', 4], ['shop', 3], ['flats', 2], ['cafe', 1.2], ['restaurant', 1], ['office', 1], ['hall', 0.3], ['church', 0.15]],
  roofs: [['gable', 4], ['flat', 2], ['mansard', 1.5], ['hip', 1]],
};
const TOWN: Style = {
  rows: 2,
  yard: [4, 9],
  front: [7, 12],
  depth: [8, 12],
  setback: [1.5, 4],
  gap: [1.2, 4],
  storeys: [1, 2],
  fill: 0.85,
  uses: [['house', 6], ['terrace', 1.5], ['flats', 0.7], ['shop', 0.5], ['workshop', 0.5], ['office', 0.3], ['school', 0.08], ['church', 0.05]],
  roofs: [['gable', 5], ['hip', 3], ['flat', 0.5], ['mansard', 0.3]],
};
const HAMLET: Style = {
  rows: 2,
  yard: [5, 12],
  front: [7, 11],
  depth: [7, 10],
  setback: [1.5, 5],
  gap: [2, 6],
  storeys: [1, 2],
  fill: 0.7,
  uses: [['house', 6], ['workshop', 0.7], ['shop', 0.4], ['farm', 0.6], ['barn', 0.6], ['church', 0.08]],
  roofs: [['gable', 6], ['hip', 2.5], ['mansard', 0.2]],
};
const COUNTRY: Style = {
  rows: 1,
  yard: [0, 0],
  front: [9, 18],
  depth: [8, 12],
  setback: [4, 10],
  gap: [25, 70],
  storeys: [1, 2],
  fill: 0.25,
  uses: [['farm', 2], ['barn', 2.5], ['house', 1], ['workshop', 0.3]],
  roofs: [['gable', 5], ['hip', 1]],
};

const PLACE_KIND: Partial<Record<BackdropUse, TownPlace['kind']>> = { shop: 'shop', cafe: 'cafe', restaurant: 'restaurant', office: 'office', school: 'school', church: 'church', workshop: 'workshop', farm: 'farm' };
const PLACE_NAMES: Record<TownPlace['kind'], readonly string[]> = {
  shop: ['Corner Shop', 'Greengrocer', 'Hardware', 'Bookshop', 'Florist', 'Butcher', 'Chemist', 'Newsagent', 'Bakery', 'Deli'],
  cafe: ['Little Café', 'Tea Room', 'Coffee House', 'Morning Cup'],
  restaurant: ['Bistro', 'Trattoria', 'Noodle Bar', 'Fish Kitchen', 'The Old Oak'],
  office: ['Town Office', 'Surveyors', 'Insurance', 'Architects', 'The Gazette'],
  school: ['Primary School', 'Junior School'],
  church: ['St Anne’s', 'Chapel'],
  workshop: ['Joinery', 'Garage', 'Pottery', 'Smithy'],
  farm: ['Farm Shop', 'Dairy'],
  park: ['Park'],
  outdoor: ['Common'],
};
const PLACE_WORK: Partial<Record<TownPlace['kind'], string[]>> = { shop: ['retail'], cafe: ['culinary'], restaurant: ['culinary'], office: ['business', 'writing'], school: ['education'], workshop: ['trades'], farm: ['farming'] };

function pick<T>(rand: () => number, list: readonly [T, number][]): T {
  let total = 0;
  for (const [, w] of list) total += w;
  let r = rand() * total;
  for (const [v, w] of list) if ((r -= w) <= 0) return v;
  return list[list.length - 1][0];
}

const between = (rand: () => number, [a, b]: [number, number]) => a + rand() * (b - a);

function build(world: WorldStructure): LtFixture {
  const meta = world.meta!;
  const rand = mulberry32(((meta.seed ?? 1) ^ 0x1b7f0c3) >>> 0);
  const W = world.width;
  const D = groundDepth(world);
  const grid = new Blocked(W, D);
  const roads: readonly RoadPlan[] = meta.roads ?? [];

  // What nothing may stand on: lots (and a gap round them), roads with their pavements, junction pads, water, greens and paths.
  for (const p of world.plots) grid.rect(p.x - LOT_GAP, p.z - LOT_GAP, p.x + p.w + LOT_GAP, p.z + p.d + LOT_GAP);
  for (const r of roads) for (let i = 0; i + 1 < r.points.length; i++) grid.capsule(r.points[i], r.points[i + 1], r.width / 2 + ROAD_GAP);
  for (const j of junctionsOf(world, roads)) grid.capsule([j.x, j.z], [j.x, j.z], j.r);
  for (const z of meta.zones ?? []) if (z.kind === 'water') grid.poly(z.poly, 3);
  for (const g of meta.greens ?? []) grid.poly(g, 1);
  for (const p of meta.paths ?? []) grid.rect(p.x - 0.5, p.z - 0.5, p.x + p.w + 0.5, p.z + p.d + 0.5);
  const water = meta.world?.landscape.water;
  if (water?.kind === 'sea') {
    if (water.side === 'zmin') grid.rect(0, 0, W, water.shore + 8);
    else grid.rect(0, water.shore - 8, W, D);
  } else if (water?.kind === 'lake') {
    const n = 32;
    grid.poly(Array.from({ length: n }, (_, k): Pt => [water.x + Math.cos((k / n) * Math.PI * 2) * (water.rx + 8), water.z + Math.sin((k / n) * Math.PI * 2) * (water.rz + 8)]));
  }
  for (const [x, z] of world.exits) grid.capsule([x, z], [x, z], 4);

  // The main town's centre: a square round the junction (or road point) nearest its middle.
  const areas = meta.areas ?? [];
  const main = areas[0] ?? { name: '', kind: 'hamlet' as const, x: 0, z: 0, w: W, d: D };
  const mid: Pt = [main.x + main.w / 2, main.z + main.d / 2];
  let best: Pt = mid;
  let bestD = Infinity;
  for (const r of roads) {
    if (r.kind === 'country' || r.kind === 'trail' || (r.area !== undefined && r.area !== 0)) continue;
    for (const p of r.points) {
      const d = Math.hypot(p[0] - mid[0], p[1] - mid[1]);
      if (d < bestD) [best, bestD] = [p, d];
    }
  }
  const centrePoly: Pt[] = ccw([
    [best[0] - SQUARE_HALF, best[1] - SQUARE_HALF],
    [best[0] + SQUARE_HALF, best[1] - SQUARE_HALF],
    [best[0] + SQUARE_HALF, best[1] + SQUARE_HALF],
    [best[0] - SQUARE_HALF, best[1] + SQUARE_HALF],
  ]);
  const inCentre = (x: number, z: number) => Math.abs(x - best[0]) < CENTRE_HALF && Math.abs(z - best[1]) < CENTRE_HALF;
  const areaAt = (x: number, z: number): number => {
    const i = areas.findIndex((a) => x >= a.x && z >= a.z && x < a.x + a.w && z < a.z + a.d);
    return i < 0 ? 0 : i;
  };

  const backdrop: BackdropBuilding[] = [];
  const places: TownPlace[] = [];
  const named = new Map<TownPlace['kind'], number>();

  /** Tries a building at arc length `t` along road `r` on `side` (+1 left), in `style`; its frontage and gap (0: none fitted). */
  const tryAt = (r: RoadPlan, ls: number[], t: number, side: 1 | -1, style: Style, back: number): number => {
    const front = between(rand, style.front);
    // (Along the chord of the road over its frontage: a curved road's houses step round it.)
    const a = pointAt(r.points, t, ls).p;
    const b = pointAt(r.points, t + front, ls).p;
    const len = Math.hypot(b[0] - a[0], b[1] - a[1]);
    if (len < front * 0.8) return 0;
    const [ux, uz] = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
    const [nx, nz] = [-uz * side, ux * side];
    const t0 = 0;
    const depth = between(rand, style.depth);
    const setback = between(rand, style.setback);
    const off = r.width / 2 + ROAD_GAP + setback + back + depth / 2;
    const px = a[0] + ux * (t0 + front / 2) + nx * off;
    const pz = a[1] + uz * (t0 + front / 2) + nz * off;
    const poly = rectPoly(px, pz, ux, uz, front, depth);
    const gap = between(rand, style.gap);
    if (!grid.free(rectPoly(px, pz, ux, uz, front + 0.6, depth + 0.6))) return 0;
    const centre = style === CENTRE;
    let use = pick(rand, style.uses);
    // (The centre's corner plots and its few big ones: a hall or a church only once or twice.)
    if ((use === 'church' || use === 'school') && backdrop.some((o) => o.use === use && Math.hypot(o.poly[0][0] - px, o.poly[0][1] - pz) < 250)) use = centre ? 'shop' : 'house';
    let storeys = Math.round(between(rand, style.storeys));
    if (use === 'flats') storeys = Math.max(storeys, centre ? 3 : 2) + (rand() < 0.3 ? 1 : 0);
    if (use === 'barn' || use === 'church' || use === 'workshop') storeys = 1;
    let roof = pick(rand, style.roofs);
    if (use === 'flats' && rand() < 0.5) roof = 'flat';
    if (use === 'barn' || use === 'church') roof = 'gable';
    const area = areaAt(px, pz);
    // The ridge along the road (a terrace's, a shop's), or across it for some houses (the gable end on the street).
    const gableOn = !centre && use === 'house' && front < depth + 1 && rand() < 0.35;
    // (A turn round +y: the ridge runs along (cos r, −sin r) in x, z, `game/backdrop.ts`.)
    const ridge = Math.atan2(-(gableOn ? nz : uz), gableOn ? nx : ux);
    const b0: BackdropBuilding = { poly: ccw(poly), y: 0, storeys: Math.max(1, Math.min(6, storeys)), roof, ...(roof === 'gable' || roof === 'mansard' ? { ridge } : {}), use, look: Math.floor(rand() * 1024), area };
    const kind = PLACE_KIND[use];
    if (kind && (use !== 'farm' || rand() < 0.4)) {
      const n = named.get(kind) ?? 0;
      named.set(kind, n + 1);
      const names = PLACE_NAMES[kind];
      const name = `${names[n % names.length]}${n >= names.length ? ` ${Math.floor(n / names.length) + 1}` : ''}`;
      const open = kind === 'restaurant' ? 17 * 60 : kind === 'cafe' ? 7 * 60 : kind === 'church' ? 9 * 60 : 9 * 60;
      const close = kind === 'restaurant' ? 23 * 60 : kind === 'cafe' ? 17 * 60 : kind === 'church' ? 18 * 60 : kind === 'office' ? 17 * 60 + 30 : 18 * 60;
      const days = kind === 'office' || kind === 'school' ? 0b0011111 : kind === 'church' ? 0b1111111 : 0b0111111;
      b0.place = places.length;
      places.push({ name, kind, x: a[0] + ux * (front / 2) + nx * (off - depth / 2), z: a[1] + uz * (front / 2) + nz * (off - depth / 2), hours: { open, close, days }, ...(PLACE_WORK[kind] ? { work: PLACE_WORK[kind] } : {}), area });
    }
    backdrop.push(b0);
    grid.poly(rectPoly(px, pz, ux, uz, front + 0.6, depth + 0.6));
    return front + gap;
  };

  // The roads with the most to line first (the centre's, then the town's, the hamlets', the country's).
  const order = roads
    .map((r, i) => ({ r, i, rank: r.kind === 'trail' ? 9 : r.kind === 'country' ? 3 : r.points.some((p) => inCentre(p[0], p[1])) ? 0 : r.area === 0 || r.area === undefined ? 1 : 2 }))
    .filter((o) => o.rank < 9)
    .sort((a, b) => a.rank - b.rank || a.i - b.i);
  // Row by row: every road's frontage first, then the rows behind it.
  for (let row = 0; row < 3; row++) {
    for (const { r } of order) {
      const ls = lengths(r.points);
      const len = ls[ls.length - 1];
      for (const side of [1, -1] as const) {
        if (r.lots && r.lots !== side && r.kind !== 'country') continue;
        for (let t = 2; t < len - 8; ) {
          const [x, z] = pointAt(r.points, t, ls).p;
          const area = areas[areaAt(x, z)];
          const style = r.kind === 'country' || area?.kind === 'countryside' ? COUNTRY : inCentre(x, z) ? CENTRE : areaAt(x, z) === 0 ? TOWN : HAMLET;
          if (row >= style.rows) {
            t += 8;
            continue;
          }
          // (Behind the first row: past a front row's depth and a yard, each row a little further.)
          const back = row * (style.depth[1] + between(rand, style.yard));
          const taken = rand() < style.fill ? tryAt(r, ls, t, side, style, back) : 0;
          t += taken > 0 ? taken : STEP;
        }
      }
    }
  }
  return { backdrop, places, centre: [{ area: 0, poly: centrePoly }] };
}

/** The world's junctions as pads: where three or more road ends meet (or the sim's list), a radius each. */
function junctionsOf(world: WorldStructure, roads: readonly RoadPlan[]): { x: number; z: number; r: number }[] {
  const out: { x: number; z: number; r: number }[] = [];
  const widest = (x: number, z: number) => {
    let w = 0;
    for (const r of roads) for (const p of r.points) if (Math.hypot(p[0] - x, p[1] - z) < 1) w = Math.max(w, r.width);
    return w;
  };
  for (const j of world.junctions ?? []) out.push({ x: j.x, z: j.z, r: (widest(j.x, j.z) || 10) / 2 + PAD });
  // (Without the sim's list: every point three or more roads share.)
  if (!world.junctions?.length) {
    const count = new Map<string, { x: number; z: number; n: number; w: number }>();
    for (const r of roads) {
      for (const p of r.points) {
        const k = `${Math.round(p[0])},${Math.round(p[1])}`;
        const c = count.get(k) ?? { x: p[0], z: p[1], n: 0, w: 0 };
        c.n++;
        c.w = Math.max(c.w, r.width);
        count.set(k, c);
      }
    }
    for (const c of count.values()) if (c.n >= 2) out.push({ x: c.x, z: c.z, r: c.w / 2 + PAD });
  }
  return out;
}
