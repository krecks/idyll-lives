// Follows Babylon.js 9.30's PBR fog (`fogFragment.fx`: the eye's distance, the factor in linear space).
// Changed for Idyll Lives: as two lines of PlayCanvas's `fogPS`.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
/**
 * The haze as the look was tuned with (the PBR fog this follows), in the game's lit materials
 * (`GAME_LOOK`): `fogPS` with two lines changed (anchors also in `tools/render/chunks.json`).
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
 *
 * Other materials (unlit decals, particles) keep PlayCanvas's own.
 */

import type * as pc from '../engine';
import type { Language } from './chunks';

/** The lines of `fogPS` replaced: the fragment's distance, and the exponential-squared factor. */
export const FOG_ANCHORS = {
  glsl: { depth: 'float depth = gl_FragCoord.z / gl_FragCoord.w;', exp2: 'fogFactor = exp(-depth * depth * fog_density * fog_density);' },
  wgsl: { depth: 'let depth = pcPosition.z / pcPosition.w;', exp2: 'return evaluateFogFactorExp2(depth, uniform.fog_density);' },
} as const;

/** The haze's start's declaration, put before the engine's `fogPS` (whose own uniforms are declared inside its `#if`s). */
const FOG_FROM_DECL: Record<Language, string> = {
  glsl: '#ifdef GAME_LOOK\n\tuniform float game_fogFrom;\n\tuniform vec4 game_landVeil;\n\tuniform vec4 game_veilFade;\n#endif\n',
  wgsl: '#ifdef GAME_LOOK\n\tuniform game_fogFrom: f32;\n\tuniform game_landVeil: vec4f;\n\tuniform game_veilFade: vec4f;\n#endif\n',
};

const FOG_CODE: Record<Language, { depth: string; exp2: string }> = {
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
				fogFactor *= 1.0 - game_veilFade.y * smoothstep(0.0, game_veilFade.x, length(max(abs(vPositionW.xz - game_landVeil.xy) - game_landVeil.zw, vec2(0.0)))) * smoothstep(0.0, game_veilFade.z, distance(vPositionW, view_position));
			#endif
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
			#ifdef VERTEXSHADER
				return pow(evaluateFogFactorExp2(depth, uniform.fog_density), 2.2);
			#else
				let veilOut = length(max(abs(vPositionW.xz - uniform.game_landVeil.xy) - uniform.game_landVeil.zw, vec2f(0.0)));
				let veilNear = smoothstep(0.0, uniform.game_veilFade.z, distance(vPositionW, uniform.view_position));
				return pow(evaluateFogFactorExp2(depth, uniform.fog_density), 2.2) * (1.0 - uniform.game_veilFade.y * smoothstep(0.0, uniform.game_veilFade.x, veilOut) * veilNear);
			#endif
		#else
			return evaluateFogFactorExp2(depth, uniform.fog_density);
		#endif`,
  },
};

/** The game's `fogPS` from the engine's (`engine`, in `lang`). */
export function gameFog(engine: string, lang: Language, replaceOnce: (code: string, name: string, anchor: string, replacement: string) => string): string {
  const a = FOG_ANCHORS[lang];
  const c = FOG_CODE[lang];
  return FOG_FROM_DECL[lang] + replaceOnce(replaceOnce(engine, 'fogPS', a.depth, c.depth), 'fogPS', a.exp2, c.exp2);
}

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
 * `strength` (0: none, the game's), and whole from `near` m from the eye. Allocation-free after a device's first.
 */
export function setLandVeil(device: pc.GraphicsDevice, cx: number, cz: number, hx: number, hz: number, fade: number, strength: number, near: number): void {
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
  device.scope.resolve('game_landVeil').setValue(rect);
  device.scope.resolve('game_veilFade').setValue(at);
}

/** The arrays the veil's uniforms are set from, a pair a device (`setLandVeil`). */
const veilRects = new WeakMap<pc.GraphicsDevice, Float32Array>();
const veilFades = new WeakMap<pc.GraphicsDevice, Float32Array>();
