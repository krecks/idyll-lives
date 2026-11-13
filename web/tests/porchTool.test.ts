/**
 * Build mode's Porch tool (`game/porch.ts`): a porch over a door stands on its outdoor side, three
 * tiles across and two out, where the ground is open and nothing stands at its posts; a piece goes
 * on whole tiles round the pointer, turned with R, and keeps to the simulation's rules (sim-core
 * `porches.rs`): outdoors on the lot, no wall through it, one of a kind a tile, a post or a
 * platform clear of where the car parks and a post clear of things.
 */
import { describe, expect, test } from 'bun:test';
import { innerEdges, pieceBoxes, pieceFits, piecePlan, piecePrice, porchAt, porchBoxes, porchPostTiles, porchTiles, type PorchSite } from '../src/game/porch';
import type { PorchPieceInfo } from '../src/core/protocol';

/** A lot 0..20 × 0..20 with a house 4..10 × 4..8 (indoors), a fence along x = 14 for z 10..11, a chair at (13, 9). */
function site(extra: Partial<PorchSite> = {}): PorchSite {
  return {
    plot: { x: 0, z: 0, w: 20, d: 20 },
    indoors: (x, z) => x >= 4 && x < 10 && z >= 4 && z < 8,
    walled: (axis, x, z) => axis === 'v' && x === 14 && z >= 10 && z < 12,
    thingAt: (x, z) => (x === 13 && z === 9 ? 'chair' : undefined),
    garage: () => false,
    pieces: [],
    drive: { x: 16, z: 0, w: 3, d: 8 },
    affords: () => true,
    ...extra,
  };
}

describe('the Porch tool', () => {
  test('a porch stands on the outdoor side of a door, three tiles across and two out', () => {
    // The house's front wall at z = 8, a door at x 6: outdoors is +z.
    const { porch, reason } = porchAt('h', 6, 8, true, site(), 600);
    expect(reason).toBeNull();
    expect(porch).toEqual({ axis: 'h', x: 6, z: 8, out: 1 });
    expect(porchTiles(porch!)).toEqual([
      [5, 8],
      [6, 8],
      [7, 8],
      [5, 9],
      [6, 9],
      [7, 9],
    ]);
    expect(porchPostTiles(porch!)).toEqual([
      [5, 9],
      [7, 9],
    ]);
    // Its back wall at z = 4: outdoors is -z.
    expect(porchAt('h', 6, 4, true, site(), 600).porch).toEqual({ axis: 'h', x: 6, z: 4, out: -1 });
    // A side door (x = 10): +x.
    expect(porchAt('v', 10, 5, true, site(), 600).porch).toEqual({ axis: 'v', x: 10, z: 5, out: 1 });
  });

  test("a porch needs a door, open ground, nothing at its posts and the money", () => {
    expect(porchAt('h', 6, 8, false, site(), 600).reason).toBe('Goes over a door');
    expect(porchAt('h', 6, 6, true, site(), 600).reason).toBe('That door is indoors');
    expect(porchAt('h', 6, 14, true, site(), 600).reason).toBe('Goes on a door into the house');
    // At the lot's edge: no ground for it.
    expect(porchAt('h', 6, 4, true, site({ plot: { x: 0, z: 3, w: 20, d: 17 } }), 600).reason).toBe('Needs open ground on your lot in front of the door');
    // A chair where a post goes, a fence across it.
    expect(porchAt('h', 12, 8, true, site({ indoors: (x, z) => x >= 4 && x < 14 && z >= 4 && z < 8 }), 600).reason).toBe('The chair is in the way');
    expect(porchAt('h', 13, 10, true, site({ indoors: (x, z) => x >= 12 && x < 16 && z < 10 }), 600).reason).toBe('A wall or a fence is in the way');
    expect(porchAt('h', 6, 8, true, site({ affords: () => false }), 600).reason).toBe('Not enough money');
  });

  test('a piece goes on whole tiles round the pointer, turned with R', () => {
    expect(piecePlan('post', 3.4, 12.8, 2, 2, 0)).toEqual({ kind: 'post', x: 3, z: 12, w: 1, d: 1, rot: 0 });
    // A canopy 2 across, 3 deep, facing +z: 2 × 3 tiles; turned a quarter (facing +x): 3 × 2.
    expect(piecePlan('canopy', 3, 12, 2, 3, 0)).toEqual({ kind: 'canopy', x: 2, z: 11, w: 2, d: 3, rot: 0 });
    expect(piecePlan('canopy', 3, 12, 2, 3, 1)).toEqual({ kind: 'canopy', x: 2, z: 11, w: 3, d: 2, rot: 1 });
    // No bigger than it can be.
    expect(piecePlan('canopy', 3, 12, 9, 9, 0)).toMatchObject({ w: 3, d: 6 });
    expect(piecePlan('step', 3, 12, 9, 9, 2)).toMatchObject({ w: 4, d: 4, rot: 2 });
    // Prices: a post, the others a tile.
    const prices = { porchPost: 60, porchCanopy: 90, porchStep: 30 };
    expect(piecePrice('post', 1, prices)).toBe(60);
    expect(piecePrice('canopy', 6, prices)).toBe(540);
    expect(piecePrice('step', 4, prices)).toBe(120);
  });

  test('pieces keep to the rules', () => {
    const fits = (p: Omit<PorchPieceInfo, 'plot'>, s = site()) => pieceFits(p, s, 0);
    expect(fits({ kind: 'step', x: 1, z: 12, w: 2, d: 2, rot: 0 })).toBe(true);
    expect(fits({ kind: 'step', x: 19, z: 12, w: 2, d: 2, rot: 0 })).toBe('Has to go on your lot');
    expect(fits({ kind: 'step', x: 5, z: 6, w: 2, d: 2, rot: 0 })).toBe('Goes outdoors, not in a room');
    expect(fits({ kind: 'canopy', x: 13, z: 10, w: 2, d: 1, rot: 0 })).toBe('A wall or a fence is in the way');
    // Along the fence, not across it: fine.
    expect(fits({ kind: 'canopy', x: 14, z: 10, w: 2, d: 2, rot: 0 })).toBe(true);
    const pieces: PorchPieceInfo[] = [{ plot: 0, kind: 'step', x: 1, z: 12, w: 2, d: 2, rot: 0 }];
    expect(fits({ kind: 'step', x: 2, z: 13, w: 1, d: 1, rot: 0 }, site({ pieces }))).toBe("There's a step platform there already");
    expect(fits({ kind: 'post', x: 2, z: 13, w: 1, d: 1, rot: 0 }, site({ pieces }))).toBe(true);
    expect(fits({ kind: 'canopy', x: 1, z: 12, w: 2, d: 2, rot: 0 }, site({ pieces }))).toBe(true);
    // Where the car parks: a canopy (a car port), not a post or a platform.
    expect(fits({ kind: 'post', x: 17, z: 3, w: 1, d: 1, rot: 0 })).toBe("That's where the car parks");
    expect(fits({ kind: 'step', x: 17, z: 3, w: 1, d: 1, rot: 0 })).toBe("That's where the car parks");
    expect(fits({ kind: 'canopy', x: 16, z: 2, w: 3, d: 3, rot: 0 })).toBe(true);
    expect(fits({ kind: 'post', x: 13, z: 9, w: 1, d: 1, rot: 0 })).toBe('The chair is in the way');
    expect(fits({ kind: 'post', x: 1, z: 1, w: 1, d: 1, rot: 0 }, site({ garage: (x, z) => x === 1 && z === 1 }))).toBe('The garage is in the way');
  });

  test('previews: a porch as its landing, posts and canopy; a piece as its slab or column', () => {
    const boxes = porchBoxes({ axis: 'h', x: 6, z: 8, out: 1 });
    expect(boxes).toHaveLength(4);
    // All of it outdoors, past the wall's face, and across the door's middle.
    for (const [, , z0, , , z1] of boxes) expect(Math.min(z0, z1)).toBeGreaterThanOrEqual(8.07 - 1e-9);
    expect(boxes[0][0] + boxes[0][3]).toBeCloseTo(13, 6);
    expect(pieceBoxes({ kind: 'post', x: 3, z: 4, w: 1, d: 1, rot: 0 })[0][4]).toBeCloseTo(2.34, 6);
    expect(pieceBoxes({ kind: 'step', x: 3, z: 4, w: 2, d: 1, rot: 0 })).toEqual([[3, 0, 4, 5, 0.08, 5]]);
    expect(innerEdges([[0, 0], [1, 0], [0, 1], [1, 1]]).map((e) => e.join(':')).sort()).toEqual(['h:0:1', 'h:1:1', 'v:1:0', 'v:1:1']);
  });
});
