/**
 * Builds the landscape's data off the main thread (`renderData.ts`): each world's land laid out
 * once (`Land`), then its grass field and terrain shading as asked, sent back as typed arrays.
 */

import type { WorldStructure } from '../../core/protocol';
import { Land, type FromRenderData, type ToRenderData } from './renderData';

interface WorkerScope {
  onmessage: ((e: MessageEvent<ToRenderData>) => void) | null;
  postMessage(message: FromRenderData, transfer?: Transferable[]): void;
}
const scope = self as unknown as WorkerScope;

/** Worlds open, each laid out the first time it's asked for. */
const lands = new Map<number, { world: WorldStructure; land: Land | null }>();

function landOf(id: number): { world: WorldStructure; land: Land } {
  const open = lands.get(id);
  if (!open) throw new Error('no such land');
  open.land ??= new Land(open.world);
  return { world: open.world, land: open.land };
}

scope.onmessage = (e) => {
  const q = e.data;
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
      const { world, land } = landOf(q.land);
      if (q.type === 'grass') {
        const field = await land.grass(world, q.margin);
        scope.postMessage({ id: q.id, type: 'grass', field }, [field.colour.buffer, field.ground.buffer]);
      } else {
        const colors = await land.shade(q.positions);
        scope.postMessage({ id: q.id, type: 'shade', positions: q.positions, colors }, [q.positions.buffer, colors.buffer]);
      }
    } catch (err) {
      scope.postMessage({ id: q.id, type: 'error', message: err instanceof Error ? err.message : String(err) });
    }
  })();
};
