/**
 * The flats of the building in view (`render/shared/flatLids.ts`), on the real town's fixture: viewed as their
 * building's lot, those that may not be shown open lidded on their storey, nothing drawn in them.
 */
import { describe, expect, test } from 'bun:test';
import { closedFlats, flatLids, inFlats, viewedFlats } from '../src/render/shared/flatLids';
import { BUILDING, FLATS, realTown } from './fixtures/realTown';

describe('the flats of the building in view', () => {
  const world = realTown();
  const lot = world.plots[BUILDING];
  const view = { x: lot.x, z: lot.z, w: lot.w, d: lot.d };

  test('are the viewed lot’s flats; another lot, or none, has none', () => {
    expect(viewedFlats(world, view).map((f) => f.id)).toEqual([...FLATS]);
    expect(viewedFlats(world, { ...view, x: view.x + 1 })).toEqual([]);
    expect(viewedFlats(world, null)).toEqual([]);
  });

  test('those that may not be shown open are closed; without a rule none is', () => {
    const flats = viewedFlats(world, view);
    // (Ash's home, flat 1, and flat 3 where one of them is visiting; flats 2 and 4 not.)
    const open = new Set([1, 3]);
    expect(closedFlats(flats, (plot) => open.has(plot)).map((f) => f.id)).toEqual([2, 4]);
    expect(closedFlats(flats, null)).toEqual([]);
  });

  test('a point in a closed flat is on its storey only, its rectangle on the ground’s rows', () => {
    const closed = closedFlats(viewedFlats(world, view), (plot) => plot !== 4);
    // Flat 4: storey 1, x 15..22, z 8..17.
    expect(inFlats(closed, 18.5, 12.5, 1)).toBe(true);
    expect(inFlats(closed, 18.5, 12.5, 0)).toBe(false);
    expect(inFlats(closed, 8.5, 12.5, 1)).toBe(false);
    expect(inFlats(closed, 13.5, 12.5, 1)).toBe(false);
  });

  test('a lid over each closed flat at the top of its walls, by storey', () => {
    const lids = flatLids(closedFlats(viewedFlats(world, view), (plot) => plot === 1), 2.8);
    expect([...lids.keys()].sort()).toEqual([0, 1]);
    const box = (k: number) => {
      const p = lids.get(k)!.pos;
      const [xs, ys, zs] = [0, 1, 2].map((a) => p.filter((_, i) => i % 3 === a));
      return [Math.min(...xs), Math.max(...xs), Math.min(...ys), Math.min(...zs), Math.max(...zs)];
    };
    // Storey 0: flat 2 (x 15..22) over its walls' 2.8 m; storey 1: flats 3 and 4 (x 5..22) over 5.6 m.
    expect(box(0)).toEqual([15, 22, 2.8, 8, 17]);
    expect(box(1)).toEqual([5, 22, 5.6, 8, 17]);
  });
});
