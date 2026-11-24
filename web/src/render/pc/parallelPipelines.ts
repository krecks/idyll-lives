/**
 * The warm-up's pipelines made side by side (WebGPU). PlayCanvas makes a render pipeline in the frame that first
 * draws with it, with the device's `createRenderPipeline`: the browser's GPU process compiles each in turn before it
 * draws again, and on a machine that has never compiled the game's shaders (a player's first game; the browser's
 * shader cache emptied by an update) the warm-up frame's 160 or so took 50 s in a row, frames withheld throughout
 * (2026-11-24, M3 Pro, Chrome 156, Metal). Made with `createRenderPipelineAsync`, all at once, the browser compiles
 * them on its worker threads: the same 160, cold, 9.9 s against 69.6 s in turn.
 *
 * `ParallelPipelines.frame(draw)` draws one frame (`draw`) with every pipeline the cache misses made that way: the
 * cache keeps a stand-in for it, a draw with a stand-in is left out of the frame (as PlayCanvas leaves out a draw
 * whose shader isn't ready), and once all are made the stand-ins are swapped for them in the cache. It says how many
 * it made: the caller draws the frame again, in full. With a warm cache each is made at once; on WebGL2, or a
 * browser without `createRenderPipelineAsync`, it only draws the frame.
 */

import type * as pc from './engine';

/** PlayCanvas 2.23's render pipeline cache (`WebgpuRenderPipeline`, `device.renderPipeline`; pinned, `check:chunks`). */
interface PipelineCache {
  create(...args: unknown[]): unknown;
  cache: Map<number, { pipeline: unknown }[]>;
}

/** The pass encoder's methods a stand-in must not reach (it is not a GPU object). */
const DRAWS = ['draw', 'drawIndexed', 'drawIndirect', 'drawIndexedIndirect'] as const;

export class ParallelPipelines {
  private readonly gpu: GPUDevice | null;
  private readonly pipelines: PipelineCache | null;
  /** The device's current pipeline (`WebgpuGraphicsDevice.pipeline`), set again by the draw that changes it. */
  private readonly current: { pipeline: unknown };

  constructor(device: pc.GraphicsDevice) {
    // SAFETY: on WebGPU, PlayCanvas 2.23's device keeps its `GPUDevice` in `wgpu` and its pipeline cache in
    // `renderPipeline` (`get` → `create` on a miss → `wgpu.createRenderPipeline(desc)`; tools/render/chunks.json).
    const d = device as unknown as { isWebGPU: boolean; wgpu?: GPUDevice; renderPipeline?: PipelineCache; pipeline: unknown };
    this.current = d;
    const usable = d.isWebGPU && !!d.wgpu && typeof d.wgpu.createRenderPipelineAsync === 'function' && !!d.renderPipeline?.cache;
    this.gpu = usable ? d.wgpu! : null;
    this.pipelines = usable ? d.renderPipeline! : null;
  }

  /** Draws a frame (`draw` resolves once it is drawn) with its new pipelines made side by side; how many it made. */
  async frame(draw: () => Promise<void>): Promise<number> {
    const gpu = this.gpu;
    const cache = this.pipelines;
    if (!gpu || !cache) {
      await draw();
      return 0;
    }
    const standIns = new WeakSet<object>();
    const made = new Map<object, GPURenderPipeline>();
    const jobs: Promise<void>[] = [];
    // (Whatever `create` is now: the shader watch's count wraps it, `gpuTiming.ts`.)
    const ownCreate = Object.prototype.hasOwnProperty.call(cache, 'create');
    const create = cache.create;
    cache.create = (...args: unknown[]) => {
      let desc: GPURenderPipelineDescriptor | null = null;
      const standIn = {};
      const g = gpu as unknown as { createRenderPipeline: (d: GPURenderPipelineDescriptor) => unknown };
      g.createRenderPipeline = (d) => {
        desc = d;
        return standIn;
      };
      try {
        create.apply(cache, args);
      } finally {
        delete (g as { createRenderPipeline?: unknown }).createRenderPipeline;
      }
      if (!desc) return create.apply(cache, args);
      const asked: GPURenderPipelineDescriptor = desc;
      standIns.add(standIn);
      jobs.push(
        gpu.createRenderPipelineAsync(asked).then(
          (p) => void made.set(standIn, p),
          // (Refused: made as before, so the error is the device's as it would have been.)
          () => void made.set(standIn, gpu.createRenderPipeline(asked)),
        ),
      );
      return standIn;
    };
    const undoDraws = skipStandIns(standIns);
    try {
      try {
        await draw();
      } finally {
        if (ownCreate) cache.create = create;
        else delete (cache as { create?: unknown }).create;
      }
      await Promise.all(jobs);
      for (const entries of cache.cache.values()) {
        for (const e of entries) {
          const p = made.get(e.pipeline as object);
          if (p) e.pipeline = p;
        }
      }
      // (The device's current pipeline may be a stand-in: the next draw asks the cache again.)
      if (standIns.has(this.current.pipeline as object)) this.current.pipeline = null;
    } finally {
      undoDraws();
    }
    return jobs.length;
  }
}

/** Until undone, a pass encoder given a stand-in (`standIns`) leaves out its draws until it is given a pipeline. */
function skipStandIns(standIns: WeakSet<object>): () => void {
  const proto = GPURenderPassEncoder.prototype as unknown as Record<string, (...args: unknown[]) => unknown>;
  const skipping = new WeakSet<object>();
  const setPipeline = proto.setPipeline;
  proto.setPipeline = function (this: object, pipeline: unknown) {
    if (standIns.has(pipeline as object)) {
      skipping.add(this);
      return;
    }
    skipping.delete(this);
    return setPipeline.call(this, pipeline);
  };
  const draws = DRAWS.map((name) => {
    const fn = proto[name];
    proto[name] = function (this: object, ...args: unknown[]) {
      if (!skipping.has(this)) return fn.apply(this, args);
    };
    return [name, fn] as const;
  });
  return () => {
    proto.setPipeline = setPipeline;
    for (const [name, fn] of draws) proto[name] = fn;
  };
}
