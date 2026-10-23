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
  // render/pc/PlayCanvasRenderer.ts: draw calls per frame, read and reset each frame.
  { file: 'platform/graphics/graphics-device.js', text: 'this._drawCallsPerFrame = 0;', who: 'PlayCanvasRenderer.ts: draw calls' },
  // render/pc/device.ts: the app's own frame loop stopped once started.
  { file: 'framework/app-base.js', text: 'this.frameRequestId = hasFrameLoop ? requestAnimationFrame(this.tick) : null;', who: 'device.ts: createApp' },
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
