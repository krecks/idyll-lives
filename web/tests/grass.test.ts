// The lawn's grass, engine-neutral (`render/shared/grass.ts`): where it grows on a small lot (none
// indoors, on the indoor half of a split tile, under furniture, on patios and paths, nor right by a
// wall), made again only round what changed (`lawnChanges`) into the same bytes as grown afresh, the
// field built the same twice from the same town, the gusts carried along the wind as the shader
// reads them, and the draws' blade counts covering the share asked for at every distance.
import { describe, expect, test } from 'bun:test';
import type { WorldStructure } from '../src/core/protocol';
import { GRASS, grassAt, grassField, grassLod, grassShare, gustAt, gustNoise, sampleGust, townGrass, type GrassLand } from '../src/render/shared/grass';
import { CountryRoads } from '../src/render/shared/countryRoads';
import { GROUND_SHADE } from '../src/render/shared/groundShade';
import { bareTiles, keepBareTiles, LandscapeLayout, lawnChanges, lawnMask, WIND } from '../src/render/shared/nature';

/** A 12 × 10 lot: a 3 × 3 room (walls round it) at (2, 2), a sofa at (8, 2), a patio tile, a path, and a split tile. */
function lot(): WorldStructure {
  const width = 12;
  const depth = 10;
  const rooms = new Array(width * depth).fill(0);
  for (let z = 2; z < 5; z++) for (let x = 2; x < 5; x++) rooms[z * width + x] = 1;
  const walls: WorldStructure['walls'] = [];
  for (let x = 2; x < 5; x++) walls.push({ axis: 'h', x, z: 2 }, { axis: 'h', x, z: 5 });
  for (let z = 2; z < 5; z++) walls.push({ axis: 'v', x: 2, z }, { axis: 'v', x: 5, z });
  return {
    version: 1,
    lotVersion: 1,
    mode: 'living',
    width,
    depth,
    objects: [{ id: 1, def: 'sofa', x: 8, z: 2, rot: 0, w: 2, d: 1 } as WorldStructure['objects'][number]],
    sims: [],
    households: [],
    plots: [],
    exits: [],
    rooms,
    walls,
    openings: [],
    // Half 1 indoors (a 'dp' wall: half 0 is v < u).
    diagonals: [{ axis: 'dp', x: 9, z: 7, kind: 'wall', rooms: [0, 2] }],
    floors: [[6, 7, 1]],
    meta: { paths: [{ x: 0, z: 8, w: 4, d: 1 }] } as unknown as WorldStructure['meta'],
    meshes: {} as WorldStructure['meshes'],
  };
}

/** Flat land, all lawn, one green. */
const land: GrassLand = {
  heightAt: () => 0,
  terrainColour: (_x, _z, _h, out) => {
    out.r = 0.5;
    out.g = 0.7;
    out.b = 0.3;
    return 1;
  },
  distanceToTown: (x, z) => Math.hypot(Math.max(-x, 0, x - 12), Math.max(-z, 0, z - 10)),
};

describe('where the grass grows', () => {
  test('none indoors, under furniture, on patios and paths, nor by a wall; the rest is lawn', async () => {
    const world = lot();
    const field = await grassField(world, land);
    expect(field.townW).toBe(12 * GRASS.townTexels);
    // Open lawn, and outside the town.
    expect(grassAt(field, 7.5, 5.5)).toBe(1);
    expect(grassAt(field, -5, -5)).toBe(1);
    // Indoors, under the sofa, the patio, the path.
    expect(grassAt(field, 3.5, 3.5)).toBe(0);
    expect(grassAt(field, 8.5, 2.5)).toBe(0);
    expect(grassAt(field, 9.5, 2.5)).toBe(0);
    expect(grassAt(field, 6.5, 7.5)).toBe(0);
    expect(grassAt(field, 1.5, 8.5)).toBe(0);
    // Right outside the room's wall (x = 5), and a little further out.
    expect(grassAt(field, 5.1, 3.5)).toBe(0);
    expect(grassAt(field, 5.6, 3.5)).toBe(1);
    // The split tile: its outdoor half grows, its indoor half doesn't.
    expect(grassAt(field, 9.8, 7.2)).toBe(1);
    expect(grassAt(field, 9.2, 7.8)).toBe(0);
  });

  test('a room built on the lawn clears it, and taking it down brings it back', async () => {
    let world = lot();
    const field = await grassField(world, land);
    let mask = lawnMask(world);
    keepBareTiles(world, mask);
    const refresh = (rooms: WorldStructure['rooms']) => {
      const next = { ...world, rooms };
      const rects = lawnChanges(world, next);
      mask = lawnMask(next, mask, rects);
      townGrass(field, next, mask, keepBareTiles(next, mask, rects && { was: world, rects }), rects && { was: world, rects });
      world = next;
    };
    expect(grassAt(field, 7.5, 5.5)).toBe(1);
    refresh(world.rooms.map((r, i) => (i === 5 * world.width + 7 ? 2 : r)));
    expect(grassAt(field, 7.5, 5.5)).toBe(0);
    refresh(world.rooms.map((r, i) => (i === 5 * world.width + 7 ? 0 : r)));
    expect(grassAt(field, 7.5, 5.5)).toBe(1);
  });

  test('a patch with any grass is drawn; one all indoors is not', async () => {
    const field = await grassField(lot(), land);
    const at = (x: number, z: number) => field.patches[Math.floor((z - field.z0) / GRASS.patch) * field.pw + Math.floor((x - field.x0) / GRASS.patch)];
    expect(at(7, 6)).toBe(1);
    expect(at(-10, -10)).toBe(1);
    // (2..4 × 2..4 is all room, its walls included.)
    expect(at(3, 3)).toBe(0);
  });

  test('the same town grows the same field', async () => {
    const a = await grassField(lot(), land);
    const b = await grassField(lot(), land);
    expect(Buffer.from(a.colour).equals(Buffer.from(b.colour))).toBe(true);
    expect(Buffer.from(a.town).equals(Buffer.from(b.town))).toBe(true);
    expect(Buffer.from(a.patches).equals(Buffer.from(b.patches))).toBe(true);
  });
});

describe('roads', () => {
  test('no grass on the roads out of town, nor on the side roads joining the streets past its ends', async () => {
    const width = 60;
    const depth = 40;
    const world = {
      ...lot(),
      width,
      depth,
      rooms: new Array(width * depth).fill(0),
      objects: [],
      walls: [],
      diagonals: [],
      floors: [],
      meta: { seed: 7, streets: [{ x: 0, z: 6, w: width, d: 6 }, { x: 0, z: 28, w: width, d: 6 }], paths: [] },
    } as unknown as WorldStructure;
    const layout = new LandscapeLayout(world, (l, w, name) => new CountryRoads(l, w, name));
    const field = await grassField(world, layout);
    expect(layout.roads.links.length).toBe(2);
    for (const r of layout.roads.links) {
      for (let z = r.z; z <= r.z + r.d; z += 2) for (let x = r.x; x <= r.x + r.w; x += 1.5) expect(grassAt(field, x, z)).toBe(0);
    }
    for (const road of layout.roads.roads) for (const [x, z] of road) expect(grassAt(field, x, z)).toBe(0);
    // The lawn beside them grows.
    expect(grassAt(field, -15, 20)).toBeGreaterThan(0);
  });
});

describe('the gusts', () => {
  const noise = gustNoise();

  test('the noise tiles: read a tile over, it is the same', () => {
    for (const [u, v] of [
      [0.13, 0.71],
      [0.5, 0.02],
      [0.99, 0.99],
    ]) {
      expect(sampleGust(noise, u + 1, v)).toBeCloseTo(sampleGust(noise, u, v), 9);
      expect(sampleGust(noise, u, v - 1)).toBeCloseTo(sampleGust(noise, u, v), 9);
    }
  });

  test('a gust rolls along the wind: what is here now is downwind a second later', () => {
    const v = GRASS.gust.speed;
    for (const [x, z, t] of [
      [3, 4, 0],
      [-20, 15, 7.5],
      [60, -2, 123],
    ]) {
      expect(gustAt(noise, t + 1, x + WIND.dir[0] * v, z + WIND.dir[1] * v)).toBeCloseTo(gustAt(noise, t, x, z), 6);
    }
  });

  test('calm and strong gusts both blow over a lawn', () => {
    let lo = 1;
    let hi = 0;
    for (let z = 0; z < 120; z += 1.5) {
      for (let x = 0; x < 120; x += 1.5) {
        const g = gustAt(noise, 0, x, z);
        lo = Math.min(lo, g);
        hi = Math.max(hi, g);
      }
    }
    expect(lo).toBeLessThan(0.05);
    expect(hi).toBeGreaterThan(0.95);
  });
});

describe('density with distance', () => {
  test('every draw has the blades its share asks for, and not many more', () => {
    for (const density of [1, 0.5]) {
      for (let d = 0; d < 200; d += 0.25) {
        const want = grassShare(d, density) * GRASS.perPatch;
        const lod = grassLod(d, density);
        const have = GRASS.lods[lod].blades;
        expect(have).toBeGreaterThanOrEqual(Math.min(want, GRASS.lods[0].blades) - 1e-9);
        // The sparsest draw is as sparse as it gets; elsewhere within a step of √2.
        if (lod < GRASS.lods.length - 1) expect(have).toBeLessThanOrEqual(want * Math.SQRT2 + 1);
      }
    }
  });

  test('the share falls as near / d past `near`, and with the preset\'s density', () => {
    expect(grassShare(5)).toBe(1);
    expect(grassShare(GRASS.near * 2)).toBeCloseTo(0.5, 9);
    expect(grassShare(GRASS.near * 2, 0.5)).toBeCloseTo(0.25, 9);
  });
});

type Thing = WorldStructure['objects'][number];
type Edge = WorldStructure['walls'][number];

/** A 40 × 30 town of two storeys: two plots, a house with a door, a split tile and a patio on the first, a fence and a gate, a path, a bench, a tree and a bed. */
function town(): WorldStructure {
  const [width, sd] = [40, 30];
  const rooms = new Array(width * sd * 2).fill(0);
  const room = (x0: number, z0: number, x1: number, z1: number, id: number, storey = 0) => {
    for (let z = z0; z < z1; z++) for (let x = x0; x < x1; x++) rooms[(z + storey * sd) * width + x] = id;
  };
  room(5, 5, 10, 9, 1);
  room(5, 5, 10, 9, 2, 1);
  const walls: Edge[] = [];
  for (let x = 5; x < 10; x++) walls.push({ axis: 'h', x, z: 5 }, { axis: 'h', x, z: 9 });
  for (let z = 5; z < 9; z++) walls.push({ axis: 'v', x: 5, z }, { axis: 'v', x: 10, z });
  const thing = (id: number, def: string, x: number, z: number, w = 1, d = 1, more: Partial<Thing> = {}): Thing => ({ id, def, x, z, rot: 0, w, d, ...more }) as Thing;
  return {
    version: 1,
    lotVersion: 1,
    mode: 'living',
    width,
    depth: sd * 2,
    storeys: 2,
    storeyDepth: sd,
    objects: [thing(1, 'sofa', 6, 6, 2, 1), thing(2, 'parkBench', 12, 6, 2, 1), thing(3, 'garden.maple', 14, 10, 2, 2), thing(4, 'raisedBed', 6, 12, 2, 1, { offset: [30, -20] })],
    sims: [],
    households: [],
    plots: [{ x: 2, z: 2, w: 16, d: 14 }, { x: 22, z: 2, w: 14, d: 14 }] as WorldStructure['plots'],
    exits: [],
    rooms,
    walls,
    openings: [{ kind: 'door', axis: 'h', x: 7, z: 9 }, { kind: 'window', axis: 'v', x: 10, z: 6 }] as WorldStructure['openings'],
    diagonals: [{ axis: 'dp', x: 11, z: 8, kind: 'wall', rooms: [0, 1] }],
    fences: [...Array.from({ length: 6 }, (_, k): NonNullable<WorldStructure['fences']>[number] => ({ axis: 'v', x: 18, z: 4 + k, kind: k === 2 ? 'gate' : 'fence' }))],
    floors: [[11, 10, 1], [12, 10, 1]],
    meta: { seed: 3, paths: [{ x: 7, z: 10, w: 1, d: 5 }] } as unknown as WorldStructure['meta'],
    meshes: {} as WorldStructure['meshes'],
  };
}

describe('the lawn made again only round what changed', () => {
  test('a thing bought, moved and sold, a wall, a room, another lot and a first build: each as grown afresh', async () => {
    const wide: GrassLand = { ...land, distanceToTown: (x, z) => Math.hypot(Math.max(-x, 0, x - 40), Math.max(-z, 0, z - 30)) };
    let world = town();
    const field = await grassField(world, wide);
    let mask = lawnMask(world);
    keepBareTiles(world, mask);
    const area = (rects: readonly number[][]) => rects.reduce((a, r) => a + (r[2] - r[0] + 1) * (r[3] - r[1] + 1), 0);
    // `next` (only what changed new objects, as `mergeWorld` makes it) made again as the landscape's `refreshLawn`
    // does, and compared with all of it grown afresh: the mask, the bare tiles, and the grass's mask, shade, map
    // (the two textures' texels) and patches.
    const step = async (what: string, next: WorldStructure, most: number) => {
      const rects = lawnChanges(world, next);
      expect(`${what}: ${rects === null ? 'all' : rects.length ? 'some' : 'none'}`).toBe(`${what}: some`);
      expect(area(rects!)).toBeLessThanOrEqual(most);
      const change = { was: world, rects: rects! };
      mask = lawnMask(next, mask, rects);
      const bare = keepBareTiles(next, mask, change);
      townGrass(field, next, mask, bare, change);
      world = next;
      const freshMask = lawnMask(world);
      const fresh = await grassField(world, wide);
      const parts: [string, ArrayLike<number>, ArrayLike<number>][] = [
        ['hidden', mask.hidden, freshMask.hidden],
        ['bare', bare, bareTiles(world, freshMask)],
        ['town', field.town, fresh.town],
        ['shade', field.shade, fresh.shade],
        ['lawn', field.lawn, fresh.lawn],
        ['patches', field.patches, fresh.patches],
      ];
      for (const [part, a, b] of parts) expect(`${what} ${part}: ${Buffer.from(a as Uint8Array).equals(Buffer.from(b as Uint8Array))}`).toBe(`${what} ${part}: true`);
      expect([...mask.splitAxis]).toEqual([...freshMask.splitAxis]);
      // (And the birds' bare tiles, asked again of the same world: kept.)
      expect(bareTiles(world)).toBe(bare);
    };
    const thing = (o: Partial<Thing>) => ({ rot: 0, w: 1, d: 1, ...o }) as Thing;
    // A bench bought on the lawn (things after it unchanged), moved and turned, sold.
    const bench = thing({ id: 10, def: 'parkBench', x: 3, z: 12, w: 2 });
    await step('bought', { ...world, objects: [...world.objects.slice(0, 2), bench, ...world.objects.slice(2)] }, 200);
    await step('moved', { ...world, objects: world.objects.map((o) => (o.id === 10 ? { ...o, x: 25, z: 20, rot: 1, w: 1, d: 2 } : o)) }, 400);
    await step('sold', { ...world, objects: world.objects.filter((o) => o.id !== 10) }, 200);
    // Only turned (a resized thing: its shade turns with it), and one set well off its tile.
    const stool = thing({ id: 15, def: 'stool', x: 20, z: 18, size: [40, 90] });
    await step('stool', { ...world, objects: [...world.objects, stool] }, 200);
    await step('turned', { ...world, objects: world.objects.map((o) => (o.id === 15 ? { ...o, rot: 1 } : o)) }, 200);
    await step('off its tile', { ...world, objects: world.objects.map((o) => (o.id === 15 ? { ...o, offset: [400, 0] } : o)) }, 300);
    // A patio tile, and a door in a wall that stood.
    await step('patio', { ...world, floors: [...world.floors!, [20, 24, 1]] }, 100);
    await step('door', { ...world, openings: [...world.openings, { kind: 'door', axis: 'v', x: 10, z: 7 } as WorldStructure['openings'][number]] }, 100);
    // A room tile under the corner of a long table: the table indoors now, all of it.
    await step('table', { ...world, objects: [...world.objects, thing({ id: 16, def: 'picnicTable', x: 20, z: 26, w: 7 })] }, 300);
    await step('under the table', { ...world, rooms: world.rooms.map((r, i) => (i === 26 * 40 + 20 ? 5 : r)) }, 300);
    // Two things that shade the same ground swapped in the list (their shades laid in the other order; the things
    // between them made again too).
    const chair = thing({ id: 11, def: 'chairGarden', x: 13, z: 7 });
    await step('chair', { ...world, objects: [...world.objects, chair] }, 200);
    const [a, b] = [world.objects.findIndex((o) => o.id === 2), world.objects.findIndex((o) => o.id === 11)];
    await step('swapped', { ...world, objects: world.objects.map((o, i) => (i === a ? world.objects[b] : i === b ? world.objects[a] : o)) }, 1000);
    // A wall, and the room behind it (round the bench now indoors), its floor and a door.
    await step('wall', { ...world, walls: [...world.walls, { axis: 'h', x: 12, z: 5 }, { axis: 'h', x: 13, z: 5 }] }, 300);
    const rooms = world.rooms.slice();
    for (let z = 5; z < 8; z++) for (let x = 12; x < 14; x++) rooms[z * 40 + x] = 3;
    await step('room', { ...world, rooms, walls: [...world.walls, { axis: 'v', x: 14, z: 5 }, { axis: 'v', x: 14, z: 6 }], openings: [...world.openings, { kind: 'door', axis: 'h', x: 12, z: 8 } as WorldStructure['openings'][number]], floors: [...world.floors!, [12, 7, 2]] }, 400);
    // The split tile's diagonal turned, and the room on the storey above made larger (no grass changes there).
    await step('diagonal', { ...world, diagonals: [{ axis: 'dn', x: 11, z: 8, kind: 'wall', rooms: [1, 0] }] }, 100);
    await step('upstairs', { ...world, rooms: world.rooms.map((r, i) => (i === 36 * 40 + 10 ? 2 : r)) }, 100);
    // The other lot: a tree, a fence and a gate.
    await step('other lot', { ...world, objects: [...world.objects, thing({ id: 12, def: 'tree', x: 28, z: 6 })], fences: [...world.fences!, { axis: 'h', x: 25, z: 12, kind: 'fence' }, { axis: 'h', x: 26, z: 12, kind: 'gate' }] }, 500);
    // A first build on it: walls, rooms on both storeys, floors, a door, a diagonal and things, all at once.
    const built = world.rooms.slice();
    for (let z = 3; z < 7; z++) for (let x = 30; x < 34; x++) built[z * 40 + x] = built[(z + 30) * 40 + x] = 4;
    const walls: Edge[] = [];
    for (let x = 30; x < 34; x++) walls.push({ axis: 'h', x, z: 3 }, { axis: 'h', x, z: 7 });
    for (let z = 3; z < 7; z++) walls.push({ axis: 'v', x: 30, z }, { axis: 'v', x: 34, z });
    await step(
      'first build',
      {
        ...world,
        rooms: built,
        walls: [...walls.slice(0, 4), ...world.walls, ...walls.slice(4)],
        openings: [...world.openings, { kind: 'door', axis: 'h', x: 31, z: 7 } as WorldStructure['openings'][number]],
        diagonals: [...world.diagonals!, { axis: 'dp', x: 34, z: 8, kind: 'wall', rooms: [4, 0] }],
        floors: [...world.floors!, [31, 4, 1], [32, 4, 1]],
        objects: [...world.objects, thing({ id: 13, def: 'sofa', x: 31, z: 4, w: 2 }), thing({ id: 14, def: 'swingSet', x: 31, z: 10, w: 2, d: 2 })],
      },
      1200,
    );
    // Nothing the lawn grows round changed (a new world, its parts the same): nothing to do.
    expect(lawnChanges(world, { ...world, version: 2 })).toEqual([]);
    // Its plots or its paths changed: all of it.
    expect(lawnChanges(world, { ...world, plots: [...world.plots.slice(0, 1), { ...world.plots[1], w: 15 }] })).toBeNull();
    expect(lawnChanges(world, { ...world, meta: { ...world.meta! } })).toBeNull();
  });

  test('a wall whose shade is tuned to reach further: made again as far, as grown afresh', async () => {
    // (`?tune=shade.wall:0.65,3.5`, as the bench's look sheets ask: no tune in a test, so set on the map itself.)
    const was = GROUND_SHADE.wall;
    GROUND_SHADE.wall = [0.65, 3.5];
    try {
      const world = town();
      const field = await grassField(world, land);
      const mask = lawnMask(world);
      keepBareTiles(world, mask);
      const next = { ...world, walls: [...world.walls, { axis: 'h', x: 25, z: 8 } as Edge] };
      const rects = lawnChanges(world, next)!;
      const change = { was: world, rects };
      const now = lawnMask(next, mask, rects);
      townGrass(field, next, now, keepBareTiles(next, now, change), change);
      const fresh = await grassField(next, land);
      expect(Buffer.from(field.shade).equals(Buffer.from(fresh.shade))).toBe(true);
    } finally {
      GROUND_SHADE.wall = was;
    }
  });
});
