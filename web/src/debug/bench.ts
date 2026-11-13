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
 *   Build mode, paused), `dusk` (orbit at speed 5 from the scene's 18:00, the light changing every frame),
 *   `mirror` (the home lot's first mirror from 4.4 m, from over its room with the walls up, the camera
 *   swinging round the glass every 20 s), `mirror-still` (the same, held still in the swing's middle),
 *   `street` (the home lot from across its street, low: the house's front, the pavement, the road),
 *   `town` (the menus' town from the menu's own angle, lit for `hour`),
 *   `over` (the town view, docs/design/living-town.md: the game's camera held `radius=` m out, 400 by default,
 *   over the main town's centre or the home, `over=centre|home`), `zoom` (the one zoom from the home at the
 *   game's opening distance out to 900 m and back in, once in 60 s).
 * - `townCentre=1`: the orbit round the drawn world's main town (the town view) instead of the home lot.
 * - `radius=`, `beta=`, `tx=`, `tz=`: the orbit's distance, angle and target instead of its path's (the game
 *   opens at beta 0.86); `walls=up|cutaway|down`: the walls held that way (up: the house whole, roof on).
 * - `res=1080` that resolution (720, 1080, 1440, 2160; `res=1` every line the window has); `res=auto` (default) the player's setting.
 * - `secs=20` measured after `warmup=5`; `cap=0` frames per second at most (0: every frame shown).
 * - `hud=0` hides the interface; `gpu=0` doesn't time the GPU; `label=…` is kept with the result.
 * - `diag=noshadows,noaa,nonature,nolamps,nomirrors,noglass,nonight,noao,noclouds,freeze`: features
 *   left out (to measure what each costs; `nowoods`, `nofarwoods`, `noflowers` and `nograss` leave out
 *   one part of the nature); `freeze` stops time for screenshots (`at=<s>` along the path);
 *   `shaders` logs every shader and pipeline made (the warm-up should leave none for play).
 *
 * The result lands in `window.__benchResult` (with the dev server also in `bench-results/`) and in
 * a small panel to copy. `tools/bench/render.mjs` runs a whole matrix of them.
 */

import { host } from '../game/host';
import { whenStreamSettled } from '../assets/stream';
import { deleteSave } from '../persistence/saves';
import type { WorldStructure } from '../core/protocol';
import { DEFAULT_ORBIT, TOWN_FULL, TOWN_MAX } from '../render/shared/lens';
import type { FrameCounts, Renderer, RenderPassInfo } from '../render/types';
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
/**
 * The mirror camera, round the glass's centre: the eye's distance from it, its height angle, its turn
 * from straight in front (rad; the path's middle), how far it swings either side of that and in how
 * long (s). The walls are up: the cutaway drops the wall a mirror hangs on whenever the camera faces
 * it. The eye is over the room, closer to the glass than the game camera's target ever is (`radius`,
 * its nearest): the camera aims at a point `radius` along the view, behind the glass, so the glass
 * stays in the middle of the view. Its height is above what `keepCameraClear` asks (4 m over the
 * land), so neither renderer moves it. The turn and the swing keep the village bathroom's ceiling
 * lamp beside the glass, not in front of it.
 */
const MIRROR_PATH = { distance: 4.4, radius: 6, beta: 0.8, turn: -0.4, swing: 0.2, period: 20 };
/** `cam=street`: aimed at the home lot's entry from across the street, low enough for the house's front and the far pavement. */
const STREET = { radius: 22, beta: 1.2 };
/** `cam=town`: the menus' town as the main menu frames it (`menuScene.ts`), over the whole picture. */
const TOWN = { beta: 1.32, alpha: 0.55, zoom: 1.55 };
/**
 * The town view (`cam=over`, `cam=zoom`): the game's own camera, from the angle a game opens at. `settle`: the
 * seconds it first stays at the opening distance over the place, so the lot there becomes the focus (drawn in full,
 * as the home is when a player zooms out of it: past `TOWN_FROM` no lot becomes one). `period`: `cam=zoom`'s way
 * out and back in (s), the distance changing by the same share each second.
 */
const TOWN_VIEW = { beta: DEFAULT_ORBIT.beta, settle: 2.5, period: 60 };

/**
 * Where the town view is held over, in the world drawn: the main town's centre (its square once the world has one,
 * `meta.centre`; else the middle of its area) and the home. `home`: the player's lot in the world the game plays;
 * the bench's real big world (`debug/bigTown.ts`) has no household of the player's, so there a lot with a house
 * three quarters of the way out from the main town's middle stands in for it (the same one every run).
 */
function townSpots(world: WorldStructure | null, home: [number, number, number] | null): { centre: [number, number, number]; home: [number, number, number] } {
  if (!world) return { centre: home ?? [0, 0, 0], home: home ?? [0, 0, 0] };
  const main = world.meta?.areas?.[0];
  const square = world.meta?.centre?.find((c) => c.area === 0) ?? world.meta?.centre?.[0];
  const mid = (pts: [number, number][]): [number, number] => [pts.reduce((a, p) => a + p[0], 0) / pts.length, pts.reduce((a, p) => a + p[1], 0) / pts.length];
  const [cx, cz] = square?.poly.length ? mid(square.poly) : main ? [main.x + main.w / 2, main.z + main.d / 2] : [world.width / 2, world.depth / 2];
  let at = home;
  if (!at) {
    const inMain = (p: { x: number; z: number; w: number; d: number }) => !main || (p.x + p.w / 2 >= main.x && p.x + p.w / 2 <= main.x + main.w && p.z + p.d / 2 >= main.z && p.z + p.d / 2 <= main.z + main.d);
    const away = (p: { x: number; z: number; w: number; d: number }) => Math.hypot(p.x + p.w / 2 - cx, p.z + p.d / 2 - cz);
    const homes = world.plots.filter((p) => p.house && inMain(p)).sort((a, b) => away(a) - away(b) || a.id - b.id);
    const p = homes[Math.floor(homes.length * 0.75)];
    at = p ? [p.x + p.w / 2, 0, p.z + p.d / 2] : [cx, 0, cz];
  }
  return { centre: [cx, 0, cz], home: at };
}
/**
 * `cam=roam`: the camera's target along a loop over the town at `ROAM.speed` m/s (an ellipse
 * through the home lot, inset from the edge), looking the way it goes from `radius` m up and
 * behind: the street streams in ahead. It stops for `hold` s every `every` s, so the lot it stops
 * over becomes the focus (built in full) while it waits, and it moves on.
 */
const ROAM = { speed: 12, radius: 30, beta: 0.95, inset: 30, every: 8, hold: 2 };

function roamPose(t: number, b: { x0: number; z0: number; x1: number; z1: number; from: [number, number, number]; areas?: [number, number][] }): { alpha: number; beta: number; radius: number; target: [number, number, number] } {
  const moving = Math.floor(t / ROAM.every) * (ROAM.every - ROAM.hold) + Math.min(t % ROAM.every, ROAM.every - ROAM.hold);
  // A big world's: from hamlet to hamlet (its areas' middles, round and back to the first).
  if (b.areas) {
    const pts = [...b.areas, b.areas[0]];
    const lens = pts.slice(1).map((p, i) => Math.hypot(p[0] - pts[i][0], p[1] - pts[i][1]));
    let s = (ROAM.speed * moving) % lens.reduce((a, l) => a + l, 0);
    let i = 0;
    while (s > lens[i]) s -= lens[i++];
    const [a, c] = [pts[i], pts[i + 1]];
    const k = s / (lens[i] || 1);
    const [hx, hz] = [(c[0] - a[0]) / (lens[i] || 1), (c[1] - a[1]) / (lens[i] || 1)];
    return { alpha: Math.atan2(-hz, -hx), beta: ROAM.beta, radius: ROAM.radius, target: [a[0] + (c[0] - a[0]) * k, 0, a[1] + (c[1] - a[1]) * k] };
  }
  const cx = (b.x0 + b.x1) / 2;
  const cz = (b.z0 + b.z1) / 2;
  const a = Math.max(20, (b.x1 - b.x0) / 2 - ROAM.inset);
  const c = Math.max(20, (b.z1 - b.z0) / 2 - ROAM.inset);
  // Starting where the home lot is round the loop; at about `speed` m/s along it (the ellipse's mean radius).
  const start = Math.atan2((b.from[2] - cz) / c, (b.from[0] - cx) / a);
  const th = start + (ROAM.speed * moving) / ((a + c) / 2);
  const x = cx + a * Math.cos(th);
  const z = cz + c * Math.sin(th);
  // Heading along the loop; the eye behind it (the arc camera's eye is at target + (cos α, sin α) · …).
  const hx = -a * Math.sin(th);
  const hz = c * Math.cos(th);
  return { alpha: Math.atan2(-hz, -hx), beta: ROAM.beta, radius: ROAM.radius, target: [x, 0, z] };
}

/** Where every path starts (the game's own opening angle). */
const START_ALPHA = -Math.PI * 0.62;
/** Longest wait for the game to start (loading, shaders). */
const START_LIMIT_MS = 120_000;
/** Longest wait, before a picture, for the town's files to arrive and for what they hold to be built. */
const SETTLE_LIMIT_MS = 60_000;

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
  /** Main-thread work per frame: all of it, the game's update (residents, camera, bubbles…) and drawing (PlayCanvas's update and render), ms. */
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
  /** Reflections drawn a second (all mirrors). */
  reflectionsPerSecond: number;
  longTasks: { count: number; maxMs: number; totalMs: number };
  /** Frames that took longer than two at 60 Hz (33 ms between them). */
  over33?: number;
  /** Those frames: when (s into the measurement), how long (ms), and what the renderer was building then. */
  slow?: { at: number; ms: number; building: string }[];
  render: Record<string, unknown>;
  /**
   * Scene counts at the start (meshes, materials, shadow casters and their triangles, plant instances,
   * residents, mirrors: `Renderer.benchInfo`): equal fingerprints mean two runs drew the same scene.
   * The meshes and materials are PlayCanvas's draws (runs before the PlayCanvas port counted meshes): compare runs
   * since the port only.
   */
  fingerprint: string;
}

declare global {
  interface Window {
    __benchResult?: BenchResult;
    /** The scene is up and the warm-up is over (screenshots of a frozen scene are taken then). */
    __benchReady?: boolean;
  }
}

/** The game's interface hidden (`hud=0`): the top bar and the household strip, what's over the world, the toasts, the pill of what streams in. */
const HUD_HIDDEN = '.hud-root, .world-overlay, .toast-layer, .stream { display: none !important; }';

/** `hud=0` on a page that isn't the benchmark's (`?state=…&debug&hud=0`: pictures of the world for checks): the interface hidden. */
export function hideHud(): void {
  document.head.append(Object.assign(document.createElement('style'), { textContent: HUD_HIDDEN }));
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
  if (!b.hud) css.push(HUD_HIDDEN);
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
  if (b.summary) showPanel(result);
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
  // (The big world is the village's, tiled for the renderer: `tile`.)
  const saveId = await loadState(await sceneText(b.scene === 'bigworld' ? 'village' : b.scene), `bench ${b.scene}`);
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
  const town = renderer.benchInfo().world as { width: number; depth: number; areas?: [number, number][] } | undefined;
  // (`townCentre=1`: round the drawn world's main town, the town view's.)
  const main = b.orbit.town ? town?.areas?.[0] : undefined;
  const centre: [number, number, number] = main ? [main[0], 0, main[1]] : plot ? [plot.x + plot.w / 2, 0, plot.z + plot.d / 2] : [0, 0, 0];
  const path = PATHS[b.cam === 'wide' ? 'wide' : 'orbit'];
  // The roam's loop: over the town (tiled, the big world's), from the home lot's side.
  const bounds = { x0: 0, z0: 0, x1: town?.width ?? 100, z1: town?.depth ?? 100, from: centre, areas: town?.areas && town.areas.length > 1 ? town.areas : undefined };
  const frozen = b.diag.has('freeze');
  const mirror = b.cam === 'mirror' || b.cam === 'mirror-still' ? mirrorView(renderer.benchInfo().scene as { mirrorSpots?: Spot[] }, plot) : null;
  if (mirror) controls.setWallMode('up');
  else if (b.walls) controls.setWallMode(b.walls);
  // The town view: over the main town's centre or the home, in the world drawn (the bench's big world has its own).
  const spots = b.cam === 'over' || b.cam === 'zoom' ? townSpots(renderer.benchWorld(), b.big || !plot ? null : centre) : null;
  // From across the street: the eye past the lot's entry, on the line from the lot's middle through it.
  const street = b.cam === 'street' && plot ? { alpha: plot.entry ? Math.atan2(plot.entry[1] - centre[2], plot.entry[0] - centre[0]) : START_ALPHA, target: (plot.entry ? [plot.entry[0], 0, plot.entry[1]] : centre) as [number, number, number] } : null;

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
      const t = frozen || b.hold ? b.at : (now - t0) / 1000;
      if (mirror) {
        const m = MIRROR_PATH;
        const alpha = mirror.alpha + m.turn + (b.cam === 'mirror' ? m.swing * Math.sin((2 * Math.PI * t) / m.period) : 0);
        // The arc camera sits at target + radius × (cos α sin β, cos β, sin α sin β): the eye `distance` from the glass.
        const k = m.distance - m.radius;
        const s = Math.sin(m.beta);
        const g = mirror.glass;
        renderer.setCameraPose({ alpha, beta: m.beta, radius: m.radius, target: [g[0] + k * Math.cos(alpha) * s, g[1] + k * Math.cos(m.beta), g[2] + k * Math.sin(alpha) * s] });
      } else if (b.cam === 'roam') {
        renderer.setCameraPose(roamPose(t, bounds));
      } else if (spots) {
        const v = TOWN_VIEW;
        const near = DEFAULT_ORBIT.radius;
        // (Its first seconds at the opening distance, whatever the clock: the lot there becomes the focus.)
        const settling = (now - t0) / 1000 < v.settle;
        // The zoom: out and back in once a period, by the same share each second (a triangle in the radius's logarithm).
        const out = 1 - Math.abs(1 - (2 * (t % v.period)) / v.period);
        const radius = settling ? near : b.cam === 'zoom' ? near * Math.pow(TOWN_MAX / near, out) : (b.orbit.radius ?? TOWN_FULL);
        const at = b.cam === 'zoom' ? spots.home : spots[b.over];
        renderer.setCameraPose({ alpha: START_ALPHA, beta: b.orbit.beta ?? v.beta, radius, target: [at[0] + b.orbit.tx, at[1], at[2] + b.orbit.tz] });
      } else if (b.cam === 'town') {
        // (The town's own camera: `setTownShot`.)
      } else if (street) {
        renderer.setCameraPose({ alpha: street.alpha, beta: b.orbit.beta ?? STREET.beta, radius: b.orbit.radius ?? STREET.radius, target: street.target });
      } else {
        const o = b.orbit;
        renderer.setCameraPose({ alpha: START_ALPHA + (2 * Math.PI * t) / path.period, beta: o.beta ?? path.beta, radius: o.radius ?? path.radius, target: [centre[0] + o.tx, centre[1], centre[2] + o.tz] });
      }
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
    // A picture waits for the town's files and for what is built from them (a measurement keeps its timing).
    if (frozen) {
      await Promise.race([whenStreamSettled(), sleep(SETTLE_LIMIT_MS)]);
      const until = performance.now() + SETTLE_LIMIT_MS;
      while (renderer.benchInfo().building && performance.now() < until) await sleep(250);
      await renderer.framesRendered(3);
    }
    if (b.cam === 'town') await showTown(renderer, (b.hour ?? 13) * 60);
    window.__benchReady = true;
    // The measured stretch.
    const info = renderer.benchInfo();
    renderer.gpuPasses(true);
    const lighting0 = (info.scene as { lightingUpdates: number }).lightingUpdates;
    const reflections0 = (info.scene as { mirrorDraws: number }).mirrorDraws;
    const long: number[] = [];
    const observer = typeof PerformanceObserver !== 'undefined' && PerformanceObserver.supportedEntryTypes?.includes('longtask')
      ? new PerformanceObserver((list) => list.getEntries().forEach((e) => long.push(e.duration)))
      : null;
    observer?.observe({ type: 'longtask' });
    let last = 0;
    const slow: { at: number; ms: number; building: string }[] = [];
    let rafCount = 0;
    let stop = false;
    const raf = (time: number) => {
      if (stop) return;
      if (last) intervals.push(time - last);
      // (When the slow ones came, s into the measurement, and what the frame's build was then: `slow`.)
      if (last && time - last > 33.4) slow.push({ at: Math.round(time - started) / 1000, ms: Math.round(time - last), building: renderer.benchInfo().building as string });
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
      reflectionsPerSecond: ((end.scene as { mirrorDraws: number }).mirrorDraws - reflections0) / secs,
      longTasks: { count: long.length, maxMs: Math.max(0, ...long), totalMs: long.reduce((a, x) => a + x, 0) },
      over33: intervals.filter((ms) => ms > 33.4).length,
      slow,
      render: { ...info, end: end.scene, endStreet: end.street, focusSwitches: end.focusSwitches },
      fingerprint,
    };
  } finally {
    // A frozen picture stays where it is; otherwise the camera is the player's again.
    if (!frozen) renderer.setFrameProbe(null);
    renderer.measureGpu(false, 'bench');
  }
}

/**
 * `cam=town`: the menus' town of the world in play, framed as the main menu frames it (`MenuScene.baseShot`)
 * and lit for `minute`. After the warm-up, when the game's own start has let go of the town (it hides the
 * menus' as it reveals the game); a moment more for the town's models to arrive.
 */
async function showTown(renderer: Renderer, minute: number): Promise<void> {
  const world = renderer.benchWorld();
  if (!world) throw new Error('No town to show');
  const main = world.meta?.areas?.[0];
  const inTown = main ? world.plots.filter((p) => p.x + p.w / 2 >= main.x && p.x + p.w / 2 <= main.x + main.w && p.z + p.d / 2 >= main.z && p.z + p.d / 2 <= main.z + main.d) : world.plots;
  const [x0, z0] = [Math.min(...inTown.map((p) => p.x)), Math.min(...inTown.map((p) => p.z))];
  const focus = { x: x0, z: z0, w: Math.max(...inTown.map((p) => p.x + p.w)) - x0, d: Math.max(...inTown.map((p) => p.z + p.d)) - z0 };
  await renderer.showTown(world, minute);
  renderer.setTownShot({ focus, ...TOWN, sway: 0, drift: 0, duration: 0, minute }, true);
  await sleep(3000);
  if (!renderer.townShown) throw new Error('The town did not stay on screen');
}

type Spot = { x: number; y: number; z: number; nx: number; ny: number; nz: number };

/** The mirror camera's aim: the first mirror on the home lot (or anywhere), its glass's centre and the turn that faces it. */
function mirrorView(scene: { mirrorSpots?: Spot[] }, plot: { x: number; z: number; w: number; d: number } | null): { alpha: number; glass: [number, number, number] } {
  const spots = scene.mirrorSpots ?? [];
  const home = spots.find((s) => !plot || (s.x >= plot.x && s.x <= plot.x + plot.w && s.z >= plot.z && s.z <= plot.z + plot.d)) ?? spots[0];
  if (!home) throw new Error('No mirror in the scene');
  // (α along the mirror's normal: the camera in front of the glass.)
  return { alpha: Math.atan2(home.nz, home.nx), glass: [home.x, home.y, home.z] };
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
    reflectionsPerSecond: 0,
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
