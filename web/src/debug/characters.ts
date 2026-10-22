/**
 * Dev-only character bench (`bun run dev`, then /characters.html): residents in a grid, drawn by the
 * game's own `Characters` with stage directions (clip, held time, face) and a synthetic snapshot, to
 * check bodies, faces, hair and clothes without a game. Not part of the build.
 *
 * URL (combine freely; each sheet is one row per family):
 * - `?family=male|female` (default both), `?stage=child` or `stage=all` (default: the body sheet,
 *   every stage, when no other sheet is asked for);
 * - `?hair=all` every hairstyle (and the beard), `?tops=all` every garment, `?skins=all` eight skin
 *   tones, `?builds=all` slim / as built / broad, `?face=moods|visemes` every mood (and the friendly
 *   resting face, a half and a full blink) / viseme;
 * - `?clip=walk&t=0.4` the clip held at t seconds (default: idle, playing), `?view=side|back`,
 *   `?closeup=1` frames the heads, `?build=1.2`, `?cols=8`, `?gap=0.9`, `?labels=0`, `?hairColor=%23C89B5A`;
 * - `?alpha=…&beta=…&radius=…` the camera; `__bench.advance(s)` steps time when `?manual`;
 * - The residents are shaded in the game's soft look (`softLook.ts`); `?look=classic` (dev only)
 *   draws the PBR shading they had before, to compare.
 * Sets `window.__ready` once everything is drawn.
 */

import { ArcRotateCamera, Color3, Color4, DirectionalLight, Engine, HemisphericLight, ImageProcessingConfiguration, Matrix, MeshBuilder, PBRMaterial, Scene, ShadowGenerator, Vector3 } from '../render/babylon/core';
import { AssetRegistry } from '../assets/registry';
import type { FrameState } from '../core/bridge';
import type { SimInfo } from '../core/protocol';
import { Pose, type SnapshotLayout } from '../core/snapshot';
import type { Appearance, HairStyle } from '../game/household';
import { HAIR_STYLES } from '../game/household';
import { Characters, EXPRESSIONS, FRIENDLY, MAX_CHARACTERS, type StageDirection } from '../render/babylon/characters';
import { GARMENTS } from '../render/babylon/characters/outfit';
import { WALL_HEIGHT } from '../render/babylon/house';
import { MaterialLibrary } from '../render/babylon/materials';
import { VISEMES } from '../voice/visemes';

const params = new URLSearchParams(location.search);
const STAGES = ['baby', 'child', 'teen', 'youngAdult', 'adult', 'elder'];
const SKINS = ['#F3D6C2', '#E8C0A0', '#D9A882', '#C08A64', '#A9714B', '#8D5A3B', '#6B4430', '#4A2E22'];
const families = params.get('family') ? [params.get('family')!] : ['male', 'female'];

interface Cell {
  label: string;
  gender: string;
  look: Partial<Appearance>;
  face?: StageDirection['face'];
  viseme?: number;
}

const base = (gender: string): Appearance => ({
  body: gender === 'female' ? '#C9725F' : '#5F86C9',
  skin: '#C08A64',
  hair: params.get('hairColor') ?? '#4A3020',
  hairStyle: gender === 'female' ? 'long' : 'short',
  eyes: '#4E7BA6',
  height: 1,
  top: 'top.tee',
  bottom: gender === 'female' ? 'bottom.skirt' : 'bottom.trousers',
  shoes: 'shoes.sneakers',
  bottomColor: '#33415C',
});

// One row per family per sheet.
const rows: Cell[][] = [];
const stageParam = params.get('stage');
const stage = stageParam && stageParam !== 'all' ? stageParam : undefined;
const build = params.get('build') ? Number(params.get('build')) : undefined;
const common = (): Partial<Appearance> => ({ ...(stage ? { stage } : {}), ...(build ? { build } : {}) });
const sheets = ['hair', 'tops', 'skins', 'builds', 'face'].filter((k) => params.has(k));
if (!sheets.length || stageParam === 'all') {
  for (const g of families) rows.push(STAGES.map((s) => ({ label: `${g} ${s}`, gender: g, look: { stage: s, ...(build ? { build } : {}) } })));
}
for (const g of families) {
  if (params.has('hair')) {
    const styles: string[] = [...HAIR_STYLES, ...(g === 'male' ? ['beard'] : [])];
    rows.push(styles.map((h) => ({ label: h, gender: g, look: h === 'beard' ? { ...common(), hairStyle: 'short', beard: true } : { ...common(), hairStyle: h as HairStyle } })));
  }
  if (params.has('tops')) {
    const tops = GARMENTS.TOPS[g] ?? [];
    const bottoms = GARMENTS.BOTTOMS[g] ?? [];
    const ids = [...new Set([...tops, ...bottoms, ...GARMENTS.SHOES])];
    rows.push(
      ids.map((id) => {
        const look: Partial<Appearance> = { ...common() };
        if (id.startsWith('top.')) look.top = id;
        else if (id.startsWith('bottom.')) look.bottom = id;
        else look.shoes = id;
        // Bottoms under a short top, so the waistband shows.
        if (id.startsWith('bottom.')) look.top = g === 'female' ? 'top.tank' : 'top.tee';
        return { label: id, gender: g, look };
      }),
    );
  }
  if (params.has('skins')) rows.push(SKINS.map((s) => ({ label: s, gender: g, look: { ...common(), skin: s } })));
  if (params.has('builds')) rows.push([0.85, 1, 1.2].map((b) => ({ label: `build ${b}`, gender: g, look: { ...common(), build: b } })));
  if (params.get('face') === 'moods') {
    rows.push([
      { label: 'neutral', gender: g, look: common(), face: {} },
      ...Object.keys(EXPRESSIONS).map((m) => ({ label: m, gender: g, look: common(), face: EXPRESSIONS[m] })),
      { label: 'friendly', gender: g, look: common(), face: FRIENDLY },
      // Half (eyeBlink 0.5) and full blinks.
      { label: 'half blink', gender: g, look: common(), face: { lidU: 1 / 0.9 } },
      { label: 'blink', gender: g, look: common(), face: { mood: 'asleep' } },
    ]);
  } else if (params.get('face') === 'visemes') {
    rows.push(VISEMES.map((v, i) => ({ label: v, gender: g, look: common(), viseme: i })));
  }
}

const cells = rows.flatMap((row, r) => row.map((c, k) => ({ ...c, r, k })));
if (cells.length > MAX_CHARACTERS) throw new Error(`bench: ${cells.length} residents (max ${MAX_CHARACTERS})`);
const closeup = params.get('closeup') === '1';
const cols = Number(params.get('cols')) || Math.max(...rows.map((r) => r.length));
const gap = Number(params.get('gap')) || (closeup ? 0.36 : 0.9);
// Close-ups stack the lines on raised floors (one head height apart), each lower one nearer the camera.
const rowGap = closeup ? 0.36 : 2.1;
// Wrap long rows.
const placed = cells.map((c, i) => {
  const k = c.k % cols;
  const line = rows.slice(0, c.r).reduce((n, row) => n + Math.ceil(row.length / cols), 0) + Math.floor(c.k / cols);
  // The camera looks down -z, where +x is on the viewer's left: mirror so rows read left to right.
  return { ...c, i, x: -(k - (Math.min(cols, rows[c.r].length) - 1) / 2) * gap, line };
});
const lines = Math.max(...placed.map((p) => p.line)) + 1;

const canvas = document.getElementById('c') as HTMLCanvasElement;
const engine = new Engine(canvas, true, { preserveDrawingBuffer: true, stencil: true });
const scene = new Scene(engine);
scene.clearColor = new Color4(0.62, 0.77, 0.9, 1);
scene.imageProcessingConfiguration.toneMappingEnabled = true;
scene.imageProcessingConfiguration.toneMappingType = ImageProcessingConfiguration.TONEMAPPING_ACES;
scene.imageProcessingConfiguration.exposure = 1.1;
const sky = new HemisphericLight('sky', new Vector3(0, 1, 0), scene);
sky.intensity = 0.7;
sky.groundColor = new Color3(0.32, 0.36, 0.25);
const sun = new DirectionalLight('sun', new Vector3(-0.35, -1, -0.6), scene);
sun.intensity = 3;
sun.diffuse = new Color3(1, 0.96, 0.88);
const shadowGen = new ShadowGenerator(2048, sun);
shadowGen.usePercentageCloserFiltering = true;
shadowGen.bias = 0.002;

const baseUrl = import.meta.env.BASE_URL;
const assets = await AssetRegistry.load(`${baseUrl}assets/manifest.json`, []);
const lib = MaterialLibrary.for(scene).setAssets(assets);
const env = assets.get('environment.sky', 'image');
if (env?.url) await lib.loadEnvironment(env.url);

const width = cols * gap + 4;
const depth = lines * rowGap + 4;
const ground = MeshBuilder.CreateGround('ground', { width, height: depth }, scene);
const gm = new PBRMaterial('ground', scene);
gm.albedoColor = Color3.FromHexString('#8E9A7A').toLinearSpace();
gm.roughness = 1;
gm.metallic = 0;
ground.material = gm;
ground.receiveShadows = true;

const zOf = (line: number) => (closeup ? line * 0.3 : -(line - (lines - 1) / 2) * rowGap);
const liftOf = (line: number) => (closeup ? (lines - 1 - line) * rowGap : 0);
const sims: SimInfo[] = placed.map((p) => ({
  id: p.i,
  name: p.label,
  household: 0,
  gender: p.gender,
  attractedTo: [],
  appearance: { ...base(p.gender), ...p.look } as Appearance,
  traits: [],
  perks: [],
}));
const characters = new Characters(scene, assets, lib);
if (!(await characters.init())) throw new Error('characters unavailable');
await characters.build({ sims, objects: [] });
for (const m of characters.casters()) shadowGen.addShadowCaster(m);

const clipName = params.get('clip');
const held = params.get('t');
const view = params.get('view');
const yaw = view === 'side' ? Math.PI / 2 : view === 'back' ? Math.PI : 0;
placed.forEach((p) => {
  const dir: StageDirection = { face: p.face ?? null };
  if (clipName) dir.clip = clipName;
  if (held !== null) {
    dir.still = true;
    dir.time = Number(held);
    dir.clip ??= 'idle';
  }
  characters.directions[p.i] = dir;
});

const FIELDS = ['id', 'x', 'z', 'yaw', 'pose', 'moving', 'social', 'outcome', 'partner', 'anim', 'emotion', 'role', 'away', 'object', 'action', 'mood', 'height'] as const;
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
};
const snap = new Float32Array(layout.capacity);
const frame: FrameState = { prev: snap, curr: snap, alpha: 0, steps: 1, layout, now: 0 };
const shadowMatrices = new Float32Array(MAX_CHARACTERS * 16);
{
  const h = layout.header;
  const k = layout.sim;
  snap[h.speed] = 1;
  snap[h.simCount] = placed.length;
  for (const p of placed) {
    const o = HEADER_LEN + p.i * STRIDE;
    snap[o + k.id] = p.i;
    snap[o + k.x] = p.x;
    snap[o + k.z] = zOf(p.line);
    snap[o + k.yaw] = yaw;
    snap[o + k.pose] = Pose.Stand;
    snap[o + k.partner] = -1;
    snap[o + k.object] = -1;
    snap[o + k.action] = -1;
    snap[o + k.mood!] = 0.8;
    snap[o + k.height!] = liftOf(p.line) / WALL_HEIGHT;
  }
}

const faceY = 1.5 + (lines - 1) * rowGap * 0.5;
const camera = new ArcRotateCamera(
  'cam',
  Number(params.get('alpha') ?? Math.PI / 2),
  Number(params.get('beta') ?? (closeup ? Math.PI / 2 : 1.3)),
  Number(params.get('radius')) || (closeup ? Math.max(cols * gap * 1.5, lines * rowGap * 2.6) : Math.max(width * 0.75, lines * rowGap * 1.3, 3)),
  new Vector3(0, closeup ? faceY : 0.85, 0),
  scene,
);
camera.attachControl(canvas, true);
camera.minZ = 0.02;
camera.wheelPrecision = 60;

const labels = document.getElementById('labels')!;
const spots: { at: Vector3; el: HTMLSpanElement }[] = [];
if (params.get('labels') !== '0') {
  for (const p of placed) {
    const el = document.createElement('span');
    el.textContent = p.label;
    labels.append(el);
    spots.push({ at: new Vector3(p.x, closeup ? liftOf(p.line) + 1.32 : -0.02, zOf(p.line) + 0.25), el });
  }
}
scene.onAfterRenderObservable.add(() => {
  const vp = camera.viewport.toGlobal(engine.getRenderWidth(), engine.getRenderHeight());
  const m = scene.getTransformMatrix();
  for (const s of spots) {
    const q = Vector3.Project(s.at, Matrix.IdentityReadOnly, m, vp);
    s.el.style.left = `${(q.x / engine.getRenderWidth()) * canvas.clientWidth}px`;
    s.el.style.top = `${(q.y / engine.getRenderHeight()) * canvas.clientHeight}px`;
  }
});

let clock = 0;
function step(dt: number): void {
  clock += dt;
  frame.now = clock * 1000;
  // Visemes: a one-segment voice line per resident, restarted so it never ends.
  const now = performance.now() / 1000;
  for (const p of placed) {
    if (p.viseme !== undefined) characters.setSpeech(p.i, { times: Float32Array.of(0, 1e4), visemes: Uint8Array.of(p.viseme), energy: new Float32Array(0) }, now - 0.2);
  }
  characters.update(frame, () => true, shadowMatrices);
}
const manual = params.has('manual');
let last = performance.now();
engine.runRenderLoop(() => {
  if (!manual) {
    const now = performance.now();
    step(Math.min(0.1, (now - last) / 1000));
    last = now;
  }
  scene.render();
});
addEventListener('resize', () => engine.resize());
// A few frames so faces settle and lazy stages draw before screenshots.
for (let n = 0; n < 30; n++) step(1 / 30);
scene.render();

Object.assign(window, {
  __bench: {
    cells: placed.map((p) => p.label),
    advance(seconds: number) {
      for (let t = 0; t < seconds - 1e-6; t += 1 / 30) step(1 / 30);
    },
    characters,
    camera,
  },
  __ready: true,
});
