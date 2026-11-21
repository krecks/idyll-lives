#!/usr/bin/env bun
/**
 * The new game's camera shots and lot tints on the menus' town, without going through the screens: a bare land (the
 * town stripped, `landDraft`) from the `world` shot, then the town on it from the `world` shot and from the `area`
 * shot of the main town and of a village, then the lots for sale lit up (`marked`) with the dearer ones in their own
 * tint (`dear`). A pan far past the land's edge shows the camera kept over the land. On each renderer.
 *
 * Then it times the town view built again after a change to the draft, as step 4's change of home does it
 * (`menuScene.show` with another draft: the preview on the page, then the renderer's `showTown`): a lot removed, a
 * street added, the same town with a home chosen. Printed per change, with the renderer's spans in it.
 *
 * The page's UI is hidden in the pictures; a grey band on the left stands for the side panel the shots keep clear of.
 *
 *   bun tools/bench/townShots.mjs [options]
 *     --renderers webgpu,webgl     the page's renderer
 *     --size 1920x1080             the window (CSS px)
 *     --world country --options town=town,villages=some    the place and options (`worldFlags.mjs`)
 *     --seed 7                     the world's seed
 *     --port 5297 | --url <u>      a dev server on the port (started if none answers), or one already running
 *     --out <dir>                  default /tmp/ng-render/
 *     --timings                    how long each picture waited to be ready (the glide, the textures, the frames)
 *     --no-time                    pictures only
 *     --no-shots                   timings only (from the bare land)
 *     --follow                     the town on a land it follows (`withLand`), as every new game's will be
 *
 * Needs Playwright like render.mjs (`PLAYWRIGHT=<path to playwright/index.mjs>`).
 */
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { benchContext, devServer, launch, settleShot } from './lib/browser.mjs';
import { worldFlags } from './worldFlags.mjs';

const args = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 && args[i + 1] !== undefined && !args[i + 1].startsWith('--') ? args[i + 1] : fallback;
};
const has = (name) => args.includes(`--${name}`);

const [width, height] = flag('size', '1920x1080').split('x').map(Number);
const renderers = flag('renderers', 'webgpu,webgl').split(',');
const seed = Number(flag('seed', '7'));
const made = worldFlags(flag, 'country');
const port = Number(flag('port', '5297'));
const out = flag('out', '/tmp/ng-render');
const timing = !has('no-time');
const shots = !has('no-shots');
mkdirSync(out, { recursive: true });

const { url: base, stop } = await devServer({ url: flag('url', null), port });

/** In the page: the modules the shots need, the UI hidden, a band where the side panel would be. */
async function setUp({ kind, options, seed, side, follow }) {
  const [{ menuScene }, { landDraft }, { generateWorld, loadTemplates }, { services }, { withLand }, { previewWorld }] = await Promise.all([import('/src/game/menuScene.ts'), import('/src/game/macro.ts'), import('/src/game/town.ts'), import('/src/ui/services.ts'), import('/src/game/landFollow.ts'), import('/src/game/townPreview.ts')]);
  const { removeLots } = await import('/src/game/edit/lots.ts');
  const t = await loadTemplates();
  const land = landDraft(services.content, t, kind, options, seed);
  const made = generateWorld(services.content, t, kind, seed, undefined, undefined, options);
  const town = follow && !made.open?.land ? withLand(made, previewWorld(services.content, t, made, null)) : made;
  const style = document.createElement('style');
  style.textContent = 'body * { visibility: hidden !important; } canvas, .ng-side { visibility: visible !important; }';
  document.head.append(style);
  const band = document.createElement('div');
  band.className = 'ng-side';
  Object.assign(band.style, { position: 'fixed', left: '0', top: '0', bottom: '0', width: `${side}px`, background: 'rgba(40, 44, 52, 0.55)', zIndex: '99999', pointerEvents: 'none' });
  document.body.append(band);
  window.__ng = { menuScene, land, town, removeLots, frame: { left: side, top: 0, right: innerWidth, bottom: innerHeight } };
  return { areas: town.open?.areas.length ?? 0, slots: town.slots.length, roads: town.roads.length, width: town.width, depth: town.depth, land: town.open?.land?.seeded ?? 'none' };
}

/** In the page: shows a draft (`land`, `town` or one put there) and frames a shot, cut, then waits for its frames. */
async function frameShot({ which, shot, home = null }) {
  const ng = window.__ng;
  const draft = ng[which];
  await ng.menuScene.show(draft, home);
  ng.menuScene.shoot({ ...shot, frame: ng.frame }, 0, true);
  // (The player's pan and zoom from the shot before let go.)
  ng.menuScene.reframe(0);
  await ng.menuScene.settled();
}

const { browser } = await launch({ headless: !has('headed') });
let faults = 0;
try {
  for (const renderer of renderers) {
    const context = await benchContext(browser, { viewport: { width, height }, deviceScaleFactor: 1 });
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', (e) => errors.push(e.message));
    page.on('console', (m) => m.type() === 'error' && errors.push(m.text()));
    await page.goto(`${base}/?${renderer === 'webgl' ? 'renderer=webgl' : ''}`);
    await page.waitForFunction(() => !!document.querySelector('.menu') && !document.querySelector('.loading'), null, { timeout: 180_000, polling: 200 });
    const facts = await page.evaluate(setUp, { kind: made.world, options: made.options, seed, side: Math.round(width * 0.24), follow: has('follow') });
    console.log(`townShots: ${renderer}: ${made.world} ${made.query || '(its own options)'} seed ${seed}: ${facts.width}x${facts.depth} m, ${facts.areas} areas, ${facts.slots} lots, ${facts.roads} roads, land ${facts.land}`);
    const shoot = async (name) => {
      const ms = await page.evaluate(settleShot, { hook: '__ng', minMs: 1200 });
      if (has('timings')) console.log(`  settled in ${ms.join(' + ')} ms (glide, textures, frames)`);
      const file = join(out, `${renderer}-${name}.png`);
      await page.screenshot({ path: file });
      console.log(`townShots: ${file}`);
    };

    if (shots) {
      await page.evaluate(frameShot, { which: 'land', shot: { kind: 'world' } });
      await shoot('land-world');
      // Panned far past the land's edge: the camera stops over the land (its target at the corner).
      await page.evaluate(() => window.__ng.menuScene.pan(-5000, -5000));
      await shoot('land-panned');
      // Dragged from near the top of the picture (the far land, or the sky) down: the land comes towards the camera.
      await page.evaluate(() => {
        window.__ng.menuScene.reframe(0);
        for (let k = 0; k < 10; k++) window.__ng.menuScene.drag(innerWidth * 0.6, 40 + k * 30, innerWidth * 0.6, 70 + k * 30);
      });
      await shoot('land-dragged');

      await page.evaluate(frameShot, { which: 'town', shot: { kind: 'world' } });
      await shoot('town-world');
      await page.evaluate(frameShot, { which: 'town', shot: { kind: 'area', area: 0 } });
      await shoot('town-area0');
      if (facts.areas > 1) {
        await page.evaluate(frameShot, { which: 'town', shot: { kind: 'area', area: 1 } });
        await shoot('town-area1');
      }
      // Step 2's ghosts over the main town: a street drawn, one that may not go (red), a bridge, a lot placed, one that
      // may not go and one to be removed (its house under a veil), a village's box placed and one that may not go;
      // then the time one change of them takes (as on every pointer move).
      await page.evaluate(frameShot, { which: 'town', shot: { kind: 'area', area: 0 } });
      const ghostMs = await page.evaluate(() => {
        const ng = window.__ng;
        const world = ng.menuScene.preview;
        const a = ng.town.open?.areas[0] ?? { x: 0, z: 0, w: world.width, d: world.depth };
        const inside = (p) => p.x + p.w / 2 >= a.x && p.x + p.w / 2 <= a.x + a.w && p.z + p.d / 2 >= a.z && p.z + p.d / 2 <= a.z + a.d;
        const homes = world.plots.filter((p) => p.house && inside(p));
        const [cx, cz] = [a.x + a.w / 2, a.z + a.d / 2];
        const bend = (x0, z0, len, dz) => Array.from({ length: 13 }, (_, k) => [x0 + (len * k) / 12, z0 + dz * Math.sin((k / 12) * Math.PI)]);
        const bridged = ng.town.roads.find((r) => r.bridge);
        const bridge = bridged
          ? (() => {
              const [s0, s1] = [bridged.bridge[0] - 12, bridged.bridge[1] + 12];
              let s = 0;
              const pts = [];
              for (let k = 0; k < bridged.points.length; k++) {
                if (k) s += Math.hypot(bridged.points[k][0] - bridged.points[k - 1][0], bridged.points[k][1] - bridged.points[k - 1][1]);
                if (s >= s0 && s <= s1) pts.push(bridged.points[k]);
              }
              return pts.map(([x, z]) => [x + 14, z + 14]);
            })()
          : bend(cx - 60, cz + 60, 50, 0);
        const g = {
          roads: [
            { points: bend(cx - 80, cz - 30, 160, 25), width: 9, tone: 'new' },
            { points: bend(cx - 70, cz + 25, 120, -15), width: 7, tone: 'bad' },
            { points: bridge, width: 7, tone: 'bridge' },
          ],
          lots: [
            ...homes.slice(0, 2).map((p) => ({ x: p.x, z: p.z, w: p.w, d: p.d, tone: 'gone' })),
            ...homes.slice(2, 3).map((p) => ({ x: p.x + p.w * 3, z: p.z, w: p.w, d: p.d, tone: 'new' })),
            ...homes.slice(3, 4).map((p) => ({ x: p.x, z: p.z + p.d * 0.5, w: p.w, d: p.d, tone: 'bad' })),
          ],
          boxes: [
            { x: a.x + a.w + 30, z: a.z + 20, w: 110, d: 80, tone: 'new' },
            { x: a.x - 120, z: a.z + a.d - 90, w: 100, d: 80, tone: 'bad' },
          ],
        };
        ng.menuScene.ghosts(g);
        window.__ghosts = g;
        // (As on pointer moves: the street being drawn grows a little each time.)
        const t0 = performance.now();
        for (let k = 0; k < 100; k++) {
          g.roads[0].points[g.roads[0].points.length - 1] = [cx + 80 + k * 0.3, cz - 30];
          ng.menuScene.ghosts(g);
        }
        return (performance.now() - t0) / 100;
      });
      console.log(`townShots: ${renderer}: ghosts set again in ${ghostMs.toFixed(2)} ms`);
      await shoot('ghosts-area0');
      // Closer (the player's zoom on the main town's shot): the ribbons, the lots' outlines, the veil over a house.
      await page.evaluate(() => window.__ng.menuScene.nudge(0, 2.6));
      await shoot('ghosts-close');
      await page.evaluate(() => window.__ng.menuScene.reframe(0));
      await page.evaluate(() => {
        const ng = window.__ng;
        ng.menuScene.shoot({ kind: 'world', frame: ng.frame }, 0, true);
      });
      await shoot('ghosts-world');
      await page.evaluate(() => window.__ng.menuScene.ghosts(null));
      // Lots for sale lit up, the dearer half in the second tint; one hovered, one selected.
      await page.evaluate(() => {
        const ng = window.__ng;
        const homes = ng.town.slots.map((s, i) => (s.kind === 'house' ? i : -1)).filter((i) => i >= 0);
        const marked = homes.filter((_, k) => k % 2 === 0);
        const dear = marked.filter((_, k) => k % 3 === 0);
        ng.menuScene.shoot({ kind: 'area', area: 0, frame: ng.frame }, 0, true);
        ng.menuScene.highlight({ marked, dear, hover: marked[1] ?? null, selected: marked[2] ?? null, outlines: false }, true);
      });
      await shoot('town-tints');
      // The same from the whole land's shot, where step 4 opens: the lit lots washed over whole.
      await page.evaluate(() => {
        const ng = window.__ng;
        ng.menuScene.shoot({ kind: 'world', frame: ng.frame }, 0, true);
      });
      await shoot('town-tints-world');
      await page.evaluate(() => window.__ng.menuScene.highlight({}, true));
      // Back to the bare land with every lot outlined (as step 1 asks): no ring of the town's lots is left on it.
      await page.evaluate(() => window.__ng.menuScene.highlight({ outlines: true }, true));
      await page.evaluate(frameShot, { which: 'land', shot: { kind: 'area', area: 0 } });
      await shoot('land-again');
      await page.evaluate(() => window.__ng.menuScene.highlight({}, true));
    } else await page.evaluate(frameShot, { which: 'land', shot: { kind: 'world' } });

    if (timing) {
      const times = await page.evaluate(async () => {
        const ng = window.__ng;
        const town = ng.town;
        const report = () => window.__perfReport();
        const timed = async (name, draft, home = null) => {
          const before = report();
          const [spans, tasks] = [before.entries.length, before.longTasks.length];
          const t0 = performance.now();
          await ng.menuScene.show(draft, home);
          const ms = performance.now() - t0;
          const after = report();
          const parts = after.entries
            .slice(spans)
            .filter((e) => e.ms >= 5)
            .map((e) => `${e.name} ${Math.round(e.ms)}${e.detail ? ` (${e.detail})` : ''}`);
          const long = after.longTasks.slice(tasks);
          parts.push(`long tasks: ${long.length}, ${Math.round(long.reduce((a, t) => a + t.ms, 0))} ms in all: ${long.map((t) => `${Math.round(t.ms)} (${t.phase})`).join(', ')}`);
          return { name, ms: Math.round(ms), parts };
        };
        const results = [];
        // The town on screen first (after the bare land, a whole build: the land's landscape is another).
        results.push(await timed('the town after the bare land (whole)', town));
        results.push(await timed('the same town as a new draft (nothing changed)', { ...town }));
        const lot = town.slots.findIndex((s) => s.kind === 'house');
        // (As the editor removes one: the lots after it numbered again, and what names them with them.)
        const lessOne = ng.removeLots(town, new Set([lot]));
        results.push(await timed('a lot removed', lessOne));
        const road = town.roads.find((r) => r.kind !== 'trail' && r.points.length >= 2);
        const added = road ? { ...lessOne, roads: [...lessOne.roads, { ...road, name: 'Bench Lane', points: road.points.map(([x, z]) => [x + 30, z + 30]) }] } : lessOne;
        results.push(await timed('a street added', added));
        results.push(await timed('back to the town', town));
        results.push(await timed('a home chosen (step 4)', town, lot));
        results.push(await timed('another home chosen (step 4)', town, town.slots.findIndex((s, i) => i > lot + 5 && s.kind === 'house')));
        return results;
      });
      for (const t of times) console.log(`townShots: ${renderer}: ${t.name}: ${t.ms} ms\n    ${t.parts.join('\n    ')}`);
    }
    for (const e of errors) {
      faults++;
      console.warn(`townShots: ${renderer}: page error: ${e}`);
    }
    await context.close();
  }
} finally {
  await browser.close();
  stop();
}
console.log(faults ? `townShots: ${faults} fault(s); pictures in ${out}` : `townShots: clean; pictures in ${out}`);
