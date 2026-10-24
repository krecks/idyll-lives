/**
 * The residents' shader on PlayCanvas (the port of `CharacterPlugin`, `babylon/characters/material.ts`,
 * and of Babylon's baked-vertex-animation skinning it ran on). Materials: `characterMaterial.ts`.
 *
 * Every resident has a row of the pose texture (`game_simPoses`, RGBA32F, one per app; row layout in
 * `shared/characters/layout.ts`): its bone matrices, then its appearance texels, then its face morph
 * weights. An instanced draw of a part (one per body and part, one copy per resident wearing it) has
 * the row per copy (`game_simRow`, the instance stream beside the matrix), so nothing about a
 * resident is in a material or a define: residents come and go, change clothes and faces without a
 * shader compiling.
 *
 * Vertex (`GAME_RESIDENT`):
 * - Skinned from the row, four influences (`game_simJoints`, `game_simWeights`): the model matrix is
 *   `matrix_model · instance · Σ wᵢ · boneᵢ` (`transformInstancingVS`, set per resident material,
 *   `RESIDENT_INSTANCING`), so the world position (`vPositionW`) and the normal (`mat3` of the model
 *   matrix, PlayCanvas's own for instanced draws) are the skinned ones every later stage reads: the
 *   lamps' rooms, basements, the fog, the diag views.
 * - Rows a pass leaves out (`game_simShown`, 16 bits a component: `residentRowFilter`, all of them but
 *   in a mirror's reflection, which draws only the residents its glass shows) scale to nothing.
 * - Before skinning (`getLocalPosition`, `vsLocal`): the face morphs (`GAME_SIM_MORPH`: the set's
 *   morph texture `game_simMorphs`, its block per vertex in `game_simMorph`, the row's (channel,
 *   weight) pairs), position and normal (`normalCoreVS`'s `getLocalNormal` wrapped,
 *   `RESIDENT_NORMAL`); on shells (`GAME_SIM_CLOTH`) the seated corrective (tone.xyz, as far as
 *   appearance texel 4 b says).
 * - `game_simLayout`: x the first appearance texel (bones · 4), y the morph texture's width, z the
 *   texels of a part's morph header. Uniforms, not constants: one program per surface whatever the set.
 *
 * Fragment, the surfaces as Babylon's plugin drew them (see its file for what each does): the
 * colours in `getAlbedo` (`albedo` slot, by surface: `GAME_SIM_BODY`, `_CLOTH`, `_HAIR`, `_BROW`,
 * `_LASH`, `_EYE`, `_MOUTH`), the roughness in `getGlossiness` (`gloss`: painted clothes rougher,
 * hair matte, polished shoes glossy), the eye's catchlight in `getEmission` (`emissive`). The light
 * itself is the game's look, as for every lit material (`softLook.ts`, `roomLights.ts`,
 * `underground.ts`, `fog.ts`, `diagView.ts`).
 *
 * Derivatives (`fwidth`) are taken in uniform control flow only (WGSL insists): branches are
 * `select`s and `mix`es, as in Babylon's WGSL.
 */

import { APPEARANCE_TEXELS, MORPH_TEXELS } from '../../shared/characters/layout';
import type { Language, ShaderFeature } from './chunks';

/** Microsurface (1 - roughness) of painted clothes; skin keeps the material's. */
const CLOTH_MICRO = '0.15';
/** Hair is matte: no more glossy than this (the sheen band gives it its soft highlight). */
const HAIR_MICRO = '0.38';
/** Polished shoes: glossier than the other shells. */
const POLISH_MICRO = '0.55';
/**
 * Fabric along the painted cuts and a skirt's hem (`simHem`): a turned band per edge, its width and its
 * stitch line's distance from the edge (m on the fields).
 */
const BAND = { cuff: '0.022, 0.012', hem: '0.024, 0.013', rib: '0.016, 0.015', binding: '0.011, 0.007', waistband: '0.034, 0.03', welt: '0.012, 0.008' };
/** What shows in an open jacket's V (texel 4 r = 1 + n + its neck line / 4; outfit.ts `SHIRTS`): 1 white, 2 charcoal, 3 grey (linear). */
const SHIRTS = ['0.78, 0.78, 0.74', '0.036, 0.039, 0.046', '0.357, 0.376, 0.406'];
/** The baked occlusion (tone.w): down to x (1 - AO_DARK) at most, at AO_GAIN per unit; the body's deepen with age (texel 4 g). */
const AO_DARK = '0.8';
const AO_GAIN = '1.5';
const AO_AGE = '0.6';
/** How dark the face's age folds (body tone.z) are on an elder (texel 4 g = 1); younger, less. */
const FOLD = '0.3';
/** The eye's catchlight (linear, emissive). */
const CATCHLIGHT = '0.85';
/** The occlusion's darkening, at `gain` per unit of tone.w. */
const ao = (gain: string): string => `(1.0 - ${AO_DARK} * clamp(${gain} * simT.w, 0.0, 1.0))`;
/** The painted cuts' bands on the body (`simHem`): the top's edges and the bottom's. */
const HEM_TOP = `max(max(simHem((simA.w - simF.x) * 0.55, ${BAND.cuff}), simHem(simF.y - c1, ${BAND.hem})), max(simHem(simF.z - c2, ${BAND.rib}), simHem(simF.w - c3, ${BAND.binding}))) * simPlain`;
const HEM_BOTTOM = `max(simHem(simT4.w - simF.y, ${BAND.waistband}), simHem(simF.y - c5, ${BAND.hem})) * simPlain`;
/** A shoe's laces: across the instep between the eyelets, under the collar. */
const LACE = `(1.0 - smoothstep(0.009 - lW, 0.009 + lW, abs(simF.w))) * (1.0 - smoothstep(0.06, 0.065, simF.x)) * smoothstep(0.01, 0.014, simF.x) * mix(smoothstep(0.28 - lAA, 0.28 + lAA, abs(fract(lBar) - 0.5)), 0.44, smoothstep(0.25, 0.5, lAA))`;
/** The hair's highlight band: sharp along the strands up close, broad where they blur. */
const HAIR_BAND = `mix(pow(hS, 6.0), 0.35 * hS * hS, smoothstep(0.08, 0.25, fwidth(hTH))) * smoothstep(0.0, 0.35, dot(normalW, hHalf)) * mix(0.6, 1.2, hClump)`;
/** The first morph pair texel, after the appearance texels. */
const PAIRS = APPEARANCE_TEXELS;

/** The per-material `transformInstancingVS`: the instance's matrix times the resident's skin. */
export const RESIDENT_INSTANCING: Record<Language, string> = {
  glsl: `
attribute vec4 instance_line1;
attribute vec4 instance_line2;
attribute vec4 instance_line3;
attribute vec4 instance_line4;
mat4 getModelMatrix() {
	return matrix_model * mat4(instance_line1, instance_line2, instance_line3, instance_line4) * gameSimSkin() * gameSimShown();
}
`,
  wgsl: `
attribute instance_line1: vec4f;
attribute instance_line2: vec4f;
attribute instance_line3: vec4f;
attribute instance_line4: vec4f;
fn getModelMatrix() -> mat4x4f {
	return uniform.matrix_model * mat4x4f(instance_line1, instance_line2, instance_line3, instance_line4) * gameSimSkin() * gameSimShown();
}
`,
};

/** `normalCoreVS`'s `getLocalNormal`, wrapped (the face morphs' normal deltas, `gameSimMorphN`). */
export const RESIDENT_NORMAL = {
  glsl: {
    anchor: 'vec3 getLocalNormal(vec3 vertexNormal) {',
    code: `
vec3 getLocalNormal(vec3 vertexNormal) {
	return getLocalNormalEngine(vertexNormal) + gameSimMorphN;
}
`,
  },
  wgsl: {
    anchor: 'fn getLocalNormal(vertexNormal: vec3f) -> vec3f {',
    code: `
fn getLocalNormal(vertexNormal: vec3f) -> vec3f {
	return getLocalNormalEngine(vertexNormal) + gameSimMorphN;
}
`,
  },
} as const;

const VS_GLSL = {
  vsDecl: `
#ifdef GAME_RESIDENT
	attribute vec4 game_simJoints;
	attribute vec4 game_simWeights;
	attribute vec4 game_simField;
	attribute vec4 game_simTone;
	attribute float game_simRow;
	uniform highp sampler2D game_simPoses;
	uniform vec4 game_simLayout;
	varying float vSimRow;
	varying vec4 vSimField;
	varying vec4 vSimTone;
	vec3 gameSimMorphN = vec3(0.0);
	mat4 gameSimBone(int b, int row) {
		int o = b * 4;
		return mat4(texelFetch(game_simPoses, ivec2(o, row), 0), texelFetch(game_simPoses, ivec2(o + 1, row), 0), texelFetch(game_simPoses, ivec2(o + 2, row), 0), texelFetch(game_simPoses, ivec2(o + 3, row), 0));
	}
	mat4 gameSimSkin() {
		int row = int(game_simRow + 0.5);
		ivec4 j = ivec4(game_simJoints + 0.5);
		vec4 w = game_simWeights;
		return gameSimBone(j.x, row) * w.x + gameSimBone(j.y, row) * w.y + gameSimBone(j.z, row) * w.z + gameSimBone(j.w, row) * w.w;
	}
	// The rows drawn (16 bits a component, residentRowFilter): 1, or 0 for a row left out (its copy collapses to nothing).
	uniform vec4 game_simShown;
	float gameSimShown() {
		int row = int(game_simRow + 0.5);
		float bits = floor(game_simShown[row / 16] / exp2(float(row % 16)));
		return bits - 2.0 * floor(bits * 0.5);
	}
	#ifdef GAME_SIM_MORPH
		attribute vec4 game_simMorph;
		uniform highp sampler2D game_simMorphs;
		vec4 gameSimMorphTexel(int i, int width) { return texelFetch(game_simMorphs, ivec2(i % width, i / width), 0); }
	#endif
#endif`,
  vsLocal: `
#ifdef GAME_RESIDENT
	{
		int simRow = int(game_simRow + 0.5);
		int simOff = int(game_simLayout.x + 0.5);
		#ifdef GAME_SIM_MORPH
			// A first channel of -1: no morphs on this face this frame (too small on screen).
			if (game_simMorph.x >= 0.0 && texelFetch(game_simPoses, ivec2(simOff + ${PAIRS}, simRow), 0).x > -0.5) {
				int mW = int(game_simLayout.y + 0.5);
				int mH = int(game_simLayout.z + 0.5);
				int mBase = int(game_simMorph.x + 0.5);
				int mSlot = int(game_simMorph.y + 0.5);
				int mCount = int(game_simMorph.z + 0.5);
				for (int k = 0; k < ${MORPH_TEXELS}; k++) {
					vec4 pw = texelFetch(game_simPoses, ivec2(simOff + ${PAIRS} + k, simRow), 0);
					for (int j = 0; j < 2; j++) {
						float w = j == 1 ? pw.w : pw.y;
						if (abs(w) < 0.0001) continue;
						int ch = int((j == 1 ? pw.z : pw.x) + 0.5);
						float lf = gameSimMorphTexel(mBase + ch / 4, mW)[ch % 4];
						if (lf < -0.5) continue;
						int t = mBase + mH + (int(lf + 0.5) * mCount + mSlot) * 2;
						localPos += gameSimMorphTexel(t, mW).xyz * w;
						gameSimMorphN += gameSimMorphTexel(t + 1, mW).xyz * w;
					}
				}
			}
		#endif
		#ifdef GAME_SIM_CLOTH
			// Seated: the shell's corrective (its tone) as far as the thighs are turned forward (texel 4 b).
			localPos += game_simTone.xyz * texelFetch(game_simPoses, ivec2(simOff + 4, simRow), 0).z;
		#endif
	}
#endif`,
  vsEnd: `
#ifdef GAME_RESIDENT
	vSimRow = game_simRow;
	vSimField = game_simField;
	vSimTone = game_simTone;
#endif`,
};

const VS_WGSL = {
  vsDecl: `
#ifdef GAME_RESIDENT
	attribute game_simJoints: vec4f;
	attribute game_simWeights: vec4f;
	attribute game_simField: vec4f;
	attribute game_simTone: vec4f;
	attribute game_simRow: f32;
	var game_simPoses: texture_2d<uff>;
	uniform game_simLayout: vec4f;
	varying vSimRow: f32;
	varying vSimField: vec4f;
	varying vSimTone: vec4f;
	var<private> gameSimMorphN: vec3f = vec3f(0.0);
	fn gameSimBone(b: i32, row: i32) -> mat4x4f {
		let o = b * 4;
		return mat4x4f(textureLoad(game_simPoses, vec2i(o, row), 0), textureLoad(game_simPoses, vec2i(o + 1, row), 0), textureLoad(game_simPoses, vec2i(o + 2, row), 0), textureLoad(game_simPoses, vec2i(o + 3, row), 0));
	}
	fn gameSimSkin() -> mat4x4f {
		let row = i32(game_simRow + 0.5);
		let j = vec4i(game_simJoints + vec4f(0.5));
		let w = game_simWeights;
		return gameSimBone(j.x, row) * w.x + gameSimBone(j.y, row) * w.y + gameSimBone(j.z, row) * w.z + gameSimBone(j.w, row) * w.w;
	}
	// The rows drawn (16 bits a component, residentRowFilter): 1, or 0 for a row left out (its copy collapses to nothing).
	uniform game_simShown: vec4f;
	fn gameSimShown() -> f32 {
		let row = i32(game_simRow + 0.5);
		let bits = floor(uniform.game_simShown[row / 16] / exp2(f32(row % 16)));
		return bits - 2.0 * floor(bits * 0.5);
	}
	#ifdef GAME_SIM_MORPH
		attribute game_simMorph: vec4f;
		var game_simMorphs: texture_2d<f32>;
		fn gameSimMorphTexel(i: i32, width: i32) -> vec4f { return textureLoad(game_simMorphs, vec2i(i % width, i / width), 0); }
	#endif
#endif`,
  vsLocal: `
#ifdef GAME_RESIDENT
	{
		let simRow = i32(game_simRow + 0.5);
		let simOff = i32(uniform.game_simLayout.x + 0.5);
		#ifdef GAME_SIM_MORPH
			// A first channel of -1: no morphs on this face this frame (too small on screen).
			if (game_simMorph.x >= 0.0 && textureLoad(game_simPoses, vec2i(simOff + ${PAIRS}, simRow), 0).x > -0.5) {
				let mW = i32(uniform.game_simLayout.y + 0.5);
				let mH = i32(uniform.game_simLayout.z + 0.5);
				let mBase = i32(game_simMorph.x + 0.5);
				let mSlot = i32(game_simMorph.y + 0.5);
				let mCount = i32(game_simMorph.z + 0.5);
				for (var k = 0; k < ${MORPH_TEXELS}; k++) {
					let pw = textureLoad(game_simPoses, vec2i(simOff + ${PAIRS} + k, simRow), 0);
					for (var j = 0; j < 2; j++) {
						let w = select(pw.y, pw.w, j == 1);
						if (abs(w) < 0.0001) { continue; }
						let ch = i32(select(pw.x, pw.z, j == 1) + 0.5);
						let lf = gameSimMorphTexel(mBase + ch / 4, mW)[ch % 4];
						if (lf < -0.5) { continue; }
						let t = mBase + mH + (i32(lf + 0.5) * mCount + mSlot) * 2;
						localPos = localPos + gameSimMorphTexel(t, mW).xyz * w;
						gameSimMorphN = gameSimMorphN + gameSimMorphTexel(t + 1, mW).xyz * w;
					}
				}
			}
		#endif
		#ifdef GAME_SIM_CLOTH
			// Seated: the shell's corrective (its tone) as far as the thighs are turned forward (texel 4 b).
			localPos = localPos + game_simTone.xyz * textureLoad(game_simPoses, vec2i(simOff + 4, simRow), 0).z;
		#endif
	}
#endif`,
  vsEnd: `
#ifdef GAME_RESIDENT
	output.vSimRow = game_simRow;
	output.vSimField = game_simField;
	output.vSimTone = game_simTone;
#endif`,
};

const PS_GLSL = {
  psDecl: `
#if defined(GAME_RESIDENT) && !defined(SHADOW_PASS)
	varying float vSimRow;
	varying vec4 vSimField;
	varying vec4 vSimTone;
	uniform highp sampler2D game_simPoses;
	uniform vec4 game_simLayout;
	// Painted clothes' share (the body's roughness), and the eye's catchlight (its emission).
	float gameSimCloth = 0.0;
	float gameSimCatch = 0.0;
	// Appearance texel k of the resident's row.
	vec4 simTexel(int k) { return texelFetch(game_simPoses, ivec2(int(game_simLayout.x + 0.5) + k, int(vSimRow + 0.5)), 0); }
	#if defined(GAME_SIM_BODY) || defined(GAME_SIM_CLOTH)
		// Fabric: a band turned along a cut edge, w m wide, with a stitch line s m in, from d (m inside the
		// edge); x the band, y the stitch. Both fade out as they come under a few pixels (far away) rather than alias.
		vec2 simHem(float d, float w, float s) {
			float fw = max(fwidth(d), 1e-6);
			float band = (1.0 - smoothstep(w - fw, w + fw, d)) * smoothstep(1.5, 4.0, w / fw);
			return vec2(band, band * (1.0 - smoothstep(0.0, fw, abs(d - s) - 0.001)) * min(0.002 / fw, 1.0));
		}
		// The cloth's colour with its bands (h): a little darker, or lighter on dark cloth; the stitch more so.
		vec3 simFabric(vec3 c, vec2 h) {
			float dark = 1.0 - smoothstep(0.015, 0.06, dot(c, vec3(0.2126, 0.7152, 0.0722)));
			vec3 band = mix(c * 0.82, c * 1.4 + vec3(0.006), dark);
			vec3 stitch = mix(c * 0.6, c * 1.9 + vec3(0.02), dark);
			return mix(mix(c, band, h.x), stitch, h.y);
		}
	#endif
	// The resident's colour where the material's is albedo (white, or the teeth's and tongue's).
	vec3 gameSimAlbedo(vec3 albedo, vec3 normalW, vec3 viewDirectionW) {
		vec4 simF = vSimField;
		vec4 simT = vSimTone;
		vec4 simA = simTexel(0);
		#ifdef GAME_SIM_BODY
			// Painted clothes: coverage of the top, bottom and shoes (m, > 0 covered), edges anti-aliased.
			float c1 = simTexel(1).w;
			float c2 = simTexel(2).w;
			float c3 = simTexel(3).w;
			vec4 simT4 = simTexel(4);
			float c5 = simTexel(5).w;
			float c6 = simTexel(6).w;
			float gTop = min(min((simA.w - simF.x) * 0.55, simF.y - c1), min(simF.z - c2, simF.w - c3));
			float gBottom = min(min(simT4.w - simF.y, simF.y - c5), (-0.5 - simF.x) * 0.55);
			float gShoe = c6 - simF.y;
			float simTop = smoothstep(-1.0, 1.0, gTop / max(fwidth(gTop), 1e-5));
			float simBottom = smoothstep(-1.0, 1.0, gBottom / max(fwidth(gBottom), 1e-5));
			float simShoe = smoothstep(-1.0, 1.0, gShoe / max(fwidth(gShoe), 1e-5));
			gameSimCloth = max(max(simTop, simBottom), simShoe);
			// Fabric along the cuts; none on plain clothes (texel 4 r < 0: the towel, swimwear).
			float simPlain = simT4.x < -0.5 ? 0.0 : 1.0;
			vec2 simHemTop = ${HEM_TOP};
			vec2 simHemBottom = ${HEM_BOTTOM};
			vec2 simHemShoe = simHem(gShoe, ${BAND.welt}) * simPlain;
			// Skin: the Sim's colour (relative to the reference), blushed by the tone (x), the line between the lips
			// and the lids' crease darkened (y), and the age folds (z) as far as the Sim's age (texel 4 g).
			vec4 simRef = simTexel(5);
			vec3 skin = simA.rgb * clamp(albedo / max(simRef.rgb, vec3(0.01)), vec3(0.0), vec3(2.0));
			skin = skin * mix(vec3(1.0), vec3(0.98, 0.62, 0.6), clamp(simT.x, 0.0, 1.0)) * (1.0 - 0.45 * clamp(simT.y, 0.0, 1.0)) * (1.0 - ${FOLD} * clamp(simT.z * simT4.y, 0.0, 1.0));
			float scalp = clamp(-simF.z - 1.0, 0.0, 1.0) * simTexel(7).w;
			skin = mix(skin, simTexel(3).rgb * 0.7, scalp);
			vec3 cTop = simTexel(1).rgb;
			vec3 cBottom = simTexel(2).rgb;
			vec3 cShoe = simTexel(6).rgb;
			// An open jacket's shirt (texel 4 r = 1 + n + its neck line / 4) on the torso, from the waist to the
			// neck line and in a collar above it (the jacket shell covers the rest); then the bands.
			float shirtN = floor(simT4.x) - 1.0;
			vec3 shirt = shirtN < 1.5 ? vec3(${SHIRTS[0]}) : shirtN < 2.5 ? vec3(${SHIRTS[1]}) : vec3(${SHIRTS[2]});
			float neckY = fract(simT4.x) * 4.0;
			float inV = max(1.0 - smoothstep(neckY - 0.008, neckY, simF.y), 1.0 - smoothstep(0.01, 0.014, simF.z - c2));
			cTop = simFabric(mix(cTop, shirt, step(0.5, shirtN) * (1.0 - smoothstep(0.1, 0.3, simF.x)) * smoothstep(0.0, 0.01, simF.y - simT4.w) * inV), simHemTop);
			cBottom = simFabric(cBottom, simHemBottom);
			cShoe = simFabric(cShoe, simHemShoe);
			vec3 topOver = mix(mix(skin, cBottom, simBottom), cTop, simTop);
			vec3 topUnder = mix(mix(skin, cTop, simTop), cBottom, simBottom);
			return mix(simT4.x > 0.5 ? topOver : topUnder, cShoe, simShoe) * ${ao(`(${AO_GAIN} + ${AO_AGE} * simT4.y)`)};
		#elif defined(GAME_SIM_CLOTH)
			// Shells: the flat top / bottom / shoe colour (fields.z = category); soles darker; a skirt's or a
			// jacket's hem (fields.x: the distance from it; a collar's is out of reach).
			vec2 simHemShell = simHem(simF.x, ${BAND.hem}) * (abs(simF.y - 3.0) < 0.5 || simF.z < 0.5 ? 1.0 : 0.0);
			vec3 c = simFabric(simF.z < 0.5 ? simTexel(1).rgb : simF.z < 1.5 ? simTexel(2).rgb : simTexel(6).rgb, simHemShell);
			vec3 a = albedo * c * (simF.z > 1.5 && abs(simF.y - 1.0) < 0.5 ? 0.35 : 1.0);
			// Shoes: a light midsole (flag 4), the collar's lining (a negative hem distance) dark, and laces:
			// bars across the instep (fields.w: m off its middle; x: m from the collar), evened out when too small to draw.
			float lBar = simF.x / 0.012;
			float lAA = fwidth(lBar);
			float lW = fwidth(simF.w);
			float lace = ${LACE};
			vec3 laceC = dot(c, vec3(0.333)) < 0.3 ? mix(c, vec3(0.85), 0.7) : c * 0.45;
			bool shoe = simF.z > 1.5;
			if (shoe && abs(simF.y - 4.0) < 0.5) a = vec3(0.8, 0.78, 0.74);
			a = mix(a, c * 0.3, shoe ? 1.0 - smoothstep(-0.002, 0.0, simF.x) : 0.0);
			return mix(a, laceC, shoe ? lace : 0.0) * ${ao(AO_GAIN)};
		#elif defined(GAME_SIM_HAIR)
			// The hair colour in clumps (tone.z: light in a lock's middle), darker deep in (tone.y) and where it is
			// enclosed (tone.w); on dark hair the clumps and a sheen band round the crown lifted towards a lighter
			// tint (Kajiya-Kay from the normal alone: strands straight down the head, a light from above).
			vec3 hc = simTexel(3).rgb;
			float hDark = 1.0 - smoothstep(0.015, 0.1, dot(hc, vec3(0.2126, 0.7152, 0.0722)));
			vec3 hTint = min(hc * 1.7 + vec3(0.075, 0.066, 0.058), vec3(1.0));
			float hClump = mix(0.5, simT.z, 1.0 - smoothstep(0.2, 0.5, fwidth(simT.z)));
			vec3 hDown = normalize(vec3(0.0, 1.0001, 0.0) - normalW * normalW.y);
			vec3 hHalf = normalize(viewDirectionW + vec3(0.0, 1.0, 0.0));
			float hTH = dot(hDown, hHalf);
			float hS = 1.0 - hTH * hTH;
			float hBand = ${HAIR_BAND};
			vec3 hCol = mix(hc * (1.0 + 0.45 * (hClump - 0.5)), hTint, 0.28 * hDark * hClump);
			hCol = mix(hCol, hTint, hBand * mix(0.3, 0.75, hDark));
			return hCol * (1.0 - 0.5 * clamp(simT.y, 0.0, 1.0)) * ${ao(AO_GAIN)};
		#elif defined(GAME_SIM_BROW)
			return simTexel(3).rgb * 0.45;
		#elif defined(GAME_SIM_LASH)
			return vec3(0.03, 0.022, 0.02);
		#elif defined(GAME_SIM_EYE)
			// Sclera, the iris (a darker limbal ring at its edge) and the pupil (fields: offset from the iris
			// centre, iris radius, pupil radius); a catchlight up and to the viewer's left of the pupil.
			float simR = length(simF.xy);
			float simAA = max(fwidth(simR), 1e-5);
			float simC = length(simF.xy - vec2(0.38, 0.42) * simF.z);
			gameSimCatch = 1.0 - smoothstep(0.2 * simF.z - simAA, 0.2 * simF.z + simAA, simC);
			vec3 eyeColour = simTexel(7).rgb;
			if (dot(eyeColour, vec3(1.0)) <= 0.001) eyeColour = vec3(0.24, 0.14, 0.07);
			float iris = 1.0 - smoothstep(simF.z - simAA, simF.z + simAA, simR);
			float pupil = 1.0 - smoothstep(simF.w - simAA, simF.w + simAA, simR);
			float ring = smoothstep(simF.z * 0.62, simF.z, simR);
			vec3 irisColour = eyeColour * (1.0 - 0.55 * ring) * (0.85 + 0.3 * smoothstep(simF.w, simF.z * 0.7, simR));
			return mix(mix(vec3(0.93, 0.91, 0.88), irisColour, iris), vec3(0.02), pupil);
		#elif defined(GAME_SIM_MOUTH)
			// Inside the mouth: the tongue red (tone.x), shaded (tone.y); the mouth plate (tone.y = 1) dark red.
			float shade = clamp(simT.y, 0.0, 1.0);
			vec3 c = mix(albedo, vec3(0.6, 0.17, 0.17), clamp(simT.x, 0.0, 1.0)) * (1.0 - 0.6 * shade);
			return mix(c, vec3(0.09, 0.018, 0.022), smoothstep(0.7, 1.0, shade));
		#else
			return albedo;
		#endif
	}
#endif`,
  albedo: `
#ifdef GAME_RESIDENT
	dAlbedo = gameSimAlbedo(dAlbedo, dVertexNormalW, dViewDirW);
#endif`,
  gloss: `
#ifdef GAME_RESIDENT
	#ifdef GAME_SIM_BODY
		// Painted clothes are rougher than skin.
		dGlossiness = mix(dGlossiness, ${CLOTH_MICRO}, gameSimCloth);
	#endif
	#ifdef GAME_SIM_HAIR
		dGlossiness = min(dGlossiness, ${HAIR_MICRO});
	#endif
	#ifdef GAME_SIM_CLOTH
		// Polished shoes (fields: category 2, flag 5) are glossier.
		dGlossiness = mix(dGlossiness, ${POLISH_MICRO}, step(1.5, vSimField.z) * step(4.5, vSimField.y) * step(vSimField.y, 5.5));
	#endif
#endif`,
  emissive: `
#if defined(GAME_RESIDENT) && defined(GAME_SIM_EYE)
	dEmission += vec3(${CATCHLIGHT}) * gameSimCatch;
#endif`,
};

const PS_WGSL = {
  psDecl: `
#if defined(GAME_RESIDENT) && !defined(SHADOW_PASS)
	varying vSimRow: f32;
	varying vSimField: vec4f;
	varying vSimTone: vec4f;
	var game_simPoses: texture_2d<uff>;
	uniform game_simLayout: vec4f;
	var<private> gameSimCloth: f32 = 0.0;
	var<private> gameSimCatch: f32 = 0.0;
	fn simTexel(k: i32) -> vec4f { return textureLoad(game_simPoses, vec2i(i32(uniform.game_simLayout.x + 0.5) + k, i32(vSimRow + 0.5)), 0); }
	#if defined(GAME_SIM_BODY) || defined(GAME_SIM_CLOTH)
		// See the GLSL: a turned band and its stitch line along a cut edge.
		fn simHem(d: f32, w: f32, s: f32) -> vec2f {
			let fw = max(fwidth(d), 1e-6);
			let band = (1.0 - smoothstep(w - fw, w + fw, d)) * smoothstep(1.5, 4.0, w / fw);
			return vec2f(band, band * (1.0 - smoothstep(0.0, fw, abs(d - s) - 0.001)) * min(0.002 / fw, 1.0));
		}
		fn simFabric(c: vec3f, h: vec2f) -> vec3f {
			let dark = 1.0 - smoothstep(0.015, 0.06, dot(c, vec3f(0.2126, 0.7152, 0.0722)));
			let band = mix(c * 0.82, c * 1.4 + vec3f(0.006), dark);
			let stitch = mix(c * 0.6, c * 1.9 + vec3f(0.02), dark);
			return mix(mix(c, band, h.x), stitch, h.y);
		}
	#endif
	// See the GLSL.
	fn gameSimAlbedo(albedo: vec3f, normalW: vec3f, viewDirectionW: vec3f) -> vec3f {
		let simF = vSimField;
		let simT = vSimTone;
		let simA = simTexel(0);
		#ifdef GAME_SIM_BODY
			let c1 = simTexel(1).w;
			let c2 = simTexel(2).w;
			let c3 = simTexel(3).w;
			let simT4 = simTexel(4);
			let c5 = simTexel(5).w;
			let c6 = simTexel(6).w;
			let gTop = min(min((simA.w - simF.x) * 0.55, simF.y - c1), min(simF.z - c2, simF.w - c3));
			let gBottom = min(min(simT4.w - simF.y, simF.y - c5), (-0.5 - simF.x) * 0.55);
			let gShoe = c6 - simF.y;
			let simTop = smoothstep(-1.0, 1.0, gTop / max(fwidth(gTop), 1e-5));
			let simBottom = smoothstep(-1.0, 1.0, gBottom / max(fwidth(gBottom), 1e-5));
			let simShoe = smoothstep(-1.0, 1.0, gShoe / max(fwidth(gShoe), 1e-5));
			gameSimCloth = max(max(simTop, simBottom), simShoe);
			let simPlain = select(1.0, 0.0, simT4.x < -0.5);
			let simHemTop = ${HEM_TOP};
			let simHemBottom = ${HEM_BOTTOM};
			let simHemShoe = simHem(gShoe, ${BAND.welt}) * simPlain;
			let simRef = simTexel(5);
			var skin = simA.rgb * clamp(albedo / max(simRef.rgb, vec3f(0.01)), vec3f(0.0), vec3f(2.0));
			skin = skin * mix(vec3f(1.0), vec3f(0.98, 0.62, 0.6), clamp(simT.x, 0.0, 1.0)) * (1.0 - 0.45 * clamp(simT.y, 0.0, 1.0)) * (1.0 - ${FOLD} * clamp(simT.z * simT4.y, 0.0, 1.0));
			let scalp = clamp(-simF.z - 1.0, 0.0, 1.0) * simTexel(7).w;
			skin = mix(skin, simTexel(3).rgb * 0.7, scalp);
			var cTop = simTexel(1).rgb;
			var cBottom = simTexel(2).rgb;
			var cShoe = simTexel(6).rgb;
			let shirtN = floor(simT4.x) - 1.0;
			let shirt = select(select(vec3f(${SHIRTS[2]}), vec3f(${SHIRTS[1]}), shirtN < 2.5), vec3f(${SHIRTS[0]}), shirtN < 1.5);
			let neckY = fract(simT4.x) * 4.0;
			let inV = max(1.0 - smoothstep(neckY - 0.008, neckY, simF.y), 1.0 - smoothstep(0.01, 0.014, simF.z - c2));
			cTop = simFabric(mix(cTop, shirt, step(0.5, shirtN) * (1.0 - smoothstep(0.1, 0.3, simF.x)) * smoothstep(0.0, 0.01, simF.y - simT4.w) * inV), simHemTop);
			cBottom = simFabric(cBottom, simHemBottom);
			cShoe = simFabric(cShoe, simHemShoe);
			let topOver = mix(mix(skin, cBottom, simBottom), cTop, simTop);
			let topUnder = mix(mix(skin, cTop, simTop), cBottom, simBottom);
			return mix(select(topUnder, topOver, simT4.x > 0.5), cShoe, simShoe) * ${ao(`(${AO_GAIN} + ${AO_AGE} * simT4.y)`)};
		#elif defined(GAME_SIM_CLOTH)
			let simHemShell = simHem(simF.x, ${BAND.hem}) * select(0.0, 1.0, abs(simF.y - 3.0) < 0.5 || simF.z < 0.5);
			let c = simFabric(select(select(simTexel(6).rgb, simTexel(2).rgb, simF.z < 1.5), simTexel(1).rgb, simF.z < 0.5), simHemShell);
			var a = albedo * c * select(1.0, 0.35, simF.z > 1.5 && abs(simF.y - 1.0) < 0.5);
			let lBar = simF.x / 0.012;
			let lAA = fwidth(lBar);
			let lW = fwidth(simF.w);
			let lace = ${LACE};
			let laceC = select(c * 0.45, mix(c, vec3f(0.85), 0.7), dot(c, vec3f(0.333)) < 0.3);
			let shoe = simF.z > 1.5;
			a = select(a, vec3f(0.8, 0.78, 0.74), shoe && abs(simF.y - 4.0) < 0.5);
			a = mix(a, c * 0.3, select(0.0, 1.0 - smoothstep(-0.002, 0.0, simF.x), shoe));
			return mix(a, laceC, select(0.0, lace, shoe)) * ${ao(AO_GAIN)};
		#elif defined(GAME_SIM_HAIR)
			let hc = simTexel(3).rgb;
			let hDark = 1.0 - smoothstep(0.015, 0.1, dot(hc, vec3f(0.2126, 0.7152, 0.0722)));
			let hTint = min(hc * 1.7 + vec3f(0.075, 0.066, 0.058), vec3f(1.0));
			let hClump = mix(0.5, simT.z, 1.0 - smoothstep(0.2, 0.5, fwidth(simT.z)));
			let hDown = normalize(vec3f(0.0, 1.0001, 0.0) - normalW * normalW.y);
			let hHalf = normalize(viewDirectionW + vec3f(0.0, 1.0, 0.0));
			let hTH = dot(hDown, hHalf);
			let hS = 1.0 - hTH * hTH;
			let hBand = ${HAIR_BAND};
			var hCol = mix(hc * (1.0 + 0.45 * (hClump - 0.5)), hTint, 0.28 * hDark * hClump);
			hCol = mix(hCol, hTint, hBand * mix(0.3, 0.75, hDark));
			return hCol * (1.0 - 0.5 * clamp(simT.y, 0.0, 1.0)) * ${ao(AO_GAIN)};
		#elif defined(GAME_SIM_BROW)
			return simTexel(3).rgb * 0.45;
		#elif defined(GAME_SIM_LASH)
			return vec3f(0.03, 0.022, 0.02);
		#elif defined(GAME_SIM_EYE)
			let simR = length(simF.xy);
			let simAA = max(fwidth(simR), 1e-5);
			let simC = length(simF.xy - vec2f(0.38, 0.42) * simF.z);
			gameSimCatch = 1.0 - smoothstep(0.2 * simF.z - simAA, 0.2 * simF.z + simAA, simC);
			var eyeColour = simTexel(7).rgb;
			eyeColour = select(vec3f(0.24, 0.14, 0.07), eyeColour, dot(eyeColour, vec3f(1.0)) > 0.001);
			let iris = 1.0 - smoothstep(simF.z - simAA, simF.z + simAA, simR);
			let pupil = 1.0 - smoothstep(simF.w - simAA, simF.w + simAA, simR);
			let ring = smoothstep(simF.z * 0.62, simF.z, simR);
			let irisColour = eyeColour * (1.0 - 0.55 * ring) * (0.85 + 0.3 * smoothstep(simF.w, simF.z * 0.7, simR));
			return mix(mix(vec3f(0.93, 0.91, 0.88), irisColour, iris), vec3f(0.02), pupil);
		#elif defined(GAME_SIM_MOUTH)
			let shade = clamp(simT.y, 0.0, 1.0);
			let c = mix(albedo, vec3f(0.6, 0.17, 0.17), clamp(simT.x, 0.0, 1.0)) * (1.0 - 0.6 * shade);
			return mix(c, vec3f(0.09, 0.018, 0.022), smoothstep(0.7, 1.0, shade));
		#else
			return albedo;
		#endif
	}
#endif`,
  albedo: `
#ifdef GAME_RESIDENT
	dAlbedo = gameSimAlbedo(dAlbedo, dVertexNormalW, dViewDirW);
#endif`,
  gloss: `
#ifdef GAME_RESIDENT
	#ifdef GAME_SIM_BODY
		dGlossiness = mix(dGlossiness, ${CLOTH_MICRO}, gameSimCloth);
	#endif
	#ifdef GAME_SIM_HAIR
		dGlossiness = min(dGlossiness, ${HAIR_MICRO});
	#endif
	#ifdef GAME_SIM_CLOTH
		dGlossiness = mix(dGlossiness, ${POLISH_MICRO}, step(1.5, vSimField.z) * step(4.5, vSimField.y) * step(vSimField.y, 5.5));
	#endif
#endif`,
  emissive: `
#if defined(GAME_RESIDENT) && defined(GAME_SIM_EYE)
	dEmission = dEmission + vec3f(${CATCHLIGHT}) * gameSimCatch;
#endif`,
};

/** The residents' feature, at the Babylon plugin's priority (after the cutaway, basements and plain finishes; before the look). */
export const RESIDENT: ShaderFeature = {
  name: 'resident',
  priority: 230,
  glsl: { ...VS_GLSL, ...PS_GLSL },
  wgsl: { ...VS_WGSL, ...PS_WGSL },
};
