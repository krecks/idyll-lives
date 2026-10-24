/**
 * How the game's shader features reach PlayCanvas's standard material: one set of chunk overrides,
 * the same text for every material the library makes or adopts (so materials with the same
 * features share their programs), each feature's code gated by its own define (`GAME_*`,
 * `material.setDefine`). GLSL (WebGL2) and WGSL (WebGPU) side by side, always both: PlayCanvas
 * compiles a material's WGSL only when it has WGSL chunks.
 *
 * The overrides are the engine's own chunks with a function renamed and a wrapper of ours in its
 * place (`getAlbedo` runs the engine's, then the pattern), so a PlayCanvas upgrade keeps its own
 * code; the names and lines they rely on are checked by `tools/render/check_chunks.ts`. The slots:
 *
 * - `vsDecl`, `vsEnd`: `litUserDeclarationVS`, `litUserMainEndVS`.
 * - `normalVS`: an instanced draw's normal as Babylon's thin instances had it (`INSTANCE_NORMAL`).
 * - `vsLocal`: the vertex's local (game-space) position, `localPos`, after morphing and before the
 *   model matrix, in `transformCoreVS`'s `getLocalPosition` (instanced draws too). Wind and
 *   blankets add their snippets here, or a `transformInstancingVS` of their own.
 * - `psDecl`, `psStart`: `litUserDeclarationPS`, `litUserMainStartPS` (which runs in every pass,
 *   the shadow pass included).
 * - `albedo`, `metalness`, `gloss`, `reflDir`, `emissive`: after `getAlbedo`, `getMetalness`,
 *   `getGlossiness`, `getReflDir`, `getEmission` (`diffusePS`, `metalnessPS`, `glossPS`,
 *   `reflDirPS`, `emissivePS`).
 * - `combine`: `combineColor` (`combinePS`): the engine's is `combineColorEngine`.
 * - `end`: replaces `endPS` (the colour, fog, tone mapping and gamma) where its defines say so.
 * - `fogPS`: Babylon's haze in the game's materials (`fog.ts`); `reflectionEnvPS`, `reflectionEnvHQPS`:
 *   the sky's reflection rising at a slant, as Babylon's environment BRDF had it, and `fresnelSchlickPS`:
 *   Babylon's Schlick term for the lights' highlights (`envBrdf.ts`); `normalMapPS`: normal maps at
 *   Babylon's strength (`bump.ts`).
 *
 * Features run in the order of the Babylon plugins they port (their priority), as Babylon injected
 * them. Engine-wide patches (`patchEngineChunks`, once per device) put the game's light where no
 * per-material chunk reaches: the sky's hemispheric fill and the split of the ambient from the
 * direct light (`litForwardBackendPS`), the sun kept out of basements (`lightFunctionLightPS`), and
 * the lamps kept in their rooms (`clusteredLightPS`, `roomLights.ts`). They only act under the
 * game's defines, so other materials draw as PlayCanvas has them. The lamps' falloff
 * (`falloffInvSquaredPS`, `spotPS`) is Babylon's in the game's materials (`roomLights.ts` too).
 *
 * World positions in the fragment shader are PlayCanvas's (`vPositionW`, mirrored in z from game
 * space by the world root, `space.ts`): masks in game coordinates read `gamePos(vPositionW)`.
 * Vertex snippets work on the local position, which is game space already.
 */

import * as pc from '../engine';
import { CUTAWAY } from './cutaway';
import { DIAG_VIEW } from './diagView';
import { gameNormalMap } from './bump';
import { gameFresnel, gameReflection } from './envBrdf';
import { gameFog } from './fog';
import { SOFT_LOOK } from './softLook';
import { ROOM_LIGHT_CODE } from './roomLights';
import { PATTERN, PLAIN_FINISH, VERTEX_TWEAKS } from './surface';
import { UNDERGROUND } from './underground';

export type Language = 'glsl' | 'wgsl';

export interface FeatureCode {
  vsDecl?: string;
  vsLocal?: string;
  vsEnd?: string;
  psDecl?: string;
  psStart?: string;
  albedo?: string;
  metalness?: string;
  gloss?: string;
  reflDir?: string;
  emissive?: string;
  /** The body of `combineColor` (one feature at most: the soft look). */
  combine?: string;
  /** `endPS`'s replacement where the feature's defines are set (one feature at most: the diag view). */
  end?: { when: string; code: string };
}

export interface ShaderFeature {
  name: string;
  /** The Babylon plugin's priority: lower runs first. */
  priority: number;
  glsl: FeatureCode;
  wgsl: FeatureCode;
}

/** Vertex attribute of the cutaway (`Geo`'s `cut`: mask A, kind, mask B, 0). */
export const WALL_CUT_SEMANTIC = pc.SEMANTIC_ATTR7;
/**
 * Per-instance colour stream (RGBA, 4 floats; sRGB where the material converts vertex colours):
 * location 10, free beside the instancing matrix (`instance_line1..4` at ATTR11, 12, 14, 15).
 */
export const INSTANCE_COLOR_SEMANTIC = pc.SEMANTIC_ATTR10;

const COMMON: ShaderFeature = {
  name: 'common',
  priority: 0,
  glsl: {
    psDecl: `
vec3 gamePos(vec3 p) { return vec3(p.x, p.y, -p.z); }
`,
  },
  wgsl: {
    psDecl: `
fn gamePos(p: vec3f) -> vec3f { return vec3f(p.x, p.y, -p.z); }
`,
  },
};

const features: ShaderFeature[] = [COMMON, VERTEX_TWEAKS, PATTERN, CUTAWAY, UNDERGROUND, PLAIN_FINISH, SOFT_LOOK, DIAG_VIEW].sort((a, b) => a.priority - b.priority);

/**
 * Adds a feature (wind, blankets). Before the first material is made: the chunks
 * are built once per device.
 */
export function registerShaderFeature(feature: ShaderFeature): void {
  features.push(feature);
  features.sort((a, b) => a.priority - b.priority);
}

/** The engine functions a slot wraps: the declaration renamed, and how the wrapper starts and calls it. */
const WRAPS = {
  diffusePS: {
    slot: 'albedo',
    glsl: { decl: 'void getAlbedo() {', head: 'void getAlbedo() {\n\tgetAlbedoEngine();' },
    wgsl: { decl: 'fn getAlbedo() {', head: 'fn getAlbedo() {\n\tgetAlbedoEngine();' },
  },
  metalnessPS: {
    slot: 'metalness',
    glsl: { decl: 'void getMetalness() {', head: 'void getMetalness() {\n\tgetMetalnessEngine();' },
    wgsl: { decl: 'fn getMetalness() {', head: 'fn getMetalness() {\n\tgetMetalnessEngine();' },
  },
  glossPS: {
    slot: 'gloss',
    glsl: { decl: 'void getGlossiness() {', head: 'void getGlossiness() {\n\tgetGlossinessEngine();' },
    wgsl: { decl: 'fn getGlossiness() {', head: 'fn getGlossiness() {\n\tgetGlossinessEngine();' },
  },
  reflDirPS: {
    slot: 'reflDir',
    glsl: { decl: 'void getReflDir(vec3 worldNormal, vec3 viewDir, float gloss, mat3 tbn) {', head: 'void getReflDir(vec3 worldNormal, vec3 viewDir, float gloss, mat3 tbn) {\n\tgetReflDirEngine(worldNormal, viewDir, gloss, tbn);' },
    wgsl: { decl: 'fn getReflDir(worldNormal: vec3f, viewDir: vec3f, gloss: f32, tbn: mat3x3f) {', head: 'fn getReflDir(worldNormal: vec3f, viewDir: vec3f, gloss: f32, tbn: mat3x3f) {\n\tgetReflDirEngine(worldNormal, viewDir, gloss, tbn);' },
  },
  emissivePS: {
    slot: 'emissive',
    glsl: { decl: 'void getEmission() {', head: 'void getEmission() {\n\tgetEmissionEngine();' },
    wgsl: { decl: 'fn getEmission() {', head: 'fn getEmission() {\n\tgetEmissionEngine();' },
  },
} as const;

/** The anchors the overrides and patches rely on (also listed in `tools/render/check_chunks.ts`). */
export const ANCHORS = {
  glsl: {
    transformCoreVS: 'vec3 getLocalPosition(vec3 vertexPosition) {',
    normalVS: 'return normalize(dNormalMatrix * localNormal);',
    combinePS: 'vec3 combineColor(vec3 albedo, vec3 sheenSpecularity, float clearcoatSpecularity) {',
    litForwardBackendPS: '#ifdef LIT_LIGHTING || LIT_REFLECTIONS',
    lightFunctionLightPS: 'vec3 lightColor = light{i}_color;',
    clusteredLightDecode: 'ClusterLightData clusterLightData = decodeClusterLightCore(lightIndex);',
    clusteredLightStart: '\tif (numClusteredLights <= 1)',
    falloffInvSquaredPS: 'float getFalloffInvSquared(float lightRadius, vec3 lightDir) {',
    spotPS: 'float getSpotEffect(vec3 lightSpotDir, float lightInnerConeAngle, float lightOuterConeAngle, vec3 lightDirNorm) {',
  },
  wgsl: {
    transformCoreVS: 'fn getLocalPosition(vertexPosition: vec3f) -> vec3f {',
    normalVS: 'return normalize(dNormalMatrix * localNormal);',
    combinePS: 'fn combineColor(albedo: vec3f, sheenSpecularity: vec3f, clearcoatSpecularity: f32) -> vec3f {',
    litForwardBackendPS: '#ifdef LIT_LIGHTING || LIT_REFLECTIONS',
    lightFunctionLightPS: 'var lightColor: vec3f = uniform.light{i}_color;',
    clusteredLightDecode: 'let clusterLightData: ClusterLightData = decodeClusterLightCore(lightIndex);',
    clusteredLightStart: '\tif (uniform.numClusteredLights <= 1) {',
    falloffInvSquaredPS: 'fn getFalloffInvSquared(lightRadius: f32, lightDir: vec3f) -> f32 {',
    spotPS: 'fn getSpotEffect(lightSpotDir: vec3f, lightInnerConeAngle: f32, lightOuterConeAngle: f32, lightDirNorm: vec3f) -> f32 {',
  },
} as const;

/** The engine's chunk `name` in `lang` for `device` (as PlayCanvas has it, before any override). */
export function engineChunk(device: pc.GraphicsDevice, lang: Language, name: string): string {
  const code = pc.ShaderChunks.get(device, lang === 'glsl' ? pc.SHADERLANGUAGE_GLSL : pc.SHADERLANGUAGE_WGSL).get(name);
  if (typeof code !== 'string') throw new Error(`[render] PlayCanvas has no shader chunk ${name}`);
  return code;
}

/** `code` with `anchor` replaced (exactly once: a missing anchor is an upgrade to look at). */
export function replaceOnce(code: string, name: string, anchor: string, replacement: string): string {
  const at = code.indexOf(anchor);
  if (at < 0 || code.indexOf(anchor, at + anchor.length) >= 0) throw new Error(`[render] shader chunk ${name}: anchor not found once: ${anchor}`);
  return code.slice(0, at) + replacement + code.slice(at + anchor.length);
}

function joined(lang: Language, slot: keyof Omit<FeatureCode, 'end'>): string {
  return features
    .map((f) => f[lang][slot])
    .filter((c): c is string => !!c)
    .join('\n');
}

const cache = new WeakMap<pc.GraphicsDevice, Record<Language, Record<string, string>>>();

/** The chunk overrides for `device` (built once). */
export function gameChunks(device: pc.GraphicsDevice): Record<Language, Record<string, string>> {
  let chunks = cache.get(device);
  if (!chunks) cache.set(device, (chunks = { glsl: build(device, 'glsl'), wgsl: build(device, 'wgsl') }));
  return chunks;
}

function build(device: pc.GraphicsDevice, lang: Language): Record<string, string> {
  const glsl = lang === 'glsl';
  const out: Record<string, string> = {};
  out.litUserDeclarationVS = joined(lang, 'vsDecl');
  out.litUserMainEndVS = joined(lang, 'vsEnd');
  out.litUserDeclarationPS = joined(lang, 'psDecl');
  out.litUserMainStartPS = joined(lang, 'psStart');

  const core = ANCHORS[lang].transformCoreVS;
  out.transformCoreVS =
    replaceOnce(engineChunk(device, lang, 'transformCoreVS'), 'transformCoreVS', core, glsl ? 'vec3 getLocalPositionEngine(vec3 vertexPosition) {' : 'fn getLocalPositionEngine(vertexPosition: vec3f) -> vec3f {') +
    (glsl
      ? `\nvec3 getLocalPosition(vec3 vertexPosition) {\n\tvec3 localPos = getLocalPositionEngine(vertexPosition);\n${joined(lang, 'vsLocal')}\n\treturn localPos;\n}\n`
      : `\nfn getLocalPosition(vertexPosition: vec3f) -> vec3f {\n\tvar localPos: vec3f = getLocalPositionEngine(vertexPosition);\n${joined(lang, 'vsLocal')}\n\treturn localPos;\n}\n`);

  for (const [chunk, wrap] of Object.entries(WRAPS)) {
    const w = wrap[lang];
    const engine = w.decl.replace(/(get\w+)\(/, '$1Engine(');
    out[chunk] = `${replaceOnce(engineChunk(device, lang, chunk), chunk, w.decl, engine)}\n${w.head}\n${joined(lang, wrap.slot)}\n}\n`;
  }

  const combine = ANCHORS[lang].combinePS;
  out.combinePS = `${replaceOnce(engineChunk(device, lang, 'combinePS'), 'combinePS', combine, glsl ? 'vec3 combineColorEngine(vec3 albedo, vec3 sheenSpecularity, float clearcoatSpecularity) {' : 'fn combineColorEngine(albedo: vec3f, sheenSpecularity: vec3f, clearcoatSpecularity: f32) -> vec3f {')}\n${combine}\n${joined(lang, 'combine') || '\treturn combineColorEngine(albedo, sheenSpecularity, clearcoatSpecularity);'}\n}\n`;

  const ends = features.map((f) => f[lang].end).filter((e) => !!e);
  const engineEnd = engineChunk(device, lang, 'endPS');
  out.endPS = ends.length ? `#if ${ends.map((e) => e.when).join(' || ')}\n${ends.map((e) => e.code).join('\n')}\n#else\n${engineEnd}\n#endif\n` : engineEnd;

  // An instance's normal as Babylon's thin instances had it (`INSTANCE_NORMAL`).
  out.normalVS = replaceOnce(engineChunk(device, lang, 'normalVS'), 'normalVS', ANCHORS[lang].normalVS, INSTANCE_NORMAL[lang]);

  out.uv0VS = glsl ? UV0_GLSL : UV0_WGSL;
  // The haze as Babylon's PBR drew it (`fog.ts`).
  out.fogPS = gameFog(engineChunk(device, lang, 'fogPS'), lang, replaceOnce);
  // The sky's reflection rising at a slant, as Babylon's environment BRDF had it (`envBrdf.ts`).
  for (const chunk of ['reflectionEnvPS', 'reflectionEnvHQPS']) out[chunk] = gameReflection(engineChunk(device, lang, chunk), chunk, lang, replaceOnce);
  // The lights' highlights rising towards grazing angles, as Babylon's Schlick term had them (`envBrdf.ts`).
  out.fresnelSchlickPS = gameFresnel(engineChunk(device, lang, 'fresnelSchlickPS'), lang, replaceOnce);
  // Normal maps at Babylon's strength (`bump.ts`).
  out.normalMapPS = gameNormalMap(engineChunk(device, lang, 'normalMapPS'), lang, replaceOnce);

  // The lamps' falloff, Babylon's (`roomLights.ts`): the engine's functions renamed, ours in their place.
  for (const chunk of ['falloffInvSquaredPS', 'spotPS'] as const) {
    const anchor = ANCHORS[lang][chunk];
    const code = ROOM_LIGHT_CODE[lang][chunk === 'spotPS' ? 'spot' : 'falloff'];
    out[chunk] = `${replaceOnce(engineChunk(device, lang, chunk), chunk, anchor, anchor.replace(/(get\w+)\(/, '$1Engine('))}\n${code}`;
  }
  return out;
}

/**
 * `normalVS`'s normal for an instanced draw of the game's lit materials: Babylon's thin instances'
 * (its PBR vertex shader), `normalize(M · (n / (|M₀|², |M₁|², |M₂|²)))` with M the model matrix's 3 × 3
 * (PlayCanvas's normal matrix for instanced draws), the inverse transpose for a matrix of axis scales
 * and a rotation: an instance scaled unevenly (a quilt fitted to its bed, a resized object) shades as
 * it did. The world root's mirror leaves the columns' lengths as they are; tangents keep the plain
 * matrix, as Babylon's did. (A collapsed copy, all zeros, gets no normal; it draws nothing.)
 */
const INSTANCE_NORMAL: Record<Language, string> = {
  glsl: `#if defined(GAME_LOOK) && defined(INSTANCING)
		vec3 gameScale2 = max(vec3(dot(dNormalMatrix[0], dNormalMatrix[0]), dot(dNormalMatrix[1], dNormalMatrix[1]), dot(dNormalMatrix[2], dNormalMatrix[2])), vec3(1e-20));
		return normalize(dNormalMatrix * (localNormal / gameScale2));
	#else
		return normalize(dNormalMatrix * localNormal);
	#endif`,
  wgsl: `#if defined(GAME_LOOK) && defined(INSTANCING)
		let gameScale2 = max(vec3f(dot(dNormalMatrix[0], dNormalMatrix[0]), dot(dNormalMatrix[1], dNormalMatrix[1]), dot(dNormalMatrix[2], dNormalMatrix[2])), vec3f(1e-20));
		return normalize(dNormalMatrix * (localNormal / gameScale2));
	#else
		return normalize(dNormalMatrix * localNormal);
	#endif`,
};

/** `uv0VS` for planar UVs (`GAME_PLANAR_UV`: the local XZ in metres); the engine's otherwise. */
const UV0_GLSL = `
#ifdef NINESLICED
	vec2 getUv0() {
		vec2 uv = vertex_position.xz;
		vec2 positiveUnitOffset = clamp(vertex_position.xz, vec2(0.0), vec2(1.0));
		vec2 negativeUnitOffset = clamp(-vertex_position.xz, vec2(0.0), vec2(1.0));
		uv += (-positiveUnitOffset * innerOffset.xy + negativeUnitOffset * innerOffset.zw) * vertex_texCoord0.xy;
		uv = uv * -0.5 + 0.5;
		uv = uv * atlasRect.zw + atlasRect.xy;
		vMask = vertex_texCoord0.xy;
		return uv;
	}
#elif defined(GAME_PLANAR_UV)
	vec2 getUv0() {
		return vertex_position.xz;
	}
#else
	vec2 getUv0() {
		return vertex_texCoord0;
	}
#endif
`;

const UV0_WGSL = `
#ifdef NINESLICED
	fn getUv0() -> vec2f {
		var uv = vertex_position.xz;
		let positiveUnitOffset = clamp(vertex_position.xz, vec2f(0.0, 0.0), vec2f(1.0, 1.0));
		let negativeUnitOffset = clamp(-vertex_position.xz, vec2f(0.0, 0.0), vec2f(1.0, 1.0));
		uv = uv + ((-positiveUnitOffset * uniform.innerOffset.xy) + (negativeUnitOffset * uniform.innerOffset.zw)) * vertex_texCoord0.xy;
		uv = uv * -0.5 + vec2f(0.5, 0.5);
		uv = uv * uniform.atlasRect.zw + uniform.atlasRect.xy;
		dMaskGlobal = vertex_texCoord0.xy;
		return uv;
	}
#elif defined(GAME_PLANAR_UV)
	fn getUv0() -> vec2f {
		return vertex_position.xz;
	}
#else
	fn getUv0() -> vec2f {
		return vertex_texCoord0;
	}
#endif
`;

/** The engine-wide patches' code, per language (see the file's header). */
const ENGINE_PATCHES: Record<Language, { backend: string; sun: string }> = {
  glsl: {
    // The ambient PlayCanvas added (the sky's harmonics, occluded) is kept apart, and the sky's
    // hemispheric fill comes in as direct light, as Babylon's hemispheric light did.
    backend: `
	#if defined(GAME_LOOK) && defined(LIT_NEEDS_NORMAL)
		gameAmbient = dDiffuseLight;
		dDiffuseLight = gameHemi(litArgs_worldNormal);
	#endif
`,
    sun: `
	#ifdef GAME_UNDERGROUND
		#if LIGHT{i}TYPE == DIRECTIONAL
			lightColor *= gameUnder.x;
		#endif
	#endif`,
  },
  wgsl: {
    backend: `
	#if defined(GAME_LOOK) && defined(LIT_NEEDS_NORMAL)
		gameAmbient = dDiffuseLight;
		dDiffuseLight = gameHemi(litArgs_worldNormal);
	#endif
`,
    sun: `
	#ifdef GAME_UNDERGROUND
		#if LIGHT{i}TYPE == DIRECTIONAL
			lightColor = lightColor * gameUnder.x;
		#endif
	#endif`,
  },
};

const PATCHED = '// (idyll-lives: patched)';
const patchedDevices = new WeakSet<pc.GraphicsDevice>();

/** Patches `device`'s engine chunks (once; before the first shader is built). */
export function patchEngineChunks(device: pc.GraphicsDevice): void {
  if (patchedDevices.has(device)) return;
  patchedDevices.add(device);
  for (const lang of ['glsl', 'wgsl'] as const) {
    const chunks = pc.ShaderChunks.get(device, lang === 'glsl' ? pc.SHADERLANGUAGE_GLSL : pc.SHADERLANGUAGE_WGSL);
    const backend = engineChunk(device, lang, 'litForwardBackendPS');
    if (!backend.includes(PATCHED)) {
      const anchor = ANCHORS[lang].litForwardBackendPS;
      chunks.set('litForwardBackendPS', `${PATCHED}\n${replaceOnceFirst(backend, anchor, `${ENGINE_PATCHES[lang].backend}\t${anchor}`)}`);
    }
    const light = engineChunk(device, lang, 'lightFunctionLightPS');
    if (!light.includes(PATCHED)) {
      const anchor = ANCHORS[lang].lightFunctionLightPS;
      chunks.set('lightFunctionLightPS', `${PATCHED}\n${replaceOnce(light, 'lightFunctionLightPS', anchor, `${anchor}${ENGINE_PATCHES[lang].sun}`)}`);
    }
    // The lamps kept in their rooms, and out of the meshes they don't light (`roomLights.ts`).
    const clustered = engineChunk(device, lang, 'clusteredLightPS');
    if (!clustered.includes(PATCHED)) {
      const room = ROOM_LIGHT_CODE[lang];
      const decode = ANCHORS[lang].clusteredLightDecode;
      const start = ANCHORS[lang].clusteredLightStart;
      const skipped = replaceOnce(clustered, 'clusteredLightPS', decode, `${decode}${room.skip}`);
      chunks.set('clusteredLightPS', `${PATCHED}\n${room.decl}\n${replaceOnce(skipped, 'clusteredLightPS', start, `${room.noLamps}${start}`)}`);
    }
  }
}

/** `code` with the first `anchor` replaced (`litForwardBackendPS` has it once at the top level). */
function replaceOnceFirst(code: string, anchor: string, replacement: string): string {
  const at = code.indexOf(anchor);
  if (at < 0) throw new Error(`[render] shader chunk litForwardBackendPS: anchor not found: ${anchor}`);
  return code.slice(0, at) + replacement + code.slice(at + anchor.length);
}

/**
 * Gives `material` the game's chunks (both languages) and its base define. Its features are then
 * switched with their defines (`material.setDefine`), followed by `material.update()`.
 */
export function installGameChunks(material: pc.Material, device: pc.GraphicsDevice): void {
  patchEngineChunks(device);
  const chunks = gameChunks(device);
  material.shaderChunksVersion = '2.8';
  material.getShaderChunks(pc.SHADERLANGUAGE_GLSL).add(chunks.glsl);
  material.getShaderChunks(pc.SHADERLANGUAGE_WGSL).add(chunks.wgsl);
  material.setDefine('GAME_LOOK', '');
}
