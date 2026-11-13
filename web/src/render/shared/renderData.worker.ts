/**
 * Builds the landscape's data off the main thread (`renderData.ts`): each world's land laid out
 * once (`Land`), then its grass field, its town's road mesh and its terrain shading as asked, sent
 * back as typed arrays.
 */

import type { WorldStructure } from '../../core/protocol';
import type { WorkerAnswer, WorkerAsk } from '../../core/lazyWorker';
import { Geo, type MeshData } from './geometry';
import { junctionsOf } from './junctions';
import { Land, type RenderDataAnswer, type RoadMeshData, type ToRenderData } from './renderData';
import { roadMeshes, SIDEWALK, townRoads } from './roads';

interface WorkerScope {
  onmessage: ((e: MessageEvent<WorkerAsk<ToRenderData>>) => void) | null;
  postMessage(message: WorkerAnswer<RenderDataAnswer>, transfer?: Transferable[]): void;
}
const scope = self as unknown as WorkerScope;

/** Worlds open, each laid out the first time it's asked for. */
const lands = new Map<number, { world: WorldStructure; land: Land | null }>();

function worldOf(id: number): WorldStructure {
  const open = lands.get(id);
  if (!open) throw new Error('no such land');
  return open.world;
}

function landOf(id: number): { world: WorldStructure; land: Land } {
  const open = lands.get(id);
  if (!open) throw new Error('no such land');
  open.land ??= new Land(open.world);
  return { world: open.world, land: open.land };
}

/** The town's road mesh, as the menu overview's ground builds it (`overview.ts` `buildGround`). */
function roadsOf(world: WorldStructure): RoadMeshData {
  const start = performance.now();
  const roads = townRoads(world);
  const { surface, marks } = roadMeshes(roads, null, 0, { marks: new Geo() }, 'surface', junctionsOf(world, roads), (ri) => roads[ri].width / 2 - SIDEWALK);
  return {
    surface: surface.meshData({ colors: true }),
    marks: marks && !marks.empty ? marks.meshData({ colors: true }) : null,
    ms: performance.now() - start,
  };
}

/** A mesh's buffers, to hand over rather than copy. */
function meshBuffers(mesh: MeshData | null): Transferable[] {
  if (!mesh) return [];
  return [mesh.positions.buffer, mesh.normals.buffer, mesh.uvs.buffer, mesh.indices.buffer, ...(mesh.colors ? [mesh.colors.buffer] : [])] as Transferable[];
}

scope.onmessage = (e) => {
  const { id, message: q } = e.data;
  if (q.type === 'open') {
    lands.set(q.land, { world: q.world, land: null });
    return;
  }
  if (q.type === 'close') {
    lands.delete(q.land);
    return;
  }
  void (async () => {
    try {
      if (q.type === 'roads') {
        const roads = roadsOf(worldOf(q.land));
        scope.postMessage({ id, answer: { type: 'roads', roads } }, [...meshBuffers(roads.surface), ...meshBuffers(roads.marks)]);
        return;
      }
      const { world, land } = landOf(q.land);
      if (q.type === 'grass') {
        const field = await land.grass(world, q.margin);
        scope.postMessage({ id, answer: { type: 'grass', field } }, [field.colour.buffer, field.ground.buffer]);
      } else {
        const colors = await land.shade(q.positions);
        scope.postMessage({ id, answer: { type: 'shade', positions: q.positions, colors } }, [q.positions.buffer, colors.buffer]);
      }
    } catch (err) {
      scope.postMessage({ id, error: err instanceof Error ? err.message : String(err) });
    }
  })();
};
