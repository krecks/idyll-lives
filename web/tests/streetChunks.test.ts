/**
 * The street planned chunk by chunk (`TownStreet`, `shared/chunks.ts`): a long street of houses
 * either side of a road, crossing several 64 m chunks, a taxi rank at each end of its rows. Moving the focus from lot to lot plans only
 * the chunks round the two lots again; a thing moved in a garden only its lot's chunk; and what a
 * street that has planned many focus lots gives is what a fresh one gives (its cache never keeps a
 * part it should have planned again).
 */
import { describe, expect, test } from 'bun:test';
import type { ObjectPlacement, PlotInfo, WorldStructure } from '../src/core/protocol';
import type { ChunkPlan } from '../src/render/shared/street';

const LOTS = 12;
const LOT_W = 20;

/** Lots along both sides of a road running along x at z = 23 (lots north z 0..20, south z 26..46). */
function town(objects: ObjectPlacement[] = []): WorldStructure {
  const plots: PlotInfo[] = [];
  const paths: { x: number; z: number; w: number; d: number }[] = [];
  const facing: number[] = [];
  for (let i = 0; i < LOTS; i++) {
    for (const north of [true, false]) {
      const x = i * LOT_W;
      const z = north ? 0 : 26;
      const id = plots.length;
      const house: [number, number, number, number] = north ? [x + 3, 5, x + 13, 14] : [x + 4, 31, x + 14, 40];
      const door = x + 8;
      plots.push({ id, name: `${id}`, x, z, w: LOT_W, d: 20, public: i === 5 && north, entry: [door + 0.5, north ? 18 : 28], house: i === 5 && north ? undefined : house } as PlotInfo);
      paths.push(north ? { x: door, z: 14, w: 1, d: 6 } : { x: door, z: 26, w: 1, d: 5 });
      paths.push(north ? { x: x + 14.6, z: 9.6, w: 3, d: 10.4 } : { x: x + 15.6, z: 26, w: 3, d: 10.4 });
      facing.push(north ? 2 : 0);
    }
  }
  const [w, d] = [LOTS * LOT_W, 46];
  return {
    width: w,
    depth: d,
    objects,
    plots,
    rooms: new Array(w * d).fill(0),
    openings: [],
    walls: [],
    households: [],
    sims: [],
    exits: [],
    meta: { kind: 'town', name: 'Test', streets: [], paths, facing, seed: 1, roads: [{ points: [[-10, 23], [w + 10, 23]], width: 6, ends: ['out', 'out'] }] },
  } as unknown as WorldStructure;
}

/** Streets of houses along x at each of `roads` (z), lots either side (20 deep), as `town` has one. */
function streets(roads: readonly number[]): WorldStructure {
  const base = town();
  const plots: PlotInfo[] = [];
  const paths: { x: number; z: number; w: number; d: number }[] = [];
  const facing: number[] = [];
  for (const rz of roads) {
    for (let i = 0; i < LOTS; i++) {
      for (const north of [true, false]) {
        const x = i * LOT_W;
        const z = north ? rz - 23 : rz + 3;
        const id = plots.length;
        const house: [number, number, number, number] = north ? [x + 3, z + 5, x + 13, z + 14] : [x + 4, z + 5, x + 14, z + 14];
        const door = x + 8;
        plots.push({ id, name: `${id}`, x, z, w: LOT_W, d: 20, public: false, entry: [door + 0.5, north ? z + 18 : z + 2], house } as PlotInfo);
        paths.push(north ? { x: door, z: z + 14, w: 1, d: 6 } : { x: door, z, w: 1, d: 5 });
        facing.push(north ? 2 : 0);
      }
    }
  }
  const depth = Math.max(...roads) + 23;
  return {
    ...base,
    depth,
    plots,
    rooms: new Array(base.width * depth).fill(0),
    meta: { ...base.meta, paths, facing, roads: roads.map((z) => ({ points: [[-10, z], [base.width + 10, z]], width: 6, ends: ['out', 'out'] })) },
  } as unknown as WorldStructure;
}

const thing = (id: number, x: number, z: number): ObjectPlacement => ({ id, def: 'gardenBench', x, z, w: 2, d: 1, rot: 0 }) as ObjectPlacement;
const options = { has: () => true, entry: () => undefined, lived: true, ground: true, look: () => ({ model: 'model.gardenBench', footprint: [2, 1] as [number, number], standIn: 'box' as never }) };

/** What a chunk draws, as text (vertex positions rounded, instances), to compare two plans of it. */
function drawn(c: ChunkPlan): string {
  const r = (v: number) => (Math.round(v * 1000) / 1000 + 0).toFixed(3);
  const geo = (pos: number[]) => pos.map(r).join(',');
  return JSON.stringify([
    c.silhouettes.parts.map((p) => [p.name, geo(p.geo.pos)]),
    c.silhouettes.homes.map((h) => geo(h.panes.pos)),
    [c.walk, c.props, c.glow, c.pools, c.shadows].map((g) => geo(g.pos)),
    c.lot && geo(c.lot.props.pos),
    [...c.batches].map(([k, l]) => [k, l.map((i) => i.map((v) => r(v ?? 0)).join(' '))]),
    c.ground.map((g) => [g.name, geo(g.geo.pos)]),
  ]);
}

describe('the street in chunks', async () => {
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' }, requestAnimationFrame: () => 0, window: {} });
  const { HousePlanner } = await import('../src/render/shared/house');
  const { TownStreet, NO_FOCUS } = await import('../src/render/shared/street');
  const { chunkAt } = await import('../src/render/shared/chunks');
  const objects = [thing(1, 44, 2), thing(2, 61, 30), thing(3, 141, 3)];

  test('the town spans chunks, and every one has its share', () => {
    const street = new TownStreet(new HousePlanner(), town(objects), options, 'game');
    const ids = street.chunkIds();
    expect(ids.length).toBeGreaterThanOrEqual(4);
    for (const id of ids) {
      const c = street.chunk(id, NO_FOCUS);
      // (A road running out of town: a street sign past its end is a chunk's all.)
      expect(c.props.pos.length + c.walk.pos.length + c.ground.length + [...c.batches.values()].reduce((n, l) => n + l.length, 0)).toBeGreaterThan(0);
    }
    expect(ids.filter((id) => street.chunk(id, NO_FOCUS).specs.length).length).toBeGreaterThanOrEqual(4);
  });

  test('a focus moving to the next lot plans only the chunks round the two lots again', () => {
    const world = town(objects);
    const street = new TownStreet(new HousePlanner(), world, options, 'game');
    const ids = street.chunkIds();
    let keys = new Map(ids.map((id) => [id, street.chunkKey(id, 0)]));
    for (let f = 2; f < world.plots.length; f += 2) {
      const now = new Map(ids.map((id) => [id, street.chunkKey(id, f)]));
      const changed = ids.filter((id) => keys.get(id) !== now.get(id));
      const near = new Set([f - 2, f].map((p) => chunkAt(world.plots[p].x + 10, world.plots[p].z + 10)));
      // (The lots beside each keep their fences clear of it: their chunks, if another.)
      for (const p of [f - 3, f - 1, f + 1, f + 2]) if (world.plots[p]) near.add(chunkAt(world.plots[p].x + 10, world.plots[p].z + 10));
      for (const id of changed) expect(near.has(id)).toBe(true);
      expect(changed.length).toBeLessThanOrEqual(3);
      keys = now;
    }
  });

  test('the focus lot is the game’s own: no house of its own in its chunk, its garden left out', () => {
    const world = town(objects);
    const street = new TownStreet(new HousePlanner(), world, options, 'game');
    const id = chunkAt(world.plots[4].x + 10, 10);
    const without = street.chunk(id, 4);
    const withIt = street.chunk(id, NO_FOCUS);
    expect(without.specs.length).toBe(withIt.specs.length - 1);
    expect(without.props.pos.length).toBeLessThan(withIt.props.pos.length);
  });

  test('far off, a chunk’s houses are boxes with roofs, the focus lot’s left out', () => {
    const world = town(objects);
    const street = new TownStreet(new HousePlanner(), world, options, 'game');
    const id = chunkAt(world.plots[4].x + 10, 10);
    const all = street.farChunk(id, NO_FOCUS)!;
    const without = street.farChunk(id, 4)!;
    expect(all.geo.pos.length).toBeGreaterThan(0);
    expect(without.geo.pos.length).toBeLessThan(all.geo.pos.length);
    // (Kept: asked again, the same.)
    expect(street.farChunk(id, NO_FOCUS)!.key).toBe(all.key);
    // A box and a roof: a few dozen triangles a house, not a silhouette's hundreds.
    const houses = world.plots.filter((p) => p.house && chunkAt(p.x + 10, p.z + 10) === id).length;
    expect(all.geo.idx.length / 3 / houses).toBeLessThan(40);
  });

  test('a thing moved in a garden plans only its lot’s chunk again', () => {
    const street = new TownStreet(new HousePlanner(), town(objects), options, 'game');
    const ids = street.chunkIds();
    const before = new Map(ids.map((id) => [id, street.chunkKey(id, 0)]));
    const moved = town([thing(1, 46, 3), objects[1], objects[2]]);
    street.setWorld(moved);
    const changed = ids.filter((id) => before.get(id) !== street.chunkKey(id, 0));
    expect(changed).toEqual([chunkAt(50, 10)]);
  });

  test('planned after many focus lots and edits, a chunk is what a fresh street plans', () => {
    const planner = new HousePlanner();
    const world = town(objects);
    const street = new TownStreet(planner, world, options, 'game');
    const focuses = [0, 9, 10, 11, 23, 4, NO_FOCUS, 10, 5];
    let w = world;
    for (const [i, f] of focuses.entries()) {
      if (i === 4) {
        w = town([thing(1, 44, 2), thing(2, 63, 31), thing(3, 141, 3), thing(4, 100, 17)]);
        street.setWorld(w);
      }
      for (const id of street.chunkIds()) street.chunk(id, f);
      const fresh = new TownStreet(new HousePlanner(), w, options, 'game');
      for (const id of street.chunkIds()) expect(drawn(street.chunk(id, f))).toBe(drawn(fresh.chunk(id, f)));
    }
  });


  /** The ranks' shelters a street draws (every chunk's), and its own account of them. */
  const ranksOf = (street: InstanceType<typeof TownStreet>) => ({
    drawn: street.chunkIds().flatMap((id) => street.chunk(id, NO_FOCUS).batches.get('model.busStop') ?? []).map((i) => [i[0], i[1]] as [number, number]),
    signs: street.chunkIds().flatMap((id) => street.chunk(id, NO_FOCUS).batches.get('model.street.taxiSign') ?? []),
    spots: street.rankSpots(),
  });

  test('a street gets one taxi rank at most, past the end of its row behind the pavement, its sign at the kerb', () => {
    const { drawn, signs, spots } = ranksOf(new TownStreet(new HousePlanner(), town(), options, 'game'));
    expect(drawn.length).toBe(1);
    expect(signs.length).toBe(1);
    expect(spots).toEqual(drawn);
    const [x, z] = drawn[0];
    // Past the row's ends (the lots run from 0 to its length), off the road (z 20..26 with its pavements).
    expect(x < 0 || x > LOTS * LOT_W).toBe(true);
    expect(z < 20 || z > 26).toBe(true);
  });

  test('no two taxi ranks within 60 m, the town’s own on a lot among them, chosen the same each time', async () => {
    const { RANK_APART } = await import('../src/render/shared/street');
    expect(RANK_APART).toBe(60);
    // Three streets of houses 46 m apart: each could have a rank at either end, two ends 46 m from the next street's.
    const w = streets([23, 69, 115]);
    const a = ranksOf(new TownStreet(new HousePlanner(), w, options, 'game'));
    expect(a.drawn.length).toBeGreaterThanOrEqual(2);
    expect(a.drawn.length).toBeLessThanOrEqual(3);
    const apart = (list: [number, number][]) => {
      for (let i = 0; i < list.length; i++) for (let j = i + 1; j < list.length; j++) expect(Math.hypot(list[i][0] - list[j][0], list[i][1] - list[j][1])).toBeGreaterThanOrEqual(60);
    };
    apart(a.drawn);
    // One a street: each rank by its street (the nearest road).
    const roadOf = ([, z]: [number, number]) => [23, 69, 115].reduce((b, r) => (Math.abs(r - z) < Math.abs(b - z) ? r : b));
    expect(new Set(a.drawn.map(roadOf)).size).toBe(a.drawn.length);
    // The same town again (and with another lot in view) chooses the same.
    const b = new TownStreet(new HousePlanner(), w, options, 'game');
    expect(b.rankSpots()).toEqual(a.spots);
    const viewed = b.chunkIds().flatMap((id) => b.chunk(id, 7).batches.get('model.busStop') ?? []).map((i) => [i[0], i[1]]);
    expect(viewed).toEqual(a.drawn);
    // A rank on a lot (the Market Square's shelter) keeps the row ends' 60 m off.
    for (const [x, z] of a.drawn) {
      const own = { ...w, objects: [{ id: 99, def: 'busStop', x: x + 10, z: z > 50 ? z + 2 : z - 2, w: 3, d: 1, rot: 0 } as ObjectPlacement] } as WorldStructure;
      const c = ranksOf(new TownStreet(new HousePlanner(), own, options, 'game'));
      for (const [rx, rz] of c.drawn) expect(Math.hypot(rx - (x + 11.5), rz - (z > 50 ? z + 2.5 : z - 1.5))).toBeGreaterThanOrEqual(60);
      apart(c.drawn);
    }
  });
});
