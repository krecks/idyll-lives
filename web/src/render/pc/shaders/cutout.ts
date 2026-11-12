/**
 * Cut-out finishes (`MaterialEntry.cutout`: a tree's leaf cards), `GAME_CUTOUT`: the opacity the
 * engine's alpha test and alpha to coverage cut them by, made to hold at every distance. In every
 * pass that reads the opacity (the shadow pass too: a card's shadow is its leaves').
 *
 * - No thinning far off. A mipmap averages a cluster's leaves with the clear texels between them, so
 *   its alpha sinks towards the cluster's coverage: a crown seen from afar would test away, or go
 *   see-through under alpha to coverage. The alpha is raised by `CUTOUT_MIP` for each mip level the
 *   texture is drawn at (from the UV's change across a pixel), so about as much of a card stays as
 *   close up.
 * - An edge one pixel wide. The alpha is then stretched about a half by its own change across a pixel
 *   (`fwidth`): a magnified texel's ramp, which alpha to coverage would spread over several pixels as
 *   a soft halo, becomes one pixel of coverage, and the test cuts where the alpha crosses a half
 *   whatever it is set to.
 * - Nothing in the lens. A card is drawn from both sides, so the camera at its closest under a crown
 *   would look through leaves the size of the screen (a lump there was culled, seen from inside): the
 *   alpha falls away from `CUTOUT_CLEAR` m of the eye to none at `CUTOUT_NEAR`, in the forward pass
 *   only (the leaves still shade the ground), by the eye's distance, not the depth (a catalog picture
 *   is drawn without perspective, from afar).
 */

import type { ShaderFeature } from './chunks';

/** How much a cut-out's alpha is raised for each mip level it is drawn at (a share of itself). */
const CUTOUT_MIP = '0.25';
/** A cut-out is gone this near the eye (m), and whole from `CUTOUT_CLEAR` on. */
const CUTOUT_NEAR = '1.2';
const CUTOUT_CLEAR = '2.4';

export const CUTOUT: ShaderFeature = {
  name: 'cutout',
  priority: 150,
  glsl: {
    opacity: `
#if defined(GAME_CUTOUT) && defined(STD_OPACITY_TEXTURE)
	{
		vec2 cutTexel = {STD_OPACITY_TEXTURE_UV} * vec2(textureSize({STD_OPACITY_TEXTURE_NAME}, 0));
		vec2 cutDx = dFdx(cutTexel);
		vec2 cutDy = dFdy(cutTexel);
		float cutLevel = max(0.0, 0.5 * log2(max(dot(cutDx, cutDx), dot(cutDy, cutDy))));
		dAlpha *= 1.0 + ${CUTOUT_MIP} * cutLevel;
		#ifdef FORWARD_PASS
			dAlpha *= smoothstep(${CUTOUT_NEAR}, ${CUTOUT_CLEAR}, distance(view_position, vPositionW));
		#endif
		dAlpha = clamp((dAlpha - 0.5) / max(fwidth(dAlpha), 0.0001) + 0.5, 0.0, 1.0);
	}
#endif`,
  },
  wgsl: {
    opacity: `
#if defined(GAME_CUTOUT) && defined(STD_OPACITY_TEXTURE)
	{
		let cutTexel = {STD_OPACITY_TEXTURE_UV} * vec2f(textureDimensions({STD_OPACITY_TEXTURE_NAME}, 0));
		let cutDx = dpdx(cutTexel);
		let cutDy = dpdy(cutTexel);
		let cutLevel = max(0.0, 0.5 * log2(max(dot(cutDx, cutDx), dot(cutDy, cutDy))));
		dAlpha = dAlpha * (1.0 + ${CUTOUT_MIP} * cutLevel);
		#ifdef FORWARD_PASS
			dAlpha = dAlpha * smoothstep(${CUTOUT_NEAR}, ${CUTOUT_CLEAR}, distance(uniform.view_position, vPositionW));
		#endif
		dAlpha = clamp((dAlpha - 0.5) / max(fwidth(dAlpha), 0.0001) + 0.5, 0.0, 1.0);
	}
#endif`,
  },
};
