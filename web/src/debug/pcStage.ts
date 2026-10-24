/**
 * The stage the PlayCanvas dev pages draw on (`gallery.ts`, `clips.ts`, `characters.ts`, `poses.ts`): the device (WebGPU, or
 * WebGL2 with `?renderer=webgl`), an app without its own frame loop, the world root in game space
 * (`pc/space.ts`), the game camera on an orbit (the player's dragging, `shared/orbit.ts`), a noon
 * sun with its shadow over a fixed square, the sky's light, a lawn, and the frame loop. The models'
 * finishes come from the manifest (`MaterialLibrary`), the plants' wind with them (`installNature`).
 * Not part of the build.
 */

import * as pc from '../render/pc/engine';
import { GameCamera } from '../render/pc/camera';
import { createApp, createDevice } from '../render/pc/device';
import { MaterialLibrary, SUN_LIGHT_MASK } from '../render/pc/materials';
import { ENV_UNITS, SKY_UNITS, SUN_UNITS } from '../render/pc/lightUnits';
import { installNature } from '../render/pc/nature';
import { pageLook } from '../render/pc/post';
import { DIAG_VIEW_DEFINE } from '../render/pc/shaders/diagView';
import { createWorldRoot, toEngine } from '../render/pc/space';
import { SunShadows } from '../render/pc/sunShadows';
import { AssetRegistry } from '../assets/registry';
import { OrbitControls } from '../render/shared/orbit';

export interface StageOptions {
  /** The sky's light, and the sun's direction (game space: the way it shines). */
  sky: number;
  sun: [number, number, number];
  /** The sun's strength (Babylon's units; 3.2), and the sky's in reflections and ambient (Babylon's `environmentIntensity`; 0.9). */
  sunIntensity?: number;
  environment?: number;
  /** The camera's pose and lens (Babylon's `ArcRotateCamera` defaults: a 0.8 rad field of view). */
  alpha: number;
  beta: number;
  radius: number;
  fov?: number;
  near?: number;
  /** The wheel's zoom: Babylon's `wheelPrecision` (higher: slower; 20). */
  wheelPrecision?: number;
}

export interface Stage {
  canvas: HTMLCanvasElement;
  device: pc.GraphicsDevice;
  app: pc.AppBase;
  /** The world, in game space. */
  root: pc.Entity;
  camera: GameCamera;
  /** The camera's pose (game space), turned by dragging. */
  orbit: OrbitControls;
  assets: AssetRegistry;
  lib: MaterialLibrary;
  /** The sun's shadow: over a fixed square round what is shown (`shadeSquare`). */
  shadows: SunShadows;
  /** Puts the camera where the orbit is now. */
  pose(): void;
  /** Where the game-space point shows on the canvas (CSS pixels); false off screen or behind. */
  project(x: number, y: number, z: number, out: { x: number; y: number }): boolean;
  /** Clip-space x and y (-1..1 across the view) of the game-space point, as Babylon's view and projection give them. */
  ndc(x: number, y: number, z: number): [number, number];
  /** A lawn `width` × `depth` metres round the origin, of `colour` (none with `?ground=0`: models alone, e.g. to compare diag views). */
  lawn(width: number, depth: number, colour?: string): void;
  /**
   * Each frame, `frame(seconds since the last)` then the picture; from now on. `still`: the app's
   * own clock stands still (the wind), so only what the page moves moves (Babylon's pages' `getDeltaTime` held at 0).
   */
  run(frame: (dt: number) => void, still?: boolean): void;
}

const params = new URLSearchParams(location.search);

export async function createStage(options: StageOptions): Promise<Stage> {
  const canvas = document.getElementById('c') as HTMLCanvasElement;
  const size = () => {
    canvas.width = Math.max(1, canvas.clientWidth);
    canvas.height = Math.max(1, canvas.clientHeight);
  };
  size();
  // (Multisampled, as Babylon's pages' engines were: `antialias`.)
  const device = await createDevice(canvas, params.get('renderer') === 'webgl', true);
  const app = createApp(canvas, device);
  const root = createWorldRoot(app);
  const base = import.meta.env.BASE_URL;
  const assets = await AssetRegistry.load(`${base}assets/manifest.json`, params.getAll('pack'));
  // Finishes (fabric, wood, bark, foliage, ...) come from the manifest's material entries; the plants' sway in the wind.
  const lib = MaterialLibrary.for(app).setAssets(assets);
  installNature(app);
  const env = assets.get('environment.sky', 'image');
  // The sky lights the models (its reflections and ambient), as Babylon's pages had it; it isn't drawn behind them.
  if (env?.url) await lib.loadEnvironment(env.url);
  // (In the game's units, `lightUnits.ts`, as `PlayCanvasRenderer.applyLighting` sets them.)
  lib.setEnvironmentIntensity((options.environment ?? 0.9) * ENV_UNITS);
  app.scene.layers.getLayerById(pc.LAYERID_SKYBOX)!.enabled = false;

  const camera = new GameCamera(app.root, { fov: options.fov ?? 0.8, near: options.near ?? 0.1, far: 2000 });
  const cam = camera.camera;
  cam.clearColor = new pc.Color(0.62, 0.77, 0.9, 1);
  // Babylon's pages' image processing, in the materials: ACES at an exposure of 1.1 (`post.ts`). A
  // diag view's materials skip it themselves; the lawn and the models keep it, as on Babylon's pages.
  pageLook(app, cam, 1.1);
  cam.aspectRatioMode = pc.ASPECT_MANUAL;
  cam.aspectRatio = canvas.width / canvas.height;
  // Babylon's pages' hemispheric light: white from straight above, the lawn's green from below.
  lib.setSkyFill({ r: 1, g: 1, b: 1 }, { r: 0.32, g: 0.36, b: 0.25 }, options.sky * SKY_UNITS);
  lib.setSkyDirection({ x: 0, y: 1, z: 0 });
  app.scene.ambientLight = new pc.Color(0, 0, 0);

  const strength = (options.sunIntensity ?? 3.2) * SUN_UNITS;
  const gamma = (c: number) => Math.pow(c * strength, 1 / 2.2);
  const sun = new pc.Entity('sun');
  sun.addComponent('light', {
    type: 'directional',
    castShadows: true,
    // Babylon's linear colour and strength (in the game's units, `SUN_UNITS`) as PlayCanvas takes them: sRGB, the
    // strength in the colour (as `PlayCanvasRenderer.applyLighting`: an intensity below 1 would be linearised with it).
    intensity: 1,
    color: new pc.Color(gamma(1), gamma(0.96), gamma(0.88)),
    mask: SUN_LIGHT_MASK,
  });
  // A directional light shines along its entity's -y: turned from straight down to the sun's way.
  const to = toEngine(options.sun[0], options.sun[1], options.sun[2], new pc.Vec3()).normalize();
  const down = new pc.Vec3(0, -1, 0);
  const axis = new pc.Vec3().cross(down, to);
  sun.setRotation(axis.length() > 1e-6 ? new pc.Quat().setFromAxisAngle(axis.normalize(), (Math.acos(Math.max(-1, Math.min(1, down.dot(to)))) * 180) / Math.PI) : pc.Quat.IDENTITY);
  app.root.addChild(sun);
  // (Babylon's pages: a 2048 map, no fade at its edge.)
  const shadows = new SunShadows(app, sun, { size: 2048, rate: 0, fade: 0 });

  // As Babylon's `ArcRotateCamera`: no zoom limits, beta kept off the poles.
  const orbit = new OrbitControls({ alpha: options.alpha, beta: options.beta, radius: options.radius }, { radius: [0.01, Infinity], beta: [0.01, Math.PI - 0.01] });
  orbit.wheelPrecision = options.wheelPrecision ?? 20;
  orbit.attach(canvas);

  const pose = () => camera.setPose(orbit.alpha, orbit.beta, orbit.radius, orbit.target.x, orbit.target.y, orbit.target.z);
  const view = new pc.Mat4();
  const viewProj = new pc.Mat4();
  const clip = new pc.Vec4();
  const ndc = (x: number, y: number, z: number): [number, number] => {
    pose();
    view.copy(camera.entity.getWorldTransform()).invert();
    viewProj.mul2(cam.projectionMatrix, view);
    viewProj.transformVec4(clip.set(x, y, -z, 1), clip);
    return [clip.x / clip.w, clip.y / clip.w];
  };

  addEventListener('resize', () => {
    size();
    app.setCanvasResolution(pc.RESOLUTION_FIXED, canvas.width, canvas.height);
    cam.aspectRatio = canvas.width / canvas.height;
  });

  return {
    canvas,
    device,
    app,
    root,
    camera,
    orbit,
    assets,
    lib,
    shadows,
    pose,
    project: (x, y, z, out) => camera.project(x, y, z, canvas.clientWidth, canvas.clientHeight, out),
    ndc,
    lawn(width, depth, colour = '#6F9A3E') {
      if (params.get('ground') === '0') return;
      const material = new pc.StandardMaterial();
      material.name = 'ground';
      material.diffuse = new pc.Color().fromString(colour);
      material.useMetalness = true;
      material.metalness = 0;
      material.gloss = 0;
      // Lit as the library's materials are (the sky's fill and its ambient), but in plain PBR and in
      // every view: Babylon's pages' lawn was a PBR material of their own, outside the soft look and the diag views.
      lib.adopt(material);
      material.setDefine('GAME_SOFT', false);
      if (DIAG_VIEW_DEFINE) material.setDefine(DIAG_VIEW_DEFINE, false);
      material.update();
      // (Round the origin, the same either way along z: in the world, as the game's ground.)
      const ground = new pc.Entity('ground');
      ground.addComponent('render', { type: 'plane', material, castShadows: false, receiveShadows: true });
      ground.setLocalScale(width, 1, depth);
      root.addChild(ground);
    },
    run(frame, still = false) {
      let last = performance.now();
      const tick = (now: number) => {
        const dt = Math.min(0.1, (now - last) / 1000);
        last = now;
        orbit.step(dt * 1000);
        pose();
        frame(dt);
        app.update(still ? 0 : dt);
        app.render();
        requestAnimationFrame(tick);
      };
      requestAnimationFrame(tick);
    },
  };
}
