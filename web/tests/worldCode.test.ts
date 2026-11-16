/**
 * The world code (`game/worldCode.ts`): written and read back the same, any case and spaces round
 * it taken, a bare number read as the seed alone, and what isn't a code refused. And the land
 * sizes the code names (`world.ts` `sizeFor`, `townFits`).
 */
import { describe, expect, test } from 'bun:test';
import { readCode, writeCode, type WorldCode } from '../src/game/worldCode';
import { LAND_SIZE, sizeFor, townFits } from '../src/game/world';

describe('world code', () => {
  const codes: WorldCode[] = [
    { kind: 'river', size: 'medium', hills: 'hilly', woods: 'some', seed: 123456 },
    { kind: 'island', size: 'large', hills: 'flat', woods: 'wooded', seed: 0xffff_ffff, town: 'city', villages: 'many', townSeed: 47, peopleSeed: 7 },
    { kind: 'country', size: 'small', hills: 'mountains', woods: 'open', seed: 0, villages: 'few' },
    { kind: 'heath', size: 'small', hills: 'rolling', woods: 'some', seed: 99, town: 'hamlet' },
    { kind: 'farm', size: 'medium', hills: 'flat', woods: 'open', seed: 5, peopleSeed: 12 },
    { kind: 'lakeside', size: 'large', hills: 'hilly', woods: 'wooded', seed: 5, townSeed: 6 },
  ];

  test('written and read back the same', () => {
    for (const code of codes) expect(readCode(writeCode(code))).toEqual(code);
  });

  test('written short: the flags and seeds not chosen left off the end', () => {
    expect(writeCode(codes[0])).toBe(`river-mhs-${(123456).toString(36)}`);
    expect(writeCode(codes[1])).toBe('island-lfwcm-1z141z3-1b-7');
    expect(writeCode(codes[2])).toBe('country-smo_f-0');
    expect(writeCode(codes[4])).toBe('farm-mfo-5--c');
  });

  test('any case, spaces round it', () => {
    expect(readCode('  ISLAND-LFWCM-1Z141Z3-1B-7 ')).toEqual(codes[1]);
  });

  test('a bare number is the seed alone', () => {
    expect(readCode('4242')).toEqual({ seed: 4242 });
    expect(readCode(' 0 ')).toEqual({ seed: 0 });
    expect(readCode(String(2 ** 32))).toBeNull();
  });

  test('not a code', () => {
    for (const text of ['', 'river', 'river-mhs', 'moon-mhs-1', 'river-xhs-1', 'river-mh-1', 'river-mhsq-1', 'river-mhs_x-1', 'river-mhs-1-', 'river-mhs-!', 'river-mhs-1-2-3-4', 'river-mhs-zzzzzzzz']) {
      expect(readCode(text)).toBeNull();
    }
  });
});

describe('land sizes', () => {
  test('a town and its villages get the smallest land they fit on', () => {
    expect(LAND_SIZE).toEqual({ small: 640, medium: 832, large: 960 });
    expect(sizeFor('hamlet', 'none')).toBe('small');
    expect(sizeFor('village', 'none')).toBe('small');
    expect(sizeFor('city', 'none')).toBe('medium');
    expect(sizeFor('town', 'few')).toBe('medium');
    expect(sizeFor('city', 'few')).toBe('large');
    expect(sizeFor('city', 'many')).toBe('large');
  });

  test('a land too small for the town and villages chosen', () => {
    expect(townFits('small', 'town', 'none')).toBe(false);
    expect(townFits('medium', 'hamlet', 'many')).toBe(false);
    expect(townFits('large', 'city', 'many')).toBe(true);
    expect(townFits('large', 'hamlet', 'none')).toBe(true);
  });
});
