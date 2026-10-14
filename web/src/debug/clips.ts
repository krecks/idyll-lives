/**
 * Dev-only clips bench (`pnpm dev`, then /clips.html): models side by side, each playing every clip
 * of its file in turn, the way the game plays it, so a model's moving parts can be checked without
 * setting up a game. Built with the game's own `buildModel` and `clipPose`. Not part of the build.
 *
 * How each clip plays follows the model entry's `motions`:
 * - a use's clip (`uses`, `open`, `night`, and clips nothing plays): at rest, opening, held open,
 *   closing again (played backwards);
 * - a loop (`loops`): at rest, then round after round, stopping at rest;
 * - the ambient clip (`idle`): round after round, always, alongside the others;
 * - the growth clip (`growth`): each stage in turn (planted, growing, ripe, picked), moving on to the
 *   next at the clip's pace, as the game does (`GROWTH_TIMES`).
 *
 * URL: `?keys=model.fridge,model.sink` (which models; default every model whose entry has `motions`),
 * `?clip=water_on` (only that clip), `?t=0.2` (hold every clip at that time, for screenshots),
 * `?cols=4` (models a row), `?colors=body:%23c46b4a,stone:%232f3236` (the parts' colours),
 * `?renderer=webgl` (WebGL2; default WebGPU), `?alpha=…&beta=…&radius=…` (the camera), `?target=1.6`
 * (the height it looks at; `?target=x,y,z` a point), `?labels=0`.
 * From the console: `__clips.set(t)` holds every clip at `t`, `__clips.play()` lets them run again,
 * `__clips.solo('name')` plays only that clip (`solo(null)`: all).
 */
import {
  ArcRotateCamera,
  Color3,
  Color4,
  DirectionalLight,
  Engine,
  HemisphericLight,
  ImageProcessingConfiguration,
  Matrix,
  MeshBuilder,
  PBRMaterial,
  Scene,
  ShadowGenerator,
  Vector3,
  WebGPUEngine,
  type AbstractEngine,
} from '../render/babylon/core';
import { AssetRegistry } from '../assets/registry';
import type { ModelEntry } from '../assets/types';
import type { PartColors } from '../core/protocol';
import { compressedTextureFeatures } from '../render/babylon/loaders';
import { MaterialLibrary } from '../render/babylon/materials';
import { buildModel, clipPose, painted, type ModelClip, type ModelTemplate } from '../render/babylon/models';
import { installNature } from '../render/babylon/nature';
import { GROWTH_TIMES } from '../render/babylon/objectMotion';

/** How a clip plays (see the file header). */
type Kind = 'use' | 'loop' | 'idle' | 'growth';

/** Seconds at rest before a clip plays, held open (a use), and running (a loop's rounds, at least two). */
const REST = 0.9;
const HOLD = 1.6;
const RUNNING = 2.4;
/** How long each growth stage shows (planted, growing, ripe, picked), moving on from the one before included. */
const STAGE = 1.6;
/** Metres between the models, past their own sizes. */
const GAP = 0.6;

const params = new URLSearchParams(location.search);
const canvas = document.getElementById('c') as HTMLCanvasElement;
const engine: AbstractEngine = await (async () => {
  if (params.get('renderer') !== 'webgl' && navigator.gpu) {
    const gpu = new WebGPUEngine(canvas, { antialias: true, ...compressedTextureFeatures() });
    await gpu.initAsync();
    return gpu;
  }
  return new Engine(canvas, true);
})();
const scene = new Scene(engine);
scene.clearColor = new Color4(0.62, 0.77, 0.9, 1);
scene.imageProcessingConfiguration.toneMappingEnabled = true;
scene.imageProcessingConfiguration.toneMappingType = ImageProcessingConfiguration.TONEMAPPING_ACES;
scene.imageProcessingConfiguration.exposure = 1.1;
installNature(scene);

const sky = new HemisphericLight('sky', new Vector3(0, 1, 0), scene);
sky.intensity = 0.6;
sky.groundColor = new Color3(0.32, 0.36, 0.25);
const sun = new DirectionalLight('sun', new Vector3(-0.45, -1, -0.35), scene);
sun.intensity = 3.2;
sun.diffuse = new Color3(1, 0.96, 0.88);
const shadows = new ShadowGenerator(2048, sun);
shadows.usePercentageCloserFiltering = true;
shadows.bias = 0.002;

const base = import.meta.env.BASE_URL;
const assets = await AssetRegistry.load(`${base}assets/manifest.json`, params.getAll('pack'));
const lib = MaterialLibrary.for(scene).setAssets(assets);
const env = assets.get('environment.sky', 'image');
if (env?.url) await lib.loadEnvironment(env.url);
scene.environmentIntensity = 0.9;

/** `?colors=body:#hex,stone:#hex` as the parts' colours. */
function colorsParam(): PartColors | undefined {
  const raw = params.get('colors');
  if (!raw) return undefined;
  return Object.fromEntries(raw.split(',').map((pair) => pair.split(':') as [string, string]));
}

/** The entry's `motions` as written (read loosely: a bench shows what's there). */
interface RawMotions {
  uses?: Record<string, string | { clip: string }>;
  open?: string;
  loops?: Record<string, string>;
  idle?: string;
  night?: string;
  growth?: string;
}

/** How each clip the entry names plays; clips it doesn't name play as a use's. */
function kinds(entry: ModelEntry | undefined): Map<string, Kind> {
  const m = (entry as { motions?: RawMotions } | undefined)?.motions ?? {};
  const out = new Map<string, Kind>();
  for (const clip of Object.values(m.loops ?? {})) out.set(clip, 'loop');
  if (m.idle) out.set(m.idle, 'idle');
  if (m.growth) out.set(m.growth, 'growth');
  return out;
}

const withMotions = () => assets.keys('model').filter((key) => (assets.get(key, 'model') as { motions?: unknown } | undefined)?.motions);
const keys = params.get('keys')?.split(',') ?? withMotions();
const colors = colorsParam();

interface Shown {
  name: string;
  kind: Kind;
  clip: ModelClip;
}

interface Bench {
  key: string;
  template: ModelTemplate;
  x: number;
  z: number;
  /** Its clips, in the order they take their turn (an `idle` one plays throughout). */
  clips: Shown[];
  label: HTMLElement;
}

const benches: Bench[] = [];
const labels = document.getElementById('labels')!;
const cols = Number(params.get('cols')) || (keys.length <= 7 ? keys.length : Math.ceil(Math.sqrt(keys.length * 1.5)));
for (const key of keys) {
  const entry = assets.get(key, 'model');
  const template = await buildModel(scene, key, painted(entry, colors), [1, 1]);
  const of = kinds(entry);
  const clips = [...(template.motion?.clips ?? [])].map(([name, clip]) => ({ name, kind: of.get(name) ?? 'use', clip }));
  const label = document.createElement('span');
  label.textContent = key.replace(/^model\./, '');
  if (params.get('labels') === '0') label.hidden = true;
  labels.appendChild(label);
  benches.push({ key, template, x: 0, z: 0, clips, label });
  for (const mesh of template.meshes) {
    mesh.receiveShadows = true;
    shadows.addShadowCaster(mesh);
  }
}

// Rows of `cols`, each centred, the first model on the left as the camera sees it (from the front, -x is its right).
const size = (b: Bench) => (b.template.outline ? [b.template.outline.maxX - b.template.outline.minX, b.template.outline.maxZ - b.template.outline.minZ] : [1, 1]);
let depth = 0;
let widest = 0;
for (let row = 0; row * cols < benches.length; row++) {
  const line = benches.slice(row * cols, row * cols + cols);
  const width = line.reduce((w, b) => w + size(b)[0], 0) + GAP * (line.length - 1);
  const deep = Math.max(...line.map((b) => size(b)[1]));
  let x = width / 2;
  for (const b of line) {
    const [w] = size(b);
    const o = b.template.outline;
    // Its outline's middle on the spot (a wall bed's isn't on its origin).
    b.x = x - w / 2 - (o ? (o.minX + o.maxX) / 2 : 0);
    b.z = -depth - deep / 2 - (o ? (o.minZ + o.maxZ) / 2 : 0);
    x -= w + GAP;
  }
  depth += deep + GAP * 2;
  widest = Math.max(widest, width);
}
depth -= GAP * 2;
for (const b of benches) b.z += depth / 2;

const ground = MeshBuilder.CreateGround('ground', { width: widest + 8, height: depth + 8 }, scene);
const gm = new PBRMaterial('ground', scene);
gm.albedoColor = Color3.FromHexString('#6F9A3E').toLinearSpace();
gm.roughness = 1;
gm.metallic = 0;
ground.material = gm;
ground.receiveShadows = true;

/** Seconds one clip's turn takes. */
function turnLength({ kind, clip }: Shown): number {
  const d = clip.duration;
  if (kind === 'loop') return REST + d * Math.max(2, Math.ceil(RUNNING / Math.max(d, 1e-3)));
  if (kind === 'growth') return GROWTH_TIMES.length * STAGE;
  return REST + d + HOLD + d;
}

/** How far into its clip a turn is at `t` seconds into it. */
function clipTime({ kind, clip }: Shown, t: number): number {
  const d = clip.duration;
  if (kind === 'growth') {
    const n = GROWTH_TIMES.length;
    const stage = Math.min(Math.floor(t / STAGE), n - 1);
    const from = GROWTH_TIMES[(stage + n - 1) % n] * d;
    const to = GROWTH_TIMES[stage] * d;
    return from + Math.sign(to - from) * Math.min(Math.abs(to - from), t - stage * STAGE);
  }
  if (t < REST) return 0;
  t -= REST;
  if (kind === 'loop') return d > 0 ? t % d : 0;
  if (t < d) return t;
  if (t < d + HOLD) return d;
  return Math.max(0, 2 * d + HOLD - t);
}

let solo: string | null = params.get('clip');
const held = params.get('t');
let hold: number | null = held === null ? null : Number(held);
let started = performance.now();

/** Where each clip of `b` is at `seconds` into the bench (by clip name), and the one taking its turn. */
function times(b: Bench, seconds: number): { at: Map<string, number>; turn: string | null } {
  const at = new Map<string, number>();
  const playing = b.clips.filter((c) => solo === null || c.name === solo);
  if (hold !== null) {
    for (const c of playing) at.set(c.name, Math.min(hold, c.clip.duration));
    return { at, turn: playing.length === 1 ? playing[0].name : null };
  }
  for (const c of playing) if (c.kind === 'idle') at.set(c.name, c.clip.duration > 0 ? seconds % c.clip.duration : 0);
  const turns = playing.filter((c) => c.kind !== 'idle');
  const cycle = turns.reduce((sum, c) => sum + turnLength(c), 0);
  let t = cycle > 0 ? seconds % cycle : 0;
  for (const c of turns) {
    const length = turnLength(c);
    if (t < length) {
      at.set(c.name, clipTime(c, t));
      return { at, turn: c.name };
    }
    t -= length;
  }
  return { at, turn: null };
}

const pose = new Float32Array(16);
const posed = new Float32Array(16);
/** Poses every model at `seconds` into the bench. */
function poseAll(seconds: number): void {
  for (const b of benches) {
    const { template } = b;
    const { at, turn } = times(b, seconds);
    const place = Matrix.Translation(b.x, 0, b.z);
    for (let i = 0; i < template.meshes.length; i++) {
      const part = template.motion ? template.motion.parts[i] : -1;
      // As the game: each clip that moves the part poses it, the later ones over the earlier.
      let moved = false;
      if (part >= 0) {
        for (const c of b.clips) {
          const t = at.get(c.name);
          if (t !== undefined && clipPose(c.clip, part, t, pose)) {
            posed.set(pose);
            moved = true;
          }
        }
      }
      const m = moved ? Matrix.FromArray(posed).multiply(place) : place;
      template.meshes[i].thinInstanceSetBuffer('matrix', m.asArray() as unknown as Float32Array, 16, true);
    }
    const name = b.key.replace(/^model\./, '');
    b.label.textContent = !b.clips.length ? `${name} (no clips)` : turn ? `${name} · ${turn}` : name;
  }
}

const controls = document.getElementById('controls')!;
const toggle = document.getElementById('toggle') as HTMLButtonElement;
function play(): void {
  hold = null;
  started = performance.now();
  toggle.textContent = 'Pause';
}
function set(t: number): void {
  hold = t;
  toggle.textContent = 'Play';
}
function setSolo(name: string | null): void {
  solo = name;
  started = performance.now();
  for (const button of controls.querySelectorAll<HTMLButtonElement>('button[data-clip]')) button.classList.toggle('on', (button.dataset.clip || null) === solo);
}
toggle.textContent = hold === null ? 'Pause' : 'Play';
toggle.onclick = () => (hold === null ? set(0) : play());
document.getElementById('rest')!.onclick = () => set(0);
document.getElementById('open')!.onclick = () => set(Infinity);
// A button per clip name (every model's clip of that name plays), and one for all of them.
for (const name of ['', ...new Set(benches.flatMap((b) => b.clips.map((c) => c.name)))]) {
  const button = document.createElement('button');
  button.dataset.clip = name;
  button.textContent = name || 'All clips';
  button.onclick = () => setSolo(name || null);
  controls.appendChild(button);
}
setSolo(solo);

// From the front (models face +z), a little above, everything in view: at most 0.75 m up for things
// on the floor, at the middle of things that all hang higher (on a wall), or at `target`.
const tallest = Math.max(1, ...benches.map((b) => b.template.height));
const top = Math.max(...benches.map((b) => b.template.height));
const lowest = Math.min(...benches.map((b) => b.template.bottom ?? 0));
const aim = new Vector3(0, lowest > 0.3 ? (lowest + top) / 2 : Math.min(0.75, tallest / 2), 0);
const target = params.get('target')?.split(',').map(Number);
if (target?.length === 1) aim.y = target[0];
else if (target?.length === 3) aim.set(target[0], target[1], target[2]);
const radius = Number(params.get('radius')) || Math.max(2.6, widest * 0.75 + depth * 0.8 + tallest * 0.4);
const camera = new ArcRotateCamera('cam', Number(params.get('alpha') ?? Math.PI / 2), Number(params.get('beta') ?? 1.1), radius, aim, scene);
camera.attachControl(canvas, true);
camera.minZ = 0.02;
camera.wheelPrecision = 20;

const projected = new Vector3();
engine.runRenderLoop(() => {
  poseAll((performance.now() - started) / 1000);
  scene.render();
  // Each name in front of its model.
  const w = engine.getRenderWidth();
  const h = engine.getRenderHeight();
  const scale = canvas.clientWidth / w;
  for (const b of benches) {
    const o = b.template.outline;
    Vector3.ProjectToRef(new Vector3(b.x + (o ? (o.minX + o.maxX) / 2 : 0), 0, b.z + (o ? o.maxZ : 0.5) + 0.12), Matrix.Identity(), scene.getTransformMatrix(), camera.viewport.toGlobal(w, h), projected);
    b.label.style.transform = `translate(${projected.x * scale}px, ${projected.y * scale}px) translate(-50%, 0)`;
  }
});
addEventListener('resize', () => engine.resize());
Object.assign(window, { __scene: scene, __clips: { set, play, solo: setSolo, benches }, __ready: true });
