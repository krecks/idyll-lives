#!/usr/bin/env bun
/**
 * (For the port of the sounds to Rust; removed with the TypeScript synthesis.) Every sound the TypeScript makes,
 * as raw f32 samples, for `cargo run -p sound --example compare -- <dir>` to hold the Rust's against.
 *
 *   bun tools/audio/dump.ts <dir>
 */
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { SPECIES, songSamples, VERSIONS } from '../../web/src/audio/birdsong';
import { BLIPS, blipSamples, type Blip } from '../../web/src/audio/blips';
import { makeSound, type WorkerKind } from '../../web/src/audio/made';
import { MONO, nodeFilter, type MonoKind } from '../../web/src/audio/mono';
import { MUSIC } from '../../web/src/audio/music';
import { OUTDOORS } from '../../web/src/audio/outdoors';
import { ROOM_SIZES, roomResponse } from '../../web/src/audio/rooms';
import { FOOTWEAR, STEP_VERSIONS, stepSamples, SURFACES } from '../../web/src/audio/steps';
import { MADE } from '../../web/src/audio/things';
import { random, type Stereo } from '../../web/src/audio/water';

const dir = process.argv[2];
if (!dir) throw new Error('dump: which folder?');
mkdirSync(dir, { recursive: true });
const manifest: { name: string; channels: number; length: number }[] = [];
const only = process.argv[3];

function put(name: string, made: () => Stereo | Float32Array): void {
  if (only && !name.startsWith(only)) return;
  const t = performance.now();
  const s = made();
  const chans = s instanceof Float32Array ? [s] : [s.left, s.right];
  const out = new Float32Array(chans.reduce((n, c) => n + c.length, 0));
  let at = 0;
  for (const c of chans) {
    out.set(c, at);
    at += c.length;
  }
  writeFileSync(join(dir, name.replaceAll('/', '__') + '.f32'), new Uint8Array(out.buffer));
  manifest.push({ name, channels: chans.length, length: chans[0].length });
  console.log(`${name} ${chans[0].length} ${(performance.now() - t).toFixed(0)} ms`);
}

for (const rate of [48000, 44100]) {
  const all = rate === 48000;
  for (const kind of [...Object.keys(MADE), ...Object.keys(OUTDOORS), ...Object.keys(MUSIC)]) {
    if (all || ['tap', 'darts', 'carEngine', 'leaves', 'musicSlow'].includes(kind)) put(`make/${rate}/${kind}`, () => makeSound(kind as WorkerKind, rate));
  }
  for (const kind of Object.keys(MONO) as MonoKind[]) put(`mono/${rate}/${kind}`, () => MONO[kind](rate));
  for (const name of Object.keys(BLIPS) as Blip[]) put(`blip/${rate}/${name}`, () => blipSamples(name, rate));
  for (const surface of SURFACES) for (const wear of FOOTWEAR) for (let v = 0; v < STEP_VERSIONS; v++) if (all || v === 1) put(`step/${rate}/${surface}/${wear}/${v}`, () => stepSamples(surface, wear, v, rate));
  SPECIES.forEach((sp, i) => {
    for (let v = 0; v < VERSIONS; v++) {
      if (!all && v) continue;
      // The catalog's unseen bird of the species, and a bird of the world's: its own pitch, pace and nearness.
      put(`song/${rate}/${sp.name}/${i}/${(sp.pitch[0] + sp.pitch[1]) / 2}/1/0.575/${v}`, () => songSamples({ species: sp.name, seed: i, pitch: (sp.pitch[0] + sp.pitch[1]) / 2, pace: 1, near: 0.575 }, v, rate));
      const p = sp.pitch[0] + 0.3 * (sp.pitch[1] - sp.pitch[0]);
      put(`song/${rate}/${sp.name}/${i + 7}/${p}/1.07/1/${v}`, () => songSamples({ species: sp.name, seed: i + 7, pitch: p, pace: 1.07, near: 1 }, v, rate));
    }
  });
  for (const size of ROOM_SIZES) put(`room/${rate}/${size}`, () => roomResponse(size, rate));
  const r = random(99);
  const noise = Float32Array.from({ length: rate }, () => r() * 2 - 1);
  for (const [kind, hz, q] of [['lowpass', 650, 0.5], ['highpass', 1800, -3.01], ['bandpass', 1400, 1.2]] as const) {
    for (const looped of [true, false]) put(`filter/${rate}/${kind}/${hz}/${q}/${looped}`, () => nodeFilter(noise, rate, kind, hz, q, looped));
  }
}
writeFileSync(join(dir, 'manifest.json'), JSON.stringify(manifest, null, 1));
