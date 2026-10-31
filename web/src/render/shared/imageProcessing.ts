// Ported from Babylon.js 9.30: `ImageProcessingConfiguration.bind` and `ColorCurves` (`_bind`,
// `getColorGradingDataToRef`, `FromHSBToRef`). Changed for Idyll Lives: pure functions that write
// the uniforms of the PlayCanvas port of the shader.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
/**
 * The CPU half of the game's image processing: the uniforms the image processing shader
 * (`render/pc/shaders/post.ts`, `applyImageProcessing`) reads, worked out from the look's settings.
 * The same numbers as the original's for the same settings, checked against the uniforms recorded
 * in `web/tests/fixtures` (`tests/imageProcessing.test.ts`).
 *
 * Pure: no engine, no DOM.
 */

/** The colour curves (hue in degrees 0..360; density, saturation and exposure -100..100). */
export interface ColorCurveSettings {
  globalHue: number;
  globalDensity: number;
  globalSaturation: number;
  globalExposure: number;
  highlightsHue: number;
  highlightsDensity: number;
  highlightsSaturation: number;
  highlightsExposure: number;
  midtonesHue: number;
  midtonesDensity: number;
  midtonesSaturation: number;
  midtonesExposure: number;
  shadowsHue: number;
  shadowsDensity: number;
  shadowsSaturation: number;
  shadowsExposure: number;
}

/** A new `ColorCurves`' settings: no change to the picture. */
export const NEUTRAL_CURVES: Readonly<ColorCurveSettings> = {
  globalHue: 30,
  globalDensity: 0,
  globalSaturation: 0,
  globalExposure: 0,
  highlightsHue: 30,
  highlightsDensity: 0,
  highlightsSaturation: 0,
  highlightsExposure: 0,
  midtonesHue: 30,
  midtonesDensity: 0,
  midtonesSaturation: 0,
  midtonesExposure: 0,
  shadowsHue: 30,
  shadowsDensity: 0,
  shadowsSaturation: 0,
  shadowsExposure: 0,
};

/** The curves' three uniforms (`vCameraColorCurveNeutral`, `...Positive`, `...Negative`), RGBA each. */
export interface CurveUniforms {
  neutral: Float32Array;
  positive: Float32Array;
  negative: Float32Array;
}

const clamp = (v: number, min: number, max: number) => Math.min(Math.max(v, min), max);

/** `ColorCurves._ApplyColorGradingSliderNonlinear`: finer control near 0. */
function sliderNonlinear(value: number): number {
  value /= 100;
  let x = Math.abs(value) ** 2;
  if (value < 0) x *= -1;
  return x * 100;
}

/** `ColorCurves._FromHSBToRef` (alpha 1). */
function fromHSB(hue: number, saturation: number, brightness: number, out: number[]): void {
  let h = clamp(hue, 0, 360);
  const s = clamp(saturation / 100, 0, 1);
  const v = clamp(brightness / 100, 0, 1);
  if (s === 0) {
    out[0] = out[1] = out[2] = v;
  } else {
    h /= 60;
    const i = Math.floor(h);
    const f = h - i;
    const p = v * (1 - s);
    const q = v * (1 - s * f);
    const t = v * (1 - s * (1 - f));
    const rgb = i === 0 ? [v, t, p] : i === 1 ? [q, v, p] : i === 2 ? [p, v, t] : i === 3 ? [p, q, v] : i === 4 ? [t, p, v] : [v, p, q];
    out[0] = rgb[0];
    out[1] = rgb[1];
    out[2] = rgb[2];
  }
  out[3] = 1;
}

/** `ColorCurves._getColorGradingDataToRef`: one band's filter colour (RGB) and saturation (A). */
function band(hue: number, density: number, saturation: number, exposure: number, out: number[]): void {
  hue = clamp(hue, 0, 360);
  density = clamp(density, -100, 100);
  saturation = clamp(saturation, -100, 100);
  exposure = clamp(exposure, -100, 100);
  // At most a 50 % filter, finer for small values.
  density = sliderNonlinear(density) * 0.5;
  exposure = sliderNonlinear(exposure);
  if (density < 0) {
    density *= -1;
    hue = (hue + 180) % 360;
  }
  fromHSB(hue, density, 50 + 0.25 * exposure, out);
  out[0] *= 2;
  out[1] *= 2;
  out[2] *= 2;
  out[3] = 1 + 0.01 * saturation;
}

/** The colour curves' uniforms for `c` (`ColorCurves._bind`): midtones as neutral, the highlights' and shadows' differences from it. */
export function curveUniforms(c: Readonly<ColorCurveSettings>, out: CurveUniforms): CurveUniforms {
  const global = [0, 0, 0, 0];
  const tmp = [0, 0, 0, 0];
  const high = [0, 0, 0, 0];
  const mid = [0, 0, 0, 0];
  const low = [0, 0, 0, 0];
  band(c.globalHue, c.globalDensity, c.globalSaturation, c.globalExposure, global);
  band(c.highlightsHue, c.highlightsDensity, c.highlightsSaturation, c.highlightsExposure, tmp);
  for (let i = 0; i < 4; i++) high[i] = tmp[i] * global[i];
  band(c.midtonesHue, c.midtonesDensity, c.midtonesSaturation, c.midtonesExposure, tmp);
  for (let i = 0; i < 4; i++) mid[i] = tmp[i] * global[i];
  band(c.shadowsHue, c.shadowsDensity, c.shadowsSaturation, c.shadowsExposure, tmp);
  for (let i = 0; i < 4; i++) low[i] = tmp[i] * global[i];
  for (let i = 0; i < 4; i++) {
    out.neutral[i] = mid[i];
    out.positive[i] = high[i] - mid[i];
    out.negative[i] = mid[i] - low[i];
  }
  return out;
}

/** The vignette (multiply blend). */
export interface VignetteSettings {
  /** Strength (`vignetteWeight`, 1.5 by default); 0 leaves the picture as it is. */
  weight: number;
  /** 0: an ellipse with the picture's proportions .. 1: a circle (`vignetteStretch`). */
  stretch: number;
  /** The colour the corners turn towards (linear, multiplied in before tone mapping). */
  color: readonly [number, number, number];
  /** `vignetteCameraFov`: 0.5 rad by default (the original's default), not the camera's own. */
  cameraFov?: number;
  centerX?: number;
  centerY?: number;
}

/**
 * The vignette's uniforms for a picture `width` × `height` pixels (`vignetteSettings1`: scale and
 * offset of the centred view position; `vignetteSettings2`: colour and power).
 */
export function vignetteUniforms(width: number, height: number, v: Readonly<VignetteSettings>, out1: Float32Array, out2: Float32Array): void {
  const aspect = width / Math.max(1, height);
  let scaleY = Math.tan((v.cameraFov ?? 0.5) * 0.5);
  let scaleX = scaleY * aspect;
  const mean = Math.sqrt(scaleX * scaleY);
  scaleX = scaleX * (1 - v.stretch) + mean * v.stretch;
  scaleY = scaleY * (1 - v.stretch) + mean * v.stretch;
  out1[0] = scaleX;
  out1[1] = scaleY;
  out1[2] = -scaleX * (v.centerX ?? 0);
  out1[3] = -scaleY * (v.centerY ?? 0);
  out2[0] = v.color[0];
  out2[1] = v.color[1];
  out2[2] = v.color[2];
  out2[3] = -2 * v.weight;
}
