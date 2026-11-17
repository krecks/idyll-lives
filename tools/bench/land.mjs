#!/usr/bin/env bun
/**
 * The land brushes as a player uses them, by script (`window.__land`, `?debug`): step 1's land made,
 * "Shape the land" chosen, then a mound flattened on top (a plateau), a pond on its top, a river from the hill down to the sea (or
 * the nearest water), woods painted and half cleared, and a peak, each a stroke a point a frame; the
 * frames' times during each stroke, and pictures before and after. Then "Lay out the town" (step 2's
 * town laid on the land as shaped), "Choose a household", a household played (step 3, with the game's
 * mode chosen there) and step 4's Move in: the step-1 pond is water to the simulation (where the town left it), someone is sent across it, and the
 * camera looks at it; then Build mode's Land tab: a mound by the home (paid in Living, free in
 * Creative; undo pays back, redo pays again) and a pond, a raise across the road by the home and a lower under the
 * home (built land); saved, continued from the menu, and the
 * land and the water are still there. In step 1 too, the ends of the size range: 120 m raise, smooth
 * and lake strokes timed, a 1 m point stroke (one sample changed at each point it crosses), the
 * palette at 1 m and at 120 m, and the 1 m cursor close up.
 *
 *   bun tools/bench/land.mjs [options]
 *     --renderers webgpu,webgl     the page's renderer (`?renderer=webgl`)
 *     --size 2380x1546             the window (CSS px)
 *     --seed 7                     the page's Math.random seed (the same world every run)
 *     --world country --options …  step 1's place and options (`?world=`, `?options=`)
 *     --no-play                    step 1 only
 *     --plain                      no shaping in step 1: the game's world has no land of its own until its first stroke
 *     --mode living|creative       the game's mode (step 3's choice)
 *     --headed                     in a window
 *     --port 5186 | --url <u>      a dev server on the port (started if none answers), or one already running
 *     --out <dir>                  default bench-results/land-<time>/
 *
 * Needs Playwright like render.mjs (`PLAYWRIGHT=<path to playwright/index.mjs>`).
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

const [width, height] = flag('size', '2380x1546').split('x').map(Number);
const renderers = flag('renderers', 'webgpu,webgl').split(',');
const seed = Number(flag('seed', '7'));
const mode = flag('mode', 'living');
/** Step 1 left as generated: the game's first stroke is the land's first (its header and every tile sent with it). */
const plain = has('plain');
const made = worldFlags(flag, null);
const worldQuery = [made.world ? `world=${made.world}` : '', made.query ? `options=${made.query}` : ''];
const port = Number(flag('port', '5186'));
const stamp = new Date().toISOString().replace(/\.\d+Z$/, '').replace(/[:T]/g, '-');
const out = flag('out', join(ROOT, 'bench-results', `land-${stamp}`));
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

/** Frame times (ms): median, 95th percentile, worst. */
function spread(xs) {
  if (!xs.length) return { n: 0 };
  const s = [...xs].sort((a, b) => a - b);
  const at = (q) => s[Math.min(s.length - 1, Math.floor(q * s.length))];
  return { n: s.length, median: +at(0.5).toFixed(1), p95: +at(0.95).toFixed(1), max: +s[s.length - 1].toFixed(1) };
}

const { chromium } = await playwright();
const browser = await chromium.launch({
  headless: !has('headed'),
  ...(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chromium' }),
  args: ['--enable-unsafe-webgpu', '--disable-backgrounding-occluded-windows', '--disable-renderer-backgrounding', '--disable-background-timer-throttling'],
});

const report = {};
let faults = 0;
const fault = (line) => {
  faults++;
  console.warn(`land: ${line}`);
};

/**
 * Points the step-1 camera at town point (`x`, `z`) and zooms by `zoom`: the ground under the view's
 * middle panned there (a few times, as the camera eases), with the town camera's own pan and zoom.
 */
async function lookAt(page, x, z, zoom = 1) {
  await page.evaluate(async ([x, z, zoom]) => {
    const view = document.querySelector('.view').getBoundingClientRect();
    if (zoom !== 1) window.__land.zoom(zoom);
    for (let k = 0; k < 4; k++) {
      await new Promise((r) => setTimeout(r, 700));
      const c = window.__land.pick(view.left + view.width / 2, view.top + view.height / 2);
      if (c) window.__land.pan(x - c.x, z - c.z);
    }
    await new Promise((r) => setTimeout(r, 1200));
  }, [x, z, zoom]);
}

/** Waits for the renderer's land work (shading, grass, woods) to be done. */
const settled = (page) => page.evaluate(() => window.__landDebug?.settled?.()).then(() => page.waitForTimeout(600));

/** A circle of `n` points round (x, z), `r` m across, closed. */
const ring = (x, z, r, n) => Array.from({ length: n + 1 }, (_, k) => [x + Math.cos((k / n) * Math.PI * 2) * r, z + Math.sin((k / n) * Math.PI * 2) * r]);
/** Points every `step` m along a line. */
const line = (a, b, step = 2) => {
  const n = Math.max(1, Math.ceil(Math.hypot(b[0] - a[0], b[1] - a[1]) / step));
  return Array.from({ length: n + 1 }, (_, k) => [a[0] + ((b[0] - a[0]) * k) / n, a[1] + ((b[1] - a[1]) * k) / n]);
};

/** In the game, its loading screen gone. */
const inGame = (page) => page.waitForFunction(() => !!document.querySelector('.hud-root') && !document.querySelector('.loading'), null, { timeout: 180_000, polling: 100 });

/** Build mode's Land tab up (G), its land surveyed. */
async function landTab(page) {
  await page.keyboard.press('g');
  await page.waitForFunction(() => {
    try {
      return !!window.__land?.stats();
    } catch {
      return false;
    }
  }, null, { timeout: 30_000, polling: 100 });
  await page.waitForTimeout(2500);
}

const funds = (page) => page.evaluate(() => window.__osw.structure().funds);

/** The game camera on town point `at`, `radius` m off, looking down at it (`__camera`), and the town round it streamed in. */
async function aim(page, at, radius) {
  await page.evaluate(
    ([[x, z], radius]) => {
      window.__controls.lookAt(x, z);
      return new Promise((res) =>
        setTimeout(() => {
          window.__camera({ target: [x, 0, z], radius, beta: 0.95 });
          res();
        }, 2500),
      );
    },
    [at, radius],
  );
  await page.waitForTimeout(3500);
}

/**
 * In play: the pond made in step 1 is water the residents walk round (someone sent across it, the
 * camera on it); then the Land tab: a mound by the home (paid in Living, free in Creative; undo pays
 * back, redo pays again) and a pond; saved, continued from the menu, and the land and water still there.
 */
async function play(page, renderer, pond, shot) {
  const r = {};
  if (pond) await stepOnePond(page, renderer, pond, shot, r);
  return { ...r, ...(await landTabInPlay(page, renderer, pond, shot)) };
}

/** The pond made in step 1 in play: water to the simulation, someone sent across it, the camera on it. */
async function stepOnePond(page, renderer, pond, shot, r) {
  r.stepOnePond = await page.evaluate(async ([x, z]) => {
    window.__controls.setMode('build');
    window.__controls.setBuildTab('land');
    for (let k = 0; k < 100 && !(() => { try { return window.__land.stats(); } catch { return null; } })(); k++) await new Promise((res) => setTimeout(res, 100));
    return window.__land.simWet(x, z);
  }, pond);
  if (!r.stepOnePond) fault(`${renderer} the pond made in step 1 isn't water in the game`);
  // Someone sent across the pond: the camera on them as they go round it.
  await page.evaluate(async ([x, z]) => {
    window.__controls.setMode('live');
    const sim = window.__osw.sims()[0];
    window.__controls.walkTo(sim.id, x + 14, z + 2);
    window.__controls.follow(sim.id);
  }, pond);
  await page.waitForTimeout(9000);
  await shot('12-walking-to-the-pond');
  await page.evaluate(() => window.__controls.follow(null));
  await aim(page, pond, 70);
  await shot('13-pond-in-play');
}

/**
 * Build mode's Land tab in play: a mound by the home (paid in Living, free in Creative; undo pays
 * back, redo pays again) and a pond; saved, continued from the menu, and the land and the water
 * still there, the land away from the strokes too (the whole land went with the first stroke).
 */
async function landTabInPlay(page, renderer, stepOne, shot) {
  const r = {};

  // The Land tab by the home.
  await page.evaluate(() => window.__controls.setMode('live'));
  await landTab(page);
  await shot('14-land-tab');
  const home = await page.evaluate(() => {
    const h = window.__osw.home();
    return h ? [h.x + h.w / 2, h.z + h.d / 2] : null;
  });
  const spot = await page.evaluate(([x, z]) => window.__land.findFree(14, [x, z]), home);
  if (!spot) {
    fault(`${renderer} no free land by the home`);
    return r;
  }
  // The land round about, away from the strokes: as it is after the game comes back.
  const grid = await page.evaluate(([sx, sz]) => {
    const { cols, rows } = window.__land.stats();
    const out = [];
    for (let a = 1; a < 6; a++) for (let b = 1; b < 6; b++) {
      const [x, z] = [(a / 6) * cols * 2, (b / 6) * rows * 2];
      if (Math.hypot(x - sx, z - sz) > 80) out.push([x, z, window.__land.heightAt(x, z), window.__land.wet(x, z)]);
    }
    return out;
  }, spot);
  await aim(page, spot, 75);
  const before = await funds(page);
  const h0 = await page.evaluate(([x, z]) => window.__land.heightAt(x, z), spot);
  const raised = await page.evaluate(([x, z]) => window.__land.stroke('raise', Array.from({ length: 40 }, (_, k) => [x + Math.cos(k / 6) * 3, z + Math.sin(k / 6) * 3]), 10, 0.8), spot);
  await page.waitForTimeout(800);
  const paid = await funds(page);
  const h1 = await page.evaluate(([x, z]) => window.__land.heightAt(x, z), spot);
  await page.evaluate(() => window.__land.undo());
  await page.waitForTimeout(800);
  const refunded = await funds(page);
  await page.evaluate(() => window.__land.redo());
  await page.waitForTimeout(800);
  const again = await funds(page);
  r.money = { mode, before, paid, refunded, again, rose: +(h1 - h0).toFixed(2), refused: raised.refused, frames: spread(raised.frames.slice(1)) };
  console.log(`land: ${renderer} in play: ${JSON.stringify(r.money)}`);
  if (raised.refused) fault(`${renderer} the mound was refused: ${raised.refused}`);
  if (mode === 'creative' ? paid !== before : !(paid < before)) fault(`${renderer} funds ${before} → ${paid} after the mound (${mode})`);
  if (refunded !== before) fault(`${renderer} undo gave back ${refunded - paid}, not ${before - paid}`);
  if (again !== paid) fault(`${renderer} redo: funds ${again}, not ${paid}`);
  // A pond beside it.
  const pondAt = await page.evaluate(([x, z]) => window.__land.findFree(10, [x + 30, z]), spot);
  const pond = pondAt && (await page.evaluate(([x, z]) => window.__land.stroke('lake', [[x, z], [x + 2, z], [x + 4, z]], 7, 0.5), pondAt));
  r.pond = pondAt && { at: pondAt, refused: pond.refused, sim: await page.evaluate(([x, z]) => window.__land.simWet(x, z), pondAt) };
  if (pondAt && !r.pond.sim) fault(`${renderer} the pond in play isn't water to the simulation (${pond.refused ?? 'not refused'})`);
  await aim(page, [spot[0] + 15, spot[1]], 75);
  await shot('15-land-tab-strokes');

  // By hand, in Living: a stroke dragged with the pointer while the funds run low stops short of them, and is paid.
  if (mode === 'living') r.short = await fundsRunOut(page, renderer, spot, shot);

  // Built land: a raise across the street in front of the home and a lower under the home itself (the shots say what
  // the land, the road and the lot do; the earth tools keep off built land in the game for now).
  r.built = await builtStrokes(page, renderer, home, shot);

  // Saved, continued from the menu: the land and the water as they were.
  await page.evaluate(() => window.__osw.save('land-check'));
  await page.waitForTimeout(1000);
  await page.reload();
  await page.waitForFunction(() => !!document.querySelector('.menu') && !document.querySelector('.loading'), null, { timeout: 180_000, polling: 200 });
  await page.waitForTimeout(1500);
  await page.locator('button.continue').click();
  await inGame(page);
  await page.waitForTimeout(3000);
  await page.evaluate(() => window.__controls.setMode('live'));
  await landTab(page);
  const kept = await page.evaluate(
    async ([[x, z], pond, stepOne]) => ({
      height: window.__land.heightAt(x, z),
      pond: pond ? await window.__land.simWet(pond[0], pond[1]) : null,
      pondDrawn: pond ? window.__land.wet(pond[0], pond[1]) : null,
      stepOnePond: stepOne ? await window.__land.simWet(stepOne[0], stepOne[1]) : null,
    }),
    [spot, pondAt, stepOne],
  );
  const away = await page.evaluate((grid) => grid.filter(([x, z, h, wet]) => Math.abs(window.__land.heightAt(x, z) - h) > 0.02 || window.__land.wet(x, z) !== wet).length, grid);
  kept.awayChanged = away;
  if (away) fault(`${renderer} after continuing, ${away} of ${grid.length} points away from the strokes differ`);
  r.reloaded = kept;
  console.log(`land: ${renderer} after save and continue: ${JSON.stringify(kept)} (the mound was ${h1.toFixed(2)} m)`);
  if (Math.abs(kept.height - h1) > 0.05) fault(`${renderer} the mound came back at ${kept.height.toFixed(2)} m, not ${h1.toFixed(2)}`);
  if (pondAt && !(kept.pond && kept.pondDrawn)) fault(`${renderer} the pond didn't come back (sim ${kept.pond}, drawn ${kept.pondDrawn})`);
  await aim(page, [spot[0] + 15, spot[1]], 75);
  await shot('16-continued');
  return r;
}

/**
 * Strokes on built land in play: a 12 m raise across the road nearest the home, then a lower under the home's lot.
 * Each is sent and taken (not refused); the heights at the road and the home before and after are told, and the
 * shots show the road and the lot on the land after (`17-…`, `18-…`).
 */
async function builtStrokes(page, renderer, home, shot) {
  const out = {};
  const road = await page.evaluate(([x, z]) => window.__land.nearestRoad(x, z), home);
  if (!road) return out;
  const [rx, rz] = road.at;
  const [ux, uz] = road.along;
  await page.evaluate(() => window.__addFunds(200000));
  await aim(page, [rx, rz], 60);
  await shot('17a-road-before');
  const h0 = await page.evaluate(([x, z]) => window.__land.heightAt(x, z), [rx, rz]);
  // (Across the road: from one side to the other, back and forth.)
  const across = [];
  for (let k = 0; k < 6; k++) for (const s of k % 2 ? [-12, 12] : [12, -12]) across.push([rx - uz * s + ux * k, rz + ux * s + uz * k]);
  const raised = await page.evaluate((pts) => window.__land.stroke('raise', pts, 12, 0.7), across);
  await settled(page);
  await page.waitForTimeout(2500);
  const h1 = await page.evaluate(([x, z]) => window.__land.heightAt(x, z), [rx, rz]);
  // (`end`: the frame the stroke ended in, ms: its settling under the town and the surface solved again.)
  out.road = { at: [Math.round(rx), Math.round(rz)], rose: +(h1 - h0).toFixed(2), refused: raised.refused, stamps: spread(raised.stamps), end: Math.round(raised.frames.at(-1)) };
  console.log(`land: ${renderer} a raise across the road: ${JSON.stringify(out.road)}`);
  if (raised.refused) fault(`${renderer} the raise across the road was refused: ${raised.refused}`);
  await shot('17b-road-raised');
  // Under the home.
  const g0 = await page.evaluate(([x, z]) => window.__land.heightAt(x, z), home);
  const under = await page.evaluate(([x, z]) => window.__land.stroke('lower', Array.from({ length: 30 }, (_, k) => [x + Math.cos(k / 4) * 3, z + Math.sin(k / 4) * 3]), 10, 0.5), home);
  await settled(page);
  await page.waitForTimeout(2500);
  const g1 = await page.evaluate(([x, z]) => window.__land.heightAt(x, z), home);
  out.home = { fell: +(g0 - g1).toFixed(2), refused: under.refused, end: Math.round(under.frames.at(-1)) };
  console.log(`land: ${renderer} a lower under the home: ${JSON.stringify(out.home)}`);
  if (under.refused) fault(`${renderer} the lower under the home was refused: ${under.refused}`);
  await aim(page, home, 45);
  await shot('18-home-lowered');
  // Water across a country road away from the town: the road bridged there (its deck kept dry for walkers), or the
  // stroke refused with why; then undone (the bridge goes) and redone (it comes back).
  let crossing = null;
  for (const d of [140, 220, 320]) {
    for (const [ax, az] of [[1, 0], [0, 1], [-1, 0], [0, -1]]) {
      const c = await page.evaluate(([x, z]) => window.__land.nearestRoad(x, z, 'country'), [home[0] + ax * d, home[1] + az * d]);
      if (c && c.dist < 40 && c.s > 30) crossing ??= c;
    }
  }
  if (crossing) {
    const [cx, cz] = crossing.at;
    const [vx, vz] = crossing.along;
    await aim(page, [cx, cz], 70);
    await shot('19a-before-river');
    const before = crossing.bridges.length;
    // (A pond dug across it: on flat land a river with nowhere to run down to stands as a pond does.)
    const river = await page.evaluate((pts) => window.__land.stroke('lake', pts, 6, 0.6), Array.from({ length: 11 }, (_, k) => [cx - vz * (k * 2 - 10), cz + vx * (k * 2 - 10)]));
    await settled(page);
    await page.waitForTimeout(4000);
    const after = (await page.evaluate(([x, z]) => window.__land.nearestRoad(x, z, 'country'), [cx, cz]))?.bridges ?? [];
    const walk = await page.evaluate(([x, z]) => window.__land.simWet(x, z), [cx, cz]);
    out.bridge = { at: [Math.round(cx), Math.round(cz)], refused: river.refused, bridges: after, deckWalkable: !walk };
    console.log(`land: ${renderer} water across a country road: ${JSON.stringify(out.bridge)}`);
    if (!river.refused && after.length <= before) fault(`${renderer} the water across the road made no bridge`);
    if (!river.refused && walk) fault(`${renderer} the bridge's deck is water to the simulation`);
    await shot('19b-bridge');
    if (!river.refused) {
      await page.evaluate(() => window.__land.undo());
      await page.waitForTimeout(2500);
      const undone = (await page.evaluate(([x, z]) => window.__land.nearestRoad(x, z, 'country'), [cx, cz]))?.bridges ?? [];
      await page.evaluate(() => window.__land.redo());
      await page.waitForTimeout(2500);
      const redone = (await page.evaluate(([x, z]) => window.__land.nearestRoad(x, z, 'country'), [cx, cz]))?.bridges ?? [];
      out.bridge.undo = { undone: undone.length, redone: redone.length };
      if (undone.length !== before || redone.length !== after.length) fault(`${renderer} undo/redo of the bridge: ${JSON.stringify(out.bridge.undo)}`);
    }
  } else console.log(`land: ${renderer} no country road near the town for water to cross`);
  return out;
}

/**
 * The funds cut to $300, then a wide Raise held down with the pointer for two seconds beside `spot`:
 * the stroke stops before it costs more than the household has (the hint says so), it is sent and
 * paid, the funds don't go below nothing; then the funds given back.
 */
async function fundsRunOut(page, renderer, spot, shot) {
  const at = await page.evaluate(([x, z]) => window.__land.findFree(20, [x - 40, z]), spot);
  if (!at) return null;
  await aim(page, at, 90);
  const start = await funds(page);
  await page.evaluate((amount) => window.__addFunds(amount), 300 - start);
  await page.waitForTimeout(600);
  await page.getByRole('radio', { name: 'Raise' }).click();
  const xy = await page.evaluate(([x, z]) => window.__osw.project(x, window.__land.heightAt(x, z), z), at);
  const undo = await page.evaluate(() => window.__land.stats().undo);
  await page.mouse.move(xy.x, xy.y);
  await page.mouse.down();
  for (let k = 0; k < 40; k++) {
    await page.mouse.move(xy.x + Math.cos(k / 4) * 20, xy.y + Math.sin(k / 4) * 12);
    await page.waitForTimeout(50);
  }
  await shot('15a-funds-running-out');
  await page.mouse.up();
  await page.waitForTimeout(1200);
  const left = await funds(page);
  const hint = await page.evaluate(() => document.querySelector('.palette .hint')?.textContent?.trim() ?? '');
  const strokes = (await page.evaluate(() => window.__land.stats().undo)) - undo;
  await page.evaluate((amount) => window.__addFunds(amount), start - left);
  await page.waitForTimeout(600);
  const out = { left, hint, strokes };
  console.log(`land: ${renderer} with $300: ${JSON.stringify(out)}`);
  if (!(left >= 0 && left < 300 && strokes === 1 && /Out of money/.test(hint))) fault(`${renderer} the stroke at the funds' end: ${JSON.stringify(out)}`);
  return out;
}

/**
 * The ends of the brush's size range in step 1: 120 m raise, smooth and lake strokes on free land
 * (their frames timed: the stamp budget is 8 ms), a 1 m raise dragged across (each sample it
 * crosses changed, nothing else), the palette at 1 m ("one point") and at 120 m set with the
 * slider's Home and End keys, and the 1 m cursor close up, a 2 m ring on its sample.
 */
async function sizeEnds(page, renderer, mound, shot, stroke) {
  const r = {};
  const wide = await page.evaluate(([x, z]) => window.__land.findFree(70, [x + 40, z - 160]), mound);
  if (wide) {
    await lookAt(page, wide[0], wide[1], 0.8);
    r.wide = {};
    for (const [tool, strength] of [['raise', 0.5], ['smooth', 1], ['lake', 0.5]]) {
      const res = await stroke(`${tool} 120 m`, tool, line([wide[0] - 30, wide[1] - 10], [wide[0] + 30, wide[1] + 10], 1.5), 60, strength);
      r.wide[tool] = { frames: spread(res.frames.slice(1)), stamps: spread(res.stamps) };
      // (Noted, not a fault: the time is mostly the land drawn again, and was over before the range changed.)
      if (r.wide[tool].stamps.p95 > 8) console.log(`land: ${renderer} a 120 m ${tool} stamps and redraws at ${r.wide[tool].stamps.p95} ms (p95), over the 8 ms budget`);
    }
    await settled(page);
    await shot('9e-120m-strokes');
  } else console.log(`land: ${renderer} no free land 140 m across for the 120 m strokes`);

  // A 1 m raise dragged 40 m on the slant: one sample at each grid point it crosses, side to side.
  const spot = await page.evaluate(([x, z]) => window.__land.findFree(24, [x + 60, z + 60]), mound);
  if (!spot) {
    fault(`${renderer} no free land for the 1 m stroke`);
    return r;
  }
  const [a, b] = [[spot[0] - 20.3, spot[1] - 10.6], [spot[0] + 19.7, spot[1] + 9.4]];
  const grid = (p) => [Math.round(p[0] / 2), Math.round(p[1] / 2)];
  const heights = (p) =>
    page.evaluate(([[i0, j0]]) => {
      const out = [];
      for (let j = j0 - 14; j <= j0 + 14; j++) for (let i = i0 - 24; i <= i0 + 24; i++) out.push(window.__land.heightAt(i * 2, j * 2));
      return out;
    }, [p]);
  const centre = grid(spot);
  const before = await heights(centre);
  const res = await stroke('raise 1 m', 'raise', line(a, b, 0.7), 0.5, 1);
  const after = await heights(centre);
  const changed = after.filter((h, k) => h !== before[k]).length;
  const [ga, gb] = [grid(a), grid(b)];
  const crossed = Math.abs(gb[0] - ga[0]) + Math.abs(gb[1] - ga[1]) + 1;
  r.point = { frames: spread(res.frames.slice(1)), stamps: spread(res.stamps), changed, crossed };
  console.log(`land: ${renderer} a 1 m raise across ${crossed} grid points changed ${changed} samples`);
  if (changed !== crossed) fault(`${renderer} a 1 m raise across ${crossed} grid points changed ${changed} samples`);

  // The palette at both ends of the shape slider, set from the keyboard.
  await page.getByRole('radio', { name: 'Raise' }).click();
  const slider = page.locator('.palette .slider input[type=range]').first();
  const palette = page.locator('.palette').first();
  const ends = {};
  for (const [key, name] of [['End', '120m'], ['Home', '1m']]) {
    await slider.focus();
    await page.keyboard.press(key);
    await page.waitForTimeout(200);
    ends[name] = await page.evaluate(() => document.querySelector('.palette .slider .value')?.textContent?.trim());
    const file = join(out, `${renderer}-palette-${name}.png`);
    const box = await palette.boundingBox();
    await page.screenshot({ path: file, clip: { x: box.x - 8, y: box.y - 8, width: box.width + 16, height: box.height + 16 } });
    console.log(`land: ${file}`);
  }
  r.palette = ends;
  if (ends['120m'] !== '120 m' || ends['1m'] !== 'one point') fault(`${renderer} the palette says ${JSON.stringify(ends)}`);

  // The 1 m cursor close up, beside the stroke, on the pointer off its sample.
  await lookAt(page, spot[0] + 0.7, spot[1] - 6.6, 30);
  const v = await page.evaluate(() => {
    const r = document.querySelector('.view').getBoundingClientRect();
    return { x: r.left + r.width * 0.5, y: r.top + r.height * 0.5 };
  });
  await page.mouse.move(v.x - 3, v.y + 2);
  await page.waitForTimeout(600);
  await shot('9f-cursor-1m');
  const file = join(out, `${renderer}-9g-cursor-1m-close.png`);
  await page.screenshot({ path: file, clip: { x: v.x - 360, y: v.y - 220, width: 720, height: 440 } });
  console.log(`land: ${file}`);
  return r;
}

/** A step's way on (`label`, a pattern) enabled: its world made and built. */
const enabled = (page, label) => page.waitForFunction((src) => [...document.querySelectorAll('button')].some((b) => new RegExp(src).test(b.textContent ?? '') && !b.disabled), label.source, { timeout: 120_000, polling: 100 });

try {
  for (const renderer of renderers) {
    const context = await browser.newContext({ viewport: { width, height }, deviceScaleFactor: 1 });
    await context.addInitScript(quiet);
    await context.addInitScript(seeded, seed);
    const page = await context.newPage();
    page.on('pageerror', (e) => fault(`${renderer} page error: ${e.message}`));
    page.on('console', (m) => m.type() === 'error' && console.warn(`  console: ${m.text().slice(0, 300)}`));
    const query = ['debug', renderer === 'webgl' ? 'renderer=webgl' : '', ...worldQuery].filter(Boolean).join('&');
    await page.goto(`${base}/?${query}`);
    await page.waitForFunction(() => !!document.querySelector('.menu') && !document.querySelector('.loading'), null, { timeout: 180_000, polling: 200 });
    await page.waitForTimeout(1500);
    await page.getByText('New game', { exact: true }).click();
    await enabled(page, /Lay out the town/);
    await page.waitForTimeout(1500);
    const shot = async (name) => {
      const file = join(out, `${renderer}-${name}.png`);
      await page.screenshot({ path: file });
      console.log(`land: ${file}`);
    };
    await shot('1-world');
    const r = { strokes: {} };
    report[renderer] = r;
    let mound = null;
    let pond = null;

    if (!plain) {
    // Shape the land: surveyed behind a spinner, then drawn from the field.
    const t0 = Date.now();
    await page.getByRole('radio', { name: 'Shape the land' }).click();
    await page.waitForFunction(() => {
      try {
        return !!window.__land?.stats();
      } catch {
        return false;
      }
    }, null, { timeout: 30_000, polling: 50 });
    const surveyed = Date.now() - t0;
    await settled(page);
    await page.waitForTimeout(2500);
    await shot('2-shape');
    r.surveyMs = surveyed;

    const stroke = async (name, ...a) => {
      const res = await page.evaluate((a) => window.__land.stroke(...a), a);
      r.strokes[name] = { frames: spread(res.frames.slice(1)), stamps: spread(res.stamps), changed: res.changed, hint: res.hint, refused: res.refused };
      console.log(`land: ${renderer} ${name}: frames ${JSON.stringify(r.strokes[name].frames)} stamps ${JSON.stringify(r.strokes[name].stamps)}${res.hint ? ` · ${res.hint}` : ''}${res.refused ? ` · refused: ${res.refused}` : ''}`);
      if (!res.changed) fault(`${renderer} ${name} changed nothing`);
      return res;
    };

    // A mound on free land near the middle of the town's view: raised, then its top flattened (a plateau, the pond's
    // ground: on a dome the pond cut into its side).
    const view = await page.evaluate(() => {
      const v = document.querySelector('.view').getBoundingClientRect();
      return window.__land.pick(v.left + v.width * 0.5, v.top + v.height * 0.55);
    });
    mound = await page.evaluate(([x, z]) => window.__land.findFree(38, [x, z]), [view?.x ?? 0, view?.z ?? 0]);
    if (!mound) throw new Error('no free land for the mound');
    await lookAt(page, mound[0], mound[1], 2.2);
    await shot('3-before');
    const before = await page.evaluate(([x, z]) => window.__land.heightAt(x, z), mound);
    for (let k = 0; k < 3; k++) await stroke(`mound ${k + 1}`, 'raise', ring(mound[0], mound[1], 4, 40), 14, 0.3);
    // (Flattened to the height at its middle, the stroke's first point: the land round the top brought up to it.)
    for (let k = 0; k < 2; k++) await stroke(`plateau ${k + 1}`, 'flatten', [mound, ...ring(mound[0], mound[1], 5, 40)], 12, 1);
    const top = await page.evaluate(([x, z]) => window.__land.heightAt(x, z), mound);
    // (How flat its top is: the spread of the heights 6 m out round it.)
    const rim = await page.evaluate(([x, z]) => Array.from({ length: 12 }, (_, k) => window.__land.heightAt(x + Math.cos((k / 12) * Math.PI * 2) * 6, z + Math.sin((k / 12) * Math.PI * 2) * 6)), mound);
    r.mound = { at: mound, rose: +(top - before).toFixed(2), rim: +(Math.max(...rim) - Math.min(...rim)).toFixed(2), rimBelow: +(top - Math.min(...rim)).toFixed(2) };
    console.log(`land: ${renderer} the plateau: ${JSON.stringify(r.mound)}`);
    console.log(`land: ${renderer} the mound rose ${(top - before).toFixed(1)} m`);
    if (top - before < 2) fault(`${renderer} the mound rose only ${(top - before).toFixed(1)} m`);
    await settled(page);
    await shot('4-mound');

    // A pond on its top, at its own level, inside the plateau.
    pond = [...mound];
    await stroke('pond', 'lake', ring(pond[0], pond[1], 2, 12), 6, 0.4);
    const wet = await page.evaluate(([x, z]) => window.__land.wet(x, z), pond);
    if (!wet) fault(`${renderer} no pond on the mound`);
    await settled(page);
    await shot('5-pond');

    // A river down to the nearest water: from as far up the land towards the mound as a river can
    // start and still run down to it (no more than a few metres over the water, never rising on the way).
    const water = await page.evaluate(([x, z]) => window.__land.findWater([x, z]), pond);
    if (water) {
      // (The longest straight run in to some shore over free land that only falls on the way, from
      // the shores nearest the mound first.)
      const run = await page.evaluate(
        ([mx, mz]) => {
          const L = window.__land;
          const { cols, rows } = L.stats();
          const shores = [];
          for (let z = 4; z < rows * 2; z += 12) for (let x = 4; x < cols * 2; x += 12) if (L.wet(x, z) && [[12, 0], [-12, 0], [0, 12], [0, -12]].some(([dx, dz]) => !L.wet(x + dx, z + dz))) shores.push([x, z]);
          shores.sort((a, b) => Math.hypot(a[0] - mx, a[1] - mz) - Math.hypot(b[0] - mx, b[1] - mz));
          for (const [wx, wz] of shores.slice(0, 60)) {
            const low = L.heightAt(wx, wz);
            let best = null;
            let longest = 0;
            for (let a = 0; a < 24; a++) {
              const [ux, uz] = [Math.cos((a / 24) * Math.PI * 2), Math.sin((a / 24) * Math.PI * 2)];
              let last = null;
              let top = -Infinity;
              for (let s = 4; s <= 160; s += 4) {
                const [x, z] = [wx + ux * s, wz + uz * s];
                if (L.wet(x, z)) continue;
                const h = L.heightAt(x, z);
                // (Out from the water the land only rises: in to it, the river only falls.)
                if (L.free(x, z) < 0.6 || h - low > 5 || h < top - 0.3) break;
                top = Math.max(top, h);
                last = [x, z, s];
              }
              if (last && last[2] > longest) [best, longest] = [[last[0], last[1]], last[2]];
            }
            if (longest >= 40) return { from: best, to: [wx, wz] };
          }
          return null;
        },
        pond,
      );
      const from = run?.from;
      if (run) water.splice(0, 2, ...run.to);
      if (from) {
        await lookAt(page, (from[0] + water[0]) / 2, (from[1] + water[1]) / 2, 1);
        await shot('6-before-river');
        const res = await stroke('river', 'river', line(from, water, 3), 7, 0.5);
        r.river = { from, to: water, length: Math.round(Math.hypot(water[0] - from[0], water[1] - from[1])), hint: res.hint };
        await settled(page);
        await shot('6-river');
      } else console.log(`land: ${renderer} no way down to the water for a river`);
    } else console.log(`land: ${renderer} no water to run a river to`);

    // Woods painted on open land near the mound, then a strip cleared through them.
    const woods = await page.evaluate(([x, z]) => window.__land.findFree(30, [x + 80, z + 20]), mound);
    if (woods) {
      await lookAt(page, woods[0], woods[1], 1);
      await stroke('woods', 'woods', line([woods[0] - 22, woods[1]], [woods[0] + 22, woods[1]], 2), 24, 1);
      await settled(page);
      await page.waitForTimeout(1500);
      await shot('7-woods');
      await stroke('clear', 'clear', line([woods[0], woods[1] - 26], [woods[0], woods[1] + 26], 2), 12, 1);
      await settled(page);
      await page.waitForTimeout(1500);
      await shot('8-cleared');
    }

    // A peak away from the town.
    const peak = await page.evaluate(([x, z]) => window.__land.findFree(80, [x - 160, z - 120]), mound);
    if (peak) {
      await lookAt(page, peak[0], peak[1], 0.6);
      await stroke('peak', 'peak', [peak, [peak[0] + 1, peak[1]]], 80, 0.5, 45);
      await settled(page);
      await page.waitForTimeout(1500);
      await shot('9-peak');
    }

    // By hand: the cursor over the land, then a range dragged out with the pointer (its ghost
    // while dragging), let go, and taken back with the palette's Undo.
    await lookAt(page, mound[0], mound[1], 1);
    const v = await page.evaluate(() => {
      const r = document.querySelector('.view').getBoundingClientRect();
      return { x: r.left + r.width * 0.5, y: r.top + r.height * 0.5, w: r.width, h: r.height };
    });
    await page.getByRole('radio', { name: 'Range' }).click();
    await page.mouse.move(v.x - v.w * 0.2, v.y + v.h * 0.1);
    await page.waitForTimeout(500);
    await shot('9a-cursor');
    const undoBefore = await page.evaluate(() => window.__land.stats().undo);
    await page.mouse.down();
    for (let k = 1; k <= 30; k++) {
      await page.mouse.move(v.x - v.w * 0.2 + k * v.w * 0.012, v.y + v.h * 0.1 - k * v.h * 0.006);
      await page.waitForTimeout(16);
    }
    await page.waitForTimeout(300);
    const ghost = await page.evaluate(() => document.querySelector('svg.ghost path')?.getAttribute('d')?.length ?? 0);
    await shot('9b-range-ghost');
    if (!ghost) fault(`${renderer} no ghost while dragging a range`);
    await page.mouse.up();
    await settled(page);
    await page.waitForTimeout(1200);
    await shot('9c-range');
    const undoAfter = await page.evaluate(() => window.__land.stats().undo);
    if (undoAfter !== undoBefore + 1) fault(`${renderer} the dragged range made ${undoAfter - undoBefore} strokes`);
    await page.getByRole('button', { name: 'Undo' }).click();
    await settled(page);
    if ((await page.evaluate(() => window.__land.stats().undo)) !== undoBefore) fault(`${renderer} Undo didn't take the range back`);
    // The camera by hand: a right-drag turns it, a middle-drag pans it.
    const pick = () => page.evaluate(([x, y]) => window.__land.pick(x, y), [v.x, v.y]);
    const p0 = await pick();
    await page.mouse.move(v.x, v.y);
    await page.mouse.down({ button: 'middle' });
    await page.mouse.move(v.x + 200, v.y + 60, { steps: 10 });
    await page.mouse.up({ button: 'middle' });
    await page.waitForTimeout(1500);
    const p1 = await pick();
    r.panned = p0 && p1 ? Math.round(Math.hypot(p1.x - p0.x, p1.z - p0.z)) : null;
    if (!r.panned) fault(`${renderer} a middle-drag didn't pan (${JSON.stringify([p0, p1])})`);
    await page.mouse.down({ button: 'right' });
    await page.mouse.move(v.x + 300, v.y + 60, { steps: 10 });
    await page.mouse.up({ button: 'right' });
    await page.waitForTimeout(1500);
    await shot('9d-turned');

    // The ends of the size range (last in step 1: it leaves the camera close in).
    r.sizes = await sizeEnds(page, renderer, mound, shot, stroke);
    const stats = await page.evaluate(() => window.__land.stats());
    r.stats = stats;
    console.log(`land: ${renderer} stats ${JSON.stringify(stats)} · panned ${r.panned} m`);
    }

    if (!has('no-play')) {
      await page.getByRole('button', { name: /Lay out the town/ }).click();
      await enabled(page, /Choose a household/);
      await page.waitForTimeout(1500);
      await shot('10-step2');
      await page.getByRole('button', { name: /Choose a household/ }).click();
      // Step 3's cards (their portraits drawn), a household's Play, then step 4's Move in (as gameplay.mjs goes).
      await page.waitForFunction(() => {
        const faces = [...document.querySelectorAll('.offer .face')];
        return faces.length > 0 && faces.every((f) => f.querySelector('img'));
      }, null, { timeout: 180_000, polling: 100 });
      if (mode === 'creative') await page.getByRole('radio', { name: 'Creative' }).click();
      await page.waitForTimeout(1500);
      await shot('10-step3');
      await page.locator('.offer .card[data-slot] .go').first().click();
      await page.waitForFunction(() => !!document.querySelector('.btn.move-in:not(:disabled)'), null, { timeout: 180_000, polling: 100 });
      await page.waitForTimeout(500);
      await page.locator('.btn.move-in').click();
      const asked = () => [...document.querySelectorAll('button')].some((b) => b.textContent?.trim() === 'Move in anyway');
      await page.waitForFunction(`(${asked})() || (!!document.querySelector('.hud-root') && !document.querySelector('.loading'))`, null, { timeout: 180_000, polling: 100 });
      if (await page.evaluate(asked)) await page.getByRole('button', { name: 'Move in anyway' }).click();
      await inGame(page);
      await page.waitForTimeout(4000);
      await shot('11-play');
      r.play = await play(page, renderer, pond, shot);
    }
    await context.close();
  }
} finally {
  writeFileSync(join(out, 'land.json'), JSON.stringify(report, null, 2));
  await browser.close();
  server?.kill();
}
console.log(`land: ${faults ? `${faults} faults` : 'no faults'} · ${out}`);
process.exit(faults ? 1 : 0);
