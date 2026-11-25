#!/usr/bin/env bun
/**
 * Makes a benchmark scene (`web/public/bench/<name>.json.gz`, see web/src/debug/bench.ts): a new game
 * as the menus make one (a new world in the place `--world`, default the countryside, with the
 * options `--options town=city,villages=many`, the place's own where not given; its neighbours as the
 * neighbourhood step makes them; a household of four in the first home for sale), played at full
 * speed to `--hour` (5 minutes before it) on day 1, paused and saved.
 *
 *   bun tools/bench/scene.mjs --name village [--world country|forest|mountain|…] [--options town=village] [--hour 18] [--port 5221]
 *
 * `--seed <n>`: the new world's seed (the same place, options and seed give the same town, neighbours and household).
 * `--home town`: the household moves into a home for sale in the main town (the one
 * halfway out among those for sale there) instead of the first for sale anywhere. `--home flat`: into a
 * flat for sale (the main town's nearest its middle, upstairs if one is), so the game starts in a flat. The town view's scenes
 * (`render.mjs --poses town`: `town-day`, `town-dusk`, `town-night`) are made so, one town at three hours:
 *
 *   bun tools/bench/scene.mjs --name town-day --world country --options town=village --seed 7 --home town --hour 13
 *   bun tools/bench/scene.mjs --name town-dusk --world country --options town=village --seed 7 --home town --hour 19.5
 *   bun tools/bench/scene.mjs --name town-night --world country --options town=village --seed 7 --home town --hour 23
 *
 * (`--world small`, as those scenes were first made, is the same world: the countryside as a village.)
 *
 * Starts its own Vite on `--port` (or uses `--url`). Needs Playwright like render.mjs, and runs
 * headless on the real GPU like it (`--headed` for a window).
 */

import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { gzipSync } from 'node:zlib';
import { launch, serve, WEB } from './lib/browser.mjs';
import { quiet } from './quiet.mjs';
import { worldFlags } from './worldFlags.mjs';

const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined ? args[i + 1] : fallback;
};
const name = flag('name', 'village');
const { world, options } = worldFlags(flag);
const seed = flag('seed', null) === null ? null : Number(flag('seed', null));
const homeIn = flag('home', 'first');
const goal = Number(flag('hour', '18')) * 60 - 5;
const port = Number(flag('port', '5221'));

// (Its own Vite, always: not a server another tree may have left on the port.)
const { url: base, stop } = flag('url', null) ? { url: flag('url', null), stop: () => {} } : await serve({ port });

const { browser } = await launch({ headless: !process.argv.includes('--headed') });
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  await page.addInitScript(quiet);
  page.on('pageerror', (e) => console.error(`page error: ${e.message}`));
  await page.goto(`${base}/?debug&seed=7`);
  // (Dev server modules, imported by their paths: the same instances the game uses.)
  await page.waitForFunction(async () => !(await import('/src/ui/app.svelte.ts')).app.booting, null, { timeout: 120_000, polling: 500 });
  const started = await page.evaluate(
    async ({ kind, seed, homeIn, options }) => {
      const { app } = await import('/src/ui/app.svelte.ts');
      const { services } = await import('/src/ui/services.ts');
      const { generateWorld, loadTemplates } = await import('/src/game/town.ts');
      const { sceneHome, sceneTown } = await import('/src/debug/sceneRecipe.ts');
      const t = await loadTemplates();
      const empty = generateWorld(services.content, t, kind, seed ?? undefined, undefined, undefined, options);
      const { town, household } = sceneTown(services.content, services.assets, t, empty, seed);
      const { home, ref } = sceneHome(services.content, t, town, homeIn);
      Object.assign(app, { world: empty, town, household, home: ref });
      // (The rest of the homes for sale are filled on the way in: from the seed too, when there is one.)
      const request = app.homeRequest(ref);
      app.start(seed === null ? request : { ...request, peopleSeed: seed });
      return { world: town.world?.kind, seed: town.seed, home: home.name, storey: home.storey, households: town.households.length, neighbours: town.households.reduce((n, h) => n + h.household.members.length, 0) };
    },
    { kind: world, seed, homeIn, options },
  );
  console.log(`scene: started`, started);
  await page.waitForFunction(() => !!window.__osw && !!document.querySelector('.hud-root'), null, { timeout: 120_000, polling: 500 });
  await page.waitForTimeout(2000);
  const minute = () => page.evaluate(() => window.__osw.snapshotMinute());
  // Fast to just before the hour, then at normal speed to it.
  for (const [speed, until] of [[3, goal - 40], [1, goal]]) {
    await page.evaluate((s) => window.__osw.send({ type: 'setSpeed', speed: s }), speed);
    for (let i = 0; i < 1200 && (await minute()) < until; i++) await page.waitForTimeout(150);
  }
  await page.evaluate(() => window.__osw.send({ type: 'setSpeed', speed: 0 }));
  const reached = await minute();
  if (reached < goal) throw new Error(`the clock stopped at minute ${reached.toFixed(0)}, not ${goal}: no scene saved`);
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
  stop();
}
