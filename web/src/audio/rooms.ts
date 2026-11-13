/**
 * The rooms' reverb, synthesised (no sound files): what a room of a size gives back of a sound made in
 * it, as an impulse response the mix convolves the sound with (`space.ts`, a `ConvolverNode`). No DOM or
 * audio context here: a response is samples made at a sample rate, the same every time.
 *
 * A response is what a furnished room at home does, not a hall: a few early echoes off the walls (their
 * times from the room's width), then a dense tail that dies away in a few tenths of a second, its highs
 * sooner than its lows (soft furniture and air take them first) and nothing under about 200 Hz (a room
 * that small has no reverb down there, only a boom). It holds the echoes only, not the sound itself (it
 * is a send: the sound goes its own way dry), and its energy is 1: a steady sound comes back from it as
 * loud as it went in, so a send's gain is how loud the room is against the sound (`ROOMS[size].send`).
 */

import { random, type Stereo } from './water';

export type RoomSize = 'small' | 'medium' | 'large';

export interface RoomDef {
  /** A room up to this floor area (m², the game's tiles) is this size. */
  area: number;
  /** Metres across, for the early echoes' times. */
  width: number;
  /** Seconds for the tail to fall by 60 dB (its lows; the highs take `bright` of that). */
  decay: number;
  bright: number;
  /** How loud the room is against the sound made in it (a gain: 0.2 is 14 dB under). */
  send: number;
}

/**
 * The sizes: a bathroom, a hall or a pantry (tiled as often as not: short and bright); a bedroom, a
 * kitchen, a study; a living room or anything bigger. Decays as measured in furnished homes (0.3 to
 * 0.6 s), each a quiet 13 to 15 dB under the sound.
 */
export const ROOMS: Record<RoomSize, RoomDef> = {
  small: { area: 9, width: 2.4, decay: 0.32, bright: 0.75, send: 0.2 },
  medium: { area: 28, width: 4.2, decay: 0.42, bright: 0.6, send: 0.18 },
  large: { area: Infinity, width: 7, decay: 0.6, bright: 0.55, send: 0.22 },
};

export const ROOM_SIZES = Object.keys(ROOMS) as RoomSize[];

/** The size of a room of `area` m². */
export function roomSize(area: number): RoomSize {
  return ROOM_SIZES.find((s) => area <= ROOMS[s].area) ?? 'large';
}

/** Metres a second. */
const SOUND = 343;
/** The tail's highs and lows part here (Hz), and nothing comes back under `FLOOR` Hz. */
const SPLIT = 2500;
const FLOOR = 200;

/** A room's impulse response (two channels, each its own echoes so the room is wide), its energy 1 a channel on average. */
export function roomResponse(size: RoomSize, rate: number): Stereo {
  const def = ROOMS[size];
  const n = Math.round(def.decay * 1.15 * rate);
  const out: Stereo = { left: new Float32Array(n), right: new Float32Array(n) };
  const seed = 0x700 + ROOM_SIZES.indexOf(size) * 16;
  // The first echo comes back after the way to the nearest wall and back.
  const first = def.width / 2 / SOUND;
  for (const [c, ch] of [out.left, out.right].entries()) {
    const r = random(seed + c);
    // Early echoes: six, each a little later and weaker than the last, off one wall or another.
    let at = first * (0.8 + 0.4 * r());
    for (let k = 0; k < 6; k++) {
      const i = Math.round(at * rate);
      if (i < n) ch[i] += (r() < 0.5 ? -1 : 1) * 0.9 * 0.72 ** k;
      at += (def.width / SOUND) * (0.45 + 0.6 * r());
    }
    // The tail: noise, its lows dying away over `decay`, its highs sooner, swelling in as the echoes thicken.
    const k = 1 - Math.exp((-2 * Math.PI * SPLIT) / rate);
    const swell = first * 2.5;
    let low = 0;
    for (let i = Math.round(first * rate); i < n; i++) {
      const t = i / rate;
      const w = (r() + r() + r() - 1.5) * 1.15;
      low += (w - low) * k;
      const rise = Math.min(1, (t - first) / swell);
      ch[i] += 0.5 * rise * (low * 1.6 * Math.exp((-6.91 * t) / def.decay) + (w - low) * Math.exp((-6.91 * t) / (def.decay * def.bright)));
    }
    // Nothing under the floor (two passes of a gentle high-pass), and the very end eased to nothing.
    const h = Math.exp((-2 * Math.PI * FLOOR) / rate);
    for (let pass = 0; pass < 2; pass++) {
      let x1 = 0;
      let y1 = 0;
      for (let i = 0; i < n; i++) {
        const y = h * (y1 + ch[i] - x1);
        x1 = ch[i];
        y1 = y;
        ch[i] = y;
      }
    }
    const ease = Math.round(0.02 * rate);
    for (let i = 0; i < ease; i++) ch[n - 1 - i] *= i / ease;
  }
  let sum = 0;
  for (const ch of [out.left, out.right]) for (const v of ch) sum += v * v;
  const scale = 1 / Math.sqrt(sum / 2);
  for (const ch of [out.left, out.right]) for (let i = 0; i < n; i++) ch[i] *= scale;
  return out;
}
