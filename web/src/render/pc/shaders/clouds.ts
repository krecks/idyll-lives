/**
 * Cloud shadows, in shader code: the sun's light, and only the sun's, is dimmed where a cloud's shadow lies
 * on the land. An engine patch of `lightFunctionLightPS` (`chunks.ts`, beside the sun kept out of basements),
 * acting only in the game's materials (`GAME_LOOK`) and on the directional light: one read of a small map of
 * the clouds (`pc/cloudShadows.ts`, which makes it and drifts it) where the fragment stands.
 *
 * Uniforms, the device's (zero and blank until a renderer sets them, `patchEngineChunks`): `game_cloudMap`
 * (how much of a cloud is over a point, tiling) and `game_cloud` (x, y: the map's offset, as the clouds have
 * drifted; z: 1 / the metres the map spans; w: the share of the sun a cloud's middle takes, 0 at night).
 */

import type { Language } from './chunks';

export const CLOUD_CODE: Record<Language, { decl: string; sun: string }> = {
  glsl: {
    decl: `
#ifndef GAME_CLOUD_DECL
#define GAME_CLOUD_DECL
	uniform sampler2D game_cloudMap;
	uniform vec4 game_cloud;
#endif
`,
    sun: `
	#ifdef GAME_LOOK
		#if LIGHT{i}TYPE == DIRECTIONAL
			lightColor *= 1.0 - game_cloud.w * texture2DLod(game_cloudMap, vPositionW.xz * game_cloud.z + game_cloud.xy, 0.0).r;
		#endif
	#endif`,
  },
  wgsl: {
    decl: `
#ifndef GAME_CLOUD_DECL
#define GAME_CLOUD_DECL
	var game_cloudMap: texture_2d<f32>;
	var game_cloudMapSampler: sampler;
	uniform game_cloud: vec4f;
#endif
`,
    sun: `
	#ifdef GAME_LOOK
		#if LIGHT{i}TYPE == DIRECTIONAL
			lightColor = lightColor * (1.0 - uniform.game_cloud.w * textureSampleLevel(game_cloudMap, game_cloudMapSampler, vPositionW.xz * uniform.game_cloud.z + uniform.game_cloud.xy, 0.0).r);
		#endif
	#endif`,
  },
};
