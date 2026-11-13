/**
 * The world's sounds of one channel, synthesised (no sound files): a sizzling pan, a knife on a board, a
 * television's murmur, drums, the outdoor bed of wind, crickets and a knock at the door. Each is made
 * whole here, its filter included, as samples at a sample rate (no DOM or audio context), so the game
 * plays exactly what `bun run sounds` renders and the tests measure; `world.ts` gives each a place
 * left to right (a stereo panner) and its level (`catalog.ts`).
 *
 * They were Web Audio graphs once (a buffer through a `BiquadFilterNode`; the knock oscillators and a
 * burst of noise): `nodeFilter` and `sweep` are those nodes' own arithmetic, and what they make here is
 * what the graphs made (compared sample by sample against Chromium's rendering of them when they moved).
 */

import { random } from './water';

/** The sounds of one channel, by name. */
export const MONO = {
  sizzle: sizzleSound,
  chop: chopSound,
  tv: murmurSound,
  drums: drumsSound,
  music: musicSound,
  bed: bedSound,
  crickets: cricketSound,
  knock: knockSound,
} satisfies Record<string, (rate: number) => Float32Array>;

export type MonoKind = keyof typeof MONO;

/** Those played once (the rest loop). */
export const ONCE_MONO = ['knock'] as const satisfies readonly MonoKind[];

/** A hot pan: hiss with crackles. */
function sizzleSound(rate: number): Float32Array {
  const r = random(4);
  const d = raw(rate, 2, (d) => {
    let pop = 0;
    let popLevel = 0;
    for (let i = 0; i < d.length; i++) {
      if (pop <= 0 && r() < 60 / rate) {
        pop = Math.round((0.002 + r() * 0.006) * rate);
        popLevel = 0.3 + r() * 0.7;
      }
      const crackle = pop > 0 ? (r() * 2 - 1) * popLevel * (pop-- / (0.008 * rate)) : 0;
      d[i] = (r() * 2 - 1) * 0.18 + crackle;
    }
  });
  return nodeFilter(d, rate, 'highpass', 1800, 0.5, true);
}

/** A knife on a chopping board: short woody knocks in uneven runs. */
function chopSound(rate: number): Float32Array {
  const r = random(9);
  const d = raw(rate, 3, (d) => {
    let i = 0;
    while (i < d.length) {
      // A run of quick chops, then a pause to move the food along.
      const chops = 3 + Math.floor(r() * 6);
      for (let c = 0; c < chops && i < d.length; c++) {
        const len = Math.round(0.035 * rate);
        const level = 0.6 + r() * 0.4;
        const tone = 900 + r() * 500;
        for (let j = 0; j < len && i + j < d.length; j++) {
          const decay = Math.exp((-j / rate) * 90);
          d[i + j] = ((r() * 2 - 1) * 0.5 + Math.sin((2 * Math.PI * tone * j) / rate)) * level * decay;
        }
        i += Math.round((0.16 + r() * 0.08) * rate);
      }
      i += Math.round((0.4 + r() * 0.6) * rate);
    }
  });
  return nodeFilter(d, rate, 'bandpass', 1400, 0.8, true);
}

/** Voices through a wall: noise shaped into syllables, phrases and pauses (the TV). */
function murmurSound(rate: number): Float32Array {
  const r = random(6);
  const d = raw(rate, 5, (d) => {
    let i = 0;
    while (i < d.length) {
      // A phrase of a few syllables, then a pause.
      const syllables = 3 + Math.floor(r() * 7);
      for (let s = 0; s < syllables && i < d.length; s++) {
        const len = Math.round((0.09 + r() * 0.14) * rate);
        const level = 0.5 + r() * 0.5;
        for (let j = 0; j < len && i < d.length; j++, i++) d[i] = (r() * 2 - 1) * level * Math.sin((Math.PI * j) / len);
      }
      i += Math.round((0.15 + r() * 0.5) * rate);
    }
  });
  return nodeFilter(d, rate, 'bandpass', 700, 1.1, true);
}

/** A little tune on a loop (a radio, a record). */
function musicSound(rate: number): Float32Array {
  const r = random(7);
  const scale = [0, 2, 4, 7, 9, 12, 14, 16];
  const root = 261.63;
  const d = raw(rate, 4, (d) => {
    const pluck = (at: number, freq: number, level: number, decay: number) => {
      const s = Math.round(at * rate);
      const len = Math.min(d.length - s, Math.round(decay * 4 * rate));
      for (let j = 0; j < len; j++) {
        const t = j / rate;
        d[s + j] += level * Math.exp(-t / decay) * (Math.sin(2 * Math.PI * freq * t) + 0.3 * Math.sin(4 * Math.PI * freq * t));
      }
    };
    for (let n = 0; n < 16; n++) {
      const note = scale[Math.floor(r() * scale.length)];
      if (r() < 0.85) pluck(n * 0.25, root * 2 ** (note / 12), 0.5, 0.18);
    }
    for (let n = 0; n < 4; n++) pluck(n, (root / 2) * 2 ** ([0, 9, 5, 7][n] / 12), 0.6, 0.5);
  });
  return nodeFilter(d, rate, 'lowpass', 2600, 0.7, true);
}

/** A drum kit played: a rock beat (kick, snare on two and four, the hi-hat on the eighths) at 110 bpm, four bars with a crash at the top and a fill on the toms at the end. */
function drumsSound(rate: number): Float32Array {
  const r = random(10);
  const beat = 60 / 110;
  const d = raw(rate, 16 * beat, (d) => {
    // A hit's tail runs on past the loop's end into its start.
    const hit = (at: number, seconds: number, level: number, f: (t: number, noise: number) => number) => {
      const s = Math.round(at * rate);
      for (let j = 0, last = 0; j < seconds * rate; j++) {
        const noise = r() * 2 - 1;
        d[(s + j) % d.length] += level * (1 + 0.1 * (r() - 0.5)) * f(j / rate, noise - last);
        last = noise;
      }
    };
    // A pitched skin: a sine falling from `from` to `to` Hz as it dies away, with a little noise in its attack.
    const skin = (at: number, from: number, to: number, decay: number, level: number) => {
      let phase = 0;
      hit(at, decay * 5, level, (t, noise) => {
        phase += (2 * Math.PI * (to + (from - to) * Math.exp(-t / 0.03))) / rate;
        return Math.exp(-t / decay) * Math.sin(phase) + noise * 0.15 * Math.exp(-t / 0.004);
      });
    };
    const snare = (at: number, level: number) => {
      let phase = 0;
      hit(at, 0.4, level, (t, noise) => {
        phase += (2 * Math.PI * 190) / rate;
        return 0.5 * noise * Math.exp(-t / 0.08) + 0.6 * Math.sin(phase) * Math.exp(-t / 0.04);
      });
    };
    // Cymbals: high noise (each sample's change, so only the highs), short for the hi-hat, long for the crash.
    const cymbal = (at: number, decay: number, level: number) => hit(at, decay * 5, level, (t, noise) => noise * Math.exp(-t / decay));
    cymbal(0, 0.8, 0.35);
    for (let bar = 0; bar < 4; bar++) {
      const b = bar * 4 * beat;
      const fill = bar === 3;
      for (let e = 0; e < (fill ? 4 : 8); e++) cymbal(b + e * (beat / 2), 0.03, e % 2 ? 0.12 : 0.2);
      for (const k of fill ? [0] : [0, 2, 2.5]) skin(b + k * beat, 120, 48, 0.16, 0.9);
      for (const s of fill ? [1] : [1, 3]) snare(b + s * beat, 0.7);
    }
    // The fill: sixteenths down the toms.
    for (let n = 0; n < 8; n++) {
      const tom = n < 3 ? 210 : n < 6 ? 160 : 115;
      skin(14 * beat + n * (beat / 4), tom * 1.3, tom, 0.12, 0.5 + 0.1 * (n % 2));
    }
  });
  return nodeFilter(d, rate, 'lowpass', 7000, 0.7, true);
}

/** Soft, low noise: wind and a distant street. */
function bedSound(rate: number): Float32Array {
  const r = random(2);
  const d = raw(rate, 4, (d) => {
    let b = 0;
    for (let i = 0; i < d.length; i++) {
      b = (b + 0.02 * (r() * 2 - 1)) / 1.02;
      d[i] = b;
    }
    // Ease the loop's seam.
    const n = Math.min(2000, d.length >> 2);
    for (let i = 0; i < n; i++) d[d.length - n + i] = d[d.length - n + i] * (1 - i / n) + d[i] * (i / n);
  });
  return nodeFilter(d, rate, 'lowpass', 650, 0.5, true);
}

/** Two crickets, each chirping in short trills. */
function cricketSound(rate: number): Float32Array {
  const r = random(3);
  return raw(rate, 3, (d) => {
    for (const [freq, level, offset] of [
      [4600, 1, 0],
      [4250, 0.6, 0.21],
    ]) {
      for (let start = offset; start < 2.8; start += 0.42 + r() * 0.12) {
        for (let p = 0; p < 3; p++) {
          const s = Math.round((start + p * 0.032) * rate);
          const len = Math.round(0.014 * rate);
          for (let j = 0; j < len && s + j < d.length; j++) {
            const env = Math.sin((Math.PI * j) / len);
            d[s + j] += level * env * Math.sin((2 * Math.PI * freq * j) / rate);
          }
        }
      }
    }
  });
}

/** Where in a second of noise each of the knock's three clicks is read from. */
const CLICKS = [0.25, 0.5, 0.75];

/**
 * Three knocks on a wooden door: each a low tone falling from 220 to 95 Hz as it dies away, with a
 * click of noise (round 1400 Hz) on top; the third a touch late.
 */
function knockSound(rate: number): Float32Array {
  const r = random(1);
  const white = raw(rate, 1, (d) => {
    for (let i = 0; i < d.length; i++) d[i] = r() * 2 - 1;
  });
  const d = new Float32Array(Math.round(0.6 * rate));
  for (let n = 0; n < 3; n++) {
    const at = 0.02 + n * 0.17 + (n === 2 ? 0.02 : 0);
    const first = sampleAt(at, rate);
    // The tone: a sine sliding down, up to half level in 4 ms and away by 110 ms.
    const env = sweep([0.0001, 0.5, 0.0001], [0, 0.004, 0.11]);
    const hz = sweep([220, 95], [0, 0.08]);
    let phase = 0;
    for (let i = first; i < Math.min(d.length, sampleAt(at + 0.13, rate)); i++) {
      const t = i / rate - at;
      d[i] += env(t) * Math.sin(2 * Math.PI * phase);
      phase += hz(t) / rate;
    }
    // The click: a moment of the noise through a band, gone in 30 ms.
    const len = sampleAt(at + 0.04, rate) - first;
    const from = Math.round(CLICKS[n] * rate);
    const band = nodeFilter(white.subarray(from, from + len), rate, 'bandpass', 1400, 1.2, false);
    const fade = sweep([0.25, 0.0001], [0, 0.03]);
    for (let j = 0; j < len && first + j < d.length; j++) d[first + j] += band[j] * fade((first + j) / rate - at);
  }
  return d;
}

/** The first sample at or after `seconds`. */
export function sampleAt(seconds: number, rate: number): number {
  return Math.ceil(seconds * rate);
}

/** `seconds` of samples filled by `fill`, then brought to a peak of 1. */
function raw(rate: number, seconds: number, fill: (data: Float32Array, rate: number) => void): Float32Array {
  const data = new Float32Array(Math.round(rate * seconds));
  fill(data, rate);
  let peak = 0;
  for (let i = 0; i < data.length; i++) peak = Math.max(peak, Math.abs(data[i]));
  if (peak > 0) for (let i = 0; i < data.length; i++) data[i] /= peak;
  return data;
}

/**
 * A value moving through `values` at `times` (seconds, rising) the way an audio parameter does under
 * `exponentialRampToValueAtTime`: each step a constant ratio a second, held at the last value after it.
 */
export function sweep(values: readonly number[], times: readonly number[]): (t: number) => number {
  return (t) => {
    for (let k = 1; k < values.length; k++) {
      if (t < times[k]) return values[k - 1] * (values[k] / values[k - 1]) ** ((t - times[k - 1]) / (times[k] - times[k - 1]));
    }
    return values[values.length - 1];
  };
}

/**
 * `x` through a two-pole filter as a Web Audio `BiquadFilterNode` computes it (its `Q` in dB for a
 * low-pass and a high-pass, a plain quality factor for a band-pass). `loop`: `x` is a loop, and the
 * filter has run over its end before its start, so the result loops as smoothly as `x` did.
 */
export function nodeFilter(x: Float32Array, rate: number, type: 'lowpass' | 'highpass' | 'bandpass', frequency: number, q: number, loop: boolean): Float32Array {
  const w = (2 * Math.PI * Math.min(frequency, rate / 2)) / rate;
  const cos = Math.cos(w);
  const alpha = Math.sin(w) / (2 * (type === 'bandpass' ? q : 10 ** (q / 20)));
  const a0 = 1 + alpha;
  const edge = type === 'lowpass' ? (1 - cos) / 2 : (1 + cos) / 2;
  const b0 = (type === 'bandpass' ? alpha : edge) / a0;
  const b1 = (type === 'bandpass' ? 0 : type === 'lowpass' ? 2 * edge : -2 * edge) / a0;
  const b2 = (type === 'bandpass' ? -alpha : edge) / a0;
  const a1 = (-2 * cos) / a0;
  const a2 = (1 - alpha) / a0;
  const y = new Float32Array(x.length);
  let x1 = 0;
  let x2 = 0;
  let y1 = 0;
  let y2 = 0;
  for (let i = loop ? -Math.min(x.length, rate) : 0; i < x.length; i++) {
    const v = x[i < 0 ? x.length + i : i];
    const out = b0 * v + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
    x2 = x1;
    x1 = v;
    y2 = y1;
    y1 = out;
    if (i >= 0) y[i] = out;
  }
  return y;
}
