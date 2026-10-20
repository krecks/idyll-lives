#!/usr/bin/env node
/**
 * The rendering benchmark (`web/src/debug/bench.ts`) over a matrix: starts Vite, opens
 * `?bench=<scene>` in Chromium on the real GPU (WebGPU, and WebGL2 with `&renderer=webgl`)
 * at 1920×1080, device pixel ratio 1, once per preset × backend × scenario × CPU slow-down ×
 * diagnostic, and writes every result as JSON and a markdown table (medians when `--runs` > 1).
 *
 *   node tools/bench/render.mjs [options]
 *     --port 5221              Vite's port (it starts its own; `--url http://localhost:5221` uses a running one)
 *     --preview                the production build (`vite preview` of web/dist: compressed textures and
 *                              models, minified code; what players get) instead of the dev server; `--build` builds it first
 *     --scene village          web/public/bench/<scene>.json(.gz)
 *     --presets low,medium,high
 *     --backends webgpu,webgl
 *     --scenarios orbit-day,orbit-night,dusk,build,wide   (also mirror-day, mirror-night, mirror-still:
 *                              the home lot's mirror in view, the camera swinging before it or held)
 *     --throttle 1,3           CPU slow-down factors (CDP Emulation.setCPUThrottlingRate)
 *     --diags none,noshadows   one run each ('+' joins several: noshadows+noaa)
 *     --res 1                  render scale: 1, 0.5–1, or auto
 *     --secs 20 --warmup 5 --runs 1
 *     --throughput             no vsync and no frame-rate limit: frames as fast as they can be made
 *     --hud 0|1                the game's interface during the run (default 0: measure the renderer;
 *                              1 measures the interface too, which Build mode's catalog dominates)
 *     --extra 'k=v&k=v'        more URL flags for every run
 *     --out <dir>              default <repo>/bench-results/<time>/
 *   --shots <dir>              fixed poses, paused (`diag=freeze`, `hud=0`): one PNG per preset × backend × pose
 *   --compare <dirA> <dirB>    pixel differences between two shot folders (and `--out` for the difference images)
 *   --headless / --headed      without or with a window (see below)
 *
 * Needs `pnpm wasm` first, and Playwright: `PLAYWRIGHT=<path to playwright/index.mjs>` if it isn't
 * installed, `CHROME=<executable>` for a browser of your own.
 *
 * Measurements run in windows (far off to the right, `--left`): their timing is the real
 * display's. Shots, which don't time anything, run headless: no window, nothing takes the focus
 * from the app in front, and still the real GPU (Chromium's full build in its new headless mode;
 * not `chrome-headless-shell`, which draws in software, so a `CHROME` must be Chrome or Chrome for
 * Testing). `--headless` and `--headed` choose for either. A window's shots come out about 2 of
 * 255 brighter (the screen's colour profile), so compare shots taken the same way.
 */

import { spawn } from 'node:child_process';
import { mkdirSync, readdirSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { quiet } from './quiet.mjs';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const WEB = join(ROOT, 'web/');

const SCENARIOS = {
  'orbit-day': 'cam=orbit&hour=13',
  'orbit-night': 'cam=orbit&hour=21',
  dusk: 'cam=dusk',
  build: 'cam=build&hour=13',
  wide: 'cam=wide&hour=13',
  // The home lot's mirror on screen all the time (not in the default list: `--scenarios mirror-day,…`).
  'mirror-day': 'cam=mirror&hour=13',
  'mirror-night': 'cam=mirror&hour=21',
  'mirror-still': 'cam=mirror-still&hour=13',
};
/** Screenshot poses: along the paths (seconds), by day and by night. */
const POSES = {
  'orbit-day-0': 'cam=orbit&hour=13&at=0',
  'orbit-day-10': 'cam=orbit&hour=13&at=10',
  'orbit-day-20': 'cam=orbit&hour=13&at=20',
  'orbit-day-30': 'cam=orbit&hour=13&at=30',
  'orbit-night-0': 'cam=orbit&hour=21&at=0',
  'orbit-night-20': 'cam=orbit&hour=21&at=20',
  'wide-day-0': 'cam=wide&hour=13&at=0',
  'wide-night-45': 'cam=wide&hour=21&at=45',
  'build-day-0': 'cam=build&hour=13&at=0',
};

const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};
const has = (name) => args.includes(`--${name}`);
const list = (name, fallback) => flag(name, fallback).split(',').map((s) => s.trim()).filter(Boolean);

if (has('compare')) {
  const i = args.indexOf('--compare');
  await compare(args[i + 1], args[i + 2], flag('out', null));
  process.exit(0);
}

const port = Number(flag('port', '5221'));
const base = flag('url', null) ?? `http://localhost:${port}`;
const scene = flag('scene', 'village');
const presets = list('presets', has('shots') ? 'medium' : 'low,medium,high');
const backends = list('backends', has('shots') ? 'webgpu' : 'webgpu,webgl');
const scenarios = list('scenarios', 'orbit-day,orbit-night,dusk,build,wide');
const throttles = list('throttle', '1').map(Number);
const diags = list('diags', 'none');
const res = flag('res', '1');
const secs = Number(flag('secs', '20'));
const warmup = Number(flag('warmup', has('shots') ? '6' : '5'));
const runs = Number(flag('runs', '1'));
const extra = flag('extra', '');
const throughput = has('throughput');
const hud = flag('hud', '0');
const headed = has('headed') || (!has('shots') && !has('headless'));
const left = Number(flag('left', '2600'));
const stamp = new Date().toISOString().replace(/\.\d+Z$/, '').replace(/[:T]/g, '-');
const out = flag('out', has('shots') ? flag('shots') : join(ROOT, 'bench-results', stamp));
mkdirSync(out, { recursive: true });

async function playwright() {
  if (process.env.PLAYWRIGHT) return import(process.env.PLAYWRIGHT);
  try {
    return await import(createRequire(WEB + 'package.json').resolve('playwright'));
  } catch {
    throw new Error('Playwright not found: install it, or set PLAYWRIGHT=<path to playwright/index.mjs>');
  }
}

let vite = null;
if (has('build')) {
  const build = spawn(`${WEB}node_modules/.bin/vite`, ['build'], { cwd: WEB, stdio: ['ignore', 'ignore', 'inherit'] });
  await new Promise((resolve, reject) => build.on('exit', (code) => (code ? reject(new Error(`vite build failed (${code})`)) : resolve())));
}
if (!flag('url', null)) {
  vite = spawn(`${WEB}node_modules/.bin/vite`, [...(has('preview') ? ['preview'] : []), '--port', String(port), '--strictPort'], { cwd: WEB, stdio: ['ignore', 'pipe', 'inherit'] });
  process.on('exit', () => vite.kill());
  await new Promise((resolve, reject) => {
    vite.stdout.on('data', (d) => String(d).includes('Local') && resolve());
    vite.on('exit', (code) => reject(new Error(`vite exited (${code})`)));
  });
}

const { chromium } = await playwright();
const browser = await chromium.launch({
  headless: !headed,
  // (Without one of its own: Playwright's full Chromium, which has the GPU headless too.)
  ...(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chromium' }),
  args: [
    '--enable-unsafe-webgpu',
    // Full-precision GPU timestamps (otherwise rounded to 0.1 ms).
    '--enable-webgpu-developer-features',
    // Keep drawing at full rate in a window nobody looks at.
    '--disable-backgrounding-occluded-windows',
    '--disable-renderer-backgrounding',
    '--disable-background-timer-throttling',
    ...(throughput ? ['--disable-gpu-vsync', '--disable-frame-rate-limit'] : []),
  ],
});

/** Opens one benchmark page and waits for `key` on window (its result, or readiness for a screenshot). */
async function open(query, throttle, key, timeoutMs) {
  const context = await browser.newContext({ viewport: { width: 1920, height: 1080 }, deviceScaleFactor: 1 });
  await context.addInitScript(quiet);
  const page = await context.newPage();
  const cdp = await context.newCDPSession(page);
  if (headed) {
    try {
      const { windowId } = await cdp.send('Browser.getWindowForTarget');
      await cdp.send('Browser.setWindowBounds', { windowId, bounds: { left, top: 0 } });
    } catch {
      // (Some window managers refuse; it still runs.)
    }
  }
  if (throttle > 1) await cdp.send('Emulation.setCPUThrottlingRate', { rate: throttle });
  page.on('pageerror', (e) => console.error(`  page error: ${e.message}`));
  page.on('console', (m) => m.type() === 'error' && console.error(`  page: ${m.text().slice(0, 300)}`));
  await page.goto(`${base}/?${query}`);
  await page.waitForFunction((k) => !!window[k], key, { timeout: timeoutMs, polling: 250 });
  return { context, page };
}

const queryOf = (preset, backend, more) =>
  [`bench=${scene}`, `quality=${preset}`, backend === 'webgl' ? 'renderer=webgl' : '', `res=${res}`, more, extra].filter(Boolean).join('&');

try {
  if (has('shots')) await shots();
  else await matrix();
} finally {
  await browser.close();
  vite?.kill();
}
process.exit(0);

async function matrix() {
  const results = [];
  const total = runs * backends.length * presets.length * throttles.length * scenarios.length * diags.length;
  let n = 0;
  for (let run = 0; run < runs; run++) {
    for (const backend of backends) {
      for (const preset of presets) {
        for (const throttle of throttles) {
          for (const scenario of scenarios) {
            for (const diag of diags) {
              const d = diag === 'none' ? '' : diag.replaceAll('+', ',');
              const label = [preset, backend, scenario, `cpu${throttle}`, diag, throughput ? 'fast' : 'vsync'].join('-');
              const query = queryOf(preset, backend, [SCENARIOS[scenario] ?? scenario, d && `diag=${d}`, `secs=${secs}`, `warmup=${warmup}`, `hud=${hud}`, `label=${label}`].filter(Boolean).join('&'));
              n++;
              process.stdout.write(`[${n}/${total}] ${label} … `);
              let result;
              try {
                const { context, page } = await open(query, throttle, '__benchResult', (warmup + secs) * 1000 + 240_000);
                result = await page.evaluate(() => window.__benchResult);
                await context.close();
              } catch (err) {
                result = { ok: false, error: String(err), label };
              }
              Object.assign(result, { cell: { preset, backend, scenario, throttle, diag, throughput, run } });
              results.push(result);
              console.log(result.ok ? `${result.fps.toFixed(1)} fps, frame p50 ${result.interval?.p50.toFixed(1)} ms, CPU ${result.cpu?.frame.avg.toFixed(1)} ms, GPU ${result.gpu.frame ? result.gpu.frame.avg.toFixed(1) : 'n/a'} ms` : `failed: ${result.error}`);
              writeFileSync(join(out, 'results.json'), JSON.stringify(results, null, 1));
            }
          }
        }
      }
    }
  }
  const md = table(results);
  writeFileSync(join(out, 'results.md'), md);
  console.log(`\n${md}\nwritten to ${out}`);
}

/** Medians over runs, one row per cell. */
function table(results) {
  const median = (xs) => {
    const s = xs.filter((x) => Number.isFinite(x)).sort((a, b) => a - b);
    return s.length ? s[Math.floor((s.length - 1) / 2)] : NaN;
  };
  const cells = new Map();
  for (const r of results) {
    const c = r.cell;
    const key = [c.preset, c.backend, c.scenario, c.throttle, c.diag].join('|');
    if (!cells.has(key)) cells.set(key, []);
    cells.get(key).push(r);
  }
  const f = (x, d = 1) => (Number.isFinite(x) ? x.toFixed(d) : '–');
  const rows = [
    `Scene \`${scene}\`, 1920×1080 at DPR 1, render scale ${res}, ${throughput ? 'no vsync (throughput)' : 'vsync'}, ${secs} s after ${warmup} s warm-up, median of ${runs} run(s).`,
    '',
    '| preset | backend | scenario | diag | CPU× | fps | frame p50 / p95 / p99 ms | CPU ms (update + draw) | GPU ms avg / p95 | draws (shadow / mirror) | triangles M (shadow / mirror) | passes · M px×samples · targets MB | residents ms | lighting /s | reflections /s | long tasks | fingerprint |',
    '|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|',
  ];
  for (const list of cells.values()) {
    const ok = list.filter((r) => r.ok);
    const c = list[0].cell;
    if (!ok.length) {
      rows.push(`| ${c.preset} | ${c.backend} | ${c.scenario} | ${c.diag} | ${c.throttle} | failed: ${list[0].error ?? ''} |`);
      continue;
    }
    const m = (get) => median(ok.map(get));
    const r = ok[0];
    rows.push(
      `| ${c.preset} | ${c.backend} | ${c.scenario} | ${c.diag} | ${c.throttle} | ${f(m((x) => x.fps))} | ${f(m((x) => x.interval?.p50))} / ${f(m((x) => x.interval?.p95))} / ${f(m((x) => x.interval?.p99))} | ${f(m((x) => x.cpu?.frame.avg), 2)} (${f(m((x) => x.cpu?.update.avg), 2)} + ${f(m((x) => x.cpu?.render.avg), 2)}) | ${r.gpu.frame ? `${f(m((x) => x.gpu.frame?.avg), 2)} / ${f(m((x) => x.gpu.frame?.p95), 2)}` : 'n/a'} | ${f(m((x) => x.drawCalls.total), 0)} (${f(m((x) => x.drawCalls.shadow), 0)} / ${f(m((x) => x.drawCalls.mirrors), 0)}) | ${f(m((x) => x.triangles.total / 1e6), 2)} (${f(m((x) => x.triangles.shadow / 1e6), 2)} / ${f(m((x) => x.triangles.mirrors / 1e6), 2)}) | ${r.passes.length} · ${f(r.pixelSamples / 1e6, 1)} · ${f(m((x) => x.targetBytes / 1e6), 0)} | ${f(m((x) => x.residentsMs), 2)} | ${f(m((x) => x.lightingPerSecond), 1)} | ${f(m((x) => x.reflectionsPerSecond ?? 0), 1)} | ${f(m((x) => x.longTasks.count), 0)} | ${r.fingerprint} |`,
    );
  }
  return rows.join('\n') + '\n';
}

async function shots() {
  const poses = list('poses', Object.keys(POSES).join(','));
  for (const backend of backends) {
    for (const preset of presets) {
      for (const pose of poses) {
        for (const diag of diags) {
          const d = ['freeze', ...(diag === 'none' ? [] : diag.split('+'))].join(',');
          const name = `${preset}-${backend}-${pose}${diag === 'none' ? '' : `-${diag}`}.png`;
          process.stdout.write(`${name} … `);
          try {
            const { context, page } = await open(queryOf(preset, backend, `${POSES[pose]}&diag=${d}&hud=0&secs=0&warmup=${warmup}`), 1, '__benchReady', warmup * 1000 + 240_000);
            // Two more frames, then the picture.
            await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
            await page.screenshot({ path: join(out, name) });
            await context.close();
            console.log('ok');
          } catch (err) {
            console.log(`failed: ${err}`);
          }
        }
      }
    }
  }
  console.log(`shots in ${out}`);
}

/** Per picture in both folders: how many pixels differ (by more than 2 and more than 16 of 255), the largest difference and the PSNR. */
async function compare(a, b, diffDir) {
  const sharp = (await import(createRequire(WEB + 'package.json').resolve('sharp'))).default;
  if (diffDir) mkdirSync(diffDir, { recursive: true });
  const names = readdirSync(a).filter((f) => f.endsWith('.png') && readdirSync(b).includes(f)).sort();
  const rows = ['| picture | pixels differing (>2 / >16) | max | mean | PSNR dB |', '|---|---|---|---|---|'];
  let worst = 0;
  for (const name of names) {
    const [x, y] = await Promise.all([a, b].map((dir) => sharp(join(dir, name)).removeAlpha().raw().toBuffer({ resolveWithObject: true })));
    if (x.info.width !== y.info.width || x.info.height !== y.info.height) {
      rows.push(`| ${name} | sizes differ | | | |`);
      continue;
    }
    const pixels = x.info.width * x.info.height;
    const diff = Buffer.alloc(pixels * 3);
    let over2 = 0;
    let over16 = 0;
    let max = 0;
    let sum = 0;
    let squares = 0;
    for (let i = 0; i < pixels; i++) {
      let d = 0;
      for (let c = 0; c < 3; c++) {
        const e = Math.abs(x.data[i * 3 + c] - y.data[i * 3 + c]);
        d = Math.max(d, e);
        sum += e;
        squares += e * e;
      }
      if (d > 2) over2++;
      if (d > 16) over16++;
      max = Math.max(max, d);
      // The difference picture: a dim copy of the first, differences in red.
      const grey = (x.data[i * 3] + x.data[i * 3 + 1] + x.data[i * 3 + 2]) / 12;
      diff[i * 3] = d > 2 ? Math.min(255, 64 + d * 8) : grey;
      diff[i * 3 + 1] = d > 2 ? 0 : grey;
      diff[i * 3 + 2] = d > 2 ? 0 : grey;
    }
    const mse = squares / (pixels * 3);
    worst = Math.max(worst, over2 / pixels);
    rows.push(`| ${name} | ${((100 * over2) / pixels).toFixed(3)}% / ${((100 * over16) / pixels).toFixed(3)}% | ${max} | ${(sum / (pixels * 3)).toFixed(3)} | ${mse ? (10 * Math.log10((255 * 255) / mse)).toFixed(1) : '∞'} |`);
    if (diffDir && over2) await sharp(diff, { raw: { width: x.info.width, height: x.info.height, channels: 3 } }).png().toFile(join(diffDir, name));
  }
  console.log(rows.join('\n'));
  console.log(`\n${names.length} pictures compared; the most changed has ${(100 * worst).toFixed(3)}% of its pixels off by more than 2.`);
}
