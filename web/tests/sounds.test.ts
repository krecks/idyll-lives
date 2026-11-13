// Every sound the game makes (`audio/catalog.ts`: the water's and the things', the sounds of one channel, the
// birds' songs, the interface's blips), made at 48 kHz as the game plays it: two channels of the same length, no
// gaps or overflows, sound in it and a level within its group's part of the mix; a loop's end runs on into its
// start without a click. The ones made off the main thread (`audio/water.ts`, `audio/things.ts`) also as made,
// before their level. And every world sound key in the content (`sound`, `endSound`, `sounds`, `ambient`,
// `ambientWhile`, a recipe step's `sound`) is one `audio/world.ts` plays, or `none` (an unknown key is silent in
// the game, with only a console warning).
import { describe, expect, test } from 'bun:test';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { BLIPS, blipSamples, type Blip } from '../src/audio/blips';
import { everySound, GUARD, LOOPS, ONCE_KINDS, ONCE_LEVEL } from '../src/audio/catalog';
import { loudness } from '../src/audio/loudness';
import { MONO, nodeFilter, sweep } from '../src/audio/mono';
import { MADE, type MadeKind } from '../src/audio/things';
import type { Stereo } from '../src/audio/water';

const WEB = new URL('../', import.meta.url).pathname;
const RATE = 48000;
const once = ONCE_KINDS;

/** Each sound as the game plays it, made once for all the tests here. */
const heard = new Map<string, Stereo>();
const listed = everySound();
const hear = (name: string): Stereo => {
  let s = heard.get(name);
  if (!s) {
    s = listed.find((l) => l.name === name)!.heard(RATE);
    heard.set(name, s);
  }
  return s;
};

/** The biggest step between neighbouring samples of a channel, and the step from its last sample round to its first. */
function steps(ch: Float32Array): { inside: number; seam: number } {
  let inside = 0;
  for (let i = 1; i < ch.length; i++) inside = Math.max(inside, Math.abs(ch[i] - ch[i - 1]));
  return { inside, seam: Math.abs(ch[0] - ch[ch.length - 1]) };
}

describe('loudness', () => {
  const tone = (amp: number) => Float32Array.from({ length: RATE * 5 }, (_, i) => amp * Math.sin((2 * Math.PI * 997 * i) / RATE));

  test("the standard's reference tones read as it says (ITU-R BS.1770)", () => {
    // 997 Hz at full scale in one channel is -3.01 LUFS; at -23 dBFS in both, -23.0.
    expect(loudness(tone(1), new Float32Array(RATE * 5), RATE)).toBeCloseTo(-3.01, 2);
    expect(loudness(tone(10 ** (-23 / 20)), tone(10 ** (-23 / 20)), RATE)).toBeCloseTo(-23, 2);
  });

  test('silence has none, and a pause in a sound does not make it quieter', () => {
    expect(loudness(new Float32Array(RATE), new Float32Array(RATE), RATE)).toBe(-Infinity);
    const full = tone(0.1);
    const half = Float32Array.from(full, (v, i) => (i < full.length / 2 ? v : 0));
    expect(loudness(half, half, RATE)).toBeCloseTo(loudness(full, full, RATE), 0);
  });
});

describe('the mix', () => {
  // At the level the game plays it, a thing in use is between about -35 LUFS and -20, the outdoors and the
  // interface's blips in ranges of their own (`GUARD`; `bun run sounds` lists them). One far outside is a level
  // left off, or a sound made much louder or quieter than it was, without the mix having been looked at.
  test('as the game plays them, the sounds are within the mix', () => {
    expect(listed.length).toBeGreaterThanOrEqual(70);
    for (const kind of Object.keys(MADE)) expect(kind in LOOPS || kind in ONCE_LEVEL).toBe(true);
    const off: string[] = [];
    for (const { name, group } of listed) {
      const { left, right } = hear(name);
      const lufs = loudness(left, right, RATE);
      const [low, high] = GUARD[group];
      if (!(lufs > low && lufs < high)) off.push(`${name} ${lufs.toFixed(1)} (${group}: ${low} to ${high})`);
    }
    expect(off).toEqual([]);
  }, 120_000);

  test('every sound as the game plays it: whole, under full scale, and a loop without a click at its seam', () => {
    const off: string[] = [];
    for (const { name, plays } of listed) {
      const { left, right } = hear(name);
      let peak = 0;
      let sum = 0;
      for (const ch of [left, right]) {
        for (const v of ch) {
          peak = Math.max(peak, Math.abs(v));
          sum += v * v;
        }
      }
      // (A NaN or an infinity anywhere carries into the sum.)
      if (left.length !== right.length || !left.length || !Number.isFinite(sum) || !(sum > 0)) off.push(`${name}: not whole`);
      if (peak > 1) off.push(`${name}: peak ${peak.toFixed(2)}`);
      if (plays !== 'loop') continue;
      for (const ch of [left, right]) {
        const { inside, seam } = steps(ch);
        if (seam > inside) off.push(`${name}: a step of ${seam.toFixed(4)} at its seam, ${inside.toFixed(4)} at most inside`);
      }
    }
    expect(off).toEqual([]);
  }, 120_000);

  test('the sounds of one channel and the blips are the same every time', () => {
    for (const kind of Object.keys(MONO) as (keyof typeof MONO)[]) expect(MONO[kind](RATE)).toEqual(MONO[kind](RATE));
    for (const blip of Object.keys(BLIPS) as Blip[]) expect(blipSamples(blip, RATE)).toEqual(blipSamples(blip, RATE));
  });
});

describe('the parts the sounds of one channel are made from', () => {
  test('a filter as the browser\'s node computes it: a low-pass passes the lows, stops the highs, and a loop comes out a loop', () => {
    const tone = (hz: number) => Float32Array.from({ length: RATE }, (_, i) => Math.sin((2 * Math.PI * hz * i) / RATE));
    const rms = (d: Float32Array) => Math.sqrt(d.reduce((s, v) => s + v * v, 0) / d.length);
    const low = nodeFilter(tone(100), RATE, 'lowpass', 1000, 0, true);
    const high = nodeFilter(tone(8000), RATE, 'lowpass', 1000, 0, true);
    expect(rms(low)).toBeCloseTo(Math.SQRT1_2, 2);
    // Two poles: 12 dB an octave, three octaves up.
    expect(20 * Math.log10(rms(high) / Math.SQRT1_2)).toBeLessThan(-34);
    // At its cutoff a low-pass is as far up as its `Q` says in dB (the node's way; 0: level with the lows); a band-pass passes its centre whole.
    expect(20 * Math.log10(rms(nodeFilter(tone(1000), RATE, 'lowpass', 1000, 0, true)) / Math.SQRT1_2)).toBeCloseTo(0, 2);
    expect(20 * Math.log10(rms(nodeFilter(tone(1000), RATE, 'highpass', 1000, 6, true)) / Math.SQRT1_2)).toBeCloseTo(6, 2);
    expect(rms(nodeFilter(tone(1000), RATE, 'bandpass', 1000, 1, true))).toBeCloseTo(Math.SQRT1_2, 3);
    // Whole periods in a second: the loop's last sample runs on into its first.
    const { inside, seam } = steps(low);
    expect(seam).toBeLessThanOrEqual(inside);
  });

  test('a sweep moves by a constant ratio and holds its last value', () => {
    const f = sweep([220, 95], [0, 0.08]);
    expect(f(0)).toBe(220);
    expect(f(0.04)).toBeCloseTo(Math.sqrt(220 * 95), 9);
    expect(f(0.08)).toBe(95);
    expect(f(1)).toBe(95);
  });

  test('the knock is three knocks', () => {
    const d = MONO.knock(RATE);
    // Loud within 20 ms of each knock's start, quiet just before it.
    const level = (from: number, to: number) => Math.max(...d.subarray(Math.round(from * RATE), Math.round(to * RATE)).map(Math.abs));
    for (const at of [0.02, 0.19, 0.38]) {
      expect(level(at, at + 0.02)).toBeGreaterThan(0.3);
      expect(level(at - 0.015, at - 0.001)).toBeLessThan(0.05);
    }
  });
});

describe('world sounds', () => {
  // The ones made off the main thread, as made (before the game's level): long enough, never past full scale
  // (rounded off above 0.7: `setLevel`), with sound in them, and a loop's end running on into its start.
  for (const kind of Object.keys(MADE) as MadeKind[]) {
    test(kind, () => {
      const { left, right } = hear(kind);
      const level = listed.find((l) => l.name === kind)!.level;
      expect(left.length).toBe(right.length);
      expect(left.length).toBeGreaterThan(RATE / 2);
      let peak = 0;
      let sum = 0;
      for (const ch of [left, right]) {
        for (const v of ch) {
          peak = Math.max(peak, Math.abs(v));
          sum += v * v;
        }
      }
      // (A NaN or an infinity anywhere carries into the sum.)
      expect(Number.isFinite(sum)).toBe(true);
      expect(peak / level).toBeLessThanOrEqual(1 + 1e-6);
      expect(Math.sqrt(sum / (2 * left.length)) / level).toBeGreaterThan(0.004);
      if (!once.includes(kind)) {
        // The step from the last sample to the first is no bigger than the ones inside the loop.
        for (const ch of [left, right]) {
          const { inside, seam } = steps(ch);
          expect(seam).toBeLessThanOrEqual(inside);
        }
      }
    }, 20_000);
  }

  test('content keys are sounds the world plays', () => {
    const loops = Object.keys(LOOPS);
    expect(loops.length).toBeGreaterThan(30);
    const known = new Set([...loops, ...once, 'none']);
    const unknown: string[] = [];
    const visit = (v: unknown, path: string): void => {
      if (Array.isArray(v)) v.forEach((x, i) => visit(x, `${path}[${i}]`));
      else if (v && typeof v === 'object') {
        for (const [k, x] of Object.entries(v)) {
          if ((k === 'sound' || k === 'endSound' || k === 'ambient' || k === 'ambientWhile') && typeof x === 'string' && !known.has(x)) unknown.push(`${path}.${k}: ${x}`);
          else if (k === 'sounds' && x && typeof x === 'object' && !Array.isArray(x)) for (const [tag, s] of Object.entries(x)) if (typeof s === 'string' && !known.has(s)) unknown.push(`${path}.sounds.${tag}: ${s}`);
          visit(x, `${path}.${k}`);
        }
      }
    };
    const walk = (dir: string): string[] => readdirSync(dir).flatMap((f) => (statSync(join(dir, f)).isDirectory() ? walk(join(dir, f)) : f.endsWith('.json') ? [join(dir, f)] : []));
    const files = walk(join(WEB, 'public/content'));
    expect(files.length).toBeGreaterThan(10);
    for (const file of files) visit(JSON.parse(readFileSync(file, 'utf8')), file.slice(file.indexOf('content/')));
    expect(unknown).toEqual([]);
  });
});
