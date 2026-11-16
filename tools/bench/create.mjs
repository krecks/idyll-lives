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
 * With `--shots …,edit`, between steps 2 and 3, a scripted edit session in step 2's town editor
 * (`web/src/ui/townedit/`), each step shot at the edit sizes (`--edit-sizes`): a lot picked, removed,
 * the removal undone, a home put down, a lot dragged elsewhere, a park made another public place, a
 * street renamed, then a street removed with its lots. It measures the frames while the Lot tool's
 * ghost follows the pointer, and each edit's way to the town built again (the pause, the settle, the
 * town shown: the editor's own timings), and says them (`create: edit …`).
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
 *     --shots 1,2,3                          the steps shot (the others only gone through); `edit`: step 2's edit session
 *     --edit-sizes 1920x1080,740x1000        the edit session's shots' sizes (it runs at the first)
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
const shots = new Set(flag('shots', '1,2,3').split(','));
const editSizes = flag('edit-sizes', '1920x1080,740x1000')
  .split(',')
  .map((s) => s.split('x').map(Number))
  .map(([width, height]) => ({ width, height }));
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

/**
 * In the page: helpers for the edit session (`window.__bench`), over the town editor (`window.__townEditor`, dev only)
 * and the step's town: where a thing of the plan is on screen (a point the stage itself is under, no panel over it),
 * frames counted while the pointer moves.
 */
function benchHelpers() {
  return (async () => {
    const [{ app }, { menuScene }, lots, streets, town, pickMod] = await Promise.all([
      import('/src/ui/app.svelte.ts'),
      import('/src/game/menuScene.ts'),
      import('/src/game/edit/lots.ts'),
      import('/src/game/edit/streets.ts'),
      import('/src/game/town.ts'),
      import('/src/ui/townedit/pick.ts'),
    ]);
    const t = await town.loadTemplates();
    const ed = () => window.__townEditor;
    const plan = () => ed().draft;
    /** A ground point (m) on screen (CSS px), at the land's height there; null if no stage is under it. */
    const screen = (x, z) => {
      const out = { x: 0, y: 0 };
      let y = 0;
      for (let k = 0; k < 3; k++) {
        if (!menuScene.project(x, y, z, out)) return null;
        const g = menuScene.landAt(out.x, out.y);
        if (!g) break;
        y = g.y;
      }
      const el = document.elementFromPoint(out.x, out.y);
      return el?.classList.contains('stage') ? { x: out.x, y: out.y } : null;
    };
    const centre = (s) => {
      const r = town.plotRect(t, s);
      return [r.x + r.w / 2, r.z + r.d / 2];
    };
    const pointAt = (pts, s) => {
      let at = 0;
      for (let i = 0; i + 1 < pts.length; i++) {
        const l = Math.hypot(pts[i + 1][0] - pts[i][0], pts[i + 1][1] - pts[i][1]);
        if (at + l >= s) {
          const f = (s - at) / (l || 1);
          return [pts[i][0] + (pts[i + 1][0] - pts[i][0]) * f, pts[i][1] + (pts[i + 1][1] - pts[i][1]) * f];
        }
        at += l;
      }
      return pts[pts.length - 1];
    };
    const length = (pts) => pts.reduce((a, p, i) => (i ? a + Math.hypot(p[0] - pts[i - 1][0], p[1] - pts[i - 1][1]) : 0), 0);
    let frames = null;
    window.__bench = {
      /** A lot of the main town of kind `kind` ('home', 'park') on screen: its slot and point. */
      lot(kind, skip = 0) {
        const d = plan();
        const ok = (s) => (kind === 'home' ? s.kind === 'house' && s.lot !== 'farm' && s.lot !== 'terrace' : s.kind === kind) && (s.area ?? 0) === 0;
        let n = 0;
        for (const s of d.slots) {
          if (!ok(s)) continue;
          const p = screen(...centre(s));
          if (p && n++ >= skip) return { slot: s.index, name: s.name, ...p };
        }
        return null;
      },
      /** A road of the main town on screen (its middle), and what removing it takes: one taking homes (`withLots`) or none. */
      road(withLots) {
        const d = plan();
        for (const [i, r] of d.roads.entries()) {
          if (r.kind === 'trail' || r.bridge || (r.area ?? -1) !== 0) continue;
          const goes = streets.whatGoes(d, t, i);
          if (withLots !== undefined && ('refused' in goes || (goes.lots.length > 0) !== withLots)) continue;
          const L = length(r.points);
          for (const f of [0.5, 0.35, 0.65, 0.2, 0.8]) {
            const p = screen(...pointAt(r.points, L * f));
            if (!p) continue;
            const picked = pickMod.pick(d, t, ...pointAt(r.points, L * f));
            if (picked?.kind === 'road' && picked.road === i) return { road: i, name: r.name, lots: 'refused' in goes ? 0 : goes.lots.length, ...p };
          }
        }
        return null;
      },
      /** Where a lot of `what` (a `LotChoice`, or `{ slot }` to move one) may go along the roads (the main town's first, then the rest): points on screen of the lots' middles, in turn. */
      spots(what, most = 12) {
        const d = plan();
        const out = [];
        const order = d.roads.map((r, i) => i).sort((a, b) => ((d.roads[a].area ?? 9) === 0 ? 0 : 1) - ((d.roads[b].area ?? 9) === 0 ? 0 : 1));
        for (const i of order) {
          const r = d.roads[i];
          if (r.kind === 'trail' || r.bridge) continue;
          const L = length(r.points);
          for (let s = 8; s < L - 8 && out.length < most; s += 7) {
            for (const side of [1, -1]) {
              const fit = lots.canPlace(d, t, { road: i, s, side }, what, 1);
              if ('refused' in fit) continue;
              const p = screen(fit.x + fit.w / 2, fit.z + fit.d / 2);
              if (p) out.push(p);
            }
          }
        }
        return out;
      },
      /** The page's frames from now (their gaps, ms) until `stop`: the 50th, 95th percentiles and the longest. */
      startFrames() {
        frames = [];
        let last = performance.now();
        const tick = (now) => {
          if (!frames) return;
          frames.push(now - last);
          last = now;
          requestAnimationFrame(tick);
        };
        requestAnimationFrame(tick);
      },
      stopFrames() {
        const f = (frames ?? []).slice(1).sort((a, b) => a - b);
        frames = null;
        const q = (k) => f[Math.min(f.length - 1, Math.floor(f.length * k))] ?? 0;
        return { frames: f.length, p50: q(0.5), p95: q(0.95), max: f[f.length - 1] ?? 0 };
      },
      /** The editor: what it says, whether it is settling, the settles' timings, the Problems list. */
      state() {
        const e = ed();
        return { hint: e.hintText, note: e.note, settling: e.settling, timings: [...e.timings], blocking: e.issues.blocking.map((i) => i.text), warnings: e.issues.warnings.length, edits: e.edits, selected: e.selected, slots: e.draft.slots.length, roads: e.draft.roads.length, edited: app.world?.edited };
      },
    };
    return true;
  })();
}

/**
 * Step 2's edit session (see the top of this file): each step shot at every edit size, each edit's way to the town
 * built again measured, the frames while the Lot tool's ghost follows the pointer.
 */
async function editSession(page, renderer, shootAs, setStep) {
  setStep('edit');
  const [first] = editSizes;
  await page.setViewportSize(first);
  await page.waitForTimeout(2500);
  await page.evaluate(benchHelpers);
  const bench = (fn, arg) => page.evaluate(fn, arg);
  const state = () => bench(() => window.__bench.state());
  let n = 0;
  /** Each edit size shot (`label`), back at the first after. */
  const shots = async (label) => {
    n++;
    const name = `${String(n).padStart(2, '0')}-${label}`;
    for (const size of editSizes) {
      if (size !== first) {
        await page.setViewportSize(size);
        await page.waitForTimeout(2200);
      }
      await shootAs(`${size.width}x${size.height}-${name}`);
    }
    if (editSizes.length > 1) {
      await page.setViewportSize(first);
      await page.waitForTimeout(2200);
    }
  };
  /** The town built again with the edits (the editor no longer settling); its settle's timing said. */
  const built = async (what) => {
    const t0 = Date.now();
    await page.waitForFunction(() => !window.__townEditor.settling, null, { timeout: 60_000, polling: 50 }).catch(() => fault(`${renderer} edit: ${what}: the town not built again within 60 s`));
    const s = await state();
    const last = s.timings[s.timings.length - 1];
    if (last) console.log(`create: ${renderer} edit ${what}: built again ${Math.round(last.fromEdit)} ms after the edit (the pause, then settle ${Math.round(last.settle)} ms, town shown ${Math.round(last.show)} ms; waited ${Date.now() - t0} ms)`);
    return s;
  };
  const tool = async (id) => {
    const on = await page.evaluate((id) => document.querySelector(`[data-tool="${id}"]`)?.getAttribute('aria-checked') === 'true', id);
    if (!on) await page.locator(`[data-tool="${id}"]`).click();
    await page.waitForTimeout(150);
  };
  const click = async (p) => {
    await page.mouse.move(p.x, p.y, { steps: 4 });
    await page.waitForTimeout(80);
    await page.mouse.click(p.x, p.y);
    await page.waitForTimeout(250);
  };

  // 1. A home picked: its card.
  await tool('select');
  const home = await bench(() => window.__bench.lot('home', 2));
  if (!home) return fault(`${renderer} edit: no home on screen to pick`);
  await click(home);
  if ((await state()).selected?.kind !== 'lot') fault(`${renderer} edit: a click on ${home.name} picked ${JSON.stringify((await state()).selected)}`);
  await shots('select');

  // 2. Removed from its card: its ghost greyed at once, the town built again after the pause.
  const before = (await state()).slots;
  await page.locator('.card button', { hasText: /^\s*Remove\s*$/ }).click();
  await page.waitForTimeout(200);
  if ((await state()).slots !== before - 1) fault(`${renderer} edit: removing ${home.name} left ${(await state()).slots} lots (of ${before})`);
  await shots('remove');
  await built('remove a lot');
  await shots('removed-built');

  // 3. Undone.
  await page.keyboard.press(process.platform === 'darwin' ? 'Meta+z' : 'Control+z');
  await page.waitForTimeout(250);
  if ((await state()).slots !== before) fault(`${renderer} edit: undo left ${(await state()).slots} lots (of ${before})`);
  await built('undo');
  await shots('undo');

  // 4. A home put down with the Lot tool, the frames measured while its ghost follows the pointer.
  await tool('lot');
  await page.locator('.lot-card [role="radio"]', { hasText: 'Home' }).first().click();
  const spots = await bench(() => window.__bench.spots({ kind: 'house', size: 'medium' }));
  if (!spots.length) fault(`${renderer} edit: nowhere on screen for a home`);
  if (spots.length > 1) {
    /** The frames while the pointer goes along the spots (the same way each time). */
    const along = async () => {
      await page.mouse.move(spots[0].x, spots[0].y);
      await bench(() => window.__bench.startFrames());
      for (const p of spots.slice(1)) await page.mouse.move(p.x, p.y, { steps: 20 });
      return bench(() => window.__bench.stopFrames());
    };
    const line = (f) => `${f.frames} frames, p50 ${f.p50.toFixed(1)} ms, p95 ${f.p95.toFixed(1)} ms, longest ${f.max.toFixed(1)} ms`;
    // (The same way with no tool in hand first: what the page does anyway, the lots lit as the pointer goes.)
    await page.keyboard.press('Escape');
    const bare = await along();
    await tool('lot');
    const f = await along();
    console.log(`create: ${renderer} edit: frames while the lot's ghost follows the pointer: ${line(f)} (with no tool in hand: ${line(bare)})`);
  }
  let placed = false;
  for (const p of spots) {
    await page.mouse.move(p.x, p.y, { steps: 3 });
    await page.waitForTimeout(60);
    if (!/^Click to put it down/.test((await state()).hint ?? '')) continue;
    await shots('add-ghost');
    const had = (await state()).slots;
    await page.mouse.click(p.x, p.y);
    await page.waitForTimeout(200);
    placed = (await state()).slots === had + 1;
    break;
  }
  if (!placed) fault(`${renderer} edit: no home put down (${(await state()).hint})`);
  await shots('add');
  await built('add a home');

  // 5. A lot dragged elsewhere with the Lot tool.
  const mover = await bench(() => window.__bench.lot('home', 6));
  if (mover) {
    const targets = await bench((slot) => window.__bench.spots({ slot }), mover.slot);
    await page.mouse.move(mover.x, mover.y, { steps: 3 });
    await page.mouse.down();
    let moved = false;
    for (const p of targets) {
      await page.mouse.move(p.x, p.y, { steps: 8 });
      await page.waitForTimeout(60);
      if (/^Let go to move/.test((await state()).hint ?? '')) {
        await shots('move-drag');
        await page.mouse.up();
        moved = true;
        break;
      }
    }
    if (!moved) {
      await page.mouse.up();
      fault(`${renderer} edit: ${mover.name} found nowhere to go (${(await state()).hint})`);
    }
    await page.waitForTimeout(200);
    await shots('move');
    await built('move a lot');
  } else fault(`${renderer} edit: no home on screen to move`);

  // 6. A park made another public place, from its card.
  await tool('select');
  const park = await bench(() => window.__bench.lot('park'));
  if (park) {
    await click(park);
    const chip = page.locator('.card .chip[aria-checked="false"]').first();
    if (await chip.count()) {
      await chip.click();
      await page.waitForTimeout(250);
      await shots('swap-park');
      await built('swap a park');
    } else fault(`${renderer} edit: ${park.name}'s card offers no other place`);
  } else fault(`${renderer} edit: no park on screen`);

  // 7. A street renamed from its card.
  const street = await bench(() => window.__bench.road());
  if (street) {
    await click(street);
    const field = page.locator('.card input.rename');
    await field.fill('Bench Street');
    await field.press('Enter');
    await page.waitForTimeout(250);
    const named = await page.evaluate(() => window.__townEditor.draft.roads.some((r) => r.name === 'Bench Street'));
    if (!named) fault(`${renderer} edit: ${street.name} not renamed (${(await state()).note})`);
    await shots('rename');
  } else fault(`${renderer} edit: no street on screen`);

  // 8. A street removed with its lots, from its card (what goes listed first).
  const doomed = await bench(() => window.__bench.road(true));
  if (doomed) {
    await click(doomed);
    await shots('street-card');
    const had = (await state()).slots;
    await page.locator('.card button', { hasText: /^\s*Remove / }).click();
    await page.waitForTimeout(250);
    const s = await state();
    if (s.slots >= had) fault(`${renderer} edit: removing ${doomed.name} took no lots (${s.note})`);
    await shots('remove-street');
    await built('remove a street with its lots');
    await shots('remove-street-built');
  } else fault(`${renderer} edit: no street with lots that may go`);
  const end = await state();
  console.log(`create: ${renderer} edit: ${end.edits} edits, ${end.slots} lots, ${end.roads} roads; problems: ${end.blocking.length} blocking${end.blocking.length ? ` (${end.blocking.join('; ')})` : ''}, ${end.warnings} warnings; edited: ${end.edited}`);
  await page.keyboard.press('Escape');
  await page.waitForTimeout(300);
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
    if (shots.has('1')) await everySize();

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
    if (shots.has('1')) await shoot(`${sizes[0].width}x${sizes[0].height}-making`);
    await ready(page);
    await page.waitForTimeout(2500);
    if (shots.has('1')) await shoot(`${sizes[0].width}x${sizes[0].height}-place`);

    // Step 2: the town laid on that land (its making panel shot while it is laid), then at every size.
    await page.getByRole('button', { name: /Lay out the town/ }).click();
    stepName = 'step2';
    await page.waitForFunction(() => !!document.querySelector('.making'), null, { timeout: 10_000, polling: 50 }).catch(() => fault(`${renderer}: no making panel while the town is laid out`));
    await page.waitForTimeout(700);
    if (shots.has('2')) await shoot(`${sizes[0].width}x${sizes[0].height}-making`);
    await townReady(page);
    const eyebrow = await page.evaluate(() => document.querySelector('.side .head .eyebrow')?.textContent ?? '');
    if (!/Step 2 of 4 · Town/.test(eyebrow)) fault(`${renderer}: step 2 says "${eyebrow}"`);
    if (shots.has('2')) await everySize();

    if (shots.has('edit')) await editSession(page, renderer, (name) => shoot(name), (name) => (stepName = name));

    if (shots.has('3')) {
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
