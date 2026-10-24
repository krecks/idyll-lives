/**
 * The wall cutaway and the dug-out ground. Neither applies in the shadow pass: the house casts with
 * its walls at full height in every wall mode, the sun coming in through windows and doorways.
 *
 * - `GAME_WALLCUT`: wall vertices drop in the vertex shader. Per vertex `game_wallCut` (the
 *   `WALL_CUT_SEMANTIC` stream, `Geo`'s `cut`) = [mask A, kind, mask B, 0]: the vertex is cut when
 *   the camera looks along a direction set in both masks (`game_wallCutLook`, the `LOOK` bits of
 *   `shared/geometry.ts`), or all walls are down (`game_wallCutParams.x`); kind 1 clamps it to the
 *   stub height (`.y`, in the mesh's own space: a storey's), kind 2 hides it below the ground
 *   (window frames, lintels). Fragments within `game_wallHoleSpan.x` of the segment `game_wallHole`
 *   (x0, z0, x1, z1), between heights `.y` and `.z`, are dropped (Build mode's preview stands in
 *   for that stretch). Every mesh drawn with such a material must have the attribute (WebGPU
 *   fails a draw without it).
 * - `GAME_DIG`: fragments of the landscape inside the dug-out rectangle (`game_digRect`, x0, z0,
 *   x1, z1) above `game_digSpan.y` are dropped while `game_digSpan.x` is on, for the meshes that
 *   ask for it (`game_digMesh`, a mesh instance parameter over the material's 0: the landscape's
 *   finishes are shared with every model's).
 */

import type { ShaderFeature } from './chunks';

export const CUTAWAY: ShaderFeature = {
  name: 'cutaway',
  priority: 210,
  glsl: {
    vsDecl: `
#ifdef GAME_WALLCUT
	attribute vec4 game_wallCut;
	uniform vec4 game_wallCutLook;
	uniform vec4 game_wallCutParams;
#endif`,
    vsLocal: `
#if defined(GAME_WALLCUT) && !defined(SHADOW_PASS)
	{
		float cutA = step(0.5, dot(floor(fract(game_wallCut.x / vec4(2.0, 4.0, 8.0, 16.0)) * 2.0), game_wallCutLook));
		float cutB = step(0.5, dot(floor(fract(game_wallCut.z / vec4(2.0, 4.0, 8.0, 16.0)) * 2.0), game_wallCutLook));
		if (max(game_wallCutParams.x, cutA * cutB) > 0.5) {
			if (game_wallCut.y > 1.5) { localPos.y = -2.0; }
			else if (game_wallCut.y > 0.5) { localPos.y = min(localPos.y, game_wallCutParams.y); }
		}
	}
#endif`,
    psDecl: `
#ifdef GAME_WALLCUT
	uniform vec4 game_wallHole;
	uniform vec4 game_wallHoleSpan;
#endif
#ifdef GAME_DIG
	uniform vec4 game_digRect;
	uniform vec4 game_digSpan;
	uniform float game_digMesh;
#endif`,
    psStart: `
#if defined(GAME_WALLCUT) && !defined(SHADOW_PASS)
	if (game_wallHoleSpan.x > 0.0) {
		vec3 holeP = gamePos(vPositionW);
		vec2 ab = game_wallHole.zw - game_wallHole.xy;
		float t = dot(holeP.xz - game_wallHole.xy, ab) / max(dot(ab, ab), 1e-6);
		vec2 off = holeP.xz - (game_wallHole.xy + ab * t);
		if (t >= 0.0 && t <= 1.0 && dot(off, off) < game_wallHoleSpan.x * game_wallHoleSpan.x && holeP.y > game_wallHoleSpan.y && holeP.y < game_wallHoleSpan.z) { discard; }
	}
#endif
#if defined(GAME_DIG) && !defined(SHADOW_PASS)
	{
		vec3 digP = gamePos(vPositionW);
		if (game_digMesh > 0.5 && game_digSpan.x > 0.5 && digP.y > game_digSpan.y && digP.x > game_digRect.x && digP.x < game_digRect.z && digP.z > game_digRect.y && digP.z < game_digRect.w) { discard; }
	}
#endif`,
  },
  wgsl: {
    vsDecl: `
#ifdef GAME_WALLCUT
	attribute game_wallCut: vec4f;
	uniform game_wallCutLook: vec4f;
	uniform game_wallCutParams: vec4f;
#endif`,
    vsLocal: `
#if defined(GAME_WALLCUT) && !defined(SHADOW_PASS)
	{
		let cutA = step(0.5, dot(floor(fract(game_wallCut.x / vec4f(2.0, 4.0, 8.0, 16.0)) * 2.0), uniform.game_wallCutLook));
		let cutB = step(0.5, dot(floor(fract(game_wallCut.z / vec4f(2.0, 4.0, 8.0, 16.0)) * 2.0), uniform.game_wallCutLook));
		if (max(uniform.game_wallCutParams.x, cutA * cutB) > 0.5) {
			if (game_wallCut.y > 1.5) { localPos.y = -2.0; }
			else if (game_wallCut.y > 0.5) { localPos.y = min(localPos.y, uniform.game_wallCutParams.y); }
		}
	}
#endif`,
    psDecl: `
#ifdef GAME_WALLCUT
	uniform game_wallHole: vec4f;
	uniform game_wallHoleSpan: vec4f;
#endif
#ifdef GAME_DIG
	uniform game_digRect: vec4f;
	uniform game_digSpan: vec4f;
	uniform game_digMesh: f32;
#endif`,
    psStart: `
#if defined(GAME_WALLCUT) && !defined(SHADOW_PASS)
	if (uniform.game_wallHoleSpan.x > 0.0) {
		let holeP = gamePos(vPositionW);
		let ab: vec2f = uniform.game_wallHole.zw - uniform.game_wallHole.xy;
		let t = dot(holeP.xz - uniform.game_wallHole.xy, ab) / max(dot(ab, ab), 1e-6);
		let off: vec2f = holeP.xz - (uniform.game_wallHole.xy + ab * t);
		if (t >= 0.0 && t <= 1.0 && dot(off, off) < uniform.game_wallHoleSpan.x * uniform.game_wallHoleSpan.x && holeP.y > uniform.game_wallHoleSpan.y && holeP.y < uniform.game_wallHoleSpan.z) { discard; }
	}
#endif
#if defined(GAME_DIG) && !defined(SHADOW_PASS)
	{
		let digP = gamePos(vPositionW);
		if (uniform.game_digMesh > 0.5 && uniform.game_digSpan.x > 0.5 && digP.y > uniform.game_digSpan.y && digP.x > uniform.game_digRect.x && digP.x < uniform.game_digRect.z && digP.z > uniform.game_digRect.y && digP.z < uniform.game_digRect.w) { discard; }
	}
#endif`,
  },
};
