/**
 * The parts of the building in view that may not be shown open (engine-neutral; drawn by `pc/PlayCanvasRenderer.ts`). A
 * flats building (or a place with flats over it) is viewed as its lot, every flat of it on its own storey. A flat may
 * be shown open or not by the game's rule (`game/openLots.ts` `mayOpen`, asked of its plot id), and so may a place's
 * own ground floor under its flats (asked of the place's lot: its rooms, but the flats' stair hall, which is theirs). A
 * part that may not keeps a lid over it, a slab over its rooms, and nothing in it is drawn: its things, its food, the
 * residents in it.
 */

import type { FlatInfo, PlotInfo, WorldStructure } from '../../core/protocol';
import { Geo } from './geometry';

/** A closed part's lid: its colour (sRGB, a ceiling's plaster seen from above) and how thick (m). */
const LID_COLOUR = '#D6D1C7';
export const LID_THICK = 0.15;

/** A rectangle of ground tiles (x, z its corner on the ground's rows). */
export interface TileRect {
  x: number;
  z: number;
  w: number;
  d: number;
}

/** A part of the building in view that may not be shown open: a flat, or a place's ground floor; its plot, its storey, its tiles. */
export interface ClosedPart {
  plot: number;
  storey: number;
  rects: TileRect[];
}

/** The lot viewed (`view`: its rectangle, as `Renderer.setWorld` has it), if it is one. */
export function viewedLot(world: Pick<WorldStructure, 'plots'>, view: { x: number; z: number; w: number; d: number } | null): PlotInfo | undefined {
  return view ? world.plots.find((p) => p.x === view.x && p.z === view.z && p.w === view.w && p.d === view.d) : undefined;
}

/** The flats of the lot viewed (`view`: its rectangle, as `Renderer.setWorld` has it); none for another lot or none. */
export function viewedFlats(world: Pick<WorldStructure, 'plots' | 'flats'>, view: { x: number; z: number; w: number; d: number } | null): FlatInfo[] {
  const lot = viewedLot(world, view);
  return lot ? (world.flats ?? []).filter((f) => f.of === lot.id) : [];
}

/** Those of `flats` that may not be shown open (`mayOpen` false for their plot id); none without a rule. */
export function closedFlats(flats: readonly FlatInfo[], mayOpen: ((plot: number) => boolean) | null): FlatInfo[] {
  return mayOpen ? flats.filter((f) => !mayOpen(f.id)) : [];
}

/**
 * A place's ground floor under its flats (`lot`, not a flats building's, with `flats` over it): its rooms on the
 * ground, but the flats' stair hall (the room under the landings their doors open off: the hall is the same on every
 * storey), as rectangles of tiles. `rowOf`: the first row of storey `k`. None for a flats building or a lot with no
 * flats.
 */
export function placeFloor(world: Pick<WorldStructure, 'rooms' | 'width'>, lot: PlotInfo, flats: readonly FlatInfo[], rowOf: (k: number) => number): TileRect[] {
  if (lot.kind === 'flats' || !flats.length) return [];
  const W = world.width;
  const ground = rowOf(0);
  const room = (x: number, z: number) => world.rooms[(ground + z) * W + x] ?? 0;
  const hall = new Set<number>();
  for (const f of flats) {
    const z = f.door[1] - rowOf(f.storey);
    if (f.door[0] >= lot.x && f.door[0] < lot.x + lot.w && z >= lot.z && z < lot.z + lot.d) hall.add(room(f.door[0], z));
  }
  hall.delete(0);
  // (Row by row the runs of its tiles, then each run as tall as the rows under it have it alike.)
  const runs: TileRect[] = [];
  let above = new Map<string, TileRect>();
  for (let z = lot.z; z < lot.z + lot.d; z++) {
    const here = new Map<string, TileRect>();
    for (let x = lot.x; x < lot.x + lot.w; ) {
      const r = room(x, z);
      if (!r || hall.has(r)) {
        x++;
        continue;
      }
      let e = x + 1;
      while (e < lot.x + lot.w && room(e, z) && !hall.has(room(e, z))) e++;
      const key = `${x},${e}`;
      let run = above.get(key);
      if (run) run.d++;
      else runs.push((run = { x, z, w: e - x, d: 1 }));
      here.set(key, run);
      x = e;
    }
    above = here;
  }
  return runs;
}

/**
 * The parts of the building in view (`lot`, its `flats`) that may not be shown open now (`mayOpen` false for their
 * plot: each flat, and the lot itself for a place's ground floor under its flats, `placeFloor`); none without a rule.
 */
export function closedParts(world: Pick<WorldStructure, 'rooms' | 'width'>, lot: PlotInfo | undefined, flats: readonly FlatInfo[], mayOpen: ((plot: number) => boolean) | null, rowOf: (k: number) => number): ClosedPart[] {
  if (!mayOpen || !lot) return [];
  const out: ClosedPart[] = closedFlats(flats, mayOpen).map((f) => ({ plot: f.id, storey: f.storey, rects: [{ x: f.x, z: f.z, w: f.w, d: f.d }] }));
  if (lot.kind !== 'flats' && flats.length && !mayOpen(lot.id)) {
    const rects = placeFloor(world, lot, flats, rowOf);
    if (rects.length) out.push({ plot: lot.id, storey: 0, rects });
  }
  return out;
}

/** Whether ground point (`x`, `z`, on the ground's rows) on storey `k` is in one of `parts`. */
export function inClosed(parts: readonly ClosedPart[], x: number, z: number, k: number): boolean {
  for (const p of parts) {
    if (p.storey !== k) continue;
    for (const r of p.rects) if (x >= r.x && x < r.x + r.w && z >= r.z && z < r.z + r.d) return true;
  }
  return false;
}

/**
 * The lids of `parts` (those closed), by storey: on storey `k` a slab over each one's tiles at the top of its walls
 * (`wall` m a storey), in the lot's frame (the ground's rows, its floor at 0). Drawn lower as the walls are
 * (`PlayCanvasRenderer.applyStoreys`).
 */
export function flatLids(parts: readonly ClosedPart[], wall: number): Map<number, Geo> {
  const out = new Map<number, Geo>();
  for (const p of parts) {
    let geo = out.get(p.storey);
    if (!geo) out.set(p.storey, (geo = new Geo().color(LID_COLOUR)));
    const y = (p.storey + 1) * wall;
    for (const r of p.rects) geo.box(r.x, y, r.z, r.x + r.w, y + LID_THICK, r.z + r.d, 'ny');
  }
  return out;
}
