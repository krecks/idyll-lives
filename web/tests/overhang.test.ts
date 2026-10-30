// The tiles a thing reaches over beside its own (content `overhang`: the arc lamp's shade), as build mode and the
// house generator find them: the same as the simulation's `ObjectInstance::overhang_tiles` and `overhang_edges`
// (`crates/sim-core/tests/sim/layers.rs` `an_arc_lamp_reaches_over_the_sofa_in_front_of_it`).
import { describe, expect, test } from 'bun:test';
import { overhangEdges, overhangTiles } from '../src/game/surfaces';

describe('overhangTiles', () => {
  test('the tile in front turns with the thing', () => {
    // rot 0 faces +z, 1 +x, 2 -z, 3 -x.
    expect([0, 1, 2, 3].map((rot) => overhangTiles([[0, 1]], 5, 5, rot))).toEqual([[[5, 6]], [[6, 5]], [[5, 4]], [[4, 5]]]);
  });
  test('its right turns with it too', () => {
    expect([0, 1, 2, 3].map((rot) => overhangTiles([[1, 0]], 5, 5, rot))).toEqual([[[6, 5]], [[5, 4]], [[4, 5]], [[5, 6]]]);
  });
  test('none without an overhang', () => {
    expect(overhangTiles(undefined, 5, 5, 0)).toEqual([]);
  });
});

describe('overhangEdges', () => {
  test('along the row, then the column', () => {
    expect(overhangEdges([[7, 6]], 5, 5)).toEqual([
      [[5, 5], [6, 5]],
      [[6, 5], [7, 5]],
      [[7, 5], [7, 6]],
    ]);
  });
});
