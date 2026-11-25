// Step 2's town editor as state (`ui/townedit/`): an edit taken at once and the town settled once
// edits pause (one settle at a time, the latest plan), undo, redo and reset; what a settle refused
// shown (red for a while, a line, a Problems entry); a settle that fails taking its edits back, so no
// ghost stays; a town settled but not shown still recorded; the Problems list read
// from a draft as the checks read a whole town file (`problems.ts` `checkedTown`), sorted into what
// blocks the way on and what only warns; what is picked found again after an edit; the edits' ghosts.
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';
import type { RoadPlan } from '../src/core/protocol';
import type { Edited } from '../src/game/edit/edited';
import type { NeighbourhoodDraft } from '../src/game/town';
import type { UseLayer } from '../src/game/uses';
import { editGhosts as ghostsOf } from '../src/ui/townedit/ghosts';
import { EditSession, CHECK_AFTER, QUIET, QUIET_ONE, REFUSED_SHOW, SETTLE_FAILED, type Timers } from '../src/ui/townedit/session';
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
const edit = (d: NeighbourhoodDraft): Edited => ({ draft: d, cleared: { lots: [] } });

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
  test("an edit of the zones alone isn't settled: the town on screen takes the zones, and its undo too; one of the land after it is", async () => {
    const layer = (n: number): UseLayer => ({ v: 1, cols: 1, rows: 1, cells: [1, n] });
    const base = { name: 'base', slots: [], roads: [], open: { areas: [], uses: layer(0) } } as unknown as NeighbourhoodDraft;
    const { s, c, settles, shown } = session(base);
    const zoned = { ...base, open: { ...base.open!, uses: layer(1) } };
    s.apply({ ...edit(zoned), needsSettle: false } as unknown as Edited);
    await c.advance(QUIET * 2);
    expect(settles).toEqual([]);
    expect(shown).toEqual([]);
    expect(s.pending).toBe(false);
    expect(s.settled.open?.uses).toBe(zoned.open!.uses);
    s.undo();
    await c.advance(QUIET * 2);
    expect(settles).toEqual([]);
    expect(s.settled.open?.uses).toBe(base.open!.uses);
    // (A road drawn after it: settled as ever, zones and all.)
    const road = { ...zoned, roads: [{ points: [], width: 7, ends: ['out', 'out'] }] } as unknown as NeighbourhoodDraft;
    s.apply(edit(road));
    await c.advance(QUIET * 2);
    expect(settles).toEqual([road]);
  });

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

  test('a settle that fails takes every edit since the town on screen back, so no ghost of them stays; tried again only after the next edit', async () => {
    const c = clock();
    const base = draft('base');
    // (Each edit a street more, so the plan has ghosts over the town on screen.)
    const street = (x: number): RoadPlan => ({ points: [[x, 0], [x, 50]], width: 7, ends: ['join', 'dead'], kind: 'street' });
    const a = { ...draft('a'), roads: [street(10)] } as NeighbourhoodDraft;
    const b = { ...draft('b'), roads: [street(10), street(40)] } as NeighbourhoodDraft;
    const told: string[] = [];
    let changed = 0;
    let fail = true;
    const settles: NeighbourhoodDraft[] = [];
    const s = new EditSession(base, {
      timers: c.timers,
      now: c.now,
      settle: async (plan) => {
        settles.push(plan);
        if (fail) throw new Error('the worker fell over');
        return { draft: plan, preview: {} } as unknown as Settled;
      },
      show: async () => {},
      changed: () => void changed++,
      report: (why) => void told.push(why),
    });
    const t0 = {} as Parameters<typeof ghostsOf>[2];
    s.apply(edit(a));
    s.apply(edit(b));
    expect(ghostsOf(s.settled, s.plan, t0)).not.toBeNull();
    const before = changed;
    await c.advance(QUIET);
    expect(settles).toEqual([b]);
    expect(told).toEqual([SETTLE_FAILED]);
    expect(s.plan).toBe(base);
    expect(s.settledFrom).toBe(base);
    expect(s.pending).toBe(false);
    expect(s.edits).toBe(0);
    expect(s.canRedo).toBe(false);
    expect(changed).toBeGreaterThan(before);
    // (The town on screen and the plan are one again: no ghosts.)
    expect(ghostsOf(s.settled, s.plan, t0)).toBeNull();
    expect(s.refusalLine).toMatch(/^The town could not be built again with your changes, so they were taken back\.$/);
    expect(s.refusalGhosts()).toBeNull();
    await c.advance(QUIET * 3);
    expect(settles).toEqual([b]);
    // The next edit settles again, and clears the line.
    fail = false;
    s.apply(edit(a));
    expect(s.refusalLine).toBeNull();
    await c.advance(QUIET);
    expect(s.settledFrom).toBe(a);
    expect(s.pending).toBe(false);
  });

  test('a town settled well but not put on screen is still the one settled', async () => {
    const c = clock();
    const base = draft('base');
    const a = draft('a');
    const settledA = draft('settled a');
    const s = new EditSession(base, {
      timers: c.timers,
      now: c.now,
      settle: async () => ({ draft: settledA, preview: {}, marks: ['m'] }) as unknown as Settled,
      show: async () => {
        throw new Error('the renderer fell over');
      },
      changed: () => {},
    });
    s.apply(edit(a));
    await c.advance(QUIET);
    expect(s.settled).toBe(settledA);
    expect(s.settledFrom).toBe(a);
    expect(s.pending).toBe(false);
    expect(s.plan).toBe(a);
  });

  test('what a settle refused: red over the town for a few seconds, a line and a Problems entry until the next edit', async () => {
    const c = clock();
    const base = draft('base');
    const [a, b] = [draft('a'), draft('b')];
    let changed = 0;
    const road: [number, number][] = [
      [10, 10],
      [30, 10],
      [50, 10],
    ];
    const s = new EditSession(base, {
      timers: c.timers,
      now: c.now,
      settle: async () =>
        ({
          draft: a,
          preview: {},
          refused: [
            { op: 1, why: 'Mill Lane would climb 21 %, more than a street may (8 %)', at: road, width: 7 },
            { op: 2, why: 'the ground under 3 Mill Lane would have to be dug out or built up 5.0 m (4.0 m at most)', at: [[0, 0], [10, 20]] },
          ],
          step: 1,
        }) as unknown as Settled,
      show: async () => {},
      changed: () => void changed++,
    });
    s.apply(edit(a));
    s.apply(edit(b));
    await c.advance(QUIET);
    expect(s.plan).toBe(a);
    expect(s.refusalLine).toBe('Mill Lane would climb 21 %, more than a street may (8 %). The change was taken back.');
    expect(s.refusalGhosts()).toEqual({ roads: [{ points: road, width: 7, tone: 'bad' }], lots: [{ x: 0, z: 0, w: 10, d: 20, tone: 'bad' }], boxes: [] });
    expect(s.refusalIssues()).toEqual([
      { text: 'Mill Lane would climb 21 %, more than a street may (8 %). The change was taken back.', at: { x: 30, z: 10 } },
      { text: 'The ground under 3 Mill Lane would have to be dug out or built up 5.0 m (4.0 m at most). The change was taken back.', at: { x: 5, z: 10 } },
    ]);
    // The red goes after a few seconds (the editor told, to draw again); the line and the entry stay.
    const before = changed;
    await c.advance(REFUSED_SHOW);
    expect(changed).toBeGreaterThan(before);
    expect(s.refusalGhosts()).toBeNull();
    expect(s.refusalLine).not.toBeNull();
    // The next edit clears them.
    s.apply(edit(b));
    expect(s.refusalLine).toBeNull();
    expect(s.refusalIssues()).toEqual([]);
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
  usePublicFetch();
  const { AssetRegistry } = await import('../src/assets/registry');
  const { randomHousehold } = await import('../src/game/household');
  const { mulberry32 } = await import('../src/game/noise');
  const { townProblems } = await import('../src/game/townCheck');
  const { assembleTown, generateWorld, homesOn, plotRect, useTownContent, vacantSlots } = await import('../src/game/town');
  const { removeLotsEdit } = await import('../src/game/edit/lots');
  const { removeRoad } = await import('../src/game/edit/streets');
  const { checkedTown, townIssues } = await import('../src/ui/townedit/problems');
  const { find, keyOf, pick, spotNear } = await import('../src/ui/townedit/pick');
  const { editGhosts } = await import('../src/ui/townedit/ghosts');
  const { packUses, useGrid } = await import('../src/game/uses');
  const content = await loadContent();
  const assets = await AssetRegistry.load('http://localhost/assets/manifest.json', []);
  useTownContent(content);
  const t = await templatesOf(content);
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

  test("step 2's layout (no lots, its homes zones) isn't blocked for want of homes; a town with none is", () => {
    const { cols, rows } = useGrid(hamlet.width, hamlet.depth);
    const bare = { ...hamlet, slots: [], open: { ...hamlet.open!, pads: {}, places: [], parking: [], carParks: [] } };
    const layout = { ...bare, open: { ...bare.open, uses: packUses(new Uint8Array(cols * rows), cols, rows) } };
    expect(townIssues(layout, t, content).blocking.map((i) => i.text)).not.toContainEqual(expect.stringMatching(/fewer than two homes/i));
    expect(townIssues(bare, t, content).blocking.map((i) => i.text)).toContainEqual(expect.stringMatching(/fewer than two homes/i));
    // (Nor does it list what only lots can mend: its villages' lots, the workplaces.)
    const all = (i: ReturnType<typeof townIssues>) => [...i.blocking, ...i.warnings].map((x) => x.text);
    expect(hamlet.open!.areas.length).toBeGreaterThan(1);
    expect(all(townIssues(bare, t, content))).toContainEqual(expect.stringMatching(/has 0 lots/));
    expect(all(townIssues(layout, t, content))).not.toContainEqual(expect.stringMatching(/ lots$|workers work/));
  });

  test('a street made one-way that leaves its homes unreachable blocks the way on, with how many', async () => {
    const { entriesOf } = await import('../src/game/edit/ground');
    const spots = entriesOf(hamlet, t);
    // (A street with homes on it and a dead end: one-way, nobody gets back out of it.)
    const road = hamlet.roads.findIndex((r, i) => r.kind !== 'trail' && r.ends.includes('dead') && spots.some((s) => s?.road === i));
    expect(road).toBeGreaterThanOrEqual(0);
    const homes = hamlet.slots.reduce((n, s, i) => n + (spots[i]?.road === road ? homesOn(s, { farms: true }) : 0), 0);
    const oneWay = { ...hamlet, roads: hamlet.roads.map((r, i) => (i === road ? { ...r, oneway: 1 as const } : r)) };
    const blocking = townIssues(oneWay, t, content).blocking.map((i) => i.text);
    expect(blocking).toContainEqual(expect.stringMatching(/^One-way streets leave \d+ homes? unreachable$/));
    expect(Number(/leave (\d+)/.exec(blocking.find((b) => b.startsWith('One-way'))!)![1])).toBeGreaterThanOrEqual(homes);
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
    // (Every other home a park: a building of flats too, and no flats over a place.)
    const few: NeighbourhoodDraft = { ...hamlet, slots: hamlet.slots.map(({ upper: _upper, ...s }) => ((s.kind === 'house' || s.kind === 'flats') && s.index !== homes[0] ? { ...s, kind: 'park' as const } : s)) };
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
    // A road drawn shows as the road it will be at once (laid), its deck as a bridge.
    const drawn = { points: [[10, 10], [60, 10]] as [number, number][], width: 7, ends: ['join', 'dead'] as RoadPlan['ends'], kind: 'street' as const };
    const added = editGhosts(hamlet, { ...hamlet, roads: [...hamlet.roads, drawn, { ...drawn, points: [[60, 10], [90, 10]], bridge: [0, 30] as [number, number] }] }, t)!;
    expect(added.roads.map((r) => r.tone)).toEqual(['laid', 'bridge']);
  });

  test("the Lot tool's building of flats: drawn as the generator draws one where it goes, named by its template, its flats homes priced as flats, blocking nothing", async () => {
    const { addLot, canPlace } = await import('../src/game/edit/lots');
    const { flatsInId, flatsTemplate } = await import('../src/game/flats');
    const { homesOf } = await import('../src/game/people');
    const { flatPrice } = await import('../src/game/price');
    const { lengths } = await import('../src/game/roads');
    const choice = { kind: 'flats' } as const;
    const spots = hamlet.roads.flatMap((r, road) => (r.kind === 'trail' ? [] : Array.from({ length: Math.floor(lengths(r.points).at(-1)! / 3) }, (_, k) => [1, -1].map((side) => ({ road, s: k * 3, side: side as 1 | -1 }))).flat()));
    const at = spots.find((p) => !('refused' in canPlace(hamlet, t, p, choice, 11)))!;
    expect(at).toBeDefined();
    const fit = canPlace(hamlet, t, at, choice, 11) as { x: number; z: number; w: number; d: number; about?: string };
    const town = made(addLot(hamlet, t, content, at, choice, 11));
    const s = town.slots.at(-1)!;
    const b = flatsTemplate(t, s.template)!;
    expect(s.kind).toBe('flats');
    expect(plotRect(t, s)).toEqual({ x: fit.x, z: fit.z, w: fit.w, d: fit.d });
    expect(s.name).toBe(b.name);
    expect(fit.about).toBe(b.description);
    // Every flat a home of its own, at a flat's price; the building no place.
    const flats = homesOf(content, t, town).filter((h) => h.slot.index === s.index);
    expect(flats.length).toBe(b.units.length);
    expect(flats.length).toBe(flatsInId(s.template));
    expect(flats.every((h, k) => h.flat === k && h.price === flatPrice(content, t, s, k) && h.price > 0)).toBe(true);
    expect(town.open!.places!.some((p) => p.plot === s.index)).toBe(false);
    // (The same seed, the same building.)
    expect(made(addLot(hamlet, t, content, at, choice, 11)).slots.at(-1)!.template).toBe(s.template);
    expect(townIssues(town, t, content).blocking).toEqual([]);
  });

  // (A city's checks take seconds, which is why they run in their own worker, `checkJob.ts`: printed here to keep an eye on.)
  test('a laid city blocks nothing', () => {
    const city = generateWorld(content, t, 'country', 77, undefined, undefined, { town: 'city', villages: 'many' });
    const t0 = performance.now();
    const found = townIssues(city, t, content);
    console.log(`townIssues on a city with many villages (${city.slots.length} lots, ${city.open?.carParks?.length ?? 0} car parks): ${(performance.now() - t0).toFixed(0)} ms`);
    expect(found.blocking).toEqual([]);
  }, 60_000);
});
