/**
 * Chairs pulled up to a table (`shared/tableTuck.ts`), as the Babylon renderer draws the table: its
 * template's meshes at its thin instance's matrix (and its moving parts' at theirs).
 */

import type { ObjectPlacement, WorldStructure } from '../../core/protocol';
import type { Content } from '../../content/content';
import { Matrix, VertexBuffer, type Mesh } from './core';
import { tableRooms as sharedTableRooms, type DrawnGeometry, type TableRoom } from '../shared/tableTuck';

/** An object as drawn: its template's meshes and where its instance matrix is (the renderer's `PlacedObject.pop`). */
export interface Drawn {
  meshes: Mesh[];
  matrices: Float32Array;
  index: number;
  moving?: { parts: { meshes: Mesh[]; matrices: Float32Array }[] };
}

/**
 * The tables the seats in `world` face, with what of each is in front of the seat (`TableRoom`), by
 * seat id. `drawn`: an object's instance, if it is drawn; `floor`: the floor height of its row.
 */
export function tableRooms(world: WorldStructure, content: Content, drawn: (id: number) => Drawn | undefined, floor: (o: ObjectPlacement) => number): Map<number, TableRoom> {
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
          const add = (meshes: readonly Mesh[], matrices: Float32Array) => {
            const instance = Matrix.FromArray(matrices, d.index * 16);
            for (const mesh of meshes) {
              const positions = mesh.getVerticesData(VertexBuffer.PositionKind);
              const indices = mesh.getIndices();
              if (!positions || !indices) continue;
              // (A thin instance is drawn at its own matrix, then its mesh's.)
              out.push({ positions, indices, matrix: instance.multiply(mesh.computeWorldMatrix(true)).asArray() });
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
