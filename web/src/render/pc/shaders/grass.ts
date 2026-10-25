/**
 * The lawn's blades, built in the vertex shader (`pc/grass.ts` draws them; where they grow and how
 * dense: `shared/grass.ts`), and the gusts on the ground beyond them.
 *
 * `GAME_GRASS`, on the grass material: each copy of a patch mesh is a 2 m patch (its matrix only
 * moves it: its corner, on the ground), and each vertex of the mesh is (t along its blade, side -1,
 * 0 or 1, the blade's number), nothing more. From a hash of the patch and the number come the
 * blade's root, turn, height, width and shade; then
 *
 * - the blade is drawn only while its number is within the share drawn at its distance
 *   (`grassShare`), shrinking to nothing over the last fifth of it, and widened by the share's
 *   inverse, so a farther, sparser lawn covers the ground as a near one does; towards the reach
 *   (`game_grassEye` w) every blade shrinks away;
 * - its height follows the lawn where it grows (`game_grassColour`'s alpha, the town's mask
 *   `game_grassTown`), with patches of longer grass (the gust noise, standing still);
 * - its tip leans its own way, is carried along the wind by the gust rolling over it (two octaves
 *   of `game_grassGust` drifting along `WIND.dir`, as `gustAt`), flutters a little, and is pushed
 *   away from the residents near it (`game_grassPush`: x, z, strength each, the strongest first,
 *   within `BLADE.push`); the tip stays
 *   about its height from the root, so a bent blade is lower;
 * - its shape is a quadratic curve from root to tip; its normal is the curve's, turned to the eye
 *   (both faces are drawn, lit alike), rounded across the blade and bent to the sky with distance,
 *   so a far lawn never draws dark;
 * - its colour is the ground's under it (`game_grassColour`, sRGB) times the ground texture's mean,
 *   dark at the root and lighter, a touch yellower, at the tip, brighter in a gust (the blades
 *   bent over show their pale sides), and far off the lawn's plain colour (`LAWN.blend`):
 *   `vGameGrass` (linear rgb, t).
 *
 * `GAME_GRASS_GROUND`, on the ground: the same gusts as a slight lightening of the lawn under them
 * (scaled by how much grass grows there, as the blades read it: the land's, and in town the lawn's mask), so the waves roll on past the blades. Colour only: the ground
 * doesn't move.
 *
 * Uniforms (`pc/grass.ts`): `game_wind` (`nature.ts`: time, strength, direction x, z),
 * `game_grassEye` (the eye, game space, and in w how far the grass is drawn: the ground blends
 * into the lawn's colour towards it too), `game_grassDensity` (the preset's, `grassShare`), `game_grassArea` (the colour texture's corner and 1 / its
 * size in metres), `game_grassTownSize` (the town's ground in metres), `game_grassPush[16]`.
 * Per copy: `game_grassSlope` (the ground's slope along x and z under its patch).
 */

import { codeFor, float as f, type ShaderFeature } from './chunks';
import { GRASS, GRASS_PUSHERS, GUST_ROTATION } from '../../shared/grass';

/**
 * Blade shape: heights (m) of short and long grass, root widths (m), the share of the height a tip
 * leans of itself, and how far a resident pushes the blades round them (m).
 */
const BLADE = { short: 0.09, long: 0.26, thin: 0.016, wide: 0.027, lean: [0.12, 0.42] as const, maxWiden: 6, push: 0.5 } as const;

/**
 * The lawn's light: a blade's shade from root to tip (times the ground's colour, desaturated to
 * `saturation`), and the ground under the blades near the eye (`under`) and past them (`far`),
 * so the lawn averages the same near and far. Over `blend` (shares of the reach) the ground goes
 * from `under` to `far` and the blades from their shade to the ground's `far` colour, together.
 */
const LAWN = { root: 0.55, tip: 1.35, saturation: 0.85, under: 0.55, far: 0.8, blend: [0.45, 0.85] as const } as const;

function grassCode(lang: 'glsl' | 'wgsl'): ShaderFeature['glsl'] {
  const { wgsl, v2, v3, v4, u, decl: d } = codeFor(lang);
  const [u2, u3, i2, uint] = wgsl ? ['vec2u', 'vec3u', 'vec2i', 'u32'] : ['uvec2', 'uvec3', 'ivec2', 'uint'];
  // (Level 0 everywhere: the textures have no mips, and the ground reads them in a branch.)
  const tex = (name: string, uv: string) => (wgsl ? `textureSampleLevel(${name}, ${name}Sampler, ${uv}, 0.0)` : `textureLod(${name}, ${uv}, 0.0)`);
  const { patch, perPatch, near } = GRASS;
  const { span, span2, weight2, speed, calm, strong } = GRASS.gust;
  const [gc, gs] = GUST_ROTATION;
  /** The gust at `p` (game xz, carried back along the wind), as `gustAt`. */
  const gust = (p: string) => `smoothstep(${f(calm)}, ${f(strong)}, (${tex('game_grassGust', `${p} / ${f(span)}`)}.r + ${tex('game_grassGust', `${v2}(${p}.x * ${f(gc)} - ${p}.y * ${f(gs)}, ${p}.x * ${f(gs)} + ${p}.y * ${f(gc)}) / ${f(span2)}`)}.r * ${f(weight2)}) / ${f(1 + weight2)})`;

  const windDecl = wgsl
    ? `
#ifndef GAME_WIND
	uniform game_wind: vec4f;
#endif`
    : `
#ifndef GAME_WIND
	uniform vec4 game_wind;
#endif`;

  const vsDecl = wgsl
    ? `
#ifdef GAME_GRASS
${windDecl}
	uniform game_grassEye: vec4f;
	uniform game_grassDensity: f32;
	uniform game_grassArea: vec4f;
	uniform game_grassTownSize: vec4f;
	uniform game_grassPush: array<vec4f, ${GRASS_PUSHERS}>;
	var game_grassColour: texture_2d<f32>;
	var game_grassColourSampler: sampler;
	var game_grassTown: texture_2d<f32>;
	var game_grassTownSampler: sampler;
	var game_grassGust: texture_2d<f32>;
	var game_grassGustSampler: sampler;
	attribute game_grassSlope: vec2f;
	varying vGameGrass: vec4f;
	var<private> gameGrassN: vec3f = vec3f(0.0, 1.0, 0.0);
	var<private> gameGrassC: vec4f = vec4f(1.0);
	fn gameGrassHash(vin: vec3u) -> vec3u {
		var v = vin * 1664525u + vec3u(1013904223u);
		v.x += v.y * v.z; v.y += v.z * v.x; v.z += v.x * v.y;
		v = v ^ (v >> vec3u(16u));
		v.x += v.y * v.z; v.y += v.z * v.x; v.z += v.x * v.y;
		return v;
	}
#endif`
    : `
#ifdef GAME_GRASS
${windDecl}
	uniform vec4 game_grassEye;
	uniform float game_grassDensity;
	uniform vec4 game_grassArea;
	uniform vec4 game_grassTownSize;
	uniform vec4 game_grassPush[${GRASS_PUSHERS}];
	uniform sampler2D game_grassColour;
	uniform sampler2D game_grassTown;
	uniform sampler2D game_grassGust;
	attribute vec2 game_grassSlope;
	varying vec4 vGameGrass;
	vec3 gameGrassN = vec3(0.0, 1.0, 0.0);
	vec4 gameGrassC = vec4(1.0);
	uvec3 gameGrassHash(uvec3 v) {
		v = v * 1664525u + 1013904223u;
		v.x += v.y * v.z; v.y += v.z * v.x; v.z += v.x * v.y;
		v ^= v >> 16u;
		v.x += v.y * v.z; v.y += v.z * v.x; v.z += v.x * v.y;
		return v;
	}
#endif`;

  /** Whether `t` (a point over the town's size) is in the town. */
  const inTown = (t: string) => (wgsl ? `all(${t} >= vec2f(0.0)) && all(${t} < vec2f(1.0))` : `all(greaterThanEqual(${t}, vec2(0.0))) && all(lessThan(${t}, vec2(1.0)))`);
  const loop = wgsl ? `for (var gi = 0; gi < ${GRASS_PUSHERS}; gi++)` : `for (int gi = 0; gi < ${GRASS_PUSHERS}; gi++)`;
  const select = (a: string, b: string, cond: string) => (wgsl ? `select(${b}, ${a}, ${cond})` : `(${cond} ? ${a} : ${b})`);

  const vsLocal = `
#if defined(GAME_GRASS) && !defined(SHADOW_PASS)
	{
		${d('gT', 'float')} = localPos.x;
		${d('gSide', 'float')} = localPos.y;
		${d('gB', 'float')} = localPos.z;
		${d('gO', v3, true)} = ${v3}(0.0);
		#ifdef INSTANCING
			gO = instance_line4.xyz;
		#endif
		${d('gH', u3)} = gameGrassHash(${u3}(${u2}(${i2}(floor(gO.xz / ${f(patch)} + 0.5)) + ${i2}(32768)), ${uint}(gB)));
		${d('gR', v3)} = ${v3}(gH) / 4294967295.0;
		${d('gR2', v3)} = ${v3}(gameGrassHash(gH ^ ${u3}(0x9e3779b9u, 0x85ebca6bu, 0xc2b2ae35u))) / 4294967295.0;
		${d('gRoot', v2)} = gR.xy * ${f(patch)};
		${d('gW', v2)} = gO.xz + gRoot;
		${d('gY', 'float')} = dot(game_grassSlope, gRoot);
		${d('gEye', v3)} = ${u('game_grassEye')}.xyz;
		${d('gD', 'float')} = length(${v3}(gW.x, gO.y + gY, gW.y) - gEye);
		${d('gReach', 'float')} = ${u('game_grassEye')}.w;
		${d('gShare', 'float')} = min(1.0, ${f(near)} / max(gD, 0.001)) * ${u('game_grassDensity')};
		// (And towards the reach, they all shrink away: the ground carries the lawn past it.)
		${d('gKeep', 'float')} = (1.0 - smoothstep(gShare * 0.8, gShare, (gB + 0.5) / ${f(perPatch)})) * (1.0 - smoothstep(gReach * 0.75, gReach, gD));
		// Where it grows: the land's grassiness, and in town the lawn's mask.
		${d('gField', v4)} = ${tex('game_grassColour', `(gW - ${u('game_grassArea')}.xy) * ${u('game_grassArea')}.zw`)};
		${d('gCov', 'float', true)} = gField.a;
		${d('gTu', v2)} = gW / ${u('game_grassTownSize')}.xy;
		if (${inTown('gTu')}) { gCov = min(gCov, ${tex('game_grassTown', 'gTu')}.r); }
		// Patches of longer grass (the gust noise, standing still).
		${d('gLong', 'float')} = ${tex('game_grassGust', `gW / 23.0 + ${v2}(0.37, 0.61)`)}.r;
		${d('gHt', 'float')} = mix(${f(BLADE.short)}, ${f(BLADE.long)}, smoothstep(0.2, 0.8, gLong)) * (0.65 + 0.7 * gR.z) * smoothstep(0.15, 0.7, gCov) * gKeep;
		${d('gWidth', 'float')} = mix(${f(BLADE.thin)}, ${f(BLADE.wide)}, gR2.x) * min(1.0 / gShare, ${f(BLADE.maxWiden)});
		${d('gYaw', 'float')} = gR2.y * 6.2831853;
		${d('gF', v2)} = ${v2}(cos(gYaw), sin(gYaw));
		${d('gAcross', v3)} = ${v3}(-gF.y, 0.0, gF.x);
		// The wind: the gust rolling over it, and a flutter of its own.
		${d('gTime', 'float')} = ${u('game_wind')}.x;
		${d('gDir', v2)} = ${u('game_wind')}.zw;
		${d('gP', v2)} = gW - gDir * ${f(speed)} * gTime;
		${d('gGust', 'float')} = ${gust('gP')} * ${u('game_wind')}.y;
		${d('gFl', 'float')} = sin(gTime * (2.1 + gR2.z * 1.8) + gR.x * 40.0 + gGust * 2.5);
		${d('gBend', 'float')} = (0.1 + 0.62 * gGust + 0.07 * gFl * (0.35 + gGust)) * ${u('game_wind')}.y;
		${d('gTip', v2, true)} = gF * gHt * mix(${f(BLADE.lean[0])}, ${f(BLADE.lean[1])}, gR2.z) + gDir * gHt * gBend;
		// Pushed aside by the residents walking through it.
		${loop} {
			${d('gPu', v4)} = ${u('game_grassPush')}[gi];
			// (The strongest first: the rest push nothing.)
			if (gPu.z <= 0.0) { break; }
			${d('gDv', v2)} = gW - gPu.xy;
			if (dot(gDv, gDv) < ${f(BLADE.push * BLADE.push)}) {
				${d('gDl', 'float')} = length(gDv);
				${d('gPf', 'float')} = gPu.z * (1.0 - smoothstep(${f(BLADE.push * 0.35)}, ${f(BLADE.push)}, gDl));
				gTip = gTip + ${select('gDv / max(gDl, 0.0001)', 'gF', 'gDl > 0.0001')} * gPf * gHt * 1.2;
			}
		}
		${d('gL', 'float')} = length(gTip);
		if (gL > gHt * 0.94) { gTip = gTip * (gHt * 0.94 / max(gL, 0.0001)); }
		${d('gUp', 'float')} = sqrt(max(gHt * gHt - dot(gTip, gTip), 0.0));
		${d('gP1', v3)} = ${v3}(gTip.x * 0.12, gUp * 0.8, gTip.y * 0.12);
		${d('gP2', v3)} = ${v3}(gTip.x, gUp, gTip.y);
		${d('gPos', v3)} = 2.0 * gT * (1.0 - gT) * gP1 + gT * gT * gP2;
		${d('gTan', v3)} = 2.0 * (1.0 - gT) * gP1 + 2.0 * gT * (gP2 - gP1) + ${v3}(0.0, 0.0001, 0.0);
		${d('gHalf', 'float')} = 0.5 * gWidth * pow(max(1.0 - gT, 0.0), 0.7);
		${d('gAt', v3)} = ${v3}(gRoot.x, gY, gRoot.y) + gPos + gAcross * (gSide * gHalf);
		// Its normal: the curve's, to the eye, rounded across it, bent to the sky with distance.
		${d('gN', v3, true)} = normalize(cross(gAcross, gTan));
		if (dot(gN, gEye - (gO + gAt)) < 0.0) { gN = -gN; }
		gN = normalize(gN + gAcross * (gSide * 0.45));
		// (vsNormal normalizes it.)
		gameGrassN = mix(gN, ${v3}(0.0, 1.0, 0.0), 0.3 + 0.45 * smoothstep(8.0, 50.0, gD));
		// Its colour: the ground's, dark at the root, paler and yellower at the tip, lighter in a gust.
		${d('gLin', v3)} = pow(gField.rgb, ${v3}(2.2)) * ${f(GRASS.groundAlbedo)};
		${d('gBase', v3)} = mix(${v3}(dot(gLin, ${v3}(0.2126, 0.7152, 0.0722))), gLin, ${f(LAWN.saturation)});
		${d('gTint', v3)} = mix(${v3}(1.0), ${v3}(1.1, 1.06, 0.8), gT * gT * 0.5) * (0.9 + 0.2 * gR.z);
		${d('gNearC', v3)} = gBase * gTint * mix(${f(LAWN.root)}, ${f(LAWN.tip)}, pow(gT, 0.8)) * (1.0 + gGust * gT * 0.2);
		// Far off, a blade is a pixel or two: it takes the lawn's plain colour, as the ground under it does over
		// the same distances, or its light tip would twinkle against the shade (as GAME_GRASS_GROUND blends the ground).
		${d('gFar', 'float')} = smoothstep(gReach * ${f(LAWN.blend[0])}, gReach * ${f(LAWN.blend[1])}, gD);
		gameGrassC = ${v4}(mix(gNearC, gLin * ${f(LAWN.far)}, gFar), gT);
		// (A blade with nothing to show is a point.)
		localPos = ${select(`${v3}(gRoot.x, gY - 0.05, gRoot.y)`, 'gAt', 'gHt < 0.004')};
	}
#endif`;

  const vsNormal = `
#if defined(GAME_GRASS) && !defined(SHADOW_PASS)
	return normalize(dNormalMatrix * gameGrassN);
#endif`;

  const vsEnd = `
#if defined(GAME_GRASS) && !defined(SHADOW_PASS)
	${wgsl ? 'output.vGameGrass' : 'vGameGrass'} = gameGrassC;
#endif`;

  const psDecl = wgsl
    ? `
#if defined(GAME_GRASS) && !defined(SHADOW_PASS)
	varying vGameGrass: vec4f;
#endif
#if defined(GAME_GRASS_GROUND) && !defined(SHADOW_PASS)
${windDecl}
	uniform game_grassArea: vec4f;
	uniform game_grassEye: vec4f;
	uniform game_grassTownSize: vec4f;
	var game_grassColour: texture_2d<f32>;
	var game_grassColourSampler: sampler;
	var game_grassTown: texture_2d<f32>;
	var game_grassTownSampler: sampler;
	var game_grassGust: texture_2d<f32>;
	var game_grassGustSampler: sampler;
#endif`
    : `
#if defined(GAME_GRASS) && !defined(SHADOW_PASS)
	varying vec4 vGameGrass;
#endif
#if defined(GAME_GRASS_GROUND) && !defined(SHADOW_PASS)
${windDecl}
	uniform vec4 game_grassArea;
	uniform vec4 game_grassEye;
	uniform vec4 game_grassTownSize;
	uniform sampler2D game_grassColour;
	uniform sampler2D game_grassTown;
	uniform sampler2D game_grassGust;
#endif`;

  const albedo = `
#if defined(GAME_GRASS) && !defined(SHADOW_PASS)
	dAlbedo = vGameGrass.rgb;
#endif
#if defined(GAME_GRASS_GROUND) && !defined(SHADOW_PASS)
	{
		${d('ggQ', v3)} = gamePos(vPositionW);
		${d('ggCov', 'float', true)} = ${tex('game_grassColour', `(ggQ.xz - ${u('game_grassArea')}.xy) * ${u('game_grassArea')}.zw`)}.a;
		// In town, as much as the blades grow there (none by a wall, under furniture or on a patio: no shade of theirs).
		if (ggCov > 0.0 && ${u('game_grassTownSize')}.x > 0.0) {
			${d('ggTu', v2)} = ggQ.xz / ${u('game_grassTownSize')}.xy;
			if (${inTown('ggTu')}) { ggCov = min(ggCov, ${tex('game_grassTown', 'ggTu')}.r); }
		}
		// (Only where the land has lawn: roads, fields and the rest of the landscape skip the gusts.)
		if (ggCov > 0.0) {
			${d('ggP', v2)} = ggQ.xz - ${u('game_wind')}.zw * ${f(speed)} * ${u('game_wind')}.x;
			${d('ggGust', 'float')} = ${gust('ggP')} * ${u('game_wind')}.y;
			// Darker under the blades near the eye (the shade between them), blending out towards the
			// grass's reach into the lawn's own colour, as dark as the blades and the shade between them average.
			${d('ggNear', 'float')} = 1.0 - smoothstep(${u('game_grassEye')}.w * ${f(LAWN.blend[0])}, ${u('game_grassEye')}.w * ${f(LAWN.blend[1])}, length(ggQ - ${u('game_grassEye')}.xyz));
			dAlbedo = dAlbedo * mix(1.0, mix(${f(LAWN.far)}, ${f(LAWN.under)}, ggNear), ggCov) * (1.0 + (ggGust - ${f(calm)}) * 0.13 * ggCov);
		}
	}
#endif`;

  return { vsDecl, vsLocal, vsNormal, vsEnd, psDecl, albedo };
}

export const GRASS_SHADER: ShaderFeature = { name: 'grass', priority: 235, glsl: grassCode('glsl'), wgsl: grassCode('wgsl') };
