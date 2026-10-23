/**
 * Lamps light their own room, and the game's lamps fall off as Babylon's did (the port of
 * `babylon/roomLights.ts`, M-lighting-08, and of Babylon's physical light falloff).
 *
 * Room lights. The lamps are clustered lights without shadows, so a lamp beside a wall would light
 * the room behind it. Every lit fragment knows its room, and every lamp its own, from a mask of the
 * rooms of the lot in view (`RoomLightState` in `materials.ts`: a texel per tile, on every storey
 * shown); in the clustered lighting loop a lamp in another room than the fragment is skipped (an
 * engine patch after `decodeClusterLightCore(lightIndex)` in `clusteredLightPS`, `chunks.ts`). A
 * fragment is classed by a point a little out along its normal, so a wall's two faces belong to
 * the rooms they face (and its outside face to the garden, which lamps indoors don't light). Tiles
 * the mask doesn't know (off the lot in view, split by a diagonal wall, storeys not shown) are lit by
 * every lamp. The fragment's own room is read once (`gameRoomHere`), a lamp's at each lamp.
 *
 * Meshes kept out of the lamps (`MaterialLibrary.excludeFromLights`: the landscape, the street) don't
 * run the clustered loop at all (the same patch's early return): the lamps never light them, and the
 * loop would only read the cluster for nothing.
 *
 * Falloff (per-material chunk overrides, `falloffInvSquaredPS` and `spotPS`): Babylon's PBR lit a
 * lamp's surface with `1/d²` (its physical falloff, the clusters' only one) and a shade's cone with a
 * spherical Gaussian (`computeDirectionalLightFalloff_Physical`: full along it, about a quarter 45°
 * off and next to none side-on, for a half-angle of 90°); its forward lights (four, on the low
 * preset) took the glTF falloff, the cone `saturate((cos - cos outer) / (cos inner - cos outer))²`.
 * PlayCanvas's own are `16/(d² + 1)` and a smoothstep across the cone. The game's materials
 * (`GAME_LOOK`) take Babylon's: the distance with PlayCanvas's window to zero at the light's range
 * (`getFalloffWindow`, as the glTF falloff had it; the many lamps' ranges are set past their reach
 * for it, `lamps.ts`), the physical cone for a light without an inner cone (its inner angle its
 * outer one) and the glTF cone for one with.
 */

import type { Language } from './chunks';

/** How far out along its normal a fragment is classed (m): off a wall's face, into the room it faces. */
const OUT = '0.1';

/** `-log2(1/100)`: Babylon's spot falloff, a hundredth of the light at the cone's edge. */
const CONE_EDGE = '6.64385618977';

export const ROOM_LIGHT_CODE: Record<Language, { decl: string; skip: string; noLamps: string; falloff: string; spot: string }> = {
  glsl: {
    // At the top of `clusteredLightPS`.
    decl: `
#ifdef GAME_LOOK
	uniform vec4 game_roomLightArea;
	uniform vec4 game_roomLightBand;
	uniform highp sampler2D game_roomLightMask;
	float gameRoomHere = -1.0;
	// The room at a point (PlayCanvas space) as the mask has it: 0 unknown, 1 outdoors, 2 and up a room.
	float gameRoomOf(vec3 at) {
		if (game_roomLightBand.w < 0.5) return 0.0;
		vec3 p = gamePos(at);
		float k = floor(p.y / game_roomLightBand.z) - game_roomLightBand.x;
		vec2 t = floor(p.xz - game_roomLightArea.xy);
		if (k < 0.0 || k >= game_roomLightBand.y || t.x < 0.0 || t.y < 0.0 || t.x >= game_roomLightArea.z || t.y >= game_roomLightArea.w) return 0.0;
		vec4 v = texelFetch(game_roomLightMask, ivec2(int(t.x), int(k * game_roomLightArea.w + t.y)), 0);
		return floor(v.r * 255.0 + 0.5) + floor(v.g * 255.0 + 0.5) * 256.0;
	}
	// Whether a lamp at lightPos is in another room than the fragment (normal n).
	bool gameRoomSkip(vec3 lightPos, vec3 n) {
		if (gameRoomHere < 0.0) gameRoomHere = gameRoomOf(vPositionW + n * ${OUT});
		float there = gameRoomOf(lightPos);
		return gameRoomHere > 0.0 && there > 0.0 && gameRoomHere != there;
	}
#endif
`,
    // After `decodeClusterLightCore(lightIndex)` in `evaluateClusterLight`.
    skip: `
	#ifdef GAME_LOOK
		if (gameRoomSkip(clusterLightData.position, worldNormal)) return;
	#endif`,
    // At the start of `addClusteredLights`.
    noLamps: `
	#if defined(GAME_LOOK) && !defined(CLUSTER_MESH_DYNAMIC_LIGHTS)
		if (numClusteredLights >= 0) return;
	#endif
`,
    falloff: `
float getFalloffInvSquared(float lightRadius, vec3 lightDir) {
	#ifdef GAME_LOOK
		return getFalloffWindow(lightRadius, lightDir) / max(dot(lightDir, lightDir), 1e-4);
	#else
		return getFalloffInvSquaredEngine(lightRadius, lightDir);
	#endif
}
`,
    spot: `
float getSpotEffect(vec3 lightSpotDir, float lightInnerConeAngle, float lightOuterConeAngle, vec3 lightDirNorm) {
	#ifdef GAME_LOOK
		float cosAngle = dot(lightDirNorm, lightSpotDir);
		if (lightInnerConeAngle <= lightOuterConeAngle + 1e-4) return exp2(${CONE_EDGE} / max(1.0 - lightOuterConeAngle, 1e-4) * (cosAngle - 1.0));
		float cone = saturate((cosAngle - lightOuterConeAngle) / (lightInnerConeAngle - lightOuterConeAngle));
		return cone * cone;
	#else
		return getSpotEffectEngine(lightSpotDir, lightInnerConeAngle, lightOuterConeAngle, lightDirNorm);
	#endif
}
`,
  },
  wgsl: {
    decl: `
#ifdef GAME_LOOK
	uniform game_roomLightArea: vec4f;
	uniform game_roomLightBand: vec4f;
	var game_roomLightMask: texture_2d<f32>;
	var<private> gameRoomHere: f32 = -1.0;
	fn gameRoomOf(at: vec3f) -> f32 {
		if (uniform.game_roomLightBand.w < 0.5) { return 0.0; }
		let p = gamePos(at);
		let k = floor(p.y / uniform.game_roomLightBand.z) - uniform.game_roomLightBand.x;
		let t = floor(p.xz - uniform.game_roomLightArea.xy);
		if (k < 0.0 || k >= uniform.game_roomLightBand.y || t.x < 0.0 || t.y < 0.0 || t.x >= uniform.game_roomLightArea.z || t.y >= uniform.game_roomLightArea.w) { return 0.0; }
		let v = textureLoad(game_roomLightMask, vec2<i32>(i32(t.x), i32(k * uniform.game_roomLightArea.w + t.y)), 0);
		return floor(v.r * 255.0 + 0.5) + floor(v.g * 255.0 + 0.5) * 256.0;
	}
	fn gameRoomSkip(lightPos: vec3f, n: vec3f) -> bool {
		if (gameRoomHere < 0.0) { gameRoomHere = gameRoomOf(vPositionW + n * ${OUT}); }
		let there = gameRoomOf(lightPos);
		return gameRoomHere > 0.0 && there > 0.0 && gameRoomHere != there;
	}
#endif
`,
    skip: `
	#ifdef GAME_LOOK
		if (gameRoomSkip(clusterLightData.position, worldNormal)) { return; }
	#endif`,
    noLamps: `
	#if defined(GAME_LOOK) && !defined(CLUSTER_MESH_DYNAMIC_LIGHTS)
		if (uniform.numClusteredLights >= 0) { return; }
	#endif
`,
    falloff: `
fn getFalloffInvSquared(lightRadius: f32, lightDir: vec3f) -> f32 {
	#ifdef GAME_LOOK
		return getFalloffWindow(lightRadius, lightDir) / max(dot(lightDir, lightDir), 1e-4);
	#else
		return getFalloffInvSquaredEngine(lightRadius, lightDir);
	#endif
}
`,
    spot: `
fn getSpotEffect(lightSpotDir: vec3f, lightInnerConeAngle: f32, lightOuterConeAngle: f32, lightDirNorm: vec3f) -> f32 {
	#ifdef GAME_LOOK
		let cosAngle = dot(lightDirNorm, lightSpotDir);
		if (lightInnerConeAngle <= lightOuterConeAngle + 1e-4) { return exp2(${CONE_EDGE} / max(1.0 - lightOuterConeAngle, 1e-4) * (cosAngle - 1.0)); }
		let cone = saturate((cosAngle - lightOuterConeAngle) / (lightInnerConeAngle - lightOuterConeAngle));
		return cone * cone;
	#else
		return getSpotEffectEngine(lightSpotDir, lightInnerConeAngle, lightOuterConeAngle, lightDirNorm);
	#endif
}
`,
  },
};
