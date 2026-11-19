// The sounds of things happening (`audio/events.ts`; made in Rust, `crates/sound/src/things/events.rs`, and tested
// there): a busy street's doors and gates never sound more than a few at a time; a car's row tells a door shut and
// a parked car setting off, and a car only coming into sight says nothing; curtains drawn or opened are the ids that
// changed, none on the first sight of them.
import { describe, expect, test } from 'bun:test';
import { EVENTS, ONCE_KINDS, ONCE_LEVEL } from '../src/audio/catalog';
import { carEvents, changed, doorMay } from '../src/audio/events';
import { EVENT_THINGS, MADE, ONCE_EVENTS } from '../src/audio/kinds';

describe('doors and gates', () => {
  const d = EVENTS.door;

  test('never two closer than `apart`', () => {
    const recent: number[] = [];
    expect(doorMay(recent, 10)).toBe(true);
    expect(doorMay(recent, 10 + d.apart / 2)).toBe(false);
    expect(doorMay(recent, 10 + d.apart + 0.01)).toBe(true);
    expect(recent).toHaveLength(2);
  });

  test('at most `voices` within `within`, then more once the first have gone by', () => {
    const recent: number[] = [];
    const gap = d.apart + 0.01;
    let t = 0;
    for (let k = 0; k < d.voices; k++, t += gap) expect(doorMay(recent, t)).toBe(true);
    // (`voices` doors `apart` from each other fit well within `within`.)
    expect(gap * d.voices).toBeLessThan(d.within);
    expect(doorMay(recent, t)).toBe(false);
    expect(recent).toHaveLength(d.voices);
    // Once the first is older than `within`, one more may sound.
    expect(doorMay(recent, d.within + 0.001)).toBe(true);
  });

  test('a street of gates all swinging at once stays calm: never more than `voices` in any `within`', () => {
    const recent: number[] = [];
    const heard: number[] = [];
    for (let t = 0; t < 30; t += 0.02) if (doorMay(recent, t)) heard.push(t);
    expect(heard.length).toBeGreaterThan(10);
    for (const t of heard) expect(heard.filter((u) => u >= t && u < t + d.within).length).toBeLessThanOrEqual(d.voices);
    for (let k = 1; k < heard.length; k++) expect(heard[k] - heard[k - 1]).toBeGreaterThanOrEqual(d.apart);
  });
});

describe("a car's row", () => {
  test('a door bit clearing is a door shut', () => {
    expect(carEvents({ shown: 1, doors: 0b01 }, 1, 0)).toEqual({ door: true, start: false });
    expect(carEvents({ shown: 1, doors: 0b11 }, 1, 0b10)).toEqual({ door: true, start: false });
    // A door opening, or one staying open, is not.
    expect(carEvents({ shown: 1, doors: 0 }, 1, 0b01)).toEqual({ door: false, start: false });
    expect(carEvents({ shown: 1, doors: 0b01 }, 1, 0b01)).toEqual({ door: false, start: false });
    // A taxi's door shut as it drives off.
    expect(carEvents({ shown: 2, doors: 0b100 }, 2, 0).door).toBe(true);
    // Gone out of sight with a door open: nothing to hear.
    expect(carEvents({ shown: 1, doors: 0b01 }, 0, 0).door).toBe(false);
  });

  test('parked then driving is a start; driving on, or parking, is not', () => {
    expect(carEvents({ shown: 1, doors: 0 }, 2, 0)).toEqual({ door: false, start: true });
    expect(carEvents({ shown: 2, doors: 0 }, 2, 0).start).toBe(false);
    expect(carEvents({ shown: 2, doors: 0 }, 1, 0).start).toBe(false);
    // Both in one look.
    expect(carEvents({ shown: 1, doors: 0b1 }, 2, 0)).toEqual({ door: true, start: true });
  });

  test('a first sight, or a car coming into town, says nothing', () => {
    expect(carEvents(undefined, 2, 0)).toEqual({ door: false, start: false });
    expect(carEvents(undefined, 1, 0b1)).toEqual({ door: false, start: false });
    expect(carEvents({ shown: 0, doors: 0b1 }, 2, 0)).toEqual({ door: false, start: false });
    expect(carEvents({ shown: 0, doors: 0 }, 1, 0)).toEqual({ door: false, start: false });
  });
});

describe('curtains', () => {
  test('the ids drawn or opened since the last; none on the first sight', () => {
    expect(changed(null, new Set([1, 2]))).toEqual([]);
    expect(changed(new Set([1, 2]), new Set([1, 2]))).toEqual([]);
    expect(changed(new Set([1, 2]), new Set([2, 3])).sort()).toEqual([1, 3]);
    expect(changed(new Set(), new Set([4]))).toEqual([4]);
    expect(changed(new Set([4]), new Set())).toEqual([4]);
  });
});

describe('the sounds', () => {
  test('each is made, and those played once have a level (the clock a loop)', () => {
    for (const kind of EVENT_THINGS) expect(MADE).toContain(kind);
    for (const kind of ONCE_EVENTS) {
      expect(ONCE_KINDS).toContain(kind);
      expect(ONCE_LEVEL[kind]).toBeGreaterThan(0);
    }
    expect(ONCE_KINDS).not.toContain('clockTick');
  });
});
