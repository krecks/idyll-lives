/**
 * Dev-only check of the PlayCanvas material library (`render/pc/materials.ts`, the shader features
 * in `render/pc/shaders/`): `/pcload.html?scene=materials`. A small lot in game space under the
 * mirrored world root: a road (planar UVs), floors (wood whitewashed, laminate), a striped wall that
 * follows the cutaway (`?walls=up|cutaway|down`, `?hole`), plain finishes from matte to chrome in one
 * draw, a glTF model adopted into the look, contact shadows, ground dug away over a basement
 * (`?dig`), the basement kept out of daylight (`?under`), a windowless room (`?dark`), and instanced
 * boxes tinted per instance. The sky's environment lights it at `?hour=` (12 by default), in the
 * soft look or `?look=classic`; `?diag=albedo|normals` work as in the game; `?renderer=webgl`.
 * `window.__result` says what compiled; `__ready` once drawn.
 */

import * as pc from '../render/pc/engine';
import { GameCamera } from '../render/pc/camera';
import { createApp, createDevice } from '../render/pc/device';
import { containerOptions, setupDecoders } from '../render/pc/loaders';
import { encodeFinish, MaterialLibrary, SUN_LIGHT_MASK, WINDOWLESS_BITS } from '../render/pc/materials';
import { meshFrom } from '../render/pc/meshes';
import { diagViewActive } from '../render/pc/shaders/diagView';
import { INSTANCE_COLOR_SEMANTIC } from '../render/pc/shaders/chunks';
import { createWorldRoot, toEngine } from '../render/pc/space';
import { AssetRegistry } from '../assets/registry';
import { LOOK } from '../render/look';
import { Cut, Geo, LOOK as CUT } from '../render/shared/geometry';
import { createLighting, lightingAt } from '../render/shared/environment';

const params = new URLSearchParams(location.search);
const canvas = document.getElementById('c') as HTMLCanvasElement;
canvas.width = canvas.clientWidth;
canvas.height = canvas.clientHeight;
const device = await createDevice(canvas, params.get('renderer') === 'webgl');
const app = createApp(canvas, device);
setupDecoders();
const result: Record<string, unknown> = { backend: device.isWebGPU ? 'webgpu' : 'webgl2' };
const errors: string[] = [];
const origError = console.error;
console.error = (...args: unknown[]) => {
  errors.push(args.map(String).join(' ').slice(0, 400));
  origError(...args);
};

const assets = await AssetRegistry.load(`${import.meta.env.BASE_URL}assets/manifest.json`);
const lib = MaterialLibrary.for(app).setAssets(assets);
const root = createWorldRoot(app);

// The camera and the light as the game has them.
const camera = new GameCamera(app.root, { fov: 0.7, near: 0.3, far: 400 });
camera.camera.layers = camera.camera.layers.filter((id) => id !== pc.LAYERID_SKYBOX);
camera.camera.toneMapping = diagViewActive ? pc.TONEMAP_LINEAR : pc.TONEMAP_ACES2;
camera.camera.gammaCorrection = pc.GAMMA_SRGB;
const [tx, tz] = (params.get('target') ?? '4,4').split(',').map(Number);
camera.setPose(Number(params.get('alpha') ?? -Math.PI * 0.62), Number(params.get('beta') ?? 0.95), Number(params.get('radius') ?? 15), tx, 0.6, tz);
const sun = new pc.Entity('sun');
sun.addComponent('light', { type: 'directional', castShadows: !params.has('noshadows'), shadowResolution: 2048, shadowType: pc.SHADOW_PCF3_32F, shadowBias: 0.05, normalOffsetBias: 0.02, shadowDistance: 40, mask: SUN_LIGHT_MASK });
app.root.addChild(sun);
app.scene.ambientLight = new pc.Color(0, 0, 0);
const l = lightingAt(Number(params.get('hour') ?? 12) * 60, LOOK, createLighting());
const to = toEngine(l.sunDirection.x, l.sunDirection.y, l.sunDirection.z, new pc.Vec3()).normalize();
const from = new pc.Vec3(0, -1, 0);
const axis = new pc.Vec3().cross(from, to);
sun.setRotation(new pc.Quat().setFromAxisAngle(axis.normalize(), (Math.acos(from.dot(to)) * 180) / Math.PI));
const g = (v: number) => Math.pow(Math.max(0, v), 1 / 2.2);
sun.light!.color = new pc.Color(g(l.sunColor.r), g(l.sunColor.g), g(l.sunColor.b));
sun.light!.intensity = l.sunIntensity / Math.PI;
lib.setSkyFill(l.skyColor, l.groundColor, l.skyIntensity);
lib.setEnvironmentIntensity(l.envIntensity);
camera.camera.clearColor = new pc.Color(g(l.clearColor.r), g(l.clearColor.g), g(l.clearColor.b));
const env = assets.get('environment.sky', 'image');
result.environment = env ? await lib.loadEnvironment(env.url) : false;

function add(name: string, geo: Geo, material: pc.Material, options: { colors?: boolean; cut?: boolean; shadows?: boolean } = {}): pc.MeshInstance {
  const mesh = meshFrom(device, geo.meshData({ colors: options.colors, cut: options.cut }))!;
  const mi = new pc.MeshInstance(mesh, material);
  const entity = new pc.Entity(name);
  entity.addComponent('render', { meshInstances: [mi], castShadows: options.shadows ?? true, receiveShadows: true });
  root.addChild(entity);
  return mi;
}

// The road and the ground round the lot; the ground dug away over the basement (`?dig`).
add('road', new Geo().box(-6, -0.05, -4, 14, 0, -1), lib.surface('material.road', { doubleSided: true }), { shadows: false });
const ground = add('ground', new Geo().box(-6, -0.05, -1, 14, 0, 12), lib.surface('material.ground', { doubleSided: true }), { shadows: false });
lib.digInto(ground);
lib.dig.rect.set(0.5, 0.5, 3.5, 3.5);
lib.dig.on = params.has('dig');
lib.dig.below = -0.5;
// The basement under it, out of daylight (`?under`), and a windowless room (`?dark`).
add('basement', new Geo().box(0.5, -3, 0.5, 3.5, -2.9, 3.5), lib.surface('material.floor', { doubleSided: true }));
lib.dig.under = params.has('under');
lib.dig.lit.set(0, 0.1, 0.1, 0);
if (params.has('dark')) {
  const tiles = new Uint8Array(4 * 4);
  tiles.fill(WINDOWLESS_BITS.tile);
  tiles[0] = WINDOWLESS_BITS.half0;
  lib.dig.darken({ x0: 5, z0: 0, w: 4, d: 4, y0: -0.5, y1: 3, tiles });
}

// Floors: laminate, wood limed pale.
add('floor', new Geo().box(5, 0, 0, 9, 0.05, 4), lib.surface('material.floor', { doubleSided: true }));
add('floorWood', new Geo().box(5, 0, 5, 9, 0.05, 9), lib.surface('material.floor.wood', { doubleSided: true, pattern: 'whitewash' }));

// A striped wall that follows the cutaway: it drops when the camera looks along -z (the masks'
// NegZ, the way this camera looks at it from the north).
const wall = new Geo().cutting(Cut.Clamp, CUT.All, CUT.All).box(5, 0, 4.4, 9, 2.6, 4.6);
add('wall', wall, lib.surface('material.wall.interior', { doubleSided: true, cutaway: true, pattern: 'stripes' }), { cut: true });
const walls = params.get('walls') ?? 'up';
lib.wallCut.params.set(walls === 'down' ? 1 : 0, 0.35, 0, 0);
lib.wallCut.look.set(walls === 'cutaway' ? 1 : 0, walls === 'cutaway' ? 1 : 0, walls === 'cutaway' ? 1 : 0, walls === 'cutaway' ? 1 : 0);
if (params.has('hole')) {
  lib.wallCut.hole.set(6.5, 4.5, 7.5, 4.5);
  lib.wallCut.holeSpan.set(0.4, 0.5, 2, 0);
}
// Contact shadows: soft black under the boxes, following the cutaway like the wall.
const shade = new Geo().cutting(Cut.Keep);
for (let i = 0; i < 6; i++) shade.color('#000000', 0.5).poly([[i * 1.4 - 0.15, 0.01, 9.85], [i * 1.4 + 0.95, 0.01, 9.85], [i * 1.4 + 0.95, 0.01, 11.05], [i * 1.4 - 0.15, 0.01, 11.05]], [0, 1, 0], undefined, [0.6, 0.6, 0.6, 0.6]);
add('contactShadows', shade, lib.contactShadow(), { colors: true, cut: true, shadows: false });

// Plain finishes in one draw: matte, satin, gloss, metal, chrome, painted (vertex colour alpha).
const plain = new Geo();
const finishes: [string, number, number][] = [['#D84A3A', 0.9, 0], ['#3A8AD8', 0.5, 0], ['#3AD86A', 0.2, 0], ['#C0C0C8', 0.35, 1], ['#E8E8F0', 0.05, 1], ['#E8C040', 0.6, 1]];
finishes.forEach(([hex, rough, metal], i) => plain.color(hex, encodeFinish(rough, metal)).box(i * 1.4, 0, 10, i * 1.4 + 0.8, 0.8, 10.8));
add('plain', plain, lib.plainFinish(), { colors: true });
// A vertex-coloured finish (fabric).
add('fabric', new Geo().color('#8A5AB0').box(-2, 0, 6, -0.8, 1.2, 7.2), lib.finish('material.finish.fabric'), { colors: true });

// Instanced boxes tinted per instance (`INSTANCE_COLOR_SEMANTIC`).
{
  const box = meshFrom(device, new Geo().box(-0.3, 0, -0.3, 0.3, 0.6, 0.3).meshData())!;
  const format = new pc.VertexFormat(device, [
    { semantic: pc.SEMANTIC_ATTR11, components: 4, type: pc.TYPE_FLOAT32 },
    { semantic: pc.SEMANTIC_ATTR12, components: 4, type: pc.TYPE_FLOAT32 },
    { semantic: pc.SEMANTIC_ATTR14, components: 4, type: pc.TYPE_FLOAT32 },
    { semantic: pc.SEMANTIC_ATTR15, components: 4, type: pc.TYPE_FLOAT32 },
    { semantic: INSTANCE_COLOR_SEMANTIC, components: 4, type: pc.TYPE_FLOAT32 },
  ]);
  const count = 5;
  const data = new Float32Array(count * 20);
  const tints = ['#FF6040', '#40A0FF', '#60E060', '#FFE040', '#E060E0'];
  for (let i = 0; i < count; i++) {
    const m = new pc.Mat4().setTranslate(-2.5, 0, 0.5 + i * 1.1);
    data.set(m.data, i * 20);
    const c = new pc.Color().fromString(tints[i]);
    data.set([c.r, c.g, c.b, 1], i * 20 + 16);
  }
  const vb = new pc.VertexBuffer(device, format, count, { data });
  const mi = new pc.MeshInstance(box, lib.instanceColored(lib.finish('material.finish.satin')));
  mi.setInstancing(vb);
  const entity = new pc.Entity('instanced');
  entity.addComponent('render', { meshInstances: [mi], castShadows: true, receiveShadows: true });
  root.addChild(entity);
}

// A glTF model, adopted into the look (`?model=`, a path under assets/models).
const model = params.get('model') ?? 'decor/fireplace.glb';
try {
  const url = `${import.meta.env.BASE_URL}assets/models/${model}`;
  const container = await new Promise<pc.ContainerResource>((resolve, reject) => {
    const asset = new pc.Asset(url, 'container', { url }, undefined, containerOptions as never);
    asset.once('load', () => resolve(asset.resource as pc.ContainerResource));
    asset.once('error', (err: unknown) => reject(new Error(`${url}: ${err}`)));
    app.assets.add(asset);
    app.assets.load(asset);
  });
  const entity = container.instantiateRenderEntity();
  entity.setLocalPosition(11, 0, 6);
  root.addChild(entity);
  for (const r of entity.findComponents('render') as pc.RenderComponent[]) for (const mi of r.meshInstances) lib.adopt(mi.material);
  result.model = model;
} catch (err) {
  result.modelError = String(err);
}

await lib.texturesReady();
let frames = 0;
const tick = () => {
  app.update(1 / 60);
  app.render();
  if (++frames === 20) {
    const shaders = (device as unknown as { shaders: pc.Shader[] }).shaders;
    const failed = shaders.filter((s) => s.failed).map((s) => s.name);
    Object.assign(window, { __result: { ...result, shaders: shaders.length, failed, errors }, __ready: true, __lib: lib });
  }
  requestAnimationFrame(tick);
};
requestAnimationFrame(tick);
