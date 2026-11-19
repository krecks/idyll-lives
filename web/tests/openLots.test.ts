import { describe, expect, test } from 'bun:test';
import { mayOpen, type OpenLots } from '../src/game/openLots';
import { BUILDING, FLATS, realTown } from './fixtures/realTown';

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

describe('mayOpen in a building of flats (the real town: Mill Court, its flats 1 to 4)', () => {
  const { flats } = realTown();
  const [one, two, three] = FLATS;
  // (The player's household lives in flat 1; a neighbour, resident 20, is at home in flat 2.)
  const flatState = (over: Partial<OpenLots> = {}): OpenLots => ({ home: one, lookInside: null, sims: [sim(10, one), sim(20, two)], mine: new Set([10]), flats, ...over });

  test('the home flat opens, and its building with it; the neighbours stay closed', () => {
    const s = flatState();
    expect(mayOpen(one, s)).toBe(true);
    expect(mayOpen(BUILDING, s)).toBe(true);
    expect(mayOpen(two, s)).toBe(false);
    expect(mayOpen(three, s)).toBe(false);
  });
  test("one of yours visiting a neighbour's flat opens it", () => {
    expect(mayOpen(two, flatState({ sims: [sim(10, two), sim(20, two)] }))).toBe(true);
  });
  test('one of yours in the hall or the yard opens the building, not its flats', () => {
    const s = flatState({ home: null, sims: [sim(10, BUILDING)] });
    expect(mayOpen(BUILDING, s)).toBe(true);
    expect(FLATS.some((f) => mayOpen(f, s))).toBe(false);
  });
  test('a flat looked inside opens with its building; the building looked inside opens none of its flats', () => {
    const flat = flatState({ home: null, sims: [], lookInside: three });
    expect(mayOpen(BUILDING, flat)).toBe(true);
    expect(FLATS.filter((f) => mayOpen(f, flat))).toEqual([three]);
    const building = flatState({ home: null, sims: [], lookInside: BUILDING });
    expect(mayOpen(BUILDING, building)).toBe(true);
    expect(FLATS.some((f) => mayOpen(f, building))).toBe(false);
  });
  test('a building none of yours lives in, is in or looks inside stays closed', () => {
    expect(mayOpen(BUILDING, flatState({ home: 7, sims: [sim(20, two)] }))).toBe(false);
  });
});
