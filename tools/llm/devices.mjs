#!/usr/bin/env bun
/**
 * The device test (`web/src/talk/devices.svelte.ts`) in a real browser: starts Vite, opens the
 * game with `?debug` in Chromium with WebGPU, runs `__llmBench()` and prints (the language model
 * and the voices, both on the GPU) load time, prompt and writing speed, time
 * to write a line, to make its first piece of speech and from asking to hearing it, how fast
 * speech is made against its length, sentences per line, frame times and whether it ran right,
 * and which setup is used (none: the model's lines are switched off). Exits non-zero if a setup that ran
 * wrote garbage (logits that aren't numbers).
 *
 * Needs `bun run wasm`, `bun run voice` and `bun run llm` first, and Playwright:
 * `PLAYWRIGHT=<path to playwright/index.mjs>` if it isn't installed, `CHROME=<executable>` for
 * a browser of your own (Chrome or Chrome for Testing: `chrome-headless-shell` has no GPU).
 * In a window, for the display's real timing; `HEADLESS=1` runs without one (still on the real
 * GPU, frames at 60 a second). Usage: `bun tools/llm/devices.mjs`.
 */

import { spawn } from 'node:child_process';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { quiet } from '../bench/quiet.mjs';

const WEB = fileURLToPath(new URL('../../web/', import.meta.url));
const PORT = 5299;

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
  headless: !!process.env.HEADLESS,
  // (Without one of its own: Playwright's full Chromium, which has the GPU headless too.)
  ...(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chromium' }),
  args: ['--enable-unsafe-webgpu', '--enable-features=Vulkan,WebGPU', '--use-angle=metal', '--ignore-gpu-blocklist'],
});
let failed = false;
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  await page.addInitScript(quiet);
  page.on('console', (m) => m.type() === 'error' && console.error(`page: ${m.text()}`));
  await page.goto(`http://localhost:${PORT}/?debug`);
  await page.waitForFunction(() => typeof window.__llmBench === 'function', null, { timeout: 60_000 });
  const gpu = await page.evaluate(async () => !!(await navigator.gpu?.requestAdapter()));
  console.log(`devices: WebGPU adapter ${gpu ? 'found' : 'missing (nothing runs; try HEADED=1)'}`);
  const report = await page.evaluate(() => window.__llmBench());
  const r = report.result;
  console.table([{
    ran: r.ran ? 'yes' : `no: ${r.error}`,
    'load s': (r.loadMs / 1000).toFixed(1),
    'prompt ms': `${r.prefillMs.toFixed(0)} / ${r.prefillCachedMs.toFixed(0)} cached`,
    'tok/s': r.tokPerS.toFixed(1),
    'write ms': r.writeMs.toFixed(0),
    'speech ms': r.speechMs.toFixed(0),
    'line ms': r.lineMs.toFixed(0),
    'speech rtf': r.speechRtf.toFixed(2),
    sentences: `${r.sentences.toFixed(1)} (${r.cut} cut)`,
    'frame p50/p95': `${r.frameP50.toFixed(1)} / ${r.frameP95.toFixed(1)}`,
    'clean': `${r.clean}/${r.written}`,
    right: r.ok ? 'yes' : 'no',
    usable: report.usable ? 'yes' : 'no',
  }]);
  if (r.ran) console.log(r.lines.map((l) => JSON.stringify(l)).join(' · '));
  const use = report.usable ? "the model writes residents' lines" : "the model's lines are off (not fast and right enough)";
  console.log(`devices: ${report.model}; idle frames ${report.idleP50.toFixed(1)} / ${report.idleP95.toFixed(1)} ms; GPU "${report.gpu}"; ${use}; voices ${report.voicesOk ? 'ok' : 'too slow'}`);
  console.log(JSON.stringify(report));
  if (r.ran && r.nonFinite) {
    console.error("devices FAILED: the model wrote garbage (logits that aren't numbers)");
    failed = true;
  }
} finally {
  await browser.close();
  stop();
}
process.exit(failed ? 1 : 0);
