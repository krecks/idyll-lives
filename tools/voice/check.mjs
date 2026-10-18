#!/usr/bin/env node
/**
 * Checks the voices as the browser makes them (`engine.mjs`: our voice WASM, ONNX Runtime Web's
 * WASM build, the worker's own `supertonic.ts`), with Supertonic 3:
 *
 * - the same text and voice give bit-identical samples, in one session and in a fresh one (the
 *   clip cache in `web/src/voice/service.svelte.ts` relies on it), and match golden hashes (a
 *   change to the model, the noise, the shaping or the phonemizer shows up here);
 * - every voice style speaks, and prints its median pitch (`Preset.f0` in `voices.ts`);
 * - the voice controls work: pitch on a line changes the sound but not the timing, speed the
 *   length, depth the brightness; silence is trimmed;
 * - and prints the time per line.
 *
 * In Node the model runs on the CPU (no WebGPU): the game's GPU rounds a little differently, so
 * the golden hashes hold here, not in the browser. Needs `pnpm voice` first. Usage (from web/):
 * `pnpm check:voice` (`VOICE_THREADS` to change the threads, default 4).
 */

import { createHash } from 'node:crypto';
import { PRESETS, SAMPLE_RATE, STYLE_IDS, engine, medianPitch, phonemizer, synthesizeLine } from './engine.mjs';

const LINES = [
  'Hello there!',
  "I'm so hungry, I could eat a whole pizza by myself.",
  'What a lovely garden. I should water the roses before it gets dark.',
];
const plain = { pitch: 1, depth: 1, speed: 1 };
/** Arlo (one of the men), and Ada (a woman) as a child would sound. */
const ARLO = { ...plain, style: 'M1' };
const CHILD = { style: 'F1', pitch: 1.3, depth: 1.08, speed: 1.04 };
/**
 * Supertonic's output for `LINES` as `ARLO`, then line 2 as `CHILD` (SHA-256, first 12 hex
 * digits). Update them only on purpose (a new model, noise, steps, shaping or phonemizer rule).
 */
const GOLDEN = ['8cff13df1304', '222e114d9825', '1f0834091af4', 'd5f80af5ed12'];

const hash = (a) => createHash('sha256').update(Buffer.from(a.buffer, a.byteOffset, a.byteLength)).digest('hex').slice(0, 12);
const same = (a, b) => a.length === b.length && Buffer.from(a.buffer, a.byteOffset, a.byteLength).equals(Buffer.from(b.buffer, b.byteOffset, b.byteLength));
const fail = (message) => {
  console.error(`voice check FAILED: ${message}`);
  process.exit(1);
};
const rms = (x) => Math.sqrt(x.reduce((s, v) => s + v * v, 0) / Math.max(1, x.length));

/** Mean spectral centroid (Hz) of the loud frames, by a direct DFT (no dependencies). */
function centroid(x) {
  const n = 512;
  let sum = 0;
  let count = 0;
  for (let at = 0; at + n <= x.length; at += n * 2) {
    const frame = x.subarray(at, at + n);
    if (Math.sqrt(frame.reduce((s, v) => s + v * v, 0) / n) < 0.02) continue;
    let num = 0;
    let den = 0;
    for (let k = 1; k < n / 2; k++) {
      let re = 0;
      let im = 0;
      for (let i = 0; i < n; i++) {
        const v = frame[i] * (0.5 - 0.5 * Math.cos((2 * Math.PI * i) / n));
        re += v * Math.cos((2 * Math.PI * k * i) / n);
        im += v * Math.sin((2 * Math.PI * k * i) / n);
      }
      const mag = Math.hypot(re, im);
      num += (mag * k * SAMPLE_RATE) / n;
      den += mag;
    }
    sum += num / den;
    count++;
  }
  return sum / count;
}

const first = await engine();
const second = await engine();
const say = async (e, text, voice) => (await synthesizeLine(e, text, voice)).samples;

/** Same samples twice in one engine and once in a fresh one; prints the time. */
async function steady(label, line, voice) {
  const a = await say(first, line, voice);
  const start = performance.now();
  const b = await say(first, line, voice);
  const ms = performance.now() - start;
  const c = await say(second, line, voice);
  if (!same(a, b)) fail(`${label}: "${line}" differs between two runs`);
  if (!same(a, c)) fail(`${label}: "${line}" differs between two sessions`);
  if (!(rms(a) > 0.01)) fail(`${label}: "${line}" is silent`);
  const secs = a.length / SAMPLE_RATE;
  console.log(`voice: ${label}: ${secs.toFixed(2)} s of audio in ${ms.toFixed(0)} ms (${(secs / (ms / 1000)).toFixed(1)}x real time), identical every time [${hash(a)}]  ${line}`);
  return a;
}

const line = LINES[1];
const got = [];
for (const l of LINES) got.push(await steady('Supertonic', l, ARLO));
got.push(await steady('Supertonic child', line, CHILD));
const hashes = got.map(hash);
if (GOLDEN[0] === 'PENDING') console.log(`voice: golden samples not pinned yet: [${hashes.map((h) => `'${h}'`).join(', ')}]`);
else if (GOLDEN.some((g, i) => g !== hashes[i])) fail(`Supertonic: samples changed: [${hashes.join(', ')}], expected [${GOLDEN.join(', ')}] (update GOLDEN only on purpose)`);
else console.log('voice: Supertonic: matches the golden samples');

// Every voice style speaks; its median pitch is what children's voices aim from.
const pitches = [];
for (const id of STYLE_IDS) {
  const x = await say(first, LINES[2], { ...plain, style: id });
  if (!(rms(x) > 0.01)) fail(`voice ${id} is silent`);
  pitches.push(`${PRESETS.find((p) => p.style === id)?.name ?? id} (${id}) ${medianPitch(x).toFixed(0)} Hz`);
}
console.log(`voice: median pitch: ${pitches.join(', ')}`);

const base = got[1];
const low = await say(first, line, { ...ARLO, pitch: 0.85 });
const fast = await say(first, line, { ...ARLO, speed: 1.25 });
const [p0, p1] = [medianPitch(base), medianPitch(low)];
if (same(low, base) || !(p1 < p0 * 0.93)) fail(`pitch 0.85 should lower the pitch (${p0.toFixed(0)} Hz to ${p1.toFixed(0)} Hz)`);
if (Math.abs(low.length - base.length) > 0.05 * base.length) fail('pitch should not change the timing');
if (!(fast.length < base.length * 0.85)) fail('a faster voice should be shorter');
const ada = await say(first, line, { ...plain, style: 'F1' });
const large = await say(first, line, { ...plain, style: 'F1', depth: 0.88 });
if (!(centroid(large) < centroid(ada) * 0.95)) fail(`a larger voice should sound darker (${centroid(ada).toFixed(0)} Hz to ${centroid(large).toFixed(0)} Hz)`);
if (Math.abs(large.length - ada.length) > 0.05 * ada.length) fail('depth should not change the timing');
if (phonemizer.line('…', 1, 1, 1).text !== '') fail('text without words should give nothing to say (the worker then skips the model)');
if ((await say(first, '', ARLO)).length !== 0) fail('empty text should give no samples');
// Trimmed: speech starts within 0.1 s.
const peak = base.reduce((m, x) => Math.max(m, Math.abs(x)), 0);
const onset = base.findIndex((v) => Math.abs(v) > 0.05 * peak);
if (onset > 0.1 * SAMPLE_RATE) fail(`speech starts after ${(onset / SAMPLE_RATE).toFixed(2)} s: silence not trimmed`);
console.log(`voice: Supertonic: line pitch (${p0.toFixed(0)} to ${p1.toFixed(0)} Hz at x0.85), speed and depth work; speech starts at ${(onset / SAMPLE_RATE).toFixed(2)} s`);
