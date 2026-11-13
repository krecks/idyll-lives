/**
 * The space a bus's sounds are heard in (`acoustics.ts` has the rules): each sound's place left to right,
 * the wall it may be behind and the room it is made in (`Placed`), then, for the world's sounds, the
 * camera's distance (`Space`).
 *
 *   sound → [wall: low-pass, level] → panner ─┬──────────────────────────────→ [distance: low-pass, level] → bus
 *                                             └→ send → room reverb (by size) ─┘
 *
 * A room's reverb is a `ConvolverNode` with a response made in code (`rooms.ts`), one per size, made the
 * first time a sound is heard in a room that size. What goes into it is the sound's two sides added
 * (one channel), what comes out is wide (the response's two channels differ), so the room is all round
 * whichever side the sound is on. It joins the bus before the distance, so a far room is far too.
 *
 * Built on a `BaseAudioContext`, so the same graph renders in an `OfflineAudioContext` to be measured.
 */

import { DISTANCE, distanceCutoff, distanceGain, muffled } from './acoustics';
import { ROOM_SIZES, ROOMS, roomResponse, type RoomSize } from './rooms';

/** Seconds for a place, a wall or the distance to ease to a new value (a third of it is the time constant). */
const EASE = 0.3;
/** A low-pass without a bump at its cutoff (the node's Q is in dB: 3 dB down there). */
const FLAT_Q = -3.01;

/** Where a bus's placed sounds play into, and its rooms. */
export class Space {
  /** What every placed sound and every room joins the bus through: the distance's low-pass and level, or straight in. */
  readonly dry: AudioNode;
  private readonly level: GainNode;
  private readonly dull: BiquadFilterNode | null = null;
  private readonly rooms = new Map<RoomSize, GainNode>();

  /** `far`: the camera's distance changes these sounds (`distance`); else they are as near as made. */
  constructor(
    private readonly ac: BaseAudioContext,
    out: AudioNode,
    far: boolean,
  ) {
    this.level = ac.createGain();
    this.level.connect(out);
    if (far) {
      this.dull = ac.createBiquadFilter();
      this.dull.type = 'lowpass';
      this.dull.Q.value = FLAT_Q;
      this.dull.frequency.value = Math.min(DISTANCE.open, ac.sampleRate / 2);
      this.dull.connect(this.level);
    }
    this.dry = this.dull ?? this.level;
  }

  /** The way into the reverb of a room this size (made the first time). */
  room(size: RoomSize): AudioNode {
    let send = this.rooms.get(size);
    if (!send) {
      const { left, right } = roomResponse(size, this.ac.sampleRate);
      const response = this.ac.createBuffer(2, left.length, this.ac.sampleRate);
      response.copyToChannel(left as Float32Array<ArrayBuffer>, 0);
      response.copyToChannel(right as Float32Array<ArrayBuffer>, 1);
      const reverb = this.ac.createConvolver();
      // (The response's energy is 1 already: the node's own levelling would only move it.)
      reverb.normalize = false;
      reverb.buffer = response;
      // Both sides added into one channel: the room answers the same wherever the sound stands.
      send = new GainNode(this.ac, { gain: ROOMS[size].send, channelCount: 1, channelCountMode: 'explicit', channelInterpretation: 'speakers' });
      send.connect(reverb).connect(this.dry);
      this.rooms.set(size, send);
    }
    return send;
  }

  /** The camera is `radius` m from what it looks at: the sounds' level and highs (`acoustics.ts`), eased from `t`; `now`: at once. */
  distance(radius: number, t: number, now = false): void {
    if (!this.dull) return;
    const gain = distanceGain(radius);
    const cutoff = Math.min(distanceCutoff(radius), this.ac.sampleRate / 2);
    if (now) {
      this.level.gain.value = gain;
      this.dull.frequency.value = cutoff;
      return;
    }
    this.level.gain.setTargetAtTime(gain, t, EASE / 3);
    this.dull.frequency.setTargetAtTime(cutoff, t, EASE / 3);
  }
}

/** Where a sound is: left to right, behind a wall or not, and the rooms it is made in. */
export interface Place {
  /** -1 (left) .. 1 (right). */
  pan: number;
  /** 0: in the room in view; 1: through a wall (`acoustics.ts` `muffled`). */
  muffle?: number;
  /** The share of it made in a room of each size (a loop of several users in different rooms; the rest outdoors: dry). */
  rooms?: Partial<Record<RoomSize, number>>;
}

/** A sound's own part of the graph: plays `input` at its place in `space`. */
export class Placed {
  readonly input: GainNode;
  private readonly wall: BiquadFilterNode;
  private readonly panner: StereoPannerNode;
  private readonly sends = new Map<RoomSize, GainNode>();

  constructor(
    private readonly ac: BaseAudioContext,
    private readonly space: Space,
    place: Place,
  ) {
    this.input = ac.createGain();
    this.wall = ac.createBiquadFilter();
    this.wall.type = 'lowpass';
    this.wall.Q.value = FLAT_Q;
    this.panner = ac.createStereoPanner();
    this.input.connect(this.wall).connect(this.panner).connect(space.dry);
    this.place(place, 0, true);
  }

  /** Moves it (eased from `t`; `now`: at once). */
  place(place: Place, t: number, now = false): void {
    const pan = Number.isFinite(place.pan) ? Math.max(-1, Math.min(1, place.pan)) : 0;
    const { cutoff, gain } = muffled(place.muffle ?? 0);
    const hz = Math.min(cutoff, this.ac.sampleRate / 2);
    if (now) {
      this.panner.pan.value = pan;
      this.wall.frequency.value = hz;
      this.input.gain.value = gain;
    } else {
      this.panner.pan.setTargetAtTime(pan, t, EASE / 3);
      this.wall.frequency.setTargetAtTime(hz, t, EASE / 3);
      this.input.gain.setTargetAtTime(gain, t, EASE / 3);
    }
    for (const size of ROOM_SIZES) {
      const share = place.rooms?.[size] ?? 0;
      let send = this.sends.get(size);
      if (!send) {
        if (share <= 0) continue;
        send = this.ac.createGain();
        send.gain.value = now ? share : 0;
        this.panner.connect(send).connect(this.space.room(size));
        this.sends.set(size, send);
      }
      if (now) send.gain.value = share;
      else send.gain.setTargetAtTime(share, t, EASE / 3);
    }
  }

  /** Takes it out of the graph (its sound has ended). */
  disconnect(): void {
    this.panner.disconnect();
    for (const send of this.sends.values()) send.disconnect();
  }
}
