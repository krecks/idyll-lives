/**
 * `?diag=albedo` and `?diag=normals` (`debug/benchFlags.ts`) on PlayCanvas: every lit surface drawn
 * as its base colour or its world normal instead of its shading, for screenshots that compare
 * geometry and colour with the lighting left out (also against the game's pictures up to 0.3.2).
 *
 * `endPS` is replaced (`GAME_DIAG_ALBEDO`, `GAME_DIAG_NORMALS`): no fog, no tone mapping, the
 * colour itself. The base colour as the soft look takes it (`BASE`: the base colour less what its
 * dielectric reflects, and metals in their own colour, not PBR's black diffuse), in gamma space;
 * the normal as `n * 0.5 + 0.5` in game space (z mirrored back, `space.ts`), as the earlier
 * pictures have it. It is the last feature, after the others have set the colour. Unlit materials
 * keep their look (the library doesn't give them the define).
 */

import { benchDiag } from '../../../debug/benchFlags';
import type { ShaderFeature } from './chunks';
import { BASE } from './softLook';

/** What the picture shows instead of the shading, if a diag view is asked for (read once). */
export const DIAG_VIEW_MODE: 'albedo' | 'normals' | null = (() => {
  const diag = benchDiag();
  return diag.has('albedo') ? 'albedo' : diag.has('normals') ? 'normals' : null;
})();

/** Whether a diag view replaces the shading (the renderer then leaves tone mapping and the grade out too). */
export const diagViewActive = DIAG_VIEW_MODE !== null;

/** The define a lit material gets for the diag view, if one is asked for. */
export const DIAG_VIEW_DEFINE = DIAG_VIEW_MODE === 'albedo' ? 'GAME_DIAG_ALBEDO' : DIAG_VIEW_MODE === 'normals' ? 'GAME_DIAG_NORMALS' : null;

export const DIAG_VIEW: ShaderFeature = {
  name: 'diagView',
  priority: 1000,
  glsl: {
    end: {
      when: 'defined(GAME_DIAG_ALBEDO) || defined(GAME_DIAG_NORMALS)',
      code: `
	#ifdef GAME_DIAG_NORMALS
		gl_FragColor.rgb = gammaCorrectOutput(pow(gamePos(litArgs_worldNormal) * 0.5 + vec3(0.5), vec3(2.2)));
	#else
		${BASE.glsl}
		gl_FragColor.rgb = gammaCorrectOutput(max(base, vec3(0.0)));
	#endif`,
    },
  },
  wgsl: {
    end: {
      when: 'defined(GAME_DIAG_ALBEDO) || defined(GAME_DIAG_NORMALS)',
      code: `
	#ifdef GAME_DIAG_NORMALS
		output.color = vec4f(gammaCorrectOutput(pow(gamePos(litArgs_worldNormal) * 0.5 + vec3f(0.5), vec3f(2.2))), output.color.a);
	#else
		${BASE.wgsl}
		output.color = vec4f(gammaCorrectOutput(max(base, vec3f(0.0))), output.color.a);
	#endif`,
    },
  },
};
