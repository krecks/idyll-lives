/**
 * ONNX Runtime Web for a worker: the language model runs on the GPU (WebGPU), the voices on
 * the GPU or, in Auto's fallback, on the processor (WASM). Its WASM binary comes from our own
 * download (`ml/assets.ts`), and it runs in the worker itself (no proxy worker).
 */

import wasmUrl from 'onnxruntime-web/ort-wasm-simd-threaded.asyncify.wasm?url';

export type OrtModule = typeof import('onnxruntime-web/webgpu');

/** The runtime's WASM binary, and its size in bytes (for progress bars). */
export const ORT_WASM = { url: wasmUrl, bytes: 28_747_738 };

/** The runtime's WebGPU build. */
export function importOrt(): Promise<OrtModule> {
  return import('onnxruntime-web/webgpu');
}

let hadWebGpu = false;

/**
 * Whether this browser offers WebGPU with an adapter (in a worker or a page). A yes is remembered;
 * a no is asked again (no adapter can be passing, as just after the graphics were reset).
 */
export async function hasWebGpu(): Promise<boolean> {
  if (hadWebGpu) return true;
  const gpu = (navigator as Navigator & { gpu?: { requestAdapter(): Promise<unknown> } }).gpu;
  if (!gpu) return false;
  try {
    return (hadWebGpu = !!(await gpu.requestAdapter()));
  } catch {
    return false;
  }
}

/** Where ONNX Runtime runs: the graphics card (WebGPU) or the processor (WASM). */
export type OrtDevice = 'gpu' | 'cpu';

/**
 * Threads for the runtime on the processor: two, so a line still comes faster than it plays, but
 * only beside the three the game keeps busy (the page, the simulation, the browser's compositor):
 * on four cores one, so speech doesn't take the game's.
 */
export function cpuThreads(cores = navigator.hardwareConcurrency ?? 1): number {
  return Math.min(2, Math.max(1, cores - 3));
}

/**
 * Sets the runtime up: its WASM binary (already downloaded), errors only in the log. On the GPU
 * one thread is enough (it does the work); on the processor `cpuThreads`, where the page can
 * share memory between threads.
 */
export function configureOrt(ort: OrtModule, wasmBinary: ArrayBuffer, device: OrtDevice = 'gpu'): void {
  ort.env.wasm.numThreads = device === 'cpu' && crossOriginIsolated ? cpuThreads() : 1;
  ort.env.wasm.proxy = false;
  ort.env.wasm.wasmPaths = { wasm: ORT_WASM.url };
  ort.env.wasm.wasmBinary = wasmBinary;
  ort.env.logLevel = 'error';
  // Ask for the discrete GPU on laptops that have one (the game does the same).
  ort.env.webgpu.powerPreference = 'high-performance';
}

/**
 * A session's options: the chosen device, full graph optimisation, and errors only in the log.
 * (A session logs warnings by default whatever `env.logLevel` says; it warns once that shape
 * calculations stay on the CPU, which ONNX Runtime does on purpose.) A GPU buffer output only
 * makes sense on the GPU.
 */
export function sessionOptions(device: OrtDevice = 'gpu', preferredOutputLocation?: 'cpu' | 'gpu-buffer') {
  return {
    executionProviders: [device === 'gpu' ? 'webgpu' : 'wasm'],
    graphOptimizationLevel: 'all' as const,
    logSeverityLevel: 3 as const,
    ...(device === 'gpu' && preferredOutputLocation ? { preferredOutputLocation } : {}),
  };
}
