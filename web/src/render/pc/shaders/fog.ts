// Follows Babylon.js 9.30's PBR fog (`fogFragment.fx`: the eye's distance, the factor in linear space).
// Changed for Idyll Lives: as lines of PlayCanvas's `fogPS`.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
/**
 * The haze as the look was tuned with (the PBR fog this follows), in the game's lit materials
 * (`GAME_LOOK`): `fogPS` with three lines changed (anchors also in `tools/render/chunks.json`).
 *
 * - The distance is the eye's (`length` of the view-space position), not the depth along the view
 *   (PlayCanvas's `gl_FragCoord.z / w`): the haze is as thick at the picture's sides as in its
 *   middle.
 * - The fog factor is taken into linear space (`pow(f, 2.2)`, as the original does for PBR
 *   materials): the haze is about 2.2 times as strong near.
 * - The haze begins `game_fogFrom` m from the eye (`setFogFrom`, one value a device; 0, from the eye, unless
 *   told): the distance is counted from there. The play camera's is 0; from high over the town (the town view,
 *   the menus' town: `shared/townCamera.ts` `townFogFrom`, `menuFogFrom`) it is a share of the camera's distance,
 *   so the town in the middle of the picture is as clear as the play camera's lot and only what lies well past it
 *   fades into the horizon. A density low enough to clear the town from up there washed the near half of the
 *   picture, and left nothing for the far half; one high enough to fade the far land greyed the town.
 * - The menus' land fades into the haze at its edge (`setLandVeil`, `shared/landVeil.ts`): what is left of a thing
 *   is taken down by the veil where it stands (its x and z, clear `game_landVeil.z/w` m either side of the middle
 *   `game_landVeil.xy`, whole `game_veilFade.x` m past that, times its strength `game_veilFade.y`; 0 but in the
 *   menus' town), and by how far it is from the eye (clear at the eye, whole from `game_veilFade.z` m: a camera in it
 *   sees what is round it). By the ground each pixel shows, so it lies on the land as seen from any camera.
 *   What the veil takes goes to the sky's dome behind the thing (`gameSkyAt`, from `setSky`), not to the haze's
 *   one colour: the dome is that colour only below the horizon's band (above it the zenith's blue comes in, and in
 *   it the glow round the sun), so the ring's far hills, wholly in the veil, stood as flat wedges of the haze's colour
 *   against the sky where they rose to the horizon. The haze by distance keeps its colour; without a veil (the game
 *   over its landscape) nothing of the sky is read.
 *
 * Other materials (unlit decals, particles) keep PlayCanvas's own.
 */

import * as pc from '../engine';
import { GLOW_FROM, SKY_BAND, SKY_COLUMNS, SKY_GLOW, SKY_ROWS, type DomeSky } from '../../shared/skyMap';
import { VEIL_EDGE_SOFT } from '../../shared/landVeil';
import type { Language, ShaderFeature } from './chunks';

/** A number as a shader's float literal (`chunks.ts`'s, which imports this file). */
const float = (n: number): string => (Number.isInteger(n) ? n.toFixed(1) : String(n));

/** The lines of `fogPS` replaced: the fragment's distance, and the exponential-squared factor. */
const FOG_ANCHORS = {
  glsl: { depth: 'float depth = gl_FragCoord.z / gl_FragCoord.w;', exp2: 'fogFactor = exp(-depth * depth * fog_density * fog_density);', mix: 'return mix(fog_color * dBlendModeFogFactor, color, getFogFactor());' },
  wgsl: { depth: 'let depth = pcPosition.z / pcPosition.w;', exp2: 'return evaluateFogFactorExp2(depth, uniform.fog_density);', mix: 'return mix(uniform.fog_color * dBlendModeFogFactor, color, getFogFactor());' },
} as const;

/**
 * The sky's dome behind a direction (PlayCanvas space, `gameSkyAt`), as `shared/skyMap.ts` has it: its colour
 * (`gameDome`, `domeColour`: from the uniforms `setSky` tells, the dome's own) at its four vertices round the direction,
 * read between them as the dome's triangles have it between them. What the land's veil takes a thing to where it is
 * whole. (Uniforms, not a map of the dome: a ground's program on WebGL2 reads its 16 textures already.)
 */
function skyCode(lang: Language): string {
  const glsl = lang === 'glsl';
  const [rows, cols] = [float(SKY_ROWS - 1), float(SKY_COLUMNS - 1)];
  const [perRow, perCol] = [float((SKY_ROWS - 1) / Math.PI), float((SKY_COLUMNS - 1) / (2 * Math.PI))];
  const [rowAngle, colAngle] = [float(Math.PI / (SKY_ROWS - 1)), float((2 * Math.PI) / (SKY_COLUMNS - 1))];
  const [b0, b1, from] = [float(SKY_BAND[0]), float(SKY_BAND[1]), float(GLOW_FROM)];
  const [g0, p0, g1, p1] = SKY_GLOW.map(float);
  const u = (n: string) => (glsl ? n : `uniform.${n}`);
  const glow = `pow(s, ${p0}) * ${g0} + pow(s, ${p1}) * ${g1}`;
  const dome = `${u('game_skyLow')} + (${u('game_skyHigh')} - ${u('game_skyLow')}) * smoothstep(${b0}, ${b1}, d.y) + ${u('game_skyGlow')} * g`;
  // (Straight up or down any column will do: the row's vertices are all the pole.)
  return glsl
    ? `
	uniform vec3 game_skyLow;
	uniform vec3 game_skyHigh;
	uniform vec3 game_skySun;
	uniform vec3 game_skyGlow;
	vec3 gameDome(vec3 d) {
		float s = max(0.0, dot(d, game_skySun));
		float g = d.y > ${from} ? ${glow} : 0.0;
		return pow(pow(${dome}, vec3(2.2)), vec3(2.2));
	}
	vec3 gameDomeAt(vec2 at) {
		float a = at.y * ${rowAngle};
		float b = at.x * ${colAngle};
		return gameDome(vec3(sin(a) * cos(b), cos(a), sin(a) * sin(b)));
	}
	vec3 gameSkyAt(vec3 d) {
		float row = acos(clamp(d.y, -1.0, 1.0)) * ${perRow};
		float col = atan(d.z, abs(d.x) + abs(d.z) > 0.0 ? d.x : 1.0) * ${perCol};
		if (col < 0.0) col += ${cols};
		vec2 at = vec2(min(floor(col), ${cols} - 1.0), min(floor(row), ${rows} - 1.0));
		vec2 f = vec2(col, row) - at;
		return mix(mix(gameDomeAt(at), gameDomeAt(at + vec2(1.0, 0.0)), f.x), mix(gameDomeAt(at + vec2(0.0, 1.0)), gameDomeAt(at + vec2(1.0, 1.0)), f.x), f.y);
	}
`
    : `
	uniform game_skyLow: vec3f;
	uniform game_skyHigh: vec3f;
	uniform game_skySun: vec3f;
	uniform game_skyGlow: vec3f;
	fn gameDome(d: vec3f) -> vec3f {
		let s = max(0.0, dot(d, uniform.game_skySun));
		let g = select(0.0, ${glow}, d.y > ${from});
		return pow(pow(${dome}, vec3f(2.2)), vec3f(2.2));
	}
	fn gameDomeAt(at: vec2f) -> vec3f {
		let a = at.y * ${rowAngle};
		let b = at.x * ${colAngle};
		return gameDome(vec3f(sin(a) * cos(b), cos(a), sin(a) * sin(b)));
	}
	fn gameSkyAt(d: vec3f) -> vec3f {
		let row = acos(clamp(d.y, -1.0, 1.0)) * ${perRow};
		var col = atan2(d.z, select(1.0, d.x, abs(d.x) + abs(d.z) > 0.0)) * ${perCol};
		if (col < 0.0) { col += ${cols}; }
		let at = vec2f(min(floor(col), ${cols} - 1.0), min(floor(row), ${rows} - 1.0));
		let f = vec2f(col, row) - at;
		return mix(mix(gameDomeAt(at), gameDomeAt(at + vec2f(1.0, 0.0)), f.x), mix(gameDomeAt(at + vec2f(0.0, 1.0)), gameDomeAt(at + vec2f(1.0, 1.0)), f.x), f.y);
	}
`;
}

/**
 * The haze's start's and the veil's declarations, put before the engine's `fogPS` (whose own uniforms are declared
 * inside its `#if`s): in the fragment shader, how much of the veil a fragment has (`gameVeil`, set by
 * `getFogFactor`, read by `addFog`) and the sky behind it (`gameSkyAt`).
 */
const FOG_FROM_DECL: Record<Language, string> = {
  glsl: `#ifdef GAME_LOOK\n\tuniform float game_fogFrom;\n\tuniform vec4 game_landVeil;\n\tuniform vec4 game_veilFade;\n\t#ifndef VERTEXSHADER\n\tfloat gameVeil = 0.0;${skyCode('glsl')}\t#endif\n#endif\n`,
  wgsl: `#ifdef GAME_LOOK\n\tuniform game_fogFrom: f32;\n\tuniform game_landVeil: vec4f;\n\tuniform game_veilFade: vec4f;\n\t#ifndef VERTEXSHADER\n\tvar<private> gameVeil: f32 = 0.0;${skyCode('wgsl')}\t#endif\n#endif\n`,
};

/**
 * The veil's strength at a point `out` m past its clear rectangle (`setLandVeil`) and `eye` m from the eye: its strength
 * (`game_veilFade.y`), but rising to whole over the landscape's last `game_veilFade.w` m (`VEIL_EDGE`, 0: not), over the
 * whole band near the eye (`VEIL_EDGE_SOFT`), while the curtain closing its cut edge is drawn (`landHaze.ts`), so the
 * land meets that curtain in the sky's colour at any strength (`shared/landVeil.ts` `veilShown`).
 */
function veilStrength(lang: Language, out: string, eye: string): string {
  const v = lang === 'glsl' ? 'game_veilFade' : 'uniform.game_veilFade';
  const over = `mix(${v}.x, max(${v}.w, 1e-3), smoothstep(${float(VEIL_EDGE_SOFT[0])}, ${float(VEIL_EDGE_SOFT[1])}, ${eye}))`;
  return `max(${v}.y, step(1e-6, ${v}.w) * smoothstep(${v}.x - ${over}, ${v}.x, ${out}))`;
}

const FOG_CODE: Record<Language, { depth: string; exp2: string; mix: string }> = {
  glsl: {
    depth: `
		#ifdef GAME_LOOK
			float depth = max(0.0, distance(vPositionW, view_position) - game_fogFrom);
		#else
			float depth = gl_FragCoord.z / gl_FragCoord.w;
		#endif`,
    exp2: `
		fogFactor = exp(-depth * depth * fog_density * fog_density);
		#ifdef GAME_LOOK
			fogFactor = pow(fogFactor, 2.2);
			#ifndef VERTEXSHADER
				// (Nothing to work out without a veil, as the game nearly always is.)
				if (game_veilFade.y > 0.0 || game_veilFade.w > 0.0) {
					float veilOut = length(max(abs(vPositionW.xz - game_landVeil.xy) - game_landVeil.zw, vec2(0.0)));
					float veilEye = distance(vPositionW, view_position);
					gameVeil = ${veilStrength('glsl', 'veilOut', 'veilEye')} * smoothstep(0.0, game_veilFade.x, veilOut) * smoothstep(0.0, game_veilFade.z, veilEye);
				}
			#endif
		#endif`,
    mix: `
		#ifdef GAME_LOOK
			vec3 gameHazed = mix(fog_color * dBlendModeFogFactor, color, getFogFactor());
			if (gameVeil > 0.0) gameHazed = mix(gameHazed, gameSkyAt(normalize(vPositionW - view_position)) * dBlendModeFogFactor, gameVeil);
			return gameHazed;
		#else
			return mix(fog_color * dBlendModeFogFactor, color, getFogFactor());
		#endif`,
  },
  wgsl: {
    depth: `
		#ifdef GAME_LOOK
			let depth = max(0.0, distance(vPositionW, uniform.view_position) - uniform.game_fogFrom);
		#else
			let depth = pcPosition.z / pcPosition.w;
		#endif`,
    exp2: `
		#ifdef GAME_LOOK
			#ifndef VERTEXSHADER
				// (Nothing to work out without a veil, as the game nearly always is.)
				if (uniform.game_veilFade.y > 0.0 || uniform.game_veilFade.w > 0.0) {
					let veilOut = length(max(abs(vPositionW.xz - uniform.game_landVeil.xy) - uniform.game_landVeil.zw, vec2f(0.0)));
					let veilEye = distance(vPositionW, uniform.view_position);
					gameVeil = ${veilStrength('wgsl', 'veilOut', 'veilEye')} * smoothstep(0.0, uniform.game_veilFade.x, veilOut) * smoothstep(0.0, uniform.game_veilFade.z, veilEye);
				}
			#endif
			return pow(evaluateFogFactorExp2(depth, uniform.fog_density), 2.2);
		#else
			return evaluateFogFactorExp2(depth, uniform.fog_density);
		#endif`,
    mix: `
		#ifdef GAME_LOOK
			var gameHazed = mix(uniform.fog_color * dBlendModeFogFactor, color, getFogFactor());
			if (gameVeil > 0.0) { gameHazed = mix(gameHazed, gameSkyAt(normalize(vPositionW - uniform.view_position)) * dBlendModeFogFactor, gameVeil); }
			return gameHazed;
		#else
			return mix(uniform.fog_color * dBlendModeFogFactor, color, getFogFactor());
		#endif`,
  },
};

/** The game's `fogPS` from the engine's (`engine`, in `lang`). */
export function gameFog(engine: string, lang: Language, replaceOnce: (code: string, name: string, anchor: string, replacement: string) => string): string {
  const a = FOG_ANCHORS[lang];
  const c = FOG_CODE[lang];
  let code = replaceOnce(engine, 'fogPS', a.depth, c.depth);
  code = replaceOnce(code, 'fogPS', a.exp2, c.exp2);
  return FOG_FROM_DECL[lang] + replaceOnce(code, 'fogPS', a.mix, c.mix);
}

/**
 * `fogPS` for what stands wholly in the veil whatever the eye's distance: the curtain closing the landscape's cut edge
 * (`landHaze.ts`). Its colour is the sky's dome behind it (`gameSkyAt`), as the veil takes the land beside it to.
 */
export function skyFog(lang: Language): string {
  return lang === 'glsl'
    ? `float dBlendModeFogFactor = 1.0;
#ifdef VERTEXSHADER
	float getFogFactor(float depth) { return 1.0; }
	vec3 addFog(vec3 color, float depth) { return color; }
#else${skyCode('glsl')}
	float getFogFactor() { return 0.0; }
	vec3 addFog(vec3 color) { return gameSkyAt(normalize(vPositionW - view_position)); }
#endif
`
    : `var<private> dBlendModeFogFactor : f32 = 1.0;
#ifdef VERTEXSHADER
	fn getFogFactor(depth: f32) -> f32 { return 1.0; }
	fn addFog(color: vec3f, depth: f32) -> vec3f { return color; }
#else${skyCode('wgsl')}
	fn getFogFactor() -> f32 { return 0.0; }
	fn addFog(color: vec3f) -> vec3f { return gameSkyAt(normalize(vPositionW - uniform.view_position)); }
#endif
`;
}

/**
 * The sky's dome as the haze reads it (`gameSkyAt`) for `device`: what its colour is made of (`shared/skyMap.ts`
 * `DomeSky`, the way to the sun in PlayCanvas space), as the dome has it (`pc/nature.ts` `Sky`); null: a grey one
 * (the gallery and the previews have no sky, and the veil is never told there). Allocation-free after a device's first.
 */
export function setSky(device: pc.GraphicsDevice, sky: DomeSky | null): void {
  let held = skies.get(device);
  if (!held) skies.set(device, (held = [0, 1, 2, 3].map(() => new Float32Array(3))));
  const parts = sky ? [sky.low, sky.high, sky.sun, sky.glow] : [[0.6, 0.6, 0.6], [0.6, 0.6, 0.6], [0, 1, 0], [0, 0, 0]];
  ['game_skyLow', 'game_skyHigh', 'game_skySun', 'game_skyGlow'].forEach((name, k) => {
    held[k].set(parts[k]);
    device.scope.resolve(name).setValue(held[k]);
  });
}
const skies = new WeakMap<pc.GraphicsDevice, Float32Array[]>();

/** Where the haze begins for `device`'s game materials: `metres` from the eye (0: at the eye, the play camera's). */
export function setFogFrom(device: pc.GraphicsDevice, metres: number): void {
  device.scope.resolve('game_fogFrom').setValue(Math.max(0, metres));
}

/** The haze's start for `device` as last set (`setFogFrom`; 0 before). */
export function fogFrom(device: pc.GraphicsDevice): number {
  return (device.scope.resolve('game_fogFrom').value as number | undefined) ?? 0;
}

/**
 * The menus' land's veil for `device`'s game materials (`shared/landVeil.ts`): clear within `hx` × `hz` m either side
 * of (`cx`, `cz`) (engine space: x and z as the shaders' world positions have them), whole `fade` m past that, at
 * `strength` (0: none, the game's), and whole from `near` m from the eye; whole whatever its strength over the last
 * `edge` m before it is whole (0: not; the curtain closing the landscape's cut edge drawn, `landHaze.ts`).
 * Allocation-free after a device's first.
 */
export function setLandVeil(device: pc.GraphicsDevice, cx: number, cz: number, hx: number, hz: number, fade: number, strength: number, near: number, edge = 0): void {
  const rect = veilRects.get(device) ?? new Float32Array(4);
  const at = veilFades.get(device) ?? new Float32Array(4);
  veilRects.set(device, rect);
  veilFades.set(device, at);
  rect[0] = cx;
  rect[1] = cz;
  rect[2] = hx;
  rect[3] = hz;
  at[0] = Math.max(1, fade);
  at[1] = Math.min(1, Math.max(0, strength));
  at[2] = Math.max(1, near);
  at[3] = Math.max(0, edge);
  device.scope.resolve('game_landVeil').setValue(rect);
  device.scope.resolve('game_veilFade').setValue(at);
}

/**
 * Where the land's veil stands as last told to `device` (`setLandVeil`, engine space: its middle and half its clear
 * rectangle's sides) and how strong it is; null before it was. Allocation-free.
 */
export function landVeilTold(device: pc.GraphicsDevice): { cx: number; cz: number; hx: number; hz: number; strength: number; edge: number } | null {
  const rect = veilRects.get(device);
  const at = veilFades.get(device);
  if (!rect || !at) return null;
  const told = veilTold.get(device) ?? { cx: 0, cz: 0, hx: 0, hz: 0, strength: 0, edge: 0 };
  veilTold.set(device, told);
  [told.cx, told.cz, told.hx, told.hz, told.strength, told.edge] = [rect[0], rect[1], rect[2], rect[3], at[1], at[3]];
  return told;
}
const veilTold = new WeakMap<pc.GraphicsDevice, { cx: number; cz: number; hx: number; hz: number; strength: number; edge: number }>();

/** The arrays the veil's uniforms are set from, a pair a device (`setLandVeil`). */
const veilRects = new WeakMap<pc.GraphicsDevice, Float32Array>();
const veilFades = new WeakMap<pc.GraphicsDevice, Float32Array>();

/**
 * What stands in the veil sinks into the land as it fades (`GAME_VEIL_SINK`, instanced draws: the woods' trees and
 * bushes): each copy scaled down about its foot by the veil where it stands (`setLandVeil`: its x and
 * z, and how far it is from the eye, as `fogPS` has it), gone where the veil is whole. The haze takes a thing and the
 * ground under it alike, so a dark crown on a pale field kept its share of their difference whatever the haze:
 * halfway into the veil the woods stood as dark domes on a ground gone to haze. Not in the shadow pass (the sun's map
 * is drawn again only when something in it changes).
 */
function veilSinkCode(lang: Language): ShaderFeature['glsl'] {
  const glsl = lang === 'glsl';
  const u = (n: string) => (glsl ? n : `uniform.${n}`);
  const [v2, v4, decl] = glsl ? ['vec2', 'vec4', 'float'] : ['vec2f', 'vec4f', 'let'];
  return {
    vsDecl: glsl
      ? '\n#ifdef GAME_VEIL_SINK\n\tuniform vec4 game_landVeil;\n\tuniform vec4 game_veilFade;\n\tuniform vec3 view_position;\n#endif\n'
      : '\n#ifdef GAME_VEIL_SINK\n\tuniform game_landVeil: vec4f;\n\tuniform game_veilFade: vec4f;\n\tuniform view_position: vec3f;\n#endif\n',
    vsLocal: `
#if defined(GAME_VEIL_SINK) && defined(INSTANCING) && !defined(SHADOW_PASS)
	// (Nothing to work out without a veil, as the game nearly always is.)
	if (${u('game_veilFade')}.y > 0.0 || ${u('game_veilFade')}.w > 0.0) {
		${glsl ? 'vec3' : 'let'} sinkAt = (${u('matrix_model')} * ${v4}(instance_line4.xyz, 1.0)).xyz;
		${decl} sinkOut = length(max(abs(sinkAt.xz - ${u('game_landVeil')}.xy) - ${u('game_landVeil')}.zw, ${v2}(0.0)));
		${decl} sinkEye = distance(sinkAt, ${u('view_position')});
		${decl} sinkLeft = 1.0 - ${veilStrength(lang, 'sinkOut', 'sinkEye')} * smoothstep(0.0, ${u('game_veilFade')}.x, sinkOut) * smoothstep(0.0, ${u('game_veilFade')}.z, sinkEye);
		localPos = localPos * (sinkLeft * sinkLeft);
	}
#endif`,
  };
}

/** The veil's sinking (`veilSinkCode`), after the plants' wind (`nature.ts`, 230). */
export const VEIL_SINK: ShaderFeature = { name: 'veilSink', priority: 239, glsl: veilSinkCode('glsl'), wgsl: veilSinkCode('wgsl') };
