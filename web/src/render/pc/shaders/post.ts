/**
 * The picture after the light, in shader code (the port of Babylon.js 9.30's image processing,
 * `imageProcessingFunctions.fx` and `fxaa.fragment.fx`, which `post.ts` puts on the camera):
 *
 * - `IMAGE_PROCESSING`: Babylon's `applyImageProcessing` in its own order: exposure; the vignette
 *   (multiplied, before tone mapping); ACES (`ACESFitted`, Babylon's, without the 1 / 0.6 that
 *   PlayCanvas's `TONEMAP_ACES2` adds); to gamma space as Babylon goes (`pow(x, 1 / 2.2)`, not the
 *   sRGB curve) and clamped; contrast; the colour curves (analytic, no LUT). Every step always
 *   runs, driven by uniforms: Babylon's steps it leaves out by a define (exposure 1, contrast 1, no
 *   vignette, neutral curves) are exactly no change here, so the look changes live without a
 *   shader compiling. Its uniforms: `shared/imageProcessing.ts`.
 * - `composeChunks`: hosts it in PlayCanvas's compose pass (`composeDeclarationsPS`,
 *   `composeMainEndPS`), on the scene's linear colour (the compose's own tone mapping is
 *   `TONEMAP_NONE`, its vignette and LUT off), with the night grade after it (`nightGrade.ts`). The
 *   compose then encodes for its target (`gammaCorrectOutput`, the same `pow(x, 1 / 2.2)`), so the
 *   chunk hands it Babylon's display colour decoded the same way.
 * - `FXAA`: Babylon's FXAA 3.11 pass (quality preset 12), verbatim, on the image-processed colour,
 *   with the night grade at its end where it runs (Babylon's order: image processing, FXAA, grade).
 * - `PAGE_TONE_MAPPING`: the dev pages' look (Babylon's pages applied the image processing in
 *   each material: exposure and ACES only), as `tonemappingAces2PS`.
 *
 * GLSL (WebGL2) and WGSL (WebGPU) side by side. The engine lines this relies on are checked by
 * `tools/render/check_chunks.ts`.
 */

import { NIGHT_GRADE } from '../nightGrade';
import type { Language } from './chunks';

/** Babylon's `ACESFitted` (Stephen Hill's fit, MJP's BakingLab `ACES.hlsl`): input matrix, RRT + ODT fit, output matrix, clamped. */
export const ACES: Record<Language, string> = {
  glsl: `
const mat3 gameAcesIn = mat3(vec3(0.59719, 0.07600, 0.02840), vec3(0.35458, 0.90834, 0.13383), vec3(0.04823, 0.01566, 0.83777));
const mat3 gameAcesOut = mat3(vec3(1.60475, -0.10208, -0.00327), vec3(-0.53108, 1.10813, -0.07276), vec3(-0.07367, -0.00605, 1.07602));
vec3 gameRrtAndOdtFit(vec3 v) {
	vec3 a = v * (v + 0.0245786) - 0.000090537;
	vec3 b = v * (0.983729 * v + 0.4329510) + 0.238081;
	return a / b;
}
vec3 gameAcesFitted(vec3 color) {
	color = gameAcesIn * color;
	color = gameRrtAndOdtFit(color);
	color = gameAcesOut * color;
	return clamp(color, 0.0, 1.0);
}
`,
  wgsl: `
const gameAcesIn = mat3x3f(vec3f(0.59719, 0.07600, 0.02840), vec3f(0.35458, 0.90834, 0.13383), vec3f(0.04823, 0.01566, 0.83777));
const gameAcesOut = mat3x3f(vec3f(1.60475, -0.10208, -0.00327), vec3f(-0.53108, 1.10813, -0.07276), vec3f(-0.07367, -0.00605, 1.07602));
fn gameRrtAndOdtFit(v: vec3f) -> vec3f {
	let a = v * (v + 0.0245786) - 0.000090537;
	let b = v * (0.983729 * v + 0.4329510) + 0.238081;
	return a / b;
}
fn gameAcesFitted(color: vec3f) -> vec3f {
	var c = gameAcesIn * color;
	c = gameRrtAndOdtFit(c);
	c = gameAcesOut * c;
	return clamp(c, vec3f(0.0), vec3f(1.0));
}
`,
};

/**
 * `gameImageProcessing(linear, uv)`: Babylon's display colour (gamma space) for the scene's linear
 * colour at `uv` (0..1 across the picture; the vignette is symmetric, so either way up). Uniforms:
 * `gameExposure`, `gameContrast`, `gameVignette1`/`2` (Babylon's `vignetteSettings1`/`2`) and
 * `gameCurveNeutral`/`Positive`/`Negative` (`vCameraColorCurve*`).
 */
export const IMAGE_PROCESSING: Record<Language, string> = {
  glsl: `
uniform float gameExposure;
uniform float gameContrast;
uniform vec4 gameVignette1;
uniform vec4 gameVignette2;
uniform vec4 gameCurveNeutral;
uniform vec4 gameCurvePositive;
uniform vec4 gameCurveNegative;
${ACES.glsl}
vec3 gameImageProcessing(vec3 rgb, vec2 uv) {
	rgb *= gameExposure;
	vec2 viewportXY = uv * 2.0 - 1.0;
	vec3 vignetteXY1 = vec3(viewportXY * gameVignette1.xy + gameVignette1.zw, 1.0);
	float vignette = pow(dot(vignetteXY1, vignetteXY1), gameVignette2.w);
	rgb *= mix(gameVignette2.rgb, vec3(1.0), vignette);
	rgb = gameAcesFitted(rgb);
	rgb = clamp(pow(rgb, vec3(1.0 / 2.2)), 0.0, 1.0);
	vec3 highContrast = rgb * rgb * (3.0 - 2.0 * rgb);
	if (gameContrast < 1.0) {
		rgb = mix(vec3(0.5), rgb, gameContrast);
	} else {
		rgb = mix(rgb, highContrast, gameContrast - 1.0);
	}
	rgb = max(rgb, 0.0);
	float luma = clamp(dot(rgb, vec3(0.2126, 0.7152, 0.0722)), 0.0, 1.0);
	vec2 curveMix = clamp(vec2(luma * 3.0 - 1.5, luma * -3.0 + 1.5), vec2(0.0), vec2(1.0));
	vec4 colorCurve = gameCurveNeutral + curveMix.x * gameCurvePositive - curveMix.y * gameCurveNegative;
	rgb *= colorCurve.rgb;
	return mix(vec3(luma), rgb, colorCurve.a);
}
`,
  wgsl: `
uniform gameExposure: f32;
uniform gameContrast: f32;
uniform gameVignette1: vec4f;
uniform gameVignette2: vec4f;
uniform gameCurveNeutral: vec4f;
uniform gameCurvePositive: vec4f;
uniform gameCurveNegative: vec4f;
${ACES.wgsl}
fn gameImageProcessing(color: vec3f, uv: vec2f) -> vec3f {
	var rgb = color * uniform.gameExposure;
	let viewportXY = uv * 2.0 - 1.0;
	let vignetteXY1 = vec3f(viewportXY * uniform.gameVignette1.xy + uniform.gameVignette1.zw, 1.0);
	let vignette = pow(dot(vignetteXY1, vignetteXY1), uniform.gameVignette2.w);
	rgb = rgb * mix(uniform.gameVignette2.rgb, vec3f(1.0), vignette);
	rgb = gameAcesFitted(rgb);
	rgb = clamp(pow(rgb, vec3f(1.0 / 2.2)), vec3f(0.0), vec3f(1.0));
	let highContrast = rgb * rgb * (3.0 - 2.0 * rgb);
	if (uniform.gameContrast < 1.0) {
		rgb = mix(vec3f(0.5), rgb, uniform.gameContrast);
	} else {
		rgb = mix(rgb, highContrast, uniform.gameContrast - 1.0);
	}
	rgb = max(rgb, vec3f(0.0));
	let luma = clamp(dot(rgb, vec3f(0.2126, 0.7152, 0.0722)), 0.0, 1.0);
	let curveMix = clamp(vec2f(luma * 3.0 - 1.5, luma * -3.0 + 1.5), vec2f(0.0), vec2f(1.0));
	let colorCurve = uniform.gameCurveNeutral + curveMix.x * uniform.gameCurvePositive - curveMix.y * uniform.gameCurveNegative;
	rgb = rgb * colorCurve.rgb;
	return mix(vec3f(luma), rgb, colorCurve.a);
}
`,
};

/**
 * The exact sRGB decode, for a target the hardware encodes (an sRGB format, `GAMMA` `NONE`): the
 * colour written is then the display colour given. (The game's targets aren't sRGB: the compose
 * encodes with `pow(x, 1 / 2.2)` itself, which `pow(x, 2.2)` undoes.)
 */
const SRGB_DECODE: Record<Language, string> = {
  glsl: `
vec3 gameSrgbDecode(vec3 c) {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(vec3(0.04045), c));
}
`,
  wgsl: `
fn gameSrgbDecode(c: vec3f) -> vec3f {
	return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3f(2.4)), step(vec3f(0.04045), c));
}
`,
};

export interface ComposeOptions {
  /** Babylon's image processing (off: the diag views and `?diag=linear`, which show the colour itself). */
  imageProcessing: boolean;
  /** The night grade at the end (off where the FXAA pass has it, and with `?diag=nonight`). */
  night: boolean;
  /** The compose draws for the FXAA pass: the colour unclamped above 1, as Babylon's half-float FXAA input had it. */
  toFxaa: boolean;
}

/** The compose pass's chunks for `options` (`composeDeclarationsPS`, `composeMainEndPS`; empty: the compose as PlayCanvas has it). */
export function composeChunks(lang: Language, options: ComposeOptions): { declarations: string; mainEnd: string } {
  if (!options.imageProcessing) return { declarations: '', mainEnd: '' };
  const declarations = `${IMAGE_PROCESSING[lang]}${options.night ? NIGHT_GRADE[lang] : ''}${SRGB_DECODE[lang]}`;
  const v3 = lang === 'glsl' ? 'vec3' : 'vec3f';
  // (Babylon's next pass read the image-processed colour from a half-float texture before the
  // FXAA, an 8-bit one before the grade; the canvas clamps. Negative values, which the compose's
  // encoding can't take, are left out for the FXAA too.)
  const range = options.toFxaa ? `max(result, ${v3}(0.0))` : `clamp(result, ${v3}(0.0), ${v3}(1.0))`;
  const mainEnd = `
	result = gameImageProcessing(result, uv);
	result = ${range};
	${options.night ? 'result = gameNightGrade(result);' : ''}
	#if (GAMMA == SRGB)
		result = pow(result, ${v3}(2.2));
	#else
		result = gameSrgbDecode(result);
	#endif
`;
  return { declarations, mainEnd };
}

/**
 * Babylon's FXAA fragment shader (quality 12, `fxaa.fragment.fx`; `MALI` where Babylon set it),
 * reading `gameFxaaSource` (`gameFxaaTexel`: one texel), its eight neighbour coordinates worked
 * out here rather than in the vertex shader. `GAME_NIGHT`: the night grade on the result (clamped
 * to 0..1 first, as Babylon's grade read it); `GAME_SRGB_TARGET`: the target encodes itself.
 */
export const FXAA: Record<Language, string> = {
  glsl: `
uniform sampler2D gameFxaaSource;
uniform vec2 gameFxaaTexel;
varying vec2 uv0;
#ifdef GAME_NIGHT
${NIGHT_GRADE.glsl}
#endif
${SRGB_DECODE.glsl}
const float fxaaQualitySubpix = 1.0;
const float fxaaQualityEdgeThreshold = 0.166;
const float fxaaQualityEdgeThresholdMin = 0.0833;
const vec3 kLumaCoefficients = vec3(0.2126, 0.7152, 0.0722);
#define FxaaLuma(rgba) dot(rgba.rgb, kLumaCoefficients)
#define TEXTUREFUNC(c) texture2DLod(gameFxaaSource, c, 0.0)
vec4 gameFxaaOut(vec4 c) {
	vec3 rgb = c.rgb;
	#ifdef GAME_NIGHT
		rgb = gameNightGrade(clamp(rgb, 0.0, 1.0));
	#endif
	#ifdef GAME_SRGB_TARGET
		rgb = gameSrgbDecode(clamp(rgb, 0.0, 1.0));
	#endif
	return vec4(rgb, c.a);
}
void main() {
	vec2 texelSize = gameFxaaTexel;
	vec2 vUV = uv0;
	vec2 sampleCoordS = vUV + vec2(0.0, 1.0) * texelSize;
	vec2 sampleCoordE = vUV + vec2(1.0, 0.0) * texelSize;
	vec2 sampleCoordN = vUV + vec2(0.0, -1.0) * texelSize;
	vec2 sampleCoordW = vUV + vec2(-1.0, 0.0) * texelSize;
	vec2 sampleCoordNW = vUV + vec2(-1.0, -1.0) * texelSize;
	vec2 sampleCoordSE = vUV + vec2(1.0, 1.0) * texelSize;
	vec2 sampleCoordNE = vUV + vec2(1.0, -1.0) * texelSize;
	vec2 sampleCoordSW = vUV + vec2(-1.0, 1.0) * texelSize;
	vec2 posM;
	posM.x = vUV.x;
	posM.y = vUV.y;
	vec4 rgbyM = TEXTUREFUNC(vUV);
	float lumaM = FxaaLuma(rgbyM);
	float lumaS = FxaaLuma(TEXTUREFUNC(sampleCoordS));
	float lumaE = FxaaLuma(TEXTUREFUNC(sampleCoordE));
	float lumaN = FxaaLuma(TEXTUREFUNC(sampleCoordN));
	float lumaW = FxaaLuma(TEXTUREFUNC(sampleCoordW));
	float maxSM = max(lumaS, lumaM);
	float minSM = min(lumaS, lumaM);
	float maxESM = max(lumaE, maxSM);
	float minESM = min(lumaE, minSM);
	float maxWN = max(lumaN, lumaW);
	float minWN = min(lumaN, lumaW);
	float rangeMax = max(maxWN, maxESM);
	float rangeMin = min(minWN, minESM);
	float rangeMaxScaled = rangeMax * fxaaQualityEdgeThreshold;
	float range = rangeMax - rangeMin;
	float rangeMaxClamped = max(fxaaQualityEdgeThresholdMin, rangeMaxScaled);
	#ifndef MALI
	if (range < rangeMaxClamped) {
		gl_FragColor = gameFxaaOut(rgbyM);
		return;
	}
	#endif
	float lumaNW = FxaaLuma(TEXTUREFUNC(sampleCoordNW));
	float lumaSE = FxaaLuma(TEXTUREFUNC(sampleCoordSE));
	float lumaNE = FxaaLuma(TEXTUREFUNC(sampleCoordNE));
	float lumaSW = FxaaLuma(TEXTUREFUNC(sampleCoordSW));
	float lumaNS = lumaN + lumaS;
	float lumaWE = lumaW + lumaE;
	float subpixRcpRange = 1.0 / range;
	float subpixNSWE = lumaNS + lumaWE;
	float edgeHorz1 = (-2.0 * lumaM) + lumaNS;
	float edgeVert1 = (-2.0 * lumaM) + lumaWE;
	float lumaNESE = lumaNE + lumaSE;
	float lumaNWNE = lumaNW + lumaNE;
	float edgeHorz2 = (-2.0 * lumaE) + lumaNESE;
	float edgeVert2 = (-2.0 * lumaN) + lumaNWNE;
	float lumaNWSW = lumaNW + lumaSW;
	float lumaSWSE = lumaSW + lumaSE;
	float edgeHorz4 = (abs(edgeHorz1) * 2.0) + abs(edgeHorz2);
	float edgeVert4 = (abs(edgeVert1) * 2.0) + abs(edgeVert2);
	float edgeHorz3 = (-2.0 * lumaW) + lumaNWSW;
	float edgeVert3 = (-2.0 * lumaS) + lumaSWSE;
	float edgeHorz = abs(edgeHorz3) + edgeHorz4;
	float edgeVert = abs(edgeVert3) + edgeVert4;
	float subpixNWSWNESE = lumaNWSW + lumaNESE;
	float lengthSign = texelSize.x;
	bool horzSpan = edgeHorz >= edgeVert;
	float subpixA = subpixNSWE * 2.0 + subpixNWSWNESE;
	if (!horzSpan) lumaN = lumaW;
	if (!horzSpan) lumaS = lumaE;
	if (horzSpan) lengthSign = texelSize.y;
	float subpixB = (subpixA * (1.0 / 12.0)) - lumaM;
	float gradientN = lumaN - lumaM;
	float gradientS = lumaS - lumaM;
	float lumaNN = lumaN + lumaM;
	float lumaSS = lumaS + lumaM;
	bool pairN = abs(gradientN) >= abs(gradientS);
	float gradient = max(abs(gradientN), abs(gradientS));
	if (pairN) lengthSign = -lengthSign;
	float subpixC = clamp(abs(subpixB) * subpixRcpRange, 0.0, 1.0);
	vec2 posB;
	posB.x = posM.x;
	posB.y = posM.y;
	vec2 offNP;
	offNP.x = (!horzSpan) ? 0.0 : texelSize.x;
	offNP.y = (horzSpan) ? 0.0 : texelSize.y;
	if (!horzSpan) posB.x += lengthSign * 0.5;
	if (horzSpan) posB.y += lengthSign * 0.5;
	vec2 posN;
	posN.x = posB.x - offNP.x * 1.5;
	posN.y = posB.y - offNP.y * 1.5;
	vec2 posP;
	posP.x = posB.x + offNP.x * 1.5;
	posP.y = posB.y + offNP.y * 1.5;
	float subpixD = ((-2.0) * subpixC) + 3.0;
	float lumaEndN = FxaaLuma(TEXTUREFUNC(posN));
	float subpixE = subpixC * subpixC;
	float lumaEndP = FxaaLuma(TEXTUREFUNC(posP));
	if (!pairN) lumaNN = lumaSS;
	float gradientScaled = gradient * 1.0 / 4.0;
	float lumaMM = lumaM - lumaNN * 0.5;
	float subpixF = subpixD * subpixE;
	bool lumaMLTZero = lumaMM < 0.0;
	lumaEndN -= lumaNN * 0.5;
	lumaEndP -= lumaNN * 0.5;
	bool doneN = abs(lumaEndN) >= gradientScaled;
	bool doneP = abs(lumaEndP) >= gradientScaled;
	if (!doneN) posN.x -= offNP.x * 3.0;
	if (!doneN) posN.y -= offNP.y * 3.0;
	bool doneNP = (!doneN) || (!doneP);
	if (!doneP) posP.x += offNP.x * 3.0;
	if (!doneP) posP.y += offNP.y * 3.0;
	if (doneNP) {
		if (!doneN) lumaEndN = FxaaLuma(TEXTUREFUNC(posN.xy));
		if (!doneP) lumaEndP = FxaaLuma(TEXTUREFUNC(posP.xy));
		if (!doneN) lumaEndN = lumaEndN - lumaNN * 0.5;
		if (!doneP) lumaEndP = lumaEndP - lumaNN * 0.5;
		doneN = abs(lumaEndN) >= gradientScaled;
		doneP = abs(lumaEndP) >= gradientScaled;
		if (!doneN) posN.x -= offNP.x * 12.0;
		if (!doneN) posN.y -= offNP.y * 12.0;
		doneNP = (!doneN) || (!doneP);
		if (!doneP) posP.x += offNP.x * 12.0;
		if (!doneP) posP.y += offNP.y * 12.0;
	}
	float dstN = posM.x - posN.x;
	float dstP = posP.x - posM.x;
	if (!horzSpan) dstN = posM.y - posN.y;
	if (!horzSpan) dstP = posP.y - posM.y;
	bool goodSpanN = (lumaEndN < 0.0) != lumaMLTZero;
	float spanLength = (dstP + dstN);
	bool goodSpanP = (lumaEndP < 0.0) != lumaMLTZero;
	float spanLengthRcp = 1.0 / spanLength;
	bool directionN = dstN < dstP;
	float dst = min(dstN, dstP);
	bool goodSpan = directionN ? goodSpanN : goodSpanP;
	float subpixG = subpixF * subpixF;
	float pixelOffset = (dst * (-spanLengthRcp)) + 0.5;
	float subpixH = subpixG * fxaaQualitySubpix;
	float pixelOffsetGood = goodSpan ? pixelOffset : 0.0;
	float pixelOffsetSubpix = max(pixelOffsetGood, subpixH);
	if (!horzSpan) posM.x += pixelOffsetSubpix * lengthSign;
	if (horzSpan) posM.y += pixelOffsetSubpix * lengthSign;
	#ifdef MALI
	if (range < rangeMaxClamped) {
		gl_FragColor = gameFxaaOut(rgbyM);
	} else {
		gl_FragColor = gameFxaaOut(TEXTUREFUNC(posM));
	}
	#else
	gl_FragColor = gameFxaaOut(TEXTUREFUNC(posM));
	#endif
}
`,
  wgsl: `
var gameFxaaSource: texture_2d<f32>;
var gameFxaaSourceSampler: sampler;
uniform gameFxaaTexel: vec2f;
varying uv0: vec2f;
#ifdef GAME_NIGHT
${NIGHT_GRADE.wgsl}
#endif
${SRGB_DECODE.wgsl}
const fxaaQualitySubpix: f32 = 1.0;
const fxaaQualityEdgeThreshold: f32 = 0.166;
const fxaaQualityEdgeThresholdMin: f32 = 0.0833;
const kLumaCoefficients: vec3f = vec3f(0.2126, 0.7152, 0.0722);
fn FxaaLuma(rgba: vec4f) -> f32 { return dot(rgba.rgb, kLumaCoefficients); }
fn gameFxaaTex(c: vec2f) -> vec4f { return textureSampleLevel(gameFxaaSource, gameFxaaSourceSampler, c, 0.0); }
fn gameFxaaOut(c: vec4f) -> vec4f {
	var rgb = c.rgb;
	#ifdef GAME_NIGHT
		rgb = gameNightGrade(clamp(rgb, vec3f(0.0), vec3f(1.0)));
	#endif
	#ifdef GAME_SRGB_TARGET
		rgb = gameSrgbDecode(clamp(rgb, vec3f(0.0), vec3f(1.0)));
	#endif
	return vec4f(rgb, c.a);
}
@fragment
fn fragmentMain(input: FragmentInput) -> FragmentOutput {
	var output: FragmentOutput;
	let texelSize = uniform.gameFxaaTexel;
	let vUV = uv0;
	let sampleCoordS = vUV + vec2f(0.0, 1.0) * texelSize;
	let sampleCoordE = vUV + vec2f(1.0, 0.0) * texelSize;
	let sampleCoordN = vUV + vec2f(0.0, -1.0) * texelSize;
	let sampleCoordW = vUV + vec2f(-1.0, 0.0) * texelSize;
	let sampleCoordNW = vUV + vec2f(-1.0, -1.0) * texelSize;
	let sampleCoordSE = vUV + vec2f(1.0, 1.0) * texelSize;
	let sampleCoordNE = vUV + vec2f(1.0, -1.0) * texelSize;
	let sampleCoordSW = vUV + vec2f(-1.0, 1.0) * texelSize;
	var posM: vec2f;
	posM.x = vUV.x;
	posM.y = vUV.y;
	let rgbyM = gameFxaaTex(vUV);
	let lumaM = FxaaLuma(rgbyM);
	var lumaS = FxaaLuma(gameFxaaTex(sampleCoordS));
	let lumaE = FxaaLuma(gameFxaaTex(sampleCoordE));
	var lumaN = FxaaLuma(gameFxaaTex(sampleCoordN));
	let lumaW = FxaaLuma(gameFxaaTex(sampleCoordW));
	let maxSM = max(lumaS, lumaM);
	let minSM = min(lumaS, lumaM);
	let maxESM = max(lumaE, maxSM);
	let minESM = min(lumaE, minSM);
	let maxWN = max(lumaN, lumaW);
	let minWN = min(lumaN, lumaW);
	let rangeMax = max(maxWN, maxESM);
	let rangeMin = min(minWN, minESM);
	let rangeMaxScaled = rangeMax * fxaaQualityEdgeThreshold;
	let range = rangeMax - rangeMin;
	let rangeMaxClamped = max(fxaaQualityEdgeThresholdMin, rangeMaxScaled);
	#ifndef MALI
	if (range < rangeMaxClamped) {
		output.color = gameFxaaOut(rgbyM);
		return output;
	}
	#endif
	let lumaNW = FxaaLuma(gameFxaaTex(sampleCoordNW));
	let lumaSE = FxaaLuma(gameFxaaTex(sampleCoordSE));
	let lumaNE = FxaaLuma(gameFxaaTex(sampleCoordNE));
	let lumaSW = FxaaLuma(gameFxaaTex(sampleCoordSW));
	let lumaNS = lumaN + lumaS;
	let lumaWE = lumaW + lumaE;
	let subpixRcpRange = 1.0 / range;
	let subpixNSWE = lumaNS + lumaWE;
	let edgeHorz1 = (-2.0 * lumaM) + lumaNS;
	let edgeVert1 = (-2.0 * lumaM) + lumaWE;
	let lumaNESE = lumaNE + lumaSE;
	let lumaNWNE = lumaNW + lumaNE;
	let edgeHorz2 = (-2.0 * lumaE) + lumaNESE;
	let edgeVert2 = (-2.0 * lumaN) + lumaNWNE;
	let lumaNWSW = lumaNW + lumaSW;
	let lumaSWSE = lumaSW + lumaSE;
	let edgeHorz4 = (abs(edgeHorz1) * 2.0) + abs(edgeHorz2);
	let edgeVert4 = (abs(edgeVert1) * 2.0) + abs(edgeVert2);
	let edgeHorz3 = (-2.0 * lumaW) + lumaNWSW;
	let edgeVert3 = (-2.0 * lumaS) + lumaSWSE;
	let edgeHorz = abs(edgeHorz3) + edgeHorz4;
	let edgeVert = abs(edgeVert3) + edgeVert4;
	let subpixNWSWNESE = lumaNWSW + lumaNESE;
	var lengthSign = texelSize.x;
	let horzSpan = edgeHorz >= edgeVert;
	let subpixA = subpixNSWE * 2.0 + subpixNWSWNESE;
	if (!horzSpan) { lumaN = lumaW; }
	if (!horzSpan) { lumaS = lumaE; }
	if (horzSpan) { lengthSign = texelSize.y; }
	let subpixB = (subpixA * (1.0 / 12.0)) - lumaM;
	let gradientN = lumaN - lumaM;
	let gradientS = lumaS - lumaM;
	var lumaNN = lumaN + lumaM;
	let lumaSS = lumaS + lumaM;
	let pairN = abs(gradientN) >= abs(gradientS);
	let gradient = max(abs(gradientN), abs(gradientS));
	if (pairN) { lengthSign = -lengthSign; }
	let subpixC = clamp(abs(subpixB) * subpixRcpRange, 0.0, 1.0);
	var posB: vec2f;
	posB.x = posM.x;
	posB.y = posM.y;
	var offNP: vec2f;
	offNP.x = select(texelSize.x, 0.0, (!horzSpan));
	offNP.y = select(texelSize.y, 0.0, (horzSpan));
	if (!horzSpan) { posB.x += lengthSign * 0.5; }
	if (horzSpan) { posB.y += lengthSign * 0.5; }
	var posN: vec2f;
	posN.x = posB.x - offNP.x * 1.5;
	posN.y = posB.y - offNP.y * 1.5;
	var posP: vec2f;
	posP.x = posB.x + offNP.x * 1.5;
	posP.y = posB.y + offNP.y * 1.5;
	let subpixD = ((-2.0) * subpixC) + 3.0;
	var lumaEndN = FxaaLuma(gameFxaaTex(posN));
	let subpixE = subpixC * subpixC;
	var lumaEndP = FxaaLuma(gameFxaaTex(posP));
	if (!pairN) { lumaNN = lumaSS; }
	let gradientScaled = gradient * 1.0 / 4.0;
	let lumaMM = lumaM - lumaNN * 0.5;
	let subpixF = subpixD * subpixE;
	let lumaMLTZero = lumaMM < 0.0;
	lumaEndN -= lumaNN * 0.5;
	lumaEndP -= lumaNN * 0.5;
	var doneN = abs(lumaEndN) >= gradientScaled;
	var doneP = abs(lumaEndP) >= gradientScaled;
	if (!doneN) { posN.x -= offNP.x * 3.0; }
	if (!doneN) { posN.y -= offNP.y * 3.0; }
	var doneNP = (!doneN) || (!doneP);
	if (!doneP) { posP.x += offNP.x * 3.0; }
	if (!doneP) { posP.y += offNP.y * 3.0; }
	if (doneNP) {
		if (!doneN) { lumaEndN = FxaaLuma(gameFxaaTex(posN.xy)); }
		if (!doneP) { lumaEndP = FxaaLuma(gameFxaaTex(posP.xy)); }
		if (!doneN) { lumaEndN = lumaEndN - lumaNN * 0.5; }
		if (!doneP) { lumaEndP = lumaEndP - lumaNN * 0.5; }
		doneN = abs(lumaEndN) >= gradientScaled;
		doneP = abs(lumaEndP) >= gradientScaled;
		if (!doneN) { posN.x -= offNP.x * 12.0; }
		if (!doneN) { posN.y -= offNP.y * 12.0; }
		doneNP = (!doneN) || (!doneP);
		if (!doneP) { posP.x += offNP.x * 12.0; }
		if (!doneP) { posP.y += offNP.y * 12.0; }
	}
	var dstN = posM.x - posN.x;
	var dstP = posP.x - posM.x;
	if (!horzSpan) { dstN = posM.y - posN.y; }
	if (!horzSpan) { dstP = posP.y - posM.y; }
	let goodSpanN = (lumaEndN < 0.0) != lumaMLTZero;
	let spanLength = (dstP + dstN);
	let goodSpanP = (lumaEndP < 0.0) != lumaMLTZero;
	let spanLengthRcp = 1.0 / spanLength;
	let directionN = dstN < dstP;
	let dst = min(dstN, dstP);
	let goodSpan = select(goodSpanP, goodSpanN, directionN);
	let subpixG = subpixF * subpixF;
	let pixelOffset = (dst * (-spanLengthRcp)) + 0.5;
	let subpixH = subpixG * fxaaQualitySubpix;
	let pixelOffsetGood = select(0.0, pixelOffset, goodSpan);
	let pixelOffsetSubpix = max(pixelOffsetGood, subpixH);
	if (!horzSpan) { posM.x += pixelOffsetSubpix * lengthSign; }
	if (horzSpan) { posM.y += pixelOffsetSubpix * lengthSign; }
	#ifdef MALI
	if (range < rangeMaxClamped) {
		output.color = gameFxaaOut(rgbyM);
	} else {
		output.color = gameFxaaOut(gameFxaaTex(posM));
	}
	#else
	output.color = gameFxaaOut(gameFxaaTex(posM));
	#endif
	return output;
}
`,
};

/**
 * The dev pages' tone mapping (`tonemappingAces2PS` replaced on their device): Babylon's pages
 * had the image processing in each material, exposure then ACES (`scene.exposure`, the engine's
 * `getExposure()`); the engine's gamma output after it is Babylon's `pow(x, 1 / 2.2)`.
 */
export const PAGE_TONE_MAPPING: Record<Language, string> = {
  glsl: `${ACES.glsl}
vec3 toneMap(vec3 color) {
	return gameAcesFitted(color * getExposure());
}
`,
  wgsl: `${ACES.wgsl}
fn toneMap(color: vec3f) -> vec3f {
	return gameAcesFitted(color * getExposure());
}
`,
};

/**
 * `PAGE_TONE_MAPPING` with Babylon's contrast after it (`ImageProcessingConfiguration.contrast`,
 * in the material as the previews' scenes had it): in gamma space, after the clamp, and back to
 * linear for the engine's gamma output (`pow(x, 2.2)` undoes its `pow(x, 1 / 2.2)`). `contrast`
 * goes into the code as a constant.
 */
export function pageToneMapping(contrast: number): Record<Language, string> {
  const k = contrast.toFixed(4);
  return {
    glsl: `${ACES.glsl}
vec3 toneMap(vec3 color) {
	vec3 rgb = clamp(pow(gameAcesFitted(color * getExposure()), vec3(1.0 / 2.2)), 0.0, 1.0);
	vec3 highContrast = rgb * rgb * (3.0 - 2.0 * rgb);
	rgb = ${contrast < 1 ? `mix(vec3(0.5), rgb, ${k})` : `mix(rgb, highContrast, ${k} - 1.0)`};
	return pow(rgb, vec3(2.2));
}
`,
    wgsl: `${ACES.wgsl}
fn toneMap(color: vec3f) -> vec3f {
	var rgb = clamp(pow(gameAcesFitted(color * getExposure()), vec3f(1.0 / 2.2)), vec3f(0.0), vec3f(1.0));
	let highContrast = rgb * rgb * (3.0 - 2.0 * rgb);
	rgb = ${contrast < 1 ? `mix(vec3f(0.5), rgb, ${k})` : `mix(rgb, highContrast, ${k} - 1.0)`};
	return pow(rgb, vec3f(2.2));
}
`,
  };
}
