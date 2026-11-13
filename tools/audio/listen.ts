#!/usr/bin/env bun
/**
 * Renders every synthesised sound of the game to a WAV, with the numbers that can be measured and
 * the ones only a listener can judge: the sounds are made
 * in code (`web/src/audio/`: `water.ts`, `things.ts`, `mono.ts`, `birdsong.ts`, `blips.ts`), so they
 * can be made here, off the browser, and listened to on any player. `bun run sounds` (from `web/`), or
 * `bun tools/audio/listen.ts [--out <folder>]`.
 *
 * Each is written to `bench-results/sounds/<name>.wav` (16-bit stereo) with a line in `index.md`:
 * its length, its level (peak and RMS, dBFS; its loudness to the ear, LUFS), where its energy sits (the spectral centroid, Hz) and
 * whether it loops or is played once; then the objects and uses of the content that sound it, so a
 * listener knows what to expect before pressing play.
 *
 * Each is written at the level the game plays it and as it comes out in the middle of the picture
 * (`web/src/audio/catalog.ts`, the list `web/tests/sounds.test.ts` measures too), so one sound
 * against another sounds as it does in the game.
 *
 * What a machine can say is here; whether a microwave sounds like a microwave is the ear's part.
 */

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { isAbsolute, join, resolve } from 'node:path';
import { everySound, GUARD, STEPS, type Group } from '../../web/src/audio/catalog';
import { loudness as loudnessAt } from '../../web/src/audio/loudness';
import { fft, spaced, type Hearing } from '../../web/src/audio/offline';
import { OUTDOORS } from '../../web/src/audio/outdoors';
import { carSound, leavesLevel, nearness } from '../../web/src/audio/outside';
import { ROOM_SIZES, ROOMS, roomResponse } from '../../web/src/audio/rooms';
import { stepSamples, STEP_VERSIONS, SURFACES, type Footwear, type Surface } from '../../web/src/audio/steps';
import { random } from '../../web/src/audio/water';

const RATE = 48_000;
/** The checkout. */
const ROOT = resolve(import.meta.dir, '../..');
/** Where the WAVs go: `--out <folder>` (from the checkout), else `bench-results/sounds`. */
const named = process.argv.indexOf('--out') >= 0 ? process.argv[process.argv.indexOf('--out') + 1] : undefined;
const OUT = named ? (isAbsolute(named) ? named : join(ROOT, named)) : join(ROOT, 'bench-results/sounds');

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
 * The spectral centroid (Hz): where the sound's energy sits on average, from the spectra of 4096
 * samples of the mono mix every 2048 along the whole sound, added up by their energy (so the silences
 * between a game's knocks or a cricket's trills don't count). A fan sits low, a hi-hat high; a sound
 * whose centroid is where its kind should not be is the first thing to hear.
 */
function centroid(l: Float32Array, r: Float32Array): number {
  const N = 4096;
  const len = Math.min(l.length, r.length);
  const power = new Float64Array(N / 2);
  const re = new Float64Array(N);
  const im = new Float64Array(N);
  for (let start = 0; start === 0 || start + N <= len; start += N / 2) {
    re.fill(0);
    im.fill(0);
    // A Hann window, so a steady tone does not smear across the whole band; a sound shorter than N
    // is padded with silence.
    for (let i = 0; i < Math.min(N, len - start); i++) {
      re[i] = ((l[start + i] + r[start + i]) / 2) * (0.5 - 0.5 * Math.cos((2 * Math.PI * i) / (N - 1)));
    }
    fft(re, im);
    for (let k = 1; k < N / 2; k++) power[k] += re[k] * re[k] + im[k] * im[k];
  }
  let sum = 0;
  let weighted = 0;
  for (let k = 1; k < N / 2; k++) {
    const mag = Math.sqrt(power[k]);
    sum += mag;
    weighted += (mag * k * RATE) / N;
  }
  return sum > 0 ? weighted / sum : 0;
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

/** Who makes the sounds no content names. */
const WHO: Record<Group, string> = { things: 'by its animation', steps: 'a resident walking', ambience: 'the outdoors', interface: 'the interface' };

/** How far apart the sounds are in loudness, by what they belong to, loops and sounds played once apart: what a mix has to even out. */
function spread(): string[] {
  const out: string[] = ['| of | plays | sounds | quietest | median | loudest | spread | held within |', '|---|---|---|---|---|---|---|---|'];
  for (const group of Object.keys(GUARD) as Group[]) {
    for (const plays of ['loop', 'once']) {
      const list = heard.filter((h) => h.group === group && h.plays === plays && Number.isFinite(h.loud)).toSorted((a, b) => a.loud - b.loud);
      if (!list.length) continue;
      const first = list[0];
      const last = list[list.length - 1];
      const mid = list[Math.floor(list.length / 2)];
      out.push(`| ${group} | ${plays} | ${list.length} | \`${first.name}\` ${num(first.loud)} | ${num(mid.loud)} | \`${last.name}\` ${num(last.loud)} | ${num(last.loud - first.loud)} LU | ${GUARD[group][0]} to ${GUARD[group][1]} |`);
    }
  }
  return out;
}

const sounds = everySound();
const where = users();
mkdirSync(OUT, { recursive: true });
const rows: string[] = [];
const heard: { name: string; group: Group; plays: string; loud: number }[] = [];
console.log(`${sounds.length} sounds, ${RATE} Hz\n`);
for (const { name, group, plays, level, note, who: by, heard: make } of sounds) {
  // As the game plays it: at its level, in the middle of the picture.
  const { left, right } = make(RATE);
  const { peak, rms } = level_(left, right);
  const loud = loudnessAt(left, right, RATE);
  const c = centroid(left, right);
  const secs = left.length / RATE;
  writeFileSync(join(OUT, `${name}.wav`), wav(left, right));
  const used = group === 'things' ? (where.get(name) ?? []) : [];
  const who = used.join(', ') || (by ?? WHO[group]) + (note ? ` (${note})` : '');
  const shown = String(Number(level.toFixed(4)));
  rows.push(`| \`${name}\` | ${group} | ${num(secs, 2)} | ${plays} | ${shown} | ${num(peak)} | ${num(rms)} | ${num(loud)} | ${num(c, 0)} | ${who} |`);
  heard.push({ name, group, plays, loud });
  console.log(
    `${name.padEnd(15)} ${group.padEnd(9)} ${num(secs, 2).padStart(5)} s  ${plays.padEnd(4)}  level ${shown.padEnd(6)}  peak ${num(peak).padStart(6)}  rms ${num(rms).padStart(6)} dBFS  ${num(loud).padStart(6)} LUFS  centroid ${num(c, 0).padStart(6)} Hz  ${who}`,
  );
}
// The same sounds heard from somewhere: in a room, through a wall, from far off (`space.ts`, worked out by `offline.ts`).
const SPACED = ['tv', 'shower', 'knock'];
const FROM: [string, Hearing, string][] = [
  ...ROOM_SIZES.map((size): [string, Hearing, string] => [`room-${size}`, { room: size }, `in a ${size} room (its reverb ${num(db(ROOMS[size].send))} dB under the sound)`]),
  ['wall', { muffle: 1 }, 'from another room, the camera close in on this one'],
  ['far', { radius: 95 }, 'from the play camera\'s farthest, 95 m'],
  ['town', { radius: 400 }, 'from the town view, 400 m'],
];
const spacedRows: string[] = [];
console.log('\nheard from somewhere');
for (const name of SPACED) {
  const sound = sounds.find((s) => s.name === name)!;
  const dry = sound.heard(RATE);
  const plain = loudnessAt(dry.left, dry.right, RATE);
  for (const [tag, from, words] of FROM) {
    const { left, right } = spaced(dry, RATE, from, sound.plays === 'loop');
    const loud = loudnessAt(left, right, RATE);
    const c = centroid(left, right);
    const file = `space-${name}-${tag}.wav`;
    writeFileSync(join(OUT, file), wav(left, right));
    spacedRows.push(`| \`${file}\` | \`${name}\` ${words} | ${num(loud)} | ${num(loud - plain)} | ${num(c, 0)} |`);
    console.log(`${file.padEnd(30)} ${num(loud).padStart(6)} LUFS  ${num(loud - plain).padStart(6)} LU  centroid ${num(c, 0).padStart(6)} Hz  ${words}`);
  }
}
for (const size of ROOM_SIZES) {
  const { left, right } = roomResponse(size, RATE);
  writeFileSync(join(OUT, `room-${size}.wav`), wav(left, right));
}

/**
 * Someone walking for `seconds` at `cycle` s a stride (two steps), as `footsteps.ts` plays it in the middle of the
 * picture: a version of the step picked each time, the left foot a touch lower than the right, no two quite alike.
 */
function walk(surface: Surface, wear: Footwear, cycle: number, gear: number, seconds: number): Float32Array {
  const r = random(0x3a1c + SURFACES.indexOf(surface));
  const d = new Float32Array(Math.round(seconds * RATE));
  for (let n = 0; 0.2 + n * (cycle / 2) < seconds - 0.4; n++) {
    const step = stepSamples(surface, wear, Math.floor(r() * STEP_VERSIONS), RATE);
    const speed = (n % 2 ? 1.02 : 0.98) * (0.95 + 0.1 * r());
    const gain = STEPS.level[surface][wear] * STEPS.gear[gear] * (0.8 + 0.2 * r()) * Math.SQRT1_2;
    const at = Math.round((0.2 + n * (cycle / 2)) * RATE);
    // (Played a little faster or slower: read between its samples.)
    for (let i = 0; i / speed < step.length - 1 && at + i < d.length; i++) {
      const p = i * speed;
      const k = Math.floor(p);
      if (k + 1 < step.length) d[at + i] += gain * (step[k] + (step[k + 1] - step[k]) * (p - k));
    }
  }
  return d;
}
const walkRows: string[] = [];
console.log('\nwalking');
for (const [surface, wear, cycle, gear, words] of [
  ...SURFACES.map((s): [Surface, Footwear, number, number, string] => [s, 'shoe', 1.1, 0, `a walk in shoes on ${s}`]),
  ...SURFACES.map((s): [Surface, Footwear, number, number, string] => [s, 'bare', 1.1, 0, `a walk barefoot on ${s}`]),
  ['paving', 'shoe', 0.7, 1, 'a jog on paving'] as [Surface, Footwear, number, number, string],
  ['grass', 'shoe', 0.6, 2, 'a run on grass'] as [Surface, Footwear, number, number, string],
]) {
  const d = walk(surface, wear, cycle, gear, 6);
  const loud = loudnessAt(d, d, RATE);
  const file = `walk-${surface}-${wear}${gear ? (gear === 1 ? '-jog' : '-run') : ''}.wav`;
  writeFileSync(join(OUT, file), wav(d, d));
  walkRows.push(`| \`${file}\` | ${words}, 6 s | ${num(loud)} | ${num(centroid(d, d), 0)} |`);
  console.log(`${file.padEnd(30)} ${num(loud).padStart(6)} LUFS  centroid ${num(centroid(d, d), 0).padStart(6)} Hz  ${words}`);
}

/** A loop read at `at` samples (between its samples, round its end). */
function read(loop: Float32Array, at: number): number {
  const k = Math.floor(at);
  const a = loop[k % loop.length];
  return a + (loop[(k + 1) % loop.length] - a) * (at - k);
}

/**
 * A car driving by at `speed` m/s on screen along a road `aside` m from where the camera looks (24 m out), left
 * to right over `seconds`, as `outside.ts` plays it: its level by how near it is, its place by where it is.
 */
function carPass(speed: number, aside: number, seconds: number): [Float32Array, Float32Array] {
  const engine = OUTDOORS.carEngine(RATE);
  const tyres = OUTDOORS.carTyres(RATE);
  const left = new Float32Array(Math.round(seconds * RATE));
  const right = new Float32Array(left.length);
  const s = carSound(speed, 1);
  for (let i = 0, e = 0, t = 0; i < left.length; i++, e += s.engineRate, t += s.tyresRate) {
    const along = speed * (i / RATE - seconds / 2);
    const near = nearness(Math.hypot(aside, along), 24);
    // (The picture is about 14 m to either side at that distance; off it, 0.8 of the way to that side.)
    const pan = Math.max(-1, Math.min(1, along / 14)) * 0.8;
    const v = (read(engine, e) * s.engine + read(tyres, t) * s.tyres) * near;
    left[i] = v * Math.cos(((pan + 1) * Math.PI) / 4);
    right[i] = v * Math.sin(((pan + 1) * Math.PI) / 4);
  }
  return [left, right];
}

/** The wind in the leaves for `seconds` as gusts roll by (the gust rising and falling every few seconds), from the game's own zoom, outdoors. */
function windy(seconds: number): [Float32Array, Float32Array] {
  const loop = OUTDOORS.leaves(RATE);
  const left = new Float32Array(Math.round(seconds * RATE));
  const right = new Float32Array(left.length);
  for (let i = 0; i < left.length; i++) {
    const t = i / RATE;
    const gust = Math.max(0, Math.min(1, 0.5 + 0.45 * Math.sin(t * 0.9) + 0.25 * Math.sin(t * 0.37 + 1)));
    const level = leavesLevel(gust, 24, 0);
    left[i] = loop.left[i % loop.left.length] * level;
    right[i] = loop.right[i % loop.right.length] * level;
  }
  return [left, right];
}

const outsideRows: string[] = [];
console.log('\nthe street and the weather');
for (const [file, [left, right], words] of [
  ['car-pass-street.wav', carPass(30, 10, 6), 'a car driving by at a street\'s speed (30 m/s on screen), 10 m from where the camera looks, left to right'],
  ['car-pass-slow.wav', carPass(8, 10, 10), 'a car driving by slowly (8 m/s on screen), 10 m from where the camera looks'],
  ['car-pass-far.wav', carPass(30, 40, 6), 'the same car at a street\'s speed, 40 m off'],
  ['wind-gusts.wav', windy(20), 'the wind in the leaves over 20 s of gusts'],
] as [string, [Float32Array, Float32Array], string][]) {
  const loud = loudnessAt(left, right, RATE);
  const { peak } = level_(left, right);
  writeFileSync(join(OUT, file), wav(left, right));
  outsideRows.push(`| \`${file}\` | ${words} | ${num(loud)} | ${num(peak)} | ${num(centroid(left, right), 0)} |`);
  console.log(`${file.padEnd(30)} ${num(loud).padStart(6)} LUFS  peak ${num(peak).padStart(6)}  centroid ${num(centroid(left, right), 0).padStart(6)} Hz  ${words}`);
}

writeFileSync(
  join(OUT, 'index.md'),
  [
    '# The game\'s sounds, rendered',
    '',
    '`bun run sounds` (`tools/audio/listen.ts`): every synthesised sound of the game as a WAV, with',
    'what can be measured — length, level, where its energy sits — and the objects and uses that sound',
    'it. Listen to them here; the numbers say what to expect, not whether it sounds like the thing.',
    '',
    '| sound | of | s | plays | level | peak | rms | LUFS | centroid | who sounds it |',
    '|---|---|---|---|---|---|---|---|---|---|',
    ...rows,
    '',
    ...spread(),
    '',
    'Each WAV is written at the level the game plays it (`web/src/audio/catalog.ts`) and as it comes out in the',
    'middle of the picture (a sound of one channel through its panner: 0.71 of it each side), so one',
    'against another sounds as it does in the game; peak and RMS are dBFS at that level, LUFS is how loud',
    'it is to the ear (ITU-R BS.1770, gated; a sound under 0.4 s over its own length: the second table is how far',
    'apart the sounds are, and the range the tests hold each group to), and the centroid',
    'is the middle of the sound\'s energy in Hz (a fan sits low, a hi-hat high). A loop is made to be',
    'repeated, its end running on into its start; a sound played once is a use starting or ending, a bird\'s',
    'song, a knock, a click of the interface.',
    '',
    '## Heard from somewhere',
    '',
    'Three of them as the mix places them (`web/src/audio/space.ts`): in a room of each size (the sound plus',
    'its reverb), through a wall, and from far off. `room-small.wav`, `room-medium.wav` and `room-large.wav`',
    'are the rooms\' responses themselves (a click heard in each).',
    '',
    '| file | what | LUFS | against the sound alone | centroid |',
    '|---|---|---|---|---|',
    ...spacedRows,
    '',
    '## Walking',
    '',
    'One resident walking for six seconds on each surface, as the footsteps are played (`web/src/audio/footsteps.ts`:',
    'a version of the step picked each time, the feet a touch apart in pitch), in the middle of the picture, dry.',
    '',
    '| file | what | LUFS | centroid |',
    '|---|---|---|---|',
    ...walkRows,
    '',
    '## The street and the weather',
    '',
    'A car passing and the wind over a while, as `web/src/audio/outside.ts` plays them (the rows `car`, `town`',
    'and `leaves` above are their loops at full level).',
    '',
    '| file | what | LUFS | peak | centroid |',
    '|---|---|---|---|---|',
    ...outsideRows,
    '',
    '"by its animation" means no content names the sound: it comes from the animation tag the use',
    'plays (`world.ts` \'s `ACTION_SOUNDS`), so it is heard wherever that animation is (an oven\'s fan',
    'on every `bake`, a shower on every `shower`). The water\'s sounds are chosen by what is washed',
    '(`soundOf`), which is why `shower`, `washUp` and `bathFill` name no object either.',
    '',
  ].join('\n'),
);
console.log(`\nwrote ${sounds.length} WAVs and index.md to ${OUT}`);
