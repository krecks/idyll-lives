/**
 * The stage the PlayCanvas dev pages draw on (`gallery.ts`, `clips.ts`): the device (WebGPU, or
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
import { installNature } from '../render/pc/nature';
import { pageLook } from '../render/pc/post';
import { createWorldRoot, toEngine } from '../render/pc/space';
import { SunShadows } from '../render/pc/sunShadows';
import { AssetRegistry } from '../assets/registry';
import { OrbitControls } from '../render/shared/orbit';

export interface StageOptions {
  /** The sky's light, and the sun's direction (game space: the way it shines). */
  sky: number;
  sun: [number, number, number];
  /** The camera's pose and lens (Babylon's `ArcRotateCamera` defaults: a 0.8 rad field of view). */
  alpha: number;
  beta: number;
  radius: number;
  fov?: number;
  near?: number;
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
  /** A lawn `width` × `depth` metres round the origin (none with `?ground=0`: models alone, e.g. to compare diag views). */
  lawn(width: number, depth: number): void;
  /** Each frame, `frame(seconds since the last)` then the picture; from now on. */
  run(frame: (dt: number) => void): void;
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
  lib.setEnvironmentIntensity(0.9);
  app.scene.layers.getLayerById(pc.LAYERID_SKYBOX)!.enabled = false;

  const camera = new GameCamera(app.root, { fov: options.fov ?? 0.8, near: options.near ?? 0.1, far: 2000 });
  const cam = camera.camera;
  cam.clearColor = new pc.Color(0.62, 0.77, 0.9, 1);
  // Babylon's pages' image processing, in the materials: ACES at an exposure of 1.1 (`post.ts`). A
  // diag view's materials skip it themselves; the lawn and the models keep it, as on Babylon's pages.
  pageLook(app, cam, 1.1);
  cam.aspectRatioMode = pc.ASPECT_MANUAL;
  cam.aspectRatio = canvas.width / canvas.height;
  // Babylon's pages' hemispheric light: white from above, the lawn's green from below.
  lib.setSkyFill({ r: 1, g: 1, b: 1 }, { r: 0.32, g: 0.36, b: 0.25 }, options.sky);
  app.scene.ambientLight = new pc.Color(0, 0, 0);

  const sun = new pc.Entity('sun');
  sun.addComponent('light', {
    type: 'directional',
    castShadows: true,
    // Babylon's 3.2: its PBR lit `albedo / π` per unit, PlayCanvas `albedo` (as the renderer's `SUN_UNITS`).
    intensity: 3.2 / Math.PI,
    // (Babylon's linear colour, as PlayCanvas takes it: sRGB.)
    color: new pc.Color(1, Math.pow(0.96, 1 / 2.2), Math.pow(0.88, 1 / 2.2)),
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
  orbit.wheelPrecision = 20;
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
    lawn(width, depth) {
      if (params.get('ground') === '0') return;
      const material = new pc.StandardMaterial();
      material.name = 'ground';
      material.diffuse = new pc.Color().fromString('#6F9A3E');
      material.useMetalness = true;
      material.metalness = 0;
      material.gloss = 0;
      material.update();
      // (Round the origin, the same either way along z: under the app's root, unmirrored, in its own winding.)
      const ground = new pc.Entity('ground');
      ground.addComponent('render', { type: 'plane', material, castShadows: false, receiveShadows: true });
      ground.setLocalScale(width, 1, depth);
      app.root.addChild(ground);
    },
    run(frame) {
      let last = performance.now();
      const tick = (now: number) => {
        const dt = Math.min(0.1, (now - last) / 1000);
        last = now;
        orbit.step(dt * 1000);
        pose();
        frame(dt);
        app.update(dt);
        app.render();
        requestAnimationFrame(tick);
      };
      requestAnimationFrame(tick);
    },
  };
}
