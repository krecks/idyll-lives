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
 *   `game_grassTown`) and what the lawn is like there (the lawn's map `game_grassLawn`, `shared/lawn.ts`):
 *   short and even on a mown lot, with patches of longer grass on rough land (the gust noise, standing
 *   still), thinned out over worn earth;
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
 *   bent over show their pale sides), lighter or darker, drier or lusher as the lawn's map has it, and
 *   far off the lawn's plain colour (`LAWN.blend`): `vGameGrass` (linear rgb, t).
 *
 * `GAME_GRASS_GROUND`, on the ground: the shade under the blades near the eye, where they grow (the
 * land's grassiness, and in town the lawn's mask, as the blades read it), and the lawn's tone, as
 * dark as the blades and the shade between them average, with the same gusts as a slight lightening,
 * so the waves roll on past the blades, and the lawn's map as the blades read it: its tone, and bare
 * earth where it is worn. The tone and the gusts are wherever the land is lawn (the
 * terrain's vertex alpha), on past the grass's field round the town, fading out further off into the
 * land's own colour, so there is no edge where the field ends. Colour only: the ground doesn't move.
 *
 * Uniforms (`pc/grass.ts`): `game_wind` (`nature.ts`: time, strength, direction x, z),
 * `game_grassEye` (the eye, game space, and in w how far the grass is drawn: the ground blends
 * into the lawn's colour towards it too), `game_grassDensity` (the preset's, `grassShare`), `game_grassArea` (the colour texture's corner and 1 / its
 * size in metres), `game_grassTownSize` (the town's ground in metres, and from how far round it to
 * how far the lawn's tone fades out), `game_grassPush[16]`.
 * Per copy: `game_grassSlope` (the ground's slope along x and z under its patch).
 */

import { codeFor, float as f, type ShaderFeature } from './chunks';
import { GRASS, GRASS_PUSHERS, GUST_ROTATION } from '../../shared/grass';
import { tuned } from '../../tune';

/**
 * Blade shape: heights (m) of short and long grass on rough land and of a mown lot's (the lawn's map
 * says which, `shared/lawn.ts`), root widths (m), the share of the height a tip leans of itself, and how
 * far a resident pushes the blades round them (m).
 */
const BLADE = { short: 0.09, long: 0.26, mown: 0.085, thin: 0.016, wide: 0.027, lean: [0.12, 0.42] as const, maxWiden: 6, push: 0.5 } as const;

/**
 * The lawn's light: a blade's shade from root to tip (times the ground's colour, desaturated to
 * `saturation`), and the ground under the blades near the eye (`under`) and past them (`far`),
 * so the lawn averages the same near and far. Over `blend` (shares of the reach) the ground goes
 * from `under` to `far` and the blades from their shade to the ground's `far` colour, together.
 *
 * `sky`: how far a blade's normal is bent to the sky, near the eye and far off (a blade is lit as
 * turned to the eye; bent nearly all the way, every blade takes the lawn's own light and none stands
 * out bright or dark). `yellow`: the tip's yellow tint (1: the whole of it); `vary`: a blade's own
 * lightness, either way; `gust`: how much lighter a gust makes the tips (the ground's lightening
 * follows it).
 *
 * `grade`: the lawn's colour itself, for the blades and the ground alike wherever the land is lawn
 * (a worn patch keeps its earth): its saturation and value, and the most red its green may carry
 * (`maxRG`, red / green in linear light; 0: as much as it has), which keeps a meadow from reading yellow.
 *
 * The lawn's map (`shared/lawn.ts`, `game_grassLawn`), for the blades and the ground alike: `tone`, how much
 * lighter or darker its tone makes the lawn at most; `dryTint`, what its driest adds to red, green and blue
 * (its lushest takes the same away); `rough`, the tint of rough land against a mown lot, and `mown`, how
 * much shorter and more even a lot is mown (0: as the land); `earth`, bare earth's colour (linear), which
 * the ground takes where the map says so, over `earthEdge` of its values, `earthRagged` of them the
 * raggedness of its edge. `soften`: how much of the ground
 * texture's own light and dark speckle is taken out under the lawn (it shows between the blades, and is
 * most of what changes from one frame to the next as the camera moves). `shade`: how much of the ground's
 * shade round what stands on it (the town mask's second channel, `shared/groundShade.ts`) the lawn takes.
 *
 * The values are the calm lawn chosen from pictures side by side (2026-11-11, `tools/bench/play.mjs
 * --sheet lawn`): the lawn is most of a play picture, and with dark roots under light yellow tips (0.55
 * to 1.35, every blade 10 % its own way) it read as bright noise beside the flat walls and furniture.
 * The same day the ground under the blades came up to nearly their tone (`under` 0.68 to 0.76, the tip
 * down from 1.15 to 1.02) and lost most of its speckle (`soften`): measured from pictures a quarter of a
 * pixel apart, what changed from one to the next was the light blades against the darker ground between
 * them and that speckle, not a blade's own gradient or its light (`bench-results/jump/ground/notes.md`).
 */
const LAWN = tuned('lawn', {
  root: 0.72,
  tip: 1.02,
  saturation: 0.85,
  under: 0.76,
  far: 0.8,
  blend: [0.45, 0.85],
  sky: [0.85, 0.95],
  yellow: 0.4,
  vary: 0.04,
  gust: 0.08,
  grade: { saturation: 0.88, value: 0.95, maxRG: 0.8 },
  tone: 0.5,
  dryTint: [0.22, 0.05, -0.13],
  rough: [0.78, 0.88, 0.68],
  mown: 1,
  earth: [0.2, 0.14, 0.075],
  earthEdge: [0.15, 0.7],
  earthRagged: 0.5,
  soften: 0.6,
  shade: 1,
});

function grassCode(lang: 'glsl' | 'wgsl'): ShaderFeature['glsl'] {
  const { wgsl, v2, v3, v4, u, decl: d } = codeFor(lang);
  const [u2, u3, i2, uint] = wgsl ? ['vec2u', 'vec3u', 'vec2i', 'u32'] : ['uvec2', 'uvec3', 'ivec2', 'uint'];
  // (Level 0 in the vertex shader, which has no derivatives, and where the ground reads in a branch (the
  // town's mask and the gusts have no mips).)
  const tex = (name: string, uv: string) => (wgsl ? `textureSampleLevel(${name}, ${name}Sampler, ${uv}, 0.0)` : `textureLod(${name}, ${uv}, 0.0)`);
  // The ground's read of the lawn's colour, outside any branch: filtered by its mips, so the field's
  // texels don't shimmer on the ground far off (a texel a metre, a pixel several metres there).
  const texMip = (name: string, uv: string) => (wgsl ? `textureSample(${name}, ${name}Sampler, ${uv})` : `texture(${name}, ${uv})`);
  const { patch, perPatch, near } = GRASS;
  const { span, span2, weight2, speed, calm, strong } = GRASS.gust;
  const [gc, gs] = GUST_ROTATION;
  /** `c` (linear rgb, assignable) as the lawn's grade has it, by the share `w`; `tmp` names the working value. */
  const grade = (c: string, w: string, tmp: string) => {
    const g = LAWN.grade;
    if (g.saturation === 1 && g.value === 1 && g.maxRG <= 0) return '';
    return `{
			${d(tmp, v3, true)} = ${c};
			${g.maxRG > 0 ? `${tmp}.x = min(${tmp}.x, ${tmp}.y * ${f(g.maxRG)});` : ''}
			${tmp} = mix(${v3}(dot(${tmp}, ${v3}(0.2126, 0.7152, 0.0722))), ${tmp}, ${f(g.saturation)}) * ${f(g.value)};
			${c} = mix(${c}, ${tmp}, ${w});
		}`;
  };
  const list = (v: readonly number[]) => `${v3}(${v.map(f).join(', ')})`;
  /** `c` (linear rgb, assignable) as the lawn's map `m` (a texel of it) has it: its tone, how dry and how rough, by the share `w`. */
  const mapped = (c: string, m: string, w: string) =>
    `${c} = ${c} * mix(${v3}(1.0), (1.0 + (${m}.r * 2.0 - 1.0) * ${f(LAWN.tone)}) * (${v3}(1.0) + (${m}.g * 2.0 - 1.0) * ${list(LAWN.dryTint)}) * mix(${v3}(1.0), ${list(LAWN.rough)}, ${m}.a), ${w});`;
  /**
   * Where the lawn's map is read for `p` (game xz), a texel a tile: level across most of a tile and over to the
   * next in a smooth step about its edge (a smooth step taken twice), so a mown stripe has an edge and nothing
   * shows the straight lines a filter draws between texels. `s` names the working values.
   */
  const lawnUv = (p: string, s: string) => `${d(`${s}S`, v2)} = ${p} - 0.5;
			${d(`${s}F`, v2, true)} = fract(${s}S);
			${s}F = ${s}F * ${s}F * (3.0 - 2.0 * ${s}F);
			${s}F = ${s}F * ${s}F * (3.0 - 2.0 * ${s}F);
			${d(`${s}Uv`, v2)} = (floor(${s}S) + ${s}F + 0.5) / max(${u('game_grassTownSize')}.xy, ${v2}(1.0));`;
  /**
   * How bare the earth is (0..1) where the map's blue is `b`, at `p` (game xz): over `earthEdge` of its values,
   * its edge made ragged by the gusts' noise, standing still (read only where there is any, so a worn patch is
   * no square of tiles).
   */
  const earth = (name: string, b: string, p: string) => `${d(name, 'float', true)} = 0.0;
			if (${b} > 0.02) { ${name} = smoothstep(${f(LAWN.earthEdge[0])}, ${f(LAWN.earthEdge[1])}, ${b} + (${tex('game_grassGust', `${p} / 9.0 + ${v2}(0.13, 0.71)`)}.r - 0.45) * ${f(LAWN.earthRagged)}); }`;
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
	var game_grassLawn: texture_2d<f32>;
	var game_grassLawnSampler: sampler;
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
	uniform sampler2D game_grassLawn;
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
		// What the lawn is like here (the lawn's map; past the town, the land as it is, rough).
		${d('gMap', v4, true)} = ${v4}(0.5, 0.5, 0.0, 1.0);
		// The shade round what stands on the ground here (the mask's second channel, \`shared/groundShade.ts\`).
		${d('gShade', 'float', true)} = 0.0;
		if (${inTown('gTu')}) {
			${d('gTown', v4)} = ${tex('game_grassTown', 'gTu')};
			gCov = min(gCov, gTown.r);
			gShade = gTown.g;
			${lawnUv('gW', 'gM')}
			gMap = ${tex('game_grassLawn', 'gMUv')};
		}
		${d('gRough', 'float')} = max(gMap.a, ${f(1 - LAWN.mown)});
		${earth('gEarth', 'gMap.b', 'gW')}
		// Patches of longer grass on rough land (the gust noise, standing still); a mown lot short and even.
		${d('gLong', 'float')} = ${tex('game_grassGust', `gW / 23.0 + ${v2}(0.37, 0.61)`)}.r;
		${d('gWild', 'float')} = mix(${f(BLADE.short)}, ${f(BLADE.long)}, smoothstep(0.2, 0.8, gLong)) * (0.65 + 0.7 * gR.z);
		// (Over worn earth they thin out, one by one, and those left are shorter.)
		${d('gWorn', 'float')} = step(gEarth * 1.1, fract(gR.z * 61.7 + gR2.x * 17.3)) * (1.0 - 0.45 * gEarth);
		${d('gHt', 'float')} = mix(${f(BLADE.mown)} * (0.8 + 0.4 * gR.z), gWild, gRough) * smoothstep(0.15, 0.7, gCov) * gKeep * gWorn;
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
		gameGrassN = mix(gN, ${v3}(0.0, 1.0, 0.0), ${f(LAWN.sky[0])} + ${f(LAWN.sky[1] - LAWN.sky[0])} * smoothstep(8.0, 50.0, gD));
		// Its colour: the ground's, dark at the root, paler and yellower at the tip, lighter in a gust.
		${d('gLin', v3, true)} = pow(gField.rgb, ${v3}(2.2)) * ${f(GRASS.groundAlbedo)};
		${mapped('gLin', 'gMap', '1.0')}
		gLin = gLin * (1.0 - gShade * ${f(LAWN.shade)});
		${grade('gLin', 'smoothstep(0.15, 0.7, gCov)', 'gGrade')}
		${d('gBase', v3)} = mix(${v3}(dot(gLin, ${v3}(0.2126, 0.7152, 0.0722))), gLin, ${f(LAWN.saturation)});
		${d('gTint', v3)} = mix(${v3}(1.0), ${v3}(1.1, 1.06, 0.8), gT * gT * ${f(0.5 * LAWN.yellow)}) * (${f(1 - LAWN.vary)} + ${f(2 * LAWN.vary)} * gR.z);
		${d('gNearC', v3)} = gBase * gTint * mix(${f(LAWN.root)}, ${f(LAWN.tip)}, pow(gT, 0.8)) * (1.0 + gGust * gT * ${f(LAWN.gust)});
		// Far off, a blade is a pixel or two: it takes the lawn's plain colour, as the ground under it does over
		// the same distances, or its light tip would twinkle against the shade (as GAME_GRASS_GROUND blends the ground).
		${d('gFar', 'float')} = smoothstep(gReach * ${f(LAWN.blend[0])}, gReach * ${f(LAWN.blend[1])}, gD);
		gameGrassC = ${v4}(mix(gNearC, gLin * ${f(LAWN.far)}, gFar), gT);
		// (A blade with nothing to show is a point.)
		localPos = ${select(`${v3}(gRoot.x, gY - 0.05, gRoot.y)`, 'gAt', 'gHt < 0.004')};
	}
#endif`;

  const vsNormal = { when: 'defined(GAME_GRASS) && !defined(SHADOW_PASS)', code: '\treturn normalize(dNormalMatrix * gameGrassN);' };

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
	var game_grassLawn: texture_2d<f32>;
	var game_grassLawnSampler: sampler;
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
	uniform sampler2D game_grassLawn;
	uniform sampler2D game_grassGust;
#endif`;

  const albedo = `
#if defined(GAME_GRASS) && !defined(SHADOW_PASS)
	dAlbedo = vGameGrass.rgb;
#endif
#if defined(GAME_GRASS_GROUND) && !defined(SHADOW_PASS)
	{
		${d('ggQ', v3)} = gamePos(vPositionW);
		// (Outside any branch: how fast the lawn's map goes by a pixel, for its mips.)
		${d('ggDx', v2)} = ${wgsl ? 'dpdx' : 'dFdx'}(ggQ.xz) / max(${u('game_grassTownSize')}.xy, ${v2}(1.0));
		${d('ggDy', v2)} = ${wgsl ? 'dpdy' : 'dFdy'}(ggQ.xz) / max(${u('game_grassTownSize')}.xy, ${v2}(1.0));
		// Where the blades grow (the field's grassiness, fading out round its margin).
		${d('ggCov', 'float', true)} = ${texMip('game_grassColour', `(ggQ.xz - ${u('game_grassArea')}.xy) * ${u('game_grassArea')}.zw`)}.a;
		// The lawn's tone: wherever the land is lawn (the terrain's vertex alpha), fading out past the field's margin.
		#ifdef VARYING_VVERTEXCOLOR
			${d('ggLawn', 'float', true)} = vVertexColor.a;
		#else
			${d('ggLawn', 'float', true)} = ggCov;
		#endif
		${d('ggOut', v2)} = max(max(-ggQ.xz, ggQ.xz - ${u('game_grassTownSize')}.xy), ${v2}(0.0));
		ggLawn = ggLawn * (1.0 - smoothstep(${u('game_grassTownSize')}.z, ${u('game_grassTownSize')}.w, length(ggOut)));
		// (The lawn as the land has it, whatever stands on it: the map's tone and its earth go on under a bench.)
		${d('ggLand', 'float')} = ggLawn;
		${d('ggShade', 'float', true)} = 0.0;
		// In town, as much as the blades grow there (none by a wall, under furniture or on a patio: no shade of theirs).
		if (ggCov + ggLawn > 0.0 && ${u('game_grassTownSize')}.x > 0.0) {
			${d('ggTu', v2)} = ggQ.xz / ${u('game_grassTownSize')}.xy;
			if (${inTown('ggTu')}) {
				${d('ggTown', v4)} = ${tex('game_grassTown', 'ggTu')};
				${d('ggMask', 'float')} = ggTown.r;
				ggCov = min(ggCov, ggMask);
				ggLawn = min(ggLawn, ggMask);
				ggShade = ggTown.g;
			}
		}
		// (Only where the land has lawn: roads, fields and the rest of the landscape skip the gusts.)
		if (ggCov + ggLawn > 0.0) {
			${d('ggP', v2)} = ggQ.xz - ${u('game_wind')}.zw * ${f(speed)} * ${u('game_wind')}.x;
			${d('ggGust', 'float')} = ${gust('ggP')} * ${u('game_wind')}.y;
			// The lawn's own colour, as dark as the blades and the shade between them average, and darker
			// under the blades near the eye (the shade between them), blending out towards the grass's reach.
			${d('ggNear', 'float')} = 1.0 - smoothstep(${u('game_grassEye')}.w * ${f(LAWN.blend[0])}, ${u('game_grassEye')}.w * ${f(LAWN.blend[1])}, length(ggQ - ${u('game_grassEye')}.xyz));
			// What the lawn is like here (the lawn's map, as the blades read it; past the town its edge: the land as it is).
			${lawnUv('ggQ.xz', 'ggM')}
			${d('ggMap', v4)} = ${wgsl ? 'textureSampleGrad(game_grassLawn, game_grassLawnSampler, ggMUv, ggDx, ggDy)' : 'textureGrad(game_grassLawn, ggMUv, ggDx, ggDy)'};
			${earth('ggBare', 'ggMap.b', 'ggQ.xz')}
			${d('ggEarth', 'float')} = ggBare * ggLand;
			// The ground texture's own light and dark (about 1), and less of it under the lawn.
			#ifdef VARYING_VVERTEXCOLOR
				${d('ggFlat', v3)} = vVertexColor.rgb * ${f(GRASS.groundAlbedo)};
				${d('ggSpeck', 'float')} = dot(dAlbedo, ${v3}(0.2126, 0.7152, 0.0722)) / max(dot(ggFlat, ${v3}(0.2126, 0.7152, 0.0722)), 0.0001);
				dAlbedo = mix(dAlbedo, ggFlat, ${f(LAWN.soften)} * ggLand);
			#else
				${d('ggSpeck', 'float')} = 1.0;
			#endif
			${mapped('dAlbedo', 'ggMap', 'ggLand')}
			${grade('dAlbedo', 'ggLawn', 'ggGrade')}
			dAlbedo = dAlbedo * (1.0 - ggLawn * ${f(1 - LAWN.far)} - ggCov * (1.0 - ggEarth) * ggNear * ${f(LAWN.far - LAWN.under)}) * (1.0 + (ggGust - ${f(calm)}) * ${f(0.65 * LAWN.gust)} * ggLawn);
			// Bare earth where the lawn is worn away.
			dAlbedo = mix(dAlbedo, ${list(LAWN.earth)} * ggSpeck, ggEarth);
		}
		// The shade round what stands on the ground (walls, fences, things, trunks), as the blades have it.
		dAlbedo = dAlbedo * (1.0 - ggShade * ${f(LAWN.shade)});
	}
#endif`;

  return { vsDecl, vsLocal, vsNormal, vsEnd, psDecl, albedo };
}

export const GRASS_SHADER: ShaderFeature = { name: 'grass', priority: 235, glsl: grassCode('glsl'), wgsl: grassCode('wgsl') };
