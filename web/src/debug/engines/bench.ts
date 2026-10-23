/**
 * Dev-only engine comparison (`bun run dev`, then /engines.html): the same scene of the game's
 * furniture (`scene.ts`) drawn by Babylon.js and by PlayCanvas, case by case, each run in a
 * fresh frame of its own so the engines never share a page or a GPU device. Not part of the build.
 *
 * What it measures, per run: the main thread's time for a whole engine frame (update, culling,
 * draw submission: Babylon's beginFrame/render/endFrame, PlayCanvas's update/render), the time
 * between frames (capped by the display, so it shows the GPU only once it is the slower side),
 * draw calls, and the load and first frame (shaders). Not the GPU's own time.
 *
 * URL flags (the runner):
 * - `engines=babylon,playcanvas`, `apis=webgpu,webgl2` (WebGPU only where the browser has it).
 * - `cases=furniture,instanced,lamps,shadows,game` (see `CASES`). The game-sized cases, with
 *   residents and post-processing: `cases=lot,lot-nopost,lot-bloom,lot-crowd` (one house; they
 *   ignore `rooms`).
 * - `rooms=8` (20 models each), `secs=8` measured after `warmup=3`, `w=1600&h=900` drawn pixels.
 * A single run is the same page with `engine=…&api=…&case=…`.
 *
 * The results land in a table, in `window.__engineBench`, and with the dev server in
 * `bench-results/<time>-engines-<label>.json`. Keep the tab in front while it runs: a hidden
 * tab's frames are throttled.
 *
 * Safari (WebKit 26.5) slows down for good after a few runs in one browser, whichever engine ran:
 * there, open each single run in a freshly started browser and read `window.__engineResult`.
 */

import { CASES, type Driver, type DriverOptions } from './scene';

const params = new URLSearchParams(location.search);
const num = (name: string, fallback: number) => {
  const value = Number(params.get(name));
  return params.has(name) && Number.isFinite(value) && value > 0 ? value : fallback;
};
const list = (name: string, fallback: string[]) => (params.get(name)?.split(',').filter(Boolean) ?? fallback);

const ROOMS = num('rooms', 8);
const SECS = num('secs', 8);
const WARMUP = num('warmup', 3);
const WIDTH = num('w', 1600);
const HEIGHT = num('h', 900);
/** Longest a run may take (loading included) before the runner gives up on it. */
const RUN_LIMIT_MS = (WARMUP + SECS) * 1000 + 90_000;

interface Stats {
  avg: number;
  p50: number;
  p95: number;
  p99: number;
  max: number;
}

export interface RunResult {
  ok: boolean;
  error?: string;
  engine: string;
  version: string;
  api: string;
  case: string;
  rooms: number;
  size: [number, number];
  /** From the start to the scene being ready to draw (models, materials), and the first frame's main-thread time (ms). */
  loadMs: number;
  firstFrameMs: number;
  frames: number;
  fps: number;
  /** Main-thread time of a whole engine frame (ms). */
  cpu: Stats | null;
  /** Time between frames (ms). */
  interval: Stats | null;
  drawCalls: number | null;
}

function stats(values: number[]): Stats | null {
  if (values.length === 0) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const at = (q: number) => sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))];
  return { avg: values.reduce((a, b) => a + b, 0) / values.length, p50: at(0.5), p95: at(0.95), p99: at(0.99), max: sorted[sorted.length - 1] };
}

const nextFrame = () => new Promise<number>((resolve) => requestAnimationFrame(resolve));

/** One engine, one case: warms up, measures, and hands the result to the runner. */
async function runOne(engine: string, api: DriverOptions['api'], name: string): Promise<RunResult> {
  const spec = CASES[name];
  const rooms = spec?.rooms ?? ROOMS;
  const base: RunResult = { ok: false, engine, version: '', api, case: name, rooms, size: [WIDTH, HEIGHT], loadMs: 0, firstFrameMs: 0, frames: 0, fps: 0, cpu: null, interval: null, drawCalls: null };
  if (!spec) return { ...base, error: `no case ${name}` };
  const canvas = document.createElement('canvas');
  document.body.append(canvas);
  const started = performance.now();
  const options: DriverOptions = { canvas, api, width: WIDTH, height: HEIGHT, rooms, spec };
  const driver: Driver = engine === 'babylon' ? await (await import('./babylon')).babylonDriver(options) : await (await import('./playcanvas')).playcanvasDriver(options);
  const result = { ...base, version: driver.version, api: driver.api as DriverOptions['api'], loadMs: performance.now() - started };
  if (driver.api !== api) {
    driver.dispose();
    return { ...result, error: `asked for ${api}, the engine runs on ${driver.api}` };
  }

  const cpu: number[] = [];
  const interval: number[] = [];
  let draws = 0;
  const t0 = await nextFrame();
  let last = t0;
  let measureFrom = Infinity;
  for (let first = true; ; first = false) {
    const now = await nextFrame();
    const t = (now - t0) / 1000;
    const begin = performance.now();
    driver.frame(t);
    const spent = performance.now() - begin;
    if (first) result.firstFrameMs = spent;
    if (measureFrom === Infinity && t >= WARMUP) {
      measureFrom = now;
    } else if (now > measureFrom) {
      cpu.push(spent);
      interval.push(now - last);
      draws += driver.drawCalls();
      if (now - measureFrom >= SECS * 1000) break;
    }
    last = now;
  }
  driver.dispose();
  const measured = interval.reduce((a, b) => a + b, 0) / 1000;
  return { ...result, ok: true, frames: cpu.length, fps: cpu.length / measured, cpu: stats(cpu), interval: stats(interval), drawCalls: cpu.length ? draws / cpu.length : null };
}

/** The runs the runner makes, in order: each case on each API, the engines side by side. */
function plan(): { engine: string; api: DriverOptions['api']; case: string }[] {
  const apis = list('apis', ['webgpu', 'webgl2']).filter((api): api is DriverOptions['api'] => (api === 'webgpu' ? 'gpu' in navigator : api === 'webgl2'));
  const engines = list('engines', ['babylon', 'playcanvas']);
  const cases = list('cases', ['furniture', 'instanced', 'lamps', 'shadows', 'game']);
  const runs: ReturnType<typeof plan> = [];
  for (const api of apis) {
    for (const name of cases) {
      for (const engine of engines) runs.push({ engine, api, case: name });
    }
  }
  return runs;
}

/** One run in a fresh frame, its result posted back. */
function inFrame(run: ReturnType<typeof plan>[number], stage: HTMLElement): Promise<RunResult> {
  return new Promise((resolve) => {
    const frame = document.createElement('iframe');
    const url = new URL(location.href);
    url.searchParams.set('engine', run.engine);
    url.searchParams.set('api', run.api);
    url.searchParams.set('case', run.case);
    frame.src = url.href;
    const done = (result: RunResult) => {
      clearTimeout(timer);
      removeEventListener('message', onMessage);
      frame.remove();
      resolve(result);
    };
    const onMessage = (event: MessageEvent) => {
      if (event.source === frame.contentWindow && event.data?.type === 'engine-bench') done(event.data.result as RunResult);
    };
    const failed = (error: string): RunResult => ({ ok: false, error, engine: run.engine, version: '', api: run.api, case: run.case, rooms: ROOMS, size: [WIDTH, HEIGHT], loadMs: 0, firstFrameMs: 0, frames: 0, fps: 0, cpu: null, interval: null, drawCalls: null });
    const timer = setTimeout(() => done(failed(`no result after ${RUN_LIMIT_MS / 1000} s`)), RUN_LIMIT_MS);
    addEventListener('message', onMessage);
    stage.replaceChildren(frame);
  });
}

const ms = (value: number | undefined) => (value === undefined ? '–' : value.toFixed(2));

/** The table: a row per case and API, each engine's frame time beside the other's. */
function table(results: RunResult[]): HTMLTableElement {
  const el = document.createElement('table');
  const head = ['case', 'api', 'engine', 'cpu avg', 'cpu p95', 'interval avg', 'fps', 'draws', 'load', '1st frame', 'vs babylon'];
  el.innerHTML = `<thead><tr>${head.map((h) => `<th>${h}</th>`).join('')}</tr></thead>`;
  const body = el.createTBody();
  for (const r of results) {
    const row = body.insertRow();
    const babylon = results.find((o) => o.engine === 'babylon' && o.api === r.api && o.case === r.case);
    const b = babylon?.cpu?.avg;
    const own = r.cpu?.avg;
    const ratio = r.ok && babylon?.ok && babylon !== r && b && own ? (b > own ? `${(b / own).toFixed(2)}× faster` : `${(own / b).toFixed(2)}× slower`) : '';
    const cells = r.ok
      ? [r.case, r.api, `${r.engine} ${r.version}`, ms(r.cpu?.avg), ms(r.cpu?.p95), ms(r.interval?.avg), r.fps.toFixed(0), r.drawCalls === null ? 'n/a' : r.drawCalls.toFixed(0), `${(r.loadMs / 1000).toFixed(1)} s`, `${ms(r.firstFrameMs)}`, ratio]
      : [r.case, r.api, r.engine, `failed: ${r.error}`];
    for (const cell of cells) row.insertCell().textContent = cell;
  }
  return el;
}

async function machine(): Promise<Record<string, unknown>> {
  const adapter = 'gpu' in navigator ? await navigator.gpu.requestAdapter().catch(() => null) : null;
  const info = adapter?.info;
  return {
    userAgent: navigator.userAgent,
    devicePixelRatio,
    cores: navigator.hardwareConcurrency,
    adapter: info ? { vendor: info.vendor, architecture: info.architecture, description: info.description } : null,
  };
}

async function runner(): Promise<void> {
  const out = document.getElementById('out')!;
  const stage = document.getElementById('stage')!;
  const runs = plan();
  const results: RunResult[] = [];
  const startedAt = new Date().toISOString();
  for (const [i, run] of runs.entries()) {
    out.firstElementChild!.textContent = `run ${i + 1} of ${runs.length}: ${run.engine} · ${run.api} · ${run.case} (${ROOMS} rooms, ${WIDTH}×${HEIGHT}, ${WARMUP} s warm-up, ${SECS} s measured)`;
    results.push(await inFrame(run, stage));
    out.querySelector('table')?.remove();
    out.append(table(results));
  }
  stage.remove();
  const report = { startedAt, scene: 'engines', label: `rooms${ROOMS}-${WIDTH}x${HEIGHT}`, rooms: ROOMS, secs: SECS, warmup: WARMUP, machine: await machine(), runs: results };
  (window as unknown as { __engineBench: unknown }).__engineBench = report;
  let saved = '';
  if (import.meta.env.DEV) {
    try {
      const res = await fetch('/__debug/bench', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(report) });
      if (res.ok) saved = ` · saved to ${((await res.json()) as { path: string }).path}`;
    } catch {
      // No dev server endpoint: the table and window.__engineBench still hold it.
    }
  }
  out.firstElementChild!.textContent = `done: ${results.length} runs${saved}. cpu = main-thread ms per frame (lower is better); "vs babylon" compares cpu avg.`;
}

if (params.has('engine')) {
  document.body.classList.add('run');
  const api = params.get('api') === 'webgl2' ? 'webgl2' : 'webgpu';
  void runOne(params.get('engine') ?? 'babylon', api, params.get('case') ?? 'furniture')
    .catch((err: unknown): RunResult => ({ ok: false, error: String(err), engine: params.get('engine') ?? '', version: '', api, case: params.get('case') ?? '', rooms: ROOMS, size: [WIDTH, HEIGHT], loadMs: 0, firstFrameMs: 0, frames: 0, fps: 0, cpu: null, interval: null, drawCalls: null }))
    .then((result) => {
      (window as unknown as { __engineResult: RunResult }).__engineResult = result;
      if (parent !== window) parent.postMessage({ type: 'engine-bench', result }, location.origin);
      else console.log('[engines]', result);
    });
} else {
  void runner();
}
