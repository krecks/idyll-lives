/**
 * Dev-only model gallery (`bun run dev`, then /gallery.html): draws manifest models in a grid with the
 * game's own loader and finishes under a noon sun, drawn by PlayCanvas (`pcStage.ts`), to check art
 * without starting a game. Not part of the build.
 *
 * URL: `?keys=model.sofa,model.fridge` (default: every `model.tree*`, `model.garden.*` and nature key),
 * `?filter=garden` (keys containing it), `?cols=6`, `?gap=5`, `?labels=0`, `?pack=…` like the game.
 * The camera looks at the models' fronts from the front right (a three-quarter view) and from as
 * close as frames them all, at their middle height (things on a wall included); `?alpha=0.6&beta=1.1&radius=6`
 * and `?target=1.4` (a height) or `?target=x,y,z` (a point) change that. (`alpha=1.571` is straight
 * from the front, `-1.571` from the back.)
 *
 * Still to come on PlayCanvas: the plants (their wind and tint, stage 5 of the port: nature keys are
 * left out, and the default list shows nothing) and the residents `?scale=adult|child|all` stands
 * beside each model (stage 7): the room for them is kept, so the framing is the same, but nobody
 * stands there yet.
 *
 * `?renderer=webgl` draws with WebGL2 (default WebGPU). `?still`: nothing moves (pictures that
 * compare pixel for pixel; `tools/bench/render.mjs --pages`).
 * `?camera=game` is the game's Live camera as a game starts (radius 24, beta 0.86, its longer lens),
 * from the game's angle onto a model facing that camera, aimed at the floor under the first model,
 * or the one `?at=<key or index>` names; `__gallery.aim(n)` aims it at model n. `alpha`, `beta`,
 * `radius` and `target` still change it. Sets `window.__ready` once the models are drawn.
 */

import { createStage } from './pcStage';
import { Instances } from '../render/pc/instancing';
import { buildModel } from '../render/pc/models';
import { translation } from '../render/shared/math';

const params = new URLSearchParams(location.search);
/** `?scale`: how many residents stand beside each model (stage 7 draws them). */
const SCALES: Record<string, number> = { adult: 2, child: 1, all: 4 };
const scaleParam = params.get('scale');
if (scaleParam && !SCALES[scaleParam]) console.warn(`[gallery] scale=${scaleParam}: say adult, child or all`);
const scale = (scaleParam && SCALES[scaleParam]) || 0;
if (scale) console.info('[gallery] residents for scale come with stage 7 of the PlayCanvas port: their room is kept, nobody stands there yet');
/** Residents beside a model: the nearest this far past its left side, the rest this far apart (m). */
const BESIDE = 0.45;
const APART = 0.6;
/** At most this many residents (the residents' `MAX_CHARACTERS`): the first models get theirs. */
const MAX_RESIDENTS = 64;
/** The game's Live camera as a new game starts; `alpha` is its angle turned half round, onto a model's front facing it. */
const GAME = { alpha: Math.PI * 0.38, beta: 0.86, radius: 24, fov: 0.7 };
/** The plants, drawn with the wind and their tint on PlayCanvas from stage 5. */
const NATURE = /^model\.(tree|pine|bush|hedge|garden|flowers|flowerbed|plant)/;
const game = params.get('camera') === 'game';
const alpha = Number(params.get('alpha') ?? (game ? GAME.alpha : Math.PI / 2 - 0.6));
const stage = await createStage({
  sky: 0.55,
  sun: [-0.45, -1, 0.35],
  alpha,
  beta: Number(params.get('beta') ?? (game ? GAME.beta : 1.15)),
  radius: 1,
  ...(game ? { fov: GAME.fov, near: 0.3 } : {}),
});
const { app, assets, orbit } = stage;

// Every model the game knows, from the base manifest and the packs it lists (most models live in packs).
const filter = params.get('filter');
const asked = params.get('keys')?.split(',') ?? assets.keys('model').filter((k) => (filter ? k.includes(filter) : NATURE.test(k)) && !k.endsWith('.far'));
const keys = asked.filter((k) => !NATURE.test(k));
if (keys.length < asked.length) console.info(`[gallery] ${asked.length - keys.length} plants left out: they come with stage 5 of the PlayCanvas port`);
if (asked.length === 0) console.warn(`[gallery] no models match ${filter ? `'${filter}'` : 'the default list'}`);

const cols = Number(params.get('cols')) || Math.max(1, Math.ceil(Math.sqrt(keys.length)));
const rows = Math.ceil(keys.length / cols);
const templates = await Promise.all(keys.map((key) => buildModel(app, key, assets.get(key, 'model'), [1, 1])));
const outlines = templates.map((t) => t.outline ?? { minX: -0.5, maxX: 0.5, minZ: -0.5, maxZ: 0.5 });
// Spots the widest model's size apart, plus a metre (small things close together), plus the residents beside each.
const widest = Math.max(...outlines.map((o) => Math.max(o.maxX - o.minX, o.maxZ - o.minZ)));
const gap = Number(params.get('gap')) || Math.max(1.5, widest + 1) + (scale ? BESIDE + scale * APART : 0);
// (From the game camera's distance, as far as it sees.)
stage.lawn(Math.max(game ? 90 : 0, cols * gap + 8), Math.max(game ? 90 : 0, rows * gap + 8));

const labels = document.getElementById('labels')!;
const spots: { key: string; at: [number, number, number]; el: HTMLSpanElement }[] = [];
/** Where each model stands. */
const places: [number, number, number][] = [];
const withResidents = Math.min(keys.length, Math.floor(MAX_RESIDENTS / Math.max(1, scale)));
/** What the models take up (m): x and z round their spots, y from the lowest bottom to the highest top. */
const box = { minX: Infinity, maxX: -Infinity, minY: Infinity, maxY: -Infinity, minZ: Infinity, maxZ: -Infinity };
const place = new Float32Array(16);
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
  places.push([x, 0, z]);
  // The residents' room, on the model's own left (-x: the right of the picture), as tall as an adult.
  if (i < withResidents && scale) {
    box.minX = Math.min(box.minX, x + out.minX - BESIDE - (scale - 1) * APART - 0.3);
    box.maxY = Math.max(box.maxY, 1.8);
  }
  translation(x, 0, z, place);
  for (const m of template.meshes) {
    const draw = new Instances(stage.device, stage.root, m.mesh, m.material, { name: m.name });
    draw.setMatrices(place, 1);
    draw.update();
  }
  if (params.get('labels') !== '0') {
    const el = document.createElement('span');
    el.textContent = key.replace(/^model\./, '');
    labels.append(el);
    spots.push({ key, at: [x, 0, z + Math.min(gap * 0.38, out.maxZ + 0.15)], el });
  }
});

// With nothing to show, frame an empty metre round the origin rather than infinities (NaN camera).
if (keys.length === 0) Object.assign(box, { minX: -0.5, maxX: 0.5, minY: 0, maxY: 1, minZ: -0.5, maxZ: 0.5 });

// Fronts face +z: from the front, a little to the right, at the models' middle height; as close
// as keeps the sphere round them all in view (a single cup from about 1 m, a bed from about 5).
const centre = orbit.target;
[centre.x, centre.y, centre.z] = [(box.minX + box.maxX) / 2, (box.minY + box.maxY) / 2, (box.minZ + box.maxZ) / 2];
const target = params.get('target')?.split(',').map(Number);
if (target?.length === 1) centre.y = target[0];
else if (target?.length === 3) [centre.x, centre.y, centre.z] = target;
const reach = Math.max(0.3, Math.hypot(box.maxX - box.minX, box.maxY - box.minY, box.maxZ - box.minZ) / 2);
orbit.radius = Number(params.get('radius')) || (game ? GAME.radius : (reach / Math.sin(0.8 / 2)) * 1.1);
/** Aims the game camera at the floor under model `n` (as the game focuses a point on the ground). */
const aim = (n: number): void => {
  if (places[n]) [centre.x, centre.y, centre.z] = places[n];
};
if (game) {
  const at = params.get('at') ?? '0';
  if (!target) aim(keys.includes(at) ? keys.indexOf(at) : Number(at) || 0);
}

// The sun's shadow over everything shown.
stage.shadows.setSquare((box.minX + box.maxX) / 2, (box.minZ + box.maxZ) / 2, Math.hypot(box.maxX - box.minX, box.maxZ - box.minZ) / 2 + 2);

const projected = { x: 0, y: 0 };
stage.run(() => {
  for (const s of spots) {
    if (!stage.project(s.at[0], s.at[1], s.at[2], projected)) continue;
    s.el.style.left = `${projected.x}px`;
    s.el.style.top = `${projected.y}px`;
  }
});
// (Two frames drawn before saying so: the shadow's and the picture's.)
requestAnimationFrame(() => requestAnimationFrame(() => Object.assign(window, { __app: app, __scene: app.scene, __gallery: { keys, aim, characters: null }, __ready: true })));
