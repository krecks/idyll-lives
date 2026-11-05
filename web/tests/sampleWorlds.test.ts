// The sample worlds the generated-world checks run on (`tools/worlds/sample.ts`, `ci-local.sh
// generated`) come out the same every time for the same seeds, byte for byte (the households drawn
// from the world's seed too): a world that fails a check there can be made again and run alone.
import { expect, test } from 'bun:test';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const SAMPLE = new URL('../../tools/worlds/sample.ts', import.meta.url).pathname;

test('sampling the same worlds twice makes the same files', () => {
  const dir = mkdtempSync(join(tmpdir(), 'sample-worlds-'));
  try {
    const sample = (tag: string) => {
      const [out, big] = [join(dir, `${tag}.json`), join(dir, `${tag}-big.json`)];
      // (A cosy and a big world of one kind: the sizes' households and layouts both.)
      const run = Bun.spawnSync([process.execPath, SAMPLE, out, '1', '--sizes', 'cosy,big', '--kinds', 'small', '--big-out', big], { stdout: 'pipe', stderr: 'pipe' });
      expect(run.stderr.toString()).not.toContain('error');
      return [readFileSync(out, 'utf8'), readFileSync(big, 'utf8')];
    };
    const [first, second] = [sample('a'), sample('b')];
    expect(first[0].length).toBeGreaterThan(0);
    expect(first[1].length).toBeGreaterThan(0);
    expect(second[0] === first[0]).toBe(true);
    expect(second[1] === first[1]).toBe(true);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}, 120_000);
