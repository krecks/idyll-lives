// Step 2's Street tool (`ui/townedit/streetTool.ts`) and its live checks (`streetLive.ts`), on a small
// hand-made town: a high street across the world with a lane off it, a river south of it and a far
// lane past the river. The live checks agree with `drawStreet` on the same strokes, reason for reason
// (a tight bend, a junction too near another, a road alongside, a crossing too wide, a footpath over
// the water), tell an unfinished street from a refused one, and tint a street up a steep slope; on a
// generated river world every live verdict is `drawStreet`'s, within the per-move budget. (The slope is
// the settle's to judge, through the town worker: `editSettle.slow.test.ts`; here only its forecast.) The tool
// itself, driven as the town stage drives it: clicks, a freehand drag, Backspace, Esc and Enter, the
// street drawn picked; the magnet; Enter carrying a street on to a road nearby; a street clearly too
// steep refused up front, its climb measured as `measureLand` measures it; the hint on bare land. The road hierarchy: an
// avenue's live checks, a one-way street drawn with arrows along it the way cars go (flipped on the card), the
// motorway not crossed. `bun test`.
import { describe, expect, mock, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';
import type { Content } from '../src/content/content';
import type { RoadPlan } from '../src/core/protocol';
import type { Edited } from '../src/game/edit/edited';
import { drawStreet } from '../src/game/edit/streets';
import { blankLand, LAND_CELL, type Land } from '../src/game/land';
import { measureLand, type FollowWorld } from '../src/game/landFollow';
import { rle } from '../src/game/landMasks';
import type { Pt } from '../src/game/roads';
import type { NeighbourhoodDraft, PlotSlot, Templates } from '../src/game/town';
import { GRADE, profileOf } from '../src/render/shared/builtSurface';
import type { TownGhosts } from '../src/render/types';
import type { Picked } from '../src/ui/townedit/pick';
import type { TownEditor } from '../src/ui/townedit/tools';

// (The card and its runes are the page's: the tool reads its choices from a plain object here.)
const choice = { kind: 'auto' as string, oneway: false, flip: false, clear: true, along: 'homes' as string, shown: 'street' as string, shownArea: -1, drawn: [] as { road: number; name: string }[] };
mock.module('../src/ui/townedit/StreetCard.svelte', () => ({ default: {} }));
mock.module('../src/ui/townedit/streetChoice.svelte', () => ({
  streetChoice: choice,
  strokeKind: (k: string) => (k === 'auto' ? 'street' : k),
  alongFor: (a: string, area: number) => (a !== 'auto' ? a : area === 0 ? 'both' : 'homes'),
  fillsHomes: (a: string) => a === 'homes',
  strokeWay: (c: { kind: string; oneway: boolean; flip: boolean }) => (c.oneway && c.kind !== 'trail' ? (c.flip ? -1 : 1) : undefined),
}));
const { climbOf, drawnStretches, liveStreet, runsOf, unfinished, verdictOf } = await import('../src/ui/townedit/streetLive');
const { arrowGhosts, askOf, AUTO_JOIN, carryOn, FIRST_STREET, kindLine, lineWay, pullAt, pullReach, StreetDrawing, startAt, streetGhosts, streetTool, stretchNear } = await import('../src/ui/townedit/streetTool');
const { CROSS_MOTORWAY, ONE_WAY_ENDS } = await import('../src/game/edit/streets');
const { roadHierarchy } = await import('../src/render/roadHierarchyFixture');

const W = 400;
const D = 400;
const MAIN_Z = 100;
const RIVER_Z = 200;
const RIVER_END = 340;
const FAR_Z = 290;
const LANE_X = 170;

const line = (x0: number, z0: number, x1: number, z1: number): Pt[] => {
  const n = Math.max(1, Math.ceil(Math.hypot(x1 - x0, z1 - z0)));
  return Array.from({ length: n + 1 }, (_, i): Pt => [x0 + ((x1 - x0) * i) / n, z0 + ((z1 - z0) * i) / n]);
};

const lot = (index: number, x: number, z: number, facing: 0 | 2, name: string): PlotSlot => ({ index, kind: 'house', template: 'house', name, x, z, facing, lot: 'medium', w: 14, d: 18 });

/** The town (as `editStreets.test.ts` has it): the high street out both ways, Mill Lane off it (a dead end), the river `river` m across, Far Lane past it (a dead end at its west end), Round Road joining the two round the river's end. */
function town(river = 30, land?: Land): NeighbourhoodDraft {
  const roads: RoadPlan[] = [
    { points: line(0, MAIN_Z, W, MAIN_Z), width: 7, ends: ['out', 'out'], kind: 'street', area: 0, name: 'High Street' },
    { points: line(LANE_X, MAIN_Z + 2.15, LANE_X, 130), width: 7, ends: ['join', 'dead'], kind: 'lane', area: 0, name: 'Mill Lane' },
    { points: line(40, FAR_Z, 380, FAR_Z), width: 7, ends: ['dead', 'join'], kind: 'country', name: 'Far Lane' },
    { points: line(380, MAIN_Z + 2.15, 380, FAR_Z), width: 7, ends: ['join', 'join'], kind: 'country', name: 'Round Road' },
  ];
  const slots: PlotSlot[] = [];
  for (let x = 60; x <= 320; x += 40) slots.push(lot(slots.length, x, MAIN_Z + 6, 0, `${slots.length * 2 + 1} High Street`));
  const water = new Uint8Array(W * D);
  for (let z = RIVER_Z; z < RIVER_Z + river; z++) for (let x = 0; x < RIVER_END; x++) water[z * W + x] = 1;
  return {
    name: 'Ashby',
    width: W,
    depth: D,
    slots,
    roads,
    greens: [],
    households: [],
    open: { areas: [{ name: 'Ashby', kind: 'hamlet', x: 50, z: 50, w: 300, d: 200 }], zones: [], water: rle(water), pads: {}, parking: [], places: [], ...(land ? { land } : {}) },
  } as unknown as NeighbourhoodDraft;
}

const t: Templates = { plot: { width: 14, depth: 18 }, houses: [], parks: [] };
const content = { names: { streets: ['Orchard Way', 'Church Road', 'Kiln Lane', 'Station Road', 'Brook Street', 'Elm Close'] } } as unknown as Content;
const refusal = (e: Edited) => ('refused' in e ? e.refused : null);

/** The live checks' verdict and `drawStreet`'s on the same stroke. */
function both(draft: NeighbourhoodDraft, points: Pt[], kind: RoadPlan['kind'] & string = 'street') {
  return { live: liveStreet(draft, t, points, { kind, clear: true }), drawn: drawStreet(draft, t, content, { points, kind, clear: true, fill: false, seed: 7 }) };
}

describe('a street being drawn', () => {
  test('points are set, taken back and dropped; the pointer is its last point', () => {
    const s = new StreetDrawing();
    expect(s.drawing).toBe(false);
    s.add([10, 10]);
    s.add([10.2, 10.1]); // (as good as the same point: not set again)
    s.add([40, 10]);
    s.cursor = [60, 20];
    expect(s.line()).toEqual([
      [10, 10],
      [40, 10],
      [60, 20],
    ]);
    expect(s.back()).toBe(true);
    expect(s.points).toEqual([[10, 10]]);
    s.clear();
    expect(s.drawing).toBe(false);
    expect(s.back()).toBe(false);
  });

  test('where a street would start: a T on the road nearby, a dead end carried on, or the open', () => {
    const draft = town();
    const off = startAt(draft, [120, MAIN_Z - 9], 'street')!;
    expect(off.road).toBe(0);
    expect(off.extend).toBe(false);
    expect(off.at[0]).toBeCloseTo(120, 0);
    expect(off.at[1]).toBeCloseTo(MAIN_Z, 1);
    const on = startAt(draft, [LANE_X + 3, 136], 'street')!;
    expect(on).toMatchObject({ road: 1, extend: true, at: [LANE_X, 130] });
    expect(startAt(draft, [250, 40], 'street')).toBeNull();
  });
});

describe('the live checks agree with drawStreet', () => {
  test('a street from the high street across the river to Far Lane: a bridge, finished, no reason against it', () => {
    const draft = town();
    const { live, drawn } = both(draft, [
      [87, MAIN_Z + 8],
      [87, FAR_Z - 8],
    ]);
    expect(live.why).toBeNull();
    expect(live.reached).toBe(true);
    expect(live.joins.map((j) => j.ok)).toEqual([true, true]);
    expect(live.bridges).toEqual([30]);
    expect(live.stretches.filter((s) => s.tone === 'bridge')).toHaveLength(1);
    expect(runsOf(live.line, live.stretches).map((r) => r.tone)).toEqual(['new', 'bridge', 'new']);
    expect(refusal(drawn)).toBeNull();
    // Once drawStreet has had its say, the ghosts are the roads it makes: the bridge pale; and Far Lane, which it
    // finishes on, green.
    expect(live.finishOn).toBe(2);
    const g = streetGhosts({ draft, t }, live, drawn);
    expect(g.roads!.map((r) => r.tone).sort()).toEqual(['bridge', 'join', 'new', 'new']);
    expect(g.roads!.find((r) => r.tone === 'join')!.points).toEqual(draft.roads[2].points);
  });

  test('over a footpath by the bank and the river into the open on the far side: a bridge, the footpath crossed, no reason against it, nothing left open', () => {
    const draft = town();
    draft.roads.push({ points: [...line(60, MAIN_Z + 3.5, 60, 192), ...line(60, 192, 200, 192).slice(1)], width: 2, ends: ['join', 'dead'], kind: 'trail', area: 0, name: 'Ashmouth Fishing Jetty Trail' });
    const { live, drawn } = both(draft, [
      [130, MAIN_Z + 8],
      [126, 190],
      [120, 265],
    ]);
    expect(live.why).toBeNull();
    expect(live.open).toBeNull();
    expect(live.bridges).toHaveLength(1);
    expect(refusal(drawn)).toBeNull();
    expect('draft' in drawn && drawn.draft.roads.filter((r) => r.name?.startsWith('Ashmouth')).length).toBe(2);
  });

  test('a crossing too wide for a bridge: red over the water, the same reason', () => {
    const { live, drawn } = both(town(80), [
      [87, MAIN_Z + 8],
      [87, FAR_Z - 8],
    ]);
    expect(live.why).not.toBeNull();
    expect(live.why).toBe(refusal(drawn));
    const bad = live.stretches.find((s) => s.tone === 'bad')!;
    // (Its stretch: the water and the straight run onto it either side.)
    expect(bad.from).toBeLessThan(RIVER_Z - MAIN_Z);
    expect(bad.to).toBeGreaterThan(RIVER_Z + 80 - MAIN_Z - 10);
  });

  test('a tight bend: red where it bends, the same reason', () => {
    const { live, drawn } = both(town(), [
      [250, MAIN_Z - 8],
      [250, 60],
      [262, 66],
      [250, 30],
    ]);
    expect(live.why).toBe('it bends tighter than 9 m');
    expect(refusal(drawn)).toBe(live.why);
    expect(live.stretches.some((s) => s.tone === 'bad' && s.to - s.from < 40)).toBe(true);
  });

  test('a junction within 14 m of another slides along the road (16 m at most) to where there is room; with none either side, its mark red, the same reason', () => {
    const slid = both(town(), [
      [LANE_X + 6, MAIN_Z - 8],
      [LANE_X + 6, 62],
    ]);
    expect(slid.live.why).toBeNull();
    expect(refusal(slid.drawn)).toBeNull();
    expect(slid.live.joins).toHaveLength(1);
    // (Off the high street 6 m from Mill Lane: slid 8 m on, 14 m from it.)
    expect(slid.live.joins[0].at[0]).toBeCloseTo(LANE_X + 14, 0);
    expect(slid.live.joins[0].at[1]).toBeCloseTo(MAIN_Z, 1);
    // (Bell Lane north of the high street 24 m from Mill Lane: 16 m either way is too near one or the other.)
    const crowded = town();
    crowded.roads.push({ points: line(LANE_X + 24, MAIN_Z - 2.15, LANE_X + 24, 70), width: 7, ends: ['join', 'dead'], kind: 'lane', area: 0, name: 'Bell Lane' });
    const { live, drawn } = both(crowded, [
      [LANE_X + 10, MAIN_Z - 8],
      [LANE_X + 10, 62],
    ]);
    expect(live.why).toMatch(/within 14 m of another junction/);
    expect(refusal(drawn)).toBe(live.why);
    expect(live.joins.some((j) => !j.ok)).toBe(true);
  });

  test('a road alongside another: red where it runs alongside, the same reason', () => {
    const { live, drawn } = both(town(), [
      [100, MAIN_Z - 30],
      [100, MAIN_Z - 12],
      [300, MAIN_Z - 12],
    ]);
    expect(live.why).not.toBeNull();
    expect(refusal(drawn)).toBe(live.why);
    expect(live.stretches.some((s) => s.tone === 'bad')).toBe(true);
  });

  test('a footpath over the water: the same reason', () => {
    const { live, drawn } = both(
      town(),
      [
        [87, MAIN_Z + 8],
        [87, FAR_Z - 8],
      ],
      'trail',
    );
    expect(live.why).toBe('a footpath does not cross the water');
    expect(refusal(drawn)).toBe(live.why);
  });

  test("a dead end longer than a court is drawn as it is; a bridge with no road beyond it is a street not finished yet: said, not red; drawStreet's refusal is of the same kind", () => {
    const long = both(town(), [
      [250, MAIN_Z - 8],
      [250, 30],
    ]);
    expect(long.live.why).toBeNull();
    expect(long.live.open).toBeNull();
    expect(refusal(long.drawn)).toBeNull();
    // (Over the river and ending 5 m past it: the bridge has no road on beyond it yet.)
    const { live, drawn } = both(town(), [
      [87, MAIN_Z + 8],
      [87, RIVER_Z + 35],
    ]);
    expect(live.why).toBeNull();
    expect(live.reached).toBe(false);
    expect(live.open).toMatch(/^a bridge needs a road on at each end/);
    expect(live.stretches.filter((x) => x.tone === 'bad')).toEqual([]);
    expect(unfinished(refusal(drawn)!)).toBe(true);
    // (A street joining no road is unfinished too; a tight bend is not.)
    expect(unfinished(refusal(drawStreet(town(), t, content, { points: [[250, 20], [250, 70]], kind: 'street', clear: true, fill: false, seed: 1 }))!)).toBe(true);
    expect(unfinished('it bends tighter than 9 m')).toBe(false);
  });

  test('a street up a steep slope is tinted where it would need more cut or fill than the settle allows, a forecast for the settle to judge; on flat land it is not', () => {
    const steep = blankLand(W, D);
    // (Rising 30 m over 40 m north of the high street: z 60 to 20.)
    for (let j = 0; j < steep.rows; j++) for (let i = 0; i < steep.cols; i++) steep.h[j * steep.cols + i] = Math.round(Math.min(30, Math.max(0, (60 - j * 2) * 0.75)) * 100);
    const up: Pt[] = [
      [250, MAIN_Z - 8],
      [250, 20],
      [250, 2],
    ];
    // (Out of the town's box: inside it the settle lays the land to the town's plane first.)
    const draft = town(30, steep);
    draft.open!.areas[0] = { ...draft.open!.areas[0], z: MAIN_Z - 5, d: 150 };
    const live = liveStreet(draft, t, up, { kind: 'street' });
    expect(live.steep).toBe(true);
    // (In the settle's own words: `landFollow.ts` `overWords`.)
    expect(live.slope).toMatch(/^the ground under it would have to be dug out or built up [\d.]+ m \(4\.0 m at most\)$/);
    expect(live.excess).toBe(true);
    expect(live.why).toBeNull();
    expect(live.stretches.some((s) => s.tone === 'bad')).toBe(true);
    const flat = liveStreet(town(30, blankLand(W, D)), t, up, { kind: 'street' });
    expect(flat.steep).toBe(false);
    expect(flat.why).toBeNull();
    // Inside the town's box the same slope is not judged.
    const inTown = town(30, steep);
    inTown.open!.areas[0] = { ...inTown.open!.areas[0], z: 0, d: 250 };
    expect(liveStreet(inTown, t, up, { kind: 'street' }).steep).toBe(false);
  });
});

/** An editor over `draft` whose ground is the screen (a screen pixel a metre), recording what the tool asks of it. */
function fakeEditor(draft: NeighbourhoodDraft) {
  const said = { hints: [] as (string | null)[], refusals: [] as string[], ghosts: null as Partial<TownGhosts> | null, selected: null as Picked | null };
  let seed = 1;
  const editor: { -readonly [K in keyof TownEditor]: TownEditor[K] } & { said: typeof said } = {
    said,
    step: 'build',
    draft,
    t,
    content,
    version: 0,
    apply(edited) {
      if ('refused' in edited) {
        said.refusals.push(edited.refused);
        return false;
      }
      editor.draft = edited.draft;
      editor.version++;
      return true;
    },
    refuse: (why) => void said.refusals.push(why),
    seed: () => seed++,
    ground: (x, y) => ({ x, z: y }),
    pick: () => null,
    get selected() {
      return said.selected;
    },
    select: (p) => void (said.selected = p),
    hover: () => {},
    preview: (g) => void (said.ghosts = g),
    hint: (text) => void said.hints.push(text),
    brush: () => {},
    showUses: () => {},
    townName: 'Ashby',
    toolId: 'street',
    take: () => {},
  };
  return editor;
}

const key = (k: string) => ({ key: k }) as KeyboardEvent;

describe('the hint names what the street will be', () => {
  test("a country road whose dead-end stretch drawStreet makes a street (past a court) says so, once drawStreet has had its say", () => {
    const draft = town();
    // (South off Far Lane, out of the town's box: a country road as the live checks have it; drawn, a street of no area.)
    const pts: Pt[] = [
      [200, FAR_Z + 6],
      [200, 380],
    ];
    const live = liveStreet(draft, t, pts, { kind: 'street', clear: true });
    expect(live.kind).toBe('country');
    expect(kindLine(draft, live)).toBe('A country road');
    expect(kindLine(draft, live, verdictOf(draft, t, content, pts, { kind: 'street', clear: true }, 7))).toBe('A street');
    // (A short one stays a country road.)
    const short: Pt[] = [
      [200, FAR_Z + 6],
      [200, FAR_Z + 40],
    ];
    expect(kindLine(draft, liveStreet(draft, t, short, { kind: 'street', clear: true }), verdictOf(draft, t, content, short, { kind: 'street', clear: true }, 7))).toBe('A country road');
    // Split at a bridge, the stretch before it a country road and the one past it made a street.
    const road = (kind: RoadPlan['kind'], bridge?: [number, number]): RoadPlan => ({ points: line(0, 0, 0, 40), width: 7, ends: ['join', 'join'], kind, ...(bridge ? { bridge } : {}) });
    const split = { draft: { ...draft, roads: [...draft.roads, road('country'), road('country', [5, 30]), road('street')] }, cleared: { lots: [] }, made: [4, 5, 6] } as Edited;
    expect(kindLine(draft, { ...live, bridges: [24] }, split)).toBe('A country road with a 24 m bridge, then a street to its dead end');
  });
});

describe('the tool', () => {
  test('clicks set points; the one on a road ends the street, which is drawn and picked', () => {
    choice.along = 'none';
    const editor = fakeEditor(town());
    const tool = streetTool.stage(editor);
    tool.hover(87, MAIN_Z + 12);
    expect(editor.said.hints.at(-1)).toMatch(/Click to start a street off High Street/);
    tool.down(87, MAIN_Z + 8);
    tool.up();
    tool.hover(87, 160);
    expect(editor.said.ghosts?.roads?.length).toBeGreaterThan(0);
    tool.down(87, 160);
    tool.up();
    // (Not on a road yet: still drawing, nothing drawn.)
    expect(editor.draft.roads).toHaveLength(4);
    tool.hover(87, FAR_Z - 3);
    expect(editor.said.hints.at(-1)).toMatch(/with a 30 m bridge\. Joins Far Lane — click to finish/);
    tool.down(87, FAR_Z - 3);
    tool.up();
    expect(editor.said.refusals).toEqual([]);
    expect(editor.draft.roads.some((r) => r.bridge)).toBe(true);
    // Split at its bridge: the stretch picked is the one nearest the last click (on to Far Lane), not the deck nor
    // the stretch before the bridge; the card lists all three.
    const stretches = drawnStretches(editor.draft, choice.drawn);
    expect(stretches.map((x) => x.deck)).toEqual([false, true, false]);
    const picked = editor.said.selected as { kind: 'road'; road: number };
    expect(picked.kind).toBe('road');
    expect(stretches.map((x) => x.road)).toContain(picked.road);
    const road = editor.draft.roads[picked.road];
    expect(road.bridge).toBeUndefined();
    expect(Math.max(...road.points.map((p) => p[1]))).toBeGreaterThan(FAR_Z - 10);
    // Nothing left being drawn: a key goes to the editor (Esc puts the tool down).
    expect(tool.key!(key('Escape'))).toBe(false);
  });

  test('the stretch picked after a split: the nearest the point given, a deck only when nothing else is near', () => {
    const r = (points: Pt[], bridge?: [number, number]): RoadPlan => ({ points, width: 7, ends: ['join', 'join'], kind: 'street', ...(bridge ? { bridge } : {}) });
    const draft = { roads: [r(line(0, 0, 0, 50)), r(line(0, 50, 0, 80), [0, 30]), r(line(0, 80, 0, 140))] } as unknown as NeighbourhoodDraft;
    expect(stretchNear(draft, [0, 1, 2], [2, 130])).toBe(2);
    expect(stretchNear(draft, [0, 1, 2], [2, 10])).toBe(0);
    // (On the deck itself, the stretch it leads onto from the nearer bank.)
    expect(stretchNear(draft, [0, 1, 2], [0, 70])).toBe(2);
    expect(stretchNear(draft, [1], [0, 70])).toBe(1);
    expect(stretchNear(draft, [], [0, 70])).toBeUndefined();
    // The card lists them while the draft has them as named; not once one is gone or renamed.
    const named = draft.roads.map((r, road) => ({ road, name: r.name ?? '' }));
    expect(drawnStretches(draft, named).map((x) => x.deck)).toEqual([false, true, false]);
    expect(drawnStretches(draft, [{ ...named[0], name: 'Other Road' }, ...named.slice(1)])).toEqual([]);
    expect(drawnStretches(draft, named.slice(0, 1))).toEqual([]);
  });

  test('Backspace takes the last point back, Esc drops the street, Enter ends it as a dead end', () => {
    const editor = fakeEditor(town());
    const tool = streetTool.stage(editor);
    for (const [x, y] of [
      [250, MAIN_Z - 8],
      [250, 75],
      [250, 60],
    ]) {
      tool.down(x, y);
      tool.up();
    }
    expect(tool.key!(key('Backspace'))).toBe(true);
    expect(tool.key!(key('Escape'))).toBe(true);
    expect(tool.key!(key('Enter'))).toBe(false);
    expect(editor.draft.roads).toHaveLength(4);
    tool.down(250, MAIN_Z - 8);
    tool.up();
    tool.down(250, 70);
    tool.up();
    expect(tool.key!(key('Enter'))).toBe(true);
    expect(editor.said.refusals).toEqual([]);
    const made = editor.draft.roads[4];
    expect(made.ends).toEqual(['join', 'dead']);
    expect(editor.said.selected).toEqual({ kind: 'road', road: 4 });
  });

  test('Enter on a bridge with no road beyond it, or a street refused, keeps what was drawn and says why', () => {
    const editor = fakeEditor(town());
    const tool = streetTool.stage(editor);
    tool.down(87, MAIN_Z + 8);
    tool.up();
    tool.down(87, RIVER_Z + 35);
    tool.up();
    tool.key!(key('Enter'));
    expect(editor.said.refusals.at(-1)).toMatch(/a bridge needs a road on at each end/);
    expect(editor.draft.roads).toHaveLength(4);
    expect(tool.key!(key('Backspace'))).toBe(true);
  });

  test('Enter ends the street where the pointer rests, as the hint says: the last click over the water, the pointer on the far bank', () => {
    choice.along = 'none';
    const editor = fakeEditor(town());
    const tool = streetTool.stage(editor);
    tool.down(87, MAIN_Z + 8);
    tool.up();
    tool.down(87, RIVER_Z + 15);
    tool.up();
    // (Ended at the last click, it would be a bridge with no road beyond it.)
    tool.hover(87, RIVER_Z + 55);
    expect(editor.said.hints.at(-1)).toMatch(/Enter or a double click ends it here as a dead end/);
    tool.key!(key('Enter'));
    expect(editor.said.refusals).toEqual([]);
    const made = editor.draft.roads.slice(4);
    expect(made.some((r) => r.bridge)).toBe(true);
    const far = made.find((r) => r.ends.includes('dead'))!;
    expect(far.points.at(-1)![1]).toBeCloseTo(RIVER_Z + 55, 0);
  });

  test('a freehand drag from a road to another draws the street when let go', () => {
    const editor = fakeEditor(town());
    const tool = streetTool.stage(editor);
    tool.down(87, MAIN_Z - 8);
    for (let y = MAIN_Z - 10; y >= 40; y -= 3) tool.move(87 + (MAIN_Z - y) * 0.3, y);
    tool.move(LANE_X + 0.5, 40);
    // (The lane runs only to z 130, south of the high street: this one ends in the open.)
    tool.up();
    expect(editor.draft.roads).toHaveLength(4);
    tool.key!(key('Escape'));
    // From the high street south over the river to Far Lane, drawn in one drag.
    tool.down(87, MAIN_Z + 8);
    for (let y = MAIN_Z + 12; y <= FAR_Z - 6; y += 3) tool.move(87, y);
    tool.up();
    expect(editor.said.refusals).toEqual([]);
    expect(editor.draft.roads.some((r) => r.bridge)).toBe(true);
    expect(editor.said.selected?.kind).toBe('road');
  });
});

/** The town with Hill Road east-west at z 140 (joining Round Road), the land 12 m up from z 125 north of the high street, and the town's box moved west out of the way. */
function hillTown(): NeighbourhoodDraft {
  const land = blankLand(W, D);
  for (let j = 0; j < land.rows; j++) for (let i = 0; i < land.cols; i++) land.h[j * land.cols + i] = j * LAND_CELL >= 125 ? 1200 : 0;
  const draft = town(30, land);
  draft.roads.push({ points: line(200, 140, 380, 140), width: 7, ends: ['dead', 'join'], kind: 'country', name: 'Hill Road' });
  draft.open!.areas[0] = { ...draft.open!.areas[0], x: 50, z: 20, w: 150, d: 60 };
  return draft;
}

describe('the climb, as measureLand measures it', () => {
  test("a road's steepest climb from its profile is measureLand's, over a gentle slope and a steep one, a street and a country road", () => {
    for (const rise of [0.05, 0.1, 0.2]) {
      for (const kind of ['street', 'country'] as const) {
        const land = blankLand(W, D);
        for (let j = 0; j < land.rows; j++) for (let i = 0; i < land.cols; i++) land.h[j * land.cols + i] = Math.round(Math.max(0, Math.min(120, i * LAND_CELL) - 100) * rise * 100);
        const points = Array.from({ length: 151 }, (_, k): Pt => [60 + k, 200 + Math.sin(k / 20) * 10]);
        const world = { width: W, depth: D, plots: [], meta: { roads: [{ points, width: 7, ends: ['dead', 'dead'], kind }] } } as unknown as FollowWorld;
        const measured = measureLand(world, land, land, 0, false);
        const live = climbOf(profileOf(points, kind, land), kind);
        expect(live.steepest).toBeCloseTo(measured.slopes.grade[kind], 6);
        expect(live.over.length > 0).toBe(measured.over.roads.has(0));
        expect(live.steepest).toBeLessThanOrEqual(GRADE[kind] + 1e-4);
      }
    }
  });

  test("a street between two roads at different heights: red, said in the settle's words, clearly too steep; the tool refuses it", () => {
    const draft = hillTown();
    const up: Pt[] = [
      [250, MAIN_Z - 8],
      [250, 142],
    ];
    const live = liveStreet(draft, t, up, { kind: 'street' });
    expect(live.why).toBeNull();
    expect(live.reached).toBe(true);
    expect(live.kind).toBe('country');
    expect(live.climb).toBeGreaterThan(GRADE.country + 0.01 + 0.03);
    expect(live.slope).toMatch(/^it would climb \d+ %, more than a country may \(12 %\)$/);
    expect(live.excess).toBe(true);
    expect(live.stretches.some((x) => x.tone === 'bad')).toBe(true);
    // The same street on flat land is not.
    const flat = hillTown();
    flat.open!.land = blankLand(W, D);
    expect(liveStreet(flat, t, up, { kind: 'street' })).toMatchObject({ steep: false, excess: false, slope: null });
    // The tool refuses it when it reaches Hill Road, and keeps drawing.
    const editor = fakeEditor(draft);
    const tool = streetTool.stage(editor);
    tool.down(250, MAIN_Z - 8);
    tool.up();
    tool.hover(250, 142);
    expect(editor.said.hints.at(-1)).toMatch(/^It would climb \d+ %.*Draw it another way/);
    tool.down(250, 142);
    tool.up();
    expect(editor.said.refusals.at(-1)).toBe(live.slope!);
    expect(editor.draft.roads).toHaveLength(5);
    expect(tool.key!(key('Backspace'))).toBe(true);
  });
});

describe('the magnet, Enter carrying a street on, the kind asked for, bare land', () => {
  test('the pointer is pulled to a road, a dead end or the edge within 28 px (10 to 40 m), not back to the road it starts on near its start', () => {
    expect(pullReach(1)).toBe(28);
    expect(pullReach(0.1)).toBe(10);
    expect(pullReach(5)).toBe(40);
    const draft = town();
    const pulled = pullAt(draft, [120, 80], 28, 'street');
    expect(pulled.pull).toMatchObject({ kind: 'road', road: 0 });
    expect(pulled.at[1]).toBeCloseTo(MAIN_Z - 5.5, 6);
    expect(pullAt(draft, [120, 80], 10, 'street')).toEqual({ at: [120, 80], pull: null });
    // (A street begun by the high street at x 120: not pulled back onto it within 30 m, but further along.)
    expect(pullAt(draft, [125, 75], 28, 'street', [120, 92]).pull).toBeNull();
    expect(pullAt(draft, [160, 75], 28, 'street', [120, 92]).pull?.road).toBe(0);
  });

  test('a click 20 m off a road starts the street on it; with the ground ten times nearer (zoomed in), not', () => {
    choice.along = 'none';
    const editor = fakeEditor(town());
    const tool = streetTool.stage(editor);
    tool.hover(120, 80);
    expect(editor.said.hints.at(-1)).toMatch(/Click to start a street off High Street/);
    tool.down(120, 80);
    tool.up();
    tool.down(120, 62);
    tool.up();
    tool.key!(key('Enter'));
    expect(editor.said.refusals).toEqual([]);
    expect(editor.draft.roads[4].ends).toEqual(['join', 'dead']);
    // (The editor's own scale: 0.1 m a px, so the magnet reaches its least, 10 m.)
    const near = Object.assign(fakeEditor(town()), { scale: () => 0.1 });
    const zoomed = streetTool.stage(near);
    zoomed.hover(120, 80);
    expect(near.said.hints.at(-1)).toMatch(/Click on a road to start a street off it/);
  });

  test('a corner set near a road stays where it is set when the street pulled there would be refused', () => {
    choice.along = 'none';
    const editor = fakeEditor(town());
    const tool = streetTool.stage(editor);
    tool.down(100, MAIN_Z - 8);
    tool.up();
    // (25 m north of the high street, 60 m along from where the street starts: pulled down onto it, the street would
    // run alongside it; so it stays a corner.)
    tool.down(160, 75);
    tool.up();
    expect(editor.draft.roads).toHaveLength(4);
    expect(editor.said.refusals).toEqual([]);
    // (The line as drawn ends at the corner, not on the high street.)
    const ends = editor.said.ghosts!.roads!.map((r) => r.points[r.points.length - 1]);
    expect(ends.some(([x, z]) => Math.hypot(x - 160, z - 75) < 1)).toBe(true);
  });

  test(`Enter carries a street ended short of a road on to it, within ${AUTO_JOIN} m; past that it ends as a dead end`, async () => {
    choice.along = 'none';
    const ask = { kind: 'street' as const, asked: false, clear: true };
    // (From Far Lane north towards the high street, ended 20 m short of its middle.)
    const pts: Pt[] = [
      [350, FAR_Z],
      [350, 120],
    ];
    const on = carryOn(town(), t, pts, ask)!;
    expect(on.points.at(-1)![1]).toBeCloseTo(MAIN_Z, 0);
    expect(on.live.finishOn).toBe(0);
    expect(carryOn(town(), t, [[350, FAR_Z], [350, 150]], ask)).toBeNull();
    // The tool, zoomed in so the click is not pulled there itself.
    const editor = Object.assign(fakeEditor(town()), { scale: () => 0.1 });
    const tool = streetTool.stage(editor);
    for (const [x, y] of pts) {
      tool.down(x, y);
      tool.up();
    }
    expect(editor.draft.roads).toHaveLength(4);
    // (Once the pointer rests there, the hint says what Enter does: carries it on.)
    tool.hover(pts[1][0], pts[1][1]);
    await new Promise((r) => setTimeout(r, 300));
    expect(editor.said.hints.at(-1)).toMatch(/Enter or a double click carries it on to High Street just ahead\.$/);
    tool.key!(key('Enter'));
    expect(editor.said.refusals).toEqual([]);
    const made = editor.draft.roads.slice(4);
    expect(made.length).toBeGreaterThan(0);
    expect(made.every((r) => r.ends.every((e) => e === 'join'))).toBe(true);
  });

  test('a street asked for on the card stays a street out of town, with no area', () => {
    choice.along = 'none';
    const draft = hillTown();
    draft.open!.land = blankLand(W, D);
    const editor = fakeEditor(draft);
    const tool = streetTool.stage(editor);
    choice.kind = 'street';
    try {
      tool.down(250, MAIN_Z - 8);
      tool.up();
      tool.down(250, 142);
      tool.up();
    } finally {
      choice.kind = 'auto';
    }
    expect(editor.said.refusals).toEqual([]);
    const road = editor.draft.roads[5];
    expect(road.kind).toBe('street');
    expect(road.area).toBeUndefined();
  });

  test('on bare land (after Clear all) the hint says the first street runs from road to road', () => {
    const bare = { ...town(), roads: [], slots: [] };
    const editor = fakeEditor(bare);
    const tool = streetTool.stage(editor);
    tool.hover(200, 150);
    expect(editor.said.hints.at(-1)).toBe(FIRST_STREET);
    tool.leave();
    expect(editor.said.hints.at(-1)).toBeNull();
    // From the edge to the edge: drawn, a way out at both ends.
    tool.down(5, 150);
    tool.up();
    tool.down(395, 150);
    tool.up();
    expect(editor.said.refusals).toEqual([]);
    expect(editor.draft.roads.map((r) => r.ends)).toEqual([['out', 'out']]);
  });
});

// On a generated river world: random strokes in and round the main town, each judged by the live
// checks and by drawStreet. Every reason the live checks give (the slope is not one) is
// drawStreet's own; drawStreet refuses nothing the live checks pass but what they leave to it.
describe('on a generated world', async () => {
  usePublicFetch();
  const { generateWorld, useTownContent } = await import('../src/game/town');
  const { mulberry32 } = await import('../src/game/noise');
  const { townIssues } = await import('../src/ui/townedit/problems');
  const real = await loadContent();
  useTownContent(real);
  const templates = await templatesOf(real);
  const world = generateWorld(real, templates, 'river', 4242, undefined, undefined, { town: 'village', villages: 'few' });

  test('every live verdict is drawStreet’s, within the per-move budget', () => {
    const a0 = world.open!.areas[0];
    const rand = mulberry32(9);
    let [agreed, refused, bridged, cut, checked] = [0, 0, 0, 0, 0];
    const left: string[] = [];
    const blocking: string[] = [];
    const base = townIssues(world, templates, real);
    const times: number[] = [];
    for (let k = 0; k < 400; k++) {
      const pts: Pt[] = [];
      let [x, z] = [a0.x - 60 + rand() * (a0.w + 120), a0.z - 60 + rand() * (a0.d + 120)];
      let way = rand() * Math.PI * 2;
      for (let i = 2 + Math.floor(rand() * 3); i > 0; i--) {
        pts.push([x, z]);
        way += (rand() - 0.5) * 1.6;
        const l = 20 + rand() * 80;
        [x, z] = [x + Math.cos(way) * l, z + Math.sin(way) * l];
      }
      const kind = rand() < 0.15 ? 'trail' : 'street';
      const t0 = performance.now();
      const live = liveStreet(world, templates, pts, { kind });
      times.push(performance.now() - t0);
      const verdict = verdictOf(world, templates, real, pts, { kind, clear: true }, 1);
      const why = refusal(verdict);
      const liveWhy = live.why;
      // (What it makes blocks nothing in the Problems list the town didn't have: every street bridged or cutting a footpath, and one in eight of the rest.)
      const crossed = 'draft' in verdict && verdict.draft.roads.some((r, i) => r.kind === 'trail' && world.roads[i] !== r);
      if (crossed) cut++;
      if ('draft' in verdict && (k % 8 === 0 || crossed || verdict.draft.roads.some((r, i) => r.bridge && world.roads[i] !== r))) {
        checked++;
        const fresh = townIssues(verdict.draft, templates, real, base).blocking.map((i) => i.text);
        if (fresh.length) blocking.push(`${JSON.stringify(pts.map(([x, z]) => [Math.round(x), Math.round(z)]))} ${kind}: ${fresh.join('; ')}`);
      }
      if (live.bridges.length) bridged++;
      if (liveWhy) {
        refused++;
        expect(why).toBe(liveWhy);
        agreed++;
      } else if (why && !unfinished(why)) left.push(why);
      else agreed++;
    }
    times.sort((a, b) => a - b);
    const median = times[times.length >> 1];
    const p95 = times[Math.floor(times.length * 0.95)];
    console.log(`streetTool: ${agreed} of 400 strokes judged alike (${refused} refused alike, ${bridged} bridged, ${cut} cutting a footpath; ${checked} drawn checked for problems); left to drawStreet: ${left.length ? left.join('; ') : 'none'}; live checks ${median.toFixed(2)} ms median, ${p95.toFixed(2)} ms p95`);
    // (What the live checks leave to drawStreet: the network.)
    for (const why of left) expect(why).toMatch(/joins none/);
    expect(agreed + left.length).toBe(400);
    expect(blocking).toEqual([]);
    expect(median).toBeLessThan(5);
  });
});

describe('avenues, one-way streets and the motorway', () => {
  test('an avenue: the live checks agree with drawStreet, 14 m wide', () => {
    const draft = town();
    const ok: Pt[] = [[240, MAIN_Z + 8], [240, 150], [200, 150], [100, 150]];
    const live = liveStreet(draft, t, ok, { kind: 'avenue', clear: true, asked: true });
    expect(live.why).toBeNull();
    expect(live.width).toBe(14);
    expect(live.kind).toBe('avenue');
    const made = verdictOf(draft, t, content, ok, { kind: 'avenue', clear: true, asked: true }, 7);
    if ('refused' in made) throw new Error(made.refused);
    expect(made.draft.roads.at(-1)!.width).toBe(14);
    // (10 m past Mill Lane's dead end: a street passes, an avenue touches it.)
    const near: Pt[] = [[240, MAIN_Z + 8], [240, 140], [200, 140], [100, 140]];
    expect(liveStreet(draft, t, near, { kind: 'street', clear: true, asked: true }).finishOn).toBe(-1);
    expect(liveStreet(draft, t, near, { kind: 'avenue', clear: true, asked: true }).finishOn).toBe(1);
  });

  test("the card's one-way and flip: the way asked; never a footpath's", () => {
    try {
      choice.oneway = true;
      expect(askOf().oneway).toBe(1);
      choice.flip = true;
      expect(askOf().oneway).toBe(-1);
      choice.kind = 'trail';
      expect(askOf().oneway).toBeUndefined();
    } finally {
      [choice.oneway, choice.flip, choice.kind] = [false, false, 'auto'];
    }
    expect(askOf().oneway).toBeUndefined();
  });

  test('arrows along a one-way line, the way cars go', () => {
    const l = line(0, 50, 100, 50);
    expect(arrowGhosts(l, 0)).toEqual([]);
    const ahead = arrowGhosts(l, 1);
    expect(ahead.length).toBe(5);
    // (Each a chevron: its tip, the middle point, ahead of its arms.)
    for (const a of ahead) expect(a.points[1][0]).toBeGreaterThan(a.points[0][0]);
    for (const a of arrowGhosts(l, -1)) expect(a.points[1][0]).toBeLessThan(a.points[0][0]);
    expect(lineWay(l, 1, [100, 50])).toBe(-1);
    expect(lineWay(l, -1, [0, 50])).toBe(-1);
  });

  test('a one-way street drawn: open till it ends on a road, arrows along it; drawn one-way the way the card says', () => {
    choice.along = 'none';
    for (const flip of [false, true]) {
      const editor = fakeEditor(town());
      const tool = streetTool.stage(editor);
      [choice.oneway, choice.flip] = [true, flip];
      try {
        tool.down(300, MAIN_Z + 8);
        tool.up();
        tool.down(300, 160);
        tool.up();
        tool.hover(340, 160);
        expect(editor.said.hints.at(-1)).toContain('A one-way street ends on a road at both ends');
        const arrows = (editor.said.ghosts?.roads ?? []).filter((g) => g.width < 1);
        expect(arrows.length).toBeGreaterThan(0);
        // (Up from the high street, then east: the last arrow points east, or west flipped.)
        const last = arrows.at(-1)!;
        expect(Math.sign(last.points[1][0] - last.points[0][0])).toBe(flip ? -1 : 1);
        // Enter here: a one-way street may not end in the open.
        expect(tool.key!(key('Enter'))).toBe(true);
        expect(editor.said.refusals.at(-1)).toBe(ONE_WAY_ENDS);
        tool.down(376, 160);
        tool.up();
      } finally {
        [choice.oneway, choice.flip] = [false, false];
      }
      const road = editor.draft.roads.at(-1)!;
      expect(road.ends).toEqual(['join', 'join']);
      expect(road.oneway).toBe(flip ? -1 : 1);
      expect(road.points[0][1]).toBeLessThan(road.points.at(-1)![1]);
    }
    expect(unfinished(ONE_WAY_ENDS)).toBe(true);
  });

  test('a street drawn across the motorway: refused live, as drawStreet refuses it', () => {
    const { width, depth, roads } = roadHierarchy();
    const draft = { name: 'Junction', width, depth, slots: [], roads, greens: [], households: [] } as unknown as NeighbourhoodDraft;
    const pts: Pt[] = [[250, 92], [250, 260]];
    expect(liveStreet(draft, t, pts, { kind: 'street' }).why).toBe(CROSS_MOTORWAY);
    expect(refusal(verdictOf(draft, t, content, pts, { kind: 'street', clear: true }, 7))).toBe(CROSS_MOTORWAY);
    expect(startAt(draft, [250, 202], 'street')).toBeNull();
  });
});
