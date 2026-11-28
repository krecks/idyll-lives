/**
 * Flowing water (`GAME_WATER`, on the landscape's water: the sea's sheet and a height field's ponds, `pc/water.ts`).
 * The water's surface is rippled by a small tiling texture made at run time (`game_waterRipple`: the ripples' slope
 * along x and z in RG, their height in B, and a finer noise for the foam in A), read in two ways and blended by how
 * fast the water runs:
 *
 * - Still water (lakes, the sea, a river's slack edges): two layers of ripples at different sizes, drifting slowly
 *   two ways, so nothing runs anywhere.
 * - Running water: the ripples carried along the flow map (`game_waterFlow`, a texel a sample of the land: RG its
 *   way times its speed about 0.5, B its foam; `shared/waterFlow.ts`), the usual flow map's two phases: each layer
 *   pulled along the flow for a cycle and then started again, half a cycle apart, each weighted out as it starts
 *   again, so the ripples run on downstream without the pattern ever stretching away.
 *
 * The slope tilts the normal (the sky's reflection breaks up in it). The foam is where the flow map has it (the
 * rapids), in streaks drawn out along the flow (the foam's noise read three times back up the way it runs, carried
 * as the ripples are); it lightens the water, makes it opaque and rough. The sun's own highlight on the water is held
 * to a sheen (`sunGlint`, at most `sunMost`: a glint a few pixels across, never a white patch; the sky's reflection
 * as it is). Eleven reads a pixel of water: five cost less than what the frame's time wanders by on every preset and
 * both backends (the river filling the play camera's view, M3 Pro, 2026-11-17), so Low has it too.
 *
 * Uniforms: `game_waterArea` (1 / the flow map's size in metres along x and z, then where its first sample's texel
 * is: the map's uv at a game-space point), `game_waterClock` (`pc/nature.ts`: the water's time, s, slowed with
 * Reduce motion; and the second still layer's drift, 0 with Reduce motion), and the look (`WATER`).
 */

import { tuned } from '../../tune';
import { codeFor, float as f, registerShaderFeature, type Language, type ShaderFeature } from './chunks';

/**
 * The water's look: `still` and `stillSmall`, the two still layers' sizes (m a repeat of the ripples), `drift` and
 * `driftSmall` their drift (m/s along x, z); `stillSlope`, how far their ripples tilt the water; `run`, the running
 * ripples' size (m), `runSlope` their tilt at the slowest and `runFast` more at full speed; `speed`, how fast the
 * water runs at a speed of 1 (m/s); `cycle`, a phase's length (s); `tone`, how much lighter a ripple's crest is than
 * its trough (the ripples seen from straight above, where the sky's reflection hardly changes), `runTone` on running
 * water. The foam: its colour (linear), `foamGloss` its smoothness; its noise `foamSize` m a repeat, drawn out
 * `streak` m along the flow; it covers the water from `foamReach` of the noise's height down at a full rapid (less
 * where the flow map has less), fading in over `foamEdge`, at most `foamMost`, darker in its troughs and lighter on
 * its crests (`foamShade`). `sunGlint`, the share of the sun's highlight the water keeps, and `sunMost` the most of
 * it (linear).
 */
const WATER = tuned('water', {
  still: 5.3,
  stillSmall: 2.1,
  drift: [0.05, 0.022],
  driftSmall: [-0.031, 0.043],
  stillSlope: 0.08,
  run: 3.4,
  runSlope: 0.1,
  runFast: 0.08,
  speed: 1.7,
  cycle: 1.4,
  tone: 0.16,
  runTone: 0.26,
  foam: [0.78, 0.84, 0.84],
  foamGloss: 0.3,
  foamSize: 5,
  streak: 1.4,
  foamReach: 0.5,
  foamEdge: 0.18,
  foamMost: 0.85,
  foamShade: [0.75, 1.05],
  sunGlint: 0.25,
  sunMost: 0.6,
});

function code(lang: Language): ShaderFeature['glsl'] {
  const { wgsl, v2, v3, v4, u, decl } = codeFor(lang);
  const tex = (name: string, uv: string) => (wgsl ? `textureSample(${name}, ${name}Sampler, ${uv})` : `texture(${name}, ${uv})`);
  const v2c = (p: readonly number[]) => `${v2}(${f(p[0])}, ${f(p[1])})`;
  /** The foam's noise at `uv`, drawn out back up the flow (`step`: a tap's step in uv). */
  const streak = (uv: string) => `(${tex('game_waterRipple', uv)}.a + ${tex('game_waterRipple', `${uv} - gwSt`)}.a + ${tex('game_waterRipple', `${uv} - gwSt * 2.0`)}.a) / 3.0`;
  const psDecl = wgsl
    ? `
#if defined(GAME_WATER) && !defined(SHADOW_PASS)
	uniform game_waterArea: vec4f;
	uniform game_waterClock: vec4f;
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
	uniform vec4 game_waterArea;
	uniform vec4 game_waterClock;
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
		${decl('gwT', 'float')} = ${u('game_waterClock')}.x;
		${decl('gwF', v4)} = ${tex('game_waterFlow', `gwP * ${u('game_waterArea')}.xy + ${u('game_waterArea')}.zw`)};
		// The way times the speed (128 still), and the speed.
		${decl('gwV', v2)} = (gwF.rg * 255.0 - ${v2}(128.0)) / 127.0;
		${decl('gwS', 'float')} = clamp(length(gwV), 0.0, 1.0);
		// Still water: two layers drifting two ways (the second still with Reduce motion).
		${decl('gwA', v4)} = ${tex('game_waterRipple', `gwP / ${f(WATER.still)} + ${v2c(WATER.drift)} * (gwT / ${f(WATER.still)})`)};
		${decl('gwB', v4)} = ${tex('game_waterRipple', `gwP / ${f(WATER.stillSmall)} + ${v2c(WATER.driftSmall)} * (gwT * ${u('game_waterClock')}.y / ${f(WATER.stillSmall)}) + ${v2}(0.37, 0.71)`)};
		${decl('gwSlope', v2, true)} = (gwA.rg * 2.0 - ${v2}(1.0) + (gwB.rg * 2.0 - ${v2}(1.0)) * 0.6) * ${f(WATER.stillSlope)};
		// Running water: the ripples carried along the flow, two phases half a cycle apart.
		${decl('gwPh', 'float')} = gwT / ${f(WATER.cycle)};
		${decl('gwP0', 'float')} = fract(gwPh);
		${decl('gwP1', 'float')} = fract(gwPh + 0.5);
		${decl('gwW', 'float')} = abs(1.0 - 2.0 * gwP0);
		${decl('gwM', v2)} = gwV * ${f(WATER.speed * WATER.cycle)};
		${decl('gwC', v4)} = ${tex('game_waterRipple', `(gwP - gwM * gwP0) / ${f(WATER.run)}`)};
		${decl('gwD', v4)} = ${tex('game_waterRipple', `(gwP - gwM * gwP1) / ${f(WATER.run)} + ${v2}(0.5, 0.5)`)};
		${decl('gwE', v4)} = mix(gwC, gwD, gwW);
		${decl('gwRun', 'float')} = smoothstep(0.02, 0.2, gwS);
		gwSlope = mix(gwSlope, (gwE.rg * 2.0 - ${v2}(1.0)) * (${f(WATER.runSlope)} + ${f(WATER.runFast)} * gwS), gwRun);
		${decl('gwH', 'float')} = mix(gwA.b * 0.5 + gwB.b * 0.5, gwE.b, gwRun);
		${decl('gwTone', 'float')} = mix(${f(WATER.tone)}, ${f(WATER.runTone)}, gwRun);
		gameWaterSlope = gwSlope;
		gameWaterTone = 1.0 + (gwH - 0.5) * gwTone;
		// The foam: its noise carried as the ripples are, drawn out back up the flow (streaks along it), its
		// contrast given back (three reads averaged are flatter than one).
		${decl('gwSt', v2)} = gwV / max(length(gwV), 0.05) * ${f(WATER.streak / WATER.foamSize)};
		${decl('gwQ0', v2)} = (gwP - gwM * gwP0) / ${f(WATER.foamSize)};
		${decl('gwQ1', v2)} = (gwP - gwM * gwP1) / ${f(WATER.foamSize)} + ${v2}(0.31, 0.77);
		${decl('gwN0', 'float')} = ${streak('gwQ0')};
		${decl('gwN1', 'float')} = ${streak('gwQ1')};
		${decl('gwN', 'float')} = clamp((mix(gwN0, gwN1, gwW) - 0.5) * 1.7 + 0.5, 0.0, 1.0);
		${decl('gwFoamFrom', 'float')} = 1.0 - gwF.b * ${f(WATER.foamReach)};
		gameWaterFoam = smoothstep(gwFoamFrom, gwFoamFrom + ${f(WATER.foamEdge)}, gwN) * smoothstep(0.0, 0.05, gwF.b) * ${f(WATER.foamMost)};
		gameWaterFoamTone = ${f(WATER.foamShade[0])} + ${f(WATER.foamShade[1] - WATER.foamShade[0])} * gwN;
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
  // The sun's highlight held to a sheen (the lights' specular only: the sky's reflection is `dReflection`).
  const combine = `
#if defined(GAME_WATER) && defined(LIT_SPECULAR) && !defined(SHADOW_PASS)
	dSpecularLight = min(dSpecularLight * ${f(WATER.sunGlint)}, ${v3}(${f(WATER.sunMost)}));
#endif`;
  return { psDecl, psStart, opacity, albedo, gloss, combine };
}

/** After the town's ground (237); its `combine` before the soft look's (300), which combines the light. */
const WATER_SHADER: ShaderFeature = { name: 'water', priority: 238, glsl: code('glsl'), wgsl: code('wgsl') };

registerShaderFeature(WATER_SHADER);
