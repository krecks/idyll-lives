// Two drafts of the menus' town with the same content (`core/sameContent.ts`): the town on screen is not built again
// for a new draft object of it (`menuScene.ts` `show`), and is for any difference. `bun test tests/sameContent.test.ts`.
import { describe, expect, test } from 'bun:test';
import { sameContent } from '../src/core/sameContent';

describe('the same content', () => {
  const draft = { name: 'Elm', width: 640, slots: [{ x: 1, z: 2, template: 'a' }], open: { land: { h: ['AAE'], cover: [3, 1] }, edited: undefined } };

  test('a copy, keys in another order, and an undefined key are the same', () => {
    expect(sameContent(draft, structuredClone(draft))).toBe(true);
    expect(sameContent({ a: 1, b: [2, 3] }, { b: [2, 3], a: 1 })).toBe(true);
    expect(sameContent({ a: 1, c: undefined }, { a: 1 })).toBe(true);
    expect(sameContent(new Int16Array([1, 2]), new Int16Array([1, 2]))).toBe(true);
  });

  test('any difference, however deep, is not', () => {
    const moved = structuredClone(draft);
    moved.slots[0].x = 1.5;
    expect(sameContent(draft, moved)).toBe(false);
    expect(sameContent(draft, { ...draft, open: { land: { h: ['AAF'], cover: [3, 1] } } })).toBe(false);
    expect(sameContent({ a: [1, 2] }, { a: [1, 2, 3] })).toBe(false);
    expect(sameContent({ a: 1 }, { a: 1, b: 2 })).toBe(false);
    expect(sameContent({ a: null }, { a: 0 })).toBe(false);
    expect(sameContent(new Int16Array([1, 2]), new Int32Array([1, 2]))).toBe(false);
    expect(sameContent([1], { 0: 1 })).toBe(false);
  });

  test('a Map or a class is the same only as itself', () => {
    const m = new Map([[1, 2]]);
    expect(sameContent(m, m)).toBe(true);
    expect(sameContent(m, new Map([[1, 2]]))).toBe(false);
    expect(sameContent(new Date(0), new Date(0))).toBe(false);
  });
});
