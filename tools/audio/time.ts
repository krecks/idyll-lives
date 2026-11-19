#!/usr/bin/env bun
/**
 * How long each of the game's sounds takes to make, through the sounds' WebAssembly as the game makes them
 * (`crates/sound`; `bun run sound` first): `bun run sounds:time` (from `web/`), or `bun tools/audio/time.ts`.
 *
 * Every sound once at 48 kHz: the pieces of music, the water's and the things' loops, the outdoors', the sounds
 * of one channel, the interface's blips, each step (surface × footwear, version 0), each species' song
 * (version 0) and the three rooms. A sound's time is the least of `--runs <n>` (default 3; the first also warms
 * the WebAssembly up). One line a sound, slowest first, then the total of each family.
 *
 * - `--json <file>`: also writes the numbers there (from the checkout, as `listen.ts`'s `--out`).
 * - `--compare <file>`: a file `--json` wrote before; prints before → after and the ratio of each row.
 * - `--max-music <ms>`: exits 1 when the music's total is over it (the machine's own speed: by hand, not in CI).
 * - `--wasm <file>`: times that build of `crates/sound-wasm` instead (its exports the same as the built one's).
 */

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, isAbsolute, join, resolve } from 'node:path';
import { BLIPS, FOOTWEAR, MADE, MONO, MUSIC, OUTDOORS, ROOM_SIZES, SPECIES, SURFACES } from '../../web/src/audio/kinds';
import { blipSamples, loadSynth, makeSound, monoSamples, roomResponse, songSamples, stepSamples } from '../../web/src/audio/synth';

const RATE = 48_000;
/** The checkout. */
const ROOT = resolve(import.meta.dir, '../..');

/** The value after `--name`, if given. */
function arg(name: string): string | undefined {
  const i = process.argv.indexOf(name);
  return i >= 0 ? process.argv[i + 1] : undefined;
}
/** A path from the checkout (or as it is, if absolute). */
const path = (p: string) => (isAbsolute(p) ? p : join(ROOT, p));

// The sounds' WebAssembly as `bun run sound` built it, or `--wasm <file>`: another build of the same crate (an
// older one kept aside, timed against this one on the same machine at the same time).
const wasm = arg('--wasm');
await loadSynth(readFileSync(wasm ? path(wasm) : new URL('../../web/src/audio/wasm-pkg/sound_wasm_bg.wasm', import.meta.url)));

const RUNS = Math.max(1, Number(arg('--runs') ?? 3) | 0);
const FAMILIES = ['music', 'made', 'outdoors', 'mono', 'blips', 'steps', 'birds', 'rooms'] as const;
type Family = (typeof FAMILIES)[number];

/** A sound to time: its name (as `bun run sounds` lists it), its family and how it is made. */
interface Job {
  name: string;
  family: Family;
  make: () => unknown;
}

const jobs: Job[] = [
  ...MUSIC.map((name): Job => ({ name, family: 'music', make: () => makeSound(name, RATE) })),
  ...MADE.map((name): Job => ({ name, family: 'made', make: () => makeSound(name, RATE) })),
  ...OUTDOORS.map((name): Job => ({ name, family: 'outdoors', make: () => makeSound(name, RATE) })),
  ...MONO.map((name): Job => ({ name, family: 'mono', make: () => monoSamples(name, RATE) })),
  ...BLIPS.map((name): Job => ({ name: `ui-${name}`, family: 'blips', make: () => blipSamples(name, RATE) })),
  ...SURFACES.flatMap((surface) =>
    FOOTWEAR.map((wear): Job => ({ name: `step-${surface}-${wear}`, family: 'steps', make: () => stepSamples(surface, wear, 0, RATE) })),
  ),
  ...SPECIES.map((sp, i): Job => {
    // (A typical bird of its species, as `catalog.ts` lists it: the middle of its range.)
    const voice = { species: sp.name, seed: i, pitch: (sp.pitch[0] + sp.pitch[1]) / 2, pace: 1, near: 0.575 };
    return { name: `bird-${sp.name}`, family: 'birds', make: () => songSamples(voice, 0, RATE) };
  }),
  ...ROOM_SIZES.map((size): Job => ({ name: `room-${size}`, family: 'rooms', make: () => roomResponse(size, RATE) })),
];

/** The numbers `--json` writes. */
interface Timing {
  rate: number;
  runs: number;
  sounds: { name: string; family: Family; ms: number }[];
  families: Record<Family, number>;
}

const sounds: Timing['sounds'] = [];
for (const job of jobs) {
  let best = Infinity;
  for (let run = 0; run < RUNS; run++) {
    const t0 = performance.now();
    job.make();
    best = Math.min(best, performance.now() - t0);
  }
  sounds.push({ name: job.name, family: job.family, ms: best });
}
const families = Object.fromEntries(FAMILIES.map((f) => [f, sounds.filter((s) => s.family === f).reduce((a, s) => a + s.ms, 0)])) as Record<Family, number>;
const timing: Timing = { rate: RATE, runs: RUNS, sounds, families };

const ms = (x: number | undefined) => (x === undefined ? '-' : x.toFixed(1)).padStart(8);
const ratio = (a: number | undefined, b: number) => (a === undefined || a <= 0 ? '' : `${(b / a).toFixed(2)}×`.padStart(7));
const compared = arg('--compare');
const before: Timing | undefined = compared ? JSON.parse(readFileSync(path(compared), 'utf8')) : undefined;
const was = new Map(before?.sounds.map((s) => [s.name, s.ms]));

console.log(`${sounds.length} sounds at ${RATE} Hz, the least of ${RUNS} run${RUNS > 1 ? 's' : ''} each${before ? ` (before: ${compared})` : ''}\n`);
console.log(before ? `${'sound'.padEnd(18)} ${'family'.padEnd(9)} ${'before'.padStart(8)} ${'ms'.padStart(8)}  ratio` : `${'sound'.padEnd(18)} ${'family'.padEnd(9)} ${'ms'.padStart(8)}`);
for (const s of sounds.toSorted((a, b) => b.ms - a.ms)) {
  console.log(before ? `${s.name.padEnd(18)} ${s.family.padEnd(9)} ${ms(was.get(s.name))} ${ms(s.ms)}  ${ratio(was.get(s.name), s.ms)}` : `${s.name.padEnd(18)} ${s.family.padEnd(9)} ${ms(s.ms)}`);
}
console.log('');
let total = 0;
let totalBefore = 0;
for (const f of FAMILIES) {
  total += families[f];
  totalBefore += before?.families[f] ?? 0;
  console.log(before ? `${f.padEnd(28)} ${ms(before.families[f])} ${ms(families[f])}  ${ratio(before.families[f], families[f])}` : `${f.padEnd(28)} ${ms(families[f])}`);
}
console.log(before ? `${'all'.padEnd(28)} ${ms(totalBefore)} ${ms(total)}  ${ratio(totalBefore, total)}` : `${'all'.padEnd(28)} ${ms(total)}`);

const json = arg('--json');
if (json) {
  mkdirSync(dirname(path(json)), { recursive: true });
  writeFileSync(path(json), `${JSON.stringify(timing, null, 2)}\n`);
  console.log(`\nwrote ${path(json)}`);
}
const maxMusic = arg('--max-music');
if (maxMusic !== undefined && families.music > Number(maxMusic)) {
  console.error(`\nthe music took ${families.music.toFixed(1)} ms, over ${maxMusic}`);
  process.exit(1);
}
