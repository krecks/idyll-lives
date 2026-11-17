/**
 * Flowing water (`GAME_WATER`, on the landscape's water: the sea's sheet and a height field's ponds, `pc/water.ts`).
 * The water's surface is rippled by a small tiling texture of ripples made at run time (`game_waterRipple`: the
 * ripples' slope along x and z in RG, their height in B), read in two ways and blended by how fast the water runs:
 *
 * - Still water (lakes, the sea, a river's slack edges): two layers of ripples at different sizes, drifting slowly
 *   two ways, so nothing runs anywhere.
 * - Running water: the ripples carried along the flow map (`game_waterFlow`, a texel a sample of the land: RG its
 *   way times its speed about 0.5, B its foam; `shared/waterFlow.ts`), the usual flow map's two phases: each layer
 *   pulled along the flow for a cycle and then started again, half a cycle apart, each weighted out as it starts
 *   again, so the ripples run on downstream without the pattern ever stretching away.
 *
 * The slope tilts the normal (the sky's reflection and the sun's highlight break up in it); the foam (where the flow
 * map has it, broken up by the ripples' height) lightens the water, makes it opaque and rough. `GAME_WATER_STILL`
 * (the Low preset, `?diag=noflow` with the material's plain look kept apart) reads one layer of still ripples and no
 * flow. Time is the wind's clock (`game_wind.x`, `pc/nature.ts`).
 *
 * Uniforms: `game_waterArea` (1 / the flow map's size in metres along x and z, then half a texel along each: the
 * map's uv at a game-space point), and the look (`WATER`).
 */

import { tuned } from '../../tune';
import { codeFor, float as f, registerShaderFeature, type Language, type ShaderFeature } from './chunks';

/**
 * The water's look: `still` and `stillSmall`, the two still layers' sizes (m a repeat of the ripples), `drift` and
 * `driftSmall` their drift (m/s along x, z); `stillSlope`, how far their ripples tilt the water; `run`, the running
 * ripples' size (m), `runSlope` their tilt at the slowest and `runFast` more at full speed; `speed`, how fast the
 * water runs at a speed of 1 (m/s); `cycle`, a phase's length (s); `foam`, the foam's colour (linear) and `foamGloss`
 * its smoothness; the foam is on the ripples' crests, from `foamReach` of the way down at a full rapid (less where the
 * flow map has less), fading in over `foamEdge` of their height, at most `foamMost`, darker in the troughs and lighter
 * on the crests (`foamShade`); `tone`, how much lighter a
 * ripple's crest is than its trough (the ripples seen from straight above, where the sky's reflection hardly
 * changes), `runTone` on running water.
 */
export const WATER = tuned('water', {
  still: 5.3,
  stillSmall: 2.1,
  drift: [0.05, 0.022],
  driftSmall: [-0.031, 0.043],
  stillSlope: 0.09,
  run: 3.4,
  runSlope: 0.16,
  runFast: 0.22,
  speed: 1.7,
  cycle: 1.4,
  foam: [0.78, 0.84, 0.84],
  foamGloss: 0.3,
  foamReach: 0.8,
  foamEdge: 0.25,
  foamMost: 0.85,
  foamShade: [0.72, 1.08],
  tone: 0.16,
  runTone: 0.3,
});

function code(lang: Language): ShaderFeature['glsl'] {
  const { wgsl, v2, v3, v4, u, decl } = codeFor(lang);
  const tex = (name: string, uv: string) => (wgsl ? `textureSample(${name}, ${name}Sampler, ${uv})` : `texture(${name}, ${uv})`);
  const v2c = (p: readonly number[]) => `${v2}(${f(p[0])}, ${f(p[1])})`;
  const psDecl = wgsl
    ? `
#if defined(GAME_WATER) && !defined(SHADOW_PASS)
#if !defined(GAME_WIND) && !defined(GAME_GRASS_GROUND)
	uniform game_wind: vec4f;
#endif
	uniform game_waterArea: vec4f;
	var game_waterFlow: texture_2d<f32>;
	var game_waterFlowSampler: sampler;
	var game_waterRipple: texture_2d<f32>;
	var game_waterRippleSampler: sampler;
	var<private> gameWaterSlope: vec2f;
	var<private> gameWaterFoam: f32;
	var<private> gameWaterTone: f32;
	var<private> gameWaterFoamTone: f32;
#endif`
    : `
#if defined(GAME_WATER) && !defined(SHADOW_PASS)
#if !defined(GAME_WIND) && !defined(GAME_GRASS_GROUND)
	uniform vec4 game_wind;
#endif
	uniform vec4 game_waterArea;
	uniform sampler2D game_waterFlow;
	uniform sampler2D game_waterRipple;
	vec2 gameWaterSlope;
	float gameWaterFoam;
	float gameWaterTone;
	float gameWaterFoamTone;
#endif`;
  // (Every read outside any branch: WGSL wants its implicit derivatives in uniform control flow.)
  const psStart = `
#if defined(GAME_WATER) && !defined(SHADOW_PASS)
	{
		${decl('gwP', v2)} = gamePos(vPositionW).xz;
		${decl('gwT', 'float')} = ${u('game_wind')}.x;
		${decl('gwF', v4)} = ${tex('game_waterFlow', `gwP * ${u('game_waterArea')}.xy + ${u('game_waterArea')}.zw`)};
		// The way times the speed (128 still), and the speed.
		${decl('gwV', v2)} = (gwF.rg * 255.0 - ${v2}(128.0)) / 127.0;
		${decl('gwS', 'float')} = clamp(length(gwV), 0.0, 1.0);
		// Still water: two layers drifting two ways.
		${decl('gwA', v4)} = ${tex('game_waterRipple', `gwP / ${f(WATER.still)} + ${v2c(WATER.drift)} * (gwT / ${f(WATER.still)})`)};
	#ifdef GAME_WATER_STILL
		${decl('gwSlope', v2, true)} = (gwA.rg * 2.0 - ${v2}(1.0)) * ${f(WATER.stillSlope * 1.4)};
		${decl('gwH', 'float', true)} = gwA.b;
		${decl('gwTone', 'float')} = ${f(WATER.tone)};
	#else
		${decl('gwB', v4)} = ${tex('game_waterRipple', `gwP / ${f(WATER.stillSmall)} + ${v2c(WATER.driftSmall)} * (gwT / ${f(WATER.stillSmall)}) + ${v2}(0.37, 0.71)`)};
		${decl('gwSlope', v2, true)} = (gwA.rg * 2.0 - ${v2}(1.0) + (gwB.rg * 2.0 - ${v2}(1.0)) * 0.6) * ${f(WATER.stillSlope)};
		// Running water: the ripples carried along the flow, two phases half a cycle apart.
		${decl('gwPh', 'float')} = gwT / ${f(WATER.cycle)};
		${decl('gwP0', 'float')} = fract(gwPh);
		${decl('gwP1', 'float')} = fract(gwPh + 0.5);
		${decl('gwM', v2)} = gwV * ${f(WATER.speed * WATER.cycle)};
		${decl('gwC', v4)} = ${tex('game_waterRipple', `(gwP - gwM * gwP0) / ${f(WATER.run)}`)};
		${decl('gwD', v4)} = ${tex('game_waterRipple', `(gwP - gwM * gwP1) / ${f(WATER.run)} + ${v2}(0.5, 0.5)`)};
		${decl('gwE', v4)} = mix(gwC, gwD, abs(1.0 - 2.0 * gwP0));
		${decl('gwRun', 'float')} = smoothstep(0.02, 0.2, gwS);
		gwSlope = mix(gwSlope, (gwE.rg * 2.0 - ${v2}(1.0)) * (${f(WATER.runSlope)} + ${f(WATER.runFast)} * gwS), gwRun);
		${decl('gwH', 'float', true)} = mix(gwA.b * 0.5 + gwB.b * 0.5, gwE.b, gwRun);
		${decl('gwTone', 'float')} = mix(${f(WATER.tone)}, ${f(WATER.runTone)}, gwRun);
	#endif
		gameWaterSlope = gwSlope;
		gameWaterTone = 1.0 + (gwH - 0.5) * gwTone;
		// The foam, broken up by the ripples: solid where there is most, in streaks and flecks where there is less.
		${decl('gwFoamFrom', 'float')} = 1.0 - gwF.b * ${f(WATER.foamReach)};
		gameWaterFoam = smoothstep(gwFoamFrom, gwFoamFrom + ${f(WATER.foamEdge)}, gwH) * smoothstep(0.0, 0.05, gwF.b) * ${f(WATER.foamMost)};
		gameWaterFoamTone = ${f(WATER.foamShade[0])} + ${f(WATER.foamShade[1] - WATER.foamShade[0])} * gwH;
	}
#endif`;
  const opacity = `
#if defined(GAME_WATER) && !defined(SHADOW_PASS)
	dAlpha = mix(dAlpha, 1.0, gameWaterFoam * 0.9);
#endif`;
  const albedo = `
#if defined(GAME_WATER) && !defined(SHADOW_PASS)
	dAlbedo = mix(dAlbedo * gameWaterTone, ${v3}(${WATER.foam.map(f).join(', ')}) * gameWaterFoamTone, gameWaterFoam);
#endif`;
  // (After the normal is worked out, `getNormal`: the slope tilts it; in engine space z runs the other way.)
  const gloss = `
#if defined(GAME_WATER) && !defined(SHADOW_PASS)
	litArgs_worldNormal = normalize(litArgs_worldNormal + ${v3}(-gameWaterSlope.x, 0.0, gameWaterSlope.y));
	dGlossiness = mix(dGlossiness, ${f(WATER.foamGloss)}, gameWaterFoam);
#endif`;
  return { psDecl, psStart, opacity, albedo, gloss };
}

/** After the town's ground (237). */
export const WATER_SHADER: ShaderFeature = { name: 'water', priority: 238, glsl: code('glsl'), wgsl: code('wgsl') };

registerShaderFeature(WATER_SHADER);
