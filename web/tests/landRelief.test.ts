// The land a big world's relief grid shapes (`render/shared/nature.ts`): a world without one (older saves, the
// classic specs) draws exactly as it did, to the bit; one with it stands in its valley, flat by what is built and no
// steeper from it than its `slope`, its mountains carried on past the world's edge; its trees, bushes and beaches
// follow `scatter`, `scrub` and `beach`; and the landscape's key tells such worlds apart. `bun test`.
import { describe, expect, test } from 'bun:test';
import type { LandscapeParams, WorldStructure } from '../src/core/protocol';
import { CountryRoads } from '../src/render/shared/countryRoads';
import { APRON, LandscapeLayout } from '../src/render/shared/nature';
import { MOUNTAINS, plainWorld, reliefWorld, SIDE } from './support/landFixtures';

/** FNV-1a over the bytes of `values` (as 32-bit floats). */
function digest(values: ArrayLike<number>): string {
  const bytes = new Uint8Array(Float32Array.from(values).buffer);
  let h = 0x811c9dc5;
  for (const b of bytes) h = Math.imul(h ^ b, 0x01000193) >>> 0;
  return h.toString(16);
}

type Grown = { woods(): Promise<{ near: Map<string, Float32Array[]>; far: Map<string, Float32Array[]> }>; lawns(world: WorldStructure): Promise<{ key: string; matrices: Float32Array[] }[]>; shadeTerrain(p: Float32Array, c: Float32Array): Promise<void> };

function layout(world: WorldStructure): LandscapeLayout & Grown {
  return new LandscapeLayout(world, (l, w, name) => new CountryRoads(l, w, name)) as unknown as LandscapeLayout & Grown;
}

/** Heights on a grid over the world and its ring (every `step` m), and the terrain's colours there. */
async function terrain(land: LandscapeLayout & Grown, from = -260, to = SIDE + 260, step = 6): Promise<{ heights: string; colours: string }> {
  const n = Math.floor((to - from) / step) + 1;
  const positions = new Float32Array(n * n * 3);
  for (let v = 0; v < n * n; v++) {
    positions[v * 3] = from + (v % n) * step;
    positions[v * 3 + 2] = from + Math.floor(v / n) * step;
  }
  const colors = new Float32Array(n * n * 4);
  await land.shadeTerrain(positions, colors);
  return { heights: digest(positions), colours: digest(colors) };
}

/** The woods' placements, model by model. */
async function woods(land: LandscapeLayout & Grown): Promise<Record<string, string>> {
  const w = await land.woods();
  const out: Record<string, string> = {};
  for (const [part, map] of [['near', w.near], ['far', w.far]] as const) {
    for (const [key, list] of [...map].sort(([a], [b]) => a.localeCompare(b))) out[`${part}:${key}`] = `${list.length}:${digest(list.flatMap((m) => [...m]))}`;
  }
  return out;
}

/** An older town (before worlds): one street, no `world` look. */
function classicTown(): WorldStructure {
  const world = plainWorld();
  world.width = 64;
  world.depth = 40;
  world.rooms = new Array(64 * 40).fill(0);
  world.plots = [];
  world.meta = { seed: 9, name: 'Oldton', streets: [{ x: 0, z: 18, w: 64, d: 6 }] };
  return world;
}

describe('a world without relief draws as it did', () => {
  // (Taken on the code before relief: `ground`, `terrainColour`, `woods` and `lawns` unchanged for such worlds. Taken
  // again when the zones past the edge were mirrored rather than carried straight out: the same inside the world. Taken
  // again when the farmland became whole fields in hedges: its colours, no woods on it, its hedges' bushes and trees,
  // and hay bales on the stubble.)
  const GOLDEN = {
    country: {"heights":"cc4631e9","colours":"122b4e4","woods":{"far:model.bush":"1311:bad2e84b","far:model.bush.small":"542:9086949e","far:model.farm.hayBale":"13:2a5f984a","far:model.pine":"491:88290570","far:model.rock":"679:6f860a79","far:model.spruce":"343:a5ab8f02","far:model.tree":"3120:58b5a013","far:model.tree.birch":"281:1839acf0","far:model.tree.fruit":"28:afaa0365","far:model.tree.maple":"1725:bb6d8737"},"lawns":"4b71cdc7"},
    mountain: {"heights":"99aa25fc","colours":"63b95b90","woods":{"far:model.bush":"359:63a80636","far:model.bush.small":"200:d758ce5a","far:model.pine":"1045:6a45873b","far:model.rock":"271:28fb031c","far:model.spruce":"629:f7a4db69","far:model.tree":"399:353b5678","far:model.tree.birch":"36:d9a2a06e","far:model.tree.fruit":"28:220213d6","far:model.tree.maple":"214:8aa06b91"}},
    classic: {"heights":"f391c28a","colours":"8c5a0ae9","woods":{"near:model.bush":"2:c5faa332","near:model.bush.small":"5:6bb28f64","near:model.spruce":"1:b1c81838","near:model.tree":"11:ae55fa60","near:model.tree.birch":"3:5440220c","near:model.tree.maple":"8:e7c6ae","far:model.bush":"205:852147cf","far:model.bush.small":"105:f84ed439","far:model.pine":"251:e78b8c23","far:model.rock":"155:2ae41767","far:model.spruce":"153:89be27b5","far:model.tree":"560:e6daf014","far:model.tree.birch":"54:461e762d","far:model.tree.maple":"313:d3e19b5e"}},
  };

  test('a big world with zones, no relief', async () => {
    const world = plainWorld();
    const land = layout(world);
    const t = await terrain(land);
    const wd = await woods(land);
    const lawns = await land.lawns(world);
    const got = { ...t, woods: wd, lawns: digest(lawns.flatMap((l) => l.matrices.flatMap((m) => [...m]))) };
    expect(got).toEqual(GOLDEN.country);
  }, 60_000);

  test('a big world among mountains (the classic mountain look), no relief', async () => {
    const land = layout(plainWorld(MOUNTAINS, 'mountain'));
    const got = { ...(await terrain(land)), woods: await woods(land) };
    expect(got).toEqual(GOLDEN.mountain);
  }, 60_000);

  test('an older town', async () => {
    const land = layout(classicTown());
    const got = { ...(await terrain(land, -150, 214, 3)), woods: await woods(land) };
    expect(got).toEqual(GOLDEN.classic);
  }, 60_000);
});

describe('a world with a relief grid', () => {
  /** The valley with no swell or rim (`hills`, `rim` 0): its land is the relief's alone. */
  const bare = (top: number, extra: Partial<LandscapeParams> = {}, kind = 'country') => reliefWorld(top, { hills: 0, rim: 0, slope: 0.4, ...extra }, kind);

  test('the land stands as the grid has it, flat by what is built and no steeper from it than its slope', () => {
    const world = bare(150);
    const land = layout(world);
    let highest = 0;
    let free = 0;
    for (let z = 4; z < SIDE; z += 7) {
      for (let x = 4; x < SIDE; x += 7) {
        const h = land.ground(x, z);
        const dist = land.distanceToTown(x, z);
        if (land.waterEdge(x, z) > -12) continue;
        highest = Math.max(highest, h);
        if (dist <= APRON) expect(h).toBe(0);
        // Never above what the slope lets it rise to.
        expect(h).toBeLessThanOrEqual(0.4 * Math.max(0, dist - APRON) + 1e-9);
        // Where the slope doesn't hold it down: the grid's height, give or take its roughening.
        const r = land.reliefAt(x, z);
        if (0.4 * (dist - APRON) > r * 1.2 + 8) {
          expect(h).toBeGreaterThanOrEqual(r * 0.85 - 1e-6);
          expect(h).toBeLessThanOrEqual(r * 1.15 + 1e-6);
          free++;
        }
      }
    }
    expect(free).toBeGreaterThan(500);
    // The valley's sides climb high (held down near the hamlets by the slope).
    expect(highest).toBeGreaterThan(80);
  });

  test('the grid is read between its cells, and past the world edge as at the edge', () => {
    const world = bare(120);
    const land = layout(world);
    const R = world.meta!.relief!;
    // At a cell's middle, its own height (m).
    expect(land.reliefAt(8 + 16 * 3, 8 + 16 * 30)).toBeCloseTo(R.h[30 * R.cols + 3] / 10, 5);
    // Halfway between two, their mean.
    expect(land.reliefAt(16 * 4, 8 + 16 * 30)).toBeCloseTo((R.h[30 * R.cols + 3] + R.h[30 * R.cols + 4]) / 20, 5);
    for (const z of [20, 300, 500]) {
      expect(land.reliefAt(-80, z)).toBe(land.reliefAt(0, z));
      expect(land.reliefAt(SIDE + 80, z)).toBe(land.reliefAt(SIDE, z));
    }
    expect(land.reliefAt(-50, -50)).toBe(land.reliefAt(0, 0));
  });

  test('a world among mountains: ridges on its high ground, and the range carried on past its edge', () => {
    // (Steep enough that the slope from what is built doesn't hold the high ground down.)
    const plain = layout(bare(140, { slope: 3 }));
    const alpine = layout(bare(140, { slope: 3, mountains: 140 }, 'mountain'));
    // Ridges roughen the high ground (and only it: the low ground is as the plain grid's).
    let rough = 0;
    for (let x = 10; x < SIDE; x += 9) {
      const [low, high] = [[x, 260], [x, 505]] as const;
      if (plain.ground(...low) < 12) expect(alpine.ground(...low)).toBe(plain.ground(...low));
      rough = Math.max(rough, Math.abs(alpine.ground(...high) - plain.ground(...high)));
    }
    expect(rough).toBeGreaterThan(5);
    // Past the edge (away from the way out east) the mountains rise on from the grid's edge: a little over its
    // high ground (the massif is the grid's), well over its low ground.
    const mean = (f: (t: number) => [number, number]) => {
      let sum = 0;
      for (let t = 0; t < 1; t += 0.01) sum += alpine.ground(...f(t));
      return sum / 100;
    };
    const north = [mean((t) => [t * SIDE, SIDE]), mean((t) => [t * SIDE, SIDE + 180])];
    const west = [mean((t) => [0, SIDE / 2 + (t * SIDE) / 2]), mean((t) => [-180, SIDE / 2 + (t * SIDE) / 2])];
    expect(north[0]).toBeGreaterThan(100);
    expect(north[1]).toBeGreaterThan(north[0] + 4);
    expect(north[1]).toBeLessThan(north[0] + 60);
    expect(west[1]).toBeGreaterThan(west[0] + 50);
    // Inside the world they add nothing of their own.
    expect(alpine.ground(256, 256)).toBe(0);
  });

  test('a treeline at about 0.3 of the mountains, not the classic 12 m', async () => {
    const land = layout(bare(150, { mountains: 150, forest: 0.1 }, 'mountain'));
    const w = await land.woods();
    const ys = [...w.far].filter(([k]) => k !== 'model.tree.fruit').flatMap(([, list]) => list.map((m) => m[13]));
    expect(Math.max(...ys)).toBeLessThan(150 * 0.3 + 10);
    // (Above the classic line there are trees now.)
    expect(ys.filter((y) => y > 12 + 10).length).toBeGreaterThan(20);
  });

  test('without a slope given, the grid rises no steeper than 0.5 from what is built', () => {
    const land = layout(reliefWorld(150, { hills: 0, rim: 0 }));
    for (let z = 4; z < SIDE; z += 11) for (let x = 4; x < SIDE; x += 11) if (land.waterEdge(x, z) < -12) expect(land.ground(x, z)).toBeLessThanOrEqual(0.5 * Math.max(0, land.distanceToTown(x, z) - APRON) + 1e-9);
  });
});

describe('woods, scrub and beaches from the landscape', () => {
  const count = async (extra: Partial<LandscapeParams>) => {
    const w = await layout(plainWorld(extra)).woods();
    const n = (k: string) => w.far.get(k)?.length ?? 0;
    const all = [...w.far.values()].reduce((s, l) => s + l.length, 0);
    return { all, bushes: n('model.bush') + n('model.bush.small') };
  };

  test('scatter thins or thickens the trees out on open land', async () => {
    const [few, usual, many] = [await count({ scatter: 0.5 }), await count({}), await count({ scatter: 2 })];
    expect(few.all).toBeLessThan(usual.all * 0.85);
    expect(many.all).toBeGreaterThan(usual.all * 1.15);
  });

  test('scrub is the share of bushes', async () => {
    const [usual, heath] = [await count({}), await count({ scrub: 0.6 })];
    expect(usual.bushes / usual.all).toBeLessThan(0.3);
    expect(heath.bushes / heath.all).toBeGreaterThan(0.5);
  });

  test('beach: how far the sand reaches from the shore', () => {
    const wide = layout(plainWorld({ beach: 8 }));
    const narrow = layout(plainWorld({ beach: 3 }));
    const out = { r: 0, g: 0, b: 0 };
    // Points round the pond 5 m from its shore: sand with the usual beach, grass with a narrow one.
    let tried = 0;
    for (let a = 0; a < Math.PI * 2; a += 0.2) {
      for (let r = 20; r < 70; r += 0.5) {
        const [x, z] = [60 + Math.cos(a) * r, 300 + Math.sin(a) * r];
        const e = wide.waterEdge(x, z);
        if (e > -5.5 || e < -6) continue;
        expect(wide.terrainColour(x, z, wide.heightAt(x, z), out)).toBeLessThan(0.6);
        expect(narrow.terrainColour(x, z, narrow.heightAt(x, z), out)).toBeGreaterThan(0.6);
        tried++;
        break;
      }
    }
    expect(tried).toBeGreaterThan(10);
  });
});

describe("the landscape's key", () => {
  test('tells apart worlds that differ only in their options, zones or relief', () => {
    const plain = plainWorld();
    const keys = new Set([LandscapeLayout.key(plain)]);
    const withOptions = plainWorld();
    withOptions.meta!.world!.options = { town: 'town', villages: 'few', hills: 'rolling', woods: 'wooded' };
    keys.add(LandscapeLayout.key(withOptions));
    const fewerZones = plainWorld();
    fewerZones.meta!.zones = fewerZones.meta!.zones!.slice(1);
    keys.add(LandscapeLayout.key(fewerZones));
    const relief = reliefWorld(120, {});
    keys.add(LandscapeLayout.key(relief));
    const other = reliefWorld(120, {});
    other.meta!.relief!.h[500] += 1;
    keys.add(LandscapeLayout.key(other));
    expect(keys.size).toBe(5);
    // The same world, the same key.
    expect(LandscapeLayout.key(reliefWorld(120, {}))).toBe(LandscapeLayout.key(relief));
  });
});
