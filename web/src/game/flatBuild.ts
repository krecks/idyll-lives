/**
 * Building in a flat, as the simulation allows it: inside the flat only. Its building's roof, outside, yard and
 * stairs are the building's, so Build mode's tools for them do nothing there (`ui/buy/BuildTools.svelte` greys them
 * out, saying why), and a wall, door or window goes only between two of the flat's own tiles (`buildmode.ts`).
 */

import { flatOf, type FlatInfo, type PlotInfo } from '../core/protocol';
import type { BuildTool } from '../ui/state.svelte';

/**
 * What the simulation refuses in a flat, by build tool, in its words (sim-core `not_a_flat` and its stairs rule): a
 * flat is built on inside only, its building's roof, outside, yard and stairs the building's.
 */
const IN_A_FLAT: Partial<Record<BuildTool, string>> = {
  roof: 'In a flat you build inside it only: no roof',
  garage: 'In a flat you build inside it only: no garage',
  porch: 'In a flat you build inside it only: no porch',
  blueprint: 'In a flat you build inside it only: no blueprints',
  move: 'In a flat you build inside it only: no moving rooms',
  stairs: 'A flat is all on one storey: no stairs in it',
  fence: "A flat has no garden of its own: its building's yard is the building's",
  gate: "A flat has no garden of its own: its building's yard is the building's",
};

/** Why the household can't use build tool `tool` at home `home` (a flat of `lots`), or null: a house refuses none. */
export function flatRefuses(tool: BuildTool, home: number | null | undefined, lots: { plots: readonly PlotInfo[]; flats?: readonly FlatInfo[] }): string | null {
  return home != null && flatOf(lots, home) ? (IN_A_FLAT[tool] ?? null) : null;
}

/** A wall, door or window in a flat goes between two of its own tiles: its outer walls are the building's (sim-core `edge_in_flat`). */
export const FLAT_WALLS = "A flat's outer walls are the building's: build inside it";
