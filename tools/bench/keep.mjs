#!/usr/bin/env bun
/**
 * Keeps bench pages loaded: one browser (headless, its profile kept, a DevTools port open) with a bench page per build,
 * the scene loaded and settled, drawing nothing until a script uses it. Bench scripts attach to it
 * (`render.mjs --attach`, or `lib/browser.mjs` `attach` + `openBench(…, { attached: true })`) and measure in the
 * pages as they are, through `window.__bench` (pose, hour, freeze, settle, counts), each page reset when they are done:
 * a probe or an A/B run pays the game's load once per build per session instead of once per run.
 *
 *   bun tools/bench/keep.mjs [--build <tree|url>]… [--prod] [--scene town-day] [--preset medium] [--backend webgl]
 *                            [--query 'cam=over&over=centre&radius=400&hour=13'] [--port 9333] [--headed]
 *
 * `--build`: a tree (served by its own Vite on a free port from 5300, or with `--prod` its bench build, `lib/build.mjs`,
 * under `vite preview`) or a running server's URL; several for an A/B (this tree when none is given). Writes the
 * endpoint and the builds' URLs to `bench-results/.keep.json` (`--attach keep` reads it) and runs until Ctrl-C.
 *
 * Reload a kept page (stop and start keep.mjs, or `page.reload()`) when its code changed (a dev server's page may
 * reload on its own as files change: wait for `__benchReady` again), and never measure in a kept page what needs a
 * fresh start: load and entry times, memory (`gameplay.mjs`, `memory-ab.mjs`), the first frames' shader compiles.
 */

import { existsSync, rmSync, writeFileSync } from 'node:fs';
import { basename, join, resolve } from 'node:path';
import { freePort, launch, openBench, ROOT, serve } from './lib/browser.mjs';
import { benchBuild } from './lib/build.mjs';

const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};
const all = (name) => args.flatMap((a, i) => (a === `--${name}` && args[i + 1] && !args[i + 1].startsWith('--') ? [args[i + 1]] : []));
const builds = all('build').length ? all('build') : [ROOT];
const prod = args.includes('--prod');
const scene = flag('scene', 'town-day');
const preset = flag('preset', 'medium');
const backend = flag('backend', 'webgl');
const debugPort = Number(flag('port', '9333'));
const query = [`bench=${scene}`, `quality=${preset}`, backend === 'webgl' ? 'renderer=webgl' : '', 'res=1', 'diag=freeze', 'hud=0', 'secs=0', 'warmup=6', flag('query', '')].filter(Boolean).join('&');
const INFO = join(ROOT, 'bench-results', '.keep.json');

if ((await freePort(debugPort, debugPort).catch(() => null)) !== debugPort) throw new Error(`port ${debugPort} is taken (--port)`);
const stops = [];
const served = [];
for (const [i, where] of builds.entries()) {
  const name = builds.length > 1 ? String.fromCharCode(97 + i) : 'a';
  if (/^https?:\/\//.test(where)) {
    served.push({ name, where, url: where.replace(/\/$/, '') });
    continue;
  }
  const tree = resolve(where);
  const web = join(tree, 'web');
  if (!existsSync(join(web, 'node_modules/.bin/vite'))) throw new Error(`${tree}: no web/node_modules (link the main checkout's)`);
  const port = await freePort(5300, 5399);
  // (Builds one after another: each wants every core.)
  const outDir = prod ? (await benchBuild({ tree })).dir : null;
  const { url, stop } = await serve({ mode: prod ? 'preview' : 'dev', port, web, outDir });
  stops.push(stop);
  served.push({ name, where: tree, url });
}

const { context } = await launch({ headless: !args.includes('--headed'), profileDir: join(ROOT, 'bench-results', '.browser', `keep-${debugPort}`), debugPort });
const quit = async () => {
  rmSync(INFO, { force: true });
  await context.close().catch(() => {});
  for (const stop of stops) stop();
  process.exit(0);
};
process.on('SIGINT', quit);
process.on('SIGTERM', quit);

for (const b of served) {
  const started = performance.now();
  // (One build at a time: two loading side by side would time each other.)
  const { page } = await openBench(context, b.url, query);
  await page.evaluate(() => window.__bench.idle(true));
  b.page = page.url();
  b.loadSecs = +((performance.now() - started) / 1000).toFixed(1);
  console.log(`${b.name}: ${basename(b.where)} at ${b.url}, loaded in ${b.loadSecs} s`);
}
writeFileSync(INFO, JSON.stringify({ endpoint: `http://127.0.0.1:${debugPort}`, query, builds: served }, null, 2));
console.log(`kept: ${INFO} (endpoint http://127.0.0.1:${debugPort}); Ctrl-C to stop`);
await new Promise(() => {});
