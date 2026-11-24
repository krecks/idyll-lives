/**
 * The farmland's colour map on the ground (`groundTiles.ts` `FieldMap`): a texture over the box a big world's fields
 * reach, each field's crop in its rows inside its grass margin and its hedges, laid over the ground tiles' own colour
 * (their points are 2 m apart: a field's edge or its rows can't be drawn by them). Its alpha says where a field is
 * (the rest is the land's own colour, the margins and pastures too); its colours premultiplied by it and sRGB.
 *
 * `GAME_FIELD_MAP` on the ground's material (`fieldGround`); read in the town view too, under its map
 * (`GAME_TOWN_GROUND`, which has no fields in it: they don't fade out and back in with its blend). With both, the
 * ground's program reads 16 textures there, all WebGL2 has (`samplerCheck.ts` says so in dev): none more on it.
 */

import { registerShaderFeature, type Language, type ShaderFeature } from './chunks';

function code(lang: Language): ShaderFeature['glsl'] {
  const wgsl = lang === 'wgsl';
  const psDecl = wgsl
    ? `
#if defined(GAME_FIELD_MAP) && !defined(SHADOW_PASS)
	var game_fieldMap: texture_2d<f32>;
	var game_fieldMapSampler: sampler;
	uniform game_fieldArea: vec4f;
#endif`
    : `
#if defined(GAME_FIELD_MAP) && !defined(SHADOW_PASS)
	uniform sampler2D game_fieldMap;
	uniform vec4 game_fieldArea;
#endif`;
  const albedo = wgsl
    ? `
#if defined(GAME_FIELD_MAP) && !defined(SHADOW_PASS)
	{
		let fmQ = gamePos(vPositionW);
		let fmUv = (fmQ.xz - uniform.game_fieldArea.xy) * uniform.game_fieldArea.zw;
		let fmM = textureSample(game_fieldMap, game_fieldMapSampler, fmUv);
		let fmIn = step(0.0, fmUv.x) * step(0.0, fmUv.y) * step(fmUv.x, 1.0) * step(fmUv.y, 1.0);
		let fmC = pow(fmM.rgb / max(fmM.a, 0.001), vec3f(2.2));
		dAlbedo = mix(dAlbedo, fmC, clamp(fmM.a, 0.0, 1.0) * fmIn);
	}
#endif`
    : `
#if defined(GAME_FIELD_MAP) && !defined(SHADOW_PASS)
	{
		vec3 fmQ = gamePos(vPositionW);
		vec2 fmUv = (fmQ.xz - game_fieldArea.xy) * game_fieldArea.zw;
		vec4 fmM = texture(game_fieldMap, fmUv);
		float fmIn = step(0.0, fmUv.x) * step(0.0, fmUv.y) * step(fmUv.x, 1.0) * step(fmUv.y, 1.0);
		vec3 fmC = pow(fmM.rgb / max(fmM.a, 0.001), vec3(2.2));
		dAlbedo = mix(dAlbedo, fmC, clamp(fmM.a, 0.0, 1.0) * fmIn);
	}
#endif`;
  return { psDecl, albedo };
}

/** After the grass's ground (235) and the town's buildings (236), before the town view's map (237). */
export const FIELD_GROUND: ShaderFeature = { name: 'fieldGround', priority: 236.5, glsl: code('glsl'), wgsl: code('wgsl') };

registerShaderFeature(FIELD_GROUND);
