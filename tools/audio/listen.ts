#!/usr/bin/env bun
/**
 * Renders every synthesised sound of the world to a WAV, with the numbers that can be measured and
 * the ones only a listener can judge (`G-53`, `docs/authenticity-review.md`): the sounds are made
 * in code (`web/src/audio/water.ts`, `things.ts`), so they can be made here, off the browser, and
 * listened to on any player. `bun run sounds` (from `web/`), or `bun tools/audio/listen.ts`.
 *
 * Each is written to `bench-results/sounds/<kind>.wav` (16-bit stereo) with a line in `index.md`:
 * its length, its level (peak and RMS, dBFS; its loudness to the ear, LUFS), where its energy sits (the spectral centroid, Hz) and
 * whether it loops or is played once; then the objects and uses of the content that sound it, so a
 * listener knows what to expect before pressing play.
 *
 * Each is written at the level the game plays it (`world.ts`'s `LOOPS` and `ONCE_LEVEL`, read the way
 * `web/tests/sounds.test.ts` reads them), so one sound against another sounds as it does in the game.
 *
 * What a machine can say is here; whether a microwave sounds like a microwave is the ear's part.
 */

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { MADE, ONCE_THINGS, type MadeKind } from '../../web/src/audio/things';
import { ONCE, type WaterKind } from '../../web/src/audio/water';

const RATE = 48_000;
/** The checkout. */
const ROOT = resolve(import.meta.dir, '../..');
const OUT = join(ROOT, 'bench-results/sounds');

/** The kinds played once rather than looped (a use starting or ending, a flush). */
const ONCE_KINDS = new Set<string>([...ONCE, ...ONCE_THINGS]);

/** A 16-bit stereo WAV of two channels of samples. */
function wav(left: Float32Array, right: Float32Array): Buffer {
  const n = Math.min(left.length, right.length);
  const data = Buffer.alloc(n * 4);
  for (let i = 0; i < n; i++) {
    // Clamped, and rounded away from the sample's own quantisation noise.
    data.writeInt16LE(Math.max(-32768, Math.min(32767, Math.round(left[i] * 32767))), i * 4);
    data.writeInt16LE(Math.max(-32768, Math.min(32767, Math.round(right[i] * 32767))), i * 4 + 2);
  }
  const head = Buffer.alloc(44);
  head.write('RIFF', 0);
  head.writeUInt32LE(36 + data.length, 4);
  head.write('WAVEfmt ', 8);
  head.writeUInt32LE(16, 16);
  head.writeUInt16LE(1, 20); // PCM
  head.writeUInt16LE(2, 22); // stereo
  head.writeUInt32LE(RATE, 24);
  head.writeUInt32LE(RATE * 4, 28); // bytes a second
  head.writeUInt16LE(4, 32); // bytes a frame
  head.writeUInt16LE(16, 34); // bits a sample
  head.write('data', 36);
  head.writeUInt32LE(data.length, 40);
  return Buffer.concat([head, data]);
}

const db = (x: number) => (x > 0 ? 20 * Math.log10(x) : -Infinity);
const num = (x: number, digits = 1) => (Number.isFinite(x) ? x.toFixed(digits) : 'silent');

/** Peak and RMS over both channels, in dBFS. */
function level_(l: Float32Array, r: Float32Array): { peak: number; rms: number } {
  let peak = 0;
  let sum = 0;
  const n = Math.min(l.length, r.length);
  for (let i = 0; i < n; i++) {
    peak = Math.max(peak, Math.abs(l[i]), Math.abs(r[i]));
    sum += l[i] * l[i] + r[i] * r[i];
  }
  return { peak: db(peak), rms: db(Math.sqrt(sum / Math.max(1, n * 2))) };
}

/** `x` through a biquad (`b` over 1, `a[0]`, `a[1]`), in double precision. */
function biquad(x: Float64Array, b: readonly [number, number, number], a: readonly [number, number]): Float64Array {
  const y = new Float64Array(x.length);
  let x1 = 0;
  let x2 = 0;
  let y1 = 0;
  let y2 = 0;
  for (let i = 0; i < x.length; i++) {
    const v = b[0] * x[i] + b[1] * x1 + b[2] * x2 - a[0] * y1 - a[1] * y2;
    x2 = x1;
    x1 = x[i];
    y2 = y1;
    y1 = v;
    y[i] = v;
  }
  return y;
}

/**
 * How loud the sound is to the ear (LUFS, ITU-R BS.1770-4): both channels through the K weighting (a shelf that
 * lifts what the head lifts, then a high-pass: the standard's coefficients for 48 kHz, which `RATE` is), the mean
 * square of 0.4 s blocks a tenth of a second apart, the blocks under -70 LUFS and then those more than 10 LU under
 * the rest's mean left out (a sound's own silences don't make it quieter). A sound shorter than a block is one
 * block. Peak and RMS say what the samples hold; this says what one sound is against another when heard, which
 * is what a mix is levelled by.
 */
function loudness(l: Float32Array, r: Float32Array): number {
  const weighted = (x: Float32Array) =>
    biquad(biquad(Float64Array.from(x), [1.53512485958697, -2.69169618940638, 1.19839281085285], [-1.69065929318241, 0.73248077421585]), [1, -2, 1], [-1.99004745483398, 0.99007225036621]);
  const a = weighted(l);
  const b = weighted(r);
  const n = Math.min(a.length, b.length);
  const block = Math.min(n, Math.round(0.4 * RATE));
  const step = Math.round(0.1 * RATE);
  if (block === 0) return -Infinity;
  const powers: number[] = [];
  for (let start = 0; start + block <= n; start += step) {
    let sum = 0;
    for (let i = start; i < start + block; i++) sum += a[i] * a[i] + b[i] * b[i];
    powers.push(sum / block);
  }
  const lufs = (power: number) => (power > 0 ? -0.691 + 10 * Math.log10(power) : -Infinity);
  const mean = (list: number[]) => list.reduce((s, v) => s + v, 0) / Math.max(1, list.length);
  const heard = powers.filter((p) => lufs(p) > -70);
  if (!heard.length) return -Infinity;
  const floor = lufs(mean(heard)) - 10;
  return lufs(mean(heard.filter((p) => lufs(p) > floor)));
}

/** The DFT of `re` + i·`im`, in place (radix-2: the length a power of two). */
function fft(re: Float64Array, im: Float64Array): void {
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
    const step = (-2 * Math.PI) / len;
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
}

/**
 * The spectral centroid (Hz): where the sound's energy sits on average, from the spectrum of the
 * middle 4096 samples of the mono mix. A fan sits low, a hi-hat high; a sound whose centroid is
 * where its kind should not be is the first thing to hear.
 */
function centroid(l: Float32Array, r: Float32Array): number {
  const N = 4096;
  const len = Math.min(l.length, r.length);
  const start = Math.max(0, Math.floor((len - N) / 2));
  const re = new Float64Array(N);
  const im = new Float64Array(N);
  // A Hann window, so a steady tone does not smear across the whole band; a sound shorter than N
  // is padded with silence.
  for (let i = 0; i < Math.min(N, len - start); i++) {
    re[i] = ((l[start + i] + r[start + i]) / 2) * (0.5 - 0.5 * Math.cos((2 * Math.PI * i) / (N - 1)));
  }
  fft(re, im);
  let sum = 0;
  let weighted = 0;
  for (let k = 1; k < N / 2; k++) {
    const mag = Math.hypot(re[k], im[k]);
    sum += mag;
    weighted += (mag * k * RATE) / N;
  }
  return sum > 0 ? weighted / sum : 0;
}

/** The level the game plays each sound at (`world.ts`; 1 where it names none). */
function levels(): Map<string, number> {
  const world = readFileSync(join(ROOT, 'web/src/audio/world.ts'), 'utf8');
  const out = new Map<string, number>();
  const loops = world.slice(world.indexOf('const LOOPS'), world.indexOf('\n};', world.indexOf('const LOOPS')));
  for (const m of loops.matchAll(/^ {2}(\w+): \{([^}]*)\}/gm)) {
    const level = /level: ([\d.]+)/.exec(m[2]);
    if (level) out.set(m[1], Number(level[1]));
  }
  const once = /const ONCE_LEVEL[^=]*= \{([^}]*)\}/.exec(world);
  if (once) for (const m of once[1].matchAll(/(\w+): ([\d.]+)/g)) out.set(m[1], Number(m[2]));
  return out;
}

/** Where the content sounds each kind: the objects and uses that name it. */
function users(): Map<string, string[]> {
  const found = new Map<string, string[]>();
  const add = (kind: string, who: string) => {
    const list = found.get(kind) ?? [];
    if (!list.includes(who)) list.push(who);
    found.set(kind, list);
  };
  const files = new Bun.Glob('web/public/content/**/*.json');
  for (const file of files.scanSync({ cwd: ROOT })) {
    let json: unknown;
    try {
      json = JSON.parse(readFileSync(join(ROOT, file), 'utf8'));
    } catch {
      continue;
    }
    const walk = (node: unknown, id: string) => {
      if (!node || typeof node !== 'object') return;
      if (Array.isArray(node)) return void node.forEach((n) => walk(n, id));
      const o = node as Record<string, unknown>;
      const name = typeof o.id === 'string' ? o.id : id;
      for (const key of ['sound', 'endSound', 'ambient']) if (typeof o[key] === 'string') add(o[key] as string, name);
      if (o.sounds && typeof o.sounds === 'object') for (const s of Object.values(o.sounds)) if (typeof s === 'string') add(s, name);
      for (const [k, v] of Object.entries(o)) if (k !== 'id') walk(v, name);
    };
    walk(json, '');
  }
  return found;
}

/** How far apart the sounds are in loudness, loops and sounds played once apart: what a mix has to even out. */
function spread(): string[] {
  const out: string[] = ['| plays | sounds | quietest | median | loudest | spread |', '|---|---|---|---|---|---|'];
  for (const loop of ['loop', 'once']) {
    const list = heard.filter((h) => h.loop === loop && Number.isFinite(h.loud)).toSorted((a, b) => a.loud - b.loud);
    if (!list.length) continue;
    const first = list[0];
    const last = list[list.length - 1];
    const mid = list[Math.floor(list.length / 2)];
    out.push(`| ${loop} | ${list.length} | \`${first.kind}\` ${num(first.loud)} | ${num(mid.loud)} | \`${last.kind}\` ${num(last.loud)} | ${num(last.loud - first.loud)} LU |`);
  }
  return out;
}

const kinds = Object.keys(MADE) as MadeKind[];
const where = users();
const level = levels();
mkdirSync(OUT, { recursive: true });
const rows: string[] = [];
const heard: { kind: string; loop: string; loud: number }[] = [];
console.log(`${kinds.length} sounds, ${RATE} Hz\n`);
for (const kind of kinds) {
  const made = (MADE[kind] as (rate: number) => { left: Float32Array; right: Float32Array })(RATE);
  // As the game plays it: the mixer's level for this sound (one against another, as heard).
  const gain = level.get(kind) ?? 1;
  const left = made.left.map((v) => v * gain);
  const right = made.right.map((v) => v * gain);
  const { peak, rms } = level_(left, right);
  const loud = loudness(left, right);
  const c = centroid(left, right);
  const secs = left.length / RATE;
  writeFileSync(join(OUT, `${kind}.wav`), wav(left, right));
  const used = where.get(kind) ?? [];
  const loop = ONCE_KINDS.has(kind) ? 'once' : 'loop';
  rows.push(`| \`${kind}\` | ${num(secs, 2)} | ${loop} | ${gain} | ${num(peak)} | ${num(rms)} | ${num(loud)} | ${num(c, 0)} | ${used.join(', ') || 'by its animation'} |`);
  heard.push({ kind, loop, loud });
  console.log(
    `${kind.padEnd(14)} ${num(secs, 2).padStart(5)} s  ${loop.padEnd(4)}  level ${String(gain).padEnd(4)}  peak ${num(peak).padStart(6)}  rms ${num(rms).padStart(6)} dBFS  ${num(loud).padStart(6)} LUFS  centroid ${num(c, 0).padStart(6)} Hz  ${used.join(', ') || '(by its animation)'}`,
  );
}
writeFileSync(
  join(OUT, 'index.md'),
  [
    '# The world\'s sounds, rendered',
    '',
    '`bun run sounds` (`tools/audio/listen.ts`): every synthesised sound of the world as a WAV, with',
    'what can be measured — length, level, where its energy sits — and the objects and uses that sound',
    'it. Listen to them here; the numbers say what to expect, not whether it sounds like the thing.',
    '',
    '| sound | s | plays | level | peak | rms | LUFS | centroid | the content that sounds it |',
    '|---|---|---|---|---|---|---|---|---|',
    ...rows,
    '',
    ...spread(),
    '',
    'Each WAV is written at the level the game plays it (`world.ts` \'s `LOOPS` and `ONCE_LEVEL`), so one',
    'against another sounds as it does in the game; peak and RMS are dBFS at that level, LUFS is how loud',
    'it is to the ear (ITU-R BS.1770, gated: the second table is how far apart the sounds are), and the centroid',
    'is the middle of the sound\'s energy in Hz (a fan sits low, a hi-hat high). A loop is made to be',
    'repeated, its end cross-faded into its start; a sound played once is a use starting or ending.',
    '',
    '"by its animation" means no content names the sound: it comes from the animation tag the use',
    'plays (`world.ts` \'s `ACTION_SOUNDS`), so it is heard wherever that animation is (an oven\'s fan',
    'on every `bake`, a shower on every `shower`). The water\'s sounds are chosen by what is washed',
    '(`soundOf`), which is why `shower`, `washUp` and `bathFill` name no object either.',
    '',
  ].join('\n'),
);
console.log(`\nwrote ${kinds.length} WAVs and index.md to ${OUT}`);
