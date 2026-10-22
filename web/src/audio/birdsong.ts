/**
 * The neighbourhood birds' songs (`world.ts` decides when they sing): the species, and how each
 * version of a bird's song is made. No DOM or audio context here: the songs are made off the main
 * thread, by the sound worker (`water.worker.ts`).
 */

/** One note of a song. */
interface Note {
  /** Seconds, and seconds of silence after it. */
  len: number;
  gap: number;
  /** Pitch through the note in multiples of the bird's own pitch, spread evenly over the note and joined in a smooth curve. */
  pitch: number[];
  level?: number;
}

export interface Species {
  name: string;
  /** Range of an individual's own pitch (Hz). */
  pitch: [number, number];
  /** A fast warble on every note: rate (Hz) and depth (part of the pitch); the level dips with it by `flutter`. */
  vibrato: [number, number];
  flutter: number;
  /** Level of the 2nd harmonic (a third of it for the 3rd). */
  overtone: number;
  /** Seconds to fade each note in and out (less on short notes). */
  fade: number;
  /** Highest frequencies kept (Hz), before distance takes more off. */
  bright: number;
  level: number;
  /** Songs in a bout, seconds between them and seconds of rest after the bout, when birds are busiest. */
  bout: [number, number];
  pause: [number, number];
  rest: [number, number];
  /** Whether it takes turns with the other birds (waits for a quiet moment, sometimes answers one). */
  turns: boolean;
  /** One song; `r` varies it a little, so each bird has a few versions of its own. */
  song: (r: () => number) => Note[];
}

export const SPECIES: Species[] = [
  {
    // "Tee-cher tee-cher tee-cher", like a great tit.
    name: 'tit',
    pitch: [3700, 4300],
    vibrato: [34, 0.004],
    flutter: 0.06,
    overtone: 0.05,
    fade: 0.018,
    bright: 7000,
    level: 0.8,
    bout: [3, 6],
    pause: [2, 4],
    rest: [25, 60],
    turns: true,
    song: (r) => {
      const notes: Note[] = [];
      const pairs = 3 + Math.floor(r() * 3);
      for (let i = 0; i < pairs; i++) {
        const level = i === pairs - 1 ? 0.85 : 1;
        notes.push({ len: 0.11, gap: 0.04, pitch: [1.04, 1.01, 0.98], level });
        notes.push({ len: 0.13, gap: 0.09, pitch: [0.8, 0.74, 0.7, 0.68], level: 0.8 * level });
      }
      return notes;
    },
  },
  {
    // A run of quick notes falling in steps, then a flourish, like a chaffinch.
    name: 'finch',
    pitch: [3300, 3800],
    vibrato: [26, 0.006],
    flutter: 0.1,
    overtone: 0.09,
    fade: 0.008,
    bright: 8000,
    level: 0.75,
    bout: [4, 8],
    pause: [5, 9],
    rest: [30, 80],
    turns: true,
    song: (r) => {
      const notes: Note[] = [];
      const steps = [
        { count: 5 + Math.floor(r() * 3), pitch: 1.3, len: 0.042, gap: 0.032 },
        { count: 4 + Math.floor(r() * 2), pitch: 1.1, len: 0.05, gap: 0.03 },
        { count: 3 + Math.floor(r() * 2), pitch: 0.94, len: 0.06, gap: 0.032 },
      ];
      for (const [s, step] of steps.entries()) {
        for (let i = 0; i < step.count; i++) {
          const p = step.pitch * (1 - 0.01 * i);
          notes.push({ len: step.len, gap: step.gap, pitch: [p * 1.12, p, p * 0.9], level: 0.65 + 0.12 * s });
        }
      }
      notes.push({ len: 0.09, gap: 0.03, pitch: [1, 0.86, 0.8], level: 0.9 });
      notes.push({ len: 0.26, gap: 0, pitch: [0.9, 1.2, 1.3, 1.1, 0.78], level: 1 });
      return notes;
    },
  },
  {
    // A soft cascade of liquid notes running down the scale, like a willow warbler.
    name: 'warbler',
    pitch: [2900, 3300],
    vibrato: [21, 0.02],
    flutter: 0.3,
    overtone: 0.12,
    fade: 0.014,
    bright: 6500,
    level: 0.75,
    bout: [3, 6],
    pause: [6, 12],
    rest: [30, 80],
    turns: true,
    song: (r) => {
      const count = 11 + Math.floor(r() * 4);
      return Array.from({ length: count }, (_, i) => {
        const x = i / (count - 1);
        const p = 1.5 - 0.6 * x;
        const last = i >= count - 2;
        const level = Math.min(1, 0.45 + 1.2 * x) * (last ? 0.9 : 1);
        return last ? { len: 0.12, gap: 0.04, pitch: [p, p * 1.08, p * 0.92], level } : { len: 0.075, gap: 0.035, pitch: [p * 0.97, p * 1.04, p * 0.98], level };
      });
    },
  },
  {
    // A blackbird: an unhurried, fluting phrase of mellow slurred whistles, never quite the same
    // twice, trailing off into a soft, high twitter.
    name: 'blackbird',
    pitch: [1900, 2300],
    vibrato: [9, 0.008],
    flutter: 0.08,
    overtone: 0.05,
    fade: 0.03,
    bright: 5000,
    level: 0.9,
    bout: [4, 9],
    pause: [2.5, 5],
    rest: [30, 70],
    turns: true,
    song: (r) => {
      // Slurs it puts its phrases together from: rising, falling, a dip, a turn.
      const slurs = [
        [1, 1.12, 1.06],
        [0.84, 0.94, 1],
        [1.22, 1.1, 0.9],
        [0.9, 1.24, 1.2, 1.02],
        [1.02, 0.8, 0.82],
        [1.34, 1.3, 1.18],
        [0.95, 1.06, 0.98, 1.1],
        [0.76, 0.86, 0.8],
      ];
      const notes: Note[] = [];
      const count = 3 + Math.floor(r() * 4);
      for (let i = 0; i < count; i++) {
        const slur = slurs[Math.floor(r() * slurs.length)];
        const key = 0.9 + r() * 0.25;
        const last = i === count - 1;
        notes.push({ len: 0.14 + r() * 0.18, gap: last ? 0.05 : 0.03 + r() * 0.07, pitch: slur.map((p) => p * key), level: 0.75 + r() * 0.25 });
      }
      // The twitter: a few quick, thin high notes, quieter.
      const twitter = Math.floor(r() * 6);
      for (let i = 0; i < twitter; i++) {
        const p = 1.8 + r() * 0.5;
        notes.push({ len: 0.03 + r() * 0.02, gap: 0.018, pitch: [p * 1.08, p, p * 1.12], level: 0.3 + r() * 0.12 });
      }
      return notes;
    },
  },
  {
    // "Coo-COOO-coo", low and soft, like a collared dove; now and then.
    name: 'dove',
    pitch: [470, 560],
    vibrato: [7, 0.005],
    flutter: 0.15,
    overtone: 0.15,
    fade: 0.06,
    bright: 1600,
    level: 1.6,
    bout: [3, 6],
    pause: [1.4, 2.2],
    rest: [90, 200],
    turns: false,
    song: () => [
      { len: 0.2, gap: 0.08, pitch: [0.96, 1, 0.97], level: 0.8 },
      { len: 0.55, gap: 0.12, pitch: [0.98, 1.06, 1.06, 1.02, 0.94], level: 1 },
      { len: 0.3, gap: 0, pitch: [0.95, 0.97, 0.88], level: 0.75 },
    ],
  },
];

/** What makes one bird's song its own (sent to the worker that makes it). */
export interface Voice {
  /** Its species' `name`. */
  species: string;
  seed: number;
  /** Its own pitch (Hz), pace (1 = as written) and nearness (1 = nearest). */
  pitch: number;
  pace: number;
  near: number;
}

/** Versions of its song each bird has. */
export const VERSIONS = 3;

/**
 * One version of a bird's song at a sample rate: each note a sine following its pitch curve with a
 * fast warble and a quiet overtone, faded in and out, then softened and given a faint echo as if
 * from a garden away.
 */
export function songSamples(bird: Voice, version: number, rate: number): Float32Array {
  const sp = SPECIES.find((s) => s.name === bird.species) ?? SPECIES[0];
  const r = random(Math.imul(bird.seed * VERSIONS + version + 1, 0x9e3779b1));
  const notes = sp.song(r);
  const shift = 1 + (r() - 0.5) * 0.03;
  let seconds = 0.3;
  for (const n of notes) seconds += (n.len * 1.04 + n.gap) * bird.pace;
  const d = new Float32Array(Math.ceil(seconds * rate));
  const [wobble, depth] = sp.vibrato;
  let at = 0;
  for (const n of notes) {
    const len = n.len * bird.pace * (1 + (r() - 0.5) * 0.08);
    const count = Math.round(len * rate);
    const fade = Math.min(sp.fade, len / 3);
    const tune = bird.pitch * shift * (1 + (r() - 0.5) * 0.02);
    const speed = (2 * Math.PI * wobble * (0.9 + r() * 0.2)) / rate;
    const start = Math.round(at * rate);
    let phase = 0;
    let vib = r() * 2 * Math.PI;
    for (let j = 0; j < count && start + j < d.length; j++) {
      const time = j / rate;
      const w = Math.sin(vib);
      phase += (2 * Math.PI * tune * curve(n.pitch, j / count) * (1 + depth * w)) / rate;
      vib += speed;
      const edge = Math.min(1, time / fade, (len - time) / fade);
      const amp = (0.5 - 0.5 * Math.cos(Math.PI * edge)) * (n.level ?? 1) * (1 - sp.flutter * (0.5 + 0.5 * w));
      d[start + j] += amp * (Math.sin(phase) + sp.overtone * (Math.sin(2 * phase) + Math.sin(3 * phase) / 3));
    }
    at += len + n.gap * bird.pace;
  }
  distance(d, rate, sp.bright * (0.7 + 0.3 * bird.near), 1 - bird.near);
  return d;
}

/** A smooth curve through evenly spaced points, at `x` from 0 to 1. */
function curve(points: number[], x: number): number {
  if (points.length === 1) return points[0];
  const u = x * (points.length - 1);
  const i = Math.min(points.length - 2, Math.floor(u));
  const f = u - i;
  const p0 = points[Math.max(0, i - 1)];
  const p1 = points[i];
  const p2 = points[i + 1];
  const p3 = points[Math.min(points.length - 1, i + 2)];
  return p1 + 0.5 * f * (p2 - p0 + f * (2 * p0 - 5 * p1 + 4 * p2 - p3 + f * (3 * (p1 - p2) + p3 - p0)));
}

/** Softens a sound as if from across the gardens: highs rolled off, and faint echoes off nearby walls. */
function distance(d: Float32Array, rate: number, cutoff: number, far: number): void {
  const a = 1 - Math.exp((-2 * Math.PI * cutoff) / rate);
  let lp = 0;
  for (let i = 0; i < d.length; i++) d[i] = lp += (d[i] - lp) * a;
  const taps = [0.023, 0.037, 0.053, 0.071, 0.097, 0.127, 0.163, 0.211].map((delay, i) => [Math.round(delay * rate), (i % 2 ? -0.07 : 0.08) * Math.exp(-delay / 0.09) * (0.6 + far)]);
  // From the end back, so echoes don't echo again.
  for (let i = d.length - 1; i >= 0; i--) for (const [k, level] of taps) if (i >= k) d[i] += d[i - k] * level;
}

/** A small seeded generator, so the sounds are the same every time. */
function random(seed: number): () => number {
  let s = seed >>> 0 || 1;
  return () => {
    s ^= s << 13;
    s ^= s >>> 17;
    s ^= s << 5;
    return (s >>> 0) / 4294967296;
  };
}
