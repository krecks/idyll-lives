/**
 * How much of the sky a surface reflects, as Babylon's PBR had it, in the game's lit materials
 * (`GAME_LOOK`): `reflectionEnvPS` (and its HQ twin) with `addReflection` wrapped.
 *
 * PlayCanvas 2.23 scales the environment's reflection by the surface's reflectance at normal
 * incidence alone (`specularity`, 0.04 for a dielectric): its Schlick term in `getFresnel` never
 * rises for a surface with gloss below 1 (`max(gloss² · F0, F0)` is F0). Babylon's environment BRDF
 * rises towards grazing angles, rough surfaces too: the lawn far off, seen at a slant, reflected
 * the bright sky above it (about +0.02 of light in linear at the top of the noon orbit shot). Here
 * the reflection added is scaled to Lagarde's roughness-aware Schlick term
 * (`F0 + (max(gloss, F0) − F0) · (1 − n·v)⁵`) over F0, so after PlayCanvas multiplies it by F0 it is
 * that term: F0 face on (as the lawn at your feet reflects in Babylon's pictures; Karis's fit of the
 * split sum falls under F0 there, and darkened it), up to the gloss at a slant.
 */

import type { Language } from './chunks';

/** The declaration wrapped (also in `tools/render/check_chunks.ts`). */
export const ENV_BRDF_ANCHORS = {
  glsl: 'void addReflection(vec3 reflDir, float gloss) {',
  wgsl: 'fn addReflection(reflDir: vec3f, gloss: f32) {',
} as const;

const CODE: Record<Language, string> = {
  glsl: `
void addReflection(vec3 reflDir, float gloss) {
	#ifdef GAME_LOOK
		vec3 gameBefore = dReflection.rgb;
		addReflectionEngine(reflDir, gloss);
		vec3 gameF0 = max(litArgs_specularity, vec3(0.001));
		float gameSlant = pow(1.0 - saturate(dot(dViewDirW, litArgs_worldNormal)), 5.0);
		vec3 gameScale = 1.0 + (max(vec3(gloss), gameF0) - gameF0) * gameSlant / gameF0;
		dReflection.rgb = gameBefore + (dReflection.rgb - gameBefore) * gameScale;
	#else
		addReflectionEngine(reflDir, gloss);
	#endif
}
`,
  wgsl: `
fn addReflection(reflDir: vec3f, gloss: f32) {
	#ifdef GAME_LOOK
		let gameBefore = dReflection.rgb;
		addReflectionEngine(reflDir, gloss);
		let gameF0 = max(litArgs_specularity, vec3f(0.001));
		let gameSlant = pow(1.0 - saturate(dot(dViewDirW, litArgs_worldNormal)), 5.0);
		let gameScale = 1.0 + (max(vec3f(gloss), gameF0) - gameF0) * gameSlant / gameF0;
		dReflection = vec4f(gameBefore + (dReflection.rgb - gameBefore) * gameScale, dReflection.a);
	#else
		addReflectionEngine(reflDir, gloss);
	#endif
}
`,
};

/** The game's `reflectionEnvPS` / `reflectionEnvHQPS` from the engine's (`engine`, in `lang`). */
export function gameReflection(engine: string, name: string, lang: Language, replaceOnce: (code: string, name: string, anchor: string, replacement: string) => string): string {
  const anchor = ENV_BRDF_ANCHORS[lang];
  return `${replaceOnce(engine, name, anchor, anchor.replace('addReflection(', 'addReflectionEngine('))}\n${CODE[lang]}`;
}
