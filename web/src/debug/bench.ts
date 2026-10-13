/**
 * Rendering benchmark: `?bench=<scene>` plays the game state `public/bench/<scene>.json.gz` with the
 * camera on a path fixed in time, and measures how frames are made. Loaded only with the flag;
 * the settings it changes (no watching camera, no autosave, its resolution and frame cap) apply
 * to its page alone and are never saved.
 *
 * URL flags (see `benchFlags.ts`):
 * - `quality=low|medium|high`: the graphics presets (`render/quality.ts`); without it, the player's setting.
 * - `hour=13|21`: the lighting held at that hour; without it, the game's clock (the scene's evening).
 * - `cam=orbit` (24 m round the home lot, a turn in 40 s), `wide` (80 m, 90 s), `build` (orbit in
 *   Build mode, paused), `dusk` (orbit at speed 5 from the scene's 18:00, the light changing every frame).
 * - `res=1` the full resolution (or 0.5–1); `res=auto` (default) what the game picks.
 * - `secs=20` measured after `warmup=5`; `cap=0` frames per second at most (0: every frame shown).
 * - `hud=0` hides the interface; `gpu=0` doesn't time the GPU; `label=…` is kept with the result.
 * - `diag=noshadows,noaa,nonature,nolamps,nomirrors,noglass,nonight,freeze`: features
 *   left out (to measure what each costs); `freeze` stops time for screenshots (`at=<s>` along the path).
 *
 * The result lands in `window.__benchResult` (with the dev server also in `bench-results/`) and in
 * a small panel to copy. `tools/bench/render.mjs` runs a whole matrix of them.
 */

import { host } from '../game/host';
import { deleteSave } from '../persistence/saves';
import type { FrameCounts, RenderPassInfo } from '../render/types';
import { settings, unsavedSettings } from '../settings/settings.svelte';
import { app } from '../ui/app.svelte';
import { services } from '../ui/services';
import { game } from '../ui/state.svelte';
import { BENCH, type BenchFlags } from './benchFlags';
import { gpuInfo, loadState } from './report';

/** Camera paths: distance, height angle and seconds per turn round the home lot. */
const PATHS = {
  orbit: { radius: 24, beta: 0.9, period: 40 },
  wide: { radius: 80, beta: 0.95, period: 90 },
} as const;
/** Where every path starts (the game's own opening angle). */
const START_ALPHA = -Math.PI * 0.62;
/** Longest wait for the game to start (loading, shaders). */
const START_LIMIT_MS = 120_000;

interface Stats {
  avg: number;
  p50: number;
  p95: number;
  p99: number;
  max: number;
}

export interface BenchResult {
  ok: boolean;
  error?: string;
  version: string;
  scene: string;
  label: string;
  url: string;
  flags: Omit<BenchFlags, 'diag'> & { diag: string[] };
  startedAt: string;
  machine: { userAgent: string; devicePixelRatio: number; screen: [number, number]; cores: number; adapter: Record<string, string> | null };
  frames: number;
  secs: number;
  fps: number;
  /** Time between frames (requestAnimationFrame timestamps), ms. */
  interval: Stats | null;
  /** Main-thread work per frame: all of it, the game's update (residents, camera, bubbles…) and drawing (Babylon's scene.render), ms. */
  cpu: { frame: Stats; update: Stats; render: Stats } | null;
  /** GPU time per frame, ms (null: the browser can't measure it); per pass on WebGPU. */
  gpu: { frame: Stats | null; source: 'frame' | 'passes' | 'none'; passes: { name: string; ms: number; frames: number }[] };
  drawCalls: { total: number; shadow: number; mirrors: number };
  triangles: { total: number; shadow: number; mirrors: number };
  /** Render passes of a frame and their pixels × samples, and bytes × samples (colour plus depth, written once). */
  passes: RenderPassInfo[];
  pixelSamples: number;
  targetBytes: number;
  residentsMs: number;
  lightingPerSecond: number;
  longTasks: { count: number; maxMs: number; totalMs: number };
  render: Record<string, unknown>;
  /** Scene counts at the start: equal fingerprints mean two runs drew the same scene. */
  fingerprint: string;
}

declare global {
  interface Window {
    __benchResult?: BenchResult;
    /** The scene is up and the warm-up is over (screenshots of a frozen scene are taken then). */
    __benchReady?: boolean;
  }
}

/** Before start-up reads them: the benchmark's settings, for this page only. */
export function prepareBench(): void {
  const b = BENCH!;
  unsavedSettings();
  // (The renderer takes `?quality=` itself; the setting too, for what shows it.)
  if (b.quality) settings.graphics = b.quality;
  if (b.res !== 'auto') settings.resolution = b.res;
  Object.assign(settings, { directorDelay: 0, autosaveMinutes: 0, fpsCap: b.cap, autoSlow: false, voices: false, showFps: false, reducedMotion: false });
  const css: string[] = [];
  if (!b.hud) css.push('.hud-root, .world-overlay, .toast-layer { display: none !important; }');
  if (b.diag.has('noglass')) css.push('* { backdrop-filter: none !important; -webkit-backdrop-filter: none !important; }');
  if (css.length) document.head.append(Object.assign(document.createElement('style'), { textContent: css.join('\n') }));
}

/** After start-up: plays the scene, measures, reports. */
export async function runBench(): Promise<void> {
  const b = BENCH!;
  const startedAt = new Date().toISOString();
  let result: BenchResult;
  try {
    result = await measure(b, startedAt);
  } catch (err) {
    console.error('[bench]', err);
    result = { ...emptyResult(b, startedAt), error: err instanceof Error ? err.message : String(err) };
  }
  window.__benchResult = result;
  console.info(`[bench] ${summary(result)}`);
  if (b.hud) showPanel(result);
  if (import.meta.env.DEV) {
    try {
      const res = await fetch('/__debug/bench', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(result) });
      if (res.ok) console.info(`[bench] saved to ${((await res.json()) as { path: string }).path}`);
    } catch (err) {
      console.warn('[bench] not saved', err);
    }
  }
}

async function measure(b: BenchFlags, startedAt: string): Promise<BenchResult> {
  const saveId = await loadState(await sceneText(b.scene), `bench ${b.scene}`);
  app.start({ kind: 'load', saveId });
  // Live once the game screen shows its interface (the session is revealed by then).
  const until = performance.now() + START_LIMIT_MS;
  while (!(app.screen === 'game' && !app.loading && services.controls && document.querySelector('.hud-root'))) {
    if (performance.now() > until) throw new Error('The game did not start');
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  // The scene was copied into a save slot to start it; it isn't the player's.
  await deleteSave(saveId).catch(() => {});
  const renderer = await host.rendererNow(settings);
  const controls = services.controls;
  if (b.cam === 'build') controls.setMode('build');
  renderer.setGameShot(null);
  renderer.followSim(null);

  const home = game.households.find((h) => h.player)?.plot;
  const plot = home == null ? null : game.plots[home];
  const centre: [number, number, number] = plot ? [plot.x + plot.w / 2, 0, plot.z + plot.d / 2] : [0, 0, 0];
  const path = PATHS[b.cam === 'wide' ? 'wide' : 'orbit'];
  const frozen = b.diag.has('freeze');

  const intervals: number[] = [];
  const cpuFrame: number[] = [];
  const cpuUpdate: number[] = [];
  const cpuRender: number[] = [];
  const gpuFrame: number[] = [];
  const sums = { draws: 0, shadowDraws: 0, mirrorDraws: 0, tris: 0, shadowTris: 0, mirrorTris: 0, residents: 0 };
  let measuring = false;
  let lastGpuFrames = -1;
  const t0 = performance.now();
  renderer.measureGpu(b.gpu, 'bench');
  renderer.setFrameProbe({
    before(now) {
      const t = frozen ? b.at : (now - t0) / 1000;
      renderer.setCameraPose({ alpha: START_ALPHA + (2 * Math.PI * t) / path.period, beta: path.beta, radius: path.radius, target: centre });
    },
    after(start, updated, end, c: FrameCounts) {
      if (!measuring) return;
      cpuFrame.push(end - start);
      cpuUpdate.push(updated - start);
      cpuRender.push(end - updated);
      sums.draws += c.drawCalls;
      sums.shadowDraws += c.shadowDrawCalls;
      sums.mirrorDraws += c.mirrorDrawCalls;
      sums.tris += c.triangles;
      sums.shadowTris += c.shadowTriangles;
      sums.mirrorTris += c.mirrorTriangles;
      sums.residents += c.residentsMs;
      // A new whole-frame GPU time (they arrive a few frames late).
      if (c.gpuFrames !== lastGpuFrames) {
        lastGpuFrames = c.gpuFrames;
        gpuFrame.push(c.gpuMs);
      }
    },
  });

  try {
    await sleep(b.warmup * 1000);
    window.__benchReady = true;
    // The measured stretch.
    const info = renderer.benchInfo();
    renderer.gpuPasses(true);
    const lighting0 = (info.scene as { lightingUpdates: number }).lightingUpdates;
    const long: number[] = [];
    const observer = typeof PerformanceObserver !== 'undefined' && PerformanceObserver.supportedEntryTypes?.includes('longtask')
      ? new PerformanceObserver((list) => list.getEntries().forEach((e) => long.push(e.duration)))
      : null;
    observer?.observe({ type: 'longtask' });
    let last = 0;
    let rafCount = 0;
    let stop = false;
    const raf = (time: number) => {
      if (stop) return;
      if (last) intervals.push(time - last);
      last = time;
      rafCount++;
      requestAnimationFrame(raf);
    };
    measuring = true;
    const started = performance.now();
    requestAnimationFrame(raf);
    await sleep(b.secs * 1000);
    stop = true;
    measuring = false;
    const secs = (performance.now() - started) / 1000;
    observer?.disconnect();
    const passes = renderer.gpuPasses();
    const end = renderer.benchInfo();
    const frames = cpuFrame.length;
    const per = (n: number) => (frames ? n / frames : 0);
    const timed = gpuFrame.filter((ms) => ms > 0);
    const s = info.scene as Record<string, number>;
    const fingerprint = ['meshes', 'materials', 'shadowCasters', 'shadowCasterTriangles', 'natureInstances', 'residents', 'mirrors'].map((k) => `${k[0]}${s[k]}`).join('-');
    return {
      ...emptyResult(b, startedAt),
      ok: true,
      machine: { ...emptyResult(b, startedAt).machine, adapter: await gpuInfo() },
      frames,
      secs,
      fps: rafCount / secs,
      interval: stats(intervals),
      cpu: frames ? { frame: stats(cpuFrame)!, update: stats(cpuUpdate)!, render: stats(cpuRender)! } : null,
      gpu: { frame: stats(timed), source: timed.length ? 'frame' : passes.length ? 'passes' : 'none', passes },
      drawCalls: { total: per(sums.draws), shadow: per(sums.shadowDraws), mirrors: per(sums.mirrorDraws) },
      triangles: { total: per(sums.tris), shadow: per(sums.shadowTris), mirrors: per(sums.mirrorTris) },
      passes: info.passes,
      pixelSamples: info.passes.reduce((n, p) => n + p.width * p.height * p.samples, 0),
      targetBytes: info.passes.reduce((n, p) => n + p.width * p.height * p.samples * (p.colorBytes + p.depthBytes), 0),
      residentsMs: per(sums.residents),
      lightingPerSecond: ((end.scene as { lightingUpdates: number }).lightingUpdates - lighting0) / secs,
      longTasks: { count: long.length, maxMs: Math.max(0, ...long), totalMs: long.reduce((a, x) => a + x, 0) },
      render: { ...info, end: end.scene },
      fingerprint,
    };
  } finally {
    // A frozen picture stays where it is; otherwise the camera is the player's again.
    if (!frozen) renderer.setFrameProbe(null);
    renderer.measureGpu(false, 'bench');
  }
}

/**
 * The scene's game state: `bench/<scene>.json.gz` (a whole town's save is half a megabyte of JSON,
 * a tenth of that gzipped), or a plain `.json`.
 */
async function sceneText(scene: string): Promise<string> {
  const base = `${import.meta.env.BASE_URL}bench/${scene}.json`;
  const packed = await fetch(`${base}.gz`);
  if (packed.ok && packed.body) {
    // Served as a file (no Content-Encoding): unpacked here, unless the server already did.
    const bytes = new Uint8Array(await packed.arrayBuffer());
    if (bytes[0] !== 0x1f || bytes[1] !== 0x8b) return new TextDecoder().decode(bytes);
    return new Response(new Blob([bytes]).stream().pipeThrough(new DecompressionStream('gzip'))).text();
  }
  const plain = await fetch(base);
  if (!plain.ok) throw new Error(`Benchmark scene "${scene}" not found`);
  return plain.text();
}

function emptyResult(b: BenchFlags, startedAt: string): BenchResult {
  return {
    ok: false,
    version: __APP_VERSION__,
    scene: b.scene,
    label: b.label,
    url: location.href,
    flags: { ...b, diag: [...b.diag] },
    startedAt,
    machine: { userAgent: navigator.userAgent, devicePixelRatio: window.devicePixelRatio, screen: [screen.width, screen.height], cores: navigator.hardwareConcurrency ?? 0, adapter: null },
    frames: 0,
    secs: 0,
    fps: 0,
    interval: null,
    cpu: null,
    gpu: { frame: null, source: 'none', passes: [] },
    drawCalls: { total: 0, shadow: 0, mirrors: 0 },
    triangles: { total: 0, shadow: 0, mirrors: 0 },
    passes: [],
    pixelSamples: 0,
    targetBytes: 0,
    residentsMs: 0,
    lightingPerSecond: 0,
    longTasks: { count: 0, maxMs: 0, totalMs: 0 },
    render: {},
    fingerprint: '',
  };
}

function stats(values: number[]): Stats | null {
  if (!values.length) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const at = (q: number) => sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))];
  return { avg: values.reduce((a, x) => a + x, 0) / values.length, p50: at(0.5), p95: at(0.95), p99: at(0.99), max: sorted[sorted.length - 1] };
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/** A few lines to read (and paste). */
export function summary(r: BenchResult): string {
  if (!r.ok) return `bench ${r.scene} failed: ${r.error}`;
  const f = r.flags;
  const render = r.render as { backend?: string; render?: { width: number; height: number }; quality?: { shadowMapSize: number; postSamples: number } };
  const ms = (x: number | undefined) => (x ?? 0).toFixed(1);
  const mtris = (x: number) => `${(x / 1e6).toFixed(2)} M`;
  return [
    `${r.scene} · ${f.quality ?? 'own settings'} · ${render.backend} · ${f.cam}${f.hour !== null ? ` · ${f.hour}:00` : ''}${f.diag.length ? ` · ${f.diag.join(',')}` : ''}${f.label ? ` · ${f.label}` : ''}`,
    `${render.render?.width}×${render.render?.height} · ${render.quality?.postSamples}× MSAA · ${render.quality?.shadowMapSize} shadows · fingerprint ${r.fingerprint}`,
    `${r.fps.toFixed(1)} fps · frame ${ms(r.interval?.p50)} / p95 ${ms(r.interval?.p95)} / p99 ${ms(r.interval?.p99)} ms`,
    `CPU ${ms(r.cpu?.frame.avg)} ms (update ${ms(r.cpu?.update.avg)}, draw ${ms(r.cpu?.render.avg)}) · GPU ${r.gpu.frame ? `${ms(r.gpu.frame.avg)} ms (p95 ${ms(r.gpu.frame.p95)})` : r.gpu.passes.length ? `${ms(r.gpu.passes.reduce((a, p) => a + p.ms, 0))} ms (passes)` : 'n/a'}`,
    `${r.drawCalls.total.toFixed(0)} draws (shadow ${r.drawCalls.shadow.toFixed(0)}) · ${mtris(r.triangles.total)} triangles (shadow ${mtris(r.triangles.shadow)}) · ${r.passes.length} passes, ${(r.pixelSamples / 1e6).toFixed(1)} M pixel samples`,
    `residents ${r.residentsMs.toFixed(2)} ms · lighting ${r.lightingPerSecond.toFixed(1)}/s · long tasks ${r.longTasks.count}${r.longTasks.count ? ` (max ${r.longTasks.maxMs.toFixed(0)} ms)` : ''}`,
  ].join('\n');
}

function showPanel(r: BenchResult): void {
  const panel = document.createElement('div');
  panel.className = 'glass';
  panel.style.cssText = 'position:fixed;left:12px;bottom:12px;z-index:100;padding:10px 14px;font:12px/1.5 ui-monospace,monospace;white-space:pre;pointer-events:auto;max-width:90vw;overflow:auto';
  const text = document.createElement('div');
  text.textContent = summary(r);
  const copy = document.createElement('button');
  copy.className = 'btn';
  copy.textContent = 'Copy result';
  copy.style.marginTop = '6px';
  copy.onclick = () => void navigator.clipboard.writeText(JSON.stringify(r, null, 2)).then(() => (copy.textContent = 'Copied'));
  panel.append(text, copy);
  document.body.append(panel);
}
