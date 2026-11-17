#!/usr/bin/env bun
/**
 * Step 2's painted uses (`menuScene.setUses`, `render/pc/useOverlay.ts`) and the free camera on the menus' town,
 * without going through the screens. A generated town is shown and painted as its own lots and buildings are (each
 * lot's and backdrop building's cells by what it is: houses homes, terraces and flats rows, shops and work shops,
 * parks green, the hall, church and school public; the cells under its roads left out), with the zoned cells more
 * than 36 m from a road hatched; then shot from the whole world's view, the main town's and close up. Then the time
 * a brush stamp's change takes (`setUses` with a few cells changed) against the whole layer drawn again.
 *
 * Then the camera as the player moves it: tilted low over the main town, come in to 10 m (`OVERVIEW_CLOSE`), turned
 * round, and panned along the view, each shot.
 *
 * The page's UI is hidden in the pictures; a grey band on the left stands for the side panel the shots keep clear of.
 *
 *   bun tools/bench/zoneShots.mjs [options]
 *     --renderers webgpu,webgl     the page's renderer
 *     --size 1920x1080             the window (CSS px)
 *     --world country --options town=town,villages=some    the place and options (`worldFlags.mjs`)
 *     --seed 7                     the world's seed
 *     --port 5431 | --url <u>      a dev server on the port (started if none answers), or one already running
 *     --out <dir>                  default /tmp/zone-shots/
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

const [width, height] = flag('size', '1920x1080').split('x').map(Number);
const renderers = flag('renderers', 'webgpu,webgl').split(',');
const seed = Number(flag('seed', '7'));
const made = worldFlags(flag, 'country');
const port = Number(flag('port', '5431'));
const out = flag('out', '/tmp/zone-shots');
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

/** In the page: the town made and shown, its uses painted from what stands on it, the UI hidden, a band where the side panel would be. */
async function setUp({ kind, options, seed, side }) {
  const [{ menuScene }, { generateWorld, loadTemplates }, { services }, uses, { OVERVIEW_CLOSE }] = await Promise.all([
    import('/src/game/menuScene.ts'),
    import('/src/game/town.ts'),
    import('/src/ui/services.ts'),
    import('/src/game/uses.ts'),
    import('/src/render/shared/overviewCamera.ts'),
  ]);
  const t = await loadTemplates();
  const town = generateWorld(services.content, t, kind, seed, undefined, undefined, options);
  await menuScene.show(town, null);
  const world = menuScene.preview;
  const { USE_CELL, USE_CODE, useGrid, packUses } = uses;
  const { cols, rows } = useGrid(town.width, town.depth);
  const codes = new Uint8Array(cols * rows);
  const paint = (x0, z0, x1, z1, code) => {
    for (let j = Math.max(0, Math.floor(z0 / USE_CELL)); j < Math.min(rows, Math.ceil(z1 / USE_CELL)); j++)
      for (let i = Math.max(0, Math.floor(x0 / USE_CELL)); i < Math.min(cols, Math.ceil(x1 / USE_CELL)); i++) codes[j * cols + i] = code;
  };
  const BACKDROP = { flats: 'rows', terrace: 'rows', house: 'homes', shop: 'shops', cafe: 'shops', restaurant: 'shops', office: 'shops', workshop: 'shops', school: 'public', church: 'public', hall: 'public' };
  for (const b of world.meta?.backdrop ?? []) {
    const use = BACKDROP[b.use];
    if (!use) continue;
    const xs = b.poly.map((p) => p[0]);
    const zs = b.poly.map((p) => p[1]);
    paint(Math.min(...xs) - 2, Math.min(...zs) - 2, Math.max(...xs) + 2, Math.max(...zs) + 2, USE_CODE[use]);
  }
  for (const s of town.slots) {
    const p = world.plots[s.index];
    if (!p) continue;
    const use = s.kind === 'house' ? ((p.storeys ?? 1) > 1 ? 'rows' : 'homes') : s.kind === 'shop' ? 'shops' : 'green';
    paint(p.x, p.z, p.x + p.w, p.z + p.d, USE_CODE[use]);
  }
  // The cells under the roads left out, and those far from any road found (for the hatching).
  const near = new Uint8Array(cols * rows);
  const stamp = (x, z, r, fn) => {
    for (let j = Math.max(0, Math.floor((z - r) / USE_CELL)); j <= Math.min(rows - 1, Math.floor((z + r) / USE_CELL)); j++)
      for (let i = Math.max(0, Math.floor((x - r) / USE_CELL)); i <= Math.min(cols - 1, Math.floor((x + r) / USE_CELL)); i++) {
        if (Math.hypot((i + 0.5) * USE_CELL - x, (j + 0.5) * USE_CELL - z) <= r) fn(j * cols + i);
      }
  };
  for (const r of town.roads) {
    for (let k = 0; k + 1 < r.points.length; k++) {
      const [[ax, az], [bx, bz]] = [r.points[k], r.points[k + 1]];
      const n = Math.max(1, Math.ceil(Math.hypot(bx - ax, bz - az) / 2));
      for (let s = 0; s <= n; s++) {
        const [x, z] = [ax + ((bx - ax) * s) / n, az + ((bz - az) * s) / n];
        stamp(x, z, 36, (c) => (near[c] = 1));
        stamp(x, z, r.width / 2, (c) => (codes[c] = 0));
      }
    }
  }
  const hatched = new Uint8Array(cols * rows);
  // (Some far fields painted too, to show the hatching: a block of homes round the land's quarter points.)
  for (const [fx, fz] of [
    [0.2, 0.2],
    [0.8, 0.75],
  ]) {
    const [x, z] = [town.width * fx, town.depth * fz];
    paint(x - 30, z - 20, x + 30, z + 20, USE_CODE.green);
  }
  let far = 0;
  for (let c = 0; c < codes.length; c++) if (codes[c] && !near[c]) hatched[c] = (far++, 1);
  const layer = packUses(codes, cols, rows);
  menuScene.setUses(layer, hatched);
  const style = document.createElement('style');
  style.textContent = 'body * { visibility: hidden !important; } canvas, .ng-side { visibility: visible !important; }';
  document.head.append(style);
  const band = document.createElement('div');
  band.className = 'ng-side';
  Object.assign(band.style, { position: 'fixed', left: '0', top: '0', bottom: '0', width: `${side}px`, background: 'rgba(40, 44, 52, 0.55)', zIndex: '99999', pointerEvents: 'none' });
  document.body.append(band);
  window.__zs = { menuScene, town, layer, codes, hatched, cols, rows, packUses, OVERVIEW_CLOSE, frame: { left: side, top: 0, right: innerWidth, bottom: innerHeight } };
  let painted = 0;
  for (const c of codes) if (c) painted++;
  return { width: town.width, depth: town.depth, areas: town.open?.areas.length ?? 0, painted, far, cols, rows };
}

/** In the page: frames a shot, cut, with the player's moves let go. */
async function frameShot(shot) {
  const zs = window.__zs;
  zs.menuScene.shoot({ ...shot, frame: zs.frame }, 0, true);
  zs.menuScene.reframe(0);
  await zs.menuScene.settled();
}

const { chromium } = await playwright();
const browser = await chromium.launch({
  headless: !has('headed'),
  ...(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chromium' }),
  args: ['--enable-unsafe-webgpu', '--disable-backgrounding-occluded-windows', '--disable-renderer-backgrounding', '--disable-background-timer-throttling'],
});
let faults = 0;
try {
  for (const renderer of renderers) {
    const context = await browser.newContext({ viewport: { width, height }, deviceScaleFactor: 1 });
    await context.addInitScript(quiet);
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', (e) => errors.push(e.message));
    page.on('console', (m) => m.type() === 'error' && errors.push(m.text()));
    await page.goto(`${base}/?${renderer === 'webgl' ? 'renderer=webgl' : ''}`);
    await page.waitForFunction(() => !!document.querySelector('.menu') && !document.querySelector('.loading'), null, { timeout: 180_000, polling: 200 });
    const facts = await page.evaluate(setUp, { kind: made.world, options: made.options, seed, side: Math.round(width * 0.24) });
    console.log(`zoneShots: ${renderer}: ${made.world} seed ${seed}: ${facts.width}x${facts.depth} m (${facts.cols}x${facts.rows} cells), ${facts.areas} areas, ${facts.painted} cells painted, ${facts.far} hatched`);
    const shoot = async (name, wait = 1500) => {
      await page.waitForTimeout(wait);
      const file = join(out, `${renderer}-${name}.png`);
      await page.screenshot({ path: file });
      console.log(`zoneShots: ${file}`);
    };

    await page.evaluate(frameShot, { kind: 'world' });
    await shoot('uses-world', 3000);
    await page.evaluate(frameShot, { kind: 'area', area: 0 });
    await shoot('uses-area0');
    // Closer, on the town's middle: the runs, their edges, the hatching where there is some.
    await page.evaluate(() => window.__zs.menuScene.nudge(0, 3));
    await shoot('uses-close');

    // A far field (no road within 36 m), hatched, close up: from the whole world's shot, the wheel's zoom towards it.
    await page.evaluate(frameShot, { kind: 'world' });
    // (Projected through the camera as drawn: a frame or two after the cut.)
    await page.waitForTimeout(500);
    await page.evaluate(() => {
      const zs = window.__zs;
      const out = { x: 0, y: 0 };
      if (zs.menuScene.project(zs.town.width * 0.2, 0, zs.town.depth * 0.2, out)) zs.menuScene.zoomAt(out.x, out.y, 14);
    });
    await shoot('uses-hatched', 3000);

    // A brush's stamp (a disc of shops 16 m across) against the whole layer drawn again.
    const ms = await page.evaluate(() => {
      const zs = window.__zs;
      const a = zs.town.open?.areas[0] ?? { x: 0, z: 0, w: zs.town.width, d: zs.town.depth };
      const codes = zs.codes.slice();
      const [ci, cj] = [Math.floor((a.x + a.w / 2) / 4), Math.floor((a.z + a.d / 2) / 4)];
      const stamps = [];
      for (let k = 0; k < 40; k++) {
        for (let dj = -2; dj <= 2; dj++) for (let di = -2; di <= 2; di++) if (di * di + dj * dj <= 4) codes[(cj + dj) * zs.cols + ci + di + k] = 3;
        stamps.push(zs.packUses(codes, zs.cols, zs.rows));
      }
      const t0 = performance.now();
      for (const l of stamps) zs.menuScene.setUses(l, zs.hatched);
      const stamp = (performance.now() - t0) / stamps.length;
      const t1 = performance.now();
      zs.menuScene.setUses(null);
      zs.menuScene.setUses(stamps.at(-1), zs.hatched);
      return { stamp, whole: performance.now() - t1 };
    });
    console.log(`zoneShots: ${renderer}: a stamp's change drawn in ${ms.stamp.toFixed(2)} ms; the whole layer in ${ms.whole.toFixed(1)} ms`);
    await shoot('uses-stamped');

    // The free camera: tilted low over the main town, come in to 10 m, turned, panned along the view.
    await page.evaluate(frameShot, { kind: 'area', area: 0 });
    await page.evaluate(() => {
      const zs = window.__zs;
      zs.menuScene.setNearest(zs.OVERVIEW_CLOSE);
      zs.menuScene.nudge(0, 2, 0.6);
    });
    await shoot('camera-tilted-low', 2500);
    await page.evaluate(() => {
      const zs = window.__zs;
      // (Towards the land under the stage's middle, as the wheel does.)
      const [x, y] = [(zs.frame.left + zs.frame.right) / 2, innerHeight / 2];
      for (let k = 0; k < 60; k++) zs.menuScene.zoomAt(x, y, 1.2);
    });
    await shoot('camera-close-10m', 3000);
    const close = await page.evaluate(() => window.__zs.menuScene.distance);
    console.log(`zoneShots: ${renderer}: in as near as ${close.toFixed(2)} m`);
    await page.evaluate(() => window.__zs.menuScene.nudge(0, 1 / 6, -0.3));
    await shoot('camera-near', 2500);
    await page.evaluate(() => window.__zs.menuScene.nudge(2.2, 1));
    await shoot('camera-turned', 2500);
    await page.evaluate(() => window.__zs.menuScene.panView(60, 0));
    await shoot('camera-panned-right', 2500);
    await page.evaluate(() => window.__zs.menuScene.setUses(null));
    await shoot('camera-turned-no-uses');
    await page.evaluate(() => window.__zs.menuScene.reframe());
    await shoot('camera-reframed', 2500);

    for (const e of errors) {
      faults++;
      console.warn(`zoneShots: ${renderer}: page error: ${e}`);
    }
    await context.close();
  }
} finally {
  await browser.close();
  server?.kill();
}
console.log(faults ? `zoneShots: ${faults} fault(s); pictures in ${out}` : `zoneShots: clean; pictures in ${out}`);
