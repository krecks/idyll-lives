/**
 * Shaping a world's land: strokes of the land brushes (`landBrush.ts`) over its height field, with
 * undo and redo per stroke, the built land held off, and what a stroke changed turned into what
 * the world keeps (the sim's water tiles, the zones the renderer and the map read) and its price.
 * The same in step 1 and in Build mode's Land tab.
 *
 * STAGE 0 CONTRACT: the signatures are fixed; the Data stream (stage B1) builds the bodies.
 */

import type { WorldZone } from '../core/protocol';
import type { Brush } from './landBrush';
import type { Land, LandPatch, LandRect } from './land';

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

export class LandEditor {
  /** Strokes to undo (the last last) and to redo: what each changed held before it. */
  private readonly undone: LandPatch[] = [];
  private readonly redone: LandPatch[] = [];

  /**
   * `land` is changed in place. `free(i, j)` is how far sample (i, j) may change: 0 on built land,
   * rising to 1 clear of it (the protection mask).
   */
  constructor(
    readonly land: Land,
    readonly free: (i: number, j: number) => number,
  ) {}

  /** Starts a stroke of `brush` at world point (`x`, `z`); returns the samples it changed. */
  begin(_brush: Brush, _x: number, _z: number): LandRect | null {
    throw new Error('not built yet');
  }

  /** The stroke dragged on to (`x`, `z`), `dt` seconds later; returns the samples it changed. */
  move(_x: number, _z: number, _dt: number): LandRect | null {
    throw new Error('not built yet');
  }

  /** Ends the stroke: what it changed held before it (pushed on the undo stack), or null if it changed nothing. */
  end(): LandPatch | null {
    throw new Error('not built yet');
  }

  /** Takes back the last stroke; returns the patch that redoes it (its rectangle is what changed), or null if there is none. */
  undo(): LandPatch | null {
    void this.undone;
    throw new Error('not built yet');
  }

  /** Puts back the last stroke taken back; returns the patch that undoes it again, or null if there is none. */
  redo(): LandPatch | null {
    void this.redone;
    throw new Error('not built yet');
  }

  /** Every stroke taken back, the land as it was first surveyed; returns what changed, or null if nothing had. */
  reset(): LandRect | null {
    throw new Error('not built yet');
  }

  /** The water within `rect` (samples) as the sim's 1 m tiles: 1 wet, row-major over the rectangle's tiles. */
  waterTiles(_rect: LandRect): Uint8Array {
    throw new Error('not built yet');
  }

  /** The woods, meadows, fields and water of the land as `meta.zones` (`landMasks.ts` `outlines`). */
  zonesOf(): WorldZone[] {
    throw new Error('not built yet');
  }

  /** What the change `patch` holds the old values of costs, at `prices` (negative: a refund). */
  cost(_patch: LandPatch, _prices: LandPrices): number {
    throw new Error('not built yet');
  }
}
