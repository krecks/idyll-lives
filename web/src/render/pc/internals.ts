/**
 * The PlayCanvas 2.23 internals this code reaches into from more than one place. They are private API, pinned
 * by `check:chunks` (which fails if a name moves), so the shapes are named once here. Most are declared in
 * the engine's own per-file types (`engine.ts` imports those), marked `@ignore` in its docs; the one that is
 * `private` there (WebGPU's adapter) is cast to, once, here.
 */

import * as pc from './engine';

/** The forward renderer's directional shadow renderer (`Renderer._shadowRendererDirectional`). */
export interface DirectionalShadows {
  cull(light: object, comp: object, camera: pc.Camera, casters?: object, mask?: number): void;
  getLightRenderPass(light: object, camera: pc.Camera): object | null;
}

/** `app`'s directional shadow renderer, or `undefined` on a renderer that has none. */
export function directionalShadows(app: pc.AppBase): DirectionalShadows | undefined {
  // (The forward renderer keeps it in `_shadowRendererDirectional`, PlayCanvas 2.23, pinned.)
  return app.renderer?._shadowRendererDirectional;
}

/** A shader the device made, as far as `shadersCompiling` and `pcmaterials` read it. */
export interface CompiledShader extends pc.Shader {
  ready: boolean;
  failed: boolean;
  impl: { isLinked(device: pc.GraphicsDevice): boolean };
}

/** The device's live shaders (`GraphicsDevice.shaders`; WebGPU has none of its own). */
export function deviceShaders(device: pc.GraphicsDevice): CompiledShader[] {
  // (`GraphicsDevice` keeps them in `shaders`, PlayCanvas 2.23, pinned.)
  return device.shaders ?? [];
}

/**
 * What a WebGPU adapter reports (`GPUAdapter.info`): the debug report and the FXAA's `MALI` variant read it.
 * Named, not a record, so a field the browser stops reporting is a compile error.
 */
interface AdapterInfo {
  vendor?: string;
  architecture?: string;
  device?: string;
  description?: string;
}

/**
 * The context's driver names, whichever its backend has: WebGL2's `unmasked*` on the device itself,
 * WebGPU's adapter in `gpuAdapter` (so never both — the callers read what they find).
 */
export interface DriverInfo {
  unmaskedRenderer?: string;
  unmaskedVendor?: string;
  gpuAdapter?: { info?: AdapterInfo };
}

/** `device`'s driver names, for the FXAA's `MALI` variant and the debug report. */
export function deviceDriverInfo(device: pc.GraphicsDevice): DriverInfo {
  if (device instanceof pc.WebglGraphicsDevice) return { unmaskedRenderer: device.unmaskedRenderer, unmaskedVendor: device.unmaskedVendor };
  // SAFETY: WebGPU keeps the adapter in `gpuAdapter`, private in the engine's types (PlayCanvas 2.23, pinned).
  return { gpuAdapter: (device as unknown as { gpuAdapter?: { info?: AdapterInfo } }).gpuAdapter };
}
