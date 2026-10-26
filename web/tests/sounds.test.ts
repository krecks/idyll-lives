// The world sounds made off the main thread (`audio/water.ts`, `audio/things.ts`): each one made at 48 kHz has
// two channels of the same length, no gaps or overflows, sound in it and a level the mix can take; a loop's end
// runs on into its start without a click. And every world sound key in the content (`sound`, `endSound`,
// `sounds`, `ambient`, `ambientWhile`, a recipe step's `sound`) is one `audio/world.ts` plays, or `none`
// (an unknown key is silent in the game, with only a console warning).
import { describe, expect, test } from 'bun:test';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { MADE, ONCE_THINGS, type MadeKind } from '../src/audio/things';
import { ONCE } from '../src/audio/water';

const WEB = new URL('../', import.meta.url).pathname;
const RATE = 48000;
const once: readonly string[] = [...ONCE, ...ONCE_THINGS];

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
