#!/usr/bin/env node
/**
 * The language model benchmark (`web/src/llm/bench.ts`): starts Vite, opens `llm-bench.html` in
 * Chromium with WebGPU and runs the game's own worker and prompts with no game around it, once
 * cold (nothing in the Cache API) and once with the page reloaded (the model comes from the
 * browser cache), and prints load time, prompt reading (cold and cached), writing speed
 * (tokens a second) and the game's five test lines end to end. Writes every report as JSON under
 * `bench-results/llm-<time>/`.
 *
 * Needs `pnpm install` and `pnpm llm` first, and Playwright: `PLAYWRIGHT=<path to
 * playwright/index.mjs>` if it isn't installed, `CHROME=<executable>` for a browser of your own.
 * Usage (from the repo root or `web/`):
 *
 *   node tools/llm/bench.mjs [options]
 *     --port 5399        Vite's port (it starts its own; `--url http://localhost:5399` uses a running one)
 *     --tokens 96        tokens a decode run may write
 *     --runs 3           decode runs (median, min, max)
 *     --no-lines         skip the game's five test lines
 *     --cold / --warm    only the first load / only the cached reload (default: both)
 *     --headed / --headless  with a window (the default: a timing run, like `devices.mjs`) or
 *                        without (still the real GPU)
 *     --left 2600        how far right the window goes, out of the way
 *     --out <dir>        default <repo>/bench-results/llm-<time>/
 */

import { execFileSync, spawn } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import os from 'node:os';
import { fileURLToPath } from 'node:url';

const WEB = fileURLToPath(new URL('../../web/', import.meta.url));
const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const args = process.argv.slice(2);
const flag = (name, fallback = undefined) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 ? (args[i + 1] && !args[i + 1].startsWith('--') ? args[i + 1] : true) : fallback;
};
const has = (name) => args.includes(`--${name}`);
const PORT = Number(flag('port', 5399));
const TOKENS = Number(flag('tokens', 96));
const RUNS = Number(flag('runs', 3));
const HEADLESS = has('headless') && !has('headed');
const LEFT = Number(flag('left', 2600));
const COLD = !has('warm');
const WARM = !has('cold');
const LINES = !has('no-lines');
const OUT = String(flag('out', `${ROOT}bench-results/llm-${new Date().toISOString().replace(/[:.]/g, '-')}/`));

async function playwright() {
  if (process.env.PLAYWRIGHT) return import(process.env.PLAYWRIGHT);
  try {
    return await import(createRequire(WEB + 'package.json').resolve('playwright'));
  } catch {
    throw new Error('Playwright not found: install it, or set PLAYWRIGHT=<path to playwright/index.mjs>');
  }
}

/** A running Vite, or the one the caller started (`--url`). */
async function startVite() {
  if (flag('url')) return { url: String(flag('url')), stop: () => {} };
  const vite = spawn(`${WEB}node_modules/.bin/vite`, ['--port', String(PORT), '--strictPort'], { cwd: WEB, stdio: ['ignore', 'pipe', 'inherit'] });
  const stop = () => vite.kill();
  process.on('exit', stop);
  await new Promise((resolve, reject) => {
    vite.stdout.on('data', (d) => String(d).includes('Local') && resolve());
    vite.on('exit', (code) => reject(new Error(`vite exited (${code})`)));
  });
  return { url: `http://localhost:${PORT}`, stop };
}

const fmt = (n, digits = 0) => (Number.isFinite(n) ? n.toFixed(digits) : '—');

/** The machine the benchmark runs on: model, OS, and on macOS power and thermal state (Low Power Mode throttles). */
function machine() {
  const base = { os: `${os.type()} ${os.release()}`, chip: os.cpus()[0]?.model ?? '?', cores: os.cpus().length, memoryGb: os.totalmem() / 1073741824 };
  if (process.platform !== 'darwin') return { ...base, power: '?', lowPowerMode: '?', thermal: '' };
  const run = (cmd, args) => {
    try {
      return execFileSync(cmd, args, { encoding: 'utf8' }).trim();
    } catch {
      return '';
    }
  };
  const pmset = run('pmset', ['-g']);
  return {
    ...base,
    os: `macOS ${run('sw_vers', ['-productVersion'])}`,
    power: run('pmset', ['-g', 'batt']).split('\n').slice(1).join(' ').trim(),
    lowPowerMode: /lowpowermode\s+(\d)/.exec(pmset)?.[1] ?? '?',
    thermal: run('pmset', ['-g', 'therm']).split('\n').filter((l) => /warning|CPU_Scheduler_Limit|CPU_Available_CPUs/.test(l)).join('; '),
  };
}
const table = (report) => {
  const cold = report.lines.length ? report.cold : report.decode.runs[0];
  const cached = report.lines.length ? report.cached : report.decode.runs[0];
  return [
    {
      adapter: [report.adapter.vendor, report.adapter.architecture, report.adapter.device].filter(Boolean).join(' ') || 'none',
      isolated: report.isolated ? 'yes' : 'no',
      'load s': fmt(report.loadMs / 1000, 1),
      'warm-up ms': fmt(report.warmupMs),
      'prefill ms (cold/cached)': report.lines.length ? `${fmt(report.cold.prefillMs)} / ${fmt(report.cached.prefillMs)}` : fmt(report.decode.prefillMs),
      'cached share': report.cached.promptTokens ? `${fmt((report.cached.cachedTokens / report.cached.promptTokens) * 100)}%` : '—',
      'tok/s (med/min/max)': `${fmt(report.decode.tokPerS, 1)} / ${fmt(report.decode.minTokPerS, 1)} / ${fmt(report.decode.maxTokPerS, 1)}`,
      'line ms': report.lines.length ? fmt(report.lines[report.lines.length - 1].totalMs) : '—',
      lines: report.lines.length,
    },
  ];
};

async function runPage(page, url, options) {
  await page.goto(`${url}/llm-bench.html`);
  await page.waitForFunction(() => typeof window.__llmPerf === 'function', null, { timeout: 120_000 });
  return page.evaluate((o) => window.__llmPerf(o), options);
}

const state = machine();
console.log(`llm bench: ${state.chip}, ${state.cores} cores, ${state.memoryGb.toFixed(0)} GB, ${state.os}; Low Power Mode ${state.lowPowerMode}; ${state.power}`);

const vite = await startVite();
const { chromium } = await playwright();
const browser = await chromium.launch({
  headless: HEADLESS,
  ...(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chromium' }),
  args: ['--enable-unsafe-webgpu', '--enable-features=Vulkan,WebGPU', '--use-angle=metal', '--ignore-gpu-blocklist'],
});
let failed = false;
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  if (!HEADLESS) {
    try {
      const cdp = await page.context().newCDPSession(page);
      const { windowId } = await cdp.send('Browser.getWindowForTarget');
      await cdp.send('Browser.setWindowBounds', { windowId, bounds: { left: LEFT, top: 0 } });
    } catch {
      // (Some window managers refuse; it still runs.)
    }
  }
  page.on('console', (m) => m.type() === 'error' && console.error(`page: ${m.text()}`));
  page.on('pageerror', (e) => console.error(`page error: ${e.message}`));
  const options = { decodeTokens: TOKENS, decodeRuns: RUNS, lines: LINES };
  const reports = {};
  if (COLD) {
    console.log('llm bench: cold (nothing cached)');
    reports.cold = await runPage(page, vite.url, options);
    console.table(table(reports.cold));
  }
  if (WARM) {
    console.log('llm bench: warm (page reloaded, model from the browser cache)');
    reports.warm = await runPage(page, vite.url, options);
    console.table(table(reports.warm));
  }
  const report = reports.warm ?? reports.cold;
  if (report) {
    for (const r of [reports.cold, reports.warm].filter(Boolean)) {
      console.log(`${r === reports.cold ? 'cold' : 'warm'} load progress: ${r.loadProgress.map((p) => `${fmt(p.atMs)}ms ${fmt((p.loaded / p.total) * 100)}%`).slice(-4).join(', ')}`);
      for (const line of r.lines) console.log(`  ${line.name}: ${fmt(line.totalMs)} ms, ${fmt(line.tokPerS, 1)} tok/s, ${line.tokens} tokens, stop ${line.stop} — ${JSON.stringify(line.text)}`);
      for (const d of r.decode.runs) console.log(`  ${d.name}: ${fmt(d.totalMs)} ms, ${fmt(d.tokPerS, 1)} tok/s, ${d.tokens} tokens, stop ${d.stop}`);
    }
    mkdirSync(OUT, { recursive: true });
    writeFileSync(`${OUT}report.json`, JSON.stringify({ startedAt: new Date().toISOString(), machine: state, options, ...reports }, null, 2));
    console.log(`llm bench: wrote ${OUT}report.json`);
  }
} catch (err) {
  console.error(`llm bench FAILED: ${err instanceof Error ? err.message : err}`);
  failed = true;
} finally {
  await browser.close();
  vite.stop();
}
process.exit(failed ? 1 : 0);
