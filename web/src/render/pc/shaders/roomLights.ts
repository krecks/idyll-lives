// The falloff follows Babylon.js 9.30's PBR lights (`computeDistanceLightFalloff_Physical`,
// `computeDirectionalLightFalloff_Physical`, the glTF cone). Changed for Idyll Lives: as PlayCanvas
// chunk overrides with its range window, in GLSL and WGSL.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
/**
 * Lamps light their own room (M-lighting-08), and the game's lamps fall off as they were tuned to,
 * physically.
 *
 * Room lights. The lamps are lights without shadows, so a lamp beside a wall would light the room
 * behind it. Every lit fragment knows its room, and every lamp its own, from a mask of the rooms of
 * the lot in view (`RoomLightState` in `materials.ts`: a texel per tile, on every storey shown; a
 * tile split by a diagonal wall has a room per half); a lamp in another room than the fragment is
 * skipped, on every preset: in the clustered lighting loop (an engine patch after
 * `decodeClusterLightCore(lightIndex)` in `clusteredLightPS`, `chunks.ts`) and, for forward lights
 * (`?lights=forward`), in each light's own function (after `lightColor` in `lightFunctionLightPS`).
 * The test itself is a feature of the game's materials (`ROOM_LIGHTS`, declared once for both). A
 * fragment is classed by a point a little out along its normal, so a wall's two faces belong to
 * the rooms they face (and its outside face to the garden, which lamps indoors don't light); what is inside an
 * outside wall (a window's opening past its glass, a cut wall's top) has the wall's room's own light, its share of
 * the sky and its lamps' bounce (`gameRoomLit`; the lamps and the reflections keep the mask's own answer,
 * `gameRoomHere`). Tiles
 * the mask doesn't know (off the lot in view, storeys not shown) are lit by every lamp. The
 * fragment's own room is read once (`gameRoomHere`), a lamp's at each lamp.
 *
 * Meshes kept out of the lamps (`MaterialLibrary.excludeFromLights`: the landscape, the street) don't
 * run the clustered loop at all (the same patch's early return): the lamps never light them, and the
 * loop would only read the cluster for nothing.
 *
 * Falloff (per-material chunk overrides, `falloffInvSquaredPS` and `spotPS`): the lamps were tuned
 * with the physical falloff, `1/d²`, and a shade's cone as a spherical Gaussian (the original's
 * `computeDirectionalLightFalloff_Physical`: full along it, about a quarter 45° off and next to
 * none side-on, for a half-angle of 90°); the low preset's four lights with the glTF falloff, the
 * cone `saturate((cos - cos outer) / (cos inner - cos outer))²`. PlayCanvas's own are `16/(d² + 1)`
 * and a smoothstep across the cone. The game's materials (`GAME_LOOK`) take the tuned ones: the
 * distance with PlayCanvas's window to zero at the light's range (`getFalloffWindow`, as the glTF
 * falloff had it; the many lamps' ranges are set past their reach for it, `lamps.ts`), the physical
 * cone for a light without an inner cone (its inner angle its outer one) and the glTF cone for one
 * with.
 *
 * Shadows (`lampShadows.ts`, where the preset has them): a lamp that casts is a spot light with one shadow
 * map, read in the clustered loop; a lamp without a shade stays all round (`ALL_ROUND_COS`), and no lamp is
 * shadowed past its map's cone (`gameLampShadow`, an engine patch round the read).
 */

import type { Language, ShaderFeature } from './chunks';
import { WALL_HALF } from '../../shared/walls';

/** Rooms with a light of their own (`RoomLightState.light`, a texel each): the mask's first, far more than a lot has. */
export const ROOM_LIGHT_ROOMS = 256;

/** How far out along its normal a fragment is classed (m): off a wall's face, into the room it faces. */
export const OUT = '0.1';

/**
 * A spot light whose inner cone is wider than this (its cosine: 120°, far past any cone a shade has) lights all
 * round: a lamp without a shade drawn as a spot light for the one shadow map a spot has (`lamps.ts`
 * `ALL_ROUND_CONE`).
 */
const ALL_ROUND_COS = '-0.5';

/**
 * Half a wall's thickness (m, `shared/walls.ts` `WALL_HALF`): a point nearer than this to a tile's edge with a
 * room beyond it is inside that room's outside wall (`gameRoomLit`).
 */
const IN_WALL = String(WALL_HALF);

/** `-log2(1/100)`: the physical spot falloff, a hundredth of the light at the cone's edge. */
const CONE_EDGE = '6.64385618977';

/**
 * The room test, in the game's materials' declarations (`litUserDeclarationPS`, before the lights'
 * code). A mask texel: r, g the tile's room (low, high byte; on a split tile its half 0's), b, a its
 * half 1's (a's low six bits the high ones), a's top bit set on a split tile and the next one for a
 * 'dn' wall. In tile-local u, v a 'dp' wall's half 1 is v > u, a 'dn' wall's u + v > 1 (as the
 * windowless mask has them, `underground.ts`).
 */
export const ROOM_LIGHTS: ShaderFeature = {
  name: 'roomLights',
  priority: 205,
  glsl: {
    psDecl: `
#ifdef GAME_LOOK
	uniform vec4 game_roomLightArea;
	uniform vec4 game_roomLightBand;
	uniform highp sampler2D game_roomLightMask;
	// A texel per room of the mask (\`RoomLightState.light\`): r its lamps' bounce, a the sky's share.
	uniform sampler2D game_roomLight;
	float gameRoomHere = -1.0;
	// The room at a point (PlayCanvas space) as the mask has it: 0 unknown, 1 outdoors, 2 and up a room.
	float gameRoomOf(vec3 at) {
		if (game_roomLightBand.w < 0.5) return 0.0;
		vec3 p = gamePos(at);
		float k = floor(p.y / game_roomLightBand.z) - game_roomLightBand.x;
		vec2 u = p.xz - game_roomLightArea.xy;
		vec2 t = floor(u);
		if (k < 0.0 || k >= game_roomLightBand.y || t.x < 0.0 || t.y < 0.0 || t.x >= game_roomLightArea.z || t.y >= game_roomLightArea.w) return 0.0;
		vec4 v = floor(texelFetch(game_roomLightMask, ivec2(int(t.x), int(k * game_roomLightArea.w + t.y)), 0) * 255.0 + 0.5);
		if (v.a >= 128.0) {
			vec2 f = u - t;
			if (mod(v.a, 128.0) >= 64.0 ? f.x + f.y > 1.0 : f.y > f.x) return v.b + mod(v.a, 64.0) * 256.0;
		}
		return v.r + v.g * 256.0;
	}
	// The room whose own light (\`RoomLightState.light\`: its share of the sky, its lamps' bounce) the fragment at pos
	// (normal n) has: its own (\`gameRoomHere\`), but for what is outdoors by the mask and inside an outside wall (the
	// sides of a window's opening past the glass, a door's edge, the outer half of a cut wall's top, the back of a
	// thing standing against the wall), which has the wall's room's: not the garden's, vivid blue beside a lamplit
	// room. The lamps themselves keep to the mask's rooms (\`gameRoomSkip\`): a seam in an outside wall's corner would
	// shine with the room's lamps.
	float gameRoomLit = -1.0;
	float gameRoomLitAt(vec3 pos, vec3 n) {
		if (gameRoomLit >= 0.0) return gameRoomLit;
		vec3 at = pos + n * ${OUT};
		if (gameRoomHere < 0.0) gameRoomHere = gameRoomOf(at);
		gameRoomLit = gameRoomHere;
		if (gameRoomHere != 1.0) return gameRoomLit;
		vec2 f = fract(gamePos(at).xz - game_roomLightArea.xy);
		// (-1 near a tile's low edge, 1 near its high one, in game x and z; PlayCanvas's z is the other way.)
		vec2 side = step(vec2(1.0 - ${IN_WALL}), f) - step(f, vec2(${IN_WALL}));
		float beyond = side.x != 0.0 ? gameRoomOf(at + vec3(side.x * 2.0 * ${IN_WALL}, 0.0, 0.0)) : 0.0;
		if (beyond < 1.5 && side.y != 0.0) beyond = gameRoomOf(at - vec3(0.0, 0.0, side.y * 2.0 * ${IN_WALL}));
		if (beyond > 1.5) gameRoomLit = beyond;
		return gameRoomLit;
	}
	// Whether a lamp at lightPos is in another room than the fragment at pos (normal n).
	bool gameRoomSkip(vec3 lightPos, vec3 pos, vec3 n) {
		if (gameRoomHere < 0.0) gameRoomHere = gameRoomOf(pos + n * ${OUT});
		float there = gameRoomOf(lightPos);
		return gameRoomHere > 0.0 && there > 0.0 && gameRoomHere != there;
	}
#endif
`,
  },
  wgsl: {
    psDecl: `
#ifdef GAME_LOOK
	uniform game_roomLightArea: vec4f;
	uniform game_roomLightBand: vec4f;
	var game_roomLightMask: texture_2d<f32>;
	var game_roomLight: texture_2d<f32>;
	var<private> gameRoomHere: f32 = -1.0;
	fn gameRoomOf(at: vec3f) -> f32 {
		if (uniform.game_roomLightBand.w < 0.5) { return 0.0; }
		let p = gamePos(at);
		let k = floor(p.y / uniform.game_roomLightBand.z) - uniform.game_roomLightBand.x;
		let u = p.xz - uniform.game_roomLightArea.xy;
		let t = floor(u);
		if (k < 0.0 || k >= uniform.game_roomLightBand.y || t.x < 0.0 || t.y < 0.0 || t.x >= uniform.game_roomLightArea.z || t.y >= uniform.game_roomLightArea.w) { return 0.0; }
		let v = floor(textureLoad(game_roomLightMask, vec2<i32>(i32(t.x), i32(k * uniform.game_roomLightArea.w + t.y)), 0) * 255.0 + 0.5);
		if (v.a >= 128.0) {
			let f = u - t;
			if (select(f.y > f.x, f.x + f.y > 1.0, v.a % 128.0 >= 64.0)) { return v.b + (v.a % 64.0) * 256.0; }
		}
		return v.r + v.g * 256.0;
	}
	var<private> gameRoomLit: f32 = -1.0;
	fn gameRoomLitAt(pos: vec3f, n: vec3f) -> f32 {
		if (gameRoomLit >= 0.0) { return gameRoomLit; }
		let at = pos + n * ${OUT};
		if (gameRoomHere < 0.0) { gameRoomHere = gameRoomOf(at); }
		gameRoomLit = gameRoomHere;
		if (gameRoomHere != 1.0) { return gameRoomLit; }
		let f = fract(gamePos(at).xz - uniform.game_roomLightArea.xy);
		let side = step(vec2f(1.0 - ${IN_WALL}), f) - step(f, vec2f(${IN_WALL}));
		var beyond: f32 = 0.0;
		if (side.x != 0.0) { beyond = gameRoomOf(at + vec3f(side.x * 2.0 * ${IN_WALL}, 0.0, 0.0)); }
		if (beyond < 1.5 && side.y != 0.0) { beyond = gameRoomOf(at - vec3f(0.0, 0.0, side.y * 2.0 * ${IN_WALL})); }
		if (beyond > 1.5) { gameRoomLit = beyond; }
		return gameRoomLit;
	}
	fn gameRoomSkip(lightPos: vec3f, pos: vec3f, n: vec3f) -> bool {
		if (gameRoomHere < 0.0) { gameRoomHere = gameRoomOf(pos + n * ${OUT}); }
		let there = gameRoomOf(lightPos);
		return gameRoomHere > 0.0 && there > 0.0 && gameRoomHere != there;
	}
#endif
`,
  },
};

export const ROOM_LIGHT_CODE: Record<Language, { skip: string; forward: string; noLamps: string; falloff: string; spot: string; shadowDecl: string; shadow: (read: string) => string }> = {
  glsl: {
    // Before `clusteredLightPS`, and round a spot's shadow read in it (`lampShadows.ts`): a lamp's shadow map covers
    // a cone round its way (`game_lampShadowCone`: the cosines of its edge and of where the shadows start to fade
    // towards it); past it the lamp shines unshadowed (PlayCanvas would read the maps beside it in the atlas).
    shadowDecl: `
#ifdef CLUSTER_SHADOWS
	uniform vec2 game_lampShadowCone;
	float gameLampShadow(float shadow, vec3 spotDir, vec3 lightDirNorm) {
		return mix(1.0, shadow, smoothstep(game_lampShadowCone.x, game_lampShadowCone.y, dot(lightDirNorm, spotDir)));
	}
#endif
`,
    shadow: (read) => `gameLampShadow(${read}, light.direction, lightDirNormW)`,
    // After `decodeClusterLightCore(lightIndex)` in `evaluateClusterLight`.
    skip: `
	#ifdef GAME_LOOK
		if (gameRoomSkip(clusterLightData.position, vPositionW, worldNormal)) return;
	#endif`,
    // After `lightColor` in `evaluateLight{i}` (`lightFunctionLightPS`): a forward light, the sun aside.
    forward: `
	#ifdef GAME_LOOK
		#if LIGHT{i}TYPE != DIRECTIONAL
			if (gameRoomSkip(light{i}_position, vPositionW, litArgs_worldNormal)) return;
		#endif
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
		if (lightInnerConeAngle < ${ALL_ROUND_COS}) return 1.0;
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
    shadowDecl: `
#ifdef CLUSTER_SHADOWS
	uniform game_lampShadowCone: vec2f;
	fn gameLampShadow(shadow: f32, spotDir: vec3f, lightDirNorm: vec3f) -> f32 {
		return mix(1.0, shadow, smoothstep(uniform.game_lampShadowCone.x, uniform.game_lampShadowCone.y, dot(lightDirNorm, spotDir)));
	}
#endif
`,
    shadow: (read) => `gameLampShadow(${read}, light.direction, lightDirNormW)`,
    skip: `
	#ifdef GAME_LOOK
		if (gameRoomSkip(clusterLightData.position, vPositionW, worldNormal)) { return; }
	#endif`,
    forward: `
	#ifdef GAME_LOOK
		#if LIGHT{i}TYPE != DIRECTIONAL
			if (gameRoomSkip(uniform.light{i}_position, vPositionW, litArgs_worldNormal)) { return; }
		#endif
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
		if (lightInnerConeAngle < ${ALL_ROUND_COS}) { return 1.0; }
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
