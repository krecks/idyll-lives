/**
 * What the mix's space does to a sound (`space.ts`: a wall, a room's reverb, the camera's distance), worked
 * out on the samples off the browser: for the tools and the tests (`tools/audio/listen.ts`,
 * `web/tests/sounds.test.ts`), not something the game runs. The same arithmetic as the graph's nodes, so
 * what is measured here is what the browser plays (compared against Chromium's rendering of the graph).
 */

import { distanceCutoff, distanceGain, muffled } from './acoustics';
import { nodeFilter } from './mono';
import { ROOMS, roomResponse, type RoomSize } from './rooms';
import type { Stereo } from './water';

/** The DFT of `re` + i·`im`, in place (radix-2: the length a power of two); `inverse`: back again. */
export function fft(re: Float64Array, im: Float64Array, inverse = false): void {
  const n = re.length;
  // Bit-reversed order, then the butterflies, each stage twice as wide.
  for (let i = 1, j = 0; i < n; i++) {
    let bit = n >> 1;
    for (; j & bit; bit >>= 1) j ^= bit;
    j ^= bit;
    if (i < j) {
      [re[i], re[j]] = [re[j], re[i]];
      [im[i], im[j]] = [im[j], im[i]];
    }
  }
  for (let len = 2; len <= n; len <<= 1) {
    const half = len >> 1;
    const step = ((inverse ? 2 : -2) * Math.PI) / len;
    for (let k = 0; k < half; k++) {
      const wr = Math.cos(step * k);
      const wi = Math.sin(step * k);
      for (let i = k; i < n; i += len) {
        const j = i + half;
        const tr = re[j] * wr - im[j] * wi;
        const ti = re[j] * wi + im[j] * wr;
        re[j] = re[i] - tr;
        im[j] = im[i] - ti;
        re[i] += tr;
        im[i] += ti;
      }
    }
  }
  if (inverse) {
    for (let i = 0; i < n; i++) {
      re[i] /= n;
      im[i] /= n;
    }
  }
}

/**
 * `x` convolved with the response `h` (what a `ConvolverNode` does). `loop`: `x` is a loop, and the
 * response's tail runs on from its end into its start (as long as `x`); else the tail follows `x`
 * (`x.length + h.length - 1` samples).
 */
export function convolve(x: Float32Array, h: Float32Array, loop: boolean): Float32Array {
  const full = x.length + h.length - 1;
  let n = 1;
  while (n < full) n <<= 1;
  const xr = new Float64Array(n);
  const xi = new Float64Array(n);
  const hr = new Float64Array(n);
  const hi = new Float64Array(n);
  xr.set(x);
  hr.set(h);
  fft(xr, xi);
  fft(hr, hi);
  for (let k = 0; k < n; k++) {
    const re = xr[k] * hr[k] - xi[k] * hi[k];
    xi[k] = xr[k] * hi[k] + xi[k] * hr[k];
    xr[k] = re;
  }
  fft(xr, xi, true);
  if (!loop) return Float32Array.from(xr.subarray(0, full));
  const out = new Float32Array(x.length);
  for (let i = 0; i < full; i++) out[i % x.length] += xr[i];
  return out;
}

/** Where a sound is heard from (`space.ts` `Place` and the camera's distance). */
export interface Hearing {
  /** The room it is made in (its reverb); none: outdoors. */
  room?: RoomSize;
  /** 0: in the room in view; 1: through a wall. */
  muffle?: number;
  /** The camera's distance (m); none: the game's own zoom. */
  radius?: number;
}

/**
 * A sound as heard in the middle of the picture (`catalog.ts` `Listed.heard`) from `from`: behind its wall,
 * with its room, at the camera's distance. `loop`: it is one (the room's tail and the filters run on round
 * its seam); a sound played once comes back longer by its room's tail.
 */
export function spaced(sound: Stereo, rate: number, from: Hearing, loop: boolean): Stereo {
  const wall = muffled(from.muffle ?? 0);
  const low = (x: Float32Array, hz: number) => (hz >= rate / 2 ? x : nodeFilter(x, rate, 'lowpass', hz, -3.01, loop));
  let left = low(sound.left, wall.cutoff).map((v) => v * wall.gain);
  let right = low(sound.right, wall.cutoff).map((v) => v * wall.gain);
  if (from.room) {
    const response = roomResponse(from.room, rate);
    // Both sides added into one channel on the way in (the send's down-mix: half of each).
    const send = left.map((v, i) => (v + right[i]) * 0.5 * ROOMS[from.room!].send);
    const wet = [convolve(send, response.left, loop), convolve(send, response.right, loop)];
    const n = wet[0].length;
    const sum = (dry: Float32Array, w: Float32Array) => Float32Array.from({ length: n }, (_, i) => (dry[i] ?? 0) + w[i]);
    left = sum(left, wet[0]);
    right = sum(right, wet[1]);
  }
  if (from.radius !== undefined) {
    const gain = distanceGain(from.radius);
    const hz = distanceCutoff(from.radius);
    left = low(left, hz).map((v) => v * gain);
    right = low(right, hz).map((v) => v * gain);
  }
  return { left, right };
}
