#!/usr/bin/env bun
/**
 * The rendering benchmark (`web/src/debug/bench.ts`) over a matrix: starts Vite, opens
 * `?bench=<scene>` in Chromium on the real GPU (WebGPU, and WebGL2 with `&renderer=webgl`)
 * at 1920×1080, device pixel ratio 1, once per preset × backend × scenario × CPU slow-down ×
 * diagnostic, and writes every result as JSON and a markdown table (medians when `--runs` > 1).
 *
 *   bun tools/bench/render.mjs [options]
 *     --port 5221              Vite's port (it starts its own; `--url http://localhost:5221` uses a running one)
 *     --preview                the production build (`vite preview` of web/dist: compressed textures and
 *                              models, minified code; what players get) instead of the dev server; `--build` builds it first
 *     --scene village          web/public/bench/<scene>.json(.gz)
 *     --presets low,medium,high
 *     --backends webgpu,webgl
 *     --scenarios orbit-day,orbit-night,dusk,build,wide   (also roam: a loop over the town at 12 m/s, the
 *                              street streaming and the focus lot changing as it goes; mirror-day, mirror-night, mirror-still:
 *                              the home lot's mirror in view from inside its room, the walls up, the
 *                              camera swinging before it or held)
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
 *     --poses orbit-day-0,…    which (`POSES`; `play`: the play set, the pictures a change to the look is judged by)
 *     --pages [gallery,…]      with --shots: the dev pages instead (`PAGES`: the gallery, clips, residents and
 *                              poses benches, each held still), one PNG per page × backend × diag
 *   --compare <dirA> <dirB>    pixel differences between two shot folders (and `--out` for the difference images)
 *   --headless / --headed      without or with a window (see below)
 *   --jobs <file.json>         several runs in one go: a JSON list of command lines (each an array of the
 *                              options above), sharing one Vite server per mode and one browser per kind
 *   --fresh                    a new browser context per page, nothing cached (see below)
 *
 * Needs `bun run wasm` first, and Playwright: `PLAYWRIGHT=<path to playwright/index.mjs>` if it isn't
 * installed, `CHROME=<executable>` for a browser of your own.
 *
 * Measurements run in windows (far off to the right, `--left`): their timing is the real
 * display's. Shots, which don't time anything, run headless: no window, nothing takes the focus
 * from the app in front, and still the real GPU (Chromium's full build in its new headless mode;
 * not `chrome-headless-shell`, which draws in software, so a `CHROME` must be Chrome or Chrome for
 * Testing). `--headless` and `--headed` choose for either. A window's shots come out about 2 of
 * 255 brighter (the screen's colour profile), so compare shots taken the same way.
 *
 * The browser is kept for the whole invocation, with a profile under `bench-results/.browser/`: its
 * HTTP cache and compiled GPU shaders carry over from page to page and to later invocations (the
 * site's own storage is cleared before every page, so each one starts as a new player's would).
 * That shortens the loads and the shader compiles before a measurement's warm-up, not what is
 * measured; `--fresh` runs without it.
 */

import { spawn } from 'node:child_process';
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
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
  // The camera's target along a loop over the town at 12 m/s (`--scenarios roam`; `--scene bigworld`: the village tiled 5 × 5 as a big world).
  roam: 'cam=roam&hour=13',
  // The home lot's mirror on screen all the time, the walls up (not in the default list: `--scenarios mirror-day,…`).
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
  // The home lot's mirror in the middle of the view, the walls up (not in the default list: `--poses mirror-day-0,…`):
  // the swing's middle (at 0 s) and its far end (at 15 s).
  'mirror-day-0': 'cam=mirror&hour=13&at=0',
  'mirror-day-15': 'cam=mirror&hour=13&at=15',
  'mirror-night-0': 'cam=mirror&hour=21&at=0',
  // The play set (`--poses play`): the home lot as a game opens on it (24 m, beta 0.86) with the sun behind the
  // camera and ahead of it, at dusk and by night; the same with the walls up (the house whole, roof on); the lot
  // from across its street; and the menus' town from the main menu's angle.
  'play-day': 'cam=orbit&hour=13&at=0&beta=0.86',
  'play-day-b': 'cam=orbit&hour=13&at=20&beta=0.86',
  'play-dusk': 'cam=orbit&hour=19&at=0&beta=0.86',
  'play-night': 'cam=orbit&hour=21&at=0&beta=0.86',
  'up-day': 'cam=orbit&hour=13&at=0&beta=0.86&walls=up',
  'up-dusk': 'cam=orbit&hour=19&at=0&beta=0.86&walls=up',
  'up-night': 'cam=orbit&hour=21&at=0&beta=0.86&walls=up',
  'street-day': 'cam=street&hour=13&walls=up',
  'street-night': 'cam=street&hour=21&walls=up',
  'town-day': 'cam=town&hour=16.5',
};
/** `--poses play`: the pictures a change to the look is judged by. */
const PLAY = ['play-day', 'play-day-b', 'play-dusk', 'play-night', 'up-day', 'up-dusk', 'up-night', 'wide-day-0', 'wide-night-45', 'street-day', 'street-night', 'town-day'];
/** Poses outside the default list. */
const NAMED_ONLY = /^(mirror|play|up|street|town)-/;

/**
 * Dev pages for `--shots --pages`: each held still (no wind, clips at a set time, residents' time
 * stepped by hand) so the same build draws the same picture. `webgl`: whether the page can draw
 * with WebGL2 (`renderer=webgl`); `prepare`: run in the page once it is ready (residents posed).
 */
const PAGES = {
  'gallery-objects': { path: '/gallery.html', query: 'keys=model.sofa,model.armchair,model.bedDouble,model.fridge,model.bathtub,model.bookcaseTall,model.wardrobe,model.toilet,model.mirror,model.tableLamp&scale=adult&still', webgl: true },
  'gallery-nature': { path: '/gallery.html', query: 'still', webgl: true },
  // A few things close up, without labels: the furniture's finishes and the trees' foliage (the sheets to judge a look by).
  'gallery-furniture': { path: '/gallery.html', query: 'keys=model.sofa,model.armchair,model.fridge,model.bedDouble,model.bathtub,model.bookcaseTall&cols=3&labels=0&still', webgl: true },
  'gallery-trees': { path: '/gallery.html', query: 'keys=model.tree,model.tree.maple,model.tree.birch,model.pine,model.bush,model.hedge&cols=3&labels=0&still', webgl: true },
  // The broadleaved trees, whose crowns are leaf cards (`--extra 'camera=game&at=model.tree.birch'`: one from the game's camera; `--extra far`: their far versions).
  'gallery-broadleaves': { path: '/gallery.html', query: 'keys=model.tree,model.tree.maple,model.tree.birch,model.tree.blossom,model.tree.fruit,model.tree.japaneseMaple,model.tree.magnolia,model.tree.willow,model.garden.lemonTree&cols=3&labels=0&still', webgl: true },
  clips: { path: '/clips.html', query: 'keys=model.fridge,model.sofa,model.wardrobe,model.shower&t=0.6', webgl: true },
  characters: { path: '/characters.html', query: 'manual', webgl: true },
  // The residents' sheets (`characters.ts`): from the side, the heads close up, every skin, hairstyle, garment and
  // build, and the stages from the game's camera as a game opens (24 m) and zoomed all the way in (6 m).
  'characters-side': { path: '/characters.html', query: 'manual&view=side&labels=0', webgl: true },
  'characters-heads': { path: '/characters.html', query: 'manual&closeup=1&labels=0', webgl: true },
  'characters-close': { path: '/characters.html', query: 'manual&builds=all&closeup=1&labels=0&radius=1.3', webgl: true },
  'characters-hands': { path: '/characters.html', query: 'manual&builds=all&family=male&closeup=1&gap=0.9&radius=0.75&y=0.84&labels=0', webgl: true },
  'characters-feet': { path: '/characters.html', query: 'manual&builds=all&family=female&closeup=1&gap=0.9&radius=0.8&y=0.1&beta=1.15&alpha=1.0&labels=0', webgl: true },
  'characters-skins': { path: '/characters.html', query: 'manual&skins=all&closeup=1&labels=0', webgl: true },
  'characters-hair': { path: '/characters.html', query: 'manual&hair=all&labels=0', webgl: true },
  'characters-tops': { path: '/characters.html', query: 'manual&tops=all&labels=0', webgl: true },
  'characters-builds': { path: '/characters.html', query: 'manual&builds=all&labels=0', webgl: true },
  'characters-game': { path: '/characters.html', query: 'manual&camera=game&labels=0', webgl: true },
  'characters-near': { path: '/characters.html', query: 'manual&camera=near&labels=0', webgl: true },
  // One resident as the household creator's face view frames them (half a metre of face down the picture), bald, to
  // judge the head's outline: `--extra hd=1` draws the close-up body the creator's stage draws, without it the body.
  'characters-creator': { path: '/characters.html', query: 'manual&family=male&stage=youngAdult&closeup=1&labels=0&fov=0.46&radius=1.07&skin=%23F3D6C2&hairStyle=none&t=0', webgl: true },
  'characters-creator-female': { path: '/characters.html', query: 'manual&family=female&stage=youngAdult&closeup=1&labels=0&fov=0.46&radius=1.07&skin=%23F3D6C2&hairStyle=none&t=0', webgl: true },
  'characters-faces': { path: '/characters.html', query: 'manual&face=moods&family=female&cols=6&closeup=1&labels=0', webgl: true },
  // The faces of their own (the identity channels): the shared face, then each channel at -1 and at +1, four channels
  // a line, and a line of random faces (`--extra only=jawWidth,chinLength&per=2&radius=1.6`: a few of them, larger).
  'characters-identity': { path: '/characters.html', query: 'manual&faces=all&family=male&closeup=1&t=0', webgl: true },
  'characters-identity-female': { path: '/characters.html', query: 'manual&faces=all&family=female&closeup=1&t=0', webgl: true },
  // Random faces alone, eight men and eight women (`--extra family=male`: one family, larger).
  'characters-identity-random': { path: '/characters.html', query: 'manual&faces=random&closeup=1&t=0&cols=4', webgl: true },
  // The sliders' corners: every channel at -1, at +1, and at one end or the other (the furthest a face goes).
  'characters-identity-ends': { path: '/characters.html', query: 'manual&faces=ends&closeup=1&t=0&cols=5', webgl: true },
  poses: { path: '/poses.html', query: 'manual&labels=0', webgl: true, prepare: () => window.__poses.advance(2) },
  previews: { path: '/previews.html', query: '', webgl: true },
  // The household creator's stage itself, at its face view (`--extra hd=0`: with the body, where it draws the close-up body).
  'previews-stage': { path: '/previews.html', query: 'stage=male&zoom=1&bald', webgl: true },
  'previews-stage-female': { path: '/previews.html', query: 'stage=female&zoom=1&bald', webgl: true },
  'poses-close': { path: '/poses.html', query: 'manual&only=sofa,sleep,cook,bath,piano,hug&cols=3&labels=0', webgl: true, prepare: () => window.__poses.advance(2) },
};

/** One run's options, from its command line or from one job of `--jobs`. */
function optionsOf(args) {
  const flag = (name, fallback) => {
    const i = args.indexOf(`--${name}`);
    return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
  };
  const has = (name) => args.includes(`--${name}`);
  const list = (name, fallback) => flag(name, fallback).split(',').map((s) => s.trim()).filter(Boolean);
  const shots = has('shots');
  const stamp = new Date().toISOString().replace(/\.\d+Z$/, '').replace(/[:T]/g, '-');
  return {
    args,
    flag,
    has,
    list,
    port: Number(flag('port', '5221')),
    url: flag('url', null),
    preview: has('preview'),
    build: has('build'),
    fresh: has('fresh'),
    scene: flag('scene', 'village'),
    presets: list('presets', shots ? 'medium' : 'low,medium,high'),
    backends: list('backends', shots ? 'webgpu' : 'webgpu,webgl'),
    scenarios: list('scenarios', 'orbit-day,orbit-night,dusk,build,wide'),
    throttles: list('throttle', '1').map(Number),
    diags: list('diags', 'none'),
    res: flag('res', '1'),
    secs: Number(flag('secs', '20')),
    warmup: Number(flag('warmup', shots ? '6' : '5')),
    runs: Number(flag('runs', '1')),
    extra: flag('extra', ''),
    throughput: has('throughput'),
    hud: flag('hud', '0'),
    headed: has('headed') || (!shots && !has('headless')),
    left: Number(flag('left', '2600')),
    out: flag('out', shots ? flag('shots') : join(ROOT, 'bench-results', stamp)),
  };
}

async function playwright() {
  if (process.env.PLAYWRIGHT) return import(process.env.PLAYWRIGHT);
  try {
    return await import(createRequire(WEB + 'package.json').resolve('playwright'));
  } catch {
    throw new Error('Playwright not found: install it, or set PLAYWRIGHT=<path to playwright/index.mjs>');
  }
}

/**
 * What all the jobs of one invocation share: a Vite server per mode (dev, preview), started on first
 * use, and a browser per kind (headless, a window, a window without vsync), each launched once and
 * kept until the end. A browser keeps its profile in `bench-results/.browser/<kind>-<port>/`, so its
 * HTTP cache and its compiled GPU shaders carry over from page to page and from one invocation to the
 * next; each page still starts with the site's storage (saves, settings) cleared. `--fresh`: a new
 * context per page, nothing kept (as before).
 */
const servers = new Map();
const browsers = new Map();
let built = false;
let nextPort = null;
/** The port the browser profiles are named by: the first job's. */
let profilePort = null;
const quit = [];
process.on('exit', () => {
  for (const kill of quit) kill();
});

/** The base URL of the server a job's pages come from (building `web/dist` first for `--build`, once). */
async function serverFor(o) {
  profilePort ??= o.port;
  if (o.url) return o.url;
  if (o.build && !built) {
    const build = spawn(`${WEB}node_modules/.bin/vite`, ['build'], { cwd: WEB, stdio: ['ignore', 'ignore', 'inherit'] });
    await new Promise((resolve, reject) => build.on('exit', (code) => (code ? reject(new Error(`vite build failed (${code})`)) : resolve())));
    built = true;
  }
  const mode = o.preview ? 'preview' : 'dev';
  if (!servers.has(mode)) {
    nextPort ??= o.port;
    const port = nextPort++;
    const vite = spawn(`${WEB}node_modules/.bin/vite`, [...(o.preview ? ['preview'] : []), '--port', String(port), '--strictPort'], { cwd: WEB, stdio: ['ignore', 'pipe', 'inherit'] });
    quit.push(() => vite.kill());
    servers.set(
      mode,
      new Promise((resolve, reject) => {
        vite.stdout.on('data', (d) => String(d).includes('Local') && resolve(`http://localhost:${port}`));
        vite.on('exit', (code) => reject(new Error(`vite exited (${code})`)));
      }),
    );
  }
  return servers.get(mode);
}

const VIEWPORT = { viewport: { width: 1920, height: 1080 }, deviceScaleFactor: 1 };
/** What a kept browser clears before each page: everything the site stores, not the HTTP or shader caches. */
const SITE_STORAGE = 'cookies,file_systems,indexeddb,local_storage,service_workers,cache_storage';

/** The browser for a job's kind: `{ context }` (kept, with its profile) or `{ browser }` (`--fresh`). */
async function browserFor(o) {
  const kind = `${o.headed ? 'window' : 'headless'}${o.throughput ? '-fast' : ''}${o.fresh ? '-fresh' : ''}`;
  if (!browsers.has(kind)) {
    const { chromium } = await playwright();
    const options = {
      headless: !o.headed,
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
        ...(o.throughput ? ['--disable-gpu-vsync', '--disable-frame-rate-limit'] : []),
      ],
    };
    browsers.set(
      kind,
      (async () => {
        if (o.fresh) return { browser: await chromium.launch(options) };
        // (One profile per kind and port, the first job's: two invocations at once, on their own ports, don't share one.)
        const context = await chromium.launchPersistentContext(join(ROOT, 'bench-results', '.browser', `${kind}-${profilePort ?? o.port}`), { ...options, ...VIEWPORT });
        await context.addInitScript(quiet);
        // Its first window stays open (the browser with it), out of the way like the others; each page gets a window of its own (`newWindow`).
        const first = context.pages()[0] ?? (await context.newPage());
        if (o.headed) await place(context, first, o.left);
        return { context };
      })(),
    );
  }
  return browsers.get(kind);
}

/** A new page in a window of its own (a kept browser's `newPage` opens a tab in its first window instead). */
async function newWindow(context) {
  const cdp = await context.newCDPSession(context.pages()[0]);
  const page = context.waitForEvent('page');
  await cdp.send('Target.createTarget', { url: 'about:blank', newWindow: true });
  await cdp.detach();
  return page;
}

/** Moves a page's window far off to the right (`--left`). */
async function place(context, page, left) {
  try {
    const cdp = await context.newCDPSession(page);
    const { windowId } = await cdp.send('Browser.getWindowForTarget');
    await cdp.send('Browser.setWindowBounds', { windowId, bounds: { left, top: 0 } });
    await cdp.detach();
  } catch {
    // (Some window managers refuse; it still runs.)
  }
}

/** Opens one benchmark page and waits for `key` on window (its result, or readiness for a screenshot); `done()` closes it. */
async function open(o, query, throttle, key, timeoutMs, path = '/') {
  const base = await serverFor(o);
  const b = await browserFor(o);
  let context = b.context;
  if (!context) {
    context = await b.browser.newContext(VIEWPORT);
    await context.addInitScript(quiet);
  }
  const page = b.context ? await newWindow(context) : await context.newPage();
  const done = () => (b.context ? page.close() : context.close());
  try {
    const cdp = await context.newCDPSession(page);
    if (b.context) await cdp.send('Storage.clearDataForOrigin', { origin: new URL(base).origin, storageTypes: SITE_STORAGE });
    if (o.headed) await place(context, page, o.left);
    if (throttle > 1) await cdp.send('Emulation.setCPUThrottlingRate', { rate: throttle });
    page.on('pageerror', (e) => console.error(`  page error: ${e.message}`));
    page.on('console', (m) => m.type() === 'error' && console.error(`  page: ${m.text().slice(0, 300)}`));
    await page.goto(`${base}${path}?${query}`);
    await page.waitForFunction((k) => !!window[k], key, { timeout: timeoutMs, polling: 250 });
  } catch (err) {
    await done();
    throw err;
  }
  return { page, done };
}

const queryOf = (o, preset, backend, more) =>
  [`bench=${o.scene}`, `quality=${preset}`, backend === 'webgl' ? 'renderer=webgl' : '', `res=${o.res}`, more, o.extra].filter(Boolean).join('&');

/** One job: a comparison, shots, page shots or a measured matrix. */
async function run(o) {
  if (o.has('compare')) {
    const i = o.args.indexOf('--compare');
    return compare(o.args[i + 1], o.args[i + 2], o.flag('out', null));
  }
  mkdirSync(o.out, { recursive: true });
  if (o.has('shots') && o.has('pages')) await pageShots(o);
  else if (o.has('shots')) await shots(o);
  else await matrix(o);
}

const argv = process.argv.slice(2);
const top = optionsOf(argv);
// `--jobs <file.json>`: a list of command lines (arrays of arguments) run one after another in this one session.
const jobs = top.flag('jobs', null) ? JSON.parse(readFileSync(top.flag('jobs'), 'utf8')) : [argv];
let failed = 0;
try {
  for (const [i, job] of jobs.entries()) {
    const started = performance.now();
    if (jobs.length > 1) console.log(`\n=== job ${i + 1}/${jobs.length}: ${job.join(' ')}`);
    try {
      await run(optionsOf(job));
    } catch (err) {
      failed++;
      console.log(`job failed: ${err}`);
    }
    if (jobs.length > 1) console.log(`=== job ${i + 1} took ${((performance.now() - started) / 1000).toFixed(0)} s`);
  }
} finally {
  for (const b of browsers.values()) {
    const { context, browser } = await b.catch(() => ({}));
    await (context ?? browser)?.close();
  }
  for (const kill of quit) kill();
}
process.exit(failed ? 1 : 0);

async function matrix(o) {
  const { runs, backends, presets, throttles, scenarios, diags, secs, warmup, hud, throughput, out } = o;
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
              const query = queryOf(o, preset, backend, [SCENARIOS[scenario] ?? scenario, d && `diag=${d}`, `secs=${secs}`, `warmup=${warmup}`, `hud=${hud}`, `label=${label}`].filter(Boolean).join('&'));
              n++;
              process.stdout.write(`[${n}/${total}] ${label} … `);
              let result;
              try {
                const { page, done } = await open(o, query, throttle, '__benchResult', (warmup + secs) * 1000 + 240_000);
                result = await page.evaluate(() => window.__benchResult);
                await done();
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
  const md = table(o, results);
  writeFileSync(join(out, 'results.md'), md);
  console.log(`\n${md}\nwritten to ${out}`);
}

/** Medians over runs, one row per cell. */
function table(o, results) {
  const { scene, res, throughput, secs, warmup, runs } = o;
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

async function shots(o) {
  const { backends, presets, diags, warmup, out } = o;
  const poses = o.list('poses', Object.keys(POSES).filter((p) => !NAMED_ONLY.test(p)).join(',')).flatMap((p) => (p === 'play' ? PLAY : [p]));
  for (const backend of backends) {
    for (const preset of presets) {
      for (const pose of poses) {
        for (const diag of diags) {
          const d = ['freeze', ...(diag === 'none' ? [] : diag.split('+'))].join(',');
          const name = `${preset}-${backend}-${pose}${diag === 'none' ? '' : `-${diag}`}.png`;
          process.stdout.write(`${name} … `);
          try {
            const { page, done } = await open(o, queryOf(o, preset, backend, `${POSES[pose]}&diag=${d}&hud=0&secs=0&warmup=${warmup}`), 1, '__benchReady', warmup * 1000 + 240_000);
            // Two more frames, then the picture.
            await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
            await page.screenshot({ path: join(out, name) });
            await done();
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

/** The dev pages (`PAGES`), drawn still: one picture per page × backend × diag (the page's `__ready`). */
async function pageShots(o) {
  const { backends, diags, extra, out } = o;
  const names = o.list('pages', Object.keys(PAGES).join(','));
  for (const backend of backends) {
    for (const name of names) {
      const p = PAGES[name];
      if (!p) {
        console.log(`no page ${name} (${Object.keys(PAGES).join(', ')})`);
        continue;
      }
      if (backend === 'webgl' && !p.webgl) continue;
      for (const diag of diags) {
        const file = `${name}-${backend}${diag === 'none' ? '' : `-${diag}`}.png`;
        process.stdout.write(`${file} … `);
        const query = [p.query, backend === 'webgl' ? 'renderer=webgl' : '', diag === 'none' ? '' : `diag=${diag.replaceAll('+', ',')}`, extra].filter(Boolean).join('&');
        try {
          const { page, done } = await open(o, query, 1, '__ready', 240_000, p.path);
          if (p.prepare) await page.evaluate(p.prepare);
          // A moment for late loads (hair, props), then two more frames, then the picture.
          await page.waitForTimeout(1500);
          await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
          await page.screenshot({ path: join(out, file) });
          await done();
          console.log('ok');
        } catch (err) {
          console.log(`failed: ${err}`);
        }
      }
    }
  }
  console.log(`page shots in ${out}`);
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
