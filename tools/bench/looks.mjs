#!/usr/bin/env bun
/**
 * The same pictures in the soft look and in classic PBR, side by side: one sheet per dev page (and per game
 * pose), to judge a change to shading or art in both looks at once. It runs `render.mjs --shots` once per look
 * (`?look=soft|classic`, dev server only) and tiles each pair into one PNG with the looks' names over them.
 *
 *   bun tools/bench/looks.mjs --port 5231 [options]
 *     --pages characters,characters-heads,…   the dev pages (`render.mjs` `PAGES`; default: the residents' sheets and the gallery's objects)
 *     --poses orbit-day-0,orbit-night-0        also the game's own pictures of the bench scene (`render.mjs` `POSES`; default none)
 *     --backends webgpu,webgl                  default webgpu
 *     --out <dir>                              default <repo>/bench-results/looks-<time>/: soft/, classic/ and pairs/
 *   bun tools/bench/looks.mjs --pair <dirA> <dirB> [--names before,after] --out <dir>
 *     tiles the pictures two folders share (before and after a change, two presets) the same way
 *
 * Needs what `render.mjs` needs (the wasm packages, Playwright).
 */

import { spawnSync } from 'node:child_process';
import { mkdirSync, readdirSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};

const DEFAULT_PAGES = 'characters,characters-side,characters-heads,characters-close,characters-hands,characters-feet,characters-skins,characters-hair,characters-tops,characters-builds,characters-game,characters-near,characters-faces,poses,poses-close,previews,gallery-objects,gallery-furniture,gallery-trees,gallery-nature';
const LABEL = 44;

/** Tiles the pictures `a` and `b` share, `a`'s on the left, under their names; returns how many. */
async function pair(a, b, names, out) {
  const sharp = (await import(createRequire(join(ROOT, 'web/package.json')).resolve('sharp'))).default;
  mkdirSync(out, { recursive: true });
  const inB = new Set(readdirSync(b));
  const shared = readdirSync(a).filter((f) => f.endsWith('.png') && inB.has(f)).sort();
  for (const name of shared) {
    const [x, y] = [sharp(join(a, name)), sharp(join(b, name))];
    const [mx, my] = await Promise.all([x.metadata(), y.metadata()]);
    const width = mx.width + my.width;
    const height = Math.max(mx.height, my.height) + LABEL;
    const text = (label, at) => `<text x="${at}" y="${LABEL - 13}" font-family="Helvetica, Arial, sans-serif" font-size="26" fill="#fff" text-anchor="middle">${label}</text>`;
    const labels = Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${LABEL}"><rect width="100%" height="100%" fill="#1b1d21"/>${text(names[0], mx.width / 2)}${text(names[1], mx.width + my.width / 2)}</svg>`);
    await sharp({ create: { width, height, channels: 3, background: '#1b1d21' } })
      .composite([
        { input: labels, left: 0, top: 0 },
        { input: await x.toBuffer(), left: 0, top: LABEL },
        { input: await y.toBuffer(), left: mx.width, top: LABEL },
      ])
      .png()
      .toFile(join(out, name));
  }
  return shared.length;
}

if (args.includes('--pair')) {
  const i = args.indexOf('--pair');
  const [a, b] = [args[i + 1], args[i + 2]];
  const out = flag('out', null);
  if (!a || !b || !out) {
    console.error('usage: looks.mjs --pair <dirA> <dirB> [--names before,after] --out <dir>');
    process.exit(2);
  }
  const n = await pair(a, b, flag('names', 'before,after').split(','), out);
  console.log(`${n} pairs in ${out}`);
  process.exit(0);
}

const stamp = new Date().toISOString().replace(/\.\d+Z$/, '').replace(/[:T]/g, '-');
const out = flag('out', join(ROOT, 'bench-results', `looks-${stamp}`));
const port = flag('port', '5231');
const backends = flag('backends', 'webgpu');
const pages = flag('pages', DEFAULT_PAGES);
const poses = flag('poses', '');
mkdirSync(out, { recursive: true });
const LOOKS = ['soft', 'classic'];
const jobs = LOOKS.flatMap((look) => {
  const common = ['--port', port, '--backends', backends, '--extra', `look=${look}`];
  return [
    ['--shots', join(out, look), '--pages', pages, ...common],
    ...(poses ? [['--shots', join(out, look), '--poses', poses, ...common]] : []),
  ];
});
const file = join(out, 'jobs.json');
writeFileSync(file, JSON.stringify(jobs, null, 1));
const run = spawnSync('bun', [join(ROOT, 'tools/bench/render.mjs'), '--jobs', file], { stdio: 'inherit' });
const n = await pair(join(out, LOOKS[0]), join(out, LOOKS[1]), LOOKS, join(out, 'pairs'));
console.log(`${n} pairs in ${join(out, 'pairs')}`);
process.exit(run.status ?? 1);
