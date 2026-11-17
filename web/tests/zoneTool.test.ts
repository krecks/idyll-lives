// Step 2's zone brush (`ui/townedit/zoneTool.ts`) and the layout's own edits it and the panel make
// (`ui/townedit/layoutApi.ts`), on a small hand-made town: a high street across the world with a
// lane off it and a river south of it. A stroke is one edit (one undo step), shown on the overlay as
// it goes; zones never go on the water, its shore, a road or off the world; "Beside streets only"
// keeps them near a road, Erase goes anywhere; a dropped stroke leaves the plan as it was. The summary
// counts what the zones would build, far zones are hatched, and Clear all and Clear zones clear what
// they say. `bun test`.
import { describe, expect, mock, test } from 'bun:test';
import type { Content } from '../src/content/content';
import type { RoadPlan } from '../src/core/protocol';
import type { Edited } from '../src/game/edit/edited';
import { rle } from '../src/game/landMasks';
import type { Pt } from '../src/game/roads';
import type { NeighbourhoodDraft, PlotSlot, Templates } from '../src/game/town';
import { unpackUses, USE_CELL, USE_CODE, usesFor, type UseKind } from '../src/game/uses';
import type { LandCursor } from '../src/render/types';
import type { TownEditor } from '../src/ui/townedit/tools';

// (The card and its runes are the page's: the tool reads its choices from a plain object here.)
const choice = { kind: 'homes' as UseKind | null, size: 16 as 8 | 16 | 32 | 64, beside: false };
mock.module('../src/ui/townedit/ZoneCard.svelte', () => ({ default: {} }));
mock.module('../src/ui/townedit/zoneChoice.svelte', () => ({ ZONE_SIZES: [8, 16, 32, 64], zoneChoice: choice }));
const { zoneTool } = await import('../src/ui/townedit/zoneTool');
const { BESIDE_STREETS, clearAll, clearZones, needsSettle, noUseCells, paintUses, roadReach, shownUses, summaryLine, villageAsLayout } = await import('../src/ui/townedit/layoutApi');

const W = 400;
const D = 400;
const MAIN_Z = 100;
const RIVER_Z = 200;
const LANE_X = 170;

const line = (x0: number, z0: number, x1: number, z1: number): Pt[] => {
  const n = Math.max(1, Math.ceil(Math.hypot(x1 - x0, z1 - z0)));
  return Array.from({ length: n + 1 }, (_, i): Pt => [x0 + ((x1 - x0) * i) / n, z0 + ((z1 - z0) * i) / n]);
};

/** The layout: the high street out both ways, Mill Lane off it, a river 30 m across, no lots, zones empty. */
function town(): NeighbourhoodDraft {
  const roads: RoadPlan[] = [
    { points: line(0, MAIN_Z, W, MAIN_Z), width: 7, ends: ['out', 'out'], kind: 'country', name: 'High Street' },
    { points: line(LANE_X, MAIN_Z + 2.15, LANE_X, 160), width: 7, ends: ['join', 'dead'], kind: 'lane', area: 0, name: 'Mill Lane' },
  ];
  const water = new Uint8Array(W * D);
  for (let z = RIVER_Z; z < RIVER_Z + 30; z++) for (let x = 0; x < W; x++) water[z * W + x] = 1;
  const draft = {
    name: 'Ashby',
    width: W,
    depth: D,
    slots: [],
    roads,
    greens: [],
    households: [],
    open: { areas: [{ name: 'Ashby', kind: 'hamlet', x: 50, z: 50, w: 300, d: 140 }], zones: [], water: rle(water), pads: {}, parking: [], places: [], backdrop: [] },
  } as unknown as NeighbourhoodDraft;
  return { ...draft, open: { ...draft.open!, uses: usesFor(draft) } };
}

const t: Templates = { plot: { width: 14, depth: 18 }, houses: [], parks: [] };
const content = {} as Content;
const cellAt = (d: NeighbourhoodDraft, x: number, z: number) => unpackUses(d.open!.uses!)[Math.floor(z / USE_CELL) * d.open!.uses!.cols + Math.floor(x / USE_CELL)];

/** An editor over `draft` whose ground is the screen (a screen pixel a metre), recording what the tool asks of it. */
function fakeEditor(draft: NeighbourhoodDraft) {
  const said = { applied: 0, edits: [] as Edited[], shown: [] as (NeighbourhoodDraft | null)[], rings: [] as (LandCursor | null)[], hints: [] as (string | null)[] };
  const editor: { -readonly [K in keyof TownEditor]: TownEditor[K] } & { said: typeof said } = {
    said,
    step: 'layout',
    draft,
    t,
    content,
    version: 0,
    apply(edited: Edited) {
      if ('refused' in edited) return false;
      said.applied++;
      said.edits.push(edited);
      editor.draft = edited.draft;
      editor.version++;
      return true;
    },
    refuse: () => {},
    seed: () => 1,
    ground: (x, y) => (x < 0 || y < 0 ? null : { x, z: y }),
    pick: () => null,
    selected: null,
    select: () => {},
    hover: () => {},
    preview: () => {},
    hint: (text) => void said.hints.push(text),
    brush: (c) => void said.rings.push(c),
    showUses: (p) => void said.shown.push(p),
    townName: 'Ashby',
    toolId: 'zones',
    take: () => {},
  };
  return editor;
}

describe('the zone brush', () => {
  test('a drag is one stroke: shown as it goes, one edit taken when let go', () => {
    Object.assign(choice, { kind: 'homes', size: 16, beside: false });
    const ed = fakeEditor(town());
    const stage = zoneTool.stage(ed);
    expect(stage.down(60, 120)).toBe(true);
    for (let x = 60; x <= 140; x += 7) stage.move(x, 120);
    expect(ed.said.applied).toBe(0);
    expect(ed.said.shown.filter(Boolean).length).toBeGreaterThan(1);
    stage.up();
    expect(ed.said.applied).toBe(1);
    // (A stroke changes the zones alone: no land to settle under it.)
    expect(needsSettle(ed.said.edits[0])).toBe(false);
    expect(ed.said.shown.at(-1)).toBeNull();
    // (Painted all along the drag, no gaps for a quick one.)
    for (let x = 60; x <= 140; x += 4) expect(cellAt(ed.draft, x, 120)).toBe(USE_CODE.homes);
    expect(cellAt(ed.draft, 60, 140)).toBe(0);
    // (The ring follows in the use's colour, as big as the brush.)
    expect(ed.said.rings.at(-1)).toMatchObject({ radius: 8, tone: 'homes' });
  });

  test('a stroke dropped (right-click, Esc) leaves the plan as it was', () => {
    Object.assign(choice, { kind: 'shops', size: 32, beside: false });
    const before = town();
    const ed = fakeEditor(before);
    const stage = zoneTool.stage(ed);
    stage.down(80, 130);
    stage.move(120, 130);
    stage.cancel();
    stage.up();
    expect(ed.said.applied).toBe(0);
    expect(ed.draft).toBe(before);
    expect(ed.said.shown.at(-1)).toBeNull();
  });

  test('a press off the land is not taken; a stroke that changes nothing is no edit', () => {
    Object.assign(choice, { kind: null, size: 8, beside: false });
    const ed = fakeEditor(town());
    const stage = zoneTool.stage(ed);
    expect(stage.down(-5, 10)).toBe(false);
    expect(stage.down(60, 130)).toBe(true);
    stage.up();
    expect(ed.said.applied).toBe(0);
  });

  test('Erase takes zones off, anywhere; [ and ] step the size', () => {
    Object.assign(choice, { kind: 'green', size: 64, beside: false });
    const ed = fakeEditor(town());
    const stage = zoneTool.stage(ed);
    stage.down(300, 150);
    stage.up();
    expect(cellAt(ed.draft, 300, 150)).toBe(USE_CODE.green);
    choice.kind = null;
    choice.beside = true;
    stage.down(300, 150);
    stage.up();
    expect(cellAt(ed.draft, 300, 150)).toBe(0);
    expect(ed.said.applied).toBe(2);
    expect(stage.key?.({ key: '[' } as KeyboardEvent)).toBe(true);
    expect(choice.size).toBe(32);
    stage.key?.({ key: ']' } as KeyboardEvent);
    stage.key?.({ key: ']' } as KeyboardEvent);
    expect(choice.size).toBe(64);
  });

  test('beside streets only: paints within 40 m of a road and no further', () => {
    Object.assign(choice, { kind: 'homes', size: 64, beside: true });
    const ed = fakeEditor(town());
    const stage = zoneTool.stage(ed);
    // (East of Mill Lane's end, from the high street's kerb down past the reach.)
    stage.down(300, MAIN_Z + 20);
    for (let z = MAIN_Z + 20; z <= MAIN_Z + 90; z += 5) stage.move(300, z);
    stage.up();
    const d = roadReach(ed.draft);
    const codes = unpackUses(ed.draft.open!.uses!);
    let near = 0;
    for (let k = 0; k < codes.length; k++) {
      if (!codes[k]) continue;
      expect(d[k]).toBeLessThanOrEqual(BESIDE_STREETS);
      near++;
    }
    expect(near).toBeGreaterThan(20);
    expect(cellAt(ed.draft, 300, MAIN_Z + 20)).toBe(USE_CODE.homes);
    expect(cellAt(ed.draft, 300, MAIN_Z + 80)).toBe(0);
  });
});

describe("the layout's edits", () => {
  test("a stroke never zones the water, its shore, a road or off the world; the summary's line", () => {
    Object.assign(choice, { kind: 'rows', size: 64, beside: false });
    const ed = fakeEditor(town());
    const stage = zoneTool.stage(ed);
    stage.down(10, 10);
    for (let z = 10; z <= D + 30; z += 30) for (const x of (z / 30) % 2 ? [W + 30, 10] : [10, W + 30]) stage.move(x, z);
    stage.up();
    const codes = unpackUses(ed.draft.open!.uses!);
    const none = noUseCells(ed.draft);
    let zoned = 0;
    for (let k = 0; k < codes.length; k++) {
      if (!codes[k]) continue;
      zoned++;
      expect(none[k]).toBe(0);
    }
    expect(zoned).toBeGreaterThan(1000);
    expect(summaryLine({ homes: 84, shops: 12, parks: 3, public: 5 })).toBe('Homes ≈ 84 · Shops ≈ 12 · Parks 3 · Public 5');
  });

  test('a street drawn across zones: the overlay draws none under it, the zones kept beside it (and under it, to come back should it go)', () => {
    const e = paintUses(town(), [{ x: 120, z: 140, size: 32 }], 'homes');
    if (!('draft' in e)) throw new Error(e.refused);
    const zoned = e.draft;
    expect(shownUses(zoned)).toBe(zoned.open!.uses!);
    // (A street across the zone, as a drawn one leaves it: the cells under it still painted.)
    const crossed: NeighbourhoodDraft = { ...zoned, roads: [...zoned.roads, { points: line(100, 140, 140, 140), width: 7, ends: ['dead', 'dead'], kind: 'street', area: 0, name: 'New Street' }] };
    expect(cellAt(crossed, 120, 140)).toBe(USE_CODE.homes);
    const drawn = shownUses(crossed)!;
    expect(shownUses(crossed)).toBe(drawn);
    const shownAt = (x: number, z: number) => unpackUses(drawn)[Math.floor(z / USE_CELL) * drawn.cols + Math.floor(x / USE_CELL)];
    expect(shownAt(120, 140)).toBe(0);
    expect(shownAt(120, 130)).toBe(USE_CODE.homes);
    expect(shownAt(120, 150)).toBe(USE_CODE.homes);
    expect(shownUses({ ...crossed, open: { ...crossed.open!, uses: undefined } })).toBeNull();
  });

  test('Clear zones takes every zone; Clear all leaves bare land and the main town', () => {
    const draft = town();
    const painted = paintUses(draft, [{ x: 100, z: 130, size: 60 }], 'shops');
    if (!('draft' in painted)) throw new Error('refused');
    const zones = clearZones(painted.draft);
    expect('draft' in zones && unpackUses(zones.draft.open!.uses!).every((c) => c === 0)).toBe(true);
    const all = clearAll(painted.draft);
    if (!('draft' in all)) throw new Error('refused');
    expect(all.draft.roads).toEqual([]);
    expect(all.draft.slots).toEqual([]);
    expect(all.draft.open!.areas.map((a) => a.name)).toEqual(['Ashby']);
    expect(unpackUses(all.draft.open!.uses!).every((c) => c === 0)).toBe(true);
  });

  test("the Village tool's village in step 2: its lots go, painted as the zones they stood for", () => {
    const draft = town();
    const lot = (index: number, x: number, kind: PlotSlot['kind'], area: number): PlotSlot => ({ index, kind, template: 'h', name: `${index}`, x, z: 300, facing: 0, lot: 'medium', w: 14, d: 18, area });
    const placed: Edited = {
      draft: {
        ...draft,
        slots: [lot(0, 250, 'house', 1), lot(1, 280, 'shop', 1)],
        open: { ...draft.open!, areas: [...draft.open!.areas, { name: 'Brook', kind: 'hamlet', x: 240, z: 280, w: 80, d: 60 }] },
      } as NeighbourhoodDraft,
      cleared: { lots: [], backdrop: [] },
    };
    const zoned = villageAsLayout(draft, placed, t);
    if (!('draft' in zoned)) throw new Error('refused');
    expect(zoned.draft.slots).toEqual([]);
    expect(zoned.draft.open!.areas.length).toBe(2);
    expect(cellAt(zoned.draft, 256, 308)).toBe(USE_CODE.homes);
    expect(cellAt(zoned.draft, 286, 308)).toBe(USE_CODE.shops);
  });
});
