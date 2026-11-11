// `render/tune.ts`: the dev server's `?tune=` (`bun test`).
import { describe, expect, test } from 'bun:test';
import { applyTune, parseTune, tuned } from '../src/render/tune';

/** `fn`'s result and what it said on the console's warnings. */
function warned<T>(fn: () => T): [T, number] {
  const warn = console.warn;
  let said = 0;
  console.warn = () => void said++;
  try {
    return [fn(), said];
  } finally {
    console.warn = warn;
  }
}

describe('tune', () => {
  test('reads paths and values as written', () => {
    expect([...parseTune('lawn.tip:1.12; look.vignette:0.8;key.7.5.sky:0.2;;odd')]).toEqual([
      ['lawn.tip', '1.12'],
      ['look.vignette', '0.8'],
      ['key.7.5.sky', '0.2'],
    ]);
  });

  test('a value takes the type of what it replaces', () => {
    const base = { tip: 1.35, sky: [0.3, 0.75], colour: '#A9C4E4', name: 'lawn', deep: { x: 1 } };
    const out = applyTune('lawn', base, parseTune('lawn.tip:1.12;lawn.sky:0.85,0.95;lawn.colour:8FB4EE;lawn.name:calm;lawn.deep.x:2'));
    expect(out).toEqual({ tip: 1.12, sky: [0.85, 0.95], colour: '#8FB4EE', name: 'calm', deep: { x: 2 } });
    // (A copy: the constants stay as written.)
    expect(base.tip).toBe(1.35);
    expect(base.deep.x).toBe(1);
  });

  test("another group's entries, and names of nothing, change nothing", () => {
    const base = { tip: 1.35 };
    expect(applyTune('lawn', base, parseTune('look.tip:2'))).toBe(base);
    const [out, said] = warned(() => applyTune('lawn', base, parseTune('lawn.top:2')));
    expect(out).toEqual(base);
    expect(said).toBe(1);
  });

  test('a keyframe is named by its hour, dots and all', () => {
    const key = { hour: 7.5, sky: 0.12 };
    expect(applyTune('key.7.5', key, parseTune('key.7.5.sky:0.2')).sky).toBe(0.2);
    // (An hour 7's would take it for its own `5.sky`, which it hasn't.)
    expect(warned(() => applyTune('key.7', { hour: 7, sky: 0.12 }, parseTune('key.7.5.sky:0.2')).sky)).toEqual([0.12, 1]);
  });

  test('nothing is tuned outside the dev server', () => {
    const base = { tip: 1.35 };
    expect(tuned('lawn', base)).toBe(base);
  });
});
