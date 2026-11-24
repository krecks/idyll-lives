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
 *   Build mode, paused), `dusk` (orbit at 10×, speed 3, from the scene's 18:00, the light changing every frame),
 *   `mirror` (the home lot's first mirror from 4.4 m, from over its room with the walls up, the camera
 *   swinging round the glass every 20 s), `mirror-still` (the same, held still in the swing's middle),
 *   `street` (the home lot from across its street, low: the house's front, the pavement, the road),
 *   `town` (the menus' town from the menu's own angle, lit for `hour`),
 *   `over` (the town view: the game's camera held `radius=` m out, 400 by default,
 *   over the main town's centre or the home, `over=centre|home`), `zoom` (the one zoom from the home at the
 *   game's opening distance out to 900 m and back in, once in 60 s).
 * - `townCentre=1`: the orbit round the drawn world's main town (the town view) instead of the home lot.
 * - `radius=`, `beta=`, `tx=`, `tz=`: the orbit's distance, angle and target instead of its path's (the game
 *   opens at beta 0.86); `walls=up|cutaway|down`: the walls held that way (up: the house whole, roof on).
 * - `res=1080` that resolution (720, 1080, 1440, 2160; `res=1` every line the window has); `res=auto` (default) the player's setting.
 * - `secs=20` measured after `warmup=5`; `cap=0` frames per second at most (0: every frame shown).
 * - `hud=0` hides the interface; `gpu=0` doesn't time the GPU; `label=…` is kept with the result.
 * - `diag=noshadows,noaa,nonature,nolamps,nomirrors,noglass,nonight,noao,nobloom,noclouds,nolayers,noflow,freeze`: features
 *   left out (to measure what each costs; `nowoods`, `nofarwoods`, `noflowers` and `nograss` leave out
 *   one part of the nature; `noflow` draws the water still and plain, as before it flowed); `freeze` stops
 *   time for screenshots (`at=<s>` along the path);
 *   `shaders` logs every shader and pipeline made (the warm-up should leave none for play).
 *
 * The result lands in `window.__benchResult` (with the dev server also in `bench-results/`) and in
 * a small panel to copy. `tools/bench/render.mjs` runs a whole matrix of them. Once the warm-up is over the page
 * answers `window.__bench` (`BenchApi`): other poses, hours and counts on the same page, without a reload.
 */

import { host } from '../game/host';
import type { WorldStructure } from '../core/protocol';
import { within } from '../core/async';
import { DEFAULT_ORBIT, TOWN_FULL, TOWN_MAX } from '../render/shared/lens';
import type { FrameCounts, FrameProbe, Renderer, RenderPassInfo } from '../render/types';
import { settings, unsavedSettings } from '../settings/settings.svelte';
import { services } from '../ui/services';
import { game } from '../ui/state.svelte';
import { plotInfo } from '../ui/place';
import { BENCH, type BenchCamera, type BenchFlags } from './benchFlags';
import { gpuInfo } from './report';
import { benchState, benchTime, SETTLE_LIMIT_MS, sleep, START_ALPHA, sunDraws, sunSince, townCentre, type Loaded, type RunPose, type StepRun, type SunDraws } from './benchTime';

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
  const centre = townCentre(world);
  const [cx, , cz] = centre;
  let at = home;
  if (!at) {
    const inMain = (p: { x: number; z: number; w: number; d: number }) => !main || (p.x + p.w / 2 >= main.x && p.x + p.w / 2 <= main.x + main.w && p.z + p.d / 2 >= main.z && p.z + p.d / 2 <= main.z + main.d);
    const away = (p: { x: number; z: number; w: number; d: number }) => Math.hypot(p.x + p.w / 2 - cx, p.z + p.d / 2 - cz);
    const homes = world.plots.filter((p) => p.house && inMain(p)).sort((a, b) => away(a) - away(b) || a.id - b.id);
    const p = homes[Math.floor(homes.length * 0.75)];
    at = p ? [p.x + p.w / 2, 0, p.z + p.d / 2] : [cx, 0, cz];
  }
  return { centre, home: at };
}
/** The camera's pose as `?bench=`'s flags give it (`benchFlags.ts`), and as `__bench.pose` changes it. */
export interface BenchView {
  cam: BenchCamera;
  /** Seconds along the path (held, with `freeze` or `hold`). */
  at: number;
  hold: boolean;
  orbit: BenchFlags['orbit'];
  /** The orbit's turn instead of the path's at `at` (rad; the game opens at about -1.95). */
  alpha: number | null;
  over: BenchFlags['over'];
  walls: BenchFlags['walls'];
}

/**
 * `__bench.pose`: a whole pose, as a URL's flags would give it; what it leaves out is the flags' default (an orbit from
 * the path's start, over the home, the game's own walls), so one pose never carries over into the next.
 */
export interface BenchPose {
  cam?: BenchCamera;
  at?: number;
  radius?: number | null;
  beta?: number | null;
  alpha?: number | null;
  tx?: number;
  tz?: number;
  over?: 'centre' | 'home';
  walls?: 'up' | 'cutaway' | 'down' | null;
  townCentre?: boolean;
}

/** What `__bench.counts` read over its frames: sums and per-frame means of the frame probe's counts, and the sun's map. */
export interface BenchCounts {
  frames: number;
  sum: { draws: number; shadowDraws: number; mirrorDraws: number; triangles: number; shadowTriangles: number; mirrorTriangles: number };
  mean: { draws: number; shadowDraws: number; mirrorDraws: number; triangles: number; shadowTriangles: number; mirrorTriangles: number };
  /** The middle frame's draws (sorted). */
  medianDraws: number;
  /** The sun's map drawn again over those frames: whole (the still casters too), only what moves, and why (`SunShadows.draws`). */
  sun: SunDraws;
}

/**
 * The page's own bench, without a reload (`window.__bench`): one page, many poses. `render.mjs --shots` steps a page
 * through its poses with it (`tools/bench/lib/browser.mjs` `step`), and a script can probe as it likes:
 * `await __bench.hour(19.5); __bench.pose({ cam: 'over', over: 'centre', radius: 400 }); await __bench.settle(); …`.
 */
export interface BenchApi {
  /** The camera held to `pose` (its target jumping to another spot of the town view stays near for a moment first: `settle`). */
  pose(pose: BenchPose): void;
  /** The light held at `hour` (null: the game's clock), lit at once; resolves once the birds have settled for it. */
  hour(hour: number | null): Promise<void>;
  /** Time stood still (as `diag=freeze`: the game paused, frames advance nothing) or going again at the bench's speed. */
  freeze(on: boolean): void;
  /**
   * Resolves once the picture is ready to take: the town view's moment over a new spot passed (its lot the focus), the
   * files and the street streamed in, nothing building, the textures there, the birds settled and three frames drawn.
   */
  settle(options?: { limitMs?: number }): Promise<void>;
  /** The draws and triangles of the next `frames` frames, and the sun's map's redraws over them. */
  counts(frames: number): Promise<BenchCounts>;
  /** The walls held that way (null: the game's own). */
  walls(mode: 'up' | 'cutaway' | 'down' | null): void;
  /** The pose held now. */
  view(): BenchView;
  /** The camera the player's (and `__camera`'s) again, until the next `pose`. */
  release(): void;
  /** Back to the page's own pose, hour and freeze (its URL's flags): what a script attached to a kept page leaves. */
  reset(): Promise<void>;
  /** Nothing drawn while `on` (a page kept loaded beside the one measured: `keep.mjs`), or drawing again. */
  idle(on: boolean): void;
  /**
   * Bench time (`benchTime.ts`): the game in this page replaced by a game state (its JSON, or a name in `debug-states/`
   * or `public/bench/`) as Continue opens a save, without a page load; resolves once it is up and settled.
   */
  load(state: string, options?: { settle?: boolean }): Promise<Loaded>;
  /** Fixed-step playback at `ticks` a frame (null: real time again). */
  step(ticks: number | null): void;
  /**
   * Exactly `frames` frames in fixed steps from where the game is, with the pose held (`__bench.pose` first, or
   * `pose`: the camera let go by the bench's and put over the home or the centre, or a pose as it is): frames drawn
   * with the clock held until the picture stands still, then each frame's counts. The same state, pose and frames
   * give the same counts and pictures, however busy the machine.
   */
  run(frames: number, options?: { ticks?: number; pose?: RunPose }): Promise<StepRun & { settleFrames: number }>;
  /** The game's state as it is now (JSON). */
  state(): Promise<string>;
}

/**
 * How long `settle` gives a new hour or a town view come out to: the far houses' windows follow who is up within half a
 * second (`findTownUp`), and the street's buildings' windows are drawn again at most once a second (`street.ts` `WINDOWS_EVERY_MS`).
 */
const HOUR_SETTLE_MS = 1600;

/** Where a view's camera goes (`BenchRig.placeOf`). */
interface BenchPlace {
  centre: [number, number, number];
  path: (typeof PATHS)[keyof typeof PATHS];
  bounds: Parameters<typeof roamPose>[1];
  mirror: { alpha: number; glass: [number, number, number] } | null;
  spots: { centre: [number, number, number]; home: [number, number, number] } | null;
  street: { alpha: number; target: [number, number, number] } | null;
}


/**
 * The bench's camera: the frame probe that holds the camera to `view` every frame (as the URL's flags have it, or as
 * `__bench.pose` changed it), and the counts it gathers for `__bench.counts`.
 */
class BenchRig {
  view: BenchView;
  frozen: boolean;
  /** Pending birds (`__bench.hour`), for `settle`, and when the hour was last changed. */
  birds: Promise<void> = Promise.resolve();
  hourAt = 0;
  /** When `settle` last finished. */
  settledAt = 0;
  private readonly t0 = performance.now();
  /** When the camera began its moment near over the town view's spot (`TOWN_VIEW.settle`). */
  private focusFrom = performance.now();
  private place: BenchPlace;
  private measured: FrameProbe['after'] | null = null;
  private counting: { left: number; draws: number[]; sum: BenchCounts['sum']; done: () => void } | null = null;
  /** Takes the rig's frame probe off (null: not on). */
  private detach: (() => void) | null = null;
  /** The walls as the game had them before the bench held them. */
  private readonly ownWalls = game.wallMode;

  constructor(
    readonly renderer: Renderer,
    readonly flags: BenchFlags,
  ) {
    this.view = { cam: flags.cam, at: flags.at, hold: flags.hold, orbit: { ...flags.orbit }, alpha: null, over: flags.over, walls: flags.walls };
    this.frozen = flags.diag.has('freeze');
    this.place = this.placeOf(this.view);
    if (this.place.mirror) services.controls.setWallMode('up');
    else if (this.view.walls) services.controls.setWallMode(this.view.walls);
  }

  /** Where the view's camera goes: round what, the town view's spots, the street's eye, the mirror's glass. */
  private placeOf(view: BenchView): BenchPlace {
    const renderer = this.renderer;
    const home = game.households.find((h) => h.player)?.plot;
    const plot = plotInfo(game, home) ?? null;
    const town = renderer.benchInfo().world as { width: number; depth: number; areas?: [number, number][] } | undefined;
    // (`townCentre=1`: round the drawn world's main town, the town view's.)
    const main = view.orbit.town ? town?.areas?.[0] : undefined;
    const centre: [number, number, number] = main ? [main[0], 0, main[1]] : plot ? [plot.x + plot.w / 2, 0, plot.z + plot.d / 2] : [0, 0, 0];
    const path = PATHS[view.cam === 'wide' ? 'wide' : 'orbit'];
    // The roam's loop: over the town (tiled, the big world's), from the home lot's side.
    const bounds = { x0: 0, z0: 0, x1: town?.width ?? 100, z1: town?.depth ?? 100, from: centre, areas: town?.areas && town.areas.length > 1 ? town.areas : undefined };
    const mirror = view.cam === 'mirror' || view.cam === 'mirror-still' ? mirrorView(renderer.benchInfo().scene as { mirrorSpots?: Spot[] }, plot) : null;
    // The town view: over the main town's centre or the home, in the world drawn (the bench's big world has its own).
    const spots = view.cam === 'over' || view.cam === 'zoom' ? townSpots(renderer.benchWorld(), this.flags.big || !plot ? null : centre) : null;
    // From across the street: the eye past the lot's entry, on the line from the lot's middle through it.
    const street = view.cam === 'street' && plot ? { alpha: plot.entry ? Math.atan2(plot.entry[1] - centre[2], plot.entry[0] - centre[0]) : START_ALPHA, target: (plot.entry ? [plot.entry[0], 0, plot.entry[1]] : centre) as [number, number, number] } : null;
    return { centre, path, bounds, mirror, spots, street };
  }

  /** The town view's pose as a key (null: another camera). */
  private townPose(): string | null {
    const at = this.townTarget();
    return at && `${this.view.cam} ${at.join(' ')} ${this.view.orbit.radius} ${this.view.orbit.beta} ${this.view.alpha}`;
  }

  /** The town view's target (null: another camera). */
  private townTarget(): [number, number, number] | null {
    const { spots } = this.place;
    if (!spots) return null;
    const at = this.view.cam === 'zoom' ? spots.home : spots[this.view.over];
    return [at[0] + this.view.orbit.tx, at[1], at[2] + this.view.orbit.tz];
  }

  /** The probe on the renderer, with `after` (the measurement's) besides the counts. */
  probe({ after }: Pick<FrameProbe, 'after'>): void {
    this.measured = after;
    this.attach();
  }

  private attach(): void {
    this.detach ??= this.renderer.addFrameProbe({ before: (now) => this.before(now), after: (s, u, e, c) => this.after(s, u, e, c) });
  }

  /** The camera the player's again. */
  release(): void {
    this.detach?.();
    this.detach = null;
    this.measured = null;
  }

  setView(pose: BenchPose): void {
    const was = this.townPose();
    const cam = pose.cam ?? 'orbit';
    this.view = {
      cam,
      at: pose.at ?? 0,
      hold: this.flags.hold,
      orbit: { radius: pose.radius ?? null, beta: pose.beta ?? null, tx: pose.tx ?? 0, tz: pose.tz ?? 0, town: pose.townCentre ?? false },
      alpha: pose.alpha ?? null,
      over: pose.over ?? 'home',
      walls: pose.walls ?? null,
    };
    this.place = this.placeOf(this.view);
    this.walls(this.place.mirror ? 'up' : this.view.walls);
    // Another town view (another spot, distance or tilt, or from another camera): first a moment near over its spot, as a
    // page opened there has, so the picture is the same as that page's (what was streamed and built for the last
    // distance and spot otherwise stays: other far chunks, another focus lot).
    const now = this.townPose();
    if (now && now !== was) this.focusFrom = performance.now();
    this.attach();
  }

  walls(mode: BenchView['walls']): void {
    this.view.walls = mode;
    // (The game being played now: `load` may have swapped it.)
    services.controls.setWallMode(mode ?? this.ownWalls);
  }

  /** When the town view's moment near over its spot is over (ms, performance clock; 0: no town view). */
  focusUntil(): number {
    return this.place.spots ? this.focusFrom + TOWN_VIEW.settle * 1000 : 0;
  }

  count(frames: number): Promise<{ draws: number[]; sum: BenchCounts['sum'] }> {
    this.attach();
    return new Promise((resolve) => {
      const c = { left: Math.max(1, Math.round(frames)), draws: [] as number[], sum: { draws: 0, shadowDraws: 0, mirrorDraws: 0, triangles: 0, shadowTriangles: 0, mirrorTriangles: 0 }, done: () => resolve(c) };
      this.counting = c;
    });
  }

  private before(now: number): void {
    const renderer = this.renderer;
    const b = this.view;
    const { mirror, bounds, spots, street, centre, path } = this.place;
    const t = this.frozen || b.hold ? b.at : (now - this.t0) / 1000;
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
      // (Its first seconds at the opening distance over the spot, whatever the clock: the lot there becomes the focus.)
      const settling = now < this.focusUntil();
      // The zoom: out and back in once a period, by the same share each second (a triangle in the radius's logarithm).
      const out = 1 - Math.abs(1 - (2 * (t % v.period)) / v.period);
      const radius = settling ? near : b.cam === 'zoom' ? near * Math.pow(TOWN_MAX / near, out) : (b.orbit.radius ?? TOWN_FULL);
      renderer.setCameraPose({ alpha: b.alpha ?? START_ALPHA, beta: b.orbit.beta ?? v.beta, radius, target: this.townTarget()! });
    } else if (b.cam === 'town') {
      // (The town's own camera: `setTownShot`.)
    } else if (street) {
      renderer.setCameraPose({ alpha: b.alpha ?? street.alpha, beta: b.orbit.beta ?? STREET.beta, radius: b.orbit.radius ?? STREET.radius, target: street.target });
    } else {
      const o = b.orbit;
      renderer.setCameraPose({ alpha: b.alpha ?? START_ALPHA + (2 * Math.PI * t) / path.period, beta: o.beta ?? path.beta, radius: o.radius ?? path.radius, target: [centre[0] + o.tx, centre[1], centre[2] + o.tz] });
    }
  }

  private after(start: number, updated: number, end: number, c: FrameCounts): void {
    this.measured?.(start, updated, end, c);
    const n = this.counting;
    if (!n) return;
    n.draws.push(c.drawCalls);
    n.sum.draws += c.drawCalls;
    n.sum.shadowDraws += c.shadowDrawCalls;
    n.sum.mirrorDraws += c.mirrorDrawCalls;
    n.sum.triangles += c.triangles;
    n.sum.shadowTriangles += c.shadowTriangles;
    n.sum.mirrorTriangles += c.mirrorTriangles;
    if (--n.left > 0) return;
    this.counting = null;
    n.done();
  }
}

/** `window.__bench` over the bench's camera. */
function benchApi(camera: BenchRig): BenchApi {
  const { renderer, flags } = camera;
  return {
    pose: (pose) => camera.setView(pose),
    hour(hour) {
      camera.hourAt = performance.now();
      camera.birds = renderer.setFixedHour(hour).catch((err: unknown) => console.warn('[bench] the birds failed to settle', err));
      return camera.birds;
    },
    freeze(on) {
      camera.frozen = on;
      renderer.setFrozen(on);
      // (The game being played now: `load` may have swapped it.)
      services.controls.setSpeed(on ? 0 : flags.speed || 1);
    },
    async settle({ limitMs = SETTLE_LIMIT_MS } = {}) {
      const until = performance.now() + limitMs;
      const left = () => Math.max(0, until - performance.now());
      // (The probe poses the camera at the next frame; the town view first stays near over a new spot.)
      await renderer.framesRendered(2);
      const focus = camera.focusUntil() - performance.now();
      if (focus > 0) await sleep(Math.min(focus, left()));
      // (A new hour, or the camera out from its moment near: the far houses with someone up are looked for twice a second
      // while the town view is in, and their windows lit then.)
      if (camera.hourAt > camera.settledAt || focus > 0) {
        await renderer.framesRendered(2);
        await sleep(Math.min(HOUR_SETTLE_MS, left()));
      }
      await renderer.whenSettled(left());
      await within(renderer.texturesReady(), left(), undefined);
      await within(camera.birds, left(), undefined);
      await renderer.framesRendered(3);
      camera.settledAt = performance.now();
    },
    async counts(frames) {
      const sun0 = sunDraws(renderer);
      const { draws, sum } = await camera.count(frames);
      const sun = sunDraws(renderer);
      const n = draws.length;
      const mean = Object.fromEntries(Object.entries(sum).map(([k, v]) => [k, v / n])) as BenchCounts['mean'];
      return { frames: n, sum, mean, medianDraws: [...draws].sort((a, z) => a - z)[Math.floor(n / 2)], sun: sunSince(sun, sun0) };
    },
    walls: (mode) => camera.walls(mode),
    view: () => structuredClone(camera.view),
    release: () => camera.release(),
    idle: (on) => renderer.setActive(!on),
    ...timeApi(() => camera.release()),
    async reset() {
      const f = flags;
      camera.setView({ cam: f.cam, at: f.at, radius: f.orbit.radius, beta: f.orbit.beta, tx: f.orbit.tx, tz: f.orbit.tz, over: f.over, walls: f.walls, townCentre: f.orbit.town });
      this.freeze(f.diag.has('freeze'));
      await this.hour(f.hour);
    },
  };
}

/** A `?debug` page's bench: the renderer its games draw with (`installDebugBench`), and its camera once asked for. */
let debugBench: { renderer: Renderer; camera: BenchRig | null; made: BenchApi | null } | null = null;

/**
 * `?debug` pages (`?state=…&debug`, or from the main menu on: bench time loads a state there): `window.__bench`
 * without the benchmark, put up once; its camera the player's until the first `pose` (the bench's defaults otherwise:
 * an orbit round the home lot, held at the path's start), and held through the games loaded after it, as a bench
 * page's is. A game started gives it its `renderer` (the camera's and the counts' need one).
 */
export function installDebugBench(renderer?: Renderer): void {
  if (renderer && debugBench?.renderer !== renderer) {
    debugBench?.camera?.release();
    debugBench = { renderer, camera: null, made: null };
  }
  if (window.__bench) return;
  const flags: BenchFlags = { scene: '', tile: 1, big: null, options: {}, quality: null, hour: null, cam: 'orbit', res: 'auto', secs: 0, warmup: 0, cap: 0, speed: 1, hud: true, summary: false, gpu: false, at: 0, hold: true, orbit: { radius: null, beta: null, tx: 0, tz: 0, town: false }, over: 'home', walls: null, diag: new Set(), label: '' };
  const api = (): BenchApi => {
    const d = debugBench;
    if (!d) throw new Error('No game is being played.');
    return (d.made ??= benchApi((d.camera ??= new BenchRig(d.renderer, flags))));
  };
  const release = () => debugBench?.camera?.release();
  window.__bench = {
    pose: (pose) => api().pose(pose),
    hour: (hour) => api().hour(hour),
    freeze: (on) => api().freeze(on),
    settle: (options) => api().settle(options),
    counts: (frames) => api().counts(frames),
    walls: (mode) => api().walls(mode),
    view: () => api().view(),
    release,
    reset: async () => release(),
    idle: (on) => debugBench?.renderer.setActive(!on),
    ...timeApi(release),
  };
}

/** `__bench`'s bench time (`benchTime.ts`): a state loaded in the page, frames in fixed steps. `release`: the bench's camera let go. */
function timeApi(release: () => void): Pick<BenchApi, 'load' | 'step' | 'run' | 'state'> {
  return {
    load: (state, options) => benchTime.load(state, options),
    step: (ticks) => benchTime.step(ticks),
    // (The pose as `__bench.pose` holds it, or `pose`'s with the bench's camera let go: settled with the clock held, in
    // frames of the game's time.)
    run: (frames, options) => {
      if (options?.pose) release();
      return benchTime.run(frames, { ticks: options?.ticks, pose: options?.pose, settle: true });
    },
    state: () => benchTime.state(),
  };
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
  /** The sun's map drawn again a second: with the still casters too, and only what moves over their picture (`sunShadows.ts`). */
  sunShadowsPerSecond?: { whole: number; moving: number };
  /** Why the still casters' picture was drawn again over the measured stretch (`SunShadows.draws`: the check that failed, and on which draws). */
  sunShadowWhy?: { why: Record<string, number>; by: Record<string, number> };
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
    /** The same page posed, lit and counted again without a reload (`benchApi`): set with `__benchReady`, and on `?debug` game pages. */
    __bench?: BenchApi;
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
  const scene = await benchState(b.scene === 'bigworld' ? 'village' : b.scene);
  if (scene === null) throw new Error(`Benchmark scene "${b.scene}" not found`);
  // Live once the game screen shows its interface (the session is revealed by then).
  await benchTime.load(scene, { settle: false, name: `bench ${b.scene}` });
  const renderer = await host.rendererNow(settings);
  const controls = services.controls;
  if (b.cam === 'build') controls.setMode('build');
  renderer.setGameShot(null);
  renderer.followSim(null);

  const camera = new BenchRig(renderer, b);
  const frozen = b.diag.has('freeze');

  const intervals: number[] = [];
  const cpuFrame: number[] = [];
  const cpuUpdate: number[] = [];
  const cpuRender: number[] = [];
  const gpuFrame: number[] = [];
  const sums = { draws: 0, shadowDraws: 0, mirrorDraws: 0, tris: 0, shadowTris: 0, mirrorTris: 0, residents: 0 };
  let measuring = false;
  let lastGpuFrames = -1;
  renderer.measureGpu(b.gpu, 'bench');
  camera.probe({
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
      await renderer.whenSettled(SETTLE_LIMIT_MS);
      await renderer.framesRendered(3);
    }
    if (b.cam === 'town') await showTown(renderer, (b.hour ?? 13) * 60);
    window.__bench = benchApi(camera);
    window.__benchReady = true;
    // The measured stretch.
    const info = renderer.benchInfo();
    renderer.gpuPasses(true);
    const lighting0 = (info.scene as { lightingUpdates: number }).lightingUpdates;
    const reflections0 = (info.scene as { mirrorDraws: number }).mirrorDraws;
    const sun0 = sunDraws(renderer);
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
      if (last && time - last > 33.4) slow.push({ at: Math.round(time - started) / 1000, ms: Math.round(time - last), building: renderer.building() });
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
    const sun = sunSince(sunDraws(renderer), sun0);
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
      sunShadowsPerSecond: { whole: sun.whole / secs, moving: sun.moving / secs },
      sunShadowWhy: { why: sun.why, by: sun.by },
      longTasks: { count: long.length, maxMs: Math.max(0, ...long), totalMs: long.reduce((a, x) => a + x, 0) },
      over33: intervals.filter((ms) => ms > 33.4).length,
      slow,
      render: { ...info, end: end.scene, endStreet: end.street, focusSwitches: end.focusSwitches },
      fingerprint,
    };
  } finally {
    // A frozen picture stays where it is; otherwise the camera is the player's again (until `__bench.pose`).
    if (!frozen) camera.release();
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
