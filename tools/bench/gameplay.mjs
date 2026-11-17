#!/usr/bin/env bun
/**
 * The end-to-end gameplay benchmark: a cold browser from process start through the menus into a
 * household's game, played in real time for `--secs` seconds while everything that can be measured
 * without root is measured, then a clean shutdown. Raw samples, a summary and a report are written
 * under `bench-results/gameplay-<time>/`.
 *
 * Stages, each timed on the wall clock from just before the browser launch:
 * - `startup`: browser launch → the main menu shown (the loading screen gone);
 * - `entry`: "New game" → "Choose a household" → that household's "Play" → the game interface up
 *   (`.hud-root`, the loading screen gone);
 * - `steady`: `--secs` seconds from the entry (default 300), sampled every second.
 *
 * What is sampled:
 * - frames: every `requestAnimationFrame` timestamp in the page (raw gaps kept), and the renderer's
 *   own smoothed fps/frame time (`window.__gameStats()`, `?gpustats`);
 * - CPU: CDP `Performance.getMetrics` deltas (TaskDuration, ScriptDuration, LayoutDuration) per
 *   interval — the main thread's busy share — and the browser process tree's CPU time from `ps`;
 * - GPU: the renderer's own GPU frame time (`RenderStats.gpuMs`, measured for the session by
 *   `?gpustats`), plus the GPU process's CPU time from `ps`;
 * - memory: per-process RSS by role (the renderer — the tab running the game —, the GPU process, the
 *   browser), their sum (a ceiling: shared pages count once per process), CDP JSHeapUsedSize + DOM
 *   nodes at 1 Hz, and the renderer's `phys_footprint` at the edges of the steady window;
 * - I/O and network: every request (CDP Network: URL, type, status, encoded bytes, timing, cache),
 *   the page's resource timings (decoded bytes), asset bytes and request counts per stage, and
 *   external (non-localhost) requests, which the game should make none of at runtime. Per-process
 *   disk bytes need root on macOS (fs_usage), so they are reported unavailable, not faked.
 *
 *   bun tools/bench/gameplay.mjs [options]
 *     --runs 2                 full runs (each in a fresh context), compared at the end
 *     --secs 300               steady state after the household is entered
 *     --seed 7                 the page's Math.random seed
 *     --household 0            which household's Play is pressed (its index in the list)
 *     --preset medium          graphics preset (low, medium, high)
 *     --renderer webgpu        or webgl
 *     --hud 1                  the interface shown
 *     --warm                   prime the context with a first load (default: cold, nothing kept)
 *     --headed                 in a window (default headless: the real GPU either way)
 *     --port 5222 | --url <u>  a dev server on the port (started if none answers), or one already running
 *     --startup-timeout 120    seconds for startup, --entry-timeout 180 for the entry
 *     --steady-grace 30        seconds the steady state may overrun `--secs` before it fails
 *     --out <dir>              default bench-results/gameplay-<time>/
 *
 * Needs the dev server (the `?gpustats` hook is dev-only), so `--url` must point at one.
 */
import { spawn, spawnSync } from 'node:child_process';
import { appendFileSync, existsSync, mkdirSync, readdirSync, writeFileSync } from 'node:fs';
import { homedir, cpus } from 'node:os';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { quiet } from './quiet.mjs';

const STARTED = Date.now();
const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const WEB = join(ROOT, 'web/');
const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};
const has = (name) => args.includes(`--${name}`);

const runs = Number(flag('runs', '2'));
const secs = Number(flag('secs', '300'));
const seed = Number(flag('seed', '7'));
const pick = Number(flag('household', '0'));
const preset = flag('preset', 'medium');
const renderer = flag('renderer', 'webgpu');
const hud = flag('hud', '1');
const warm = has('warm');
/** Sample the JS heap through the steady window (writes `heapprofile.json`, reports the biggest sites). */
const profileHeap = has('heapprofile');
const startupTimeout = Number(flag('startup-timeout', '120'));
const entryTimeout = Number(flag('entry-timeout', '180'));
const steadyGrace = Number(flag('steady-grace', '30'));
const port = Number(flag('port', '5222'));
const stamp = new Date().toISOString().replace(/\.\d+Z$/, '').replace(/[:T]/g, '-');
const out = flag('out', join(ROOT, 'bench-results', `gameplay-${stamp}`));

if (has('help') || args.includes('-h')) {
  console.log(`Usage: bun tools/bench/gameplay.mjs [options]

  --runs 2                 full runs (each in a fresh context), compared at the end
  --secs 300               steady state after the household is entered
  --seed 7                 the page's Math.random seed
  --household 0            which household's Play is pressed (its index in the list)
  --preset medium          graphics preset (low, medium, high)
  --renderer webgpu        or webgl
  --hud 1                  the interface shown (0 hides it)
  --warm                   prime the context with a first load (default: cold)
  --heapprofile            sample the JS heap through the steady state (heapprofile.json, top sites in the report)
  --headed                 in a window (default headless: the real GPU either way)
  --port 5222 | --url <u>  a dev server on the port (started if none answers), or one already running
  --startup-timeout 120    seconds for startup, --entry-timeout 180 for the entry
  --steady-grace 30        seconds the steady state may overrun --secs before it fails
  --out <dir>              default bench-results/gameplay-<time>/

Needs a dev server (the ?gpustats hook is dev-only). See the comment at the top of this file.`);
  process.exit(0);
}

const bad = (msg) => {
  console.error(`gameplay.mjs: ${msg}\n(run with --help for the options)`);
  process.exit(2);
};
if (!Number.isInteger(runs) || runs < 1) bad(`--runs must be a positive integer, not ${JSON.stringify(flag('runs', '2'))}`);
if (!(secs > 0)) bad(`--secs must be a positive number, not ${JSON.stringify(flag('secs', '300'))}`);
if (!Number.isInteger(seed)) bad(`--seed must be an integer, not ${JSON.stringify(flag('seed', '7'))}`);
if (!Number.isInteger(pick) || pick < 0) bad(`--household must be a non-negative index, not ${JSON.stringify(flag('household', '0'))}`);
if (!['low', 'medium', 'high'].includes(preset)) bad(`--preset must be low, medium or high, not ${preset}`);
if (!['webgpu', 'webgl'].includes(renderer)) bad(`--renderer must be webgpu or webgl, not ${renderer}`);
if (!['0', '1'].includes(hud)) bad(`--hud must be 0 or 1, not ${hud}`);
if (!(startupTimeout > 0 && entryTimeout > 0 && steadyGrace >= 0)) bad('the timeouts must be positive');

mkdirSync(out, { recursive: true });

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const sh = (cmd, a, opts) => {
  try {
    // (`footprint` prints megabytes of lines for a large process.)
    return String(spawnSync(cmd, a, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, ...opts }).stdout ?? '').trim();
  } catch {
    return '';
  }
};
/** Milliseconds since this process started. */
const since = () => Date.now() - STARTED;

async function playwright() {
  if (process.env.PLAYWRIGHT) return import(process.env.PLAYWRIGHT);
  const require = createRequire(WEB + 'package.json');
  try {
    return await import(require.resolve('playwright'));
  } catch {}
  // (`playwright-core` is what the repo has: same Chromium API, an explicit executable.)
  try {
    return await import(require.resolve('playwright-core'));
  } catch {}
  throw new Error('Playwright not found: install it, or set PLAYWRIGHT=<path to playwright/index.mjs>');
}

/** The Chromium in Playwright's browser cache (for `playwright-core`, which has no registry). */
function cachedChromium() {
  const cache = join(homedir(), 'Library/Caches/ms-playwright');
  if (!existsSync(cache)) return null;
  const dirs = readdirSync(cache).filter((d) => /^chromium-\d+$/.test(d)).sort((a, b) => Number(b.split('-')[1]) - Number(a.split('-')[1]));
  for (const d of dirs) {
    for (const flavour of readdirSync(join(cache, d)).filter((f) => f.startsWith('chrome-mac'))) {
      const apps = readdirSync(join(cache, d, flavour)).filter((f) => f.endsWith('.app'));
      for (const app of apps) {
        for (const exe of ['Chromium', 'Google Chrome for Testing', 'Google Chrome']) {
          const f = join(cache, d, flavour, app, 'Contents/MacOS', exe);
          if (existsSync(f)) return f;
        }
      }
    }
  }
  return null;
}

async function running(url) {
  try {
    return (await fetch(url, { signal: AbortSignal.timeout(1500) })).ok;
  } catch {
    return false;
  }
}

// ---------------------------------------------------------------- the server (started only if needed)

let server = null;
let serverMode = 'reused';
const base = flag('url', null) ?? `http://localhost:${port}`;
let serverReadyAt = STARTED;
if (!flag('url', null) && !(await running(base))) {
  serverMode = 'started';
  server = spawn(`${WEB}node_modules/.bin/vite`, ['--port', String(port), '--strictPort'], { cwd: WEB, stdio: ['ignore', 'pipe', 'inherit'] });
  process.on('exit', () => server?.kill());
  await new Promise((resolve, reject) => {
    server.stdout.on('data', (d) => String(d).includes('Local') && resolve());
    server.on('exit', (code) => reject(new Error(`vite exited (${code})`)));
    setTimeout(() => reject(new Error('vite did not report "Local" in 120 s')), 120_000);
  });
  serverReadyAt = Date.now();
}

// ---------------------------------------------------------------- page-side helpers

/** The page's Math.random, seeded (mulberry32), before any of its scripts. */
function seeded(n) {
  let a = (n ^ 0x9e3779b9) >>> 0;
  Math.random = () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Every drawn frame's timestamp and the gap before it (ms, the page's clock), from the first script. */
function frameRecorder() {
  // (Keep every request's timing: the default buffer holds 250 and the dev server makes far more.)
  performance.setResourceTimingBufferSize?.(20000);
  window.__frames = [];
  let last = 0;
  const tick = (t) => {
    if (last) window.__frames.push([Math.round(t), Math.round((t - last) * 1000) / 1000]);
    last = t;
    requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
}

// ---------------------------------------------------------------- the browser process tree

/** `ps` once: pid, ppid, RSS (KB), cumulative CPU time (s) and the command line. */
function psTable() {
  const text = sh('ps', ['-axo', 'pid=,ppid=,rss=,time=,command=']);
  const rows = [];
  for (const line of text.split('\n')) {
    const m = line.match(/^\s*(\d+)\s+(\d+)\s+(\d+)\s+([\d:.-]+)\s+(.*)$/);
    if (m) rows.push({ pid: +m[1], ppid: +m[2], rssKb: +m[3], cpuS: cpuSeconds(m[4]), command: m[5] });
  }
  return rows;
}

/** `ps`'s CPU time ("mm:ss.cc", "hh:mm:ss", "d-hh:mm:ss") in seconds. */
function cpuSeconds(text) {
  const days = text.includes('-') ? Number(text.split('-')[0]) : 0;
  const rest = text.includes('-') ? text.split('-')[1] : text;
  const parts = rest.split(':').map(Number);
  while (parts.length < 3) parts.unshift(0);
  return days * 86_400 + parts[0] * 3600 + parts[1] * 60 + parts[2];
}

/** The launched browser's pid: the Chromium child of this process (Playwright's own, if it knows it). */
function browserRootPid(rows, known) {
  if (known && rows.some((r) => r.pid === known)) return known;
  const mine = rows.filter((r) => r.ppid === process.pid && /Chromium|Google Chrome|chrome/i.test(r.command));
  return mine.length ? mine[0].pid : null;
}

/** The root's whole process tree. */
function processTree(rows, root) {
  const byPid = new Map(rows.map((r) => [r.pid, r]));
  const kids = new Map();
  for (const r of rows) {
    if (!kids.has(r.ppid)) kids.set(r.ppid, []);
    kids.get(r.ppid).push(r.pid);
  }
  const seen = new Set();
  const tree = [];
  const stack = [root];
  while (stack.length) {
    const pid = stack.pop();
    if (seen.has(pid)) continue;
    seen.add(pid);
    const row = byPid.get(pid);
    if (row) tree.push(row);
    for (const k of kids.get(pid) ?? []) stack.push(k);
  }
  return tree;
}

/**
 * The tree's processes by role: the browser (no `--type=`), the GPU process, and the largest renderer
 * (a page can have more than one: a pre-render, a spare, an extension).
 */
function byRole(tree) {
  const renderers = tree.filter((r) => /--type=renderer/.test(r.command)).sort((a, b) => b.rssKb - a.rssKb);
  return {
    renderers,
    renderer: renderers[0] ?? null,
    gpu: tree.find((r) => /--type=gpu-process|Helper \(GPU\)/.test(r.command)) ?? null,
    browser: tree.find((r) => !/--type=/.test(r.command)) ?? null,
  };
}

const roleOf = (r) =>
  /--type=renderer/.test(r.command) ? 'renderer' : /--type=gpu-process|Helper \(GPU\)/.test(r.command) ? 'gpu' : /--type=/.test(r.command) ? (r.command.match(/--type=([\w-]+)/)?.[1] ?? 'other') : 'browser';
const rssMB = (r) => (r ? +(r.rssKb / 1024).toFixed(1) : null);
/** Bytes as MB with one decimal; null passes through (a number the page couldn't give). */
const mb = (bytes) => (bytes == null ? null : +(bytes / 1048576).toFixed(1));

/**
 * V8's own accounting: the only CDP call that sees the ArrayBuffers behind typed arrays, which
 * `JSHeapUsedSize` does not count and where a geometry- and texture-heavy renderer keeps most of its bytes.
 * Null when the target is gone (the caller writes null and the report shows `—`).
 */
async function heapUsage(cdp) {
  try {
    return await cdp.send('Runtime.getHeapUsage');
  } catch {
    return null;
  }
}

/** Every `<canvas>` on the page, and the RGBA bytes its backing store implies (the GPU may hold more). */
async function canvasCensus(page) {
  try {
    return await page.evaluate(() => {
      const canvases = [...document.querySelectorAll('canvas')].map((c) => ({
        w: c.width,
        h: c.height,
        mb: +((c.width * c.height * 4) / 1048576).toFixed(1),
        cls: c.className || c.id || '',
      }));
      return {
        canvases: canvases.length,
        canvasMB: +canvases.reduce((a, c) => a + c.mb, 0).toFixed(1),
        largest: canvases.sort((a, b) => b.mb - a.mb).slice(0, 6),
      };
    });
  } catch {
    return null;
  }
}
const rssSumMB = (tree) => +(tree.reduce((a, r) => a + r.rssKb, 0) / 1024).toFixed(1);

/**
 * A process's `phys_footprint` (dirty + compressed, the number Activity Monitor shows), from
 * `/usr/bin/footprint`: a few hundred ms, and it suspends the process while it reads it. Only taken
 * at the edges of the steady state, never inside the sampled window.
 */
function footprintMB(pid) {
  if (!pid) return { available: false, reason: 'no process' };
  if (!existsSync('/usr/bin/footprint')) return { available: false, reason: 'no /usr/bin/footprint' };
  const out = sh('/usr/bin/footprint', [String(pid)], { timeout: 60_000 });
  const m = out.match(/phys_footprint:\s*([\d.]+)\s*(B|KB|MB|GB)/);
  if (!m) return { available: false, reason: 'footprint printed no phys_footprint' };
  const mult = { B: 1 / 1048576, KB: 1 / 1024, MB: 1, GB: 1024 }[m[2]];
  return { available: true, mb: +(Number(m[1]) * mult).toFixed(1), command: `footprint ${pid}` };
}

// ---------------------------------------------------------------- stats

/** min/mean/median/p95/p99/max of the finite values (nearest rank). */
function stats(xs) {
  const s = xs.filter((x) => Number.isFinite(x)).sort((a, b) => a - b);
  if (!s.length) return null;
  const at = (q) => s[Math.min(s.length - 1, Math.floor(q * s.length))];
  return {
    n: s.length,
    min: +s[0].toFixed(2),
    mean: +(s.reduce((a, b) => a + b, 0) / s.length).toFixed(2),
    median: +at(0.5).toFixed(2),
    p95: +at(0.95).toFixed(2),
    p99: +at(0.99).toFixed(2),
    max: +s[s.length - 1].toFixed(2),
  };
}

// ---------------------------------------------------------------- one run

/**
 * One full run: launch, menus, a household, `secs` seconds of play, shutdown. Writes `raw.jsonl` and
 * `summary.json` in `dir` and returns the summary. Throws on any failed stage (never partial success).
 */
async function runOnce(n, chromium, executablePath) {
  const dir = join(out, `run${n}`);
  mkdirSync(dir, { recursive: true });
  const rawPath = join(dir, 'raw.jsonl');
  writeFileSync(rawPath, '');
  const raw = (o) => appendFileSync(rawPath, `${JSON.stringify(o)}\n`);
  const fail = (msg) => {
    throw new Error(`run ${n}: ${msg}`);
  };

  let stage = 'startup';
  const timeline = {};
  const pageErrors = [];
  const samples = [];
  const frames = [];
  const net = [];
  const requests = new Map();
  const stagesSeen = {};

  let perf = null;
  let heapProfileRows = null;
  let resources = [];
  let footprintStart = { available: false, reason: 'not taken' };
  let footprintEnd = { available: false, reason: 'not taken' };
  const launchAt = Date.now();
  const browser = await chromium.launch({
    headless: !has('headed'),
    ...(executablePath ? { executablePath } : { channel: 'chromium' }),
    args: [
      '--enable-unsafe-webgpu',
      '--enable-webgpu-developer-features',
      '--disable-backgrounding-occluded-windows',
      '--disable-renderer-backgrounding',
      '--disable-background-timer-throttling',
    ],
  });
  timeline.launchMs = Date.now() - launchAt;
  const browserVersion = browser.version();
  let browserPid = null;
  const boundaries = [];
  /** Memory and CPU-time at a stage boundary: taken at the edges, never per frame. */
  const boundary = async (name) => {
    let metrics = {};
    try {
      metrics = Object.fromEntries((await cdp.send('Performance.getMetrics')).metrics.map((m) => [m.name, m.value]));
    } catch {
      metrics = {};
    }
    const rowsNow = psTable();
    const usage = await heapUsage(cdp);
    if (!browserPid) browserPid = browserRootPid(rowsNow, null);
    const tree = browserPid ? processTree(rowsNow, browserPid) : [];
    const role = byRole(tree);
    let game = null;
    try {
      game = await page.evaluate(() => window.__gameStats?.() ?? null);
    } catch {
      game = null;
    }
    const census = await canvasCensus(page);
    const b = {
      type: 'boundary',
      stage: name,
      at: since(),
      treeRssMB: tree.length ? rssSumMB(tree) : null,
      rendererRssMB: rssMB(role.renderer),
      renderersRssMB: role.renderers.length ? rssSumMB(role.renderers) : null,
      rendererProcs: role.renderers.length,
      gpuRssMB: rssMB(role.gpu),
      browserRssMB: rssMB(role.browser),
      heapMB: metrics.JSHeapUsedSize != null ? +(metrics.JSHeapUsedSize / 1048576).toFixed(1) : null,
      backingMB: mb(usage?.backingStorageSize),
      heapTotalMB: mb(usage?.totalSize),
      canvases: census?.canvases ?? null,
      canvasMB: census?.canvasMB ?? null,
      canvasLargest: census?.largest ?? null,
      vramMB: mb(game?.vram?.total),
      vramTexMB: mb(game?.vram?.tex),
      simMemoryMB: mb(game?.simMemoryBytes),
      domNodes: metrics.Nodes ?? null,
      procs: tree.length,
      treeCpuS: tree.length ? +tree.reduce((a, r) => a + r.cpuS, 0).toFixed(2) : null,
    };
    boundaries.push(b);
    raw(b);
    return b;
  };
  try {
    browserPid = browser.process?.()?.pid ?? null;
  } catch {}
  raw({ type: 'stage', stage: 'startup', event: 'begin', at: since(), wall: new Date().toISOString(), launchMs: timeline.launchMs });

  const context = await browser.newContext({ viewport: { width: 1280, height: 720 }, deviceScaleFactor: 1 });
  await context.addInitScript(quiet);
  await context.addInitScript(seeded, seed);
  await context.addInitScript(frameRecorder);
  const page = await context.newPage();
  page.on('pageerror', (e) => pageErrors.push(String(e.message ?? e)));
  page.on('crash', () => pageErrors.push('page crashed'));
  const cdp = await context.newCDPSession(page);
  await cdp.send('Performance.enable');
  await cdp.send('Network.enable', { maxTotalBufferSize: 50_000_000, maxResourceBufferSize: 10_000_000 });

  cdp.on('Network.requestWillBeSent', (e) => {
    requests.set(e.requestId, { url: e.request.url, type: e.type, method: e.request.method, stage, wallAt: since(), ts: e.timestamp, initiator: e.initiator?.type });
  });
  cdp.on('Network.responseReceived', (e) => {
    const r = requests.get(e.requestId);
    if (!r) return;
    r.status = e.response.status;
    r.mime = e.response.mimeType;
    r.fromCache = !!(e.response.fromDiskCache || e.response.fromMemoryCache || e.response.fromServiceWorker);
    r.protocol = e.response.protocol;
  });
  cdp.on('Network.loadingFinished', (e) => {
    const r = requests.get(e.requestId);
    if (!r) return;
    r.encodedBytes = e.encodedDataLength;
    r.finishedAt = since();
    requests.delete(e.requestId);
    net.push(r);
    raw({ type: 'net', ...r });
  });
  cdp.on('Network.loadingFailed', (e) => {
    const r = requests.get(e.requestId);
    if (!r) return;
    requests.delete(e.requestId);
    net.push({ ...r, failed: e.errorText });
    raw({ type: 'net', ...r, failed: e.errorText });
  });

  const query = ['profile', 'gpustats', `quality=${preset}`, `hud=${hud}`, renderer === 'webgl' ? 'renderer=webgl' : ''].filter(Boolean).join('&');
  const url = `${base}/?${query}`;
  const menuUp = () => page.waitForFunction(() => !!document.querySelector('.main-menu, .menu') && !document.querySelector('.loading'), null, { timeout: startupTimeout * 1000, polling: 100 });

  try {
    if (warm) {
      // (`--warm`: the browser's cache primed by a first load in this context, before the measured one.)
      await page.goto(url);
      await menuUp();
      await page.waitForTimeout(2000);
      await page.goto('about:blank');
    }

    await page.goto(url);
    await menuUp();
    timeline.startupMs = Date.now() - launchAt;
    timeline.pageMarks = { menu: await page.evaluate(() => performance.now()) };
    stagesSeen.startup = Date.now();
    raw({ type: 'stage', stage: 'startup', event: 'end', at: since(), ms: timeline.startupMs, frames: (await drainFrames(page, frames, 'startup')).length });
    await boundary('startup');
    if (pageErrors.length) fail(`page error during startup: ${pageErrors.join('; ')}`);

    // ---- entry: "New game" → step 1 → step 2's households → that household's Play → the game up.
    stage = 'entry';
    const entryFrom = Date.now();
    raw({ type: 'stage', stage: 'entry', event: 'begin', at: since() });
    await page.getByText('New game', { exact: true }).click({ timeout: entryTimeout * 1000 });
    const next = page.getByRole('button', { name: /Choose a household/ });
    await next.waitFor({ timeout: entryTimeout * 1000 });
    await page.waitForFunction(() => [...document.querySelectorAll('button')].some((b) => /Choose a household/.test(b.textContent ?? '') && !b.disabled), null, { timeout: entryTimeout * 1000, polling: 100 });
    await page.waitForTimeout(500);
    await next.click();
    await page.waitForFunction(
      () => {
        const faces = [...document.querySelectorAll('.households .face')];
        return faces.length > 0 && faces.every((f) => f.querySelector('img'));
      },
      null,
      { timeout: entryTimeout * 1000, polling: 50 },
    );
    await page.waitForTimeout(500);
    const plays = page.locator('.households .btn.play');
    const count = await plays.count();
    if (!count) fail('no household to Play (no .households .btn.play)');
    const pressedAt = Date.now();
    await plays.nth(Math.min(pick, count - 1)).click({ timeout: entryTimeout * 1000 });
    await page.waitForFunction(() => !!document.querySelector('.hud-root') && !document.querySelector('.loading'), null, { timeout: entryTimeout * 1000, polling: 100 });
    const enteredAt = Date.now();
    timeline.entryMs = enteredAt - pressedAt;
    timeline.entryStageMs = enteredAt - entryFrom;
    timeline.pageMarks.entered = await page.evaluate(() => performance.now());
    timeline.household = Math.min(pick, count - 1);
    stagesSeen.entry = enteredAt;
    raw({ type: 'stage', stage: 'entry', event: 'end', at: since(), ms: timeline.entryMs, households: count, frames: (await drainFrames(page, frames, 'entry')).length });
    await boundary('entry');
    if (pageErrors.length) fail(`page error during entry: ${pageErrors.join('; ')}`);

    // ---- steady: `secs` seconds of play, sampled every second.
    stage = 'steady';
    const steadyFrom = enteredAt;
    raw({ type: 'stage', stage: 'steady', event: 'begin', at: since(), pageT: await page.evaluate(() => performance.now()) });
    if (profileHeap) {
      try {
        await cdp.send('HeapProfiler.enable');
        await cdp.send('HeapProfiler.startSampling', { samplingInterval: 32768 });
      } catch (err) {
        raw({ type: 'heapprofile', stage: 'steady', error: String(err).split('\n')[0] });
      }
    }
    // (`footprint` suspends the renderer while it reads it: taken before the window and after it.)
    {
      const r0 = byRole(processTree(psTable(), browserPid ?? 0));
      footprintStart = footprintMB(r0.renderer?.pid);
      raw({ type: 'footprint', stage: 'steadyStart', ...footprintStart, rendererRssMB: rssMB(r0.renderer) });
    }
    let previous = null;
    for (let i = 0; Date.now() - steadyFrom < secs * 1000; i++) {
      const target = steadyFrom + (i + 1) * 1000;
      const at = Date.now();
      let metrics;
      try {
        metrics = Object.fromEntries((await cdp.send('Performance.getMetrics')).metrics.map((m) => [m.name, m.value]));
      } catch (err) {
        fail(`CDP Performance.getMetrics failed at ${(at - steadyFrom) / 1000}s: ${String(err).split('\n')[0]}`);
      }
      let gameStats = null;
      try {
        gameStats = await page.evaluate(() => window.__gameStats?.() ?? null);
      } catch (err) {
        fail(`page gone at ${((Date.now() - steadyFrom) / 1000).toFixed(0)}s: ${String(err).split('\n')[0]}`);
      }
      if (!gameStats) fail('window.__gameStats() is missing: the page is not the dev server with ?gpustats, or the game is not up');
      const rows = psTable();
      const usage = await heapUsage(cdp);
      if (!browserPid) browserPid = browserRootPid(rows, null);
      if (!browserPid) fail('could not find the browser process in `ps` (no Chromium child of this process)');
      const tree = processTree(rows, browserPid);
      const role = byRole(tree);
      const gpu = role.gpu;
      const cpuS = tree.reduce((a, r) => a + r.cpuS, 0);
      const dt = previous ? (at - previous.at) / 1000 : null;
      const sample = {
        type: 'sample',
        stage,
        t: +( (at - steadyFrom) / 1000 ).toFixed(3),
        at: since(),
        metrics: {
          // (CDP's durations are seconds, cumulative: the interval's share is a delta.)
          taskBusyPct: previous && dt ? +((((metrics.TaskDuration - previous.metrics.TaskDuration) / dt) * 100).toFixed(2)) : null,
          scriptPct: previous && dt ? +((((metrics.ScriptDuration - previous.metrics.ScriptDuration) / dt) * 100).toFixed(2)) : null,
          layoutPct: previous && dt ? +((((metrics.LayoutDuration - previous.metrics.LayoutDuration) / dt) * 100).toFixed(2)) : null,
          heapMB: +(metrics.JSHeapUsedSize / 1048576).toFixed(2),
          backingMB: mb(usage?.backingStorageSize),
          heapTotalMB: mb(usage?.totalSize),
          domNodes: metrics.Nodes ?? null,
          documents: metrics.Documents ?? null,
          // (The sum counts each process's RSS whole, so shared pages are counted once per process:
          // the renderer's own RSS and the JS heap are the numbers to read, the sum is the ceiling.)
          rssTreeMB: rssSumMB(tree),
          rssRendererMB: rssMB(role.renderer),
          rssRenderersMB: role.renderers.length ? rssSumMB(role.renderers) : null,
          rendererProcs: role.renderers.length,
          rssGpuMB: rssMB(role.gpu),
          rssBrowserMB: rssMB(role.browser),
          procs: tree.length,
          treeCpuPct: previous && dt ? +((((cpuS - previous.cpuS) / dt) * 100).toFixed(1)) : null,
          gpuCpuPct: gpu && previous?.gpuCpuS != null && dt ? +((((gpu.cpuS - previous.gpuCpuS) / dt) * 100).toFixed(1)) : null,
          gpuMs: gameStats.gpuMs,
          rendererFps: +gameStats.fps.toFixed(2),
          rendererFrameMs: +gameStats.frameMs.toFixed(2),
          rendererCpuMs: +gameStats.cpuMs.toFixed(2),
          drawCalls: gameStats.drawCalls,
          renderWidth: gameStats.renderWidth,
          renderHeight: gameStats.renderHeight,
          renderScale: +gameStats.renderScale.toFixed(3),
          backend: gameStats.backend,
          preset: gameStats.preset,
          // (The renderer's own GPU counters, and the sim worker's WASM memory as it reports it.)
          vramMB: mb(gameStats.vram?.total),
          vramTexMB: mb(gameStats.vram?.tex),
          vramGeomMB: mb(gameStats.vram?.geom),
          simMemoryMB: mb(gameStats.simMemoryBytes),
        },
      };
      samples.push(sample);
      raw(sample);
      if (i === 0) {
        raw({ type: 'procs', stage: 'steady', at, tree: tree.map((r) => ({ pid: r.pid, role: roleOf(r), mb: rssMB(r), command: r.command.slice(0, 140) })) });
      }
      previous = { at, metrics, cpuS, gpuCpuS: gpu?.cpuS ?? null };
      for (const f of await drainFrames(page, frames, 'steady')) raw({ type: 'frame', stage: 'steady', t: f[0], gap: f[1] });
      if (pageErrors.length) fail(`page error during steady state: ${pageErrors.join('; ')}`);
      const rest = target - Date.now();
      if (rest > 0) await sleep(rest);
    }
    timeline.steadyMs = Date.now() - steadyFrom;
    if (profileHeap) {
      try {
        const { profile } = await cdp.send('HeapProfiler.stopSampling');
        writeFileSync(join(dir, 'heapprofile.json'), JSON.stringify(profile));
        heapProfileRows = topAllocations(profile);
        raw({ type: 'heapprofile', stage: 'steady', rows: heapProfileRows });
      } catch (err) {
        raw({ type: 'heapprofile', stage: 'steady', error: String(err).split('\n')[0] });
      }
    }
    timeline.steadySamples = samples.length;
    stagesSeen.steady = steadyFrom;
    // (The frames between the last sample and the end.)
    await drainFrames(page, frames, 'steady');

    // ---- the window is fully covered, or this is not a measurement.
    const steadyFrames = frames.filter((f) => f[2] === 'steady');
    const frameSpan = steadyFrames.length ? steadyFrames[steadyFrames.length - 1][0] - steadyFrames[0][0] : 0;
    if (timeline.steadyMs < secs * 1000) fail(`steady state lasted ${(timeline.steadyMs / 1000).toFixed(1)} s, less than ${secs} s`);
    if (timeline.steadyMs > (secs + steadyGrace) * 1000) fail(`steady state overran: ${(timeline.steadyMs / 1000).toFixed(1)} s > ${secs} + ${steadyGrace} s`);
    if (samples.length < secs) fail(`only ${samples.length} samples in ${(timeline.steadyMs / 1000).toFixed(0)} s (need >= ${secs})`);
    if (frameSpan < secs * 1000 * 0.95) fail(`frames cover only ${(frameSpan / 1000).toFixed(1)} s of the ${secs} s steady state`);
    timeline.steadyFrameSpanMs = Math.round(frameSpan);

    // ---- what the page and the browser say at the end.
    perf = await page.evaluate(() => window.__perfReport?.() ?? null).catch(() => null);
    resources = await page.evaluate(() =>
      performance.getEntriesByType('resource').map((e) => ({ name: e.name, type: e.initiatorType, start: +e.startTime.toFixed(1), duration: +e.duration.toFixed(1), transfer: e.transferSize, decoded: e.decodedBodySize, encoded: e.encodedBodySize })),
    ).catch(() => []);
    const roleEnd = byRole(processTree(psTable(), browserPid ?? 0));
    footprintEnd = footprintMB(roleEnd.renderer?.pid);
    raw({ type: 'footprint', stage: 'steadyEnd', ...footprintEnd, rendererRssMB: rssMB(roleEnd.renderer) });
    await boundary('steadyEnd');
    raw({ type: 'stage', stage: 'steady', event: 'end', at: since(), ms: timeline.steadyMs });
    raw({ type: 'perf', report: perf });
    raw({ type: 'resources', entries: resources });
  } finally {
    await context.close().catch(() => {});
    await browser.close().catch(() => {});
  }

  // ---------------------------------------------------------------- summary of this run
  const perStage = {};
  for (const s of ['startup', 'entry', 'steady']) {
    const own = samples.filter((x) => x.stage === s);
    const ownFrames = frames.filter((f) => f[2] === s);
    perStage[s] = {
      frames: ownFrames.length,
      frameGapMs: stats(ownFrames.map((f) => f[1])),
      fps: stats(perSecondFps(ownFrames)),
      rendererFps: stats(own.map((x) => x.metrics.rendererFps)),
      rendererFrameMs: stats(own.map((x) => x.metrics.rendererFrameMs)),
      rendererCpuMs: stats(own.map((x) => x.metrics.rendererCpuMs)),
      gpuMs: stats(own.map((x) => x.metrics.gpuMs)),
      drawCalls: stats(own.map((x) => x.metrics.drawCalls)),
      taskBusyPct: stats(own.map((x) => x.metrics.taskBusyPct)),
      scriptPct: stats(own.map((x) => x.metrics.scriptPct)),
      layoutPct: stats(own.map((x) => x.metrics.layoutPct)),
      treeCpuPct: stats(own.map((x) => x.metrics.treeCpuPct)),
      gpuCpuPct: stats(own.map((x) => x.metrics.gpuCpuPct)),
      rssRendererMB: stats(own.map((x) => x.metrics.rssRendererMB)),
      rssRenderersMB: stats(own.map((x) => x.metrics.rssRenderersMB)),
      rssTreeMB: stats(own.map((x) => x.metrics.rssTreeMB)),
      rssGpuMB: stats(own.map((x) => x.metrics.rssGpuMB)),
      rssBrowserMB: stats(own.map((x) => x.metrics.rssBrowserMB)),
      heapMB: stats(own.map((x) => x.metrics.heapMB)),
    backingMB: stats(own.map((x) => x.metrics.backingMB)),
    heapTotalMB: stats(own.map((x) => x.metrics.heapTotalMB)),
      vramMB: stats(own.map((x) => x.metrics.vramMB)),
      vramTexMB: stats(own.map((x) => x.metrics.vramTexMB)),
      vramGeomMB: stats(own.map((x) => x.metrics.vramGeomMB)),
      simMemoryMB: stats(own.map((x) => x.metrics.simMemoryMB)),
      domNodes: stats(own.map((x) => x.metrics.domNodes)),
    };
  }
  // (Startup and entry have no 1 Hz samples: their own frames and spans carry them.)

  const steady = samples;
  const first = steady[0]?.metrics ?? {};
  const last = steady[steady.length - 1]?.metrics ?? {};
  const series = steady.map((s) => ({
    t: s.t,
    rendererMB: s.metrics.rssRendererMB,
    renderersMB: s.metrics.rssRenderersMB,
    treeMB: s.metrics.rssTreeMB,
    gpuMB: s.metrics.rssGpuMB,
    heapMB: s.metrics.heapMB,
    backingMB: s.metrics.backingMB,
    heapTotalMB: s.metrics.heapTotalMB,
    vramMB: s.metrics.vramMB,
    vramTexMB: s.metrics.vramTexMB,
    vramGeomMB: s.metrics.vramGeomMB,
    simMemoryMB: s.metrics.simMemoryMB,
    domNodes: s.metrics.domNodes,
  }));
  const growth = (key) => {
    const a = series[0]?.[key];
    const b = series[series.length - 1]?.[key];
    return a != null && b != null ? +(b - a).toFixed(1) : null;
  };
  const peak = (key) => {
    const values = series.map((s) => s[key]).filter((x) => x != null);
    return values.length ? Math.max(...values) : null;
  };
  const memory = {
    series,
    rendererStartMB: series[0]?.rendererMB ?? null,
    rendererEndMB: series[series.length - 1]?.rendererMB ?? null,
    rendererPeakMB: series.length ? peak('rendererMB') : null,
    rendererGrowthMB: growth('rendererMB'),
    treeStartMB: series[0]?.treeMB ?? null,
    treeEndMB: series[series.length - 1]?.treeMB ?? null,
    treePeakMB: series.length ? peak('treeMB') : null,
    treeGrowthMB: growth('treeMB'),
    gpuStartMB: series[0]?.gpuMB ?? null,
    gpuEndMB: series[series.length - 1]?.gpuMB ?? null,
    gpuGrowthMB: growth('gpuMB'),
    footprintStart: footprintStart ?? null,
    footprintEnd,
    boundaries,
    heapStartMB: first.heapMB ?? null,
    heapEndMB: last.heapMB ?? null,
    heapPeakMB: Math.max(...steady.map((s) => s.metrics.heapMB)),
    heapGrowthMB: first.heapMB != null ? +(last.heapMB - first.heapMB).toFixed(1) : null,
    backingStartMB: first.backingMB ?? null,
    backingEndMB: last.backingMB ?? null,
    backingPeakMB: peak('backingMB'),
    heapTotalPeakMB: peak('heapTotalMB'),
    renderersStartMB: series[0]?.renderersMB ?? null,
    renderersEndMB: series[series.length - 1]?.renderersMB ?? null,
    renderersPeakMB: peak('renderersMB'),
    vramStartMB: series[0]?.vramMB ?? null,
    vramEndMB: series[series.length - 1]?.vramMB ?? null,
    vramPeakMB: peak('vramMB'),
    vramTexPeakMB: peak('vramTexMB'),
    vramGeomPeakMB: peak('vramGeomMB'),
    simMemoryStartMB: series[0]?.simMemoryMB ?? null,
    simMemoryEndMB: series[series.length - 1]?.simMemoryMB ?? null,
    simMemoryPeakMB: peak('simMemoryMB'),
    domNodesStart: first.domNodes ?? null,
    domNodesEnd: last.domNodes ?? null,
  };

  /** A request off the machine (http/https, and not localhost): the game should make none at runtime. */
  const isExternal = (r) => {
    try {
      const u = new URL(r.url);
      return /^https?:$/.test(u.protocol) && !/^(localhost|127\.0\.0\.1|\[::1\])$/.test(u.hostname);
    } catch {
      return false;
    }
  };
  const perStageNet = {};
  for (const s of ['startup', 'entry', 'steady']) {
    const own = net.filter((r) => r.stage === s);
    const ownRes = resources.filter((e) => stageOfResource(e, timeline) === s);
    perStageNet[s] = {
      requests: own.length,
      encodedBytes: own.reduce((a, r) => a + (r.encodedBytes ?? 0), 0),
      decodedBytes: ownRes.reduce((a, r) => a + (r.decoded ?? 0), 0),
      fromCache: own.filter((r) => r.fromCache).length,
      byType: countBy(own, (r) => r.type ?? 'other'),
      external: own.filter(isExternal).length,
      slowest: own.slice().sort((a, b) => (b.encodedBytes ?? 0) - (a.encodedBytes ?? 0)).slice(0, 8).map((r) => ({ url: shortUrl(r.url), bytes: r.encodedBytes ?? 0, type: r.type })),
    };
  }
  const external = net.filter(isExternal);

  const summary = {
    run: n,
    runType: warm ? 'warm' : 'cold',
    browserVersion,
    startedAt: new Date(STARTED).toISOString(),
    finishedAt: new Date().toISOString(),
    url,
    seed,
    preset,
    renderer,
    hud,
    viewport: '1280x720@1',
    household: timeline.household,
    timeline,
    backend: last.backend ?? null,
    presetDrawn: last.preset ?? null,
    renderSize: last.renderWidth ? `${last.renderWidth}x${last.renderHeight}` : null,
    pageErrors,
    stages: perStage,
    memory,
    heapSites: heapProfileRows,
    net: { perStage: perStageNet, total: net.length, external: external.length, externalUrls: external.map((r) => r.url), disk: { available: false, reason: 'macOS: per-process disk I/O needs root (fs_usage); not faked' } },
    perf: perf
      ? {
          entries: perf.entries.length,
          slowest: perf.slowest.slice(0, 12),
          longTasks: perf.longTasks.filter((t) => t.ms >= 50).map((t) => ({ ms: Math.round(t.ms), at: Math.round(t.start), phase: t.phase })),
          droppedFrames: perf.droppedFrames.length,
        }
      : null,
    raw: rawPath,
    samples: steady.length,
  };
  writeFileSync(join(dir, 'summary.json'), JSON.stringify(summary, null, 2));
  return summary;
}

/**
 * The heap profile's self sizes by call site: where the objects still alive at the end of the window were
 * allocated. Sampling misses short-lived objects (it walks the heap at a fixed interval), which is the
 * point: this answers "what is still there", not "what was allocated".
 */
/** The canvas census, or nothing when the run predates it (`boundary()` samples it). */
function censusSection(summaries) {
  const rows = summaries.flatMap((x, i) =>
    (x.memory?.boundaries ?? [])
      .filter((b) => b.canvasMB != null)
      .map(
        (b) =>
          `| run ${i + 1} | ${b.stage} | ${b.canvases} | ${b.canvasMB} | ${(b.canvasLargest ?? []).map((c) => `${c.w}×${c.h}${c.cls ? ` .${c.cls}` : ''} (${c.mb})`).join(', ')} |`,
      ),
  );
  if (!rows.length) return [];
  return [
    '## Canvases',
    '',
    'Every `<canvas>` in the page at its own pixel size (RGBA, 4 bytes a pixel). Chrome keeps these outside',
    "both the JS heap and the engine's VRAM counter, so a page full of offscreen surfaces is invisible to both.",
    '',
    '| run | boundary | canvases | canvas MB | largest (MB) |',
    '| --- | --- | --- | --- | --- |',
    ...rows,
    '',
  ];
}

function topAllocations(profile) {
  const bySite = new Map();
  // A stack, not recursion: a long run's profile nests thousands of frames deep. `head` is absent when
  // sampling never started, and `selfSize`/`callFrame` are absent on synthetic roots — both defaulted.
  const stack = profile?.head ? [profile.head] : [];
  while (stack.length) {
    const node = stack.pop();
    const { functionName = '', url = '', lineNumber = -1 } = node.callFrame ?? {};
    const site = `${functionName || '(anonymous)'} ${shortUrl(url)}:${lineNumber + 1}`;
    bySite.set(site, (bySite.get(site) ?? 0) + (node.selfSize ?? 0));
    if (node.children) stack.push(...node.children);
  }
  return [...bySite.entries()]
    .sort((a, b) => b[1] - a[1])
    .slice(0, 20)
    .map(([site, bytes]) => ({ site, mb: mb(bytes) }));
}

/** Frames per second, second by second, from the raw frame timestamps. */
function perSecondFps(own) {
  if (!own.length) return [];
  const from = own[0][0];
  const buckets = new Map();
  for (const [t] of own) buckets.set(Math.floor((t - from) / 1000), (buckets.get(Math.floor((t - from) / 1000)) ?? 0) + 1);
  return [...buckets.values()];
}

/** Drains the page's frame buffer into `frames` (tagged with the stage) and returns the batch. */
async function drainFrames(page, frames, stage) {
  let batch = [];
  try {
    batch = await page.evaluate(() => {
      const f = window.__frames ?? [];
      window.__frames = [];
      return f;
    });
  } catch {
    return [];
  }
  for (const [t, gap] of batch) frames.push([t, gap, stage]);
  return batch;
}

/** The stage a resource timing entry began in (by the page's clock marks). */
function stageOfResource(entry, timeline) {
  // (The page's own marks: startup ends at the menu, entry at the game. Resource start times are page ms,
  // and the timeline is wall ms from launch; compare against the page's navigation origin.)
  if (!timeline.pageMarks) return 'startup';
  if (entry.start < timeline.pageMarks.menu) return 'startup';
  if (entry.start < timeline.pageMarks.entered) return 'entry';
  return 'steady';
}

const shortUrl = (u) => {
  try {
    const x = new URL(u);
    return `${x.pathname}${x.search}`.slice(0, 120);
  } catch {
    return String(u).slice(0, 120);
  }
};

const countBy = (list, key) => {
  const out = {};
  for (const item of list) out[key(item)] = (out[key(item)] ?? 0) + 1;
  return out;
};

// ---------------------------------------------------------------- environment

const environment = {
  machine: sh('sysctl', ['-n', 'hw.model']),
  cpu: sh('sysctl', ['-n', 'machdep.cpu.brand_string']),
  cores: cpus().length,
  ramGB: +(Number(sh('sysctl', ['-n', 'hw.memsize'])) / 2 ** 30).toFixed(1),
  os: `macOS ${sh('sw_vers', ['-productVersion'])} (${sh('sw_vers', ['-buildVersion'])})`,
  gpu: (sh('system_profiler', ['SPDisplaysDataType']).match(/Chipset Model:\s*(.+)/)?.[1] ?? '').trim(),
  display: (sh('system_profiler', ['SPDisplaysDataType']).match(/Resolution:\s*(.+)/)?.[1] ?? '').trim(),
  bun: sh('bun', ['--version']),
  node: process.version,
  commit: sh('git', ['-C', ROOT, 'rev-parse', 'HEAD']),
  dirty: sh('git', ['-C', ROOT, 'status', '--porcelain']).length > 0,
  serverMode,
  serverReadyMs: serverReadyAt - STARTED,
  runType: warm ? 'warm' : 'cold',
  flags: { runs, secs, seed, household: pick, preset, renderer, hud, warm, headed: has('headed'), startupTimeout, entryTimeout, steadyGrace },
};

// ---------------------------------------------------------------- the runs

const { chromium } = await playwright();
const executablePath = process.env.CHROME ?? cachedChromium();
if (!executablePath) throw new Error('no Chromium found (set CHROME, or install Playwright browsers)');
const summaries = [];
try {
  for (let n = 1; n <= runs; n++) {
    console.log(`run ${n}/${runs}: ${warm ? 'warm' : 'cold'}, ${preset}/${renderer}, ${secs} s steady …`);
    const summary = await runOnce(n, chromium, executablePath);
    summaries.push(summary);
    const st = summary.stages.steady;
    console.log(
      `  startup ${summary.timeline.startupMs} ms · entry ${summary.timeline.entryMs} ms · ${summary.samples} samples / ${(summary.timeline.steadyMs / 1000).toFixed(0)} s`,
    );
    console.log(
      `  steady: ${st.frames} frames, gap p50 ${st.frameGapMs?.median} ms p95 ${st.frameGapMs?.p95} ms p99 ${st.frameGapMs?.p99} ms · gpu ${st.gpuMs?.mean ?? 'n/a'} ms · task ${st.taskBusyPct?.mean ?? 'n/a'} % · renderer RSS ${summary.memory.rendererStartMB}→${summary.memory.rendererEndMB} MB`,
    );
  }
} finally {
  server?.kill();
}

// ---------------------------------------------------------------- the report

const env = environment;
const rows = [];
const metricOrder = [
  ['frameGapMs', 'frame gap', 'ms'],
  ['fps', 'frames/s (counted)', ''],
  ['rendererFps', 'renderer fps', ''],
  ['rendererFrameMs', 'renderer frame', 'ms'],
  ['rendererCpuMs', 'main-thread frame work', 'ms'],
  ['gpuMs', 'GPU frame', 'ms'],
  ['drawCalls', 'draw calls', ''],
  ['taskBusyPct', 'main-thread busy', '%'],
  ['scriptPct', 'script', '%'],
  ['layoutPct', 'layout', '%'],
  ['treeCpuPct', 'browser tree CPU', '%'],
  ['gpuCpuPct', 'GPU process CPU', '%'],
  ['rssRendererMB', 'renderer RSS', 'MB'],
  ['rssRenderersMB', 'renderers RSS (every renderer)', 'MB'],
  ['rssTreeMB', 'tree RSS (sum of per-process)', 'MB'],
  ['rssGpuMB', 'GPU process RSS', 'MB'],
  ['heapMB', 'JS heap', 'MB'],
  ['backingMB', 'JS external (ArrayBuffers)', 'MB'],
  ['heapTotalMB', 'V8 heap total', 'MB'],
  ['vramMB', 'GPU memory (VRAM)', 'MB'],
  ['vramTexMB', 'GPU memory: textures', 'MB'],
  ['simMemoryMB', 'sim WASM memory', 'MB'],
  ['domNodes', 'DOM nodes', ''],
];
for (const s of ['startup', 'entry', 'steady']) {
  for (const [key, label, unit] of metricOrder) {
    const perRun = summaries.map((x) => x.stages[s]?.[key]).filter(Boolean);
    if (!perRun.length) continue;
    // (One row per run; the table names the run so a comparison is read line by line.)
    for (const [i, st] of perRun.entries()) {
      rows.push(`| ${label}${unit ? ` (${unit})` : ''} | ${s} | run ${i + 1} | ${st.min} | ${st.mean} | ${st.median} | ${st.p95} | ${st.p99} | ${st.max} | ${st.n} |`);
    }
  }
}

const memoryRows = [];
for (const [i, x] of summaries.entries()) {
  const series = x.memory.series;
  for (let t = 0; t < series.length; t += 30) {
    const s = series[t];
    memoryRows.push(`| run ${i + 1} | ${s.t.toFixed(0)} | ${s.rendererMB} | ${s.treeMB} | ${s.gpuMB} | ${s.heapMB} | ${s.domNodes} |`);
  }
  const s = series[series.length - 1];
  memoryRows.push(`| run ${i + 1} | ${s.t.toFixed(0)} (end) | ${s.rendererMB} | ${s.treeMB} | ${s.gpuMB} | ${s.heapMB} | ${s.domNodes} |`);
}

/** Metrics whose run 1 and run 2 means differ by more than `pct`. */
function stability() {
  if (summaries.length < 2) return ['(one run: nothing to compare)'];
  const lines = [];
  for (const s of ['steady']) {
    for (const [key, label, unit] of metricOrder) {
      const a = summaries[0].stages[s]?.[key];
      const b = summaries[1].stages[s]?.[key];
      if (!a || !b || !a.mean || !b.mean) continue;
      const d = ((b.mean - a.mean) / a.mean) * 100;
      const dp99 = a.p99 ? ((b.p99 - a.p99) / a.p99) * 100 : 0;
      lines.push(`| ${label}${unit ? ` (${unit})` : ''} | ${a.mean} | ${b.mean} | ${d >= 0 ? '+' : ''}${d.toFixed(1)}% | ${a.p99} | ${b.p99} | ${dp99 >= 0 ? '+' : ''}${dp99.toFixed(1)}% |${Math.abs(d) > 10 || Math.abs(dp99) > 20 ? ' **unstable**' : ''}`);
    }
  }
  return lines;
}

const netLines = [];
for (const [i, x] of summaries.entries()) {
  for (const s of ['startup', 'entry', 'steady']) {
    const n = x.net.perStage[s];
    netLines.push(`| run ${i + 1} | ${s} | ${n.requests} | ${(n.encodedBytes / 1024).toFixed(0)} kB | ${(n.decodedBytes / 1024).toFixed(0)} kB | ${n.fromCache} | ${n.external} |`);
  }
}

const md = [
  `# Gameplay baseline: ${summaries.length} run${summaries.length === 1 ? '' : 's'}, ${preset}/${renderer}, seed ${seed}, ${secs} s steady`,
  '',
  `${env.machine}, ${env.cpu}, ${env.cores} cores, ${env.ramGB} GB RAM, ${env.gpu}, ${env.os}`,
  '',
  `commit \`${env.commit}\`${env.dirty ? ' (dirty working tree)' : ''} · Chromium ${summaries[0]?.browserVersion ?? '?'} · dev server ${env.serverMode} (ready ${env.serverReadyMs} ms after process start) · ${warm ? 'warm' : 'cold'} · 1280x720 @1 · quality ${preset} · HUD ${hud}`,
  '',
  '## Command',
  '',
  '```',
  `bun tools/bench/gameplay.mjs --runs ${runs} --secs ${secs} --seed ${seed} --preset ${preset} --renderer ${renderer} --hud ${hud}${warm ? ' --warm' : ''}${has('headed') ? ' --headed' : ''}`,
  '```',
  '',
  `Expected runtime: ~${Math.round((secs + 40) * runs / 60)} min (${secs} s steady + ~40 s startup/entry/shutdown per run).`,
  '',
  '## Stage timeline',
  '',
  '| run | launch | startup (launch → menu) | entry (Play → game) | steady | frames | samples |',
  '| --- | --- | --- | --- | --- | --- | --- |',
  ...summaries.map((x, i) => `| run ${i + 1} | ${x.timeline.launchMs} ms | ${x.timeline.startupMs} ms | ${x.timeline.entryMs} ms (stage ${x.timeline.entryStageMs} ms) | ${(x.timeline.steadyMs / 1000).toFixed(1)} s | ${x.stages.steady.frames} | ${x.samples} |`),
  '',
  'The game is up and interactive when `.hud-root` exists and the loading screen (`.loading`) is gone — the',
  'same signal `loading.mjs` and `profile.mjs` wait on — not a fixed sleep. The steady state must cover',
  `all ${secs} s: the run fails if it lasts less, overruns by more than ${steadyGrace} s, samples fewer than ${secs} seconds, or the frames span under 95% of it.`,
  '',
  '## Per-metric summary',
  '',
  '| metric | stage | run | min | mean | median | p95 | p99 | max | n |',
  '| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |',
  ...rows,
  '',
  '`frame gap` and `frames/s` come from the page\'s own `requestAnimationFrame` timestamps (raw kept);',
  '`renderer *` and `GPU frame` from `window.__gameStats()` (`?gpustats`, the renderer\'s own smoothed numbers);',
  '`main-thread busy`/`script`/`layout` from CDP `Performance.getMetrics` deltas; `browser tree CPU` and',
  '`GPU process CPU` from `ps` CPU-time deltas across the whole process tree (may exceed 100% across cores).',
  'Startup and entry have no 1 Hz samples: their frames and perf spans carry them.',
  '',
  '## Memory',
  '',
  'At each stage boundary (the menu up, the household entered, the window over):',
  '',
  '| run | boundary | renderer RSS (MB) | tree RSS (MB) | GPU proc RSS (MB) | JS heap (MB) | DOM nodes |',
  '| --- | --- | --- | --- | --- | --- | --- |',
  ...summaries.flatMap((x, i) => x.memory.boundaries.map((b) => `| run ${i + 1} | ${b.stage} | ${b.rendererRssMB ?? '—'} | ${b.treeRssMB ?? '—'} | ${b.gpuRssMB ?? '—'} | ${b.heapMB ?? '—'} | ${b.domNodes ?? '—'} |`)),
  '',
  'Through the steady state (every 30 s):',
  '',
  '| run | t (s) | renderer RSS (MB) | tree RSS (MB) | GPU proc RSS (MB) | JS heap (MB) | DOM nodes |',
  '| --- | --- | --- | --- | --- | --- | --- |',
  ...memoryRows,
  '',
  ...summaries.map(
    (x, i) =>
      `run ${i + 1}: renderer ${x.memory.rendererStartMB} → ${x.memory.rendererEndMB} MB (peak ${x.memory.rendererPeakMB}, ${x.memory.rendererGrowthMB >= 0 ? '+' : ''}${x.memory.rendererGrowthMB} MB), tree ${x.memory.treeStartMB} → ${x.memory.treeEndMB} MB (peak ${x.memory.treePeakMB}), GPU proc ${x.memory.gpuStartMB} → ${x.memory.gpuEndMB} MB, heap ${x.memory.heapStartMB} → ${x.memory.heapEndMB} MB (peak ${x.memory.heapPeakMB}), DOM nodes ${x.memory.domNodesStart} → ${x.memory.domNodesEnd}.`,
  ),
  '',
  '**How to read it.** The renderer\'s RSS is the primary number: the tab that runs the game. The *tree RSS*',
  'column is the plain sum of every Chromium process\'s RSS, so shared pages (the framework, the code, the',
  'GPU buffers) are counted once per process and the figure is a ceiling, not a footprint: do not compare it',
  'with Activity Monitor\'s total. `ps` RSS also over-counts a process\'s own shared and file-backed pages.',
  'The JS heap is V8\'s `JSHeapUsedSize` as CDP reports it, unforced (no GC before the read), so it includes',
  'garbage waiting for a collection. `phys_footprint` (from `/usr/bin/footprint`, which suspends the process',
  'for a few hundred ms while it reads it, hence only at the edges of the window) is the number Activity',
  'Monitor would show for the renderer:',
  '',
  ...summaries.map(
    (x, i) =>
      `run ${i + 1}: renderer phys_footprint ${x.memory.footprintStart?.available ? `${x.memory.footprintStart.mb} → ${x.memory.footprintEnd?.mb} MB` : `unavailable (${x.memory.footprintStart?.reason ?? 'not taken'})`}.`,
  ),
  '',
  ...censusSection(summaries),
  '## GPU memory and the simulation',
  '',
  '`GPU memory` is PlayCanvas\'s own counter for what the device holds (textures, vertex, index, uniform and',
  'storage buffers); `textures` is the share of it textures take (asset textures and render targets alike).',
  '`sim WASM` is the sim worker\'s linear memory as the worker reports it once a second: the main thread',
  'cannot read another thread\'s memory, so this is the only way to see it. `renderers RSS` is every renderer',
  'process summed (a worker can be a process of its own, and then it is not the largest renderer).',
  '',
  '| run | boundary | GPU memory (MB) | textures (MB) | sim WASM (MB) | renderers RSS (MB) |',
  '| --- | --- | --- | --- | --- | --- |',
  ...summaries.flatMap((x, i) => x.memory.boundaries.map((b) => `| run ${i + 1} | ${b.stage} | ${b.vramMB ?? '—'} | ${b.vramTexMB ?? '—'} | ${b.simMemoryMB ?? '—'} | ${b.renderersRssMB ?? '—'} |`)),
  '',
  ...summaries.map(
    (x, i) =>
      `run ${i + 1}: GPU memory ${x.memory.vramStartMB ?? '—'} → ${x.memory.vramEndMB ?? '—'} MB (peak ${x.memory.vramPeakMB ?? '—'}; textures peak ${x.memory.vramTexPeakMB ?? '—'}, geometry peak ${x.memory.vramGeomPeakMB ?? '—'}), sim WASM ${x.memory.simMemoryStartMB ?? '—'} → ${x.memory.simMemoryEndMB ?? '—'} MB (peak ${x.memory.simMemoryPeakMB ?? '—'}), renderers ${x.memory.renderersStartMB ?? '—'} → ${x.memory.renderersEndMB ?? '—'} MB (peak ${x.memory.renderersPeakMB ?? '—'}).`,
  ),
  '',
  ...(summaries.some((x) => x.heapSites?.length)
    ? [
        '## Heap profile (what the window allocated and kept)',
        '',
        'Sampled at 32 kB through the steady window (`--heapprofile`); self size by call site, top 20. Sampling costs frame time, so read these against a run without it.',
        '',
        '| run | site | MB |',
        '| --- | --- | --- |',
        ...summaries.flatMap((x, i) => (x.heapSites ?? []).map((r) => `| run ${i + 1} | \`${r.site}\` | ${r.mb} |`)),
        '',
      ]
    : []),
  '## Disk and network',
  '',
  '| run | stage | requests | encoded | decoded | from cache | external |',
  '| --- | --- | --- | --- | --- | --- | --- |',
  ...netLines,
  '',
  `External (non-localhost) requests: ${summaries.map((x) => `run ${x.run}: ${x.net.external}`).join(', ')}.${summaries.every((x) => x.net.external === 0) ? ' The game made none at runtime.' : ` URLs: ${summaries.flatMap((x) => x.net.externalUrls).join(', ')}`}`,
  '',
  'Disk bytes read/written by the processes are **not measured**: macOS has no per-process I/O counters',
  'without root (`fs_usage`/`iotop` need sudo). What is reported instead: the browser\'s wire bytes per',
  'request (CDP `encodedDataLength`), the page\'s resource timings (`decodedBodySize`, i.e. server response',
  'bytes after decoding), and cache hits.',
  '',
  '## Stability: run 1 vs run 2',
  '',
  summaries.length < 2
    ? '(only one run)'
    : ['| metric | run 1 mean | run 2 mean | Δ | run 1 p99 | run 2 p99 | Δ p99 |', '| --- | --- | --- | --- | --- | --- | --- |', ...stability()].join('\n'),
  '',
  'Marked **unstable** where the means differ by more than 10% or the p99s by more than 20%.',
  '',
  '## What could not be fixed or measured',
  '',
  '- GPU utilisation (%): not exposed by the browser. GPU frame time (`RenderStats.gpuMs`) and the GPU',
  '  process\'s CPU time are reported instead.',
  '- Disk I/O per process: needs root on macOS (above).',
  '- The display: the host\'s refresh rate and compositor still schedule frames; a 60 Hz display caps at',
  '  60 fps and the game paces to it (`refreshDivisor`). Headless uses the same GPU but no vsync of a',
  '  visible window, so frame pacing can differ from a headed run (`--headed`).',
  '- Other load on the machine (other browsers, builds) is not controlled; the two runs are sequential,',
  '  not isolated.',
  '- Chromium version and flags are recorded; the page\'s own settings are seeded and fixed, but',
  '  `Math.random` seeding covers the game\'s generation only, not the browser\'s internals.',
  '',
  '## Raw output',
  '',
  ...summaries.map((x) => `- run ${x.run}: \`${x.raw}\`, \`${join(out, `run${x.run}`, 'summary.json')}\``),
  '',
];
writeFileSync(join(out, 'report.md'), md.join('\n'));
writeFileSync(join(out, 'summary.json'), JSON.stringify({ environment, runs: summaries }, null, 2));
console.log(`\nwritten to ${out}`);
console.log(`  report.md · summary.json · ${summaries.map((x) => `run${x.run}/raw.jsonl`).join(' · ')}`);
