// `render/shared/imageProcessing.ts` against Babylon.js: the colour curves' and the vignette's
// uniforms as Babylon's `ColorCurves` and `ImageProcessingConfiguration` bind them (`bun test`).
import { describe, expect, test } from 'bun:test';
import { ColorCurves } from '@babylonjs/core/Materials/colorCurves';
import { ImageProcessingConfiguration } from '@babylonjs/core/Materials/imageProcessingConfiguration';
import { Color4 } from '@babylonjs/core/Maths/math.color';
import { LOOK } from '../src/render/look';
import { curveUniforms, NEUTRAL_CURVES, vignetteUniforms, type ColorCurveSettings } from '../src/render/shared/imageProcessing';

/** What Babylon hands an effect: `setFloat4`/`setFloat2`/`setFloat` calls, by uniform. */
function recorder() {
  const values = new Map<string, number[]>();
  const effect = {
    setFloat4: (name: string, ...v: number[]) => values.set(name, v),
    setFloat2: (name: string, ...v: number[]) => values.set(name, v),
    setFloat: (name: string, v: number) => values.set(name, [v]),
    setMatrix3x3: () => {},
    getEngine: () => ({ getRenderWidth: () => 1920, getRenderHeight: () => 1080 }),
  };
  return { values, effect };
}

const close = (a: ArrayLike<number>, b: ArrayLike<number> | undefined, eps = 1e-6) => {
  expect(b).toBeDefined();
  expect(a.length).toBe(b!.length);
  for (let i = 0; i < a.length; i++) expect(Math.abs(a[i] - b![i])).toBeLessThan(eps);
};

/** A fixed sequence of numbers in 0..1 (the same every run). */
function numbers(seed: number): () => number {
  let s = seed >>> 0;
  return () => (s = (Math.imul(s, 1664525) + 1013904223) >>> 0) / 2 ** 32;
}

function babylonCurves(c: ColorCurveSettings) {
  const curves = new ColorCurves();
  Object.assign(curves, c);
  const { values, effect } = recorder();
  // SAFETY: `_bind` is the method `ImageProcessingConfiguration.bind` calls (`ColorCurvesBind`).
  (curves as unknown as { _bind(effect: unknown): void })._bind(effect);
  return values;
}

function ours(c: ColorCurveSettings) {
  return curveUniforms(c, { neutral: new Float32Array(4), positive: new Float32Array(4), negative: new Float32Array(4) });
}

describe('shared/imageProcessing matches Babylon', () => {
  test("the look's colour curves", () => {
    const c = { ...NEUTRAL_CURVES, globalSaturation: LOOK.saturation, highlightsHue: LOOK.highlightsHue, highlightsDensity: LOOK.highlightsDensity, shadowsHue: LOOK.shadowsHue, shadowsDensity: LOOK.shadowsDensity };
    const b = babylonCurves(c);
    const u = ours(c);
    close(u.neutral, b.get('vCameraColorCurveNeutral'));
    close(u.positive, b.get('vCameraColorCurvePositive'));
    close(u.negative, b.get('vCameraColorCurveNegative'));
  });

  test('neutral curves leave the picture as it is', () => {
    const u = ours(NEUTRAL_CURVES);
    close(u.neutral, [1, 1, 1, 1]);
    close(u.positive, [0, 0, 0, 0]);
    close(u.negative, [0, 0, 0, 0]);
  });

  test('curves over their whole range (negative densities, every hue sector)', () => {
    const r = numbers(11);
    for (let i = 0; i < 60; i++) {
      const c = { ...NEUTRAL_CURVES };
      for (const key of Object.keys(c) as (keyof ColorCurveSettings)[]) c[key] = key.endsWith('Hue') ? r() * 400 - 20 : r() * 240 - 120;
      const b = babylonCurves(c);
      const u = ours(c);
      close(u.neutral, b.get('vCameraColorCurveNeutral'));
      close(u.positive, b.get('vCameraColorCurvePositive'));
      close(u.negative, b.get('vCameraColorCurveNegative'));
    }
  });

  test("the game's vignette at several picture sizes", () => {
    for (const [w, h] of [[1920, 1080], [1280, 1024], [800, 1200], [1, 1]]) {
      const config = new ImageProcessingConfiguration();
      config.vignetteEnabled = true;
      config.vignetteStretch = 0.5;
      config.vignetteWeight = LOOK.vignette;
      config.vignetteColor = new Color4(0.02, 0.02, 0.04, 0);
      config.outputTextureWidth = w;
      config.outputTextureHeight = h;
      const { values, effect } = recorder();
      // (The post-process hands its own aspect, its texture's: the same picture size.)
      config.bind(effect as never, w / h);
      const one = new Float32Array(4);
      const two = new Float32Array(4);
      vignetteUniforms(w, h, { weight: LOOK.vignette, stretch: 0.5, color: [0.02, 0.02, 0.04] }, one, two);
      close(one, values.get('vignetteSettings1'));
      close(two, values.get('vignetteSettings2'));
    }
  });
});
