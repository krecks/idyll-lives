// `render/shared/environment.ts` `sunTurned`: a lighting step turns the sun (draws its map again) only past 0.25°.
import { describe, expect, test } from 'bun:test';
import { sunTurned } from '../src/render/shared/environment';

const way = (deg: number) => ({ x: Math.sin((deg * Math.PI) / 180), y: -Math.cos((deg * Math.PI) / 180), z: 0 });

describe("the sun's turn", () => {
  test('held under a quarter of a degree, turned past it, and always from no way yet', () => {
    expect(sunTurned(way(40.2), way(40))).toBe(false);
    expect(sunTurned(way(40.3), way(40))).toBe(true);
    expect(sunTurned(way(40), way(40))).toBe(false);
    expect(sunTurned(way(40), { x: NaN, y: NaN, z: NaN })).toBe(true);
  });
});
