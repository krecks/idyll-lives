/**
 * Placed objects' moving parts (a fridge's doors): the clips of their models (`ModelMotion`)
 * played while residents use them, as the model entry's `motions` say. A use that starts opens
 * its clips; each plays back after its `hold` seconds or when the use ends. A loop (`loops`: a
 * rocking chair) goes round from rest to rest while the use lasts, then finishes its round. An
 * object that is open (the sim's `open`: a wall bed folded down) holds its `motions.open` clip
 * open as long as it stays so. Each instance's parts draw from buffers of their own
 * (`MovingTarget`), written only while something moves.
 *
 * Time is the residents' animation time: it stops while the game is paused and runs faster
 * (up to 2.5 times) at higher speeds, as in `characters.ts`.
 */

import { writeMoving, type PopTarget } from './buildFx';
import type { Mesh } from './core';
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
}

/** A clip of one object: how far it has played, and what plays it. */
interface ClipState {
  clip: ModelClip;
  /** Seconds into the clip (0: at rest). */
  time: number;
  /** The use that opened it (resident, action, interaction), so a new one opens it again. */
  use: string;
  /** Animation time it plays back from (Infinity: when the use ends). */
  until: number;
  /** Goes round while used (`loops`), rather than opening and closing. */
  loop: boolean;
}

/** A clip a use wants this frame. */
interface Wanted {
  /** The use (resident, action, interaction). */
  use: string;
  /** Seconds it stays open after the use starts (undefined: while it lasts). */
  hold?: number;
  loop: boolean;
}

export class ObjectMotion {
  private objects = new Map<number, MovingObject>();
  /** By object id, then clip name. Kept across world rebuilds, so a door mid-swing carries on. */
  private states = new Map<number, Map<string, ClipState>>();
  private clock = 0;
  private lastNow = 0;
  /** Uses this frame: by object id, the clips wanted by clip name. */
  private readonly wanted = new Map<number, Map<string, Wanted>>();

  /** The objects of a newly built world; those still mid-clip are posed again. */
  set(objects: MovingObject[]): void {
    const before = this.objects;
    this.objects = new Map(objects.map((o) => [o.id, o]));
    for (const id of [...this.states.keys()]) if (!this.objects.has(id)) this.states.delete(id);
    // What is open when first seen (a game loaded, a lot looked at) is shown open, without the
    // opening; what opens later (`update`) opens on screen.
    for (const o of objects) {
      const name = o.open ? o.motions.open : undefined;
      const clip = name ? o.motion.clips.get(name) : undefined;
      if (!name || !clip || before.has(o.id)) continue;
      let clips = this.states.get(o.id);
      if (!clips) this.states.set(o.id, (clips = new Map()));
      if (!clips.has(name)) clips.set(name, { clip, time: clip.duration, use: '', until: Infinity, loop: false });
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
  }

  /** Plays this frame's clips. True if something moved (its meshes' bounds and shadows need refreshing). */
  update(frame: FrameState, actionTags: readonly string[]): boolean {
    const { curr, layout, now } = frame;
    let dt = this.lastNow ? Math.min(Math.max((now - this.lastNow) / 1000, 0), 0.1) : 0;
    this.lastNow = now;
    if (!this.objects.size) return false;
    const speed = curr[layout.header.speed] | 0;
    const quiet = layout.header.calm !== undefined && curr[layout.header.calm] > 0;
    const gameSpeed = quiet ? Math.max(speed, QUIET_SPEED) : speed;
    if (gameSpeed === 0) dt = 0;
    dt *= Math.min(GAME_RATE[gameSpeed] ?? 1, 2.5);
    this.clock += dt;

    this.findUses(frame, actionTags);
    const moved = new Set<Mesh>();
    for (const [id, o] of this.objects) {
      const wanted = this.wanted.get(id);
      const held = o.open ? o.motions.open : undefined;
      let clips = this.states.get(id);
      if ((wanted || held) && !clips) this.states.set(id, (clips = new Map()));
      if (!clips) continue;
      const heldClip = held ? o.motion.clips.get(held) : undefined;
      if (held && heldClip && !clips.has(held)) clips.set(held, { clip: heldClip, time: 0, use: '', until: Infinity, loop: false });
      let changed = false;
      for (const [name, want] of wanted ?? []) {
        const clip = o.motion.clips.get(name);
        if (!clip) continue;
        const state = clips.get(name);
        if (state?.use === want.use) continue;
        // A new use: open (again), or go round.
        const until = want.hold === undefined ? Infinity : this.clock + want.hold;
        if (state) Object.assign(state, { use: want.use, until, loop: want.loop });
        else clips.set(name, { clip, time: 0, use: want.use, until, loop: want.loop });
      }
      for (const [name, state] of clips) {
        const using = state.use !== '' && wanted?.get(name)?.use === state.use;
        // (A use that ended leaves `use` empty: nothing wanted then is no use keeping it open.)
        if (!using && !wanted?.has(name)) state.use = '';
        const d = state.clip.duration;
        if (state.loop && name !== held) {
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
        const open = name === held || (using && this.clock < state.until);
        const time = Math.min(Math.max(state.time + (open ? dt : -dt), 0), d);
        if (time !== state.time) changed = true;
        state.time = time;
        // Closed again after its `hold`, it stays so (kept) until the use ends; a new use opens it again.
        if (!time && !open && !using) clips.delete(name);
      }
      if (changed) {
        this.pose(o, clips);
        for (const part of o.target.moving!.parts) for (const mesh of part.meshes) moved.add(mesh);
      }
      if (!clips.size) this.states.delete(id);
    }
    // Culled by where the parts are now (an open door reaches out of the closed model's box).
    for (const mesh of moved) if (!mesh.isDisposed()) mesh.thinInstanceRefreshBoundingInfo(false);
    return moved.size > 0;
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
      for (const { clip, hold } of clips ?? []) wanted.set(clip, { use, hold, loop: false });
      if (loop) wanted.set(loop, { use, loop: true });
      this.wanted.set(id, wanted);
    }
  }

  /** Writes an object's parts where its clips have them (at rest where none does). */
  private pose(o: MovingObject, clips: Map<string, ClipState>): void {
    const moving = o.target.moving!;
    const n = moving.parts.length;
    for (let part = 0; part < n; part++) {
      let posed = false;
      for (const state of clips.values()) posed = clipPose(state.clip, part, state.time, moving.poses, part * 16) || posed;
      if (!posed) identity(moving.poses, part * 16);
    }
    writeMoving(o.target);
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
