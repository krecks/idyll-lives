#!/usr/bin/env bun
/**
 * The pictures a change to the look is judged by: the bench scene from the play camera, by day (the sun behind
 * the camera and ahead of it), at dusk and by night, the same with the walls up, from 80 m, from across the
 * street, and the menus' town (`render.mjs` `--poses play`).
 *
 *   bun tools/bench/play.mjs --port 5241 --out <dir>
 *     the whole set on medium/webgpu and low/webgl (before a change and after it; pair the two folders with
 *     `looks.mjs --pair`)
 *   bun tools/bench/play.mjs --port 5241 --sheet <group> [--out <dir>] [--low] [--with 'look.vignette:0.8']
 *     one build in the variants of `play-variants.json`'s group, each a `?tune=` (dev server only, `web/src/render/tune.ts`),
 *     on the group's poses, and a sheet per pose with the variants side by side under their names (`<out>/sheets`).
 *     `--low` adds low/webgl; `--with` is a tune every variant starts from; `--poses a,b` instead of the group's.
 *
 * Needs what `render.mjs` needs (the wasm packages, Playwright).
 */

import { spawnSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { tile } from './sheet.mjs';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};

const stamp = new Date().toISOString().replace(/\.\d+Z$/, '').replace(/[:T]/g, '-');
const port = flag('port', '5241');
const group = flag('sheet', null);
const out = flag('out', join(ROOT, 'bench-results', `play-${group ?? 'set'}-${stamp}`));
const COMBOS = [['medium', 'webgpu'], ...(group && !args.includes('--low') ? [] : [['low', 'webgl']])];
const shots = (dir, poses, extra) => COMBOS.map(([preset, backend]) => ['--shots', dir, '--poses', poses, '--presets', preset, '--backends', backend, '--port', port, ...(extra ? ['--extra', extra] : [])]);

let jobs;
let variants = [];
if (group) {
  const groups = JSON.parse(readFileSync(join(ROOT, 'tools/bench/play-variants.json'), 'utf8'));
  const g = groups[group];
  if (!g) {
    console.error(`no group ${group} (${Object.keys(groups).join(', ')})`);
    process.exit(2);
  }
  const base = flag('with', '');
  variants = Object.entries(g.variants);
  jobs = variants.flatMap(([name, tune]) => shots(join(out, name), flag('poses', g.poses.join(',')), [base, tune].filter(Boolean).join(';') && `tune=${[base, tune].filter(Boolean).join(';')}`));
} else {
  jobs = shots(out, flag('poses', 'play'), '');
}
mkdirSync(out, { recursive: true });
const file = join(out, 'jobs.json');
writeFileSync(file, JSON.stringify(jobs, null, 1));
const run = spawnSync('bun', [join(ROOT, 'tools/bench/render.mjs'), '--jobs', file], { stdio: 'inherit' });
if (group) {
  const n = await tile(variants.map(([name]) => join(out, name)), variants.map(([name]) => name), join(out, 'sheets'));
  console.log(`${n} sheets in ${join(out, 'sheets')}`);
}
process.exit(run.status ?? 1);
