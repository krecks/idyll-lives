// The ground's shade round what stands on it, engine-neutral (`render/shared/groundShade.ts`): darkest at the
// foot of an outside wall and fading out, none in the open, a fence's narrower than a wall's, a thing's round
// its footprint and a tree's round its trunk, nothing from a thing indoors or on a top.
import { describe, expect, test } from 'bun:test';
import type { WorldStructure } from '../src/core/protocol';
import { GROUND_SHADE, groundShade } from '../src/render/shared/groundShade';

const [W, D, N] = [16, 14, 4];

/** A 16 × 14 town with a 3 × 3 room at (5, 5) and a fence along x = 12 from z = 9 to 11. */
function town(objects: Partial<WorldStructure['objects'][number]>[] = []): WorldStructure {
  const rooms = new Array(W * D).fill(0);
  for (let z = 5; z < 8; z++) for (let x = 5; x < 8; x++) rooms[z * W + x] = 1;
  const walls: WorldStructure['walls'] = [];
  for (let x = 5; x < 8; x++) walls.push({ axis: 'h', x, z: 5 }, { axis: 'h', x, z: 8 });
  for (let z = 5; z < 8; z++) walls.push({ axis: 'v', x: 5, z }, { axis: 'v', x: 8, z });
  return {
    version: 1,
    lotVersion: 1,
    mode: 'living',
    width: W,
    depth: D,
    objects: objects as WorldStructure['objects'],
    sims: [],
    households: [],
    plots: [],
    exits: [],
    rooms,
    walls,
    openings: [],
    fences: [
      { axis: 'v', x: 12, z: 9, kind: 'fence' },
      { axis: 'v', x: 12, z: 10, kind: 'fence' },
    ],
    meta: null,
    meshes: {} as WorldStructure['meshes'],
  };
}

/** The shade at a point (m). */
const at = (map: Uint8Array, x: number, z: number) => map[Math.floor(z * N) * W * N + Math.floor(x * N)] / 255;

describe('the ground’s shade', () => {
  test('darkest at the foot of an outside wall, fading out, none in the open', () => {
    const map = groundShade(town(), N);
    expect(map.length).toBe(W * N * D * N);
    const [strength, reach] = GROUND_SHADE.wall;
    expect(at(map, 6.5, 8.3)).toBeGreaterThan(strength * 0.7);
    expect(at(map, 6.5, 8.3)).toBeLessThanOrEqual(strength + 0.01);
    expect(at(map, 6.5, 8.9)).toBeLessThan(at(map, 6.5, 8.3));
    expect(at(map, 6.5, 8.9)).toBeGreaterThan(0);
    expect(at(map, 6.5, 8 + reach + 0.5)).toBe(0);
    expect(at(map, 2, 2)).toBe(0);
  });

  test('a fence shades less far than a wall', () => {
    const map = groundShade(town(), N);
    expect(at(map, 12.2, 10)).toBeGreaterThan(0.2);
    expect(at(map, 12 + GROUND_SHADE.fence[1] + 0.3, 10)).toBe(0);
    expect(GROUND_SHADE.fence[1]).toBeLessThan(GROUND_SHADE.wall[1]);
  });

  test('a thing shades the ground round its footprint, a tree round its trunk', () => {
    const chair = { id: 1, def: 'chairGarden', x: 2, z: 11, w: 1, d: 1, rot: 0 };
    const tree = { id: 2, def: 'garden.maple', x: 13, z: 2, w: 2, d: 2, rot: 0 };
    const map = groundShade(town([chair, tree]), N);
    expect(at(map, 2.5, 11.5)).toBeGreaterThan(GROUND_SHADE.thing[0] * 0.9);
    expect(at(map, 3.1, 11.5)).toBeGreaterThan(0);
    expect(at(map, 2.5 + 0.5 + GROUND_SHADE.thing[1] + 0.2, 11.5)).toBe(0);
    // The tree: its trunk in the middle of its two tiles.
    expect(at(map, 14, 3)).toBeGreaterThan(GROUND_SHADE.trunk[0] * 0.9);
    expect(at(map, 14 + GROUND_SHADE.trunkRadius + GROUND_SHADE.trunk[1] + 0.3, 3)).toBe(0);
  });

  test('nothing from a thing indoors or standing on a top', () => {
    const bare = groundShade(town(), N);
    const indoors = { id: 1, def: 'sofa', x: 6, z: 6, w: 1, d: 1, rot: 0 };
    const onTop = { id: 2, def: 'vase', x: 2, z: 11, w: 1, d: 1, rot: 0, lift: 0.8 };
    expect(Buffer.from(groundShade(town([indoors, onTop]), N)).equals(Buffer.from(bare))).toBe(true);
  });

  test('made again into the same bytes', () => {
    const first = groundShade(town(), N);
    const copy = first.slice();
    expect(groundShade(town(), N, first)).toBe(first);
    expect(Buffer.from(first).equals(Buffer.from(copy))).toBe(true);
  });
});
