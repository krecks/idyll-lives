/**
 * The land brushes: what one stamp of a tool does to the height field (`land.ts`), as the player
 * drags it over the land, in step 1 and in Build mode's Land tab.
 *
 * STAGE 0 CONTRACT: the types are fixed; the Data stream (stage B1) builds `stamp` and fills in
 * `StrokeState`.
 */

import type { Land, LandRect } from './land';

/**
 * A land tool. Shape: `raise`, `lower`, `smooth`, `flatten`. Water: `lake` (or the sea pushed
 * inland, started on it), `river`, `dry`. Cover: `woods`, `meadow`, `fields`, `clear`. Mountains:
 * `peak`, `range`, `level`.
 */
export type LandTool = 'raise' | 'lower' | 'smooth' | 'flatten' | 'lake' | 'river' | 'dry' | 'woods' | 'meadow' | 'fields' | 'clear' | 'peak' | 'range' | 'level';

/** A tool as the palette set it: its radius (m), its strength (0..1) and, for `peak` and `range`, its height (m). */
export interface Brush {
  tool: LandTool;
  radius: number;
  strength: number;
  height?: number;
}

/** One stamp along a stroke: where (world m) and the seconds since the last one. */
export interface Stamp {
  x: number;
  z: number;
  dt: number;
}

/**
 * What a stroke keeps from one stamp to the next (the height at the press for `flatten`, the
 * water's level for `lake` and `river`, the dragged line for `range`, what it has touched).
 * Placeholder: the Data stream adds what the tools need.
 */
export interface StrokeState {
  brush: Brush;
  /** Everything the stroke has touched so far (samples), or null before its first stamp. */
  touched: LandRect | null;
}

/**
 * One stamp of `brush` at `at` on `land`: changes the land in place and returns the rectangle of
 * samples it changed (null: none). `free(i, j)` is how far sample (i, j) may change (0 on built
 * land, 1 well clear of it).
 */
export function stamp(_land: Land, _brush: Brush, _at: Stamp, _stroke: StrokeState, _free: (i: number, j: number) => number): LandRect | null {
  throw new Error('not built yet');
}
