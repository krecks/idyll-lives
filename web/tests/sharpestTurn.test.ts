/**
 * How sharply a road turns (`game/roads.ts` `sharpestTurn`, the measure a big world's country
 * roads keep under 70°): nothing for a straight road, a corner's whole angle, and a steady bend
 * of radius r turning 20 / r over two spans of 20 m.
 */
import { describe, expect, test } from 'bun:test';
import { sharpestTurn, type Pt } from '../src/game/roads';

const line = (a: Pt, b: Pt, n: number): Pt[] => Array.from({ length: n + 1 }, (_, i): Pt => [a[0] + ((b[0] - a[0]) * i) / n, a[1] + ((b[1] - a[1]) * i) / n]);
/** A steady bend of radius `r`, `len` m long, a point a metre. */
const arc = (r: number, len: number): Pt[] => Array.from({ length: len + 1 }, (_, i): Pt => [r * Math.sin(i / r), r - r * Math.cos(i / r)]);

describe('sharpestTurn', () => {
  test('a straight road turns nowhere', () => {
    expect(sharpestTurn(line([0, 0], [100, 30], 100)).angle).toBeCloseTo(0);
  });

  test('a corner turns by its whole angle, where it is', () => {
    const pts = [...line([0, 0], [60, 0], 60), ...line([60, 0], [60, -60], 60).slice(1)];
    const { angle, s } = sharpestTurn(pts);
    expect(angle).toBeCloseTo(Math.PI / 2);
    expect(s).toBeCloseTo(60);
  });

  test('a hairpin turns by half a turn', () => {
    const pts = [...line([0, 0], [60, 0], 60), ...line([60, 0], [60, 4], 4).slice(1), ...line([60, 4], [0, 4], 60).slice(1)];
    expect(sharpestTurn(pts).angle).toBeGreaterThan((150 * Math.PI) / 180);
  });

  test('a steady bend turns 20 / r', () => {
    for (const r of [16, 25, 40]) expect(sharpestTurn(arc(r, 80)).angle).toBeCloseTo(20 / r, 2);
  });

  test('a road shorter than two spans has nothing to measure', () => {
    expect(sharpestTurn(line([0, 0], [30, 0], 30))).toEqual({ angle: 0, s: 0 });
  });
});
