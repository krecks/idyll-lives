/**
 * Chairs pulled up to a table (`shared/tableTuck.ts`), as the PlayCanvas renderer draws the table:
 * its template's meshes (model space, `models.ts`) at its instance's matrix (and its moving parts'
 * at theirs), their vertices and triangles read back from the meshes' own copies.
 */

import type * as pc from './engine';
import type { PopTarget } from './objectMotion';
import type { Content } from '../../content/content';
import type { ObjectPlacement, WorldStructure } from '../../core/protocol';
import { tableRooms as sharedTableRooms, type DrawnGeometry, type TableRoom } from '../shared/tableTuck';

/**
 * The tables the seats in `world` face, with what of each is in front of the seat (`TableRoom`), by
 * seat id. `drawn`: an object's instance, if it is drawn; `floor`: the floor height of its row.
 */
export function tableRooms(world: WorldStructure, content: Content, drawn: (id: number) => PopTarget | undefined, floor: (o: ObjectPlacement) => number): Map<number, TableRoom> {
  return sharedTableRooms(
    world,
    content,
    (id) => {
      const d = drawn(id);
      if (!d) return undefined;
      const o = d.index * 16;
      return {
        x: d.matrices[o + 12],
        z: d.matrices[o + 14],
        geometry: () => {
          const out: DrawnGeometry[] = [];
          const add = (meshes: readonly { mesh: pc.Mesh }[], matrices: Float32Array) => {
            // (An instance is drawn at its own matrix: the template's meshes have none of their own.)
            const matrix = matrices.slice(o, o + 16);
            for (const { mesh } of meshes) {
              const positions: number[] = [];
              const indices: number[] = [];
              if (!mesh.getPositions(positions) || !mesh.getIndices(indices)) continue;
              out.push({ positions, indices, matrix });
            }
          };
          add(d.meshes, d.matrices);
          for (const part of d.moving?.parts ?? []) add(part.meshes, part.matrices);
          return out;
        },
      };
    },
    floor,
  );
}
