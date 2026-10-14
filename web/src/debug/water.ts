/**
 * Dev-only sink bench (`pnpm dev`, then /water.html): the seven sinks side by side, each playing its
 * `water_on` clip the way a resident washing does: the tap turns on (the stream grows from the spout),
 * runs, and turns off again, over and over. Built with the game's own `buildModel` and `clipPose`, so
 * it shows what play shows, without setting up a game. Not part of the build.
 *
 * URL: `?keys=model.sink,model.vanityDouble` (which models; default the seven sinks), `?t=0.2` (hold every
 * stream at that time, for screenshots), `?colors=body:%23c46b4a,stone:%232f3236` (the parts' colours),
 * `?alpha=…&beta=…&radius=…` (the camera). `__water.set(t)` holds the streams at `t` from the console;
 * `__water.play()` lets them run again.
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
} from '../render/babylon/core';
import { AssetRegistry } from '../assets/registry';
import type { PartColors } from '../core/protocol';
import { MaterialLibrary } from '../render/babylon/materials';
import { buildModel, clipPose, painted, type ModelTemplate } from '../render/babylon/models';
import { installNature } from '../render/babylon/nature';

const SINKS = ['model.sink', 'model.sinkPedestal', 'model.vanityDouble', 'model.sinkVessel', 'model.sinkConsole', 'model.farmhouseSink', 'model.counterSinkDouble'];
/** The clip a wash plays. */
const CLIP = 'water_on';
/** One turn of the loop (seconds): off, turning on (the clip), running, turning off (the clip backwards). */
const OFF = 0.9;
const RUNNING = 2.2;
/** Metres between the sinks' middles, past their own widths. */
const GAP = 0.6;

const params = new URLSearchParams(location.search);
const canvas = document.getElementById('c') as HTMLCanvasElement;
const engine = new Engine(canvas, true);
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

const keys = params.get('keys')?.split(',') ?? SINKS;
const colors = colorsParam();

interface Sink {
  key: string;
  template: ModelTemplate;
  x: number;
  /** The stream's part and how long its clip turns it on (seconds); part -1: no stream. */
  part: number;
  duration: number;
  label: HTMLElement;
}

const sinks: Sink[] = [];
const labels = document.getElementById('labels')!;
let x = 0;
for (const key of keys) {
  const entry = assets.get(key, 'model');
  // Footprint widths: 2 tiles for the double vanity and the double counter sink.
  const wide = /Double/.test(key) ? 2 : 1;
  const template = await buildModel(scene, key, painted(entry, colors), [wide, 1]);
  const clip = template.motion?.clips.get(CLIP);
  const part = clip?.tracks[0]?.part ?? -1;
  const width = template.outline ? template.outline.maxX - template.outline.minX : wide;
  x += width / 2;
  const label = document.createElement('span');
  label.textContent = clip ? key.replace(/^model\./, '') : `${key.replace(/^model\./, '')} (no ${CLIP})`;
  labels.appendChild(label);
  sinks.push({ key, template, x, part, duration: clip?.duration ?? 0, label });
  for (const mesh of template.meshes) {
    mesh.receiveShadows = true;
    shadows.addShadowCaster(mesh);
  }
  x += width / 2 + GAP;
}
// The row centred on the origin, the first on the left as the camera sees it (from the front: -x is its right).
const middle = (x - GAP) / 2;
for (const s of sinks) s.x = middle - s.x;

const ground = MeshBuilder.CreateGround('ground', { width: Math.max(12, x + 4), height: 8 }, scene);
const gm = new PBRMaterial('ground', scene);
gm.albedoColor = Color3.FromHexString('#6F9A3E').toLinearSpace();
gm.roughness = 1;
gm.metallic = 0;
ground.material = gm;
ground.receiveShadows = true;

/** How far into its clip each stream is at `seconds` into the loop: on, running, off, and off again. */
function clipTime(sink: Sink, seconds: number): number {
  const d = sink.duration;
  const t = seconds % (OFF + d + RUNNING + d);
  if (t < OFF) return 0;
  if (t < OFF + d) return t - OFF;
  if (t < OFF + d + RUNNING) return d;
  return d - (t - OFF - d - RUNNING);
}

const pose = new Float32Array(16);
/** Poses every sink: its stream `at(sink)` seconds into its clip. */
function poseAll(at: (sink: Sink) => number): void {
  for (const sink of sinks) {
    const { template } = sink;
    const clip = template.motion?.clips.get(CLIP);
    const place = Matrix.Translation(sink.x, 0, 0);
    for (let i = 0; i < template.meshes.length; i++) {
      const part = template.motion ? template.motion.parts[i] : -1;
      const m = part >= 0 && clip && clipPose(clip, part, at(sink), pose) ? Matrix.FromArray(pose).multiply(place) : place;
      template.meshes[i].thinInstanceSetBuffer('matrix', m.asArray() as unknown as Float32Array, 16, true);
    }
  }
}

const held = params.get('t');
let hold: number | null = held === null ? null : Number(held);
let started = performance.now();
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
toggle.textContent = hold === null ? 'Pause' : 'Play';
toggle.onclick = () => (hold === null ? set(0) : play());
for (const [id, t] of [['on', Infinity], ['off', 0]] as const) {
  document.getElementById(id)!.onclick = () => set(t);
}

// From the front (models face +z), a little above, the whole row in view.
const camera = new ArcRotateCamera('cam', Number(params.get('alpha') ?? Math.PI / 2), Number(params.get('beta') ?? 1.1), Number(params.get('radius')) || Math.max(2.6, x * 0.62), new Vector3(0, 0.75, 0), scene);
camera.attachControl(canvas, true);
camera.minZ = 0.02;
camera.wheelPrecision = 20;

const projected = new Vector3();
engine.runRenderLoop(() => {
  const seconds = (performance.now() - started) / 1000;
  poseAll((sink) => (hold === null ? clipTime(sink, seconds) : Math.min(hold, sink.duration)));
  scene.render();
  // Each name under its sink.
  const w = engine.getRenderWidth();
  const h = engine.getRenderHeight();
  const scale = canvas.clientWidth / w;
  for (const sink of sinks) {
    Vector3.ProjectToRef(new Vector3(sink.x, 0, 0.55), Matrix.Identity(), scene.getTransformMatrix(), camera.viewport.toGlobal(w, h), projected);
    sink.label.style.transform = `translate(${projected.x * scale}px, ${projected.y * scale}px) translate(-50%, 0)`;
  }
});
addEventListener('resize', () => engine.resize());
Object.assign(window, { __scene: scene, __water: { set, play, sinks }, __ready: true });
