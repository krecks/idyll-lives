/**
 * The residents' footsteps: each one in view walking, jogging or running is heard putting its feet down, on
 * what it walks on (`surfaces.ts`: a wooden floor, tiles, carpet, grass, paving) and in what it wears (shoes,
 * or barefoot when the renderer draws it without: asleep, off to the bath). The steps are made in `steps.ts`.
 *
 * The renderer says how far through its stride a resident is and how fast it goes (`Renderer.simStride`: the
 * left heel strikes at 0, the right at 0.5). The world's sound looks ten times a second and starts the heel
 * strikes due before the next look at the time on the audio clock at which they will be heard as the foot
 * lands (by the frames' own clock, the one the stride is drawn by, with the sound card's delay taken off:
 * `getOutputTimestamp`), however the looks fall.
 *
 * Quiet, since they are many: each step is placed like any of the world's sounds (left to right, its room,
 * a wall, the camera's distance), the more walkers the lower each, a few voices at once at most, and none
 * from so far out that they could not be heard anyway.
 */

import type { Content } from '../content/content';
import type { FrameState } from '../core/bridge';
import type { Renderer, SimStride } from '../render/types';
import { acoustics, distanceGain } from './acoustics';
import { STEPS } from './catalog';
import type { RoomSize } from './rooms';
import { Placed, type Space } from './space';
import { stepSamples, STEP_VERSIONS, type Footwear, type Surface } from './steps';
import { Ground, type GroundWorld } from './surfaces';

/** A walker: when its last step was started for (the frames' clock, s), and on which look it was last seen. */
export interface Walker {
  stepped: number;
  seen: number;
}

/**
 * Seconds until the next heel strike of a stride at `phase` (0..1) going `rate` cycles a second, and which foot
 * (0 left, 1 right), if it comes within `horizon` s; null if not (or the stride stands still).
 */
export function nextStrike(phase: number, rate: number, horizon: number): { in: number; foot: 0 | 1 } | null {
  if (!(rate > 0) || !(phase >= 0 && phase < 1)) return null;
  const right = phase < 0.5;
  const wait = ((right ? 0.5 : 1) - phase) / rate;
  return wait <= horizon ? { in: wait, foot: right ? 1 : 0 } : null;
}

/**
 * A look at a walker, `phase` through its stride at time `t` (s) and going `rate` cycles a second: the step to
 * start now, if a heel strikes within `horizon` s (before the next look: when, and which foot). One step a strike
 * (the same strike can be due on two looks), and never two of a walker's closer than `STEPS.apart` (a sprint in a
 * sped-up game is not a rattle).
 */
export function stepDue(w: Walker, phase: number, rate: number, t: number, horizon = STEPS.ahead): { when: number; foot: 0 | 1 } | null {
  const due = nextStrike(phase, rate, horizon);
  if (!due || t + due.in - w.stepped < STEPS.apart) return null;
  w.stepped = t + due.in;
  return { when: w.stepped, foot: due.foot };
}

/** One of the few steps that can sound at once: its place in the world's space, and when it is free again. */
interface Voice {
  placed: Placed;
  gain: GainNode;
  until: number;
}

export class Footsteps {
  private readonly ground: Ground;
  private readonly walkers = new Map<number, Walker>();
  private readonly voices: Voice[] = [];
  private readonly buffers = new Map<string, AudioBuffer>();
  private readonly stride: SimStride = { phase: 0, rate: 0, gear: 0, bare: false };
  private readonly head = { x: 0, y: 0, z: 0 };
  private readonly screen = { x: 0, y: 0 };
  private look = 0;
  /** Steps started so far, and the last few (what, on what, when: tests and checks). */
  private count = 0;
  private recent: { surface: Surface; wear: Footwear; gear: number; at: number }[] = [];
  private inView = 0;

  constructor(
    private readonly renderer: Renderer,
    content: Content,
  ) {
    this.ground = new Ground(content.floorCoverings.map((c) => c.finish));
  }

  setWorld(world: GroundWorld | null): void {
    this.ground.setWorld(world);
  }

  /** What was heard: steps started so far, the walkers in view on the last look, and the last few steps. */
  heard(): { steps: number; walkers: number; recent: { surface: Surface; wear: Footwear; gear: number; at: number }[] } {
    return { steps: this.count, walkers: this.inView, recent: this.recent };
  }

  /**
   * A look at the residents (`frame`), at audio time `t`: starts the steps due before the next look. `still`: nothing
   * steps (the game is paused, the tab hidden).
   */
  update(ac: AudioContext, around: Space, frame: FrameState, t: number, still: boolean): void {
    const look = ++this.look;
    // From far out no step would be heard: none is made.
    const near = !still && distanceGain(acoustics.radius) >= STEPS.farthest;
    const { curr, layout } = frame;
    const k = layout.sim;
    const count = curr[layout.header.simCount];
    const width = window.innerWidth || 1;
    const height = window.innerHeight || 1;
    let walking = 0;
    let started = 0;
    // The frame's time (the stride is drawn by it), and the audio clock's time at which a sound is heard at one of
    // the frames' times: a step is started further ahead by what the sound card delays it.
    const now = frame.now / 1000;
    const stamp = ac.getOutputTimestamp?.();
    const heardAt = stamp?.performanceTime && stamp.contextTime ? (at: number) => Math.max(t, stamp.contextTime! + at - stamp.performanceTime! / 1000) : (at: number) => t + Math.max(0, at - now);
    const horizon = STEPS.ahead + Math.min(STEPS.ahead, ac.outputLatency || 0);
    for (let i = 0; i < count && near; i++) {
      if (!this.renderer.simStride(i, this.stride)) continue;
      const o = layout.headerLen + i * layout.simStride;
      const id = curr[o + k.id];
      const { phase, rate, gear, bare } = this.stride;
      let w = this.walkers.get(id);
      if (!w) this.walkers.set(id, (w = { stepped: -1, seen: look }));
      w.seen = look;
      const due = stepDue(w, phase, rate, now, horizon);
      // Only those in the picture are heard.
      if (!this.renderer.simHead(i, this.head) || !this.renderer.project(this.head.x, this.head.y, this.head.z, this.screen)) continue;
      const { x: px, y: py } = this.screen;
      if (px < -0.05 * width || px > 1.05 * width || py < -0.05 * height || py > 1.05 * height) continue;
      walking++;
      if (!due || started >= STEPS.perLook) continue;
      const sx = curr[o + k.x];
      const sz = curr[o + k.z];
      const pan = Math.max(-1, Math.min(1, (px / width) * 2 - 1)) * 0.7;
      if (this.step(ac, around, t, heardAt(due.when), this.ground.at(sx, sz), bare ? 'bare' : 'shoe', gear, due.foot, Number.isFinite(pan) ? pan : 0, sx, sz)) started++;
    }
    this.inView = walking;
    for (const [id, w] of this.walkers) if (w.seen !== look) this.walkers.delete(id);
  }

  /** Starts one step at audio time `when` (it is `t` now); false if every voice is busy. */
  private step(ac: AudioContext, around: Space, t: number, when: number, surface: Surface, wear: Footwear, gear: number, foot: 0 | 1, pan: number, x: number, z: number): boolean {
    // (A voice whose last step has ended: it is moved to the new step's place at once.)
    let voice = this.voices.find((v) => v.until <= t);
    if (!voice) {
      if (this.voices.length >= STEPS.voices) return false;
      const placed = new Placed(ac, around, { pan });
      const gain = ac.createGain();
      gain.connect(placed.input);
      voice = { placed, gain, until: 0 };
      this.voices.push(voice);
    }
    const version = Math.floor(Math.random() * STEP_VERSIONS);
    const buffer = this.buffer(ac, surface, wear, version);
    const size: RoomSize | null = acoustics.sizeAt(x, z);
    voice.placed.place({ pan, muffle: acoustics.muffleAt(x, z), rooms: { small: 0, medium: 0, large: 0, ...(size ? { [size]: 1 } : {}) } }, when, true);
    // The more walk, the lower each; a jog and a run land harder; no two steps quite alike.
    const crowd = Math.min(1, STEPS.crowd / Math.sqrt(Math.max(1, this.inView)));
    voice.gain.gain.value = STEPS.level[surface][wear] * (STEPS.gear[gear] ?? 1) * crowd * (0.8 + 0.2 * Math.random());
    const source = ac.createBufferSource();
    source.buffer = buffer;
    source.playbackRate.value = (foot ? 1.02 : 0.98) * (0.95 + 0.1 * Math.random());
    source.connect(voice.gain);
    source.onended = () => source.disconnect();
    source.start(when);
    voice.until = when + buffer.duration / source.playbackRate.value;
    this.count++;
    this.recent = [...this.recent.slice(-7), { surface, wear, gear, at: when }];
    return true;
  }

  /** A step's samples as an audio buffer, made the first time it is heard (a fraction of a millisecond). */
  private buffer(ac: AudioContext, surface: Surface, wear: Footwear, version: number): AudioBuffer {
    const key = `${surface}:${wear}:${version}:${ac.sampleRate}`;
    let b = this.buffers.get(key);
    if (!b) {
      const samples = stepSamples(surface, wear, version, ac.sampleRate);
      b = new AudioBuffer({ numberOfChannels: 1, length: samples.length, sampleRate: ac.sampleRate });
      b.copyToChannel(samples as Float32Array<ArrayBuffer>, 0);
      this.buffers.set(key, b);
    }
    return b;
  }

  dispose(): void {
    for (const v of this.voices) v.placed.disconnect();
    this.voices.length = 0;
    this.walkers.clear();
  }
}
