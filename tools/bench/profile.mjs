#!/usr/bin/env bun
/**
 * Profiles the game itself (not the fixed-camera render bench): opens `?state=<name>&profile`,
 * plays for `--secs` seconds at a game speed, and collects `window.__perfReport()` (spans, long
 * tasks, frames the display dropped, each named by the phase it happened in) plus its own frame
 * gaps so the hitches below the 50 ms the game records are kept too.
 *
 *   bun tools/bench/profile.mjs [options]
 *     --state family-check     a state in debug-states/ (dev server; `--url` for one already running)
 *     --speed 3                game speed (0 paused, 1..3, 4 super speed in quiet hours only)
 *     --secs 60 --warmup 5     measured after the warm-up
 *     --preset medium          graphics preset (low, medium, high)
 *     --renderer webgl         WebGL2 instead of WebGPU
 *     --hud 1                  the interface shown
 *     --out <dir>              default bench-results/profile-<time>/
 */
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { benchContext, devServer, launch, ROOT } from './lib/browser.mjs';

const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};
const has = (name) => args.includes(`--${name}`);

const state = flag('state', 'family-check');
const speed = Number(flag('speed', '3'));
const secs = Number(flag('secs', '60'));
const warmup = Number(flag('warmup', '5'));
const preset = flag('preset', 'medium');
const renderer = flag('renderer', 'webgpu');
const hud = flag('hud', '1');
const port = Number(flag('port', '5222'));
const headed = !has('headless');
const stamp = new Date().toISOString().replace(/\.\d+Z$/, '').replace(/[:T]/g, '-');
const out = flag('out', join(ROOT, 'bench-results', `profile-${stamp}`));
mkdirSync(out, { recursive: true });

// (Dev server only: `?state=` opens nothing in a production build.)
const { url: base, stop } = await devServer({ url: flag('url', null), port });

const { browser } = await launch({ headless: !headed });
const context = await benchContext(browser);
const page = await context.newPage();
const cdp = await context.newCDPSession(page);
if (headed) {
  try {
    const { windowId } = await cdp.send('Browser.getWindowForTarget');
    await cdp.send('Browser.setWindowBounds', { windowId, bounds: { left: 2600, top: 0 } });
  } catch {}
}
page.on('pageerror', (e) => console.error(`  page error: ${e.message}`));
page.on('console', (m) => m.type() === 'error' && console.error(`  page: ${m.text().slice(0, 300)}`));

const query = [`state=${state}`, 'profile', 'debug', `quality=${preset}`, renderer === 'webgl' ? 'renderer=webgl' : '', hud === '0' ? 'hud=0' : '']
  .filter(Boolean)
  .join('&');
console.log(`opening ${base}/?${query} …`);
await page.goto(`${base}/?${query}`);

// The game screen is up once the interface is there and the loading screen is gone.
await page.waitForFunction(() => document.querySelector('.hud-root') && !document.querySelector('.loading'), { timeout: 180_000, polling: 250 });
console.log('game up; warm-up');
await page.waitForTimeout(warmup * 1000);
await page.keyboard.press(String(speed));
await page.evaluate(() => window.__perfReport && (window.__profileStart = performance.now()));

// Its own frame-gap recorder: every gap, so hitches under the game's 50 ms threshold are kept too.
const gaps = await page.evaluate(async (ms) => {
  const list = [];
  let last = 0;
  await new Promise((resolve) => {
    const t0 = performance.now();
    const raf = (t) => {
      if (last) list.push({ at: t - t0, gap: t - last });
      last = t;
      if (performance.now() - t0 >= ms) resolve();
      else requestAnimationFrame(raf);
    };
    requestAnimationFrame(raf);
  });
  return list;
}, secs * 1000);

const report = await page.evaluate(() => window.__perfReport());
const stamp2 = new Date().toISOString();
writeFileSync(join(out, 'frames.json'), JSON.stringify({ query, state, speed, preset, renderer, secs, gaps }, null, 2));
writeFileSync(join(out, 'perf.json'), JSON.stringify({ query, state, speed, preset, renderer, secs, stamp: stamp2, report }, null, 2));

const sorted = gaps.map((g) => g.gap).sort((a, b) => a - b);
const at = (q) => sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))] ?? 0;
const over = (ms) => gaps.filter((g) => g.gap >= ms).length;
const drops = report.droppedFrames.filter((d) => d.at >= 0);
const phases = {};
for (const d of drops) phases[d.phase] = (phases[d.phase] ?? 0) + 1;
const tasks = report.longTasks;
const taskPhases = {};
for (const t of tasks) taskPhases[t.phase] = (taskPhases[t.phase] ?? 0) + 1;
const worst = [...gaps].sort((a, b) => b.gap - a.gap).slice(0, 15);

const md = [
  `# Profile: ${state}, speed ${speed}, ${preset}/${renderer}, ${secs} s`,
  '',
  `${gaps.length} frames · ${(gaps.length / secs).toFixed(1)} fps · gap p50 ${at(0.5).toFixed(1)} / p95 ${at(0.95).toFixed(1)} / p99 ${at(0.99).toFixed(1)} / max ${at(1).toFixed(1)} ms`,
  '',
  `Frames over: 20 ms ${over(20)} · 33 ms ${over(33)} · 50 ms ${over(50)} · 100 ms ${over(100)}`,
  '',
  `Long tasks (main thread): ${tasks.length}, total ${tasks.reduce((a, t) => a + t.ms, 0).toFixed(0)} ms, max ${Math.max(0, ...tasks.map((t) => t.ms)).toFixed(0)} ms`,
  '',
  '## Dropped frames by phase (the game\'s ≥50 ms record)',
  ...Object.entries(phases).sort((a, b) => b[1] - a[1]).map(([p, n]) => `- ${p || '(none)'}: ${n}`),
  '',
  '## Long tasks by phase',
  ...Object.entries(taskPhases).sort((a, b) => b[1] - a[1]).map(([p, n]) => `- ${p || '(none)'}: ${n}`),
  '',
  '## Worst frames (own recorder)',
  ...worst.map((g) => `- ${g.gap.toFixed(1)} ms at ${(g.at / 1000).toFixed(2)} s`),
  '',
  '## Slowest spans',
  ...report.slowest.slice(0, 20).map((e) => `- ${e.name}: ${e.ms.toFixed(0)} ms at ${(e.start / 1000).toFixed(2)} s${e.detail ? ` (${e.detail})` : ''}`),
  '',
  '## Every dropped frame',
  ...report.droppedFrames.map((d) => `- ${d.gap.toFixed(0)} ms at ${(d.at / 1000).toFixed(2)} s (${d.phase})`),
  '',
  '## Every long task',
  ...report.longTasks.map((t) => `- ${t.ms.toFixed(0)} ms at ${(t.start / 1000).toFixed(2)} s (${t.phase})`),
  '',
];
writeFileSync(join(out, 'report.md'), md.join('\n'));
console.log(md.slice(0, 40).join('\n'));
console.log(`\nwritten to ${out}`);

await browser.close();
stop();
process.exit(0);
