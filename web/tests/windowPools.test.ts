// `render/shared/gardenPools.ts` `windowPools`: where a lit room's windows throw its light on the garden (`bun test`).
import { describe, expect, test } from 'bun:test';
import type { Opening } from '../src/core/protocol';
import { backEdges, CURTAIN_LEAK, curtainsLetThrough, WINDOW_POOL, windowPools } from '../src/render/shared/gardenPools';

/** A 10 × 10 lot with one room (2) from (3, 3) to (6, 6), and a second (3) beside it from (7, 3) to (8, 6). */
function lot(openings: Opening[], depth = 10) {
  const width = 10;
  const rooms = new Array<number>(width * depth).fill(0);
  for (let z = 3; z <= 6; z++) {
    for (let x = 3; x <= 6; x++) rooms[z * width + x] = 2;
    for (let x = 7; x <= 8; x++) rooms[z * width + x] = 3;
  }
  return { width, depth, rooms, openings };
}
const window = (axis: 'h' | 'v', x: number, z: number): Opening => ({ kind: 'window', axis, x, z });
const all = () => true;

describe('the light a lit room throws on the garden through its windows', () => {
  test('half a pool at the foot of the wall under each window, on the garden side, following its room', () => {
    // (The room's front wall is the line z = 3, the garden towards -z; its left wall the line x = 3, the garden towards -x.)
    const pools = windowPools(lot([window('h', 4, 3), window('v', 3, 5)]), all, 0, all);
    expect(pools).toEqual([
      { x: 4.5, y: 0, z: 3 - WINDOW_POOL.near, radius: 0.5 + WINDOW_POOL.spread, depth: WINDOW_POOL.far, half: 'z-', strength: WINDOW_POOL.strength, room: 2 },
      { x: 3 - WINDOW_POOL.near, y: 0, z: 5.5, radius: WINDOW_POOL.far, depth: 0.5 + WINDOW_POOL.spread, half: 'x-', strength: WINDOW_POOL.strength, room: 2 },
    ]);
    // (The back wall, the line z = 7: the garden towards +z.)
    expect(windowPools(lot([window('h', 5, 7)]), all, 0, all)[0]).toMatchObject({ x: 5.5, z: 7 + WINDOW_POOL.near, half: 'z+', room: 2 });
  });

  test('a wide window is one pool; windows apart are two', () => {
    const wide = windowPools(lot([window('h', 3, 3), window('h', 4, 3), window('h', 6, 3)]), all, 0, all);
    expect(wide.map((p) => [p.x, p.radius])).toEqual([
      [4, 1 + WINDOW_POOL.spread],
      [6.5, 0.5 + WINDOW_POOL.spread],
    ]);
  });

  test('none for a window between two rooms, a door, another storey or another lot', () => {
    // (The wall between the two rooms is the line x = 7.)
    expect(windowPools(lot([window('v', 7, 4)]), all, 0, all)).toEqual([]);
    expect(windowPools(lot([{ kind: 'door', axis: 'h', x: 4, z: 3 }]), all, 0, all)).toEqual([]);
    expect(windowPools(lot([window('h', 4, 3)]), () => false, 0, all)).toEqual([]);
    expect(windowPools(lot([window('h', 4, 3)]), all, 0, () => false)).toEqual([]);
  });

  test('the ground storey is where its rows are: a pool is placed from its first row', () => {
    // (The same room on rows 10 to 19 of a lot whose ground storey starts at row 10.)
    const width = 10;
    const rooms = new Array<number>(width * 20).fill(0);
    for (let z = 13; z <= 16; z++) for (let x = 3; x <= 6; x++) rooms[z * width + x] = 2;
    const pools = windowPools({ width, depth: 20, rooms, openings: [window('h', 4, 13)] }, (z) => z >= 10, 10, all);
    expect(pools[0]).toMatchObject({ x: 4.5, half: 'z-' });
    expect(pools[0].z).toBeCloseTo(3 - WINDOW_POOL.near, 9);
  });

  test('curtains drawn over a window keep most of its light in', () => {
    // (Curtains hung in room 2 with their backs on the front wall, the line z = 3: over the first two of three windows.)
    const hung = { x: 3, z: 3, w: 2, d: 1, rot: 0 };
    expect(backEdges(hung)).toEqual(['h:3:3', 'h:4:3']);
    expect(backEdges({ x: 3, z: 5, w: 1, d: 1, rot: 3 })).toEqual(['v:4:5']);
    const curtains = new Map(backEdges(hung).map((e) => [e, 9]));
    const [pool] = windowPools(lot([window('h', 3, 3), window('h', 4, 3), window('h', 5, 3)]), all, 0, all, (e) => curtains.get(e));
    expect(pool.covers).toEqual([9, 9, -1]);
    expect(curtainsLetThrough(pool, new Set())).toBe(1);
    expect(curtainsLetThrough(pool, new Set([9]))).toBeCloseTo(1 - (2 / 3) * (1 - CURTAIN_LEAK), 9);
    // (A window with none over it has nothing to keep in.)
    const [bare] = windowPools(lot([window('h', 4, 3)]), all, 0, all, () => undefined);
    expect(bare.covers).toBeUndefined();
    expect(curtainsLetThrough(bare, new Set([9]))).toBe(1);
  });
});
