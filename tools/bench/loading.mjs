#!/usr/bin/env bun
/**
 * Times the three waits a new player has before playing, through the menus as a player goes:
 *
 * - boot: the page opened to the main menu (the loading screen gone, its picture shown);
 * - world: "New game" to step 1's land made and built (its "Lay out the streets" enabled), in the
 *   countryside with its own options, as a new player gets it, unless `--world` and `--options` (or
 *   `--code`) say otherwise (the page's `?world=`, `?options=`, `?code=`);
 * - layout: "Lay out the streets" to step 2's layout proposed on that land and shown (its "Build the
 *   town" enabled);
 * - town: "Build the town" to step 3's town built from that layout and shown (its "Choose a household" enabled);
 * - inView, portraits: step 4 ("Choose a household") opened to the portraits in view on its cards
 *   shown, and to every card's (`.offer .face img`);
 * - home: "Play" on a household's card (or Creation's "Choose a home", `--create`) to step 5's way on
 *   enabled (a household of the town's has its own home chosen there);
 * - entry: step 5's "Move in" (or, for a household of the town's staying in its home, "Play the …s")
 *   to the game up (its interface there, the loading screen gone), the rest of the homes for sale
 *   filled on the way. Step 5 is reached by "Play" on a household's card, or with `--create` through
 *   Creation (its household as it comes, "Choose a home"), and its home as it opens. Then the residents
 *   in the game, the homes they live in, and the homes the game shows for sale;
 * - total: the page's first byte to the game up (`marks.entered`, the harness's waits between steps
 *   included).
 *
 * With `--back`, then the way back and in again: the long tasks in the 5 s after the entry (`entryLong`),
 * "Save and quit" from the menu shown to its picture on screen (`back`), Continue (once the latest save has been prepared behind the
 * menu) to the game up (`continue`, and `continueLong` after it), the heap in that game (CDP, after a
 * collection: `heapMB`, its array buffers `buffersMB`), and "Quit without saving" from the menu shown
 * to its picture (`again`, with the long tasks meanwhile, `againLong`).
 *
 * Each run is a new browser context (nothing kept: a first visit); `--again` also opens the page a
 * second time in the same context (a returning player: what the browser kept between visits). The
 * page's `Math.random` is seeded (`--seed`), so the world of step 1 and its households are
 * the same in every run and in every build measured. The page's spans (`window.__perfReport()`, with
 * `?profile`) are kept per run; they are the dev play log's `perf` lines too.
 *
 *   bun tools/bench/loading.mjs [options]
 *     --runs 3                 runs (medians reported)
 *     --again                  a second visit per run, in the same context
 *     --back                   the way back to the menu and in again (above)
 *     --seed 7                 the page's Math.random seed
 *     --world country          step 1's place (a kind from before places, such as dense, is its place at its size)
 *     --options town=city,villages=many   step 1's options (the place's own where not given)
 *     --code river-mhs-k2x9q   the world's code instead (a bare number: the land's seed)
 *     --household 0            which household's Play is pressed (its card's index)
 *     --create                 step 5 through Creation instead
 *     --renderer webgpu        or webgl
 *     --headed                 in a window (default headless: the real GPU either way)
 *     --port 5222 | --url <u>  a dev server on the port (started if none answers), or one already running
 *     --out <dir>              default bench-results/loading-<time>/
 *     --shots                  also a picture of step 5 once its way on is enabled, and of the game 3 s
 *                              after the entry (in `--out`, per run and renderer)
 */
import { spawn } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { quiet } from './quiet.mjs';
import { worldFlags } from './worldFlags.mjs';

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
// (None given: step 1 as a player gets it, the URL without them.)
const made = worldFlags(flag, null);
const code = flag('code', null);
const worldQuery = [made.world ? `world=${made.world}` : '', made.query ? `options=${made.query}` : '', code ? `code=${encodeURIComponent(code)}` : ''];
const port = Number(flag('port', '5222'));
const again = has('again');
const back = has('back');
const create = has('create');
const shots = has('shots');
/** The current visit's pictures' name: renderer, run and visit. */
let shotName = '';
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

const query = ['profile', renderer === 'webgl' ? 'renderer=webgl' : '', ...worldQuery].filter(Boolean).join('&');

/** The page's long tasks (start and duration, ms), from its first script on. */
function longTasks() {
  window.__longTasks = [];
  new PerformanceObserver((list) => list.getEntries().forEach((e) => window.__longTasks.push({ start: e.startTime, ms: e.duration }))).observe({ type: 'longtask', buffered: true });
}

/** A step's way on (`label`, a pattern) enabled: its world made and built. */
const enabled = (page, label) => page.waitForFunction((src) => [...document.querySelectorAll('button')].some((b) => new RegExp(src).test(b.textContent ?? '') && !b.disabled), label.source, { timeout: 120_000, polling: 100 });

/** One visit: boot, step 1's land, step 2's layout, step 3's town, step 4's portraits, a household's entry (ms each), and the page's spans. */
async function visit(page, cdp) {
  const now = () => page.evaluate(() => performance.now());
  // (The page keeps its last 500 spans: read them after each moment, so none is lost.)
  const entries = new Map();
  const keep = async () => {
    for (const e of (await page.evaluate(() => window.__perfReport?.().entries).catch(() => null)) ?? []) entries.set(`${e.name}@${e.start}`, e);
  };
  const polling = setInterval(() => void keep(), 1000);
  await page.goto(`${base}/?${query}`);
  await page.waitForFunction(() => !!document.querySelector('.menu') && !document.querySelector('.loading') && !!document.querySelector('.backdrop .still.loaded'), null, { timeout: 180_000, polling: 100 });
  const boot = await now();
  await keep();
  // (The menu's entrance, and what the menu itself sets going, before "New game".)
  await page.waitForTimeout(1500);
  const newGame = await now();
  await page.getByText('New game', { exact: true }).click();
  const toLayout = page.getByRole('button', { name: /Lay out the streets/ });
  await toLayout.waitFor({ timeout: 60_000 });
  await enabled(page, /Lay out the streets/);
  const made = await now();
  await page.waitForTimeout(500);
  const proposePressed = await now();
  await toLayout.click();
  const toTown = page.getByRole('button', { name: /Build the town/ });
  await toTown.waitFor({ timeout: 60_000 });
  await enabled(page, /Build the town/);
  const proposed = await now();
  await page.waitForTimeout(500);
  const layPressed = await now();
  await toTown.click();
  const next = page.getByRole('button', { name: /Choose a household/ });
  await next.waitFor({ timeout: 60_000 });
  await enabled(page, /Choose a household/);
  const laid = await now();
  await page.waitForTimeout(500);
  await next.click();
  const opened = await now();
  // (The faces in view first: the cards that show without scrolling.)
  await page.waitForFunction(
    () => {
      const list = document.querySelector('.offer .cards');
      const faces = [...document.querySelectorAll('.offer .face')];
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
      const faces = document.querySelectorAll('.offer .face');
      return faces.length > 0 && [...faces].every((f) => f.querySelector('img'));
    },
    null,
    { timeout: 120_000, polling: 50 },
  );
  const shown = await now();
  await keep();
  const faces = await page.evaluate(() => document.querySelectorAll('.offer .face').length);
  await page.waitForTimeout(500);
  const toHome = await now();
  if (create) {
    await page.getByRole('button', { name: /Create your own/ }).first().click();
    const choose = page.getByRole('button', { name: /Choose a home/ });
    await choose.waitFor({ timeout: 60_000 });
    await choose.click();
  } else {
    const plays = page.locator('.offer .card[data-home] .go');
    await plays.nth(Math.min(pick, (await plays.count()) - 1)).click();
  }
  // (Step 5's way on: "Move in", or "Play the …s" for a household of the town's staying in its home.)
  const moveIn = page.locator('.btn.move-in');
  await moveIn.waitFor({ timeout: 60_000 });
  await page.waitForFunction(() => !!document.querySelector('.btn.move-in:not(:disabled)'), null, { timeout: 120_000, polling: 100 });
  const home = (await now()) - toHome;
  await page.waitForTimeout(500);
  if (shots) await page.screenshot({ path: join(out, `step4-${shotName}.png`) });
  const pressed = await now();
  await moveIn.click();
  // (Fewer beds than sleepers: step 5 asks first, and is answered.)
  const asked = () => [...document.querySelectorAll('button')].some((b) => b.textContent?.trim() === 'Move in anyway');
  await page.waitForFunction(`(${asked})() || (!!document.querySelector('.hud-root') && !document.querySelector('.loading'))`, null, { timeout: 180_000, polling: 100 });
  if (await page.evaluate(asked)) await page.getByRole('button', { name: 'Move in anyway' }).click();
  await page.waitForFunction(() => !!document.querySelector('.hud-root') && !document.querySelector('.loading'), null, { timeout: 180_000, polling: 100 });
  const entered = await now();
  if (shots) {
    await page.waitForTimeout(3000);
    await page.screenshot({ path: join(out, `game-${shotName}.png`) });
  }
  // Who lives in the town as the game starts (the player's household among them), and in how many homes.
  // (And the homes the game shows for sale: its own list, as the town panel and the tags have it.)
  const people = await page.evaluate(async () => {
    const sims = window.__osw?.sims?.() ?? [];
    const { game } = await import('/src/ui/state.svelte.ts');
    const { homesForSale } = await import('/src/ui/town/places.ts');
    const { services } = await import('/src/ui/services.ts');
    const forSale = homesForSale(game.plots, game.households, services.content.economy.homePrice, game.flats).length;
    return { residents: sims.length, homes: new Set(sims.map((s) => s.plot)).size, forSale };
  });
  const marks = { boot, newGame, made, proposePressed, proposed, layPressed, laid, opened, shown, pressed, entered };
  const more = back ? await backAndIn(page, cdp, now, marks) : {};
  clearInterval(polling);
  await keep();
  const report = { entries: [...entries.values()].sort((a, b) => a.start - b.start), marks };
  // (The marks as page times too: the whole way from the page's first byte to the game up,
  // `marks.entered`, is reported as the total.)
  return { boot, world: made - newGame, layout: proposed - proposePressed, town: laid - layPressed, inView: inView - opened, portraits: shown - opened, faces, home, entry: entered - pressed, ...people, ...more, report, marks };
}

/** `--back`: from the game to the menu and in again (see the header), ms each. */
async function backAndIn(page, cdp, now, marks) {
  /** The long tasks (ms) that began between `from` and `to`. */
  const long = (from, to) => page.evaluate(([a, b]) => window.__longTasks.filter((t) => t.start >= a && t.start < b).map((t) => Math.round(t.ms)), [from, to]);
  const inGame = () => page.waitForFunction(() => !!document.querySelector('.hud-root') && !document.querySelector('.loading'), null, { timeout: 180_000, polling: 100 });
  /** Leaves the game by the pause menu's `choice`: from the main menu shown to its picture (ms), and the long tasks meanwhile. */
  const leave = async (choice) => {
    await page.keyboard.press('Escape');
    await page.getByText('Quit to main menu', { exact: true }).click();
    await page.getByText(choice, { exact: true }).click();
    await page.waitForFunction(() => !!document.querySelector('.menu') && !document.querySelector('.pause'), null, { timeout: 60_000, polling: 20 });
    const menu = await now();
    await page.waitForFunction(() => !!document.querySelector('.menu') && !!document.querySelector('.game-layer.hidden') && !!document.querySelector('.backdrop .still.loaded'), null, { timeout: 60_000, polling: 20 });
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
    shotName = `${renderer}-${r + 1}`;
    visits = [await visit(page, cdp)];
    shotName = `${renderer}-${r + 1}-again`;
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
    console.log(`run ${r + 1} (${label}): total ${v.marks.entered.toFixed(0)} ms · boot ${v.boot.toFixed(0)} ms · land ${v.world.toFixed(0)} ms · layout ${v.layout.toFixed(0)} ms · town ${v.town.toFixed(0)} ms · step-4 portraits in view ${v.inView.toFixed(0)} ms, all ${v.portraits.toFixed(0)} ms (${v.faces}) · step 5 ${v.home.toFixed(0)} ms · entry ${v.entry.toFixed(0)} ms · ${v.residents} residents in ${v.homes} homes, ${v.forSale} for sale`);
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
  md.push(`| total (page load → game up) | ${median(list.map((v) => v.marks.entered)).toFixed(0)} | ${list.map((v) => v.marks.entered.toFixed(0)).join(', ')} |`);
  for (const k of ['boot', 'world', 'layout', 'town', 'inView', 'portraits', 'home', 'entry', 'residents', 'homes', 'forSale', ...(back ? ['back', 'continue', 'heapMB', 'buffersMB', 'again'] : [])]) md.push(`| ${k} | ${median(list.map((v) => v[k])).toFixed(0)} | ${list.map((v) => v[k].toFixed(0)).join(', ')} |`);
  // (`--back`: the longest task in each stretch.)
  if (back) for (const k of ['entryLong', 'backLong', 'continueLong', 'againLong']) md.push(`| ${k} (longest) | ${median(list.map((v) => Math.max(0, ...v[k]))).toFixed(0)} | ${list.map((v) => Math.max(0, ...v[k])).join(', ')} |`);
  md.push('', '| span | median ms (summed) | count |', '|---|---|---|');
  const names = [...new Set(list.flatMap((v) => v.report?.entries.map((e) => named(e.name)) ?? []))];
  for (const name of names.filter((n) => /^(boot|session|host|warm|portraits|preview|studio|menu: showTown|menu: still|render: nature)/.test(n))) {
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
