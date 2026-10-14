/**
 * Placed objects' moving parts (a fridge's doors): the clips of their models (`ModelMotion`)
 * played while residents use them, as the model entry's `motions` say. A use that starts opens
 * its clips; each plays back after its `hold` seconds or when the use ends. A loop (`loops`: a
 * rocking chair) goes round from rest to rest while the use lasts, then finishes its round. An
 * object that is open (the sim's `open`: a wall bed folded down) holds its `motions.open` clip
 * open as long as it stays so, and its `motions.night` clip (curtains drawn) from dusk to dawn,
 * whoever is asleep. A crop's `motions.growth` clip is held where its growth stage is, moving on
 * to the next stage at the clip's own pace. An ambient clip (`idle`: a ceiling fan, a fountain's water) goes
 * round all the time, but only within `ambientRange` of the camera (none on Low), without the
 * shadows being redrawn for it. Each instance's parts draw from buffers of their own
 * (`MovingTarget`), written only while something moves.
 *
 * Time is the residents' animation time: it stops while the game is paused and runs faster
 * (up to 2.5 times) at higher speeds, as in `characters.ts`.
 */

import { writeMoving, type PopTarget } from './buildFx';
import { Matrix, type Mesh, type Vector3 } from './core';
import { GAME_RATE, QUIET_SPEED } from './characters';
import { clipPose, type ModelClip, type ModelMotion, type Motions } from './models';
import type { FrameState } from '../../core/bridge';

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
}

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
  /** Opens and closes (a use's, or held open), goes round while used (`loops`), or goes round always (`idle`). */
  kind: 'use' | 'loop' | 'idle' | 'growth';
}

/** A clip a use wants this frame. */
interface Wanted {
  /** The use (resident, action, interaction). */
  use: string;
  /** Seconds it stays open after the use starts (undefined: while it lasts). */
  hold?: number;
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
        clips.set(growth.name, { clip: growth.clip, time: growth.at, use: '', until: Infinity, kind: 'growth' });
      }
      for (const name of this.held(o)) {
        const clip = o.motion.clips.get(name);
        if (!clip) continue;
        let clips = this.states.get(o.id);
        if (!clips) this.states.set(o.id, (clips = new Map()));
        if (!clips.has(name)) clips.set(name, { clip, time: clip.duration, use: '', until: Infinity, kind: 'use' });
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
        if (clip && !clips.has(name)) clips.set(name, { clip, time: 0, use: '', until: Infinity, kind: 'use' });
      }
      // The first lighting of a world: night clips as it is, without playing.
      const snap = this.snapNight && o.motions.night !== undefined ? o.motions.night : undefined;
      if (growth && !clips.has(growth.name)) clips.set(growth.name, { clip: growth.clip, time: growth.at, use: '', until: Infinity, kind: 'growth' });
      const idleClip = idle ? o.motion.clips.get(idle) : undefined;
      if (idle && idleClip && !clips.has(idle)) clips.set(idle, { clip: idleClip, time: 0, use: '', until: Infinity, kind: 'idle' });
      let changed = false;
      let drifted = false;
      for (const [name, want] of wanted ?? []) {
        const clip = o.motion.clips.get(name);
        if (!clip) continue;
        const state = clips.get(name);
        if (state?.use === want.use) continue;
        // A new use: open (again), or go round.
        const until = want.hold === undefined ? Infinity : this.clock + want.hold;
        if (state) Object.assign(state, { use: want.use, until, kind: want.kind });
        else clips.set(name, { clip, time: 0, use: want.use, until, kind: want.kind });
      }
      for (const [name, state] of clips) {
        const d = state.clip.duration;
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
        const open = held.includes(name) || (using && this.clock < state.until);
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
    const state = name ? this.states.get(id)?.get(name) : undefined;
    const part = state?.clip.tracks[0]?.part;
    if (!o || !state || part === undefined || !clipPose(state.clip, part, state.time, this.rideTmp)) return null;
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
      for (const { clip, hold } of clips ?? []) wanted.set(clip, { use, hold, kind: 'use' });
      if (loop) wanted.set(loop, { use, kind: 'loop' });
      this.wanted.set(id, wanted);
    }
  }

  /** Writes an object's parts where its clips have them (at rest where none does); `written` takes the meshes whose buffers changed (else they go up now). */
  private pose(o: MovingObject, clips: Map<string, ClipState>, written?: Set<Mesh>): void {
    const moving = o.target.moving!;
    const n = moving.parts.length;
    for (let part = 0; part < n; part++) {
      let posed = false;
      for (const state of clips.values()) posed = clipPose(state.clip, part, state.time, moving.poses, part * 16) || posed;
      if (!posed) identity(moving.poses, part * 16);
    }
    writeMoving(o.target, written);
  }
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
