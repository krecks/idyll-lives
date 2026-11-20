// Step 2's zone brush (`game/edit/uses.ts` `paintUses`): a stroke paints the cells within half a stamp's
// width of it, from stamp to stamp, but never the water with its shore strip, a road or off the world;
// erasing clears them again; "Beside streets only" keeps within its reach of a road; a stroke that changes
// nothing is refused; and the edit says it needs no settle (`needsSettle`), the session showing the plan it
// settled last with the new uses (`withUses`).
import { describe, expect, test } from 'bun:test';
import { needsSettle, noUseCells, paintUses, roadReach, withUses } from '../src/game/edit/uses';
import { rle } from '../src/game/landMasks';
import { SHORE_GAP } from '../src/game/macro';
import { nearest } from '../src/game/roads';
import type { NeighbourhoodDraft } from '../src/game/town';
import { unpackUses, USE_CELL, USE_CODE, useAt } from '../src/game/uses';

const [W, D] = [128, 96];
/** A pond's tiles (x 88 to 120, z 56 to 88). */
const pond = new Uint8Array(W * D);
for (let z = 56; z < 88; z++) for (let x = 88; x < 120; x++) pond[z * W + x] = 1;

/** A little world: a road across it at z = 30, a pond, one area. */
function world(): NeighbourhoodDraft {
  return {
    name: 'Brushton',
    width: W,
    depth: D,
    slots: [],
    roads: [{ points: [[-10, 30], [W + 10, 30]], width: 6, ends: ['out', 'out'], kind: 'street', area: 0, name: 'High Street' }],
    greens: [],
    households: [],
    open: { areas: [{ name: 'Brushton', kind: 'hamlet', x: 10, z: 10, w: 100, d: 60 }], zones: [], water: rle(pond), pads: {}, parking: [] },
  };
}
const painted = (e: ReturnType<typeof paintUses>): NeighbourhoodDraft => {
  if ('refused' in e) throw new Error(`refused: ${e.refused}`);
  return e.draft;
};

describe('the zone brush', () => {
  test('a stroke over everything paints the land, never the water, its shore strip, the road or off the world', () => {
    const d = world();
    // (Stamps all over the world and past its edges.)
    const stamps = [];
    for (let z = -20; z <= D + 20; z += 16) for (let x = -20; x <= W + 20; x += 16) stamps.push({ x, z, size: 40 });
    const after = painted(paintUses(d, stamps, 'homes'));
    const layer = after.open!.uses!;
    const cells = unpackUses(layer);
    const road = d.roads[0];
    let homes = 0;
    for (let j = 0; j < layer.rows; j++) {
      for (let i = 0; i < layer.cols; i++) {
        const [x, z] = [(i + 0.5) * USE_CELL, (j + 0.5) * USE_CELL];
        if (!cells[j * layer.cols + i]) continue;
        homes++;
        // Off the water and its strip of bank, off the road's middle by half its width, on the world.
        const [tx, tz] = [Math.floor(x), Math.floor(z)];
        for (let dz = -SHORE_GAP; dz <= SHORE_GAP; dz++) for (let dx = -SHORE_GAP; dx <= SHORE_GAP; dx++) expect(pond[(tz + dz) * W + tx + dx] ?? 0).toBe(0);
        expect(nearest(road.points, x, z).dist).toBeGreaterThan(road.width / 2);
        expect(x < W && z < D).toBe(true);
      }
    }
    expect(homes).toBeGreaterThan(400);
    // (What no use goes on is all that was left.)
    const none = noUseCells(d);
    for (let k = 0; k < cells.length; k++) expect(cells[k] === USE_CODE.homes).toBe(!none[k]);
    expect(useAt(layer, 100, 70)).toBeNull();
    expect(useAt(layer, 50, 30)).toBeNull();
    expect(useAt(layer, 50, 50)).toBe('homes');
  });

  test('a stroke paints from stamp to stamp, as wide as its stamps; erasing takes it off again', () => {
    const d = world();
    const after = painted(paintUses(d, [{ x: 12, z: 50, size: 8 }, { x: 70, z: 50, size: 8 }], 'shops'));
    const layer = after.open!.uses!;
    for (let x = 14; x <= 68; x += 4) expect(useAt(layer, x, 50)).toBe('shops');
    expect(useAt(layer, 40, 62)).toBeNull();
    // (A one-cell brush still paints the cell under its stamp.)
    const dab = painted(paintUses(d, [{ x: 41, z: 61, size: 1 }], 'green'));
    expect(useAt(dab.open!.uses!, 41, 61)).toBe('green');
    const erased = painted(paintUses(after, [{ x: 40, z: 50, size: 16 }], null));
    expect(useAt(erased.open!.uses!, 40, 50)).toBeNull();
    expect(useAt(erased.open!.uses!, 14, 50)).toBe('shops');
    // (Nothing to erase, nothing to paint: refused.)
    expect('refused' in paintUses(d, [{ x: 40, z: 50, size: 16 }], null)).toBe(true);
    expect('refused' in paintUses(d, [{ x: 100, z: 70, size: 8 }], 'homes')).toBe(true);
    expect('refused' in paintUses(after, [{ x: 30, z: 50, size: 4 }], 'shops')).toBe(true);
  });

  test('beside streets only keeps within its reach of a road', () => {
    const d = world();
    const after = painted(paintUses(d, [{ x: 40, z: 40, size: 200 }], 'rows', { beside: 20 }));
    const layer = after.open!.uses!;
    const reach = roadReach(d);
    const cells = unpackUses(layer);
    let some = 0;
    for (let k = 0; k < cells.length; k++) {
      if (!cells[k]) continue;
      some++;
      expect(reach[k]).toBeLessThanOrEqual(20);
    }
    expect(some).toBeGreaterThan(20);
    expect(useAt(layer, 40, 90)).toBeNull();
  });

  test('a stroke needs no settle: the plan settled last is shown with its uses', () => {
    const d = world();
    const e = paintUses(d, [{ x: 40, z: 50, size: 16 }], 'public');
    expect('needsSettle' in e && e.needsSettle).toBe(false);
    expect(needsSettle(e)).toBe(false);
    expect(needsSettle({ draft: d, cleared: { lots: [] } })).toBe(true);
    const plan = painted(e);
    // Its roads, lots and land as they were: the same objects.
    expect(plan.roads).toBe(d.roads);
    expect(plan.slots).toBe(d.slots);
    const shown = withUses(d, plan);
    expect(shown.open!.uses).toBe(plan.open!.uses);
    expect(shown.roads).toBe(d.roads);
    expect(withUses(shown, plan)).toBe(shown);
  });
});
