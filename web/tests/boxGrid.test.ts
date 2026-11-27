// `render/shared/boxGrid.ts`: the box a point is in, found through the grid as testing every box in order finds it.
import { describe, expect, test } from 'bun:test';
import { BoxGrid, type Box } from '../src/render/shared/boxGrid';

describe('boxes on a grid', () => {
  test('the first box in order holding a point, edges in, as testing them all finds; none where none holds it', () => {
    // (Boxes across cells, overlapping, at the grid's lines and below 0; a seeded scatter of points.)
    const boxes: Box[] = [[0, 0, 10, 8], [5, 5, 70, 40], [64, 0, 64, 64], [-20, -20, -1, -1], [30, 30, 35, 36], [100, 100, 140, 101]];
    const grid = new BoxGrid(boxes, 32);
    let seed = 7;
    const rand = () => ((seed = (seed * 16807) % 2147483647) / 2147483647) * 180 - 30;
    const points: [number, number][] = [[10, 8], [64, 64], [64, 10], [-1, -1], [32, 32], [140, 101], [5, 5]];
    for (let i = 0; i < 2000; i++) points.push([rand(), rand()]);
    for (const [x, z] of points) {
      const first = boxes.findIndex((b) => x >= b[0] && x <= b[2] && z >= b[1] && z <= b[3]);
      expect(grid.find(boxes, x, z)).toBe(first);
    }
  });
});
