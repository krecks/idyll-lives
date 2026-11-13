// `render/shared/roomLight.ts`: a room's own light by night, from its lamplight (`bun test`).
import { describe, expect, test } from 'bun:test';
import { ROOM, roomLight } from '../src/render/shared/roomLight';

describe('a room\'s light', () => {
  test('by day a room is lit as it was: the whole sky, whatever lamps are on', () => {
    expect(roomLight(0, 12, 0)).toEqual({ bounce: 0, sky: 1 });
    // (A windowless room's lamps are on by day: they bounce, the sky's share stays.)
    expect(roomLight(3, 12, 0).sky).toBe(1);
    expect(roomLight(3, 12, 0).bounce).toBe(1);
  });

  test('at night a dark room keeps the moon, a lamplit one loses it', () => {
    expect(roomLight(0, 12, 1)).toEqual({ bounce: 0, sky: 1 });
    expect(roomLight(2, 12, 1).bounce).toBe(1);
    expect(roomLight(2, 12, 1).sky).toBeCloseTo(ROOM.sky, 9);
  });

  test('the change comes in with the lamps, a lamp at a time and from dusk to night', () => {
    // (A lamp a quarter on: the room a quarter lamplit, the sky's share a quarter of the way down.)
    const one = roomLight(0.25, 12, 1);
    expect(one.bounce).toBeCloseTo(0.25 / ROOM.full, 9);
    expect(one.sky).toBeCloseTo(1 + (ROOM.sky - 1) * one.bounce, 9);
    // (Sunset, lamps 0.4: the sky outside is still light.)
    expect(roomLight(2, 12, ROOM.night[0]).sky).toBe(1);
    const dusk = roomLight(2, 12, (ROOM.night[0] + ROOM.night[1]) / 2).sky;
    expect(dusk).toBeCloseTo((1 + ROOM.sky) / 2, 6);
  });

  test('a big room needs more lamps for the same light', () => {
    expect(roomLight(ROOM.full, ROOM.area * 3, 1).bounce).toBeCloseTo(1 / 3, 9);
    expect(roomLight(ROOM.full * 3, ROOM.area * 3, 1).bounce).toBe(1);
    expect(roomLight(ROOM.full / 2, 4, 1)).toEqual(roomLight(ROOM.full / 2, ROOM.area, 1));
  });
});
