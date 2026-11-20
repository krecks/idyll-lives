// The town's parking bays keep to the simulation's rules (sim-core `cars/parking.rs`): as far off a driveway as a trip's
// car parks (`MOUTH_CLEAR`, `DRIVE_CLEAR`), as big as its bays (`BAY_WIDE`, `BAY_DEEP`) and their lines (`BAY_LINES`).
// `bun test`.
import { describe, expect, test } from 'bun:test';
import { BAY_DEEP, BAY_LINES, BAY_WIDE, DRIVE_CLEAR, MOUTH_CLEAR } from '../src/game/parking';
import { constant, rustConst } from './snapshotLayout';

describe("parking.ts's rules", () => {
  test('are the simulation\'s', () => {
    expect(MOUTH_CLEAR).toBe(constant('MOUTH_CLEAR'));
    expect(DRIVE_CLEAR).toBe(constant('DRIVE_CLEAR', undefined, 'cars/parking.rs'));
    expect(BAY_WIDE).toBe(constant('BAY_WIDE'));
    expect(BAY_DEEP).toBe(constant('BAY_DEEP'));
    expect(BAY_LINES).toEqual(JSON.parse(rustConst('BAY_LINES')));
  });

  test("a bay's lines are inside it", () => {
    expect(BAY_LINES[0]).toBeLessThan(BAY_WIDE);
    expect(BAY_LINES[1]).toBeLessThan(BAY_DEEP);
  });
});
