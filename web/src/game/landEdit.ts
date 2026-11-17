/**
 * Shaping a world's land: strokes of the land brushes (`landBrush.ts`) over its height field, with
 * undo and redo per stroke, the built land held off, and what a stroke changed turned into what
 * the world keeps (the sim's water tiles, the zones the renderer and the map read) and its price.
 * The same in step 1 and in Build mode's Land tab, and headless (tests, benches, `window.__land`).
 *
 * The editor keeps two copies of the land besides the land itself: as it was first surveyed (for
 * `reset`, and to tell what has changed since) and as it was before the stroke under way (a
 * stroke's undo is cut from it at the stroke's end, so a stamp keeps no undo of its own).
 */

import type { NeighbourhoodDraft, Rect } from './town';
import type { Rle, RoadPlan, WorldStructure, WorldZone } from '../core/protocol';
import { COVER, COVER_CELL, coverSpan, flowsOf, LAND_CELL, NONE, SEA_CM, type Land, type LandPatch, type LandRect } from './land';
import { aimOf, coverCells, coverSamples, isPoint, mountainGhost, mountainOf, placeMountain, pointGrid, stamp, strokeOf, TOOL_KIND, unionRect, type Brush, type Free, type LandTool, type MountainGhost, type StrokeState, type Wettable } from './landBrush';
import { outlines, rle, unrle } from './landMasks';
import { deckTiles } from './macro';
import { smoothstep } from './noise';
import { bridgeSpans, inside, type Pt } from './roads';

/** Strokes kept to undo, and the most their old values may hold (bytes). */
export const UNDO_STROKES = 32;
export const UNDO_BYTES = 16 * 1024 * 1024;

/**
 * What shaping the land costs (content `build.land`): `earth` and `mountain` a cubic metre moved
 * (the mountain tools at `mountain`), the rest a square metre painted.
 */
export interface LandPrices {
  earth: number;
  mountain: number;
  water: number;
  woods: number;
  meadow: number;
  fields: number;
  clear: number;
}

// ---- Built land: what the brushes keep off --------------------------------------------------

/** The flat ground round what is built before the brushes may change the land at all (m: the renderer's `APRON`), and over how far they come in fully. */
export const APRON = 6;
export const APRON_FADE = 8;
/** Water keeps this far off a lot (m), and this much more than half its width off a road. */
export const LOT_SHORE = 3;
export const ROAD_SHORE = 2;
/** What counts as built round a road: half its width and this verge (m), as the renderer's distance-from-built grid has it. */
const ROAD_VERGE = 2;

/** A world's built land: its size, its lots (and the backdrop's buildings), its roads and trails. */
export interface BuiltLand {
  width: number;
  depth: number;
  plots: readonly Rect[];
  roads: readonly RoadPlan[];
  backdrop?: readonly { poly: Pt[] }[];
}

/** The built land of a world as the game or the menus' preview has it (`WorldStructure`: its plots, `meta.roads`, `meta.backdrop`). */
export function builtOf(world: Pick<WorldStructure, 'width' | 'depth' | 'plots' | 'meta'>): BuiltLand {
  const meta = world.meta as { roads?: RoadPlan[]; backdrop?: { poly: Pt[] }[] } | null | undefined;
  return { width: world.width, depth: world.depth, plots: world.plots.map(({ x, z, w, d }) => ({ x, z, w, d })), roads: meta?.roads ?? [], backdrop: meta?.backdrop ?? [] };
}

/**
 * What the brushes may do where, from a world's built land, per sample of a `cols` × `rows` land:
 * `free` (how far a sample may change) is 0 within `APRON` m of anything built (a lot, a backdrop
 * building and a metre round it, a road or trail and `ROAD_VERGE` m either side) and rises to 1 over
 * `APRON_FADE` m more; `wettable` is false within `LOT_SHORE` m of a lot or a building and
 * `ROAD_SHORE` m past a road's side.
 */
export class Protection {
  /** Each sample's distance (m) from what is built (two chamfer passes). */
  readonly dist: Float32Array;
  /** 1 where water is refused. */
  readonly dry: Uint8Array;
  /** 1 where water is refused but for the roads (a lot, a building and their shores): where water may bridge a road. */
  readonly dryLots: Uint8Array;

  constructor(
    readonly cols: number,
    readonly rows: number,
    built: BuiltLand | null,
  ) {
    this.dist = new Float32Array(cols * rows).fill(1e9);
    this.dry = new Uint8Array(cols * rows);
    this.dryLots = new Uint8Array(cols * rows);
    if (built) this.survey(built);
  }

  /** How far sample (i, j) may change (0..1). */
  readonly free: Free = (i, j) => {
    const d = this.dist[j * this.cols + i];
    return d >= APRON + APRON_FADE ? 1 : smoothstep(APRON, APRON + APRON_FADE, d);
  };

  /** Whether sample (i, j) may be under water. */
  readonly wettable: Wettable = (i, j) => !this.dry[j * this.cols + i];

  /** Whether sample (i, j) may be under water if the roads there are bridged (`landBridges.ts`): off lots and buildings. */
  readonly bridgeable: Wettable = (i, j) => !this.dryLots[j * this.cols + i];

  private survey(built: BuiltLand): void {
    const { cols, rows, dist, dry } = this;
    const c = LAND_CELL;
    const span = (lo: number, hi: number, n: number) => [Math.max(0, Math.ceil(lo / c)), Math.min(n - 1, Math.floor(hi / c))];
    const mark = (into: Float32Array | Uint8Array, value: number, x0: number, z0: number, x1: number, z1: number) => {
      const [i0, i1] = span(x0, x1, cols);
      const [j0, j1] = span(z0, z1, rows);
      for (let j = j0; j <= j1; j++) for (let i = i0; i <= i1; i++) into[j * cols + i] = value;
    };
    for (const p of built.plots) {
      mark(dist, 0, p.x, p.z, p.x + p.w, p.z + p.d);
      mark(dry, 1, p.x - LOT_SHORE, p.z - LOT_SHORE, p.x + p.w + LOT_SHORE, p.z + p.d + LOT_SHORE);
      mark(this.dryLots, 1, p.x - LOT_SHORE, p.z - LOT_SHORE, p.x + p.w + LOT_SHORE, p.z + p.d + LOT_SHORE);
    }
    for (const b of built.backdrop ?? []) {
      const xs = b.poly.map((q) => q[0]);
      const zs = b.poly.map((q) => q[1]);
      const [x0, z0, x1, z1] = [Math.min(...xs), Math.min(...zs), Math.max(...xs), Math.max(...zs)];
      mark(dist, 0, x0 - 1, z0 - 1, x1 + 1, z1 + 1);
      mark(dry, 1, x0 - LOT_SHORE, z0 - LOT_SHORE, x1 + LOT_SHORE, z1 + LOT_SHORE);
      mark(this.dryLots, 1, x0 - LOT_SHORE, z0 - LOT_SHORE, x1 + LOT_SHORE, z1 + LOT_SHORE);
    }
    // A road: every sample within its half-width and the verge (built), or the shore (dry), of its middle line.
    for (const r of built.roads) {
      const reach = r.width / 2 + Math.max(ROAD_VERGE, ROAD_SHORE);
      for (let n = 0; n + 1 < r.points.length; n++) {
        const [[ax, az], [bx, bz]] = [r.points[n], r.points[n + 1]];
        const [dx, dz] = [bx - ax, bz - az];
        const l2 = dx * dx + dz * dz || 1;
        const [i0, i1] = span(Math.min(ax, bx) - reach, Math.max(ax, bx) + reach, cols);
        const [j0, j1] = span(Math.min(az, bz) - reach, Math.max(az, bz) + reach, rows);
        for (let j = j0; j <= j1; j++) {
          for (let i = i0; i <= i1; i++) {
            const [px, pz] = [i * c - ax, j * c - az];
            const t = Math.max(0, Math.min(1, (px * dx + pz * dz) / l2));
            const d = Math.hypot(px - dx * t, pz - dz * t) - r.width / 2;
            if (d <= ROAD_VERGE) dist[j * cols + i] = 0;
            if (d <= ROAD_SHORE) dry[j * cols + i] = 1;
          }
        }
      }
    }
    const diag = c * Math.SQRT2;
    const relax = (k: number, n: number, cost: number) => {
      if (dist[n] + cost < dist[k]) dist[k] = dist[n] + cost;
    };
    for (let j = 0; j < rows; j++) {
      for (let i = 0; i < cols; i++) {
        const k = j * cols + i;
        if (i > 0) relax(k, k - 1, c);
        if (j > 0) {
          relax(k, k - cols, c);
          if (i > 0) relax(k, k - cols - 1, diag);
          if (i < cols - 1) relax(k, k - cols + 1, diag);
        }
      }
    }
    for (let j = rows - 1; j >= 0; j--) {
      for (let i = cols - 1; i >= 0; i--) {
        const k = j * cols + i;
        if (i < cols - 1) relax(k, k + 1, c);
        if (j < rows - 1) {
          relax(k, k + cols, c);
          if (i < cols - 1) relax(k, k + cols + 1, diag);
          if (i > 0) relax(k, k + cols - 1, diag);
        }
      }
    }
  }
}

// ---- What the land makes of the world: water tiles and zones --------------------------------

/** Samples past the world's edge a water outline is carried on (`landZones`): the sea and rivers run on out of the world, as the generator's do. */
const WATER_PAST = 30;
/** The smallest piece of each kind kept as a zone (cells: water 2 m, the cover 4 m). */
const MIN_CELLS = { water: 4, woods: 6, meadow: 6, fields: 8 };

/** Whether the land is wet at 1 m tile (`x`, `z`): its nearest sample is. */
export function wetTile(land: Land, x: number, z: number): boolean {
  const i = Math.min(land.cols - 1, Math.round((x + 0.5) / LAND_CELL));
  const j = Math.min(land.rows - 1, Math.round((z + 0.5) / LAND_CELL));
  const k = j * land.cols + i;
  return land.wl[k] !== NONE && land.wl[k] > land.h[k];
}

/** The land's water as a `W` × `D` world's 1 m tiles (1 wet; each tile as its nearest sample). */
export function wetTiles(land: Land, W: number, D: number): Uint8Array {
  const mask = new Uint8Array(W * D);
  for (let z = 0; z < D; z++) for (let x = 0; x < W; x++) if (wetTile(land, x, z)) mask[z * W + x] = 1;
  return mask;
}

/**
 * The land's water and cover as `meta.zones` (`landMasks.ts` `outlines`): the water's outlines
 * from its 2 m samples (carried `WATER_PAST` samples on past the world's edge, as its edge stands),
 * the fields, meadows and woods from the 4 m cover (not where it is wet). In the generator's
 * order: water, fields, meadow, woods.
 */
export function landZones(land: Land, which: { water?: boolean; cover?: boolean } = { water: true, cover: true }): WorldZone[] {
  const { cols, rows } = land;
  const out: WorldZone[] = [];
  const wet = (i: number, j: number) => {
    const k = Math.min(rows - 1, Math.max(0, j)) * cols + Math.min(cols - 1, Math.max(0, i));
    return land.wl[k] !== NONE && land.wl[k] > land.h[k];
  };
  if (which.water) {
    const M = WATER_PAST;
    const [gw, gd] = [cols + 2 * M, rows + 2 * M];
    const mask = new Uint8Array(gw * gd);
    for (let j = 0; j < gd; j++) for (let i = 0; i < gw; i++) if (wet(i - M, j - M)) mask[j * gw + i] = 1;
    // (A sample stands at its cell's corner, 2i m; its cell in the mask is the 2 m round it.)
    const shift = -M * LAND_CELL - LAND_CELL / 2;
    for (const poly of outlines(mask, gw, gd, MIN_CELLS.water, LAND_CELL)) out.push({ kind: 'water', poly: poly.map(([x, z]): [number, number] => [x + shift, z + shift]) });
  }
  if (which.cover) {
    const [cw, cd] = [coverSpan(cols), coverSpan(rows)];
    const k = COVER_CELL / LAND_CELL;
    for (const kind of ['fields', 'meadow', 'woods'] as const) {
      const mask = new Uint8Array(cw * cd);
      let any = false;
      for (let r = 0; r < cd; r++) {
        for (let c = 0; c < cw; c++) {
          if (land.cover[r * cw + c] !== COVER[kind] || wet(c * k + 1, r * k + 1)) continue;
          mask[r * cw + c] = 1;
          any = true;
        }
      }
      if (any) for (const poly of outlines(mask, cw, cd, MIN_CELLS[kind], COVER_CELL)) out.push({ kind, poly });
    }
  }
  return out;
}

/** Zones `derived` from the land put in place of `base`'s kinds they cover, in the generator's order (water, hills, fields, meadows, woods, orchards). */
function mergeZones(base: readonly WorldZone[], derived: readonly WorldZone[], water: boolean, cover: boolean): WorldZone[] {
  const from = (kind: WorldZone['kind']) => (kind === 'water' ? water : kind === 'hill' || kind === 'orchard' ? false : cover) ? derived.filter((z) => z.kind === kind) : base.filter((z) => z.kind === kind);
  return (['water', 'hill', 'fields', 'meadow', 'woods', 'orchard'] as const).flatMap(from);
}

// ---- The editor ------------------------------------------------------------------------------

/** What `LandEditor` needs to know of its world beyond the land. */
export interface LandEditorOptions {
  /** Whether a sample may be under water (every sample if not given). */
  wettable?: Wettable;
  /** The world's size (m; the land's own if not given) and roads (their bridge decks stay dry). */
  world?: { width: number; depth: number; roads?: readonly RoadPlan[] };
  /** The generator's water tiles (`Rle`) and zones: what the land's changes are laid over (`water`, `zonesOf`). */
  water?: Rle;
  zones?: readonly WorldZone[];
  /**
   * The earth tools (raise, lower, smooth, flatten) work on built land too: its town follows the land (its roads and
   * lots stand on a surface solved from it, `builtSurface.ts`), so they move with it. The water and cover tools, and
   * the mountains, keep off it as ever.
   */
  earthOnBuilt?: boolean;
  /**
   * At the end of an earth stroke, the land that changed (`rect`) settled under what is built (`Renderer.settleLand`:
   * no road or lot cut or filled past its limits); returns the samples it changed besides (null: none), which the
   * stroke's patch takes in, so undo puts them back too.
   */
  settle?: (rect: LandRect, before: Int16Array) => LandRect | null;
}

/** Every sample free: the earth tools on a land its town follows (`LandEditorOptions.earthOnBuilt`). */
const ALL_FREE: Free = () => 1;

/** What a stroke (or a reset) on the undo or redo stack is, for its price. */
interface Kept {
  tool: LandTool | 'reset';
  /** The patch takes the stroke back (it is on the redo stack): its cost is a refund. */
  refund: boolean;
  /**
   * An earth stroke on built land priced as the brush left it, before the land was settled under what is built
   * (`settle`, free: the town's doing, not the player's): the samples it changed then, their heights after and before.
   */
  priced?: { rect: LandRect; h: Int16Array; was: Int16Array };
}

/** A stroke's stamps are this far apart along its path at least (m), a quarter of its radius on larger brushes. */
const SPACING_MIN = LAND_CELL / 2;
/** Held still, a brush stamps again after this long (s). */
const HOLD_EVERY = 1 / 30;
/** The first stamp of a stroke counts as this long (s). */
const FIRST_DT = 1 / 60;

/** Copies `rect` of a row-major `cols`-wide array out (row-major over the rectangle). */
function cut<T extends Int16Array | Uint8Array>(from: T, cols: number, rect: LandRect): T {
  const w = rect.i1 - rect.i0;
  const out = new (from.constructor as new (n: number) => T)(w * (rect.j1 - rect.j0));
  for (let j = rect.j0; j < rect.j1; j++) out.set(from.subarray(j * cols + rect.i0, j * cols + rect.i1), (j - rect.j0) * w);
  return out;
}

/** Copies `rect` of `from` into `to` (both row-major, `cols` wide). */
function copyRect(from: Int16Array | Uint8Array, to: Int16Array | Uint8Array, cols: number, rect: LandRect): void {
  for (let j = rect.j0; j < rect.j1; j++) to.set(from.subarray(j * cols + rect.i0, j * cols + rect.i1), j * cols + rect.i0);
}

/** Exchanges `rect` of `land` (`cols` wide) with `patch` (row-major over the rectangle), and copies the new values into `shadow`. */
function swapRect(patch: Int16Array | Uint8Array, land: Int16Array | Uint8Array, shadow: Int16Array | Uint8Array, cols: number, rect: LandRect): void {
  const w = rect.i1 - rect.i0;
  for (let j = rect.j0; j < rect.j1; j++) {
    for (let i = rect.i0; i < rect.i1; i++) {
      const [k, p] = [j * cols + i, (j - rect.j0) * w + (i - rect.i0)];
      const v = land[k];
      land[k] = shadow[k] = patch[p];
      patch[p] = v;
    }
  }
}

const patchBytes = (p: LandPatch) => (p.h?.byteLength ?? 0) + (p.wl?.byteLength ?? 0) + (p.flow?.byteLength ?? 0) + (p.cover?.data.byteLength ?? 0);

/** The smallest rectangle of a `cols` × `rows` grid where `a` and `b` differ; null if they don't. */
function differs(a: ArrayLike<number>, b: ArrayLike<number>, cols: number, rows: number): LandRect | null {
  let [i0, j0, i1, j1] = [cols, rows, -1, -1];
  for (let j = 0; j < rows; j++) {
    for (let i = 0, k = j * cols; i < cols; i++, k++) {
      if (a[k] === b[k]) continue;
      if (i < i0) i0 = i;
      if (i > i1) i1 = i;
      if (j < j0) j0 = j;
      j1 = j;
    }
  }
  return i1 < 0 ? null : { i0, j0, i1: i1 + 1, j1: j1 + 1 };
}

export class LandEditor {
  /** Strokes to undo (the last last) and to redo: what each changed held before it. */
  private readonly undone: LandPatch[] = [];
  private readonly redone: LandPatch[] = [];
  /** What each patch on the stacks is. */
  private readonly kept = new WeakMap<LandPatch, Kept>();
  /** The land as first surveyed, and as it was before the stroke under way. */
  private readonly original: { h: Int16Array; wl: Int16Array; flow: Uint8Array; cover: Uint8Array };
  private readonly shadow: { h: Int16Array; wl: Int16Array; flow: Uint8Array; cover: Uint8Array };
  /** The land had been shaped before the editor had it (read from a world's `meta.land`). */
  private readonly editedBefore: boolean;
  private readonly wettable?: Wettable;
  private readonly earthOnBuilt: boolean;
  private readonly settle?: (rect: LandRect, before: Int16Array) => LandRect | null;
  private world: { width: number; depth: number; roads: readonly RoadPlan[] };
  /** The world's water tiles and zones as the generator laid them, if the editor was given them. */
  readonly generated: { water?: Rle; zones?: readonly WorldZone[] };
  /** The bridge decks' tiles (the water under them stays dry to walk on), made when first asked for. */
  private decks: Set<number> | null = null;
  private active: StrokeState | null = null;
  /** Where the stroke last stamped, and the seconds since. */
  private last: [number, number] = [0, 0];
  private pending = 0;
  /** Why the last stroke stopped or left something out (null: it didn't), for the palette's hint line. */
  hint: string | null = null;

  /**
   * `land` is changed in place (given its `flow`, all still, if it had none). `free(i, j)` is how far
   * sample (i, j) may change: 0 on built land, rising to 1 clear of it (the protection mask:
   * `Protection`, or `LandEditor.forWorld`).
   */
  constructor(
    readonly land: Land,
    readonly free: Free,
    options: LandEditorOptions = {},
  ) {
    const flow = flowsOf(land);
    this.original = { h: land.h.slice(), wl: land.wl.slice(), flow: flow.slice(), cover: land.cover.slice() };
    this.shadow = { h: land.h.slice(), wl: land.wl.slice(), flow: flow.slice(), cover: land.cover.slice() };
    this.editedBefore = land.edited;
    this.wettable = options.wettable;
    this.earthOnBuilt = !!options.earthOnBuilt;
    this.settle = options.settle;
    this.world = { width: options.world?.width ?? land.cols * LAND_CELL, depth: options.world?.depth ?? land.rows * LAND_CELL, roads: options.world?.roads ?? [] };
    this.generated = { water: options.water, zones: options.zones };
  }

  /**
   * An editor for a world's land, holding the brushes off its built land (`Protection`), its
   * bridge decks dry; `generated`: the world's water tiles and zones as the generator laid them
   * (a draft's `open.water` and `open.zones`, or the game's), for `water` and `zonesOf`.
   */
  static forWorld(land: Land, built: BuiltLand, generated: { water?: Rle; zones?: readonly WorldZone[] } = {}, follow: Pick<LandEditorOptions, 'earthOnBuilt' | 'settle'> & { bridges?: boolean } = {}): LandEditor {
    const guard = new Protection(land.cols, land.rows, built);
    // (`bridges`: water may cross a road, which is bridged there or the stroke refused: `landBridges.ts`.)
    const { bridges, ...rest } = follow;
    return new LandEditor(land, guard.free, { wettable: bridges ? guard.bridgeable : guard.wettable, world: { width: built.width, depth: built.depth, roads: built.roads }, ...generated, ...rest });
  }

  /**
   * How far sample (i, j) may change under `tool`: `free`, but every sample for the earth and water tools on a land its
   * town follows (the water still kept off lots by `wettable`; across a road it is bridged, `landBridges.ts`).
   */
  freeFor(tool: LandTool): Free {
    return this.earthOnBuilt && (TOOL_KIND[tool] === 'earth' || TOOL_KIND[tool] === 'water') ? ALL_FREE : this.free;
  }

  /** The world's roads the editor knows (their bridges' decks kept dry). */
  get roads(): readonly RoadPlan[] {
    return this.world.roads;
  }

  /** The world's roads as they are now (bridges made or taken away: `landBridges.ts`): their decks kept dry from now on. */
  setRoads(roads: readonly RoadPlan[]): void {
    this.world = { ...this.world, roads };
    this.decks = null;
  }

  /** The stroke under way, or null. */
  get current(): StrokeState | null {
    return this.active;
  }

  get canUndo(): boolean {
    return this.undone.length > 0;
  }

  get canRedo(): boolean {
    return this.redone.length > 0;
  }

  /** Starts a stroke of `brush` at world point (`x`, `z`); returns the samples it changed. */
  begin(brush: Brush, x: number, z: number): LandRect | null {
    if (this.active) this.end();
    this.active = strokeOf({ ...brush }, this.land, { h: this.shadow.h, wl: this.shadow.wl, flow: this.shadow.flow });
    this.hint = null;
    this.last = aimOf(brush, x, z);
    this.pending = 0;
    return this.stampAt(x, z, FIRST_DT);
  }

  /**
   * The stroke dragged on to (`x`, `z`), `dt` seconds later; returns the samples it changed. It
   * stamps every quarter of the brush's radius along the way (the time shared between them), and
   * held still, again every `HOLD_EVERY` s. A point brush stamps once on each grid point it
   * crosses (`pointsTo`).
   */
  move(x: number, z: number, dt: number): LandRect | null {
    const stroke = this.active;
    if (!stroke) return null;
    this.pending += Math.max(0, dt);
    if (isPoint(stroke.brush)) return this.pointsTo(stroke, x, z);
    const spacing = Math.max(SPACING_MIN, stroke.brush.radius / 4);
    const [px, pz] = this.last;
    const d = Math.hypot(x - px, z - pz);
    if (d < spacing) {
      if (this.pending < HOLD_EVERY || (TOOL_KIND[stroke.brush.tool] !== 'earth' && stroke.brush.tool !== 'level')) return null;
      const rect = this.stampAt(px, pz, this.pending);
      this.pending = 0;
      return rect;
    }
    const n = Math.floor(d / spacing);
    const each = this.pending / n;
    let out: LandRect | null = null;
    for (let k = 1; k <= n; k++) out = unionRect(out, this.stampAt(px + ((x - px) * k * spacing) / d, pz + ((z - pz) * k * spacing) / d, each));
    this.last = [px + ((x - px) * n * spacing) / d, pz + ((z - pz) * n * spacing) / d];
    this.pending = 0;
    return out;
  }

  /**
   * A whole stroke of `brush` along `points` (world m), `dt` seconds between them: `begin` at the
   * first, `move` to each of the rest, `end`. For tests, benches and the `window.__land` hook.
   */
  stroke(brush: Brush, points: readonly (readonly [number, number])[], dt = 1 / 60): LandPatch | null {
    if (!points.length) return null;
    this.begin(brush, points[0][0], points[0][1]);
    for (let k = 1; k < points.length; k++) this.move(points[k][0], points[k][1], dt);
    return this.end();
  }

  /**
   * What the stroke under way has cost so far at `prices` (0 if none), to show at the cursor and
   * to stop a stroke at the household's funds; a `peak` or `range` what its ghost would cost.
   */
  running(prices: LandPrices): number {
    const stroke = this.active;
    if (!stroke) return 0;
    const tool = stroke.brush.tool;
    if (tool === 'peak' || tool === 'range') {
      const m = mountainOf(stroke);
      if (!m) return 0;
      const [x0, z0, x1, z1] = m.bounds;
      let moved = 0;
      for (let j = Math.max(0, Math.ceil(z0 / LAND_CELL)); j < Math.min(this.land.rows, Math.floor(z1 / LAND_CELL) + 1); j++) {
        for (let i = Math.max(0, Math.ceil(x0 / LAND_CELL)); i < Math.min(this.land.cols, Math.floor(x1 / LAND_CELL) + 1); i++) {
          const fr = this.free(i, j);
          if (fr > 0) moved += m.lift(i * LAND_CELL, j * LAND_CELL) * fr;
        }
      }
      return Math.round(moved * LAND_CELL * LAND_CELL * prices.mountain);
    }
    const patch = this.patchOf(stroke);
    if (!patch) return 0;
    this.kept.set(patch, { tool, refund: false });
    return this.cost(patch, prices);
  }

  /** The ghost of the mountain a `peak` or `range` stroke under way would raise on release, every `step` m (null: none). */
  ghost(step = 4): MountainGhost | null {
    const tool = this.active?.brush.tool;
    return tool === 'peak' || tool === 'range' ? mountainGhost(this.active!, step) : null;
  }

  /** Ends the stroke: what it changed held before it (pushed on the undo stack), or null if it changed nothing. */
  end(): LandPatch | null {
    const stroke = this.active;
    if (!stroke) return null;
    this.active = null;
    const tool = stroke.brush.tool;
    if (tool === 'peak' || tool === 'range') placeMountain(this.land, stroke, this.free, this.wettable);
    this.hint = stroke.stopped ?? stroke.hint ?? null;
    // (On built land, the land settled under what is built: what that changed is the stroke's too.)
    let priced: Kept['priced'];
    if (this.settle && this.earthOnBuilt && TOOL_KIND[tool] === 'earth' && stroke.touched) {
      priced = { rect: stroke.touched, h: cut(this.land.h, this.land.cols, stroke.touched), was: cut(this.shadow.h, this.land.cols, stroke.touched) };
      const more = this.settle(stroke.touched, this.shadow.h);
      if (more) stroke.touched = unionRect(stroke.touched, more);
    }
    const patch = this.patchOf(stroke);
    if (!patch || !this.differ(patch)) return null;
    this.sync(patch.rect, patch.cover?.rect);
    this.push(patch, tool, priced);
    return patch;
  }

  /** Takes back the last stroke; returns the patch that redoes it (its rectangle is what changed), or null if there is none. */
  undo(): LandPatch | null {
    if (this.active) this.end();
    const patch = this.undone.pop();
    if (!patch) return null;
    this.swap(patch);
    this.kept.get(patch)!.refund = true;
    this.redone.push(patch);
    this.markEdited();
    return patch;
  }

  /** Forgets the strokes taken back (a stroke the game refused, taken back as if it had never been). */
  dropRedo(): void {
    this.redone.length = 0;
  }

  /**
   * Whether `tool` would do nothing at world point (`x`, `z`) (the cursor's red): the land is
   * built there (not free to change), or it is a lake or river where water may not go.
   */
  refuses(tool: LandTool, x: number, z: number): boolean {
    const { cols, rows } = this.land;
    const i = Math.min(cols - 1, Math.max(0, Math.round(x / LAND_CELL)));
    const j = Math.min(rows - 1, Math.max(0, Math.round(z / LAND_CELL)));
    if (this.freeFor(tool)(i, j) <= 0) return true;
    return (tool === 'lake' || tool === 'river') && !!this.wettable && !this.wettable(i, j);
  }

  /** Puts back the last stroke taken back; returns the patch that undoes it again, or null if there is none. */
  redo(): LandPatch | null {
    if (this.active) this.end();
    const patch = this.redone.pop();
    if (!patch) return null;
    this.swap(patch);
    this.kept.get(patch)!.refund = false;
    this.undone.push(patch);
    this.markEdited();
    return patch;
  }

  /**
   * The land as it was first surveyed, as one more stroke (undone as one); returns what changed,
   * or null if nothing had. Charged (`cost`) as a refund of the heights at the `mountain` rate, so
   * a reset never pays back more than the strokes cost.
   */
  reset(): LandRect | null {
    if (this.active) this.end();
    const { cols, rows } = this.land;
    const [cw, cd] = [coverSpan(cols), coverSpan(rows)];
    const cells = differs(this.land.cover, this.original.cover, cw, cd);
    const flow = flowsOf(this.land);
    const samples = unionRect(unionRect(differs(this.land.h, this.original.h, cols, rows), differs(this.land.wl, this.original.wl, cols, rows)), differs(flow, this.original.flow, cols, rows));
    const rect = unionRect(samples, cells && coverSamples(this.land, cells));
    if (!rect) return null;
    const coverRect = coverCells(this.land, rect);
    const patch: LandPatch = { rect, h: cut(this.land.h, cols, rect), wl: cut(this.land.wl, cols, rect), flow: cut(flow, cols, rect), cover: { rect: coverRect, data: cut(this.land.cover, cw, coverRect) } };
    copyRect(this.original.h, this.land.h, cols, rect);
    copyRect(this.original.wl, this.land.wl, cols, rect);
    copyRect(this.original.flow, flow, cols, rect);
    copyRect(this.original.cover, this.land.cover, cw, coverRect);
    this.sync(rect, coverRect);
    this.push(patch, 'reset');
    return rect;
  }

  /** What has changed since the land was first surveyed: its heights, its water (where it is wet), its cover. */
  changed(): { heights: boolean; water: boolean; cover: boolean } {
    const { h, wl, cover } = this.land;
    const o = this.original;
    let [heights, water] = [false, false];
    for (let k = 0; k < h.length && !(heights && water); k++) {
      if (h[k] !== o.h[k]) heights = true;
      if ((wl[k] !== NONE && wl[k] > h[k]) !== (o.wl[k] !== NONE && o.wl[k] > o.h[k])) water = true;
    }
    let coverChanged = false;
    for (let k = 0; k < cover.length && !coverChanged; k++) if (cover[k] !== o.cover[k]) coverChanged = true;
    return { heights, water, cover: coverChanged };
  }

  /** The strokes kept, and the bytes their undo holds. */
  stats(): { undo: number; redo: number; bytes: number; edited: boolean } {
    return { undo: this.undone.length, redo: this.redone.length, bytes: this.bytes(), edited: this.land.edited };
  }

  /**
   * The water within `rect` (samples) as the sim's 1 m tiles: 1 wet (as each tile's nearest
   * sample is), but 0 on a bridge's deck; row-major over the tiles from (2 `i0`, 2 `j0`) to
   * (2 `i1`, 2 `j1`) m, cut to the world (`tileRect`).
   */
  waterTiles(rect: LandRect): Uint8Array {
    const t = this.tileRect(rect);
    const w = t.x1 - t.x0;
    const out = new Uint8Array(Math.max(0, w * (t.z1 - t.z0)));
    const decks = this.deckSet();
    for (let z = t.z0; z < t.z1; z++) for (let x = t.x0; x < t.x1; x++) if (wetTile(this.land, x, z) && !decks.has(z * this.world.width + x)) out[(z - t.z0) * w + (x - t.x0)] = 1;
    return out;
  }

  /** The 1 m tiles `waterTiles(rect)` covers: x0 ≤ x < x1, z0 ≤ z < z1. */
  tileRect(rect: LandRect): { x0: number; z0: number; x1: number; z1: number } {
    const { width: W, depth: D } = this.world;
    return { x0: Math.max(0, rect.i0 * LAND_CELL), z0: Math.max(0, rect.j0 * LAND_CELL), x1: Math.min(W, rect.i1 * LAND_CELL), z1: Math.min(D, rect.j1 * LAND_CELL) };
  }

  /**
   * The world's whole water mask as the sim walks it (`terrain.water`, `Rle`): the generator's
   * (`base`, else the one the editor was given; none: dry), with each tile whose sample's water
   * changed since the land was surveyed taken from the land, and the bridge decks dry.
   */
  water(base: Rle | undefined = this.generated.water): Rle {
    const { width: W, depth: D } = this.world;
    const mask = base ? unrle(base, W * D) : new Uint8Array(W * D);
    const { cols, rows, h, wl } = this.land;
    const o = this.original;
    for (let j = 0; j < rows; j++) {
      for (let i = 0; i < cols; i++) {
        const k = j * cols + i;
        const now = wl[k] !== NONE && wl[k] > h[k];
        if (now === (o.wl[k] !== NONE && o.wl[k] > o.h[k])) continue;
        // (Sample i is the nearest to tiles 2i - 1 and 2i.)
        for (let z = Math.max(0, j * 2 - 1); z <= Math.min(D - 1, j * 2); z++) for (let x = Math.max(0, i * 2 - 1); x <= Math.min(W - 1, i * 2); x++) mask[z * W + x] = now ? 1 : 0;
      }
    }
    for (const t of this.deckSet()) mask[t] = 0;
    return rle(mask);
  }

  /**
   * The woods, meadows, fields and water of the land as `meta.zones` (`landZones`), laid over
   * `base` (else the generator's the editor was given): of those, only the kinds the land has
   * changed (its water, its cover) are taken from it; with no base, all four are, and no hills or
   * orchards.
   */
  zonesOf(base: readonly WorldZone[] | undefined = this.generated.zones): WorldZone[] {
    if (!base) return landZones(this.land);
    const { water, cover } = this.changed();
    return mergeZones(base, landZones(this.land, { water, cover }), water, cover);
  }

  /**
   * What the change `patch` holds the old values of costs, at `prices` (negative: a refund, for a
   * patch `undo` returned). The shape tools: the earth moved, Σ|Δh| × 4 m², at `earth` a m³ (the
   * mountain tools and a reset at `mountain`); the water tools: the ground made wet or dry, at
   * `water` a m²; the cover tools: the ground painted, at its cover's price a m² (`clear` for
   * open ground). Whole dollars.
   */
  cost(patch: LandPatch, prices: LandPrices): number {
    const kept = this.kept.get(patch) ?? { tool: 'raise', refund: false };
    const kind = kept.tool === 'reset' ? 'reset' : TOOL_KIND[kept.tool];
    const { cols } = this.land;
    const { rect } = patch;
    const w = rect.i1 - rect.i0;
    let total = 0;
    if (patch.h && (kind === 'earth' || kind === 'mountain' || kind === 'reset')) {
      let moved = 0;
      const p = kept.priced;
      if (p) {
        // (As the brush left it, against the land before the stroke: the settling is free.)
        for (let k = 0; k < p.h.length; k++) moved += Math.abs(p.h[k] - p.was[k]);
      } else for (let j = rect.j0; j < rect.j1; j++) for (let i = rect.i0; i < rect.i1; i++) moved += Math.abs(this.land.h[j * cols + i] - patch.h[(j - rect.j0) * w + (i - rect.i0)]);
      total += (moved / 100) * LAND_CELL * LAND_CELL * (kind === 'earth' ? prices.earth : prices.mountain);
    }
    if (patch.h && patch.wl && (kind === 'water' || kind === 'reset')) {
      let wetted = 0;
      for (let j = rect.j0; j < rect.j1; j++) {
        for (let i = rect.i0; i < rect.i1; i++) {
          const [k, p] = [j * cols + i, (j - rect.j0) * w + (i - rect.i0)];
          if ((this.land.wl[k] !== NONE && this.land.wl[k] > this.land.h[k]) !== (patch.wl[p] !== NONE && patch.wl[p] > patch.h[p])) wetted++;
        }
      }
      total += wetted * LAND_CELL * LAND_CELL * prices.water;
    }
    if (patch.cover) {
      const { rect: c, data } = patch.cover;
      const cw = coverSpan(cols);
      const names = ['clear', 'woods', 'meadow', 'fields'] as const;
      for (let r = c.j0; r < c.j1; r++) {
        for (let q = c.i0; q < c.i1; q++) {
          const [now, then] = [this.land.cover[r * cw + q], data[(r - c.j0) * (c.i1 - c.i0) + (q - c.i0)]];
          // (Priced by what the stroke painted: the new cover, or for a refund, the cover it takes away.)
          if (now !== then) total += COVER_CELL * COVER_CELL * prices[names[kept.refund ? then : now] ?? 'clear'];
        }
      }
    }
    const dollars = Math.round(total);
    return kept.refund || kind === 'reset' ? -dollars : dollars;
  }

  // ---- The stacks ----------------------------------------------------------------------------

  /** What `stroke` has touched as it was before it (cut from the copy of the land before the stroke), or null if it touched nothing. */
  private patchOf(stroke: StrokeState): LandPatch | null {
    const rect = stroke.touched;
    if (!rect) return null;
    const { cols } = this.land;
    if (TOOL_KIND[stroke.brush.tool] === 'cover') {
      const cells = coverCells(this.land, rect);
      return { rect, cover: { rect: cells, data: cut(this.shadow.cover, coverSpan(cols), cells) } };
    }
    return { rect, h: cut(this.shadow.h, cols, rect), wl: cut(this.shadow.wl, cols, rect), flow: cut(this.shadow.flow, cols, rect) };
  }

  /**
   * A point brush dragged on to (`x`, `z`): a stamp on each grid point from the last one to the one
   * nearest (`x`, `z`), stepping a point across or along at a time, so the trail it leaves is
   * unbroken side to side (water runs along it) and no point is stamped twice in a frame; the time
   * is shared between them. On the same point, held still, as a larger brush.
   */
  private pointsTo(stroke: StrokeState, x: number, z: number): LandRect | null {
    const { unit, offset } = pointGrid(stroke.brush.tool);
    const index = (v: number) => Math.round((v - offset) / unit);
    const [ia, ja] = [index(this.last[0]), index(this.last[1])];
    const [ib, jb] = [index(x), index(z)];
    const [n, m] = [Math.abs(ib - ia), Math.abs(jb - ja)];
    if (!n && !m) {
      if (this.pending < HOLD_EVERY || (TOOL_KIND[stroke.brush.tool] !== 'earth' && stroke.brush.tool !== 'level')) return null;
      const rect = this.stampAt(this.last[0], this.last[1], this.pending);
      this.pending = 0;
      return rect;
    }
    const each = this.pending / (n + m);
    let out: LandRect | null = null;
    // (Each step across or along, whichever the straight line from the last point meets first.)
    for (let a = 0, b = 0; a < n || b < m; ) {
      if ((a + 0.5) / n < (b + 0.5) / m) a++;
      else b++;
      this.last = [offset + (ia + Math.sign(ib - ia) * a) * unit, offset + (ja + Math.sign(jb - ja) * b) * unit];
      out = unionRect(out, this.stampAt(this.last[0], this.last[1], each));
    }
    this.pending = 0;
    return out;
  }

  private stampAt(x: number, z: number, dt: number): LandRect | null {
    const stroke = this.active!;
    return stamp(this.land, stroke.brush, { x, z, dt }, stroke, this.freeFor(stroke.brush.tool), this.wettable);
  }

  /** Whether the land differs from `patch`'s old values anywhere in it. */
  private differ(patch: LandPatch): boolean {
    const { cols } = this.land;
    const same = (old: Int16Array | Uint8Array, now: Int16Array | Uint8Array, width: number, r: LandRect) => {
      const w = r.i1 - r.i0;
      for (let j = r.j0; j < r.j1; j++) for (let i = r.i0; i < r.i1; i++) if (old[(j - r.j0) * w + (i - r.i0)] !== now[j * width + i]) return false;
      return true;
    };
    if (patch.h && !same(patch.h, this.land.h, cols, patch.rect)) return true;
    if (patch.wl && !same(patch.wl, this.land.wl, cols, patch.rect)) return true;
    if (patch.flow && !same(patch.flow, flowsOf(this.land), cols, patch.rect)) return true;
    return !!patch.cover && !same(patch.cover.data, this.land.cover, coverSpan(cols), patch.cover.rect);
  }

  /** The copy of the land before a stroke brought up to it over `rect` (samples) and `cells` (cover). */
  private sync(rect: LandRect, cells?: LandRect): void {
    const { cols } = this.land;
    copyRect(this.land.h, this.shadow.h, cols, rect);
    copyRect(this.land.wl, this.shadow.wl, cols, rect);
    copyRect(flowsOf(this.land), this.shadow.flow, cols, rect);
    copyRect(this.land.cover, this.shadow.cover, coverSpan(cols), cells ?? coverCells(this.land, rect));
  }

  private swap(patch: LandPatch): void {
    const { cols } = this.land;
    if (patch.h) swapRect(patch.h, this.land.h, this.shadow.h, cols, patch.rect);
    if (patch.wl) swapRect(patch.wl, this.land.wl, this.shadow.wl, cols, patch.rect);
    if (patch.flow) swapRect(patch.flow, flowsOf(this.land), this.shadow.flow, cols, patch.rect);
    if (patch.cover) swapRect(patch.cover.data, this.land.cover, this.shadow.cover, coverSpan(cols), patch.cover.rect);
  }

  private push(patch: LandPatch, tool: LandTool | 'reset', priced?: Kept['priced']): void {
    this.kept.set(patch, { tool, refund: false, ...(priced ? { priced } : {}) });
    this.undone.push(patch);
    this.redone.length = 0;
    while (this.undone.length > 1 && (this.undone.length > UNDO_STROKES || this.bytes() > UNDO_BYTES)) this.undone.shift();
    this.markEdited();
  }

  private bytes(): number {
    let n = 0;
    for (const p of this.undone) n += patchBytes(p);
    for (const p of this.redone) n += patchBytes(p);
    return n;
  }

  /** `land.edited`: shaped before the editor had it, or differing now from how it was surveyed. */
  private markEdited(): void {
    const { h, wl, cover } = this.land;
    const flow = flowsOf(this.land);
    const o = this.original;
    let differs = this.editedBefore;
    for (let k = 0; k < h.length && !differs; k++) if (h[k] !== o.h[k] || wl[k] !== o.wl[k] || flow[k] !== o.flow[k]) differs = true;
    for (let k = 0; k < cover.length && !differs; k++) if (cover[k] !== o.cover[k]) differs = true;
    this.land.edited = differs;
  }

  private deckSet(): Set<number> {
    if (this.decks) return this.decks;
    const set = new Set<number>();
    const { width: W, depth: D, roads } = this.world;
    for (const r of roads) for (const span of bridgeSpans(r)) deckTiles(r, span, W, D, (t) => set.add(t));
    return (this.decks = set);
  }
}

// ---- The draft ------------------------------------------------------------------------------

/**
 * `draft` (a world as the generator laid it out) on `editor`'s land, for the town file and step 2:
 * `open.land` the land (written to `meta.land` once it has been shaped), the water the sim walks
 * the generator's with the tiles whose water changed taken from the land (`LandEditor.water`), and
 * the zones from the land where its water or cover changed (`LandEditor.zonesOf`). The draft
 * itself is left as it was; an unshaped land leaves it as generated (no `open.land`, but for a land
 * its town follows the slopes of, `seeded` `'follow'`: that is the world's own, kept).
 */
export function applyLand(draft: NeighbourhoodDraft, editor: LandEditor): NeighbourhoodDraft {
  const open = draft.open;
  if (!open) return draft;
  const { land: _old, ...rest } = open;
  if (!editor.land.edited) return open.land?.seeded === 'follow' ? draft : { ...draft, open: rest };
  const changed = editor.changed();
  return {
    ...draft,
    open: {
      ...rest,
      land: editor.land,
      ...(changed.water ? { water: editor.water(editor.generated.water ?? open.water) } : {}),
      ...(changed.water || changed.cover ? { zones: editor.zonesOf(editor.generated.zones ?? open.zones) } : {}),
    },
  };
}

/**
 * A rough survey of a generated world's land without the renderer (the renderer's `landOf` is the
 * true one, from the land as drawn): its relief for heights, its water tiles as the sea's level,
 * its woods, meadow and fields zones as cover. For tests, benches, and a page with no renderer yet.
 */
export function surveyDraft(draft: NeighbourhoodDraft): Land {
  const W = draft.width;
  const D = draft.depth;
  const cols = Math.ceil(W / LAND_CELL);
  const rows = Math.ceil(D / LAND_CELL);
  const [cw, cd] = [coverSpan(cols), coverSpan(rows)];
  const land: Land = { cols, rows, h: new Int16Array(cols * rows), wl: new Int16Array(cols * rows).fill(NONE), cover: new Uint8Array(cw * cd), seeded: 'drawn', edited: false };
  const relief = draft.open?.relief;
  const water = draft.open ? unrle(draft.open.water, W * D) : null;
  for (let j = 0; j < rows; j++) {
    for (let i = 0; i < cols; i++) {
      const [x, z] = [i * LAND_CELL, j * LAND_CELL];
      const k = j * cols + i;
      if (relief) {
        const gx = Math.min(relief.cols - 1, Math.max(0, x / relief.cell - 0.5));
        const gz = Math.min(relief.rows - 1, Math.max(0, z / relief.cell - 0.5));
        const [x0, z0] = [Math.floor(gx), Math.floor(gz)];
        const [x1, z1] = [Math.min(relief.cols - 1, x0 + 1), Math.min(relief.rows - 1, z0 + 1)];
        const [u, v] = [gx - x0, gz - z0];
        const at = (c: number, r: number) => relief.h[r * relief.cols + c];
        land.h[k] = Math.round(((at(x0, z0) * (1 - u) + at(x1, z0) * u) * (1 - v) + (at(x0, z1) * (1 - u) + at(x1, z1) * u) * v) * 10);
      }
      if (water?.[Math.min(D - 1, z) * W + Math.min(W - 1, x)]) {
        land.wl[k] = SEA_CM;
        land.h[k] = SEA_CM - 100;
      }
    }
  }
  const zones = draft.open?.zones ?? [];
  for (const kind of ['fields', 'meadow', 'woods'] as const) {
    const polys = zones.filter((zone) => zone.kind === kind).map((zone) => zone.poly);
    for (const poly of polys) {
      const xs = poly.map((p) => p[0]);
      const zs = poly.map((p) => p[1]);
      const [c0, c1] = [Math.max(0, Math.floor(Math.min(...xs) / COVER_CELL)), Math.min(cw, Math.ceil(Math.max(...xs) / COVER_CELL))];
      const [r0, r1] = [Math.max(0, Math.floor(Math.min(...zs) / COVER_CELL)), Math.min(cd, Math.ceil(Math.max(...zs) / COVER_CELL))];
      for (let r = r0; r < r1; r++) for (let c = c0; c < c1; c++) if (inside(poly, (c + 0.5) * COVER_CELL, (r + 0.5) * COVER_CELL)) land.cover[r * cw + c] = COVER[kind];
    }
  }
  return land;
}

