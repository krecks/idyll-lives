/**
 * Moonlight grade (the port of `babylon/nightGrade.ts`): at night, moonlit tones lose their
 * colour and turn blue-grey while warm, lamp-lit pixels (windows, interiors, streetlight pools:
 * red above blue) and very bright ones keep their colour, so they pop.
 *
 * Why a grade: lawns and leaves reflect almost no blue, so a blue moon and sky fill alone render a
 * night as dark olive green, and the colour curves can only tint multiplicatively (a weak filter
 * that cannot create blue).
 *
 * Babylon ran it as a pass of its own after the image processing (and the FXAA), attached only
 * while it was getting dark. Here it is a function at the end of the last full-screen pass the
 * picture already takes (`post.ts`: the compose, or the FXAA on Low), on the image-processed colour
 * clamped to 0..1 as Babylon's 8-bit pass input had it. By day its amount is 0, which leaves the
 * colour exactly as it is: no pass to attach, no shader to change.
 */

import type { Language } from './shaders/chunks';

/** Multiplier for the desaturated dark tones (display RGB). */
export const NIGHT_TINT: readonly [number, number, number] = [0.8, 0.94, 1.45];

/** Below this amount Babylon detached the pass: the grade is off. */
export const NIGHT_OFF = 0.001;

/** The grade's uniforms: `gameNightAmount` (0..1) and `gameNightTint`. */
export const NIGHT_GRADE: Record<Language, string> = {
  glsl: `
uniform float gameNightAmount;
uniform vec3 gameNightTint;
vec3 gameNightGrade(vec3 c) {
	float l = dot(c, vec3(0.299, 0.587, 0.114));
	float hi = max(c.r, max(c.g, c.b));
	float keep = max(smoothstep(0.02, 0.14, c.r - c.b) * smoothstep(0.08, 0.3, hi), smoothstep(0.62, 0.9, hi));
	vec3 moon = mix(vec3(l), c, 0.18) * gameNightTint;
	return mix(c, mix(moon, c, keep), gameNightAmount);
}
`,
  wgsl: `
uniform gameNightAmount: f32;
uniform gameNightTint: vec3f;
fn gameNightGrade(c: vec3f) -> vec3f {
	let l = dot(c, vec3f(0.299, 0.587, 0.114));
	let hi = max(c.r, max(c.g, c.b));
	let keep = max(smoothstep(0.02, 0.14, c.r - c.b) * smoothstep(0.08, 0.3, hi), smoothstep(0.62, 0.9, hi));
	let moon = mix(vec3f(l), c, 0.18) * uniform.gameNightTint;
	return mix(c, mix(moon, c, keep), uniform.gameNightAmount);
}
`,
};

/** The grade's amount as Babylon applied it: none below `NIGHT_OFF` (its pass detached). */
export function nightAmount(amount: number): number {
  return amount > NIGHT_OFF ? amount : 0;
}
