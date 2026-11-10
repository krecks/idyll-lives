/**
 * The residents' shader on PlayCanvas, skinned from a pose texture. Materials:
 * `characterMaterial.ts`.
 *
 * Every resident has a row of the pose texture (`game_simPoses`, RGBA32F, one per app; row layout in
 * `shared/characters/layout.ts`): its bone matrices, then its appearance texels, then its face morph
 * weights. An instanced draw of a part (one per body and part, one copy per resident wearing it) has
 * the row per copy (`game_simRow`, the instance stream beside the matrix), so nothing about a
 * resident is in a material or a define: residents come and go, change clothes and faces without a
 * shader compiling.
 *
 * Vertex (`GAME_RESIDENT`):
 * - Skinned from the row, four influences (`game_simJoints`, `game_simWeights`; a bone three texels, its
 *   matrix's affine rows, and one of no weight not read): the model matrix is
 *   `matrix_model · instance · Σ wᵢ · boneᵢ` (`transformInstancingVS`, set per resident material,
 *   `RESIDENT_INSTANCING`), so the world position (`vPositionW`) and the normal (`mat3` of the model
 *   matrix, PlayCanvas's own for instanced draws) are the skinned ones everything after reads: the
 *   lamps' rooms, basements, the fog, the diag views.
 * - Rows a pass leaves out (`game_simShown`, 16 bits a component: `residentRowFilter`, all of them but
 *   in a mirror's reflection, which draws only the residents its glass shows) scale to nothing. It covers
 *   the first 64 rows; the rows past them (the town's servers in a full town) always draw.
 * - Before skinning (`getLocalPosition`, `vsLocal`): the face morphs (`GAME_SIM_MORPH`: the set's
 *   morph texture `game_simMorphs`, its block per vertex in `game_simMorph`, the row's (channel,
 *   weight) pairs), position and normal (`normalCoreVS`'s `getLocalNormal` wrapped,
 *   `RESIDENT_NORMAL`); on shells (`GAME_SIM_CLOTH`) the seated corrective (tone.xyz, as far as
 *   appearance texel 4 b says).
 * - `game_simLayout`: x the first appearance texel (bones · `BONE_TEXELS`), y the morph texture's width, z the
 *   texels of a part's morph header. Uniforms, not constants: one program per surface whatever the set.
 *
 * Fragment, by surface: the colours in `getAlbedo` (`albedo` slot, by surface: `GAME_SIM_BODY`,
 * `_CLOTH`, `_HAIR`, `_BROW`, `_LASH`, `_EYE`, `_MOUTH`), the roughness in `getGlossiness`
 * (`gloss`: painted clothes rougher, hair matte, polished shoes glossy), the eye's catchlight in
 * `getEmission` (`emissive`). The light itself is the game's look, as for every lit material
 * (`softLook.ts`, `roomLights.ts`, `underground.ts`, `fog.ts`, `diagView.ts`).
 *
 * Derivatives (`fwidth`) are taken in uniform control flow only (WGSL insists): branches are
 * `select`s and `mix`es.
 */

import { APPEARANCE_TEXELS, BONE_TEXELS, MORPH_TEXELS, SHIRT } from '../../shared/characters/layout';
import type { Language, ShaderFeature } from './chunks';

/** Microsurface (1 - roughness) of painted clothes; skin keeps the material's. */
const CLOTH_MICRO = '0.15';
/** Hair is matte: no more glossy than this (the sheen band gives it its soft highlight). */
const HAIR_MICRO = '0.38';
/** Polished shoes: glossier than the other shells. */
const POLISH_MICRO = '0.55';
/**
 * The lips (body tone.y < 0): how much of the lip's colour over the skin's, and how matte they are
 * (skin is glossier, and a lip's tube catches the key light along its whole length when it isn't).
 */
const LIP_MIX = '0.9';
const LIP_SHADE = '0.3';
const LIP_MICRO = '0.25';
/**
 * Fabric along the painted cuts and a skirt's hem (`simHem`): a turned band per edge, its width and its
 * stitch line's distance from the edge (m on the fields).
 */
const BAND = { cuff: '0.022, 0.012', hem: '0.024, 0.013', rib: '0.016, 0.015', binding: '0.011, 0.007', waistband: '0.034, 0.03', welt: '0.012, 0.008' };
/** The shirts' colours (texel 4 r = 1 + n + its neck line / 4; layout.ts `SHIRT` n): white, charcoal, grey (linear). */
const SHIRTS = ['0.78, 0.78, 0.74', '0.036, 0.039, 0.046', '0.357, 0.376, 0.406'];
/** Whether shirt `shirtN` is white: a suit's, a jacket's white tee, or an apron's. */
const SHIRT_WHITE = `(shirtN < ${SHIRT.white + 0.5} || shirtN > ${SHIRT.apron - 0.5})`;
/** How much of the top is the shirt: an open jacket's V from the waist to its neck line (`inV`), an apron's all of it. */
const SHIRT_MIX = `step(0.5, shirtN) * max(step(${SHIRT.apron - 0.5}, shirtN), (1.0 - smoothstep(0.1, 0.3, simF.x)) * smoothstep(0.0, 0.01, simF.y - simT4.w) * inV)`;
/** The baked occlusion (tone.w): down to x (1 - AO_DARK) at most, at AO_GAIN per unit; the body's deepen with age (texel 4 g). */
const AO_DARK = '0.8';
const AO_GAIN = '1.5';
const AO_AGE = '0.6';
/** How dark the face's age folds (body tone.z) are on an elder (texel 4 g = 1); younger, less. */
const FOLD = '0.3';
/**
 * How far (m) the shadow-only copies (`GAME_SIM_CASTER`: the low-detail body and shells every resident casts its sun
 * shadow from) are drawn in along their normals, the hands left as they are (thinner than it). The low-detail surface
 * stands up to about a centimetre outside the full one the camera sees (the neck, the chest, the face: the cage's
 * limit against the drawn head and body), so the full body lay behind its own caster there and took its shadow in
 * stripes (acne) wherever the map's texels were small: the creator, a close camera, the pages.
 */
const CASTER_INSET = '0.012';
/**
 * The eye's catchlight (linear, emissive): the light's reflection, so as bright as the light on the eye
 * (`RESIDENT_LIGHT`; at its full strength whatever the light, it glowed at night).
 */
const CATCHLIGHT = '0.85';
/**
 * Skin lets light through: the light of a lamp or the sun comes round the terminator (`SKIN_WRAP`: how far past
 * it, as a share of the way round; `RESIDENT_DIFFUSE`), and what came through is red (`SKIN_SCATTER`, its
 * colour; `RESIDENT_LIGHT`). Its highlight is a weak one (`SKIN_SPECULAR` of a plastic's), weaker in a crease.
 * The soft look's extra saturation (`softLook.ts` `gameSaturation`) is left off skin (`SKIN_SATURATION`): on a
 * tone already warm it turned every face orange.
 */
const SKIN_SATURATION = '0.88';
const SKIN_WRAP = '0.3';
const SKIN_SCATTER = '1.0, 0.52, 0.4';
const SKIN_SPECULAR = '0.55';
/**
 * The white of the eye (linear; a real one is far from white, and in the lids' shade), how far the upper lid
 * shades the top of the eye and how far the ball darkens towards its corners as it turns away.
 */
const SCLERA = '0.8, 0.775, 0.75';
const LID_SHADE = '0.45';
const EYE_ROUND = '0.3';
/** The occlusion's darkening, at `gain` per unit of tone.w. */
const ao = (gain: string): string => `(1.0 - ${AO_DARK} * clamp(${gain} * simT.w, 0.0, 1.0))`;
/** The painted cuts' bands on the body (`simHem`): the top's edges and the bottom's. */
const HEM_TOP = `max(max(simHem((simA.w - simF.x) * 0.55, ${BAND.cuff}), simHem(simF.y - c1, ${BAND.hem})), max(simHem(simF.z - c2, ${BAND.rib}), simHem(simF.w - c3, ${BAND.binding}))) * simPlain`;
const HEM_BOTTOM = `max(simHem(simT4.w - simF.y, ${BAND.waistband}), simHem(simF.y - c5, ${BAND.hem})) * simPlain`;
/** A shoe's laces: across the instep between the eyelets, under the collar. */
const LACE = `(1.0 - smoothstep(0.009 - lW, 0.009 + lW, abs(simF.w))) * (1.0 - smoothstep(0.06, 0.065, simF.x)) * smoothstep(0.01, 0.014, simF.x) * mix(smoothstep(0.28 - lAA, 0.28 + lAA, abs(fract(lBar) - 0.5)), 0.44, smoothstep(0.25, 0.5, lAA))`;
/** The hair's highlight band: sharp along the strands up close, broad where they blur. */
const HAIR_BAND = `mix(pow(hS, 6.0), 0.35 * hS * hS, smoothstep(0.08, 0.25, fwidth(hTH))) * smoothstep(0.0, 0.35, dot(normalW, hHalf)) * mix(0.6, 1.2, hClump)`;
/**
 * Cloth stands off the body it is painted on (the vertex shader, `GAME_SIM_BODY`): the covered skin is drawn out
 * along its normal, as far as the resident's garments are loose (appearance texel 8: the top's and the bottom's,
 * 0 for what clings or lies under a shell), so a tee hangs from the chest past the waist, a trouser leg falls
 * nearly straight from the knee over the shoe, and a cuff or a hem stands a step off the skin. Metres on the base
 * body by its fields' heights, scaled to the life stage: `thick` everywhere, `hang` more from `chest` down to
 * `waist` (the top), `seat` more below the hips and `leg` more from `knee` down to `ankle` (the bottom), `sleeve`
 * on the arms; within `neck` m of the neckline it thins out, under the collars' shells.
 */
const EASE = { thick: '0.004', hang: '0.016', chest: '1.30', waist: '1.12', sleeve: '0.007', neck: '0.06', hips: '0.88', thigh: '0.6', seat: '0.006', knee: '0.5', ankle: '0.12', leg: '0.016' };
/** How dark the skin (or the cloth under it) is just under a loose garment's edge (the step's shade), and how far (m). */
const EDGE_SHADE = '0.3';
const EDGE_REACH = '0.018';
/** The ease's code for a language's vertex shader: `f` the fields, `tx(k)` appearance texel k. */
function easeCode(lang: Language): string {
  const w = lang === 'wgsl';
  const let_ = (name: string, type: string) => (w ? `let ${name}` : `${type} ${name}`);
  const tx = (k: number) => (w ? `textureLoad(game_simPoses, vec2i(simOff + ${k}, simRow), 0)` : `texelFetch(game_simPoses, ivec2(simOff + ${k}, simRow), 0)`);
  const add = (v: string) => (w ? `localPos = localPos + ${v};` : `localPos += ${v};`);
  return `
		#if defined(GAME_SIM_BODY) && !defined(SHADOW_PASS)
		{
			${let_('simEase', 'vec4')} = ${tx(8)};
			if (simEase.x + simEase.y > 0.0) {
				${let_('simF', 'vec4')} = game_simField;
				${let_('simT4', 'vec4')} = ${tx(4)};
				${let_('simNeck', 'float')} = min(simF.z - ${tx(2)}.w, simF.w - ${tx(3)}.w);
				${let_('simTopIn', 'float')} = min(min((${tx(0)}.w - simF.x) * 0.55, simF.y - ${tx(1)}.w), simNeck);
				${let_('simBottomIn', 'float')} = min(min(simT4.w - simF.y, simF.y - ${tx(5)}.w), (-0.5 - simF.x) * 0.55);
				${let_('simArm', 'float')} = clamp(simF.x + 1.0, 0.0, 1.0);
				${let_('simHang', 'float')} = 1.0 - smoothstep(${EASE.waist}, ${EASE.chest}, simF.y);
				${let_('simTopEase', 'float')} = mix(${EASE.thick} + ${EASE.hang} * simHang, ${EASE.sleeve}, simArm) * (0.25 + 0.75 * smoothstep(0.0, ${EASE.neck}, simNeck)) * smoothstep(-0.004, 0.004, simTopIn) * simEase.x * (1.0 - simEase.z * (1.0 - smoothstep(simT4.w, simT4.w + 0.04, simF.y)));
				${let_('simBottomEase', 'float')} = (${EASE.thick} + ${EASE.seat} * (1.0 - smoothstep(${EASE.thigh}, ${EASE.hips}, simF.y)) + ${EASE.leg} * (1.0 - smoothstep(${EASE.ankle}, ${EASE.knee}, simF.y))) * smoothstep(-0.004, 0.004, simBottomIn) * simEase.y;
				// (The life stage's size: the vertex's height over its height on the base body, the fields'.)
				${let_('simSize', 'float')} = clamp(localPos.y / max(simF.y, 0.05), 0.3, 1.2);
				${add('normalize(vertex_normal) * (max(simTopEase, simBottomEase) * simSize)')}
			}
		}
		#endif`;
}

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

/**
 * The body's `lightDiffuseLambertPS` (per material): a light's diffuse share on the skin wrapped round the
 * terminator (`SKIN_WRAP`), Lambert's on painted clothes; what it added is kept for `RESIDENT_LIGHT`. The lights'
 * shadows still fall on it, so the skin only softens where the body turns from the light.
 */
export const RESIDENT_DIFFUSE: Record<Language, string> = {
  glsl: `
float getLightDiffuse(vec3 worldNormal, vec3 viewDir, vec3 lightDirNorm) {
	float nl = dot(worldNormal, -lightDirNorm);
	float lambert = max(nl, 0.0);
	#if defined(GAME_RESIDENT) && !defined(SHADOW_PASS)
		float skin = 1.0 - gameSimCloth;
		float wrapped = mix(lambert, max((nl + ${SKIN_WRAP}) / (1.0 + ${SKIN_WRAP}), 0.0), skin);
		gameSimWrapped += wrapped;
		gameSimWrap += wrapped - lambert;
		return wrapped;
	#else
		return lambert;
	#endif
}
`,
  wgsl: `
fn getLightDiffuse(worldNormal: vec3f, viewDir: vec3f, lightDirNorm: vec3f) -> f32 {
	let nl = dot(worldNormal, -lightDirNorm);
	let lambert = max(nl, 0.0);
	#if defined(GAME_RESIDENT) && !defined(SHADOW_PASS)
		let skin = 1.0 - gameSimCloth;
		let wrapped = mix(lambert, max((nl + ${SKIN_WRAP}) / (1.0 + ${SKIN_WRAP}), 0.0), skin);
		gameSimWrapped = gameSimWrapped + wrapped;
		gameSimWrap = gameSimWrap + wrapped - lambert;
		return wrapped;
	#else
		return lambert;
	#endif
}
`,
};

/**
 * The residents' part of the look's `combineColor` (`softLook.ts`, before it splits the light from the
 * environment, so in both looks): the skin's light that came round the terminator turned red and its highlight
 * weakened, and the eye's catchlight as bright as the light on the eye.
 */
export const RESIDENT_LIGHT: Record<Language, string> = {
  glsl: `
#if defined(GAME_RESIDENT) && !defined(SHADOW_PASS)
	#ifdef GAME_SIM_BODY
		float simSkin = 1.0 - gameSimCloth;
		gameSaturation = mix(gameSaturation, ${SKIN_SATURATION}, simSkin);
		dDiffuseLight *= mix(vec3(1.0), vec3(${SKIN_SCATTER}), simSkin * gameSimWrap / max(gameSimWrapped, 1e-4));
		#ifdef LIT_SPECULAR
			dSpecularLight *= mix(1.0, ${SKIN_SPECULAR} * gameSimOcc, simSkin);
		#endif
	#endif
	#ifdef GAME_SIM_EYE
		litArgs_emission *= clamp(dot(dDiffuseLight + gameAmbient, vec3(0.2126, 0.7152, 0.0722)), 0.05, 1.0);
	#endif
#endif`,
  wgsl: `
#if defined(GAME_RESIDENT) && !defined(SHADOW_PASS)
	#ifdef GAME_SIM_BODY
		let simSkin = 1.0 - gameSimCloth;
		gameSaturation = mix(gameSaturation, ${SKIN_SATURATION}, simSkin);
		dDiffuseLight = dDiffuseLight * mix(vec3f(1.0), vec3f(${SKIN_SCATTER}), simSkin * gameSimWrap / max(gameSimWrapped, 1e-4));
		#ifdef LIT_SPECULAR
			dSpecularLight = dSpecularLight * mix(1.0, ${SKIN_SPECULAR} * gameSimOcc, simSkin);
		#endif
	#endif
	#ifdef GAME_SIM_EYE
		litArgs_emission = litArgs_emission * clamp(dot(dDiffuseLight + gameAmbient, vec3f(0.2126, 0.7152, 0.0722)), 0.05, 1.0);
	#endif
#endif`,
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
	#if defined(GAME_SIM_BODY) && !defined(SHADOW_PASS)
		// (The cloth's ease reads the normal in getLocalPosition, before normalCoreVS declares it: RESIDENT_NORMAL leaves it out there.)
		attribute vec3 vertex_normal;
	#endif
	// Bone b's skinning matrix's three rows (a texel each, layout.ts BONE_TEXELS), weighted by w, added to r0..r2.
	void gameSimBone(int b, int row, float w, inout vec4 r0, inout vec4 r1, inout vec4 r2) {
		int o = b * ${BONE_TEXELS};
		r0 += texelFetch(game_simPoses, ivec2(o, row), 0) * w;
		r1 += texelFetch(game_simPoses, ivec2(o + 1, row), 0) * w;
		r2 += texelFetch(game_simPoses, ivec2(o + 2, row), 0) * w;
	}
	mat4 gameSimSkin() {
		int row = int(game_simRow + 0.5);
		ivec4 j = ivec4(game_simJoints + 0.5);
		vec4 w = game_simWeights;
		vec4 r0 = vec4(0.0);
		vec4 r1 = vec4(0.0);
		vec4 r2 = vec4(0.0);
		// (A bone of no weight is not read: most vertices have one or two.)
		gameSimBone(j.x, row, w.x, r0, r1, r2);
		if (w.y > 0.0) gameSimBone(j.y, row, w.y, r0, r1, r2);
		if (w.z > 0.0) gameSimBone(j.z, row, w.z, r0, r1, r2);
		if (w.w > 0.0) gameSimBone(j.w, row, w.w, r0, r1, r2);
		// (Columns from the rows; the fourth row the weights' sum, as the four matrices' 0 0 0 1 rows weighted.)
		return mat4(vec4(r0.x, r1.x, r2.x, 0.0), vec4(r0.y, r1.y, r2.y, 0.0), vec4(r0.z, r1.z, r2.z, 0.0), vec4(r0.w, r1.w, r2.w, w.x + w.y + w.z + w.w));
	}
	// The rows drawn (16 bits a component, residentRowFilter): 1, or 0 for a row left out (its copy collapses to nothing).
	uniform vec4 game_simShown;
	float gameSimShown() {
		int row = int(game_simRow + 0.5);
		// (The filter covers the first 64 rows, the residents'; rows past them, servers in a full town, always draw.)
		if (row >= 64) return 1.0;
		float bits = floor(game_simShown[row / 16] / exp2(float(row % 16)));
		return bits - 2.0 * floor(bits * 0.5);
	}
	#ifdef GAME_SIM_MORPH
		attribute vec4 game_simMorph;
		uniform highp sampler2D game_simMorphs;
		vec4 gameSimMorphTexel(int i, int width) { return texelFetch(game_simMorphs, ivec2(i % width, i / width), 0); }
	#endif
	#if defined(GAME_SIM_CASTER) && defined(SHADOW_PASS)
		// (The mesh's normals: the shadow pass declares none of its own.)
		attribute vec3 game_simCasterN;
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
${easeCode('glsl')}
		#ifdef GAME_SIM_CLOTH
			// Seated: the shell's corrective (its tone) as far as the thighs are turned forward (texel 4 b).
			localPos += game_simTone.xyz * texelFetch(game_simPoses, ivec2(simOff + 4, simRow), 0).z;
		#endif
		#if defined(GAME_SIM_CASTER) && defined(SHADOW_PASS)
			// The shadow caster drawn in (\`CASTER_INSET\`), not the hands (the body's fields.x past the wrist).
			#ifdef GAME_SIM_BODY
				float simInset = ${CASTER_INSET} * (1.0 - smoothstep(0.85, 1.0, game_simField.x));
			#else
				float simInset = ${CASTER_INSET};
			#endif
			localPos -= game_simCasterN * simInset;
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
	// Bone b's skinning matrix's three rows (a texel each, layout.ts BONE_TEXELS), weighted by w, added to r.
	fn gameSimBone(b: i32, row: i32, w: f32, r: ptr<function, array<vec4f, 3>>) {
		let o = b * ${BONE_TEXELS};
		(*r)[0] = (*r)[0] + textureLoad(game_simPoses, vec2i(o, row), 0) * w;
		(*r)[1] = (*r)[1] + textureLoad(game_simPoses, vec2i(o + 1, row), 0) * w;
		(*r)[2] = (*r)[2] + textureLoad(game_simPoses, vec2i(o + 2, row), 0) * w;
	}
	fn gameSimSkin() -> mat4x4f {
		let row = i32(game_simRow + 0.5);
		let j = vec4i(game_simJoints + vec4f(0.5));
		let w = game_simWeights;
		var r = array<vec4f, 3>(vec4f(0.0), vec4f(0.0), vec4f(0.0));
		// (A bone of no weight is not read: most vertices have one or two.)
		gameSimBone(j.x, row, w.x, &r);
		if (w.y > 0.0) { gameSimBone(j.y, row, w.y, &r); }
		if (w.z > 0.0) { gameSimBone(j.z, row, w.z, &r); }
		if (w.w > 0.0) { gameSimBone(j.w, row, w.w, &r); }
		// (Columns from the rows; the fourth row the weights' sum, as the four matrices' 0 0 0 1 rows weighted.)
		return mat4x4f(vec4f(r[0].x, r[1].x, r[2].x, 0.0), vec4f(r[0].y, r[1].y, r[2].y, 0.0), vec4f(r[0].z, r[1].z, r[2].z, 0.0), vec4f(r[0].w, r[1].w, r[2].w, w.x + w.y + w.z + w.w));
	}
	// The rows drawn (16 bits a component, residentRowFilter): 1, or 0 for a row left out (its copy collapses to nothing).
	uniform game_simShown: vec4f;
	fn gameSimShown() -> f32 {
		let row = i32(game_simRow + 0.5);
		// (The filter covers the first 64 rows, the residents'; rows past them, servers in a full town, always draw.)
		if (row >= 64) { return 1.0; }
		let bits = floor(uniform.game_simShown[min(row, 63) / 16] / exp2(f32(row % 16)));
		return bits - 2.0 * floor(bits * 0.5);
	}
	#ifdef GAME_SIM_MORPH
		attribute game_simMorph: vec4f;
		var game_simMorphs: texture_2d<f32>;
		fn gameSimMorphTexel(i: i32, width: i32) -> vec4f { return textureLoad(game_simMorphs, vec2i(i % width, i / width), 0); }
	#endif
	#if defined(GAME_SIM_CASTER) && defined(SHADOW_PASS)
		attribute game_simCasterN: vec3f;
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
${easeCode('wgsl')}
		#ifdef GAME_SIM_CLOTH
			// Seated: the shell's corrective (its tone) as far as the thighs are turned forward (texel 4 b).
			localPos = localPos + game_simTone.xyz * textureLoad(game_simPoses, vec2i(simOff + 4, simRow), 0).z;
		#endif
		#if defined(GAME_SIM_CASTER) && defined(SHADOW_PASS)
			#ifdef GAME_SIM_BODY
				let simInset = ${CASTER_INSET} * (1.0 - smoothstep(0.85, 1.0, game_simField.x));
			#else
				let simInset = ${CASTER_INSET};
			#endif
			localPos = localPos - game_simCasterN * simInset;
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
	// Painted clothes' share (the body's roughness), the lips' (theirs), and the eye's catchlight (its emission).
	float gameSimCloth = 0.0;
	float gameSimLip = 0.0;
	float gameSimCatch = 0.0;
	// The baked occlusion's shade (the skin's highlight goes with it), and the lights' diffuse on the skin: all
	// of it, and how much of it came round the terminator (RESIDENT_DIFFUSE).
	float gameSimOcc = 1.0;
	float gameSimWrapped = 0.0;
	float gameSimWrap = 0.0;
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
			// Skin: the Sim's colour (relative to the reference), blushed by the tone (x), the lids' crease and
			// the line between the lips darkened (y > 0), the lips themselves tinted (y < 0: how much of the lip),
			// and the age folds (z) as far as the Sim's age (texel 4 g); on the lips z is how far they turn down
			// at rest (baked, so the shade under them is the lip's shape whichever way the head faces).
			vec4 simRef = simTexel(5);
			vec3 skin = simA.rgb * clamp(albedo / max(simRef.rgb, vec3(0.01)), vec3(0.0), vec3(2.0));
			skin = skin * mix(vec3(1.0), vec3(0.98, 0.62, 0.6), clamp(simT.x, 0.0, 1.0));
			// The lips: a red-rose tint that reads on a pale face and on a dark one alike (the skin darkened and
			// reddened, then mixed towards a rose), and a shade under them where they turn down.
			gameSimLip = clamp(-simT.y, 0.0, 1.0);
			float simLipDown = simT.y < 0.0 ? simT.z : 0.0;
			float simFold = simT.y < 0.0 ? 0.0 : simT.z;
			vec3 simLipC = mix(skin * vec3(0.62, 0.34, 0.33), vec3(0.42, 0.14, 0.13), 0.45);
			skin = mix(skin, simLipC, ${LIP_MIX} * gameSimLip) * (1.0 - 0.45 * clamp(simT.y, 0.0, 1.0));
			skin = skin * (1.0 - ${LIP_SHADE} * gameSimLip * clamp(simLipDown, 0.0, 1.0)) * (1.0 - ${FOLD} * clamp(simFold * simT4.y, 0.0, 1.0));
			float scalp = clamp(-simF.z - 1.0, 0.0, 1.0) * simTexel(7).w;
			skin = mix(skin, simTexel(3).rgb * 0.7, scalp);
			vec3 cTop = simTexel(1).rgb;
			vec3 cBottom = simTexel(2).rgb;
			vec3 cShoe = simTexel(6).rgb;
			// An open jacket's shirt (texel 4 r = 1 + n + its neck line / 4) on the torso, from the waist to the
			// neck line and in a collar above it (the jacket shell covers the rest); an apron's (n 4) all over the
			// top, sleeves too (the apron shell takes the top's colour); then the bands.
			float shirtN = floor(simT4.x) - 1.0;
			vec3 shirt = ${SHIRT_WHITE} ? vec3(${SHIRTS[0]}) : shirtN < ${SHIRT.charcoal + 0.5} ? vec3(${SHIRTS[1]}) : vec3(${SHIRTS[2]});
			float neckY = fract(simT4.x) * 4.0;
			float inV = max(1.0 - smoothstep(neckY - 0.008, neckY, simF.y), 1.0 - smoothstep(0.01, 0.014, simF.z - c2));
			cTop = simFabric(mix(cTop, shirt, ${SHIRT_MIX}), simHemTop);
			cBottom = simFabric(cBottom, simHemBottom);
			cShoe = simFabric(cShoe, simHemShoe);
			vec3 topOver = mix(mix(skin, cBottom, simBottom), cTop, simTop);
			vec3 topUnder = mix(mix(skin, cTop, simTop), cBottom, simBottom);
			gameSimOcc = ${ao(`(${AO_GAIN} + ${AO_AGE} * simT4.y)`)};
			// The shade under a loose garment's edge (EASE: it stands a step off what is under it).
			vec2 simEase = simTexel(8).xy;
			float simUnderTop = (1.0 - smoothstep(0.0, ${EDGE_REACH}, -gTop)) * (1.0 - simTop) * (simT4.x > 0.5 ? 1.0 : 1.0 - simBottom) * simEase.x;
			float simUnderBottom = (1.0 - smoothstep(0.0, ${EDGE_REACH}, -gBottom)) * (1.0 - simBottom) * (1.0 - simTop) * simEase.y;
			float simEdge = 1.0 - ${EDGE_SHADE} * max(simUnderTop, simUnderBottom) * (1.0 - simShoe);
			return mix(simT4.x > 0.5 ? topOver : topUnder, cShoe, simShoe) * gameSimOcc * simEdge;
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
			// The upper lid's shade over the top of the eye, and the ball darker towards its corners: a white left
			// flat reads as a sticker on the face.
			float simLid = 1.0 - ${LID_SHADE} * smoothstep(0.1 * simF.z, 1.5 * simF.z, simF.y);
			float simRound = 1.0 - ${EYE_ROUND} * smoothstep(1.2 * simF.z, 2.6 * simF.z, simR);
			return mix(mix(vec3(${SCLERA}), irisColour, iris), vec3(0.02), pupil) * simLid * simRound;
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
		// Painted clothes are rougher than skin, the lips a little too (they don't take a highlight).
		dGlossiness = mix(dGlossiness, ${CLOTH_MICRO}, gameSimCloth);
		dGlossiness = mix(dGlossiness, ${LIP_MICRO}, gameSimLip);
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
	var<private> gameSimLip: f32 = 0.0;
	var<private> gameSimCatch: f32 = 0.0;
	var<private> gameSimOcc: f32 = 1.0;
	var<private> gameSimWrapped: f32 = 0.0;
	var<private> gameSimWrap: f32 = 0.0;
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
			skin = skin * mix(vec3f(1.0), vec3f(0.98, 0.62, 0.6), clamp(simT.x, 0.0, 1.0));
			gameSimLip = clamp(-simT.y, 0.0, 1.0);
			let simLipDown = select(0.0, simT.z, simT.y < 0.0);
			let simFold = select(simT.z, 0.0, simT.y < 0.0);
			let simLipC = mix(skin * vec3f(0.62, 0.34, 0.33), vec3f(0.42, 0.14, 0.13), 0.45);
			skin = mix(skin, simLipC, ${LIP_MIX} * gameSimLip) * (1.0 - 0.45 * clamp(simT.y, 0.0, 1.0));
			skin = skin * (1.0 - ${LIP_SHADE} * gameSimLip * clamp(simLipDown, 0.0, 1.0)) * (1.0 - ${FOLD} * clamp(simFold * simT4.y, 0.0, 1.0));
			let scalp = clamp(-simF.z - 1.0, 0.0, 1.0) * simTexel(7).w;
			skin = mix(skin, simTexel(3).rgb * 0.7, scalp);
			var cTop = simTexel(1).rgb;
			var cBottom = simTexel(2).rgb;
			var cShoe = simTexel(6).rgb;
			let shirtN = floor(simT4.x) - 1.0;
			let shirt = select(select(vec3f(${SHIRTS[2]}), vec3f(${SHIRTS[1]}), shirtN < ${SHIRT.charcoal + 0.5}), vec3f(${SHIRTS[0]}), ${SHIRT_WHITE});
			let neckY = fract(simT4.x) * 4.0;
			let inV = max(1.0 - smoothstep(neckY - 0.008, neckY, simF.y), 1.0 - smoothstep(0.01, 0.014, simF.z - c2));
			cTop = simFabric(mix(cTop, shirt, ${SHIRT_MIX}), simHemTop);
			cBottom = simFabric(cBottom, simHemBottom);
			cShoe = simFabric(cShoe, simHemShoe);
			let topOver = mix(mix(skin, cBottom, simBottom), cTop, simTop);
			let topUnder = mix(mix(skin, cTop, simTop), cBottom, simBottom);
			gameSimOcc = ${ao(`(${AO_GAIN} + ${AO_AGE} * simT4.y)`)};
			let simEase = simTexel(8).xy;
			let simUnderTop = (1.0 - smoothstep(0.0, ${EDGE_REACH}, -gTop)) * (1.0 - simTop) * select(1.0 - simBottom, 1.0, simT4.x > 0.5) * simEase.x;
			let simUnderBottom = (1.0 - smoothstep(0.0, ${EDGE_REACH}, -gBottom)) * (1.0 - simBottom) * (1.0 - simTop) * simEase.y;
			let simEdge = 1.0 - ${EDGE_SHADE} * max(simUnderTop, simUnderBottom) * (1.0 - simShoe);
			return mix(select(topUnder, topOver, simT4.x > 0.5), cShoe, simShoe) * gameSimOcc * simEdge;
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
			let simLid = 1.0 - ${LID_SHADE} * smoothstep(0.1 * simF.z, 1.5 * simF.z, simF.y);
			let simRound = 1.0 - ${EYE_ROUND} * smoothstep(1.2 * simF.z, 2.6 * simF.z, simR);
			return mix(mix(vec3f(${SCLERA}), irisColour, iris), vec3f(0.02), pupil) * simLid * simRound;
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
		dGlossiness = mix(dGlossiness, ${LIP_MICRO}, gameSimLip);
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

/** The residents' feature, after the cutaway, basements and plain finishes and before the look. */
export const RESIDENT: ShaderFeature = {
  name: 'resident',
  priority: 230,
  glsl: { ...VS_GLSL, ...PS_GLSL },
  wgsl: { ...VS_WGSL, ...PS_WGSL },
};
