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
 * Paving is laid by whole tiles (a tile is paved when its middle is in a piece, less its holes):
 * the lawn's grass is hidden tile by tile too, so the paving and the grass meet on the same line
 * near and far. A car park's apron is turned to its road and is drawn by its own outline.
 */

import type { PavedPiece, RoadPlan, WorldStructure } from '../../core/protocol';
import { fillPoly } from '../../game/footprints';
import { rasterRoads } from '../../game/roads';
import { carriageway } from './junctions';

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
   * What each tile laid by tiles is paved with, over `w` × `d`: `TILE.stone`, `TILE.yard`, else 0
   * (the pieces laid by their own outlines are not in it). Near and far draw these tiles.
   */
  tiles(w: number, d: number): Uint8Array;
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
 * A bay: 3 m along its road by 6 deep; its lines (`LINE` m wide) run along its sides, 1.45 m either side
 * of its middle (two bays side by side share one), and across its far end, 2.75 m in from its middle.
 * A lone bay's apron is a little bigger than the bay, so its lines lie on asphalt.
 */
export const BAY = { w: 3, d: 6, side: 1.45, end: 2.75, half: 2.8 } as const;
export const LINE = 0.1;
const LONE = { w: 3.4, d: 6.2 } as const;
/** A tile wholly under a road's carriageway is the road's, not paved: its middle this far inside the kerb (half its diagonal). */
const UNDER_ROAD = Math.SQRT1_2;
/** How a piece's courses are by kind. */
const COURSES: Record<PavedPiece['kind'], number> = { centre: COURSE, square: COURSE, court: COURSE, yard: 0 };

/** Each meta's paving, for the lots, bays and car parks it was drawn with. */
const made = new WeakMap<object, { plots: unknown; parking: unknown; carParks: unknown; paving: TownPaving }>();
/** Nothing paved: a world without a meta (the build previews, older tests). */
const NONE: TownPaving = { pieces: [], bays: [], signs: [], mask: (w, d) => new Uint8Array(w * d), tiles: (w, d) => new Uint8Array(w * d) };

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
    const lines: BayDraw['lines'] = [at.seg(-BAY.side, -BAY.half, -BAY.side, BAY.half), at.seg(BAY.side, -BAY.half, BAY.side, BAY.half), at.seg(-BAY.w / 2, BAY.end, BAY.w / 2, BAY.end)];
    bays.push({ corners: at.rect(BAY.w, BAY.d), lines });
    sides.push({ park, a: lines[0][0], b: lines[0][1] }, { park, a: lines[1][0], b: lines[1][1] });
  });
  // Two bays side by side share the line between them: the two sides' lines as one, between where they were.
  for (let i = 0; i < sides.length; i++) {
    for (let j = i + 1; j < sides.length; j++) {
      const [s, t] = [sides[i], sides[j]];
      if (s.park !== t.park || Math.hypot(s.a[0] - t.a[0], s.a[1] - t.a[1]) > 0.3 || Math.hypot(s.b[0] - t.b[0], s.b[1] - t.b[1]) > 0.3) continue;
      const mid = (p: [number, number], q: [number, number]) => [(p[0] + q[0]) / 2, (p[1] + q[1]) / 2] as [number, number];
      [s.a, s.b] = [mid(s.a, t.a), mid(s.b, t.b)];
      [t.a, t.b] = [s.a, s.b];
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
  const tileMemo = new Map<string, Uint8Array>();
  const maskMemo = new Map<string, Uint8Array>();
  const tiles = (w: number, d: number): Uint8Array => {
    const key = `${w}x${d}`;
    let out = tileMemo.get(key);
    if (!out) tileMemo.set(key, (out = layTiles(pieces, roads, w, d)));
    return out;
  };
  const mask = (w: number, d: number, any = false): Uint8Array => {
    const key = `${w}x${d}${any ? '+' : ''}`;
    let out = maskMemo.get(key);
    if (out) return out;
    out = new Uint8Array(w * d);
    const laid = tiles(w, d);
    for (let i = 0; i < out.length; i++) if (laid[i]) out[i] = 1;
    for (const p of pieces) if (p.edge === 'exact') fillConvex(out, w, d, p.poly, p.holes, any);
    maskMemo.set(key, out);
    return out;
  };
  return { pieces, bays, signs, mask, tiles };
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

/** `poly` wound counter-clockwise seen from above (x right, z down the screen: its signed area x·z' − x'·z negative). */
function ccw(poly: [number, number][]): [number, number][] {
  let n = 0;
  for (let i = 0; i < poly.length; i++) {
    const [a, b] = [poly[i], poly[(i + 1) % poly.length]];
    n += a[0] * b[1] - b[0] * a[1];
  }
  return n > 0 ? poly.reverse() : poly;
}

/** A polygon's box: x0, z0, x1, z1. */
export function boxOf(poly: readonly (readonly [number, number])[]): [number, number, number, number] {
  let [x0, z0, x1, z1] = [Infinity, Infinity, -Infinity, -Infinity];
  for (const [x, z] of poly) [x0, z0, x1, z1] = [Math.min(x0, x), Math.min(z0, z), Math.max(x1, x), Math.max(z1, z)];
  return [x0, z0, x1, z1];
}

/**
 * The tiles the pieces laid by tiles pave (`TownPaving.tiles`): each piece's tiles less its holes and
 * cuts, stone over yard where two meet, and none wholly under a carriageway (the road is drawn there).
 */
function layTiles(pieces: readonly PavingDraw[], roads: readonly RoadPlan[], w: number, d: number): Uint8Array {
  const out = new Uint8Array(w * d);
  const mine = new Uint8Array(w * d);
  for (const p of pieces) {
    if (p.edge !== 'tiles') continue;
    const code = p.surface === 'yard' ? TILE.yard : TILE.stone;
    const marked: number[] = [];
    fillPoly(p.poly, w, d, (i) => {
      mine[i] = 1;
      marked.push(i);
    });
    for (const r of p.holes) for (let z = Math.max(0, Math.floor(r.z)); z < Math.min(d, Math.ceil(r.z + r.d)); z++) for (let x = Math.max(0, Math.floor(r.x)); x < Math.min(w, Math.ceil(r.x + r.w)); x++) mine[z * w + x] = 0;
    for (const c of p.cuts ?? []) fillPoly(c, w, d, (i) => (mine[i] = 0));
    for (const i of marked) {
      if (mine[i] && out[i] !== TILE.stone) out[i] = code;
      mine[i] = 0;
    }
  }
  if (roads.length) {
    const under = rasterRoads(roads, w, d, (r) => carriageway(r) / 2 - UNDER_ROAD);
    for (let i = 0; i < out.length; i++) if (under[i]) out[i] = 0;
  }
  return out;
}

/**
 * Marks (1) the tiles of a `w` × `d` grid a convex outline `q` covers the middle of, less `holes`
 * (`any`: those it reaches into at all, near enough: a tile's middle within half its diagonal of it).
 */
export function fillConvex(out: Uint8Array, w: number, d: number, q: readonly (readonly [number, number])[], holes: readonly Rect[] = [], any = false): void {
  const grow = any ? Math.SQRT1_2 : 0;
  const [bx0, bz0, bx1, bz1] = boxOf(q);
  const [x0, x1] = [Math.max(0, Math.floor(bx0 - grow)), Math.min(w - 1, Math.ceil(bx1 + grow))];
  const [z0, z1] = [Math.max(0, Math.floor(bz0 - grow)), Math.min(d - 1, Math.ceil(bz1 + grow))];
  const [cx, cz] = [q.reduce((a, p) => a + p[0], 0) / q.length, q.reduce((a, p) => a + p[1], 0) / q.length];
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
