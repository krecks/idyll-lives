/**
 * The soft look on PlayCanvas: PBR's light, drawn as a wide, soft ramp in the light's own colour.
 * The light that falls on the surface (the sun and the lamps, with their cast shadows, and the
 * sky's hemispheric fill) keeps its amount and colour but is redrawn as a wide ramp; the
 * environment around it (the sky's ambient, with its occlusion, and the reflections) stays at a
 * fraction, so shadows stay cool and readable instead of dark. A soft rim, a little more saturation
 * and no outlines finish it. Metals are shaded in their base colour; unlit materials keep their own
 * look.
 *
 * Where it sits: an engine patch (`chunks.ts`) moves the ambient PlayCanvas adds out of the direct
 * light (`gameAmbient`) and adds the sky's fill as direct light (`gameHemi`, a hemispheric light:
 * `mix(ground, sky, n·dir * 0.5 + 0.5)`). `combineColor` then has both apart: the classic look adds
 * them back (plain PBR), the soft look (`GAME_SOFT`) ramps the direct part. It replaces the colour
 * before the emission, fog and tone mapping, so those still apply; among the features it comes
 * after the others (cutaway, basements, plain finishes).
 *
 * It is the game's look (Settings, Performance, Shading: Soft). `classic` draws the PBR shading it
 * replaced, and switching is live: `MaterialLibrary.setShading` sets every lit material's define
 * (its shaders compile again). `?look=soft|classic` on the dev server overrides the setting.
 */

import { settings } from '../../../settings/settings.svelte';
import type { Shading } from '../../types';
import type { ShaderFeature } from './chunks';
import { SPECULAR_UNITS } from '../lightUnits';

const LUM = { glsl: 'vec3(0.2126, 0.7152, 0.0722)', wgsl: 'vec3f(0.2126, 0.7152, 0.0722)' };

/**
 * The diffuse colour of a dielectric as the look was tuned with (up to 0.3.2): the base colour less
 * its reflectance at normal incidence (F0 0.04:
 * `surfaceAlbedo = base * (1 - F0) * (1 - metallic)`), which PlayCanvas doesn't take off. The soft
 * look and the diag view see the base colour so: that, plus the metal's own colour.
 */
export const DIELECTRIC = '0.96';

/** `base`: the colour the soft look ramps and the diag view shows (metals in their own colour). */
export const BASE = {
  glsl: `
	#if defined(LIT_METALNESS) && defined(LIT_SPECULAR_OR_REFLECTION)
		vec3 base = dAlbedo * mix(${DIELECTRIC}, 1.0, litArgs_metalness);
	#else
		vec3 base = dAlbedo * ${DIELECTRIC};
	#endif`,
  wgsl: `
	#if defined(LIT_METALNESS) && defined(LIT_SPECULAR_OR_REFLECTION)
		let base = dAlbedo * mix(${DIELECTRIC}, 1.0, litArgs_metalness);
	#else
		let base = dAlbedo * ${DIELECTRIC};
	#endif`,
};

export const SOFT_LOOK: ShaderFeature = {
  name: 'softLook',
  priority: 300,
  glsl: {
    psDecl: `
#ifdef GAME_LOOK
	uniform vec3 game_hemiSky;
	uniform vec3 game_hemiGround;
	uniform vec3 game_hemiDir;
	vec3 gameAmbient = vec3(0.0);
	vec3 gameHemi(vec3 n) {
		vec3 c = mix(game_hemiGround, game_hemiSky, dot(n, game_hemiDir) * 0.5 + 0.5);
		#ifdef GAME_UNDERGROUND
			c *= gameUnder.y;
		#endif
		return c;
	}
#endif`,
    combine: `
#ifdef GAME_LOOK
	// The AO map occludes the light falling on a surface as well as the sky's (an ARM map's red,
	// with no ambient texture), as the look was tuned; PlayCanvas occludes only the sky's.
	#ifdef STD_AO
		dDiffuseLight *= litArgs_ao;
	#endif
	// The lights' highlights at the look's strength: PlayCanvas's light is the look's / π (its Lambert
	// has no 1 / π, \`lightUnits.ts\` \`SUN_UNITS\`), but its GGX keeps the 1 / π (\`SPECULAR_UNITS\`).
	#ifdef LIT_SPECULAR
		dSpecularLight *= ${SPECULAR_UNITS};
	#endif
	// The look's diffuse colour: the base colour less what its dielectric reflects (DIELECTRIC).
	vec3 gameAlbedo = albedo * ${DIELECTRIC};
	vec3 gameDirect = gameAlbedo * dDiffuseLight;
	#ifdef GAME_UNDERGROUND
		gameAmbient *= gameUnder.z;
		#ifdef LIT_REFLECTIONS
			dReflection.rgb *= gameUnder.z;
		#endif
	#endif
	dDiffuseLight += gameAmbient;
	vec3 pbr = combineColorEngine(gameAlbedo, sheenSpecularity, clearcoatSpecularity);
	#ifdef GAME_SOFT
		// The PBR result, split into the light that falls on the surface and the environment
		// around it: the ramp replaces the first in the light's own colour, the second keeps the
		// scene's ambient shading at a fraction.
		${BASE.glsl}
		vec3 ambient = max(pbr - gameDirect, vec3(0.0));
		vec3 albedoLum = vec3(max(dot(base, ${LUM.glsl}), 0.06));
		vec3 light = gameDirect / albedoLum;
		float lightAmt = dot(light, ${LUM.glsl});
		vec3 lightCol = light / max(lightAmt, 1e-3);
		float lit = smoothstep(0.0, 1.3, lightAmt);
		vec3 shade = mix(vec3(0.72, 0.8, 1.15), vec3(2.3, 2.15, 1.85), lit);
		vec3 soft = ambient * 0.6 + base * shade * lightCol * clamp(lightAmt * 0.42, 0.0, 1.0);
		float fres = pow(1.0 - max(dot(litArgs_worldNormal, dViewDirW), 0.0), 3.0);
		soft *= 1.0 + 0.35 * fres;
		soft = mix(vec3(dot(soft, ${LUM.glsl})), soft, 1.15);
		return max(soft, vec3(0.0));
	#else
		return pbr;
	#endif
#else
	return combineColorEngine(albedo, sheenSpecularity, clearcoatSpecularity);
#endif`,
  },
  wgsl: {
    psDecl: `
#ifdef GAME_LOOK
	uniform game_hemiSky: vec3f;
	uniform game_hemiGround: vec3f;
	uniform game_hemiDir: vec3f;
	var<private> gameAmbient: vec3f = vec3f(0.0);
	fn gameHemi(n: vec3f) -> vec3f {
		var c = mix(uniform.game_hemiGround, uniform.game_hemiSky, dot(n, uniform.game_hemiDir) * 0.5 + 0.5);
		#ifdef GAME_UNDERGROUND
			c = c * gameUnder.y;
		#endif
		return c;
	}
#endif`,
    combine: `
#ifdef GAME_LOOK
	// See the GLSL: the AO map occludes the direct light too.
	#ifdef STD_AO
		dDiffuseLight = dDiffuseLight * litArgs_ao;
	#endif
	// The lights' highlights at the look's strength (see the GLSL).
	#ifdef LIT_SPECULAR
		dSpecularLight = dSpecularLight * ${SPECULAR_UNITS};
	#endif
	let gameAlbedo = albedo * ${DIELECTRIC};
	let gameDirect = gameAlbedo * dDiffuseLight;
	#ifdef GAME_UNDERGROUND
		gameAmbient = gameAmbient * gameUnder.z;
		#ifdef LIT_REFLECTIONS
			dReflection = vec4f(dReflection.rgb * gameUnder.z, dReflection.a);
		#endif
	#endif
	dDiffuseLight = dDiffuseLight + gameAmbient;
	let pbr = combineColorEngine(gameAlbedo, sheenSpecularity, clearcoatSpecularity);
	#ifdef GAME_SOFT
		// See the GLSL: the PBR light is split from the environment, and the ramp replaces the
		// first (in the light's own colour) while the second keeps the ambient shading.
		${BASE.wgsl}
		let ambient = max(pbr - gameDirect, vec3f(0.0));
		let albedoLum = vec3f(max(dot(base, ${LUM.wgsl}), 0.06));
		let light = gameDirect / albedoLum;
		let lightAmt = dot(light, ${LUM.wgsl});
		let lightCol = light / max(lightAmt, 1e-3);
		let lit = smoothstep(0.0, 1.3, lightAmt);
		let shade = mix(vec3f(0.72, 0.8, 1.15), vec3f(2.3, 2.15, 1.85), lit);
		var soft = ambient * 0.6 + base * shade * lightCol * clamp(lightAmt * 0.42, 0.0, 1.0);
		let fres = pow(1.0 - max(dot(litArgs_worldNormal, dViewDirW), 0.0), 3.0);
		soft = soft * (1.0 + 0.35 * fres);
		soft = mix(vec3f(dot(soft, ${LUM.wgsl})), soft, 1.15);
		return max(soft, vec3f(0.0));
	#else
		return pbr;
	#endif
#else
	return combineColorEngine(albedo, sheenSpecularity, clearcoatSpecularity);
#endif`,
  },
};

/** Whether the soft look is drawn: the setting (`settings.shading`), or `?look=` on the dev server. */
export function softLookWanted(): boolean {
  return (softLookFromURL() ?? settings.shading) === 'soft';
}

/** The shading the setting asks for (what `LiveRenderOptions.shading` carries). */
export function shadingFromSettings(): Shading {
  return softLookWanted() ? 'soft' : 'classic';
}

/** `?look=soft|classic|pbr` on the dev server: the look to draw whatever the setting says (read once). */
const LOOKS: Record<string, Shading> = { soft: 'soft', classic: 'classic', pbr: 'classic' };
let askedLook: Shading | null | undefined;
export function softLookFromURL(): Shading | null {
  if (!import.meta.env.DEV) return null;
  if (askedLook === undefined) askedLook = LOOKS[new URLSearchParams(location.search).get('look') ?? ''] ?? null;
  return askedLook;
}
