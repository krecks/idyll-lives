/**
 * Step 2's painted uses over the town (`Renderer.setTownUses`, `game/uses.ts`): each zoned `USE_CELL` m cell filled
 * in its use's colour, half see-through, with a stronger band round the edge of each patch of one use, and the cells
 * too far from a street to be built on (`hatched`) striped across. Draped over the land `LIFT` m up, unlit and clear
 * of the fog, drawn before the other blended draws (step 2's ghosts and the brush's cursor lie over it).
 *
 * The world's cells are cut into square tiles of `TILE` cells, a mesh each. A row's cells of one use are drawn as
 * one strip (a vertex pair at each cell's edge, so it follows the land), and a new layer is compared with the last:
 * only the tiles that hold a changed cell (or a neighbour of one, whose edge band may come or go) are made again, so
 * a brush's stamp costs a tile or four, not the world.
 *
 * From far out (the whole world's shot) the edge bands are kept no thinner on screen than `MIN_EDGE_PX`, by the
 * scale the camera gives (`setScale`), in steps, so a zoom draws the overlay again only now and then.
 */

import * as pc from './engine';
import { GAME_FRONT_FACE } from './materials';
import { USE_CELL, USE_COLORS, unpackUses, useOfCode, type UseLayer } from '../../game/uses';

/** How high over the ground the overlay hangs (m): under step 2's ghosts (0.18). */
const LIFT = 0.12;
/** A tile's side, in cells (256 m). */
export const TILE = 64;
/** A cell's fill and an edge band's strength (alpha). */
const FILL = 0.36;
const EDGE = 0.9;
/** A cell too far from a street: its fill fainter, and its stripes (darker) this strong. */
const HATCH_FILL = 0.16;
const STRIPE = 0.78;
/** How dark a stripe is next to its use's colour (a factor on its channels). */
const STRIPE_SHADE = 0.72;
/** A stripe's reach either side of a cell's diagonal, along a side (m): stripes 1.4 m wide, 2.8 m apart. */
const STRIPE_HALF = 1;
/** An edge band's width (m), at least, and the fewest CSS px it is on screen; no wider than this share of a cell. */
const EDGE_W = 0.45;
const MIN_EDGE_PX = 1.6;
const EDGE_MAX = 0.4;
/**
 * The scale (m a CSS px) goes in steps of this ratio, rounded up, only between the scales where the edge bands are
 * `EDGE_W` and `EDGE_MAX` of a cell wide (nearer or further out they are drawn the same: no need to draw them again).
 */
const SCALE_STEP = Math.SQRT2;
const SCALE_LEAST = EDGE_W / MIN_EDGE_PX;
const SCALE_MOST = (USE_CELL * EDGE_MAX) / MIN_EDGE_PX;

/** Where the overlay stands: the ground (m) at a town point. */
export type Drape = (x: number, z: number) => number;

/** A rectangle of cells: columns `i0` up to (not with) `i1`, rows `j0` up to `j1`. */
export interface CellRect {
  i0: number;
  j0: number;
  i1: number;
  j1: number;
}

/** A row's run of one use: `n` cells from column `i`, with its `code`, and whether they are hatched. */
export interface UseRun {
  i: number;
  n: number;
  code: number;
  hatched: boolean;
}

/**
 * The cells that differ between the last codes and hatching (`before`, `hadHatch`; null: none drawn) and the new
 * ones, as the smallest rectangle round them; null where none do. The grids are `cols` wide and as long.
 */
export function dirtyRect(before: Uint8Array | null, codes: Uint8Array, hadHatch: Uint8Array | null, hatch: Uint8Array | null, cols: number): CellRect | null {
  const n = codes.length;
  const rows = cols > 0 ? Math.ceil(n / cols) : 0;
  if (!before || before.length !== n) return n ? { i0: 0, j0: 0, i1: cols, j1: rows } : null;
  let [i0, j0, i1, j1] = [Infinity, Infinity, -Infinity, -Infinity];
  for (let k = 0; k < n; k++) {
    const changed = before[k] !== codes[k] || (codes[k] !== 0 && (hadHatch?.[k] ?? 0) !== (hatch?.[k] ?? 0));
    if (!changed) continue;
    const [i, j] = [k % cols, (k / cols) | 0];
    if (i < i0) i0 = i;
    if (i >= i1) i1 = i + 1;
    if (j < j0) j0 = j;
    j1 = j + 1;
  }
  return i0 === Infinity ? null : { i0, j0, i1, j1 };
}

/**
 * Row `j`'s runs of one use (none left out) from column `i0` up to `i1`, a new run where the use or the hatching
 * changes. Into `out` (emptied first).
 */
export function rowRuns(codes: Uint8Array, hatch: Uint8Array | null, cols: number, j: number, i0: number, i1: number, out: UseRun[] = []): UseRun[] {
  out.length = 0;
  const base = j * cols;
  let run: UseRun | null = null;
  for (let i = i0; i < i1; i++) {
    const code = codes[base + i];
    const hatched = code !== 0 && !!hatch?.[base + i];
    if (run && run.code === code && run.hatched === hatched) {
      run.n++;
      continue;
    }
    run = code !== 0 && useOfCode(code) ? { i, n: 1, code, hatched } : null;
    if (run) out.push(run);
  }
  return out;
}

/** The tiles a rectangle of cells touches, for a grid `cols` × `rows` cells: their indices (row-major in tiles). */
export function tilesOf(r: CellRect, cols: number, rows: number): number[] {
  const across = Math.ceil(cols / TILE);
  const out: number[] = [];
  const [t0, u0] = [Math.max(0, Math.floor(r.i0 / TILE)), Math.max(0, Math.floor(r.j0 / TILE))];
  const [t1, u1] = [Math.min(across, Math.ceil(Math.min(cols, r.i1) / TILE)), Math.min(Math.ceil(rows / TILE), Math.ceil(Math.min(rows, r.j1) / TILE))];
  for (let u = u0; u < u1; u++) for (let t = t0; t < t1; t++) out.push(u * across + t);
  return out;
}

/** Each use's colour (sRGB bytes) by code, and its stripes' darker one. */
const RGB: (readonly number[])[] = [[0, 0, 0]];
const SHADE: (readonly number[])[] = [[0, 0, 0]];
for (let code = 1; useOfCode(code); code++) {
  const hex = USE_COLORS[useOfCode(code)!];
  const rgb = [1, 3, 5].map((k) => parseInt(hex.slice(k, k + 2), 16));
  RGB.push(rgb);
  SHADE.push(rgb.map((c) => Math.round(c * STRIPE_SHADE)));
}

/** A growing vertex and index buffer, written from the start for each tile. */
class Buffers {
  pos = new Float32Array(3 * 8192);
  col = new Uint8Array(4 * 8192);
  idx = new Uint32Array(6 * 8192);
  nv = 0;
  ni = 0;

  reset(): void {
    this.nv = this.ni = 0;
  }

  /** Makes room for `v` more vertices and `i` more indices. */
  reserve(v: number, i: number): void {
    if ((this.nv + v) * 3 > this.pos.length) {
      const size = Math.max(this.pos.length * 2, (this.nv + v) * 3);
      const [pos, col] = [new Float32Array(size), new Uint8Array((size / 3) * 4)];
      pos.set(this.pos);
      col.set(this.col);
      [this.pos, this.col] = [pos, col];
    }
    if (this.ni + i > this.idx.length) {
      const idx = new Uint32Array(Math.max(this.idx.length * 2, this.ni + i));
      idx.set(this.idx);
      this.idx = idx;
    }
  }

  vertex(x: number, y: number, z: number, rgb: readonly number[], alpha: number): number {
    const v = this.nv++;
    this.pos[v * 3] = x;
    this.pos[v * 3 + 1] = y;
    this.pos[v * 3 + 2] = z;
    this.col[v * 4] = rgb[0];
    this.col[v * 4 + 1] = rgb[1];
    this.col[v * 4 + 2] = rgb[2];
    this.col[v * 4 + 3] = Math.round(alpha * 255);
    return v;
  }

  tri(a: number, b: number, c: number): void {
    const i = this.ni;
    this.idx[i] = a;
    this.idx[i + 1] = b;
    this.idx[i + 2] = c;
    this.ni += 3;
  }
}

/** A tile's draw: its mesh, and the vertices its buffers were made for (past 65,535 they are made again, wider). */
interface Tile {
  entity: pc.Entity;
  mesh: pc.Mesh;
  room: number;
}

export class UseOverlayDraw {
  private readonly root: pc.Entity;
  private readonly material: pc.StandardMaterial;
  private readonly tiles = new Map<number, Tile>();
  private readonly buf = new Buffers();
  private readonly runs: UseRun[] = [];
  /** What is drawn: the codes and hatching (copies), the grid's size, the land's key and the scale, and the layer. */
  private codes: Uint8Array | null = null;
  private hatch: Uint8Array | null = null;
  private cols = 0;
  private rows = 0;
  private land: unknown = null;
  private drawnScale = -1;
  private layerId = -1;
  /** M a CSS px where the camera looks, in `SCALE_STEP` steps, within `SCALE_LEAST`–`SCALE_MOST` (0: not given yet). */
  private scale = 0;
  /** Tiles made again by the last `set` (for the benches and the tests). */
  remade = 0;

  constructor(
    private readonly app: pc.AppBase,
    parent: pc.Entity,
  ) {
    const mat = (this.material = new pc.StandardMaterial());
    mat.name = 'useOverlay';
    mat.frontFace = GAME_FRONT_FACE;
    mat.useLighting = false;
    mat.useFog = false;
    mat.useSkybox = false;
    mat.diffuse.set(0, 0, 0);
    mat.emissive.set(1, 1, 1);
    mat.emissiveVertexColor = true;
    // (The colours are sRGB: linearised.)
    (mat as pc.StandardMaterial & { vertexColorGamma: boolean }).vertexColorGamma = true;
    mat.opacityVertexColor = true;
    mat.opacityVertexColorChannel = 'a';
    mat.blendType = pc.BLEND_NORMAL;
    mat.depthWrite = false;
    mat.cull = pc.CULLFACE_NONE;
    // (In front of the ground it lies on, as the ghosts are.)
    mat.slopeDepthBias = -4;
    mat.depthBias = -2;
    mat.update();
    this.root = new pc.Entity('useOverlay', app);
    parent.addChild(this.root);
  }

  /**
   * How many m a CSS px is where the camera looks. True if that crossed a step: the overlay is to be drawn again
   * (`set`, the same layer: every tile) for its edge bands' least width on screen.
   */
  setScale(metresPerPx: number): boolean {
    const m = Math.min(SCALE_MOST, Math.max(SCALE_LEAST, metresPerPx));
    const scale = SCALE_STEP ** Math.ceil(Math.log(m) / Math.log(SCALE_STEP) - 1e-9);
    if (scale === this.scale) return false;
    this.scale = scale;
    return true;
  }

  /**
   * Draws `uses` (null: none) with `hatched` (a byte per cell; null: none) draped over `drape`, on `layer`. `land`:
   * anything that changes when the land under it does (every tile is draped again then). Only the tiles with a cell
   * changed since the last call are made again.
   */
  set(uses: UseLayer | null, hatched: Uint8Array | null | undefined, drape: Drape, layer: number, land: unknown): void {
    this.remade = 0;
    if (!uses || uses.cols <= 0 || uses.rows <= 0) {
      this.clear();
      return;
    }
    const codes = unpackUses(uses);
    const hatch = hatched && hatched.length === codes.length ? hatched : null;
    const { cols, rows } = uses;
    const whole = cols !== this.cols || rows !== this.rows || land !== this.land || this.scale !== this.drawnScale || layer !== this.layerId;
    const rect = whole ? { i0: 0, j0: 0, i1: cols, j1: rows } : dirtyRect(this.codes, codes, this.hatch, hatch, cols);
    if (whole) {
      for (const t of this.tiles.values()) t.entity.enabled = false;
      [this.cols, this.rows, this.land, this.drawnScale, this.layerId] = [cols, rows, land, this.scale, layer];
    }
    this.codes = codes.slice();
    this.hatch = hatch ? hatch.slice() : null;
    if (!rect) return;
    // (A neighbour of a changed cell may gain or lose its edge band.)
    const grown = { i0: rect.i0 - 1, j0: rect.j0 - 1, i1: rect.i1 + 1, j1: rect.j1 + 1 };
    for (const t of tilesOf(grown, cols, rows)) this.drawTile(t, drape, layer);
  }

  /** Nothing drawn (the game's world shown, or the uses let go); the next `set` draws every tile. */
  clear(): void {
    for (const t of this.tiles.values()) t.entity.enabled = false;
    this.codes = this.hatch = null;
    this.cols = this.rows = 0;
  }

  destroy(): void {
    for (const t of this.tiles.values()) t.mesh.destroy();
    this.tiles.clear();
    this.root.destroy();
    this.material.destroy();
  }

  /** Tile `t` made again from the codes drawn (`this.codes`). */
  private drawTile(t: number, drape: Drape, layer: number): void {
    const codes = this.codes!;
    const hatch = this.hatch;
    const { cols, rows, buf } = this;
    const across = Math.ceil(cols / TILE);
    const [i0, j0] = [(t % across) * TILE, Math.floor(t / across) * TILE];
    const [i1, j1] = [Math.min(cols, i0 + TILE), Math.min(rows, j0 + TILE)];
    const edge = Math.min(USE_CELL * EDGE_MAX, Math.max(EDGE_W, MIN_EDGE_PX * this.scale));
    buf.reset();
    const at = (i: number, j: number) => (i < 0 || j < 0 || i >= cols || j >= rows ? 0 : codes[j * cols + i]);
    for (let j = j0; j < j1; j++) {
      const z = j * USE_CELL;
      for (const run of rowRuns(codes, hatch, cols, j, i0, i1, this.runs)) {
        const rgb = RGB[run.code];
        const x0 = run.i * USE_CELL;
        const x1 = (run.i + run.n) * USE_CELL;
        this.strip(x0, x1, run.n, z, z + USE_CELL, rgb, run.hatched ? HATCH_FILL : FILL, drape);
        if (run.hatched) for (let k = 0; k < run.n; k++) this.stripes((run.i + k) * USE_CELL, z, SHADE[run.code], drape);
        // Edge bands: along the run's top and bottom where the row beyond has another use, and at its ends where the
        // cell beside it has (a run cut short by the tile's side or by its hatching goes on with the same use).
        for (const [dj, za, zb] of [
          [-1, z, z + edge],
          [1, z + USE_CELL - edge, z + USE_CELL],
        ] as const) {
          let from = -1;
          for (let k = 0; k <= run.n; k++) {
            const open = k < run.n && at(run.i + k, j + dj) !== run.code;
            if (open && from < 0) from = k;
            if (!open && from >= 0) {
              this.strip((run.i + from) * USE_CELL, (run.i + k) * USE_CELL, k - from, za, zb, rgb, EDGE, drape);
              from = -1;
            }
          }
        }
        if (at(run.i - 1, j) !== run.code) this.strip(x0, x0 + edge, 1, z, z + USE_CELL, rgb, EDGE, drape);
        if (at(run.i + run.n, j) !== run.code) this.strip(x1 - edge, x1, 1, z, z + USE_CELL, rgb, EDGE, drape);
      }
    }
    this.upload(t, layer);
  }

  /** A band from `x0` to `x1` m (in `pieces` pieces, so it follows the land) and from `za` to `zb` m, draped. */
  private strip(x0: number, x1: number, pieces: number, za: number, zb: number, rgb: readonly number[], alpha: number, drape: Drape): void {
    const buf = this.buf;
    buf.reserve((pieces + 1) * 2, pieces * 6);
    const first = buf.nv;
    for (let k = 0; k <= pieces; k++) {
      const x = x0 + ((x1 - x0) * k) / pieces;
      buf.vertex(x, drape(x, za) + LIFT, za, rgb, alpha);
      buf.vertex(x, drape(x, zb) + LIFT, zb, rgb, alpha);
    }
    for (let k = 0; k < pieces; k++) {
      const a = first + k * 2;
      buf.tri(a, a + 2, a + 3);
      buf.tri(a, a + 3, a + 1);
    }
  }

  /**
   * A hatched cell's stripes, from (`x`, `z`): the band along its diagonal (x − z the same as at its corner, a
   * hexagon in the cell) and the corners of the bands either side, so they run on unbroken across hatched cells.
   */
  private stripes(x: number, z: number, rgb: readonly number[], drape: Drape): void {
    const buf = this.buf;
    const [c, a] = [USE_CELL, STRIPE_HALF];
    buf.reserve(12, 18);
    const v = (u: number, w: number) => buf.vertex(x + u, drape(x + u, z + w) + LIFT, z + w, rgb, STRIPE);
    const hex = [v(0, 0), v(a, 0), v(c, c - a), v(c, c), v(c - a, c), v(0, a)];
    for (let k = 1; k < 5; k++) buf.tri(hex[0], hex[k], hex[k + 1]);
    buf.tri(v(c - a, 0), v(c, 0), v(c, a));
    buf.tri(v(0, c - a), v(a, c), v(0, c));
  }

  /** The buffers into tile `t`'s mesh (made the first time), on `layer`; an empty tile is put out. */
  private upload(t: number, layer: number): void {
    const buf = this.buf;
    let tile = this.tiles.get(t);
    if (!buf.ni) {
      if (tile) tile.entity.enabled = false;
      return;
    }
    if (!tile) {
      const mesh = new pc.Mesh(this.app.graphicsDevice);
      // (Written again as the uses change, in place while it fits.)
      mesh.clear(true, true, 8192, 6 * 8192);
      // (Its buffers made now, with their colours: a mesh instance reads what its mesh's vertices carry when it is made.)
      mesh.setPositions(new Float32Array(3), 3, 1);
      mesh.setColors32(new Uint8Array(4), 1);
      mesh.setIndices(new Uint32Array(3), 3);
      mesh.update(pc.PRIMITIVE_TRIANGLES, false);
      const mi = new pc.MeshInstance(mesh, this.material);
      // (Its box is never made: the overlay is never left out by an old one.)
      mi.cull = false;
      // Before the other blended draws (a higher bucket goes first, `meshes.ts` `drawAfter`): the ghosts over it.
      mi.drawBucket = 128;
      const entity = new pc.Entity(`useOverlay.${t}`, this.app);
      entity.addComponent('render', { meshInstances: [mi], castShadows: false, receiveShadows: false });
      this.root.addChild(entity);
      tile = { entity, mesh, room: 8192 };
      this.tiles.set(t, tile);
    }
    const mesh = tile.mesh;
    // (Past 65,535 vertices its indices need 32 bits: its buffers made again, larger.)
    if (buf.nv > tile.room) {
      tile.room = buf.nv * 2;
      mesh.clear(true, true, tile.room, buf.ni * 2);
    }
    mesh.setPositions(buf.pos, 3, buf.nv);
    mesh.setColors32(buf.col, buf.nv);
    mesh.setIndices(buf.idx, buf.ni);
    mesh.update(pc.PRIMITIVE_TRIANGLES, false);
    const render = tile.entity.render!;
    if (render.layers.length !== 1 || render.layers[0] !== layer) render.layers = [layer];
    tile.entity.enabled = true;
    this.remade++;
  }
}
