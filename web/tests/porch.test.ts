/**
 * A house looks the same from the street as in the game (`HousePlanner.silhouettes` and `build`):
 *
 * - The porches over its doors are the simulation's (`WorldStructure.porches`: the town's over its
 *   front doors, `frontPorch`, and the ones the player built). Each stands on a neighbour's
 *   silhouette where its door is in the outline of the box (`porchDoors`), on the viewed lot's house
 *   (the street's `lot` share, `dressingOnly`: the same canopy, posts and lamp, going with its wall
 *   in the cutaway, nothing of it indoors) and on the player's house in the menus' town. Porch
 *   pieces (posts, canopies, step platforms) stand on their lot in the house's colours.
 * - Every storey of the game's house has the lot's look and the roof the player chose, as the
 *   silhouette has: an upper storey's view lies on its own rows (`storeyRow`), and its look was once
 *   chosen from there (brick upstairs over siding, another roof).
 */
import { describe, expect, test } from 'bun:test';
import type { Opening, PlotInfo, PorchInfo, WallEdge, WorldStructure } from '../src/core/protocol';
import { Cut, type Geo, LOOK } from '../src/render/shared/geometry';
import type { HousePart, HousePlanner as Planner, SilhouetteSpec } from '../src/render/shared/house';

describe('porches and looks', async () => {
  // The renderer's modules read the page's address (and its frames' timing) as they load.
  Object.assign(globalThis, { location: { href: 'http://localhost/', search: '', origin: 'http://localhost', pathname: '/' }, requestAnimationFrame: () => 0, window: {} });
  const { HousePlanner, houseScheme, porchDoors } = await import('../src/render/shared/house');
  const { planStreet } = await import('../src/render/shared/street');
  const { frontPorch, turner } = await import('../src/game/town');

  // ---- the porch ------------------------------------------------------------------------

  /** A porch over the door at `x` of a wall along x at `z`, out towards +z (`out` -1: -z). */
  const porch = (x: number, z = 14, out = 1, axis: 'h' | 'v' = 'h'): PorchInfo => ({ plot: 0, axis, x, z, out });
  /** A house box 3..14 × 5..14 whose front (the street) is its z1 side, the garden path meeting it at x 8.5, a porch over its front door. */
  const spec = (extra: Partial<SilhouetteSpec> = {}): SilhouetteSpec => ({ x0: 3, z0: 5, x1: 14, z1: 14, lot: { x: 0, z: 0 }, front: [0, 1], door: 8.5, garage: null, porches: [porch(8)], ...extra });
  const door = (x: number, z = 14, axis: 'h' | 'v' = 'h'): Opening => ({ axis, x, z, kind: 'door' });

  test("a porch stands over the door the simulation has it over: on a silhouette where the door is in the outline of its box", () => {
    expect(porchDoors(spec())).toEqual([{ axis: 'h', plane: 14.07, n: [0, 0, 1], mid: 8.5 }]);
    // Over a door in the back wall, out the back: there too (the player built it).
    expect(porchDoors(spec({ porches: [porch(5, 5, -1)] }))).toEqual([{ axis: 'h', plane: 4.93, n: [0, 0, -1], mid: 5.5 }]);
    // None, or over a door set back inside the box: none on the silhouette; on the viewed lot's house, where the game draws the house, it stands.
    expect(porchDoors(spec({ porches: [] }))).toEqual([]);
    expect(porchDoors(spec({ porches: undefined }))).toEqual([]);
    expect(porchDoors(spec({ porches: [porch(8, 12)] }))).toEqual([]);
    expect(porchDoors(spec({ porches: [porch(8, 12)], dressingOnly: true }))).toHaveLength(1);
    // On a wall along z: a house whose street is west of it.
    const west = spec({ front: [-1, 0], door: 9.5, porches: [porch(3, 9, -1, 'v')] });
    expect(porchDoors(west)).toEqual([{ axis: 'v', plane: 2.93, n: [-1, 0, 0], mid: 9.5 }]);
  });

  test('the town puts a porch over a front door in the front of its walls, turned onto the town', () => {
    // A plot 12 × 10 at (20, 30), its street at its z = 0 side; a house box 2..10 × 3..8, the front door at (5, 3).
    const walls: [number, number, number, number][] = [[2, 3, 10, 3], [10, 3, 10, 8], [2, 8, 10, 8], [2, 3, 2, 8]];
    const front = { x: 5, z: 3, axis: 'x' as const };
    expect(frontPorch(walls, front, turner(20, 30, 12, 10, 0), 4)).toEqual({ plot: 4, axis: 'h', x: 25, z: 33, out: -1 });
    // Turned onto the town with the street on the plot's +z side (facing 2) and its west side (facing 1).
    expect(frontPorch(walls, front, turner(20, 30, 12, 10, 2), 4)).toEqual({ plot: 4, axis: 'h', x: 26, z: 37, out: 1 });
    expect(frontPorch(walls, front, turner(20, 30, 12, 10, 1), 4)).toEqual({ plot: 4, axis: 'v', x: 23, z: 36, out: -1 });
    // A door set back from the front of the walls, or in a side wall: none.
    expect(frontPorch(walls, { x: 5, z: 4, axis: 'x' }, turner(20, 30, 12, 10, 0), 4)).toBeNull();
    expect(frontPorch(walls, { x: 2, z: 4, axis: 'z' }, turner(20, 30, 12, 10, 0), 4)).toBeNull();
  });

  const part = (parts: readonly HousePart[], name: string): Geo => parts.find((p) => p.name === name)!.geo;
  /** Vertex positions, rounded to the millimetre, sorted. */
  const points = (g: Geo): string[] => {
    const out: string[] = [];
    for (let i = 0; i < g.pos.length; i += 3) out.push([g.pos[i], g.pos[i + 1], g.pos[i + 2]].map((v) => (Math.round(v * 1000) / 1000 + 0).toFixed(3)).join(','));
    return out.sort();
  };

  test("the viewed lot's porch is the neighbour's: the same canopy and lamp, going with its wall in the cutaway", () => {
    const planner = new HousePlanner();
    const neighbour = planner.silhouettes([spec()], [door(8)]);
    const viewed = planner.silhouettes([spec({ dressingOnly: true })], [door(8)]);
    // Only the porch (no shell: the game draws the house), every bit of it on the neighbour's too.
    expect(part(viewed.parts, 'silhouettePlinth').empty).toBe(true);
    const roof = points(part(viewed.parts, 'silhouetteRoofs'));
    expect(roof.length).toBeGreaterThan(0);
    const all = new Set(points(part(neighbour.parts, 'silhouetteRoofs')));
    expect(roof.filter((p) => !all.has(p))).toEqual([]);
    // Its lamp in the same place, with the look bits that drop its wall (the street's z1 side: looking along -z).
    expect(neighbour.lamps).toEqual([[9.22, 1.75, 14.15, 0, 1]]);
    expect(viewed.lamps).toEqual([[9.22, 1.75, 14.15, 0, 1, LOOK.NegZ]]);
    // The posts drop to stubs with the wall and the canopy goes; the landing stays. The neighbour's stays whole.
    const kinds = (g: Geo) => new Set(Array.from({ length: g.cut.length / 4 }, (_, v) => `${g.cut[v * 4 + 1]}:${g.cut[v * 4]}:${g.cut[v * 4 + 2]}`));
    expect(kinds(part(viewed.parts, 'silhouetteTrim'))).toEqual(new Set([`${Cut.Keep}:${LOOK.All}:${LOOK.All}`, `${Cut.Clamp}:${LOOK.NegZ}:${LOOK.NegZ}`, `${Cut.Hide}:${LOOK.NegZ}:${LOOK.NegZ}`]));
    expect(kinds(part(viewed.parts, 'silhouetteRoofs'))).toEqual(new Set([`${Cut.Hide}:${LOOK.NegZ}:${LOOK.NegZ}`]));
    for (const name of ['silhouetteTrim', 'silhouetteRoofs', 'silhouetteWalls', 'silhouetteBrick']) {
      expect(kinds(part(neighbour.parts, name)).size).toBeLessThanOrEqual(1);
    }
    // Nothing of it indoors: no further in than the wall's inner face (z 14 - 0.07).
    for (const name of ['silhouetteRoofs', 'silhouetteTrim', 'silhouetteWalls', 'silhouetteBrick']) {
      const g = part(viewed.parts, name);
      for (let i = 2; i < g.pos.length; i += 3) expect(g.pos[i]).toBeGreaterThan(14 - 0.07);
    }
    // No porch (taken down, or gone with its door): nothing, no lamp.
    const none = planner.silhouettes([spec({ dressingOnly: true, porches: [] })], [door(8)]);
    expect(none.lamps).toEqual([]);
    expect(none.parts.every((p) => p.geo.empty)).toBe(true);
  });

  test("the player's house in the menus' town has its porch (and no chimney), as the game draws it", () => {
    const planner = new HousePlanner();
    const home = planner.silhouettes([spec({ home: true })], [door(8)]);
    expect(home.lamps).toEqual([[9.22, 1.75, 14.15, 0, 1]]);
  });

  /** Two houses side by side on a street along x (their fronts on it, z 20), each a front door on its garden path with a porch over it; `taken`: the first's porch taken down. */
  function town(taken = false): WorldStructure {
    const lots = [
      { r: { x: 0, z: 0, w: 20, d: 20 }, house: [3, 5, 14, 14] as [number, number, number, number], door: 8 },
      { r: { x: 20, z: 0, w: 18, d: 20 }, house: [24, 6, 33, 15] as [number, number, number, number], door: 28 },
    ];
    const plots: PlotInfo[] = lots.map((l, id) => ({ id, name: `${id}`, ...l.r, public: false, entry: [l.door + 0.5, 18], house: l.house }));
    const paths = lots.map((l) => ({ x: l.door, z: l.house[3], w: 1, d: 20 - l.house[3] }));
    const walls: WallEdge[] = [];
    const openings: Opening[] = [];
    lots.forEach((l) => {
      const [x0, z0, x1, z1] = l.house;
      for (let x = x0; x < x1; x++) walls.push({ axis: 'h', x, z: z0 }, { axis: 'h', x, z: z1 });
      for (let z = z0; z < z1; z++) walls.push({ axis: 'v', x: x0, z }, { axis: 'v', x: x1, z });
      openings.push({ axis: 'h', x: l.door, z: z1, kind: 'door' });
    });
    const porches = lots.flatMap((l, i) => (taken && i === 0 ? [] : [{ plot: i, axis: 'h' as const, x: l.door, z: l.house[3], out: 1 }]));
    const [w, d] = [38, 26];
    return {
      width: w,
      depth: d,
      objects: [],
      plots,
      rooms: new Array(w * d).fill(0),
      openings,
      walls,
      households: [],
      sims: [],
      exits: [],
      porches,
      meta: { kind: 'town', name: 'Test', streets: [], paths, facing: [2, 2], seed: 1, home: 0 },
    } as unknown as WorldStructure;
  }
  const options = { has: () => true, entry: () => undefined, lived: false };
  const view = (p: PlotInfo) => ({ x: p.x, z: p.z, w: p.w, d: p.d });

  test("the street gives the viewed lot its house's porch, and the player's house in the menus' town too", async () => {
    const world = town();
    // Viewed: the lot's share has the porch (its lamp apart, with its wall's look bits), the house the game's.
    const game = await planStreet(new HousePlanner(), world, view(world.plots[0]), options);
    expect(game.lot?.porches.map((p) => p.look)).toEqual([LOOK.NegZ]);
    expect(game.lot!.porches[0].glow.empty).toBe(false);
    expect(game.lot!.silhouettes.parts.find((p) => p.name === 'silhouetteRoofs')!.geo.empty).toBe(false);
    const viewed = game.specs.find((s) => s.lot.x === 0)!;
    expect(viewed.dressingOnly).toBe(true);
    // From the next lot: a silhouette with the same porch (the same spec but for being viewed).
    const next = await planStreet(new HousePlanner(), world, view(world.plots[1]), options);
    const seen = next.specs.find((s) => s.lot.x === 0)!;
    expect(seen.dressingOnly).toBe(false);
    expect(porchDoors(seen)).toEqual(porchDoors(viewed));
    expect(porchDoors(seen)).toHaveLength(1);
    // The menus' town: the player's house (lot 0) a shell with its porch.
    const menus = await planStreet(new HousePlanner(), world, null, options);
    const home = menus.specs.find((s) => s.lot.x === 0)!;
    expect(home.home).toBe(true);
    expect(porchDoors(home)).toHaveLength(1);
    // Its porch taken down: none on the viewed lot, nor on its silhouette.
    const taken = town(true);
    const gone = await planStreet(new HousePlanner(), taken, view(taken.plots[0]), options);
    expect(gone.lot?.porches ?? []).toEqual([]);
    expect(gone.lot?.silhouettes.parts.every((p) => p.geo.empty) ?? true).toBe(true);
    const far = await planStreet(new HousePlanner(), taken, view(taken.plots[1]), options);
    expect(porchDoors(far.specs.find((s) => s.lot.x === 0)!)).toEqual([]);
    // Porch pieces on the viewed lot: drawn with its garage and porches, in the house's colours.
    const built = { ...world, porchPieces: [{ plot: 0, kind: 'canopy' as const, x: 15, z: 15, w: 2, d: 3, rot: 0 }] };
    const withPieces = await planStreet(new HousePlanner(), built, view(world.plots[0]), options);
    const roofs = (plan: typeof game) => points(plan.lot!.silhouettes.parts.find((p) => p.name === 'silhouetteRoofs')!.geo).length;
    expect(roofs(withPieces)).toBeGreaterThan(roofs(game));
  });

  test("porch pieces are the porch's parts in the house's colours: a post, a canopy over posts or against a wall, a step platform", () => {
    const planner = new HousePlanner();
    const scheme = houseScheme(0, 0);
    const pieces = [
      { plot: 0, kind: 'post' as const, x: 16, z: 16, w: 1, d: 1, rot: 0 },
      { plot: 0, kind: 'canopy' as const, x: 15, z: 15, w: 2, d: 3, rot: 0 },
      { plot: 0, kind: 'step' as const, x: 15, z: 15, w: 2, d: 3, rot: 1 },
    ];
    const plan = planner.silhouettes([spec({ dressingOnly: true, porches: [], pieces })], []);
    // The canopy: the house's roof, its gables the house's wall colour (siding or brick, whichever it has), over its tiles, its eaves stopping
    // inside a wall on their outline (its back against the house: nothing of it past a wall's inner face, 0.07 in).
    const roof = part(plan.parts, 'silhouetteRoofs');
    expect(colour(roof)).toBe(scheme.roof);
    const gables = part(plan.parts, scheme.brick ? 'silhouetteBrick' : 'silhouetteWalls');
    expect(colour(gables)).toBe(scheme.wall);
    for (const [k, a, b] of [[0, 15, 17], [2, 15, 18]] as const) {
      let [lo, hi] = [Infinity, -Infinity];
      for (let i = k; i < roof.pos.length; i += 3) [lo, hi] = [Math.min(lo, roof.pos[i]), Math.max(hi, roof.pos[i])];
      expect(lo).toBeGreaterThan(a - 0.07);
      expect(lo).toBeLessThan(a);
      expect(hi).toBeLessThan(b + 0.07);
      expect(hi).toBeGreaterThan(b);
    }
    // The ridge runs the way it faces (along z): the roof's top (its ridge cap) is in the middle across x, all along z.
    let top = -Infinity;
    for (let i = 1; i < roof.pos.length; i += 3) top = Math.max(top, roof.pos[i]);
    const ridge: number[] = [];
    for (let i = 0; i < roof.pos.length; i += 3) if (roof.pos[i + 1] > top - 0.001) ridge.push(roof.pos[i]);
    expect(ridge.length).toBeGreaterThan(0);
    expect(ridge.every((x) => Math.abs(x - 16) < 0.12)).toBe(true);
    // The post and the platform: trim, low (the platform at most 5 cm) but for the post; no lamp; nothing goes in the cutaway.
    const trim = part(plan.parts, 'silhouetteTrim');
    let tall = 0;
    for (let i = 0; i < trim.pos.length; i += 3) if (trim.pos[i] > 16.4 && trim.pos[i] < 16.6 && trim.pos[i + 2] > 16.4 && trim.pos[i + 2] < 16.6) tall = Math.max(tall, trim.pos[i + 1]);
    expect(tall).toBeCloseTo(2.34, 2);
    expect(plan.lamps).toEqual([]);
    for (const g of [trim, roof]) for (let v = 0; v < g.cut.length / 4; v++) expect(g.cut[v * 4 + 1]).toBe(Cut.Keep);
  });

  // ---- the look -----------------------------------------------------------------------

  const DEPTH = 12;
  const W = 12;
  /** The sRGB colour (`#rrggbb`) of a Geo's first vertex. */
  const colour = (g: Geo) => `#${[g.col[0], g.col[1], g.col[2]].map((c) => Math.round(c * 255).toString(16).padStart(2, '0')).join('').toUpperCase()}`;

  /** A box house 6 × 5 from (ox + 2, 2) on both storeys of a lot at (ox, 0), front door on its z 2 side; `roof`: the player's choice. */
  function twoStoreys(ox: number, roof?: [number, number]): WorldStructure {
    const walls: WallEdge[] = [];
    const openings: Opening[] = [];
    const width = ox + W;
    const rooms = new Array(width * DEPTH * 2).fill(0);
    for (let k = 0; k < 2; k++) {
      const z0 = 2 + k * DEPTH;
      for (let x = ox + 2; x < ox + 8; x++) walls.push({ axis: 'h', x, z: z0 }, { axis: 'h', x, z: z0 + 5 });
      for (let z = z0; z < z0 + 5; z++) walls.push({ axis: 'v', x: ox + 2, z }, { axis: 'v', x: ox + 8, z });
      for (let z = z0; z < z0 + 5; z++) for (let x = ox + 2; x < ox + 8; x++) rooms[z * width + x] = k + 1;
    }
    openings.push({ axis: 'h', x: ox + 4, z: 2, kind: 'door' });
    const plot: PlotInfo = { id: 0, name: '0', x: ox, z: 0, w: W, d: DEPTH, public: false, entry: null, house: [ox + 2, 2, ox + 8, 7], storeys: 2, ...(roof ? { roof } : {}) };
    return { width, depth: DEPTH * 2, storeyDepth: DEPTH, storeys: 2, walls, openings, rooms, objects: [], plots: [plot], households: [], sims: [], exits: [] } as unknown as WorldStructure;
  }

  /** A lot whose rows upstairs would have given another look (brick or siding, wall colour, roof colour). */
  const lotX = (() => {
    for (let x = 0; x < 200; x++) {
      const [a, b] = [houseScheme(x, 0), houseScheme(x, DEPTH)];
      if (a.brick !== b.brick && a.roof !== b.roof) return x;
    }
    throw new Error('no such lot');
  })();

  /** The game's house: each storey's outside walls (finish and colour), and the roof over the top one. */
  function game(planner: Planner, world: WorldStructure) {
    const storeys = [0, 1].map((k) => {
      const plan = planner.build(world, { x: lotX, z: k * DEPTH, w: W, d: DEPTH }, k === 0 ? { covered: () => true } : { upper: true, covered: () => false })!;
      const ext = plan.parts.find((p) => p.name === 'wallsExterior')!;
      return { finish: ext.surface.kind === 'surface' ? ext.surface.key : '', wall: colour(ext.geo), roof: plan.parts.find((p) => p.name === 'roof')!.geo };
    });
    return { storeys, roof: colour(storeys[1].roof) };
  }
  /** The street's silhouette of it: its walls (finish and colour) and roof. */
  function street(planner: Planner, world: WorldStructure) {
    const plot = world.plots[0];
    const [x0, z0, x1, z1] = plot.house!;
    const plan = planner.silhouettes([{ x0, z0, x1, z1, lot: { x: plot.x, z: plot.z }, front: [0, -1], door: null, garage: null, storeys: 2, roof: plot.roof }], world.openings!, world);
    const walls = plan.parts.find((p) => (p.name === 'silhouetteWalls' || p.name === 'silhouetteBrick') && !p.geo.empty)!;
    return { finish: walls.surface.kind === 'surface' ? walls.surface.key : '', wall: colour(walls.geo), roof: colour(plan.parts.find((p) => p.name === 'silhouetteRoofs')!.geo) };
  }

  test("every storey of the game's house has the lot's look, as the street's silhouette of it has", () => {
    const planner = new HousePlanner();
    const world = twoStoreys(lotX);
    const seen = street(planner, world);
    const own = game(planner, world);
    const scheme = houseScheme(lotX, 0);
    expect(seen.wall).toBe(scheme.wall);
    for (const s of own.storeys) {
      expect(s.finish).toBe(seen.finish);
      expect(s.wall).toBe(seen.wall);
    }
    expect(own.storeys[0].roof.empty).toBe(true);
    expect(own.roof).toBe(seen.roof);
    expect(own.roof).toBe(scheme.roof);
  });

  test('the roof the player chose, on the top storey of the house as on its silhouette', () => {
    const planner = new HousePlanner();
    planner.setLooks({ coverings: [], floors: [], doors: [], windows: [], roofs: [{ id: 'gable', label: 'Gable', shape: 'gable', pitch: 30 }], roofColors: [{ id: 'a', label: 'A', color: '#112233' }, { id: 'b', label: 'B', color: '#A1B2C3' }] });
    const world = twoStoreys(lotX, [0, 1]);
    const own = game(planner, world);
    expect(own.roof).toBe('#A1B2C3');
    expect(street(planner, world).roof).toBe('#A1B2C3');
  });
});
