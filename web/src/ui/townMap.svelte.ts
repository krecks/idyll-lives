/**
 * What the in-game town map (`TownPanel.svelte`) draws besides the plots: the town's size and its
 * presentation data (roads, greens, a big world's zones and areas), and live positions. The game
 * session fills it in (docs/design/open-world.md; the render branch wires it up): `world` when the
 * world structure arrives, `source` once per game.
 */

import type { WorldMeta } from '../core/protocol';

/** Live things on the map, asked for about twice a second while it is open. */
export interface TownMapSource {
  /** Residents outdoors and in, town tiles (`mine`: the player's household). */
  dots?(): { x: number; z: number; mine: boolean }[];
  /** Where the camera is looking (town tiles). */
  camera?(): [number, number] | null;
}

class TownMapState {
  /** The town's ground (tiles) and its presentation data; null outside a game (the map then works from the plots). */
  world = $state.raw<{ width: number; depth: number; meta: WorldMeta | null } | null>(null);
  source: TownMapSource = {};
}

export const townMap = new TownMapState();
