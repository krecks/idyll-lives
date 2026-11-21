#!/usr/bin/env bun
/**
 * The voice benchmark (`web/src/voice/bench.ts`): starts Vite, opens `voice-bench.html` in
 * Chromium with WebGPU and runs the game's own voice worker with no game around it: once to fill
 * the browser cache, then measured. Prints load time, the GPU buffers the model holds (once
 * loaded and at the peak), the worker's memory, each line's time and how many times faster than
 * it plays it was made, and writes the report and every line's first run as a WAV under
 * `bench-results/voice-<time>/`.
 *
 * `--compare <dirA> <dirB>` compares two runs' WAVs line by line instead (the same lines in the
 * same voices): whether the samples are identical, and if not how alike their spectra are
 * (correlation of the log-magnitude spectrograms, 1 = the same; log-spectral distance in dB).
 *
 * Needs `bun install`, `bun run voice` and Playwright: `PLAYWRIGHT=<path to
 * playwright/index.mjs>` if it isn't installed, `CHROME=<executable>` for a browser of your own.
 * Usage (from the repo root or `web/`):
 *
 *   bun tools/voice/bench.mjs [options]
 *     --port 5398        Vite's port (it starts its own; `--url http://localhost:5398` uses a running one)
 *     --device gpu       or cpu (Auto's fallback)
 *     --runs 3           runs per line (median)
 *     --headless         without a window (still the real GPU)
 *     --out <dir>        default <repo>/bench-results/voice-<time>/
 *   bun tools/voice/bench.mjs --compare <dirA> <dirB>
 */

import { spawn } from 'node:child_process';
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const WEB = fileURLToPath(new URL('../../web/', import.meta.url));
const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const args = process.argv.slice(2);
const flag = (name, fallback = undefined) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 ? (args[i + 1] && !args[i + 1].startsWith('--') ? args[i + 1] : true) : fallback;
};
const RATE = 24000;
const fmt = (n, digits = 0) => (Number.isFinite(n) ? n.toFixed(digits) : '—');
const mb = (b) => fmt(b / 1e6, 1);

/** 16-bit PCM mono WAV. */
function wav(samples) {
  const out = Buffer.alloc(44 + samples.length * 2);
  out.write('RIFF', 0);
  out.writeUInt32LE(36 + samples.length * 2, 4);
  out.write('WAVEfmt ', 8);
  out.writeUInt32LE(16, 16);
  out.writeUInt16LE(1, 20);
  out.writeUInt16LE(1, 22);
  out.writeUInt32LE(RATE, 24);
  out.writeUInt32LE(RATE * 2, 28);
  out.writeUInt16LE(2, 32);
  out.writeUInt16LE(16, 34);
  out.write('data', 36);
  out.writeUInt32LE(samples.length * 2, 40);
  for (let i = 0; i < samples.length; i++) out.writeInt16LE(Math.round(Math.max(-1, Math.min(1, samples[i])) * 32767), 44 + i * 2);
  return out;
}

/** A WAV's samples (16-bit PCM mono, as `wav` writes them). */
function readWav(path) {
  const b = readFileSync(path);
  const n = b.readUInt32LE(40) / 2;
  const out = new Float32Array(n);
  for (let i = 0; i < n; i++) out[i] = b.readInt16LE(44 + i * 2) / 32767;
  return out;
}

/** Log-magnitude spectrogram (1024-point Hann frames, hop 256), frames × 513. */
function spectrogram(x) {
  const N = 1024;
  const hop = 256;
  const win = Float32Array.from({ length: N }, (_, i) => 0.5 - 0.5 * Math.cos((2 * Math.PI * i) / N));
  const frames = [];
  const re = new Float64Array(N);
  const im = new Float64Array(N);
  for (let start = 0; start + N <= x.length; start += hop) {
    for (let i = 0; i < N; i++) {
      re[i] = x[start + i] * win[i];
      im[i] = 0;
    }
    fft(re, im);
    const mag = new Float32Array(N / 2 + 1);
    for (let k = 0; k <= N / 2; k++) mag[k] = Math.log10(re[k] * re[k] + im[k] * im[k] + 1e-10) * 10;
    frames.push(mag);
  }
  return frames;
}

/** In-place radix-2 FFT. */
function fft(re, im) {
  const n = re.length;
  for (let i = 1, j = 0; i < n; i++) {
    let bit = n >> 1;
    for (; j & bit; bit >>= 1) j ^= bit;
    j ^= bit;
    if (i < j) {
      [re[i], re[j]] = [re[j], re[i]];
      [im[i], im[j]] = [im[j], im[i]];
    }
  }
  for (let len = 2; len <= n; len <<= 1) {
    const a = (-2 * Math.PI) / len;
    for (let i = 0; i < n; i += len) {
      for (let k = 0; k < len / 2; k++) {
        const cr = Math.cos(a * k);
        const ci = Math.sin(a * k);
        const xr = re[i + k + len / 2] * cr - im[i + k + len / 2] * ci;
        const xi = re[i + k + len / 2] * ci + im[i + k + len / 2] * cr;
        re[i + k + len / 2] = re[i + k] - xr;
        im[i + k + len / 2] = im[i + k] - xi;
        re[i + k] += xr;
        im[i + k] += xi;
      }
    }
  }
}

/** How alike two clips sound: spectrogram correlation and log-spectral distance (dB), over the shorter one. */
function compareClips(a, b) {
  const sa = spectrogram(a);
  const sb = spectrogram(b);
  const n = Math.min(sa.length, sb.length);
  let sum = 0;
  let sumA = 0;
  let sumB = 0;
  let sumAA = 0;
  let sumBB = 0;
  let sumAB = 0;
  let count = 0;
  for (let f = 0; f < n; f++) {
    let d2 = 0;
    for (let k = 0; k < sa[f].length; k++) {
      // Below -60 dB both are silence: clamp so noise floors don't dominate.
      const x = Math.max(sa[f][k], -60);
      const y = Math.max(sb[f][k], -60);
      d2 += (x - y) ** 2;
      sumA += x;
      sumB += y;
      sumAA += x * x;
      sumBB += y * y;
      sumAB += x * y;
      count++;
    }
    sum += Math.sqrt(d2 / sa[f].length);
  }
  const cov = sumAB / count - (sumA / count) * (sumB / count);
  const corr = cov / Math.sqrt((sumAA / count - (sumA / count) ** 2) * (sumBB / count - (sumB / count) ** 2));
  return { corr, lsd: sum / Math.max(n, 1) };
}

if (flag('compare')) {
  const [a, b] = [String(flag('compare')), args[args.indexOf('--compare') + 2]];
  if (!b) throw new Error('usage: --compare <dirA> <dirB>');
  const rows = [];
  for (const name of readdirSync(a).filter((f) => f.endsWith('.wav')).sort()) {
    const x = readWav(`${a}/${name}`);
    let y;
    try {
      y = readWav(`${b}/${name}`);
    } catch {
      rows.push({ line: name, note: 'missing in B' });
      continue;
    }
    const identical = x.length === y.length && x.every((v, i) => v === y[i]);
    const { corr, lsd } = identical ? { corr: 1, lsd: 0 } : compareClips(x, y);
    rows.push({ line: name.replace(/\.wav$/, ''), 'A s': fmt(x.length / RATE, 2), 'B s': fmt(y.length / RATE, 2), identical: identical ? 'yes' : 'no', 'spectral corr': fmt(corr, 4), 'LSD dB': fmt(lsd, 2) });
  }
  console.table(rows);
  process.exit(0);
}

const PORT = Number(flag('port', 5398));
const DEVICE = String(flag('device', 'gpu'));
const RUNS = Number(flag('runs', 3));
const HEADLESS = args.includes('--headless');
const OUT = String(flag('out', `${ROOT}bench-results/voice-${new Date().toISOString().replace(/[:.]/g, '-')}/`)).replace(/\/?$/, '/');

async function playwright() {
  if (process.env.PLAYWRIGHT) return import(process.env.PLAYWRIGHT);
  try {
    return await import(createRequire(WEB + 'package.json').resolve('playwright'));
  } catch {
    throw new Error('Playwright not found: install it, or set PLAYWRIGHT=<path to playwright/index.mjs>');
  }
}

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

const vite = await startVite();
const { chromium } = await playwright();
const browser = await chromium.launch({
  headless: HEADLESS,
  ...(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chromium' }),
  args: ['--enable-unsafe-webgpu', '--enable-features=Vulkan,WebGPU', '--use-angle=metal', '--ignore-gpu-blocklist', '--enable-blink-features=ForceEagerMeasureMemory', '--window-position=2600,0'],
});
let failed = false;
try {
  const page = await browser.newPage({ viewport: { width: 900, height: 600 } });
  page.on('console', (m) => m.type() === 'error' && console.error(`page: ${m.text()}`));
  page.on('pageerror', (e) => console.error(`page error: ${e.message}`));
  const run = async (options) => {
    await page.goto(`${vite.url}/voice-bench.html`);
    await page.waitForFunction(() => typeof window.__voicePerf === 'function', null, { timeout: 120_000 });
    return page.evaluate((o) => window.__voicePerf(o), options);
  };
  console.log(`voice bench: ${DEVICE}, filling the cache`);
  await run({ device: DEVICE, runs: 1 });
  console.log(`voice bench: ${DEVICE}, measured (${RUNS} runs a line)`);
  const report = await run({ device: DEVICE, runs: RUNS });
  mkdirSync(OUT, { recursive: true });
  const rows = report.lines.map((l) => {
    const samples = new Float32Array(Buffer.from(l.samples, 'base64').buffer.slice(0));
    writeFileSync(`${OUT}${l.name}.wav`, wav(samples));
    l.sha = createHash('sha256').update(Buffer.from(l.samples, 'base64')).digest('hex').slice(0, 12);
    delete l.samples;
    const t = (k) => fmt(Math.min(...l.runs.map((r) => r.timing[k])));
    return { line: l.name, 's': fmt(l.seconds, 2), 'ms (med)': fmt(l.ms), '× real time': fmt(l.speed, 1), 'dur/text/vector/vocoder/finish ms (min)': ['duration', 'text', 'vector', 'vocoder', 'finish'].map(t).join(' / '), stable: l.stable ? 'yes' : 'NO', sha: l.sha };
  });
  console.table(rows);
  const p = report.perSecond;
  console.log(`load ${fmt(report.loadMs)} ms, warm-up ${fmt(report.warmupMs)} ms; ${fmt(report.speed, 1)}× real time over all lines`);
  console.log(`per second of speech (median): duration ${fmt(p.duration, 1)}, text ${fmt(p.text, 1)}, vector ${fmt(p.vector, 1)}, vocoder ${fmt(p.vocoder, 1)}, finish ${fmt(p.finish, 1)} ms`);
  console.log(`GPU buffers: ${mb(report.gpuLoaded)} MB loaded, ${mb(report.gpuPeak)} MB peak, ${mb(report.gpuAfter)} MB after; worker memory ${mb(report.workerLoaded)} MB loaded, ${mb(report.workerAfter)} MB after`);
  writeFileSync(`${OUT}report.json`, JSON.stringify({ startedAt: new Date().toISOString(), ...report }, null, 2));
  console.log(`voice bench: wrote ${OUT}`);
} catch (err) {
  console.error(`voice bench FAILED: ${err instanceof Error ? err.message : err}`);
  failed = true;
} finally {
  await browser.close();
  vite.stop();
}
process.exit(failed ? 1 : 0);
