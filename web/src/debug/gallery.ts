/**
 * Dev-only model gallery (`bun run dev`, then /gallery.html): draws manifest models in a grid with the
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
 *
 * `?scale=adult|child|all` stands residents beside each model for scale, drawn by the game's own
 * `Characters` (as `poses.ts` does) at the idle, on its own left (the right of the picture, the side
 * away from the camera) and facing the camera: a man and a
 * woman (adult), a child (child), or a woman, a teen, a child and an elder (all). At most 64 are
 * drawn: the first models get theirs.
 * `?camera=game` is the game's Live camera as a game starts (`BabylonRenderer.setupCamera`: radius
 * 24, beta 0.86, its longer lens), from the game's angle onto a model facing that camera, aimed at
 * the floor under the first model, or the one `?at=<key or index>` names; `__gallery.aim(n)` aims
 * it at model n. `alpha`, `beta`, `radius` and `target` still change it. Sets `window.__ready`
 * once the models (and the residents) are drawn.
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
import { Content } from '../content/content';
import type { FrameState } from '../core/bridge';
import type { SimInfo } from '../core/protocol';
import { Pose, type SnapshotLayout } from '../core/snapshot';
import { lookAtStage, palette } from '../game/household';
import { Characters, MAX_CHARACTERS } from '../render/babylon/characters';
import { MaterialLibrary } from '../render/babylon/materials';
import { compressedTextureFeatures } from '../render/babylon/loaders';
import { buildModel } from '../render/babylon/models';
import { installNature } from '../render/babylon/nature';

const params = new URLSearchParams(location.search);
/** `?scale`: who stands beside each model, nearest first. */
const SCALES: Record<string, { stage: string; female: boolean }[]> = {
  adult: [
    { stage: 'adult', female: false },
    { stage: 'adult', female: true },
  ],
  child: [{ stage: 'child', female: false }],
  all: [
    { stage: 'adult', female: true },
    { stage: 'teen', female: false },
    { stage: 'child', female: true },
    { stage: 'elder', female: false },
  ],
};
const scaleParam = params.get('scale');
if (scaleParam && !SCALES[scaleParam]) console.warn(`[gallery] scale=${scaleParam}: say adult, child or all`);
const scale = (scaleParam && SCALES[scaleParam]) || [];
/** Residents beside a model: the nearest this far past its left side, the rest this far apart (m). */
const BESIDE = 0.45;
const APART = 0.6;
/** The game's Live camera as a new game starts (`BabylonRenderer.setupCamera`); `alpha` is its angle turned half round, onto a model's front facing it. */
const GAME = { alpha: Math.PI * 0.38, beta: 0.86, radius: 24, fov: 0.7 };
const game = params.get('camera') === 'game';
const alpha = Number(params.get('alpha') ?? (game ? GAME.alpha : Math.PI / 2 - 0.6));
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
// Every model the game knows, from the base manifest and the packs it lists (most models live in packs).
const filter = params.get('filter');
const keys = params.get('keys')?.split(',') ?? assets.keys('model').filter((k) => (filter ? k.includes(filter) : /^model\.(tree|pine|bush|hedge|garden|flowers|flowerbed|plant)/.test(k)) && !k.endsWith('.far'));
if (keys.length === 0) console.warn(`[gallery] no models match ${filter ? `'${filter}'` : 'the default list'}`);

const cols = Number(params.get('cols')) || Math.max(1, Math.ceil(Math.sqrt(keys.length)));
const rows = Math.ceil(keys.length / cols);
const templates = await Promise.all(keys.map((key) => buildModel(scene, key, assets.get(key, 'model'), [1, 1])));
const outlines = templates.map((t) => t.outline ?? { minX: -0.5, maxX: 0.5, minZ: -0.5, maxZ: 0.5 });
// Spots the widest model's size apart, plus a metre (small things close together), plus the residents beside each.
const widest = Math.max(...outlines.map((o) => Math.max(o.maxX - o.minX, o.maxZ - o.minZ)));
const gap = Number(params.get('gap')) || Math.max(1.5, widest + 1) + (scale.length ? BESIDE + scale.length * APART : 0);
// (From the game camera's distance, as far as it sees.)
const ground = MeshBuilder.CreateGround('ground', { width: Math.max(game ? 90 : 0, cols * gap + 8), height: Math.max(game ? 90 : 0, rows * gap + 8) }, scene);
const gm = new PBRMaterial('ground', scene);
gm.albedoColor = Color3.FromHexString('#6F9A3E').toLinearSpace();
gm.roughness = 1;
gm.metallic = 0;
ground.material = gm;
ground.receiveShadows = true;

const labels = document.getElementById('labels')!;
const spots: { key: string; at: Vector3; el: HTMLSpanElement }[] = [];
/** Where each model stands, and the residents beside them (`?scale`). */
const places: Vector3[] = [];
const residents: { x: number; z: number; stage: string; female: boolean }[] = [];
const yaw = Math.PI / 2 - alpha; // facing the camera
const withResidents = Math.min(keys.length, Math.floor(MAX_CHARACTERS / Math.max(1, scale.length)));
if (withResidents < keys.length && scale.length) console.warn(`[gallery] residents beside the first ${withResidents} models only (${MAX_CHARACTERS} at most)`);
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
  places.push(new Vector3(x, 0, z));
  if (i < withResidents) {
    // In a line on the model's own left (-x: the right of the picture, away from the camera, so they never hide
    // it), level with its front (deep things: a little behind it).
    scale.forEach((r, j) => residents.push({ x: x + out.minX - BESIDE - j * APART, z: z + Math.max(0, out.maxZ - 0.3), ...r }));
    if (scale.length) {
      box.minX = Math.min(box.minX, x + out.minX - BESIDE - (scale.length - 1) * APART - 0.3);
      box.maxY = Math.max(box.maxY, 1.8);
    }
  }
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

// With nothing to show, frame an empty metre round the origin rather than infinities (NaN camera).
if (keys.length === 0) Object.assign(box, { minX: -0.5, maxX: 0.5, minY: 0, maxY: 1, minZ: -0.5, maxZ: 0.5 });

// Fronts face +z: from the front, a little to the right, at the models' middle height; as close
// as keeps the sphere round them all in view (a single cup from about 1 m, a bed from about 5).
const centre = new Vector3((box.minX + box.maxX) / 2, (box.minY + box.maxY) / 2, (box.minZ + box.maxZ) / 2);
const target = params.get('target')?.split(',').map(Number);
if (target?.length === 1) centre.y = target[0];
else if (target?.length === 3) centre.set(target[0], target[1], target[2]);
const reach = Math.max(0.3, Math.hypot(box.maxX - box.minX, box.maxY - box.minY, box.maxZ - box.minZ) / 2);
const camera = new ArcRotateCamera('cam', alpha, Number(params.get('beta') ?? (game ? GAME.beta : 1.15)), 1, centre, scene);
camera.radius = Number(params.get('radius')) || (game ? GAME.radius : (reach / Math.sin(camera.fov / 2)) * 1.1);
camera.attachControl(canvas, true);
camera.minZ = 0.1;
camera.wheelPrecision = 20;
/** Aims the game camera at the floor under model `n` (as the game focuses a point on the ground). */
const aim = (n: number): void => {
  if (places[n]) camera.setTarget(places[n].clone(), false, false, true);
};
if (game) {
  camera.fov = GAME.fov;
  camera.minZ = 0.3;
  const at = params.get('at') ?? '0';
  if (!target) aim(keys.includes(at) ? keys.indexOf(at) : Number(at) || 0);
}

// Residents for scale (`?scale`): the game's `Characters` with a still snapshot, at the idle.
let characters: Characters | null = null;
const FIELDS = ['id', 'x', 'z', 'yaw', 'pose', 'moving', 'social', 'outcome', 'partner', 'anim', 'emotion', 'role', 'away', 'object', 'action', 'mood', 'held'] as const;
const HEADER_LEN = 6;
const STRIDE = FIELDS.length;
const layout: SnapshotLayout = {
  capacity: HEADER_LEN + STRIDE * MAX_CHARACTERS,
  headerLen: HEADER_LEN,
  simStride: STRIDE,
  maxSims: MAX_CHARACTERS,
  ticksPerSecond: 20,
  header: { tick: 0, day: 1, minute: 2, speed: 3, simCount: 4, structureVersion: 5 },
  sim: Object.fromEntries(FIELDS.map((f, i) => [f, i])) as unknown as SnapshotLayout['sim'],
  actions: [],
  items: [],
};
const snap = new Float32Array(layout.capacity);
const frame: FrameState = { prev: snap, curr: snap, alpha: 0, steps: 1, layout, now: 0 };
const shadowMatrices = new Float32Array(MAX_CHARACTERS * 16);
if (residents.length) {
  const content = await Content.load(`${base}content/base.json`);
  const hairPalette = palette(assets, 'palette.hair');
  const sims: SimInfo[] = residents.map((r, i) => ({
    id: i,
    name: '',
    household: 0,
    gender: r.female ? 'female' : 'male',
    attractedTo: [],
    appearance: lookAtStage(
      {
        body: r.female ? '#C9725F' : '#5F86C9',
        skin: ['#C08A64', '#8D5A3B', '#E8C0A0', '#6B4430'][i % 4],
        hair: ['#2B2320', '#7A4A2A', '#C9A15A', '#1A1A1A'][i % 4],
        hairStyle: r.female ? 'long' : 'short',
        height: 1,
      },
      content.lifeStages.find((s) => s.id === r.stage),
      hairPalette,
    ),
    traits: [],
    perks: [],
  }));
  characters = new Characters(scene, assets, lib, content);
  if (!(await characters.init())) throw new Error('characters unavailable');
  await characters.build({ sims, objects: [] });
  for (const m of characters.casters()) shadows.addShadowCaster(m);
  const h = layout.header;
  const k = layout.sim;
  snap[h.speed] = 1;
  snap[h.simCount] = residents.length;
  residents.forEach((r, i) => {
    const o = HEADER_LEN + i * STRIDE;
    snap[o + k.id] = i;
    snap[o + k.x] = r.x;
    snap[o + k.z] = r.z;
    snap[o + k.yaw] = yaw;
    snap[o + k.pose] = Pose.Stand;
    snap[o + k.partner] = -1;
    snap[o + k.object] = -1;
    snap[o + k.action] = -1;
    snap[o + k.mood!] = 0.8;
  });
}
let clock = 0;
function step(dt: number): void {
  if (!characters) return;
  clock += dt;
  frame.now = clock * 1000;
  characters.update(frame, () => true, shadowMatrices);
}

scene.onAfterRenderObservable.add(() => {
  const vp = camera.viewport.toGlobal(engine.getRenderWidth(), engine.getRenderHeight());
  const m = scene.getTransformMatrix();
  for (const s of spots) {
    const p = Vector3.Project(s.at, Matrix.IdentityReadOnly, m, vp);
    s.el.style.left = `${(p.x / engine.getRenderWidth()) * canvas.clientWidth}px`;
    s.el.style.top = `${(p.y / engine.getRenderHeight()) * canvas.clientHeight}px`;
  }
});
let last = performance.now();
engine.runRenderLoop(() => {
  const now = performance.now();
  step(Math.min(0.1, (now - last) / 1000));
  last = now;
  scene.render();
});
addEventListener('resize', () => engine.resize());
// A few frames so the residents settle into the idle before screenshots.
for (let n = 0; n < 30; n++) step(1 / 30);
Object.assign(window, { __scene: scene, __camera: camera, __gallery: { keys, aim, characters }, __ready: true });
