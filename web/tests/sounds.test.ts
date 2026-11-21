// Every sound the game makes (`audio/catalog.ts`: the water's and the things', the sounds of one channel, the
// birds' songs, the interface's blips; made in Rust, `crates/sound`, whose own tests hold each one as made), at
// 48 kHz as the game plays it: two channels of the same length, no gaps or overflows, sound in it and a level
// within its group's part of the mix; a loop's end runs on into its start without a click. Making them all is also
// the check that every name the player asks for (`audio/kinds.ts`) is one the synthesis makes. And every world
// sound key in the content (`sound`, `endSound`, `sounds`, `ambient`,
// `ambientWhile`, a recipe step's `sound`) is one `audio/world.ts` plays, or `none` (an unknown key is silent in
// the game, with only a console warning).
import { describe, expect, test } from 'bun:test';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import './support/synth';
import { everySound, GUARD, LOOPS, ONCE_KINDS, ONCE_LEVEL } from '../src/audio/catalog';
import { MADE, type Stereo } from '../src/audio/kinds';
import { kWeighting, loudness } from '../src/audio/loudness';

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

  test('at other sample rates the same weighting (the voices\' 24 kHz, a sound card\'s 44.1 kHz)', () => {
    for (const rate of [24000, 44100]) {
      const at = (amp: number) => Float32Array.from({ length: rate * 5 }, (_, i) => amp * Math.sin((2 * Math.PI * 997 * i) / rate));
      expect(Math.abs(loudness(at(1), new Float32Array(rate * 5), rate) + 3.01)).toBeLessThan(0.05);
      expect(Math.abs(loudness(at(10 ** (-23 / 20)), at(10 ** (-23 / 20)), rate) + 23)).toBeLessThan(0.05);
    }
    // The filters worked out for 48 kHz are the standard's own.
    const [shelf, high] = kWeighting(48_000);
    const [s2, h2] = kWeighting(47_999.999);
    for (let i = 0; i < 3; i++) expect(s2[0][i]).toBeCloseTo(shelf[0][i], 6);
    for (let i = 0; i < 2; i++) {
      expect(s2[1][i]).toBeCloseTo(shelf[1][i], 6);
      expect(h2[1][i]).toBeCloseTo(high[1][i], 6);
    }
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
    expect(listed.length).toBeGreaterThanOrEqual(85);
    for (const kind of MADE) expect(kind in LOOPS || kind in ONCE_LEVEL).toBe(true);
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

});

describe('world sounds', () => {
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
