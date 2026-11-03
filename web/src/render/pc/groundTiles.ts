/**
 * A big world's ground tiles (`nature.ts` `terrainTiles`): each has three levels of detail and draws
 * one of them, chosen by distance (`frameTiles`). Apart, so it can be checked on the engine's null
 * device without the rest of the landscape.
 */

import type * as pc from './engine';

/** A ground tile as `terrainTiles` builds it: its draw, its meshes by level, where its middle is and the level drawn. */
export interface GroundTile {
  mi: pc.MeshInstance;
  meshes: pc.Mesh[];
  x: number;
  z: number;
  lod: number;
}

/**
 * Destroys the ground tiles' levels no draw holds (all but the one each draws now, which goes with its
 * entity): left to themselves, about 40 MB of a big world's vertex buffers with every new game or load.
 */
export function disposeTiles(tiles: readonly GroundTile[]): void {
  for (const t of tiles) for (const m of t.meshes) if (m !== t.mi.mesh) m.destroy();
}
