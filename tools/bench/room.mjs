#!/usr/bin/env bun
/**
 * The furnished living room from the play camera: the pictures a change to a room's look is judged by. The
 * `review-stage` state (README: States for checks) furnished as a living room (ROOM), shot from the play camera at
 * 9.5 m and at 6 m (as near as it goes), by day (11:00) and at dusk (19:30, the lamps on), on medium/webgpu and
 * low/webgl.
 *
 *   bun tools/bench/room.mjs --port 5490 --sheet <group> [--out <dir>]
 *     the room in the variants of `play-variants.json`'s group, each a `?tune=` (dev server only), and a sheet per
 *     picture with the variants side by side under their names (`<out>/sheets`), as `play.mjs --sheet` does for
 *     the bench scene (the group's own poses are that tool's: the guard that the look is no worse out of doors).
 *     `--only a,b`: these variants; `--with <tune>`: a tune every variant starts from; a variant whose folder
 *     holds pictures is not shot again (`--again` shoots all).
 *   bun tools/bench/room.mjs --port 5490 --out <dir> [--tune 'contact.radius:0.5']
 *     the room once, as it is (or in one tune).
 *   --hours 11,19.5   --radii 9.5,6   --combos medium/webgpu,low/webgl   --at <n> pages at once (default 3)
 *   --clip x,y,w,h    the part of the 1920 × 1080 picture kept (default 280,40,1480,940: the furniture, so that
 *                     three variants side by side are still the play camera's own pixels)
 *   --gpu <secs>      instead of pictures: the GPU's time per frame (`?gpustats`), the 9.5 m pose held for <secs>,
 *                     one page at a time, each variant in turn and the round repeated `--runs` times (default 3);
 *                     medians to `<out>/gpu.json` and the console
 *   --floor <id>      the room's floor laid in a floor covering first (`wood.honey`, `tile.white`: content's `floorCoverings`)
 *   --nudge <m>       the camera's target moved this far along x and z (0.004: a quarter of a pixel at 9.5 m; the same
 *                     pictures taken so and `cargo shots steady <dirA> <dirB>` say what shimmers as the camera moves)
 *   --url <base>      a dev server already running (else its own Vite on --port, --strictPort)
 *
 * Needs what `render.mjs` needs (the wasm packages, Playwright) and `debug-states/review-stage.json`.
 */

import { spawn } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { quiet } from './quiet.mjs';
import { tile } from './sheet.mjs';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const WEB = join(ROOT, 'web/');
const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};

/**
 * The living room: what is bought, where (`[x, z, turn]`, the state's tiles; then centimetres off the tile's middle)
 * and in this order (a thing on a top after the top). The room is the state's, (4, 6) to (16, 18), its windows in the left wall.
 */
const ROOM = [
  ['rugLarge', [6, 14, 0]],
  ['sofa', [7, 17, 2]],
  ['lamp', [5, 17, 2]],
  ['sideTable', [9, 17, 2]],
  ['lampTableRound', [9, 17, 2]],
  ['bookshelf', [10, 17, 2]],
  ['plant', [11, 17, 2]],
  ['armchair', [4, 13, 1]],
  ['coffeeTable', [7, 15, 0]],
  ['mugs', [7, 15, 0], [30, 0]],
  ['tvStandLow', [10, 13, 3]],
];
/** The play camera over the room: from the front right, looking at the sofa's corner. */
const CAMERA = { alpha: -0.95, beta: 0.86, target: [8, 0, 15] };
const HOURS = { 11: 'day', 16: 'afternoon', 19.5: 'dusk', 22: 'night' };

const stamp = new Date().toISOString().replace(/\.\d+Z$/, '').replace(/[:T]/g, '-');
const group = flag('sheet', null);
const out = flag('out', join(ROOT, 'bench-results', `room-${group ?? 'set'}-${stamp}`));
const port = Number(flag('port', '5242'));
const hours = flag('hours', '11,19.5').split(',').map(Number);
const radii = flag('radii', '9.5,6').split(',').map(Number);
const combos = flag('combos', 'medium/webgpu,low/webgl').split(',').map((c) => c.split('/'));
const clip = flag('clip', '280,40,1480,940').split(',').map(Number);
const gpuSecs = Number(flag('gpu', '0'));
const atOnce = gpuSecs ? 1 : Number(flag('at', '3'));
const nudge = Number(flag('nudge', '0'));
/** The floor covering to lay (`--floor`): its number in content's list, from 1. */
const floor = flag('floor', null);
const covering = floor ? JSON.parse(readFileSync(join(WEB, 'public/content/base.json'), 'utf8')).floorCoverings.findIndex((c) => c.id === floor) + 1 : 0;
if (floor && !covering) {
  console.error(`no floor covering ${floor}`);
  process.exit(2);
}

const dirOf = (name) => join(out, name.replace(/[^\w.-]+/g, '-').replace(/^-|-$/g, ''));
let variants;
if (group) {
  const groups = JSON.parse(readFileSync(join(ROOT, 'tools/bench/play-variants.json'), 'utf8'));
  if (!groups[group]) {
    console.error(`no group ${group} (${Object.keys(groups).join(', ')})`);
    process.exit(2);
  }
  const only = flag('only', null)?.split(',');
  const base = flag('with', '');
  variants = Object.entries(groups[group].variants)
    .filter(([name]) => !only || only.some((o) => name.startsWith(o)))
    .map(([name, tune]) => ({ name, tune: [base, tune].filter(Boolean).join(';'), dir: dirOf(name) }));
} else {
  variants = [{ name: 'room', tune: flag('tune', ''), dir: out }];
}
mkdirSync(out, { recursive: true });

let vite = null;
let base = flag('url', null);
if (!base) {
  vite = spawn(`${WEB}node_modules/.bin/vite`, ['--port', String(port), '--strictPort'], { cwd: WEB, stdio: ['ignore', 'pipe', 'inherit'] });
  process.on('exit', () => vite.kill());
  await new Promise((resolve, reject) => {
    vite.stdout.on('data', (d) => String(d).includes('Local') && resolve());
    vite.on('exit', (code) => reject(new Error(`vite exited (${code}): is the port free?`)));
  });
  base = `http://localhost:${port}`;
}

const require = createRequire(import.meta.url);
const { chromium } = process.env.PLAYWRIGHT ? await import(process.env.PLAYWRIGHT) : require('playwright');
const browser = await chromium.launch({
  headless: true,
  ...(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chromium' }),
  args: ['--enable-unsafe-webgpu', '--enable-webgpu-developer-features', '--disable-backgrounding-occluded-windows', '--disable-renderer-backgrounding', '--disable-background-timer-throttling'],
});

/** The room's page in one variant, preset, backend and hour: furnished, paused, nobody in it. */
async function open(tune, preset, backend, hour, more = '') {
  const context = await browser.newContext({ viewport: { width: 1920, height: 1080 }, deviceScaleFactor: 1 });
  await context.addInitScript(quiet);
  await context.addInitScript(() => {
    const key = 'open-sims-wasm.settings';
    localStorage.setItem(key, JSON.stringify({ ...JSON.parse(localStorage.getItem(key) ?? '{}'), directorDelay: 0, autonomy: false }));
    window.__loadingNow = () => !!document.querySelector('.loading[role=progressbar]') || ['Moving in', 'Drawing portraits', 'Building the world', 'Warming up'].some((l) => (document.body?.innerText ?? '').includes(l));
  });
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', (e) => errors.push(String(e)));
  page.on('console', (m) => m.text().startsWith('[tune]') && errors.push(m.text()));
  const query = ['state=review-stage', 'debug', 'hud=0', `hour=${hour}`, `quality=${preset}`, backend === 'webgl' ? 'renderer=webgl' : '', 'res=1', tune ? `tune=${encodeURIComponent(tune)}` : '', more].filter(Boolean).join('&');
  await page.goto(`${base}/?${query}`, { timeout: 120_000 });
  await page.waitForFunction(() => typeof window.__sims === 'function' && window.__world?.() && window.__sims().length > 0, null, { timeout: 120_000 });
  const settled = () => page.waitForFunction(() => !window.__loadingNow(), null, { timeout: 120_000, polling: 250 });
  await settled();
  await page.waitForTimeout(1000);
  await page.evaluate(() => {
    window.__send({ type: 'setAutonomy', enabled: false, household: 0 });
    window.__addFunds(1_000_000);
  });
  for (const [def, at, offset] of ROOM) {
    const before = await page.evaluate(() => window.__world().objects.map((o) => o.id));
    await page.evaluate(({ def, at, offset }) => window.__send({ type: 'buy', household: 0, object: def, at, ...(offset ? { offset } : {}) }), { def, at, offset });
    const id = await page
      .waitForFunction(({ def, before }) => window.__world().objects.find((o) => o.def === def && !before.includes(o.id))?.id, { def, before }, { timeout: 10_000 })
      .then((h) => h.jsonValue())
      .catch(() => null);
    if (id === null) errors.push(`not bought: ${def} at ${at} (${await page.evaluate(() => [...document.querySelectorAll('.toast')].map((e) => e.textContent.trim()).join(' | '))})`);
  }
  if (covering) {
    // (The state's room: (4, 6) to (16, 18).)
    const tiles = [];
    for (let z = 6; z < 18; z++) for (let x = 4; x < 16; x++) tiles.push({ x, z, covering });
    await page.evaluate((tiles) => window.__send({ type: 'paintFloor', household: 0, tiles }), tiles);
  }
  // A moment of play (the lamps come on, the things settle), then the picture held still.
  await page.evaluate(() => window.__send({ type: 'setSpeed', speed: 1 }));
  await page.waitForTimeout(1500);
  await page.evaluate(() => window.__send({ type: 'setSpeed', speed: 0 }));
  const look = async (radius) => {
    await page.evaluate((pose) => {
      window.__controls.follow(null);
      window.__camera(pose);
    }, { ...CAMERA, target: [CAMERA.target[0] + nudge, CAMERA.target[1], CAMERA.target[2] + nudge], radius });
    await page.waitForTimeout(600);
    await settled();
    await page.waitForTimeout(2200);
  };
  return { page, look, errors, done: () => context.close() };
}

const median = (xs) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)];

if (gpuSecs) {
  // GPU time: one page at a time, the variants in turn, the round repeated (interleaved: the machine's drift is every variant's).
  const runs = Number(flag('runs', '3'));
  const rows = [];
  const times = new Map();
  for (let run = 0; run < runs; run++) {
    for (const [preset, backend] of combos) {
      for (const hour of hours) {
        for (const v of variants) {
          const key = `${preset}/${backend} ${HOURS[hour] ?? hour} · ${v.name}`;
          try {
            const { page, look, done } = await open(v.tune, preset, backend, hour, 'gpustats');
            await look(radii[0]);
            const samples = [];
            const until = Date.now() + gpuSecs * 1000;
            while (Date.now() < until) {
              await page.waitForTimeout(250);
              const ms = await page.evaluate(() => window.__gameStats?.().gpuMs ?? null);
              if (ms !== null) samples.push(ms);
            }
            await done();
            const ms = samples.length ? median(samples) : null;
            times.set(key, [...(times.get(key) ?? []), ms]);
            console.log(`${key}: ${ms === null ? 'n/a' : ms.toFixed(2)} ms (${samples.length} samples)`);
          } catch (err) {
            console.log(`${key}: failed: ${err}`);
          }
        }
      }
    }
  }
  for (const [key, ms] of times) {
    const got = ms.filter((x) => x !== null);
    rows.push({ key, runs: ms, median: got.length ? median(got) : null });
  }
  writeFileSync(join(out, 'gpu.json'), JSON.stringify(rows, null, 1));
  console.log(rows.map((r) => `${r.key}: ${r.median === null ? 'n/a' : r.median.toFixed(2)} ms (${r.runs.map((x) => (x === null ? 'n/a' : x.toFixed(2))).join(', ')})`).join('\n'));
} else {
  const shot = (dir) => !args.includes('--again') && group && existsSync(dir) && readdirSync(dir).some((f) => f.endsWith('.png'));
  const jobs = variants.filter((v) => !shot(v.dir)).flatMap((v) => combos.flatMap(([preset, backend]) => hours.map((hour) => ({ v, preset, backend, hour }))));
  let failed = 0;
  const work = async () => {
    for (let job = jobs.shift(); job; job = jobs.shift()) {
      const { v, preset, backend, hour } = job;
      const what = `${v.name} ${preset}/${backend} ${HOURS[hour] ?? hour}`;
      try {
        mkdirSync(v.dir, { recursive: true });
        const { page, look, errors, done } = await open(v.tune, preset, backend, hour);
        for (const radius of radii) {
          await look(radius);
          await page.screenshot({ path: join(v.dir, `${preset}-${backend}-room-${HOURS[hour] ?? hour}-${Math.round(radius * 100)}.png`), clip: { x: clip[0], y: clip[1], width: clip[2], height: clip[3] } });
        }
        await done();
        console.log(`${what}: ok${errors.length ? ` (${errors.join('; ')})` : ''}`);
      } catch (err) {
        failed++;
        console.log(`${what}: failed: ${err}`);
      }
    }
  };
  await Promise.all(Array.from({ length: atOnce }, work));
  if (group) {
    const n = await tile(variants.map((v) => v.dir), variants.map((v) => v.name), join(out, 'sheets'));
    console.log(`${n} sheets in ${join(out, 'sheets')}`);
  }
  if (failed) process.exitCode = 1;
}
await browser.close();
vite?.kill();
process.exit(process.exitCode ?? 0);
