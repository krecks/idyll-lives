/**
 * The game's renderer on PlayCanvas (`Renderer`, `render/types.ts`).
 *
 * Built up stage by stage from the Babylon renderer it replaces (`render/babylon/BabylonRenderer.ts`,
 * kept as the reference until the port is done; see the plan in docs). What this one does so far:
 * the device (WebGPU, else WebGL2), frames paced in whole display refreshes, the game camera (the
 * player's orbit input, the director's shots) and the menus' overview camera, the time of day's
 * sun, sky and fog, the lot's walls and floors as the simulation builds them, picking the ground,
 * projecting points onto the screen, and the stats. The rest is stubbed where the port reaches it.
 *
 * Game space is the Babylon renderer's left-handed one: everything in the world hangs under a root
 * mirrored in z (`space.ts`), and only cameras, lights and rays are converted.
 */

import * as pc from './engine';
import { GameCamera } from './camera';
import { createApp, createDevice } from './device';
import { meshFrom } from './meshes';
import { createWorldRoot, toEngine } from './space';
import { SunShadows } from './sunShadows';
import { benchDiag } from '../../debug/benchFlags';
import { LOOK } from '../look';
import type { RenderQuality } from '../quality';
import { GameCameraRig } from '../shared/cameraRig';
import { createLighting, lightingAt, type RGB } from '../shared/environment';
import { OrbitControls } from '../shared/orbit';
import { OverviewCamera } from '../shared/overviewCamera';
import { FramePacer } from '../shared/pacing';
import { renderScale, watchPixelRatio } from '../shared/resolution';
import type { FrameState } from '../../core/bridge';
import type { ItemPlacement, WorldStructure } from '../../core/protocol';
import type { VisemeTrack } from '../../voice/visemes';
import type {
  BuildEffect,
  CameraPose,
  EdgePreview,
  FrameCounts,
  FrameProbe,
  GameShot,
  LiveRenderOptions,
  LotHighlight,
  ObjectBackgroundClips,
  ObjectClipInfo,
  PaintPreviewFace,
  PickResult,
  PlacementGhost,
  RenderFailure,
  RenderPassInfo,
  Renderer,
  RendererDebugHooks,
  RendererDeps,
  RenderStats,
  ResizeHandle,
  RoomOverlayTile,
  SeenBird,
  TownShot,
  ViewRect,
  WallMode,
} from '../types';

/** The game camera's zoom and tilt limits (as the Babylon renderer's). */
const RADIUS_LIMITS = [6, 95] as const;
const BETA_LIMITS = [0.3, 1.35] as const;
/** Its lens: vertical field of view (a slightly long lens flattens the perspective: the dollhouse look), near and far (m). */
const LENS = { fov: 0.7, near: 0.3, far: 1600 };
/** Frames per second at most in the menus. */
const MENU_FPS = 30;
/** Light is recomputed when the game's clock moved this many minutes, at most every `LIGHT_MIN_MS`. */
const LIGHT_STEP = 0.5;
const LIGHT_MIN_MS = 100;
/** Frame after frame failing this many times is drawing broken for good. */
const FAILED_FRAMES_LIMIT = 60;
/** Nothing waits for frames longer than this (ms). */
const FRAME_WAIT_LIMIT_MS = 15_000;
/** The longest step a frame takes (ms): a stall isn't a jump. */
const MAX_FRAME_MS = 100;
/** Frames still drawn once a full-screen menu covers the view (its frosted glass shows the last). */
const HOLD_FRAMES = 2;
/** Metres the sun's shadow square reaches past the viewed lot. */
const SHADOW_MARGIN = 10;

/** A Babylon linear colour (the look's keys are used as linear there) as PlayCanvas takes it (sRGB, linearised when lit). */
function gammaColor(c: RGB, scale = 1, out = new pc.Color()): pc.Color {
  const g = (v: number) => Math.pow(Math.max(0, v * scale), 1 / 2.2);
  return out.set(g(c.r), g(c.g), g(c.b), 1);
}

export class PlayCanvasRenderer implements Renderer, RendererDebugHooks {
  onFailure: ((why: RenderFailure, error?: unknown) => void) | null = null;

  private device!: pc.GraphicsDevice;
  private app!: pc.AppBase;
  /** The world, in game space (`space.ts`). */
  private root!: pc.Entity;
  private camera!: GameCamera;
  private sun!: pc.Entity;
  private sunShadows!: SunShadows;
  private disposed = false;

  private readonly diag = benchDiag();
  private readonly debug = new URLSearchParams(location.search).has('debug');
  /** `?hour=`: the light held at that hour. */
  private readonly fixedHour: number | null;

  /** The player's camera input and the game camera's pose (the director's rig drives it too). */
  private readonly orbit = new OrbitControls({ alpha: -Math.PI * 0.62, beta: 0.86, radius: 24 }, { radius: RADIUS_LIMITS, beta: BETA_LIMITS });
  private detachOrbit: (() => void) | null = null;
  private readonly gameRig: GameCameraRig;
  /** The overview camera's pose (`OverviewCamera` writes it; the camera takes it each town frame). */
  private readonly townPose = { alpha: 0, beta: 0.9, radius: 60, target: { x: 0, y: 0, z: 0 }, targetScreenOffset: { x: 0, y: 0 }, fov: LENS.fov };
  private readonly townRig = new OverviewCamera(this.townPose);

  private readonly pacer = new FramePacer();
  private options: LiveRenderOptions = { resolution: 'auto', cameraSensitivity: 1, fpsCap: 0 };
  private active = false;
  private covered = false;
  private coveredFrames = 0;
  private readonly frameWaiters: { left: number; resolve: () => void }[] = [];
  private onFrame: ((now: number) => void) | null = null;
  private onView: (() => void) | null = null;
  private townListener: (() => void) | null = null;
  private lastTime = 0;
  private frameDt = 1000 / 60;
  private failedFrames = 0;

  private townMode = false;
  private flyingIn = false;
  private townMinute = 12 * 60;
  private townMinuteGoal = 12 * 60;
  private lastMinute = 12 * 60;
  private lastLightMinute = -1;
  private lastLightAt = 0;
  private readonly lighting = createLighting();

  /** What the current world drew (removed on the next `setWorld` or `clear`). */
  private worldEntities: pc.Entity[] = [];
  private ground: pc.Entity | null = null;
  private readonly materials = new Map<string, pc.StandardMaterial>();

  private readonly cssSize = { width: 1, height: 1 };
  private sizeWatch: ResizeObserver | null = null;
  private unwatchPixelRatio: (() => void) | null = null;
  private renderScaleNow = 1;

  /** The last pick's ray in game space (`pickPlane` & co.). */
  private readonly rayFrom = { x: 0, y: 0, z: 0 };
  private readonly rayDir = { x: 0, y: -1, z: 0 };
  private rayValid = false;
  private readonly vTmp = new pc.Vec3();
  private readonly vTmp2 = new pc.Vec3();

  private probe: FrameProbe | null = null;
  private readonly counts: FrameCounts = { drawCalls: 0, triangles: 0, shadowDrawCalls: 0, shadowTriangles: 0, mirrorDrawCalls: 0, mirrorTriangles: 0, lightingUpdates: 0, gpuFrames: 0, gpuMs: 0, residentsMs: 0 };
  private frameMs = 1000 / 60;
  private cpuMs = 0;
  private drawCalls = 0;
  private readonly gpuWanted = new Set<'overlay' | 'bench'>();

  constructor(
    private readonly deps: RendererDeps,
    private readonly quality: RenderQuality,
  ) {
    const params = new URLSearchParams(location.search);
    const hour = Number(params.get('hour'));
    this.fixedHour = Number.isFinite(hour) && params.has('hour') ? hour : null;
    this.gameRig = new GameCameraRig(this.orbit, { radius: RADIUS_LIMITS, beta: BETA_LIMITS }, (i, out) => this.simHead(i, out), () => this.orbit.stopInertia());
  }

  async init(canvas: HTMLCanvasElement): Promise<void> {
    const measure = () => {
      this.cssSize.width = canvas.clientWidth || 1;
      this.cssSize.height = canvas.clientHeight || 1;
    };
    measure();
    this.sizeWatch = new ResizeObserver(() => {
      measure();
      this.applyResolution();
    });
    this.sizeWatch.observe(canvas);
    const forceWebGL = this.deps.backend === 'webgl' || new URLSearchParams(location.search).get('renderer') === 'webgl';
    this.device = await createDevice(canvas, forceWebGL);
    this.app = createApp(canvas, this.device);
    // A GPU reset (driver update, a laptop switching GPUs, a crashed GPU process): what was on the
    // GPU is gone, and the game's own buffers can't be rebuilt in place.
    this.device.on('devicelost', () => this.onFailure?.('device'));
    this.applyResolution();
    this.unwatchPixelRatio = watchPixelRatio(this.onResize);
    window.addEventListener('resize', this.onResize);

    const scene = this.app.scene;
    // Distance fog in the horizon colour makes the hills fade into the sky (aerial haze).
    scene.fog.type = pc.FOG_EXP2;
    scene.fog.density = LOOK.fog;

    this.root = createWorldRoot(this.app);
    this.camera = new GameCamera(this.app.root, LENS);
    const cam = this.camera.camera;
    cam.toneMapping = pc.TONEMAP_ACES2;
    cam.gammaCorrection = pc.GAMMA_SRGB;

    this.sun = new pc.Entity('sun');
    this.sun.addComponent('light', {
      type: 'directional',
      castShadows: !this.diag.has('noshadows'),
      intensity: 1,
      shadowResolution: this.quality.shadowMapSize,
      numCascades: 1,
      shadowType: pc.SHADOW_PCF3_32F,
      shadowBias: 0.05,
      normalOffsetBias: 0.02,
      // (The menus' town: fitted round this much of the view.)
      shadowDistance: 400,
    });
    this.app.root.addChild(this.sun);
    this.sunShadows = new SunShadows(this.app, this.sun, this.quality.shadowRate);

    const o = this.orbit;
    o.wheelDeltaPercentage = 0.012;
    o.panningSensibility = 90;
    o.mapPanning = true;
    o.inertia = 0.82;
    o.panningInertia = 0.82;
    this.detachOrbit = o.attach(canvas);

    if (this.debug) Object.assign(window, { __scene: this.app.scene, __app: this.app, __renderer: this });
    this.applyLighting(this.lastMinute);
    this.applyCamera();
  }

  // --- the frame loop ------------------------------------------------------------------

  run(onFrame: (now: number) => void, onView?: () => void): void {
    this.onFrame = onFrame;
    this.onView = onView ?? null;
    this.setActive(true);
  }

  setActive(active: boolean): void {
    if (active === this.active) return;
    this.active = active;
    if (active) {
      this.lastTime = 0;
      this.pacer.requestAnimationFrame(this.frame);
    } else this.pacer.cancelAnimationFrame();
  }

  hold(on: boolean): void {
    if (on === this.covered) return;
    this.covered = on;
    this.coveredFrames = HOLD_FRAMES;
  }

  framesRendered(frames: number): Promise<void> {
    return new Promise((resolve) => {
      const waiter = { left: frames, resolve };
      this.frameWaiters.push(waiter);
      setTimeout(() => {
        const i = this.frameWaiters.indexOf(waiter);
        if (i < 0) return;
        this.frameWaiters.splice(i, 1);
        resolve();
      }, FRAME_WAIT_LIMIT_MS);
    });
  }

  private readonly frame = (time: number): void => {
    if (!this.active || this.disposed) return;
    this.pacer.requestAnimationFrame(this.frame);
    // Behind a full-screen menu: its last frames, then only those something waits for.
    if (this.covered) {
      if (this.coveredFrames > 0) this.coveredFrames--;
      else if (!this.frameWaiters.length) return;
    }
    const start = performance.now();
    const frozen = this.diag.has('freeze');
    const dt = (this.frameDt = frozen ? 0 : Math.min(this.lastTime ? time - this.lastTime : 1000 / 60, MAX_FRAME_MS));
    this.lastTime = time;
    const probe = this.probe;
    let updated = start;
    try {
      probe?.before(start);
      this.onFrame?.(this.pacer.frameTime || performance.now());
      updated = performance.now();
      if (this.townMode) this.townFrame(dt);
      else this.orbit.step(dt, Math.max(0.05, (dt * 60) / 1000));
      this.sunShadows.frame();
      this.applyCamera();
      if (this.townMode) this.townListener?.();
      else this.onView?.();
      this.app.update(dt / 1000);
      this.app.render();
      this.drawCalls = this.device._drawCallsPerFrame;
      this.device._drawCallsPerFrame = 0;
      this.failedFrames = 0;
    } catch (err) {
      if (this.failedFrames === 0) console.error('[render] a frame failed', err);
      if (++this.failedFrames === FAILED_FRAMES_LIMIT) this.onFailure?.('frames', err);
      return;
    }
    for (let i = this.frameWaiters.length - 1; i >= 0; i--) {
      const w = this.frameWaiters[i];
      if (--w.left <= 0) {
        this.frameWaiters.splice(i, 1);
        w.resolve();
      }
    }
    const end = performance.now();
    this.cpuMs += (end - start - this.cpuMs) * 0.1;
    if (dt > 0) this.frameMs += (dt - this.frameMs) * 0.1;
    if (probe) {
      const c = this.counts;
      c.drawCalls = this.drawCalls;
      const gpu = this.device.gpuProfiler;
      c.gpuMs = gpu?.enabled ? (gpu.frameTime ?? 0) : 0;
      if (c.gpuMs > 0) c.gpuFrames++;
      probe.after(start, updated, end, c);
    }
  };

  // --- the camera ----------------------------------------------------------------------

  /** Puts the camera where the game (or the overview) has it this frame. */
  private applyCamera(): void {
    if (this.townMode) {
      const p = this.townPose;
      this.camera.setPose(p.alpha, p.beta, p.radius, p.target.x, p.target.y, p.target.z, p.targetScreenOffset.x, p.targetScreenOffset.y);
    } else {
      const o = this.orbit;
      this.camera.setPose(o.alpha, o.beta, o.radius, o.target.x, o.target.y, o.target.z);
    }
  }

  cameraPose(): CameraPose {
    const o = this.orbit;
    return { alpha: o.alpha, beta: o.beta, radius: o.radius, target: [o.target.x, o.target.y, o.target.z] };
  }

  setCameraPose(pose: CameraPose): void {
    const o = this.orbit;
    [o.target.x, o.target.y, o.target.z] = pose.target;
    o.alpha = pose.alpha;
    o.beta = pose.beta;
    o.radius = pose.radius;
    o.checkLimits();
  }

  focus(x: number, z: number): void {
    // Keep the viewing angle; move the camera with its target.
    const o = this.orbit;
    o.target.x = x;
    o.target.y = 0;
    o.target.z = z;
    o.radius = Math.min(o.radius, 32);
  }

  setGameShot(shot: GameShot | null, cut = false): void {
    if (!shot) return this.gameRig.release();
    this.gameRig.setShot(shot, cut || this.townMode);
  }

  followSim(_index: number | null): void {
    // (Keeping a resident in the middle needs the residents: stage 7.)
  }

  setLeftDragCamera(on: boolean): void {
    this.orbit.buttons = on ? [0, 1, 2] : [1, 2];
  }

  cameraInfo(): { fov: number; radius: readonly [number, number]; beta: readonly [number, number] } {
    return { fov: LENS.fov, radius: RADIUS_LIMITS, beta: BETA_LIMITS };
  }

  project(x: number, y: number, z: number, out: { x: number; y: number }): boolean {
    return this.camera.project(x, y, z, this.cssSize.width, this.cssSize.height, out);
  }

  // --- the world -----------------------------------------------------------------------

  setWorld(world: WorldStructure, view: ViewRect | null): Promise<void> {
    this.clearWorld();
    const add = (name: string, data: WorldStructure['meshes']['walls'], material: pc.StandardMaterial) => {
      const mesh = meshFrom(this.device, data);
      if (!mesh) return;
      const entity = new pc.Entity(name);
      entity.addComponent('render', { meshInstances: [new pc.MeshInstance(mesh, material)], castShadows: true, receiveShadows: true });
      this.root.addChild(entity);
      this.worldEntities.push(entity);
    };
    // (Stage 3 of the port: the simulation's lot meshes in flat colours; the house builder and the
    // material library come with stages 4 and 5.)
    add('floors', world.meshes.floors, this.flat('floor', '#B48A60'));
    add('walls', world.meshes.walls, this.flat('wall', '#E6E1D6'));
    this.showGround(world.width, world.depth);
    const v = view;
    this.sunShadows.setSquare(v ? v.x + v.w / 2 : world.width / 2, v ? v.z + v.d / 2 : world.depth / 2, Math.hypot(v?.w ?? world.width, v?.d ?? world.depth) / 2 + SHADOW_MARGIN);
    return Promise.resolve();
  }

  clear(): void {
    this.clearWorld();
  }

  private clearWorld(): void {
    for (const e of this.worldEntities) e.destroy();
    this.worldEntities = [];
  }

  /** A flat matte material in an sRGB colour, two-sided (the simulation's meshes don't keep a winding). */
  private flat(key: string, hex: string): pc.StandardMaterial {
    let m = this.materials.get(key);
    if (!m) {
      m = new pc.StandardMaterial();
      m.diffuse = new pc.Color().fromString(hex);
      m.useMetalness = true;
      m.metalness = 0;
      m.gloss = 0.15;
      m.cull = pc.CULLFACE_NONE;
      m.update();
      this.materials.set(key, m);
    }
    return m;
  }

  /** The ground round the lot: a plain plane for now (the landscape comes with stage 5). */
  private showGround(width: number, depth: number): void {
    if (!this.ground) {
      this.ground = new pc.Entity('ground');
      this.ground.addComponent('render', { type: 'plane', material: this.flat('grass', '#5E8A3A'), castShadows: false, receiveShadows: true });
      this.root.addChild(this.ground);
    }
    const size = Math.max(width, depth) + 200;
    this.ground.setLocalPosition(width / 2, -0.01, depth / 2);
    this.ground.setLocalScale(size, 1, size);
  }

  update(frame: FrameState): void {
    const { curr, layout } = frame;
    const minute = this.fixedHour !== null ? this.fixedHour * 60 : (curr[layout.header.minute] ?? this.lastMinute);
    this.lastMinute = minute;
    if (this.townMode) return;
    const due = this.lastLightMinute < 0 || frame.now - this.lastLightAt >= LIGHT_MIN_MS;
    if (due && Math.abs(minute - this.lastLightMinute) >= LIGHT_STEP) {
      this.applyLighting(minute);
      this.lastLightMinute = minute;
      this.lastLightAt = frame.now;
    }
    this.gameRig.frame(this.frameDt / 1000);
  }

  warmGame(_frame: FrameState): Promise<void> {
    return this.framesRendered(1);
  }

  setItems(_items: readonly ItemPlacement[]): void {}
  setLampsOff(_off: ReadonlySet<number>): void {}
  holdModelRefresh(_on: boolean): void {}

  // --- light ---------------------------------------------------------------------------

  /** The time of day: the sun, the sky's fill, the fog and the clear colour (`shared/environment.ts`). */
  private applyLighting(minute: number): void {
    this.counts.lightingUpdates++;
    const l = lightingAt(minute, LOOK, this.lighting);
    // A directional light shines along its entity's -y: turned from straight down to the sun's way (game space to PlayCanvas).
    const to = toEngine(l.sunDirection.x, l.sunDirection.y, l.sunDirection.z, this.vTmp).normalize();
    const from = this.vTmp2.set(0, -1, 0);
    const axis = new pc.Vec3().cross(from, to);
    const angle = (Math.acos(Math.max(-1, Math.min(1, from.dot(to)))) * 180) / Math.PI;
    this.sun.setRotation(axis.length() > 1e-6 ? new pc.Quat().setFromAxisAngle(axis.normalize(), angle) : pc.Quat.IDENTITY);
    const light = this.sun.light!;
    light.color = gammaColor(l.sunColor);
    light.intensity = l.sunIntensity;
    this.sunShadows.moved();
    // (The sky's hemispheric fill as a flat ambient until the look is ported, stage 6.)
    this.app.scene.ambientLight = gammaColor(l.skyColor, l.skyIntensity * 0.6);
    const clear = gammaColor(l.clearColor, 1);
    this.camera.camera.clearColor = clear;
    this.app.scene.fog.color = clear;
  }

  // --- the menus' overview -------------------------------------------------------------

  get townShown(): boolean {
    return this.townMode;
  }

  showTown(world: WorldStructure, minute: number): Promise<void> {
    if (!this.townMode) {
      const o = this.orbit;
      const p = this.townPose;
      Object.assign(p, { alpha: o.alpha, beta: o.beta, radius: o.radius });
      Object.assign(p.target, o.target);
      this.townRig.syncFromCamera();
      this.orbit.enabled = false;
      this.orbit.stopInertia();
      this.townMinute = minute;
    }
    this.townMode = true;
    this.sunShadows.setOverview(true);
    this.townMinuteGoal = minute;
    this.lastLightMinute = -1;
    this.applyPacing();
    // (Stage 3: the town's land as a plane; the overview of its lots comes with stage 9.)
    this.showGround(world.width, world.depth);
    return Promise.resolve();
  }

  hideTown(glide = 0): Promise<void> {
    if (!this.townMode) return Promise.resolve();
    return new Promise((resolve) => {
      const done = () => {
        this.townMode = false;
        this.sunShadows.setOverview(false);
        this.flyingIn = false;
        this.orbit.enabled = true;
        this.orbit.stopInertia();
        this.lastLightMinute = -1;
        this.app.scene.fog.density = LOOK.fog;
        this.applyLighting(this.lastMinute);
        this.lastLightMinute = this.lastMinute;
        this.applyPacing();
        resolve();
      };
      if (glide > 0) {
        const o = this.orbit;
        this.flyingIn = true;
        this.applyPacing();
        this.townRig.glideTo({ alpha: o.alpha, beta: o.beta, radius: o.radius, x: o.target.x, z: o.target.z }, glide, done);
      } else done();
    });
  }

  setTownShot(shot: TownShot, cut = false): void {
    this.townRig.setShot(shot, cut);
    if (shot.minute !== undefined) this.townMinuteGoal = shot.minute;
  }

  nudgeTown(dAlpha: number, zoom: number): void {
    this.townRig.nudge(dAlpha, zoom);
  }

  setLotHighlight(_highlight: LotHighlight): void {}

  onTownFrame(fn: (() => void) | null): void {
    this.townListener = fn;
  }

  private townFrame(dt: number): void {
    const w = this.cssSize.width || 1;
    const h = this.cssSize.height || 1;
    this.townRig.setViewport(w, h);
    this.townRig.frame(Math.min(0.1, dt / 1000), w / h);
    // The light eases between the screens' hours (about two seconds).
    const goal = this.townMinuteGoal;
    if (this.townMinute !== goal) {
      const step = Math.max(0.5, Math.abs(goal - this.townMinute) * Math.min(1, dt / 600));
      this.townMinute = Math.abs(goal - this.townMinute) <= step ? goal : this.townMinute + Math.sign(goal - this.townMinute) * step;
    }
    if (this.lastLightMinute !== this.townMinute) {
      this.applyLighting(this.townMinute);
      this.lastLightMinute = this.townMinute;
    }
    // Haze is tuned for the game's close camera; from high above it would wash the town out.
    const fog = LOOK.fog * Math.min(1, Math.max(0.3, 55 / this.townPose.radius));
    if (Math.abs(fog - this.app.scene.fog.density) > LOOK.fog * 0.02) this.app.scene.fog.density = fog;
  }

  // --- picking -------------------------------------------------------------------------

  pick(x: number, y: number): PickResult {
    this.setRay(x, y);
    const ground = this.planeHit(0);
    return { ground, objectId: null, itemId: null };
  }

  pickPlane(height: number): { x: number; z: number } | null {
    return this.rayValid ? this.planeHit(height) : null;
  }

  pickVertical(_x: number, _z: number): number | null {
    return null;
  }

  pickOnWall(_axis: 'h' | 'v', _line: number, _storeyRow: number): { along: number; y: number } | null {
    return null;
  }

  handleAt(_x: number, _y: number): string | null {
    return null;
  }

  objectBox(_id: number): { height: number; bottom: number } | null {
    return null;
  }

  /** The camera's ray through canvas point (`x`, `y`) (CSS pixels), in game space. */
  private setRay(x: number, y: number): void {
    const cam = this.camera.camera;
    const eye = this.camera.entity.getPosition();
    const far = cam.screenToWorld(x, y, cam.farClip, this.vTmp);
    const dx = far.x - eye.x;
    const dy = far.y - eye.y;
    const dz = far.z - eye.z;
    const len = Math.hypot(dx, dy, dz) || 1;
    this.rayFrom.x = eye.x;
    this.rayFrom.y = eye.y;
    this.rayFrom.z = -eye.z;
    this.rayDir.x = dx / len;
    this.rayDir.y = dy / len;
    this.rayDir.z = -dz / len;
    this.rayValid = true;
  }

  /** Where the last ray meets the level `height` (game space x, z), or null. */
  private planeHit(height: number): { x: number; z: number } | null {
    const d = this.rayDir;
    if (Math.abs(d.y) < 1e-6) return null;
    const t = (height - this.rayFrom.y) / d.y;
    if (t <= 0) return null;
    return { x: this.rayFrom.x + d.x * t, z: this.rayFrom.z + d.z * t };
  }

  // --- settings, stats, the benchmark --------------------------------------------------

  configure(options: LiveRenderOptions): void {
    this.options = { ...this.options, ...options };
    const s = Math.max(0.25, options.cameraSensitivity);
    this.orbit.angularSensibilityX = this.orbit.angularSensibilityY = 1000 / s;
    this.orbit.panningSensibility = 90 / s;
    this.orbit.wheelDeltaPercentage = 0.012 * s;
    this.applyPacing();
    this.applyResolution();
  }

  /** The frame rate cap, and in the menus at most `MENU_FPS` (not while the camera flies into the game). */
  private applyPacing(): void {
    const cap = this.options.fpsCap;
    const menu = this.townMode && !this.flyingIn;
    this.pacer.setCap(menu ? (cap > 0 ? Math.min(cap, MENU_FPS) : MENU_FPS) : cap);
  }

  private readonly onResize = () => this.applyResolution();

  /** Sizes the drawing to the canvas, the device pixel ratio and the resolution setting (`shared/resolution.ts`). */
  private applyResolution(): void {
    if (!this.app) return;
    const dpr = window.devicePixelRatio || 1;
    const w = this.cssSize.width * dpr;
    const h = this.cssSize.height * dpr;
    const scale = (this.renderScaleNow = renderScale(w, h, this.options.resolution));
    this.app.setCanvasResolution(pc.RESOLUTION_FIXED, Math.max(1, Math.round(w * scale)), Math.max(1, Math.round(h * scale)));
  }

  stats(): RenderStats {
    const gpu = this.device.gpuProfiler;
    return {
      backend: this.device.isWebGPU ? 'WebGPU' : 'WebGL2',
      preset: this.quality.preset,
      fps: this.frameMs > 0 ? 1000 / this.frameMs : 0,
      frameMs: this.frameMs,
      cpuMs: this.cpuMs,
      drawCalls: this.drawCalls,
      holdStillFrames: false,
      renderWidth: this.device.width,
      renderHeight: this.device.height,
      renderScale: this.renderScaleNow,
      refreshHz: this.pacer.refreshHz,
      refreshDivisor: this.pacer.divisor,
      gpuMs: this.gpuWanted.size && gpu?.enabled && (gpu.frameTime ?? 0) > 0 ? (gpu.frameTime ?? null) : null,
      residentsMs: 0,
    };
  }

  setFrameProbe(probe: FrameProbe | null): void {
    this.probe = probe;
  }

  measureGpu(on: boolean, who: 'overlay' | 'bench' = 'overlay'): void {
    if (on) this.gpuWanted.add(who);
    else this.gpuWanted.delete(who);
    if (this.device.gpuProfiler) this.device.gpuProfiler.enabled = this.gpuWanted.size > 0;
  }

  gpuPasses(_reset?: boolean): { name: string; ms: number; frames: number }[] {
    return [];
  }

  benchInfo(): Record<string, unknown> & { passes: RenderPassInfo[] } {
    return { engine: `PlayCanvas ${pc.version}`, backend: this.device.isWebGPU ? 'webgpu' : 'webgl2', preset: this.quality.preset, passes: [] };
  }

  captureThumbnail(_width: number, _height: number): Promise<string | null> {
    return Promise.resolve(null);
  }

  // --- residents, birds, voices (stage 7) ----------------------------------------------

  simHead(_index: number, _out: { x: number; y: number; z: number }): boolean {
    return false;
  }

  birds(): readonly SeenBird[] {
    return [];
  }

  birdSings(_id: number, _seconds: number): void {}
  setSelectedSim(_id: number | null): void {}
  setSpeech(_index: number, _track: VisemeTrack | null, _startTime: number): void {}

  // --- build and buy (stage 8) ---------------------------------------------------------

  setHoverTile(_tile: { x: number; z: number } | null): void {}
  setWallMode(_mode: WallMode): void {}
  setPlacementGhost(_ghost: PlacementGhost | null): void {}
  setResizeHandles(_handles: readonly ResizeHandle[] | null, _hot?: string | null): void {}
  setHeldObject(_id: number | null, _moved?: boolean): void {}
  setEdgePreview(_edges: readonly EdgePreview[], _valid: boolean): void {}
  setPaintPreview(_faces: readonly PaintPreviewFace[], _cover: number | null): void {}
  setFloorPreview(_tiles: readonly { x: number; z: number }[], _floor: number | null): void {}
  setRoomOverlay(_tiles: readonly RoomOverlayTile[]): void {}
  setBuildGrid(_rect: ViewRect | null): void {}
  setStorey(_storey: number): void {}
  buildEffect(_fx: BuildEffect): void {}

  // --- dev tools (`RendererDebugHooks`) ------------------------------------------------

  objectClips(_id: number): ReadonlyMap<string, ObjectClipInfo> | null {
    return null;
  }

  objectMotions(_id: number): ObjectBackgroundClips | null {
    return null;
  }

  objectPoints(_id: number, _max?: number): Float32Array | null {
    return null;
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.setActive(false);
    this.onFailure = null;
    this.onFrame = null;
    this.onView = null;
    this.townListener = null;
    this.detachOrbit?.();
    this.detachOrbit = null;
    this.sizeWatch?.disconnect();
    this.unwatchPixelRatio?.();
    window.removeEventListener('resize', this.onResize);
    const debug = window as unknown as { __renderer?: unknown; __scene?: unknown; __app?: unknown };
    if (debug.__renderer === this) {
      delete debug.__renderer;
      delete debug.__scene;
      delete debug.__app;
    }
    for (const w of this.frameWaiters.splice(0)) w.resolve();
    this.app?.destroy();
  }
}
