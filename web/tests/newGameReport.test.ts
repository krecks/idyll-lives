// New game's state in a debug report (`debug/newGameReport.ts`): packed and read back it is the same state (the
// land's height field, the layout, the objects the drafts share), with the same keys, so `?debugReport=` opens New
// game again with nothing made again; later steps open at the streets, step 1 with its land as on screen.
import { describe, expect, test } from 'bun:test';
import { loadContent, templatesOf, usePublicFetch } from './support/content';
import { blankLand, landHash, NONE } from '../src/game/land';
import { TownHistory } from '../src/game/edit/history';
import { newGameReport, pack, readNewGameReport, reportShownLand, unpack, type NewGameState } from '../src/debug/newGameReport';
import { landKey, startCode, townKey, worldKeyOf } from '../src/ui/newGame';
import type { Layout, MadeLand } from '../src/ui/app.svelte';
import type { NeighbourhoodDraft } from '../src/game/town';

describe('packing a graph', () => {
  test('typed arrays, shared objects, a height field and odd numbers come back as they were', () => {
    const land = blankLand(64, 48);
    land.h[5] = 1234;
    land.wl[7] = 300;
    land.cover[2] = 3;
    land.edited = true;
    const heights = new Float32Array([1.5, -2.25, NaN]);
    const shared = { a: [1, 2], heights };
    const value = { one: shared, two: shared, land, again: land, odd: [Infinity, -Infinity, undefined, null], map: new Map([['k', shared]]), set: new Set([3, 4]), gone: undefined };
    const packed = JSON.parse(JSON.stringify(pack(value)));
    const back = unpack(packed) as typeof value;
    expect(back.one).toBe(back.two);
    expect(back.map.get('k')).toBe(back.one);
    expect(back.land).toBe(back.again);
    expect(back.one.heights).toBeInstanceOf(Float32Array);
    expect([...back.one.heights].map(String)).toEqual(['1.5', '-2.25', 'NaN']);
    expect(back.land.h).toBeInstanceOf(Int16Array);
    expect(landHash(back.land)).toBe(landHash(land));
    expect(back.land.edited).toBe(true);
    expect(back.land.seeded).toBe(land.seeded);
    expect(back.odd).toEqual([Infinity, -Infinity, undefined, null]);
    expect(back.odd.length).toBe(4);
    expect([...back.set]).toEqual([3, 4]);
    expect('gone' in back).toBe(false);
  });

  test('a block met in two objects is written once', () => {
    const a = blankLand(200, 200);
    const b = { ...a, h: a.h.slice(), wl: a.wl.slice(), cover: a.cover.slice() };
    const packed = pack({ a, b, f: new Float32Array(8), g: new Float32Array(8) });
    expect(packed.blobs.length).toBe(2);
    const back = unpack(packed) as { a: typeof a; b: typeof a };
    // (Two objects then, two now.)
    expect(back.a).not.toBe(back.b);
    expect(landHash(back.b)).toBe(landHash(a));
  });
});

describe("New game's report", async () => {
  usePublicFetch();
  const { useTownContent } = await import('../src/game/town');
  const { runTownJob } = await import('../src/game/townJobs');
  const content = await loadContent();
  useTownContent(content);
  const t = await templatesOf(content);

  // A large river land (480 × 480 samples) with a lake painted on it, and the layout proposed on it.
  const code = { ...startCode('river', 4242, { size: 'large', villages: 'few' }), townSeed: 77 };
  const made = runTownJob(content, t, { kind: 'land', world: code.kind, seed: code.seed, options: { size: code.size, hills: code.hills, woods: code.woods } });
  if (made.kind !== 'land') throw new Error('no land');
  const field = made.draft.open!.land!;
  const shaped = { ...field, h: field.h.slice(), wl: field.wl.slice(), cover: field.cover.slice(), edited: true };
  for (let j = 200; j < 260; j++) for (let i = 100; i < 380; i++) shaped.wl[j * shaped.cols + i] = shaped.h[j * shaped.cols + i] + 50;
  const draft: NeighbourhoodDraft = { ...made.draft, open: { ...made.draft.open!, land: shaped } };
  const land: MadeLand = { draft, key: landKey(code, shaped), waterKept: false };
  const site: [number, number] = [400, 520];
  const proposed = runTownJob(content, t, { kind: 'layout', land: draft, seed: code.townSeed, options: { town: code.town, villages: code.villages }, site, waterKept: false });
  if (proposed.kind !== 'layout') throw new Error('no layout');
  const layout: Layout = { draft: proposed.draft, key: townKey(land.key, code, site), proposal: proposed.proposal, edited: false, plan: proposed.plan ?? proposed.proposal.plan, ...(proposed.marks ? { marks: proposed.marks } : {}) };
  const state: NewGameState = { screen: 'streets', code, land, site, townName: 'Testbury', buildSeed: 5, layout };

  test('read back, it is the state it was, with the same keys', () => {
    const text = newGameReport(state)!;
    const back = readNewGameReport(text);
    expect(back.step).toBe('streets');
    expect(back.code).toEqual(code);
    expect(back.site).toEqual(site);
    expect(back.townName).toBe('Testbury');
    expect(back.buildSeed).toBe(5);
    expect(back.land!.waterKept).toBe(false);
    expect(back.land!.key).toBe(land.key);
    expect(back.land!.draft).toEqual(land.draft);
    // The steps' own keys, worked out again from what came back, still match: nothing is made again.
    expect(landKey(back.code!, back.land!.draft.open!.land)).toBe(land.key);
    expect(townKey(back.land!.key, back.code!, back.site)).toBe(layout.key);
    expect(back.layout).toEqual(layout);
    expect(worldKeyOf(back.layout!, back.buildSeed)).toBe(worldKeyOf(layout, 5));
    // The code as the player would copy it rides along.
    expect(JSON.parse(text).code).toMatch(/^river-/);
    // (A sane size for a large land and its layout.)
    expect(text.length).toBeLessThan(8_000_000);
    console.log(`[newGameReport] ${(text.length / 1e6).toFixed(2)} MB for a large land and its layout`);
  });

  test('a later step opens at the streets; off New game there is no part', () => {
    for (const screen of ['town', 'household', 'create', 'home'] as const) expect(readNewGameReport(newGameReport({ ...state, screen })!).step).toBe('streets');
    expect(newGameReport({ ...state, screen: 'menu' })).toBeNull();
    expect(newGameReport({ ...state, screen: 'game' })).toBeNull();
  });

  test('edits not yet settled come back ahead of the plan', () => {
    const plan = layout.plan!;
    const history = new TownHistory(plan);
    const edited = { ...plan, roads: plan.roads.slice(1) };
    history.push(edited);
    const back = readNewGameReport(newGameReport({ ...state, layout: { ...layout, edited: true, history } })!).layout!;
    expect(back.edited).toBe(true);
    expect(back.plan).toEqual(plan);
    expect(back.history!.draft).toEqual(edited);
    expect(worldKeyOf(back, 0)).toBe(worldKeyOf({ ...layout, history }, 0));
  });

  test("step 1's land as on screen is what opens, its town's site going with the land taken on before", () => {
    const more = { ...shaped, h: shaped.h.slice(), edited: true };
    more.h[0] += 300;
    more.wl[1] = NONE;
    const onScreen: MadeLand = { draft: { ...draft, open: { ...draft.open!, land: more } }, key: landKey(code, more), waterKept: false };
    const stop = reportShownLand(() => onScreen);
    try {
      const back = readNewGameReport(newGameReport({ ...state, screen: 'land' })!);
      expect(back.step).toBe('land');
      expect(back.land!.key).toBe(onScreen.key);
      expect(landHash(back.land!.draft.open!.land)).toBe(landHash(more));
      expect(back.site).toBeNull();
      // The same land as taken on: nothing extra, the site kept.
      const same = reportShownLand(() => land);
      const kept = readNewGameReport(newGameReport({ ...state, screen: 'land' })!);
      expect(kept.land!.key).toBe(land.key);
      expect(kept.site).toEqual(site);
      same();
    } finally {
      stop();
    }
  });
});
