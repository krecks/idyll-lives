// The dev pages' clip timing (`src/debug/clipBench.ts`, shared by `/clips.html` and `/inspect.html`): which kind each
// clip is by the entry's `motions`, and where each is at a time into the bench. `bun test`.
import { describe, expect, test } from 'bun:test';
import type { ModelEntry } from '../src/assets/types';
import { HOLD, REST, clipTime, cycleLength, kinds, shownClips, times, turnLength } from '../src/debug/clipBench';
import type { ModelClip } from '../src/render/shared/modelClips';

const clip = (duration: number): ModelClip => ({ duration, tracks: [] });
const entry = (motions: unknown) => ({ type: 'model', url: 'x.glb', motions }) as ModelEntry;

describe('kinds', () => {
  test('reads loops, the ambient clip, growth, clock hands and a use that repeats', () => {
    const of = kinds(
      entry({
        uses: { use: [{ clip: 'water', every: 2 }, 'door'], wait: { clip: 'lid', delay: 0.5 } },
        loops: { run: 'belt' },
        idle: 'sway',
        growth: 'grow',
        clock: { hour: 'hour', minute: 'minute', at: 30 },
      }),
    );
    expect(of.get('water')).toEqual({ kind: 'repeat', delay: undefined, every: 2 });
    expect(of.get('lid')).toEqual({ kind: 'use', delay: 0.5, every: undefined });
    expect(of.get('door')).toBeUndefined();
    expect(of.get('belt')?.kind).toBe('loop');
    expect(of.get('sway')?.kind).toBe('idle');
    expect(of.get('grow')?.kind).toBe('growth');
    expect(of.get('hour')).toEqual({ kind: 'clock', period: 720, at: 30 });
    expect(of.get('minute')).toEqual({ kind: 'clock', period: 60, at: 30 });
  });

  test('a clip the entry names nowhere plays as a use', () => {
    const shown = shownClips(new Map([['door', clip(1)]]), entry(undefined));
    expect(shown).toEqual([{ name: 'door', kind: 'use', clip: shown[0].clip }]);
  });
});

describe('a use', () => {
  const door = shownClips(new Map([['door', clip(1)]]), undefined)[0];

  test('rests, opens, holds open and closes', () => {
    expect(turnLength(door)).toBeCloseTo(REST + 1 + HOLD + 1);
    expect(clipTime(door, REST / 2)).toBe(0);
    expect(clipTime(door, REST + 0.5)).toBeCloseTo(0.5);
    expect(clipTime(door, REST + 1 + HOLD / 2)).toBe(1);
    expect(clipTime(door, REST + 1 + HOLD + 0.25)).toBeCloseTo(0.75);
    expect(clipTime(door, turnLength(door))).toBe(0);
  });

  test('held at a time, every clip stops there (no further than its end)', () => {
    const lid = { ...door, name: 'lid', clip: clip(0.4) };
    const { at, turn } = times([door, lid], 123, null, 0.6);
    expect(at.get('door')).toBe(0.6);
    expect(at.get('lid')).toBe(0.4);
    expect(turn).toBeNull();
    expect(times([door, lid], 123, 'lid', 0.6).turn).toBe('lid');
  });
});

describe('turns', () => {
  test('clips take their turns one after another; the ambient one plays throughout', () => {
    const shown = shownClips(
      new Map([
        ['door', clip(1)],
        ['belt', clip(0.5)],
        ['sway', clip(2)],
      ]),
      entry({ loops: { run: 'belt' }, idle: 'sway' }),
    );
    const cycle = cycleLength(shown);
    expect(cycle).toBeCloseTo(turnLength(shown[0]) + turnLength(shown[1]));
    const early = times(shown, REST + 0.5, null, null);
    expect(early.turn).toBe('door');
    expect(early.at.get('door')).toBeCloseTo(0.5);
    expect(early.at.get('sway')).toBeCloseTo((REST + 0.5) % 2);
    expect(early.at.has('belt')).toBe(false);
    const later = times(shown, turnLength(shown[0]) + REST + 0.7, null, null);
    expect(later.turn).toBe('belt');
    expect(later.at.get('belt')).toBeCloseTo(0.2);
    // Solo: only that clip, the others (even the ambient one) still.
    const solo = times(shown, REST + 0.5, 'belt', null);
    expect([...solo.at.keys()]).toEqual(['belt']);
  });
});
