#!/usr/bin/env node
/**
 * Checking a model in a real browser, headless on the real GPU, at 1 % volume: the screenshots each
 * object task takes. Starts its own Vite from this checkout on the port given (or uses `--url`, a
 * server you started yourself from this checkout), and writes PNGs (and a JSON log) to `--out`.
 *
 *   node tools/bench/check.mjs <what> --port 5227 [options]
 *
 *   gallery   the gallery (`/gallery.html`)
 *               --keys model.sofa,model.lamp   which models (required)
 *               --each   one picture per key (`gallery-<key>.png`) on one page, else all in one
 *               --cols 4  --alpha 0.97 --beta 1.15 --radius 4 --target 1.4   the gallery's camera
 *               --scale adult|child|all   residents beside each model, for scale
 *               --camera game   the game's Live camera: one picture per model, aimed at it
 *   clips     the clips bench (`/clips.html`) held at set times, one picture per time
 *               --keys model.fridge (required)  --clip door_open  --times 0,0.3,0.6,1,99
 *               --each   each key on its own (`clips-<key>-<clip>-t<t>.png`), one page for all
 *               --colors body:#c46b4a  --target 1.6  --alpha/--beta/--radius
 *   play      buy and use in play: opens `?state=<state>&debug`, free will off, buys each object,
 *             sends each use, waits until the resident is at it and using it, takes pictures
 *             framing the object, then sells it and goes on to the next, all on one page. In the
 *             page, every animation frame, it logs the clips playing on the object
 *             (`__renderer.motion`) and the resident's pose and animation (`__sims()`). One JSON
 *             line per use goes to `<out>/play-<run>.jsonl`.
 *               --objects toilet,fridge@23,14,0   which objects (`def` or `def@x,z,rot`), or
 *               --objects-file <path>   one a line (`#` comments), each
 *                 `def[@x,z,rot | @existing] [with helper[@x,z,rot] helper…]`. Without a place: on
 *                 a `review-*` state a free spot for its layer and footprint (tools/bench/README.md),
 *                 else `--at`, else wherever the game finds room
 *               --with deskSimple@10,13,2,…   helpers for every object: bought first, kept while its
 *                 uses run, sold after (a desk at a chair, a table for café chairs); a line's own
 *                 `with …` adds to them
 *               --use-existing crib,garden.tomatoes   the state's own (the first of each kind: the
 *                 crib with the baby, `review-garden`'s ripe crops), run first, never sold
 *               --skills cooking=5,creativity=3   the resident's skills, set as the state opens
 *               --use all|<id>|<index>   (default all)  --sim 0  --colors body:12  --turn 0
 *               --state bathroom-check  --shots 3 --every 1000 (ms)  --wait 60 (s, from the use
 *                 sent to the resident using it)
 *               --walk 3 (speed walking there, at most 3 = 10×)  --speed 1 (from 3 tiles off)
 *               --radius --beta --alpha --y (the camera; default from the model's size and facing)
 *               --hour 12 (the renderer's light only; the state sets the clock; on a `*-night`
 *                 state none, the light the sim's clock gives; `--hour state` for none anywhere)
 *               --free-will (leave it on: with `needy-check`, to watch them go on their own)
 *             The old single object: --object deskSimple --at 23,14,0 (corner and rot) --use sit
 *               (default 0) --shots 6 --every 500; the camera follows the resident as before.
 *
 *   sheet     contact sheets for reading an evidence folder (`bench-results/review/<cat>`) at a glance:
 *             per object `sheets/obj-<id>.png`, up to six of its pictures in a grid (gallery, game
 *             camera, in use in play, clip held open, then a close-up, at night, other framings of
 *             the clip and the other uses),
 *             and `sheets/age-<id>.png` (`-2`, … past six) for its pose and play shots by life stage,
 *             each picture labelled with its file. Files are found by the names the other commands
 *             give them, anywhere in the folder (the newest of a name taken twice). PNGs 1600 wide
 *             (about 1000 high for 2 × 2), drawn in the headless browser: no server, no --port.
 *               --dir <folder> (required)  --ids fridge,sink | --ids-file <path> (default: the
 *                 folder's `../lists/<folder>.txt` or `.models.txt`, else the ids in backticks that
 *                 open the rows of its `index.md`)  --out (default <folder>/sheets)
 *
 *   Common:  --port <n> (its own Vite, --strictPort: a port in use is an error, never someone
 *            else's server) | --url http://localhost:<n>;  --out <dir> (default
 *            <repo>/bench-results/check-<time>);  --name <run> (the run's log is
 *            `<what>-<run>.json`, default the time it started, so runs into one `--out` keep theirs;
 *            given, `play`'s pictures start with it too);  --quality low|medium|high;  --renderer webgl;
 *            --size 1280x800;  --extra 'k=v&k=v' (more URL flags);  --headed (a window: only to
 *            measure timing).
 *
 * The setup fails loudly: the wasm and the asset manifest are fetched through the server first
 * (each must answer 200), and every HTTP error, failed request, page error and console error is
 * logged with the job it happened in (`<what>-<run>.json`, `play-<run>.jsonl`).
 *
 * Needs `pnpm wasm` first (play), and Playwright: `PLAYWRIGHT=<path to playwright/index.mjs>` if it
 * isn't installed in web/, `CHROME=<executable>` for a browser of your own (Chrome or Chrome for
 * Testing, not `chrome-headless-shell`, which draws in software). See tools/bench/README.md.
 */

import { spawn } from 'node:child_process';
import { appendFileSync, existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { createConnection } from 'node:net';
import { basename, join, resolve, sep } from 'node:path';
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
const num = (name, fallback) => (flag(name) === undefined ? fallback : Number(flag(name)));
function fail(message) {
  console.error(`check: ${message}`);
  process.exit(1);
}

if (!['gallery', 'clips', 'play', 'sheet'].includes(what)) fail('say what to check: gallery, clips, play or sheet (options at the top of tools/bench/check.mjs)');
/** Every command but `sheet` opens the game's pages from a server. */
const served = what !== 'sheet';
const url = flag('url', null);
const port = Number(flag('port', null));
if (served && !url && !port) fail('give your own port (--port <n>, a free one) or --url of a server you started from this checkout');
if (port === 5199) fail('5199 is the main checkout\'s server: pick a port of your own');
const base = url ?? `http://localhost:${port}`;
const stamp = new Date().toISOString().replace(/\.\d+Z$/, '').replace(/[:T]/g, '-');
const out = flag('out', what === 'sheet' ? join(resolve(need('dir')), 'sheets') : join(ROOT, 'bench-results', `check-${stamp}`));
mkdirSync(out, { recursive: true });
/** This run's name: its log files are `<what>-<run>.json(l)`, never another run's. */
const run = flag('name')?.replace(/[^A-Za-z0-9_.-]+/g, '_') ?? `${stamp}-${process.pid}`;
const [width, height] = flag('size', '1280x800').split('x').map(Number);
/** URL flags every page gets. */
const common = new URLSearchParams(flag('extra', ''));
if (flag('quality')) common.set('quality', flag('quality'));
if (flag('renderer')) common.set('renderer', flag('renderer'));
const log = { what, run, args, setup: {}, jobs: [], shots: [] };
const started = Date.now();
const since = () => Math.round((Date.now() - started) / 100) / 10;

const portTaken = (p) =>
  new Promise((resolve) => {
    const socket = createConnection({ port: p, host: 'localhost' }, () => (socket.end(), resolve(true)));
    socket.on('error', () => resolve(false));
  });

let vite = null;
if (served && !url) {
  if (await portTaken(port)) fail(`port ${port} is in use (someone else's server?): pick a free one`);
  vite = spawn(`${WEB}node_modules/.bin/vite`, ['--port', String(port), '--strictPort'], { cwd: WEB, stdio: ['ignore', 'pipe', 'inherit'] });
  process.on('exit', () => vite.kill());
  await new Promise((resolve, reject) => {
    vite.stdout.on('data', (d) => String(d).includes('Local') && resolve());
    vite.on('exit', (code) => reject(new Error(`vite exited (${code})`)));
  });
}

// Fail loudly when the server can't give what the pages need: a wasm package that is missing, or
// linked from outside the server's allowed folders (403), leaves pictures of an empty scene.
for (const [name, path, needed] of served
  ? [
      ['wasm', '/src/core/wasm-pkg/sim_wasm_bg.wasm', what === 'play'],
      ['manifest', '/assets/manifest.json', true],
    ]
  : []) {
  const answer = await fetch(base + path).then((r) => r.status, (err) => String(err));
  log.setup[name] = answer;
  if (answer !== 200 && needed) fail(`${base}${path} answered ${answer}, not 200 (copy the wasm packages into this checkout, run \`pnpm wasm\`; see tools/bench/README.md)`);
  if (answer !== 200) console.warn(`check: ${base}${path} answered ${answer}`);
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

/** The job running (a picture, an object's use): what goes wrong in the page is logged with it. */
let job = null;
function startJob(name, extra = {}) {
  job = { name, at: since(), errors: [], ...extra };
  log.jobs.push(job);
  return job;
}
function noteError(text) {
  console.warn(`${job ? `[${job.name}] ` : ''}${text}`);
  (job?.errors ?? (log.setup.errors ??= [])).push(text);
}

/**
 * In every page, before its own scripts: `__check.loading()`, the loading screen's text (or null);
 * and `__check.watch(...)`, which from then on, every animation frame, logs the clips playing on an
 * object and what a resident does (short clips last a few frames: sampling from node missed them),
 * and slows the game down as the resident comes near (`__check.stop()` returns the log).
 */
function pageHelpers() {
  // The loading screen's labels; "Getting ready" only on the screen itself (a resident may be doing it).
  const LOADING = ['Moving in', 'Drawing portraits', 'Building the world', 'Warming up'];
  const loading = () => {
    const screen = document.querySelector('.loading[role=progressbar]');
    if (screen) return screen.getAttribute('aria-valuetext') || 'loading';
    const text = document.body?.innerText ?? '';
    return LOADING.find((l) => text.includes(l)) ?? null;
  };
  // The game's toasts as they come (a buy or a use refused says why there, for a few seconds).
  const toasts = [];
  new MutationObserver(() => {
    for (const el of document.querySelectorAll('.toast:not([data-check-seen])')) {
      el.setAttribute('data-check-seen', '');
      if (toasts.length < 500) toasts.push({ t: performance.now(), text: el.textContent.trim() });
    }
  }).observe(document, { childList: true, subtree: true });
  /** Most changes logged per clip and for the resident (a loop's rounds would go on). */
  const MAX_EVENTS = 200;
  let w = null;
  function sample() {
    const t = Math.round(performance.now() - w.t0);
    w.frames++;
    const states = window.__renderer?.motion?.states?.get(w.id);
    for (const [name, c] of states ?? []) {
      const duration = c.clip.duration;
      const clip = (w.clips[name] ??= { kind: c.kind, duration: +duration.toFixed(2), first: t, max: 0, state: 'rest', events: [] });
      clip.max = Math.max(clip.max, +c.time.toFixed(2));
      const state = c.time <= 1e-3 ? 'rest' : c.time >= duration - 1e-3 ? 'open' : 'moving';
      if (state !== clip.state && clip.events.length < MAX_EVENTS) clip.events.push([t, state]);
      clip.state = state;
    }
    for (const [name, clip] of Object.entries(w.clips)) {
      if (states?.has(name) || clip.state === 'rest') continue;
      clip.state = 'rest';
      if (clip.events.length < MAX_EVENTS) clip.events.push([t, 'rest']);
    }
    const s = window.__sims?.().find((x) => x.id === w.sim);
    if (!s) return;
    const using = s.using && s.object === w.id;
    const now = { pose: s.pose, using, action: s.action, interaction: s.object === w.id ? s.interaction : null, doing: s.doing };
    const key = JSON.stringify(now);
    if (key !== w.last && w.resident.length < MAX_EVENTS) w.resident.push({ t, ...now });
    w.last = key;
    if (using && w.usingAt === null) w.usingAt = t;
    if (s.doing || s.queue.length || using) w.busyAt = t;
    // Fast while walking there, the use's own speed from a few tiles off (fast skips short uses);
    // a cook's steps at the fridge and the counter first go fast too (they aren't at it yet).
    if (w.slowedAt === null && (using || (s.object === w.id && Math.hypot(s.x - w.near.x, s.z - w.near.z) < w.near.r))) {
      window.__send({ type: 'setSpeed', speed: w.near.speed });
      w.slowedAt = t;
    }
  }
  /**
   * An object's body as drawn: its vertices in the world, x, y, z flat (at most 50,000 a mesh),
   * through its instance's matrix; null when it isn't drawn (yet). Its vertices, not its meshes'
   * boxes: a box turned with the model is far bigger than a round crown.
   */
  function points(id) {
    const pop = window.__renderer?.placed?.find((p) => p.id === id)?.pop;
    if (!pop?.matrices) return null;
    const m = pop.matrices.subarray(pop.index * 16, pop.index * 16 + 16);
    const all = [];
    for (const mesh of pop.meshes) {
      const v = mesh.getVerticesData?.('position');
      if (!v) continue;
      const step = 3 * Math.max(1, Math.floor(v.length / 3 / 50_000));
      for (let i = 0; i < v.length; i += step) {
        const [x, y, z] = [v[i], v[i + 1], v[i + 2]];
        all.push(x * m[0] + y * m[4] + z * m[8] + m[12], x * m[1] + y * m[5] + z * m[9] + m[13], x * m[2] + y * m[6] + z * m[10] + m[14]);
      }
    }
    return all.length ? all : null;
  }
  /** The corners of what `points` gives (`lo`, `hi`: x, y, z), or null. */
  function bounds(id) {
    const p = points(id);
    if (!p) return null;
    const lo = [Infinity, Infinity, Infinity];
    const hi = [-Infinity, -Infinity, -Infinity];
    for (let i = 0; i < p.length; i++) {
      lo[i % 3] = Math.min(lo[i % 3], p[i]);
      hi[i % 3] = Math.max(hi[i % 3], p[i]);
    }
    return { lo, hi };
  }
  /**
   * How far off the game camera at `alpha`, `beta` looking at `target` has to stand for all of
   * the object to be in the picture, inside the share `keep` of it across and up (the HUD covers
   * the top and the bottom); 0 when it isn't drawn. The camera stands at target + r (cos α sin β,
   * cos β, sin α sin β).
   */
  function fit(id, { target, alpha, beta, keep = [0.9, 0.75] }) {
    const p = points(id);
    const cam = window.__scene?.activeCamera;
    if (!p || !cam) return 0;
    const [sa, ca, sb, cb] = [Math.sin(alpha), Math.cos(alpha), Math.sin(beta), Math.cos(beta)];
    const tanUp = Math.tan(cam.fov / 2) * keep[1];
    const tanAcross = Math.tan(cam.fov / 2) * (innerWidth / innerHeight) * keep[0];
    let r = 0;
    for (let i = 0; i < p.length; i += 3) {
      const [x, y, z] = [p[i] - target[0], p[i + 1] - target[1], p[i + 2] - target[2]];
      const toward = x * ca * sb + y * cb + z * sa * sb;
      const up = -x * ca * cb + y * sb - z * sa * cb;
      const across = -x * sa + z * ca;
      r = Math.max(r, toward + Math.abs(up) / tanUp, toward + Math.abs(across) / tanAcross);
    }
    return r;
  }
  const frame = () => {
    if (w) {
      try {
        sample();
      } catch (err) {
        if (w.errors.length < 5) w.errors.push(String(err));
      }
    }
    requestAnimationFrame(frame);
  };
  requestAnimationFrame(frame);
  window.__check = {
    loading,
    bounds,
    fit,
    /** The page's clock now, and the toasts' texts since such a time. */
    now: () => performance.now(),
    toasts: (since = 0) => toasts.filter((x) => x.t >= since).map((x) => x.text),
    watch(o) {
      w = { ...o, t0: performance.now(), frames: 0, clips: {}, resident: [], last: '', usingAt: null, slowedAt: null, busyAt: 0, errors: [] };
    },
    watched: () => w && { t: performance.now() - w.t0, usingAt: w.usingAt, busyAt: w.busyAt, slowedAt: w.slowedAt },
    stop() {
      // The frame the caller saw last (a clip just back at rest), too.
      if (w) sample();
      const r = w && { ...w, last: undefined, fps: +((w.frames * 1000) / Math.max(1, performance.now() - w.t0)).toFixed(1) };
      w = null;
      return r;
    },
  };
}

/**
 * A page at 1 % volume, the camera never taking over on its own (the director's delay 0), and free
 * will off unless `--free-will` (the game's setting applies to the player's household as a game
 * opens, over what a state says). Reused for every job of a run.
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
  await context.addInitScript(pageHelpers);
  const page = await context.newPage();
  page.on('pageerror', (err) => noteError(`page error: ${err.message}`));
  page.on('console', (m) => {
    if (m.type() === 'error' || (m.type() === 'warning' && /lights only|unavailable|failed/i.test(m.text()))) noteError(`console ${m.type()}: ${m.text()}`);
  });
  page.on('response', (r) => r.status() >= 400 && noteError(`HTTP ${r.status()} ${r.url()}`));
  page.on('requestfailed', (r) => !/ERR_ABORTED/.test(r.failure()?.errorText ?? '') && noteError(`request failed: ${r.url()} (${r.failure()?.errorText})`));
  return page;
}

function pageUrl(path, params) {
  const q = new URLSearchParams(common);
  for (const [k, v] of Object.entries(params)) if (v !== undefined && v !== null) q.set(k, String(v));
  return `${base}${path}?${q}`;
}

/** Whether a picture of an earlier run was written over (said once). */
let overwrote = false;
async function shot(page, name, extra = {}) {
  const file = join(out, `${name}.png`);
  if (!overwrote && existsSync(file) && !log.shots.some((s) => s.file === file)) {
    overwrote = true;
    console.warn(`check: ${file} was there from an earlier run and is written over (--name <run> keeps both)`);
  }
  await page.screenshot({ path: file });
  log.shots.push({ file, ...extra });
  console.log(file);
  return file;
}

const settle = (page, ms) => page.waitForTimeout(ms);
const camera = () => ({ alpha: flag('alpha'), beta: flag('beta'), radius: flag('radius'), target: flag('target') });
/** A file name part from a model key or content id. */
const slug = (s) => s.replace(/^model\./, '').replace(/[^A-Za-z0-9_.-]+/g, '_');

/** Opens a page of the benches and waits until it says it's drawn (`__ready`). */
async function openBench(page, path, params) {
  await page.goto(pageUrl(path, params), { timeout: 90_000 });
  await page.waitForFunction(() => window.__ready === true, null, { timeout: 60_000 });
}

if (what === 'gallery') {
  const keys = need('keys');
  const params = { cols: flag('cols'), scale: flag('scale'), camera: flag('camera'), ...camera() };
  const game = flag('camera') === 'game';
  const page = await newPage();
  // With the game camera every model gets its own picture, aimed at it in turn.
  for (const key of has('each') ? keys.split(',') : [keys]) {
    const name = has('each') ? `gallery-${slug(key)}` : `gallery-${keys.replace(/model\./g, '').replace(/,/g, '+')}`.slice(0, 120);
    startJob(name);
    try {
      await openBench(page, '/gallery.html', { keys: key, ...params });
      await settle(page, 1500);
      if (game) {
        // As far off as players see things.
        for (const [n, k] of key.split(',').entries()) {
          await page.evaluate((i) => window.__gallery.aim(i), n);
          await settle(page, 500);
          await shot(page, `gallery-game-${slug(k)}`, { kind: 'model', key: k, camera: 'game' });
        }
      } else {
        await shot(page, name, { kind: 'model', key });
      }
    } catch (err) {
      noteError(`no picture: ${err.message.split('\n')[0]}`);
    }
  }
}

if (what === 'clips') {
  const keys = need('keys');
  const times = list('times', '0,0.3,0.6,1,99');
  const page = await newPage();
  for (const key of has('each') ? keys.split(',') : [keys]) {
    const name = has('each') ? `clips-${slug(key)}` : `clips-${keys.replace(/model\./g, '').replace(/,/g, '+')}`.slice(0, 100);
    startJob(name);
    try {
      // One page a key, held at each time in turn (`__clips.set`), from its first.
      await openBench(page, '/clips.html', { keys: key, clip: flag('clip'), t: times[0], colors: flag('colors'), cols: flag('cols'), ...camera() });
      for (const t of times) {
        await page.evaluate((t) => window.__clips.set(t), Number(t));
        await settle(page, 800);
        await shot(page, `${name}-${flag('clip') ?? 'all'}-t${t}`, { key, t });
      }
    } catch (err) {
      noteError(`no picture: ${err.message.split('\n')[0]}`);
    }
  }
}

if (what === 'play') {
  const state = flag('state', 'bathroom-check');
  const batch = Boolean(flag('objects') || flag('objects-file') || flag('use-existing'));
  /** `x,z,rot` as numbers (null: none given); anything else stops the run. */
  const place = (text, line) => {
    if (!text) return null;
    const at = text.split(',').map(Number);
    if (at.length !== 3 || at.some((n) => !Number.isInteger(n))) fail(`'${text}' in '${line}' isn't a place: x,z,rot`);
    return at;
  };
  // `a,b@23,14,0,c` (or with spaces between, for helpers): the numbers after an `@` belong to it.
  const items = (text, by = ',') =>
    text
      .split(by)
      .map((s) => s.trim())
      .filter(Boolean)
      .reduce((all, part) => (/^-?[\d.]+$/.test(part) && all.length && all.at(-1).includes('@') ? (all[all.length - 1] += `,${part}`) : all.push(part), all), []);
  /** A helper (`def` or `def@x,z,rot`): where the game finds room without a place. */
  const helper = (item, line) => {
    const [def, at] = item.split('@');
    return { def, at: place(at, line) };
  };
  const withAll = items(flag('with', '')).map((h) => helper(h, '--with'));
  /**
   * Each thing to use: its content id; where (`[x, z, rot]`, if given) or `existing` (the state's
   * own); and its helpers (`with`), the line's after `--with`'s.
   */
  const parse = (line) => {
    const [main, ...more] = line.split(/\s+with\s+/);
    const [def, at] = main.split('@').map((s) => s.trim());
    const helpers = [...withAll, ...more.flatMap((m) => items(m, /[\s,]+/)).map((h) => helper(h, line))];
    if (at === 'existing') return { def, existing: true, at: null, helpers };
    return { def, at: place(at, line) ?? place(flag('at'), '--at'), helpers };
  };
  const things = batch
    ? [
        ...items(flag('use-existing', '')).map((def) => parse(`${def}@existing`)),
        ...(flag('objects-file') ? readFileSync(flag('objects-file'), 'utf8').split('\n') : items(flag('objects', '')))
          .map((l) => l.replace(/#.*/, '').trim())
          .filter(Boolean)
          .map(parse),
      ]
    : [{ def: need('object'), at: place(flag('at', '23,14,0'), '--at'), helpers: withAll }];
  /** `--skills cooking=5,creativity=3`: the resident's, set each time the state opens. */
  const skills = items(flag('skills', '')).map((p) => {
    const [skill, level] = p.split('=');
    if (!skill || !Number.isFinite(Number(level))) fail(`'${p}' in --skills isn't <skill>=<level>`);
    return { skill, level: Number(level) };
  });
  const sim = num('sim', 0);
  const which = flag('use', batch ? 'all' : '0');
  const shots = num('shots', batch ? 3 : 6);
  const every = num('every', batch ? 1000 : 500);
  const cap = num('wait', 60) * 1000;
  const speed = num('speed', 1);
  // Never above 10× (speed 3): at 30× short uses are over between two frames.
  const walk = Math.min(3, num('walk', batch ? 3 : speed));
  // The old single object: the camera follows the resident.
  const follow = !batch || has('follow');
  const colors = flag('colors') ? Object.fromEntries(flag('colors').split(',').map((p) => p.split(':')).map(([k, v]) => [k, Number(v)])) : undefined;
  const turn = num('turn', 0);
  // A night state's light is its clock's (a fixed noon would light the night up).
  const hourFlag = flag('hour', batch && !/-night$/.test(state) ? '12' : undefined);
  const hour = hourFlag === 'state' ? undefined : hourFlag;
  /** The stage's states (`review-stage`, `-night`, `review-garden`): the same free spots. */
  const stage = state.startsWith('review-');
  /** Pictures of a named run start with its name. */
  const prefix = flag('name') ? `${run}-` : '';
  const records = join(out, `play-${run}.jsonl`);
  writeFileSync(records, '');
  const write = (record) => appendFileSync(records, JSON.stringify(record) + '\n');

  let page = await newPage();
  /** Until nothing is loading (each loading screen's text), at most `timeout` ms. */
  const loaded = (timeout = 90_000) => page.waitForFunction(() => !window.__check.loading(), null, { timeout, polling: 250 });

  /** Opens the state (again): in play, nothing loading, free will off, money in. */
  async function openState() {
    const t0 = Date.now();
    await page.goto(pageUrl('/', { state, debug: '', hour }), { timeout: 90_000 });
    // In play: the world is there and the household's portraits are drawn.
    await page.waitForFunction(() => typeof window.__sims === 'function' && window.__world?.() && window.__sims().length > 0, null, { timeout: 90_000 });
    await loaded();
    await settle(page, 1500);
    const unknown = await page.evaluate(({ freeWill, sim, skills }) => {
      window.__send({ type: 'setAutonomy', enabled: freeWill, household: 0 });
      window.__addFunds(1_000_000);
      const known = window.__content.skills.map((s) => s.id);
      for (const s of skills) if (known.includes(s.skill)) window.__send({ type: 'setSkill', sim, skill: s.skill, level: s.level });
      return skills.map((s) => s.skill).filter((s) => !known.includes(s));
    }, { freeWill: has('free-will'), sim, skills });
    if (unknown.length) fail(`no skill ${unknown.join(', ')} in the content`);
    const s = await resident();
    if (!s) throw new Error(`no resident ${sim} in ${state}`);
    park = [s.x, s.z];
    return Date.now() - t0;
  }
  /** Where the resident stood as the state opened: they go back there after each object, out of the way of the next. */
  let park = null;
  async function goPark() {
    await page.evaluate(({ sim, walk, park }) => {
      window.__send({ type: 'setSpeed', speed: walk });
      window.__send({ type: 'moveTo', sim, x: park[0], z: park[1] });
    }, { sim, walk, park });
    await page
      .waitForFunction(({ sim, park }) => {
        const s = window.__sims().find((x) => x.id === sim);
        return s && !s.moving && Math.hypot(s.x - park[0], s.z - park[1]) < 1;
      }, { sim, park }, { timeout: 15_000, polling: 200 })
      .catch(() => noteError(`not back at ${park} after 15 s`));
    await page.evaluate(() => window.__send({ type: 'setSpeed', speed: 1 }));
  }

  /** Where `def` goes by default on the `review-*` states (`check_states.rs`), by its layer and footprint; null: anywhere free. */
  const stageSpot = (def) =>
    page.evaluate((id) => {
      const d = window.__content.object(id);
      if (!d) return null;
      const [w, dd] = d.footprint ?? [1, 1];
      const x = 10 - Math.floor(w / 2);
      if (d.layer === 'surface') return null;
      if (d.outdoors) return [x, 22, 0];
      if (d.layer === 'wall') return d.overWindow ? [4, 14, 1] : [x, 18 - dd, 2];
      // Against the back wall what fills its height (a wardrobe) or stands against a wall (a wall bed); the rest in the middle.
      return d.fillsWall || d.againstWall ? [x, 18 - dd, 2] : [x, 12 - Math.floor(dd / 2), 0];
    }, def);

  /**
   * The object's footprint middle, its facing and layer, and for the camera the model's bounds as
   * drawn (null: gone). `bounds`: its body's corners (`__check.bounds`: a tree's crown reaches well
   * past its footprint); `box`: its height and bottom (`objectBox`); either null when it isn't
   * drawn yet.
   */
  const placed = (id) =>
    page.evaluate((id) => {
      const o = window.__world().objects.find((x) => x.id === id);
      if (!o) return null;
      const d = window.__content.object(o.def);
      const [fw, fd] = d?.footprint ?? [1, 1];
      const [w, dd] = o.rot % 2 ? [fd, fw] : [fw, fd];
      const box = window.__renderer?.objectBox?.(id) ?? null;
      return { x: o.x, z: o.z, rot: o.rot, open: Boolean(o.open), growth: o.growth ?? null, mid: [o.x + w / 2, o.z + dd / 2], size: [w, dd], layer: d?.layer ?? 'floor', box, bounds: window.__check.bounds(id), fov: window.__scene?.activeCamera?.fov ?? 0.7 };
    }, id);

  /** Models up to this tall (m) are looked at from the usual height; taller ones at their middle. */
  const TALL = 1.6;

  /**
   * Frames the object (not the resident): its footprint's middle, from in front and a little to
   * the side, as far off as its size needs and the model's bounds as drawn (a tree's crown, a
   * wardrobe, a shower: never cut off), a tall one looked at halfway up. Without bounds (not
   * drawn yet) its height from `objectBox`, else a guess from the footprint. Set again before each
   * picture (the intro's tween and the game's own watching move the camera). The view.
   */
  async function aim(id, use) {
    if (follow) return null;
    const p = await placed(id);
    if (!p) return null;
    const { mid, size, layer, rot, box, bounds, fov } = p;
    // The way it faces (rot 0: +z), so the camera stands in front, 0.7 rad round to the side (the
    // resident at it hides less of it).
    const facing = [Math.PI / 2, 0, -Math.PI / 2, Math.PI][rot % 4];
    const alpha = num('alpha', facing + 0.7);
    const beta = num('beta', layer === 'ceiling' ? 1.3 : 0.95);
    const [bottom, height, from] = bounds ? [bounds.lo[1], bounds.hi[1], 'bounds'] : box ? [box.bottom, box.height, 'box'] : [0, layer === 'floor' ? 1 + 0.5 * Math.max(...size) : 0.8, 'guess'];
    const tall = layer !== 'ceiling' && height - bottom > TALL;
    const usual = layer === 'wall' ? 1.3 : layer === 'ceiling' ? 1.9 : use?.pose === 'lie' ? 0.5 : 0.7;
    const target = [mid[0], num('y', tall ? (bottom + height) / 2 : usual), mid[1]];
    // At least as far as the footprint needs (with room for the resident at it); further when the
    // model's vertices need it (`__check.fit`), or without them a tall one's height as seen from
    // `beta`. (The game camera never comes nearer than its own limit, 6.)
    const near = Math.max(3.2, 1.2 * Math.max(...size) + 2.4);
    const across = 0.5 * ((height - bottom) * Math.sin(beta) + Math.max(...size) * Math.abs(Math.cos(beta)));
    const fit = bounds ? await page.evaluate((a) => window.__check.fit(a.id, a), { id, target, alpha, beta }) : tall ? (1.4 * across) / Math.tan(fov / 2) : 0;
    const view = { alpha, beta, radius: num('radius', Math.max(near, fit)), target };
    // The pose the camera takes (its radius within its limits, as from its next frame).
    const took = await page.evaluate((view) => {
      window.__controls.follow(null);
      const pose = window.__camera(view);
      const cam = window.__scene?.activeCamera;
      return { ...pose, radius: Math.min(Math.max(pose.radius, cam?.lowerRadiusLimit ?? 0), cam?.upperRadiusLimit ?? Infinity) };
    }, view);
    const round = (v) => v.map((x) => +x.toFixed(2));
    return { alpha: +took.alpha.toFixed(3), beta: +took.beta.toFixed(3), radius: +took.radius.toFixed(2), target: round(took.target), height: +height.toFixed(2), bottom: +bottom.toFixed(2), from, ...(bounds ? { bounds: { lo: round(bounds.lo), hi: round(bounds.hi) } } : {}) };
  }

  /** The resident as `__sims()` has them (null: not there). */
  const resident = () =>
    page.evaluate((sim) => {
      const s = window.__sims().find((x) => x.id === sim);
      return s && { pose: s.pose, using: s.using, object: s.object, interaction: s.interaction, action: s.action, doing: s.doing, queue: s.queue, x: +s.x.toFixed(2), z: +s.z.toFixed(2), needs: s.needs };
    }, sim);

  /** Buys `def` (at `at`, or where the game finds room); its id and where it stands, or an error. */
  async function buy(def, at) {
    await loaded();
    const [before, sent] = await page.evaluate(() => [window.__world().objects.map((o) => o.id), window.__check.now()]);
    await page.evaluate(
      ({ def, at, colors, turn }) => window.__send({ type: 'buy', household: 0, object: def, ...(at ? { at } : {}), ...(colors ? { colors } : {}), ...(turn ? { turn } : {}) }),
      { def, at, colors, turn },
    );
    const id = await page
      .waitForFunction(({ def, before }) => window.__world().objects.find((o) => o.def === def && !before.includes(o.id))?.id, { def, before }, { timeout: 10_000 })
      .then((h) => h.jsonValue())
      .catch(() => null);
    if (id === null) {
      // The game's own reason, from its toast.
      const said = await page.evaluate((sent) => window.__check.toasts(sent), sent);
      return { error: `not bought: ${def}${at ? ` at ${at}` : ''} (${said.length ? `the game: ${said.join('; ')}` : `is the spot free? on screen: ${await screenText()}`})` };
    }
    const where = await placed(id);
    // A late or misplaced buy puts the wrong thing in a plausible picture: say so.
    const mismatch = at && (where.x !== at[0] || where.z !== at[1] || where.rot !== at[2]) ? `stands at ${where.x},${where.z},${where.rot}, not ${at}` : null;
    return { id, where, mismatch };
  }

  const screenText = () => page.evaluate(() => document.body.innerText.replace(/\s+/g, ' ').slice(-300));

  /** Sells `id` (stopping whoever uses it first); whether it went. */
  async function sell(id) {
    await page.evaluate((id) => window.__send({ type: 'sell', household: 0, object: id }), id);
    return page
      .waitForFunction((id) => !window.__world().objects.some((o) => o.id === id), id, { timeout: 10_000 })
      .then(() => true)
      .catch(() => false);
  }

  /** Stops what the resident does and waits until they're free (and the object's use clips at rest, but for `held` ones). */
  async function stop(id, held) {
    for (let i = 0; i < 3; i++) {
      await page.evaluate((sim) => {
        window.__send({ type: 'setSpeed', speed: 1 });
        window.__send({ type: 'cancel', sim, index: 0 });
      }, sim);
      const free = await page
        .waitForFunction((sim) => {
          const s = window.__sims().find((x) => x.id === sim);
          return s && !s.doing && !s.queue.length && !s.using;
        }, sim, { timeout: 8_000, polling: 200 })
        .then(() => true)
        .catch(() => false);
      if (free) break;
    }
    // A use's clips close as it ends (a door swinging shut): wait for them, a few seconds at most.
    await page
      .waitForFunction(
        ({ id, held }) => ![...(window.__renderer?.motion?.states?.get(id) ?? [])].some(([name, c]) => ['use', 'loop', 'repeat'].includes(c.kind) && c.time > 1e-3 && !held.includes(name)),
        { id, held },
        { timeout: 5_000, polling: 'raf' },
      )
      .catch(() => {});
  }

  /** The clips that always run, or run without a use (ambient, growth, night, clock hands): not "a clip played". */
  const background = (id) =>
    page.evaluate((id) => {
      const m = window.__renderer?.motion?.objects?.get(id)?.motions;
      return { names: m ? [m.idle, m.growth, m.night, m.clock?.hour, m.clock?.minute].filter(Boolean) : [], open: m?.open ?? null, any: Boolean(m) };
    }, id);

  /**
   * One use: watched from the frame it's sent, walking fast and slowing down near the object, up
   * to `cap` ms until the resident is using it, then `shots` pictures `every` ms apart framing the
   * object, then stopped. One record in `play-<run>.jsonl`.
   */
  async function runUse(thing, bought, use, attempt, helper = null) {
    // (A kind used twice in a run, in two places: the place in its pictures' names too.)
    const twice = things.filter((t) => t.def === thing.def).length > 1 ? `-${bought.where.x}-${bought.where.z}-${bought.where.rot}` : '';
    const name = `${prefix}${slug(thing.def)}${twice}-${slug(use.id)}`;
    const record = { object: thing.def, id: bought.id, at: [bought.where.x, bought.where.z, bought.where.rot], use: use.id, index: use.index, label: use.label, pose: use.pose ?? null, anim: use.anim ?? null, attempt, ...(helper ? { helper } : {}), ...about(thing, bought), outcome: null, shots: [], errors: [] };
    const j = startJob(`${thing.def}/${use.id}`, { attempt });
    const t0 = Date.now();
    await loaded();
    const bg = await background(bought.id);
    record.before = await resident();
    const near = { x: bought.where.mid[0], z: bought.where.mid[1], r: 3 + Math.max(...bought.where.size) / 2, speed };
    const sent = await page.evaluate(() => window.__check.now());
    await page.evaluate(
      ({ id, sim, interaction, walk, near, follow, view }) => {
        window.__check.watch({ id, sim, near });
        window.__send({ type: 'setSpeed', speed: walk });
        window.__send({ type: 'use', sim, object: id, interaction });
        if (follow) window.__controls.follow(sim, view);
      },
      { id: bought.id, sim, interaction: use.index, walk, near, follow, view: Object.fromEntries(Object.entries({ radius: num('radius', 7), beta: num('beta'), alpha: num('alpha'), y: num('y') }).filter(([, v]) => v)) },
    );
    // Using it (seen in a frame), not taken (nothing to do for 4 s), or out of time.
    record.outcome = await page
      .waitForFunction(
        (cap) => {
          const w = window.__check.watched();
          if (w.usingAt !== null) return 'using';
          if (w.t > 4000 && w.t - w.busyAt > 4000) return 'not taken';
          return w.t > cap ? 'timeout' : false;
        },
        cap,
        { timeout: cap + 30_000, polling: 'raf' },
      )
      .then((h) => h.jsonValue())
      .catch((err) => `error: ${err.message.split('\n')[0]}`);
    record.usingAfter = (await page.evaluate(() => window.__check.watched()?.usingAt)) ?? null;
    if (record.outcome !== 'using') record.screen = await screenText();
    // What the game said meanwhile (a use refused: "needs a table", "isn't ripe yet").
    record.toasts = await page.evaluate((sent) => window.__check.toasts(sent), sent);
    if (record.outcome === 'using' && !helper) {
      for (let i = 0; i < shots; i++) {
        await loaded();
        record.view = (await aim(bought.id, use)) ?? undefined;
        await settle(page, 120);
        const info = await page.evaluate(
          ({ sim, id }) => {
            const s = window.__sims().find((x) => x.id === sim);
            const clips = window.__renderer?.motion?.states?.get(id);
            const o = window.__world().objects.find((x) => x.id === id);
            // Each clip playing on it: how far in (s), of how long, and how it plays.
            const playing = clips ? Object.fromEntries([...clips].map(([name, c]) => [name, { time: +c.time.toFixed(2), duration: +c.clip.duration.toFixed(2), kind: c.kind }])) : null;
            return {
              t: Math.round(window.__check.watched()?.t ?? 0),
              object: o ? { x: o.x, z: o.z, rot: o.rot, open: Boolean(o.open) } : null,
              sim: s && { pose: s.pose, using: s.using && s.object === id, interaction: s.interaction, action: s.action, x: +s.x.toFixed(2), z: +s.z.toFixed(2) },
              clips: playing,
            };
          },
          { sim, id: bought.id },
        );
        const file = await shot(page, batch ? `${name}-${String(i).padStart(2, '0')}` : `${prefix}play-${thing.def}-${bought.where.x}-${bought.where.z}-${bought.where.rot}-${String(i).padStart(2, '0')}`, info);
        record.shots.push({ file, ...info });
        if (i < shots - 1) await settle(page, every);
      }
    }
    // Held open whatever the use: the night clip; the open one while the object stays open.
    await stop(bought.id, [...bg.names, ...(bg.open ? [bg.open] : [])]);
    const watched = await page.evaluate(() => window.__check.stop());
    record.after = await resident();
    record.clips = watched?.clips ?? {};
    // Played: a clip of a use (not the ambient, growth, night or clock ones) that moved.
    record.played = Object.entries(record.clips)
      .filter(([n, c]) => !bg.names.includes(n) && !['idle', 'growth', 'clock'].includes(c.kind) && c.max > 0)
      .map(([n]) => n);
    record.motions = bg.any;
    record.resident = watched?.resident ?? [];
    record.timings = { total: Date.now() - t0, usingAt: watched?.usingAt ?? null, slowedAt: watched?.slowedAt ?? null, fps: watched?.fps ?? null };
    record.errors = [...j.errors, ...(watched?.errors ?? [])];
    record.open = Boolean((await placed(bought.id))?.open);
    write(record);
    console.log(`${record.object} / ${record.use}${helper ? ` (${helper})` : ''}: ${record.outcome}${record.usingAfter !== null ? ` after ${(record.usingAfter / 1000).toFixed(1)} s` : ''}, ${(record.timings.total / 1000).toFixed(1)} s, played ${record.played.join(', ') || 'nothing'}`);
    return record;
  }

  /** The object's uses to run, each with what it needs (`when`) and leaves (`leaves`). */
  const usesOf = (def) =>
    page.evaluate((def) => (window.__content.object(def)?.interactions ?? []).map((i, index) => ({ id: i.id, index, label: i.label, pose: i.pose, anim: i.anim, when: i.when, leaves: i.leaves })), def);

  /** Puts an object that is open or closed the way `use` needs it (a wall bed down to sleep in), with one of its own uses. */
  async function ready(thing, bought, uses, want, attempt) {
    const open = Boolean((await placed(bought.id))?.open);
    if (want === undefined || want === open) return;
    const by = uses.find((u) => u.leaves === (want ? 'open' : 'closed') && u.when !== (want ? 'open' : 'closed'));
    if (by) await runUse(thing, bought, by, attempt, want ? 'open' : 'close');
  }

  /** What a record says of how its object was set up: the state's own, its helpers, the skills given. */
  const about = (thing, bought) => ({
    ...(thing.existing ? { existing: true } : {}),
    ...(bought.with?.length ? { with: bought.with } : {}),
    ...(skills.length ? { skills: Object.fromEntries(skills.map((s) => [s.skill, s.level])) } : {}),
  });

  /** The state's own `def` (the first of its kind): its id and where it stands, or an error. */
  async function existing(def) {
    await loaded();
    const id = await page.evaluate((def) => window.__world().objects.find((o) => o.def === def)?.id ?? null, def);
    if (id === null) return { error: `no ${def} in ${state} (--use-existing takes what the state has)` };
    return { id, where: await placed(id) };
  }

  /** Sells what was bought for one object, the last bought first; whether each went is logged. */
  async function sellAll(ids) {
    for (const { def, id } of [...ids].reverse()) if (!(await sell(id))) noteError(`not sold: ${def} (${id})`);
  }

  /**
   * Buys `thing`'s helpers, then `thing` (or takes the state's own), runs `uses` (those `pick`
   * keeps), sells what it bought. The uses that failed.
   */
  async function runObject(thing, pick, attempt) {
    startJob(`${thing.def} (buy)`, { attempt });
    const missing = await page.evaluate((defs) => defs.filter((d) => !window.__content.object(d)), [thing.def, ...thing.helpers.map((h) => h.def)]);
    if (missing.length) {
      noteError(`no object '${missing.join("', '")}' in the content`);
      write({ object: thing.def, use: null, attempt, outcome: 'unknown object', unknown: missing });
      return { failed: [], bought: false, unknown: true };
    }
    // The helpers first (a desk, a table), where asked or wherever the game finds room.
    const helpers = [];
    for (const h of thing.helpers) {
      const got = await buy(h.def, h.at);
      if (got.error) {
        noteError(got.error);
        write({ object: thing.def, use: null, attempt, outcome: 'helper not bought', helper: h.def, at: h.at, errors: [got.error] });
        await sellAll(helpers);
        return { failed: null, bought: false };
      }
      if (got.mismatch) noteError(got.mismatch);
      helpers.push({ def: h.def, id: got.id, at: [got.where.x, got.where.z, got.where.rot] });
    }
    const at = thing.existing ? null : (thing.at ?? (stage ? await stageSpot(thing.def) : null));
    const bought = thing.existing ? await existing(thing.def) : await buy(thing.def, at);
    if (bought.error) {
      noteError(bought.error);
      write({ object: thing.def, use: null, at, attempt, outcome: thing.existing ? 'not there' : 'not bought', errors: [bought.error] });
      await sellAll(helpers);
      return { failed: null, bought: false };
    }
    if (bought.mismatch) noteError(bought.mismatch);
    bought.with = helpers.map((h) => `${h.def}@${h.at.join(',')}`);
    const uses = await usesOf(thing.def);
    const chosen = uses.filter(pick);
    if (!chosen.length) {
      // Decor (no uses): bought, one picture, sold.
      const offered = uses.map((u) => u.id);
      await aim(bought.id);
      await settle(page, 300);
      const view = await aim(bought.id);
      await settle(page, 300);
      const file = await shot(page, `${prefix}${slug(thing.def)}-${uses.length ? 'no-such-use' : 'no-uses'}`);
      write({ object: thing.def, id: bought.id, at: [bought.where.x, bought.where.z, bought.where.rot], use: which, attempt, ...about(thing, bought), outcome: uses.length ? 'no such use' : 'no uses', offers: offered, view, shots: [{ file }] });
      console.warn(uses.length ? `${thing.def} has no interaction '${which}' (it offers: ${offered.join(', ')})` : `${thing.def} has no uses: one picture`);
    }
    const failed = [];
    for (const use of chosen) {
      // A crop bought new is just planted, one picked lies fallow: its "when ripe" uses aren't
      // offered (`review-garden`'s are ripe: `--use-existing`).
      if (use.when === 'ripe' && (await placed(bought.id))?.growth !== 2) {
        const reason = thing.existing ? 'only when ripe (not ripe now)' : 'only when ripe (bought new, a crop is just planted: --use-existing it on review-garden)';
        write({ object: thing.def, use: use.id, index: use.index, attempt, ...about(thing, bought), outcome: 'skipped', reason });
        continue;
      }
      await ready(thing, bought, uses, use.when === 'open' ? true : use.when === 'closed' ? false : undefined, attempt);
      const record = await runUse(thing, bought, use, attempt);
      if (record.outcome !== 'using') failed.push({ ...use, outcome: record.outcome });
    }
    // Shut what a use left open (a wall bed folded down), then sell it (not the state's own) and
    // its helpers.
    await ready(thing, bought, uses, false, attempt);
    await sellAll([...helpers, ...(thing.existing ? [] : [{ def: thing.def, id: bought.id }])]);
    await goPark();
    return { failed, bought: true };
  }

  const pickUse = (u) => (which === 'all' ? true : /^\d+$/.test(which) ? u.index === Number(which) : u.id === which);
  const timings = [];
  log.setup.open = await openState();
  for (const thing of things) {
    const t0 = Date.now();
    let result = null;
    for (let attempt = 0; attempt < 2; attempt++) {
      try {
        // A long run wears the resident down (a use ends when its need is full, or runs out), and a
        // retry after anything but uses not taken (not for them: a baby's, one needing something
        // near) starts afresh.
        // (No resident to ask: the page broke, so afresh too.)
        const now = await resident().catch(() => null);
        const lowest = now ? Math.min(...Object.values(now.needs ?? { none: 1 })) : 0;
        const afresh = attempt > 0 && (!result?.bought || result.failed.some((u) => u.outcome !== 'not taken'));
        if (afresh || lowest < 0.25) log.setup[`reopened-${thing.def}-${attempt}`] = await openState();
        const pick = attempt === 0 || !result?.failed ? pickUse : (u) => result.failed.some((f) => f.index === u.index);
        result = await runObject(thing, pick, attempt);
        if (result.unknown || (result.bought && !result.failed.length)) break;
        console.warn(`${thing.def}: ${result.bought ? `${result.failed.map((u) => u.id).join(', ')} failed` : 'not bought'}${attempt === 0 ? ', once more' : ''}`);
      } catch (err) {
        noteError(`${thing.def}: ${err.message.split('\n')[0]}`);
        write({ object: thing.def, use: null, attempt, outcome: `error: ${err.message.split('\n')[0]}` });
        // A page that broke: a new one.
        if (page.isClosed()) page = await newPage();
        result = null;
      }
    }
    timings.push({ object: thing.def, seconds: Math.round((Date.now() - t0) / 100) / 10 });
  }
  log.timings = timings;
}

if (what === 'sheet') {
  const dir = resolve(need('dir'));
  const folder = basename(dir);
  /** Life stages as pictures name them, in the order a sheet shows them (`default`: the page's own, grown-ups). */
  const STAGES = ['baby', 'child', 'teen', 'youngAdult', 'adult', 'default', 'elder'];
  /** Most pictures a sheet: 2 × 2, or 3 × 2. */
  const TILES = 6;
  const lines = (file) => readFileSync(file, 'utf8').split('\n').map((l) => l.replace(/#.*/, '').trim()).filter(Boolean);
  // The objects: given, else the review's list for this folder (`lists/<cat>.txt`, or the models for
  // folders of models), else the ids in backticks that open the rows of the folder's index.
  const lists = join(dir, '..', 'lists');
  const listed = [`${folder}.txt`, `${folder}.models.txt`].map((f) => join(lists, f)).find((f) => existsSync(f));
  const ids = flag('ids')
    ? list('ids')
    : flag('ids-file')
      ? lines(flag('ids-file'))
      : listed
        ? lines(listed)
        : [...new Set(readFileSync(join(dir, 'index.md'), 'utf8').split('\n').flatMap((l) => l.match(/^\|(?:[^|`]*\|)?\s*`([^`]+)`/)?.[1] ?? []))];
  if (!ids.length) fail(`no objects for ${dir}: give --ids or --ids-file`);
  // Each object's model, from the game's content: gallery and clip pictures are named by it.
  const contentDir = join(WEB, 'public/content');
  const baseContent = JSON.parse(readFileSync(join(contentDir, 'base.json'), 'utf8'));
  const models = new Map();
  for (const file of [baseContent, ...(baseContent.include ?? []).map((f) => JSON.parse(readFileSync(join(contentDir, f), 'utf8')))]) {
    for (const o of file.objects ?? []) models.set(o.id, o.model);
  }
  // Every picture in the folder but the sheets, with the folders it is in and when it was taken.
  const pngs = readdirSync(dir, { recursive: true })
    .filter((rel) => rel.endsWith('.png') && !rel.split(sep).includes('sheets') && !resolve(dir, rel).startsWith(out + sep))
    .map((rel) => ({ rel, name: basename(rel), dirs: rel.split(sep).slice(0, -1), time: statSync(join(dir, rel)).mtimeMs }));
  // `poses.html`'s own stations, whose shots are named by label ("feed baby": the crib's): label → object.
  const bare = (s) => s.toLowerCase().replace(/[^a-z0-9]/g, '');
  const stationDefs = new Map([...readFileSync(join(WEB, 'src/debug/poses.ts'), 'utf8').matchAll(/\{ label: '([^']+)', def: '([^']+)'/g)].map(([, label, def]) => [bare(label), def]));
  /** Whether a pose shot is of object `id`: the station its name ends with ("…-2-sleep-two"), else an id in its name ("poses-seats-teen-sofa"). */
  function posedOn(stem, id, names) {
    const words = stem.split(/[-_]/);
    for (let k = 1; k < words.length; k++) {
      const def = stationDefs.get(bare(words.slice(k).join('')));
      if (def) return def === id;
    }
    return stem.split('-').some((t) => names.includes(t));
  }
  const escape = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const stageIn = (tokens) => STAGES.find((s) => tokens.includes(s));
  /** The newest picture of each file name (a use taken again in a later chunk), in name order. */
  const newest = (files) => [...files.reduce((m, f) => ((m.get(f.name)?.time ?? -1) >= f.time ? m : m.set(f.name, f)), new Map()).values()].sort((a, b) => a.name.localeCompare(b.name));
  /** The one nearest the folder's top (the main run, not a redo in a subfolder), newest first. */
  const first = (files) => [...files].sort((a, b) => a.dirs.length - b.dirs.length || b.time - a.time)[0];

  /** An object's pictures, by what they show, found by the names `gallery`, `clips`, `play` and `poses.html` shots get. */
  function pictures(id) {
    const names = [...new Set([slug(id), models.has(id) ? slug(models.get(id)) : null].filter(Boolean))];
    const any = names.map(escape).join('|');
    const gallery = new RegExp(`^gallery-(${any})\\.png$`);
    const game = new RegExp(`^gallery-game-(${any})\\.png$`);
    const clip = new RegExp(`^clips-(${any})-(.+)-t([\\d.]+)\\.png$`);
    const use = new RegExp(`^(${any})-(.+?)(?:-(\\d+))?\\.png$`);
    const found = { gallery: [], close: [], game: [], clips: [], uses: [], night: [], staged: [], ages: [] };
    for (const f of pngs) {
      const stem = f.name.slice(0, -4);
      if (gallery.test(f.name)) (f.dirs.includes('close') ? found.close : found.gallery).push(f);
      else if (game.test(f.name)) found.game.push(f);
      else if (clip.test(f.name)) found.clips.push({ ...f, clip: f.name.match(clip)[2], t: Number(f.name.match(clip)[3]) });
      else if (stem.startsWith('poses-') && posedOn(stem, id, names)) found.ages.push({ ...f, stage: stageIn(stem.split('-')) ?? 'default' });
      else if (use.test(f.name)) {
        // In play: at another life stage (a folder named for it), at night, or as the state has them.
        const stage = stageIn(f.dirs.flatMap((d) => d.split('-')));
        const [, , what, shot] = f.name.match(use);
        (stage ? found.staged : f.dirs.some((d) => d.startsWith('night')) ? found.night : found.uses).push({ ...f, use: what, shot, stage });
      }
    }
    // Each use once: its middle picture (`-01` of three), the uses with none (`no-uses`) last.
    const middles = (files, key) => {
      const byUse = new Map();
      for (const f of files) byUse.set(key(f), [...(byUse.get(key(f)) ?? []), f]);
      return [...byUse.values()].map((shots) => shots.find((f) => f.shot === '01') ?? shots[Math.floor(shots.length / 2)]);
    };
    const uses = middles(newest(found.uses), (f) => f.use).sort((a, b) => (a.use === 'no-uses') - (b.use === 'no-uses'));
    found.ages.push(...middles(found.staged, (f) => `${f.dirs.join('/')}:${f.use}`));
    // Each clip held open (its latest time), the main run's first, then other framings (`clips/side/`, `clips-tall/`).
    const open = new Map();
    for (const f of found.clips) {
      const key = `${f.dirs.join('/')}:${f.clip}`;
      if (!open.has(key) || open.get(key).t < f.t) open.set(key, f);
    }
    const clips = [...open.values()].sort((a, b) => a.dirs.length - b.dirs.length || a.rel.localeCompare(b.rel));
    const tiles = [];
    const add = (f) => f && tiles.length < TILES && !tiles.some((t) => t.rel === f.rel) && tiles.push(f);
    const night = newest(found.night);
    for (const f of [first(found.gallery), first(found.game), uses[0], clips[0], first(found.close), night[0], ...clips.slice(1), ...uses.slice(1), ...night.slice(1)]) add(f);
    // By life stage: the same picture (its name without the stage) at each stage side by side, a
    // sheet holding as many such sets as fit whole.
    const sets = new Map();
    for (const f of newest(found.ages)) {
      const key = f.rel.split(sep).map((part) => part.replace(/\.png$/, '').split('-').filter((t) => t !== f.stage).join('-')).join('/');
      sets.set(key, [...(sets.get(key) ?? []), f]);
    }
    const ages = [];
    for (const set of [...sets.values()].map((s) => s.sort((a, b) => STAGES.indexOf(a.stage) - STAGES.indexOf(b.stage)))) {
      for (let k = 0; k < set.length; k += TILES) {
        const part = set.slice(k, k + TILES);
        if (ages.length && ages.at(-1).length + part.length <= TILES) ages.at(-1).push(...part);
        else ages.push(part);
      }
    }
    return { tiles, ages };
  }

  const page = await (await browser.newContext({ viewport: { width: 1600, height: 1000 }, deviceScaleFactor: 1 })).newPage();
  const html = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;');
  /**
   * One sheet: `tiles` in a grid under `title`, each picture whole and labelled with its file; 1600
   * wide and as tall as the pictures need (about 1000 for 2 × 2, 700 for 3 × 2).
   */
  async function compose(name, title, tiles) {
    const cols = tiles.length <= 4 ? 2 : 3;
    const rows = tiles.length <= 2 ? 1 : 2;
    const data = tiles.map((f) => readFileSync(join(dir, f.rel)));
    // The tallest picture's shape (a PNG's width and height are at bytes 16 and 20).
    const aspect = Math.max(...data.map((b) => b.readUInt32BE(20) / b.readUInt32BE(16)));
    const height = Math.round(32 + 4 + rows * (((1600 - 8 - 4 * (cols - 1)) / cols) * aspect) + 4 * (rows - 1));
    await page.setViewportSize({ width: 1600, height });
    const figures = tiles.map((f, i) => `<figure><img src="data:image/png;base64,${data[i].toString('base64')}"><figcaption>${html(f.rel)}</figcaption></figure>`);
    await page.setContent(
      `<!doctype html><style>
        body { margin: 0; width: 1600px; height: ${height}px; display: flex; flex-direction: column; background: #1d1f21; color: #eee; font: 15px system-ui, sans-serif; }
        h1 { margin: 0; padding: 0 12px; height: 32px; line-height: 32px; font-size: 17px; font-weight: 600; }
        main { flex: 1; min-height: 0; display: grid; grid-template-columns: repeat(${cols}, 1fr); grid-template-rows: repeat(${rows}, 1fr); gap: 4px; padding: 0 4px 4px; }
        figure { margin: 0; position: relative; min-height: 0; background: #000; }
        img { width: 100%; height: 100%; object-fit: contain; display: block; }
        figcaption { position: absolute; left: 0; top: 0; max-width: calc(100% - 16px); padding: 3px 8px; background: #000c; color: #fff; overflow-wrap: anywhere; }
      </style><h1>${html(title)}</h1><main>${figures.join('')}</main>`,
      { waitUntil: 'load' },
    );
    await shot(page, name, { kind: 'sheet', tiles: tiles.map((f) => f.rel) });
  }

  log.sheets = [];
  log.without = [];
  for (const id of ids) {
    startJob(slug(id));
    try {
      const { tiles, ages } = pictures(id);
      const sheets = [];
      if (tiles.length) {
        await compose(`obj-${slug(id)}`, `${id} · ${folder}`, tiles);
        sheets.push(`obj-${slug(id)}.png`);
      }
      for (const [k, part] of ages.entries()) {
        const name = `age-${slug(id)}${k ? `-${k + 1}` : ''}`;
        await compose(name, `${id} · ${folder} · by life stage: ${[...new Set(part.map((f) => f.stage))].join(', ')}`, part);
        sheets.push(`${name}.png`);
      }
      log.sheets.push({ id, sheets, tiles: tiles.length, ages: ages.flat().length });
      if (!sheets.length) log.without.push(id);
    } catch (err) {
      noteError(`no sheet: ${err.message.split('\n')[0]}`);
      log.without.push(id);
    }
  }
  const count = (prefix) => log.sheets.reduce((n, s) => n + s.sheets.filter((f) => f.startsWith(prefix)).length, 0);
  console.log(`${folder}: ${ids.length} objects, ${count('obj-')} object sheets, ${count('age-')} age sheets in ${out}${log.without.length ? `; no pictures for ${log.without.join(', ')}` : ''}`);
}

writeFileSync(join(out, `${what}-${run}.json`), JSON.stringify(log, null, 2));
console.log(`check: ${join(out, `${what}-${run}.json`)}`);
await browser.close();
vite?.kill();
