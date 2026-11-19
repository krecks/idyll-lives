/**
 * The street and the weather as the camera has them (the loops are made in `crates/sound/src/outdoors.rs`):
 * - the cars driving near where the camera looks (the households' and the work taxi: the snapshot's car rows):
 *   each an engine and its tyres, higher and louder the faster it goes on screen, louder the nearer it passes
 *   the middle of the picture, left or right by where it is, the nearest few at once;
 * - a car's door shut and a parked car setting off, near the middle of the picture (a row's open doors and
 *   whether it is parked or driving, against the last look: `events.ts` `carEvents`), a couple a look;
 * - the town from above: a bed that comes in as the camera zooms out into the town view and is full at its
 *   full height;
 * - the wind in the leaves and the grass, by the gust rolling over the ground the camera looks at
 *   (`Renderer.gust`: the gusts the grass shows).
 *
 * The rules are plain functions (`carSound`, `nearness`, `townLevel`, `leavesLevel`); the class plays by them.
 */

import type { FrameState } from '../core/bridge';
import { townBlend } from '../render/shared/lens';
import { townView } from '../render/shared/townView';
import type { Renderer } from '../render/types';
import { acoustics } from './acoustics';
import { EVENTS, ONCE_LEVEL, OUTSIDE } from './catalog';
import { carEvents } from './events';
import type { WorkerKind } from './kinds';
import { Placed, playOnce, type Place, type Space } from './space';

/** Seconds for a loop here to fade to a new level. */
const FADE = 0.4;

/** How near a car `d` m from where the camera looks counts as, with the camera `radius` m out: 1 right there, a half at the camera's reach, on down. */
export function nearness(d: number, radius: number): number {
  const reach = Math.max(OUTSIDE.car.least, OUTSIDE.car.reach * radius);
  return 1 / (1 + (d / reach) ** 2);
}

/**
 * A car going `speed` m/s on screen, `near` (0..1) the middle of the picture: its engine's and its tyres' level
 * and how fast each loop plays (the engine's hum and the tyres' hiss rise with the speed).
 */
export function carSound(speed: number, near: number): { engine: number; tyres: number; engineRate: number; tyresRate: number } {
  const c = OUTSIDE.car;
  const u = Math.max(0, Math.min(1, speed / c.full));
  return { engine: c.engine * (c.idle + (1 - c.idle) * u) * near, tyres: c.tyres * u ** 1.5 * near, engineRate: 0.75 + 0.55 * u, tyresRate: 0.8 + 0.35 * u };
}

/** The town bed's level with the camera `radius` m out, by `day` (0 night .. 1 day): nothing from the play camera, full from the full town view. */
export function townLevel(radius: number, day: number): number {
  const t = OUTSIDE.town;
  return t.level * townBlend(radius) * (t.night + (1 - t.night) * Math.max(0, Math.min(1, day)));
}

/**
 * The leaves' level in a gust of `gust` (0 calm .. 1), with the camera `radius` m out and `inside` (0..1) in on a
 * room indoors.
 */
export function leavesLevel(gust: number, radius: number, inside: number): number {
  const l = OUTSIDE.leaves;
  const g = Math.max(0, Math.min(1, gust));
  return l.level * (l.calm + (1 - l.calm) * g) * (1 - (1 - l.indoors) * inside) * (1 - (1 - l.far) * townBlend(radius));
}

/** What makes the sounds here (`world.ts` `WorkerSounds`). */
interface Maker {
  sound(kind: WorkerKind, rate: number): AudioBuffer | null;
}

/** A loop all round, at its own level. */
interface Bed {
  source: AudioBufferSourceNode;
  gain: GainNode;
}

/** A car being heard: its two loops at its place. */
interface CarVoice {
  engine: AudioBufferSourceNode;
  tyres: AudioBufferSourceNode;
  engineGain: GainNode;
  tyresGain: GainNode;
  placed: Placed;
}

/** The sounds of a car's row changing (`carEvents`). */
type CarEvent = 'carDoor' | 'carStart';
const CAR_EVENTS: readonly CarEvent[] = ['carDoor', 'carStart'];

/** A car as last looked at: where, when (the frames' clock, s) and how fast it went (m/s on screen, smoothed). */
interface Seen {
  x: number;
  z: number;
  at: number;
  speed: number;
  look: number;
}

export class Outside {
  private town: Bed | null = null;
  private leaves: Bed | null = null;
  private readonly voices = new Map<number, CarVoice>();
  private readonly cars = new Map<number, Seen>();
  /** Every car row as last looked at, by row: in sight or not (`shown`) and its open doors. */
  private readonly rows = new Map<number, { shown: number; doors: number }>();
  /** The car doors and starts heard so far. */
  private events = 0;
  private readonly screen = { x: 0, y: 0 };
  private look = 0;
  private said: { cars: { row: number; speed: number; near: number }[]; driving: number; town: number; leaves: number; gust: number; events: number } = { cars: [], driving: 0, town: 0, leaves: 0, gust: 0, events: 0 };

  constructor(
    private readonly renderer: Renderer,
    private readonly made: Maker,
  ) {}

  /** What was heard on the last look: the cars sounding (their speed, m/s on screen, and how near), how many drive, the town's and the leaves' level and the gust; and the car doors and starts heard so far. */
  heard(): Outside['said'] {
    return this.said;
  }

  /**
   * A look (`frame`) at audio time `t`. `quiet`: nothing sounds (a hidden tab); `still`: no cars (the game is
   * paused, or the town's map is up); `fast`: no car doors or starts (the game runs too fast for them); `day`: 0
   * at night .. 1 by day. The car doors and starts heard on it.
   */
  update(ac: AudioContext, out: AudioNode, around: Space, frame: FrameState, t: number, o: { quiet: boolean; still: boolean; fast: boolean; day: number }): readonly CarEvent[] {
    const radius = acoustics.radius;
    const target = townView.target;

    // The town, from above.
    const town = o.quiet ? 0 : townLevel(radius, o.day);
    if (town > 0 || this.town) this.town ??= this.bed(ac, out, 'town');
    this.town?.gain.gain.setTargetAtTime(town, t, FADE / 3);

    // The wind, by the gust over what the camera looks at.
    const gust = this.renderer.gust(target.x, target.z);
    const leaves = o.quiet ? 0 : leavesLevel(gust, radius, acoustics.inside);
    this.leaves ??= this.bed(ac, out, 'leaves');
    this.leaves?.gain.gain.setTargetAtTime(leaves, t, FADE / 3);

    // The cars driving, nearest first.
    const look = ++this.look;
    const { curr, layout } = frame;
    const c = layout.car;
    const count = c && layout.header.carCount !== undefined ? curr[layout.header.carCount] : 0;
    const now = frame.now / 1000;
    const near: { row: number; x: number; z: number; speed: number; near: number }[] = [];
    const played: CarEvent[] = [];
    const hears = !o.quiet && !o.still && !o.fast;
    let driving = 0;
    // (Every row each look, heard or not, so what changed is always since the look before.)
    for (let i = 0; c && i < count; i++) {
      const at = (layout.carBase ?? 0) + i * (layout.carStride ?? 0);
      const shown = curr[at + c.shown];
      const doors = c.doors === undefined ? 0 : curr[at + c.doors];
      const last = this.rows.get(i);
      const happened = carEvents(last, shown, doors);
      if (last) {
        last.shown = shown;
        last.doors = doors;
      } else this.rows.set(i, { shown, doors });
      const x = curr[at + c.x];
      const z = curr[at + c.z];
      const n = shown === 0 ? 0 : nearness(Math.hypot(x - target.x, z - target.z), radius);
      if (n >= EVENTS.car.near) {
        // A car that near may shut a door or set off any moment: its sounds made before they're wanted.
        const made = CAR_EVENTS.map((kind) => this.made.sound(kind, ac.sampleRate));
        CAR_EVENTS.forEach((kind, k) => {
          const buffer = made[k];
          if (!hears || !buffer || played.length >= EVENTS.car.perLook || !(kind === 'carDoor' ? happened.door : happened.start)) return;
          playOnce(ac, around, buffer, ONCE_LEVEL[kind] * n, this.place(x, z));
          played.push(kind);
        });
      }
      if (shown !== 2 || o.quiet || o.still) continue;
      driving++;
      let seen = this.cars.get(i);
      if (!seen) this.cars.set(i, (seen = { x, z, at: now, speed: 0, look }));
      const dt = now - seen.at;
      // (A look long ago, or a car put somewhere else: no speed to read from it.)
      const moved = Math.hypot(x - seen.x, z - seen.z);
      if (dt > 0.02 && dt < 0.5 && moved < 40) seen.speed += (moved / dt - seen.speed) * 0.5;
      else if (dt >= 0.5) seen.speed = 0;
      seen.x = x;
      seen.z = z;
      seen.at = now;
      seen.look = look;
      if (n >= OUTSIDE.car.drop) near.push({ row: i, x, z, speed: seen.speed, near: n });
    }
    for (const row of this.rows.keys()) if (row >= count) this.rows.delete(row);
    this.events += played.length;
    for (const [row, seen] of this.cars) if (seen.look !== look) this.cars.delete(row);
    near.sort((a, b) => b.near - a.near);
    near.length = Math.min(near.length, OUTSIDE.car.voices);
    const engine = near.length ? this.made.sound('carEngine', ac.sampleRate) : null;
    const tyres = near.length ? this.made.sound('carTyres', ac.sampleRate) : null;
    for (const [row, v] of this.voices) {
      if (near.some((n) => n.row === row)) continue;
      // Gone by (or parked): it fades out and leaves the graph.
      v.engineGain.gain.setTargetAtTime(0, t, FADE / 3);
      v.tyresGain.gain.setTargetAtTime(0, t, FADE / 3);
      this.voices.delete(row);
      setTimeout(() => this.drop(v), FADE * 3000);
    }
    for (const n of near) {
      if (!engine || !tyres) break;
      const place = this.place(n.x, n.z);
      let v = this.voices.get(n.row);
      if (!v) {
        const placed = new Placed(ac, around, place);
        const loop = (buffer: AudioBuffer): [AudioBufferSourceNode, GainNode] => {
          const source = ac.createBufferSource();
          source.buffer = buffer;
          source.loop = true;
          const gain = ac.createGain();
          gain.gain.value = 0;
          source.connect(gain).connect(placed.input);
          source.start(0, Math.random() * buffer.duration);
          return [source, gain];
        };
        const [e, eg] = loop(engine);
        const [ty, tg] = loop(tyres);
        v = { engine: e, tyres: ty, engineGain: eg, tyresGain: tg, placed };
        this.voices.set(n.row, v);
      }
      const s = carSound(n.speed, n.near);
      v.placed.place(place, t);
      v.engineGain.gain.setTargetAtTime(s.engine, t, FADE / 3);
      v.tyresGain.gain.setTargetAtTime(s.tyres, t, FADE / 3);
      v.engine.playbackRate.setTargetAtTime(s.engineRate, t, 0.25);
      v.tyres.playbackRate.setTargetAtTime(s.tyresRate, t, 0.25);
    }
    this.said = { cars: near.map((n) => ({ row: n.row, speed: n.speed, near: n.near })), driving, town, leaves, gust, events: this.events };
    return played;
  }

  /** A car's place at town point (`x`, `z`): left or right by where it is on screen, behind a wall from a room in view; dry. */
  private place(x: number, z: number): Place {
    const pan = this.renderer.project(x, 0.7, z, this.screen) ? Math.max(-1, Math.min(1, (this.screen.x / (window.innerWidth || 1)) * 2 - 1)) * 0.8 : 0;
    return { pan: Number.isFinite(pan) ? pan : 0, muffle: acoustics.muffleAt(x, z) };
  }

  /** Fades everything out (no looks for a while: the picture stopped). */
  hush(t: number): void {
    for (const b of [this.town, this.leaves]) b?.gain.gain.setTargetAtTime(0, t, FADE / 3);
    for (const v of this.voices.values()) {
      v.engineGain.gain.setTargetAtTime(0, t, FADE / 3);
      v.tyresGain.gain.setTargetAtTime(0, t, FADE / 3);
    }
  }

  dispose(): void {
    for (const b of [this.town, this.leaves]) {
      if (!b) continue;
      b.source.stop();
      b.gain.disconnect();
    }
    this.town = this.leaves = null;
    for (const v of this.voices.values()) this.drop(v);
    this.voices.clear();
  }

  /** A loop all round, silent until faded in; null while it is still being made. */
  private bed(ac: AudioContext, out: AudioNode, kind: WorkerKind): Bed | null {
    const buffer = this.made.sound(kind, ac.sampleRate);
    if (!buffer) return null;
    const source = ac.createBufferSource();
    source.buffer = buffer;
    source.loop = true;
    const gain = ac.createGain();
    gain.gain.value = 0;
    source.connect(gain).connect(out);
    source.start(0, Math.random() * buffer.duration);
    return { source, gain };
  }

  private drop(v: CarVoice): void {
    for (const s of [v.engine, v.tyres]) {
      try {
        s.stop();
      } catch {
        // Never started.
      }
    }
    v.placed.disconnect();
  }
}
