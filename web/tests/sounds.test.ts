// The world sounds made off the main thread (`audio/water.ts`, `audio/things.ts`): each one made at 48 kHz has
// two channels of the same length, no gaps or overflows, sound in it and a level the mix can take; a loop's end
// runs on into its start without a click. And every world sound key in the content (`sound`, `endSound`,
// `sounds`, `ambient`, `ambientWhile`, a recipe step's `sound`) is one `audio/world.ts` plays, or `none`
// (an unknown key is silent in the game, with only a console warning).
import { describe, expect, test } from 'bun:test';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { loudness } from '../src/audio/loudness';
import { MADE, ONCE_THINGS, type MadeKind } from '../src/audio/things';
import { ONCE } from '../src/audio/water';

const WEB = new URL('../', import.meta.url).pathname;
const RATE = 48000;
const once: readonly string[] = [...ONCE, ...ONCE_THINGS];

/** The level the game plays each sound at (`world.ts`'s `LOOPS` and `ONCE_LEVEL`; 1 where it names none). */
function levels(): Map<string, number> {
  const world = readFileSync(join(WEB, 'src/audio/world.ts'), 'utf8');
  const out = new Map<string, number>();
  const loops = world.slice(world.indexOf('const LOOPS'), world.indexOf('\n};', world.indexOf('const LOOPS')));
  for (const m of loops.matchAll(/^ {2}(\w+): \{([^}]*)\}/gm)) {
    const level = /level: ([\d.]+)/.exec(m[2]);
    if (level) out.set(m[1], Number(level[1]));
  }
  const one = /const ONCE_LEVEL[^=]*= \{([^}]*)\}/.exec(world);
  if (one) for (const m of one[1].matchAll(/(\w+): ([\d.]+)/g)) out.set(m[1], Number(m[2]));
  return out;
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
  // At the level the game plays it, the quietest sound is about -37 LUFS and the loudest about -20
  // (`bun run sounds` lists them). One far outside that is a level left off, or a sound made much
  // louder or quieter than it was, without the mix having been looked at.
  test('as the game plays them, the sounds are within the mix', () => {
    const level = levels();
    const kinds = Object.keys(MADE) as MadeKind[];
    expect(kinds.every((kind) => level.has(kind))).toBe(true);
    const off: string[] = [];
    for (const kind of kinds) {
      const { left, right } = MADE[kind](RATE);
      const gain = level.get(kind) ?? 1;
      const lufs = loudness(left.map((v) => v * gain), right.map((v) => v * gain), RATE);
      if (!(lufs > -40 && lufs < -18)) off.push(`${kind} ${lufs.toFixed(1)}`);
    }
    expect(off).toEqual([]);
  }, 120_000);
});

describe('world sounds', () => {
  for (const kind of Object.keys(MADE) as MadeKind[]) {
    test(kind, () => {
      const { left, right } = MADE[kind](RATE);
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
      // Rounded off above 0.7 (`setLevel`): never past full scale.
      expect(peak).toBeLessThanOrEqual(1);
      expect(Math.sqrt(sum / (2 * left.length))).toBeGreaterThan(0.004);
      if (!once.includes(kind)) {
        // The step from the last sample to the first is no bigger than the ones inside the loop.
        let step = 0;
        for (const ch of [left, right]) for (let i = 1; i < ch.length; i++) step = Math.max(step, Math.abs(ch[i] - ch[i - 1]));
        for (const ch of [left, right]) expect(Math.abs(ch[0] - ch[ch.length - 1])).toBeLessThanOrEqual(step);
      }
    }, 20_000);
  }

  test('content keys are sounds the world plays', () => {
    const world = readFileSync(join(WEB, 'src/audio/world.ts'), 'utf8');
    const block = world.slice(world.indexOf('const LOOPS'), world.indexOf('\n};', world.indexOf('const LOOPS')));
    const loops = [...block.matchAll(/^ {2}(\w+): \{/gm)].map((m) => m[1]);
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
