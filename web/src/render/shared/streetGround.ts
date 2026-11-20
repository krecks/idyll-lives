/**
 * The town's paved ground near, as the street's chunks draw it (`street.ts` `TownStreet.chunk`):
 * for each stream chunk, the tiles `townPaving` paves, in cells of 2 m or less draped over the
 * ground (a tile along a piece's edge only as far as the edge, `TownPaving.parts`), the stone's
 * courses as joints between them; the car parks' aprons, their bays' lines just
 * over the asphalt, and a P sign by each car park. The viewed lot draws what is paved on it itself
 * (`pavingLayers`, `pc/PlayCanvasRenderer.ts`), as it does its paths; the menus' town draws the whole
 * town's at once (`pc/overview.ts`). `streetGroundKey` says what a chunk's share is drawn from, so a
 * chunk whose share is the same is not planned again.
 *
 * Each surface is the street's own (the pavement's slabs, the garden paths' paving, the road's
 * asphalt), toned by vertex so that it shows the colour the far map paints it (`PAVING_LOOK`): the
 * town looks the same paved from the play camera and from the town view.
 */

import { groundDepth, type WorldStructure } from '../../core/protocol';
import { valueNoise } from '../../game/noise';
import { CHUNK, chunkAt, chunkRect, hashNumbers, hashString } from './chunks';
import { LAND_CELL } from '../../game/land';
import { centroid } from '../../game/roads';
import type { BuiltSurface } from './builtSurface';
import { Geo, type V3 } from './geometry';
import { linearHex } from './math';
import { lotsFilled } from './nature';
import type { ChunkPlan, GroundLayer } from './street';
import { boxOf, clipBy, clipRect, COURSE, courseCells, LINE, PAVING_LOOK, TILE, townPaving, type BayDraw, type PavingDraw, type Poly, type Rect, type TownPaving } from './townPaving';

/** What the near paving reads (as `townPaving.ts` `PavingWorld`), and the world's size. */
export type StreetGroundWorld = Pick<WorldStructure, 'plots' | 'meta' | 'parking' | 'carParks' | 'width' | 'depth' | 'storeyDepth'>;

/** The P sign's model, one by each car park (`CarPark.sign`). */
export const PARKING_SIGN = 'model.street.parkingSign';

/**
 * How high the paving lies over the ground as the ground tiles draw it (m): under the road (`roads.ts`
 * `ROAD_Y`), which is drawn over its edge as the far map paints the roads over it; a car park's apron a
 * touch over the paving, and its bays' lines over the apron.
 */
export const PAVING_Y = 0.006;
const APRON_Y = 0.008;
const LINE_Y = 0.016;
/** The largest cell the paving is cut into (m), on a flat town. */
const CELL = 2;
/** How much lighter and darker the stone is in patches a few metres across (a share of its light). */
const VARY = 0.05;
/** The lines' paint: the plain finish's roughness in its alpha, as the road's markings (`roads.ts`). */
const PAINT_FINISH = 0.35;
const UP: V3 = [0, 1, 0];

/**
 * What each surface the paving is drawn with makes of a vertex colour (linear light, a channel each):
 * its texture's mean times its material's colour (the base manifest's `material.sidewalk`,
 * `material.path`, `material.road`). The vertex colours are the paving's colours through them
 * (`toner`); a texture whose mean moves is another number here.
 */
const SURFACE_TONE: Record<'sidewalk' | 'path' | 'road', V3> = {
  sidewalk: scaled(linear('#E4E0D8'), 0.7362),
  path: [0.4222, 0.3531, 0.2546],
  road: [0.1154, 0.1129, 0.103],
};

/** An sRGB colour's channels in linear light (as the far map's shader reads them: a power of 2.2, `toLinear`). */
function linear(hex: string): V3 {
  const { r, g, b } = linearHex(hex);
  return [r, g, b];
}

/** `v` times `k`. */
function scaled(v: V3, k: number): V3 {
  return [v[0] * k, v[1] * k, v[2] * k];
}

/**
 * A colour (sRGB hex) as the vertex colour that shows it on `surface` (no brighter than white): worked out once, and
 * then for `k` times its light (1: as it is) at each vertex.
 */
function toner(hex: string, surface: V3): (k?: number) => V3 {
  const c = linear(hex).map((v, i) => Math.pow(v / surface[i], 1 / 2.2));
  return (k = 1) => {
    const f = Math.pow(k, 1 / 2.2);
    return [Math.min(1, c[0] * f), Math.min(1, c[1] * f), Math.min(1, c[2] * f)];
  };
}

/** A layer's name, material and what its tiles and the joints between their courses are toned to. */
const LAYERS = {
  [TILE.stone]: { name: 'townPaving', material: 'material.sidewalk', tone: toner(PAVING_LOOK.stone, SURFACE_TONE.sidewalk), joint: toner(PAVING_LOOK.joint, SURFACE_TONE.sidewalk) },
  [TILE.yard]: { name: 'townYards', material: 'material.path', tone: toner(PAVING_LOOK.yard, SURFACE_TONE.path), joint: toner(PAVING_LOOK.joint, SURFACE_TONE.path) },
} as const;
/** A car park's apron, toned as the road's asphalt shows it. */
const ASPHALT = toner(PAVING_LOOK.asphalt, SURFACE_TONE.road)();

/** A chunk's share of what is drawn by its own outline: the car parks' aprons, the bays and the P signs whose middles are in it. */
interface Share {
  aprons: PavingDraw[];
  bays: BayDraw[];
  signs: [number, number, number][];
}

/** Each paving's shares, by chunk. */
const sharesOf = new WeakMap<TownPaving, Map<number, Share>>();

function shares(paving: TownPaving): Map<number, Share> {
  let out = sharesOf.get(paving);
  if (out) return out;
  const by = (out = new Map<number, Share>());
  const of = (id: number) => {
    let s = by.get(id);
    if (!s) by.set(id, (s = { aprons: [], bays: [], signs: [] }));
    return s;
  };
  for (const p of paving.pieces) if (p.edge === 'exact') of(chunkAt(...centroid(p.poly))).aprons.push(p);
  for (const b of paving.bays) of(chunkAt(...centroid(b.corners))).bays.push(b);
  for (const s of paving.signs) of(chunkAt(s[0], s[1])).signs.push(s);
  sharesOf.set(paving, out);
  return out;
}

/** The ground's tiles across and down (`groundDepth`: the ground storey's rows). */
function groundOf(world: StreetGroundWorld): [number, number] {
  return [world.width, groundDepth(world)];
}

/**
 * The ground as the ground tiles draw it on a land the town follows (`pc/nature.ts`): the graded land with the
 * lots filled under their pads (`lotsFilled`), a sample every `LAND_CELL` m, each square of four drawn as two
 * triangles split from its corner at (x + cell, z) to (x, z + cell). The paving lies on it piece by piece, each
 * piece within one of those triangles, so it is as flat as the ground under it: laid on the land between its
 * corners (`BuiltSurface.landAt`), the ground came through it in patches where the land creases.
 */
interface Terrain {
  at(x: number, z: number): number;
}

/** Each surface's ground as drawn, for the lots it was filled for and the moves it had made. */
const terrains = new WeakMap<BuiltSurface, { moves: number; plots: number; terrain: Terrain }>();

function terrainOf(surface: BuiltSurface, plots: number): Terrain {
  const known = terrains.get(surface);
  if (known && known.moves === surface.moves && known.plots === plots) return known.terrain;
  const field = lotsFilled(surface, plots);
  const { cols, rows } = surface;
  const h = (i: number, j: number) => field[Math.min(rows - 1, Math.max(0, j)) * cols + Math.min(cols - 1, Math.max(0, i))];
  const terrain: Terrain = {
    at(x, z) {
      const [fx, fz] = [x / LAND_CELL, z / LAND_CELL];
      const [i, j] = [Math.floor(fx), Math.floor(fz)];
      const [u, v] = [fx - i, fz - j];
      if (u + v <= 1) return h(i, j) + (h(i + 1, j) - h(i, j)) * u + (h(i, j + 1) - h(i, j)) * v;
      return h(i + 1, j + 1) + (h(i, j + 1) - h(i + 1, j + 1)) * (1 - u) + (h(i + 1, j) - h(i + 1, j + 1)) * (1 - v);
    },
  };
  terrains.set(surface, { moves: surface.moves, plots, terrain });
  return terrain;
}

/**
 * A flat convex piece (`poly`) of the paving into `g`, `y` over the ground, its vertices coloured by `tint`: on
 * the ground as drawn (`terrain`), cut into the parts in each of its triangles; on a flat town, as it is.
 */
function lay(g: Geo, poly: [number, number][], y: number, tint: (x: number, z: number) => V3, terrain: Terrain | null): void {
  if (poly.length < 3) return;
  if (!terrain) {
    g.poly(poly.map(([x, z]) => [x, y, z] as V3), UP, undefined, undefined, poly.map(([x, z]) => tint(x, z)));
    return;
  }
  const [x0, z0, x1, z1] = boxOf(poly);
  const c = LAND_CELL;
  for (let j = Math.floor(z0 / c); j * c < z1; j++) {
    for (let i = Math.floor(x0 / c); i * c < x1; i++) {
      const [a, b, d, e]: [number, number][] = [[i * c, j * c], [(i + 1) * c, j * c], [(i + 1) * c, (j + 1) * c], [i * c, (j + 1) * c]];
      // (The square's two triangles, each wound the same way: what is left of each of its sides is in it.)
      for (const tri of [[a, b, e], [b, d, e]] as [number, number][][]) {
        let part = poly;
        for (let k = 0; k < 3 && part.length >= 3; k++) part = clipBy(part, tri[k], tri[(k + 1) % 3]);
        if (part.length < 3) continue;
        g.poly(part.map(([x, z]) => [x, terrain.at(x, z) + y, z] as V3), UP, undefined, undefined, part.map(([x, z]) => tint(x, z)));
      }
    }
  }
}

/**
 * The layers of the paved ground in `area` (none: the whole town): its tiles (those in `skip`, the
 * viewed lot's, left to it), and unless `tilesOnly` the car parks' aprons and the bays' lines whose
 * middles are in it (`share`: those, as a chunk's share), on the ground as the ground tiles draw it where
 * the town follows its land (`surface`; none: at 0).
 */
export function pavingLayers(world: StreetGroundWorld, area: Rect | null, options: { surface?: BuiltSurface | null; skip?: Rect | null; tilesOnly?: boolean; share?: Share } = {}): GroundLayer[] {
  const paving = townPaving(world);
  const [w, d] = groundOf(world);
  const tiles = paving.tiles(w, d);
  const parts = paving.parts(w, d);
  const { skip } = options;
  const terrain = options.surface ? terrainOf(options.surface, world.plots.length) : null;
  const a = area ?? { x: 0, z: 0, w, d };
  const [x0, z0, x1, z1] = [Math.max(0, Math.floor(a.x)), Math.max(0, Math.floor(a.z)), Math.min(w, Math.ceil(a.x + a.w)), Math.min(d, Math.ceil(a.z + a.d))];
  const code = (x: number, z: number) => (x < x0 || z < z0 || x >= x1 || z >= z1 || (skip && x >= skip.x && x < skip.x + skip.w && z >= skip.z && z < skip.z + skip.d) ? 0 : tiles[z * w + x]);
  const geos = new Map<number, Geo>();
  const cell = (x: number, z: number, s: number, t: number, c: number) => {
    let g = geos.get(c);
    if (!g) geos.set(c, (g = new Geo()));
    // (A tile laid in part: the part of it paved, `TownPaving.parts`.)
    layCell(g, x, z, s, t, c, terrain, s === 1 ? parts.get(z * w + x) : undefined);
  };
  // In 2 m cells where all four of its tiles are paved alike and whole, else a tile at a time.
  const whole = (x: number, z: number) => !parts.has(z * w + x);
  for (let z = z0 - (z0 & 1); z < z1; z += CELL) {
    for (let x = x0 - (x0 & 1); x < x1; x += CELL) {
      const cs = [code(x, z), code(x + 1, z), code(x, z + 1), code(x + 1, z + 1)];
      if (cs[0] && cs.every((c) => c === cs[0]) && whole(x, z) && whole(x + 1, z) && whole(x, z + 1) && whole(x + 1, z + 1)) cell(x, z, 2, 2, cs[0]);
      else cs.forEach((c, k) => c && cell(x + (k & 1), z + (k >> 1), 1, 1, c));
    }
  }
  const out: GroundLayer[] = [];
  for (const [c, g] of geos) {
    const L = LAYERS[c as keyof typeof LAYERS];
    out.push({ name: L.name, material: L.material, geo: g, toned: true });
  }
  if (options.tilesOnly) return out;
  const within = (x: number, z: number) => x >= a.x && z >= a.z && x < a.x + a.w && z < a.z + a.d;
  const share = options.share ?? { aprons: paving.pieces.filter((p) => p.edge === 'exact' && within(...centroid(p.poly))), bays: paving.bays.filter((b) => within(...centroid(b.corners))) };
  const aprons = new Geo();
  for (const p of share.aprons) if (p.surface === 'asphalt') lay(aprons, p.poly.map(([x, z]) => [x, z]), APRON_Y, () => ASPHALT, terrain);
  if (!aprons.empty) out.push({ name: 'carParks', material: 'material.road', geo: aprons, toned: true });
  const lines = new Geo().color(PAVING_LOOK.line, PAINT_FINISH);
  const paint = (): V3 => [lines.r, lines.g, lines.b];
  for (const b of share.bays) {
    for (const [p, q] of b.lines) {
      const l = Math.hypot(q[0] - p[0], q[1] - p[1]);
      if (l < 1e-6) continue;
      const [nx, nz] = [(-(q[1] - p[1]) / l) * (LINE / 2), ((q[0] - p[0]) / l) * (LINE / 2)];
      lay(lines, [[p[0] + nx, p[1] + nz], [q[0] + nx, q[1] + nz], [q[0] - nx, q[1] - nz], [p[0] - nx, p[1] - nz]], LINE_Y, paint, terrain);
    }
  }
  if (!lines.empty) out.push({ name: 'bayLines', material: 'paint', geo: lines, paint: true });
  return out;
}

/**
 * A paved cell (`s` × `t` m from (`x`, `z`)) of tiles paved with `c` into `g`: the stone cut at its joints, each part
 * toned; a tile laid in part (`part`), only that part of it.
 */
function layCell(g: Geo, x: number, z: number, s: number, t: number, c: number, terrain: Terrain | null, part?: readonly Poly[]): void {
  const L = LAYERS[c as keyof typeof LAYERS];
  for (const { cell: [a0, b0, a1, b1], joint } of courseCells(x, z, x + s, z + t, c === TILE.stone ? COURSE : 0)) {
    const tone = joint ? L.joint : L.tone;
    const tint = (u: number, v: number) => tone(1 + (valueNoise(u / 2.3, v / 2.3, 977) - 0.5) * 2 * VARY);
    if (!part) lay(g, [[a0, b0], [a1, b0], [a1, b1], [a0, b1]], PAVING_Y, tint, terrain);
    else for (const q of part) lay(g, clipRect(q, a0, b0, a1, b1), PAVING_Y, tint, terrain);
  }
}

/** Each paving's chunk keys (`streetGroundKey` without the viewed lot), by chunk. */
const keysOf = new WeakMap<TownPaving, Map<number, string>>();

/** The chunks the paved ground has a share in (the street plans them, `TownStreet.chunkIds`). */
export function streetGroundChunks(world: StreetGroundWorld): number[] {
  const paving = townPaving(world);
  const [w, d] = groundOf(world);
  const tiles = paving.tiles(w, d);
  const ids = new Set<number>();
  // (A chunk with a paved tile: its rows looked along a tile at a time.)
  for (let z = 0; z < d; z++) {
    const row = z * w;
    for (let x = 0; x < w; x++) {
      if (!tiles[row + x]) continue;
      ids.add(chunkAt(x, z));
      x = (Math.floor(x / CHUNK) + 1) * CHUNK - 1;
    }
  }
  for (const id of shares(paving).keys()) ids.add(id);
  return [...ids];
}

/**
 * What chunk `id`'s share of the paved ground is drawn from, as a key ('': none): its tiles, aprons,
 * bays and signs; with the viewed lot `focus` (whose own paving it draws) where that lot is in it.
 * `layers`: the ground's layers as well as the signs (the game's street; the menus' town draws its
 * ground whole, `pc/overview.ts`).
 */
export function streetGroundKey(world: StreetGroundWorld, id: number, focus = -1, layers = true): string {
  const paving = townPaving(world);
  let keys = keysOf.get(paving);
  if (!keys) keysOf.set(paving, (keys = new Map()));
  let key = keys.get(id);
  if (key === undefined) {
    const [w, d] = groundOf(world);
    const tiles = paving.tiles(w, d);
    const parts = paving.parts(w, d);
    const r = chunkRect(id);
    const codes: number[] = [];
    let any = false;
    for (let z = Math.max(0, r.z); z < Math.min(d, r.z + r.d); z++) {
      for (let x = Math.max(0, r.x); x < Math.min(w, r.x + r.w); x++) {
        const c = tiles[z * w + x];
        codes.push(c);
        any ||= c > 0;
        // (A tile laid in part: where its edge runs.)
        for (const q of parts.get(z * w + x) ?? []) codes.push(-1, ...q.flat());
      }
    }
    const share = shares(paving).get(id);
    const aprons = share?.aprons.flatMap((p) => p.poly.flat()) ?? [];
    const bays = share?.bays.flatMap((b) => b.lines.flat(2)) ?? [];
    const signs = share?.signs.flat() ?? [];
    const tilesKey = any ? `t${hashNumbers(codes)}` : '';
    const rest = aprons.length || bays.length ? `a${hashNumbers([...aprons, -1, ...bays])}` : '';
    key = tilesKey || rest || signs.length ? `${tilesKey}|${rest}|${signs.length ? hashNumbers(signs) : ''}` : '';
    keys.set(id, key);
  }
  if (!key) return '';
  if (!layers) {
    const signs = key.split('|')[2];
    return signs ? `s${signs}` : '';
  }
  const lot = focus >= 0 ? world.plots[focus] : undefined;
  if (!lot || !key.startsWith('t')) return key;
  const r = chunkRect(id);
  const meets = lot.x < r.x + r.w && lot.x + lot.w > r.x && lot.z < r.z + r.d && lot.z + lot.d > r.z;
  return meets ? `${key}|f${hashString(`${lot.x},${lot.z},${lot.w},${lot.d}`)}` : key;
}

/**
 * Adds chunk `id`'s share of the paved ground to its plan: its P signs (`ChunkPlan.batches`) and,
 * with `layers`, its layers (`ChunkPlan.ground`; the viewed lot `focus`'s own paving left to it),
 * on the ground as the ground tiles draw it where the town follows its land (`surface`; none: at 0).
 */
export function streetGround(plan: ChunkPlan, world: StreetGroundWorld, id: number, surface?: BuiltSurface | null, focus = -1, layers = true): void {
  if (!streetGroundKey(world, id, focus, layers)) return;
  const share = shares(townPaving(world)).get(id);
  const signs = share?.signs ?? [];
  if (signs.length) {
    let list = plan.batches.get(PARKING_SIGN);
    if (!list) plan.batches.set(PARKING_SIGN, (list = []));
    const terrain = surface ? terrainOf(surface, world.plots.length) : null;
    for (const [x, z, yaw] of signs) {
      const y = terrain ? terrain.at(x, z) : 0;
      list.push(y ? [x, z, 1, 1, yaw, y] : [x, z, 1, 1, yaw]);
    }
  }
  if (!layers) return;
  const lot = focus >= 0 ? world.plots[focus] : undefined;
  plan.ground.push(...pavingLayers(world, chunkRect(id), { surface, skip: lot ?? null, share: share ?? { aprons: [], bays: [], signs: [] } }));
}

