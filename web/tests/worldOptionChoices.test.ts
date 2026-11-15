// A world's options as the tools name them and step 1 sums a world up: lists in either form read
// the same, a wrong entry is told and left out, and the neighbours' cap is only said when the
// homes could hold it.
import { describe, expect, test } from 'bun:test';
import { OPTION_FIELDS, parseOptions, sameOptions, worldLine as line } from '../src/game/worldOptions';

describe("a world's options", () => {
  test('read from a list, with : or =', () => {
    expect(parseOptions('town:city,villages:many')).toEqual({ town: 'city', villages: 'many' });
    expect(parseOptions(' town=hamlet , hills=mountains,woods:wooded ')).toEqual({ town: 'hamlet', hills: 'mountains', woods: 'wooded' });
    expect(parseOptions('')).toEqual({});
    expect(parseOptions(null)).toEqual({});
  });

  test('an unknown option or choice is left out and told', () => {
    const bad: string[] = [];
    expect(parseOptions('town:metropolis,size:big,villages:few,hills', (e) => bad.push(e))).toEqual({ villages: 'few' });
    expect(bad).toEqual(['town:metropolis', 'size:big', 'hills']);
  });

  test('every choice of every option reads back', () => {
    for (const f of OPTION_FIELDS) for (const c of f.choices) expect(parseOptions(`${f.key}:${c.value}`)).toEqual({ [f.key]: c.value });
  });

  test('the same options, whatever their order', () => {
    expect(sameOptions({ town: 'town', villages: 'few', hills: 'flat', woods: 'open' }, { woods: 'open', hills: 'flat', villages: 'few', town: 'town' })).toBe(true);
    expect(sameOptions({ town: 'town' }, { town: 'city' })).toBe(false);
    expect(sameOptions(undefined, undefined)).toBe(true);
    expect(sameOptions(undefined, { town: 'town' })).toBe(false);
  });
});

describe('the world in a line', () => {
  // (The land's size and the cap each on one line: no-break spaces, read here as plain ones.)
  const worldLine = (w: Parameters<typeof line>[0]) => line(w).replaceAll('\u00a0', ' ');

  test('a hamlet alone, too few homes for the cap', () => {
    expect(worldLine({ homes: 24, villages: 0, width: 640, depth: 640, town: 'hamlet', cap: 220 })).toBe('24 homes in a hamlet · 640 × 640 m');
  });

  test('a town with villages and room for the cap', () => {
    expect(worldLine({ homes: 344, villages: 6, width: 960, depth: 960, town: 'town', cap: 220 })).toBe('344 homes in a town and 6 villages · 960 × 960 m · up to 220 neighbours');
    expect(worldLine({ homes: 221, villages: 1, width: 896, depth: 832, town: 'city', cap: 220 })).toBe('221 homes in a city and 1 village · 896 × 832 m · up to 220 neighbours');
    expect(worldLine({ homes: 220, villages: 0, width: 896, depth: 832, town: 'city', cap: 220 })).toBe('220 homes in a city · 896 × 832 m');
  });

  test('a world from before options is a town', () => {
    expect(worldLine({ homes: 1, villages: 0, width: 576, depth: 576, cap: 220 })).toBe('1 home in a town · 576 × 576 m');
  });
});
