/**
 * The backdrop's buildings near (`render/shared/backdropDress.ts`, `TownStreet` with `backdrop`): the game's street
 * draws every backdrop building of the chunks it draws, dressed on the box the town view draws it as, so a building
 * seen far off is there when the camera comes in. Its walls stand on its footprint (turned with it), its door faces
 * its road, a terrace has a door for each home, a café its awning; the street plans it into the chunk its middle is
 * in, and a building added to a chunk plans only that chunk again.
 */
import { describe, expect, test } from 'bun:test';
import type { BackdropBuilding, PlotInfo, RoadPlan, WorldStructure } from '../src/core/protocol';
import { dressBackdrop, frontSide, PANE_SETS, paneSetLit } from '../src/render/shared/backdropDress';
import { backdropBuilding } from '../src/render/shared/townBuildings';
import type { Geo } from '../src/render/shared/geometry';

const flat = () => 0;
/** The door knobs' brass (`Dresser.door`): one knob a door, 16 vertices a box (four faces: none against the wall, none under). */
const KNOB = [0xc8, 0xa6, 0x4a];

/** A `w` × `d` building of `use` turned `a` round (cx, cz), its ridge along `w`. */
function building(use: BackdropBuilding['use'], w: number, d: number, a = 0, cx = 100, cz = 50, roof: BackdropBuilding['roof'] = 'gable', storeys = 2): BackdropBuilding {
  const [ux, uz] = [Math.cos(a), Math.sin(a)];
  const at = (u: number, v: number): [number, number] => [cx + u * ux - v * uz, cz + u * uz + v * ux];
  // (Counter-clockwise seen from above, as the contract has it; `ridge` turns round +y: (cos r, −sin r) in x, z.)
  return { poly: [at(-w / 2, d / 2), at(w / 2, d / 2), at(w / 2, -d / 2), at(-w / 2, -d / 2)], y: 0, storeys, roof, ridge: -a, use, look: 1, area: 0 };
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

const part = (parts: { name: string; geo: Geo }[], name: string) => parts.find((p) => p.name === name)?.geo;

describe('a backdrop building dressed', () => {
  test('its walls stand on its footprint, turned with it, as high as the town view’s box; its roof as high as the box’s', () => {
    const a = Math.PI / 6;
    const b = building('house', 12, 8, a);
    const t = backdropBuilding(b, 7, flat);
    const { parts } = dressBackdrop(t, b.use, null);
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

  test('its door faces the road its way leaves for', () => {
    const b = building('house', 12, 8, 0);
    const t = backdropBuilding(b, 3, flat);
    // The road to the north (−z): the door on the box's −z side.
    expect(frontSide(t, [100, 20])).toBe(1);
    expect(frontSide(t, [100, 80])).toBe(0);
    expect(frontSide(t, [140, 50])).toBe(2);
    const { parts } = dressBackdrop(t, b.use, [100, 20]);
    const trim = part(parts, 'silhouetteTrim')!;
    const knobs = local(trim, t).filter((_, i) => {
      const c = trim.col;
      return Math.abs(c[i * 4] - KNOB[0]) <= 1 && Math.abs(c[i * 4 + 1] - KNOB[1]) <= 1 && Math.abs(c[i * 4 + 2] - KNOB[2]) <= 1;
    });
    expect(knobs.length).toBe(16);
    for (const [, v] of knobs) expect(v).toBeLessThan(-3.99);
  });

  test('a terrace is a row of homes, a door each; a café has its awning; a church its spire', () => {
    const row = building('terrace', 22, 9, 0.4);
    const t = backdropBuilding(row, 2, flat);
    const trim = part(dressBackdrop(t, row.use, null).parts, 'silhouetteTrim')!;
    expect(coloured(trim, KNOB) / 16).toBe(4);
    // A café: the awning's cloth, in its stripe and white.
    const cafe = building('cafe', 12, 9, 0, 100, 50, 'flat', 2);
    const ct = backdropBuilding(cafe, 5, flat);
    const cafeTrim = part(dressBackdrop(ct, cafe.use, [100, 80]).parts, 'silhouetteTrim')!;
    expect(coloured(cafeTrim, [0xf2, 0xee, 0xe6])).toBeGreaterThan(8);
    // A church: its spire over its roof.
    const church = building('church', 18, 10, 0, 100, 50, 'gable', 3);
    const ch = backdropBuilding(church, 9, flat);
    const roof = part(dressBackdrop(ch, church.use, null).parts, 'silhouetteRoofs')!;
    expect(Math.max(...local(roof, ch).map((q) => q[2]))).toBeGreaterThan(ch.top + ch.rise + 3);
  });

  test('flats and offices in render or brick; every window’s glass in a set lit by the share of the town view', () => {
    const flats = building('flats', 16, 11, 0, 100, 50, 'flat', 4);
    const t = backdropBuilding(flats, 1, flat);
    const out = dressBackdrop(t, flats.use, null);
    expect(out.parts.some((p) => p.name === 'backdropRender' || p.name === 'silhouetteBrick')).toBe(true);
    expect(out.panes.length).toBe(PANE_SETS);
    const quads = out.panes.reduce((n, g) => n + g.vertexCount / 4, 0);
    // Four storeys of windows round it.
    expect(quads).toBeGreaterThan(4 * 8);
    // More sets lit the more of the windows are.
    const lit = (share: number) => Array.from({ length: PANE_SETS }, (_, k) => paneSetLit(k, share)).filter(Boolean).length;
    expect([0, 0.2, 0.5, 0.8, 1].map(lit)).toEqual([0, 1, 2, 3, 4]);
  });
});

/** A road along x at z = 23, lots north of it (z 0..20) and an empty stretch south where the backdrop stands. */
function town(backdrop: BackdropBuilding[]): WorldStructure {
  const plots: PlotInfo[] = [];
  const paths: { x: number; z: number; w: number; d: number }[] = [];
  const facing: number[] = [];
  for (let i = 0; i < 8; i++) {
    const x = i * 20;
    plots.push({ id: i, name: `${i}`, x, z: 0, w: 20, d: 20, public: false, entry: [x + 8.5, 18], house: [x + 3, 5, x + 13, 14] } as PlotInfo);
    paths.push({ x: x + 8, z: 14, w: 1, d: 6 });
    facing.push(2);
  }
  const [w, d] = [160, 140];
  const road: RoadPlan = { points: [[-10, 23], [w + 10, 23]], width: 6, ends: ['out', 'out'] } as RoadPlan;
  return {
    width: w,
    depth: d,
    objects: [],
    plots,
    rooms: new Array(w * d).fill(0),
    openings: [],
    walls: [],
    households: [],
    sims: [],
    exits: [],
    meta: { kind: 'town', name: 'Test', streets: [], paths, facing, seed: 1, roads: [road], backdrop, places: [] },
  } as unknown as WorldStructure;
}

describe('the backdrop in the street', async () => {
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' }, requestAnimationFrame: () => 0, window: { addEventListener: () => {}, removeEventListener: () => {} } });
  const { HousePlanner } = await import('../src/render/shared/house');
  const { TownStreet, FOLIAGE } = await import('../src/render/shared/street');
  const { chunkAt } = await import('../src/render/shared/chunks');
  const options = { has: () => true, entry: () => undefined, lived: true, ground: true, backdrop: true };
  const houses = [building('house', 10, 8, 0, 30, 40), building('flats', 14, 10, 0, 110, 42, 'flat', 3)];

  test('each building is drawn in the chunk its middle is in, dressed, with what it stands on', () => {
    const street = new TownStreet(new HousePlanner(), town(houses), options, 'game');
    const ids = street.chunkIds();
    for (const [x, z] of [[30, 40], [110, 42]]) expect(ids).toContain(chunkAt(x, z));
    const plan = street.chunk(chunkAt(110, 42), -1);
    expect(plan.backdrop.length).toBe(1);
    expect(plan.backdrop[0].t.chunk).toBe(chunkAt(110, 42));
    expect(plan.silhouettes.parts.some((p) => p.name === 'backdropRender' || p.name === 'silhouetteBrick')).toBe(true);
    // The flats' forecourt; the house's garden path and its hedges.
    expect(plan.ground.some((g) => g.name === 'backdropCourts')).toBe(true);
    const home = street.chunk(chunkAt(30, 40), -1);
    expect(home.ground.some((g) => g.name === 'backdropPaths')).toBe(true);
    expect(home.batches.get(FOLIAGE.hedge)!.length).toBeGreaterThan(10);
    // The game's street without the backdrop (the menus' town) draws none of it.
    const none = new TownStreet(new HousePlanner(), town(houses), { ...options, backdrop: false }, 'game');
    expect(none.chunk(chunkAt(110, 42), -1).backdrop.length).toBe(0);
  });

  test('no lawn grows on what they stand on: a forecourt, a way to the road; a garden keeps its lawn', async () => {
    const { unlawned } = await import('../src/render/shared/nature');
    const world = town(houses);
    const bare = unlawned(world, false);
    const at = (x: number, z: number) => bare[Math.floor(z) * world.width + Math.floor(x)];
    // The flats' forecourt, 2 m round their walls (110 ± 7, 42 ± 5).
    expect(at(118.5, 42.5)).toBe(1);
    expect(at(110.5, 48.5)).toBe(1);
    // The house's garden beside it stays lawn; its path to the road (north of it, at z = 23) doesn't.
    expect(at(37.5, 40.5)).toBe(0);
    expect(at(30.5, 30.5)).toBe(1);
  });

  test('a building added plans only its chunk again', () => {
    const street = new TownStreet(new HousePlanner(), town(houses), options, 'game');
    const ids = street.chunkIds();
    const before = new Map(ids.map((id) => [id, street.chunkKey(id, -1)]));
    street.setWorld(town([...houses, building('shop', 9, 8, 0, 125, 60, 'flat', 2)]));
    const changed = ids.filter((id) => street.chunkKey(id, -1) !== before.get(id));
    expect(changed).toEqual([chunkAt(125, 60)]);
  });
});
