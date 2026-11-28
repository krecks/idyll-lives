/**
 * Step 2's zone brush (`game/uses.ts`: the uses, "zones" to the player): a stroke of it painted on a
 * town's plan, and what the brush and the town's Build read of the uses' grid: the cells no use goes
 * on (the water with its shore strip, the roads, off the world) and how far each cell is from a road.
 *
 * A stroke changes the uses alone: no road, lot or building, nor the land under them. So its edit says
 * it needs no settle (`needsSettle`): the session shows the plan it settled last with the new uses
 * (`withUses`) and asks the worker for nothing.
 */

import { unrle } from '../landMasks';
import { grown, SHORE_GAP } from '../macro';
import type { Pt, RoadPlan } from '../roads';
import type { NeighbourhoodDraft } from '../town';
import { packUses, unpackUses, USE_CELL, USE_CODE, useGrid, usesFor, type UseKind, type UseLayer } from '../uses';
import type { Edited } from './edited';
import * as pmath from '../pmath';

/** A dab of the brush: its middle on the town (m) and how wide it paints (m). */
export interface UseStamp {
  x: number;
  z: number;
  size: number;
}

/** A stroke painted (`paintUses`): an edit of the uses alone, which needs no settle of the land. */
type UsesPainted = Extract<Edited, { draft: NeighbourhoodDraft }> & { needsSettle: false };

/** A cell is a road's if its middle is this near the road's middle line past half its width (m). */
const ROAD_REACH = 0.5;

// ---- the grid ---------------------------------------------------------------------------------------

/** `draft`'s uses (`open.uses`), or none painted yet on its world's grid. */
export function usesOfDraft(draft: Pick<NeighbourhoodDraft, 'width' | 'depth' | 'open'>): UseLayer {
  const layer = draft.open?.uses;
  const { cols, rows } = useGrid(draft.width, draft.depth);
  return layer && layer.cols === cols && layer.rows === rows ? layer : usesFor(draft);
}

/** What each set of roads' grids were made from, and the grids: the cells no use goes on, the cells on a road, and each cell's distance to a road. */
interface Grids {
  w: number;
  d: number;
  water: unknown;
  /** Per cell: 1 where no use goes (`noUseCells`). */
  none: Uint8Array;
  /** Per cell: its distance from the nearest road (`roadReach`), lazily. */
  reach?: Float32Array;
}
const grids = new WeakMap<readonly RoadPlan[], Grids>();

/** Each cell a road (not a footpath) runs over: its middle within half the road's width and `ROAD_REACH` of the road's middle line. */
function roadCells(roads: readonly RoadPlan[], cols: number, rows: number): Uint8Array {
  const out = new Uint8Array(cols * rows);
  for (const r of roads) {
    if (r.kind === 'trail') continue;
    const reach = r.width / 2 + ROAD_REACH;
    const span = Math.ceil(reach / USE_CELL) + 1;
    for (let k = 0; k + 1 < r.points.length; k++) {
      const [[ax, az], [bx, bz]] = [r.points[k], r.points[k + 1]];
      const steps = Math.max(1, Math.ceil(pmath.hypot(bx - ax, bz - az)));
      for (let j = 0; j <= steps; j++) {
        const [px, pz] = [ax + ((bx - ax) * j) / steps, az + ((bz - az) * j) / steps];
        const [ci, cj] = [Math.floor(px / USE_CELL), Math.floor(pz / USE_CELL)];
        for (let y = Math.max(0, cj - span); y <= Math.min(rows - 1, cj + span); y++) {
          for (let x = Math.max(0, ci - span); x <= Math.min(cols - 1, ci + span); x++) {
            if (pmath.hypot((x + 0.5) * USE_CELL - px, (y + 0.5) * USE_CELL - pz) <= reach) out[y * cols + x] = 1;
          }
        }
      }
    }
  }
  return out;
}

/** `draft`'s grids, made once per set of roads (and its world and water). */
function gridsOf(draft: Pick<NeighbourhoodDraft, 'width' | 'depth' | 'roads' | 'open'>): Grids {
  const { width: W, depth: D, roads } = draft;
  const water = draft.open?.water;
  const had = grids.get(roads);
  if (had && had.w === W && had.d === D && had.water === water) return had;
  const { cols, rows } = useGrid(W, D);
  const none = roadCells(roads, cols, rows);
  // The water and its shore strip (lots and buildings keep off it: `macro.ts` `SHORE_GAP`), at each cell's middle.
  const wet = water ? grown(unrle(water, W * D), W, D, SHORE_GAP) : null;
  for (let j = 0; j < rows; j++) {
    for (let i = 0; i < cols; i++) {
      const [x, z] = [Math.floor((i + 0.5) * USE_CELL), Math.floor((j + 0.5) * USE_CELL)];
      if (x >= W || z >= D || (wet && wet[z * W + x])) none[j * cols + i] = 1;
    }
  }
  const out: Grids = { w: W, d: D, water, none };
  grids.set(roads, out);
  return out;
}

/**
 * The cells of `draft`'s uses grid no use goes on (1, row-major): the water with its shore strip, the
 * roads (not the footpaths) and those whose middle is off the world. Made once per set of roads: to read, not change.
 */
export function noUseCells(draft: Pick<NeighbourhoodDraft, 'width' | 'depth' | 'roads' | 'open'>): Uint8Array {
  return gridsOf(draft).none;
}

/**
 * How far each cell of `draft`'s uses grid is from the nearest road (not a footpath; m, row-major, to
 * within a cell): from the middle of a cell the road runs over to the cell's middle, as a chamfer
 * distance over the grid. Infinity with no roads. Made once per set of roads: to read, not change.
 */
export function roadReach(draft: Pick<NeighbourhoodDraft, 'width' | 'depth' | 'roads' | 'open'>): Float32Array {
  const g = gridsOf(draft);
  if (g.reach) return g.reach;
  const { cols, rows } = useGrid(draft.width, draft.depth);
  const on = roadCells(draft.roads, cols, rows);
  const dist = new Float32Array(cols * rows).fill(Infinity);
  for (let k = 0; k < on.length; k++) if (on[k]) dist[k] = 0;
  const [a, b] = [USE_CELL, USE_CELL * Math.SQRT2];
  const relax = (k: number, n: number, w: number) => {
    if (dist[n] + w < dist[k]) dist[k] = dist[n] + w;
  };
  for (let j = 0; j < rows; j++) {
    for (let i = 0; i < cols; i++) {
      const k = j * cols + i;
      if (i > 0) relax(k, k - 1, a);
      if (j > 0) {
        relax(k, k - cols, a);
        if (i > 0) relax(k, k - cols - 1, b);
        if (i < cols - 1) relax(k, k - cols + 1, b);
      }
    }
  }
  for (let j = rows - 1; j >= 0; j--) {
    for (let i = cols - 1; i >= 0; i--) {
      const k = j * cols + i;
      if (i < cols - 1) relax(k, k + 1, a);
      if (j < rows - 1) {
        relax(k, k + cols, a);
        if (i < cols - 1) relax(k, k + cols + 1, b);
        if (i > 0) relax(k, k + cols - 1, b);
      }
    }
  }
  g.reach = dist;
  return dist;
}

// ---- the brush --------------------------------------------------------------------------------------

/** How far a point is from the segment `a`–`b`. */
function toSegment([px, pz]: Pt, [ax, az]: Pt, [bx, bz]: Pt): number {
  const [dx, dz] = [bx - ax, bz - az];
  const l2 = dx * dx + dz * dz;
  const u = l2 > 0 ? Math.max(0, Math.min(1, ((px - ax) * dx + (pz - az) * dz) / l2)) : 0;
  return pmath.hypot(px - ax - dx * u, pz - az - dz * u);
}

/**
 * `draft` with a stroke of the brush painted as `kind` (null: erased): every cell whose middle is
 * within half a stamp's width of the stroke (from each stamp to the next, as one line; the cell under
 * each stamp's middle at least), but never one no use goes on (`noUseCells`: water and its shore
 * strip, roads, off the world). `beside`: only cells within so many metres of a road ("Beside streets
 * only": 40). One stroke, one edit; refused if it changes no cell. It needs no settle (`needsSettle`).
 */
export function paintUses(draft: NeighbourhoodDraft, stamps: readonly UseStamp[], kind: UseKind | null, { beside }: { beside?: number } = {}): UsesPainted | { refused: string } {
  const open = draft.open;
  if (!open) return { refused: 'Zones are painted on a big world only.' };
  if (!stamps.length) return { refused: 'Nothing was painted.' };
  const layer = usesOfDraft(draft);
  const { cols, rows } = layer;
  const none = noUseCells(draft);
  const reach = beside === undefined ? null : roadReach(draft);
  const code = kind ? USE_CODE[kind] : 0;
  const was = unpackUses(layer);
  const cells = was.slice();
  let changed = 0;
  const paint = (k: number) => {
    if (none[k] || (reach && !(reach[k] <= beside!)) || cells[k] === code) return;
    cells[k] = code;
    changed++;
  };
  stamps.forEach((st, n) => {
    const prev = stamps[n - 1] ?? st;
    const r = Math.max(st.size, prev.size) / 2;
    const [a, b]: [Pt, Pt] = [
      [prev.x, prev.z],
      [st.x, st.z],
    ];
    const [i0, i1] = [Math.max(0, Math.floor((Math.min(a[0], b[0]) - r) / USE_CELL)), Math.min(cols - 1, Math.floor((Math.max(a[0], b[0]) + r) / USE_CELL))];
    const [j0, j1] = [Math.max(0, Math.floor((Math.min(a[1], b[1]) - r) / USE_CELL)), Math.min(rows - 1, Math.floor((Math.max(a[1], b[1]) + r) / USE_CELL))];
    for (let j = j0; j <= j1; j++) for (let i = i0; i <= i1; i++) if (toSegment([(i + 0.5) * USE_CELL, (j + 0.5) * USE_CELL], a, b) <= r) paint(j * cols + i);
    // (The cell under the stamp, however small the brush.)
    const [ci, cj] = [Math.floor(st.x / USE_CELL), Math.floor(st.z / USE_CELL)];
    if (ci >= 0 && cj >= 0 && ci < cols && cj < rows) paint(cj * cols + ci);
  });
  if (!changed) return { refused: kind ? 'There is nothing there to zone: zones go on the land beside the streets.' : 'There is no zone there to erase.' };
  return { draft: { ...draft, open: { ...open, uses: packUses(cells, cols, rows) } }, cleared: { lots: [] }, needsSettle: false };
}

/** Whether an edit's town must be settled on its land again: every edit's but a stroke of the zone brush (`paintUses`). */
export function needsSettle(edit: Edited | UsesPainted): boolean {
  return !('needsSettle' in edit && edit.needsSettle === false);
}

/** `settled` (a plan settled on its land, as the session shows it) with `plan`'s uses: what an edit of the uses alone shows, with no settle. */
export function withUses(settled: NeighbourhoodDraft, plan: NeighbourhoodDraft): NeighbourhoodDraft {
  if (!settled.open || settled.open.uses === plan.open?.uses) return settled;
  const { uses: _uses, ...open } = settled.open;
  return { ...settled, open: plan.open?.uses ? { ...open, uses: plan.open.uses } : open };
}
