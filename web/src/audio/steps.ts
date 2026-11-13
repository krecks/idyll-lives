/**
 * Footsteps, synthesised (no sound files): one step on a wooden floor, tiles, carpet, grass or paving,
 * in shoes or barefoot, a few versions of each so a walk isn't one sound over and over. No DOM or audio
 * context here: a step is samples made at a sample rate, the same every time; `footsteps.ts` plays them
 * as the residents in view put their feet down.
 *
 * A step is the heel coming down and, a moment later, the ball of the foot: on a hard floor each a
 * click of the sole (a short band of noise) over the floor's own knock (a low tone that dies at once: a
 * board lower and longer than a tile); on carpet a dull pat with no click; on grass the blades brushed
 * aside (a soft swell of high noise) over soft ground; on paving the grit under the sole. Barefoot there
 * is no sole to click: a soft slap on a hard floor, next to nothing on carpet and grass.
 */

import { Biquad, random } from './water';

export type Surface = 'wood' | 'tile' | 'carpet' | 'grass' | 'paving';
export type Footwear = 'shoe' | 'bare';

export const SURFACES: readonly Surface[] = ['wood', 'tile', 'carpet', 'grass', 'paving'];
export const FOOTWEAR: readonly Footwear[] = ['shoe', 'bare'];
/** Versions of each step. */
export const STEP_VERSIONS = 4;
/** Seconds a step lasts at most. */
const SECONDS = 0.3;

/** A low knock: a tone at `hz` (starting a third higher, as a struck thing does) gone in a few times `decay` s. */
function knock(d: Float32Array, rate: number, at: number, hz: number, decay: number, amp: number): void {
  const s = Math.round(at * rate);
  let phase = 0;
  for (let i = s; i < d.length; i++) {
    const t = (i - s) / rate;
    if (t > decay * 7) break;
    phase += (2 * Math.PI * hz * (1 + 0.33 * Math.exp(-t / 0.008))) / rate;
    d[i] += amp * Math.sin(phase) * Math.exp(-t / decay) * Math.min(1, t / 0.0015);
  }
}

/** A puff of noise through a filter, up in `attack` s and gone in a few times `decay` s. */
function puff(d: Float32Array, rate: number, r: () => number, at: number, type: 'lowpass' | 'highpass' | 'bandpass', hz: number, q: number, attack: number, decay: number, amp: number): void {
  const f = new Biquad(rate).set(type, hz, q);
  const s = Math.round(at * rate);
  for (let i = s; i < d.length; i++) {
    const t = (i - s) / rate;
    if (t > attack + decay * 7) break;
    d[i] += amp * f.run(r() * 2 - 1) * Math.min(1, t / attack) * Math.exp(-Math.max(0, t - attack) / decay);
  }
}

/**
 * One step on `surface` in `wear`, version `version` (0 to `STEP_VERSIONS` - 1: each a little different in
 * pitch, weight and the time between heel and ball), as samples of one channel with a peak of 1.
 */
export function stepSamples(surface: Surface, wear: Footwear, version: number, rate: number): Float32Array {
  const r = random(0x57e9 + SURFACES.indexOf(surface) * 64 + FOOTWEAR.indexOf(wear) * 16 + version);
  const d = new Float32Array(Math.round(SECONDS * rate));
  // This version's own pitch and weight, and when the ball of the foot follows the heel.
  const tune = 0.9 + 0.2 * r();
  const heel = 0.85 + 0.3 * r();
  const ball = 0.065 + 0.03 * r();
  const at = 0.004;
  if (wear === 'bare') {
    switch (surface) {
      case 'wood':
        puff(d, rate, r, at, 'lowpass', 900 * tune, 0.7, 0.003, 0.022, 0.8 * heel);
        knock(d, rate, at, 95 * tune, 0.04, 0.5 * heel);
        puff(d, rate, r, at + ball, 'lowpass', 1200 * tune, 0.7, 0.003, 0.016, 0.45);
        break;
      case 'tile':
        puff(d, rate, r, at, 'lowpass', 1500 * tune, 0.7, 0.002, 0.018, 0.9 * heel);
        knock(d, rate, at, 130 * tune, 0.02, 0.25 * heel);
        puff(d, rate, r, at + ball, 'bandpass', 1900 * tune, 0.8, 0.002, 0.014, 0.6);
        break;
      case 'carpet':
        puff(d, rate, r, at, 'lowpass', 320 * tune, 0.7, 0.005, 0.035, 1 * heel);
        puff(d, rate, r, at + ball, 'lowpass', 420 * tune, 0.7, 0.005, 0.025, 0.4);
        break;
      case 'grass':
        puff(d, rate, r, at, 'bandpass', 2600 * tune, 0.5, 0.014, 0.06, 0.5 * heel);
        puff(d, rate, r, at, 'lowpass', 260 * tune, 0.7, 0.005, 0.04, 0.8 * heel);
        break;
      case 'paving':
        puff(d, rate, r, at, 'lowpass', 750 * tune, 0.7, 0.003, 0.02, 0.9 * heel);
        knock(d, rate, at, 100 * tune, 0.02, 0.3 * heel);
        puff(d, rate, r, at + ball, 'lowpass', 950 * tune, 0.7, 0.003, 0.015, 0.45);
        break;
    }
  } else {
    switch (surface) {
      case 'wood':
        knock(d, rate, at, 110 * tune, 0.045, 0.9 * heel);
        knock(d, rate, at, 215 * tune, 0.028, 0.4 * heel);
        puff(d, rate, r, at, 'bandpass', 1700 * tune, 1.2, 0.001, 0.012, 1.6 * heel);
        knock(d, rate, at + ball, 145 * tune, 0.03, 0.35);
        puff(d, rate, r, at + ball, 'bandpass', 2200 * tune, 1.2, 0.001, 0.008, 0.8);
        break;
      case 'tile':
        puff(d, rate, r, at, 'bandpass', 3200 * tune, 1.5, 0.0008, 0.01, 2.4 * heel);
        puff(d, rate, r, at, 'highpass', 5000, 0.7, 0.0005, 0.004, 0.3 * heel);
        knock(d, rate, at, 165 * tune, 0.02, 0.35 * heel);
        puff(d, rate, r, at + ball, 'bandpass', 3800 * tune, 1.5, 0.0008, 0.007, 1.1);
        knock(d, rate, at + ball, 195 * tune, 0.015, 0.15);
        break;
      case 'carpet':
        puff(d, rate, r, at, 'lowpass', 420 * tune, 0.7, 0.004, 0.04, 1 * heel);
        knock(d, rate, at, 80 * tune, 0.04, 0.5 * heel);
        puff(d, rate, r, at + ball, 'lowpass', 600 * tune, 0.7, 0.004, 0.03, 0.4);
        break;
      case 'grass':
        puff(d, rate, r, at, 'bandpass', 3500 * tune, 0.6, 0.012, 0.07, 0.9 * heel);
        puff(d, rate, r, at, 'lowpass', 300 * tune, 0.7, 0.004, 0.04, 0.6 * heel);
        puff(d, rate, r, at + ball, 'bandpass', 4200 * tune, 0.6, 0.01, 0.045, 0.45);
        break;
      case 'paving':
        puff(d, rate, r, at, 'bandpass', 1400 * tune, 0.8, 0.002, 0.03, 1.4 * heel);
        puff(d, rate, r, at, 'highpass', 3500, 0.7, 0.001, 0.012, 0.25 * heel);
        knock(d, rate, at, 120 * tune, 0.025, 0.5 * heel);
        puff(d, rate, r, at + ball, 'bandpass', 1800 * tune, 0.8, 0.002, 0.02, 0.7);
        break;
    }
  }
  // Eased to nothing at its end, then a peak of 1.
  const ease = Math.round(0.03 * rate);
  for (let i = 0; i < ease; i++) d[d.length - 1 - i] *= i / ease;
  let peak = 0;
  for (const v of d) peak = Math.max(peak, Math.abs(v));
  if (peak > 0) for (let i = 0; i < d.length; i++) d[i] /= peak;
  return d;
}
