/**
 * Dev-only model gallery (`pnpm dev`, then /gallery.html): draws manifest models in a grid with the
 * game's own loader, finishes and plant shader (wind, tint) under a noon sun, to check art without
 * starting a game. Not part of the build.
 *
 * URL: `?keys=model.tree,model.pine` (default: every `model.tree*`, `model.garden.*` and nature key),
 * `?filter=garden` (keys containing it), `?cols=6`, `?gap=5`, `?labels=0`, `?pack=…` like the game.
 * The camera looks at the models' fronts from the front right (a three-quarter view) and from as
 * close as frames them all, at their middle height (things on a wall included); `?alpha=0.6&beta=1.1&radius=6`
 * and `?target=1.4` (a height) or `?target=x,y,z` (a point) change that. (`alpha=1.571` is straight
 * from the front, `-1.571` from the back.)
 *
 * The models are shaded in the game's soft look (`softLook.ts`), as the game draws them;
 * `?look=classic` (dev only) draws the PBR shading they had before, to compare.
 */

import {
  ArcRotateCamera,
  Color3,
  Color4,
  DirectionalLight,
  HemisphericLight,
  ImageProcessingConfiguration,
  Matrix,
  MeshBuilder,
  PBRMaterial,
  Scene,
  ShadowGenerator,
  Vector3,
  WebGPUEngine,
} from '../render/babylon/core';
import { AssetRegistry } from '../assets/registry';
import { MaterialLibrary } from '../render/babylon/materials';
import { compressedTextureFeatures } from '../render/babylon/loaders';
import { buildModel } from '../render/babylon/models';
import { installNature } from '../render/babylon/nature';

const params = new URLSearchParams(location.search);
const canvas = document.getElementById('c') as HTMLCanvasElement;
const engine = new WebGPUEngine(canvas, { antialias: true, ...compressedTextureFeatures() });
await engine.initAsync();
const scene = new Scene(engine);
scene.clearColor = new Color4(0.62, 0.77, 0.9, 1);
scene.imageProcessingConfiguration.toneMappingEnabled = true;
scene.imageProcessingConfiguration.toneMappingType = ImageProcessingConfiguration.TONEMAPPING_ACES;
scene.imageProcessingConfiguration.exposure = 1.1;
installNature(scene);

const sky = new HemisphericLight('sky', new Vector3(0, 1, 0), scene);
sky.intensity = 0.55;
sky.groundColor = new Color3(0.32, 0.36, 0.25);
const sun = new DirectionalLight('sun', new Vector3(-0.45, -1, 0.35), scene);
sun.intensity = 3.2;
sun.diffuse = new Color3(1, 0.96, 0.88);
const shadows = new ShadowGenerator(2048, sun);
shadows.usePercentageCloserFiltering = true;
shadows.bias = 0.002;

const base = import.meta.env.BASE_URL;
const assets = await AssetRegistry.load(`${base}assets/manifest.json`, params.getAll('pack'));
const env = assets.get('environment.sky', 'image');
// Finishes (fabric, wood, bark, foliage, ...) come from the manifest's material entries.
const lib = MaterialLibrary.for(scene).setAssets(assets);
if (env?.url) await lib.loadEnvironment(env.url);
scene.environmentIntensity = 0.9;
const manifest = (await (await fetch(`${base}assets/manifest.json`)).json()) as { entries: Record<string, { type: string }> };
const filter = params.get('filter');
const keys = params.get('keys')?.split(',') ?? Object.keys(manifest.entries).filter((k) => manifest.entries[k].type === 'model' && (filter ? k.includes(filter) : /^model\.(tree|pine|bush|hedge|garden|flowers|flowerbed|plant)/.test(k)) && !k.endsWith('.far'));

const cols = Number(params.get('cols')) || Math.ceil(Math.sqrt(keys.length));
const rows = Math.ceil(keys.length / cols);
const templates = await Promise.all(keys.map((key) => buildModel(scene, key, assets.get(key, 'model'), [1, 1])));
const outlines = templates.map((t) => t.outline ?? { minX: -0.5, maxX: 0.5, minZ: -0.5, maxZ: 0.5 });
// Spots the widest model's size apart, plus a metre (small things close together).
const widest = Math.max(...outlines.map((o) => Math.max(o.maxX - o.minX, o.maxZ - o.minZ)));
const gap = Number(params.get('gap')) || Math.max(1.5, widest + 1);
const ground = MeshBuilder.CreateGround('ground', { width: cols * gap + 8, height: rows * gap + 8 }, scene);
const gm = new PBRMaterial('ground', scene);
gm.albedoColor = Color3.FromHexString('#6F9A3E').toLinearSpace();
gm.roughness = 1;
gm.metallic = 0;
ground.material = gm;
ground.receiveShadows = true;

const labels = document.getElementById('labels')!;
const spots: { key: string; at: Vector3; el: HTMLSpanElement }[] = [];
/** What the models take up (m): x and z round their spots, y from the lowest bottom to the highest top. */
const box = { minX: Infinity, maxX: -Infinity, minY: Infinity, maxY: -Infinity, minZ: Infinity, maxZ: -Infinity };
keys.forEach((key, i) => {
  const x = (i % cols - (cols - 1) / 2) * gap;
  const z = -(Math.floor(i / cols) - (rows - 1) / 2) * gap;
  const template = templates[i];
  const out = outlines[i];
  const bottom = Math.min(template.bottom ?? 0, template.height);
  box.minX = Math.min(box.minX, x + out.minX);
  box.maxX = Math.max(box.maxX, x + out.maxX);
  box.minZ = Math.min(box.minZ, z + out.minZ);
  box.maxZ = Math.max(box.maxZ, z + out.maxZ);
  box.minY = Math.min(box.minY, bottom);
  box.maxY = Math.max(box.maxY, template.height);
  for (const mesh of template.meshes) {
    mesh.thinInstanceSetBuffer('matrix', Matrix.Translation(x, 0, z).asArray() as unknown as Float32Array, 16, true);
    mesh.receiveShadows = true;
    shadows.addShadowCaster(mesh);
  }
  if (params.get('labels') !== '0') {
    const el = document.createElement('span');
    el.textContent = key.replace(/^model\./, '');
    labels.append(el);
    spots.push({ key, at: new Vector3(x, 0, z + Math.min(gap * 0.38, out.maxZ + 0.15)), el });
  }
});

// Fronts face +z: from the front, a little to the right, at the models' middle height; as close
// as keeps the sphere round them all in view (a single cup from about 1 m, a bed from about 5).
const centre = new Vector3((box.minX + box.maxX) / 2, (box.minY + box.maxY) / 2, (box.minZ + box.maxZ) / 2);
const target = params.get('target')?.split(',').map(Number);
if (target?.length === 1) centre.y = target[0];
else if (target?.length === 3) centre.set(target[0], target[1], target[2]);
const reach = Math.max(0.3, Math.hypot(box.maxX - box.minX, box.maxY - box.minY, box.maxZ - box.minZ) / 2);
const camera = new ArcRotateCamera('cam', Number(params.get('alpha') ?? Math.PI / 2 - 0.6), Number(params.get('beta') ?? 1.15), 1, centre, scene);
camera.radius = Number(params.get('radius')) || (reach / Math.sin(camera.fov / 2)) * 1.1;
camera.attachControl(canvas, true);
camera.minZ = 0.1;
camera.wheelPrecision = 20;

scene.onAfterRenderObservable.add(() => {
  const vp = camera.viewport.toGlobal(engine.getRenderWidth(), engine.getRenderHeight());
  const m = scene.getTransformMatrix();
  for (const s of spots) {
    const p = Vector3.Project(s.at, Matrix.IdentityReadOnly, m, vp);
    s.el.style.left = `${(p.x / engine.getRenderWidth()) * canvas.clientWidth}px`;
    s.el.style.top = `${(p.y / engine.getRenderHeight()) * canvas.clientHeight}px`;
  }
});
engine.runRenderLoop(() => scene.render());
addEventListener('resize', () => engine.resize());
Object.assign(window, { __scene: scene, __camera: camera, __ready: true });
