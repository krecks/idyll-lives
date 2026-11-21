// The street and the weather (`audio/outside.ts`; its loops are made in Rust, `crates/sound/src/outdoors.rs`, and
// tested there): a car is louder and higher the faster and the nearer it passes; the town's bed comes only with the town view; the leaves follow the gust.
import { describe, expect, test } from 'bun:test';
import { OUTSIDE } from '../src/audio/catalog';
import { carSound, leavesLevel, nearness, townLevel } from '../src/audio/outside';
import { DEFAULT_ORBIT, RADIUS_LIMITS, TOWN_FROM, TOWN_FULL, TOWN_MAX } from '../src/render/shared/lens';

describe('a car passing', () => {
  test('stood at a light it idles; the faster it goes the louder and higher, to its full speed', () => {
    const still = carSound(0, 1);
    expect(still.tyres).toBe(0);
    expect(still.engine).toBeCloseTo(OUTSIDE.car.engine * OUTSIDE.car.idle, 9);
    let last = still;
    for (let v = 2; v <= OUTSIDE.car.full; v += 2) {
      const s = carSound(v, 1);
      expect(s.engine).toBeGreaterThan(last.engine);
      expect(s.tyres).toBeGreaterThan(last.tyres);
      expect(s.engineRate).toBeGreaterThan(last.engineRate);
      expect(s.tyresRate).toBeGreaterThan(last.tyresRate);
      last = s;
    }
    // A country road's 60 m/s on screen is no louder or higher than a street's full speed.
    expect(carSound(60, 1)).toEqual(carSound(OUTSIDE.car.full, 1));
    expect(last.engine).toBeCloseTo(OUTSIDE.car.engine, 9);
    expect(last.tyres).toBeCloseTo(OUTSIDE.car.tyres, 9);
    expect(last.engineRate).toBeLessThan(1.5);
    // Half as near, half the level, the same pitch.
    expect(carSound(20, 0.5).engine).toBeCloseTo(carSound(20, 1).engine / 2, 9);
    expect(carSound(20, 0.5).engineRate).toBe(carSound(20, 1).engineRate);
  });

  test('it is nearest in the middle of the picture, half at the camera\'s reach, and the reach grows with the zoom', () => {
    const r = DEFAULT_ORBIT.radius;
    expect(nearness(0, r)).toBe(1);
    expect(nearness(OUTSIDE.car.reach * r, r)).toBeCloseTo(0.5, 9);
    for (let d = 5; d < 300; d += 5) expect(nearness(d, r)).toBeLessThan(nearness(d - 5, r));
    // Past about four reaches it is not heard (`drop`).
    expect(nearness(4 * OUTSIDE.car.reach * r, r)).toBeLessThan(OUTSIDE.car.drop);
    expect(nearness(40, RADIUS_LIMITS[1])).toBeGreaterThan(nearness(40, r));
    // Close in, the reach is never under `least` m: a car at the kerb is still heard from the kitchen's close-up.
    expect(nearness(OUTSIDE.car.least, RADIUS_LIMITS[0])).toBeCloseTo(0.5, 9);
  });
});

describe('the town and the wind', () => {
  test('the town\'s bed comes with the town view, and is lower at night', () => {
    for (const r of [RADIUS_LIMITS[0], DEFAULT_ORBIT.radius, RADIUS_LIMITS[1], TOWN_FROM]) expect(townLevel(r, 1)).toBe(0);
    let last = 0;
    for (let r = TOWN_FROM + 10; r <= TOWN_FULL; r += 10) {
      expect(townLevel(r, 1)).toBeGreaterThan(last);
      last = townLevel(r, 1);
    }
    expect(townLevel(TOWN_FULL, 1)).toBeCloseTo(OUTSIDE.town.level, 9);
    expect(townLevel(TOWN_MAX, 1)).toBeCloseTo(OUTSIDE.town.level, 9);
    expect(townLevel(TOWN_FULL, 0)).toBeCloseTo(OUTSIDE.town.level * OUTSIDE.town.night, 9);
  });

  test('the leaves follow the gust, are lower from indoors and from high up, and never gone', () => {
    const r = DEFAULT_ORBIT.radius;
    expect(leavesLevel(1, r, 0)).toBeCloseTo(OUTSIDE.leaves.level, 9);
    expect(leavesLevel(0, r, 0)).toBeCloseTo(OUTSIDE.leaves.level * OUTSIDE.leaves.calm, 9);
    for (let g = 0.1; g <= 1; g += 0.1) expect(leavesLevel(g, r, 0)).toBeGreaterThan(leavesLevel(g - 0.1, r, 0));
    expect(leavesLevel(1, r, 1)).toBeCloseTo(OUTSIDE.leaves.level * OUTSIDE.leaves.indoors, 9);
    expect(leavesLevel(1, TOWN_FULL, 0)).toBeCloseTo(OUTSIDE.leaves.level * OUTSIDE.leaves.far, 9);
    expect(leavesLevel(0, TOWN_MAX, 1)).toBeGreaterThan(0);
    // A gust that says nothing sensible is held to its range.
    expect(leavesLevel(7, r, 0)).toBe(leavesLevel(1, r, 0));
    expect(leavesLevel(-1, r, 0)).toBe(leavesLevel(0, r, 0));
  });
});
