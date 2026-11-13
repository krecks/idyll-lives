#!/usr/bin/env bun
/**
 * Times the three waits a new player has before playing, through the menus as a player goes:
 *
 * - boot: the page opened to the main menu (the loading screen gone);
 * - inView, portraits: step 2 ("Choose a household", after "New game" and step 1) opened to the
 *   portraits in view in its list shown, and to every household's (`.households .face img`);
 * - entry: "Play" on a household to the game up (its interface there, the loading screen gone).
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

/** One visit: boot, step 2's portraits, a household's entry (ms each), and the page's spans. */
async function visit(page) {
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
  clearInterval(polling);
  await keep();
  const report = { entries: [...entries.values()].sort((a, b) => a.start - b.start), marks: { boot, opened, shown, pressed, entered } };
  return { boot, inView: inView - opened, portraits: shown - opened, faces, entry: entered - pressed, report };
}

const results = [];
let retried = 0;
for (let r = 0; r < runs; r++) {
  const context = await browser.newContext({ viewport: { width: 1920, height: 1080 }, deviceScaleFactor: 1 });
  await context.addInitScript(quiet);
  await context.addInitScript(seeded, seed);
  const page = await context.newPage();
  page.on('pageerror', (e) => console.error(`  page error: ${e.message}`));
  let visits;
  try {
    visits = [await visit(page)];
    if (again) visits.push(await visit(page));
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
  for (const k of ['boot', 'inView', 'portraits', 'entry']) md.push(`| ${k} | ${median(list.map((v) => v[k])).toFixed(0)} | ${list.map((v) => v[k].toFixed(0)).join(', ')} |`);
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
