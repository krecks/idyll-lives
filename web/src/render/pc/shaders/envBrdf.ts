// Follows Babylon.js 9.30's PBR Fresnel for lights (Schlick, F90 = 1). Changed for Idyll Lives: in
// PlayCanvas's `fresnelSchlickPS` and `reflectionEnvPS`, the sky's reflection with Lagarde's term.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
/**
 * Fresnel as Babylon's PBR had it, in the game's lit materials (`GAME_LOOK`): `fresnelSchlickPS`
 * with its term replaced, and `reflectionEnvPS` (and its HQ twin) with `addReflection` wrapped.
 *
 * PlayCanvas 2.23's Schlick term never rises for a surface with gloss below 1 (`getFresnel`:
 * `max(gloss² · F0, F0)` is F0), so every highlight and reflection stays at the reflectance face on
 * (`specularity`, 0.04 for a dielectric).
 *
 * - The lights (`getFresnel` of v·h, the sun's and the lamps'): Babylon's Schlick, `F0 + (1 − F0) ·
 *   (1 − v·h)⁵`, whatever the roughness. Looking towards a low sun, v·h is small and the rough
 *   ground's highlight rises to several times F0: the golden hour's sheen on the lawn and the
 *   hillsides (without it the menus' golden-hour shot was about a tenth dark, and its sun shadows
 *   faint against the unlit ground).
 * - The sky's reflection (`getFresnel` of n·v, after `addReflection`): Lagarde's roughness-aware
 *   Schlick, `F0 + (max(gloss, F0) − F0) · (1 − n·v)⁵` (Babylon's environment BRDF also rises at a
 *   slant, rough surfaces too, but stays near F0 face on for them: Karis's fit of the split sum
 *   falls under F0 there and darkened the lawn at your feet). The reflection added is scaled by
 *   that over the Schlick term, so after PlayCanvas multiplies it by `getFresnel` it is Lagarde's.
 */

import type { Language } from './chunks';

/** The lines replaced and wrapped (also in `tools/render/check_chunks.ts`). */
export const ENV_BRDF_ANCHORS = {
  glsl: {
    reflection: 'void addReflection(vec3 reflDir, float gloss) {',
    fresnel: 'vec3 ret = specularity + (max(vec3(glossSq * specIntensity), specularity) - specularity) * fresnel;',
  },
  wgsl: {
    reflection: 'fn addReflection(reflDir: vec3f, gloss: f32) {',
    fresnel: 'let ret: vec3f = specularity + (max(vec3f(glossSq * specIntensity), specularity) - specularity) * fresnel;',
  },
} as const;

const FRESNEL: Record<Language, string> = {
  glsl: `
	#ifdef GAME_LOOK
		vec3 ret = specularity + (vec3(1.0) - specularity) * fresnel;
	#else
		${ENV_BRDF_ANCHORS.glsl.fresnel}
	#endif`,
  wgsl: `
	#ifdef GAME_LOOK
		let ret: vec3f = specularity + (vec3f(1.0) - specularity) * fresnel;
	#else
		${ENV_BRDF_ANCHORS.wgsl.fresnel}
	#endif`,
};

const REFLECTION: Record<Language, string> = {
  glsl: `
void addReflection(vec3 reflDir, float gloss) {
	#ifdef GAME_LOOK
		vec3 gameBefore = dReflection.rgb;
		addReflectionEngine(reflDir, gloss);
		vec3 gameF0 = max(litArgs_specularity, vec3(0.001));
		float gameSlant = pow(1.0 - saturate(dot(dViewDirW, litArgs_worldNormal)), 5.0);
		vec3 gameLagarde = gameF0 + (max(vec3(gloss), gameF0) - gameF0) * gameSlant;
		vec3 gameSchlick = gameF0 + (vec3(1.0) - gameF0) * gameSlant;
		dReflection.rgb = gameBefore + (dReflection.rgb - gameBefore) * gameLagarde / gameSchlick;
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
		let gameLagarde = gameF0 + (max(vec3f(gloss), gameF0) - gameF0) * gameSlant;
		let gameSchlick = gameF0 + (vec3f(1.0) - gameF0) * gameSlant;
		dReflection = vec4f(gameBefore + (dReflection.rgb - gameBefore) * gameLagarde / gameSchlick, dReflection.a);
	#else
		addReflectionEngine(reflDir, gloss);
	#endif
}
`,
};

type Replace = (code: string, name: string, anchor: string, replacement: string) => string;

/** The game's `reflectionEnvPS` / `reflectionEnvHQPS` from the engine's (`engine`, in `lang`). */
export function gameReflection(engine: string, name: string, lang: Language, replaceOnce: Replace): string {
  const anchor = ENV_BRDF_ANCHORS[lang].reflection;
  return `${replaceOnce(engine, name, anchor, anchor.replace('addReflection(', 'addReflectionEngine('))}\n${REFLECTION[lang]}`;
}

/** The game's `fresnelSchlickPS` from the engine's. */
export function gameFresnel(engine: string, lang: Language, replaceOnce: Replace): string {
  return replaceOnce(engine, 'fresnelSchlickPS', ENV_BRDF_ANCHORS[lang].fresnel, FRESNEL[lang]);
}
