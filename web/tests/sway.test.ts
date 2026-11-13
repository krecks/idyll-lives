// The hair's and the hems' lag (`render/shared/characters/sway.ts`): the same swing at any frame rate, nothing drawn
// of a resident standing still, a pause holding it, and a resident put somewhere else starting again at rest.
import { describe, expect, test } from 'bun:test';
import { SWAY, SWAY_FLOATS, swayStep } from '../src/render/shared/characters/sway';

/** A walk: off at 1.3 m/s along z with a step's bob and sway, a stop after 2 s. */
function towed(t: number): [number, number, number] {
  const walking = Math.min(t, 2);
  const stride = t < 2 ? 1 : 0;
  return [0.015 * Math.sin(2 * Math.PI * 0.9 * walking) * stride, 1.5 + 0.02 * Math.sin(2 * Math.PI * 1.8 * walking) * stride, 1.3 * walking];
}

/** The offsets drawn at every tenth of a second over `seconds`, stepped at `fps`. */
function run(fps: number, seconds: number): number[][] {
  const state = new Float32Array(SWAY_FLOATS);
  const out = new Float32Array(3);
  const seen: number[][] = [];
  const per = fps / 10;
  for (let f = 0; f <= seconds * fps; f++) {
    const [x, y, z] = towed(f / fps);
    swayStep(state, 0, x, y, z, f ? 1 / fps : 0, SWAY.hair, 1, out, 0);
    if (f % per === 0) seen.push([...out]);
  }
  return seen;
}

describe('sway', () => {
  test('the same swing at 30, 60 and 120 frames a second', () => {
    const [a, b, c] = [run(30, 4), run(60, 4), run(120, 4)];
    let most = 0;
    let far = 0;
    // (From half a second in: the walk begins at full speed here, and each rate first knows that speed a frame on.)
    for (let k = 5; k < a.length; k++) {
      for (let n = 0; n < 3; n++) {
        most = Math.max(most, Math.abs(a[k][n] - c[k][n]), Math.abs(b[k][n] - c[k][n]));
        far = Math.max(far, Math.abs(c[k][n]));
      }
    }
    // (It swings centimetres; the frame rates differ by under 2 mm: how finely each samples the walk's bob.)
    expect(far).toBeGreaterThan(0.02);
    expect(most).toBeLessThan(0.002);
  });

  test('trails a walker, swings on past a stop and comes to rest', () => {
    const seen = run(60, 6);
    // Walking (1 to 2 s): behind (-z). Just stopped: thrown forward. Four seconds on: nothing, exactly.
    for (let k = 10; k < 20; k++) expect(seen[k][2]).toBeLessThan(-0.005);
    expect(Math.max(...seen.slice(20, 25).map((s) => s[2]))).toBeGreaterThan(0.02);
    expect(seen[60]).toEqual([0, 0, 0]);
  });

  test('a resident standing still, breathing, draws none', () => {
    const state = new Float32Array(SWAY_FLOATS);
    const out = new Float32Array(3);
    for (let f = 0; f < 600; f++) {
      // (The idle's breath: 3 mm up and down, a little forward and back.)
      const t = f / 60;
      const drawn = swayStep(state, 0, 0, 1.5 + 0.003 * Math.sin(2 * Math.PI * 0.4 * t), 0.002 * Math.sin(2 * Math.PI * 0.25 * t), f ? 1 / 60 : 0, SWAY.hair, 1, out, 0);
      expect(drawn).toBe(false);
      expect([...out]).toEqual([0, 0, 0]);
    }
  });

  test('a pause holds it, and a jump starts it again at rest', () => {
    const state = new Float32Array(SWAY_FLOATS);
    const out = new Float32Array(3);
    for (let f = 0; f <= 90; f++) swayStep(state, 0, 0, 1.5, 1.3 * (f / 60), f ? 1 / 60 : 0, SWAY.hair, 1, out, 0);
    const held = [...out];
    expect(held[2]).toBeLessThan(-0.005);
    swayStep(state, 0, 0, 1.5, 1.3 * 1.5, 0, SWAY.hair, 1, out, 0);
    expect([...out]).toEqual(held);
    // (Put 5 m off in one frame: not flung.)
    swayStep(state, 0, 5, 1.5, 0, 1 / 60, SWAY.hair, 1, out, 0);
    expect([...out]).toEqual([0, 0, 0]);
    swayStep(state, 0, 5, 1.5, 0, 1 / 60, SWAY.hair, 1, out, 0);
    expect([...out]).toEqual([0, 0, 0]);
  });

  test('no further than its reach', () => {
    const state = new Float32Array(SWAY_FLOATS);
    const out = new Float32Array(3);
    // (A runner at 10 m/s stopping dead.)
    for (let f = 0; f <= 60; f++) swayStep(state, 0, 0, 1.5, 10 * (f / 60), f ? 1 / 60 : 0, SWAY.hair, 1, out, 0);
    for (let f = 0; f < 30; f++) {
      swayStep(state, 0, 0, 1.5, 10, 1 / 60, SWAY.hair, 1, out, 0);
      expect(Math.hypot(out[0], out[1], out[2])).toBeLessThanOrEqual(SWAY.hair.max + 1e-6);
    }
  });
});
