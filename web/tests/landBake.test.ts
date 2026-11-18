// The land surveyed as a height field (`render/shared/landBake.ts` `bakeLand`) and drawn from it
// (`LandscapeLayout.setLand`): the survey stands where the land stood, to the centimetre at every sample; drawn from
// it, the ground is the field's inside the world and the landscape's past it (moved by the edge's difference,
// fading out); the water's edge is the field's; its water above the sea's level is drawn as quads at its level,
// sloping where a river steps down and meeting with no gap, the sea's left to the sheet; a stroke grows again only the
// woods of the squares it reached; a ray finds the ground. `bun test`.
import { describe, expect, test } from 'bun:test';
import type { WorldStructure } from '../src/core/protocol';
import { COVER, coverSpan, encodeLand, LAND_CELL, NONE, SEA_CM, type Land } from '../src/game/land';
import { CountryRoads } from '../src/render/shared/countryRoads';
import { landZones } from '../src/game/landEdit';
import { bakeLand, copyLand, EDGE_DRY, EDGE_POND, EDGE_SEA, edgeWater, edgeWaterAt, EXIT_MOST, landExits, marchToGround, pondQuads, wetDistance, wetEdgeAt } from '../src/render/shared/landBake';
import { LAND_BLOCK, LandscapeLayout, WATER_Y, type Woods } from '../src/render/shared/nature';
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

  test('a river stepping down slopes between its levels (a rapid), and its cells meet with no gap', () => {
    // A channel along x over land 2 m up, its bed at 0: the water at 1.2 m west of x 60, 0.8 m east of it (a step
    // between neighbouring samples), then at the sea's level east of x 100 (the sea's own samples left to the sheet).
    const land = flat();
    land.h.fill(200);
    for (let j = 28; j < 36; j++) {
      for (let i = 4; i < 60; i++) {
        const k = j * 64 + i;
        land.h[k] = i < 50 ? 0 : -100;
        land.wl[k] = i * 2 < 60 ? 120 : i * 2 < 100 ? 80 : SEA_CM;
      }
    }
    const quads = pondQuads(land, 0, 0, 128, 128)!;
    const { positions: p, normals: n, indices } = quads;
    // Across the step, the water slopes from one level down to the next over the cell between them, its normal
    // leaning downstream; flat (straight up) away from it.
    let [rapid, flatUp] = [0, 0];
    for (let v = 0; v < p.length; v += 3) {
      const [x, y] = [p[v], p[v + 1]];
      expect(y).toBeGreaterThanOrEqual(SEA_CM / 100 - 1e-6);
      expect(y).toBeLessThanOrEqual(1.2 + 1e-6);
      // (The cells from x 58 to 60: their points at 1.2 m on the west side and down to 0.8 m on the east, leaning east.)
      if (x >= 58 && x <= 60 && n[v] > 0.1) {
        if (x === 58) expect(y).toBeCloseTo(1.2, 6);
        if (x === 60 && Math.abs(y - 0.8) < 1e-6) rapid++;
      }
      if (x < 50 && x > 20) {
        expect(y).toBeCloseTo(1.2, 6);
        expect(n[v + 1]).toBe(1);
        flatUp++;
      }
    }
    expect(rapid).toBeGreaterThan(0);
    expect(flatUp).toBeGreaterThan(0);
    // No gap: every edge only one triangle has lies on the shore (the water there within a lap of the ground), never
    // out in the water between two cells (but where it meets the sea's sheet).
    const key = (a: number) => `${p[a * 3].toFixed(4)},${p[a * 3 + 1].toFixed(4)},${p[a * 3 + 2].toFixed(4)}`;
    const edges = new Map<string, number>();
    for (let t = 0; t < indices.length; t += 3) {
      for (const [a, b] of [[indices[t], indices[t + 1]], [indices[t + 1], indices[t + 2]], [indices[t + 2], indices[t]]]) {
        const id = [key(a), key(b)].sort().join('|');
        edges.set(id, (edges.get(id) ?? 0) + 1);
      }
    }
    let open = 0;
    for (const [id, count] of edges) {
      if (count !== 1) continue;
      open++;
      const [a, b] = id.split('|').map((s) => s.split(',').map(Number));
      // (Where the river meets the sea: the sheet goes on from there.)
      if (a[1] === SEA_CM / 100 && b[1] === SEA_CM / 100) continue;
      const [mx, my, mz] = [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2, (a[2] + b[2]) / 2];
      const [i, j] = [Math.floor(mx / LAND_CELL), Math.floor(mz / LAND_CELL)];
      const [u, v] = [mx / LAND_CELL - i, mz / LAND_CELL - j];
      const g = ((land.h[j * 64 + i] * (1 - u) + land.h[j * 64 + i + 1] * u) * (1 - v) + (land.h[(j + 1) * 64 + i] * (1 - u) + land.h[(j + 1) * 64 + i + 1] * u) * v) / 100;
      expect(Math.abs(my - g)).toBeLessThan(0.6);
    }
    expect(open).toBeGreaterThan(0);
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

describe('past the edge only the sea runs on', () => {
  /** A flat, dry land `n` samples a side, 2 m up. */
  const flat = (n = 64): Land => ({ cols: n, rows: n, h: new Int16Array(n * n).fill(200), wl: new Int16Array(n * n).fill(NONE), cover: new Uint8Array(coverSpan(n) ** 2), seeded: 'drawn', edited: false });

  test('each edge sample knows the nearest water along the edge, round the corners', () => {
    const land = flat();
    const wet = (i: number, j: number, level: number) => ([land.h[j * 64 + i], land.wl[j * 64 + i]] = [-200, level]);
    // A river 5 samples wide out through the east side (x = 63), rows 20..24; the sea along the south (z = 0) from x 0 to 9.
    for (let j = 20; j <= 24; j++) for (let i = 40; i < 64; i++) wet(i, j, 100);
    for (let i = 0; i < 10; i++) wet(i, 0, SEA_CM);
    const edge = edgeWater(land, 8);
    // (Past the east side by its rows; 8 m along it is 4 samples.)
    const east = (j: number) => edgeWaterAt(land, edge, 63 * LAND_CELL + 5, j * LAND_CELL);
    expect([east(22), east(20), east(16), east(15), east(28), east(29)]).toEqual([EDGE_POND, EDGE_POND, EDGE_POND, EDGE_DRY, EDGE_POND, EDGE_DRY]);
    // The sea, south of the land, and round its corner up the west side.
    expect(edgeWaterAt(land, edge, 8, -30)).toBe(EDGE_SEA);
    expect(edgeWaterAt(land, edge, -10, 6)).toBe(EDGE_SEA);
    expect(edgeWaterAt(land, edge, -10, 20)).toBe(EDGE_DRY);
    // Inside the land's samples: none.
    expect(edgeWaterAt(land, edge, 60, 44)).toBe(EDGE_DRY);
    // Past the river's end the land is dry, its sand round the end only; past the sea, the sea.
    const sd = wetDistance(land);
    expect(wetEdgeAt(land, sd, 63 * LAND_CELL + 4, 44, edge)).toBeCloseTo(-4, 6);
    expect(wetEdgeAt(land, sd, 63 * LAND_CELL + 4, 44)).toBeGreaterThan(0);
    expect(wetEdgeAt(land, sd, 8, -30, edge)).toBeGreaterThan(30);
  });

  /** The plain world's land surveyed, with a river 20 m wide carved from x 300 east out through its edge, its level `level` cm. */
  const carved = (level = 50) => {
    const field = bakeLand(layout(plainWorld()), SIDE, SIDE);
    for (let j = 55; j <= 64; j++) {
      for (let i = 150; i < field.cols; i++) {
        const k = j * field.cols + i;
        [field.h[k], field.wl[k]] = [Math.min(field.h[k], level - 200), level];
      }
    }
    return field;
  };

  test('a river through the edge ends there, behind a bank: no sand, no bed, no water past it, step 1 or 2', () => {
    const field = carved();
    const world = plainWorld();
    // Step 1: the world's own zones; step 2: the land's water among them (as the town laid on it has them).
    const laid = { ...world, meta: { ...world.meta!, zones: [...world.meta!.zones!.filter((z) => z.kind !== 'water'), ...landZones(field, { water: true })] } } as WorldStructure;
    for (const w of [world, laid]) {
      const drawn = layout(w);
      drawn.setLand(copyLand(field));
      const z = 119;
      // (In the land: the river.)
      expect(drawn.waterEdge(SIDE - 6, z)).toBeGreaterThan(0);
      // Past the edge: its end's bank, then the land's own, never under the sea's level; its sand only at the end.
      expect(drawn.ground(SIDE - 1, z)).toBeGreaterThanOrEqual(0.75 - 1e-6);
      expect(drawn.ground(SIDE + 1, z)).toBeGreaterThan(0.5);
      for (let x = SIDE + 1; x < SIDE + 400; x += 7) {
        expect(drawn.ground(x, z)).toBeGreaterThan(WATER_Y + 0.2);
        expect(drawn.waterEdge(x, z)).toBeLessThan(0);
        if (x > SIDE + 12) expect(drawn.waterEdge(x, z)).toBeLessThan(-8);
      }
      // (Beside it as before: the field's edge followed out.)
      expect(Math.abs(drawn.ground(SIDE + 1, 300) - drawn.ground(SIDE - 1, 300))).toBeLessThan(0.5);
    }
    // A river run down under the sea's level ends behind a bank over the sea's sheet.
    const low = layout(world);
    low.setLand(carved(-60));
    for (let x = SIDE - 1; x < SIDE + 80; x += 3) expect(low.ground(x, 119)).toBeGreaterThan(WATER_Y);
  });

  test('a generated land reaches the edge only with the sea: drawn past it as before', () => {
    for (const world of [plainWorld(), islandWorld()]) {
      const baked = bakeLand(layout(world), SIDE, SIDE);
      const edge = edgeWater(baked, 20)!;
      expect(edge.includes(EDGE_POND)).toBe(false);
      const drawn = layout(world);
      drawn.setLand(baked);
      const sd = wetDistance(baked);
      for (let a = 0; a < 360; a += 5) {
        const [x, z] = [SIDE / 2 + Math.cos((a * Math.PI) / 180) * 400, SIDE / 2 + Math.sin((a * Math.PI) / 180) * 400];
        expect(drawn.waterEdge(x, z)).toBe(wetEdgeAt(baked, sd, x, z));
      }
    }
    // The island's sea runs on past it all round.
    const island = layout(islandWorld());
    island.setLand(bakeLand(layout(islandWorld()), SIDE, SIDE));
    for (const [x, z] of [[-30, 256], [SIDE + 30, 100], [256, -60], [256, SIDE + 60]]) expect(island.waterEdge(x, z)).toBeGreaterThan(0);
  });
});

describe("a river's way out opens into a valley past the edge", () => {
  /**
   * The plain world's land surveyed, with a river 20 m wide (rows 55..64) carved east out through its edge, at `level`
   * cm, its banks graded at 30 % up to the land (as a generated river's are; the land past the edge follows its own).
   */
  const carved = (level: number) => {
    const field = bakeLand(layout(plainWorld()), SIDE, SIDE);
    for (let j = 45; j <= 74; j++) {
      for (let i = 150; i < field.cols; i++) {
        const k = j * field.cols + i;
        const off = Math.max(0, Math.abs(j - 59.5) - 5) * LAND_CELL;
        if (off === 0) [field.h[k], field.wl[k]] = [Math.min(field.h[k], level - 200), level];
        else field.h[k] = Math.min(field.h[k], level - 200 + off * 30);
      }
    }
    return field;
  };
  /** The steepest the ground rises (rise over run, either way) round (x, z), over 1 m. */
  const steepness = (l: LandscapeLayout, x: number, z: number) => Math.hypot(l.ground(x + 0.5, z) - l.ground(x - 0.5, z), l.ground(x, z + 0.5) - l.ground(x, z - 0.5));
  /** tan 35°: the steepest a valley's side may be. */
  const LIMIT = Math.tan((35 * Math.PI) / 180);

  test('a river at the sea\'s level: the channel widens out, the land beside it slopes back no steeper than 35°', () => {
    const field = carved(SEA_CM);
    const exits = landExits(field, EXIT_MOST);
    expect(exits).toEqual([{ side: 1, at: 119, half: 10, sea: true, level: SEA_CM / 100 }]);
    const drawn = layout(plainWorld());
    drawn.setLand(field);
    let steepest = 0;
    for (let d = 4; d <= 300; d += 4) {
      // (Across the river's way out and well beyond the valley's floor, either side.)
      for (let a = -140 - d * 0.5; a <= 140 + d * 0.5; a += 3) steepest = Math.max(steepest, steepness(drawn, SIDE + d, 119 + a));
      // Its water goes on, wider further out, under the sea's sheet.
      expect(drawn.ground(SIDE + d, 119)).toBeLessThan(WATER_Y);
      expect(drawn.waterEdge(SIDE + d, 119)).toBeGreaterThan(0);
    }
    expect(steepest).toBeLessThan(LIMIT);
    const wet = (d: number) => [...Array(200).keys()].filter((k) => drawn.waterEdge(SIDE + d, 19 + k) > 0).length;
    expect(wet(200)).toBeGreaterThan(wet(20) + 30);
    // Beside the water a floor over the sea's level, not flooded; the hills come back further off.
    expect(drawn.ground(SIDE + 150, 119 + 40)).toBeGreaterThan(WATER_Y + 0.3);
    expect(drawn.ground(SIDE + 150, 119 + 40)).toBeLessThan(5);
  });

  test('a river above it (one drawn to the edge) ends behind its bank, the valley dry past it', () => {
    const drawn = layout(plainWorld());
    drawn.setLand(carved(50));
    let steepest = 0;
    for (let d = 4; d <= 300; d += 4) {
      for (let a = -140 - d * 0.5; a <= 140 + d * 0.5; a += 3) steepest = Math.max(steepest, steepness(drawn, SIDE + d, 119 + a));
      expect(drawn.ground(SIDE + d, 119)).toBeGreaterThanOrEqual(0.75 - 1e-6);
      expect(drawn.waterEdge(SIDE + d, 119)).toBeLessThan(0);
    }
    expect(steepest).toBeLessThan(LIMIT);
  });

  test('a coast is no way out: the sea all round an island is left as it was', () => {
    expect(landExits(bakeLand(layout(islandWorld()), SIDE, SIDE), EXIT_MOST)).toEqual([]);
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
