/**
 * Buy mode's red see-through copy of a thing that can't go there (`placementGhost.ts`), shaded as
 * Babylon's `StandardMaterial` shaded it (its Blinn-Phong `default.fragment.fx`, without specular):
 * the sun's diffuse term and the sky's hemispheric one, in Babylon's light units and colours (game
 * space), summed on the red, plus the emissive grey, clamped, then times the mesh's vertex colours
 * where it has them (`GAME_GHOST_COLORS`: a model of the game's own parts keeps its colours under a
 * red tint; a file's model without them is the plain red), then taken to linear (`toLinearSpace`:
 * Babylon's image processing ran after, as the game's does). No environment light, no lamps (the
 * game's ghost had them only at night), no fog. Both sides with the front's normal (no two-sided
 * lighting), alpha blended (`gameGhostAlpha.x`: a vec4 written in place, nothing boxed per frame), no depth written.
 *
 * Instanced as the game's draws are (`instancing.ts`: the matrix in game space, in `instance_line1..4`).
 *
 * GLSL (WebGL2) and WGSL (WebGPU) side by side.
 */

import type { Language } from './chunks';

export const GHOST_VS: Record<Language, string> = {
  glsl: `
attribute vec3 vertex_position;
attribute vec3 vertex_normal;
#ifdef GAME_GHOST_COLORS
attribute vec4 vertex_color;
varying vec3 vGameColor;
#endif
attribute vec4 instance_line1;
attribute vec4 instance_line2;
attribute vec4 instance_line3;
attribute vec4 instance_line4;
uniform mat4 matrix_model;
uniform mat4 matrix_viewProjection;
varying vec3 vGameNormal;
void main(void) {
	mat4 instance = mat4(instance_line1, instance_line2, instance_line3, instance_line4);
	gl_Position = matrix_viewProjection * (matrix_model * (instance * vec4(vertex_position, 1.0)));
	vGameNormal = mat3(instance) * vertex_normal;
	#ifdef GAME_GHOST_COLORS
	vGameColor = vertex_color.rgb;
	#endif
}
`,
  wgsl: `
attribute vertex_position: vec3f;
attribute vertex_normal: vec3f;
#ifdef GAME_GHOST_COLORS
attribute vertex_color: vec4f;
varying vGameColor: vec3f;
#endif
attribute instance_line1: vec4f;
attribute instance_line2: vec4f;
attribute instance_line3: vec4f;
attribute instance_line4: vec4f;
uniform matrix_model: mat4x4f;
uniform matrix_viewProjection: mat4x4f;
varying vGameNormal: vec3f;
@vertex
fn vertexMain(input: VertexInput) -> VertexOutput {
	var output: VertexOutput;
	let instance = mat4x4f(input.instance_line1, input.instance_line2, input.instance_line3, input.instance_line4);
	output.position = uniform.matrix_viewProjection * (uniform.matrix_model * (instance * vec4f(input.vertex_position, 1.0)));
	output.vGameNormal = mat3x3f(instance[0].xyz, instance[1].xyz, instance[2].xyz) * input.vertex_normal;
	#ifdef GAME_GHOST_COLORS
	output.vGameColor = input.vertex_color.rgb;
	#endif
	return output;
}
`,
};

export const GHOST_PS: Record<Language, string> = {
  glsl: `
#include "gammaPS"
uniform vec3 gameGhostSunDir;
uniform vec3 gameGhostSun;
uniform vec3 gameGhostHemiDir;
uniform vec3 gameGhostSky;
uniform vec3 gameGhostGround;
uniform vec4 gameGhostAlpha;
varying vec3 vGameNormal;
#ifdef GAME_GHOST_COLORS
varying vec3 vGameColor;
#endif
void main(void) {
	vec3 n = normalize(vGameNormal);
	vec3 light = gameGhostSun * max(0.0, dot(n, gameGhostSunDir)) + mix(gameGhostGround, gameGhostSky, dot(n, gameGhostHemiDir) * 0.5 + 0.5);
	vec3 color = clamp(light * vec3(1.0, 0.45, 0.42) + vec3(0.25), 0.0, 1.0);
	#ifdef GAME_GHOST_COLORS
	color *= vGameColor;
	#endif
	gl_FragColor = vec4(gammaCorrectOutput(pow(color, vec3(2.2))), gameGhostAlpha.x);
}
`,
  wgsl: `
#include "gammaPS"
uniform gameGhostSunDir: vec3f;
uniform gameGhostSun: vec3f;
uniform gameGhostHemiDir: vec3f;
uniform gameGhostSky: vec3f;
uniform gameGhostGround: vec3f;
uniform gameGhostAlpha: vec4f;
varying vGameNormal: vec3f;
#ifdef GAME_GHOST_COLORS
varying vGameColor: vec3f;
#endif
@fragment
fn fragmentMain(input: FragmentInput) -> FragmentOutput {
	var output: FragmentOutput;
	let n = normalize(input.vGameNormal);
	let light = uniform.gameGhostSun * max(0.0, dot(n, uniform.gameGhostSunDir)) + mix(uniform.gameGhostGround, uniform.gameGhostSky, dot(n, uniform.gameGhostHemiDir) * 0.5 + 0.5);
	var color = clamp(light * vec3f(1.0, 0.45, 0.42) + vec3f(0.25), vec3f(0.0), vec3f(1.0));
	#ifdef GAME_GHOST_COLORS
	color = color * input.vGameColor;
	#endif
	output.color = vec4f(gammaCorrectOutput(pow(color, vec3f(2.2))), uniform.gameGhostAlpha.x);
	return output;
}
`,
};

/**
 * The arrow on the tile the thing is used from (`placementGhost.ts`): Babylon's unlit
 * `StandardMaterial`, its emissive colour (`gameArrowColor`: linear, and its alpha), alpha blended.
 * Drawn with `GHOST_VS`.
 */
export const ARROW_PS: Record<Language, string> = {
  glsl: `
#include "gammaPS"
uniform vec4 gameArrowColor;
void main(void) {
	gl_FragColor = vec4(gammaCorrectOutput(gameArrowColor.rgb), gameArrowColor.a);
}
`,
  wgsl: `
#include "gammaPS"
uniform gameArrowColor: vec4f;
@fragment
fn fragmentMain(input: FragmentInput) -> FragmentOutput {
	var output: FragmentOutput;
	output.color = vec4f(gammaCorrectOutput(uniform.gameArrowColor.rgb), uniform.gameArrowColor.a);
	return output;
}
`,
};
