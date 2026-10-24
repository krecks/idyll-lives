/**
 * The graphics device: WebGPU where the browser has it and it starts, else WebGL2 (`?renderer=webgl`
 * or the setting forces WebGL2). A WebGPU that had an adapter but failed to start is noted for the
 * session (`shared/gpu.ts`): later devices (the previews') go to WebGL2 straight away, and the game
 * says so once.
 *
 * No multisampling on the canvas: only the last full-screen pass draws to it. The scene itself is
 * smoothed in the post-processing (`QualitySettings.msaaSamples`, or FXAA; `post.ts`). The dev
 * pages draw straight onto a multisampled canvas (`antialias`), as Babylon's did.
 */

import * as pc from './engine';
import { noteWebGpuFailed, webGpuFailed } from '../shared/gpu';

/** `alpha`: a canvas that shows the page through where nothing is drawn (the previews' stages; premultiplied). */
export async function createDevice(canvas: HTMLCanvasElement, forceWebGL: boolean, antialias = false, alpha = false): Promise<pc.GraphicsDevice> {
  const wantGpu = !forceWebGL && !webGpuFailed() && 'gpu' in navigator && !!(await navigator.gpu.requestAdapter().catch(() => null));
  const device = await pc.createGraphicsDevice(canvas, {
    deviceTypes: wantGpu ? [pc.DEVICETYPE_WEBGPU, pc.DEVICETYPE_WEBGL2] : [pc.DEVICETYPE_WEBGL2],
    antialias,
    alpha,
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

/**
 * An app beside the game's on the same page (the previews', `render/preview`). The trap: PlayCanvas
 * keeps one current app for the page (`getApplication()`, the default `app` of every `new Entity()`),
 * and the `AppBase` constructor makes the newest app current, its `destroy` none. A second app made
 * with `createApp` would take the game's entities made after it (their components added through
 * its systems), and destroying it would leave the game none. So a second app is made here, the
 * page's current app put back, and destroyed with `destroySideApp`. Its own code passes it
 * explicitly (`new pc.Entity(name, app)`; `Instances` take their parent's).
 */
export function createSideApp(canvas: HTMLCanvasElement, device: pc.GraphicsDevice): pc.AppBase {
  const current = pc.getApplication() as pc.AppBase | null | undefined;
  const app = createApp(canvas, device);
  pc.setApplication(current ?? null);
  return app;
}

/** Destroys an app made by `createSideApp` (its device too); the page's current app stays current (the game's, made before or after it). */
export function destroySideApp(app: pc.AppBase): void {
  const current = pc.getApplication() as pc.AppBase | null | undefined;
  // (PlayCanvas leaves a WebGL2 context to the garbage collector: let it go now, as a page may make and drop several.)
  const gl = app.graphicsDevice.isWebGL2 ? (app.graphicsDevice as pc.WebglGraphicsDevice).gl : null;
  app.destroy();
  gl?.getExtension('WEBGL_lose_context')?.loseContext();
  pc.setApplication(current && current !== app ? current : null);
}
