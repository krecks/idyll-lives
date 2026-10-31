// Follows Babylon.js 9.30's particle shaders (`particles.vertex.fx`, `particles.fragment.fx`: view-space
// billboards and their blending). Changed for Idyll Lives: one instanced draw with the rings, in GLSL
// and WGSL for PlayCanvas.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
/**
 * Build mode's effects in one instanced quad draw (`buildFx.ts`): its rings lying on the floor and
 * the sparkles and dust facing the camera (billboards in view space, as the original particle
 * shaders draw them).
 *
 * Per instance, in the four `instance_line` attributes `Instances` has (`instancing.ts`, its matrix
 * columns read as data here): line 1 the middle (game space) and the size (m), line 2 the colour
 * (gamma space, and its alpha), line 3 the angle (radians, in the view's plane) and the kind (0 a
 * ring, 1 a sparkle, 2 dust). The quad's corners are its positions' x and y (0..1, also its UVs).
 *
 * The colour goes to linear before the image processing (`pow(c, 2.2)`), and out premultiplied: a
 * sparkle adds its colour times its alpha (its alpha out 0), a ring and dust blend over: one blend
 * (`BLEND_PREMULTIPLIED`) for all three. The textures are sampled outside any branch (WGSL wants
 * them in uniform control flow) and picked by kind.
 *
 * GLSL (WebGL2) and WGSL (WebGPU) side by side.
 */

import type { Language } from './chunks';

export const BUILD_FX_VS: Record<Language, string> = {
  glsl: `
attribute vec3 vertex_position;
attribute vec4 instance_line1;
attribute vec4 instance_line2;
attribute vec4 instance_line3;
uniform mat4 matrix_model;
uniform mat4 matrix_view;
uniform mat4 matrix_projection;
varying vec2 vGameUv;
varying vec4 vGameColor;
varying float vGameKind;
void main(void) {
	vec2 corner = (vertex_position.xy - 0.5) * instance_line1.w;
	float kind = instance_line3.y;
	vec4 viewPos;
	if (kind < 0.5) {
		// A ring: flat on the floor (x across, z deep).
		viewPos = matrix_view * (matrix_model * vec4(instance_line1.xyz + vec3(corner.x, 0.0, corner.y), 1.0));
	} else {
		float c = cos(instance_line3.x);
		float s = sin(instance_line3.x);
		viewPos = matrix_view * (matrix_model * vec4(instance_line1.xyz, 1.0));
		viewPos.xy += vec2(corner.x * c - corner.y * s, corner.x * s + corner.y * c);
	}
	gl_Position = matrix_projection * viewPos;
	vGameUv = vertex_position.xy;
	vGameColor = instance_line2;
	vGameKind = kind;
}
`,
  wgsl: `
attribute vertex_position: vec3f;
attribute instance_line1: vec4f;
attribute instance_line2: vec4f;
attribute instance_line3: vec4f;
uniform matrix_model: mat4x4f;
uniform matrix_view: mat4x4f;
uniform matrix_projection: mat4x4f;
varying vGameUv: vec2f;
varying vGameColor: vec4f;
varying vGameKind: f32;
@vertex
fn vertexMain(input: VertexInput) -> VertexOutput {
	var output: VertexOutput;
	let line1 = input.instance_line1;
	let corner = (input.vertex_position.xy - vec2f(0.5)) * line1.w;
	let kind = input.instance_line3.y;
	var viewPos: vec4f;
	if (kind < 0.5) {
		viewPos = uniform.matrix_view * (uniform.matrix_model * vec4f(line1.xyz + vec3f(corner.x, 0.0, corner.y), 1.0));
	} else {
		let c = cos(input.instance_line3.x);
		let s = sin(input.instance_line3.x);
		viewPos = uniform.matrix_view * (uniform.matrix_model * vec4f(line1.xyz, 1.0));
		viewPos = vec4f(viewPos.xy + vec2f(corner.x * c - corner.y * s, corner.x * s + corner.y * c), viewPos.zw);
	}
	output.position = uniform.matrix_projection * viewPos;
	output.vGameUv = input.vertex_position.xy;
	output.vGameColor = input.instance_line2;
	output.vGameKind = kind;
	return output;
}
`,
};

export const BUILD_FX_PS: Record<Language, string> = {
  glsl: `
#include "gammaPS"
uniform sampler2D gameFxRing;
uniform sampler2D gameFxSparkle;
uniform sampler2D gameFxPuff;
varying vec2 vGameUv;
varying vec4 vGameColor;
varying float vGameKind;
void main(void) {
	vec4 ring = texture2D(gameFxRing, vGameUv);
	vec4 sparkle = texture2D(gameFxSparkle, vGameUv);
	vec4 puff = texture2D(gameFxPuff, vGameUv);
	float kind = vGameKind;
	// A ring is its colour, its texture only its alpha (an opacity texture); a particle its texture times its colour.
	vec4 color = kind < 0.5 ? vec4(vGameColor.rgb, vGameColor.a * ring.a) : vGameColor * (kind < 1.5 ? sparkle : puff);
	vec3 linear = pow(max(color.rgb, vec3(0.0)), vec3(2.2));
	float alpha = clamp(color.a, 0.0, 1.0);
	gl_FragColor = vec4(gammaCorrectOutput(linear) * alpha, kind > 0.5 && kind < 1.5 ? 0.0 : alpha);
}
`,
  wgsl: `
#include "gammaPS"
var gameFxRing: texture_2d<f32>;
var gameFxRingSampler: sampler;
var gameFxSparkle: texture_2d<f32>;
var gameFxSparkleSampler: sampler;
var gameFxPuff: texture_2d<f32>;
var gameFxPuffSampler: sampler;
varying vGameUv: vec2f;
varying vGameColor: vec4f;
varying vGameKind: f32;
@fragment
fn fragmentMain(input: FragmentInput) -> FragmentOutput {
	var output: FragmentOutput;
	let ring = textureSample(gameFxRing, gameFxRingSampler, input.vGameUv);
	let sparkle = textureSample(gameFxSparkle, gameFxSparkleSampler, input.vGameUv);
	let puff = textureSample(gameFxPuff, gameFxPuffSampler, input.vGameUv);
	let kind = input.vGameKind;
	let particle = input.vGameColor * select(puff, sparkle, kind < 1.5);
	let color = select(particle, vec4f(input.vGameColor.rgb, input.vGameColor.a * ring.a), kind < 0.5);
	let linear = pow(max(color.rgb, vec3f(0.0)), vec3f(2.2));
	let alpha = clamp(color.a, 0.0, 1.0);
	output.color = vec4f(gammaCorrectOutput(linear) * alpha, select(alpha, 0.0, kind > 0.5 && kind < 1.5));
	return output;
}
`,
};
