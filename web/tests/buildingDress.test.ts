/**
 * A lot's building closed near (`render/shared/buildingDress.ts`, `TownStreet`): the game's street draws a lot with a
 * look (a named place's, a flats building's) dressed on the box the town view draws it as, as tall as its real storeys,
 * so a building seen far off is there when the camera comes in. Its door faces its street, a terrace has a door for
 * each home, a café its awning, a place with flats over it the flats' own street door; open, the game draws its own
 * walls.
 */
import { describe, expect, test } from 'bun:test';
import type { BuildingUse, LotLook, Opening, PlotInfo } from '../src/core/protocol';
import { dressBuilding, frontSide, PANE_SETS, paneSetLit } from '../src/render/shared/buildingDress';
import { litAt, lotLookBuilding, MOST_STOREYS, STYLE } from '../src/render/shared/townBuildings';
import type { Geo } from '../src/render/shared/geometry';
import { BUILDING, realTown } from './fixtures/realTown';

const flat = () => 0;
/** The door knobs' brass (`Dresser.door`): one knob a door, 16 vertices a box (four faces: none against the wall, none under). */
const KNOB = [0xc8, 0xa6, 0x4a];

/** A `w` × `d` building of `use` closed round (cx, cz), along x. */
function building(use: BuildingUse, w: number, d: number, cx = 100, cz = 50, roof: LotLook['roof'] = 'gable', storeys = 2) {
  return lotLookBuilding([cx - w / 2, cz - d / 2, cx + w / 2, cz + d / 2], { use, storeys, roof, look: 1 }, storeys, 0, flat);
}

/** The vertices of `g` whose colour is `rgb`. */
function coloured(g: Geo, rgb: readonly number[]): number {
  const c = g.col;
  let n = 0;
  for (let i = 0; i < c.length; i += 4) if (Math.abs(c[i] - rgb[0]) <= 1 && Math.abs(c[i + 1] - rgb[1]) <= 1 && Math.abs(c[i + 2] - rgb[2]) <= 1) n++;
  return n;
}

/** Each vertex of `g` in a building's own frame: along it (u), across it (v), up (y). */
function local(g: Geo, t: { cx: number; cz: number; ux: number; uz: number }): [number, number, number][] {
  const p = g.pos;
  const out: [number, number, number][] = [];
  for (let i = 0; i < p.length; i += 3) {
    const [dx, dz] = [p[i] - t.cx, p[i + 2] - t.cz];
    out.push([dx * t.ux + dz * t.uz, -dx * t.uz + dz * t.ux, p[i + 1]]);
  }
  return out;
}

/** The door knobs of `trim` (town x, z of each vertex). */
function knobs(trim: Geo): [number, number][] {
  const out: [number, number][] = [];
  const c = trim.col;
  for (let i = 0; i < c.length / 4; i++) if (Math.abs(c[i * 4] - KNOB[0]) <= 1 && Math.abs(c[i * 4 + 1] - KNOB[1]) <= 1 && Math.abs(c[i * 4 + 2] - KNOB[2]) <= 1) out.push([trim.pos[i * 3], trim.pos[i * 3 + 2]]);
  return out;
}

const part = (parts: { name: string; geo: Geo }[], name: string) => parts.find((p) => p.name === name)?.geo;

describe('a building dressed', () => {
  test('its walls stand on its box, as high as the town view’s; its roof as high as the box’s', () => {
    const t = building('house', 12, 8);
    const { parts } = dressBuilding(t, 'house', null);
    const walls = part(parts, 'silhouetteWalls')!;
    expect(walls).toBeDefined();
    const pts = local(walls, t);
    expect(Math.max(...pts.map((q) => Math.abs(q[0])))).toBeCloseTo(6, 1);
    expect(Math.max(...pts.map((q) => Math.abs(q[1])))).toBeCloseTo(4, 1);
    // (Its walls up to the box's top; its gable ends on up to the ridge, the box's rise over it.)
    const roof = local(part(parts, 'silhouetteRoofs')!, t);
    const ridge = Math.max(...roof.map((q) => q[2]));
    expect(ridge).toBeGreaterThan(t.top + t.rise - 0.05);
    expect(ridge).toBeLessThan(t.top + t.rise + 0.4);
    // Nothing of it reaches more than its eaves' and its porch's width out of its footprint.
    for (const p of parts) for (const [u, v] of local(p.geo, t)) expect(Math.abs(u) <= 6 + 1.0 && Math.abs(v) <= 4 + 1.0).toBe(true);
  });

  test('its door faces its street', () => {
    const t = building('house', 12, 8);
    // The street to the north (−z): the door on the box's −z side.
    expect(frontSide(t, [100, 20])).toBe(1);
    expect(frontSide(t, [100, 80])).toBe(0);
    expect(frontSide(t, [140, 50])).toBe(2);
    const k = knobs(part(dressBuilding(t, 'house', [100, 20]).parts, 'silhouetteTrim')!);
    expect(k.length).toBe(16);
    for (const [, z] of k) expect(z).toBeLessThan(50 - 3.99);
  });

  test('a terrace is a row of homes, a door each; a café has its awning; a church its spire', () => {
    const row = building('terrace', 22, 9);
    expect(coloured(part(dressBuilding(row, 'terrace', null).parts, 'silhouetteTrim')!, KNOB) / 16).toBe(4);
    // A café: the awning's cloth, in its stripe and white.
    const cafe = building('cafe', 12, 9, 100, 50, 'flat', 2);
    expect(coloured(part(dressBuilding(cafe, 'cafe', [100, 80]).parts, 'silhouetteTrim')!, [0xf2, 0xee, 0xe6])).toBeGreaterThan(8);
    // A church: its spire over its roof.
    const church = building('church', 18, 10, 100, 50, 'gable', 3);
    const roof = part(dressBuilding(church, 'church', null).parts, 'silhouetteRoofs')!;
    expect(Math.max(...local(roof, church).map((q) => q[2]))).toBeGreaterThan(church.top + church.rise + 3);
  });

  test('flats and offices in render or brick; every window’s glass in a set lit by the share of the town view', () => {
    const t = building('flats', 16, 11, 100, 50, 'flat', MOST_STOREYS);
    const out = dressBuilding(t, 'flats', null);
    expect(out.parts.some((p) => p.name === 'buildingRender' || p.name === 'silhouetteBrick')).toBe(true);
    expect(out.panes.length).toBe(PANE_SETS);
    const quads = out.panes.reduce((n, g) => n + g.vertexCount / 4, 0);
    // Three storeys of windows round it.
    expect(quads).toBeGreaterThan(3 * 8);
    // More sets lit the more of the windows are.
    const lit = (share: number) => Array.from({ length: PANE_SETS }, (_, k) => paneSetLit(k, share)).filter(Boolean).length;
    expect([0, 0.2, 0.5, 0.8, 1].map(lit)).toEqual([0, 1, 2, 3, 4]);
  });

  test('a shop with flats over it: the flats’ own street door beside the shopfront, and flats’ windows over it', () => {
    const box: [number, number, number, number] = [90, 45, 110, 55];
    const shop = lotLookBuilding(box, { use: 'shop', storeys: 2, roof: 'flat', look: 1 }, 2, 0, flat, undefined, true);
    expect(shop.flats).toBe(true);
    const alone = lotLookBuilding(box, { use: 'shop', storeys: 2, roof: 'flat', look: 1 }, 2, 0, flat);
    // The street to the south (+z): the shop's door in the middle, the flats' at x 93 where its own is.
    const mixed = dressBuilding(shop, 'shop', [100, 80], [100, 55], [93, 55]);
    const k = knobs(part(mixed.parts, 'silhouetteTrim')!);
    expect(k.length).toBe(32);
    expect(k.filter(([x]) => Math.abs(x - 93) < 1).length).toBe(16);
    expect(knobs(part(dressBuilding(alone, 'shop', [100, 80], [100, 55]).parts, 'silhouetteTrim')!).length).toBe(16);
    // Upstairs, the flats' wider windows: fewer and wider than a shop's plain ones.
    const glass = (d: { panes: Geo[] }) => d.panes.reduce((n, g) => n + g.vertexCount / 4, 0);
    expect(glass(mixed)).not.toBe(glass(dressBuilding(alone, 'shop', [100, 80], [100, 55])));
  });

  test('a café with flats over it: its awning stops short of the flats’ door, which has a canopy of its own', () => {
    const box: [number, number, number, number] = [90, 45, 110, 55];
    const cafe = lotLookBuilding(box, { use: 'cafe', storeys: 2, roof: 'flat', look: 1 }, 2, 0, flat, undefined, true);
    const alone = lotLookBuilding(box, { use: 'cafe', storeys: 2, roof: 'flat', look: 1 }, 2, 0, flat);
    const cloth = (t: typeof cafe, flatsDoor: [number, number] | null) => {
      const trim = part(dressBuilding(t, 'cafe', [100, 80], [100, 55], flatsDoor).parts, 'silhouetteTrim')!;
      const out: number[] = [];
      for (let i = 0; i < trim.col.length / 4; i++) if (Math.abs(trim.col[i * 4] - 0xf2) <= 1 && Math.abs(trim.col[i * 4 + 1] - 0xee) <= 1 && Math.abs(trim.col[i * 4 + 2] - 0xe6) <= 1) out.push(trim.pos[i * 3]);
      return out;
    };
    // (The flats' door at x 93, 1.1 m wide: the white stripes keep 0.55 m off its frame either side.)
    const mixed = cloth(cafe, [93, 55]);
    expect(mixed.length).toBeGreaterThan(8);
    expect(mixed.some((x) => x > 93 - 0.55 - 0.55 + 0.01 && x < 93 + 0.55 + 0.55 - 0.01)).toBe(false);
    expect(cloth(alone, null).some((x) => Math.abs(x - 93) < 0.5)).toBe(true);
    // A canopy over the flats' door alone: its flat top (trim, 0.12 m thick) 1.5 m wider than the door.
    const trim = part(dressBuilding(cafe, 'cafe', [100, 80], [100, 55], [93, 55]).parts, 'silhouetteTrim')!;
    const top = 2.1 + 0.42;
    const canopy = local(trim, cafe).filter(([, v, y]) => Math.abs(y - top) < 1e-6 && v > 5.5);
    expect(canopy.length).toBeGreaterThan(0);
    expect(Math.min(...canopy.map(([u]) => u + 100))).toBeCloseTo(93 - 0.55 - 0.35, 5);
    expect(Math.max(...canopy.map(([u]) => u + 100))).toBeCloseTo(93 + 0.55 + 0.35, 5);
  });

  test('a place with flats over it: its flats’ glass in sets of their own, lit as homes', () => {
    const box: [number, number, number, number] = [90, 45, 110, 55];
    const hours = { open: 540, close: 1080, days: 127 };
    const shop = lotLookBuilding(box, { use: 'shop', storeys: 2, roof: 'flat', look: 1 }, 2, 0, flat, hours, true);
    const mixed = dressBuilding(shop, 'shop', [100, 80], [100, 55], [93, 55]);
    expect(mixed.panes.length).toBe(2 * PANE_SETS);
    // (The flats' sets hold the glass upstairs, over 3 m; the shop's the rest.)
    const ys = (gs: Geo[]) => gs.flatMap((g) => local(g, shop).map(([, , y]) => y));
    expect(Math.min(...ys(mixed.panes.slice(PANE_SETS)))).toBeGreaterThan(3);
    expect(Math.max(...ys(mixed.panes.slice(0, PANE_SETS)))).toBeLessThan(3);
    // At noon the shop is open, lit whole; at nine in the evening it is shut and its flats are lit, fewer of them in the
    // small hours.
    expect(litAt(shop, 12 * 60, 0, 'place')).toBe(shop.lit);
    expect(litAt(shop, 3 * 60, 0, 'flats')).toBeLessThan(litAt(shop, 21 * 60, 0, 'flats'));
    expect(litAt(shop, 21 * 60, 0, 'place')).toBeLessThan(0.1);
    expect(litAt(shop, 21 * 60, 0, 'flats')).toBe(STYLE.flats[1]);
    // (A building alone keeps its one set of sets.)
    expect(dressBuilding(lotLookBuilding(box, { use: 'shop', storeys: 2, roof: 'flat', look: 1 }, 2, 0, flat), 'shop', [100, 80]).panes.length).toBe(PANE_SETS);
  });
});

describe('a lot with a look closed in the street', async () => {
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' }, requestAnimationFrame: () => 0, window: { addEventListener: () => {}, removeEventListener: () => {} } });
  const { HousePlanner } = await import('../src/render/shared/house');
  const { TownStreet, lotDresses } = await import('../src/render/shared/street');
  const { chunkAt } = await import('../src/render/shared/chunks');
  const options = { has: () => true, entry: () => undefined, lived: true, ground: true };
  /** The flats building's middle (its walls 5..22 × 8..17) and its chunk. */
  const [mx, mz] = [13.5, 12.5];
  const id = chunkAt(mx, mz);
  const at = (b: { t: { cx: number } }) => Math.abs(b.t.cx - mx) < 0.01;

  test('a flats building: drawn as its look says, its real storeys, lit as homes; open, the game’s own', () => {
    const street = new TownStreet(new HousePlanner(), realTown(), options, 'game');
    const plan = street.chunk(id, -1);
    const mine = plan.buildings.filter(at);
    expect(mine.length).toBe(1);
    const t = mine[0].t;
    // Its walls' outer faces (5..22 × 8..17, half a wall out), two storeys, flats' windows, a home's lights.
    expect(t.w).toBeCloseTo(17.14, 2);
    expect(t.d).toBeCloseTo(9.14, 2);
    expect(t.top - t.floor).toBeCloseTo(6, 6);
    expect([t.roof, t.style, t.home]).toEqual(['flat', STYLE.flats[0], true]);
    expect(plan.silhouettes.parts.some((p) => p.name === 'buildingRender' || p.name === 'silhouetteBrick')).toBe(true);
    // Its street door where the building's is (13..14 on its front wall, z 17), facing the street.
    const k = knobs(plan.silhouettes.parts.find((p) => p.name === 'silhouetteTrim')!.geo).filter(([, z]) => z > 17);
    expect(k.length).toBe(16);
    for (const [x] of k) expect(Math.abs(x - 13.5)).toBeLessThan(1);
    // In view (the building's lot the focus): none of it, the game draws its walls.
    expect(street.chunk(id, BUILDING).buildings.some(at)).toBe(false);
    // Far off: the town view's lot carries its look.
    const far = street.townHouses(-1).find((h) => h.plot.id === BUILDING)!;
    expect(far.dress).toEqual({ look: { use: 'flats', storeys: 2, roof: 'flat', look: 3 }, flats: true });
    let top = -Infinity;
    const geo = street.farChunk(id, -1)!.geo;
    for (let i = 0; i < geo.pos.length; i += 3) if (geo.pos[i] > 4.5 && geo.pos[i] < 22.5) top = Math.max(top, geo.pos[i + 1]);
    expect(top).toBeCloseTo(6 + 0.7, 2);
  });

  test('a home’s lot has no look; a look taller than the building comes down to its real storeys', () => {
    const home = realTown();
    home.plots[0] = { ...home.plots[0], kind: 'home' } as PlotInfo;
    expect(lotDresses(home).size).toBe(0);
    const tall = realTown();
    tall.meta!.lotLooks = { [BUILDING]: { use: 'flats', storeys: 3, roof: 'flat', look: 3 } };
    const t = new TownStreet(new HousePlanner(), tall, options, 'game').chunk(id, -1).buildings.find(at)!.t;
    expect(t.top - t.floor).toBeCloseTo(6, 6);
  });

  test('a shop with flats over it: its own door and the flats’ beside it, lit by its hours and its flats', () => {
    const world = realTown();
    world.plots[0] = { ...world.plots[0], public: true, kind: 'shop' } as PlotInfo;
    world.openings = [...world.openings, { axis: 'h', x: 7, z: 17, kind: 'door' } as Opening];
    world.meta!.lotLooks = { [BUILDING]: { use: 'shop', storeys: 2, roof: 'flat', look: 3 } };
    world.meta!.places = [{ name: 'Mill Stores', kind: 'shop', x: 13.5, z: 17.5, plot: BUILDING, hours: { open: 540, close: 1080, days: 127 }, area: 0 }];
    expect(lotDresses(world).get(BUILDING)).toEqual({ look: world.meta!.lotLooks[BUILDING], hours: { open: 540, close: 1080, days: 127 }, flats: true });
    const plan = new TownStreet(new HousePlanner(), world, options, 'game').chunk(id, -1);
    const t = plan.buildings.find(at)!.t;
    expect([t.flats, t.hours?.open]).toEqual([true, 540]);
    const k = knobs(plan.silhouettes.parts.find((p) => p.name === 'silhouetteTrim')!.geo).filter(([, z]) => z > 17);
    expect(k.length).toBe(32);
    expect(k.filter(([x]) => Math.abs(x - 7.5) < 1).length).toBe(16);
  });

  test('its look changed plans only its chunk again', () => {
    const world = realTown();
    const street = new TownStreet(new HousePlanner(), world, options, 'game');
    const ids = street.chunkIds();
    const before = new Map(ids.map((c) => [c, street.chunkKey(c, -1)]));
    const next = realTown();
    next.meta!.lotLooks = { [BUILDING]: { use: 'flats', storeys: 2, roof: 'hip', look: 3 } };
    street.setWorld(next);
    expect(ids.filter((c) => street.chunkKey(c, -1) !== before.get(c))).toEqual([id]);
  });
});
