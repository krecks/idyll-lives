/**
 * What the bench tools share: Playwright, Chromium's launch, a Vite server (dev or a preview of a build; a tree's, for
 * an A/B: `serveBuild`), a bench page
 * opened once and stepped through its poses (`window.__bench`, `web/src/debug/bench.ts`), and pages kept loaded in a
 * browser another process started (`keep.mjs`, `--attach`). The recipe for a script of one's own is in
 * tools/bench/README.md ("One page, many poses").
 */

import { spawn } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { createServer } from 'node:net';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { quiet } from '../quiet.mjs';
import { benchBuild } from './build.mjs';

export const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
export const WEB = join(ROOT, 'web/');

/** Playwright: `PLAYWRIGHT=<path to playwright/index.mjs>`, else the one web/ resolves (`playwright-core`: the same API). */
export async function playwright() {
  if (process.env.PLAYWRIGHT) return import(process.env.PLAYWRIGHT);
  const require = createRequire(WEB + 'package.json');
  for (const name of ['playwright', 'playwright-core']) {
    try {
      return await import(require.resolve(name));
    } catch {}
  }
  throw new Error('Playwright not found: install it, or set PLAYWRIGHT=<path to playwright/index.mjs>');
}

/** The window every bench page has: 1920 × 1080 at device pixel ratio 1. */
export const VIEWPORT = { viewport: { width: 1920, height: 1080 }, deviceScaleFactor: 1 };
/** What a kept browser clears before each page: everything the site stores, not the HTTP or shader caches. */
export const SITE_STORAGE = 'cookies,file_systems,indexeddb,local_storage,service_workers,cache_storage';

/** Chromium's arguments for the bench: WebGPU, full-precision GPU timestamps, drawing on in windows nobody looks at; `throughput`: no vsync, no frame cap. */
export function launchArgs({ throughput = false, debugPort = null } = {}) {
  return [
    '--enable-unsafe-webgpu',
    '--enable-webgpu-developer-features',
    '--use-angle=metal',
    '--disable-backgrounding-occluded-windows',
    '--disable-renderer-backgrounding',
    '--disable-background-timer-throttling',
    ...(throughput ? ['--disable-gpu-vsync', '--disable-frame-rate-limit'] : []),
    ...(debugPort ? [`--remote-debugging-port=${debugPort}`] : []),
  ];
}

/**
 * A browser for the bench: `{ context }` with a profile kept in `profileDir` (its HTTP cache and compiled shaders carry
 * over), or `{ browser }` with nothing kept (`profileDir` null). Headless is Chromium's new headless mode, on the real GPU.
 */
export async function launch({ headless = true, throughput = false, profileDir = null, debugPort = null } = {}) {
  const { chromium } = await playwright();
  const options = {
    headless,
    ...(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chromium' }),
    args: launchArgs({ throughput, debugPort }),
  };
  if (!profileDir) return { browser: await chromium.launch(options) };
  const context = await chromium.launchPersistentContext(profileDir, { ...options, ...VIEWPORT });
  await context.addInitScript(quiet);
  return { context };
}

/** A new context in a browser `launch` gave without a profile: nothing kept, the page's console quiet; `options` its window (`VIEWPORT`). */
export async function benchContext(browser, options = VIEWPORT) {
  const context = await browser.newContext(options);
  await context.addInitScript(quiet);
  return context;
}

/**
 * A browser another process keeps (`keep.mjs`): `attach` is its DevTools port, a `ws://` endpoint, or `keep` (the
 * endpoint `keep.mjs` wrote to `bench-results/.keep.json`). Its pages are already loaded: `openBench` reuses them.
 */
export async function attach(to) {
  const { chromium } = await playwright();
  const endpoint = to === 'keep' || to === true ? keepInfo().endpoint : /^\d+$/.test(String(to)) ? `http://127.0.0.1:${to}` : to;
  const browser = await chromium.connectOverCDP(endpoint);
  return { browser, context: browser.contexts()[0], attached: true };
}

/** What `keep.mjs` keeps: its endpoint and its builds (`{ name, where, url }`). */
export function keepInfo() {
  const file = join(ROOT, 'bench-results', '.keep.json');
  if (!existsSync(file)) throw new Error('no kept browser (bench-results/.keep.json): start tools/bench/keep.mjs first');
  return JSON.parse(readFileSync(file, 'utf8'));
}

/** A free TCP port on this machine (between `from` and `to` when given). */
export async function freePort(from = 0, to = from) {
  for (let port = from; port <= to; port++) {
    const free = await new Promise((resolve) => {
      const srv = createServer();
      srv.once('error', () => resolve(null));
      srv.listen(port, '127.0.0.1', () => {
        const { port: p } = srv.address();
        srv.close(() => resolve(p));
      });
    });
    if (free) return free;
  }
  throw new Error(`no free port in ${from}–${to}`);
}

/**
 * Vite for `web` (a tree's web/; this one's by default) on `port`: the dev server, or `vite preview` of `outDir` (a
 * build: `lib/build.mjs`). Resolves `{ url, stop }` once it says where it listens.
 */
export async function serve({ mode = 'dev', port, web = WEB, outDir = null } = {}) {
  port ??= await freePort();
  const args = [...(mode === 'preview' ? ['preview', ...(outDir ? ['--outDir', outDir] : [])] : []), '--port', String(port), '--strictPort', '--host', '127.0.0.1'];
  const vite = spawn(join(web, 'node_modules/.bin/vite'), args, { cwd: web, stdio: ['ignore', 'pipe', 'inherit'] });
  const stop = () => vite.kill();
  process.on('exit', stop);
  let limit;
  const url = await new Promise((resolve, reject) => {
    vite.stdout.on('data', (d) => String(d).includes('Local') && resolve(`http://127.0.0.1:${port}`));
    vite.on('exit', (code) => reject(new Error(`vite exited (${code})`)));
    limit = setTimeout(() => reject(new Error('vite did not report "Local" in 120 s')), 120_000);
  }).finally(() => clearTimeout(limit));
  return { url, stop, port };
}

/** Whether a server answers at `url` (within 1.5 s). */
export async function answers(url) {
  try {
    return (await fetch(url, { signal: AbortSignal.timeout(1500) })).ok;
  } catch {
    return false;
  }
}

/**
 * The dev server a tool's pages come from: `url` as it is, else the one already answering on `port`, else one
 * started there (`serve`). Resolves `{ url, stop, started }` (`stop` does nothing to a server this didn't start).
 */
export async function devServer({ url = null, port }) {
  if (url) return { url: url.replace(/\/$/, ''), stop: () => {}, started: false };
  const there = `http://localhost:${port}`;
  if (await answers(there)) return { url: there, stop: () => {}, started: false };
  const { url: started, stop } = await serve({ port });
  return { url: started, stop, started: true };
}

/**
 * A build to measure: `where` a running server's URL, used as it is, or a tree (a checkout or a worktree, its wasm
 * built) served by its own Vite on `port` (a free one when not given): its dev server, or with `prod` its build for
 * the benchmark (`lib/build.mjs`, made on a miss) under `vite preview`. Resolves `{ where, url, stop }`, `where` the
 * tree's full path.
 */
export async function serveBuild(where, { prod = false, port } = {}) {
  if (/^https?:\/\//.test(where)) return { where, url: where.replace(/\/$/, ''), stop: () => {} };
  const tree = resolve(where);
  const web = join(tree, 'web');
  if (!existsSync(join(web, 'node_modules/.bin/vite'))) throw new Error(`${tree}: no web/node_modules (link the main checkout's)`);
  if (!existsSync(join(web, 'src/core/wasm-pkg/sim_wasm_bg.wasm'))) throw new Error(`${tree}: its wasm isn't built (bun run wasm and bun run sound in its web/)`);
  const outDir = prod ? (await benchBuild({ tree })).dir : null;
  const { url, stop } = await serve({ mode: prod ? 'preview' : 'dev', port, web, outDir });
  return { where: tree, url, stop };
}

/** The flags that make one bench page another (the scene, the build's settings): the rest a page changes as it goes. */
const PAGE_KEYS = ['bench', 'quality', 'renderer', 'res', 'diag', 'big', 'options', 'tile', 'hud'];

/** A page's identity among kept pages: its origin and `PAGE_KEYS` (the diagnostics as a set). */
export function pageKey(url) {
  const u = new URL(url);
  const p = u.searchParams;
  return [u.origin, ...PAGE_KEYS.map((k) => `${k}=${k === 'diag' ? (p.get(k) ?? '').split(',').filter(Boolean).sort().join(',') : (p.get(k) ?? '')}`)].join('&');
}

/**
 * A bench page: `base` + `/?` + `query` (`bench=<scene>&quality=…`), waited for `window.__benchReady`. In a kept
 * browser (`attached`) a page already open with the same scene and settings is used as it is (its `__bench` reset when
 * done); otherwise a new page, the site's storage cleared first (each a new player's). `done()` closes or resets it.
 */
export async function openBench(context, base, query, { attached = false, timeoutMs = 240_000, path = '/', key = '__benchReady', newPage = null } = {}) {
  const url = `${base}${path}?${query}`;
  if (attached) {
    const want = pageKey(url);
    const kept = context.pages().find((p) => p.url().startsWith(base) && pageKey(p.url()) === want);
    if (kept) {
      await kept.waitForFunction((k) => !!window[k] && !!window.__bench, key, { timeout: timeoutMs, polling: 250 });
      // (Kept pages draw nothing while nobody uses them: the one measured has the GPU to itself.)
      await kept.evaluate(() => window.__bench.idle(false));
      return {
        page: kept,
        kept: true,
        done: () =>
          kept.evaluate(async () => {
            await window.__bench.reset();
            window.__bench.idle(true);
          }),
      };
    }
  }
  const page = newPage ? await newPage() : await context.newPage();
  const done = () => page.close();
  try {
    const cdp = await context.newCDPSession(page);
    await cdp.send('Storage.clearDataForOrigin', { origin: new URL(base).origin, storageTypes: SITE_STORAGE });
    await cdp.detach();
    page.on('pageerror', (e) => console.error(`  page error: ${e.message}`));
    page.on('console', (m) => m.type() === 'error' && console.error(`  page: ${m.text().slice(0, 300)}`));
    await page.goto(url);
    await page.waitForFunction((k) => !!window[k], key, { timeout: timeoutMs, polling: 250 });
  } catch (err) {
    await done();
    throw err;
  }
  return { page, kept: false, done };
}

/**
 * A pose as the bench's URL flags spell it (`cam=over&over=centre&radius=400&hour=13`, render.mjs's `POSES`): the hour
 * to light it at (undefined: as it is) and the pose for `__bench.pose`.
 */
export function poseOf(query) {
  const p = new URLSearchParams(query);
  const num = (k) => (p.has(k) ? Number(p.get(k)) : undefined);
  const pose = { cam: p.get('cam') ?? 'orbit', at: num('at'), radius: num('radius'), beta: num('beta'), alpha: num('alpha'), tx: num('tx'), tz: num('tz'), over: p.get('over') === 'centre' ? 'centre' : 'home', walls: p.get('walls'), townCentre: p.get('townCentre') === '1' };
  return { hour: p.has('hour') ? Number(p.get('hour')) : undefined, pose };
}

/**
 * Steps a ready bench page through `poses` (each `{ name, query }`, or `{ name, hour, pose }`): the hour where it
 * changes (none given: the game's clock), the pose, `__bench.settle()`, then `fn(page, pose)` (a screenshot,
 * `__bench.counts(600)`, …). `before`: run first (the pose the page opened at, already settled). The poses are taken
 * one after another in the one page: no reload, no fixed sleeps.
 */
export async function step(page, poses, fn, { limitMs = 60_000, before = null } = {}) {
  if (before) await before();
  let hour = await page.evaluate(() => new URLSearchParams(location.search).get('hour'));
  hour = hour === null ? null : Number(hour);
  for (const p of poses) {
    const { hour: h = null, pose } = p.query !== undefined ? poseOf(p.query) : p;
    if (h !== hour) {
      await page.evaluate((x) => window.__bench.hour(x), h);
      hour = h;
    }
    await page.evaluate((x) => window.__bench.pose(x), pose);
    await page.evaluate((ms) => window.__bench.settle({ limitMs: ms }), limitMs);
    await fn(page, p);
  }
}

/** Two more frames drawn (what was placed a frame after the camera, then the picture). */
export const twoFrames = (page) => page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));

/**
 * In a dev page showing the menus' town (`page.evaluate(settleShot, { hook, minMs })`, `window[hook].menuScene`): the
 * picture ready, instead of a fixed wait: the camera's glide over (`menuScene.settled`), the town's model files and
 * textures there, three animation frames and at least `minMs` in all. Resolves the time each part took (ms).
 */
export async function settleShot({ hook, minMs }) {
  const t = [performance.now()];
  const [{ host }, { settings }] = await Promise.all([import('/src/game/host.ts'), import('/src/settings/settings.svelte.ts')]);
  await window[hook].menuScene.settled();
  t.push(performance.now());
  const renderer = await host.rendererNow(settings);
  await renderer.texturesReady();
  t.push(performance.now());
  // (Animation frames, not the renderer's: the menus' town holds its picture while nothing in it moves, and draws none.)
  for (let k = 0; k < 3; k++) await new Promise((resolve) => requestAnimationFrame(resolve));
  // A frame drawn just after a cut is softer for about a second: the shots always waited this long.
  await new Promise((resolve) => setTimeout(resolve, Math.max(0, t[0] + minMs - performance.now())));
  t.push(performance.now());
  return t.slice(1).map((x, i) => Math.round(x - t[i]));
}
