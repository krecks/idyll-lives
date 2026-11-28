/**
 * A dev check of WebGL2's texture units: a fragment shader may read at most `MAX_TEXTURE_IMAGE_UNITS` textures (16 on
 * the machines the game runs on), and a program past it fails to link ("17 of 16"), drawing nothing. Each program
 * PlayCanvas links is counted as it asks whether the link went through (`getProgramParameter(LINK_STATUS)`, which it
 * asks once a program, so nothing more is waited for): its active samplers, by name, and the game's defines it was
 * built with (`GAME_*`, what says which material it is). A program at the limit, or one past it that failed, is
 * logged; `report()` lists every program from the most samplers down (`window.__samplers()` in dev).
 *
 * A program that failed has no active uniforms to read: its fragment shader's declared samplers that the code reads
 * are counted instead.
 */

import * as pc from './engine';

interface SamplerCount {
  /** Its samplers (the active ones; a failed program's read in its fragment shader). */
  count: number;
  samplers: string[];
  /** The game's defines (`GAME_*`) in its fragment shader. */
  defines: string[];
  linked: boolean;
}

/** Counts every program linked on `device` from now on (WebGL2 only; nothing on WebGPU). */
export class SamplerCheck {
  private readonly counts: SamplerCount[] = [];
  private readonly undo: (() => void) | null = null;
  /** The fragment stage's limit (0 on WebGPU). */
  readonly limit: number = 0;

  constructor(device: pc.GraphicsDevice) {
    if (!(device instanceof pc.WebglGraphicsDevice)) return;
    const gl = device.gl as WebGL2RenderingContext;
    this.limit = gl.getParameter(gl.MAX_TEXTURE_IMAGE_UNITS) as number;
    const types = new Set<number>([
      gl.SAMPLER_2D,
      gl.SAMPLER_3D,
      gl.SAMPLER_CUBE,
      gl.SAMPLER_2D_SHADOW,
      gl.SAMPLER_2D_ARRAY,
      gl.SAMPLER_2D_ARRAY_SHADOW,
      gl.SAMPLER_CUBE_SHADOW,
      gl.INT_SAMPLER_2D,
      gl.INT_SAMPLER_3D,
      gl.INT_SAMPLER_CUBE,
      gl.INT_SAMPLER_2D_ARRAY,
      gl.UNSIGNED_INT_SAMPLER_2D,
      gl.UNSIGNED_INT_SAMPLER_3D,
      gl.UNSIGNED_INT_SAMPLER_CUBE,
      gl.UNSIGNED_INT_SAMPLER_2D_ARRAY,
    ]);
    const seen = new WeakSet<WebGLProgram>();
    const original = gl.getProgramParameter;
    const check = (program: WebGLProgram, linked: boolean) => {
      const fragment = (gl.getAttachedShaders(program) ?? []).find((s) => gl.getShaderParameter(s, gl.SHADER_TYPE) === gl.FRAGMENT_SHADER);
      const source = (fragment && gl.getShaderSource(fragment)) || '';
      const defines = [...new Set([...source.matchAll(/#define\s+(GAME_\w+)/g)].map((m) => m[1]))].sort();
      let samplers: string[] = [];
      if (linked) {
        const n = original.call(gl, program, gl.ACTIVE_UNIFORMS) as number;
        for (let i = 0; i < n; i++) {
          const info = gl.getActiveUniform(program, i);
          if (info && types.has(info.type)) samplers.push(info.name);
        }
      } else {
        // (Declared, and read somewhere past the declaration.)
        samplers = [...source.matchAll(/uniform\s+(?:(?:highp|mediump|lowp)\s+)?\w*sampler\w*\s+(\w+)\s*;/g)].map((m) => m[1]).filter((name) => source.split(new RegExp(`\\b${name}\\b`)).length > 2);
      }
      const entry: SamplerCount = { count: samplers.length, samplers, defines, linked };
      this.counts.push(entry);
      if (!linked || entry.count >= this.limit) console.warn(`[samplers] a program ${linked ? 'at' : 'failed, past'} WebGL2's ${this.limit} texture units: ${entry.count} (${defines.join(' ') || 'no game defines'}): ${samplers.join(', ')}`);
    };
    gl.getProgramParameter = function (this: WebGL2RenderingContext, program: WebGLProgram, pname: GLenum) {
      const value = original.call(gl, program, pname);
      if (pname === gl.LINK_STATUS && !seen.has(program)) {
        seen.add(program);
        check(program, !!value);
      }
      return value;
    } as typeof gl.getProgramParameter;
    this.undo = () => delete (gl as { getProgramParameter?: unknown }).getProgramParameter;
  }

  /** Every program counted, from the most samplers down. */
  report(): SamplerCount[] {
    return [...this.counts].sort((a, b) => b.count - a.count);
  }

  destroy(): void {
    this.undo?.();
  }
}
