/**
 * The ground's colour map at altitude (`groundTiles.ts` `TownGroundMap`, docs/design/living-town.md):
 * a texture over the whole world, baked once per town, of what the town lays on its land (the streets
 * and their pavements, the junctions' pads, the centre's paving, the water, the farm fields), laid over
 * the ground tiles' own colour as the town view comes in (`game_townGround`, its blend). Its alpha says
 * where it has something (the rest is the land's own colour); its colours are premultiplied by it (so
 * its mips stay true at an edge) and sRGB. The streets streamed in draw over it as ever: it is what
 * stands in for them past the chunks streamed in, where the street draws no road.
 *
 * `GAME_TOWN_GROUND` on the ground tiles' material only while the town view is in (`groundTiles.ts`):
 * the play camera draws its ground with the shader it always had.
 */

import { registerShaderFeature, type Language, type ShaderFeature } from './chunks';

function code(lang: Language): ShaderFeature['glsl'] {
  const wgsl = lang === 'wgsl';
  const psDecl = wgsl
    ? `
#ifdef GAME_TOWN_GROUND
	var game_townMap: texture_2d<f32>;
	var game_townMapSampler: sampler;
	uniform game_townMapArea: vec4f;
	uniform game_townGround: f32;
#endif`
    : `
#ifdef GAME_TOWN_GROUND
	uniform sampler2D game_townMap;
	uniform vec4 game_townMapArea;
	uniform float game_townGround;
#endif`;
  const albedo = wgsl
    ? `
#ifdef GAME_TOWN_GROUND
	{
		let tgQ = gamePos(vPositionW);
		let tgUv = (tgQ.xz - uniform.game_townMapArea.xy) * uniform.game_townMapArea.zw;
		let tgM = textureSample(game_townMap, game_townMapSampler, tgUv);
		let tgIn = step(0.0, tgUv.x) * step(0.0, tgUv.y) * step(tgUv.x, 1.0) * step(tgUv.y, 1.0);
		let tgC = pow(tgM.rgb / max(tgM.a, 0.001), vec3f(2.2));
		dAlbedo = mix(dAlbedo, tgC, clamp(tgM.a, 0.0, 1.0) * tgIn * uniform.game_townGround);
	}
#endif`
    : `
#ifdef GAME_TOWN_GROUND
	{
		vec3 tgQ = gamePos(vPositionW);
		vec2 tgUv = (tgQ.xz - game_townMapArea.xy) * game_townMapArea.zw;
		vec4 tgM = texture(game_townMap, tgUv);
		float tgIn = step(0.0, tgUv.x) * step(0.0, tgUv.y) * step(tgUv.x, 1.0) * step(tgUv.y, 1.0);
		vec3 tgC = pow(tgM.rgb / max(tgM.a, 0.001), vec3(2.2));
		dAlbedo = mix(dAlbedo, tgC, clamp(tgM.a, 0.0, 1.0) * tgIn * game_townGround);
	}
#endif`;
  return { psDecl, albedo };
}

/** After the grass's ground (235) and the town's buildings (236). */
export const TOWN_GROUND: ShaderFeature = { name: 'townGround', priority: 237, glsl: code('glsl'), wgsl: code('wgsl') };

registerShaderFeature(TOWN_GROUND);
