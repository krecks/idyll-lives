// Step 2's settle (`landFollow.ts` `settleEdit`, the town worker's `edit` job, `townJobs.ts`): the town's
// plan draft settled on its land as laid, as the worker would (the job and its answer through a structured
// clone). The town as laid settles again to what the `town` job gave; nothing is ever given up, however
// steep the land; a street drawn up a 20 % slope is refused with its reason and the plan before it settled
// instead; a street bridged over a river keeps its deck dry for walkers; undo settles to the same bytes; a
// lot removed and added again settles like the original; an edit's settle on the biggest worlds is timed.
// `SLOW_TESTS=1 bun test tests/editSettle.slow.test.ts`.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import type { RoadPlan } from '../src/core/protocol';

const WEB = new URL('../', import.meta.url).pathname;

describe("step 2's settle", async () => {
  // --- the page's world in bun: content from web/public (put back for whatever runs next) ---
  const realLocation = globalThis.location;
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' } });
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async (input: string | URL | Request) => {
    const url = new URL(typeof input === 'string' ? input : input instanceof URL ? input.href : input.url, 'http://localhost/');
    const path = `${WEB}public${decodeURIComponent(url.pathname)}`;
    return existsSync(path) ? new Response(readFileSync(path)) : new Response('missing', { status: 404 });
  }) as typeof fetch;
  afterAll(() => {
    globalThis.fetch = realFetch;
    Object.assign(globalThis, { location: realLocation });
  });
  const { Content } = await import('../src/content/content');
  const { objectInfos } = await import('../src/game/housegen');
  const { useTownContent } = await import('../src/game/town');
  const { runTownJob } = await import('../src/game/townJobs');
  const { settleLand } = await import('../src/game/landFollow');
  const { previewWorld } = await import('../src/game/townPreview');
  const { drawStreet } = await import('../src/game/edit/streets');
  const { addLot, moveLot, removeLotsEdit, lotKey } = await import('../src/game/edit/lots');
  const { entriesOf } = await import('../src/game/edit/ground');
  const { TownHistory } = await import('../src/game/edit/history');
  const { deckTiles } = await import('../src/game/macro');
  const { wetTiles } = await import('../src/game/landEdit');
  const { unrle } = await import('../src/game/landMasks');
  const { LAND_CELL, NONE } = await import('../src/game/land');
  const { lengths, nearest, pointAt } = await import('../src/game/roads');
  type Templates = import('../src/game/town').Templates;
  type NeighbourhoodDraft = import('../src/game/town').NeighbourhoodDraft;
  type TownJob = import('../src/game/townJobs').TownJob;
  type TownResult = import('../src/game/townJobs').TownResult;
  type WorldOptions = import('../src/game/world').WorldOptions;
  type Pt = import('../src/game/roads').Pt;
  const content = await Content.load('http://localhost/content/base.json');
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };

  /** A job as the worker runs it: sent, run, and its result sent back. */
  const inWorker = <K extends TownJob['kind']>(job: Extract<TownJob, { kind: K }>) => structuredClone(runTownJob(content, t, structuredClone(job))) as Extract<TownResult, { kind: K }>;
  /** A town laid on a place's land (the `land` and `town` jobs), with its plan. */
  const laid = (world: string, seed: number, options: Partial<WorldOptions>) => {
    const land = inWorker({ kind: 'land', world, seed, options });
    const town = inWorker({ kind: 'town', land: land.draft, seed, options, waterKept: true });
    if (!town.plan) throw new Error('the town has no plan');
    return town;
  };
  /** The plans `steps` went through settled (`steps[0]` the plan last settled), as step 2's editor asks; `plan` alone with none. */
  const settle = (steps: readonly NeighbourhoodDraft[], before?: readonly string[]) => inWorker({ kind: 'edit', plan: steps[steps.length - 1], ops: steps.length - 1, ...(steps.length > 1 ? { steps } : {}), ...(before ? { before } : {}), check: false });
  /** A draft as bytes: its typed arrays (the land) written out. */
  const bytes = (v: unknown) => JSON.stringify(v, (_, x) => (ArrayBuffer.isView(x) ? Array.from(x as Int16Array).join(',') : x));
  const done = <T>(e: { draft: T } | { refused: string }): T => {
    if ('refused' in e) throw new Error(`refused: ${e.refused}`);
    return e.draft;
  };
  /** Nothing of `plan` given up in `settled`: the same lots, roads, backdrop and places. */
  const keptAll = (settled: NeighbourhoodDraft, plan: NeighbourhoodDraft) => {
    expect(settled.slots).toEqual(plan.slots);
    expect(settled.roads).toEqual(plan.roads);
    expect(settled.open?.backdrop).toEqual(plan.open?.backdrop);
    expect(settled.open?.places).toEqual(plan.open?.places);
    expect(settled.diagnostics?.slopes).toEqual(plan.diagnostics?.slopes);
  };

  const village = laid('country', 7, { town: 'village', villages: 'few', size: 'medium' });

  test('the town as laid settles again to what the town job gave, its zones and water too', () => {
    const e = settle([village.plan!]);
    expect(e.refused).toEqual([]);
    expect(e.step).toBe(0);
    expect(bytes(e.draft)).toBe(bytes(village.draft));
    expect(bytes(e.preview)).toBe(bytes(village.preview));
    // (Its land is the plan's graded: the plan's own is as laid.)
    expect(bytes(village.plan!.open!.land!.h)).not.toBe(bytes(e.draft.open!.land!.h));
  }, 120_000);

  test('a street drawn up a 20 % slope is refused with its reason; nothing is ever given up', () => {
    const plan = village.plan!;
    const [W, D] = [plan.width, plan.depth];
    const areas = plan.open!.areas;
    const off = ([x, z]: Pt) => Math.min(...areas.map((a) => Math.hypot(Math.max(a.x - x, 0, x - a.x - a.w), Math.max(a.z - z, 0, z - a.z - a.d))));
    // A country road well out of every village, and a street drawn from it square to it out of the world.
    let found: { drawn: NeighbourhoodDraft; from: Pt; dir: Pt; road: RoadPlan } | null = null;
    for (const r of plan.roads) {
      if (r.kind !== 'country' || r.bridge || found) continue;
      const ls = lengths(r.points);
      for (let s = 30; s < ls[ls.length - 1] - 30 && !found; s += 15) {
        const { p, t: tan } = pointAt(r.points, s, ls);
        if (off(p) < 120) continue;
        for (const side of [1, -1]) {
          const dir: Pt = [-tan[1] * side, tan[0] * side];
          const toEdge = Math.min(...[dir[0] > 0 ? (W - p[0]) / dir[0] : dir[0] < 0 ? -p[0] / dir[0] : Infinity, dir[1] > 0 ? (D - p[1]) / dir[1] : dir[1] < 0 ? -p[1] / dir[1] : Infinity]);
          if (toEdge < 140 || toEdge > 320) continue;
          const points: Pt[] = [p, [p[0] + dir[0] * (toEdge + 10), p[1] + dir[1] * (toEdge + 10)]];
          const e = drawStreet(plan, t, content, { points, kind: 'street', clear: false, fill: false, seed: 5 });
          if ('refused' in e) continue;
          const road = e.draft.roads[e.draft.roads.length - 1];
          if (road.kind !== 'country' || road.ends[1] !== 'out' || nearest(road.points, ...p).dist > 6) continue;
          found = { drawn: e.draft, from: p, dir, road };
          break;
        }
      }
    }
    if (!found) throw new Error('no country road to draw a street from');
    // The land under it made a 20 % slope rising from the road it leaves (to either side of it too).
    const land = plan.open!.land!;
    const h = land.h.slice();
    for (let k = 0; k < h.length; k++) {
      if (land.wl[k] !== NONE && land.wl[k] > land.h[k]) continue;
      const [x, z] = [(k % land.cols) * LAND_CELL - found.from[0], Math.floor(k / land.cols) * LAND_CELL - found.from[1]];
      const along = x * found.dir[0] + z * found.dir[1];
      const across = Math.abs(-x * found.dir[1] + z * found.dir[0]);
      const fade = across < 60 ? 1 : across > 120 ? 0 : 1 - (across - 60) / 60;
      if (along > 0) h[k] += Math.round(along * 0.2 * fade * 100);
    }
    const steep = (d: NeighbourhoodDraft): NeighbourhoodDraft => ({ ...d, open: { ...d.open!, land: { ...land, h } } });
    const [before, after] = [steep(plan), steep(found.drawn)];
    const e = settle([before, after]);
    expect(e.refused.length).toBe(1);
    expect(e.refused[0].op).toBe(0);
    expect(e.refused[0].why).toContain(found.road.name!);
    expect(e.refused[0].why).toMatch(/would climb \d+ %|dug out or built up [\d.]+ m/);
    console.log(`refused: ${e.refused[0].why}`);
    // The plan before it settled instead, as the town on screen was.
    expect(e.step).toBe(0);
    expect(bytes(e.draft)).toBe(bytes(settle([before]).draft));
    // Settled as it is, nothing goes (the new town's last resort, `settleLand`, would give some of it up).
    const kept = settle([after]);
    expect(kept.refused).toEqual([]);
    keptAll(kept.draft, after);
    expect(kept.marks.length).toBeGreaterThan(0);
    const given = settleLand(after, (d) => previewWorld(content, t, d, null)).draft.diagnostics?.slopes;
    expect(given && given.lots + given.roads + given.backdrop + given.courts).toBeGreaterThan(0);
  }, 300_000);

  test('a street bridged over a river keeps its deck dry for walkers once settled', () => {
    const river = laid('river', 7, { town: 'village', villages: 'few', size: 'medium' });
    const plan = river.plan!;
    const [W, D] = [plan.width, plan.depth];
    const wet = unrle(plan.open!.water, W * D);
    const wetAt = ([x, z]: Pt) => x >= 0 && z >= 0 && x < W && z < D && wet[Math.floor(z) * W + Math.floor(x)] === 1;
    const had = plan.roads.filter((r) => r.bridge).length;
    let tries = 0;
    for (const r of plan.roads) {
      if (r.kind === 'trail' || r.bridge) continue;
      const ls = lengths(r.points);
      for (let s = 20; s < ls[ls.length - 1] - 20; s += 12) {
        const { p, t: tan } = pointAt(r.points, s, ls);
        for (const side of [1, -1]) {
          const dir: Pt = [-tan[1] * side, tan[0] * side];
          // (Water within 50 m, no wider than 35 m, then dry land: drawn on to the world's edge or the first road it meets.)
          let [into, out] = [-1, -1];
          for (let d = 4; d < 120; d += 1) {
            const w = wetAt([p[0] + dir[0] * d, p[1] + dir[1] * d]);
            if (w && into < 0) into = d;
            if (!w && into >= 0) {
              out = d;
              break;
            }
          }
          if (into < 0 || into > 50 || out < 0 || out - into > 35) continue;
          const e = drawStreet(plan, t, content, { points: [p, [p[0] + dir[0] * 2000, p[1] + dir[1] * 2000]], kind: 'street', clear: false, fill: false, seed: 3 });
          if ('refused' in e || e.draft.roads.filter((q) => q.bridge).length <= had) continue;
          if (++tries > 6) throw new Error('no bridge settled');
          const settled = settle([plan, e.draft]);
          if (settled.refused.length) continue;
          keptAll(settled.draft, e.draft);
          // Every deck's tiles dry in the walkers' water, and the new one's over water the land has.
          const walk = unrle(settled.draft.open!.water, W * D);
          const water = wetTiles(settled.draft.open!.land!, W, D);
          let [decks, over] = [0, 0];
          for (const q of settled.draft.roads) {
            if (!q.bridge) continue;
            deckTiles(q, q.bridge, W, D, (tile) => {
              decks++;
              expect(walk[tile]).toBe(0);
              if (water[tile]) over++;
            });
          }
          expect(over).toBeGreaterThan(100);
          // (The rest of the water still water.)
          let wetLeft = 0;
          for (let k = 0; k < walk.length; k++) wetLeft += walk[k];
          expect(wetLeft).toBeGreaterThan(over);
          console.log(`bridged: ${settled.draft.roads.filter((q) => q.bridge).length - had} new, ${decks} deck tiles dry`);
          return;
        }
      }
    }
    throw new Error('no street could be bridged over the river');
  }, 300_000);

  test('undo settles to the same bytes', () => {
    const plan = village.plan!;
    const history = new TownHistory(plan);
    const home = plan.slots.findIndex((s) => s.kind === 'house');
    const removed = done(removeLotsEdit(plan, new Set([home])));
    history.push(removed);
    const first = settle([plan]);
    const edited = settle([plan, removed], first.marks);
    expect(edited.refused).toEqual([]);
    expect(edited.draft.slots.length).toBe(plan.slots.length - 1);
    const back = history.undo()!;
    expect(back).toBe(plan);
    const again = settle([removed, back], edited.marks);
    expect(again.refused).toEqual([]);
    expect(bytes(again.draft)).toBe(bytes(first.draft));
    expect(bytes(again.preview)).toBe(bytes(first.preview));
    // A refused edit is dropped from the history: none to redo.
    history.redo();
    expect(history.revert(plan)).toBe(plan);
    expect(history.canRedo).toBe(false);
  }, 120_000);

  test('a lot removed and added again settles like the original', () => {
    const plan = village.plan!;
    const original = settle([plan]);
    const spots = entriesOf(plan, t);
    const cell = (n: number) => [(n % plan.open!.land!.cols) * LAND_CELL, Math.floor(n / plan.open!.land!.cols) * LAND_CELL];
    let checked = 0;
    for (const [i, s] of plan.slots.entries()) {
      if (s.kind !== 'house' || s.lot === 'terrace' || s.lot === 'farm' || checked) continue;
      const removed = removeLotsEdit(plan, new Set([i]));
      if ('refused' in removed) continue;
      // A home of its size where it stood (another house: its seed's).
      const again = [1, 2, 3, 4, 5, 6].map((seed) => addLot(removed.draft, t, content, spots[i], { kind: 'house', size: s.lot as 'small' | 'medium' | 'large' }, seed)).find((e) => !('refused' in e));
      if (!again || 'refused' in again) continue;
      const added = again.draft.slots[again.draft.slots.length - 1];
      if (Math.hypot(added.x - s.x, added.z - s.z) > 3 || added.facing !== s.facing) continue;
      const e = settle([plan, removed.draft, again.draft], original.marks);
      expect(e.refused).toEqual([]);
      keptAll(e.draft, again.draft);
      // Every other lot where it was; the land as the original's, but by the lot.
      expect(new Set(e.draft.slots.slice(0, -1).map(lotKey))).toEqual(new Set(plan.slots.filter((_, k) => k !== i).map(lotKey)));
      const [a, b] = [original.draft.open!.land!.h, e.draft.open!.land!.h];
      let [far, near] = [0, 0];
      for (let k = 0; k < a.length; k++) {
        const [x, z] = cell(k);
        const off = Math.hypot(Math.max(s.x - x, 0, x - s.x - (s.w ?? 20)), Math.max(s.z - z, 0, z - s.z - (s.d ?? 20)));
        if (off > 40) far = Math.max(far, Math.abs(a[k] - b[k]));
        else near = Math.max(near, Math.abs(a[k] - b[k]));
      }
      expect(far).toBeLessThanOrEqual(2);
      expect(near).toBeLessThanOrEqual(20);
      expect(bytes(e.draft.open!.zones)).toBe(bytes(original.draft.open!.zones));
      checked++;
    }
    expect(checked).toBe(1);
    // A lot taken up and put back where it stood: the same land to the centimetre.
    for (const [i, s] of plan.slots.entries()) {
      if (s.kind !== 'house' || s.lot === 'terrace') continue;
      for (const [road, r] of plan.roads.entries()) {
        if (r.kind === 'trail' || road === spots[i].road) continue;
        const L = lengths(r.points)[r.points.length - 1];
        const away = Array.from({ length: Math.floor(L / 8) }, (_, k) => [1, -1].map((side) => moveLot(plan, t, i, { road, s: k * 8 + 4, side: side as 1 | -1 }))).flat().find((e) => !('refused' in e));
        if (!away || 'refused' in away) continue;
        const back = moveLot(away.draft, t, i, spots[i]);
        if ('refused' in back) continue;
        const b = back.draft.slots[i];
        if (b.x !== s.x || b.z !== s.z || b.facing !== s.facing) continue;
        const e = settle([plan, away.draft, back.draft], original.marks);
        expect(e.refused).toEqual([]);
        expect(bytes(e.draft.open!.land)).toBe(bytes(original.draft.open!.land));
        return;
      }
    }
    throw new Error('no lot could be moved and put back');
  }, 300_000);

  test('an edit settled on the biggest worlds, timed', () => {
    for (const kind of ['country', 'seaside', 'river']) {
      const city = laid(kind, 7, { town: 'city', villages: 'many', size: 'large' });
      const plan = city.plan!;
      const first = settle([plan]);
      expect(bytes(first.draft)).toBe(bytes(city.draft));
      const home = plan.slots.findIndex((s) => s.kind === 'house');
      const removed = done(removeLotsEdit(plan, new Set([home])));
      const times: number[] = [];
      for (let k = 0; k < 3; k++) {
        const t0 = performance.now();
        const e = runTownJob(content, t, { kind: 'edit', plan: removed, ops: 1, steps: [plan, removed], before: first.marks, check: false });
        times.push(performance.now() - t0);
        if (e.kind !== 'edit') throw new Error('not an edit');
        expect(e.refused).toEqual([]);
      }
      let t0 = performance.now();
      runTownJob(content, t, { kind: 'edit', plan: removed, ops: 1, steps: [plan, removed], before: first.marks });
      const checked = performance.now() - t0;
      // (The first edit after the town is laid: no marks yet, so the town as laid is settled too.)
      t0 = performance.now();
      runTownJob(content, t, { kind: 'edit', plan: removed, ops: 1, steps: [plan, removed], check: false });
      const unmarked = performance.now() - t0;
      times.sort((a, b) => a - b);
      console.log(`${kind} city, many villages (${plan.slots.length} lots, ${plan.roads.length} roads, ${plan.open?.backdrop?.length ?? 0} buildings): an edit settled in ${times[1].toFixed(0)} ms (M3), ${checked.toFixed(0)} ms with the checks, ${unmarked.toFixed(0)} ms with no marks yet`);
      // (Generous: the plan's estimate is about 2 s on the M1.)
      expect(times[1]).toBeLessThan(3000);
    }
  }, 600_000);
});
