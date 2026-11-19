/**
 * The flats of the building in view, open or closed (engine-neutral; drawn by `pc/PlayCanvasRenderer.ts`). A flats
 * building (or a place with flats over it) is viewed as its lot, every flat of it on its own storey. A flat may be shown
 * open or not by the game's rule (`game/openLots.ts` `mayOpen`, asked of its plot id); one that may not keeps a lid
 * over it on the storey in view, a slab at the top of its walls, and nothing in it is drawn: its things, its food, the
 * residents in it.
 */

import type { FlatInfo, WorldStructure } from '../../core/protocol';
import { Geo } from './geometry';

/** A closed flat's lid: its colour (sRGB, a ceiling's plaster seen from above) and how thick (m). */
const LID_COLOUR = '#D6D1C7';
const LID_THICK = 0.15;

/** The flats of the lot viewed (`view`: its rectangle, as `Renderer.setWorld` has it); none for another lot or none. */
export function viewedFlats(world: Pick<WorldStructure, 'plots' | 'flats'>, view: { x: number; z: number; w: number; d: number } | null): FlatInfo[] {
  const lot = view && world.plots.find((p) => p.x === view.x && p.z === view.z && p.w === view.w && p.d === view.d);
  return lot ? (world.flats ?? []).filter((f) => f.of === lot.id) : [];
}

/** Those of `flats` that may not be shown open (`mayOpen` false for their plot id); none without a rule. */
export function closedFlats(flats: readonly FlatInfo[], mayOpen: ((plot: number) => boolean) | null): FlatInfo[] {
  return mayOpen ? flats.filter((f) => !mayOpen(f.id)) : [];
}

/** Whether ground point (`x`, `z`, on the ground's rows) on storey `k` is in one of `flats`. */
export function inFlats(flats: readonly FlatInfo[], x: number, z: number, k: number): boolean {
  for (const f of flats) if (f.storey === k && x >= f.x && x < f.x + f.w && z >= f.z && z < f.z + f.d) return true;
  return false;
}

/**
 * The lids of `flats` (those closed), by storey: on storey `k` a slab over each one's rectangle at the top of its walls
 * (`wall` m a storey), under the floor of the storey over it; in the lot's frame (the ground's rows, its floor at 0).
 */
export function flatLids(flats: readonly FlatInfo[], wall: number): Map<number, Geo> {
  const out = new Map<number, Geo>();
  for (const f of flats) {
    let geo = out.get(f.storey);
    if (!geo) out.set(f.storey, (geo = new Geo().color(LID_COLOUR)));
    const y = (f.storey + 1) * wall;
    geo.box(f.x, y, f.z, f.x + f.w, y + LID_THICK, f.z + f.d, 'ny');
  }
  return out;
}
