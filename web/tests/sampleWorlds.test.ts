// The sample worlds the generated-world checks run on (`tools/worlds/sample.ts`, `ci-local.sh
// generated`) come out the same every time for the same seeds, byte for byte (the households drawn
// from the world's seed too): a world that fails a check there can be made again and run alone.
// Seed base 0 (the default) keeps the worlds CI has always checked; another draws other worlds.
import { afterAll, expect, test } from 'bun:test';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const SAMPLE = new URL('../../tools/worlds/sample.ts', import.meta.url).pathname;
const dir = mkdtempSync(join(tmpdir(), 'sample-worlds-'));
afterAll(() => rmSync(dir, { recursive: true, force: true }));

/** A cosy and a big world of one kind (the sizes' households and layouts both): the two files' text. */
function sample(tag: string, more: string[] = []): [string, string] {
  const [out, big] = [join(dir, `${tag}.json`), join(dir, `${tag}-big.json`)];
  const run = Bun.spawnSync([process.execPath, SAMPLE, out, '1', '--sizes', 'cosy,big', '--kinds', 'small', '--big-out', big, ...more], { stdout: 'pipe', stderr: 'pipe' });
  expect(run.stderr.toString()).not.toContain('error');
  return [readFileSync(out, 'utf8'), readFileSync(big, 'utf8')];
}
const seeds = (text: string) => (JSON.parse(text) as { seed: number }[]).map((w) => w.seed);

test('sampling the same worlds twice makes the same files', () => {
  const [first, second] = [sample('a'), sample('b')];
  expect(first[0].length).toBeGreaterThan(0);
  expect(first[1].length).toBeGreaterThan(0);
  expect(second[0] === first[0]).toBe(true);
  expect(second[1] === first[1]).toBe(true);
  // (Seed base 0 is the default, CI's worlds: the first of a kind seeded 104729 and its name's length.)
  const zero = sample('zero', ['--seed-base', '0']);
  expect(zero[0] === first[0] && zero[1] === first[1]).toBe(true);
  expect(seeds(first[0])).toEqual([104729 + 'small'.length]);
  expect(seeds(first[1])).toEqual([104729 + 'small'.length]);
}, 120_000);

test('another seed base samples other worlds, the same each time', () => {
  const [one, again] = [sample('one', ['--seed-base', '1']), sample('again', ['--seed-base', '1'])];
  expect(seeds(one[0])[0]).not.toBe(104729 + 'small'.length);
  expect(one[0] === again[0] && one[1] === again[1]).toBe(true);
}, 120_000);
