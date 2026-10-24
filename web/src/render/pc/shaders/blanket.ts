/**
 * Bed blankets on PlayCanvas (the port of `BlanketPlugin`, `babylon/characters/blanket.ts`): one quilt
 * mesh per body type (`shared/characters/blanket.ts`), drawn instanced, a copy per resident asleep
 * in a bed, in that bed's colours. Material: `blanketMaterial` (`characterMaterial.ts`).
 *
 * `GAME_BLANKET`, per instance beside the matrix:
 * - `game_blanketExt` (m, local; `BLANKET_EXT`): x moves the quilt past the toes towards the bed's
 *   foot end (stretching it between `game_blanketRange` x, the toes, and y, its foot edge), so one
 *   mesh fits beds of any length while the part over the body keeps its shape; y is how far up it is
 *   pulled (local z): from there to its foot end it keeps its shape, the rest folds down under that
 *   line, flat on the mattress.
 * - `game_blanketTint` (linear rgb; `BLANKET_TINT`, `quiltTint`): its bed's bedding colour, over the
 *   cover, not over the turned-down sheet band (v, metres from the top edge, under `SHEET`).
 *
 * The fold and the stretch apply in the shadow pass too (the quilt's shadow is its shape's).
 */

import { SHEET } from '../../shared/characters/blanket';
import type { ShaderFeature } from './chunks';

const sheet = SHEET.toFixed(3);

export const BLANKET: ShaderFeature = {
  name: 'blanket',
  priority: 240,
  glsl: {
    vsDecl: `
#ifdef GAME_BLANKET
	attribute vec2 game_blanketExt;
	attribute vec3 game_blanketTint;
	uniform vec2 game_blanketRange;
	varying vec4 vGameBlanket;
#endif`,
    vsLocal: `
#ifdef GAME_BLANKET
	if (localPos.z < game_blanketExt.y) localPos = vec3(localPos.x, 0.0, game_blanketExt.y);
	localPos.z += game_blanketExt.x * smoothstep(game_blanketRange.x, game_blanketRange.y, localPos.z);
#endif`,
    vsEnd: `
#ifdef GAME_BLANKET
	#ifdef UV0
		vGameBlanket = vec4(game_blanketTint, vertex_texCoord0.y);
	#else
		vGameBlanket = vec4(game_blanketTint, 1.0);
	#endif
#endif`,
    psDecl: `
#if defined(GAME_BLANKET) && !defined(SHADOW_PASS)
	varying vec4 vGameBlanket;
#endif`,
    albedo: `
#ifdef GAME_BLANKET
	if (vGameBlanket.w >= ${sheet}) dAlbedo *= vGameBlanket.rgb;
#endif`,
  },
  wgsl: {
    vsDecl: `
#ifdef GAME_BLANKET
	attribute game_blanketExt: vec2f;
	attribute game_blanketTint: vec3f;
	uniform game_blanketRange: vec2f;
	varying vGameBlanket: vec4f;
#endif`,
    vsLocal: `
#ifdef GAME_BLANKET
	if (localPos.z < game_blanketExt.y) { localPos = vec3f(localPos.x, 0.0, game_blanketExt.y); }
	localPos.z = localPos.z + game_blanketExt.x * smoothstep(uniform.game_blanketRange.x, uniform.game_blanketRange.y, localPos.z);
#endif`,
    vsEnd: `
#ifdef GAME_BLANKET
	#ifdef UV0
		output.vGameBlanket = vec4f(game_blanketTint, vertex_texCoord0.y);
	#else
		output.vGameBlanket = vec4f(game_blanketTint, 1.0);
	#endif
#endif`,
    psDecl: `
#if defined(GAME_BLANKET) && !defined(SHADOW_PASS)
	varying vGameBlanket: vec4f;
#endif`,
    albedo: `
#ifdef GAME_BLANKET
	dAlbedo = select(dAlbedo, dAlbedo * vGameBlanket.rgb, vGameBlanket.w >= ${sheet});
#endif`,
  },
};
