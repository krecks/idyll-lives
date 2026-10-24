// Build mode's particles (`render/shared/buildFx.ts` `ParticleSim`, which the PlayCanvas renderer draws)
// against Babylon's own CPU particle system set up as the Babylon renderer sets it up
// (`render/babylon/buildFx.ts` `particleSystem`), on a null engine: the same random numbers, the same
// bursts, the same frames (varied lengths, a frame past a particle's death, a second burst while the
// first is in the air, more particles than fit) give the same particles, number for number, in the
// same order. Also: the rings' spread and the pop's spring of `BuildEffectsCore`.
import { describe, expect, test } from 'bun:test';
import { NullEngine } from '@babylonjs/core/Engines/nullEngine';
import { Vector3 } from '@babylonjs/core/Maths/math.vector';
import { ParticleSystem } from '@babylonjs/core/Particles/particleSystem';
import { Scene } from '@babylonjs/core/scene';
import { particleSystem } from '../src/render/babylon/buildFx';
import { animationRatio, BuildEffectsCore, Bursts, DUST, PARTICLE, ParticleSim, POP_TIME, RING_TIME, SPARKLES, type ParticleSettings, type PopInstance, type RingState } from '../src/render/shared/buildFx';
import type { BuildEffect } from '../src/render/types';

/** A fixed sequence of numbers in 0..1 (the same every run). */
function numbers(seed: number): () => number {
  let s = seed >>> 0;
  return () => (s = (Math.imul(s, 1664525) + 1013904223) >>> 0) / 2 ** 32;
}

/** Runs `fn` with `Math.random` drawing from `seed`. */
function seeded<T>(seed: number, fn: () => T): T {
  const random = Math.random;
  Math.random = numbers(seed);
  try {
    return fn();
  } finally {
    Math.random = random;
  }
}

const FX: BuildEffect = { kind: 'place', x: 3, z: 5, w: 2, d: 1, y: 2.8 };
/** Frames (ms), and a burst before some of them: [frame, count, y0, y1]. */
const FRAMES = [16.7, 16.7, 33.4, 8.3, 50, 16.7, 16.7, 100, 16.7, 16.7, 16.7, 25, 16.7, 16.7, 16.7, 16.7, 400, 16.7, 16.7, 400, 400, 400];
const BURSTS: [number, number, number, number][] = [
  [0, 40, 0.15, 0.7],
  [4, 22, 0.05, 0.4],
  [9, 300, 0.3, 0.8],
];

/** Each frame's particles as Babylon has them: position, colour, size, angle, per particle. */
function babylonRun(settings: ParticleSettings): number[][] {
  const engine = new NullEngine();
  const scene = new Scene(engine);
  const ps = particleSystem(scene, ParticleSystem, settings, null);
  const bursts = new Bursts();
  ps.emitter = Vector3.Zero();
  ps.startPositionFunction = (_world, position) => bursts.start(position);
  ps.start();
  const frames: number[][] = [];
  FRAMES.forEach((ms, f) => {
    for (const [at, count, y0, y1] of BURSTS) {
      if (at !== f) continue;
      bursts.add(FX, count, y0, y1);
      ps.manualEmitCount = Math.max(0, ps.manualEmitCount) + count;
    }
    // (`animate(true)` steps as a frame does, by `preWarmStepOffset` in place of the scene's animation ratio.)
    ps.preWarmStepOffset = animationRatio(ms);
    ps.animate(true);
    frames.push(ps.particles.flatMap((p) => [p.position.x, p.position.y, p.position.z, p.color.r, p.color.g, p.color.b, p.color.a, p.size * p.scale.x, p.angle].map(Math.fround)));
  });
  ps.dispose();
  scene.dispose();
  engine.dispose();
  return frames;
}

function simRun(settings: ParticleSettings): number[][] {
  const sim = new ParticleSim(settings);
  const frames: number[][] = [];
  FRAMES.forEach((ms, f) => {
    for (const [at, count, y0, y1] of BURSTS) if (at === f) sim.emit(FX, count, y0, y1);
    sim.step(animationRatio(ms));
    const d = sim.data;
    const out: number[] = [];
    for (let i = 0; i < sim.count; i++) {
      const o = i * PARTICLE.stride;
      out.push(d[o + PARTICLE.x], d[o + PARTICLE.x + 1], d[o + PARTICLE.x + 2], d[o + PARTICLE.color], d[o + PARTICLE.color + 1], d[o + PARTICLE.color + 2], d[o + PARTICLE.color + 3], d[o + PARTICLE.size], d[o + PARTICLE.angle]);
    }
    frames.push(out);
  });
  return frames;
}

describe('build effects', () => {
  for (const settings of [SPARKLES, DUST]) {
    test(`${settings.name}: the same particles as Babylon's particle system, frame for frame`, () => {
      const babylon = seeded(7, () => babylonRun(settings));
      const sim = seeded(7, () => simRun(settings));
      expect(sim.length).toBe(babylon.length);
      // (Something to compare: particles in the air, the capacity reached, all gone by the end.)
      expect(Math.max(...babylon.map((f) => f.length / 9))).toBe(settings.capacity);
      expect(babylon[babylon.length - 1].length).toBe(0);
      babylon.forEach((frame, f) => expect(sim[f]).toEqual(frame));
    });
  }

  test("the rings spread and fade, the pop springs as Babylon's did", () => {
    class Recorder extends BuildEffectsCore<PopInstance> {
      readonly drawn: string[] = [];
      protected placeRing(slot: number, r: RingState): void {
        this.drawn.push(`place ${slot} ${r.kind} ${r.x} ${r.y} ${r.z}`);
      }
      protected drawRing(slot: number, r: RingState): void {
        this.drawn.push(`ring ${slot} ${r.scale.toFixed(5)} ${r.alpha.toFixed(5)} ${r.on}`);
      }
      protected emit(kind: string, _fx: BuildEffect, count: number): void {
        this.drawn.push(`emit ${kind} ${count}`);
      }
      protected popWritten(t: PopInstance): void {
        this.drawn.push(`pop ${Array.from(t.matrices.subarray(16, 32), (v) => v.toFixed(5)).join(' ')}`);
      }
    }
    const fx = new Recorder();
    const target = { matrices: new Float32Array(32), index: 1 };
    target.matrices.set([0, 0, -1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 4.5, 2.8, 6, 1], 16);
    fx.play(FX);
    fx.play({ kind: 'sell', x: 0, z: 0, w: 1, d: 1 });
    fx.pop(target);
    let t = 0;
    while (fx.busy) {
      fx.update(16.7);
      t += 16.7;
    }
    expect(t).toBeGreaterThan(Math.max(RING_TIME, POP_TIME) * 1000);
    // At rest where it was placed.
    expect(Array.from(target.matrices.subarray(16, 32))).toEqual(Array.from(Float32Array.of(0, 0, -1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 4.5, 2.8, 6, 1)));
    expect(fx.drawn).toMatchSnapshot();
  });
});
