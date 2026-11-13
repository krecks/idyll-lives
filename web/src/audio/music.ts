/**
 * Music for the stereo, the radio and dancing, made in code (no sound files): a few pieces half a minute
 * long, each written by a small composer from a seed (`score`: the same seed, the same piece) and played
 * on a handful of synthesised voices (`perform`), as a loop whose last notes ring on into its start.
 *
 * Three plain styles:
 * - `folk`: a plucked tune over a walking bass and an off-beat strum, 100 beats a minute, in G;
 * - `dance`: a kick on every beat, hats between, a clap on two and four, a pumping bass and a plucked
 *   arpeggio over held chords, 120 beats a minute, in A minor;
 * - `slow`: a waltz, soft held chords and a tune of long plucked notes, 72 beats a minute, in F.
 *
 * A piece is sixteen bars in four phrases, the first tune coming back (A A B A) and ending on the key's own
 * note, over a chord a bar from a few well-worn progressions; the tune moves by step along a five-note scale
 * and lands on a note of the chord on the strong beats. No DOM or audio context here; the pieces take a
 * moment to make and are made off the main thread (`made.ts`).
 */

import { nodeFilter } from './mono';
import { gains, random, setLevel, type Stereo } from './water';

export type Style = 'folk' | 'dance' | 'slow';

export type Voice = 'pluck' | 'strum' | 'bass' | 'pad' | 'kick' | 'clap' | 'hat';

/** A note of a piece: when (beats from its start) and how long (beats), its pitch (MIDI: 69 is the A at 440 Hz), which voice plays it, how loud (0..1) and where (-1 left .. 1 right). */
export interface Note {
  at: number;
  len: number;
  midi: number;
  voice: Voice;
  level: number;
  pan: number;
}

/** A piece as written: its style, pace, length, key and notes. */
export interface Score {
  style: Style;
  /** Beats a minute, beats a bar, bars. */
  bpm: number;
  meter: number;
  bars: number;
  /** The key's note (MIDI) and whether it is minor. */
  key: number;
  minor: boolean;
  /** The chord of each bar: its root, as a step of the key's scale (0: the key's own chord). */
  chords: number[];
  notes: Note[];
}

/** What a style is made of. */
const STYLES: Record<Style, { bpm: number; meter: number; key: number; minor: boolean; progressions: number[][]; bridge: number[][] }> = {
  // (Chords by scale step: 0 the key's, 3 the fourth, 4 the fifth, 5 the sixth.)
  folk: { bpm: 100, meter: 4, key: 55, minor: false, progressions: [[0, 3, 0, 4], [0, 5, 3, 4], [0, 3, 4, 0]], bridge: [[3, 0, 3, 4], [5, 3, 0, 4]] },
  dance: { bpm: 120, meter: 4, key: 57, minor: true, progressions: [[0, 5, 2, 6], [0, 3, 5, 6], [0, 5, 3, 4]], bridge: [[5, 6, 0, 0], [3, 5, 6, 6]] },
  slow: { bpm: 72, meter: 3, key: 53, minor: false, progressions: [[0, 5, 3, 4], [0, 3, 0, 4], [0, 4, 5, 3]], bridge: [[3, 4, 0, 5], [5, 3, 4, 4]] },
};

const MAJOR = [0, 2, 4, 5, 7, 9, 11];
const MINOR = [0, 2, 3, 5, 7, 8, 10];
/** The five notes a tune keeps to (steps of the seven-note scale): no half steps, so nothing it plays grates. */
const FIVE = [0, 1, 2, 4, 5];
const FIVE_MINOR = [0, 2, 3, 4, 6];

/** The pitch (semitones over the key's note) of scale step `step` (any octave: 7 is the key's note an octave up). */
function pitch(step: number, minor: boolean): number {
  const scale = minor ? MINOR : MAJOR;
  const octave = Math.floor(step / 7);
  return octave * 12 + scale[step - octave * 7];
}

/** The three notes of the chord on scale step `root`, as scale steps. */
const triad = (root: number) => [root, root + 2, root + 4];

/** A piece in a style from a seed: the same seed, the same piece. */
export function score(style: Style, seed: number): Score {
  const s = STYLES[style];
  const r = random(0x5c0e + seed * 977 + Object.keys(STYLES).indexOf(style) * 131);
  const pick = <T>(list: readonly T[]): T => list[Math.floor(r() * list.length)];
  const main = pick(s.progressions);
  const chords = [...main, ...main, ...pick(s.bridge), ...main];
  const bars = chords.length;
  const notes: Note[] = [];
  const five = s.minor ? FIVE_MINOR : FIVE;
  // The tune's notes to choose from: the five-note scale over two octaves, as scale steps.
  const ladder = [0, 1, 2].flatMap((o) => five.map((d) => d + 7 * o)).filter((d) => d <= 16);
  const nearest = (to: number, among: number[]) => among.reduce((best, d) => (Math.abs(d - to) < Math.abs(best - to) ? d : best));
  const chordRungs = (root: number) => ladder.filter((d) => triad(root).some((c) => (d - c) % 7 === 0));

  /** One phrase's tune (four bars): where each note falls in its bar (in `grid`s of a beat) and its rung of the ladder. */
  const slots = s.meter * (style === 'slow' ? 1 : 2);
  const grid = style === 'slow' ? 1 : 0.5;
  const phrase = (bar0: number, from: number): { bar: number; slot: number; len: number; rung: number }[] => {
    const out: { bar: number; slot: number; len: number; rung: number }[] = [];
    let at = nearest(from, ladder);
    for (let b = 0; b < 4; b++) {
      const rungs = chordRungs(chords[bar0 + b]);
      // A rhythm for the bar: which slots start a note (the first always; a slow tune holds its notes).
      const starts = [0];
      for (let k = 1; k < slots; k++) if (r() < (style === 'slow' ? 0.45 : k % 2 === 0 ? 0.75 : 0.4)) starts.push(k);
      for (const [i, slot] of starts.entries()) {
        const strong = slot === 0 || (style !== 'slow' && slot === slots / 2);
        // On a strong beat a note of the chord near where the tune is; between, a step up or down, drawn back to the middle.
        const index = ladder.indexOf(at);
        const pull = at > 9 ? -1 : at < 5 ? 1 : r() < 0.5 ? 1 : -1;
        at = strong ? nearest(at + (r() < 0.3 ? pull * 2 : 0), rungs) : ladder[Math.max(0, Math.min(ladder.length - 1, index + (r() < 0.75 ? pull : -pull)))];
        out.push({ bar: b, slot, len: ((starts[i + 1] ?? slots) - slot) * grid, rung: at });
      }
    }
    return out;
  };
  const a = phrase(0, 7);
  const b = phrase(8, a[a.length - 1].rung + 2);
  // The first phrase three times (bars 1-4, 5-8, 13-16), the bridge between; the last note of the piece is the key's own.
  const tune: { bar: number; slot: number; len: number; rung: number }[] = [];
  for (const [n, part] of [a, a, b, a].entries()) for (const t of part) tune.push({ ...t, bar: t.bar + n * 4 });
  const last = tune[tune.length - 1];
  tune[tune.length - 1] = { ...last, rung: nearest(last.rung, [0, 7, 14]) };
  for (const t of tune) notes.push({ at: t.bar * s.meter + t.slot * grid, len: t.len, midi: s.key + 12 + pitch(t.rung, s.minor), voice: 'pluck', level: 0.5 + (t.slot === 0 ? 0.1 : 0), pan: 0.15 });

  // Under the tune, bar by bar.
  for (let bar = 0; bar < bars; bar++) {
    const at = bar * s.meter;
    const root = chords[bar];
    const low = s.key - 12 + pitch(root, s.minor) - (pitch(root, s.minor) > 7 ? 12 : 0);
    const chord = triad(root).map((d) => s.key + pitch(d, s.minor));
    if (style === 'folk') {
      // The bass on one and three (the chord's note, then its fifth), a strum of the chord on two and four.
      notes.push({ at, len: 2, midi: low, voice: 'bass', level: 0.6, pan: -0.1 });
      notes.push({ at: at + 2, len: 2, midi: low + 7, voice: 'bass', level: 0.5, pan: -0.1 });
      for (const beat of [1, 3]) chord.forEach((midi, i) => notes.push({ at: at + beat + i * 0.04, len: 0.5, midi, voice: 'strum', level: 0.2, pan: -0.35 }));
    } else if (style === 'dance') {
      // A kick on every beat, hats between, a clap on two and four; the bass on the off-beats; the chord held, and run up and down in sixteenths.
      for (let beat = 0; beat < 4; beat++) {
        notes.push({ at: at + beat, len: 0.5, midi: 36, voice: 'kick', level: 0.9, pan: 0 });
        notes.push({ at: at + beat + 0.5, len: 0.25, midi: 90, voice: 'hat', level: 0.25, pan: 0.3 });
        notes.push({ at: at + beat + 0.5, len: 0.5, midi: low + (beat === 3 ? 12 : 0), voice: 'bass', level: 0.55, pan: 0 });
        if (beat % 2) notes.push({ at: at + beat, len: 0.5, midi: 60, voice: 'clap', level: 0.4, pan: -0.1 });
      }
      chord.forEach((midi) => notes.push({ at, len: 4, midi, voice: 'pad', level: 0.16, pan: 0 }));
      const run = [0, 1, 2, 1];
      for (let k = 0; k < 16; k++) notes.push({ at: at + k * 0.25, len: 0.25, midi: chord[run[k % 4]] + 12, voice: 'strum', level: 0.16 + (k % 4 === 0 ? 0.05 : 0), pan: k % 2 ? -0.4 : -0.2 });
    } else {
      // A waltz: the bass on one, the chord (soft, held) under the bar, two light plucks of it on two and three.
      notes.push({ at, len: 3, midi: low, voice: 'bass', level: 0.5, pan: -0.1 });
      chord.forEach((midi) => notes.push({ at, len: 3, midi, voice: 'pad', level: 0.2, pan: 0 }));
      for (const beat of [1, 2]) notes.push({ at: at + beat, len: 1, midi: chord[beat], voice: 'strum', level: 0.18, pan: -0.3 });
    }
  }
  notes.sort((x, y) => x.at - y.at);
  return { style, bpm: s.bpm, meter: s.meter, bars, key: s.key, minor: s.minor, chords, notes };
}

/** How long a piece lasts (s). */
export function seconds(piece: Score): number {
  return (piece.bars * piece.meter * 60) / piece.bpm;
}

const hz = (midi: number) => 440 * 2 ** ((midi - 69) / 12);

/**
 * A piece played, as a loop of two channels at a sample rate: every note on its voice at its place, a note's
 * tail past the end ringing on into the start, then the whole through the cloth of a loudspeaker (a low-pass)
 * and brought to an even level.
 */
export function perform(piece: Score, rate: number): Stereo {
  const n = Math.round(seconds(piece) * rate);
  const out: Stereo = { left: new Float32Array(n), right: new Float32Array(n) };
  const beat = 60 / piece.bpm;
  const r = random(0xbea7 + piece.notes.length);
  for (const note of piece.notes) {
    const [l, rr] = gains(note.pan);
    const start = Math.round(note.at * beat * rate);
    const held = note.len * beat;
    const f = hz(note.midi);
    /** Adds the voice's sound, `sample(t)` for `secs`, round the loop's end. */
    const add = (secs: number, sample: (t: number) => number) => {
      const count = Math.round(secs * rate);
      for (let j = 0; j < count; j++) {
        const v = note.level * sample(j / rate);
        const i = (start + j) % n;
        out.left[i] += v * l;
        out.right[i] += v * rr;
      }
    };
    switch (note.voice) {
      case 'pluck':
      case 'strum': {
        // A plucked string: its tone and the octave over it, dying away (sooner for a strum's).
        const decay = note.voice === 'pluck' ? 0.16 + 0.12 * Math.min(2, held) : 0.14;
        add(decay * 5, (t) => Math.exp(-t / decay) * Math.min(1, t / 0.002) * (Math.sin(2 * Math.PI * f * t) + 0.3 * Math.sin(4 * Math.PI * f * t) + 0.08 * Math.sin(6 * Math.PI * f * t)));
        break;
      }
      case 'bass':
        add(Math.min(2.5, held + 0.6), (t) => Math.exp(-t / 0.5) * Math.min(1, t / 0.004) * Math.min(1, (held + 0.6 - t) / 0.1) * (Math.sin(2 * Math.PI * f * t) + 0.3 * Math.sin(4 * Math.PI * f * t)));
        break;
      case 'pad':
        // A soft held tone: three voices a hair apart, in over a third of a second and out again.
        add(held + 0.5, (t) => Math.min(1, t / 0.35) * Math.min(1, Math.max(0, held + 0.5 - t) / 0.5) * (Math.sin(2 * Math.PI * f * t) + Math.sin(2 * Math.PI * f * 1.004 * t) + Math.sin(2 * Math.PI * f * 0.996 * t) + 0.25 * Math.sin(4 * Math.PI * f * t)) * 0.33);
        break;
      case 'kick': {
        let phase = 0;
        add(0.5, (t) => {
          phase += (2 * Math.PI * (48 + 72 * Math.exp(-t / 0.03))) / rate;
          return Math.exp(-t / 0.14) * Math.sin(phase);
        });
        break;
      }
      case 'clap': {
        let lastNoise = 0;
        add(0.3, (t) => {
          const w = r() * 2 - 1;
          const high = w - lastNoise;
          lastNoise = w;
          return 0.5 * high * Math.exp(-t / 0.06) + 0.3 * Math.sin(2 * Math.PI * 190 * t) * Math.exp(-t / 0.03);
        });
        break;
      }
      case 'hat': {
        let lastNoise = 0;
        add(0.15, (t) => {
          const w = r() * 2 - 1;
          const high = w - lastNoise;
          lastNoise = w;
          return high * Math.exp(-t / 0.025);
        });
        break;
      }
    }
  }
  // Through a loudspeaker's cloth: the highs softened (a dance's less, for its hats), run round the loop.
  const cutoff = piece.style === 'dance' ? 5200 : 3200;
  const played = { left: nodeFilter(out.left, rate, 'lowpass', cutoff, -3.01, true), right: nodeFilter(out.right, rate, 'lowpass', cutoff, -3.01, true) };
  return setLevel(played, 0.12);
}

/** The pieces the game plays, by name: each its style and seed. */
export const PIECES = {
  musicFolk: { style: 'folk', seed: 3 },
  musicDance: { style: 'dance', seed: 5 },
  musicSlow: { style: 'slow', seed: 2 },
} as const satisfies Record<string, { style: Style; seed: number }>;

export type MusicKind = keyof typeof PIECES;

/** The pieces as sounds made by kind (`made.ts`). */
export const MUSIC = Object.fromEntries((Object.keys(PIECES) as MusicKind[]).map((kind) => [kind, (rate: number) => perform(score(PIECES[kind].style, PIECES[kind].seed), rate)])) as Record<MusicKind, (rate: number) => Stereo>;
