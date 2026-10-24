/**
 * Buy-mode feedback in the world on PlayCanvas (ported from the Babylon renderer, removed: see git history up to f15b839b; what plays when,
 * the rings' spread, the pops' spring and the particles' simulation are `shared/buildFx.ts`'s):
 * the rings, sparkles and dust in one instanced quad draw (`shaders/buildFx.ts`), on the effects
 * layer, after the world's opaque draws and before its blended ones, as Babylon's particles were
 * (`layers.ts`); the pops scale placed objects' instances (`objectMotion.ts` `PopTarget`).
 *
 * Idle, the draw is off and nothing runs; while something plays, each game frame writes the
 * draw's instances (rings, then the sparkles, then the dust: Babylon's order) and sends them up
 * once. No allocations per frame.
 */

import * as pc from './engine';
import { Instances } from './instancing';
import { effectsLayer } from './layers';
import { writeMoving, type PopTarget } from './objectMotion';
import { BUILD_FX_PS, BUILD_FX_VS } from './shaders/buildFx';
import { animationRatio, BuildEffectsCore, DUST, PARTICLE, ParticleSim, RING_COLORS, SPARKLES, type ParticleKind, type RingState } from '../shared/buildFx';
import type { BuildEffect } from '../types';

/** Instances room is made for at first: every ring and particle there can be. */
const CAPACITY = 8 + SPARKLES.capacity + DUST.capacity;

/** The rings' colours (Babylon's emissive, gamma space), by effect. */
const RING_RGB = Object.fromEntries(Object.entries(RING_COLORS).map(([kind, hex]) => [kind, [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255)])) as Record<BuildEffect['kind'], number[]>;

export class BuildEffects extends BuildEffectsCore<PopTarget> {
  private readonly sparkles = new ParticleSim(SPARKLES);
  private readonly dust = new ParticleSim(DUST);
  private readonly draw: Instances;
  private readonly material: pc.ShaderMaterial;
  private readonly quad: pc.Mesh;
  private readonly textures: pc.Texture[];
  /** One instance's 16 floats (`shaders/buildFx.ts`), written in turn. */
  private readonly line = new Float32Array(16);

  constructor(app: pc.AppBase, root: pc.Entity, camera: pc.CameraComponent) {
    super();
    const device = app.graphicsDevice;
    this.material = new pc.ShaderMaterial({
      uniqueName: 'GameBuildFx',
      vertexGLSL: BUILD_FX_VS.glsl,
      fragmentGLSL: BUILD_FX_PS.glsl,
      vertexWGSL: BUILD_FX_VS.wgsl,
      fragmentWGSL: BUILD_FX_PS.wgsl,
      attributes: { vertex_position: pc.SEMANTIC_POSITION, instance_line1: pc.SEMANTIC_ATTR11, instance_line2: pc.SEMANTIC_ATTR12, instance_line3: pc.SEMANTIC_ATTR14 },
    });
    this.material.name = 'buildFx';
    this.material.blendType = pc.BLEND_PREMULTIPLIED;
    this.material.depthWrite = false;
    this.material.cull = pc.CULLFACE_NONE;
    // (Babylon's rings' `zOffset` -6; the camera-facing particles have no slope to take it.)
    this.material.slopeDepthBias = -6;
    this.textures = [ringCanvas(), sparkleCanvas(), puffCanvas()].map((canvas, i) => {
      const t = new pc.Texture(device, { name: ['buildRing', 'buildSparkle', 'buildPuff'][i], width: canvas.width, height: canvas.height, format: pc.PIXELFORMAT_RGBA8, mipmaps: true, minFilter: pc.FILTER_LINEAR_MIPMAP_LINEAR, magFilter: pc.FILTER_LINEAR, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE });
      t.setSource(canvas);
      return t;
    });
    this.material.setParameter('gameFxRing', this.textures[0]);
    this.material.setParameter('gameFxSparkle', this.textures[1]);
    this.material.setParameter('gameFxPuff', this.textures[2]);
    this.material.update();
    const quad = (this.quad = new pc.Mesh(device));
    quad.setPositions([0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0]);
    quad.setIndices([0, 1, 2, 0, 2, 3]);
    quad.update(pc.PRIMITIVE_TRIANGLES);
    // Spread over the lot (and its storeys): never culled.
    this.draw = new Instances(device, root, quad, this.material, { name: 'buildFx', castShadows: false, receiveShadows: false, layers: [effectsLayer(app, camera).id], capacity: CAPACITY, cull: false });
    this.draw.enabled = false;
  }

  /** Whether anything plays (a ring spreading, a pop, particles in the air). */
  get playing(): boolean {
    return this.busy || this.sparkles.alive || this.dust.alive;
  }

  /** Each game frame (`dtMs`): the rings and pops (`BuildEffectsCore.update`), the particles a step on, and the draw written. */
  frame(dtMs: number): void {
    if (!this.playing) return;
    this.update(dtMs);
    const ratio = animationRatio(dtMs);
    this.sparkles.step(ratio);
    this.dust.step(ratio);
    this.write();
  }

  dispose(): void {
    this.draw.destroy();
    this.quad.destroy();
    for (const t of this.textures) t.destroy();
    this.material.destroy();
  }

  protected placeRing(): void {
    // (Drawn from its state each frame: `write`.)
  }

  protected drawRing(): void {}

  protected emit(kind: ParticleKind, fx: BuildEffect, count: number, y0: number, y1: number): void {
    (kind === 'sparkle' ? this.sparkles : this.dust).emit(fx, count, y0, y1);
  }

  protected popWritten(target: PopTarget): void {
    const o = target.index * 16;
    for (const draw of target.meshes) {
      if (draw.destroyed) continue;
      draw.setMatrix(target.index, target.matrices, o);
      draw.update();
    }
    writeMoving(target);
  }

  /** The draw's instances: the rings shown, then the sparkles, then the dust; off with none. */
  private write(): void {
    const draw = this.draw;
    let n = 0;
    for (const r of this.rings) if (r.on) n++;
    n += this.sparkles.count + this.dust.count;
    draw.setCount(n);
    let i = 0;
    for (const r of this.rings) if (r.on) this.writeRing(i++, r);
    i = this.writeParticles(this.sparkles, 1, i);
    this.writeParticles(this.dust, 2, i);
    draw.update(false);
    if (draw.enabled !== n > 0) draw.enabled = n > 0;
  }

  /** A particle system's particles as instances from `i` on (`kind` 1 sparkles, 2 dust); the next instance's index. */
  private writeParticles(sim: ParticleSim, kind: number, i: number): number {
    const line = this.line;
    const d = sim.data;
    for (let p = 0; p < sim.count; p++) {
      const o = p * PARTICLE.stride;
      line[0] = d[o + PARTICLE.x];
      line[1] = d[o + PARTICLE.x + 1];
      line[2] = d[o + PARTICLE.x + 2];
      line[3] = d[o + PARTICLE.size];
      for (let k = 0; k < 4; k++) line[4 + k] = d[o + PARTICLE.color + k];
      line[8] = d[o + PARTICLE.angle];
      line[9] = kind;
      this.draw.setMatrix(i++, line);
    }
    return i;
  }

  private writeRing(i: number, r: RingState): void {
    const line = this.line;
    const rgb = RING_RGB[r.kind];
    line[0] = r.x;
    line[1] = r.y;
    line[2] = r.z;
    line[3] = r.scale;
    line[4] = rgb[0];
    line[5] = rgb[1];
    line[6] = rgb[2];
    line[7] = r.alpha;
    line[8] = 0;
    line[9] = 0;
    this.draw.setMatrix(i, line);
  }
}

/** A canvas `n` pixels square, drawn by `paint`. */
function canvasOf(n: number, paint: (ctx: CanvasRenderingContext2D) => void): HTMLCanvasElement {
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = n;
  paint(canvas.getContext('2d')!);
  return canvas;
}

/** The ring: a soft band near the edge (Babylon's `ringTexture`). */
function ringCanvas(): HTMLCanvasElement {
  const n = 256;
  return canvasOf(n, (ctx) => {
    const g = ctx.createRadialGradient(n / 2, n / 2, 0, n / 2, n / 2, n / 2);
    g.addColorStop(0, 'rgba(255,255,255,0)');
    g.addColorStop(0.5, 'rgba(255,255,255,0.12)');
    g.addColorStop(0.76, 'rgba(255,255,255,1)');
    g.addColorStop(0.9, 'rgba(255,255,255,0.85)');
    g.addColorStop(1, 'rgba(255,255,255,0)');
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, n, n);
  });
}

/** A soft four-pointed twinkle (Babylon's `sparkleTexture`). */
function sparkleCanvas(): HTMLCanvasElement {
  const n = 64;
  return canvasOf(n, (ctx) => {
    ctx.clearRect(0, 0, n, n);
    const glow = ctx.createRadialGradient(n / 2, n / 2, 0, n / 2, n / 2, n / 2);
    glow.addColorStop(0, 'rgba(255,255,255,1)');
    glow.addColorStop(0.25, 'rgba(255,255,255,0.5)');
    glow.addColorStop(1, 'rgba(255,255,255,0)');
    ctx.fillStyle = glow;
    ctx.fillRect(0, 0, n, n);
    ctx.fillStyle = 'rgba(255,255,255,0.95)';
    ctx.beginPath();
    const c = n / 2;
    ctx.moveTo(c, 2);
    ctx.quadraticCurveTo(c + 3, c - 3, n - 2, c);
    ctx.quadraticCurveTo(c + 3, c + 3, c, n - 2);
    ctx.quadraticCurveTo(c - 3, c + 3, 2, c);
    ctx.quadraticCurveTo(c - 3, c - 3, c, 2);
    ctx.fill();
  });
}

/** A soft puff (Babylon's `puffTexture`). */
function puffCanvas(): HTMLCanvasElement {
  const n = 64;
  return canvasOf(n, (ctx) => {
    ctx.clearRect(0, 0, n, n);
    const g = ctx.createRadialGradient(n / 2, n / 2, 0, n / 2, n / 2, n / 2);
    g.addColorStop(0, 'rgba(255,255,255,0.9)');
    g.addColorStop(0.5, 'rgba(255,255,255,0.45)');
    g.addColorStop(1, 'rgba(255,255,255,0)');
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, n, n);
  });
}
