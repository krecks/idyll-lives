// The lawn's map, engine-neutral (`render/shared/lawn.ts`): a plot mown and the land round it rough, the
// town's outermost tiles as the land beyond it, the foot of a wall darker (more on its shaded side), bare
// earth outside a door, at a path's end and under a swing, and the same town the same map twice.
import { describe, expect, test } from 'bun:test';
import type { WorldStructure } from '../src/core/protocol';
import { LAWN_MAP, lawnMap } from '../src/render/shared/lawn';

const [W, D] = [16, 14];

/** A 16 × 14 town: a plot from (2, 2) to (14, 12) with a 3 × 3 room at (5, 5), its door in the wall towards low z, a path to it and a swing. */
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
    plots: [{ id: 0, name: 'Home', x: 2, z: 2, w: 12, d: 10, public: false, entry: null, house: null }],
    exits: [],
    rooms,
    walls,
    openings: [{ axis: 'h', x: 6, z: 5, kind: 'door' }],
    fences: [{ axis: 'v', x: 12, z: 9, kind: 'gate' }],
    meta: { seed: 7, paths: [{ x: 10, z: 3, w: 1, d: 3 }] } as unknown as WorldStructure['meta'],
    meshes: {} as WorldStructure['meshes'],
  };
}

const at = (map: Uint8Array, x: number, z: number) => Array.from(map.subarray((z * W + x) * 4, (z * W + x) * 4 + 4));
const none = new Uint8Array(W * D);

describe('the lawn’s map', () => {
  test('a plot is mown, the land round it rough, the town’s edge as the land beyond', () => {
    const map = lawnMap(town(), none);
    expect(map.length).toBe(W * D * 4);
    expect(at(map, 3, 3)[3]).toBe(0);
    expect(at(map, 13, 11)[3]).toBe(0);
    expect(at(map, 1, 3)[3]).toBe(255);
    expect(at(map, 14, 12)[3]).toBe(255);
    for (const [x, z] of [[0, 0], [W - 1, 4], [5, D - 1], [0, 9]]) expect(at(map, x, z)).toEqual([128, 128, 0, 255]);
  });

  test('the foot of a wall is darker, more where its shade lies', () => {
    const bare = lawnMap({ ...town(), walls: [], openings: [] }, none);
    const map = lawnMap(town(), none);
    // (The wall along z = 8: the tile past it towards high z is in its shade, the one before z = 5 in the sun.)
    const shaded = at(bare, 7, 8)[0] - at(map, 7, 8)[0];
    const sunny = at(bare, 7, 4)[0] - at(map, 7, 4)[0];
    expect(sunny).toBeGreaterThan(20);
    expect(shaded).toBeGreaterThan(sunny);
    // Lusher too, and nothing two tiles off on the sunny side.
    expect(at(map, 7, 4)[1]).toBeLessThan(at(bare, 7, 4)[1]);
    expect(at(map, 7, 3)).toEqual(at(bare, 7, 3));
  });

  test('bare earth outside a door, at a gate, past a path’s ends and under a swing', () => {
    const map = lawnMap(town([{ id: 1, def: 'swingSet', x: 3, z: 9, rot: 0, w: 2, d: 2 }, { id: 2, def: 'sofa', x: 10, z: 9, rot: 0, w: 2, d: 1 }]), none);
    const earth = (x: number, z: number) => at(map, x, z)[2] / 255;
    // The door opens onto (6, 4); a little of it a tile further out, none indoors.
    expect(earth(6, 4)).toBeCloseTo(LAWN_MAP.door, 1);
    expect(earth(6, 3)).toBeGreaterThan(0);
    expect(earth(6, 3)).toBeLessThan(earth(6, 4));
    expect(earth(6, 5)).toBe(0);
    // The gate between (11, 9) and (12, 9).
    expect(earth(11, 9)).toBeGreaterThan(0.5);
    expect(earth(12, 9)).toBeGreaterThan(0.5);
    // The path from z = 3 to 6 at x = 10: past both its ends.
    expect(earth(10, 2)).toBeCloseTo(LAWN_MAP.path, 1);
    expect(earth(10, 6)).toBeCloseTo(LAWN_MAP.path, 1);
    // Under the swing, and less a tile round it; a sofa wears nothing.
    expect(earth(3, 9)).toBeCloseTo(LAWN_MAP.play, 1);
    expect(earth(2, 9)).toBeGreaterThan(0);
    expect(earth(2, 9)).toBeLessThan(earth(3, 9));
    expect(earth(10, 10)).toBe(0);
  });

  test('the same town, the same map; a thing moved, and only that changes', () => {
    const a = lawnMap(town([{ id: 1, def: 'raisedBed', x: 3, z: 9, rot: 0, w: 2, d: 1 }]), none);
    const b = lawnMap(town([{ id: 1, def: 'raisedBed', x: 3, z: 9, rot: 0, w: 2, d: 1 }]), none);
    expect(b).toEqual(a);
    // Into the same map again with the bed gone: as a town without it.
    const again = lawnMap(town(), none, b);
    expect(again).toBe(b);
    expect(again).toEqual(lawnMap(town(), none));
    expect(again).not.toEqual(a);
  });
});
