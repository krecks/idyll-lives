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
