// The new game's steps keep what they made while the player goes back and forth: a step's key says
// what it was made from, so a later step is made again only when that changed. Also the code step 1
// opens with, the code the player types, the town choices a land is too small for, and the land's figures.
import { describe, expect, test } from 'bun:test';
import { blankLand, COVER, coverSpan } from '../src/game/land';
import type { NeighbourhoodDraft } from '../src/game/town';
import { townFits } from '../src/game/world';
import { codeFrom, firstCode, fitted, landKey, landStats, startCode, townKey, withSeed } from '../src/ui/newGame';

describe("the steps' keys", () => {
  const code = startCode('river', 1234);

  test('the land is keyed by its place, size, hills, woods and seed, not by the town', () => {
    const key = landKey(code, null);
    expect(landKey({ ...code, town: 'city', villages: 'many', townSeed: 9 }, null)).toBe(key);
    for (const other of [{ kind: 'heath' as const }, { size: 'large' as const }, { hills: 'mountains' as const }, { woods: 'open' as const }, { seed: 1235 }]) {
      expect(landKey({ ...code, ...other }, null)).not.toBe(key);
    }
  });

  test('a land shaped by hand is another land, and shaped again another still', () => {
    const field = blankLand(64, 64);
    expect(landKey(code, field)).toBe(landKey(code, null));
    field.edited = true;
    field.h[10] = 120;
    const shaped = landKey(code, field);
    expect(shaped).not.toBe(landKey(code, null));
    field.h[11] = 80;
    expect(landKey(code, field)).not.toBe(shaped);
  });

  test("the town is keyed by its land, town, villages, own seed and site; its name doesn't count", () => {
    const land = landKey(code, null);
    const key = townKey(land, code, null);
    expect(townKey(land, { ...code }, null)).toBe(key);
    expect(townKey(landKey({ ...code, seed: 5 }, null), code, null)).not.toBe(key);
    expect(townKey(land, { ...code, town: 'hamlet' }, null)).not.toBe(key);
    expect(townKey(land, { ...code, villages: 'none' }, null)).not.toBe(key);
    expect(townKey(land, { ...code, townSeed: 77 }, null)).not.toBe(key);
    expect(townKey(land, code, [120, 80])).not.toBe(key);
    expect(townKey(land, code, [120.2, 79.8])).toBe(townKey(land, code, [120, 80]));
  });

  test("a new land seed lets the town's and the people's own seeds go", () => {
    expect(withSeed({ ...code, townSeed: 3, peopleSeed: 4 }, 99)).toEqual({ ...code, seed: 99 });
  });
});

describe("the land's room for a town", () => {
  test('a new code fits its town and villages on its land', () => {
    for (const kind of ['country', 'river', 'island', 'mountain'] as const) {
      const c = startCode(kind, 1);
      expect(townFits(c.size, c.town, c.villages)).toBe(true);
    }
  });

  test('too much for the land: fewer villages first, then a smaller town', () => {
    const big = { ...startCode('country', 1), size: 'small' as const, town: 'city' as const, villages: 'many' as const };
    const fit = fitted(big);
    expect(townFits(fit.size, fit.town, fit.villages)).toBe(true);
    const fits = fitted({ ...big, size: 'large' });
    expect(townFits(fits.size, fits.town, fits.villages)).toBe(true);
    // (What already fits is left alone.)
    const ok = startCode('country', 1);
    expect(fitted(ok)).toBe(ok);
  });
});

describe('the code', () => {
  test('step 1 opens on the countryside, the timing tools\' place and options, or the URL\'s code', () => {
    expect(firstCode(null, null, 42)).toEqual(startCode('country', 42));
    expect(firstCode(null, { kind: 'island', options: { hills: 'flat' } }, 42)).toMatchObject({ kind: 'island', hills: 'flat', seed: 42 });
    // (A kind from before places: its place at its size.)
    expect(firstCode(null, { kind: 'dense', options: {} }, 42)).toMatchObject({ kind: 'country', town: 'city' });
    expect(firstCode('river-lhs-k2x9q', { kind: 'island', options: {} }, 42)).toMatchObject({ kind: 'river', size: 'large', hills: 'hilly', woods: 'some', seed: parseInt('k2x9q', 36) });
    expect(firstCode('777', { kind: 'island', options: {} }, 42)).toMatchObject({ kind: 'island', seed: 777 });
    // (Not a code: as if there were none.)
    expect(firstCode('not a code', null, 42)).toEqual(startCode('country', 42));
  });

  test('typed: a number is a new land seed, a whole code is taken as it is', () => {
    const now = { ...startCode('heath', 5), townSeed: 8 };
    expect(codeFrom('12', now)).toEqual(withSeed(now, 12));
    expect(codeFrom('island-lfwcm-k2x9q-1b-7', now)).toEqual({ kind: 'island', size: 'large', hills: 'flat', woods: 'wooded', town: 'city', villages: 'many', seed: parseInt('k2x9q', 36), townSeed: parseInt('1b', 36), peopleSeed: 7 });
    expect(codeFrom('nonsense-xyz', now)).toBeNull();
  });
});

describe("the land's figures", () => {
  const draft = (over: Partial<NeighbourhoodDraft>): NeighbourhoodDraft => ({ name: 'Test', width: 100, depth: 100, slots: [], roads: [], greens: [], households: [], ...over });

  test("from the draft: its water tiles and its woods' area", () => {
    const d = draft({
      open: {
        areas: [],
        zones: [{ kind: 'woods', poly: [[0, 0], [50, 0], [50, 40], [0, 40]] }],
        // (Run-length: 2500 dry, 1000 wet, the rest dry.)
        water: [2500, 0, 1000, 1, 6500, 0],
        pads: {},
        parking: [],
      },
    });
    expect(landStats(d)).toEqual({ width: 100, depth: 100, water: 0.1, woods: 0.2 });
  });

  test('from the land as shaped: its wet samples and its wooded cells', () => {
    const field = blankLand(100, 100);
    for (let k = 0; k < field.h.length / 4; k++) field.wl[k] = field.h[k] + 50;
    const cells = coverSpan(field.cols) * coverSpan(field.rows);
    for (let k = 0; k < cells / 2; k++) field.cover[k] = COVER.woods;
    const s = landStats(draft({}), field);
    expect(s.water).toBeCloseTo(0.25, 2);
    expect(s.woods).toBeCloseTo(0.5, 2);
  });
});
