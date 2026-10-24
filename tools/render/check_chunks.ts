#!/usr/bin/env bun
/**
 * Checks that what the PlayCanvas renderer patches or reads inside PlayCanvas is still there, in
 * the release build the game imports (`web/node_modules/playcanvas/build/playcanvas/src`): the
 * shader chunks' anchor lines the renderer inserts its code at, and the few private names it
 * reaches (`render/pc/*`, marked `SAFETY`). A PlayCanvas upgrade that moves one fails here, in CI,
 * rather than quietly drawing without it. `cd web && bun run check:chunks`.
 *
 * Each entry: the file (below the release build's src), what must be in it, and who relies on it.
 */

import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const SRC = join(import.meta.dir, '../../web/node_modules/playcanvas/build/playcanvas/src');

/** The same anchor in a chunk's GLSL and its WGSL (`scene/shader-lib/{glsl,wgsl}/chunks/<file>`). */
function both(file: string, glsl: string, wgsl: string, who: string): { file: string; text: string; who: string }[] {
  return [
    { file: `scene/shader-lib/glsl/chunks/${file}`, text: glsl, who: `${who} (GLSL)` },
    { file: `scene/shader-lib/wgsl/chunks/${file}`, text: wgsl, who: `${who} (WGSL)` },
  ];
}

const NEEDED: { file: string; text: string; who: string }[] = [
  // render/pc/sunShadows.ts: the fixed square handed to the directional shadow fit.
  { file: 'scene/renderer/renderer.js', text: 'this._shadowRendererDirectional = new ShadowRendererDirectional(', who: 'sunShadows.ts: the directional shadow renderer it patches' },
  { file: 'scene/renderer/culler.js', text: 'renderer._shadowRendererDirectional.cull(light, comp, camera, null, shadowCascadeMask);', who: 'sunShadows.ts: cull called on the instance (so patching it works)' },
  { file: 'scene/renderer/shadow-renderer-directional.js', text: 'cull(light, comp, camera, casters = null, cascadeMask = this.getCascadeMask(light))', who: 'sunShadows.ts: the cull it wraps' },
  { file: 'scene/renderer/shadow-renderer-directional.js', text: 'const frustumPoints = camera.getFrustumCorners(frustumNearDist, frustumFarDist);', who: 'sunShadows.ts: the corners it hands the fit' },
  { file: 'scene/renderer/shadow-renderer-directional.js', text: 'const cameraWorldMat = camera.node.getWorldTransform();', who: 'sunShadows.ts: corners given in the camera\'s space' },
  // render/pc/mirrors.ts: the reflection cameras share the game camera's sun shadow matrices and pass.
  { file: 'scene/light.js', text: 'getRenderData(camera, face) {', who: 'mirrors.ts: the sun\'s render data per camera, wrapped on the light' },
  { file: 'scene/renderer/render-pass-forward.js', text: 'const shadowPass = renderer._shadowRendererDirectional.getLightRenderPass(light, camera);', who: 'mirrors.ts: the sun\'s pass per camera, asked of the instance (so wrapping it works)' },
  { file: 'scene/renderer/culler.js', text: 'scene?.fire(EVENT_PRECULL, cameraComponent);', who: 'mirrors.ts: the helpers hidden from a reflection camera\'s cull' },
  { file: 'scene/renderer/render-pass-forward.js', text: 'scene.fire(EVENT_PRERENDER_LAYER, cameraComponent, layer, transparent);', who: 'mirrors.ts: the residents\' rows set per reflection camera' },
  // render/pc/PlayCanvasRenderer.ts: draw calls per frame, read and reset each frame.
  { file: 'platform/graphics/graphics-device.js', text: 'this._drawCallsPerFrame = 0;', who: 'PlayCanvasRenderer.ts: draw calls' },
  // render/pc/device.ts: the app's own frame loop stopped once started.
  { file: 'framework/app-base.js', text: 'this.frameRequestId = hasFrameLoop ? requestAnimationFrame(this.tick) : null;', who: 'device.ts: createApp' },
  // render/pc/device.ts, instancing.ts, engine.ts: a second app on the page (the previews') keeps the game's current.
  { file: 'framework/app-base.js', text: 'setApplication(this);', who: 'device.ts: createSideApp (the constructor makes the newest app current)' },
  { file: 'framework/globals.js', text: 'function setApplication(app) {', who: 'engine.ts: getApplication and setApplication, by path' },
  { file: 'framework/entity.js', text: 'constructor(name, app = getApplication()) {', who: 'instancing.ts: an entity on its parent\'s app (`_app`), not the current one' },
  // render/preview/portraits.ts: a read sent at once (otherwise its buffer is mapped before the copy is submitted).
  { file: 'platform/graphics/webgpu/webgpu-graphics-device.js', text: 'async readBuffer(stagingBuffer, size, data = null, immediate = false) {', who: 'preview/portraits.ts: the portraits read back with `immediate`' },
  // render/pc/shaders/*, materials.ts, env.ts (and crates/tools/src/envconv.rs): the chunks the game's
  // shader features wrap and patch, and the engine's ways they rely on.
  ...both('common/vert/transformCore.js', 'vec3 getLocalPosition(vec3 vertexPosition) {', 'fn getLocalPosition(vertexPosition: vec3f) -> vec3f {', 'shaders/chunks.ts: the vertex position hook (the cutaway; wind and blankets later)'),
  ...both('standard/frag/diffuse.js', 'void getAlbedo() {', 'fn getAlbedo() {', 'shaders/chunks.ts: the albedo wrap (patterns, instance colours)'),
  ...both('standard/frag/metalness.js', 'void getMetalness() {', 'fn getMetalness() {', 'shaders/chunks.ts: the metalness wrap (plain finish)'),
  ...both('standard/frag/gloss.js', 'void getGlossiness() {', 'fn getGlossiness() {', 'shaders/chunks.ts: the gloss wrap (plain finish)'),
  ...both('lit/frag/reflDir.js', 'void getReflDir(vec3 worldNormal, vec3 viewDir, float gloss, mat3 tbn) {', 'fn getReflDir(worldNormal: vec3f, viewDir: vec3f, gloss: f32, tbn: mat3x3f) {', 'shaders/chunks.ts: the reflection wrap (plain finish: metals never reflect the ground)'),
  ...both('lit/frag/combine.js', 'vec3 combineColor(vec3 albedo, vec3 sheenSpecularity, float clearcoatSpecularity) {', 'fn combineColor(albedo: vec3f, sheenSpecularity: vec3f, clearcoatSpecularity: f32) -> vec3f {', 'shaders/softLook.ts: the soft look, and the environment kept out of basements'),
  ...both('lit/frag/end.js', 'combineColor(litArgs_albedo, litArgs_sheen_specularity, litArgs_clearcoat_specularity)', 'combineColor(litArgs_albedo, litArgs_sheen_specularity, litArgs_clearcoat_specularity)', 'shaders/diagView.ts: endPS, replaced by the diag views'),
  ...both('lit/frag/pass-forward/litForwardBackend.js', '#ifdef LIT_LIGHTING || LIT_REFLECTIONS', '#ifdef LIT_LIGHTING || LIT_REFLECTIONS', 'shaders/chunks.ts: engine patch, the ambient split from the direct light and the sky fill added'),
  ...both('lit/frag/pass-forward/litForwardBackend.js', 'addAmbient(litArgs_worldNormal);', 'addAmbient(litArgs_worldNormal);', 'shaders/chunks.ts: the ambient is in dDiffuseLight before that patch'),
  ...both('lit/frag/lighting/lightFunctionLight.js', 'vec3 lightColor = light{i}_color;', 'var lightColor: vec3f = uniform.light{i}_color;', 'shaders/chunks.ts: engine patch, the sun kept out of basements'),
  ...both('lit/vert/uv0.js', 'vec2 getUv0() {', 'fn getUv0() -> vec2f {', 'shaders/chunks.ts: uv0VS replaced (planar UVs) by a copy of the engine\'s'),
  ...both('lit/frag/pass-forward/litForwardMain.js', '#include "litUserMainStartPS"', '#include "litUserMainStartPS"', 'shaders/cutaway.ts, underground.ts: the fragment start hook (forward)'),
  ...both('lit/frag/pass-shadow/litShadowMain.js', '#include "litUserMainStartPS"', '#include "litUserMainStartPS"', 'shaders/cutaway.ts: the fragment start hook (shadow pass, where the cutaway is skipped)'),
  ...both('lit/vert/litMain.js', '#include "litUserMainEndVS"', '#include "litUserMainEndVS"', 'shaders/surface.ts: instance colours, at the end of the vertex shader'),
  ...both('standard/frag/litShaderArgs.js', 'litArgs_worldNormal', 'litArgs_worldNormal', 'shaders/softLook.ts, diagView.ts: the shading normal'),
  { file: 'scene/shader-lib/programs/lit-shader.js', text: 'this.varyingsCode += `#define VARYING_${name.toUpperCase()}', who: 'shaders/surface.ts: VARYING_VVERTEXCOLOR tells the plain finish there are vertex colours' },
  { file: 'scene/shader-lib/glsl/chunks/lit/frag/ambient.js', text: 'vec3 n = cubeMapRotate(worldNormal);', who: 'envconv.rs: the ambient harmonics are read at the world normal, without the atlas\'s x mirror' },
  { file: 'scene/shader-lib/glsl/chunks/lit/frag/reflectionEnv.js', text: 'vec3 dir = cubeMapProject(reflDir) * vec3(-1.0, 1.0, 1.0);', who: 'envconv.rs: the atlas is read mirrored in x (the faces are baked for it)' },
  { file: 'scene/renderer/forward-renderer.js', text: 'drawCall.restoreReplacedParameters(material);', who: 'materials.ts digInto: a draw\'s parameter over its material\'s is undone after the draw' },
  { file: 'scene/graphics/env-lighting.js', text: 'static generateLightingSource(source, options) {', who: 'env.ts: the environment atlas built from the faces' },
  { file: 'scene/materials/standard-material-options-builder.js', text: 'options.litOptions.useVertexColorGamma = stdMat.vertexColorGamma;', who: 'materials.ts: vertex colours converted from sRGB (typed by hand there)' },
  { file: 'scene/materials/standard-material-options-builder.js', text: 'options.litOptions.ambientSH = !!stdMat.ambientSH;', who: 'materials.ts: the ambient harmonics (typed by hand there)' },
  { file: 'framework/app-base.js', text: 'this.fire("prerender");', who: 'materials.ts: the shader features\' state handed over before every frame' },
  // render/pc/lamps.ts, shaders/roomLights.ts: the lamps as clustered lights, kept in their rooms, with the game's falloff.
  ...both('lit/frag/clusteredLight.js', 'ClusterLightData clusterLightData = decodeClusterLightCore(lightIndex);', 'let clusterLightData: ClusterLightData = decodeClusterLightCore(lightIndex);', 'shaders/chunks.ts: engine patch, a lamp in another room skipped (after its decode, in evaluateClusterLight)'),
  ...both('lit/frag/clusteredLight.js', '\tif (numClusteredLights <= 1)', '\tif (uniform.numClusteredLights <= 1) {', 'shaders/chunks.ts: engine patch, no clustered loop for meshes kept out of the lamps (addClusteredLights)'),
  ...both('lit/frag/clusteredLight.js', 'void evaluateClusterLight(\n\tint lightIndex, \n\tvec3 worldNormal, ', 'fn evaluateClusterLight(\n\tlightIndex: i32,\n\tworldNormal: vec3f,', 'shaders/roomLights.ts: evaluateClusterLight has the shading normal (worldNormal) the room test classes a fragment by'),
  ...both('lit/frag/clusteredLight.js', 'falloffAttenuation = getFalloffInvSquared(light.range, lightDirW);', 'falloffAttenuation = getFalloffInvSquared(light.range, lightDirW);', 'shaders/roomLights.ts: the clustered lights fall off through getFalloffInvSquared (the physical 1/d² there)'),
  ...both('lit/frag/clusteredLight.js', 'falloffAttenuation *= getSpotEffect(light.direction, spotData.innerConeAngleCos, spotData.outerConeAngleCos, lightDirNormW);', 'falloffAttenuation = falloffAttenuation * getSpotEffect(light.direction, spotData.innerConeAngleCos, spotData.outerConeAngleCos, lightDirNormW);', 'shaders/roomLights.ts: a spot light\'s cone through getSpotEffect, with the cosine of its outer angle (the game\'s cone there)'),
  ...both('lit/frag/falloffInvSquared.js', 'float getFalloffInvSquared(float lightRadius, vec3 lightDir) {', 'fn getFalloffInvSquared(lightRadius: f32, lightDir: vec3f) -> f32 {', 'shaders/chunks.ts: falloffInvSquaredPS wrapped (the game\'s falloff)'),
  ...both('lit/frag/falloffInvSquared.js', 'float getFalloffWindow(float lightRadius, vec3 lightDir) {', 'fn getFalloffWindow(lightRadius: f32, lightDir: vec3f) -> f32 {', 'shaders/roomLights.ts: the window to zero at a light\'s range'),
  ...both('lit/frag/spot.js', 'float getSpotEffect(vec3 lightSpotDir, float lightInnerConeAngle, float lightOuterConeAngle, vec3 lightDirNorm) {', 'fn getSpotEffect(lightSpotDir: vec3f, lightInnerConeAngle: f32, lightOuterConeAngle: f32, lightDirNorm: vec3f) -> f32 {', 'shaders/chunks.ts: spotPS wrapped (the game\'s cone)'),
  { file: 'scene/shader-lib/programs/lit-shader.js', text: 'if (options.lightMaskDynamic) fDefines.set("CLUSTER_MESH_DYNAMIC_LIGHTS", true);', who: 'shaders/roomLights.ts: CLUSTER_MESH_DYNAMIC_LIGHTS marks the meshes the lamps light' },
  { file: 'scene/lighting/world-clusters.js', text: 'light.visibleThisFrame && light.intensity > 0 && runtimeLight', who: 'lamps.ts: a dark light (no intensity) is left out of the clusters' },
  { file: 'scene/light.js', text: 'tmpColor.copy(color).mulScalar(intensity).linear();', who: 'lamps.ts setIntensity: an intensity below 1 is linearised with the colour' },
  { file: 'scene/scene.js', text: 'if (this.device.isWebGPU && !value) {', who: 'lamps.ts: WebGPU has only clustered lights (`?lights=forward` is WebGL2\'s)' },
  // render/pc/post.ts, shaders/post.ts: the game's image processing, FXAA and night grade on PlayCanvas's CameraFrame.
  { file: 'extras/render-passes/frame-pass-camera-frame.js', text: 'pass.toneMapping = TONEMAP_NONE;', who: 'post.ts: the scene drawn in linear light (no tone mapping) into the frame' },
  { file: 'extras/render-passes/frame-pass-camera-frame.js', text: 'pass.gammaCorrection = GAMMA_NONE;', who: 'post.ts: the scene drawn in linear light (no gamma) into the frame' },
  { file: 'extras/render-passes/frame-pass-camera-frame.js', text: 'this.composePass.init(targetRenderTarget);', who: 'post.ts: the compose draws into the camera\'s target (the FXAA\'s source on Low)' },
  { file: 'extras/render-passes/frame-pass-camera-frame.js', text: 'resizeSource: targetRenderTarget,', who: 'post.ts: the scene sized by the camera\'s target' },
  { file: 'extras/render-passes/render-pass-compose.js', text: '["composeMainEndPS", ""]', who: 'post.ts: the compose takes the device\'s composeDeclarationsPS/composeMainEndPS (and compiles again when they change)' },
  { file: 'extras/render-passes/render-pass-compose.js', text: 'const neededGammaCorrection = srgb ? GAMMA_NONE : GAMMA_SRGB;', who: 'shaders/post.ts: GAMMA SRGB on a target that isn\'t sRGB, NONE on one that is' },
  ...both('render-pass/frag/compose/compose.js', 'vec2 uv = vec2(uv0.x, mix(uv0.y, 1.0 - uv0.y, composeTargetFlipY));', 'var uv = vec2f(uv0.x, mix(uv0.y, 1.0 - uv0.y, uniform.composeTargetFlipY));', 'shaders/post.ts: the vignette reads the compose\'s uv'),
  ...both('render-pass/frag/compose/compose.js', 'result = toneMap(max(vec3(0.0), result));', 'result = toneMap(max(vec3f(0.0), result));', 'shaders/post.ts: the scene colour clamped at 0, TONEMAP_NONE leaving it as it is'),
  ...both('render-pass/frag/compose/compose.js', '#include "composeMainEndPS"', '#include "composeMainEndPS"', 'shaders/post.ts: the image processing and the night grade on the compose\'s result'),
  ...both('render-pass/frag/compose/compose.js', 'result = gammaCorrectOutput(result);', 'result = gammaCorrectOutput(result);', 'shaders/post.ts: the compose encodes after composeMainEndPS (which hands it the display colour decoded)'),
  ...both('common/frag/gamma.js', 'return pow(color + 0.0000001, vec3(1.0 / 2.2));', 'return pow(color + 0.0000001, vec3f(1.0 / 2.2));', 'shaders/post.ts: gammaCorrectOutput is the image processing\'s pow(x, 1 / 2.2), which pow(x, 2.2) undoes'),
  ...both('common/frag/tonemapping/tonemapping.js', '#include "tonemappingAces2PS"', '#include "tonemappingAces2PS"', 'post.ts pageLook: the dev pages\' ACES replaces tonemappingAces2PS'),
  ...both('common/frag/tonemapping/tonemapping.js', 'float getExposure() { return exposure; }', 'fn getExposure() -> f32 { return uniform.exposure; }', 'shaders/post.ts PAGE_TONE_MAPPING: the dev pages\' exposure (scene.exposure)'),
  ...both('common/vert/quad.js', 'uv0 = getImageEffectUV(', 'output.uv0 = getImageEffectUV(', 'shaders/post.ts: the FXAA pass reads uv0 from quadVS'),
  // render/pc/materials.ts: colour maps in a linear format, decoded by the material with pow(2.2), as gamma-space textures.
  { file: 'platform/graphics/texture.js', text: 'return requiresManualGamma(this.format) ? "srgb" : "linear";', who: 'materials.ts textureWithFallback: an 8-bit or block format that isn\'t sRGB is decoded by the material' },
  { file: 'scene/materials/standard-material-options-builder.js', text: 'options.diffuseEncoding = stdMat.diffuseMap?.encoding;', who: 'materials.ts: the base colour map decoded by its encoding' },
  { file: 'scene/materials/standard-material-options-builder.js', text: 'options.emissiveEncoding = stdMat.emissiveMap?.encoding;', who: 'nature.ts clouds: the emissive map decoded by its encoding' },
  { file: 'scene/shader-lib/chunk-utils.js', text: '"srgb": "decodeGamma",', who: 'materials.ts: srgb-encoded maps decoded by decodeGamma' },
  ...both('common/frag/decode.js', 'return pow(raw, vec3(2.2));', 'return pow(raw, vec3f(2.2));', 'materials.ts: decodeGamma is the game\'s texture decode, pow(x, 2.2)'),
  // render/pc/sunShadows.ts: the directional shadow lookup lit outside the map, fading at its edge.
  ...both('lit/frag/lighting/lightFunctionShadow.js', 'vec3 shadowCoord = getShadowSampleCoord{i}(shadowMatrix, light{i}_shadowParams, vPositionW, vec3(0.0), lightDirW, dLightDirNormW, dVertexNormalW);', 'var shadowCoord: vec3f = getShadowSampleCoord{i}(shadowMatrix, uniform.light{i}_shadowParams, vPositionW, vec3f(0.0), &lightDirArg, dLightDirNormW, dVertexNormalW);', 'sunShadows.ts: engine patch, the sun\'s shadow coordinate tested against the map'),
  ...both('lit/frag/lighting/lightFunctionShadow.js', 'return getShadowPCF3x3(SHADOWMAP_PASS(light{i}_shadowMap), shadowCoord, light{i}_shadowParams);', 'return getShadowPCF3x3(light{i}_shadowMap, light{i}_shadowMapSampler, shadowCoord, uniform.light{i}_shadowParams);', 'sunShadows.ts: engine patch, the PCF read faded to lit at the map\'s edge'),
  ...both('lit/frag/lighting/shadowPCF3.js', 'float z = shadowCoord.z;', 'let z: f32 = shadowCoord.z;', 'sunShadows.ts: the map read at shadowCoord.xy in 0..1 over the whole map (one cascade)'),
  { file: 'scene/light.js', text: 'tmpColor.copy(color).mulScalar(intensity).linear();', who: 'PlayCanvasRenderer.ts applyLighting: the sun\'s strength goes in its colour (an intensity below 1 is linearised)' },
  // render/pc/shaders/fog.ts: the game's haze (the eye's distance, the factor linearised).
  ...both('common/frag/fog.js', 'float depth = gl_FragCoord.z / gl_FragCoord.w;', 'let depth = pcPosition.z / pcPosition.w;', 'shaders/fog.ts: the fragment\'s fog distance'),
  ...both('common/frag/fog.js', 'fogFactor = exp(-depth * depth * fog_density * fog_density);', 'return evaluateFogFactorExp2(depth, uniform.fog_density);', 'shaders/fog.ts: the exponential-squared factor'),
  ...both('lit/frag/base.js', 'uniform vec3 view_position;', 'uniform view_position: vec3f;', 'shaders/fog.ts: the eye\'s position, declared before fogPS'),
  // render/pc/shaders/envBrdf.ts: the sky's reflection rising at a slant (PlayCanvas's Fresnel stays at F0).
  ...both('lit/frag/reflectionEnv.js', 'void addReflection(vec3 reflDir, float gloss) {', 'fn addReflection(reflDir: vec3f, gloss: f32) {', 'shaders/envBrdf.ts: addReflection wrapped'),
  ...both('lit/frag/reflectionEnvHQ.js', 'void addReflection(vec3 reflDir, float gloss) {', 'fn addReflection(reflDir: vec3f, gloss: f32) {', 'shaders/envBrdf.ts: addReflection wrapped (HQ)'),
  ...both('lit/frag/fresnelSchlick.js', 'vec3 ret = specularity + (max(vec3(glossSq * specIntensity), specularity) - specularity) * fresnel;', 'let ret: vec3f = specularity + (max(vec3f(glossSq * specIntensity), specularity) - specularity) * fresnel;', 'shaders/envBrdf.ts: PlayCanvas\'s Fresnel that never rises, replaced by the full Schlick term in the game\'s materials'),
  // render/pc/shaders/bump.ts: normal maps at the game's strength.
  ...both('standard/frag/normalMap.js', 'normalMap = mix(vec3(0.0, 0.0, 1.0), normalMap, material_bumpiness);', 'normalMap = mix(vec3f(0.0, 0.0, 1.0), normalMap, uniform.material_bumpiness);', 'shaders/bump.ts: the normal map\'s strength line, replaced in the game\'s materials'),
  // render/pc/shaders/softLook.ts: the AO map on the direct light, the lights' highlights scaled.
  ...both('standard/frag/stdFrontEnd.js', 'litArgs_ao = dAo;', 'litArgs_ao = dAo;', 'shaders/softLook.ts: litArgs_ao is the AO map (STD_AO) when the combine reads it'),
  ...both('lit/frag/lighting/lightFunctionLight.js', 'dSpecularLight += lightSpecular;', 'dSpecularLight = dSpecularLight + lightSpecular;', 'shaders/softLook.ts: the lights\' highlights in dSpecularLight, scaled in the combine (SPECULAR_UNITS)'),
  // render/pc/nature.ts: the plants' wind and tint, and the shadow-only layer (layers.ts).
  ...both('common/vert/transformInstancing.js', 'attribute vec4 instance_line4;', 'attribute instance_line4: vec4f;', 'nature.ts: the wind reads the instance\'s matrix (`instance_line1..4`) in the vertex position hook'),
  ...both('lit/vert/litMain.js', 'vVertexColor = gammaCorrectInput(vertex_color);', 'output.vVertexColor = gammaCorrectInputVec4(vertex_color);', 'nature.ts: the foliage\'s tint on the converted vertex colour, at the end of the vertex shader'),
  { file: 'scene/shader-pass.js', text: 'this.defines.set(`${this.name.toUpperCase()}_PASS`, "");', who: 'nature.ts, cutaway.ts: SHADOW_PASS set in both stages of the shadow pass' },
  // render/pc/characterMaterial.ts, shaders/character.ts: residents skinned from the pose texture in the instancing transform.
  ...both('common/vert/transformCore.js', '#include "transformInstancingVS"', '#include "transformInstancingVS"', 'shaders/character.ts: an instanced draw\'s model matrix from transformInstancingVS (the residents\' skin)'),
  ...both('common/vert/transformInstancing.js', 'mat4 getModelMatrix() {', 'fn getModelMatrix() -> mat4x4f {', 'shaders/character.ts: RESIDENT_INSTANCING replaces transformInstancingVS'),
  ...both('common/vert/normalCore.js', 'vec3 getLocalNormal(vec3 vertexNormal) {', 'fn getLocalNormal(vertexNormal: vec3f) -> vec3f {', 'shaders/character.ts: getLocalNormal wrapped (the face morphs\' normals)'),
  ...both('common/vert/normalCore.js', '#if defined(SKIN) || defined(BATCH) || defined(INSTANCING) || defined(SHADOW_PASS)', '#elif defined(SKIN) || defined(BATCH) || defined(INSTANCING) || defined(SHADOW_PASS)', 'shaders/character.ts: an instanced draw\'s normal matrix is its model matrix\'s (the skinned normal)'),
  ...both('lit/vert/litMain.js', '#include  "litUserDeclarationVS"', '#include  "litUserDeclarationVS"', 'shaders/character.ts: the residents\' attributes and skin declared before transformCoreVS'),
  ...both('lit/frag/pass-forward/litForwardMain.js', 'getViewDir();', 'getViewDir();', 'shaders/character.ts: dViewDirW and dVertexNormalW set before getAlbedo (the hair\'s sheen)'),
  ...both('lit/vert/normal.js', 'return normalize(dNormalMatrix * localNormal);', 'return normalize(dNormalMatrix * localNormal);', 'shaders/chunks.ts: an instance\'s normal corrected for uneven scale (INSTANCE_NORMAL)'),
  ...both('lit/vert/normal.js', 'dNormalMatrix = getNormalMatrix(dModelMatrix);', 'dNormalMatrix = getNormalMatrix(dModelMatrix);', 'shaders/chunks.ts INSTANCE_NORMAL: dNormalMatrix is the model matrix\'s 3 × 3 for instanced draws (normalCore.js)'),
  ...both('standard/frag/emissive.js','void getEmission() {', 'fn getEmission() {', 'shaders/chunks.ts: the emission wrap (the eyes\' catchlight)'),
  { file: 'scene/shader-lib/programs/lit-shader.js', text: 'if (this.chunks.get("transformInstancingVS") === languageChunks.get("transformInstancingVS")) {', who: 'characterMaterial.ts: with transformInstancingVS replaced, instance_line1..4 are the material\'s to map (setAttribute)' },
  { file: 'platform/graphics/webgpu/webgpu-shader-processor-wgsl.js', text: 'type = WebgpuShaderProcessorWGSL.floatAttributeToInt(type, isSignedType);', who: 'characterMaterial.ts: byte bone indices read as vec4f in WGSL' },
  { file: 'platform/graphics/webgpu/webgpu-shader-processor-wgsl.js', text: 'case "uff":', who: 'shaders/character.ts: the RGBA32F pose texture declared texture_2d<uff>' },
  { file: 'scene/shader-lib/shader-utils.js', text: '(params.objDefs & (SHADERDEF_INSTANCING | SHADERDEF_BATCH | SHADERDEF_INSTANCEINDEX)) === 0', who: 'shaders/character.ts: instanced draws keep the model-matrix normal (no MESH_INSTANCE_STORAGE)' },
  { file: 'scene/renderer/shadow-renderer.js', text: 'if (layer._lightsSet.has(light)) {', who: 'layers.ts: a light casts from every layer it is on, drawn by a camera or not' },
  // render/pc/poseTexture.ts: the residents' pose texture sent a band of rows at a time.
  { file: 'platform/graphics/webgpu/webgpu-graphics-device.js', text: 'this.wgpu = wgpu;', who: 'poseTexture.ts: the WebGPU device\'s GPUDevice (its queue writes the band)' },
  { file: 'platform/graphics/webgpu/webgpu-texture.js', text: 'this.gpuTexture = wgpu.createTexture(this.desc);', who: 'poseTexture.ts: the texture\'s GPUTexture' },
  { file: 'platform/graphics/webgl/webgl-texture.js', text: 'this._glTexture = gl.createTexture();', who: 'poseTexture.ts: the texture\'s GL object' },
  { file: 'platform/graphics/webgl/webgl-graphics-device.js', text: 'bindTexture(texture) {', who: 'poseTexture.ts: binding through the device\'s texture-unit cache' },
  { file: 'platform/graphics/webgl/webgl-graphics-device.js', text: 'activeTexture(textureUnit) {', who: 'poseTexture.ts: the unit it binds on' },
  { file: 'platform/graphics/webgl/webgl-graphics-device.js', text: 'setUnpackAlignment(alignment) {', who: 'poseTexture.ts: unpack state through the device\'s cache' },
  { file: 'platform/graphics/texture.js', text: 'this._needsUpload = true;', who: 'poseTexture.ts: a texture not yet on the GPU is sent whole by its first draw' },
  // render/pc/layers.ts overLayer (stage 8, the thing in hand): the depth cleared as the layer starts.
  { file: 'scene/renderer/render-pass-forward.js', text: 'scene.fire(EVENT_PRERENDER_LAYER, cameraComponent, layer, transparent);', who: 'layers.ts overLayer: its depth clear, inside the pass, just before the layer draws' },
  { file: 'extras/render-passes/frame-pass-camera-frame.js', text: 'clearRenderTarget = false;', who: 'layers.ts overLayer: under CameraFrame only the first layer clears (the reason for its own clear)' },
  { file: 'platform/graphics/webgpu/webgpu-graphics-device.js', text: 'this.clearRenderer.clear(this, this.renderTarget, options, this.defaultClearOptions);', who: 'layers.ts overLayer: the WebGPU device\'s clear (a quad in the pass)' },
  { file: 'platform/graphics/webgl/webgl-graphics-device.js', text: 'clear(options) {', who: 'layers.ts overLayer: the WebGL device\'s clear' },
  // render/pc/gpuTiming.ts (Stage 11B): the benchmark's GPU times, draws per pass and shader compiles.
  { file: 'platform/graphics/gpu-profiler.js', text: 'report(renderVersion, timings, frameTime) {', who: 'gpuTiming.ts GpuTiming: each frame\'s timings as they come back, wrapped on the profiler' },
  { file: 'platform/graphics/gpu-profiler.js', text: 'const allocations = this.pastFrameAllocations.get(renderVersion);', who: 'gpuTiming.ts GpuTiming: a report counts only for a frame the profiler asked about' },
  { file: 'platform/graphics/gpu-profiler.js', text: 'this._passTimings.set(parsedName, (this._passTimings.get(parsedName) || 0) + timing);', who: 'gpuTiming.ts GpuTiming: the frame\'s passes by name, read after the report' },
  { file: 'platform/graphics/webgpu/webgpu-query-set.js', text: 'const frameTime = count > 0 ? Number(maxTime - minTime) * 1e-6 : 0;', who: 'gpuTiming.ts GpuTiming: WebGPU\'s frame is its first pass\'s start to its last pass\'s end' },
  { file: 'platform/graphics/webgpu/webgpu-graphics-device.js', text: 'this.setupTimeStampWrites(renderPassDesc, renderPass.name);', who: 'gpuTiming.ts GpuTiming: a pass\'s stamps named when its target is set, wrapped on the device' },
  { file: 'platform/graphics/webgpu/webgpu-graphics-device.js', text: 'const computePassDesc = this.setupTimeStampWrites(void 0, name);', who: 'gpuTiming.ts GpuTiming: compute passes come without a descriptor' },
  { file: 'platform/graphics/webgpu/webgpu-graphics-device.js', text: 'draw(primitive, indexBuffer, numInstances = 1, drawCommands, first = true, last = true, firstInstance = 0) {', who: 'gpuTiming.ts DrawCounts: the triangles of a draw, wrapped on the device (WebGPU)' },
  { file: 'platform/graphics/webgl/webgl-graphics-device.js', text: 'draw(primitive, indexBuffer, numInstances, drawCommands, first = true, last = true, firstInstance = 0) {', who: 'gpuTiming.ts DrawCounts: the triangles of a draw, wrapped on the device (WebGL2)' },
  { file: 'platform/graphics/webgpu/webgpu-graphics-device.js', text: 'startRenderPass(renderPass) {', who: 'gpuTiming.ts DrawCounts: the draws of a pass (WebGPU)' },
  { file: 'platform/graphics/webgl/webgl-graphics-device.js', text: 'endRenderPass(renderPass) {', who: 'gpuTiming.ts DrawCounts: the draws of a pass (WebGL2)' },
  { file: 'platform/graphics/shader.js', text: 'this.impl = graphicsDevice.createShaderImpl(this);', who: 'gpuTiming.ts ShaderWatch: every shader made goes through the device' },
  { file: 'platform/graphics/graphics-device.js', text: 'this._shaderStats = {', who: 'gpuTiming.ts ShaderWatch: PlayCanvas\'s own counters, filled there' },
  { file: 'platform/graphics/webgpu/webgpu-render-pipeline.js', text: 'cacheEntry.pipeline = this.create(', who: 'gpuTiming.ts ShaderWatch: a pipeline made on a cache miss, wrapped on the cache' },
  { file: 'scene/light.js', text: 'get shadowMap() {', who: 'PlayCanvasRenderer.ts: the sun\'s shadow map\'s target (the benchmark\'s shadow pass)' },
  { file: 'extras/render-passes/frame-pass-camera-frame.js', text: 'this.rt = this.createRenderTarget("SceneColor"', who: 'post.ts targets: the scene\'s target (the benchmark\'s scene pass)' },
  // render/pc/shaderKeys.ts (Stage 11B): shaders and pipelines made once for what draws the same.
  { file: 'scene/materials/standard-material.js', text: 'if (this.onUpdateShader) {', who: 'shaderKeys.ts: onUpdateShader runs before the shader\'s key is made' },
  { file: 'scene/materials/standard-material-options-builder.js', text: 'options.litOptions.lightMaskDynamic = !!(mask & MASK_AFFECT_DYNAMIC);', who: 'shaderKeys.ts: lightMaskDynamic is set for lit materials only (an unlit one keeps the last)' },
  { file: 'platform/graphics/vertex-format.js', text: 'this.renderingHash = stringIds.get(this.renderingHashString);', who: 'shaderKeys.ts: the vertex format\'s hash set in _evaluateHash (wrapped)' },
  { file: 'platform/graphics/webgpu/webgpu-vertex-buffer-layout.js', text: 'offset: interleaved ? element.offset : 0,', who: 'shaderKeys.ts: a non-interleaved format\'s pipeline layout doesn\'t depend on its offsets' },
  { file: 'platform/graphics/webgpu/webgpu-render-pipeline.js', text: 'lookupHashes[5] = vertexFormat0?.renderingHash ?? 0;', who: 'shaderKeys.ts: WebGPU\'s pipelines cached by the vertex format\'s hash' },
];

let failed = 0;
const cache = new Map<string, string>();
for (const { file, text, who } of NEEDED) {
  let src = cache.get(file);
  if (src === undefined) {
    try {
      src = readFileSync(join(SRC, file), 'utf8');
    } catch {
      src = '';
    }
    cache.set(file, src);
  }
  if (!src.includes(text)) {
    failed++;
    console.error(`check_chunks: not found in ${file}: ${text}\n  (${who})`);
  }
}
const version = JSON.parse(readFileSync(join(SRC, '../../../package.json'), 'utf8')).version;
if (failed) {
  console.error(`check_chunks: ${failed} of ${NEEDED.length} missing in PlayCanvas ${version}`);
  process.exit(1);
}
console.log(`check_chunks: ${NEEDED.length} anchors and names found in PlayCanvas ${version}`);
