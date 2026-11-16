// The land surveyed as a height field (`render/shared/landBake.ts` `bakeLand`) and drawn from it
// (`LandscapeLayout.setLand`): the survey stands where the land stood, to the centimetre at every sample; drawn from
// it, the ground is the field's inside the world and the landscape's past it (moved by the edge's difference,
// fading out); the water's edge is the field's; its water above the sea's level is drawn as quads at its level,
// the sea's left to the sheet; a stroke grows again only the woods of the squares it reached; a ray finds the
// ground. `bun test`.
import { describe, expect, test } from 'bun:test';
import type { WorldStructure } from '../src/core/protocol';
import { COVER, coverSpan, encodeLand, LAND_CELL, NONE, SEA_CM, type Land } from '../src/game/land';
import { CountryRoads } from '../src/render/shared/countryRoads';
import { bakeLand, copyLand, marchToGround, pondQuads, wetDistance, wetEdgeAt } from '../src/render/shared/landBake';
import { LAND_BLOCK, LandscapeLayout, type Woods } from '../src/render/shared/nature';
import { islandWorld, plainWorld, SIDE } from './support/landFixtures';

type Grown = {
  woods(): Promise<Woods>;
  woodsAgain(box: [number, number, number, number] | null): Promise<{ woods: Woods; changed: Set<string> }>;
};

function layout(world: WorldStructure): LandscapeLayout & Grown {
  return new LandscapeLayout(world, (l, w, name) => new CountryRoads(l, w, name)) as unknown as LandscapeLayout & Grown;
}

describe('the land surveyed', () => {
  const world = plainWorld();
  const land = layout(world);
  const start = performance.now();
  const baked = bakeLand(land, SIDE, SIDE);
  const ms = performance.now() - start;

  test(`each sample's height is the land's as drawn, to the centimetre (${ms.toFixed(0)} ms for ${baked.cols} × ${baked.rows})`, () => {
    expect(baked.cols).toBe(SIDE / LAND_CELL);
    let worst = 0;
    for (let j = 0; j < baked.rows; j++) for (let i = 0; i < baked.cols; i++) worst = Math.max(worst, Math.abs(baked.h[j * baked.cols + i] / 100 - land.ground(i * LAND_CELL, j * LAND_CELL)));
    expect(worst).toBeLessThanOrEqual(0.005 + 1e-9);
  });

  test('the sea’s level past the water’s edge, and the cover of the zones at 4 m', () => {
    let wet = 0;
    for (let j = 0; j < baked.rows; j++) {
      for (let i = 0; i < baked.cols; i++) {
        const k = j * baked.cols + i;
        const e = land.waterEdge(i * LAND_CELL, j * LAND_CELL);
        expect(baked.wl[k]).toBe(e > 0 ? SEA_CM : NONE);
        if (e > 0) wet++;
      }
    }
    // (The fixture's pond.)
    expect(wet).toBeGreaterThan(400);
    const cw = coverSpan(baked.cols);
    const kinds = new Set(baked.cover);
    expect([...kinds].sort()).toEqual([COVER.open, COVER.woods, COVER.meadow, COVER.fields].sort());
    // (The woods' blob round (400, 120).)
    expect(baked.cover[Math.floor(120 / 4) * cw + Math.floor(400 / 4)]).toBe(COVER.woods);
  });

  test('drawn from it, the ground is the field’s inside the world, the landscape’s well past it', () => {
    const drawn = layout(world);
    drawn.setLand(copyLand(baked));
    for (let j = 0; j < baked.rows; j += 7) for (let i = 0; i < baked.cols; i += 5) expect(drawn.ground(i * LAND_CELL, j * LAND_CELL)).toBe(baked.h[j * baked.cols + i] / 100);
    // Between samples: between their heights.
    const k = 100 * baked.cols + 100;
    expect(drawn.ground(201, 200)).toBeCloseTo((baked.h[k] + baked.h[k + 1]) / 200, 6);
    // Past the edge: the landscape's own from 60 m out; at the edge the field's.
    for (const z of [40, 260, 470]) {
      expect(drawn.ground(-61, z)).toBe(land.ground(-61, z));
      expect(drawn.ground(SIDE + 70, z)).toBe(land.ground(SIDE + 70, z));
      expect(Math.abs(drawn.ground(-0.001, z) - drawn.ground(0, z))).toBeLessThan(0.01);
    }
    // Near the edge, off the land as it was drawn by no more than the field is between its samples (and in its last
    // 2 m, which holds the last sample's height).
    let worst = 0;
    for (let x = -40; x < SIDE + 40; x += 13) worst = Math.max(worst, Math.abs(drawn.ground(x, -20) - land.ground(x, -20)));
    expect(worst).toBeLessThan(0.3);
  });

  test('raised inside the world, the land past the edge follows it there and less further out', () => {
    const field = copyLand(baked);
    for (let j = 0; j < field.rows; j++) field.h[j * field.cols] += 500;
    const drawn = layout(world);
    drawn.setLand(field);
    const z = 256;
    const lift = (x: number) => drawn.ground(x, z) - land.ground(x, z);
    expect(lift(0)).toBeCloseTo(5, 1);
    expect(lift(-20)).toBeGreaterThan(2);
    expect(lift(-20)).toBeLessThan(5);
    expect(lift(-60)).toBeCloseTo(0, 6);
  });

  test("the landscape's key tells a world with a field apart, its zones aside", () => {
    const withLand = { ...world, meta: { ...world.meta!, land: encodeLand(baked) } } as WorldStructure;
    expect(LandscapeLayout.key(withLand)).toBe(`${LandscapeLayout.baseKey(world)}:L`);
    expect(LandscapeLayout.key({ ...withLand, meta: { ...withLand.meta!, zones: [] } } as WorldStructure)).toBe(LandscapeLayout.key(withLand));
    expect(LandscapeLayout.key(world)).not.toBe(LandscapeLayout.key(withLand));
    // (Built from it, it is drawn from it from the start.)
    const loaded = layout(withLand);
    expect(loaded.heightField).not.toBeNull();
    expect(loaded.ground(300, 300)).toBe(baked.h[150 * baked.cols + 150] / 100);
    // Let go of a field given: back to the world's own.
    loaded.setLand(copyLand(baked));
    loaded.setLand(null);
    expect(loaded.heightField).not.toBeNull();
  });
});

describe("the field's water", () => {
  /** A flat, dry land `n` samples a side, 2 m up. */
  const flat = (n = 64): Land => ({ cols: n, rows: n, h: new Int16Array(n * n).fill(200), wl: new Int16Array(n * n).fill(NONE), cover: new Uint8Array(coverSpan(n) ** 2), seeded: 'drawn', edited: false });

  test('its edge: inside the water above 0, outside below, carried on past the edge', () => {
    const land = flat();
    for (let j = 10; j < 20; j++) for (let i = 10; i < 20; i++) land.wl[j * 64 + i] = 300;
    const sd = wetDistance(land);
    expect(sd).not.toBeNull();
    expect(wetEdgeAt(land, sd, 30, 30)).toBeGreaterThan(0);
    expect(wetEdgeAt(land, sd, 4, 4)).toBeLessThan(0);
    expect(wetEdgeAt(land, sd, 80, 30)).toBeCloseTo(-(80 - 38) + 1, 0);
    expect(wetDistance(flat())).toBeNull();
    expect(wetEdgeAt(land, null, 0, 0)).toBe(-Infinity);
  });

  test('water above the sea’s level is a quad a cell at its level; the sea’s is left to the sheet', () => {
    // A pond at 3 m over 4 × 3 samples 2 m up, in land 4 m up; and the sea over a strip.
    const land = flat();
    land.h.fill(400);
    for (let j = 10; j < 13; j++) for (let i = 10; i < 14; i++) [land.h[j * 64 + i], land.wl[j * 64 + i]] = [200, 300];
    for (let j = 40; j < 64; j++) for (let i = 0; i < 64; i++) [land.h[j * 64 + i], land.wl[j * 64 + i]] = [-100, SEA_CM];
    const quads = pondQuads(land, 0, 0, 128, 128)!;
    // (Every cell with a corner in it: 5 × 4.)
    expect(quads.cells).toBe(5 * 4);
    for (let v = 1; v < quads.positions.length; v += 3) expect(quads.positions[v]).toBe(3);
    // Wound as the sheet (front up), and cut where the ground meets the level: halfway (and a lap up the bank)
    // between a sample under 1 m of it and one 1 m over it.
    const p = quads.positions;
    for (let t = 0; t < quads.indices.length; t += 3) {
      const [a, b, c] = [quads.indices[t], quads.indices[t + 1], quads.indices[t + 2]];
      expect((p[b * 3] - p[a * 3]) * (p[c * 3 + 2] - p[a * 3 + 2]) - (p[b * 3 + 2] - p[a * 3 + 2]) * (p[c * 3] - p[a * 3])).toBeGreaterThan(0);
    }
    let [x0, x1] = [Infinity, -Infinity];
    for (let v = 0; v < p.length; v += 3) [x0, x1] = [Math.min(x0, p[v]), Math.max(x1, p[v])];
    expect(x0).toBeCloseTo(20 - 1.08, 5);
    expect(x1).toBeCloseTo(26 + 1.08, 5);
    // Split between two tiles, each cell once.
    const [a, b] = [pondQuads(land, 0, 0, 20, 128)!, pondQuads(land, 20, 0, 128, 128)!];
    expect(a.cells + b.cells).toBe(quads.cells);
    // None where only the sea is.
    expect(pondQuads(land, 0, 70, 128, 128)).toBeNull();
    // A sample at the level, not under it, is dry.
    land.wl.fill(NONE, 0, 40 * 64);
    land.wl[5] = 400;
    expect(pondQuads(land, 0, 0, 128, 70)).toBeNull();
  });

  test('an island’s survey: its sea at the sea’s level all round', () => {
    const island = layout(islandWorld());
    const baked = bakeLand(island, SIDE, SIDE);
    expect(baked.wl[0]).toBe(SEA_CM);
    expect(baked.h[0]).toBeLessThan(SEA_CM);
    const drawn = layout(islandWorld());
    drawn.setLand(baked);
    expect(drawn.seaSheet).toBe(true);
    expect(drawn.watered).toBe(true);
    // (Its sea's edge where the field's water ends.)
    expect(drawn.waterEdge(0, 0)).toBeGreaterThan(0);
    expect(drawn.waterEdge(256, 256)).toBeLessThan(0);
  });
});

describe('woods on a field grow square by square', () => {
  test('a stroke grows again only the squares it reached', async () => {
    const world = plainWorld();
    const field = bakeLand(layout(world), SIDE, SIDE);
    const land = layout(world);
    land.setLand(field);
    const before = await land.woods();
    const all = (w: Woods) => [...w.far.values()].flat();
    // Woods painted over a meadow 80 m round (256, 420), then grown again there.
    const cw = coverSpan(field.cols);
    for (let j = 85; j < 125; j++) for (let i = 44; i < 84; i++) field.cover[j * cw + i] = COVER.woods;
    land.landChanged({ i0: 88, j0: 170, i1: 168, j1: 250 });
    const box: [number, number, number, number] = [176, 340, 336, 500];
    const { woods: after, changed } = await land.woodsAgain(box);
    expect(changed.size).toBeGreaterThan(0);
    // Out of the squares reached (and the zones' soft edge), every tree where it stood.
    const reach = (m: Float32Array) => m[12] >= Math.floor(box[0] / LAND_BLOCK) * LAND_BLOCK - 1 && m[12] < (Math.floor(box[2] / LAND_BLOCK) + 1) * LAND_BLOCK + 1 && m[14] >= Math.floor(box[1] / LAND_BLOCK) * LAND_BLOCK - 1 && m[14] < (Math.floor(box[3] / LAND_BLOCK) + 1) * LAND_BLOCK + 1;
    const key = (m: Float32Array) => [...m].map((v) => v.toFixed(4)).join();
    const outside = (w: Woods) => new Set(all(w).filter((m) => !reach(m) && m[11] === 0 && !isFruit(w, m)).map(key));
    const isFruit = (w: Woods, m: Float32Array) => (w.far.get('model.tree.fruit') ?? []).includes(m);
    expect(outside(after)).toEqual(outside(before));
    // In them, more trees on the new woods.
    const count = (w: Woods) => all(w).filter((m) => m[12] > 190 && m[12] < 320 && m[14] > 350 && m[14] < 490).length;
    expect(count(after)).toBeGreaterThan(count(before) * 1.5);
  }, 60_000);
});

describe('a ray meets the ground', () => {
  const hill = (x: number, z: number) => 10 * Math.exp(-((x - 50) ** 2 + z ** 2) / 400);

  test('marched and bisected to where it crosses', () => {
    const origin = { x: 0, y: 30, z: 0 };
    const len = Math.hypot(1, -0.5);
    const hit = marchToGround(origin, { x: 1 / len, y: -0.5 / len, z: 0 }, hill)!;
    expect(hit).not.toBeNull();
    expect(Math.abs(origin.y - 0.5 * hit.x - hill(hit.x, hit.z))).toBeLessThan(1e-3);
    expect(hit.y).toBeCloseTo(hill(hit.x, hit.z), 9);
    // (The hill's near face, before its top at 50.)
    expect(hit.x).toBeLessThan(50);
  });

  test('none looking up, nor from under the ground', () => {
    expect(marchToGround({ x: 0, y: 5, z: 0 }, { x: 0, y: 1, z: 0 }, hill, 500)).toBeNull();
    expect(marchToGround({ x: 50, y: 5, z: 0 }, { x: 0, y: -1, z: 0 }, hill)).toBeNull();
  });
});
