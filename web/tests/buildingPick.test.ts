import { describe, expect, test } from 'bun:test';
import { rayPrism, rectFootprint } from '../src/render/shared/buildingPick';

const norm = (x: number, y: number, z: number) => {
  const l = Math.hypot(x, y, z);
  return { x: x / l, y: y / l, z: z / l };
};

describe('rayPrism', () => {
  const box = rectFootprint(10, -5, 20, 5);
  test('a ray down at a tall roof meets it, short of the ground behind it', () => {
    // From high up and to the side, aimed at the roof's middle (15, 12, 0).
    const o = { x: -15, y: 42, z: 0 };
    const d = norm(30, -30, 0);
    const t = rayPrism(o, d, box, 0, 12);
    expect(t).toBeCloseTo(30 * Math.SQRT2, 3);
    // (And the ground under the roof, past the house, is further.)
    expect(t).toBeLessThan(-o.y / d.y);
  });
  test('a flatter ray in through the wall facing it', () => {
    const o = { x: -15, y: 14, z: 0 };
    const d = norm(25, -5, 0);
    const t = rayPrism(o, d, box, 0, 12);
    expect(o.x + d.x * t).toBeCloseTo(10, 3);
    expect(o.y + d.y * t).toBeCloseTo(9, 3);
  });
  test('in through the top', () => {
    const t = rayPrism({ x: 15, y: 30, z: 0 }, { x: 0, y: -1, z: 0 }, box, 0, 12);
    expect(t).toBeCloseTo(18, 6);
  });
  test('a miss: past the side, or over the roof', () => {
    expect(rayPrism({ x: 0, y: 5, z: 20 }, norm(1, 0, 0), box, 0, 12)).toBe(Infinity);
    expect(rayPrism({ x: 0, y: 20, z: 0 }, norm(1, 0, 0), box, 0, 12)).toBe(Infinity);
    expect(rayPrism({ x: 0, y: 30, z: 0 }, norm(0, -1, 1), box, 0, 12)).toBe(Infinity);
  });
  test("an L's notch is no building", () => {
    // An L: the square 0..10 × 0..10 less its corner 5..10 × 5..10.
    const l: [number, number][] = [
      [0, 0],
      [10, 0],
      [10, 5],
      [5, 5],
      [5, 10],
      [0, 10],
    ];
    // Straight down into the notch: nothing; into the L's arm: its roof.
    expect(rayPrism({ x: 8, y: 20, z: 8 }, { x: 0, y: -1, z: 0 }, l, 0, 6)).toBe(Infinity);
    expect(rayPrism({ x: 2, y: 20, z: 8 }, { x: 0, y: -1, z: 0 }, l, 0, 6)).toBeCloseTo(14, 6);
    // Level through the notch from the far side: in through the notch's inner wall at x = 5.
    expect(rayPrism({ x: 20, y: 3, z: 8 }, { x: -1, y: 0, z: 0 }, l, 0, 6)).toBeCloseTo(15, 6);
  });
});
