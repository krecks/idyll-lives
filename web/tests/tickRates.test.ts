// The sim's two clocks on the web side: real seconds (`ticksPerSecond`) and game minutes (`ticksPerMinute`), both from
// the snapshot layout, and what keeps to sim-core's own (`lib.rs`, `clock.rs`) without a layout to hand. `bun test`.
import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { clockGain, maxAlpha } from '../src/core/bridge';
import { gameMinutesPerSecond, minutesPerTick, readLayout, tickRates } from '../src/core/snapshot';
import { PLAYBACK_RATE, QUIET_SPEED } from '../src/render/shared/gameSpeed';
import { constant, TICKS_PER_MINUTE, TICKS_PER_SECOND } from './snapshotLayout';

const CLOCK_RS = readFileSync(join(import.meta.dir, '../../crates/sim-core/src/clock.rs'), 'utf8');

describe('tick rates', () => {
  test("before a game, the rates are sim-core's (`lib.rs`)", () => {
    expect(tickRates).toEqual({ ticksPerSecond: TICKS_PER_SECOND, ticksPerMinute: TICKS_PER_MINUTE });
  });

  test("the playback rate is the ticks a step runs at each speed (`clock::TICKS_PER_STEP`), quiet hours sim-core's", () => {
    const steps = /pub const TICKS_PER_STEP: \[u32; \d+\] = \[([^\]]+)\];/.exec(CLOCK_RS);
    expect(steps).not.toBeNull();
    expect(PLAYBACK_RATE).toEqual(steps![1].split(',').map((n) => Number(n.trim())));
    expect(QUIET_SPEED).toBe(constant('AUTO_FAST_SPEED', undefined, CLOCK_RS));
  });

  test('a layout without `ticksPerMinute` (an older sim) has a game minute a real second', () => {
    expect(readLayout('{"ticksPerSecond": 20}').ticksPerMinute).toBe(20);
    expect(readLayout('{"ticksPerSecond": 60, "ticksPerMinute": 75}').ticksPerMinute).toBe(75);
    expect(minutesPerTick({ ticksPerMinute: 20 })).toBe(1 / 20);
    expect(gameMinutesPerSecond({ ticksPerSecond: 20, ticksPerMinute: 20 })).toBe(1);
    expect(gameMinutesPerSecond({ ticksPerSecond: 60, ticksPerMinute: 75 })).toBeCloseTo(0.8, 12);
  });

  test('the frame clock keeps to real time at any step rate, as it was at 20 steps a second', () => {
    // At 50 ms steps, the numbers it always had.
    expect(clockGain(50)).toBe(0.05);
    expect(maxAlpha(50)).toBe(1.3);
    // At 60 a second, three arrivals take up as much lateness as one did, and it shows on past the newest as long.
    const g = clockGain(1000 / 60);
    expect(1 - (1 - g) ** 3).toBeCloseTo(0.05, 12);
    expect((maxAlpha(1000 / 60) - 1) * (1000 / 60)).toBeCloseTo(15, 9);
  });
});
