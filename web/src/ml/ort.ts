/**
 * ONNX Runtime Web for a worker, on the CPU (its WASM build with SIMD) or the GPU (its WebGPU
 * build). Shared by the voice and language model workers: the runtime is imported only for the
 * device asked for, its WASM binary comes from our own download (`ml/assets.ts`), and it runs
 * in the worker itself (no proxy worker).
 */

import cpuWasmUrl from 'onnxruntime-web/ort-wasm-simd-threaded.wasm?url';
import gpuWasmUrl from 'onnxruntime-web/ort-wasm-simd-threaded.asyncify.wasm?url';

export type Device = 'cpu' | 'gpu';
export type OrtModule = typeof import('onnxruntime-web/wasm');

/** The runtime's WASM binary for each device, and its size in bytes (for progress bars). */
export const ORT_WASM: Record<Device, { url: string; bytes: number }> = {
  cpu: { url: cpuWasmUrl, bytes: 14_239_897 },
  gpu: { url: gpuWasmUrl, bytes: 26_781_914 },
};

/** The runtime build for a device. */
export async function importOrt(device: Device): Promise<OrtModule> {
  return device === 'gpu' ? ((await import('onnxruntime-web/webgpu')) as unknown as OrtModule) : await import('onnxruntime-web/wasm');
}

/** Whether this browser offers WebGPU with an adapter (in a worker or a page). */
export async function hasWebGpu(): Promise<boolean> {
  const gpu = (navigator as Navigator & { gpu?: { requestAdapter(): Promise<unknown> } }).gpu;
  if (!gpu) return false;
  try {
    return !!(await gpu.requestAdapter());
  } catch {
    return false;
  }
}

/**
 * Threads for a model that wants more than one: up to four, leaving two cores for the game and
 * the simulation. Threads need cross-origin isolation (SharedArrayBuffer); without it, one.
 */
export function ortThreads(): number {
  if (!self.crossOriginIsolated) return 1;
  return Math.max(1, Math.min(4, (navigator.hardwareConcurrency ?? 2) - 2));
}

/** Sets the runtime up: threads, its WASM binary (already downloaded), errors only in the log. */
export function configureOrt(ort: OrtModule, device: Device, threads: number, wasmBinary: ArrayBuffer): void {
  ort.env.wasm.numThreads = threads;
  ort.env.wasm.proxy = false;
  ort.env.wasm.wasmPaths = { wasm: ORT_WASM[device].url };
  ort.env.wasm.wasmBinary = wasmBinary;
  ort.env.logLevel = 'error';
}

/**
 * A session's options for a device: its execution provider, full graph optimisation, and
 * errors only in the log. (A session logs warnings by default whatever `env.logLevel` says; on
 * the GPU it warns once that shape calculations stay on the CPU, which ONNX Runtime does on
 * purpose.)
 */
export function sessionOptions(device: Device) {
  return {
    executionProviders: device === 'gpu' ? ['webgpu'] : ['wasm'],
    graphOptimizationLevel: 'all' as const,
    logSeverityLevel: 3 as const,
  };
}
