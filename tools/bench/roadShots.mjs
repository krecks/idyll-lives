#!/usr/bin/env bun
/**
 * Pictures of the road hierarchy's fixture (`?bench=bigworld&big=roads`: `web/src/debug/roadFixture.ts`): the motorway
 * over the street (its overpass, barrier and markings), a ramp merging, the motorway's bridge, the avenue's junctions
 * and median, the one-way loop, each from about 40 m; the whole of it from 400 m and from 783 m. Each picture's draws
 * (`__bench.counts`) are printed and written to `counts.json` beside the pictures. On each renderer.
 *
 *   bun tools/bench/roadShots.mjs [--renderers webgpu,webgl] [--quality medium] [--port 5311 | --url <u>] [--out <dir>]
 *     [--only overpass,overview-400] [--follow]
 *
 * `--follow`: the fixture on a land the town follows (`&follow`: its built surface gives the decks' heights and the ground
 * under them); without it, a land drawn flat, where the decks climb on their own (`motorway.ts` `deckLevels`).
 *
 * Needs Playwright like render.mjs (`PLAYWRIGHT=<path to playwright/index.mjs>`).
 */
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { benchContext, devServer, launch, openBench } from './lib/browser.mjs';

const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};
const renderers = flag('renderers', 'webgpu,webgl').split(',');
const quality = flag('quality', 'medium');
const out = flag('out', 'bench-results/road-shots');
const only = flag('only', null)?.split(',') ?? null;
const follow = args.includes('--follow') ? '&follow' : '';
mkdirSync(out, { recursive: true });

// (Over the fixture's middle, `tx`/`tz` m off it: `roadFixture.ts`. `beta`: from straight down.)
const near = { cam: 'over', over: 'centre', radius: 40, beta: 1.05 };
const POSES = [
  { name: 'overpass', pose: { ...near, tx: -100, tz: 30, alpha: -2.2, beta: 1.2 } },
  { name: 'overpass-under', pose: { ...near, tx: -100, tz: 60, radius: 30, alpha: -1.0, beta: 1.32 } },
  { name: 'merge', pose: { ...near, tx: -190, tz: 40, alpha: -2.6 } },
  { name: 'motorway', pose: { ...near, tx: 0, tz: 44, alpha: -1.8 } },
  { name: 'bridge', pose: { ...near, tx: 99, tz: 30, alpha: -2.4, beta: 1.15 } },
  { name: 'avenue', pose: { ...near, tx: -150, tz: -80, alpha: -2.0 } },
  { name: 'avenue-mid', pose: { ...near, tx: -90, tz: -80, alpha: -1.4 } },
  { name: 'loop', pose: { ...near, tx: 60, tz: -100, alpha: -2.2 } },
  { name: 'ramp-street', pose: { ...near, tx: -100, tz: 15, alpha: -2.6 } },
  { name: 'overview-400', pose: { cam: 'over', over: 'centre', radius: 400 } },
  { name: 'overview-783', pose: { cam: 'over', over: 'centre', radius: 783 } },
].filter((p) => !only || only.includes(p.name));

const { url: base, stop } = await devServer({ url: flag('url', null), port: Number(flag('port', '5311')) });
const { browser } = await launch({ headless: true });
const counts = {};
try {
  for (const renderer of renderers) {
    const context = await benchContext(browser);
    const { page, done } = await openBench(context, base, `bench=bigworld&big=roads${follow}&quality=${quality}&renderer=${renderer}&res=1&hud=0&secs=0&warmup=6&diag=freeze&cam=over&over=centre&radius=400&hour=13`);
    for (const p of POSES) {
      await page.evaluate((x) => window.__bench.pose(x), p.pose);
      await page.evaluate(() => window.__bench.hour(13));
      await page.evaluate(() => window.__bench.settle({ limitMs: 90_000 }));
      const file = join(out, `${p.name}-${renderer}${follow ? '-follow' : ''}.png`);
      await page.screenshot({ path: file });
      const c = await page.evaluate(() => window.__bench.counts(30));
      counts[`${p.name}-${renderer}${follow ? '-follow' : ''}`] = c;
      console.log(`${p.name} ${renderer}: ${file}, draws ${c.mean?.draws ?? JSON.stringify(c).slice(0, 160)}`);
    }
    await done();
    await context.close();
  }
} finally {
  writeFileSync(join(out, `counts${follow ? '-follow' : ''}.json`), JSON.stringify(counts, null, 1));
  await browser.close();
  stop();
}
