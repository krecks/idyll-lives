/**
 * The interface's little sounds, synthesised (no sound files): picking things up, putting them down,
 * money changing hands, walls going up. Each is a few pitched blips and puffs of noise (`BLIPS`), made
 * here as samples at a sample rate (no DOM or audio context), so `ui/sfx.ts` plays exactly what
 * `bun run sounds` renders and the tests measure.
 *
 * They were Web Audio graphs once (an `OscillatorNode` under a gain envelope; noise through a
 * low-pass): the waves here are the browser's own band-limited ones (`wave`), and what is made is what
 * the graphs made (compared against Chromium's rendering of them when they moved).
 */

import { nodeFilter, sampleAt, sweep } from './mono';
import { random } from './water';

export type Blip = 'pick' | 'place' | 'sell' | 'rotate' | 'error' | 'build' | 'remove' | 'paint' | 'upgrade' | 'tab' | 'open' | 'close';

type Wave = 'sine' | 'triangle' | 'square';

/** A pitched blip gliding from `from` to `to` Hz, with a fast attack and an exponential tail; `at` and `length` in seconds. */
interface Tone {
  wave: Wave;
  from: number;
  to: number;
  at: number;
  length: number;
  gain: number;
}

/** Noise under `cutoff` Hz dying away (dust, scrapes, the click of a knob). */
interface Hiss {
  cutoff: number;
  at: number;
  length: number;
  gain: number;
}

const tone = (wave: Wave, from: number, to: number, at: number, length: number, gain: number): Tone => ({ wave, from, to, at, length, gain });
const hiss = (cutoff: number, at: number, length: number, gain: number): Hiss => ({ cutoff, at, length, gain });

/** What each sound is made of. */
export const BLIPS: Record<Blip, readonly (Tone | Hiss)[]> = {
  pick: [tone('triangle', 660, 990, 0, 0.09, 0.16), tone('sine', 1320, 1320, 0.03, 0.06, 0.05)],
  // A soft thump, then a bright "ting-ting".
  place: [tone('sine', 170, 55, 0, 0.16, 0.45), hiss(900, 0, 0.06, 0.12), tone('sine', 1568, 1568, 0.06, 0.12, 0.07), tone('sine', 2093, 2093, 0.11, 0.18, 0.06)],
  sell: [tone('triangle', 880, 330, 0, 0.16, 0.14), tone('sine', 2637, 2637, 0.1, 0.16, 0.06), tone('sine', 3136, 3136, 0.15, 0.2, 0.05)],
  rotate: [tone('square', 1400, 1100, 0, 0.025, 0.035), hiss(3000, 0, 0.03, 0.05)],
  error: [tone('square', 220, 210, 0, 0.08, 0.05), tone('square', 175, 165, 0.09, 0.12, 0.05)],
  // Wood knock.
  build: [tone('sine', 240, 110, 0, 0.09, 0.4), hiss(500, 0, 0.08, 0.2)],
  remove: [hiss(280, 0, 0.28, 0.3), tone('sine', 130, 50, 0, 0.18, 0.3)],
  // A brush stroke: a soft swish, rising.
  paint: [hiss(1800, 0, 0.16, 0.12), tone('sine', 520, 780, 0.04, 0.12, 0.05)],
  upgrade: [1047, 1319, 1568, 2093].map((f, i) => tone('triangle', f, f, i * 0.065, 0.14, 0.09)),
  tab: [tone('sine', 1800, 1500, 0, 0.03, 0.04)],
  open: [tone('triangle', 523, 523, 0, 0.1, 0.08), tone('triangle', 784, 784, 0.07, 0.14, 0.08)],
  close: [tone('triangle', 784, 784, 0, 0.1, 0.07), tone('triangle', 523, 523, 0.07, 0.14, 0.07)],
};

/** Seconds a part sounds on after its length (its envelope held at nothing). */
const TAIL = 0.02;
/** Seconds of noise the puffs read from, the same every time. */
const NOISE = 0.5;

const noises = new Map<number, Float32Array>();

function noise(rate: number): Float32Array {
  let d = noises.get(rate);
  if (!d) {
    const r = random(0x5f);
    d = new Float32Array(Math.round(rate * NOISE));
    for (let i = 0; i < d.length; i++) d[i] = r() * 2 - 1;
    noises.set(rate, d);
  }
  return d;
}

/** A sound of the interface, as samples (one channel). */
export function blipSamples(sound: Blip, rate: number): Float32Array {
  const parts = BLIPS[sound];
  const d = new Float32Array(sampleAt(Math.max(...parts.map((p) => p.at + p.length + TAIL)), rate) + 1);
  for (const p of parts) {
    const first = sampleAt(p.at, rate);
    const last = Math.min(d.length, sampleAt(p.at + p.length + TAIL, rate));
    if ('cutoff' in p) {
      // (The node's own resonance: 1 dB.)
      const low = nodeFilter(noise(rate).subarray(0, last - first), rate, 'lowpass', p.cutoff, 1, false);
      const env = sweep([p.gain, 0.0001], [0, p.length]);
      for (let i = first; i < last; i++) d[i] += low[i - first] * env(i / rate - p.at);
      continue;
    }
    const env = sweep([0.0001, p.gain, 0.0001], [0, 0.006, p.length]);
    const hz = p.from === p.to ? () => p.from : sweep([p.from, p.to], [0, p.length]);
    let phase = 0;
    for (let i = first; i < last; i++) {
      const t = i / rate - p.at;
      const f = hz(t);
      d[i] += env(t) * wave(p.wave, phase, f, rate);
      phase += f / rate;
    }
  }
  return d;
}

/** The browser's wave tables hold a period in this many samples (so half as many overtones at most), at the sample rates of sound cards. */
const TABLE = 4096;
/** The tables thin out their overtones in steps of a third of an octave. */
const STEPS = 3;
/**
 * The square's level: the browser brings each wave's fullest table to a peak of 1, and a square built
 * from overtones overshoots its corners by 18 % (Gibbs), so its flat parts sit at 0.85.
 */
const SQUARE_LEVEL = 0.8482;

/**
 * A wave at `phase` (in periods) played at `hz`, as a Web Audio `OscillatorNode` makes it: from its
 * overtones, only those under half the sample rate (the last third of an octave of them fading out as
 * the pitch rises), so a high square does not fold back into the band as false tones.
 */
function wave(type: Wave, phase: number, hz: number, rate: number): number {
  const angle = 2 * Math.PI * (phase - Math.floor(phase));
  if (type === 'sine') return Math.sin(angle);
  // The two tables the pitch lies between: how many overtones each holds, and how far from the fuller to the thinner.
  const lowest = rate / TABLE;
  const range = Math.max(0, 1 + STEPS * Math.log2(Math.max(hz, lowest / 2) / lowest));
  const fuller = Math.floor((TABLE / 2) * 2 ** (-Math.floor(range) / STEPS));
  const thinner = Math.floor((TABLE / 2) * 2 ** (-(Math.floor(range) + 1) / STEPS));
  const fade = range - Math.floor(range);
  let sum = 0;
  for (let n = 1; n <= fuller; n += 2) {
    const share = n <= thinner ? 1 : 1 - fade;
    const k = 2 / (n * Math.PI);
    sum += share * (type === 'square' ? 2 * k : 2 * k * k * (((n - 1) >> 1) & 1 ? -1 : 1)) * Math.sin(n * angle);
  }
  return type === 'square' ? sum * SQUARE_LEVEL : sum;
}
