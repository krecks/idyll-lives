/**
 * Placed objects' moving parts (a fridge's doors): the clips of their models (`ModelMotion`)
 * played while residents use them, as the model entry's `motions` say. A use that starts opens
 * the clip; it plays back after `hold` seconds or when the use ends. Each instance's parts draw
 * from buffers of their own (`MovingTarget`), written only while something moves.
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
}

export class ObjectMotion {
  private objects = new Map<number, MovingObject>();
  /** By object id, then clip name. Kept across world rebuilds, so a door mid-swing carries on. */
  private states = new Map<number, Map<string, ClipState>>();
  private clock = 0;
  private lastNow = 0;
  /** Uses this frame: by object id, the clip names and the use that plays each. */
  private readonly wanted = new Map<number, Map<string, string>>();

  /** The objects of a newly built world; those still mid-clip are posed again. */
  set(objects: MovingObject[]): void {
    this.objects = new Map(objects.map((o) => [o.id, o]));
    for (const id of [...this.states.keys()]) if (!this.objects.has(id)) this.states.delete(id);
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
      let clips = this.states.get(id);
      if (wanted && !clips) this.states.set(id, (clips = new Map()));
      if (!clips) continue;
      let changed = false;
      for (const [name, use] of wanted ?? []) {
        const clip = o.motion.clips.get(name);
        if (!clip) continue;
        const state = clips.get(name);
        if (state?.use === use) continue;
        // A new use: open (again).
        const until = o.motions.hold === undefined ? Infinity : this.clock + o.motions.hold;
        if (state) Object.assign(state, { use, until });
        else clips.set(name, { clip, time: 0, use, until });
      }
      for (const [name, state] of clips) {
        const open = (wanted?.get(name) ?? '') === state.use && this.clock < state.until;
        if (!open && !wanted?.has(name)) state.use = '';
        const time = Math.min(Math.max(state.time + (open ? dt : -dt), 0), state.clip.duration);
        if (time !== state.time) changed = true;
        state.time = time;
        if (!time && !open) clips.delete(name);
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
      const uses = object.motions.uses;
      const clip = uses[object.interactions[interaction] ?? ''] ?? uses[actionTags[act] ?? ''];
      if (!clip) continue;
      const wanted = this.wanted.get(id) ?? new Map<string, string>();
      wanted.set(clip, `${curr[o + k.id]}:${act}:${interaction}`);
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
