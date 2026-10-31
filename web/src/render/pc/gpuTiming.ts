/**
 * What the benchmark (`debug/bench.ts`) and the stats overlay measure of a PlayCanvas frame, with
 * the meanings they had up to 0.3.2, so results from before the renderer changed and after it stay
 * comparable:
 *
 * - GPU time (`GpuTiming`): PlayCanvas's GPU profiler (`device.gpuProfiler`). On WebGPU every
 *   render pass is stamped (the 'timestamp-query' feature), and the whole frame is from its first
 *   pass's start to its last pass's end (`spanPasses`): passes overlap, so their own times add up
 *   to more. Each pass is named by the target it draws into (`canvas`, `shadow`, `mirror0`,
 *   `scene`…; passes into one target add up). On WebGL2 the whole frame only
 *   (`EXT_disjoint_timer_query_webgl2`), no passes.
 * - Draw calls and triangles (`DrawCounts`): every draw of the frame, the sun's shadow map and the
 *   reflections included (`device._drawCallsPerFrame`), and those of the shadow map's passes and of
 *   the reflections' passes on their own. Triangles are counted per draw (index count / 3 ×
 *   instances), where results up to 0.3.2 summed the active meshes' indices: the same for meshes,
 *   and PlayCanvas's few full-screen triangles (post-processing) count too.
 * - Shader compiles (`ShaderWatch`): shaders made (a WebGL2 program compiled and linked at its first
 *   draw; a WebGPU shader module) and, on WebGPU, render pipelines made. They fill
 *   `device._shaderStats` (PlayCanvas's release build leaves it at zero) and, with `?diag=shaders`,
 *   each is logged with the time and what the renderer was doing (`phase`).
 * - Render targets' sizes (`colorBytes`, `depthBytes`): bytes per sample, for the passes' pixel × sample
 *   counts and memory.
 *
 * The wrappers are own properties on the device (and its profiler and pipeline cache) over
 * PlayCanvas 2.23's methods (pinned; `check:chunks` checks they're there), removed again by `destroy`.
 * Per frame: no allocations.
 */

import * as pc from './engine';

/** A pass's GPU time, averaged over the frames it was drawn in (ms). */
export interface PassTime {
  name: string;
  ms: number;
  frames: number;
}

/** PlayCanvas 2.23's GPU profiler (not exported by name). */
type Profiler = pc.GraphicsDevice['gpuProfiler'];

/** Frames in `GpuTiming.recentMs`'s average (about a second). */
const RECENT = 60;

export class GpuTiming {
  /** Whole frames measured so far, and the newest's time (ms). */
  frames = 0;
  ms = 0;
  private readonly recent = new Float32Array(RECENT);
  private readonly sums = new Map<string, { ms: number; frames: number }>();
  private readonly profiler: Profiler | null;
  private readonly undo: (() => void)[] = [];

  /** `label`: the name of a pass drawing into `target` (null: the canvas). */
  constructor(
    private readonly device: pc.GraphicsDevice,
    label: (target: pc.RenderTarget | null) => string,
  ) {
    // `gpuProfiler` is a GraphicsDevice field of PlayCanvas 2.23 (none on the null device, whatever its
    // type says); its `report` gets each frame's pass timings as they come back, a few frames late.
    const profiler: Profiler | null = (this.profiler = device.gpuProfiler ?? null);
    if (!profiler) return;
    const report = profiler.report;
    profiler.report = (version: number, timings: number[] | null, frameTime?: number) => {
      const known = profiler.pastFrameAllocations.has(version);
      report.call(profiler, version, timings, frameTime);
      if (known && timings?.length) this.reported(profiler);
    };
    this.undo.push(() => delete (profiler as { report?: unknown }).report);
    if (device instanceof pc.WebgpuGraphicsDevice) {
      // WebgpuGraphicsDevice's `setupTimeStampWrites(desc, name)` names each pass's two stamps while the
      // profiler is on (`startRenderPass` has set `renderTarget` by then; compute passes come without a
      // descriptor).
      const stamp = device.setupTimeStampWrites;
      device.setupTimeStampWrites = (desc: object | undefined, name: string): unknown => stamp.call(device, desc, desc && profiler.enabled ? label(device.renderTarget === device.backBuffer ? null : device.renderTarget) : name);
      this.undo.push(() => delete (device as { setupTimeStampWrites?: unknown }).setupTimeStampWrites);
    }
  }

  get available(): boolean {
    return !!this.profiler;
  }

  set enabled(on: boolean) {
    if (!this.profiler || this.profiler.enabled === on) return;
    this.profiler.enabled = on;
    this.sums.clear();
    this.recent.fill(0);
  }

  get enabled(): boolean {
    return !!this.profiler?.enabled;
  }

  /** The whole frame's GPU time over about the last second (ms; null: none measured yet). */
  get recentMs(): number | null {
    const n = Math.min(this.frames, RECENT);
    if (!this.enabled || n === 0) return null;
    let sum = 0;
    for (let i = 0; i < n; i++) sum += this.recent[i];
    return sum / n;
  }

  /** Each pass's average since the last reset (WebGPU; none on WebGL2, which times whole frames only). */
  passes(reset = false): PassTime[] {
    const out: PassTime[] = [];
    if (this.device.isWebGPU && this.enabled) for (const [name, s] of this.sums) out.push({ name, ms: s.ms / s.frames, frames: s.frames });
    if (reset) this.sums.clear();
    return out;
  }

  private reported(profiler: Profiler): void {
    const ms = profiler.frameTime;
    if (ms === undefined) return;
    this.recent[this.frames % RECENT] = ms;
    this.frames++;
    this.ms = ms;
    if (!this.device.isWebGPU) return;
    for (const [name, t] of profiler.passTimings) {
      const s = this.sums.get(name);
      if (s) {
        s.ms += t;
        s.frames++;
      } else this.sums.set(name, { ms: t, frames: 1 });
    }
  }

  destroy(): void {
    this.enabled = false;
    for (const undo of this.undo.splice(0)) undo();
  }
}

/** What a pass draws for the counts: 0 the rest, 1 the sun's shadow map, 2 a reflection. */
export type PassKind = 0 | 1 | 2;

/** Draw calls and triangles of a frame, in all, in the shadow map's passes and in the reflections' (see the file's header). */
export class DrawCounts {
  readonly draws = [0, 0, 0];
  readonly triangles = [0, 0, 0];
  private kind: PassKind = 0;
  private drawsAtStart = 0;
  private undo: (() => void)[] = [];

  constructor(private readonly device: pc.GraphicsDevice) {}

  /** Counts from now on (`kind`: what a pass into `target` draws). */
  install(kind: (target: pc.RenderTarget | null) => PassKind): void {
    if (this.undo.length) return;
    // Both of PlayCanvas 2.23's devices draw through `draw(primitive, indexBuffer, numInstances, …)`
    // between `startRenderPass(renderPass)` and `endRenderPass(renderPass)`; `_drawCallsPerFrame` counts
    // the draws made.
    const device = this.device;
    const { draw, startRenderPass, endRenderPass } = device;
    device.draw = (primitive, indexBuffer, instances, commands, first, last, firstInstance) => {
      const p = primitive.type;
      const n = p === pc.PRIMITIVE_TRIANGLES ? primitive.count / 3 : p === pc.PRIMITIVE_TRISTRIP || p === pc.PRIMITIVE_TRIFAN ? Math.max(0, primitive.count - 2) : 0;
      this.triangles[this.kind] += n * Math.max(1, instances ?? 1);
      // (The devices' own defaults for what isn't given.)
      draw.call(device, primitive, indexBuffer, instances, commands, first ?? true, last ?? true, firstInstance ?? 0);
    };
    device.startRenderPass = (pass: pc.RenderPass) => {
      startRenderPass.call(device, pass);
      this.kind = kind(pass.renderTarget ?? null);
      this.drawsAtStart = device._drawCallsPerFrame;
    };
    device.endRenderPass = (pass: pc.RenderPass) => {
      endRenderPass.call(device, pass);
      this.draws[this.kind] += device._drawCallsPerFrame - this.drawsAtStart;
      this.kind = 0;
    };
    this.undo.push(() => {
      const own = device as Partial<typeof device>;
      delete own.draw;
      delete own.startRenderPass;
      delete own.endRenderPass;
    });
  }

  remove(): void {
    for (const undo of this.undo.splice(0)) undo();
  }

  reset(): void {
    this.draws.fill(0);
    this.triangles.fill(0);
  }

  get allTriangles(): number {
    return this.triangles[0] + this.triangles[1] + this.triangles[2];
  }
}

/** PlayCanvas's own shader counters (`GraphicsDevice._shaderStats`). */
interface ShaderStats {
  vsCompiled: number;
  fsCompiled: number;
  linked: number;
  materialShaders: number;
  compileTime: number;
}

/** Shaders and pipelines made (see the file's header); `log`: each one to the console (`?diag=shaders`). */
export class ShaderWatch {
  shaders = 0;
  pipelines = 0;
  /** What the renderer is doing (logged with each compile): 'loading', 'warm-up', 'play'… */
  phase = 'loading';
  private readonly undo: (() => void)[] = [];
  private readonly started = performance.now();

  /** With the log: the material each shader was made for (`getShaderVariant`), to name it. */
  private readonly madeFor = new WeakMap<pc.Shader, string>();
  private making = '';

  constructor(
    device: pc.GraphicsDevice,
    private readonly log: boolean,
  ) {
    // SAFETY: PlayCanvas 2.23's `Shader` constructor makes its implementation through the device's
    // `createShaderImpl(shader)` (a WebGL2 program, compiled and linked at its first draw; WebGPU's
    // modules); WebGPU's render pipelines are made in `renderPipeline.create(…, shader, renderTarget, …)`
    // when the cache misses.
    const d = device as unknown as { createShaderImpl(shader: pc.Shader): unknown; _shaderStats: ShaderStats; renderPipeline?: { create(...args: unknown[]): unknown } };
    const stats = d._shaderStats;
    const create = d.createShaderImpl;
    d.createShaderImpl = (shader) => {
      this.shaders++;
      stats.vsCompiled++;
      stats.fsCompiled++;
      stats.linked++;
      if (log) {
        this.madeFor.set(shader, this.making);
        console.info(`[shaders] ${this.at()} shader ${this.shaders}: ${shader.name}${this.making ? ` for ${this.making}` : ''}`);
      }
      return create.call(d, shader);
    };
    this.undo.push(() => delete (d as { createShaderImpl?: unknown }).createShaderImpl);
    if (log) {
      // The material and pass a shader is made for: its classes' `getShaderVariant` (on the prototypes, so
      // every app's materials while this watch lives; only with `?diag=shaders`).
      for (const type of [pc.StandardMaterial, pc.ShaderMaterial]) {
        const proto: { getShaderVariant(this: pc.Material, params: { pass: number }): pc.Shader } = type.prototype;
        const own = Object.prototype.hasOwnProperty.call(proto, 'getShaderVariant');
        const variant = proto.getShaderVariant;
        const watch = this;
        proto.getShaderVariant = function (params) {
          watch.making = `${this.name || this.constructor.name} (pass ${params.pass})`;
          try {
            return variant.call(this, params);
          } finally {
            watch.making = '';
          }
        };
        this.undo.push(() => {
          if (own) proto.getShaderVariant = variant;
          else delete (proto as { getShaderVariant?: unknown }).getShaderVariant;
        });
      }
    }
    const cache = d.renderPipeline;
    if (cache) {
      const make = cache.create;
      cache.create = (...args) => {
        this.pipelines++;
        if (log) {
          const shader = args[2] as pc.Shader | undefined;
          const target = args[3] as pc.RenderTarget | undefined;
          const material = shader && this.madeFor.get(shader);
          const blend = args[5] as { blend?: boolean } | undefined;
          const depth = args[6] as { write?: boolean; func?: number } | undefined;
          const layout = (args[7] as { arrayStride: number; attributes: { format: string; offset: number }[] }[] | undefined)?.map((b) => `${b.arrayStride}:${b.attributes.map((a) => `${a.format}@${a.offset}`).join(',')}`).join(' + ');
          console.info(`[shaders] ${this.at()} pipeline ${this.pipelines}: ${shader?.name}${material ? ` for ${material}` : ''} → ${target?.name ?? '?'} (${target?.samples ?? 1}×, ${String(args[0])}, cull ${String(args[8])}, front ${String(args[12])}, blend ${blend?.blend ? 1 : 0}, depth ${depth?.write ? 'w' : '-'}${String(depth?.func)}, vertices ${layout})`);
        }
        return make.apply(cache, args);
      };
      this.undo.push(() => delete (cache as { create?: unknown }).create);
    }
  }

  /** A line in the log (`?diag=shaders`), with the counts so far. */
  note(text: string): void {
    if (this.log) console.info(`[shaders] ${this.at()} ${text} (${this.shaders} shaders, ${this.pipelines} pipelines so far)`);
  }

  private at(): string {
    return `${((performance.now() - this.started) / 1000).toFixed(1)} s, ${this.phase}:`;
  }

  destroy(): void {
    for (const undo of this.undo.splice(0)) undo();
  }
}

/** Bytes per sample of `target`'s colour (its first colour buffer; 0: none). */
export function colorBytes(target: pc.RenderTarget): number {
  const color = target.colorBuffer;
  return color ? (pc.pixelFormatInfo.get(color.format)?.size ?? 4) : 0;
}

/** Bytes per sample of `target`'s depth (and stencil; 0: none). */
export function depthBytes(target: pc.RenderTarget): number {
  const depth = target.depthBuffer;
  if (depth) return pc.pixelFormatInfo.get(depth.format)?.size ?? 4;
  return target.depth || target.stencil ? 4 : 0;
}
