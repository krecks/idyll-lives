// `render/shared/lampShadows.ts` and `shared/house.ts` `lampCasts`: which lamps cast shadows (`bun test`).
import { describe, expect, test } from 'bun:test';
import { lampCasts } from '../src/render/shared/house';
import { pickCasters } from '../src/render/shared/lampShadows';

describe('the lamps that cast shadows', () => {
  const pick = (distance: number[], casting: boolean[], count: number, keep = 0.7) => {
    const want: boolean[] = [];
    const any = pickCasters(distance, casting, count, keep, want);
    return { any, want };
  };

  test('the nearest the camera looks at, as many as the preset has', () => {
    expect(pick([5, 1, 3, 9], [false, false, false, false], 2)).toEqual({ any: true, want: [false, true, true, false] });
    expect(pick([5, 1, 3, 9], [false, false, false, false], 4).want).toEqual([true, true, true, true]);
    expect(pick([5, 1, 3, 9], [false, false, false, false], 0)).toEqual({ any: false, want: [false, false, false, false] });
  });

  test('a lamp that cannot cast now is never chosen, however few there are', () => {
    expect(pick([Infinity, 4, Infinity], [false, false, false], 2)).toEqual({ any: true, want: [false, true, false] });
    expect(pick([Infinity, Infinity], [true, false], 2)).toEqual({ any: false, want: [false, false] });
  });

  test('a lamp that casts keeps its place until another is clearly nearer', () => {
    // (3 m against 2.5 m: the one casting stays; against 2 m it gives way.)
    expect(pick([3, 2.5], [true, false], 1).want).toEqual([true, false]);
    expect(pick([3, 2], [true, false], 1).want).toEqual([false, true]);
    // (Without the margin the nearer one has it at once.)
    expect(pick([3, 2.5], [true, false], 1, 1).want).toEqual([false, true]);
  });

  test('lamps that light the room cast; a fire, a light that follows a use and an uplighter do not', () => {
    expect(lampCasts({})).toBe(true);
    expect(lampCasts({ aim: 'down' })).toBe(true);
    expect(lampCasts({ aim: 'out' })).toBe(true);
    expect(lampCasts({ aim: 'ceiling' })).toBe(true);
    expect(lampCasts({ aim: 'up' })).toBe(false);
    expect(lampCasts({ fire: true })).toBe(false);
    expect(lampCasts({ while: 'cook' })).toBe(false);
  });
});
