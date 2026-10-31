// `render/shared/imageProcessing.ts` against Babylon.js: the colour curves' and the vignette's uniforms as
// Babylon's `ColorCurves` and `ImageProcessingConfiguration` bind them (`bun test`). Babylon's uniforms, for
// the settings recorded with them, are fixtures recorded from Babylon 9.30 (`fixtures/babylon/imageProcessing.json`,
// made by `fixtures/record-babylon.ts`, which needs a Babylon reference checkout to run again).
import { describe, expect, test } from 'bun:test';
import { curveUniforms, NEUTRAL_CURVES, vignetteUniforms, type ColorCurveSettings } from '../src/render/shared/imageProcessing';
import { fixture } from './fixtures/babylon';

interface Curves {
  settings: ColorCurveSettings;
  neutral: number[];
  positive: number[];
  negative: number[];
}
const B = fixture<{
  look: Curves;
  neutral: Curves;
  curves: Curves[];
  vignette: { w: number; h: number; weight: number; stretch: number; color: [number, number, number]; settings1: number[]; settings2: number[] }[];
}>('imageProcessing');

const close = (a: ArrayLike<number>, b: ArrayLike<number> | undefined, eps = 1e-6) => {
  expect(b).toBeDefined();
  expect(a.length).toBe(b!.length);
  for (let i = 0; i < a.length; i++) expect(Math.abs(a[i] - b![i])).toBeLessThan(eps);
};

function same(c: Curves) {
  // (Every setting the curves take, recorded.)
  expect(Object.keys(c.settings).sort()).toEqual(Object.keys(NEUTRAL_CURVES).sort());
  const u = curveUniforms(c.settings, { neutral: new Float32Array(4), positive: new Float32Array(4), negative: new Float32Array(4) });
  close(u.neutral, c.neutral);
  close(u.positive, c.positive);
  close(u.negative, c.negative);
}

describe('shared/imageProcessing matches Babylon', () => {
  test("the look's colour curves (as they were when recorded)", () => same(B.look));

  test('neutral curves leave the picture as it is', () => {
    same(B.neutral);
    const u = curveUniforms(NEUTRAL_CURVES, { neutral: new Float32Array(4), positive: new Float32Array(4), negative: new Float32Array(4) });
    close(u.neutral, [1, 1, 1, 1]);
    close(u.positive, [0, 0, 0, 0]);
    close(u.negative, [0, 0, 0, 0]);
  });

  test('curves over their whole range (negative densities, every hue sector)', () => {
    expect(B.curves.length).toBe(60);
    for (const c of B.curves) same(c);
  });

  test("the game's vignette, and others, at several picture sizes", () => {
    expect(B.vignette.length).toBe(16);
    for (const { w, h, weight, stretch, color, settings1, settings2 } of B.vignette) {
      const one = new Float32Array(4);
      const two = new Float32Array(4);
      vignetteUniforms(w, h, { weight, stretch, color }, one, two);
      close(one, settings1);
      close(two, settings2);
    }
  });
});
