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
/** The street of `town(objects)` drawn, hashed: as it was before land that slopes (see the test). Taken again when the neighbours' silhouettes, the parked cars or the kerbs change (last: the walls', posts' and roofs' tops eased, the mailboxes' lids and the bridges' noses). */
const FLAT_STREET = 2124564854;
const FLAT_STREET_FOCUS = 3672397484;
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
  const geo = (pos: ArrayLike<number>) => Array.from(pos, r).join(',');
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
  // (A window the camera's tests can use too, run in the same process: listeners taken and dropped.)
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' }, requestAnimationFrame: () => 0, window: { addEventListener: () => {}, removeEventListener: () => {} } });
  const { HousePlanner } = await import('../src/render/shared/house');
  const { TownStreet, NO_FOCUS } = await import('../src/render/shared/street');
  const { chunkAt, chunkRect } = await import('../src/render/shared/chunks');
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
    // (Its house the game's: only its garage and porch here, `dressingOnly`.)
    expect(without.specs.filter((s) => !s.dressingOnly).length).toBe(withIt.specs.length - 1);
    expect(without.specs.filter((s) => s.dressingOnly).map((s) => s.lot.x)).toEqual([world.plots[4].x]);
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

  test('a porch or a porch piece built or taken down plans only its lot’s chunk again, and draws it', () => {
    // (The same world, its porches changed in place as a world message merges into it: `mergeWorld`.)
    const world: WorldStructure = town(objects);
    // (Fence styles kept as the game keeps them: the same list each time.)
    const opts = { ...options, fenceStyles: [] };
    const street = new TownStreet(new HousePlanner(), world, opts, 'game');
    street.setWorld(world);
    const ids = street.chunkIds();
    const before = new Map(ids.map((id) => [id, street.chunkKey(id, 0)]));
    const focus = drawn(street.chunk(chunkAt(30, 10), 0));
    // Lot 2 (x 20..40, north of the road): a porch over its front door, a post in its garden.
    world.porches = [{ plot: 2, axis: 'h', x: 28, z: 14, out: 1 }];
    world.porchPieces = [{ plot: 2, kind: 'post', x: 22, z: 16, w: 1, d: 1, rot: 0 }];
    street.setWorld(world);
    const changed = ids.filter((id) => before.get(id) !== street.chunkKey(id, 0));
    expect(changed).toEqual([chunkAt(30, 10)]);
    const built = drawn(street.chunk(chunkAt(30, 10), 0));
    expect(built).not.toBe(focus);
    expect(built).toBe(drawn(new TownStreet(new HousePlanner(), { ...world }, opts, 'game').chunk(chunkAt(30, 10), 0)));
    // Taken down again: as before.
    delete world.porches;
    delete world.porchPieces;
    street.setWorld(world);
    expect(ids.filter((id) => before.get(id) !== street.chunkKey(id, 0))).toEqual([]);
    expect(drawn(street.chunk(chunkAt(30, 10), 0))).toBe(focus);
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
  }, 30_000);

  test('a street that lets go of the parts of chunks not drawn keeps their keys and plans them again as a fresh street does', () => {
    const world = town(objects);
    const street = new TownStreet(new HousePlanner(), world, options, 'game');
    const ids = street.chunkIds();
    for (const id of ids) street.chunk(id, 0);
    const keys = new Map(ids.map((id) => [id, street.chunkKey(id, 0)]));
    // Only the first chunk's parts kept (and none for another focus lot): every key the same.
    const units = (street as unknown as { plotUnits: Map<string, unknown> }).plotUnits;
    const before = units.size;
    street.keepUnits(new Set([ids[0]]), 0);
    expect(units.size).toBeLessThan(before);
    for (const id of ids) expect(street.chunkKey(id, 0)).toBe(keys.get(id)!);
    // A focus elsewhere, a thing moved, nothing kept at all: each chunk as a fresh street plans it.
    const moved = town([thing(1, 46, 3), objects[1], objects[2]]);
    street.setWorld(moved);
    for (const [kept, f] of [[[ids[0]], 9], [[], 10], [ids, NO_FOCUS]] as const) {
      for (const id of ids) street.chunk(id, f);
      street.keepUnits(new Set(kept), f);
      const fresh = new TownStreet(new HousePlanner(), moved, options, 'game');
      for (const id of ids) expect(drawn(street.chunk(id, f))).toBe(drawn(fresh.chunk(id, f)));
    }
  }, 30_000);


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

  test('round a taxi rank every kerbside thing keeps a clear gap from its shelter, its sign and each other; the corner keeps its street-name sign', async () => {
    const { RANK_GAP, RANK_HALF } = await import('../src/render/shared/street');
    for (const w of [town(), streets([23, 69, 115])]) {
      const street = new TownStreet(new HousePlanner(), w, options, 'game');
      const roads = (w.meta as { roads: unknown[] }).roads.length;
      let seen = 0;
      for (let r = 0; r < roads; r++) {
        for (const side of [1, -1] as const) {
          const rank = street.rankKerb(r, side);
          if (!rank) continue;
          seen++;
          const { shelter, sign, others } = rank;
          // (The shelter's ends along the kerb, and its sign: a gap of RANK_GAP clear of each.)
          const gap = (s: number) => Math.abs(s - shelter) - RANK_HALF;
          if (sign !== null) expect(gap(sign)).toBeGreaterThanOrEqual(RANK_GAP);
          for (const o of others) {
            expect(gap(o)).toBeGreaterThanOrEqual(RANK_GAP);
            if (sign !== null) expect(Math.abs(o - sign)).toBeGreaterThanOrEqual(RANK_GAP);
          }
          // The kerbside things beside it, those within a couple of gaps of its ends, clear of each other too.
          const close = others.filter((o) => gap(o) < 3 * RANK_GAP).sort((a, b) => a - b);
          for (let i = 1; i < close.length; i++) expect(close[i] - close[i - 1]).toBeGreaterThanOrEqual(RANK_GAP);
        }
      }
      expect(seen).toBe(street.rankSpots().length);
      expect(seen).toBeGreaterThan(0);
      // A rank at the corner of a row with its street-name sign (the road's first side, its first end):
      // the name sign still there, its TAXI sign well clear of it, at the shelter's far end.
      const get = (k: string) => street.chunkIds().flatMap((id) => street.chunk(id, NO_FOCUS).batches.get(k) ?? []);
      const names = get('model.street.sign');
      const signs = get('model.street.taxiSign');
      for (const [x, z] of get('model.busStop')) {
        if (x > 0) continue;
        const name = names.find((n) => Math.hypot(n[0] - x, n[1] - z) < 6);
        expect(name).toBeDefined();
        for (const t of signs) expect(Math.hypot(t[0] - name![0], t[1] - name![1])).toBeGreaterThanOrEqual(RANK_HALF + 2 * RANK_GAP);
      }
    }
  });

  // ---- on a land the town follows (`landSurface.ts`), and on one it doesn't --------------------------------------

  /** Every chunk of a street as drawn, for focus `focus`, hashed (`drawn`). */
  const hashOf = (street: InstanceType<typeof TownStreet>, focus = NO_FOCUS) => {
    let h = 2166136261;
    for (const id of street.chunkIds()) for (const ch of drawn(street.chunk(id, focus))) h = Math.imul(h ^ ch.charCodeAt(0), 16777619);
    return h >>> 0;
  };
  const { blankLand, encodeLand, LAND_CELL } = await import('../src/game/land');
  const { surfaceOf } = await import('../src/render/shared/landSurface');
  const { ROAD_Y } = await import('../src/render/shared/roads');
  /** `world` on a land rising 6 % along x and 3 % along z, its town following it (or drawn, `seeded`). */
  const onLand = (world: WorldStructure, seeded: 'follow' | 'drawn' = 'follow'): WorldStructure => {
    const land = blankLand(world.width, world.depth, seeded);
    for (let j = 0; j < land.rows; j++) for (let i = 0; i < land.cols; i++) land.h[j * land.cols + i] = Math.round((0.06 * i + 0.03 * j) * LAND_CELL * 100);
    return { ...world, meta: { ...world.meta!, land: encodeLand(land) } };
  };

  test('a world its town doesn’t follow draws as it always has', () => {
    // (Pinned before land that slopes: the street of a world without one, or with one drawn flat under its town.)
    const flat = hashOf(new TownStreet(new HousePlanner(), town(objects), options, 'game'));
    expect(flat).toBe(FLAT_STREET);
    expect(hashOf(new TownStreet(new HousePlanner(), onLand(town(objects), 'drawn'), options, 'game'))).toBe(flat);
    expect(hashOf(new TownStreet(new HousePlanner(), town(objects), options, 'game'), 4)).toBe(FLAT_STREET_FOCUS);
  });

  test('on a land the town follows, each lot’s share stands on its pad, the road and its kerbside things on the road', () => {
    const world = onLand(town(objects));
    const surface = surfaceOf(world)!;
    expect(surface).not.toBeNull();
    const flat = new TownStreet(new HousePlanner(), town(objects), options, 'game');
    const street = new TownStreet(new HousePlanner(), world, options, 'game');
    // A house's walls: as on flat land, its pad higher.
    const lot = 16;
    const id = chunkAt(world.plots[lot].x + 10, world.plots[lot].z + 10);
    const pad = surface.padY(lot);
    expect(pad).toBeGreaterThan(5);
    const minY = (c: ChunkPlan, x0: number, x1: number, z0: number, z1: number) => {
      let low = Infinity;
      for (const part of c.silhouettes.parts) {
        const p = part.geo.pos;
        for (let k = 0; k < p.length; k += 3) if (p[k] >= x0 && p[k] <= x1 && p[k + 2] >= z0 && p[k + 2] <= z1) low = Math.min(low, p[k + 1]);
      }
      return low;
    };
    const P = world.plots[lot];
    const [a, b] = [flat.chunk(id, NO_FOCUS), street.chunk(id, NO_FOCUS)];
    expect(minY(b, P.x, P.x + P.w, P.z, P.z + P.d) - minY(a, P.x, P.x + P.w, P.z, P.z + P.d)).toBeCloseTo(pad, 4);
    // A thing in a garden lifted by its lot's pad (lot 4's bench at x 44).
    const bench = (st: InstanceType<typeof TownStreet>) => st.chunk(chunkAt(44, 2), NO_FOCUS).batches.get('model.gardenBench')!.find(([x]) => x > 40 && x < 50)!;
    expect((bench(street)[5] ?? 0) - (bench(flat)[5] ?? 0)).toBeCloseTo(surface.padY(4), 4);
    // The road's surface on the road's height (`ROAD_Y` over it), every vertex.
    const road = world.meta!.roads![0];
    for (const cid of street.chunkIds()) {
      const g = street.chunk(cid, NO_FOCUS).ground.find((l) => l.name === 'streets');
      if (!g) continue;
      const p = g.geo.pos;
      for (let k = 0; k < p.length; k += 3) {
        const s = p[k] + 10;
        if (s < 0.5 || s > road.points[1][0] + 9.5) continue;
        expect(Math.abs(p[k + 1] - (surface.roadY(0, s) + ROAD_Y))).toBeLessThan(0.01);
      }
    }
    // The kerbside things on the sidewalk where they stand: the street's signs at the road's height there.
    const signs = street.chunkIds().flatMap((cid) => street.chunk(cid, NO_FOCUS).batches.get('model.street.sign') ?? []);
    expect(signs.length).toBeGreaterThan(0);
    for (const [x, z, , , , y] of signs) expect(y ?? 0).toBeCloseTo(surface.at(x, z), 4);
    // Lifted, not moved: the same houses' vertices across, their heights the only change; the props as many.
    const xs = (c: ChunkPlan) => c.silhouettes.parts.map((q) => q.geo.pos.filter((_, k) => k % 3 !== 1).join(','));
    expect(xs(b)).toEqual(xs(a));
    for (const cid of street.chunkIds()) expect(street.chunk(cid, NO_FOCUS).props.pos.length).toBe(flat.chunk(cid, NO_FOCUS).props.pos.length);
  });

  test('a chunk whose heights moved is planned again (its key has the heights in it)', () => {
    const world = onLand(town(objects));
    const surface = surfaceOf(world)!;
    const street = new TownStreet(new HousePlanner(), world, options, 'game');
    const id = chunkAt(world.plots[16].x + 10, 10);
    const key = street.chunkKey(id, NO_FOCUS);
    const land = surface.land;
    // The land under lot 16 and its road lowered (a stroke the brushes may make from stage E): its pad and road move.
    for (let j = 0; j < land.rows; j++) for (let i = 70; i < 95 && i < land.cols; i++) land.h[j * land.cols + i] -= 300;
    surface.update({ i0: 70, j0: 0, i1: 95, j1: land.rows });
    expect(surface.version(id)).toBeGreaterThan(0);
    expect(street.chunkKey(id, NO_FOCUS)).not.toBe(key);
  });

  /** `world` without the lot at index `gone`: the lots after it numbered again, their paths and facings with them (as the editor removes one). */
  const without = (world: WorldStructure, gone: number): WorldStructure => {
    const meta = world.meta!;
    return {
      ...world,
      plots: world.plots.filter((_, i) => i !== gone).map((p, i) => ({ ...p, id: i, name: `${i}` })),
      meta: { ...meta, paths: meta.paths!.filter((_, k) => Math.floor(k / 2) !== gone), facing: meta.facing!.filter((_, i) => i !== gone) },
    } as WorldStructure;
  };

  test('another town with a lot removed: the chunks away from it keep their keys, and draw the same (its lots by place, not by index)', () => {
    const world = town(objects);
    // (A lot in the middle of the south row: every lot after it gets another index.)
    const gone = 13;
    const lot = world.plots[gone];
    const a = new TownStreet(new HousePlanner(), world, options, 'town');
    const b = new TownStreet(new HousePlanner(), without(world, gone), options, 'town');
    let kept = 0;
    for (const id of a.chunkIds()) {
      const r = chunkRect(id);
      const near = r.x < lot.x + lot.w + 64 && r.x + r.w > lot.x - 64;
      const same = a.chunkKey(id, NO_FOCUS) === b.chunkKey(id, NO_FOCUS);
      if (!near) expect(same).toBe(true);
      // (A key the same is a chunk drawn the same: what lets the overview keep it.)
      if (same) {
        expect(drawn(b.chunk(id, NO_FOCUS))).toBe(drawn(a.chunk(id, NO_FOCUS)));
        kept++;
      }
    }
    expect(kept).toBeGreaterThan(0);
    // Its own chunk is planned again.
    expect(a.chunkKey(chunkAt(lot.x + lot.w / 2, lot.z + lot.d / 2), NO_FOCUS)).not.toBe(b.chunkKey(chunkAt(lot.x + lot.w / 2, lot.z + lot.d / 2), NO_FOCUS));
  });

  test('a street handed on from the town before a lot was removed plans as a fresh one, the street’s own and most kerbs taken over', () => {
    // (The menus' parked cars: one on each driveway, the removed lot's going with it.)
    const parked = (w: WorldStructure, gone = -1): WorldStructure => ({ ...w, cars: w.plots.flatMap((p, i) => (i === gone || p.public ? [] : [{ plot: i, x: p.x + 16, z: p.z + 10, yaw: 0, body: i % 3, paint: i % 4 }])) });
    for (const world of [town(objects), onLand(town(objects)), parked(onLand(town(objects)))]) {
      for (const ground of [false, true]) {
        const opts = { ...options, ground };
        const gone = 13;
        const before = new TownStreet(new HousePlanner(), world, opts, 'town');
        // (Planned as the overview plans it: every chunk's key, every road side's kerbs.)
        for (const id of before.chunkIds()) before.chunkKey(id, NO_FOCUS);
        const after = world.cars ? parked(without(world, gone), gone) : without(world, gone);
        const fresh = new TownStreet(new HousePlanner(), after, opts, 'town');
        const handed = new TownStreet(new HousePlanner(), after, opts, 'town', before);
        expect(handed.handedOn.statics).toBe(true);
        expect(handed.handedOn.kerbs).toBeGreaterThan(0);
        expect(handed.chunkIds()).toEqual(fresh.chunkIds());
        for (const id of fresh.chunkIds()) {
          expect(handed.chunkKey(id, NO_FOCUS)).toBe(fresh.chunkKey(id, NO_FOCUS));
          expect(drawn(handed.chunk(id, NO_FOCUS))).toBe(drawn(fresh.chunk(id, NO_FOCUS)));
        }
      }
    }
    // Another road: nothing handed on.
    const one = streets([23]);
    const two = streets([23, 120]);
    const a = new TownStreet(new HousePlanner(), one, options, 'town');
    for (const id of a.chunkIds()) a.chunkKey(id, NO_FOCUS);
    const b = new TownStreet(new HousePlanner(), two, options, 'town', a);
    expect(b.handedOn).toEqual({ statics: false, kerbs: 0 });
  });

  test('another town with a road added: the chunks it does not reach keep their keys (the menus’ street, which draws no road surface)', () => {
    const one = streets([23]);
    const two = streets([23, 120]);
    const overview = { ...options, ground: false };
    const a = new TownStreet(new HousePlanner(), one, overview, 'town');
    const b = new TownStreet(new HousePlanner(), { ...two, plots: one.plots, depth: two.depth, rooms: two.rooms, meta: { ...two.meta!, paths: one.meta!.paths, facing: one.meta!.facing } } as WorldStructure, overview, 'town');
    for (const id of a.chunkIds()) {
      const r = chunkRect(id);
      if (r.z + r.d > 64) continue;
      expect(b.chunkKey(id, NO_FOCUS)).toBe(a.chunkKey(id, NO_FOCUS));
    }
  });

  /**
   * `town`'s lots on open land 400 by 200 m, the road running on past them to x 390; `country`: with a country road
   * off it at x 320, south across the open land (a T, no lot along it: as step 2's Street tool draws one out of town).
   */
  const withCountry = (country: boolean): WorldStructure => {
    const base = town(objects);
    const [w, d] = [400, 200];
    const main = { points: [[-10, 23], [390, 23]], width: 6, ends: ['out', 'out'] };
    const out = { points: [[320, 26], [320, 110], [330, 190]], width: 5, ends: ['join', 'out'], kind: 'country' };
    return { ...base, width: w, depth: d, rooms: new Array(w * d).fill(0), meta: { ...base.meta!, roads: country ? [main, out] : [main] } } as unknown as WorldStructure;
  };

  test('a country road drawn out of town has its own chunks, the chunks away from it as they were, a street handed on as a fresh one', async () => {
    const { roadKey, townKey } = await import('../src/render/shared/overviewKeys');
    for (const land of [false, true]) {
      const [was, now] = [withCountry(false), withCountry(true)].map((w) => (land ? onLand(w) : w));
      // The menus' town builds again for it: no lot came or went, its roads did.
      expect(roadKey(now)).not.toBe(roadKey(was));
      expect(townKey(now)).not.toBe(townKey(was));
      for (const ground of [false, true]) {
        const opts = { ...options, ground };
        const before = new TownStreet(new HousePlanner(), was, opts, 'town');
        for (const id of before.chunkIds()) before.chunkKey(id, NO_FOCUS);
        const fresh = new TownStreet(new HousePlanner(), now, opts, 'town');
        const handed = new TownStreet(new HousePlanner(), now, opts, 'town', before);
        // (The roads changed: the street's own is planned again, not taken over.)
        expect(handed.handedOn.statics).toBe(false);
        expect(handed.chunkIds()).toEqual(fresh.chunkIds());
        for (const id of fresh.chunkIds()) {
          expect(handed.chunkKey(id, NO_FOCUS)).toBe(fresh.chunkKey(id, NO_FOCUS));
          expect(drawn(handed.chunk(id, NO_FOCUS))).toBe(drawn(fresh.chunk(id, NO_FOCUS)));
        }
        // The game's street paves it (its surface in every chunk it crosses), and the road it leaves stays paved.
        if (ground) {
          for (const z of [60, 110, 170]) {
            const id = chunkAt(322, z);
            expect(fresh.chunkIds()).toContain(id);
            expect(fresh.chunk(id, NO_FOCUS).ground.some((g) => g.name === 'streets' && !g.geo.empty)).toBe(true);
          }
          const streetsIn = (s: InstanceType<typeof TownStreet>, id: number) => s.chunk(id, NO_FOCUS).ground.find((g) => g.name === 'streets')?.geo.pos.length ?? 0;
          for (const x of [100, 200, 290, 360]) expect(streetsIn(fresh, chunkAt(x, 23))).toBeGreaterThan(0);
        }
        // Nothing goes missing anywhere else: the paving (seen from above) in every 20 m along the road away from the
        // junction as it was, and sidewalks wherever they were. (Those are laid out between the roads that stop them,
        // and one more stops them far off: their slabs and the ranks along them come a little otherwise.)
        const areas = (street: InstanceType<typeof TownStreet>) => {
          const out = new Map<string, number>();
          const add = (what: string, geo: { pos: ArrayLike<number>; idx: ArrayLike<number> }) => {
            const { pos, idx } = geo;
            for (let t = 0; t < idx.length; t += 3) {
              const [i, j, k] = [idx[t] * 3, idx[t + 1] * 3, idx[t + 2] * 3];
              const area = Math.abs((pos[j] - pos[i]) * (pos[k + 2] - pos[i + 2]) - (pos[k] - pos[i]) * (pos[j + 2] - pos[i + 2])) / 2;
              const key = `${what}|${Math.floor((pos[i] + pos[j] + pos[k]) / 3 / 20)}`;
              out.set(key, (out.get(key) ?? 0) + area);
            }
          };
          for (const id of street.chunkIds()) {
            const c = street.chunk(id, NO_FOCUS);
            add('walk', c.walk);
            for (const g of c.ground) if (g.name === 'streets') add('road', g.geo);
          }
          return out;
        };
        const [had, has] = [areas(before), areas(fresh)];
        for (const [key, area] of had) {
          const x = Number(key.split('|')[1]) * 20;
          if (x > 280 && x < 340) continue;
          if (key.startsWith('walk')) expect(has.get(key) ?? 0).toBeGreaterThan(area / 2);
          else expect(Math.abs((has.get(key) ?? 0) - area)).toBeLessThan(area * 0.02 + 0.5);
        }
      }
    }
  });
});
