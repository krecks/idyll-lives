#!/usr/bin/env node
/**
 * The device test (`web/src/talk/devices.svelte.ts`) in a real browser: starts Vite, opens the
 * game with `?debug` in Chromium with WebGPU, runs `__llmBench()` and prints, per setup (A: the
 * language model on the GPU, voices on the CPU; B: both on the GPU; C: both on the CPU, run only
 * if neither GPU setup is usable, or when asked for), load time,
 * prompt and writing speed, time per line, frame times and whether it ran right, and which
 * setup is used (none: the model's lines are switched off). Exits non-zero if a setup that ran
 * wrote garbage (logits that aren't numbers).
 *
 * Needs `pnpm wasm`, `pnpm voice` and `pnpm llm` first, and Playwright:
 * `PLAYWRIGHT=<path to playwright/index.mjs>` if it isn't installed, `CHROME=<executable>` for
 * a browser of your own. `HEADED=1` shows the window (headless Chromium may have no GPU
 * adapter). `MODEL=<id>` tests another model (`web/src/llm/models.ts`, default the default).
 * Usage: `node tools/llm/devices.mjs [A,B,C]`.
 */

import { spawn } from 'node:child_process';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const WEB = fileURLToPath(new URL('../../web/', import.meta.url));
const PORT = 5299;
const only = process.argv[2]?.split(',');

async function playwright() {
  if (process.env.PLAYWRIGHT) return import(process.env.PLAYWRIGHT);
  try {
    return await import(createRequire(WEB + 'package.json').resolve('playwright'));
  } catch {
    throw new Error('Playwright not found: install it, or set PLAYWRIGHT=<path to playwright/index.mjs>');
  }
}

const vite = spawn(`${WEB}node_modules/.bin/vite`, ['--port', String(PORT), '--strictPort'], { cwd: WEB, stdio: ['ignore', 'pipe', 'inherit'] });
const stop = () => vite.kill();
process.on('exit', stop);
await new Promise((resolve, reject) => {
  vite.stdout.on('data', (d) => String(d).includes('Local') && resolve());
  vite.on('exit', (code) => reject(new Error(`vite exited (${code})`)));
});

const { chromium } = await playwright();
const browser = await chromium.launch({
  headless: !process.env.HEADED,
  executablePath: process.env.CHROME || undefined,
  args: ['--enable-unsafe-webgpu', '--enable-features=Vulkan,WebGPU', '--use-angle=metal', '--ignore-gpu-blocklist'],
});
let failed = false;
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  page.on('console', (m) => m.type() === 'error' && console.error(`page: ${m.text()}`));
  if (process.env.MODEL) {
    await page.addInitScript((model) => {
      const key = 'open-sims-wasm.settings';
      localStorage.setItem(key, JSON.stringify({ ...JSON.parse(localStorage.getItem(key) ?? '{}'), llmModel: model }));
    }, process.env.MODEL);
  }
  await page.goto(`http://localhost:${PORT}/?debug`);
  await page.waitForFunction(() => typeof window.__llmBench === 'function', null, { timeout: 60_000 });
  const gpu = await page.evaluate(async () => !!(await navigator.gpu?.requestAdapter()));
  console.log(`devices: WebGPU adapter ${gpu ? 'found' : 'missing (GPU setups are skipped; try HEADED=1)'}`);
  const report = await page.evaluate((o) => window.__llmBench(o ?? undefined), only ?? null);
  const rows = report.results.map((r) => ({
    setup: r.id,
    llm: r.llm,
    voice: r.voice,
    ran: r.ran ? 'yes' : `no: ${r.error}`,
    'load s': (r.loadMs / 1000).toFixed(1),
    'prompt ms': `${r.prefillMs.toFixed(0)} / ${r.prefillCachedMs.toFixed(0)} cached`,
    'tok/s': r.tokPerS.toFixed(1),
    'speech ms': r.speechMs.toFixed(0),
    'line ms': r.lineMs.toFixed(0),
    'frame p50/p95': `${r.frameP50.toFixed(1)} / ${r.frameP95.toFixed(1)}`,
    'clean': `${r.clean}/${r.written}`,
    right: r.ok ? 'yes' : 'no',
    usable: r.usable ? 'yes' : 'no',
  }));
  console.table(rows);
  for (const r of report.results.filter((x) => x.ran)) console.log(`${r.id}: ${r.lines.map((l) => JSON.stringify(l)).join(' · ')}`);
  const pick = report.pick ? `uses ${report.pick.setup} (model on the ${report.pick.llm.toUpperCase()}, voices on the ${report.pick.voice.toUpperCase()})` : "the model's lines are off (no setup fast and right enough)";
  console.log(`devices: ${report.model}; idle frames ${report.idleP50.toFixed(1)} / ${report.idleP95.toFixed(1)} ms; GPU "${report.gpu}"; ${pick}; voices ${report.voicesOk ? 'ok' : 'too slow'}`);
  console.log(JSON.stringify(report));
  for (const r of report.results.filter((x) => x.ran && x.nonFinite)) {
    console.error(`devices FAILED: setup ${r.id} wrote garbage (logits that aren't numbers)`);
    failed = true;
  }
} finally {
  await browser.close();
  stop();
}
process.exit(failed ? 1 : 0);
