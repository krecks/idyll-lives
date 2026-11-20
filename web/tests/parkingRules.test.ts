// The town's parking bays keep to the simulation's rules (sim-core `cars/parking.rs`): as far off a driveway as a trip's
// car parks (`MOUTH_CLEAR`, `DRIVE_CLEAR`), as big as its bays (`BAY_WIDE`, `BAY_DEEP`) and their lines (`BAY_LINES`).
// `bun test`.
import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { BAY_DEEP, BAY_LINES, BAY_WIDE, DRIVE_CLEAR, MOUTH_CLEAR } from '../src/game/parking';
import { constant } from './snapshotLayout';

const PARKING_RS = readFileSync(join(import.meta.dir, '../../crates/sim-core/src/cars/parking.rs'), 'utf8');

describe("parking.ts's rules", () => {
  test('are the simulation\'s', () => {
    expect(MOUTH_CLEAR).toBe(constant('MOUTH_CLEAR', undefined, PARKING_RS));
    expect(DRIVE_CLEAR).toBe(constant('DRIVE_CLEAR', undefined, PARKING_RS));
    expect(BAY_WIDE).toBe(constant('BAY_WIDE', undefined, PARKING_RS));
    expect(BAY_DEEP).toBe(constant('BAY_DEEP', undefined, PARKING_RS));
    const lines = /pub const BAY_LINES: \[f32; 2\] = \[([0-9.]+), ([0-9.]+)\];/.exec(PARKING_RS);
    expect(lines).not.toBeNull();
    expect(BAY_LINES).toEqual([Number(lines![1]), Number(lines![2])]);
  });

  test("a bay's lines are inside it", () => {
    expect(BAY_LINES[0]).toBeLessThan(BAY_WIDE);
    expect(BAY_LINES[1]).toBeLessThan(BAY_DEEP);
  });
});
