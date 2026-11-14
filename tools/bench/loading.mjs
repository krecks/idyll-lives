#!/usr/bin/env bun
/**
 * Times the three waits a new player has before playing, through the menus as a player goes:
 *
 * - boot: the page opened to the main menu (the loading screen gone);
 * - inView, portraits: step 2 ("Choose a household", after "New game" and step 1) opened to the
 *   portraits in view in its list shown, and to every household's (`.households .face img`);
 * - entry: "Play" on a household to the game up (its interface there, the loading screen gone).
 *
 * With `--back`, then the way back and in again: the long tasks in the 5 s after the entry (`entryLong`),
 * "Save and quit" from the menu shown to its live town on screen (`back`; the menus' town was the new
 * game's, so the main menu's is built), Continue (once the latest save has been prepared behind the
 * menu) to the game up (`continue`, and `continueLong` after it), the heap in that game (CDP, after a
 * collection: `heapMB`, its array buffers `buffersMB`), and "Quit without saving" from the menu shown
 * to its live town (`again`, with the long tasks meanwhile, `againLong`).
 *
 * Each run is a new browser context (nothing kept: a first visit); `--again` also opens the page a
 * second time in the same context (a returning player: what the browser kept between visits). The
 * page's `Math.random` is seeded (`--seed`), so the showcase, the world of step 1 and its households are
 * the same in every run and in every build measured. The page's spans (`window.__perfReport()`, with
 * `?profile`) are kept per run; they are the dev play log's `perf` lines too.
 *
 *   bun tools/bench/loading.mjs [options]
 *     --runs 3                 runs (medians reported)
 *     --again                  a second visit per run, in the same context
 *     --back                   the way back to the menu and in again (above)
 *     --seed 7                 the page's Math.random seed
 *     --household 0            which household's Play is pressed (its index in the list)
 *     --renderer webgpu        or webgl
 *     --headed                 in a window (default headless: the real GPU either way)
 *     --port 5222 | --url <u>  a dev server on the port (started if none answers), or one already running
 *     --out <dir>              default bench-results/loading-<time>/
 */
import { spawn } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { quiet } from './quiet.mjs';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const WEB = join(ROOT, 'web/');
const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};
const has = (name) => args.includes(`--${name}`);

const runs = Number(flag('runs', '3'));
const seed = Number(flag('seed', '7'));
const pick = Number(flag('household', '0'));
const renderer = flag('renderer', 'webgpu');
const port = Number(flag('port', '5222'));
const again = has('again');
const back = has('back');
const stamp = new Date().toISOString().replace(/\.\d+Z$/, '').replace(/[:T]/g, '-');
const out = flag('out', join(ROOT, 'bench-results', `loading-${stamp}`));
mkdirSync(out, { recursive: true });

async function playwright() {
  if (process.env.PLAYWRIGHT) return import(process.env.PLAYWRIGHT);
  try {
    return await import(createRequire(WEB + 'package.json').resolve('playwright'));
  } catch {
    throw new Error('Playwright not found: install it, or set PLAYWRIGHT=<path to playwright/index.mjs>');
  }
}

async function running(url) {
  try {
    return (await fetch(url, { signal: AbortSignal.timeout(1500) })).ok;
  } catch {
    return false;
  }
}

let server = null;
const base = flag('url', null) ?? `http://localhost:${port}`;
if (!flag('url', null) && !(await running(base))) {
  server = spawn(`${WEB}node_modules/.bin/vite`, ['--port', String(port), '--strictPort'], { cwd: WEB, stdio: ['ignore', 'pipe', 'inherit'] });
  process.on('exit', () => server.kill());
  await new Promise((resolve, reject) => {
    server.stdout.on('data', (d) => String(d).includes('Local') && resolve());
    server.on('exit', (code) => reject(new Error(`vite exited (${code})`)));
  });
}

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

const { chromium } = await playwright();
const browser = await chromium.launch({
  headless: !has('headed'),
  ...(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chromium' }),
  args: ['--enable-unsafe-webgpu', '--disable-backgrounding-occluded-windows', '--disable-renderer-backgrounding', '--disable-background-timer-throttling'],
});

const query = ['profile', renderer === 'webgl' ? 'renderer=webgl' : ''].filter(Boolean).join('&');

/** The page's long tasks (start and duration, ms), from its first script on. */
function longTasks() {
  window.__longTasks = [];
  new PerformanceObserver((list) => list.getEntries().forEach((e) => window.__longTasks.push({ start: e.startTime, ms: e.duration }))).observe({ type: 'longtask', buffered: true });
}

/** One visit: boot, step 2's portraits, a household's entry (ms each), and the page's spans. */
async function visit(page, cdp) {
  const now = () => page.evaluate(() => performance.now());
  // (The page keeps its last 500 spans: read them after each moment, so none is lost.)
  const entries = new Map();
  const keep = async () => {
    for (const e of (await page.evaluate(() => window.__perfReport?.().entries).catch(() => null)) ?? []) entries.set(`${e.name}@${e.start}`, e);
  };
  const polling = setInterval(() => void keep(), 1000);
  await page.goto(`${base}/?${query}`);
  await page.waitForFunction(() => !!document.querySelector('.main-menu, .menu') && !document.querySelector('.loading'), null, { timeout: 180_000, polling: 100 });
  const boot = await now();
  await keep();
  // (The menu's entrance, and what the menu itself sets going, before "New game".)
  await page.waitForTimeout(1500);
  await page.getByText('New game', { exact: true }).click();
  const next = page.getByRole('button', { name: /Choose a household/ });
  await next.waitFor({ timeout: 60_000 });
  await page.waitForFunction(() => [...document.querySelectorAll('button')].some((b) => /Choose a household/.test(b.textContent ?? '') && !b.disabled), null, { timeout: 60_000, polling: 100 });
  await page.waitForTimeout(500);
  await next.click();
  const opened = await now();
  // (The faces in view first: the list's rows that show without scrolling.)
  await page.waitForFunction(
    () => {
      const list = document.querySelector('.households .list');
      const faces = [...document.querySelectorAll('.households .face')];
      if (!list || !faces.length) return false;
      const box = list.getBoundingClientRect();
      const inView = faces.filter((f) => {
        const r = f.getBoundingClientRect();
        return r.bottom > box.top && r.top < box.bottom;
      });
      return inView.length > 0 && inView.every((f) => f.querySelector('img'));
    },
    null,
    { timeout: 120_000, polling: 50 },
  );
  const inView = await now();
  await page.waitForFunction(
    () => {
      const faces = document.querySelectorAll('.households .face');
      return faces.length > 0 && [...faces].every((f) => f.querySelector('img'));
    },
    null,
    { timeout: 120_000, polling: 50 },
  );
  const shown = await now();
  await keep();
  const faces = await page.evaluate(() => document.querySelectorAll('.households .face').length);
  await page.waitForTimeout(500);
  const plays = page.locator('.households .btn.play');
  const pressed = await now();
  await plays.nth(Math.min(pick, (await plays.count()) - 1)).click();
  await page.waitForFunction(() => !!document.querySelector('.hud-root') && !document.querySelector('.loading'), null, { timeout: 180_000, polling: 100 });
  const entered = await now();
  const marks = { boot, opened, shown, pressed, entered };
  const more = back ? await backAndIn(page, cdp, now, marks) : {};
  clearInterval(polling);
  await keep();
  const report = { entries: [...entries.values()].sort((a, b) => a.start - b.start), marks };
  return { boot, inView: inView - opened, portraits: shown - opened, faces, entry: entered - pressed, ...more, report };
}

/** `--back`: from the game to the menu and in again (see the header), ms each. */
async function backAndIn(page, cdp, now, marks) {
  /** The long tasks (ms) that began between `from` and `to`. */
  const long = (from, to) => page.evaluate(([a, b]) => window.__longTasks.filter((t) => t.start >= a && t.start < b).map((t) => Math.round(t.ms)), [from, to]);
  const inGame = () => page.waitForFunction(() => !!document.querySelector('.hud-root') && !document.querySelector('.loading'), null, { timeout: 180_000, polling: 100 });
  /** Leaves the game by the pause menu's `choice`: from the main menu shown to its live town (ms), and the long tasks meanwhile. */
  const leave = async (choice) => {
    await page.keyboard.press('Escape');
    await page.getByText('Quit to main menu', { exact: true }).click();
    await page.getByText(choice, { exact: true }).click();
    await page.waitForFunction(() => !!document.querySelector('.menu') && !document.querySelector('.pause'), null, { timeout: 60_000, polling: 20 });
    const menu = await now();
    await page.waitForFunction(() => !!document.querySelector('.menu') && !document.querySelector('.game-layer.hidden'), null, { timeout: 60_000, polling: 20 });
    const live = await now();
    return { ms: live - menu, long: await long(menu, live + 500) };
  };
  await page.waitForTimeout(5000);
  const entryLong = await long(marks.entered, marks.entered + 5000);
  const first = await leave('Save and quit');
  // (The latest save prepared behind the menu meanwhile: Continue as a player finds it.)
  await page.waitForTimeout(15_000);
  marks.continuePressed = await now();
  await page.locator('button.continue').click();
  await inGame();
  marks.continued = await now();
  await page.waitForTimeout(5000);
  const continueLong = await long(marks.continued, marks.continued + 5000);
  for (let i = 0; i < 3; i++) await cdp.send('HeapProfiler.collectGarbage');
  const heap = await cdp.send('Runtime.getHeapUsage');
  const second = await leave('Quit without saving');
  return {
    entryLong,
    back: first.ms,
    backLong: first.long,
    continue: marks.continued - marks.continuePressed,
    continueLong,
    heapMB: heap.usedSize / 1048576,
    buffersMB: (heap.backingStorageSize ?? 0) / 1048576,
    again: second.ms,
    againLong: second.long,
  };
}

const results = [];
let retried = 0;
for (let r = 0; r < runs; r++) {
  const context = await browser.newContext({ viewport: { width: 1920, height: 1080 }, deviceScaleFactor: 1 });
  await context.addInitScript(quiet);
  await context.addInitScript(seeded, seed);
  await context.addInitScript(longTasks);
  const page = await context.newPage();
  const cdp = back ? await context.newCDPSession(page) : null;
  page.on('pageerror', (e) => console.error(`  page error: ${e.message}`));
  let visits;
  try {
    visits = [await visit(page, cdp)];
    if (again) visits.push(await visit(page, cdp));
  } catch (err) {
    // (Headless Chromium's page sometimes crashes on a load with others running: that run again, once.)
    await context.close();
    if (retried >= 2) throw err;
    retried++;
    console.error(`  run ${r + 1} failed (${String(err).split('\n')[0]}): again`);
    r--;
    continue;
  }
  for (const [i, v] of visits.entries()) {
    const label = i === 0 ? 'first' : 'again';
    console.log(`run ${r + 1} (${label}): boot ${v.boot.toFixed(0)} ms · step-2 portraits in view ${v.inView.toFixed(0)} ms, all ${v.portraits.toFixed(0)} ms (${v.faces}) · entry ${v.entry.toFixed(0)} ms`);
    if (back) {
      const tasks = (xs) => (xs.length ? `${xs.length} long (max ${Math.max(...xs)} ms)` : 'no long tasks');
      console.log(`  after entry ${tasks(v.entryLong)} · back ${v.back.toFixed(0)} ms, ${tasks(v.backLong)} · continue ${v.continue.toFixed(0)} ms, then ${tasks(v.continueLong)} · heap ${v.heapMB.toFixed(0)} MB, buffers ${v.buffersMB.toFixed(0)} MB · again ${v.again.toFixed(0)} ms, ${tasks(v.againLong)}`);
    }
    writeFileSync(join(out, `run${r + 1}-${label}.json`), JSON.stringify({ query, seed, ...v }, null, 2));
    results.push({ label, ...v });
  }
  await context.close();
}

const median = (xs) => {
  const s = [...xs].sort((a, b) => a - b);
  return s.length ? (s.length % 2 ? s[(s.length - 1) / 2] : (s[s.length / 2 - 1] + s[s.length / 2]) / 2) : NaN;
};
/** A span's name, the warm-up's lots as one. */
const named = (name) => name.replace(/^warm: lot \d+ of \d+$/, 'warm: lots');
/** A span's median over the runs (summed where a run has it more than once), with how many times it came. */
const spans = (list, name) => {
  const per = list.map((v) => v.report?.entries.filter((e) => named(e.name) === name) ?? []);
  return { ms: median(per.map((es) => es.reduce((a, e) => a + e.ms, 0))), n: median(per.map((es) => es.length)) };
};
const md = [`# Loading: ${runs} runs, ${renderer}, seed ${seed}`, ''];
for (const label of ['first', 'again']) {
  const list = results.filter((v) => v.label === label);
  if (!list.length) continue;
  md.push(`## ${label === 'first' ? 'First visit' : 'Second visit'}`, '');
  md.push('| moment | median ms | runs |', '|---|---|---|');
  for (const k of ['boot', 'inView', 'portraits', 'entry', ...(back ? ['back', 'continue', 'heapMB', 'buffersMB', 'again'] : [])]) md.push(`| ${k} | ${median(list.map((v) => v[k])).toFixed(0)} | ${list.map((v) => v[k].toFixed(0)).join(', ')} |`);
  // (`--back`: the longest task in each stretch.)
  if (back) for (const k of ['entryLong', 'backLong', 'continueLong', 'againLong']) md.push(`| ${k} (longest) | ${median(list.map((v) => Math.max(0, ...v[k]))).toFixed(0)} | ${list.map((v) => Math.max(0, ...v[k])).join(', ')} |`);
  md.push('', '| span | median ms (summed) | count |', '|---|---|---|');
  const names = [...new Set(list.flatMap((v) => v.report?.entries.map((e) => named(e.name)) ?? []))];
  for (const name of names.filter((n) => /^(boot|session|host|warm|portraits|preview|studio|menu: showTown|render: nature)/.test(n))) {
    const s = spans(list, name);
    if (s.ms >= 5) md.push(`| ${name} | ${s.ms.toFixed(0)} | ${s.n} |`);
  }
  md.push('');
}
writeFileSync(join(out, 'report.md'), md.join('\n'));
console.log(`\n${md.join('\n')}\nwritten to ${out}`);
await browser.close();
server?.kill();
process.exit(0);
