// The land as a height field (`game/land.ts`): it codes to `meta.land` and back to the same bytes (any heights, any
// water, tiles cut short at the land's edge), a flat tile codes to a few bytes, a stroke's tiles code on their own,
// the height between samples is bilinear and held past the edge, and a 1024 m land of hills, a town and a lake codes
// to a size the town file can carry (printed). `bun test tests/land.test.ts`.
import { describe, expect, test } from 'bun:test';
import { blankLand, COVER, coverSpan, decodeLand, encodeLand, encodeTiles, flowCode, flowOf, FLOW_WAYS, heightAt, landHash, NONE, SEA_CM, TILE, tilesOf, wetAt, type Land } from '../src/game/land';
import { fbm, mulberry32 } from '../src/game/noise';

/** A land of `cols` × `rows` samples, its heights, water levels and cover from `fill`. */
function landOf(cols: number, rows: number, fill: (k: number) => [number, number, number]): Land {
  const land = blankLand(cols * 2, rows * 2);
  for (let k = 0; k < cols * rows; k++) [land.h[k], land.wl[k]] = fill(k);
  for (let k = 0; k < land.cover.length; k++) land.cover[k] = fill(k)[2];
  return land;
}

const same = (a: Land, b: Land) => {
  expect(b.cols).toBe(a.cols);
  expect(b.rows).toBe(a.rows);
  expect(b.seeded).toBe(a.seeded);
  expect(Buffer.from(b.h.buffer).equals(Buffer.from(a.h.buffer))).toBe(true);
  expect(Buffer.from(b.wl.buffer).equals(Buffer.from(a.wl.buffer))).toBe(true);
  expect(Buffer.from(b.cover).equals(Buffer.from(a.cover))).toBe(true);
};

/** Through JSON, as the town file and saves carry it. */
const roundTrip = (land: Land) => decodeLand(JSON.parse(JSON.stringify(encodeLand(land))));

describe('coding', () => {
  test('random heights, water and cover, tiles cut short at the edge, read back the same', () => {
    const rand = mulberry32(7);
    // (150 × 97: neither a multiple of a tile; the last column of tiles 22 wide, the last row 33 down.)
    const land = landOf(150, 97, () => [Math.floor(rand() * 65536) - 32768, rand() < 0.3 ? NONE : Math.floor(rand() * 65536) - 32768, Math.floor(rand() * 4)]);
    land.seeded = 'follow';
    const file = encodeLand(land);
    expect(file.h.length).toBe(Math.ceil(150 / TILE) * Math.ceil(97 / TILE));
    same(land, roundTrip(land));
  });

  test('every value there is, and runs of each length, read back the same', () => {
    const land = landOf(256, 256, (k) => [k - 32768, 32767 - k, k % 4]);
    same(land, roundTrip(land));
    // Smooth slopes with flat stretches and steps: zero runs of every length, across rows and tiles.
    const rand = mulberry32(11);
    let v = 0;
    const stepped = landOf(200, 130, () => {
      if (rand() < 0.08) v += Math.floor(rand() * 400) - 200;
      return [v, rand() < 0.5 ? NONE : SEA_CM, rand() < 0.95 ? 0 : 1];
    });
    same(stepped, roundTrip(stepped));
  });

  test("a river's way codes by tile and reads back the same; a land with none carries no flow at all", () => {
    const rand = mulberry32(19);
    const land = landOf(150, 97, () => [Math.floor(rand() * 2000), rand() < 0.5 ? NONE : 1500, 0]);
    // No way anywhere: no `flow` in the file, nor in what a stroke sends, and the hash as it was.
    const hash = landHash(land);
    expect(encodeLand(land).flow).toBeUndefined();
    expect(encodeTiles(land, [0, 1]).flow).toBeUndefined();
    land.flow = new Uint8Array(land.cols * land.rows);
    expect(encodeLand(land).flow).toBeUndefined();
    expect(landHash(land)).toBe(hash);
    // A river's way in tiles 0 and 4 only (the rest of the file's flow '').
    for (let j = 10; j < 90; j++) for (let i = 5; i < 60; i++) land.flow[j * land.cols + i] = 1 + Math.floor(rand() * FLOW_WAYS);
    const file = encodeLand(land);
    expect(file.flow!.map((t) => t !== '')).toEqual([true, false, false, true, false, false]);
    const back = roundTrip(land);
    same(land, back);
    expect(Buffer.from(back.flow!).equals(Buffer.from(land.flow))).toBe(true);
    expect(landHash(land)).not.toBe(hash);
    // Tiles on their own: the way where it is, '' where there is none.
    expect(encodeTiles(land, [0, 2]).flow).toEqual({ 0: file.flow![0], 2: '' });
    // A file without flow reads back still.
    const { flow: _flow, ...still } = file;
    expect(decodeLand(still).flow).toBeUndefined();
  });

  test('a way codes to within half a step of itself, all round', () => {
    expect(flowCode(0, 0)).toBe(0);
    expect(flowOf(0)).toBeNull();
    for (let a = -Math.PI; a < Math.PI; a += 0.013) {
      const code = flowCode(Math.cos(a) * 3, Math.sin(a) * 3);
      expect(code).toBeGreaterThanOrEqual(1);
      expect(code).toBeLessThanOrEqual(FLOW_WAYS);
      const [x, z] = flowOf(code)!;
      expect(Math.acos(Math.min(1, x * Math.cos(a) + z * Math.sin(a)))).toBeLessThanOrEqual(Math.PI / FLOW_WAYS + 1e-9);
    }
  });

  test('no water anywhere codes every tile of it as nothing', () => {
    const rand = mulberry32(3);
    const land = landOf(130, 70, () => [Math.floor(rand() * 2000), NONE, 0]);
    const file = encodeLand(land);
    expect(file.wl.every((t) => t === '')).toBe(true);
    expect(file.cover).toEqual([coverSpan(130) * coverSpan(70), 0]);
    same(land, roundTrip(land));
  });

  test('a flat tile codes to under 16 bytes, at any height', () => {
    for (const h of [0, 1234, -500, 32767, -32768]) {
      const land = landOf(TILE, TILE, () => [h, NONE, 0]);
      const file = encodeLand(land);
      expect(file.h).toHaveLength(1);
      expect(atob(file.h[0]).length).toBeLessThan(16);
      same(land, roundTrip(land));
    }
    // A flat sea codes as little.
    const sea = landOf(TILE, TILE, () => [-200, SEA_CM, 0]);
    expect(atob(encodeLand(sea).wl[0]).length).toBeLessThan(16);
  });

  test('a stroke codes only the tiles it touched', () => {
    const rand = mulberry32(5);
    const land = landOf(200, 140, () => [Math.floor(rand() * 3000), rand() < 0.5 ? NONE : 100, 0]);
    const all = encodeLand(land);
    const only = encodeLand(land, new Set([1, 5]));
    expect(Object.keys(only.h)).toEqual(['1', '5']);
    expect(Object.keys(only.wl)).toEqual(['1', '5']);
    expect([only.h[1], only.h[5], only.wl[1], only.wl[5]]).toEqual([all.h[1], all.h[5], all.wl[1], all.wl[5]]);
    expect(only.cover).toEqual(all.cover);
    expect(encodeTiles(land, [5, 1])).toEqual({ h: { 1: all.h[1], 5: all.h[5] }, wl: { 1: all.wl[1], 5: all.wl[5] } });
  });

  test('tilesOf: the tiles a rectangle reaches, clipped to the land across', () => {
    // 200 samples across: 4 tiles a row.
    expect(tilesOf({ i0: 0, j0: 0, i1: 1, j1: 1 }, 200)).toEqual([0]);
    expect(tilesOf({ i0: 60, j0: 60, i1: 70, j1: 70 }, 200)).toEqual([0, 1, 4, 5]);
    expect(tilesOf({ i0: 190, j0: 128, i1: 260, j1: 129 }, 200)).toEqual([10, 11]);
    expect(tilesOf({ i0: -5, j0: 0, i1: 64, j1: 64 }, 200)).toEqual([0]);
  });
});

describe('reading', () => {
  test('heightAt is bilinear between samples (m) and holds the edge past it', () => {
    // 3 × 2 samples every 2 m: heights (cm) 0 100 200 / 300 400 500.
    const land = landOf(3, 2, (k) => [k * 100, NONE, 0]);
    expect(heightAt(land, 0, 0)).toBe(0);
    expect(heightAt(land, 2, 0)).toBe(1);
    expect(heightAt(land, 1, 0)).toBeCloseTo(0.5, 9);
    expect(heightAt(land, 1, 1)).toBeCloseTo(2, 9);
    expect(heightAt(land, 3, 0.5)).toBeCloseTo(1.5 + 0.75, 9);
    // Past the edge: the edge's height.
    expect(heightAt(land, -10, -10)).toBe(0);
    expect(heightAt(land, 50, 0)).toBe(2);
    expect(heightAt(land, 50, 50)).toBe(5);
    expect(heightAt(land, 1, 50)).toBeCloseTo(3.5, 9);
    // One sample: flat everywhere.
    expect(heightAt(landOf(1, 1, () => [250, NONE, 0]), 7, -3)).toBe(2.5);
  });

  test('wetAt: water standing over the ground, nothing off the land', () => {
    const land = landOf(2, 2, (k) => [[0, 0, 100, 100][k], [NONE, 50, 50, 100][k], 0]);
    expect([wetAt(land, 0, 0), wetAt(land, 1, 0), wetAt(land, 0, 1), wetAt(land, 1, 1)]).toEqual([false, true, false, false]);
    expect(wetAt(land, -1, 0) || wetAt(land, 2, 0) || wetAt(land, 0, 2)).toBe(false);
  });

  test('blankLand: flat, dry and open over the world', () => {
    const land = blankLand(1024, 900);
    expect([land.cols, land.rows]).toEqual([512, 450]);
    expect(land.h.every((v) => v === 0) && land.wl.every((v) => v === NONE)).toBe(true);
    expect(land.cover.length).toBe(256 * 225);
    expect(land.edited).toBe(false);
  });
});

test('a 1024 m land of hills, a town and a lake: its coded size', () => {
  const W = 1024;
  const land = blankLand(W, W);
  const smooth = (a: number, b: number, v: number) => Math.min(1, Math.max(0, (v - a) / (b - a))) ** 2 * (3 - 2 * Math.min(1, Math.max(0, (v - a) / (b - a))));
  for (let j = 0; j < land.rows; j++) {
    for (let i = 0; i < land.cols; i++) {
      const [x, z] = [i * 2, j * 2];
      // Flat where the town is built, hills rising from it to 60 m and a rim to 140 m, a lake to the north-east.
      const town = Math.hypot(x - 420, z - 560);
      const rise = smooth(160, 320, town);
      let h = rise * (fbm(x * 0.006, z * 0.006, 9) * 60 + fbm(x * 0.03, z * 0.03, 10, 3) * 4) + smooth(380, 512, Math.max(Math.abs(x - 512), Math.abs(z - 512))) * 140;
      const lake = Math.hypot(x - 760, z - 260) - 90 * (1 + 0.2 * Math.sin(Math.atan2(z - 260, x - 760) * 3));
      if (lake < 0) {
        h = -0.35 - Math.min(4, -lake * 0.22 + lake * lake * 0.004);
        land.wl[j * land.cols + i] = SEA_CM;
      } else h = Math.max(h, 0.3 * smooth(0, 12, lake));
      land.h[j * land.cols + i] = Math.round(h * 100);
    }
  }
  for (let k = 0; k < land.cover.length; k++) {
    const [x, z] = [(k % 256) * 4, Math.floor(k / 256) * 4];
    const n = fbm(x * 0.01, z * 0.01, 21, 3);
    land.cover[k] = Math.hypot(x - 420, z - 560) < 200 ? COVER.open : n > 0.62 ? COVER.woods : n < 0.38 ? COVER.meadow : n < 0.42 ? COVER.fields : COVER.open;
  }
  const file = encodeLand(land);
  const bytes = (list: string[]) => list.reduce((n, t) => n + t.length, 0);
  const json = JSON.stringify(file).length;
  console.log(`1024 m land: ${(json / 1024).toFixed(1)} KB in JSON (heights ${(bytes(file.h) / 1024).toFixed(1)} KB, water ${(bytes(file.wl) / 1024).toFixed(1)} KB in ${file.wl.filter(Boolean).length} tiles, cover ${file.cover.length} numbers)`);
  same(land, decodeLand(file));
  // (The plan's budget for `meta.land`: 200 to 400 KB.)
  expect(json).toBeLessThan(600 * 1024);
});
