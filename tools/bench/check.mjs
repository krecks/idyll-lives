#!/usr/bin/env node
/**
 * Checking a model in a real browser, headless on the real GPU, at 1 % volume: the screenshots each
 * object task takes. Starts its own Vite from this checkout on the port given (or uses `--url`, a
 * server you started yourself from this checkout), and writes PNGs (and a JSON log) to `--out`.
 *
 *   node tools/bench/check.mjs <what> --port 5227 [options]
 *
 *   gallery   the gallery (`/gallery.html`), and with `--placeholder` the same with every .glb
 *             answered 404 (the fallback the game draws), side by side in one picture
 *               --keys model.sofa,model.lamp   which models (required)
 *               --cols 4  --alpha 0.97 --beta 1.15 --radius 4 --target 1.4   the gallery's camera
 *   clips     the clips bench (`/clips.html`) held at set times, one picture per time
 *               --keys model.fridge (required)  --clip door_open  --times 0,0.3,0.6,1,99
 *               --colors body:#c46b4a  --target 1.6  --alpha/--beta/--radius
 *   play      buy and use in play: opens `?state=<state>&debug`, free will off, buys the
 *             object, sends the use, waits until the resident is at it and using it (`__sims()`),
 *             then takes a burst of pictures and logs the clips playing (`__renderer.motion`)
 *               --object deskSimple (required)  --at 23,14,0 (corner and rot)  --turn 0
 *               --use sit (interaction id or index, default 0)  --sim 0  --colors body:12
 *               --state bathroom-check  --shots 6 --every 500 (ms)  --wait 60 (s, for the use)
 *               --radius 7 --beta 1.0 --alpha … --y 1.2 (the camera following the resident)
 *               --speed 1  --hour (the renderer's light only; the state sets the clock)
 *               --free-will (leave it on: with `needy-check`, to watch them go on their own)
 *
 *   Common:  --port <n> (its own Vite, --strictPort: a port in use is an error, never someone
 *            else's server) | --url http://localhost:<n>;  --out <dir> (default
 *            <repo>/bench-results/check-<time>);  --quality low|medium|high;  --renderer webgl;
 *            --size 1280x800;  --extra 'k=v&k=v' (more URL flags);  --headed (a window: only to
 *            measure timing).
 *
 * Needs `pnpm wasm` first (play), and Playwright: `PLAYWRIGHT=<path to playwright/index.mjs>` if it
 * isn't installed in web/, `CHROME=<executable>` for a browser of your own (Chrome or Chrome for
 * Testing, not `chrome-headless-shell`, which draws in software). See tools/bench/README.md.
 */

import { spawn } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { createConnection } from 'node:net';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { quiet } from './quiet.mjs';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const WEB = join(ROOT, 'web/');

const [what, ...args] = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};
const has = (name) => args.includes(`--${name}`);
const list = (name, fallback) => (flag(name, fallback) ?? '').split(',').map((s) => s.trim()).filter(Boolean);
const need = (name) => flag(name) ?? fail(`--${name} is needed for '${what}'`);
function fail(message) {
  console.error(`check: ${message}`);
  process.exit(1);
}

if (!['gallery', 'clips', 'play'].includes(what)) fail('say what to check: gallery, clips or play (options at the top of tools/bench/check.mjs)');
const url = flag('url', null);
const port = Number(flag('port', null));
if (!url && !port) fail('give your own port (--port <n>, a free one) or --url of a server you started from this checkout');
if (port === 5199) fail('5199 is the main checkout\'s server: pick a port of your own');
const base = url ?? `http://localhost:${port}`;
const stamp = new Date().toISOString().replace(/\.\d+Z$/, '').replace(/[:T]/g, '-');
const out = flag('out', join(ROOT, 'bench-results', `check-${stamp}`));
mkdirSync(out, { recursive: true });
const [width, height] = flag('size', '1280x800').split('x').map(Number);
/** URL flags every page gets. */
const common = new URLSearchParams(flag('extra', ''));
if (flag('quality')) common.set('quality', flag('quality'));
if (flag('renderer')) common.set('renderer', flag('renderer'));
const log = { what, args, shots: [] };

const portTaken = (p) =>
  new Promise((resolve) => {
    const socket = createConnection({ port: p, host: 'localhost' }, () => (socket.end(), resolve(true)));
    socket.on('error', () => resolve(false));
  });

let vite = null;
if (!url) {
  if (await portTaken(port)) fail(`port ${port} is in use (someone else's server?): pick a free one`);
  vite = spawn(`${WEB}node_modules/.bin/vite`, ['--port', String(port), '--strictPort'], { cwd: WEB, stdio: ['ignore', 'pipe', 'inherit'] });
  process.on('exit', () => vite.kill());
  await new Promise((resolve, reject) => {
    vite.stdout.on('data', (d) => String(d).includes('Local') && resolve());
    vite.on('exit', (code) => reject(new Error(`vite exited (${code})`)));
  });
}

async function playwright() {
  if (process.env.PLAYWRIGHT) return import(process.env.PLAYWRIGHT);
  try {
    return await import(createRequire(WEB + 'package.json').resolve('playwright'));
  } catch {
    throw new Error('Playwright not found: install it, or set PLAYWRIGHT=<path to playwright/index.mjs>');
  }
}
const { chromium } = await playwright();
const browser = await chromium.launch({
  headless: !has('headed'),
  ...(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chromium' }),
  args: ['--enable-unsafe-webgpu', '--disable-backgrounding-occluded-windows', '--disable-renderer-backgrounding', '--disable-background-timer-throttling'],
});

/**
 * A page at 1 % volume, the camera never taking over on its own (the director's delay 0), and free
 * will off unless `--free-will` (the game's setting applies to the player's household as a game
 * opens, over what a state says).
 */
async function newPage() {
  const context = await browser.newContext({ viewport: { width, height }, deviceScaleFactor: 1 });
  await context.addInitScript(quiet);
  await context.addInitScript((autonomy) => {
    const key = 'open-sims-wasm.settings';
    let settings = {};
    try {
      settings = JSON.parse(localStorage.getItem(key) ?? '{}');
    } catch {
      // Defaults.
    }
    localStorage.setItem(key, JSON.stringify({ ...settings, directorDelay: 0, autonomy }));
  }, has('free-will'));
  const page = await context.newPage();
  page.on('pageerror', (err) => console.warn(`page error: ${err.message}`));
  page.on('console', (m) => m.type() === 'error' && console.warn(`console: ${m.text()}`));
  return page;
}

function pageUrl(path, params) {
  const q = new URLSearchParams(common);
  for (const [k, v] of Object.entries(params)) if (v !== undefined && v !== null) q.set(k, String(v));
  return `${base}${path}?${q}`;
}

async function shot(page, name, extra = {}) {
  const file = join(out, `${name}.png`);
  await page.screenshot({ path: file });
  log.shots.push({ file, ...extra });
  console.log(file);
  return file;
}

/** Two pictures side by side in one (left, right), with their captions. */
async function sideBySide(left, right, name, captions) {
  const page = await newPage();
  const img = (f) => `data:image/png;base64,${readFileSync(f).toString('base64')}`;
  const [a, b] = [img(left), img(right)];
  await page.setViewportSize({ width: width * 2, height });
  await page.setContent(`<body style="margin:0;display:flex;font:14px system-ui"><figure style="margin:0;position:relative"><img src="${a}"><figcaption style="position:absolute;left:8px;top:8px;background:#000a;color:#fff;padding:2px 6px">${captions[0]}</figcaption></figure><figure style="margin:0;position:relative"><img src="${b}"><figcaption style="position:absolute;left:8px;top:8px;background:#000a;color:#fff;padding:2px 6px">${captions[1]}</figcaption></figure></body>`);
  await shot(page, name);
  await page.context().close();
}

const settle = (page, ms) => page.waitForTimeout(ms);
const camera = () => ({ alpha: flag('alpha'), beta: flag('beta'), radius: flag('radius'), target: flag('target') });

if (what === 'gallery') {
  const keys = need('keys');
  const name = `gallery-${keys.replace(/model\./g, '').replace(/,/g, '+')}`.slice(0, 120);
  const params = { keys, cols: flag('cols'), ...camera() };
  const page = await newPage();
  await page.goto(pageUrl('/gallery.html', params));
  await page.waitForFunction(() => window.__ready === true, null, { timeout: 60_000 });
  await settle(page, 1500);
  const model = await shot(page, name, { kind: 'model' });
  if (has('placeholder')) {
    const fallback = await newPage();
    await fallback.route(/\.glb(\?|$)/, (route) => route.fulfill({ status: 404, body: '' }));
    await fallback.goto(pageUrl('/gallery.html', params));
    await fallback.waitForFunction(() => window.__ready === true, null, { timeout: 60_000 });
    await settle(fallback, 1500);
    const placeholder = await shot(fallback, `${name}-placeholder`, { kind: 'placeholder' });
    await sideBySide(model, placeholder, `${name}-side`, ['model', 'placeholder']);
  }
}

if (what === 'clips') {
  const keys = need('keys');
  const name = `clips-${keys.replace(/model\./g, '').replace(/,/g, '+')}`.slice(0, 100);
  const page = await newPage();
  for (const t of list('times', '0,0.3,0.6,1,99')) {
    await page.goto(pageUrl('/clips.html', { keys, clip: flag('clip'), t, colors: flag('colors'), cols: flag('cols'), ...camera() }));
    await page.waitForFunction(() => window.__ready === true, null, { timeout: 60_000 });
    await settle(page, 800);
    await shot(page, `${name}-${flag('clip') ?? 'all'}-t${t}`, { t });
  }
}

if (what === 'play') {
  const object = need('object');
  const state = flag('state', 'bathroom-check');
  const [x, z, rot] = flag('at', '23,14,0').split(',').map(Number);
  const sim = Number(flag('sim', '0'));
  const shots = Number(flag('shots', '6'));
  const every = Number(flag('every', '500'));
  const colors = flag('colors') ? Object.fromEntries(flag('colors').split(',').map((p) => p.split(':')).map(([k, v]) => [k, Number(v)])) : undefined;
  const page = await newPage();
  await page.goto(pageUrl('/', { state, debug: '', hour: flag('hour') }));
  // In play: the world is there and the household's portraits are drawn (the strip shows its name).
  await page.waitForFunction(() => typeof window.__sims === 'function' && window.__world?.() && window.__sims().length > 0, null, { timeout: 120_000 });
  await page.waitForFunction(() => !document.body.innerText.includes('Drawing portraits'), null, { timeout: 60_000 });
  await settle(page, 1500);
  const before = await page.evaluate(() => new Set(window.__world().objects.map((o) => o.id)).size);
  await page.evaluate(
    ({ object, x, z, rot, colors, turn, freeWill }) => {
      window.__send({ type: 'setAutonomy', enabled: freeWill, household: 0 });
      window.__addFunds(100_000);
      window.__send({ type: 'buy', household: 0, object, at: [x, z, rot], ...(colors ? { colors } : {}), ...(turn ? { turn } : {}) });
    },
    { object, x, z, rot, colors, turn: Number(flag('turn', '0')), freeWill: has('free-will') },
  );
  const bought = await page
    .waitForFunction(({ object, x, z }) => window.__world().objects.find((o) => o.def === object && o.x === x && o.z === z)?.id, { object, x, z }, { timeout: 10_000 })
    .then((h) => h.jsonValue())
    .catch(() => null);
  if (bought === null) fail(`not bought: ${object} at ${x},${z},${rot} (${before} objects; is the spot free? see the game's toast)`);
  const use = flag('use', '0');
  const interaction = await page.evaluate(
    ({ id, use }) => {
      const def = window.__world().objects.find((o) => o.id === id).def;
      return /^\d+$/.test(use) ? Number(use) : window.__content.object(def)?.interactions.findIndex((i) => i.id === use);
    },
    { id: bought, use },
  );
  const offered = await page.evaluate((id) => window.__content.object(window.__world().objects.find((o) => o.id === id).def)?.interactions.map((i) => i.id) ?? [], bought);
  if (!(interaction >= 0 && interaction < offered.length)) fail(`${object} has no interaction '${use}' (it offers: ${offered.join(', ') || 'none'})`);
  const speed = Number(flag('speed', '1'));
  await page.evaluate(({ sim, id, interaction, speed, view }) => {
    window.__send({ type: 'setSpeed', speed });
    window.__send({ type: 'use', sim, object: id, interaction });
    window.__controls.follow(sim, view);
  }, { sim, id: bought, interaction, speed, view: Object.fromEntries(Object.entries({ radius: Number(flag('radius', '7')), beta: flag('beta') && Number(flag('beta')), alpha: flag('alpha') && Number(flag('alpha')), y: flag('y') && Number(flag('y')) }).filter(([, v]) => v)) });
  const wait = Number(flag('wait', '60')) * 1000;
  const using = await page
    .waitForFunction(({ sim, id }) => window.__sims().find((s) => s.id === sim && s.object === id && s.using), { sim, id: bought }, { timeout: wait, polling: 100 })
    .then(() => true)
    .catch(() => false);
  if (!using) {
    const [resident, screen] = await page.evaluate((s) => [window.__sims().find((x) => x.id === s), document.body.innerText.replace(/\s+/g, ' ').slice(-300)], sim);
    console.warn(`the resident wasn't using it after ${wait / 1000} s: ${JSON.stringify(resident)}; on screen: ${screen}`);
  }
  const name = `play-${object}-${x}-${z}-${rot}`;
  for (let i = 0; i < shots; i++) {
    const info = await page.evaluate(({ sim, id }) => {
      const s = window.__sims().find((x) => x.id === sim);
      const clips = window.__renderer?.motion?.states?.get(id);
      // Each clip playing on it: how far in (s), of how long, and how it plays.
      const playing = clips ? Object.fromEntries([...clips].map(([name, c]) => [name, { time: +c.time.toFixed(2), duration: +c.clip.duration.toFixed(2), kind: c.kind }])) : null;
      return { sim: s && { pose: s.pose, using: s.using, interaction: s.interaction, action: s.action, x: +s.x.toFixed(2), z: +s.z.toFixed(2) }, clips: playing };
    }, { sim, id: bought });
    await shot(page, `${name}-${String(i).padStart(2, '0')}`, info);
    if (i < shots - 1) await settle(page, every);
  }
}

writeFileSync(join(out, `${what}.json`), JSON.stringify(log, null, 2));
await browser.close();
vite?.kill();
