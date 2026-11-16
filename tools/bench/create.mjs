#!/usr/bin/env bun
/**
 * Pictures of a new game's first three steps as a player gets there: the main menu, "New game", step 1's
 * land made and built, then shot at each size (`--sizes`) on each renderer (`--renderers`): the land's
 * view, the side panel, the way on and the figures. At the narrowest size, also the panel scrolled to
 * its end. Then, at the first size, an arrow key in the place grid (down a row: another place) and the
 * making panel while that land is made. Then "Lay out the town →": step 2's town laid on that land
 * (its making panel shot meanwhile), shot at each size the same way. Then "Choose a household →":
 * step 3's households offered as cards, shot at each size once the portraits in view are drawn (the
 * narrowest with its cards scrolled to their end), then at the first size with a card chosen (the
 * camera on its area) and after "Show others".
 *
 * The stream pill is shown throughout (as on a first visit, while models still come).
 *
 * With `--shots …,edit`, between steps 2 and 3, a scripted edit session in step 2's town editor
 * (`web/src/ui/townedit/`), each step shot at the edit sizes (`--edit-sizes`): a lot picked, removed,
 * the removal undone, a home put down, a lot dragged elsewhere, a park made another public place, a
 * street renamed, then a street removed with its lots. It measures the frames while the Lot tool's
 * ghost follows the pointer, and each edit's way to the town built again (the pause, the settle, the
 * town shown: the editor's own timings), and says them (`create: edit …`). Then the Street tool: a
 * street drawn by clicks across open land from one road to another (the frames and the editor's own
 * work per pointer move while its ghost follows the pointer, and `drawStreet`'s own time, the one
 * the tool asks once the pointer rests), one over the water on a bridge if the world has a crossing
 * on screen, a refused tight bend, a stretch too steep if the land has one on screen, and a street
 * being drawn taken back a point (Backspace) and dropped (Esc).
 *
 * Then build-up (`--edit …,buildup`): a main-town street built up from its card (homes and the town's
 * buildings), a village's street built up with both, its buildings cleared, a civic place the village
 * lacks put down with the Lot tool, and a street drawn with the Street tool's "Along it" left as it
 * is (both in the main town); each checked by what the plan gained or lost, and shot.
 * Then villages (`--edit …,villages`): a village put down with the Village tool, dragged elsewhere,
 * removed from its card and the removal undone (`villageSession`).
 *
 * Besides the pictures it checks what it can by measure, and says so (`create: …`): the side panel,
 * the way on and the figures clear of the stream pill (if it is up) and of every lot tag shown; the arrow
 * key chose the place a row down and moved the focus there; step 2 says it is step 2; step 3 says it is
 * step 3, offers 10 to 30 households (all of them in a smaller town), draws every portrait (how long
 * that took, and how many), and "Show others" offers others.
 *
 *   bun tools/bench/create.mjs [options]
 *     --sizes 2380x1546,1920x1080,740x1000   window sizes (CSS px)
 *     --renderers webgpu,webgl               the page's renderer (`?renderer=webgl`)
 *     --seed 7                               the page's Math.random seed (the same world in every run)
 *     --world country --options town=city    step 1's place and options (the URL's `?world=`, `?options=`)
 *     --code river-mhs-k2x9q                 the world's code instead (the URL's `?code=`; a bare number: the land's seed)
 *     --shots 1,2,3                          the steps shot (the others only gone through); `edit`: step 2's edit session
 *     --stay                                 step 1's place kept as given (no arrow key down the place grid: a place's
 *                                            own options would replace `--options`; the village edits want a large land)
 *     --edit-sizes 1920x1080,740x1000        the edit session's shots' sizes (it runs at the first)
 *     --edit lots,streets,buildup            the edit session's parts: the lot and street edits above, the Street tool, build-up
 *     --keep-place                           no arrow key in step 1's place grid: the place asked for (`--code`) kept
 *     --headed                               in a window (default headless: the real GPU either way)
 *     --port 5184 | --url <u>                a dev server on the port (started if none answers), or one already running
 *     --out <dir>                            default bench-results/create-<time>/
 *
 * Needs Playwright like render.mjs (`PLAYWRIGHT=<path to playwright/index.mjs>`).
 */
import { spawn } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { quiet } from './quiet.mjs';
import { worldFlags } from './worldFlags.mjs';

const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const WEB = join(ROOT, 'web/');
const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};
const has = (name) => args.includes(`--${name}`);

const sizes = flag('sizes', '2380x1546,1920x1080,740x1000')
  .split(',')
  .map((s) => s.split('x').map(Number))
  .map(([width, height]) => ({ width, height }));
const renderers = flag('renderers', 'webgpu,webgl').split(',');
const seed = Number(flag('seed', '7'));
// (None given: step 1 as a player gets it, the URL without them.)
const made = worldFlags(flag, null);
const code = flag('code', null);
const shots = new Set(flag('shots', '1,2,3').split(','));
const editSizes = flag('edit-sizes', '1920x1080,740x1000')
  .split(',')
  .map((s) => s.split('x').map(Number))
  .map(([width, height]) => ({ width, height }));
const editParts = new Set(flag('edit', 'lots,streets,buildup,villages').split(','));
const worldQuery = [made.world ? `world=${made.world}` : '', made.query ? `options=${made.query}` : '', code ? `code=${encodeURIComponent(code)}` : ''];
const port = Number(flag('port', '5184'));
const stamp = new Date().toISOString().replace(/\.\d+Z$/, '').replace(/[:T]/g, '-');
const out = flag('out', join(ROOT, 'bench-results', `create-${stamp}`));
mkdirSync(out, { recursive: true });

async function playwright() {
  if (process.env.PLAYWRIGHT) return import(process.env.PLAYWRIGHT);
  try {
    return await import(createRequire(WEB + 'package.json').resolve('playwright'));
  } catch {
    throw new Error('Playwright not found: install it, or set PLAYWRIGHT=<path to playwright/index.mjs>');
  }
}

async function running(url) {
  try {
    return (await fetch(url, { signal: AbortSignal.timeout(1500) })).ok;
  } catch {
    return false;
  }
}

let server = null;
const base = flag('url', null) ?? `http://localhost:${port}`;
if (!flag('url', null) && !(await running(base))) {
  server = spawn(`${WEB}node_modules/.bin/vite`, ['--port', String(port), '--strictPort'], { cwd: WEB, stdio: ['ignore', 'pipe', 'inherit'] });
  process.on('exit', () => server.kill());
  await new Promise((resolve, reject) => {
    server.stdout.on('data', (d) => String(d).includes('Local') && resolve());
    server.on('exit', (code) => reject(new Error(`vite exited (${code})`)));
  });
}

/** The page's Math.random, seeded (mulberry32), before any of its scripts. */
function seeded(n) {
  let a = (n ^ 0x9e3779b9) >>> 0;
  Math.random = () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** A step's world made and built: its way on (`label`, a pattern) can be pressed. */
const enabled = (page, label) => page.waitForFunction((src) => [...document.querySelectorAll('button')].some((b) => new RegExp(src).test(b.textContent ?? '') && !b.disabled), label.source, { timeout: 120_000, polling: 100 });
/** Step 1's land made and built: "Lay out the town" can be pressed. */
const ready = (page) => enabled(page, /Lay out the town/);
/** Step 2's town laid and built: "Choose a household" can be pressed. */
const townReady = (page) => enabled(page, /Choose a household/);

/**
 * What overlaps on screen now: the UI over the town (the side panel, or its two parts when narrow; the
 * way on; the stats) against the stream pill and each lot tag shown. Returns one line per overlap.
 */
function overlaps() {
  const rect = (el) => {
    const r = el.getBoundingClientRect();
    return r.width && r.height ? { l: r.left, t: r.top, r: r.right, b: r.bottom } : null;
  };
  const hit = (a, b) => a.l < b.r && b.l < a.r && a.t < b.b && b.t < a.b;
  const panels = [...document.querySelectorAll('.side, .side .head, .side .panel, .view .actions .btn, .view .stats span, .view .offer')].map((el) => ({ name: el.className.split(' ')[0] || el.tagName, r: rect(el) })).filter((p) => p.r);
  const found = [];
  const stream = document.querySelector('.stream');
  const pill = stream && rect(stream);
  if (pill) for (const p of panels) if (hit(p.r, pill)) found.push(`the stream pill over .${p.name}`);
  for (const tag of document.querySelectorAll('.tags .pin')) {
    if (getComputedStyle(tag).visibility === 'hidden') continue;
    const t = tag.firstElementChild && rect(tag.firstElementChild);
    if (!t) continue;
    for (const p of panels) if (hit(p.r, t)) found.push(`the tag "${tag.textContent?.trim().slice(0, 30)}" over .${p.name}`);
  }
  return found;
}

/**
 * In the page: helpers for the edit session (`window.__bench`), over the town editor (`window.__townEditor`, dev only)
 * and the step's town: where a thing of the plan is on screen (a point the stage itself is under, no panel over it),
 * frames counted while the pointer moves.
 */
function benchHelpers() {
  return (async () => {
    const [{ app }, { menuScene }, lots, streets, town, pickMod, live, buildup, villages] = await Promise.all([
      import('/src/ui/app.svelte.ts'),
      import('/src/game/menuScene.ts'),
      import('/src/game/edit/lots.ts'),
      import('/src/game/edit/streets.ts'),
      import('/src/game/town.ts'),
      import('/src/ui/townedit/pick.ts'),
      import('/src/ui/townedit/streetLive.ts'),
      import('/src/game/edit/buildup.ts'),
      import('/src/game/edit/villages.ts'),
    ]);
    const t = await town.loadTemplates();
    const ed = () => window.__townEditor;
    const plan = () => ed().draft;
    /** A ground point (m) on screen (CSS px), at the land's height there; null if no stage is under it. */
    const screen = (x, z) => {
      const out = { x: 0, y: 0 };
      let y = 0;
      for (let k = 0; k < 3; k++) {
        if (!menuScene.project(x, y, z, out)) return null;
        const g = menuScene.landAt(out.x, out.y);
        if (!g) break;
        y = g.y;
      }
      const el = document.elementFromPoint(out.x, out.y);
      return el?.classList.contains('stage') ? { x: out.x, y: out.y } : null;
    };
    const centre = (s) => {
      const r = town.plotRect(t, s);
      return [r.x + r.w / 2, r.z + r.d / 2];
    };
    const pointAt = (pts, s) => {
      let at = 0;
      for (let i = 0; i + 1 < pts.length; i++) {
        const l = Math.hypot(pts[i + 1][0] - pts[i][0], pts[i + 1][1] - pts[i][1]);
        if (at + l >= s) {
          const f = (s - at) / (l || 1);
          return [pts[i][0] + (pts[i + 1][0] - pts[i][0]) * f, pts[i][1] + (pts[i + 1][1] - pts[i][1]) * f];
        }
        at += l;
      }
      return pts[pts.length - 1];
    };
    const length = (pts) => pts.reduce((a, p, i) => (i ? a + Math.hypot(p[0] - pts[i - 1][0], p[1] - pts[i - 1][1]) : 0), 0);
    let frames = null;
    window.__bench = {
      /** A lot of the main town of kind `kind` ('home', 'park') on screen: its slot and point. */
      lot(kind, skip = 0) {
        const d = plan();
        const ok = (s) => (kind === 'home' ? s.kind === 'house' && s.lot !== 'farm' && s.lot !== 'terrace' : s.kind === kind) && (s.area ?? 0) === 0;
        let n = 0;
        for (const s of d.slots) {
          if (!ok(s)) continue;
          const p = screen(...centre(s));
          if (p && n++ >= skip) return { slot: s.index, name: s.name, ...p };
        }
        return null;
      },
      /** A road of the main town on screen (its middle), and what removing it takes: one taking homes (`withLots`) or none. */
      road(withLots) {
        const d = plan();
        for (const [i, r] of d.roads.entries()) {
          if (r.kind === 'trail' || r.bridge || (r.area ?? -1) !== 0) continue;
          const goes = streets.whatGoes(d, t, i);
          if (withLots !== undefined && ('refused' in goes || (goes.lots.length > 0) !== withLots)) continue;
          const L = length(r.points);
          for (const f of [0.5, 0.35, 0.65, 0.2, 0.8]) {
            const p = screen(...pointAt(r.points, L * f));
            if (!p) continue;
            const picked = pickMod.pick(d, t, ...pointAt(r.points, L * f));
            if (picked?.kind === 'road' && picked.road === i) return { road: i, name: r.name, lots: 'refused' in goes ? 0 : goes.lots.length, ...p };
          }
        }
        return null;
      },
      /** Where a lot of `what` (a `LotChoice`, or `{ slot }` to move one) may go along the roads (the main town's first, then the rest): points on screen of the lots' middles, in turn. */
      spots(what, most = 12) {
        const d = plan();
        const out = [];
        const order = d.roads.map((r, i) => i).sort((a, b) => ((d.roads[a].area ?? 9) === 0 ? 0 : 1) - ((d.roads[b].area ?? 9) === 0 ? 0 : 1));
        for (const i of order) {
          const r = d.roads[i];
          if (r.kind === 'trail' || r.bridge) continue;
          const L = length(r.points);
          for (let s = 8; s < L - 8 && out.length < most; s += 7) {
            for (const side of [1, -1]) {
              const fit = lots.canPlace(d, t, { road: i, s, side }, what, 1);
              if ('refused' in fit) continue;
              const p = screen(fit.x + fit.w / 2, fit.z + fit.d / 2);
              if (p) out.push(p);
            }
          }
        }
        return out;
      },
      /**
       * A street for the Street tool to draw, off a road, square to it, on screen: `join` (across open land to another
       * road, nothing in its way, no bridge; `join-any`: clearing what is in its way), `bridge` (over the water on a bridge), `tight` (a bend tighter than 9 m)
       * or `steep` (a stretch too steep, nothing else wrong); `area`: a street of that area only. Its ground points and their screen points (the last on
       * the road it reaches), and `drawStreet`'s own time on it.
       */
      stroke(what, skip = 0, area) {
        const d = plan();
        /** Steep ones found, the reddest first (the likelier the settle refuses it). */
        const found = [];
        const red = (v) => v.stretches.filter((x) => x.tone === 'bad').reduce((a, x) => a + x.to - x.from, 0);
        const order = d.roads.map((r, i) => i).sort((a, b) => ((d.roads[a].area ?? 9) === 0 ? 0 : 1) - ((d.roads[b].area ?? 9) === 0 ? 0 : 1));
        const add = (p, v, k) => [p[0] + v[0] * k, p[1] + v[1] * k];
        /** The live street through `pts` if it is what is wanted (`drawStreet`'s own time on it, `ms`, for a street drawn), else null. */
        const judge = (pts) => {
          const v = live.liveStreet(d, t, pts, 'street', true);
          if (area !== undefined && v.area !== area) return null;
          if (what === 'tight') return v.why === 'it bends tighter than 9 m' ? v : null;
          if (what === 'steep') {
            // (One that may be drawn, a street to another road or a short dead end, for the settle to refuse.)
            if (!v.steep || v.why || v.bridges.length || (v.open && !v.reached)) return null;
            const e = live.verdictOf(d, t, ed().content, pts, 'street', true, 1);
            return 'draft' in e ? v : null;
          }
          if (v.why || !v.reached || v.end?.kind !== 'join' || (what === 'bridge') !== v.bridges.length > 0) return null;
          // (`join-any`: a street to another road that may clear what is in its way, where none runs over open land.)
          const t0 = performance.now();
          const e = live.verdictOf(d, t, ed().content, pts, 'street', true, 1);
          const ms = performance.now() - t0;
          return 'draft' in e && (what !== 'join' || e.cleared.lots.length + e.cleared.backdrop.length === 0) ? { ...v, ms } : null;
        };
        for (const i of order) {
          const r = d.roads[i];
          if (r.kind === 'trail' || r.bridge) continue;
          const L = length(r.points);
          for (let s = 20; s < L - 20; s += 9) {
            const p = pointAt(r.points, s);
            const q = pointAt(r.points, s + 1);
            const l = Math.hypot(q[0] - p[0], q[1] - p[1]) || 1;
            const tq = [(q[0] - p[0]) / l, (q[1] - p[1]) / l];
            for (const side of [1, -1]) {
              const n = [-tq[1] * side, tq[0] * side];
              const from = add(p, n, r.width / 2 + 3);
              const lens = what === 'steep' ? [30, 40, 60, 90, 120, 160, 200] : [60, 90, 120, 160, 200, 260, 320];
              const tries = what === 'tight' ? [[from, add(from, n, 34), add(add(from, n, 34), tq, 9), add(from, n, 50)]] : lens.map((len) => [from, add(from, n, len)]);
              for (const tried of tries) {
                const v = judge(tried);
                if (!v) continue;
                // (A line that reaches a road ends on it: the last click where the line drawn meets that road. The clicks
                // are judged again from the ground under their screen points, as the tool will have them.)
                let clicks = tried;
                if (what === 'join' || what === 'join-any' || what === 'bridge' || (what === 'steep' && v.reached)) {
                  const [a, b] = tried;
                  const e = v.line[v.line.length - 1];
                  const u = [b[0] - a[0], b[1] - a[1]];
                  const f = ((e[0] - a[0]) * u[0] + (e[1] - a[1]) * u[1]) / (u[0] * u[0] + u[1] * u[1]);
                  const end = add(a, u, f);
                  clicks = [a, add(a, u, f / 2), end];
                }
                const screens = clicks.map((g) => screen(g[0], g[1]));
                if (screens.some((x) => !x)) continue;
                const back = screens.map((p) => {
                  const g = ed().ground(p.x, p.y);
                  return [g.x, g.z];
                });
                const w = judge(back);
                if (!w) continue;
                if (what === 'steep') {
                  found.push({ red: red(w), road: r.name, ground: back, screens, slope: w.slope, reached: w.reached });
                  if (found.length >= 40) break;
                  continue;
                }
                return { road: r.name, ground: back, screens, verdictMs: w.ms ?? null, bridges: w.bridges, why: w.why, slope: w.slope, reached: w.reached };
              }
            }
          }
        }
        found.sort((a, b) => b.red - a.red);
        return found[skip] ?? null;
      },
      /**
       * A road to build up on screen (`where`: `town`, the main town's; `village`, a village's): of the first few that
       * may be built up, the one whose build-up (`mode`), once its buildings are cleared, adds the most buildings; its
       * point, and what it would add.
       */
      buildable(where, mode) {
        const d = plan();
        let best = null;
        let tried = 0;
        for (const [i, r] of d.roads.entries()) {
          const a = buildup.areaOfRoad(d, i);
          if ((where === 'town' ? a !== 0 : a < 1) || buildup.notBuilt(d, i)) continue;
          const L = length(r.points);
          let at = null;
          for (const f of [0.5, 0.35, 0.65, 0.2, 0.8]) {
            const g = pointAt(r.points, L * f);
            const p = screen(...g);
            const picked = p && pickMod.pick(d, t, ...g);
            if (picked?.kind === 'road' && picked.road === i) {
              at = p;
              break;
            }
          }
          if (!at) continue;
          const c = buildup.clearBuildings(d, i);
          const bare = 'draft' in c ? c.draft : d;
          const e = buildup.buildUp(bare, t, ed().content, { road: i, mode, seed: 1 });
          if ('refused' in e) continue;
          const adds = { buildings: e.draft.open.backdrop.length - bare.open.backdrop.length, homes: e.draft.slots.length - bare.slots.length };
          if (!best || adds.buildings > best.buildings) best = { road: i, name: r.name, area: a, ...adds, ...at };
          if (++tried >= 14) break;
        }
        return best;
      },
      /** Points on screen along a village's streets where the Lot tool's civic place (the first the village lacks) may go: each with its village. */
      civicSpots(most = 30) {
        const d = plan();
        const out = [];
        for (const [i, r] of d.roads.entries()) {
          const a = buildup.areaOfRoad(d, i);
          if (a < 1 || buildup.notBuilt(d, i)) continue;
          const [first] = buildup.civicOffered(d, a);
          if (!first) continue;
          const L = length(r.points);
          for (let s = 10; s < L - 10 && out.length < most; s += 3) {
            for (const side of [1, -1]) {
              const e = buildup.addCivic(d, t, ed().content, { civic: first.civic, road: i, s, side, seed: 1 });
              if ('refused' in e) continue;
              const b = e.draft.open.backdrop[e.draft.open.backdrop.length - 1];
              const c = b.poly.reduce((m, p) => [m[0] + p[0] / b.poly.length, m[1] + p[1] / b.poly.length], [0, 0]);
              // (The pointer by the road, a little out from it on that side: where a home's ghost is aimed too.)
              const g = pointAt(r.points, s);
              const p = screen((g[0] + c[0]) / 2, (g[1] + c[1]) / 2);
              if (p) out.push({ ...p, area: a, village: d.open.areas[a].name, civic: first.label });
            }
          }
        }
        return out;
      },
      /** Where a village may be put down (`canPlaceVillage`, a 20 m grid), on screen, the nearest the middle of the view first; and how long each test took (ms). */
      villageSpots(swap = false) {
        const d = plan();
        const out = [];
        const t0 = performance.now();
        let [n, fits] = [0, 0];
        for (let z = 60; z < d.depth - 60; z += 20) {
          for (let x = 60; x < d.width - 60; x += 20) {
            n++;
            if ('refused' in villages.canPlaceVillage(d, t, { x, z, swap })) continue;
            fits++;
            const p = screen(x, z);
            if (p) out.push({ ...p, gx: x, gz: z });
          }
        }
        const each = (performance.now() - t0) / n;
        const [cx, cy] = [innerWidth / 2, innerHeight / 2];
        out.sort((a, b) => Math.hypot(a.x - cx, a.y - cy) - Math.hypot(b.x - cx, b.y - cy));
        return { spots: out.slice(0, 16), each, fits, world: `${d.world?.kind} ${d.width} m, ${d.open?.areas.length} areas` };
      },
      /** Village `area` on screen: a point in its box the Village tool picks it at (no lot or road there), and the ground there. */
      village(area) {
        const d = plan();
        const a = d.open?.areas[area];
        if (!a) return null;
        for (const fz of [0.5, 0.3, 0.7, 0.15, 0.85]) {
          for (const fx of [0.5, 0.3, 0.7, 0.15, 0.85]) {
            const [gx, gz] = [a.x + a.w * fx, a.z + a.d * fz];
            const picked = pickMod.pick(d, t, gx, gz);
            if (picked?.kind !== 'village' || picked.area !== area) continue;
            const p = screen(gx, gz);
            if (p) return { ...p, gx, gz, name: a.name };
          }
        }
        return null;
      },
      /** Where village `area`, taken at ground point (`gx`, `gz`), may be dragged to (`canMoveVillage`): points on screen, the nearest first. */
      villageMoves(area, gx, gz) {
        const d = plan();
        const out = [];
        for (let k = 6; k <= 24 && out.length < 8; k += 2) {
          for (const [ux, uz] of [[1, 0], [-1, 0], [0, 1], [0, -1], [1, 1], [-1, 1], [1, -1], [-1, -1]]) {
            const [dx, dz] = [ux * k * 10, uz * k * 10];
            if ('refused' in villages.canMoveVillage(d, t, area, dx, dz)) continue;
            const p = screen(gx + dx, gz + dz);
            if (p) out.push({ ...p, dx, dz });
          }
        }
        return out;
      },
      /** How many backdrop buildings and places the plan has. */
      built() {
        const d = plan();
        return { buildings: d.open?.backdrop?.length ?? 0, places: d.open?.places?.length ?? 0 };
      },
      /** The page's frames from now (their gaps, ms) until `stop`: the 50th, 95th percentiles and the longest. */
      startFrames() {
        frames = [];
        let last = performance.now();
        const tick = (now) => {
          if (!frames) return;
          frames.push(now - last);
          last = now;
          requestAnimationFrame(tick);
        };
        requestAnimationFrame(tick);
      },
      stopFrames() {
        const f = (frames ?? []).slice(1).sort((a, b) => a - b);
        frames = null;
        const q = (k) => f[Math.min(f.length - 1, Math.floor(f.length * k))] ?? 0;
        return { frames: f.length, p50: q(0.5), p95: q(0.95), max: f[f.length - 1] ?? 0 };
      },
      /** The editor: what it says, whether it is settling, the settles' timings, the Problems list. */
      state() {
        const e = ed();
        const areas = e.draft.open?.areas ?? [];
        const shown = app.world?.draft?.open?.areas ?? [];
        return { hint: e.hintText, note: e.note, settling: e.settling, timings: [...e.timings], blocking: e.issues.blocking.map((i) => i.text), warnings: e.issues.warnings.length, edits: e.edits, selected: e.selected, slots: e.draft.slots.length, roads: e.draft.roads.length, areas: areas.map((a) => ({ name: a.name, x: a.x, z: a.z })), shownAreas: shown.length, edited: app.world?.edited };
      },
    };
    return true;
  })();
}

/**
 * Step 2's edit session (see the top of this file): each step shot at every edit size, each edit's way to the town
 * built again measured, the frames while the Lot tool's ghost follows the pointer.
 */
async function editSession(page, renderer, shootAs, setStep) {
  setStep('edit');
  const [first] = editSizes;
  await page.setViewportSize(first);
  await page.waitForTimeout(2500);
  await page.evaluate(benchHelpers);
  const bench = (fn, arg) => page.evaluate(fn, arg);
  const state = () => bench(() => window.__bench.state());
  let n = 0;
  /** Each edit size shot (`label`), back at the first after. */
  const shots = async (label) => {
    n++;
    const name = `${String(n).padStart(2, '0')}-${label}`;
    for (const size of editSizes) {
      if (size !== first) {
        await page.setViewportSize(size);
        await page.waitForTimeout(2200);
      }
      await shootAs(`${size.width}x${size.height}-${name}`);
    }
    if (editSizes.length > 1) {
      await page.setViewportSize(first);
      await page.waitForTimeout(2200);
    }
  };
  /** The town built again with the edits (the editor no longer settling); its settle's timing said. */
  const built = async (what) => {
    const t0 = Date.now();
    await page.waitForFunction(() => !window.__townEditor.settling, null, { timeout: 60_000, polling: 50 }).catch(() => fault(`${renderer} edit: ${what}: the town not built again within 60 s`));
    const s = await state();
    const last = s.timings[s.timings.length - 1];
    if (last) console.log(`create: ${renderer} edit ${what}: built again ${Math.round(last.fromEdit)} ms after the edit (the pause, then settle ${Math.round(last.settle)} ms, town shown ${Math.round(last.show)} ms; waited ${Date.now() - t0} ms)`);
    return s;
  };
  const tool = async (id) => {
    const on = await page.evaluate((id) => document.querySelector(`[data-tool="${id}"]`)?.getAttribute('aria-checked') === 'true', id);
    if (!on) await page.locator(`[data-tool="${id}"]`).click();
    await page.waitForTimeout(150);
  };
  const click = async (p) => {
    await page.mouse.move(p.x, p.y, { steps: 4 });
    await page.waitForTimeout(80);
    await page.mouse.click(p.x, p.y);
    await page.waitForTimeout(250);
  };

  const lotEdits = async () => {
    // 1. A home picked: its card.
    await tool('select');
    const home = await bench(() => window.__bench.lot('home', 2));
    if (!home) return fault(`${renderer} edit: no home on screen to pick`);
    await click(home);
    if ((await state()).selected?.kind !== 'lot') fault(`${renderer} edit: a click on ${home.name} picked ${JSON.stringify((await state()).selected)}`);
    await shots('select');

    // 2. Removed from its card: its ghost greyed at once, the town built again after the pause.
    const before = (await state()).slots;
    await page.locator('.card button', { hasText: /^\s*Remove\s*$/ }).click();
    await page.waitForTimeout(200);
    if ((await state()).slots !== before - 1) fault(`${renderer} edit: removing ${home.name} left ${(await state()).slots} lots (of ${before})`);
    await shots('remove');
    await built('remove a lot');
    await shots('removed-built');

    // 3. Undone.
    await page.keyboard.press(process.platform === 'darwin' ? 'Meta+z' : 'Control+z');
    await page.waitForTimeout(250);
    if ((await state()).slots !== before) fault(`${renderer} edit: undo left ${(await state()).slots} lots (of ${before})`);
    await built('undo');
    await shots('undo');

    // 4. A home put down with the Lot tool, the frames measured while its ghost follows the pointer.
    await tool('lot');
    await page.locator('.lot-card [role="radio"]', { hasText: 'Home' }).first().click();
    const spots = await bench(() => window.__bench.spots({ kind: 'house', size: 'medium' }));
    if (!spots.length) fault(`${renderer} edit: nowhere on screen for a home`);
    if (spots.length > 1) {
      /** The frames while the pointer goes along the spots (the same way each time). */
      const along = async () => {
        await page.mouse.move(spots[0].x, spots[0].y);
        await bench(() => window.__bench.startFrames());
        for (const p of spots.slice(1)) await page.mouse.move(p.x, p.y, { steps: 20 });
        return bench(() => window.__bench.stopFrames());
      };
      const line = (f) => `${f.frames} frames, p50 ${f.p50.toFixed(1)} ms, p95 ${f.p95.toFixed(1)} ms, longest ${f.max.toFixed(1)} ms`;
      // (The same way with no tool in hand first: what the page does anyway, the lots lit as the pointer goes.)
      await page.keyboard.press('Escape');
      const bare = await along();
      await tool('lot');
      const f = await along();
      // The editor's own work per pointer move (the lot fitted, its ghost made and handed to the renderer), apart from the frame.
      const each = await bench((pts) => {
        const tool = window.__townEditor.stageTool;
        const t0 = performance.now();
        for (let k = 0; k < 5; k++) for (const p of pts) tool.hover(p.x + k, p.y);
        return (performance.now() - t0) / (pts.length * 5);
      }, spots);
      console.log(`create: ${renderer} edit: frames while the lot's ghost follows the pointer: ${line(f)} (with no tool in hand: ${line(bare)}); the editor's own work ${each.toFixed(2)} ms a pointer move`);
    }
    let placed = false;
    for (const p of spots) {
      await page.mouse.move(p.x, p.y, { steps: 3 });
      await page.waitForTimeout(60);
      if (!/^Click to put it down/.test((await state()).hint ?? '')) continue;
      await shots('add-ghost');
      const had = (await state()).slots;
      await page.mouse.click(p.x, p.y);
      await page.waitForTimeout(200);
      placed = (await state()).slots === had + 1;
      break;
    }
    if (!placed) fault(`${renderer} edit: no home put down (${(await state()).hint})`);
    await shots('add');
    await built('add a home');

    // 5. A lot dragged elsewhere with the Lot tool.
    const mover = await bench(() => window.__bench.lot('home', 6));
    if (mover) {
      const targets = await bench((slot) => window.__bench.spots({ slot }), mover.slot);
      await page.mouse.move(mover.x, mover.y, { steps: 3 });
      await page.mouse.down();
      let moved = false;
      for (const p of targets) {
        await page.mouse.move(p.x, p.y, { steps: 8 });
        await page.waitForTimeout(60);
        if (/^Let go to move/.test((await state()).hint ?? '')) {
          await shots('move-drag');
          await page.mouse.up();
          moved = true;
          break;
        }
      }
      if (!moved) {
        await page.mouse.up();
        fault(`${renderer} edit: ${mover.name} found nowhere to go (${(await state()).hint})`);
      }
      await page.waitForTimeout(200);
      await shots('move');
      await built('move a lot');
    } else fault(`${renderer} edit: no home on screen to move`);

    // 6. A park made another public place, from its card.
    await tool('select');
    const park = await bench(() => window.__bench.lot('park'));
    if (park) {
      await click(park);
      const chip = page.locator('.card .chip[aria-checked="false"]').first();
      if (await chip.count()) {
        await chip.click();
        await page.waitForTimeout(250);
        await shots('swap-park');
        await built('swap a park');
      } else fault(`${renderer} edit: ${park.name}'s card offers no other place`);
    } else fault(`${renderer} edit: no park on screen`);

    // 7. A street renamed from its card.
    const street = await bench(() => window.__bench.road());
    if (street) {
      await click(street);
      const field = page.locator('.card input.rename');
      await field.fill('Bench Street');
      await field.press('Enter');
      await page.waitForTimeout(250);
      const named = await page.evaluate(() => window.__townEditor.draft.roads.some((r) => r.name === 'Bench Street'));
      if (!named) fault(`${renderer} edit: ${street.name} not renamed (${(await state()).note})`);
      await shots('rename');
    } else fault(`${renderer} edit: no street on screen`);

    // 8. A street removed with its lots, from its card (what goes listed first).
    const doomed = await bench(() => window.__bench.road(true));
    if (doomed) {
      await click(doomed);
      await shots('street-card');
      const had = (await state()).slots;
      await page.locator('.card button', { hasText: /^\s*Remove / }).click();
      await page.waitForTimeout(250);
      const s = await state();
      if (s.slots >= had) fault(`${renderer} edit: removing ${doomed.name} took no lots (${s.note})`);
      await shots('remove-street');
      await built('remove a street with its lots');
      await shots('remove-street-built');
    } else fault(`${renderer} edit: no street with lots that may go`);
  };

  /** The Street tool (see the top of this file). */
  const streetEdits = async () => {
    await tool('street');
    await page.locator('.street-card [role="radio"]', { hasText: 'Nothing' }).first().click();
    await page.waitForTimeout(100);
    const line = (f) => `${f.frames} frames, p50 ${f.p50.toFixed(1)} ms, p95 ${f.p95.toFixed(1)} ms, longest ${f.max.toFixed(1)} ms`;
    /** The pointer from `a` to `b` (screen points) in `steps`, the frames meanwhile. */
    const sweep = async (a, b, steps = 40) => {
      await page.mouse.move(a.x, a.y);
      await bench(() => window.__bench.startFrames());
      await page.mouse.move(b.x, b.y, { steps });
      return bench(() => window.__bench.stopFrames());
    };
    /** The editor's own work per pointer move along `a` to `b` (the street worked out, its ghost made and handed to the renderer). */
    const perMove = (a, b) =>
      bench(
        ([a, b]) => {
          const tool = window.__townEditor.stageTool;
          const n = 60;
          const times = [];
          for (let k = 0; k <= n; k++) {
            const t0 = performance.now();
            tool.hover(a.x + ((b.x - a.x) * k) / n, a.y + ((b.y - a.y) * k) / n);
            times.push(performance.now() - t0);
          }
          times.sort((x, y) => x - y);
          return { median: times[n >> 1], p95: times[Math.floor(n * 0.95)], max: times[n] };
        },
        [a, b],
      );
    const clickAt = async (p) => {
      await page.mouse.move(p.x, p.y, { steps: 6 });
      await page.waitForTimeout(60);
      await page.mouse.down();
      await page.mouse.up();
      await page.waitForTimeout(120);
    };
    /** The street through `s` (a `stroke`) drawn by clicks: the frames and the per-move work while the pointer goes to its last point, its ghost shot before the last click. */
    const draw = async (s, label) => {
      const had = (await state()).roads;
      await clickAt(s.screens[0]);
      for (const p of s.screens.slice(1, -1)) await clickAt(p);
      const from = s.screens[s.screens.length - 2];
      const to = s.screens[s.screens.length - 1];
      const f = await sweep(from, to);
      const work = await perMove(from, to);
      await page.mouse.move(to.x, to.y, { steps: 2 });
      await page.waitForTimeout(400);
      const ghost = await state();
      console.log(`create: ${renderer} edit ${label}: off ${s.road}${s.bridges.length ? `, bridges ${s.bridges.join(', ')} m` : ''}; frames while its ghost follows the pointer: ${line(f)}; the editor's own work a pointer move ${work.median.toFixed(2)} ms median, ${work.p95.toFixed(2)} ms p95, ${work.max.toFixed(2)} ms longest; drawStreet once the pointer rests ${s.verdictMs?.toFixed(1)} ms; says "${ghost.hint}"`);
      await shots(`${label}-ghost`);
      await page.mouse.down();
      await page.mouse.up();
      await page.waitForTimeout(250);
      const after = await state();
      if (after.roads <= had) fault(`${renderer} edit ${label}: not drawn (${after.note ?? after.hint})`);
      else if (after.selected?.kind !== 'road') fault(`${renderer} edit ${label}: drawn, but ${JSON.stringify(after.selected)} picked, not the street`);
      await shots(label);
      await built(label);
      await shots(`${label}-built`);
    };

    // 9. A street across open land from one road to another, by clicks.
    const across = (await bench(() => window.__bench.stroke('join'))) ?? (await bench(() => window.__bench.stroke('join-any')));
    if (across) await draw(across, 'street');
    else fault(`${renderer} edit: no street from one road to another on screen to draw`);

    // 10. Over the water on a bridge (if the world has a crossing on screen).
    const over = await bench(() => window.__bench.stroke('bridge'));
    if (over) await draw(over, 'street-bridge');
    else console.log(`create: ${renderer} edit: no crossing of the water on screen for a bridge`);

    // 11. A tight bend: red where it bends, why under the tools; Enter refuses it; Esc drops it.
    const tight = await bench(() => window.__bench.stroke('tight'));
    if (tight) {
      const had = (await state()).roads;
      for (const p of tight.screens.slice(0, -1)) await clickAt(p);
      const last = tight.screens[tight.screens.length - 1];
      await page.mouse.move(last.x, last.y, { steps: 6 });
      await page.waitForTimeout(300);
      const s = await state();
      if (!/tighter than 9 m/.test(s.hint ?? '')) fault(`${renderer} edit tight bend: says "${s.hint}"`);
      await shots('street-tight');
      await page.mouse.down();
      await page.mouse.up();
      await page.waitForTimeout(200);
      await page.keyboard.press('Enter');
      await page.waitForTimeout(200);
      const after = await state();
      if (after.roads !== had) fault(`${renderer} edit tight bend: drawn all the same`);
      console.log(`create: ${renderer} edit tight bend: says "${s.hint}"; Enter: "${after.note}"`);
      await page.keyboard.press('Escape');
      await page.waitForTimeout(150);
    } else fault(`${renderer} edit: no tight bend on screen to try`);

    // 12. A street up a slope too steep out in the country (if the land has one on screen): tinted red as it is drawn, a
    // forecast; drawn all the same, the settle refuses it with its reason and the town goes back to how it was.
    let steep = null;
    for (let k = 0; k < 3; k++) {
      steep = await bench((k) => window.__bench.stroke('steep', k), k);
      if (!steep) break;
      const had = (await state()).roads;
      for (const p of steep.screens.slice(0, -1)) await clickAt(p);
      const last = steep.screens[steep.screens.length - 1];
      await page.mouse.move(last.x, last.y, { steps: 10 });
      await page.waitForTimeout(400);
      const s = await state();
      if (!/too steep/.test(s.hint ?? '')) fault(`${renderer} edit too steep: says "${s.hint}"`);
      console.log(`create: ${renderer} edit too steep: off ${steep.road} (${Math.round(steep.red)} m of it red), says "${s.hint}"`);
      await shots(`street-steep-${k + 1}`);
      await page.mouse.down();
      await page.mouse.up();
      await page.waitForTimeout(150);
      if (!steep.reached) await page.keyboard.press('Enter');
      await page.waitForTimeout(150);
      if ((await state()).roads <= had) {
        fault(`${renderer} edit too steep: not drawn for the settle to judge (${(await state()).note})`);
        await page.keyboard.press('Escape');
        break;
      }
      // (The settle's refusal is a toast; the town goes back to the plan before the street.)
      const toasts = [];
      const watch = setInterval(async () => {
        for (const text of await page.evaluate(() => [...document.querySelectorAll('.toasts .toast')].map((el) => el.textContent?.trim() ?? '')).catch(() => [])) if (!toasts.includes(text)) toasts.push(text);
      }, 100);
      await built('a street too steep');
      await page.waitForTimeout(400);
      clearInterval(watch);
      const after = await state();
      const why = toasts.find((t) => /dug out|built up|steep|cut|fill/i.test(t));
      if (after.roads === had && why) {
        console.log(`create: ${renderer} edit too steep: drawn, then the settle refused it: "${why}"; ${after.roads} roads, as before`);
        await shots(`street-steep-${k + 1}-refused`);
        break;
      }
      // (The forecast is the settle's to judge: one it keeps is taken back, and the next tried.)
      console.log(`create: ${renderer} edit too steep: the settle kept it (${after.roads} roads of ${had}; toasts: ${toasts.join(' | ') || 'none'}): undone, the next tried`);
      await page.keyboard.press(process.platform === 'darwin' ? 'Meta+z' : 'Control+z');
      await built('undo a street the settle kept');
      if (k === 2) fault(`${renderer} edit too steep: the settle kept all three streets the tool tinted too steep`);
    }
    if (!steep) console.log(`create: ${renderer} edit: no (more) streets too steep out in the country on screen`);

    // 13. Backspace takes the last point back; Esc drops the street (and a second Esc puts the tool down).
    const again = (await bench(() => window.__bench.stroke('tight'))) ?? (await bench(() => window.__bench.stroke('join')));
    if (again) {
      for (const p of again.screens.slice(0, 3)) await clickAt(p);
      const end = again.screens[again.screens.length - 1];
      await page.mouse.move(end.x + 30, end.y + 30, { steps: 4 });
      await page.waitForTimeout(150);
      await shots('street-points');
      await page.keyboard.press('Backspace');
      await page.waitForTimeout(150);
      await shots('street-backspace');
      await page.keyboard.press('Escape');
      await page.mouse.move(end.x + 34, end.y + 30, { steps: 2 });
      await page.waitForTimeout(150);
      const s = await state();
      if (!/start/.test(s.hint ?? '')) fault(`${renderer} edit Esc: the street not dropped ("${s.hint}")`);
      const toolOn = await page.evaluate(() => document.querySelector('[data-tool="street"]')?.getAttribute('aria-checked') === 'true');
      if (!toolOn) fault(`${renderer} edit Esc: the tool put down with the street`);
      await shots('street-esc');
      await page.keyboard.press('Escape');
      await page.waitForTimeout(150);
      const off = await page.evaluate(() => document.querySelector('[data-tool="street"]')?.getAttribute('aria-checked') !== 'true');
      if (!off) fault(`${renderer} edit Esc: a second Esc did not put the tool down`);
    }
  };

  /** Build-up (see the top of this file). */
  const buildupEdits = async () => {
    const counts = () => bench(() => window.__bench.built());
    const card = (name) => page.locator('.card button', { hasText: name });
    /** A street of `where` built up from its card with `mode`'s choice ('Both', or as the card has it). */
    const buildFromCard = async (where, mode, label) => {
      await tool('select');
      const road = await bench(([w, m]) => window.__bench.buildable(w, m), [where, mode]);
      if (!road) return fault(`${renderer} edit ${label}: no ${where} street on screen to build up`), null;
      await click(road);
      if ((await state()).selected?.kind !== 'road') return fault(`${renderer} edit ${label}: a click on ${road.name} picked ${JSON.stringify((await state()).selected)}`), null;
      if (mode === 'both') await page.locator('.card [role="radio"]', { hasText: /^Both$/ }).click();
      await page.waitForTimeout(150);
      await shots(`${label}-card`);
      // (A generated street is built up already: its buildings cleared first, then built up again.)
      if (await card('Clear its buildings').isEnabled()) {
        const full = await counts();
        await card('Clear its buildings').click();
        await page.waitForTimeout(300);
        const now = await counts();
        if (now.buildings >= full.buildings) fault(`${renderer} edit ${label}: ${road.name}'s buildings not cleared (${(await state()).note})`);
        console.log(`create: ${renderer} edit ${label}: ${full.buildings - now.buildings} buildings and ${full.places - now.places} places cleared from ${road.name} first`);
        await shots(`${label}-cleared`);
      }
      const had = await counts();
      await card('Build up this street').click();
      await page.waitForTimeout(300);
      const now = await counts();
      const s = await state();
      if (now.buildings <= had.buildings) fault(`${renderer} edit ${label}: ${road.name} built up with no buildings (${s.note})`);
      console.log(`create: ${renderer} edit ${label}: ${road.name} built up: ${now.buildings - had.buildings} buildings, ${now.places - had.places} places, ${s.slots} lots; card says "${await page.locator('.card .option').last().locator('.hint').last().textContent()}"`);
      await shots(label);
      await built(label);
      await shots(`${label}-built`);
      return road;
    };

    // 14. A main-town street built up from its card (both, the card's own choice there).
    await buildFromCard('town', 'both', 'buildup-town');
    // 15. A village's street built up with both (the card's choice for a village is homes).
    const village = await buildFromCard('village', 'both', 'buildup-village');
    // 16. What it was built up with cleared from its card again.
    if (village) {
      const had = await counts();
      await card('Clear its buildings').click();
      await page.waitForTimeout(300);
      const now = await counts();
      if (now.buildings >= had.buildings) fault(`${renderer} edit clear: ${village.name}'s buildings not cleared (${(await state()).note})`);
      console.log(`create: ${renderer} edit clear: ${had.buildings - now.buildings} buildings and ${had.places - now.places} places cleared from ${village.name}`);
      await shots('buildup-clear');
      await built('clear a street');
      await shots('buildup-clear-built');
    }
    // 17. A civic place a village lacks, with the Lot tool.
    await page.keyboard.press('Escape');
    await tool('lot');
    await page.locator('.lot-card [role="radio"]', { hasText: /^Civic$/ }).click();
    await page.waitForTimeout(150);
    const spots = await bench(() => window.__bench.civicSpots());
    let put = false;
    for (const p of spots) {
      await page.mouse.move(p.x, p.y, { steps: 4 });
      await page.waitForTimeout(120);
      const s = await state();
      if (!/^Click to put/.test(s.hint ?? '')) continue;
      await shots('civic-ghost');
      const had = await counts();
      await page.mouse.click(p.x, p.y);
      await page.waitForTimeout(300);
      const now = await counts();
      if (now.buildings !== had.buildings + 1 || now.places !== had.places + 1) fault(`${renderer} edit civic: ${now.buildings - had.buildings} buildings and ${now.places - had.places} places added (${(await state()).note})`);
      const named = await page.evaluate(() => window.__townEditor.draft.open.places.at(-1)?.name);
      console.log(`create: ${renderer} edit civic: "${s.hint}" in ${p.village}: ${named}`);
      await page.mouse.move(p.x + 200, p.y + 120, { steps: 3 });
      await shots('civic');
      await built('a civic place');
      await shots('civic-built');
      put = true;
      break;
    }
    if (!put) fault(`${renderer} edit civic: no spot on screen where a village's civic place may go (${spots.length} tried)`);
    // 18. A street drawn with "Along it" by where it runs: both in the main town.
    await tool('street');
    const by = page.locator('.street-card button.link', { hasText: 'By where it runs' });
    for (const k of [0, 1]) if ((await by.count()) > k) await by.first().click();
    const across = (await bench(() => window.__bench.stroke('join', 0, 0))) ?? (await bench(() => window.__bench.stroke('join-any', 0, 0)));
    if (across) {
      const had = await counts();
      for (const p of across.screens) await click(p);
      const now = await counts();
      const s = await state();
      if (s.selected?.kind !== 'road') fault(`${renderer} edit street-both: not drawn (${s.note ?? s.hint})`);
      console.log(`create: ${renderer} edit street-both: off ${across.road}: ${now.buildings - had.buildings} buildings along it`);
      await shots('buildup-street-both');
      await built('a street drawn with both along it');
      await shots('buildup-street-both-built');
    } else fault(`${renderer} edit: no street on screen to draw with both along it`);
  };

  if (editParts.has('lots')) await lotEdits();
  if (editParts.has('streets')) await streetEdits();
  if (editParts.has('buildup')) await buildupEdits();
  if (editParts.has('villages')) await villageSession(page, renderer, { bench, state, shots, built, tool, click });
  const end = await state();
  console.log(`create: ${renderer} edit: ${end.edits} edits, ${end.slots} lots, ${end.roads} roads; problems: ${end.blocking.length} blocking${end.blocking.length ? ` (${end.blocking.join('; ')})` : ''}, ${end.warnings} warnings; edited: ${end.edited}`);
  await page.keyboard.press('Escape');
  await page.waitForTimeout(300);
}

/**
 * Step 2's villages (part of the edit session): a village put down with the Village tool (its ghost shot where it may go
 * and over the main town, where it may not), the town built again with it (the real `edit` job's settle: what it took,
 * what it refused); dragged elsewhere and built again; picked and removed from its card (what goes listed first); the
 * removal undone. Each step shot; the settles' timings and the Problems list said.
 */
async function villageSession(page, renderer, { bench, state, shots, built, tool, click }) {
  /** What the editor says once the town is built again after a village edit: the areas the settled town has, the refusal if any, what blocks. */
  const settled = async (what, areas) => {
    const s = await built(what);
    console.log(`create: ${renderer} edit ${what}: settled town has ${s.shownAreas} areas (the plan ${s.areas.length}, wanted ${areas})${s.note ? `; said "${s.note}"` : ''}; problems: ${s.blocking.length} blocking${s.blocking.length ? ` (${s.blocking.join('; ')})` : ''}, ${s.warnings} warnings`);
    if (s.areas.length !== areas || s.shownAreas !== areas) fault(`${renderer} edit ${what}: ${s.areas.length} areas in the plan, ${s.shownAreas} in the town shown (wanted ${areas})`);
    return s;
  };
  await tool('village');
  const had = (await state()).areas.length;
  const { spots, each, fits, world } = await bench(() => window.__bench.villageSpots(false));
  console.log(`create: ${renderer} edit: ${world}: ${fits} village sites, ${spots.length} of them on screen; canPlaceVillage ${each.toFixed(3)} ms a test`);
  if (!spots.length) return fault(`${renderer} edit: nowhere on screen for a village`);
  // (Its ghost over the main town first: red, with why.)
  const main = await bench(() => window.__bench.village(0));
  const town = main ?? (await bench(() => window.__bench.lot('home', 0)));
  if (town) {
    await page.mouse.move(town.x, town.y, { steps: 4 });
    await page.waitForTimeout(150);
    console.log(`create: ${renderer} edit: the village's ghost over the main town says "${(await state()).hint}"`);
    await shots('village-refused');
  }
  let placed = false;
  for (const p of spots) {
    await page.mouse.move(p.x, p.y, { steps: 4 });
    await page.waitForTimeout(80);
    if (!/^Click to put a village down/.test((await state()).hint ?? '')) continue;
    await shots('village-ghost');
    const t0 = Date.now();
    await page.mouse.click(p.x, p.y);
    await page.waitForTimeout(200);
    const s = await state();
    placed = s.areas.length === had + 1;
    console.log(`create: ${renderer} edit: village ${placed ? `${s.areas[had].name} put down (${Date.now() - t0} ms to the click's answer)` : `not put down (${s.note})`}`);
    break;
  }
  if (!placed) return fault(`${renderer} edit: no village put down (${(await state()).hint})`);
  const area = had;
  await shots('village');
  await settled('place a village', had + 1);
  await shots('village-built');

  // Dragged elsewhere by its box.
  const v = await bench((a) => window.__bench.village(a), area);
  if (!v) return fault(`${renderer} edit: the village is not on screen to drag`);
  const targets = await bench(([a, gx, gz]) => window.__bench.villageMoves(a, gx, gz), [area, v.gx, v.gz]);
  const from = (await state()).areas[area];
  await page.mouse.move(v.x, v.y, { steps: 3 });
  await page.mouse.down();
  let dragged = false;
  for (const p of targets) {
    await page.mouse.move(p.x, p.y, { steps: 10 });
    await page.waitForTimeout(80);
    if (!/^Let go to move/.test((await state()).hint ?? '')) continue;
    await shots('village-drag');
    await page.mouse.up();
    dragged = true;
    break;
  }
  if (!dragged) {
    await page.mouse.up();
    return fault(`${renderer} edit: ${v.name} found nowhere to go (${(await state()).hint})`);
  }
  await page.waitForTimeout(250);
  const to = (await state()).areas[area];
  if (!to || (to.x === from.x && to.z === from.z)) fault(`${renderer} edit: ${v.name} not moved (${(await state()).note})`);
  else console.log(`create: ${renderer} edit: ${v.name} moved by ${to.x - from.x}, ${to.z - from.z}`);
  await shots('village-moved');
  await settled('move a village', had + 1);
  await shots('village-moved-built');

  // Picked (a click on it with the Village tool) and removed from its card.
  const w = await bench((a) => window.__bench.village(a), area);
  if (!w) return fault(`${renderer} edit: the village is not on screen to pick`);
  await click(w);
  if ((await state()).selected?.kind !== 'village') return fault(`${renderer} edit: a click on ${w.name} picked ${JSON.stringify((await state()).selected)}`);
  const says = await page.evaluate(() => document.querySelector('.card .option .hint')?.textContent ?? '');
  console.log(`create: ${renderer} edit: ${w.name}'s card: "${says}"`);
  await shots('village-card');
  await page.locator('.card button', { hasText: /^\s*Remove / }).click();
  await page.waitForTimeout(250);
  if ((await state()).areas.length !== had) fault(`${renderer} edit: removing ${w.name} left ${(await state()).areas.length} areas (wanted ${had})`);
  await shots('village-removed');
  await settled('remove a village', had);

  // The removal undone.
  await page.keyboard.press(process.platform === 'darwin' ? 'Meta+z' : 'Control+z');
  await page.waitForTimeout(300);
  if ((await state()).areas.length !== had + 1) fault(`${renderer} edit: undoing the removal left ${(await state()).areas.length} areas`);
  await settled('undo the removal', had + 1);
  await shots('village-undo');
  await page.keyboard.press('Escape');
}

/** Step 1: an arrow key down the place grid, the place a row down chosen and its land made (see where it is called). */
async function anotherPlace(page, renderer, shoot) {
  await page.locator('.places [role="radio"][aria-checked="true"]').focus();
  const { before, row, count } = await page.evaluate(() => {
    const grid = document.querySelector('.places');
    const radios = [...grid.querySelectorAll('[role="radio"]')];
    return { before: radios.findIndex((b) => b.getAttribute('aria-checked') === 'true'), row: getComputedStyle(grid).gridTemplateColumns.split(' ').length, count: radios.length };
  });
  const [key, wanted] = before + row < count ? ['ArrowDown', before + row] : ['ArrowRight', (before + 1) % count];
  await page.keyboard.press(key);
  const after = await page.evaluate(() => {
    const radios = [...document.querySelectorAll('.places [role="radio"]')];
    return { checked: radios.findIndex((b) => b.getAttribute('aria-checked') === 'true'), focused: radios.indexOf(document.activeElement) };
  });
  if (after.checked !== wanted || after.focused !== wanted) fault(`${renderer}: ${key} from place ${before} chose ${after.checked}, focus on ${after.focused} (wanted ${wanted})`);
  await page.waitForFunction(() => !!document.querySelector('.making'), null, { timeout: 10_000, polling: 50 }).catch(() => fault(`${renderer}: no making panel after a new place`));
  await page.waitForTimeout(700);
  if (shots.has('1')) await shoot(`${sizes[0].width}x${sizes[0].height}-making`);
  await ready(page);
  await page.waitForTimeout(2500);
  if (shots.has('1')) await shoot(`${sizes[0].width}x${sizes[0].height}-place`);
}

const { chromium } = await playwright();
const browser = await chromium.launch({
  headless: !has('headed'),
  ...(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chromium' }),
  args: ['--enable-unsafe-webgpu', '--disable-backgrounding-occluded-windows', '--disable-renderer-backgrounding', '--disable-background-timer-throttling'],
});
let faults = 0;
const fault = (line) => {
  faults++;
  console.warn(`create: ${line}`);
};
try {
  for (const renderer of renderers) {
    const context = await browser.newContext({ viewport: sizes[0], deviceScaleFactor: 1 });
    await context.addInitScript(quiet);
    await context.addInitScript(seeded, seed);
    const page = await context.newPage();
    page.on('pageerror', (e) => console.error(`page error: ${e.message}`));
    const query = [renderer === 'webgl' ? 'renderer=webgl' : '', ...worldQuery].filter(Boolean).join('&');
    await page.goto(`${base}/?${query}`);
    await page.waitForFunction(() => !!document.querySelector('.menu') && !document.querySelector('.loading'), null, { timeout: 180_000, polling: 200 });
    await page.waitForTimeout(1500);
    await page.getByText('New game', { exact: true }).click();
    await ready(page);
    // The stream pill up, as on a first visit while the models still come (by then they may all have
    // come): the shots show it, and the UI over the town is checked against it.
    await page.evaluate(async () => {
      const { streamState } = await import('/src/assets/streamState.svelte.ts');
      streamState.apply({ total: 400, done: 260, label: 'Loading houses', active: true });
    });

    let stepName = 'step1';
    const shoot = async (name) => {
      const file = join(out, `${stepName}-${renderer}-${name}.png`);
      await page.screenshot({ path: file });
      for (const line of await page.evaluate(overlaps)) fault(`${renderer} ${name}: ${line}`);
      const pill = await page.evaluate(() => !!document.querySelector('.stream'));
      console.log(`create: ${file}${pill ? ' (the stream pill up)' : ''}`);
    };
    /** The step at every size, and the narrowest with its choices (`scroller`) scrolled to their end. `settled`: waited for after each resize. */
    const everySize = async (scroller = '.side .panel', settled = async () => {}) => {
      for (const size of sizes) {
        await page.setViewportSize(size);
        // (The camera glides to frame the world in the resized view; its tags settle after it.)
        await page.waitForTimeout(2500);
        await settled();
        await shoot(`${size.width}x${size.height}`);
      }
      const narrow = sizes.reduce((a, b) => (b.width < a.width ? b : a));
      await page.setViewportSize(narrow);
      await page.waitForTimeout(800);
      const scrolls = await page.evaluate((selector) => {
        const panel = document.querySelector(selector);
        if (!panel) return false;
        panel.scrollTop = panel.scrollHeight;
        return panel.scrollHeight > panel.clientHeight + 1;
      }, scroller);
      await settled();
      await page.waitForTimeout(400);
      await shoot(`${narrow.width}x${narrow.height}-scrolled`);
      if (!scrolls) console.log(`create: ${renderer} ${stepName}: the panel fits at ${narrow.width}x${narrow.height} (nothing to scroll)`);
    };
    if (shots.has('1')) await everySize();

    // Back to the first size: an arrow key down the place grid chooses the place a row down (as
    // many as the grid shows in a row; across by one if there is no row under it), and its world is
    // made (the making panel shown meanwhile). Not with `--keep-place` (`--code`'s place kept through
    // step 1) or `--stay`.
    await page.setViewportSize(sizes[0]);
    await page.waitForTimeout(1200);
    if (!has('keep-place') && !has('stay')) await anotherPlace(page, renderer, shoot);

    // Step 2: the town laid on that land (its making panel shot while it is laid), then at every size.
    await page.getByRole('button', { name: /Lay out the town/ }).click();
    stepName = 'step2';
    await page.waitForFunction(() => !!document.querySelector('.making'), null, { timeout: 10_000, polling: 50 }).catch(() => fault(`${renderer}: no making panel while the town is laid out`));
    await page.waitForTimeout(700);
    if (shots.has('2')) await shoot(`${sizes[0].width}x${sizes[0].height}-making`);
    await townReady(page);
    const eyebrow = await page.evaluate(() => document.querySelector('.side .head .eyebrow')?.textContent ?? '');
    if (!/Step 2 of 4 · Town/.test(eyebrow)) fault(`${renderer}: step 2 says "${eyebrow}"`);
    if (shots.has('2')) await everySize();

    if (shots.has('edit')) await editSession(page, renderer, (name) => shoot(name), (name) => (stepName = name));

    if (shots.has('3')) {
      // Step 3: the town lived in, its households offered as cards; shot once the portraits in view are drawn.
      await page.setViewportSize(sizes[0]);
      await page.waitForTimeout(800);
      stepName = 'step3';
      const opened = Date.now();
      await page.getByRole('button', { name: /Choose a household/ }).click();
      await page.waitForFunction(() => document.querySelectorAll('.offer .card').length > 0, null, { timeout: 60_000, polling: 50 });
      const cardsAt = Date.now() - opened;
      /** The portraits in the cards on screen drawn (each has its picture). */
      const inView = () =>
        page
          .waitForFunction(
            () => {
              const list = document.querySelector('.offer .cards');
              if (!list) return false;
              const box = list.getBoundingClientRect();
              const faces = [...list.querySelectorAll('.face')].filter((f) => {
                const r = f.getBoundingClientRect();
                return r.bottom > box.top && r.top < box.bottom;
              });
              // (Scrolled to the end, the cards in view may show no faces: the last card's are above.)
              return list.querySelector('.card') !== null && faces.every((f) => f.querySelector('img'));
            },
            null,
            { timeout: 60_000, polling: 50 },
          )
          .catch(() => fault(`${renderer}: step 3's portraits in view not drawn within 60 s`));
      await inView();
      const viewAt = Date.now() - opened;
      await page
        .waitForFunction(() => [...document.querySelectorAll('.offer .face')].every((f) => f.querySelector('img')), null, { timeout: 120_000, polling: 100 })
        .catch(() => fault(`${renderer}: step 3's portraits not all drawn within 120 s`));
      const allAt = Date.now() - opened;
      const offer = await page.evaluate(() => ({
        step: document.querySelector('.side .head .eyebrow')?.textContent ?? '',
        cards: [...document.querySelectorAll('.offer .card[data-slot]')].map((c) => Number(c.getAttribute('data-slot'))),
        faces: document.querySelectorAll('.offer .face').length,
        households: Number(/(\d+) households/.exec(document.querySelector('.view .stats')?.textContent ?? '')?.[1] ?? NaN),
      }));
      console.log(`create: ${renderer} step 3: ${offer.cards.length} cards of ${offer.households} households, ${offer.faces} portraits; cards ${cardsAt} ms, portraits in view ${viewAt} ms, all ${allAt} ms`);
      if (!/Step 3 of 4 · Household/.test(offer.step)) fault(`${renderer}: step 3 says "${offer.step}"`);
      if (offer.cards.length > 30 || offer.cards.length < Math.min(10, offer.households)) fault(`${renderer}: step 3 offers ${offer.cards.length} of ${offer.households} households`);
      await everySize('.offer .cards', inView);

      // A card chosen: its home lit, the camera on its area, the way on naming them.
      await page.setViewportSize(sizes[0]);
      await page.waitForTimeout(1200);
      await page.locator('.offer .card[data-slot] .pick').nth(2).click();
      await page.waitForTimeout(2500);
      const way = await page.evaluate(() => document.querySelector('.view .actions .btn.primary')?.textContent?.trim() ?? '');
      if (!/^Play the .+s →$/.test(way)) fault(`${renderer}: with a card chosen the way on says "${way}"`);
      await shoot(`${sizes[0].width}x${sizes[0].height}-chosen`);
      // Show others: other households.
      await page.getByRole('button', { name: /Show others/ }).click();
      await page.waitForTimeout(400);
      await inView();
      await page.waitForTimeout(1500);
      const others = await page.evaluate(() => [...document.querySelectorAll('.offer .card[data-slot]')].map((c) => Number(c.getAttribute('data-slot'))));
      const again = others.filter((s) => offer.cards.includes(s)).length;
      console.log(`create: ${renderer} step 3: Show others offers ${others.length}, ${again} of them shown before`);
      if (offer.households > offer.cards.length + 10 && again) fault(`${renderer}: Show others offered ${again} households again`);
      await shoot(`${sizes[0].width}x${sizes[0].height}-others`);
    }
    await context.close();
  }
} finally {
  await browser.close();
  server?.kill();
}
console.log(faults ? `create: ${faults} fault(s); pictures in ${out}` : `create: clean; pictures in ${out}`);
