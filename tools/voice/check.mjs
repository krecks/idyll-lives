#!/usr/bin/env node
/**
 * Checks the voices as the browser makes them (`engine.mjs`: our voice WASM, ONNX Runtime Web's
 * WASM build, the worker's own `pocket.ts`), with Pocket TTS:
 *
 * - the same text and voice give bit-identical samples, in one session and in a fresh one (the
 *   clip cache in `web/src/voice/service.svelte.ts` relies on it), and match golden hashes (a
 *   change to the model, the noise, the shaping or the phonemizer shows up here);
 * - the voice controls work: pitch on a line changes the sound but not the timing, speed the
 *   length, depth the brightness; pitch put into the clip moves the voice the model speaks in;
 *   expressiveness changes the samples; silence is trimmed;
 * - and prints the time per line.
 *
 * Needs `pnpm voice` first. Usage (from web/): `pnpm check:voice` (`VOICE_THREADS=4` to try threads).
 */

import { createHash } from 'node:crypto';
import { SAMPLE_RATE, engine, medianPitch, phonemizer, prepareVoice, synthesizeLine } from './engine.mjs';

const LINES = [
  'Hello there!',
  "I'm so hungry, I could eat a whole pizza by myself.",
  'What a lovely garden. I should water the roses before it gets dark.',
];
const plain = { clipPitch: 1, clipDepth: 1, pitch: 1, depth: 1, speed: 1, temperature: 0.7 };
/** Charles (one of the men), and Anna (a woman) as a child would sound. */
const CHARLES = { ...plain, clip: 'preset:Charles' };
const CHILD = { ...plain, clip: 'preset:Anna', clipPitch: 1.2, clipDepth: 1.08, pitch: 1.03, depth: 1.01, speed: 1.04, temperature: 0.62 };
/**
 * Pocket TTS's output for `LINES` in `CHARLES`, then line 2 as `CHILD` (SHA-256, first 12 hex
 * digits). WASM arithmetic is the same on every machine, so these hold everywhere; update them
 * only on purpose (a new model, noise, shaping or phonemizer rule).
 */
const GOLDEN = ['85cedf692337', '9f04fb79f73a', '854d44ca42c7', 'dc9eb65ad122'];

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

// Preparing a voice (shaping its clip, encoding it, feeding it to the model), then reusing it.
const prep = await prepareVoice(first, CHARLES);
if (!(prep.ms > 0)) fail('preparing a new voice should take time');
if ((await prepareVoice(first, CHARLES)).ms !== 0) fail('a prepared voice should be kept');
console.log(`voice: preparing a voice took ${prep.ms.toFixed(0)} ms (kept after that)`);

const line = LINES[1];
const got = [];
for (const l of LINES) got.push(await steady('Pocket TTS', l, CHARLES));
got.push(await steady('Pocket TTS child', line, CHILD));
const hashes = got.map(hash);
if (GOLDEN.some((g, i) => g !== hashes[i])) fail(`Pocket TTS: samples changed: [${hashes.join(', ')}], expected [${GOLDEN.join(', ')}] (update GOLDEN only on purpose)`);
console.log('voice: Pocket TTS: matches the golden samples');

const base = got[1];
const low = await say(first, line, { ...CHARLES, pitch: 0.85 });
const fast = await say(first, line, { ...CHARLES, speed: 1.25 });
const [p0, p1] = [medianPitch(base), medianPitch(low)];
if (same(low, base) || !(p1 < p0 * 0.93)) fail(`pitch 0.85 should lower the pitch (${p0.toFixed(0)} Hz to ${p1.toFixed(0)} Hz)`);
if (Math.abs(low.length - base.length) > 0.05 * base.length) fail('pitch should not change the timing');
if (!(fast.length < base.length * 0.85)) fail('a faster voice should be shorter');
// Depth on a clear voice (Charles is low and creaky: few grains to move).
const anna = await say(first, line, { ...CHARLES, clip: 'preset:Anna' });
const large = await say(first, line, { ...CHARLES, clip: 'preset:Anna', depth: 0.88 });
if (!(centroid(large) < centroid(anna) * 0.95)) fail(`a larger voice should sound darker (${centroid(anna).toFixed(0)} Hz to ${centroid(large).toFixed(0)} Hz)`);
if (Math.abs(large.length - anna.length) > 0.05 * anna.length) fail('depth should not change the timing');
// Pitch put into the clip: the model speaks higher.
const annaHigh = await say(first, line, { ...CHARLES, clip: 'preset:Anna', clipPitch: 1.2 });
const [a0, a1] = [medianPitch(anna), medianPitch(annaHigh)];
if (!(a1 > a0 * 1.08)) fail(`clip pitch 1.2 should raise the voice (${a0.toFixed(0)} Hz to ${a1.toFixed(0)} Hz)`);
const lively = await say(first, line, { ...CHARLES, temperature: 1 });
if (same(lively, base)) fail('expressiveness should change the samples');
if (phonemizer.line('…', 1, 1, 1).text !== '') fail('text without words should give nothing to say (the worker then skips the model)');
if ((await say(first, '', CHARLES)).length !== 0) fail('empty text should give no samples');
// Trimmed: speech starts within 0.1 s.
const peak = base.reduce((m, x) => Math.max(m, Math.abs(x)), 0);
const onset = base.findIndex((v) => Math.abs(v) > 0.05 * peak);
if (onset > 0.1 * SAMPLE_RATE) fail(`speech starts after ${(onset / SAMPLE_RATE).toFixed(2)} s: silence not trimmed`);
console.log(
  `voice: Pocket TTS: line pitch (${p0.toFixed(0)} to ${p1.toFixed(0)} Hz at x0.85), clip pitch (${a0.toFixed(0)} to ${a1.toFixed(0)} Hz at x1.2), speed, depth and expressiveness work; speech starts at ${(onset / SAMPLE_RATE).toFixed(2)} s`,
);
