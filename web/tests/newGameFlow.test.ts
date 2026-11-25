// The new game's steps keep what they made while the player goes back and forth: a step's key says
// what it was made from, so a later step is made again only when that changed. Also the code step 1
// opens with, the code the player types, the town choices a land is too small for, and the land's figures.
import { describe, expect, test } from 'bun:test';
import { blankLand, COVER, coverSpan } from '../src/game/land';
import type { NeighbourhoodDraft } from '../src/game/town';
import { townFits } from '../src/game/world';
import type { RoadPlan } from '../src/core/protocol';
import { TownHistory } from '../src/game/edit/history';
import { usesEmpty } from '../src/game/uses';
import {
  addedStreets,
  buildKey,
  changesWord,
  codeFrom,
  firstCode,
  fitted,
  goesWords,
  landKey,
  landStats,
  layoutHash,
  layoutPlan,
  peopleKey,
  startCode,
  stepLabel,
  STEPS,
  townKey,
  whatGoes,
  withSeed,
  worldKeyOf,
} from '../src/ui/newGame';

describe("the steps' keys", () => {
  const code = startCode('river', 1234);

  test('the land is keyed by its place, size, hills, woods and seed, not by the town', () => {
    const key = landKey(code, null);
    expect(landKey({ ...code, town: 'city', villages: 'many', townSeed: 9 }, null)).toBe(key);
    for (const other of [{ kind: 'heath' as const }, { size: 'large' as const }, { hills: 'mountains' as const }, { woods: 'open' as const }, { seed: 1235 }]) {
      expect(landKey({ ...code, ...other }, null)).not.toBe(key);
    }
  });

  test('a land shaped by hand is another land, and shaped again another still', () => {
    const field = blankLand(64, 64);
    expect(landKey(code, field)).toBe(landKey(code, null));
    field.edited = true;
    field.h[10] = 120;
    const shaped = landKey(code, field);
    expect(shaped).not.toBe(landKey(code, null));
    field.h[11] = 80;
    expect(landKey(code, field)).not.toBe(shaped);
  });

  test("the town is keyed by its land, town, villages, own seed and site; its name doesn't count", () => {
    const land = landKey(code, null);
    const key = townKey(land, code, null);
    expect(townKey(land, { ...code }, null)).toBe(key);
    expect(townKey(landKey({ ...code, seed: 5 }, null), code, null)).not.toBe(key);
    expect(townKey(land, { ...code, town: 'hamlet' }, null)).not.toBe(key);
    expect(townKey(land, { ...code, villages: 'none' }, null)).not.toBe(key);
    expect(townKey(land, { ...code, townSeed: 77 }, null)).not.toBe(key);
    expect(townKey(land, code, [120, 80])).not.toBe(key);
    expect(townKey(land, code, [120.2, 79.8])).toBe(townKey(land, code, [120, 80]));
  });

  test("a new land seed lets the town's and the people's own seeds go", () => {
    expect(withSeed({ ...code, townSeed: 3, peopleSeed: 4 }, 99)).toEqual({ ...code, seed: 99 });
  });
});

describe('the five steps', () => {
  test('in order, each with its label, the first three with their buttons on', () => {
    expect(STEPS.map((s) => s.id)).toEqual(['land', 'streets', 'town', 'household', 'home']);
    expect(stepLabel(2)).toBe('Step 2 of 5 · Streets & zones');
    expect(stepLabel(3)).toBe('Step 3 of 5 · Build');
    expect(STEPS.slice(0, 3).map((s) => s.next)).toEqual(['Lay out the streets →', 'Build the town →', 'Choose a household →']);
  });

  test("the built town is keyed by the layout's key, the layout as left and the build's seed", () => {
    const key = buildKey('L', 'h', 7);
    expect(buildKey('L', 'h', 7)).toBe(key);
    for (const other of [buildKey('M', 'h', 7), buildKey('L', 'i', 7), buildKey('L', 'h', 8)]) expect(other).not.toBe(key);
  });
});

describe('step 2 and step 3: the layout, the built town and what each one remakes', () => {
  const road = (x0: number, z0: number, x1: number, z1: number, name?: string): RoadPlan => ({
    points: [
      [x0, z0],
      [(x0 + x1) / 2, (z0 + z1) / 2],
      [x1, z1],
    ],
    width: 7,
    ends: ['out', 'out'],
    kind: 'street',
    ...(name ? { name } : {}),
  });
  const plan = (roads: RoadPlan[]) => ({ name: 'Ashby', width: 400, depth: 400, slots: [], roads, greens: [], households: [], open: { areas: [], uses: usesEmpty(100, 100) } }) as unknown as NeighbourhoodDraft;
  const proposed = plan([road(0, 100, 400, 100, 'High Street')]);
  const layout = { key: 'L', proposal: { plan: proposed } };

  test("Build makes the town from the layout as left: the edits' plan, else the proposal's", () => {
    expect(layoutPlan(layout)).toBe(proposed);
    const history = new TownHistory(proposed);
    const drawn = plan([...proposed.roads, road(200, 100, 200, 300)]);
    history.push(drawn);
    expect(layoutPlan({ ...layout, plan: proposed, history })).toBe(drawn);
  });

  test("the built town's key follows the layout's key, the layout as left and the build's seed; its people's follow it", () => {
    const key = worldKeyOf(layout, 0);
    expect(key).toBe(buildKey('L', layoutHash(proposed), 0));
    // (Back and forth with nothing changed: the same key, the tweaks and the households kept.)
    expect(worldKeyOf({ ...layout, proposal: { plan: structuredClone(proposed) } }, 0)).toBe(key);
    // (A street drawn in step 2, the build seed rolled, another proposal: built again, lived in again.)
    const history = new TownHistory(proposed);
    history.push(plan([...proposed.roads, road(200, 100, 200, 300)]));
    for (const other of [worldKeyOf({ ...layout, history }, 0), worldKeyOf(layout, 9), worldKeyOf({ ...layout, key: 'M' }, 0)]) {
      expect(other).not.toBe(key);
      expect(peopleKey(other, 5)).not.toBe(peopleKey(key, 5));
    }
  });

  test('what each change remakes, and what it asks about first', () => {
    const none = { layoutEdited: false, townTweaked: false };
    expect(whatGoes('land', none)).toEqual({ remake: ['layout', 'town', 'people'], ask: [] });
    expect(whatGoes('propose', { layoutEdited: true, townTweaked: false }).ask).toEqual(['layout']);
    expect(whatGoes('propose', { layoutEdited: true, townTweaked: true }).ask).toEqual(['layout', 'town']);
    // (A layout change or Build again keeps the layout: only the built town's tweaks are asked about.)
    expect(whatGoes('layout', { layoutEdited: true, townTweaked: false })).toEqual({ remake: ['town', 'people'], ask: [] });
    expect(whatGoes('build', { layoutEdited: true, townTweaked: true }).ask).toEqual(['town']);
    expect(goesWords(['layout', 'town'])).toBe('your streets and zones, and your changes to the built town');
    expect([changesWord(1), changesWord(4), changesWord(0)]).toEqual(['your change', 'your 4 changes', 'your changes']);
  });

  test("a street drawn in step 3 goes into the layout; one split by its junction is replaced by its pieces", () => {
    const built = plan([road(0, 100, 400, 100, '1 High Street')]);
    const side = road(200, 100, 200, 300, 'New Road');
    // (The built town names its roads its own way: the layout's are matched by their line.)
    const after = { ...built, roads: [...built.roads, side] };
    const added = addedStreets(proposed, built, after);
    expect(added.roads.map((r) => r.name)).toEqual(['High Street', 'New Road']);
    expect(addedStreets(added, built, after)).toEqual(added);
    // (Nothing drawn: the layout as it was.)
    expect(addedStreets(proposed, built, { ...built })).toBe(proposed);
    expect(addedStreets(proposed, after, built)).toBe(proposed);
    // (The new street undone or removed in step 3: out of the layout again.)
    expect(addedStreets(added, after, built).roads.map((r) => r.name)).toEqual(['High Street']);
    // (The high street split where the new street meets it.)
    const west = road(0, 100, 200, 100);
    const east = road(200, 100, 400, 100);
    const split = addedStreets(proposed, built, { ...built, roads: [west, east, side] });
    expect(split.roads.map((r) => r.points[0][0] + ',' + r.points.at(-1)![0])).toEqual(['0,200', '200,400', '200,200']);
    // (A piece of a road the layout hasn't got stays out.)
    const other = plan([road(0, 50, 400, 50, 'Back Lane')]);
    expect(addedStreets(other, built, { ...built, roads: [west, east, side] }).roads.map((r) => r.name)).toEqual(['Back Lane', 'New Road']);
  });
});

describe("the land's room for a town", () => {
  test('a new code fits its town and villages on its land', () => {
    for (const kind of ['country', 'river', 'island', 'mountain'] as const) {
      const c = startCode(kind, 1);
      expect(townFits(c.size, c.town, c.villages)).toBe(true);
    }
  });

  test('too much for the land: fewer villages first, then a smaller town', () => {
    const big = { ...startCode('country', 1), size: 'small' as const, town: 'city' as const, villages: 'many' as const };
    const fit = fitted(big);
    expect(townFits(fit.size, fit.town, fit.villages)).toBe(true);
    const fits = fitted({ ...big, size: 'large' });
    expect(townFits(fits.size, fits.town, fits.villages)).toBe(true);
    // (What already fits is left alone.)
    const ok = startCode('country', 1);
    expect(fitted(ok)).toBe(ok);
  });
});

describe('the code', () => {
  test('step 1 opens on the countryside, the timing tools\' place and options, or the URL\'s code', () => {
    expect(firstCode(null, null, 42)).toEqual(startCode('country', 42));
    expect(firstCode(null, { kind: 'island', options: { hills: 'flat' } }, 42)).toMatchObject({ kind: 'island', hills: 'flat', seed: 42 });
    // (A kind from before places: its place at its size.)
    expect(firstCode(null, { kind: 'dense', options: {} }, 42)).toMatchObject({ kind: 'country', town: 'city' });
    expect(firstCode('river-lhs-k2x9q', { kind: 'island', options: {} }, 42)).toMatchObject({ kind: 'river', size: 'large', hills: 'hilly', woods: 'some', seed: parseInt('k2x9q', 36) });
    expect(firstCode('777', { kind: 'island', options: {} }, 42)).toMatchObject({ kind: 'island', seed: 777 });
    // (Not a code: as if there were none.)
    expect(firstCode('not a code', null, 42)).toEqual(startCode('country', 42));
  });

  test('typed: a number is a new land seed, a whole code is taken as it is', () => {
    const now = { ...startCode('heath', 5), townSeed: 8 };
    expect(codeFrom('12', now)).toEqual(withSeed(now, 12));
    expect(codeFrom('island-lfwcm-k2x9q-1b-7', now)).toEqual({ kind: 'island', size: 'large', hills: 'flat', woods: 'wooded', town: 'city', villages: 'many', seed: parseInt('k2x9q', 36), townSeed: parseInt('1b', 36), peopleSeed: 7 });
    expect(codeFrom('nonsense-xyz', now)).toBeNull();
  });
});

describe("the land's figures", () => {
  const draft = (over: Partial<NeighbourhoodDraft>): NeighbourhoodDraft => ({ name: 'Test', width: 100, depth: 100, slots: [], roads: [], greens: [], households: [], ...over });

  test("from the draft: its water tiles and its woods' area", () => {
    const d = draft({
      open: {
        areas: [],
        zones: [{ kind: 'woods', poly: [[0, 0], [50, 0], [50, 40], [0, 40]] }],
        // (Run-length: 2500 dry, 1000 wet, the rest dry.)
        water: [2500, 0, 1000, 1, 6500, 0],
        pads: {},
        parking: [],
      },
    });
    expect(landStats(d)).toEqual({ width: 100, depth: 100, water: 0.1, woods: 0.2 });
  });

  test('from the land as shaped: its wet samples and its wooded cells', () => {
    const field = blankLand(100, 100);
    for (let k = 0; k < field.h.length / 4; k++) field.wl[k] = field.h[k] + 50;
    const cells = coverSpan(field.cols) * coverSpan(field.rows);
    for (let k = 0; k < cells / 2; k++) field.cover[k] = COVER.woods;
    const s = landStats(draft({}), field);
    expect(s.water).toBeCloseTo(0.25, 2);
    expect(s.woods).toBeCloseTo(0.5, 2);
  });
});
