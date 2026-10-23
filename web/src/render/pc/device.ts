/**
 * The graphics device: WebGPU where the browser has it and it starts, else WebGL2 (`?renderer=webgl`
 * or the setting forces WebGL2). A WebGPU that had an adapter but failed to start is noted for the
 * session (`shared/gpu.ts`): later devices (the previews') go to WebGL2 straight away, and the game
 * says so once.
 *
 * No multisampling on the canvas: only the last full-screen pass draws to it. The scene itself is
 * smoothed in the post-processing (`QualitySettings.msaaSamples`, or FXAA).
 */

import * as pc from './engine';
import { noteWebGpuFailed, webGpuFailed } from '../shared/gpu';

export async function createDevice(canvas: HTMLCanvasElement, forceWebGL: boolean): Promise<pc.GraphicsDevice> {
  const wantGpu = !forceWebGL && !webGpuFailed() && 'gpu' in navigator && !!(await navigator.gpu.requestAdapter().catch(() => null));
  const device = await pc.createGraphicsDevice(canvas, {
    deviceTypes: wantGpu ? [pc.DEVICETYPE_WEBGPU, pc.DEVICETYPE_WEBGL2] : [pc.DEVICETYPE_WEBGL2],
    antialias: false,
    alpha: false,
    powerPreference: 'high-performance',
  });
  if (wantGpu && !device.isWebGPU) {
    // An adapter but no device (a blocklisted driver, a device lost while starting): WebGL2, for the rest of the session.
    console.warn('[render] WebGPU failed to start; drawing with WebGL2');
    noteWebGpuFailed();
  }
  // The drawing size is the renderer's (`applyResolution`): never the device's own pixel-ratio scaling.
  device.maxPixelRatio = 1;
  return device;
}

/** A PlayCanvas app on `device` without its own frame loop: the renderer calls `update` and `render` itself, paced (`shared/pacing.ts`). */
export function createApp(canvas: HTMLCanvasElement, device: pc.GraphicsDevice): pc.AppBase {
  const options = new pc.AppOptions();
  options.graphicsDevice = device;
  options.componentSystems = [pc.RenderComponentSystem, pc.CameraComponentSystem, pc.LightComponentSystem];
  options.resourceHandlers = [pc.ContainerHandler, pc.TextureHandler];
  const app = new pc.AppBase(canvas);
  app.init(options);
  app.setCanvasFillMode(pc.FILLMODE_NONE);
  app.setCanvasResolution(pc.RESOLUTION_FIXED, Math.max(1, canvas.width), Math.max(1, canvas.height));
  app.start();
  cancelAnimationFrame(app.frameRequestId);
  app.frameRequestId = null;
  return app;
}
