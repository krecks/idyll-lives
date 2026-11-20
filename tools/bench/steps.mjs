#!/usr/bin/env bun
/**
 * Draw counts in fixed steps (\`web/src/debug/benchTime.ts\`, through \`window.__bench\`): a game state loaded into one page, the camera held at each
 * pose, and exactly \`--frames\` frames drawn \`--ticks\` ticks apart, each run starting from the state as saved. The
 * counts depend on the state, the frames and the camera only, not on the machine's load: two runs of the same give
 * the same numbers (\`hash\`), so one run a pose is enough, and two builds compare frame for frame.
 *
 *   bun tools/bench/steps.mjs --state town-day --poses centre:24,centre:60,home:24 [--frames 600] [--ticks 1]
 *     [--backends webgl,webgpu] [--runs 1] [--quality medium] [--port <p> | --url http://127.0.0.1:5317] [--out dir]
 *
 * \`--state\`: a state by name (\`debug-states/<name>.json\`, else \`web/public/bench/<name>.json.gz\`: \`state.mjs --name\`).
 * A pose is \`home\` or \`centre\` and the camera's distance (m), from the game's opening angle. \`--runs 2\` runs each pose
 * twice, the state loaded again in the same page between: what proves the counts repeat. Prints a line a pose and
 * backend (draws a frame: mean, least, most; the sun's still picture drawn again; the fingerprint) and writes every
 * frame's counts to \`bench-results/steps-<time>/\` (or \`--out\`), with the page's load times (\`__bench.load\`).
 */

import { mkdirSync, writeFileSync } from 'node:fs';
import { quiet } from './quiet.mjs';
import { launch, ROOT, serve, VIEWPORT } from './lib/browser.mjs';

const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf('--' + name);
  return i >= 0 && args[i + 1] !== undefined ? args[i + 1] : fallback;
};
const state = flag('state', 'town-day');
const frames = Number(flag('frames', '600'));
const ticks = Number(flag('ticks', '1'));
const runs = Number(flag('runs', '1'));
const quality = flag('quality', 'medium');
const backends = flag('backends', 'webgl,webgpu').split(',');
const poses = flag('poses', 'centre:24,centre:60,home:24').split(',').map((p) => {
  const [over, radius] = p.split(':');
  if (!['home', 'centre'].includes(over) || !(Number(radius) > 0)) throw new Error('steps: a pose is home:<m> or centre:<m>, not ' + p);
  return { over, radius: Number(radius) };
});
const port = flag('port', null);
const out = flag('out', ROOT + 'bench-results/steps-' + new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19));

// (Its own Vite unless `--url` names one.)
const vite = flag('url', null) ? null : await serve({ port: port === null ? undefined : Number(port) });
const base = flag('url', null) ?? vite.url;
mkdirSync(out, { recursive: true });
const results = [];
try {
  for (const backend of backends) {
    const { browser } = await launch({ headless: !args.includes('--headed') });
    try {
      const page = await browser.newPage(VIEWPORT);
      await page.addInitScript(quiet);
      page.on('pageerror', (e) => console.error('page error: ' + e.message));
      const t0 = Date.now();
      await page.goto(base + '/?debug&hud=0&quality=' + quality + (backend === 'webgl' ? '&renderer=webgl' : ''));
      await page.waitForFunction(() => !!window.__bench, null, { timeout: 120_000, polling: 250 });
      // (The page to its menu: what a fresh page load costs before any game, against `load`'s swap in an open page.)
      const bootMs = Date.now() - t0;
      console.error('steps: ' + backend + ' page up in ' + (bootMs / 1000).toFixed(1) + ' s');
      results.push({ backend, bootMs });
      for (const pose of poses) {
        // A warm-up first, not counted: what the frames ask for the first time in a page (a prop, a model, a face)
        // is built then, so every counted run draws it from its first frame.
        if (!args.includes('--cold')) {
          await page.evaluate((s) => window.__bench.load(s), state);
          await page.evaluate(([n, t, p]) => window.__bench.run(n, { ticks: t, pose: p }), [frames, ticks, pose]);
        }
        for (let run = 0; run < runs; run++) {
          const loaded = await page.evaluate((s) => window.__bench.load(s), state);
          const r = await page.evaluate(([n, t, p]) => window.__bench.run(n, { ticks: t, pose: p }), [frames, ticks, pose]);
          r.minute = await page.evaluate(() => window.__osw?.snapshotMinute?.() ?? null);
          const line = { backend, pose: pose.over + ':' + pose.radius, run, loaded, frames: r.frames, ticks: r.ticks, settleFrames: r.settleFrames, drawsMean: +r.drawsMean.toFixed(1), drawsMin: r.drawsMin, drawsMax: r.drawsMax, shadowMean: +(r.shadowDraws.reduce((a, n) => a + n, 0) / r.frames).toFixed(1), sunStill: r.sunShadows.whole, sunMoving: r.sunShadows.moving, why: r.sunShadows.why, hash: r.hash, building: r.building, ms: Math.round(r.ms) };
          results.push(line);
          writeFileSync(out + '/' + backend + '-' + pose.over + pose.radius + '-' + run + '.json', JSON.stringify(r));
          // (The run's last frame as drawn: two runs of the same give the same picture.)
          if (args.includes('--shots')) await page.screenshot({ path: out + '/' + backend + '-' + pose.over + pose.radius + '-' + run + '.png' });
          console.log('steps: ' + backend + ' ' + line.pose + ' run ' + run + ': draws ' + line.drawsMean + ' (' + r.drawsMin + '–' + r.drawsMax + '), shadow ' + line.shadowMean + ', late builds ' + r.lateBuilds.length + ', still picture ' + r.sunShadows.whole + ' ' + JSON.stringify(r.sunShadows.why) + ', hash ' + r.hash + ', minute ' + r.minute + ', ' + r.frames + ' frames in ' + (r.ms / 1000).toFixed(1) + ' s, load ' + (loaded.upMs / 1000).toFixed(1) + '/' + (loaded.settledMs / 1000).toFixed(1) + ' s');
        }
      }
      console.error('steps: ' + backend + ' done in ' + ((Date.now() - t0) / 1000).toFixed(0) + ' s');
    } finally {
      // (Not waited for long: a headless Chromium sometimes never says it has closed.)
      await Promise.race([browser.close(), new Promise((resolve) => setTimeout(resolve, 10_000))]);
    }
  }
} finally {
  writeFileSync(out + '/summary.json', JSON.stringify({ state, frames, ticks, runs, quality, results }, null, 2));
  vite?.stop();
}
console.log('steps: ' + out);
// (A browser that never said it closed would keep the process.)
process.exit(0);
