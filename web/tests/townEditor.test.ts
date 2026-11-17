// Step 2's town editor as state (`ui/townedit/`): an edit taken at once and the town settled once
// edits pause (one settle at a time, the latest plan), undo, redo and reset; the Problems list read
// from a draft as the checks read a whole town file (`problems.ts` `checkedTown`), sorted into what
// blocks the way on and what only warns; what is picked found again after an edit; the edits' ghosts.
import { afterAll, describe, expect, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import type { Edited } from '../src/game/edit/edited';
import type { NeighbourhoodDraft } from '../src/game/town';
import { EditSession, CHECK_AFTER, QUIET, QUIET_ONE, type Timers } from '../src/ui/townedit/session';
import type { Settled } from '../src/ui/townedit/settle';

/** Timers the test moves on by hand. */
function clock() {
  let now = 0;
  let id = 0;
  const due = new Map<number, { at: number; fn: () => void }>();
  const timers: Timers = {
    set(fn, ms) {
      due.set(++id, { at: now + ms, fn });
      return id;
    },
    clear: (h) => void due.delete(h as number),
  };
  /** Time moved on `ms`, every timer due meanwhile run in turn. */
  const advance = async (ms: number) => {
    const end = now + ms;
    for (;;) {
      const next = [...due.entries()].filter(([, d]) => d.at <= end).sort((a, b) => a[1].at - b[1].at)[0];
      if (!next) break;
      due.delete(next[0]);
      now = next[1].at;
      next[1].fn();
      await Promise.resolve();
    }
    now = end;
    // (The settles' promises run out.)
    for (let k = 0; k < 10; k++) await Promise.resolve();
  };
  return { timers, advance, now: () => now };
}

/** A draft standing for a town (the session compares them by identity only). */
const draft = (name: string) => ({ name, slots: [], roads: [] }) as unknown as NeighbourhoodDraft;
const edit = (d: NeighbourhoodDraft): Edited => ({ draft: d, cleared: { lots: [], backdrop: [] } });

/** A session on town `base`, its settles recorded (each held until `release`d when `hold` is set). */
function session(base = draft('base'), hold = false) {
  const c = clock();
  const settles: NeighbourhoodDraft[] = [];
  const shown: NeighbourhoodDraft[] = [];
  const waiting: (() => void)[] = [];
  let changed = 0;
  const s = new EditSession(base, {
    timers: c.timers,
    now: c.now,
    settle: async (plan) => {
      settles.push(plan);
      if (hold) await new Promise<void>((resolve) => waiting.push(resolve));
      return { draft: plan, preview: {} } as unknown as Settled;
    },
    show: async (settled) => void shown.push(settled.draft),
    changed: () => void changed++,
    check: (plan) => ({ blocking: plan.name === 'bad' ? [{ text: 'Bad' }] : [], warnings: [] }),
  });
  const release = async () => {
    waiting.shift()?.();
    await c.advance(0);
  };
  return { s, c, settles, shown, release, changes: () => changed };
}

describe('the edit session', () => {
  test('an edit takes at once; the town is settled once edits have paused, once for several', async () => {
    const { s, c, settles, shown } = session();
    const a = draft('a');
    const b = draft('b');
    // (Edits made in a run, as the Lot tool's homes: the longer pause.)
    expect(s.apply(edit(a), true)).toBeNull();
    expect(s.plan).toBe(a);
    expect(s.pending).toBe(true);
    await c.advance(QUIET - 100);
    expect(s.apply(edit(b), true)).toBeNull();
    await c.advance(QUIET - 1);
    expect(settles).toEqual([]);
    await c.advance(1);
    expect(settles).toEqual([b]);
    expect(shown).toEqual([b]);
    expect(s.settled).toBe(b);
    expect(s.pending).toBe(false);
    expect(s.timings.length).toBe(1);
  });

  test('an edit on its own is settled after the short pause; the marks the town was laid with go to the first settle', async () => {
    const c = clock();
    const base = draft('base');
    const asked: (readonly string[] | undefined)[] = [];
    const s = new EditSession(
      base,
      {
        timers: c.timers,
        now: c.now,
        settle: async (plan, ops, _signal, batch) => {
          asked.push(batch.before);
          return { draft: plan, preview: {}, refused: [], step: ops, marks: ['n'] } as unknown as Settled;
        },
        show: async () => {},
        changed: () => {},
      },
      undefined,
      undefined,
      ['m'],
    );
    s.apply(edit(draft('a')));
    await c.advance(QUIET_ONE - 1);
    expect(asked).toEqual([]);
    await c.advance(1);
    expect(asked).toEqual([['m']]);
    s.apply(edit(draft('b')));
    await c.advance(QUIET_ONE);
    expect(asked).toEqual([['m'], ['n']]);
  });

  test('an edit the settle refuses is undone, and the plan before it settled; the next settle has the batch and the marks', async () => {
    const c = clock();
    const base = draft('base');
    const [a, b, d] = [draft('a'), draft('b'), draft('d')];
    const asked: { plan: NeighbourhoodDraft; ops: number; steps: readonly NeighbourhoodDraft[]; before?: readonly string[] }[] = [];
    const told: string[] = [];
    const s = new EditSession(base, {
      timers: c.timers,
      now: c.now,
      settle: async (plan, ops, _signal, batch) => {
        asked.push({ plan, ops, ...batch });
        // (The second edit of the first batch goes over a limit.)
        return (asked.length === 1 ? { draft: a, preview: {}, refused: [{ op: 1, why: 'too steep' }], step: 1, marks: ['m'] } : { draft: plan, preview: {}, refused: [], step: ops, marks: ['m', 'n'] }) as unknown as Settled;
      },
      show: async () => {},
      changed: () => {},
      report: (why) => void told.push(why),
    });
    s.apply(edit(a));
    s.apply(edit(b));
    await c.advance(QUIET);
    expect(asked[0].steps).toEqual([base, a, b]);
    expect(asked[0].before).toBeUndefined();
    expect(told).toEqual(['too steep']);
    expect(s.plan).toBe(a);
    expect(s.settledFrom).toBe(a);
    expect(s.pending).toBe(false);
    expect(s.canRedo).toBe(false);
    expect(s.edits).toBe(1);
    s.apply(edit(d));
    await c.advance(QUIET);
    expect(asked[1].steps).toEqual([a, d]);
    expect(asked[1].before).toEqual(['m']);
    expect(s.undo()).toBe(true);
    expect(s.plan).toBe(a);
  });

  test('a refusal is given back and changes nothing', () => {
    const { s } = session();
    const before = s.plan;
    expect(s.apply({ refused: 'The town keeps its last home.' })).toBe('The town keeps its last home.');
    expect(s.plan).toBe(before);
    expect(s.edits).toBe(0);
  });

  test('edits during a settle are settled after it, the latest plan once', async () => {
    const { s, c, settles, release } = session(draft('base'), true);
    s.apply(edit(draft('a')));
    await c.advance(QUIET);
    expect(settles.map((d) => d.name)).toEqual(['a']);
    expect(s.settling).toBe(true);
    s.apply(edit(draft('b')));
    s.apply(edit(draft('c')));
    await c.advance(QUIET);
    // (Still the first: one settle at a time.)
    expect(settles.map((d) => d.name)).toEqual(['a']);
    await release();
    expect(settles.map((d) => d.name)).toEqual(['a', 'c']);
    await release();
    expect(s.settled.name).toBe('c');
    expect(s.pending).toBe(false);
    expect(s.settling).toBe(false);
  });

  test('flush settles now, without waiting for the pause', async () => {
    const { s, settles } = session();
    s.apply(edit(draft('a')));
    await s.flush();
    expect(settles.map((d) => d.name)).toEqual(['a']);
    expect(s.pending).toBe(false);
  });

  test('undo, redo and reset give the drafts back as they were, and settle them like any edit', async () => {
    const base = draft('base');
    const { s, c, settles } = session(base);
    const [a, b] = [draft('a'), draft('b')];
    s.apply(edit(a));
    s.apply(edit(b));
    expect(s.edits).toBe(2);
    expect(s.undo()).toBe(true);
    expect(s.plan).toBe(a);
    expect(s.canRedo).toBe(true);
    expect(s.redo()).toBe(true);
    expect(s.plan).toBe(b);
    s.reset();
    expect(s.plan).toBe(base);
    expect(s.edits).toBe(0);
    expect(s.canUndo).toBe(false);
    // Back where the town on screen is: nothing to settle.
    await c.advance(QUIET);
    expect(settles).toEqual([]);
    expect(s.undo()).toBe(false);
  });

  test("the Problems list is checked shortly after an edit, the laid town's own problems blocking nothing", async () => {
    const { s, c } = session();
    expect(s.issues.blocking).toEqual([]);
    s.apply(edit(draft('bad')));
    expect(s.issues.blocking).toEqual([]);
    await c.advance(CHECK_AFTER);
    expect(s.issues.blocking.map((i) => i.text)).toEqual(['Bad']);
  });

  test('a session over edits made before (back from step 3) settles the plan if the town on screen is behind it', async () => {
    const first = session();
    const a = draft('a');
    first.s.apply(edit(a));
    const c = clock();
    const settles: NeighbourhoodDraft[] = [];
    const again = new EditSession(first.s.settled, { timers: c.timers, now: c.now, settle: async (plan) => (settles.push(plan), { draft: plan, preview: {} } as unknown as Settled), show: async () => {}, changed: () => {} }, first.s.history, first.s.settledFrom);
    expect(again.plan).toBe(a);
    expect(again.edits).toBe(1);
    await c.advance(QUIET);
    expect(settles).toEqual([a]);
  });

  test('a disposed session settles nothing more', async () => {
    const { s, c, settles } = session();
    s.apply(edit(draft('a')));
    s.dispose();
    await c.advance(QUIET * 2);
    expect(settles).toEqual([]);
  });
});

describe('the Problems list and picking, on a generated town', async () => {
  const WEB = new URL('../', import.meta.url).pathname;
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
  const { AssetRegistry } = await import('../src/assets/registry');
  const { objectInfos } = await import('../src/game/housegen');
  const { randomHousehold } = await import('../src/game/household');
  const { mulberry32 } = await import('../src/game/noise');
  const { townProblems } = await import('../src/game/townCheck');
  const { assembleTown, generateWorld, plotRect, useTownContent, vacantSlots } = await import('../src/game/town');
  const { removeLotsEdit } = await import('../src/game/edit/lots');
  const { removeRoad } = await import('../src/game/edit/streets');
  const { checkedTown, townIssues } = await import('../src/ui/townedit/problems');
  const { find, keyOf, pick, spotNear } = await import('../src/ui/townedit/pick');
  const { editGhosts } = await import('../src/ui/townedit/ghosts');
  const { packUses, useGrid } = await import('../src/game/uses');
  type Templates = import('../src/game/town').Templates;
  const content = await Content.load('http://localhost/content/base.json');
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const t: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(content), lampTiles: content.lampTiles, windowTiles: content.windowTiles };
  const careers = content.careerCategories.map((c) => c.id);
  const hamlet = generateWorld(content, t, 'country', 4242, undefined, undefined, { town: 'hamlet', villages: 'few' });
  const made = (e: Edited): NeighbourhoodDraft => {
    if ('refused' in e) throw new Error(`refused: ${e.refused}`);
    return e.draft;
  };

  test("a draft's checks are the whole town file's (nobody living there yet)", () => {
    const home = vacantSlots(hamlet)[0].index;
    const file = JSON.parse(assembleTown(content, t, { ...hamlet, households: [] }, randomHousehold(content, assets, 2, [], {}, mulberry32(1)), home, undefined, mulberry32(2)));
    const whole = townProblems(file, undefined, { careers });
    const quick = townProblems(checkedTown(hamlet, t), undefined, { careers });
    expect(quick).toEqual(whole);
  });

  test("a draft's painted uses never go into its town file", () => {
    const { cols, rows } = useGrid(hamlet.width, hamlet.depth);
    const painted = packUses(new Uint8Array(cols * rows).map((_, k) => k % 6), cols, rows);
    const file = (draft: NeighbourhoodDraft) => assembleTown(content, t, { ...draft, households: [] }, randomHousehold(content, assets, 2, [], {}, mulberry32(1)), vacantSlots(hamlet)[0].index, undefined, mulberry32(2));
    const withUses = { ...hamlet, open: { ...hamlet.open!, uses: painted } };
    expect(file(withUses)).toBe(file(hamlet));
  });

  test('the laid town blocks nothing; a lot moved onto a road blocks the way on, named, with where it is', () => {
    const base = townIssues(hamlet, t, content);
    expect(base.blocking).toEqual([]);
    // A home put down across the middle of a road.
    const road = hamlet.roads.findIndex((r) => r.kind !== 'trail' && r.area === 0);
    const [x, z] = hamlet.roads[road].points[Math.floor(hamlet.roads[road].points.length / 2)];
    const slot = hamlet.slots.findIndex((s) => s.kind === 'house');
    const r = plotRect(t, hamlet.slots[slot]);
    const moved: NeighbourhoodDraft = { ...hamlet, slots: hamlet.slots.map((s, i) => (i === slot ? { ...s, x: s.x + Math.round(x - r.x - r.w / 2), z: s.z + Math.round(z - r.z - r.d / 2) } : s)) };
    const found = townIssues(moved, t, content, base);
    const onRoad = found.blocking.find((i) => /stands on a road|on a road$/.test(i.text));
    expect(onRoad?.text).toBe(`${hamlet.slots[slot].name} on a road`);
    expect(onRoad?.lot).toBe(slot);
    expect(onRoad?.at).toBeDefined();
  });

  test('fewer than two homes blocks the way on', () => {
    const homes = hamlet.slots.filter((s) => s.kind === 'house').map((s) => s.index);
    const few: NeighbourhoodDraft = { ...hamlet, slots: hamlet.slots.map((s) => (s.kind === 'house' && s.index !== homes[0] ? { ...s, kind: 'park' as const } : s)) };
    expect(townIssues(few, t, content).blocking.some((i) => i.text.startsWith('Fewer than two homes'))).toBe(true);
  });

  test('what is under a point is found on the plan, and found again after an edit moved the indices', () => {
    const slot = hamlet.slots.findIndex((s) => s.kind === 'house');
    const r = plotRect(t, hamlet.slots[slot]);
    expect(pick(hamlet, t, r.x + r.w / 2, r.z + r.d / 2)).toEqual({ kind: 'lot', slot });
    const road = hamlet.roads.findIndex((q) => q.kind === 'street');
    const [x, z] = hamlet.roads[road].points[1];
    expect(pick(hamlet, t, x, z, { roads: true })).toEqual({ kind: 'road', road });
    expect(spotNear(hamlet, x + 3, z)?.road).toBe(road);
    const village = hamlet.open!.areas[1];
    expect(pick(hamlet, t, village.x + 1, village.z + 1, { areas: true })).toEqual({ kind: 'village', area: 1 });
    // A lot before it removed: the same lot, one index down.
    const last = hamlet.slots.findLastIndex((s) => s.kind === 'house');
    const key = keyOf(hamlet, { kind: 'lot', slot: last })!;
    const fewer = made(removeLotsEdit(hamlet, new Set([slot])));
    expect(find(fewer, key)).toEqual({ kind: 'lot', slot: last - 1 });
    expect(find(fewer, keyOf(hamlet, { kind: 'lot', slot })!)).toBeNull();
  });

  test("an edit's ghosts: what goes grey, a road gone red, nothing for a rename", () => {
    const slot = hamlet.slots.findIndex((s) => s.kind === 'house');
    const fewer = made(removeLotsEdit(hamlet, new Set([slot])));
    const g = editGhosts(hamlet, fewer, t)!;
    expect(g.lots).toEqual([{ ...plotRect(t, hamlet.slots[slot]), tone: 'gone' }]);
    expect(g.roads).toEqual([]);
    // A road removed with what stands on it.
    const road = hamlet.roads.findIndex((r, i) => r.kind === 'lane' && 'draft' in removeRoad(hamlet, t, i));
    if (road >= 0) {
      const without = made(removeRoad(hamlet, t, road));
      const r = editGhosts(hamlet, without, t)!;
      expect(r.roads.some((q) => q.tone === 'bad')).toBe(true);
    }
    expect(editGhosts(hamlet, { ...hamlet, name: 'Elsewhere' }, t)).toBeNull();
  });

  // (A city's checks take seconds, which is why they run in their own worker, `checkJob.ts`: printed here to keep an eye on.)
  test('a laid city blocks nothing', () => {
    const city = generateWorld(content, t, 'country', 77, undefined, undefined, { town: 'city', villages: 'many' });
    const t0 = performance.now();
    const found = townIssues(city, t, content);
    console.log(`townIssues on a city with many villages (${city.slots.length} lots, ${city.open?.backdrop?.length ?? 0} buildings): ${(performance.now() - t0).toFixed(0)} ms`);
    expect(found.blocking).toEqual([]);
  }, 60_000);
});
