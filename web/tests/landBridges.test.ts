// Bridges the land's water brushes make in the game (`game/landBridges.ts`): water across a road bridges it there
// (its deck over the water and 2 m either side, along the road as it runs), or the stroke is refused with why; undo
// and redo put a road's list back. `bun test`.
import { describe, expect, test } from 'bun:test';
import type { RoadPlan } from '../src/core/protocol';
import { bridgesMeta, bridgesOrWhy, withBridges } from '../src/game/landBridges';

const W = 200;
const D = 200;
/** Water across the world between x `x0` and `x1`. */
function river(x0: number, x1: number): Uint8Array {
  const wet = new Uint8Array(W * D);
  for (let z = 0; z < D; z++) for (let x = x0; x < x1; x++) wet[z * W + x] = 1;
  return wet;
}
const road: RoadPlan = { points: Array.from({ length: 201 }, (_, k): [number, number] => [k, 100]), width: 7, ends: ['out', 'out'], kind: 'country', name: 'Mill Road' };

describe('bridges from water', () => {
  test('water across a road bridges it there; across a wider one, the stroke is refused', () => {
    const made = bridgesOrWhy([road], [], river(90, 102), W, D);
    if ('refused' in made) throw new Error(made.refused);
    expect(made.changes).toEqual([{ road: 0, before: [], after: [[88, 103.75]] }]);
    expect(made.roads[0].bridges).toEqual([[88, 103.75]]);
    // (Its deck is dry now: the same water asks nothing more.)
    const again = bridgesOrWhy(made.roads, [], river(90, 102), W, D);
    expect('refused' in again ? again.refused : again.changes).toEqual([]);
    const wide = bridgesOrWhy([road], [], river(40, 120), W, D);
    expect('refused' in wide ? wide.refused : '').toContain("Water can't cross Mill Road there");
    // A lot where the deck would stand: refused.
    const lot = bridgesOrWhy([road], [{ x: 92, z: 104, w: 10, d: 10 }], river(90, 102), W, D);
    expect('refused' in lot ? lot.refused : '').toContain('lot');
  });

  test('undo and redo put the list back; the command carries the whole list', () => {
    const changes = [{ road: 0, before: [], after: [[88, 104]] as [number, number][] }];
    expect(withBridges([road], changes, 'after')[0].bridges).toEqual([[88, 104]]);
    expect('bridges' in withBridges([{ ...road, bridges: [[1, 2]] }], changes, 'before')[0]).toBe(false);
    expect(bridgesMeta(changes, 'after')).toEqual({ 0: { bridges: [[88, 104]] } });
  });
});
