#!/usr/bin/env bun
/**
 * Makes a benchmark scene (`web/public/bench/<name>.json.gz`, see web/src/debug/bench.ts): a new game
 * as the menus make one (the menu's valley, or a new world of `--world`; its neighbours as the
 * neighbourhood step makes them; a household of four in the first home for sale), played at full
 * speed to `--hour` (5 minutes before it) on day 1, paused and saved.
 *
 *   bun tools/bench/scene.mjs --name village [--world medium|forest|mountain|…] [--hour 18] [--port 5221]
 *
 * `--seed <n>`: the new world's seed (the same kind and seed give the same town, neighbours and household; with it
 * `medium` is a new world too, not the menu's valley). `--home town`: the household moves into a home for sale in the main town (the one
 * halfway out among those for sale there) instead of the first for sale anywhere. The town view's scenes
 * (`render.mjs --poses town`: `town-day`, `town-dusk`, `town-night`) are made so, one town at three hours:
 *
 *   bun tools/bench/scene.mjs --name town-day --world small --seed 7 --home town --hour 13
 *   bun tools/bench/scene.mjs --name town-dusk --world small --seed 7 --home town --hour 19.5
 *   bun tools/bench/scene.mjs --name town-night --world small --seed 7 --home town --hour 23
 *
 * Starts its own Vite on `--port` (or uses `--url`). Needs Playwright like render.mjs, and runs
 * headless on the real GPU like it (`--headed` for a window).
 */

import { spawn } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { gzipSync } from 'node:zlib';
import { quiet } from './quiet.mjs';

const WEB = fileURLToPath(new URL('../../web/', import.meta.url));
const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined ? args[i + 1] : fallback;
};
const name = flag('name', 'village');
const world = flag('world', 'medium');
const seed = flag('seed', null) === null ? null : Number(flag('seed', null));
const homeIn = flag('home', 'first');
const goal = Number(flag('hour', '18')) * 60 - 5;
const port = Number(flag('port', '5221'));
const base = flag('url', null) ?? `http://localhost:${port}`;

async function playwright() {
  if (process.env.PLAYWRIGHT) return import(process.env.PLAYWRIGHT);
  return import(createRequire(WEB + 'package.json').resolve('playwright'));
}

let vite = null;
if (!flag('url', null)) {
  vite = spawn(`${WEB}node_modules/.bin/vite`, ['--port', String(port), '--strictPort'], { cwd: WEB, stdio: ['ignore', 'pipe', 'inherit'] });
  process.on('exit', () => vite.kill());
  await new Promise((resolve, reject) => {
    vite.stdout.on('data', (d) => String(d).includes('Local') && resolve());
    vite.on('exit', (code) => reject(new Error(`vite exited (${code})`)));
  });
}

const { chromium } = await playwright();
const browser = await chromium.launch({ headless: !process.argv.includes('--headed'), ...(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chromium' }), args: ['--enable-unsafe-webgpu', '--disable-backgrounding-occluded-windows', '--disable-renderer-backgrounding', '--disable-background-timer-throttling'] });
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  await page.addInitScript(quiet);
  page.on('pageerror', (e) => console.error(`page error: ${e.message}`));
  await page.goto(`${base}/?debug&seed=7`);
  // (Dev server modules, imported by their paths: the same instances the game uses.)
  await page.waitForFunction(async () => !(await import('/src/ui/app.svelte.ts')).app.booting, null, { timeout: 120_000, polling: 500 });
  const started = await page.evaluate(
    async ({ kind, seed, homeIn }) => {
      const { app } = await import('/src/ui/app.svelte.ts');
      const { menuScene } = await import('/src/game/menuScene.ts');
      const { services } = await import('/src/ui/services.ts');
      const { generateWorld, populate, loadTemplates, vacantSlots } = await import('/src/game/town.ts');
      const { randomHousehold } = await import('/src/game/household.ts');
      const t = await loadTemplates();
      const empty = kind === 'medium' && seed === null ? { ...(await menuScene.showcaseDraft()), households: [] } : generateWorld(services.content, t, kind, seed ?? undefined);
      // (With a seed, the neighbours and the household are the same ones every time too: their dice are the seed's
      // while they are made, the household's its own, however many the neighbours took.)
      const dice = Math.random;
      const seeded = (n) => {
        let a = (n ^ 0x9e3779b9) >>> 0;
        Math.random = () => {
          a = (a + 0x6d2b79f5) >>> 0;
          let t = Math.imul(a ^ (a >>> 15), 1 | a);
          t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
          return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
        };
      };
      if (seed !== null) seeded(seed);
      const town = populate(services.content, services.assets, t, empty);
      if (seed !== null) seeded(seed + 1);
      const household = randomHousehold(services.content, services.assets, 4, town.households.map((h) => h.household.name));
      Math.random = dice;
      // The home: the first for sale, or (`--home town`) the one halfway out among those for sale in the main town.
      const vacant = vacantSlots(town);
      let slot = vacant[0].index;
      const main = town.areas?.[0];
      if (homeIn === 'town' && main) {
        const [cx, cz] = [main.x + main.w / 2, main.z + main.d / 2];
        const far = (s) => Math.hypot(s.x - cx, s.z - cz);
        const inMain = vacant.filter((s) => (s.area ?? 0) === 0).sort((a, b) => far(a) - far(b) || a.index - b.index);
        if (inMain.length) slot = inMain[Math.floor(inMain.length / 2)].index;
      }
      Object.assign(app, { world: empty, town, household, homeSlot: slot });
      app.start(app.homeRequest(slot));
      return { world: town.world?.kind, seed: town.seed, slot, households: town.households.length, neighbours: town.households.reduce((n, h) => n + h.household.members.length, 0) };
    },
    { kind: world, seed, homeIn },
  );
  console.log(`scene: started`, started);
  await page.waitForFunction(() => !!window.__osw && !!document.querySelector('.hud-root'), null, { timeout: 120_000, polling: 500 });
  await page.waitForTimeout(2000);
  const minute = () => page.evaluate(() => window.__osw.snapshotMinute());
  // Fast to just before the hour, then at normal speed to it.
  for (const [speed, until] of [[5, goal - 40], [1, goal]]) {
    await page.evaluate((s) => window.__osw.send({ type: 'setSpeed', speed: s }), speed);
    for (let i = 0; i < 1200 && (await minute()) < until; i++) await page.waitForTimeout(150);
  }
  await page.evaluate(() => window.__osw.send({ type: 'setSpeed', speed: 0 }));
  await page.waitForTimeout(500);
  const state = await page.evaluate(async () => (await import('/src/ui/services.ts')).services.controls.currentState());
  const home = await page.evaluate(() => {
    const plot = window.__osw.home()?.id;
    return window.__osw.sims().filter((s) => s.plot === plot).map((s) => `${s.name}: ${s.doing ?? 'idle'}`);
  });
  const file = join(WEB, 'public/bench', `${name}.json.gz`);
  writeFileSync(file, gzipSync(state, { level: 9 }));
  console.log(`scene: ${file} at minute ${(await minute()).toFixed(0)}, ${(state.length / 1024).toFixed(0)} KB of JSON; at home: ${home.join(', ')}`);
} finally {
  await browser.close();
  vite?.kill();
}
