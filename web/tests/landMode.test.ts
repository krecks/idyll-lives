// Build mode's Land tab (`game/landMode.ts`) against a stand-in renderer and simulation: the land surveyed once,
// with the simulation's water mask; each stroke sent as a `shapeLand` command, the whole land with its header the
// first time (a tile not sent would read as flat and dry), then only the tiles it touched; the cover, zones and
// water mask when they changed; priced in Living, free in Creative, refunded on undo; a refused command taken back
// byte for byte with the simulation's reason told; and the commands merged as the simulation merges them giving
// back the land as shaped. `bun test tests/landMode.test.ts`.
import { describe, expect, test } from 'bun:test';
import { mergeShapeLand, type Rle, type ShapeLandCommand, type WorldMeta, type WorldStructure } from '../src/core/protocol';
import { blankLand, decodeLand, LAND_CELL, tilesOf, type Land } from '../src/game/land';
import type { LandPrices } from '../src/game/landEdit';
import { rle, unrle } from '../src/game/landMasks';
import { landCommand, LandMode, refusal, wetChanged, type LandModeHost } from '../src/game/landMode';
import type { Renderer } from '../src/render/types';
import { NO_LAND, type LandState } from '../src/ui/land/landTools';

const PRICES: LandPrices = { earth: 3, mountain: 0.02, water: 2, woods: 6, meadow: 1, fields: 2, clear: 1.5 };
/** A 256 m world: 128 × 128 samples, 2 × 2 tiles; a lot in its corner. */
const W = 256;

function setup(o: { creative?: boolean; meta?: WorldMeta; refuse?: (c: ShapeLandCommand) => string | null } = {}) {
  const land = blankLand(W, W);
  for (let j = 0; j < land.rows; j++) for (let i = 0; i < land.cols; i++) land.h[j * land.cols + i] = 100 + i;
  const sent: ShapeLandCommand[] = [];
  const told: string[] = [];
  let state: LandState = NO_LAND;
  let setLand: Land | null = null;
  let funds = 100_000;
  const world = { width: W, depth: W, plots: [{ id: 0, x: 0, z: 0, w: 30, d: 30 }], meta: o.meta ?? { zones: [] } } as unknown as WorldStructure;
  const renderer = {
    landOf: async () => land,
    setLand: (l: Land | null) => (setLand = l),
    landChanged: () => {},
    pickLand: () => null,
    setLandBrush: () => {},
    project: () => false,
    setLeftDragCamera: () => {},
  } as unknown as Renderer;
  const host: LandModeHost = {
    renderer,
    shapeLand: async (c) => {
      const why = o.refuse?.(c);
      if (why) throw new Error(why);
      sent.push(c);
      funds -= c.cost;
    },
    waterMask: async () => ({ water: [W * W, 0], width: W, depth: W }),
    world: () => world,
    household: () => 0,
    prices: () => PRICES,
    creative: () => !!o.creative,
    funds: () => funds,
    brush: () => ({ tool: 'raise', radius: 10, strength: 0.5 }),
    changed: (s) => (state = s),
    refused: (why) => told.push(why),
  };
  const mode = new LandMode(host);
  return { mode, land, sent, told, state: () => state, setLand: () => setLand, funds: () => funds };
}

const hold = (x: number, z: number, n = 30): [number, number][] => Array.from({ length: n }, () => [x, z]);

describe('the Land tab', () => {
  test('surveyed once: the renderer draws from the land the editor shapes', async () => {
    const t = setup();
    const e = await t.mode.ready();
    expect(e).not.toBeNull();
    expect(await t.mode.ready()).toBe(e);
    expect(t.setLand()).toBe(t.land);
    expect(t.state().ready).toBe(true);
  });

  test('the first stroke sends the whole land and its header; the next only its tiles; merged, they are the land', async () => {
    const t = setup();
    const e = (await t.mode.ready())!;
    const first = e.stroke({ tool: 'raise', radius: 10, strength: 0.5 }, hold(200, 200))!;
    await t.mode.commit(first);
    const [a] = t.sent;
    expect(a.meta.land!.cols).toBe(128);
    expect(Object.keys(a.meta.land!.h!).length).toBe(4);
    expect(a.cost).toBeGreaterThan(0);
    expect(a.water).toBeUndefined();
    expect(a.meta.zones).toBeUndefined();
    const second = e.stroke({ tool: 'lower', radius: 6, strength: 0.5 }, hold(20 + 100, 20, 10))!;
    await t.mode.commit(second);
    const b = t.sent[1];
    expect(b.meta.land!.cols).toBeUndefined();
    expect(Object.keys(b.meta.land!.h!).map(Number)).toEqual(tilesOf(second.rect, 128));
    let meta: WorldMeta | null = { zones: [] };
    for (const c of t.sent) meta = mergeShapeLand(meta, c.meta);
    const back = decodeLand(meta!.land!);
    expect(Buffer.from(back.h.buffer).equals(Buffer.from(t.land.h.buffer))).toBe(true);
    expect(t.state().canUndo).toBe(true);
  });

  test('a world that has its land already gets only the tiles from the first stroke on', async () => {
    const t = setup({ meta: { zones: [], land: { v: 1, cell: 2, cols: 128, rows: 128, tile: 64, h: [], wl: [], cover: [], seeded: 'drawn' } } });
    const e = (await t.mode.ready())!;
    await t.mode.commit(e.stroke({ tool: 'raise', radius: 10, strength: 0.5 }, hold(40 + 60, 40, 10))!);
    expect(t.sent[0].meta.land!.cols).toBeUndefined();
    expect(Object.keys(t.sent[0].meta.land!.h!)).toEqual(['0']);
  });

  test('a pond sends the water mask (the whole ground) and the zones; undo refunds what it cost', async () => {
    const t = setup();
    const e = (await t.mode.ready())!;
    const pond = e.stroke({ tool: 'lake', radius: 12, strength: 0.5 }, hold(180, 180, 4))!;
    expect(wetChanged(t.land, pond)).toBe(true);
    await t.mode.commit(pond);
    const c = t.sent[0];
    const mask = unrle(c.water as Rle, W * W);
    expect(mask.length).toBe(W * W);
    expect(mask[180 * W + 180]).toBe(1);
    expect(mask[10 * W + 10]).toBe(0);
    expect(c.meta.zones!.some((z) => z.kind === 'water')).toBe(true);
    expect(c.cost).toBeGreaterThan(0);
    const paid = t.funds();
    await t.mode.undo();
    const u = t.sent[1];
    expect(u.cost).toBe(-c.cost);
    expect(unrle(u.water as Rle, W * W)[180 * W + 180]).toBe(0);
    expect(t.funds()).toBe(paid + c.cost);
    await t.mode.redo();
    expect(t.sent[2].cost).toBe(c.cost);
  });

  test('painted woods send the cover and the zones, not the water', async () => {
    const t = setup();
    const e = (await t.mode.ready())!;
    const woods = e.stroke({ tool: 'woods', radius: 16, strength: 1 }, [[150, 150], [170, 150]])!;
    expect(wetChanged(t.land, woods)).toBe(false);
    await t.mode.commit(woods);
    const c = t.sent[0];
    expect(c.meta.land!.cover).toEqual(rle(t.land.cover));
    expect(c.meta.zones!.some((z) => z.kind === 'woods')).toBe(true);
    expect(c.water).toBeUndefined();
  });

  test('free in Creative', async () => {
    const t = setup({ creative: true });
    const e = (await t.mode.ready())!;
    await t.mode.commit(e.stroke({ tool: 'raise', radius: 10, strength: 0.5 }, hold(200, 200))!);
    expect(t.sent[0].cost).toBe(0);
  });

  test('refused: the stroke taken back byte for byte, nothing to redo, the reason told', async () => {
    const t = setup({ refuse: (c) => (c.cost > 0 ? `not enough money: shaping the land costs $${c.cost}` : null) });
    const e = (await t.mode.ready())!;
    const before = t.land.h.slice();
    await t.mode.commit(e.stroke({ tool: 'raise', radius: 10, strength: 0.5 }, hold(200, 200))!);
    expect(t.sent.length).toBe(0);
    expect(Buffer.from(t.land.h.buffer).equals(Buffer.from(before.buffer))).toBe(true);
    expect(e.canUndo || e.canRedo).toBe(false);
    expect(t.told[0]).toMatch(/^Not enough money: that would cost \$[\d,]+\.$/);
  });

  test('a refused undo puts the stroke back', async () => {
    let refuseUndo = false;
    const t = setup({ refuse: (c) => (refuseUndo && c.cost < 0 ? "water can't go where Ada is standing" : null) });
    const e = (await t.mode.ready())!;
    await t.mode.commit(e.stroke({ tool: 'raise', radius: 10, strength: 0.5 }, hold(200, 200))!);
    const after = t.land.h.slice();
    refuseUndo = true;
    await t.mode.undo();
    expect(Buffer.from(t.land.h.buffer).equals(Buffer.from(after.buffer))).toBe(true);
    expect(e.canUndo).toBe(true);
    expect(e.canRedo).toBe(false);
    expect(t.told[0]).toBe("Water can't go where Ada is standing.");
  });
});

describe('the command for a change', () => {
  test('only the tiles the change reaches, the cover when it changed', () => {
    const land = blankLand(W, W);
    const rect = { i0: 70, j0: 10, i1: 80, j1: 20 };
    const c = landCommand({ land, patch: { rect, h: new Int16Array(100), wl: new Int16Array(100) }, whole: false, household: 3, cost: 12 });
    expect(c).toMatchObject({ type: 'shapeLand', household: 3, cost: 12 });
    expect(Object.keys(c.meta.land!.h!)).toEqual(['1']);
    expect(c.meta.land!.cover).toBeUndefined();
    const painted = landCommand({ land, patch: { rect, cover: { rect: { i0: 35, j0: 5, i1: 40, j1: 10 }, data: new Uint8Array(25) } }, whole: false, household: 0, cost: 0 });
    expect(painted.meta.land!.cover).toEqual([land.cover.length, 0]);
    expect(LAND_CELL).toBe(2);
  });

  test("the simulation's reasons as the player reads them", () => {
    expect(refusal('not enough money: shaping the land costs $12345')).toBe('Not enough money: that would cost $12,345.');
    expect(refusal("water can't go on a lot (Home)")).toBe("Water can't go on a lot (Home).");
    expect(refusal('Error: water can\'t go across a road')).toBe("Water can't go across a road.");
  });
});
