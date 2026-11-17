// Step 2's Street tool (`ui/townedit/streetTool.ts`) and its live checks (`streetLive.ts`), on a small
// hand-made town: a high street across the world with a lane off it, a river south of it and a far
// lane past the river. The live checks agree with `drawStreet` on the same strokes, reason for reason
// (a tight bend, a junction too near another, a road alongside, a crossing too wide, a footpath over
// the water), tell an unfinished street from a refused one, and tint a street up a steep slope; on a
// generated river world every live verdict is `drawStreet`'s, within the per-move budget. (The slope is
// the settle's to judge, through the town worker: `editSettle.slow.test.ts`; here only its forecast.) The tool
// itself, driven as the town stage drives it: clicks, a freehand drag, Backspace, Esc and Enter, the
// street drawn picked. `bun test`.
import { afterAll, describe, expect, mock, test } from 'bun:test';
import { existsSync, readFileSync } from 'node:fs';
import type { Content } from '../src/content/content';
import type { RoadPlan } from '../src/core/protocol';
import type { Edited } from '../src/game/edit/edited';
import { drawStreet } from '../src/game/edit/streets';
import { blankLand, type Land } from '../src/game/land';
import { rle } from '../src/game/landMasks';
import type { Pt } from '../src/game/roads';
import type { NeighbourhoodDraft, PlotSlot, Templates } from '../src/game/town';
import type { TownGhosts } from '../src/render/types';
import type { Picked } from '../src/ui/townedit/pick';
import type { TownEditor } from '../src/ui/townedit/tools';

// (The card and its runes are the page's: the tool reads its choices from a plain object here.)
const choice = { kind: 'auto' as string, clear: true, along: 'homes' as string, shown: 'street' as string, shownArea: -1, drawn: [] as { road: number; name: string }[] };
mock.module('../src/ui/townedit/StreetCard.svelte', () => ({ default: {} }));
mock.module('../src/ui/townedit/streetChoice.svelte', () => ({
  streetChoice: choice,
  strokeKind: (k: string) => (k === 'auto' ? 'street' : k),
  alongFor: (a: string, area: number) => (a !== 'auto' ? a : area === 0 ? 'both' : 'homes'),
  fillsHomes: (a: string) => a === 'homes',
}));
const { drawnStretches, liveStreet, runsOf, unfinished, verdictOf } = await import('../src/ui/townedit/streetLive');
const { StreetDrawing, startAt, streetGhosts, streetTool, stretchNear } = await import('../src/ui/townedit/streetTool');

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
    open: { areas: [{ name: 'Ashby', kind: 'hamlet', x: 50, z: 50, w: 300, d: 200 }], zones: [], water: rle(water), pads: {}, parking: [], places: [], backdrop: [], ...(land ? { land } : {}) },
  } as unknown as NeighbourhoodDraft;
}

const t: Templates = { plot: { width: 14, depth: 18 }, houses: [], parks: [] };
const content = { names: { streets: ['Orchard Way', 'Church Road', 'Kiln Lane', 'Station Road', 'Brook Street', 'Elm Close'] } } as unknown as Content;
const refusal = (e: Edited) => ('refused' in e ? e.refused : null);

/** The live checks' verdict and `drawStreet`'s on the same stroke. */
function both(draft: NeighbourhoodDraft, points: Pt[], kind: RoadPlan['kind'] & string = 'street') {
  return { live: liveStreet(draft, t, points, kind, true), drawn: drawStreet(draft, t, content, { points, kind, clear: true, fill: false, seed: 7 }) };
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
    // Once drawStreet has had its say, the ghosts are the roads it makes: the bridge pale.
    const g = streetGhosts({ draft, t }, live, drawn);
    expect(g.roads!.map((r) => r.tone).sort()).toEqual(['bridge', 'new', 'new']);
  });

  test('a crossing too wide for a bridge: red over the water, the same reason', () => {
    const { live, drawn } = both(town(60), [
      [87, MAIN_Z + 8],
      [87, FAR_Z - 8],
    ]);
    expect(live.why).not.toBeNull();
    expect(live.why).toBe(refusal(drawn));
    const bad = live.stretches.find((s) => s.tone === 'bad')!;
    // (Its stretch: the water and the straight run onto it either side.)
    expect(bad.from).toBeLessThan(RIVER_Z - MAIN_Z);
    expect(bad.to).toBeGreaterThan(RIVER_Z + 60 - MAIN_Z);
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

  test('a junction within 14 m of another: its mark red, the same reason', () => {
    const { live, drawn } = both(town(), [
      [LANE_X + 6, MAIN_Z - 8],
      [LANE_X + 6, 50],
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

  test("a dead end longer than a court is a street not finished yet: said, not red; drawStreet's refusal is of the same kind", () => {
    const { live, drawn } = both(town(), [
      [250, MAIN_Z - 8],
      [250, 30],
    ]);
    expect(live.why).toBeNull();
    expect(live.reached).toBe(false);
    expect(live.open).toMatch(/dead end reaches 45 m at most/);
    expect(live.stretches).toEqual([]);
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
    const live = liveStreet(draft, t, up, 'street');
    expect(live.steep).toBe(true);
    expect(live.slope).toMatch(/too steep here/);
    expect(live.why).toBeNull();
    expect(live.stretches.some((s) => s.tone === 'bad')).toBe(true);
    const flat = liveStreet(town(30, blankLand(W, D)), t, up, 'street');
    expect(flat.steep).toBe(false);
    expect(flat.why).toBeNull();
    // Inside the town's box the same slope is not judged.
    const inTown = town(30, steep);
    inTown.open!.areas[0] = { ...inTown.open!.areas[0], z: 0, d: 250 };
    expect(liveStreet(inTown, t, up, 'street').steep).toBe(false);
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
    expect(editor.said.hints.at(-1)).toMatch(/with a 30 m bridge\. Click to draw it/);
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

  test('Enter on a dead end too long, or a street refused, keeps what was drawn and says why', () => {
    const editor = fakeEditor(town());
    const tool = streetTool.stage(editor);
    tool.down(250, MAIN_Z - 8);
    tool.up();
    tool.down(250, 30);
    tool.up();
    tool.key!(key('Enter'));
    expect(editor.said.refusals.at(-1)).toMatch(/dead end reaches 45 m at most/);
    expect(editor.draft.roads).toHaveLength(4);
    expect(tool.key!(key('Backspace'))).toBe(true);
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

// On a generated river world: random strokes in and round the main town, each judged by the live
// checks and by drawStreet. Every reason the live checks give (the slope is not one) is
// drawStreet's own; drawStreet refuses nothing the live checks pass but what they leave to it.
describe('on a generated world', async () => {
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
  const { Content: RealContent } = await import('../src/content/content');
  const { objectInfos } = await import('../src/game/housegen');
  const { generateWorld, useTownContent } = await import('../src/game/town');
  const { mulberry32 } = await import('../src/game/noise');
  const real = await RealContent.load('http://localhost/content/base.json');
  useTownContent(real);
  const json = <T>(file: string) => JSON.parse(readFileSync(`${WEB}public/content/${file}`, 'utf8')) as T;
  const templates: Templates = { ...json<Templates>('houses.json'), gen: json('housegen.json'), info: objectInfos(real), lampTiles: real.lampTiles, windowTiles: real.windowTiles };
  const world = generateWorld(real, templates, 'river', 4242, undefined, undefined, { town: 'village', villages: 'few' });

  test('every live verdict is drawStreet’s, within the per-move budget', () => {
    const a0 = world.open!.areas[0];
    const rand = mulberry32(9);
    let [agreed, refused, bridged] = [0, 0, 0];
    const left: string[] = [];
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
      const live = liveStreet(world, templates, pts, kind);
      times.push(performance.now() - t0);
      const why = refusal(verdictOf(world, templates, real, pts, kind, true, 1));
      const liveWhy = live.why;
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
    console.log(`streetTool: ${agreed} of 400 strokes judged alike (${refused} refused alike, ${bridged} bridged); left to drawStreet: ${left.length ? left.join('; ') : 'none'}; live checks ${median.toFixed(2)} ms median, ${p95.toFixed(2)} ms p95`);
    // (What the live checks leave to drawStreet: the turns onto a bridge, the network.)
    for (const why of left) expect(why).toMatch(/onto its bridge|over the water where it would not be a bridge|joins none/);
    expect(agreed + left.length).toBe(400);
    expect(median).toBeLessThan(5);
  });
});
