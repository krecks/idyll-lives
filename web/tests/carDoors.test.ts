// A car's doors as drawn (`render/shared/cars.ts` `doorSwing`): wide open on their own, only part way between two
// cars in a row of bays, as the simulation opens them there (sim-core `cars/mod.rs` `Car::door_in_bay`: `SWING`,
// `DOOR_OUT`), and never into the next car. `bun test`.
import { describe, expect, test } from 'bun:test';
import { carDoor, CARS, DOOR_OPEN, DOOR_OUT, DOOR_SWING_ROOM, doorSwing, type CarAt } from '../src/render/shared/cars';
import { constant } from './snapshotLayout';


/** A row of `n` cars of `body` nose in, 3 m apart along x (as a car park's bays), facing +z or turned `yaw`. */
function row(body: number, n: number, yaw = 0): CarAt[] {
  const [len, wid] = [CARS[body].len, CARS[body].wid];
  return Array.from({ length: n }, (_, i) => ({ x: Math.cos(yaw) * 3 * i, z: -Math.sin(yaw) * 3 * i, yaw, len, wid }));
}

/** How far out from its side (m) door `d`'s rear edge reaches, opened `deg`. */
function reach(body: number, d: number, deg: number): number {
  const { hinge, rearBelt, rearLow } = carDoor(CARS[body], d);
  return (hinge - Math.min(rearBelt, rearLow)) * Math.sin((deg * Math.PI) / 180);
}

describe('a car’s doors as drawn', () => {
  test('keep to the simulation’s room and part way', () => {
    expect(DOOR_OUT).toBe(constant('DOOR_OUT'));
    expect(DOOR_SWING_ROOM).toBe(constant('SWING'));
  });

  test('open wide with no car beside them', () => {
    const [car] = row(0, 1);
    for (let d = 0; d < 4; d++) expect(doorSwing(CARS[0], d, car, [car])).toBe(DOOR_OPEN);
  });

  test('in a full row of bays, open part way, as far as the simulation has them, never into the next car', () => {
    for (let body = 0; body < CARS.length; body++) {
      for (const yaw of [0, 0.12, Math.PI / 2 + 0.3]) {
        const cars = row(body, 3, yaw);
        const mid = cars[1];
        for (let d = 0; d < 4; d++) {
          const deg = doorSwing(CARS[body], d, mid, cars);
          expect(deg).toBeLessThan(DOOR_OPEN);
          expect(deg).toBeGreaterThan(0);
          expect(reach(body, d, deg)).toBeLessThanOrEqual(DOOR_OUT + 1e-6);
          // (The gap to the next car, 3 m apart, less the two halves.)
          expect(reach(body, d, deg)).toBeLessThan(3 - CARS[body].wid);
        }
        // At the row's end (the next car on its +x side, its right: the left doors, 0 and 2, on -x), the doors on its
        // other side open wide.
        const end = cars[0];
        expect([0, 1, 2, 3].map((d) => doorSwing(CARS[body], d, end, cars) === DOOR_OPEN)).toEqual([true, false, true, false]);
      }
    }
  });

  test('a car ahead or behind leaves the doors wide', () => {
    const [len, wid] = [CARS[0].len, CARS[0].wid];
    const car: CarAt = { x: 0, z: 0, yaw: 0, len, wid };
    const ahead: CarAt = { x: 0, z: len + 0.5, yaw: 0, len, wid };
    for (let d = 0; d < 4; d++) expect(doorSwing(CARS[0], d, car, [ahead])).toBe(DOOR_OPEN);
  });
});
