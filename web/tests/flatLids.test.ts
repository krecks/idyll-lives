/**
 * The parts of the building in view that may not be shown open (`render/shared/flatLids.ts`), on the real town's
 * fixture: its flats, viewed as their building's lot, lidded on their storey with nothing drawn in them; and a place's
 * own ground floor under flats, lidded but for the flats' stair hall.
 */
import { describe, expect, test } from 'bun:test';
import { closedFlats, closedParts, flatLids, inClosed, placeFloor, viewedFlats, viewedLot } from '../src/render/shared/flatLids';
import { BUILDING, FLATS, realTown, STOREY } from './fixtures/realTown';

const rowOf = (k: number) => k * STOREY;

describe('the flats of the building in view', () => {
  const world = realTown();
  const lot = world.plots[BUILDING];
  const view = { x: lot.x, z: lot.z, w: lot.w, d: lot.d };

  test('are the viewed lot’s flats; another lot, or none, has none', () => {
    expect(viewedLot(world, view)?.id).toBe(BUILDING);
    expect(viewedFlats(world, view).map((f) => f.id)).toEqual([...FLATS]);
    expect(viewedFlats(world, { ...view, x: view.x + 1 })).toEqual([]);
    expect(viewedFlats(world, null)).toEqual([]);
  });

  test('those that may not be shown open are closed; without a rule none is', () => {
    const flats = viewedFlats(world, view);
    // (Ash's home, flat 1, and flat 3 where one of them is visiting; flats 2 and 4 not.)
    const open = new Set([1, 3]);
    expect(closedFlats(flats, (plot) => open.has(plot)).map((f) => f.id)).toEqual([2, 4]);
    expect(closedParts(world, lot, flats, (plot) => open.has(plot), rowOf).map((p) => p.plot)).toEqual([2, 4]);
    expect(closedParts(world, lot, flats, null, rowOf)).toEqual([]);
  });

  test('a flats building’s own ground (its hall and yard) is never closed', () => {
    const flats = viewedFlats(world, view);
    expect(closedParts(world, lot, flats, () => false, rowOf).map((p) => p.plot)).toEqual([...FLATS]);
    expect(placeFloor(world, lot, flats, rowOf)).toEqual([]);
  });

  test('a point in a closed flat is on its storey only, its rectangle on the ground’s rows', () => {
    const closed = closedParts(world, lot, viewedFlats(world, view), (plot) => plot !== 4, rowOf);
    // Flat 4: storey 1, x 15..22, z 8..17.
    expect(inClosed(closed, 18.5, 12.5, 1)).toBe(true);
    expect(inClosed(closed, 18.5, 12.5, 0)).toBe(false);
    expect(inClosed(closed, 8.5, 12.5, 1)).toBe(false);
    expect(inClosed(closed, 13.5, 12.5, 1)).toBe(false);
  });

  test('a lid over each closed flat at the top of its walls, by storey', () => {
    const lids = flatLids(closedParts(world, lot, viewedFlats(world, view), (plot) => plot === 1, rowOf), 2.8);
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

describe('a place with flats over it', () => {
  // The fixture's building as a shop under its two flats upstairs: its ground floor's rooms the shop's, the hall
  // between them (x 12..15) the flats' way up.
  const world = realTown();
  world.plots[BUILDING] = { ...world.plots[BUILDING], kind: 'shop', public: true };
  world.flats = world.flats!.filter((f) => f.storey === 1);
  const lot = world.plots[BUILDING];
  const view = { x: lot.x, z: lot.z, w: lot.w, d: lot.d };
  const flats = viewedFlats(world, view);

  test('its ground floor is its rooms there but the flats’ stair hall', () => {
    const rects = placeFloor(world, lot, flats, rowOf);
    const tiles = new Set<string>();
    for (const r of rects) for (let z = r.z; z < r.z + r.d; z++) for (let x = r.x; x < r.x + r.w; x++) tiles.add(`${x},${z}`);
    expect(tiles.size).toBe(2 * 7 * 9);
    expect(tiles.has('5,8') && tiles.has('21,16')).toBe(true);
    expect(tiles.has('13,12') || tiles.has('4,12') || tiles.has('13,17')).toBe(false);
    // (Each side's rooms one rectangle: the runs of the rows under each other merged.)
    expect(rects).toEqual([
      { x: 5, z: 8, w: 7, d: 9 },
      { x: 15, z: 8, w: 7, d: 9 },
    ]);
  });

  test('stays closed while its flats are open unless it may be shown itself; then the hall and the shop open', () => {
    // (The household at home in flat 3 upstairs: the flat opens, the shop under it does not.)
    const parts = closedParts(world, lot, flats, (plot) => plot === 3, rowOf);
    expect(parts.map((p) => [p.plot, p.storey])).toEqual([
      [4, 1],
      [BUILDING, 0],
    ]);
    expect(inClosed(parts, 8.5, 12.5, 0)).toBe(true);
    expect(inClosed(parts, 13.5, 12.5, 0)).toBe(false);
    // (Looked inside, or one of the household in it: open.)
    expect(closedParts(world, lot, flats, (plot) => plot === 3 || plot === BUILDING, rowOf).map((p) => p.plot)).toEqual([4]);
  });
});
