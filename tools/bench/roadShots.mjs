#!/usr/bin/env bun
/**
 * Pictures of the road hierarchy's fixture (`?bench=bigworld&big=roads`: `web/src/debug/roadFixture.ts`): the motorway
 * over the street (its overpass, barrier and markings), a ramp merging, the motorway's bridge, the avenue's junctions
 * and median, the one-way loop, each from about 40 m; the whole of it from 400 m and from 783 m. Each picture's draws
 * (`__bench.counts`) are printed and written to `counts.json` beside the pictures. On each renderer.
 *
 *   bun tools/bench/roadShots.mjs [--renderers webgpu,webgl] [--quality medium] [--port 5311 | --url <u>] [--out <dir>]
 *     [--only overpass,overview-400] [--follow] [--plain]
 *
 * `--plain`: every road a plain street on the ground (`&plainRoads`), the draws to count the new kinds' against.
 * `--follow`: the fixture on a land the town follows (`&follow`: its built surface gives the decks' heights and the ground
 * under them); without it, a land drawn flat, where the decks climb on their own (`motorway.ts` `deckLevels`).
 *
 * `--big <place> --options town:city --spots <file>`: a generated Big world's motorway instead (`?bench=bigworld`, its
 * town from seed 7): its junction and the overpass of the road under it from about 40 m, the junction from 250 m and the
 * town from 783 m, where `tools/worlds/map.ts --seed 7 --options town=city --spots <file>` found them (the file's
 * entry for that place).
 *
 * Needs Playwright like render.mjs (`PLAYWRIGHT=<path to playwright/index.mjs>`).
 */
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { benchContext, devServer, launch, openBench, step } from './lib/browser.mjs';

const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};
const renderers = flag('renderers', 'webgpu,webgl').split(',');
const quality = flag('quality', 'medium');
const out = flag('out', 'bench-results/road-shots');
const only = flag('only', null)?.split(',') ?? null;
const follow = (args.includes('--follow') ? '&follow' : '') + (args.includes('--plain') ? '&plainRoads' : '');
mkdirSync(out, { recursive: true });

// (Over the fixture's middle, `tx`/`tz` m off it: `roadFixture.ts`. `beta`: from straight down.)
const near = { cam: 'over', over: 'centre', radius: 40, beta: 1.05 };
const big = flag('big', null);
const bigOptions = flag('options', '');
const spot = big ? JSON.parse(readFileSync(flag('spots', ''), 'utf8')).find((e) => e.kind === big) : null;
if (big && !spot) throw new Error(`no spot for ${big} in the spots file`);
const SPOT_POSES = spot
  ? [
      { name: 'junction', pose: { ...near, ...spot.junction, radius: 55, alpha: -2.2, beta: 1.05 } },
      { name: 'overpass', pose: { ...near, ...spot.overpass, radius: 35, alpha: -1.2, beta: 1.25 } },
      { name: 'overpass-under', pose: { ...near, ...spot.overpass, radius: 22, alpha: 0.4, beta: 1.38 } },
      { name: 'junction-250', pose: { ...near, ...spot.junction, radius: 250, beta: 0.9 } },
      { name: 'overview-783', pose: { cam: 'over', over: 'centre', radius: 783 } },
    ]
  : null;
const POSES = SPOT_POSES ?? [
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
];
// (All at 13:00: the page opens at it.)
const SHOTS = POSES.filter((p) => !only || only.includes(p.name)).map((p) => ({ ...p, hour: 13 }));
const tag = follow ? follow.replaceAll('&', '-') : '';

const { url: base, stop } = await devServer({ url: flag('url', null), port: Number(flag('port', '5311')) });
const { browser } = await launch({ headless: true });
const counts = {};
try {
  for (const renderer of renderers) {
    const context = await benchContext(browser);
    const { page, done } = await openBench(context, base, `bench=bigworld&big=${big ?? 'roads'}${bigOptions ? `&options=${bigOptions}` : ''}${follow}&quality=${quality}&renderer=${renderer}&res=1&hud=0&secs=0&warmup=6&diag=freeze&cam=over&over=centre&radius=400&hour=13`);
    // (Back to 13:00 before each pose: counting lets the clock run.)
    const noon = () => page.evaluate(() => window.__bench.hour(13));
    await step(
      page,
      SHOTS,
      async (page, p) => {
        const file = join(out, `${p.name}-${renderer}${tag}.png`);
        await page.screenshot({ path: file });
        // (Counted at 1x, then frozen again for the next pose: a frozen page draws nothing new to count.)
        await page.evaluate(() => window.__bench.freeze(false));
        const c = await page.evaluate(() => window.__bench.counts(120));
        await page.evaluate(() => window.__bench.freeze(true));
        counts[`${p.name}-${renderer}${tag}`] = c;
        console.log(`${p.name} ${renderer}: ${file}, draws ${c.mean?.draws ?? JSON.stringify(c).slice(0, 160)}`);
        await noon();
      },
      { limitMs: 90_000, before: noon },
    );
    await done();
    await context.close();
  }
} finally {
  writeFileSync(join(out, `counts${tag}.json`), JSON.stringify(counts, null, 1));
  await browser.close();
  stop();
}
