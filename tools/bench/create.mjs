#!/usr/bin/env bun
/**
 * Pictures of a new game's first three steps as a player gets there: the main menu, "New game", step 1's
 * land made and built, then shot at each size (`--sizes`) on each renderer (`--renderers`): the land's
 * view, the side panel, the way on and the figures. At the narrowest size, also the panel scrolled to
 * its end. Then, at the first size, an arrow key in the place grid (down a row: another place) and the
 * making panel while that land is made. Then "Lay out the town →": step 2's town laid on that land
 * (its making panel shot meanwhile), shot at each size the same way. Then "Choose a household →":
 * step 3's households offered as cards, shot at each size once the portraits in view are drawn (the
 * narrowest with its cards scrolled to their end), then at the first size with a card chosen (the
 * camera on its area) and after "Show others".
 *
 * The stream pill is shown throughout (as on a first visit, while models still come).
 *
 * Besides the pictures it checks what it can by measure, and says so (`create: …`): the side panel,
 * the way on and the figures clear of the stream pill (if it is up) and of every lot tag shown; the arrow
 * key chose the place a row down and moved the focus there; step 2 says it is step 2; step 3 says it is
 * step 3, offers 10 to 30 households (all of them in a smaller town), draws every portrait (how long
 * that took, and how many), and "Show others" offers others.
 *
 *   bun tools/bench/create.mjs [options]
 *     --sizes 2380x1546,1920x1080,740x1000   window sizes (CSS px)
 *     --renderers webgpu,webgl               the page's renderer (`?renderer=webgl`)
 *     --seed 7                               the page's Math.random seed (the same world in every run)
 *     --world country --options town=city    step 1's place and options (the URL's `?world=`, `?options=`)
 *     --code river-mhs-k2x9q                 the world's code instead (the URL's `?code=`; a bare number: the land's seed)
 *     --shots 1,2,3                          the steps shot (the others only gone through)
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
const code = flag('code', null);
const shots = new Set(flag('shots', '1,2,3').split(',').map(Number));
const worldQuery = [made.world ? `world=${made.world}` : '', made.query ? `options=${made.query}` : '', code ? `code=${encodeURIComponent(code)}` : ''];
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

/** A step's world made and built: its way on (`label`, a pattern) can be pressed. */
const enabled = (page, label) => page.waitForFunction((src) => [...document.querySelectorAll('button')].some((b) => new RegExp(src).test(b.textContent ?? '') && !b.disabled), label.source, { timeout: 120_000, polling: 100 });
/** Step 1's land made and built: "Lay out the town" can be pressed. */
const ready = (page) => enabled(page, /Lay out the town/);
/** Step 2's town laid and built: "Choose a household" can be pressed. */
const townReady = (page) => enabled(page, /Choose a household/);

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
  const panels = [...document.querySelectorAll('.side, .side .head, .side .panel, .view .actions .btn, .view .stats span, .view .offer')].map((el) => ({ name: el.className.split(' ')[0] || el.tagName, r: rect(el) })).filter((p) => p.r);
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

    let stepName = 'step1';
    const shoot = async (name) => {
      const file = join(out, `${stepName}-${renderer}-${name}.png`);
      await page.screenshot({ path: file });
      for (const line of await page.evaluate(overlaps)) fault(`${renderer} ${name}: ${line}`);
      const pill = await page.evaluate(() => !!document.querySelector('.stream'));
      console.log(`create: ${file}${pill ? ' (the stream pill up)' : ''}`);
    };
    /** The step at every size, and the narrowest with its choices (`scroller`) scrolled to their end. `settled`: waited for after each resize. */
    const everySize = async (scroller = '.side .panel', settled = async () => {}) => {
      for (const size of sizes) {
        await page.setViewportSize(size);
        // (The camera glides to frame the world in the resized view; its tags settle after it.)
        await page.waitForTimeout(2500);
        await settled();
        await shoot(`${size.width}x${size.height}`);
      }
      const narrow = sizes.reduce((a, b) => (b.width < a.width ? b : a));
      await page.setViewportSize(narrow);
      await page.waitForTimeout(800);
      const scrolls = await page.evaluate((selector) => {
        const panel = document.querySelector(selector);
        if (!panel) return false;
        panel.scrollTop = panel.scrollHeight;
        return panel.scrollHeight > panel.clientHeight + 1;
      }, scroller);
      await settled();
      await page.waitForTimeout(400);
      await shoot(`${narrow.width}x${narrow.height}-scrolled`);
      if (!scrolls) console.log(`create: ${renderer} ${stepName}: the panel fits at ${narrow.width}x${narrow.height} (nothing to scroll)`);
    };
    if (shots.has(1)) await everySize();

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
    if (shots.has(1)) await shoot(`${sizes[0].width}x${sizes[0].height}-making`);
    await ready(page);
    await page.waitForTimeout(2500);
    if (shots.has(1)) await shoot(`${sizes[0].width}x${sizes[0].height}-place`);

    // Step 2: the town laid on that land (its making panel shot while it is laid), then at every size.
    await page.getByRole('button', { name: /Lay out the town/ }).click();
    stepName = 'step2';
    await page.waitForFunction(() => !!document.querySelector('.making'), null, { timeout: 10_000, polling: 50 }).catch(() => fault(`${renderer}: no making panel while the town is laid out`));
    await page.waitForTimeout(700);
    if (shots.has(2)) await shoot(`${sizes[0].width}x${sizes[0].height}-making`);
    await townReady(page);
    const eyebrow = await page.evaluate(() => document.querySelector('.side .head .eyebrow')?.textContent ?? '');
    if (!/Step 2 of 4 · Town/.test(eyebrow)) fault(`${renderer}: step 2 says "${eyebrow}"`);
    if (shots.has(2)) await everySize();

    if (shots.has(3)) {
      // Step 3: the town lived in, its households offered as cards; shot once the portraits in view are drawn.
      await page.setViewportSize(sizes[0]);
      await page.waitForTimeout(800);
      stepName = 'step3';
      const opened = Date.now();
      await page.getByRole('button', { name: /Choose a household/ }).click();
      await page.waitForFunction(() => document.querySelectorAll('.offer .card').length > 0, null, { timeout: 60_000, polling: 50 });
      const cardsAt = Date.now() - opened;
      /** The portraits in the cards on screen drawn (each has its picture). */
      const inView = () =>
        page
          .waitForFunction(
            () => {
              const list = document.querySelector('.offer .cards');
              if (!list) return false;
              const box = list.getBoundingClientRect();
              const faces = [...list.querySelectorAll('.face')].filter((f) => {
                const r = f.getBoundingClientRect();
                return r.bottom > box.top && r.top < box.bottom;
              });
              // (Scrolled to the end, the cards in view may show no faces: the last card's are above.)
              return list.querySelector('.card') !== null && faces.every((f) => f.querySelector('img'));
            },
            null,
            { timeout: 60_000, polling: 50 },
          )
          .catch(() => fault(`${renderer}: step 3's portraits in view not drawn within 60 s`));
      await inView();
      const viewAt = Date.now() - opened;
      await page
        .waitForFunction(() => [...document.querySelectorAll('.offer .face')].every((f) => f.querySelector('img')), null, { timeout: 120_000, polling: 100 })
        .catch(() => fault(`${renderer}: step 3's portraits not all drawn within 120 s`));
      const allAt = Date.now() - opened;
      const offer = await page.evaluate(() => ({
        step: document.querySelector('.side .head .eyebrow')?.textContent ?? '',
        cards: [...document.querySelectorAll('.offer .card[data-slot]')].map((c) => Number(c.getAttribute('data-slot'))),
        faces: document.querySelectorAll('.offer .face').length,
        households: Number(/(\d+) households/.exec(document.querySelector('.view .stats')?.textContent ?? '')?.[1] ?? NaN),
      }));
      console.log(`create: ${renderer} step 3: ${offer.cards.length} cards of ${offer.households} households, ${offer.faces} portraits; cards ${cardsAt} ms, portraits in view ${viewAt} ms, all ${allAt} ms`);
      if (!/Step 3 of 4 · Household/.test(offer.step)) fault(`${renderer}: step 3 says "${offer.step}"`);
      if (offer.cards.length > 30 || offer.cards.length < Math.min(10, offer.households)) fault(`${renderer}: step 3 offers ${offer.cards.length} of ${offer.households} households`);
      await everySize('.offer .cards', inView);

      // A card chosen: its home lit, the camera on its area, the way on naming them.
      await page.setViewportSize(sizes[0]);
      await page.waitForTimeout(1200);
      await page.locator('.offer .card[data-slot] .pick').nth(2).click();
      await page.waitForTimeout(2500);
      const way = await page.evaluate(() => document.querySelector('.view .actions .btn.primary')?.textContent?.trim() ?? '');
      if (!/^Play the .+s →$/.test(way)) fault(`${renderer}: with a card chosen the way on says "${way}"`);
      await shoot(`${sizes[0].width}x${sizes[0].height}-chosen`);
      // Show others: other households.
      await page.getByRole('button', { name: /Show others/ }).click();
      await page.waitForTimeout(400);
      await inView();
      await page.waitForTimeout(1500);
      const others = await page.evaluate(() => [...document.querySelectorAll('.offer .card[data-slot]')].map((c) => Number(c.getAttribute('data-slot'))));
      const again = others.filter((s) => offer.cards.includes(s)).length;
      console.log(`create: ${renderer} step 3: Show others offers ${others.length}, ${again} of them shown before`);
      if (offer.households > offer.cards.length + 10 && again) fault(`${renderer}: Show others offered ${again} households again`);
      await shoot(`${sizes[0].width}x${sizes[0].height}-others`);
    }
    await context.close();
  }
} finally {
  await browser.close();
  server?.kill();
}
console.log(faults ? `create: ${faults} fault(s); pictures in ${out}` : `create: clean; pictures in ${out}`);
