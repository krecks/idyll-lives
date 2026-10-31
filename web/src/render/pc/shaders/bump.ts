/**
 * Normal maps at the strength the game's textures were tuned with (up to 0.3.2), in the game's lit
 * materials (`GAME_LOOK`): `normalMapPS` with its strength line replaced.
 *
 * PlayCanvas mixes the map's normal towards flat by `bumpiness` (`mix((0, 0, 1), n, b)`, which also
 * lifts z by `1 − b`); the game's materials scale its x and y by the bump level and leave z
 * (`normalize(n · (b, b, 1))`), as the textures were tuned. At the lawn's 0.8 PlayCanvas's bumps
 * came out about 6 % flatter, and with them the highlights tighter at noon and the sheen fainter
 * towards a low sun.
 */

import type { Language } from './chunks';

/** The line replaced (also in `tools/render/check_chunks.ts`). */
export const BUMP_ANCHORS = {
  glsl: 'normalMap = mix(vec3(0.0, 0.0, 1.0), normalMap, material_bumpiness);',
  wgsl: 'normalMap = mix(vec3f(0.0, 0.0, 1.0), normalMap, uniform.material_bumpiness);',
} as const;

const CODE: Record<Language, string> = {
  glsl: `
	#ifdef GAME_LOOK
		normalMap = normalize(normalMap * vec3(material_bumpiness, material_bumpiness, 1.0));
	#else
		${BUMP_ANCHORS.glsl}
	#endif`,
  wgsl: `
	#ifdef GAME_LOOK
		normalMap = normalize(normalMap * vec3f(uniform.material_bumpiness, uniform.material_bumpiness, 1.0));
	#else
		${BUMP_ANCHORS.wgsl}
	#endif`,
};

/** The game's `normalMapPS` from the engine's (`engine`, in `lang`). */
export function gameNormalMap(engine: string, lang: Language, replaceOnce: (code: string, name: string, anchor: string, replacement: string) => string): string {
  return replaceOnce(engine, 'normalMapPS', BUMP_ANCHORS[lang], CODE[lang]);
}
