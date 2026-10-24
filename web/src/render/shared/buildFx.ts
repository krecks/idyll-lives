// `ParticleSim` is ported from Babylon.js 9.30's CPU particle system (`thinParticleSystem.ts`: its
// creation and update queues, gradients and random draws). Changed for Idyll Lives: only what the
// game's sparkles and dust use, on the game's own particle records.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
/**
 * Buy-mode feedback in the world, engine-neutral (each renderer's `buildFx.ts` draws it): a ring
 * that spreads over the footprint, sparkles for new and upgraded furniture, dust for walls going up
 * or coming down and a puff for things sold; and the "pop" that springs newly placed furniture into
 * place (its instance matrix is scaled for a moment). Everything is pooled and costs nothing when
 * idle.
 *
 * - `BuildEffectsCore`: which effect plays what (`play`), the rings' pool and how they spread and
 *   fade, the pops' spring, and the bursts the particles start in (`Bursts`). An engine says how a
 *   ring is drawn, how particles are emitted and how a popped instance goes up.
 * - The particles are Babylon's CPU `ParticleSystem` as the game set it up (`SPARKLES`, `DUST`):
 *   the Babylon renderer still runs Babylon's; `ParticleSim` is the same simulation, step for step
 *   (Babylon.js 9.30 `thinParticleSystem`: its creation and update queues, in their order, drawing
 *   the same random numbers in the same order), for an engine that draws the particles itself
 *   (PlayCanvas: one instanced quad draw, `pc/buildFx.ts`). `tests/buildFx.test.ts` runs both.
 */

import type { BuildEffect } from '../types';

/** Seconds a ring takes to spread and fade. */
export const RING_TIME = 0.65;
/** Rings at most (the oldest is taken again). */
export const RING_POOL = 8;
/** Seconds the pop takes to settle. */
export const POP_TIME = 0.6;
/** Rings lie this far (m) above the floor. */
export const RING_LIFT = 0.045;

/** A ring's colour per effect (sRGB, as Babylon's emissive colour). */
export const RING_COLORS: Record<BuildEffect['kind'], string> = {
  place: '#FFC94D',
  upgrade: '#FFB020',
  sell: '#D9DEE8',
  build: '#F1E3C8',
  remove: '#E8D9D3',
};

export type ParticleKind = 'sparkle' | 'dust';

/** One ring of the pool: where it lies (world space), how big it spreads, and how far it is. */
export interface RingState {
  kind: BuildEffect['kind'];
  x: number;
  y: number;
  z: number;
  /** The footprint's larger side (tiles). */
  size: number;
  age: number;
  /** As drawn now: its width (m, a square), its alpha (Babylon's `visibility`), and whether it's drawn. */
  scale: number;
  alpha: number;
  on: boolean;
}

/** What a pop needs of a placed object's instance: where its matrix lives (16 floats at `index * 16`). */
export interface PopInstance {
  matrices: Float32Array;
  index: number;
}

/** Particles still to emit inside a box (world space). */
interface Burst {
  x0: number;
  y0: number;
  z0: number;
  x1: number;
  y1: number;
  z1: number;
  left: number;
}

/**
 * The bursts queued on a particle system, in order: each new particle starts in the first one with
 * particles left (Babylon's `startPositionFunction`), three random numbers for x, y and z.
 */
export class Bursts {
  private readonly queue: Burst[] = [];

  /** Queues `count` particles inside the effect's footprint, between heights `y0` and `y1` above its floor. */
  add(fx: BuildEffect, count: number, y0: number, y1: number): void {
    const base = fx.y ?? 0;
    this.queue.push({ x0: fx.x, y0: base + y0, z0: fx.z, x1: fx.x + fx.w, y1: base + y1, z1: fx.z + fx.d, left: count });
  }

  /** Where the next particle starts (`out` x, y, z; far below the world if nothing is queued). */
  start(out: { x: number; y: number; z: number }): void {
    const queue = this.queue;
    while (queue.length && queue[0].left <= 0) queue.shift();
    const b = queue[0];
    if (!b) {
      out.x = 0;
      out.y = -50;
      out.z = 0;
      return;
    }
    b.left--;
    out.x = b.x0 + Math.random() * (b.x1 - b.x0);
    out.y = b.y0 + Math.random() * (b.y1 - b.y0);
    out.z = b.z0 + Math.random() * (b.z1 - b.z0);
  }
}

interface Pop<T extends PopInstance> {
  target: T;
  base: Float32Array;
  age: number;
}

/**
 * The effects' state and timing; an engine draws it. `T`: its placed objects' instances (with
 * their draws), `play`ed and `pop`ped from the renderer's `buildEffect`, `update`d each game frame.
 */
export abstract class BuildEffectsCore<T extends PopInstance> {
  /** The rings' pool (`RING_POOL` at most), each drawn by the engine in its slot. */
  protected readonly rings: RingState[] = [];
  private readonly pops: Pop<T>[] = [];
  /** Reduce motion: rings fade where they are (no spreading), no sparkles or dust, no pop. */
  reducedMotion = false;

  /** A ring starts in slot `slot` (made the first time it's used, `rings[slot]` is new): its colour and place are set. */
  protected abstract placeRing(slot: number, ring: RingState): void;
  /** A ring in slot `slot` spread on (`ring.scale`, `ring.alpha`), or is done (`ring.on` false). */
  protected abstract drawRing(slot: number, ring: RingState): void;
  /** `count` particles of `kind` inside the effect's footprint, between heights `y0` and `y1` above its floor. */
  protected abstract emit(kind: ParticleKind, fx: BuildEffect, count: number, y0: number, y1: number): void;
  /** A pop wrote its instance's matrix: it goes up (with its moving parts). */
  protected abstract popWritten(target: T): void;
  play(fx: BuildEffect): void {
    const cx = fx.x + fx.w / 2;
    const cz = fx.z + fx.d / 2;
    const size = Math.max(fx.w, fx.d);
    if (this.reducedMotion) {
      if (fx.kind === 'place' || fx.kind === 'upgrade' || fx.kind === 'sell') this.ring(fx.kind, cx, cz, size, fx.y ?? 0);
      return;
    }
    switch (fx.kind) {
      case 'place':
      case 'upgrade':
        this.ring(fx.kind, cx, cz, size, fx.y ?? 0);
        this.emit('sparkle', fx, fx.kind === 'upgrade' ? 56 : 40, 0.15, fx.kind === 'upgrade' ? 1.2 : 0.7);
        break;
      case 'sell':
        this.ring(fx.kind, cx, cz, size, fx.y ?? 0);
        this.emit('dust', fx, 22, 0.05, 0.4);
        // A little glitter: the money comes back.
        this.emit('sparkle', fx, 12, 0.3, 0.8);
        break;
      default:
        // Walls going up or coming down: dust all along their height.
        this.emit('dust', fx, 7, 0.05, 2.2);
    }
  }

  /** Springs a just-placed object into place: it grows from nothing, overshoots and settles. */
  pop(target: T): void {
    if (this.reducedMotion) return;
    const o = target.index * 16;
    if (o + 16 > target.matrices.length) return;
    const base = target.matrices.slice(o, o + 16);
    const pop = { target, base, age: 0 };
    this.pops.push(pop);
    this.writePop(pop, 0);
  }

  /** Forgets pops of a world that was rebuilt (its buffers are gone). */
  clearPops(): void {
    this.pops.length = 0;
  }

  get busy(): boolean {
    if (this.pops.length > 0) return true;
    for (const r of this.rings) if (r.age < RING_TIME) return true;
    return false;
  }

  update(dtMs: number): void {
    const dt = Math.min(dtMs, 50) / 1000;
    for (let slot = 0; slot < this.rings.length; slot++) {
      const r = this.rings[slot];
      if (r.age >= RING_TIME) continue;
      r.age += dt;
      const t = Math.min(1, r.age / RING_TIME);
      const ease = this.reducedMotion ? 1 : 1 - (1 - t) ** 3;
      r.scale = r.size * (0.55 + 0.95 * ease) + 0.4;
      r.alpha = (1 - t) ** 1.6;
      if (t >= 1) r.on = false;
      this.drawRing(slot, r);
    }
    for (let i = this.pops.length - 1; i >= 0; i--) {
      const p = this.pops[i];
      p.age += dt;
      const done = p.age >= POP_TIME;
      this.writePop(p, done ? POP_TIME : p.age);
      if (done) this.pops.splice(i, 1);
    }
  }

  /** A ring from the pool (a free one, a new one, or the oldest), started over the footprint's middle. */
  private ring(kind: BuildEffect['kind'], cx: number, cz: number, size: number, y: number): void {
    let slot = this.rings.findIndex((r) => r.age >= RING_TIME);
    if (slot < 0) {
      if (this.rings.length >= RING_POOL) slot = this.rings.indexOf(this.rings.reduce((a, b) => (a.age > b.age ? a : b)));
      else {
        slot = this.rings.length;
        this.rings.push({ kind, x: 0, y: 0, z: 0, size: 1, age: RING_TIME, scale: 1, alpha: 1, on: false });
      }
    }
    const ring = this.rings[slot];
    Object.assign(ring, { kind, x: cx, y: y + RING_LIFT, z: cz, size, age: 0, on: true });
    this.placeRing(slot, ring);
  }

  /**
   * Damped spring from 0 to 1: grows in, overshoots by ~20% and settles. Squash and stretch on y.
   * The scale goes before the placement (Babylon's `Matrix.Scaling(xz, y, xz).multiply(base)`: its
   * rows scaled, in single precision as Babylon's matrices are).
   */
  private writePop(p: Pop<T>, age: number): void {
    const done = age >= POP_TIME;
    const xz = Math.fround(done ? 1 : Math.max(0.001, popSpring(age)));
    const y = Math.fround(done ? 1 : Math.max(0.001, popSpring(age - 0.035)));
    const { matrices, index } = p.target;
    const o = index * 16;
    for (let k = 0; k < 16; k++) matrices[o + k] = p.base[k] * (k < 4 || (k >= 8 && k < 12) ? xz : k < 8 ? y : 1);
    this.popWritten(p.target);
  }
}

/** The pop's spring at `s` seconds. */
export function popSpring(s: number): number {
  return s <= 0 ? 0 : 1 - Math.exp(-7 * s) * Math.cos(13 * s);
}

// --- the particles -------------------------------------------------------------------

type RGBA = readonly [number, number, number, number];
type XYZ = readonly [number, number, number];

/** A particle system as the game sets Babylon's up (its `ParticleSystem` properties of the same names). */
export interface ParticleSettings {
  name: string;
  capacity: number;
  /** Babylon's `BLENDMODE_ADD` (added, times its alpha) or `BLENDMODE_STANDARD` (blended over). */
  blend: 'add' | 'standard';
  color1: RGBA;
  color2: RGBA;
  colorDead: RGBA;
  minSize: number;
  maxSize: number;
  /** Size over the particle's life (`addSizeGradient`, in order): replaces min/max size. */
  sizeGradients?: readonly (readonly [number, number])[];
  minLifeTime: number;
  maxLifeTime: number;
  direction1: XYZ;
  direction2: XYZ;
  minEmitPower: number;
  maxEmitPower: number;
  gravity: XYZ;
  minAngularSpeed: number;
  maxAngularSpeed: number;
  updateSpeed: number;
}

/** Twinkles rising from what was placed, falling back as they fade. */
export const SPARKLES: ParticleSettings = {
  name: 'buildSparkles',
  capacity: 256,
  blend: 'add',
  color1: [1, 0.88, 0.45, 1],
  color2: [1, 0.98, 0.85, 1],
  colorDead: [1, 1, 1, 0],
  minSize: 0.1,
  maxSize: 0.28,
  minLifeTime: 0.5,
  maxLifeTime: 1.05,
  direction1: [-0.6, 2.6, -0.6],
  direction2: [0.6, 4.2, 0.6],
  minEmitPower: 0.5,
  maxEmitPower: 0.9,
  gravity: [0, -3.2, 0],
  minAngularSpeed: -2,
  maxAngularSpeed: 2,
  updateSpeed: 1 / 60,
};

/** Puffs of dust drifting out and up, growing as they fade. */
export const DUST: ParticleSettings = {
  name: 'buildDust',
  capacity: 256,
  blend: 'standard',
  color1: [0.88, 0.84, 0.76, 0.55],
  color2: [0.8, 0.8, 0.82, 0.45],
  colorDead: [0.85, 0.85, 0.85, 0],
  minSize: 0.25,
  maxSize: 0.55,
  sizeGradients: [
    [0, 0.5],
    [1, 1.6],
  ],
  minLifeTime: 0.55,
  maxLifeTime: 1.0,
  direction1: [-1, 0.6, -1],
  direction2: [1, 1.4, 1],
  minEmitPower: 0.3,
  maxEmitPower: 0.8,
  gravity: [0, 0.2, 0],
  minAngularSpeed: -1,
  maxAngularSpeed: 1,
  updateSpeed: 1 / 60,
};

/** Babylon's scene animation ratio for a frame of `ms` (its `getAnimationRatio`). */
export function animationRatio(ms: number): number {
  return ms * (60.0 / 1000.0);
}

/** Babylon's `RandomRange`: no random number drawn when the range is empty. */
function randomRange(min: number, max: number): number {
  return min === max ? min : Math.random() * (max - min) + min;
}

/** Floats per particle in `ParticleSim.data`. */
const P = 20;
// Offsets: position, direction, colour, colour step, age, life, size, angle, angular speed.
const PX = 0;
const DX = 3;
const CR = 6;
const SR = 10;
const AGE = 14;
const LIFE = 15;
const SIZE = 16;
const ANGLE = 17;
const SPIN = 18;

/**
 * Babylon's CPU particle system for one `ParticleSettings` (a box emitter at the origin, world
 * space, billboards, no sheet, noise, drag or ramps), its particles in one array (`data`, `P`
 * floats each, `count` alive, in Babylon's order: one that dies takes the last one's place).
 * Allocation-free once made.
 */
export class ParticleSim {
  readonly data: Float32Array;
  count = 0;
  /** Particles to make at the next step (Babylon's `manualEmitCount`). */
  private pending = 0;
  readonly bursts = new Bursts();
  private readonly start = { x: 0, y: 0, z: 0 };
  /** (Babylon keeps particles in doubles: so does the simulation, and `data` takes them as drawn.) */
  private readonly state: Float64Array;

  constructor(readonly settings: ParticleSettings) {
    this.state = new Float64Array(settings.capacity * P);
    this.data = new Float32Array(settings.capacity * P);
  }

  /** Queues `count` particles in the effect's footprint (`Bursts.add`), made at the next step. */
  emit(fx: BuildEffect, count: number, y0: number, y1: number): void {
    this.bursts.add(fx, count, y0, y1);
    this.pending += count;
  }

  get alive(): boolean {
    return this.count > 0 || this.pending > 0;
  }

  /**
   * One step (a frame): the living particles move on (those whose life is over go), then those
   * queued are made. `data` then has them as drawn. `ratio`: Babylon's animation ratio, the frame's
   * milliseconds times 60 / 1000 (`animationRatio`); the step is `updateSpeed` times it, worked out
   * as Babylon does.
   */
  step(ratio: number): void {
    const s = this.state;
    const set = this.settings;
    const seconds = set.updateSpeed * ratio;
    const g = set.gravity;
    const gx = g[0];
    const gy = g[1];
    const gz = g[2];
    const grads = set.sizeGradients;
    for (let i = 0; i < this.count; i++) {
      const o = i * P;
      let speed = seconds;
      const before = s[o + AGE];
      s[o + AGE] += speed;
      // The step to its death is cut short there.
      if (s[o + AGE] > s[o + LIFE]) {
        const diff = s[o + AGE] - before;
        speed = ((s[o + LIFE] - before) * speed) / diff;
        s[o + AGE] = s[o + LIFE];
      }
      const ratio = s[o + AGE] / s[o + LIFE];
      // (Babylon's update queue: colour, angle, direction, position, gravity, then the size gradients.)
      for (let k = 0; k < 4; k++) s[o + CR + k] += s[o + SR + k] * seconds;
      if (s[o + CR + 3] < 0) s[o + CR + 3] = 0;
      s[o + ANGLE] += s[o + SPIN] * seconds;
      for (let k = 0; k < 3; k++) s[o + PX + k] += s[o + DX + k] * speed;
      s[o + DX] += gx * speed;
      s[o + DX + 1] += gy * speed;
      s[o + DX + 2] += gz * speed;
      if (grads) s[o + SIZE] = sizeAt(grads, ratio);
      if (s[o + AGE] >= s[o + LIFE]) {
        // Gone: the last one takes its place (and moves on now, if it hasn't yet).
        const last = (this.count - 1) * P;
        if (last !== o) s.copyWithin(o, last, last + P);
        this.count--;
        i--;
      }
    }
    for (; this.pending > 0; this.pending--) {
      if (this.count === set.capacity) {
        this.pending = 0;
        break;
      }
      this.create(this.count * P);
      this.count++;
    }
    for (let i = 0, n = this.count * P; i < n; i++) this.data[i] = s[i];
  }

  /** A new particle at `o`, Babylon's creation queue in order: life and power, position, direction, size, angle, colour. */
  private create(o: number): void {
    const s = this.state;
    const set = this.settings;
    s[o + AGE] = 0;
    s[o + LIFE] = randomRange(set.minLifeTime, set.maxLifeTime);
    const power = randomRange(set.minEmitPower, set.maxEmitPower);
    this.bursts.start(this.start);
    s[o + PX] = this.start.x;
    s[o + PX + 1] = this.start.y;
    s[o + PX + 2] = this.start.z;
    const d1 = set.direction1;
    const d2 = set.direction2;
    const dx = randomRange(d1[0], d2[0]);
    const dy = randomRange(d1[1], d2[1]);
    const dz = randomRange(d1[2], d2[2]);
    // (Babylon's emit power 0 keeps the direction aside; the game's are never 0.)
    s[o + DX] = dx * power;
    s[o + DX + 1] = dy * power;
    s[o + DX + 2] = dz * power;
    // (Its scale is 1 by 1: Babylon draws no random numbers for it.)
    s[o + SIZE] = set.sizeGradients ? set.sizeGradients[0][1] : randomRange(set.minSize, set.maxSize);
    s[o + SPIN] = randomRange(set.minAngularSpeed, set.maxAngularSpeed);
    s[o + ANGLE] = 0;
    const t = randomRange(0, 1);
    const c1 = set.color1;
    const c2 = set.color2;
    const dead = set.colorDead;
    for (let k = 0; k < 4; k++) {
      // (Babylon's `Color4.LerpToRef`, then the step to the dead colour over its life.)
      const c = c1[k] + (c2[k] - c1[k]) * t;
      s[o + CR + k] = c;
      s[o + SR + k] = (dead[k] - c) * (1 / s[o + LIFE]);
    }
  }
}

/** A size gradient at `ratio` of the life (Babylon's `_GetCurrentGradientToRef`, then the lerp). */
function sizeAt(grads: readonly (readonly [number, number])[], ratio: number): number {
  if (grads[0][0] > ratio) return grads[0][1];
  for (let i = 0; i < grads.length - 1; i++) {
    const g0 = grads[i][0];
    const f0 = grads[i][1];
    const g1 = grads[i + 1][0];
    const f1 = grads[i + 1][1];
    if (ratio >= g0 && ratio <= g1) return f0 + (f1 - f0) * ((ratio - g0) / (g1 - g0));
  }
  return grads[grads.length - 1][1];
}

/** Where a particle's values are in `ParticleSim.data` (an engine reads them to draw). */
export const PARTICLE = { stride: P, x: PX, color: CR, size: SIZE, angle: ANGLE } as const;
