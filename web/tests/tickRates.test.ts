// The sim's clocks on the web side: real seconds (`ticksPerSecond`), game minutes (`ticksPerMinute`) and the traffic's
// (`driveScale`), all from the snapshot layout, and what keeps to sim-core's own (`lib.rs`, `clock.rs`) without a
// layout to hand. `bun test`.
import { describe, expect, test } from 'bun:test';
import { clockGain, maxAlpha } from '../src/core/bridge';
import { gameMinutesPerSecond, minutesPerTick, readLayout, tickClock, tickRates } from '../src/core/snapshot';
import { SUPER_SPEED as GAME_SUPER_SPEED } from '../src/game/speed';
import { PLAYBACK_RATE, playbackRate, SUPER_SPEED } from '../src/render/shared/gameSpeed';
import { constant, DRIVE_SCALE, rustConst, TICKS_PER_MINUTE, TICKS_PER_SECOND } from './snapshotLayout';

describe('tick rates', () => {
  test("before a game, the rates are sim-core's (`lib.rs`)", () => {
    expect(tickRates).toEqual({ ticksPerSecond: TICKS_PER_SECOND, ticksPerMinute: TICKS_PER_MINUTE, driveScale: DRIVE_SCALE });
    expect(DRIVE_SCALE).toBeGreaterThanOrEqual(1);
  });

  test("the playback rate is the ticks a step runs at each speed (`clock::TICKS_PER_STEP`), super speed sim-core's", () => {
    expect(PLAYBACK_RATE).toEqual(JSON.parse(rustConst('TICKS_PER_STEP')));
    expect(SUPER_SPEED).toBe(constant('SUPER_SPEED'));
    expect(GAME_SUPER_SPEED).toBe(SUPER_SPEED);
    expect(SUPER_SPEED).toBe(constant('MAX_SPEED'));
  });

  test("a snapshot plays at the ticks its step ran (the header's rate), else at its speed's", () => {
    // Super speed on a machine that fitted 17 ticks in a step.
    expect(playbackRate([0, 0, 0, 4, 0, 0, 17], { speed: 3, rate: 6 })).toBe(17);
    expect(playbackRate([0, 0, 0, 0, 0, 0, 0], { speed: 3, rate: 6 })).toBe(0);
    expect(playbackRate([0, 0, 0, 3], { speed: 3 })).toBe(10);
  });

  test('a layout without `ticksPerMinute` (an older sim) has a game minute a real second', () => {
    expect(readLayout('{"ticksPerSecond": 20}').ticksPerMinute).toBe(20);
    expect(readLayout('{"ticksPerSecond": 60, "ticksPerMinute": 75}').ticksPerMinute).toBe(75);
    // (Nor `driveScale`: vehicles went at real time.)
    expect(readLayout('{"ticksPerSecond": 20}').driveScale).toBe(1);
    expect(readLayout('{"ticksPerSecond": 60, "ticksPerMinute": 75, "driveScale": 4}').driveScale).toBe(4);
    expect(minutesPerTick({ ticksPerMinute: 20 })).toBe(1 / 20);
    expect(gameMinutesPerSecond({ ticksPerSecond: 20, ticksPerMinute: 20 })).toBe(1);
    expect(gameMinutesPerSecond({ ticksPerSecond: 60, ticksPerMinute: 75 })).toBeCloseTo(0.8, 12);
  });

  test("a tick's day and time are the sim's clock: from 08:00 on day 1 (`clock.rs`), as a pasted state's save shows them", () => {
    expect(rustConst('START_MINUTE', 'clock.rs')).toBe('8 * 60');
    expect(tickClock(0)).toEqual({ day: 1, minute: 8 * 60 });
    // Day 6, 09:40 (a state the bench tool made): not 01:40, the time since tick 0 alone.
    const tick = (5 * 1440 + 9 * 60 + 40 - 8 * 60) * TICKS_PER_MINUTE;
    expect(tickClock(tick)).toEqual({ day: 6, minute: 9 * 60 + 40 });
    // Midnight is the next day's first minute.
    expect(tickClock(16 * 60 * TICKS_PER_MINUTE)).toEqual({ day: 2, minute: 0 });
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
