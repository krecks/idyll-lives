/**
 * What item pictures and the colour studio's stage share: the product-shot scene on an app of its
 * own beside the game's (`createSideApp`: PlayCanvas's current app stays the game's, and everything
 * here is made on this app explicitly), its lights and shade.
 *
 * Light units: the studio's lights are in the look's units, converted as the game and the dev
 * pages convert the look's (`lightUnits.ts`): the directional lights' strengths in their colours
 * (`SUN_UNITS`), the hemispheric light as the soft look's sky fill (`SKY_UNITS`), the sky's
 * reflections and ambient (`ENV_UNITS`). The image processing is in the materials: exposure, ACES
 * and contrast (`pageLook`).
 */

import * as pc from '../pc/engine';
import type { AssetRegistry } from '../../assets/registry';
import { createDevice, createSideApp, destroySideApp } from '../pc/device';
import { ENV_UNITS, SKY_UNITS, SUN_UNITS } from '../pc/lightUnits';
import { MaterialLibrary } from '../pc/materials';
import { disposeTemplate, type ModelTemplate } from '../pc/models';
import { Instances } from '../pc/instancing';
import { deviceShaders } from '../pc/internals';
import { installNature, setWind, windStill } from '../pc/nature';
import { pageLook } from '../pc/post';
import { shadingFromSettings } from '../pc/shaders/softLook';
import { createWorldRoot, toEngine } from '../pc/space';
import type { StandIn } from '../shared/modelEntry';

/** What a thing whose file fails is pictured as (`BuildOptions.standIn`): a plain box on its footprint. */
export const PICTURE_STAND_IN: StandIn = { layer: 'floor', radius: 0.12 };

/** The studio's image processing (in the materials): ACES at this exposure, then this contrast. */
const EXPOSURE = 1.2;
const CONTRAST = 1.1;

/** The product-shot scene: its app (on a device of its own), the world root (game space) and the camera. */
export interface StudioScene {
  app: pc.AppBase;
  device: pc.GraphicsDevice;
  /** Game space (`space.ts`): models, the shade under them. */
  root: pc.Entity;
  camera: pc.CameraComponent;
  lib: MaterialLibrary;
  /** Destroys the app and its device. */
  destroy(): void;
}

export interface StudioOptions {
  /** `webgl` forces WebGL2 (the game's renderer setting, or `?renderer=webgl`). */
  backend: 'auto' | 'webgl';
  /** A multisampled canvas (the colour studio's stage; pictures multisample their own target). */
  antialias: boolean;
  /** A canvas that shows the page through where nothing is drawn (the stage). */
  alpha: boolean;
}

/**
 * The product-shot scene item pictures (and the colour studio's stage) are drawn in, on `canvas`:
 * transparent behind, tone-mapped like the game, lit by the sky's reflections and `lights`.
 */
export async function studioScene(canvas: HTMLCanvasElement, assets: AssetRegistry, options: StudioOptions): Promise<StudioScene> {
  const forceWebGL = options.backend === 'webgl' || new URLSearchParams(location.search).get('renderer') === 'webgl';
  const device = await createDevice(canvas, forceWebGL, options.antialias, options.alpha);
  const app = createSideApp(canvas, device);
  try {
    const root = createWorldRoot(app);
    const entity = new pc.Entity('studioCamera', app);
    entity.addComponent('camera', { clearColor: new pc.Color(0, 0, 0, 0) });
    const camera = entity.camera!;
    // (Not PlayCanvas's skybox: the environment atlas would draw as one.)
    camera.layers = camera.layers.filter((id) => id !== pc.LAYERID_SKYBOX);
    app.root.addChild(entity);
    // Before the first material compiles.
    pageLook(app, camera, EXPOSURE, CONTRAST);
    app.scene.ambientLight = new pc.Color(0, 0, 0);
    const lib = MaterialLibrary.for(app).setAssets(assets);
    lib.setShading(shadingFromSettings());
    // Foliage cards turn to face the camera here too (and sway a little in the turntable), still with
    // reduce motion as in the game: the setting (the page's `reduce-motion` class) or the system's,
    // looked at as each frame is drawn (before the wind's own `prerender`, which hands it over).
    const system = matchMedia('(prefers-reduced-motion: reduce)');
    app.on('prerender', () => setWind(app, system.matches || document.documentElement.classList.contains('reduce-motion') ? 0 : 1));
    installNature(app);
    const env = assets.get('environment.sky', 'image');
    if (env) await lib.loadEnvironment(env.url);
    lib.setEnvironmentIntensity(0.75 * ENV_UNITS);
    // (After the sky's lighting is made: its passes must draw.)
    asyncShaders(device);
    lights(app, lib);
    return { app, device, root, camera, lib, destroy: () => destroySideApp(app) };
  } catch (err) {
    destroySideApp(app);
    throw err;
  }
}

/** The 8 corners of the meshes' combined bounding box (model space, game space). */
export function boxCorners(meshes: readonly { mesh: pc.Mesh }[]): pc.Vec3[] {
  const min = new pc.Vec3(Infinity, Infinity, Infinity);
  const max = new pc.Vec3(-Infinity, -Infinity, -Infinity);
  for (const { mesh } of meshes) {
    if (!mesh.vertexBuffer?.numVertices) continue;
    min.min(mesh.aabb.getMin());
    max.max(mesh.aabb.getMax());
  }
  if (!Number.isFinite(min.x)) return [];
  const out: pc.Vec3[] = [];
  for (const x of [min.x, max.x]) for (const y of [min.y, max.y]) for (const z of [min.z, max.z]) out.push(new pc.Vec3(x, y, z));
  return out;
}

/** A soft round shade under the model, so it stands on something: a 1 m square in game space under `parent`, to be scaled. */
export function contactShadow(scene: StudioScene, parent: pc.Entity): pc.Entity {
  const n = 128;
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = n;
  const ctx = canvas.getContext('2d')!;
  const g = ctx.createRadialGradient(n / 2, n / 2, 0, n / 2, n / 2, n / 2);
  g.addColorStop(0, 'rgba(0,0,0,0.42)');
  g.addColorStop(0.55, 'rgba(0,0,0,0.2)');
  g.addColorStop(1, 'rgba(0,0,0,0)');
  ctx.fillStyle = g;
  ctx.fillRect(0, 0, n, n);
  const texture = new pc.Texture(scene.device, { name: 'itemShadow', width: n, height: n, format: pc.PIXELFORMAT_RGBA8, mipmaps: true, addressU: pc.ADDRESS_CLAMP_TO_EDGE, addressV: pc.ADDRESS_CLAMP_TO_EDGE });
  texture.setSource(canvas);
  const material = new pc.StandardMaterial();
  material.name = 'itemShadow';
  material.useLighting = false;
  material.useSkybox = false;
  material.useFog = false;
  material.diffuse.set(0, 0, 0);
  material.emissive.set(0, 0, 0);
  material.opacityMap = texture;
  material.opacityMapChannel = 'a';
  material.blendType = pc.BLEND_NORMAL;
  material.depthWrite = false;
  material.cull = pc.CULLFACE_NONE;
  material.update();
  const entity = new pc.Entity('itemShadow', scene.app);
  entity.addComponent('render', { type: 'plane', material, castShadows: false, receiveShadows: false });
  entity.setLocalPosition(0, 0.004, 0);
  parent.addChild(entity);
  return entity;
}

/** A model built for the studio, drawn once in place under `parent` (game space); `destroy` takes its draws and its meshes. */
export interface StudioModel {
  template: ModelTemplate;
  draws: Instances[];
  /** Draws it (or not). */
  show(on: boolean): void;
  destroy(): void;
}

/** One copy of `template` under `parent`, at `matrix` (game space; identity by default), never culled. */
export function placeModel(scene: StudioScene, parent: pc.Entity, template: ModelTemplate, matrix: ArrayLike<number> = IDENTITY): StudioModel {
  const draws = template.meshes.map((m) => {
    const draw = new Instances(scene.device, parent, m.mesh, m.material, { name: m.name, castShadows: false, receiveShadows: false, cull: false });
    windStill(draw.meshInstance, m);
    draw.setMatrices(matrix, 1);
    draw.update();
    return draw;
  });
  return {
    template,
    draws,
    show(on) {
      for (const d of draws) d.enabled = on;
    },
    destroy() {
      for (const d of draws) d.destroy();
      disposeTemplate(template);
    },
  };
}

export const IDENTITY: readonly number[] = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];

/**
 * WebGL2 compiles a shader where the driver can (`KHR_parallel_shader_compile`), but PlayCanvas waits
 * for it at its first draw: a new model's first frame would hold the page (the game's frames) while
 * its shaders compile. On a previews' device, draws whose shaders are still compiling are left out
 * of the frame instead (PlayCanvas's own `asyncCompile`), and pictures are drawn once none is
 * (`shadersCompiling`). Only the scene's
 * draws: one-off passes (the sky's lighting, made before) would be lost.
 */
function asyncShaders(device: pc.GraphicsDevice): void {
  if (!(device instanceof pc.WebglGraphicsDevice)) return;
  const set = device.setShader.bind(device);
  device.setShader = (shader) => set(shader, true);
}

/** Whether a shader of `device` is still compiling (WebGL2; draws with it are left out, `asyncShaders`). */
export function shadersCompiling(device: pc.GraphicsDevice): boolean {
  if (device.isWebGPU) return false;
  return deviceShaders(device).some((s) => !s.ready && !s.failed && !s.impl.isLinked(device));
}

/** Give up waiting for a model's textures (ms). */
const READY_TIMEOUT = 8000;

/** Waits until the textures asked for so far have loaded (the model's finishes); false if they took too long. */
export async function texturesReady(lib: MaterialLibrary): Promise<boolean> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const late = new Promise<boolean>((resolve) => (timer = setTimeout(() => resolve(false), READY_TIMEOUT)));
  try {
    return await Promise.race([lib.texturesReady().then(() => true), late]);
  } finally {
    clearTimeout(timer);
  }
}

/**
 * Product-shot lighting: a warm key from the upper front, a cool fill, a rim and the sky, with the
 * directions (game space: the way each shines) and strengths the pictures had up to 0.3.2. The
 * fill's highlights were a tenth of its light then (`specular` 0.1); PlayCanvas's lights have all
 * or none, and all came out nearer those pictures (the 12 of `/previews.html`: 0.15 % fewer pixels
 * off by more than 16).
 */
function lights(app: pc.AppBase, lib: MaterialLibrary): void {
  directional(app, 'itemKey', [0.45, -0.75, -0.5], 2.6, [1, 0.96, 0.9]);
  directional(app, 'itemFill', [-0.8, -0.3, -0.4], 0.9, [0.86, 0.91, 1]);
  directional(app, 'itemRim', [-0.2, -0.4, 0.9], 1.6, [1, 1, 1]);
  // The hemispheric light: white from straight above, a warm grey from below.
  lib.setSkyFill({ r: 1, g: 1, b: 1 }, { r: 0.5, g: 0.46, b: 0.42 }, 0.45 * SKY_UNITS);
  lib.setSkyDirection({ x: 0, y: 1, z: 0 });
}

function directional(app: pc.AppBase, name: string, way: [number, number, number], intensity: number, colour: [number, number, number]): void {
  const strength = intensity * SUN_UNITS;
  // (The linear colour and strength in the look's units as PlayCanvas takes them: sRGB, the strength in the colour.)
  const gamma = (c: number) => Math.pow(c * strength, 1 / 2.2);
  const light = new pc.Entity(name, app);
  light.addComponent('light', { type: 'directional', castShadows: false, intensity: 1, color: new pc.Color(gamma(colour[0]), gamma(colour[1]), gamma(colour[2])) });
  // A directional light shines along its entity's -y: turned from straight down to its way.
  const to = toEngine(way[0], way[1], way[2], new pc.Vec3()).normalize();
  const down = new pc.Vec3(0, -1, 0);
  const axis = new pc.Vec3().cross(down, to);
  light.setRotation(axis.length() > 1e-6 ? new pc.Quat().setFromAxisAngle(axis.normalize(), (Math.acos(Math.max(-1, Math.min(1, down.dot(to)))) * 180) / Math.PI) : pc.Quat.IDENTITY);
  app.root.addChild(light);
}
