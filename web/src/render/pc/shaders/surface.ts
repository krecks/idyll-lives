/**
 * Shader features of the surfaces themselves (vertex tweaks, patterns and plain finishes):
 *
 * - Vertex tweaks. Vertex colours are authored in sRGB: PlayCanvas converts them itself
 *   (`vertexColorGamma`), and `GAME_SRGB_COLORS` does the same for a per-instance colour
 *   (`GAME_INSTANCE_COLOR`: the `INSTANCE_COLOR_SEMANTIC` stream, multiplied into the albedo; with
 *   `GAME_TINT_WHITE` only where the vertex colour is pure white, a parked car's paint, `shared/cars.ts`). A
 *   glow's strength per instance (`GAME_GLOW_TINT`, `models.ts` `glowByInstance`) multiplies the
 *   same stream into its (unlit, emissive) vertex colour, linearised as the glow's is; its alpha up
 *   to 1 into the vertex colour's (the overview's lot highlights blend by it), and past 1 into the
 *   emission after it (linear: PlayCanvas clamps the vertex colour to 1; a lamp's glass lit
 *   `LAMP_GLASS` times as bright, `lightUnits.ts`).
 *   Planar UVs (`GAME_PLANAR_UV`) are the local XZ position in metres, for flat meshes built
 *   with 0..1 UVs such as roads (`uv0VS`, in `chunks.ts`).
 * - Patterns over a surface's texture: `GAME_PATTERN_STRIPES`, wallpaper's faint stripes 12.5 cm
 *   apart along the wall (by world position, so walls along x and z both get them), fading where
 *   they'd be finer than a pixel or two; `GAME_PATTERN_WHITEWASH`, wood limed pale with its grain
 *   still showing.
 * - The plain finish (`GAME_PLAIN_FINISH`): metalness and roughness from the vertex colour's alpha
 *   (`encodeFinish`), so a model with five untextured finishes is one draw call; and metals don't
 *   reflect the ground: a reflection that would go down goes as far up instead, flattened towards
 *   the horizon (`SKY_FOR_GROUND`), and the low sky stands in for the room the game has no map of.
 * - A finish's layers (`GAME_FINISH_LAYERS`, `materials.ts` `layer`): cloth's sheen and a lacquer's clear
 *   coat are PlayCanvas's own lobes (`useSheen`, `clearCoat`), whose reflections read the sky wherever
 *   the surface is (`reflectionSheenPS`, `reflectionCCPS`). In a room of the lot in view they reflect
 *   the room instead, as the surface under them does (`envBrdf.ts` `ROOM`): a unit of its picture
 *   here, given the room's light in `combineColor` (`softLook.ts`), which also brings the lights'
 *   highlights in the layers to the look's strength.
 */

import type { Language, ShaderFeature } from './chunks';
import { ROOM } from './envBrdf';
import { OUT } from './roomLights';

/** How much darker (linear albedo) wallpaper's darker stripes are. */
const STRIPE_DEPTH = '0.12';
/** How far up a metal's reflection goes for one that would go down (1: as far up as it went down). */
const SKY_FOR_GROUND = '0.3';
/** Under this a vertex colour's channel isn't white (`GAME_TINT_WHITE`): 255 is 1, 254 under it. */
const WHITE = '0.998';
/** The plain finish where a mesh has no vertex colours: matte (`encodeFinish(0.75, 0)`). */
const MATTE = '0.375';

export const VERTEX_TWEAKS: ShaderFeature = {
  name: 'vertexTweaks',
  priority: 200,
  glsl: {
    vsDecl: `
#ifdef GAME_INSTANCE_COLOR
	attribute vec4 game_instanceColor;
	varying vec4 vGameTint;
#elif defined(GAME_GLOW_TINT)
	attribute vec4 game_instanceColor;
	varying float vGameGlow;
#endif`,
    vsEnd: `
#ifdef GAME_INSTANCE_COLOR
	vGameTint = game_instanceColor;
	#ifdef GAME_SRGB_COLORS
		vGameTint.rgb = pow(max(vGameTint.rgb, vec3(0.0)), vec3(2.2));
	#endif
	#if defined(GAME_TINT_WHITE) && defined(VERTEX_COLOR)
		if (min(vVertexColor.r, min(vVertexColor.g, vVertexColor.b)) < ${WHITE}) vGameTint = vec4(1.0);
	#endif
#endif
#if defined(GAME_GLOW_TINT) && defined(VERTEX_COLOR)
	vVertexColor *= vec4(pow(max(game_instanceColor.rgb, vec3(0.0)), vec3(2.2)), min(game_instanceColor.a, 1.0));
#endif
#if defined(GAME_GLOW_TINT) && !defined(GAME_INSTANCE_COLOR)
	vGameGlow = max(game_instanceColor.a, 1.0);
#endif`,
    psDecl: `
#ifdef GAME_INSTANCE_COLOR
	varying vec4 vGameTint;
#elif defined(GAME_GLOW_TINT)
	varying float vGameGlow;
#endif`,
    albedo: `
#ifdef GAME_INSTANCE_COLOR
	dAlbedo *= vGameTint.rgb;
#endif`,
    emissive: `
#if defined(GAME_GLOW_TINT) && !defined(GAME_INSTANCE_COLOR)
	dEmission *= vGameGlow;
#endif`,
  },
  wgsl: {
    vsDecl: `
#ifdef GAME_INSTANCE_COLOR
	attribute game_instanceColor: vec4f;
	varying vGameTint: vec4f;
#elif defined(GAME_GLOW_TINT)
	attribute game_instanceColor: vec4f;
	varying vGameGlow: f32;
#endif`,
    vsEnd: `
#ifdef GAME_INSTANCE_COLOR
	output.vGameTint = game_instanceColor;
	#ifdef GAME_SRGB_COLORS
		output.vGameTint = vec4f(pow(max(game_instanceColor.rgb, vec3f(0.0)), vec3f(2.2)), game_instanceColor.a);
	#endif
	#if defined(GAME_TINT_WHITE) && defined(VERTEX_COLOR)
		if (min(output.vVertexColor.r, min(output.vVertexColor.g, output.vVertexColor.b)) < ${WHITE}) {
			output.vGameTint = vec4f(1.0);
		}
	#endif
#endif
#if defined(GAME_GLOW_TINT) && defined(VERTEX_COLOR)
	output.vVertexColor = output.vVertexColor * vec4f(pow(max(game_instanceColor.rgb, vec3f(0.0)), vec3f(2.2)), min(game_instanceColor.a, 1.0));
#endif
#if defined(GAME_GLOW_TINT) && !defined(GAME_INSTANCE_COLOR)
	output.vGameGlow = max(game_instanceColor.a, 1.0);
#endif`,
    psDecl: `
#ifdef GAME_INSTANCE_COLOR
	varying vGameTint: vec4f;
#elif defined(GAME_GLOW_TINT)
	varying vGameGlow: f32;
#endif`,
    albedo: `
#ifdef GAME_INSTANCE_COLOR
	dAlbedo = dAlbedo * vGameTint.rgb;
#endif`,
    emissive: `
#if defined(GAME_GLOW_TINT) && !defined(GAME_INSTANCE_COLOR)
	dEmission = dEmission * vGameGlow;
#endif`,
  },
};

export const PATTERN: ShaderFeature = {
  name: 'pattern',
  priority: 210,
  glsl: {
    albedo: `
#ifdef GAME_PATTERN_STRIPES
	{
		vec3 stripePos = gamePos(vPositionW);
		float stripeAt = (stripePos.x + stripePos.z) * 8.0;
		float stripeFade = clamp(1.5 - fwidth(stripeAt) * 3.0, 0.0, 1.0);
		dAlbedo *= 1.0 - ${STRIPE_DEPTH} * stripeFade * smoothstep(0.35, 0.65, 0.5 + 0.5 * cos(6.2831853 * stripeAt));
	}
#endif
#ifdef GAME_PATTERN_WHITEWASH
	dAlbedo = vec3(1.0) - (vec3(1.0) - mix(dAlbedo, vec3(dot(dAlbedo, vec3(0.3, 0.59, 0.11))), 0.3)) * 0.65;
#endif`,
  },
  wgsl: {
    albedo: `
#ifdef GAME_PATTERN_STRIPES
	{
		let stripePos = gamePos(vPositionW);
		let stripeAt = (stripePos.x + stripePos.z) * 8.0;
		let stripeFade = clamp(1.5 - fwidth(stripeAt) * 3.0, 0.0, 1.0);
		dAlbedo = dAlbedo * (1.0 - ${STRIPE_DEPTH} * stripeFade * smoothstep(0.35, 0.65, 0.5 + 0.5 * cos(6.2831853 * stripeAt)));
	}
#endif
#ifdef GAME_PATTERN_WHITEWASH
	dAlbedo = vec3f(1.0) - (vec3f(1.0) - mix(dAlbedo, vec3f(dot(dAlbedo, vec3f(0.3, 0.59, 0.11))), 0.3)) * 0.65;
#endif`,
  },
};

export const PLAIN_FINISH: ShaderFeature = {
  name: 'plainFinish',
  priority: 220,
  glsl: {
    psDecl: `
#ifdef GAME_PLAIN_FINISH
	float gameFinish() {
		#ifdef VARYING_VVERTEXCOLOR
			return vVertexColor.a;
		#else
			return ${MATTE};
		#endif
	}
#endif`,
    metalness: `
#ifdef GAME_PLAIN_FINISH
	dMetalness = step(0.5, gameFinish());
#endif`,
    gloss: `
#ifdef GAME_PLAIN_FINISH
	{
		float finish = gameFinish();
		dGlossiness = 1.0 - clamp((finish - 0.5 * step(0.5, finish)) * 2.0, 0.03, 1.0) + 0.0000001;
	}
#endif`,
    reflDir: `
#ifdef GAME_PLAIN_FINISH
	if (gameFinish() >= 0.5 && dReflDirW.y < 0.0) {
		dReflDirW.y *= -${SKY_FOR_GROUND};
		dReflDirW = normalize(dReflDirW);
	}
#endif`,
  },
  wgsl: {
    psDecl: `
#ifdef GAME_PLAIN_FINISH
	fn gameFinish() -> f32 {
		#ifdef VARYING_VVERTEXCOLOR
			return vVertexColor.a;
		#else
			return ${MATTE};
		#endif
	}
#endif`,
    metalness: `
#ifdef GAME_PLAIN_FINISH
	dMetalness = step(0.5, gameFinish());
#endif`,
    gloss: `
#ifdef GAME_PLAIN_FINISH
	{
		let finish = gameFinish();
		dGlossiness = 1.0 - clamp((finish - 0.5 * step(0.5, finish)) * 2.0, 0.03, 1.0) + 0.0000001;
	}
#endif`,
    reflDir: `
#ifdef GAME_PLAIN_FINISH
	if (gameFinish() >= 0.5 && dReflDirW.y < 0.0) {
		dReflDirW = normalize(vec3f(dReflDirW.x, dReflDirW.y * -${SKY_FOR_GROUND}, dReflDirW.z));
	}
#endif`,
  },
};

/** The lines of PlayCanvas's sheen and clear coat reflections replaced (also in `tools/render/chunks.json`). */
export const FINISH_LAYER_ANCHORS = {
  glsl: {
    reflectionSheenPS: 'sReflection += calcReflection(worldNormal, 0.0) * saturate(DG);',
    reflectionCCPS: 'ccReflection += calcReflection(reflDir, gloss);',
  },
  wgsl: {
    reflectionSheenPS: 'sReflection = sReflection + (calcReflection(worldNormal, 0.0) * saturate(dg));',
    reflectionCCPS: 'ccReflection = ccReflection + calcReflection(reflDir, gloss);',
  },
} as const;

/**
 * The replaced lines on a finish with layers: in a room a unit of the room's picture, the sheen the room all
 * round, the coat its walls over its floor. (The sky's is read either way: WGSL reads no texture under a
 * branch on the fragment's room.)
 */
const LAYER_ROOM: Record<Language, Record<keyof (typeof FINISH_LAYER_ANCHORS)['glsl'], string>> = {
  glsl: {
    reflectionSheenPS: `vec3 gameSky = calcReflection(worldNormal, 0.0);
		sReflection += (gameRoomHere > 1.5 ? vec3(1.0) : gameSky) * saturate(DG);`,
    reflectionCCPS: `vec3 gameSky = calcReflection(reflDir, gloss);
		ccReflection += gameRoomHere > 1.5 ? vec3(mix(${ROOM.floor}, 1.0, smoothstep(-${ROOM.turn}, ${ROOM.turn}, reflDir.y))) : gameSky;`,
  },
  wgsl: {
    reflectionSheenPS: `let gameSky = calcReflection(worldNormal, 0.0);
		sReflection = sReflection + select(gameSky, vec3f(1.0), gameRoomHere > 1.5) * saturate(dg);`,
    reflectionCCPS: `let gameSky = calcReflection(reflDir, gloss);
		ccReflection = ccReflection + select(gameSky, vec3f(mix(${ROOM.floor}, 1.0, smoothstep(-${ROOM.turn}, ${ROOM.turn}, reflDir.y))), gameRoomHere > 1.5);`,
  },
};

type Replace = (code: string, name: string, anchor: string, replacement: string) => string;

/**
 * The game's `reflectionSheenPS` or `reflectionCCPS` from the engine's (`engine`, in `lang`): on a finish
 * with layers (`GAME_FINISH_LAYERS`) the room's picture in a room, the sky's elsewhere and on every other material.
 */
export function finishLayerReflection(engine: string, name: keyof (typeof FINISH_LAYER_ANCHORS)['glsl'], lang: Language, replaceOnce: Replace): string {
  const anchor = FINISH_LAYER_ANCHORS[lang][name];
  const room = lang === 'glsl' ? `if (gameRoomHere < 0.0) gameRoomHere = gameRoomOf(vPositionW + litArgs_worldNormal * ${OUT});` : `if (gameRoomHere < 0.0) { gameRoomHere = gameRoomOf(vPositionW + litArgs_worldNormal * ${OUT}); }`;
  return replaceOnce(
    engine,
    name,
    anchor,
    `
	#ifdef GAME_FINISH_LAYERS
		${room}
		${LAYER_ROOM[lang][name]}
	#else
		${anchor}
	#endif`,
  );
}
