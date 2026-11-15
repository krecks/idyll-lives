// A `shapeLand` stroke merged into the page's copy of `meta` (`mergeShapeLand`) as the simulation merges it into its
// own (sim-core `land.rs`, tests/sim/shape_land.rs): the land's tiles by index over the old ones, the rest of the land
// and the zones replaced, a new `meta` with the old one left as it was; with the land's decoder reading the merged tiles.
// `bun test tests/shapeLandMeta.test.ts`.
import { describe, expect, test } from 'bun:test';
import { mergeShapeLand, type WorldMeta } from '../src/core/protocol';
import { blankLand, decodeLand, encodeTiles } from '../src/game/land';

describe('mergeShapeLand', () => {
  test('the first stroke makes the land; a later one merges its tiles and replaces the zones', () => {
    const meta: WorldMeta = { name: 'Elmbury' };
    const header = { v: 1 as const, cell: 2 as const, cols: 200, rows: 100, tile: 64 as const, cover: [5000, 0], seeded: 'drawn' as const };
    const first = mergeShapeLand(meta, { land: { ...header, h: { 0: 'AAA', 5: 'BBB' }, wl: { 5: 'WWW' } }, zones: [] })!;
    expect(first.land!.h).toEqual(['AAA', '', '', '', '', 'BBB', '', '']);
    expect(first.land!.wl).toEqual(['', '', '', '', '', 'WWW', '', '']);
    expect(first.name).toBe('Elmbury');
    expect(meta.land).toBeUndefined();
    const second = mergeShapeLand(first, { land: { h: { 5: 'CCC', 7: 'DDD' }, wl: { 5: '' }, cover: [5000, 1] } })!;
    expect(second.land!.h).toEqual(['AAA', '', '', '', '', 'CCC', '', 'DDD']);
    expect(second.land!.wl.every((c) => c === '')).toBe(true);
    expect(second.land!.cover).toEqual([5000, 1]);
    expect(second.zones).toEqual([]);
    expect(first.land!.h[5]).toBe('BBB');
  });

  test('tiles coded by encodeTiles decode from the merged land', () => {
    const land = blankLand(256, 256);
    land.h[land.cols * 70 + 40] = 1234;
    const header = { v: 1 as const, cell: 2 as const, cols: land.cols, rows: land.rows, tile: 64 as const, cover: [land.cover.length, 0], seeded: land.seeded };
    const merged = mergeShapeLand(null, { land: { ...header, ...encodeTiles(land, [2]) } })!;
    expect(decodeLand(merged.land!).h[land.cols * 70 + 40]).toBe(1234);
  });

  test('nothing to merge leaves meta as it is', () => {
    const meta: WorldMeta = { name: 'Elmbury' };
    expect(mergeShapeLand(meta, {})).toBe(meta);
  });
});
