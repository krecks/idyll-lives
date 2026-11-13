// `core/async.ts`: `both` waits for both promises to settle before it answers, so a failure never
// leaves the other one running on behind the caller (two builds side by side).
import { describe, expect, test } from 'bun:test';
import { both, within } from '../src/core/async';

const later = <T>(ms: number, value: T, fail = false) => new Promise<T>((resolve, reject) => setTimeout(() => (fail ? reject(new Error(String(value))) : resolve(value)), ms));

describe('both', () => {
  test('gives both results, in order', async () => {
    expect(await both(later(10, 'a'), later(1, 2))).toEqual(['a', 2]);
  });

  test('a failure waits for the other to settle, then is thrown', async () => {
    let done = false;
    const slow = later(30, 'slow').then((v) => ((done = true), v));
    await expect(both(later(1, 'fast', true), slow)).rejects.toThrow('fast');
    expect(done).toBe(true);
  });

  test('the first one\'s error when both fail', async () => {
    await expect(both(later(20, 'first', true), later(1, 'second', true))).rejects.toThrow('first');
  });
});

describe('within', () => {
  test('the fallback once the time is up', async () => {
    expect(await within(later(50, 'late'), 5, 'fallback')).toBe('fallback');
    expect(await within(later(1, 'soon'), 50, 'fallback')).toBe('soon');
  });
});
