// The lawn's grass, engine-neutral (`render/shared/grass.ts`): where it grows on a small lot (none
// indoors, on the indoor half of a split tile, under furniture, on patios and paths, nor right by a
// wall), the field built the same twice from the same town, the gusts carried along the wind as
// the shader reads them, and the draws' blade counts covering the share asked for at every distance.
import { describe, expect, test } from 'bun:test';
import type { WorldStructure } from '../src/core/protocol';
import { GRASS, grassAt, grassField, grassLod, grassShare, gustAt, gustNoise, sampleGust, townGrass, type GrassLand } from '../src/render/shared/grass';
import { CountryRoads } from '../src/render/shared/countryRoads';
import { LandscapeLayout, WIND } from '../src/render/shared/nature';

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
    mode: 'life',
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
  } as WorldStructure;
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
    const world = lot();
    const field = await grassField(world, land);
    expect(grassAt(field, 7.5, 5.5)).toBe(1);
    world.rooms[5 * world.width + 7] = 2;
    townGrass(field, world);
    expect(grassAt(field, 7.5, 5.5)).toBe(0);
    world.rooms[5 * world.width + 7] = 0;
    townGrass(field, world);
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
