/**
 * What a worker's model holds on the graphics card, for the benchmarks: every GPU buffer made in
 * this worker (ONNX Runtime's weights, activations and its buffer pool) counted as it is made and
 * destroyed, live and at its peak. Off unless a benchmark asks (`trackGpuBuffers` before the
 * runtime makes its device); the game never turns it on.
 */

let live = 0;
let peak = 0;
const sizes = new WeakMap<object, number>();

interface Prototypes {
  GPUDevice?: { prototype: { createBuffer(desc: { size: number }): object; __counted?: boolean } };
  GPUBuffer?: { prototype: { destroy(): void } };
}

/** Starts counting this worker's GPU buffers (once; a no-op without WebGPU). */
export function trackGpuBuffers(): void {
  const g = globalThis as unknown as Prototypes;
  const device = g.GPUDevice?.prototype;
  const buffer = g.GPUBuffer?.prototype;
  if (!device || !buffer || device.__counted) return;
  device.__counted = true;
  const create = device.createBuffer;
  device.createBuffer = function (this: unknown, desc: { size: number }) {
    const b = create.call(this, desc);
    sizes.set(b, desc.size);
    live += desc.size;
    peak = Math.max(peak, live);
    return b;
  };
  const destroy = buffer.destroy;
  buffer.destroy = function (this: object) {
    const n = sizes.get(this);
    if (n !== undefined) {
      live -= n;
      sizes.delete(this);
    }
    return destroy.call(this);
  };
}

/** Bytes in GPU buffers now and at most so far. */
export function gpuBuffers(): { live: number; peak: number } {
  return { live, peak };
}
