/**
 * Model loading setup, imported by everything that loads glTF files (`models.ts`).
 *
 * - The glTF 2.0 loader and its extensions are fetched on first use, not with the main script
 *   (Babylon's lazy registration, like `registerBuiltInLoaders` but for glTF only).
 * - Decoders (meshopt geometry, the KTX2 texture transcoder and its WASM) come from our own
 *   `decoders/` folder, a mirror of Babylon's CDN layout (`tools/assets/decoders.mjs`): the
 *   page is cross-origin isolated, and the game shouldn't depend on a CDN anyway.
 */

import { RegisterSceneLoaderPlugin } from '@babylonjs/core/Loading/sceneLoader';
import { Tools } from '@babylonjs/core/Misc/tools';
import { registerBuiltInGLTFExtensions } from '@babylonjs/loaders/glTF/2.0/Extensions/dynamic';
import { GLTFFileLoaderMetadata } from '@babylonjs/loaders/glTF/glTFFileLoader.metadata';

// Every Babylon CDN URL (decoders, transcoders) resolves under this folder instead.
Tools.ScriptBaseUrl = new URL(`${import.meta.env.BASE_URL}decoders/`, location.href).href;

RegisterSceneLoaderPlugin({
  ...GLTFFileLoaderMetadata,
  createPlugin: async (options) => {
    const [{ GLTFFileLoader, RegisterGLTF2Loader }, { RegisterInstancedMesh }] = await Promise.all([
      import('@babylonjs/loaders/glTF/2.0/glTFLoader.pure'),
      import('@babylonjs/core/Meshes/instancedMesh.pure'),
    ]);
    RegisterInstancedMesh();
    RegisterGLTF2Loader();
    return new GLTFFileLoader(options[GLTFFileLoaderMetadata.name]);
  },
});
registerBuiltInGLTFExtensions();

/**
 * WebGPU engine options that let KTX2 textures stay block-compressed on the GPU. Babylon asks
 * for no optional device features by default, so the transcoder would expand every texture to
 * RGBA; it drops the ones the adapter doesn't have. (WebGL2 enables its extensions itself.)
 * Also timestamp queries: GPU time in the F3 overlay and the benchmark, measured only while
 * one of them is open (`BabylonRenderer.measureGpu`).
 */
export function compressedTextureFeatures(): { deviceDescriptor: GPUDeviceDescriptor } {
  return { deviceDescriptor: { requiredFeatures: ['texture-compression-bc', 'texture-compression-etc2', 'texture-compression-astc', 'timestamp-query'] } };
}

/**
 * Babylon logs WebGPU's uncaptured errors with a listener on the GPUDevice. It is added inside
 * `initAsync`, and `device.destroy()` does not remove listeners, so the closure keeps the engine
 * (and its renderer) alive after a preset change. Remember it here so `dispose` can take it off.
 */
type DeviceWithErrorListener = GPUDevice & { __llErrorListener?: EventListener };
if (typeof GPUDevice !== 'undefined') {
  const add = GPUDevice.prototype.addEventListener;
  GPUDevice.prototype.addEventListener = function (this: DeviceWithErrorListener, type: string, listener: EventListenerOrEventListenerObject | null, options?: boolean | AddEventListenerOptions) {
    if (type === 'uncapturederror' && listener && typeof listener === 'function') this.__llErrorListener = listener;
    return add.call(this, type, listener as EventListenerOrEventListenerObject, options);
  };
  // Every engine (ours, the previews') destroys its device: take the listener off there, whatever
  // disposed it, so the device's caches cannot keep the engine alive.
  const destroy = GPUDevice.prototype.destroy;
  GPUDevice.prototype.destroy = function (this: DeviceWithErrorListener) {
    const listener = this.__llErrorListener;
    if (listener) this.removeEventListener('uncapturederror', listener);
    delete this.__llErrorListener;
    return destroy.call(this);
  };
}

/** WebGPU had an adapter but failed to start once this session: later engines use WebGL2 straight away. */
let webGpuBroken = false;
export const webGpuFailed = (): boolean => webGpuBroken;
export function noteWebGpuFailed(): void {
  webGpuBroken = true;
}
