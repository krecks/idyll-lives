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

/** A big world of one place, the countryside (every world is a big one now): the file's text. */
function sample(tag: string, more: string[] = []): string {
  const out = join(dir, `${tag}.json`);
  // (Not timed: beside the other tests on CI's runner a big world's layout took over twice its budget,
  // which `ci-local.sh generated` checks running alone.)
  // (The place at its own options alone: the extremes are `ci-local.sh generated`'s.)
  const run = Bun.spawnSync([process.execPath, SAMPLE, out, '1', '--kinds', 'country', '--no-extremes', '--no-timing', ...more], { stdout: 'pipe', stderr: 'pipe' });
  // (A sample that found a layout problem still writes its file, so the exit status is the check.)
  expect(run.exitCode).toBe(0);
  expect(run.stderr.toString()).not.toContain('error');
  return readFileSync(out, 'utf8');
}
const seeds = (text: string) => (JSON.parse(text) as { seed: number }[]).map((w) => w.seed);

test('sampling the same worlds twice makes the same files', () => {
  // (Twice: once by default, once at seed base 0, which is the default, CI's worlds: the first of a place seeded
  // 104729 and its name's length. Two runs, not three: each is a big world, about 20 s.)
  const [first, zero] = [sample('a'), sample('zero', ['--seed-base', '0'])];
  expect(first.length).toBeGreaterThan(0);
  expect(seeds(first)).toEqual([104729 + 'country'.length]);
  expect(zero === first).toBe(true);
}, 120_000);

test('another seed base samples other worlds, the same each time', () => {
  const [one, again] = [sample('one', ['--seed-base', '1']), sample('again', ['--seed-base', '1'])];
  expect(seeds(one)[0]).not.toBe(104729 + 'country'.length);
  expect(one === again).toBe(true);
}, 120_000);
