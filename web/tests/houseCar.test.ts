/**
 * The town's cars where the street parked them (`shared/cars.ts` `houseCar`, which `game/town.ts`
 * writes into the town file's `cars`): on the driveway's inner end, along it, the same car on a lot
 * turned a quarter (turned with it), on about four houses in five, and on the player's always.
 */
import { describe, expect, test } from 'bun:test';
import { CARS, houseCar } from '../src/render/shared/cars';

type Rect = { x: number; z: number; w: number; d: number };

/** The town turned a quarter: (x, z) at (-z, x), as `streetFacing.test.ts` turns it. */
const rect = (r: Rect): Rect => ({ x: -(r.z + r.d), z: r.x, w: r.d, d: r.w });

// North of a street along x (its street on the max-z side: facing 2) and south of it (facing 0).
const lots: { plot: Rect; drive: Rect; facing: number }[] = [
  { plot: { x: 0, z: 0, w: 20, d: 20 }, drive: { x: 14.6, z: 9.6, w: 3, d: 10.4 }, facing: 2 },
  { plot: { x: 20, z: 0, w: 18, d: 20 }, drive: { x: 33.6, z: 10.6, w: 3, d: 9.4 }, facing: 2 },
  { plot: { x: 0, z: 26, w: 20, d: 20 }, drive: { x: 15.6, z: 26, w: 3, d: 10.4 }, facing: 0 },
  { plot: { x: 20, z: 26, w: 18, d: 20 }, drive: { x: 32.6, z: 26, w: 3, d: 9.4 }, facing: 0 },
];

describe('the town cars', () => {
  test('stand on the inner end of the driveway, along it', () => {
    for (const { plot, drive, facing } of lots) {
      const car = houseCar(plot, facing, drive, true)!;
      const { len, wid } = CARS[car.body];
      expect([car.len, car.wid]).toEqual([len, wid]);
      expect(car.x).toBeCloseTo(drive.x + drive.w / 2, 6);
      // 0.45 m from the end away from the street (`PARK_GAP`), its length along the driveway.
      const inner = facing === 2 ? drive.z : drive.z + drive.d;
      expect(Math.abs(car.z - inner)).toBeCloseTo(0.45 + len / 2, 6);
      expect(car.z - len / 2).toBeGreaterThanOrEqual(drive.z - 1e-6);
      expect(car.z + len / 2).toBeLessThanOrEqual(drive.z + drive.d + 1e-6);
      expect(Math.abs(Math.sin(car.yaw))).toBeLessThan(0.05);
    }
  });

  test('turned a quarter with their lot', () => {
    for (const { plot, drive, facing } of lots) {
      // Its street south of it turned is west of it (1), north of it east (3).
      const [a, b] = [houseCar(plot, facing, drive, true)!, houseCar(rect(plot), facing === 2 ? 1 : 3, rect(drive), true)!];
      expect(b.x).toBeCloseTo(-a.z, 6);
      expect(b.z).toBeCloseTo(a.x, 6);
      expect(b.yaw).toBeCloseTo(a.yaw - Math.PI / 2, 6);
      expect([b.body, b.paint]).toEqual([a.body, a.paint]);
      // On a house without one, none either way.
      expect(houseCar(rect(plot), facing === 2 ? 1 : 3, rect(drive)) === null).toBe(houseCar(plot, facing, drive) === null);
    }
  });

  test('on about four houses in five, on the player’s always', () => {
    let some = 0;
    const tries = 400;
    for (let i = 0; i < tries; i++) {
      const plot = { x: (i % 20) * 23, z: Math.floor(i / 20) * 31, w: 20, d: 20 };
      const drive = { x: plot.x + 14.6, z: plot.z + 9.6, w: 3, d: 10.4 };
      if (houseCar(plot, 2, drive)) some++;
      expect(houseCar(plot, 2, drive, true)).not.toBeNull();
    }
    expect(some / tries).toBeGreaterThan(0.7);
    expect(some / tries).toBeLessThan(0.9);
  });
});
