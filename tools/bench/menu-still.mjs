#!/usr/bin/env bun
/**
 * Takes the main menu's picture shipped with the game (`web/public/menu/still.jpg`, what a first visit's
 * menu shows: `game/menuStill.ts` `FIRST_STILL`): a world made as step 1 makes one (`--world`, `--options`, `--seed`),
 * built behind the menu and taken as the menu takes its own (`menuScene.keepStill`: the menu's shot, at
 * golden hour), once every model and texture has come. Prints the town's name: `FIRST_STILL.name` is it.
 *
 *   bun tools/bench/menu-still.mjs [--world country] [--options town=city,villages=many] [--seed 2098028765] [--width 1920 --height 1080] [--port 5223]
 *
 * `--world`: a place (default the countryside, as a new player's step 1 makes it), `--options` the
 * player's options (the place's own where not given; `worldFlags.mjs`).
 *
 * Starts its own Vite on `--port` (or uses `--url`). Needs Playwright like render.mjs (`PLAYWRIGHT`), and
 * runs headless on the real GPU like it (`--headed` for a window).
 */

import { spawn } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { quiet } from './quiet.mjs';
import { worldFlags } from './worldFlags.mjs';

const WEB = fileURLToPath(new URL('../../web/', import.meta.url));
const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined ? args[i + 1] : fallback;
};
const { world, options } = worldFlags(flag);
const seed = Number(flag('seed', '2098028765'));
const width = Number(flag('width', '1920'));
const height = Number(flag('height', '1080'));
const port = Number(flag('port', '5223'));
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
  const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: 1 });
  await page.addInitScript(quiet);
  page.on('pageerror', (e) => console.error(`page error: ${e.message}`));
  await page.goto(`${base}/?debug`);
  // (Dev server modules, imported by their paths: the same instances the game uses.)
  await page.waitForFunction(async () => !(await import('/src/ui/app.svelte.ts')).app.booting, null, { timeout: 120_000, polling: 500 });
  // Every model and texture there, so none is drawn as its stand-in (the stream indicator goes once all have come).
  await page.waitForTimeout(3000);
  await page.waitForFunction(() => !document.querySelector('.stream'), null, { timeout: 300_000, polling: 1000 }).catch(() => console.warn('menu-still: the game data still streams; taken anyway'));
  const taken = await page.evaluate(
    async ({ kind, seed, options }) => {
      const { menuScene } = await import('/src/game/menuScene.ts');
      const { townJob, keepHouses } = await import('/src/game/townJobs.ts');
      const { services } = await import('/src/ui/services.ts');
      const made = await townJob(services.content, { kind: 'world', world: kind, seed, options });
      keepHouses(made);
      menuScene.prepare(made.draft, made.preview);
      menuScene.shoot({ kind: 'menu' }, 0);
      await menuScene.show(made.draft);
      // (Its lawns, trees and shadows settled.)
      await new Promise((resolve) => setTimeout(resolve, 4000));
      const still = await menuScene.keepStill(true);
      if (!still) throw new Error('no picture');
      const bytes = new Uint8Array(await (await fetch(still.url)).arrayBuffer());
      let binary = '';
      for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
      return { name: still.name, data: btoa(binary) };
    },
    { kind: world, seed, options },
  );
  const dir = join(WEB, 'public/menu');
  mkdirSync(dir, { recursive: true });
  const file = join(dir, 'still.jpg');
  const data = Buffer.from(taken.data, 'base64');
  writeFileSync(file, data);
  console.log(`menu-still: ${file}, ${(data.length / 1024).toFixed(0)} KB, the town ${JSON.stringify(taken.name)} (FIRST_STILL.name)`);
} finally {
  await browser.close();
  vite?.kill();
}
