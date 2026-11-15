#!/usr/bin/env bun
/**
 * Pictures of step 1 ("A new world") as a player gets there: the main menu, "New game", step 1's world
 * made and built, then shot at each size (`--sizes`) on each renderer (`--renderers`): the town's view,
 * the side panel, the way on and the stats. At the narrowest size, also the panel scrolled to its end.
 * Then, at the first size, an arrow key in the place grid (down a row: another place) and the
 * making panel while that world is made.
 *
 * The stream pill is shown throughout (as on a first visit, while models still come).
 *
 * Besides the pictures it checks what it can by measure, and says so (`create: …`): the side panel,
 * the way on and the stats clear of the stream pill (if it is up) and of every lot tag shown; the arrow
 * key chose the place a row down and moved the focus there.
 *
 *   bun tools/bench/create.mjs [options]
 *     --sizes 2380x1546,1920x1080,740x1000   window sizes (CSS px)
 *     --renderers webgpu,webgl               the page's renderer (`?renderer=webgl`)
 *     --seed 7                               the page's Math.random seed (the same world in every run)
 *     --world country --options town=city    step 1's place and options (the URL's `?world=`, `?options=`)
 *     --headed                               in a window (default headless: the real GPU either way)
 *     --port 5184 | --url <u>                a dev server on the port (started if none answers), or one already running
 *     --out <dir>                            default bench-results/create-<time>/
 *
 * Needs Playwright like render.mjs (`PLAYWRIGHT=<path to playwright/index.mjs>`).
 */
import { spawn } from 'node:child_process';
import { mkdirSync } from 'node:fs';
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

const sizes = flag('sizes', '2380x1546,1920x1080,740x1000')
  .split(',')
  .map((s) => s.split('x').map(Number))
  .map(([width, height]) => ({ width, height }));
const renderers = flag('renderers', 'webgpu,webgl').split(',');
const seed = Number(flag('seed', '7'));
// (None given: step 1 as a player gets it, the URL without them.)
const made = worldFlags(flag, null);
const worldQuery = [made.world ? `world=${made.world}` : '', made.query ? `options=${made.query}` : ''];
const port = Number(flag('port', '5184'));
const stamp = new Date().toISOString().replace(/\.\d+Z$/, '').replace(/[:T]/g, '-');
const out = flag('out', join(ROOT, 'bench-results', `create-${stamp}`));
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

/** Step 1's world made and built: "Choose a household" can be pressed. */
const ready = (page) => page.waitForFunction(() => [...document.querySelectorAll('button')].some((b) => /Choose a household/.test(b.textContent ?? '') && !b.disabled), null, { timeout: 120_000, polling: 100 });

/**
 * What overlaps on screen now: the UI over the town (the side panel, or its two parts when narrow; the
 * way on; the stats) against the stream pill and each lot tag shown. Returns one line per overlap.
 */
function overlaps() {
  const rect = (el) => {
    const r = el.getBoundingClientRect();
    return r.width && r.height ? { l: r.left, t: r.top, r: r.right, b: r.bottom } : null;
  };
  const hit = (a, b) => a.l < b.r && b.l < a.r && a.t < b.b && b.t < a.b;
  const panels = [...document.querySelectorAll('.side, .side .head, .side .panel, .view .actions .btn, .view .stats span')].map((el) => ({ name: el.className.split(' ')[0] || el.tagName, r: rect(el) })).filter((p) => p.r);
  const found = [];
  const stream = document.querySelector('.stream');
  const pill = stream && rect(stream);
  if (pill) for (const p of panels) if (hit(p.r, pill)) found.push(`the stream pill over .${p.name}`);
  for (const tag of document.querySelectorAll('.tags .pin')) {
    if (getComputedStyle(tag).visibility === 'hidden') continue;
    const t = tag.firstElementChild && rect(tag.firstElementChild);
    if (!t) continue;
    for (const p of panels) if (hit(p.r, t)) found.push(`the tag "${tag.textContent?.trim().slice(0, 30)}" over .${p.name}`);
  }
  return found;
}

const { chromium } = await playwright();
const browser = await chromium.launch({
  headless: !has('headed'),
  ...(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chromium' }),
  args: ['--enable-unsafe-webgpu', '--disable-backgrounding-occluded-windows', '--disable-renderer-backgrounding', '--disable-background-timer-throttling'],
});
let faults = 0;
const fault = (line) => {
  faults++;
  console.warn(`create: ${line}`);
};
try {
  for (const renderer of renderers) {
    const context = await browser.newContext({ viewport: sizes[0], deviceScaleFactor: 1 });
    await context.addInitScript(quiet);
    await context.addInitScript(seeded, seed);
    const page = await context.newPage();
    page.on('pageerror', (e) => console.error(`page error: ${e.message}`));
    const query = [renderer === 'webgl' ? 'renderer=webgl' : '', ...worldQuery].filter(Boolean).join('&');
    await page.goto(`${base}/?${query}`);
    await page.waitForFunction(() => !!document.querySelector('.menu') && !document.querySelector('.loading'), null, { timeout: 180_000, polling: 200 });
    await page.waitForTimeout(1500);
    await page.getByText('New game', { exact: true }).click();
    await ready(page);
    // The stream pill up, as on a first visit while the models still come (by then they may all have
    // come): the shots show it, and the UI over the town is checked against it.
    await page.evaluate(async () => {
      const { streamState } = await import('/src/assets/streamState.svelte.ts');
      streamState.apply({ total: 400, done: 260, label: 'Loading houses', active: true });
    });

    const shoot = async (name) => {
      const file = join(out, `step1-${renderer}-${name}.png`);
      await page.screenshot({ path: file });
      for (const line of await page.evaluate(overlaps)) fault(`${renderer} ${name}: ${line}`);
      const pill = await page.evaluate(() => !!document.querySelector('.stream'));
      console.log(`create: ${file}${pill ? ' (the stream pill up)' : ''}`);
    };
    for (const size of sizes) {
      await page.setViewportSize(size);
      // (The camera glides to frame the town in the resized view; its tags settle after it.)
      await page.waitForTimeout(2500);
      await shoot(`${size.width}x${size.height}`);
    }
    // The narrowest: its choices scrolled to their end.
    const narrow = sizes.reduce((a, b) => (b.width < a.width ? b : a));
    await page.setViewportSize(narrow);
    await page.waitForTimeout(800);
    const scrolls = await page.evaluate(() => {
      const panel = document.querySelector('.side .panel');
      if (!panel) return false;
      panel.scrollTop = panel.scrollHeight;
      return panel.scrollHeight > panel.clientHeight + 1;
    });
    await page.waitForTimeout(400);
    await shoot(`${narrow.width}x${narrow.height}-scrolled`);
    if (!scrolls) console.log(`create: ${renderer}: the panel fits at ${narrow.width}x${narrow.height} (nothing to scroll)`);

    // Back to the first size: an arrow key down the place grid chooses the place a row down (as
    // many as the grid shows in a row; across by one if there is no row under it), and its world is
    // made (the making panel shown meanwhile).
    await page.setViewportSize(sizes[0]);
    await page.waitForTimeout(1200);
    await page.locator('.places [role="radio"][aria-checked="true"]').focus();
    const { before, row, count } = await page.evaluate(() => {
      const grid = document.querySelector('.places');
      const radios = [...grid.querySelectorAll('[role="radio"]')];
      return { before: radios.findIndex((b) => b.getAttribute('aria-checked') === 'true'), row: getComputedStyle(grid).gridTemplateColumns.split(' ').length, count: radios.length };
    });
    const [key, wanted] = before + row < count ? ['ArrowDown', before + row] : ['ArrowRight', (before + 1) % count];
    await page.keyboard.press(key);
    const after = await page.evaluate(() => {
      const radios = [...document.querySelectorAll('.places [role="radio"]')];
      return { checked: radios.findIndex((b) => b.getAttribute('aria-checked') === 'true'), focused: radios.indexOf(document.activeElement) };
    });
    if (after.checked !== wanted || after.focused !== wanted) fault(`${renderer}: ${key} from place ${before} chose ${after.checked}, focus on ${after.focused} (wanted ${wanted})`);
    await page.waitForFunction(() => !!document.querySelector('.making'), null, { timeout: 10_000, polling: 50 }).catch(() => fault(`${renderer}: no making panel after a new place`));
    await page.waitForTimeout(700);
    await shoot(`${sizes[0].width}x${sizes[0].height}-making`);
    await ready(page);
    await page.waitForTimeout(2500);
    await shoot(`${sizes[0].width}x${sizes[0].height}-place`);
    await context.close();
  }
} finally {
  await browser.close();
  server?.kill();
}
console.log(faults ? `create: ${faults} fault(s); pictures in ${out}` : `create: clean; pictures in ${out}`);
