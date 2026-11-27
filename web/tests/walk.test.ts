// `game/walk.ts`: WoW keys in game x/z (z is the engine's -z): W ahead, S back, Q/E step left/right, A/D turn.
import { expect, test } from 'bun:test';
import { inFront, moveDir, type WalkKey } from '../src/game/walk';

const dir = (keys: WalkKey[], heading = 0) => moveDir(new Set(keys), heading);
const near = (v: [number, number] | null, x: number, z: number) => {
  expect(v![0]).toBeCloseTo(x);
  expect(v![1]).toBeCloseTo(z);
};

test('W/S walk along the heading, Q/E across it', () => {
  near(dir(['w']).dir, 1, 0);
  near(dir(['s']).dir, -1, 0);
  near(dir(['e']).dir, 0, -1);
  near(dir(['q']).dir, 0, 1);
  near(dir(['w'], Math.PI / 2).dir, 0, 1);
  near(dir(['w', 'e']).dir, Math.SQRT1_2, -Math.SQRT1_2);
});

test('nothing held, or opposites, stands; A/D only turn', () => {
  expect(dir([]).dir).toBeNull();
  expect(dir(['w', 's']).dir).toBeNull();
  expect(dir(['a'])).toEqual({ dir: null, turn: -1 });
  expect(dir(['d']).turn).toBe(1);
});

test('F reaches whoever is in front, near enough', () => {
  const me = { x: 0, z: 0 };
  expect(inFront(me, 0, 1.5, 0.3, 2.5)).toBe(true);
  expect(inFront(me, 0, 1.5, 1.5, 2.5)).toBe(false); // 45° off
  expect(inFront(me, 0, -1, 0, 2.5)).toBe(false); // behind
  expect(inFront(me, 0, 3, 0, 2.5)).toBe(false); // out of reach
  expect(inFront(me, Math.PI / 2, 0, 2, 2.5)).toBe(true);
});
