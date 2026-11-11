// `render/shared/tilt.ts`: the rows the miniature focus keeps sharp, by the camera's tilt (`bun test`).
import { describe, expect, test } from 'bun:test';
import { DEFAULT_ORBIT, LENS, RADIUS_LIMITS } from '../src/render/shared/lens';
import { TILT, tiltRows, tiltShare } from '../src/render/shared/tilt';

describe('the miniature focus', () => {
  test('at the camera a game opens with, the band chosen from the pictures', () => {
    const [top, bottom] = tiltRows(DEFAULT_ORBIT.beta, LENS.fov);
    expect(top).toBeCloseTo(0.34, 2);
    expect(bottom).toBeCloseTo(0.72, 2);
  });

  test('the rows are where the ground is that far from the eye', () => {
    const beta = 0.86;
    const [top, bottom] = tiltRows(beta, LENS.fov);
    // A row's ray, and how far along it the ground is, in camera distances.
    const off = (row: number) => Math.cos(beta) / Math.cos(beta + Math.atan((0.5 - row) * 2 * Math.tan(LENS.fov / 2)));
    expect(off(top)).toBeCloseTo(TILT.far, 6);
    expect(off(bottom)).toBeCloseTo(TILT.near, 6);
    expect(off(0.5)).toBeCloseTo(1, 9);
  });

  test('looking straight down nothing blurs; looking low, the far ground blurs from further down the picture', () => {
    expect(tiltRows(0.02, LENS.fov)).toEqual([0.001, 0.999]);
    const rows = [0.5, 0.86, 1.1, 1.3].map((beta) => tiltRows(beta, LENS.fov));
    for (let i = 1; i < rows.length; i++) expect(rows[i][0]).toBeGreaterThan(rows[i - 1][0]);
    // (Always a band round the middle, whatever the camera.)
    for (const [top, bottom] of rows) {
      expect(top).toBeLessThan(0.5);
      expect(bottom).toBeGreaterThan(0.5);
    }
  });

  test('zoomed in it is gone, at the opening distance all there', () => {
    expect(tiltShare(RADIUS_LIMITS[0])).toBe(0);
    expect(tiltShare(DEFAULT_ORBIT.radius)).toBe(1);
    expect(tiltShare(13.5)).toBeCloseTo(0.5, 6);
  });
});
