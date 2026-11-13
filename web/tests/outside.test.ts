// The street and the weather (`audio/outdoors.ts`, `audio/outside.ts`): the loops are what they say (an engine's
// hum on its firing's tone, a town's low rumble, bright leaves) and the same every time; a car is louder and higher
// the faster and the nearer it passes; the town's bed comes only with the town view; the leaves follow the gust.
import { describe, expect, test } from 'bun:test';
import { OUTSIDE } from '../src/audio/catalog';
import { OUTDOORS } from '../src/audio/outdoors';
import { carSound, leavesLevel, nearness, townLevel } from '../src/audio/outside';
import type { Stereo } from '../src/audio/water';
import { DEFAULT_ORBIT, RADIUS_LIMITS, TOWN_FROM, TOWN_FULL, TOWN_MAX } from '../src/render/shared/lens';

const RATE = 48000;

/** How bright a sound is: the root mean square of its sample-to-sample change over that of its samples, as a frequency (Hz). */
function brightness(d: Float32Array): number {
  let change = 0;
  let level = 0;
  for (let i = 1; i < d.length; i++) {
    change += (d[i] - d[i - 1]) ** 2;
    level += d[i] ** 2;
  }
  return (Math.sqrt(change / level) * RATE) / (2 * Math.PI);
}

/** How much of `d` is a tone at `hz` (the size of its part at that frequency, per sample). */
function tone(d: Float32Array, hz: number): number {
  let re = 0;
  let im = 0;
  for (let i = 0; i < d.length; i++) {
    re += d[i] * Math.cos((2 * Math.PI * hz * i) / RATE);
    im += d[i] * Math.sin((2 * Math.PI * hz * i) / RATE);
  }
  return Math.hypot(re, im) / d.length;
}

describe('the loops', () => {
  const engine = OUTDOORS.carEngine(RATE);
  const tyres = OUTDOORS.carTyres(RATE);
  const town = OUTDOORS.town(RATE) as Stereo;
  const leaves = OUTDOORS.leaves(RATE) as Stereo;

  test('the same every time, whole and under full scale', () => {
    expect(OUTDOORS.carEngine(RATE)).toEqual(engine);
    expect((OUTDOORS.leaves(RATE) as Stereo).right).toEqual(leaves.right);
    for (const d of [engine, tyres, town.left, town.right, leaves.left, leaves.right]) {
      let peak = 0;
      let sum = 0;
      for (const v of d) {
        peak = Math.max(peak, Math.abs(v));
        sum += v * v;
      }
      expect(Number.isFinite(sum)).toBe(true);
      expect(peak).toBeLessThanOrEqual(1);
      expect(Math.sqrt(sum / d.length)).toBeGreaterThan(0.1);
    }
  });

  test('an engine hums on its firing\'s tone and its overtones, a low sound', () => {
    // 55 Hz and its multiples stand out of what lies between them.
    for (const k of [1, 2, 3]) expect(tone(engine, 55 * k)).toBeGreaterThan(8 * tone(engine, 55 * k + 27));
    expect(brightness(engine)).toBeLessThan(700);
    // Whole turns in the loop: it meets itself at the seam.
    expect(Math.abs(engine[0] - engine[engine.length - 1])).toBeLessThan(0.02);
  });

  test('tyres hiss, a town rumbles, leaves are bright; what is all round differs side to side', () => {
    expect(brightness(tyres)).toBeGreaterThan(1500);
    expect(brightness(town.left)).toBeLessThan(700);
    expect(brightness(leaves.left)).toBeGreaterThan(3500);
    for (const s of [town, leaves]) {
      let cross = 0;
      let own = 0;
      for (let i = 0; i < s.left.length; i++) {
        cross += s.left[i] * s.right[i];
        own += s.left[i] ** 2;
      }
      expect(Math.abs(cross / own)).toBeLessThan(0.2);
    }
  });
});

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
