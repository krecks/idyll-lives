/**
 * Bench time: a game state swapped into the page as it stands, and played in fixed steps, so that a benchmark counts
 * what exactly the same frames of the same moment draw, without waiting for the game's time to pass and without
 * repeats to even out the wall clock. On `window.__bench` (`bench.ts`), on `?bench=` pages and the `?debug` pages (from
 * the main menu on):
 *
 * - `load(state)`: the game in this page replaced by `state` (a game state's JSON, or the name of one: in
 *   `debug-states/` on the dev server, else a scene in `public/bench/`), as Continue starts a saved game, without a page
 *   load; resolves once the game is up and what it streams in has come (`settle: false`: once it is up). Its timings.
 * - `step(ticks)`: fixed-step playback (`GameSession.stepMode`): the simulation's clock stops, and every frame drawn
 *   from then on runs exactly `ticks` ticks first (1 at normal speed: a 60th of a second of the game's time, the
 *   game held at normal speed); `step(null)` goes back to real time.
 * - `run(frames)`: draws exactly `frames` frames in fixed steps (`step(1)` first if it isn't on), as fast as the
 *   machine draws them, and gives each frame's draws (all, the sun's map's, the mirrors'), triangles, whether the sun's
 *   picture of the still casters was drawn again and why (`SunShadows.draws`), with their sums. The bench's own
 *   helpers live here too (the town's centre, the start's angle and limits, the sun's redraws, a state's text).
 * - `state()`: the game's state as it is now (JSON), to load again.
 *
 * The same state and the same frames give the same counts on any machine, however busy: what is drawn follows the
 * game's ticks and the camera, not the time a frame took. What a frame costs in time is still the wall clock's (the
 * real-time bench, `bench.ts`).
 */

import type { WorldStructure } from '../core/protocol';
import { within } from '../core/async';
import { host } from '../game/host';
import type { GameSession } from '../game/session';
import { deleteSave } from '../persistence/saves';
import type { CameraPose, FrameCounts, Renderer } from '../render/types';
import { templateBuilds } from '../render/shared/templateCache';
import { DEFAULT_ORBIT } from '../render/shared/lens';
import { plotInfo } from '../ui/place';
import { game } from '../ui/state.svelte';
import { settings } from '../settings/settings.svelte';
import { app } from '../ui/app.svelte';
import { services } from '../ui/services';
import { loadState } from './report';

/** Longest wait for a game loaded to come up (loading, shaders), and for what it streams in to come and be built. */
export const START_LIMIT_MS = 120_000;
export const SETTLE_LIMIT_MS = 60_000;
/** Where every bench camera starts from: the game's own opening angle. */
export const START_ALPHA = -Math.PI * 0.62;

export const sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

/** The sun's map's redraws as the renderer counts them (`benchInfo().scene.sunShadowDraws`, `SunShadows.draws`). */
export interface SunDraws {
  /** With the still casters' picture drawn again, and with only what moves over it. */
  whole: number;
  moving: number;
  /** Why the picture was drawn again (the check that failed), and on which draws. */
  why: Record<string, number>;
  by: Record<string, number>;
}

/** The sun's map's redraws so far. */
export function sunDraws(renderer: Renderer): SunDraws {
  return (renderer.benchInfo().scene as { sunShadowDraws?: SunDraws }).sunShadowDraws ?? { whole: 0, moving: 0, why: {}, by: {} };
}

/** Each count of `end` less its count in `start`, the ones that grew. */
export function since(end: Record<string, number>, start: Record<string, number>): Record<string, number> {
  const out: Record<string, number> = {};
  for (const [k, n] of Object.entries(end)) if (n > (start[k] ?? 0)) out[k] = n - (start[k] ?? 0);
  return out;
}

/** The sun's map's redraws from `start` to `end`. */
export function sunSince(end: SunDraws, start: SunDraws): SunDraws {
  return { whole: end.whole - start.whole, moving: end.moving - start.moving, why: since(end.why, start.why), by: since(end.by, start.by) };
}

/** The main town's centre in `world`: its square once the world has one (`meta.centre`), else the middle of its area, else the world's. */
export function townCentre(world: WorldStructure): [number, number, number] {
  const main = world.meta?.areas?.[0];
  const square = world.meta?.centre?.find((c) => c.area === 0) ?? world.meta?.centre?.[0];
  const mid = (pts: [number, number][]): [number, number] => [pts.reduce((a, p) => a + p[0], 0) / pts.length, pts.reduce((a, p) => a + p[1], 0) / pts.length];
  const [cx, cz] = square?.poly.length ? mid(square.poly) : main ? [main.x + main.w / 2, main.z + main.d / 2] : [world.width / 2, world.depth / 2];
  return [cx, 0, cz];
}

/**
 * The benchmark's state `name` in `public/bench/` (`<name>.json.gz`: a whole town's save is half a megabyte of JSON,
 * a tenth of that gzipped; or a plain `.json`), or null.
 */
export async function benchState(name: string): Promise<string | null> {
  const base = `${import.meta.env.BASE_URL}bench/${name}.json`;
  const packed = await fetch(`${base}.gz`);
  if (packed.ok) {
    // (Served as a file, with no Content-Encoding: unpacked here, unless the server already did.)
    const bytes = new Uint8Array(await packed.arrayBuffer());
    if (bytes[0] !== 0x1f || bytes[1] !== 0x8b) return new TextDecoder().decode(bytes);
    return new Response(new Blob([bytes]).stream().pipeThrough(new DecompressionStream('gzip'))).text();
  }
  const plain = await fetch(base);
  return plain.ok ? plain.text() : null;
}

/** What `load` took (ms from the call): the game up (its interface shown), and what it streams in come and built. */
export interface Loaded {
  tick: number;
  upMs: number;
  settledMs: number;
}

/** What `run` counted, frame by frame (the arrays) and in all. */
export interface StepRun {
  frames: number;
  /** Ticks run (fewer than frames × the ticks a frame if the game was paused). */
  ticks: number;
  /** The wall clock's ms for it all. */
  ms: number;
  draws: number[];
  shadowDraws: number[];
  mirrorDraws: number[];
  triangles: number[];
  /** The frames (from 0) in which the sun's picture of the still casters was drawn again, and why (the check that failed). */
  sunStill: { frame: number; why: string[] }[];
  /** The sun's map drawn over the run: with the still casters' picture drawn again, and with only what moves. */
  sunShadows: SunDraws;
  /** Draws a frame: the mean, and the least and most. */
  drawsMean: number;
  drawsMin: number;
  drawsMax: number;
  /** A fingerprint of the frames' counts (FNV-1a of draws, shadow draws and triangles): two runs that drew the same have the same. */
  hash: string;
  /** What the renderer was building when the run started and ended ('' for nothing): a run over a build may not repeat. */
  building: [string, string];
  /**
   * The frames that asked for a model template not built yet (a prop first held, a thing first bought): drawn without
   * it, the frames after with it. A page that built it before draws those frames with it: compare runs of warm pages
   * (the steps tool's first run of a pose is a warm-up).
   */
  lateBuilds: number[];
}

/** The game being played, or an error. */
function session(): GameSession {
  const s = host.session;
  if (!s) throw new Error('No game is being played.');
  return s;
}

/** A state's JSON: as given, or by name from the dev server's `debug-states/` or the benchmark's scenes. */
async function stateText(state: string): Promise<{ text: string; name: string }> {
  if (state.trimStart().startsWith('{')) return { text: state, name: 'bench time' };
  if (import.meta.env.DEV) {
    const res = await fetch(`/__debug/states/${encodeURIComponent(state)}`);
    if (res.ok) return { text: await res.text(), name: state };
  }
  const text = await benchState(state);
  if (text === null) throw new Error(`No game state "${state}" (debug-states/ or public/bench/)`);
  return { text, name: state };
}

/**
 * Most frames `run` draws, the game paused and the clock held (`GameSession.step`'s `still`), for what a camera put
 * somewhere streams in and builds; and how long (ms of the wall clock, and frames) they must draw the same with nothing
 * being built to end it: a resident's look or a model that comes later shows as a frame that draws more.
 */
const SETTLE_FRAMES = 2000;
const STEADY_MS = 3000;
const STEADY_FRAMES = 10;
/** The wall clock's pause between those frames, for what loads meanwhile. */
const SETTLE_PAUSE_MS = 25;

/**
 * Where the camera is held for a run: `over` the home or the main town's centre (its square once the world has one,
 * else the middle of its area, as the benchmark's town view finds it: `townCentre`) at `radius` m, from the game's opening angle;
 * or a pose as it is.
 */
export type RunPose = CameraPose | { over: 'home' | 'centre'; radius: number; beta?: number; alpha?: number };

async function poseOf(pose: RunPose): Promise<CameraPose> {
  if (!('over' in pose)) return pose;
  const renderer = await host.rendererNow(settings);
  let at: [number, number, number];
  if (pose.over === 'home') {
    const home = plotInfo(game, game.households.find((h) => h.player)?.plot);
    if (!home) throw new Error('No home to look at.');
    at = [home.x + home.w / 2, 0, home.z + home.d / 2];
  } else {
    const world = renderer.benchWorld();
    if (!world) throw new Error('No world to look at.');
    at = townCentre(world);
  }
  return { alpha: pose.alpha ?? START_ALPHA, beta: pose.beta ?? DEFAULT_ORBIT.beta, radius: pose.radius, target: at };
}

/** FNV-1a over a few arrays of whole numbers, as hex. */
function fingerprint(...lists: number[][]): string {
  let h = 0x811c9dc5;
  for (const list of lists) {
    for (const n of list) {
      h ^= n | 0;
      h = Math.imul(h, 0x01000193) >>> 0;
    }
  }
  return h.toString(16).padStart(8, '0');
}

export const benchTime = {
  /** `name`: the save's while it starts (its name, else `bench time`). */
  async load(state: string, options: { settle?: boolean; name?: string } = {}): Promise<Loaded> {
    const t0 = performance.now();
    const { text, name } = await stateText(state);
    const before = host.session;
    before?.stepMode(null);
    // Continue's way into a saved game: the state as a save, started over whatever was played.
    const { saveId, tick } = await loadState(text, options.name ?? name);
    app.start({ kind: 'load', saveId });
    const until = t0 + START_LIMIT_MS;
    while (!(app.screen === 'game' && !app.loading && host.session && host.session !== before && services.controls && document.querySelector('.hud-root'))) {
      if (performance.now() > until) throw new Error('The game did not start');
      await sleep(50);
    }
    // (It was copied into a save slot to start it; it isn't the player's.)
    await deleteSave(saveId).catch(() => {});
    const upMs = performance.now() - t0;
    if (options.settle !== false) {
      // What the renderer streams in and builds (the town's files, the street round the camera, a world), for a while at most.
      const renderer = await host.rendererNow(settings);
      await renderer.whenSettled(SETTLE_LIMIT_MS);
      await renderer.framesRendered(3);
    }
    return { tick, upMs, settledMs: performance.now() - t0 };
  },

  /** Fixed-step playback at `ticks` a frame (the game at normal speed first, unless `speed` says), or back to real time (null). */
  step(ticks: number | null, speed = 1): void {
    const s = session();
    if (ticks !== null) services.controls.setSpeed(speed);
    s.stepMode(ticks);
  },

  /**
   * `frames` frames in fixed steps of `ticks`. With `pose` (or `settle`, the camera as it is held), the camera is put
   * there first and frames are drawn with the game paused and the clock held until the picture stands still
   * (`SETTLE_FRAMES` at most), then the game goes on at normal speed (`speed`) for the run. A camera that moves with
   * the wall clock (the bench's orbit paths) doesn't repeat: hold it (`cam=over`, `hold=1`).
   */
  async run(frames: number, options: { ticks?: number; pose?: RunPose; settle?: boolean; speed?: number } = {}): Promise<StepRun & { settleFrames: number }> {
    const s = session();
    const ticks = options.ticks ?? 1;
    const renderer = await host.rendererNow(settings);
    s.stepMode(ticks);
    let settleFrames = 0;
    if (options.pose || options.settle) {
      services.controls.setSpeed(0);
      if (options.pose) renderer.setCameraPose(await poseOf(options.pose));
      // (The clock held: however many frames the builds take, what is held back by its rate is due at the same frames of
      // the run. The frames drawn until they draw the same for a while with nothing built: in fixed steps a frame is
      // drawn only when asked for, and what a frame asks for as it draws (a resident's look, a model, the street round
      // the camera) comes in over the frames after it, so `Renderer.whenSettled` alone, between frames, can't tell.)
      let drawn = -1;
      const remove = renderer.addFrameProbe({
        before() {},
        after(_start, _updated, _end, c) {
          drawn = c.drawCalls + c.triangles;
        },
      });
      try {
        let steadyFrom = performance.now();
        let steadyFrames = 0;
        let last = -2;
        while (settleFrames < SETTLE_FRAMES) {
          await s.step(true);
          await within(templateBuilds().settled(), SETTLE_LIMIT_MS, undefined);
          settleFrames++;
          if (drawn !== last || renderer.building()) {
            last = drawn;
            steadyFrom = performance.now();
            steadyFrames = 0;
          } else if (++steadyFrames >= STEADY_FRAMES && performance.now() - steadyFrom >= STEADY_MS) break;
          await sleep(SETTLE_PAUSE_MS);
        }
      } finally {
        remove();
      }
    }
    // (However the frames before went and however many settling took, the run starts alike.)
    await s.restartPlayback();
    services.controls.setSpeed(options.speed ?? 1);
    const draws: number[] = [];
    const shadowDraws: number[] = [];
    const mirrorDraws: number[] = [];
    const triangles: number[] = [];
    const sunStill: StepRun['sunStill'] = [];
    const lateBuilds: number[] = [];
    // (The counts the frame before the run left: the run's sums start from them, and each frame's redraw from the last.)
    const from = sunDraws(renderer);
    let last = { whole: from.whole, why: from.why };
    // (Added after the bench's camera's hooks, which go on holding the camera.)
    const remove = renderer.addFrameProbe({
      before() {},
      after(_start, _updated, _end, c: FrameCounts) {
        const now = c.sunShadows;
        if (now.whole > last.whole) {
          const was = last.why;
          sunStill.push({ frame: draws.length, why: Object.keys(now.why).filter((k) => now.why[k] > (was[k] ?? 0)) });
        }
        last = { whole: now.whole, why: { ...now.why } };
        draws.push(c.drawCalls);
        shadowDraws.push(c.shadowDrawCalls);
        mirrorDraws.push(c.mirrorDrawCalls);
        triangles.push(c.triangles);
      },
    });
    const building0 = renderer.building();
    const t0 = performance.now();
    let ran = 0;
    try {
      for (let i = 0; i < frames; i++) {
        const builds = templateBuilds().started;
        ran += await s.step();
        // (A frame that asked for a template not built yet drew without it; the next ones wait for it.)
        if (templateBuilds().started !== builds) {
          lateBuilds.push(i);
          await within(templateBuilds().settled(), SETTLE_LIMIT_MS, undefined);
        }
      }
    } finally {
      remove();
    }
    const ms = performance.now() - t0;
    return {
      frames: draws.length,
      ticks: ran,
      ms,
      draws,
      shadowDraws,
      mirrorDraws,
      triangles,
      sunStill,
      sunShadows: sunSince(sunDraws(renderer), from),
      drawsMean: draws.reduce((a, n) => a + n, 0) / Math.max(1, draws.length),
      drawsMin: Math.min(...draws),
      drawsMax: Math.max(...draws),
      hash: fingerprint(draws, shadowDraws, triangles),
      building: [building0, renderer.building()],
      lateBuilds,
      settleFrames,
    };
  },

  state(): Promise<string> {
    return services.controls.currentState();
  },
};
