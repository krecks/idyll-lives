/**
 * Placed objects' moving parts (a fridge's doors): the clips of their models (`ModelMotion`)
 * played while residents use them, as the model entry's `motions` say. A use that starts opens
 * its clips (each `delay` seconds in, if it says; only the one for the resident's slot, if it
 * says); each plays back after its `hold` seconds or when the use ends, or, with `every`, plays from
 * rest to rest again every so many seconds while the use lasts. A loop (`loops`: a
 * rocking chair) goes round from rest to rest while the use lasts, then finishes its round. An
 * object that is open (the sim's `open`: a wall bed folded down) holds its `motions.open` clip
 * open as long as it stays so, and its `motions.night` clip (curtains drawn) from dusk to dawn,
 * whoever is asleep. A crop's `motions.growth` clip is held where its growth stage is, moving on
 * to the next stage at the clip's own pace. An ambient clip (`idle`: a ceiling fan, a fountain's water) goes
 * round all the time, but only within `ambientRange` of the camera (none on Low), without the
 * shadows being redrawn for it. A clock's hands (`clock`) show the game's time, set each game
 * minute (each five on Low) within `CLOCK_RANGE` of the camera or the ambient range. A part moved by
 * several clips at once takes all their poses (`partPose`: a record spinning as it swings up; a use's clip
 * on top of a crop's growth stage, the veggie bed's plants rustling as tall as they have grown). Each
 * instance's parts draw from buffers of their own (`MovingTarget`), written only while something moves.
 *
 * Time is the residents' animation time: it stops while the game is paused and runs faster
 * (up to 2.5 times) at higher speeds, as in `characters.ts`.
 */

import { writeMoving, type PopTarget } from './buildFx';
import { Matrix, type Mesh, type Vector3 } from './core';
import { GAME_RATE, QUIET_SPEED } from './characters';
import { partPose, type ModelClip, type ModelMotion, type Motions } from './models';
import type { FrameState } from '../../core/bridge';
import type { ObjectPlacement } from '../../core/protocol';

/** A placed object with moving parts. */
export interface MovingObject {
  id: number;
  target: PopTarget;
  motion: ModelMotion;
  motions: Motions;
  /** Its definition's interaction ids, by the snapshot's `interaction` index. */
  interactions: readonly string[];
  /** Open (the placement's `open`): its `motions.open` clip is held open. */
  open?: boolean;
  /** A crop's growth stage (the placement's `growth`: 0 planted, 1 growing, 2 ripe, 3 picked). */
  stage?: number;
  /** The slot (content `slots`, from 0) of a resident standing, sitting or lying at `x`, `z` (the snapshot's), for clips with a `slot`. */
  slotOf?: (x: number, z: number) => number;
}

/** Metres from the camera within which clocks show the time, at least (Low has no ambient range). */
const CLOCK_RANGE = 30;
/** Game minutes between the clocks' hands being set: on Medium and High, and on Low. */
const CLOCK_STEP = 1;
const CLOCK_STEP_LOW = 5;
/** Frames between looks at clocks that came into range while the time stood still. */
const CLOCK_LOOK = 30;

/**
 * Where a crop's growth clip stands (shares of it) for each stage, planted to picked: ripe at
 * rest (the catalog's picture), then picked (the crop gone), growing (smaller) and just planted
 * (seedlings). Each stage moves on to the next along it: picking plays forward to picked, planting
 * on to planted, growing back through growing and picked (no crop yet) to ripe.
 */
export const GROWTH_TIMES = [1, 2 / 3, 0, 1 / 3];

/** A clip of one object: how far it has played, and what plays it. */
interface ClipState {
  clip: ModelClip;
  /** Seconds into the clip (0: at rest). */
  time: number;
  /** The use that opened it (resident, action, interaction), so a new one opens it again. */
  use: string;
  /** Animation time it plays back from (Infinity: when the use ends). */
  until: number;
  /** Animation time it starts opening (its `delay` after the use starts), or its next round (`every`). */
  start: number;
  /** Seconds between its rounds (`repeat`). */
  every?: number;
  /**
   * Opens and closes (a use's, or held open), goes round while used (`loops`), plays from rest to rest
   * every so often while used (`repeat`), goes round always (`idle`), or is held where a crop's stage
   * (`growth`) or the game's time (`clock`) has it.
   */
  kind: 'use' | 'loop' | 'repeat' | 'idle' | 'growth' | 'clock';
}

/** A clip a use wants this frame. */
interface Wanted {
  /** The use (resident, action, interaction). */
  use: string;
  /** Seconds it stays open after it starts (undefined: while it lasts). */
  hold?: number;
  /** Seconds after the use starts that it starts. */
  delay?: number;
  /** Seconds between its rounds, if it repeats. */
  every?: number;
  kind: 'use' | 'loop';
}

/** What `update` moved: parts that a use moves (bounds and shadows follow), or only ambient ones (neither does). */
export interface Moved {
  moved: boolean;
  ambient: boolean;
}

export class ObjectMotion {
  private objects = new Map<number, MovingObject>();
  /** By object id, then clip name. Kept across world rebuilds, so a door mid-swing carries on. */
  private states = new Map<number, Map<string, ClipState>>();
  private clock = 0;
  private lastNow = 0;
  /** Uses this frame: by object id, the clips wanted by clip name. */
  private readonly wanted = new Map<number, Map<string, Wanted>>();
  /** Metres from the camera within which ambient clips play (0: none; the quality's `ambientRange`). */
  private ambientRange = 0;
  /** The objects with clock hands, and the step of the game's time each shows (absent: to be set). */
  private clocks: MovingObject[] = [];
  private readonly clockShown = new Map<number, number>();
  private clockStep = -1;
  private clockFrames = 0;
  /** The game's minute of the day, as the last frame had it. */
  private minute = 0;

  /** Night (the lamps are lit): `motions.night` clips are held open. Null until the lighting first says. */
  private night: boolean | null = null;
  /** The night clips go straight to where the lighting has them (the first lighting of a world), without playing. */
  private snapNight = false;

  /**
   * The lamps' level (the lighting's `lamps`, 0 by day to 1 at night): night clips close from about
   * half way up and open again below. The first level after a world is built shows them as they
   * are, without playing.
   */
  setLamps(level: number): void {
    const night = this.night ? level > 0.4 : level > 0.55;
    if (this.night === null) this.snapNight = true;
    this.night = night;
  }

  /** Ambient clips play within `range` metres of the camera (0: nowhere, all at rest). */
  setAmbientRange(range: number): void {
    this.ambientRange = range;
  }

  /** The objects of a newly built world; those still mid-clip are posed again. */
  set(objects: MovingObject[]): void {
    const before = this.objects;
    this.objects = new Map(objects.map((o) => [o.id, o]));
    for (const id of [...this.states.keys()]) if (!this.objects.has(id)) this.states.delete(id);
    // What is open when first seen (a game loaded, a lot looked at) is shown open, without the
    // opening; what opens later (`update`) opens on screen.
    for (const o of objects) {
      if (before.has(o.id)) continue;
      const growth = this.growth(o);
      if (growth) {
        let clips = this.states.get(o.id);
        if (!clips) this.states.set(o.id, (clips = new Map()));
        clips.set(growth.name, { clip: growth.clip, time: growth.at, use: '', until: Infinity, start: 0, kind: 'growth' });
      }
      for (const name of this.held(o)) {
        const clip = o.motion.clips.get(name);
        if (!clip) continue;
        let clips = this.states.get(o.id);
        if (!clips) this.states.set(o.id, (clips = new Map()));
        if (!clips.has(name)) clips.set(name, { clip, time: clip.duration, use: '', until: Infinity, start: 0, kind: 'use' });
      }
    }
    // Clocks: their hands are set by the next frame (`setClocks`).
    this.clocks = objects.filter((o) => o.motions.clock);
    this.clockShown.clear();
    this.clockStep = -1;
    for (const o of this.clocks) {
      let clips = this.states.get(o.id);
      for (const name of [o.motions.clock!.hour, o.motions.clock!.minute]) {
        const clip = name === undefined ? undefined : o.motion.clips.get(name);
        if (!clip) continue;
        if (!clips) this.states.set(o.id, (clips = new Map()));
        if (!clips.has(name!)) clips.set(name!, { clip, time: 0, use: '', until: Infinity, start: 0, kind: 'clock' });
      }
    }
    for (const [id, clips] of this.states) {
      const o = this.objects.get(id)!;
      for (const [name, state] of [...clips]) {
        const clip = o.motion.clips.get(name);
        if (clip) state.clip = clip;
        else clips.delete(name);
      }
      this.pose(o, clips);
    }
  }

  clear(): void {
    this.objects.clear();
    this.states.clear();
    this.clocks = [];
    this.clockShown.clear();
    this.night = null;
  }

  /** A crop's growth clip and where its stage has it (seconds), if it has both. */
  private growth(o: MovingObject): { name: string; clip: ModelClip; at: number } | undefined {
    const name = o.motions.growth;
    const clip = name !== undefined && o.stage !== undefined ? o.motion.clips.get(name) : undefined;
    return clip ? { name: name!, clip, at: (GROWTH_TIMES[o.stage!] ?? 0) * clip.duration } : undefined;
  }

  /** The clips `o` holds open now: its `open` one while open, its `night` one at night. */
  private held(o: MovingObject): string[] {
    const held: string[] = [];
    if (o.open && o.motions.open) held.push(o.motions.open);
    if (this.night && o.motions.night) held.push(o.motions.night);
    return held;
  }

  /** Plays this frame's clips, the ambient ones near `eye` (the camera). */
  update(frame: FrameState, actionTags: readonly string[], eye: Vector3): Moved {
    const { curr, layout, now } = frame;
    let dt = this.lastNow ? Math.min(Math.max((now - this.lastNow) / 1000, 0), 0.1) : 0;
    this.lastNow = now;
    this.minute = curr[layout.header.minute] ?? 0;
    if (!this.objects.size) return { moved: false, ambient: false };
    const speed = curr[layout.header.speed] | 0;
    const quiet = layout.header.calm !== undefined && curr[layout.header.calm] > 0;
    const gameSpeed = quiet ? Math.max(speed, QUIET_SPEED) : speed;
    if (gameSpeed === 0) dt = 0;
    dt *= Math.min(GAME_RATE[gameSpeed] ?? 1, 2.5);
    this.clock += dt;

    this.findUses(frame, actionTags);
    const moved = new Set<Mesh>();
    const written = new Set<Mesh>();
    let ambient = false;
    const reach = this.ambientRange * this.ambientRange;
    for (const [id, o] of this.objects) {
      const wanted = this.wanted.get(id);
      const held = this.held(o);
      const idle = o.motions.idle !== undefined && reach > 0 && this.distance2(o, eye) < reach ? o.motions.idle : undefined;
      let clips = this.states.get(id);
      const growth = this.growth(o);
      if ((wanted || held.length || idle || growth) && !clips) this.states.set(id, (clips = new Map()));
      if (!clips) continue;
      for (const name of held) {
        const clip = o.motion.clips.get(name);
        if (clip && !clips.has(name)) clips.set(name, { clip, time: 0, use: '', until: Infinity, start: 0, kind: 'use' });
      }
      // The first lighting of a world: night clips as it is, without playing.
      const snap = this.snapNight && o.motions.night !== undefined ? o.motions.night : undefined;
      if (growth && !clips.has(growth.name)) clips.set(growth.name, { clip: growth.clip, time: growth.at, use: '', until: Infinity, start: 0, kind: 'growth' });
      const idleClip = idle ? o.motion.clips.get(idle) : undefined;
      if (idle && idleClip && !clips.has(idle)) clips.set(idle, { clip: idleClip, time: 0, use: '', until: Infinity, start: 0, kind: 'idle' });
      let changed = false;
      let drifted = false;
      for (const [name, want] of wanted ?? []) {
        const clip = o.motion.clips.get(name);
        if (!clip) continue;
        const state = clips.get(name);
        if (state?.use === want.use) continue;
        // A new use: open (again, from its delay on), go round, or start its rounds.
        const start = this.clock + (want.delay ?? 0);
        const until = want.hold === undefined ? Infinity : start + want.hold;
        const kind = want.every ? 'repeat' : want.kind;
        if (state) Object.assign(state, { use: want.use, until, start, every: want.every, kind });
        else clips.set(name, { clip, time: 0, use: want.use, until, start, every: want.every, kind });
      }
      for (const [name, state] of clips) {
        const d = state.clip.duration;
        if (state.kind === 'clock') continue;
        if (state.kind === 'idle') {
          // Round and round near the camera; at rest further off.
          if (name !== idle) {
            clips.delete(name);
            drifted = true;
          } else if (dt && d > 0) {
            state.time = (state.time + dt) % d;
            drifted = true;
          }
          continue;
        }
        if (state.kind === 'growth') {
          // On towards where the stage has it.
          const at = growth?.name === name ? growth.at : 0;
          const time = state.time < at ? Math.min(at, state.time + dt) : Math.max(at, state.time - dt);
          if (time !== state.time) changed = true;
          state.time = time;
          if (!growth && !time) clips.delete(name);
          continue;
        }
        const using = state.use !== '' && wanted?.get(name)?.use === state.use;
        // (A use that ended leaves `use` empty: nothing wanted then is no use keeping it open.)
        if (!using && !wanted?.has(name)) state.use = '';
        if (state.kind === 'repeat') {
          // From rest to rest each round, the first its `delay` into the use, then every `every`
          // seconds; once not used, it finishes its round.
          if (!state.time) {
            if (!using) {
              clips.delete(name);
              continue;
            }
            if (!dt || this.clock < state.start) continue;
            state.start = Math.max(state.start + (state.every ?? d), this.clock);
          }
          const time = state.time + dt;
          state.time = time >= d ? 0 : time;
          changed = true;
          if (!state.time && !using) clips.delete(name);
          continue;
        }
        if (state.kind === 'loop' && !held.includes(name)) {
          // Round and round while used; once not, on to the end of the round and stop there (at
          // rest: a loop ends where it starts). One that never got going stops where it is.
          if (!using && (state.time === 0 || state.time + dt >= d)) {
            changed = true;
            clips.delete(name);
            continue;
          }
          const time = using && d > 0 ? (state.time + dt) % d : state.time + dt;
          if (time !== state.time) changed = true;
          state.time = time;
          continue;
        }
        const open = held.includes(name) || (using && this.clock >= state.start && this.clock < state.until);
        const step = name === snap ? d : dt;
        const time = Math.min(Math.max(state.time + (open ? step : -step), 0), d);
        if (time !== state.time) changed = true;
        state.time = time;
        // Closed again after its `hold`, it stays so (kept) until the use ends; a new use opens it again.
        if (!time && !open && !using) clips.delete(name);
      }
      if (changed || drifted) {
        this.pose(o, clips, written);
        if (changed) for (const part of o.target.moving!.parts) for (const mesh of part.meshes) moved.add(mesh);
        else ambient = true;
      }
      if (!clips.size) this.states.delete(id);
    }
    this.snapNight = false;
    if (this.clocks.length && this.setClocks(eye, written)) ambient = true;
    // Each buffer written goes up once, however many of its instances moved.
    for (const mesh of written) {
      if (mesh.isDisposed()) continue;
      mesh.thinInstanceAllowAutomaticStaticBufferRecreation = true;
      mesh.thinInstanceBufferUpdated('matrix');
    }
    // Culled by where the parts are now (an open door reaches out of the closed model's box). Ambient
    // clips stay about where they are at rest (a fan's blades, water in its basin): not worth it.
    for (const mesh of moved) if (!mesh.isDisposed()) mesh.thinInstanceRefreshBoundingInfo(false);
    return { moved: moved.size > 0, ambient };
  }

  /**
   * How object `id`'s seat (`motions.ride`: the first part its clip moves) has moved from rest, in
   * world space (a point at rest times it is where it is now), for the residents sitting or lying
   * on it; null while it rests.
   */
  ride(id: number): Matrix | null {
    const o = this.objects.get(id);
    const name = o?.motions.ride;
    const clips = this.states.get(id);
    const state = name ? clips?.get(name) : undefined;
    const part = state?.clip.tracks[0]?.part;
    if (!o || !state || part === undefined || !partPose([...clips!.values()], part, this.rideTmp)) return null;
    Matrix.FromArrayToRef(o.target.matrices, o.target.index * 16, this.placed);
    this.placed.invertToRef(this.rideOut);
    this.rideOut.multiplyToRef(Matrix.FromArrayToRef(this.rideTmp, 0, this.posed), this.rideOut);
    return this.rideOut.multiplyToRef(this.placed, this.rideOut);
  }
  private readonly rideTmp = new Float32Array(16);

  /** How far round (0..1) object `id`'s loop is now (`loops`: a bike's pedals, for the rider's feet); null while none plays. */
  loopPhase(id: number): number | null {
    const clips = this.states.get(id);
    if (!clips) return null;
    for (const state of clips.values()) if (state.kind === 'loop' && state.clip.duration > 0) return (state.time / state.clip.duration) % 1;
    return null;
  }
  private readonly placed = new Matrix();
  private readonly posed = new Matrix();
  private readonly rideOut = new Matrix();

  /** Squared distance (m) from `eye` to where `o` stands. */
  private distance2(o: MovingObject, eye: Vector3): number {
    const m = o.target.matrices;
    const k = o.target.index * 16;
    return (m[k + 12] - eye.x) ** 2 + (m[k + 13] - eye.y) ** 2 + (m[k + 14] - eye.z) ** 2;
  }

  /** Which objects residents use this frame, and the clip each use plays. */
  private findUses({ curr, layout }: FrameState, actionTags: readonly string[]): void {
    this.wanted.clear();
    const k = layout.sim;
    if (k.object === undefined || k.action === undefined) return;
    const H = layout.headerLen;
    const S = layout.simStride;
    const count = curr[layout.header.simCount] | 0;
    for (let j = 0; j < count; j++) {
      const o = H + j * S;
      const id = curr[o + k.object] | 0;
      const act = curr[o + k.action] | 0;
      // Only while using it, not while walking to it.
      if (id < 0 || act < 0) continue;
      const object = this.objects.get(id);
      if (!object) continue;
      const interaction = k.interaction === undefined ? -1 : curr[o + k.interaction] | 0;
      const { uses, loops } = object.motions;
      const it = object.interactions[interaction] ?? '';
      const tag = actionTags[act] ?? '';
      const clips = uses[it] ?? uses[tag];
      const loop = loops[it] ?? loops[tag];
      if (!clips && !loop) continue;
      const use = `${curr[o + k.id]}:${act}:${interaction}`;
      const wanted = this.wanted.get(id) ?? new Map<string, Wanted>();
      let slot: number | undefined;
      for (const { clip, hold, delay, every, slot: only } of clips ?? []) {
        // A clip for one slot (a bunk's duvet): only for the resident there.
        if (only !== undefined && only !== (slot ??= object.slotOf?.(curr[o + k.x], curr[o + k.z]) ?? 0)) continue;
        wanted.set(clip, { use, hold, delay, every, kind: 'use' });
      }
      if (loop) wanted.set(loop, { use, kind: 'loop' });
      this.wanted.set(id, wanted);
    }
  }

  /** Writes an object's parts where its clips have them (at rest where none does); `written` takes the meshes whose buffers changed (else they go up now). */
  private pose(o: MovingObject, clips: Map<string, ClipState>, written?: Set<Mesh>): void {
    poseParts(o.target, [...clips.values()], written);
  }

  /**
   * Sets the hands of the clocks near `eye` that show another time than the game's (to the game
   * minute; to five on Low), including those come near while it stood still (looked for every
   * `CLOCK_LOOK` frames). True if any moved; `written` takes their meshes.
   */
  private setClocks(eye: Vector3, written: Set<Mesh>): boolean {
    const step = this.ambientRange > 0 ? CLOCK_STEP : CLOCK_STEP_LOW;
    const shown = Math.floor(this.minute / step);
    const look = ++this.clockFrames >= CLOCK_LOOK;
    if (look) this.clockFrames = 0;
    else if (shown === this.clockStep) return false;
    this.clockStep = shown;
    const range = Math.max(this.ambientRange, CLOCK_RANGE);
    const minute = shown * step;
    let moved = false;
    for (const o of this.clocks) {
      const clips = this.states.get(o.id);
      if (!clips || this.clockShown.get(o.id) === shown || this.distance2(o, eye) > range * range) continue;
      const { hour, minute: hand, at = 0 } = o.motions.clock!;
      for (const [name, period] of [[hour, 720], [hand, 60]] as const) {
        const state = name === undefined ? undefined : clips.get(name);
        if (state?.kind === 'clock') state.time = (((((minute - at) / period) % 1) + 1) % 1) * state.clip.duration;
      }
      this.clockShown.set(o.id, shown);
      this.pose(o, clips, written);
      moved = true;
    }
    return moved;
  }
}

/**
 * Which of `o`'s slots (content `slots`, from 0) a resident at `x`, `z` is in, as sim-core's
 * `slot_position` spreads them along the object's width (a bunk's two a hair apart, the upper one right).
 */
export function slotOf(o: ObjectPlacement, def: { slots?: number; bunk?: boolean } | undefined): (x: number, z: number) => number {
  const n = Math.max(1, def?.slots ?? 1);
  const yaw = (o.rot * Math.PI) / 2;
  const width = o.size ? o.size[0] / 100 : o.rot % 2 ? o.d : o.w;
  const spread = def?.bunk ? 0.04 : width / n;
  return (x, z) => {
    const across = (x - o.x - o.w / 2) * Math.cos(yaw) - (z - o.z - o.d / 2) * Math.sin(yaw);
    return Math.min(n - 1, Math.max(0, Math.round(across / spread + (n - 1) / 2)));
  };
}

/**
 * Writes an instance's moving parts where the clips playing (`playing`, each at its seconds) have
 * them together (`partPose`; at rest where none does), after its placement (`writeMoving`, which
 * `written` is passed on to). The game's objects, and the colour studio's stage.
 */
export function poseParts(target: PopTarget, playing: readonly { clip: ModelClip; time: number; kind?: string }[], written?: Set<Mesh>): void {
  const moving = target.moving!;
  for (let part = 0; part < moving.parts.length; part++) if (!partPose(playing, part, moving.poses, part * 16)) identity(moving.poses, part * 16);
  writeMoving(target, written);
}

/** Poses for `parts` moving parts, all at rest. */
export function restPoses(parts: number): Float32Array {
  const poses = new Float32Array(parts * 16);
  for (let k = 0; k < parts; k++) identity(poses, k * 16);
  return poses;
}

function identity(out: Float32Array, offset: number): void {
  out.fill(0, offset, offset + 16);
  for (let i = 0; i < 4; i++) out[offset + i * 5] = 1;
}
