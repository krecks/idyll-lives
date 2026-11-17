import { describe, expect, test } from 'bun:test';
import { mayOpen, type OpenLots } from '../src/game/openLots';

const sim = (id: number, plot: number | null, awayUntil: number | null = null) => ({ id, plot, awayUntil });
const state = (over: Partial<OpenLots> = {}): OpenLots => ({ home: 1, lookInside: null, sims: [sim(10, 1), sim(20, 2)], mine: new Set([10]), ...over });

describe('mayOpen', () => {
  test('the home opens', () => expect(mayOpen(1, state())).toBe(true));
  test("a neighbour's house stays closed, though they are in", () => expect(mayOpen(2, state())).toBe(false));
  test('a lot one of the household is on opens', () => expect(mayOpen(2, state({ sims: [sim(10, 2), sim(20, 2)] }))).toBe(true));
  test('one away at work opens nothing', () => expect(mayOpen(3, state({ sims: [sim(10, 3, 1020)] }))).toBe(false));
  test('the lot looked inside opens', () => expect(mayOpen(2, state({ lookInside: 2 }))).toBe(true));
  test('without a home only those lots open', () => {
    expect(mayOpen(1, state({ home: null, sims: [] }))).toBe(false);
    expect(mayOpen(4, state({ home: null, sims: [sim(10, 4)] }))).toBe(true);
  });
});
