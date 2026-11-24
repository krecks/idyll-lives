/**
 * The town's paved ground, one source for near and far: the centre's outlines and squares, the
 * courts and yards the town lays (`WorldMeta.paving`), the car parks' aprons and their bays' lines
 * (`WorldStructure.carParks`, `parking`), and the P signs. The far colour map
 * (`pc/groundTiles.ts`), the near ground (`streetGround.ts`) and what is no lawn (`nature.ts`
 * `unlawned`) all read it, so the centre is paved at every distance.
 *
 * The rule: a centre's outline is paved, but for its lots, its greens and its water; a place's or a
 * flats building's lot in it is paved through its `court` pieces. Nothing here guesses what the
 * town did not say: a town whose generator wrote no `paving` has its centres and squares paved from
 * `meta.centre`, and its bays drawn where the simulation parks.
 *
 * Paving is laid by tiles, a tile whole where it is wholly in a piece (less its holes and cuts) and
 * in part along a piece's edge, cut straight along it (`parts`), so a slanted outline is not
 * stepped; the far map and the near ground draw the same tiles and the same parts. The lawn's grass
 * is hidden on every tile paved at all (`mask`'s `any`). A car park's apron is turned to its road
 * and is drawn by its own outline.
 */

import type { PavedPiece, RoadPlan, WorldStructure } from '../../core/protocol';
import { ccw, signedArea2, toOutline } from '../../game/footprints';
import { LAND_CELL } from '../../game/land';
import { centroid, rasterRoads } from '../../game/roads';
import { carriageway } from './junctions';

/** An outline on the ground (town metres, x and z). */
export type Poly = [number, number][];

/** A rectangle on the ground (town metres, x and z its corner). */
export interface Rect {
  x: number;
  z: number;
  w: number;
  d: number;
}

/** What `townPaving` reads: the lots (to leave homes out), the meta's centres and paving, the bays and car parks. */
export type PavingWorld = Pick<WorldStructure, 'plots' | 'meta' | 'parking' | 'carParks'>;

/** What a piece is paved with: the centre's stone in courses (`stone`), a workshop's yard, a car park's asphalt. */
export type PavingSurface = 'stone' | 'yard' | 'asphalt';

/**
 * A piece to pave: what it is (a car park's apron is `carpark`), its outline (counter-clockwise seen from above), the lots cut out of it (`holes`) and
 * the greens and water (`cuts`, outlines), how far apart its courses' joints are (m, 0: none), what
 * it is paved with, and how it is laid: by the tiles whose middles it covers (`tiles`), or by its
 * own outline (`exact`: a convex outline turned to its road). `plot`: the lot it is the court or
 * yard of (that lot draws it while it is in view).
 */
export interface PavingDraw {
  kind: PavedPiece['kind'] | 'carpark';
  poly: [number, number][];
  holes: Rect[];
  course: number;
  surface: PavingSurface;
  edge: 'tiles' | 'exact';
  cuts?: [number, number][][];
  plot?: number;
}

/** A parking bay as drawn: its corners (the bay's own rectangle) and its painted lines (each a segment). */
export interface BayDraw {
  corners: [number, number][];
  lines: [[number, number], [number, number]][];
}

/** The town's paving as drawn (`townPaving`): the pieces, the bays' lines, the P signs (x, z, yaw) and a mask of what is paved. */
export interface TownPaving {
  pieces: PavingDraw[];
  bays: BayDraw[];
  signs: [number, number, number][];
  /**
   * One per tile over `w` × `d` (row-major from the world's corner): 1 where the ground is paved
   * (`any`: where any of a piece reaches into the tile; else where it covers the tile's middle).
   */
  mask(w: number, d: number, any?: boolean): Uint8Array;
  /**
   * What each tile laid by tiles is paved with, over `w` × `d`, whole or in part: `TILE.stone`,
   * `TILE.yard`, else 0 (the pieces laid by their own outlines are not in it). Near and far draw
   * these tiles.
   */
  tiles(w: number, d: number): Uint8Array;
  /**
   * The tiles laid in part, along a piece's edge (by index over `w` × `d`): the part of each that
   * is paved, convex outlines inside the tile (more than one where two pieces' edges meet in it).
   */
  parts(w: number, d: number): ReadonlyMap<number, readonly Poly[]>;
}

/** What `TownPaving.tiles` holds per tile. */
export const TILE = { none: 0, stone: 1, yard: 2 } as const;

/**
 * The paving's colours as they show (sRGB, the texture's mean through its material and tone): the
 * centre's stone and the joints between its courses, a yard, a car park's asphalt and the bays'
 * paint. The far map paints them (`groundTiles.ts`
 * `MAP`) and the near ground tones its surfaces to them (`streetGround.ts`).
 */
export const PAVING_LOOK = {
  stone: '#B3A58E',
  joint: '#A4967F',
  yard: '#9A8C74',
  asphalt: '#55544F',
  line: '#D6D6D0',
} as const;

/** The centre's courses (m apart), and how wide a joint between two is (m). */
export const COURSE = 5;
export const JOINT = 0.5;
/**
 * A bay as the simulation parks in it (`parking.ts` `BAY_WIDE`, `BAY_DEEP` and `BAY_LINES`, which `paving.test.ts` holds
 * these to: that module brings the town's generator with it): 3 m along its road by 6 deep, its lines (`LINE` m wide)
 * 2.8 m apart along its sides (two bays side by side share one, between them) and 5.6 m long, and one across its far
 * end, as far out as they reach. A lone bay's apron is a little bigger than the bay, so its lines lie on asphalt.
 */
export const BAY = { w: 3, d: 6, lines: [2.8, 5.6] } as const;
export const LINE = 0.1;
/** Where a bay's lines run from its middle: its sides' across it, how far they reach along it, its far end's along it. */
const [SIDE, HALF] = [BAY.lines[0] / 2, BAY.lines[1] / 2];
const END = HALF - LINE / 2;
const LONE = { w: 3.4, d: 6.2 } as const;
/** A tile wholly under a road's carriageway is the road's, not paved: its middle this far inside the kerb (half its diagonal). */
const UNDER_ROAD = Math.SQRT1_2;
/** A tile's part along an edge smaller than this (m²) is not paved; a corner this near an outline (m) is on it. */
const SLIVER = 1e-3;
const ON_EDGE = 1e-6;
/** How a piece's courses are by kind. */
const COURSES: Record<PavedPiece['kind'], number> = { centre: COURSE, square: COURSE, court: COURSE, yard: 0 };

/** Each meta's paving, for the lots, bays and car parks it was drawn with. */
const made = new WeakMap<object, { plots: unknown; parking: unknown; carParks: unknown; paving: TownPaving }>();
/** Nothing paved: a world without a meta (the build previews, older tests). */
const NONE: TownPaving = { pieces: [], bays: [], signs: [], mask: (w, d) => new Uint8Array(w * d), tiles: (w, d) => new Uint8Array(w * d), parts: () => new Map() };

/** The town's paved ground (see the top of this file), the same object for the same meta, lots, bays and car parks. */
export function townPaving(world: PavingWorld): TownPaving {
  const meta = world.meta;
  if (!meta) return NONE;
  const known = made.get(meta);
  if (known && known.plots === world.plots && known.parking === world.parking && known.carParks === world.carParks) return known.paving;
  const paving = drawPaving(world);
  made.set(meta, { plots: world.plots, parking: world.parking, carParks: world.carParks, paving });
  return paving;
}

function drawPaving(world: PavingWorld): TownPaving {
  const meta = world.meta!;
  const lots: Rect[] = world.plots.map((p) => ({ x: p.x, z: p.z, w: p.w, d: p.d }));
  const holesIn = (poly: readonly (readonly [number, number])[]): Rect[] => {
    const [x0, z0, x1, z1] = boxOf(poly);
    return lots.filter((r) => r.x < x1 && r.x + r.w > x0 && r.z < z1 && r.z + r.d > z0);
  };
  const unpaved = [...(meta.greens ?? []), ...(meta.zones ?? []).filter((z) => z.kind === 'water').map((z) => z.poly)];
  const cutsIn = (poly: readonly (readonly [number, number])[]) => {
    const [x0, z0, x1, z1] = boxOf(poly);
    return unpaved.filter((c) => {
      const [a, b, e, f] = boxOf(c);
      return a < x1 && e > x0 && b < z1 && f > z0;
    });
  };
  const pieces: PavingDraw[] = [];
  const written = meta.paving ?? [];
  const has = new Set(written.map((p) => p.kind));
  // The centres and their squares as the town file has them, where its paving does not say them itself.
  for (const c of meta.centre ?? []) {
    if (!has.has('centre') && c.poly.length >= 3) pieces.push({ kind: 'centre', poly: c.poly, holes: holesIn(c.poly), cuts: cutsIn(c.poly), course: COURSE, surface: 'stone', edge: 'tiles' });
    if (!has.has('square') && c.square && c.square.length >= 3) pieces.push({ kind: 'square', poly: c.square, holes: holesIn(c.square), cuts: cutsIn(c.square), course: COURSE, surface: 'stone', edge: 'tiles' });
  }
  // What the town lays: a centre or square less its lots, greens and water; a court or yard as it is (on its lot).
  for (const p of written) {
    if (p.poly.length < 3) continue;
    const open = p.kind === 'centre' || p.kind === 'square';
    pieces.push({ kind: p.kind, poly: p.poly, holes: open ? holesIn(p.poly) : [], cuts: open ? cutsIn(p.poly) : undefined, course: COURSES[p.kind], surface: p.kind === 'yard' ? 'yard' : 'stone', edge: 'tiles', plot: p.plot });
  }
  // The car parks' aprons, and a lone bay's own (a bay of no car park the town file has).
  const parks = world.carParks ?? [];
  for (const c of parks) if (c.poly.length >= 3) pieces.push({ kind: 'carpark', poly: c.poly, holes: [], course: 0, surface: 'asphalt', edge: 'exact' });
  const bays: BayDraw[] = [];
  const sides: { park: number; a: [number, number]; b: [number, number] }[] = [];
  (world.parking ?? []).forEach((bay, i) => {
    const lone = bay.park === undefined || !parks[bay.park];
    const park = lone ? -1 - i : bay.park!;
    const at = bayFrame(bay.x, bay.z, bay.yaw);
    if (lone) pieces.push({ kind: 'carpark', poly: at.rect(LONE.w, LONE.d), holes: [], course: 0, surface: 'asphalt', edge: 'exact' });
    const lines: BayDraw['lines'] = [at.seg(-SIDE, -HALF, -SIDE, HALF), at.seg(SIDE, -HALF, SIDE, HALF), at.seg(-BAY.w / 2, END, BAY.w / 2, END)];
    bays.push({ corners: at.rect(BAY.w, BAY.d), lines });
    sides.push({ park, a: lines[0][0], b: lines[0][1] }, { park, a: lines[1][0], b: lines[1][1] });
  });
  // Two bays side by side share the line between them: the two sides' lines as one, between where they were.
  // (Only a car park's own: a lone bay is a park of its own.)
  const mid = (p: [number, number], q: [number, number]) => [(p[0] + q[0]) / 2, (p[1] + q[1]) / 2] as [number, number];
  for (const row of Map.groupBy(sides, (s) => s.park).values()) {
    for (let i = 0; i < row.length; i++) {
      for (let j = i + 1; j < row.length; j++) {
        const [s, t] = [row[i], row[j]];
        if (Math.hypot(s.a[0] - t.a[0], s.a[1] - t.a[1]) > 0.3 || Math.hypot(s.b[0] - t.b[0], s.b[1] - t.b[1]) > 0.3) continue;
        [s.a, s.b] = [mid(s.a, t.a), mid(s.b, t.b)];
        [t.a, t.b] = [s.a, s.b];
      }
    }
  }
  bays.forEach((b, i) => {
    const [l, r] = [sides[i * 2], sides[i * 2 + 1]];
    b.lines[0] = [l.a, l.b];
    b.lines[1] = [r.a, r.b];
  });
  // (A shared line once: the second bay's copy of it goes.)
  const seen = new Set<string>();
  for (const b of bays) {
    b.lines = b.lines.filter(([a, c]) => {
      const key = [a, c].flat().map((v) => Math.round(v * 100)).join();
      const back = [c, a].flat().map((v) => Math.round(v * 100)).join();
      if (seen.has(key) || seen.has(back)) return false;
      seen.add(key);
      return true;
    });
  }
  const signs = parks.map((c) => [c.sign[0], c.sign[1], c.sign[2]] as [number, number, number]);
  const roads = meta.roads ?? [];
  const tileMemo = new Map<string, ReturnType<typeof layTiles>>();
  const maskMemo = new Map<string, Uint8Array>();
  const laidAt = (w: number, d: number) => {
    const key = `${w}x${d}`;
    let out = tileMemo.get(key);
    if (!out) tileMemo.set(key, (out = layTiles(pieces, roads, w, d)));
    return out;
  };
  const tiles = (w: number, d: number): Uint8Array => laidAt(w, d).codes;
  const parts = (w: number, d: number): ReadonlyMap<number, readonly Poly[]> => laidAt(w, d).parts;
  const mask = (w: number, d: number, any = false): Uint8Array => {
    const key = `${w}x${d}${any ? '+' : ''}`;
    let out = maskMemo.get(key);
    if (out) return out;
    out = new Uint8Array(w * d);
    const { codes, parts: cut } = laidAt(w, d);
    // (A tile laid in part: paved at all with `any`, else where its middle is.)
    for (let i = 0; i < out.length; i++) {
      if (!codes[i]) continue;
      const polys = cut.get(i);
      if (!polys || any || polys.some((q) => inConvex(q, (i % w) + 0.5, Math.floor(i / w) + 0.5))) out[i] = 1;
    }
    for (const p of pieces) if (p.edge === 'exact') fillConvex(out, w, d, p.poly, p.holes, any);
    maskMemo.set(key, out);
    return out;
  };
  return { pieces, bays, signs, mask, tiles, parts };
}

/** A bay's frame: a point `u` across it and `v` along the way its car faces (from its middle), as segments and rectangles. */
function bayFrame(x: number, z: number, yaw: number) {
  const [fx, fz] = [Math.sin(yaw), Math.cos(yaw)];
  const P = (u: number, v: number): [number, number] => [x + fz * u + fx * v, z - fx * u + fz * v];
  return {
    seg: (u0: number, v0: number, u1: number, v1: number): [[number, number], [number, number]] => [P(u0, v0), P(u1, v1)],
    /** Its rectangle `w` across by `d` along, counter-clockwise seen from above. */
    rect: (w: number, d: number): [number, number][] => ccw([P(-w / 2, -d / 2), P(w / 2, -d / 2), P(w / 2, d / 2), P(-w / 2, d / 2)]),
  };
}

/** A polygon's box: x0, z0, x1, z1. */
export function boxOf(poly: readonly (readonly [number, number])[]): [number, number, number, number] {
  let [x0, z0, x1, z1] = [Infinity, Infinity, -Infinity, -Infinity];
  for (const [x, z] of poly) [x0, z0, x1, z1] = [Math.min(x0, x), Math.min(z0, z), Math.max(x1, x), Math.max(z1, z)];
  return [x0, z0, x1, z1];
}

/**
 * The tiles the pieces laid by tiles pave (`TownPaving.tiles`), and the part of each tile they pave in part
 * (`TownPaving.parts`): each piece's ground less its holes and cuts, stone over yard where two meet, and none on a tile
 * wholly under a carriageway (the road is drawn there). A tile is paved whole where its four corners are in the piece;
 * where some are, it is paved as far as the piece's edge, which runs straight across the tile between where its sides
 * cross the edge (each side cut where the distance to the outline, signed, comes to nothing): so a slanted outline
 * is paved along its line, not in steps of a tile, near and far alike. A lot (a hole) is whole tiles, as lots are.
 */
function layTiles(pieces: readonly PavingDraw[], roads: readonly RoadPlan[], w: number, d: number): { codes: Uint8Array; parts: Map<number, Poly[]> } {
  const codes = new Uint8Array(w * d);
  const whole = new Uint8Array(w * d);
  const parts = new Map<number, Poly[]>();
  for (const p of pieces) {
    if (p.edge !== 'tiles') continue;
    const code = p.surface === 'yard' ? TILE.yard : TILE.stone;
    const [bx0, bz0, bx1, bz1] = boxOf(p.poly);
    const [x0, z0] = [Math.max(0, Math.floor(bx0)), Math.max(0, Math.floor(bz0))];
    const [x1, z1] = [Math.min(w, Math.ceil(bx1)), Math.min(d, Math.ceil(bz1))];
    if (x1 <= x0 || z1 <= z0) continue;
    const cuts = p.cuts ?? [];
    // Which of the tiles' corners are in the piece (and in none of its cuts), row by row.
    const cw = x1 - x0 + 1;
    const corner = new Uint8Array(cw * (z1 - z0 + 1));
    for (let z = z0; z <= z1; z++) {
      const row = (z - z0) * cw;
      spans(p.poly, z, x0, x1, (x) => (corner[row + x - x0] = 1));
      for (const c of cuts) spans(c, z, x0, x1, (x) => (corner[row + x - x0] = 0));
    }
    // (The signed distance at a corner, worked out only where a tile's corners differ: in the piece, positive.)
    const signed = new Map<number, number>();
    const at = (x: number, z: number) => {
      const k = (z - z0) * cw + x - x0;
      let v = signed.get(k);
      if (v === undefined) {
        v = toOutline(p.poly, x, z);
        for (const c of cuts) v = Math.min(v, toOutline(c, x, z));
        // (A corner on the outline takes the side its tile's corners were counted on, a hair from it.)
        v = Math.max(v, ON_EDGE);
        signed.set(k, (v = corner[k] ? v : -v));
      }
      return v;
    };
    // (The tiles whose middles are in a hole, marked once for the piece.)
    const bw = x1 - x0;
    const holes = new Uint8Array(bw * (z1 - z0));
    for (const r of p.holes) {
      for (let z = Math.max(z0, Math.ceil(r.z - 0.5)); z < Math.min(z1, Math.ceil(r.z + r.d - 0.5)); z++) {
        for (let x = Math.max(x0, Math.ceil(r.x - 0.5)); x < Math.min(x1, Math.ceil(r.x + r.w - 0.5)); x++) holes[(z - z0) * bw + x - x0] = 1;
      }
    }
    const holed = (x: number, z: number) => holes[(z - z0) * bw + x - x0] === 1;
    for (let z = z0; z < z1; z++) {
      for (let x = x0; x < x1; x++) {
        const k = (z - z0) * cw + x - x0;
        const n = corner[k] + corner[k + 1] + corner[k + cw] + corner[k + cw + 1];
        const i = z * w + x;
        if (!n || whole[i] === TILE.stone || (whole[i] && code === TILE.yard) || holed(x, z)) continue;
        if (n === 4) {
          whole[i] = codes[i] = code;
          parts.delete(i);
          continue;
        }
        if (whole[i]) continue;
        const poly = crossTile(x, z, [at(x, z), at(x + 1, z), at(x + 1, z + 1), at(x, z + 1)]);
        if (!poly) continue;
        // (Cut along a tile's own sides, it is whole after all.)
        if (area(poly) > 1 - SLIVER) {
          whole[i] = codes[i] = code;
          parts.delete(i);
          continue;
        }
        const list = parts.get(i);
        if (!list) parts.set(i, [poly]);
        else if (!list.some((q) => q.length === poly.length && q.every((c, k) => c[0] === poly[k][0] && c[1] === poly[k][1]))) list.push(poly);
        if (codes[i] !== TILE.stone) codes[i] = code;
      }
    }
  }
  if (roads.length) {
    const under = rasterRoads(roads, w, d, (r) => carriageway(r) / 2 - UNDER_ROAD);
    for (let i = 0; i < codes.length; i++) {
      if (!under[i]) continue;
      codes[i] = 0;
      parts.delete(i);
    }
  }
  return { codes, parts };
}

/**
 * Calls `each` with every whole `x` from `x0` to `x1` (both in) at which point (`x`, `z`) is inside `poly` (even-odd)
 * or on its outline.
 */
function spans(poly: readonly (readonly [number, number])[], z: number, x0: number, x1: number, each: (x: number) => void): void {
  const xs: number[] = [];
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const [[xi, zi], [xj, zj]] = [poly[i], poly[j]];
    if (zi > z !== zj > z) xs.push(xi + ((z - zi) * (xj - xi)) / (zj - zi));
    // (On a side along the row, or at a corner on it: on the outline.)
    if (zi === z && zj === z) for (let x = Math.max(x0, Math.ceil(Math.min(xi, xj))); x <= Math.min(x1, Math.max(xi, xj)); x++) each(x);
    else if (zi === z && Number.isInteger(xi) && xi >= x0 && xi <= x1) each(xi);
  }
  xs.sort((a, b) => a - b);
  for (let k = 0; k + 1 < xs.length; k += 2) for (let x = Math.max(x0, Math.ceil(xs[k])); x <= Math.min(x1, xs[k + 1]); x++) each(x);
}


/**
 * The part of tile (`x`, `z`) on the paved side of an edge (its corners' signed distances `v`, round from (`x`, `z`)
 * along x first, paved where over 0: a corner on the edge is not): its paved corners and where its sides cross the
 * edge, convex; null if it is a sliver.
 */
function crossTile(x: number, z: number, v: readonly number[]): Poly | null {
  const c: [number, number][] = [[x, z], [x + 1, z], [x + 1, z + 1], [x, z + 1]];
  const out: Poly = [];
  for (let k = 0; k < 4; k++) {
    const [a, b] = [v[k], v[(k + 1) % 4]];
    if (a > 0) out.push(c[k]);
    if (a > 0 !== b > 0) {
      const t = a / (a - b);
      const [p, q] = [c[k], c[(k + 1) % 4]];
      const e: [number, number] = [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t];
      if (!out.length || out[out.length - 1][0] !== e[0] || out[out.length - 1][1] !== e[1]) out.push(e);
    }
  }
  return out.length >= 3 && area(out) > SLIVER ? out : null;
}

/** The area of outline `poly` (m²). */
const area = (poly: Poly): number => Math.abs(signedArea2(poly)) / 2;

/** `poly` (convex) less what lies on the right of the line from `a` to `b` (seen from above, x right and z down). */
export function clipBy(poly: readonly (readonly [number, number])[], a: readonly [number, number], b: readonly [number, number]): Poly {
  const side = (p: readonly [number, number]) => (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
  const out: Poly = [];
  for (let k = 0; k < poly.length; k++) {
    const [p, q] = [poly[k], poly[(k + 1) % poly.length]];
    const [sp, sq] = [side(p), side(q)];
    if (sp >= 0) out.push([p[0], p[1]]);
    if (sp >= 0 !== sq >= 0) {
      const t = sp / (sp - sq);
      out.push([p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]);
    }
  }
  return out;
}

/**
 * `poly` (convex) cut into its parts in each of the triangles the ground tiles draw the land as (`streetGround.ts`
 * `terrainOf`: each `LAND_CELL` square split from its corner at (x + cell, z) to (x, z + cell)), so each part lies
 * flat on the ground under it.
 */
export function terrainPieces(poly: readonly (readonly [number, number])[]): Poly[] {
  if (poly.length < 3) return [];
  const [x0, z0, x1, z1] = boxOf(poly);
  const c = LAND_CELL;
  const out: Poly[] = [];
  for (let j = Math.floor(z0 / c); j * c < z1; j++) {
    for (let i = Math.floor(x0 / c); i * c < x1; i++) {
      const [a, b, d, e]: [number, number][] = [[i * c, j * c], [(i + 1) * c, j * c], [(i + 1) * c, (j + 1) * c], [i * c, (j + 1) * c]];
      // (The square's two triangles, each wound the same way: what is left of each of its sides is in it.)
      for (const tri of [[a, b, e], [b, d, e]] as [number, number][][]) {
        let part = clipBy(poly, tri[0], tri[1]);
        for (let k = 1; k < 3 && part.length >= 3; k++) part = clipBy(part, tri[k], tri[(k + 1) % 3]);
        if (part.length >= 3) out.push(part);
      }
    }
  }
  return out;
}

/** `poly` (convex) inside the rectangle `x0`..`x1` by `z0`..`z1` (fewer than three corners: none of it is). */
export function clipRect(poly: readonly (readonly [number, number])[], x0: number, z0: number, x1: number, z1: number): Poly {
  const c: [number, number][] = [[x0, z0], [x1, z0], [x1, z1], [x0, z1]];
  let out: Poly = poly.map(([x, z]) => [x, z]);
  for (let k = 0; k < 4 && out.length >= 3; k++) out = clipBy(out, c[k], c[(k + 1) % 4]);
  return out;
}

/** Where a side from `a` to `b` is cut: its ends, and each joint's two edges between them (`course` m apart; 0: none). */
function stops(a: number, b: number, course: number): number[] {
  const out = [a];
  if (course) {
    for (let k = Math.ceil((a - JOINT / 2) / course); k * course - JOINT / 2 < b; k++) {
      for (const e of [k * course - JOINT / 2, k * course + JOINT / 2]) if (e > a + 1e-6 && e < b - 1e-6) out.push(e);
    }
  }
  out.push(b);
  return out;
}

/** Whether `v` is in a joint between two courses `course` m apart. */
const inJoint = (v: number, course: number) => course > 0 && Math.abs(v - Math.round(v / course) * course) < JOINT / 2;

/**
 * A stretch of paving (`x0`..`x1` by `z0`..`z1`) cut at its joints (`course` m apart; 0: none): each cell
 * (x0, z0, x1, z1) and whether it is in a joint. The near ground lays these and the far map paints them.
 */
export function courseCells(x0: number, z0: number, x1: number, z1: number, course: number): { cell: [number, number, number, number]; joint: boolean }[] {
  const [xs, zs] = [stops(x0, x1, course), stops(z0, z1, course)];
  const out: { cell: [number, number, number, number]; joint: boolean }[] = [];
  for (let j = 0; j + 1 < zs.length; j++) {
    for (let i = 0; i + 1 < xs.length; i++) {
      out.push({ cell: [xs[i], zs[j], xs[i + 1], zs[j + 1]], joint: inJoint((xs[i] + xs[i + 1]) / 2, course) || inJoint((zs[j] + zs[j + 1]) / 2, course) });
    }
  }
  return out;
}

/**
 * Marks (1) the tiles of a `w` × `d` grid a convex outline `q` covers the middle of, less `holes`
 * (`any`: those it reaches into at all, near enough: a tile's middle within half its diagonal of it).
 */
export function fillConvex(out: Uint8Array, w: number, d: number, q: readonly [number, number][], holes: readonly Rect[] = [], any = false): void {
  const grow = any ? Math.SQRT1_2 : 0;
  const [bx0, bz0, bx1, bz1] = boxOf(q);
  const [x0, x1] = [Math.max(0, Math.floor(bx0 - grow)), Math.min(w - 1, Math.ceil(bx1 + grow))];
  const [z0, z1] = [Math.max(0, Math.floor(bz0 - grow)), Math.min(d - 1, Math.ceil(bz1 + grow))];
  const [cx, cz] = centroid(q);
  const holed = (x: number, z: number) => holes.some((r) => x >= r.x && x < r.x + r.w && z >= r.z && z < r.z + r.d);
  for (let z = z0; z <= z1; z++) {
    for (let x = x0; x <= x1; x++) {
      if (holed(x + 0.5, z + 0.5)) continue;
      let [px, pz] = [x + 0.5, z + 0.5];
      // (Grown: the tile's middle pulled towards the outline's by up to `grow`.)
      if (grow) {
        const l = Math.hypot(cx - px, cz - pz);
        if (l > 0) [px, pz] = [px + ((cx - px) / l) * Math.min(grow, l), pz + ((cz - pz) / l) * Math.min(grow, l)];
      }
      if (inConvex(q, px, pz)) out[z * w + x] = 1;
    }
  }
}

/** Whether (`x`, `z`) is inside convex outline `q`: on the same side of every edge, whichever way round it is wound. */
export function inConvex(q: readonly (readonly [number, number])[], x: number, z: number): boolean {
  let [pos, neg] = [false, false];
  for (let k = 0; k < q.length; k++) {
    const [a, b] = [q[k], q[(k + 1) % q.length]];
    const v = (b[0] - a[0]) * (z - a[1]) - (b[1] - a[1]) * (x - a[0]);
    if (v > 0) pos = true;
    else if (v < 0) neg = true;
  }
  return !(pos && neg);
}
