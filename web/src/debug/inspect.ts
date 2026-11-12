/**
 * Dev-only model inspector (`bun run dev`, then /inspect.html, or the main menu's "Inspect models", which the
 * dev server shows under Settings): one model at a time, picked from every
 * model the manifests know, drawn with the game's own loader and finishes under a noon sun by
 * PlayCanvas (`pcStage.ts`), to look at its every side, what it's made of and its clips without
 * starting a game. Not part of the build.
 *
 * - The list on the left: every model by family (the folder its file is in), filtered as you type
 *   (`/` goes there); a dot marks those whose entry has `motions`. ↑/↓ pick the one before or after.
 *   Drop a `.glb` onto the page to look at a file the manifests don't name (drawn with its own
 *   materials: none map onto the game's finishes).
 * - The panel on the right (`inspectPanel.ts`): its content objects (layer, footprint, a top's radius),
 *   its size, triangles and draw calls (and its far version's), its meshes (hover one to see it alone,
 *   click to keep it so), its moving parts and clips, the manifest entry, and the gate's report on it
 *   (`cargo check-glb <key>`, run by the dev server: `vite.config.ts` `modelWatch`).
 * - The timeline: every clip as the game plays it (`clipBench.ts`, as `/clips.html`), or one clip on
 *   its own, from rest to open; play (Space), Rest (0), Open (9), a frame back or on (← →, paused),
 *   the speed and looping; drag the bar to scrub.
 * - Over the picture: the camera from the front, a side, the back or above (1–6) and framed (F);
 *   wireframe (W), the box round it as it is posed (B), its content footprint and the wall's face (P),
 *   a top's disc (R), the origin's axes (A: x its right red, y green, z its front blue), a 1.75 m figure for
 *   scale (S), its far version (L), the lawn (G), and the shading (lit, or `?diag=albedo|normals`).
 * - When `cargo art <family>` writes the model's file again, it is built again from it in place
 *   (camera, clip and time kept); a manifest written again reloads the page (the view kept: it is
 *   all in the URL).
 *
 * URL: `?key=model.fridge` (default the first), `&clip=door` (one clip), `&t=0.4` (held there),
 * `&show=wire,box,foot,radius,axes,scale,far` (and `noground`), `&alpha=…&beta=…&radius=…&target=x,y,z`
 * (the camera; the model is framed without them), `&renderer=webgl`, `&pack=…`, `&diag=albedo|normals`.
 * From the console: `__inspect.select(key)`, `.set(t)`, `.play()`, `.solo(name | null)`, `.current`.
 * Sets `window.__ready` once the first model is drawn.
 */

import * as pc from '../render/pc/engine';
import { cycleLength, shownClips, times } from './clipBench';
import { esc, familyOf, filePath, gateHtml, panelHtml, type PanelModel } from './inspectPanel';
import { createStage } from './pcStage';
import type { ModelEntry } from '../assets/types';
import { Content, type ObjectDef } from '../content/content';
import { Instances } from '../render/pc/instancing';
import { buildModel, disposeTemplate, type ModelTemplate } from '../render/pc/models';
import { windStill } from '../render/pc/nature';
import { transformPoint } from '../render/shared/math';
import { partPose } from '../render/shared/modelClips';
import { WALL_HEIGHT } from '../render/shared/walls';

const params = new URLSearchParams(location.search);
const num = (name: string): number | null => {
  const v = Number(params.get(name));
  return params.has(name) && Number.isFinite(v) ? v : null;
};

const stage = await createStage({
  sky: 0.6,
  sun: [-0.45, -1, -0.35],
  alpha: num('alpha') ?? Math.PI / 2 - 0.6,
  beta: num('beta') ?? 1.15,
  radius: num('radius') ?? 3,
  near: 0.01,
  wheelPrecision: 40,
});
const { app, assets, root, orbit, canvas } = stage;
// The camera's fov (rad), as `createStage` sets it.
const FOV = 0.8;
stage.lawn(60, 60);
const ground = root.findByName('ground') as pc.Entity | null;
const content = await Content.load(`${import.meta.env.BASE_URL}content/base.json`).catch((err: unknown) => {
  console.warn('[inspect] no content: the objects, footprints and radii are left out', err);
  return null;
});

// ---- What's shown

/** A template drawn: a draw (one copy) per mesh. */
interface Drawn {
  template: ModelTemplate;
  draws: Instances[];
}

interface Current extends PanelModel {
  full: Drawn;
  farDrawn: Drawn | null;
}

let current: Current | null = null;
/** Every model the manifests know (no far versions), and the files dropped onto the page. */
const keys = assets.keys('model').filter((k) => !k.endsWith('.far'));
const dropped = new Map<string, { name: string; entry: ModelEntry }>();

/** Toggles over the picture (`?show=`). */
const show = new Set((params.get('show') ?? 'foot').split(',').filter(Boolean));
if (params.get('ground') === '0') show.add('noground');

/** The timeline: one clip on its own (`solo`), or all as the game plays them; where it is, and how it goes. */
let solo: string | null = params.get('clip');
let t = num('t') ?? 0;
let playing = !params.has('t');
/** In the game's cycle (no `solo`): every clip held at this time (Rest 0, Open Infinity), until played again. */
let hold: number | null = null;
let speed = 1;
let loop = true;
/** A mesh (or a moving part's meshes) seen alone: hovered, and kept by a click. */
let alone: { mesh?: number; part?: number } | null = null;
let kept: { mesh?: number; part?: number } | null = null;

function draw(template: ModelTemplate): Drawn {
  const draws = template.meshes.map((m) => {
    const d = new Instances(stage.device, root, m.mesh, m.material, { name: m.name, cull: false });
    windStill(d.meshInstance, m);
    return d;
  });
  return { template, draws };
}

function undraw(d: Drawn | null): void {
  if (!d) return;
  for (const x of d.draws) x.destroy();
  disposeTemplate(d.template);
}

/** `url` asked for afresh (the file cache is by URL): `?v=<version>`. */
function fresh(url: string | undefined, version: number): string | undefined {
  if (!url) return url;
  const u = new URL(url, location.href);
  u.searchParams.set('v', String(version));
  return u.href;
}

let selecting = 0;
/**
 * Shows `key` (a model's, or a dropped file's), built from its file afresh if `version` is given;
 * frames it unless `keepView`.
 */
async function select(key: string, { version, keepView = false, retried = false }: { version?: number; keepView?: boolean; retried?: boolean } = {}): Promise<void> {
  const token = ++selecting;
  const drop = dropped.get(key);
  const base = drop ? drop.entry : assets.get(key, 'model');
  const entry = base && version ? { ...base, url: fresh(base.url, version), far: fresh(base.far, version) } : base;
  const defs: ObjectDef[] = content?.objectList.filter((o) => o.model === key) ?? [];
  const footprint = defs[0]?.footprint ?? [1, 1];
  const [template, far] = await Promise.all([buildModel(app, key, entry, footprint), entry?.far ? buildModel(app, key, entry, footprint, { far: true }) : Promise.resolve(null)]);
  if (token !== selecting) {
    disposeTemplate(template);
    if (far) disposeTemplate(far);
    return;
  }
  // (A file half written when the change was told: once more, shortly; then as it is, failed.)
  if (version && template.failed?.length && !retried) {
    disposeTemplate(template);
    if (far) disposeTemplate(far);
    setTimeout(() => void select(key, { version: version + 1, keepView: true, retried: true }), 600);
    return;
  }
  const same = current?.key === key;
  undraw(current?.full ?? null);
  undraw(current?.farDrawn ?? null);
  const clips = shownClips(template.motion?.clips, base);
  current = { key, entry: base, dropped: drop?.name, template, far, clips, defs, full: draw(template), farDrawn: far ? draw(far) : null };
  if (!same) {
    alone = kept = null;
    if (solo && !clips.some((c) => c.name === solo)) solo = null;
    if (!keepView) t = 0;
  }
  placeFigure();
  applyLooks();
  renderList();
  renderPanel();
  renderClips();
  if (!keepView) frame();
  else stage.shadows.setSquare(0, 0, shadowReach());
  saveUrl();
  for (const b of document.querySelectorAll<HTMLButtonElement>('#models button.on')) b.scrollIntoView({ block: 'nearest' });
}

// ---- Posing

const posed = new Float32Array(16);
const identity = new Float32Array([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]);
/** Each mesh's matrix now (model space), for the box drawn round it. */
let matrices: Float32Array[] = [];

/** How long the timeline is: the clip shown alone, or the game's cycle of them all (an ambient clip's round if that's all). */
function length(): number {
  if (!current) return 0;
  if (solo) return current.clips.find((c) => c.name === solo)?.clip.duration ?? 0;
  const cycle = cycleLength(current.clips);
  return cycle > 0 ? cycle : Math.max(0, ...current.clips.map((c) => c.clip.duration));
}

/** Where each clip is now. */
function clipTimes(): { at: Map<string, number>; turn: string | null } {
  if (!current) return { at: new Map(), turn: null };
  if (solo) return { at: new Map([[solo, Math.min(t, length())]]), turn: solo };
  return times(current.clips, t, null, hold);
}

function pose(): string | null {
  matrices = [];
  if (!current) return null;
  const farShown = show.has('far') && current.farDrawn;
  const d = farShown ? current.farDrawn! : current.full;
  const { at, turn } = clipTimes();
  const playingClips = current.clips.flatMap((c) => (at.has(c.name) ? [{ clip: c.clip, time: at.get(c.name)!, kind: c.kind }] : []));
  const motion = farShown ? undefined : d.template.motion;
  d.draws.forEach((draw, i) => {
    const part = motion ? motion.parts[i] : -1;
    const m = part >= 0 && partPose(playingClips, part, posed) ? posed.slice() : identity;
    draw.setMatrices(m, 1);
    draw.update();
    matrices.push(m);
  });
  return turn;
}

/** Which draws show: the full model or its far version, all its meshes or the one (or the part) seen alone. */
function applyLooks(): void {
  if (!current) return;
  const farShown = show.has('far') && !!current.farDrawn;
  const only = alone ?? kept;
  const parts = current.template.motion?.parts;
  current.full.draws.forEach((d, i) => {
    d.enabled = !farShown && (!only || (only.mesh !== undefined ? only.mesh === i : parts?.[i] === only.part));
  });
  current.farDrawn?.draws.forEach((d) => (d.enabled = farShown));
  for (const d of [...current.full.draws, ...(current.farDrawn?.draws ?? [])]) d.meshInstance.renderStyle = show.has('wire') ? pc.RENDERSTYLE_WIREFRAME : pc.RENDERSTYLE_SOLID;
  if (ground) ground.enabled = !show.has('noground');
  for (const b of document.querySelectorAll<HTMLElement>('[data-show]')) b.classList.toggle('on', b.dataset.show === 'ground' ? !show.has('noground') : show.has(b.dataset.show!));
  for (const row of document.querySelectorAll<HTMLElement>('#panel tr[data-mesh], #panel tr[data-part]')) {
    const mesh = row.dataset.mesh === undefined ? undefined : Number(row.dataset.mesh);
    const part = row.dataset.part === undefined ? undefined : Number(row.dataset.part);
    row.classList.toggle('on', !!kept && kept.mesh === mesh && kept.part === part);
  }
}

// ---- Framing

/** The corners of every mesh's box, at rest and with every clip open (model space). */
function corners(): number[][] {
  if (!current) return [];
  const { template, clips } = current;
  const open = clips.map((c) => ({ clip: c.clip, time: c.clip.duration, kind: c.kind }));
  const out: number[][] = [];
  template.meshes.forEach(({ mesh }, i) => {
    const lo = mesh.aabb.getMin();
    const hi = mesh.aabb.getMax();
    const part = template.motion ? template.motion.parts[i] : -1;
    for (const at of part >= 0 && partPose(open, part, posed) ? [identity, posed] : [identity]) {
      for (let k = 0; k < 8; k++) out.push(Array.from(transformPoint(at, k & 1 ? hi.x : lo.x, k & 2 ? hi.y : lo.y, k & 4 ? hi.z : lo.z)));
    }
  });
  return out;
}

/**
 * A 1.75 m grown-up for scale (`S`), 0.45 m across and 0.25 deep, 1 m marked, on the model's own left (-x: the right of
 * the picture from the front, as the gallery stands its residents): where it stands for the model shown.
 */
let figure: { lo: number[]; hi: number[] } | null = null;
function placeFigure(): void {
  const box = corners();
  const side = box.length ? Math.min(...box.map((p) => p[0])) : -0.5;
  figure = { lo: [side - 0.7, 0, -0.125], hi: [side - 0.25, 1.75, 0.125] };
}

function shadowReach(): number {
  const points = corners();
  return points.length ? Math.max(...points.map(([x, , z]) => Math.hypot(x, z))) + 1.5 : 3;
}

/** Whether all of `points` are well on screen. */
function inView(points: number[][]): boolean {
  return points.every(([x, y, z]) => {
    const [qx, qy] = stage.ndc(x, y, z);
    return Math.abs(qx) <= 0.9 && Math.abs(qy) <= 0.9;
  });
}

/** The camera on the model's middle, as close as keeps it whole (at rest and open) in view (`keepRadius`: as far as it is). */
function frame(keepRadius = false): void {
  const points = corners();
  if (show.has('scale') && figure) for (let k = 0; k < 8; k++) points.push([k & 1 ? figure.hi[0] : figure.lo[0], k & 2 ? figure.hi[1] : figure.lo[1], k & 4 ? figure.hi[2] : figure.lo[2]]);
  stage.shadows.setSquare(0, 0, shadowReach());
  if (!points.length) return;
  const lo = [0, 1, 2].map((k) => Math.min(...points.map((p) => p[k])));
  const hi = [0, 1, 2].map((k) => Math.max(...points.map((p) => p[k])));
  [orbit.target.x, orbit.target.y, orbit.target.z] = [(lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, (lo[2] + hi[2]) / 2];
  if (keepRadius) return saveUrl();
  const reach = Math.max(0.05, Math.hypot(hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]) / 2);
  orbit.radius = (reach / Math.sin(FOV / 2)) * 0.8;
  for (let n = 0; n < 60 && !inView(points); n++) orbit.radius *= 1.05;
  saveUrl();
}

/** The camera's angles (`OrbitControls`: alpha π/2 looks at a model's front, 0 at its right side, +x, and π at its left). */
const VIEWS: { label: string; key: string; alpha: number; beta: number }[] = [
  { label: 'Front', key: '1', alpha: Math.PI / 2, beta: Math.PI / 2 - 0.12 },
  { label: 'Right', key: '2', alpha: 0, beta: Math.PI / 2 - 0.12 },
  { label: 'Back', key: '3', alpha: -Math.PI / 2, beta: Math.PI / 2 - 0.12 },
  { label: 'Left', key: '4', alpha: Math.PI, beta: Math.PI / 2 - 0.12 },
  { label: 'Top', key: '5', alpha: Math.PI / 2, beta: 0.02 },
  { label: '¾', key: '6', alpha: Math.PI / 2 - 0.6, beta: 1.15 },
];
function view(v: (typeof VIEWS)[number]): void {
  orbit.alpha = v.alpha;
  orbit.beta = v.beta;
  saveUrl();
}

// ---- Overlays (lines in engine space: game space with z mirrored, `pc/space.ts`)

const lines: pc.Vec3[] = [];
const colours: pc.Color[] = [];
const COLOUR = {
  box: new pc.Color(1, 0.55, 0.1),
  foot: new pc.Color(1, 0.9, 0.2),
  wall: new pc.Color(0.85, 0.85, 0.9),
  radius: new pc.Color(0.2, 0.9, 1),
  x: new pc.Color(0.95, 0.2, 0.2),
  y: new pc.Color(0.2, 0.85, 0.3),
  z: new pc.Color(0.25, 0.45, 1),
  scale: new pc.Color(0.3, 0.3, 0.35),
};
function seg(a: number[], b: number[], colour: pc.Color): void {
  lines.push(new pc.Vec3(a[0], a[1], -a[2]), new pc.Vec3(b[0], b[1], -b[2]));
  colours.push(colour, colour);
}
function boxLines(lo: number[], hi: number[], colour: pc.Color): void {
  const c = (k: number) => [k & 1 ? hi[0] : lo[0], k & 2 ? hi[1] : lo[1], k & 4 ? hi[2] : lo[2]];
  for (const [a, b] of [[0, 1], [2, 3], [4, 5], [6, 7], [0, 2], [1, 3], [4, 6], [5, 7], [0, 4], [1, 5], [2, 6], [3, 7]]) seg(c(a), c(b), colour);
}
function rect(x0: number, z0: number, x1: number, z1: number, y: number, colour: pc.Color): void {
  seg([x0, y, z0], [x1, y, z0], colour);
  seg([x1, y, z0], [x1, y, z1], colour);
  seg([x1, y, z1], [x0, y, z1], colour);
  seg([x0, y, z1], [x0, y, z0], colour);
}

/** The wall's face lies this far into a footprint's back tile (m; the gate's `WALL`, `checkglb.rs`). */
const WALL_FACE = 0.07;
const FLOOR = 0.004;

function overlays(): void {
  lines.length = 0;
  colours.length = 0;
  if (!current) return;
  const def = current.defs[0];
  if (show.has('box') && matrices.length) {
    const lo = [Infinity, Infinity, Infinity];
    const hi = [-Infinity, -Infinity, -Infinity];
    const d = show.has('far') && current.farDrawn ? current.farDrawn : current.full;
    d.template.meshes.forEach(({ mesh }, i) => {
      if (!d.draws[i].enabled) return;
      const a = mesh.aabb.getMin();
      const b = mesh.aabb.getMax();
      for (let k = 0; k < 8; k++) {
        const p = transformPoint(matrices[i] ?? identity, k & 1 ? b.x : a.x, k & 2 ? b.y : a.y, k & 4 ? b.z : a.z);
        for (let j = 0; j < 3; j++) {
          lo[j] = Math.min(lo[j], p[j]);
          hi[j] = Math.max(hi[j], p[j]);
        }
      }
    });
    if (lo[0] <= hi[0]) boxLines(lo, hi, COLOUR.box);
  }
  if (show.has('foot') && def?.footprint && def.layer !== 'surface') {
    const [w, d] = def.footprint;
    // At the floor, or up where a wall thing or a ceiling thing hangs (its own bottom).
    const y = def.layer === 'wall' || def.layer === 'ceiling' ? Math.max(FLOOR, current.template.bottom ?? 0) : FLOOR;
    rect(-w / 2, -d / 2, w / 2, d / 2, y, COLOUR.foot);
    for (let x = 1; x < w; x++) seg([-w / 2 + x, y, -d / 2], [-w / 2 + x, y, d / 2], COLOUR.foot);
    for (let z = 1; z < d; z++) seg([-w / 2, y, -d / 2 + z], [w / 2, y, -d / 2 + z], COLOUR.foot);
    // The wall's face, for what hangs on a wall or stands against one indoors.
    if (def.layer === 'wall' || ((def.layer ?? 'floor') === 'floor' && !def.outdoors)) {
      const z = -d / 2 + WALL_FACE;
      const top = def.layer === 'wall' ? WALL_HEIGHT : FLOOR;
      seg([-w / 2, FLOOR, z], [w / 2, FLOOR, z], COLOUR.wall);
      if (top > FLOOR) {
        seg([-w / 2, top, z], [w / 2, top, z], COLOUR.wall);
        seg([-w / 2, FLOOR, z], [-w / 2, top, z], COLOUR.wall);
        seg([w / 2, FLOOR, z], [w / 2, top, z], COLOUR.wall);
      }
    }
  }
  if (show.has('radius') && def?.layer === 'surface' && content) {
    const r = content.radius(def);
    const n = 48;
    for (let i = 0; i < n; i++) {
      const a = (i / n) * Math.PI * 2;
      const b = ((i + 1) / n) * Math.PI * 2;
      seg([Math.cos(a) * r, FLOOR, Math.sin(a) * r], [Math.cos(b) * r, FLOOR, Math.sin(b) * r], COLOUR.radius);
    }
  }
  if (show.has('axes')) {
    seg([0, FLOOR, 0], [0.5, FLOOR, 0], COLOUR.x);
    seg([0, 0, 0], [0, 0.5, 0], COLOUR.y);
    seg([0, FLOOR, 0], [0, FLOOR, 0.5], COLOUR.z);
  }
  if (show.has('scale') && figure) {
    boxLines(figure.lo, figure.hi, COLOUR.scale);
    seg([figure.lo[0], 1, figure.hi[2]], [figure.hi[0], 1, figure.hi[2]], COLOUR.scale);
  }
  if (lines.length) app.drawLines(lines, colours, false);
}

// ---- The list

const search = document.getElementById('search') as HTMLInputElement;
search.value = params.get('filter') ?? '';
const models = document.getElementById('models')!;

/** The keys the list shows now, in its order. */
let listed: string[] = [];
function renderList(): void {
  const q = search.value.trim().toLowerCase();
  const families = new Map<string, string[]>();
  if (dropped.size) families.set('dropped files', [...dropped.keys()]);
  for (const key of keys) {
    if (q && !key.toLowerCase().includes(q)) continue;
    const family = familyOf(assets.get(key, 'model'));
    families.set(family, [...(families.get(family) ?? []), key]);
  }
  const order = [...families.keys()].sort((a, b) => (a === 'dropped files' ? -1 : b === 'dropped files' ? 1 : a.localeCompare(b)));
  listed = order.flatMap((f) => families.get(f)!.sort());
  models.innerHTML = order
    .map((f) => {
      const items = families
        .get(f)!
        .map((key) => {
          const motions = !dropped.has(key) && (assets.get(key, 'model') as { motions?: unknown } | undefined)?.motions;
          const label = dropped.get(key)?.name ?? key.replace(/^model\./, '');
          return `<button data-key="${esc(key)}" class="${key === current?.key ? 'on' : ''}" title="${esc(key)}"><span class="dot ${motions ? 'clips' : ''}"></span>${esc(label)}</button>`;
        })
        .join('');
      return `<h3>${esc(f)} <span>${families.get(f)!.length}</span></h3>${items}`;
    })
    .join('');
}
search.addEventListener('input', () => {
  renderList();
  saveUrl();
});
search.addEventListener('keydown', (e) => {
  // Enter: the first one listed.
  if (e.key === 'Enter' && listed.length) void select(listed[0]);
  if (e.key === 'Escape') search.blur();
});
models.addEventListener('click', (e) => {
  const key = (e.target as HTMLElement).closest<HTMLElement>('button[data-key]')?.dataset.key;
  if (key) void select(key);
});

// ---- The panel

const panel = document.getElementById('panel')!;
/** The key the gate last ran for (run again when its file comes again). */
let gated: string | null = null;
function renderPanel(): void {
  if (!current) return;
  const closed = new Set([...panel.querySelectorAll('details')].filter((d) => !d.open).map((d) => d.querySelector('summary')?.textContent?.replace(/ \(\d+\)$/, '')));
  panel.innerHTML = panelHtml(current, content);
  // (Sections closed by hand stay closed from model to model.)
  for (const d of panel.querySelectorAll('details')) if (closed.has(d.querySelector('summary')?.textContent?.replace(/ \(\d+\)$/, ''))) d.open = false;
  document.getElementById('gate')?.addEventListener('click', () => void runGate());
  if (gated === current.key) void runGate();
  applyLooks();
}

async function runGate(): Promise<void> {
  const key = current?.key;
  const state = document.getElementById('gate-state');
  const out = document.getElementById('gate-out');
  if (!key || !state || !out) return;
  gated = key;
  state.textContent = 'running…';
  try {
    const res = await fetch(`/__debug/gate?key=${encodeURIComponent(key)}`);
    const { code, text } = (await res.json()) as { code: number; text: string };
    if (current?.key !== key) return;
    state.innerHTML = code === 0 ? '<span style="color:var(--ok)">passed</span>' : '<span style="color:var(--fail)">failed</span>';
    out.innerHTML = gateHtml(text.trim());
    out.hidden = false;
  } catch (err) {
    state.textContent = `couldn't run it: ${String(err)}`;
  }
}

function rowTarget(e: Event): { mesh?: number; part?: number; clip?: string } | null {
  const row = (e.target as HTMLElement).closest<HTMLElement>('tr[data-mesh], tr[data-part], tr[data-clip]');
  if (!row) return null;
  if (row.dataset.clip !== undefined) return { clip: row.dataset.clip };
  return row.dataset.mesh !== undefined ? { mesh: Number(row.dataset.mesh) } : { part: Number(row.dataset.part) };
}
panel.addEventListener('mouseover', (e) => {
  const target = rowTarget(e);
  const next = target && target.clip === undefined ? target : null;
  if (JSON.stringify(next) === JSON.stringify(alone)) return;
  alone = next;
  applyLooks();
});
panel.addEventListener('mouseleave', () => {
  alone = null;
  applyLooks();
});
panel.addEventListener('click', (e) => {
  const target = rowTarget(e);
  if (!target) return;
  if (target.clip !== undefined) return setSolo(target.clip);
  kept = JSON.stringify(kept) === JSON.stringify(target) ? null : target;
  alone = null;
  applyLooks();
});

// ---- The timeline

const clipbar = document.getElementById('clipbar')!;
const playButton = document.getElementById('play') as HTMLButtonElement;
const scrub = document.getElementById('scrub') as HTMLInputElement;
const timeText = document.getElementById('time')!;
const speedSelect = document.getElementById('speed') as HTMLSelectElement;
const loopBox = document.getElementById('loop') as HTMLInputElement;

function renderClips(): void {
  const clips = current?.clips ?? [];
  const buttons = clips.length ? [['', 'All, as the game plays them'], ...clips.map((c) => [c.name, c.name])] : [['', 'No clips']];
  clipbar.innerHTML = buttons.map(([name, label]) => `<button class="pill ${(name || null) === solo ? 'on' : ''}" data-clip="${esc(name)}">${esc(label)}</button>`).join('');
  document.getElementById('timeline')!.style.opacity = clips.length ? '1' : '0.5';
}
clipbar.addEventListener('click', (e) => {
  const name = (e.target as HTMLElement).closest<HTMLElement>('button[data-clip]')?.dataset.clip;
  if (name !== undefined) setSolo(name || null);
});

function setSolo(name: string | null): void {
  solo = name;
  t = 0;
  hold = null;
  playing = true;
  renderClips();
  saveUrl();
}
function set(seconds: number): void {
  playing = false;
  if (!solo && !Number.isFinite(seconds)) hold = Infinity;
  else {
    hold = null;
    t = Math.max(0, Math.min(seconds, length()));
  }
  saveUrl();
}
function play(): void {
  hold = null;
  if (t >= length() - 1e-6) t = 0;
  playing = true;
  saveUrl();
}
function rest(): void {
  playing = false;
  t = 0;
  hold = solo ? null : 0;
  saveUrl();
}
function open(): void {
  playing = false;
  if (solo) t = length();
  else hold = Infinity;
  saveUrl();
}
playButton.onclick = () => (playing ? set(t) : play());
document.getElementById('rest')!.onclick = rest;
document.getElementById('open')!.onclick = open;
speedSelect.onchange = () => (speed = Number(speedSelect.value));
loopBox.onchange = () => (loop = loopBox.checked);
scrub.addEventListener('input', () => set(Number(scrub.value)));

/** Moves the timeline on by `dt` real seconds. */
function advance(dt: number): void {
  if (!playing) return;
  const end = length();
  t += dt * speed;
  if (end <= 0) t = 0;
  else if (t > end) {
    if (loop || !solo) t %= end;
    else {
      t = end;
      playing = false;
    }
  }
}

// ---- The camera's buttons and the toggles over the picture

const TOGGLES: { show: string; label: string; key: string }[] = [
  { show: 'wire', label: 'Wire', key: 'w' },
  { show: 'box', label: 'Box', key: 'b' },
  { show: 'foot', label: 'Footprint', key: 'p' },
  { show: 'radius', label: 'Radius', key: 'r' },
  { show: 'axes', label: 'Axes', key: 'a' },
  { show: 'scale', label: 'Scale', key: 's' },
  { show: 'far', label: 'Far', key: 'l' },
  { show: 'ground', label: 'Lawn', key: 'g' },
];
const DIAGS = ['lit', 'albedo', 'normals'];
const cameraBar = document.getElementById('camera')!;
cameraBar.innerHTML = [
  ...VIEWS.map((v) => `<button class="pill" data-view="${v.key}">${esc(v.label)}<kbd>${v.key}</kbd></button>`),
  `<button class="pill" data-frame>Frame<kbd>F</kbd></button>`,
  '<span style="width:8px"></span>',
  ...TOGGLES.map((x) => `<button class="pill" data-show="${x.show}">${esc(x.label)}<kbd>${x.key.toUpperCase()}</kbd></button>`),
  `<select id="diag">${DIAGS.map((d) => `<option ${(params.get('diag') ?? 'lit') === d ? 'selected' : ''}>${d}</option>`).join('')}</select>`,
].join('');
function toggle(name: string): void {
  if (name === 'ground') {
    if (show.has('noground')) show.delete('noground');
    else show.add('noground');
  } else if (show.has(name)) show.delete(name);
  else show.add(name);
  applyLooks();
  saveUrl();
}
cameraBar.addEventListener('click', (e) => {
  const el = (e.target as HTMLElement).closest<HTMLElement>('button');
  if (!el) return;
  if (el.dataset.view) view(VIEWS.find((v) => v.key === el.dataset.view)!);
  else if (el.dataset.frame !== undefined) frame();
  else if (el.dataset.show) toggle(el.dataset.show);
});
// (The shading is read once, as the page loads: `render/pc/shaders/diagView.ts`.)
(document.getElementById('diag') as HTMLSelectElement).onchange = (e) => {
  const value = (e.target as HTMLSelectElement).value;
  const url = new URL(location.href);
  if (value === 'lit') url.searchParams.delete('diag');
  else url.searchParams.set('diag', value);
  location.href = url.href;
};

// ---- Keys

window.addEventListener('keydown', (e: KeyboardEvent) => {
  const target = e.target as HTMLElement;
  if (target instanceof HTMLInputElement && target.type !== 'range') return;
  if (target instanceof HTMLSelectElement || e.metaKey || e.ctrlKey || e.altKey) return;
  const onCanvas = target === canvas;
  const k = e.key.toLowerCase();
  if (e.key === '/') {
    search.focus();
    search.select();
  } else if (e.key === ' ') playing ? set(t) : play();
  else if (e.key === '0') rest();
  else if (e.key === '9') open();
  else if (!onCanvas && (e.key === 'ArrowDown' || e.key === 'ArrowUp')) {
    const at = current ? listed.indexOf(current.key) : -1;
    const next = listed[Math.max(0, Math.min(listed.length - 1, at + (e.key === 'ArrowDown' ? 1 : -1)))];
    if (next && next !== current?.key) void select(next);
  } else if (!onCanvas && (e.key === 'ArrowLeft' || e.key === 'ArrowRight')) {
    set((hold === Infinity ? length() : t) + (e.key === 'ArrowRight' ? 1 : -1) / 60);
  } else if (k === 'f') frame();
  else if (VIEWS.some((v) => v.key === k)) view(VIEWS.find((v) => v.key === k)!);
  else if (TOGGLES.some((x) => x.key === k)) toggle(TOGGLES.find((x) => x.key === k)!.show);
  else return;
  e.preventDefault();
});

// ---- A dropped file

const dropHint = document.getElementById('drop')!;
window.addEventListener('dragover', (e: DragEvent) => {
  if (!e.dataTransfer?.types.includes('Files')) return;
  e.preventDefault();
  dropHint.classList.add('show');
});
window.addEventListener('dragleave', (e: DragEvent) => {
  if (e.relatedTarget === null) dropHint.classList.remove('show');
});
window.addEventListener('drop', (e: DragEvent) => {
  e.preventDefault();
  dropHint.classList.remove('show');
  const file = [...(e.dataTransfer?.files ?? [])].find((f) => /\.glb$/i.test(f.name));
  if (!file) return toast('drop a .glb file');
  const key = `file:${file.name}`;
  const old = dropped.get(key);
  if (old?.entry.url) URL.revokeObjectURL(old.entry.url);
  dropped.set(key, { name: file.name, entry: { type: 'model', url: URL.createObjectURL(file) } });
  void select(key);
});

// ---- A file written again (`cargo art`): the dev server says so (`vite.config.ts` `modelWatch`)

const toastEl = document.getElementById('toast')!;
let toastTimer: ReturnType<typeof setTimeout> | undefined;
function toast(text: string): void {
  toastEl.textContent = text;
  toastEl.classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => toastEl.classList.remove('show'), 2500);
}
import.meta.hot?.on('inspect:changed', ({ files }: { files: string[] }) => {
  // A manifest: its entries (and maybe which models there are) changed: everything again, the view kept (the URL).
  if (files.some((f) => f.endsWith('manifest.json'))) {
    saveUrl();
    location.reload();
    return;
  }
  if (!current || current.dropped) return;
  const mine = [filePath(current.entry?.url), filePath(current.entry?.far)].filter(Boolean);
  const hit = files.find((f) => mine.some((p) => p.endsWith(f)));
  if (!hit) return;
  void select(current.key, { version: Date.now(), keepView: true }).then(() => toast(`reloaded ${hit.split('/').pop()}`));
});

// ---- The URL: the view, so a reload (or a link) shows the same

let savedAt = 0;
function saveUrl(): void {
  const u = new URL(location.href);
  const p = u.searchParams;
  const put = (name: string, value: string | null) => (value === null || value === '' ? p.delete(name) : p.set(name, value));
  put('key', current && !current.dropped ? current.key : p.get('key'));
  put('clip', solo);
  put('t', playing ? null : (hold === Infinity ? length() : t).toFixed(3));
  put('show', [...show].filter((s) => s !== 'noground').join(',') || 'none');
  put('ground', show.has('noground') ? '0' : null);
  put('filter', search.value.trim() || null);
  put('alpha', orbit.alpha.toFixed(3));
  put('beta', orbit.beta.toFixed(3));
  put('radius', orbit.radius.toFixed(3));
  put('target', [orbit.target.x, orbit.target.y, orbit.target.z].map((v) => v.toFixed(3)).join(','));
  history.replaceState(null, '', u.href);
  savedAt = performance.now();
}

// ---- Start

const savedPose = { alpha: orbit.alpha, beta: orbit.beta, radius: orbit.radius };
stage.run((dt) => {
  advance(dt);
  const turn = pose();
  overlays();
  const end = length();
  const now = hold === Infinity ? end : hold === 0 ? 0 : Math.min(t, end);
  scrub.max = String(Math.max(end, 0.001));
  if (document.activeElement !== scrub) scrub.value = String(now);
  playButton.firstChild!.textContent = playing ? 'Pause' : 'Play';
  playButton.classList.toggle('on', playing);
  timeText.textContent = current?.clips.length ? `${now.toFixed(2)} / ${end.toFixed(2)} s${turn && !solo ? ` · ${turn}` : ''}${hold === Infinity ? ' · open' : hold === 0 ? ' · at rest' : ''}` : '';
  // The camera moved by hand: into the URL, now and then.
  const moved = savedPose.alpha !== orbit.alpha || savedPose.beta !== orbit.beta || savedPose.radius !== orbit.radius;
  if (moved && performance.now() - savedAt > 500) {
    Object.assign(savedPose, { alpha: orbit.alpha, beta: orbit.beta, radius: orbit.radius });
    saveUrl();
  }
});

renderList();
const first = params.get('key');
const target = params.get('target')?.split(',').map(Number);
const viewGiven = num('radius') !== null;
await select(first && keys.includes(first) ? first : (listed[0] ?? keys[0]), { keepView: viewGiven });
if (viewGiven && target?.length === 3 && target.every(Number.isFinite)) [orbit.target.x, orbit.target.y, orbit.target.z] = target;
else if (viewGiven) frame(true);
if (params.has('t')) set(num('t') ?? 0);
// (Two frames drawn before saying so: the shadow's and the picture's.)
requestAnimationFrame(() =>
  requestAnimationFrame(() => Object.assign(window, { __app: app, __scene: app.scene, __inspect: { select, set, play, solo: setSolo, get current() { return current; } }, __ready: true })),
);
