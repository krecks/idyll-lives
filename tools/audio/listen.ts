#!/usr/bin/env bun
/**
 * Renders every synthesised sound of the world to a WAV, with the numbers that can be measured and
 * the ones only a listener can judge (`G-53`, `docs/authenticity-review.md`): the sounds are made
 * in code (`web/src/audio/water.ts`, `things.ts`), so they can be made here, off the browser, and
 * listened to on any player. `bun run sounds` (from `web/`), or `bun tools/audio/listen.ts`.
 *
 * Each is written to `bench-results/sounds/<kind>.wav` (16-bit stereo) with a line in `index.md`:
 * its length, its level (peak and RMS, dBFS), where its energy sits (the spectral centroid, Hz) and
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
const OUT = resolve(import.meta.dir, '../../bench-results/sounds');

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

/**
 * The spectral centroid (Hz): where the sound's energy sits on average, from a plain DFT over the
 * middle 4096 samples of the mono mix. A fan sits low, a hi-hat high; a sound whose centroid is
 * where its kind should not be is the first thing to hear.
 */
function centroid(l: Float32Array, r: Float32Array): number {
  const N = 4096;
  const start = Math.max(0, Math.floor((Math.min(l.length, r.length) - N) / 2));
  const x: number[] = [];
  for (let i = 0; i < N; i++) x.push((l[start + i] + r[start + i]) / 2);
  // A Hann window, so a steady tone does not smear across the whole band.
  for (let i = 0; i < N; i++) x[i] *= 0.5 - 0.5 * Math.cos((2 * Math.PI * i) / (N - 1));
  let sum = 0;
  let weighted = 0;
  for (let k = 1; k < N / 2; k++) {
    let re = 0;
    let im = 0;
    for (let i = 0; i < N; i++) {
      const a = (2 * Math.PI * k * i) / N;
      re += x[i] * Math.cos(a);
      im -= x[i] * Math.sin(a);
    }
    const mag = Math.hypot(re, im);
    const hz = (k * RATE) / N;
    sum += mag;
    weighted += mag * hz;
  }
  return sum > 0 ? weighted / sum : 0;
}

/** The level the game plays each sound at (`world.ts`; 1 where it names none). */
function levels(): Map<string, number> {
  const world = readFileSync(resolve(import.meta.dir, '../../web/src/audio/world.ts'), 'utf8');
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
  for (const file of files.scanSync({ cwd: resolve(import.meta.dir, '../..') })) {
    let json: unknown;
    try {
      json = JSON.parse(require('node:fs').readFileSync(join(resolve(import.meta.dir, '../..'), file), 'utf8'));
    } catch {
      continue;
    }
    const walk = (node: unknown, id: string) => {
      if (!node || typeof node !== 'object') return;
      if (Array.isArray(node)) return void node.forEach((n) => walk(n, id));
      const o = node as Record<string, unknown>;
      const name = typeof o.id === 'string' ? o.id : id;
      for (const key of ['sound', 'endSound', 'ambient']) if (typeof o[key] === 'string') add(o[key] as string, name);
      if (Array.isArray(o.sounds)) for (const s of o.sounds) if (typeof s === 'string') add(s, name);
      if (o.sounds && typeof o.sounds === 'object') for (const s of Object.values(o.sounds)) if (typeof s === 'string') add(s, name);
      for (const [k, v] of Object.entries(o)) if (k !== 'id') walk(v, name);
    };
    walk(json, '');
  }
  return found;
}

const kinds = Object.keys(MADE) as MadeKind[];
const where = users();
const level = levels();
mkdirSync(OUT, { recursive: true });
const rows: string[] = [];
console.log(`${kinds.length} sounds, ${RATE} Hz\n`);
for (const kind of kinds) {
  const made = (MADE[kind] as (rate: number) => { left: Float32Array; right: Float32Array })(RATE);
  // As the game plays it: the mixer's level for this sound (one against another, as heard).
  const gain = level.get(kind) ?? 1;
  const left = made.left.map((v) => v * gain);
  const right = made.right.map((v) => v * gain);
  const { peak, rms } = level_(left, right);
  const c = centroid(left, right);
  const secs = left.length / RATE;
  writeFileSync(join(OUT, `${kind}.wav`), wav(left, right));
  const used = where.get(kind) ?? [];
  const loop = ONCE_KINDS.has(kind) ? 'once' : 'loop';
  rows.push(`| \`${kind}\` | ${num(secs, 2)} | ${loop} | ${gain} | ${num(peak)} | ${num(rms)} | ${num(c, 0)} | ${used.join(', ') || 'by its animation'} |`);
  console.log(
    `${kind.padEnd(14)} ${num(secs, 2).padStart(5)} s  ${loop.padEnd(4)}  level ${String(gain).padEnd(4)}  peak ${num(peak).padStart(6)}  rms ${num(rms).padStart(6)} dBFS  centroid ${num(c, 0).padStart(6)} Hz  ${used.join(', ') || '(by its animation)'}`,
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
    '| sound | s | plays | level | peak | rms | centroid | the content that sounds it |',
    '|---|---|---|---|---|---|---|---|',
    ...rows,
    '',
    'Each WAV is written at the level the game plays it (`world.ts` \'s `LOOPS` and `ONCE_LEVEL`), so one',
    'against another sounds as it does in the game; peak and RMS are dBFS at that level, and the centroid',
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
