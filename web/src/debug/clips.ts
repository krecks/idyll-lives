/**
 * Dev-only clips bench (`bun run dev`, then /clips.html): models side by side, each playing every clip
 * of its file in turn, the way the game plays it, so a model's moving parts can be checked without
 * setting up a game. Built with the game's own `buildModel` and `partPose`, drawn by PlayCanvas
 * (`pcStage.ts`). Not part of the build.
 *
 * How each clip plays follows the model entry's `motions` (`clipBench.ts`).
 * A part moved by several clips at once takes all their poses, as in the game (`partPose`).
 *
 * URL: `?keys=model.fridge,model.sink` (which models; default every model whose entry has `motions`),
 * `?clip=water_on` (only that clip), `?t=0.2` (hold every clip at that time, for screenshots),
 * `?cols=4` (models a row), `?colors=body:%23c46b4a,stone:%232f3236` (the parts' colours),
 * `?renderer=webgl` (WebGL2; default WebGPU), `?alpha=…&beta=…&radius=…` (the camera), `?target=1.6`
 * (the height it looks at; `?target=x,y,z` a point), `?labels=0`.
 * From the console: `__clips.set(t)` holds every clip at `t`, `__clips.play()` lets them run again,
 * `__clips.solo('name')` plays only that clip (`solo(null)`: all).
 */

import { shownClips, times as clipTimes, type Shown } from './clipBench';
import { createStage } from './pcStage';
import type { PartColors } from '../core/protocol';
import { Instances } from '../render/pc/instancing';
import { buildModel, type ModelTemplate } from '../render/pc/models';
import { multiply, transformPoint, translation } from '../render/shared/math';
import { partPose } from '../render/shared/modelClips';
import { painted } from '../render/shared/modelEntry';

/** Metres between the models, past their own sizes. */
const GAP = 0.6;

const params = new URLSearchParams(location.search);
const stage = await createStage({
  sky: 0.6,
  sun: [-0.45, -1, -0.35],
  alpha: Number(params.get('alpha') ?? Math.PI / 2),
  beta: Number(params.get('beta') ?? 1.1),
  radius: 1,
  near: 0.02,
});
const { app, assets, root, orbit } = stage;

/** `?colors=body:#hex,stone:#hex` as the parts' colours. */
function colorsParam(): PartColors | undefined {
  const raw = params.get('colors');
  if (!raw) return undefined;
  return Object.fromEntries(raw.split(',').map((pair) => pair.split(':') as [string, string]));
}

const withMotions = () => assets.keys('model').filter((key) => (assets.get(key, 'model') as { motions?: unknown } | undefined)?.motions);
const keys = params.get('keys')?.split(',') ?? withMotions();
const colors = colorsParam();

interface Bench {
  key: string;
  template: ModelTemplate;
  /** Each mesh's draw (one copy). */
  draws: Instances[];
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
  const template = await buildModel(app, key, painted(entry, colors), [1, 1]);
  const clips = shownClips(template.motion?.clips, entry);
  const label = document.createElement('span');
  label.textContent = key.replace(/^model\./, '');
  if (params.get('labels') === '0') label.hidden = true;
  labels.appendChild(label);
  const draws = template.meshes.map((m) => new Instances(stage.device, root, m.mesh, m.material, { name: m.name }));
  benches.push({ key, template, draws, x: 0, z: 0, clips, label });
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

stage.lawn(widest + 8, depth + 8);

let solo: string | null = params.get('clip');
const held = params.get('t');
let hold: number | null = held === null ? null : Number(held);
let started = performance.now();

const posed = new Float32Array(16);
const place = new Float32Array(16);
const matrix = new Float32Array(16);
/** Poses every model at `seconds` into the bench. */
function poseAll(seconds: number): void {
  for (const b of benches) {
    const { template } = b;
    const { at, turn } = clipTimes(b.clips, seconds, solo, hold);
    translation(b.x, 0, b.z, place);
    // As the game: every clip that moves the part poses it, together (`partPose`).
    const playing = b.clips.flatMap((c) => (at.has(c.name) ? [{ clip: c.clip, time: at.get(c.name)!, kind: c.kind }] : []));
    for (let i = 0; i < template.meshes.length; i++) {
      const part = template.motion ? template.motion.parts[i] : -1;
      const moved = part >= 0 && partPose(playing, part, posed);
      b.draws[i].setMatrices(moved ? multiply(posed, place, matrix) : place, 1);
      b.draws[i].update();
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
// on the floor, at the middle of things that all hang higher (on a wall), or at `target`. Models that
// don't fit so (a tall wardrobe, a shower, a tree), at rest or with every clip open, are framed whole:
// looked at from their middle height, from as far as it takes.
const tallest = Math.max(1, ...benches.map((b) => b.template.height));
const top = Math.max(...benches.map((b) => b.template.height));
const lowest = Math.min(...benches.map((b) => b.template.bottom ?? 0));
const aim = orbit.target;
aim.y = lowest > 0.3 ? (lowest + top) / 2 : Math.min(0.75, tallest / 2);
const target = params.get('target')?.split(',').map(Number);
if (target?.length === 1) aim.y = target[0];
else if (target?.length === 3) [aim.x, aim.y, aim.z] = target;
orbit.radius = Number(params.get('radius')) || Math.max(2.6, widest * 0.75 + depth * 0.8 + tallest * 0.4);

/** The corners of every model's parts as placed, at rest and with all its clips open (`t=99`). */
function corners(): number[][] {
  const out: number[][] = [];
  for (const b of benches) {
    const open = b.clips.map((c) => ({ clip: c.clip, time: c.clip.duration, kind: c.kind }));
    translation(b.x, 0, b.z, place);
    b.template.meshes.forEach(({ mesh }, i) => {
      const lo = mesh.aabb.getMin();
      const hi = mesh.aabb.getMax();
      const part = b.template.motion ? b.template.motion.parts[i] : -1;
      for (const at of part >= 0 && partPose(open, part, posed) ? [place, multiply(posed, place, new Float32Array(16))] : [place]) {
        for (let k = 0; k < 8; k++) out.push(Array.from(transformPoint(at, k & 1 ? hi.x : lo.x, k & 2 ? hi.y : lo.y, k & 4 ? hi.z : lo.z)));
      }
    });
  }
  return out;
}
/** Whether all of `points` are on screen, clear of the buttons along the top. */
function inView(points: number[][]): boolean {
  return points.every(([x, y, z]) => {
    const [qx, qy] = stage.ndc(x, y, z);
    return Math.abs(qx) <= 0.95 && qy >= -0.95 && qy <= 0.86;
  });
}
if (!params.get('radius')) {
  const points = corners();
  if (points.length && !inView(points)) {
    if (!target) aim.y = (Math.min(...points.map((p) => p[1])) + Math.max(...points.map((p) => p[1]))) / 2;
    for (let n = 0; n < 80 && !inView(points); n++) orbit.radius *= 1.04;
  }
}

// The sun's shadow over everything shown.
stage.shadows.setSquare(0, 0, Math.hypot(widest, depth) / 2 + 2);

const projected = { x: 0, y: 0 };
stage.run(() => {
  poseAll((performance.now() - started) / 1000);
  // Each name in front of its model.
  for (const b of benches) {
    const o = b.template.outline;
    if (!stage.project(b.x + (o ? (o.minX + o.maxX) / 2 : 0), 0, b.z + (o ? o.maxZ : 0.5) + 0.12, projected)) continue;
    b.label.style.transform = `translate(${projected.x}px, ${projected.y}px) translate(-50%, 0)`;
  }
});
poseAll(0);
// (Two frames drawn before saying so: the shadow's and the picture's.)
requestAnimationFrame(() => requestAnimationFrame(() => Object.assign(window, { __app: app, __scene: app.scene, __clips: { set, play, solo: setSolo, benches }, __ready: true })));
