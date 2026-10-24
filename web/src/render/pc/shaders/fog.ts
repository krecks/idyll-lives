// Follows Babylon.js 9.30's PBR fog (`fogFragment.fx`: the eye's distance, the factor in linear space).
// Changed for Idyll Lives: as two lines of PlayCanvas's `fogPS`.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
/**
 * The haze as Babylon's PBR drew it, in the game's lit materials (`GAME_LOOK`): `fogPS` with two
 * lines changed (anchors also in `tools/render/check_chunks.ts`).
 *
 * - The distance is the eye's (`length` of the view-space position, Babylon's `vFogDistance`), not
 *   the depth along the view (PlayCanvas's `gl_FragCoord.z / w`): the haze is as thick at the
 *   picture's sides as in its middle.
 * - The fog factor is taken into linear space (`pow(f, 2.2)`: Babylon's `fogFragment` does
 *   `fog = toLinearSpace(fog)` for PBR materials): the haze is about 2.2 times as strong near.
 *
 * Other materials (unlit decals, particles) keep PlayCanvas's own.
 */

import type { Language } from './chunks';

/** The lines of `fogPS` replaced: the fragment's distance, and the exponential-squared factor. */
export const FOG_ANCHORS = {
  glsl: { depth: 'float depth = gl_FragCoord.z / gl_FragCoord.w;', exp2: 'fogFactor = exp(-depth * depth * fog_density * fog_density);' },
  wgsl: { depth: 'let depth = pcPosition.z / pcPosition.w;', exp2: 'return evaluateFogFactorExp2(depth, uniform.fog_density);' },
} as const;

const FOG_CODE: Record<Language, { depth: string; exp2: string }> = {
  glsl: {
    depth: `
		#ifdef GAME_LOOK
			float depth = distance(vPositionW, view_position);
		#else
			float depth = gl_FragCoord.z / gl_FragCoord.w;
		#endif`,
    exp2: `
		fogFactor = exp(-depth * depth * fog_density * fog_density);
		#ifdef GAME_LOOK
			fogFactor = pow(fogFactor, 2.2);
		#endif`,
  },
  wgsl: {
    depth: `
		#ifdef GAME_LOOK
			let depth = distance(vPositionW, uniform.view_position);
		#else
			let depth = pcPosition.z / pcPosition.w;
		#endif`,
    exp2: `
		#ifdef GAME_LOOK
			return pow(evaluateFogFactorExp2(depth, uniform.fog_density), 2.2);
		#else
			return evaluateFogFactorExp2(depth, uniform.fog_density);
		#endif`,
  },
};

/** The game's `fogPS` from the engine's (`engine`, in `lang`). */
export function gameFog(engine: string, lang: Language, replaceOnce: (code: string, name: string, anchor: string, replacement: string) => string): string {
  const a = FOG_ANCHORS[lang];
  const c = FOG_CODE[lang];
  return replaceOnce(replaceOnce(engine, 'fogPS', a.depth, c.depth), 'fogPS', a.exp2, c.exp2);
}
